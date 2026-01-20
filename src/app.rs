//! Main logic for the app
use crate::commands::Function;
use crate::{app::tile::ExtSender, clipboard::ClipBoardContentType};

pub mod apps;
pub mod menubar;
pub mod pages;
pub mod tile;

use iced::window::{self, Id, Settings};
/// The default window width
pub const WINDOW_WIDTH: f32 = 500.;

/// The default window height
pub const DEFAULT_WINDOW_HEIGHT: f32 = 80.;

/// The rustcast descriptor name to be put for all rustcast commands
pub const RUSTCAST_DESC_NAME: &str = "Utility";

/// The different pages that rustcast can have / has
#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Main,
    ClipboardHistory,
    EmojiSearch,
}

/// The types of arrow keys
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ArrowKey {
    Up,
    Down,
    Left,
    Right,
}

/// The ways the cursor can move when a key is pressed
#[derive(Debug, Clone)]
pub enum Move {
    Back,
    Forwards(String),
}

/// The message type that iced uses for actions that can do something
#[derive(Debug, Clone)]
pub enum Message {
    OpenWindow,
    SearchQueryChanged(String, Id),
    KeyPressed(u32),
    FocusTextInput(Move),
    HideWindow(Id),
    RunFunction(Function),
    OpenFocused,
    ReturnFocus,
    EscKeyPressed(Id),
    ClearSearchResults,
    WindowFocusChanged(Id, bool),
    ClearSearchQuery,
    HideTrayIcon,
    ReloadConfig,
    SetSender(ExtSender),
    SwitchToPage(Page),
    ClipboardHistory(ClipBoardContentType),
    ChangeFocus(ArrowKey),
}

/// The window settings for rustcast
pub fn default_settings() -> Settings {
    Settings {
        resizable: false,
        decorations: false,
        minimizable: false,
        level: window::Level::AlwaysOnTop,
        transparent: true,
        blur: true,
        size: iced::Size {
            width: WINDOW_WIDTH,
            height: DEFAULT_WINDOW_HEIGHT,
        },
        ..Default::default()
    }
}
