//! This module handles the logic for the tile, AKA rustcast's main window
pub mod elm;
pub mod update;

mod search_query;

#[cfg(target_os = "windows")]
use {
    windows::Win32::Foundation::HWND, windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow,
};

use std::{collections::BTreeMap, fs, ops::Bound, path::PathBuf, time::Duration};

use iced::{
    Subscription, Theme, event, futures,
    futures::{
        SinkExt,
        channel::mpsc::{Sender, channel},
    },
    keyboard::{self, Modifiers, key::Named},
    stream, window,
};

#[cfg(not(target_os = "linux"))]
use global_hotkey::{GlobalHotKeyEvent, HotKeyState, hotkey::HotKey};

use crate::{
    app::{ArrowKey, Message, Move, Page, apps::App, tile::elm::default_app_paths},
    clipboard::ClipBoardContentType,
    config::Config,
    cross_platform::open_settings,
};

use arboard::Clipboard;
use rayon::prelude::*;
use tray_icon::TrayIcon;

#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2_app_kit::NSRunningApplication;

/// This is a wrapper around the sender to disable dropping
#[derive(Clone, Debug)]
pub struct ExtSender(pub Sender<Message>);

/// Disable dropping the sender
impl Drop for ExtSender {
    fn drop(&mut self) {}
}

/// All the indexed apps that rustcast can search for
#[derive(Clone, Debug)]
struct AppIndex {
    by_name: BTreeMap<String, App>,
}

impl AppIndex {
    /// Search for an element in the index that starts with the provided prefix
    fn search_prefix<'a>(&'a self, prefix: &'a str) -> impl Iterator<Item = &'a App> + 'a {
        self.by_name
            .range::<str, _>((Bound::Included(prefix), Bound::Unbounded))
            .take_while(move |(k, _)| k.starts_with(prefix))
            .map(|(_, v)| v)
    }

    /// Factory function for creating
    pub fn from_apps(options: Vec<App>) -> Self {
        let mut bmap = BTreeMap::new();
        for app in options {
            bmap.insert(app.alias.clone(), app);
        }

        AppIndex { by_name: bmap }
    }
}

/// This is the base window, and its a "Tile"
/// Its fields are:
/// - Theme ([`iced::Theme`])
/// - Query (String)
/// - Query Lowercase (String, but lowercase)
/// - Previous Query Lowercase (String)
/// - Results (Vec<[`App`]>) the results of the search
/// - Options (Vec<[`App`]>) the options to search through
/// - Visible (bool) whether the window is visible or not
/// - Focused (bool) whether the window is focused or not
/// - Frontmost ([`Option<Retained<NSRunningApplication>>`]) the frontmost application before the window was opened
/// - Config ([`Config`]) the app's config
/// - Open Hotkey ID (`u32`) the id of the hotkey that opens the window
/// - Clipboard Content (`Vec<`[`ClipBoardContentType`]`>`) all of the cliboard contents
/// - Page ([`Page`]) the current page of the window (main or clipboard history)
#[derive(Clone)]
pub struct Tile {
    pub theme: iced::Theme,
    pub focus_id: u32,
    pub query: String,
    query_lc: String,
    results: Vec<App>,
    options: AppIndex,
    emoji_apps: AppIndex,
    visible: bool,
    focused: bool,
    #[cfg(target_os = "macos")]
    frontmost: Option<Retained<NSRunningApplication>>,
    #[cfg(target_os = "windows")]
    frontmost: Option<HWND>,
    pub config: Config,
    /// The opening hotkey
    #[cfg(not(target_os = "linux"))]
    hotkey: HotKey,
    #[cfg(not(target_os = "linux"))]
    clipboard_hotkey: Option<HotKey>,
    clipboard_content: Vec<ClipBoardContentType>,
    tray_icon: Option<TrayIcon>,
    sender: Option<ExtSender>,
    page: Page,
}

impl Tile {
    /// This returns the theme of the window
    pub fn theme(&self, _: window::Id) -> Option<Theme> {
        Some(self.theme.clone())
    }

