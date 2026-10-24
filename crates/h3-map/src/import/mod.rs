//! The import (ADR-0015 point 4, E08 T5a, T6a and T12a): a raw map and the game's rules into
//! `h3_game::GameSetup`. It maps the format's codes to content keys and handles (the object
//! classes to object types, ADV-OBJ-002), draws nothing (a hero without its own army keeps the
//! starting-army placeholder, a pile without an amount draws it when the game is built) and
//! rejects what the current increment cannot represent with a typed [`Unsupported`] error. It checks no game
//! invariant: `GameSetup::validate` does, with the checks commands use. Notes:
//! `docs/formats/h3m.md`, "Import".

pub(crate) mod codes;

use std::fmt;

use h3_adventure::ObjectKind;
use h3_adventure::{Guards, MapSetup, MonsterSetup, ObjectSetup, Pos, TileMask, TileSetup};
use h3_content::{CreatureTypeId, Formation, ResourceId, SkillId, SpellId};
use h3_contracts::{ArmyStack, PlayerId};
use h3_game::{
    DEFAULT_DIFFICULTY, GameHeroSetup, GameRuleset, GameSetup, HeroArmy, HeroArtifactsSetup,
    HeroDetailsSetup, PlayerSetup,
};

use crate::model::{
    Army, Artifacts, Bits, Body, CustomHero, HeroBody, MonsterBody, Object, RawMap, Stack,
    Template, Version,
};

/// Object classes the import tells apart beside the typed ones (`codes::object_type`); every other
/// object is a blocker.
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
    /// The monster at this index holds an artifact in its treasure (artifacts on the map:
    /// increment 5's sketch, E08 decisions 22 and 48). Its static layer keeps it without the artifact.
    MonsterArtifact(usize),
}

