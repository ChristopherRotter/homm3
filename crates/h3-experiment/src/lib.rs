//! Experiment specs (E04a T2, OPEN-QUESTIONS Q28; `docs/experiments/README.md`).
//!
//! A spec lists *encounters*: a hero of the user's (or a second human player's) next to the
//! opponent it fights, with a procedure and the values to record; or, for the adventure map's
//! experiments (E08 T10a), heroes placed on chosen tiles of a map with rectangles of other
//! terrains, some of them as their hero type makes them. [`build`] compiles a spec into a
//! SoD map (written with [`h3_map::writer`]) and a results template the user fills in. Names
//! resolve against the original tables of the user's installation ([`Tables`]), so a spec reads
//! like the game ("Archangel", "Basic Armorer") and a typo fails at build time instead of in the
//! game. From E08 T16b the map's description and the template's top carry the spec's [`stamp`],
//! so a map left from an earlier build of the spec shows itself before it is played.

pub mod tables;

use std::collections::BTreeMap;
use std::fmt::Write as _;

use h3_map::writer::{
    ARMY_SLOTS, ARTIFACT_SLOTS, Ground, Guards, Hero, HeroArtifacts, Map, MapObject, Monster,
    ObjectTemplate, PLAYERS, Patch, Pile, Plain, Player, Road, RoadKind, Terrain, Town,
    TownBuildings, Underground,
};
use serde::Deserialize;

pub use tables::{RESOURCES, Tables};

/// Heroes a player may have on the map.
pub const MAX_HEROES_PER_PLAYER: usize = 8;
/// Horizontal distance between encounters on the map.
const COLUMN_STEP: u8 = 6;
/// Vertical distance between encounters on the map.
const ROW_STEP: u8 = 5;
/// First encounter column and row.
const ORIGIN: u8 = 3;
/// Slot names of a hero's artifacts, in map slot order.
pub const SLOT_NAMES: [&str; ARTIFACT_SLOTS] = [
    "head",
    "shoulders",
    "neck",
    "right hand",
    "left hand",
    "torso",
    "right ring",
    "left ring",
    "feet",
    "misc 1",
    "misc 2",
    "misc 3",
    "misc 4",
    "war machine 1",
    "war machine 2",
    "war machine 3",
    "war machine 4",
    "spell book",
    "misc 5",
];
/// Faction names of towns (`TOWNTYPE.TXT` order: the town object's subclass).
pub const FACTIONS: [&str; 9] = [
    "castle",
    "rampart",
    "tower",
    "inferno",
    "necropolis",
    "dungeon",
    "stronghold",
    "fortress",
    "conflux",
];
/// A town's buildings by name and their bit in the map (`docs/formats/h3m.md`, "Town
/// buildings"): every faction's. The bit order is VCMI 1.7.5's (`config/gameConfig.json`,
/// `mapFormat.restorationOfErathia.buildingsCommon`; GPL-2.0-or-later, the table ported); the map
/// has no bit for the Village Hall, which every town has.
const BUILDINGS: [(&str, u8); 36] = [
    ("town hall", 0),
    ("city hall", 1),
    ("capitol", 2),
    ("fort", 3),
    ("citadel", 4),
    ("castle", 5),
    ("tavern", 6),
    ("blacksmith", 7),
    ("marketplace", 8),
    ("resource silo", 9),
    ("mage guild 1", 11),
    ("mage guild 2", 12),
    ("mage guild 3", 13),
    ("mage guild 4", 14),
    ("mage guild 5", 15),
    ("shipyard", 16),
    ("grail", 17),
    ("dwelling 1", 22),
    ("upgraded dwelling 1", 23),
    ("horde 1", 24),
    ("dwelling 2", 25),
    ("upgraded dwelling 2", 26),
    ("horde 2", 27),
    ("dwelling 3", 28),
    ("upgraded dwelling 3", 29),
    ("horde 3", 30),
    ("dwelling 4", 31),
    ("upgraded dwelling 4", 32),
    ("horde 4", 33),
    ("dwelling 5", 34),
    ("upgraded dwelling 5", 35),
    ("horde 5", 36),
    ("dwelling 6", 37),
    ("upgraded dwelling 6", 38),
    ("dwelling 7", 39),
    ("upgraded dwelling 7", 40),
];
/// The special buildings by faction and their bits (VCMI 1.7.5, the same file's
/// `mapFormat.restorationOfErathia.buildings`; Conflux's are not listed there).
const SPECIAL_BUILDINGS: [(&str, &str, u8); 27] = [
    ("castle", "lighthouse", 18),
    ("castle", "brotherhood of the sword", 19),
    ("castle", "stables", 20),
    ("rampart", "mystic pond", 18),
    ("rampart", "fountain of fortune", 19),
    ("rampart", "treasury", 20),
    ("tower", "artifact merchants", 10),
    ("tower", "library", 18),
    ("tower", "wall of knowledge", 19),
    ("tower", "lookout tower", 20),
    ("inferno", "brimstone stormclouds", 18),
    ("inferno", "castle gate", 19),
    ("inferno", "order of fire", 20),
    ("necropolis", "cover of darkness", 18),
    ("necropolis", "necromancy amplifier", 19),
    ("necropolis", "skeleton transformer", 20),
    ("dungeon", "artifact merchants", 10),
    ("dungeon", "mana vortex", 18),
    ("dungeon", "portal of summoning", 19),
    ("dungeon", "battle scholar academy", 20),
    ("stronghold", "escape tunnel", 18),
    ("stronghold", "freelancers guild", 19),
    ("stronghold", "ballista yard", 20),
    ("stronghold", "hall of valhalla", 21),
    ("fortress", "glyphs of fear", 18),
    ("fortress", "blood obelisk", 19),
    ("fortress", "cage of warlords", 20),
];
/// Skill level names, 1-based.
const LEVELS: [&str; 3] = ["Basic", "Advanced", "Expert"];
/// Spells a map's hero may have in its spell book (bit per spell id in the map; `SPTRAITS.TXT`
/// ids below this).
pub const MAP_SPELLS: usize = 70;
/// The slot of the Spell Book in [`SLOT_NAMES`].
const SPELL_BOOK_SLOT: &str = "spell book";
/// A wandering monster's dispositions, the map's value in order (`docs/formats/h3m.md`, the
/// monster body; E08 T22a1).
pub const DISPOSITIONS: [&str; 5] = ["compliant", "friendly", "aggressive", "hostile", "savage"];
/// The game's difficulties as the scenario list names them, Easy (0) to Impossible (4) (E08
/// T22a1: the difficulty a batch is played on).
pub const DIFFICULTIES: [&str; 5] = ["easy", "normal", "hard", "expert", "impossible"];

/// A whole spec file.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    /// The map.
    pub map: MapSpec,
    /// The encounters, in the order they are laid out and listed.
    #[serde(default)]
    pub encounter: Vec<Encounter>,
    /// The spec text's [`stamp`], set by [`parse`] (never read from the file).
    #[serde(skip)]
    pub stamp: String,
}

/// Map-wide settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapSpec {
    /// Name in the scenario list.
    pub name: String,
    /// Description in the scenario list.
    pub description: String,
    /// Width and height (36, 72, 108, 144).
    #[serde(default = "default_size")]
    pub size: u8,
    /// Terrain of the whole map (also the battlefield): dirt, sand, grass, snow, swamp, rough,
    /// subterranean, lava.
    pub terrain: String,
    /// General instructions at the top of the template (how to start the map, saving, …).
    #[serde(default)]
    pub instructions: Vec<String>,
    /// Rectangles of other terrains (`[[map.patch]]`, E08 T10a); a later one covers an earlier.
    #[serde(default)]
    pub patch: Vec<PatchSpec>,
    /// Straight roads from west to east (`[[map.road]]`, E08 T10a).
    #[serde(default)]
    pub road: Vec<RoadSpec>,
    /// Rows of resource piles (`[[map.pile]]`, E08 T16b).
    #[serde(default)]
    pub pile: Vec<PileSpec>,
    /// Special grounds (`[[map.ground]]`, E04c T3): Cursed Ground, Magic Plains, Clover Field, ….
    #[serde(default)]
    pub ground: Vec<GroundSpec>,
    /// The difficulty the batch is played on ([`DIFFICULTIES`]; E08 T22a1): the template says
    /// to choose it in the scenario list (the map does not hold it). None: the template is silent.
    #[serde(default)]
    pub difficulty: Option<String>,
    /// The underground level (`[map.underground]`, E08 T22a1); none: the map has one level.
    #[serde(default)]
    pub underground: Option<UndergroundSpec>,
    /// Objects without a body (`[[map.object]]`, E08 T32a): the Cartographers; since E08 T36a
    /// the trees and the Border Gates.
    #[serde(default)]
    pub object: Vec<ObjectSpec>,
    /// The scenario's level limit (E08 T32a; 0, the default: none).
    #[serde(default)]
    pub level_limit: u8,
    /// The secondary skills the map bans, by `SSTRAITS.TXT` name (E08 T36b; none, the default):
    /// no level-up offers one the hero does not know (batch E5e's forced offers).
    #[serde(default)]
    pub banned_skills: Vec<String>,
    /// Towns that are no encounter's opponent (`[[map.town]]`, E23 T7): a player's own town
    /// elsewhere on the map (batch 4b: the Tavern the computer's retreat test asks for).
    #[serde(default)]
    pub town: Vec<MapTownSpec>,
}

/// A town of the map that no encounter fights (E23 T7): its entrance tile and the town as an
/// opponent's ([`TownSpec`]), without a visiting hero.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapTownSpec {
    /// The entrance tile (the picture covers 5 columns west of it, 2 east and 5 rows north).
    pub entrance: [u8; 2],
    /// Faction ([`FACTIONS`]).
    pub faction: String,
    /// "blue" (the default), "tan" or "neutral".
    #[serde(default = "blue")]
    pub owner: String,
    /// Built buildings by name, as a [`TownSpec`]'s.
    #[serde(default)]
    pub buildings: Option<Vec<String>>,
    /// Garrison as (creature, count), at most seven stacks.
    #[serde(default)]
    pub garrison: Vec<(String, u16)>,
}

/// The underground level: its own terrain and rectangles; encounters and piles with `level = 1`
/// stand on it (no roads or grounds there).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UndergroundSpec {
    /// Terrain of the level, as [`MapSpec::terrain`].
    pub terrain: String,
    /// Rectangles of other terrains (`[[map.underground.patch]]`); a later one covers an earlier.
    #[serde(default)]
    pub patch: Vec<PatchSpec>,
}

/// A special ground: one of its pictures, placed with its bottom-right tile on `at`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundSpec {
    /// The picture's name in `Objects.txt` (`AVXcf0` or `AVXcf0.def`, any case), which names the
    /// class (Clover Field 222, the SoD Cursed Ground 223, …).
    pub picture: String,
    /// The object's position, `[column, row]`: the picture's bottom-right tile. The picture covers
    /// its width to the left and its height upwards (its `.msk`: the `0` pictures 6 x 4, the SoD
    /// pictures `AVXcg5` and `AVXmp5` 3 x 3).
    pub at: [u8; 2],
}

/// An object without a body (E08 T32a, T36a): one of its pictures, placed with its bottom-right
/// tile on `at`, on the surface. No hero, monster, pile or town may stand on a tile it blocks or
/// is visited from: a hero comes to it in play.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectSpec {
    /// The picture's name in `Objects.txt` (`AVXmaps0` or `AVXmaps0.def`, any case), which names
    /// the class ([`h3_map::writer::PLAIN_OBJECTS`]: the Cartographers, water `AVXmapw0`, land
    /// `AVXmaps0`, subterranean `AVXmapu0`; the trees, `AVLtr3d0` one tile among them; the Border
    /// Gates `AVXbgt00` light blue … `AVXbgt70` black).
    pub picture: String,
    /// The object's position, `[column, row]`: the picture's bottom-right tile (the land
    /// Cartographer covers it and the tile west of it, its entrance; a Border Gate it and the two
    /// west of it, the middle one its entrance).
    pub at: [u8; 2],
}

