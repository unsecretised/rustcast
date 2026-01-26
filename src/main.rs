mod app;
mod calculator;
mod clipboard;
mod commands;
mod config;
mod styles;
mod unit_conversion;
mod utils;

mod cross_platform;

use std::env::temp_dir;
use std::fs::File;

// import from utils
use crate::utils::{create_config_file_if_not_exists, get_config_file_path, read_config_file};

use crate::app::tile::{self, Tile};

use global_hotkey::GlobalHotKeyManager;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;

fn main() -> iced::Result {
    #[cfg(target_os = "macos")]
    cross_platform::macos::set_activation_policy_accessory();

    let file_path = get_config_file_path();
    let config = read_config_file(&file_path).unwrap();
    create_config_file_if_not_exists(&file_path, &config).unwrap();

    {
        let log_path = temp_dir().join("rustcast/log.log");
        let vv_log_path = temp_dir().join("rustcast/vv_log.log");

        create_config_file_if_not_exists(&log_path, &config).unwrap();

        let file = File::create(&log_path).expect("Failed to create logfile");
        let vv_file = File::create(&vv_log_path).expect("Failed to create logfile");

        let log_file = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(file)
            .with_filter(LevelFilter::DEBUG);
        let vv_log_file = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(vv_file);
        let console_out = tracing_subscriber::fmt::layer().with_filter(LevelFilter::INFO);

        let subscriber = tracing_subscriber::registry()
            .with(log_file)
            .with(vv_log_file)
            .with(console_out);

        tracing::subscriber::set_global_default(subscriber).expect("Error initing tracing");

        tracing::info!("Main log file at    : {}", &vv_log_path.display());
        tracing::info!("Verbose log file at : {}", &log_path.display());
        tracing::info!("Config file at      : {}", &file_path.display());
    }

    tracing::debug!("Loaded config data: {:#?}", &config);

    let manager = GlobalHotKeyManager::new().unwrap();

    let show_hide = config.toggle_hotkey.parse().unwrap();

    let mut hotkeys = vec![show_hide];

    if let Some(show_clipboard) = &config.clipboard_hotkey
        && let Some(cb_page_hk) = show_clipboard.parse().ok()
    {
        hotkeys.push(cb_page_hk);
    }

    let result = manager.register_all(&hotkeys);

    if let Err(global_hotkey::Error::AlreadyRegistered(key)) = result {
        if key == show_hide {
            // It probably should give up here.
            panic!("Couldn't register the key to open ({})", key)
        } else {
            tracing::warn!("Couldn't register hotkey {}", key)
        }
    } else if let Err(e) = result {
        tracing::error!("{}", e.to_string());
    }

    tracing::info!("Starting.");

    iced::daemon(
        move || tile::elm::new(show_hide, &config),
        tile::update::handle_update,
        tile::elm::view,
    )
    .subscription(Tile::subscription)
    .theme(Tile::theme)
    .run()
}
