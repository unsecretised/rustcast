//! This has all the logic regarding the cliboard history
use arboard::ImageData;
use iced::{
    Length::Fill,
    alignment::Vertical,
    widget::{Button, Row, Text, container},
};

use crate::{app::Message, commands::Function, config::Theme as ConfigTheme};

/// The kinds of clipboard content that rustcast can handle and their contents
#[derive(Debug, Clone)]
pub enum ClipBoardContentType {
    Text(String),
    Image(ImageData<'static>),
}

impl ClipBoardContentType {
    /// Returns the iced element for rendering the clipboard item
    pub fn render_clipboard_item(
        &self,
        theme: ConfigTheme,
    ) -> impl Into<iced::Element<'_, Message>> {
        let mut tile = Row::new().width(Fill).height(55);

        let text = match self {
            ClipBoardContentType::Text(text) => text,
            ClipBoardContentType::Image(_) => "<img>",
        };

        let bg_color = theme.bg_color();
        let bg_color_clone = bg_color;

        let text_color = theme.text_color(1.);
        let text_color_clone = text_color;

        tile = tile.push(
            Button::new(
                Text::new(text.to_owned())
                    .font(theme.font())
                    .height(Fill)
                    .width(Fill)
                    .align_y(Vertical::Center),
            )
            .on_press(Message::RunFunction(Function::CopyToClipboard(
                self.to_owned(),
            )))
            .style(move |_, _| iced::widget::button::Style {
                background: Some(iced::Background::Color(bg_color_clone)),
                text_color: text_color_clone,
                ..Default::default()
            })
            .width(Fill)
            .height(55),
        );

        container(tile)
            .style(move |_| iced::widget::container::Style {
                text_color: Some(text_color),
                background: Some(iced::Background::Color(bg_color)),
                ..Default::default()
            })
            .width(Fill)
            .height(Fill)
    }
}

impl PartialEq for ClipBoardContentType {
    /// Let cliboard items be comparable
    fn eq(&self, other: &Self) -> bool {
        if let Self::Text(a) = self
            && let Self::Text(b) = other
        {
            return a == b;
        } else if let Self::Image(image_data) = self
            && let Self::Image(other_image_data) = other
        {
            return image_data.bytes == other_image_data.bytes;
        }
        false
    }
}
