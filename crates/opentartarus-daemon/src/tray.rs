use ksni::menu::StandardItem;
use ksni::Handle;
use ksni::Tray;
use opentartarus_core::pack::{shipped_profile, SHIPPED_IDS};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

pub const TRAY_ID: &str = "opentartarus";
pub const MENU_OPEN: &str = "Open";
pub const MENU_QUIT: &str = "Quit";
const TRAY_ICON_NAME: &str = "opentartarus";

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

pub fn tray_menu_labels(_active_name: &str) -> Vec<String> {
    let mut v: Vec<String> = opentartarus_core::pack::SHIPPED_IDS
        .iter()
        .map(|id| opentartarus_core::pack::shipped_profile(id).unwrap().name)
        .collect();
    v.push(MENU_OPEN.into());
    v.push(MENU_QUIT.into());
    v
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

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            icon_name: TRAY_ICON_NAME.into(),
            icon_pixmap: Vec::new(),
            title: tray_tooltip(&self.profile_name),
            description: String::new(),
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        self.send(TrayCmd::Open);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        let mut items = Vec::with_capacity(SHIPPED_IDS.len() + 3);
        for id in SHIPPED_IDS {
            let name = shipped_profile(id).map(|p| p.name).unwrap_or_else(|_| (*id).to_string());
            let profile_id = (*id).to_string();
            items.push(
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
        items.push(ksni::MenuItem::Separator);
        items.push(
            StandardItem {
                label: MENU_OPEN.into(),
                activate: Box::new(|this: &mut Self| this.send(TrayCmd::Open)),
                ..Default::default()
            }
            .into(),
        );
        items.push(
            StandardItem {
                label: MENU_QUIT.into(),
                activate: Box::new(|this: &mut Self| this.send(TrayCmd::Quit)),
                ..Default::default()
            }
            .into(),
        );
        items
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
    fn tray_menu_lists_shipped_then_open_quit() {
        let labels = tray_menu_labels("Default");
        assert_eq!(labels[0], "Default");
        assert_eq!(labels.len(), SHIPPED_IDS.len() + 2);
        assert_eq!(labels[labels.len() - 2], "Open");
        assert_eq!(labels[labels.len() - 1], "Quit");
        assert_eq!(tray_tooltip("Default"), "OpenTartarus — Default");
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
}
