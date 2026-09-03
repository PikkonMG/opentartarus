use crate::app::{App, Message, Phase};
use crate::theme;
use crate::view::widgets::{
    close_control_button, dot, hairline_color, quiet_button, surface_container,
    window_control_button,
};
use iced::widget::{button, container, mouse_area, row, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

fn logo<'a>() -> Element<'a, Message> {
    container(
        text(theme::LOGO_GLYPH)
            .size(theme::TEXT_BODY)
            .color(Color::WHITE),
    )
    .width(Length::Fixed(theme::LOGO_SIZE))
    .height(Length::Fixed(theme::LOGO_SIZE))
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .style(|_theme: &Theme| container::Style {
        background: Some(Background::Color(theme::COLOR_ACCENT)),
        border: Border {
            color: Color::TRANSPARENT,
            width: theme::BORDER_NONE,
            radius: theme::RADIUS_CONTROL.into(),
        },
        ..container::Style::default()
    })
    .into()
}

fn device_pill(app: &App) -> Element<'_, Message> {
    let live = app.device_present && app.model.is_some();
    container(
        row![
            dot(hairline_color(live)),
            text(app.device_pill_text())
                .size(theme::TEXT_SMALL)
                .color(theme::COLOR_TEXT_DIM),
        ]
        .spacing(theme::SPACE_SM)
        .align_y(Alignment::Center),
    )
    .padding([theme::SPACE_XS, theme::SPACE_MD])
    .style(|_theme: &Theme| container::Style {
        background: Some(Background::Color(theme::COLOR_RAISED)),
        border: Border {
            color: theme::COLOR_LINE,
            width: theme::BORDER_HAIRLINE,
            radius: theme::RADIUS_PILL.into(),
        },
        ..container::Style::default()
    })
    .into()
}

pub fn header_bar(app: &App) -> Element<'_, Message> {
    let menu_button = button(text(theme::MENU_GLYPH).size(theme::TEXT_TITLE))
        .width(Length::Fixed(theme::ICON_BUTTON_SIZE))
        .height(Length::Fixed(theme::ICON_BUTTON_SIZE))
        .on_press(Message::ToggleMenu)
        .style(quiet_button);

    // The window has no system title bar, so this row is the title bar: the
    // left half drags the window, the right half holds the window controls.
    let grip = mouse_area(
        row![
            logo(),
            text(theme::APP_TITLE).size(theme::TEXT_HEADING),
            device_pill(app),
            Space::with_width(Length::Fill),
        ]
        .spacing(theme::SPACE_MD)
        .align_y(Alignment::Center),
    )
    .on_press(Message::DragWindow);

    let bar = row![
        grip,
        menu_button,
        window_button(theme::MINIMIZE_GLYPH, Message::MinimizeWindow),
        window_button(theme::MAXIMIZE_GLYPH, Message::ToggleMaximize),
        close_button(),
    ]
    .spacing(theme::SPACE_XS)
    .align_y(Alignment::Center);

    container(bar)
        .width(Length::Fill)
        .height(Length::Fixed(theme::HEADER_HEIGHT))
        .padding([0.0, theme::SPACE_MD])
        .style(surface_container)
        .into()
}

fn window_button<'a>(glyph: &'a str, message: Message) -> Element<'a, Message> {
    button(text(glyph).size(theme::TEXT_BODY))
        .width(Length::Fixed(theme::ICON_BUTTON_SIZE))
        .height(Length::Fixed(theme::ICON_BUTTON_SIZE))
        .on_press(message)
        .style(window_control_button)
        .into()
}

fn close_button<'a>() -> Element<'a, Message> {
    button(text(theme::CLOSE_GLYPH).size(theme::TEXT_TITLE))
        .width(Length::Fixed(theme::ICON_BUTTON_SIZE))
        .height(Length::Fixed(theme::ICON_BUTTON_SIZE))
        .on_press(Message::CloseWindow)
        .style(close_control_button)
        .into()
}

/// True while the daemon is up. The header dot and the status dot share it.
pub fn session_is_live(app: &App) -> bool {
    app.phase == Phase::Running
}
