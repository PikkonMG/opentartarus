pub mod app;
pub mod client;
pub mod keypad;
pub mod theme;
pub mod view;

use crate::app::App;
use crate::theme::{WINDOW_HEIGHT, WINDOW_WIDTH};
use iced::Size;

pub fn run() -> iced::Result {
    iced::application("OpenTartarus", App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .style(|_state, _theme| iced::application::Appearance {
            background_color: theme::COLOR_BACKGROUND,
            text_color: theme::COLOR_TEXT,
        })
        .window_size(Size::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .exit_on_close_request(false)
        .run()
}
