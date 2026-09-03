use crate::app::{App, Message};
use crate::theme;
use crate::view::header::session_is_live;
use crate::view::widgets::{dot, hairline_color, surface_container};
use iced::widget::{container, row, text};
use iced::{Alignment, Element, Length};

pub fn status_bar(app: &App) -> Element<'_, Message> {
    let bar = row![
        dot(hairline_color(session_is_live(app))),
        text(app.status_line())
            .size(theme::TEXT_SMALL)
            .color(theme::COLOR_TEXT_FAINT),
    ]
    .spacing(theme::SPACE_SM)
    .align_y(Alignment::Center);

    container(bar)
        .width(Length::Fill)
        .height(Length::Fixed(theme::STATUS_BAR_HEIGHT))
        .padding([0.0, theme::SPACE_LG])
        .style(surface_container)
        .into()
}
