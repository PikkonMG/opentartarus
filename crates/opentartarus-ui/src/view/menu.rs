use crate::app::{App, Banner, Message};
use crate::theme;
use crate::view::widgets::{danger_text_button, quiet_button};
use iced::widget::{button, column, container, text, Space};
use iced::{Alignment, Element, Length, Theme};

/// One row of the header menu. Kept as data so the choice is testable without
/// rendering anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEntry {
    Minimize,
    Maximize,
    Close,
    FixPermissions,
    Quit,
}

impl MenuEntry {
    pub fn label(self) -> &'static str {
        match self {
            MenuEntry::Minimize => theme::MENU_MINIMIZE,
            MenuEntry::Maximize => theme::MENU_MAXIMIZE,
            MenuEntry::Close => theme::MENU_CLOSE,
            MenuEntry::FixPermissions => theme::BUTTON_FIX_PERMISSIONS,
            MenuEntry::Quit => theme::BUTTON_QUIT,
        }
    }

    pub fn message(self) -> Message {
        match self {
            MenuEntry::Minimize => Message::MinimizeWindow,
            MenuEntry::Maximize => Message::ToggleMaximize,
            MenuEntry::Close => Message::CloseWindow,
            MenuEntry::FixPermissions => Message::FixPermissions,
            MenuEntry::Quit => Message::Quit,
        }
    }

    /// Quit stops the daemon too, so it is the only destructive entry.
    pub fn is_destructive(self) -> bool {
        matches!(self, MenuEntry::Quit)
    }
}

/// Repair is offered only while the permission banner is showing, so the menu
/// never advertises a fix for a problem the user does not have.
pub fn menu_entries(app: &App) -> Vec<MenuEntry> {
    let mut entries = vec![
        MenuEntry::Minimize,
        MenuEntry::Maximize,
        MenuEntry::Close,
    ];
    if matches!(app.banner(), Some(Banner::Permission)) {
        entries.push(MenuEntry::FixPermissions);
    }
    entries.push(MenuEntry::Quit);
    entries
}

pub fn menu_popup(app: &App) -> Element<'_, Message> {
    let mut items = column![].spacing(theme::SPACE_XS);
    for entry in menu_entries(app) {
        let style = if entry.is_destructive() {
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

    /// The window controls the menu always offers, in order.
    const WINDOW_ENTRIES: [MenuEntry; 3] =
        [MenuEntry::Minimize, MenuEntry::Maximize, MenuEntry::Close];

    #[test]
    fn a_healthy_session_offers_window_controls_then_quit() {
        let app = running();
        assert_eq!(app.banner(), None);
        let mut expected = WINDOW_ENTRIES.to_vec();
        expected.push(MenuEntry::Quit);
        assert_eq!(menu_entries(&app), expected);
    }

    #[test]
    fn a_permission_problem_adds_the_repair_entry_above_quit() {
        let mut app = running();
        app.evdev_ok = false;
        assert_eq!(app.banner(), Some(Banner::Permission));
        let mut expected = WINDOW_ENTRIES.to_vec();
        expected.push(MenuEntry::FixPermissions);
        expected.push(MenuEntry::Quit);
        assert_eq!(menu_entries(&app), expected);
    }

    #[test]
    fn other_banners_do_not_offer_the_repair_entry() {
        let mut app = running();
        app.device_present = false;
        app.ever_present = false;
        assert_eq!(app.banner(), Some(Banner::NoDevice));
        assert!(!menu_entries(&app).contains(&MenuEntry::FixPermissions));
    }

    #[test]
    fn window_controls_are_always_offered_and_come_first() {
        let mut app = running();
        assert_eq!(&menu_entries(&app)[..WINDOW_ENTRIES.len()], &WINDOW_ENTRIES);
        app.evdev_ok = false;
        assert_eq!(&menu_entries(&app)[..WINDOW_ENTRIES.len()], &WINDOW_ENTRIES);
    }

    #[test]
    fn only_quit_is_destructive() {
        // Quit stops the daemon; closing the window does not.
        for entry in WINDOW_ENTRIES {
            assert!(!entry.is_destructive(), "{entry:?} must not read as danger");
        }
        assert!(!MenuEntry::FixPermissions.is_destructive());
        assert!(MenuEntry::Quit.is_destructive());
    }

    #[test]
    fn closing_the_window_is_not_quitting_the_app() {
        assert!(matches!(MenuEntry::Close.message(), Message::CloseWindow));
        assert!(matches!(MenuEntry::Quit.message(), Message::Quit));
        assert_ne!(MenuEntry::Close.label(), MenuEntry::Quit.label());
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
        assert_eq!(MenuEntry::Minimize.label(), theme::MENU_MINIMIZE);
        assert_eq!(MenuEntry::Maximize.label(), theme::MENU_MAXIMIZE);
        assert_eq!(MenuEntry::Close.label(), theme::MENU_CLOSE);

        assert!(matches!(MenuEntry::Quit.message(), Message::Quit));
        assert!(matches!(
            MenuEntry::FixPermissions.message(),
            Message::FixPermissions
        ));
        assert!(matches!(
            MenuEntry::Minimize.message(),
            Message::MinimizeWindow
        ));
        assert!(matches!(
            MenuEntry::Maximize.message(),
            Message::ToggleMaximize
        ));

        // No two entries may share a label, or the menu reads as a duplicate.
        let mut labels: Vec<&str> = menu_entries(&{
            let mut app = running();
            app.evdev_ok = false;
            app
        })
        .into_iter()
        .map(MenuEntry::label)
        .collect();
        let total = labels.len();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), total, "menu labels must be unique");
    }
}
