//! The import (ADR-0015 point 4, E08 T5a, T6a and T12a): a raw map and the game's rules into
//! `h3_game::GameSetup`. It maps the format's codes to content keys and handles (the object
//! classes to object types, ADV-OBJ-002), draws nothing (a hero without its own army keeps the
//! starting-army placeholder, a pile without an amount draws it when the game is built) and
//! rejects what the current increment cannot represent with a typed [`Unsupported`] error. It checks no game
//! invariant: `GameSetup::validate` does, with the checks commands use. Notes:
//! `docs/formats/h3m.md`, "Import".

pub(crate) mod codes;

use std::fmt;

use h3_adventure::{
    ArtifactInstance, BoatSetup, EventSetup, Guards, HeroReward, Holds, MapSetup, MonsterSetup,
    ObjectSetup, Pos, PrimarySkill, RewardRecord, ScholarGift, TileMask, TileSetup,
};
use h3_adventure::{ObjectKind, Passage};
use h3_content::{CreatureTypeId, Formation, ResourceId, SkillId, SpellId};
use h3_contracts::{ArmyStack, HeroSex, PlayerId};
use h3_game::{
    DEFAULT_DIFFICULTY, GameHeroSetup, GameRuleset, GameSetup, HeroArmy, HeroArtifactsSetup,
    HeroDetailsSetup, PlayerSetup,
};

use crate::model::{
    Army, Artifacts, Bits, Body, CustomHero, HeroBody, Message, MonsterBody, Object, RawMap,
    Rewards, Stack, Template, Version,
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
    /// The Pandora's Box or event at this index gives more creature stacks than an army's slots
    /// (A-24, always fixed: E08 decision 133; T61a). Its static layer keeps it without its
    /// creatures.
    RewardArmy(usize),
    /// The hero object at this index stands on water (ADV-SEA-007, E08 T72; D-44, decisions 163
    /// and 178: batch E8's E8-5 showed the original keeping such a hero on foot without a boat,
    /// VCMI gives it one; none of the shipped maps has one).
    HeroOnWater(usize),
}

