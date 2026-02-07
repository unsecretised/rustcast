//! This module handles the logic for the new and view functions according to the elm
//! architecture. If the subscription function becomes too large, it should be moved to this file

#[cfg(not(target_os = "linux"))]
use global_hotkey::hotkey::HotKey;
use iced::border::Radius;
use iced::widget::scrollable::{Anchor, Direction, Scrollbar};
use iced::widget::text::LineHeight;
use iced::widget::{Column, Row, Scrollable, Text, container, space};
use iced::{Alignment, Color, Length, Vector, window};
use iced::{Element, Task};
use iced::{Length::Fill, widget::text_input};

use rayon::slice::ParallelSliceMut;

#[cfg(target_os = "windows")]
use crate::app;
use crate::app::WINDOW_WIDTH;
use crate::app::pages::clipboard::clipboard_view;
use crate::app::pages::emoji::emoji_page;
use crate::app::tile::AppIndex;
use crate::config::Theme;
use crate::styles::{contents_style, rustcast_text_input_style, tint, with_alpha};
use crate::utils::index_installed_apps;
use crate::{
    app::{Message, Page, apps::App, default_settings, tile::Tile},
    config::Config,
};

#[cfg(target_os = "macos")]
use crate::cross_platform::macos::{self, transform_process_to_ui_element};

pub fn default_app_paths() -> Vec<String> {
    #[cfg(target_os = "macos")]
    {
        let user_local_path = std::env::var("HOME").unwrap() + "/Applications/";

        let paths = vec![
            "/Applications/".to_string(),
            user_local_path,
            "/System/Applications/".to_string(),
            "/System/Applications/Utilities/".to_string(),
        ];
        paths
    }

    #[cfg(target_os = "windows")]
    {
        Vec::new()
    }

    #[cfg(target_os = "linux")]
    {
        use std::path::PathBuf;

        let mut dirs = Vec::new();

        let user_dir: PathBuf = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".local/share"));
        dirs.push(user_dir.join("applications").to_string_lossy().to_string());

        let sys_dirs = std::env::var("XDG_DATA_DIRS")
            .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());

        for dir in sys_dirs.split(':') {
            dirs.push(PathBuf::from(dir).to_string_lossy().to_string());
        }

        dirs
    }
}

/// Initialise the base window
pub fn new(
    #[cfg(not(target_os = "linux"))] hotkey: HotKey,
    config: &Config,
) -> (Tile, Task<Message>) {
    #[allow(unused_mut)]
    let mut settings = default_settings();

    // get normal settings and modify position
    #[cfg(target_os = "windows")]
    {
        use iced::window::Position;

        use crate::cross_platform::windows::open_on_focused_monitor;
        let pos = open_on_focused_monitor();
        settings.position = Position::Specific(pos);
    }

    // id unused on windows, but not macos
    #[allow(unused)]
    let (id, open) = window::open(settings);

    #[cfg(target_os = "windows")]
    let open: Task<app::Message> = open.discard();

    #[cfg(target_os = "linux")]
    let open = open
        .discard()
        .chain(window::run(id, |_| Message::OpenWindow));

    #[cfg(target_os = "macos")]
    let open = open.discard().chain(window::run(id, |handle| {
        macos::macos_window_config(&handle.window_handle().expect("Unable to get window handle"));
        transform_process_to_ui_element();
        Message::OpenWindow
    }));

    let options = index_installed_apps(config);

    if let Err(ref e) = options {
        tracing::error!("Error indexing apps: {e}")
    }

    // Still try to load the rest
    let mut options = options.unwrap_or_default();

    options.extend(config.shells.iter().map(|x| x.to_app()));
    options.extend(App::basic_apps());
    options.par_sort_by_key(|x| x.name.len());
    let options = AppIndex::from_apps(options);

    (
        Tile {
            query: String::new(),
            query_lc: String::new(),
            focus_id: 0,
            results: vec![],
            options,
            emoji_apps: AppIndex::from_apps(App::emoji_apps()),
            visible: true,
            focused: false,
            config: config.clone(),
            theme: config.theme.to_owned().into(),
            clipboard_content: vec![],
            tray_icon: None,
            sender: None,
            page: Page::Main,

            #[cfg(target_os = "macos")]
            frontmost: None,

            #[cfg(target_os = "windows")]
            frontmost: unsafe {
                use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

                Some(GetForegroundWindow())
            },

            #[cfg(not(target_os = "linux"))]
            hotkey,

            #[cfg(not(target_os = "linux"))]
            clipboard_hotkey: config
                .clipboard_hotkey
                .clone()
                .and_then(|x| x.parse::<HotKey>().ok()),
        },
        open,
    )
}

