//! The map's situations (E24 M1 T3b2, *provisional*): what the capture set shows the original
//! and compares with the viewer's still (`clients/map/situations/`, the harness's `h3map.py`).
//! A situation is a map built with the adventure lane's writer (`h3_map::writer::write_with`), a
//! view and the tiles the viewer has not explored. Its file (TOML) gives the map's side, its
//! terrain, patches and roads as the writer takes them, tiles copied from a rectangle of a
//! shipped map (its transitions, rivers and roads as the editor wrote them; the file names the
//! map, the harness's copy of the installation holds it) and tiles given one by one (a later
//! one replaces an earlier on its tile), objects (E24 M1 T4a: a copy's objects with its tiles,
//! and objects given by their template's DEF, its line of the game's `Objects.txt`), the heroes
//! of up to eight colours and the boats without a hero (E24 M1 T5a: each with the facing the
//! harness turns it to), the view's top-left and the rectangles left unexplored. The surface only. A situation that names a shipped map whole (`map`, E24 M1 T4b)
//! is the harness's alone: it loads that map, this module does not read it.

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use h3_map::RawMap;
use h3_map::writer::{
    AnyObject, Extras, GivenTile, Hero, Map, MapObject, ObjectTemplate, Patch, Player, RiverKind,
    Road, RoadKind, Terrain, Town, TownBuildings,
};
use serde::Deserialize;

/// One situation as written.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Situation {
    /// The map's side (36, 72, 108 or 144).
    pub size: u8,
    /// The terrain of every tile nothing else covers.
    pub terrain: Ground,
    /// Rectangles of other terrains, as the writer takes them (corners inclusive).
    #[serde(default)]
    pub patches: Vec<PatchSpec>,
    /// Straight roads along a row, as the writer takes them.
    #[serde(default)]
    pub roads: Vec<RoadSpec>,
    /// Rectangles copied from shipped maps.
    #[serde(default)]
    pub copy: Vec<CopySpec>,
    /// Tiles given one by one, after the copies.
    #[serde(default)]
    pub tiles: Vec<TileSpec>,
    /// Objects given one by one, after the copies' (in this order: the map's order).
    #[serde(default)]
    pub objects: Vec<ObjectSpec>,
    /// The towns (E24 M1 T5b1), after the heroes and before the objects in the map's order.
    #[serde(default)]
    pub towns: Vec<TownSpec>,
    /// The heroes (E24 M1 T5a), the map's first objects in this order: Red's (the human player)
    /// and another colour's at least, at most eight of a colour; every colour with a hero a
    /// player.
    pub heroes: Vec<HeroSpec>,
    /// The boats without a hero (E24 M1 T5a), the map's last objects in this order.
    #[serde(default)]
    pub boats: Vec<BoatSpec>,
    /// The view's top-left tile and level (0).
    pub view: [i32; 3],
    /// Rectangles of tiles Red has not explored (corners inclusive); every other tile explored.
    #[serde(default)]
    pub hidden: Vec<[i32; 4]>,
    /// What the harness selects from its list after its walks and writes (E24 M3 T4a; design
    /// §4.5): the selection scrolls its list to show it and redraws both lists (and builds the
    /// status window again, M3 T5b1), so a situation whose heroes or towns the harness writes
    /// into needs one, of another hero or town than the turn's start selects.
    #[serde(default)]
    pub select: Option<SelectSpec>,
    /// Red's resources the harness writes before its selection (E24 M3 T5a, design §4.7), in the
    /// original's order: a hero's selection draws the resource bar again (`hero_select`
    /// 0x417F48), a town's does not, so a situation that gives them selects a hero.
    #[serde(default)]
    pub resources: Option<[i32; 7]>,
    /// The game's date the harness writes before its selection (E24 M3 T5a): [month, week, day],
    /// a week from 1 to 4 and a day from 1 to 7; as the resources, with a hero's selection.
    #[serde(default)]
    pub date: Option<[u16; 3]>,
    /// The status window's kind the harness asks for after its selection (E24 M3 T5b2, design
    /// §4.9): 1 the date's, 2 the kingdom's, until long after the still (the manager's +0x398 and
    /// +0x39C, as `status_window_set_kind` 0x415D60 sets them); the next view draw builds it.
    #[serde(default)]
    pub status: Option<u8>,
    /// The players' teams by colour, which the harness writes into the game (E24 M3 T5b2, design
    /// §4.9: the kingdom's window shows the players of Red's team and the others; the writer's
    /// maps have none, each player its own), each from 0 to 7.
    #[serde(default)]
    pub teams: Option<[u8; 8]>,
}

