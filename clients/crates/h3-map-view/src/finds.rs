//! The finds told (E24 M4 T2a2, *provisional*; `docs/epics/E24-design.md` §5.3): what a visit
//! with nothing to answer shows a human, an OK box (`message_box` 0x4F6570, `h3_ui::message::told`)
//! with its text and up to three pictures. [`picture`] is a picture slot's DEF, frame and caption
//! by its type and subtype (`message_box_picture` 0x4F4EB0, read whole); [`Find`] a find's facts
//! as the viewer knows them and [`Find::told`] its box's text and pictures through the string
//! catalogue (ADVEVENT's records, ARTEVENT's, RANDSIGN's, GENRLTXT's and the names' families);
//! [`town_lost`] and [`days_left`] the turn's two boxes of a player without a town (GENRLTXT 7,
//! ArrayTxt 128 and 129, the player's crest). Part B: a pile's kind 6 ([`Find::found`],
//! `status_resource_set` 0x416060), an artifact's plain pickup, the Magic Well's boxes, and
//! [`rewards`], an event's boxes of up to eight pictures (`grant_rewards` 0x49FA90).
//!
//! *Provisional* (the viewer has no engine): a find's facts are what the viewer knows of the map
//! at the sequence's 0 ([`Find::of`]: the object's class and subclass, its entrance word and its
//! marks from the live part's `[hover]` facts, a sign's text from the map), and the visit's
//! results are not applied but for the find itself ([`Find::visited`]: told again it tells the
//! visited box) and what the panels show ([`Gain`]). What the facts do not hold is taken as a new
//! game's hero has it: a Witch Hut's skill and a Shrine's spell not known unless the live part's
//! contents say so, fewer than eight skills, a Spell Book carried and Wisdom enough, the Learning
//! Stone's experience without the Learning skill ([`FindArt::experience`]); an event's skills
//! each learned, its spells each new, its artifacts each with room in the backpack, its creatures
//! each with a slot.
//!
//! *Not built:* a picture of a town's building (types 22 to 30, `town_building_name`), the
//! finds' results on the kingdom but those the panels show (the experience, the skills and
//! spells, the artifacts: the engine's), a level-up, Pandora's Box (its question: T2b), an
//! event's guards and an event with a text and no reward (`event_fire` 0x4A0ED0 not read past
//! `grant_rewards`).
//!
//! A flag raised (E24 M4 T2d1, design §5.6, *provisional*): a mine's and a Lighthouse's visit
//! by a hero not of their owner's team ([`Find::flag_of`]: no guards) tells its box (MINEEVNT's
//! record of the mine's resource with the resource's daily income, ADVEVENT 69), then the object
//! is the visitor's ([`Find::flagged`]: nothing told again). *Not built:* an abandoned mine
//! without guards (its record by the resource or 7 not read: it stays the question's,
//! `crate::asks`), a mine's or a Lighthouse's owner's own visit (the Leave Guards dialog, M5's),
//! the reveal of radius 3 around the entrance (nothing new shows: the step revealed the hero's
//! scouting radius) and the scenario's test after it.

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use h3_map::model::Rewards;
use h3_ui::text::{Args, Catalogue};
use serde::Deserialize;

use crate::hover::{Content, Visit};
use crate::live::{FoundStatus, Stack};

/// The pictures' DEFs by type and the finds' numbers (the tables' `[map.finds]`; design §5.3,
/// `message_box_picture` 0x4F4EB0).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FindArt {
    /// The resources' (types 0 to 7, 18 and 19: `resour82.def`, the frame the type).
    pub resources: String,
    /// The artifacts' (type 8: `artifact.def`, the frame the artifact).
    pub artifacts: String,
    /// The spells' (type 9: `spellscr.def`, the frame the spell).
    pub spells: String,
    /// The players' crests (type 10: `crest58.def`, the frame the colour).
    pub crests: String,
    /// The luck's (types 11 to 13: `ilck82.def`, frames 4, 3, 2).
    pub luck: String,
    /// The morale's (types 14 to 16: `imrl82.def`, frames 4, 3, 2).
    pub morale: String,
    /// The primary skills', the experience's and the spell points' (types 17, 31 to 35:
    /// `pskill.def`, frames 4, type − 31 and 5).
    pub skills: String,
    /// The secondary skills' (type 20: `secsk82.def`, the frame the subtype).
    pub secondary: String,
    /// The creatures' portraits (type 21: `twcrport.def`, the frame the creature + 2).
    pub creatures: String,
    /// Type 36's (`resource.def`, frame 6).
    pub gold: String,
    /// The Learning Stone's experience (`visit_learning_stone` 0x4A6330: 1 000 times the hero's
    /// Learning, `hero_learning`; *provisional*: the viewer knows no hero's skills, so 1 000).
    pub experience: i32,
    /// How long a pile's kind 6 is asked for, ms (`status_resource_set` 0x416060: 5 000; E24 M4
    /// T2a2 part B).
    pub found_ms: u32,
    /// An object taken (`visit_resource_pile` 0x4A4BE0, `artifact_pickup_remove` 0x49E170): the
    /// rectangle dissolved (192, 160, 224 × 224: the view's middle seven tiles) and the time of
    /// each of its eight steps (65 ms, `screen_dissolve` 0x602DC0's argument).
    pub dissolve: [i32; 4],
    /// The dissolve's step, ms.
    pub dissolve_ms: u32,
    /// The pickup's sounds (`pickup01` to `pickup07`, `pickup_sound_format`: the game's
    /// `random_between(1, 7)` at 0x4A4D18; the viewer's own choice, [`pickup`]).
    pub pickups: Vec<String>,
    /// A monster's removal's sound (`killfade.82M`: `hero_loss_effect` 0x4ACBB0 with 0; E24 M4
    /// T2b2 part A2, `crate::asks::Fade`).
    pub killfade: String,
    /// The artifacts' primary skills, each `[artifact, attack, defense, power, knowledge]` (the
    /// packs' `primary_skills` effects; *provisional*: the engine's in time, an artifact picked
    /// up taken as worn): what the hero's window shows at a pickup's end.
    pub artifact_skills: Vec<[i32; 5]>,
    /// Each resource's mine's daily income, in the resource bar's order (`mine_daily_income`
    /// 0x678288: 2, 1, 2, 1, 1, 1, 1 000; E24 M4 T2d1): a mine's box shows it as its picture's
    /// caption (GENRLTXT 4).
    pub mine_income: [i32; 7],
}