impl Unsupported {
    /// The reason without its position, for counting maps by reason.
    pub const fn reason(self) -> &'static str {
        match self {
            Self::HeroAtMainTown(_) => "a hero generated at a main town",
            Self::RandomHero(_) => "a random hero",
            Self::HeroPlaceholder(_) => "a campaign hero placeholder",
            Self::RandomStack(_) => {
                "a random stack in a hero's army, an object's guards or a reward's creatures"
            }
            Self::HeroInTown(_) => "a hero standing in a town",
            Self::RewardArmy(_) => "a reward of more creature stacks than an army's slots",
            Self::HeroOnWater(_) => "a hero placed on water",
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
    /// The boat object at this index has a template without exactly one visitable tile on the
    /// map, the tile the boat lies on (E08 T72).
    BoatTile(usize),
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
            Self::BoatTile(object) => {
                write!(f, "boat object {object} does not name one tile to lie on")
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
/// map, its first object with guards or reward creatures it cannot represent, then the hero
/// objects in the map's order.
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
                // rule: ADV-SEA-007
                if on_water(rules, &static_map, hero.pos) {
                    return Err(Unsupported::HeroOnWater(index).into());
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
        banned_spells: spell_set(rules, &map.banned_spells)?,
        banned_artifacts: banned_artifacts(map, rules)?,
    })
}

/// The secondary skills the map bans (SoD's set, `docs/formats/h3m.md`; E08 T29): the skill of
/// each set bit's code, the original's skill ids (the pack order, HER-LVL-003); a set bit past
/// them is an unknown code.
fn banned_skills(map: &RawMap, rules: &GameRuleset) -> Result<Vec<SkillId>, ImportError> {
    skill_set(rules, &map.banned_skills)
}

/// The skills of a stored set: the skill of each set bit's code; a set bit past them is an
/// unknown code.
fn skill_set(rules: &GameRuleset, bits: &Bits) -> Result<Vec<SkillId>, ImportError> {
    (0..bits.bytes().len().saturating_mul(8))
        .filter(|&code| bits.get(code))
        .map(|code| skill(rules, u8::try_from(code).unwrap_or(u8::MAX)))
        .collect()
}

/// Imports the static map (E08 T5a, T12a): the size, every tile's terrain, road and river, and
/// every object but the heroes with its footprint (ADV-OBJ-001) and, for the typed classes, its
/// object type and fixed amount (ADV-OBJ-002), a monster's data (ADV-MON-001, E08 T20a2a), a
/// special ground's tiles (ADV-GRD-001), a guarded object's guards (ADV-PIK-004, E08 T21a) and,
/// since E08 T61a, what increment 7's objects hold; every other object is a blocker (the whole
/// [`import`] refuses guards or reward creatures with a random stack and reward creatures beyond
/// an army's slots, which this leaves out). Since E08 T72 the boats (class 8) are no objects but
/// the map's boats, each on its template's visitable tile with its subclass as its type
/// (ADV-SEA-001), and a ground of the sea and a hole lie on their pictures' tiles.
///
/// # Errors
///
/// The first [`ImportError`] found: the map's size, the tiles, then the objects' templates and
/// types and the boats' tiles and types in the map's order.
pub fn import_map(map: &RawMap, rules: &GameRuleset) -> Result<MapSetup, ImportError> {
    static_layer(map, rules).map(|(setup, _)| setup)
}

/// [`import_map`] with what the first object's guards or reward creatures with a random stack, or
/// reward creatures beyond an army's slots, make unsupported.
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
    let mut boats = Vec::new();
    for (index, object, template) in objects(map)? {
        if matches!(template.class, HERO | RANDOM_HERO | HERO_PLACEHOLDER) {
            continue;
        }
        if template.class == codes::BOAT {
            boats.push(boat(index, (object, template))?);
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
        boats,
    };
    Ok((setup, unsupported))
}

/// An object with its footprint and, for a typed class, its object type and the amount the map
/// fixes (0 is none: drawn when the game is built), a monster's data, a special ground's tiles, a
/// guarded object's guards ([`message_guards`]) and, since E08 T61a, what the map puts in an
/// object of increment 7's kinds ([`Holds`]); what it makes unsupported: guards with a random
/// stack, kept unguarded, or a reward's creatures with a random stack or beyond an army's slots,
/// kept without them ([`record`]).
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
    let has_text = |m: &Message| !m.text.as_bytes().is_empty();
    let guarded = |message: Option<&Message>, ask: &dyn Fn(&Message) -> bool| {
        message_guards(rules, version, index, message, ask)
    };
    let mut unsupported = None;
    match &object.body {
        Body::Resource { message, amount } => {
            setup.amount = (*amount != 0).then_some(*amount);
            (setup.guards, unsupported) = guarded(message.as_ref(), &has_text)?;
        }
        Body::Monster(body) => setup.monster = Some(monster(rules, template, body)?),
        // E08 T61a (increment 7's design note §7): a placed artifact's is its subclass; a guarded
        // artifact always asks (D-29), a random one is drawn when the game is built.
        Body::Artifact(message) => {
            if template.class == codes::ARTIFACT {
                let code = u16::try_from(template.subclass).unwrap_or(u16::MAX);
                setup.holds = Holds::Artifact(artifact(rules, code)?.into());
            }
            (setup.guards, unsupported) = guarded(message.as_ref(), &|_| true)?;
        }
        // The spell is the format's u32's low byte (`load_scroll_body` 0x4FF2F0); its guards ask
        // only with a text (survey `E08-rewards.md` §4).
        Body::Scroll {
            message,
            spell: code,
        } => {
            let scroll = artifact(rules, codes::SPELL_SCROLL)?;
            let spell = spell(rules, code.to_le_bytes()[0])?;
            setup.holds = Holds::Artifact(ArtifactInstance::scroll(scroll, spell));
            (setup.guards, unsupported) = guarded(message.as_ref(), &has_text)?;
        }
        Body::Shrine { spell: code } => {
            setup.holds = Holds::Spell(
                (*code != u32::from(u8::MAX))
                    .then(|| spell(rules, u16::try_from(*code).unwrap_or(u16::MAX)))
                    .transpose()?,
            );
        }
        Body::WitchHut(mask) => setup.holds = Holds::Skills(witch_hut(version, rules, mask)?),
        Body::Scholar { kind, code } => setup.holds = Holds::Gift(scholar(rules, *kind, *code)?),
        // A box's and an event's guards are announced, never asked (survey §7.2, §7.3).
        Body::Pandora { message, rewards } => {
            let (record, what) = record(rules, version, index, rewards)?;
            setup.holds = Holds::Record(Box::new(record));
            (setup.guards, unsupported) = guarded(message.as_ref(), &|_| false)?;
            unsupported = unsupported.or(what);
        }
        Body::Event(event) => {
            let (record, what) = record(rules, version, index, &event.rewards)?;
            setup.holds = Holds::Event(Box::new(EventSetup {
                record,
                colours: event.players,
                computer: event.computer,
                remove: event.remove_after_visit,
            }));
            (setup.guards, unsupported) = guarded(event.message.as_ref(), &|_| false)?;
            unsupported = unsupported.or(what);
        }
        _ => {}
    }
    // E08 T72: a ground of the sea and a hole lie on their pictures' tiles too.
    if matches!(
        rules
            .adventure()
            .object_type(object_type)
            .map(|ty| &ty.kind),
        Some(ObjectKind::Ground { .. } | ObjectKind::SeaGround | ObjectKind::Hole)
    ) {
        setup.drawn = ground_tiles(codes::ground_size(template.def.as_bytes()));
    }
    setup.object_type = Some(object_type);
    Ok((setup, unsupported))
}

