use crate::app::{App, Message};
use crate::keys::COMBO_INPUT_ID;
use crate::theme::{self, INSPECTOR_WIDTH};
use crate::view::widgets::{danger_text_button, primary_button, quiet_button, surface_container};
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};
use opentartarus_core::labels::bind_label;
use opentartarus_core::types::{Action, MouseButton, MouseTarget, ScrollDir};

pub fn inspector(app: &App) -> Element<'_, Message> {
    let content: Element<'_, Message> = if app.selected_key.is_none() {
        text("").into()
    } else {
        let action = app.selected_action();
        let summary = bind_label(action.as_ref());
        let record_label = if app.recording {
            theme::BUTTON_CANCEL
        } else {
            theme::BUTTON_RECORD
        };
        let record_msg = if app.recording {
            Message::StopRecord
        } else {
            Message::Record
        };
        let hold_on = matches!(action, Some(Action::HoldRepeat { .. }));
        let hold_enabled = match &action {
            Some(Action::Key { .. }) => true,
            Some(Action::HoldRepeat { inner, .. }) => matches!(inner.as_ref(), Action::Key { .. }),
            _ => false,
        };
        let mut hold = checkbox(theme::HOLD_REPEAT_LABEL, hold_on);
        if hold_enabled {
            hold = hold.on_toggle(Message::HoldRepeat);
        }
        let mouse_row = row![
            mouse_btn(
                theme::MOUSE_LEFT,
                MouseTarget::Button {
                    button: MouseButton::Left
                }
            ),
            mouse_btn(
                theme::MOUSE_RIGHT,
                MouseTarget::Button {
                    button: MouseButton::Right
                }
            ),
            mouse_btn(
                theme::MOUSE_MIDDLE,
                MouseTarget::Button {
                    button: MouseButton::Middle
                }
            ),
            mouse_btn(
                theme::MOUSE_BACK,
                MouseTarget::Button {
                    button: MouseButton::Back
                }
            ),
            mouse_btn(
                theme::MOUSE_FORWARD,
                MouseTarget::Button {
                    button: MouseButton::Forward
                }
            ),
            mouse_btn(
                theme::MOUSE_WHEEL_UP,
                MouseTarget::Scroll {
                    scroll: ScrollDir::Up
                }
            ),
            mouse_btn(
                theme::MOUSE_WHEEL_DOWN,
                MouseTarget::Scroll {
                    scroll: ScrollDir::Down
                }
            ),
        ]
        .spacing(4);
        let mut col = column![
            text(summary).size(16),
            button(text(record_label))
                .on_press(record_msg)
                .style(primary_button),
            text_input(theme::COMBO_PLACEHOLDER, &app.combo_text)
                .id(iced::widget::text_input::Id::new(COMBO_INPUT_ID))
                .on_input(Message::ComboChanged),
            button(text(theme::BUTTON_CLEAR))
                .on_press(Message::ClearBinding)
                .style(quiet_button),
            hold,
            mouse_row,
        ]
        .spacing(8)
        .padding(8);
        if let Some(Action::Macro { steps }) = &action {
            col = col.push(text("Macro").size(14));
            for (i, step) in steps.iter().enumerate() {
                let delay = step.delay_ms.to_string();
                col = col.push(
                    row![
                        text(format!("Step {}", i + 1)).width(Length::Fill),
                        text_input("ms", &delay)
                            .on_input(move |s| Message::MacroDelay(i, s))
                            .width(Length::Fixed(64.0)),
                        button(text("×"))
                            .on_press(Message::DeleteMacroStep(i))
                            .style(danger_text_button),
                    ]
                    .spacing(6)
                    .align_y(iced::Alignment::Center),
                );
            }
            col = col.push(
                button(text(theme::BUTTON_ADD_STEP))
                    .on_press(Message::AddMacroStep)
                    .style(quiet_button),
            );
        }
        scrollable(col).into()
    };
    container(content)
        .width(Length::Fixed(INSPECTOR_WIDTH))
        .height(Length::Fill)
        .style(surface_container)
        .into()
}

fn mouse_btn(label: &'static str, target: MouseTarget) -> Element<'static, Message> {
    button(text(label).size(11))
        .on_press(Message::MousePick(target))
        .style(quiet_button)
        .into()
}
