use std::collections::{BTreeMap, BTreeSet};
use std::io::prelude::*;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use base64::prelude::*;
use eyre::Result;
use flate2::write::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use globwalk::GlobWalkerBuilder;
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};

use crate::env::PATH_KEY;
use crate::env_diff::{EnvDiffOperation, EnvDiffPatches};
use crate::hash::hash_to_str;
use crate::shell::Shell;
use crate::{dirs, env, hooks, watch_files};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct WatchFilePattern {
    pub root: Option<PathBuf>,
    pub patterns: Vec<String>,
}

impl From<&Path> for WatchFilePattern {
    fn from(path: &Path) -> Self {
        Self {
            root: None,
            patterns: vec![path.to_string_lossy().to_string()],
        }
    }
}

/// this function will early-exit the application if hook-env is being
/// called and it does not need to be
pub fn should_exit_early(watch_files: impl IntoIterator<Item = WatchFilePattern>) -> bool {
    let args = env::ARGS.read().unwrap();
    if args.len() < 2 || args[1] != "hook-env" {
        return false;
    }
    if dir_change().is_some() {
        hooks::schedule_hook(hooks::Hooks::Cd);
        hooks::schedule_hook(hooks::Hooks::Enter);
        hooks::schedule_hook(hooks::Hooks::Leave);
        return false;
    }
    let watch_files = get_watch_files(watch_files);
    match &*env::__MISE_WATCH {
        Some(watches) => {
            if have_files_been_modified(watches, watch_files) {
                return false;
            }
            if have_mise_env_vars_been_modified(watches) {
                return false;
            }
        }
        None => {
            return false;
        }
    };
    trace!("early-exit");
    true
}

pub fn dir_change() -> Option<(PathBuf, PathBuf)> {
    match (&*env::__MISE_DIR, &*dirs::CWD) {
        (Some(old), Some(new)) if old != new => {
            trace!("dir change: {:?} -> {:?}", old, new);
            Some((old.clone(), new.clone()))
        }
        _ => None,
    }
}

fn have_files_been_modified(watches: &HookEnvWatches, watch_files: BTreeSet<PathBuf>) -> bool {
    // make sure they have exactly the same config filenames
    let watch_keys = watches.files.keys().cloned().collect::<BTreeSet<_>>();
    if watch_keys != watch_files {
        trace!(
            "config files do not match {:?}",
            watch_keys.symmetric_difference(&watch_files)
        );
        return true;
    }

    // check the files to see if they've been altered
    let mut modified = false;
    for (fp, prev_modtime) in &watches.files {
        if let Ok(modtime) = fp
            .metadata()
            .expect("accessing config file modtime")
            .modified()
        {
            let modtime = modtime
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if modtime != *prev_modtime {
                trace!("file modified: {:?}", fp);
                modified = true;
                watch_files::add_modified_file(fp.clone());
            }
        }
    }
    if !modified {
        trace!("config files unmodified");
    }
    modified
}