/// What a situation selects (E24 M3 T4a): Red's hero standing on a tile (after its walk) or Red's
/// town anchored on one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub enum SelectSpec {
    /// The hero on this tile.
    Hero([u8; 2]),
    /// The town anchored on this tile.
    Town([u8; 2]),
}

/// A terrain by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ground {
    /// Dirt.
    Dirt,
    /// Sand.
    Sand,
    /// Grass.
    Grass,
    /// Snow.
    Snow,
    /// Swamp.
    Swamp,
    /// Rough.
    Rough,
    /// Subterranean.
    Subterranean,
    /// Lava.
    Lava,
    /// Water.
    Water,
    /// Rock.
    Rock,
}

/// A river by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiverName {
    /// Clear.
    Clear,
    /// Icy.
    Icy,
    /// Muddy.
    Muddy,
    /// Lava.
    Lava,
}

/// A road by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoadName {
    /// Dirt.
    Dirt,
    /// Gravel.
    Gravel,
    /// Cobblestone.
    Cobblestone,
}

/// A patch of terrain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchSpec {
    /// Its terrain.
    pub terrain: Ground,
    /// Its top-left tile.
    pub from: [u8; 2],
    /// Its bottom-right tile.
    pub to: [u8; 2],
}

/// A road along a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoadSpec {
    /// Its kind.
    pub kind: RoadName,
    /// The row.
    pub row: u8,
    /// Its first column.
    pub from: u8,
    /// Its last column.
    pub to: u8,
}

/// A rectangle of a shipped map's tiles copied onto the map.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopySpec {
    /// The shipped map's file name (in the installation's `Maps`).
    pub map: String,
    /// The rectangle's top-left tile there.
    pub from: [u8; 2],
    /// Its bottom-right tile.
    pub to: [u8; 2],
    /// Its level there (0 the surface).
    #[serde(default)]
    pub level: u8,
    /// Where its top-left goes on the situation's map.
    pub at: [u8; 2],
    /// Also the objects anchored in the rectangle (on its level), moved with its tiles, in the
    /// shipped map's order, each with its template and the editor's body, neutral. Heroes,
    /// prisons, random heroes and hero placeholders are left out (their bodies name a hero: the
    /// writer writes only the situation's two).
    #[serde(default)]
    pub objects: bool,
}

/// An object given by its template: the first line of the game's `Objects.txt` with its DEF
/// (ignoring case) and, when given, its class and subclass.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectSpec {
    /// The template's DEF.
    pub def: String,
    /// The class, to choose among the DEF's templates.
    #[serde(default)]
    pub class: Option<u32>,
    /// The subclass, likewise.
    #[serde(default)]
    pub subclass: Option<u32>,
    /// Its anchor (the bottom-right corner of its picture), which may lie past the map's right
    /// and bottom edges (the writer's limits).
    pub at: [u8; 2],
    /// Its owner's colour (0 Red … 7 Pink; E24 M1 T5b1), for a class whose body names one (the
    /// writer's `AnyObject`): a colour with a hero, which makes it a player. None is neutral.
    #[serde(default)]
    pub owner: Option<u8>,
}

/// A town (E24 M1 T5b1): its faction's template (`Objects.txt`'s first of class 98 and the
/// faction), its owner and buildings.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TownSpec {
    /// Its anchor; its entrance is two columns left of it.
    pub at: [u8; 2],
    /// Its faction (0 Castle … 8 Conflux).
    pub faction: u32,
    /// Its owner's colour (a colour with a hero), none for a neutral town.
    #[serde(default)]
    pub owner: Option<u8>,
    /// Its buildings, which give its picture (design §2.9).
    pub buildings: TownBuilt,
    /// A building built in it today (+0x2), written by the harness before the still (E24 M3 T4a;
    /// design §4.5: the town list's picture): Red's towns only.
    #[serde(default)]
    pub built: bool,
    /// A Resource Silo among its buildings (E24 M3 T5b1, design §4.8: the status window's silo;
    /// the map's building 9): with chosen buildings only (a Citadel, Castle or Capitol).
    #[serde(default)]
    pub silo: bool,
    /// Its garrison's slots, [creature, count] each, written by the harness into its record
    /// before the selection (E24 M3 T5b1: the status window's army): Red's towns only, at most
    /// seven.
    #[serde(default)]
    pub garrison: Vec<[i32; 2]>,
    /// A hero of Red's (on its tile, after its walk) the harness makes its garrison hero in its
    /// record (+0xC) before the selection (E24 M3 T5b1: only the status window reads it, its mark
    /// and that hero's army): Red's towns only.
    #[serde(default)]
    pub garrison_hero: Option<[u8; 2]>,
}

