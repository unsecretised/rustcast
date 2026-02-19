//! This modules handles the logic for each "app" that rustcast can load
//!
//! An "app" is effectively, one of the results that rustcast returns when you search for something

use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use iced::{
    Alignment,
    Length::Fill,
    widget::{self, Button, Row, Text, container, image::Viewer, text::Wrapping},
};

use crate::{
    app::{Message, Page, RUSTCAST_DESC_NAME},
    clipboard::ClipBoardContentType,
    commands::Function,
    cross_platform::get_img_handle,
    styles::{result_button_style, result_row_container_style},
};

/// This tells each "App" what to do when it is clicked, whether it is a function, a message, or a display
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum AppCommand {
    Function(Function),
    Message(Message),
    Display,
}

impl PartialEq for AppCommand {
    fn eq(&self, other: &Self) -> bool {
        // TODO: make an *actual* impl of PartialEq for Message
        match (&self, &other) {
            (Self::Function(a), Self::Function(b)) => a == b,
            (Self::Display, Self::Display) => true,
            _ => false,
        }
    }
}

/// A container for [`App`] data specific to a certain type of app.
#[derive(Debug, Clone, PartialEq)]
pub enum AppData {
    /// A platform specific executable
    Executable {
        /// The executable path
        path: PathBuf,
        /// The executable icon
        icon: Option<iced::widget::image::Handle>,
    },
    /// A shell command to be run
    Command {
        /// The command to run
        command: String,
        alias: String,
        /// The icon to display in search results
        icon: Option<iced::widget::image::Handle>,
    },
    /// Any builtin function
    Builtin {
        /// The [`AppCommand`] to run
        command: AppCommand,
    },
}

/// The main app struct, that represents an "App"
///
/// This struct represents a command that rustcast can perform, providing the rustcast
/// the data needed to search for the app, to display the app in search results, and to actually
/// run the app.
#[derive(Clone, Debug)]
pub struct App {
    /// The app name
    pub name: String,

    /// An alias to use while searching
    pub alias: String,

    /// The description for the app
    pub desc: String,

    /// The information specific to a certain type of app
    pub data: AppData,

    /// A unique ID generated for each instance of an App.
    #[allow(unused)]
    id: usize,
}

impl PartialEq for App {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data && self.name == other.name
    }
}

impl App {
    /// Get the numeric id of an app
    #[allow(unused)]
    pub fn id(&self) -> usize {
        self.id
    }

    /// Creates a new instance
    pub fn new(name: &str, name_lc: &str, desc: &str, data: AppData) -> Self {
        static ID: AtomicUsize = AtomicUsize::new(0);

        Self {
            alias: name_lc.to_string(),
            name: name.to_string(),
            desc: desc.to_string(),
            id: ID.fetch_add(1, Ordering::Relaxed),
            data,
        }
    }

    /// Creates a new instance of the type [`AppData::Builtin`].
    ///
    /// This is mainly for convenience.
    pub fn new_builtin(name: &str, name_lc: &str, desc: &str, command: AppCommand) -> Self {
        Self::new(name, name_lc, desc, AppData::Builtin { command })
    }

    /// Creates a new instance of the type [`AppData::Executable`].
    ///
    /// This is mainly for convenience.
    pub fn new_executable(
        name: &str,
        name_lc: &str,
        desc: &str,
        path: impl AsRef<Path>,
        icon: Option<widget::image::Handle>,
    ) -> Self {
        Self::new(
            name,
            name_lc,
            desc,
            AppData::Executable {
                path: path.as_ref().to_path_buf(),
                icon,
            },
        )
    }

    /// A vec of all the emojis as App structs
    pub fn emoji_apps() -> Vec<App> {
        emojis::iter()
            .filter(|x| x.unicode_version() < emojis::UnicodeVersion::new(17, 13))
            .map(|x| {
                App::new_builtin(
                    x.name(),
                    x.name(),
                    "emoji",
                    AppCommand::Function(Function::CopyToClipboard(ClipBoardContentType::Text(
                        x.to_string(),
                    ))),
                )
            })
            .collect()
    }
    /// This returns the basic apps that rustcast has, such as quiting rustcast and opening preferences
    pub fn basic_apps() -> Vec<App> {
        let app_version = option_env!("APP_VERSION").unwrap_or("Unknown Version");

        vec![
            Self::new_builtin(
                "Quit RustCast",
                "quit",
                RUSTCAST_DESC_NAME,
                AppCommand::Function(Function::Quit),
            ),
            Self::new_builtin(
                "Open RustCast Preferences",
                "settings",
                RUSTCAST_DESC_NAME,
                AppCommand::Function(Function::OpenPrefPane),
            ),
            Self::new_builtin(
                "Search for an Emoji",
                "emoji",
                RUSTCAST_DESC_NAME,
                AppCommand::Message(Message::SwitchToPage(Page::EmojiSearch)),
            ),
            Self::new_builtin(
                "Clipboard History",
                "clipboard",
                RUSTCAST_DESC_NAME,
                AppCommand::Message(Message::SwitchToPage(Page::ClipboardHistory)),
            ),
            Self::new_builtin(
                "Reload RustCast",
                "refresh",
                RUSTCAST_DESC_NAME,
                AppCommand::Message(Message::ReloadConfig),
            ),
            Self::new_builtin(
                &format!("Current RustCast Version: {app_version}"),
                "version",
                RUSTCAST_DESC_NAME,
                AppCommand::Display,
            ),
            #[cfg(target_os = "macos")]
            Self::new_executable(
                "Finder",
                "finder",
                "Application",
                PathBuf::from("/System/Library/CoreServices/Finder.app"),
                get_img_handle(Path::new(
                    "/System/Library/CoreServices/Finder.app/Contents/Resources/Finder.icns",
                )),
            ),
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

        if theme.show_icons {
            match self.data {
                AppData::Command {
                    icon: Some(ref icon),
                    ..
                }
                | AppData::Executable {
                    icon: Some(ref icon),
                    ..
                } => {
                    row = row.push(
                        container(Viewer::new(icon).height(40).width(40))
                            .width(40)
                            .height(40),
                    );
                }
                AppData::Builtin { .. } => {
                    let icon = get_img_handle(Path::new(
                        "/Applications/Rustcast.app/Contents/Resources/icon.icns",
                    ));
                    if let Some(icon) = icon {
                        row = row.push(
                            container(Viewer::new(icon).height(40).width(40))
                                .width(40)
                                .height(40),
                        );
                    }
                }
                _ => {}
            }
        }
        row = row.push(container(text_block).width(Fill));

        let msg = match self.data {
            AppData::Builtin {
                command: AppCommand::Function(func),
                ..
            } => Some(Message::RunFunction(func)),
            AppData::Builtin {
                command: AppCommand::Message(msg),
                ..
            } => Some(msg),
            AppData::Builtin {
                command: AppCommand::Display,
                ..
            } => None,
            AppData::Executable { path, .. } => Some(Message::RunFunction(Function::OpenApp(path))),
            AppData::Command { command, alias, .. } => Some(Message::RunFunction(
                Function::RunShellCommand(command, alias),
            )),
        };

        let theme_clone = theme.clone();

        let content = Button::new(row)
            .on_press_maybe(msg)
            .style(move |_, _| result_button_style(&theme_clone))
            .width(Fill)
            .padding(0)
            .height(50);

        container(content)
            .id(format!("result-{id_num}"))
            .style(move |_| result_row_container_style(&theme, focused))
            .padding(8)
            .width(Fill)
            .into()
    }
}
