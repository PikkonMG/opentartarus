use crate::app::{App, Message, Phase};
use crate::theme;
use crate::view::widgets::{
    close_control_button, dot, hairline_color, header_container, quiet_button,
    window_control_button,
};
use iced::widget::{button, container, mouse_area, row, text};
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
    // The window has no system title bar, so this row is the title bar: the
    // left half drags the window, the right half holds the window controls.
    // The grip must claim every pixel left over after the controls, or only
    // the logo and title drag and the empty middle of the bar does nothing.
    // A bare `Space::with_width(Fill)` inside the row is not enough: the
    // mouse_area has to be told to fill too.
    let grip = mouse_area(
        container(
            row![
                logo(),
                text(theme::APP_TITLE).size(theme::TEXT_HEADING),
                device_pill(app),
            ]
            .spacing(theme::SPACE_MD)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(Alignment::Center),
    )
    .on_press(Message::DragWindow);

    // All four controls are the same square, the same glyph size and the same
    // style, so they read as one row rather than a boxed button beside three
    // loose glyphs. The menu button only looks different while its menu is
    // open, which is the one time that difference means something.
    let controls = row![
        header_control(
            theme::MENU_GLYPH,
            Message::ToggleMenu,
            if app.menu_open {
                quiet_button
            } else {
                window_control_button
            },
        ),
        header_control(
            theme::MINIMIZE_GLYPH,
            Message::MinimizeWindow,
            window_control_button,
        ),
        header_control(
            theme::MAXIMIZE_GLYPH,
            Message::ToggleMaximize,
            window_control_button,
        ),
        header_control(
            theme::CLOSE_GLYPH,
            Message::CloseWindow,
            close_control_button,
        ),
    ]
    .spacing(theme::SPACE_XXS)
    .align_y(Alignment::Center);

    container(
        row![grip, controls]
            .spacing(theme::SPACE_SM)
            .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(theme::HEADER_HEIGHT))
    .padding([0.0, theme::SPACE_SM])
    .style(header_container)
    .into()
}

/// One square title-bar control. Every caller passes the same size, so the
/// glyphs sit on one optical line however wide or tall they draw.
fn header_control<'a>(
    glyph: &'a str,
    message: Message,
    style: fn(&Theme, button::Status) -> button::Style,
) -> Element<'a, Message> {
    button(
        container(text(glyph).size(theme::TEXT_HEADING))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .width(Length::Fixed(theme::ICON_BUTTON_SIZE))
    .height(Length::Fixed(theme::ICON_BUTTON_SIZE))
    .padding(0)
    .on_press(message)
    .style(style)
    .into()
}

/// True while the daemon is up. The header dot and the status dot share it.
pub fn session_is_live(app: &App) -> bool {
    app.phase == Phase::Running
}
