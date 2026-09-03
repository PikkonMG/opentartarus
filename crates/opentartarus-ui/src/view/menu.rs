use crate::app::{App, Banner, Message};
use crate::theme;
use crate::view::widgets::{danger_text_button, quiet_button};
use iced::widget::{button, column, container, text, Space};
use iced::{Alignment, Element, Length, Theme};

/// One row of the header menu. Kept as data so the choice is testable without
/// rendering anything.
/// The app menu holds only what the title bar cannot.
///
/// Minimize, maximize and close are buttons in the header, three pixels away;
/// repeating them here made the menu read as a duplicate of its own neighbour.
/// What is left is the pair that has nowhere else to live: repairing a broken
/// install, and quitting for real. Closing the window hides it to the tray and
/// the remaps keep running, so Quit is a genuinely different action, not a
/// louder Close.
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

    /// What the entry does, spelled out, because "Quit" alone does not say
    /// that it also stops the background service.
    pub fn detail(self) -> &'static str {
        match self {
            MenuEntry::FixPermissions => theme::MENU_FIX_DETAIL,
            MenuEntry::Quit => theme::MENU_QUIT_DETAIL,
        }
    }

    pub fn message(self) -> Message {
        match self {
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
    let mut entries = Vec::new();
    if matches!(app.banner(), Some(Banner::Permission)) {
        entries.push(MenuEntry::FixPermissions);
    }
    entries.push(MenuEntry::Quit);
    entries
}

pub fn menu_popup(app: &App) -> Element<'_, Message> {
    let mut items = column![].spacing(theme::SPACE_XXS);
    for entry in menu_entries(app) {
        let style = if entry.is_destructive() {
            danger_text_button
        } else {
            quiet_button
        };
        let label_color = if entry.is_destructive() {
            theme::COLOR_DANGER
        } else {
            theme::COLOR_TEXT
        };
        items = items.push(
            button(
                column![
                    text(entry.label())
                        .size(theme::TEXT_BODY)
                        .color(label_color),
                    text(entry.detail())
                        .size(theme::TEXT_SMALL)
                        .color(theme::COLOR_TEXT_FAINT),
                ]
                .spacing(theme::SPACE_XXS),
            )
            .width(Length::Fill)
            .padding([theme::SPACE_SM, theme::SPACE_MD])
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

    // Struct-update syntax, not `App::default()` then field assignment, which
    // trips clippy::field_reassign_with_default and would break the warning
    // budget this repo holds at 22.
    fn running() -> App {
        App {
            phase: Phase::Running,
            device_present: true,
            ever_present: true,
            ..App::default()
        }
    }

    #[test]
    fn the_menu_never_repeats_a_window_control() {
        // Minimize, maximize and close are buttons in the header. Repeating
        // them here is what made the menu read as a duplicate of its
        // neighbour, so no entry may map to a window-control message.
        let mut app = running();
        app.evdev_ok = false;
        for entry in menu_entries(&app) {
            assert!(
                !matches!(
                    entry.message(),
                    Message::MinimizeWindow | Message::ToggleMaximize | Message::CloseWindow
                ),
                "{entry:?} duplicates a title-bar button"
            );
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
    fn only_quit_is_destructive() {
        assert!(MenuEntry::Quit.is_destructive());
        assert!(!MenuEntry::FixPermissions.is_destructive());
    }

    #[test]
    fn every_entry_says_what_it_does() {
        // "Quit" alone does not tell a user it also stops the remapping that
        // keeps working after the window closes.
        for entry in [MenuEntry::FixPermissions, MenuEntry::Quit] {
            assert!(!entry.label().is_empty(), "{entry:?} needs a label");
            let detail = entry.detail();
            assert!(!detail.is_empty(), "{entry:?} needs a detail line");
            assert_ne!(detail, entry.label(), "{entry:?} detail repeats its label");
        }
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

    #[test]
    fn entries_are_distinct() {
        let mut app = running();
        app.evdev_ok = false;
        let entries = menu_entries(&app);
        let mut labels: Vec<&str> = entries.iter().copied().map(MenuEntry::label).collect();
        let total = labels.len();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), total, "menu labels must be unique");
    }
}
