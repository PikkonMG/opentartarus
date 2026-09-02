use crate::app::{uses_color, App, Banner, Message, LIGHTING_EFFECTS};
use crate::keypad::{self, Keypad};
use crate::keys::COMBO_INPUT_ID;
use crate::theme::{
    self, BIND_PANEL_WIDTH, COLOR_ACCENT, COLOR_BACKGROUND, COLOR_DANGER, COLOR_SURFACE,
    COLOR_TEXT, LIGHTING_STRIP_HEIGHT, PROFILE_LIST_WIDTH,
};
use iced::widget::{
    button, checkbox, column, container, pick_list, row, scrollable, slider, text, text_input,
    Space,
};
use iced::{Background, Border, Color, Element, Length, Theme};
use opentartarus_core::error::ErrorCode;
use opentartarus_core::labels::bind_label;
use opentartarus_core::types::{Action, MouseButton, MouseTarget, ScrollDir};

pub fn view(app: &App) -> Element<'_, Message> {
    let banner = banner_bar(app);
    let body = row![
        profile_list(app),
        keypad::widget(Keypad {
            model: app.model,
            bindings: app.bindings.clone(),
            selected: app.selected_key,
            faded: app.keypad_faded(),
        }),
        bind_panel(app),
    ]
    .spacing(8)
    .height(Length::Fill);
    let lighting = lighting_strip(app);
    container(
        column![banner, body, lighting]
            .spacing(8)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(Background::Color(COLOR_BACKGROUND)),
        text_color: Some(COLOR_TEXT),
        ..container::Style::default()
    })
    .into()
}

fn banner_bar(app: &App) -> Element<'_, Message> {
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
                .style(danger_button),
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

fn profile_list(app: &App) -> Element<'_, Message> {
    let mut list = column![].spacing(4);
    for row in &app.profiles {
        let selected = app.selected_profile_id.as_deref() == Some(row.id.as_str()) || row.is_active;
        let label = text(&row.name).size(14);
        let mut btn = button(label)
            .width(Length::Fill)
            .on_press(Message::SelectProfile(row.id.clone()));
        if selected {
            btn = btn.style(selected_button);
        } else {
            btn = btn.style(surface_button);
        }
        list = list.push(btn);
        if row.is_active && row.can_revert {
            list = list.push(
                button(text(theme::BUTTON_REVERT).size(12))
                    .on_press(Message::RevertProfile)
                    .style(surface_button),
            );
        }
    }
    let quit = button(text(theme::BUTTON_QUIT))
        .width(Length::Fill)
        .on_press(Message::Quit)
        .style(danger_button);
    container(
        column![scrollable(list.padding(8)).height(Length::Fill), quit,]
            .spacing(8)
            .padding(8)
            .height(Length::Fill),
    )
    .width(Length::Fixed(PROFILE_LIST_WIDTH))
    .height(Length::Fill)
    .style(surface_container)
    .into()
}

fn bind_panel(app: &App) -> Element<'_, Message> {
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
                .style(accent_button),
            text_input(theme::COMBO_PLACEHOLDER, &app.combo_text)
                .id(iced::widget::text_input::Id::new(COMBO_INPUT_ID))
                .on_input(Message::ComboChanged),
            button(text(theme::BUTTON_CLEAR))
                .on_press(Message::ClearBinding)
                .style(surface_button),
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
                            .style(danger_button),
                    ]
                    .spacing(6)
                    .align_y(iced::Alignment::Center),
                );
            }
            col = col.push(
                button(text(theme::BUTTON_ADD_STEP))
                    .on_press(Message::AddMacroStep)
                    .style(surface_button),
            );
        }
        scrollable(col).into()
    };
    container(content)
        .width(Length::Fixed(BIND_PANEL_WIDTH))
        .height(Length::Fill)
        .style(surface_container)
        .into()
}

fn mouse_btn(label: &'static str, target: MouseTarget) -> Element<'static, Message> {
    button(text(label).size(11))
        .on_press(Message::MousePick(target))
        .style(surface_button)
        .into()
}

fn lighting_strip(app: &App) -> Element<'_, Message> {
    let mut row = row![].spacing(10).align_y(iced::Alignment::Center);
    if let Some(copy) = app.lighting_footer_message() {
        row = row.push(text(copy).size(13));
    } else if app.openrazer {
        let selected = LIGHTING_EFFECTS
            .iter()
            .copied()
            .find(|e| e.0 == app.lighting.effect);
        row = row.push(
            pick_list(LIGHTING_EFFECTS, selected, Message::LightingEffect)
                .placeholder("effect")
                .text_size(13)
                .width(Length::Fixed(120.0)),
        );
        row = row.push(
            slider(
                theme::BRIGHTNESS_MIN..=theme::BRIGHTNESS_MAX,
                app.lighting.brightness,
                Message::LightingBrightness,
            )
            .width(Length::Fixed(140.0)),
        );
        if uses_color(app.lighting.effect) {
            let rgb = app.lighting.color.unwrap_or(theme::DEFAULT_LIGHT_COLOR);
            row = row.push(color_swatch(rgb));
            row = row.push(
                slider(0..=theme::COLOR_CHANNEL_MAX, rgb[0], |v| {
                    Message::LightingColor(0, v)
                })
                .width(80.0),
            );
            row = row.push(
                slider(0..=theme::COLOR_CHANNEL_MAX, rgb[1], |v| {
                    Message::LightingColor(1, v)
                })
                .width(80.0),
            );
            row = row.push(
                slider(0..=theme::COLOR_CHANNEL_MAX, rgb[2], |v| {
                    Message::LightingColor(2, v)
                })
                .width(80.0),
            );
        }
    }
    container(row.padding([0, 10]))
        .width(Length::Fill)
        .height(Length::Fixed(LIGHTING_STRIP_HEIGHT))
        .style(surface_container)
        .into()
}

fn color_swatch<'a>(rgb: [u8; 3]) -> Element<'a, Message> {
    container(Space::new(Length::Fixed(22.0), Length::Fixed(22.0)))
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb8(rgb[0], rgb[1], rgb[2]))),
            border: Border {
                color: COLOR_TEXT,
                width: 1.0,
                radius: 3.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn surface_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(COLOR_SURFACE)),
        text_color: Some(COLOR_TEXT),
        border: Border {
            color: COLOR_SURFACE,
            width: 0.0,
            radius: 6.0.into(),
        },
        ..container::Style::default()
    }
}

fn surface_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => COLOR_ACCENT,
        _ => COLOR_SURFACE,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: COLOR_TEXT,
        border: Border {
            color: COLOR_ACCENT,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..button::Style::default()
    }
}

fn selected_button(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(COLOR_ACCENT)),
        text_color: COLOR_TEXT,
        border: Border {
            color: COLOR_ACCENT,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..button::Style::default()
    }
}

fn accent_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => COLOR_ACCENT,
        _ => COLOR_ACCENT,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: COLOR_TEXT,
        border: Border {
            color: COLOR_ACCENT,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..button::Style::default()
    }
}

fn danger_button(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(COLOR_DANGER)),
        text_color: COLOR_TEXT,
        border: Border {
            color: COLOR_DANGER,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..button::Style::default()
    }
}
