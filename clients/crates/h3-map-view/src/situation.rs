//! The map's situations (E24 M1 T3b2, *provisional*): what the capture set shows the original
//! and compares with the viewer's still (`clients/map/situations/`, the harness's `h3map.py`).
//! A situation is a map built with the adventure lane's writer (`h3_map::writer::write_with`), a
//! view and the tiles the viewer has not explored. Its file (TOML) gives the map's side, its
//! terrain, patches and roads as the writer takes them, tiles copied from a rectangle of a
//! shipped map (its transitions, rivers and roads as the editor wrote them; the file names the
//! map, the harness's copy of the installation holds it) and tiles given one by one (a later
//! one replaces an earlier on its tile), Red's and Blue's heroes (outside the view: T5 draws
//! heroes), the view's top-left and the rectangles left unexplored. The surface only.

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use h3_map::RawMap;
use h3_map::writer::{
    Extras, GivenTile, Hero, Map, MapObject, Patch, Player, RiverKind, Road, RoadKind, Terrain,
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
    /// Red's hero's tile, then Blue's.
    pub heroes: [[u8; 2]; 2],
    /// The view's top-left tile and level (0).
    pub view: [i32; 3],
    /// Rectangles of tiles Red has not explored (corners inclusive); every other tile explored.
    #[serde(default)]
    pub hidden: Vec<[i32; 4]>,
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
}

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
    /// The parser's message, or a view on another level.
    pub fn parse(text: &str) -> Result<Self, String> {
        let situation: Self = toml::from_str(text).map_err(|e| e.to_string())?;
        if situation.view[2] != 0 {
            return Err("a situation's view is on the surface (level 0)".into());
        }
        Ok(situation)
    }

    /// The shipped maps the copies read.
    #[must_use]
    pub fn sources(&self) -> BTreeSet<String> {
        self.copy.iter().map(|c| c.map.clone()).collect()
    }

    /// The map and its given tiles, the copies read from `sources` (the shipped maps by file
    /// name; [`Situation::sources`]). Red and Blue are human and computer players with a hero
    /// each (hero types 16 and 0, classes 2 and 0), Red's first.
    ///
    /// # Errors
    /// A source missing, a copied rectangle outside its map or the situation's, a code with no
    /// name.
    pub fn build(&self, sources: &BTreeMap<String, RawMap>) -> Result<(Map, Extras), String> {
        let mut given: BTreeMap<(u8, u8), GivenTile> = BTreeMap::new();
        for c in &self.copy {
            let raw = sources
                .get(&c.map)
                .ok_or_else(|| format!("{}: not read", c.map))?;
            for tile in copied(c, raw, self.size)? {
                given.insert(tile.tile, tile);
            }
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
        let player = Some(Player {
            human: true,
            computer: true,
            towns: 0x1ff,
        });
        let hero = |owner: u8, [x, y]: [u8; 2], hero_type, class| {
            MapObject::Hero(Hero {
                tile: (x, y),
                owner,
                hero_type,
                class,
                name: None,
                experience: None,
                skills: None,
                army: None,
                tight_formation: false,
                artifacts: None,
                spells: None,
                primary: None,
            })
        };
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
            players: [player.clone(), player, None, None, None, None, None, None],
            objects: vec![
                hero(0, self.heroes[0], 16, 2),
                hero(1, self.heroes[1], 0, 0),
            ],
            underground: None,
            level_limit: 0,
            banned_skills: vec![],
        };
        let extras = Extras {
            tiles: given.into_values().collect(),
            objects: vec![],
        };
        Ok((map, extras))
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
