use crate::handler::{handle_request, DaemonState};
use opentartarus_core::error::ErrorCode;
use opentartarus_core::ipc::{parse_request, ResTag, ResponseMsg, WireError};
use opentartarus_core::lighting::LightingClient;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn serve_forever() {}

pub fn dispatch_json<L: LightingClient>(state: &mut DaemonState<L>, bytes: &[u8]) -> Vec<u8> {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let value: Value = match serde_json::from_slice(bytes) {
        Ok(v) => v,
        Err(_) => return encode_error(generated_id(), ErrorCode::InvalidProfile),
    };
    match parse_request(&value) {
        Ok(req) => {
            let result = handle_request(state, req.method, req.params, now_ms);
            encode_response(req.id, result)
        }
        Err(code) => {
            let id = value
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .unwrap_or_else(generated_id);
            encode_error(id, code)
        }
    }
}

fn generated_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn encode_error(id: String, code: ErrorCode) -> Vec<u8> {
    encode_response(id, Err(code))
}

fn encode_response(id: String, result: Result<Value, ErrorCode>) -> Vec<u8> {
    let msg = match result {
        Ok(value) => ResponseMsg {
            r#type: ResTag,
            id,
            ok: true,
            result: Some(value),
            error: None,
        },
        Err(code) => ResponseMsg {
            r#type: ResTag,
            id,
            ok: false,
            result: None,
            error: Some(WireError {
                code,
                message: code.user_message().to_string(),
            }),
        },
    };
    serde_json::to_vec(&msg).expect("ResponseMsg is serializable")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::DaemonState;
    use opentartarus_core::lighting::RecordingLighting;
    use opentartarus_core::paths::Paths;
    use opentartarus_core::record::Recorder;
    use opentartarus_core::remap::RemapEngine;
    use opentartarus_core::types::DeviceModel;
    use serde_json::Value;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn state() -> DaemonState<RecordingLighting> {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("opentartarus-s-{n}"));
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
    fn dispatch_json_get_status_ok() {
        let mut st = state();
        let bytes = br#"{"type":"req","id":"u1","method":"GetStatus","params":{}}"#;
        let out = dispatch_json(&mut st, bytes);
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["type"], "res");
        assert_eq!(v["id"], "u1");
        assert_eq!(v["ok"], true);
        assert_eq!(v["result"]["device"]["pid"], "022b");
    }

    #[test]
    fn dispatch_json_unknown_method_is_not_found() {
        let mut st = state();
        let bytes = br#"{"type":"req","id":"u2","method":"Explode","params":{}}"#;
        let out = dispatch_json(&mut st, bytes);
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["ok"], false);
        assert_eq!(v["error"]["code"], "not_found");
    }
}
