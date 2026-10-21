//! `h3 ai census`: the statistics of a set of maps that the encounter distribution v1 is derived
//! from (E05 T5a, `docs/epics/E05-log.md` §4.2). It reads each map through `h3-map` (the raw
//! model and the static layer's import) and adds up **aggregate numbers only** (ADR-0009: counts,
//! no map content, no per-map rows): the maps' versions and sizes, the tiles by passage, every
//! battle place (terrain and battlefield) with the open land tiles, the wandering monsters and the
//! guarded piles fought there, the monsters by level, the towns by faction and fort, and the
//! other objects that start battles (heroes, garrisons, creature banks, boats).
//!
//! A battle's place is the engine's: [`h3_adventure::AdventureState::battle_ground`] of the
//! defender's tile (GAM-BAT-001, ADV-GRD-001; survey `E04c-battlefields.md` §4), so the census and
//! the adventure map never disagree on where a battle is fought.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use h3_adventure::{MapSetup, ObjectKind, ObjectSetup, Passage, Pos};
use h3_game::{DEFAULT_DIFFICULTY, GameRuleset, GameSetup, GameState, PlayerSetup};
use h3_map::Limits;
use h3_map::model::{Body, Buildings, RawMap, Version};
use serde::Serialize;

/// The census format's version: bump it when a number changes meaning or a section is added.
pub const CENSUS_VERSION: u32 = 1;

/// The ruleset's profile the maps are imported with.
pub const PROFILE: &str = "base:faithful";

/// A town's faction by its map subclass, 0 to 8 (`docs/formats/h3m.md`, objects 77 and 98: the
/// original's town order).
const TOWN_FACTIONS: [&str; 9] = [
    "base:castle",
    "base:rampart",
    "base:tower",
    "base:inferno",
    "base:necropolis",
    "base:dungeon",
    "base:stronghold",
    "base:fortress",
    "ab:conflux",
];

/// The object classes the census counts from the raw map (`docs/formats/h3m.md`; VCMI 1.7.5's
/// `Obj`): a boat, a creature bank and the four banks with their own class, a garrison (subclass
/// 1 the anti-magic one, E04c), a hero, a prison, a random hero, a town and a random town.
mod class {
    pub const BOAT: u32 = 8;
    pub const CREATURE_BANK: u32 = 16;
    pub const DERELICT_SHIP: u32 = 24;
    pub const DRAGON_UTOPIA: u32 = 25;
    pub const GARRISON: u32 = 33;
    pub const HERO: u32 = 34;
    pub const PRISON: u32 = 62;
    pub const RANDOM_HERO: u32 = 70;
    pub const RANDOM_TOWN: u32 = 77;
    pub const CRYPT: u32 = 84;
    pub const SHIPWRECK: u32 = 85;
    pub const TOWN: u32 = 98;
    pub const GARRISON_VERTICAL: u32 = 219;
}

/// The bits of a town's custom `built` set for its fort, citadel and castle
/// (`docs/formats/h3m.md`, "Town buildings").
const FORT_BITS: [usize; 3] = [3, 4, 5];

/// The census of a set of maps: the file `crates/h3-sim/data/census-v<N>.toml`.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct Census {
    /// What it is and what it was read with.
    pub census: Header,
    /// The maps.
    pub maps: Maps,
    /// Every tile of every level.
    pub tiles: Tiles,
    /// The wandering monsters.
    pub monsters: Monsters,
    /// The other objects that start battles, and the special grounds.
    pub objects: Objects,
    /// The towns by faction (a random town under `random`) and fort.
    pub towns: BTreeMap<String, Forts>,
    /// Every battle place, ordered by terrain and battlefield.
    pub place: Vec<Place>,
}

/// The census's identity.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct Header {
    /// Always `census`.
    pub id: String,
    /// [`CENSUS_VERSION`].
    pub version: u32,
    /// The packs the maps are imported with.
    pub packs: Vec<String>,
    /// Their profile.
    pub profile: String,
    /// The maps counted.
    pub maps: u32,
}

