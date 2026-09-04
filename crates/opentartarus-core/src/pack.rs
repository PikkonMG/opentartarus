use crate::error::ErrorCode;
use crate::types::Profile;

pub const SHIPPED_IDS: [&str; 16] = [
    "default",
    "league-of-legends",
    "dota-2",
    "world-of-warcraft",
    "final-fantasy-xiv",
    "path-of-exile",
    "overwatch-2",
    "valorant",
    "counter-strike-2",
    "apex-legends",
    "fortnite",
    "diablo-4",
    "elden-ring",
    "guild-wars-2",
    "elder-scrolls-online",
    "minecraft",
];

pub fn shipped_json(id: &str) -> Option<&'static str> {
    match id {
        "default" => Some(include_str!("../profiles/default.json")),
        "league-of-legends" => Some(include_str!("../profiles/league-of-legends.json")),
        "dota-2" => Some(include_str!("../profiles/dota-2.json")),
        "world-of-warcraft" => Some(include_str!("../profiles/world-of-warcraft.json")),
        "final-fantasy-xiv" => Some(include_str!("../profiles/final-fantasy-xiv.json")),
        "path-of-exile" => Some(include_str!("../profiles/path-of-exile.json")),
        "overwatch-2" => Some(include_str!("../profiles/overwatch-2.json")),
        "valorant" => Some(include_str!("../profiles/valorant.json")),
        "counter-strike-2" => Some(include_str!("../profiles/counter-strike-2.json")),
        "apex-legends" => Some(include_str!("../profiles/apex-legends.json")),
        "fortnite" => Some(include_str!("../profiles/fortnite.json")),
        "diablo-4" => Some(include_str!("../profiles/diablo-4.json")),
        "elden-ring" => Some(include_str!("../profiles/elden-ring.json")),
        "guild-wars-2" => Some(include_str!("../profiles/guild-wars-2.json")),
        "elder-scrolls-online" => Some(include_str!("../profiles/elder-scrolls-online.json")),
        "minecraft" => Some(include_str!("../profiles/minecraft.json")),
        _ => None,
    }
}

