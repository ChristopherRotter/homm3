//! The Artifact Merchants' stock (TWN-STK-001, -002; E09 T57, the town epic's design note §108,
//! decisions 207, 212 and 214; survey `E09-trades.md` §3): one stock for the whole game, an
//! artifact or nothing per class of the town part's `merchants_stock` (the original's three
//! treasures, three minors and a major at game +0x1F664), drawn on the random artifacts' marks
//! the build left (ADV-OBJ-006, the original's game +0x4E224), which it keeps: at the new game
//! after every other setup draw and on each month's first day from the second, every entry anew
//! whether sold or not. The game calls the draws (`h3_game`'s `town/merchants.rs`); the market
//! at a town reads and empties the entries (E09 T58).

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem;

use h3_content::{ArtifactClass, ArtifactTypeId, Ruleset};
use h3_kernel::{CanonicalHash, Rng, StateHasher};

use crate::state::{AdventureState, ArtifactMarks, ArtifactPool};

/// The merchants' block of the adventure state (E09 T57): the random artifacts' marks and the
/// stock drawn on them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Merchants {
    /// The random artifacts' marks as the build's draws and every stock's since left them.
    marks: ArtifactMarks,
    /// The stock, an entry per class of the last draw's classes in order: an artifact, or none
    /// where the pool gave none (`TWN-QUIRK-21`). Empty until the game's first draw.
    stock: Vec<Option<ArtifactTypeId>>,
}

impl Merchants {
    /// The block of a state just built: the build's `marks` and no stock yet.
    pub(crate) const fn new(marks: ArtifactMarks) -> Self {
        Self {
            marks,
            stock: Vec::new(),
        }
    }
}

impl AdventureState {
    /// The Artifact Merchants' stock (TWN-STK-001, -002; E09 T57): an entry per class of the
    /// town part's `merchants_stock` in order, an artifact or none (a draw that found none,
    /// `TWN-QUIRK-21`); empty before the game's first draw. Every town's and every player's.
    pub fn merchants_stock(&self) -> &[Option<ArtifactTypeId>] {
        &self.merchants().stock
    }

    /// Draws the Artifact Merchants' whole stock anew from `draws` (TWN-STK-001 at the new game,
    /// TWN-STK-002 on a month's first day; E09 T57, the town epic's design note §108): for each of
    /// `classes` in order (the town part's `merchants_stock`) one artifact of that class from the
    /// random artifacts' pool on the marks kept (ADV-OBJ-006's picker: equal chances among the
    /// class's unmarked artifacts, the marks restarting from the bans when none is left, then any
    /// class but Special; the drawn one marked), the old entries replaced whether sold or not and
    /// their marks kept. **`TWN-QUIRK-21`** (T-24 fixed): where even the fallback finds none
    /// (every artifact banned) the entry stays empty; the original's picker loops for ever. With
    /// no class nothing is drawn and the stock is empty.
    // rule: TWN-STK-001, TWN-STK-002
    pub fn draw_merchants(&mut self, core: &Ruleset, classes: &[ArtifactClass], draws: &mut Rng) {
        // A fork's shared block is copied first; the game's own is drawn in place.
        let merchants = Arc::make_mut(self.merchants_mut());
        let mut pool = ArtifactPool::resume(core, mem::take(&mut merchants.marks));
        merchants.stock.clear();
        let stock = classes.iter().map(|&class| pool.draw(&[class], draws));
        merchants.stock.extend(stock);
        merchants.marks = pool.into_marks();
    }

    /// Hashes the merchants' block (the adventure state hash v30, E09 T57): the stock as its
    /// length and each entry (0 for none, else 1 and the artifact), then the marks
    /// ([`ArtifactMarks::hash_into`]).
    pub(crate) fn hash_merchants(&self, h: &mut StateHasher) {
        let merchants = self.merchants();
        h.write_len(merchants.stock.len());
        for entry in &merchants.stock {
            match entry {
                None => h.write_u8(0),
                Some(artifact) => {
                    h.write_u8(1);
                    artifact.hash_into(h);
                }
            }
        }
        merchants.marks.hash_into(h);
    }
}

#[cfg(test)]
mod tests;
