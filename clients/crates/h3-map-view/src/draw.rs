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
//! Each drawer visits only a frame's [`Spans`] (E24 M3 T2b2): its rows' literal pixels and runs,
//! worked out once a frame, not its transparent pixels.
//!
//! The clip: the passes give the drawer a source rectangle of the placed picture (0x411E67 to
//! 0x411ED3 for the roads); `draw_frame_raw` turns it into the frame's own when mirrored (0x47DDD9:
//! `full_width − x − w`), so the rectangle cuts the picture as drawn and a clip on the screen is
//! the same.

use alloc::vec::Vec;

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

/// A frame's rows as the drawers visit them (E24 M3 T2b2, *provisional*; design §4.3): each row's
/// pieces of literal pixels and of runs of one index, in order, worked out once when the frame is
/// decoded ([`crate::batch::Batch`]); its runs of index 0, which no drawer draws, are left out, so
/// a drawer never visits a transparent pixel. A pixel past the end of the frame's runs is literal
/// (a decoded frame has a run flag for every pixel).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Spans {
    /// Row `r`'s spans: `spans[starts[r]..starts[r + 1]]`.
    starts: Vec<u32>,
    spans: Vec<Span>,
}

/// One piece of a row: its columns `from` to `to` (exclusive), and what they hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Span {
    from: u32,
    to: u32,
    kind: Kind,
}

/// What a span's pixels are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// Literal pixels: the frame's indices, each its palette colour.
    Literal,
    /// A run's pixels of this index (never 0).
    Run(u8),
}

impl Spans {
    /// `frame`'s rows as spans.
    #[must_use]
    pub fn of(frame: &Frame) -> Self {
        let width = usize::try_from(frame.width).unwrap_or(usize::MAX).max(1);
        let mut starts = Vec::with_capacity(frame.pixels.len() / width + 2);
        let mut spans = Vec::new();
        for (r, row) in frame.pixels.chunks(width).enumerate() {
            starts.push(to_u32(spans.len()));
            let first = r.saturating_mul(width);
            let mut open: Option<Span> = None;
            for (c, &index) in row.iter().enumerate() {
                let run = frame.runs.get(first + c).copied().unwrap_or(false);
                let kind = match (run, index) {
                    (false, _) => Some(Kind::Literal),
                    (true, 0) => None,
                    (true, i) => Some(Kind::Run(i)),
                };
                let c = to_u32(c);
                match (&mut open, kind) {
                    (Some(span), Some(kind)) if span.kind == kind => span.to = c + 1,
                    (_, kind) => {
                        spans.extend(open.take());
                        open = kind.map(|kind| Span {
                            from: c,
                            to: c + 1,
                            kind,
                        });
                    }
                }
            }
            spans.extend(open);
        }
        starts.push(to_u32(spans.len()));
        Self { starts, spans }
    }

    /// Row `r`'s spans (none past the last row).
    fn row(&self, r: usize) -> &[Span] {
        let at = |i: usize| self.starts.get(i).map_or(0, |&s| to_usize(i64::from(s)));
        self.spans.get(at(r)..at(r + 1)).unwrap_or_default()
    }
}

/// A frame and its spans, as the drawers take them.
pub type FrameSpans<'a> = (&'a Frame, &'a Spans);

/// Draws `frame`'s literal pixels in `palette`'s colours and skips its runs (0x47DD40).
pub fn raw(
    screen: &mut Screen,
    frame: FrameSpans<'_>,
    palette: &ScreenPalette,
    at: Place,
    clip: [i32; 4],
) {
    each_span(screen, frame, at, clip, |dst, src, kind, mirrored| {
        if kind == Kind::Literal {
            literal(dst, src, mirrored, palette, |_, colour| colour);
        }
    });
}

/// Darkens the screen under `frame`'s runs of index 1 and 2 as a shadow's border, 3 and 4 as its
/// body (0x47E820).
pub fn shadows(screen: &mut Screen, frame: FrameSpans<'_>, at: Place, clip: [i32; 4]) {
    let shade = Shade::of(screen);
    each_span(screen, frame, at, clip, |dst, _, kind, _| match kind {
        Kind::Run(1 | 2) => dst.iter_mut().for_each(|p| *p = shade.border(*p)),
        Kind::Run(3 | 4) => dst.iter_mut().for_each(|p| *p = shade.body(*p)),
        _ => {}
    });
}

/// A shadow's masks worked out once a draw, not once a pixel (E24 M3 T2b1): what
/// [`PixelFormat::shadow_body`](h3_render::screen::PixelFormat::shadow_body) and `shadow_border`
/// compute.
#[derive(Debug, Clone, Copy)]
struct Shade {
    half: u16,
    quarter: u16,
}

impl Shade {
    fn of(screen: &Screen) -> Self {
        let format = screen.format();
        Self {
            half: format.half_mask(),
            quarter: format.quarter_mask(),
        }
    }

    /// Each channel halved.
    fn body(self, pixel: u16) -> u16 {
        (pixel >> 1) & self.half
    }

    /// Each channel's quarter plus its half.
    fn border(self, pixel: u16) -> u16 {
        ((pixel >> 2) & self.quarter) + ((pixel >> 1) & self.half)
    }
}

