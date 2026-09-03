use ksni::menu::StandardItem;
use ksni::Handle;
use ksni::Tray;
use opentartarus_core::pack::{shipped_profile, SHIPPED_IDS};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

pub const TRAY_ID: &str = "opentartarus";
pub const MENU_PROFILES: &str = "Profiles";
pub const MENU_OPEN: &str = "Open";
pub const MENU_QUIT: &str = "Quit";
const TRAY_ICON_NAME: &str = "opentartarus";

const TRAY_ICON_PNG: &[u8] = include_bytes!("../../../packaging/icons/opentartarus-64.png");
pub const TRAY_ICON_SIZE: i32 = 64;
pub const ICON_BYTES_PER_PIXEL: usize = 4;

const ALPHA_INDEX: usize = 3;

#[derive(Debug, Clone)]
pub enum TrayCmd {
    Open,
    Quit,
    ApplyProfile(String),
}

pub fn tray_tooltip(profile_name: &str) -> String {
    format!("OpenTartarus — {profile_name}")
}

pub fn tray_name_from_profile_id(id: &str) -> String {
    shipped_profile(id)
        .map(|profile| profile.name)
        .unwrap_or_else(|_| id.to_string())
}

pub fn tray_name_from_applied_params(params: &Value) -> Option<String> {
    params
        .get("id")
        .and_then(Value::as_str)
        .map(tray_name_from_profile_id)
}

pub fn tray_cmd_requests_quit(cmd: Option<TrayCmd>) -> bool {
    matches!(cmd, Some(TrayCmd::Quit))
}

/// The names of every shipped profile, in pack order. Used by the tray submenu
/// and by its test.
pub fn tray_profile_submenu_labels() -> Vec<String> {
    SHIPPED_IDS
        .iter()
        .map(|id| {
            shipped_profile(id)
                .map(|profile| profile.name)
                .unwrap_or_else(|_| (*id).to_string())
        })
        .collect()
}

/// The top level of the tray menu. Profiles live one level down.
pub fn tray_menu_labels(_active_name: &str) -> Vec<String> {
    vec![
        String::from(MENU_PROFILES),
        String::from(MENU_OPEN),
        String::from(MENU_QUIT),
    ]
}

/// Decodes the embedded PNG. Returns `None` rather than panicking, so a bad
/// asset degrades to the old blank icon instead of taking the daemon down.
pub fn tray_icon_rgba() -> Option<(u32, u32, Vec<u8>)> {
    let decoder = png::Decoder::new(TRAY_ICON_PNG);
    let mut reader = decoder.read_info().ok()?;
    let mut buffer = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).ok()?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    buffer.truncate(info.buffer_size());
    Some((info.width, info.height, buffer))
}

/// The tray icon, in the ARGB32 network byte order `ksni` asks for.
pub fn tray_icon_pixmap() -> Vec<ksni::Icon> {
    let Some((width, height, rgba)) = tray_icon_rgba() else {
        return Vec::new();
    };
    let mut argb = Vec::with_capacity(rgba.len());
    for pixel in rgba.chunks_exact(ICON_BYTES_PER_PIXEL) {
        argb.push(pixel[ALPHA_INDEX]);
        argb.push(pixel[0]);
        argb.push(pixel[1]);
        argb.push(pixel[2]);
    }
    vec![ksni::Icon {
        width: width as i32,
        height: height as i32,
        data: argb,
    }]
}

pub struct OpenTartarusTray {
    pub profile_name: String,
    tx: UnboundedSender<TrayCmd>,
}

impl OpenTartarusTray {
    pub fn new(profile_name: String, tx: UnboundedSender<TrayCmd>) -> Self {
        Self { profile_name, tx }
    }

    fn send(&self, cmd: TrayCmd) {
        let _ = self.tx.send(cmd);
    }
}

impl Tray for OpenTartarusTray {
    fn id(&self) -> String {
        TRAY_ID.into()
    }

    fn title(&self) -> String {
        tray_tooltip(&self.profile_name)
    }

