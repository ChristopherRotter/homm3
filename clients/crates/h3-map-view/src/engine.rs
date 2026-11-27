//! The engine's games (E24 M4 T3a, *provisional*; ADR-0020 point 8; `docs/epics/E24-design.md`
//! §5.9): a whole game's record (`h3-server record`'s file, the record format's: its first line a
//! [`GameRecordHeader`], every further line one numbered server message the player was sent)
//! read, checked against the viewer's own rules and the map file it draws, and its first
//! observation turned into a live part as a capture of the original keeps it (`h3map.py still`'s
//! `live.toml`: `viewer`, `defs` and `classes`, `owners`, `heroes`, `boats`, `explored`; since
//! T3b1 the panels' `kingdom`, `status`, `realm` and `[chosen]`, since T3b2 the mouse-over's
//! `[hover]`), which the binary's `render --record` applies as `render --live` applies a
//! capture's (or, with `--check`, compares with it).
//!
//! - **The record is this build's game:** the manifest the protocol makes of the viewer's rules
//!   and of the map file (`h3_protocol::map::game::manifest`, the map by its content hash and the
//!   import's version) equals the record's, and the engine's import takes the map (MCL-Q21 (b):
//!   the viewer's load refuses a map the import refuses).
//! - **The record is the whole game's** (its header's `first` is 1, its first message the game's
//!   snapshot): a server's bounded record that let its oldest go begins at a later snapshot,
//!   which the start's new-game rules (heroes facing right) would misread, so it is refused
//!   ([`Refused::Cut`]); showing one from there is not M4's.
//! - **A live start** (E24 M5 T3b, [`live`], *provisional*): the hot seat's host sets a human's
//!   view up from its record since it last left it, its first message a snapshot of the view then
//!   (a later one than the game's), with what the host saw of each hero's facing ([`Live`]): the
//!   cut is let through, every hero the host did not see facing right as at a new game. The view
//!   names no facing (MCL-Q58): the host's is the last the viewer's timeline drew.
//! - **Each object of the view** (in tile order, without an index: `docs/06-security.md`) is
//!   found in the map's file by its type, anchor and masks (two alike special grounds either
//!   way), else as a random object at its anchor (a random dwelling's also one column right of
//!   it, the Angel Portal's shift); an object the file does not hold there refuses the record. The
//!   file's objects pair with the import's static objects in order, the heroes and boats left
//!   out (`h3_map::import_map`).
//! - **What a random object became** (design §2.7): a random resource the pile of its drawn
//!   resource (`avtwood0` … `avtgold0`, 0x677958), a random monster the last template of class 54
//!   with its creature (0x505EA0). A random artifact or dwelling keeps its random template
//!   (`MCL-STUB-1`: the view names no drawn artifact, a dwelling's class and subclass are the
//!   import's table).
//! - **A town's picture** (T3b2) by its faction and its view's fort and Capitol, a random town a
//!   town of its faction (class 98); a town the view does not show by the map's buildings at a
//!   new game, as the capture's.
//! - **The owners, heroes and boats** by colour (the view names players: its `players` give their
//!   colours); a hero's class its type's, facing right as the original's new game has them (the
//!   engine keeps no facing: M4 T4 takes it from the steps), its boat's flag its owner's; an
//!   object the view does not show keeps the file's owner (under the fog it is not drawn).
//! - **The explored tiles** as the view's bits, `size × levels` rows of `size` tiles.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use h3_adventure::{ObjectKind, ObjectSetup, ObjectTypeId, Pickup};
use h3_game::{GameRuleset, GameSetup};
use h3_kernel::Handle;
use h3_kernel::hash::StateHasher;
use h3_map::model::Body;
use h3_map::writer::ObjectTemplate;
use h3_map::{Limits, MAP_IMPORT_VERSION, RawMap};
use h3_protocol::map::game::manifest;
use h3_protocol::record::{self, RecordError};
use h3_protocol::wire::game::{GameObservation, GameRecordHeader, MapIdentity, Object};
use h3_protocol::wire::{ServerMessage, WireU64};
use h3_protocol::{PROTOCOL_VERSION, RECORD_FORMAT_VERSION};
use toml::{Table, Value};

