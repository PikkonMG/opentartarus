mod banner;
mod inspector;
mod lighting_tab;
mod sidebar;
mod widgets;

use crate::app::{App, Message};
use crate::keypad::{self, Keypad};
use crate::theme;
use iced::widget::{column, container, row};
use iced::{Background, Element, Length, Theme};

pub fn view(app: &App) -> Element<'_, Message> {
    let body = row![
        sidebar::profile_list(app),
        keypad::widget(Keypad {
            model: app.model,
            bindings: app.bindings.clone(),
            selected: app.selected_key,
            faded: app.keypad_faded(),
            hovered: app.hovered_key,
        }),
        inspector::inspector(app),
    ]
    .spacing(theme::SPACE_SM)
    .height(Length::Fill);

    container(
        column![
            banner::banner_bar(app),
            body,
            lighting_tab::lighting_tab(app)
        ]
        .spacing(theme::SPACE_SM)
        .padding(theme::SPACE_MD)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(Background::Color(theme::COLOR_BACKGROUND)),
        text_color: Some(theme::COLOR_TEXT),
        ..container::Style::default()
    })
    .into()
}
