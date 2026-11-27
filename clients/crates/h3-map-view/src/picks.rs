//! The client's own random picks in play (E24 M5 T4b, *provisional*; `docs/epics/E24-design.md`
//! §5.27; M4's review item 2, the user at M5 T0): where the original draws at random for its own
//! drawing (a pickup's sound, `random_between(1, 7)` at 0x4A4D18; a computer turn's music,
//! `random_between(1, 3)`; an unnamed town's name at the new game), a recording and a record take
//! the draws in turn (the same each run), and play draws them from a generator of its own seeded
//! by the game: [`Picks`], splitmix64, no float. None of them is a rule: the engine never sees
//! them.

use alloc::vec::Vec;

use crate::live::TOWN_NAMES;

/// The client's generator of picks (splitmix64), a plain value: a loop run again from a kept
/// state draws the same.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Picks {
    state: u64,
}

impl Picks {
    /// The generator seeded by `seed`.
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// The next draw.
    pub const fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A draw below `n` (0 for `n` 0).
    pub fn below(&mut self, n: usize) -> usize {
        let n64 = u64::try_from(n).unwrap_or(u64::MAX);
        match n64 {
            0 => 0,
            _ => usize::try_from(self.next() % n64).unwrap_or(0),
        }
    }
}

/// The name of an unnamed town in play (E24 M5 T4b): its record of `TownName.txt` (the
/// catalogue's `town-name` family) among its faction's sixteen, by the game's `seed`: the
/// faction's names in an order drawn for the faction, the town's by its id, so every view of the
/// game names it alike and two of a faction's towns apart by fewer than sixteen ids differ
/// (*provisional*: the original draws unused names at the new game). Without a seed (a record's,
/// a recording's), the faction's first.
#[must_use]
pub fn town_name(faction: u16, town: u16, seed: Option<u64>) -> u16 {
    let first = faction.saturating_mul(TOWN_NAMES);
    let Some(seed) = seed else {
        return first;
    };
    let mut order: Vec<u16> = (0..TOWN_NAMES).collect();
    let mut picks = Picks::new(seed ^ u64::from(faction));
    for i in (1..order.len()).rev() {
        order.swap(i, picks.below(i + 1));
    }
    let k = usize::from(town) % order.len();
    first.saturating_add(order[k])
}

#[cfg(test)]
mod tests;
