//! The Blacksmith and the Stronghold's Ballista Yard in the game (TWN-SMT-001; E09 T59a, the town
//! epic's design note §110, decision 211; survey `E09-trades.md` §4): `TownCommand::BuyWarMachine`
//! buys the war machine a standing building sells for the town's hero at the price of its
//! creature, as one command (the original's own Buy-or-Cancel dialog, `town_blacksmith_buy`
//! 0x5D1D30); the legal listing offers each purchase `apply` accepts (`town::services`).

use h3_content::{ArtifactTypeId, Ruleset};
use h3_contracts::{PlayerId, TownId};
use h3_heroes::{HeroChange, HeroDetails};
use h3_towns::{BuildingId, TownCommandError};

use super::command::owner;
use super::market::window_hero;
use crate::command::{GameCommandError, Sink};
use crate::{GameRuleset, GameState};

/// Buys the war machine `building` of `player`'s `town` sells (TWN-SMT-001; E09 T59a, design
/// §110; survey §4.2 to §4.6), `player`'s turn without an open decision checked by
/// `GameState::check_ready`, in this order, each refusal changing nothing:
///
/// 1. the town in the store of towns, else [`TownCommandError::UnknownTown`];
/// 2. its owner `player`, else [`TownCommandError::NotOwner`] (the original tests no owner: an
///    ally's town waits for E10's teams, `TWN-STUB-21`);
/// 3. [`h3_towns::check_war_machine`]: the building of the record, selling a war machine,
///    standing, else [`TownCommandError::UnknownBuilding`], [`TownCommandError::NoWarMachine`] or
///    [`TownCommandError::NotStanding`]; the machine and its price;
/// 4. the town's hero, the market window's ([`window_hero`]: the visitor, `player`'s; with
///    `TWN-QUIRK-15`'s flag off the garrison hero when no visitor stands there, T-28), else
///    [`TownCommandError::NoVisitor`] (the original's GENRLTXT 274);
/// 5. the hero holds no such machine, worn or carried, and has a position free for it
///    ([`receives`]), else [`TownCommandError::HasWarMachine`] (the original's Buy disabled);
/// 6. `player`'s treasury holds the price, else [`GameCommandError::CannotPay`] (every resource
///    of it, `TWN-QUIRK-22`).
///
/// Then, infallibly: the price paid (`ResourcePaid` per resource, the creature's order), the
/// machine's artifact given to the hero (`HeroChange::AddArtifact` into its position; `HeroChanged`
/// to `player`). No day mark, no stock, no movement point spent, any number a turn: the next
/// purchase finds the hero holding the machine. No field of the command is trusted but its two ids:
/// the machine and the price are the part's and the core's (ADR-0004).
///
/// # Errors
///
/// [`GameCommandError::Town`] with the refusal, or [`GameCommandError::CannotPay`].
// rule: TWN-SMT-001
pub(crate) fn buy<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    player: PlayerId,
    (town, building): (TownId, BuildingId),
    sink: &mut S,
) -> Result<(), GameCommandError> {
    let record = state
        .towns()
        .get(town)
        .ok_or(GameCommandError::Town(TownCommandError::UnknownTown))?;
    if owner(state, town) != Some(player) {
        return Err(GameCommandError::Town(TownCommandError::NotOwner));
    }
    let core = rules.core();
    let sale = h3_towns::check_war_machine(record, rules.towns(), core, building)
        .map_err(GameCommandError::Town)?;
    let hero = window_hero(state, rules, (town, player))
        .ok_or(GameCommandError::Town(TownCommandError::NoVisitor))?;
    if !state
        .hero(hero)
        .is_some_and(|details| receives(details, core, sale.artifact))
    {
        return Err(GameCommandError::Town(TownCommandError::HasWarMachine));
    }
    if !state.can_pay(player, sale.price) {
        return Err(GameCommandError::CannotPay);
    }
    state.pay_price(player, sale.price, sink);
    let change = HeroChange::AddArtifact(sale.artifact.into());
    state.change_hero(core, hero, Some(player), change, sink);
    Ok(())
}

/// Whether the hero `details` would receive the war machine `artifact` (TWN-SMT-001; survey §4.4):
/// it holds none, worn or carried (the original's `hero_has_artifact` 0x4D91B0; a war machine is
/// never carried, HER-ART-002), and a position takes it (`HeroChange::AddArtifact`'s check,
/// HER-CHG-002: in the original packs the machine's own, free while the hero holds none).
pub(super) fn receives(details: &HeroDetails, core: &Ruleset, artifact: ArtifactTypeId) -> bool {
    let held = details
        .worn
        .iter()
        .flatten()
        .chain(&details.backpack)
        .any(|item| item.artifact == artifact);
    let change = HeroChange::AddArtifact(artifact.into());
    !held && h3_heroes::check_change(details, core, change).is_ok()
}
