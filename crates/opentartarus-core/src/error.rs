use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    Permission,
    Disconnect,
    Lighting,
    GrabConflict,
    InvalidProfile,
    UnknownKey,
    RecordBusy,
    NotFound,
    Io,
    AlreadyRunning,
}

impl ErrorCode {
    pub fn wire_name(self) -> &'static str {
        match self {
            Self::Permission => "permission",
            Self::Disconnect => "disconnect",
            Self::Lighting => "lighting",
            Self::GrabConflict => "grab_conflict",
            Self::InvalidProfile => "invalid_profile",
            Self::UnknownKey => "unknown_key",
            Self::RecordBusy => "record_busy",
            Self::NotFound => "not_found",
            Self::Io => "io",
            Self::AlreadyRunning => "already_running",
        }
    }

    pub fn log_line(self, os: Option<&str>) -> String {
        format!(
            "{} {} os={}",
            self.wire_name(),
            self.user_message(),
            os.unwrap_or("")
        )
    }

    pub fn user_message(self) -> &'static str {
        match self {
            Self::Permission => "OpenTartarus can’t talk to your keypad yet.",
            Self::Disconnect => "Tartarus disconnected.",
            Self::Lighting => "Lighting needs OpenRazer or OpenRGB.",
            Self::GrabConflict => {
                "Something else is using the Tartarus. Quit that app and reopen OpenTartarus."
            }
            Self::InvalidProfile => {
                "That profile file is damaged. Revert to shipped or pick another."
            }
            Self::UnknownKey => "That key isn’t on this keypad.",
            Self::RecordBusy => "Already recording. Press a key or click Cancel.",
            Self::NotFound => "That profile isn’t installed.",
            Self::Io => "OpenTartarus couldn’t save. Try again.",
            Self::AlreadyRunning => "already_running",
        }
    }
}