use crate::art::MapArt;
use crate::live::{BOAT_CLASS, HERO_CLASS, TOWN_CLASS, built_at_start, owner};
use crate::objects;
use crate::record::Records;
use crate::tables::{CAPITOL, FORT, TownArt};

mod hover;
mod panels;
mod played;
pub mod visits;

pub use played::{
    Battled, Command, Held, Leveled, OtherEnd, OtherStep, OtherTurn, OtherWalk, Played, REVISIT,
    Visit, pickup, played,
};

/// A random hero (class 70): the import places it as a hero, not a static object.
const RANDOM_HERO: u32 = 70;
/// A hero's placeholder (class 214), placed as a hero too.
const HERO_PLACEHOLDER: u32 = 214;
/// A random resource (class 76).
const RANDOM_RESOURCE: u32 = 76;
/// A resource pile (class 79), what a random resource becomes.
const PILE: u32 = 79;
/// A wandering monster (class 54), what a random monster becomes.
const MONSTER: u32 = 54;
/// The random monsters' classes: any level, levels 1 to 4 (71 to 75), 5 to 7 (162 to 164).
const RANDOM_MONSTERS: [u32; 8] = [71, 72, 73, 74, 75, 162, 163, 164];
/// The DEF a random resource's copy names, by resource (0x677958).
const PILES: [&str; 7] = [
    "avtwood0.def",
    "avtmerc0.def",
    "avtore0.def",
    "avtsulf0.def",
    "avtcrys0.def",
    "avtgems0.def",
    "avtgold0.def",
];
/// A hero's facing at a new game's start: right (`live::Facing::Right`).
const RIGHT: i64 = 2;

/// A whole game's record, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRecord {
    /// What the game is made of and whose record it is.
    pub header: GameRecordHeader,
    /// The player's messages, in order.
    pub messages: Vec<ServerMessage>,
}

/// Why a record cannot be shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    /// The record holds no line.
    Empty,
    /// A line does not read: its number from 1 and why.
    Line(usize, String),
    /// Its format is not this viewer's.
    Format(u32),
    /// Its protocol is not this viewer's.
    Protocol(u32),
    /// Its map is not the file's: the record's identity and the file's.
    Map(MapIdentity, MapIdentity),
    /// Its game is not of the viewer's rules: the manifest the protocol makes of them differs.
    Rules,
    /// The map does not read: why.
    Read(String),
    /// The engine's import refuses the map (MCL-Q21 (b)): why.
    Import(String),
    /// Its first message is not the game's snapshot.
    NoStart,
    /// It does not begin at the game's start: its header's first message's number (a bounded
    /// record let its oldest go and begins at a later snapshot, which the start's new-game rules
    /// would misread).
    Cut(u32),
    /// The import's static objects are not the file's objects but its heroes and boats.
    Setup,
    /// The view shows an object the file holds none of there: its type and anchor.
    Object(u16, [u16; 3]),
    /// A hero the view shows whose type the view or the rules lack: its id.
    Hero(u16),
    /// A player the view names that its players do not list: its number.
    Player(u8),
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("the record is empty"),
            Self::Line(line, why) => write!(f, "line {line}: {why}"),
            Self::Format(format) => {
                write!(f, "record format {format}, not {RECORD_FORMAT_VERSION}")
            }
            Self::Protocol(protocol) => write!(f, "protocol {protocol}, not {PROTOCOL_VERSION}"),
            Self::Map(theirs, ours) => write!(
                f,
                "the record's map is {:#018x} (import {}), the file is {:#018x} (import {})",
                theirs.content_hash.0,
                theirs.import_version,
                ours.content_hash.0,
                ours.import_version
            ),
            Self::Rules => f.write_str("the record's game is not of this viewer's rules"),
            Self::Read(why) => write!(f, "the map does not read: {why}"),
            Self::Import(why) => write!(f, "the engine's import refuses the map: {why}"),
            Self::NoStart => f.write_str("the record does not begin with the game's snapshot"),
            Self::Cut(first) => write!(
                f,
                "the record begins at its message {first}, not at the game's start (its bound let \
                 the oldest go): only a whole game's record is shown"
            ),
            Self::Setup => f.write_str("the import's objects are not the map's"),
            Self::Object(object_type, [x, y, z]) => write!(
                f,
                "the view shows an object of type {object_type} at ({x}, {y}, {z}) the map has none of"
            ),
            Self::Hero(id) => write!(f, "hero {id}'s type is not known"),
            Self::Player(player) => write!(f, "player {player} is not among the game's"),
        }
    }
}

