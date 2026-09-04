use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceModel {
    V2,
    Pro,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum KeyId {
    #[serde(rename = "kp01")]
    Kp01,
    #[serde(rename = "kp02")]
    Kp02,
    #[serde(rename = "kp03")]
    Kp03,
    #[serde(rename = "kp04")]
    Kp04,
    #[serde(rename = "kp05")]
    Kp05,
    #[serde(rename = "kp06")]
    Kp06,
    #[serde(rename = "kp07")]
    Kp07,
    #[serde(rename = "kp08")]
    Kp08,
    #[serde(rename = "kp09")]
    Kp09,
    #[serde(rename = "kp10")]
    Kp10,
    #[serde(rename = "kp11")]
    Kp11,
    #[serde(rename = "kp12")]
    Kp12,
    #[serde(rename = "kp13")]
    Kp13,
    #[serde(rename = "kp14")]
    Kp14,
    #[serde(rename = "kp15")]
    Kp15,
    #[serde(rename = "kp16")]
    Kp16,
    #[serde(rename = "kp17")]
    Kp17,
    #[serde(rename = "kp18")]
    Kp18,
    #[serde(rename = "kp19")]
    Kp19,
    #[serde(rename = "kp20")]
    Kp20,
    #[serde(rename = "wheel_up")]
    WheelUp,
    #[serde(rename = "wheel_down")]
    WheelDown,
    #[serde(rename = "wheel_click")]
    WheelClick,
    #[serde(rename = "mode")]
    Mode,
    #[serde(rename = "thumb_n")]
    ThumbN,
    #[serde(rename = "thumb_ne")]
    ThumbNe,
    #[serde(rename = "thumb_e")]
    ThumbE,
    #[serde(rename = "thumb_se")]
    ThumbSe,
    #[serde(rename = "thumb_s")]
    ThumbS,
    #[serde(rename = "thumb_sw")]
    ThumbSw,
    #[serde(rename = "thumb_w")]
    ThumbW,
    #[serde(rename = "thumb_nw")]
    ThumbNw,
    #[serde(rename = "analog_up")]
    AnalogUp,
    #[serde(rename = "analog_down")]
    AnalogDown,
    #[serde(rename = "analog_left")]
    AnalogLeft,
    #[serde(rename = "analog_right")]
    AnalogRight,
}

impl KeyId {
    /// Every physical key, in declaration order.
    /// 20 grid keys + 3 wheel + mode + 8 thumb directions + 4 analog = 36.
    /// Used by exhaustive tests and by the keypad widget's geometry check.
    pub const ALL: [KeyId; 36] = [
        KeyId::Kp01,
        KeyId::Kp02,
        KeyId::Kp03,
        KeyId::Kp04,
        KeyId::Kp05,
        KeyId::Kp06,
        KeyId::Kp07,
        KeyId::Kp08,
        KeyId::Kp09,
        KeyId::Kp10,
        KeyId::Kp11,
        KeyId::Kp12,
        KeyId::Kp13,
        KeyId::Kp14,
        KeyId::Kp15,
        KeyId::Kp16,
        KeyId::Kp17,
        KeyId::Kp18,
        KeyId::Kp19,
        KeyId::Kp20,
        KeyId::WheelUp,
        KeyId::WheelDown,
        KeyId::WheelClick,
        KeyId::Mode,
        KeyId::ThumbN,
        KeyId::ThumbNe,
        KeyId::ThumbE,
        KeyId::ThumbSe,
        KeyId::ThumbS,
        KeyId::ThumbSw,
        KeyId::ThumbW,
        KeyId::ThumbNw,
        KeyId::AnalogUp,
        KeyId::AnalogDown,
        KeyId::AnalogLeft,
        KeyId::AnalogRight,
    ];

    pub fn is_analog(self) -> bool {
        matches!(
            self,
            Self::AnalogUp | Self::AnalogDown | Self::AnalogLeft | Self::AnalogRight
        )
    }

    pub fn is_thumb(self) -> bool {
        matches!(
            self,
            Self::ThumbN
                | Self::ThumbNe
                | Self::ThumbE
                | Self::ThumbSe
                | Self::ThumbS
                | Self::ThumbSw
                | Self::ThumbW
                | Self::ThumbNw
        )
    }

    pub(crate) fn from_binding_key(key: &str) -> Option<Self> {
        serde_json::from_value(serde_json::Value::String(key.to_owned())).ok()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyToken {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    #[serde(rename = "0")]
    Num0,
    #[serde(rename = "1")]
    Num1,
    #[serde(rename = "2")]
    Num2,
    #[serde(rename = "3")]
    Num3,
    #[serde(rename = "4")]
    Num4,
    #[serde(rename = "5")]
    Num5,
    #[serde(rename = "6")]
    Num6,
    #[serde(rename = "7")]
    Num7,
    #[serde(rename = "8")]
    Num8,
    #[serde(rename = "9")]
    Num9,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Escape,
    Tab,
    Backspace,
    Enter,
    Space,
    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
    Minus,
    Equal,
    LeftBrace,
    RightBracket,
    Backslash,
    Semicolon,
    Apostrophe,
    Grave,
    Comma,
    Dot,
    Slash,
    #[serde(rename = "num0")]
    NumPad0,
    #[serde(rename = "num1")]
    NumPad1,
    #[serde(rename = "num2")]
    NumPad2,
    #[serde(rename = "num3")]
    NumPad3,
    #[serde(rename = "num4")]
    NumPad4,
    #[serde(rename = "num5")]
    NumPad5,
    #[serde(rename = "num6")]
    NumPad6,
    #[serde(rename = "num7")]
    NumPad7,
    #[serde(rename = "num8")]
    NumPad8,
    #[serde(rename = "num9")]
    NumPad9,
    NumEnter,
    NumPlus,
    NumMinus,
    NumSlash,
    NumStar,
    NumDot,
    PrintScreen,
    ScrollLock,
    Pause,
    CapsLock,
    NumLock,
    VolumeUp,
    VolumeDown,
    Mute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modifier {
    Ctrl,
    Shift,
    Alt,
    Super,
    CtrlR,
    ShiftR,
    AltR,
    SuperR,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScrollDir {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Edge {
    Down,
    Up,
    Tap,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MouseTarget {
    Button { button: MouseButton },
    Scroll { scroll: ScrollDir },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum MacroKind {
    Key {
        key: KeyToken,
        modifiers: Vec<Modifier>,
    },
    Mouse {
        #[serde(flatten)]
        target: MouseTarget,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacroStep {
    #[serde(flatten)]
    pub kind: MacroKind,
    pub edge: Edge,
    pub delay_ms: u32,
}

fn default_hold_repeat_rate_ms() -> u32 {
    crate::constants::HOLD_REPEAT_RATE_DEFAULT_MS
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Key {
        key: KeyToken,
        modifiers: Vec<Modifier>,
    },
    Macro {
        steps: Vec<MacroStep>,
    },
    Mouse {
        #[serde(flatten)]
        target: MouseTarget,
    },
    HoldRepeat {
        inner: Box<Action>,
        #[serde(default = "default_hold_repeat_rate_ms")]
        rate_ms: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LightingEffect {
    Static,
    Wave,
    Spectrum,
    Breath,
    Reactive,
    Starlight,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lighting {
    pub effect: LightingEffect,
    pub brightness: u8,
    #[serde(default)]
    pub color: Option<[u8; 3]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GameId {
    Default,
    LeagueOfLegends,
    #[serde(rename = "dota-2")]
    Dota2,
    WorldOfWarcraft,
    FinalFantasyXiv,
    PathOfExile,
    #[serde(rename = "overwatch-2")]
    Overwatch2,
    Valorant,
    #[serde(rename = "counter-strike-2")]
    CounterStrike2,
    ApexLegends,
    Fortnite,
    #[serde(rename = "diablo-4")]
    Diablo4,
    EldenRing,
    #[serde(rename = "guild-wars-2")]
    GuildWars2,
    ElderScrollsOnline,
    Minecraft,
    /// A profile the user made, not one from the shipped pack.
    Custom,
}

#[derive(Debug, Deserialize)]
struct ProfileDe {
    id: String,
    name: String,
    game: GameId,
    device_models: Vec<DeviceModel>,
    bindings: BTreeMap<String, Action>,
    lighting: Lighting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "ProfileDe")]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub game: GameId,
    pub device_models: Vec<DeviceModel>,
    pub bindings: BTreeMap<KeyId, Action>,
    pub lighting: Lighting,
    #[serde(skip)]
    pub(crate) unknown_key_ids: bool,
}

impl From<ProfileDe> for Profile {
    fn from(raw: ProfileDe) -> Self {
        let mut bindings = BTreeMap::new();
        let mut unknown_key_ids = false;
        for (key, action) in raw.bindings {
            match KeyId::from_binding_key(&key) {
                Some(id) => {
                    bindings.insert(id, action);
                }
                None => {
                    unknown_key_ids = true;
                }
            }
        }
        Self {
            id: raw.id,
            name: raw.name,
            game: raw.game,
            device_models: raw.device_models,
            bindings,
            lighting: raw.lighting,
            unknown_key_ids,
        }
    }
}
