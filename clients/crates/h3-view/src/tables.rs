//! The asset tables (E17 M1 T4; ADR-0020 points 5 and 6): `clients/assets/<id>.toml`, one per
//! rule pack (`base`, `ab`, `sod`), in the packs' dependency graph and key namespace. They map the
//! engine's content keys to archive entry names and the numbers the drawing needs, and hold the
//! asset fixes. Identifiers and numbers only (ADR-0009). The format is described in
//! `clients/README.md` ("The asset tables"); it is *provisional* until the M1 review.
//!
//! [`parse`] reads one file; [`load`] takes an expansion selection (the id of the topmost table,
//! as a ruleset names its topmost pack) and merges it with its dependencies into one [`Tables`],
//! keys qualified by their table's id (`base:pikeman`). As `h3-pack` does for the packs, unknown
//! sections and fields are errors, and so are unknown or circular dependencies and a key defined
//! twice.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use serde::Deserialize;

use crate::anim::Sound;

/// One table file as written.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableFile {
    /// The table's id and dependencies.
    pub table: Header,
    /// The battle screen's settings (one table of a selection holds them).
    #[serde(default)]
    pub battle: Option<Battle>,
    /// Creatures, by key.
    #[serde(default)]
    pub creature: Vec<Creature>,
    /// The heroes' battle pictures, by hero type key (E17 M3 T3a).
    #[serde(default)]
    pub hero: Vec<Hero>,
    /// The spells' sounds and magic animations, by spell key (E17 M3 T7b-1).
    #[serde(default)]
    pub spell: Vec<Spell>,
    /// Battlefield backgrounds, by a terrain's or a special battlefield's key.
    #[serde(default)]
    pub battlefield: Vec<Battlefield>,
    /// Obstacle graphics, by key.
    #[serde(default)]
    pub obstacle: Vec<ObstacleEntry>,
    /// Asset fixes.
    #[serde(default)]
    pub fix: Vec<Fix>,
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

/// The battle screen's settings.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Battle {
    /// The creature DEF groups the battle shows.
    pub groups: Groups,
    /// A battle hero's DEF groups (E17 M3 T3a).
    pub hero_groups: HeroGroups,
    /// The flags' DEFs beside the heroes: the left side's, the right side's (0x4634AB; E17 M3
    /// T3a).
    pub flags: [String; 2],
    /// The magic animations by number (`PlayMagicAnimation` 0x496840's table 0x641E08, 83
    /// records; E17 M3 T3b): the tent's heal is number 79.
    pub magic: Vec<Magic>,
    /// The sound the First Aid Tent's heal plays (`first_aid_tent_action` 0x478C46; E17 M3 T3b).
    pub heal_sound: String,
    /// The sound of a spell resisted by the stack it was aimed at (`cast_spell` 0x5A1F98; E17
    /// M3 T7b-1).
    #[serde(default)]
    pub resist_sound: Option<String>,
    /// The sounds of the rolled effects over a stack (E17 M3 T7b-2b-2b, design §7.16): good
    /// morale's (`good_morale_roll` 0x464A6C), bad morale's (`bad_morale_check` 0x464C84),
    /// Fear's (`fear_check` 0x464E93), each category 2 as the heal's, and good luck's (as a strike
    /// or a shot starts, 0x44185B, 0x43F979: `goodluck.82m`, its stem the entry; category 3).
    #[serde(default)]
    pub good_morale_sound: Option<String>,
    /// Bad morale's ([`Battle::good_morale_sound`]).
    #[serde(default)]
    pub bad_morale_sound: Option<String>,
    /// Fear's ([`Battle::good_morale_sound`]).
    #[serde(default)]
    pub fear_sound: Option<String>,
    /// Good luck's ([`Battle::good_morale_sound`]).
    #[serde(default)]
    pub luck_sound: Option<String>,
    /// The sounds of the striker's effects (E17 M3 T7b-2b-3, design §7.16), each category 2 as the
    /// heal's: Life Drain's (`striker_effects` 0x440E68, `DrainLif.wav`) and the Thunderbird's bolt
    /// (0x4412FA, `LightBlt.wav`); Death Stare and the Rust Dragon's acid play their spell's.
    #[serde(default)]
    pub drain_sound: Option<String>,
    /// The Thunderbird's bolt's ([`Battle::drain_sound`]).
    #[serde(default)]
    pub thunderbolt_sound: Option<String>,
}

impl Battle {
    /// The battle's own sounds by name: the heal's, then each optional one given.
    pub fn sounds(&self) -> impl Iterator<Item = &String> {
        core::iter::once(&self.heal_sound)
            .chain(&self.resist_sound)
            .chain(&self.good_morale_sound)
            .chain(&self.bad_morale_sound)
            .chain(&self.fear_sound)
            .chain(&self.luck_sound)
            .chain(&self.drain_sound)
            .chain(&self.thunderbolt_sound)
    }
}

/// A magic animation (`docs/epics/E17-design.md` §7.15; E17 M3 T3b): its DEF (group 0), where it
/// is drawn over a stack (`place`, the table entry's low byte: 0 standing on the hex, 1 centred on
/// the creature, 2 above it, 3 beside it; higher ones not over a stack, not read) and whether
/// the translucent drawer draws it (the entry's bit 8).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Magic {
    /// The DEF.
    pub def: String,
    /// Where it is drawn.
    pub place: u8,
    /// Drawn half and half with what is under it.
    #[serde(default)]
    pub translucent: bool,
}

/// A battle hero's DEF groups (`docs/epics/E17-design.md` §4.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeroGroups {
    /// Standing.
    pub standing: u8,
    /// The idle motion, from time to time.
    pub idle: u8,
    /// After a stack of its side died.
    pub defeat: u8,
    /// After a stack of the other side died.
    pub victory: u8,
    /// Casting a spell.
    pub cast: u8,
}

