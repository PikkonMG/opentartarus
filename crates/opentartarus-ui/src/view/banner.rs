use crate::app::{App, Banner, Message};
use crate::theme;
use crate::view::widgets::primary_button;
use iced::widget::{button, container, row, text, Space};
use iced::{Background, Border, Element, Length, Theme};
use opentartarus_core::error::ErrorCode;

pub fn banner_bar(app: &App) -> Element<'_, Message> {
    let Some(banner) = app.banner() else {
        return Space::with_height(0).into();
    };
    let (copy, show_fix) = match banner {
        Banner::Starting => (theme::BANNER_STARTING.to_string(), false),
        Banner::CouldNotStart(copy) => (copy, false),
        Banner::NoDevice => (theme::BANNER_NO_DEVICE.to_string(), false),
        Banner::Permission => (ErrorCode::Permission.user_message().to_string(), true),
        Banner::Disconnect => (ErrorCode::Disconnect.user_message().to_string(), false),
        Banner::GrabConflict { name } => {
            let base = ErrorCode::GrabConflict.user_message().to_string();
            match name {
                Some(n) => (format!("{base} — {n}"), false),
                None => (base, false),
            }
        }
        Banner::UnplugAfterFix => (theme::BANNER_UNPLUG_AFTER_FIX.to_string(), false),
        Banner::SignOut => (theme::BANNER_SIGN_OUT.to_string(), false),
        Banner::Other(msg) => (msg, false),
    };
    let mut row = row![text(copy).size(theme::TEXT_BODY)]
        .spacing(theme::SPACE_SM)
        .align_y(iced::Alignment::Center);
    if show_fix {
        row = row.push(
            button(text(theme::BUTTON_FIX_PERMISSIONS))
                .on_press(Message::FixPermissions)
                .style(primary_button),
        );
    }
    container(row)
        .width(Length::Fill)
        .padding([theme::SPACE_SM, theme::SPACE_LG])
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(theme::COLOR_SURFACE)),
            text_color: Some(theme::COLOR_TEXT),
            border: Border {
                color: theme::COLOR_DANGER,
                width: theme::BORDER_HAIRLINE,
                radius: theme::RADIUS_CONTROL.into(),
            },
            ..container::Style::default()
        })
        .into()
}