impl Unsupported {
    /// The reason without its position, for counting maps by reason.
    pub const fn reason(self) -> &'static str {
        match self {
            Self::HeroAtMainTown(_) => "a hero generated at a main town",
            Self::RandomHero(_) => "a random hero",
            Self::HeroPlaceholder(_) => "a campaign hero placeholder",
            Self::RandomStack(_) => "a random stack in a hero's army or a pile's guards",
            Self::HeroInTown(_) => "a hero standing in a town",
            Self::MonsterArtifact(_) => "a monster with an artifact in its treasure",
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
    /// The object at this index has a subclass its class does not define (a resource pile of
    /// resource 7).
    Subclass(usize),
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
            Self::Subclass(object) => {
                write!(
                    f,
                    "object {object} has a subclass its class does not define"
                )
            }
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
/// map, its first guarded pile or monster with an artifact, then the hero objects in the map's
/// order.
pub fn import(map: &RawMap, rules: &GameRuleset) -> Result<GameSetup, ImportError> {
    let (players, colours) = players(map)?;
    let (static_map, unsupported) = static_layer(map, rules)?;
    if let Some(what) = unsupported {
        return Err(what.into());
    }
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
                let custom = map
                    .custom_heroes
                    .iter()
                    .find(|c| c.hero_type == body.hero_type);
                let hero = hero(
                    map.header.version,
                    rules,
                    &colours,
                    index,
                    &footprint(object, template),
                    (body, custom),
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
        difficulty: DEFAULT_DIFFICULTY,
        map: static_map,
        heroes,
        level_limit: map.header.level_limit,
        banned_skills: banned_skills(map, rules)?,
    })
}

/// The secondary skills the map bans (SoD's set, `docs/formats/h3m.md`; E08 T29): the skill of
/// each set bit's code, the original's skill ids (the pack order, HER-LVL-003); a set bit past
/// them is an unknown code.
fn banned_skills(map: &RawMap, rules: &GameRuleset) -> Result<Vec<SkillId>, ImportError> {
    let bits = &map.banned_skills;
    (0..bits.bytes().len().saturating_mul(8))
        .filter(|&code| bits.get(code))
        .map(|code| skill(rules, u8::try_from(code).unwrap_or(u8::MAX)))
        .collect()
}

/// Imports the static map (E08 T5a, T12a): the size, every tile's terrain, road and river, and
/// every object but the heroes with its footprint (ADV-OBJ-001) and, for the typed classes, its
/// object type and fixed amount (ADV-OBJ-002), a monster's data (ADV-MON-001, E08 T20a2a), a
/// special ground's tiles (ADV-GRD-001) and a pile's guards (ADV-PIK-004, E08 T21a); every other
/// object is a blocker (the whole [`import`] refuses a monster with an artifact, which this keeps
/// without it, and a pile's guards with a random stack, which this leaves unguarded).
///
/// # Errors
///
/// The first [`ImportError`] found: the map's size, the tiles, then the objects' templates and
/// types in the map's order.
pub fn import_map(map: &RawMap, rules: &GameRuleset) -> Result<MapSetup, ImportError> {
    static_layer(map, rules).map(|(setup, _)| setup)
}

/// [`import_map`] with what the first monster with an artifact or pile's guards with a random
/// stack makes unsupported.
fn static_layer(
    map: &RawMap,
    rules: &GameRuleset,
) -> Result<(MapSetup, Option<Unsupported>), ImportError> {
    let side = u16::try_from(map.terrain.size);
    let levels = u8::try_from(map.terrain.levels);
    let (Ok(side), Ok(levels)) = (side, levels) else {
        return Err(ImportError::MapSize(map.terrain.size, map.terrain.levels));
    };
    let tiles = tiles(map, rules)?;
    let mut unsupported = None;
    let mut placed = Vec::new();
    for (index, object, template) in objects(map)? {
        if matches!(template.class, HERO | RANDOM_HERO | HERO_PLACEHOLDER) {
            continue;
        }
        let (setup, what) = typed(rules, map.header.version, index, (object, template))?;
        unsupported = unsupported.or(what);
        placed.push(setup);
    }
    let setup = MapSetup {
        width: side,
        height: side,
        levels,
        tiles,
        objects: placed,
    };
    Ok((setup, unsupported))
}

/// An object with its footprint and, for a typed class, its object type and the amount the map
/// fixes (0 is none: drawn when the game is built), a monster's data, a special ground's tiles and
/// a pile's guards ([`guards`]); what it makes unsupported: a monster with an artifact, kept
/// without it, or a pile's guards with a random stack, kept unguarded.
// rule: ADV-OBJ-002
fn typed(
    rules: &GameRuleset,
    version: Version,
    index: usize,
    (object, template): (&Object, &Template),
) -> Result<(ObjectSetup, Option<Unsupported>), ImportError> {
    let mut setup = footprint(object, template);
    let Some(key) = codes::object_type(template.class, template.subclass) else {
        return Ok((setup, None));
    };
    let key = key.ok_or(ImportError::Subclass(index))?;
    let object_type = rules
        .adventure()
        .object_type_by_key(key)
        .ok_or(ImportError::MissingContent(key))?;
    let mut unsupported = None;
    match &object.body {
        Body::Resource { message, amount } => {
            setup.amount = (*amount != 0).then_some(*amount);
            if let Some(message) = message {
                let army = message.guards.as_deref().unwrap_or_default();
                if army
                    .iter()
                    .flatten()
                    .any(|s| random_stack(version, s.creature))
                {
                    unsupported = Some(Unsupported::RandomStack(index));
                } else {
                    setup.guards = guards(rules, army, !message.text.as_bytes().is_empty())?;
                }
            }
        }
        Body::Monster(body) => {
            let (monster, artifact) = monster(rules, template, body)?;
            setup.monster = Some(monster);
            unsupported = artifact.then_some(Unsupported::MonsterArtifact(index));
        }
        _ => {}
    }
    if matches!(
        rules
            .adventure()
            .object_type(object_type)
            .map(|ty| &ty.kind),
        Some(ObjectKind::Ground { .. })
    ) {
        setup.drawn = ground_tiles(codes::ground_size(template.def.as_bytes()));
    }
    setup.object_type = Some(object_type);
    Ok((setup, unsupported))
}

/// A wandering monster's data (ADV-MON-001, E08 T20a2a; `docs/formats/h3m.md`, "Import"): the
/// creature of class 54's subclass (a random monster's is drawn when the game is built), the count
/// (0 drawn), the disposition code, never flees, does not grow, and the treasure's resources that
/// are above 0 (its message is presentation); whether the treasure holds an artifact.
// rule: ADV-MON-001
fn monster(
    rules: &GameRuleset,
    template: &Template,
    body: &MonsterBody,
) -> Result<(MonsterSetup, bool), ImportError> {
    let creature = (template.class == codes::MONSTER)
        .then(|| {
            let code = u16::try_from(template.subclass).unwrap_or(u16::MAX);
            creature(rules, code)
        })
        .transpose()?;
    let treasure = body.treasure.as_ref();
    let resources = treasure.map_or(Ok(Vec::new()), |t| {
        codes::RESOURCES
            .iter()
            .zip(t.resources)
            .filter_map(|(&key, amount)| {
                u32::try_from(amount)
                    .ok()
                    .filter(|&a| a > 0)
                    .map(|a| (key, a))
            })
            .map(|(key, amount)| {
                rules
                    .core()
                    .resources()
                    .iter()
                    .position(|r| r.key.as_str() == key)
                    .and_then(ResourceId::from_index)
                    .map(|resource| (resource, amount))
                    .ok_or(ImportError::MissingContent(key))
            })
            .collect()
    })?;
    Ok((
        MonsterSetup {
            creature,
            count: u32::from(body.count),
            disposition: body.disposition,
            never_flees: body.never_flees,
            no_growth: body.no_growth,
            treasure: resources,
        },
        treasure.is_some_and(|t| t.artifact.is_some()),
    ))
}

/// A pile's guards (ADV-PIK-004, E08 T21a; `docs/formats/h3m.md`, "Import"): the guards record's
/// stacks by slot, none random (the caller's check), and whether the message asks (it has a text;
/// the text itself is presentation). A record without stacks is a plain pile (survey
/// `E08-battles.md` §5.1: the original tests the stack count, 0x4A47E0), as one without a record.
// rule: ADV-PIK-004
fn guards(
    rules: &GameRuleset,
    army: &[Option<Stack>],
    question: bool,
) -> Result<Option<Guards>, ImportError> {
    let army = stacks(rules, army)?;
    Ok((!army.is_empty()).then_some(Guards { army, question }))
}

/// The tiles of a picture `width` columns wide and `height` rows high, its bottom right tile the
/// anchor (ADV-GRD-001).
fn ground_tiles((width, height): (u8, u8)) -> TileMask {
    let row = (1u64 << width.min(TileMask::COLUMNS)) - 1;
    TileMask::from_bits((0..height.min(TileMask::ROWS)).fold(0, |bits, r| {
        bits | row << (u32::from(r) * u32::from(TileMask::COLUMNS))
    }))
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

/// Every tile's terrain, road and river as the ruleset's handles, and whether it is on the coast
/// (since v5).
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
                coast: tile.flags & codes::COAST != 0,
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
        object_type: None,
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
    }
}