/// Reads a whole game's record's text; blank lines are skipped (`h3_protocol::record`'s reader,
/// the client lane's too: E24 M5 T2c).
///
/// # Errors
/// [`Refused::Empty`], the first line that does not read, a format or protocol not this
/// viewer's.
pub fn parse(text: &str) -> Result<GameRecord, Refused> {
    let (header, messages) = record::read_game(text)?;
    Ok(GameRecord { header, messages })
}

impl From<RecordError> for Refused {
    fn from(error: RecordError) -> Self {
        match error {
            RecordError::Empty => Self::Empty,
            RecordError::Line(line, why) => Self::Line(line, format!("{why:?}")),
            RecordError::Format(format) => Self::Format(format),
            RecordError::Protocol(protocol) => Self::Protocol(protocol),
        }
    }
}

/// A map file's content hash, as an adventure replay names its map
/// (`h3_sim::adventure::replay::map_content_hash`).
#[must_use]
pub fn content_hash(bytes: &[u8]) -> u64 {
    let mut h = StateHasher::new();
    h.write_len(bytes.len());
    h.write_bytes(bytes);
    h.finish()
}

/// A record checked against the map file it is drawn on: the record, the map read and the
/// engine's setup of it.
#[derive(Debug, Clone)]
pub struct Loaded {
    /// The record.
    pub record: GameRecord,
    /// The map, as its file holds it.
    pub raw: RawMap,
    /// The engine's import of it.
    pub setup: GameSetup,
    /// A live start's facts (E24 M5 T3b, [`live`]); none for a record's file.
    pub live: Option<Live>,
}

/// A live start's facts the view lacks (E24 M5 T3b, *provisional*): each hero's facing as the
/// host last saw it drawn, by hero id, in the original's order of eight (0 up, 2 right); a hero
/// not among them faces right (MCL-Q58: the view names no facing).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Live {
    /// The facings, by hero id.
    pub facings: Vec<(u16, u8)>,
    /// The seed of the client's own picks in play (E24 M5 T4b, `crate::picks`: an unnamed
    /// town's name); none: the picks in turn.
    pub picks: Option<u64>,
}

impl Live {
    /// The facing of hero `id`: the host's, else right.
    fn facing(&self, id: u16) -> i64 {
        self.facings
            .iter()
            .find(|&&(hero, _)| hero == id)
            .map_or(RIGHT, |&(_, facing)| i64::from(facing % 8))
    }
}

/// Reads the record `text` of a game on the map file `map` under the viewer's `rules`.
///
/// # Errors
/// The record does not read ([`parse`]); its map is not the file; its manifest is not the one
/// the protocol makes of `rules` and the map; the map does not read; the import refuses it.
pub fn load(text: &str, map: &[u8], rules: &GameRuleset) -> Result<Loaded, Refused> {
    check(parse(text)?, map, rules)
}

/// The record `record`, already read ([`parse`]: its header names the rules to read), checked
/// as [`load`] checks it.
///
/// # Errors
/// As [`load`]'s but the reading's.
pub fn check(record: GameRecord, map: &[u8], rules: &GameRuleset) -> Result<Loaded, Refused> {
    let ours = MapIdentity {
        content_hash: WireU64(content_hash(map)),
        import_version: MAP_IMPORT_VERSION,
    };
    let theirs = record.header.manifest.map;
    if theirs != ours {
        return Err(Refused::Map(theirs, ours));
    }
    if manifest(rules, ours) != record.header.manifest {
        return Err(Refused::Rules);
    }
    let raw = h3_map::read(map, &Limits::default()).map_err(|e| Refused::Read(e.to_string()))?;
    let setup = h3_map::import(&raw, rules).map_err(|e| Refused::Import(e.to_string()))?;
    Ok(Loaded {
        record,
        raw,
        setup,
        live: None,
    })
}

