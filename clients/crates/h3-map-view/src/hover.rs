//! What the mouse finds over the adventure screen (E24 M3 T6a1, *provisional*;
//! `docs/epics/E24-design.md` §4.10): the cursor over the map by the tile under the mouse and the
//! status bar's hint, over the map's tiles and over the panels' items.
//!
//! The original (`adventure_mouse_move` 0x40E360, on a human player's turn shown here): over the
//! view's item (8, 8, 592 × 544) the tile under the mouse is the view's corner plus the mouse's
//! screen point divided by a tile's side (`Rules::tile_at`: so the last 8 pixel rows of a drawn
//! row give the tile below, design §1.2), looked at only when the view's column or row under the
//! mouse changes (the manager's +0xEC, +0xF0; -1 after a walk's end and a key's step); then the
//! tile's hint (0x40B150) and the cursor by what the tile holds and whether and in how many days
//! the selected hero reaches it (the pathfinder's, run for it at every change: never the
//! client's, given). Elsewhere the panel's hint (0x403010): the item under the mouse, looked at
//! only when it differs from the last one looked at (a static, 0x65F228, which the map's hints
//! leave: MCL-Q17).
//!
//! What a tile holds is the map's ([`crate::record`]: the object a tile records, with the heroes
//! and boats standing on it); what only the game knows (a monster's count, a town's name, which
//! towns and garrisons hold an army, which tiles a monster guards, the selected hero's reach and
//! day points, the players' teams) is the live part's [`Facts`]: a comparison's from the
//! original's memory, the engine's from M4. The hint's cases of the 58 classes that have one of
//! their own (visits, owners, contents) are T6a2's: until then such a class shows its plain name
//! (MCL-STUB-7).

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_map::model::Pos;
use h3_ui::text::{Args, Catalogue};

use crate::art::MapArt;
use crate::layout::AdventureLayout;
use crate::lists::SLOTS;
use crate::live::{Buttons, Explored, Kingdom, Live};
use crate::panels::{self, id};
use crate::objects::{self, DrawLists, Entry};
use crate::record::Records;
use crate::status::{hero_key, town_key};

/// The terrain code of water (8).
pub const WATER: u8 = 8;

/// The cursor's frames of `cradvntr.def` over the map (`adventure_mouse_move` 0x40E360, the
/// mouse manager's type 1).
pub mod frame {
    /// The arrow.
    pub const ARROW: u32 = 0;
    /// The selected hero, or one of the player's heroes without one selected.
    pub const HERO: u32 = 2;
    /// A town of the player's team.
    pub const TOWN: u32 = 3;
    /// A move: the first of the six kinds of a day's group (move, attack, board, disembark,
    /// exchange, visit); a further day's group six frames on, the fourth for four days or more.
    pub const MOVE: u32 = 4;
    /// An attack.
    pub const ATTACK: u32 = 5;
    /// A boat boarded, or a shipyard of the player's team.
    pub const BOARD: u32 = 6;
    /// Land a hero in a boat lands on.
    pub const DISEMBARK: u32 = 7;
    /// A hero of the player's team.
    pub const EXCHANGE: u32 = 8;
    /// An object visited.
    pub const VISIT: u32 = 9;
    /// A day's group's frames.
    pub const DAY: u32 = 6;
    /// An object visited from a boat (28 to 31 by the days, not by six).
    pub const SAIL_VISIT: u32 = 28;
}

/// What the click there would do (the manager's +0x40, which a click's handler reads: T6b's):
/// -1 nothing, 1 a move, 2 the selected hero, 3 a town selected, 5 a town entered, 8 a shipyard.
pub type Action = i32;

/// The game's facts the mouse-over reads that the map cannot give (the live part's `[hover]`;
/// design §4.10): a comparison reads them from the original's memory (`h3mem.py`), the engine
/// gives them from M4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    /// Per map object (the map's order), the word of its entrance tile as the game holds it with
    /// a hero standing there taken off (`hero_hide` 0x4D76E0): a monster's count in its low 12
    /// bits, a town's index among the game's towns; none for an object without an entrance on
    /// the map.
    pub words: Vec<Option<u32>>,
    /// Per map object, whether it holds an army that defends it: a town (`town_defended`
    /// 0x5BDFE0: a garrisoned or visiting hero's army, else its own) or a garrison (its army).
    pub armed: Vec<bool>,
    /// The game's towns by index: each one's name (a record of `TownName.txt`, the catalogue's
    /// `town-name` family) and type.
    pub towns: Vec<(u16, u8)>,
    /// Per hero of the live part's (`Live::heroes`, in its order), its record's index (its type,
    /// whose name is the `hero` family's) and whether its name is the map's own.
    pub heroes: Vec<(u8, bool)>,
    /// The viewing player's heroes in its list's order (the player record's +0x8): each one's
    /// record index, class and own name.
    pub hero_list: Vec<(u8, u8, bool)>,
    /// The viewing player's towns in its list's order (the player record's +0x40): each one's
    /// index among the game's towns.
    pub town_list: Vec<usize>,
    /// Each colour's team (the game's +0x1F879).
    pub teams: [u8; 8],
    /// The tiles a monster guards (the vision word's bit 8, `tile_vision_word` 0x4F79B0).
    pub guarded: Explored,
    /// The selected hero's reach on its level (the pathfinder's nodes, 0x699284 +0x24, of the
    /// tiles it reaches: their +0x18, the movement points the path there costs), by tile.
    pub reach: BTreeMap<(i32, i32), u32>,
    /// The selected hero's movement points a day (its record's +0x49): the days a path takes.
    pub day_points: i32,
}

