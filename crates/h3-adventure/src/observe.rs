//! What a player may know of the adventure map (ADR-0016 points 5 and 6, E08 T8b1;
//! `docs/06-security.md`, "Information hiding"): the observation, materialised per viewer as flat
//! planes, and the projection of events.
//!
//! A viewer sees on its explored tiles their terrain, roads and rivers, what objects still on the
//! map cover there and the heroes standing there (ADV-FOG-004); on the fringe their terrain, roads
//! and rivers, and under `ADV-QUIRK-1` (the `faithful` profile) also what objects cover there and
//! the heroes standing there (ADV-FOG-005), in either profile the wandering monsters there (E08
//! decision 40: a monster guards explored tiles, which the viewer's moves would reveal); nothing
//! of any other tile. A typed object still on the
//! map shows whole, its type and tiles, once the viewer sees any tile it covers so (ADR-0016 point
//! 6, E08 T12b); its contents never show, nor another player's purchases or keys (its own
//! Cartographer types bought show, T14b, and its own keys, T15a). Of a hero it sees its tile,
//! owner and
//! identity; its army and movement points only if the hero is its own (ADV-FOG-006). Other players'
//! explored tiles, the seed and the streams never show: an event's projection names the viewer
//! alone among the players who see it, and a step tells only the tiles the viewer sees the hero
//! on.

use alloc::vec::Vec;

use h3_content::{CreatureTypeId, FRINGE_SHOWS_OBJECTS, Formation, Ruleset, TerrainId};
use h3_contracts::{ArmyStack, HeroId, PlayerId};

use crate::command::{AdventureEvent, AdventureEventKind, Viewers};
use crate::fog::Sight;
use crate::ruleset::{AdventureRuleset, ObjectTypeId, RiverId, RoadId};
use crate::setup::{Pos, TileMask};
use crate::state::{AdventureState, StaticTile};

/// What one player may know of the adventure map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdventureObservation {
    /// The player this observation is for.
    pub viewer: PlayerId,
    /// Columns (the map's size is public).
    pub width: u16,
    /// Rows.
    pub height: u16,
    /// Levels.
    pub levels: u8,
    /// Words a row of [`Self::explored`] and [`Self::fringe`]: the width divided by 64, rounded
    /// up.
    pub stride: usize,
    /// The tiles the viewer has explored, level by level, row by row, [`Self::stride`] words a
    /// row, the tile of column `x` as bit `x % 64` of word `x / 64` (bits past the width clear).
    pub explored: Vec<u64>,
    /// The fringe, in the same layout: the tiles not explored with an explored one among the
    /// eight around them on their level (ADV-FOG-005).
    pub fringe: Vec<u64>,
    /// What the viewer sees of each tile, level by level, row by row, column by column.
    pub tiles: Vec<TileView>,
    /// The heroes the viewer sees, by id: its own and those on a tile it sees them on.
    pub heroes: Vec<HeroView>,
    /// The typed objects still on the map that the viewer sees, in the map's order (E08 T12b).
    pub objects: Vec<ObjectView>,
    /// The Cartographer types the viewer has bought, in type order (ADV-VIS-001, E08 T14b);
    /// other players' purchases never show (ADR-0016 point 6).
    pub bought: Vec<ObjectTypeId>,
    /// The keys the viewer holds, bit `c` for the colour `c` (ADV-VIS-003, E08 T15a); other
    /// players' keys never show (ADR-0016 point 6).
    pub keys: u8,
}

impl AdventureObservation {
    /// The word of the planes holding `pos` and the bit in it, if `pos` is on the map.
    fn locate(&self, pos: Pos) -> Option<(usize, u32)> {
        (pos.x < self.width && pos.y < self.height && pos.level < self.levels).then(|| {
            let row = usize::from(pos.level) * usize::from(self.height) + usize::from(pos.y);
            (
                row * self.stride + usize::from(pos.x / 64),
                u32::from(pos.x % 64),
            )
        })
    }

    /// Whether the viewer has explored the tile at `pos` (a tile off the map is not).
    pub fn is_explored(&self, pos: Pos) -> bool {
        self.locate(pos)
            .is_some_and(|(word, bit)| self.explored[word] >> bit & 1 == 1)
    }

    /// Whether the tile at `pos` is on the viewer's fringe (a tile off the map is not).
    pub fn is_fringe(&self, pos: Pos) -> bool {
        self.locate(pos)
            .is_some_and(|(word, bit)| self.fringe[word] >> bit & 1 == 1)
    }

