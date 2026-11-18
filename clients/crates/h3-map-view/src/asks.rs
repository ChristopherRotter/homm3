//! The questions a visit asks (E24 M4 T2b2 part A1, *provisional*; `docs/epics/E24-design.md`
//! §5.4): a box of style 2 (`message_box` 0x4F6570, `h3_ui::message::told` with its OK and
//! Cancel) that a visit shows a human before what it gives. [`Ask`] is a question's facts as the
//! viewer knows them ([`Ask::of`]), [`Ask::told`] its box's text and pictures through the string
//! catalogue (ADVEVENT's, GENRLTXT's and ARRAYTXT's records and the names' families), [`Ask::yes`]
//! what its OK shows where the viewer builds it: the Warrior's Tomb's find and a dwelling's
//! creatures joining free. A few boxes of the same handlers ask nothing (a looted bank's, the
//! Tree of Knowledge's visited and too poor): [`Ask::asks`] tells them.
//!
//! *Provisional* (the viewer has no engine): the facts are the viewer's at the sequence's 0 (the
//! object's class, subclass, entrance word and owner, the selected hero's mark, the kingdom's
//! resources and the hero's army, a pile's or an artifact's message from the map), a dwelling's a
//! new game's (its creature's week of growth to hire; its guards three weeks' growth of a creature
//! of level 5 or more: the dwellings' survey §2, the tables' [`AskArt`]), and the Tomb's morale is
//! lost at its first search (the hero's status bit 21 not known).
//!
//! *Not built:* every other answer (a battle, a level-up, the recruit dialog: [`Ask::yes`] none;
//! the engine's, M4 T5 and M5), a mine of another player with guards (ADVEVENT 187: none at a new
//! game), a guarded artifact's several stacks (", " and GENRLTXT 142 between them), a pile's and an
//! artifact's message without guards, a dwelling of class 20 (ADVEVENT 36) and the Refugee Camp
//! (112), a free Tree (148: its level-up at once), and part A2's questions (a monster's, a Border
//! Guard's, a Cartographer's, Pandora's Box's).

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use h3_ui::text::{Args, Catalogue};
use serde::Deserialize;

use crate::finds::{FindArt, Gain, Told};
use crate::hover::SIZES;
use crate::live::Stack;

/// The questions' numbers (the tables' `[map.asks]`; design §5.4).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskArt {
    /// Each creature's level, 1 to 7, by its number (`creature_table` 0x6703B8's +4, plus 1; 0
    /// where no creature has the number).
    pub levels: Vec<u8>,
    /// A dwelling's guards at a new game: the least level a creature guards its dwelling from (5)
    /// and the weeks of its growth they are (3; the dwellings' survey §2).
    pub guards: [i32; 2],
    /// The Tree of Knowledge's prices 1 and 2 (`visit_tree_of_knowledge` 0x4A6710, its word's bits
    /// 13 to 15), each `[resource, amount]`: 2 000 gold, 10 gems.
    pub tree_prices: Vec<[i32; 2]>,
    /// The morale the Warrior's Tomb's search costs (−3, `visit_warriors_tomb` 0x4A7C30).
    pub tomb_morale: i8,
}

impl AskArt {
    /// The level of creature `creature` (0: none known).
    #[must_use]
    pub fn level(&self, creature: u16) -> u8 {
        self.levels.get(usize::from(creature)).copied().unwrap_or(0)
    }
}

