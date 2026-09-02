use opentartarus_core::error::ErrorCode;
use std::fs::OpenOptions;
use std::path::Path;

pub fn probe_uinput() -> bool {
    OpenOptions::new().write(true).open("/dev/uinput").is_ok()
}

pub fn probe_evdev_readable(path: &Path) -> bool {
    OpenOptions::new().read(true).open(path).is_ok()
}

pub fn permission_error_from_probes(uinput_ok: bool, evdev_ok: bool) -> Option<ErrorCode> {
    if uinput_ok && evdev_ok {
        None
    } else {
        Some(ErrorCode::Permission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentartarus_core::error::ErrorCode;

    #[test]
    fn probes_to_permission() {
        assert_eq!(
            permission_error_from_probes(false, true),
            Some(ErrorCode::Permission)
        );
        assert_eq!(permission_error_from_probes(true, true), None);
        assert_eq!(
            ErrorCode::Permission.log_line(Some("EACCES")),
            "permission OpenTartarus can’t talk to your keypad yet. os=EACCES"
        );
    }
}
