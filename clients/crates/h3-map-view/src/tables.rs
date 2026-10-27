//! The map's asset tables (E24 M1 T2; ADR-0020 points 5 and 6): `clients/map/assets/<id>.toml`,
//! one per rule pack, in the packs' dependency graph. They give the archive entries and the
//! numbers the view draws with: the terrains', rivers' and roads' DEFs by the `.h3m`'s codes and
//! their palette cycles, the map's edge, the unexplored tiles' pieces and their table, the
//! moment's steps (`docs/epics/E24-design.md` §1.3 to §1.6). Identifiers and numbers only
//! (ADR-0009). The format is described in `clients/README.md` ("The map's asset tables"); it is
//! *provisional* until the M1 review (keyed by the map's codes, not by the packs' keys: the
//! static art is the raw map's).
//!
//! [`parse`] reads one file; [`load`] takes an expansion selection (the topmost table's id) and
//! merges it with its dependencies, dependencies first. Unknown sections and fields, unknown or
//! circular dependencies, a code defined twice and a `[map]` in other than exactly one table of
//! the selection are errors.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use serde::Deserialize;

use crate::art::Layer;

/// One table file as written.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableFile {
    /// The table's id and dependencies.
    pub table: Header,
    /// The view's own art and numbers (one table of a selection holds them).
    #[serde(default)]
    pub map: Option<ViewArt>,
    /// The terrains, by code (0 dirt … 9 rock).
    #[serde(default)]
    pub terrain: Vec<LayerArt>,
    /// The rivers, by code (1 clear … 4 lava; 0 is none).
    #[serde(default)]
    pub river: Vec<LayerArt>,
    /// The roads, by code (1 dirt … 3 cobblestone; 0 is none).
    #[serde(default)]
    pub road: Vec<LayerArt>,
}

/// A table's id (its pack's) and the tables it builds on.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Header {
    /// The table's id: its pack's.
    pub id: String,
    /// The ids of the tables it depends on (its pack's dependencies).
    pub depends: Vec<String>,
}

/// The view's own art and numbers.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewArt {
    /// The map's edge, drawn on the view's tiles off the map (`edg.def`, design §1.4).
    pub edge: String,
    /// The path's arrows and their shadows (`adag.def`, the manager's +0xB0): a tile's arrow a
    /// draws frame a − 1, 1 to 25 green, 26 to 50 red (passes 6 and 8; E24 M1 T5b2, design
    /// §2.2, §2.10).
    pub path: String,
    /// How far below its tile a road's picture is drawn (16: its lower half reaches into the
    /// tile below, design §1.3).
    pub road_offset: i32,
    /// The palette cycles' step (200 ms: the tick 0x4ED490 on the map, design §1.6).
    pub cycle_ms: u32,
    /// The objects' frame step (180 ms: the manager's +0x100, design §1.6).
    pub frame_ms: u32,
    /// The tiles a scroll step moves the view at the scroll speeds 0 to 2 (1, 2, 3:
    /// `scroll_steps` 0x63A66C; E24 M2 T2, design §3.4).
    pub scroll_steps: Vec<i32>,
    /// The edge scroll's time from the last step, or the pointer's arrival, to the next (70 ms:
    /// `edge_scroll` 0x419820).
    pub scroll_ms: u32,
    /// The mouse poll's (33 ms: `mouse_poll` 0x50D680, its next time moved on by at least this):
    /// the cursor is drawn where the poll last read the mouse (E17 design §9.10).
    pub poll_ms: u32,
    /// The unexplored tiles' pieces (design §1.5).
    pub shroud: Shroud,
    /// The heroes' and boats' pictures (E24 M1 T5a; design §2.3, §2.8).
    pub heroes: HeroArt,
    /// The owned objects' colours (E24 M1 T5b1; design §2.9).
    pub owners: OwnerArt,
    /// The towns' pictures by their buildings (E24 M1 T5b1; design §2.9).
    pub towns: TownArt,
    /// The hero's walk (E24 M2 T3a; design §3.5).
    pub walk: WalkArt,
}

