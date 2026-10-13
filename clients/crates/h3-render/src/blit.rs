//! Drawing decoded DEF frames and images onto a [`Screen`] as the executable's drawers do
//! (`docs/epics/E17-design.md` §3.3).
//!
//! A frame's pixels are palette indices, each from a run or from literal bytes
//! ([`Frame::runs`]). Literal pixels are colours. A run of a keyed index is an instruction: a
//! shadow darkens the pixel already on the screen ([`PixelFormat::shadow_body`],
//! [`PixelFormat::shadow_border`]), so overlapping shadows compound; an outline or a flag takes a
//! colour the caller gives; the rest are skipped. Which runs do what depends on the drawer the
//! caller picks ([`Drawer`]) and on the frame's encoding, as the executable's drawers branch:
//!
//! | Encoding | [`Drawer::Plain`] | [`Drawer::Shadowed`] | [`Drawer::ShadowPass`] |
//! |---|---|---|---|
//! | 0 | every pixel a colour, 0–7 included (0x47DD40) | the same | literal pixels skipped; runs as below |
//! | 1 | runs in their palette colour, or skipped (0x47C570) | runs 1, 7 border; 4, 6 body; 5 skipped; with a colour 5, 6, 7 take it; others skipped (0x47C9E0) | runs 1 border, 4 body, others skipped |
//! | 2 | runs skipped (0x47DD40) | the same | as above |
//! | 3 | runs of 5 take the colour if one is given, others skipped (0x47D0A0) | the same | as above (0x47D930) |
//!
//! The shadow pass is read for encoding 3 only; for the others it is applied the same way
//! (*provisional*). 0x47C9E0's 50/50 blend of literal pixels is left out: no caller enables it.
//!
//! The frame is placed by its canvas: the canvas's top-left corner goes to the given point, the
//! frame at its own offset inside, and a mirrored frame is reflected across its own canvas's width
//! (the frame's, not the DEF's: design §3.4). Pixels outside the canvas or the screen are clipped.

use h3_assets::def::{Encoding, Frame};
use h3_assets::pcx::Pcx;

use crate::palette::ScreenPalette;
use crate::screen::{PixelFormat, Screen, index};

/// Which of the executable's frame drawers to follow (the table in the module documentation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Drawer {
    /// 0x47C570 (the plain drawer): no shadows. `skip_runs`: encoding 1's runs are skipped instead
    /// of drawn in their palette colour. `colour`: encoding 3's runs of 5 (a flag) take it.
    Plain {
        /// Skip encoding 1's runs.
        skip_runs: bool,
        /// The colour for encoding 3's runs of 5, none to skip them.
        colour: Option<u16>,
    },
    /// 0x47C9E0 (the shadowed drawer; battle stacks): shadows, and the outline or flag `colour`.
    Shadowed {
        /// The colour for runs of 5, 6 and 7 (encoding 1) or 5 (encoding 3), none for no outline.
        colour: Option<u16>,
    },
    /// 0x47D930 (the shadow pass): only the shadows.
    ShadowPass,
}

/// Where a frame goes: its canvas's top-left corner on the screen, and whether it is mirrored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Placement {
    /// The canvas's left edge.
    pub x: i32,
    /// The canvas's top edge.
    pub y: i32,
    /// Reflected left to right across the canvas.
    pub mirrored: bool,
}

/// What one frame pixel does to the screen pixel under it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Skip,
    Index(u8),
    Colour(u16),
    Body,
    Border,
}

fn op(drawer: Drawer, encoding: Encoding, index: u8, run: bool) -> Op {
    match drawer {
        Drawer::ShadowPass => match (run, index) {
            (true, 1) => Op::Border,
            (true, 4) => Op::Body,
            _ => Op::Skip,
        },
        Drawer::Plain { skip_runs, colour } => match encoding {
            Encoding::Rows if run && skip_runs => Op::Skip,
            Encoding::Rows => Op::Index(index),
            _ => other_encodings(encoding, index, run, colour),
        },
        Drawer::Shadowed { colour } => match (encoding, run, index, colour) {
            (Encoding::Rows, true, 5..=7, Some(c)) => Op::Colour(c),
            (Encoding::Rows, true, 1 | 7, _) => Op::Border,
            (Encoding::Rows, true, 4 | 6, _) => Op::Body,
            (Encoding::Rows, true, ..) => Op::Skip,
            _ => other_encodings(encoding, index, run, colour),
        },
    }
}

