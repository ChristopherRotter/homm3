//! What a viewer newly sees since an explored grid of its own (SRV-Q26 (a), the user's decision of
//! 2026-11-09; the server lane's design note `E16-design.md` §14): a server keeps each viewer's
//! view current by its projected events alone, and a step's reveal has no event (an object's
//! `TilesRevealed` only a count), so the server keeps each viewer's last explored grid and, when
//! the viewer's explored count grew, asks [`AdventureState::reveal`] for what the viewer sees now
//! that it did not see through that grid, and sends it with the command's events (E16 M5 T5c).
//!
//! The answer is [`AdventureState::observe`]'s view, restricted to what the grids' difference
//! decides: the tiles whose sight changed with their views, and the heroes, typed objects, empty
//! boats and towns the viewer sees through its grid now and did not through `since`, each as the
//! observation shows it (ADV-FOG-004 to ADV-FOG-008, ADV-OWN-003, TWN-VEW-001). What the viewer
//! already saw through `since` is the events' to keep current; what nothing of the grids decides
//! (its own heroes, its purchases, keys and knowledge) is never in it. Each tile's planes are read
//! once and only the changed tiles' views are built, so the answer costs the planes' words, the
//! heroes, the boats and one pass over the objects' tiles, not a tile view of the whole map.

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use h3_content::{FRINGE_SHOWS_OBJECTS, Ruleset};
use h3_contracts::{PlayerId, TownId};

use super::{BoatView, HeroView, ObjectView, TileView, boat_view, tile_view};
use crate::fog::Sight;
use crate::persistent::BitGrid;
use crate::ruleset::AdventureRuleset;
use crate::setup::Pos;
use crate::state::AdventureState;

/// What a viewer sees now that it did not see through an earlier explored grid of its own
/// ([`AdventureState::reveal`]): [`crate::AdventureObservation`]'s parts that the grids'
/// difference decides, each in the observation's order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdventureReveal {
    /// The player this answer is for.
    pub viewer: PlayerId,
    /// The tiles whose sight changed (explored, on the fringe or neither), in tile order (level,
    /// row, column), each with its view now.
    pub tiles: Vec<RevealedTile>,
    /// The heroes the viewer sees now on a tile where it did not see heroes, by id: never its
    /// own, which it always sees.
    pub heroes: Vec<HeroView>,
    /// The typed objects still on the map the viewer sees now and did not, in the map's order (the
    /// objects added in play after the static map's).
    pub objects: Vec<ObjectView>,
    /// The empty boats the viewer sees now on a tile where it did not see heroes, in tile order.
    pub boats: Vec<BoatView>,
    /// The towns a tile of which the viewer has explored now and none before (TWN-VEW-001), in
    /// `TownId` order: what it sees of them is the game's (`h3_game::reveal`).
    pub towns: Vec<TownId>,
}

/// A tile whose sight changed ([`AdventureReveal::tiles`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevealedTile {
    /// The tile.
    pub pos: Pos,
    /// Whether the viewer has explored it now; a tile not explored whose view is not
    /// [`TileView::Hidden`] is on its fringe (ADV-FOG-005).
    pub explored: bool,
    /// What the viewer sees of it now, as [`crate::AdventureObservation::tile`] shows it.
    pub view: TileView,
}

/// A sight's planes, read tile by tile.
struct Planes {
    sight: Sight,
    height: usize,
}

impl Planes {
    /// The word of the planes holding `pos`, a tile of the map, and the bit in it.
    fn locate(&self, pos: Pos) -> (usize, u32) {
        let row = usize::from(pos.level) * self.height + usize::from(pos.y);
        (
            row * self.sight.stride + usize::from(pos.x / 64),
            u32::from(pos.x % 64),
        )
    }

    /// Whether the sight has explored `pos`.
    fn explored(&self, pos: Pos) -> bool {
        let (word, bit) = self.locate(pos);
        self.sight
            .explored
            .get(word)
            .is_some_and(|w| w >> bit & 1 == 1)
    }

