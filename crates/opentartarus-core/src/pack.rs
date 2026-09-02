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
    use crate::types::GameId;

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
    }

    #[test]
    fn default_is_empty_passthrough() {
        let p = shipped_profile("default").unwrap();
        assert!(p.bindings.is_empty());
    }
}
