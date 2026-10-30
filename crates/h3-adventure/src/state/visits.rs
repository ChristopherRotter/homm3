//! The visits' block (ADR-0016 point 1, E08 decision 133; increment 7's design note §10 in
//! `E08-log.md`): what visits leave on the map's objects beyond their removal
//! (`Occupancy`'s), with no bound of the original's on how many objects carry it (A-17's 32 marks
//! per class not reproduced): each hero's once-per-hero marks, each player's knowledge of what a
//! visit left (ADV-VIS-006, §13) and the objects emptied (a find searched, a Pyramid looted). The
//! state holds one only once a visit writes it, then one block the forks share (a write copies
//! it, its bit grids sharing their leaves). E08 T61b; the visits that write it come with T62.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use h3_contracts::{HeroId, PlayerId};
use h3_kernel::{CanonicalHash, StateHasher};

use crate::persistent::BitGrid;
use crate::setup::Pos;

/// Objects per row of an [`ObjectBits`]: a leaf's side, so a row of objects is one word, as
/// `Occupancy`'s removed objects.
const ROW: usize = BitGrid::<16>::SIDE as usize;

/// A bit per object of the static map, by index: object `i` is column `i % 32` of row `i / 32`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ObjectBits(BitGrid);

impl ObjectBits {
    /// No bit set, for `objects` objects (at most `32 × u16::MAX`, as the static map holds).
    fn new(objects: usize) -> Self {
        let rows = u16::try_from(objects.div_ceil(ROW)).unwrap_or(u16::MAX);
        Self(BitGrid::new(BitGrid::<16>::SIDE, rows, 1))
    }

    /// The grid tile standing for object `index`.
    fn slot(index: usize) -> Pos {
        let column = u16::try_from(index % ROW).unwrap_or(u16::MAX);
        Pos::new(column, u16::try_from(index / ROW).unwrap_or(u16::MAX), 0)
    }

    /// Whether object `index`'s bit is set (an index past the objects: no).
    fn get(&self, index: usize) -> bool {
        self.0.get(Self::slot(index))
    }

    /// Sets object `index`'s bit (an index past the objects: nothing).
    fn set(&mut self, index: usize) {
        self.0.set(Self::slot(index), true);
    }
}

/// What visits left on the map's objects (increment 7's design note §10): a hero's marks, a
/// player's knowledge and the objects emptied, each a bit per object index. A value only the state
/// stores: building or changing one changes no game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Visits {
    /// How many objects the static map holds.
    objects: usize,
    /// Each hero's once-per-hero marks (the once-per-hero objects' visited flags), by hero id;
    /// none for a hero that marked nothing.
    marks: BTreeMap<HeroId, ObjectBits>,
    /// Each player's knowledge of what a visit left on an object, by player id (ADV-VIS-006: the
    /// visitor's owner's, the team's marks as its own until E10).
    known: Vec<ObjectBits>,
    /// The objects a visit emptied: a find searched, a Pyramid looted.
    emptied: ObjectBits,
}

impl Visits {
    /// Nothing marked, known or emptied, for a static map of `objects` objects and a game of
    /// `players` players.
    pub fn new(objects: usize, players: u8) -> Self {
        Self {
            objects,
            marks: BTreeMap::new(),
            known: (0..players).map(|_| ObjectBits::new(objects)).collect(),
            emptied: ObjectBits::new(objects),
        }
    }

    /// Whether `hero` has marked the object at `index` (a once-per-hero object it visited).
    pub fn is_marked(&self, hero: HeroId, index: usize) -> bool {
        self.marks.get(&hero).is_some_and(|marks| marks.get(index))
    }

    /// `hero` marks the object at `index` (an index past the objects: nothing).
    pub fn mark(&mut self, hero: HeroId, index: usize) {
        if index < self.objects {
            self.marks
                .entry(hero)
                .or_insert_with(|| ObjectBits::new(self.objects))
                .set(index);
        }
    }

    /// Whether `player` knows what a visit left on the object at `index` (a player not in the
    /// game knows nothing).
    pub fn knows(&self, player: PlayerId, index: usize) -> bool {
        self.known
            .get(usize::from(player.raw()))
            .is_some_and(|known| known.get(index))
    }

    /// `player` learns what a visit left on the object at `index` (a player not in the game:
    /// nothing).
    pub fn learn(&mut self, player: PlayerId, index: usize) {
        if let Some(known) = self.known.get_mut(usize::from(player.raw())) {
            known.set(index);
        }
    }

    /// Whether a visit emptied the object at `index`.
    pub fn is_emptied(&self, index: usize) -> bool {
        self.emptied.get(index)
    }

    /// A visit empties the object at `index`.
    pub fn empty(&mut self, index: usize) {
        self.emptied.set(index);
    }
}

impl CanonicalHash for Visits {
    /// The marks as their number and, in hero id order, the hero's id and its grid; each player's
    /// knowledge as their number and grids in player order; the emptied objects' grid
    /// ([`BitGrid`]'s encoding). The number of objects follows from the static map.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_len(self.marks.len());
        for (hero, marks) in &self.marks {
            h.write_u16(hero.raw());
            marks.0.hash_into(h);
        }
        h.write_len(self.known.len());
        for known in &self.known {
            known.0.hash_into(h);
        }
        self.emptied.0.hash_into(h);
    }
}
