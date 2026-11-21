//! A record's visits told (E24 M4 T4b1, *provisional*; `docs/epics/E24-design.md` §5.14): what
//! one of the viewer's visits in a whole game's record shows, as Part A built it (T2: the finds,
//! the questions and choices), its facts the record's events and its answers the record's.
//!
//! - **A find told** ([`Told::Find`], [`crate::finds::Find`]): a Temple (96: the once-mark's
//!   morale gained, else nothing gained, visited), a resource pile (79: the resource gained), an
//!   artifact (5: the artifact added), a Learning Stone (100: the experience gained; its box's
//!   picture the record's amount), a Star Axis (61: its Spell Power gained) and a Windmill (112:
//!   the resource gained, else emptied); the object taken where an `ObjectRemoved` says so.
//! - **A question or a choice** ([`Told::Ask`], [`crate::asks::Ask`]) with the record's answer
//!   ([`Answered`]): the Arena (4: the skill raised by the answer's gain), Pandora's Box (6: the
//!   answer's gains as its rewards) and a treasure chest (101: the part chosen by the answer's
//!   gain; the other is not in the record, MCL-Q41: none).
//! - **What the panels show** ([`gain`]): the resources gained, the primary skills raised, a
//!   once-mark's morale and luck, the experience.
//! - **A flag raised** (E24 M4 T4b2, design §5.15): a mine (53: its resource its type's) and a
//!   Lighthouse (42) whose visit the record's `ObjectFlagged` makes the visitor's, a find told
//!   with the record's new owner; a dwelling (17, 20) flagged so, then its hire's question with
//!   the record's answer: accepted, its free creatures as the record tells them (joined, refused
//!   for want of room, none; another answer, or none told: nothing, its gap).
//!
//! A visit of another class, or whose events tell none of these, tells nothing here (its frames
//! differ from the original's: their gaps). A find's numbers the record gives replace the tables'
//! (Part A's rules for the facts at a sequence's 0); its texts and pictures are Part A's.

use alloc::string::String;

use h3_map::model::Rewards;
use h3_protocol::wire::game::{
    AdventureEvent, Answer, ArenaChoice, ChestChoice, Consent, GameEvent, HeroChange, NothingWhy,
};
use h3_protocol::wire::{Outcome, Side};

use super::played::Visit;
use crate::asks::Ask;
use crate::finds::{Find, Gain};
use crate::live::Stack;

/// A Temple's class.
const TEMPLE: u32 = 96;
/// A resource pile's.
const PILE: u32 = 79;
/// An artifact's.
const ARTIFACT: u32 = 5;
/// A Learning Stone's.
const STONE: u32 = 100;
/// A Star Axis's.
const AXIS: u32 = 61;
/// A Windmill's.
const WINDMILL: u32 = 112;
/// The Arena's.
const ARENA: u32 = 4;
/// Pandora's Box's.
const PANDORA: u32 = 6;
/// A treasure chest's.
const CHEST: u32 = 101;
/// A mine's.
const MINE: u32 = 53;
/// A wandering monster's (E24 M4 T5b2).
const MONSTER: u32 = 54;
/// A Lighthouse's.
const LIGHTHOUSE: u32 = 42;
/// A dwelling's.
const DWELLING: u32 = 17;
/// A dwelling's of several creatures.
const DWELLINGS: u32 = 20;
/// Gold's place among the resources.
const GOLD: usize = 6;