impl Facts {
    /// The facts of the live part's `[hover]` table for a map of `size` and `levels` with
    /// `objects` objects and `heroes` heroes: `words` (one per object, -1 none), `armed` (one
    /// per object, 0 or 1), `towns` (`[[name, type], ...]` by the game's town index),
    /// `hero_ids` (`[[index, own name], ...]`, one per live hero), `hero_list` (`[[index,
    /// class, own name], ...]`), `town_list` (town indices), `teams` (eight), `guarded` (a
    /// string per row of every level in turn, `1` guarded) and `reach` (`[[x, y, cost], ...]`
    /// of the selected hero's level) with `day_points`. None when a part is missing or wrong.
    #[must_use]
    pub fn read(
        value: &toml::Value,
        objects: usize,
        heroes: usize,
        size: usize,
        levels: usize,
    ) -> Option<Self> {
        let table = value.as_table()?;
        let array = |key: &str| table.get(key).and_then(toml::Value::as_array);
        let int = |v: &toml::Value| v.as_integer();
        let ints = |v: &toml::Value| -> Option<Vec<i64>> {
            v.as_array()?.iter().map(toml::Value::as_integer).collect()
        };
        let flag = |n: i64| match n {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        };
        let byte = |n: i64| u8::try_from(n).ok();
        let words: Vec<Option<u32>> = array("words")?
            .iter()
            .map(|v| match int(v)? {
                -1 => Some(None),
                n => u32::try_from(n).ok().map(Some),
            })
            .collect::<Option<_>>()?;
        let armed: Vec<bool> = array("armed")?
            .iter()
            .map(|v| int(v).and_then(flag))
            .collect::<Option<_>>()?;
        let towns = array("towns")?
            .iter()
            .map(|v| match ints(v)?.as_slice() {
                &[name, kind] => Some((u16::try_from(name).ok()?, byte(kind)?)),
                _ => None,
            })
            .collect::<Option<_>>()?;
        let hero_ids: Vec<(u8, bool)> = array("hero_ids")?
            .iter()
            .map(|v| match ints(v)?.as_slice() {
                &[id, own] => Some((byte(id)?, flag(own)?)),
                _ => None,
            })
            .collect::<Option<_>>()?;
        let hero_list = array("hero_list")?
            .iter()
            .map(|v| match ints(v)?.as_slice() {
                &[id, class, own] => Some((byte(id)?, byte(class)?, flag(own)?)),
                _ => None,
            })
            .collect::<Option<_>>()?;
        let town_list = array("town_list")?
            .iter()
            .map(|v| int(v).and_then(|n| usize::try_from(n).ok()))
            .collect::<Option<_>>()?;
        let teams: Vec<u8> = array("teams")?
            .iter()
            .map(|v| int(v).and_then(byte))
            .collect::<Option<_>>()?;
        let rows: Vec<&str> = array("guarded")?
            .iter()
            .map(toml::Value::as_str)
            .collect::<Option<_>>()?;
        let mut guarded = Explored::all(size, levels, false);
        if rows.len() != size.saturating_mul(levels) {
            return None;
        }
        for (r, row) in rows.iter().enumerate() {
            if row.len() != size {
                return None;
            }
            let (z, y) = (r / size.max(1), r % size.max(1));
            for (x, c) in row.bytes().enumerate() {
                let on = match c {
                    b'0' => false,
                    b'1' => true,
                    _ => return None,
                };
                let to_i32 = |v: usize| i32::try_from(v).ok();
                guarded.set(to_i32(x)?, to_i32(y)?, u8::try_from(z).ok()?, on);
            }
        }
        let reach = array("reach")?
            .iter()
            .map(|v| match ints(v)?.as_slice() {
                &[x, y, cost] => Some((
                    (i32::try_from(x).ok()?, i32::try_from(y).ok()?),
                    u32::try_from(cost).ok()?,
                )),
                _ => None,
            })
            .collect::<Option<_>>()?;
        let day_points = int(table.get("day_points")?).and_then(|n| i32::try_from(n).ok())?;
        let fits = words.len() == objects && armed.len() == objects && hero_ids.len() == heroes;
        fits.then_some(Self {
            words,
            armed,
            towns,
            heroes: hero_ids,
            hero_list,
            town_list,
            teams: teams.try_into().ok()?,
            guarded,
            reach,
            day_points,
        })
    }
}

