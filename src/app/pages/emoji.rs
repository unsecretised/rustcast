use iced::{Border, Length::Fill, border::Radius, widget::tooltip};

use crate::{app::pages::prelude::*, clipboard::ClipBoardContentType, commands::Function};

pub fn emoji_page(
    tile_theme: Theme,
    emojis: &[App],
    focussed_id: u32,
) -> Element<'static, Message> {
    let emoji_vec = emojis
        .chunks(6)
        .map(<[App]>::to_vec)
        .collect::<Vec<Vec<App>>>();

    let mut column = Vec::new();

    let mut id_num = 0;

    for emoji_row in emoji_vec {
        let mut emoji_row_element = Row::new().spacing(10);
        for emoji in emoji_row {
            let theme_clone = tile_theme.clone();
            let element_column = Column::new().push(
                Text::new(emoji.name.clone())
                    .font(tile_theme.font())
                    .size(30)
                    .width(Length::Fill)
                    .height(Fill)
                    .align_y(Alignment::Center)
                    .align_x(Alignment::Center),
            );
            let value = tile_theme.clone();
            let value_two = tile_theme.clone();
            emoji_row_element = emoji_row_element.push(tooltip(
                container(
                    Button::new(element_column)
                        .width(70)
                        .height(70)
                        .on_press(Message::RunFunction(Function::CopyToClipboard(
                            ClipBoardContentType::Text(emoji.name),
                        )))
                        .style(move |_, _| emoji_button_style(&value)),
                )
                .width(70)
                .height(70)
                .id(format!("result-{id_num}"))
                .style(move |_| emoji_button_container_style(&theme_clone, focussed_id == id_num)),
                container(
                    Text::new(emoji.desc)
                        .font(tile_theme.font())
                        .size(20)
                        .color(tile_theme.text_color(0.7)),
                )
                .style(move |_| container::Style {
                    background: Some(Background::Color(value_two.bg_color())),
                    ..Default::default()
                }),
                tooltip::Position::Top,
            ));

            id_num += 1;
        }

        column.push(container(emoji_row_element).center_y(70).into());
    }

    let tile_theme_clone = tile_theme.clone();

    container(Column::from_vec(column).spacing(10))
        .padding(5)
        .style(move |_| {
            result_row_container_style(&tile_theme_clone, false)
                .background({
                    let mut clr = tile_theme_clone.bg_color();
                    clr.a = 1.;
                    Background::Color(tint(clr, 0.02))
                })
                .border(Border {
                    color: tile_theme.bg_color(),
                    width: 1.,
                    radius: Radius::new(0),
                })
        })
        .center_x(WINDOW_WIDTH)
        .into()
}
