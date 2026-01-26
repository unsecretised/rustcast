//! This has the menubar icon logic for the app

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use image::DynamicImage;
use tokio::runtime::Runtime;
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{
        AboutMetadataBuilder, Icon as Ico, Menu, MenuEvent, MenuItem, PredefinedMenuItem,
        accelerator::Accelerator,
    },
};

use crate::{
    app::{Message, tile::ExtSender},
    cross_platform::{open_settings, open_url},
};

const DISCORD_LINK: &str = "https://discord.gg/bDfNYPbnC5";

use tokio::runtime::Runtime;

/// This create a new menubar icon for the app
pub fn menu_icon(hotkey: HotKey, sender: ExtSender) -> TrayIcon {
    let builder = TrayIconBuilder::new();

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
    #[cfg(target_os = "macos")]
    {
        use image::ImageReader;

        let image_path = if cfg!(debug_assertions) && !cfg!(target_os = "macos") {
            "docs/icon.png"
        } else {
            "/Applications/Rustcast.app/Contents/Resources/icon.png"
        };

        ImageReader::open(image_path).unwrap().decode().unwrap()
    }

    #[cfg(target_os = "windows")]
    {
        DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            64,
            64,
            image::Rgba([0, 0, 0, 255]),
        ))
    }
}

fn init_event_handler(sender: ExtSender, hotkey_id: u32) {
    tracing::debug!("Initing event handler");
    let runtime = Runtime::new().unwrap();

    MenuEvent::set_event_handler(Some(move |x: MenuEvent| {
        let sender = sender.clone();
        let sender = sender.0.clone();
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
            _ => {}
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
