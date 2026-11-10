//! The capture of a town as the game composes it (TWN-CAP-001, TWN-CAP-002; E09 T8a2, the town
//! epic's design note §11, ADR-0017's amendment point 7): the step that entered an undefended
//! town, or since E09 T8b1 a siege won (after the winner's levels, TWN-CAP-003), returned it
//! (`AdventureOutcome::capture`), and the game applies it at once.

use alloc::vec::Vec;

use h3_adventure::AdventureState;
use h3_contracts::{PlayerId, TownId};
use h3_towns::{TownRuleset, Towns};

use super::adventure_reveal;
use super::view::Told;
use crate::command::{AdventureSink, Sink};
use crate::{GameRuleset, GameState};

/// Captures `town` for `player`, whose hero entered it undefended or won its siege: the town's
/// reveal (`h3_towns::reveal`, [`adventure_reveal`]), then the adventure's capture
/// ([`AdventureState::capture_town`]: the owner, the empty garrison, the reveal; `TownCaptured`
/// then `TilesRevealed`), then, only after a capture, the day's mark's rule (since E09 T18b1,
/// decision 43: [`Towns::captured`] clears it when the old owner, read before the capture, is
/// not human or the town was neutral, and `player` is; otherwise it is kept, so a town built in
/// today by a human is not built in again today by the human who took it), then the Capitol's
/// limit on the store of towns ([`Towns::apply_capitol_limit`]) with whether `player` owns another
/// town (by the adventure's owners) with a standing Capitol ([`holds_capitol`]), then each view
/// the capture changed told to its viewer (E09 T18b2a, [`Told`]: the town's by every player that
/// sees it, its owner, mark, Capitol and garrison; for a town with a standing Tavern, the old and
/// the new owner's of every town, whose garrison levels it counts, TWN-VEW-002; E09 T8a2 told the
/// Capitol's limit alone). One infallible application: the step checked it; a town the store or
/// the map lacks changes nothing (the store holds every town of the map). A team is its player
/// alone until E10 (`h3_adventure::allied`).
// rule: TWN-CAP-001, TWN-CAP-002, TWN-BLD-004, TWN-VEW-002
pub(crate) fn capture<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    (town, player): (TownId, PlayerId),
    sink: &mut S,
) {
    let part = rules.towns();
    let adventure = state.adventure();
    let old_owner = adventure
        .map()
        .town_object(town)
        .and_then(|index| adventure.owner(index));
    let Some(record) = state.towns().get(town) else {
        return;
    };
    let reveal = adventure_reveal(record, rules);
    // rule: TWN-VEW-002
    let told = if S::RECORDS {
        let every: Vec<PlayerId> = if h3_towns::has_tavern(record, part) {
            old_owner.into_iter().chain([player]).collect()
        } else {
            Vec::new()
        };
        Told::before::<S>(state, rules, &[town], &every)
    } else {
        Told::default()
    };
    // rule: TWN-CAP-001
    let (adventure, _) = state.adventure_with_heroes();
    let events = &mut AdventureSink(&mut *sink);
    if adventure
        .capture_town(rules.core(), town, player, reveal, events)
        .is_none()
    {
        return;
    }
    // rule: TWN-BLD-004
    let human = |p: PlayerId| state.human(p) == Some(true);
    let mark = !old_owner.is_some_and(human) && human(player);
    state.towns_mut().captured(town, mark);
    // rule: TWN-CAP-002
    let holds_another = holds_capitol(state.adventure(), state.towns(), part, player, town);
    state
        .towns_mut()
        .apply_capitol_limit(town, part, holds_another);
    told.tell(state, rules, sink);
}

/// Whether `player` owns a town other than `town` (by the adventure's owners) with a standing
/// Capitol (TWN-CAP-002, TWN-BLD-002: one Capitol a player, the original's `player_has_capitol`
/// 0x4B9F40): the capture's limit and the build's test (E09 T18b1) read it alike.
pub(crate) fn holds_capitol(
    adventure: &AdventureState,
    towns: &Towns,
    part: &TownRuleset,
    player: PlayerId,
    town: TownId,
) -> bool {
    let map = adventure.map();
    map.towns().iter().any(|&index| {
        adventure.owner(index) == Some(player)
            && map
                .town_id(index)
                .is_some_and(|other| other != town && towns.capitol(other, part))
    })
}
