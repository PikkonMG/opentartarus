//! The three window-control glyphs, drawn rather than typed.
//!
//! A font's `\u{2013}`, `\u{25a1}` and `\u{00d7}` come from three different
//! glyph designs at three different optical weights and three different
//! heights above the baseline, so a row of them never lines up. Stroking them
//! on one canvas with one width and one box guarantees it does.

use crate::app::Message;
use crate::theme;
use iced::mouse;
use iced::widget::canvas::{Frame, Geometry, LineCap, Path, Program, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};

/// Edge of the square the glyph is drawn inside.
const GLYPH_BOX: f32 = 16.0;
/// Stroke width, shared by all three so they read as one set.
const STROKE: f32 = 1.5;
/// How far the cross reaches from the centre, as a fraction of the box. Kept
/// below half so the stroke's round cap stays inside the box.
const REACH: f32 = 0.30;
/// The chevron is wider than the cross is tall, so the two read at the same
/// optical size rather than the chevron shrinking to a dash.
const CHEVRON_REACH: f32 = 0.34;
/// How far the chevron's point drops below its arms. Flatter than the cross,
/// or it reads as an arrowhead; too flat and it reads as a dash.
const CHEVRON_RISE: f32 = 0.26;
const HALF: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowGlyph {
    /// Minimize: a chevron pointing down, matching the desktop's own controls.
    ChevronDown,
    /// Maximize and restore: the same chevron pointing up.
    ChevronUp,
    /// Close.
    Cross,
}

/// Canvas program for one glyph. Colour is fixed at construction because a
/// canvas cannot read the button status wrapping it.
#[derive(Debug, Clone, Copy)]
pub struct GlyphCanvas {
    pub glyph: WindowGlyph,
    pub color: Color,
}

impl Program<Message> for GlyphCanvas {
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
        let centre = Point::new(bounds.width / HALF, bounds.height / HALF);
        let reach = GLYPH_BOX * REACH;
        let arm = GLYPH_BOX * CHEVRON_REACH;
        let rise = GLYPH_BOX * CHEVRON_RISE;

        let path = match self.glyph {
            WindowGlyph::ChevronDown => Path::new(|builder| {
                builder.move_to(Point::new(centre.x - arm, centre.y - rise / HALF));
                builder.line_to(Point::new(centre.x, centre.y + rise / HALF));
                builder.line_to(Point::new(centre.x + arm, centre.y - rise / HALF));
            }),
            WindowGlyph::ChevronUp => Path::new(|builder| {
                builder.move_to(Point::new(centre.x - arm, centre.y + rise / HALF));
                builder.line_to(Point::new(centre.x, centre.y - rise / HALF));
                builder.line_to(Point::new(centre.x + arm, centre.y + rise / HALF));
            }),
            WindowGlyph::Cross => Path::new(|builder| {
                builder.move_to(Point::new(centre.x - reach, centre.y - reach));
                builder.line_to(Point::new(centre.x + reach, centre.y + reach));
                builder.move_to(Point::new(centre.x + reach, centre.y - reach));
                builder.line_to(Point::new(centre.x - reach, centre.y + reach));
            }),
        };

        frame.stroke(
            &path,
            Stroke::default()
                .with_color(self.color)
                .with_width(STROKE)
                .with_line_cap(LineCap::Round),
        );
        vec![frame.into_geometry()]
    }
}

/// One glyph, sized to its own box so every control in the row matches.
pub fn glyph<'a>(glyph: WindowGlyph, color: Color) -> Element<'a, Message> {
    iced::widget::canvas(GlyphCanvas { glyph, color })
        .width(Length::Fixed(GLYPH_BOX))
        .height(Length::Fixed(GLYPH_BOX))
        .into()
}

/// The diameter of a window control. Exposed so the header sizes its buttons
/// and this module sizes its glyph from the same source.
pub const CONTROL_SIZE: f32 = theme::ICON_BUTTON_SIZE;


#[cfg(test)]
mod tests {
    use super::*;

    /// Comparing two `const`s directly lets clippy fold the assertion and warn.
    /// Reading them through a runtime slice keeps these as real tests, the same
    /// pattern `theme/tokens.rs` and `view/widgets.rs` already use.
    fn assert_less(name: &str, smaller: f32, larger: f32) {
        let pair = [smaller, larger];
        assert!(
            pair[0] < pair[1],
            "{name}: {} must stay under {}",
            pair[0],
            pair[1]
        );
    }

    #[test]
    fn every_glyph_fits_inside_its_box_with_room_for_the_stroke() {
        // The furthest a path reaches from centre, plus half the stroke's
        // round cap, must stay inside the box or the glyph clips.
        assert_less(
            "cross reach",
            GLYPH_BOX * REACH + STROKE / HALF,
            GLYPH_BOX / HALF,
        );
        assert_less(
            "chevron reach",
            GLYPH_BOX * CHEVRON_REACH + STROKE / HALF,
            GLYPH_BOX / HALF,
        );
    }

    #[test]
    fn the_chevron_is_flatter_than_the_cross() {
        // A chevron as steep as the cross reads as an arrowhead, not a
        // window control.
        assert_less("chevron rise", CHEVRON_RISE, CHEVRON_REACH);
        // and wide enough not to read as a dash
        assert_less("dash guard", REACH, CHEVRON_REACH);
    }

    #[test]
    fn the_glyph_box_is_smaller_than_the_control_it_sits_in() {
        assert_less("glyph box", GLYPH_BOX, CONTROL_SIZE);
    }

    #[test]
    fn the_three_glyphs_are_distinct() {
        let all = [
            WindowGlyph::ChevronDown,
            WindowGlyph::ChevronUp,
            WindowGlyph::Cross,
        ];
        let mut seen = all.to_vec();
        seen.dedup();
        assert_eq!(seen.len(), all.len());
        assert_ne!(WindowGlyph::ChevronDown, WindowGlyph::ChevronUp);
    }
}
