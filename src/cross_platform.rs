#![warn(missing_docs)]

use std::path::Path;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

/// Opens the settings file
pub fn open_settings() {
    #[cfg(target_os = "macos")]
    macos::open_settings()
}

/// Gets an iced image handle
pub fn get_img_handle(path: &Path) -> Option<iced::widget::image::Handle> {
    if !path.exists() {
        return None;
    }

    #[cfg(target_os = "macos")]
    return macos::handle_from_icns(path);

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    return Some(iced::widget::image::Handle::from_path(path));
}
