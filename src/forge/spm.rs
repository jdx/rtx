use std::env::temp_dir;
use std::fmt::Debug;
use std::path::Path;
use std::str::FromStr;

use git2::Repository;
use url::Url;

use crate::cache::CacheManager;
use crate::cli::args::ForgeArg;
use crate::cmd::CmdLineRunner;
use crate::config::Settings;
use crate::forge::{Forge, ForgeType};
use crate::{file, github};

#[derive(Debug)]
pub struct SpmForge {
    fa: ForgeArg,
    remote_version_cache: CacheManager<Vec<String>>,
}

// https://github.com/apple/swift-package-manager
impl Forge for SpmForge {
    fn get_type(&self) -> ForgeType {
        ForgeType::Spm
    }

    fn fa(&self) -> &ForgeArg {
        &self.fa
    }

    fn get_dependencies(&self, _tvr: &crate::toolset::ToolRequest) -> eyre::Result<Vec<ForgeArg>> {
        // TODO: swift as dependencies (wait for swift core plugin: https://github.com/jdx/mise/pull/1708)
        Ok(vec![])
    }

    fn _list_remote_versions(&self) -> eyre::Result<Vec<String>> {
        self.remote_version_cache
            .get_or_try_init(|| {
                Ok(github::list_releases(self.name())?
                    .into_iter()
                    .map(|r| r.tag_name)
                    .rev()
                    .collect())
            })
            .cloned()
    }

    fn install_version_impl(
        &self,
        ctx: &crate::install_context::InstallContext,
    ) -> eyre::Result<()> {
        let settings = Settings::get();
        settings.ensure_experimental("spm backend")?;

        //
        // 1. Checkout the swift package repo:
        // - name could be github repo shorthand or full url
        // - if full url, clone it
        // - if shorthand, convert to full url and clone it
        //
        // - version is a release tag
        // - if version not specified ("latest"), get last release tag
        // - if there are no release tags, get error
        //
        let repo_url = SwiftPackageRepo::from_str(self.name())?.0;
        let version = if ctx.tv.version == "latest" {
            self.latest_stable_version()?
                .ok_or_else(|| eyre::eyre!("No stable versions found"))?
        } else {
            ctx.tv.version.clone()
        };

        // TODO: should we use cache for this (cache | downloads)? Now i think it's not necessary
        let tmp_repo_dir = temp_dir()
            .join("spm")
            .join(self.filename_safe_url(&repo_url) + "@" + &version);
        file::remove_all(&tmp_repo_dir)?;
        file::create_dir_all(tmp_repo_dir.parent().unwrap())?;

        debug!(
            "Cloning swift package repo: {}, tag: {}, path: {}",
            repo_url,
            version,
            tmp_repo_dir.display()
        );
        // TODO: use project git module (now it doesn't support checkout by tag)
        let repo = Repository::clone(&repo_url, &tmp_repo_dir)?;
        let (object, reference) = repo.revparse_ext(&version)?;
        repo.checkout_tree(&object, None)?;
        repo.set_head(reference.unwrap().name().unwrap())?;

        //
        // 2. Build the swift package
        // - TODO: validate if package have executables
        // - TODO: should specify concrete product (executable) or `swift build` compile all of them?
        // - TODO: specify arch, platform, target?
        //
        debug!("Building swift package");
        let build_cmd = CmdLineRunner::new("swift")
            .arg("build")
            .arg("--configuration")
            .arg("release")
            .arg("--package-path")
            .arg(&tmp_repo_dir);
        build_cmd.execute()?;
        let bin_path = cmd!(
            "swift",
            "build",
            "--configuration",
            "release",
            "--package-path",
            &tmp_repo_dir,
            "--show-bin-path"
        )
        .read()?;

        //
        // 3. Copy all binaries to the install path
        // - TODO: copy resources and other related files
        //
        let install_bin_path = ctx.tv.install_path().join("bin");
        debug!(
            "Copying binaries to install path: {}",
            install_bin_path.display()
        );
        file::create_dir_all(&install_bin_path)?;
        let files = file::ls(Path::new(&bin_path))?;
        for file in files {
            // TODO: check if file is executable
            file::copy(&file, &install_bin_path.join(file.file_name().unwrap()))?;
        }

        debug!("Cleaning up temporary files");
        file::remove_all(&tmp_repo_dir)?;

        Ok(())
    }
}

impl SpmForge {
    pub fn new(name: String) -> Self {
        let fa = ForgeArg::new(ForgeType::Spm, &name);
        Self {
            remote_version_cache: CacheManager::new(
                fa.cache_path.join("remote_versions-$KEY.msgpack.z"),
            ),
            fa,
        }
    }

    fn filename_safe_url(&self, url: &str) -> String {
        url.replace("://", "_")
            .replace("/", "_")
            .replace("?", "_")
            .replace("&", "_")
            .replace(":", "_")
    }
}

/// https://github.com/owner/repo.git
struct SwiftPackageRepo(String);

impl FromStr for SwiftPackageRepo {
    type Err = eyre::Error;

    // swift package github repo shorthand:
    // - owner/repo
    //
    // swift package github repo full url:
    // - https://github.com/owner/repo.git
    // TODO: support more type of git urls
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(s);
        if url.is_ok()
            && url.as_ref().unwrap().host_str() == Some("github.com")
            && url.as_ref().unwrap().path().ends_with(".git")
        {
            Ok(Self(s.to_string()))
        } else if regex!(r"^[a-zA-Z0-9_-]+/[a-zA-Z0-9_-]+$").is_match(s) {
            Ok(Self(format!("https://github.com/{}.git", s.to_string())))
        } else {
            Err(eyre::eyre!("Invalid swift package repo: {}", s))
        }
    }
}
