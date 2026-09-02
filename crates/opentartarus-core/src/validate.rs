use crate::constants::{
    HOLD_REPEAT_RATE_MAX_MS, HOLD_REPEAT_RATE_MIN_MS, MACRO_MAX_DELAY_MS, MACRO_MAX_STEPS,
};
use crate::error::ErrorCode;
use crate::types::{
    Action, DeviceModel, KeyId, Lighting, LightingEffect, MacroKind, MacroStep, Modifier, Profile,
};
use std::collections::BTreeMap;

const PROFILE_ID_MAX_LEN: usize = 64;
const PROFILE_NAME_MAX_LEN: usize = 64;
const BRIGHTNESS_MAX: u8 = 100;
const MODIFIERS_MAX: usize = 4;

impl Profile {
    pub fn validate(&self) -> Result<(), ErrorCode> {
        if self.unknown_key_ids {
            return Err(ErrorCode::InvalidProfile);
        }
        if !valid_id(&self.id) {
            return Err(ErrorCode::InvalidProfile);
        }
        if !valid_name(&self.name) {
            return Err(ErrorCode::InvalidProfile);
        }
        validate_device_models(&self.device_models)?;
        validate_lighting(&self.lighting)?;
        for action in self.bindings.values() {
            validate_action(action)?;
        }
        Ok(())
    }

    pub fn bindings_for_model(&self, model: DeviceModel) -> BTreeMap<KeyId, Action> {
        self.bindings
            .iter()
            .filter(|(key, _)| match model {
                DeviceModel::V2 => !key.is_analog(),
                DeviceModel::Pro => !key.is_thumb(),
            })
            .map(|(key, action)| (*key, action.clone()))
            .collect()
    }
}

fn valid_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    if bytes.is_empty() || bytes.len() > PROFILE_ID_MAX_LEN {
        return false;
    }
    let first = bytes[0];
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }
    bytes[1..]
        .iter()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
}

fn valid_name(name: &str) -> bool {
    let count = name.trim().chars().count();
    (1..=PROFILE_NAME_MAX_LEN).contains(&count)
}

fn validate_device_models(models: &[DeviceModel]) -> Result<(), ErrorCode> {
    if models.is_empty() {
        return Err(ErrorCode::InvalidProfile);
    }
    let mut seen = Vec::with_capacity(models.len());
    for model in models {
        if seen.contains(model) {
            return Err(ErrorCode::InvalidProfile);
        }
        seen.push(*model);
    }
    Ok(())
}

fn validate_lighting(lighting: &Lighting) -> Result<(), ErrorCode> {
    if lighting.brightness > BRIGHTNESS_MAX {
        return Err(ErrorCode::InvalidProfile);
    }
    if lighting.effect == LightingEffect::Static && lighting.color.is_none() {
        return Err(ErrorCode::InvalidProfile);
    }
    Ok(())
}

fn validate_modifiers(modifiers: &[Modifier]) -> Result<(), ErrorCode> {
    if modifiers.len() > MODIFIERS_MAX {
        return Err(ErrorCode::InvalidProfile);
    }
    let mut seen = Vec::with_capacity(modifiers.len());
    for modifier in modifiers {
        if seen.contains(modifier) {
            return Err(ErrorCode::InvalidProfile);
        }
        seen.push(*modifier);
    }
    Ok(())
}

fn validate_macro(steps: &[MacroStep]) -> Result<(), ErrorCode> {
    if steps.is_empty() || steps.len() > MACRO_MAX_STEPS {
        return Err(ErrorCode::InvalidProfile);
    }
    for step in steps {
        if step.delay_ms > MACRO_MAX_DELAY_MS {
            return Err(ErrorCode::InvalidProfile);
        }
        match &step.kind {
            MacroKind::Key { modifiers, .. } => validate_modifiers(modifiers)?,
            MacroKind::Mouse { .. } => {}
        }
    }
    Ok(())
}

