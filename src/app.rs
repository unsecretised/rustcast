use crate::commands::Function;
use crate::config::Config;
use crate::macos::{focus_this_app, transform_process_to_ui_element};
use crate::{macos, utils::get_installed_apps};

use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use iced::futures::SinkExt;
use iced::{
    Alignment, Element, Fill, Subscription, Task, Theme,
    alignment::Vertical,
    futures,
    keyboard::{self, key::Named},
    stream,
    widget::{
        Button, Column, Row, Text, container, image::Viewer, operation, scrollable, space,
        text::LineHeight, text_input,
    },
    window::{self, Id, Settings},
};

use objc2::rc::Retained;
use objc2_app_kit::NSRunningApplication;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;

use std::cmp::min;
use std::fs;
use std::time::Duration;

pub const WINDOW_WIDTH: f32 = 500.;
pub const DEFAULT_WINDOW_HEIGHT: f32 = 65.;

#[derive(Debug, Clone)]
pub struct App {
    pub open_command: Function,
    pub icons: Option<iced::widget::image::Handle>,
    pub name: String,
    pub name_lc: String,
}

impl App {
    pub fn basic_apps() -> Vec<App> {
        vec![
            App {
                open_command: Function::Quit,
                icons: None,
                name: "Quit RustCast".to_string(),
                name_lc: "quit".to_string(),
            },
            App {
                open_command: Function::OpenPrefPane,
                icons: None,
                name: "Open RustCast Preferences".to_string(),
                name_lc: "settings".to_string(),
            },
        ]
    }
    pub fn render(&self, show_icons: bool) -> impl Into<iced::Element<'_, Message>> {
        let mut tile = Row::new().width(Fill).height(55);

