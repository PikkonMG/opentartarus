use crate::handler::DaemonState;
use crate::uinput_sink::UinputSink;
use opentartarus_core::ipc::Method;
use opentartarus_core::lighting::LightingClient;
use opentartarus_core::remap::{Clock, Emitted, EventSink, RawEvent, RemapEngine};
use opentartarus_core::types::DeviceModel;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

pub struct EngineEpoch {
    value: AtomicU64,
}

impl EngineEpoch {
    pub fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    pub fn load(&self) -> u64 {
        self.value.load(Ordering::SeqCst)
    }

    pub fn bump(&self) {
        self.value.fetch_add(1, Ordering::SeqCst);
    }
}

impl Default for EngineEpoch {
    fn default() -> Self {
        Self::new()
    }
}

pub fn method_mutates_engine(method: Method) -> bool {
    matches!(
        method,
        Method::ApplyProfile
            | Method::SetBinding
            | Method::ClearBinding
            | Method::SetLighting
            | Method::RevertProfile
            | Method::SubmitRecord
    )
}

pub struct BriefLockSink<'a> {
    sink: &'a Mutex<Option<UinputSink>>,
}

impl<'a> BriefLockSink<'a> {
    pub fn new(sink: &'a Mutex<Option<UinputSink>>) -> Self {
        Self { sink }
    }
}

impl EventSink for BriefLockSink<'_> {
    fn emit(&mut self, e: Emitted) {
        if let Ok(mut held) = self.sink.lock() {
            if let Some(device) = held.as_mut() {
                device.emit(e);
            }
        }
    }
}

pub fn remap_physical_event<L, C>(
    state: &Mutex<DaemonState<L>>,
    sink: &Mutex<Option<UinputSink>>,
    epoch: &EngineEpoch,
    ev: RawEvent,
    clock: &mut C,
) where
    L: LightingClient,
    C: Clock,
{
    let epoch_at_take = epoch.load();
    let mut engine = {
        let mut st = state.lock().expect("daemon state");
        take_engine(&mut st)
    };
    let mut emit = BriefLockSink::new(sink);
    engine.handle(ev, &mut emit, clock);
    let mut st = state.lock().expect("daemon state");
    restore_engine_if_current(&mut st, engine, epoch_at_take, epoch);
}

fn take_engine<L: LightingClient>(state: &mut DaemonState<L>) -> RemapEngine {
    let model = state.model.unwrap_or(DeviceModel::V2);
    std::mem::replace(&mut state.engine, RemapEngine::new(model))
}

fn restore_engine_if_current<L: LightingClient>(
    state: &mut DaemonState<L>,
    engine: RemapEngine,
    epoch_at_take: u64,
    epoch: &EngineEpoch,
) {
    if epoch.load() == epoch_at_take {
        state.engine = engine;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::{handle_request, DaemonState};
    use opentartarus_core::constants::{USB_PID_TARTARUS_V2, USB_VID_RAZER};
    use opentartarus_core::ipc::Method;
    use opentartarus_core::keymap::EV_KEY;
    use opentartarus_core::lighting::RecordingLighting;
    use opentartarus_core::paths::Paths;
    use opentartarus_core::record::Recorder;
    use opentartarus_core::remap::RemapEngine;
    use opentartarus_core::types::DeviceModel;
    use serde_json::json;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    const KP01_NATIVE_CODE: u16 = 2;
    const KEY_DOWN: i32 = 1;

    struct UnlockProbeClock {
        state: Arc<Mutex<DaemonState<RecordingLighting>>>,
        sink: Arc<Mutex<Option<UinputSink>>>,
        state_free: AtomicBool,
        sink_free: AtomicBool,
    }

    impl Clock for UnlockProbeClock {
        fn now_ms(&self) -> u64 {
            0
        }

        fn sleep_ms(&mut self, _ms: u64) {
            self.state_free
                .store(self.state.try_lock().is_ok(), Ordering::SeqCst);
            self.sink_free
                .store(self.sink.try_lock().is_ok(), Ordering::SeqCst);
        }
    }

    fn state() -> DaemonState<RecordingLighting> {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("opentartarus-pb-{n}"));
        let paths = Paths::from_dirs(root.join("c"), root.join("r"), root.join("s"));
        DaemonState {
            paths,
            engine: RemapEngine::new(DeviceModel::V2),
            recorder: Recorder::default(),
            lighting: RecordingLighting {
                available: true,
                last: None,
            },
            active_id: None,
            device_present: true,
            model: Some(DeviceModel::V2),
            openrazer_available: true,
            evdev_ok: true,
            uinput_ok: true,
            grab_conflict: None,
        }
    }

    #[test]
    fn sleep_does_not_hold_state_or_sink_locks() {
        let mut st = state();
        handle_request(
            &mut st,
            Method::ApplyProfile,
            json!({ "id": "default" }),
            0,
        )
        .unwrap();
        handle_request(
            &mut st,
            Method::SetBinding,
            json!({
                "profile_id": "default",
                "key_id": "kp01",
                "action": {
                    "type": "macro",
                    "steps": [{
                        "kind": "key",
                        "key": "q",
                        "modifiers": [],
                        "edge": "down",
                        "delay_ms": 25
                    }]
                }
            }),
            0,
        )
        .unwrap();
        let state = Arc::new(Mutex::new(st));
        let sink = Arc::new(Mutex::new(None));
        let epoch = EngineEpoch::new();
        let mut clock = UnlockProbeClock {
            state: Arc::clone(&state),
            sink: Arc::clone(&sink),
            state_free: AtomicBool::new(false),
            sink_free: AtomicBool::new(false),
        };
        remap_physical_event(
            &state,
            &sink,
            &epoch,
            RawEvent {
                vid: USB_VID_RAZER,
                pid: USB_PID_TARTARUS_V2,
                ev_type: EV_KEY,
                code: KP01_NATIVE_CODE,
                value: KEY_DOWN,
            },
            &mut clock,
        );
        assert!(
            clock.state_free.load(Ordering::SeqCst),
            "state mutex must be free during Clock::sleep_ms"
        );
        assert!(
            clock.sink_free.load(Ordering::SeqCst),
            "uinput mutex must be free during Clock::sleep_ms"
        );
    }
}
