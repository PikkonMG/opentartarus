use crate::app::{App, Message};
use crate::keys::COMBO_INPUT_ID;
use crate::theme;
use crate::view::widgets::{
    chip_button, danger_text_button, primary_button, quiet_button, section_label,
    surface_container,
};
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};
use opentartarus_core::labels::{bind_label, key_display_name, key_position_text};
use opentartarus_core::types::{Action, MouseButton, MouseTarget, ScrollDir};

pub const MOUSE_TARGETS: [(&str, MouseTarget); 7] = [
    (theme::MOUSE_LEFT, MouseTarget::Button { button: MouseButton::Left }),
    (theme::MOUSE_RIGHT, MouseTarget::Button { button: MouseButton::Right }),
    (theme::MOUSE_MIDDLE, MouseTarget::Button { button: MouseButton::Middle }),
    (theme::MOUSE_BACK, MouseTarget::Button { button: MouseButton::Back }),
    (theme::MOUSE_FORWARD, MouseTarget::Button { button: MouseButton::Forward }),
    (theme::MOUSE_WHEEL_UP, MouseTarget::Scroll { scroll: ScrollDir::Up }),
    (theme::MOUSE_WHEEL_DOWN, MouseTarget::Scroll { scroll: ScrollDir::Down }),
];

/// iced 0.13 has no flow layout, so the chips wrap in two fixed rows.
const MOUSE_ROW_ONE: usize = 4;

/// What the big readout shows, and whether the hold-repeat toggle is live.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoldRepeat {
    pub on: bool,
    pub enabled: bool,
}

pub fn bind_readout(action: Option<&Action>) -> String {
    let label = mouse_target_readout(action).unwrap_or_else(|| bind_label(action));
    if label.is_empty() {
        String::from(theme::UNBOUND_PLACEHOLDER)
    } else {
        label
    }
}

/// The big readout names a mouse target the same way the picker chips do
/// ("Left", "Wheel+"), not `bind_label`'s compact form ("M1"): that form
/// is for the tight space on a keycap in `keypad.rs`, not this headline
/// value. Looks the label up in `MOUSE_TARGETS` rather than restating it,
/// so the readout and the chip that picked it can never say different
/// things. `HoldRepeat` is unwrapped the same way `bind_label` unwraps it.
fn mouse_target_readout(action: Option<&Action>) -> Option<String> {
    match action {
        Some(Action::Mouse { target }) => MOUSE_TARGETS
            .iter()
            .find(|(_, candidate)| candidate == target)
            .map(|(label, _)| String::from(*label)),
        Some(Action::HoldRepeat { inner, .. }) => mouse_target_readout(Some(inner)),
        _ => None,
    }
}

/// Only a plain key can repeat. A hold-repeat that already wraps something
/// else reads as on, but the toggle stays dead so the user cannot make it
/// worse.
pub fn hold_repeat_state(action: Option<&Action>) -> HoldRepeat {
    match action {
        Some(Action::Key { .. }) => HoldRepeat { on: false, enabled: true },
        Some(Action::HoldRepeat { inner, .. }) => HoldRepeat {
            on: true,
            enabled: matches!(inner.as_ref(), Action::Key { .. }),
        },
        _ => HoldRepeat { on: false, enabled: false },
    }
}

