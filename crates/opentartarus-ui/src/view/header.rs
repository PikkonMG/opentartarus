use crate::app::{App, Message, Phase};
use crate::theme;
use crate::view::widgets::{
    app_menu_button, close_control_button, dot, hairline_color, header_container,
    quiet_button, window_control_button, window_glyph_color,
};
use crate::view::window_icon::{self, WindowGlyph};
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

    // Three circles, matching how this desktop draws its own window buttons.
    // The app menu sits apart from them, with a gap, because it is not a
    // window control and must not read as a fourth one.
    let controls = row![
        window_control(WindowGlyph::ChevronDown, Message::MinimizeWindow, false),
        window_control(WindowGlyph::ChevronUp, Message::ToggleMaximize, false),
        window_control(WindowGlyph::Cross, Message::CloseWindow, true),
    ]
    .spacing(theme::SPACE_SM)
    .align_y(Alignment::Center);

    // The app menu is deliberately NOT a circle. Circles are this desktop's
    // vocabulary for window controls; making the menu one more circle is what
    // made it read as a fourth window button. A rounded square, quiet until
    // hovered, says "different kind of thing" before the glyph is even read.
    let menu_button = button(
        container(text(theme::MENU_GLYPH).size(theme::TEXT_HEADING))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .width(Length::Fixed(theme::ICON_BUTTON_SIZE))
    .height(Length::Fixed(theme::ICON_BUTTON_SIZE))
    .padding(0)
    .on_press(Message::ToggleMenu)
    .style(if app.menu_open {
        quiet_button
    } else {
        app_menu_button
    });

    container(
        row![grip, menu_button, controls]
            .spacing(theme::SPACE_XL)
            .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(theme::HEADER_HEIGHT))
    .padding([0.0, theme::SPACE_MD])
    .style(header_container)
    .into()
}

/// One circular window control. The glyph is stroked on a canvas rather than
/// typed, so all three share a width, a box and a cap.
fn window_control<'a>(
    glyph: WindowGlyph,
    message: Message,
    is_close: bool,
) -> Element<'a, Message> {
    let style = if is_close {
        close_control_button
    } else {
        window_control_button
    };
    button(
        container(window_icon::glyph(
            glyph,
            window_glyph_color(false, is_close),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center),
    )
    .width(Length::Fixed(window_icon::CONTROL_SIZE))
    .height(Length::Fixed(window_icon::CONTROL_SIZE))
    .padding(0)
    .on_press(message)
    .style(style)
    .into()
}

/// True while the daemon is up. The header dot and the status dot share it.
pub fn session_is_live(app: &App) -> bool {
    app.phase == Phase::Running
}
