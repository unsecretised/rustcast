mod app;
mod commands;
mod config;
mod macos;
mod utils;
mod windows;

// import from utils
use crate::utils::{create_config_file_if_not_exists, get_config_file_path, read_config_file};

use crate::{app::Tile, utils::to_key_code};

use global_hotkey::{
    GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};

fn main() -> iced::Result {
    #[cfg(target_os = "macos")]
    {
        macos::set_activation_policy_accessory();
    }

    let file_path = get_config_file_path();
    let config = read_config_file(&file_path).unwrap();
    create_config_file_if_not_exists(&file_path, &config).unwrap();

    let manager = GlobalHotKeyManager::new().unwrap();

    let show_hide = HotKey::new(
        Some(Modifiers::from_name(&config.toggle_mod).unwrap_or(Modifiers::ALT)),
        to_key_code(&config.toggle_key).unwrap_or(Code::Space),
    );

    manager
        .register_all(&[show_hide])
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
