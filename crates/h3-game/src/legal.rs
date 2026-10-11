//! The legal commands (ADR-0017 point 1, ADR-0004 §7; E08 T8b2a): the commands an actor may
//! issue now, built from the checks `apply` makes. Their consistency with `apply` is
//! property-tested on imported maps (`h3-map`'s tests).

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::AdventureCommand;

use crate::command::{Actor, GameCommand};
use crate::{GameRuleset, GameState};

/// Every command `actor` may issue now, in this order: `EndTurn`, then for each of its heroes on
/// the map, by id, a `MoveHero` to each tile of the reachable-tiles query
/// ([`h3_adventure::AdventureState::reach`]: the tiles its owner has explored, ADV-MOV-008), row
/// by row, along the query's path. Empty for a player whose turn is not running or who is not in
/// the game: the turn check is `apply`'s (ADR-0017's consequence).
///
/// `MoveHero` is listed once per tile, with the query's path: `apply` also accepts every other
/// path the hero can pay, also one back to its own tile. The query keeps one path per tile, so a
/// longer path may rarely reach a tile it does not list ([`h3_adventure::Reach`]); every tile one
/// step away is listed.
pub fn legal_commands(state: &GameState, rules: &GameRuleset, actor: Actor) -> Vec<GameCommand> {
    let Actor::Player(player) = actor;
    if state.check_acting(player).is_err() {
        return Vec::new();
    }
    let adventure = state.adventure();
    let mut commands = vec![GameCommand::EndTurn];
    let reaches = adventure
        .heroes()
        .iter()
        .filter(|hero| hero.owner == player)
        .filter_map(|hero| {
            adventure
                .reach(rules.core(), rules.adventure(), hero.id)
                .map(|reach| (hero.id, reach))
        });
    for (hero, reach) in reaches {
        for (pos, _) in reach.tiles() {
            commands.extend(
                reach
                    .path(pos)
                    .map(|path| GameCommand::Adventure(AdventureCommand::MoveHero { hero, path })),
            );
        }
    }
    commands
}
