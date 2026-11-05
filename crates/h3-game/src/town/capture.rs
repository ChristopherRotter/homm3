//! The capture of a town as the game composes it (TWN-CAP-001, TWN-CAP-002; E09 T8a2, the town
//! epic's design note §11, ADR-0017's amendment point 7): the step that entered an undefended
//! town, or since E09 T8b1 a siege won (after the winner's levels, TWN-CAP-003), returned it
//! (`AdventureOutcome::capture`), and the game applies it at once.

use h3_adventure::AdventureState;
use h3_contracts::{PlayerId, TownId};
use h3_towns::Towns;

use super::adventure_reveal;
use crate::GameRuleset;
use crate::command::{AdventureSink, GameEvent, Sink};

/// Captures `town` for `player`, whose hero entered it undefended or won its siege: the town's
/// reveal
/// (`h3_towns::reveal`, [`adventure_reveal`]), then the adventure's capture
/// ([`AdventureState::capture_town`]: the owner, the empty garrison, the reveal; `TownCaptured`
/// then `TilesRevealed`), then, only after a capture, the Capitol's limit on the store of towns
/// ([`Towns::apply_capitol_limit`]) with whether `player` owns another town (by the adventure's
/// owners) with a standing Capitol, and [`GameEvent::TownChanged`] when it changed the town, told
/// to the players the capture was told to. One infallible application: the step checked it; a
/// town the store or the map lacks changes nothing (the store holds every town of the map).
// rule: TWN-CAP-001, TWN-CAP-002
pub(crate) fn capture<S: Sink>(
    adventure: &mut AdventureState,
    towns: &mut Towns,
    rules: &GameRuleset,
    (town, player): (TownId, PlayerId),
    sink: &mut S,
) {
    // rule: TWN-CAP-001
    let Some(seen_by) = towns
        .get(town)
        .map(|record| adventure_reveal(record, rules))
        .and_then(|reveal| {
            let events = &mut AdventureSink(&mut *sink);
            adventure.capture_town(rules.core(), town, player, reveal, events)
        })
    else {
        return;
    };
    // rule: TWN-CAP-002
    let (part, map) = (rules.towns(), adventure.map());
    let holds_another = map.towns().iter().any(|&index| {
        adventure.owner(index) == Some(player)
            && map
                .town_id(index)
                .is_some_and(|other| other != town && towns.capitol(other, part))
    });
    if towns.apply_capitol_limit(town, part, holds_another) {
        let capitol = towns.capitol(town, part);
        sink.emit(GameEvent::TownChanged {
            town,
            capitol,
            seen_by,
        });
    }
}
