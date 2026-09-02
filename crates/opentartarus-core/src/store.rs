use crate::error::ErrorCode;
use crate::paths::Paths;
use crate::types::Profile;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppStateFile {
    pub active_profile_id: String,
}

fn profile_path(paths: &Paths, id: &str) -> PathBuf {
    paths.profiles_dir.join(format!("{id}.json"))
}

fn ensure_profile_dirs(paths: &Paths) -> Result<(), ErrorCode> {
    fs::create_dir_all(&paths.profiles_dir).map_err(|_| ErrorCode::Io)?;
    if let Some(parent) = paths.state_file.parent() {
        fs::create_dir_all(parent).map_err(|_| ErrorCode::Io)?;
    }
    Ok(())
}

fn parse_profile(bytes: &[u8]) -> Result<Profile, ErrorCode> {
    serde_json::from_slice(bytes).map_err(|_| ErrorCode::InvalidProfile)
}

pub fn write_profile(paths: &Paths, profile: &Profile) -> Result<(), ErrorCode> {
    ensure_profile_dirs(paths)?;
    let path = profile_path(paths, &profile.id);
    let json = serde_json::to_vec_pretty(profile).map_err(|_| ErrorCode::InvalidProfile)?;
    fs::write(path, json).map_err(|_| ErrorCode::Io)
}

pub fn read_user_profile(paths: &Paths, id: &str) -> Result<Profile, ErrorCode> {
    let path = profile_path(paths, id);
    let bytes = fs::read(path).map_err(|_| ErrorCode::Io)?;
    parse_profile(&bytes)
}

pub fn copy_on_apply(
    paths: &Paths,
    shipped_json: &str,
    id: &str,
) -> Result<Profile, ErrorCode> {
    ensure_profile_dirs(paths)?;
    let path = profile_path(paths, id);
    if path.exists() {
        return read_user_profile(paths, id);
    }
    fs::write(&path, shipped_json.as_bytes()).map_err(|_| ErrorCode::Io)?;
    parse_profile(shipped_json.as_bytes())
}

pub fn revert_to_shipped(
    paths: &Paths,
    shipped_json: &str,
    id: &str,
) -> Result<Profile, ErrorCode> {
    ensure_profile_dirs(paths)?;
    let path = profile_path(paths, id);
    fs::write(&path, shipped_json.as_bytes()).map_err(|_| ErrorCode::Io)?;
    parse_profile(shipped_json.as_bytes())
}

pub fn write_active_id(paths: &Paths, id: &str) -> Result<(), ErrorCode> {
    ensure_profile_dirs(paths)?;
    let state = AppStateFile {
        active_profile_id: id.to_owned(),
    };
    let json = serde_json::to_string(&state).map_err(|_| ErrorCode::InvalidProfile)?;
    fs::write(&paths.state_file, json).map_err(|_| ErrorCode::Io)
}

pub fn read_active_id(paths: &Paths) -> Result<Option<String>, ErrorCode> {
    if !paths.state_file.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&paths.state_file).map_err(|_| ErrorCode::Io)?;
    let state: AppStateFile =
        serde_json::from_slice(&bytes).map_err(|_| ErrorCode::InvalidProfile)?;
    Ok(Some(state.active_profile_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::Paths;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_paths() -> Paths {
        let n = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("opentartarus-store-{n}"));
        fs::create_dir_all(&root).unwrap();
        Paths::from_dirs(root.join("cfg"), root.join("run"), root.join("state"))
    }

    const SHIPPED: &str = r#"{
      "id": "default",
      "name": "Default",
      "game": "default",
      "device_models": ["v2", "pro"],
      "bindings": {},
      "lighting": { "effect": "none", "brightness": 80 }
    }"#;

    #[test]
    fn copy_on_apply_creates_user_file_and_does_not_mutate_shipped_bytes() {
        let paths = tmp_paths();
        let p = copy_on_apply(&paths, SHIPPED, "default").unwrap();
        assert_eq!(p.id, "default");
        let on_disk = fs::read_to_string(paths.profiles_dir.join("default.json")).unwrap();
        assert_eq!(on_disk, SHIPPED);
        let mutated = SHIPPED.replace("Default", "Hacked");
        fs::write(paths.profiles_dir.join("default.json"), mutated.as_bytes()).unwrap();
        let again = copy_on_apply(&paths, SHIPPED, "default").unwrap();
        assert_eq!(again.name, "Hacked");
        assert_eq!(SHIPPED.contains("Default"), true);
    }

    #[test]
    fn revert_overwrites_user_file_with_shipped_bytes() {
        let paths = tmp_paths();
        copy_on_apply(&paths, SHIPPED, "default").unwrap();
        fs::write(
            paths.profiles_dir.join("default.json"),
            SHIPPED.replace("Default", "Edited"),
        )
        .unwrap();
        let p = revert_to_shipped(&paths, SHIPPED, "default").unwrap();
        assert_eq!(p.name, "Default");
        assert_eq!(
            fs::read_to_string(paths.profiles_dir.join("default.json")).unwrap(),
            SHIPPED
        );
    }

    #[test]
    fn state_roundtrip() {
        let paths = tmp_paths();
        write_active_id(&paths, "default").unwrap();
        assert_eq!(read_active_id(&paths).unwrap().as_deref(), Some("default"));
    }
}
