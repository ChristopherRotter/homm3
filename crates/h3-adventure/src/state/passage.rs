//! The passage of a flying or water-walking hero (E08 T41; ADV-MOV-007, ADV-MOV-009, ADV-MOV-010;
//! survey `E08-hero-movement.md` §3, the second reading; increment 5's design note §7's table,
//! `E08-log.md`): what a step may enter, whether the hero may stop there or only pass over, and
//! what it visits. One place for `MoveHero` and the reachable-tiles query, so that a later
//! compromise for the unexplored tiles (`ADV-QUIRK-15`, ADR-0016 point 4) changes one row here. A
//! walking hero's steps keep their own checks (`check_step` and the walking search), whose rows
//! this table repeats for walking. Since E08 T73 (increment 8, design §7) the table has a fourth
//! column, a hero in a boat ([`Mode::Sail`]), and the rows of boats: boarding an empty boat, a
//! hero in a boat, a landing onto a free coastal tile, and a water tile an object is visited from
//! closed to a flight (D-46).

use h3_contracts::{HeroId, PlayerId};
use h3_heroes::Movement;

use super::{AdventureState, Meets};
use crate::command::AdventureCommandError;
use crate::movement::DIRECTIONS;
use crate::ruleset::{AdventureRuleset, Passage, Visit};
use crate::setup::{Pos, StandError};
use crate::state::StaticTile;

/// How a hero moves over the map (ADR-0016 point 4): walking, flying (a flying level, ADV-MOV-009)
/// or water walking (a water-walking level, ADV-MOV-010); a hero that does both flies. A hero in a
/// boat sails, whatever its levels (ADR-0016 point 3; E08 T73).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    /// Neither level.
    Walk,
    /// A flying level.
    Fly,
    /// A water-walking level and no flying level.
    WaterWalk,
    /// In the boat of this index (ADV-SEA-004): neither flying nor water walking (the original
    /// clears both levels at a boarding, survey `E08-sea.md` §3.1).
    Sail(u32),
}