impl TownSpec {
    /// The writer's buildings: its set's, with the Resource Silo (the map's building 9) when it
    /// has one (E24 M3 T5b1; the checks refuse a silo with a standard set).
    fn writer_buildings(&self) -> TownBuildings {
        match self.buildings.writer() {
            TownBuildings::Custom {
                mut built,
                forbidden,
            } if self.silo => {
                built[1] |= 1 << 1;
                TownBuildings::Custom { built, forbidden }
            }
            other => other,
        }
    }
}

/// A town's buildings as a situation names them: the editor's standard set without or with a
/// Fort, or chosen ones up to a Citadel, a Castle or a Capitol (with the Town and City Halls).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TownBuilt {
    /// The standard set without a Fort.
    Village,
    /// The standard set with a Fort.
    Fort,
    /// A Fort and a Citadel.
    Citadel,
    /// The Town Hall and a Fort (E24 M3 T5b2: the kingdom's window counts the towns by their
    /// halls).
    TownHall,
    /// The Town and City Halls and a Fort (E24 M3 T5b2).
    CityHall,
    /// A Fort, a Citadel and a Castle.
    Castle,
    /// The Town, City Hall and Capitol, a Fort, a Citadel and a Castle.
    Capitol,
}

impl TownBuilt {
    /// The writer's buildings (the map's bits: 0 Town Hall, 1 City Hall, 2 Capitol, 3 Fort,
    /// 4 Citadel, 5 Castle; `docs/formats/h3m.md`).
    #[must_use]
    pub fn writer(self) -> TownBuildings {
        let custom = |byte: u8| TownBuildings::Custom {
            built: [byte, 0, 0, 0, 0, 0],
            forbidden: [0; 6],
        };
        match self {
            Self::Village => TownBuildings::Standard { fort: false },
            Self::Fort => TownBuildings::Standard { fort: true },
            Self::Citadel => custom(0b1_1000),
            Self::TownHall => custom(0b1001),
            Self::CityHall => custom(0b1011),
            Self::Castle => custom(0b11_1000),
            Self::Capitol => custom(0b11_1111),
        }
    }
}

/// The towns' class.
const TOWN_CLASS: u32 = 98;

/// A hero (E24 M1 T5a).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeroSpec {
    /// The tile it stands on (the writer puts its anchor one column right).
    pub at: [u8; 2],
    /// Its owner's colour (0 Red … 7 Pink).
    #[serde(default)]
    pub owner: u8,
    /// Its class (0–17), which gives its map picture: its hero type is the class's first not
    /// taken by an earlier hero ([`hero_type`]).
    #[serde(default)]
    pub class: u8,
    /// The tiles the harness walks it to in turn before the still, by play (Red's heroes only:
    /// the human player's; a hero boards a boat by walking onto it, design §2.8).
    #[serde(default)]
    pub walk: Vec<[u8; 2]>,
    /// Where the harness turns it before the still, after its walk (0–7, the hero record's +0x47;
    /// design §2.8): a new game faces every hero right (2).
    #[serde(default)]
    pub facing: Option<u8>,
    /// The tile its path leads to, shown by the harness after the view is set: the hero selected
    /// and the tile clicked once (E24 M1 T5b2; design §2.10, passes 6 and 8). Red's heroes only,
    /// one at most (the selected hero's path is the one shown).
    #[serde(default)]
    pub path: Option<[u8; 2]>,
    /// Its movement points left (+0x4D), written by the harness after its walk (E24 M3 T4a;
    /// design §4.5: the hero list's movement bar): Red's heroes only.
    #[serde(default)]
    pub points: Option<i32>,
    /// Its mana (+0x18), likewise (the mana bar).
    #[serde(default)]
    pub mana: Option<i32>,
    /// Its attack, defense, power and knowledge as stored (i8 +0x476), likewise (E24 M3 T5b1,
    /// design §4.8: the status window).
    #[serde(default)]
    pub skills: Option<[i8; 4]>,
    /// Its morale until its next battle (i8 +0x11A), likewise: the status window shows its
    /// morale with it.
    #[serde(default)]
    pub morale: Option<i8>,
    /// Its luck until its next battle (i8 +0x11B), likewise.
    #[serde(default)]
    pub luck: Option<i8>,
    /// Its army's slots, [creature, count] each (+0x91, +0xAD), likewise; at most seven.
    #[serde(default)]
    pub army: Vec<[i32; 2]>,
}

