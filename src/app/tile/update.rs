//! This handles the update logic for the tile (AKA rustcast's main window)
use std::fs;
use std::thread;

use iced::Task;
use iced::widget::{operation, operation::AbsoluteOffset};
use iced::window;
use rayon::slice::ParallelSliceMut;

use crate::app::apps::AppData;
use crate::app::{
    ArrowKey, DEFAULT_WINDOW_HEIGHT, Message, Move, Page, WINDOW_WIDTH, apps::App,
    apps::AppCommand, default_settings, menubar::menu_icon, tile::AppIndex, tile::Tile,
    tile::search_query,
};

#[cfg(target_os = "macos")]
use crate::cross_platform::macos;

use crate::commands::Function;
use crate::config::Config;
use crate::utils::index_installed_apps;

pub fn handle_update(tile: &mut Tile, message: Message) -> Task<Message> {
    tracing::trace!(target: "update", "{:?}", message);

    match message {
        Message::OpenWindow => {
            #[cfg(target_os = "macos")]
            {
                tile.capture_frontmost();
                macos::focus_this_app();
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
                // Tray icon seems to not work on linux (gdk only but this is wgpu?)
                // I do not know so much abt rendering stuff
                #[cfg(not(target_os = "linux"))]
                {
                    tile.tray_icon = Some(menu_icon(
                        #[cfg(not(target_os = "linux"))]
                        tile.hotkey,
                        sender,
                    ));
                }
            }
            Task::none()
        }

        Message::EscKeyPressed(id) => {
            if tile.page == Page::EmojiSearch && !tile.query_lc.is_empty() {
                return Task::none();
            }

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

        Message::ClearSearchQuery => {
            tile.query_lc = String::new();
            tile.query = String::new();
            Task::none()
        }

        Message::ChangeFocus(key) => {
            let len = match tile.page {
                Page::ClipboardHistory => tile.clipboard_content.len() as u32,
                Page::EmojiSearch => tile.emoji_apps.search_prefix(&tile.query_lc).count() as u32, // or tile.results.len()
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

        Message::OpenFocused => match tile.results.get(tile.focus_id as usize).map(|x| &x.data) {
            Some(AppData::Builtin {
                command: AppCommand::Function(func),
                ..
            }) => Task::done(Message::RunFunction(func.to_owned())),
            Some(AppData::Builtin {
                command: AppCommand::Message(msg),
                ..
            }) => Task::done(msg.to_owned()),
            Some(AppData::Builtin {
                command: AppCommand::Display,
                ..
            }) => Task::done(Message::ReturnFocus),
            _ => Task::none(),
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
            let mut options = Vec::new();

            match index_installed_apps(&new_config) {
                Ok(x) => options.extend(x),
                Err(e) => tracing::error!("Error indexing apps: {e}"),
            }

            options.extend(new_config.shells.iter().map(|x| x.to_app()));
            options.extend(App::basic_apps());
            options.par_sort_by_key(|x| x.name.len());

            tile.theme = new_config.theme.to_owned().into();
            tile.config = new_config;
            tile.options = AppIndex::from_apps(options);
            Task::none()
        }

        Message::OpenToPage(page) => {
            if !tile.visible {
                return Task::batch([open_window(), Task::done(Message::SwitchToPage(page))]);
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
        }

        Message::KeyPressed(_) => Task::none(),

        #[cfg(not(target_os = "linux"))]
        Message::HotkeyPressed(hk_id) => {
            // Linux Clipboard and Open Hotkey are gonna be handled via a socket
            let is_clipboard_hotkey = tile
                .clipboard_hotkey
                .map(|hotkey| hotkey.id == hk_id)
                .unwrap_or(false);

            let is_open_hotkey = hk_id == tile.hotkey.id;

            if is_clipboard_hotkey {
                handle_update(tile, Message::OpenToPage(Page::ClipboardHistory))
            } else if is_open_hotkey {
                handle_update(tile, Message::OpenToPage(Page::Main))
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
                if cfg!(target_os = "macos") {
                    Task::done(Message::HideWindow(wid))
                        .chain(Task::done(Message::ClearSearchQuery))
                } else {
                    // linux seems to not wanna unfocus it on start making it not show
                    Task::none()
                }
            } else {
                Task::none()
            }
        }

        Message::ClipboardHistory(content) => {
            tile.clipboard_content.insert(0, content);
            Task::none()
        }

        Message::SearchQueryChanged(input, id) => search_query::handle_change(tile, &input, id),
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