fn have_mise_env_vars_been_modified(watches: &HookEnvWatches) -> bool {
    if get_mise_env_vars_hashed() != watches.env_var_hash {
        return true;
    }
    false
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HookEnvWatches {
    files: BTreeMap<PathBuf, u64>,
    env_var_hash: String,
}

pub fn serialize_watches(watches: &HookEnvWatches) -> Result<String> {
    let mut gz = ZlibEncoder::new(Vec::new(), Compression::fast());
    gz.write_all(&rmp_serde::to_vec_named(watches)?)?;
    Ok(BASE64_STANDARD_NO_PAD.encode(gz.finish()?))
}

pub fn deserialize_watches(raw: String) -> Result<HookEnvWatches> {
    let mut writer = Vec::new();
    let mut decoder = ZlibDecoder::new(writer);
    let bytes = BASE64_STANDARD_NO_PAD.decode(raw)?;
    decoder.write_all(&bytes[..])?;
    writer = decoder.finish()?;
    Ok(rmp_serde::from_slice(&writer[..])?)
}

pub fn build_watches(
    watch_files: impl IntoIterator<Item = WatchFilePattern>,
) -> Result<HookEnvWatches> {
    let mut watches = BTreeMap::new();
    for cf in get_watch_files(watch_files) {
        watches.insert(
            cf.clone(),
            cf.metadata()?
                .modified()?
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
        );
    }

    Ok(HookEnvWatches {
        files: watches,
        env_var_hash: get_mise_env_vars_hashed(),
    })
}

pub fn get_watch_files(
    watch_files: impl IntoIterator<Item = WatchFilePattern>,
) -> BTreeSet<PathBuf> {
    let mut watches = BTreeSet::new();
    if dirs::DATA.exists() {
        watches.insert(dirs::DATA.to_path_buf());
    }
    for (root, patterns) in &watch_files.into_iter().chunk_by(|wfp| wfp.root.clone()) {
        if let Some(root) = root {
            let patterns = patterns.flat_map(|wfp| wfp.patterns).collect::<Vec<_>>();
            let glob = GlobWalkerBuilder::from_patterns(root, &patterns)
                .follow_links(true)
                .build();
            if let Ok(glob) = glob {
                for entry in glob.flatten() {
                    watches.insert(entry.path().to_path_buf());
                }
            }
        } else {
            watches.extend(patterns.flat_map(|wfp| wfp.patterns).map(PathBuf::from));
        }
    }

    watches
}

/// gets a hash of all MISE_ environment variables
fn get_mise_env_vars_hashed() -> String {
    let env_vars: Vec<(&String, &String)> = env::PRISTINE_ENV
        .deref()
        .iter()
        .filter(|(k, _)| k.starts_with("MISE_"))
        .sorted()
        .collect();
    hash_to_str(&env_vars)
}

pub fn clear_old_env(shell: &dyn Shell) -> String {
    let mut patches = env::__MISE_DIFF.reverse().to_patches();
    if let Some(path) = env::PRISTINE_ENV.deref().get(&*PATH_KEY) {
        patches.push(EnvDiffOperation::Change(
            PATH_KEY.to_string(),
            path.to_string(),
        ));
    }
    build_env_commands(shell, &patches)
}

pub fn build_env_commands(shell: &dyn Shell, patches: &EnvDiffPatches) -> String {
    let mut output = String::new();

    for patch in patches.iter() {
        match patch {
            EnvDiffOperation::Add(k, v) | EnvDiffOperation::Change(k, v) => {
                output.push_str(&shell.set_env(k, v));
            }
            EnvDiffOperation::Remove(k) => {
                output.push_str(&shell.unset_env(k));
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use std::time::UNIX_EPOCH;

    use pretty_assertions::{assert_eq, assert_str_eq};
    use test_log::test;

    use super::*;

    #[test]
    fn test_have_config_files_been_modified() {
        let files = BTreeSet::new();
        let watches = HookEnvWatches {
            files: BTreeMap::new(),
            env_var_hash: "".into(),
        };
        assert!(!have_files_been_modified(&watches, files));

        let fp = env::current_dir().unwrap().join(".test-tool-versions");
        let watches = HookEnvWatches {
            files: BTreeMap::from([(
                fp.clone(),
                UNIX_EPOCH
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            )]),
            env_var_hash: "".into(),
        };
        let files = BTreeSet::from([fp.clone()]);
        assert!(have_files_been_modified(&watches, files));

        let modtime = fp.metadata().unwrap().modified().unwrap();
        let watches = HookEnvWatches {
            files: BTreeMap::from([(
                fp.clone(),
                modtime
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            )]),
            env_var_hash: "".into(),
        };
        let files = BTreeSet::from([fp]);
        assert!(!have_files_been_modified(&watches, files));
    }

    #[test]
    fn test_serialize_watches_empty() {
        let watches = HookEnvWatches {
            files: BTreeMap::new(),
            env_var_hash: "".into(),
        };
        let serialized = serialize_watches(&watches).unwrap();
        let deserialized = deserialize_watches(serialized).unwrap();
        assert_eq!(deserialized.files.len(), 0);
    }

    #[test]
    fn test_serialize_watches() {
        let serialized = serialize_watches(&HookEnvWatches {
            files: BTreeMap::from([(
                PathBuf::from("foo"),
                UNIX_EPOCH
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            )]),
            env_var_hash: "testing-123".into(),
        })
        .unwrap();
        let deserialized = deserialize_watches(serialized).unwrap();
        assert_eq!(deserialized.files.len(), 1);
        assert_str_eq!(deserialized.env_var_hash, "testing-123");
        assert_eq!(
            *deserialized
                .files
                .get(PathBuf::from("foo").as_path())
                .unwrap(),
            UNIX_EPOCH
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
    }
}