/// A placed hero: its owner, type, tile, army, formation and the customisations of its details
/// (E08 T18b), the body's over its hero type's map-wide entry's, field by field (VCMI 1.7.5
/// `CMapLoaderH3M::readHero`: "using the latter set").
fn hero(
    version: Version,
    rules: &GameRuleset,
    colours: &[Option<PlayerId>; 8],
    index: usize,
    footprint: &ObjectSetup,
    (body, custom): (&HeroBody, Option<&CustomHero>),
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
    let details = details(version, rules, hero_type, (body, custom))?;
    Ok(GameHeroSetup {
        hero_type,
        owner,
        pos: stands_on(footprint).ok_or(ImportError::HeroTile(index))?,
        army: army(version, rules, index, body.army.as_ref())?,
        formation: if body.formation == 0 {
            Formation::Loose
        } else {
            Formation::Tight
        },
        details,
    })
}

/// AB's one spell byte: no spell (the spell book empty), or the hero type's own.
const NO_SPELL: u8 = 0xFF;
const TYPE_SPELL: u8 = 0xFE;

/// The customisations of a placed hero's details (E08 T18b; `docs/formats/h3m.md`, "Import"):
/// experience (before SoD 0 is the default; passed as it is since E08 T29, the game gives its
/// levels at load, GAM-SET-002), primary skills (clamped to the hero parameters' bounds, the
/// clamp of the original's carried-over hero, `carried_hero_setup` 0x486590, survey
/// `E08-battles.md` §6.3: the placed hero's are copied as they are, `placed_hero_setup` 0x4D8BF1,
/// survey `E08-progression.md` §3.1, and no shipped map has one above 9, §6.1), secondary skills,
/// artifacts worn and in the backpack, spells (SoD's set; AB's one spell beside the type's,
/// `0xFF` none, `0xFE` the type's); the body's field over the map-wide entry's.
fn details(
    version: Version,
    rules: &GameRuleset,
    hero_type: h3_content::HeroTypeId,
    (body, custom): (&HeroBody, Option<&CustomHero>),
) -> Result<HeroDetailsSetup, ImportError> {
    let experience = body
        .experience
        .filter(|&e| version == Version::Sod || e != 0)
        .or_else(|| custom.and_then(|c| c.experience));
    let params = rules.core().hero_params();
    let primary = body
        .primary_skills
        .or_else(|| custom.and_then(|c| c.primary_skills))
        .map(|skills| {
            let mut out = skills;
            for (i, value) in out.iter_mut().enumerate() {
                let (low, high) =
                    params.map_or((0, u8::MAX), |p| (p.primary_min[i], p.primary_max));
                *value = (*value).clamp(low, high);
            }
            out
        });
    let skills = body
        .secondary_skills
        .as_ref()
        .or_else(|| custom.and_then(|c| c.secondary_skills.as_ref()))
        .map(|list| {
            list.iter()
                .map(|&(code, level)| Ok((skill(rules, code)?, level)))
                .collect::<Result<Vec<_>, ImportError>>()
        })
        .transpose()?;
    let artifacts = body
        .artifacts
        .as_ref()
        .or_else(|| custom.and_then(|c| c.artifacts.as_ref()))
        .map(|a| hero_artifacts(rules, a))
        .transpose()?;
    let spells = match (body.spells.as_ref(), body.spell) {
        (Some(bits), _) => Some(spell_set(rules, bits)?),
        (None, Some(NO_SPELL)) => Some(Vec::new()),
        (None, Some(TYPE_SPELL)) | (None, None) => custom
            .and_then(|c| c.spells.as_ref())
            .map(|bits| spell_set(rules, bits))
            .transpose()?,
        (None, Some(code)) => {
            let mut own = rules
                .core()
                .hero_type(hero_type)
                .map(|t| t.spells.clone())
                .unwrap_or_default();
            let spell = spell(rules, code)?;
            if !own.contains(&spell) {
                own.push(spell);
            }
            Some(own)
        }
    };
    Ok(HeroDetailsSetup {
        experience,
        primary,
        skills,
        artifacts,
        spells,
    })
}

