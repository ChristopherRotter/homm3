//! The passage of a flying or water-walking hero (E08 T41; ADV-MOV-007, ADV-MOV-009, ADV-MOV-010;
//! survey `E08-hero-movement.md` §3, the second reading; increment 5's design note §7's table,
//! `E08-log.md`): what a step may enter, whether the hero may stop there or only pass over, and
//! what it visits. One place for `MoveHero` and the reachable-tiles query, so that a later
//! compromise for the unexplored tiles (`ADV-QUIRK-15`, ADR-0016 point 4) changes one row here. A
//! walking hero's steps keep their own checks (`check_step` and the walking search), whose rows
//! this table repeats for walking.

use h3_contracts::{HeroId, PlayerId};
use h3_heroes::Movement;

use super::{AdventureState, Meets};
use crate::command::AdventureCommandError;
use crate::movement::DIRECTIONS;
use crate::ruleset::{AdventureRuleset, Passage, Visit};
use crate::setup::{Pos, StandError};
use crate::state::StaticTile;

/// How a hero moves over the map (ADR-0016 point 4): walking, flying (a flying level, ADV-MOV-009)
/// or water walking (a water-walking level, ADV-MOV-010); a hero that does both flies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    /// Neither level.
    Walk,
    /// A flying level.
    Fly,
    /// A water-walking level and no flying level.
    WaterWalk,
}

impl Mode {
    /// How a hero with these levels moves (`h3_heroes::movement`).
    pub(crate) const fn of(movement: &Movement) -> Self {
        match (movement.flying, movement.water_walking) {
            (Some(_), _) => Self::Fly,
            (None, Some(_)) => Self::WaterWalk,
            (None, None) => Self::Walk,
        }
    }
}

/// The kind of a node of a move (the original's two nodes per tile, flags bit 10, 0x4B1C4B): one
/// the hero may end a move on and visit from, or one it only passes over (a flight over a blocked
/// tile or water, a water walk over water).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Node {
    /// The hero may stop here. The start of a move is one.
    Stop,
    /// The hero only passes over.
    Pass,
}

/// What a step onto a tile allows (§7's table): a stop, a pass over it, or both.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Entry<'p> {
    /// The tile as it stands now (the live occupancy: the price and the drop read it).
    pub(crate) tile: StaticTile,
    /// The stop the step may make: `Some(None)` the hero stands on the tile (a free land tile, a
    /// border gate whose key its owner holds; a guard then decides), `Some(Some(_))` a visit,
    /// which ends the move (an entered object's, from any node; a neighbour's object or another
    /// player's hero visited from the tile before, which must be a stop node).
    pub(crate) stop: Option<Option<Meets<'p>>>,
    /// Whether the hero may pass over the tile, its flight or water walk going on. Never together
    /// with a stop on the tile (`Some(None)`): a stop node goes on too.
    pub(crate) pass: bool,
}