/// The record `record` of a live start (E24 M5 T3b: the hot seat's host's, a human's record since
/// it last left its view, beginning at a snapshot of that view), checked as [`check`] checks it,
/// with the facts the host saw (`live`): its first message may come after the game's start.
///
/// # Errors
/// As [`check`]'s.
pub fn live(
    record: GameRecord,
    map: &[u8],
    rules: &GameRuleset,
    live: Live,
) -> Result<Loaded, Refused> {
    let mut loaded = check(record, map, rules)?;
    loaded.live = Some(live);
    Ok(loaded)
}

impl Loaded {
    /// The game's first observation: the record's first message, its snapshot (a live start's:
    /// the view where its host set it up, E24 M5 T3b).
    ///
    /// # Errors
    /// [`Refused::Cut`]: the record does not begin at the game's start (its header's `first` is
    /// not 1) and is no live start's; [`Refused::NoStart`]: it begins with another message, or
    /// with none.
    pub fn first(&self) -> Result<&GameObservation, Refused> {
        let first = self.record.header.first;
        if first != 1 && self.live.is_none() {
            return Err(Refused::Cut(first));
        }
        match self.record.messages.first() {
            Some(ServerMessage::GameSnapshot { observation, .. }) => Ok(observation),
            _ => Err(Refused::NoStart),
        }
    }
}

/// A record's start as a live part's table.
#[derive(Debug, Clone, PartialEq)]
pub struct Start {
    /// The table, in `live.toml`'s keys.
    pub table: Table,
    /// Per map object, whether the view shows it.
    pub seen: Vec<bool>,
}

