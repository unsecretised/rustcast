//! This has all the utility functions that rustcast uses
use std::{
    fs::{self},
    io,
    path::{Path, PathBuf},
    thread,
};

#[cfg(target_os = "macos")]
use icns::IconFamily;
use tracing::instrument;

#[cfg(target_os = "macos")]
use {
    crate::cross_platform::macos::get_installed_macos_apps, objc2_app_kit::NSWorkspace,
    objc2_foundation::NSURL, std::os::unix::fs::PermissionsExt,
};

#[cfg(target_os = "windows")]
use std::process::Command;

use crate::{
    app::apps::{App, AppCommand},
    commands::Function,
};

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
            Command::new("xdg-open").arg(path).status().ok();
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
    tracing::debug!("Exclude patterns: {:?}", &config.index_exclude_patterns);
    tracing::debug!("Include patterns: {:?}", &config.index_include_patterns);

    #[cfg(target_os = "macos")]
    {
        let start = time::Instant::now();
        
        let res = get_installed_macos_apps(config);

        let end = time::Instant::now();


        tracing::info!("Finished indexing apps (t = {}s)", (end - start).as_secs_f32());


        res
    }

    #[cfg(target_os = "windows")]
    {
        use std::time;
        use crate::cross_platform::windows::app_finding::get_installed_windows_apps;

        let start = time::Instant::now();

        let res = get_installed_windows_apps(
            &config.index_exclude_patterns,
            &config.index_include_patterns,
        );

        let end = time::Instant::now();

        tracing::info!("Finished indexing apps (t = {}s)", (end - start).as_secs_f32());

        res
    }
}
