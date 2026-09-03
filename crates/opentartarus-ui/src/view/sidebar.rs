use crate::app::{App, Message};
use crate::theme::{self, PROFILE_LIST_WIDTH};
use crate::view::widgets::{quiet_button, selected_row_button, surface_container};
use iced::widget::{button, column, container, scrollable, text};
use iced::{Element, Length};

pub fn profile_list(app: &App) -> Element<'_, Message> {
    let mut list = column![].spacing(4);
    for row in &app.profiles {
        let selected = app.selected_profile_id.as_deref() == Some(row.id.as_str()) || row.is_active;
        let label = text(&row.name).size(14);
        let mut btn = button(label)
            .width(Length::Fill)
            .on_press(Message::SelectProfile(row.id.clone()));
        if selected {
            btn = btn.style(selected_row_button);
        } else {
            btn = btn.style(quiet_button);
        }
        list = list.push(btn);
        if row.is_active && row.can_revert {
            list = list.push(
                button(text(theme::BUTTON_REVERT).size(12))
                    .on_press(Message::RevertProfile)
                    .style(quiet_button),
            );
        }
    }
    container(
        column![scrollable(list.padding(8)).height(Length::Fill)]
            .spacing(8)
            .padding(8)
            .height(Length::Fill),
    )
    .width(Length::Fixed(PROFILE_LIST_WIDTH))
    .height(Length::Fill)
    .style(surface_container)
    .into()
}