    /// Whether `pos` is on the sight's fringe.
    fn fringe(&self, pos: Pos) -> bool {
        let (word, bit) = self.locate(pos);
        self.sight
            .fringe
            .get(word)
            .is_some_and(|w| w >> bit & 1 == 1)
    }

    /// Whether the sight shows the heroes and the empty boats on `pos` (ADV-FOG-004, ADV-FOG-005,
    /// ADV-FOG-008): explored, or on the fringe under `ADV-QUIRK-1`.
    fn sees_heroes(&self, pos: Pos, objects_on_fringe: bool) -> bool {
        self.explored(pos) || (objects_on_fringe && self.fringe(pos))
    }

    /// Whether the sight shows an object covering `pos`: explored, or on the fringe under
    /// `ADV-QUIRK-1` or for a wandering monster (E08 decision 40).
    fn sees(&self, pos: Pos, objects_on_fringe: bool, monster: bool) -> bool {
        self.explored(pos) || ((objects_on_fringe || monster) && self.fringe(pos))
    }
}

/// The smallest box of tiles holding the changed tiles of each level, by level.
struct Boxes(Vec<Option<(Pos, Pos)>>);

impl Boxes {
    /// Adds `pos` to its level's box.
    fn add(&mut self, pos: Pos) {
        if let Some(entry) = self.0.get_mut(usize::from(pos.level)) {
            let (low, high) = entry.get_or_insert((pos, pos));
            (low.x, low.y) = (low.x.min(pos.x), low.y.min(pos.y));
            (high.x, high.y) = (high.x.max(pos.x), high.y.max(pos.y));
        }
    }

    /// Whether an object anchored at `anchor` may cover a tile of its level's box: its masks
    /// reach [`MASK_COLUMNS`] − 1 columns left of and [`MASK_ROWS`] − 1 rows above the anchor.
    fn reaches(&self, anchor: Pos) -> bool {
        self.0
            .get(usize::from(anchor.level))
            .copied()
            .flatten()
            .is_some_and(|(low, high)| {
                (low.x..=high.x.saturating_add(MASK_COLUMNS - 1)).contains(&anchor.x)
                    && (low.y..=high.y.saturating_add(MASK_ROWS - 1)).contains(&anchor.y)
            })
    }
}

/// The columns of an object's masks (`TileMask`: eight a row).
const MASK_COLUMNS: u16 = 8;
/// The rows of an object's masks (six).
const MASK_ROWS: u16 = 6;

