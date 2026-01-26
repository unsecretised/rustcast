//! This has all the utility functions that rustcast uses
use std::{fs::File, io::Write, path::Path, process::exit, thread};

use iced::widget::image::Handle;
use icns::IconFamily;
use image::RgbaImage;
use objc2_app_kit::NSWorkspace;
use objc2_foundation::NSURL;

/// The default error log path (works only on unix systems, and must be changed for windows
/// support)
const ERR_LOG_PATH: &str = "/tmp/rustscan-err.log";

/// This logs an error to the error log file
pub(crate) fn log_error(msg: &str) {
    eprintln!("{msg}");
    if let Ok(mut file) = File::options().create(true).append(true).open(ERR_LOG_PATH) {
        let _ = file.write_all(msg.as_bytes()).ok();
    }
}

/// This logs an error to the error log file, and exits the program
pub(crate) fn log_error_and_exit(msg: &str) -> ! {
    log_error(msg);
    exit(-1)
}

/// This converts an icns file to an iced image handle
pub(crate) fn handle_from_icns(path: &Path) -> Option<Handle> {
    let data = std::fs::read(path).ok()?;
    let family = IconFamily::read(std::io::Cursor::new(&data)).ok()?;

    let icon_type = family.available_icons();

    let icon = family.get_icon_with_type(*icon_type.first()?).ok()?;
    let image = RgbaImage::from_raw(
        icon.width() as u32,
        icon.height() as u32,
        icon.data().to_vec(),
    )?;
    Some(Handle::from_rgba(
        image.width(),
        image.height(),
        image.into_raw(),
    ))
}

/// Open the settings file with the system default editor
pub fn open_settings() {
    thread::spawn(move || {
        NSWorkspace::new().openURL(&NSURL::fileURLWithPath(
            &objc2_foundation::NSString::from_str(
                &(std::env::var("HOME").unwrap_or("".to_string())
                    + "/.config/rustcast/config.toml"),
            ),
        ));
    });
}

/// Open a provided URL (Platform specific)
pub fn open_url(url: &str) {
    let url = url.to_owned();
    thread::spawn(move || {
        NSWorkspace::new().openURL(
            &NSURL::URLWithString_relativeToURL(&objc2_foundation::NSString::from_str(&url), None)
                .unwrap(),
        );
    });
}

/// Check if the provided string is a valid url
pub fn is_valid_url(s: &str) -> bool {
    s.ends_with(".com")
        || s.ends_with(".net")
        || s.ends_with(".org")
        || s.ends_with(".edu")
        || s.ends_with(".gov")
        || s.ends_with(".io")
        || s.ends_with(".co")
        || s.ends_with(".me")
        || s.ends_with(".app")
        || s.ends_with(".dev")
}
