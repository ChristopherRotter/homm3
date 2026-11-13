//! Recruitment at a town as the game runs it (TWN-REC-001, -002; GAM-DEC-008 amended; E09 T27,
//! the town epic's design note §49 to §51; survey `E09-growth.md` §5, §8): `TownCommand::Recruit`
//! opens the hire dialog of what a building sells, its target the town's garrison (the upper bar,
//! until a garrison hero stands there, E09 T28a), its source the town's index or Portal; a
//! purchase is checked by the town (the creature, the amount), the adventure (the garrison's
//! room) and the treasury, in that order, then paid, the town's count lowered and the creatures
//! joined, each change told. The dialog itself, its legal answers and its view are the game's
//! (`state/hire.rs`), shared with the map's dwellings.

use alloc::vec::Vec;

use h3_adventure::{HireDialog, HireError, HireSource, HireTarget};
use h3_content::CreatureTypeId;
use h3_contracts::{PlayerId, TownId};
use h3_towns::{BuildingId, RecruitError, RecruitSource, TownCommandError};

use super::command::owner;
use super::view::Told;
use crate::command::{AdventureSink, GameCommandError, GameEvent, Sink};
use crate::{GameRuleset, GameState};

/// Opens `player`'s hire dialog of what `building` of `town` sells (TWN-REC-001, -002; E09 design
/// §49), `player`'s turn without an open decision checked by `GameState::check_ready`, in this
/// order, each refusal changing nothing:
///
/// 1. the town in the store of towns, else [`TownCommandError::UnknownTown`];
/// 2. its owner `player`, else [`TownCommandError::NotOwner`] (an ally's town or a visited one
///    sells nothing: TD-11; E10's teams);
/// 3. [`h3_towns::recruit_source`]'s tests (the building of the record, of a selling role,
///    standing; a Portal holding something, else `NothingSummoned` with no draw, decision 71).
///
/// Then the dialog is the player's open decision (`GameState::open_hire`, `HireOpened` to it),
/// its target the town's garrison, at a count of 0 too (the original's dialog shows it; its only
/// legal answer is to close it).
///
/// # Errors
///
/// [`GameCommandError::Town`] with the refusal.
// rule: TWN-REC-001, TWN-REC-002, GAM-DEC-008
pub(crate) fn open<S: Sink>(
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
    let source = h3_towns::recruit_source(record, rules.towns(), building)
        .map_err(GameCommandError::Town)?;
    let source = match source {
        RecruitSource::Index(index) => HireSource::Town { town, index },
        RecruitSource::Portal => HireSource::Portal { town },
    };
    let dialog = HireDialog {
        target: HireTarget::Garrison(town),
        source,
    };
    state.open_hire(player, dialog, sink);
    Ok(())
}

/// The town and its source of a hire dialog's `source`, if it is a town's (E09 T27).
pub(crate) const fn town_source(source: &HireSource) -> Option<(TownId, RecruitSource)> {
    match *source {
        HireSource::Town { town, index } => Some((town, RecruitSource::Index(index))),
        HireSource::Portal { town } => Some((town, RecruitSource::Portal)),
        HireSource::Object { .. } => None,
    }
}

/// The creatures `source` of `town` sells with their counts as they stand, in its order
/// ([`h3_towns::recruit_stock`]: an upgraded index's two from one count); none for a town the
/// store does not have.
pub(crate) fn stock(
    state: &GameState,
    rules: &GameRuleset,
    (town, source): (TownId, RecruitSource),
) -> Option<Vec<(CreatureTypeId, u32)>> {
    let record = state.towns().get(town)?;
    let stock = h3_towns::recruit_stock(record, rules.towns(), source);
    Some(
        stock
            .into_iter()
            .map(|(creature, count)| (creature, u32::from(count)))
            .collect(),
    )
}

/// A town's hire dialog's purchase of `amount` `creature`s (TWN-REC-001, -002; E09 design §50,
/// §51; the original's `recruit_dialog_proc` 0x550940), checked in this order, each refusal
/// changing nothing (the store, the army, the treasury):
///
/// 1. the town in the store ([`HireError::NoSource`]; defensive: nothing removes a town);
/// 2. the town's tests ([`h3_towns::check_recruit`]): the creature one of the source's
///    ([`HireError::NotOffered`]), the amount from 1 to its count ([`HireError::Amount`]), the
///    town's refusals given in the dialog's one vocabulary;
/// 3. the garrison's room ([`h3_adventure::AdventureState::check_join`]: [`HireError::NoRoom`]);
/// 4. the treasury holds the price ([`GameCommandError::CannotPay`]).
///
/// Then, infallibly: the price paid (`ResourcePaid` per resource), the town's count lowered
/// ([`h3_towns::recruit`]: an upgraded index's one count whichever creature, the Portal's with
/// its creature kept), the creatures joined into the garrison, a stack of the creature first,
/// else the first free slot (ADV-ARM-007), `StackChanged` of its slot to `player`, then each view
/// the purchase changed told to its viewer ([`Told`]: the owner's counts, every viewer's
/// garrison at its level). The caller closes a one-creature dialog after it.
///
/// # Errors
///
/// [`GameCommandError::Hire`] with the refusal, or [`GameCommandError::CannotPay`].
// rule: TWN-REC-001, TWN-REC-002, GAM-DEC-008, ADV-ARM-007
pub(crate) fn buy<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    player: PlayerId,
    (dialog, (town, source)): (&HireDialog, (TownId, RecruitSource)),
    (creature, amount): (CreatureTypeId, u32),
    sink: &mut S,
) -> Result<(), GameCommandError> {
    let record = state
        .towns()
        .get(town)
        .ok_or(GameCommandError::Hire(HireError::NoSource))?;
    h3_towns::check_recruit(record, rules.towns(), source, (creature, amount)).map_err(
        |error| {
            GameCommandError::Hire(match error {
                RecruitError::NotOffered(creature) => HireError::NotOffered(creature),
                RecruitError::Amount(amount) => HireError::Amount(amount),
            })
        },
    )?;
    let bought = h3_towns::recruit(record, source, amount);
    let purchase = state
        .adventure()
        .check_join(
            (rules.core(), rules.adventure()),
            &dialog.target,
            (creature, amount),
        )
        .map_err(GameCommandError::Hire)?;
    if !state.can_pay(player, purchase.price()) {
        return Err(GameCommandError::CannotPay);
    }
    let told = Told::before::<S>(state, rules, &[town], &[]);
    state.pay_price(player, purchase.price(), sink);
    let _changed = state.towns_mut().write(town, bought);
    let (adventure, _) = state.adventure_with_heroes();
    if let Some(at) = adventure.hire(purchase, &mut AdventureSink(sink)) {
        let stack = state
            .adventure()
            .map()
            .town_object(town)
            .and_then(|index| state.adventure().town_garrison(index))
            .and_then(|army| army.iter().find(|s| s.slot == at.slot).copied());
        sink.emit(GameEvent::StackChanged { player, at, stack });
    }
    told.tell(state, rules, sink);
    Ok(())
}
