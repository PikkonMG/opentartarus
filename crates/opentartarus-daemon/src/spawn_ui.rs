use opentartarus_core::binpath::resolve_bin_from_env;
use std::process::{Child, Command, Stdio};

pub const UI_BIN: &str = "opentartarus-ui";

pub fn ui_command() -> Command {
    Command::new(resolve_bin_from_env(UI_BIN))
}

pub fn spawn_ui() -> std::io::Result<Child> {
    ui_command()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}

pub struct UiSupervisor {
    child: Option<Child>,
}

impl UiSupervisor {
    pub fn new() -> Self {
        Self { child: None }
    }

    pub fn is_running(&mut self) -> bool {
        match self.child.as_mut() {
            Some(child) => matches!(child.try_wait(), Ok(None)),
            None => false,
        }
    }

    /// Returns true when the UI is already running and the caller should send FocusWindow.
    pub fn ensure_open(&mut self, clients_connected: bool) -> bool {
        if clients_connected || self.is_running() {
            return true;
        }
        match spawn_ui() {
            Ok(child) => {
                self.child = Some(child);
                false
            }
            Err(err) => {
                crate::log::write(&format!("spawn_ui failed os={err}"));
                false
            }
        }
    }
}

impl Default for UiSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_bin_is_opentartarus_ui() {
        assert_eq!(UI_BIN, "opentartarus-ui");
        let program = ui_command().get_program().to_string_lossy().into_owned();
        assert!(
            program.ends_with(UI_BIN),
            "expected command ending in {UI_BIN}, got {program}"
        );
    }
}
