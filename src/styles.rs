use crate::config::Theme as ConfigTheme;
use iced::border::Radius;
use iced::widget::{button, container};
use iced::{Background, Border, Color, widget::text_input};
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
pub fn rustcast_text_input_style(
    theme: &ConfigTheme,
    round_bottom_edges: bool,
) -> text_input::Style {
    let base = theme.bg_color();
    let focused = false; // if you have state, pass it in and use it
    let surface = glass_surface(base, focused);
    text_input::Style {
        background: Background::Color(surface),
        border: Border {
            color: glass_border(theme.text_color(1.0), focused),
            width: 1.0,
            radius: Radius::new(15.).bottom(if round_bottom_edges { 15. } else { 0. }),
        },
        icon: theme.text_color(0.75),
        placeholder: theme.text_color(0.50),
        value: theme.text_color(1.0),
        selection: with_alpha(theme.text_color(1.0), 0.20),
    }
}
pub fn contents_style(theme: &ConfigTheme) -> container::Style {
    container::Style {
        background: None,
        text_color: None,
        border: iced::Border {
            color: theme.text_color(0.7),
            width: 0.,
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
    container::Style {
        background: Some(Background::Color(glass_surface(tile.bg_color(), focused))),
        border: Border {
            color: glass_border(tile.text_color(1.), focused),
            width: 1.,
            radius: Radius::new(0.0),
        },
        text_color: Some(tile.text_color(1.0)),
        ..Default::default()
    }
}
pub fn emoji_button_container_style(tile_theme: &ConfigTheme, focused: bool) -> container::Style {
    container::Style {
        background: Some(Background::Color(glass_surface(
            tile_theme.bg_color(),
            focused,
        ))),
        text_color: Some(tile_theme.text_color(1.0)),
        border: Border {
            color: glass_border(tile_theme.text_color(1.0), focused),
            width: 1.0,
            radius: Radius::new(10.0),
        },
        ..Default::default()
    }
}
pub fn emoji_button_style(tile_theme: &ConfigTheme) -> button::Style {
    let base = tile_theme.bg_color();
    let bg = with_alpha(tint(base, 0.10), 0.28);
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: tile_theme.text_color(1.0),
        border: Border {
            color: glass_border(tile_theme.text_color(1.0), false),
            width: 1.0,
            radius: Radius::new(10.0),
        },
        ..Default::default()
    }
}

pub fn glass_surface(base: Color, focused: bool) -> Color {
    let t = if focused { 0.5 } else { 0.06 };
    let a = if focused { 0.5 } else { 0.22 };
    with_alpha(tint(base, t), a)
}

pub fn glass_border(base_text: Color, focused: bool) -> Color {
    let a = if focused { 0.35 } else { 0.22 };
    with_alpha(base_text, a)
}
