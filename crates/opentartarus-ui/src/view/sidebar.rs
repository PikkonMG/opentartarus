use crate::app::{App, Message, ProfileRow};
use crate::theme;
use crate::view::widgets::{
    accent_text_button, row_button, section_label, selected_row_button, surface_container, swatch,
};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

/// Shown when a profile declares no lighting colour, such as `default`.
/// Reuses `theme::SWATCH_FALLBACK_RGB` rather than restating its bytes, so
/// the fallback swatch and `COLOR_TEXT_FAINT` can never drift apart.
pub const FALLBACK_SWATCH: [u8; 3] = theme::SWATCH_FALLBACK_RGB;

pub fn profile_swatch_color(row: &ProfileRow) -> [u8; 3] {
    row.color.unwrap_or(FALLBACK_SWATCH)
}

/// A row reads as selected when the user clicked it or when it is the profile
/// the daemon has applied. This matches the behaviour before the redesign.
pub fn row_is_selected(app: &App, row: &ProfileRow) -> bool {
    app.selected_profile_id.as_deref() == Some(row.id.as_str()) || row.is_active
}

pub fn profile_list(app: &App) -> Element<'_, Message> {
    let mut list = column![].spacing(theme::SPACE_XS);
    for profile in &app.profiles {
        let selected = row_is_selected(app, profile);
        let entry = row![
            swatch(profile_swatch_color(profile), theme::SWATCH_SIZE),
            text(&profile.name).size(theme::TEXT_BODY),
        ]
        .spacing(theme::SPACE_SM)
        .align_y(Alignment::Center);

        let style = if selected { selected_row_button } else { row_button };
        list = list.push(
            button(entry)
                .width(Length::Fill)
                .padding([theme::SPACE_SM, theme::SPACE_MD])
                .on_press(Message::SelectProfile(profile.id.clone()))
                .style(style),
        );

        if profile.is_active && profile.can_revert {
            list = list.push(
                button(text(theme::BUTTON_REVERT).size(theme::TEXT_SMALL))
                    .on_press(Message::RevertProfile)
                    .style(accent_text_button),
            );
        }
    }

    container(
        column![
            section_label(theme::LABEL_PROFILES),
            scrollable(list).height(Length::Fill),
        ]
        .spacing(theme::SPACE_SM)
        .padding(theme::SPACE_MD)
        .height(Length::Fill),
    )
    .width(Length::Fixed(theme::PROFILE_LIST_WIDTH))
    .height(Length::Fill)
    .style(surface_container)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, Phase};

    fn row(id: &str, is_active: bool, color: Option<[u8; 3]>) -> ProfileRow {
        ProfileRow {
            id: id.into(),
            name: id.into(),
            is_active,
            can_revert: false,
            color,
        }
    }

    #[test]
    fn a_profile_without_a_colour_falls_back_to_the_faint_swatch() {
        assert_eq!(profile_swatch_color(&row("default", false, None)), FALLBACK_SWATCH);
        assert_eq!(
            profile_swatch_color(&row("x", false, Some([1, 2, 3]))),
            [1, 2, 3]
        );
    }

    #[test]
    fn the_active_row_and_the_clicked_row_both_read_as_selected() {
        // Struct-update syntax, not `App::default()` followed by a field
        // assignment: clippy's `field_reassign_with_default` flags the
        // reassignment form, and the workspace clippy budget is exact.
        let mut app = App {
            phase: Phase::Running,
            ..App::default()
        };

        let active = row("league-of-legends", true, None);
        assert!(row_is_selected(&app, &active), "the applied profile is selected");

        let other = row("dota-2", false, None);
        assert!(!row_is_selected(&app, &other));

        app.selected_profile_id = Some("dota-2".into());
        assert!(row_is_selected(&app, &other), "the clicked profile is selected");
    }
}