/// The maps by version and size.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct Maps {
    /// Maps of the base game (RoE).
    pub roe: u32,
    /// Maps of Armageddon's Blade.
    pub ab: u32,
    /// Maps of the Shadow of Death.
    pub sod: u32,
    /// Maps with an underground level.
    pub two_levels: u32,
    /// Maps whose terrain (every tile) equals an earlier map's: the shipped maps' allied twins.
    pub terrain_twins: u32,
    /// Maps by size.
    pub size: Sizes,
}

/// Maps by size, as the game names them by side length in tiles.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct Sizes {
    /// Small, 36.
    pub s: u32,
    /// Medium, 72.
    pub m: u32,
    /// Large, 108.
    pub l: u32,
    /// Extra large, 144.
    pub xl: u32,
    /// Any other side length.
    pub other: u32,
}

/// Tiles by passage (the adventure ruleset's terrains) and by what stands on them.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct Tiles {
    /// Every tile.
    pub total: u64,
    /// Walked on foot.
    pub land: u64,
    /// Sailed.
    pub water: u64,
    /// Never entered (rock).
    pub impassable: u64,
    /// Land tiles on the underground level.
    pub underground_land: u64,
    /// Land tiles on the coast (the map's flag).
    pub coast: u64,
    /// Land tiles no object blocks or is visited from: where a hero can stand and be attacked.
    pub open_land: u64,
    /// Water tiles no object blocks or is visited from.
    pub open_water: u64,
}

/// The wandering monsters (the nine monster classes).
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct Monsters {
    /// Placements.
    pub placements: u32,
    /// Placements whose count the map fixes (the others draw it from the creature's range).
    pub fixed_count: u32,
    /// Placements by level, `1` to `7` and `any` (class 71): `fixed` names its creature (its
    /// level the creature's), `random` draws it among the level's creatures.
    pub level: BTreeMap<String, Creatures>,
}

/// Monster placements of one level.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct Creatures {
    /// The map names the creature.
    pub fixed: u32,
    /// The game draws it.
    pub random: u32,
}

/// The objects other than monsters that lead to battles, and the special grounds.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct Objects {
    /// Piles with guards (ADV-PIK-004).
    pub guarded_piles: u32,
    /// Special grounds (ADV-GRD-001).
    pub special_grounds: u32,
    /// Heroes placed on the map (class 34), owned or not.
    pub heroes: u32,
    /// Random heroes (class 70).
    pub random_heroes: u32,
    /// Prisons (class 62).
    pub prisons: u32,
    /// Garrisons (classes 33, 219) other than anti-magic ones.
    pub garrisons: u32,
    /// Anti-magic garrisons (subclass 1).
    pub anti_magic_garrisons: u32,
    /// Creature banks (class 16, every subclass).
    pub creature_banks: u32,
    /// Derelict ships (class 24).
    pub derelict_ships: u32,
    /// Dragon Utopias (class 25).
    pub dragon_utopias: u32,
    /// Crypts (class 84).
    pub crypts: u32,
    /// Shipwrecks (class 85).
    pub shipwrecks: u32,
    /// Boats (class 8).
    pub boats: u32,
}

/// Towns of one faction by their fort at the start.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct Forts {
    /// No fort.
    pub none: u32,
    /// A fort.
    pub fort: u32,
    /// A citadel.
    pub citadel: u32,
    /// A castle.
    pub castle: u32,
}

/// One battle place: a terrain and a battlefield (none for the plain field), with what is fought
/// there.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct Place {
    /// The battle's terrain key.
    pub terrain: String,
    /// The battlefield's key; absent for none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battlefield: Option<String>,
    /// Open land tiles ([`Tiles::open_land`]) whose battles are fought here.
    pub open_land: u64,
    /// Wandering monsters standing where their battles are fought here.
    pub monsters: u32,
    /// Guarded piles whose battles are fought here.
    pub guarded_piles: u32,
}

