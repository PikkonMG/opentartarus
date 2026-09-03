use crate::app::{uses_color, App, Message, LIGHTING_EFFECTS};
use crate::theme::{self, COLOR_TEXT};
use crate::view::widgets::surface_container;
use iced::widget::{container, pick_list, row, slider, text, Space};
use iced::{Background, Border, Color, Element, Length, Theme};

pub fn lighting_tab(app: &App) -> Element<'_, Message> {
    let mut row = row![].spacing(10).align_y(iced::Alignment::Center);
    if let Some(copy) = app.lighting_footer_message() {
        row = row.push(text(copy).size(13));
    } else if app.openrazer {
        let selected = LIGHTING_EFFECTS
            .iter()
            .copied()
            .find(|e| e.0 == app.lighting.effect);
        row = row.push(
            pick_list(LIGHTING_EFFECTS, selected, Message::LightingEffect)
                .placeholder("effect")
                .text_size(13)
                .width(Length::Fixed(120.0)),
        );
        row = row.push(
            slider(
                theme::BRIGHTNESS_MIN..=theme::BRIGHTNESS_MAX,
                app.lighting.brightness,
                Message::LightingBrightness,
            )
            .width(Length::Fixed(140.0)),
        );
        if uses_color(app.lighting.effect) {
            let rgb = app.lighting.color.unwrap_or(theme::DEFAULT_LIGHT_COLOR);
            row = row.push(color_swatch(rgb));
            row = row.push(
                slider(0..=theme::COLOR_CHANNEL_MAX, rgb[0], |v| {
                    Message::LightingColor(0, v)
                })
                .width(80.0),
            );
            row = row.push(
                slider(0..=theme::COLOR_CHANNEL_MAX, rgb[1], |v| {
                    Message::LightingColor(1, v)
                })
                .width(80.0),
            );
            row = row.push(
                slider(0..=theme::COLOR_CHANNEL_MAX, rgb[2], |v| {
                    Message::LightingColor(2, v)
                })
                .width(80.0),
            );
        }
    }
    container(row.padding([0, 10]))
        .width(Length::Fill)
        .height(Length::Shrink)
        .style(surface_container)
        .into()
}

fn color_swatch<'a>(rgb: [u8; 3]) -> Element<'a, Message> {
    container(Space::new(Length::Fixed(22.0), Length::Fixed(22.0)))
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb8(rgb[0], rgb[1], rgb[2]))),
            border: Border {
                color: COLOR_TEXT,
                width: 1.0,
                radius: 3.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}
