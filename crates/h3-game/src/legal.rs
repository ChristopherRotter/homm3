//! The legal commands (ADR-0017 point 1, ADR-0004 §7; E08 T8b2a): what an actor may issue now,
//! built from the checks `apply` makes; while an actor has an open decision, its answers only
//! (ADR-0017 point 6, GAM-DEC-001; E08 T14b), or its battle's commands (T19a). Moves are not listed one by one: a hero with many points
//! reaches thousands of tiles, each with its own path. They are given per hero as the
//! reachable-tiles query ([`legal_moves`]), a path built only for the destination chosen, as the
//! battle gives casts by their targets. Consistency with `apply` is property-tested on imported
//! maps (`h3-map`'s tests).

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::Reach;
use h3_battle::LegalCommands;
use h3_contracts::HeroId;

use crate::command::{Actor, Answer, GameCommand};
use crate::{GameRuleset, GameState};

/// The commands `actor` may issue now but its moves: with an open battle the battle's legal
/// commands for its side while that side is to act, and none while the other side is
/// (`h3_battle::legal_commands`; GAM-BAT-001, E08 T19a); with an open offer its answers, `Accept`
/// then `Decline`, in its turn or not, and nothing else (GAM-DEC-001); `Decline` alone when the
/// offer no longer stands (ADR-0017 point 10: a border guard another player of the group passed,
/// E08 T15a); otherwise `EndTurn`. Empty for a player whose turn is not running or who is not in
/// the game, by `apply`'s own checks (ADR-0017's consequence). The moves are [`legal_moves`].
// rule: GAM-DEC-001, GAM-BAT-001
pub fn legal_commands(state: &GameState, rules: &GameRuleset, actor: Actor) -> Vec<GameCommand> {
    let Actor::Player(player) = actor;
    if let Some(battle) = state.battle(player) {
        let mut legal = LegalCommands::new();
        if let Some(side) = battle
            .state()
            .side_to_act()
            .filter(|side| battle.owners()[side.index()] == Some(player))
        {
            h3_battle::legal_commands(battle.state(), rules.battle(), side, &mut legal);
        }
        return legal
            .iter()
            .map(|&command| GameCommand::Battle(command))
            .collect();
    }
    if let Some(offer) = state.decision(player) {
        let accept = state.adventure().offer_stands(offer);
        return [Answer::Accept, Answer::Decline]
            .into_iter()
            .filter(|&answer| accept || answer == Answer::Decline)
            .map(GameCommand::Answer)
            .collect();
    }
    if state.check_acting(player).is_err() {
        return Vec::new();
    }
    vec![GameCommand::EndTurn]
}

/// Where `actor` may move `hero` now: the hero's reachable-tiles query
/// ([`h3_adventure::AdventureState::reach`]: the tiles its owner has explored, ADV-MOV-008), each
/// tile with the points the hero would have left there and the query's path to it, which a
/// `MoveHero` takes as it is. `None` when the actor's turn is not running or it has an open
/// decision, by `apply`'s own checks (GAM-DEC-001), or the hero is not the actor's.
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
    state.check_ready(player).ok()?;
    let adventure = state.adventure();
    adventure
        .hero(hero)
        .filter(|h| h.owner == player)
        .and_then(|h| adventure.reach(rules.core(), rules.adventure(), state.heroes(), h.id))
}
