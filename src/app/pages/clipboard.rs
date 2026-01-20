use crate::{app::pages::prelude::*, clipboard::ClipBoardContentType};

pub fn clipboard_view(
    clipboard_content: Vec<ClipBoardContentType>,
    focussed_id: u32,
    theme: Theme,
    focus_id: u32,
) -> Element<'static, Message> {
    let theme_clone = theme.clone();
    Row::from_vec(vec![
        Column::from_iter(
            clipboard_content
                .iter()
                .enumerate()
                .map(|(i, content)| content.to_app().render(theme.clone(), i as u32, focus_id)),
        )
        .width(WINDOW_WIDTH as u32 / 2)
        .into(),
        container(
            Text::new(
                clipboard_content
                    .get(focussed_id as usize)
                    .map(|x| x.to_app().name)
                    .unwrap_or("".to_string()),
            )
            .wrapping(Wrapping::WordOrGlyph)
            .font(theme.font())
            .size(22),
        )
        .style(move |_| clipboard_side_view_style(&theme_clone))
        .width(WINDOW_WIDTH as u32 / 2)
        .into(),
    ])
    .into()
}

fn clipboard_side_view_style(theme: &Theme) -> container::Style {
    container::Style {
        text_color: None,
        background: Some(Background::Color(theme.bg_color())),
        ..Default::default()
    }
}
