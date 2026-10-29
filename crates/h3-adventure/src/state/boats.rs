//! The boats (ADR-0016 point 1, ADV-SEA-001; E08 T72, design §6): no objects of the static map,
//! because they move, but records in a persistent table by boat index (the setup's order) with the
//! tiles where an empty boat lies as a persistent grid, which the movement rules and the
//! reachable-tiles query read (T73). A map without boats has none, so a fork costs nothing more
//! (`bench-check`); with boats a fork costs two reference counts, and a boarding, a landing or a
//! boat leaving the map writes one record and one leaf.

use h3_contracts::{HeroId, PlayerId};
use h3_kernel::{CanonicalHash, StateHasher};

use super::{AdventureState, StaticMap};
use crate::persistent::{BitGrid, Table};
use crate::ruleset::{AdventureRuleset, Passage};
use crate::setup::{BoatSetup, Pos, SetupError};

/// The original's facing of a boat the map places (`create_boat` 0x4BB250; survey `E08-sea.md`
/// §2): 2 in its order of eight directions.
pub const SETUP_FACING: u8 = 2;

/// A boat (ADR-0016 point 1, ADV-SEA-001): the original's record (`structs.toml` "boat") without
/// its 64-record bound (decision 164, `ADV-QUIRK-32`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Boat {
    /// Where it lies: an empty boat's tile; while a hero is aboard, the tile it was boarded on
    /// (the hero's tile is the boat's, T73).
    pub tile: Pos,
    /// Its type, appearance only (0 to 2 in the original).
    pub kind: u8,
    /// Its facing in the original's order of eight directions: [`SETUP_FACING`] for the map's, a
    /// landing's step direction from T73.
    pub facing: u8,
    /// The player of the last hero that boarded it: kept, read by no rule until Summon Boat, never
    /// shown (an empty boat shows no flag).
    pub owner: Option<PlayerId>,
    /// The last hero that boarded it: kept, read by no rule until Summon Boat, never shown.
    pub captain: Option<HeroId>,
    /// Whether its captain is aboard.
    pub aboard: bool,
    /// Whether it left the map with its hero (ADV-SEA-006, T74): the record stays, so indexes
    /// never move.
    pub gone: bool,
}

/// The boats of a map with boats: their records by index and the tiles empty ones lie on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Boats {
    /// The records, by boat index; every index below their number holds one.
    pub(super) records: Table<Boat>,
    /// The tiles where an empty boat lies; follows from the records, so the hash leaves it out.
    pub(super) empty: BitGrid,
}

impl Boats {
    /// The setup's boats on `map`, each empty with the original's facing and neither owner nor
    /// captain (ADV-SEA-001); none for a map without boats.
    ///
    /// # Errors
    ///
    /// [`SetupError::Boat`] for the first boat in the setup's order that lies off the map or off
    /// water, on a tile an object blocks or is visited from, or on an earlier boat's tile (no
    /// hero stands on water: [`AdventureState::check_stand`] refuses it, so none stands on a
    /// boat's tile either).
    // rule: ADV-SEA-001
    pub(crate) fn new(
        part: &AdventureRuleset,
        map: &StaticMap,
        setup: &[BoatSetup],
    ) -> Result<Option<Self>, SetupError> {
        if setup.is_empty() {
            return Ok(None);
        }
        let mut boats = Self {
            records: Table::new(),
            empty: BitGrid::new(map.width(), map.height(), map.levels()),
        };
        for (index, boat) in setup.iter().enumerate() {
            let free = map.tile(boat.tile).is_some_and(|tile| {
                matches!(part.passage(tile.terrain), Some(Passage::Water(_)))
                    && !tile.blocked
                    && !tile.visitable
            });
            if !free || boats.empty.get(boat.tile) {
                return Err(SetupError::Boat(index));
            }
            boats.empty.set(boat.tile, true);
            boats.records.set(
                index,
                Some(Boat {
                    tile: boat.tile,
                    kind: boat.kind,
                    facing: SETUP_FACING,
                    owner: None,
                    captain: None,
                    aboard: false,
                    gone: false,
                }),
            );
        }
        Ok(Some(boats))
    }

    /// The records in index order.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (usize, &Boat)> {
        self.records.iter()
    }

    /// The record at `index`, if any.
    pub(crate) fn get(&self, index: usize) -> Option<&Boat> {
        self.records.get(index)
    }

    /// Whether an empty boat lies on `pos`.
    pub(crate) fn empty_at(&self, pos: Pos) -> bool {
        self.empty.get(pos)
    }

    /// The tiles where an empty boat lies.
    pub(crate) const fn empty_tiles(&self) -> &BitGrid {
        &self.empty
    }
}