/// What a visit shows (the module's documentation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Told {
    /// A find told: its facts, what the panels show at its box's close and whether its object is
    /// taken off the map.
    Find {
        /// The find.
        find: Find,
        /// What the panels show.
        gain: Gain,
        /// The object taken.
        taken: bool,
        /// The colour its flag is raised in (E24 M4 T4b2: the record's `ObjectFlagged`).
        flag: Option<u8>,
    },
    /// A question or a choice and the answer the record gives it.
    Ask {
        /// The question.
        ask: Ask,
        /// The answer.
        answered: Answered,
        /// The colour the object's flag is raised in before it asks (E24 M4 T4b2: a dwelling's).
        flag: Option<u8>,
    },
    /// A wandering monster fought at once (E24 M4 T5b2, design §5.21): the record's battle, its
    /// winner, what the map shows after it and the experience gained.
    Fight {
        /// The record's number of the `GameEvents` whose `BattleStarted` began it.
        battle: u32,
        /// The winner: 0 the attacker, 1 the defender (an attacker's retreat or surrender), −1
        /// none.
        winner: i32,
        /// The monster taken off the map after it (`ObjectRemoved`).
        taken: bool,
        /// The hero gone after it (`HeroRemoved`).
        lost: bool,
        /// The experience gained.
        experience: i32,
    },
    /// A neutral mine's guards' question accepted and their battle (E24 M4 T5b2, design §5.21):
    /// the battle, its winner, the colour of the flag raised after it (the record's
    /// `ObjectFlagged`), the mine's resource (its subclass: an abandoned mine's 7, MCL-Q48: the
    /// record tells none) and the experience gained.
    Guarded {
        /// The record's number of the `GameEvents` whose `BattleStarted` began it.
        battle: u32,
        /// The winner, as [`Told::Fight`]'s.
        winner: i32,
        /// The flag's colour.
        flag: Option<u8>,
        /// The resource.
        resource: u8,
        /// The experience gained.
        experience: i32,
    },
}

/// Which of a question's answers the record's takes: its OK or a choice's first picture, a
/// choice's second picture, its Cancel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answered {
    /// The OK, or the first picture.
    Yes,
    /// The second picture.
    Second,
    /// The Cancel.
    No,
}

/// What `visit` of an object of `class` and `subclass` shows (the module's documentation), the
/// hero named `hero` in Pandora's Box's boxes; none for another class or events that tell none.
#[must_use]
pub fn told(visit: &Visit, [class, subclass]: [u32; 2], hero: &str) -> Option<Told> {
    let events = &visit.events;
    // A wandering monster's battle begun by the step (E24 M4 T5b2), once it ended.
    if class == MONSTER
        && let Some(battle) = &visit.battle
        && let Some(GameEvent::BattleEnded { result, .. }) = battle.ended.first()
    {
        let removed = |hero: bool| {
            battle.ended.iter().any(|e| match e {
                GameEvent::Adventure {
                    event: AdventureEvent::ObjectRemoved { .. },
                } => !hero,
                GameEvent::Adventure {
                    event: AdventureEvent::HeroRemoved { .. },
                } => hero,
                _ => false,
            })
        };
        return Some(Told::Fight {
            battle: battle.started,
            winner: winner(result),
            taken: removed(false),
            lost: removed(true),
            experience: gain(&battle.ended).experience,
        });
    }
    // A neutral mine's guards fought after their question (E24 M4 T5b2), once it ended.
    if class == MINE
        && let Some(battle) = &visit.battle
        && let Some(GameEvent::BattleEnded { result, .. }) = battle.ended.first()
        && let Some((
            Answer::Offer {
                consent: Consent::Accept,
            },
            _,
        )) = visit.answers.first()
    {
        let flag = battle.ended.iter().find_map(|e| match e {
            GameEvent::Adventure {
                event: AdventureEvent::ObjectFlagged { owner, .. },
            } => Some(*owner),
            _ => None,
        });
        return Some(Told::Guarded {
            battle: battle.started,
            winner: winner(result),
            flag,
            resource: u8::try_from(subclass).unwrap_or(u8::MAX),
            experience: gain(&battle.ended).experience,
        });
    }
    let flag = events.iter().find_map(|e| match e {
        GameEvent::Adventure {
            event: AdventureEvent::ObjectFlagged { owner, .. },
        } => Some(*owner),
        _ => None,
    });
    let gained = gain(events);
    let taken = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::Adventure {
                event: AdventureEvent::ObjectRemoved { .. }
            }
        )
    });
    let visited = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::Adventure {
                event: AdventureEvent::NothingGained {
                    why: NothingWhy::Visited,
                    ..
                }
            }
        )
    });
    let emptied = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::Adventure {
                event: AdventureEvent::ObjectEmpty { .. }
            }
        )
    });
    let find = |find: Find| {
        Some(Told::Find {
            find,
            gain: gained,
            taken,
            flag: None,
        })
    };
    match class {
        TEMPLE if visited => find(Find::Temple { visited: true }),
        TEMPLE if gained.morale != [0; 2] => find(Find::Temple { visited: false }),
        AXIS if visited => find(Find::Axis { visited: true }),
        AXIS if gained.skills != [0; 4] => find(Find::Axis { visited: false }),
        STONE if visited => find(Find::Stone { visited: true }),
        STONE if gained.experience > 0 => find(Find::Stone { visited: false }),
        PILE | WINDMILL => {
            let Some((resource, amount)) = resource(&gained) else {
                return (class == WINDMILL && emptied).then_some(Told::Find {
                    find: Find::Windmill {
                        resource: 0,
                        amount: 0,
                    },
                    gain: gained,
                    taken,
                    flag: None,
                });
            };
            find(if class == PILE {
                Find::Pile { resource, amount }
            } else {
                Find::Windmill { resource, amount }
            })
        }
        ARTIFACT => events.iter().find_map(|e| match e {
            GameEvent::HeroChanged {
                change: HeroChange::AddArtifact { artifact },
                ..
            } => find(Find::Artifact {
                artifact: artifact.artifact,
            }),
            _ => None,
        }),
        ARENA | PANDORA | CHEST => asked(visit, class, hero),
        MINE | LIGHTHOUSE => {
            let find = match class {
                MINE => Find::Mine {
                    resource: u8::try_from(subclass).ok().filter(|r| *r <= 6)?,
                },
                _ => Find::Lighthouse,
            };
            flag.map(|_| Told::Find {
                find,
                gain: gained,
                taken,
                flag,
            })
        }
        DWELLING | DWELLINGS => hired(visit, subclass, flag),
        _ => None,
    }
}