/// Draws `frame`'s literal pixels in `palette`'s colours and its runs of index 5 in `colour`, if
/// one is given; every other run is skipped (0x47D0A0).
pub fn flagged(
    screen: &mut Screen,
    frame: FrameSpans<'_>,
    palette: &ScreenPalette,
    at: Place,
    clip: [i32; 4],
    colour: Option<u16>,
) {
    each_span(screen, frame, at, clip, |dst, src, kind, mirrored| {
        match (kind, colour) {
            (Kind::Literal, _) => literal(dst, src, mirrored, palette, |_, c| c),
            (Kind::Run(5), Some(c)) => dst.fill(c),
            _ => {}
        }
    });
}

/// Darkens the screen under `frame`'s runs of index 1 as a shadow's border, 4 as its body
/// (0x47D930).
pub fn shadow_pass(screen: &mut Screen, frame: FrameSpans<'_>, at: Place, clip: [i32; 4]) {
    let shade = Shade::of(screen);
    each_span(screen, frame, at, clip, |dst, _, kind, _| match kind {
        Kind::Run(1) => dst.iter_mut().for_each(|p| *p = shade.border(*p)),
        Kind::Run(4) => dst.iter_mut().for_each(|p| *p = shade.body(*p)),
        _ => {}
    });
}

/// Blends `frame`'s literal pixels in `palette`'s colours half and half with the screen under
/// them and skips its runs (0x47D4F0 with no colour).
pub fn blend(
    screen: &mut Screen,
    frame: FrameSpans<'_>,
    palette: &ScreenPalette,
    at: Place,
    clip: [i32; 4],
) {
    let half = screen.format().half_mask();
    each_span(screen, frame, at, clip, |dst, src, kind, mirrored| {
        if kind == Kind::Literal {
            literal(dst, src, mirrored, palette, |p, c| {
                ((c >> 1) & half) + ((p >> 1) & half)
            });
        }
    });
}

/// Puts literal indices `src` in `palette`'s colours onto `dst` through `mix` (the screen's pixel
/// and the colour), the last first when mirrored.
fn literal(
    dst: &mut [u16],
    src: &[u8],
    mirrored: bool,
    palette: &ScreenPalette,
    mix: impl Fn(u16, u16) -> u16,
) {
    if mirrored {
        for (p, &index) in dst.iter_mut().rev().zip(src) {
            *p = mix(*p, palette.get(index));
        }
    } else {
        for (p, &index) in dst.iter_mut().zip(src) {
            *p = mix(*p, palette.get(index));
        }
    }
}

/// Hands `put` each span of `frame` placed at `at`, cut to its canvas, `clip` and the screen: the
/// screen's pixels under it (left to right on the screen), the frame's indices there (in the
/// frame's order), its kind, and whether it is mirrored left to right. Row by row (E24 M2 T4b3,
/// *provisional*): each row's columns worked out once from the canvas, the clip and the screen,
/// then only its spans visited (E24 M3 T2b2).
fn each_span(
    screen: &mut Screen,
    (frame, spans): FrameSpans<'_>,
    at: Place,
    clip: [i32; 4],
    mut put: impl FnMut(&mut [u16], &[u8], Kind, bool),
) {
    let (screen_w, screen_h) = (i64::from(screen.width()), i64::from(screen.height()));
    let [left, top, right, bottom] = clip.map(i64::from);
    let (left, right) = (left.max(0), right.min(screen_w));
    let (top, bottom) = (top.max(0), bottom.min(screen_h));
    let width = usize::try_from(frame.width).unwrap_or(usize::MAX).max(1);
    let (canvas_w, canvas_h) = (i64::from(frame.full_width), i64::from(frame.full_height));
    let (frame_x, frame_y) = (i64::from(frame.left), i64::from(frame.top));
    // The screen's x of the row's column c: base + c, or base − c mirrored.
    let base = if at.mirror_x {
        i64::from(at.x) + canvas_w - 1 - frame_x
    } else {
        i64::from(at.x) + frame_x
    };
    let pixels = screen.pixels_mut();
    for (r, row) in frame.pixels.chunks(width).enumerate() {
        let cy = frame_y + to_i64(r);
        let sy = if at.mirror_y { canvas_h - 1 - cy } else { cy };
        let y = i64::from(at.y) + sy;
        if !(0..canvas_h).contains(&cy) || !(top..bottom).contains(&y) {
            continue;
        }
        // The columns on the canvas, then those whose x lies in [left, right).
        let (on_from, on_to) = (
            (-frame_x).max(0),
            (canvas_w - frame_x).min(to_i64(row.len())),
        );
        let (from, to) = if at.mirror_x {
            (on_from.max(base - right + 1), on_to.min(base - left + 1))
        } else {
            (on_from.max(left - base), on_to.min(right - base))
        };
        if from >= to {
            continue;
        }
        for span in spans.row(r) {
            let (a, b) = (i64::from(span.from).max(from), i64::from(span.to).min(to));
            if a >= b {
                continue;
            }
            // The screen's pixels from the span's leftmost x on.
            let x = if at.mirror_x { base - b + 1 } else { base + a };
            let start = to_usize(y * screen_w + x);
            let n = to_usize(b - a);
            let dst = pixels.get_mut(start..start + n).unwrap_or_default();
            let src = row.get(to_usize(a)..to_usize(b)).unwrap_or_default();
            put(dst, src, span.kind, at.mirror_x);
        }
    }
}

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn to_i64(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/// A place worked out to lie on the screen, as an index.
fn to_usize(value: i64) -> usize {
    usize::try_from(value).unwrap_or(0)
}

#[cfg(test)]
mod tests;
