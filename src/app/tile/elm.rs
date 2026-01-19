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

use rayon::slice::ParallelSliceMut;

use crate::app::tile::AppIndex;
use crate::utils::get_installed_apps;
use crate::styles::{contents_style, rustcast_text_input_style};
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
}

/// Initialise the base window
pub fn new(hotkey: HotKey, config: &Config) -> (Tile, Task<Message>) {
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

    let (id, open) = window::open(settings);

    #[cfg(target_os = "macos")]
    let open = open.discard().chain(window::run(id, |handle| {
        macos::macos_window_config(&handle.window_handle().expect("Unable to get window handle"));
        transform_process_to_ui_element();
    }));

    let mut options: Vec<App> = get_installed_apps(&config);

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
                    .width(2)
                    .scroller_width(2)
                    .anchor(Anchor::Start),
            )
        } else {
            Direction::Vertical(Scrollbar::hidden())
        };

        let results = if tile.page == Page::ClipboardHistory {
            Column::from_iter(
                tile.clipboard_content
                    .iter()
                    .enumerate()
                    .map(|(i, content)| {
                        content
                            .to_app()
                            .render(tile.config.theme.clone(), i as u32, tile.focus_id)
                    }),
            )
        } else {
            Column::from_iter(tile.results.iter().enumerate().map(|(i, app)| {
                app.clone()
                    .render(tile.config.theme.clone(), i as u32, tile.focus_id)
            }))
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

        container(contents.clip(true))
            .style(|_| contents_style(&tile.config.theme))
            .into()
    } else {
        space().into()
    }
}
