//! The whole game's queries (E16 M5 T11, protocol v50; ADR-0018 point 10): what a seat asks of
//! the server's rules ([`GameQuery`]) and what they answer ([`GameAnswer`]), for the seat's
//! player alone: a hero's moves (`h3_game::legal_moves`, which `GameTurn` leaves out as
//! `h3_game::legal_commands` does), a town's hall (`h3_game::GameState::town_offer`) and a
//! battle's targets (the battle wire's [`Query`], for the side the player owns in its battle).
//!
//! A hero's moves travel as a path tree ([`Moves`]): each tile it can reach today with the points
//! it would have left there, and the path to it as its node's chain of steps, the paths' common
//! beginnings shared, so that an answer is a few numbers per tile, not a path per tile. A client
//! reads it with [`Moves::tiles`], [`Moves::points_left`] and [`Moves::path`], the engine's
//! `h3_adventure::Reach`'s own.
//!
//! The same rules as [`super`]'s. Provisional (E16 M5 T11; the epic's review list): the shapes,
//! the tree's packing, the directions' order.

use alloc::vec;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use super::Pos;
use crate::wire::{Query, Target, WireU64};

/// A question about the whole game for the seat's player (E16 M5 T11, v50).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GameQuery {
    /// Where the player's hero `hero` may move now (`h3_game::legal_moves`).
    Moves {
        /// The hero.
        hero: u16,
    },
    /// The hall of the player's town `town` (`h3_game::GameState::town_offer`): in its turn or
    /// not, as the original's town list opens it at any time.
    TownOffer {
        /// The town.
        town: u16,
    },
    /// A question about the player's battle, as a battle's seat asks it, for the side the player
    /// owns there.
    Battle {
        /// The question.
        query: Query,
    },
}

/// The answer to a [`GameQuery`] (E16 M5 T11, v50), by the question's kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GameAnswer {
    /// A hero's moves.
    Moves {
        /// The hero asked about.
        hero: u16,
        /// Its moves; none when the player's turn is not running, it has an open decision, or
        /// the hero is not its own on the map (`h3_game::legal_moves`).
        moves: Option<Moves>,
    },
    /// A town's hall.
    TownOffer {
        /// The town asked about.
        town: u16,
        /// Its hall; none unless the player owns the town.
        offer: Option<TownOffer>,
    },
    /// A battle's targets.
    Battle {
        /// The targets the server's rules allow, as a battle's `QueryResult` gives them: none
        /// without a battle of the player's.
        targets: Vec<Target>,
    },
}

/// The steps of a path tree, by direction (E16 M5 T11): `DIRECTIONS[d]` is the column's and the
/// row's change of a step in direction `d`, clockwise from north (the row above): north,
/// north-east, east, south-east, south, south-west, west, north-west.
pub const DIRECTIONS: [(i8, i8); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

/// Where a hero may move today (E16 M5 T11; `h3_game::legal_moves`, `h3_adventure::Reach`): the
/// tiles it can reach over the tiles its owner has explored, each with the points it would have
/// left there and the path to it, as a tree of steps from its tile.
///
/// The tree's node 0 is the hero's tile; node `k` from 1 is `steps[k - 1]`, packed as
/// `8 * parent + d`: one step in direction `d` ([`DIRECTIONS`]) from node `parent`, an earlier
/// node. A path is a node's chain back to node 0, reversed: the tiles it steps onto, as
/// `MoveHero` takes them. `left[k - 1]` is 0 for a node no path ends on (a tile the path passes),
/// else the points the hero would have left on node `k`'s tile plus 1: that tile is reachable,
/// and the chain is its path. Each reachable tile is one node's.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Moves {
    /// The hero's tile, on its level: every path stays on it.
    pub origin: Pos,
    /// The hero's points now.
    pub points: u32,
    /// The tree's steps, node 1 first.
    pub steps: Vec<u32>,
    /// Each node's points left plus 1, 0 for a tile a path passes; as many as `steps`.
    pub left: Vec<u32>,
}

impl Moves {
    /// The tile of each node, node 0's (the hero's) first; `None` for a malformed tree: `left`
    /// and `steps` of different lengths, a parent not earlier, a step off the map's range.
    pub fn nodes(&self) -> Option<Vec<Pos>> {
        if self.left.len() != self.steps.len() {
            return None;
        }
        let mut tiles = vec![self.origin];
        for &step in &self.steps {
            let (parent, direction) = unpack(step);
            let from = *tiles.get(parent)?;
            let (dx, dy) = DIRECTIONS[direction];
            tiles.push(Pos {
                x: from.x.checked_add_signed(i16::from(dx))?,
                y: from.y.checked_add_signed(i16::from(dy))?,
                level: from.level,
            });
        }
        Some(tiles)
    }

    /// Every tile the hero can reach today but its own, in the tree's order, with the points it
    /// would have left there (`h3_adventure::Reach::tiles`); none for a malformed tree.
    pub fn tiles(&self) -> Vec<(Pos, u32)> {
        let nodes = self.nodes().unwrap_or_default();
        let reached = nodes.into_iter().skip(1).zip(&self.left);
        reached
            .filter(|&(_, &left)| left > 0)
            .map(|(tile, &left)| (tile, left - 1))
            .collect()
    }

    /// The points the hero would have left on `to` (on its own tile: its points now), if it can
    /// reach it today (`h3_adventure::Reach::points_left`).
    pub fn points_left(&self, to: Pos) -> Option<u32> {
        if to == self.origin {
            return Some(self.points);
        }
        let mut tiles = self.tiles().into_iter();
        tiles.find(|&(tile, _)| tile == to).map(|(_, left)| left)
    }

    /// The path to `to`, if the hero can reach it: the tiles it steps onto, in order, without its
    /// own (empty for its own tile), as `MoveHero` takes them (`h3_adventure::Reach::path`).
    pub fn path(&self, to: Pos) -> Option<Vec<Pos>> {
        if to == self.origin {
            return Some(Vec::new());
        }
        let nodes = self.nodes()?;
        let ends = nodes
            .iter()
            .skip(1)
            .zip(&self.left)
            .position(|(&tile, &left)| tile == to && left > 0)?;
        let mut path = Vec::new();
        // Node `ends + 1`'s chain: each parent is an earlier node, so the walk ends at node 0.
        let mut node = ends + 1;
        while node > 0 {
            path.push(nodes[node]);
            node = unpack(self.steps[node - 1]).0;
        }
        path.reverse();
        Some(path)
    }
}

/// A step's parent node and direction.
fn unpack(step: u32) -> (usize, usize) {
    let parent = usize::try_from(step >> 3).unwrap_or(usize::MAX);
    (parent, usize::try_from(step & 7).unwrap_or(0))
}

/// A town's hall as its owner sees it (E16 M5 T11; `h3_game::TownOffer`), each set a bit per
/// building of the town's faction's record as [`super::TownBuildings`]' (bit `i` the building of
/// index `i`, a decimal string), each holding the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TownOffer {
    /// The buildings it may still build some day, ever built or not (the hall's "not now"
    /// against "never").
    pub possible: WireU64,
    /// The buildings the hall offers now: none while the town was built in today (TWN-BLD-004).
    pub buildable: WireU64,
    /// Those of `buildable` whose price the owner's treasury holds now: the builds `GameTurn`
    /// lists for the town.
    pub payable: WireU64,
}