impl Mode {
    /// How a hero with these levels moves (`h3_heroes::movement`), in `boat` if it is in one.
    pub(crate) const fn of(movement: &Movement, boat: Option<u32>) -> Self {
        match (boat, movement.flying, movement.water_walking) {
            (Some(boat), _, _) => Self::Sail(boat),
            (None, Some(_), _) => Self::Fly,
            (None, None, Some(_)) => Self::WaterWalk,
            (None, None, None) => Self::Walk,
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
    /// - a hero in a boat ([`Mode::Sail`]): [`Self::sail_entry`]'s rows;
    /// - rock: never; water (E08 T73, ADR-0016 point 4): a hero in a boat is visited from a stop
    ///   node and a flight passes it; an empty boat is boarded from a stop node
    ///   ([`Meets::Board`], ADV-SEA-002) and a flight passes it, a water walk never steps onto it
    ///   from water; a tile an object is visited from (a pickup's, a monster's, a water
    ///   Cartographer's, a blocker's) neither (D-46, confirmed by batch E8c: a flight neither ends
    ///   on nor passes over it); a flight passes the rest, a water walk plain water (no object);
    /// - land: another hero is visited from a stop node (another player's in a battle,
    ///   GAM-BAT-004; an own one in a meeting, GAM-MET-001, E08 T50) and a flight passes it; a
    ///   tile an object blocks a flight
    ///   passes; an object's visitable tile whose visit is not built a flight passes; a border
    ///   gate whose key the owner holds is a free tile (ADV-VIS-005); an entered object is visited
    ///   from any node, a neighbour's from a stop node (ADV-OBJ-003), and a flight passes both; a
    ///   free tile is a stop.
    ///
    /// A water walk is a walk on land: what a flight passes there it may not enter. The original's
    /// class 85 (the Shipwreck), whose water tile a flight passes (0x4B26D6), is an untyped
    /// blocker here: its tile is closed as every other object's on water until creature banks type
    /// it (ADV-STUB-12).
    // rule: ADV-MOV-007, ADV-MOV-008, ADV-MOV-009, ADV-MOV-010, ADV-OBJ-003, ADV-OBJ-004
    // rule: ADV-SEA-002, ADV-PIK-018
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
        if let Mode::Sail(boat) = mode {
            return self.sail_entry(part, (player, boat), (from, pos), tile, hero, step);
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
        match part.passage(tile.terrain) {
            Some(Passage::Land(_)) => {}
            Some(Passage::Water(_)) => {
                if let Some(other) = hero {
                    // A hero in a boat: a battle or a meeting from a stop node (GAM-BAT-004,
                    // GAM-MET-001; the deck, T74).
                    return visit((node == Node::Stop).then_some(Meets::Hero(other)));
                }
                if let Some(boat) = self.empty_boat(pos) {
                    // Boarded from a stop node, whatever the mode (ADV-SEA-002); a flight passes
                    // over it from land or water (batch E8c's E8-17d, E8-17e), a water walk never
                    // steps onto it from water.
                    return match (node, fly) {
                        (Node::Stop, _) => Ok(Entry {
                            tile,
                            stop: Some(Some(Meets::Board(boat))),
                            pass: fly,
                        }),
                        (Node::Pass, true) => Ok(pass),
                        (Node::Pass, false) => Err(AdventureCommandError::Boat { step }),
                    };
                }
                // An object's tile on water only a boat reaches (D-46); plain water a flight and
                // a water walk pass, an object's blocked cell only a flight.
                // stub: ADV-STUB-12 (class 85's water tile, a flight's pass in the original)
                let open = !tile.visitable && (fly || !tile.blocked);
                return if open {
                    Ok(pass)
                } else {
                    Err(blocked(StandError::Blocked))
                };
            }
            _ => return Err(blocked(StandError::Terrain)),
        }
        if let Some(other) = hero {
            // A battle or a meeting, from a stop node only (GAM-BAT-004, GAM-MET-001).
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

    /// What a step of `player`'s hero in the boat `boat` allows from the water tile `from` onto
    /// `pos`, whose tile is `tile` as it stands now and on which the hero at index `hero` stands,
    /// if one does (design §7's last column; ADV-SEA-003 to -005, E08 T73). Every entry is a stop,
    /// never a pass:
    ///
    /// - land: a landing ([`Meets::Land`]) onto a free coastal tile ([`Self::free_coast`], D-42),
    ///   else [`AdventureCommandError::Landing`] (a hero on land among them: a boat never reaches
    ///   it); a diagonal landing has no side test (0x4B2867, 0x481CDB); rock: never;
    /// - water: a diagonal only when both side tiles are water (0x4B2703, 0x481D33), else
    ///   [`AdventureCommandError::Coast`]; another hero (in a boat) visited from the boat's tile, a
    ///   battle or a meeting; never an empty boat ([`AdventureCommandError::Boat`], 0x4B26EB); an
    ///   object visited from the neighbouring tile visited from the boat's, an entered one
    ///   entered, a visit not built refused (ADV-SEA-005, ADV-OBJ-003); a blocked tile never;
    ///   plain water a stop.
    // rule: ADV-SEA-003, ADV-SEA-004, ADV-SEA-005
    fn sail_entry<'p>(
        &self,
        part: &'p AdventureRuleset,
        (player, boat): (PlayerId, u32),
        (from, pos): (Pos, Pos),
        tile: StaticTile,
        hero: Option<usize>,
        step: usize,
    ) -> Result<Entry<'p>, AdventureCommandError> {
        let stop = |met| {
            Ok(Entry {
                tile,
                stop: Some(met),
                pass: false,
            })
        };
        let water = |at: Pos| {
            matches!(
                part.passage(self.map.at(at).terrain),
                Some(Passage::Water(_))
            )
        };
        match part.passage(tile.terrain) {
            Some(Passage::Water(_)) => {}
            Some(Passage::Land(_)) if hero.is_none() && self.free_coast(pos, &tile) => {
                return stop(Some(Meets::Land(boat)));
            }
            Some(Passage::Land(_)) => return Err(AdventureCommandError::Landing { step }),
            _ => {
                return Err(AdventureCommandError::Blocked {
                    step,
                    error: StandError::Terrain,
                });
            }
        }
        let sides = [
            Pos::new(pos.x, from.y, from.level),
            Pos::new(from.x, pos.y, from.level),
        ];
        if !sides.into_iter().all(water) {
            return Err(AdventureCommandError::Coast { step });
        }
        if let Some(other) = hero {
            return stop(Some(Meets::Hero(other)));
        }
        if self.is_empty_boat(pos) {
            return Err(AdventureCommandError::Boat { step });
        }
        if !tile.visitable {
            return if tile.blocked {
                Err(AdventureCommandError::Blocked {
                    step,
                    error: StandError::Blocked,
                })
            } else {
                stop(None)
            };
        }
        match self.visit_at(part, player, pos) {
            None => Err(AdventureCommandError::Visit { step }),
            Some(object) if object.pass => stop(None),
            Some(object) => stop(Some(Meets::Object(object))),
        }
    }

    /// Whether a boat may land on `pos`, whose tile is `tile` as it stands now, no hero standing
    /// there (D-42, decision 162; ADV-SEA-003; the original's class 3, survey `E08-sea.md` §3.2): a
    /// tile on the map's coast that no object still on the map blocks or is visited from, with no
    /// event still on the map (whoever it fires for: the original's tile records it) and no hole.
    /// A tile an object left becomes one again (0x5056A5). The refusal tells of an event on a
    /// coastal tile, as the original's path does (batch E8's E8-3).
    // rule: ADV-SEA-003
    pub(crate) fn free_coast(&self, pos: Pos, tile: &StaticTile) -> bool {
        self.map.is_coast(pos)
            && !tile.blocked
            && !tile.visitable
            && !tile.hole
            && self.event_at(pos).is_none()
    }
}
