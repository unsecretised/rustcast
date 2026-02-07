use std::{fs, path::Path};

use freedesktop_desktop_entry::DesktopEntry;
use glob::glob;
use iced::widget::image::Handle;
use image::{ImageReader, RgbaImage};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    app::{
        apps::{App, AppCommand},
        tile::elm::default_app_paths,
    },
    config::Config,
};

pub fn get_installed_linux_apps(config: &Config) -> Vec<App> {
    let paths = default_app_paths();
    let store_icons = config.theme.show_icons;

    let apps: Vec<App> = paths
        .par_iter()
        .map(|path| {
            let mut pattern = path.clone();
            if !pattern.ends_with('/') {
                pattern.push('/');
            }
            pattern.push_str("**/*.desktop");

            get_installed_apps_glob(&pattern, store_icons)
        })
        .flatten()
        .collect();

    apps
}

fn get_installed_apps_glob(pattern: &str, store_icons: bool) -> Vec<App> {
    glob(pattern)
        .unwrap()
        .flatten()
        .flat_map(|entry| get_installed_apps(entry.as_path(), store_icons))
        .collect()
}

fn get_installed_apps(path: &Path, store_icons: bool) -> Vec<App> {
    let mut apps = Vec::new();

    let Ok(content) = fs::read_to_string(path) else {
        return apps;
    };

    let Ok(de) = DesktopEntry::from_str(path, &content, None::<&[String]>) else {
        return apps;
    };

    if de.no_display() || de.hidden() {
        return apps;
    }

    let Some(name) = de.desktop_entry("Name") else {
        return apps;
    };
    let desc = de.desktop_entry("Comment").unwrap_or("");
    let Some(exec) = de.exec() else {
        return apps;
    };

    let exec = exec.to_string();
    let mut parts = exec.split_whitespace().filter(|p| !p.starts_with("%"));

    let Some(cmd) = parts.next() else {
        return apps;
    };

    let args = parts.map(str::to_owned).collect::<Vec<_>>().join(" ");

    let icon = if store_icons {
        de.icon()
            .map(str::to_owned)
            .and_then(|icon_name| find_icon_handle(&icon_name))
    } else {
        None
    };

    apps.push(App {
        icons: icon,
        name: name.to_string(),
        name_lc: name.to_lowercase(),
        desc: desc.to_string(),
        open_command: AppCommand::Function(crate::commands::Function::RunShellCommand(
            cmd.to_string(),
            args,
        )),
    });

    apps
}

pub fn handle_from_png(path: &Path) -> Option<Handle> {
    let img = ImageReader::open(path).ok()?.decode().ok()?.to_rgba8();
    let image = RgbaImage::from_raw(img.width(), img.height(), img.to_vec())?;
    Some(Handle::from_rgba(
        image.width(),
        image.height(),
        image.into_raw(),
    ))
}

fn find_icon_handle(name: &str) -> Option<Handle> {
    let paths = default_app_paths();

    for dir in paths {
        let mut pattern = dir.clone();

        if !pattern.ends_with('/') {
            pattern.push('/');
        }
        pattern.push_str(&format!("icons/**/{}*", name));

        for entry in glob(&pattern).ok()?.flatten() {
            if let Some(handle) = handle_from_png(&entry) {
                return Some(handle);
            }
        }
    }

    None
}
