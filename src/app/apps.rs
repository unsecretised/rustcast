//! This modules handles the logic for each "app" that rustcast can load
//!
//! An "app" is effectively, one of the results that rustcast returns when you search for something
use std::path::Path;

use iced::{
    Alignment,
    Length::Fill,
    widget::{Button, Row, Text, container, image::Viewer, text::Wrapping},
};

use crate::{
    app::{Message, Page, RUSTCAST_DESC_NAME},
    clipboard::ClipBoardContentType,
    commands::Function,
    styles::{result_button_style, result_row_container_style},
    utils::handle_from_icns,
};

/// This tells each "App" what to do when it is clicked, whether it is a function, a message, or a display
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum AppCommand {
    Function(Function),
    Message(Message),
    Display,
}

/// The main app struct, that represents an "App"
///
/// This struct represents a command that rustcast can perform, providing the rustcast
/// the data needed to search for the app, to display the app in search results, and to actually
/// "run" the app.
#[derive(Debug, Clone)]
pub struct App {
    pub open_command: AppCommand,
    pub desc: String,
    pub icons: Option<iced::widget::image::Handle>,
    pub name: String,
    pub name_lc: String,
}

impl PartialEq for App {
    fn eq(&self, other: &Self) -> bool {
        self.name_lc == other.name_lc
            && self.icons == other.icons
            && self.desc == other.desc
            && self.name == other.name
    }
}

impl App {
    /// A vec of all the emojis as App structs
    pub fn emoji_apps() -> Vec<App> {
        emojis::iter()
            .filter(|x| x.unicode_version() < emojis::UnicodeVersion::new(17, 13))
            .map(|x| App {
                icons: None,
                name: x.to_string(),
                name_lc: x.name().to_string(),
                open_command: AppCommand::Function(Function::CopyToClipboard(
                    ClipBoardContentType::Text(x.to_string()),
                )),
                desc: x.name().to_string(),
            })
            .collect()
    }
    /// This returns the basic apps that rustcast has, such as quiting rustcast and opening preferences
    pub fn basic_apps() -> Vec<App> {
        let app_version = option_env!("APP_VERSION").unwrap_or("Unknown Version");

        vec![
            App {
                open_command: AppCommand::Function(Function::Quit),
                desc: RUSTCAST_DESC_NAME.to_string(),
                icons: handle_from_icns(Path::new(
                    "/Applications/Rustcast.app/Contents/Resources/icon.icns",
                )),
                name: "Quit RustCast".to_string(),
                name_lc: "quit".to_string(),
            },
            App {
                open_command: AppCommand::Function(Function::OpenPrefPane),
                desc: RUSTCAST_DESC_NAME.to_string(),
                icons: handle_from_icns(Path::new(
                    "/Applications/Rustcast.app/Contents/Resources/icon.icns",
                )),
                name: "Open RustCast Preferences".to_string(),
                name_lc: "settings".to_string(),
            },
            App {
                open_command: AppCommand::Message(Message::SwitchToPage(Page::EmojiSearch)),
                desc: RUSTCAST_DESC_NAME.to_string(),
                icons: handle_from_icns(Path::new(
                    "/Applications/Rustcast.app/Contents/Resources/icon.icns",
                )),
                name: "Search for an Emoji".to_string(),
                name_lc: "emoji".to_string(),
            },
            App {
                open_command: AppCommand::Message(Message::SwitchToPage(Page::ClipboardHistory)),
                desc: RUSTCAST_DESC_NAME.to_string(),
                icons: handle_from_icns(Path::new(
                    "/Applications/Rustcast.app/Contents/Resources/icon.icns",
                )),
                name: "Clipboard History".to_string(),
                name_lc: "clipboard".to_string(),
            },
            App {
                open_command: AppCommand::Message(Message::ReloadConfig),
                desc: RUSTCAST_DESC_NAME.to_string(),
                icons: handle_from_icns(Path::new(
                    "/Applications/Rustcast.app/Contents/Resources/icon.icns",
                )),
                name: "Reload RustCast".to_string(),
                name_lc: "refresh".to_string(),
            },
            App {
                open_command: AppCommand::Display,
                desc: RUSTCAST_DESC_NAME.to_string(),
                icons: handle_from_icns(Path::new(
                    "/Applications/Rustcast.app/Contents/Resources/icon.icns",
                )),
                name: format!("Current RustCast Version: {app_version}"),
                name_lc: "version".to_string(),
            },
        ]
    }

    /// This renders the app into an iced element, allowing it to be displayed in the search results
    pub fn render(
        self,
        theme: crate::config::Theme,
        id_num: u32,
        focussed_id: u32,
    ) -> iced::Element<'static, Message> {
        let focused = focussed_id == id_num;

        // Title + subtitle (Raycast style)
        let text_block = iced::widget::Column::new()
            .spacing(2)
            .push(
                Text::new(self.name)
                    .font(theme.font())
                    .size(16)
                    .wrapping(Wrapping::WordOrGlyph)
                    .color(theme.text_color(1.0)),
            )
            .push(
                Text::new(self.desc)
                    .font(theme.font())
                    .size(13)
                    .color(theme.text_color(0.55)),
            );

        let mut row = Row::new()
            .align_y(Alignment::Center)
            .width(Fill)
            .spacing(10)
            .height(50);

        if theme.show_icons
            && let Some(icon) = &self.icons
        {
            row = row.push(
                container(Viewer::new(icon).height(40).width(40))
                    .width(40)
                    .height(40),
            );
        }
        row = row.push(container(text_block).width(Fill));

        let msg = match self.open_command.clone() {
            AppCommand::Function(func) => Some(Message::RunFunction(func)),
            AppCommand::Message(msg) => Some(msg),
            AppCommand::Display => None,
        };

        let theme_clone = theme.clone();

        let content = Button::new(row)
            .on_press_maybe(msg)
            .style(move |_, _| result_button_style(&theme_clone))
            .width(Fill)
            .padding(0)
            .height(50);

        container(content)
            .id(format!("result-{}", id_num))
            .style(move |_| result_row_container_style(&theme, focused))
            .padding(8)
            .width(Fill)
            .into()
    }
}