impl FindArt {
    /// The primary skills artifact `artifact` gives (none: not in the table).
    #[must_use]
    pub fn skills_of(&self, artifact: u16) -> [i8; 4] {
        self.artifact_skills
            .iter()
            .find(|row| row[0] == i32::from(artifact))
            .map_or([0; 4], |row| {
                [row[1], row[2], row[3], row[4]].map(|v| i8::try_from(v).unwrap_or(0))
            })
    }
}

/// The pickup's sound of the `n`th object taken (*provisional*, a review item: the game rolls
/// `random_between(1, 7)` at 0x4A4D18; the viewer, with no generator of the game's, takes them in
/// turn from the first, so a run is the same each time). None without sounds.
#[must_use]
pub fn pickup(art: &FindArt, n: usize) -> Option<&str> {
    n.checked_rem(art.pickups.len())
        .and_then(|i| art.pickups.get(i))
        .map(String::as_str)
}

/// The string catalogue's keys the finds and the turn's boxes show (`clients/text/keys.toml`);
/// the names are the families `resource`, `artifact`, `artifact-event`, `spell`, `player-colour`,
/// `skill-level`, `secondary-skill`, `primary-skill`, `creature`, `hero`, `sign`'s and (E24 M4
/// T2d1) `mine-event`'s, a mine's box (MINEEVNT's records); since E24 M4 T5b2 a mine's guards
/// beaten (ADVEVENT 85).
pub const KEYS: [&str; 46] = [
    "map.find.per-day",
    "map.find.level",
    "map.find.count",
    "map.find.plus",
    "map.find.spell-points",
    "map.find.windmill",
    "map.find.windmill-empty",
    "map.find.stone",
    "map.find.stone-visited",
    "map.find.axis",
    "map.find.axis-visited",
    "map.find.temple",
    "map.find.temple-visited",
    "map.find.witch-hut",
    "map.find.witch-hut-known",
    "map.find.witch-hut-full",
    "map.find.witch-hut-empty",
    "map.find.shrine",
    "map.find.shrine-1",
    "map.find.shrine-2",
    "map.find.shrine-3",
    "map.find.shrine-known",
    "map.find.shrine-no-book",
    "map.find.shrine-wisdom",
    "map.find.pile",
    "map.find.well",
    "map.find.well-again",
    "map.find.well-full",
    "map.find.reward-experience",
    "map.find.reward-mana-lost",
    "map.find.reward-mana",
    "map.find.reward-morale-lost",
    "map.find.reward-morale",
    "map.find.reward-luck-lost",
    "map.find.reward-luck",
    "map.find.reward-treasure-lost",
    "map.find.reward-treasure",
    "map.find.reward-spell",
    "map.find.reward-spells",
    "map.find.reward-creature",
    "map.find.reward-creatures",
    "map.find.lighthouse",
    "map.find.mine-guards",
    "map.turn.town-lost",
    "map.turn.days-left",
    "map.turn.last-day",
];

/// The picture types (`message_box_picture`'s switch): the resources' first and last, the
/// artifact, the spell, the crest, the luck's and the morale's, the experience, the secondary
/// skill, the creature, the town's buildings', the primary skills', the spell points, type 36.
const RESOURCES: core::ops::RangeInclusive<u32> = 0..=6;
const ARTIFACT: u32 = 8;
const SPELL: u32 = 9;
/// A player's crest (type 10).
pub const CREST: u32 = 10;
const LUCK: core::ops::RangeInclusive<u32> = 11..=13;
const MORALE: core::ops::RangeInclusive<u32> = 14..=16;
const EXPERIENCE: u32 = 17;
const SECONDARY: u32 = 20;
const CREATURE: u32 = 21;
const PRIMARY: core::ops::RangeInclusive<u32> = 31..=34;
const SPELL_POINTS: u32 = 35;
const GOLD: u32 = 36;
/// Good morale, the Temple's (type 14).
const GOOD_MORALE: u32 = 14;
/// Spell Power, the Star Axis's (type 33).
const SPELL_POWER: u32 = 33;
/// The last creature a type 21's caption names (150).
const LAST_CREATURE: i32 = 150;
/// The subtype below which a resource's caption is its amount + 100 000 (−100 000).
const BELOW: i32 = -100_000;

/// A picture as a box shows it (a slot of 0x4F4EB0): its DEF, frame and caption.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shown {
    /// The DEF.
    pub def: String,
    /// Its frame of group 0.
    pub frame: u32,
    /// The caption under it (empty: none).
    pub caption: String,
}

