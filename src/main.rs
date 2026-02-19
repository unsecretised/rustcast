mod app;
mod calculator;
mod clipboard;
mod commands;
mod config;
mod icon;
mod styles;
mod unit_conversion;
mod utils;

mod cross_platform;

use std::env::temp_dir;
use std::fs::{File, create_dir_all};
use std::io;

// import from utils
use crate::utils::{get_config_file_path, get_config_installation_dir, read_config_file};

use crate::app::tile::{self, Tile};

#[cfg(not(target_os = "linux"))]
use global_hotkey::GlobalHotKeyManager;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;

#[cfg(target_os = "linux")]
const SOCKET_PATH: &str = "/tmp/rustcast.sock";

fn main() -> iced::Result {
    #[cfg(target_os = "macos")]
    cross_platform::macos::set_activation_policy_accessory();

    let config_dir = get_config_installation_dir();
    if let Err(e) = std::fs::metadata(config_dir.join("rustcast/")) {
        if e.kind() == io::ErrorKind::NotFound {
            let result = create_dir_all(config_dir.join("rustcast/"));

            if let Err(e) = result {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        } else {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }

    let file_path = get_config_file_path();
    let config = read_config_file(&file_path);
    if let Err(e) = config {
        // Tracing isn't inited yet
        eprintln!("Error parsing config: {}", e);
        std::process::exit(1);
    }

    let config = config.unwrap();

    {
        let temp_dir = temp_dir().join("rustcast");
        let log_path = temp_dir.join("log.log");
        let vv_log_path = temp_dir.join("vv_log.log");
        if !temp_dir.exists() {
            std::fs::create_dir_all(temp_dir).unwrap();
        }

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

    #[cfg(target_os = "linux")]
    {
        // error handling should really be improved soon (tm)
        use std::fs;
        use std::os::unix::net::UnixListener;
        use std::{io::Write, os::unix::net::UnixStream};
        use tracing::info;

        if UnixListener::bind(SOCKET_PATH).is_err() {
            match UnixStream::connect(SOCKET_PATH) {
                Ok(mut stream) => {
                    use std::env;

                    let clipboard = env::args().any(|arg| arg.trim() == "--cphist");
                    let cmd = if clipboard { "clipboard" } else { "toggle" };
                    info!("socket sending: {cmd}");
                    let _ = stream.write_all(cmd.as_bytes());
                    std::process::exit(0);
                }
                Err(_) => {
                    let _ = fs::remove_file(SOCKET_PATH);
                }
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    let show_hide_bind = {
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

        show_hide
    };

    tracing::info!("Starting.");

    iced::daemon(
        move || {
            tile::elm::new(
                #[cfg(not(target_os = "linux"))]
                show_hide_bind,
                &config,
            )
        },
        tile::update::handle_update,
        tile::elm::view,
    )
    .subscription(Tile::subscription)
    .theme(Tile::theme)
    .run()
}
