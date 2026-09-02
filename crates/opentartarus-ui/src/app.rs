use crate::client::{self, FixPermissionsOutcome, Outgoing};
use crate::theme;
use iced::futures::channel::mpsc;
use iced::keyboard::key::Named;
use iced::keyboard::{self, Key};
use iced::window::{self, Mode};
use iced::{event, Event, Subscription, Task, Theme};
use opentartarus_core::constants::{
    HOLD_REPEAT_RATE_DEFAULT_MS, MACRO_MAX_DELAY_MS, MACRO_MAX_STEPS, RECORD_TIMEOUT_MS,
};
use opentartarus_core::error::ErrorCode;
use opentartarus_core::ipc::{EventMethod, Method, WireError};
use opentartarus_core::pack;
use opentartarus_core::paths::Paths;
use opentartarus_core::store;
use opentartarus_core::types::{
    Action, DeviceModel, Edge, KeyId, KeyToken, Lighting, LightingEffect, MacroKind, MacroStep,
    Modifier, MouseTarget, Profile,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Connecting,
    Running,
    FailedStart,
}

#[derive(Debug, Clone)]
pub struct ProfileRow {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub can_revert: bool,
}

#[derive(Debug, Clone)]
pub enum Banner {
    Starting,
    CouldNotStart,
    NoDevice,
    Permission,
    Disconnect,
    GrabConflict { name: Option<String> },
    UnplugAfterFix,
    SignOut,
    Other(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    IpcReady(mpsc::UnboundedSender<Outgoing>),
    DaemonStarting,
    DaemonFailed,
    DaemonConnected,
    DaemonStopping,
    IpcResponse {
        method: Option<Method>,
        ok: bool,
        result: Option<Value>,
        error: Option<WireError>,
    },
    IpcEvent {
        method: EventMethod,
        params: Value,
    },
    SelectProfile(String),
    SelectKey(KeyId),
    Record,
    StopRecord,
    ClearBinding,
    ComboChanged(String),
    ComboFocus,
    ComboKey {
        key: Key,
        modifiers: keyboard::Modifiers,
    },
    HoldRepeat(bool),
    MousePick(MouseTarget),
    AddMacroStep,
    DeleteMacroStep(usize),
    MacroDelay(usize, String),
    LightingEffect(EffectChoice),
    LightingBrightness(u8),
    LightingColor(usize, u8),
    RevertProfile,
    FixPermissions,
    FixPermissionsDone(FixPermissionsOutcome),
    CloseRequested(window::Id),
    WindowId(Option<window::Id>),
}

pub struct App {
    pub ipc_tx: Option<mpsc::UnboundedSender<Outgoing>>,
    pub phase: Phase,
    pub profiles: Vec<ProfileRow>,
    pub active_profile_id: Option<String>,
    pub selected_profile_id: Option<String>,
    pub device_present: bool,
    pub ever_present: bool,
    pub model: Option<DeviceModel>,
    pub openrazer: bool,
    pub evdev_ok: bool,
    pub uinput_ok: bool,
    pub grab_conflict: Option<String>,
    pub bindings: BTreeMap<KeyId, Action>,
    pub lighting: Lighting,
    pub selected_key: Option<KeyId>,
    pub combo_text: String,
    pub combo_focused: bool,
    pub recording: bool,
    pub window_id: Option<window::Id>,
    pub after_fix_permissions: bool,
    pub last_error: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            ipc_tx: None,
            phase: Phase::Connecting,
            profiles: Vec::new(),
            active_profile_id: None,
            selected_profile_id: None,
            device_present: false,
            ever_present: false,
            model: None,
            openrazer: false,
            evdev_ok: true,
            uinput_ok: true,
            grab_conflict: None,
            bindings: BTreeMap::new(),
            lighting: Lighting {
                effect: LightingEffect::None,
                brightness: theme::DEFAULT_BRIGHTNESS,
                color: None,
            },
            selected_key: None,
            combo_text: String::new(),
            combo_focused: false,
            recording: false,
            window_id: None,
            after_fix_permissions: false,
            last_error: None,
        }
    }
}

