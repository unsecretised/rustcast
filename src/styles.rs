use iced::border::Radius;
use iced::widget::text_input::Status;
use iced::widget::{button, container};
use iced::{Background, Border, Color, widget::text_input};

use crate::config::Theme as ConfigTheme;

/// Helper: mix base color with white (simple “tint”)
pub fn tint(mut c: Color, amount: f32) -> Color {
    c.r = c.r + (1.0 - c.r) * amount;
    c.g = c.g + (1.0 - c.g) * amount;
    c.b = c.b + (1.0 - c.b) * amount;
    c
}

/// Helper: apply alpha
pub fn with_alpha(mut c: Color, a: f32) -> Color {
    c.a = a;
    c
}

pub fn rustcast_text_input_style(theme: &ConfigTheme, status: Status) -> text_input::Style {
    let base_bg = theme.bg_color();
    let surface = with_alpha(tint(base_bg, 0.06), 1.0);

    let (border_color, border_width) = match status {
        text_input::Status::Focused { .. } => (theme.text_color(0.20), 1.),
        text_input::Status::Hovered => (theme.text_color(0.20), 1.),
        text_input::Status::Active => (theme.text_color(0.20), 1.),
        text_input::Status::Disabled => (theme.text_color(0.20), 1.),
    };

    text_input::Style {
        background: Background::Color(surface),
        border: Border {
            color: border_color,
            width: border_width,
            radius: Radius::new(5.0).bottom(0.),
        },
        icon: theme.text_color(0.7),
        placeholder: theme.text_color(0.45),
        value: theme.text_color(1.0),
        selection: theme.text_color(0.2),
    }
}

pub fn contents_style(theme: &ConfigTheme) -> container::Style {
    container::Style {
        background: None,
        text_color: None,
        border: iced::Border {
            color: theme.text_color(0.7),
            width: 1.0,
            radius: Radius::new(14.0),
        },
        ..Default::default()
    }
}

pub fn result_button_style(theme: &ConfigTheme) -> button::Style {
    button::Style {
        text_color: theme.text_color(1.),
        background: Some(Background::Color(theme.bg_color())),
        ..Default::default()
    }
}

pub fn result_row_container_style(tile: &ConfigTheme, focused: bool) -> container::Style {
    let base = tile.bg_color();
    let row_bg = if focused {
        with_alpha(tint(base, 0.10), 1.0)
    } else {
        with_alpha(tint(base, 0.04), 1.0)
    };

    container::Style {
        background: Some(Background::Color(row_bg)),
        border: Border {
            color: if focused {
                tile.text_color(0.35)
            } else {
                tile.text_color(0.10)
            },
            width: 0.2,
            radius: Radius::new(0.),
        },
        ..Default::default()
    }
}

pub fn emoji_button_container_style(tile_theme: &ConfigTheme, focused: bool) -> container::Style {
    let base = tile_theme.bg_color();
    let row_bg = if focused {
        with_alpha(tint(base, 0.10), 1.0)
    } else {
        with_alpha(tint(base, 0.04), 1.0)
    };
    container::Style {
        background: Some(Background::Color(row_bg)),
        text_color: Some(tile_theme.text_color(1.)),
        border: Border {
            color: tile_theme.text_color(0.8),
            width: 0.,
            radius: Radius::new(10),
        },
        ..Default::default()
    }
}

pub fn emoji_button_style(tile_theme: &ConfigTheme) -> button::Style {
    button::Style {
        background: Some(Background::Color(tint(tile_theme.bg_color(), 0.02))),
        text_color: tile_theme.text_color(1.),
        border: Border {
            color: tile_theme.text_color(0.8),
            width: 0.1,
            radius: Radius::new(10),
        },
        ..Default::default()
    }
}
