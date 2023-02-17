use std::string::ToString;
use color_eyre::eyre::Result;
use minreq::Error;
use once_cell::sync::Lazy;
use versions::Versioning;

use crate::build_time::BUILD_TIME;
use crate::cli::command::Command;
use crate::config::Config;
use crate::output::Output;

#[derive(Debug, clap::Args)]
#[clap(about = "Show rtx version", alias = "v")]
pub struct Version {}

pub static OS: Lazy<String> = Lazy::new(|| std::env::consts::OS.into());
pub static ARCH: Lazy<String> = Lazy::new(|| {
    match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        _ => std::env::consts::ARCH,
    }
    .to_string()
});

pub static VERSION: Lazy<String> = Lazy::new(|| {
    format!(
        "{} {}-{} (built {})",
        if cfg!(debug_assertions) {
            format!("{}-DEBUG", env!("CARGO_PKG_VERSION"))
        } else {
            env!("CARGO_PKG_VERSION").to_string()
        },
        *OS,
        *ARCH,
        BUILD_TIME.format("%Y-%m-%d"),
    )
});

const LATEST_VERSION_UNKNOWN: &str = "0.0.0";

impl Command for Version {
    fn run(self, _config: Config, out: &mut Output) -> Result<()> {
        show_version(out);
        Ok(())
    }
}

pub fn print_version_if_requested(args: &[String], out: &mut Output) {
    if args.len() == 2 {
        let cmd = &args[1].to_lowercase();
        if cmd == "version" || cmd == "-v" || cmd == "--version" {
            show_latest();
            show_version(out);
            std::process::exit(0);
        }
    }
}

fn show_version(out: &mut Output) {
    rtxprintln!(out, "{}", *VERSION);
}

fn show_latest() {
    if let Some(latest) = check_for_new_version() {
        warn!("rtx version {} available", latest)
    }
}

pub fn check_for_new_version() -> Option<String> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let current = Versioning::new(current.as_str()).unwrap();

    let latest = get_latest_version().unwrap_or(LATEST_VERSION_UNKNOWN.to_string());
    let latest = Versioning::new(latest.as_str()).unwrap();

    if current < latest {
        return Some(latest.to_string());
    } else {
        return None;
    }
}

fn get_latest_version() -> Result<String, Error> {
    let response = minreq::get("https://rtx.jdxcode.com/VERSION")
        .with_timeout(1)
        .send()?;
    match response.status_code {
        200 => Ok(response.as_str()?.trim().to_string()),
        _ => Ok(LATEST_VERSION_UNKNOWN.into()),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_str_eq;

    use crate::assert_cli;

    use super::*;

    #[test]
    fn test_version() {
        let stdout = assert_cli!("version");
        assert_str_eq!(stdout, VERSION.to_string() + "\n");
    }
}