/// What a tile holds as the mouse-over reads it, with no hero or boat standing on it (those are
/// [`Hover::current`]'s).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Tile {
    /// The class it records (as the game loads it: [`crate::record::Record::class`]).
    pub class: u32,
    /// Its subclass.
    pub subclass: u32,
    /// The object it records (the map's order).
    pub object: Option<usize>,
    /// The object's entrance (+0xD bit 0x10).
    pub entrance: bool,
    /// An object blocks it (+0xD bit 0x01).
    pub blocked: bool,
    /// Water (its terrain 8).
    pub water: bool,
    /// The special ground under it (`tile_special_ground` 0x4FCE20: the class of the topmost of
    /// its objects among 21, 46, 222, 224 to 229 and 231), 0 none.
    pub ground: u32,
    /// The word: the object's entrance tile's ([`Facts::words`]), a hero's its record index.
    pub word: Option<u32>,
    /// The owner of its object as the cursor finds it (a town's, a shipyard's, a garrison's
    /// by its entrance tile: the view's owners, `Live::objects`), a hero's its own.
    pub owner: Option<u8>,
    /// A hero standing on it (its record index).
    pub hero: Option<u8>,
}

/// The special grounds a tile's hint names (`tile_special_ground` 0x4FCE20: not 223 and 230,
/// which the game loads as 21 and 46).
pub const GROUNDS: [u32; 10] = [21, 46, 222, 224, 225, 226, 227, 228, 229, 231];

/// The classes whose hint is a case of the original's own (0x40B150's switch, table 0x40D314):
/// every other class below 232 shows its name (`object_names` 0x6A79EC). T6a1 builds those of
/// 0, 3, 26 and 36 (the special ground), 34 (a hero), 54 (a monster) and 98 (a town); the rest
/// show their name too until T6a2 (MCL-STUB-7).
pub const OWN_CASES: [u32; 62] = [
    0, 3, 4, 9, 10, 11, 14, 16, 17, 20, 22, 23, 24, 25, 26, 28, 30, 31, 32, 34, 35, 36, 38, 39,
    41, 42, 47, 48, 49, 51, 52, 53, 54, 55, 56, 57, 61, 63, 64, 79, 83, 84, 85, 88, 89, 90, 92,
    94, 96, 98, 100, 102, 104, 105, 107, 108, 109, 110, 112, 113, 212, 215,
];

/// The object classes the original names (`OBJNAMES.TXT`'s records).
pub const CLASSES: u32 = 232;

/// The status bar's hint: what its text shows, its words the catalogue's.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Hint {
    /// Nothing.
    #[default]
    Empty,
    /// A panel item's (`map-hint.<n>`: Help.txt's record 291 + n).
    Panel(u8),
    /// A hero's name and its class's (`map.hint.hero`).
    Hero {
        /// Its record's index (its type).
        id: u8,
        /// Its class.
        class: u8,
        /// Its name is the map's own.
        own_name: bool,
    },
    /// A town's name and its type's (`map.hint.town`).
    Town {
        /// Its name (a record of `TownName.txt`).
        name: u16,
        /// Its type.
        kind: u8,
    },
    /// An object class's name (`object.<class>`).
    Object(u32),
    /// A monster's size word and creature (`map.hint.monster`).
    Monster {
        /// Its creature (above 150 none).
        creature: u32,
        /// Its count.
        count: u32,
    },
}

/// The catalogue's keys the hints show texts by (the families `map-hint`, `object`, `hero`,
/// `hero-class`, `town-name`, `town-type` and `creature` besides).
pub const KEYS: [&str; 12] = [
    "map.hint.hero",
    "map.hint.town",
    "map.hint.monster",
    "map.army.0",
    "map.army.1",
    "map.army.2",
    "map.army.3",
    "map.army.4",
    "map.army.5",
    "map.army.6",
    "map.army.7",
    "map.army.8",
];

/// The counts below which each size word but the last stands (`creature_quantity_word`
/// 0x44ADE0).
pub const SIZES: [u32; 8] = [5, 10, 20, 50, 100, 250, 500, 1000];

/// The last creature whose name a monster's hint shows (0x40C350).
pub const LAST_CREATURE: u32 = 150;

impl Hint {
    /// The hint's text by `texts`, a hero's own names `names` (the map's, by type).
    #[must_use]
    pub fn text(self, names: &BTreeMap<u8, String>, texts: &Catalogue) -> String {
        let plain = |key: &str| texts.text(key, &Args::new());
        match self {
            Self::Empty => String::new(),
            Self::Panel(n) => plain(&alloc::format!("map-hint.{n}")),
            Self::Hero {
                id,
                class,
                own_name,
            } => {
                let name = match names.get(&id) {
                    Some(own) if own_name => own.clone(),
                    _ => plain(&hero_key(id)),
                };
                let class = plain(&alloc::format!("hero-class.{class}"));
                let args = Args::new().text("name", &name).text("class", &class);
                texts.text("map.hint.hero", &args)
            }
            Self::Town { name, kind } => {
                let args = Args::new()
                    .text("name", &plain(&town_key(name)))
                    .text("type", &plain(&alloc::format!("town-type.{kind}")));
                texts.text("map.hint.town", &args)
            }
            Self::Object(class) if class < CLASSES => plain(&alloc::format!("object.{class}")),
            Self::Object(_) => String::new(),
            Self::Monster { creature, count } => {
                let size = SIZES.iter().take_while(|&&s| count >= s).count();
                let creature = if creature <= LAST_CREATURE {
                    let args = Args::new().count(2);
                    texts.text(&alloc::format!("creature.{creature}"), &args)
                } else {
                    String::new()
                };
                let args = Args::new()
                    .text("size", &plain(&alloc::format!("map.army.{size}")))
                    .text("creature", &creature);
                texts.text("map.hint.monster", &args)
            }
        }
    }
}