/// The string catalogue's keys the questions show (`clients/text/keys.toml`); the names are the
/// families `creature`, `artifact`, `bank` and `dwelling1`'s.
pub const KEYS: [&str; 30] = [
    "map.ask.size.0",
    "map.ask.size.1",
    "map.ask.size.2",
    "map.ask.size.3",
    "map.ask.size.4",
    "map.ask.size.5",
    "map.ask.size.6",
    "map.ask.size.7",
    "map.ask.size.8",
    "map.ask.artifact",
    "map.ask.guarded",
    "map.ask.mine",
    "map.ask.bank",
    "map.ask.bank-looted",
    "map.ask.derelict-ship",
    "map.ask.crypt",
    "map.ask.shipwreck",
    "map.ask.tree-gold",
    "map.ask.tree-gems",
    "map.ask.tree-visited",
    "map.ask.tree-poor-gold",
    "map.ask.tree-poor-gems",
    "map.ask.pyramid",
    "map.ask.tomb",
    "map.ask.tomb-found",
    "map.ask.tomb-empty",
    "map.ask.hire",
    "map.ask.joined",
    "map.ask.none",
    "map.ask.no-room",
];

/// The classes a question is asked at (as the game holds them): an artifact, a resource pile, a
/// spell scroll, a mine (the abandoned one its subclass 7), a creature bank, the search kinds
/// (the Derelict Ship, the Crypt, the Shipwreck), the Tree of Knowledge, the Pyramid, the
/// Warrior's Tomb, a dwelling of one creature.
const ARTIFACT: u32 = 5;
const PILE: u32 = 79;
const SCROLL: u32 = 93;
const MINE: u32 = 53;
const ABANDONED: u32 = 7;
const BANK: u32 = 16;
const DERELICT_SHIP: u32 = 24;
const CRYPT: u32 = 84;
const SHIPWRECK: u32 = 85;
const TREE: u32 = 102;
const PYRAMID: u32 = 63;
const TOMB: u32 = 108;
const DWELLING: u32 = 17;

/// A bank's word's looted bit (25, `visit_creature_bank` 0x4A15A0).
const LOOTED: u32 = 25;

/// The picture types the questions and their answers show: the experience's (17, subtype −1 its
/// "+1 Level"), an artifact's (8), the bad morale's (16).
const EXPERIENCE: (u32, i32) = (17, -1);
const ARTIFACT_PICTURE: u32 = 8;
const BAD_MORALE: (u32, i32) = (16, 0);

/// What the viewer knows of an object a question's facts come from (*provisional*: the live
/// part's at the sequence's 0, a dwelling's creature by the tables).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Seen {
    /// Its class and subclass as the game holds them.
    pub class: [u32; 2],
    /// Its entrance tile's word.
    pub word: Option<u32>,
    /// Its owner's colour.
    pub owner: Option<u8>,
    /// The selected hero's mark on it (a Tree's).
    pub visited: bool,
    /// A pile's, a scroll's or an artifact's message on the map: its text (empty: none) and its
    /// guards' stacks in slot order (none: no guards).
    pub message: Option<(String, Vec<Stack>)>,
    /// A dwelling's creature (`dwelling_creatures` 0x63D570 by the subclass) and its growth.
    pub dwelling: Option<(u16, i32)>,
}

/// The visiting player's facts a question reads (*provisional*: the live part's at 0).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Visitor {
    /// Its colour.
    pub colour: u8,
    /// Each colour's team.
    pub teams: [u8; 8],
    /// The kingdom's resources, in the resource bar's order.
    pub resources: [i32; 7],
    /// The selected hero's army (the status window's).
    pub army: [Option<Stack>; 7],
}

