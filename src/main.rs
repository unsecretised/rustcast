mod app;
mod calculator;
mod clipboard;
mod commands;
mod config;
mod macos;
mod utils;

use std::path::Path;

use crate::{app::tile::Tile, config::Config};

use global_hotkey::{
    GlobalHotKeyManager,
    hotkey::{HotKey, Modifiers},
};

fn main() -> iced::Result {
    #[cfg(target_os = "macos")]
    {
        macos::set_activation_policy_accessory();
    }

    let home = std::env::var("HOME").unwrap();

    let file_path = home.clone() + "/.config/rustcast/config.toml";
    if !Path::new(&file_path).exists() {
        std::fs::create_dir_all(home + "/.config/rustcast").unwrap();
        std::fs::write(
            &file_path,
            toml::to_string(&Config::default()).unwrap_or_else(|x| x.to_string()),
        )
        .unwrap();
    }
    let config: Config = match std::fs::read_to_string(&file_path) {
        Ok(a) => toml::from_str(&a).unwrap_or(Config::default()),
        Err(_) => Config::default(),
    };

    let manager = GlobalHotKeyManager::new().unwrap();

    let show_hide = HotKey::new(
        Some(Modifiers::from_name(&config.toggle_mod).unwrap_or(Modifiers::ALT)),
        config.toggle_key,
    );

    // Hotkeys are stored as a vec so that hyperkey support can be added later
    let hotkeys = vec![show_hide];

    manager
        .register_all(&hotkeys)
        .expect("Unable to register hotkey");

    iced::daemon(
        move || Tile::new(show_hide.id(), &config),
        Tile::update,
        Tile::view,
    )
    .subscription(Tile::subscription)
    .theme(Tile::theme)
    .run()
}