/// The live part of `loaded`'s first observation (the module's documentation), its map's art
/// `art` (`MapArt::from_raw` of `loaded.raw`), the game's templates `templates` (`Objects.txt`:
/// a random monster's picture), towns' pictures `town_art` (`[map.towns]`) and the classes the
/// game loads the map's as `records` (`[map.records]`: the mouse-over's class rules read the
/// game's, 219 a garrison's 33 and 220 a mine's 53, as the tiles' records do) under `rules`.
///
/// # Errors
/// [`Refused::NoStart`], [`Refused::Cut`]; [`Refused::Setup`] (`art` is not of `loaded`'s map);
/// an object the file does not hold where the view shows it; a hero of no known type; a player
/// the view does not list.
pub fn start(
    loaded: &Loaded,
    art: &MapArt,
    templates: &[ObjectTemplate],
    town_art: &TownArt,
    records: &Records,
    rules: &GameRuleset,
) -> Result<Start, Refused> {
    let observation = loaded.first()?;
    // An art of another map (its objects not the file's) would index past the file's below.
    if art.objects.len() != loaded.raw.objects.len() {
        return Err(Refused::Setup);
    }
    let colour = |player: u8| {
        observation
            .players
            .get(usize::from(player))
            .map(|p| i64::from(p.colour))
            .ok_or(Refused::Player(player))
    };
    let count = art.objects.len();
    let class = |index: usize| objects::template(art, index).map(|t| (t.class, t.subclass));
    // The file's objects the import kept as static ones, in order (`import_map`).
    let kept: Vec<usize> = (0..count)
        .filter(|&i| {
            !matches!(
                class(i),
                Some((HERO_CLASS | RANDOM_HERO | HERO_PLACEHOLDER | BOAT_CLASS, _))
            )
        })
        .collect();
    let entries = &loaded.setup.map.objects;
    if kept.len() != entries.len() {
        return Err(Refused::Setup);
    }
    let mut defs: Vec<Value> = Vec::with_capacity(count);
    let mut drawn: Vec<(u32, u32)> = Vec::with_capacity(count);
    for index in 0..count {
        let (mut def, (c, s)) = objects::template(art, index)
            .map_or((String::new(), (0, 0)), |t| {
                (t.def.clone(), (t.class, t.subclass))
            });
        // A town the view does not show: its picture at a new game by the map's buildings, as
        // the capture's (design §2.9; E24 M4 T3b2), the view's below for one it shows.
        if let (TOWN_CLASS, Some(Body::Town(town))) =
            (c, loaded.raw.objects.get(index).map(|o| &o.body))
            && let Some(picture) = town_art.picture(s, built_at_start(&town.buildings))
        {
            def = picture.into();
        }
        defs.push(Value::String(def));
        drawn.push((c, s));
    }
    let mut views: Vec<Option<&Object>> = vec![None; count];
    let mut owners: Vec<Value> = loaded
        .raw
        .objects
        .iter()
        .map(|o| Value::Integer(owner(&o.body).map_or(-1, i64::from)))
        .collect();
    let mut seen = vec![false; count];
    let mut taken = vec![false; entries.len()];
    let shift = |id: Option<ObjectTypeId>| drawn_shift(rules, id);
    for object in &observation.adventure.objects {
        let k = find(entries, &taken, object, &shift).ok_or_else(|| {
            let a = object.anchor;
            Refused::Object(object.object_type, [a.x, a.y, u16::from(a.level)])
        })?;
        taken[k] = true;
        let index = kept[k];
        seen[index] = true;
        views[index] = Some(object);
        owners[index] = Value::Integer(object.owner.map_or(Ok(-1), colour)?);
        let became = class(index).and_then(|(c, _)| became(c, object, rules, templates));
        if let Some((def, c, s)) = became {
            defs[index] = Value::String(def);
            drawn[index] = (c, s);
        }
        // A town's picture by its faction and its view's fort and Capitol (E24 M4 T3b2); a
        // random town a town of its faction, as the original's copy (design §2.9).
        if let Some((def, faction)) = town_picture(observation, object, town_art) {
            defs[index] = Value::String(def.into());
            drawn[index] = (TOWN_CLASS, faction);
        }
    }
    let classes: Vec<Value> = drawn.iter().map(|&(c, s)| pair(c, s)).collect();
    let mut heroes = Vec::new();
    for hero in &observation.adventure.heroes {
        let class = observation
            .heroes
            .iter()
            .find(|info| info.id == hero.id)
            .and_then(|info| rules.core().hero_type(Handle::from_raw(info.hero_type)))
            .ok_or(Refused::Hero(hero.id))?
            .class;
        let owner = colour(hero.owner)?;
        let (boat, flag) = hero.boat.map_or((-1, -1), |kind| (i64::from(kind), owner));
        let p = hero.pos;
        heroes.push(row(&[
            i64::from(p.x),
            i64::from(p.y),
            i64::from(p.level),
            int(class.index()),
            loaded
                .live
                .as_ref()
                .map_or(RIGHT, |live| live.facing(hero.id)),
            owner,
            boat,
            flag,
        ]));
    }
    let boats = observation
        .adventure
        .boats
        .iter()
        .map(|b| {
            let p = b.tile;
            row(&[
                i64::from(p.x),
                i64::from(p.y),
                i64::from(p.level),
                i64::from(b.kind),
                i64::from(b.facing),
            ])
        })
        .collect();
    let mut table = Table::new();
    table.insert(
        "viewer".to_string(),
        Value::Integer(colour(observation.adventure.viewer)?),
    );
    table.insert("defs".to_string(), Value::Array(defs));
    table.insert("classes".to_string(), Value::Array(classes));
    table.insert("owners".to_string(), Value::Array(owners));
    table.insert("heroes".to_string(), Value::Array(heroes));
    table.insert("boats".to_string(), Value::Array(boats));
    table.insert("explored".to_string(), explored(observation, art));
    // The panels (E24 M4 T3b1, design §5.10).
    let picks = loaded.live.as_ref().and_then(|l| l.picks);
    panels::insert(&mut table, observation, (&loaded.raw, picks), art, rules);
    // The mouse-over's facts (E24 M4 T3b2, design §5.11).
    let sight = hover::Sight {
        views,
        classes: drawn
            .iter()
            .map(|&(c, s)| (records.game_class(c), s))
            .collect(),
    };
    let facts = hover::table(observation, (&loaded.raw, picks), art, rules, &sight);
    table.insert("hover".to_string(), Value::Table(facts));
    Ok(Start { table, seen })
}