/// The picture of `kind` and `subtype` (`message_box_picture` 0x4F4EB0, read whole; design
/// §5.3): its DEF from `art`, its frame and caption through `texts`; none for a town's building
/// (22 to 30, not built) and a type past 36.
#[must_use]
pub fn picture(art: &FindArt, texts: &Catalogue, kind: u32, subtype: i32) -> Option<Shown> {
    let shown = |def: &str, frame: u32, caption: String| {
        Some(Shown {
            def: String::from(def),
            frame,
            caption,
        })
    };
    let plain = |key: &str| texts.text(key, &Args::new());
    let named = |key: String| texts.known(&key, &Args::new()).unwrap_or_default();
    let frame = u32::try_from(subtype).unwrap_or(0);
    match kind {
        k if RESOURCES.contains(&k) => shown(&art.resources, k, amount(texts, subtype)),
        7 | 18 | 19 => shown(&art.resources, kind, String::new()),
        ARTIFACT => shown(&art.artifacts, frame, named(format!("artifact.{subtype}"))),
        SPELL => shown(&art.spells, frame, named(format!("spell.{subtype}"))),
        CREST => shown(&art.crests, frame, colour(texts, subtype)),
        k if LUCK.contains(&k) => shown(&art.luck, 15 - k, String::new()),
        k if MORALE.contains(&k) => shown(&art.morale, 18 - k, String::new()),
        EXPERIENCE => {
            let caption = match subtype {
                -1 => plain("map.find.level"),
                0 => String::new(),
                n => decimal(texts, n),
            };
            shown(&art.skills, 4, caption)
        }
        SECONDARY => {
            // The level by the subtype mod 3, a space, the skill by the subtype / 3 − 1.
            let level = named(format!("skill-level.{}", subtype.rem_euclid(3)));
            let skill = named(format!("secondary-skill.{}", subtype / 3 - 1));
            shown(&art.secondary, frame, format!("{level} {skill}"))
        }
        CREATURE => {
            let (creature, count) = (subtype & 0xFFFF, subtype >> 16);
            let frame = u32::try_from(creature + 2).unwrap_or(0);
            let caption = if (0..=LAST_CREATURE).contains(&creature) {
                let key = format!("creature.{creature}");
                let name = texts
                    .known(&key, &Args::new().count(if count == 1 { 1 } else { 2 }))
                    .unwrap_or_default();
                if count == 0 {
                    name
                } else {
                    count_name(texts, count, &name)
                }
            } else {
                String::new()
            };
            shown(&art.creatures, frame, caption)
        }
        k if PRIMARY.contains(&k) => {
            let name = named(format!("primary-skill.{}", k - 31));
            let caption = match subtype {
                0 => String::new(),
                // *Provisional* (no recording): the count the low word's.
                n if n >> 16 == 1 => count_name(texts, n & 0xFFFF, &name),
                n => {
                    let args = Args::new()
                        .number("amount", i64::from(n))
                        .text("name", &name);
                    texts.text("map.find.plus", &args)
                }
            };
            shown(&art.skills, k - 31, caption)
        }
        SPELL_POINTS => shown(
            &art.skills,
            5,
            count_name(texts, subtype, &plain("map.find.spell-points")),
        ),
        GOLD => shown(&art.gold, 6, amount(texts, subtype)),
        _ => None,
    }
}

/// A number by the executable's `%d` (`decimal_format` 0x660A1C: the resource bar's key).
fn decimal(texts: &Catalogue, n: i32) -> String {
    texts.text(
        "map.resources.amount",
        &Args::new().number("amount", i64::from(n)),
    )
}

/// A resource's caption (types 0 to 6 and 36): above 0 the amount, 0 none, below −100 000 the
/// subtype + 100 000, else GENRLTXT 4 (a mine's "%d/day") with its negation.
fn amount(texts: &Catalogue, subtype: i32) -> String {
    match subtype {
        0 => String::new(),
        n if n > 0 => decimal(texts, n),
        n if n < BELOW => decimal(texts, n - BELOW),
        n => texts.text(
            "map.find.per-day",
            &Args::new().number("amount", -i64::from(n)),
        ),
    }
}

/// `count_name_format` 0x6778A4 ("%d %s") with `count` and `name`.
fn count_name(texts: &Catalogue, count: i32, name: &str) -> String {
    let args = Args::new()
        .number("amount", i64::from(count))
        .text("name", name);
    texts.text("map.find.count", &args)
}

/// The colour `colour`'s name (`player_colour_names` 0x6A7DF8: PlColors.txt's, its first letter
/// upper-cased as recorded, `turn-no-town`'s "Red"); none past the eight.
fn colour(texts: &Catalogue, colour: i32) -> String {
    let name = texts
        .known(&format!("player-colour.{colour}"), &Args::new())
        .unwrap_or_default();
    let mut chars = name.chars();
    chars.next().map_or_else(String::new, |first| {
        let mut out = String::from(first.to_ascii_uppercase());
        out.push_str(chars.as_str());
        out
    })
}

/// A Shrine's refusal (`visit_shrine` 0x4A5610): the text appended, no picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The hero knows the spell (ADVEVENT 174).
    Known,
    /// No Spell Book (ADVEVENT 131).
    NoBook,
    /// The spell's level above the hero's Wisdom + 2 (ADVEVENT 130).
    Wisdom,
}