/// A dwelling's hire with the record's answer (E24 M4 T4b2, design §5.15), its flag raised first
/// in `flag`'s colour where the record says so: accepted, its free creatures as the record tells
/// them (`CreaturesJoined`, `JoinRefused` for want of room, `NoneToRecruit`); none for another
/// answer or none told (the hire dialog, M5's).
fn hired(visit: &Visit, subclass: u32, flag: Option<u8>) -> Option<Told> {
    let (
        Answer::Offer {
            consent: Consent::Accept,
        },
        after,
    ) = visit.answers.first()?
    else {
        return None;
    };
    let stack = |creature: u16, count: u32| Stack {
        creature,
        count: i32::try_from(count).unwrap_or(i32::MAX),
    };
    let (joins, room) = after.iter().find_map(|e| match e {
        GameEvent::Adventure { event } => match *event {
            AdventureEvent::CreaturesJoined {
                creature, count, ..
            } => Some((stack(creature, count), true)),
            AdventureEvent::JoinRefused {
                creature, count, ..
            } => Some((stack(creature, count), false)),
            AdventureEvent::NoneToRecruit { creature, .. } => Some((stack(creature, 0), true)),
            _ => None,
        },
        _ => None,
    })?;
    Some(Told::Ask {
        ask: Ask::Hire {
            dwelling: subclass,
            joins,
            free: true,
            room,
            flag: flag.is_some(),
        },
        answered: Answered::Yes,
        flag,
    })
}