/// Resource piles of one resource on one row, one per tile from `from` to `to`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PileSpec {
    /// The resource ([`RESOURCES`]: wood, mercury, ore, sulfur, crystal, gems, gold).
    pub resource: String,
    /// Row.
    pub row: u8,
    /// West end's column.
    pub from: u8,
    /// East end's column (inclusive; `from` for a single pile).
    pub to: u8,
    /// The amount as the map stores it; 0 (the default) is drawn when the game starts.
    #[serde(default)]
    pub amount: u32,
    /// The level: 0 the surface (the default), 1 the underground (E08 T22a1).
    #[serde(default)]
    pub level: u8,
    /// Guards of every pile of the row as (creature, count), at most seven stacks (E08 T22a1;
    /// ADV-PIK-004); none: unguarded.
    #[serde(default)]
    pub guards: Vec<(String, u16)>,
    /// The question the game shows before the guards' battle (the map's message; needs guards).
    /// None: the guards fight at once.
    #[serde(default)]
    pub question: Option<String>,
}

/// A road on one row.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoadSpec {
    /// "dirt", "gravel" or "cobblestone".
    pub kind: String,
    /// Row.
    pub row: u8,
    /// West end's column.
    pub from: u8,
    /// East end's column, east of `from`.
    pub to: u8,
}

/// A rectangle of the map with a terrain of its own.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchSpec {
    /// Terrain, as in [`MapSpec::terrain`].
    pub terrain: String,
    /// Top left tile, `[column, row]`.
    pub from: [u8; 2],
    /// Bottom right tile, inclusive.
    pub to: [u8; 2],
}

fn default_size() -> u8 {
    36
}

/// One staged battle (or an inspection without a battle).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Encounter {
    /// Short id, also the hero's name in the game unless the hero has its own.
    pub id: String,
    /// What it settles.
    pub title: String,
    /// Question and dispute ids (OPEN-QUESTIONS, surveys).
    #[serde(default)]
    pub questions: Vec<String>,
    /// The hero the user plays.
    pub hero: HeroSpec,
    /// What the hero fights (none for an inspection).
    #[serde(default)]
    pub opponent: Option<Opponent>,
    /// The level of the encounter's heroes and opponent: 0 the surface (the default), 1 the
    /// underground (E08 T22a1; needs [`MapSpec::underground`]).
    #[serde(default)]
    pub level: u8,
    /// More heroes of the encounter, each on its own tile (`at`); their names are their hero
    /// types' unless given (E08 T10a: inspections of many heroes).
    #[serde(default)]
    pub also: Vec<HeroSpec>,
    /// Steps for the user, in order.
    #[serde(default)]
    pub procedure: Vec<String>,
    /// What the user writes down.
    #[serde(default)]
    pub record: Vec<Record>,
    /// Expected values per hypothesis; for the agent who evaluates the results, never shown in
    /// the template.
    #[serde(default)]
    pub predictions: Option<String>,
}

/// A hero on the map.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeroSpec {
    /// Hero type by name (or id, for names that occur twice).
    #[serde(rename = "type")]
    pub hero_type: NameOrId,
    /// The specialty the spec expects, as [`Tables::specialty_text`] words it; checked so that a
    /// hero with an interfering specialty is not picked by mistake.
    pub specialty: String,
    /// "red" or "blue".
    #[serde(default = "red")]
    pub owner: String,
    /// The hero's tile, `[column, row]` (default: the encounter's place in the grid, one column
    /// further east for the opponent; required for [`Encounter::also`]).
    #[serde(default)]
    pub at: Option<[u8; 2]>,
    /// Everything but owner, tile and name is the hero type's own, as the map editor places a
    /// hero (E08 T10a): army, experience, primary and secondary skills, artifacts and spells.
    /// The spec then lists none of them.
    #[serde(default)]
    pub defaults: bool,
    /// Name in the game (default: the encounter id).
    #[serde(default)]
    pub name: Option<String>,
    /// Experience (0 = level 1).
    #[serde(default)]
    pub experience: u32,
    /// Attack, defense, power, knowledge (required unless [`HeroSpec::defaults`]).
    #[serde(default)]
    pub primary: Option<[u8; 4]>,
    /// Secondary skills as (name, level 1–3); replaces the hero type's.
    #[serde(default)]
    pub skills: Vec<(String, u8)>,
    /// Army as (creature, count), one to seven stacks (none with [`HeroSpec::defaults`]).
    #[serde(default)]
    pub army: Vec<(String, u16)>,
    /// Worn artifacts by slot name ([`SLOT_NAMES`]); replaces the hero type's (so no spell book
    /// unless listed).
    #[serde(default)]
    pub artifacts: BTreeMap<String, String>,
    /// Artifacts in the backpack by name, not worn (E04b batch 2f: a Spell Book carried but not
    /// worn).
    #[serde(default)]
    pub backpack: Vec<String>,
    /// Spells in the spell book by `SPTRAITS.TXT` name; replaces the hero type's and needs a
    /// worn `spell book`. The hero starts with full mana (10 × knowledge without Intelligence).
    #[serde(default)]
    pub spells: Vec<String>,
    /// Tight formation.
    #[serde(default)]
    pub tight: bool,
}

fn red() -> String {
    "red".into()
}

/// A hero type by name or by id.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum NameOrId {
    /// `HOTRAITS.TXT` name.
    Name(String),
    /// Hero type id.
    Id(u8),
}

/// The opponent: a wandering monster, a hero of the second player, or a town (E04c T5b) with or
/// without a visiting hero.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Opponent {
    /// Creature of a wandering monster.
    #[serde(default)]
    pub monster: Option<String>,
    /// Number of creatures of the monster.
    #[serde(default)]
    pub count: Option<u16>,
    /// The monster's disposition ([`DISPOSITIONS`]; E08 T22a1); none: savage, it never joins.
    #[serde(default)]
    pub disposition: Option<String>,
    /// The monster may flee from a much stronger hero (E08 T22a1); false (the default): it never
    /// flees.
    #[serde(default)]
    pub flees: bool,
    /// A hero (owner "blue" by default for opponents); with a town, its visiting hero, on the
    /// town's entrance.
    #[serde(default)]
    pub hero: Option<HeroSpec>,
    /// A town whose entrance is the tile north of the encounter's hero (E04c T5b: sieges).
    #[serde(default)]
    pub town: Option<TownSpec>,
}

/// A town of the opponent's (E04c T5b). Its picture covers 8 columns and 6 rows up to two columns
/// east of its entrance and the entrance's row; its blocked tiles are the 5 columns around the
/// entrance on the entrance's row and the row above, and 3 on the row above those.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TownSpec {
    /// Faction ([`FACTIONS`]: castle, rampart, …, conflux).
    pub faction: String,
    /// "blue" (the default) or "neutral" (no visiting hero then).
    #[serde(default = "blue")]
    pub owner: String,
    /// Built buildings by name ([`BUILDINGS`], the faction's special buildings); the Village Hall
    /// is always built. Absent: the editor's standard set without a fort.
    #[serde(default)]
    pub buildings: Option<Vec<String>>,
    /// Garrison as (creature, count), at most seven stacks; empty: the editor's default.
    #[serde(default)]
    pub garrison: Vec<(String, u16)>,
    /// The garrison's tight formation.
    #[serde(default)]
    pub tight: bool,
}

fn blue() -> String {
    "blue".into()
}

/// Something to write down: a single value or a table.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    /// A single value ("Ballista shots in round 1").
    #[serde(default)]
    pub field: Option<String>,
    /// Column headings of a table.
    #[serde(default)]
    pub table: Option<Vec<String>>,
    /// Empty rows of the table.
    #[serde(default)]
    pub rows: Option<u8>,
    /// Rows with their first column filled in, one per label, after the empty ones (E08 T10a:
    /// a row per hero).
    #[serde(default)]
    pub labels: Vec<String>,
}

/// Parses a spec and sets its [`stamp`].
///
/// # Errors
/// TOML syntax or unknown fields.
pub fn parse(text: &str) -> Result<Spec, String> {
    let mut spec: Spec = toml::from_str(text).map_err(|e| e.to_string())?;
    spec.stamp = stamp(text);
    Ok(spec)
}

/// The spec's stamp (E08 decision 18): 8 hex digits of the CRC-32 of its text with Windows line
/// endings made Unix ones, so a checkout with either gives the same stamp. [`build`] puts it at
/// the start of the map's description (shown in the game's scenario list) and at the top of the
/// results template: a map from another build of the spec shows a different stamp.
pub fn stamp(text: &str) -> String {
    format!(
        "{:08x}",
        h3_map::crc32(text.replace("\r\n", "\n").as_bytes())
    )
}

/// Compiles a spec into a map and its results template. `map_file` is the map's file name as the
/// template mentions it.
///
/// # Errors
/// Unknown names, values outside the game's ranges, too many encounters or heroes.
pub fn build(spec: &Spec, tables: &Tables, map_file: &str) -> Result<(Map, String), String> {
    build_only(spec, tables, map_file, None)
}

