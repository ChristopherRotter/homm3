//! The shell's samples as the sound manager plays them (E27 M2 T6a; `docs/epics/E27-design.md`
//! §13): a button's press (`button_press` 0x456860) plays `button.wav` once at volume 64 in
//! category 3, the creature sounds' range of the manager's sample handles (`sample_channels`
//! 0x684AB8: handles 6 to 13), on the first handle whose sample is done, else the range's next in
//! turn, whose sample is cut (`play_sample` 0x59A210; the turn's start and step *inferred*: the
//! range's first, then one on at each use). The harness numbers the handles in the order it first
//! sees them (`xdd.dll`'s `sounds`, its `ch<k>`): [`Played::channel`] is that number, so the
//! viewer's lines are the original's.
//!
//! A menu's music asked for while its track plays is dropped (`play_music` 0x59AD43), so the
//! effects it would stop (0x59ADAC) are stopped only at the first menu, before any press: not
//! modelled.

use alloc::vec::Vec;

/// The handles of category 3.
pub const HANDLES: usize = 8;

/// A sample started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Played {
    /// When, in ms.
    pub time: u32,
    /// The harness's number of its handle.
    pub channel: usize,
    /// Its length in ms.
    pub length: u32,
}

/// Category 3's handles: when each one's sample ends, the next in turn, the harness's numbers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Samples {
    ends: [Option<u32>; HANDLES],
    turn: usize,
    seen: Vec<usize>,
    played: Vec<Played>,
}

impl Samples {
    /// A sample of `length` ms started at `time`.
    pub fn play(&mut self, time: u32, length: u32) -> Played {
        let done = self
            .ends
            .iter()
            .position(|end| end.is_none_or(|e| e <= time));
        let handle = done.unwrap_or_else(|| {
            let handle = self.turn % HANDLES;
            self.turn = handle + 1;
            handle
        });
        // Within the range: a position in it, or the turn wrapped round it.
        self.ends[handle] = Some(time.saturating_add(length));
        let channel = match self.seen.iter().position(|&h| h == handle) {
            Some(k) => k,
            None => {
                self.seen.push(handle);
                self.seen.len() - 1
            }
        };
        let played = Played {
            time,
            channel,
            length,
        };
        self.played.push(played);
        played
    }

    /// Every sample started, in order.
    #[must_use]
    pub fn played(&self) -> &[Played] {
        &self.played
    }
}

#[cfg(test)]
mod tests;