/// A hero's stored artifacts (since v9, E08 T49a): each worn entry's artifact equipped at its
/// position (ADR-0013 point 1: the map's entry `i` is [`codes::WORN`]'s slot and rank, the
/// content's position the rank-th of that slot's in `h3_game::positions`), in the map's order and
/// only where the position takes it (`h3_game::equip`: empty, of the artifact's slot, the locks
/// fitting), else left out: the original's loader equips entry by entry and drops what its
/// position refuses (`placed_hero_setup` 0x4D8D63 through `hero_equip_artifact` 0x4E2A00 and
/// `hero_can_place_artifact` 0x4E2550; the fifth misc refused before SoD), as VCMI's
/// (`loadArtifactToSlot`; the Tutorial's second hero's Shackles of War on the left hand). Since
/// v10 (E08 T49b) an equipped Titan's Thunder adds a Spell Book on the empty book position, as
/// the original's equip does (0x4E2A64, HER-ART-007): the map's own book entry, later in its
/// order, then finds the position taken. The backpack in order.
fn hero_artifacts(
    rules: &GameRuleset,
    stored: &Artifacts,
) -> Result<HeroArtifactsSetup, ImportError> {
    let core = rules.core();
    let positions = h3_game::positions(core);
    let mut worn = vec![None; positions.len()];
    for (&(key, rank), code) in codes::WORN.iter().zip(&stored.worn) {
        let Some(&code) = code.as_ref() else { continue };
        let id = artifact(rules, code)?;
        let slot = core.artifact_slot_by_key(key);
        let at = (0..positions.len())
            .filter(|&p| slot == positions.get(p).copied())
            .nth(rank);
        if let Some(at) = at {
            h3_game::equip(&mut worn, core, id.into(), h3_game::Position(at));
        }
    }
    let backpack = stored
        .backpack
        .iter()
        .flatten()
        .map(|&code| artifact(rules, code).map(Into::into))
        .collect::<Result<Vec<_>, ImportError>>()?;
    Ok(HeroArtifactsSetup { worn, backpack })
}

