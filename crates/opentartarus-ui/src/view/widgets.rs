use crate::app::{Message, Tab};
use crate::theme;
use iced::widget::{button, container, text, Space};
use iced::{Background, Border, Color, Element, Length, Theme};

// This module is the shared style vocabulary the view modules draw from.
// Every function below is covered by the tests at the bottom of this file.

/// A raised panel: the keypad card, the bind readout, the menu popup.
pub fn card_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme::COLOR_SURFACE)),
        text_color: Some(theme::COLOR_TEXT),
        border: Border {
            color: theme::COLOR_LINE,
            width: theme::BORDER_HAIRLINE,
            radius: theme::RADIUS_CARD.into(),
        },
        ..container::Style::default()
    }
}

/// A flat band: the header, the sidebar, the inspector, the status bar.
pub fn surface_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme::COLOR_SURFACE)),
        text_color: Some(theme::COLOR_TEXT),
        ..container::Style::default()
    }
}

fn control(
    background: Color,
    hovered: Color,
    text_color: Color,
    border_color: Color,
    radius: f32,
    border_width: f32,
    status: button::Status,
) -> button::Style {
    let fill = match status {
        button::Status::Hovered | button::Status::Pressed => hovered,
        _ => background,
    };
    button::Style {
        background: Some(Background::Color(fill)),
        text_color,
        border: Border {
            color: border_color,
            width: border_width,
            radius: radius.into(),
        },
        ..button::Style::default()
    }
}

/// The single accent-filled action on any screen.
pub fn primary_button(_theme: &Theme, status: button::Status) -> button::Style {
    control(
        theme::COLOR_ACCENT,
        theme::COLOR_ACCENT,
        Color::WHITE,
        theme::COLOR_ACCENT,
        theme::RADIUS_CONTROL,
        theme::BORDER_HAIRLINE,
        status,
    )
}

/// A normal secondary control.
pub fn quiet_button(_theme: &Theme, status: button::Status) -> button::Style {
    control(
        theme::COLOR_RAISED,
        theme::COLOR_KEY,
        theme::COLOR_TEXT,
        theme::COLOR_LINE,
        theme::RADIUS_CONTROL,
        theme::BORDER_HAIRLINE,
        status,
    )
}

/// A small rounded pill, used for the mouse targets.
pub fn chip_button(_theme: &Theme, status: button::Status) -> button::Style {
    control(
        theme::COLOR_RAISED,
        theme::COLOR_KEY,
        theme::COLOR_TEXT_DIM,
        theme::COLOR_LINE,
        theme::RADIUS_PILL,
        theme::BORDER_HAIRLINE,
        status,
    )
}

/// An unselected sidebar row.
pub fn row_button(_theme: &Theme, status: button::Status) -> button::Style {
    control(
        Color::TRANSPARENT,
        theme::COLOR_RAISED,
        theme::COLOR_TEXT_DIM,
        Color::TRANSPARENT,
        theme::RADIUS_CONTROL,
        theme::BORDER_NONE,
        status,
    )
}

/// The selected sidebar row: an accent tint carries the selection, with no
/// border in any state (redesign spec, sidebar section).
pub fn selected_row_button(_theme: &Theme, status: button::Status) -> button::Style {
    control(
        theme::COLOR_ACCENT_SOFT,
        theme::COLOR_ACCENT_SOFT,
        theme::COLOR_TEXT,
        Color::TRANSPARENT,
        theme::RADIUS_CONTROL,
        theme::BORDER_NONE,
        status,
    )
}

/// The selected lighting effect card. Unlike `selected_row_button`, a card
/// in a grid has no other cue for which effect is active, so this style
/// keeps a visible accent border to carry that meaning.
pub fn selected_effect_button(_theme: &Theme, status: button::Status) -> button::Style {
    control(
        theme::COLOR_ACCENT_SOFT,
        theme::COLOR_ACCENT_SOFT,
        theme::COLOR_TEXT,
        theme::COLOR_ACCENT,
        theme::RADIUS_CONTROL,
        theme::BORDER_SELECTED,
        status,
    )
}