impl App {
    pub fn theme(&self) -> Theme {
        theme::theme()
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        crate::view::view(self)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            client::subscription(),
            window::close_requests().map(Message::CloseRequested),
            event::listen_with(|event, _status, _id| match event {
                Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                    Some(Message::ComboKey { key, modifiers })
                }
                _ => None,
            }),
        ])
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::IpcReady(tx) => {
                self.ipc_tx = Some(tx);
                Task::none()
            }
            Message::DaemonStarting => {
                self.phase = Phase::Connecting;
                Task::none()
            }
            Message::DaemonFailed => {
                self.phase = Phase::FailedStart;
                Task::none()
            }
            Message::DaemonConnected => {
                self.phase = Phase::Running;
                self.send(Method::GetStatus, json!({}));
                self.send(Method::ListProfiles, json!({}));
                Task::none()
            }
            Message::DaemonStopping => iced::exit(),
            Message::IpcResponse {
                method,
                ok,
                result,
                error,
            } => {
                if !ok {
                    if let Some(err) = error {
                        self.apply_error(err.code, Some(err.message));
                    }
                    return Task::none();
                }
                if let Some(result) = result {
                    self.handle_result(method, result);
                }
                Task::none()
            }
            Message::IpcEvent { method, params } => self.handle_event(method, params),
            Message::SelectProfile(id) => {
                self.combo_focused = false;
                self.selected_profile_id = Some(id.clone());
                self.send(Method::ApplyProfile, json!({ "id": id }));
                Task::none()
            }
            Message::SelectKey(id) => {
                self.selected_key = Some(id);
                self.combo_focused = false;
                Task::none()
            }
            Message::Record => {
                if let Some(key_id) = self.selected_key {
                    self.send(
                        Method::StartRecord,
                        json!({ "key_id": key_id, "timeout_ms": RECORD_TIMEOUT_MS }),
                    );
                    self.recording = true;
                }
                Task::none()
            }
            Message::StopRecord => {
                self.send(Method::StopRecord, json!({}));
                self.recording = false;
                Task::none()
            }
            Message::ClearBinding => {
                if let (Some(profile_id), Some(key_id)) =
                    (self.profile_id(), self.selected_key)
                {
                    self.bindings.remove(&key_id);
                    self.send(
                        Method::ClearBinding,
                        json!({ "profile_id": profile_id, "key_id": key_id }),
                    );
                }
                Task::none()
            }
            Message::ComboChanged(text) => {
                self.combo_text = text;
                self.combo_focused = true;
                Task::none()
            }
            Message::ComboFocus => {
                self.combo_focused = true;
                Task::none()
            }
            Message::ComboKey { key, modifiers } => {
                if self.combo_focused {
                    if let Some(token) = map_key_token(&key) {
                        let mods = map_modifiers(modifiers);
                        self.combo_text.clear();
                        self.apply_action(Action::Key {
                            key: token,
                            modifiers: mods,
                        });
                    }
                }
                Task::none()
            }
            Message::HoldRepeat(enabled) => {
                if let Some(action) = self.selected_action() {
                    let next = if enabled {
                        match action {
                            Action::HoldRepeat { .. } => action,
                            Action::Key { .. } => Action::HoldRepeat {
                                inner: Box::new(action),
                                rate_ms: HOLD_REPEAT_RATE_DEFAULT_MS,
                            },
                            _ => action,
                        }
                    } else if let Action::HoldRepeat { inner, .. } = action {
                        *inner
                    } else {
                        action
                    };
                    self.apply_action(next);
                }
                Task::none()
            }
            Message::MousePick(target) => {
                self.apply_action(Action::Mouse { target });
                Task::none()
            }
            Message::AddMacroStep => {
                let mut steps = match self.selected_action() {
                    Some(Action::Macro { steps }) => steps,
                    _ => Vec::new(),
                };
                if steps.len() < MACRO_MAX_STEPS {
                    steps.push(MacroStep {
                        kind: MacroKind::Key {
                            key: KeyToken::A,
                            modifiers: vec![],
                        },
                        edge: Edge::Tap,
                        delay_ms: 0,
                    });
                    self.apply_action(Action::Macro { steps });
                }
                Task::none()
            }
            Message::DeleteMacroStep(index) => {
                if let Some(Action::Macro { mut steps }) = self.selected_action() {
                    if index < steps.len() {
                        steps.remove(index);
                    }
                    if steps.is_empty() {
                        if let (Some(profile_id), Some(key_id)) =
                            (self.profile_id(), self.selected_key)
                        {
                            self.bindings.remove(&key_id);
                            self.send(
                                Method::ClearBinding,
                                json!({ "profile_id": profile_id, "key_id": key_id }),
                            );
                        }
                    } else {
                        self.apply_action(Action::Macro { steps });
                    }
                }
                Task::none()
            }
            Message::MacroDelay(index, text) => {
                if let Some(Action::Macro { mut steps }) = self.selected_action() {
                    if let Some(step) = steps.get_mut(index) {
                        let parsed = text.parse::<u32>().unwrap_or(step.delay_ms);
                        step.delay_ms = parsed.min(MACRO_MAX_DELAY_MS);
                    }
                    self.apply_action(Action::Macro { steps });
                }
                Task::none()
            }
            Message::LightingEffect(choice) => {
                let effect = choice.0;
                self.lighting.effect = effect;
                if uses_color(effect) && self.lighting.color.is_none() {
                    self.lighting.color = Some(theme::DEFAULT_LIGHT_COLOR);
                }
                self.send_lighting();
                Task::none()
            }
            Message::LightingBrightness(value) => {
                self.lighting.brightness = value.min(theme::BRIGHTNESS_MAX);
                self.send_lighting();
                Task::none()
            }
            Message::LightingColor(channel, value) => {
                let mut rgb = self.lighting.color.unwrap_or(theme::DEFAULT_LIGHT_COLOR);
                if channel < 3 {
                    rgb[channel] = value;
                }
                self.lighting.color = Some(rgb);
                self.send_lighting();
                Task::none()
            }
            Message::RevertProfile => {
                if let Some(id) = self.profile_id() {
                    self.send(Method::RevertProfile, json!({ "id": id }));
                }
                Task::none()
            }
            Message::FixPermissions => Task::perform(
                client::run_fix_permissions(),
                Message::FixPermissionsDone,
            ),
            Message::FixPermissionsDone(outcome) => {
                match outcome {
                    FixPermissionsOutcome::Success => {
                        self.after_fix_permissions = true;
                        self.send(Method::GetStatus, json!({}));
                    }
                    FixPermissionsOutcome::Cancelled | FixPermissionsOutcome::Failed => {}
                }
                Task::none()
            }
            Message::CloseRequested(id) => {
                self.window_id = Some(id);
                window::change_mode(id, Mode::Hidden)
            }
            Message::WindowId(id) => {
                self.window_id = id;
                Task::none()
            }
        }
    }

    pub fn banner(&self) -> Option<Banner> {
        match self.phase {
            Phase::Connecting => Some(Banner::Starting),
            Phase::FailedStart => Some(Banner::CouldNotStart),
            Phase::Running => {
                if let Some(name) = &self.grab_conflict {
                    let name = if name.is_empty() {
                        None
                    } else {
                        Some(name.clone())
                    };
                    Some(Banner::GrabConflict { name })
                } else if !self.evdev_ok || !self.uinput_ok {
                    if self.after_fix_permissions && !self.evdev_ok {
                        Some(Banner::SignOut)
                    } else if self.after_fix_permissions {
                        Some(Banner::UnplugAfterFix)
                    } else {
                        Some(Banner::Permission)
                    }
                } else if !self.device_present {
                    if self.after_fix_permissions {
                        Some(Banner::UnplugAfterFix)
                    } else if self.ever_present {
                        Some(Banner::Disconnect)
                    } else {
                        Some(Banner::NoDevice)
                    }
                } else if let Some(msg) = &self.last_error {
                    Some(Banner::Other(msg.clone()))
                } else {
                    None
                }
            }
        }
    }

    pub fn selected_action(&self) -> Option<Action> {
        self.selected_key
            .and_then(|id| self.bindings.get(&id).cloned())
    }

    pub fn profile_id(&self) -> Option<String> {
        self.active_profile_id
            .clone()
            .or_else(|| self.selected_profile_id.clone())
    }

    pub fn keypad_faded(&self) -> bool {
        self.ever_present && !self.device_present
    }

    fn send(&self, method: Method, params: Value) {
        if let Some(tx) = &self.ipc_tx {
            let _ = tx.unbounded_send(Outgoing { method, params });
        }
    }

    fn apply_action(&mut self, action: Action) {
        if let (Some(profile_id), Some(key_id)) = (self.profile_id(), self.selected_key) {
            self.bindings.insert(key_id, action.clone());
            self.send(
                Method::SetBinding,
                json!({
                    "profile_id": profile_id,
                    "key_id": key_id,
                    "action": action,
                }),
            );
        }
    }

    fn send_lighting(&self) {
        if let Some(profile_id) = self.profile_id() {
            self.send(
                Method::SetLighting,
                json!({
                    "profile_id": profile_id,
                    "lighting": self.lighting,
                }),
            );
        }
    }

    fn handle_result(&mut self, method: Option<Method>, result: Value) {
        match method {
            Some(Method::GetStatus) | None => {
                if result.get("device").is_some() {
                    self.apply_status(&result);
                }
                if result.get("profiles").is_some() {
                    self.apply_profiles(&result);
                }
            }
            Some(Method::ListProfiles) => self.apply_profiles(&result),
            Some(Method::ApplyProfile)
            | Some(Method::RevertProfile)
            | Some(Method::SetBinding)
            | Some(Method::ClearBinding)
            | Some(Method::SetLighting) => {
                if let Some(id) = result.get("id").and_then(Value::as_str) {
                    self.active_profile_id = Some(id.to_string());
                    self.load_profile(id);
                }
                self.send(Method::ListProfiles, json!({}));
                self.send(Method::GetStatus, json!({}));
            }
            Some(Method::StartRecord) => {
                self.recording = true;
            }
            Some(Method::StopRecord) => {
                self.recording = false;
            }
            _ => {}
        }
    }

    fn handle_event(&mut self, method: EventMethod, params: Value) -> Task<Message> {
        match method {
            EventMethod::DeviceChanged => {
                self.apply_device(&params);
                Task::none()
            }
            EventMethod::Recorded => {
                self.recording = false;
                if let Ok(action) = serde_json::from_value::<Action>(params.get("action").cloned().unwrap_or(Value::Null)) {
                    if let Some(key_id) = params
                        .get("key_id")
                        .and_then(|v| serde_json::from_value::<KeyId>(v.clone()).ok())
                    {
                        self.bindings.insert(key_id, action);
                    }
                }
                Task::none()
            }
            EventMethod::RecordCancelled => {
                self.recording = false;
                Task::none()
            }
            EventMethod::Error => {
                if let Ok(code) = serde_json::from_value::<ErrorCode>(
                    params.get("code").cloned().unwrap_or(Value::Null),
                ) {
                    let message = params
                        .get("message")
                        .and_then(Value::as_str)
                        .map(str::to_owned);
                    self.apply_error(code, message);
                }
                Task::none()
            }
            EventMethod::ProfileApplied => {
                if let Some(id) = params.get("id").and_then(Value::as_str) {
                    self.active_profile_id = Some(id.to_string());
                    self.load_profile(id);
                }
                self.send(Method::ListProfiles, json!({}));
                Task::none()
            }
            EventMethod::FocusWindow => self.show_window(),
            EventMethod::DaemonStopping => iced::exit(),
        }
    }

    fn show_window(&self) -> Task<Message> {
        if let Some(id) = self.window_id {
            return Task::batch([
                window::change_mode(id, Mode::Windowed),
                window::gain_focus(id),
            ]);
        }
        window::get_latest().then(|id| {
            if let Some(id) = id {
                Task::batch([
                    window::change_mode(id, Mode::Windowed),
                    window::gain_focus(id),
                ])
            } else {
                Task::none()
            }
        })
    }

    fn apply_status(&mut self, result: &Value) {
        if let Some(device) = result.get("device") {
            self.apply_device(device);
        }
        self.openrazer = result
            .get("openrazer")
            .and_then(|v| v.get("available"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if let Some(id) = result.get("active_profile_id").and_then(Value::as_str) {
            self.active_profile_id = Some(id.to_string());
            self.load_profile(id);
        }
        self.recording = result
            .get("record")
            .and_then(|v| v.get("active"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        self.uinput_ok = result
            .get("permissions")
            .and_then(|v| v.get("uinput"))
            .and_then(Value::as_bool)
            .unwrap_or(true);
        self.evdev_ok = result
            .get("permissions")
            .and_then(|v| v.get("evdev"))
            .and_then(Value::as_bool)
            .unwrap_or(true);
        self.grab_conflict = match result.get("grab_conflict") {
            Some(Value::String(name)) => Some(name.clone()),
            _ => None,
        };
        if self.device_present && self.evdev_ok && self.uinput_ok {
            self.after_fix_permissions = false;
        }
    }

    fn apply_device(&mut self, device: &Value) {
        self.device_present = device
            .get("present")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if self.device_present {
            self.ever_present = true;
        }
        self.model = device
            .get("model")
            .and_then(|v| serde_json::from_value::<DeviceModel>(v.clone()).ok());
    }

    fn apply_profiles(&mut self, result: &Value) {
        #[derive(Deserialize)]
        struct Row {
            id: String,
            name: String,
            is_active: bool,
            can_revert: bool,
        }
        let rows: Vec<Row> = result
            .get("profiles")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        let mut ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
        ids = opentartarus_core::labels::profile_row_order(&ids);
        let mut by_id: BTreeMap<String, Row> = rows.into_iter().map(|r| (r.id.clone(), r)).collect();
        self.profiles = ids
            .into_iter()
            .filter_map(|id| by_id.remove(&id))
            .map(|r| ProfileRow {
                id: r.id,
                name: r.name,
                is_active: r.is_active,
                can_revert: r.can_revert,
            })
            .collect();
    }

    fn load_profile(&mut self, id: &str) {
        if let Some(profile) = read_profile(id) {
            self.bindings = profile.bindings;
            self.lighting = profile.lighting;
        }
    }

    fn apply_error(&mut self, code: ErrorCode, message: Option<String>) {
        match code {
            ErrorCode::Permission => {
                self.evdev_ok = false;
            }
            ErrorCode::Disconnect => {
                self.device_present = false;
            }
            ErrorCode::Lighting => {
                self.openrazer = false;
            }
            ErrorCode::GrabConflict => {
                self.grab_conflict = message.clone();
            }
            ErrorCode::AlreadyRunning => {}
            other => {
                self.last_error = Some(other.user_message().to_string());
            }
        }
        if matches!(
            code,
            ErrorCode::InvalidProfile
                | ErrorCode::UnknownKey
                | ErrorCode::RecordBusy
                | ErrorCode::NotFound
                | ErrorCode::Io
        ) {
            self.last_error = Some(code.user_message().to_string());
        }
    }
}

pub fn uses_color(effect: LightingEffect) -> bool {
    matches!(
        effect,
        LightingEffect::Static
            | LightingEffect::Breath
            | LightingEffect::Reactive
            | LightingEffect::Starlight
    )
}

pub fn effect_name(effect: LightingEffect) -> &'static str {
    match effect {
        LightingEffect::Static => "static",
        LightingEffect::Wave => "wave",
        LightingEffect::Spectrum => "spectrum",
        LightingEffect::Breath => "breath",
        LightingEffect::Reactive => "reactive",
        LightingEffect::Starlight => "starlight",
        LightingEffect::None => "none",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectChoice(pub LightingEffect);

impl std::fmt::Display for EffectChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(effect_name(self.0))
    }
}

pub const LIGHTING_EFFECTS: [EffectChoice; 7] = [
    EffectChoice(LightingEffect::Static),
    EffectChoice(LightingEffect::Wave),
    EffectChoice(LightingEffect::Spectrum),
    EffectChoice(LightingEffect::Breath),
    EffectChoice(LightingEffect::Reactive),
    EffectChoice(LightingEffect::Starlight),
    EffectChoice(LightingEffect::None),
];

fn read_profile(id: &str) -> Option<Profile> {
    let paths = Paths::from_env();
    store::read_user_profile(&paths, id)
        .or_else(|_| pack::shipped_profile(id))
        .ok()
}

fn map_modifiers(modifiers: keyboard::Modifiers) -> Vec<Modifier> {
    let mut out = Vec::new();
    if modifiers.control() {
        out.push(Modifier::Ctrl);
    }
    if modifiers.shift() {
        out.push(Modifier::Shift);
    }
    if modifiers.alt() {
        out.push(Modifier::Alt);
    }
    if modifiers.logo() {
        out.push(Modifier::Super);
    }
    out
}

fn map_key_token(key: &Key) -> Option<KeyToken> {
    match key {
        Key::Character(c) => match c.to_ascii_lowercase().as_str() {
            "a" => Some(KeyToken::A),
            "b" => Some(KeyToken::B),
            "c" => Some(KeyToken::C),
            "d" => Some(KeyToken::D),
            "e" => Some(KeyToken::E),
            "f" => Some(KeyToken::F),
            "g" => Some(KeyToken::G),
            "h" => Some(KeyToken::H),
            "i" => Some(KeyToken::I),
            "j" => Some(KeyToken::J),
            "k" => Some(KeyToken::K),
            "l" => Some(KeyToken::L),
            "m" => Some(KeyToken::M),
            "n" => Some(KeyToken::N),
            "o" => Some(KeyToken::O),
            "p" => Some(KeyToken::P),
            "q" => Some(KeyToken::Q),
            "r" => Some(KeyToken::R),
            "s" => Some(KeyToken::S),
            "t" => Some(KeyToken::T),
            "u" => Some(KeyToken::U),
            "v" => Some(KeyToken::V),
            "w" => Some(KeyToken::W),
            "x" => Some(KeyToken::X),
            "y" => Some(KeyToken::Y),
            "z" => Some(KeyToken::Z),
            "0" => Some(KeyToken::Num0),
            "1" => Some(KeyToken::Num1),
            "2" => Some(KeyToken::Num2),
            "3" => Some(KeyToken::Num3),
            "4" => Some(KeyToken::Num4),
            "5" => Some(KeyToken::Num5),
            "6" => Some(KeyToken::Num6),
            "7" => Some(KeyToken::Num7),
            "8" => Some(KeyToken::Num8),
            "9" => Some(KeyToken::Num9),
            "-" => Some(KeyToken::Minus),
            "=" => Some(KeyToken::Equal),
            "[" => Some(KeyToken::LeftBrace),
            "]" => Some(KeyToken::RightBracket),
            "\\" => Some(KeyToken::Backslash),
            ";" => Some(KeyToken::Semicolon),
            "'" => Some(KeyToken::Apostrophe),
            "`" => Some(KeyToken::Grave),
            "," => Some(KeyToken::Comma),
            "." => Some(KeyToken::Dot),
            "/" => Some(KeyToken::Slash),
            _ => None,
        },
        Key::Named(Named::Escape) => Some(KeyToken::Escape),
        Key::Named(Named::Tab) => Some(KeyToken::Tab),
        Key::Named(Named::Backspace) => Some(KeyToken::Backspace),
        Key::Named(Named::Enter) => Some(KeyToken::Enter),
        Key::Named(Named::Space) => Some(KeyToken::Space),
        Key::Named(Named::Insert) => Some(KeyToken::Insert),
        Key::Named(Named::Delete) => Some(KeyToken::Delete),
        Key::Named(Named::Home) => Some(KeyToken::Home),
        Key::Named(Named::End) => Some(KeyToken::End),
        Key::Named(Named::PageUp) => Some(KeyToken::PageUp),
        Key::Named(Named::PageDown) => Some(KeyToken::PageDown),
        Key::Named(Named::ArrowUp) => Some(KeyToken::Up),
        Key::Named(Named::ArrowDown) => Some(KeyToken::Down),
        Key::Named(Named::ArrowLeft) => Some(KeyToken::Left),
        Key::Named(Named::ArrowRight) => Some(KeyToken::Right),
        Key::Named(Named::F1) => Some(KeyToken::F1),
        Key::Named(Named::F2) => Some(KeyToken::F2),
        Key::Named(Named::F3) => Some(KeyToken::F3),
        Key::Named(Named::F4) => Some(KeyToken::F4),
        Key::Named(Named::F5) => Some(KeyToken::F5),
        Key::Named(Named::F6) => Some(KeyToken::F6),
        Key::Named(Named::F7) => Some(KeyToken::F7),
        Key::Named(Named::F8) => Some(KeyToken::F8),
        Key::Named(Named::F9) => Some(KeyToken::F9),
        Key::Named(Named::F10) => Some(KeyToken::F10),
        Key::Named(Named::F11) => Some(KeyToken::F11),
        Key::Named(Named::F12) => Some(KeyToken::F12),
        Key::Named(Named::Shift)
        | Key::Named(Named::Control)
        | Key::Named(Named::Alt)
        | Key::Named(Named::Super)
        | Key::Named(Named::Meta) => None,
        _ => None,
    }
}
