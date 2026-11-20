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
//!
//! A visit of another class, or whose events tell none of these, tells nothing here (its frames
//! differ from the original's: their gaps). A find's numbers the record gives replace the tables'
//! (Part A's rules for the facts at a sequence's 0); its texts and pictures are Part A's.

use alloc::string::String;

use h3_map::model::Rewards;
use h3_protocol::wire::game::{
    AdventureEvent, Answer, ArenaChoice, ChestChoice, Consent, GameEvent, HeroChange, NothingWhy,
};

use super::played::Visit;
use crate::asks::Ask;
use crate::finds::{Find, Gain};

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
    },
    /// A question or a choice and the answer the record gives it.
    Ask {
        /// The question.
        ask: Ask,
        /// The answer.
        answered: Answered,
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
pub fn told(visit: &Visit, [class, _]: [u32; 2], hero: &str) -> Option<Told> {
    let events = &visit.events;
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
        _ => None,
    }
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
    Some(Told::Ask { ask, answered })
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