/// A map the census could not count, with why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skipped {
    /// The file.
    pub path: PathBuf,
    /// Why.
    pub reason: String,
}

/// The original packs (RoE to SoD) under [`PROFILE`], as the maps are imported.
///
/// # Errors
///
/// A pack that does not compile (never for the built-in packs).
pub fn original_rules() -> Result<GameRuleset, String> {
    let (core, part) = h3_pack_adventure::compile(&h3_pack_adventure::ORIGINAL_PACKS, PROFILE)
        .map_err(|e| format!("the original adventure packs: {e}"))?;
    let packs = [
        h3_pack_battle::BASE_PACK,
        h3_pack_battle::AB_PACK,
        h3_pack_battle::SOD_PACK,
    ];
    let battle = h3_pack_battle::compile(&packs, PROFILE)
        .map_err(|e| format!("the original battle packs: {e}"))?;
    GameRuleset::new(Arc::new(core), Arc::new(part), Arc::new(battle))
        .map_err(|e| format!("the original packs: {e:?}"))
}

/// The map files among `paths`: a file as given, a folder's `.h3m` files (not its subfolders),
/// each list sorted by name.
///
/// # Errors
///
/// A path that cannot be read.
pub fn map_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_dir() {
            let entries =
                std::fs::read_dir(path).map_err(|e| format!("{}: {e}", path.display()))?;
            let mut found: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_file() && is_map_file(p))
                .collect();
            found.sort();
            files.extend(found);
        } else if path.is_file() {
            files.push(path.clone());
        } else {
            return Err(format!("{}: no such file or folder", path.display()));
        }
    }
    Ok(files)
}

/// The census of `files`, with the files it skipped (a map that does not read or whose static
/// layer does not import).
///
/// # Errors
///
/// A ruleset that does not compile, or a file that cannot be read from disk.
pub fn run(files: &[PathBuf], rules: &GameRuleset) -> Result<(Census, Vec<Skipped>), String> {
    let mut acc = Accumulator::default();
    let mut skipped = Vec::new();
    let limits = Limits::default();
    for path in files {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        if let Err(reason) = acc.add(&bytes, &limits, rules) {
            skipped.push(Skipped {
                path: path.clone(),
                reason,
            });
        }
    }
    Ok((acc.finish(), skipped))
}

/// Writes `census` as TOML with a header comment that says what it is.
///
/// # Errors
///
/// Serialization (never for a census).
pub fn to_toml(census: &Census) -> Result<String, String> {
    let body = toml::to_string(census).map_err(|e| e.to_string())?;
    Ok(format!(
        "# The census of the maps the encounter distribution is derived from (E05 T5a,\n\
         # docs/epics/E05-log.md §4.2), written by `h3 ai census`: aggregate counts only\n\
         # (ADR-0009), no map content. Regenerate it, never edit it by hand.\n\n{body}"
    ))
}

/// The running totals.
#[derive(Default)]
struct Accumulator {
    census: Census,
    places: BTreeMap<(String, Option<String>), Place>,
    terrains: Vec<u64>,
}

impl Accumulator {
    /// Adds one map file.
    fn add(&mut self, file: &[u8], limits: &Limits, rules: &GameRuleset) -> Result<(), String> {
        let raw = h3_map::read(file, limits).map_err(|e| format!("read: {e}"))?;
        let map = h3_map::import_map(&raw, rules).map_err(|e| format!("static layer: {e}"))?;
        let setup = GameSetup {
            players: vec![PlayerSetup { colour: 0 }],
            difficulty: DEFAULT_DIFFICULTY,
            map,
            heroes: Vec::new(),
            level_limit: 0,
            banned_skills: Vec::new(),
        };
        // The seed draws the static layer's random contents, which no number here reads.
        let state = GameState::new(rules, &setup, 0).map_err(|e| format!("setup: {e}"))?;
        self.header(&raw);
        self.tiles(&state, rules);
        self.objects(&setup.map, &state, rules);
        self.raw_objects(&raw);
        Ok(())
    }

