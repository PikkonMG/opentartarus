use crate::app::{App, Message, ProfileRow};
use crate::keys::NEW_PROFILE_INPUT_ID;
use crate::theme;
use crate::view::widgets::{
    accent_text_button, danger_text_button, row_button, section_label, selected_row_button,
    surface_container, swatch,
};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
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

/// How far `Revert to shipped` is inset so it lines up under a row's name
/// rather than under its colour chip.
const REVERT_INDENT: f32 = theme::SWATCH_SIZE + theme::SPACE_SM + theme::SPACE_MD;

pub fn profile_list(app: &App) -> Element<'_, Message> {
    // Rows sit close together so the list reads as one column of names, not a
    // stack of separate controls.
    let mut list = column![].spacing(theme::SPACE_XXS);
    for profile in &app.profiles {
        let selected = row_is_selected(app, profile);
        let entry = row![
            swatch(profile_swatch_color(profile), theme::SWATCH_SIZE),
            text(&profile.name).size(theme::TEXT_BODY),
        ]
        .spacing(theme::SPACE_SM)
        .align_y(Alignment::Center);

        let style = if selected { selected_row_button } else { row_button };
        let select = button(entry)
            .width(Length::Fill)
            .padding([theme::SPACE_XS, theme::SPACE_MD])
            .on_press(Message::SelectProfile(profile.id.clone()))
            .style(style);

        // A custom profile carries its own delete control on the row. It is
        // a sibling of the select button, not a child of it: a button inside
        // a button would swallow the click.
        if row_is_deletable(profile) {
            list = list.push(
                row![
                    select,
                    button(text(theme::BUTTON_DELETE_PROFILE).size(theme::TEXT_BODY))
                        .padding([theme::SPACE_XXS, theme::SPACE_SM])
                        .on_press(Message::DeleteProfile(profile.id.clone()))
                        .style(danger_text_button),
                ]
                .spacing(theme::SPACE_XXS)
                .align_y(Alignment::Center),
            );
        } else {
            list = list.push(select);
        }

        if profile.is_active && profile.can_revert {
            list = list.push(
                button(text(theme::BUTTON_REVERT).size(theme::TEXT_SMALL))
                    .padding([theme::SPACE_XXS, REVERT_INDENT])
                    .on_press(Message::RevertProfile)
                    .style(accent_text_button),
            );
        }
    }

    container(
        column![
            section_label(theme::LABEL_PROFILES),
            scrollable(list).height(Length::Fill),
            new_profile_control(app),
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

/// The last thing in the list: `+ New profile` at rest, or the inline name
/// box once pressed. Enter creates, Escape or the `x` cancels. The new profile
/// is a copy of whatever is selected, so this is "save this layout as mine",
/// not "start from nothing".
fn new_profile_control(app: &App) -> Element<'_, Message> {
    match &app.new_profile_name {
        None => button(text(theme::BUTTON_NEW_PROFILE).size(theme::TEXT_BODY))
            .width(Length::Fill)
            .padding([theme::SPACE_XS, theme::SPACE_MD])
            .on_press(Message::StartNewProfile)
            .style(accent_text_button)
            .into(),
        Some(name) => row![
            // `text_input::Id`, not the advanced widget `Id` that focus
            // queries compare against: same string, two id types, the way
            // the combo box does it with `COMBO_INPUT_ID`.
            text_input(theme::NEW_PROFILE_PLACEHOLDER, name)
                .id(text_input::Id::new(NEW_PROFILE_INPUT_ID))
                .size(theme::TEXT_BODY)
                .on_input(Message::NewProfileNameChanged)
                .on_submit(Message::SubmitNewProfile),
            button(text(theme::BUTTON_CANCEL_NEW_PROFILE).size(theme::TEXT_BODY))
                .padding([theme::SPACE_XXS, theme::SPACE_SM])
                .on_press(Message::CancelNewProfile)
                .style(danger_text_button),
        ]
        .spacing(theme::SPACE_XXS)
        .align_y(Alignment::Center)
        .into(),
    }
}

/// Whether a row shows its own delete control. Only custom profiles do.
pub fn row_is_deletable(row: &ProfileRow) -> bool {
    row.can_delete
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
            can_delete: false,
            color,
        }
    }

    #[test]
    fn only_custom_rows_offer_delete_and_only_shipped_rows_offer_revert() {
        let mut shipped = row("league-of-legends", true, None);
        shipped.can_revert = true;
        assert!(!row_is_deletable(&shipped), "a shipped profile is never deletable");

        let mut custom = row("my-raid-layout", false, None);
        custom.can_delete = true;
        assert!(row_is_deletable(&custom));
        assert!(!custom.can_revert, "nothing shipped to revert a custom profile to");
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
