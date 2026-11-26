//! The engine's reach in play (E24 M5 T4a, *provisional*; `docs/epics/E24-design.md` §5.26): the
//! facts a live view asks of the hot seat's host while the viewer walks, in place of a record's or
//! a recording's.
//!
//! - **A stand** ([`Stand`]): the selected hero standing between two of the engine's commands, by
//!   its place among the map's heroes and the walks and turns the loop took so far. The engine's
//!   answer for it holds until the next walk or turn, so the loop asks it once per stand.
//! - **The reach** ([`Reach`]): the selected hero's moves today (`GameQuery::Moves`, the engine's
//!   `h3_adventure::Reach`): each tile it reaches with the path there as a click shows it (every
//!   arrow green: a step the points pay today, `timeline::arrows`) and what the path costs (the
//!   cursor's, `Hover::cursor`). Only today's: the original's cursor counts the days of a longer
//!   path and its red arrows show it, which the engine's query does not answer (MCL-Q37): a tile
//!   beyond today's reach shows the arrow and a click there only moves the pointer.
//! - **The host's facts** ([`Hosted`]): the reaches given so far and the walks the engine refused
//!   (a direction key's step into an obstacle: nothing happens, as the original's
//!   `hero_step_allowed` refuses it), so that a loop run again from before them plays them alike.
//!   A refusal holds for its stand: the same walk asked again there (a held key's repeats against
//!   a wall) is refused at once, not asked again.
//! - **A walk asked** ([`WalkAsked`]): a path's walk or a key's step whose facts the loop lacks:
//!   the host plays its `MoveHero` and gives its facts (`Loop::add_walks`) or its refusal
//!   (`Loop::refuse_walk`).

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_map::model::Pos;
use h3_protocol::wire::game::{Moves, Pos as WirePos};

use crate::live::Arrow;
use crate::timeline;

/// The selected hero standing between two of the engine's commands (E24 M5 T4a): its place among
/// the map's heroes (`Start::heroes`), the engine's walks and the turns' starts the loop took.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stand {
    /// The hero's place among the map's heroes.
    pub hero: usize,
    /// The engine's walks taken so far.
    pub walks: usize,
    /// The turns' starts taken so far.
    pub turns: usize,
}

/// The selected hero's reach today, as the engine answers it (E24 M5 T4a).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Reach {
    /// The path to each tile it reaches, its arrows in order from the hero's tile.
    pub paths: Vec<Vec<Arrow>>,
    /// The points each path costs, by the tile's column and row on the hero's level.
    pub costs: BTreeMap<(i32, i32), u32>,
}

impl Reach {
    /// The reach of `moves` (the engine's answer), each path's arrows by the walk's table
    /// `table` (`path_arrow_frames`, `timeline::arrows`); none for a tree that does not read (a
    /// malformed answer, a tile off the map's 256 columns or rows).
    #[must_use]
    pub fn of(moves: &Moves, table: &[u8]) -> Option<Self> {
        moves.nodes()?;
        let origin = tile(moves.origin)?;
        let mut reach = Self::default();
        for (to, left) in moves.tiles() {
            // Each tile the tree reaches has its path.
            let path: Vec<Pos> = moves
                .path(to)
                .unwrap_or_default()
                .into_iter()
                .map(tile)
                .collect::<Option<_>>()?;
            reach.paths.push(timeline::arrows(table, origin, &path)?);
            let cost = moves.points.saturating_sub(left);
            reach.costs.insert((i32::from(to.x), i32::from(to.y)), cost);
        }
        Some(reach)
    }
}

/// What the host gave a live view so far (E24 M5 T4a): each stand's reach and the walks the engine
/// refused, by their stands and tiles.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Hosted {
    /// The reaches given, by their stands.
    pub reaches: Vec<(Stand, Arc<Reach>)>,
    /// The walks refused: each one's stand and tiles.
    pub refused: Vec<(Stand, Vec<Pos>)>,
}

impl Hosted {
    /// The reach given for `stand`.
    #[must_use]
    pub fn reach(&self, stand: Stand) -> Option<&Arc<Reach>> {
        self.reaches
            .iter()
            .find(|(s, _)| *s == stand)
            .map(|(_, r)| r)
    }

    /// Whether the walk along `tiles` from `stand` was refused.
    #[must_use]
    pub fn refuses(&self, stand: Stand, tiles: &[Pos]) -> bool {
        self.refused.iter().any(|(s, p)| *s == stand && p == tiles)
    }
}

/// A walk whose facts the loop lacks, asked of the host (E24 M5 T4a): the millisecond it starts
/// at, the stand it starts from (its hero's place among the map's heroes) and the tiles it steps
/// onto (a key's step's one).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalkAsked {
    /// The walk's millisecond.
    pub at: u32,
    /// The stand it starts from.
    pub stand: Stand,
    /// The tiles, in order, as `MoveHero` takes them.
    pub tiles: Vec<Pos>,
}

impl WalkAsked {
    /// The tiles as the wire's (`MoveHero`'s path).
    #[must_use]
    pub fn path(&self) -> Vec<WirePos> {
        self.tiles
            .iter()
            .map(|p| WirePos {
                x: u16::from(p.x),
                y: u16::from(p.y),
                level: p.z,
            })
            .collect()
    }
}

/// The map's tile of the wire's `at`, none off the map's 256 columns or rows.
fn tile(at: WirePos) -> Option<Pos> {
    Some(Pos {
        x: u8::try_from(at.x).ok()?,
        y: u8::try_from(at.y).ok()?,
        z: at.level,
    })
}

#[cfg(test)]
mod tests;
