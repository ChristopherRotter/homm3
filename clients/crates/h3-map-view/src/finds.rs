//! The finds told (E24 M4 T2a2, *provisional*; `docs/epics/E24-design.md` §5.3): what a visit
//! with nothing to answer shows a human, an OK box (`message_box` 0x4F6570, `h3_ui::message::told`)
//! with its text and up to three pictures. [`picture`] is a picture slot's DEF, frame and caption
//! by its type and subtype (`message_box_picture` 0x4F4EB0, read whole); [`Find`] a find's facts
//! as the viewer knows them and [`Find::told`] its box's text and pictures through the string
//! catalogue (ADVEVENT's records, RANDSIGN's, GENRLTXT's and the names' families);
//! [`town_lost`] and [`days_left`] the turn's two boxes of a player without a town (GENRLTXT 7,
//! ArrayTxt 128 and 129, the player's crest).
//!
//! *Provisional* (the viewer has no engine; Part B the engine's events): a find's facts are what
//! the viewer knows of the map at the sequence's 0 ([`Find::of`]: the object's class, its entrance
//! word and its marks from the live part's `[hover]` facts, a sign's text from the map), and the
//! visit's results are not applied but for the find itself ([`Find::visited`]: told again it tells
//! the visited box). What the facts do not hold is taken as a new game's hero has it: a Witch
//! Hut's skill and a Shrine's spell not known unless the live part's contents say so, a Spell Book
//! carried and Wisdom enough, the Learning Stone's experience without the Learning skill
//! ([`FindArt::experience`]).
//!
//! *Not built:* a picture of a town's building (types 22 to 30, `town_building_name`), the
//! Magic Well's and the other finds' texts (no recording shows them), the finds' results on the
//! kingdom (the resources, the experience, the skills and spells: the engine's), a pile's, an
//! artifact's and an event's (T2a2's Part B).

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use h3_ui::text::{Args, Catalogue};
use serde::Deserialize;

use crate::hover::{Content, Visit};

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
}

/// The string catalogue's keys the finds and the turn's boxes show (`clients/text/keys.toml`);
/// the names are the families `resource`, `artifact`, `spell`, `player-colour`, `skill-level`,
/// `secondary-skill`, `primary-skill`, `creature` and `sign`'s.
pub const KEYS: [&str; 26] = [
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
    "map.find.witch-hut-empty",
    "map.find.shrine",
    "map.find.shrine-1",
    "map.find.shrine-2",
    "map.find.shrine-3",
    "map.find.shrine-known",
    "map.find.shrine-no-book",
    "map.find.shrine-wisdom",
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
    /// signed; none for −1) and whether the hero has it or eight skills.
    WitchHut {
        /// The skill.
        skill: Option<u8>,
        /// The hero has it (or eight skills).
        known: bool,
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
}

/// What a find's visit changes that the panels show (E24 M4 T2a2, *provisional*: the engine's
/// from Part B; recorded at the box's close, `find-windmill`'s resource bar, `find-axis`'s and
/// `find-temple`'s status window): the resource and amount the kingdom gains, the hero's primary
/// skills' gains, its morale's on another day and on the week's seventh.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Gain {
    /// The resource and the amount.
    pub resource: Option<(usize, i32)>,
    /// Attack, defense, power and knowledge.
    pub skills: [i8; 4],
    /// The morale on the week's first six days and on its seventh (`visit_temple` 0x4A6200).
    pub morale: [i8; 2],
}

/// A box's text and pictures (type and subtype each).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Told {
    /// The text.
    pub text: String,
    /// The pictures, in order.
    pub pictures: Vec<(u32, i32)>,
}

/// The classes a find told is (a Star Axis, a Shrine's three, a sign, the Temple, the Learning
/// Stone, a Windmill, a Witch Hut).
pub const CLASSES: [u32; 9] = [61, 88, 89, 90, 91, 96, 100, 112, 113];

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
    /// The find of an object of `class` (*provisional*: the viewer's facts at the sequence's 0):
    /// its entrance `word` (the live part's `[hover]` words: a Windmill's resource, a Witch Hut's
    /// skill, a Shrine's spell), the hero's mark `visit` (a Stone's, an Axis's, a Temple's),
    /// the `content` known (a Witch Hut's skill or a Shrine's spell the selected hero has), a
    /// sign's map `text` (empty: none) and its `column`. None for another class, or a word the
    /// find needs and the facts do not give.
    #[must_use]
    pub fn of(
        class: u32,
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
                Self::WitchHut { skill, known }
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
            _ => return None,
        })
    }

    /// The find as its visit leaves it (*provisional*, the engine's from Part B): a Windmill
    /// emptied, the marks set, the skill and the spell known; a sign as it was.
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
            Self::WitchHut { skill, .. } => Self::WitchHut {
                skill: *skill,
                known: skill.is_some(),
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
            Self::Sign { .. } => self.clone(),
        }
    }

    /// What the visit gives that the panels show ([`Gain`]): a Windmill's resource, a Star
    /// Axis's Spell Power, a Temple's morale; nothing visited, nothing else.
    #[must_use]
    pub fn gain(&self) -> Gain {
        match self {
            Self::Windmill { resource, amount } if *amount > 0 => Gain {
                resource: usize::try_from(*resource).ok().map(|r| (r, *amount)),
                ..Gain::default()
            },
            Self::Axis { visited: false } => Gain {
                skills: [0, 0, 1, 0],
                ..Gain::default()
            },
            Self::Temple { visited: false } => Gain {
                morale: [1, 2],
                ..Gain::default()
            },
            _ => Gain::default(),
        }
    }

    /// The box the visit shows (design §5.3's table): its text through `texts` and its pictures,
    /// the Learning Stone's experience `art`'s.
    #[must_use]
    pub fn told(&self, art: &FindArt, texts: &Catalogue) -> Told {
        let plain = |key: &str| texts.text(key, &Args::new());
        let alone = |key: &str| Told {
            text: plain(key),
            pictures: Vec::new(),
        };
        let with = |key: &str, picture: (u32, i32)| Told {
            text: plain(key),
            pictures: vec![picture],
        };
        match self {
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
            } => {
                let name = texts
                    .known(&format!("secondary-skill.{skill}"), &Args::new())
                    .unwrap_or_default();
                let args = Args::new().text("skill", &name);
                if *known {
                    Told {
                        text: texts.text("map.find.witch-hut-known", &args),
                        pictures: Vec::new(),
                    }
                } else {
                    Told {
                        text: texts.text("map.find.witch-hut", &args),
                        pictures: vec![(SECONDARY, 3 * i32::from(*skill) + 3)],
                    }
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
        }
    }
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
