//! What the real-asset gate checks of the DEFs the map's tables and layout name (E24 M1 T2;
//! `docs/epics/E24-design.md` §1.8's open points): how many frames group 0 holds against how many
//! the view draws, the frames' encodings, and the pixels that came from runs, by index. The view's
//! terrain, river and road pictures go through `draw_frame_raw`, which skips runs (§1.3): a run
//! in a terrain frame would leave what the screen held there. And of the objects' pictures (E24 M1
//! T4b; design §2.2, §2.6, D8): the pixels the object passes never draw with a `.msk`'s shape
//! ([`undrawn`]). The gate's adapter reads the archives; this is pure.

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use h3_assets::Error;
use h3_assets::def::{self, Def, Encoding, Frame, FrameRef};

use crate::objects::Shape;

/// The side of the square an object pass draws of a picture per cell (design §2.2).
const PIECE: i32 = 32;

/// What one DEF's group 0 holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefSummary {
    /// Its frames.
    pub frames: usize,
    /// The encodings its frames use.
    pub encodings: BTreeSet<Encoding>,
    /// The pixels that came from runs, by index (0–6; 7 counts every higher index, which only
    /// encoding 1 can repeat).
    pub run_pixels: [u64; 8],
    /// Every group's id and frames, in the DEF's order (E24 M1 T5a: a hero's picture is its
    /// facing's group).
    pub groups: Vec<(u32, usize)>,
}

impl DefSummary {
    /// The pixels that came from runs of any index.
    #[must_use]
    pub fn runs(&self) -> u64 {
        self.run_pixels.iter().sum()
    }
}

/// Parses a DEF and decodes every frame of its group 0 (the group of id 0; none: no frames).
///
/// # Errors
/// The decoder's: the DEF or one of those frames does not decode.
pub fn summarize(bytes: &[u8]) -> Result<DefSummary, Error> {
    let parsed = def::parse_def(bytes)?;
    let mut summary = DefSummary {
        frames: 0,
        encodings: BTreeSet::new(),
        run_pixels: [0; 8],
        groups: parsed
            .groups
            .iter()
            .map(|g| (g.id, g.frames.len()))
            .collect(),
    };
    for frame_ref in group_0(&parsed) {
        let frame = def::decode_frame(bytes, frame_ref.offset)?;
        summary.frames += 1;
        summary.encodings.insert(frame.encoding);
        for (&index, _) in frame
            .pixels
            .iter()
            .zip(&frame.runs)
            .filter(|(_, run)| **run)
        {
            let slot = &mut summary.run_pixels[usize::from(index.min(7))];
            *slot = slot.saturating_add(1);
        }
    }
    Ok(summary)
}

/// The frames of a DEF's group of id 0 (none without one).
fn group_0(parsed: &Def) -> &[FrameRef] {
    parsed
        .groups
        .iter()
        .find(|g| g.id == 0)
        .map_or(&[][..], |g| &g.frames[..])
}

/// The pixels of an object picture that the object passes never draw (design §2.2, §2.6: D8).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Undrawn {
    /// Body pixels (literal ones and runs of 5, the flag's) in no cell of the colour mask.
    pub body: u64,
    /// Shadow pixels (runs of 1 and 4) in no cell of the shadow mask.
    pub shadow: u64,
}

impl Undrawn {
    /// Whether any pixel is never drawn.
    #[must_use]
    pub fn any(self) -> bool {
        self.body > 0 || self.shadow > 0
    }

    fn add(self, other: Self) -> Self {
        Self {
            body: self.body.saturating_add(other.body),
            shadow: self.shadow.saturating_add(other.shadow),
        }
    }
}

/// The cell of the canvas pixel (x, y) in a picture drawn with `shape`: the passes draw the cell
/// `dx` columns left of and `dy` rows above the anchor from the 32 × 32 square at
/// (32 (w − 1 − dx), 32 (h − 1 − dy)), w × h the `.msk`'s size (design §2.2); none for a pixel
/// outside those squares.
fn cell(shape: &Shape, x: i32, y: i32) -> Option<(u8, u8)> {
    let inside = |d: i32, n: u8| u8::try_from(d).ok().filter(|&d| d < n);
    let dx = i32::from(shape.width) - 1 - x.div_euclid(PIECE);
    let dy = i32::from(shape.height) - 1 - y.div_euclid(PIECE);
    inside(dx, shape.width).zip(inside(dy, shape.height))
}

/// The pixels of `frame` that the passes never draw with `shape`: a body pixel outside the
/// colour mask's cells (pass 7, the flagged drawer: literal pixels and runs of 5), a shadow pixel
/// outside the shadow mask's (pass 5: runs of 1 and 4); other runs are drawn by neither.
#[must_use]
pub fn undrawn_frame(frame: &Frame, shape: &Shape) -> Undrawn {
    let width = usize::try_from(frame.width).unwrap_or(usize::MAX).max(1);
    let mut out = Undrawn::default();
    for (i, (&index, &run)) in frame.pixels.iter().zip(&frame.runs).enumerate() {
        let x = frame
            .left
            .saturating_add(i32::try_from(i % width).unwrap_or(i32::MAX));
        let y = frame
            .top
            .saturating_add(i32::try_from(i / width).unwrap_or(i32::MAX));
        let cell = cell(shape, x, y);
        if !run || index == 5 {
            out.body += u64::from(!cell.is_some_and(|(dx, dy)| shape.body(dx, dy)));
        } else if index == 1 || index == 4 {
            out.shadow += u64::from(!cell.is_some_and(|(dx, dy)| shape.shadow(dx, dy)));
        }
    }
    out
}

/// [`undrawn_frame`] over the frames of a DEF's group 0 (an object DEF's first group, design
/// §2.6), drawn with `shape`.
///
/// # Errors
/// The decoder's: the DEF or one of those frames does not decode.
pub fn undrawn(bytes: &[u8], shape: &Shape) -> Result<Undrawn, Error> {
    let parsed = def::parse_def(bytes)?;
    let mut out = Undrawn::default();
    for frame_ref in group_0(&parsed) {
        let frame = def::decode_frame(bytes, frame_ref.offset)?;
        out = out.add(undrawn_frame(&frame, shape));
    }
    Ok(out)
}

/// What is wrong with a DEF that must hold `frames` frames in group 0 and draws the groups of id
/// 0 to `groups` (exclusive): group 0 with too few frames or none at all, a group missing or with
/// fewer frames than group 0 (the view draws a hero's facing's group at a frame counted by group
/// 0's: E24 M1 T5a, design §2.8).
#[must_use]
pub fn problems(summary: &DefSummary, frames: usize, groups: u32) -> Vec<String> {
    let mut found = Vec::new();
    if summary.frames < frames.max(1) {
        found.push(format!(
            "{} frames in group 0, the view draws {}",
            summary.frames,
            frames.max(1)
        ));
    }
    for id in 1..groups {
        match summary.groups.iter().find(|(g, _)| *g == id) {
            None => found.push(format!("no group {id}, the view draws it")),
            Some(&(_, n)) if n < summary.frames => found.push(format!(
                "{n} frames in group {id}, fewer than group 0's {}",
                summary.frames
            )),
            Some(_) => {}
        }
    }
    found
}

#[cfg(test)]
mod tests;