impl HeroSpec {
    /// Whether the harness writes anything into its record (E24 M3 T4a, T5b1).
    fn written(&self) -> bool {
        self.points.is_some()
            || self.mana.is_some()
            || self.skills.is_some()
            || self.morale.is_some()
            || self.luck.is_some()
            || !self.army.is_empty()
    }
}

/// A boat without a hero (E24 M1 T5a).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoatSpec {
    /// The tile it lies on: its template's visitable cell, the anchor one column right.
    pub at: [u8; 2],
    /// Its type (0–2): the template `AVXboat0.def` … `AVXboat2.def` of class 8.
    pub kind: u8,
    /// Where the harness turns it before the still (0–7, the boat record's +0x1B; design §2.8).
    #[serde(default)]
    pub facing: Option<u8>,
    /// The flag's colour the harness writes into its record once a Red hero boarded it (0–7, the
    /// boat record's +0x1C; E24 M1 T5b2): a still's staging, as the facings (boarding gives the
    /// boat its hero's owner, design §2.8), so that the boats' flags of every colour are drawn.
    #[serde(default)]
    pub colour: Option<u8>,
}

/// The boats' class.
const BOAT_CLASS: u32 = 8;

/// The hero type of the `nth` hero (from 0) of `class` on a map: the class's eight types in
/// order (classes 0–15 the types 8 × class onwards, the AB classes 16 and 17 the types 128 and
/// 136 onwards: the packs' hero list), `None` past the eighth or for no class.
#[must_use]
pub fn hero_type(class: u8, nth: usize) -> Option<u8> {
    let first = match class {
        0..=15 => class * 8,
        16 | 17 => 128 + (class - 16) * 8,
        _ => return None,
    };
    u8::try_from(nth).ok().filter(|&n| n < 8).map(|n| first + n)
}

/// The classes whose bodies name a hero (the writer's `AnyObject` takes none of them).
const HERO_CLASSES: [u32; 4] = [34, 62, 70, 214];

/// A tile as given: the writer's [`GivenTile`] on the surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TileSpec {
    /// The tile.
    pub at: [u8; 2],
    /// Its terrain.
    pub terrain: Ground,
    /// The terrain's picture.
    pub picture: u8,
    /// Its river and picture.
    #[serde(default)]
    pub river: Option<(RiverName, u8)>,
    /// Its road and picture.
    #[serde(default)]
    pub road: Option<(RoadName, u8)>,
    /// The mirror bits (and the coast bit) as stored.
    #[serde(default)]
    pub flags: u8,
}

impl Situation {
    /// Parses a situation and checks its view's level (the surface only).
    ///
    /// # Errors
    /// The parser's message, a view on another level or one more than the map's side off it.
    pub fn parse(text: &str) -> Result<Self, String> {
        // A shipped map loaded whole (`map`, E24 M1 T4b) is the harness's (E24 M1 T6b, R2-12).
        let table: toml::Table = toml::from_str(text).map_err(|e| e.to_string())?;
        if table.contains_key("map") {
            return Err("it names a shipped map whole (`map`): the harness loads it".into());
        }
        let situation: Self = toml::from_str(text).map_err(|e| e.to_string())?;
        if situation.view[2] != 0 {
            return Err("a situation's view is on the surface (level 0)".into());
        }
        // The view's corner near the map (M2 T6a, the reviewer's R1-8: one far off overflowed the
        // tile under a click); the original's own range, -9 to the side less 10, is the harness's
        // check (`render --live`).
        let side = i32::from(situation.size);
        if situation.view[..2]
            .iter()
            .any(|v| !(-side..=side).contains(v))
        {
            return Err(format!(
                "the view's corner {:?} lies more than the map's side ({side}) off the map",
                &situation.view[..2]
            ));
        }
        Ok(situation)
    }

