use crate::app::Message;
use crate::theme::{
    with_opacity, COLOR_ACCENT, COLOR_SURFACE, COLOR_TEXT, DISCONNECTED_OPACITY,
};
use iced::mouse;
use iced::widget::canvas::{self, Frame, Geometry, Path, Program, Stroke, Text};
use iced::{alignment, Point, Rectangle, Renderer, Size, Theme};
use opentartarus_core::labels::bind_label;
use opentartarus_core::types::{Action, DeviceModel, KeyId};
use std::collections::BTreeMap;

const GRID_COLS: usize = 4;
const GRID_ROWS: usize = 5;
const KEY_GAP: f32 = 6.0;
const PAD: f32 = 8.0;
const SIDE_WIDTH: f32 = 58.0;
const THUMB_HEIGHT: f32 = 96.0;
const KEY_TEXT_SIZE: f32 = 11.0;
const CORNER: f32 = 5.0;

const GRID_KEYS: [KeyId; 20] = [
    KeyId::Kp01,
    KeyId::Kp02,
    KeyId::Kp03,
    KeyId::Kp04,
    KeyId::Kp05,
    KeyId::Kp06,
    KeyId::Kp07,
    KeyId::Kp08,
    KeyId::Kp09,
    KeyId::Kp10,
    KeyId::Kp11,
    KeyId::Kp12,
    KeyId::Kp13,
    KeyId::Kp14,
    KeyId::Kp15,
    KeyId::Kp16,
    KeyId::Kp17,
    KeyId::Kp18,
    KeyId::Kp19,
    KeyId::Kp20,
];

#[derive(Debug, Clone)]
pub struct Keypad {
    pub model: Option<DeviceModel>,
    pub bindings: BTreeMap<KeyId, Action>,
    pub selected: Option<KeyId>,
    pub faded: bool,
}

impl Program<Message> for Keypad {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let opacity = if self.faded { DISCONNECTED_OPACITY } else { 1.0 };
        let fill = with_opacity(COLOR_SURFACE, opacity);
        let text_color = with_opacity(COLOR_TEXT, opacity);
        let accent = with_opacity(COLOR_ACCENT, opacity);
        for (id, rect) in key_rects(bounds.size(), self.model) {
            let path = Path::rounded_rectangle(
                rect.position(),
                rect.size(),
                iced::border::Radius::from(CORNER),
            );
            frame.fill(&path, fill);
            if self.selected == Some(id) {
                frame.stroke(
                    &path,
                    Stroke::default()
                        .with_color(accent)
                        .with_width(2.0),
                );
            }
            let label = bind_label(self.bindings.get(&id));
            if !label.is_empty() {
                frame.fill_text(Text {
                    content: label,
                    position: Point::new(rect.center_x(), rect.center_y()),
                    color: text_color,
                    size: iced::Pixels(KEY_TEXT_SIZE),
                    horizontal_alignment: alignment::Horizontal::Center,
                    vertical_alignment: alignment::Vertical::Center,
                    ..Text::default()
                });
            }
        }
        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (iced::event::Status, Option<Message>) {
        if let canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
            if let Some(local) = cursor.position_in(bounds) {
                if let Some(id) = hit_test(local, bounds.size(), self.model) {
                    return (iced::event::Status::Captured, Some(Message::SelectKey(id)));
                }
            }
        }
        (iced::event::Status::Ignored, None)
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if let Some(local) = cursor.position_in(bounds) {
            if hit_test(local, bounds.size(), self.model).is_some() {
                return mouse::Interaction::Pointer;
            }
        }
        mouse::Interaction::default()
    }
}

fn hit_test(local: Point, size: Size, model: Option<DeviceModel>) -> Option<KeyId> {
    key_rects(size, model)
        .into_iter()
        .find(|(_, rect)| rect.contains(local))
        .map(|(id, _)| id)
}

