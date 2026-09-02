use opentartarus_core::constants::{
    RECORD_TIMEOUT_MAX_MS, RECORD_TIMEOUT_MS, USB_PID_TARTARUS_PRO, USB_PID_TARTARUS_V2,
    USB_VID_RAZER,
};
use opentartarus_core::error::ErrorCode;
use opentartarus_core::ipc::Method;
use opentartarus_core::lighting::LightingClient;
use opentartarus_core::pack::{shipped_json, shipped_profile, SHIPPED_IDS};
use opentartarus_core::paths::Paths;
use opentartarus_core::record::{RecordOutcome, Recorder};
use opentartarus_core::remap::RemapEngine;
use opentartarus_core::store::{
    copy_on_apply, read_user_profile, revert_to_shipped, write_active_id, write_profile,
};
use opentartarus_core::types::{
    Action, DeviceModel, KeyId, KeyToken, Lighting, Modifier, MouseButton, MouseTarget, Profile,
    ScrollDir,
};
use serde_json::{json, Value};

const PROFILE_SOURCE_USER: &str = "user";
const PROFILE_SOURCE_SHIPPED: &str = "shipped";
const USB_ID_HEX_WIDTH: usize = 4;

pub struct DaemonState<L: LightingClient> {
    pub paths: Paths,
    pub engine: RemapEngine,
    pub recorder: Recorder,
    pub lighting: L,
    pub active_id: Option<String>,
    pub device_present: bool,
    pub model: Option<DeviceModel>,
    pub openrazer_available: bool,
    pub evdev_ok: bool,
    pub uinput_ok: bool,
    pub grab_conflict: Option<String>,
}

pub fn handle_request<L: LightingClient>(
    state: &mut DaemonState<L>,
    method: Method,
    params: Value,
    now_ms: u64,
) -> Result<Value, ErrorCode> {
    match method {
        Method::GetStatus => Ok(get_status(state)),
        Method::ListProfiles => list_profiles(state),
        Method::ApplyProfile => apply_profile(state, &params),
        Method::SetBinding => set_binding(state, &params),
        Method::ClearBinding => clear_binding(state, &params),
        Method::StartRecord => start_record(state, &params, now_ms),
        Method::StopRecord => Ok(stop_record(state)),
        Method::SubmitRecord => submit_record(state, &params),
        Method::SetLighting => set_lighting(state, &params),
        Method::RevertProfile => revert_profile(state, &params),
        Method::ShowWindow => Ok(json!({})),
        Method::QuitDaemon => Ok(json!({ "stopping": true })),
    }
}

fn usb_id_hex(id: u16) -> String {
    format!("{id:0width$x}", width = USB_ID_HEX_WIDTH)
}

fn get_status<L: LightingClient>(state: &DaemonState<L>) -> Value {
    let (model, vid, pid) = if state.device_present {
        let model = match state.model {
            Some(m) => json!(m),
            None => Value::Null,
        };
        let (vid, pid) = match state.model {
            Some(DeviceModel::V2) => (
                json!(usb_id_hex(USB_VID_RAZER)),
                json!(usb_id_hex(USB_PID_TARTARUS_V2)),
            ),
            Some(DeviceModel::Pro) => (
                json!(usb_id_hex(USB_VID_RAZER)),
                json!(usb_id_hex(USB_PID_TARTARUS_PRO)),
            ),
            None => (Value::Null, Value::Null),
        };
        (model, vid, pid)
    } else {
        (Value::Null, Value::Null, Value::Null)
    };
    let record = match state.recorder.session() {
        Some(session) => json!({
            "active": true,
            "key_id": session.key_id,
            "deadline_ms": session.deadline_ms,
        }),
        None => json!({
            "active": false,
            "key_id": null,
            "deadline_ms": null,
        }),
    };
    json!({
        "device": {
            "present": state.device_present,
            "model": model,
            "vid": vid,
            "pid": pid,
        },
        "openrazer": { "available": state.openrazer_available },
        "active_profile_id": state.active_id,
        "record": record,
        "permissions": {
            "uinput": state.uinput_ok,
            "evdev": state.evdev_ok,
        },
        "grab_conflict": state.grab_conflict,
    })
}

