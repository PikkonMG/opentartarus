use crate::app::{App, Message};
use crate::theme;
use crate::view::widgets::{accent_text_button, quiet_button};
use iced::widget::{button, container, row, text, Space};
use iced::{Background, Border, Element, Length, Theme};

/// The release notice sits under the banner bar and never replaces it: a
/// device problem is the more urgent news, and both can be true at once.
/// It carries the accent colour rather than the danger colour, because a
/// new release is not a fault.
pub fn update_bar(app: &App) -> Element<'_, Message> {
    let Some(tag) = &app.update_tag else {
        return Space::with_height(0).into();
    };
    let copy = format!(
        "{}{}{}",
        theme::UPDATE_NOTICE_PREFIX,
        tag,
        theme::UPDATE_NOTICE_SUFFIX
    );
    let line = row![
        text(copy).size(theme::TEXT_BODY),
        button(text(theme::BUTTON_OPEN_RELEASES))
            .on_press(Message::OpenReleases)
            .style(accent_text_button),
        button(text(theme::BUTTON_DISMISS_UPDATE))
            .on_press(Message::DismissUpdate)
            .style(quiet_button),
    ]
    .spacing(theme::SPACE_SM)
    .align_y(iced::Alignment::Center);

    container(line)
        .width(Length::Fill)
        .padding([theme::SPACE_SM, theme::SPACE_LG])
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(theme::COLOR_SURFACE)),
            text_color: Some(theme::COLOR_TEXT),
            border: Border {
                color: theme::COLOR_ACCENT,
                width: theme::BORDER_HAIRLINE,
                radius: theme::RADIUS_CONTROL.into(),
            },
            ..container::Style::default()
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_notice_is_empty_until_the_check_finds_a_release() {
        let app = App::default();
        assert!(app.update_tag.is_none());
        // Nothing to assert on the element itself without a renderer; the
        // state the view reads is what decides whether the bar is drawn.
        let _ = update_bar(&app);
    }

    #[test]
    fn dismissing_the_notice_clears_it_for_the_rest_of_the_session() {
        let mut app = App::default();
        let _ = app.update(Message::UpdateChecked(Some(String::from("v0.3.0"))));
        assert_eq!(app.update_tag.as_deref(), Some("v0.3.0"));
        let _ = app.update(Message::DismissUpdate);
        assert!(app.update_tag.is_none());
        let _ = app.update(Message::UpdateChecked(None));
        assert!(app.update_tag.is_none(), "no news must not revive it");
    }
}
