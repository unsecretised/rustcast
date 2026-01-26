//! This handles the update logic for the tile (AKA rustcast's main window)
use std::fs;
use std::path::Path;
use std::thread;
use std::{borrow::Cow, cmp::min};

use iced::Task;
use iced::widget::image::Handle;
use iced::widget::operation;
use iced::widget::operation::AbsoluteOffset;
use iced::window;
use rayon::slice::ParallelSliceMut;

use crate::app::WINDOW_WIDTH;
use crate::app::apps::App;
use crate::app::apps::AppCommand;
use crate::app::default_settings;
use crate::app::menubar::menu_icon;
use crate::app::tile::AppIndex;
use crate::app::{Message, Page, tile::Tile};
use crate::calculator::Expr;
use crate::clipboard::ClipBoardContentType;
use crate::commands::Function;
use crate::config::Config;
use crate::unit_conversion;
use crate::utils::is_valid_url;
use crate::{app::ArrowKey, platform::focus_this_app};
use crate::{app::DEFAULT_WINDOW_HEIGHT, platform::perform_haptic};
use crate::{app::Move, platform::HapticPattern};
use crate::{app::RUSTCAST_DESC_NAME, platform::get_installed_apps};

pub fn handle_update(tile: &mut Tile, message: Message) -> Task<Message> {
    match message {
        Message::OpenWindow => {
            tile.capture_frontmost();
            focus_this_app();
            tile.focused = true;
            tile.visible = true;
            Task::none()
        }
        Message::HideTrayIcon => {
            tile.tray_icon = None;
            tile.config.show_trayicon = false;
            let home = std::env::var("HOME").unwrap();
            let confg_str = toml::to_string(&tile.config).unwrap();
            thread::spawn(move || fs::write(home + "/.config/rustcast/config.toml", confg_str));
            Task::none()
        }

        Message::SetSender(sender) => {
            tile.sender = Some(sender.clone());
            if tile.config.show_trayicon {
                tile.tray_icon = Some(menu_icon(tile.hotkey, sender));
            }
            Task::none()
        }

        Message::EscKeyPressed(id) => {
            if tile.page == Page::EmojiSearch && !tile.query.is_empty() {
                return Task::none();
            }

            if tile.query.is_empty() {
                Task::batch([
                    Task::done(Message::HideWindow(id)),
                    Task::done(Message::ReturnFocus),
                ])
            } else {
                tile.page = Page::Main;

                Task::batch(vec![
                    Task::done(Message::ClearSearchQuery),
                    Task::done(Message::ClearSearchResults),
                    window::resize(
                        id,
                        iced::Size {
                            width: WINDOW_WIDTH,
                            height: DEFAULT_WINDOW_HEIGHT,
                        },
                    ),
                ])
            }
        }

        Message::ClearSearchQuery => {
            tile.query = String::new();
            Task::none()
        }

        Message::ChangeFocus(key) => {
            let len = match tile.page {
                Page::ClipboardHistory => tile.clipboard_content.len() as u32,
                Page::EmojiSearch => tile.emoji_apps.search(&tile.query).len() as u32,
                _ => tile.results.len() as u32,
            };

            let old_focus_id = tile.focus_id;

            if len == 0 {
                return Task::none();
            }

            let change_by = match tile.page {
                Page::EmojiSearch => 6,
                _ => 1,
            };

            let task = match (key, &tile.page) {
                (ArrowKey::Down, _) => {
                    tile.focus_id = (tile.focus_id + change_by) % len;
                    Task::none()
                }
                (ArrowKey::Up, _) => {
                    tile.focus_id = (tile.focus_id + len - change_by) % len;
                    Task::none()
                }
                (ArrowKey::Left, Page::EmojiSearch) => {
                    tile.focus_id = (tile.focus_id + len - 1) % len;
                    operation::focus("results")
                }
                (ArrowKey::Right, Page::EmojiSearch) => {
                    tile.focus_id = (tile.focus_id + 1) % len;
                    operation::focus("results")
                }
                _ => Task::none(),
            };

            let direction = if tile.focus_id < old_focus_id { -1 } else { 1 };
            let quantity = match tile.page {
                Page::Main => 66.5,
                Page::ClipboardHistory => 50.,
                Page::EmojiSearch => 5.,
            };

            Task::batch([
                task,
                operation::scroll_to(
                    "results",
                    AbsoluteOffset {
                        x: None,
                        y: Some((tile.focus_id as f32 * quantity) * direction as f32),
                    },
                ),
            ])
        }

        Message::OpenFocused => match tile.results.get(tile.focus_id as usize) {
            Some(App {
                open_command: AppCommand::Function(func),
                ..
            }) => Task::done(Message::RunFunction(func.to_owned())),
            Some(App {
                open_command: AppCommand::Message(msg),
                ..
            }) => Task::done(msg.to_owned()),
            Some(App {
                open_command: AppCommand::Display,
                ..
            }) => Task::done(Message::ReturnFocus),
            None => Task::none(),
        },

        Message::ReloadConfig => {
            let new_config: Config = match toml::from_str(
                &fs::read_to_string(
                    std::env::var("HOME").unwrap_or("".to_owned())
                        + "/.config/rustcast/config.toml",
                )
                .unwrap_or("".to_owned()),
            ) {
                Ok(a) => a,
                Err(_) => return Task::none(),
            };

            let mut new_options = get_installed_apps(new_config.theme.show_icons);
            new_options.extend(new_config.shells.iter().map(|x| x.to_app()));
            new_options.extend(App::basic_apps());
            new_options.par_sort_by_key(|x| x.name.len());

            tile.theme = new_config.theme.to_owned().into();
            tile.config = new_config;
            tile.options = AppIndex::from_apps(new_options);
            Task::none()
        }

        Message::KeyPressed(hk_id) => {
            let is_clipboard_hotkey = tile
                .clipboard_hotkey
                .map(|hotkey| hotkey.id == hk_id)
                .unwrap_or(false);
            let is_open_hotkey = hk_id == tile.hotkey.id;

            let clipboard_page_task = if is_clipboard_hotkey {
                Task::done(Message::SwitchToPage(Page::ClipboardHistory))
            } else if is_open_hotkey {
                Task::done(Message::SwitchToPage(Page::Main))
            } else {
                Task::none()
            };

            if is_open_hotkey || is_clipboard_hotkey {
                if !tile.visible {
                    return Task::batch([open_window(), clipboard_page_task]);
                }

                tile.visible = !tile.visible;

                let clear_search_query = if tile.config.buffer_rules.clear_on_hide {
                    Task::done(Message::ClearSearchQuery)
                } else {
                    Task::none()
                };

                let to_close = window::latest().map(|x| x.unwrap());
                Task::batch([
                    to_close.map(Message::HideWindow),
                    clear_search_query,
                    Task::done(Message::ReturnFocus),
                ])
            } else {
                Task::none()
            }
        }

        Message::SwitchToPage(page) => {
            tile.page = page;
            Task::batch([
                Task::done(Message::ClearSearchQuery),
                Task::done(Message::ClearSearchResults),
            ])
        }

        Message::RunFunction(command) => {
            command.execute(&tile.config, &tile.query);

            let return_focus_task = match &command {
                Function::OpenApp(_) | Function::OpenPrefPane | Function::GoogleSearch(_) => {
                    Task::none()
                }
                _ => Task::done(Message::ReturnFocus),
            };

            if tile.config.buffer_rules.clear_on_enter {
                window::latest()
                    .map(|x| x.unwrap())
                    .map(Message::HideWindow)
                    .chain(Task::done(Message::ClearSearchQuery))
                    .chain(return_focus_task)
            } else {
                Task::none()
            }
        }

        Message::HideWindow(a) => {
            tile.visible = false;
            tile.focused = false;
            tile.page = Page::Main;
            Task::batch([window::close(a), Task::done(Message::ClearSearchResults)])
        }

        Message::ReturnFocus => {
            tile.restore_frontmost();
            Task::none()
        }

        Message::FocusTextInput(update_query_char) => {
            match update_query_char {
                Move::Forwards(query_char) => {
                    tile.query += &query_char.clone();
                }
                Move::Back => {
                    tile.query.pop();
                }
            }
            let updated_query = tile.query.clone();
            Task::batch([
                operation::focus("query"),
                window::latest()
                    .map(|x| x.unwrap())
                    .map(move |x| Message::SearchQueryChanged(updated_query.clone(), x)),
            ])
        }

        Message::ClearSearchResults => {
            tile.results = vec![];
            Task::none()
        }
        Message::WindowFocusChanged(wid, focused) => {
            tile.focused = focused;
            if !focused {
                Task::done(Message::HideWindow(wid)).chain(Task::done(Message::ClearSearchQuery))
            } else {
                Task::none()
            }
        }

        Message::ClipboardHistory(content) => {
            tile.clipboard_content.insert(0, content);
            Task::none()
        }

        Message::SearchQueryChanged(input, id) => {
            tile.focus_id = 0;

            if tile.config.haptic_feedback {
                perform_haptic(HapticPattern::Alignment);
            }

            tile.query = input;
            let prev_size = tile.results.len();
            if tile.query.is_empty() && tile.page != Page::ClipboardHistory {
                tile.results = vec![];
                return window::resize(
                    id,
                    iced::Size {
                        width: WINDOW_WIDTH,
                        height: DEFAULT_WINDOW_HEIGHT,
                    },
                );
            } else if tile.query.eq_ignore_ascii_case("randomvar") {
                let rand_num = rand::random_range(0..100);
                tile.results = vec![App {
                    open_command: AppCommand::Function(Function::RandomVar(rand_num)),
                    desc: Cow::Borrowed("Easter egg"),
                    icons: None,
                    name: Cow::Owned(rand_num.to_string()),
                }];
                return window::resize(
                    id,
                    iced::Size {
                        width: WINDOW_WIDTH,
                        height: 55. + DEFAULT_WINDOW_HEIGHT,
                    },
                );
            } else if tile.query.eq_ignore_ascii_case("67") {
                tile.results = vec![App {
                    open_command: AppCommand::Function(Function::RandomVar(67)),
                    desc: Cow::Borrowed("Easter egg"),
                    icons: None,
                    name: Cow::Borrowed("67"),
                }];
                return window::resize(
                    id,
                    iced::Size {
                        width: WINDOW_WIDTH,
                        height: 55. + DEFAULT_WINDOW_HEIGHT,
                    },
                );
            } else if tile.query.ends_with("?") {
                tile.results = vec![App {
                    open_command: AppCommand::Function(Function::GoogleSearch(tile.query.clone())),
                    icons: None,
                    desc: Cow::Borrowed("Web Search"),
                    name: Cow::Owned(format!("Search for: {}", tile.query)),
                }];
                return window::resize(
                    id,
                    iced::Size::new(WINDOW_WIDTH, 55. + DEFAULT_WINDOW_HEIGHT),
                );
            } else if tile.query.eq_ignore_ascii_case("cbhist") {
                tile.page = Page::ClipboardHistory
            } else if tile.query.eq_ignore_ascii_case("main") {
                tile.page = Page::Main
            }
            tile.handle_search_query_changed();

            if tile.results.is_empty()
                && let Some(expr) = Expr::from_str(&tile.query).ok()
            {
                tile.results.push(App {
                    open_command: AppCommand::Function(Function::Calculate(expr.clone())),
                    desc: Cow::Borrowed(RUSTCAST_DESC_NAME),
                    icons: None,
                    name: expr
                        .eval()
                        .map(|result| Cow::Owned(result.to_string()))
                        .unwrap_or(Cow::Borrowed("")),
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
                                ClipBoardContentType::Text(Cow::Owned(target.clone())),
                            )),
                            desc: Cow::Owned(source),
                            icons: None,
                            name: Cow::Owned(target),
                        }
                    })
                    .collect();
            } else if tile.results.is_empty() && is_valid_url(&tile.query) {
                tile.results.push(App {
                    open_command: AppCommand::Function(Function::OpenWebsite(tile.query.clone())),
                    desc: Cow::Borrowed("Web Browsing"),
                    icons: None,
                    name: Cow::Owned(format!("Open Website: {}", tile.query)),
                });
            } else if tile.query.split_whitespace().count() > 1 {
                tile.results.push(App {
                    open_command: AppCommand::Function(Function::GoogleSearch(tile.query.clone())),
                    icons: None,
                    desc: Cow::Borrowed("Web Search"),
                    name: Cow::Owned(format!("Search for: {}", tile.query)),
                });
            } else if tile.results.is_empty() && tile.query.eq_ignore_ascii_case("lemon") {
                tile.results.push(App {
                    open_command: AppCommand::Display,
                    desc: Cow::Borrowed("Easter Egg"),
                    icons: Some(Handle::from_path(Path::new(
                        "/Applications/Rustcast.app/Contents/Resources/lemon.png",
                    ))),
                    name: Cow::Borrowed("Lemon"),
                });
            }
            if !tile.query.is_empty() && tile.page == Page::EmojiSearch {
                tile.results = tile
                    .emoji_apps
                    .search("")
                    .into_iter()
                    .map(|(_, app)| app.clone())
                    .collect();
            }

            let new_length = tile.results.len();
            let max_elem = min(5, new_length);

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
    }
}

fn open_window() -> Task<Message> {
    Task::chain(
        window::open(default_settings())
            .1
            .map(|_| Message::OpenWindow),
        operation::focus("query"),
    )
}