    /// The shipped maps the copies read.
    #[must_use]
    pub fn sources(&self) -> BTreeSet<String> {
        self.copy.iter().map(|c| c.map.clone()).collect()
    }

    /// Whether objects or boats are given by their DEF (then [`Situation::build`] needs the
    /// game's templates).
    #[must_use]
    pub fn needs_templates(&self) -> bool {
        !self.objects.is_empty() || !self.boats.is_empty() || !self.towns.is_empty()
    }

    /// The map, its given tiles and its objects, the copies read from `sources` (the shipped maps
    /// by file name; [`Situation::sources`]), the given objects' and boats' templates from
    /// `templates` (the game's `Objects.txt`, in its order). Every colour with a hero is a player
    /// that may be human or computer (the scenario list makes Red the human one); the heroes come
    /// first, then the towns, the objects, the boats.
    ///
    /// # Errors
    /// A source missing, a copied rectangle outside its map or the situation's, a code with no
    /// name, a given object's, town's or boat's template not among `templates`; no hero of Red's or of
    /// another colour, an owner past 7, more than eight heroes of a colour or of a class, a
    /// facing or a boat's colour past 7, a walk or a path of a hero not Red's, a second path.
    pub fn build(
        &self,
        sources: &BTreeMap<String, RawMap>,
        templates: &[ObjectTemplate],
    ) -> Result<(Map, Extras), String> {
        let mut given: BTreeMap<(u8, u8), GivenTile> = BTreeMap::new();
        let mut objects = Vec::new();
        for c in &self.copy {
            let raw = sources
                .get(&c.map)
                .ok_or_else(|| format!("{}: not read", c.map))?;
            for tile in copied(c, raw, self.size)? {
                given.insert(tile.tile, tile);
            }
            if c.objects {
                objects.extend(copied_objects(c, raw));
            }
        }
        for o in &self.objects {
            let template = templates
                .iter()
                .find(|t| {
                    t.def.eq_ignore_ascii_case(&o.def)
                        && o.class.is_none_or(|c| c == t.class)
                        && o.subclass.is_none_or(|s| s == t.subclass)
                })
                .ok_or_else(|| format!("{}: no such template in Objects.txt", o.def))?;
            objects.push(AnyObject {
                level: 0,
                tile: (o.at[0], o.at[1]),
                template: template.clone(),
                owner: o.owner,
            });
        }
        for t in &self.tiles {
            let tile = GivenTile {
                level: 0,
                tile: (t.at[0], t.at[1]),
                terrain: t.terrain.writer(),
                picture: t.picture,
                river: t.river.map(|(r, p)| (r.writer(), p)),
                road: t.road.map(|(r, p)| (r.writer(), p)),
                flags: t.flags,
            };
            given.insert(tile.tile, tile);
        }
        objects.extend(self.boats(templates)?);
        let (players, mut heroes) = self.heroes()?;
        self.selection()?;
        for t in &self.towns {
            let template = templates
                .iter()
                .find(|o| (o.class, o.subclass) == (TOWN_CLASS, t.faction))
                .ok_or_else(|| {
                    format!("town at {:?}: no template of faction {}", t.at, t.faction)
                })?;
            heroes.push(MapObject::Town(Town {
                tile: (t.at[0], t.at[1]),
                template: template.clone(),
                owner: t.owner,
                garrison: None,
                tight_formation: false,
                buildings: t.writer_buildings(),
            }));
        }
        let map = Map {
            name: "situation".into(),
            description: String::new(),
            size: self.size,
            terrain: self.terrain.writer(),
            patches: self
                .patches
                .iter()
                .map(|p| Patch {
                    terrain: p.terrain.writer(),
                    from: (p.from[0], p.from[1]),
                    to: (p.to[0], p.to[1]),
                })
                .collect(),
            roads: self
                .roads
                .iter()
                .map(|r| Road {
                    kind: r.kind.writer(),
                    row: r.row,
                    from: r.from,
                    to: r.to,
                })
                .collect(),
            players,
            objects: heroes,
            underground: None,
            level_limit: 0,
            banned_skills: vec![],
        };
        let extras = Extras {
            tiles: given.into_values().collect(),
            objects,
        };
        Ok((map, extras))
    }

