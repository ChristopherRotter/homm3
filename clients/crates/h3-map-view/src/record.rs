//! What a map tile records of the objects on it (E24 M3 T3a, *provisional*;
//! `docs/epics/E24-design.md` §4.4, `docs/rules/surveys/E08-objects.md` §2.5): the one object
//! `tile_refresh_object` 0x505610 takes from the tile's draw list, walking it from the top: the
//! first whose template marks the tile visitable (the tile its entrance, +0xD bit 0x10); without
//! one the first object of an ownable class that blocks the tile, else the first that blocks it
//! (both clear the tile's +0xC bit 0x40: an object blocks it), else the first hole; else none
//! (the class 0, 3 on coastal land). Its class as the game loads it (`object_class_values`
//! 0x63A6E4). The map's sound (a tile's ambient loop, M2 T5) and the minimap (M3 T3a) read it.
//! Heroes and boats have no draw entries: a hero standing on a tile is the caller's (its class 34
//! with the entrance bit, `hero_show` 0x4D75D0). A rock tile's bit 0x40, clear from the load
//! (0x4FE220), is not modelled: both its minimap colours are black.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use serde::Deserialize;

use crate::art::{MapArt, TileArt};
use crate::live::Live;
use crate::objects::{Entry, template};

/// The class a tile without an object keeps (0x505610's reset).
pub const NO_CLASS: u32 = 0;
/// The class coastal land keeps without an object.
pub const COAST_CLASS: u32 = 3;

/// The asset tables' `[map.records]` (E24 M2 T5's, moved from `[map.sound]` at M3 T3a): how the
/// game loads a map's classes and which objects a tile without an entrance records.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordArt {
    /// The map's classes the game loads as others, `[map's, game's]` (`object_class_values`
    /// 0x63A6E4).
    pub loaded_as: Vec<[u32; 2]>,
    /// The ownable classes (`class_is_ownable` 0x40FCA0): a tile without an entrance records the
    /// topmost one blocking it first (0x505610).
    pub ownable: Vec<u32>,
    /// The class a tile without an entrance or a blocking object records when an object of it
    /// lies there (124, the hole).
    pub hole: u32,
}

/// The tables' records resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Records {
    loaded_as: BTreeMap<u32, u32>,
    ownable: Vec<u32>,
    hole: u32,
}

/// What a tile records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Record {
    /// The object (its index among the map's), none for none.
    pub object: Option<usize>,
    /// Its class as the game loads it; without an object [`NO_CLASS`], or [`COAST_CLASS`] on
    /// coastal land.
    pub class: u32,
    /// Its class as the map gives it (what a random object became, else its template's).
    pub map_class: u32,
    /// Its subclass (0 without an object).
    pub subclass: u32,
    /// The tile is the object's entrance (+0xD bit 0x10).
    pub entrance: bool,
    /// An object blocks the tile (+0xC bit 0x40 cleared by the blocking walks).
    pub blocked: bool,
}

impl Records {
    /// The rules of `art`.
    #[must_use]
    pub fn new(art: &RecordArt) -> Self {
        Self {
            loaded_as: art
                .loaded_as
                .iter()
                .map(|&[map, game]| (map, game))
                .collect(),
            ownable: art.ownable.clone(),
            hole: art.hole,
        }
    }

    /// The class the game loads a map's `class` as.
    #[must_use]
    pub fn game_class(&self, class: u32) -> u32 {
        self.loaded_as.get(&class).copied().unwrap_or(class)
    }

    /// The classes of entry `e`'s object: the game's, its subclass and the map's (what a random
    /// object became, else its template's); none for an object without a template.
    #[must_use]
    pub fn classes(&self, art: &MapArt, live: &Live, e: &Entry) -> Option<(u32, u32, u32)> {
        template(art, e.object).map(|t| {
            let drawn = live.objects.get(e.object).and_then(|o| o.drawn.as_ref());
            let (class, subclass) = drawn.map_or((t.class, t.subclass), |d| (d.class, d.subclass));
            (self.game_class(class), subclass, class)
        })
    }

    /// What `tile`, whose draw list is `entries` (bottom to top), records (0x505610's walks).
    #[must_use]
    pub fn record(&self, art: &MapArt, live: &Live, tile: &TileArt, entries: &[Entry]) -> Record {
        let top = || entries.iter().rev();
        let found = |e: &Entry, entrance: bool, blocked: bool| {
            self.classes(art, live, e)
                .map(|(class, subclass, map_class)| Record {
                    object: Some(e.object),
                    class,
                    map_class,
                    subclass,
                    entrance,
                    blocked,
                })
        };
        if let Some(record) = top()
            .find(|e| template(art, e.object).is_some_and(|t| t.visitable(e.dx, e.dy)))
            .and_then(|e| found(e, true, false))
        {
            return record;
        }
        // Without an entrance: the topmost ownable object blocking it, else the topmost blocking
        // one, else the hole, else none.
        let blocks = |e: &&Entry| template(art, e.object).is_some_and(|t| !t.passable(e.dx, e.dy));
        let class_of = |e: &Entry| self.classes(art, live, e).map(|(c, _, _)| c);
        top()
            .filter(blocks)
            .find(|e| class_of(e).is_some_and(|c| self.ownable.contains(&c)))
            .or_else(|| top().find(blocks))
            .and_then(|e| found(e, false, true))
            .or_else(|| {
                top()
                    .find(|e| class_of(e) == Some(self.hole))
                    .and_then(|e| found(e, false, false))
            })
            .unwrap_or_else(|| {
                // Land with the map file's coast bit (0x4FE3D6; E24 M3 T6a1: not water of a
                // shore picture, which has the bit 0x02 alone).
                let coast = tile.coast() && tile.terrain.code != crate::art::WATER;
                let class = if coast { COAST_CLASS } else { NO_CLASS };
                Record {
                    object: None,
                    class,
                    map_class: class,
                    subclass: 0,
                    entrance: false,
                    blocked: false,
                }
            })
    }
}

#[cfg(test)]
mod tests;
