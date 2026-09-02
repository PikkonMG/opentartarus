use crate::error::ErrorCode;
use crate::types::Profile;

pub const SHIPPED_IDS: [&str; 6] = [
    "default",
    "league-of-legends",
    "dota-2",
    "world-of-warcraft",
    "final-fantasy-xiv",
    "path-of-exile",
];

pub fn shipped_json(id: &str) -> Option<&'static str> {
    match id {
        "default" => Some(include_str!("../profiles/default.json")),
        "league-of-legends" => Some(include_str!("../profiles/league-of-legends.json")),
        "dota-2" => Some(include_str!("../profiles/dota-2.json")),
        "world-of-warcraft" => Some(include_str!("../profiles/world-of-warcraft.json")),
        "final-fantasy-xiv" => Some(include_str!("../profiles/final-fantasy-xiv.json")),
        "path-of-exile" => Some(include_str!("../profiles/path-of-exile.json")),
        _ => None,
    }
}

pub fn shipped_profile(id: &str) -> Result<Profile, ErrorCode> {
    let raw = shipped_json(id).ok_or(ErrorCode::NotFound)?;
    let p: Profile = serde_json::from_str(raw).map_err(|_| ErrorCode::InvalidProfile)?;
    p.validate()?;
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Action, GameId, KeyId, KeyToken, Profile};

    fn key_action(token: KeyToken) -> Action {
        Action::Key {
            key: token,
            modifiers: vec![],
        }
    }

    fn assert_key_bindings(profile: &Profile, expected: &[(KeyId, KeyToken)]) {
        assert_eq!(profile.bindings.len(), expected.len());
        for (key_id, token) in expected {
            assert_eq!(
                profile.bindings.get(key_id),
                Some(&key_action(*token)),
                "binding {key_id:?}"
            );
        }
    }

    const WOW_FFXIV_THUMBS: [(KeyId, KeyToken); 4] = [
        (KeyId::ThumbN, KeyToken::W),
        (KeyId::ThumbS, KeyToken::S),
        (KeyId::ThumbW, KeyToken::A),
        (KeyId::ThumbE, KeyToken::D),
    ];

    #[test]
    fn shipped_ids_are_pack_order() {
        assert_eq!(
            SHIPPED_IDS,
            [
                "default",
                "league-of-legends",
                "dota-2",
                "world-of-warcraft",
                "final-fantasy-xiv",
                "path-of-exile"
            ]
        );
    }

    #[test]
    fn every_shipped_profile_validates() {
        for id in SHIPPED_IDS {
            let p = shipped_profile(id).unwrap();
            p.validate().unwrap();
            assert_eq!(p.id, id);
        }
    }

    #[test]
    fn league_bindings_match_spec() {
        let p = shipped_profile("league-of-legends").unwrap();
        assert_eq!(p.game, GameId::LeagueOfLegends);
        assert_eq!(p.bindings.len(), 20);
        assert_eq!(p.lighting.color, Some([0, 180, 255]));
        assert_key_bindings(
            &p,
            &[
                (KeyId::Kp01, KeyToken::Q),
                (KeyId::Kp02, KeyToken::W),
                (KeyId::Kp03, KeyToken::E),
                (KeyId::Kp04, KeyToken::R),
                (KeyId::Kp05, KeyToken::D),
                (KeyId::Kp06, KeyToken::F),
                (KeyId::Kp07, KeyToken::Num1),
                (KeyId::Kp08, KeyToken::Num2),
                (KeyId::Kp09, KeyToken::Num3),
                (KeyId::Kp10, KeyToken::Num4),
                (KeyId::Kp11, KeyToken::Num5),
                (KeyId::Kp12, KeyToken::Num6),
                (KeyId::Kp13, KeyToken::B),
                (KeyId::Kp14, KeyToken::P),
                (KeyId::Kp15, KeyToken::Tab),
                (KeyId::Kp16, KeyToken::Space),
                (KeyId::Kp17, KeyToken::Y),
                (KeyId::Kp18, KeyToken::V),
                (KeyId::Kp19, KeyToken::A),
                (KeyId::Kp20, KeyToken::S),
            ],
        );
    }

    #[test]
    fn dota_bindings_match_spec() {
        let p = shipped_profile("dota-2").unwrap();
        assert_eq!(p.game, GameId::Dota2);
        assert_key_bindings(
            &p,
            &[
                (KeyId::Kp01, KeyToken::Q),
                (KeyId::Kp02, KeyToken::W),
                (KeyId::Kp03, KeyToken::E),
                (KeyId::Kp04, KeyToken::R),
                (KeyId::Kp05, KeyToken::D),
                (KeyId::Kp06, KeyToken::F),
                (KeyId::Kp07, KeyToken::Z),
                (KeyId::Kp08, KeyToken::X),
                (KeyId::Kp09, KeyToken::C),
                (KeyId::Kp10, KeyToken::Num1),
                (KeyId::Kp11, KeyToken::Num2),
                (KeyId::Kp12, KeyToken::Num3),
                (KeyId::Kp13, KeyToken::Num4),
                (KeyId::Kp14, KeyToken::Num5),
                (KeyId::Kp15, KeyToken::Num6),
                (KeyId::Kp16, KeyToken::T),
                (KeyId::Kp17, KeyToken::G),
                (KeyId::Kp18, KeyToken::Space),
                (KeyId::Kp19, KeyToken::Tab),
                (KeyId::Kp20, KeyToken::Grave),
            ],
        );
    }

    #[test]
    fn wow_bindings_match_spec() {
        let p = shipped_profile("world-of-warcraft").unwrap();
        assert_eq!(p.game, GameId::WorldOfWarcraft);
        let mut expected = vec![
            (KeyId::Kp01, KeyToken::Num1),
            (KeyId::Kp02, KeyToken::Num2),
            (KeyId::Kp03, KeyToken::Num3),
            (KeyId::Kp04, KeyToken::Num4),
            (KeyId::Kp05, KeyToken::Num5),
            (KeyId::Kp06, KeyToken::Num6),
            (KeyId::Kp07, KeyToken::Num7),
            (KeyId::Kp08, KeyToken::Num8),
            (KeyId::Kp09, KeyToken::Num9),
            (KeyId::Kp10, KeyToken::Num0),
            (KeyId::Kp11, KeyToken::F1),
            (KeyId::Kp12, KeyToken::F2),
            (KeyId::Kp13, KeyToken::F3),
            (KeyId::Kp14, KeyToken::F4),
            (KeyId::Kp15, KeyToken::F5),
            (KeyId::Kp16, KeyToken::F6),
            (KeyId::Kp17, KeyToken::F7),
            (KeyId::Kp18, KeyToken::F8),
            (KeyId::Kp19, KeyToken::F9),
            (KeyId::Kp20, KeyToken::F10),
        ];
        expected.extend(WOW_FFXIV_THUMBS);
        assert_key_bindings(&p, &expected);
    }

    #[test]
    fn ffxiv_bindings_match_spec() {
        let p = shipped_profile("final-fantasy-xiv").unwrap();
        assert_eq!(p.game, GameId::FinalFantasyXiv);
        let mut expected = vec![
            (KeyId::Kp01, KeyToken::Num1),
            (KeyId::Kp02, KeyToken::Num2),
            (KeyId::Kp03, KeyToken::Num3),
            (KeyId::Kp04, KeyToken::Num4),
            (KeyId::Kp05, KeyToken::Num5),
            (KeyId::Kp06, KeyToken::Num6),
            (KeyId::Kp07, KeyToken::Num7),
            (KeyId::Kp08, KeyToken::Num8),
            (KeyId::Kp09, KeyToken::Num9),
            (KeyId::Kp10, KeyToken::Num0),
            (KeyId::Kp11, KeyToken::Minus),
            (KeyId::Kp12, KeyToken::Equal),
            (KeyId::Kp13, KeyToken::E),
            (KeyId::Kp14, KeyToken::Q),
            (KeyId::Kp15, KeyToken::R),
            (KeyId::Kp16, KeyToken::F),
            (KeyId::Kp17, KeyToken::T),
            (KeyId::Kp18, KeyToken::G),
            (KeyId::Kp19, KeyToken::V),
            (KeyId::Kp20, KeyToken::C),
        ];
        expected.extend(WOW_FFXIV_THUMBS);
        assert_key_bindings(&p, &expected);
    }

    #[test]
    fn poe_bindings_match_spec() {
        let p = shipped_profile("path-of-exile").unwrap();
        assert_eq!(p.game, GameId::PathOfExile);
        assert_key_bindings(
            &p,
            &[
                (KeyId::Kp01, KeyToken::Num1),
                (KeyId::Kp02, KeyToken::Num2),
                (KeyId::Kp03, KeyToken::Num3),
                (KeyId::Kp04, KeyToken::Num4),
                (KeyId::Kp05, KeyToken::Num5),
                (KeyId::Kp06, KeyToken::Q),
                (KeyId::Kp07, KeyToken::W),
                (KeyId::Kp08, KeyToken::E),
                (KeyId::Kp09, KeyToken::R),
                (KeyId::Kp10, KeyToken::T),
                (KeyId::Kp11, KeyToken::A),
                (KeyId::Kp12, KeyToken::S),
                (KeyId::Kp13, KeyToken::D),
                (KeyId::Kp14, KeyToken::F),
                (KeyId::Kp15, KeyToken::G),
                (KeyId::Kp16, KeyToken::Space),
                (KeyId::Kp17, KeyToken::I),
                (KeyId::Kp18, KeyToken::C),
                (KeyId::Kp19, KeyToken::P),
                (KeyId::Kp20, KeyToken::Z),
            ],
        );
    }

    #[test]
    fn default_is_empty_passthrough() {
        let p = shipped_profile("default").unwrap();
        assert!(p.bindings.is_empty());
    }
}