fn user_profile_exists(paths: &Paths, id: &str) -> bool {
    paths.profiles_dir.join(format!("{id}.json")).exists()
}

fn list_profiles<L: LightingClient>(state: &DaemonState<L>) -> Result<Value, ErrorCode> {
    let mut profiles = Vec::with_capacity(SHIPPED_IDS.len());
    for id in SHIPPED_IDS {
        let user_exists = user_profile_exists(&state.paths, id);
        let profile = if user_exists {
            read_user_profile(&state.paths, id)?
        } else {
            shipped_profile(id)?
        };
        profiles.push(json!({
            "id": profile.id,
            "name": profile.name,
            "game": profile.game,
            "source": if user_exists { PROFILE_SOURCE_USER } else { PROFILE_SOURCE_SHIPPED },
            "is_active": state.active_id.as_deref() == Some(id),
            "can_revert": shipped_json(id).is_some(),
        }));
    }
    Ok(json!({ "profiles": profiles }))
}

fn require_string(params: &Value, key: &str) -> Result<String, ErrorCode> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or(ErrorCode::InvalidProfile)
}

fn parse_key_id(value: &Value) -> Result<KeyId, ErrorCode> {
    serde_json::from_value(value.clone()).map_err(|_| ErrorCode::UnknownKey)
}

fn param_key_id(params: &Value) -> Result<KeyId, ErrorCode> {
    parse_key_id(params.get("key_id").unwrap_or(&Value::Null))
}