/// A boat (ADV-SEA-001, E08 T72; survey `E08-sea.md` §2): its template's one visitable tile and
/// its subclass as its type (the original's loader reads no body for class 8 and creates the boat
/// on the object's entrance tile, 0x502F28); a subclass beyond the original's three is one the
/// class does not define.
// rule: ADV-SEA-001
fn boat(index: usize, (object, template): (&Object, &Template)) -> Result<BoatSetup, ImportError> {
    let tile = stands_on(&footprint(object, template)).ok_or(ImportError::BoatTile(index))?;
    let kind = (template.subclass < codes::BOAT_KINDS)
        .then(|| u8::try_from(template.subclass).unwrap_or(u8::MAX))
        .ok_or(ImportError::Subclass(index))?;
    Ok(BoatSetup { tile, kind })
}

/// Whether `pos` lies on water in the imported static map (a tile off it: no).
fn on_water(rules: &GameRuleset, map: &MapSetup, pos: Pos) -> bool {
    let index = (usize::from(pos.level) * usize::from(map.height) + usize::from(pos.y))
        * usize::from(map.width)
        + usize::from(pos.x);
    (pos.x < map.width && pos.y < map.height)
        && map.tiles.get(index).is_some_and(|tile| {
            matches!(
                rules.adventure().passage(tile.terrain),
                Some(Passage::Water(_))
            )
        })
}

/// A wandering monster's data (ADV-MON-001, E08 T20a2a; `docs/formats/h3m.md`, "Import"): the
/// creature of class 54's subclass (a random monster's is drawn when the game is built), the count
/// (0 drawn), the disposition code, never flees, does not grow, and the treasure's resources that
/// are above 0 (its message is presentation) and, since E08 T61a, its artifact (ADV-MON-011).
// rule: ADV-MON-001
fn monster(
    rules: &GameRuleset,
    template: &Template,
    body: &MonsterBody,
) -> Result<MonsterSetup, ImportError> {
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
            .map(|(key, amount)| Ok((resource(rules, key)?, amount)))
            .collect::<Result<_, ImportError>>()
    })?;
    Ok(MonsterSetup {
        creature,
        count: u32::from(body.count),
        disposition: body.disposition,
        never_flees: body.never_flees,
        no_growth: body.no_growth,
        treasure: resources,
        artifact: treasure
            .and_then(|t| t.artifact)
            .map(|code| artifact(rules, code))
            .transpose()?,
    })
}

