use crate::error::ErrorCode;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnvelopeType {
    Req,
    Res,
    Event,
}

macro_rules! envelope_tag {
    ($name:ident, $str:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name;

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str($str)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                if s == $str {
                    Ok($name)
                } else {
                    Err(serde::de::Error::custom(format!(
                        "expected envelope type {expected}",
                        expected = $str
                    )))
                }
            }
        }
    };
}

envelope_tag!(ReqTag, "req");
envelope_tag!(ResTag, "res");
envelope_tag!(EventTag, "event");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Method {
    GetStatus,
    ListProfiles,
    ApplyProfile,
    SetBinding,
    ClearBinding,
    StartRecord,
    StopRecord,
    SubmitRecord,
    SetLighting,
    RevertProfile,
    ShowWindow,
    QuitDaemon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EventMethod {
    DeviceChanged,
    Recorded,
    RecordCancelled,
    Error,
    ProfileApplied,
    FocusWindow,
    DaemonStopping,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireError {
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestMsg {
    #[serde(rename = "type")]
    pub r#type: ReqTag,
    pub id: String,
    pub method: Method,
    pub params: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseMsg {
    #[serde(rename = "type")]
    pub r#type: ResTag,
    pub id: String,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<WireError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventMsg {
    #[serde(rename = "type")]
    pub r#type: EventTag,
    pub method: EventMethod,
    pub params: Value,
}

pub fn parse_request(v: &Value) -> Result<RequestMsg, ErrorCode> {
    serde_json::from_value(v.clone()).map_err(map_request_error)
}

fn map_request_error(err: serde_json::Error) -> ErrorCode {
    if err.to_string().contains("unknown variant") {
        ErrorCode::NotFound
    } else {
        ErrorCode::InvalidProfile
    }
}

#[cfg(test)]
mod ipc_tests {
    use super::{parse_request, Method};
    use crate::error::ErrorCode;

    #[test]
    fn parses_get_status() {
        let v = serde_json::json!({"type":"req","id":"u1","method":"GetStatus","params":{}});
        let r = parse_request(&v).unwrap();
        assert_eq!(r.method, Method::GetStatus);
        assert_eq!(r.id, "u1");
    }

    #[test]
    fn unknown_method_is_not_found() {
        let v = serde_json::json!({"type":"req","id":"u1","method":"Explode","params":{}});
        assert_eq!(parse_request(&v).unwrap_err(), ErrorCode::NotFound);
    }
}