/// A hero on the map as the mouse-over reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapHero {
    /// Its record's index.
    pub id: u8,
    /// Its tile.
    pub at: Pos,
    /// Its owner.
    pub owner: u8,
    /// Its class.
    pub class: u8,
    /// Its name is the map's own.
    pub own_name: bool,
    /// It sails a boat.
    pub boat: bool,
}

/// The selected hero as the mouse-over reads it (the timeline's).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    /// Its record's index.
    pub id: u8,
    /// Its tile.
    pub at: Pos,
    /// Lifted off its tile (the manager's +0x20C): the tile shows what lies under it, but at the
    /// view's middle, where the hint puts it back (0x40AFB0).
    pub lifted: bool,
    /// Its movement points.
    pub points: i32,
}

/// What a tile's cursor shows and a click there would do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    /// The cursor's frame ([`frame`]).
    pub frame: u32,
    /// The action ([`Action`]): -1 at every change of the tile (0x40E41A) unless the tile sets
    /// one.
    pub action: Action,
}

/// The classes the game never lets a hero enter (`object_classes_not_entered` 0x63AA58, the
/// object settings' first byte, 0x41B500: a visit from the neighbouring tile): a guarded tile's
/// such object does not show the attack (0x40E2DA).
pub const NOT_ENTERED: [u32; 42] = [
    3, 5, 6, 8, 9, 11, 12, 22, 26, 29, 34, 36, 52, 54, 59, 62, 65, 66, 67, 68, 69, 70, 71, 72, 73,
    74, 75, 162, 163, 164, 76, 79, 81, 82, 85, 86, 92, 93, 95, 101, 214, 215,
];

/// The mouse-over of a map: its tiles as the map's objects make them and the game's facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hover {
    size: usize,
    levels: usize,
    tiles: Vec<Tile>,
    entrances: BTreeMap<usize, (i32, i32)>,
    facts: Facts,
    heroes: Vec<MapHero>,
    boats: Vec<Pos>,
    explored: Explored,
    viewer: u8,
}

/// The objects classes the cursor finds the owner of by the entrance tile: a garrison (33), a
/// shipyard (87), a town (98).
const OWNED_BY_ENTRANCE: [u32; 3] = [33, 87, 98];

impl Hover {
    /// The mouse-over of `art` with `live`'s facts (none without them), the tiles as `records`
    /// takes them from each level's draw lists `lists` (level by level).
    #[must_use]
    pub fn new(
        art: &MapArt,
        live: &Live,
        records: &Records,
        lists: &[DrawLists],
    ) -> Option<Arc<Self>> {
        let facts = live.hover.clone()?;
        let size = art.size;
        let mut tiles = Vec::with_capacity(size.saturating_mul(size).saturating_mul(art.levels));
        for (z, level) in (0u8..).zip(lists.iter().take(art.levels)) {
            for y in 0..size {
                for x in 0..size {
                    let (x, y) = (to_i32(x), to_i32(y));
                    let Some(tile) = art.tile(x, y, z) else {
                        tiles.push(Tile::default());
                        continue;
                    };
                    let entries = level.at(x, y);
                    let record = records.record(art, live, tile, entries);
                    let ground = ground(records, art, live, entries);
                    let word = record
                        .object
                        .and_then(|o| facts.words.get(o).copied().flatten());
                    let owner = record
                        .object
                        .filter(|_| OWNED_BY_ENTRANCE.contains(&record.class))
                        .and_then(|o| live.objects.get(o))
                        .and_then(|o| o.owner);
                    tiles.push(Tile {
                        class: record.class,
                        subclass: record.subclass,
                        object: record.object,
                        entrance: record.entrance,
                        blocked: record.blocked,
                        water: tile.terrain.code == WATER,
                        ground,
                        word,
                        owner,
                        hero: None,
                    });
                }
            }
        }
        let heroes = live
            .heroes
            .iter()
            .zip(&facts.heroes)
            .map(|(h, &(id, own_name))| MapHero {
                id,
                at: h.at,
                owner: h.owner,
                class: h.class,
                own_name,
                boat: h.boat.is_some(),
            })
            .collect();
        Some(Arc::new(Self {
            size,
            levels: art.levels,
            tiles,
            entrances: entrances(art),
            heroes,
            boats: live.boats.iter().map(|b| b.at).collect(),
            explored: live.explored.clone(),
            viewer: live.viewer,
            facts,
        }))
    }

