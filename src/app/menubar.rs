//! This has the menubar icon logic for the app

use std::{collections::HashMap, io::Cursor};

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use image::{DynamicImage, ImageReader};
use log::info;
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{
        AboutMetadataBuilder, Icon as Ico, IsMenuItem, Menu, MenuEvent, MenuItem,
        PredefinedMenuItem, Submenu, accelerator::Accelerator,
    },
};

use crate::{
    app::{Message, tile::ExtSender},
    config::Config,
    utils::{open_settings, open_url},
};

const DISCORD_LINK: &str = "https://discord.gg/bDfNYPbnC5";

use tokio::runtime::Runtime;

/// This create a new menubar icon for the app
pub fn menu_icon(config: Config, sender: ExtSender) -> TrayIcon {
    let hotkey = config.toggle_hotkey.parse::<HotKey>().unwrap();
    let builder = TrayIconBuilder::new();

    let mut modes = config.modes;
    modes.insert("Default".to_string(), "default".to_string());

    let image = get_image();
    let icon = Icon::from_rgba(image.as_bytes().to_vec(), image.width(), image.height()).unwrap();

    init_event_handler(sender, hotkey.id());

    let menu = Menu::with_items(&[
        &version_item(),
        &about_item(image),
        &open_github_item(),
        &PredefinedMenuItem::separator(),
        &refresh_item(),
        &open_item(hotkey),
        &mode_item(modes),
        &PredefinedMenuItem::separator(),
        &open_issue_item(),
        &get_help_item(),
        &PredefinedMenuItem::separator(),
        &open_settings_item(),
        &discord_item(),
        &hide_tray_icon(),
        &quit_item(),
    ])
    .unwrap();

    builder
        .with_icon(icon)
        .with_menu(Box::new(menu))
        .build()
        .unwrap()
}

fn get_image() -> DynamicImage {
    ImageReader::new(Cursor::new(include_bytes!("../../docs/icon.png")))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
}

fn init_event_handler(sender: ExtSender, hotkey_id: u32) {
    let runtime = Runtime::new().unwrap();

    MenuEvent::set_event_handler(Some(move |x: MenuEvent| {
        let sender = sender.clone();
        let sender = sender.0.clone();
        info!("Menubar event called: {}", x.id.0);
        match x.id().0.as_str() {
            "refresh_rustcast" => {
                runtime.spawn(async move {
                    sender.clone().try_send(Message::ReloadConfig).unwrap();
                });
            }
            "hide_tray_icon" => {
                runtime
                    .spawn(async move { sender.clone().try_send(Message::HideTrayIcon).unwrap() });
            }
            "open_issue_page" => {
                open_url("https://github.com/unsecretised/rustcast/issues/new");
            }
            "show_rustcast" => {
                runtime.spawn(async move {
                    sender
                        .clone()
                        .try_send(Message::KeyPressed(hotkey_id))
                        .unwrap();
                });
            }
            "open_discord" => {
                open_url(DISCORD_LINK);
            }
            "open_help_page" => {
                open_url("https://github.com/unsecretised/rustcast/discussions/new?category=q-a");
            }
            "open_preferences" => {
                open_settings();
            }
            "open_github_page" => {
                open_url("https://github.com/unsecretised/rustcast");
            }
            id => {
                if id.starts_with("mode_switch_") {
                    let id = id.to_string();
                    runtime.spawn(async move {
                        sender
                            .clone()
                            .try_send(Message::SwitchMode(
                                id.strip_prefix("mode_switch_").unwrap_or("").to_string(),
                            ))
                            .unwrap();
                    });
                }
            }
        }
    }));
}

fn version_item() -> MenuItem {
    let version = "Version: ".to_string() + option_env!("APP_VERSION").unwrap_or("Unknown");
    MenuItem::new(version, false, None)
}

fn discord_item() -> MenuItem {
    MenuItem::with_id("open_discord", "RustCast discord", true, None)
}

fn hide_tray_icon() -> MenuItem {
    MenuItem::with_id("hide_tray_icon", "Hide Tray Icon", true, None)
}

fn mode_item(modes: HashMap<String, String>) -> Submenu {
    let owned_items: Vec<MenuItem> = modes
        .keys()
        .map(|key| {
            MenuItem::with_id(
                format!("mode_switch_{}", key), // id uses the key
                format!("{}{}", key.split_at(1).0.to_uppercase(), key.split_at(1).1),
                true,
                None,
            )
        })
        .collect();

    let items: Vec<&dyn IsMenuItem> = owned_items.iter().map(|x| x as &dyn IsMenuItem).collect();

    Submenu::with_items("Modes", true, &items).unwrap()
}

fn open_item(hotkey: HotKey) -> MenuItem {
    MenuItem::with_id(
        "show_rustcast",
        "Toggle View",
        true,
        Some(Accelerator::new(Some(hotkey.mods), hotkey.key)),
    )
}

fn open_github_item() -> MenuItem {
    MenuItem::with_id("open_github_page", "Star on Github", true, None)
}

fn open_issue_item() -> MenuItem {
    MenuItem::with_id("open_issue_page", "Report an Issue", true, None)
}

fn refresh_item() -> MenuItem {
    MenuItem::with_id(
        "refresh_rustcast",
        "Refresh",
        true,
        Some(Accelerator::new(
            Some(Modifiers::SUPER),
            global_hotkey::hotkey::Code::KeyR,
        )),
    )
}

fn open_settings_item() -> MenuItem {
    MenuItem::with_id(
        "open_preferences",
        "Open Preferences",
        true,
        Some(Accelerator::new(Some(Modifiers::SUPER), Code::Comma)),
    )
}

fn get_help_item() -> MenuItem {
    MenuItem::with_id("open_help_page", "Help", true, None)
}

fn quit_item() -> PredefinedMenuItem {
    PredefinedMenuItem::quit(Some("Quit"))
}

fn about_item(image: DynamicImage) -> PredefinedMenuItem {
    let about_metadata_builder = AboutMetadataBuilder::new()
        .name(Some("RustCast"))
        .version(Some(
            option_env!("APP_VERSION").unwrap_or("Unknown Version"),
        ))
        .authors(Some(vec!["Unsecretised".to_string()]))
        .credits(Some("Unsecretised".to_string()))
        .icon(Ico::from_rgba(image.as_bytes().to_vec(), image.width(), image.height()).ok())
        .website(Some("https://rustcast.umangsurana.com"))
        .license(Some("MIT"))
        .build();

    PredefinedMenuItem::about(Some("About.."), Some(about_metadata_builder))
}