/// The hero's walk (E24 M2 T3a; design §3.5, `hero_walk_animation` 0x480380): the group a step
/// draws, the frames' time and the pixels each frame slides the view by, at each hero speed.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WalkArt {
    /// The walking group by the step's direction, 0 north to 7 north-west (the switch 0x4805BC;
    /// mirrored above 4 as the standing groups, [`HeroArt::groups`]).
    pub groups: Vec<u32>,
    /// The frames a walking group steps through (pass 9's +0x200 mod 8, 0x47FF74): one a frame
    /// drawn, carried from step to step.
    pub frames: u32,
    /// A frame's time at the hero speeds 1 to 3 (`walk_frame_times` 0x63D6DC: 50 ms).
    pub frame_ms: u32,
    /// The pixels each frame of a step slides the view by, at the hero speeds 1 to 4
    /// (`walk_frame_pixels` 0x63D6C8: a tile in 32 / that many frames; speed 2's middle frame 12,
    /// 0x480509): none at speed 4, one draw at the step's end place without a wait (0x480483).
    pub pixels: Vec<Vec<i32>>,
}

impl WalkArt {
    /// The groups a step draws: one past the highest.
    #[must_use]
    pub fn drawn_groups(&self) -> u32 {
        self.groups.iter().max().map_or(0, |g| g + 1)
    }
}

/// The owned objects' colours (E24 M1 T5b1; design §2.9): the bodies of a few classes draw their
/// flag pixels (the runs of index 5) in an entry of a palette by the owner (passes 4 and 7,
/// tile_object_owner 0x40FD20); every other class's are skipped.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerArt {
    /// The palette (`game.pal`, as game_palette16 holds it: 0x4EE430).
    pub palette: String,
    /// The entry of an object without an owner (63); a colour's is the next but the colour (64
    /// Red … 71 Pink).
    pub neutral: u8,
    /// The classes whose flag pixels take the owner's colour (the view passes' jump tables:
    /// 0x41153C, 0x412470's), with the map's classes the game loads as one of them
    /// (object_class_values 0x63A6E4: 219 as 33, 220 as 53).
    pub classes: Vec<u32>,
}

impl OwnerArt {
    /// The palette entry a body of `class` with `owner` (a colour, `None` for no owner) draws its
    /// flag pixels in, `None` when its class's are skipped.
    #[must_use]
    pub fn entry(&self, class: u32, owner: Option<u8>) -> Option<u8> {
        let colour = owner.map_or(0, |c| c.saturating_add(1));
        self.classes
            .contains(&class)
            .then(|| self.neutral.saturating_add(colour))
    }
}

/// A town's picture by its buildings and faction (E24 M1 T5b1; design §2.9): a new game gives
/// every town the picture of its faction in one of three lists (object_template_redraw 0x4C9990,
/// from the towns' setup 0x4CAA70), each by faction (0 Castle … 8 Conflux).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TownArt {
    /// A town without a Fort, Citadel or Castle (0x677A0C).
    pub village: Vec<String>,
    /// A town with a Fort, a Citadel or a Castle but no Capitol (0x677A30).
    pub fort: Vec<String>,
    /// A town with a Capitol (0x677A54).
    pub capitol: Vec<String>,
}

/// The towns' factions.
pub const FACTIONS: usize = 9;

impl TownArt {
    /// Every DEF it names with what it is, in its lists' order.
    #[must_use]
    pub fn defs(&self) -> Vec<(String, String)> {
        [
            ("village", &self.village),
            ("fort", &self.fort),
            ("capitol", &self.capitol),
        ]
        .into_iter()
        .flat_map(|(word, list)| {
            list.iter()
                .enumerate()
                .map(move |(i, name)| (name.clone(), format!("town {word} {i}")))
        })
        .collect()
    }