/// A find's facts as the viewer knows them (design §5.3's table; *provisional*: [`Find::of`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Find {
    /// A Windmill (112): its resource and amount (`visit_windmill` 0x4A7FC0: the entrance word's
    /// bits 0 to 3, signed, and 13 to 16); an amount of 0 is empty.
    Windmill {
        /// The resource.
        resource: i32,
        /// The amount.
        amount: i32,
    },
    /// A Learning Stone (100), visited by the hero or not.
    Stone {
        /// Visited.
        visited: bool,
    },
    /// A Star Axis (61), visited by the hero or not.
    Axis {
        /// Visited.
        visited: bool,
    },
    /// A Temple (96), visited by the hero or not.
    Temple {
        /// Visited.
        visited: bool,
    },
    /// A Witch Hut (113): its skill (`visit_witch_hut` 0x4A8080: the word's bits 13 to 19,
    /// signed; none for −1), whether the hero has it (ADVEVENT 172, 0x4A814F) and whether it has
    /// eight skills (173, 0x4A8183).
    WitchHut {
        /// The skill.
        skill: Option<u8>,
        /// The hero has it.
        known: bool,
        /// The hero has eight skills (*provisional*: the facts do not tell, so never).
        full: bool,
    },
    /// A Shrine (88 to 90): its level (1 to 3, `visit_object`'s text by the class), its spell
    /// (`visit_shrine` 0x4A5610: the word's bits 13 to 22, signed) and the hero's refusal.
    Shrine {
        /// The level.
        level: u8,
        /// The spell.
        spell: u16,
        /// The refusal, none: learned.
        refusal: Option<Refusal>,
    },
    /// A sign (91): the map's text, none for the default (`sign_default_texts` 0x696C74, RANDSIGN's
    /// record by its column mod 25: 0x4AA056).
    Sign {
        /// The map's text.
        text: Option<String>,
        /// Its column.
        column: i32,
    },
    /// A resource pile (79, E24 M4 T2a2 part B): its resource (the subclass) and the amount the
    /// visit gives (`visit_resource_pile` 0x4A4BE0: the entrance word, gold's times 100). No box:
    /// kind 6 ([`Find::found`]), then taken off the map.
    Pile {
        /// The resource.
        resource: i32,
        /// The amount.
        amount: i32,
    },
    /// An artifact's plain pickup (5, `visit_artifact` 0x49F7E0's case 0: no guards, no
    /// message): the artifact (the subclass), its box, then taken off the map.
    Artifact {
        /// The artifact.
        artifact: u16,
    },
    /// A Magic Well (49, `visit_magic_well` 0x4A3730; the bonuses' survey §10.2): the hero's day
    /// mark (78), else its spell points at their maximum (79) or below (77), each alone.
    Well {
        /// The hero drank today.
        visited: bool,
        /// The hero's spell points at their maximum or above.
        full: bool,
    },
    /// A mine (53, E24 M4 T2d1, design §5.6: `visit_mine` 0x4A39A0) not of the visitor's team
    /// and without guards: its resource (its record's +1, the live part's contents), told by
    /// MINEEVNT's record of it with the resource's picture at minus its daily income; then the
    /// visitor's (`set_mine_owner` 0x4C66E0).
    Mine {
        /// The resource (0 to 6).
        resource: u8,
    },
    /// A Lighthouse (42, E24 M4 T2d1: `visit_object`'s case 42, 0x4A9479) not of the visitor's
    /// team: ADVEVENT 69 without a picture; then the visitor's.
    Lighthouse,
}

/// What a find's visit changes that the panels show (E24 M4 T2a2, *provisional*: the engine's
/// in time; recorded at the box's close, `find-windmill`'s resource bar, `find-axis`'s and
/// `find-temple`'s status window, `find-event`'s; a pile's at once, `find-pile`'s): the
/// resources the kingdom gains, the hero's primary skills' gains, its morale's on another day and
/// on the week's seventh, its luck's, its spell points' and the creatures joining it; since E24 M4
/// T2c the experience it gains, which no panel shows but whose levels' dialogs follow the visit.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Gain {
    /// The resources, in the resource bar's order.
    pub resources: [i32; 7],
    /// Attack, defense, power and knowledge.
    pub skills: [i8; 4],
    /// The morale on the week's first six days and on its seventh (`visit_temple` 0x4A6200).
    pub morale: [i8; 2],
    /// The luck.
    pub luck: i8,
    /// The spell points.
    pub mana: i32,
    /// The creatures joining the hero's army (each the slot of its kind, else the first empty
    /// one; *provisional*: one without room is dropped, `army_dialog_with` not built).
    pub army: [Option<Stack>; 7],
    /// The experience (E24 M4 T2c, design §5.5: the levels it reaches, `hero_level_up` 0x4DA720,
    /// each its dialog after the visit's boxes).
    pub experience: i32,
}

/// A box's text and pictures (type and subtype each).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Told {
    /// The text.
    pub text: String,
    /// The pictures, in order.
    pub pictures: Vec<(u32, i32)>,
}

/// The classes a find told is (an artifact, a Magic Well, a Star Axis, a resource pile, a
/// Shrine's three, a sign, the Temple, the Learning Stone, a Windmill, a Witch Hut).
pub const CLASSES: [u32; 12] = [5, 49, 61, 79, 88, 89, 90, 91, 96, 100, 112, 113];

/// An event's class (26): told by [`rewards`].
pub const EVENT: u32 = 26;

/// The classes whose visit raises the visitor's flag and tells it (E24 M4 T2d1, design §5.6:
/// [`Find::flag_of`]): a Lighthouse, a mine.
pub const FLAGS: [u32; 2] = [LIGHTHOUSE, MINE];
const LIGHTHOUSE: u32 = 42;
const MINE: u32 = 53;

/// The abandoned mine's subclass (7).
const ABANDONED: u32 = 7;

/// Gold, the resource a pile's word gives in hundreds (6).
const GOLD_PILE: i32 = 6;

/// `width` bits of `word` from bit `from`, signed.
fn signed(word: u32, from: u32, width: u32) -> i32 {
    let bits = i32::try_from((word >> from) & ((1 << width) - 1)).unwrap_or(0);
    let top = 1 << (width - 1);
    if bits & top == 0 {
        bits
    } else {
        bits - 2 * top
    }
}