    /// The game's facts.
    #[must_use]
    pub fn facts(&self) -> &Facts {
        &self.facts
    }

    /// The tile at `at` with no hero or boat on it; none off the map.
    #[must_use]
    pub fn base(&self, [x, y, z]: [i32; 3]) -> Option<Tile> {
        let (x, y) = (usize::try_from(x).ok()?, usize::try_from(y).ok()?);
        let z = usize::try_from(z).ok()?;
        (x < self.size && y < self.size && z < self.levels)
            .then(|| self.tiles.get((z * self.size + y) * self.size + x).copied())
            .flatten()
    }

    /// The tile at `at` with the heroes and boats standing there (`hero_show` 0x4D75D0: class 34
    /// or 8 and the entrance bit; a hero's word its index): the selected hero `selected` on its
    /// tile there (none when lifted off it), the others where the live part has them.
    #[must_use]
    pub fn current(&self, at: [i32; 3], selected: Option<Selection>) -> Option<Tile> {
        let base = self.base(at)?;
        let here = |p: Pos| [i32::from(p.x), i32::from(p.y), i32::from(p.z)] == at;
        let standing = |h: &&MapHero| match selected.filter(|s| s.id == h.id) {
            Some(s) => !s.lifted && here(s.at),
            None => here(h.at),
        };
        if let Some(hero) = self.heroes.iter().find(standing) {
            return Some(Tile {
                class: crate::live::HERO_CLASS,
                entrance: true,
                word: Some(u32::from(hero.id)),
                owner: Some(hero.owner),
                hero: Some(hero.id),
                ..base
            });
        }
        if self.boats.iter().any(|&b| here(b)) {
            return Some(Tile {
                class: crate::live::BOAT_CLASS,
                entrance: true,
                ..base
            });
        }
        Some(base)
    }

    /// The tile whose hint the mouse at `at` shows (0x40AFB0), with the selected hero `selected`:
    /// at the view's `middle` a lifted hero is put back on its tile and the tile is the mouse's;
    /// else the entrance tile of its object when another, with a hero or boat standing there
    /// taken off, else its own.
    #[must_use]
    pub fn hint_tile(&self, at: [i32; 3], middle: bool, selected: Option<Selection>) -> Option<Tile> {
        if middle && let Some(s) = selected.filter(|s| s.lifted) {
            let shown = Selection {
                lifted: false,
                ..s
            };
            return self.current(at, Some(shown));
        }
        let tile = self.current(at, selected)?;
        let Some(door) = self.entrance(at, &tile) else {
            return Some(tile);
        };
        if door == at {
            return Some(tile);
        }
        let there = self.current(door, selected)?;
        if there.class == tile.class {
            return Some(there);
        }
        if there.class == crate::live::HERO_CLASS || there.class == crate::live::BOAT_CLASS {
            return self.base(door);
        }
        Some(there)
    }

    /// Whether `at` lies on the map and the viewer has explored it (`position_on_map` 0x4B1330,
    /// the vision word's bit of the player, 0x40E4BA).
    #[must_use]
    pub fn explored(&self, at: [i32; 3]) -> bool {
        let [x, y, z] = at;
        self.base(at).is_some()
            && self
                .explored
                .explored(x, y, u8::try_from(z).unwrap_or(u8::MAX))
    }

    /// The record index of the hero the live part has on `at`.
    #[must_use]
    pub fn hero_at(&self, at: Pos) -> Option<u8> {
        self.heroes.iter().find(|h| h.at == at).map(|h| h.id)
    }

    /// The entrance tile of `tile`'s object (`tile_object_entrance` 0x4FCAA0): itself when it is
    /// an entrance; none for the classes 0, 3, 26 and 36 or without an object; else its object's
    /// first visitable cell on the map (`object_entrance_tile` 0x4FEC30), on its level.
    fn entrance(&self, at: [i32; 3], tile: &Tile) -> Option<[i32; 3]> {
        if tile.entrance {
            return Some(at);
        }
        if [0, 3, 26, 36].contains(&tile.class) {
            return None;
        }
        let (x, y) = self.entrances.get(&tile.object?).copied()?;
        Some([x, y, at[2]])
    }

    /// The hint of the mouse at `at` over the map (0x40B150): the hint tile's by its class
    /// ([`Hover::hint_tile`]).
    #[must_use]
    pub fn hint(&self, at: [i32; 3], middle: bool, selected: Option<Selection>) -> Hint {
        let Some(tile) = self.hint_tile(at, middle, selected) else {
            return Hint::Empty;
        };
        match tile.class {
            0 | 3 | 26 | 36 => match tile.ground {
                0 => Hint::Empty,
                ground => Hint::Object(ground),
            },
            crate::live::HERO_CLASS => tile
                .hero
                .or_else(|| tile.word.and_then(|w| u8::try_from(w).ok()))
                .and_then(|id| self.hero(id))
                .map_or(Hint::Empty, |h| Hint::Hero {
                    id: h.0,
                    class: h.1,
                    own_name: h.2,
                }),
            crate::live::TOWN_CLASS => tile
                .word
                .and_then(|w| self.facts.towns.get(usize::try_from(w).ok()?))
                .map_or(Hint::Empty, |&(name, kind)| Hint::Town { name, kind }),
            MONSTER if tile.entrance => Hint::Monster {
                creature: tile.subclass,
                count: tile.word.unwrap_or(0) & COUNT,
            },
            MONSTER => Hint::Empty,
            class => Hint::Object(class),
        }
    }

