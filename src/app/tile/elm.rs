//! This module handles the logic for the new and view functions according to the elm
//! architecture. If the subscription function becomes too large, it should be moved to this file

use global_hotkey::hotkey::HotKey;
use iced::border::Radius;
use iced::widget::scrollable::{Anchor, Direction, Scrollbar};
use iced::widget::text::LineHeight;
use iced::widget::{Column, Scrollable, container, space};
use iced::{Color, window};
use iced::{Element, Task};
use iced::{Length::Fill, widget::text_input};

use rayon::{
    iter::{IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

use crate::app::pages::clipboard::clipboard_view;
use crate::app::pages::emoji::emoji_page;
use crate::app::tile::AppIndex;
use crate::styles::{contents_style, rustcast_text_input_style};
use crate::{
    app::{Message, Page, apps::App, default_settings, tile::Tile},
    config::Config,
    macos::{self, transform_process_to_ui_element},
    utils::get_installed_apps,
};

pub fn default_app_paths() -> Vec<String> {
    let user_local_path = std::env::var("HOME").unwrap() + "/Applications/";

    let paths = vec![
        "/Applications/".to_string(),
        user_local_path,
        "/System/Applications/".to_string(),
        "/System/Applications/Utilities/".to_string(),
    ];

    paths
}

/// Initialise the base window
pub fn new(hotkey: HotKey, config: &Config) -> (Tile, Task<Message>) {
    let (id, open) = window::open(default_settings());

    let open = open.discard().chain(window::run(id, |handle| {
        macos::macos_window_config(&handle.window_handle().expect("Unable to get window handle"));
        transform_process_to_ui_element();
    }));

    let store_icons = config.theme.show_icons;

    let paths = default_app_paths();

    let mut options: Vec<App> = paths
        .par_iter()
        .map(|path| get_installed_apps(path, store_icons))
        .flatten()
        .collect();

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
            hotkey,
            visible: true,
            frontmost: None,
            focused: false,
            config: config.clone(),
            theme: config.theme.to_owned().into(),
            clipboard_content: vec![],
            tray_icon: None,
            sender: None,
            page: Page::Main,
        },
        Task::batch([open.map(|_| Message::OpenWindow)]),
    )
}

pub fn view(tile: &Tile, wid: window::Id) -> Element<'_, Message> {
    if tile.visible {
        let title_input = text_input(tile.config.placeholder.as_str(), &tile.query)
            .on_input(move |a| Message::SearchQueryChanged(a, wid))
            .on_paste(move |a| Message::SearchQueryChanged(a, wid))
            .font(tile.config.theme.font())
            .on_submit(Message::OpenFocused)
            .id("query")
            .width(Fill)
            .line_height(LineHeight::Relative(1.75))
            .style(|_, status| rustcast_text_input_style(&tile.config.theme, status))
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
            Column::from_iter(tile.results.iter().enumerate().map(|(i, app)| {
                app.clone()
                    .render(tile.config.theme.clone(), i as u32, tile.focus_id)
            }))
            .into()
        };

        let scrollable = Scrollable::with_direction(results, scrollbar_direction).id("results");
        let contents = container(Column::new().push(title_input).push(scrollable).spacing(0))
            .style(|_| container::Style {
                text_color: None,
                background: None,
                border: iced::Border {
                    color: Color::WHITE,
                    width: 1.,
                    radius: Radius::new(5),
                },
                ..Default::default()
            });

        container(contents.clip(false))
            .style(|_| contents_style(&tile.config.theme))
            .into()
    } else {
        space().into()
    }
}