impl AdventureState {
    /// What a step of `player`'s hero `mover`, moving by `mode` (not [`Mode::Walk`]: a walking
    /// hero's step is `check_step`'s), allows from `from`, a node of kind `node`, onto `pos` in
    /// `direction` (§7's table; ADV-MOV-007, ADV-MOV-009, ADV-MOV-010), or why it may not be made
    /// (`step` names the step in the error). In order:
    ///
    /// - off the map, then an unexplored tile (`ADV-QUIRK-15`: never, a flight included; nothing
    ///   about a hidden tile leaks), then a step down onto the visitable tile of an object not
    ///   visited from above (never): rejected; a step up off such a tile is a flight's pass only
    ///   (ADV-OBJ-004);
    /// - rock: never; water: a flight passes; a water walk passes plain water (no object, no hero);
    /// - land: an own hero's tile a flight passes; another player's hero is visited from a stop
    ///   node (a battle, GAM-BAT-004) and a flight passes it; a tile an object blocks a flight
    ///   passes; an object's visitable tile whose visit is not built a flight passes; a border
    ///   gate whose key the owner holds is a free tile (ADV-VIS-005); an entered object is visited
    ///   from any node, a neighbour's from a stop node (ADV-OBJ-003), and a flight passes both; a
    ///   free tile is a stop.
    ///
    /// A water walk is a walk on land: what a flight passes there it may not enter.
    // rule: ADV-MOV-007, ADV-MOV-008, ADV-MOV-009, ADV-MOV-010, ADV-OBJ-003, ADV-OBJ-004
    pub(crate) fn passage<'p>(
        &self,
        part: &'p AdventureRuleset,
        (player, mover, mode): (PlayerId, HeroId, Mode),
        (from, node): (Pos, Node),
        pos: Pos,
        direction: usize,
        step: usize,
    ) -> Result<Entry<'p>, AdventureCommandError> {
        let blocked = |error| AdventureCommandError::Blocked { step, error };
        let tile = *self.map.tile(pos).ok_or(blocked(StandError::Outside))?;
        // `ADV-QUIRK-15` (A-13 fixed for now): a flight never crosses an unexplored tile.
        if !self.is_explored(player, pos) {
            return Err(AdventureCommandError::Unexplored { step });
        }
        let dy = DIRECTIONS[direction].1;
        if dy == 1 && self.closed_above(part, pos) {
            return Err(AdventureCommandError::FromAbove { step });
        }
        let fly = mode == Mode::Fly;
        let tile = self.live(pos, tile);
        let pass = Entry {
            tile,
            stop: None,
            pass: true,
        };
        // Up off the visitable tile of an object not visited from above: a flight passes, landing
        // nowhere (§3.1 point 3); nothing else steps there.
        if dy == -1 && self.closed_above(part, from) {
            return match part.passage(tile.terrain) {
                Some(Passage::Land(_) | Passage::Water(_)) if fly => Ok(pass),
                Some(Passage::Land(_) | Passage::Water(_)) => {
                    Err(AdventureCommandError::FromAbove { step })
                }
                _ => Err(blocked(StandError::Terrain)),
            };
        }
        let hero = self
            .heroes
            .iter()
            .position(|h| h.pos == pos && h.id != mover);
        match part.passage(tile.terrain) {
            Some(Passage::Land(_)) => {}
            Some(Passage::Water(_)) => {
                // Plain water a water walk passes; water holding an object or a hero only a
                // flight (boats later).
                let plain = !tile.blocked && !tile.visitable && hero.is_none();
                return if fly || plain {
                    Ok(pass)
                } else {
                    Err(blocked(StandError::Blocked))
                };
            }
            _ => return Err(blocked(StandError::Terrain)),
        }
        let visit = |met: Option<Meets<'p>>| {
            if met.is_none() && !fly {
                return Err(AdventureCommandError::NoStop { step });
            }
            Ok(Entry {
                tile,
                stop: met.map(Some),
                pass: fly,
            })
        };
        if let Some(other) = hero {
            if self.heroes[other].owner == player {
                return if fly {
                    Ok(pass)
                } else {
                    Err(blocked(StandError::Occupied))
                };
            }
            // A battle, from a stop node only (GAM-BAT-004).
            return visit((node == Node::Stop).then_some(Meets::Hero(other)));
        }
        if !tile.visitable {
            return match (tile.blocked, fly) {
                (false, _) => Ok(Entry {
                    tile,
                    stop: Some(None),
                    pass: false,
                }),
                (true, true) => Ok(pass),
                (true, false) => Err(blocked(StandError::Blocked)),
            };
        }
        match self.visit_at(part, player, pos) {
            None if fly => Ok(pass),
            None => Err(AdventureCommandError::Visit { step }),
            Some(object) if object.pass => Ok(Entry {
                tile,
                stop: Some(None),
                pass: false,
            }),
            Some(object) => visit(
                (object.mode == Visit::Enter || node == Node::Stop)
                    .then_some(Meets::Object(object)),
            ),
        }
    }
}