    /// A hero's index, class and own name, from the map's heroes or the viewer's list.
    fn hero(&self, id: u8) -> Option<(u8, u8, bool)> {
        self.heroes
            .iter()
            .find(|h| h.id == id)
            .map(|h| (h.id, h.class, h.own_name))
            .or_else(|| self.facts.hero_list.iter().copied().find(|h| h.0 == id))
    }

    /// Whether colours `a` and `b` share a team.
    fn allied(&self, a: Option<u8>, b: u8) -> bool {
        let team = |c: u8| self.facts.teams.get(usize::from(c)).copied();
        a.is_some_and(|a| team(a).is_some() && team(a) == team(b))
    }

    /// The cursor of the mouse at `at` over the map on the viewing player's turn
    /// (`adventure_mouse_move` 0x40E392 on), with the selected hero `selected` (none: a town or
    /// nothing selected); `reach`: the facts' reach is the selected hero's where it stands (none
    /// after it walked: the engine's from M4), else no tile is reached.
    #[must_use]
    pub fn cursor(&self, at: [i32; 3], selected: Option<Selection>, reach: bool) -> Cursor {
        let arrow = Cursor {
            frame: frame::ARROW,
            action: -1,
        };
        let Some(tile) = self.current(at, selected).filter(|_| self.explored(at)) else {
            return arrow;
        };
        let ours = |owner: Option<u8>| self.allied(owner, self.viewer);
        let Some(hero) = selected else {
            // No hero selected (0x40E570): a town of the team, a hero of the player's own, a
            // shipyard of the team.
            return match tile.class {
                crate::live::TOWN_CLASS if ours(tile.owner) => Cursor {
                    frame: frame::TOWN,
                    action: 3,
                },
                crate::live::HERO_CLASS if tile.owner == Some(self.viewer) => Cursor {
                    frame: frame::HERO,
                    action: 2,
                },
                SHIPYARD if tile.owner.is_some() && ours(tile.owner) => Cursor {
                    frame: frame::BOARD,
                    action: 8,
                },
                _ => arrow,
            };
        };
        let mine = [
            i32::from(hero.at.x),
            i32::from(hero.at.y),
            i32::from(hero.at.z),
        ];
        if mine[2] != at[2] {
            return arrow;
        }
        if mine == at {
            return Cursor {
                frame: frame::HERO,
                action: 2,
            };
        }
        let boat = self
            .heroes
            .iter()
            .find(|h| h.id == hero.id)
            .is_some_and(|h| h.boat);
        if tile.blocked {
            // 0x40E730: a town's or shipyard's body of the team; else the arrow.
            return match tile.class {
                crate::live::TOWN_CLASS if ours(tile.owner) => Cursor {
                    frame: frame::TOWN,
                    action: 5,
                },
                SHIPYARD if tile.owner.is_some() && ours(tile.owner) => Cursor {
                    frame: frame::BOARD,
                    action: 8,
                },
                _ => arrow,
            };
        }
        // 0x40E862: water from land only to a hero, a boat or a shipwreck there; from a boat
        // only water or the coast.
        let shore = [crate::live::HERO_CLASS, crate::live::BOAT_CLASS, SHIPWRECK];
        if !boat && tile.water && !(tile.entrance && shore.contains(&tile.class)) {
            return arrow;
        }
        if boat && !tile.water && tile.class != COAST {
            return arrow;
        }
        // 0x40E91B: the pathfinder's node of the tile; the days by the points left.
        let Some(&cost) = self.facts.reach.get(&(at[0], at[1])).filter(|_| reach) else {
            return arrow;
        };
        let days = days(cost, hero.points, self.facts.day_points);
        let mut action = 1;
        let kind = match tile.class {
            COAST if boat => frame::DISEMBARK,
            crate::live::BOAT_CLASS if !boat => frame::BOARD,
            crate::live::BOAT_CLASS => {
                action = -1;
                frame::ARROW
            }
            GARRISON | crate::live::TOWN_CLASS
                if tile.entrance && !ours(tile.owner) && self.armed(&tile) =>
            {
                frame::ATTACK
            }
            crate::live::HERO_CLASS if ours(tile.owner) => frame::EXCHANGE,
            crate::live::HERO_CLASS | MONSTER => frame::ATTACK,
            _ => self.kind(&tile, at, boat),
        };
        let frame = if boat && kind == frame::SAIL_VISIT {
            kind + days
        } else {
            kind + frame::DAY * days
        };
        Cursor { frame, action }
    }

