//! This modules handles the logic for each "app" that rustcast can load
//!
//! An "app" is effectively, one of the results that rustcast returns when you search for something
use std::path::Path;

use iced::{
    Alignment, Background,
    Length::Fill,
    alignment::Vertical,
    widget::{Button, Row, Text, container, image::Viewer, space},
};

use crate::{
    app::{Message, Page, RUSTCAST_DESC_NAME},
    commands::Function,
    utils::handle_from_icns,
};

/// This tells each "App" what to do when it is clicked, whether it is a function, a message, or a display
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
pub struct App {
    pub open_command: AppCommand,
    pub desc: String,
    pub icons: Option<iced::widget::image::Handle>,
    pub name: String,
    pub name_lc: String,
}

impl App {
    /// This returns the basic apps that rustcast has, such as quiting rustcast and opening preferences
    pub fn basic_apps() -> Vec<App> {
        let app_version = option_env!("APP_VERSION").unwrap_or("Unknown Version");

        vec![
            App {
                open_command: AppCommand::Function(Function::Quit),
                desc: RUSTCAST_DESC_NAME.to_string(),
                icons: None,
                name: "Quit RustCast".to_string(),
                name_lc: "quit".to_string(),
            },
            App {
                open_command: AppCommand::Function(Function::OpenPrefPane),
                desc: RUSTCAST_DESC_NAME.to_string(),
                icons: None,
                name: "Open RustCast Preferences".to_string(),
                name_lc: "settings".to_string(),
            },
            App {
                open_command: AppCommand::Message(Message::SwitchToPage(Page::ClipboardHistory)),
                desc: RUSTCAST_DESC_NAME.to_string(),
                icons: None,
                name: "Clipboard History".to_string(),
                name_lc: "clipboard".to_string(),
            },
            App {
                open_command: AppCommand::Message(Message::ReloadConfig),
                desc: RUSTCAST_DESC_NAME.to_string(),
                icons: None,
                name: "Reload RustCast".to_string(),
                name_lc: "refresh".to_string(),
            },
            App {
                open_command: AppCommand::Display,
                desc: RUSTCAST_DESC_NAME.to_string(),
                icons: None,
                name: format!("Current RustCast Version: {app_version}"),
                name_lc: "version".to_string(),
            },
            App {
                open_command: AppCommand::Function(Function::OpenApp(
                    "/System/Library/CoreServices/Finder.app".to_string(),
                )),
                desc: RUSTCAST_DESC_NAME.to_string(),
                icons: handle_from_icns(Path::new(
                    "/System/Library/CoreServices/Finder.app/Contents/Resources/Finder.icns",
                )),
                name: "Finder".to_string(),
                name_lc: "finder".to_string(),
            },
        ]
    }

    /// This renders the app into an iced element, allowing it to be displayed in the search results
    pub fn render<'a>(
        &'a self,
        theme: &'a crate::config::Theme,
    ) -> impl Into<iced::Element<'a, Message>> {
        let mut tile = Row::new().width(Fill).height(55);

        if theme.show_icons {
            if let Some(icon) = &self.icons {
                tile = tile
                    .push(Viewer::new(icon).height(35).width(35))
                    .align_y(Alignment::Center);
            } else {
                tile = tile
                    .push(space().height(Fill))
                    .width(55)
                    .height(55)
                    .align_y(Alignment::Center);
            }
        }

        tile = tile.push(
            Button::new(
                Text::new(&self.name)
                    .height(Fill)
                    .width(Fill)
                    .color(theme.text_color(1.))
                    .align_y(Vertical::Center),
            )
            .on_press_maybe({
                match self.open_command.clone() {
                    AppCommand::Function(func) => Some(Message::RunFunction(func)),
                    AppCommand::Message(msg) => Some(msg),
                    AppCommand::Display => None,
                }
            })
            .style(|_, _| iced::widget::button::Style {
                background: Some(Background::Color(theme.bg_color())),
                text_color: theme.text_color(1.),
                ..Default::default()
            })
            .width(Fill)
            .height(55),
        );

        tile = tile
            .push(container(Text::new(&self.desc).color(theme.text_color(0.4))).padding(15))
            .width(Fill);

        container(tile)
            .style(|_| iced::widget::container::Style {
                text_color: Some(theme.text_color(1.)),
                background: Some(Background::Color(theme.bg_color())),
                ..Default::default()
            })
            .width(Fill)
            .height(Fill)
    }
}