fn empty_state<'a>() -> Element<'a, Message> {
    container(
        column![
            text(theme::EMPTY_STATE_TITLE)
                .size(theme::TEXT_HEADING)
                .color(theme::COLOR_TEXT_DIM),
            text(theme::EMPTY_STATE_BODY)
                .size(theme::TEXT_BODY)
                .color(theme::COLOR_TEXT_FAINT),
        ]
        .spacing(theme::SPACE_SM)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .into()
}

fn readout<'a>(action: Option<&Action>) -> Element<'a, Message> {
    container(
        column![
            section_label(theme::LABEL_SENDS),
            text(bind_readout(action)).size(theme::BIND_VALUE_TEXT),
        ]
        .spacing(theme::SPACE_XS)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(theme::SPACE_LG)
    .style(|_theme: &Theme| container::Style {
        background: Some(Background::Color(theme::COLOR_RAISED)),
        text_color: Some(theme::COLOR_TEXT),
        border: Border {
            color: theme::COLOR_LINE,
            width: theme::BORDER_HAIRLINE,
            radius: theme::RADIUS_CONTROL.into(),
        },
        ..container::Style::default()
    })
    .into()
}

fn switch<'a>(state: HoldRepeat) -> Element<'a, Message> {
    let fill = if state.on {
        theme::COLOR_ACCENT
    } else {
        theme::COLOR_RAISED
    };
    let track = container(Space::new(
        Length::Fixed(theme::SWITCH_WIDTH),
        Length::Fixed(theme::SWITCH_HEIGHT),
    ))
    .style(move |_theme: &Theme| container::Style {
        background: Some(Background::Color(fill)),
        border: Border {
            color: Color::TRANSPARENT,
            width: theme::BORDER_NONE,
            radius: theme::RADIUS_PILL.into(),
        },
        ..container::Style::default()
    });

    let label_color = if state.enabled {
        theme::COLOR_TEXT
    } else {
        theme::COLOR_TEXT_FAINT
    };
    let mut toggle = button(track).padding(0).style(quiet_button);
    if state.enabled {
        toggle = toggle.on_press(Message::HoldRepeat(!state.on));
    }

    row![
        text(theme::HOLD_REPEAT_LABEL)
            .size(theme::TEXT_BODY)
            .color(label_color),
        Space::with_width(Length::Fill),
        toggle,
    ]
    .align_y(Alignment::Center)
    .into()
}

fn mouse_chips<'a>() -> Element<'a, Message> {
    let chip = |label: &'static str, target: MouseTarget| {
        button(text(label).size(theme::TEXT_SMALL))
            .padding([theme::SPACE_XS, theme::SPACE_MD])
            .on_press(Message::MousePick(target))
            .style(chip_button)
    };
    let mut first = row![].spacing(theme::SPACE_XS);
    let mut second = row![].spacing(theme::SPACE_XS);
    for (index, (label, target)) in MOUSE_TARGETS.into_iter().enumerate() {
        if index < MOUSE_ROW_ONE {
            first = first.push(chip(label, target));
        } else {
            second = second.push(chip(label, target));
        }
    }
    column![section_label(theme::LABEL_MOUSE), first, second]
        .spacing(theme::SPACE_SM)
        .into()
}

fn macro_steps<'a>(action: &Action) -> Element<'a, Message> {
    let Action::Macro { steps } = action else {
        return Space::with_height(0).into();
    };
    let mut list = column![section_label(theme::LABEL_MACRO)].spacing(theme::SPACE_SM);
    for (index, step) in steps.iter().enumerate() {
        let delay = step.delay_ms.to_string();
        list = list.push(
            row![
                text(format!("{}{}", theme::MACRO_STEP_PREFIX, index + 1))
                    .size(theme::TEXT_SMALL)
                    .width(Length::Fill),
                text_input(theme::MACRO_DELAY_PLACEHOLDER, &delay)
                    .size(theme::TEXT_SMALL)
                    .on_input(move |raw| Message::MacroDelay(index, raw))
                    .width(Length::Fixed(theme::MENU_WIDTH / 3.0)),
                button(text(theme::BUTTON_DELETE_STEP))
                    .on_press(Message::DeleteMacroStep(index))
                    .style(danger_text_button),
            ]
            .spacing(theme::SPACE_SM)
            .align_y(Alignment::Center),
        );
    }
    list = list.push(
        button(text(theme::BUTTON_ADD_STEP).size(theme::TEXT_SMALL))
            .on_press(Message::AddMacroStep)
            .style(quiet_button),
    );
    list.into()
}

pub fn inspector(app: &App) -> Element<'_, Message> {
    let content: Element<'_, Message> = match app.selected_key {
        None => empty_state(),
        Some(key_id) => {
            let action = app.selected_action();
            let state = hold_repeat_state(action.as_ref());
            let record_label = if app.recording {
                theme::BUTTON_CANCEL
            } else {
                theme::BUTTON_RECORD_KEY
            };
            let record_message = if app.recording {
                Message::StopRecord
            } else {
                Message::Record
            };

            let mut panel = column![
                row![
                    text(key_display_name(key_id)).size(theme::TEXT_TITLE),
                    text(key_position_text(key_id))
                        .size(theme::TEXT_SMALL)
                        .color(theme::COLOR_TEXT_FAINT),
                ]
                .spacing(theme::SPACE_SM)
                .align_y(Alignment::End),
                readout(action.as_ref()),
                button(text(record_label).size(theme::TEXT_BODY))
                    .width(Length::Fill)
                    .padding(theme::SPACE_SM)
                    .on_press(record_message)
                    .style(primary_button),
                text_input(theme::COMBO_PLACEHOLDER, &app.combo_text)
                    .id(iced::widget::text_input::Id::new(COMBO_INPUT_ID))
                    .size(theme::TEXT_BODY)
                    .on_input(Message::ComboChanged),
                switch(state),
                mouse_chips(),
            ]
            .spacing(theme::SPACE_MD);

            if let Some(macro_action @ Action::Macro { .. }) = &action {
                panel = panel.push(macro_steps(macro_action));
            }

            panel = panel.push(Space::with_height(Length::Fill));
            panel = panel.push(
                button(text(theme::BUTTON_CLEAR_BINDING).size(theme::TEXT_SMALL))
                    .width(Length::Fill)
                    .padding(theme::SPACE_SM)
                    .on_press(Message::ClearBinding)
                    .style(danger_text_button),
            );
            scrollable(panel).height(Length::Fill).into()
        }
    };

    container(content)
        .width(Length::Fixed(theme::INSPECTOR_WIDTH))
        .height(Length::Fill)
        .padding(theme::SPACE_LG)
        .style(surface_container)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentartarus_core::types::{
        Action, KeyToken, MouseButton, MouseTarget,
    };

    fn key(token: KeyToken) -> Action {
        Action::Key { key: token, modifiers: vec![] }
    }

    #[test]
    fn an_unbound_key_reads_as_a_dash_not_an_empty_box() {
        assert_eq!(bind_readout(None), theme::UNBOUND_PLACEHOLDER);
    }

    #[test]
    fn a_bound_key_reads_as_its_label() {
        assert_eq!(bind_readout(Some(&key(KeyToken::Q))), "Q");
        let mouse = Action::Mouse {
            target: MouseTarget::Button { button: MouseButton::Left },
        };
        assert_eq!(bind_readout(Some(&mouse)), theme::MOUSE_LEFT);
    }

    #[test]
    fn hold_repeat_is_offered_only_for_plain_key_bindings() {
        let plain = hold_repeat_state(Some(&key(KeyToken::Q)));
        assert!(plain.enabled, "a plain key can repeat");
        assert!(!plain.on);

        let held = Action::HoldRepeat {
            inner: Box::new(key(KeyToken::Q)),
            rate_ms: 40,
        };
        let held = hold_repeat_state(Some(&held));
        assert!(held.enabled);
        assert!(held.on, "an existing hold-repeat reads as on");
    }

    #[test]
    fn hold_repeat_is_refused_for_macros_mouse_and_nothing() {
        let macro_action = Action::Macro { steps: Vec::new() };
        assert!(!hold_repeat_state(Some(&macro_action)).enabled);

        let mouse = Action::Mouse {
            target: MouseTarget::Button { button: MouseButton::Right },
        };
        assert!(!hold_repeat_state(Some(&mouse)).enabled);

        let nothing = hold_repeat_state(None);
        assert!(!nothing.enabled);
        assert!(!nothing.on);
    }

    #[test]
    fn hold_repeat_wrapping_a_macro_is_not_editable() {
        let odd = Action::HoldRepeat {
            inner: Box::new(Action::Macro { steps: Vec::new() }),
            rate_ms: 40,
        };
        let state = hold_repeat_state(Some(&odd));
        assert!(state.on, "it is on, because it is wrapped");
        assert!(!state.enabled, "but the toggle must not offer to change it");
    }

    #[test]
    fn every_mouse_target_gets_exactly_one_chip() {
        assert_eq!(MOUSE_TARGETS.len(), 7);
        let mut labels: Vec<&str> = MOUSE_TARGETS.iter().map(|(label, _)| *label).collect();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), MOUSE_TARGETS.len(), "labels must be unique");
    }
}