    /// Whether a town's or garrison's tile holds an army that defends it.
    fn armed(&self, tile: &Tile) -> bool {
        tile.object
            .and_then(|o| self.facts.armed.get(o).copied())
            .unwrap_or(false)
    }

    /// The kind of a tile without a case of its own (`tile_cursor_kind` 0x40E280): a guarded
    /// tile the attack, but an entrance of an object entered from beside it; from a boat a
    /// visit on water, else the sail; on land a visit of an entrance (on water a shipwreck's),
    /// else the move.
    fn kind(&self, tile: &Tile, at: [i32; 3], boat: bool) -> u32 {
        let guarded = self
            .facts
            .guarded
            .explored(at[0], at[1], u8::try_from(at[2]).unwrap_or(u8::MAX));
        if guarded && !(tile.entrance && NOT_ENTERED.contains(&tile.class)) {
            return frame::ATTACK;
        }
        match (boat, tile.entrance, tile.water) {
            (true, true, true) => frame::SAIL_VISIT,
            (true, ..) => frame::BOARD,
            (false, true, false) => frame::VISIT,
            (false, true, true) if tile.class == SHIPWRECK => frame::VISIT,
            _ => frame::MOVE,
        }
    }
}

/// The monsters' class (54).
pub const MONSTER: u32 = 54;
/// A monster's count's bits of its word.
pub const COUNT: u32 = 0xFFF;
/// The garrisons' class (33).
pub const GARRISON: u32 = 33;
/// The shipyards' class (87).
pub const SHIPYARD: u32 = 87;
/// The shipwrecks' class (85).
pub const SHIPWRECK: u32 = 85;
/// The coast's class (3).
pub const COAST: u32 = 3;

/// The days a path of `cost` takes beyond today with `points` left and `day` points a day
/// (0x40E98F): 0 within the points, else one more for each day's points beyond, at most 3.
#[must_use]
pub fn days(cost: u32, points: i32, day: i32) -> u32 {
    let cost = i64::from(cost);
    let points = i64::from(points);
    if cost <= points {
        return 0;
    }
    let more = (cost - points - 1)
        .checked_div(i64::from(day))
        .unwrap_or(0)
        .saturating_add(1);
    u32::try_from(more.clamp(0, 3)).unwrap_or(3)
}

/// The special ground of a tile whose draw list is `entries` (0x4FCE20): the class of the
/// topmost object among [`GROUNDS`], 0 none.
fn ground(records: &Records, art: &MapArt, live: &Live, entries: &[Entry]) -> u32 {
    entries
        .iter()
        .rev()
        .filter_map(|e| records.classes(art, live, e).map(|(class, _, _)| class))
        .find(|c| GROUNDS.contains(c))
        .unwrap_or(0)
}

fn to_i32(v: usize) -> i32 {
    i32::try_from(v).unwrap_or(i32::MAX)
}

/// The objects' entrance tiles (`object_entrance_tile` 0x4FEC30), by object.
fn entrances(art: &MapArt) -> BTreeMap<usize, (i32, i32)> {
    (0..art.objects.len())
        .filter_map(|i| objects::entrance(art, i).map(|at| (i, at)))
        .collect()
}

/// The panels' items as the mouse finds them (`dialog_item_at` 0x5FF3E0: the last in the dialog's
/// order whose box holds the point and whose state takes the mouse, bit 0x2 without 0x8 or
/// 0x1000): the adventure dialog's items in their order, the chat's edit over the status bar, the
/// resource bar's items, then the status window's of the kind shown.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PanelItems {
    /// The items' ids and boxes on the screen (left, top, right, bottom; the last two
    /// exclusive), in the order searched last to first.
    items: Vec<(u16, [i32; 4])>,
    /// Each status window kind's items' boxes ([`crate::live::Status::kind`]: 1 the date's, 2
    /// the kingdom's, 3 a hero's, 4 a town's).
    status: Vec<(u8, Vec<[i32; 4]>)>,
}

/// The id the status window's items stand for: none has a hint (0x402C50 gives none above 1015).
pub const STATUS_ITEM: u16 = 2000;

impl PanelItems {
    /// The items of `layout`.
    #[must_use]
    pub fn new(layout: &AdventureLayout) -> Self {
        let boxes = |dialog: &h3_ui::ui::Dialog| -> Vec<(u16, [i32; 4])> {
            let [px, py] = dialog.place;
            dialog
                .items
                .iter()
                .filter_map(|item| {
                    let ([x, y], [w, h]) = item.area()?;
                    let (x, y) = (px.saturating_add(x), py.saturating_add(y));
                    Some((item.id(), [x, y, x.saturating_add(w), y.saturating_add(h)]))
                })
                .collect()
        };
        let mut items = boxes(&layout.dialog);
        // The chat's edit 38, added after the status bar over its box (0x5541A0).
        if let Some(&(_, bar)) = items.iter().find(|(i, _)| *i == id::STATUS_BAR) {
            items.push((id::EDIT, bar));
        }
        items.extend(boxes(&layout.resource_bar));
        let status = [
            (1, &layout.status_date),
            (2, &layout.status_kingdom),
            (3, &layout.status_hero),
            (4, &layout.status_town),
        ]
        .into_iter()
        .map(|(kind, dialog)| (kind, boxes(dialog).into_iter().map(|(_, b)| b).collect()))
        .collect();
        Self { items, status }
    }