/// A question's facts (design §5.4's table; *provisional*: [`Ask::of`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ask {
    /// A guarded pile's, scroll's or artifact's with the map's text (`visit_guarded_resource`
    /// 0x4A4780, `visit_guarded_artifact` 0x49F070): the text; its OK the battle.
    Text(String),
    /// A guarded artifact's without a text (`visit_guarded_artifact` 0x49F070): GENRLTXT 421 with
    /// its one stack's size word (`creature_quantity_word` 0x44ADE0's form 2) and its name twice.
    Artifact(Stack),
    /// A neutral abandoned mine's (`visit_mine` 0x4A39A0: ADVEVENT 84, its Troglodytes in the
    /// text).
    Mine,
    /// A creature bank's (`visit_creature_bank` 0x4A15A0): its kind (the subclass) and whether it
    /// is looted (ADVEVENT 33, a box; else 32, the question).
    Bank {
        /// The kind.
        kind: u32,
        /// Looted.
        looted: bool,
    },
    /// A search kind's (`visit_search_bank` 0x4AC490: its class's first text, looted or not).
    Search(u32),
    /// The Tree of Knowledge's (`visit_tree_of_knowledge` 0x4A6710): its price in gems (else in
    /// gold), the hero's mark (ADVEVENT 147), the kingdom too poor (150, 152); else the price's
    /// question (149, 151) with the experience's picture.
    Tree {
        /// The price is in gems.
        gems: bool,
        /// The hero learned there.
        visited: bool,
        /// The kingdom cannot pay.
        poor: bool,
    },
    /// The Pyramid's (`visit_pyramid` 0x4A4230: ADVEVENT 105 at every visit).
    Pyramid,
    /// The Warrior's Tomb's (`visit_warriors_tomb` 0x4A7C30: ADVEVENT 161): its artifact (the
    /// word's bits 13 to 22 while its bit 0 is set), none.
    Tomb(Option<u16>),
    /// A dwelling's guards (`visit_dwelling` 0x4A18B0: GENRLTXT 422): the dwelling (the subclass)
    /// and its guards.
    Guarded {
        /// The dwelling.
        dwelling: u32,
        /// Its guards.
        guards: Stack,
    },
    /// A dwelling of the visitor's team (`visit_dwelling` 0x4A18B0: ADVEVENT 35): the dwelling,
    /// its creatures to hire, whether they join free (its own player's, level 1) and whether the
    /// hero's army has room for them.
    Hire {
        /// The dwelling.
        dwelling: u32,
        /// Its creatures.
        joins: Stack,
        /// They join free.
        free: bool,
        /// The army has a slot of their kind or an empty one.
        room: bool,
    },
}

impl Ask {
    /// The question an object of `seen` asks `visitor` (design §5.4), by the tables' `art`; none
    /// for another class or one this part does not build (*Not built* above).
    #[must_use]
    pub fn of(seen: &Seen, visitor: &Visitor, art: &AskArt) -> Option<Self> {
        let [class, subclass] = seen.class;
        let bit = |n: u32| seen.word.is_some_and(|w| (w >> n) & 1 == 1);
        Some(match class {
            ARTIFACT | PILE | SCROLL => {
                let (text, guards) = seen.message.as_ref()?;
                match guards.as_slice() {
                    [] => return None,
                    _ if !text.is_empty() => Self::Text(text.clone()),
                    [stack] if class == ARTIFACT => Self::Artifact(*stack),
                    _ => return None,
                }
            }
            MINE if subclass == ABANDONED && seen.owner.is_none() => Self::Mine,
            BANK => Self::Bank {
                kind: subclass,
                looted: bit(LOOTED),
            },
            DERELICT_SHIP | CRYPT | SHIPWRECK => Self::Search(class),
            TREE => {
                let price = seen.word.map_or(0, |w| (w >> 13) & 7);
                let cost = usize::try_from(price)
                    .ok()
                    .and_then(|p| art.tree_prices.get(p.checked_sub(1)?));
                match cost {
                    _ if seen.visited => Self::Tree {
                        gems: price == 2,
                        visited: true,
                        poor: false,
                    },
                    Some(&[resource, amount]) => {
                        let have = usize::try_from(resource)
                            .ok()
                            .and_then(|r| visitor.resources.get(r))
                            .copied()
                            .unwrap_or(0);
                        Self::Tree {
                            gems: price == 2,
                            visited: false,
                            poor: have < amount,
                        }
                    }
                    None => return None,
                }
            }
            PYRAMID => Self::Pyramid,
            TOMB => Self::Tomb(
                seen.word
                    .filter(|_| bit(0))
                    .and_then(|w| u16::try_from((w >> 13) & 0x3FF).ok()),
            ),
            DWELLING => {
                let (creature, growth) = seen.dwelling?;
                let level = art.level(creature);
                let team = |c: u8| visitor.teams.get(usize::from(c)).copied();
                let ours = seen.owner.is_some_and(|o| team(o) == team(visitor.colour));
                let [least, weeks] = art.guards;
                if ours {
                    let room = visitor
                        .army
                        .iter()
                        .any(|s| s.is_none_or(|s| s.creature == creature));
                    Self::Hire {
                        dwelling: subclass,
                        joins: Stack {
                            creature,
                            count: growth,
                        },
                        free: seen.owner == Some(visitor.colour) && level == 1,
                        room,
                    }
                } else if i32::from(level) >= least {
                    Self::Guarded {
                        dwelling: subclass,
                        guards: Stack {
                            creature,
                            count: weeks.saturating_mul(growth),
                        },
                    }
                } else {
                    return None;
                }
            }
            _ => return None,
        })
    }