/// The resource of a key of [`codes::RESOURCES`].
fn resource(rules: &GameRuleset, key: &'static str) -> Result<ResourceId, ImportError> {
    rules
        .core()
        .resources()
        .iter()
        .position(|r| r.key.as_str() == key)
        .and_then(ResourceId::from_index)
        .ok_or(ImportError::MissingContent(key))
}

/// A guarded object's guards from its message (ADV-PIK-004; E08 T61a beyond piles): none without
/// a message or stacks; whether they ask, `ask`'s answer for the message (a pile and a scroll:
/// it has a text; an artifact: always, D-29; a box and an event: never); with a random stack,
/// none, and the object unsupported.
fn message_guards(
    rules: &GameRuleset,
    version: Version,
    index: usize,
    message: Option<&Message>,
    ask: &dyn Fn(&Message) -> bool,
) -> Result<(Option<Guards>, Option<Unsupported>), ImportError> {
    let Some(message) = message else {
        return Ok((None, None));
    };
    let army = message.guards.as_deref().unwrap_or_default();
    if army
        .iter()
        .flatten()
        .any(|s| random_stack(version, s.creature))
    {
        return Ok((None, Some(Unsupported::RandomStack(index))));
    }
    Ok((guards(rules, army, ask(message))?, None))
}

/// A Pandora's Box's or an event's reward record (ADV-RWD-001, E08 T61a; survey `E08-rewards.md`
/// §7.1, `docs/formats/h3m.md`, "Import"): the map's values as they are (the experience's u32,
/// the primary skills' bytes as signed, the mana, morale and luck), its skills, artifacts and
/// spells by code, the resources that are not 0, the creatures by slot in the list's order. What
/// it makes unsupported, its creatures left out: a random stack among them, or more stacks than an
/// army's slots (A-24).
// rule: ADV-OBJ-002
fn record(
    rules: &GameRuleset,
    version: Version,
    index: usize,
    rewards: &Rewards,
) -> Result<(RewardRecord, Option<Unsupported>), ImportError> {
    let hero = HeroReward {
        experience: rewards.experience,
        primary: rewards.primary.map(|byte| i8::from_le_bytes([byte])),
        skills: rewards
            .skills
            .iter()
            .map(|&(code, level)| Ok((skill(rules, code)?, level)))
            .collect::<Result<_, ImportError>>()?,
        mana: rewards.mana,
        morale: rewards.morale,
        luck: rewards.luck,
        artifacts: rewards
            .artifacts
            .iter()
            .map(|&code| artifact(rules, code))
            .collect::<Result<_, _>>()?,
        spells: rewards
            .spells
            .iter()
            .map(|&code| spell(rules, code))
            .collect::<Result<_, _>>()?,
    };
    let resources = codes::RESOURCES
        .iter()
        .zip(rewards.resources)
        .filter(|&(_, amount)| amount != 0)
        .map(|(&key, amount)| Ok((resource(rules, key)?, amount)))
        .collect::<Result<_, ImportError>>()?;
    let slots = usize::from(rules.adventure().params().army_slots());
    let (creatures, unsupported) = if rewards
        .creatures
        .iter()
        .any(|s| random_stack(version, s.creature))
    {
        (Vec::new(), Some(Unsupported::RandomStack(index)))
    } else if rewards.creatures.len() > slots {
        (Vec::new(), Some(Unsupported::RewardArmy(index)))
    } else {
        let army: Army = rewards.creatures.iter().copied().map(Some).collect();
        (stacks(rules, &army)?, None)
    };
    Ok((
        RewardRecord {
            hero,
            resources,
            creatures,
        },
        unsupported,
    ))
}

