use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub const CARGO_BIN_EXE_PREFIX: &str = "CARGO_BIN_EXE_";
pub const CARGO_TARGET_DIR_VAR: &str = "CARGO_TARGET_DIR";
pub const CARGO_MANIFEST_DIR_VAR: &str = "CARGO_MANIFEST_DIR";
pub const CARGO_PROFILE_DEBUG: &str = "debug";
pub const CARGO_PROFILE_RELEASE: &str = "release";
pub const TARGET_DIR_NAME: &str = "target";
const WORKSPACE_FROM_CRATE_DEPTH: usize = 2;
const CARGO_BIN_EXE_HYPHEN: char = '-';
const CARGO_BIN_EXE_UNDERSCORE: &str = "_";

pub fn cargo_bin_exe_var(bin_name: &str) -> String {
    let mut name = String::from(CARGO_BIN_EXE_PREFIX);
    name.push_str(&bin_name.replace(CARGO_BIN_EXE_HYPHEN, CARGO_BIN_EXE_UNDERSCORE));
    name
}

pub fn resolve_bin(
    bin_name: &str,
    current_exe: Option<&Path>,
    mut env: impl FnMut(&str) -> Option<OsString>,
    exists: impl Fn(&Path) -> bool,
) -> PathBuf {
    let mut candidates = Vec::new();
    if let Some(exe) = current_exe {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(bin_name));
        }
    }
    if let Some(val) = env(&cargo_bin_exe_var(bin_name)) {
        candidates.push(PathBuf::from(val));
    }
    if let Some(target) = env(CARGO_TARGET_DIR_VAR) {
        let target = PathBuf::from(target);
        candidates.push(target.join(CARGO_PROFILE_DEBUG).join(bin_name));
        candidates.push(target.join(CARGO_PROFILE_RELEASE).join(bin_name));
    }
    if let Some(manifest) = env(CARGO_MANIFEST_DIR_VAR) {
        let mut workspace = PathBuf::from(manifest);
        for _ in 0..WORKSPACE_FROM_CRATE_DEPTH {
            workspace.pop();
        }
        let target = workspace.join(TARGET_DIR_NAME);
        candidates.push(target.join(CARGO_PROFILE_DEBUG).join(bin_name));
        candidates.push(target.join(CARGO_PROFILE_RELEASE).join(bin_name));
    }
    for path in candidates {
        if exists(&path) {
            return path;
        }
    }
    PathBuf::from(bin_name)
}

pub fn resolve_bin_from_env(bin_name: &str) -> PathBuf {
    let exe = std::env::current_exe().ok();
    resolve_bin(
        bin_name,
        exe.as_deref(),
        |key| std::env::var_os(key),
        |path| path.is_file(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::Path;

    const DAEMON: &str = "opentartarus-daemon";

    fn env_map(pairs: &[(&str, &str)]) -> HashMap<String, OsString> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), OsString::from(*v)))
            .collect()
    }

    fn lookup(
        current_exe: Option<&str>,
        env: &HashMap<String, OsString>,
        present: &[&str],
    ) -> PathBuf {
        resolve_bin(
            DAEMON,
            current_exe.map(Path::new),
            |key| env.get(key).cloned(),
            |path| present.iter().any(|p| Path::new(p) == path),
        )
    }

    #[test]
    fn cargo_bin_exe_var_replaces_hyphens() {
        assert_eq!(
            cargo_bin_exe_var(DAEMON),
            "CARGO_BIN_EXE_opentartarus_daemon"
        );
    }

    #[test]
    fn sibling_of_current_exe_wins() {
        let found = lookup(
            Some("/repo/target/debug/opentartarus-ui"),
            &HashMap::new(),
            &["/repo/target/debug/opentartarus-daemon"],
        );
        assert_eq!(found, Path::new("/repo/target/debug/opentartarus-daemon"));
    }

    #[test]
    fn cargo_bin_exe_used_when_sibling_missing() {
        let env = env_map(&[(
            "CARGO_BIN_EXE_opentartarus_daemon",
            "/workspace/target/debug/opentartarus-daemon",
        )]);
        let found = lookup(
            Some("/usr/bin/opentartarus-ui"),
            &env,
            &["/workspace/target/debug/opentartarus-daemon"],
        );
        assert_eq!(
            found,
            Path::new("/workspace/target/debug/opentartarus-daemon")
        );
    }

    #[test]
    fn cargo_target_dir_debug_used_from_cargo_run() {
        let env = env_map(&[("CARGO_TARGET_DIR", "/tmp/ot-target")]);
        let found = lookup(
            Some("/usr/bin/opentartarus-ui"),
            &env,
            &["/tmp/ot-target/debug/opentartarus-daemon"],
        );
        assert_eq!(found, Path::new("/tmp/ot-target/debug/opentartarus-daemon"));
    }

    #[test]
    fn cargo_manifest_dir_finds_workspace_target_debug() {
        let env = env_map(&[("CARGO_MANIFEST_DIR", "/repo/crates/opentartarus-ui")]);
        let found = lookup(
            Some("/usr/bin/opentartarus-ui"),
            &env,
            &["/repo/target/debug/opentartarus-daemon"],
        );
        assert_eq!(found, Path::new("/repo/target/debug/opentartarus-daemon"));
    }

    #[test]
    fn missing_everywhere_falls_back_to_bare_name() {
        let found = lookup(None, &HashMap::new(), &[]);
        assert_eq!(found, Path::new(DAEMON));
    }
}