    /// Whether its box is a question (style 2, its OK and Cancel); a looted bank's and the Tree's
    /// visited and too poor ones are an OK's alone (style 1).
    #[must_use]
    pub const fn asks(&self) -> bool {
        !matches!(
            self,
            Self::Bank { looted: true, .. }
                | Self::Tree { visited: true, .. }
                | Self::Tree { poor: true, .. }
        )
    }

    /// Whether its handler builds the status window again before its box (`status_window_choose`
    /// 0x415DE0 with 0, 1, 1: the pile's, the artifact's, the mine's, the bank's, the dwelling's
    /// and the Pyramid's; *provisional* for a dwelling's hire, as its guards').
    #[must_use]
    pub const fn chooses(&self) -> bool {
        !matches!(self, Self::Search(_) | Self::Tree { .. } | Self::Tomb(_))
    }

    /// Whether its object blocks its entrance (a pile, a scroll, an artifact): a key's step onto
    /// it visits it from beside.
    #[must_use]
    pub const fn beside(&self) -> bool {
        matches!(self, Self::Text(_) | Self::Artifact(_))
    }

    /// Its box's text and pictures (design §5.4's table) through `texts`.
    #[must_use]
    pub fn told(&self, texts: &Catalogue) -> Told {
        let alone = |text: String| Told {
            text,
            pictures: Vec::new(),
        };
        let plain = |key: &str| alone(texts.text(key, &Args::new()));
        match self {
            Self::Text(text) => alone(text.clone()),
            Self::Artifact(guards) => {
                let args = Args::new()
                    .text("size", &size(texts, guards.count))
                    .text("creature", &name(texts, guards.creature, 2));
                alone(texts.text("map.ask.artifact", &args))
            }
            Self::Mine => plain("map.ask.mine"),
            Self::Bank { kind, looted } => {
                let args = Args::new().text("bank", &family(texts, "bank", *kind));
                let key = if *looted {
                    "map.ask.bank-looted"
                } else {
                    "map.ask.bank"
                };
                alone(texts.text(key, &args))
            }
            Self::Search(class) => plain(match *class {
                DERELICT_SHIP => "map.ask.derelict-ship",
                CRYPT => "map.ask.crypt",
                _ => "map.ask.shipwreck",
            }),
            Self::Tree {
                gems,
                visited,
                poor,
            } => match (*visited, *poor, *gems) {
                (true, ..) => plain("map.ask.tree-visited"),
                (false, true, false) => plain("map.ask.tree-poor-gold"),
                (false, true, true) => plain("map.ask.tree-poor-gems"),
                (false, false, gems) => Told {
                    text: texts.text(
                        if gems {
                            "map.ask.tree-gems"
                        } else {
                            "map.ask.tree-gold"
                        },
                        &Args::new(),
                    ),
                    pictures: vec![EXPERIENCE],
                },
            },
            Self::Pyramid => plain("map.ask.pyramid"),
            Self::Tomb(_) => plain("map.ask.tomb"),
            Self::Guarded { dwelling, guards } => {
                let args = Args::new()
                    .text("dwelling", &family(texts, "dwelling1", *dwelling))
                    .text("size", &size(texts, guards.count))
                    .text("creature", &name(texts, guards.creature, 2));
                alone(texts.text("map.ask.guarded", &args))
            }
            Self::Hire {
                dwelling, joins, ..
            } => {
                let args = Args::new()
                    .text("dwelling", &family(texts, "dwelling1", *dwelling))
                    .text("creature", &name(texts, joins.creature, 2));
                alone(texts.text("map.ask.hire", &args))
            }
        }
    }