    /// What the viewer sees of the tile at `pos`, if it is on the map.
    pub fn tile(&self, pos: Pos) -> Option<&TileView> {
        let index =
            (pos.x < self.width && pos.y < self.height && pos.level < self.levels).then(|| {
                (usize::from(pos.level) * usize::from(self.height) + usize::from(pos.y))
                    * usize::from(self.width)
                    + usize::from(pos.x)
            })?;
        self.tiles.get(index)
    }

    /// The hero with this id, if the viewer sees it.
    pub fn hero(&self, id: HeroId) -> Option<&HeroView> {
        self.heroes
            .binary_search_by_key(&id, |h| h.id)
            .ok()
            .map(|index| &self.heroes[index])
    }
}

/// A typed object as a viewer sees it (ADR-0016 point 6): what it is and where, not what it holds.
/// Not its index in the static map either: that would count the objects before it, seen or not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectView {
    /// Its object type (a random resource's drawn pile type).
    pub object_type: ObjectTypeId,
    /// The bottom right tile its masks are relative to.
    pub anchor: Pos,
    /// The tiles no hero enters.
    pub blocked: TileMask,
    /// The tiles a hero visits it from.
    pub visitable: TileMask,
    /// The tiles its picture is drawn on beyond those: where a special ground lies (ADV-GRD-001,
    /// E08 T20a2a).
    pub drawn: TileMask,
    /// What the viewer sees of a wandering monster (ADV-FOG-006, E08 T20a2a): none for every
    /// other object.
    pub monster: Option<MonsterView>,
}

/// What a viewer sees of a wandering monster (ADV-FOG-006, E08 T20a2a; the original's hover text,
/// 0x44ADE0): its creature and the word of its count, never the count, its aggression, its flags
/// or its treasure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonsterView {
    /// The creature.
    pub creature: CreatureTypeId,
    /// The word of its count ([`crate::MonsterParams::count_word`]): 0 the first (the original's
    /// "few"), each next word from the next threshold of the monsters' parameters.
    pub count_word: u8,
}

/// What a viewer sees of a tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileView {
    /// Nothing: a tile neither explored nor on the fringe.
    Hidden,
    /// Its terrain, road and river, not what objects cover there: a tile of the fringe without
    /// `ADV-QUIRK-1` (ADV-FOG-005).
    Ground {
        /// The terrain.
        terrain: TerrainId,
        /// The road, if any.
        road: Option<RoadId>,
        /// The river, if any.
        river: Option<RiverId>,
    },
    /// The tile's terrain, road and river and what objects still on the map cover there: an
    /// explored tile (ADV-FOG-004), or a tile of the fringe under `ADV-QUIRK-1`.
    Seen(StaticTile),
}

/// A hero as a viewer sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroView {
    /// The hero.
    pub id: HeroId,
    /// The player owning it.
    pub owner: PlayerId,
    /// The tile it stands on.
    pub pos: Pos,
    /// Its army and movement points, for its owner only (ADV-FOG-006).
    pub own: Option<OwnHeroView>,
}

/// What a player knows of its own hero beyond what every viewer sees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnHeroView {
    /// Its army, by slot.
    pub army: Vec<ArmyStack>,
    /// The movement points it has left today.
    pub movement: u32,
    /// Its army's formation in battle (E08 T18b).
    pub formation: Formation,
}

