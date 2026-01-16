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

use crate::app::apps::AppCommand;
use crate::app::tile::AppIndex;
use crate::config::Theme;
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
            .line_height(LineHeight::Relative(1.5))
            .style(|_, _| text_input_style(&tile.config.theme))
            .padding(20);

        let scrollbar_direction = if tile.config.theme.show_scroll_bar {
            Direction::Vertical(
                Scrollbar::new()
                    .width(2)
                    .scroller_width(2)
                    .anchor(Anchor::Start),
            )
        } else {
            Direction::Vertical(Scrollbar::hidden())
        };
        let results = match tile.page {
            Page::Main => {
                let mut search_results = Column::new();
                for (i, result) in tile.results.iter().enumerate() {
                    search_results = search_results.push(result.render(
                        &tile.config.theme,
                        i as u32,
                        tile.focus_id,
                    ));
                }
                search_results
            }
            Page::ClipboardHistory => {
                let mut clipboard_history = Column::new();
                for result in &tile.clipboard_content {
                    clipboard_history = clipboard_history
                        .push(result.render_clipboard_item(tile.config.theme.clone()));
                }
                clipboard_history
            }
        };
        let scrollable = Scrollable::with_direction(results, scrollbar_direction).id("results");
        let contents = Column::new().push(title_input).push(scrollable);

        container(contents)
            .style(|_| iced::widget::container::Style {
                background: None,
                text_color: None,
                border: iced::Border {
                    color: tile.config.theme.text_color(1.),
                    width: 1.,
                    radius: Radius::new(0),
                },
                ..Default::default()
            })
            .padding(0)
            .clip(true)
            .into()
    } else {
        space().into()
    }
}

fn text_input_style(theme: &Theme) -> iced::widget::text_input::Style {
    text_input::Style {
        background: iced::Background::Color(Color::TRANSPARENT),
        border: iced::Border {
            color: iced::Color {
                r: 0.95,
                g: 0.95,
                b: 0.95,
                a: 0.7,
            },
            width: 0.5,
            radius: iced::border::Radius {
                top_left: 0.,
                top_right: 0.,
                bottom_right: 0.,
                bottom_left: 0.,
            },
        },
        icon: theme.text_color(0.),
        placeholder: theme.text_color(0.7),
        value: theme.text_color(1.),
        selection: theme.text_color(0.2),
    }
}
