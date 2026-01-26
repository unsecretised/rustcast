mod app;
mod calculator;
mod clipboard;
mod commands;
mod config;
mod platform;
mod styles;
mod unit_conversion;
mod utils;

use std::path::Path;

use crate::{
    app::tile::{self, Tile},
    config::Config,
};

use global_hotkey::GlobalHotKeyManager;

use self::platform::set_activation_policy_accessory;

fn main() -> iced::Result {
    set_activation_policy_accessory();

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

    let mut hotkeys = vec![show_hide];

    if let Some(show_clipboard) = &config.clipboard_hotkey
        && let Some(cb_page_hk) = show_clipboard.parse().ok()
    {
        hotkeys.push(cb_page_hk);
    }

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
