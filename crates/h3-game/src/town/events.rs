//! The town events at a player's turn start (TWN-EVT-001 to -004; E09 T18c, the town epic's design
//! note §30; survey `E09-construction.md` §8): right after the map's timed events, each event of
//! each town the player owns that fires for it today gives its resources, then builds its
//! buildings without the hall's tests and without the day's mark, each told as a build with the
//! views it changed. Its creatures wait for increment 3 (`TWN-STUB-10`).

use alloc::vec::Vec;

use h3_content::ResourceId;
use h3_contracts::{PlayerId, TownId};
use h3_towns::{BuildingSet, TownEvent};

use super::command::build_told;
use crate::command::{GameEvent, Sink};
use crate::setup::Timing;
use crate::{GameRuleset, GameState};

/// Fires the town events at `player`'s turn start (TWN-EVT-001; the original's
/// `town_timed_events` 0x4CDA10 from `player_turn_start` 0x4CC7D0, survey §8.2), called right
/// after the map's timed events: for each town in `TownId` order whose owner is `player` now (the
/// adventure's owner of its object: a neutral town's never fire), each event of its
/// `h3_towns::TownStatic::events` in order that fires for `player` today (the map's events' test,
/// [`Timing`]: its players, its kind by the player's `human`, its first day or every interval
/// after it; TD-12 the original's day):
///
/// 1. its resources (TWN-EVT-002; `apply_timed_event` 0x4CD710): the gains added saturating, then
///    the losses taken down to 0, each told to `player` ([`GameState::gain`], [`GameState::lose`]),
///    then [`GameEvent::TownEventFired`] with the event's place in the town's list;
/// 2. its buildings (TWN-EVT-003; `town_events_apply` 0x5BFEB0, survey §8.3):
///    `h3_towns::event_buildings` with whether the town has its spot (the Shipyard's water,
///    TWN-SET-011), each in index order built without the day's mark and told as a build
///    ([`build_told`]: `TownBuilt` with the event's place, then the views it changed); a building
///    that an earlier one of the same event brought (a horde's upgrade) is not built again. No
///    requirement, forbidden, Capitol, Grail, cost or mark test (T-8: an event's Capitol beside
///    its owner's other one stands; E09 decision 44);
/// 3. its creatures (TWN-EVT-004): none placed yet.
///
/// The events fire at the turn start of the town's owner only, whatever player they name, so a
/// captured town's fire for its new owner when they name it. Day 1's fire inside the game's build
/// for the first group, which keeps what fired ([`GameState::town_events_at_start`]).
// rule: TWN-EVT-001, TWN-EVT-002, TWN-EVT-003, TWN-EVT-004
pub(crate) fn fire_events<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    player: PlayerId,
    sink: &mut S,
) {
    // stub: TWN-STUB-10 (an event's creatures are not added to the town's dwellings: increment 3
    // adds them to the level's upgraded dwelling's recruits, else its dwelling's, else loses them)
    for fired in firing(state, rules, player) {
        state.gain(player, &fired.gains, sink);
        state.lose(player, &fired.losses, sink);
        sink.emit(GameEvent::TownEventFired {
            player,
            town: fired.town,
            event: fired.place,
        });
        for building in fired.buildings.iter() {
            let standing = state
                .towns()
                .get(fired.town)
                .is_some_and(|record| record.built().contains(building));
            if !standing {
                let event = Some(fired.place);
                build_told(state, rules, player, fired.town, building, event, sink);
            }
        }
    }
}

/// A town event that fires, with what it gives and builds.
struct Fired {
    /// Its town.
    town: TownId,
    /// Its place in the town's list.
    place: u16,
    /// Its gains, each resource with its amount, in the event's order.
    gains: Vec<(ResourceId, u32)>,
    /// Its losses, likewise.
    losses: Vec<(ResourceId, u32)>,
    /// Its buildings as `h3_towns::event_buildings` gives them before any event of the turn
    /// start built (an earlier event's building is skipped at its turn, [`fire_events`]: the
    /// only fact of the set that a build changes is the ever-built set).
    buildings: BuildingSet,
}

/// The events that fire for `player` at its turn start today, the towns in `TownId` order and
/// each town's in its order ([`fire_events`]'s test). No town read on a map whose towns have no
/// events, no allocation when none fires.
fn firing(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Vec<Fired> {
    let (true, Some(human)) = (state.has_town_events(), state.human(player)) else {
        return Vec::new();
    };
    let (adventure, towns, day) = (state.adventure(), state.towns(), state.day());
    let map = adventure.map();
    let statics = (0..=u16::MAX)
        .map(TownId::new)
        .map_while(|town| towns.town_static(town).map(|s| (town, s)));
    let mut fired = Vec::new();
    for ((town, statics), &index) in statics.zip(map.towns()) {
        if statics.events.is_empty() || adventure.owner(index) != Some(player) {
            continue;
        }
        let water = map.town_boat_spot(town).is_some();
        for (event, place) in statics.events.iter().zip(0..=u16::MAX) {
            if !timing(event).fires(player, human, day) {
                continue;
            }
            let buildings = towns.get(town).map_or(BuildingSet::EMPTY, |record| {
                h3_towns::event_buildings(record, rules.towns(), event, water)
            });
            fired.push(Fired {
                town,
                place,
                gains: changes(event, true),
                losses: changes(event, false),
                buildings,
            });
        }
    }
    fired
}

/// A town event's day test, the map's timed events' ([`Timing`], TWN-EVT-001).
fn timing(event: &TownEvent) -> Timing<'_> {
    Timing {
        players: &event.players,
        human: event.human,
        computer: event.computer,
        first_day: event.first_day,
        interval: event.interval,
    }
}

/// A town event's gains (`gain`) or losses, each resource with its amount, in the event's order.
fn changes(event: &TownEvent, gain: bool) -> Vec<(ResourceId, u32)> {
    event
        .resources
        .iter()
        .filter(|&&(_, amount)| (amount > 0) == gain)
        .map(|&(resource, amount)| (resource, amount.unsigned_abs()))
        .collect()
}
