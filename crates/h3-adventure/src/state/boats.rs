//! The boats (ADR-0016 point 1, ADV-SEA-001; E08 T72, increment 8's design note §6): no objects of
//! the static map, because they move, but records in a persistent table by boat index (the setup's
//! order) with the tiles where an empty boat lies as a persistent grid, which the movement rules
//! and the reachable-tiles query read (T73). A map without boats has none, so a fork costs nothing
//! more (`bench-check`); with boats a fork costs two reference counts, and a boarding, a landing or
//! a boat leaving the map writes one record and one leaf. A boarding and a landing (E08 T73;
//! ADV-SEA-002, ADV-SEA-003) are here: the boat's record, the hero's boat and its points and
//! maximum on its new surface, and their events; since E08 T74b a boat leaving the map with its
//! hero after a battle too (ADV-SEA-006).

use alloc::sync::Arc;

use h3_content::Ruleset;
use h3_contracts::{HeroId, PlayerId};
use h3_heroes::{HeroDetails, free_boarding};
use h3_kernel::{CanonicalHash, StateHasher};

use super::{AdventureState, StaticMap};
use crate::command::{AdventureEvent, AdventureEventKind, Sink, Viewers};
use crate::movement::{converted_points, day_points, details_of, sea_day_points};
use crate::persistent::{BitGrid, Table};
use crate::ruleset::{AdventureRuleset, Passage};
use crate::setup::{BoatSetup, Pos, SetupError};

