//! The towns' part of the day's start (TWN-BLD-004, TWN-VEW-004; E09 T18b1, T18b2a; the town
//! epic's design note §26, §30, §31): the day's marks cleared, and the views that changed told
//! (since E09 T26b on a week's first day the week's too, design §47).

use alloc::vec::Vec;

use h3_contracts::TownId;

use super::view::Told;
use crate::command::Sink;
use crate::{GameRuleset, GameState};

/// Starts the towns' day, in the day's advance before the income (TWN-BLD-004; the original's
/// `new_day` towns' loop 0x4C8067): every town's day's mark cleared
/// ([`h3_towns::Towns::start_day`]), then each view of a town built in the day before told to its
/// viewers ([`Told`], E09 T18b2a: the mark is public, TWN-VEW-004; for a sink that keeps
/// nothing the towns are not even listed). On a week's first day `week` holds every viewer's views
/// of every town before the week (E09 T26b): each view that the week's growth, the neutral
/// garrisons' week, the Portals' draws, the month or the mark changed is told (the counts and the
/// Portal are the owner's, TWN-VEW-004; a garrison every viewer's by its level, TWN-VEW-002).
// rule: TWN-BLD-004, TWN-VEW-004
pub(crate) fn start_day<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    week: Option<Told>,
    sink: &mut S,
) {
    let told = match week {
        Some(told) => told,
        None if S::RECORDS => Told::before::<S>(state, rules, &marked(state), &[]),
        None => Told::default(),
    };
    state.towns_mut().start_day();
    told.tell(state, rules, sink);
}

/// The towns built in today, in `TownId` order.
fn marked(state: &GameState) -> Vec<TownId> {
    let count = state.adventure().map().towns().len();
    (0..count)
        .filter_map(|t| u16::try_from(t).ok().map(TownId::new))
        .filter(|&town| state.towns().get(town).is_some_and(|t| t.built_today()))
        .collect()
}
