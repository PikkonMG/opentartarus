use crate::app::{App, Banner, Message};
use crate::theme::{self, COLOR_DANGER, COLOR_SURFACE, COLOR_TEXT};
use crate::view::widgets::danger_text_button;
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
    let mut row = row![text(copy).size(14)]
        .spacing(10)
        .align_y(iced::Alignment::Center);
    if show_fix {
        row = row.push(
            button(text(theme::BUTTON_FIX_PERMISSIONS))
                .on_press(Message::FixPermissions)
                .style(danger_text_button),
        );
    }
    container(row)
        .width(Length::Fill)
        .padding(8)
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(COLOR_SURFACE)),
            text_color: Some(COLOR_TEXT),
            border: Border {
                color: COLOR_DANGER,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}