        if show_icons {
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

        tile = tile
            .push(
                Button::new(
                    Text::new(self.name.clone())
                        .height(Fill)
                        .width(Fill)
                        .align_y(Vertical::Center),
                )
                .on_press(Message::RunFunction(self.open_command.clone()))
                .style(|_, _| iced::widget::button::Style {
                    background: Some(iced::Background::Color(
                        Theme::KanagawaDragon.palette().background,
                    )),
                    text_color: Theme::KanagawaDragon.palette().text,
                    ..Default::default()
                })
                .width(Fill)
                .height(55),
            )
            .width(Fill);
        container(tile)
            .style(|_| iced::widget::container::Style {
                text_color: Some(Theme::KanagawaDragon.palette().text),
                background: Some(iced::Background::Color(
                    Theme::KanagawaDragon.palette().background,
                )),
                ..Default::default()
            })
            .width(Fill)
            .height(Fill)
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenWindow,
    SearchQueryChanged(String, Id),
    KeyPressed(u32),
    HideWindow(Id),
    RunFunction(Function),
    ClearSearchResults,
    WindowFocusChanged(Id, bool),
    ClearSearchQuery,
    ReloadConfig,
    _Nothing,
}

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

#[derive(Debug, Clone)]
pub struct Tile {
    theme: iced::Theme,
    query: String,
    query_lc: String,
    prev_query_lc: String,
    results: Vec<App>,
    options: Vec<App>,
    visible: bool,
    focused: bool,
    frontmost: Option<Retained<NSRunningApplication>>,
    config: Config,
    open_hotkey_id: u32,
}

impl Tile {
    /// A base window
    pub fn new(keybind_id: u32, config: &Config) -> (Self, Task<Message>) {
        let (id, open) = window::open(default_settings());

        let open = open.discard().chain(window::run(id, |handle| {
            macos::macos_window_config(
                &handle.window_handle().expect("Unable to get window handle"),
            );
            // should work now that we have a window
            transform_process_to_ui_element();
        }));

        let store_icons = config.theme.show_icons;

        let user_local_path = std::env::var("HOME").unwrap() + "/Applications/";

        let paths = vec![
            "/Applications/",
            user_local_path.as_str(),
            "/System/Applications/",
            "/System/Applications/Utilities/",
        ];

        let mut options: Vec<App> = paths
            .par_iter()
            .map(|path| get_installed_apps(path, store_icons))
            .flatten()
            .collect();

        options.extend(config.shells.iter().map(|x| x.to_app()));
        options.extend(App::basic_apps());
        options.par_sort_by_key(|x| x.name.len());

        (
            Self {
                query: String::new(),
                query_lc: String::new(),
                prev_query_lc: String::new(),
                results: vec![],
                options,
                visible: true,
                frontmost: None,
                focused: false,
                config: config.clone(),
                theme: config.theme.to_owned().to_iced_theme(),
                open_hotkey_id: keybind_id,
            },
            Task::batch([open.map(|_| Message::OpenWindow)]),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenWindow => {
                self.capture_frontmost();
                focus_this_app();
                self.focused = true;
                Task::none()
            }

            Message::SearchQueryChanged(input, id) => {
                self.query_lc = input.trim().to_lowercase();
                self.query = input;
                let prev_size = self.results.len();
                if self.query_lc.is_empty() {
                    self.results = vec![];
                    return window::resize(
                        id,
                        iced::Size {
                            width: WINDOW_WIDTH,
                            height: DEFAULT_WINDOW_HEIGHT,
                        },
                    );
                } else if self.query_lc == "randomvar" {
                    let rand_num = rand::random_range(0..100);
                    self.results = vec![App {
                        open_command: Function::RandomVar(rand_num),
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
                } else if self.query_lc.ends_with("?") {
                    self.results = vec![App {
                        open_command: Function::GoogleSearch(self.query.clone()),
                        icons: None,
                        name: format!("Search for: {}", self.query),
                        name_lc: String::new(),
                    }];
                    return window::resize(
                        id,
                        iced::Size::new(WINDOW_WIDTH, 55. + DEFAULT_WINDOW_HEIGHT),
                    );
                }

                self.handle_search_query_changed();
                let new_length = self.results.len();

                let max_elem = min(5, new_length);
                if prev_size != new_length {
                    window::resize(
                        id,
                        iced::Size {
                            width: WINDOW_WIDTH,
                            height: ((max_elem * 55) + DEFAULT_WINDOW_HEIGHT as usize) as f32,
                        },
                    )
                } else {
                    Task::none()
                }
            }

            Message::ClearSearchQuery => {
                self.query_lc = String::new();
                self.query = String::new();
                Task::none()
            }

            Message::ReloadConfig => {
                self.config = toml::from_str(
                    &fs::read_to_string(
                        std::env::var("HOME").unwrap_or("".to_owned())
                            + "/.config/rustcast/config.toml",
                    )
                    .unwrap_or("".to_owned()),
                )
                .unwrap();

                Task::none()
            }

            Message::KeyPressed(hk_id) => {
                if hk_id == self.open_hotkey_id {
                    self.visible = !self.visible;
                    if self.visible {
                        Task::chain(
                            window::open(default_settings())
                                .1
                                .map(|_| Message::OpenWindow),
                            operation::focus("query"),
                        )
                    } else {
                        let to_close = window::latest().map(|x| x.unwrap());
                        Task::batch([
                            to_close.map(Message::HideWindow),
                            Task::done(if self.config.buffer_rules.clone().clear_on_hide {
                                Message::ClearSearchQuery
                            } else {
                                Message::_Nothing
                            }),
                        ])
                    }
                } else {
                    Task::none()
                }
            }

            Message::RunFunction(command) => {
                command.execute(&self.config, &self.query);

                if self.config.buffer_rules.clear_on_enter {
                    window::latest()
                        .map(|x| x.unwrap())
                        .map(Message::HideWindow)
                        .chain(Task::done(Message::ClearSearchQuery))
                } else {
                    Task::none()
                }
            }

            Message::HideWindow(a) => {
                self.restore_frontmost();
                self.visible = false;
                self.focused = false;
                Task::batch([window::close(a), Task::done(Message::ClearSearchResults)])
            }
            Message::ClearSearchResults => {
                self.results = vec![];
                Task::none()
            }
            Message::WindowFocusChanged(wid, focused) => {
                self.focused = focused;
                if !focused {
                    Task::done(Message::HideWindow(wid))
                        .chain(Task::done(Message::ClearSearchQuery))
                } else {
                    Task::none()
                }
            }

            Message::_Nothing => Task::none(),
        }
    }

    pub fn view(&self, wid: window::Id) -> Element<'_, Message> {
        if self.visible {
            let title_input = text_input(self.config.placeholder.as_str(), &self.query)
                .on_input(move |a| Message::SearchQueryChanged(a, wid))
                .on_paste(move |a| Message::SearchQueryChanged(a, wid))
                .on_submit({
                    if self.results.is_empty() {
                        Message::_Nothing
                    } else {
                        Message::RunFunction(self.results.first().unwrap().to_owned().open_command)
                    }
                })
                .id("query")
                .width(Fill)
                .padding(20)
                .line_height(LineHeight::Relative(1.5));

            let mut search_results = Column::new();
            for result in &self.results {
                search_results =
                    search_results.push(result.render(self.config.theme.clone().show_icons));
            }

            Column::new()
                .push(title_input)
                .push(scrollable(search_results))
                .into()
        } else {
            space().into()
        }
    }

    pub fn theme(&self, _: window::Id) -> Option<Theme> {
        Some(self.theme.clone())
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            Subscription::run(handle_hotkeys),
            Subscription::run(handle_hot_reloading),
            window::close_events().map(Message::HideWindow),
            keyboard::listen().filter_map(|event| {
                if let keyboard::Event::KeyPressed { key, .. } = event {
                    match key {
                        keyboard::Key::Named(Named::Escape) => Some(Message::KeyPressed(65598)),
                        _ => None,
                    }
                } else {
                    None
                }
            }),
            window::events()
                .with(self.focused)
                .filter_map(|(focused, (wid, event))| match event {
                    window::Event::Unfocused => {
                        if focused {
                            Some(Message::WindowFocusChanged(wid, false))
                        } else {
                            None
                        }
                    }
                    window::Event::Focused => Some(Message::WindowFocusChanged(wid, true)),
                    _ => None,
                }),
        ])
    }