    fn header(&mut self, raw: &RawMap) {
        let maps = &mut self.census.maps;
        match raw.header.version {
            Version::RoE => maps.roe += 1,
            Version::Ab => maps.ab += 1,
            Version::Sod => maps.sod += 1,
        }
        maps.two_levels += u32::from(raw.header.two_levels);
        let size = &mut maps.size;
        match raw.header.size {
            36 => size.s += 1,
            72 => size.m += 1,
            108 => size.l += 1,
            144 => size.xl += 1,
            _ => size.other += 1,
        }
        let fingerprint = raw
            .terrain
            .tiles
            .iter()
            .flat_map(|t| [t.terrain, t.view, t.river, t.road, t.flags])
            .fold(
                0xcbf2_9ce4_8422_2325_u64 ^ u64::from(raw.header.size),
                |h, b| (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3),
            );
        if self.terrains.contains(&fingerprint) {
            maps.terrain_twins += 1;
        } else {
            self.terrains.push(fingerprint);
        }
        self.census.census.maps += 1;
    }

    fn place(&mut self, state: &GameState, rules: &GameRuleset, pos: Pos) -> &mut Place {
        let (terrain, battlefield) = state.adventure().battle_ground(rules.adventure(), pos);
        let terrain = rules
            .core()
            .terrains()
            .get(terrain.index())
            .map_or_else(|| "?".to_owned(), |t| t.key.to_string());
        let battlefield = battlefield.map(|k| k.to_string());
        self.places
            .entry((terrain.clone(), battlefield.clone()))
            .or_insert_with(|| Place {
                terrain,
                battlefield,
                ..Place::default()
            })
    }

    fn tiles(&mut self, state: &GameState, rules: &GameRuleset) {
        let map = state.adventure().map();
        let (width, height) = (usize::from(map.width()), usize::from(map.height()));
        for (index, tile) in map.tiles().iter().enumerate() {
            let pos = Pos {
                x: u16::try_from(index % width).unwrap_or(u16::MAX),
                y: u16::try_from(index / width % height).unwrap_or(u16::MAX),
                level: u8::try_from(index / (width * height)).unwrap_or(u8::MAX),
            };
            let open = !tile.blocked && !tile.visitable;
            let tiles = &mut self.census.tiles;
            tiles.total += 1;
            match rules.adventure().passage(tile.terrain) {
                Some(Passage::Land(_)) => {
                    tiles.land += 1;
                    tiles.underground_land += u64::from(pos.level > 0);
                    tiles.coast += u64::from(map.is_coast(pos));
                    if open {
                        tiles.open_land += 1;
                        self.place(state, rules, pos).open_land += 1;
                    }
                }
                Some(Passage::Water(_)) => {
                    tiles.water += 1;
                    tiles.open_water += u64::from(open);
                }
                Some(Passage::Impassable) | None => tiles.impassable += 1,
            }
        }
    }

    fn objects(&mut self, setup: &MapSetup, state: &GameState, rules: &GameRuleset) {
        let part = rules.adventure();
        let map = state.adventure().map();
        for object in &setup.objects {
            let kind = object
                .object_type
                .and_then(|id| part.object_type(id))
                .map(|t| &t.kind);
            let tile = visited_tile(object, map.width(), map.height());
            match (kind, &object.monster) {
                (Some(ObjectKind::Monster { levels }), Some(monster)) => {
                    let monsters = &mut self.census.monsters;
                    monsters.placements += 1;
                    monsters.fixed_count += u32::from(monster.count > 0);
                    let level = match (levels, monster.creature) {
                        (Some([low, high]), _) if low == high => low.to_string(),
                        (Some(_), _) => "any".to_owned(),
                        (None, Some(creature)) => rules
                            .core()
                            .creature(creature)
                            .map_or_else(|| "?".to_owned(), |c| c.level.to_string()),
                        (None, None) => "?".to_owned(),
                    };
                    let entry = monsters.level.entry(level).or_default();
                    if levels.is_some() {
                        entry.random += 1;
                    } else {
                        entry.fixed += 1;
                    }
                    if let Some(pos) = tile {
                        self.place(state, rules, pos).monsters += 1;
                    }
                }
                (Some(ObjectKind::Ground { .. }), _) => self.census.objects.special_grounds += 1,
                _ => {}
            }
            if object.guards.is_some() {
                self.census.objects.guarded_piles += 1;
                if let Some(pos) = tile {
                    self.place(state, rules, pos).guarded_piles += 1;
                }
            }
        }
    }

