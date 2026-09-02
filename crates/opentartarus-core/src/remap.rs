use crate::constants::{
    ANALOG_PRESS_RATIO, ANALOG_RELEASE_RATIO, TAP_HOLD_MS, USB_PID_TARTARUS_PRO,
    USB_PID_TARTARUS_V2, USB_VID_RAZER,
};
use crate::keymap::{
    analog_axis_ratio, decode_hat, modifier_to_evdev, mouse_button_to_evdev, token_to_evdev,
    KeyMap, ABS_HAT0X, ABS_HAT0Y, ABS_X, ABS_Y, EV_ABS, EV_KEY, EV_REL, REL_WHEEL,
};
use crate::types::{Action, DeviceModel, Edge, KeyId, MacroKind, MouseTarget, Profile, ScrollDir};
use std::collections::BTreeMap;

const KEY_DOWN: i32 = 1;
const KEY_REPEAT: i32 = 2;
const KEY_UP: i32 = 0;
const ANALOG_AXIS_CENTER: i32 = 128;

pub fn allow_grab(vid: u16, pid: u16) -> bool {
    vid == USB_VID_RAZER && (pid == USB_PID_TARTARUS_V2 || pid == USB_PID_TARTARUS_PRO)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawEvent {
    pub vid: u16,
    pub pid: u16,
    pub ev_type: u16,
    pub code: u16,
    pub value: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Emitted {
    pub keyboard: bool,
    pub ev_type: u16,
    pub code: u16,
    pub value: i32,
}

pub trait EventSink {
    fn emit(&mut self, e: Emitted);
}

pub trait Clock {
    fn now_ms(&self) -> u64;
    fn sleep_ms(&mut self, ms: u64);
}

pub struct RemapEngine {
    model: DeviceModel,
    map: KeyMap,
    table: BTreeMap<KeyId, Action>,
    macro_busy: bool,
    hold: Option<(KeyId, u64, u32)>,
    hat_x: i32,
    hat_y: i32,
    hat_dir: Option<KeyId>,
    analog_x: i32,
    analog_y: i32,
    analog_left: bool,
    analog_right: bool,
    analog_up: bool,
    analog_down: bool,
}

impl RemapEngine {
    pub fn new(model: DeviceModel) -> Self {
        let map = match model {
            DeviceModel::V2 => KeyMap::v2(),
            DeviceModel::Pro => KeyMap::pro(),
        };
        Self {
            model,
            map,
            table: BTreeMap::new(),
            macro_busy: false,
            hold: None,
            hat_x: 0,
            hat_y: 0,
            hat_dir: None,
            analog_x: ANALOG_AXIS_CENTER,
            analog_y: ANALOG_AXIS_CENTER,
            analog_left: false,
            analog_right: false,
            analog_up: false,
            analog_down: false,
        }
    }

    pub fn apply_profile(&mut self, p: &Profile) {
        self.table = p.bindings_for_model(self.model);
    }

    pub fn handle(&mut self, ev: RawEvent, sink: &mut impl EventSink, clock: &mut impl Clock) {
        if !allow_grab(ev.vid, ev.pid) {
            return;
        }
        if ev.ev_type == EV_ABS && (ev.code == ABS_HAT0X || ev.code == ABS_HAT0Y) {
            self.handle_hat(ev, sink, clock);
            return;
        }
        if ev.ev_type == EV_ABS && (ev.code == ABS_X || ev.code == ABS_Y) {
            self.handle_analog(ev, sink, clock);
            return;
        }
        let qualifier = if ev.ev_type == EV_REL {
            ev.value
        } else {
            KEY_DOWN
        };
        let Some(key_id) = self.map.get(ev.ev_type, ev.code, qualifier) else {
            return;
        };
        if ev.ev_type == EV_REL {
            self.dispatch(key_id, KEY_DOWN, ev, sink, clock);
            if !matches!(self.table.get(&key_id), Some(Action::Macro { .. })) {
                self.dispatch(key_id, KEY_UP, ev, sink, clock);
            }
            return;
        }
        self.dispatch(key_id, ev.value, ev, sink, clock);
    }

    fn handle_hat(&mut self, ev: RawEvent, sink: &mut impl EventSink, clock: &mut impl Clock) {
        if ev.code == ABS_HAT0X {
            self.hat_x = ev.value;
        } else {
            self.hat_y = ev.value;
        }
        let next = decode_hat(self.hat_x, self.hat_y);
        if next == self.hat_dir {
            return;
        }
        if let Some(prev) = self.hat_dir {
            self.dispatch(prev, KEY_UP, ev, sink, clock);
        }
        self.hat_dir = next;
        if let Some(id) = next {
            self.dispatch(id, KEY_DOWN, ev, sink, clock);
        }
    }

    fn handle_analog(&mut self, ev: RawEvent, sink: &mut impl EventSink, clock: &mut impl Clock) {
        if ev.code == ABS_X {
            self.analog_x = ev.value;
            let ratio = analog_axis_ratio(self.analog_x);
            self.apply_analog_dir(KeyId::AnalogRight, ratio, ev, sink, clock);
            self.apply_analog_dir(KeyId::AnalogLeft, -ratio, ev, sink, clock);
        } else {
            self.analog_y = ev.value;
            let ratio = analog_axis_ratio(self.analog_y);
            self.apply_analog_dir(KeyId::AnalogDown, ratio, ev, sink, clock);
            self.apply_analog_dir(KeyId::AnalogUp, -ratio, ev, sink, clock);
        }
    }

    fn apply_analog_dir(
        &mut self,
        id: KeyId,
        ratio: f32,
        ev: RawEvent,
        sink: &mut impl EventSink,
        clock: &mut impl Clock,
    ) {
        let pressed = analog_pressed(self, id);
        if !pressed && ratio >= ANALOG_PRESS_RATIO {
            set_analog_pressed(self, id, true);
            self.dispatch(id, KEY_DOWN, ev, sink, clock);
        } else if pressed && ratio <= ANALOG_RELEASE_RATIO {
            set_analog_pressed(self, id, false);
            self.dispatch(id, KEY_UP, ev, sink, clock);
        }
    }

    fn dispatch(
        &mut self,
        key_id: KeyId,
        value: i32,
        ev: RawEvent,
        sink: &mut impl EventSink,
        clock: &mut impl Clock,
    ) {
        match self.table.get(&key_id).cloned() {
            None => {
                if value != KEY_UP {
                    sink.emit(Emitted {
                        keyboard: ev.ev_type == EV_KEY,
                        ev_type: ev.ev_type,
                        code: ev.code,
                        value: ev.value,
                    });
                }
            }
            Some(action) => match value {
                KEY_DOWN => self.play_press(key_id, &action, sink, clock),
                KEY_REPEAT => self.play_repeat(key_id, &action, sink, clock),
                KEY_UP => self.play_release(&action, sink),
                _ => {}
            },
        }
    }

    fn play_press(
        &mut self,
        key_id: KeyId,
        action: &Action,
        sink: &mut impl EventSink,
        clock: &mut impl Clock,
    ) {
        match action {
            Action::Macro { .. } if self.macro_busy => {}
            Action::Macro { steps } => {
                self.macro_busy = true;
                play_macro(steps, sink, clock);
                self.macro_busy = false;
            }
            Action::HoldRepeat { inner, rate_ms } => {
                play_action_edge(inner, KEY_DOWN, sink);
                self.hold = Some((key_id, clock.now_ms(), *rate_ms));
            }
            other => play_action_edge(other, KEY_DOWN, sink),
        }
    }

    fn play_repeat(
        &mut self,
        key_id: KeyId,
        action: &Action,
        sink: &mut impl EventSink,
        clock: &mut impl Clock,
    ) {
        let Action::HoldRepeat { inner, rate_ms } = action else {
            return;
        };
        let Some((held_id, last_ms, held_rate)) = self.hold else {
            return;
        };
        if held_id != key_id {
            return;
        }
        let now = clock.now_ms();
        if now.saturating_sub(last_ms) >= u64::from(*rate_ms) {
            play_action_edge(inner, KEY_DOWN, sink);
            self.hold = Some((key_id, now, held_rate));
        }
    }

    fn play_release(&mut self, action: &Action, sink: &mut impl EventSink) {
        match action {
            Action::Macro { .. } => {}
            Action::HoldRepeat { inner, .. } => {
                play_action_edge(inner, KEY_UP, sink);
                self.hold = None;
            }
            other => play_action_edge(other, KEY_UP, sink),
        }
    }
}

fn analog_pressed(engine: &RemapEngine, id: KeyId) -> bool {
    match id {
        KeyId::AnalogLeft => engine.analog_left,
        KeyId::AnalogRight => engine.analog_right,
        KeyId::AnalogUp => engine.analog_up,
        KeyId::AnalogDown => engine.analog_down,
        _ => false,
    }
}

fn set_analog_pressed(engine: &mut RemapEngine, id: KeyId, value: bool) {
    match id {
        KeyId::AnalogLeft => engine.analog_left = value,
        KeyId::AnalogRight => engine.analog_right = value,
        KeyId::AnalogUp => engine.analog_up = value,
        KeyId::AnalogDown => engine.analog_down = value,
        _ => {}
    }
}

fn play_macro(
    steps: &[crate::types::MacroStep],
    sink: &mut impl EventSink,
    clock: &mut impl Clock,
) {
    for step in steps {
        match step.edge {
            Edge::Down => play_kind_edge(&step.kind, KEY_DOWN, sink),
            Edge::Up => play_kind_edge(&step.kind, KEY_UP, sink),
            Edge::Tap => {
                play_kind_edge(&step.kind, KEY_DOWN, sink);
                clock.sleep_ms(TAP_HOLD_MS);
                play_kind_edge(&step.kind, KEY_UP, sink);
            }
        }
        clock.sleep_ms(u64::from(step.delay_ms));
    }
}

fn play_kind_edge(kind: &MacroKind, value: i32, sink: &mut impl EventSink) {
    match kind {
        MacroKind::Key { key, modifiers } => {
            emit_combo(sink, *key, modifiers, value);
        }
        MacroKind::Mouse { target } => emit_mouse(sink, target, value),
    }
}

fn play_action_edge(action: &Action, value: i32, sink: &mut impl EventSink) {
    match action {
        Action::Key { key, modifiers } => emit_combo(sink, *key, modifiers, value),
        Action::Mouse { target } => emit_mouse(sink, target, value),
        Action::Macro { .. } | Action::HoldRepeat { .. } => {}
    }
}

fn emit_combo(
    sink: &mut impl EventSink,
    key: crate::types::KeyToken,
    modifiers: &[crate::types::Modifier],
    value: i32,
) {
    let key_code = token_to_evdev(key);
    if value == KEY_DOWN {
        for modifier in modifiers {
            emit_key(sink, modifier_to_evdev(*modifier), KEY_DOWN);
        }
        emit_key(sink, key_code, KEY_DOWN);
    } else {
        emit_key(sink, key_code, KEY_UP);
        for modifier in modifiers.iter().rev() {
            emit_key(sink, modifier_to_evdev(*modifier), KEY_UP);
        }
    }
}

fn emit_mouse(sink: &mut impl EventSink, target: &MouseTarget, value: i32) {
    match target {
        MouseTarget::Button { button } => {
            sink.emit(Emitted {
                keyboard: false,
                ev_type: EV_KEY,
                code: mouse_button_to_evdev(*button),
                value,
            });
        }
        MouseTarget::Scroll { scroll } => {
            if value != KEY_DOWN {
                return;
            }
            let wheel = match scroll {
                ScrollDir::Up => 1,
                ScrollDir::Down => -1,
            };
            sink.emit(Emitted {
                keyboard: false,
                ev_type: EV_REL,
                code: REL_WHEEL,
                value: wheel,
            });
        }
    }
}

fn emit_key(sink: &mut impl EventSink, code: u16, value: i32) {
    sink.emit(Emitted {
        keyboard: true,
        ev_type: EV_KEY,
        code,
        value,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{USB_PID_NAGA_PRO_1, USB_PID_TARTARUS_V2, USB_VID_RAZER};
    use crate::types::{Action, DeviceModel, KeyId, KeyToken, Lighting, LightingEffect, Profile};
    use std::collections::BTreeMap;

    struct FakeSink(Vec<Emitted>);
    impl EventSink for FakeSink {
        fn emit(&mut self, e: Emitted) {
            self.0.push(e);
        }
    }
    struct FakeClock {
        t: u64,
    }
    impl Clock for FakeClock {
        fn now_ms(&self) -> u64 {
            self.t
        }
        fn sleep_ms(&mut self, ms: u64) {
            self.t += ms;
        }
    }

    fn engine_with(kp01: Action) -> RemapEngine {
        let mut p = Profile {
            id: "t".into(),
            name: "t".into(),
            game: crate::types::GameId::Default,
            device_models: vec![DeviceModel::V2],
            bindings: BTreeMap::new(),
            lighting: Lighting {
                effect: LightingEffect::None,
                brightness: 80,
                color: None,
            },
            unknown_key_ids: false,
        };
        p.bindings.insert(KeyId::Kp01, kp01);
        let mut e = RemapEngine::new(DeviceModel::V2);
        e.apply_profile(&p);
        e
    }

    fn press_kp01() -> RawEvent {
        RawEvent {
            vid: USB_VID_RAZER,
            pid: USB_PID_TARTARUS_V2,
            ev_type: 1,
            code: 2,
            value: 1,
        }
    }

    #[test]
    fn kp01_emits_q() {
        let mut e = engine_with(Action::Key {
            key: KeyToken::Q,
            modifiers: vec![],
        });
        let mut s = FakeSink(vec![]);
        let mut c = FakeClock { t: 0 };
        e.handle(press_kp01(), &mut s, &mut c);
        e.handle(
            RawEvent {
                value: 0,
                ..press_kp01()
            },
            &mut s,
            &mut c,
        );
        let q = crate::keymap::token_to_evdev(KeyToken::Q);
        assert!(s
            .0
            .iter()
            .any(|x| x.keyboard && x.code == q && x.value == 1));
        assert!(s
            .0
            .iter()
            .any(|x| x.keyboard && x.code == q && x.value == 0));
    }

    #[test]
    fn combo_ctrl_c() {
        let mut e = engine_with(Action::Key {
            key: KeyToken::C,
            modifiers: vec![crate::types::Modifier::Ctrl],
        });
        let mut s = FakeSink(vec![]);
        let mut c = FakeClock { t: 0 };
        e.handle(press_kp01(), &mut s, &mut c);
        let codes: Vec<(u16, i32)> = s.0.iter().map(|x| (x.code, x.value)).collect();
        let ctrl = crate::keymap::modifier_to_evdev(crate::types::Modifier::Ctrl);
        let c_code = crate::keymap::token_to_evdev(KeyToken::C);
        assert_eq!(codes, vec![(ctrl, 1), (c_code, 1)]);
        s.0.clear();
        e.handle(
            RawEvent {
                value: 0,
                ..press_kp01()
            },
            &mut s,
            &mut c,
        );
        let up: Vec<(u16, i32)> = s.0.iter().map(|x| (x.code, x.value)).collect();
        assert_eq!(up, vec![(c_code, 0), (ctrl, 0)]);
    }

    #[test]
    fn macro_two_taps_respects_delay() {
        use crate::types::{Edge, MacroKind, MacroStep};
        let mut e = engine_with(Action::Macro {
            steps: vec![
                MacroStep {
                    kind: MacroKind::Key {
                        key: KeyToken::Num1,
                        modifiers: vec![],
                    },
                    edge: Edge::Tap,
                    delay_ms: 50,
                },
                MacroStep {
                    kind: MacroKind::Key {
                        key: KeyToken::Num2,
                        modifiers: vec![],
                    },
                    edge: Edge::Tap,
                    delay_ms: 0,
                },
            ],
        });
        let mut s = FakeSink(vec![]);
        let mut c = FakeClock { t: 0 };
        e.handle(press_kp01(), &mut s, &mut c);
        assert!(c.t >= 50);
        let one = crate::keymap::token_to_evdev(KeyToken::Num1);
        let two = crate::keymap::token_to_evdev(KeyToken::Num2);
        let first_two =
            s.0.iter()
                .position(|x| x.code == two && x.value == 1)
                .unwrap();
        let first_one =
            s.0.iter()
                .position(|x| x.code == one && x.value == 1)
                .unwrap();
        assert!(first_one < first_two);
    }

    #[test]
    fn hold_repeat_and_passthrough_and_naga() {
        let mut e = engine_with(Action::HoldRepeat {
            inner: Box::new(Action::Key {
                key: KeyToken::E,
                modifiers: vec![],
            }),
            rate_ms: 40,
        });
        let mut s = FakeSink(vec![]);
        let mut c = FakeClock { t: 0 };
        e.handle(press_kp01(), &mut s, &mut c);
        e.handle(
            RawEvent {
                value: 2,
                ..press_kp01()
            },
            &mut s,
            &mut c,
        );
        e.handle(
            RawEvent {
                value: 0,
                ..press_kp01()
            },
            &mut s,
            &mut c,
        );
        let ecode = crate::keymap::token_to_evdev(KeyToken::E);
        assert!(s.0.iter().any(|x| x.code == ecode && x.value == 1));
        assert!(s.0.iter().any(|x| x.code == ecode && x.value == 0));

        let mut s2 = FakeSink(vec![]);
        e.handle(
            RawEvent {
                vid: USB_VID_RAZER,
                pid: USB_PID_TARTARUS_V2,
                ev_type: 1,
                code: 3,
                value: 1,
            },
            &mut s2,
            &mut c,
        );
        assert!(s2.0.iter().any(|x| x.code == 3 && x.value == 1));

        assert!(!allow_grab(USB_VID_RAZER, USB_PID_NAGA_PRO_1));
        assert!(allow_grab(USB_VID_RAZER, USB_PID_TARTARUS_V2));
        assert!(allow_grab(
            USB_VID_RAZER,
            crate::constants::USB_PID_TARTARUS_PRO
        ));
    }

    #[test]
    fn naga_handle_emits_nothing() {
        let mut e = engine_with(Action::Key {
            key: KeyToken::Q,
            modifiers: vec![],
        });
        let mut s = FakeSink(vec![]);
        let mut c = FakeClock { t: 0 };
        e.handle(
            RawEvent {
                vid: USB_VID_RAZER,
                pid: USB_PID_NAGA_PRO_1,
                ev_type: 1,
                code: 2,
                value: 1,
            },
            &mut s,
            &mut c,
        );
        assert!(s.0.is_empty());
    }

    fn profile_with(model: DeviceModel, id: KeyId, action: Action) -> RemapEngine {
        let mut p = Profile {
            id: "t".into(),
            name: "t".into(),
            game: crate::types::GameId::Default,
            device_models: vec![model],
            bindings: BTreeMap::new(),
            lighting: Lighting {
                effect: LightingEffect::None,
                brightness: 80,
                color: None,
            },
            unknown_key_ids: false,
        };
        p.bindings.insert(id, action);
        let mut e = RemapEngine::new(model);
        e.apply_profile(&p);
        e
    }

    #[test]
    fn thumb_hat_eight_way_is_first_class() {
        let mut e = profile_with(
            DeviceModel::V2,
            KeyId::ThumbNe,
            Action::Key {
                key: KeyToken::W,
                modifiers: vec![],
            },
        );
        let mut s = FakeSink(vec![]);
        let mut c = FakeClock { t: 0 };
        let w = crate::keymap::token_to_evdev(KeyToken::W);
        e.handle(
            RawEvent {
                vid: USB_VID_RAZER,
                pid: USB_PID_TARTARUS_V2,
                ev_type: 3,
                code: 16,
                value: 1,
            },
            &mut s,
            &mut c,
        );
        e.handle(
            RawEvent {
                vid: USB_VID_RAZER,
                pid: USB_PID_TARTARUS_V2,
                ev_type: 3,
                code: 17,
                value: -1,
            },
            &mut s,
            &mut c,
        );
        assert!(s.0.iter().any(|x| x.code == w && x.value == 1));
        s.0.clear();
        e.handle(
            RawEvent {
                vid: USB_VID_RAZER,
                pid: USB_PID_TARTARUS_V2,
                ev_type: 3,
                code: 16,
                value: 0,
            },
            &mut s,
            &mut c,
        );
        e.handle(
            RawEvent {
                vid: USB_VID_RAZER,
                pid: USB_PID_TARTARUS_V2,
                ev_type: 3,
                code: 17,
                value: 0,
            },
            &mut s,
            &mut c,
        );
        assert!(s.0.iter().any(|x| x.code == w && x.value == 0));
    }

    #[test]
    fn analog_uses_press_and_release_ratios() {
        let mut e = profile_with(
            DeviceModel::Pro,
            KeyId::AnalogRight,
            Action::Key {
                key: KeyToken::D,
                modifiers: vec![],
            },
        );
        let mut s = FakeSink(vec![]);
        let mut c = FakeClock { t: 0 };
        let d = crate::keymap::token_to_evdev(KeyToken::D);
        let press = RawEvent {
            vid: USB_VID_RAZER,
            pid: crate::constants::USB_PID_TARTARUS_PRO,
            ev_type: 3,
            code: 0,
            value: 180,
        };
        e.handle(press, &mut s, &mut c);
        assert!(s.0.iter().any(|x| x.code == d && x.value == 1));
        s.0.clear();
        e.handle(
            RawEvent {
                value: 168,
                ..press
            },
            &mut s,
            &mut c,
        );
        assert!(s.0.is_empty());
        e.handle(
            RawEvent {
                value: 160,
                ..press
            },
            &mut s,
            &mut c,
        );
        assert!(s.0.iter().any(|x| x.code == d && x.value == 0));
    }

    #[test]
    fn mouse_action_emits_on_mouse_device() {
        use crate::types::MouseButton;
        let mut e = engine_with(Action::Mouse {
            target: crate::types::MouseTarget::Button {
                button: MouseButton::Left,
            },
        });
        let mut s = FakeSink(vec![]);
        let mut c = FakeClock { t: 0 };
        e.handle(press_kp01(), &mut s, &mut c);
        e.handle(
            RawEvent {
                value: 0,
                ..press_kp01()
            },
            &mut s,
            &mut c,
        );
        assert!(s
            .0
            .iter()
            .any(|x| !x.keyboard && x.code == 272 && x.value == 1));
        assert!(s
            .0
            .iter()
            .any(|x| !x.keyboard && x.code == 272 && x.value == 0));
    }

    #[test]
    fn hold_repeat_emits_extra_down_when_rate_elapsed() {
        let mut e = engine_with(Action::HoldRepeat {
            inner: Box::new(Action::Key {
                key: KeyToken::E,
                modifiers: vec![],
            }),
            rate_ms: 40,
        });
        let mut s = FakeSink(vec![]);
        let mut c = FakeClock { t: 0 };
        e.handle(press_kp01(), &mut s, &mut c);
        c.t = 40;
        e.handle(
            RawEvent {
                value: 2,
                ..press_kp01()
            },
            &mut s,
            &mut c,
        );
        let ecode = crate::keymap::token_to_evdev(KeyToken::E);
        let downs =
            s.0.iter()
                .filter(|x| x.code == ecode && x.value == 1)
                .count();
        assert_eq!(downs, 2);
    }
}
