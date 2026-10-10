//! The import (ADR-0015 point 4, E08 T5a and T6a): a raw map and the game's rules into
//! `h3_game::GameSetup`. It maps the format's codes to content keys and handles, draws nothing
//! (a hero without its own army keeps the starting-army placeholder) and rejects what the
//! current increment cannot represent with a typed [`Unsupported`] error. It checks no game
//! invariant: `GameSetup::validate` does, with the checks commands use. Notes:
//! `docs/formats/h3m.md`, "Import".

pub(crate) mod codes;

use std::fmt;

use h3_adventure::{MapSetup, ObjectKind, ObjectSetup, Pos, TileMask, TileSetup};
use h3_content::CreatureTypeId;
use h3_contracts::{ArmyStack, PlayerId};
use h3_game::{GameHeroSetup, GameRuleset, GameSetup, HeroArmy, PlayerSetup};

use crate::model::{Army, Body, HeroBody, Object, RawMap, Template, Version};

/// Object classes the import tells apart; every other object is a blocker in increment 1.
const HERO: u32 = 34;
const RANDOM_HERO: u32 = 70;
const RANDOM_TOWN: u32 = 77;
const TOWN: u32 = 98;
const HERO_PLACEHOLDER: u32 = 214;

/// Random stacks by level take the 14 codes below the format's "none" (VCMI 1.7.5
/// `MapReaderH3M::readCreature32`).
const RANDOM_STACK_CODES: u16 = 14;

/// What a map holds that the current increment cannot represent yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Unsupported {
    /// The player of this colour gets a hero generated at its main town (towns: E09).
    HeroAtMainTown(u8),
    /// The object at this index is a random hero, drawn when the game starts.
    RandomHero(usize),
    /// The object at this index is a campaign hero placeholder.
    HeroPlaceholder(usize),
    /// The hero object at this index has a random stack in its army.
    RandomStack(usize),
    /// The hero object at this index stands on a blocked tile of a town: a town's visiting hero
    /// (towns: E09; VCMI 1.7.5 `CGameState::placeHeroesInTowns` moves it to its own town's
    /// entrance, an "H3M format quirk").
    HeroInTown(usize),
}

impl Unsupported {
    /// The reason without its position, for counting maps by reason.
    pub const fn reason(self) -> &'static str {
        match self {
            Self::HeroAtMainTown(_) => "a hero generated at a main town",
            Self::RandomHero(_) => "a random hero",
            Self::HeroPlaceholder(_) => "a campaign hero placeholder",
            Self::RandomStack(_) => "a random stack in a hero's army",
            Self::HeroInTown(_) => "a hero standing in a town",
        }
    }
}

/// Why a raw map cannot be imported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportError {
    /// The map holds what the current increment cannot represent.
    Unsupported(Unsupported),
    /// The map is larger than a setup can describe: the side and the levels.
    MapSize(usize, usize),
    /// A code the format does not define.
    UnknownCode {
        /// What it names: `terrain`, `road`, `river`, `creature` or `hero type`.
        what: &'static str,
        /// The code.
        code: u16,
    },
    /// The ruleset lacks the content a code stands for (an AB map with RoE's rules): its key.
    MissingContent(&'static str),
    /// The object at this index names a template the map does not have.
    Template(usize),
    /// The hero object at this index belongs to a colour nobody plays.
    HeroOwner(usize),
    /// The hero object at this index has a template without exactly one visitable tile on the
    /// map, the tile the hero stands on.
    HeroTile(usize),
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(what) => write!(f, "not supported yet: {}", what.reason()),
            Self::MapSize(side, levels) => {
                write!(f, "a map of side {side} on {levels} levels is too large")
            }
            Self::UnknownCode { what, code } => write!(f, "unknown {what} code {code}"),
            Self::MissingContent(key) => write!(f, "the ruleset lacks `{key}`"),
            Self::Template(object) => write!(f, "object {object} has no template"),
            Self::HeroOwner(object) => {
                write!(f, "hero object {object} belongs to a colour nobody plays")
            }
            Self::HeroTile(object) => {
                write!(f, "hero object {object} does not name one tile to stand on")
            }
        }
    }
}

impl std::error::Error for ImportError {}

impl From<Unsupported> for ImportError {
    fn from(what: Unsupported) -> Self {
        Self::Unsupported(what)
    }
}

