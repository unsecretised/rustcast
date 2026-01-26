#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

pub fn open_url(url: &str) {
    #[cfg(target_os = "macos")]
    macos::open_url(url)
}

pub fn open_settings() {
    #[cfg(target_os = "macos")]
    macos::open_settings()
}