impl AdventureState {
    /// What `viewer` sees now that it did not see through `since`, an earlier explored grid of
    /// its own (SRV-Q26 (a); the module's documentation): the tiles whose sight changed, with
    /// their views now, then the heroes, the typed objects and the empty boats it sees now and
    /// did not, and the towns a tile of which it explored since, each as [`Self::observe`] shows
    /// it. A grid of other dimensions than the map's counts as nothing explored, so is a viewer
    /// that is not a player's grid now. `since` is the server's own (it never comes from a
    /// client): a tile explored in it and not now (the fog never returns today) is told as it is
    /// seen now, and what the viewer saw only through `since` stays the events' to remove.
    // rule: ADV-FOG-004, ADV-FOG-005, ADV-FOG-006, ADV-FOG-007, ADV-FOG-008, ADV-OWN-003
    // rule: TWN-VEW-001
    pub fn reveal(
        &self,
        core: &Ruleset,
        part: &AdventureRuleset,
        viewer: PlayerId,
        since: &BitGrid,
    ) -> AdventureReveal {
        let map = self.map();
        let (width, height, levels) = (map.width(), map.height(), map.levels());
        let none = || Sight::none(width, height, levels);
        let planes = |sight| Planes {
            sight,
            height: usize::from(height),
        };
        let now = planes(self.explored(viewer).map_or_else(none, Sight::of));
        let fits = (since.width(), since.height(), since.levels()) == (width, height, levels);
        let before = planes(if fits { Sight::of(since) } else { none() });
        let objects_on_fringe = core.quirks().is_enabled(FRINGE_SHOWS_OBJECTS);
        let added = self.added_tiles();
        let stride = now.sight.stride;
        let words = now.sight.explored.iter().zip(&now.sight.fringe);
        let was = before.sight.explored.iter().zip(&before.sight.fringe);
        let mut tiles = Vec::new();
        let mut changed_box = Boxes(vec![None; usize::from(levels)]);
        for (word, ((&explored, &fringe), (&had, &edge))) in words.zip(was).enumerate() {
            let mut changed = (explored ^ had) | (fringe ^ edge);
            let row = word / stride;
            let at = |n: usize| u16::try_from(n).unwrap_or(u16::MAX);
            let (level, y) = (
                u8::try_from(row / usize::from(height)),
                at(row % usize::from(height)),
            );
            while changed != 0 {
                let bit = changed.trailing_zeros();
                changed &= changed - 1;
                let x = at((word % stride) * 64 + usize::try_from(bit).unwrap_or(usize::MAX));
                let pos = Pos::new(x, y, level.unwrap_or(u8::MAX));
                // Bits past the width are clear in both sights: every changed bit is a tile.
                let Some(tile) = map.tile(pos) else { continue };
                let (explored, fringe) = (explored >> bit & 1 == 1, fringe >> bit & 1 == 1);
                changed_box.add(pos);
                tiles.push(RevealedTile {
                    pos,
                    explored,
                    view: tile_view(explored, fringe, objects_on_fringe, tile, || {
                        self.live_tile(added, *tile, || pos)
                    }),
                });
            }
        }
        let new_to_heroes = |pos| {
            now.sees_heroes(pos, objects_on_fringe) && !before.sees_heroes(pos, objects_on_fringe)
        };
        let heroes = self
            .heroes()
            .iter()
            .filter(|hero| hero.owner != viewer && new_to_heroes(hero.pos))
            .map(|hero| self.hero_view(hero, viewer))
            .collect();
        // In tile order (one empty boat a tile: the tile is a unique key).
        let boats: BTreeMap<_, _> = self
            .boats()
            .filter(|(_, boat)| !boat.aboard && !boat.gone && new_to_heroes(boat.tile))
            .map(|(_, boat)| {
                let tile = boat.tile;
                ((tile.level, tile.y, tile.x), boat_view(boat))
            })
            .collect();
        // An object the viewer sees now and did not covers a tile whose sight changed: one whose
        // masks cannot reach the changed tiles is not read further.
        let objects = self
            .observable()
            .filter(|&(_, object, _)| {
                if !changed_box.reaches(object.anchor) {
                    return false;
                }
                let monster = object.monster.is_some();
                let mut seen = false;
                for pos in object.tiles(map) {
                    if before.sees(pos, objects_on_fringe, monster) {
                        return false;
                    }
                    seen = seen || now.sees(pos, objects_on_fringe, monster);
                }
                seen
            })
            .map(|seen| self.object_view(part, viewer, seen))
            .collect();
        let towns = (0..map.towns().len())
            .filter_map(|t| u16::try_from(t).ok().map(TownId::new))
            .filter(|&town| {
                map.town_object(town).is_some_and(|index| {
                    let object = &map.objects()[index];
                    changed_box.reaches(object.anchor)
                        && object.tiles(map).any(|pos| now.explored(pos))
                        && !object.tiles(map).any(|pos| before.explored(pos))
                })
            })
            .collect();
        AdventureReveal {
            viewer,
            tiles,
            heroes,
            objects,
            boats: boats.into_values().collect(),
            towns,
        }
    }
}

#[cfg(test)]
mod tests;
