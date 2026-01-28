//! This has all the utility functions that rustcast uses
#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;
use std::{
    fs::{self},
    path::{Path, PathBuf},
    thread,
};

#[cfg(target_os = "macos")]
use iced::widget::image::Handle;
#[cfg(target_os = "macos")]
use icns::IconFamily;

#[cfg(target_os = "macos")]
use {
    crate::cross_platform::macos::get_installed_macos_apps, objc2_app_kit::NSWorkspace,
    objc2_foundation::NSURL, std::os::unix::fs::PermissionsExt,
};

#[cfg(target_os = "windows")]
use crate::cross_platform::windows::get_installed_windows_apps;

#[cfg(target_os = "linux")]
use crate::cross_platform::linux::get_installed_linux_apps;

#[cfg(any(target_os = "windows", target_os = "linux"))]
use std::process::Command;

use crate::{
    app::apps::{App, AppCommand},
    commands::Function,
};

/// This converts an icns file to an iced image handle
#[cfg(target_os = "macos")]
pub(crate) fn handle_from_icns(path: &Path) -> Option<Handle> {
    use image::RgbaImage;

    let data = std::fs::read(path).ok()?;
    let family = IconFamily::read(std::io::Cursor::new(&data)).ok()?;

    let icon_type = family.available_icons();

    let icon = family.get_icon_with_type(*icon_type.first()?).ok()?;
    let image = RgbaImage::from_raw(
        icon.width() as u32,
        icon.height() as u32,
        icon.data().to_vec(),
    )?;
    Some(Handle::from_rgba(
        image.width(),
        image.height(),
        image.into_raw(),
    ))
}

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

pub fn read_config_file(file_path: &Path) -> Result<Config, std::io::Error> {
    let config: Config = match std::fs::read_to_string(file_path) {
        Ok(a) => toml::from_str(&a).unwrap(),
        Err(_) => Config::default(),
    };

    Ok(config)
}

pub fn create_config_file_if_not_exists(
    file_path: &Path,
    config: &Config,
) -> Result<(), std::io::Error> {
    // check if file exists
    if let Ok(exists) = std::fs::metadata(file_path)
        && exists.is_file()
    {
        return Ok(());
    }

    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }

    std::fs::write(
        file_path,
        toml::to_string(&config).unwrap_or_else(|x| x.to_string()),
    )
    .unwrap();

    Ok(())
}

// TODO: this should also work with args
pub fn open_application(path: &str) {
    let path_string = path.to_string();
    thread::spawn(move || {
        let path = &path_string;
        #[cfg(target_os = "windows")]
        {
            println!("Opening application: {}", path);

            Command::new("powershell")
                .arg(format!("Start-Process '{}'", path))
                .status()
                .ok();
        }

        #[cfg(target_os = "macos")]
        {
            NSWorkspace::new().openURL(&NSURL::fileURLWithPath(
                &objc2_foundation::NSString::from_str(path),
            ));
        }

        #[cfg(target_os = "linux")]
        {
            Command::new(path).status().ok();
        }
    });
}

#[allow(unused)]
pub fn index_dirs_from_config(apps: &mut Vec<App>) -> bool {
    let path = get_config_file_path();
    let config = read_config_file(path.as_path());

    // if config is not valid return false otherwise unwrap config so it is usable
    let config = match config {
        Ok(config) => config,
        Err(err) => {
            println!("Error reading config file: {}", err);
            return false;
        }
    };

    if config.index_dirs.is_empty() {
        return false;
    }

    config.index_dirs.clone().iter().for_each(|dir| {
        // check if dir exists
        if !Path::new(dir).exists() {
            println!("Directory {} does not exist", dir);
            return;
        }

        let paths = fs::read_dir(dir).unwrap();

        for path in paths {
            let path = path.unwrap().path();
            let metadata = fs::metadata(&path).unwrap();

            #[cfg(target_os = "windows")]
            let is_executable =
                metadata.is_file() && path.extension().and_then(|s| s.to_str()) == Some("exe");

            #[cfg(target_os = "macos")]
            let is_executable = {
                (metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0))
                    || path.extension().and_then(|s| s.to_str()) == Some("app")
            };

            #[cfg(target_os = "linux")]
            let is_executable = metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0);

            if is_executable {
                let display_name = path.file_name().unwrap().to_string_lossy().to_string();
                apps.push(App {
                    open_command: AppCommand::Function(Function::OpenApp(
                        path.to_string_lossy().to_string(),
                    )),
                    name: display_name.clone(),
                    desc: "Application".to_string(),
                    name_lc: display_name.clone().to_lowercase(),
                    icons: None,
                });
            }
        }
    });

    true
}

/// Use this to get installed apps
pub fn get_installed_apps(config: &Config) -> Vec<App> {
    tracing::debug!("Indexing installed apps");

    #[cfg(target_os = "macos")]
    {
        get_installed_macos_apps(config)
    }

    #[cfg(target_os = "windows")]
    {
        get_installed_windows_apps()
    }

    #[cfg(target_os = "linux")]
    {
        get_installed_linux_apps(config)
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

    let tld = match parts.next_back() {
        Some(p) => p,
        None => return false,
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
