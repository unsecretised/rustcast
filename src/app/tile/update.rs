//! This handles the update logic for the tile (AKA rustcast's main window)
use std::cmp::min;
use std::fs;
#[cfg(target_os = "macos")]
use std::path::Path;
use std::thread;

use iced::Task;
#[cfg(target_os = "macos")]
use iced::widget::image::Handle;
use iced::widget::operation;
use iced::widget::operation::AbsoluteOffset;
use iced::window;
use rayon::slice::ParallelSliceMut;

use crate::app::ArrowKey;
use crate::app::DEFAULT_WINDOW_HEIGHT;
use crate::app::Move;
use crate::app::RUSTCAST_DESC_NAME;
use crate::app::WINDOW_WIDTH;
use crate::app::apps::App;
use crate::app::apps::AppCommand;
use crate::app::default_settings;
use crate::app::menubar::menu_icon;
use crate::app::tile::AppIndex;
#[cfg(target_os = "windows")]
use crate::app::tile::elm::default_app_paths;
use crate::app::{Message, Page, tile::Tile};

#[cfg(target_os = "windows")]
use crate::utils::get_config_installation_dir;

use crate::calculator::Expr;
use crate::clipboard::ClipBoardContentType;
use crate::commands::Function;
use crate::config::Config;
use crate::unit_conversion;
use crate::utils::get_installed_apps;

use crate::utils::is_valid_url;
#[cfg(target_os = "macos")]
use crate::{
    cross_platform::macos::focus_this_app,
    cross_platform::macos::haptics::{HapticPattern, perform_haptic},
};

pub fn handle_update(tile: &mut Tile, message: Message) -> Task<Message> {
    match message {
        Message::OpenWindow => {
            #[cfg(target_os = "macos")]
            {
                tile.capture_frontmost();
                focus_this_app();
            }
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
            if tile.query_lc.is_empty() {
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

        Message::SearchQueryChanged(input, id) => {
            tile.focus_id = 0;
            #[cfg(target_os = "macos")]
            if tile.config.haptic_feedback {
                perform_haptic(HapticPattern::Alignment);
            }

            tile.query_lc = input.trim().to_lowercase();
            tile.query = input;
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
            } else if tile.query_lc == "67" {
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
            } else if tile.query_lc.ends_with("?") {
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
            } else if tile.results.is_empty() && is_valid_url(&tile.query) {
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
                tile.results.push(App {
                    open_command: AppCommand::Display,
                    desc: "Easter Egg".to_string(),
                    icons: Some(Handle::from_path(Path::new(
                        "/Applications/Rustcast.app/Contents/Resources/lemon.png",
                    ))),
                    name: "Lemon".to_string(),
                    name_lc: "".to_string(),
                });
            }
            let new_length = tile.results.len();

            let max_elem = min(5, new_length);

            if tile.results
                == vec![App {
                    open_command: AppCommand::Message(Message::SwitchToPage(
                        Page::ClipboardHistory,
                    )),
                    desc: RUSTCAST_DESC_NAME.to_string(),
                    icons: None,
                    name: "Clipboard History".to_string(),
                    name_lc: "clipboard".to_string(),
                }]
            {
                tile.page = Page::ClipboardHistory
            }

            if prev_size != new_length && tile.page != Page::ClipboardHistory {
                Task::batch([
                    window::resize(
                        id,
                        iced::Size {
                            width: WINDOW_WIDTH,
                            height: ((max_elem * 70) + DEFAULT_WINDOW_HEIGHT as usize) as f32,
                        },
                    ),
                    Task::done(Message::ChangeFocus(ArrowKey::Left)),
                ])
            } else if tile.page == Page::ClipboardHistory {
                let element_count = min(tile.clipboard_content.len(), 5);
                window::resize(
                    id,
                    iced::Size {
                        width: WINDOW_WIDTH,
                        height: ((element_count * 70) + DEFAULT_WINDOW_HEIGHT as usize) as f32,
                    },
                )
            } else {
                Task::none()
            }
        }

        Message::ClearSearchQuery => {
            tile.query_lc = String::new();
            tile.query = String::new();
            Task::none()
        }

        Message::ChangeFocus(key) => {
            let u32_len = tile.results.len() as u32;
            if u32_len > 0 {
                match key {
                    ArrowKey::Down => tile.focus_id = (tile.focus_id + 1) % u32_len,
                    ArrowKey::Up => tile.focus_id = (tile.focus_id + u32_len - 1) % u32_len,
                    _ => {}
                }

                operation::scroll_to(
                    "results",
                    AbsoluteOffset {
                        x: None,
                        y: Some(tile.focus_id as f32 * 55.),
                    },
                )
            } else {
                Task::none()
            }
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
            #[cfg(target_os = "macos")]
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

            #[cfg(target_os = "windows")]
            let new_config: Config = toml::from_str(
                &fs::read_to_string(get_config_installation_dir() + "/rustcast/config.toml")
                    .unwrap_or("".to_owned()),
            )
            .unwrap();

            let mut new_options: Vec<App> = get_installed_apps(&new_config);

            new_options.extend(new_config.shells.iter().map(|x| x.to_app()));
            new_options.extend(App::basic_apps());
            new_options.par_sort_by_key(|x| x.name.len());

            tile.theme = new_config.theme.to_owned().into();
            tile.config = new_config;
            tile.options = AppIndex::from_apps(new_options);
            Task::none()
        }

        Message::KeyPressed(hk_id) => {
            if hk_id == tile.hotkey.id {
                tile.visible = !tile.visible;
                if tile.visible {
                    #[cfg(target_os = "windows")]
                    {
                        // get normal settings and modify position

                        use crate::cross_platform::windows::open_on_focused_monitor;
                        use iced::window::Position;

                        let pos = open_on_focused_monitor();
                        let mut settings = default_settings();
                        settings.position = Position::Specific(pos);
                        Task::chain(
                            window::open(settings).1.map(|_| Message::OpenWindow),
                            operation::focus("query"),
                        )
                    }

                    #[cfg(target_os = "macos")]
                    {
                        Task::chain(
                            window::open(default_settings())
                                .1
                                .map(|_| Message::OpenWindow),
                            operation::focus("query"),
                        )
                    }
                } else {
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
                }
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
                    tile.query_lc += &query_char.clone().to_lowercase();
                }
                Move::Back => {
                    tile.query.pop();
                    tile.query_lc.pop();
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

        Message::ClipboardHistory(clip_content) => {
            tile.clipboard_content.insert(0, clip_content);
            Task::none()
        }
    }
}