/// Destructive text with no fill: Clear this binding, delete a macro step.
pub fn danger_text_button(_theme: &Theme, status: button::Status) -> button::Style {
    control(
        Color::TRANSPARENT,
        theme::with_opacity(theme::COLOR_DANGER, theme::ACCENT_SOFT_ALPHA),
        theme::COLOR_DANGER,
        Color::TRANSPARENT,
        theme::RADIUS_CONTROL,
        theme::BORDER_NONE,
        status,
    )
}

/// Accent-coloured text with no fill: a recovery action such as reverting a
/// profile, which should read as "go back," not as destructive.
pub fn accent_text_button(_theme: &Theme, status: button::Status) -> button::Style {
    control(
        Color::TRANSPARENT,
        theme::with_opacity(theme::COLOR_ACCENT, theme::ACCENT_SOFT_ALPHA),
        theme::COLOR_ACCENT,
        Color::TRANSPARENT,
        theme::RADIUS_CONTROL,
        theme::BORDER_NONE,
        status,
    )
}

/// A title-bar control: minimize and maximize. Transparent until hovered, so
/// the header reads as one surface rather than a row of buttons.
pub fn window_control_button(_theme: &Theme, status: button::Status) -> button::Style {
    control(
        Color::TRANSPARENT,
        theme::COLOR_RAISED,
        theme::COLOR_TEXT_DIM,
        Color::TRANSPARENT,
        theme::RADIUS_CONTROL,
        theme::BORDER_NONE,
        status,
    )
}

/// The title bar's close control. Same shape as its neighbours, but it turns
/// red on hover, the way a close button is expected to.
pub fn close_control_button(_theme: &Theme, status: button::Status) -> button::Style {
    let hovered = matches!(
        status,
        button::Status::Hovered | button::Status::Pressed
    );
    control(
        Color::TRANSPARENT,
        theme::COLOR_DANGER,
        if hovered {
            Color::WHITE
        } else {
            theme::COLOR_TEXT_DIM
        },
        Color::TRANSPARENT,
        theme::RADIUS_CONTROL,
        theme::BORDER_NONE,
        status,
    )
}

/// The small uppercase label above a group of controls.
/// Takes an owned string so callers can pass a freshly built label, such as
/// the brightness percentage, without fighting the borrow checker.
pub fn section_label<'a>(label: impl Into<String>) -> Element<'a, Message> {
    text(label.into().to_uppercase())
        .size(theme::TEXT_LABEL)
        .color(theme::COLOR_TEXT_FAINT)
        .into()
}

/// The status dot colour: green while the daemon runs, faint otherwise.
pub fn hairline_color(status_ok: bool) -> Color {
    if status_ok {
        theme::COLOR_OK
    } else {
        theme::COLOR_TEXT_FAINT
    }
}

/// A small filled circle, used in the header pill and the status bar.
pub fn dot<'a>(color: Color) -> Element<'a, Message> {
    container(Space::new(
        Length::Fixed(theme::STATUS_DOT_SIZE),
        Length::Fixed(theme::STATUS_DOT_SIZE),
    ))
    .style(move |_theme: &Theme| container::Style {
        background: Some(Background::Color(color)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: theme::RADIUS_PILL.into(),
        },
        ..container::Style::default()
    })
    .into()
}

/// A colour chip: the profile swatch, the lighting presets.
pub fn swatch<'a>(rgb: [u8; 3], size: f32) -> Element<'a, Message> {
    let color = Color::from_rgb8(rgb[0], rgb[1], rgb[2]);
    container(Space::new(Length::Fixed(size), Length::Fixed(size)))
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                color: theme::COLOR_LINE_STRONG,
                width: theme::BORDER_HAIRLINE,
                radius: theme::RADIUS_CONTROL.into(),
            },
            ..container::Style::default()
        })
        .into()
}

