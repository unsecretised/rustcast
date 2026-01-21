pub use iced::{
    Alignment, Background, Element, Length,
    widget::{Button, Column, Row, Text, container},
};

pub use crate::{
    app::{Message, WINDOW_WIDTH, apps::App},
    config::Theme,
    styles::{emoji_button_container_style, emoji_button_style, result_row_container_style, tint},
};