    /// This handles the subscriptions of the window
    ///
    /// The subscriptions are:
    /// - Hotkeys
    /// - Hot reloading
    /// - Clipboard history
    /// - Window close events
    /// - Keypresses (escape to close the window)
    /// - Window focus changes
    pub fn subscription(&self) -> Subscription<Message> {
        let keyboard = event::listen_with(|event, _, id| match event {
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) => Some(Message::EscKeyPressed(id)),
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(cha),
                modifiers: Modifiers::LOGO,
                ..
            }) => {
                if cha.to_string() == "," {
                    open_settings();
                }
                None
            }
            _ => None,
        });
        Subscription::batch([
            #[cfg(not(target_os = "linux"))]
            Subscription::run(handle_hotkeys),
            #[cfg(target_os = "linux")]
            Subscription::run(handle_socket),
            keyboard,
            Subscription::run(handle_recipient),
            Subscription::run(handle_hot_reloading),
            Subscription::run(handle_clipboard_history),
            window::close_events().map(Message::HideWindow),
            keyboard::listen().filter_map(|event| {
                if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                    match key {
                        keyboard::Key::Named(Named::Escape) => {
                            return Some(Message::KeyPressed(65598));
                        }
                        keyboard::Key::Named(Named::ArrowUp) => {
                            return Some(Message::ChangeFocus(ArrowKey::Up));
                        }
                        keyboard::Key::Named(Named::ArrowLeft) => {
                            return Some(Message::ChangeFocus(ArrowKey::Left));
                        }
                        keyboard::Key::Named(Named::ArrowRight) => {
                            return Some(Message::ChangeFocus(ArrowKey::Right));
                        }
                        keyboard::Key::Named(Named::ArrowDown) => {
                            return Some(Message::ChangeFocus(ArrowKey::Down));
                        }
                        keyboard::Key::Character(chr) => {
                            if modifiers.command() && chr.to_string().to_lowercase() == "r" {
                                return Some(Message::ReloadConfig);
                            } else if modifiers.command() && chr.to_string() == "," {
                                open_settings();
                            } else {
                                return Some(Message::FocusTextInput(Move::Forwards(
                                    chr.to_string(),
                                )));
                            }
                        }
                        keyboard::Key::Named(Named::Enter) => return Some(Message::OpenFocused),
                        keyboard::Key::Named(Named::Backspace) => {
                            return Some(Message::FocusTextInput(Move::Back));
                        }
                        _ => {}
                    }
                    None
                } else {
                    None
                }
            }),
            window::events()
                .with(self.focused)
                .filter_map(|(focused, (wid, event))| match event {
                    window::Event::Unfocused => {
                        if focused {
                            Some(Message::WindowFocusChanged(wid, false))
                        } else {
                            None
                        }
                    }
                    window::Event::Focused => Some(Message::WindowFocusChanged(wid, true)),
                    _ => None,
                }),
        ])
    }

    /// Handles the search query changed event.
    ///
    /// This is separate from the `update` function because it has a decent amount of logic, and
    /// should be separated out to make it easier to test. This function is called by the `update`
    /// function to handle the search query changed event.
    pub fn handle_search_query_changed(&mut self) {
        let query = self.query_lc.clone();
        let options = if self.page == Page::Main {
            &self.options
        } else if self.page == Page::EmojiSearch {
            &self.emoji_apps
        } else {
            &AppIndex::from_apps(vec![])
        };
        let results: Vec<App> = options
            .search_prefix(&query)
            .map(|x| x.to_owned())
            .collect();

        self.results = results;
    }

    // Unused, keeping it for now
    // pub fn capture_frontmost(&mut self) {
    //     #[cfg(target_os = "macos")]
    //     {
    //         use objc2_app_kit::NSWorkspace;

    //         let ws = NSWorkspace::sharedWorkspace();
    //         self.frontmost = ws.frontmostApplication();
    //     };

    //     #[cfg(target_os = "windows")]
    //     {
    //         use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    //         self.frontmost = Some(unsafe { GetForegroundWindow() });
    //     }
    // }

    /// Gets the frontmost application to focus later.
    #[cfg(target_os = "macos")]
    pub fn capture_frontmost(&mut self) {
        use objc2_app_kit::NSWorkspace;

        let ws = NSWorkspace::sharedWorkspace();
        self.frontmost = ws.frontmostApplication();
    }

    /// Restores the frontmost application.
    #[allow(deprecated, unused)]
    pub fn restore_frontmost(&mut self) {
        #[cfg(target_os = "macos")]
        {
            if let Some(app) = self.frontmost.take() {
                use objc2_app_kit::NSApplicationActivationOptions;

                app.activateWithOptions(NSApplicationActivationOptions::ActivateIgnoringOtherApps);
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Some(handle) = self.frontmost {
                unsafe {
                    let _ = SetForegroundWindow(handle);
                }
            }
        }
    }
}

