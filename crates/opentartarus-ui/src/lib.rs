pub mod app;
pub mod client;
pub mod keypad;
pub mod keys;
pub mod theme;
pub mod view;

use crate::app::App;
use crate::theme::{WINDOW_HEIGHT, WINDOW_WIDTH};
use iced::Size;

const WINDOW_ICON_PNG: &[u8] = include_bytes!("../../../packaging/icons/opentartarus-64.png");

/// The window icon. `None` when the PNG cannot be decoded; the window then
/// opens without one, which is what happens today.
fn window_icon() -> Option<iced::window::Icon> {
    let decoder = png::Decoder::new(WINDOW_ICON_PNG);
    let mut reader = decoder.read_info().ok()?;
    let mut buffer = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).ok()?;
    buffer.truncate(info.buffer_size());
    iced::window::icon::from_rgba(buffer, info.width, info.height).ok()
}

pub fn run() -> iced::Result {
    iced::application(theme::APP_TITLE, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .style(|_state, _theme| iced::application::Appearance {
            background_color: theme::COLOR_BACKGROUND,
            text_color: theme::COLOR_TEXT,
        })
        .window(iced::window::Settings {
            size: Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            icon: window_icon(),
            ..iced::window::Settings::default()
        })
        .exit_on_close_request(false)
        .run()
}