/// [`build`], with only the encounter `only` on the map when given (the others still checked and
/// in the template, their places left empty): the map for one battle, which the capture harness
/// starts with its only hero (`docs/experiments/README.md`, "Agents running a batch").
///
/// # Errors
/// As [`build`]; and an `only` that names no encounter.
pub fn build_only(
    spec: &Spec,
    tables: &Tables,
    map_file: &str,
    only: Option<&str>,
) -> Result<(Map, String), String> {
    if let Some(id) = only.filter(|id| !spec.encounter.iter().any(|e| e.id == *id)) {
        return Err(format!("no encounter `{id}`"));
    }
    let size = spec.map.size;
    if ![36, 72, 108, 144].contains(&size) {
        return Err(format!("map size {size} (36, 72, 108 or 144)"));
    }
    let base = terrain(&spec.map.terrain)?;
    let patches = level_patches(&spec.map.patch, size)?;
    let below = spec
        .map
        .underground
        .as_ref()
        .map(|u| Ok::<_, String>((terrain(&u.terrain)?, level_patches(&u.patch, size)?)))
        .transpose()?;
    let levels = 1 + u8::from(below.is_some());
    let difficulty = spec
        .map
        .difficulty
        .as_deref()
        .map(|d| {
            DIFFICULTIES
                .iter()
                .position(|n| n.eq_ignore_ascii_case(d.trim()))
                .map(|i| capitalized(DIFFICULTIES[i]))
                .ok_or_else(|| format!("unknown difficulty `{d}`"))
        })
        .transpose()?;
    let roads = spec
        .map
        .road
        .iter()
        .map(|r| {
            let kind = match r.kind.as_str() {
                "dirt" => RoadKind::Dirt,
                "gravel" => RoadKind::Gravel,
                "cobblestone" => RoadKind::Cobblestone,
                other => return Err(format!("unknown road `{other}`")),
            };
            if r.from >= r.to || r.to >= size || r.row >= size {
                return Err(format!(
                    "road on row {} from {} to {}: one tile or less, or off the map",
                    r.row, r.from, r.to
                ));
            }
            Ok(Road {
                kind,
                row: r.row,
                from: r.from,
                to: r.to,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let columns = (size - 2 * ORIGIN) / COLUMN_STEP + 1;
    let capacity = usize::from(columns) * usize::from((size - 2 * ORIGIN) / ROW_STEP + 1);
    if spec.encounter.len() > capacity {
        return Err(format!(
            "{} encounters, a {size} map holds {capacity}",
            spec.encounter.len()
        ));
    }
    // The grounds first: the objects after them are drawn above them.
    let mut objects = spec
        .map
        .ground
        .iter()
        .map(|g| Ok((0, ground(g, tables, size)?)))
        .collect::<Result<Vec<_>, String>>()?;
    for o in &spec.map.object {
        objects.push((0, plain(o, tables, size)?));
    }
    let mut used_types = Vec::new();
    let mut heroes_per_player = [0usize; PLAYERS];
    let mut towns_per_player = [0usize; PLAYERS];
    let mut text = String::new();
    let _ = writeln!(text, "# {} — results\n", spec.map.name);
    let _ = writeln!(
        text,
        "Map file: `{map_file}`. Fill in the blanks below and keep this file; an agent records the \
         results (`docs/experiments/README.md`).\n"
    );
    let _ = writeln!(
        text,
        "**Stamp `{}`.** The map's description in the scenario list starts with \"Stamp {}.\": \
         compare the two before you play. If they differ, the map is not this template's build: \
         do not play it, ask for the map again.\n",
        spec.stamp, spec.stamp
    );
    if let Some(difficulty) = &difficulty {
        let _ = writeln!(
            text,
            "**Difficulty: {difficulty}.** Choose it in the scenario list before you start.\n"
        );
    }
    for (i, step) in spec.map.instructions.iter().enumerate() {
        let _ = writeln!(text, "{}. {step}", i + 1);
    }
    for (index, enc) in spec.encounter.iter().enumerate() {
        let index = u8::try_from(index).map_err(|_| "too many encounters".to_owned())?;
        let first_object = objects.len();
        let mut placed_town = None;
        let x = ORIGIN + (index % columns) * COLUMN_STEP;
        let y = ORIGIN + (index / columns) * ROW_STEP;
        let ctx = |e: String| format!("encounter {}: {e}", enc.id);
        let level = enc.level;
        if level >= levels {
            return Err(ctx(format!(
                "level {level}: no such level (`[map.underground]` adds level 1)"
            )));
        }
        let mut add_hero = |h: &HeroSpec, tile: (u8, u8), name: Option<&str>, owner: &str| {
            let tile = h.at.map_or(tile, |[x, y]| (x, y));
            let hero = hero(h, tables, tile, name, owner).map_err(ctx)?;
            if used_types.contains(&hero.hero_type) {
                return Err(ctx(format!(
                    "hero type {} is used twice",
                    tables.heroes[usize::from(hero.hero_type)]
                )));
            }
            used_types.push(hero.hero_type);
            let n = &mut heroes_per_player[usize::from(hero.owner)];
            *n += 1;
            if *n > MAX_HEROES_PER_PLAYER {
                return Err(ctx(format!(
                    "more than {MAX_HEROES_PER_PLAYER} heroes for one player"
                )));
            }
            let summary = hero_summary(h, &hero, tables);
            objects.push((level, MapObject::Hero(hero)));
            Ok(summary)
        };
        let own = add_hero(&enc.hero, (x, y), Some(&enc.id), "red")?;
        let (x, y) = enc.hero.at.map_or((x, y), |[x, y]| (x, y));
        // The encounter's other heroes before its opponent, so that the opponent's arm is the
        // closure's last use (a monster is pushed beside it).
        let mut more = Vec::new();
        for h in &enc.also {
            if h.at.is_none() {
                return Err(ctx("a hero under `also` needs `at`".into()));
            }
            more.push(add_hero(h, (x, y), h.name.as_deref(), "red")?);
        }
        let against = match &enc.opponent {
            None => None,
            Some(Opponent {
                monster: Some(creature),
                count: Some(count),
                disposition,
                flees,
                hero: None,
                town: None,
            }) => {
                let id = lookup(&tables.creatures, creature, "creature").map_err(ctx)?;
                let template = tables
                    .monsters
                    .iter()
                    .find(|(c, _)| *c == id as u32)
                    .map(|(_, t)| t.clone())
                    .ok_or_else(|| ctx(format!("no wandering-monster template for {creature}")))?;
                if *count == 0 {
                    return Err(ctx("a monster needs at least one creature".into()));
                }
                let disposition = disposition
                    .as_deref()
                    .map(|d| {
                        DISPOSITIONS
                            .iter()
                            .position(|n| n.eq_ignore_ascii_case(d.trim()))
                            .ok_or_else(|| ctx(format!("unknown disposition `{d}`")))
                    })
                    .transpose()?;
                let tile = (x.saturating_add(1), y);
                // Savage unless given: it never joins.
                let kind = disposition.map_or(4, |d| d as u8);
                objects.push((level, monster(template, tile, *count, kind, *flees)));
                let named = disposition.map_or(String::new(), |d| format!(", {}", DISPOSITIONS[d]));
                let flight = if *flees { "may flee" } else { "never flees" };
                Some(format!(
                    "{count} {creature} (wandering monster{named}, {flight})"
                ))
            }
            Some(Opponent {
                monster: None,
                count: None,
                disposition: None,
                flees: false,
                hero: Some(h),
                town: None,
            }) => Some(add_hero(
                h,
                (x.saturating_add(1), y),
                Some(&enc.id),
                "blue",
            )?),
            Some(Opponent {
                monster: None,
                count: None,
                disposition: None,
                flees: false,
                hero,
                town: Some(t),
            }) => {
                let entrance = (x, y.saturating_sub(1));
                let (town, mut summary) = town_object(t, tables, entrance, size).map_err(ctx)?;
                if let Some(owner) = town.owner {
                    towns_per_player[usize::from(owner)] += 1;
                }
                let neutral = town.owner.is_none();
                let tan = town.owner == Some(2);
                placed_town = Some(town);
                if let Some(h) = hero {
                    if neutral {
                        return Err(ctx("a neutral town has no visiting hero".into()));
                    }
                    if h.at.is_some() {
                        return Err(ctx("a town's hero stands on its entrance: no `at`".into()));
                    }
                    if tan != (h.owner == "tan") {
                        return Err(ctx("a town's visiting hero is its owner's".into()));
                    }
                    let visiting = add_hero(h, entrance, Some(&enc.id), "blue")?;
                    summary = format!("{summary}; visiting hero: {visiting}");
                }
                Some(summary)
            }
            Some(_) => {
                return Err(ctx(
                    "an opponent is `monster` + `count`, `hero`, or `town` with or without `hero` \
                     (`disposition` and `flees` with a monster only)"
                        .into(),
                ));
            }
        };
        // The town before its visiting hero, the encounter's last object.
        if let Some(town) = placed_town {
            let visiting = enc.opponent.as_ref().is_some_and(|o| o.hero.is_some());
            objects.insert(
                objects.len() - usize::from(visiting),
                (level, MapObject::Town(town)),
            );
        }
        encounter_text(&mut text, enc, &own, against.as_deref(), &more);
        if only.is_some_and(|id| id != enc.id) {
            objects.truncate(first_object);
        }
    }
    for p in &spec.map.pile {
        if p.level >= levels {
            return Err(format!(
                "piles on row {} at level {}: no such level (`[map.underground]` adds level 1)",
                p.row, p.level
            ));
        }
        objects.extend(piles(p, tables, size)?.into_iter().map(|o| (p.level, o)));
    }
    for t in &spec.map.town {
        let [x, y] = t.entrance;
        let as_opponent = TownSpec {
            faction: t.faction.clone(),
            owner: t.owner.clone(),
            buildings: t.buildings.clone(),
            garrison: t.garrison.clone(),
            tight: false,
        };
        let (town, _) = town_object(&as_opponent, tables, (x, y), size)
            .map_err(|e| format!("the town at {:?}: {e}", t.entrance))?;
        if let Some(owner) = town.owner {
            towns_per_player[usize::from(owner)] += 1;
        }
        objects.push((0, MapObject::Town(town)));
    }
    // Heroes and monsters stand on grounds; two grounds may overlap (the game takes the last). A
    // plain object counts with every tile it covers.
    let mut tiles: Vec<(u8, (u8, u8))> = objects
        .iter()
        .flat_map(|(level, o)| match o {
            MapObject::Hero(h) => vec![(*level, h.tile)],
            MapObject::Monster(m) => vec![(*level, m.tile)],
            MapObject::Pile(p) => vec![(*level, p.tile)],
            MapObject::Town(t) => vec![(*level, t.tile)],
            MapObject::Plain(p) => covered(&p.template, p.tile)
                .into_iter()
                .map(|tile| (*level, tile))
                .collect(),
            MapObject::Reward(r) => covered(&r.template, r.tile)
                .into_iter()
                .map(|tile| (*level, tile))
                .collect(),
            MapObject::Ground(_) => vec![],
        })
        .collect();
    tiles.sort_unstable();
    if let Some(w) = tiles.windows(2).find(|w| w[0] == w[1]) {
        let below = if w[0].0 == 1 { " underground" } else { "" };
        return Err(format!("two objects on the tile {:?}{below}", w[0].1));
    }
    let mut players: [Option<Player>; PLAYERS] = Default::default();
    // Both players may be human, so the map can be played in hot seat; the allowed towns are the
    // smoke map's (proven in the game).
    players[0] = Some(Player {
        human: true,
        computer: true,
        towns: 1,
    });
    if heroes_per_player[1] > 0 || towns_per_player[1] > 0 {
        players[1] = Some(Player {
            human: true,
            computer: true,
            towns: 1 << 6,
        });
    }
    // Tan, the computer's only (E23 T7, batch 4b): hot seat makes the named players human, Red
    // and Blue, and leaves Tan to the computer.
    if heroes_per_player[2] > 0 || towns_per_player[2] > 0 {
        players[2] = Some(Player {
            human: false,
            computer: true,
            towns: 1 << 7,
        });
    }
    if heroes_per_player.iter().skip(3).any(|&n| n > 0) {
        return Err("owners are red, blue or tan".into());
    }
    let banned_skills = spec
        .map
        .banned_skills
        .iter()
        .map(|name| Ok(lookup(&tables.skills, name, "skill")? as u8))
        .collect::<Result<Vec<u8>, String>>()?;
    let map = Map {
        name: spec.map.name.clone(),
        description: format!("Stamp {}. {}", spec.stamp, spec.map.description),
        size,
        terrain: base,
        patches,
        roads,
        players,
        objects: on_level(&mut objects, 0),
        underground: below.map(|(terrain, patches)| Underground {
            terrain,
            patches,
            objects: on_level(&mut objects, 1),
        }),
        level_limit: spec.map.level_limit,
        banned_skills,
    };
    Ok((map, text))
}

/// The objects of `level`, taken out of `objects` in their order.
fn on_level(objects: &mut Vec<(u8, MapObject)>, level: u8) -> Vec<MapObject> {
    let (here, rest) = std::mem::take(objects)
        .into_iter()
        .partition::<Vec<_>, _>(|(l, _)| *l == level);
    *objects = rest;
    here.into_iter().map(|(_, o)| o).collect()
}

/// Rectangles of other terrains on one level.
fn level_patches(specs: &[PatchSpec], size: u8) -> Result<Vec<Patch>, String> {
    specs
        .iter()
        .map(|p| {
            let [fx, fy] = p.from;
            let [tx, ty] = p.to;
            if fx > tx || fy > ty || tx >= size || ty >= size {
                return Err(format!(
                    "patch from {:?} to {:?}: corners out of order or off the map",
                    p.from, p.to
                ));
            }
            Ok(Patch {
                terrain: patch_terrain(&p.terrain)?,
                from: (fx, fy),
                to: (tx, ty),
            })
        })
        .collect()
}

/// A name with its first letter upper case.
fn capitalized(name: &str) -> String {
    let mut name = name.to_owned();
    name[..1].make_ascii_uppercase();
    name
}

/// A row of piles, west to east.
fn piles(p: &PileSpec, tables: &Tables, size: u8) -> Result<Vec<MapObject>, String> {
    let resource = RESOURCES
        .iter()
        .position(|r| *r == p.resource)
        .ok_or_else(|| format!("unknown resource `{}`", p.resource))?;
    let template = tables
        .piles
        .iter()
        .find(|(r, _)| *r == resource as u32)
        .map(|(_, t)| t.clone())
        .ok_or_else(|| format!("no resource-pile template for {}", p.resource))?;
    if p.from > p.to || p.to >= size || p.row >= size {
        return Err(format!(
            "piles on row {} from {} to {}: out of order or off the map",
            p.row, p.from, p.to
        ));
    }
    let row = |e: String| format!("piles on row {}: {e}", p.row);
    let guards = if p.guards.is_empty() {
        if p.question.is_some() {
            return Err(row("a question needs guards".into()));
        }
        None
    } else {
        if p.guards.len() > ARMY_SLOTS {
            return Err(row(format!(
                "{} guard stacks (1–{ARMY_SLOTS})",
                p.guards.len()
            )));
        }
        let mut army = [None; ARMY_SLOTS];
        for (slot, (name, count)) in p.guards.iter().enumerate() {
            if *count == 0 {
                return Err(row(format!("an empty guard stack of {name}")));
            }
            army[slot] = Some((
                lookup(&tables.creatures, name, "creature").map_err(row)? as u16,
                *count,
            ));
        }
        Some(Guards {
            army,
            question: p.question.clone().unwrap_or_default(),
        })
    };
    Ok((p.from..=p.to)
        .map(|x| {
            MapObject::Pile(Pile {
                tile: (x, p.row),
                template: template.clone(),
                amount: p.amount,
                guards: guards.clone(),
            })
        })
        .collect())
}

/// A special ground by its picture's name.
fn ground(g: &GroundSpec, tables: &Tables, size: u8) -> Result<MapObject, String> {
    let name = g.picture.to_ascii_lowercase();
    let name = name.strip_suffix(".def").unwrap_or(&name);
    let template = tables
        .grounds
        .iter()
        .find(|t| t.def.to_ascii_lowercase().strip_suffix(".def") == Some(name))
        .cloned()
        .ok_or_else(|| format!("no special-ground picture `{}`", g.picture))?;
    let [x, y] = g.at;
    if x >= size || y >= size {
        return Err(format!("ground `{}` at {:?}: off the map", g.picture, g.at));
    }
    Ok(MapObject::Ground(Ground {
        tile: (x, y),
        template,
    }))
}

/// A visitable object without a body by its picture's name (E08 T32a).
fn plain(o: &ObjectSpec, tables: &Tables, size: u8) -> Result<MapObject, String> {
    let name = o.picture.to_ascii_lowercase();
    let name = name.strip_suffix(".def").unwrap_or(&name);
    let template = tables
        .plains
        .iter()
        .find(|t| t.def.to_ascii_lowercase().strip_suffix(".def") == Some(name))
        .cloned()
        .ok_or_else(|| format!("no plain-object picture `{}`", o.picture))?;
    let [x, y] = o.at;
    if x >= size || y >= size {
        return Err(format!("object `{}` at {:?}: off the map", o.picture, o.at));
    }
    Ok(MapObject::Plain(Plain {
        tile: (x, y),
        template,
    }))
}

/// The tiles a template covers with its anchor on `anchor`: those it blocks or is visited from
/// (bit `j` of mask byte `i` is the tile `7 - j` columns left of and `5 - i` rows above the
/// anchor; `docs/formats/h3m.md`), the ones off the map left out.
fn covered(template: &ObjectTemplate, anchor: (u8, u8)) -> Vec<(u8, u8)> {
    let mut tiles = Vec::new();
    let rows = template.passable.iter().zip(&template.visitable).zip(0u8..);
    for ((&passable, &visitable), i) in rows {
        for j in 0..8u8 {
            let bit = 1 << j;
            let open = passable & bit != 0 && visitable & bit == 0;
            let tile = anchor.0.checked_sub(7 - j).zip(anchor.1.checked_sub(5 - i));
            tiles.extend(tile.filter(|_| !open));
        }
    }
    tiles
}

/// A town with its entrance on `entrance`, and the line that describes it in the template.
fn town_object(
    t: &TownSpec,
    tables: &Tables,
    entrance: (u8, u8),
    size: u8,
) -> Result<(Town, String), String> {
    let faction = FACTIONS
        .iter()
        .position(|f| f.eq_ignore_ascii_case(t.faction.trim()))
        .ok_or_else(|| format!("unknown faction `{}`", t.faction))?;
    let template = tables
        .towns
        .iter()
        .find(|(f, _)| *f == faction as u32)
        .map(|(_, template)| template.clone())
        .ok_or_else(|| format!("no town template for {}", t.faction))?;
    let owner = match t.owner.as_str() {
        "blue" => Some(1),
        "tan" => Some(2),
        "neutral" => None,
        other => {
            return Err(format!(
                "a town's owner is blue, tan or neutral, not `{other}`"
            ));
        }
    };
    // The picture (8 columns, 6 rows) ends two columns east of the entrance, on its row.
    let tile = (entrance.0.saturating_add(2), entrance.1);
    if entrance.0 < 5 || entrance.1 < 5 || tile.0 >= size {
        return Err(format!(
            "a town with its entrance on {entrance:?} does not fit the map (its picture needs 5 \
             columns west of the entrance, 2 east and 5 rows north): give the hero `at`"
        ));
    }
    let buildings = match &t.buildings {
        None => TownBuildings::Standard { fort: false },
        Some(names) => {
            let mut built = [0u8; 6];
            for name in names {
                let bit = building_bit(FACTIONS[faction], name)?;
                built[usize::from(bit / 8)] |= 1 << (bit % 8);
            }
            TownBuildings::Custom {
                built,
                forbidden: [0; 6],
            }
        }
    };
    if t.garrison.len() > ARMY_SLOTS {
        return Err(format!("a garrison of {} stacks", t.garrison.len()));
    }
    let mut garrison = [None; ARMY_SLOTS];
    for (slot, (name, count)) in t.garrison.iter().enumerate() {
        let creature = lookup(&tables.creatures, name, "creature")?;
        if *count == 0 {
            return Err(format!("an empty garrison stack of {name}"));
        }
        garrison[slot] = Some((creature as u16, *count));
    }
    let owner_name = match owner {
        Some(1) => "Blue",
        Some(_) => "Tan (the computer's)",
        None => "Neutral",
    };
    let mut faction_name = FACTIONS[faction].to_owned();
    faction_name[..1].make_ascii_uppercase();
    let built = t.buildings.as_ref().map_or_else(
        || "the standard buildings without a fort".to_owned(),
        |names| format!("Village Hall, {}", names.join(", ")),
    );
    let defenders = if t.garrison.is_empty() {
        "the default garrison".to_owned()
    } else {
        let stacks: Vec<String> = t.garrison.iter().map(|(c, n)| format!("{n} {c}")).collect();
        format!("garrison {}", stacks.join(", "))
    };
    let summary = format!("{owner_name} {faction_name} town ({built}); {defenders}");
    let town = Town {
        tile,
        template,
        owner,
        garrison: (!t.garrison.is_empty()).then_some(garrison),
        tight_formation: t.tight,
        buildings,
    };
    Ok((town, summary))
}

/// A building's bit in the map, by its name or the faction's special building.
fn building_bit(faction: &str, name: &str) -> Result<u8, String> {
    let name = name.trim().to_ascii_lowercase();
    BUILDINGS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|&(_, bit)| bit)
        .or_else(|| {
            SPECIAL_BUILDINGS
                .iter()
                .find(|(f, n, _)| *f == faction && *n == name)
                .map(|&(_, _, bit)| bit)
        })
        .ok_or_else(|| format!("unknown building `{name}` for a {faction} town"))
}

fn terrain(name: &str) -> Result<Terrain, String> {
    Ok(match name {
        "dirt" => Terrain::Dirt,
        "sand" => Terrain::Sand,
        "grass" => Terrain::Grass,
        "snow" => Terrain::Snow,
        "swamp" => Terrain::Swamp,
        "rough" => Terrain::Rough,
        "subterranean" => Terrain::Subterranean,
        "lava" => Terrain::Lava,
        other => return Err(format!("unknown terrain `{other}`")),
    })
}

/// A patch's terrain: a land terrain, water (E04c T3: the coast; the map's own terrain is land)
/// or rock (E08 T22a1: the underground's walls).
fn patch_terrain(name: &str) -> Result<Terrain, String> {
    match name {
        "water" => Ok(Terrain::Water),
        "rock" => Ok(Terrain::Rock),
        _ => terrain(name),
    }
}

/// Index of a unique name in a list (ASCII case ignored).
fn lookup(list: &[String], name: &str, what: &str) -> Result<usize, String> {
    let mut hits = list
        .iter()
        .enumerate()
        .filter(|(_, n)| n.eq_ignore_ascii_case(name.trim()));
    match (hits.next(), hits.next()) {
        (Some((i, _)), None) => Ok(i),
        (Some(_), Some(_)) => Err(format!("{what} `{name}` is ambiguous; use its id")),
        (None, _) => Err(format!("unknown {what} `{name}`")),
    }
}

fn hero(
    h: &HeroSpec,
    tables: &Tables,
    tile: (u8, u8),
    name: Option<&str>,
    default_owner: &str,
) -> Result<Hero, String> {
    let hero_type = match &h.hero_type {
        NameOrId::Name(n) => lookup(&tables.heroes, n, "hero")?,
        NameOrId::Id(id) => usize::from(*id),
    };
    let class = *tables
        .hero_classes
        .get(hero_type)
        .ok_or_else(|| format!("unknown hero type {hero_type}"))?;
    let specialty = tables.specialty_text(hero_type);
    if specialty != h.specialty {
        return Err(format!(
            "{} has the specialty `{specialty}`, the spec expects `{}`",
            tables.heroes[hero_type], h.specialty
        ));
    }
    let owner = match (h.owner.as_str(), default_owner) {
        ("red", "blue") => 1, // an opponent hero always belongs to the second player
        ("red", _) => 0,
        ("blue", _) => 1,
        // A computer player's hero (E23 T7, batch 4b): the map lets no human play Tan.
        ("tan", _) => 2,
        (other, _) => return Err(format!("owner `{other}` (red, blue or tan)")),
    };
    let name = h.name.as_deref().or(name).map(str::to_owned);
    if h.defaults {
        let listed = h.primary.is_some()
            || h.experience != 0
            || !h.skills.is_empty()
            || !h.army.is_empty()
            || !h.artifacts.is_empty()
            || !h.backpack.is_empty()
            || !h.spells.is_empty();
        if listed {
            return Err("a hero with `defaults` lists no army, experience, skills, \
                        artifacts or spells"
                .into());
        }
        return Ok(Hero {
            tile,
            owner,
            hero_type: hero_type as u8,
            class,
            name,
            experience: None,
            skills: None,
            army: None,
            tight_formation: h.tight,
            artifacts: None,
            spells: None,
            primary: None,
        });
    }
    let primary = h
        .primary
        .ok_or("`primary` is required unless the hero has `defaults`")?;
    if h.skills.len() > 8 {
        return Err("at most 8 secondary skills".into());
    }
    let skills = h
        .skills
        .iter()
        .map(|(name, level)| {
            if !(1..=3).contains(level) {
                return Err(format!("skill level {level} (1–3)"));
            }
            Ok((lookup(&tables.skills, name, "skill")? as u8, *level))
        })
        .collect::<Result<Vec<_>, String>>()?;
    if h.army.is_empty() || h.army.len() > ARMY_SLOTS {
        return Err(format!("{} army stacks (1–{ARMY_SLOTS})", h.army.len()));
    }
    let mut army = [None; ARMY_SLOTS];
    for (slot, (name, count)) in h.army.iter().enumerate() {
        if *count == 0 {
            return Err(format!("empty stack of {name}"));
        }
        army[slot] = Some((lookup(&tables.creatures, name, "creature")? as u16, *count));
    }
    let mut worn = [None; ARTIFACT_SLOTS];
    for (slot_name, artifact) in &h.artifacts {
        let slot = SLOT_NAMES
            .iter()
            .position(|s| s == slot_name)
            .ok_or_else(|| format!("unknown slot `{slot_name}`"))?;
        let names: Vec<String> = tables.artifacts.iter().map(|a| a.name.clone()).collect();
        let id = lookup(&names, artifact, "artifact")?;
        if !tables.artifacts[id].slots[slot] {
            return Err(format!("{artifact} cannot be worn in `{slot_name}`"));
        }
        worn[slot] = Some(id as u16);
    }
    let backpack = h
        .backpack
        .iter()
        .map(|artifact| {
            let names: Vec<String> = tables.artifacts.iter().map(|a| a.name.clone()).collect();
            Ok(lookup(&names, artifact, "artifact")? as u16)
        })
        .collect::<Result<Vec<_>, String>>()?;
    if !h.spells.is_empty() && !h.artifacts.contains_key(SPELL_BOOK_SLOT) {
        return Err(format!("spells need a worn `{SPELL_BOOK_SLOT}`"));
    }
    let mut spells = [0u8; 9];
    for name in &h.spells {
        let id = lookup(&tables.spells, name, "spell")?;
        if id >= MAP_SPELLS {
            return Err(format!("{name} is not a hero spell"));
        }
        spells[id / 8] |= 1 << (id % 8);
    }
    Ok(Hero {
        tile,
        owner,
        hero_type: hero_type as u8,
        class,
        name,
        experience: Some(h.experience),
        skills: Some(skills),
        army: Some(army),
        tight_formation: h.tight,
        artifacts: Some(HeroArtifacts { worn, backpack }),
        spells: Some(spells),
        primary: Some(primary),
    })
}

fn monster(
    template: ObjectTemplate,
    tile: (u8, u8),
    count: u16,
    disposition: u8,
    flees: bool,
) -> MapObject {
    MapObject::Monster(Monster {
        tile,
        template,
        count,
        disposition,
        never_flees: !flees,
        fixed_count: true,
        treasure: None,
    })
}

/// One line describing a hero as the user should see it in the game.
fn hero_summary(h: &HeroSpec, hero: &Hero, tables: &Tables) -> String {
    let owner = ["Red", "Blue", "Tan (the computer)"][usize::from(hero.owner.min(2))];
    let name = hero
        .name
        .as_deref()
        .unwrap_or(&tables.heroes[usize::from(hero.hero_type)]);
    let Some([a, d, p, k]) = h.primary else {
        return format!(
            "{owner} hero **{name}** ({}, specialty {}): everything its hero type's own (army, \
             skills, artifacts, spells)",
            tables.heroes[usize::from(hero.hero_type)],
            h.specialty,
        );
    };
    let skills = if h.skills.is_empty() {
        "no skills".to_owned()
    } else {
        h.skills
            .iter()
            .map(|(s, l)| format!("{} {s}", LEVELS[usize::from(*l) - 1]))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let army = h
        .army
        .iter()
        .map(|(c, n)| format!("{n} {c}"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut artifacts = h
        .artifacts
        .iter()
        .map(|(slot, a)| format!("{a} ({slot})"))
        .collect::<Vec<_>>()
        .join(", ");
    let carried = h.backpack.iter().map(|a| format!("{a} (backpack)"));
    artifacts = (!artifacts.is_empty())
        .then_some(artifacts)
        .into_iter()
        .chain(carried)
        .collect::<Vec<_>>()
        .join(", ");
    if artifacts.is_empty() {
        artifacts = "no artifacts".into();
    }
    let spells = if h.spells.is_empty() {
        "no spells".to_owned()
    } else {
        format!("spells {}", h.spells.join(", "))
    };
    let level = if h.experience == 0 {
        String::new()
    } else {
        format!(", {} experience", h.experience)
    };
    format!(
        "{owner} hero **{}** ({}, specialty {}{level}): attack {a}, defense {d}, power {p}, \
         knowledge {k}; {skills}; {artifacts}; {spells}; army {army}",
        name,
        tables.heroes[usize::from(hero.hero_type)],
        h.specialty,
    )
}

fn encounter_text(
    out: &mut String,
    enc: &Encounter,
    own: &str,
    against: Option<&str>,
    more: &[String],
) {
    let questions = if enc.questions.is_empty() {
        String::new()
    } else {
        format!(" ({})", enc.questions.join(", "))
    };
    if !out.ends_with("\n\n") {
        out.push('\n');
    }
    let _ = writeln!(out, "## {} — {}{questions}\n", enc.id, enc.title);
    let _ = writeln!(out, "- {own}");
    if let Some(against) = against {
        let _ = writeln!(out, "- Opponent: {against}");
    }
    if enc.level == 1 {
        let _ = writeln!(out, "- On the underground level.");
    }
    for hero in more {
        let _ = writeln!(out, "- {hero}");
    }
    let _ = writeln!(out);
    for (i, step) in enc.procedure.iter().enumerate() {
        let _ = writeln!(out, "{}. {step}", i + 1);
    }
    if enc.record.is_empty() {
        return;
    }
    let _ = writeln!(out, "\n**Record**\n");
    for r in &enc.record {
        if let Some(field) = &r.field {
            let _ = writeln!(out, "- {field}: ");
        }
        if let Some(columns) = &r.table {
            if !out.ends_with("\n\n") {
                out.push('\n');
            }
            let _ = writeln!(out, "| {} |", columns.join(" | "));
            let _ = writeln!(out, "|{}", "---|".repeat(columns.len()));
            let empty = if r.labels.is_empty() { 1 } else { 0 };
            for _ in 0..r.rows.unwrap_or(empty) {
                let _ = writeln!(out, "|{}", "   |".repeat(columns.len()));
            }
            for label in &r.labels {
                let rest = "   |".repeat(columns.len().saturating_sub(1));
                let _ = writeln!(out, "| {label} |{rest}");
            }
            let _ = writeln!(out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tables::{Artifact, Specialty};

    fn tables() -> Tables {
        let mut slots = [false; ARTIFACT_SLOTS];
        slots[13] = true;
        let mut book = [false; ARTIFACT_SLOTS];
        book[17] = true;
        Tables {
            creatures: vec!["Pikeman".into(), "Archangel".into()],
            heroes: vec![
                "Orrin".into(),
                "Sylvia".into(),
                "Twin".into(),
                "Twin".into(),
            ],
            hero_classes: vec![0, 0, 1, 1],
            specialties: vec![
                Specialty { kind: 0, id: 0 },
                Specialty { kind: 2, id: 6 },
                Specialty { kind: 1, id: 1 },
                Specialty { kind: 3, id: 9 },
            ],
            skills: vec!["Archery".into(), "Armorer".into()],
            artifacts: vec![
                Artifact {
                    name: "Ballista".into(),
                    slots,
                },
                Artifact {
                    name: "Spell Book".into(),
                    slots: book,
                },
            ],
            spells: (0..=MAP_SPELLS).map(|i| format!("Spell {i}")).collect(),
            monsters: vec![(
                0,
                ObjectTemplate::from_objects_txt(
                    "AvWPike.def 011111111111111111111111111111111111111111111111 \
                     100000000000000000000000000000000000000000000000 111111111 000000100 54 0 2 0",
                )
                .expect("template"),
            )],
            // Wood and gold piles (E08 T16b); no gems template, to see the error.
            piles: [(0, "AVTwood0.def"), (6, "AVTgold0.def")]
                .map(|(id, def)| {
                    let line = format!(
                        "{def} 011111111111111111111111111111111111111111111111 \
                         100000000000000000000000000000000000000000000000 111111111 000000001 \
                         79 {id} 5 0"
                    );
                    (
                        id,
                        ObjectTemplate::from_objects_txt(&line).expect("template"),
                    )
                })
                .to_vec(),
            grounds: vec![
                ObjectTemplate::from_objects_txt(
                    "AVXcf0.def 111111111111111111111111111111111111111111111111 \
                     000000000000000000000000000000000000000000000000 111111111 011111111 222 0 0 1",
                )
                .expect("template"),
            ],
            // The land Cartographer (E08 T32a), a tree and the light blue Border Gate (E08 T36a),
            // the game's lines.
            plains: [
                "AVXmaps0.def 001111111111111111111111111111111111111111111111 \
                 010000000000000000000000000000000000000000000000 111111111 010111111 13 1 0 0",
                "AVLtr3d0.def 011111111111111111111111111111111111111111111111 \
                 000000000000000000000000000000000000000000000000 111111111 000010101 155 0 0 0",
                "avxbgt00.def 000111111111111111111111111111111111111111111111 \
                 010000000000000000000000000000000000000000000000 111111111 011111111 212 0 0 0",
            ]
            .map(|line| ObjectTemplate::from_objects_txt(line).expect("template"))
            .to_vec(),
            // A Stronghold (E04c T5b), the game's line; no Castle template, to see the error.
            towns: vec![(
                6,
                ObjectTemplate::from_objects_txt(
                    "AVCstrx0.def 000001110000011110001111111111111111111111111111 \
                     001000000000000000000000000000000000000000000000 011111111 011111111 98 6 1 0",
                )
                .expect("template"),
            )],
        }
    }

    const SIEGE: &str = r#"
        [map]
        name = "Siege"
        description = "d"
        terrain = "rough"

        [[encounter]]
        id = "S1"
        title = "A siege"
        [encounter.hero]
        type = "Sylvia"
        specialty = "resource gold"
        at = [10, 10]
        primary = [1, 1, 1, 1]
        army = [["Archangel", 2]]
        [encounter.opponent.town]
        faction = "Stronghold"
        buildings = ["fort", "citadel", "tavern", "Escape Tunnel"]
        garrison = [["Pikeman", 10], ["Archangel", 1]]
        tight = true
        [encounter.opponent.hero]
        type = "Orrin"
        specialty = "skill Archery"
        primary = [0, 0, 0, 0]
        army = [["Pikeman", 1]]
    "#;

    #[test]
    fn a_town_stands_north_of_the_hero_with_its_visitor_on_the_entrance() {
        let spec = parse(SIEGE).expect("spec");
        let (map, text) = build(&spec, &tables(), "t.h3m").expect("build");
        let [
            MapObject::Hero(attacker),
            MapObject::Town(town),
            MapObject::Hero(visitor),
        ] = &map.objects[..]
        else {
            panic!("not a hero, a town and its visitor: {:?}", map.objects);
        };
        assert_eq!((attacker.tile, attacker.owner), ((10, 10), 0));
        // The entrance two columns west of the town's position, north of the attacker.
        assert_eq!(
            (town.tile, town.owner, town.template.subclass),
            ((12, 9), Some(1), 6)
        );
        assert_eq!((visitor.tile, visitor.owner), ((10, 9), 1));
        // Fort 3, Citadel 4, Tavern 6, the Escape Tunnel 18 (the Stronghold's).
        assert_eq!(
            town.buildings,
            TownBuildings::Custom {
                built: [0b0101_1000, 0, 0b0000_0100, 0, 0, 0],
                forbidden: [0; 6]
            }
        );
        assert_eq!(
            town.garrison.map(|g| g[..2].to_vec()),
            Some(vec![Some((0, 10)), Some((1, 1))])
        );
        assert!(town.tight_formation && map.players[1].is_some());
        assert!(text.contains(
            "- Opponent: Blue Stronghold town (Village Hall, fort, citadel, tavern, Escape Tunnel); \
             garrison 10 Pikeman, 1 Archangel; visiting hero: Blue hero **S1**"
        ));
        // A neutral town with the standard buildings and the default garrison, and no Blue player.
        let neutral = SIEGE
            .replace(
                "faction = \"Stronghold\"",
                "faction = \"stronghold\"\nowner = \"neutral\"",
            )
            .replace(
                "buildings = [\"fort\", \"citadel\", \"tavern\", \"Escape Tunnel\"]\n",
                "",
            )
            .replace("garrison = [[\"Pikeman\", 10], [\"Archangel\", 1]]\n", "");
        let neutral = &neutral[..neutral.find("[encounter.opponent.hero]").expect("hero")];
        let (map, text) = build(&parse(neutral).expect("spec"), &tables(), "t.h3m").expect("build");
        let MapObject::Town(town) = &map.objects[1] else {
            panic!("not a town: {:?}", map.objects[1]);
        };
        assert_eq!(
            (town.owner, town.garrison, town.buildings),
            (None, None, TownBuildings::Standard { fort: false })
        );
        assert!(map.players[1].is_none());
        assert!(text.contains(
            "Neutral Stronghold town (the standard buildings without a fort); the default garrison"
        ));
    }

    #[test]
    fn tan_is_the_computer_s_player() {
        // E23 T7 (batch 4b): a town and its visiting hero of Tan, which no human may play.
        let tan = SIEGE.replace("tight = true\n", "tight = true\n        owner = \"tan\"\n");
        let tan = tan.replace(
            "specialty = \"skill Archery\"\n",
            "specialty = \"skill Archery\"\n        owner = \"tan\"\n",
        );
        let (map, text) = build(&parse(&tan).expect("spec"), &tables(), "t.h3m").expect("build");
        let [_, MapObject::Town(town), MapObject::Hero(visitor)] = &map.objects[..] else {
            panic!("not a hero, a town and its visitor: {:?}", map.objects);
        };
        assert_eq!((town.owner, visitor.owner), (Some(2), 2));
        let player = map.players[2].as_ref().expect("Tan");
        assert!(!player.human && player.computer && map.players[1].is_none());
        assert!(text.contains("Tan (the computer's) Stronghold town"));
        assert!(text.contains("visiting hero: Tan (the computer) hero **S1**"));
        // A town's visitor is its owner's.
        let mixed = SIEGE.replace("tight = true\n", "tight = true\n        owner = \"tan\"\n");
        let error = build(&parse(&mixed).expect("spec"), &tables(), "t.h3m").expect_err("mixed");
        assert!(
            error.contains("a town's visiting hero is its owner's"),
            "{error}"
        );
        let green = SIEGE.replace(
            "specialty = \"skill Archery\"\n",
            "specialty = \"skill Archery\"\n        owner = \"green\"\n",
        );
        let error = build(&parse(&green).expect("spec"), &tables(), "t.h3m").expect_err("green");
        assert!(
            error.contains("owner `green` (red, blue or tan)"),
            "{error}"
        );
    }

    #[test]
    fn a_map_town_belongs_to_its_owner_and_no_encounter() {
        // E23 T7 (batch 4b): Tan's own Tavern town far from the battle.
        let spec = SIEGE.replace(
            "[[encounter]]",
            "[[map.town]]\n        entrance = [24, 20]\n        faction = \"stronghold\"\n        \
             owner = \"tan\"\n        buildings = [\"tavern\"]\n\n        [[encounter]]",
        );
        let (map, _) = build(&parse(&spec).expect("spec"), &tables(), "t.h3m").expect("build");
        let MapObject::Town(town) = map.objects.last().expect("objects") else {
            panic!("not a town: {:?}", map.objects.last());
        };
        assert_eq!((town.tile, town.owner), ((26, 20), Some(2)));
        assert!(
            map.players[2]
                .as_ref()
                .is_some_and(|p| p.computer && !p.human)
        );
        let bad = spec.replace("entrance = [24, 20]", "entrance = [2, 20]");
        let error = build(&parse(&bad).expect("spec"), &tables(), "t.h3m").expect_err("bad");
        assert!(error.starts_with("the town at [2, 20]: "), "{error}");
    }

    #[test]
    fn rejects_mistakes_in_a_town() {
        let hero_part = SIEGE.find("[encounter.opponent.hero]").expect("hero");
        for (from, to, error) in [
            (
                "faction = \"Stronghold\"",
                "faction = \"Castle\"",
                "no town template for Castle",
            ),
            (
                "faction = \"Stronghold\"",
                "faction = \"Elves\"",
                "unknown faction `Elves`",
            ),
            (
                "tight = true",
                "tight = true\nowner = \"red\"",
                "a town's owner is blue, tan or neutral, not `red`",
            ),
            (
                "\"Escape Tunnel\"",
                "\"Blood Obelisk\"",
                "unknown building `blood obelisk` for a stronghold town",
            ),
            (
                "[\"Pikeman\", 10]",
                "[\"Pikeman\", 0]",
                "an empty garrison stack of Pikeman",
            ),
            (
                "[\"Pikeman\", 10]",
                "[\"Imp\", 1]",
                "unknown creature `Imp`",
            ),
            ("at = [10, 10]", "at = [4, 10]", "does not fit the map"),
            ("at = [10, 10]", "at = [10, 5]", "does not fit the map"),
            ("at = [10, 10]", "at = [34, 10]", "does not fit the map"),
            (
                "tight = true",
                "tight = true\nowner = \"neutral\"",
                "a neutral town has no visiting hero",
            ),
            (
                "type = \"Orrin\"",
                "type = \"Orrin\"\nat = [1, 1]",
                "a town's hero stands on its entrance: no `at`",
            ),
            (
                "[encounter.opponent.town]",
                "[encounter.opponent]\nmonster = \"Pikeman\"\ncount = 1\n[encounter.opponent.town]",
                "an opponent is `monster` + `count`, `hero`, or `town`",
            ),
        ] {
            let text = SIEGE.replace(from, to);
            let err = build(&parse(&text).expect("spec"), &tables(), "t.h3m").unwrap_err();
            assert!(err.contains(error), "{from} -> {to}: {err}");
        }
        let eight = format!(
            "{}garrison = [{}]\n{}",
            &SIEGE[..SIEGE.find("garrison").expect("garrison")],
            ["[\"Pikeman\", 1]"; 8].join(", "),
            &SIEGE[SIEGE.find("tight").expect("tight")..hero_part],
        );
        let err = build(&parse(&eight).expect("spec"), &tables(), "t.h3m").unwrap_err();
        assert!(err.contains("a garrison of 8 stacks"), "{err}");
    }

    const SPEC: &str = r#"
        [map]
        name = "Test"
        description = "d"
        terrain = "dirt"
        instructions = ["Start it."]

        [[encounter]]
        id = "R1"
        title = "A test"
        questions = ["Q23"]
        procedure = ["Attack."]
        predictions = "secret"
        [encounter.hero]
        type = "Sylvia"
        specialty = "resource gold"
        primary = [4, 0, 1, 1]
        skills = [["Armorer", 1]]
        army = [["Archangel", 2]]
        artifacts = { "war machine 1" = "Ballista" }
        [encounter.opponent]
        monster = "Pikeman"
        count = 10
        [[encounter.record]]
        table = ["stack", "damage"]
        rows = 2
        [[encounter.record]]
        field = "anything odd"

        [[encounter]]
        id = "B1"
        title = "Second"
        [encounter.hero]
        type = "Orrin"
        specialty = "skill Archery"
        primary = [0, 0, 0, 0]
        army = [["Pikeman", 1]]
        backpack = ["Spell Book"]
        [encounter.opponent.hero]
        type = 2
        specialty = "creature Archangel"
        primary = [1, 2, 3, 4]
        army = [["Pikeman", 5]]

        [[encounter]]
        id = "M1"
        title = "Magic"
        [encounter.hero]
        type = 3
        specialty = "spell Spell 9"
        primary = [0, 0, 1, 2]
        army = [["Pikeman", 1]]
        artifacts = { "spell book" = "Spell Book" }
        spells = ["Spell 0", "Spell 9", "Spell 69"]
    "#;

    #[test]
    fn grounds_come_first_and_only_keeps_one_encounter_in_its_place() {
        let text = SPEC.replace(
            "instructions = [\"Start it.\"]",
            "instructions = [\"Start it.\"]\n[[map.ground]]\npicture = \"avxCF0.def\"\nat = [6, 4]\n\
             [[map.ground]]\npicture = \"AVXcf0\"\nat = [12, 4]",
        );
        let spec = parse(&text).expect("spec");
        let (map, template) = build(&spec, &tables(), "t.h3m").expect("build");
        let tiles: Vec<_> = map.objects[..2]
            .iter()
            .map(|o| match o {
                MapObject::Ground(g) => (g.tile, g.template.class),
                other => panic!("not a ground: {other:?}"),
            })
            .collect();
        assert_eq!(tiles, [((6, 4), 222), ((12, 4), 222)]);
        assert_eq!(map.objects.len(), 7);
        // Only B1: the grounds, then its hero and its opponent where the whole map has them; the
        // template still lists every encounter.
        let (one, same) = build_only(&spec, &tables(), "t.h3m", Some("B1")).expect("only");
        assert_eq!(same, template);
        assert_eq!(one.objects[..2], map.objects[..2]);
        assert_eq!(one.objects[2..], map.objects[4..6]);
        assert!(one.players[1].is_some());
        let err = build_only(&spec, &tables(), "t.h3m", Some("X")).unwrap_err();
        assert_eq!(err, "no encounter `X`");
        for (picture, at, error) in [
            ("AVXzz0", "[1, 1]", "no special-ground picture `AVXzz0`"),
            (
                "AVXcf0",
                "[36, 1]",
                "ground `AVXcf0` at [36, 1]: off the map",
            ),
            (
                "AVXcf0",
                "[1, 36]",
                "ground `AVXcf0` at [1, 36]: off the map",
            ),
        ] {
            let bad = SPEC.replace(
                "instructions = [\"Start it.\"]",
                &format!("[[map.ground]]\npicture = \"{picture}\"\nat = {at}"),
            );
            let spec = parse(&bad).expect("spec");
            assert_eq!(build(&spec, &tables(), "t.h3m").unwrap_err(), error);
        }
    }

    #[test]
    fn the_level_limit_and_a_plain_object_are_written() {
        let with = |extra: &str| {
            let text = SPEC.replace(
                "instructions = [\"Start it.\"]",
                &format!("instructions = [\"Start it.\"]\n{extra}"),
            );
            build(&parse(&text).expect("spec"), &tables(), "t.h3m")
        };
        let (map, _) =
            with("level_limit = 5\n[[map.object]]\npicture = \"avxMAPS0.def\"\nat = [20, 20]")
                .expect("build");
        assert_eq!(map.level_limit, 5);
        let MapObject::Plain(plain) = &map.objects[0] else {
            panic!("not a plain object: {:?}", map.objects[0])
        };
        let template = &plain.template;
        assert_eq!(
            (plain.tile, template.class, template.subclass),
            ((20, 20), 13, 1)
        );
        // It covers its tile and its entrance west of it; the map's edge leaves tiles out.
        assert_eq!(covered(template, (20, 20)), [(19, 20), (20, 20)]);
        assert_eq!(covered(template, (0, 0)), [(0, 0)]);
        let (map, _) = build(&parse(SPEC).expect("spec"), &tables(), "t.h3m").expect("build");
        assert_eq!((map.level_limit, map.banned_skills.len()), (0, 0));
        // E08 T36b: banned skills by name, any case, as their codes.
        let (map, _) = with("banned_skills = [\"archery\", \"Armorer\"]").expect("build");
        assert_eq!(map.banned_skills, [0, 1]);
        assert_eq!(
            with("banned_skills = [\"Juggling\"]").unwrap_err(),
            "unknown skill `Juggling`"
        );
        for (picture, at, error) in [
            ("AVXzz0", "[1, 1]", "no plain-object picture `AVXzz0`"),
            (
                "AVXmaps0",
                "[36, 1]",
                "object `AVXmaps0` at [36, 1]: off the map",
            ),
            (
                "AVXmaps0",
                "[1, 36]",
                "object `AVXmaps0` at [1, 36]: off the map",
            ),
            // R1's hero stands on (3, 3), the Cartographer's entrance.
            ("AVXmaps0", "[4, 3]", "two objects on the tile (3, 3)"),
            // ... and on a tree, or on a Border Gate's blocked side.
            ("AVLtr3d0", "[3, 3]", "two objects on the tile (3, 3)"),
            ("AVXbgt00", "[5, 3]", "two objects on the tile (3, 3)"),
        ] {
            let object = format!("[[map.object]]\npicture = \"{picture}\"\nat = {at}");
            assert_eq!(with(&object).unwrap_err(), error);
        }
        // E08 T36a: a tree blocks its tile; a Border Gate its tile and the two west of it, the
        // middle one its entrance, the key's colour its subclass.
        let (map, _) = with(
            "[[map.object]]\npicture = \"AVLtr3d0\"\nat = [20, 20]\n\
             [[map.object]]\npicture = \"avxBGT00.def\"\nat = [24, 22]",
        )
        .expect("build");
        let [MapObject::Plain(tree), MapObject::Plain(gate), ..] = &map.objects[..] else {
            panic!("not two plain objects: {:?}", map.objects)
        };
        assert_eq!((tree.template.class, tree.tile), (155, (20, 20)));
        assert_eq!(covered(&tree.template, tree.tile), [(20, 20)]);
        assert_eq!(
            (gate.template.class, gate.template.subclass, gate.tile),
            (212, 0, (24, 22))
        );
        assert_eq!(
            covered(&gate.template, gate.tile),
            [(22, 22), (23, 22), (24, 22)]
        );
        assert_eq!(gate.template.visitable[5], 0x40);
    }

    #[test]
    fn builds_map_and_template() {
        let spec = parse(SPEC).expect("spec");
        let (map, text) = build(&spec, &tables(), "t.h3m").expect("build");
        assert_eq!(map.objects.len(), 5);
        let MapObject::Hero(h) = &map.objects[0] else {
            panic!("hero first")
        };
        assert_eq!((h.hero_type, h.owner, h.tile), (1, 0, (3, 3)));
        assert_eq!(h.skills, Some(vec![(1, 1)]));
        assert_eq!(h.army.expect("army")[0], Some((1, 2)));
        assert_eq!(h.artifacts.as_ref().expect("artifacts").worn[13], Some(0));
        assert_eq!(h.spells, Some([0; 9]));
        let MapObject::Monster(m) = &map.objects[1] else {
            panic!("monster second")
        };
        assert_eq!((m.tile, m.count), ((4, 3), 10));
        let MapObject::Hero(blue) = &map.objects[3] else {
            panic!("blue hero")
        };
        assert_eq!((blue.owner, blue.hero_type, blue.tile), (1, 2, (10, 3)));
        let MapObject::Hero(orrin) = &map.objects[2] else {
            panic!("B1's hero third")
        };
        let carried = orrin.artifacts.as_ref().expect("artifacts");
        let book = tables()
            .artifacts
            .iter()
            .position(|a| a.name == "Spell Book")
            .expect("book") as u16;
        assert_eq!(
            (carried.worn, carried.backpack.clone()),
            ([None; ARTIFACT_SLOTS], vec![book])
        );
        assert!(text.contains("; Spell Book (backpack); no spells;"));
        assert!(map.players[1].is_some());
        let MapObject::Hero(mage) = &map.objects[4] else {
            panic!("spell caster")
        };
        assert_eq!(mage.spells, Some([0b1, 0b10, 0, 0, 0, 0, 0, 0, 0b10_0000]));
        assert!(text.contains("Spell Book (spell book); spells Spell 0, Spell 9, Spell 69;"));
        assert!(text.contains("; no spells;"));
        assert!(text.contains("## R1 — A test (Q23)"));
        assert!(text.contains("| stack | damage |"));
        assert!(text.contains("- anything odd: "));
        assert!(text.contains("; Basic Armorer;"));
        assert!(!text.contains("secret"));
        h3_map::writer::write(&map).expect("writable");
    }

    /// The stamp (E08 T16b, decision 18): CRC-32 of the text with CRLF made LF, in the map's
    /// description and at the template's top.
    #[test]
    fn the_stamp_marks_the_map_and_the_template() {
        assert_eq!(stamp("123456789"), "cbf43926");
        assert_eq!(stamp("a\r\nb\r\n"), stamp("a\nb\n"));
        assert_ne!(stamp("a\nb\n"), stamp("a\nc\n"));
        let spec = parse(SPEC).expect("spec");
        assert_eq!(spec.stamp, stamp(SPEC));
        let (map, text) = build(&spec, &tables(), "t.h3m").expect("build");
        assert_eq!(map.description, format!("Stamp {}. d", spec.stamp));
        assert!(text.contains(&format!(
            "**Stamp `{0}`.** The map's description in the scenario list starts with \"Stamp {0}.\"",
            spec.stamp
        )));
        // A Windows checkout of the same spec: the same stamp, map and template.
        let windows = parse(&SPEC.replace('\n', "\r\n")).expect("spec");
        assert_eq!(windows.stamp, spec.stamp);
        let (map2, text2) = build(&windows, &tables(), "t.h3m").expect("build");
        assert_eq!((map2, text2), (map, text));
        // Any other change of the text changes it; the file cannot set it.
        assert_ne!(
            parse(&SPEC.replace("Attack.", "Attack!"))
                .expect("spec")
                .stamp,
            spec.stamp
        );
        let set = SPEC.replacen("[map]", "stamp = \"0\"\n[map]", 1);
        assert!(parse(&set).expect_err("stamp key").contains("stamp"));
    }

    #[test]
    fn rejects_mistakes() {
        let t = tables();
        let fails = |from: &str, to: &str| {
            let spec = parse(&SPEC.replacen(from, to, 1)).expect("spec");
            build(&spec, &t, "t.h3m").expect_err(to)
        };
        assert!(fails("resource gold", "resource gems").contains("specialty"));
        assert!(fails("\"Archangel\", 2", "\"Angel\", 2").contains("unknown creature"));
        assert!(fails("[\"Armorer\", 1]", "[\"Armorer\", 4]").contains("skill level"));
        assert!(fails("\"war machine 1\"", "\"head\"").contains("cannot be worn"));
        assert!(fails("type = 2", "type = \"Twin\"").contains("ambiguous"));
        assert!(fails("type = 2", "type = \"Sylvia\"").contains("specialty"));
        assert!(fails("terrain = \"dirt\"", "terrain = \"water\"").contains("terrain"));
        assert!(fails("count = 10", "count = 0").contains("at least one"));
        assert!(fails("\"Spell 69\"", "\"Spell 70\"").contains("not a hero spell"));
        assert!(fails("\"Spell 69\"", "\"Spell 99\"").contains("unknown spell"));
        assert!(fails("backpack = [\"Spell Book\"]", "backpack = [\"Tome\"]").contains("unknown"));
        let no_book = fails("artifacts = { \"spell book\" = \"Spell Book\" }", "");
        assert!(no_book.contains("need a worn"));
    }

    /// The adventure map's shape (E08 T10a): patches, chosen tiles, more heroes, defaults.
    const WALKS: &str = r#"
        [map]
        name = "Walks"
        description = "d"
        size = 72
        terrain = "grass"
        [[map.patch]]
        terrain = "swamp"
        from = [20, 0]
        to = [32, 71]
        [[map.patch]]
        terrain = "sand"
        from = [20, 0]
        to = [20, 0]
        [[map.patch]]
        terrain = "water"
        from = [60, 60]
        to = [61, 60]
        [[map.road]]
        kind = "gravel"
        row = 30
        from = 17
        to = 19
        [[map.pile]]
        resource = "wood"
        row = 50
        from = 10
        to = 12
        [[map.pile]]
        resource = "gold"
        row = 52
        from = 20
        to = 20
        amount = 7

        [[encounter]]
        id = "W1"
        title = "Walk into the swamp"
        [encounter.hero]
        type = "Orrin"
        specialty = "skill Archery"
        at = [19, 30]
        primary = [0, 0, 1, 1]
        army = [["Pikeman", 1]]
        [encounter.opponent.hero]
        type = "Sylvia"
        specialty = "resource gold"
        primary = [0, 0, 1, 1]
        army = [["Pikeman", 1]]
        [[encounter.also]]
        type = 2
        specialty = "creature Archangel"
        owner = "blue"
        at = [40, 8]
        defaults = true
        [[encounter.also]]
        type = 3
        specialty = "spell Spell 9"
        at = [42, 8]
        name = "Named"
        defaults = true
        [[encounter.record]]
        table = ["hero", "load 1", "load 2"]
        labels = ["Twin", "Named"]
    "#;

    #[test]
    fn builds_patches_chosen_tiles_and_heroes_of_their_type() {
        let spec = parse(WALKS).expect("spec");
        let (map, text) = build(&spec, &tables(), "w.h3m").expect("build");
        assert_eq!(map.size, 72);
        assert_eq!(
            map.patches,
            [
                Patch {
                    terrain: Terrain::Swamp,
                    from: (20, 0),
                    to: (32, 71)
                },
                Patch {
                    terrain: Terrain::Sand,
                    from: (20, 0),
                    to: (20, 0)
                },
                Patch {
                    terrain: Terrain::Water,
                    from: (60, 60),
                    to: (61, 60)
                }
            ]
        );
        assert_eq!(
            map.roads,
            [Road {
                kind: RoadKind::Gravel,
                row: 30,
                from: 17,
                to: 19
            }]
        );
        // The piles come last, row by row from west to east.
        let piles: Vec<(u8, u8, u32, u32)> = map
            .objects
            .iter()
            .filter_map(|o| match o {
                MapObject::Pile(p) => Some((p.tile.0, p.tile.1, p.template.subclass, p.amount)),
                _ => None,
            })
            .collect();
        assert_eq!(
            piles,
            [
                (10, 50, 0, 0),
                (11, 50, 0, 0),
                (12, 50, 0, 0),
                (20, 52, 6, 7)
            ]
        );
        assert!(matches!(map.objects[4], MapObject::Pile(_)));
        let heroes: Vec<&Hero> = map
            .objects
            .iter()
            .filter_map(|o| match o {
                MapObject::Hero(h) => Some(h),
                MapObject::Monster(_) => panic!("no monsters"),
                MapObject::Pile(_)
                | MapObject::Ground(_)
                | MapObject::Plain(_)
                | MapObject::Town(_)
                | MapObject::Reward(_) => None,
            })
            .collect();
        // The encounter's hero, its `also` heroes, then the opponent beside the hero.
        let tiles: Vec<_> = heroes.iter().map(|h| (h.tile, h.owner)).collect();
        assert_eq!(
            tiles,
            [((19, 30), 0), ((40, 8), 1), ((42, 8), 0), ((20, 30), 1)]
        );
        let typed = heroes[1];
        assert_eq!(typed.name, None);
        assert_eq!((typed.experience, typed.skills.as_ref()), (None, None));
        assert_eq!((typed.army, typed.artifacts.as_ref()), (None, None));
        assert_eq!((typed.spells, typed.primary), (None, None));
        assert_eq!(heroes[2].name.as_deref(), Some("Named"));
        assert_eq!(heroes[3].name.as_deref(), Some("W1"));
        assert!(text.contains(
            "- Blue hero **Twin** (Twin, specialty creature Archangel): everything its hero \
             type's own (army, skills, artifacts, spells)"
        ));
        assert!(text.contains("- Red hero **Named** (Twin, specialty spell Spell 9): everything"));
        assert!(text.contains("- Opponent: Blue hero **W1** (Sylvia,"));
        assert!(text.contains(
            "| hero | load 1 | load 2 |\n|---|---|---|\n| Twin |   |   |\n| Named |   |   |\n"
        ));
        h3_map::writer::write(&map).expect("writable");
    }

    #[test]
    fn rejects_mistakes_in_the_adventure_shape() {
        let t = tables();
        let fails = |from: &str, to: &str| {
            let spec = parse(&WALKS.replacen(from, to, 1)).expect("spec");
            build(&spec, &t, "w.h3m").expect_err(to)
        };
        assert!(fails("to = [32, 71]", "to = [32, 72]").contains("off the map"));
        assert!(fails("to = [32, 71]", "to = [72, 71]").contains("off the map"));
        assert!(fails("to = [32, 71]", "to = [19, 71]").contains("out of order"));
        assert!(fails("from = [20, 0]", "from = [20, 72]").contains("out of order"));
        // Water may be a patch (the coast), not the map's own terrain.
        assert!(fails("\"swamp\"", "\"ice\"").contains("unknown terrain"));
        assert!(fails("\"grass\"", "\"water\"").contains("unknown terrain"));
        assert!(fails("\"gravel\"", "\"paved\"").contains("unknown road"));
        for (from, to) in [
            ("from = 17", "from = 19"),
            ("to = 19", "to = 72"),
            ("row = 30", "row = 72"),
        ] {
            assert!(fails(from, to).contains("one tile or less, or off the map"));
        }
        assert!(fails("at = [40, 8]\n", "").contains("needs `at`"));
        assert!(fails("at = [40, 8]", "at = [19, 30]").contains("two objects"));
        assert!(fails("at = [40, 8]", "at = [20, 30]").contains("two objects"));
        for listed in [
            "primary = [1, 1, 1, 1]",
            "experience = 1",
            "skills = [[\"Archery\", 1]]",
            "army = [[\"Pikeman\", 1]]",
            "artifacts = { \"war machine 1\" = \"Ballista\" }",
            "backpack = [\"Ballista\"]",
            "spells = [\"Spell 1\"]",
        ] {
            let with = format!("defaults = true\n{listed}");
            assert!(fails("defaults = true", &with).contains("lists no army"));
        }
        let no_primary = fails("primary = [0, 0, 1, 1]\n", "");
        assert!(no_primary.contains("`primary` is required"));
        assert!(fails("army = [[\"Pikeman\", 1]]", "").contains("0 army stacks"));
        assert!(fails("\"wood\"", "\"timber\"").contains("unknown resource `timber`"));
        assert!(fails("\"gold\"", "\"gems\"").contains("no resource-pile template for gems"));
        for (from, to) in [
            ("from = 10\n        to = 12", "from = 13\n        to = 12"),
            ("to = 12", "to = 72"),
            ("row = 50", "row = 72"),
        ] {
            assert!(
                fails(from, to).contains("out of order or off the map"),
                "{to}"
            );
        }
        assert!(fails("row = 52", "row = 30").contains("two objects"));
    }

    /// E08 T22a1 (batch E3's shape): the difficulty, an underground level, monsters of any
    /// disposition that may flee, guarded piles.
    const BELOW: &str = r#"
        [map]
        name = "Below"
        description = "d"
        terrain = "grass"
        difficulty = "Easy"
        [map.underground]
        terrain = "subterranean"
        [[map.underground.patch]]
        terrain = "rock"
        from = [0, 0]
        to = [35, 1]

        [[map.pile]]
        resource = "wood"
        row = 20
        from = 10
        to = 11
        amount = 5
        guards = [["Pikeman", 12], ["Archangel", 1]]
        question = "Fight?"

        [[map.pile]]
        resource = "gold"
        row = 22
        from = 10
        to = 10
        level = 1
        guards = [["Pikeman", 3]]

        [[encounter]]
        id = "J"
        title = "Join or flee"
        [encounter.hero]
        type = "Orrin"
        specialty = "skill Archery"
        primary = [0, 0, 0, 0]
        army = [["Pikeman", 56]]
        [encounter.opponent]
        monster = "Pikeman"
        count = 10
        disposition = "friendly"
        flees = true

        [[encounter]]
        id = "U"
        title = "Below"
        level = 1
        [encounter.hero]
        type = "Sylvia"
        specialty = "resource gold"
        at = [11, 27]
        primary = [0, 0, 0, 0]
        army = [["Pikeman", 1]]
        [encounter.opponent]
        monster = "Pikeman"
        count = 30
    "#;

    #[test]
    fn builds_the_underground_monsters_of_any_disposition_and_guarded_piles() {
        let spec = parse(BELOW).expect("spec");
        let (map, text) = build(&spec, &tables(), "b.h3m").expect("build");
        h3_map::writer::write(&map).expect("writes");
        let below = map.underground.as_ref().expect("underground");
        assert_eq!(below.terrain, Terrain::Subterranean);
        assert_eq!(
            below.patches,
            [Patch {
                terrain: Terrain::Rock,
                from: (0, 0),
                to: (35, 1),
            }]
        );
        let monsters = |objects: &[MapObject]| -> Vec<(u8, u8, u16, u8, bool)> {
            objects
                .iter()
                .filter_map(|o| match o {
                    MapObject::Monster(m) => {
                        Some((m.tile.0, m.tile.1, m.count, m.disposition, m.never_flees))
                    }
                    _ => None,
                })
                .collect()
        };
        let piles = |objects: &[MapObject]| -> Vec<((u8, u8), Option<Guards>)> {
            objects
                .iter()
                .filter_map(|o| match o {
                    MapObject::Pile(p) => Some((p.tile, p.guards.clone())),
                    _ => None,
                })
                .collect()
        };
        assert_eq!(monsters(&map.objects), [(4, 3, 10, 1, false)]);
        assert_eq!(monsters(&below.objects), [(12, 27, 30, 4, true)]);
        let mut army = [None; ARMY_SLOTS];
        army[0] = Some((0, 12));
        army[1] = Some((1, 1));
        let asks = Guards {
            army,
            question: "Fight?".into(),
        };
        assert_eq!(
            piles(&map.objects),
            [((10, 20), Some(asks.clone())), ((11, 20), Some(asks))]
        );
        let mut army = [None; ARMY_SLOTS];
        army[0] = Some((0, 3));
        let at_once = Guards {
            army,
            question: String::new(),
        };
        assert_eq!(piles(&below.objects), [((10, 22), Some(at_once))]);
        let heroes: Vec<(u8, u8)> = below
            .objects
            .iter()
            .filter_map(|o| match o {
                MapObject::Hero(h) => Some(h.tile),
                _ => None,
            })
            .collect();
        assert_eq!(heroes, [(11, 27)]);
        assert!(text.contains("**Difficulty: Easy.** Choose it in the scenario list"));
        assert!(text.contains("Opponent: 10 Pikeman (wandering monster, friendly, may flee)"));
        assert!(text.contains("Opponent: 30 Pikeman (wandering monster, never flees)"));
        let underground = text.find("- On the underground level.").expect("the level");
        assert!(underground > text.find("## U").expect("U"));
        // `only` keeps the level of the one encounter.
        let (only, _) = build_only(&spec, &tables(), "b.h3m", Some("U")).expect("only");
        assert_eq!(monsters(&only.objects), []);
        let only_below = only.underground.expect("underground");
        assert_eq!(monsters(&only_below.objects), [(12, 27, 30, 4, true)]);
        // A one-level spec writes no underground.
        let (one, _) = build(&parse(WALKS).expect("spec"), &tables(), "w.h3m").expect("build");
        assert!(one.underground.is_none());
    }

    #[test]
    fn rejects_mistakes_in_the_underground_monsters_and_guards() {
        let t = tables();
        let fails = |from: &str, to: &str| {
            let spec = parse(&BELOW.replacen(from, to, 1)).expect("spec");
            build(&spec, &t, "b.h3m").expect_err(to)
        };
        assert!(fails("\"Easy\"", "\"medium\"").contains("unknown difficulty `medium`"));
        let one_level = BELOW.replacen("[map.underground]", "[map.unused]", 1);
        assert!(parse(&one_level).is_err());
        let no_level = |text: &str| {
            let start = text.find("        [map.underground]").expect("level");
            let end = text.find("        [[map.pile]]").expect("piles");
            format!("{}{}", &text[..start], &text[end..])
        };
        let spec = parse(&no_level(BELOW)).expect("spec");
        let err = build(&spec, &t, "b.h3m").expect_err("no level");
        assert!(err.contains("encounter U: level 1: no such level"), "{err}");
        let surface_encounter =
            BELOW.replacen("level = 1\n        [encounter.hero]", "[encounter.hero]", 1);
        let spec = parse(&no_level(&surface_encounter)).expect("spec");
        let err = build(&spec, &t, "b.h3m").expect_err("no level");
        assert!(
            err.contains("piles on row 22 at level 1: no such level"),
            "{err}"
        );
        assert!(
            fails(
                "level = 1\n        [encounter.hero]",
                "level = 2\n        [encounter.hero]"
            )
            .contains("no such level")
        );
        assert!(
            fails("terrain = \"subterranean\"", "terrain = \"rock\"").contains("unknown terrain")
        );
        assert!(fails("to = [35, 1]", "to = [36, 1]").contains("off the map"));
        assert!(fails("\"friendly\"", "\"shy\"").contains("unknown disposition `shy`"));
        let guards = "guards = [[\"Pikeman\", 3]]";
        assert!(fails(guards, "question = \"?\"").contains("a question needs guards"));
        let eight = format!("guards = [{}]", ["[\"Pikeman\", 1]"; 8].join(", "));
        assert!(fails(guards, &eight).contains("8 guard stacks (1–7)"));
        assert!(fails(guards, "guards = [[\"Pikeman\", 0]]").contains("an empty guard stack"));
        assert!(fails(guards, "guards = [[\"Imp\", 1]]").contains("unknown creature `Imp`"));
        // Two objects on one tile of the underground; the same tile on the surface is free.
        assert!(
            fails(
                "row = 22\n        from = 10\n        to = 10",
                "row = 27\n        from = 11\n        to = 11",
            )
            .contains("on the tile (11, 27) underground")
        );
        let spec = parse(&BELOW.replacen("level = 1\n        guards", "guards", 1)).expect("spec");
        let (map, _) = build(&spec, &t, "b.h3m").expect("the surface's tile");
        assert_eq!(map.underground.expect("level").objects.len(), 2);
        // A disposition or flight is a monster's.
        let hero = "[encounter.opponent]\n        monster = \"Pikeman\"\n        count = 30";
        let blue = "[encounter.opponent]\n        flees = true\n        [encounter.opponent.hero]\n        \
                    type = \"Twin\"\n        specialty = \"x\"\n        primary = [0, 0, 0, 0]\n        \
                    army = [[\"Pikeman\", 1]]";
        assert!(fails(hero, blue).contains("with a monster only"));
    }
}