/// A Witch Hut's skills (ADV-TCH-005, E08 T61a; survey `E08-rewards.md` §6.6): none on an RoE
/// map, which stores no mask (its type's skills by the profile, A-21); else the skill of each set
/// bit's code, and every skill for an empty mask (the original's new game draws among all 28
/// then, 0x4C207D).
fn witch_hut(
    version: Version,
    rules: &GameRuleset,
    mask: &Bits,
) -> Result<Option<Vec<SkillId>>, ImportError> {
    if version == Version::RoE {
        return Ok(None);
    }
    let skills = skill_set(rules, mask)?;
    if !skills.is_empty() {
        return Ok(Some(skills));
    }
    (0..codes::SKILLS.len())
        .map(|code| skill(rules, u8::try_from(code).unwrap_or(u8::MAX)))
        .collect::<Result<_, _>>()
        .map(Some)
}

/// A Scholar's gift (ADV-TCH-006, E08 T61a; survey `E08-rewards.md` §6.7): none for the kind
/// −1 (drawn when the game is built); kind 0 the primary skill of its code (0 to 3), 1 the
/// secondary skill, 2 the spell; another kind an unknown code.
fn scholar(rules: &GameRuleset, kind: i8, code: u8) -> Result<Option<ScholarGift>, ImportError> {
    let unknown = |what| ImportError::UnknownCode {
        what,
        code: code.into(),
    };
    Ok(Some(match kind {
        -1 => return Ok(None),
        0 => ScholarGift::Primary(
            *PrimarySkill::ALL
                .get(usize::from(code))
                .ok_or_else(|| unknown("primary skill"))?,
        ),
        1 => ScholarGift::Skill(skill(rules, code)?),
        2 => ScholarGift::Spell(spell(rules, code)?),
        _ => {
            return Err(ImportError::UnknownCode {
                what: "scholar kind",
                code: kind.to_le_bytes()[0].into(),
            });
        }
    }))
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

/// The colours that play, in colour order, and the player each colour is; a colour a human may
/// play counts as human (GAM-BAT-010, E08 T54: the map's player kinds; ADV-Q27).
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
        // rule: GAM-BAT-010
        players.push(PlayerSetup {
            colour,
            human: player.human,
        });
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
        scenery: codes::scenery(template.class),
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
        holds: Holds::Nothing,
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
/// `0xFF` none, `0xFE` the type's), since E08 T54 the sex ([`hero_sex`]); the body's field over
/// the map-wide entry's.
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
    // rule: GAM-BAT-012
    let sex = hero_sex(body.sex).or_else(|| custom.and_then(|c| hero_sex(c.sex)));
    Ok(HeroDetailsSetup {
        experience,
        primary,
        skills,
        artifacts,
        spells,
        sex,
    })
}

/// A hero's sex by the map's code (AB and SoD; E08 T54): 0 male, 1 female, any other (`0xFF`)
/// the hero type's own.
const fn hero_sex(code: u8) -> Option<HeroSex> {
    match code {
        0 => Some(HeroSex::Male),
        1 => Some(HeroSex::Female),
        _ => None,
    }
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

/// The artifacts the map bans (AB's and SoD's set, `docs/formats/h3m.md`; GAM-SET-003, E08
/// T61b): the artifact of each set bit's code among the map's edition's artifacts that the
/// codes name. The bits past them are the set's padding (an AB map's last 7) or the original's
/// three unused records 141 to 143, which the editor bans by default on a SoD map.
fn banned_artifacts(
    map: &RawMap,
    rules: &GameRuleset,
) -> Result<Vec<h3_content::ArtifactTypeId>, ImportError> {
    // rule: GAM-SET-003
    let codes = map.header.version.artifacts().min(codes::ARTIFACTS.len());
    (0..codes)
        .filter(|&code| map.banned_artifacts.get(code))
        .map(|code| artifact(rules, u16::try_from(code).unwrap_or(u16::MAX)))
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