/// A hero type's battle picture (`battle_load_heroes` 0x463370: by its class's town and its
/// sex; E17 M3 T3a) and its cast (`hero_battle_defs` 0x63BD40, the same record's +4 to +0xC; E17
/// M3 T7b-1, design §7.16).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hero {
    /// The hero type's key.
    pub key: String,
    /// Its original id: the executable's hero type, its record among `hotraits.txt`'s (the key
    /// catalogue's family `hero`; E17 M3 T7b-1).
    pub original: u16,
    /// Its battle DEF.
    pub def: String,
    /// Its cast point in the DEF's frame (x, y): where a spell's projectile starts.
    #[serde(default)]
    pub cast: [i16; 2],
    /// The frames of its cast group played before the spell.
    #[serde(default)]
    pub cast_frames: u8,
}

/// A spell's sign, sound and magic animation (`spell_table` 0x685450, its records' +0, +4 and +8;
/// E17 M3 T7b-1, T7b-2a, design §7.16): what a cast plays over its target when the spell's handler plays the
/// record's animation.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spell {
    /// The spell's key.
    pub key: String,
    /// Its original id: the executable's spell number, its record among `sptraits.txt`'s spell
    /// rows (the key catalogue's family `spell`).
    pub original: u16,
    /// Its magic animation's number (none: the record's −1).
    #[serde(default)]
    pub animation: Option<u8>,
    /// Its sound, an entry of the sound archives without its extension (none: the record has
    /// none).
    #[serde(default)]
    pub sound: Option<String>,
    /// Its sign, the record's +0: 1 positive, −1 negative, 0 neither (the count box's tint sums
    /// the signs of a stack's spells, 0x43E6C7; E17 M3 T7b-2a, design §4.3).
    #[serde(default)]
    pub sign: i8,
    /// The DEFs of its projectile, if its handler flies one (`spell_projectile` 0x467DB0: Magic
    /// Arrow's, Ice Bolt's, Disrupting Ray's one), from the level one to the one pointing down.
    #[serde(default)]
    pub projectile: Vec<String>,
    /// Each projectile DEF's angle in degrees (0 level, negative down).
    #[serde(default)]
    pub projectile_angles: Vec<i16>,
    /// The battlefield objects its handler places (Quicksand, Land Mine, Force Field, Fire Wall;
    /// E17 M3 T7c-2, design §7.16): one per object size it places, by its hexes.
    #[serde(default)]
    pub objects: Vec<SpellObject>,
    /// The sound its handler plays with each object it places, an entry of the sound archives
    /// without its extension (Quicksand's `Quiksand.wav` 0x5A0369, Land Mine's `landmine.wav`
    /// 0x5A0577; E17 M3 T7c-2).
    #[serde(default)]
    pub object_sound: Option<String>,
}

/// A battlefield object a spell places (`battle_object`, its descriptor 0x14 bytes from 0x63CEE8;
/// E17 M3 T7c-2, design §4.4, §7.16): drawn as a usual obstacle is, its frame the idle steps'
/// count modulo its group 0's frames.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellObject {
    /// Its DEF (the descriptor's +0x10).
    pub def: String,
    /// The hexes it covers (the descriptor's +6): the object a placement's hexes take.
    pub hexes: u8,
    /// Its height in hex rows (+4): its canvas rises 42 pixels a row above its anchor's.
    pub rows: u8,
    /// Its layer (+7) is 1: drawn in the pass before every stack, not in its row's.
    #[serde(default)]
    pub before_stacks: bool,
    /// The magic animation at a hex its handler plays as it places it (player 5).
    pub placed: u8,
    /// The magic animation at its anchor as it goes (the object's +0x14).
    pub vanish: u8,
}

/// The creature DEF groups the battle uses (`docs/epics/E17-design.md` §4.2, §4.3, §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Groups {
    /// Walking (and a flight's cycles).
    pub moving: u8,
    /// The mouse over the stack (and the fidgets).
    pub mouse_over: u8,
    /// Standing.
    pub standing: u8,
    /// A struck stack's reaction.
    pub hit: u8,
    /// A defending struck stack's reaction.
    pub defend: u8,
    /// Dying (its last frame stays: the corpse).
    pub death: u8,
    /// A melee strike up, straight and down.
    pub strike: [u8; 3],
    /// A shot up, straight and down.
    pub shoot: [u8; 3],
    /// A breath on two stacks up, straight and down, where the DEF has them (`melee_strike`
    /// 0x441ACA; E17 M2 T5c).
    pub breath: [u8; 3],
    /// Before a move.
    pub move_start: u8,
    /// After a move.
    pub move_end: u8,
    /// Turning to face left (from facing right): the group before the flip and the one after
    /// (7, 10; `stack_turn_around` 0x446720, design §7.3).
    pub turn_left: [u8; 2],
    /// Turning to face right: before the flip and after (9, 8).
    pub turn_right: [u8; 2],
}

/// A creature's battle graphics.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Creature {
    /// The content key in the table's namespace.
    pub key: String,
    /// Its original id: the executable's creature number, its record among `crtraits.txt`'s
    /// creature rows (E17 M3 T4b-1: its names, the key catalogue's family `creature`).
    pub original: u16,
    /// The battle sprite.
    pub def: String,
    /// `cranim.txt`'s four times as the executable keeps them (fidget interval, walk, attack,
    /// flight distance; scaled by 9000, 500, 500 and 115 and truncated, 0x50CA00).
    pub times: [u32; 4],
    /// Extra x of the count box outside the hex (`cranim.txt`, §4.3).
    #[serde(default)]
    pub troop_count_offset: i32,
    /// The attack's climax frame (`cranim.txt`).
    #[serde(default)]
    pub attack_climax: u32,
    /// A shooter's projectile.
    #[serde(default)]
    pub missile: Option<String>,
    /// Where the projectile starts, up, straight and down (x, y).
    #[serde(default)]
    pub missile_offsets: [[i16; 2]; 3],
    /// The projectile frames' angles in degrees, from high to low.
    #[serde(default)]
    pub missile_angles: [i16; 12],
    /// A ray shooter's colours (R, G, B) by the distance from the ray's edge, the executable's
    /// table for the creature (`DrawRay`'s colour 302 or 304; E17 M3 T1b-1, design §7.12): it
    /// draws its shot as a ray, not its projectile.
    #[serde(default)]
    pub ray: Option<Vec<[u8; 3]>>,
    /// The explosion its projectile ends in (E17 M3 T7c-3): the spell whose animation and sound
    /// it plays and each frame's time (`stack_shot` 0x43FA15: the Magogs Fireball's at 50 ms,
    /// 0x43FE07: the Liches Death Cloud's at 100 ms).
    #[serde(default)]
    pub explosion: Option<CreatureExplosion>,
    /// Its sounds' entries in the sound archives (E17 M2 T5b).
    #[serde(default)]
    pub sounds: Sounds,
}