/// The rules a boarding and a landing read: the core, the ruleset part and the game's store of
/// hero details.
type Rules<'a> = (&'a Ruleset, &'a AdventureRuleset, &'a [Arc<HeroDetails>]);

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
    /// Its facing in the original's order of eight directions, clockwise from up:
    /// [`SETUP_FACING`] for the map's, a landing's step direction since E08 T73.
    pub facing: u8,
    /// The player of the last hero that boarded it: kept, read by no rule until Summon Boat, never
    /// shown (an empty boat shows no flag).
    pub owner: Option<PlayerId>,
    /// The last hero that boarded it: kept, read by no rule until Summon Boat, never shown.
    pub captain: Option<HeroId>,
    /// Whether its captain is aboard.
    pub aboard: bool,
    /// Whether it left the map with its hero (ADV-SEA-006, T74b): the record stays, so indexes
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

    /// Takes the empty boat at `slot` aboard for `captain` of `owner` (a boarding): off the
    /// empty boats' grid, its captain, owner and aboard set, its tile kept; its type, if the
    /// boat exists.
    fn take(&mut self, slot: usize, (captain, owner): (HeroId, PlayerId)) -> Option<u8> {
        self.records.get(slot).copied().map(|mut record| {
            self.empty.set(record.tile, false);
            record.aboard = true;
            record.captain = Some(captain);
            record.owner = Some(owner);
            self.records.set(slot, Some(record));
            record.kind
        })
    }

    /// Leaves the boat at `slot` empty on `water` facing `facing` (a landing), its owner and
    /// captain kept; its type, if the boat exists.
    fn leave(&mut self, slot: usize, water: Pos, facing: u8) -> Option<u8> {
        self.records.get(slot).copied().map(|mut record| {
            record.tile = water;
            record.facing = facing;
            record.aboard = false;
            self.empty.set(water, true);
            self.records.set(slot, Some(record));
            record.kind
        })
    }

    /// The boat at `slot` leaves the map from `at` with its hero (ADV-SEA-006): gone, its tile
    /// `at`, nobody aboard, its owner and captain kept (the original's freed record keeps them,
    /// batch E8's E8-11); its type, if the boat exists. A boat aboard is on no empty boats' tile.
    fn remove(&mut self, slot: usize, at: Pos) -> Option<u8> {
        self.records.get(slot).copied().map(|mut record| {
            record.tile = at;
            record.aboard = false;
            record.gone = true;
            self.records.set(slot, Some(record));
            record.kind
        })
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

    /// The index of the empty boat lying on `pos`, if one does (E08 T73). A boat aboard keeps the
    /// tile it was boarded on, so the record is the one neither aboard nor gone.
    pub(crate) fn empty_boat(&self, pos: Pos) -> Option<u32> {
        let boats = self.boats.as_ref().filter(|boats| boats.empty_at(pos))?;
        boats
            .iter()
            .find(|(_, boat)| boat.tile == pos && !boat.aboard && !boat.gone)
            .and_then(|(index, _)| u32::try_from(index).ok())
    }

    /// A boarding (ADV-SEA-002, E08 T73; survey `E08-sea.md` §3.1, the original's
    /// `hero_boards_boat` 0x4A1010): `player`'s hero at `index` among the heroes, which a step
    /// brought onto the tile of the empty boat `boat`, is in it there. The boat leaves the empty
    /// boats; its captain becomes the hero and its owner the player (any hero boards any empty
    /// boat, ADV-SEA-001), aboard; its record keeps its tile (the original's, batch E8's E8-13),
    /// the hero's tile being the boat's while aboard. The day's maximum becomes the sea's
    /// ([`sea_day_points`]) and the points are converted ([`Self::change_surface`]). Told as
    /// [`AdventureEventKind::Boarded`] to the players that see heroes on the tile, after the step's
    /// own event.
    // rule: ADV-SEA-002
    pub(crate) fn board<S: Sink>(
        &mut self,
        (core, part, heroes): Rules<'_>,
        (player, index): (PlayerId, usize),
        boat: u32,
        sink: &mut S,
    ) {
        let details = details_of(heroes, self.heroes[index].id);
        self.change_surface(core, details, index, sea_day_points(core, part, details));
        let hero = &mut self.heroes[index];
        hero.boat = Some(boat);
        let (id, at) = (hero.id, hero.pos);
        let seen_by = self.viewers(core, player, at);
        let slot = usize::try_from(boat).unwrap_or(usize::MAX);
        let kind = self
            .boats
            .as_mut()
            .and_then(|boats| boats.take(slot, (id, player)))
            .unwrap_or_default();
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::Boarded {
                hero: id,
                boat: kind,
                at,
            },
            seen_by,
        });
    }

    /// A landing (ADV-SEA-003, E08 T73; survey `E08-sea.md` §3.2, the original's
    /// `hero_lands_from_boat` 0x49E670 after the step's 0x48095A): `player`'s hero at `index`
    /// among the heroes, which a step from the water tile `water` in `direction` brought out of
    /// its boat `boat` onto a free coastal tile, leaves the boat there: empty on `water`, facing
    /// the step's direction (the original's order of [`crate::movement::DIRECTIONS`], clockwise
    /// from up: batch E8's E8-2 east 2, E8-15 south 4), nobody aboard, its owner and captain
    /// kept. The day's maximum becomes the land's ([`day_points`]) and the points are converted
    /// ([`Self::change_surface`]). Told as [`AdventureEventKind::Landed`] to the players that see
    /// heroes on `water`, after the step's own event; the landing tile's guards visit after it.
    // rule: ADV-SEA-003
    pub(crate) fn land<S: Sink>(
        &mut self,
        (core, part, heroes): Rules<'_>,
        (player, index): (PlayerId, usize),
        (boat, water, direction): (u32, Pos, usize),
        sink: &mut S,
    ) {
        let details = details_of(heroes, self.heroes[index].id);
        let maximum = day_points(core, part, details, &self.heroes[index].army);
        self.change_surface(core, details, index, maximum);
        let hero = &mut self.heroes[index];
        hero.boat = None;
        let id = hero.id;
        let seen_by = self.viewers(core, player, water);
        let facing = direction as u8;
        let slot = usize::try_from(boat).unwrap_or(usize::MAX);
        let kind = self
            .boats
            .as_mut()
            .and_then(|boats| boats.leave(slot, water, facing))
            .unwrap_or_default();
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::Landed {
                hero: id,
                boat: kind,
                at: water,
                facing,
            },
            seen_by,
        });
    }

    /// A boat leaving the map with its hero (ADV-SEA-006, E08 T74b; survey `E08-sea.md` §7.3, the
    /// original's 0x4D9EC0 freeing the boat's record at 0x4DA009): `hero`, which a battle took off
    /// the map from the water tile `at`, was in the boat `boat`, which leaves with it: gone, its
    /// record kept by index ([`Boat::gone`]), so it lies nowhere and nobody boards it again. Told
    /// as [`AdventureEventKind::BoatRemoved`] to `seen_by`, the hero's removal's viewers (the
    /// players that see heroes on the tile, ADV-FOG-006), after that removal.
    // rule: ADV-SEA-006
    pub(crate) fn remove_boat<S: Sink>(
        &mut self,
        (hero, boat, at): (HeroId, u32, Pos),
        seen_by: Viewers,
        sink: &mut S,
    ) {
        let slot = usize::try_from(boat).unwrap_or(usize::MAX);
        let kind = self
            .boats
            .as_mut()
            .and_then(|boats| boats.remove(slot, at))
            .unwrap_or_default();
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::BoatRemoved {
                hero,
                boat: kind,
                at,
            },
            seen_by,
        });
    }

    /// The conversion of a boarding or a landing (D-41, ADR-0016 point 3; increment 8's design note
    /// §8): the hero at `index` keeps [`converted_points`] of its points, after the step's drop,
    /// with free boarding (`h3_heroes::free_boarding`, ADR-0013 point 13.7: the Admiral's Hat), and
    /// its day's maximum becomes `maximum`, its new surface's. The original keeps the old maximum
    /// without the Hat until the next turn start (A-28, confirmed by batch E8's E8-10 and E8-16):
    /// not reproduced (`ADV-QUIRK-31`, no flag; E08 decision 166).
    // rule: ADV-SEA-002, ADV-SEA-003
    fn change_surface(
        &mut self,
        core: &Ruleset,
        details: Option<&HeroDetails>,
        index: usize,
        maximum: u32,
    ) {
        let free = details.is_some_and(|details| free_boarding(details, core));
        let hero = &mut self.heroes[index];
        hero.movement = converted_points(hero.movement, (hero.max_movement, maximum), free);
        hero.max_movement = maximum;
    }

    /// Whether the boats keep the state's invariants (ADR-0016 point 1, increment 8's design note
    /// §6; the properties check it after every command): every empty boat's tile is in the empty
    /// boats' grid and no other tile is, no two empty boats share a tile and each lies on water; a
    /// boat aboard has exactly one hero holding it, its captain, on a water tile, where no empty
    /// boat lies, and is not gone; a hero's boat is such a boat, and a hero on water is in a boat.
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
    /// Puts `hero` aboard the empty boat `index` on the boat's tile, as a boarding does without
    /// its step and conversion ([`Self::board`]), its player the boat's owner: the tests of a
    /// hero in a boat.
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
