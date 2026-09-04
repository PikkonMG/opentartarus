use crate::pack::SHIPPED_IDS;
use crate::types::{Action, KeyId, KeyToken, Modifier, MouseButton, MouseTarget, ScrollDir};

/// The four rows on the face of the pad. `kp20` is not here: it is the thumb
/// key, which sits below the rows on the physical device.
const GRID_ROW_LENGTHS: [usize; 4] = [5, 5, 5, 4];
const GRID_ROW_KEYS: [KeyId; 19] = [
    KeyId::Kp01,
    KeyId::Kp02,
    KeyId::Kp03,
    KeyId::Kp04,
    KeyId::Kp05,
    KeyId::Kp06,
    KeyId::Kp07,
    KeyId::Kp08,
    KeyId::Kp09,
    KeyId::Kp10,
    KeyId::Kp11,
    KeyId::Kp12,
    KeyId::Kp13,
    KeyId::Kp14,
    KeyId::Kp15,
    KeyId::Kp16,
    KeyId::Kp17,
    KeyId::Kp18,
    KeyId::Kp19,
];

const POSITION_THUMB_KEY: &str = "thumb key";
const POSITION_THUMB_PAD: &str = "thumb pad";
const POSITION_ANALOG: &str = "analog stick";
const POSITION_WHEEL: &str = "wheel";
const POSITION_WHEEL_COLUMN: &str = "wheel column";

/// Keycap labels for the profile-switch actions. Short, because a cap is
/// narrow; the inspector spells out which profile.
const SWITCH_PROFILE_LABEL: &str = "Profile";
const NEXT_PROFILE_LABEL: &str = "Next";

/// The name shown as the inspector heading, for example `Key 07` or
/// `Thumb north`.
pub fn key_display_name(id: KeyId) -> String {
    match id {
        KeyId::Kp01 => String::from("Key 01"),
        KeyId::Kp02 => String::from("Key 02"),
        KeyId::Kp03 => String::from("Key 03"),
        KeyId::Kp04 => String::from("Key 04"),
        KeyId::Kp05 => String::from("Key 05"),
        KeyId::Kp06 => String::from("Key 06"),
        KeyId::Kp07 => String::from("Key 07"),
        KeyId::Kp08 => String::from("Key 08"),
        KeyId::Kp09 => String::from("Key 09"),
        KeyId::Kp10 => String::from("Key 10"),
        KeyId::Kp11 => String::from("Key 11"),
        KeyId::Kp12 => String::from("Key 12"),
        KeyId::Kp13 => String::from("Key 13"),
        KeyId::Kp14 => String::from("Key 14"),
        KeyId::Kp15 => String::from("Key 15"),
        KeyId::Kp16 => String::from("Key 16"),
        KeyId::Kp17 => String::from("Key 17"),
        KeyId::Kp18 => String::from("Key 18"),
        KeyId::Kp19 => String::from("Key 19"),
        KeyId::Kp20 => String::from("Key 20"),
        KeyId::WheelUp => String::from("Wheel up"),
        KeyId::WheelDown => String::from("Wheel down"),
        KeyId::WheelClick => String::from("Wheel click"),
        KeyId::Mode => String::from("Mode"),
        KeyId::ThumbN => String::from("Thumb north"),
        KeyId::ThumbNe => String::from("Thumb north-east"),
        KeyId::ThumbE => String::from("Thumb east"),
        KeyId::ThumbSe => String::from("Thumb south-east"),
        KeyId::ThumbS => String::from("Thumb south"),
        KeyId::ThumbSw => String::from("Thumb south-west"),
        KeyId::ThumbW => String::from("Thumb west"),
        KeyId::ThumbNw => String::from("Thumb north-west"),
        KeyId::AnalogUp => String::from("Analog up"),
        KeyId::AnalogDown => String::from("Analog down"),
        KeyId::AnalogLeft => String::from("Analog left"),
        KeyId::AnalogRight => String::from("Analog right"),
    }
}

/// Where the key sits on the device, shown under the inspector heading.
pub fn key_position_text(id: KeyId) -> String {
    if id == KeyId::Kp20 {
        return String::from(POSITION_THUMB_KEY);
    }
    if let Some(index) = GRID_ROW_KEYS.iter().position(|candidate| *candidate == id) {
        let mut remaining = index;
        for (row, length) in GRID_ROW_LENGTHS.iter().enumerate() {
            if remaining < *length {
                return format!("row {} · key {}", row + 1, remaining + 1);
            }
            remaining -= *length;
        }
    }
    match id {
        KeyId::WheelUp | KeyId::WheelDown | KeyId::WheelClick => String::from(POSITION_WHEEL),
        KeyId::Mode => String::from(POSITION_WHEEL_COLUMN),
        KeyId::AnalogUp | KeyId::AnalogDown | KeyId::AnalogLeft | KeyId::AnalogRight => {
            String::from(POSITION_ANALOG)
        }
        _ => String::from(POSITION_THUMB_PAD),
    }
}

