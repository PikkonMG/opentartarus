mod strings;
mod tokens;

pub use strings::*;
pub use tokens::*;

use iced::theme::Palette;
use iced::{Color, Theme};

pub fn theme() -> Theme {
    Theme::custom(
        String::from(APP_TITLE),
        Palette {
            background: COLOR_BACKGROUND,
            text: COLOR_TEXT,
            primary: COLOR_ACCENT,
            success: COLOR_OK,
            danger: COLOR_DANGER,
        },
    )
}

pub fn with_opacity(color: Color, opacity: f32) -> Color {
    Color {
        a: color.a * opacity,
        ..color
    }
}

pub fn could_not_start_message(reason: &str) -> String {
    format!("{BANNER_COULD_NOT_START_PREFIX} {reason}")
}
