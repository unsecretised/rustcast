#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

pub fn open_url(url: &str) {
    #[cfg(target_os = "macos")]
    macos::open_url(url);

    #[cfg(target_os = "windows")]
    windows::open_url(url);

    #[cfg(target_os = "linux")]
    linux::open_url(url);
}

pub fn open_settings() {
    #[cfg(target_os = "macos")]
    macos::open_settings()
}
