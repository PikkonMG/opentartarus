use opentartarus_core::error::ErrorCode;
use opentartarus_core::remap::allow_grab;
use opentartarus_core::types::DeviceModel;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const SCAN_INTERVAL_MISSING_MS: u64 = 1000;
pub const SCAN_INTERVAL_ACTIVE_MS: u64 = 250;
const HEX_RADIX: u32 = 16;
const EVENT_HANDLER_PREFIX: &str = "event";
const DEV_INPUT: &str = "/dev/input";
const PROC_INPUT_DEVICES: &str = "/proc/bus/input/devices";
const HID_DEVICES: &str = "/sys/bus/hid/devices";
const VENDOR_FIELD: &str = "Vendor=";
const PRODUCT_FIELD: &str = "Product=";
const HANDLERS_FIELD: &str = "Handlers=";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detected {
    pub model: DeviceModel,
    pub vid: u16,
    pub pid: u16,
    pub nodes: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectStatus {
    Missing,
    Permission,
    Present,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceUiState {
    pub present: bool,
    pub model: Option<DeviceModel>,
    pub evdev_ok: bool,
    pub grab_conflict: bool,
}

pub fn model_from_pid(pid: u16) -> Option<DeviceModel> {
    match pid {
        opentartarus_core::constants::USB_PID_TARTARUS_V2 => Some(DeviceModel::V2),
        opentartarus_core::constants::USB_PID_TARTARUS_PRO => Some(DeviceModel::Pro),
        _ => None,
    }
}

pub fn classify_input_id(vid: u16, pid: u16) -> Option<DeviceModel> {
    if !allow_grab(vid, pid) {
        return None;
    }
    model_from_pid(pid)
}

pub fn pick_first(found: Vec<Detected>) -> Option<Detected> {
    found.into_iter().next()
}

pub fn grab_conflict_message(os: &str) -> ErrorCode {
    if os.contains("EBUSY") || os.contains("Device or resource busy") {
        ErrorCode::GrabConflict
    } else {
        ErrorCode::Permission
    }
}

pub fn parse_hid_bus_id(name: &str) -> Option<(u16, u16)> {
    let mut parts = name.split(':');
    let _bus = parts.next()?;
    let vid = u16::from_str_radix(parts.next()?, HEX_RADIX).ok()?;
    let rest = parts.next()?;
    let pid_hex = rest.split('.').next()?;
    let pid = u16::from_str_radix(pid_hex, HEX_RADIX).ok()?;
    Some((vid, pid))
}

pub fn parse_proc_input_devices(text: &str) -> Vec<Detected> {
    let mut order: Vec<(u16, u16)> = Vec::new();
    let mut groups: HashMap<(u16, u16), Vec<PathBuf>> = HashMap::new();

    for block in text.split("\n\n") {
        let Some((vid, pid)) = parse_proc_ids(block) else {
            continue;
        };
        if !allow_grab(vid, pid) {
            continue;
        }
        let key = (vid, pid);
        if !groups.contains_key(&key) {
            order.push(key);
        }
        groups.entry(key).or_default().extend(parse_proc_event_nodes(block));
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

pub fn hid_catalog_from_names(names: &[&str]) -> Vec<Detected> {
    let mut seen: HashSet<(u16, u16)> = HashSet::new();
    let mut out = Vec::new();
    for name in names {
        let Some((vid, pid)) = parse_hid_bus_id(name) else {
            continue;
        };
        if !seen.insert((vid, pid)) {
            continue;
        }
        let Some(model) = classify_input_id(vid, pid) else {
            continue;
        };
        out.push(Detected {
            model,
            vid,
            pid,
            nodes: Vec::new(),
        });
    }
    out
}

pub fn merge_catalog(proc: Vec<Detected>, hid: Vec<Detected>) -> Vec<Detected> {
    if proc.is_empty() {
        hid
    } else {
        proc
    }
}

pub fn detect_status(found: Option<&Detected>, readable: impl Fn(&Path) -> bool) -> DetectStatus {
    let Some(dev) = found else {
        return DetectStatus::Missing;
    };
    if dev.nodes.is_empty() {
        return DetectStatus::Permission;
    }
    if dev.nodes.iter().all(|path| readable(path)) {
        DetectStatus::Present
    } else {
        DetectStatus::Permission
    }
}

pub fn device_ui_state(
    status: DetectStatus,
    model: Option<DeviceModel>,
    grab_conflict: bool,
) -> DeviceUiState {
    match status {
        DetectStatus::Missing => DeviceUiState {
            present: false,
            model: None,
            evdev_ok: true,
            grab_conflict: false,
        },
        DetectStatus::Permission => DeviceUiState {
            present: true,
            model,
            evdev_ok: false,
            grab_conflict: false,
        },
        DetectStatus::Present => DeviceUiState {
            present: true,
            model,
            evdev_ok: true,
            grab_conflict,
        },
    }
}

pub fn snapshot_changed(prev: &DeviceUiState, next: &DeviceUiState) -> bool {
    prev != next
}

pub fn scan_interval_ms(grabbed: bool) -> u64 {
    if grabbed {
        SCAN_INTERVAL_ACTIVE_MS
    } else {
        SCAN_INTERVAL_MISSING_MS
    }
}

pub fn enumerate_tartarus() -> Vec<Detected> {
    let proc_text = fs::read_to_string(PROC_INPUT_DEVICES).unwrap_or_default();
    let proc = parse_proc_input_devices(&proc_text);
    let hid_names = hid_dir_names();
    let hid_refs: Vec<&str> = hid_names.iter().map(String::as_str).collect();
    let hid = hid_catalog_from_names(&hid_refs);
    merge_catalog(proc, hid)
}

fn hid_dir_names() -> Vec<String> {
    let Ok(entries) = fs::read_dir(HID_DEVICES) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect()
}

fn parse_proc_ids(block: &str) -> Option<(u16, u16)> {
    let line = block.lines().find(|line| line.starts_with("I:"))?;
    let vid = field_hex(line, VENDOR_FIELD)?;
    let pid = field_hex(line, PRODUCT_FIELD)?;
    Some((vid, pid))
}

fn field_hex(line: &str, key: &str) -> Option<u16> {
    let start = line.find(key)? + key.len();
    let token = line[start..].split_whitespace().next()?;
    u16::from_str_radix(token, HEX_RADIX).ok()
}

fn parse_proc_event_nodes(block: &str) -> Vec<PathBuf> {
    let Some(line) = block.lines().find(|line| line.starts_with("H:")) else {
        return Vec::new();
    };
    line.split_whitespace()
        .filter_map(|token| {
            let token = token.strip_prefix(HANDLERS_FIELD).unwrap_or(token);
            is_event_handler(token).then_some(token)
        })
        .map(|token| PathBuf::from(DEV_INPUT).join(token))
        .collect()
}

fn is_event_handler(token: &str) -> bool {
    token.starts_with(EVENT_HANDLER_PREFIX)
        && token
            .as_bytes()
            .get(EVENT_HANDLER_PREFIX.len())
            .is_some_and(u8::is_ascii_digit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentartarus_core::constants::{
        USB_PID_NAGA_PRO_1, USB_PID_TARTARUS_PRO, USB_PID_TARTARUS_V2, USB_VID_RAZER,
    };
    use std::path::Path;

    const PROC_V2_AND_NAGA: &str = "\
I: Bus=0003 Vendor=1532 Product=008f Version=0111
N: Name=\"Razer Naga Pro\"
H: Handlers=event22 mouse0

I: Bus=0003 Vendor=1532 Product=022b Version=0111
N: Name=\"Razer Razer Tartarus V2\"
H: Handlers=sysrq kbd leds event270

I: Bus=0003 Vendor=1532 Product=022b Version=0111
N: Name=\"Razer Razer Tartarus V2\"
H: Handlers=sysrq kbd event271

I: Bus=0003 Vendor=1532 Product=022b Version=0111
N: Name=\"Razer Razer Tartarus V2\"
H: Handlers=event272 mouse10

I: Bus=0003 Vendor=1234 Product=5678 Version=0111
N: Name=\"OpenTartarus Keyboard\"
H: Handlers=sysrq kbd event273
";

    fn v2_detected(nodes: &[&str]) -> Detected {
        Detected {
            model: DeviceModel::V2,
            vid: USB_VID_RAZER,
            pid: USB_PID_TARTARUS_V2,
            nodes: nodes.iter().map(PathBuf::from).collect(),
        }
    }

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
        let b = v2_detected(&[]);
        assert_eq!(pick_first(vec![a, b]).unwrap().model, DeviceModel::Pro);
    }

    #[test]
    fn busy_is_grab_conflict() {
        assert_eq!(
            grab_conflict_message("EBUSY: Device or resource busy"),
            ErrorCode::GrabConflict
        );
    }

    #[test]
    fn access_denied_is_permission_not_missing() {
        assert_eq!(
            grab_conflict_message("Permission denied (os error 13)"),
            ErrorCode::Permission
        );
        assert_eq!(
            grab_conflict_message("EACCES"),
            ErrorCode::Permission
        );
    }

    #[test]
    fn proc_finds_v2_by_vid_pid_despite_openrazer_name() {
        let found = parse_proc_input_devices(PROC_V2_AND_NAGA);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].model, DeviceModel::V2);
        assert_eq!(found[0].vid, USB_VID_RAZER);
        assert_eq!(found[0].pid, USB_PID_TARTARUS_V2);
        assert_eq!(
            found[0].nodes,
            vec![
                PathBuf::from("/dev/input/event270"),
                PathBuf::from("/dev/input/event271"),
                PathBuf::from("/dev/input/event272"),
            ]
        );
    }

    #[test]
    fn empty_proc_is_missing_unless_hid_lists_tartarus() {
        assert!(parse_proc_input_devices("").is_empty());
        assert_eq!(detect_status(None, |_| true), DetectStatus::Missing);
        let hid = hid_catalog_from_names(&[
            "0003:1532:008F.0002",
            "0003:1532:022B.0019",
            "0003:1532:022B.001A",
        ]);
        assert_eq!(hid.len(), 1);
        assert_eq!(hid[0].pid, USB_PID_TARTARUS_V2);
        assert!(hid[0].nodes.is_empty());
        assert_eq!(
            detect_status(hid.first(), |_| true),
            DetectStatus::Permission
        );
        let merged = merge_catalog(Vec::new(), hid);
        assert_eq!(
            detect_status(merged.first(), |_| false),
            DetectStatus::Permission
        );
    }

    #[test]
    fn unreadable_evdev_nodes_are_permission_not_missing() {
        let found = v2_detected(&["/dev/input/event270", "/dev/input/event271"]);
        let status = detect_status(Some(&found), |_path: &Path| false);
        assert_eq!(status, DetectStatus::Permission);
        let ui = device_ui_state(status, Some(DeviceModel::V2), false);
        assert!(ui.present);
        assert!(!ui.evdev_ok);
        assert!(!ui.grab_conflict);
    }

    #[test]
    fn readable_nodes_are_present() {
        let found = v2_detected(&["/dev/input/event270"]);
        let status = detect_status(Some(&found), |_| true);
        assert_eq!(status, DetectStatus::Present);
        let busy = device_ui_state(status, Some(DeviceModel::V2), true);
        assert!(busy.present);
        assert!(busy.evdev_ok);
        assert!(busy.grab_conflict);
    }

    #[test]
    fn device_changed_emits_only_when_snapshot_changes() {
        let missing = device_ui_state(DetectStatus::Missing, None, false);
        let permission = device_ui_state(DetectStatus::Permission, Some(DeviceModel::V2), false);
        assert!(snapshot_changed(&missing, &permission));
        assert!(!snapshot_changed(&permission, &permission));
        assert_eq!(scan_interval_ms(false), SCAN_INTERVAL_MISSING_MS);
        assert_eq!(scan_interval_ms(true), SCAN_INTERVAL_ACTIVE_MS);
        assert!(SCAN_INTERVAL_MISSING_MS > SCAN_INTERVAL_ACTIVE_MS);
    }

    #[test]
    fn hid_name_parse_is_case_insensitive_hex() {
        assert_eq!(
            parse_hid_bus_id("0003:1532:022B.0019"),
            Some((USB_VID_RAZER, USB_PID_TARTARUS_V2))
        );
        assert_eq!(
            parse_hid_bus_id("0003:1532:022b.001a"),
            Some((USB_VID_RAZER, USB_PID_TARTARUS_V2))
        );
        assert_eq!(parse_hid_bus_id("0003:1532:008F.0002"), Some((USB_VID_RAZER, USB_PID_NAGA_PRO_1)));
        assert!(classify_input_id(USB_VID_RAZER, USB_PID_NAGA_PRO_1).is_none());
    }
}
