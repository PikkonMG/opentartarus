//! Draws `packaging/icons/opentartarus-64.png`.
//!
//! Run with `cargo run -p opentartarus-daemon --example make_icon`.
//! The same numbers appear in `packaging/icons/opentartarus.svg`; change both
//! together or the tray icon and the desktop icon will drift apart.

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// Final icon edge, in pixels.
const SIZE: usize = 64;
/// Supersampling factor. The icon is drawn at SIZE * SS then box-filtered down,
/// so the rounded corners are smooth without a rasteriser.
const SS: usize = 4;
const BIG: usize = SIZE * SS;
const CHANNELS: usize = 4;

/// Geometry, in units of the 64-pixel edge.
const CORNER_RADIUS: f32 = 14.0;
const STEM_LEFT: f32 = 28.0;
const STEM_RIGHT: f32 = 36.0;
const BAR_LEFT: f32 = 18.0;
const BAR_RIGHT: f32 = 46.0;
const MARK_TOP: f32 = 20.0;
const BAR_BOTTOM: f32 = 28.0;
const STEM_BOTTOM: f32 = 46.0;

const ACCENT: [u8; 3] = [0x35, 0x84, 0xe4];
const MARK: [u8; 3] = [0xff, 0xff, 0xff];
const OPAQUE: u8 = 255;
const CLEAR: u8 = 0;

/// True when the point is inside a rounded square covering the whole edge.
fn inside_rounded_square(x: f32, y: f32, edge: f32, radius: f32) -> bool {
    let clamped_x = x.max(radius).min(edge - radius);
    let clamped_y = y.max(radius).min(edge - radius);
    let dx = x - clamped_x;
    let dy = y - clamped_y;
    dx * dx + dy * dy <= radius * radius
}

fn inside_rect(x: f32, y: f32, left: f32, top: f32, right: f32, bottom: f32) -> bool {
    x >= left && x < right && y >= top && y < bottom
}

fn main() {
    let edge = SIZE as f32;
    let mut big = vec![0u8; BIG * BIG * CHANNELS];
    for row in 0..BIG {
        for column in 0..BIG {
            // Sample at the centre of the supersampled pixel, in icon units.
            let x = (column as f32 + 0.5) / SS as f32;
            let y = (row as f32 + 0.5) / SS as f32;
            let pixel = if !inside_rounded_square(x, y, edge, CORNER_RADIUS) {
                [CLEAR, CLEAR, CLEAR, CLEAR]
            } else if inside_rect(x, y, BAR_LEFT, MARK_TOP, BAR_RIGHT, BAR_BOTTOM)
                || inside_rect(x, y, STEM_LEFT, MARK_TOP, STEM_RIGHT, STEM_BOTTOM)
            {
                [MARK[0], MARK[1], MARK[2], OPAQUE]
            } else {
                [ACCENT[0], ACCENT[1], ACCENT[2], OPAQUE]
            };
            let offset = (row * BIG + column) * CHANNELS;
            big[offset..offset + CHANNELS].copy_from_slice(&pixel);
        }
    }

    // Box filter down to the final size, averaging in premultiplied space so
    // transparent corners do not bleed dark edges.
    let samples = (SS * SS) as u32;
    let mut small = vec![0u8; SIZE * SIZE * CHANNELS];
    for row in 0..SIZE {
        for column in 0..SIZE {
            let mut totals = [0u32; CHANNELS];
            for sub_row in 0..SS {
                for sub_column in 0..SS {
                    let source = ((row * SS + sub_row) * BIG + column * SS + sub_column) * CHANNELS;
                    let alpha = big[source + 3] as u32;
                    totals[0] += big[source] as u32 * alpha / OPAQUE as u32;
                    totals[1] += big[source + 1] as u32 * alpha / OPAQUE as u32;
                    totals[2] += big[source + 2] as u32 * alpha / OPAQUE as u32;
                    totals[3] += alpha;
                }
            }
            let alpha = totals[3] / samples;
            // `checked_div` folds the "would divide by zero" guard into the
            // division itself, rather than branching on `alpha == 0` by hand.
            let unpremultiply = |total: u32| -> u8 {
                ((total / samples) * OPAQUE as u32)
                    .checked_div(alpha)
                    .map_or(0, |value| value.min(OPAQUE as u32) as u8)
            };
            let target = (row * SIZE + column) * CHANNELS;
            small[target] = unpremultiply(totals[0]);
            small[target + 1] = unpremultiply(totals[1]);
            small[target + 2] = unpremultiply(totals[2]);
            small[target + 3] = alpha as u8;
        }
    }

    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packaging/icons/opentartarus-64.png");
    std::fs::create_dir_all(path.parent().expect("icons directory")).expect("create icons dir");
    let file = File::create(&path).expect("create the PNG");
    let mut encoder = png::Encoder::new(BufWriter::new(file), SIZE as u32, SIZE as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .expect("write the PNG header")
        .write_image_data(&small)
        .expect("write the PNG pixels");
    println!("wrote {}", path.display());
}