/// Literal pixels of every encoding, and the runs of encodings 0, 2 and 3 (0x47DD40, 0x47D0A0).
fn other_encodings(encoding: Encoding, index: u8, run: bool, colour: Option<u16>) -> Op {
    match (encoding, run, index, colour) {
        (_, false, ..) | (Encoding::Raw, ..) => Op::Index(index),
        (Encoding::PackedBlocks, true, 5, Some(c)) => Op::Colour(c),
        _ => Op::Skip,
    }
}

fn apply(op: Op, pixel: &mut u16, palette: &ScreenPalette, format: PixelFormat) {
    *pixel = match op {
        Op::Skip => *pixel,
        Op::Index(i) => palette.get(i),
        Op::Colour(c) => c,
        Op::Body => format.shadow_body(*pixel),
        Op::Border => format.shadow_border(*pixel),
    };
}

/// Draws `frame` with `palette` (in the screen's format) as `drawer` does, placed by `at`.
pub fn draw_frame(
    screen: &mut Screen,
    frame: &Frame,
    palette: &ScreenPalette,
    at: Placement,
    drawer: Drawer,
) {
    let format = screen.format();
    let width = index(frame.width).max(1);
    let (canvas_w, canvas_h) = (i64::from(frame.full_width), i64::from(frame.full_height));
    for (i, &pixel_index) in frame.pixels.iter().enumerate() {
        let run = frame.runs.get(i).copied().unwrap_or(false);
        let cx = i64::from(frame.left) + to_i64(i % width);
        let cy = i64::from(frame.top) + to_i64(i / width);
        if !(0..canvas_w).contains(&cx) || !(0..canvas_h).contains(&cy) {
            continue;
        }
        let sx = if at.mirrored { canvas_w - 1 - cx } else { cx };
        if let Some(pixel) = screen.pixel_mut(i64::from(at.x) + sx, i64::from(at.y) + cy) {
            apply(
                op(drawer, frame.encoding, pixel_index, run),
                pixel,
                palette,
                format,
            );
        }
    }
}

/// Draws an image opaque with its top-left corner at `(x, y)`: an 8-bit one through its palette,
/// a 24-bit one pixel by pixel, both truncated to the screen's format (0x5226D0, 0x44F010). How
/// the executable draws images (whether an index is ever transparent) is not read yet
/// (*provisional*, design §3.5).
pub fn draw_image(screen: &mut Screen, image: &Pcx, x: i32, y: i32) {
    put_image(screen, image, x, y, false);
}

/// Draws an image as [`draw_image`] does, but an 8-bit one's index 0 is transparent: the battle's
/// absolute obstacle goes into the background so (0x44FC70, design §4.4). A 24-bit image has no
/// index and is drawn opaque (*provisional*: no absolute obstacle is one).
pub fn draw_image_keyed(screen: &mut Screen, image: &Pcx, x: i32, y: i32) {
    put_image(screen, image, x, y, true);
}

fn put_image(screen: &mut Screen, image: &Pcx, x: i32, y: i32, keyed: bool) {
    let format = screen.format();
    let width = index(image.width()).max(1);
    let mut put = |i: usize, pixel: u16| {
        let (px, py) = (to_i64(i % width), to_i64(i / width));
        if let Some(p) = screen.pixel_mut(i64::from(x) + px, i64::from(y) + py) {
            *p = pixel;
        }
    };
    match image {
        Pcx::Indexed(image) => {
            let palette = ScreenPalette::new(&image.palette, format);
            for (i, &v) in image.pixels.iter().enumerate() {
                if !(keyed && v == 0) {
                    put(i, palette.get(v));
                }
            }
        }
        Pcx::Rgb(image) => {
            for (i, &c) in image.pixels.iter().enumerate() {
                put(i, format.pack(c));
            }
        }
    }
}

fn to_i64(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests;
