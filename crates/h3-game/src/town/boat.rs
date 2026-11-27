//! The town's Shipyard in the game (TWN-SHY-001; E09 T59a, the town epic's design note §111;
//! survey `E09-trades.md` §5): `TownCommand::BuildBoat` builds a boat on the town's spot for the
//! part's price, without a hero, and the legal listing offers it where `apply` accepts it.

use h3_contracts::{PlayerId, TownId};
use h3_towns::TownCommandError;

use super::command::owner;
use crate::command::{AdventureSink, GameCommandError, Sink};
use crate::{GameRuleset, GameState};

/// Builds a boat at `player`'s `town`'s Shipyard (TWN-SHY-001; E09 T59a, design §111; survey §5.2
/// to §5.4), `player`'s turn without an open decision checked by `GameState::check_ready`, in this
/// order, each refusal changing nothing:
///
/// 1. the town in the store of towns, else [`TownCommandError::UnknownTown`];
/// 2. its owner `player`, else [`TownCommandError::NotOwner`] (the original tests no owner, its
///    viewing player builds and pays: an ally's town waits for E10's teams, `TWN-STUB-21`);
/// 3. [`h3_towns::check_boat`]: a standing Shipyard, else [`TownCommandError::NotStanding`]; the
///    price and the boat's type, the part's;
/// 4. the town's spot free ([`h3_adventure::AdventureState::free_town_boat_spot`]: no empty boat,
///    no hero), else [`TownCommandError::BoatSpotTaken`] (**`TWN-QUIRK-19`**, T-21 fixed: the
///    original builds over them);
/// 5. `player`'s treasury holds the price, else [`GameCommandError::CannotPay`].
///
/// Then, infallibly: the boat launched on the spot, its builder `player`
/// (`AdventureState::build_town_boat`, `BoatBuilt` by the fog's rules), then the price paid
/// (`ResourcePaid` per resource; a map Shipyard's order, ADV-SHY-003). No hero, any time of the
/// turn, several a day while the spot frees, no mark. The ship (20) is the spot's occupancy, never
/// a building of the record (T-27). No field of the command is trusted but the town's id.
///
/// # Errors
///
/// [`GameCommandError::Town`] with the refusal, or [`GameCommandError::CannotPay`].
// rule: TWN-SHY-001
pub(crate) fn build<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    player: PlayerId,
    town: TownId,
    sink: &mut S,
) -> Result<(), GameCommandError> {
    let record = state
        .towns()
        .get(town)
        .ok_or(GameCommandError::Town(TownCommandError::UnknownTown))?;
    if owner(state, town) != Some(player) {
        return Err(GameCommandError::Town(TownCommandError::NotOwner));
    }
    let sale = h3_towns::check_boat(record, rules.towns()).map_err(GameCommandError::Town)?;
    let _spot = state
        .adventure()
        .free_town_boat_spot(town)
        .ok_or(GameCommandError::Town(TownCommandError::BoatSpotTaken))?;
    if !state.can_pay(player, sale.price) {
        return Err(GameCommandError::CannotPay);
    }
    let (adventure, _) = state.adventure_with_heroes();
    let boat = (town, sale.boat_type);
    let _built =
        adventure.build_town_boat(rules.core(), boat, player, &mut AdventureSink(&mut *sink));
    state.pay_price(player, sale.price, sink);
    Ok(())
}

/// Whether [`crate::legal_commands`] lists the boat at `player`'s own `town` (TWN-SHY-001; E09
/// T59a, design §115), as [`build`] accepts it: a standing Shipyard, the price in `player`'s
/// treasury, the spot free; the cheapest test first (the record's derived roles, then the
/// treasury, then the walk over the heroes on the map).
// rule: TWN-SHY-001
pub(crate) fn listed(
    state: &GameState,
    rules: &GameRuleset,
    (town, player): (TownId, PlayerId),
) -> bool {
    state
        .towns()
        .get(town)
        .and_then(|record| h3_towns::check_boat(record, rules.towns()).ok())
        .is_some_and(|sale| {
            state.can_pay(player, sale.price)
                && state.adventure().free_town_boat_spot(town).is_some()
        })
}