    /// The players and the heroes: every colour with a hero a player (human or computer), each
    /// hero of the next free type of its class.
    fn heroes(&self) -> Result<([Option<Player>; 8], Vec<MapObject>), String> {
        let mut players: [Option<Player>; 8] = Default::default();
        let mut of_owner = [0usize; 8];
        let mut of_class = [0usize; 18];
        let mut paths = None;
        let mut heroes = Vec::new();
        for h in &self.heroes {
            let facing = h.facing.unwrap_or(0);
            let owner = usize::from(h.owner);
            let (Some(count), true) = (of_owner.get_mut(owner), facing < 8) else {
                return Err(format!(
                    "hero at {:?}: owner {} or facing {facing} past 7",
                    h.at, h.owner
                ));
            };
            let written = h.points.is_some() || h.mana.is_some();
            if h.owner != 0 && (!h.walk.is_empty() || h.path.is_some() || written) {
                return Err(format!(
                    "hero at {:?}: only Red's heroes walk, show a path or have points or mana \
                     written",
                    h.at
                ));
            }
            if h.path.is_some() && paths.replace(h.at).is_some() {
                return Err(format!("hero at {:?}: a second path shown", h.at));
            }
            *count += 1;
            if *count > 8 {
                return Err(format!("hero at {:?}: a ninth of colour {owner}", h.at));
            }
            let nth = of_class.get(usize::from(h.class)).copied().unwrap_or(0);
            let hero_type = hero_type(h.class, nth).ok_or_else(|| {
                if h.class < 18 {
                    format!("hero at {:?}: a ninth of class {}", h.at, h.class)
                } else {
                    format!(
                        "hero at {:?}: class {} is no hero class (0-17)",
                        h.at, h.class
                    )
                }
            })?;
            of_class[usize::from(h.class)] += 1;
            players[owner] = Some(Player {
                human: true,
                computer: true,
                towns: 0x1ff,
            });
            heroes.push(MapObject::Hero(Hero {
                tile: (h.at[0], h.at[1]),
                owner: h.owner,
                hero_type,
                class: h.class,
                name: None,
                experience: None,
                skills: None,
                army: None,
                tight_formation: false,
                artifacts: None,
                spells: None,
                primary: None,
            }));
        }
        if of_owner[0] == 0 || of_owner[1..].iter().all(|&n| n == 0) {
            return Err("a situation needs a hero of Red's and of another colour".into());
        }
        Ok((players, heroes))
    }