impl AdventureState {
    /// What `viewer` may know of the adventure map (ADV-FOG-004 to ADV-FOG-006): its sight's
    /// planes, built word by word from its explored grid, then each tile's view, the heroes and
    /// the typed objects it sees (a monster's creature and the word of its count, E08 T20a2a),
    /// the Cartographer types it bought (E08 T14b) and the keys it holds (T15a). A viewer that is
    /// not a player of the game sees no tile, no hero, no object, no purchase and no key.
    // rule: ADV-FOG-004, ADV-FOG-005, ADV-FOG-006
    pub fn observe(
        &self,
        core: &Ruleset,
        part: &AdventureRuleset,
        viewer: PlayerId,
    ) -> AdventureObservation {
        let map = self.map();
        let (width, height, levels) = (map.width(), map.height(), map.levels());
        let sight = self
            .explored(viewer)
            .map_or_else(|| Sight::none(width, height, levels), Sight::of);
        let objects_on_fringe = core.quirks().is_enabled(FRINGE_SHOWS_OBJECTS);
        let mut observation = AdventureObservation {
            viewer,
            width,
            height,
            levels,
            stride: sight.stride,
            explored: sight.explored,
            fringe: sight.fringe,
            tiles: Vec::with_capacity(map.tiles().len()),
            heroes: Vec::new(),
            objects: Vec::new(),
            bought: self.bought(viewer).collect(),
            keys: self.keys(viewer),
        };
        let row_tiles = usize::from(width);
        // A tile as it stands now; only a tile an object covered can differ from the static map's.
        let live = |row: usize, x: usize, tile: &StaticTile| {
            if tile.blocked || tile.visitable {
                let at = |n: usize| u16::try_from(n).unwrap_or(u16::MAX);
                let height = usize::from(height);
                let level = u8::try_from(row / height).unwrap_or(u8::MAX);
                self.live(Pos::new(at(x), at(row % height), level), *tile)
            } else {
                *tile
            }
        };
        for (row, tiles) in map.tiles().chunks_exact(row_tiles).enumerate() {
            let start = row * observation.stride;
            for (x, tile) in tiles.iter().enumerate() {
                let (word, bit) = (start + x / 64, x % 64);
                let view = if observation.explored[word] >> bit & 1 == 1 {
                    TileView::Seen(live(row, x, tile))
                } else if observation.fringe[word] >> bit & 1 == 0 {
                    TileView::Hidden
                } else if objects_on_fringe {
                    TileView::Seen(live(row, x, tile))
                } else {
                    TileView::Ground {
                        terrain: tile.terrain,
                        road: tile.road,
                        river: tile.river,
                    }
                };
                observation.tiles.push(view);
            }
        }
        observation.heroes = self
            .heroes()
            .iter()
            .filter(|hero| {
                hero.owner == viewer
                    || observation.is_explored(hero.pos)
                    || (objects_on_fringe && observation.is_fringe(hero.pos))
            })
            .map(|hero| HeroView {
                id: hero.id,
                owner: hero.owner,
                pos: hero.pos,
                own: (hero.owner == viewer).then(|| OwnHeroView {
                    army: hero.army.clone(),
                    movement: hero.movement,
                    formation: hero.formation,
                }),
            })
            .collect();
        // A monster shows on the fringe in either profile (E08 decision 40): it guards the
        // explored tiles beside it, which its guard's effect on the viewer's moves would reveal.
        let sees = |pos, monster: bool| {
            observation.is_explored(pos)
                || ((objects_on_fringe || monster) && observation.is_fringe(pos))
        };
        let words = part.params().monsters();
        let objects =
            map.objects()
                .iter()
                .enumerate()
                .filter_map(|(index, object)| {
                    let object_type = object.object_type?;
                    let monster = object.monster.is_some();
                    (!self.is_removed(index) && object.tiles(map).any(|pos| sees(pos, monster)))
                        .then(|| ObjectView {
                            object_type,
                            anchor: object.anchor,
                            blocked: object.blocked,
                            visitable: object.visitable,
                            drawn: object.drawn,
                            monster: object.monster.as_ref().map(|monster| MonsterView {
                                creature: monster.creature,
                                count_word: words.map_or(0, |w| {
                                    w.count_word(self.monster_count(index).unwrap_or(monster.count))
                                }),
                            }),
                        })
                })
                .collect();
        observation.objects = objects;
        observation
    }
}

impl AdventureEvent {
    /// The event as `viewer` may see it, or `None` if it is not among the players who see it
    /// (ADR-0016 point 6): a step's tile where the viewer does not see the hero is left out. The
    /// projection's viewers are the viewer alone: who else sees an event tells which tiles other
    /// players have explored.
    pub fn project(&self, viewer: PlayerId) -> Option<Self> {
        let alone = Viewers::NONE.with(viewer);
        self.seen_by.contains(viewer).then(|| Self {
            kind: match self.kind {
                AdventureEventKind::HeroStepped {
                    hero,
                    from,
                    to,
                    seen_from,
                    seen_to,
                } => AdventureEventKind::HeroStepped {
                    hero,
                    from: from.filter(|_| seen_from.contains(viewer)),
                    to: to.filter(|_| seen_to.contains(viewer)),
                    seen_from: seen_from.intersection(alone),
                    seen_to: seen_to.intersection(alone),
                },
                AdventureEventKind::ObjectRemoved { .. }
                | AdventureEventKind::TilesRevealed { .. }
                | AdventureEventKind::KeyGained { .. }
                | AdventureEventKind::HeroRemoved { .. }
                | AdventureEventKind::CreaturesJoined { .. }
                | AdventureEventKind::MonsterChanged { .. } => self.kind,
            },
            seen_by: alone,
        })
    }
}

#[cfg(test)]
mod tests;