const TAB_ORDER: [(Tab, &str); 2] = [
    (Tab::Keys, theme::TAB_KEYS),
    (Tab::Lighting, theme::TAB_LIGHTING),
];

/// Returned as a function so the test can inspect both states without
/// rendering the strip.
pub fn tab_button_style(active: bool) -> fn(&Theme, button::Status) -> button::Style {
    if active {
        active_tab_button
    } else {
        idle_tab_button
    }
}

fn active_tab_button(_theme: &Theme, _status: button::Status) -> button::Style {
    control(
        theme::COLOR_RAISED,
        theme::COLOR_RAISED,
        theme::COLOR_TEXT,
        Color::TRANSPARENT,
        theme::RADIUS_CONTROL,
        theme::BORDER_NONE,
        button::Status::Active,
    )
}

fn idle_tab_button(_theme: &Theme, status: button::Status) -> button::Style {
    control(
        Color::TRANSPARENT,
        theme::COLOR_RAISED,
        theme::COLOR_TEXT_DIM,
        Color::TRANSPARENT,
        theme::RADIUS_CONTROL,
        theme::BORDER_NONE,
        status,
    )
}

/// The Keys / Lighting switcher shown above the centre panel. Lives here,
/// not in `keys_tab.rs` or `lighting_tab.rs`, so both tabs render the exact
/// same strip from one implementation.
pub fn tab_strip<'a>(active: Tab) -> Element<'a, Message> {
    let mut strip = iced::widget::row![].spacing(theme::SPACE_XS);
    for (tab, label) in TAB_ORDER {
        strip = strip.push(
            button(text(label).size(theme::TEXT_SMALL))
                .padding([theme::SPACE_SM, theme::SPACE_LG])
                .on_press(Message::SelectTab(tab))
                .style(tab_button_style(tab == active)),
        );
    }
    container(strip.padding(theme::SPACE_XS))
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(theme::COLOR_SURFACE)),
            border: Border {
                color: theme::COLOR_LINE,
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
    use iced::widget::button;

    // Excludes `primary_button`: it is deliberately accent-filled, and its
    // border is set to the same accent colour as its own fill (see
    // `primary_button` above), so it draws no visible outline. Also
    // excludes `selected_row_button` and `selected_effect_button`: both
    // mark a selected state with accent colour by design (a soft fill for
    // the row, an accent border for the effect card) — each has its own
    // dedicated test below. Every other control here must never carry an
    // accent-coloured border at rest.
    fn styles() -> [fn(&Theme, button::Status) -> button::Style; 5] {
        [
            quiet_button,
            chip_button,
            row_button,
            danger_text_button,
            accent_text_button,
        ]
    }

    #[test]
    fn only_the_selected_row_paints_an_accent_background() {
        let theme = theme::theme();
        let selected = selected_row_button(&theme, button::Status::Active);
        assert_eq!(
            selected.background,
            Some(Background::Color(theme::COLOR_ACCENT_SOFT))
        );
        assert_eq!(
            selected.border.width,
            theme::BORDER_NONE,
            "no border in any state"
        );
        assert_eq!(selected.border.color, Color::TRANSPARENT);
        let plain = row_button(&theme, button::Status::Active);
        assert_eq!(
            plain.background,
            Some(Background::Color(Color::TRANSPARENT))
        );
    }

    #[test]
    fn the_selected_effect_card_draws_a_visible_accent_border() {
        let theme = theme::theme();
        let selected = selected_effect_button(&theme, button::Status::Active);
        assert_eq!(selected.border.color, theme::COLOR_ACCENT);
        assert_eq!(selected.border.width, theme::BORDER_SELECTED);
        assert_eq!(
            selected.background,
            Some(Background::Color(theme::COLOR_ACCENT_SOFT))
        );
    }

    #[test]
    fn no_resting_control_draws_an_accent_border() {
        let theme = theme::theme();
        for style in styles() {
            let resting = style(&theme, button::Status::Active);
            assert_ne!(
                resting.border.color,
                theme::COLOR_ACCENT,
                "a resting control must not be outlined in the accent colour"
            );
        }
    }

    #[test]
    fn quiet_and_chip_controls_lift_on_hover() {
        let theme = theme::theme();
        for style in [quiet_button, chip_button, row_button] {
            let resting = style(&theme, button::Status::Active);
            let hovered = style(&theme, button::Status::Hovered);
            assert_ne!(
                resting.background, hovered.background,
                "hover must be visible"
            );
        }
    }

    #[test]
    fn the_primary_button_is_the_only_accent_filled_control() {
        let theme = theme::theme();
        assert_eq!(
            primary_button(&theme, button::Status::Active).background,
            Some(Background::Color(theme::COLOR_ACCENT))
        );
        for style in [
            quiet_button,
            chip_button,
            row_button,
            danger_text_button,
            accent_text_button,
        ] {
            assert_ne!(
                style(&theme, button::Status::Active).background,
                Some(Background::Color(theme::COLOR_ACCENT))
            );
        }
    }

    #[test]
    fn destructive_text_is_danger_coloured_and_unfilled() {
        let theme = theme::theme();
        let style = danger_text_button(&theme, button::Status::Active);
        assert_eq!(style.text_color, theme::COLOR_DANGER);
        assert_eq!(
            style.background,
            Some(Background::Color(Color::TRANSPARENT))
        );
    }

    #[test]
    fn revert_text_is_accent_coloured_unfilled_and_borderless() {
        let theme = theme::theme();
        let style = accent_text_button(&theme, button::Status::Active);
        assert_eq!(style.text_color, theme::COLOR_ACCENT);
        assert_eq!(
            style.background,
            Some(Background::Color(Color::TRANSPARENT))
        );
        assert_eq!(style.border.width, theme::BORDER_NONE);
        assert_eq!(style.border.color, Color::TRANSPARENT);
    }

    #[test]
    fn cards_are_more_rounded_than_controls() {
        let theme = theme::theme();
        let card = card_container(&theme);
        let control = quiet_button(&theme, button::Status::Active);
        // `assert!(theme::RADIUS_CARD > theme::RADIUS_CONTROL)` compares two
        // `const` values directly, so clippy folds it to a compile-time
        // constant and warns (`assertions_on_constants`). Reading both
        // through a runtime slice first keeps this a real, failure-revealing
        // check, matching the pattern used in theme/tokens.rs's own tests.
        let radii = [theme::RADIUS_CARD, theme::RADIUS_CONTROL];
        assert!(radii[0] > radii[1]);
        assert_eq!(
            card.background,
            Some(Background::Color(theme::COLOR_SURFACE))
        );
        assert_eq!(control.border.width, theme::BORDER_HAIRLINE);
    }

    #[test]
    fn the_status_dot_is_green_when_running_and_faint_otherwise() {
        assert_eq!(hairline_color(true), theme::COLOR_OK);
        assert_eq!(hairline_color(false), theme::COLOR_TEXT_FAINT);
    }

    #[test]
    fn the_tab_strip_marks_exactly_one_tab_active() {
        let theme = theme::theme();
        let active = tab_button_style(true)(&theme, button::Status::Active);
        let idle = tab_button_style(false)(&theme, button::Status::Active);
        assert_eq!(active.background, Some(Background::Color(theme::COLOR_RAISED)));
        assert_eq!(idle.background, Some(Background::Color(Color::TRANSPARENT)));
        assert_eq!(active.text_color, theme::COLOR_TEXT);
        assert_eq!(idle.text_color, theme::COLOR_TEXT_DIM);
    }
}