fn key_rects(size: Size, model: Option<DeviceModel>) -> Vec<(KeyId, Rectangle)> {
    let mut out = Vec::with_capacity(32);
    let thumb_h = match model {
        Some(DeviceModel::V2) | Some(DeviceModel::Pro) => THUMB_HEIGHT,
        None => THUMB_HEIGHT,
    };
    let grid_bottom = size.height - PAD - thumb_h - KEY_GAP;
    let grid_height = (grid_bottom - PAD).max(8.0);
    let grid_width = (size.width - PAD * 2.0 - SIDE_WIDTH - KEY_GAP).max(8.0);
    let cell_w = (grid_width - KEY_GAP * (GRID_COLS as f32 - 1.0)) / GRID_COLS as f32;
    let cell_h = (grid_height - KEY_GAP * (GRID_ROWS as f32 - 1.0)) / GRID_ROWS as f32;
    for row in 0..GRID_ROWS {
        for col in 0..GRID_COLS {
            let id = GRID_KEYS[row * GRID_COLS + col];
            let x = PAD + col as f32 * (cell_w + KEY_GAP);
            let y = PAD + row as f32 * (cell_h + KEY_GAP);
            out.push((id, Rectangle::new(Point::new(x, y), Size::new(cell_w, cell_h))));
        }
    }
    let side_x = PAD + grid_width + KEY_GAP;
    let wheel_h = (grid_height - KEY_GAP * 3.0) / 4.0;
    let wheel_keys = [KeyId::WheelUp, KeyId::WheelClick, KeyId::WheelDown, KeyId::Mode];
    for (i, id) in wheel_keys.iter().enumerate() {
        let y = PAD + i as f32 * (wheel_h + KEY_GAP);
        out.push((
            *id,
            Rectangle::new(Point::new(side_x, y), Size::new(SIDE_WIDTH, wheel_h)),
        ));
    }
    let thumb_y = grid_bottom + KEY_GAP;
    let thumb_area = Rectangle::new(
        Point::new(PAD, thumb_y),
        Size::new(size.width - PAD * 2.0, thumb_h),
    );
    match model {
        Some(DeviceModel::Pro) => push_analog(&mut out, thumb_area),
        _ => push_thumb8(&mut out, thumb_area),
    }
    out
}

fn push_thumb8(out: &mut Vec<(KeyId, Rectangle)>, root: Rectangle) {
    let dirs = [
        (KeyId::ThumbN, 0.0_f32, -1.0_f32),
        (KeyId::ThumbNe, 0.7, -0.7),
        (KeyId::ThumbE, 1.0, 0.0),
        (KeyId::ThumbSe, 0.7, 0.7),
        (KeyId::ThumbS, 0.0, 1.0),
        (KeyId::ThumbSw, -0.7, 0.7),
        (KeyId::ThumbW, -1.0, 0.0),
        (KeyId::ThumbNw, -0.7, -0.7),
    ];
    let pad = 18.0_f32;
    let cx = root.center_x();
    let cy = root.center_y();
    let rx = (root.width / 2.0 - pad).max(8.0);
    let ry = (root.height / 2.0 - pad).max(8.0);
    let w = 28.0;
    let h = 22.0;
    for (id, dx, dy) in dirs {
        out.push((
            id,
            Rectangle::new(
                Point::new(cx + dx * rx - w / 2.0, cy + dy * ry - h / 2.0),
                Size::new(w, h),
            ),
        ));
    }
}

fn push_analog(out: &mut Vec<(KeyId, Rectangle)>, root: Rectangle) {
    let w = 36.0;
    let h = 24.0;
    let cx = root.center_x();
    let cy = root.center_y();
    let span_x = (root.width / 2.0 - w).max(12.0);
    let span_y = (root.height / 2.0 - h).max(12.0);
    out.push((
        KeyId::AnalogUp,
        Rectangle::new(Point::new(cx - w / 2.0, cy - span_y - h / 2.0), Size::new(w, h)),
    ));
    out.push((
        KeyId::AnalogDown,
        Rectangle::new(Point::new(cx - w / 2.0, cy + span_y - h / 2.0), Size::new(w, h)),
    ));
    out.push((
        KeyId::AnalogLeft,
        Rectangle::new(Point::new(cx - span_x - w / 2.0, cy - h / 2.0), Size::new(w, h)),
    ));
    out.push((
        KeyId::AnalogRight,
        Rectangle::new(Point::new(cx + span_x - w / 2.0, cy - h / 2.0), Size::new(w, h)),
    ));
}

pub fn widget(keypad: Keypad) -> iced::Element<'static, Message> {
    iced::widget::canvas(keypad)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
}