/// Imports a raw map with the game's rules into a game setup: the players in colour order, the
/// static map ([`import_map`]) and the heroes placed on it.
///
/// # Errors
///
/// The first [`ImportError`] found: the players (a hero generated at a main town), the static
/// map, then the hero objects in the map's order.
pub fn import(map: &RawMap, rules: &GameRuleset) -> Result<GameSetup, ImportError> {
    let (players, colours) = players(map)?;
    let static_map = import_map(map, rules)?;
    let objects = objects(map)?;
    let towns: Vec<ObjectSetup> = objects
        .iter()
        .filter(|(_, _, template)| matches!(template.class, TOWN | RANDOM_TOWN))
        .map(|(_, object, template)| footprint(object, template))
        .collect();
    let mut heroes = Vec::new();
    for (index, object, template) in objects {
        match (template.class, &object.body) {
            (HERO, Body::Hero(body)) => {
                let hero = hero(
                    map.header.version,
                    rules,
                    &colours,
                    index,
                    &footprint(object, template),
                    body,
                )?;
                if towns.iter().any(|town| blocks(town, hero.pos)) {
                    return Err(Unsupported::HeroInTown(index).into());
                }
                heroes.push(hero);
            }
            (RANDOM_HERO, _) => return Err(Unsupported::RandomHero(index).into()),
            (HERO_PLACEHOLDER, _) => return Err(Unsupported::HeroPlaceholder(index).into()),
            _ => {}
        }
    }
    Ok(GameSetup {
        players,
        map: static_map,
        heroes,
    })
}

/// Imports the static map (E08 T5a): the size, every tile's terrain, road and river, and every
/// object but the heroes as a blocker with its footprint (ADV-OBJ-001).
///
/// # Errors
///
/// The first [`ImportError`] found: the map's size, the tiles, then the objects' templates.
pub fn import_map(map: &RawMap, rules: &GameRuleset) -> Result<MapSetup, ImportError> {
    let side = u16::try_from(map.terrain.size);
    let levels = u8::try_from(map.terrain.levels);
    let (Ok(side), Ok(levels)) = (side, levels) else {
        return Err(ImportError::MapSize(map.terrain.size, map.terrain.levels));
    };
    let tiles = tiles(map, rules)?;
    let objects = objects(map)?
        .into_iter()
        .filter(|(_, _, template)| !matches!(template.class, HERO | RANDOM_HERO | HERO_PLACEHOLDER))
        .map(|(_, object, template)| footprint(object, template))
        .collect();
    Ok(MapSetup {
        width: side,
        height: side,
        levels,
        tiles,
        objects,
    })
}

/// The objects with their index and template.
fn objects(map: &RawMap) -> Result<Vec<(usize, &Object, &Template)>, ImportError> {
    map.objects
        .iter()
        .enumerate()
        .map(|(index, object)| {
            usize::try_from(object.template)
                .ok()
                .and_then(|t| map.templates.get(t))
                .map(|template| (index, object, template))
                .ok_or(ImportError::Template(index))
        })
        .collect()
}

/// The colours that play, in colour order, and the player each colour is.
fn players(map: &RawMap) -> Result<(Vec<PlayerSetup>, [Option<PlayerId>; 8]), ImportError> {
    let mut players = Vec::new();
    let mut colours = [None; 8];
    for ((player, colour), id) in map.players.iter().zip(0u8..).zip(&mut colours) {
        if !(player.human || player.computer) {
            continue;
        }
        let generates = player
            .details
            .as_ref()
            .and_then(|d| d.main_town.as_ref())
            .is_some_and(|t| t.generate_hero);
        if generates {
            return Err(Unsupported::HeroAtMainTown(colour).into());
        }
        *id = u8::try_from(players.len()).ok().map(PlayerId::new);
        players.push(PlayerSetup { colour });
    }
    Ok((players, colours))
}

/// Every tile's terrain, road and river as the ruleset's handles.
fn tiles(map: &RawMap, rules: &GameRuleset) -> Result<Vec<TileSetup>, ImportError> {
    let (core, part) = (rules.core(), rules.adventure());
    let terrains = codes::TERRAINS.map(|key| (key, core.terrain_by_key(key)));
    let roads = codes::ROADS.map(|key| (key, part.road_by_key(key)));
    let rivers = codes::RIVERS.map(|key| (key, part.river_by_key(key)));
    map.terrain
        .tiles
        .iter()
        .map(|tile| {
            Ok(TileSetup {
                terrain: lookup(&terrains, "terrain", tile.terrain, 0)?,
                road: optional(&roads, "road", tile.road)?,
                river: optional(&rivers, "river", tile.river)?,
            })
        })
        .collect()
}

