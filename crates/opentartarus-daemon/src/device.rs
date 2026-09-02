use opentartarus_core::types::DeviceModel;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Detected {
    pub model: DeviceModel,
    pub vid: u16,
    pub pid: u16,
    pub nodes: Vec<PathBuf>,
}

pub fn model_from_pid(pid: u16) -> Option<DeviceModel> {
    match pid {
        opentartarus_core::constants::USB_PID_TARTARUS_V2 => Some(DeviceModel::V2),
        opentartarus_core::constants::USB_PID_TARTARUS_PRO => Some(DeviceModel::Pro),
        _ => None,
    }
}

pub fn classify_input_id(vid: u16, pid: u16) -> Option<DeviceModel> {
    if !opentartarus_core::remap::allow_grab(vid, pid) {
        return None;
    }
    match pid {
        opentartarus_core::constants::USB_PID_TARTARUS_V2 => Some(DeviceModel::V2),
        opentartarus_core::constants::USB_PID_TARTARUS_PRO => Some(DeviceModel::Pro),
        _ => None,
    }
}

pub fn pick_first(found: Vec<Detected>) -> Option<Detected> {
    found.into_iter().next()
}

pub fn grab_conflict_message(os: &str) -> opentartarus_core::error::ErrorCode {
    if os.contains("EBUSY") || os.contains("Device or resource busy") {
        opentartarus_core::error::ErrorCode::GrabConflict
    } else {
        opentartarus_core::error::ErrorCode::Permission
    }
}

pub fn enumerate_tartarus() -> Vec<Detected> {
    let mut order: Vec<(u16, u16)> = Vec::new();
    let mut groups: HashMap<(u16, u16), Vec<PathBuf>> = HashMap::new();

    for (path, device) in evdev::enumerate() {
        let id = device.input_id();
        let vid = id.vendor();
        let pid = id.product();
        if !opentartarus_core::remap::allow_grab(vid, pid) {
            continue;
        }
        let key = (vid, pid);
        if !groups.contains_key(&key) {
            order.push(key);
        }
        groups.entry(key).or_default().push(path);
    }

    order
        .into_iter()
        .filter_map(|(vid, pid)| {
            let model = classify_input_id(vid, pid)?;
            Some(Detected {
                model,
                vid,
                pid,
                nodes: groups.remove(&(vid, pid)).unwrap_or_default(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentartarus_core::constants::{
        USB_PID_NAGA_PRO_1, USB_PID_TARTARUS_PRO, USB_PID_TARTARUS_V2, USB_VID_RAZER,
    };
    use opentartarus_core::remap::allow_grab;
    use opentartarus_core::types::DeviceModel;

    #[test]
    fn naga_is_never_classified() {
        assert!(classify_input_id(USB_VID_RAZER, USB_PID_NAGA_PRO_1).is_none());
        assert!(!allow_grab(USB_VID_RAZER, USB_PID_NAGA_PRO_1));
        assert_eq!(
            classify_input_id(USB_VID_RAZER, USB_PID_TARTARUS_V2),
            Some(DeviceModel::V2)
        );
        assert_eq!(
            classify_input_id(USB_VID_RAZER, USB_PID_TARTARUS_PRO),
            Some(DeviceModel::Pro)
        );
    }

    #[test]
    fn first_enumerated_wins() {
        let a = Detected {
            model: DeviceModel::Pro,
            vid: USB_VID_RAZER,
            pid: USB_PID_TARTARUS_PRO,
            nodes: vec![],
        };
        let b = Detected {
            model: DeviceModel::V2,
            vid: USB_VID_RAZER,
            pid: USB_PID_TARTARUS_V2,
            nodes: vec![],
        };
        assert_eq!(pick_first(vec![a, b]).unwrap().model, DeviceModel::Pro);
    }

    #[test]
    fn busy_is_grab_conflict() {
        assert_eq!(
            grab_conflict_message("EBUSY: Device or resource busy"),
            opentartarus_core::error::ErrorCode::GrabConflict
        );
    }
}
