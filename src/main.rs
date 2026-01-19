mod app;
mod calculator;
mod clipboard;
mod commands;
mod config;
mod haptics;
mod macos;
mod styles;
mod unit_conversion;
mod utils;

use std::path::Path;

use crate::{
    app::tile::{self, Tile},
    config::Config,
};

use global_hotkey::GlobalHotKeyManager;

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

    let show_hide = config.toggle_hotkey.parse().unwrap();

    // Hotkeys are stored as a vec so that hyperkey support can be added later
    let hotkeys = vec![show_hide];

    manager
        .register_all(&hotkeys)
        .expect("Unable to register hotkey");

    iced::daemon(
        move || tile::elm::new(show_hide, &config),
        tile::update::handle_update,
        tile::elm::view,
    )
    .subscription(Tile::subscription)
    .theme(Tile::theme)
    .run()
}