impl Find {
    /// The find of an object of `class` and `subclass` (*provisional*: the viewer's facts at the
    /// sequence's 0): its entrance `word` (the live part's `[hover]` words: a Windmill's resource,
    /// a Witch Hut's skill, a Shrine's spell, a pile's amount), the hero's mark `visit` (a Stone's,
    /// an Axis's, a Temple's, a Well's), the `content` known (a Witch Hut's skill or a Shrine's
    /// spell the selected hero has), a sign's map `text` (empty: none) and its `column`. None for
    /// another class, or a word the find needs and the facts do not give. A Well's hero is below
    /// its maximum ([`Find::full`] says otherwise).
    #[must_use]
    pub fn of(
        [class, subclass]: [u32; 2],
        word: Option<u32>,
        visit: Option<Visit>,
        content: Option<&Content>,
        (text, column): (Option<&str>, i32),
    ) -> Option<Self> {
        let visited = visit.and_then(|v| v.visited) == Some(true);
        Some(match class {
            112 => {
                let word = word?;
                Self::Windmill {
                    resource: signed(word, 0, 4),
                    amount: i32::try_from((word >> 13) & 0xF).unwrap_or(0),
                }
            }
            100 => Self::Stone { visited },
            61 => Self::Axis { visited },
            96 => Self::Temple { visited },
            113 => {
                let skill = u8::try_from(signed(word?, 13, 7)).ok();
                let known = match content {
                    Some(Content::WitchHut {
                        skill: Some((s, has)),
                        ..
                    }) => Some(*s) == skill && *has,
                    _ => false,
                };
                Self::WitchHut {
                    skill,
                    known,
                    full: false,
                }
            }
            88..=90 => {
                let spell = u16::try_from(signed(word?, 13, 10)).ok()?;
                let known = matches!(
                    content,
                    Some(Content::Shrine { spell: Some((s, true)), .. }) if *s == spell
                );
                Self::Shrine {
                    level: u8::try_from(class - 87).unwrap_or(1),
                    spell,
                    refusal: known.then_some(Refusal::Known),
                }
            }
            91 => Self::Sign {
                text: text.filter(|t| !t.is_empty()).map(String::from),
                column,
            },
            79 => {
                let resource = i32::try_from(subclass).ok()?;
                let word = i32::try_from(word?).ok()?;
                let amount = if resource == GOLD_PILE {
                    word.saturating_mul(100)
                } else {
                    word
                };
                Self::Pile { resource, amount }
            }
            5 => Self::Artifact {
                artifact: u16::try_from(subclass).ok()?,
            },
            49 => Self::Well {
                visited,
                full: false,
            },
            _ => return None,
        })
    }

    /// The find of an object of `class` and `subclass` whose visit raises the visitor's flag (E24
    /// M4 T2d1, design §5.6; [`FLAGS`]), its owner `ours` when of the visitor's team: a mine of
    /// the live part's `content` without guards (`visit_mine` 0x4A39A0), a Lighthouse. None for
    /// another class, an owner of the visitor's team (nothing told: the Leave Guards dialog of a
    /// human's own mine is M5's), a mine with guards (its question: `crate::asks`) or without
    /// its contents, and (*provisional*, not read) an abandoned mine's.
    #[must_use]
    pub fn flag_of(
        [class, subclass]: [u32; 2],
        content: Option<&Content>,
        ours: bool,
    ) -> Option<Self> {
        if ours {
            return None;
        }
        match (class, content) {
            (LIGHTHOUSE, _) => Some(Self::Lighthouse),
            (MINE, Some(Content::Mine { resource, guards }))
                if guards.is_empty() && subclass != ABANDONED =>
            {
                Some(Self::Mine {
                    resource: *resource,
                })
            }
            _ => None,
        }
    }

    /// Whether the visit makes the object the visitor's (E24 M4 T2d1: a mine, a Lighthouse):
    /// nothing is told at a later visit.
    #[must_use]
    pub const fn flagged(&self) -> bool {
        matches!(self, Self::Mine { .. } | Self::Lighthouse)
    }

    /// A Magic Well's find with the hero's spell points `full` (at their maximum or above: the
    /// viewer's [`full_mana`]); any other find as it is.
    #[must_use]
    pub fn full(self, full: bool) -> Self {
        match self {
            Self::Well { visited, .. } => Self::Well { visited, full },
            other => other,
        }
    }

    /// The find as its visit leaves it (*provisional*, the engine's in time): a Windmill emptied,
    /// the marks set, the skill and the spell known, a Well drunk from when below its maximum (a
    /// full hero takes no mark); a sign, a pile and an artifact as they were (the last two taken
    /// off the map).
    #[must_use]
    pub fn visited(&self) -> Self {
        match self {
            Self::Windmill { resource, .. } => Self::Windmill {
                resource: *resource,
                amount: 0,
            },
            Self::Stone { .. } => Self::Stone { visited: true },
            Self::Axis { .. } => Self::Axis { visited: true },
            Self::Temple { .. } => Self::Temple { visited: true },
            Self::WitchHut { skill, full, .. } => Self::WitchHut {
                skill: *skill,
                known: skill.is_some(),
                full: *full,
            },
            Self::Shrine {
                level,
                spell,
                refusal,
            } => Self::Shrine {
                level: *level,
                spell: *spell,
                refusal: refusal.or(Some(Refusal::Known)),
            },
            Self::Well {
                visited: false,
                full: false,
            } => Self::Well {
                visited: true,
                full: true,
            },
            Self::Sign { .. }
            | Self::Pile { .. }
            | Self::Artifact { .. }
            | Self::Well { .. }
            | Self::Mine { .. }
            | Self::Lighthouse => self.clone(),
        }
    }