    pub fn handle_search_query_changed(&mut self) {
        let filter_vec: &Vec<App> = if self.query_lc.starts_with(&self.prev_query_lc) {
            self.prev_query_lc = self.query_lc.to_owned();
            &self.results
        } else {
            &self.options
        };

        let query = self.query_lc.clone();

        let mut exact: Vec<App> = filter_vec
            .par_iter()
            .filter(|x| match &x.open_command {
                Function::RunShellCommand(_) => x
                    .name_lc
                    .starts_with(query.split_once(" ").unwrap_or((&query, "")).0),
                _ => x.name_lc == query,
            })
            .cloned()
            .collect();

        let mut prefix: Vec<App> = filter_vec
            .par_iter()
            .filter(|x| match x.open_command {
                Function::RunShellCommand(_) => false,
                _ => x.name_lc != query && x.name_lc.starts_with(&query),
            })
            .cloned()
            .collect();

        exact.append(&mut prefix);
        self.results = exact;
    }

    pub fn capture_frontmost(&mut self) {
        use objc2_app_kit::NSWorkspace;

        let ws = NSWorkspace::sharedWorkspace();
        self.frontmost = ws.frontmostApplication();
    }

    #[allow(deprecated)]
    pub fn restore_frontmost(&mut self) {
        use objc2_app_kit::NSApplicationActivationOptions;

        if let Some(app) = self.frontmost.take() {
            app.activateWithOptions(NSApplicationActivationOptions::ActivateIgnoringOtherApps);
        }
    }
}

fn handle_hot_reloading() -> impl futures::Stream<Item = Message> {
    stream::channel(100, async |mut output| {
        let content = fs::read_to_string(
            std::env::var("HOME").unwrap_or("".to_owned()) + "/.config/rustcast/config.toml",
        )
        .unwrap_or("".to_string());
        loop {
            let current_content = fs::read_to_string(
                std::env::var("HOME").unwrap_or("".to_owned()) + "/.config/rustcast/config.toml",
            )
            .unwrap_or("".to_string());

            if current_content != content {
                output.send(Message::ReloadConfig).await.unwrap();
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
}

fn handle_hotkeys() -> impl futures::Stream<Item = Message> {
    stream::channel(100, async |mut output| {
        let receiver = GlobalHotKeyEvent::receiver();
        loop {
            if let Ok(event) = receiver.recv()
                && event.state == HotKeyState::Pressed
            {
                output.try_send(Message::KeyPressed(event.id)).unwrap();
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
}
