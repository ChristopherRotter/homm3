//! The legal commands (ADR-0017 point 1, ADR-0004 §7; E08 T8b2a): what an actor may issue now,
//! built from the checks `apply` makes. Moves are not listed one by one: a hero with many points
//! reaches thousands of tiles, each with its own path. They are given per hero as the
//! reachable-tiles query ([`legal_moves`]), a path built only for the destination chosen, as the
//! battle gives casts by their targets. Consistency with `apply` is property-tested on imported
//! maps (`h3-map`'s tests).

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::Reach;
use h3_contracts::HeroId;

use crate::command::{Actor, GameCommand};
use crate::{GameRuleset, GameState};

/// The commands `actor` may issue now but its moves: `EndTurn`. Empty for a player whose turn is
/// not running or who is not in the game, by `apply`'s own turn check (ADR-0017's consequence).
/// The moves are [`legal_moves`].
pub fn legal_commands(state: &GameState, rules: &GameRuleset, actor: Actor) -> Vec<GameCommand> {
    // Nothing listed yet reads the rules; the commands of later increments will.
    let _ = rules;
    let Actor::Player(player) = actor;
    if state.check_acting(player).is_err() {
        return Vec::new();
    }
    vec![GameCommand::EndTurn]
}

/// Where `actor` may move `hero` now: the hero's reachable-tiles query
/// ([`h3_adventure::AdventureState::reach`]: the tiles its owner has explored, ADV-MOV-008), each
/// tile with the points the hero would have left there and the query's path to it, which a
/// `MoveHero` takes as it is. `None` when the actor's turn is not running, by `apply`'s own turn
/// check, or the hero is not the actor's.
///
/// The query keeps one path per tile; `apply` accepts every path the hero can pay, also another
/// one to a listed tile or one back to the hero's own. A longer path may rarely reach a tile the
/// query does not list ([`Reach`]); every tile one step away is listed. A client may so compute
/// its own paths: `apply` checks each step against the state and the actor.
pub fn legal_moves(
    state: &GameState,
    rules: &GameRuleset,
    actor: Actor,
    hero: HeroId,
) -> Option<Reach> {
    let Actor::Player(player) = actor;
    state.check_acting(player).ok()?;
    let adventure = state.adventure();
    adventure
        .hero(hero)
        .filter(|h| h.owner == player)
        .and_then(|h| adventure.reach(rules.core(), rules.adventure(), h.id))
}
