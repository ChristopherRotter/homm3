//! What the real-asset gate checks of the DEFs the map's tables and layout name (E24 M1 T2;
//! `docs/epics/E24-design.md` §1.8's open points): how many frames group 0 holds against how many
//! the view draws, the frames' encodings, and the pixels that came from runs, by index. The view's
//! terrain, river and road pictures go through `draw_frame_raw`, which skips runs (§1.3): a run
//! in a terrain frame would leave what the screen held there. The gate's adapter reads the
//! archives; this is pure.

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use h3_assets::Error;
use h3_assets::def::{self, Encoding};

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
    };
    let refs = parsed
        .groups
        .iter()
        .find(|g| g.id == 0)
        .map_or(&[][..], |g| &g.frames[..]);
    for frame_ref in refs {
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

/// What is wrong with a DEF that must hold `frames` frames in group 0: too few, or none at all.
#[must_use]
pub fn problems(summary: &DefSummary, frames: usize) -> Vec<String> {
    let mut found = Vec::new();
    if summary.frames < frames.max(1) {
        found.push(format!(
            "{} frames in group 0, the view draws {}",
            summary.frames,
            frames.max(1)
        ));
    }
    found
}

#[cfg(test)]
mod tests;
