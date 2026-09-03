mod banner;
mod header;
mod inspector;
mod lighting_tab;
mod menu;
mod sidebar;
mod status;
mod widgets;

use crate::app::{App, Message};
use crate::keypad::{self, Keypad};
use crate::theme;
use iced::widget::{column, container, row, stack};
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
    .height(Length::Fill);

    // The lighting strip stays as a fourth band until Task 8 replaces it with
    // the Lighting tab. Removing it here would make lighting unreachable for
    // two tasks.
    let page = column![
        header::header_bar(app),
        banner::banner_bar(app),
        body,
        lighting_tab::lighting_tab(app),
        status::status_bar(app),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    let content: Element<'_, Message> = if app.menu_open {
        stack![page, menu::menu_popup(app)].into()
    } else {
        page.into()
    };

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(theme::COLOR_BACKGROUND)),
            text_color: Some(theme::COLOR_TEXT),
            ..container::Style::default()
        })
        .into()
}
