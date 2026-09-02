use opentartarus_core::constants::{USB_PID_TARTARUS_PRO, USB_PID_TARTARUS_V2, USB_VID_RAZER};
use opentartarus_core::error::ErrorCode;
use opentartarus_core::lighting::{brightness_to_razer, wave_sysfs_value, LightingClient};
use opentartarus_core::types::{Lighting, LightingEffect};
use std::fs;
use std::path::{Path, PathBuf};

pub const OPENRAZER_BUS_NAME: &str = "org.razer";
const RAZERKBD_SYSFS: &str = "/sys/bus/hid/drivers/razerkbd";
const OPENRAZER_ROOT: &str = "/org/razer";
const DEVICES_INTERFACE: &str = "razer.devices";
const MISC_INTERFACE: &str = "razer.device.misc";
const CHROMA_INTERFACE: &str = "razer.device.lighting.chroma";
const BRIGHTNESS_INTERFACE: &str = "razer.device.lighting.brightness";
const FALLBACK_RGB: [u8; 3] = [255, 255, 255];
const EFFECT_SPEED: u8 = 2;

pub struct OpenRazerClient {
    vid: Option<u16>,
    pid: Option<u16>,
}

impl OpenRazerClient {
    pub fn new(vid: Option<u16>, pid: Option<u16>) -> Self {
        Self { vid, pid }
    }

    pub fn set_usb(&mut self, vid: u16, pid: u16) {
        self.vid = Some(vid);
        self.pid = Some(pid);
    }

    pub fn clear_usb(&mut self) {
        self.vid = None;
        self.pid = None;
    }
}

impl LightingClient for OpenRazerClient {
    fn apply(&mut self, lighting: &Lighting) -> Result<(), ErrorCode> {
        if session_has_openrazer() && apply_via_dbus(lighting, self.vid, self.pid).is_ok() {
            return Ok(());
        }
        apply_via_sysfs(lighting, self.vid, self.pid)
    }

    fn available(&self) -> bool {
        session_has_openrazer() || find_sysfs_node(self.vid, self.pid).is_some()
    }
}

fn session_has_openrazer() -> bool {
    let Ok(conn) = zbus::blocking::Connection::session() else {
        return false;
    };
    let Ok(proxy) = zbus::blocking::fdo::DBusProxy::new(&conn) else {
        return false;
    };
    let Ok(name) = zbus::names::BusName::try_from(OPENRAZER_BUS_NAME) else {
        return false;
    };
    proxy.name_has_owner(name).unwrap_or(false)
}

fn apply_via_dbus(lighting: &Lighting, vid: Option<u16>, pid: Option<u16>) -> Result<(), ErrorCode> {
    let conn = zbus::blocking::Connection::session().map_err(|_| ErrorCode::Lighting)?;
    let reply = conn
        .call_method(
            Some(OPENRAZER_BUS_NAME),
            OPENRAZER_ROOT,
            Some(DEVICES_INTERFACE),
            "getDevices",
            &(),
        )
        .map_err(|_| ErrorCode::Lighting)?;
    let serials: Vec<String> = reply.body().deserialize().map_err(|_| ErrorCode::Lighting)?;
    let mut applied = false;
    for serial in serials {
        let path = format!("/org/razer/device/{serial}");
        if !dbus_device_matches(&conn, &path, vid, pid) {
            continue;
        }
        if apply_dbus_device(&conn, &path, lighting).is_ok() {
            applied = true;
            break;
        }
    }
    if applied {
        Ok(())
    } else {
        Err(ErrorCode::Lighting)
    }
}

fn dbus_device_matches(
    conn: &zbus::blocking::Connection,
    path: &str,
    vid: Option<u16>,
    pid: Option<u16>,
) -> bool {
    let (Some(want_vid), Some(want_pid)) = (vid, pid) else {
        return true;
    };
    match dbus_vid_pid(conn, path) {
        Some((got_vid, got_pid)) => got_vid == want_vid && got_pid == want_pid,
        None => true,
    }
}

fn dbus_vid_pid(conn: &zbus::blocking::Connection, path: &str) -> Option<(u16, u16)> {
    let vid = dbus_id_method(conn, path, "getVid")?;
    let pid = dbus_id_method(conn, path, "getPid")?;
    Some((vid, pid))
}

fn dbus_id_method(conn: &zbus::blocking::Connection, path: &str, method: &str) -> Option<u16> {
    let reply = conn
        .call_method(
            Some(OPENRAZER_BUS_NAME),
            path,
            Some(MISC_INTERFACE),
            method,
            &(),
        )
        .ok()?;
    if let Ok(raw) = reply.body().deserialize::<String>() {
        return parse_usb_id(&raw);
    }
    reply.body().deserialize::<u16>().ok()
}

fn parse_usb_id(raw: &str) -> Option<u16> {
    let trimmed = raw.trim().trim_start_matches("0x").trim_start_matches("0X");
    u16::from_str_radix(trimmed, 16).ok()
}