/// Whether an id names a profile from the shipped pack. Shipped profiles can
/// be reverted but never deleted; custom ones are the reverse.
pub fn is_shipped(id: &str) -> bool {
    SHIPPED_IDS.contains(&id)
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
    use crate::types::{Action, DeviceModel, GameId, KeyId, KeyToken, Profile};

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

    const WASD_THUMBS: [(KeyId, KeyToken); 4] = [
        (KeyId::ThumbN, KeyToken::W),
        (KeyId::ThumbW, KeyToken::A),
        (KeyId::ThumbS, KeyToken::S),
        (KeyId::ThumbE, KeyToken::D),
    ];

    const NEW_GAME_IDS: [&str; 10] = [
        "overwatch-2",
        "valorant",
        "counter-strike-2",
        "apex-legends",
        "fortnite",
        "diablo-4",
        "elden-ring",
        "guild-wars-2",
        "elder-scrolls-online",
        "minecraft",
    ];

    #[test]
    fn is_shipped_knows_the_pack_and_nothing_else() {
        for id in SHIPPED_IDS {
            assert!(is_shipped(id), "{id} ships with the app");
        }
        assert!(!is_shipped("my-raid-layout"));
        assert!(!is_shipped(""));
        // Case matters: ids are lowercase slugs.
        assert!(!is_shipped("Default"));
    }

    #[test]
    fn a_custom_game_round_trips_through_json_as_custom() {
        let json = serde_json::to_string(&GameId::Custom).unwrap();
        assert_eq!(json, "\"custom\"");
        let back: GameId = serde_json::from_str("\"custom\"").unwrap();
        assert_eq!(back, GameId::Custom);
    }

    #[test]
    fn shipped_ids_are_pack_order() {
        assert_eq!(SHIPPED_IDS.len(), 16);
        assert_eq!(SHIPPED_IDS[0], "default");
        assert_eq!(
            &SHIPPED_IDS[1..6],
            &[
                "league-of-legends",
                "dota-2",
                "world-of-warcraft",
                "final-fantasy-xiv",
                "path-of-exile"
            ]
        );
        assert_eq!(&SHIPPED_IDS[6..], &NEW_GAME_IDS);
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

    #[test]
    fn overwatch_bindings_match_spec() {
        let p = shipped_profile("overwatch-2").unwrap();
        assert_eq!(p.game, GameId::Overwatch2);
        assert_eq!(p.lighting.color, Some([255, 107, 53]));
        let mut expected = vec![
            (KeyId::Kp01, KeyToken::Q),
            (KeyId::Kp02, KeyToken::E),
            (KeyId::Kp03, KeyToken::F),
            (KeyId::Kp04, KeyToken::R),
            (KeyId::Kp05, KeyToken::V),
            (KeyId::Kp06, KeyToken::Num1),
            (KeyId::Kp07, KeyToken::Num2),
            (KeyId::Kp08, KeyToken::Num3),
            (KeyId::Kp09, KeyToken::Num4),
            (KeyId::Kp10, KeyToken::Num5),
            (KeyId::Kp11, KeyToken::C),
            (KeyId::Kp12, KeyToken::Z),
            (KeyId::Kp13, KeyToken::H),
            (KeyId::Kp14, KeyToken::Tab),
            (KeyId::Kp15, KeyToken::O),
            (KeyId::Kp16, KeyToken::Enter),
            (KeyId::Kp17, KeyToken::Escape),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
        assert!(!p.bindings.contains_key(&KeyId::Kp18), "kp18 stays free");
        assert!(!p.bindings.contains_key(&KeyId::Kp19), "kp19 stays free");
    }

    #[test]
    fn valorant_bindings_match_spec() {
        let p = shipped_profile("valorant").unwrap();
        assert_eq!(p.game, GameId::Valorant);
        assert_eq!(p.lighting.color, Some([255, 70, 85]));
        let mut expected = vec![
            (KeyId::Kp01, KeyToken::Q),
            (KeyId::Kp02, KeyToken::E),
            (KeyId::Kp03, KeyToken::C),
            (KeyId::Kp04, KeyToken::X),
            (KeyId::Kp05, KeyToken::R),
            (KeyId::Kp06, KeyToken::Num1),
            (KeyId::Kp07, KeyToken::Num2),
            (KeyId::Kp08, KeyToken::Num3),
            (KeyId::Kp09, KeyToken::Num4),
            (KeyId::Kp10, KeyToken::Num5),
            (KeyId::Kp11, KeyToken::G),
            (KeyId::Kp12, KeyToken::B),
            (KeyId::Kp13, KeyToken::F),
            (KeyId::Kp14, KeyToken::Tab),
            (KeyId::Kp15, KeyToken::M),
            (KeyId::Kp16, KeyToken::Enter),
            (KeyId::Kp17, KeyToken::Escape),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
        assert!(!p.bindings.contains_key(&KeyId::Kp18), "kp18 stays free");
        assert!(!p.bindings.contains_key(&KeyId::Kp19), "kp19 stays free");
    }

    #[test]
    fn counter_strike_bindings_match_spec() {
        let p = shipped_profile("counter-strike-2").unwrap();
        assert_eq!(p.game, GameId::CounterStrike2);
        assert_eq!(p.lighting.color, Some([222, 155, 53]));
        let mut expected = vec![
            (KeyId::Kp01, KeyToken::Q),
            (KeyId::Kp02, KeyToken::E),
            (KeyId::Kp03, KeyToken::G),
            (KeyId::Kp04, KeyToken::R),
            (KeyId::Kp05, KeyToken::F),
            (KeyId::Kp06, KeyToken::Num1),
            (KeyId::Kp07, KeyToken::Num2),
            (KeyId::Kp08, KeyToken::Num3),
            (KeyId::Kp09, KeyToken::Num4),
            (KeyId::Kp10, KeyToken::Num5),
            (KeyId::Kp11, KeyToken::B),
            (KeyId::Kp12, KeyToken::Z),
            (KeyId::Kp13, KeyToken::X),
            (KeyId::Kp14, KeyToken::C),
            (KeyId::Kp15, KeyToken::Tab),
            (KeyId::Kp16, KeyToken::Enter),
            (KeyId::Kp17, KeyToken::Escape),
            (KeyId::Kp18, KeyToken::M),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
        assert!(!p.bindings.contains_key(&KeyId::Kp19), "kp19 stays free");
    }

    #[test]
    fn apex_bindings_match_spec() {
        let p = shipped_profile("apex-legends").unwrap();
        assert_eq!(p.game, GameId::ApexLegends);
        assert_eq!(p.lighting.color, Some([218, 41, 46]));
        let mut expected = vec![
            (KeyId::Kp01, KeyToken::Q),
            (KeyId::Kp02, KeyToken::Z),
            (KeyId::Kp03, KeyToken::E),
            (KeyId::Kp04, KeyToken::R),
            (KeyId::Kp05, KeyToken::V),
            (KeyId::Kp06, KeyToken::Num1),
            (KeyId::Kp07, KeyToken::Num2),
            (KeyId::Kp08, KeyToken::Num3),
            (KeyId::Kp09, KeyToken::Num4),
            (KeyId::Kp10, KeyToken::Num5),
            (KeyId::Kp11, KeyToken::Tab),
            (KeyId::Kp12, KeyToken::M),
            (KeyId::Kp13, KeyToken::C),
            (KeyId::Kp14, KeyToken::X),
            (KeyId::Kp15, KeyToken::G),
            (KeyId::Kp16, KeyToken::Enter),
            (KeyId::Kp17, KeyToken::Escape),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
        assert!(!p.bindings.contains_key(&KeyId::Kp18), "kp18 stays free");
        assert!(!p.bindings.contains_key(&KeyId::Kp19), "kp19 stays free");
    }

    #[test]
    fn fortnite_bindings_match_spec() {
        let p = shipped_profile("fortnite").unwrap();
        assert_eq!(p.game, GameId::Fortnite);
        assert_eq!(p.lighting.color, Some([70, 145, 245]));
        let mut expected = vec![
            (KeyId::Kp01, KeyToken::Z),
            (KeyId::Kp02, KeyToken::X),
            (KeyId::Kp03, KeyToken::C),
            (KeyId::Kp04, KeyToken::V),
            (KeyId::Kp05, KeyToken::G),
            (KeyId::Kp06, KeyToken::Num1),
            (KeyId::Kp07, KeyToken::Num2),
            (KeyId::Kp08, KeyToken::Num3),
            (KeyId::Kp09, KeyToken::Num4),
            (KeyId::Kp10, KeyToken::Num5),
            (KeyId::Kp11, KeyToken::F),
            (KeyId::Kp12, KeyToken::R),
            (KeyId::Kp13, KeyToken::E),
            (KeyId::Kp14, KeyToken::Tab),
            (KeyId::Kp15, KeyToken::M),
            (KeyId::Kp16, KeyToken::Enter),
            (KeyId::Kp17, KeyToken::Escape),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
        assert!(!p.bindings.contains_key(&KeyId::Kp18), "kp18 stays free");
        assert!(!p.bindings.contains_key(&KeyId::Kp19), "kp19 stays free");
    }

    #[test]
    fn diablo_bindings_match_spec() {
        let p = shipped_profile("diablo-4").unwrap();
        assert_eq!(p.game, GameId::Diablo4);
        assert_eq!(p.lighting.color, Some([164, 32, 26]));
        let mut expected = vec![
            (KeyId::Kp01, KeyToken::Num1),
            (KeyId::Kp02, KeyToken::Num2),
            (KeyId::Kp03, KeyToken::Num3),
            (KeyId::Kp04, KeyToken::Num4),
            (KeyId::Kp05, KeyToken::Q),
            (KeyId::Kp06, KeyToken::T),
            (KeyId::Kp07, KeyToken::I),
            (KeyId::Kp08, KeyToken::C),
            (KeyId::Kp09, KeyToken::M),
            (KeyId::Kp10, KeyToken::Tab),
            (KeyId::Kp11, KeyToken::Z),
            (KeyId::Kp12, KeyToken::J),
            (KeyId::Kp13, KeyToken::S),
            (KeyId::Kp14, KeyToken::O),
            (KeyId::Kp15, KeyToken::P),
            (KeyId::Kp16, KeyToken::Enter),
            (KeyId::Kp17, KeyToken::Escape),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
        assert!(!p.bindings.contains_key(&KeyId::Kp18), "kp18 stays free");
        assert!(!p.bindings.contains_key(&KeyId::Kp19), "kp19 stays free");
    }

    #[test]
    fn elden_ring_bindings_match_spec() {
        let p = shipped_profile("elden-ring").unwrap();
        assert_eq!(p.game, GameId::EldenRing);
        assert_eq!(p.lighting.color, Some([201, 168, 96]));
        let mut expected = vec![
            (KeyId::Kp01, KeyToken::E),
            (KeyId::Kp02, KeyToken::R),
            (KeyId::Kp03, KeyToken::Q),
            (KeyId::Kp04, KeyToken::F),
            (KeyId::Kp05, KeyToken::G),
            (KeyId::Kp06, KeyToken::Num1),
            (KeyId::Kp07, KeyToken::Num2),
            (KeyId::Kp08, KeyToken::Num3),
            (KeyId::Kp09, KeyToken::Num4),
            (KeyId::Kp10, KeyToken::Num5),
            (KeyId::Kp11, KeyToken::I),
            (KeyId::Kp12, KeyToken::M),
            (KeyId::Kp13, KeyToken::Tab),
            (KeyId::Kp14, KeyToken::H),
            (KeyId::Kp15, KeyToken::X),
            (KeyId::Kp16, KeyToken::Enter),
            (KeyId::Kp17, KeyToken::Escape),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
        assert!(!p.bindings.contains_key(&KeyId::Kp18), "kp18 stays free");
        assert!(!p.bindings.contains_key(&KeyId::Kp19), "kp19 stays free");
    }

    #[test]
    fn guild_wars_bindings_match_spec() {
        let p = shipped_profile("guild-wars-2").unwrap();
        assert_eq!(p.game, GameId::GuildWars2);
        assert_eq!(p.lighting.color, Some([190, 30, 45]));
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
            (KeyId::Kp15, KeyToken::V),
            (KeyId::Kp16, KeyToken::I),
            (KeyId::Kp17, KeyToken::M),
            (KeyId::Kp18, KeyToken::H),
            (KeyId::Kp19, KeyToken::Escape),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
    }

    #[test]
    fn eso_bindings_match_spec() {
        let p = shipped_profile("elder-scrolls-online").unwrap();
        assert_eq!(p.game, GameId::ElderScrollsOnline);
        assert_eq!(p.lighting.color, Some([45, 130, 90]));
        let mut expected = vec![
            (KeyId::Kp01, KeyToken::Num1),
            (KeyId::Kp02, KeyToken::Num2),
            (KeyId::Kp03, KeyToken::Num3),
            (KeyId::Kp04, KeyToken::Num4),
            (KeyId::Kp05, KeyToken::Num5),
            (KeyId::Kp06, KeyToken::R),
            (KeyId::Kp07, KeyToken::Q),
            (KeyId::Kp08, KeyToken::E),
            (KeyId::Kp09, KeyToken::F),
            (KeyId::Kp10, KeyToken::G),
            (KeyId::Kp11, KeyToken::I),
            (KeyId::Kp12, KeyToken::M),
            (KeyId::Kp13, KeyToken::J),
            (KeyId::Kp14, KeyToken::K),
            (KeyId::Kp15, KeyToken::C),
            (KeyId::Kp16, KeyToken::Enter),
            (KeyId::Kp17, KeyToken::Escape),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
        assert!(!p.bindings.contains_key(&KeyId::Kp18), "kp18 stays free");
        assert!(!p.bindings.contains_key(&KeyId::Kp19), "kp19 stays free");
    }

    #[test]
    fn minecraft_bindings_match_spec() {
        let p = shipped_profile("minecraft").unwrap();
        assert_eq!(p.game, GameId::Minecraft);
        assert_eq!(p.lighting.color, Some([94, 168, 64]));
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
            (KeyId::Kp10, KeyToken::E),
            (KeyId::Kp11, KeyToken::Q),
            (KeyId::Kp12, KeyToken::F),
            (KeyId::Kp13, KeyToken::T),
            (KeyId::Kp14, KeyToken::Tab),
            (KeyId::Kp15, KeyToken::Escape),
            (KeyId::Kp16, KeyToken::F3),
            (KeyId::Kp17, KeyToken::F5),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
        assert!(!p.bindings.contains_key(&KeyId::Kp18), "kp18 stays free");
        assert!(!p.bindings.contains_key(&KeyId::Kp19), "kp19 stays free");
    }

    #[test]
    fn every_new_profile_puts_movement_on_the_thumb_pad() {
        for id in NEW_GAME_IDS {
            let p = shipped_profile(id).unwrap();
            for (key_id, token) in WASD_THUMBS {
                assert_eq!(
                    p.bindings.get(&key_id),
                    Some(&Action::Key {
                        key: token,
                        modifiers: vec![]
                    }),
                    "{id} is missing {key_id:?}"
                );
            }
        }
    }

    #[test]
    fn no_new_profile_binds_a_diagonal_or_the_analog_stick() {
        for id in NEW_GAME_IDS {
            let p = shipped_profile(id).unwrap();
            for key_id in [
                KeyId::ThumbNe,
                KeyId::ThumbSe,
                KeyId::ThumbSw,
                KeyId::ThumbNw,
                KeyId::AnalogUp,
                KeyId::AnalogDown,
                KeyId::AnalogLeft,
                KeyId::AnalogRight,
            ] {
                assert!(
                    !p.bindings.contains_key(&key_id),
                    "{id} must leave {key_id:?} free"
                );
            }
        }
    }

    #[test]
    fn every_new_binding_is_a_plain_key_on_both_models() {
        for id in NEW_GAME_IDS {
            let p = shipped_profile(id).unwrap();
            assert_eq!(
                p.device_models,
                vec![DeviceModel::V2, DeviceModel::Pro],
                "{id} must support both pads"
            );
            assert!(p.lighting.color.is_some(), "{id} needs a sidebar colour");
            for (key_id, action) in &p.bindings {
                match action {
                    Action::Key { modifiers, .. } => assert!(
                        modifiers.is_empty(),
                        "{id} {key_id:?} must not carry a modifier"
                    ),
                    other => panic!("{id} {key_id:?} must be a plain key, found {other:?}"),
                }
            }
        }
    }
}