/// This is the subscription function that handles hot reloading of the config
fn handle_hot_reloading() -> impl futures::Stream<Item = Message> {
    stream::channel(100, async |mut output| {
        let mut content = fs::read_to_string(
            std::env::var("HOME").unwrap_or("".to_owned()) + "/.config/rustcast/config.toml",
        )
        .unwrap_or("".to_string());

        let paths = default_app_paths();
        let mut total_files: usize = paths
            .par_iter()
            .map(|dir| count_dirs_in_dir(&dir.to_owned().into()))
            .sum();

        loop {
            let current_content = fs::read_to_string(
                std::env::var("HOME").unwrap_or("".to_owned()) + "/.config/rustcast/config.toml",
            )
            .unwrap_or("".to_string());

            let current_total_files: usize = paths
                .par_iter()
                .map(|dir| count_dirs_in_dir(&dir.to_owned().into()))
                .sum();

            if current_content != content {
                content = current_content;
                output.send(Message::ReloadConfig).await.unwrap();
            } else if total_files != current_total_files {
                total_files = current_total_files;
                output.send(Message::ReloadConfig).await.unwrap();
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
}

fn count_dirs_in_dir(dir: &PathBuf) -> usize {
    // Read the directory; if it fails, treat as empty
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };

    entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .count()
}

/// This is the subscription function that handles hotkeys for hiding / showing the window
#[cfg(not(target_os = "linux"))]
fn handle_hotkeys() -> impl futures::Stream<Item = Message> {
    stream::channel(100, async |mut output| {
        let receiver = GlobalHotKeyEvent::receiver();
        loop {
            if let Ok(event) = receiver.recv()
                && event.state == HotKeyState::Pressed
            {
                output.try_send(Message::HotkeyPressed(event.id)).unwrap();
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
}

#[cfg(target_os = "linux")]
fn handle_socket() -> impl futures::Stream<Item = Message> {
    stream::channel(100, async |mut output| {
        let clipboard = env::args().any(|arg| arg.trim() == "--cphist");
        if clipboard {
            output
                .try_send(Message::OpenToPage(Page::ClipboardHistory))
                .unwrap();
        }

        use std::env;

        use tokio::net::UnixListener;

        let _ = fs::remove_file(crate::SOCKET_PATH);
        let listener = UnixListener::bind(crate::SOCKET_PATH).unwrap();

        while let Ok((mut stream, _address)) = listener.accept().await {
            let mut output = output.clone();
            tokio::spawn(async move {
                use tokio::io::AsyncReadExt;
                use tracing::info;

                let mut s = String::new();
                let _ = stream.read_to_string(&mut s).await;
                info!("received socket command {s}");
                if s.trim() == "toggle" {
                    output.try_send(Message::OpenToPage(Page::Main)).unwrap();
                } else if s.trim() == "clipboard" {
                    output
                        .try_send(Message::OpenToPage(Page::ClipboardHistory))
                        .unwrap();
                }
            });
        }
    })
}

/// This is the subscription function that handles the change in clipboard history
fn handle_clipboard_history() -> impl futures::Stream<Item = Message> {
    stream::channel(100, async |mut output| {
        let mut clipboard = Clipboard::new().unwrap();
        let mut prev_byte_rep: Option<ClipBoardContentType> = None;

        loop {
            let byte_rep = if let Ok(a) = clipboard.get_image() {
                Some(ClipBoardContentType::Image(a))
            } else if let Ok(a) = clipboard.get_text() {
                Some(ClipBoardContentType::Text(a))
            } else {
                None
            };

            if byte_rep != prev_byte_rep
                && let Some(content) = &byte_rep
            {
                output
                    .send(Message::ClipboardHistory(content.to_owned()))
                    .await
                    .ok();
                prev_byte_rep = byte_rep;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
}

fn handle_recipient() -> impl futures::Stream<Item = Message> {
    stream::channel(100, async |mut output| {
        let (sender, mut recipient) = channel(100);
        let msg = Message::SetSender(ExtSender(sender));
        tracing::debug!("Sending ExtSender");
        output.send(msg).await.expect("Sender not sent");
        loop {
            let abcd = recipient
                .try_next()
                .map(async |msg| {
                    if let Some(msg) = msg {
                        output.send(msg).await.unwrap();
                    }
                })
                .ok();

            if let Some(abcd) = abcd {
                abcd.await;
            }
            tokio::time::sleep(Duration::from_nanos(10)).await;
        }
    })
}