    /// The picture of a town of `faction` with `built` (its buildings by the game's ids, bit `i`
    /// building `i`: 7 Fort, 8 Citadel, 9 Castle, 13 Capitol), if the faction has one.
    #[must_use]
    pub fn picture(&self, faction: u32, built: u64) -> Option<&str> {
        let has = |id: u32| built >> id & 1 == 1;
        let list = if has(CAPITOL) {
            &self.capitol
        } else if has(FORT) || has(CITADEL) || has(CASTLE) {
            &self.fort
        } else {
            &self.village
        };
        list.get(usize::try_from(faction).unwrap_or(usize::MAX))
            .map(String::as_str)
    }
}

/// The game's ids of the buildings a town's picture depends on (`building_bits` 0x66CD98).
pub const FORT: u32 = 7;
/// The Citadel.
pub const CITADEL: u32 = 8;
/// The Castle.
pub const CASTLE: u32 = 9;
/// The Capitol.
pub const CAPITOL: u32 = 13;

/// The heroes' and boats' pictures (E24 M1 T5a; design §2.3, §2.8): the adventure manager's DEFs
/// by their name tables, and the group each facing draws in all of them.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeroArt {
    /// A hero's map DEF by its class, 0–17 (hero_map_def_names 0x65F5A8).
    pub classes: Vec<String>,
    /// A hero's flag by its owner's colour, 0–7 (hero_flag_def_names 0x65F5F0).
    pub flags: Vec<String>,
    /// A boat by its type, 0–2 (boat_def_names 0x65F670).
    pub boats: Vec<String>,
    /// A boat's waves by its type (boat_wave_def_names 0x65F67C).
    pub waves: Vec<String>,
    /// A boat's flag by its type × 8 + its colour (boat_flag_def_names 0x65F610).
    pub boat_flags: Vec<String>,
    /// The group a facing (0–7) draws (hero_map_group 0x4D9110, boat_map_group 0x4D9160); the
    /// pictures are mirrored above facing 4.
    pub groups: Vec<u32>,
}

/// The hero classes (the map DEFs of [`HeroArt::classes`]).
pub const HERO_CLASSES: usize = 18;
/// The players' colours.
pub const COLOURS: usize = 8;
/// The boats' types.
pub const BOAT_TYPES: usize = 3;
/// The facings.
pub const FACINGS: usize = 8;

impl HeroArt {
    /// Every DEF it names with what it is, in its lists' order.
    #[must_use]
    pub fn defs(&self) -> Vec<(String, String)> {
        let lists = [
            ("hero", &self.classes),
            ("hero flag", &self.flags),
            ("boat", &self.boats),
            ("waves", &self.waves),
            ("boat flag", &self.boat_flags),
        ];
        lists
            .into_iter()
            .flat_map(|(word, list)| {
                list.iter()
                    .enumerate()
                    .map(move |(i, name)| (name.clone(), format!("{word} {i}")))
            })
            .collect()
    }

    /// The groups the facings draw: one past the highest.
    #[must_use]
    pub fn drawn_groups(&self) -> u32 {
        self.groups.iter().max().map_or(0, |g| g + 1)
    }
}

/// The unexplored tiles' pieces (design §1.5).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shroud {
    /// A tile with no explored neighbour: frame ((85 x xor 85 y) div 64) and 3 (`tshrc.def`).
    pub full: String,
    /// A tile beside an explored one: the frame its mask's piece gives (`tshre.def`).
    pub edge: String,
    /// The piece of each mask (the table 0x65F694, 256 entries: a bit per unexplored neighbour,
    /// 1 north, 2 east, 4 south, 8 west, 0x10 north-east, 0x20 south-east, 0x40 south-west, 0x80
    /// north-west): v ≥ 100 is mirrored left to right and v − 100; v of 1 or 5 on an odd x takes
    /// v + 1, v of 3 on an odd y takes 4; the frame is v − 1.
    pub pieces: Vec<u8>,
}

/// The mask of a tile none of whose neighbours is explored (the original's table gives it 0, the
/// full piece; the only mask it gives 0).
pub const ALL_HIDDEN: u8 = 0xFF;