/// The picture of the town `object` (its view among `observation`'s towns) by its faction and
/// its view's fort and Capitol (`town_art`'s lists: a Capitol's, a Fort's to a Castle's, a
/// village's; design §2.9), the buildings the original's picture reads (0x4C9ADE); none for
/// another object or a faction without one; with its faction.
fn town_picture<'a>(
    observation: &GameObservation,
    object: &Object,
    town_art: &'a TownArt,
) -> Option<(&'a str, u32)> {
    let town = observation
        .towns
        .iter()
        .find(|t| Some(t.town) == object.town)?;
    let bit = |id: u32| 1u64 << id;
    let built =
        if town.capitol { bit(CAPITOL) } else { 0 } | if town.fort > 0 { bit(FORT) } else { 0 };
    let faction = u32::from(town.faction);
    town_art.picture(faction, built).map(|def| (def, faction))
}

/// The entry of `entries` not yet `taken` that `object` is: its type, anchor and masks; else one
/// drawn when the game was built at its anchor, or as far left of it as `shift` allows.
fn find(
    entries: &[ObjectSetup],
    taken: &[bool],
    object: &Object,
    shift: &dyn Fn(Option<ObjectTypeId>) -> Option<u16>,
) -> Option<usize> {
    let a = object.anchor;
    let at = |e: &ObjectSetup, shift: u16| {
        (e.anchor.x.saturating_add(shift), e.anchor.y, e.anchor.level) == (a.x, a.y, a.level)
    };
    let same = |e: &ObjectSetup| {
        e.object_type.map(ObjectTypeId::raw) == Some(object.object_type)
            && [e.blocked.bits(), e.visitable.bits(), e.drawn.bits()]
                == [object.blocked.0, object.visitable.0, object.drawn.0]
    };
    let pick = |test: &dyn Fn(&ObjectSetup) -> bool| {
        (0..entries.len()).find(|&k| !taken[k] && test(&entries[k]))
    };
    pick(&|e| at(e, 0) && same(e))
        .or_else(|| pick(&|e| shift(e.object_type).is_some_and(|s| (0..=s).any(|d| at(e, d)))))
}

/// How far right of its file's anchor a view may show an object of the type `id` that the game
/// draws when it is built: a random dwelling one column (the Angel Portal's shift, the adventure
/// lane's E08 T94a), a random monster, artifact or resource none; none for every other type.
fn drawn_shift(rules: &GameRuleset, id: Option<ObjectTypeId>) -> Option<u16> {
    let kind = id
        .and_then(|id| rules.adventure().object_type(id))
        .map(|t| &t.kind);
    match kind {
        Some(ObjectKind::RandomDwelling) => Some(1),
        Some(
            ObjectKind::Monster { levels: Some(_) }
            | ObjectKind::Artifact { classes: Some(_) }
            | ObjectKind::Pickup(Pickup::OneOf(_)),
        ) => Some(0),
        _ => None,
    }
}

/// What a random object of the file's class `class` became, as the original's copy of its
/// template names it (design §2.7): its DEF, class and subclass; none for every other object and
/// for the random kinds `MCL-STUB-1` leaves.
fn became(
    class: u32,
    object: &Object,
    rules: &GameRuleset,
    templates: &[ObjectTemplate],
) -> Option<(String, u32, u32)> {
    if class == RANDOM_RESOURCE {
        let kind = rules
            .adventure()
            .object_type(ObjectTypeId::from_raw(object.object_type))
            .map(|t| &t.kind);
        let Some(ObjectKind::Pickup(Pickup::Resource { resource, .. })) = kind else {
            return None;
        };
        let index = resource.index();
        return PILES.get(index).map(|def| {
            let subclass = u32::try_from(index).unwrap_or(u32::MAX);
            ((*def).to_string(), PILE, subclass)
        });
    }
    // stub: MCL-STUB-1 (a random artifact or dwelling keeps its random template)
    if !RANDOM_MONSTERS.contains(&class) {
        return None;
    }
    object.monster.as_ref().and_then(|monster| {
        let creature = u32::from(monster.creature);
        templates
            .iter()
            .rev()
            .find(|t| (t.class, t.subclass) == (MONSTER, creature))
            .map(|t| (t.def.clone(), MONSTER, creature))
    })
}