/// The question of `visit` (Pandora's Box, the Arena, a treasure chest) with its first answer
/// (none without one).
fn asked(visit: &Visit, class: u32, hero: &str) -> Option<Told> {
    let (answer, after) = visit.answers.first()?;
    let gained = gain(after);
    let (ask, answered) = match (class, *answer) {
        (ARENA, Answer::Arena { choice }) => {
            let raised = gained.skills.iter().copied().max().unwrap_or(0);
            let answered = match choice {
                ArenaChoice::Attack => Answered::Yes,
                ArenaChoice::Defense => Answered::Second,
            };
            (Ask::Arena(raised), answered)
        }
        (PANDORA, Answer::Offer { consent }) => {
            let rewards = Rewards {
                experience: u32::try_from(gained.experience).unwrap_or(0),
                resources: gained.resources,
                primary: gained.skills.map(|s| u8::try_from(s).unwrap_or(0)),
                ..Rewards::default()
            };
            let answered = match consent {
                Consent::Accept => Answered::Yes,
                Consent::Decline => Answered::No,
            };
            let hero = String::from(hero);
            (Ask::OpenBox { rewards, hero }, answered)
        }
        (CHEST, Answer::Chest { choice }) => {
            // stub: MCL-Q41 (a choice's parts not in the record: the one not taken none)
            let (answered, gold, experience) = match choice {
                ChestChoice::Gold => (Answered::Yes, gained.resources[GOLD], 0),
                ChestChoice::Experience => (Answered::Second, 0, gained.experience),
            };
            (Ask::Chest { gold, experience }, answered)
        }
        _ => return None,
    };
    Some(Told::Ask {
        ask,
        answered,
        flag: None,
    })
}

/// The one resource `gained` holds, with its amount; none for none or several.
fn resource(gained: &Gain) -> Option<(i32, i32)> {
    let mut found = gained
        .resources
        .iter()
        .enumerate()
        .filter(|&(_, &amount)| amount != 0);
    let (resource, &amount) = found.next()?;
    if found.next().is_some() {
        return None;
    }
    Some((i32::try_from(resource).unwrap_or(0), amount))
}

/// A battle's winner as the timeline's `battle` line gives it (E24 M4 T5b2): 0 the attacker, 1
/// the defender (an attacker that retreated or surrendered lost), −1 a draw.
fn winner(result: &Outcome) -> i32 {
    match result {
        Outcome::Victory {
            side: Side::Attacker,
        }
        | Outcome::Retreat {
            side: Side::Defender,
        }
        | Outcome::Surrender {
            side: Side::Defender,
        } => 0,
        Outcome::Victory {
            side: Side::Defender,
        }
        | Outcome::Retreat {
            side: Side::Attacker,
        }
        | Outcome::Surrender {
            side: Side::Attacker,
        } => 1,
        _ => -1,
    }
}

/// What `events` give that the panels show: the resources gained (in the bar's order), the
/// primary skills raised, a once-mark's morale (on any day: the record's) and luck, the
/// experience.
#[must_use]
pub fn gain(events: &[GameEvent]) -> Gain {
    let mut gain = Gain::default();
    for event in events {
        match event {
            GameEvent::ResourceGained {
                resource, amount, ..
            } => {
                if let Some(r) = gain.resources.get_mut(usize::from(*resource)) {
                    *r = r.saturating_add(i32::try_from(*amount).unwrap_or(i32::MAX));
                }
            }
            GameEvent::HeroChanged { change, .. } => match change {
                HeroChange::GainPrimary { primary } => {
                    for (skill, raised) in gain.skills.iter_mut().zip(primary) {
                        *skill = skill.saturating_add(i8::try_from(*raised).unwrap_or(i8::MAX));
                    }
                }
                HeroChange::GainExperience { amount } => {
                    gain.experience = gain
                        .experience
                        .saturating_add(i32::try_from(*amount).unwrap_or(i32::MAX));
                }
                _ => {}
            },
            GameEvent::Adventure {
                event:
                    AdventureEvent::OnceMarkChanged {
                        bonus, held: true, ..
                    },
            } => {
                gain.morale = gain.morale.map(|m| m.saturating_add(bonus.morale));
                gain.luck = gain.luck.saturating_add(bonus.luck);
            }
            _ => {}
        }
    }
    gain
}