/// Which piece covers an unexplored tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShroudPiece {
    /// The full piece's frame.
    Full(u8),
    /// The edge piece's frame, mirrored left to right or not.
    Edge {
        /// The frame.
        frame: u8,
        /// Mirrored left to right.
        mirror: bool,
    },
    /// The mask's entry gives no frame (100: mirrored, frame −1; none in the original's table).
    None,
}

impl Shroud {
    /// The piece of the unexplored tile (x, y) whose unexplored neighbours make `mask` (0x40F8C0,
    /// 0x412220; design §1.5): the mask's entry, 0 the full piece (0x41237F: the pass draws
    /// `full` when 0x40F8C0 gives 0; E24 M1 T6b, the reviewer's R1-2).
    #[must_use]
    pub fn piece(&self, mask: u8, x: u8, y: u8) -> ShroudPiece {
        let entry = self.pieces.get(usize::from(mask)).copied().unwrap_or(0);
        if entry == 0 {
            let (x, y) = (u32::from(x) * 85, u32::from(y) * 85);
            return ShroudPiece::Full(u8::try_from(((x ^ y) / 64) & 3).unwrap_or(0));
        }
        let (base, mirror) = if entry >= 100 {
            (entry - 100, true)
        } else {
            (entry, false)
        };
        let base = match base {
            1 | 5 if x % 2 == 1 => base + 1,
            3 if y % 2 == 1 => 4,
            _ => base,
        };
        base.checked_sub(1)
            .map_or(ShroudPiece::None, |frame| ShroudPiece::Edge {
                frame,
                mirror,
            })
    }

    /// The frames the edge piece's DEF needs: one past the highest frame any mask gives, on an
    /// even or odd x and y.
    #[must_use]
    pub fn edge_frames(&self) -> usize {
        (0..ALL_HIDDEN)
            .flat_map(|mask| [(0, 0), (1, 0), (0, 1)].map(|(x, y)| self.piece(mask, x, y)))
            .filter_map(|p| match p {
                ShroudPiece::Edge { frame, .. } => Some(usize::from(frame) + 1),
                ShroudPiece::Full(_) | ShroudPiece::None => None,
            })
            .max()
            .unwrap_or(0)
    }
}

/// The frames of the full piece's DEF the view draws (0–3).
pub const FULL_FRAMES: usize = 4;
/// The frames of the edge's DEF the view draws (0–15 further out, 16–19 the corners, 20–35 the
/// edges; design §1.4).
pub const EDGE_FRAMES: usize = 36;
/// The frames of the path's DEF a tile's arrow can name (arrows 1–50; design §2.10).
pub const PATH_FRAMES: usize = 50;
/// The entries of the shroud's table (one per mask).
pub const MASKS: usize = 256;
/// The scroll speeds (the options dialog's three, `option_scroll_speed` 0x69878C 0 to 2).
pub const SCROLL_SPEEDS: usize = 3;
/// The hero speeds (the options dialog's four, `option_walk_speed` 0x69875C 1 to 4).
pub const HERO_SPEEDS: usize = 4;

/// A terrain's, river's or road's art.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayerArt {
    /// The `.h3m`'s code.
    pub code: u8,
    /// The DEF: group 0's frame number *picture* is a tile's (design §1.3).
    pub def: String,
    /// The ranges of the DEF's palette the tick rotates by one each step, first and last entry
    /// (each entry takes its lower neighbour's colour, the first the last's; design §1.6).
    #[serde(default)]
    pub cycles: Vec<[u8; 2]>,
}

/// A selection's tables, merged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tables {
    /// The tables loaded, dependencies first.
    pub order: Vec<String>,
    /// The view's own art.
    pub map: ViewArt,
    /// The terrains, by code.
    pub terrains: BTreeMap<u8, LayerArt>,
    /// The rivers, by code.
    pub rivers: BTreeMap<u8, LayerArt>,
    /// The roads, by code.
    pub roads: BTreeMap<u8, LayerArt>,
}