fn apply_dbus_device(
    conn: &zbus::blocking::Connection,
    path: &str,
    lighting: &Lighting,
) -> Result<(), ErrorCode> {
    let rgb = lighting.color.unwrap_or(FALLBACK_RGB);
    match lighting.effect {
        LightingEffect::None => {
            dbus_chroma(conn, path, "setNone", &())?;
        }
        LightingEffect::Static => {
            dbus_chroma(conn, path, "setStatic", &(rgb[0], rgb[1], rgb[2]))?;
        }
        LightingEffect::Wave => {
            dbus_chroma(conn, path, "setWave", &(wave_sysfs_value(),))?;
        }
        LightingEffect::Spectrum => {
            dbus_chroma(conn, path, "setSpectrum", &())?;
        }
        LightingEffect::Breath => {
            dbus_chroma(conn, path, "setBreathSingle", &(rgb[0], rgb[1], rgb[2]))?;
        }
        LightingEffect::Reactive => {
            dbus_chroma(
                conn,
                path,
                "setReactive",
                &(rgb[0], rgb[1], rgb[2], EFFECT_SPEED),
            )?;
        }
        LightingEffect::Starlight => {
            dbus_chroma(
                conn,
                path,
                "setStarlightSingle",
                &(rgb[0], rgb[1], rgb[2], EFFECT_SPEED),
            )?;
        }
    }
    conn.call_method(
        Some(OPENRAZER_BUS_NAME),
        path,
        Some(BRIGHTNESS_INTERFACE),
        "setBrightness",
        &(f64::from(lighting.brightness),),
    )
    .map_err(|_| ErrorCode::Lighting)?;
    Ok(())
}

fn dbus_chroma<A: zbus::zvariant::DynamicType + serde::Serialize>(
    conn: &zbus::blocking::Connection,
    path: &str,
    method: &str,
    body: &A,
) -> Result<(), ErrorCode> {
    conn.call_method(
        Some(OPENRAZER_BUS_NAME),
        path,
        Some(CHROMA_INTERFACE),
        method,
        body,
    )
    .map(|_| ())
    .map_err(|_| ErrorCode::Lighting)
}

fn apply_via_sysfs(
    lighting: &Lighting,
    vid: Option<u16>,
    pid: Option<u16>,
) -> Result<(), ErrorCode> {
    let node = find_sysfs_node(vid, pid).ok_or(ErrorCode::Lighting)?;
    let rgb = lighting.color.unwrap_or(FALLBACK_RGB);
    let effect_ok = match lighting.effect {
        LightingEffect::None => write_sysfs(&node, "matrix_effect_none", b"1"),
        LightingEffect::Static => write_sysfs(&node, "matrix_effect_static", &rgb),
        LightingEffect::Wave => write_sysfs(
            &node,
            "matrix_effect_wave",
            wave_sysfs_value().to_string().as_bytes(),
        ),
        LightingEffect::Spectrum => write_sysfs(&node, "matrix_effect_spectrum", b"1"),
        LightingEffect::Breath => write_sysfs(&node, "matrix_effect_breath", &rgb),
        LightingEffect::Reactive => {
            write_sysfs(&node, "matrix_effect_reactive", &[EFFECT_SPEED, rgb[0], rgb[1], rgb[2]])
        }
        LightingEffect::Starlight => {
            write_sysfs(
                &node,
                "matrix_effect_starlight",
                &[EFFECT_SPEED, rgb[0], rgb[1], rgb[2]],
            )
        }
    };
    let brightness = brightness_to_razer(lighting.brightness).to_string();
    let bright_ok = write_sysfs(&node, "matrix_brightness", brightness.as_bytes());
    if effect_ok && bright_ok {
        Ok(())
    } else {
        Err(ErrorCode::Lighting)
    }
}

fn write_sysfs(node: &Path, name: &str, bytes: &[u8]) -> bool {
    fs::write(node.join(name), bytes).is_ok()
}

pub fn find_sysfs_node(vid: Option<u16>, pid: Option<u16>) -> Option<PathBuf> {
    let dir = fs::read_dir(RAZERKBD_SYSFS).ok()?;
    let mut fallback = None;
    for entry in dir.flatten() {
        let path = entry.path();
        if !path.join("matrix_brightness").is_file() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if let (Some(vid), Some(pid)) = (vid, pid) {
            if sysfs_name_matches(&name, vid, pid) {
                return Some(path);
            }
        } else if sysfs_name_matches(&name, USB_VID_RAZER, USB_PID_TARTARUS_V2)
            || sysfs_name_matches(&name, USB_VID_RAZER, USB_PID_TARTARUS_PRO)
        {
            return Some(path);
        } else if fallback.is_none() {
            fallback = Some(path);
        }
    }
    fallback
}

pub fn sysfs_name_matches(name: &str, vid: u16, pid: u16) -> bool {
    let upper = format!("{vid:04X}:{pid:04X}");
    let lower = format!("{vid:04x}:{pid:04x}");
    name.contains(&upper) || name.contains(&lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sysfs_name_matches_tartarus_ids() {
        assert!(sysfs_name_matches("0003:1532:022B.001B", USB_VID_RAZER, USB_PID_TARTARUS_V2));
        assert!(sysfs_name_matches("0003:1532:022b.001b", USB_VID_RAZER, USB_PID_TARTARUS_V2));
        assert!(sysfs_name_matches("0003:1532:0244.0001", USB_VID_RAZER, USB_PID_TARTARUS_PRO));
        assert!(!sysfs_name_matches("0003:1532:008F.0001", USB_VID_RAZER, USB_PID_TARTARUS_V2));
        assert_eq!(OPENRAZER_BUS_NAME, "org.razer");
    }
}