/// The handle code `code` stands for in `table`, whose first entry is code `first`.
fn lookup<T: Copy>(
    table: &[(&'static str, Option<T>)],
    what: &'static str,
    code: u8,
    first: u8,
) -> Result<T, ImportError> {
    let (key, handle) = code
        .checked_sub(first)
        .and_then(|i| table.get(usize::from(i)))
        .ok_or(ImportError::UnknownCode {
            what,
            code: u16::from(code),
        })?;
    handle.ok_or(ImportError::MissingContent(key))
}

/// Like [`lookup`] for a table from code 1; code 0 is none.
fn optional<T: Copy>(
    table: &[(&'static str, Option<T>)],
    what: &'static str,
    code: u8,
) -> Result<Option<T>, ImportError> {
    (code != 0)
        .then(|| lookup(table, what, code, 1))
        .transpose()
}

/// An object's footprint from its template: blocked where the passability bit is clear,
/// visitable where the visitability bit is set (ADV-OBJ-001). The file's bit `j` of byte `i` is
/// `Objects.txt`'s character `47 − (8i + j)`, the tile `7 − j` columns left of and `5 − i` rows
/// above the anchor: [`TileMask`]'s bit `47 − (8i + j)`, the 48 bits in reverse order.
fn footprint(object: &Object, template: &Template) -> ObjectSetup {
    let reversed = |bytes: [u8; 6]| {
        let file = bytes
            .iter()
            .zip(0..)
            .fold(0u64, |acc, (&b, i)| acc | u64::from(b) << (8 * i));
        file.reverse_bits() >> 16
    };
    ObjectSetup {
        anchor: Pos::new(
            u16::from(object.pos.x),
            u16::from(object.pos.y),
            object.pos.z,
        ),
        blocked: TileMask::from_bits(!reversed(template.passable)),
        visitable: TileMask::from_bits(reversed(template.visitable)),
        kind: ObjectKind::Blocker,
    }
}

/// A placed hero: its owner, type, tile and army.
fn hero(
    version: Version,
    rules: &GameRuleset,
    colours: &[Option<PlayerId>; 8],
    index: usize,
    footprint: &ObjectSetup,
    body: &HeroBody,
) -> Result<GameHeroSetup, ImportError> {
    let owner = colours
        .get(usize::from(body.owner))
        .copied()
        .flatten()
        .ok_or(ImportError::HeroOwner(index))?;
    let key =
        codes::HERO_TYPES
            .get(usize::from(body.hero_type))
            .ok_or(ImportError::UnknownCode {
                what: "hero type",
                code: u16::from(body.hero_type),
            })?;
    let hero_type = rules
        .core()
        .hero_type_by_key(key)
        .ok_or(ImportError::MissingContent(key))?;
    Ok(GameHeroSetup {
        hero_type,
        owner,
        pos: stands_on(footprint).ok_or(ImportError::HeroTile(index))?,
        army: army(version, rules, index, body.army.as_ref())?,
    })
}

/// Whether the object's footprint blocks `pos`.
fn blocks(object: &ObjectSetup, pos: Pos) -> bool {
    let column = object.anchor.x.checked_sub(pos.x).map(u8::try_from);
    let row = object.anchor.y.checked_sub(pos.y).map(u8::try_from);
    match (column, row) {
        (Some(Ok(column)), Some(Ok(row))) => {
            object.anchor.level == pos.level && object.blocked.contains(column, row)
        }
        _ => false,
    }
}

/// The tile a hero stands on: its template's one visitable tile.
fn stands_on(footprint: &ObjectSetup) -> Option<Pos> {
    let mut tiles = footprint.visitable.offsets();
    let (column, row) = tiles.next()?;
    if tiles.next().is_some() {
        return None;
    }
    let anchor = footprint.anchor;
    Some(Pos::new(
        anchor.x.checked_sub(u16::from(column))?,
        anchor.y.checked_sub(u16::from(row))?,
        anchor.level,
    ))
}

/// A hero's army: its own stacks, or its type's starting army when the map gives none or only
/// empty slots (VCMI 1.7.5 `CGHeroInstance::initHero`: no stacks, the starting army).
fn army(
    version: Version,
    rules: &GameRuleset,
    index: usize,
    army: Option<&Army>,
) -> Result<HeroArmy, ImportError> {
    let none = if version == Version::RoE {
        u16::from(u8::MAX)
    } else {
        u16::MAX
    };
    let stacks = army
        .into_iter()
        .flatten()
        .zip(0u8..)
        .filter_map(|(stack, slot)| stack.map(|s| (s, slot)))
        .map(|(stack, slot)| {
            if (none - RANDOM_STACK_CODES..none).contains(&stack.creature) {
                return Err(Unsupported::RandomStack(index).into());
            }
            Ok(ArmyStack {
                slot,
                creature: creature(rules, stack.creature)?,
                count: u32::from(stack.count),
            })
        })
        .collect::<Result<Vec<_>, ImportError>>()?;
    Ok(if stacks.is_empty() {
        HeroArmy::Starting
    } else {
        HeroArmy::Custom(stacks)
    })
}

/// The creature type of an original creature id.
fn creature(rules: &GameRuleset, code: u16) -> Result<CreatureTypeId, ImportError> {
    let (_, key) = codes::CREATURES
        .binary_search_by_key(&code, |&(id, _)| id)
        .ok()
        .map(|i| codes::CREATURES[i])
        .ok_or(ImportError::UnknownCode {
            what: "creature",
            code,
        })?;
    rules
        .core()
        .creature_by_key(key)
        .ok_or(ImportError::MissingContent(key))
}
