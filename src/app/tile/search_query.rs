use iced::{
    Task,
    window::{self, Id},
};
use std::cmp;

use super::Tile;
use crate::{
    app::{
        ArrowKey, DEFAULT_WINDOW_HEIGHT, Message, Page, WINDOW_WIDTH,
        apps::{App, AppCommand},
    },
    calculator::Expr,
    clipboard::ClipBoardContentType,
    commands::Function,
    unit_conversion,
};

#[cfg(target_os = "macos")]
use crate::cross_platform::macos::haptics::{HapticPattern, perform_haptic};

#[allow(clippy::too_many_lines)]
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
        tile.results = vec![App::new_builtin(
            &rand_num.to_string(),
            "",
            "Easter egg",
            AppCommand::Function(Function::RandomVar(rand_num)),
        )];
        return window::resize(
            id,
            iced::Size {
                width: WINDOW_WIDTH,
                height: 55. + DEFAULT_WINDOW_HEIGHT,
            },
        );
    }
    if tile.query_lc == "67" {
        tile.results = vec![App::new_builtin(
            "67",
            "",
            "Easter egg",
            AppCommand::Function(Function::RandomVar(67)),
        )];
        return window::resize(
            id,
            iced::Size {
                width: WINDOW_WIDTH,
                height: 55. + DEFAULT_WINDOW_HEIGHT,
            },
        );
    }
    if tile.query_lc.ends_with('?') {
        tile.results = vec![App::new_builtin(
            &format!("Search for: {}", tile.query),
            "",
            "Web Search",
            AppCommand::Function(Function::GoogleSearch(tile.query.clone())),
        )];
        return window::resize(
            id,
            iced::Size::new(WINDOW_WIDTH, 55. + DEFAULT_WINDOW_HEIGHT),
        );
    } else if tile.query_lc == "cbhist" {
        tile.page = Page::ClipboardHistory;
    } else if tile.query_lc == "main" {
        tile.page = Page::Main;
    }
    tile.handle_search_query_changed();

    if tile.results.is_empty()
        && let Some(res) = Expr::from_str(&tile.query).ok()
    {
        let res_string = res.eval().map_or(String::new(), |x| x.to_string());
        tile.results.push(App::new_builtin(
            &res_string,
            "",
            "Calculation result",
            AppCommand::Function(Function::Calculate(res.clone())),
        ));
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
                App::new_builtin(
                    &source,
                    &target,
                    "Copy to clipboard",
                    AppCommand::Function(Function::CopyToClipboard(ClipBoardContentType::Text(
                        target.clone(),
                    ))),
                )
            })
            .collect();
    } else if tile.results.is_empty() && url::Url::parse(input).is_ok() {
        tile.results.push(App::new_builtin(
            "Web Browsing",
            "",
            &format!("Open website: {}", tile.query),
            AppCommand::Function(Function::OpenWebsite(tile.query.clone())),
        ));
    } else if tile.query_lc.split(' ').count() > 1 {
        tile.results.push(App::new_builtin(
            &format!("Search for: {}", tile.query),
            "",
            "Web Search",
            AppCommand::Function(Function::GoogleSearch(tile.query.clone())),
        ));
    } else if tile.results.is_empty() && tile.query_lc == "lemon" {
        #[cfg(target_os = "macos")]
        {
            use std::path::Path;

            tile.results.push(App::new_builtin(
                "Easter Egg",
                "Lemon",
                "",
                AppCommand::Display,
            ));
        }
    }
    if !tile.query_lc.is_empty() && tile.page == Page::EmojiSearch {
        tile.results = tile
            .emoji_apps
            .search_prefix("")
            .map(std::borrow::ToOwned::to_owned)
            .collect();
    }

    let new_length = tile.results.len();
    let max_elem = cmp::min(5, new_length);

    if prev_size != new_length && tile.page != Page::ClipboardHistory {
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
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
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
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