    fn icon_name(&self) -> String {
        TRAY_ICON_NAME.into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        tray_icon_pixmap()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            icon_name: TRAY_ICON_NAME.into(),
            icon_pixmap: tray_icon_pixmap(),
            title: tray_tooltip(&self.profile_name),
            description: String::new(),
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        self.send(TrayCmd::Open);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::SubMenu;

        let mut profiles = Vec::with_capacity(SHIPPED_IDS.len());
        for id in SHIPPED_IDS {
            let name = shipped_profile(id)
                .map(|profile| profile.name)
                .unwrap_or_else(|_| (*id).to_string());
            let profile_id = (*id).to_string();
            profiles.push(
                StandardItem {
                    label: name,
                    activate: Box::new(move |this: &mut Self| {
                        this.send(TrayCmd::ApplyProfile(profile_id.clone()));
                    }),
                    ..Default::default()
                }
                .into(),
            );
        }

        vec![
            SubMenu {
                label: MENU_PROFILES.into(),
                submenu: profiles,
                ..Default::default()
            }
            .into(),
            ksni::MenuItem::Separator,
            StandardItem {
                label: MENU_OPEN.into(),
                activate: Box::new(|this: &mut Self| this.send(TrayCmd::Open)),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: MENU_QUIT.into(),
                activate: Box::new(|this: &mut Self| this.send(TrayCmd::Quit)),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub async fn spawn_tray(tray: OpenTartarusTray) -> Option<Handle<OpenTartarusTray>> {
    use ksni::TrayMethods;
    match tray.spawn().await {
        Ok(handle) => Some(handle),
        Err(err) => {
            crate::log::write(&format!("tray {err}"));
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentartarus_core::pack::SHIPPED_IDS;

    #[test]
    fn the_top_level_menu_is_three_entries_long() {
        let labels = tray_menu_labels("Default");
        assert_eq!(
            labels,
            vec![
                String::from(MENU_PROFILES),
                String::from(MENU_OPEN),
                String::from(MENU_QUIT),
            ],
            "profiles must not flood the top level"
        );
    }

    #[test]
    fn the_submenu_lists_every_shipped_profile_in_pack_order() {
        let labels = tray_profile_submenu_labels();
        assert_eq!(labels.len(), SHIPPED_IDS.len());
        assert_eq!(labels[0], "Default");
        for (index, id) in SHIPPED_IDS.iter().enumerate() {
            let expected = shipped_profile(id).expect("shipped profile").name;
            assert_eq!(labels[index], expected, "row {index} out of order");
        }
    }

    #[test]
    fn the_submenu_names_are_unique() {
        let mut labels = tray_profile_submenu_labels();
        let total = labels.len();
        labels.sort();
        labels.dedup();
        assert_eq!(labels.len(), total, "two profiles must not share a name");
    }

    #[test]
    fn tooltip_follows_any_profile_applied_id() {
        let name = tray_name_from_applied_params(&serde_json::json!({
            "id": "league-of-legends"
        }))
        .unwrap();
        assert_eq!(name, "League of Legends");
        assert_eq!(tray_tooltip(&name), "OpenTartarus — League of Legends");
        assert_eq!(
            tray_name_from_applied_params(&serde_json::json!({ "id": "default" })).as_deref(),
            Some("Default")
        );
        assert_eq!(tray_name_from_applied_params(&serde_json::json!({})), None);
    }

    #[test]
    fn closed_tray_channel_is_not_quit() {
        assert!(!tray_cmd_requests_quit(None));
        assert!(tray_cmd_requests_quit(Some(TrayCmd::Quit)));
        assert!(!tray_cmd_requests_quit(Some(TrayCmd::Open)));
    }

    #[test]
    fn the_embedded_icon_decodes_to_the_declared_size() {
        let (width, height, rgba) = tray_icon_rgba().expect("the embedded PNG must decode");
        assert_eq!(width as i32, TRAY_ICON_SIZE);
        assert_eq!(height as i32, TRAY_ICON_SIZE);
        assert_eq!(
            rgba.len(),
            (TRAY_ICON_SIZE as usize) * (TRAY_ICON_SIZE as usize) * ICON_BYTES_PER_PIXEL
        );
    }

    #[test]
    fn the_tray_pixmap_is_one_icon_of_the_right_shape() {
        let icons = tray_icon_pixmap();
        assert_eq!(icons.len(), 1, "one icon, at one size");
        let icon = &icons[0];
        assert_eq!(icon.width, TRAY_ICON_SIZE);
        assert_eq!(icon.height, TRAY_ICON_SIZE);
        assert_eq!(
            icon.data.len(),
            (TRAY_ICON_SIZE as usize) * (TRAY_ICON_SIZE as usize) * ICON_BYTES_PER_PIXEL
        );
    }

    #[test]
    fn the_icon_is_not_blank() {
        let icons = tray_icon_pixmap();
        let data = &icons[0].data;
        let opaque = data
            .chunks_exact(ICON_BYTES_PER_PIXEL)
            .filter(|pixel| pixel[0] > 0)
            .count();
        assert!(
            opaque > data.len() / ICON_BYTES_PER_PIXEL / 4,
            "at least a quarter of the icon must be opaque, or it will look blank"
        );
        let mut colors: Vec<&[u8]> = data.chunks_exact(ICON_BYTES_PER_PIXEL).collect();
        colors.sort_unstable();
        colors.dedup();
        assert!(colors.len() > 1, "a one-colour icon is a blank icon");
    }

    #[test]
    fn the_pixmap_is_argb_not_rgba() {
        // The mark is a blue square with a white T. Sample a pixel inside the
        // square but outside the T: alpha first, then blue must dominate red.
        let icons = tray_icon_pixmap();
        let size = TRAY_ICON_SIZE as usize;
        let x = size / 8;
        let y = size / 2;
        let offset = (y * size + x) * ICON_BYTES_PER_PIXEL;
        let pixel = &icons[0].data[offset..offset + ICON_BYTES_PER_PIXEL];
        assert_eq!(pixel[0], u8::MAX, "byte 0 must be alpha");
        assert!(
            pixel[3] > pixel[1],
            "byte 3 must be blue and byte 1 red for an accent-blue mark"
        );
    }

    #[test]
    fn the_tray_still_offers_a_themed_name_as_well() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let tray = OpenTartarusTray::new(String::from("Default"), tx);
        assert_eq!(tray.icon_name(), TRAY_ICON_NAME);
        assert_eq!(tray.icon_pixmap().len(), 1);
        assert_eq!(tray.tool_tip().icon_pixmap.len(), 1);
    }
}