pub fn view(tile: &Tile, wid: window::Id) -> Element<'_, Message> {
    if tile.visible {
        let round_bottom_edges = match &tile.page {
            Page::Main | Page::EmojiSearch => tile.results.is_empty(),
            Page::ClipboardHistory => tile.clipboard_content.is_empty(),
        };
        let title_input = text_input(tile.config.placeholder.as_str(), &tile.query)
            .on_input(move |a| Message::SearchQueryChanged(a, wid))
            .on_paste(move |a| Message::SearchQueryChanged(a, wid))
            .font(tile.config.theme.font())
            .on_submit(Message::OpenFocused)
            .id("query")
            .width(Fill)
            .line_height(LineHeight::Relative(1.75))
            .style(move |_, _| rustcast_text_input_style(&tile.config.theme, round_bottom_edges))
            .padding(20);

        let scrollbar_direction = if tile.config.theme.show_scroll_bar {
            Direction::Vertical(
                Scrollbar::new()
                    .width(10)
                    .scroller_width(10)
                    .anchor(Anchor::Start),
            )
        } else {
            Direction::Vertical(Scrollbar::hidden())
        };

        let results = if tile.page == Page::ClipboardHistory {
            clipboard_view(
                tile.clipboard_content.clone(),
                tile.focus_id,
                tile.config.theme.clone(),
                tile.focus_id,
            )
        } else if tile.results.is_empty() {
            space().into()
        } else if tile.page == Page::EmojiSearch {
            emoji_page(
                tile.config.theme.clone(),
                tile.emoji_apps
                    .search_prefix(&tile.query_lc)
                    .map(|x| x.to_owned())
                    .collect(),
                tile.focus_id,
            )
        } else {
            container(Column::from_iter(tile.results.iter().enumerate().map(
                |(i, app)| {
                    app.clone()
                        .render(tile.config.theme.clone(), i as u32, tile.focus_id)
                },
            )))
            .into()
        };

        let results_count = match &tile.page {
            Page::Main => tile.results.len(),
            Page::ClipboardHistory => tile.clipboard_content.len(),
            Page::EmojiSearch => tile.results.len(),
        };

        let height = if tile.page == Page::ClipboardHistory {
            385
        } else {
            std::cmp::min(tile.results.len() * 60, 290)
        };

        let scrollable = Scrollable::with_direction(results, scrollbar_direction)
            .id("results")
            .height(height as u32);

        let contents = container(
            Column::new()
                .push(title_input)
                .push(scrollable)
                .push(footer(tile.config.theme.clone(), results_count))
                .spacing(0),
        )
        .width(Length::Fixed(WINDOW_WIDTH))
        .height(Length::Shrink)
        .align_x(Alignment::Center)
        .align_y(Alignment::Start)
        .style(|_| container::Style {
            text_color: None,
            background: None,
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.,
                radius: Radius::new(15),
            },
            ..Default::default()
        });

        container(contents.clip(false))
            .style(|_| contents_style(&tile.config.theme))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Start)
            .into()
    } else {
        space().into()
    }
}

fn footer(theme: Theme, results_count: usize) -> Element<'static, Message> {
    let text = if results_count == 0 {
        return space().into();
    } else if results_count == 1 {
        "1 result found"
    } else {
        &format!("{} results found", results_count)
    };

    container(
        Row::new()
            .push(
                Text::new(text.to_string())
                    .size(12)
                    .height(30)
                    .color(theme.text_color(0.7))
                    .font(theme.font())
                    .align_x(Alignment::Center),
            )
            .padding(4)
            .width(Fill)
            .height(30),
    )
    .center(Length::Fill)
    .width(WINDOW_WIDTH)
    .padding(5)
    .style(move |_| container::Style {
        text_color: None,
        background: Some(iced::Background::Color(with_alpha(
            tint(theme.bg_color(), 0.04),
            1.0,
        ))),
        border: iced::Border {
            color: Color::WHITE,
            width: 0.,
            radius: Radius::new(15).top(0),
        },
        shadow: iced::Shadow {
            color: Color::TRANSPARENT,
            offset: Vector::ZERO,
            blur_radius: 0.,
        },
        snap: false,
    })
    .into()
}
