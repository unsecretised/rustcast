use crate::macos;
use crate::macos::focus_this_app;

use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use iced::Fill;
use iced::alignment::Vertical;
use iced::futures;
use iced::keyboard;
use iced::keyboard::key::Named;
use iced::stream;
use iced::widget::Button;
use iced::widget::Row;
use iced::widget::Text;
use iced::widget::text::LineHeight;
use iced::widget::{Column, operation, space, text_input};
use iced::window::{self, Id, Settings};
use iced::{Element, Subscription, Task, Theme};
use objc2::rc::Retained;
use objc2_app_kit::NSRunningApplication;

use std::cmp::min;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::exit;
use std::time::Duration;

const WINDOW_WIDTH: f32 = 500.;
const DEFAULT_WINDOW_HEIGHT: f32 = 65.;
const ERR_LOG_PATH: &str = "/tmp/rustscan-err.log";

fn log_error(msg: &str) {
    if let Ok(mut file) = File::options().create(true).append(true).open(ERR_LOG_PATH) {
        let _ = file.write_all(msg.as_bytes()).ok();
    }
}

fn log_error_and_exit(msg: &str) {
    log_error(msg);
    exit(-1)
}

pub fn get_installed_apps(dir: impl AsRef<Path>) -> Vec<App> {
    fs::read_dir(dir)
        .unwrap_or_else(|x| {
            log_error_and_exit(&x.to_string());
            exit(-1)
        })
        .filter_map(|x| x.ok())
        .filter_map(|x| {
            let file_type = x.file_type().unwrap_or_else(|e| {
                log_error(&e.to_string());
                exit(-1)
            });

            if !file_type.is_dir() {
                return None;
            }

            let file_name_os = x.file_name();
            let file_name = file_name_os.into_string().unwrap_or_else(|e| {
                log_error(e.to_str().unwrap_or(""));
                exit(-1)
            });

            if !file_name.ends_with(".app") {
                return None;
            }

            let path_str = x.path().to_str().map(|x| x.to_string()).unwrap_or_else(|| {
                log_error("Unable to get file_name");
                exit(-1)
            });

            let name = file_name.strip_suffix(".app").unwrap().to_string();

            Some(App {
                open_command: format!("open {}", path_str),
                icon_path: None,
                name,
            })
        })
        .collect()
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct App {
    open_command: String,
    icon_path: Option<PathBuf>,
    name: String,
}

impl App {
    pub fn render(&self) -> impl Into<iced::Element<'_, Message>> {
        let mut tile = Row::new().width(Fill).height(55);

        tile = tile.push(
            Button::new(
                Text::new(self.name.clone())
                    .height(Fill)
                    .width(Fill)
                    .align_y(Vertical::Center),
            )
            .on_press(Message::RunShellCommand(self.open_command.clone()))
            .width(Fill)
            .height(Fill),
        );

        tile
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenWindow,
    SearchQueryChanged(String, Id),
    KeyPressed(u32),
    HideWindow(Id),
    _Nothing,
    RunShellCommand(String),
    ClearSearchResults,
}

#[derive(Debug, Clone)]
pub enum Hotkeys {
    AltSpace,
    Nothing,
}

impl Hotkeys {
    pub fn from_u32_hotkey_id(id: u32) -> Hotkeys {
        match id {
            65598 => Hotkeys::AltSpace,
            _ => Hotkeys::Nothing,
        }
    }
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
    query: String,
    theme: Theme,
    results: Vec<App>,
    options: Vec<App>,
    visible: bool,
    frontmost: Option<Retained<NSRunningApplication>>,
}

impl Tile {
    /// A base window
    pub fn new() -> (Self, Task<Message>) {
        let (id, open) = window::open(default_settings());
        let _ = window::run(id, |handle| {
            macos::macos_window_config(
                &handle.window_handle().expect("Unable to get window handle"),
            );
        });

        let mut apps = get_installed_apps("/Applications/");
        apps.append(&mut get_installed_apps("/System/Applications/"));
        apps.append(&mut get_installed_apps("/System/Applications/Utilities/"));
        apps.sort_by_key(|x| x.name.len());

        (
            Self {
                theme: Theme::KanagawaWave,
                query: String::new(),
                results: vec![],
                options: apps,
                visible: true,
                frontmost: None,
            },
            Task::batch([open.map(|_| Message::OpenWindow)]),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenWindow => {
                self.capture_frontmost();
                focus_this_app();
                Task::none()
            }

            Message::SearchQueryChanged(input, id) => {
                self.query = input;
                let mut results = vec![];

                results.append(
                    &mut self
                        .options
                        .iter()
                        .filter(|x| {
                            x.name.trim().to_lowercase() == self.query.trim().to_lowercase()
                        })
                        .map(|x| x.to_owned())
                        .collect(),
                );

                results.append(
                    &mut self
                        .options
                        .iter()
                        .filter(|x| {
                            x.name.to_lowercase() != self.query.to_lowercase()
                                && x.name.to_lowercase().contains(&self.query.to_lowercase())
                        })
                        .map(|x| x.to_owned())
                        .collect(),
                );

                self.results = results;

                let query_count = self.query.chars().count();
                if query_count == 0 {
                    self.results = vec![];
                }

                let max_elem = min(5, self.results.len());
                window::resize(
                    id,
                    iced::Size {
                        width: WINDOW_WIDTH,
                        height: ((max_elem * 55) + DEFAULT_WINDOW_HEIGHT as usize) as f32,
                    },
                )
            }

            Message::KeyPressed(hk_id) => match Hotkeys::from_u32_hotkey_id(hk_id) {
                Hotkeys::AltSpace => {
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
                        to_close.map(Message::HideWindow)
                    }
                }
                _ => Task::none(),
            },

            Message::RunShellCommand(shell_command) => {
                let cmd = shell_command.split_once(" ").unwrap_or(("", ""));
                Command::new(cmd.0).arg(cmd.1).spawn().ok();
                window::latest()
                    .map(|x| x.unwrap())
                    .map(Message::HideWindow)
            }

            Message::HideWindow(a) => {
                self.restore_frontmost();
                self.visible = false;
                Task::batch([window::close(a), Task::done(Message::ClearSearchResults)])
            }
            Message::ClearSearchResults => {
                self.results = vec![];
                Task::none()
            }

            Message::_Nothing => Task::none(),
        }
    }

    pub fn view(&self, wid: window::Id) -> Element<'_, Message> {
        if self.visible {
            let title_input = text_input("Time to be productive!", &self.query)
                .on_input(move |a| Message::SearchQueryChanged(a, wid))
                .on_paste(move |a| Message::SearchQueryChanged(a, wid))
                .on_submit({
                    if self.results.is_empty() {
                        Message::_Nothing
                    } else {
                        Message::RunShellCommand(
                            self.results.first().unwrap().to_owned().open_command,
                        )
                    }
                })
                .id("query")
                .width(Fill)
                .padding(20)
                .line_height(LineHeight::Relative(1.5));

            let mut search_results = Column::new();
            for result in &self.results {
                search_results = search_results.push((result).render());
            }

            Column::new().push(title_input).push(search_results).into()
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
            window::close_events().map(Message::HideWindow),
            window::resize_events().map(|_| Message::_Nothing),
            keyboard::listen().filter_map(|event| {
                if let keyboard::Event::KeyPressed {
                    key,
                    ..
                } = event
                {
                    match key {
                        keyboard::Key::Named(Named::Escape) => Some(Message::KeyPressed(65598)),
                        _ => None,
                    }
                } else {
                    None
                }
            }),
        ])
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