/// The view's explored tiles as `live.toml`'s rows: per level and row, a `0` or `1` per tile.
fn explored(observation: &GameObservation, art: &MapArt) -> Value {
    let a = &observation.adventure;
    let stride = usize::from(a.stride);
    let mut rows = Vec::with_capacity(art.size * art.levels);
    for row in 0..art.size * art.levels {
        let text: String = (0..art.size)
            .map(|x| {
                let word = a.explored.get(row * stride + x / 64).map_or(0, |w| w.0);
                if word >> (x % 64) & 1 == 1 { '1' } else { '0' }
            })
            .collect();
        rows.push(Value::String(text));
    }
    Value::Array(rows)
}

/// A class and subclass as `live.toml` gives them.
fn pair(class: u32, subclass: u32) -> Value {
    row(&[i64::from(class), i64::from(subclass)])
}

/// A row of numbers.
fn row(numbers: &[i64]) -> Value {
    Value::Array(numbers.iter().map(|&n| Value::Integer(n)).collect())
}

/// A small index as a table's number.
fn int(n: usize) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

/// What differs between two live parts' tables where the engine's start says something
/// (`ours` the engine's [`Start`], `theirs` a capture's): the viewer, the explored rows, each
/// object the view shows (its picture, by its DEF's name in any case, and its owner) and the
/// heroes and boats on explored tiles (`theirs` holds every one in the game's memory). A key
/// missing on one side differs from the other's.
#[must_use]
pub fn differences(ours: &Start, theirs: &Table) -> Vec<String> {
    let mut found = Vec::new();
    let mine = &ours.table;
    for key in ["viewer", "explored"] {
        if mine.get(key) != theirs.get(key) {
            found.push(format!("`{key}` differs"));
        }
    }
    let column = |t: &Table, key: &str| {
        t.get(key)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    // A DEF's name in any case; an owner as it is.
    let fold = |v: Option<&Value>| {
        v.map(|v| {
            v.as_str()
                .map_or_else(|| v.clone(), |s| Value::String(s.to_ascii_lowercase()))
        })
    };
    for key in ["defs", "owners"] {
        let (a, b) = (column(mine, key), column(theirs, key));
        for (index, _) in ours.seen.iter().enumerate().filter(|(_, s)| **s) {
            if fold(a.get(index)) != fold(b.get(index)) {
                found.push(format!("object {index}: `{key}` differ"));
            }
        }
    }
    for key in ["heroes", "boats"] {
        if on_explored(mine, key) != on_explored(theirs, key) {
            found.push(format!("`{key}` on explored tiles differ"));
        }
    }
    panels::differences(mine, theirs, &mut found);
    found
}

/// The rows of `key` (`[x, y, level, …]`) on tiles `table`'s `explored` marks, in order.
fn on_explored(table: &Table, key: &str) -> Vec<Vec<i64>> {
    let rows: Vec<&str> = table
        .get("explored")
        .and_then(Value::as_array)
        .map_or_else(Vec::new, |a| a.iter().filter_map(Value::as_str).collect());
    let size = rows.first().map_or(0, |r| r.len());
    let explored = |n: &[i64]| {
        let at = |i: usize| {
            n.get(i)
                .and_then(|&v| usize::try_from(v).ok())
                .unwrap_or(usize::MAX)
        };
        rows.get(at(2).saturating_mul(size).saturating_add(at(1)))
            .is_some_and(|r| r.as_bytes().get(at(0)) == Some(&b'1'))
    };
    let mut kept: Vec<Vec<i64>> =
        table
            .get(key)
            .and_then(Value::as_array)
            .map_or_else(Vec::new, |a| {
                a.iter()
                    .map(|v| {
                        v.as_array().map_or_else(Vec::new, |r| {
                            r.iter().filter_map(Value::as_integer).collect()
                        })
                    })
                    .filter(|n| explored(n))
                    .collect()
            });
    kept.sort_unstable();
    kept
}