fn apply_lighting_soft<L: LightingClient>(
    state: &mut DaemonState<L>,
    lighting: &Lighting,
) -> Result<(), ErrorCode> {
    match state.lighting.apply(lighting) {
        Ok(()) => Ok(()),
        Err(ErrorCode::Lighting) => {
            state.openrazer_available = false;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn apply_if_active<L: LightingClient>(
    state: &mut DaemonState<L>,
    profile: &Profile,
    apply_lights: bool,
) -> Result<(), ErrorCode> {
    if state.active_id.as_deref() != Some(profile.id.as_str()) {
        return Ok(());
    }
    state.engine.apply_profile(profile);
    if apply_lights {
        apply_lighting_soft(state, &profile.lighting)?;
    }
    Ok(())
}

fn load_user_copy<L: LightingClient>(
    state: &DaemonState<L>,
    id: &str,
) -> Result<Profile, ErrorCode> {
    let raw = shipped_json(id).ok_or(ErrorCode::NotFound)?;
    copy_on_apply(&state.paths, raw, id)
}

fn persist_profile<L: LightingClient>(
    state: &mut DaemonState<L>,
    profile: &Profile,
    apply_lights: bool,
) -> Result<(), ErrorCode> {
    profile.validate()?;
    write_profile(&state.paths, profile)?;
    apply_if_active(state, profile, apply_lights)
}

fn apply_profile<L: LightingClient>(
    state: &mut DaemonState<L>,
    params: &Value,
) -> Result<Value, ErrorCode> {
    let id = require_string(params, "id")?;
    let raw = shipped_json(&id).ok_or(ErrorCode::NotFound)?;
    let profile = copy_on_apply(&state.paths, raw, &id)?;
    profile.validate()?;
    state.engine.apply_profile(&profile);
    write_active_id(&state.paths, &id)?;
    state.active_id = Some(id.clone());
    apply_lighting_soft(state, &profile.lighting)?;
    Ok(json!({ "id": id }))
}

fn set_binding<L: LightingClient>(
    state: &mut DaemonState<L>,
    params: &Value,
) -> Result<Value, ErrorCode> {
    let profile_id = require_string(params, "profile_id")?;
    let key_id = param_key_id(params)?;
    let action: Action =
        serde_json::from_value(params.get("action").cloned().unwrap_or(Value::Null))
            .map_err(|_| ErrorCode::InvalidProfile)?;
    let mut profile = load_user_copy(state, &profile_id)?;
    profile.bindings.insert(key_id, action);
    persist_profile(state, &profile, false)?;
    Ok(json!({ "id": profile_id }))
}

fn clear_binding<L: LightingClient>(
    state: &mut DaemonState<L>,
    params: &Value,
) -> Result<Value, ErrorCode> {
    let profile_id = require_string(params, "profile_id")?;
    let key_id = param_key_id(params)?;
    let mut profile = load_user_copy(state, &profile_id)?;
    profile.bindings.remove(&key_id);
    persist_profile(state, &profile, false)?;
    Ok(json!({ "id": profile_id }))
}

fn start_record<L: LightingClient>(
    state: &mut DaemonState<L>,
    params: &Value,
    now_ms: u64,
) -> Result<Value, ErrorCode> {
    let key_id = param_key_id(params)?;
    let timeout_ms = params.get("timeout_ms").and_then(Value::as_u64);
    let timeout = timeout_ms
        .unwrap_or(RECORD_TIMEOUT_MS)
        .min(RECORD_TIMEOUT_MAX_MS);
    state.recorder.start(key_id, timeout_ms, now_ms)?;
    Ok(json!({
        "key_id": key_id,
        "timeout_ms": timeout,
    }))
}

fn stop_record<L: LightingClient>(state: &mut DaemonState<L>) -> Value {
    let cancelled = state.recorder.stop();
    json!({ "cancelled": cancelled })
}

fn submit_record<L: LightingClient>(
    state: &mut DaemonState<L>,
    params: &Value,
) -> Result<Value, ErrorCode> {
    let profile_id = state.active_id.clone().ok_or(ErrorCode::NotFound)?;
    let outcome = if let Some(key) = params.get("key") {
        let key: KeyToken =
            serde_json::from_value(key.clone()).map_err(|_| ErrorCode::InvalidProfile)?;
        let modifiers: Vec<Modifier> = match params.get("modifiers") {
            Some(v) => serde_json::from_value(v.clone()).map_err(|_| ErrorCode::InvalidProfile)?,
            None => Vec::new(),
        };
        state.recorder.submit_key(key, modifiers)?
    } else if let Some(button) = params.get("button") {
        let button: MouseButton =
            serde_json::from_value(button.clone()).map_err(|_| ErrorCode::InvalidProfile)?;
        state
            .recorder
            .submit_mouse(MouseTarget::Button { button })?
    } else if let Some(scroll) = params.get("scroll") {
        let scroll: ScrollDir =
            serde_json::from_value(scroll.clone()).map_err(|_| ErrorCode::InvalidProfile)?;
        state
            .recorder
            .submit_mouse(MouseTarget::Scroll { scroll })?
    } else {
        return Err(ErrorCode::InvalidProfile);
    };
    match outcome {
        RecordOutcome::Recorded { key_id, action } => {
            let mut profile = load_user_copy(state, &profile_id)?;
            profile.bindings.insert(key_id, action.clone());
            persist_profile(state, &profile, false)?;
            Ok(json!({
                "key_id": key_id,
                "action": action,
            }))
        }
        RecordOutcome::Cancelled { .. } => Ok(json!({})),
    }
}

fn set_lighting<L: LightingClient>(
    state: &mut DaemonState<L>,
    params: &Value,
) -> Result<Value, ErrorCode> {
    let profile_id = require_string(params, "profile_id")?;
    let lighting: Lighting =
        serde_json::from_value(params.get("lighting").cloned().unwrap_or(Value::Null))
            .map_err(|_| ErrorCode::InvalidProfile)?;
    let mut profile = load_user_copy(state, &profile_id)?;
    profile.lighting = lighting;
    persist_profile(state, &profile, true)?;
    Ok(json!({ "id": profile_id }))
}

fn revert_profile<L: LightingClient>(
    state: &mut DaemonState<L>,
    params: &Value,
) -> Result<Value, ErrorCode> {
    let id = require_string(params, "id")?;
    let raw = shipped_json(&id).ok_or(ErrorCode::NotFound)?;
    let profile = revert_to_shipped(&state.paths, raw, &id)?;
    profile.validate()?;
    apply_if_active(state, &profile, true)?;
    Ok(json!({ "id": id }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentartarus_core::ipc::Method;
    use opentartarus_core::lighting::RecordingLighting;
    use opentartarus_core::pack::shipped_json;
    use opentartarus_core::paths::Paths;
    use opentartarus_core::record::Recorder;
    use opentartarus_core::remap::RemapEngine;
    use opentartarus_core::types::DeviceModel;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn state() -> DaemonState<RecordingLighting> {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("opentartarus-h-{n}"));
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
    fn apply_set_binding_and_status() {
        let mut st = state();
        let v = handle_request(
            &mut st,
            Method::ApplyProfile,
            json!({"id":"league-of-legends"}),
            0,
        )
        .unwrap();
        assert_eq!(v["id"], "league-of-legends");
        assert_eq!(st.active_id.as_deref(), Some("league-of-legends"));
        handle_request(
            &mut st,
            Method::SetBinding,
            json!({"profile_id":"league-of-legends","key_id":"kp01","action":{"type":"key","key":"c","modifiers":["ctrl"]}}),
            1,
        )
        .unwrap();
        let s = handle_request(&mut st, Method::GetStatus, json!({}), 2).unwrap();
        assert_eq!(s["active_profile_id"], "league-of-legends");
        assert_eq!(s["device"]["model"], "v2");
    }

    #[test]
    fn record_submit_and_stop() {
        let mut st = state();
        handle_request(&mut st, Method::ApplyProfile, json!({"id":"default"}), 0).unwrap();
        handle_request(
            &mut st,
            Method::StartRecord,
            json!({"key_id":"kp01","timeout_ms":8000}),
            0,
        )
        .unwrap();
        handle_request(
            &mut st,
            Method::SubmitRecord,
            json!({"key":"c","modifiers":["ctrl"]}),
            1,
        )
        .unwrap();
        assert!(!st.recorder.is_active());
        handle_request(&mut st, Method::StartRecord, json!({"key_id":"kp02"}), 2).unwrap();
        let stop = handle_request(&mut st, Method::StopRecord, json!({}), 3).unwrap();
        assert_eq!(stop["cancelled"], true);
    }

    #[test]
    fn get_status_matches_spec_shape() {
        let mut st = state();
        let s = handle_request(&mut st, Method::GetStatus, json!({}), 0).unwrap();
        assert_eq!(s["device"]["present"], true);
        assert_eq!(s["device"]["model"], "v2");
        assert_eq!(s["device"]["vid"], "1532");
        assert_eq!(s["device"]["pid"], "022b");
        assert_eq!(s["openrazer"]["available"], true);
        assert_eq!(s["active_profile_id"], serde_json::Value::Null);
        assert_eq!(s["record"]["active"], false);
        assert_eq!(s["record"]["key_id"], serde_json::Value::Null);
        assert_eq!(s["record"]["deadline_ms"], serde_json::Value::Null);
        assert_eq!(s["permissions"]["uinput"], true);
        assert_eq!(s["permissions"]["evdev"], true);
        assert_eq!(s["grab_conflict"], serde_json::Value::Null);
        let _ = shipped_json("default");
        st.model = Some(DeviceModel::Pro);
        let pro = handle_request(&mut st, Method::GetStatus, json!({}), 1).unwrap();
        assert_eq!(pro["device"]["pid"], "0244");
        st.device_present = false;
        let gone = handle_request(&mut st, Method::GetStatus, json!({}), 2).unwrap();
        assert_eq!(gone["device"]["present"], false);
        assert_eq!(gone["device"]["model"], serde_json::Value::Null);
        assert_eq!(gone["device"]["vid"], serde_json::Value::Null);
        assert_eq!(gone["device"]["pid"], serde_json::Value::Null);
    }

    #[test]
    fn list_profiles_shipped_order_and_user_source() {
        let mut st = state();
        handle_request(&mut st, Method::ApplyProfile, json!({"id": "default"}), 0).unwrap();
        let v = handle_request(&mut st, Method::ListProfiles, json!({}), 1).unwrap();
        let rows = v["profiles"].as_array().unwrap();
        let ids: Vec<&str> = rows.iter().map(|r| r["id"].as_str().unwrap()).collect();
        assert_eq!(
            ids,
            vec![
                "default",
                "league-of-legends",
                "dota-2",
                "world-of-warcraft",
                "final-fantasy-xiv",
                "path-of-exile"
            ]
        );
        assert_eq!(rows[0]["source"], "user");
        assert_eq!(rows[0]["is_active"], true);
        assert_eq!(rows[0]["can_revert"], true);
        assert_eq!(rows[1]["source"], "shipped");
        assert_eq!(rows[1]["can_revert"], true);
        assert_eq!(rows[1]["is_active"], false);
    }

    #[test]
    fn unknown_key_and_missing_profile() {
        use opentartarus_core::error::ErrorCode;
        let mut st = state();
        let err = handle_request(
            &mut st,
            Method::SetBinding,
            json!({"profile_id":"league-of-legends","key_id":"kp99","action":{"type":"key","key":"c","modifiers":[]}}),
            0,
        )
        .unwrap_err();
        assert_eq!(err, ErrorCode::UnknownKey);
        let err = handle_request(
            &mut st,
            Method::ClearBinding,
            json!({"profile_id":"no-such-profile","key_id":"kp01"}),
            1,
        )
        .unwrap_err();
        assert_eq!(err, ErrorCode::NotFound);
        let err =
            handle_request(&mut st, Method::ApplyProfile, json!({"id":"nope"}), 2).unwrap_err();
        assert_eq!(err, ErrorCode::NotFound);
    }

    #[test]
    fn lighting_error_does_not_fail_apply() {
        let mut st = state();
        st.lighting.available = false;
        let v = handle_request(&mut st, Method::ApplyProfile, json!({"id":"default"}), 0).unwrap();
        assert_eq!(v["id"], "default");
        assert_eq!(st.openrazer_available, false);
    }

    #[test]
    fn stop_record_without_session_is_ok() {
        let mut st = state();
        let stop = handle_request(&mut st, Method::StopRecord, json!({}), 0).unwrap();
        assert_eq!(stop["cancelled"], false);
    }

    #[test]
    fn start_record_result_includes_timeout_ms() {
        let mut st = state();
        handle_request(&mut st, Method::ApplyProfile, json!({"id":"default"}), 0).unwrap();
        let started = handle_request(
            &mut st,
            Method::StartRecord,
            json!({"key_id":"kp01","timeout_ms":8000}),
            100,
        )
        .unwrap();
        assert_eq!(started["key_id"], "kp01");
        assert_eq!(started["timeout_ms"], 8000);
        let s = handle_request(&mut st, Method::GetStatus, json!({}), 101).unwrap();
        assert_eq!(s["record"]["active"], true);
        assert_eq!(s["record"]["key_id"], "kp01");
        assert_eq!(s["record"]["deadline_ms"], 8100);
    }

    #[test]
    fn revert_show_quit_and_set_lighting() {
        let mut st = state();
        handle_request(&mut st, Method::ApplyProfile, json!({"id":"default"}), 0).unwrap();
        handle_request(
            &mut st,
            Method::SetLighting,
            json!({"profile_id":"default","lighting":{"effect":"spectrum","brightness":60}}),
            1,
        )
        .unwrap();
        let reverted =
            handle_request(&mut st, Method::RevertProfile, json!({"id":"default"}), 2).unwrap();
        assert_eq!(reverted["id"], "default");
        let shown = handle_request(&mut st, Method::ShowWindow, json!({}), 3).unwrap();
        assert_eq!(shown, json!({}));
        let quit = handle_request(&mut st, Method::QuitDaemon, json!({}), 4).unwrap();
        assert_eq!(quit["stopping"], true);
    }

    #[test]
    fn submit_record_persists_user_binding() {
        let mut st = state();
        handle_request(&mut st, Method::ApplyProfile, json!({"id":"default"}), 0).unwrap();
        handle_request(
            &mut st,
            Method::StartRecord,
            json!({"key_id":"kp01","timeout_ms":8000}),
            0,
        )
        .unwrap();
        handle_request(
            &mut st,
            Method::SubmitRecord,
            json!({"key":"c","modifiers":["ctrl"]}),
            1,
        )
        .unwrap();
        let profile = read_user_profile(&st.paths, "default").unwrap();
        assert!(matches!(
            profile.bindings.get(&KeyId::Kp01),
            Some(Action::Key { key: KeyToken::C, modifiers }) if modifiers == &[Modifier::Ctrl]
        ));
    }
}