/// A DEF the tables name, the frames of its group 0 the view draws and the groups it draws.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefNeed {
    /// The entry's name, as written.
    pub name: String,
    /// What it is (`terrain 8`, `edge`, …).
    pub what: String,
    /// The layer and code it draws, for the terrains, rivers and roads (whose frames a map's
    /// pictures decide).
    pub layer: Option<(Layer, u8)>,
    /// The frames group 0 must hold at least.
    pub frames: usize,
    /// The groups the view draws (ids 0 to this, exclusive; 1 but the heroes' and boats'), each
    /// with group 0's frames at least: a hero's picture is its facing's group at a frame counted
    /// by group 0's frames (design §2.8).
    pub groups: u32,
}

impl Tables {
    /// The layer's art by code.
    #[must_use]
    pub fn layer(&self, layer: Layer) -> &BTreeMap<u8, LayerArt> {
        match layer {
            Layer::Terrain => &self.terrains,
            Layer::River => &self.rivers,
            Layer::Road => &self.roads,
        }
    }

    /// Every DEF the tables name with the frames the view draws of it: the layers' at least one
    /// (a map's pictures decide the rest), the edge's, the shroud's pieces', the path's arrows.
    #[must_use]
    pub fn defs(&self) -> Vec<DefNeed> {
        let mut needs = Vec::new();
        for (layer, word) in [
            (Layer::Terrain, "terrain"),
            (Layer::River, "river"),
            (Layer::Road, "road"),
        ] {
            for (code, art) in self.layer(layer) {
                needs.push(DefNeed {
                    name: art.def.clone(),
                    what: format!("{word} {code}"),
                    layer: Some((layer, *code)),
                    frames: 1,
                    groups: 1,
                });
            }
        }
        let map = &self.map;
        for (name, what, frames) in [
            (&map.edge, "edge", EDGE_FRAMES),
            (&map.shroud.full, "full shroud", FULL_FRAMES),
            (&map.shroud.edge, "edge shroud", map.shroud.edge_frames()),
            (&map.path, "path", PATH_FRAMES),
        ] {
            needs.push(DefNeed {
                name: name.clone(),
                what: what.to_string(),
                layer: None,
                frames,
                groups: 1,
            });
        }
        // The facings' standing groups and a step's walking ones (E24 M2 T3b): the lifted hero
        // draws a walking group's frames 0 to the walk's frames less one, its flag too.
        let walk_frames = usize::try_from(map.walk.frames).unwrap_or(usize::MAX);
        let groups = map.heroes.drawn_groups().max(map.walk.drawn_groups());
        for (name, what) in map.heroes.defs() {
            needs.push(DefNeed {
                name,
                what,
                layer: None,
                frames: walk_frames.max(1),
                groups,
            });
        }
        for (name, what) in map.towns.defs() {
            needs.push(DefNeed {
                name,
                what,
                layer: None,
                frames: 1,
                groups: 1,
            });
        }
        needs
    }

    /// Every archive entry name the tables mention, upper-cased, each once.
    #[must_use]
    pub fn names(&self) -> BTreeSet<String> {
        self.defs()
            .into_iter()
            .map(|n| n.name.to_ascii_uppercase())
            .collect()
    }
}

/// Why tables do not load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableError {
    /// A file is not valid TOML or not in the format (the parser's message).
    Syntax(String),
    /// Two files have the same id.
    DuplicateTable(String),
    /// The selection or a dependency names no file.
    UnknownTable(String),
    /// The dependencies go round through this table.
    Cycle(String),
    /// No table of the selection holds `[map]`, or several do.
    Map(usize),
    /// A layer's code is defined twice (`river 2`).
    DuplicateCode(String),
    /// An entry breaks a rule: what it is and the rule.
    Entry(String, String),
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(m) => write!(f, "not a table: {m}"),
            Self::DuplicateTable(id) => write!(f, "two tables have the id `{id}`"),
            Self::UnknownTable(id) => write!(f, "no table has the id `{id}`"),
            Self::Cycle(id) => write!(f, "the dependencies go round through `{id}`"),
            Self::Map(n) => write!(f, "{n} tables of the selection hold [map], not one"),
            Self::DuplicateCode(what) => write!(f, "`{what}` is defined twice"),
            Self::Entry(what, rule) => write!(f, "`{what}`: {rule}"),
        }
    }
}

