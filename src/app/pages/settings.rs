//! The settings page UI

use iced::widget::Slider;
use iced::widget::checkbox;
use iced::widget::scrollable;
use iced::widget::text_input;

use crate::app::SetConfigBufferFields;
use crate::app::SetConfigThemeFields;
use crate::styles::settings_checkbox_style;
use crate::styles::settings_save_button_style;
use crate::styles::settings_slider_style;
use crate::styles::settings_text_input_item_style;
use crate::{
    app::{SetConfigFields, pages::prelude::*},
    config::Config,
};

const SETTINGS_ITEM_PADDING: u16 = 5;
const SETTINGS_ITEM_HEIGHT: u32 = 70;
const SETTINGS_ITEM_COL_SPACING: u32 = 5;

pub fn settings_page(config: Config) -> Element<'static, Message> {
    let config = Box::new(config.clone());
    let theme = config.theme.clone();

    let hotkey_theme = theme.clone();
    let hotkey = settings_item_column([
        settings_hint_text(theme.clone(), "Set your hotkey"),
        text_input("Toggle Hotkey", &config.toggle_hotkey)
            .on_input(|input| Message::SetConfig(SetConfigFields::ToggleHotkey(input.clone())))
            .on_submit(Message::WriteConfig)
            .width(Length::Fill)
            .style(move |_, _| settings_text_input_item_style(&hotkey_theme))
            .into(),
    ]);

    let cb_theme = theme.clone();
    let cb_hotkey = settings_item_column([
        settings_hint_text(theme.clone(), "Set your hotkey"),
        text_input("Clipboard Hotkey", &config.clipboard_hotkey)
            .on_input(|input| Message::SetConfig(SetConfigFields::ClipboardHotkey(input.clone())))
            .on_submit(Message::WriteConfig)
            .width(Length::Fill)
            .style(move |_, _| settings_text_input_item_style(&cb_theme))
            .into(),
    ]);

    let placeholder_theme = theme.clone();
    let placeholder_setting = settings_item_column([
        settings_hint_text(theme.clone(), "Set the rustcast placeholder"),
        text_input("Set Placeholder", &config.placeholder)
            .on_input(|input| Message::SetConfig(SetConfigFields::PlaceHolder(input.clone())))
            .on_submit(Message::WriteConfig)
            .width(Length::Fill)
            .style(move |_, _| settings_text_input_item_style(&placeholder_theme))
            .into(),
    ]);

    let theme_clone = theme.clone();
    let search = settings_item_column([
        settings_hint_text(theme.clone(), "Set the search URL"),
        text_input("Set Search URL", &config.search_url)
            .on_input(|input| Message::SetConfig(SetConfigFields::SearchUrl(input.clone())))
            .on_submit(Message::WriteConfig)
            .width(Length::Fill)
            .style(move |_, _| settings_text_input_item_style(&theme_clone))
            .into(),
    ]);

    let theme_clone = theme.clone();
    let current_delay = config.debounce_delay;
    let debounce = settings_item_column([
        settings_hint_text(theme.clone(), "Set the debounce time"),
        text_input("Set Debounce time (ms)", &config.debounce_delay.to_string())
            .on_input(move |input: String| {
                let delay = input.parse::<u64>().unwrap_or(current_delay);
                Message::SetConfig(SetConfigFields::DebounceDelay(delay))
            })
            .on_submit(Message::WriteConfig)
            .width(Length::Fill)
            .style(move |_, _| settings_text_input_item_style(&theme_clone))
            .into(),
    ]);

    let theme_clone = theme.clone();
    let haptic = Row::from_iter([
        settings_hint_text(theme.clone(), "Haptic feedback"),
        checkbox(config.clone().haptic_feedback)
            .style(move |_, _| settings_checkbox_style(&theme_clone))
            .on_toggle(|input| Message::SetConfig(SetConfigFields::HapticFeedback(input)))
            .into(),
    ])
    .align_y(Alignment::Center)
    .spacing(SETTINGS_ITEM_COL_SPACING * 2)
    .padding(SETTINGS_ITEM_PADDING)
    .height(SETTINGS_ITEM_HEIGHT);

    let theme_clone = theme.clone();
    let tray_icon = settings_item_row([
        settings_hint_text(theme.clone(), "Show menubar icon"),
        checkbox(config.clone().show_trayicon)
            .style(move |_, _| settings_checkbox_style(&theme_clone))
            .on_toggle(|input| Message::SetConfig(SetConfigFields::ShowMenubarIcon(input)))
            .into(),
    ]);

    let theme_clone = theme.clone();
    let clear_on_hide = settings_item_row([
        settings_hint_text(theme.clone(), "Clear on hide"),
        checkbox(config.clone().buffer_rules.clear_on_hide)
            .style(move |_, _| settings_checkbox_style(&theme_clone))
            .on_toggle(move |input| {
                Message::SetConfig(SetConfigFields::SetBufferFields(
                    SetConfigBufferFields::ClearOnHide(input),
                ))
            })
            .into(),
    ]);

    let theme_clone = theme.clone();
    let clear_on_enter = settings_item_row([
        settings_hint_text(theme.clone(), "Clear on enter"),
        checkbox(config.clone().buffer_rules.clear_on_enter)
            .style(move |_, _| settings_checkbox_style(&theme_clone))
            .on_toggle(move |input| {
                Message::SetConfig(SetConfigFields::SetBufferFields(
                    SetConfigBufferFields::ClearOnEnter(input),
                ))
            })
            .into(),
    ]);

    let theme_clone = theme.clone();
    let show_icons = settings_item_row([
        settings_hint_text(theme.clone(), "Show icons"),
        checkbox(config.clone().theme.show_icons)
            .style(move |_, _| settings_checkbox_style(&theme_clone))
            .on_toggle(move |input| {
                Message::SetConfig(SetConfigFields::SetThemeFields(
                    SetConfigThemeFields::ShowIcons(input),
                ))
            })
            .into(),
    ]);

    let theme_clone = theme.clone();
    let font_family = settings_item_column([
        settings_hint_text(theme.clone(), "Set Font family"),
        text_input("Font family", &config.theme.font.unwrap_or("".to_string()))
            .on_input(move |input: String| {
                Message::SetConfig(SetConfigFields::SetThemeFields(SetConfigThemeFields::Font(
                    input,
                )))
            })
            .on_submit(Message::WriteConfig)
            .width(Length::Fill)
            .style(move |_, _| settings_text_input_item_style(&theme_clone))
            .into(),
    ]);

    let theme_clone = theme.clone();
    let theme_clone_1 = theme.clone();
    let theme_clone_2 = theme.clone();
    let theme_clone_3 = theme.clone();
    let text_clr = Column::from_iter([
        settings_hint_text(theme.clone(), "Set text colour"),
        Column::from_iter([
            settings_hint_text(theme.clone(), "Set R value"),
            Slider::new(
                0..=100,
                (theme_clone.text_color.0 * 100.) as i32,
                move |change| {
                    let txt_clr = theme_clone.text_color;
                    let change = change as f32 / 100.;
                    Message::SetConfig(SetConfigFields::SetThemeFields(
                        SetConfigThemeFields::TextColor(change, txt_clr.1, txt_clr.2),
                    ))
                },
            )
            .style(move |_, _| settings_slider_style(&theme_clone_1))
            .width((WINDOW_WIDTH / 5.) * 4.)
            .into(),
            settings_hint_text(theme.clone(), "Set G value"),
            Slider::new(
                0..=100,
                (theme_clone.text_color.1 * 100.) as i32,
                move |change| {
                    let txt_clr = theme_clone.text_color;
                    let change = change as f32 / 100.;
                    Message::SetConfig(SetConfigFields::SetThemeFields(
                        SetConfigThemeFields::TextColor(txt_clr.0, change, txt_clr.2),
                    ))
                },
            )
            .style(move |_, _| settings_slider_style(&theme_clone_2))
            .width((WINDOW_WIDTH / 5.) * 4.)
            .into(),
            settings_hint_text(theme.clone(), "Set B value"),
            Slider::new(
                0..=100,
                (theme_clone.text_color.2 * 100.) as i32,
                move |change| {
                    let txt_clr = theme_clone.text_color;
                    let change = change as f32 / 100.;
                    Message::SetConfig(SetConfigFields::SetThemeFields(
                        SetConfigThemeFields::TextColor(txt_clr.0, txt_clr.1, change),
                    ))
                },
            )
            .style(move |_, _| settings_slider_style(&theme_clone_3))
            .width((WINDOW_WIDTH / 5.) * 4.)
            .into(),
        ])
        .spacing(7)
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .into(),
    ]);

    let theme_clone = theme.clone();
    let theme_clone_1 = theme.clone();
    let theme_clone_2 = theme.clone();
    let theme_clone_3 = theme.clone();
    let bg_clr = Column::from_iter([
        settings_hint_text(theme.clone(), "Set text colour"),
        Column::from_iter([
            settings_hint_text(theme.clone(), "Set R value"),
            Slider::new(
                0..=100,
                (theme_clone.background_color.0 * 100.) as i32,
                move |change| {
                    let txt_clr = theme_clone.background_color;
                    let change = change as f32 / 100.;
                    Message::SetConfig(SetConfigFields::SetThemeFields(
                        SetConfigThemeFields::BackgroundColor(change, txt_clr.1, txt_clr.2),
                    ))
                },
            )
            .style(move |_, _| settings_slider_style(&theme_clone_1))
            .width((WINDOW_WIDTH / 5.) * 4.)
            .into(),
            settings_hint_text(theme.clone(), "Set G value"),
            Slider::new(
                0..=100,
                (theme_clone.background_color.1 * 100.) as i32,
                move |change| {
                    let txt_clr = theme_clone.background_color;
                    let change = change as f32 / 100.;
                    Message::SetConfig(SetConfigFields::SetThemeFields(
                        SetConfigThemeFields::BackgroundColor(txt_clr.0, change, txt_clr.2),
                    ))
                },
            )
            .style(move |_, _| settings_slider_style(&theme_clone_2))
            .width((WINDOW_WIDTH / 5.) * 4.)
            .into(),
            settings_hint_text(theme.clone(), "Set B value"),
            Slider::new(
                0..=100,
                (theme_clone.background_color.2 * 100.) as i32,
                move |change| {
                    let txt_clr = theme_clone.background_color;
                    let change = change as f32 / 100.;
                    Message::SetConfig(SetConfigFields::SetThemeFields(
                        SetConfigThemeFields::BackgroundColor(txt_clr.0, txt_clr.1, change),
                    ))
                },
            )
            .style(move |_, _| settings_slider_style(&theme_clone_3))
            .width((WINDOW_WIDTH / 5.) * 4.)
            .into(),
        ])
        .spacing(7)
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .into(),
    ]);

    container(scrollable(
        Column::from_iter([
            hotkey.into(),
            cb_hotkey.into(),
            placeholder_setting.into(),
            search.into(),
            debounce.into(),
            haptic.into(),
            tray_icon.into(),
            clear_on_hide.into(),
            clear_on_enter.into(),
            show_icons.into(),
            font_family.into(),
            text_clr.into(),
            bg_clr.into(),
            Row::from_iter([savebutton(theme.clone()), default_button(theme.clone())])
                .spacing(5)
                .width(Length::Fill)
                .into(),
        ])
        .spacing(10),
    ))
    .style(move |_| result_row_container_style(&theme, false))
    .height(Length::Fill)
    .width(Length::Fill)
    .padding(10)
    .align_x(Alignment::Center)
    .into()
}