    /// The harness's writes into the lists and its selection (E24 M3 T4a): a town's day written
    /// only for Red's; the selection Red's hero on its tile (after its walk) or Red's town on its
    /// anchor; a selection whenever something is written, so that the lists show it; a hero's
    /// with the resources or the date (T5a), a day of the game's; since T5b1 the status window's
    /// writes into Red's heroes and towns, a garrison hero of Red's, seven slots at most, a silo
    /// with chosen buildings.
    fn selection(&self) -> Result<(), String> {
        if let Some(t) = self.towns.iter().find(|t| t.built && t.owner != Some(0)) {
            return Err(format!(
                "town at {:?}: only Red's towns have their day written",
                t.at
            ));
        }
        if let Some([month, week, day]) = self.date
            && (month == 0 || !(1..=4).contains(&week) || !(1..=7).contains(&day))
        {
            return Err(format!("the date {:?} is no day of the game's", self.date));
        }
        let bar = self.resources.is_some() || self.date.is_some();
        if bar && !matches!(self.select, Some(SelectSpec::Hero(_))) {
            return Err("the resources or the date without a hero's selection".into());
        }
        // The status window's kind asked (E24 M3 T5b2): the date's or the kingdom's; the teams.
        if let Some(kind) = self.status
            && !(1..=2).contains(&kind)
        {
            return Err(format!("the status window's kind {kind}: not 1 or 2"));
        }
        if let Some(teams) = self.teams
            && teams.iter().any(|&t| t > 7)
        {
            return Err(format!("the teams {teams:?}: one past 7"));
        }
        // The status window's (E24 M3 T5b1): Red's towns' garrisons and garrison heroes, of
        // Red's; seven slots at most; a silo with chosen buildings.
        for t in &self.towns {
            let garrisoned = !t.garrison.is_empty() || t.garrison_hero.is_some();
            if garrisoned && t.owner != Some(0) {
                return Err(format!(
                    "town at {:?}: only Red's towns have their garrison written",
                    t.at
                ));
            }
            if let Some(at) = t.garrison_hero
                && !self
                    .heroes
                    .iter()
                    .any(|h| h.owner == 0 && h.walk.last().copied().unwrap_or(h.at) == at)
            {
                return Err(format!(
                    "town at {:?}: its garrison hero is none of Red's",
                    t.at
                ));
            }
            if t.silo && matches!(t.buildings, TownBuilt::Village | TownBuilt::Fort) {
                return Err(format!(
                    "town at {:?}: a silo with the standard buildings",
                    t.at
                ));
            }
        }
        let slots = self
            .heroes
            .iter()
            .map(|h| h.army.len())
            .chain(self.towns.iter().map(|t| t.garrison.len()));
        if slots.into_iter().any(|n| n > 7) {
            return Err("an army of more than seven slots".into());
        }
        let written = self.heroes.iter().any(HeroSpec::written)
            || self
                .towns
                .iter()
                .any(|t| t.built || !t.garrison.is_empty() || t.garrison_hero.is_some());
        let found = match self.select {
            None if written => return Err("a write into the lists without a selection".into()),
            None => true,
            Some(SelectSpec::Hero(at)) => self
                .heroes
                .iter()
                .any(|h| h.owner == 0 && h.walk.last().copied().unwrap_or(h.at) == at),
            Some(SelectSpec::Town(at)) => {
                self.towns.iter().any(|t| t.owner == Some(0) && t.at == at)
            }
        };
        if found {
            Ok(())
        } else {
            Err(format!("the selection {:?} is none of Red's", self.select))
        }
    }

    /// The boats as objects: each of its type's template, anchored one column right of its tile.
    fn boats(&self, templates: &[ObjectTemplate]) -> Result<Vec<AnyObject>, String> {
        self.boats
            .iter()
            .map(|b| {
                let template = templates
                    .iter()
                    .find(|t| (t.class, t.subclass) == (BOAT_CLASS, u32::from(b.kind)))
                    .filter(|_| b.facing.unwrap_or(0) < 8 && b.colour.unwrap_or(0) < 8)
                    .ok_or_else(|| {
                        format!(
                            "boat at {:?}: no template of type {} or a facing or colour past 7",
                            b.at, b.kind
                        )
                    })?;
                Ok(AnyObject {
                    level: 0,
                    tile: (b.at[0].saturating_add(1), b.at[1]),
                    template: template.clone(),
                    owner: None,
                })
            })
            .collect()
    }
}

/// The tiles of a copy: each of the rectangle's tiles of `raw` as given, moved to `at`.
fn copied(c: &CopySpec, raw: &RawMap, side: u8) -> Result<Vec<GivenTile>, String> {
    let (src, levels) = (raw.terrain.size, raw.terrain.levels);
    let [x0, y0] = c.from.map(usize::from);
    let [x1, y1] = c.to.map(usize::from);
    let inside = x0 <= x1 && y0 <= y1 && x1 < src && y1 < src && usize::from(c.level) < levels;
    let fits = |at: u8, from: usize, to: usize| usize::from(at) + (to - from) < usize::from(side);
    if !inside || !fits(c.at[0], x0, x1) || !fits(c.at[1], y0, y1) {
        return Err(format!(
            "{}: the rectangle {:?}-{:?} does not fit (the map's side {src}, the situation's {side})",
            c.map, c.from, c.to
        ));
    }
    let mut out = Vec::new();
    for y in y0..=y1 {
        for x in x0..=x1 {
            let name = |what: &str, code: u8| format!("{}: ({x}, {y}): {what} code {code}", c.map);
            let t = raw
                .terrain
                .tiles
                .get((usize::from(c.level) * src + y) * src + x)
                .ok_or_else(|| name("no tile", 0))?;
            let terrain = terrain(t.terrain).ok_or_else(|| name("terrain", t.terrain))?;
            let river = match t.river {
                0 => None,
                code => Some((river(code).ok_or_else(|| name("river", code))?, t.river_dir)),
            };
            let road = match t.road {
                0 => None,
                code => Some((road(code).ok_or_else(|| name("road", code))?, t.road_dir)),
            };
            let at = |base: u8, v: usize, from: usize| {
                u8::try_from(usize::from(base) + v - from).unwrap_or(u8::MAX)
            };
            out.push(GivenTile {
                level: 0,
                tile: (at(c.at[0], x, x0), at(c.at[1], y, y0)),
                terrain,
                picture: t.view,
                river,
                road,
                flags: t.flags,
            });
        }
    }
    Ok(out)
}

