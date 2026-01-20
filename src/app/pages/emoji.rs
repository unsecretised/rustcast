use iced::{
    Alignment, Background, Element, Length,
    widget::{Button, Column, Row, Text, container, text::Wrapping},
};

use crate::{
    app::{Message, WINDOW_WIDTH, apps::App},
    config::Theme,
    styles::{emoji_button_container_style, emoji_button_style, result_row_container_style, tint},
};

pub fn emoji_page(
    tile_theme: Theme,
    emojis: Vec<App>,
    focussed_id: u32,
) -> Element<'static, Message> {
    let emoji_vec = emojis
        .chunks(4)
        .map(|x| x.to_vec())
        .collect::<Vec<Vec<App>>>();

    let mut column = Vec::new();

    let mut id_num = 0;

    for emoji_row in emoji_vec {
        let mut emoji_row_element = Row::new().spacing(20);
        for emoji in emoji_row {
            let theme_clone = tile_theme.clone();
            let element_column = Column::new()
                .push(
                    Text::new(emoji.name)
                        .font(tile_theme.font())
                        .size(30)
                        .width(Length::Fill)
                        .align_y(Alignment::Center)
                        .align_x(Alignment::Center),
                )
                .push(
                    Text::new(emoji.desc)
                        .size(12)
                        .width(Length::Fill)
                        .align_y(Alignment::Center)
                        .align_x(Alignment::Center)
                        .wrapping(Wrapping::WordOrGlyph)
                        .font(tile_theme.font()),
                );
            let value = tile_theme.clone();
            emoji_row_element = emoji_row_element.push(
                container(
                    Button::new(element_column)
                        .width(100)
                        .height(100)
                        .style(move |_, _| emoji_button_style(&value)),
                )
                .width(100)
                .height(100)
                .id(format!("result-{}", id_num))
                .style(move |_| emoji_button_container_style(&theme_clone, focussed_id == id_num)),
            );

            id_num += 1;
        }

        column.push(container(emoji_row_element).center_y(90).into());
    }

    let tile_theme_clone = tile_theme.clone();

    container(Column::from_vec(column).spacing(20))
        .padding(10)
        .style(move |_| {
            result_row_container_style(&tile_theme_clone, false).background({
                let mut clr = tile_theme_clone.bg_color();
                clr.a = 1.;
                Background::Color(tint(clr, 0.02))
            })
        })
        .center_x(WINDOW_WIDTH)
        .into()
}