fn savebutton(theme: Theme) -> Element<'static, Message> {
    Button::new(
        Text::new("Save")
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .font(theme.font()),
    )
    .style(move |_, _| settings_save_button_style(&theme))
    .width(Length::Fill)
    .on_press(Message::WriteConfig)
    .into()
}

fn default_button(theme: Theme) -> Element<'static, Message> {
    Button::new(
        Text::new("To default")
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .font(theme.font()),
    )
    .style(move |_, _| settings_save_button_style(&theme))
    .width(Length::Fill)
    .on_press(Message::SetConfig(SetConfigFields::ToDefault))
    .into()
}

fn settings_hint_text(theme: Theme, text: impl ToString) -> Element<'static, Message> {
    let text = text.to_string();

    Text::new(text)
        .font(theme.font())
        .color(theme.text_color(0.7))
        .into()
}

fn settings_item_column(
    elems: impl IntoIterator<Item = Element<'static, Message>>,
) -> Column<'static, Message> {
    Column::from_iter(elems)
        .spacing(SETTINGS_ITEM_COL_SPACING)
        .padding(SETTINGS_ITEM_PADDING)
        .height(SETTINGS_ITEM_HEIGHT)
}

fn settings_item_row(
    elems: impl IntoIterator<Item = Element<'static, Message>>,
) -> Row<'static, Message> {
    Row::from_iter(elems)
        .align_y(Alignment::Center)
        .spacing(SETTINGS_ITEM_COL_SPACING)
        .padding(SETTINGS_ITEM_PADDING)
        .height(SETTINGS_ITEM_HEIGHT)
}
