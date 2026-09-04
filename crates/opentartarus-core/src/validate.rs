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

/// Joins the words of a slug. Also what a name's runs of punctuation and
/// whitespace collapse to, so `My  Raid / Layout` becomes `my-raid-layout`.
const SLUG_SEPARATOR: char = '-';
/// What a name that has no usable characters at all becomes, so a profile
/// named `!!!` still gets a valid id rather than an empty one.
const SLUG_FALLBACK: &str = "profile";

/// Turns a display name into a profile id that `valid_id` accepts and that
/// none of `taken` already uses.
///
/// Lowercase ASCII letters and digits pass through; everything else collapses
/// to one separator. A leading separator is dropped because an id must start
/// with a letter or digit. A collision gets a numeric suffix: `layout`,
/// `layout-2`, `layout-3`. The result is trimmed to the id length limit before
/// the suffix is added, so the suffix is never what pushes it over.
pub fn profile_id_for_name(name: &str, taken: &[String]) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut pending_separator = false;
    for ch in name.chars() {
        let lowered = ch.to_ascii_lowercase();
        if lowered.is_ascii_lowercase() || lowered.is_ascii_digit() {
            if pending_separator && !slug.is_empty() {
                slug.push(SLUG_SEPARATOR);
            }
            pending_separator = false;
            slug.push(lowered);
        } else {
            pending_separator = true;
        }
    }
    if slug.is_empty() {
        slug.push_str(SLUG_FALLBACK);
    }

    // Leave room for the widest suffix this loop could ever need before the
    // limit, so a long name plus `-99` still fits.
    let suffix_room = SLUG_SEPARATOR.len_utf8() + SLUG_MAX_SUFFIX_DIGITS;
    let base_max = PROFILE_ID_MAX_LEN.saturating_sub(suffix_room);
    let base: String = slug.chars().take(base_max).collect();
    let base = base.trim_end_matches(SLUG_SEPARATOR).to_owned();

    if !taken.contains(&base) {
        return base;
    }
    let mut counter: u32 = SLUG_FIRST_SUFFIX;
    loop {
        let candidate = format!("{base}{SLUG_SEPARATOR}{counter}");
        if !taken.contains(&candidate) {
            return candidate;
        }
        counter += 1;
    }
}

/// The second copy of a name is `-2`, never `-1`: the first has no suffix.
const SLUG_FIRST_SUFFIX: u32 = 2;
/// Digits reserved for the collision suffix when trimming the base.
const SLUG_MAX_SUFFIX_DIGITS: usize = 2;

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

    fn taken(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|id| (*id).to_owned()).collect()
    }

    #[test]
    fn a_plain_name_slugs_to_lowercase_hyphens() {
        assert_eq!(profile_id_for_name("My Raid Layout", &[]), "my-raid-layout");
    }

    #[test]
    fn punctuation_and_repeated_spaces_collapse_to_one_separator() {
        assert_eq!(
            profile_id_for_name("  My  Raid / Layout!! ", &[]),
            "my-raid-layout"
        );
    }

    #[test]
    fn a_leading_symbol_does_not_produce_a_leading_separator() {
        // An id must start with a letter or digit.
        assert_eq!(profile_id_for_name("#1 Setup", &[]), "1-setup");
    }

    #[test]
    fn a_name_with_nothing_usable_still_gets_a_valid_id() {
        assert_eq!(profile_id_for_name("!!!", &[]), SLUG_FALLBACK);
        assert!(valid_id(&profile_id_for_name("!!!", &[])));
    }

    #[test]
    fn a_collision_gets_a_numbered_suffix_starting_at_two() {
        assert_eq!(
            profile_id_for_name("Layout", &taken(&["layout"])),
            "layout-2"
        );
        assert_eq!(
            profile_id_for_name("Layout", &taken(&["layout", "layout-2"])),
            "layout-3"
        );
    }

    #[test]
    fn a_suffix_never_pushes_an_id_over_the_length_limit() {
        let long = "x".repeat(PROFILE_ID_MAX_LEN * 2);
        let first = profile_id_for_name(&long, &[]);
        assert!(valid_id(&first), "trimmed base must be a valid id");
        let second = profile_id_for_name(&long, std::slice::from_ref(&first));
        assert!(valid_id(&second), "suffixed id must still be valid: {second}");
        assert!(second.ends_with("-2"));
        assert!(second.len() <= PROFILE_ID_MAX_LEN);
    }

    #[test]
    fn every_generated_id_passes_the_same_rule_profiles_are_validated_by() {
        for name in ["Default", "Über Setup", "123", "a-b-c", "Trailing-", "-Leading"] {
            let id = profile_id_for_name(name, &[]);
            assert!(valid_id(&id), "{name:?} produced invalid id {id:?}");
        }
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
            "Lighting needs OpenRazer or OpenRGB.",
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