pub fn bind_label(action: Option<&Action>) -> String {
    match action {
        None => String::new(),
        Some(Action::Key { key, modifiers }) => key_combo_label(*key, modifiers),
        Some(Action::Macro { .. }) => String::from("Macro"),
        Some(Action::Mouse { target }) => mouse_label(target),
        Some(Action::SwitchProfile { .. }) => String::from(SWITCH_PROFILE_LABEL),
        Some(Action::NextProfile) => String::from(NEXT_PROFILE_LABEL),
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
        KeyToken::CapsLock => String::from("Caps"),
        KeyToken::LeftShift => String::from("Shift"),
        KeyToken::LeftCtrl => String::from("Ctrl"),
        KeyToken::LeftAlt => String::from("Alt"),
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
    fn bare_modifier_keys_read_like_the_modifiers_do() {
        for (token, label) in [
            (KeyToken::CapsLock, "Caps"),
            (KeyToken::LeftShift, "Shift"),
            (KeyToken::LeftCtrl, "Ctrl"),
            (KeyToken::LeftAlt, "Alt"),
        ] {
            assert_eq!(
                bind_label(Some(&Action::Key {
                    key: token,
                    modifiers: vec![]
                })),
                label
            );
        }
    }

    #[test]
    fn switch_labels_are_short_enough_for_a_keycap() {
        assert_eq!(
            bind_label(Some(&Action::SwitchProfile {
                profile: "dota-2".into()
            })),
            "Profile"
        );
        assert_eq!(bind_label(Some(&Action::NextProfile)), "Next");
    }

    #[test]
    fn profile_list_order_is_pack_order() {
        let ids = vec!["path-of-exile".into(), "default".into(), "dota-2".into()];
        assert_eq!(
            profile_row_order(&ids),
            vec!["default", "dota-2", "path-of-exile"]
        );
        assert_eq!(SHIPPED_IDS[0], "default");
    }

    #[test]
    fn grid_keys_are_named_and_placed_by_row() {
        assert_eq!(key_display_name(KeyId::Kp01), "Key 01");
        assert_eq!(key_position_text(KeyId::Kp01), "row 1 · key 1");
        assert_eq!(key_display_name(KeyId::Kp07), "Key 07");
        assert_eq!(key_position_text(KeyId::Kp07), "row 2 · key 2");
        assert_eq!(key_display_name(KeyId::Kp15), "Key 15");
        assert_eq!(key_position_text(KeyId::Kp15), "row 3 · key 5");
        assert_eq!(key_display_name(KeyId::Kp19), "Key 19");
        assert_eq!(key_position_text(KeyId::Kp19), "row 4 · key 4");
    }

    #[test]
    fn thumb_key_is_named_as_the_thumb_key_not_row_five() {
        assert_eq!(key_display_name(KeyId::Kp20), "Key 20");
        assert_eq!(key_position_text(KeyId::Kp20), "thumb key");
    }

    #[test]
    fn wheel_and_mode_have_their_own_names() {
        assert_eq!(key_display_name(KeyId::WheelUp), "Wheel up");
        assert_eq!(key_display_name(KeyId::WheelDown), "Wheel down");
        assert_eq!(key_display_name(KeyId::WheelClick), "Wheel click");
        assert_eq!(key_display_name(KeyId::Mode), "Mode");
        assert_eq!(key_position_text(KeyId::WheelUp), "wheel");
        assert_eq!(key_position_text(KeyId::Mode), "wheel column");
    }

    #[test]
    fn thumb_directions_and_analog_are_distinguishable() {
        assert_eq!(key_display_name(KeyId::ThumbN), "Thumb north");
        assert_eq!(key_display_name(KeyId::ThumbSw), "Thumb south-west");
        assert_eq!(key_position_text(KeyId::ThumbN), "thumb pad");
        assert_eq!(key_display_name(KeyId::AnalogLeft), "Analog left");
        assert_eq!(key_position_text(KeyId::AnalogLeft), "analog stick");
    }

    #[test]
    fn every_key_id_has_a_unique_name_and_a_position() {
        let mut names = Vec::with_capacity(KeyId::ALL.len());
        for id in KeyId::ALL {
            let name = key_display_name(id);
            assert!(!name.is_empty(), "name for {id:?}");
            assert!(!key_position_text(id).is_empty(), "position for {id:?}");
            names.push(name);
        }
        let total = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), total, "two keys must not share a display name");
    }

    #[test]
    fn the_row_tables_agree_with_the_twenty_face_keys() {
        assert_eq!(GRID_ROW_LENGTHS.iter().sum::<usize>(), GRID_ROW_KEYS.len());
        assert!(
            !GRID_ROW_KEYS.contains(&KeyId::Kp20),
            "kp20 is the thumb key"
        );
        let face_keys = GRID_ROW_KEYS.len() + 1;
        assert_eq!(face_keys, 20, "the pad has twenty numbered keys");
    }
}