/// The explosion a shooter's projectile ends in ([`Creature::explosion`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureExplosion {
    /// The spell (the executable's number).
    pub spell: u16,
    /// Each frame's time in ms (times the speed factor).
    pub time: u32,
}

/// A creature's sounds: each an entry of the sound archives, none where the creature has no such
/// sound (`stack_load_graphics` 0x43D9F0 decides, `h3-view harvest` writes them; design §8.2).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sounds {
    /// Moving (looping while it moves).
    #[serde(default, rename = "move")]
    pub moving: Option<String>,
    /// A melee strike.
    #[serde(default)]
    pub attack: Option<String>,
    /// Struck.
    #[serde(default)]
    pub wince: Option<String>,
    /// A shot (and a spellcaster's cast).
    #[serde(default)]
    pub shot: Option<String>,
    /// Dying.
    #[serde(default)]
    pub death: Option<String>,
    /// Struck while defending.
    #[serde(default)]
    pub defend: Option<String>,
    /// Before a move.
    #[serde(default)]
    pub start: Option<String>,
    /// After a move.
    #[serde(default)]
    pub end: Option<String>,
}

impl Sounds {
    /// The entry of `sound`, if the creature has it.
    #[must_use]
    pub fn name(&self, sound: Sound) -> Option<&str> {
        match sound {
            Sound::Move => self.moving.as_deref(),
            Sound::Attack => self.attack.as_deref(),
            Sound::Wince => self.wince.as_deref(),
            Sound::Shot => self.shot.as_deref(),
            Sound::Death => self.death.as_deref(),
            Sound::Defend => self.defend.as_deref(),
            Sound::Start => self.start.as_deref(),
            Sound::End => self.end.as_deref(),
            // The battle's, not the creature's ([`Battle::sounds`], a spell's [`Spell::sound`]).
            Sound::Heal
            | Sound::Resisted
            | Sound::Spell(_)
            | Sound::GoodMorale
            | Sound::BadMorale
            | Sound::Fear
            | Sound::Luck
            | Sound::Drain
            | Sound::Thunderbolt
            | Sound::SpellFile(_)
            | Sound::Object(_) => None,
        }
    }

    /// Every entry it names.
    pub fn names(&self) -> impl Iterator<Item = &String> {
        [
            &self.moving,
            &self.attack,
            &self.wince,
            &self.shot,
            &self.death,
            &self.defend,
            &self.start,
            &self.end,
        ]
        .into_iter()
        .flatten()
    }
}

/// A battlefield's background pictures.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Battlefield {
    /// The terrain's key, or a special battlefield's (E17 M3 T2; both in the pack that defines
    /// it).
    pub key: String,
    /// The pictures, by the variant the adventure map picks (0x41ADB0).
    pub backgrounds: Vec<String>,
}

/// An obstacle's graphics as written: a usual obstacle names a DEF, its height in hex rows and
/// its layer; an absolute one a picture and its screen position.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObstacleEntry {
    /// The content key in the table's namespace.
    pub key: String,
    /// A usual obstacle's DEF.
    #[serde(default)]
    pub def: Option<String>,
    /// A usual obstacle's height in hex rows.
    #[serde(default)]
    pub rows: Option<u8>,
    /// A usual obstacle's layer: 1 drawn before every stack, 0 in the row passes.
    #[serde(default)]
    pub layer: Option<u8>,
    /// An absolute obstacle's picture.
    #[serde(default)]
    pub pcx: Option<String>,
    /// An absolute obstacle's screen position (x, y).
    #[serde(default)]
    pub at: Option<[i16; 2]>,
}

/// An asset fix (ADR-0020 point 6): an id, its source and what it changes. The change itself gets
/// its fields with the first fix.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fix {
    /// `CLT-QUIRK-<n>`.
    pub id: String,
    /// A code reading or a reference frame.
    pub source: String,
    /// The key of the entry it changes.
    pub target: String,
    /// What it changes, in words.
    pub summary: String,
}

/// An obstacle's graphics, checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObstacleArt {
    /// A DEF drawn at its anchor hex (§4.4).
    Usual {
        /// The DEF.
        def: String,
        /// Its height in hex rows.
        rows: u8,
        /// Drawn before every stack (true) or in the row passes.
        before_stacks: bool,
    },
    /// A picture in the background at a fixed point.
    Absolute {
        /// The picture.
        pcx: String,
        /// Its top-left corner.
        at: [i16; 2],
    },
}

/// A selection's tables, merged; keys qualified by their table's id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tables {
    /// The tables loaded, dependencies first.
    pub order: Vec<String>,
    /// The battle screen's settings.
    pub battle: Battle,
    /// Creatures.
    pub creatures: BTreeMap<String, Creature>,
    /// The heroes' battle pictures and casts, by hero type.
    pub heroes: BTreeMap<String, Hero>,
    /// The spells' sounds and animations (E17 M3 T7b-1).
    pub spells: BTreeMap<String, Spell>,
    /// Battlefields.
    pub battlefields: BTreeMap<String, Battlefield>,
    /// Obstacles.
    pub obstacles: BTreeMap<String, ObstacleArt>,
    /// Fixes, by id.
    pub fixes: BTreeMap<String, Fix>,
}