impl CanonicalHash for Boats {
    /// The number of boats, then each in index order: the tile, the kind, the facing, the owner
    /// and the captain (0 for none, else 1 and it), whether its captain is aboard and whether it
    /// is gone.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_len(self.records.len());
        for (_, boat) in self.records.iter() {
            boat.tile.hash_into(h);
            h.write_u8(boat.kind);
            h.write_u8(boat.facing);
            match boat.owner {
                None => h.write_u8(0),
                Some(owner) => {
                    h.write_u8(1);
                    h.write_u8(owner.raw());
                }
            }
            match boat.captain {
                None => h.write_u8(0),
                Some(captain) => {
                    h.write_u8(1);
                    h.write_u16(captain.raw());
                }
            }
            h.write_bool(boat.aboard);
            h.write_bool(boat.gone);
        }
    }
}

impl AdventureState {
    /// The boats in index order (ADV-SEA-001, E08 T72): none for a map without boats.
    pub fn boats(&self) -> impl Iterator<Item = (usize, &Boat)> {
        self.boats.iter().flat_map(Boats::iter)
    }

    /// The boat at `index`, if the map has it.
    pub fn boat(&self, index: usize) -> Option<&Boat> {
        self.boats.as_ref().and_then(|boats| boats.get(index))
    }

    /// Whether an empty boat lies on `pos` (a tile off the map: no).
    pub fn is_empty_boat(&self, pos: Pos) -> bool {
        self.boats.as_ref().is_some_and(|boats| boats.empty_at(pos))
    }

    /// Whether the boats keep the state's invariants (ADR-0016 point 1, design §6; the properties
    /// check it after every command): every empty boat's tile is in the empty boats' grid and no
    /// other tile is, no two empty boats share a tile and each lies on water; a boat aboard has
    /// exactly one hero holding it, its captain, on a water tile, where no empty boat lies, and is
    /// not gone; a hero's boat is such a boat, and a hero on water is in a boat.
    pub fn check_boats(&self, part: &AdventureRuleset) -> bool {
        let water = |pos: Pos| {
            self.map
                .tile(pos)
                .is_some_and(|tile| matches!(part.passage(tile.terrain), Some(Passage::Water(_))))
        };
        let boats: alloc::vec::Vec<(usize, &Boat)> = self.boats().collect();
        let empty = |boat: &&(usize, &Boat)| !boat.1.aboard && !boat.1.gone;
        let tiles = self.boats.as_ref().map_or(0, |b| b.empty_tiles().count());
        let distinct = boats
            .iter()
            .filter(empty)
            .map(|(_, boat)| boat.tile)
            .collect::<alloc::collections::BTreeSet<_>>()
            .len();
        let empties = boats.iter().filter(empty).count();
        let holders = |index: usize| {
            self.heroes
                .iter()
                .filter(|h| h.boat == Some(u32::try_from(index).unwrap_or(u32::MAX)))
                .count()
        };
        boats
            .iter()
            .filter(empty)
            .all(|(_, boat)| self.is_empty_boat(boat.tile) && water(boat.tile))
            && usize::try_from(tiles).is_ok_and(|tiles| tiles == empties)
            && distinct == empties
            && boats
                .iter()
                .filter(|(_, boat)| boat.aboard)
                .all(|&(index, boat)| holders(index) == 1 && !boat.gone)
            && self.heroes.iter().all(|hero| {
                let aboard = hero
                    .boat
                    .and_then(|i| self.boat(usize::try_from(i).unwrap_or(usize::MAX)));
                match aboard {
                    Some(boat) => {
                        boat.aboard
                            && boat.captain == Some(hero.id)
                            && water(hero.pos)
                            && !self.is_empty_boat(hero.pos)
                    }
                    None => hero.boat.is_none() && !water(hero.pos),
                }
            })
    }
}

#[cfg(test)]
impl AdventureState {
    /// Puts `hero` aboard the empty boat `index` on the boat's tile, as a boarding will (T73), its
    /// player the boat's owner: the tests of a hero in a boat.
    pub(crate) fn put_aboard(&mut self, hero: HeroId, index: usize) {
        let boats = self.boats.as_mut().expect("a map with boats");
        let mut boat = *boats.records.get(index).expect("the boat");
        let at = self
            .heroes
            .iter()
            .position(|h| h.id == hero)
            .expect("the hero");
        boats.empty.set(boat.tile, false);
        boat.aboard = true;
        boat.captain = Some(hero);
        boat.owner = Some(self.heroes[at].owner);
        boats.records.set(index, Some(boat));
        self.heroes[at].pos = boat.tile;
        self.heroes[at].boat = Some(u32::try_from(index).expect("an index"));
    }
}
