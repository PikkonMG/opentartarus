use iced::theme::Palette;
use iced::{Color, Theme};

pub const COLOR_BACKGROUND: Color = Color::from_rgb(
    0x12 as f32 / 255.0,
    0x14 as f32 / 255.0,
    0x1a as f32 / 255.0,
);
pub const COLOR_SURFACE: Color = Color::from_rgb(
    0x1b as f32 / 255.0,
    0x1e as f32 / 255.0,
    0x27 as f32 / 255.0,
);
pub const COLOR_TEXT: Color = Color::from_rgb(
    0xe8 as f32 / 255.0,
    0xea as f32 / 255.0,
    0xed as f32 / 255.0,
);
pub const COLOR_ACCENT: Color = Color::from_rgb(
    0x3d as f32 / 255.0,
    0x8b as f32 / 255.0,
    0xfd as f32 / 255.0,
);
pub const COLOR_DANGER: Color = Color::from_rgb(
    0xe3 as f32 / 255.0,
    0x5d as f32 / 255.0,
    0x6a as f32 / 255.0,
);

pub const PROFILE_LIST_WIDTH: f32 = 220.0;
pub const BIND_PANEL_WIDTH: f32 = 280.0;
pub const LIGHTING_STRIP_HEIGHT: f32 = 48.0;
pub const WINDOW_WIDTH: f32 = 1100.0;
pub const WINDOW_HEIGHT: f32 = 720.0;
pub const DISCONNECTED_OPACITY: f32 = 0.40;
pub const BRIGHTNESS_MIN: u8 = 0;
pub const BRIGHTNESS_MAX: u8 = 100;
pub const COLOR_CHANNEL_MAX: u8 = 255;
pub const DEFAULT_LIGHT_COLOR: [u8; 3] = [255, 255, 255];
pub const DEFAULT_BRIGHTNESS: u8 = 80;

pub const BANNER_STARTING: &str = "Starting…";
pub const BANNER_COULD_NOT_START_PREFIX: &str = "OpenTartarus couldn’t start.";
pub const START_REASON_TRAY_DID_NOT_START: &str = "The tray didn’t start.";
pub const BANNER_NO_DEVICE: &str = "No Tartarus found. Unplug it, wait a second, plug it back in.";
pub const BANNER_UNPLUG_AFTER_FIX: &str = "Unplug the Tartarus, wait a second, plug it back in.";
pub const BANNER_SIGN_OUT: &str = "Sign out and sign back in, then open OpenTartarus again.";
pub const BUTTON_FIX_PERMISSIONS: &str = "Fix permissions";
pub const BUTTON_REVERT: &str = "Revert to shipped";
pub const BUTTON_RECORD: &str = "Record";
pub const BUTTON_CANCEL: &str = "Cancel";
pub const BUTTON_CLEAR: &str = "Clear";
pub const BUTTON_ADD_STEP: &str = "Add step";
pub const BUTTON_QUIT: &str = "Quit";
pub const COMBO_PLACEHOLDER: &str = "Type a combo";
pub const HOLD_REPEAT_LABEL: &str = "Hold to repeat";
pub const MOUSE_LEFT: &str = "Left";
pub const MOUSE_RIGHT: &str = "Right";
pub const MOUSE_MIDDLE: &str = "Middle";
pub const MOUSE_BACK: &str = "Back";
pub const MOUSE_FORWARD: &str = "Forward";
pub const MOUSE_WHEEL_UP: &str = "Wheel+";
pub const MOUSE_WHEEL_DOWN: &str = "Wheel-";

pub fn theme() -> Theme {
    Theme::custom(
        String::from("OpenTartarus"),
        Palette {
            background: COLOR_BACKGROUND,
            text: COLOR_TEXT,
            primary: COLOR_ACCENT,
            success: COLOR_ACCENT,
            danger: COLOR_DANGER,
        },
    )
}

pub fn with_opacity(color: Color, opacity: f32) -> Color {
    Color {
        a: color.a * opacity,
        ..color
    }
}

pub fn could_not_start_message(reason: &str) -> String {
    format!("{BANNER_COULD_NOT_START_PREFIX} {reason}")
}
