use std::{path::Path, sync::Arc};

use iced::{theme::Custom, widget::image::Handle};
use serde::{Deserialize, Serialize};

use crate::{app::App, commands::Function, utils::handle_from_icns};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub toggle_mod: String,
    pub toggle_key: String,
    pub buffer_rules: Buffer,
    pub theme: Theme,
    pub placeholder: String,
    pub search_url: String,
    pub shells: Vec<Shelly>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            toggle_mod: "ALT".to_string(),
            toggle_key: "Space".to_string(),
            buffer_rules: Buffer::default(),
            theme: Theme::default(),
            placeholder: String::from("Time to be productive!"),
            search_url: "https://google.com/search?q=%s".to_string(),
            shells: vec![],
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Theme {
    pub text_color: (f32, f32, f32),
    pub background_color: (f32, f32, f32),
    pub background_opacity: f32,
    pub blur: bool,
    pub show_icons: bool,
    pub show_scroll_bar: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            text_color: (0.95, 0.95, 0.96),
            background_color: (0.11, 0.11, 0.13),
            background_opacity: 1.,
            blur: false,
            show_icons: true,
            show_scroll_bar: true,
        }
    }
}

impl Theme {
    pub fn to_iced_theme(&self) -> iced::Theme {
        let text_color = self.text_color;
        let bg_color = self.background_color;
        let palette = iced::theme::Palette {
            background: iced::Color {
                r: bg_color.0,
                g: bg_color.1,
                b: bg_color.2,
                a: self.background_opacity,
            },
            text: iced::Color {
                r: text_color.0,
                g: text_color.1,
                b: text_color.2,
                a: 1.,
            },
            primary: iced::Color {
                r: 0.22,
                g: 0.55,
                b: 0.96,
                a: 1.0,
            },
            danger: iced::Color {
                r: 0.95,
                g: 0.26,
                b: 0.21,
                a: 1.0,
            },
            warning: iced::Color {
                r: 1.0,
                g: 0.76,
                b: 0.03,
                a: 1.0,
            },
            success: iced::Color {
                r: 0.30,
                g: 0.69,
                b: 0.31,
                a: 1.0,
            },
        };
        iced::Theme::Custom(Arc::new(Custom::new("RustCast Theme".to_string(), palette)))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Buffer {
    pub clear_on_hide: bool,
    pub clear_on_enter: bool,
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer {
            clear_on_hide: true,
            clear_on_enter: true,
        }
    }
}

/// Command is the command it will run when the button is clicked
/// Icon_path is the path to an icon, but this is optional
/// Alias is the text that is used to call this command / search for it
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Shelly {
    command: String,
    icon_path: Option<String>,
    alias: String,
    alias_lc: String,
}

impl Shelly {
    pub fn to_app(&self) -> App {
        let self_clone = self.clone();
        let icon = self_clone.icon_path.and_then(|x| {
            let x = x.replace("~", &std::env::var("HOME").unwrap());
            if x.ends_with(".icns") {
                handle_from_icns(Path::new(&x))
            } else {
                Some(Handle::from_path(Path::new(&x)))
            }
        });
        App {
            open_command: Function::RunShellCommand(self_clone.command),
            icons: icon,
            name: self_clone.alias,
            name_lc: self_clone.alias_lc,
        }
    }
}