impl Tables {
    /// Every sound archive entry the creatures name, upper-cased, each once.
    pub fn sound_names(&self) -> BTreeSet<String> {
        self.creatures
            .values()
            .flat_map(|c| c.sounds.names())
            .chain(self.battle.sounds())
            .chain(self.spells.values().filter_map(|s| s.sound.as_ref()))
            .chain(self.spells.values().filter_map(|s| s.object_sound.as_ref()))
            .map(|n| n.to_ascii_uppercase())
            .collect()
    }

    /// Every archive entry name the tables mention, upper-cased, each once.
    pub fn names(&self) -> BTreeSet<String> {
        let creatures = self
            .creatures
            .values()
            .flat_map(|c| core::iter::once(&c.def).chain(&c.missile))
            .chain(self.heroes.values().map(|h| &h.def))
            .chain(&self.battle.flags)
            .chain(self.battle.magic.iter().map(|m| &m.def))
            .chain(self.spells.values().flat_map(|s| &s.projectile))
            .chain(
                self.spells
                    .values()
                    .flat_map(|s| s.objects.iter().map(|o| &o.def)),
            );
        let fields = self.battlefields.values().flat_map(|b| &b.backgrounds);
        let obstacles = self.obstacles.values().map(|o| match o {
            ObstacleArt::Usual { def, .. } => def,
            ObstacleArt::Absolute { pcx, .. } => pcx,
        });
        creatures
            .chain(fields)
            .chain(obstacles)
            .map(|n| n.to_ascii_uppercase())
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
    /// No table of the selection holds `[battle]`, or two do.
    Battle(usize),
    /// A key (qualified) or a fix id is defined twice.
    DuplicateKey(String),
    /// An entry breaks a rule: its key and the rule.
    Entry(String, String),
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(m) => write!(f, "not a table: {m}"),
            Self::DuplicateTable(id) => write!(f, "two tables have the id `{id}`"),
            Self::UnknownTable(id) => write!(f, "no table has the id `{id}`"),
            Self::Cycle(id) => write!(f, "the dependencies go round through `{id}`"),
            Self::Battle(n) => write!(f, "{n} tables of the selection hold [battle], not one"),
            Self::DuplicateKey(key) => write!(f, "`{key}` is defined twice"),
            Self::Entry(key, rule) => write!(f, "`{key}`: {rule}"),
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
/// The [`TableError`] of the first rule broken.
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
    let battles: Vec<&Battle> = selected.iter().filter_map(|f| f.battle.as_ref()).collect();
    let [battle] = battles[..] else {
        return Err(TableError::Battle(battles.len()));
    };
    for flag in &battle.flags {
        names("[battle]", &[(flag, ".DEF")])?;
    }
    for magic in &battle.magic {
        names("[battle]", &[(&magic.def, ".DEF")])?;
    }
    if battle.sounds().any(|s| s.is_empty() || s.contains('.')) {
        return Err(entry(
            "[battle]",
            "a sound is an entry's name, without an extension",
        ));
    }
    let battle = battle.clone();
    let mut tables = Tables {
        order: order.iter().map(|id| (*id).to_string()).collect(),
        battle,
        creatures: BTreeMap::new(),
        heroes: BTreeMap::new(),
        spells: BTreeMap::new(),
        battlefields: BTreeMap::new(),
        obstacles: BTreeMap::new(),
        fixes: BTreeMap::new(),
    };
    for file in selected {
        let id = &file.table.id;
        for c in &file.creature {
            let key = qualify(id, &c.key)?;
            names(&key, &[(&c.def, ".DEF")])?;
            if let Some(m) = &c.missile {
                names(&key, &[(m, ".DEF")])?;
            }
            if c.sounds.names().any(|n| n.is_empty() || n.contains('.')) {
                return Err(entry(
                    &key,
                    "a sound is an entry's name, without an extension",
                ));
            }
            insert(&mut tables.creatures, key, c.clone())?;
        }
        for h in &file.hero {
            let key = qualify(id, &h.key)?;
            names(&key, &[(&h.def, ".DEF")])?;
            insert(&mut tables.heroes, key, h.clone())?;
        }
        for spell in &file.spell {
            let key = qualify(id, &spell.key)?;
            if [&spell.sound, &spell.object_sound]
                .iter()
                .any(|s| s.as_ref().is_some_and(|n| n.is_empty() || n.contains('.')))
            {
                return Err(entry(
                    &key,
                    "a sound is an entry's name, without an extension",
                ));
            }
            if spell
                .animation
                .is_some_and(|a| usize::from(a) >= tables.battle.magic.len())
            {
                return Err(entry(&key, "no such magic animation"));
            }
            if spell.projectile.len() != spell.projectile_angles.len() {
                return Err(entry(&key, "an angle for each projectile DEF"));
            }
            for def in &spell.projectile {
                names(&key, &[(def, ".DEF")])?;
            }
            for o in &spell.objects {
                names(&key, &[(&o.def, ".DEF")])?;
                let magic = tables.battle.magic.len();
                if [o.placed, o.vanish]
                    .iter()
                    .any(|&a| usize::from(a) >= magic)
                {
                    return Err(entry(&key, "no such magic animation"));
                }
            }
            insert(&mut tables.spells, key, spell.clone())?;
        }
        for b in &file.battlefield {
            let key = qualify(id, &b.key)?;
            if b.backgrounds.is_empty() {
                return Err(entry(&key, "no background"));
            }
            for name in &b.backgrounds {
                names(&key, &[(name, ".PCX")])?;
            }
            insert(&mut tables.battlefields, key, b.clone())?;
        }
        for o in &file.obstacle {
            let key = qualify(id, &o.key)?;
            let art = obstacle(&key, o)?;
            insert(&mut tables.obstacles, key, art)?;
        }
        for x in &file.fix {
            if !x.id.starts_with("CLT-QUIRK-") {
                return Err(entry(&x.id, "a fix's id is CLT-QUIRK-<n>"));
            }
            insert(&mut tables.fixes, x.id.clone(), x.clone())?;
        }
    }
    Ok(tables)
}

/// Depth first: dependencies before the table; `path` holds the tables being visited.
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

/// The key qualified by its table's id; a key is a pack's key, without a namespace.
fn qualify(table: &str, key: &str) -> Result<String, TableError> {
    let qualified = format!("{table}:{key}");
    if key.is_empty() || key.contains(':') {
        return Err(entry(
            &qualified,
            "a key is a pack's key, without its namespace",
        ));
    }
    Ok(qualified)
}

fn insert<T>(map: &mut BTreeMap<String, T>, key: String, value: T) -> Result<(), TableError> {
    if map.contains_key(&key) {
        return Err(TableError::DuplicateKey(key));
    }
    map.insert(key, value);
    Ok(())
}

/// Each name ends in its extension (upper or lower case) and has a stem.
fn names(key: &str, list: &[(&String, &str)]) -> Result<(), TableError> {
    for (name, ext) in list {
        let upper = name.to_ascii_uppercase();
        if upper.len() <= ext.len() || !upper.ends_with(ext) {
            return Err(entry(key, &format!("`{name}` is not a {} name", &ext[1..])));
        }
    }
    Ok(())
}

fn obstacle(key: &str, o: &ObstacleEntry) -> Result<ObstacleArt, TableError> {
    match (&o.def, o.rows, o.layer, &o.pcx, o.at) {
        (Some(def), Some(rows), Some(layer), None, None) => {
            names(key, &[(def, ".DEF")])?;
            if rows == 0 || layer > 1 {
                return Err(entry(key, "rows from 1, layer 0 or 1"));
            }
            Ok(ObstacleArt::Usual {
                def: def.clone(),
                rows,
                before_stacks: layer == 1,
            })
        }
        (None, None, None, Some(pcx), Some(at)) => {
            names(key, &[(pcx, ".PCX")])?;
            Ok(ObstacleArt::Absolute {
                pcx: pcx.clone(),
                at,
            })
        }
        _ => Err(entry(
            key,
            "an obstacle has def, rows and layer, or pcx and at",
        )),
    }
}

fn entry(key: &str, rule: &str) -> TableError {
    TableError::Entry(key.to_string(), rule.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    const BASE: &str = r#"
[table]
id = "base"
depends = []

[battle]
groups = { moving = 0, mouse_over = 1, standing = 2, hit = 3, defend = 4, death = 5, strike = [11, 12, 13], shoot = [14, 15, 16], breath = [17, 18, 19], move_start = 20, move_end = 21, turn_left = [7, 10], turn_right = [9, 8] }
hero_groups = { standing = 0, idle = 1, defeat = 2, victory = 3, cast = 4 }
flags = ["CmFlagL.def", "CmFlagR.def"]
magic = [{ def = "C10spW.def", place = 0, translucent = true }, { def = "sp12_.def", place = 1 }]
heal_sound = "Regener"
resist_sound = "MagicRes"
good_morale_sound = "GoodMrle"
bad_morale_sound = "BadMrle"
fear_sound = "Fear"
luck_sound = "goodluck"
drain_sound = "DrainLif"
thunderbolt_sound = "LightBlt"

[[hero]]
key = "orrin"
original = 0
def = "CH00.DEF"
cast = [92, 67]
cast_frames = 5

[[spell]]
key = "slow"
original = 54
animation = 1
sound = "Slow"

[[spell]]
key = "magic_arrow"
original = 15
projectile = ["c20spx0.def", "c20spx1.def"]
projectile_angles = [0, -16]

[[creature]]
key = "pikeman"
original = 0
def = "CPKMAN.DEF"
times = [9000, 575, 500, 115]
sounds = { move = "pikemove", attack = "pikeattk", wince = "pikewnce", death = "pikekill", defend = "pikedfnd" }

[[creature]]
key = "archer"
original = 2
def = "clcbow.def"
times = [9000, 500, 500, 115]
troop_count_offset = -3
attack_climax = 7
missile = "PLCBOWX.DEF"
missile_offsets = [[38, -65], [36, -59], [21, -50]]
missile_angles = [90, 72, 45, 27, 0, -27, -45, -72, -90, 0, 0, 0]
explosion = { spell = 76, time = 100 }

[[battlefield]]
key = "dirt"
backgrounds = ["CmBkDrDd.pcx", "CmBkDrMt.pcx", "CmBkDrTr.pcx"]

[[obstacle]]
key = "obstacle_0"
def = "ObDino1.def"
rows = 1
layer = 1

[[obstacle]]
key = "obstacle_100"
pcx = "ObDtL04.pcx"
at = [124, 254]
"#;

    fn file(id: &str, depends: &[&str]) -> TableFile {
        TableFile {
            table: Header {
                id: id.to_string(),
                depends: depends.iter().map(|d| (*d).to_string()).collect(),
            },
            battle: None,
            creature: Vec::new(),
            hero: Vec::new(),
            spell: Vec::new(),
            battlefield: Vec::new(),
            obstacle: Vec::new(),
            fix: Vec::new(),
        }
    }

    fn base() -> TableFile {
        parse(BASE).unwrap()
    }

    fn obstacle_entry(key: &str) -> ObstacleEntry {
        ObstacleEntry {
            key: key.to_string(),
            def: None,
            rows: None,
            layer: None,
            pcx: None,
            at: None,
        }
    }

    fn fix(id: &str) -> Fix {
        Fix {
            id: id.to_string(),
            source: "a reference frame".to_string(),
            target: "base:pikeman".to_string(),
            summary: "one pixel".to_string(),
        }
    }

    fn rule(files: &[TableFile], top: &str) -> TableError {
        load(files, top).unwrap_err()
    }

    #[test]
    fn a_selection_merges_its_tables_dependencies_first() {
        let mut ab = file("ab", &["base"]);
        ab.creature.push(base().creature[0].clone());
        ab.fix.push(fix("CLT-QUIRK-1"));
        let mut sod = file("sod", &["ab", "base"]);
        sod.obstacle.push(ObstacleEntry {
            def: Some("ObDino2.def".to_string()),
            rows: Some(2),
            layer: Some(0),
            ..obstacle_entry("obstacle_1")
        });
        let files = vec![sod, ab, base()];
        let t = load(&files, "sod").unwrap();
        assert_eq!(t.order, ["base", "ab", "sod"]);
        assert_eq!(t.battle.groups.standing, 2);
        assert_eq!(t.battle.hero_groups.idle, 1);
        assert_eq!(t.heroes["base:orrin"].def, "CH00.DEF");
        assert_eq!(t.heroes["base:orrin"].cast, [92, 67]);
        assert_eq!(t.heroes["base:orrin"].cast_frames, 5);
        assert_eq!(t.spells["base:slow"].animation, Some(1));
        assert_eq!(t.spells["base:slow"].sound.as_deref(), Some("Slow"));
        assert_eq!(t.battle.resist_sound.as_deref(), Some("MagicRes"));
        assert_eq!(
            t.battle.sounds().collect::<Vec<_>>(),
            [
                "Regener", "MagicRes", "GoodMrle", "BadMrle", "Fear", "goodluck", "DrainLif",
                "LightBlt"
            ]
        );
        assert_eq!(
            t.creatures.keys().collect::<Vec<_>>(),
            ["ab:pikeman", "base:archer", "base:pikeman"]
        );
        let archer = &t.creatures["base:archer"];
        assert_eq!(archer.missile.as_deref(), Some("PLCBOWX.DEF"));
        assert_eq!(archer.missile_offsets[2], [21, -50]);
        assert_eq!(archer.missile_angles[8], -90);
        assert_eq!(archer.troop_count_offset, -3);
        let pikeman = &t.creatures["base:pikeman"];
        assert_eq!(
            (pikeman.times, pikeman.attack_climax),
            ([9000, 575, 500, 115], 0)
        );
        assert_eq!(pikeman.missile_angles, [0; 12]);
        assert_eq!(t.battlefields["base:dirt"].backgrounds.len(), 3);
        assert_eq!(
            t.obstacles["base:obstacle_0"],
            ObstacleArt::Usual {
                def: "ObDino1.def".to_string(),
                rows: 1,
                before_stacks: true
            }
        );
        assert_eq!(
            t.obstacles["sod:obstacle_1"],
            ObstacleArt::Usual {
                def: "ObDino2.def".to_string(),
                rows: 2,
                before_stacks: false
            }
        );
        assert_eq!(
            t.obstacles["base:obstacle_100"],
            ObstacleArt::Absolute {
                pcx: "ObDtL04.pcx".to_string(),
                at: [124, 254]
            }
        );
        assert_eq!(t.fixes["CLT-QUIRK-1"].target, "base:pikeman");
        let names: Vec<String> = t.names().into_iter().collect();
        assert_eq!(
            names,
            [
                "C10SPW.DEF",
                "C20SPX0.DEF",
                "C20SPX1.DEF",
                "CH00.DEF",
                "CLCBOW.DEF",
                "CMBKDRDD.PCX",
                "CMBKDRMT.PCX",
                "CMBKDRTR.PCX",
                "CMFLAGL.DEF",
                "CMFLAGR.DEF",
                "CPKMAN.DEF",
                "OBDINO1.DEF",
                "OBDINO2.DEF",
                "OBDTL04.PCX",
                "PLCBOWX.DEF",
                "SP12_.DEF"
            ]
        );
        // The magic animations by number, the tent's heal sound (E17 M3 T3b).
        assert_eq!(
            t.battle.magic[1],
            Magic {
                def: "sp12_.def".to_string(),
                place: 1,
                translucent: false
            }
        );
        assert!(t.battle.magic[0].translucent);
        assert_eq!(t.battle.heal_sound, "Regener");
        // A selection loads only what it depends on.
        assert_eq!(load(&files, "base").unwrap().order, ["base"]);
    }

    #[test]
    fn unknown_or_missing_fields_are_errors() {
        for text in [
            "[table]\nid = \"x\"\ndepends = []\n[sounds]\n",
            "[table]\nid = \"x\"\ndepends = []\nextra = 1\n",
            "[table]\nid = \"x\"\n",
            "[table]\nid = \"x\"\ndepends = []\n[[creature]]\nkey = \"a\"\ndef = \"A.DEF\"\n",
            "not toml",
        ] {
            let error = parse(text).unwrap_err().to_string();
            assert!(error.starts_with("not a table: "), "{text}");
        }
    }

    #[test]
    fn the_dependency_graph_is_checked() {
        assert_eq!(
            rule(&[base(), base()], "base"),
            TableError::DuplicateTable("base".to_string())
        );
        assert_eq!(
            rule(&[base()], "ab"),
            TableError::UnknownTable("ab".to_string())
        );
        assert_eq!(
            rule(&[base(), file("ab", &["hota"])], "ab"),
            TableError::UnknownTable("hota".to_string())
        );
        let looped = [file("a", &["b"]), file("b", &["a"])];
        assert_eq!(rule(&looped, "a"), TableError::Cycle("a".to_string()));
        assert_eq!(rule(&[file("a", &[])], "a"), TableError::Battle(0));
        let mut twice = file("ab", &["base"]);
        twice.battle = base().battle;
        assert_eq!(rule(&[base(), twice], "ab"), TableError::Battle(2));
    }

    #[test]
    fn keys_are_unique_and_unqualified() {
        let mut b = base();
        b.creature.push(b.creature[0].clone());
        assert_eq!(
            rule(&[b], "base"),
            TableError::DuplicateKey("base:pikeman".to_string())
        );
        type Rekey = fn(&mut TableFile, &str);
        let lists: [Rekey; 5] = [
            |b, k| b.creature[0].key = k.to_string(),
            |b, k| b.hero[0].key = k.to_string(),
            |b, k| b.battlefield[0].key = k.to_string(),
            |b, k| b.obstacle[0].key = k.to_string(),
            |b, k| b.spell[0].key = k.to_string(),
        ];
        for rekey in lists {
            for (key, shown) in [("", "base:"), ("base:x", "base:base:x")] {
                let mut b = base();
                rekey(&mut b, key);
                assert_eq!(
                    rule(&[b], "base").to_string(),
                    format!("`{shown}`: a key is a pack's key, without its namespace")
                );
            }
        }
        let mut b = base();
        b.hero.push(b.hero[0].clone());
        assert_eq!(
            rule(&[b], "base"),
            TableError::DuplicateKey("base:orrin".to_string())
        );
        let mut b = base();
        b.spell.push(b.spell[0].clone());
        assert_eq!(
            rule(&[b], "base"),
            TableError::DuplicateKey("base:slow".to_string())
        );
        let mut b = base();
        b.battlefield.push(b.battlefield[0].clone());
        assert_eq!(
            rule(&[b], "base"),
            TableError::DuplicateKey("base:dirt".to_string())
        );
        let mut b = base();
        b.obstacle[1].key = "obstacle_0".to_string();
        assert_eq!(
            rule(&[b], "base"),
            TableError::DuplicateKey("base:obstacle_0".to_string())
        );
        let mut b = base();
        b.fix = vec![fix("CLT-QUIRK-1"), fix("CLT-QUIRK-1")];
        assert_eq!(
            rule(&[b], "base"),
            TableError::DuplicateKey("CLT-QUIRK-1".to_string())
        );
        let mut b = base();
        b.fix = vec![fix("BAT-QUIRK-1")];
        assert_eq!(
            rule(&[b], "base").to_string(),
            "`BAT-QUIRK-1`: a fix's id is CLT-QUIRK-<n>"
        );
    }

    #[test]
    fn names_and_shapes_are_checked() {
        type Change = fn(&mut TableFile);
        let shape = "an obstacle has def, rows and layer, or pcx and at";
        let sound = "a sound is an entry's name, without an extension";
        let broken: [(Change, &str, &str); 13] = [
            (
                |b| b.hero[0].def = "CH00.PCX".to_string(),
                "orrin",
                "`CH00.PCX` is not a DEF name",
            ),
            (
                |b| b.creature[0].sounds.wince = Some("pikewnce.wav".to_string()),
                "pikeman",
                sound,
            ),
            (
                |b| b.creature[1].sounds.end = Some(String::new()),
                "archer",
                sound,
            ),
            (
                |b| b.creature[0].def = "CPKMAN.PCX".to_string(),
                "pikeman",
                "`CPKMAN.PCX` is not a DEF name",
            ),
            (
                |b| b.creature[0].def = ".def".to_string(),
                "pikeman",
                "`.def` is not a DEF name",
            ),
            (
                |b| b.creature[1].missile = Some("arrow".to_string()),
                "archer",
                "`arrow` is not a DEF name",
            ),
            (
                |b| b.battlefield[0].backgrounds.clear(),
                "dirt",
                "no background",
            ),
            (
                |b| b.battlefield[0].backgrounds[1] = "CmBkDrMt.def".to_string(),
                "dirt",
                "`CmBkDrMt.def` is not a PCX name",
            ),
            (
                |b| b.obstacle[0].rows = Some(0),
                "obstacle_0",
                "rows from 1, layer 0 or 1",
            ),
            (
                |b| b.obstacle[0].layer = Some(2),
                "obstacle_0",
                "rows from 1, layer 0 or 1",
            ),
            (
                |b| b.obstacle[0].def = Some("ObDino1.pcx".to_string()),
                "obstacle_0",
                "`ObDino1.pcx` is not a DEF name",
            ),
            (
                |b| b.obstacle[1].pcx = Some("ObDtL04.def".to_string()),
                "obstacle_100",
                "`ObDtL04.def` is not a PCX name",
            ),
            (|b| b.obstacle[1].rows = Some(1), "obstacle_100", shape),
        ];
        for (change, key, message) in broken {
            let mut b = base();
            change(&mut b);
            assert_eq!(
                rule(&[b], "base"),
                TableError::Entry(format!("base:{key}"), message.to_string())
            );
        }
        let mut b = base();
        b.battle.as_mut().unwrap().flags[1] = "CmFlagR.pcx".to_string();
        assert_eq!(
            rule(&[b], "base").to_string(),
            "`[battle]`: `CmFlagR.pcx` is not a DEF name"
        );
        let mut b = base();
        b.battle.as_mut().unwrap().magic[1].def = "sp12_.pcx".to_string();
        assert_eq!(
            rule(&[b], "base").to_string(),
            "`[battle]`: `sp12_.pcx` is not a DEF name"
        );
        for sound in ["", "Regener.wav"] {
            let mut b = base();
            b.battle.as_mut().unwrap().heal_sound = sound.to_string();
            assert_eq!(
                rule(&[b], "base").to_string(),
                "`[battle]`: a sound is an entry's name, without an extension"
            );
        }
        let mut b = base();
        b.battle.as_mut().unwrap().resist_sound = Some("MagicRes.wav".to_string());
        assert_eq!(
            rule(&[b], "base").to_string(),
            "`[battle]`: a sound is an entry's name, without an extension"
        );
        let mut b = base();
        b.battle.as_mut().unwrap().luck_sound = Some("goodluck.82m".to_string());
        assert_eq!(
            rule(&[b], "base").to_string(),
            "`[battle]`: a sound is an entry's name, without an extension"
        );
        for sound in ["", "Slow.wav"] {
            let mut b = base();
            b.spell[0].sound = Some(sound.to_string());
            assert_eq!(
                rule(&[b], "base").to_string(),
                "`base:slow`: a sound is an entry's name, without an extension"
            );
        }
        let mut b = base();
        b.spell[0].animation = Some(2);
        assert_eq!(
            rule(&[b], "base").to_string(),
            "`base:slow`: no such magic animation"
        );
        let mut b = base();
        b.spell[1].projectile_angles.pop();
        assert_eq!(
            rule(&[b], "base").to_string(),
            "`base:magic_arrow`: an angle for each projectile DEF"
        );
        let mut b = base();
        b.spell[1].projectile[0] = "c20spx0.pcx".to_string();
        assert_eq!(
            rule(&[b], "base").to_string(),
            "`base:magic_arrow`: `c20spx0.pcx` is not a DEF name"
        );
        // E17 M3 T7c-2: a spell's objects name DEFs and magic animations, its object sound an
        // entry; both are among the tables' names.
        let object = |def: &str, placed, vanish| SpellObject {
            def: def.to_string(),
            hexes: 1,
            rows: 1,
            before_stacks: true,
            placed,
            vanish,
        };
        let mut b = base();
        b.spell[0].objects = vec![object("C17SPE1.DEF", 0, 1)];
        b.spell[0].object_sound = Some("Quiksand".to_string());
        let tables = load(&[b], "base").unwrap();
        assert!(tables.names().contains("C17SPE1.DEF"));
        assert!(tables.sound_names().contains("QUIKSAND"));
        for (def, placed, vanish, error) in [
            ("C17SPE1.PCX", 0, 0, "`C17SPE1.PCX` is not a DEF name"),
            ("C17SPE1.DEF", 9, 0, "no such magic animation"),
            ("C17SPE1.DEF", 0, 9, "no such magic animation"),
        ] {
            let mut b = base();
            b.spell[0].objects = vec![object(def, placed, vanish)];
            assert_eq!(
                rule(&[b], "base").to_string(),
                format!("`base:slow`: {error}")
            );
        }
        let mut b = base();
        b.spell[0].object_sound = Some("Quiksand.wav".to_string());
        assert_eq!(
            rule(&[b], "base").to_string(),
            "`base:slow`: a sound is an entry's name, without an extension"
        );
    }

    #[test]
    fn a_shooter_names_its_explosion() {
        // E17 M3 T7c-3: a Death Cloud as the Liches' (on the test's archer).
        let tables = load(&[base()], "base").unwrap();
        let explosion = |key: &str| tables.creatures[key].explosion;
        assert_eq!(
            explosion("base:archer"),
            Some(CreatureExplosion {
                spell: 76,
                time: 100
            })
        );
        assert_eq!(explosion("base:pikeman"), None);
    }

    #[test]
    fn a_creature_names_its_sounds() {
        let tables = load(&[base()], "base").unwrap();
        let pikeman = &tables.creatures["base:pikeman"].sounds;
        assert_eq!(pikeman.name(Sound::Move), Some("pikemove"));
        assert_eq!(pikeman.name(Sound::Attack), Some("pikeattk"));
        assert_eq!(pikeman.name(Sound::Wince), Some("pikewnce"));
        assert_eq!(pikeman.name(Sound::Death), Some("pikekill"));
        assert_eq!(pikeman.name(Sound::Defend), Some("pikedfnd"));
        for none in [
            Sound::Shot,
            Sound::Start,
            Sound::End,
            Sound::Heal,
            Sound::GoodMorale,
            Sound::BadMorale,
            Sound::Fear,
            Sound::Luck,
            Sound::Drain,
            Sound::Thunderbolt,
            Sound::SpellFile(79),
        ] {
            assert_eq!(pikeman.name(none), None);
        }
        let all = Sounds {
            shot: Some("s".to_string()),
            start: Some("a".to_string()),
            end: Some("e".to_string()),
            ..Sounds::default()
        };
        assert_eq!(all.name(Sound::Shot), Some("s"));
        assert_eq!(all.name(Sound::Start), Some("a"));
        assert_eq!(all.name(Sound::End), Some("e"));
        // The archer has none; the tent's heal sound is the battle's (E17 M3 T3b), and so are
        // morale's, Fear's and good luck's (E17 M3 T7b-2b-2b), Life Drain's and the Thunderbird's
        // bolt's (E17 M3 T7b-2b-3).
        let names: Vec<String> = tables.sound_names().into_iter().collect();
        assert_eq!(
            names,
            [
                "BADMRLE", "DRAINLIF", "FEAR", "GOODLUCK", "GOODMRLE", "LIGHTBLT", "MAGICRES",
                "PIKEATTK", "PIKEDFND", "PIKEKILL", "PIKEMOVE", "PIKEWNCE", "REGENER", "SLOW"
            ]
        );
    }

    #[test]
    fn errors_say_what_broke() {
        let shown: Vec<String> = [
            TableError::Syntax("x".to_string()),
            TableError::DuplicateTable("a".to_string()),
            TableError::UnknownTable("a".to_string()),
            TableError::Cycle("a".to_string()),
            TableError::Battle(2),
            TableError::DuplicateKey("a:b".to_string()),
            TableError::Entry("a:b".to_string(), "rule".to_string()),
        ]
        .iter()
        .map(|e| format!("{e}"))
        .collect();
        assert_eq!(
            shown,
            [
                "not a table: x",
                "two tables have the id `a`",
                "no table has the id `a`",
                "the dependencies go round through `a`",
                "2 tables of the selection hold [battle], not one",
                "`a:b` is defined twice",
                "`a:b`: rule",
            ]
        );
    }
}