    /// What its OK shows where the viewer builds it: its boxes (style 1) and what the panels show
    /// at the last one's close (recorded, `ask-tomb` and `ask-hire` at 1660), the artifacts'
    /// skills `art`'s. The Warrior's Tomb's: ADVEVENT 162 with its artifact (its skills, the
    /// artifact taken as worn), else 163 with three pictures of bad morale; the tables' morale
    /// lost either way. A dwelling's creatures joining free (`visit_dwelling` 0x4A18B0's own
    /// path): GENRLTXT 424 with their count and name (one's the singular, *provisional*: not
    /// recorded) and the army joined, else 423 (none to hire) or 426 (no room). None for every
    /// other answer (*Not built* above).
    #[must_use]
    pub fn yes(&self, art: &FindArt, asks: &AskArt, texts: &Catalogue) -> Option<(Told, Gain)> {
        let morale = [asks.tomb_morale; 2];
        match self {
            Self::Tomb(Some(artifact)) => {
                let named = texts
                    .known(&format!("artifact.{artifact}"), &Args::new())
                    .unwrap_or_default();
                let args = Args::new().text("artifact", &named);
                Some((
                    Told {
                        text: texts.text("map.ask.tomb-found", &args),
                        pictures: vec![(ARTIFACT_PICTURE, i32::from(*artifact))],
                    },
                    Gain {
                        skills: art.skills_of(*artifact),
                        morale,
                        ..Gain::default()
                    },
                ))
            }
            Self::Tomb(None) => Some((
                Told {
                    text: texts.text("map.ask.tomb-empty", &Args::new()),
                    pictures: vec![BAD_MORALE; 3],
                },
                Gain {
                    morale,
                    ..Gain::default()
                },
            )),
            Self::Hire {
                joins,
                free: true,
                room,
                ..
            } => {
                let (key, count) = match (joins.count, room) {
                    (0, _) => ("map.ask.none", 2),
                    (_, false) => ("map.ask.no-room", 2),
                    (count, true) => ("map.ask.joined", count),
                };
                let args = Args::new()
                    .number("amount", i64::from(joins.count))
                    .text("creature", &name(texts, joins.creature, count));
                let mut gain = Gain::default();
                if key == "map.ask.joined" {
                    gain.army[0] = Some(*joins);
                }
                Some((
                    Told {
                        text: texts.text(key, &args),
                        pictures: Vec::new(),
                    },
                    gain,
                ))
            }
            _ => None,
        }
    }
}

/// The size word of `count` in form 2 (`creature_quantity_word` 0x44ADE0: `army_size_words`
/// 0x6A5BB8's lower-cased form, ARRAYTXT's records 176 + 3 n).
fn size(texts: &Catalogue, count: i32) -> String {
    let count = u32::try_from(count).unwrap_or(0);
    let n = SIZES.iter().take_while(|&&s| count >= s).count();
    texts.text(&format!("map.ask.size.{n}"), &Args::new())
}

/// Creature `creature`'s name for `count` (the singular for exactly one, else the plural).
fn name(texts: &Catalogue, creature: u16, count: i32) -> String {
    let args = Args::new().count(if count == 1 { 1 } else { 2 });
    texts
        .known(&format!("creature.{creature}"), &args)
        .unwrap_or_default()
}

/// The `family`'s entry `n`'s name (empty past the family).
fn family(texts: &Catalogue, family: &str, n: u32) -> String {
    texts
        .known(&format!("{family}.{n}"), &Args::new())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
