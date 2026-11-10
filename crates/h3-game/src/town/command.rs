//! The town context's commands as the game runs them (TWN-BLD-001; E09 T18b1, the town epic's
//! design note §29; ADR-0028 point 6): `GameCommand::Town` checks the actor's turn as an
//! adventure command does (`GameState::check_ready`), then this module checks the town, its
//! owner, the hall's tests and the treasury, and only then pays and builds. No hero, no visit, no
//! decision: the original's town list opens the hall (survey `E09-construction.md` §1.1).

use h3_contracts::{PlayerId, TownId};
use h3_towns::{TownCommand, TownCommandError};

use super::holds_capitol;
use super::view::Told;
use crate::command::{GameCommandError, GameEvent, Sink};
use crate::{GameRuleset, GameState};

/// Runs `player`'s town command, whose turn [`GameState::check_ready`] checked. A build
/// (TWN-BLD-001), checked in this order, each refusal changing nothing:
///
/// 1. the town in the store of towns, else [`TownCommandError::UnknownTown`];
/// 2. its owner, the adventure's owner of its object, is `player`, else
///    [`TownCommandError::NotOwner`] (TD-11, E09 decision 42: a visitor of another player's town
///    does not build there);
/// 3. the hall's tests with whether `player` holds another town with a standing Capitol
///    ([`holds_capitol`]): [`h3_towns::check_build`]'s refusal or the building's price;
/// 4. the treasury holds the price, else [`GameCommandError::CannotPay`] (GAM-RES-001).
///
/// Then, infallibly: the price paid (`ResourcePaid` per resource, the part's order), the building
/// built with the day's mark ([`h3_towns::Towns::build`], TWN-BLD-004, -005),
/// [`GameEvent::TownBuilt`] told to `player`, then each view the build changed told to its viewer
/// ([`Told`], E09 T18b2a: the town's by every player that sees it; for a town without a standing
/// Tavern before, the owner's of every town, whose garrison levels a new Tavern raises,
/// TWN-VEW-002). No field of the command is trusted but its two ids: the price is the part's
/// (ADR-0004).
///
/// # Errors
///
/// [`GameCommandError::Town`] with the town's refusal, or [`GameCommandError::CannotPay`].
// rule: TWN-BLD-001
pub(crate) fn command<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    player: PlayerId,
    command: TownCommand,
    sink: &mut S,
) -> Result<(), GameCommandError> {
    let TownCommand::Build { town, building } = command;
    let part = rules.towns();
    let record = state
        .towns()
        .get(town)
        .ok_or(GameCommandError::Town(TownCommandError::UnknownTown))?;
    if owner(state, town) != Some(player) {
        return Err(GameCommandError::Town(TownCommandError::NotOwner));
    }
    let holds = holds_capitol(state.adventure(), state.towns(), part, player, town);
    let price =
        h3_towns::check_build(record, part, building, holds).map_err(GameCommandError::Town)?;
    if !state.can_pay(player, price) {
        return Err(GameCommandError::CannotPay);
    }
    // rule: TWN-VEW-002
    let every: &[PlayerId] = if h3_towns::has_tavern(record, part) {
        &[]
    } else {
        &[player]
    };
    let told = Told::before::<S>(state, rules, &[town], every);
    state.pay_price(player, price, sink);
    // The store has the town and its record the building (both checked above).
    let _before = state.towns_mut().build(town, building, part, true);
    sink.emit(GameEvent::TownBuilt {
        player,
        town,
        building,
        event: None,
    });
    // rule: TWN-VEW-004
    told.tell(state, rules, sink);
    Ok(())
}

/// The owner of `town` (the adventure's owner of its object; E09 increment 1's design note §11),
/// none for a neutral town or a town the map lacks.
fn owner(state: &GameState, town: TownId) -> Option<PlayerId> {
    let adventure = state.adventure();
    adventure
        .map()
        .town_object(town)
        .and_then(|index| adventure.owner(index))
}