/// Parses one table file.
///
/// # Errors
/// [`TableError::Syntax`]: not TOML, an unknown section or field, a missing or mistyped one.
pub fn parse(text: &str) -> Result<TableFile, TableError> {
    toml::from_str(text).map_err(|e| TableError::Syntax(e.to_string()))
}

/// Loads the selection `top` (a table id) and its dependencies from `files` into one [`Tables`].
///
/// # Errors
/// [`TableError`]: two files with one id, an unknown or circular dependency, `[map]` in no table
/// of the selection or in several, a code defined twice, a river or road of code 0, a name
/// without its extension, a cycle's range backwards, a shroud table not of 256 entries, a
/// heroes' or boats' list of another length than its classes, colours, types or facings.
pub fn load(files: &[TableFile], top: &str) -> Result<Tables, TableError> {
    let mut by_id = BTreeMap::new();
    for file in files {
        if by_id.insert(file.table.id.as_str(), file).is_some() {
            return Err(TableError::DuplicateTable(file.table.id.clone()));
        }
    }
    let mut order = Vec::new();
    visit(&by_id, top, &mut Vec::new(), &mut order)?;
    let selected: Vec<&TableFile> = order
        .iter()
        .filter_map(|id| by_id.get(id))
        .copied()
        .collect();
    let maps: Vec<&ViewArt> = selected.iter().filter_map(|f| f.map.as_ref()).collect();
    let [map] = maps[..] else {
        return Err(TableError::Map(maps.len()));
    };
    name("edge", &map.edge, ".DEF")?;
    name("full shroud", &map.shroud.full, ".DEF")?;
    name("edge shroud", &map.shroud.edge, ".DEF")?;
    name("path", &map.path, ".DEF")?;
    heroes(&map.heroes)?;
    owners_and_towns(map)?;
    if map.scroll_steps.len() != SCROLL_SPEEDS || map.scroll_steps.iter().any(|&s| s < 1) {
        return Err(entry(
            "scroll_steps",
            &format!("one step of at least a tile per scroll speed ({SCROLL_SPEEDS})"),
        ));
    }
    if map.poll_ms == 0 {
        return Err(entry("poll_ms", "a poll every 0 ms"));
    }
    walk(&map.walk)?;
    if map.shroud.pieces.len() != MASKS {
        return Err(entry(
            "shroud",
            &format!(
                "{} pieces, not one per mask ({MASKS})",
                map.shroud.pieces.len()
            ),
        ));
    }
    let mut tables = Tables {
        order: order.iter().map(|id| (*id).to_string()).collect(),
        map: map.clone(),
        terrains: BTreeMap::new(),
        rivers: BTreeMap::new(),
        roads: BTreeMap::new(),
    };
    for file in selected {
        // `cycle`: whether the scene cycles the layer's palettes (the terrains' and the rivers',
        // design §1.6); a road's cycles would be ignored (E24 M1 T6b, the reviewer's R1-10).
        for (word, list, into, zero, cycle) in [
            ("terrain", &file.terrain, &mut tables.terrains, true, true),
            ("river", &file.river, &mut tables.rivers, false, true),
            ("road", &file.road, &mut tables.roads, false, false),
        ] {
            for art in list {
                let what = format!("{word} {}", art.code);
                if art.code == 0 && !zero {
                    return Err(entry(&what, "code 0 is none"));
                }
                name(&what, &art.def, ".DEF")?;
                if !cycle && !art.cycles.is_empty() {
                    return Err(entry(&what, "a road's palette does not cycle"));
                }
                if art.cycles.iter().any(|[first, last]| first > last) {
                    return Err(entry(&what, "a cycle's first entry is after its last"));
                }
                if into.insert(art.code, art.clone()).is_some() {
                    return Err(TableError::DuplicateCode(what));
                }
            }
        }
    }
    Ok(tables)
}

