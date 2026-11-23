//! A town's two rows as the game runs them (TWN-ROW-002 to -004; E09 T28a, the town epic's design
//! note §53; survey `E09-growth.md` §9): `TownCommand::Heroes` checks the actor's turn as every
//! town command does (`GameState::check_ready`: its turn, no open decision), then here the town
//! and its owner, then the adventure's rules ([`AdventureState::move_town_hero`]), each refusal
//! changing nothing. The original's portraits are a human's town screen's (a computer's are E13's).

use h3_adventure::{AdventureState, TownHeroError, TownHeroMove};
use h3_contracts::{PlayerId, TownId};
use h3_towns::{TownCommand, TownCommandError, TownHeroChange};

use super::command::owner;
use super::guild;
use super::view::{Told, named};
use crate::command::{AdventureSink, GameCommand, GameCommandError, GameEvent, Sink};
use crate::{GameRuleset, GameState};

/// The three moves in their listed order (TWN-ROW-002 to -004).
const MOVES: [TownHeroChange; 3] = [
    TownHeroChange::Enter,
    TownHeroChange::Leave,
    TownHeroChange::Swap,
];

/// Runs `player`'s hero move `change` at `town` (TWN-ROW-002 to -004), the player's turn checked
/// ([`GameState::check_ready`]): the town in the store ([`TownCommandError::UnknownTown`]), its
/// owner `player` ([`TownCommandError::NotOwner`]), then the adventure's move with the town
/// part's most heroes on the map ([`AdventureState::move_town_hero`]: its refusals as the town
/// command's), its events told (the hero leaving and entering the map to the viewers of the
/// entrance, the owner moving its own view; by name only to the players that see the town's
/// garrison hero, [`named`], E09 T31); then each army slot `Enter` changed told to `player`
/// (`StackChanged`: the hero's army, the town's garrison), then each view of the town the move
/// changed told to its viewer ([`Told`]: `Enter` empties the garrison); then, while a visitor
/// stands at the town after the move (`Leave` and `Swap`, never `Enter`), the town's heroes
/// learn its guild's spells (TWN-GLD-005, E09 T48a, [`guild::learn`]: the original's town
/// screen's two bars built again after the click, `town_screen_bars` 0x5C6E10, which teaches only
/// with a visitor there).
///
/// # Errors
///
/// [`GameCommandError::Town`] with the town's refusal; nothing changed.
// rule: TWN-ROW-002, TWN-ROW-003, TWN-ROW-004, TWN-GLD-005
pub(crate) fn heroes<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    player: PlayerId,
    (town, change): (TownId, TownHeroChange),
    sink: &mut S,
) -> Result<(), GameCommandError> {
    let refused = GameCommandError::Town;
    state
        .towns()
        .get(town)
        .ok_or(refused(TownCommandError::UnknownTown))?;
    if owner(state, town) != Some(player) {
        return Err(refused(TownCommandError::NotOwner));
    }
    let told = Told::before::<S>(state, rules, &[town], &[]);
    let named = named::<S>(state, rules, town);
    let (adventure, _) = state.adventure_with_heroes();
    let slots = adventure
        .move_town_hero(
            (rules.core(), rules.adventure()),
            player,
            (town, most_heroes(rules)),
            adventure_move(change),
            named,
            &mut AdventureSink(sink),
        )
        .map_err(|error| refused(town_error(error)))?;
    for at in slots {
        let stack = state.stack_at(player, at);
        sink.emit(GameEvent::StackChanged { player, at, stack });
    }
    told.tell(state, rules, sink);
    // rule: TWN-GLD-005
    if state.adventure().town_visitor(town).is_some() {
        guild::learn(state, rules, town, sink);
    }
    Ok(())
}

/// The hero moves `player` may make at its town `town` now, in [`MOVES`]' order, each as
/// [`heroes`] accepts it (TWN-ROW-002 to -004; listed after the town's builds and recruitments):
/// the adventure's [`AdventureState::town_hero_moves`], its cheap reading of the checks.
pub(crate) fn legal_moves(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    (town, entrance): (TownId, Option<(usize, Option<usize>)>),
) -> impl Iterator<Item = GameCommand> + use<> {
    let adventure: &AdventureState = state.adventure();
    let most = (town, most_heroes(rules));
    let legal = entrance.map_or([false; 3], |at| {
        adventure.town_hero_moves_at(rules.adventure(), player, most, at)
    });
    MOVES
        .into_iter()
        .zip(legal)
        .filter(|&(_, legal)| legal)
        .map(move |(change, _)| GameCommand::Town(TownCommand::Heroes { town, change }))
}

/// The town part's most heroes on the map (TWN-ROW-003): 0 without parameters, a part that has
/// no town, whose `Leave` is then always [`TownCommandError::TooManyHeroes`] (no garrison hero
/// stands in such a game).
pub(super) fn most_heroes(rules: &GameRuleset) -> u8 {
    rules
        .towns()
        .params()
        .map_or(0, |params| params.most_heroes)
}

/// The adventure's move for the town command's.
const fn adventure_move(change: TownHeroChange) -> TownHeroMove {
    match change {
        TownHeroChange::Enter => TownHeroMove::Enter,
        TownHeroChange::Leave => TownHeroMove::Leave,
        TownHeroChange::Swap => TownHeroMove::Swap,
    }
}

/// The town command's refusal for the adventure's (one to one; a town the map lacks is the
/// store's too).
pub(super) const fn town_error(error: TownHeroError) -> TownCommandError {
    match error {
        TownHeroError::UnknownTown => TownCommandError::UnknownTown,
        TownHeroError::GarrisonHeld => TownCommandError::GarrisonHeld,
        TownHeroError::NoVisitor => TownCommandError::NoVisitor,
        TownHeroError::ArmiesDoNotFit => TownCommandError::ArmiesDoNotFit,
        TownHeroError::TooManyCreatures => TownCommandError::TooManyCreatures,
        TownHeroError::NoGarrisonHero => TownCommandError::NoGarrisonHero,
        TownHeroError::EntranceHeld => TownCommandError::EntranceHeld,
        TownHeroError::TooManyHeroes => TownCommandError::TooManyHeroes,
        TownHeroError::EmptyArmy => TownCommandError::EmptyArmy,
    }
}