    /// Whether the visit takes the object off the map (a pile, an artifact: the hero stays beside
    /// it, the object dissolves; `visit_resource_pile` 0x4A4BE0, `artifact_pickup_remove`
    /// 0x49E170).
    #[must_use]
    pub const fn taken(&self) -> bool {
        matches!(self, Self::Pile { .. } | Self::Artifact { .. })
    }

    /// What the visit gives that the panels show ([`Gain`]): a Windmill's resource, a Star
    /// Axis's Spell Power, a Temple's morale, a pile's resource, an artifact's primary skills
    /// (`art`'s); since E24 M4 T2c the Learning Stone's experience (`art`'s, its levels after its
    /// box: `visit_learning_stone` 0x4A6330); nothing visited, nothing else (*provisional*: the
    /// Well's spell points not shown, the viewer knowing no maximum).
    #[must_use]
    pub fn gain(&self, art: &FindArt) -> Gain {
        let resources = |resource: i32, amount: i32| {
            let mut resources = [0; 7];
            if let Some(r) = usize::try_from(resource)
                .ok()
                .and_then(|r| resources.get_mut(r))
            {
                *r = amount;
            }
            Gain {
                resources,
                ..Gain::default()
            }
        };
        match self {
            Self::Windmill { resource, amount } | Self::Pile { resource, amount } => {
                resources(*resource, *amount)
            }
            Self::Axis { visited: false } => Gain {
                skills: [0, 0, 1, 0],
                ..Gain::default()
            },
            Self::Temple { visited: false } => Gain {
                morale: [1, 2],
                ..Gain::default()
            },
            Self::Artifact { artifact } => Gain {
                skills: art.skills_of(*artifact),
                ..Gain::default()
            },
            Self::Stone { visited: false } => Gain {
                experience: art.experience,
                ..Gain::default()
            },
            _ => Gain::default(),
        }
    }

    /// Kind 6 a pile's visit asks (`status_resource_set` 0x416060, design §5.3): its resource,
    /// its amount and ADVEVENT 113 with the resource's name, its first letter lower-cased
    /// (`resource_names`, `c_sprintf`); none for another find.
    #[must_use]
    pub fn found(&self, texts: &Catalogue) -> Option<FoundStatus> {
        let Self::Pile { resource, amount } = self else {
            return None;
        };
        let name = texts
            .known(&format!("resource.{resource}"), &Args::new())
            .unwrap_or_default();
        let mut chars = name.chars();
        let lowered = chars.next().map_or_else(String::new, |first| {
            let mut out = String::from(first.to_ascii_lowercase());
            out.push_str(chars.as_str());
            out
        });
        let text = texts.text("map.find.pile", &Args::new().text("resource", &lowered));
        Some(FoundStatus {
            resource: *resource,
            amount: *amount,
            text,
        })
    }

    /// The box the visit shows (design §5.3's table): its text through `texts` and its pictures,
    /// the Learning Stone's experience `art`'s; none for a pile (kind 6, [`Find::found`]).
    #[must_use]
    pub fn told(&self, art: &FindArt, texts: &Catalogue) -> Option<Told> {
        let plain = |key: &str| texts.text(key, &Args::new());
        let alone = |key: &str| Told {
            text: plain(key),
            pictures: Vec::new(),
        };
        let with = |key: &str, picture: (u32, i32)| Told {
            text: plain(key),
            pictures: vec![picture],
        };
        Some(match self {
            Self::Windmill { resource, amount } if *amount > 0 && *resource >= 0 => with(
                "map.find.windmill",
                (u32::try_from(*resource).unwrap_or(0), *amount),
            ),
            Self::Windmill { .. } => alone("map.find.windmill-empty"),
            Self::Stone { visited: true } => alone("map.find.stone-visited"),
            Self::Stone { visited: false } => with("map.find.stone", (EXPERIENCE, art.experience)),
            Self::Axis { visited: true } => alone("map.find.axis-visited"),
            Self::Axis { visited: false } => with("map.find.axis", (SPELL_POWER, 1)),
            Self::Temple { visited: true } => alone("map.find.temple-visited"),
            Self::Temple { visited: false } => with("map.find.temple", (GOOD_MORALE, 1)),
            Self::WitchHut { skill: None, .. } => alone("map.find.witch-hut-empty"),
            Self::WitchHut {
                skill: Some(skill),
                known,
                full,
            } => {
                let name = texts
                    .known(&format!("secondary-skill.{skill}"), &Args::new())
                    .unwrap_or_default();
                let args = Args::new().text("skill", &name);
                // Known is tested first (0x4A814F), then the eight skills (0x4A8183).
                let refused = match (known, full) {
                    (true, _) => Some("map.find.witch-hut-known"),
                    (false, true) => Some("map.find.witch-hut-full"),
                    (false, false) => None,
                };
                match refused {
                    Some(key) => Told {
                        text: texts.text(key, &args),
                        pictures: Vec::new(),
                    },
                    None => Told {
                        text: texts.text("map.find.witch-hut", &args),
                        pictures: vec![(SECONDARY, 3 * i32::from(*skill) + 3)],
                    },
                }
            }
            Self::Shrine {
                level,
                spell,
                refusal,
            } => {
                let name = texts
                    .known(&format!("spell.{spell}"), &Args::new())
                    .unwrap_or_default();
                let args = Args::new()
                    .text("text", &plain(&format!("map.find.shrine-{level}")))
                    .text("spell", &name);
                let mut text = texts.text("map.find.shrine", &args);
                match refusal {
                    Some(r) => {
                        text.push_str(&plain(match r {
                            Refusal::Known => "map.find.shrine-known",
                            Refusal::NoBook => "map.find.shrine-no-book",
                            Refusal::Wisdom => "map.find.shrine-wisdom",
                        }));
                        Told {
                            text,
                            pictures: Vec::new(),
                        }
                    }
                    None => Told {
                        text,
                        pictures: vec![(SPELL, i32::from(*spell))],
                    },
                }
            }
            Self::Sign { text, column } => Told {
                text: text
                    .clone()
                    .unwrap_or_else(|| plain(&format!("sign.{}", column.rem_euclid(SIGNS)))),
                pictures: Vec::new(),
            },
            Self::Pile { .. } => return None,
            // `artifact_pickup_texts` by the artifact (ARTEVENT's record, `artifact-event`).
            Self::Artifact { artifact } => Told {
                text: texts
                    .known(&format!("artifact-event.{artifact}"), &Args::new())
                    .unwrap_or_default(),
                pictures: vec![(ARTIFACT, i32::from(*artifact))],
            },
            Self::Well { visited: true, .. } => alone("map.find.well-again"),
            Self::Well { full: true, .. } => alone("map.find.well-full"),
            Self::Well { .. } => alone("map.find.well"),
            // MINEEVNT's record by the resource (`mine_event_texts` 0x6A5E20), the resource's
            // picture at minus its income: GENRLTXT 4's "%d/day" (`flag-mine`'s "2/day").
            Self::Mine { resource } => Told {
                text: texts
                    .known(&format!("mine-event.{resource}"), &Args::new())
                    .unwrap_or_default(),
                pictures: vec![(
                    u32::from(*resource),
                    art.mine_income
                        .get(usize::from(*resource))
                        .map_or(0, |income| income.saturating_neg()),
                )],
            },
            Self::Lighthouse => alone("map.find.lighthouse"),
        })
    }
}