/// The heroes' and boats' lists: one name per class, colour, type or type and colour, one group
/// per facing.
fn heroes(art: &HeroArt) -> Result<(), TableError> {
    let lists = [
        ("heroes", art.classes.len(), HERO_CLASSES),
        ("hero flags", art.flags.len(), COLOURS),
        ("boats", art.boats.len(), BOAT_TYPES),
        ("waves", art.waves.len(), BOAT_TYPES),
        ("boat flags", art.boat_flags.len(), BOAT_TYPES * COLOURS),
        ("groups", art.groups.len(), FACINGS),
    ];
    for (what, len, want) in lists {
        if len != want {
            return Err(entry(what, &format!("{len} entries, not {want}")));
        }
    }
    for (def, what) in art.defs() {
        name(&what, &def, ".DEF")?;
    }
    Ok(())
}

/// The walk's numbers: a group per direction, frames and their time, and per hero speed pixels
/// that each sum to a tile's (the same for every speed but the one-draw one).
fn walk(art: &WalkArt) -> Result<(), TableError> {
    if art.groups.len() != FACINGS {
        return Err(entry(
            "walk groups",
            &format!("{} entries, not {FACINGS}", art.groups.len()),
        ));
    }
    if art.frames == 0 || art.frame_ms == 0 {
        return Err(entry("walk", "no frames, or frames of 0 ms"));
    }
    let sums: BTreeSet<i32> = art
        .pixels
        .iter()
        .filter(|p| !p.is_empty())
        .map(|p| p.iter().sum())
        .collect();
    if art.pixels.len() != HERO_SPEEDS
        || sums.len() != 1
        || art.pixels.iter().flatten().any(|&p| p < 1)
    {
        return Err(entry(
            "walk pixels",
            &format!(
                "one list per hero speed ({HERO_SPEEDS}), each of positive pixels summing to one tile, or empty"
            ),
        ));
    }
    Ok(())
}

/// The owners' palette, one town picture per faction in each list.
fn owners_and_towns(map: &ViewArt) -> Result<(), TableError> {
    name("owners' palette", &map.owners.palette, ".PAL")?;
    // The neutral entry and the eight colours' after it lie in the palette's 256 (E24 M1 T6b, the
    // reviewer's R2-6).
    if map.owners.neutral > 255 - 8 {
        return Err(entry(
            "owners",
            &format!(
                "the neutral entry {} leaves no room for 8 colours",
                map.owners.neutral
            ),
        ));
    }
    let lists = [
        ("villages", map.towns.village.len()),
        ("forts", map.towns.fort.len()),
        ("capitols", map.towns.capitol.len()),
    ];
    for (what, len) in lists {
        if len != FACTIONS {
            return Err(entry(what, &format!("{len} entries, not {FACTIONS}")));
        }
    }
    for (def, what) in map.towns.defs() {
        name(&what, &def, ".DEF")?;
    }
    Ok(())
}

fn visit<'a>(
    by_id: &BTreeMap<&'a str, &'a TableFile>,
    id: &str,
    path: &mut Vec<&'a str>,
    order: &mut Vec<&'a str>,
) -> Result<(), TableError> {
    let Some((&key, file)) = by_id.get_key_value(id) else {
        return Err(TableError::UnknownTable(id.to_string()));
    };
    if order.contains(&key) {
        return Ok(());
    }
    if path.contains(&key) {
        return Err(TableError::Cycle(id.to_string()));
    }
    path.push(key);
    for dep in &file.table.depends {
        visit(by_id, dep, path, order)?;
    }
    path.pop();
    order.push(key);
    Ok(())
}

/// A name with its extension (`ext` upper case, with the dot) and something before it.
fn name(what: &str, name: &str, ext: &str) -> Result<(), TableError> {
    let upper = name.to_ascii_uppercase();
    if upper.len() <= ext.len() || !upper.ends_with(ext) {
        return Err(entry(what, &format!("`{name}` is not a {ext} name")));
    }
    Ok(())
}

fn entry(what: &str, rule: &str) -> TableError {
    TableError::Entry(what.to_string(), rule.to_string())
}

#[cfg(test)]
mod tests;
