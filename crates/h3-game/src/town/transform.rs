//! The Necropolis's Skeleton Transformer in the game (TWN-TRF-001; E09 T59b2, the town epic's
//! design note §113, decisions 207, 228; survey `E09-trades.md` §7): `TownCommand::Transform`
//! transforms one whole stack of the town's visitor's army or of its upper bar's (the garrison
//! hero's, else the town's garrison) into the part's target of its creature, its count kept, free,
//! any number a turn. One command, one stack: the original's dialog with its holding army, its
//! moves, its Convert and its close in one atomic change (**`TWN-QUIRK-23`**, T-26 fixed by
//! construction: a timed turn's end loses nothing). The legal listing offers each transformation `transform` accepts
//! (`town::services`).

use h3_contracts::{ArmyStack, HeroId, PlayerId, TownId};
use h3_towns::{TownArmy, TownCommandError, TransformerSale};

use super::command::owner;
use super::view::Told;
use crate::command::{GameCommandError, GameEvent, Sink};
use crate::{GameRuleset, GameState};

/// Transforms the stack in `slot` of `army` at `player`'s `town` (TWN-TRF-001; E09 T59b2, design
/// §113; survey §7.1 to §7.7), `player`'s turn without an open decision checked by
/// `GameState::check_ready`, in this order, each refusal changing nothing:
///
/// 1. the town in the store of towns, else [`TownCommandError::UnknownTown`];
/// 2. its owner `player`, else [`TownCommandError::NotOwner`] (the original tests no owner, survey
///    §7.7: an ally's town waits for E10's teams, `TWN-STUB-21`);
/// 3. [`h3_towns::check_transformer`]: a standing Skeleton Transformer, else
///    [`TownCommandError::NoTransformer`]; the part's table;
/// 4. the army ([`army_of`]): for [`TownArmy::Visitor`] `player`'s hero on the town's entrance,
///    else [`TownCommandError::NoVisitor`] (an ally's visitor's army is not the town owner's to
///    change); [`TownArmy::Garrison`] always (no hero needed);
/// 5. `slot` one of the adventure part's army slots, else [`TownCommandError::UnknownSlot`];
///    a stack there, else [`TownCommandError::EmptySlot`];
/// 6. its creature other than its target ([`TransformerSale::target`]), else
///    [`TownCommandError::NothingToTransform`] (the original's Skeleton, which keeps Convert
///    disabled).
///
/// Then, infallibly: the slot's creature set to the target, its count kept, nothing merged, the
/// army's last stack allowed (`AdventureState::transform_stack`), `StackChanged` of the slot to
/// `player`, then each view of the town the change made told to its viewer ([`Told`]: the upper
/// bar's garrison at each viewer's level). No price, no movement point, no mark (survey §7.5). No
/// field of the command is trusted but its three: the target is the part's (ADR-0004).
///
/// # Errors
///
/// [`GameCommandError::Town`] with the refusal.
// rule: TWN-TRF-001
pub(crate) fn transform<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    player: PlayerId,
    (town, army, slot): (TownId, TownArmy, u8),
    sink: &mut S,
) -> Result<(), GameCommandError> {
    let refused = GameCommandError::Town;
    let record = state
        .towns()
        .get(town)
        .ok_or(refused(TownCommandError::UnknownTown))?;
    if owner(state, town) != Some(player) {
        return Err(refused(TownCommandError::NotOwner));
    }
    let sale = h3_towns::check_transformer(record, rules.towns()).map_err(refused)?;
    let (hero, stacks) =
        army_of(state, (town, player), army).ok_or(refused(TownCommandError::NoVisitor))?;
    if slot >= rules.adventure().params().army_slots() {
        return Err(refused(TownCommandError::UnknownSlot));
    }
    let creature = stacks
        .iter()
        .find(|stack| stack.slot == slot)
        .map(|stack| stack.creature)
        .ok_or(refused(TownCommandError::EmptySlot))?;
    let target = sale.target(creature);
    if target == creature {
        return Err(refused(TownCommandError::NothingToTransform));
    }
    let told = Told::before::<S>(state, rules, &[town], &[]);
    let (adventure, _) = state.adventure_with_heroes();
    let at = adventure.transform_stack(town, (hero, slot), target);
    let stack = state.stack_at(player, at);
    sink.emit(GameEvent::StackChanged { player, at, stack });
    told.tell(state, rules, sink);
    Ok(())
}

/// The army `army` of `player`'s `town` names (TWN-TRF-001; design §113, decision 207: the
/// original's default and its selection override as the command's choice, survey §7.2), with the
/// hero whose army it is: [`TownArmy::Visitor`] the army of `player`'s hero on the town's
/// entrance, none without one; [`TownArmy::Garrison`] the upper bar's, its garrison hero's where
/// one stands, else the town's garrison with no hero (`AdventureState::upper_bar`).
pub(super) fn army_of(
    state: &GameState,
    (town, player): (TownId, PlayerId),
    army: TownArmy,
) -> Option<(Option<HeroId>, &[ArmyStack])> {
    let adventure = state.adventure();
    match army {
        TownArmy::Visitor => adventure
            .town_visitor(town)
            .and_then(|hero| adventure.hero(hero))
            .filter(|on| on.owner == player)
            .map(|on| (Some(on.id), &on.army[..])),
        TownArmy::Garrison => Some((
            adventure.town_hero(town).map(|on| on.id),
            adventure.upper_bar(town),
        )),
    }
}

/// The slots of `stacks` whose stack the Transformer `sale` changes, as a mask of bits by slot
/// (TWN-TRF-001; E09 T59b2, design §115: "listed per stack that changes"): a stack whose creature
/// is not its own target. The legal listing's test; a slot past the mask's width has no bit (the
/// adventure part's seven).
// rule: TWN-TRF-001
pub(super) fn changing(stacks: &[ArmyStack], sale: TransformerSale<'_>) -> u8 {
    stacks
        .iter()
        .filter(|stack| sale.target(stack.creature) != stack.creature)
        .fold(0, |mask, stack| {
            mask | 1u8.checked_shl(u32::from(stack.slot)).unwrap_or(0)
        })
}
