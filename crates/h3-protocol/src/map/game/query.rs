//! The whole game's queries on the wire (E16 M5 T11): a seat's question answered by the engine's
//! own queries for the seat's player ([`answer`]), a hero's reachable tiles packed as a path tree
//! ([`moves`]). What leaves is the player's by construction: the engine answers a player's
//! question about its own heroes, towns and battle side only.

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::Reach;
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_game::{Actor, GameRuleset, GameState};

use super::pos;
use crate::map::query as battle_query;
use crate::wire::WireU64;
use crate::wire::game::{GameAnswer, GameQuery, Moves, Pos, TownOffer};

/// The answer to `player`'s `query` now: its hero's moves (`h3_game::legal_moves`), its town's
/// hall (`h3_game::GameState::town_offer`), its battle's targets for the side it owns there (the
/// battle's [`crate::map::query`]; none without a battle of its own).
pub fn answer(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    query: GameQuery,
) -> GameAnswer {
    match query {
        GameQuery::Moves { hero } => GameAnswer::Moves {
            hero,
            moves: h3_game::legal_moves(state, rules, Actor::Player(player), HeroId::new(hero))
                .map(|reach| moves(&reach)),
        },
        GameQuery::TownOffer { town } => GameAnswer::TownOffer {
            town,
            offer: state
                .town_offer(rules, player, TownId::new(town))
                .map(town_offer),
        },
        GameQuery::Battle { query } => GameAnswer::Battle {
            targets: state
                .battle(player)
                .and_then(|battle| battle.side_of(player).map(|side| (battle, side)))
                .map(|(battle, side)| battle_query(battle.state(), rules.battle(), side, query))
                .unwrap_or_default(),
        },
    }
}

/// A hero's reachable tiles as a path tree ([`Moves`]): each tile's path from `reach`
/// (`Reach::path`, the tiles in `Reach::tiles`' order) entered step by step, a step that some
/// earlier path took shared, a new one a new node; the tile's last node carries its points.
pub fn moves(reach: &Reach) -> Moves {
    let origin = pos(reach.origin());
    let mut tree = Tree {
        children: BTreeMap::new(),
        steps: Vec::new(),
        // Node 0's, the hero's tile, dropped below.
        left: vec![0],
    };
    for (tile, points) in reach.tiles() {
        // Every listed tile but the hero's own has a path of at least one step.
        let path = reach.path(tile).unwrap_or_default();
        let (mut node, mut at) = (0, origin);
        for step in path.into_iter().map(pos) {
            node = tree.step(node, direction(at, step));
            at = step;
        }
        // A node the tree made: within `left`.
        tree.left[usize::try_from(node).unwrap_or_default()] = points.saturating_add(1);
    }
    let mut left = tree.left;
    left.remove(0);
    Moves {
        origin,
        points: reach.points_left(reach.origin()).unwrap_or_default(),
        steps: tree.steps,
        left,
    }
}

/// A path tree being built: each node's step (from node 1) and points (from node 0), and its
/// children by direction.
struct Tree {
    children: BTreeMap<(u32, u32), u32>,
    steps: Vec<u32>,
    left: Vec<u32>,
}

impl Tree {
    /// The node one step in `direction` from `node`: an earlier path's, or a new one.
    fn step(&mut self, node: u32, direction: u32) -> u32 {
        let Self {
            children,
            steps,
            left,
        } = self;
        *children.entry((node, direction)).or_insert_with(|| {
            steps.push(node.saturating_mul(8).saturating_add(direction));
            left.push(0);
            u32::try_from(steps.len()).unwrap_or(u32::MAX)
        })
    }
}

/// The direction ([`crate::wire::game::DIRECTIONS`]) of the step from `from` to `to`, a tile
/// next to it: by the signs of the column's and the row's change.
fn direction(from: Pos, to: Pos) -> u32 {
    // By the row's sign, then the column's: north-west, north, north-east; west, none, east;
    // south-west, south, south-east.
    const BY_SIGNS: [u32; 9] = [7, 0, 1, 6, 0, 2, 5, 4, 3];
    let dx = (i32::from(to.x) - i32::from(from.x)).signum();
    let dy = (i32::from(to.y) - i32::from(from.y)).signum();
    let index = usize::try_from((dy + 1) * 3 + dx + 1).unwrap_or(4);
    BY_SIGNS[index]
}

/// A town's hall on the wire.
pub(crate) const fn town_offer(
    h3_game::TownOffer {
        possible,
        buildable,
        payable,
    }: h3_game::TownOffer,
) -> TownOffer {
    TownOffer {
        possible: WireU64(possible.bits()),
        buildable: WireU64(buildable.bits()),
        payable: WireU64(payable.bits()),
    }
}
