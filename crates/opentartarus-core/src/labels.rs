use crate::pack::SHIPPED_IDS;
use crate::types::{Action, KeyToken, Modifier, MouseButton, MouseTarget, ScrollDir};

pub fn bind_label(action: Option<&Action>) -> String {
    match action {
        None => String::new(),
        Some(Action::Key { key, modifiers }) => key_combo_label(*key, modifiers),
        Some(Action::Macro { .. }) => String::from("Macro"),
        Some(Action::Mouse { target }) => mouse_label(target),
        Some(Action::HoldRepeat { inner, .. }) => bind_label(Some(inner)),
    }
}

pub fn profile_row_order(ids: &[String]) -> Vec<String> {
    let mut ordered = Vec::with_capacity(ids.len());
    for shipped in SHIPPED_IDS {
        if ids.iter().any(|id| id == shipped) {
            ordered.push(shipped.to_string());
        }
    }
    for id in ids {
        if !SHIPPED_IDS.contains(&id.as_str()) {
            ordered.push(id.clone());
        }
    }
    ordered
}

fn key_combo_label(key: KeyToken, modifiers: &[Modifier]) -> String {
    let mut parts: Vec<String> = modifiers.iter().map(|m| modifier_label(*m)).collect();
    parts.push(token_label(key));
    parts.join("+")
}

fn modifier_label(modifier: Modifier) -> String {
    match modifier {
        Modifier::Ctrl => String::from("Ctrl"),
        Modifier::Shift => String::from("Shift"),
        Modifier::Alt => String::from("Alt"),
        Modifier::Super => String::from("Super"),
        Modifier::CtrlR => String::from("CtrlR"),
        Modifier::ShiftR => String::from("ShiftR"),
        Modifier::AltR => String::from("AltR"),
        Modifier::SuperR => String::from("SuperR"),
    }
}

fn token_label(key: KeyToken) -> String {
    match key {
        KeyToken::A => String::from("A"),
        KeyToken::B => String::from("B"),
        KeyToken::C => String::from("C"),
        KeyToken::D => String::from("D"),
        KeyToken::E => String::from("E"),
        KeyToken::F => String::from("F"),
        KeyToken::G => String::from("G"),
        KeyToken::H => String::from("H"),
        KeyToken::I => String::from("I"),
        KeyToken::J => String::from("J"),
        KeyToken::K => String::from("K"),
        KeyToken::L => String::from("L"),
        KeyToken::M => String::from("M"),
        KeyToken::N => String::from("N"),
        KeyToken::O => String::from("O"),
        KeyToken::P => String::from("P"),
        KeyToken::Q => String::from("Q"),
        KeyToken::R => String::from("R"),
        KeyToken::S => String::from("S"),
        KeyToken::T => String::from("T"),
        KeyToken::U => String::from("U"),
        KeyToken::V => String::from("V"),
        KeyToken::W => String::from("W"),
        KeyToken::X => String::from("X"),
        KeyToken::Y => String::from("Y"),
        KeyToken::Z => String::from("Z"),
        KeyToken::Num0 => String::from("0"),
        KeyToken::Num1 => String::from("1"),
        KeyToken::Num2 => String::from("2"),
        KeyToken::Num3 => String::from("3"),
        KeyToken::Num4 => String::from("4"),
        KeyToken::Num5 => String::from("5"),
        KeyToken::Num6 => String::from("6"),
        KeyToken::Num7 => String::from("7"),
        KeyToken::Num8 => String::from("8"),
        KeyToken::Num9 => String::from("9"),
        other => format!("{other:?}"),
    }
}

fn mouse_label(target: &MouseTarget) -> String {
    match target {
        MouseTarget::Button {
            button: MouseButton::Left,
        } => String::from("M1"),
        MouseTarget::Button {
            button: MouseButton::Right,
        } => String::from("M2"),
        MouseTarget::Button {
            button: MouseButton::Middle,
        } => String::from("M3"),
        MouseTarget::Button {
            button: MouseButton::Back,
        } => String::from("M4"),
        MouseTarget::Button {
            button: MouseButton::Forward,
        } => String::from("M5"),
        MouseTarget::Scroll {
            scroll: ScrollDir::Up,
        } => String::from("Wheel+"),
        MouseTarget::Scroll {
            scroll: ScrollDir::Down,
        } => String::from("Wheel-"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::SHIPPED_IDS;
    use crate::types::{
        Action, Edge, KeyToken, MacroKind, MacroStep, Modifier, MouseButton, MouseTarget, ScrollDir,
    };

    #[test]
    fn bind_labels() {
        assert_eq!(bind_label(None), "");
        assert_eq!(
            bind_label(Some(&Action::Key {
                key: KeyToken::Q,
                modifiers: vec![]
            })),
            "Q"
        );
        assert_eq!(
            bind_label(Some(&Action::Key {
                key: KeyToken::C,
                modifiers: vec![Modifier::Ctrl]
            })),
            "Ctrl+C"
        );
        assert_eq!(
            bind_label(Some(&Action::Macro {
                steps: vec![MacroStep {
                    kind: MacroKind::Key {
                        key: KeyToken::Num1,
                        modifiers: vec![]
                    },
                    edge: Edge::Tap,
                    delay_ms: 0,
                }],
            })),
            "Macro"
        );
        assert_eq!(
            bind_label(Some(&Action::Mouse {
                target: MouseTarget::Button {
                    button: MouseButton::Left
                }
            })),
            "M1"
        );
        assert_eq!(
            bind_label(Some(&Action::Mouse {
                target: MouseTarget::Scroll {
                    scroll: ScrollDir::Up
                }
            })),
            "Wheel+"
        );
    }

    #[test]
    fn profile_list_order_is_pack_order() {
        let ids = vec![
            "path-of-exile".into(),
            "default".into(),
            "league-of-legends".into(),
        ];
        assert_eq!(
            profile_row_order(&ids),
            vec!["default", "league-of-legends", "path-of-exile"]
        );
        assert_eq!(SHIPPED_IDS[0], "default");
    }
}