/// Whether a hero's spell points `mana` are at their maximum or above, its Knowledge `knowledge`
/// (the bonuses' survey §10.3, HER-MAN-001: 10 times the Knowledge clamped to 1 to 99, times the
/// Intelligence's factor; *provisional*: the viewer knows no hero's skills, so none).
#[must_use]
pub fn full_mana(mana: i32, knowledge: i8) -> bool {
    mana >= 10 * i32::from(knowledge.clamp(1, 99))
}

/// One of an event's boxes (`grant_rewards` 0x49FA90's, shown by `message_box_pictures` 0x4F7690:
/// up to eight pictures; design §5.3): its text and pictures, and what the panels show as it
/// opens (the resources given before their box).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reward {
    /// The box's text and pictures.
    pub told: Told,
    /// What shows as it opens.
    pub shows: Gain,
}

/// The most pictures a box of [`rewards`] holds (`sub_4A0B00`'s 8).
const REWARD_SLOTS: usize = 8;

/// The boxes `grant_rewards` builds (design §5.3, the reading of 2026-11-17): the box's text the
/// map's at first, a part's own when empty as its first picture is pushed; a box shown when a
/// group ends holding a picture, or when it holds eight, its text then emptied.
struct Granted<'a> {
    texts: &'a Catalogue,
    hero: &'a str,
    text: String,
    pictures: Vec<(u32, i32)>,
    shows: Gain,
    boxes: Vec<Reward>,
}

impl Granted<'_> {
    /// The catalogue's text `key` with the hero's name.
    fn said(&self, key: &str) -> String {
        self.texts.text(key, &Args::new().text("hero", self.hero))
    }

    /// A part's picture pushed, the box's text the part's `text` when empty; a box of eight shown.
    fn push(&mut self, text: String, picture: (u32, i32)) {
        if self.text.is_empty() {
            self.text = text;
        }
        self.pictures.push(picture);
        if self.pictures.len() >= REWARD_SLOTS {
            self.flush();
        }
    }

    /// The box shown when it holds a picture, then emptied with its text.
    fn flush(&mut self) {
        if self.pictures.is_empty() {
            return;
        }
        self.boxes.push(Reward {
            told: Told {
                text: core::mem::take(&mut self.text),
                pictures: core::mem::take(&mut self.pictures),
            },
            shows: core::mem::take(&mut self.shows),
        });
    }
}

