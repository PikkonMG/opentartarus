use opentartarus_core::constants::{
    USB_PID_TARTARUS_PRO, USB_PID_TARTARUS_V2, USB_VID_RAZER,
};
use opentartarus_core::error::ErrorCode;
use opentartarus_core::ipc::{EventMethod, Method};
use opentartarus_core::keymap::{mouse_button_to_evdev, EV_KEY, EV_REL, REL_WHEEL};
use opentartarus_core::lighting::LightingClient;
use opentartarus_core::pack::SHIPPED_IDS;
use opentartarus_core::paths::Paths;
use opentartarus_core::record::RecordReason;
use opentartarus_core::remap::{Clock, RawEvent, RemapEngine};
use opentartarus_core::store::read_active_id;
use opentartarus_core::types::{DeviceModel, MouseButton, ScrollDir};
use opentartarus_daemon::device::{
    enumerate_tartarus, grab_conflict_message, pick_first, Detected,
};
use opentartarus_daemon::handler::{handle_request, DaemonState};
use opentartarus_daemon::openrazer::OpenRazerClient;
use opentartarus_daemon::perms::{probe_evdev_readable, probe_uinput};
use opentartarus_daemon::playback::{
    commit_if_engine_replaced, remap_physical_event, tick_engine, EngineEpoch,
};
use opentartarus_daemon::server::{accept_loop, bind_exclusive, emit_event};
use opentartarus_daemon::spawn_ui::UiSupervisor;
use opentartarus_daemon::tray::{
    spawn_tray, tray_name_from_applied_params, tray_name_from_profile_id, TrayCmd,
};
use opentartarus_daemon::uinput_sink::{token_from_evdev, UinputSink};
use opentartarus_daemon::{log, tray};
use serde_json::json;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;

const DEFAULT_PROFILE_NAME: &str = "Default";
const POLL_INTERVAL_MS: u64 = 250;
const POLL_INTERVAL: Duration = Duration::from_millis(POLL_INTERVAL_MS);
const TICK_INTERVAL_MS: u64 = 10;
const TICK_INTERVAL: Duration = Duration::from_millis(TICK_INTERVAL_MS);
const KEY_DOWN: i32 = 1;
const EV_SYN: u16 = 0;
const ALREADY_RUNNING_EXIT: i32 = 1;
const SUCCESS_EXIT: i32 = 0;

struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        now_ms()
    }

    fn sleep_ms(&mut self, ms: u64) {
        if ms > 0 {
            std::thread::sleep(Duration::from_millis(ms));
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn active_profile_name<L: LightingClient>(state: &DaemonState<L>) -> String {
    state
        .active_id
        .as_deref()
        .map(tray_name_from_profile_id)
        .unwrap_or_else(|| DEFAULT_PROFILE_NAME.to_string())
}

const USB_ID_HEX_WIDTH: usize = 4;

fn usb_id_hex(id: u16) -> String {
    format!("{id:0width$x}", width = USB_ID_HEX_WIDTH)
}

fn device_event_params<L: LightingClient>(state: &DaemonState<L>) -> serde_json::Value {
    if !state.device_present {
        return json!({ "present": false, "model": null, "vid": null, "pid": null });
    }
    match state.model {
        Some(DeviceModel::V2) => json!({
            "present": true,
            "model": DeviceModel::V2,
            "vid": usb_id_hex(USB_VID_RAZER),
            "pid": usb_id_hex(USB_PID_TARTARUS_V2),
        }),
        Some(DeviceModel::Pro) => json!({
            "present": true,
            "model": DeviceModel::Pro,
            "vid": usb_id_hex(USB_VID_RAZER),
            "pid": usb_id_hex(USB_PID_TARTARUS_PRO),
        }),
        None => json!({ "present": true, "model": null, "vid": null, "pid": null }),
    }
}

#[tokio::main]
async fn main() {
    let code = match run().await {
        Ok(()) => SUCCESS_EXIT,
        Err(ErrorCode::AlreadyRunning) => ALREADY_RUNNING_EXIT,
        Err(_) => ALREADY_RUNNING_EXIT,
    };
    std::process::exit(code);
}

async fn run() -> Result<(), ErrorCode> {
    let paths = Paths::from_env();
    log::init(&paths.log_file);

    let listener = match bind_exclusive(&paths.socket) {
        Ok(listener) => listener,
        Err(err) if err.kind() == ErrorKind::AddrInUse => {
            log::write(&ErrorCode::AlreadyRunning.log_line(Some(&err.to_string())));
            return Err(ErrorCode::AlreadyRunning);
        }
        Err(err) => {
            log::write(&ErrorCode::Io.log_line(Some(&err.to_string())));
            return Err(ErrorCode::Io);
        }
    };

    let mut uinput_ok = probe_uinput();
    let sink = if uinput_ok {
        match UinputSink::open() {
            Ok(sink) => Some(sink),
            Err(err) => {
                log::write(&err.log_line(None));
                uinput_ok = false;
                None
            }
        }
    } else {
        None
    };

    let found = pick_first(enumerate_tartarus());
    let (device_present, model, evdev_ok, grab_conflict, vid, pid) = match &found {
        Some(detected) => {
            let evdev_ok = detected.nodes.iter().all(|path| probe_evdev_readable(path));
            (
                true,
                Some(detected.model),
                evdev_ok,
                None,
                Some(detected.vid),
                Some(detected.pid),
            )
        }
        None => (false, None, true, None, None, None),
    };

    let lighting = OpenRazerClient::new(vid, pid);
    let openrazer_available = lighting.available();
    let mut state = DaemonState {
        paths: paths.clone(),
        engine: RemapEngine::new(model.unwrap_or(DeviceModel::V2)),
        recorder: Default::default(),
        lighting,
        active_id: None,
        device_present,
        model,
        openrazer_available,
        evdev_ok,
        uinput_ok,
        grab_conflict,
    };

    if let Ok(Some(id)) = read_active_id(&paths) {
        if SHIPPED_IDS.contains(&id.as_str()) {
            let _ = handle_request(
                &mut state,
                Method::ApplyProfile,
                json!({ "id": id }),
                now_ms(),
            );
        }
    }

    let profile_name = active_profile_name(&state);
    let state = Arc::new(Mutex::new(state));
    let sink = Arc::new(Mutex::new(sink));
    let quit = Arc::new(AtomicBool::new(false));
    let clients = Arc::new(AtomicUsize::new(0));
    let epoch = Arc::new(EngineEpoch::new());
    let (event_tx, _) = broadcast::channel(64);
    let mut events_rx = event_tx.subscribe();
    let (tray_tx, mut tray_rx) = tokio::sync::mpsc::unbounded_channel();

    let tray_icon = tray::OpenTartarusTray::new(profile_name, tray_tx);
    let tray_handle = spawn_tray(tray_icon).await;

    {
        let listener_state = Arc::clone(&state);
        let events = event_tx.clone();
        let quit_flag = Arc::clone(&quit);
        let client_count = Arc::clone(&clients);
        let epoch = Arc::clone(&epoch);
        tokio::spawn(async move {
            accept_loop(
                listener,
                listener_state,
                events,
                quit_flag,
                client_count,
                epoch,
            )
            .await;
        });
    }

    {
        let device_state = Arc::clone(&state);
        let device_sink = Arc::clone(&sink);
        let events = event_tx.clone();
        let quit_flag = Arc::clone(&quit);
        let epoch = Arc::clone(&epoch);
        std::thread::spawn(move || {
            device_loop(device_state, device_sink, events, quit_flag, epoch);
        });
    }

    let mut ui = UiSupervisor::new();
    loop {
        tokio::select! {
            cmd = tray_rx.recv() => {
                match cmd {
                    Some(TrayCmd::Quit) | None => {
                        request_quit(&state, &event_tx, &quit);
                        break;
                    }
                    Some(TrayCmd::Open) => {
                        let connected = clients.load(Ordering::SeqCst) > 0;
                        if ui.ensure_open(connected) {
                            emit_event(&event_tx, EventMethod::FocusWindow, json!({}));
                        }
                    }
                    Some(TrayCmd::ApplyProfile(id)) => {
                        let applied = {
                            let mut st = state.lock().expect("daemon state");
                            let result = handle_request(
                                &mut st,
                                Method::ApplyProfile,
                                json!({ "id": id }),
                                now_ms(),
                            );
                            commit_if_engine_replaced(
                                result.is_ok(),
                                Some(Method::ApplyProfile),
                                &epoch,
                            );
                            result.is_ok()
                        };
                        if applied {
                            emit_event(
                                &event_tx,
                                EventMethod::ProfileApplied,
                                json!({ "id": id }),
                            );
                        }
                    }
                }
            }
            event = events_rx.recv() => {
                match event {
                    Ok(msg) if msg.method == EventMethod::ProfileApplied => {
                        if let Some(name) = tray_name_from_applied_params(&msg.params) {
                            if let Some(handle) = &tray_handle {
                                handle
                                    .update(|tray_state| tray_state.profile_name = name)
                                    .await;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        let name = {
                            let st = state.lock().expect("daemon state");
                            active_profile_name(&st)
                        };
                        if let Some(handle) = &tray_handle {
                            handle
                                .update(|tray_state| tray_state.profile_name = name)
                                .await;
                        }
                    }
                    Ok(_) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            _ = tokio::signal::ctrl_c() => {
                request_quit(&state, &event_tx, &quit);
                break;
            }
        }
        if quit.load(Ordering::SeqCst) {
            break;
        }
    }

    quit.store(true, Ordering::SeqCst);
    emit_event(&event_tx, EventMethod::DaemonStopping, json!({}));
    {
        let mut held = sink.lock().expect("uinput sink");
        *held = None;
    }
    let _ = std::fs::remove_file(&paths.socket);
    Ok(())
}

fn request_quit<L: LightingClient>(
    state: &Mutex<DaemonState<L>>,
    events: &broadcast::Sender<opentartarus_core::ipc::EventMsg>,
    quit: &AtomicBool,
) {
    if let Ok(mut st) = state.lock() {
        let _ = handle_request(&mut st, Method::QuitDaemon, json!({}), now_ms());
    }
    emit_event(events, EventMethod::DaemonStopping, json!({}));
    quit.store(true, Ordering::SeqCst);
}

fn device_loop(
    state: Arc<Mutex<DaemonState<OpenRazerClient>>>,
    sink: Arc<Mutex<Option<UinputSink>>>,
    events: broadcast::Sender<opentartarus_core::ipc::EventMsg>,
    quit: Arc<AtomicBool>,
    epoch: Arc<EngineEpoch>,
) {
    let mut grabbed: Vec<evdev::Device> = Vec::new();
    let mut grabbed_ids: Option<(u16, u16, DeviceModel)> = None;
    let mut grabbed_nodes: Vec<PathBuf> = Vec::new();

    while !quit.load(Ordering::SeqCst) {
        poll_record_timeout(&state, &events);

        if grabbed.is_empty() {
            if let Some(detected) = pick_first(enumerate_tartarus()) {
                match grab_nodes(&detected.nodes) {
                    Ok(devices) => {
                        grabbed = devices;
                        grabbed_ids = Some((detected.vid, detected.pid, detected.model));
                        grabbed_nodes = detected.nodes.clone();
                        on_device_appeared(&state, &detected, &events, &epoch);
                    }
                    Err((code, os)) => {
                        log::write(&code.log_line(Some(&os)));
                        let mut st = state.lock().expect("daemon state");
                        st.device_present = true;
                        st.model = Some(detected.model);
                        st.grab_conflict = None;
                        if code == ErrorCode::GrabConflict {
                            st.evdev_ok = true;
                        } else {
                            st.evdev_ok = false;
                        }
                        emit_event(
                            &events,
                            EventMethod::Error,
                            json!({
                                "code": code.wire_name(),
                                "message": code.user_message(),
                            }),
                        );
                        emit_event(&events, EventMethod::DeviceChanged, device_event_params(&st));
                    }
                }
            }
            std::thread::sleep(POLL_INTERVAL);
            continue;
        }

        if !grabbed_nodes.iter().all(|path| path.exists()) {
            grabbed.clear();
            grabbed_ids = None;
            grabbed_nodes.clear();
            on_device_vanished(&state, &events);
            continue;
        }

        let Some((vid, pid, _)) = grabbed_ids else {
            grabbed.clear();
            continue;
        };

        let mut pending = Vec::new();
        let mut disconnect = false;
        for device in &mut grabbed {
            match device.fetch_events() {
                Ok(iter) => {
                    for ev in iter {
                        pending.push((ev.event_type().0, ev.code(), ev.value()));
                    }
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => {}
                Err(_) => {
                    disconnect = true;
                    break;
                }
            }
        }
        if disconnect {
            grabbed.clear();
            grabbed_ids = None;
            grabbed_nodes.clear();
            on_device_vanished(&state, &events);
            continue;
        }
        if pending.is_empty() {
            tick_remap(&state, &sink, &epoch);
            std::thread::sleep(idle_sleep(&state));
            continue;
        }
        for (ev_type, code, value) in pending {
            if ev_type == EV_SYN {
                continue;
            }
            handle_physical_event(
                &state,
                &sink,
                &events,
                &epoch,
                RawEvent {
                    vid,
                    pid,
                    ev_type,
                    code,
                    value,
                },
            );
        }
        tick_remap(&state, &sink, &epoch);
    }
}

fn tick_remap(
    state: &Mutex<DaemonState<OpenRazerClient>>,
    sink: &Mutex<Option<UinputSink>>,
    epoch: &EngineEpoch,
) {
    let mut clock = SystemClock;
    tick_engine(state, sink, epoch, &mut clock);
}

fn idle_sleep(state: &Mutex<DaemonState<OpenRazerClient>>) -> Duration {
    let st = state.lock().expect("daemon state");
    if st.engine.has_timed_work() {
        TICK_INTERVAL
    } else {
        POLL_INTERVAL
    }
}

fn grab_nodes(nodes: &[PathBuf]) -> Result<Vec<evdev::Device>, (ErrorCode, String)> {
    let mut devices = Vec::with_capacity(nodes.len());
    for path in nodes {
        let mut device = evdev::Device::open(path).map_err(|err| {
            (
                grab_conflict_message(&err.to_string()),
                err.to_string(),
            )
        })?;
        device.grab().map_err(|err| {
            (
                grab_conflict_message(&err.to_string()),
                err.to_string(),
            )
        })?;
        let _ = device.set_nonblocking(true);
        devices.push(device);
    }
    Ok(devices)
}

fn on_device_appeared(
    state: &Mutex<DaemonState<OpenRazerClient>>,
    detected: &Detected,
    events: &broadcast::Sender<opentartarus_core::ipc::EventMsg>,
    epoch: &EngineEpoch,
) {
    let mut st = state.lock().expect("daemon state");
    let evdev_ok = detected.nodes.iter().all(|path| probe_evdev_readable(path));
    st.device_present = true;
    st.model = Some(detected.model);
    st.evdev_ok = evdev_ok;
    st.grab_conflict = None;
    st.engine = RemapEngine::new(detected.model);
    update_lighting_usb(&mut st.lighting, Some((detected.vid, detected.pid)));
    let active = st.active_id.clone();
    if let Some(id) = active {
        let applied = handle_request(&mut st, Method::ApplyProfile, json!({ "id": id }), now_ms());
        commit_if_engine_replaced(applied.is_ok(), Some(Method::ApplyProfile), epoch);
        if applied.is_ok() {
            emit_event(events, EventMethod::ProfileApplied, json!({ "id": id }));
        }
    }
    emit_event(events, EventMethod::DeviceChanged, device_event_params(&st));
}

fn on_device_vanished(
    state: &Mutex<DaemonState<OpenRazerClient>>,
    events: &broadcast::Sender<opentartarus_core::ipc::EventMsg>,
) {
    let mut st = state.lock().expect("daemon state");
    if let Some(outcome) = st.recorder.on_disconnect() {
        if let opentartarus_core::record::RecordOutcome::Cancelled { reason } = outcome {
            emit_event(
                events,
                EventMethod::RecordCancelled,
                json!({ "reason": record_reason_wire(reason) }),
            );
        }
    }
    st.device_present = false;
    st.model = None;
    st.grab_conflict = None;
    update_lighting_usb(&mut st.lighting, None);
    emit_event(events, EventMethod::DeviceChanged, device_event_params(&st));
}

fn record_reason_wire(reason: RecordReason) -> &'static str {
    match reason {
        RecordReason::Timeout => "timeout",
        RecordReason::User => "user",
        RecordReason::Disconnected => "disconnected",
    }
}

fn poll_record_timeout<L: LightingClient>(
    state: &Mutex<DaemonState<L>>,
    events: &broadcast::Sender<opentartarus_core::ipc::EventMsg>,
) {
    let mut st = state.lock().expect("daemon state");
    if let Some(outcome) = st.recorder.on_timeout(now_ms()) {
        if let opentartarus_core::record::RecordOutcome::Cancelled { reason } = outcome {
            emit_event(
                events,
                EventMethod::RecordCancelled,
                json!({ "reason": record_reason_wire(reason) }),
            );
        }
    }
}

fn handle_physical_event<L: LightingClient>(
    state: &Mutex<DaemonState<L>>,
    sink: &Mutex<Option<UinputSink>>,
    events: &broadcast::Sender<opentartarus_core::ipc::EventMsg>,
    epoch: &EngineEpoch,
    ev: RawEvent,
) {
    {
        let mut st = state.lock().expect("daemon state");
        if st.recorder.is_active() && ev.value == KEY_DOWN {
            if let Some(params) = record_params_from_event(&ev) {
                match handle_request(&mut st, Method::SubmitRecord, params, now_ms()) {
                    Ok(result) => {
                        commit_if_engine_replaced(true, Some(Method::SubmitRecord), epoch);
                        emit_event(events, EventMethod::Recorded, result);
                        if let Some(id) = st.active_id.as_deref() {
                            emit_event(events, EventMethod::ProfileApplied, json!({ "id": id }));
                        }
                    }
                    Err(err) => {
                        log::write(&err.log_line(None));
                    }
                }
                return;
            }
        }
    }
    let mut clock = SystemClock;
    remap_physical_event(state, sink, epoch, ev, &mut clock);
}

fn record_params_from_event(ev: &RawEvent) -> Option<serde_json::Value> {
    if ev.ev_type == EV_REL && ev.code == REL_WHEEL {
        let scroll = if ev.value > 0 {
            ScrollDir::Up
        } else {
            ScrollDir::Down
        };
        return Some(json!({ "scroll": scroll }));
    }
    if ev.ev_type == EV_KEY {
        if let Some(button) = mouse_button_from_code(ev.code) {
            return Some(json!({ "button": button }));
        }
        if let Some(key) = token_from_evdev(ev.code) {
            return Some(json!({ "key": key, "modifiers": [] }));
        }
    }
    None
}

fn mouse_button_from_code(code: u16) -> Option<MouseButton> {
    for button in [
        MouseButton::Left,
        MouseButton::Right,
        MouseButton::Middle,
        MouseButton::Back,
        MouseButton::Forward,
    ] {
        if mouse_button_to_evdev(button) == code {
            return Some(button);
        }
    }
    None
}

fn update_lighting_usb(lighting: &mut OpenRazerClient, ids: Option<(u16, u16)>) {
    match ids {
        Some((vid, pid)) => lighting.set_usb(vid, pid),
        None => lighting.clear_usb(),
    }
}