/// The spells of a stored set: bit `i` of byte `i / 8` is spell `i`.
fn spell_set(rules: &GameRuleset, bits: &Bits) -> Result<Vec<SpellId>, ImportError> {
    (0u16..)
        .zip(
            bits.0
                .iter()
                .flat_map(|&byte| (0..8).map(move |bit| byte & (1 << bit) != 0)),
        )
        .filter(|&(_, set)| set)
        .map(|(code, _)| spell(rules, code))
        .collect()
}

/// A content handle by original id through a code table.
fn coded<T>(
    table: &[&'static str],
    what: &'static str,
    code: u16,
    by_key: impl FnOnce(&'static str) -> Option<T>,
) -> Result<T, ImportError> {
    let key = table
        .get(usize::from(code))
        .copied()
        .ok_or(ImportError::UnknownCode { what, code })?;
    by_key(key).ok_or(ImportError::MissingContent(key))
}

fn skill(rules: &GameRuleset, code: u8) -> Result<SkillId, ImportError> {
    coded(&codes::SKILLS, "secondary skill", code.into(), |k| {
        rules.core().skill_by_key(k)
    })
}

fn spell(rules: &GameRuleset, code: impl Into<u16>) -> Result<SpellId, ImportError> {
    coded(&codes::SPELLS, "spell", code.into(), |k| {
        rules.core().spell_by_key(k)
    })
}

fn artifact(rules: &GameRuleset, code: u16) -> Result<h3_content::ArtifactTypeId, ImportError> {
    coded(&codes::ARTIFACTS, "artifact", code, |k| {
        rules.core().artifact_by_key(k)
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
    let army = army.map(Vec::as_slice).unwrap_or_default();
    if army
        .iter()
        .flatten()
        .any(|s| random_stack(version, s.creature))
    {
        return Err(Unsupported::RandomStack(index).into());
    }
    let stacks = stacks(rules, army)?;
    Ok(if stacks.is_empty() {
        HeroArmy::Starting
    } else {
        HeroArmy::Custom(stacks)
    })
}

/// Whether `code` is a random stack's code in `version`'s files: one of the
/// [`RANDOM_STACK_CODES`] below the format's "none".
fn random_stack(version: Version, code: u16) -> bool {
    let none = if version == Version::RoE {
        u16::from(u8::MAX)
    } else {
        u16::MAX
    };
    (none - RANDOM_STACK_CODES..none).contains(&code)
}

/// An army's stacks as stored, by slot, its empty slots left out; none random (the callers'
/// check).
fn stacks(rules: &GameRuleset, army: &[Option<Stack>]) -> Result<Vec<ArmyStack>, ImportError> {
    army.iter()
        .zip(0u8..)
        .filter_map(|(stack, slot)| stack.map(|s| (s, slot)))
        .map(|(stack, slot)| {
            Ok(ArmyStack {
                slot,
                creature: creature(rules, stack.creature)?,
                count: u32::from(stack.count),
            })
        })
        .collect()
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
