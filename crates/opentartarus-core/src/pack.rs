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

    // Razer's own layout: the left of a keyboard, arrows on the thumb pad.
    // Every game's defaults work with it and nothing needs setting up.
    #[test]
    fn default_layout_bindings_match_spec() {
        let p = shipped_profile("default").unwrap();
        assert_eq!(p.game, GameId::Default);
        assert_key_bindings(
            &p,
            &[
            (KeyId::Kp01, KeyToken::Num1),
            (KeyId::Kp02, KeyToken::Num2),
            (KeyId::Kp03, KeyToken::Num3),
            (KeyId::Kp04, KeyToken::Num4),
            (KeyId::Kp05, KeyToken::Num5),
            (KeyId::Kp06, KeyToken::Tab),
            (KeyId::Kp07, KeyToken::Q),
            (KeyId::Kp08, KeyToken::W),
            (KeyId::Kp09, KeyToken::E),
            (KeyId::Kp10, KeyToken::R),
            (KeyId::Kp11, KeyToken::CapsLock),
            (KeyId::Kp12, KeyToken::A),
            (KeyId::Kp13, KeyToken::S),
            (KeyId::Kp14, KeyToken::D),
            (KeyId::Kp15, KeyToken::F),
            (KeyId::Kp16, KeyToken::LeftShift),
            (KeyId::Kp17, KeyToken::Z),
            (KeyId::Kp18, KeyToken::X),
            (KeyId::Kp19, KeyToken::C),
            (KeyId::Kp20, KeyToken::Space),
            (KeyId::ThumbN, KeyToken::Up),
            (KeyId::ThumbW, KeyToken::Left),
            (KeyId::ThumbS, KeyToken::Down),
            (KeyId::ThumbE, KeyToken::Right),
            ],
        );
    }

    #[test]
    fn league_bindings_match_spec() {
        let p = shipped_profile("league-of-legends").unwrap();
        assert_eq!(p.game, GameId::LeagueOfLegends);
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
            (KeyId::Kp18, KeyToken::Num7),
            (KeyId::Kp19, KeyToken::A),
            (KeyId::Kp20, KeyToken::S),
            ],
        );
    }

    #[test]
    fn dota_bindings_match_spec() {
        let p = shipped_profile("dota-2").unwrap();
        assert_eq!(p.game, GameId::Dota2);
        assert_eq!(p.lighting.color, Some([200, 40, 40]));
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
            (KeyId::Kp10, KeyToken::V),
            (KeyId::Kp11, KeyToken::B),
            (KeyId::Kp12, KeyToken::N),
            (KeyId::Kp13, KeyToken::F4),
            (KeyId::Kp14, KeyToken::A),
            (KeyId::Kp15, KeyToken::S),
            (KeyId::Kp16, KeyToken::Grave),
            (KeyId::Kp17, KeyToken::F2),
            (KeyId::Kp18, KeyToken::F3),
            (KeyId::Kp19, KeyToken::Tab),
            (KeyId::Kp20, KeyToken::F1),
            ],
        );
    }

    #[test]
    fn wow_bindings_match_spec() {
        let p = shipped_profile("world-of-warcraft").unwrap();
        assert_eq!(p.game, GameId::WorldOfWarcraft);
        assert_eq!(p.lighting.color, Some([255, 180, 0]));
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
            (KeyId::Kp13, KeyToken::Tab),
            (KeyId::Kp14, KeyToken::F1),
            (KeyId::Kp15, KeyToken::F),
            (KeyId::Kp16, KeyToken::M),
            (KeyId::Kp17, KeyToken::B),
            (KeyId::Kp18, KeyToken::C),
            (KeyId::Kp19, KeyToken::Escape),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
    }

    #[test]
    fn ffxiv_bindings_match_spec() {
        let p = shipped_profile("final-fantasy-xiv").unwrap();
        assert_eq!(p.game, GameId::FinalFantasyXiv);
        assert_eq!(p.lighting.color, Some([80, 160, 255]));
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
            (KeyId::Kp13, KeyToken::Tab),
            (KeyId::Kp14, KeyToken::F),
            (KeyId::Kp15, KeyToken::R),
            (KeyId::Kp16, KeyToken::M),
            (KeyId::Kp17, KeyToken::I),
            (KeyId::Kp18, KeyToken::J),
            (KeyId::Kp19, KeyToken::C),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
    }

    #[test]
    fn poe_bindings_match_spec() {
        let p = shipped_profile("path-of-exile").unwrap();
        assert_eq!(p.game, GameId::PathOfExile);
        assert_eq!(p.lighting.color, Some([140, 0, 0]));
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
            (KeyId::Kp11, KeyToken::Tab),
            (KeyId::Kp12, KeyToken::X),
            (KeyId::Kp13, KeyToken::D),
            (KeyId::Kp14, KeyToken::Z),
            (KeyId::Kp15, KeyToken::G),
            (KeyId::Kp16, KeyToken::Enter),
            (KeyId::Kp17, KeyToken::I),
            (KeyId::Kp18, KeyToken::C),
            (KeyId::Kp19, KeyToken::P),
            (KeyId::Kp20, KeyToken::Space),
            ],
        );
    }

    #[test]
    fn overwatch_bindings_match_spec() {
        let p = shipped_profile("overwatch-2").unwrap();
        assert_eq!(p.game, GameId::Overwatch2);
        assert_eq!(p.lighting.color, Some([255, 107, 53]));
        let mut expected = vec![
            (KeyId::Kp01, KeyToken::Num1),
            (KeyId::Kp02, KeyToken::Num2),
            (KeyId::Kp03, KeyToken::H),
            (KeyId::Kp04, KeyToken::O),
            (KeyId::Kp05, KeyToken::Tab),
            (KeyId::Kp06, KeyToken::Q),
            (KeyId::Kp07, KeyToken::E),
            (KeyId::Kp08, KeyToken::X),
            (KeyId::Kp09, KeyToken::R),
            (KeyId::Kp10, KeyToken::V),
            (KeyId::Kp11, KeyToken::F),
            (KeyId::Kp12, KeyToken::LeftShift),
            (KeyId::Kp13, KeyToken::LeftCtrl),
            (KeyId::Kp14, KeyToken::C),
            (KeyId::Kp15, KeyToken::Z),
            (KeyId::Kp16, KeyToken::Enter),
            (KeyId::Kp17, KeyToken::Escape),
            (KeyId::Kp18, KeyToken::F1),
            (KeyId::Kp19, KeyToken::G),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
    }

    #[test]
    fn valorant_bindings_match_spec() {
        let p = shipped_profile("valorant").unwrap();
        assert_eq!(p.game, GameId::Valorant);
        assert_eq!(p.lighting.color, Some([255, 70, 85]));
        let mut expected = vec![
            (KeyId::Kp01, KeyToken::Num1),
            (KeyId::Kp02, KeyToken::Num2),
            (KeyId::Kp03, KeyToken::Num3),
            (KeyId::Kp04, KeyToken::Num4),
            (KeyId::Kp05, KeyToken::Tab),
            (KeyId::Kp06, KeyToken::Q),
            (KeyId::Kp07, KeyToken::E),
            (KeyId::Kp08, KeyToken::C),
            (KeyId::Kp09, KeyToken::X),
            (KeyId::Kp10, KeyToken::R),
            (KeyId::Kp11, KeyToken::F),
            (KeyId::Kp12, KeyToken::LeftCtrl),
            (KeyId::Kp13, KeyToken::LeftShift),
            (KeyId::Kp14, KeyToken::Z),
            (KeyId::Kp15, KeyToken::G),
            (KeyId::Kp16, KeyToken::B),
            (KeyId::Kp17, KeyToken::M),
            (KeyId::Kp18, KeyToken::Enter),
            (KeyId::Kp19, KeyToken::Escape),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
    }

    #[test]
    fn counter_strike_bindings_match_spec() {
        let p = shipped_profile("counter-strike-2").unwrap();
        assert_eq!(p.game, GameId::CounterStrike2);
        assert_eq!(p.lighting.color, Some([222, 155, 53]));
        let mut expected = vec![
            (KeyId::Kp01, KeyToken::B),
            (KeyId::Kp02, KeyToken::Tab),
            (KeyId::Kp03, KeyToken::Y),
            (KeyId::Kp04, KeyToken::Z),
            (KeyId::Kp05, KeyToken::F),
            (KeyId::Kp06, KeyToken::Num1),
            (KeyId::Kp07, KeyToken::Num2),
            (KeyId::Kp08, KeyToken::Num3),
            (KeyId::Kp09, KeyToken::Num4),
            (KeyId::Kp10, KeyToken::Num5),
            (KeyId::Kp11, KeyToken::Q),
            (KeyId::Kp12, KeyToken::E),
            (KeyId::Kp13, KeyToken::R),
            (KeyId::Kp14, KeyToken::G),
            (KeyId::Kp15, KeyToken::LeftCtrl),
            (KeyId::Kp16, KeyToken::LeftShift),
            (KeyId::Kp17, KeyToken::Escape),
            (KeyId::Kp18, KeyToken::M),
            (KeyId::Kp19, KeyToken::C),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
    }

    #[test]
    fn apex_bindings_match_spec() {
        let p = shipped_profile("apex-legends").unwrap();
        assert_eq!(p.game, GameId::ApexLegends);
        assert_eq!(p.lighting.color, Some([218, 41, 46]));
        let mut expected = vec![
            (KeyId::Kp01, KeyToken::Tab),
            (KeyId::Kp02, KeyToken::M),
            (KeyId::Kp03, KeyToken::Z),
            (KeyId::Kp04, KeyToken::V),
            (KeyId::Kp05, KeyToken::X),
            (KeyId::Kp06, KeyToken::Num1),
            (KeyId::Kp07, KeyToken::Num2),
            (KeyId::Kp08, KeyToken::Num3),
            (KeyId::Kp09, KeyToken::Num4),
            (KeyId::Kp10, KeyToken::Num5),
            (KeyId::Kp11, KeyToken::Q),
            (KeyId::Kp12, KeyToken::E),
            (KeyId::Kp13, KeyToken::R),
            (KeyId::Kp14, KeyToken::C),
            (KeyId::Kp15, KeyToken::G),
            (KeyId::Kp16, KeyToken::Enter),
            (KeyId::Kp17, KeyToken::Escape),
            (KeyId::Kp18, KeyToken::F),
            (KeyId::Kp19, KeyToken::LeftShift),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
    }

    #[test]
    fn fortnite_bindings_match_spec() {
        let p = shipped_profile("fortnite").unwrap();
        assert_eq!(p.game, GameId::Fortnite);
        assert_eq!(p.lighting.color, Some([70, 145, 245]));
        let mut expected = vec![
            (KeyId::Kp01, KeyToken::F),
            (KeyId::Kp02, KeyToken::R),
            (KeyId::Kp03, KeyToken::E),
            (KeyId::Kp04, KeyToken::Tab),
            (KeyId::Kp05, KeyToken::M),
            (KeyId::Kp06, KeyToken::Num1),
            (KeyId::Kp07, KeyToken::Num2),
            (KeyId::Kp08, KeyToken::Num3),
            (KeyId::Kp09, KeyToken::Num4),
            (KeyId::Kp10, KeyToken::Num5),
            (KeyId::Kp11, KeyToken::Z),
            (KeyId::Kp12, KeyToken::X),
            (KeyId::Kp13, KeyToken::C),
            (KeyId::Kp14, KeyToken::V),
            (KeyId::Kp15, KeyToken::G),
            (KeyId::Kp16, KeyToken::Enter),
            (KeyId::Kp17, KeyToken::Escape),
            (KeyId::Kp18, KeyToken::LeftShift),
            (KeyId::Kp19, KeyToken::LeftCtrl),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
    }

    #[test]
    fn diablo_bindings_match_spec() {
        let p = shipped_profile("diablo-4").unwrap();
        assert_eq!(p.game, GameId::Diablo4);
        assert_eq!(p.lighting.color, Some([164, 32, 26]));
        let mut expected = vec![
            (KeyId::Kp01, KeyToken::C),
            (KeyId::Kp02, KeyToken::J),
            (KeyId::Kp03, KeyToken::O),
            (KeyId::Kp04, KeyToken::P),
            (KeyId::Kp05, KeyToken::Y),
            (KeyId::Kp06, KeyToken::E),
            (KeyId::Kp07, KeyToken::F),
            (KeyId::Kp08, KeyToken::Z),
            (KeyId::Kp09, KeyToken::T),
            (KeyId::Kp10, KeyToken::Tab),
            (KeyId::Kp11, KeyToken::Num1),
            (KeyId::Kp12, KeyToken::Num2),
            (KeyId::Kp13, KeyToken::Num3),
            (KeyId::Kp14, KeyToken::Num4),
            (KeyId::Kp15, KeyToken::Q),
            (KeyId::Kp16, KeyToken::Enter),
            (KeyId::Kp17, KeyToken::Escape),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
    }

    #[test]
    fn elden_ring_bindings_match_spec() {
        let p = shipped_profile("elden-ring").unwrap();
        assert_eq!(p.game, GameId::EldenRing);
        assert_eq!(p.lighting.color, Some([201, 168, 96]));
        let mut expected = vec![
            (KeyId::Kp05, KeyToken::G),
            (KeyId::Kp06, KeyToken::Up),
            (KeyId::Kp07, KeyToken::Down),
            (KeyId::Kp08, KeyToken::Left),
            (KeyId::Kp09, KeyToken::Right),
            (KeyId::Kp11, KeyToken::E),
            (KeyId::Kp12, KeyToken::R),
            (KeyId::Kp13, KeyToken::Q),
            (KeyId::Kp14, KeyToken::F),
            (KeyId::Kp15, KeyToken::X),
            (KeyId::Kp17, KeyToken::Escape),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
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
            (KeyId::Kp16, KeyToken::Grave),
            (KeyId::Kp17, KeyToken::X),
            (KeyId::Kp18, KeyToken::F),
            (KeyId::Kp19, KeyToken::F5),
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
            (KeyId::Kp09, KeyToken::Grave),
            (KeyId::Kp10, KeyToken::H),
            (KeyId::Kp11, KeyToken::X),
            (KeyId::Kp12, KeyToken::LeftShift),
            (KeyId::Kp13, KeyToken::I),
            (KeyId::Kp14, KeyToken::M),
            (KeyId::Kp15, KeyToken::C),
            (KeyId::Kp16, KeyToken::Enter),
            (KeyId::Kp17, KeyToken::Escape),
            (KeyId::Kp18, KeyToken::K),
            (KeyId::Kp20, KeyToken::Space),
            (KeyId::Kp19, KeyToken::LeftCtrl),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
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
            (KeyId::Kp16, KeyToken::LeftShift),
            (KeyId::Kp17, KeyToken::LeftCtrl),
            (KeyId::Kp18, KeyToken::F3),
            (KeyId::Kp19, KeyToken::F5),
            (KeyId::Kp20, KeyToken::Space),
        ];
        expected.extend(WASD_THUMBS);
        assert_key_bindings(&p, &expected);
    }

    #[test]
    fn a_setup_note_exists_only_where_the_game_needs_an_in_game_step() {
        // Diablo needs a preset switched on; Elden Ring changed its layout
        // in a patch. Every other profile works with the game's defaults.
        let with_note = [
            "diablo-4",
            "elden-ring",
        ];
        for id in SHIPPED_IDS {
            let p = shipped_profile(id).unwrap();
            assert_eq!(
                p.setup_note.is_some(),
                with_note.contains(&id),
                "{id}: setup note presence"
            );
        }
        let noted = shipped_profile(with_note[0]).unwrap();
        let json = serde_json::to_value(&noted).unwrap();
        assert!(json.get("setup_note").is_some(), "a note survives a save");
        let plain = shipped_profile("league-of-legends").unwrap();
        let json = serde_json::to_value(&plain).unwrap();
        assert!(json.get("setup_note").is_none(), "no note, no field written");
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
