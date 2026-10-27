//! The view's drawers (E24 M1 T3b1, T4a, *provisional*; `docs/epics/E24-design.md` §1.3, §1.5,
//! §2.2): a DEF frame placed by its canvas, mirrored left to right and top to bottom, and clipped
//! to a rectangle of the screen. The battle's drawers (`h3_render::blit`) mirror only left to
//! right and never clip; the map's live here until another screen needs them.
//!
//! - [`raw`]: `draw_frame_raw` (0x47DD40, through `def_draw_raw` 0x47BF50): the terrains, rivers,
//!   roads and the map's edge, and the first half of the unexplored tiles. Literal pixels take
//!   their palette colour (encoding 0's every pixel, indices 0–7 included), runs are skipped
//!   (encoding 2's). The view's DEFs hold encodings 0 and 2 only (T2's gate, §1.9); the original
//!   hands encodings 1 and 3 to other drawers, which this one does not follow.
//! - [`shadows`]: 0x47E820, the second half of the unexplored tiles (`def_draw_raw_shadowed`
//!   0x47BFF0 calls both with the same arguments): runs of index 1 and 2 darken the screen as a
//!   shadow's border, 3 and 4 as its body (E17 design §3.3's masks); everything else is left.
//! - [`flagged`]: `draw_frame_flagged` (0x47D0A0, through 0x47BE10 and 0x47BDC0): the objects'
//!   bodies and the flat objects (passes 4 and 7). Literal pixels take their palette colour, runs
//!   of index 5 (a flag) the colour given, or are skipped without one, as every other run is. The
//!   object DEFs are all encoding 3 (T4a's survey of the user's archives, the radar's DEF aside,
//!   which is no map object).
//! - [`shadow_pass`]: `draw_frame_shadow_pass` (0x47D930, through 0x47BE60): the objects' shadows
//!   (pass 5). Runs of index 1 darken the screen as a shadow's border, of 4 as its body (E17
//!   design §3.3); everything else is left.
//! - [`blend`]: `draw_frame_blend` (0x47D4F0, through `def_draw_group_blend` 0x47C120, which
//!   passes no colour; E24 M2 T3b): the lifted hero's pass 9. Literal pixels are blended half and
//!   half with the screen, `((c >> 1) & H) + ((d >> 1) & H)` with the half mask H (0x6968A4), as
//!   0x47EF60's encoding 1 does (E17 design §3.3); runs of 5 would take a colour given, every
//!   other run is skipped. It reads encoding 3 whatever the frame's (the heroes', flags' and
//!   boats' DEFs are all encoding 3: the gate).
//!
//! The clip: the passes give the drawer a source rectangle of the placed picture (0x411E67 to
//! 0x411ED3 for the roads); `draw_frame_raw` turns it into the frame's own when mirrored (0x47DDD9:
//! `full_width − x − w`), so the rectangle cuts the picture as drawn and a clip on the screen is
//! the same.

use h3_assets::def::Frame;
use h3_render::palette::ScreenPalette;
use h3_render::screen::Screen;

/// Where a frame goes: its canvas's top-left corner, and its mirrors (each across the frame's own
/// canvas, as `draw_frame_raw` reflects it: E17 design §3.4).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Place {
    /// The canvas's left edge.
    pub x: i32,
    /// The canvas's top edge.
    pub y: i32,
    /// Reflected left to right.
    pub mirror_x: bool,
    /// Reflected top to bottom.
    pub mirror_y: bool,
}

/// Draws `frame`'s literal pixels in `palette`'s colours and skips its runs (0x47DD40).
pub fn raw(screen: &mut Screen, frame: &Frame, palette: &ScreenPalette, at: Place, clip: [i32; 4]) {
    each_pixel(frame, at, clip, |x, y, index, run| {
        if !run {
            screen.set(x, y, palette.get(index));
        }
    });
}

