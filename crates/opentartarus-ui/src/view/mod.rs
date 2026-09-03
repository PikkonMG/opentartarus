mod banner;
mod header;
mod inspector;
mod keys_tab;
pub(crate) mod lighting_tab;
mod menu;
mod sidebar;
mod status;
mod widgets;

use crate::app::{App, Message};
use crate::theme;
use iced::widget::{column, container, row, stack};
use iced::{Background, Element, Length, Theme};

pub fn view(app: &App) -> Element<'_, Message> {
    let body = row![
        sidebar::profile_list(app),
        keys_tab::center_panel(app),
        inspector::inspector(app),
    ]
    .height(Length::Fill);

    let page = column![
        header::header_bar(app),
        banner::banner_bar(app),
        body,
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