    /// The item under the mouse at `point` with the status window of `kind` shown: the last
    /// whose box holds it among those `takes` lets take the mouse; none over no item.
    #[must_use]
    pub fn at(&self, point: [i32; 2], kind: Option<u8>, takes: impl Fn(u16) -> bool) -> Option<u16> {
        let inside = |[l, t, r, b]: [i32; 4]| (l..r).contains(&point[0]) && (t..b).contains(&point[1]);
        let status = self
            .status
            .iter()
            .find(|(k, _)| Some(*k) == kind)
            .is_some_and(|(_, boxes)| boxes.iter().any(|&b| inside(b)));
        if status {
            return Some(STATUS_ITEM);
        }
        self.items
            .iter()
            .rev()
            .find(|&&(i, b)| inside(b) && takes(i))
            .map(|&(i, _)| i)
    }
}

/// Whether the adventure dialog's item `item` takes the mouse for `kingdom` with `buttons` on a
/// map of `levels` levels (its state's bit 0x2 without 0x8; read from the running original's
/// items, design §4.10): a disabled button not ([`panels::disabled`]), but the quest log (its
/// state 0x20, still taking it); a list's picture and bars only in a slot holding a hero or a
/// town; the hero list's selection frames always; every other item.
#[must_use]
pub fn takes(item: u16, kingdom: &Kingdom, buttons: Buttons, levels: usize) -> bool {
    let [heroes_top, towns_top] = kingdom.tops;
    let slot = |first: u16| usize::from(item.saturating_sub(first));
    match item {
        id::QUESTS => true,
        i if (id::PORTRAITS..id::PORTRAITS + 15).contains(&i) => {
            heroes_top.saturating_add(slot(id::PORTRAITS) % SLOTS) < kingdom.heroes.len()
        }
        i if (id::TOWN_PICTURES..id::TOWN_PICTURES + 5).contains(&i) => {
            towns_top.saturating_add(slot(id::TOWN_PICTURES)) < kingdom.towns.len()
        }
        i => !panels::disabled(kingdom, buttons, levels)
            .iter()
            .any(|&(d, off)| d == i && off),
    }
}

/// The status bar's hint over the panels' item `item` (`adventure_panel_hint` 0x403010; none:
/// no item): a hero list's slot its hero's (`map.hint.hero`), a town list's its town's
/// (`map.hint.town`), each by `facts` (none without them), nothing in an empty slot; the resource
/// bar's items nothing; another item its pair of Help.txt's (0x402C50), if it has one.
#[must_use]
pub fn panel_hint(item: Option<u16>, kingdom: &Kingdom, facts: Option<&Facts>) -> Hint {
    let Some(item) = item else {
        return Hint::Empty;
    };
    let [heroes_top, towns_top] = kingdom.tops;
    let hero_slot = match item {
        15..=19 => Some(item - 15),
        20..=24 => Some(item - 20),
        25..=29 => Some(item - 25),
        39..=43 => Some(item - 39),
        _ => None,
    };
    if let Some(slot) = hero_slot {
        return facts
            .and_then(|f| f.hero_list.get(heroes_top.saturating_add(usize::from(slot))))
            .map_or(Hint::Empty, |&(id, class, own_name)| Hint::Hero {
                id,
                class,
                own_name,
            });
    }
    if let 32..=36 = item {
        return facts
            .and_then(|f| {
                let town = f.town_list.get(towns_top.saturating_add(usize::from(item - 32)))?;
                f.towns.get(*town)
            })
            .map_or(Hint::Empty, |&(name, kind)| Hint::Town { name, kind });
    }
    match item {
        1..=14 => Hint::Panel(u8::try_from(item - 1).unwrap_or(0)),
        30 => Hint::Panel(15),
        31 => Hint::Panel(16),
        id::STATUS_BAR => Hint::Panel(26),
        _ => Hint::Empty,
    }
}

/// The status bar's hint of a still with the mouse parked at `point` (the staging's last move,
/// off the view): the panel's item's there ([`panel_hint`]).
#[must_use]
pub fn parked_hint(layout: &AdventureLayout, live: &Live, levels: usize, point: [i32; 2]) -> Hint {
    let panels = PanelItems::new(layout);
    let buttons = live
        .buttons
        .unwrap_or_else(|| Buttons::of(&live.kingdom, &live.path));
    let kind = live.kingdom.status.as_ref().map(crate::live::Status::kind);
    let item = panels.at(point, kind, |i| takes(i, &live.kingdom, buttons, levels));
    panel_hint(item, &live.kingdom, live.hover.as_ref())
}

#[cfg(test)]
mod tests;