    fn raw_objects(&mut self, raw: &RawMap) {
        let objects = &mut self.census.objects;
        for object in &raw.objects {
            let Some(template) = usize::try_from(object.template)
                .ok()
                .and_then(|t| raw.templates.get(t))
            else {
                continue;
            };
            match template.class {
                class::BOAT => objects.boats += 1,
                class::CREATURE_BANK => objects.creature_banks += 1,
                class::DERELICT_SHIP => objects.derelict_ships += 1,
                class::DRAGON_UTOPIA => objects.dragon_utopias += 1,
                class::CRYPT => objects.crypts += 1,
                class::SHIPWRECK => objects.shipwrecks += 1,
                class::GARRISON | class::GARRISON_VERTICAL if template.subclass == 1 => {
                    objects.anti_magic_garrisons += 1;
                }
                class::GARRISON | class::GARRISON_VERTICAL => objects.garrisons += 1,
                class::HERO => objects.heroes += 1,
                class::RANDOM_HERO => objects.random_heroes += 1,
                class::PRISON => objects.prisons += 1,
                class::TOWN | class::RANDOM_TOWN => {
                    let faction = if template.class == class::RANDOM_TOWN {
                        "random"
                    } else {
                        usize::try_from(template.subclass)
                            .ok()
                            .and_then(|s| TOWN_FACTIONS.get(s))
                            .copied()
                            .unwrap_or("?")
                    };
                    let forts = self.census.towns.entry(faction.to_owned()).or_default();
                    match &object.body {
                        Body::Town(town) => match fort_level(&town.buildings) {
                            0 => forts.none += 1,
                            1 => forts.fort += 1,
                            2 => forts.citadel += 1,
                            _ => forts.castle += 1,
                        },
                        _ => forts.none += 1,
                    }
                }
                _ => {}
            }
        }
    }

    fn finish(mut self) -> Census {
        self.census.census = Header {
            id: "census".to_owned(),
            version: CENSUS_VERSION,
            packs: vec!["base".to_owned(), "ab".to_owned(), "sod".to_owned()],
            profile: PROFILE.to_owned(),
            maps: self.census.census.maps,
        };
        self.census.place = self.places.into_values().collect();
        self.census
    }
}

/// The tile an object is visited from (its first visitable tile on the map): where a hero stands
/// to fight it, so the battle's tile is the object's (the defender's).
fn visited_tile(object: &ObjectSetup, width: u16, height: u16) -> Option<Pos> {
    object.visitable.offsets().find_map(|(column, row)| {
        let x = object.anchor.x.checked_sub(u16::from(column))?;
        let y = object.anchor.y.checked_sub(u16::from(row))?;
        (x < width && y < height).then_some(Pos {
            x,
            y,
            level: object.anchor.level,
        })
    })
}

/// A town's fort at the start: 0 none, 1 a fort, 2 a citadel, 3 a castle.
pub(crate) fn fort_level(buildings: &Buildings) -> u8 {
    match buildings {
        Buildings::Standard { fort } => u8::from(*fort),
        Buildings::Custom { built, .. } => FORT_BITS
            .iter()
            .rev()
            .position(|&bit| built[bit / 8] >> (bit % 8) & 1 == 1)
            .map_or(0, |from_top| 3 - u8::try_from(from_top).unwrap_or(3)),
    }
}

/// Whether `path` is a file the census reads when given its folder.
fn is_map_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("h3m"))
}
