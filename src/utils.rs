//! This has all the utility functions that rustcast uses
use std::{
    io,
    path::{Path, PathBuf},
};

#[cfg(target_os = "macos")]
use {objc2_app_kit::NSWorkspace, objc2_foundation::NSURL};

#[cfg(target_os = "linux")]
use crate::cross_platform::linux::get_installed_linux_apps;

#[cfg(any(target_os = "windows", target_os = "linux"))]
use std::process::Command;

pub fn get_config_installation_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        std::env::var("LOCALAPPDATA").unwrap().into()
    } else {
        std::env::var("HOME").unwrap().into()
    }
}

pub fn get_config_file_path() -> PathBuf {
    let home = get_config_installation_dir();

    if cfg!(target_os = "windows") {
        home.join("rustcast/config.toml")
    } else {
        home.join(".config/rustcast/config.toml")
    }
}

use crate::config::Config;

pub fn read_config_file(file_path: &Path) -> anyhow::Result<Config> {
    match std::fs::read_to_string(file_path) {
        Ok(a) => Ok(toml::from_str(&a)?),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let cfg = Config::default();
            std::fs::write(
                file_path,
                toml::to_string(&cfg).unwrap_or_else(|x| x.to_string()),
            )?;
            Ok(cfg)
        }
        Err(e) => Err(e.into()),
    }
}

// TODO: this should also work with args
pub fn open_application(path: impl AsRef<Path>) {
    let path = path.as_ref();

    #[cfg(target_os = "windows")]
    {
        println!("Opening application: {}", path.display());

        Command::new("powershell")
            .arg(format!("Start-Process '{}'", path.display()))
            .status()
            .ok();
    }

    #[cfg(target_os = "macos")]
    {
        NSWorkspace::new().openURL(&NSURL::fileURLWithPath(
            &objc2_foundation::NSString::from_str(&path.to_string_lossy()),
        ));
    }

    #[cfg(target_os = "linux")]
    {
        Command::new(path).status().ok();
    }
}

/// Check if the provided string looks like a valid url
pub fn is_url_like(s: &str) -> bool {
    if s.starts_with("http://") || s.starts_with("https://") {
        return true;
    }
    if !s.contains('.') {
        return false;
    }
    let mut parts = s.split('.');

    let Some(tld) = parts.next_back() else {
        return false;
    };

    if tld.is_empty() || tld.len() > 63 || !tld.chars().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }

    parts.all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            && !label.starts_with('-')
            && !label.ends_with('-')
    })
}
