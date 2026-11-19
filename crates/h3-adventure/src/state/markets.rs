//! The markets on the map (E08 T133a1; increment 14's design note part B): a Black Market's stock
//! drawn when the game is built (ADV-MKT-006, design note §9), called by
//! `draws::resolve_rewards`' second walk in object order, and its sold entries in the visits'
//! block (§8). In a module of its own, as the banks' draw (the release build's codegen units,
//! `TRAPS.md`); the visits and the adventure's checks of a trade come with T133a2.

use alloc::vec::Vec;

use h3_kernel::Rng;

use super::draws::ArtifactPool;
use super::{AdventureState, Resolved};
use crate::ruleset::Market;

/// A Black Market's stock (ADV-MKT-006, E08 T133a1; increment 14's design note §9, survey
/// `E08-trades.md` §7.1: `black_market_draw` 0x4C0E31 in `setup_map_objects`' walk): for each
/// class of its kind's stock in order (the base pack's three treasures, three minors and one
/// major), one artifact of that class from the random artifacts' pool (ADV-OBJ-006, the original's
/// `random_artifact_of_level`: the bans and the artifacts given so far marked, so the markets
/// repeat neither each other nor the map's other random artifacts until a class has none left,
/// then the marks restart from the bans); a draw that finds none (every artifact banned) leaves its
/// entry out. Never drawn again: no week, month or visit restocks it (survey §7.1).
// rule: ADV-MKT-006
pub(super) fn stock_draw(
    market: &Market,
    artifacts: &mut ArtifactPool<'_>,
    draws: &mut Rng,
) -> Resolved {
    let stock: Vec<_> = market
        .stock
        .iter()
        .filter_map(|class| artifacts.draw(&[*class], draws))
        .collect();
    Resolved::Stock(stock)
}

impl AdventureState {
    /// Whether the entry `entry` of the Black Market at object index `index` was bought
    /// (ADV-MKT-007; increment 14's design note §8): its sold bit in the visits' block, none
    /// before a purchase.
    pub fn is_sold(&self, index: usize, entry: u8) -> bool {
        self.visits
            .as_ref()
            .is_some_and(|visits| visits.is_sold(index, entry))
    }
}