fn validate_action(action: &Action) -> Result<(), ErrorCode> {
    match action {
        Action::Key { modifiers, .. } => validate_modifiers(modifiers),
        Action::Macro { steps } => validate_macro(steps),
        Action::Mouse { .. } => Ok(()),
        Action::HoldRepeat { inner, rate_ms } => {
            if *rate_ms < HOLD_REPEAT_RATE_MIN_MS || *rate_ms > HOLD_REPEAT_RATE_MAX_MS {
                return Err(ErrorCode::InvalidProfile);
            }
            match inner.as_ref() {
                Action::Key { modifiers, .. } => validate_modifiers(modifiers),
                _ => Err(ErrorCode::InvalidProfile),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lol_json() -> &'static str {
        r#"{
          "id": "league-of-legends",
          "name": "League of Legends",
          "game": "league-of-legends",
          "device_models": ["v2", "pro"],
          "bindings": {
            "kp01": { "type": "key", "key": "q", "modifiers": [] }
          },
          "lighting": { "effect": "static", "brightness": 80, "color": [0, 180, 255] }
        }"#
    }

    #[test]
    fn accepts_league_of_legends_json() {
        let p: Profile = serde_json::from_str(lol_json()).unwrap();
        p.validate().unwrap();
        assert_eq!(p.id, "league-of-legends");
    }

    #[test]
    fn rejects_unknown_key_id() {
        let raw = lol_json().replace("kp01", "kp99");
        let p: Profile = serde_json::from_str(&raw).unwrap();
        assert_eq!(p.validate().unwrap_err(), ErrorCode::InvalidProfile);
    }

    #[test]
    fn rejects_bad_id_empty_name_brightness_and_hold_repeat_macro() {
        let mut p: Profile = serde_json::from_str(lol_json()).unwrap();
        p.id = "Nope".into();
        assert_eq!(p.validate().unwrap_err(), ErrorCode::InvalidProfile);
        p.id = "ok".into();
        p.name = "   ".into();
        assert_eq!(p.validate().unwrap_err(), ErrorCode::InvalidProfile);
        p.name = "Ok".into();
        p.lighting.brightness = 101;
        assert_eq!(p.validate().unwrap_err(), ErrorCode::InvalidProfile);
        p.lighting.brightness = 80;
        p.bindings.insert(
            KeyId::Kp01,
            Action::HoldRepeat {
                inner: Box::new(Action::Macro { steps: vec![] }),
                rate_ms: 40,
            },
        );
        assert_eq!(p.validate().unwrap_err(), ErrorCode::InvalidProfile);
    }

    #[test]
    fn v2_drops_analog_keeps_thumb_pro_drops_thumb_keeps_analog() {
        let mut p: Profile = serde_json::from_str(lol_json()).unwrap();
        p.bindings.insert(
            KeyId::AnalogUp,
            Action::Key {
                key: crate::types::KeyToken::W,
                modifiers: vec![],
            },
        );
        p.bindings.insert(
            KeyId::ThumbN,
            Action::Key {
                key: crate::types::KeyToken::W,
                modifiers: vec![],
            },
        );
        let v2 = p.bindings_for_model(DeviceModel::V2);
        assert!(v2.contains_key(&KeyId::ThumbN));
        assert!(!v2.contains_key(&KeyId::AnalogUp));
        let pro = p.bindings_for_model(DeviceModel::Pro);
        assert!(pro.contains_key(&KeyId::AnalogUp));
        assert!(!pro.contains_key(&KeyId::ThumbN));
    }

    #[test]
    fn error_sentences_match_spec() {
        assert_eq!(
            ErrorCode::Permission.user_message(),
            "OpenTartarus can’t talk to your keypad yet."
        );
        assert_eq!(
            ErrorCode::Disconnect.user_message(),
            "Tartarus disconnected."
        );
        assert_eq!(
            ErrorCode::Lighting.user_message(),
            "Lighting needs OpenRazer."
        );
        assert_eq!(
            ErrorCode::GrabConflict.user_message(),
            "Something else is using the Tartarus. Quit that app and reopen OpenTartarus."
        );
        assert_eq!(
            ErrorCode::InvalidProfile.user_message(),
            "That profile file is damaged. Revert to shipped or pick another."
        );
        assert_eq!(
            ErrorCode::UnknownKey.user_message(),
            "That key isn’t on this keypad."
        );
        assert_eq!(
            ErrorCode::RecordBusy.user_message(),
            "Already recording. Press a key or click Cancel."
        );
        assert_eq!(
            ErrorCode::NotFound.user_message(),
            "That profile isn’t installed."
        );
        assert_eq!(
            ErrorCode::Io.user_message(),
            "OpenTartarus couldn’t save. Try again."
        );
    }
}
