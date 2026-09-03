use crate::app::{App, Message};
use crate::keypad::{self, Keypad};
use crate::theme;
use crate::view::widgets::{card_container, tab_strip};
use iced::widget::{column, container, text};
use iced::{Element, Length};

pub fn keys_tab(app: &App) -> Element<'_, Message> {
    let pad = keypad::widget(Keypad {
        model: app.model,
        bindings: app.bindings.clone(),
        selected: app.selected_key,
        faded: app.keypad_faded(),
        hovered: app.hovered_key,
    });

    container(
        column![
            pad,
            text(theme::KEYPAD_HINT)
                .size(theme::TEXT_SMALL)
                .color(theme::COLOR_TEXT_FAINT),
        ]
        .spacing(theme::SPACE_MD)
        .padding(theme::SPACE_XL)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(card_container)
    .into()
}

/// The centre column: the tab strip above whichever tab is showing.
pub fn center_panel(app: &App) -> Element<'_, Message> {
    let body: Element<'_, Message> = match app.tab {
        crate::app::Tab::Keys => keys_tab(app),
        crate::app::Tab::Lighting => crate::view::lighting_tab::lighting_tab(app),
    };
    column![tab_strip(app.tab), body]
        .spacing(theme::SPACE_MD)
        .padding(theme::SPACE_LG)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
