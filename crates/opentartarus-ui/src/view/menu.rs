use crate::app::{App, Banner, Message};
use crate::theme;
use crate::view::widgets::{danger_text_button, quiet_button};
use iced::widget::{button, column, container, text, Space};
use iced::{Alignment, Element, Length, Theme};

/// One row of the header menu. Kept as data so the choice is testable without
/// rendering anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEntry {
    FixPermissions,
    Quit,
}

impl MenuEntry {
    pub fn label(self) -> &'static str {
        match self {
            MenuEntry::FixPermissions => theme::BUTTON_FIX_PERMISSIONS,
            MenuEntry::Quit => theme::BUTTON_QUIT,
        }
    }

    pub fn message(self) -> Message {
        match self {
            MenuEntry::FixPermissions => Message::FixPermissions,
            MenuEntry::Quit => Message::Quit,
        }
    }
}

/// Repair is offered only while the permission banner is showing, so the menu
/// never advertises a fix for a problem the user does not have.
pub fn menu_entries(app: &App) -> Vec<MenuEntry> {
    let mut entries = Vec::new();
    if matches!(app.banner(), Some(Banner::Permission)) {
        entries.push(MenuEntry::FixPermissions);
    }
    entries.push(MenuEntry::Quit);
    entries
}

pub fn menu_popup(app: &App) -> Element<'_, Message> {
    let mut items = column![].spacing(theme::SPACE_XS);
    for entry in menu_entries(app) {
        let style = if entry == MenuEntry::Quit {
            danger_text_button
        } else {
            quiet_button
        };
        items = items.push(
            button(text(entry.label()).size(theme::TEXT_BODY))
                .width(Length::Fill)
                .on_press(entry.message())
                .style(style),
        );
    }
    let popup = container(items.padding(theme::SPACE_SM))
        .width(Length::Fixed(theme::MENU_WIDTH))
        .style(crate::view::widgets::card_container);

    // Anchor the popup under the header button, at the top right.
    container(
        column![
            Space::with_height(theme::HEADER_HEIGHT),
            container(popup).align_x(Alignment::End).width(Length::Fill),
        ]
        .padding(theme::SPACE_SM),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme: &Theme| container::Style::default())
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, Banner, Phase};

    // Built with struct-update syntax rather than `let mut app =
    // App::default(); app.phase = ...` — clippy's `field_reassign_with_default`
    // flags that shape (it already does at app.rs's own `running` helper, kept
    // as-is there since fixing it is outside this task).
    fn running() -> App {
        App {
            phase: Phase::Running,
            device_present: true,
            ever_present: true,
            ..App::default()
        }
    }

    #[test]
    fn a_healthy_session_offers_only_quit() {
        let app = running();
        assert_eq!(app.banner(), None);
        assert_eq!(menu_entries(&app), vec![MenuEntry::Quit]);
    }

    #[test]
    fn a_permission_problem_adds_the_repair_entry_above_quit() {
        let mut app = running();
        app.evdev_ok = false;
        assert_eq!(app.banner(), Some(Banner::Permission));
        assert_eq!(
            menu_entries(&app),
            vec![MenuEntry::FixPermissions, MenuEntry::Quit]
        );
    }

    #[test]
    fn other_banners_do_not_offer_the_repair_entry() {
        let mut app = running();
        app.device_present = false;
        app.ever_present = false;
        assert_eq!(app.banner(), Some(Banner::NoDevice));
        assert_eq!(menu_entries(&app), vec![MenuEntry::Quit]);
    }

    #[test]
    fn quit_is_always_the_last_entry() {
        let mut app = running();
        assert_eq!(menu_entries(&app).last(), Some(&MenuEntry::Quit));
        app.evdev_ok = false;
        assert_eq!(menu_entries(&app).last(), Some(&MenuEntry::Quit));
    }

    #[test]
    fn each_entry_carries_its_own_label_and_message() {
        assert_eq!(MenuEntry::Quit.label(), theme::BUTTON_QUIT);
        assert_eq!(
            MenuEntry::FixPermissions.label(),
            theme::BUTTON_FIX_PERMISSIONS
        );
        assert!(matches!(MenuEntry::Quit.message(), Message::Quit));
        assert!(matches!(
            MenuEntry::FixPermissions.message(),
            Message::FixPermissions
        ));
    }
}