/// Darkens the screen under `frame`'s runs of index 1 and 2 as a shadow's border, 3 and 4 as its
/// body (0x47E820).
pub fn shadows(screen: &mut Screen, frame: &Frame, at: Place, clip: [i32; 4]) {
    let format = screen.format();
    each_pixel(frame, at, clip, |x, y, index, run| {
        let shade = match (run, index) {
            (true, 1 | 2) => Some(format.shadow_border(screen.pixel(x, y).unwrap_or(0))),
            (true, 3 | 4) => Some(format.shadow_body(screen.pixel(x, y).unwrap_or(0))),
            _ => None,
        };
        if let Some(pixel) = shade {
            screen.set(x, y, pixel);
        }
    });
}

/// Draws `frame`'s literal pixels in `palette`'s colours and its runs of index 5 in `colour`, if
/// one is given; every other run is skipped (0x47D0A0).
pub fn flagged(
    screen: &mut Screen,
    frame: &Frame,
    palette: &ScreenPalette,
    at: Place,
    clip: [i32; 4],
    colour: Option<u16>,
) {
    each_pixel(frame, at, clip, |x, y, index, run| {
        let pixel = match (run, index, colour) {
            (false, ..) => Some(palette.get(index)),
            (true, 5, Some(c)) => Some(c),
            _ => None,
        };
        if let Some(pixel) = pixel {
            screen.set(x, y, pixel);
        }
    });
}

/// Darkens the screen under `frame`'s runs of index 1 as a shadow's border, 4 as its body
/// (0x47D930).
pub fn shadow_pass(screen: &mut Screen, frame: &Frame, at: Place, clip: [i32; 4]) {
    let format = screen.format();
    each_pixel(frame, at, clip, |x, y, index, run| {
        let shade = match (run, index) {
            (true, 1) => Some(format.shadow_border(screen.pixel(x, y).unwrap_or(0))),
            (true, 4) => Some(format.shadow_body(screen.pixel(x, y).unwrap_or(0))),
            _ => None,
        };
        if let Some(pixel) = shade {
            screen.set(x, y, pixel);
        }
    });
}

/// Blends `frame`'s literal pixels in `palette`'s colours half and half with the screen under
/// them and skips its runs (0x47D4F0 with no colour).
pub fn blend(
    screen: &mut Screen,
    frame: &Frame,
    palette: &ScreenPalette,
    at: Place,
    clip: [i32; 4],
) {
    let half = screen.format().half_mask();
    each_pixel(frame, at, clip, |x, y, index, run| {
        if !run {
            let under = screen.pixel(x, y).unwrap_or(0);
            screen.set(
                x,
                y,
                ((palette.get(index) >> 1) & half) + ((under >> 1) & half),
            );
        }
    });
}

/// Calls `put(x, y, index, run)` for each of `frame`'s pixels inside its canvas, placed by `at`,
/// whose screen position lies inside `clip` (left, top, right, bottom; the last two exclusive).
fn each_pixel(frame: &Frame, at: Place, clip: [i32; 4], mut put: impl FnMut(i64, i64, u8, bool)) {
    let [left, top, right, bottom] = clip.map(i64::from);
    let width = usize::try_from(frame.width).unwrap_or(usize::MAX).max(1);
    let (canvas_w, canvas_h) = (i64::from(frame.full_width), i64::from(frame.full_height));
    for (i, &index) in frame.pixels.iter().enumerate() {
        let run = frame.runs.get(i).copied().unwrap_or(false);
        let cx = i64::from(frame.left) + to_i64(i % width);
        let cy = i64::from(frame.top) + to_i64(i / width);
        if !(0..canvas_w).contains(&cx) || !(0..canvas_h).contains(&cy) {
            continue;
        }
        let sx = if at.mirror_x { canvas_w - 1 - cx } else { cx };
        let sy = if at.mirror_y { canvas_h - 1 - cy } else { cy };
        let (x, y) = (i64::from(at.x) + sx, i64::from(at.y) + sy);
        if (left..right).contains(&x) && (top..bottom).contains(&y) {
            put(x, y, index, run);
        }
    }
}

fn to_i64(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests;
