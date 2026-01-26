use std::{fs, path::Path, process::Command, thread};

use glob::glob;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{app::{apps::App, tile::elm::default_app_paths}, config::Config};

pub fn get_installed_linux_apps(config: &Config) -> Vec<App> {
    let paths = default_app_paths();
    let store_icons = config.theme.show_icons;
    
    let apps: Vec<App> = paths
        .par_iter()
        .map(|path| get_installed_apps_glob(path, store_icons))
        .flatten()
        .collect();
    todo!()
    // index_dirs_from_config(&mut apps);
    //
    // apps
}

fn get_installed_apps_glob(path: &str, store_icons: bool) -> Vec<App> {
    if path.contains("*") {
        glob(path).unwrap().flatten().flat_map(|entry| {
            get_installed_apps(entry.to_str().unwrap(), store_icons)
        }).collect()
    } else {
        get_installed_apps(path, store_icons)
    }
}

fn get_installed_apps(path: &str, store_icons: bool) -> Vec<App> {
    vec![]

    // let mut apps = Vec::new();
    // let dir = Path::new(path);
    //
    // if !dir.exists() || !dir.is_dir() {
    //     return apps;
    // }
    //
    // for entry in fs::read_dir(dir).unwrap().flatten() {
    //     let path = entry.path();
    //     if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
    //         continue;
    //     }
    //
    //     let content = fs::read_to_string(&path);
    //     if content.is_err() {
    //         continue;
    //     }
    //
    // }
    //
    // todo!()
}

pub fn open_url(url: &str) {
    let url = url.to_owned();
    thread::spawn(move || {
        Command::new("xdg-open").arg(url).spawn().unwrap();
    });
}