/// The objects of a copy (it fitted: [`copied`]): those anchored in the rectangle on its level,
/// but the hero classes, moved to `at`, with their templates as the shipped map gives them.
fn copied_objects(c: &CopySpec, raw: &RawMap) -> Vec<AnyObject> {
    let [x0, y0] = c.from;
    let [x1, y1] = c.to;
    raw.objects
        .iter()
        .filter(|o| {
            o.pos.z == c.level && (x0..=x1).contains(&o.pos.x) && (y0..=y1).contains(&o.pos.y)
        })
        .filter_map(|o| {
            let index = usize::try_from(o.template).unwrap_or(usize::MAX);
            let template = raw.templates.get(index).map(|t| ObjectTemplate {
                def: t.def.0.iter().map(|&b| char::from(b)).collect(),
                passable: t.passable,
                visitable: t.visitable,
                terrains: t.terrains,
                menu: t.editor_terrains,
                class: t.class,
                subclass: t.subclass,
                group: t.group,
                priority: t.priority,
            });
            template
                .filter(|t| !HERO_CLASSES.contains(&t.class))
                .map(|template| AnyObject {
                    level: 0,
                    tile: (c.at[0] + (o.pos.x - x0), c.at[1] + (o.pos.y - y0)),
                    template,
                    owner: None,
                })
        })
        .collect()
}

impl Ground {
    fn writer(self) -> Terrain {
        match self {
            Self::Dirt => Terrain::Dirt,
            Self::Sand => Terrain::Sand,
            Self::Grass => Terrain::Grass,
            Self::Snow => Terrain::Snow,
            Self::Swamp => Terrain::Swamp,
            Self::Rough => Terrain::Rough,
            Self::Subterranean => Terrain::Subterranean,
            Self::Lava => Terrain::Lava,
            Self::Water => Terrain::Water,
            Self::Rock => Terrain::Rock,
        }
    }
}

impl RiverName {
    fn writer(self) -> RiverKind {
        match self {
            Self::Clear => RiverKind::Clear,
            Self::Icy => RiverKind::Icy,
            Self::Muddy => RiverKind::Muddy,
            Self::Lava => RiverKind::Lava,
        }
    }
}

impl RoadName {
    fn writer(self) -> RoadKind {
        match self {
            Self::Dirt => RoadKind::Dirt,
            Self::Gravel => RoadKind::Gravel,
            Self::Cobblestone => RoadKind::Cobblestone,
        }
    }
}

/// The writer's terrain of a `.h3m` code.
fn terrain(code: u8) -> Option<Terrain> {
    [
        Terrain::Dirt,
        Terrain::Sand,
        Terrain::Grass,
        Terrain::Snow,
        Terrain::Swamp,
        Terrain::Rough,
        Terrain::Subterranean,
        Terrain::Lava,
        Terrain::Water,
        Terrain::Rock,
    ]
    .get(usize::from(code))
    .copied()
}

/// The writer's river of a `.h3m` code (1–4).
fn river(code: u8) -> Option<RiverKind> {
    [
        RiverKind::Clear,
        RiverKind::Icy,
        RiverKind::Muddy,
        RiverKind::Lava,
    ]
    .get(usize::from(code).wrapping_sub(1))
    .copied()
}

/// The writer's road of a `.h3m` code (1–3).
fn road(code: u8) -> Option<RoadKind> {
    [RoadKind::Dirt, RoadKind::Gravel, RoadKind::Cobblestone]
        .get(usize::from(code).wrapping_sub(1))
        .copied()
}

#[cfg(test)]
mod tests;