/// An event's boxes for the hero named `hero` (`grant_rewards` 0x49FA90, the human's path; design
/// §5.3): its parts in order, each a picture and a text (ADVEVENT 175 to 188 by the part): the
/// experience (17), the primary skills (31 + the skill), the secondary skills (20), the spell
/// points (35), the morale (14, 16) and the luck (11, 13); the resources (each type, a loss below
/// −100 000); the artifacts (8); the spells (9: two in a box tell 188, even over the map's
/// text); the creatures (21: 185 for one, 186 for more) — the map's `text` in the first box
/// alone; and what the panels show after the last box closes (*provisional*: the skills each
/// learned, the spells each new, the artifacts each with room; since E24 M4 T2c the experience,
/// given last with its levels: `hero_gain_experience`).
#[must_use]
pub fn rewards(
    rewards: &Rewards,
    texts: &Catalogue,
    hero: &str,
    text: &str,
) -> (Vec<Reward>, Gain) {
    let mut g = Granted {
        texts,
        hero,
        text: String::from(text),
        pictures: Vec::new(),
        shows: Gain::default(),
        boxes: Vec::new(),
    };
    let learns = g.said("map.find.reward-experience");
    let experience = i32::try_from(rewards.experience).unwrap_or(i32::MAX);
    if experience > 0 {
        g.push(learns.clone(), (EXPERIENCE, experience));
    }
    for (skill, &value) in (0u32..).zip(&rewards.primary) {
        if value > 0 {
            g.push(learns.clone(), (PRIMARY.start() + skill, i32::from(value)));
        }
    }
    for &(skill, level) in &rewards.skills {
        let subtype = 3 * i32::from(skill) + 2 + i32::from(level);
        g.push(learns.clone(), (SECONDARY, subtype));
    }
    if rewards.mana != 0 {
        let key = if rewards.mana < 0 {
            "map.find.reward-mana-lost"
        } else {
            "map.find.reward-mana"
        };
        let said = g.said(key);
        g.push(said, (SPELL_POINTS, rewards.mana));
    }
    for (value, [bad, good], [lost, gained]) in [
        (
            rewards.morale,
            [*MORALE.end(), *MORALE.start()],
            ["map.find.reward-morale-lost", "map.find.reward-morale"],
        ),
        (
            rewards.luck,
            [*LUCK.end(), *LUCK.start()],
            ["map.find.reward-luck-lost", "map.find.reward-luck"],
        ),
    ] {
        if value != 0 {
            let (kind, key) = if value < 0 {
                (bad, lost)
            } else {
                (good, gained)
            };
            let said = g.said(key);
            g.push(said, (kind, i32::from(value)));
        }
    }
    g.flush();
    // The resources' box shows them as it opens (the resource bar's redraw, `find-event`'s 1060).
    g.shows.resources = rewards.resources;
    for (kind, &amount) in (0u32..).zip(&rewards.resources) {
        if amount == 0 {
            continue;
        }
        let (subtype, key) = if amount < 0 {
            (
                amount.saturating_add(BELOW),
                "map.find.reward-treasure-lost",
            )
        } else {
            (amount, "map.find.reward-treasure")
        };
        let said = g.said(key);
        g.push(said, (kind, subtype));
    }
    g.flush();
    let treasure = g.said("map.find.reward-treasure");
    for &artifact in &rewards.artifacts {
        g.push(treasure.clone(), (ARTIFACT, i32::from(artifact)));
    }
    g.flush();
    let (one, more) = (
        g.said("map.find.reward-spell"),
        g.said("map.find.reward-spells"),
    );
    for &spell in &rewards.spells {
        // A box holding a picture already tells 188, whatever its text (0x4A05C4).
        if !g.pictures.is_empty() {
            g.text.clone_from(&more);
        }
        g.push(one.clone(), (SPELL, i32::from(spell)));
    }
    g.flush();
    let mut army: [Option<Stack>; 7] = [None; 7];
    for (stack, slot) in rewards.creatures.iter().zip(&mut army) {
        let creature = i32::from(stack.creature);
        let count = i32::from(stack.count);
        let name = |plural: bool| {
            (0..=LAST_CREATURE)
                .contains(&creature)
                .then(|| {
                    let args = Args::new().count(if plural { 2 } else { 1 });
                    texts.known(&format!("creature.{creature}"), &args)
                })
                .flatten()
                .unwrap_or_default()
        };
        let said = if count == 1 {
            let args = Args::new()
                .text("creature", &name(false))
                .text("hero", hero);
            texts.text("map.find.reward-creature", &args)
        } else {
            let args = Args::new()
                .text("creatures", &name(true))
                .text("hero", hero);
            texts.text("map.find.reward-creatures", &args)
        };
        g.push(said, (CREATURE, (count << 16) | creature));
        *slot = Some(Stack {
            creature: stack.creature,
            count,
        });
    }
    g.flush();
    let gain = Gain {
        experience: i32::try_from(rewards.experience).unwrap_or(i32::MAX),
        skills: rewards.primary.map(|v| i8::try_from(v).unwrap_or(i8::MAX)),
        morale: [rewards.morale; 2],
        luck: rewards.luck,
        mana: rewards.mana,
        army,
        ..Gain::default()
    };
    (g.boxes, gain)
}

/// The default signs' texts (RANDSIGN's 25 records, `sign_default_texts` 0x696C74).
const SIGNS: i32 = 25;

/// The player's name the turn's boxes open with (CLT-Q100 (b), as the client's shell names a
/// single scenario's player: the client's player name, the colour's when empty; the viewer has no
/// setting, so the colour's name): the original's empty name with `original` (`--bugs original`,
/// the comparisons').
fn player_name(texts: &Catalogue, player: u8, original: bool) -> String {
    if original {
        String::new()
    } else {
        colour(texts, i32::from(player))
    }
}

/// The box at a turn's end of `player` with heroes but no town (`scenario_condition_met`
/// 0x4F2CE0: GENRLTXT 7 with the player's name, the crest), its name the original's empty one
/// with `original`.
#[must_use]
pub fn town_lost(texts: &Catalogue, player: u8, original: bool) -> Told {
    let args = Args::new().text("player", &player_name(texts, player, original));
    Told {
        text: texts.text("map.turn.town-lost", &args),
        pictures: vec![(CREST, i32::from(player))],
    }
}

/// The box at a turn's start of `player` with `days` left to take a town (`player_turn_start`
/// 0x4CC7D0, the player's +0x3D at 0 or more: ArrayTxt 128 with the days, 129 on the last;
/// *provisional*: the last is one day left, not read), the crest.
#[must_use]
pub fn days_left(texts: &Catalogue, player: u8, days: i32, original: bool) -> Told {
    let name = player_name(texts, player, original);
    let text = if days > 1 {
        let args = Args::new()
            .text("player", &name)
            .number("days", i64::from(days));
        texts.text("map.turn.days-left", &args)
    } else {
        texts.text("map.turn.last-day", &Args::new().text("player", &name))
    };
    Told {
        text,
        pictures: vec![(CREST, i32::from(player))],
    }
}

#[cfg(test)]
mod tests;
