use iced::{
    Task,
    window::{self, Id},
};
use std::cmp;

use super::Tile;
use crate::{
    app::{
        ArrowKey, DEFAULT_WINDOW_HEIGHT, Message, Page, RUSTCAST_DESC_NAME, WINDOW_WIDTH,
        apps::{App, AppCommand},
    },
    calculator::Expr,
    clipboard::ClipBoardContentType,
    commands::Function,
    unit_conversion,
};

#[cfg(target_os = "macos")]
use crate::cross_platform::macos::haptics::{HapticPattern, perform_haptic};

pub(super) fn handle_change(tile: &mut Tile, input: &str, id: Id) -> iced::Task<Message> {
    tile.focus_id = 0;
    #[cfg(target_os = "macos")]
    if tile.config.haptic_feedback {
        perform_haptic(HapticPattern::Alignment);
    }

    tile.query_lc = input.trim().to_lowercase();
    tile.query = input.to_string();
    let prev_size = tile.results.len();
    if tile.query_lc.is_empty() && tile.page != Page::ClipboardHistory {
        tile.results = vec![];
        return window::resize(
            id,
            iced::Size {
                width: WINDOW_WIDTH,
                height: DEFAULT_WINDOW_HEIGHT,
            },
        );
    } else if tile.query_lc == "randomvar" {
        let rand_num = rand::random_range(0..100);
        tile.results = vec![App {
            open_command: AppCommand::Function(Function::RandomVar(rand_num)),
            desc: "Easter egg".to_string(),
            icons: None,
            name: rand_num.to_string(),
            name_lc: String::new(),
        }];
        return window::resize(
            id,
            iced::Size {
                width: WINDOW_WIDTH,
                height: 55. + DEFAULT_WINDOW_HEIGHT,
            },
        );
    } else {
        if tile.query_lc == "67" {
            tile.results = vec![App {
                open_command: AppCommand::Function(Function::RandomVar(67)),
                desc: "Easter egg".to_string(),
                icons: None,
                name: 67.to_string(),
                name_lc: String::new(),
            }];
            return window::resize(
                id,
                iced::Size {
                    width: WINDOW_WIDTH,
                    height: 55. + DEFAULT_WINDOW_HEIGHT,
                },
            );
        }
        if tile.query_lc.ends_with("?") {
            tile.results = vec![App {
                open_command: AppCommand::Function(Function::GoogleSearch(tile.query.clone())),
                icons: None,
                desc: "Web Search".to_string(),
                name: format!("Search for: {}", tile.query),
                name_lc: String::new(),
            }];
            return window::resize(
                id,
                iced::Size::new(WINDOW_WIDTH, 55. + DEFAULT_WINDOW_HEIGHT),
            );
        } else if tile.query_lc == "cbhist" {
            tile.page = Page::ClipboardHistory
        } else if tile.query_lc == "main" {
            tile.page = Page::Main
        }
    }
    tile.handle_search_query_changed();

    if tile.results.is_empty()
        && let Some(res) = Expr::from_str(&tile.query).ok()
    {
        tile.results.push(App {
            open_command: AppCommand::Function(Function::Calculate(res.clone())),
            desc: RUSTCAST_DESC_NAME.to_string(),
            icons: None,
            name: res.eval().map(|x| x.to_string()).unwrap_or("".to_string()),
            name_lc: "".to_string(),
        });
    } else if tile.results.is_empty()
        && let Some(conversions) = unit_conversion::convert_query(&tile.query)
    {
        tile.results = conversions
            .into_iter()
            .map(|conversion| {
                let source = format!(
                    "{} {}",
                    unit_conversion::format_number(conversion.source_value),
                    conversion.source_unit.name
                );
                let target = format!(
                    "{} {}",
                    unit_conversion::format_number(conversion.target_value),
                    conversion.target_unit.name
                );
                App {
                    open_command: AppCommand::Function(Function::CopyToClipboard(
                        ClipBoardContentType::Text(target.clone()),
                    )),
                    desc: source,
                    icons: None,
                    name: target,
                    name_lc: String::new(),
                }
            })
            .collect();
    } else if tile.results.is_empty() && url::Url::parse(input).is_ok() {
        tile.results.push(App {
            open_command: AppCommand::Function(Function::OpenWebsite(tile.query.clone())),
            desc: "Web Browsing".to_string(),
            icons: None,
            name: "Open Website: ".to_string() + &tile.query,
            name_lc: "".to_string(),
        });
    } else if tile.query_lc.split(' ').count() > 1 {
        tile.results.push(App {
            open_command: AppCommand::Function(Function::GoogleSearch(tile.query.clone())),
            icons: None,
            desc: "Web Search".to_string(),
            name: format!("Search for: {}", tile.query),
            name_lc: String::new(),
        });
    } else if tile.results.is_empty() && tile.query_lc == "lemon" {
        #[cfg(target_os = "macos")]
        {
            use std::path::Path;

            tile.results.push(App {
                open_command: AppCommand::Display,
                desc: "Easter Egg".to_string(),
                icons: Some(iced::widget::image::Handle::from_path(Path::new(
                    "/Applications/Rustcast.app/Contents/Resources/lemon.png",
                ))),
                name: "Lemon".to_string(),
                name_lc: "".to_string(),
            });
        }
    }
    if !tile.query_lc.is_empty() && tile.page == Page::EmojiSearch {
        tile.results = tile
            .emoji_apps
            .search_prefix("")
            .map(|x| x.to_owned())
            .collect();
    }

    let new_length = tile.results.len();
    let max_elem = cmp::min(5, new_length);

    if prev_size != new_length && tile.page != Page::ClipboardHistory {
        Task::batch([
            window::resize(
                id,
                iced::Size {
                    width: WINDOW_WIDTH,
                    height: ((max_elem * 55) + 35 + DEFAULT_WINDOW_HEIGHT as usize) as f32,
                },
            ),
            Task::done(Message::ChangeFocus(ArrowKey::Left)),
        ])
    } else if tile.page == Page::ClipboardHistory {
        Task::batch([
            window::resize(
                id,
                iced::Size {
                    width: WINDOW_WIDTH,
                    height: ((7 * 55) + 35 + DEFAULT_WINDOW_HEIGHT as usize) as f32,
                },
            ),
            Task::done(Message::ChangeFocus(ArrowKey::Left)),
        ])
    } else {
        Task::none()
    }
}
