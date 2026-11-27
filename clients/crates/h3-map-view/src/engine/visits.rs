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
//!   answer's gains as its rewards), a treasure chest (101: the part chosen by the answer's
//!   gain; the other is not in the record, MCL-Q41: none) and since E24 M4 T6 (R2-2) a School of
//!   Magic (47: a skill bought, its gain and price the answer's; left, nothing, MCL-Q41).
//! - **What the panels show** ([`gain`]): the resources gained, the primary skills raised, a
//!   once-mark's morale and luck, the experience.
//! - **A flag raised** (E24 M4 T4b2, design §5.15): a mine (53: its resource its type's) and a
//!   Lighthouse (42) whose visit the record's `ObjectFlagged` makes the visitor's, a find told
//!   with the record's new owner; a dwelling (17, 20; since E24 M4 T6 17 alone) flagged so, then
//!   its hire's question with the record's answer: accepted, its free creatures as the record
//!   tells them (joined, refused for want of room, none; another answer, or none told: nothing,
//!   its gap).
//!
//! A visit of another class, or whose events tell none of these, tells nothing here (its frames
//! differ from the original's: their gaps). A find's numbers the record gives replace the tables'
//! (Part A's rules for the facts at a sequence's 0); its texts and pictures are Part A's.
//!
//! Since E24 M4 T6 (the reviewers' R2-1 to R2-8):
//!
//! - **Each visit laid out on its own** ([`lay`], [`Laid`]): every visit of the record to a tile
//!   its own find, played at its own walk's end, with its own box, gain, flag and level-ups (a
//!   Windmill taken, then found empty; Pandora's Box declined, then opened); a question's answers
//!   but the record's left out ([`Kept`]).
//! - **Pandora's Box's rewards** as the record tells them (R2-4, [`Told::Ask`]'s `OpenBox`): the
//!   experience, the primary and secondary skills, the spell points and the morale and luck
//!   changed (against [`Held`]), the resources gained and lost, the artifacts, the spells learned
//!   and the creatures joined; a Box whose guards the answer fights tells nothing (not built:
//!   ADVEVENT 16 and the battle). What the record does not tell (a part the hero does not get) is
//!   MCL-Q54's.
//! - **An artifact picked up** (R2-6): the primary skills of the artifact the record's
//!   `AddArtifact` adds (the tables' `artifact_skills`, as Part A's), on its window, taken as worn
//!   (*provisional*: where it lands the record does not tell, SRV-STUB-7).
//! - **A dwelling of class 20** (R2-5): its hire tells nothing (not built: ADVEVENT 36 and its
//!   names).
//! - **A flag in its owner's colour** (the reviewers' R3-1, R3-4): the record's `ObjectFlagged`
//!   names the owner by its player id, which `super::played` turns into its colour
//!   ([`Visit::flag`]); a visit that raises a flag and tells nothing else Part A builds (a
//!   dwelling's hire not built, a Shipyard, an abandoned mine, a guarded dwelling) raises it
//!   alone ([`Told::Flagged`]), so that every flag the viewer's visits raise is drawn.
//!
//! Since E24 M5 T4c (design §5.28, *provisional*; `MCL-STUB-7`): live, a decision the view does
//! not ask is its notice ([`Ask::Notice`], [`crate::notice`]), its every answer its OK's; a visit
//! is placed where the engine's walk ended ([`Placed`]: from beside where it stops one short, a
//! monster's zone entered on the monster, a meeting on no object); a town entered shows the town
//! screen's notice ([`Told::Notice`]).

use alloc::string::String;

use alloc::vec::Vec;

use h3_map::model::{Pos, Rewards};
use h3_protocol::wire::game::{
    AdventureEvent, Answer, ArenaChoice, ArmyRef, ArmySlot, ChestChoice, ChoiceKind, Consent,
    Decision, GameEvent, HeroChange, NothingWhy, OfferKind, SchoolChoice,
};
use h3_protocol::wire::{Outcome, Side};

use super::played::{Held, Visit};
use crate::asks::{Ask, Offer as AskOffer, Reveal};
use crate::finds::{Find, FindArt, Gain};
use crate::live::Stack;
use crate::notice::Notice;

/// A Temple's class.
const TEMPLE: u32 = 96;
/// A resource pile's.
const PILE: u32 = 79;
/// An artifact's.
const ARTIFACT: u32 = 5;
/// The random artifacts' first and last (any, treasure, minor, major, relic): a new game made each
/// an artifact, whose picture the view does not name (`MCL-STUB-1`) but whose pickup the record
/// tells by its `AddArtifact` (E24 M5 T4c, T4's game: taken with no box before).
const RANDOM_ARTIFACT: u32 = 65;
const RANDOM_RELIC: u32 = 69;
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
/// A School of Magic's (E24 M4 T6, R2-2).
const SCHOOL: u32 = 47;
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
/// An event's (none drawn: its removal shows nothing).
const EVENT: u32 = 26;
/// Gold's place among the resources.
const GOLD: usize = 6;
/// Gold's handle (as a price's `[resource, amount]`).
const GOLD_HANDLE: i32 = 6;
/// Gems' handle.
const GEMS: u16 = 5;
/// The Tree of Knowledge's class.
const TREE: u32 = 102;
/// The Warrior's Tomb's.
const TOMB: u32 = 108;
/// The Pyramid's.
const PYRAMID: u32 = 63;
/// A Border Guard's.
const BORDER_GUARD: u32 = 9;
/// A Cartographer's.
const CARTOGRAPHER: u32 = 13;
/// A monster's count in the view's word (its first of the count's word).
const COUNT: u32 = 0xFFF;
/// A town's (as the game loads it).
const TOWN: u32 = 98;
/// The Redwood Observatory's (E24 M5 T4c).
const OBSERVATORY: u32 = 58;
/// The Pillar of Fire's.
const PILLAR: u32 = 60;

/// The object of a live visit's notice on a tile with no object's entrance ([`lay`]: a meeting's,
/// the visited hero's tile; E24 M5 T4c): none of the map's.
pub const NO_OBJECT: usize = usize::MAX;

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
    /// record tells none) and the experience gained; since E24 M4 T6 (the reviewers' R5-4,
    /// R3-2) whether the hero is gone after it.
    Guarded {
        /// The record's number of the `GameEvents` whose `BattleStarted` began it.
        battle: u32,
        /// The winner, as [`Told::Fight`]'s.
        winner: i32,
        /// The hero gone after it (`HeroRemoved`: beaten, retreated or a draw).
        lost: bool,
        /// The flag's colour.
        flag: Option<u8>,
        /// The resource.
        resource: u8,
        /// The experience gained.
        experience: i32,
    },
    /// A flag raised and nothing else built (E24 M4 T6, the reviewers' R3-4, *provisional*): a
    /// dwelling whose hire is declined, not free or of class 20, a Shipyard, an abandoned mine, a
    /// guarded dwelling's battle won; the object the owner's from the visit's end on, no box.
    Flagged {
        /// The flag's colour.
        flag: u8,
    },
    /// An object the visit takes off the map and nothing else built (E24 M4 T6, *provisional*):
    /// one visited from beside whose visit Part A does not build (a Scholar, a Spell Scroll),
    /// taken as a pickup is, no box.
    Taken,
    /// Live, a visit that tells nothing built but opens level-ups (E24 M5 T4b, design §5.27: the
    /// Tree of Knowledge's free level): no box, what the panels show, then its level-ups' dialogs
    /// (the engine holds them open).
    Levels {
        /// What the panels show.
        gain: Gain,
    },
    /// Live, a screen the visit opens with no decision (E24 M5 T4c, design §5.28, `MCL-STUB-7`: a
    /// hero's entry into a town, the town screen of M7): its notice, a box of its own whose OK
    /// closes it.
    Notice(Notice),
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
    /// None yet: live, the host holds the engine's decision open (E24 M5 T4b, design §5.27),
    /// every answer the host's ([`opened`]).
    Open,
}

/// What `visit` of an object of `class` and `subclass` shows (the module's documentation), the
/// hero named `hero` in Pandora's Box's boxes; none for another class or events that tell none.
/// Since E24 M4 T6 (the reviewers' R3-1, R3-4) a visit whose flag the record raises shows at
/// least that ([`Told::Flagged`]), in its owner's colour ([`Visit::flag`]).
///
/// Since E24 M4 T6 (*provisional*) a visit that takes an object off the map (an event's aside) and
/// tells nothing else takes it alone ([`Told::Taken`]), so that the map does not keep it.
#[must_use]
pub fn told(visit: &Visit, class: [u32; 2], hero: &str) -> Option<Told> {
    shown(visit, class, hero)
        .or_else(|| visit.flag.map(|flag| Told::Flagged { flag }))
        .or_else(|| (class[0] != EVENT && removes(&visit.events)).then_some(Told::Taken))
}

/// The question a live visit of an object of `class` and subclass asks while the host holds
/// `decision` open (E24 M5 T4b, design §5.27, *provisional*): its facts the engine's (the viewer's
/// open decision as the engine projects it; a monster's creature its subclass and its count the
/// first of its word as the view shows it, `word`), its hero named `hero` (Pandora's Box's
/// boxes); its answers the host's. A treasure chest's gold and experience, the Arena's gain, a
/// School's gain and price, Pandora's Box (its rewards told by the answer), the Warrior's Tomb
/// (its artifact told by the answer), the Tree of Knowledge's price in gold or gems, a
/// Cartographer's map, a Border Guard's passage, the Pyramid, a wandering monster's offer to join
/// free or for its price and its flight (each next offer the engine's next decision: no flight
/// of the client's); none for another decision or class. Since E24 M5 T4c (design §5.28) a
/// decision the view does not ask its notice ([`Ask::Notice`], [`Notice::of`]: a guarded object's
/// fight and a dwelling's guards, M5 T6's battles; a dwelling's hire, a Shipyard's boat and every
/// window of M6 and M7 but a level-up's), whatever the object.
#[must_use]
pub fn opened(
    decision: &Decision,
    [class, subclass]: [u32; 2],
    word: Option<u32>,
    hero: &str,
) -> Option<Ask> {
    let clamp = |n: u32| i32::try_from(n).unwrap_or(i32::MAX);
    let monster = |offer| Ask::Monster {
        stack: Stack {
            creature: u16::try_from(subclass).unwrap_or(u16::MAX),
            count: word.map_or(0, |w| clamp(w & COUNT)),
        },
        offer,
        room: true,
        flees: false,
    };
    let asked = match decision {
        Decision::Choice { choice } => match (&choice.kind, class) {
            (ChoiceKind::Chest { gold, experience }, CHEST) => Some(Ask::Chest {
                gold: clamp(gold.amount),
                experience: clamp(*experience),
            }),
            (ChoiceKind::Primary { amount, .. }, ARENA) => {
                Some(Ask::Arena(i8::try_from(*amount).unwrap_or(i8::MAX)))
            }
            (ChoiceKind::Primary { amount, price, .. }, SCHOOL) => {
                let price = price.first().map_or([GOLD_HANDLE, 0], |p| {
                    [i32::from(p.resource), clamp(p.amount)]
                });
                Some(Ask::School {
                    gain: i8::try_from(*amount).unwrap_or(i8::MAX),
                    price,
                })
            }
            _ => None,
        },
        Decision::Offer { offer } => match (&offer.kind, class) {
            (OfferKind::OpenBox {}, PANDORA) => Some(Ask::OpenBox {
                rewards: Rewards::default(),
                hero: String::from(hero),
            }),
            (OfferKind::Search {}, TOMB) => Some(Ask::Tomb {
                artifact: None,
                morale: 0,
            }),
            (OfferKind::Gift { price }, TREE) => Some(Ask::Tree {
                gems: price.iter().any(|p| p.resource == GEMS),
                visited: false,
                poor: false,
            }),
            (OfferKind::Reveal { price }, CARTOGRAPHER) => Some(Ask::Maps {
                kind: subclass,
                price: price.first().map_or([GOLD_HANDLE, 0], |p| {
                    [i32::from(p.resource), clamp(p.amount)]
                }),
                reveal: Reveal::None,
            }),
            (OfferKind::Pass {}, BORDER_GUARD) => Some(Ask::Pass),
            (OfferKind::SearchPyramid {}, PYRAMID) => Some(Ask::Pyramid),
            (OfferKind::Join {}, MONSTER) => Some(monster(AskOffer::Join)),
            (OfferKind::JoinForGold { price }, MONSTER) => {
                let gold = price.iter().map(|p| clamp(p.amount)).sum();
                Some(monster(AskOffer::Gold(gold)))
            }
            (OfferKind::Pursue {}, MONSTER) => Some(monster(AskOffer::Flee)),
            _ => None,
        },
        _ => None,
    };
    asked.or_else(|| Notice::of(decision).map(Ask::Notice))
}

/// What a live visit tells (E24 M5 T4b, design §5.27): the question of the decision the host held
/// open there ([`opened`], `class` its object's class and subclass and its word), unanswered
/// every answer the host's ([`Answered::Open`]); answered, the record's answer, the question's
/// facts brought up to what it told (Pandora's Box's rewards, the Tomb's artifact, a monster's
/// creatures joined). None where the decision asks no question here. A notice's every answer its
/// OK's (E24 M5 T4c: the host's closing answer).
#[must_use]
pub fn live(
    visit: &Visit,
    decision: &Decision,
    (class, word): ([u32; 2], Option<u32>),
    hero: &str,
) -> Option<Told> {
    let ask = opened(decision, class, word, hero)?;
    let flag = visit.flag;
    let Some((answer, after)) = visit.answers.first() else {
        return Some(Told::Ask {
            ask,
            answered: Answered::Open,
            flag,
        });
    };
    let answered = match *answer {
        _ if matches!(ask, Ask::Notice(_)) => Answered::Yes,
        Answer::Offer {
            consent: Consent::Accept,
        }
        | Answer::Chest {
            choice: ChestChoice::Gold,
        }
        | Answer::Arena {
            choice: ArenaChoice::Attack,
        }
        | Answer::School {
            choice: SchoolChoice::First,
        } => Answered::Yes,
        Answer::Chest {
            choice: ChestChoice::Experience,
        }
        | Answer::Arena {
            choice: ArenaChoice::Defense,
        }
        | Answer::School {
            choice: SchoolChoice::Second,
        } => Answered::Second,
        _ => Answered::No,
    };
    let ask = match ask {
        Ask::OpenBox { hero, .. } => Ask::OpenBox {
            rewards: rewards(visit),
            hero,
        },
        // The Tomb's artifact and its morale lost (`OnceMarkChanged`), the record's.
        Ask::Tomb { .. } => Ask::Tomb {
            artifact: after.iter().find_map(|e| match e {
                GameEvent::HeroChanged {
                    change: HeroChange::AddArtifact { artifact },
                    ..
                } => Some(artifact.artifact),
                _ => None,
            }),
            morale: gain(after).morale[0],
        },
        // The tiles the answer revealed (its `GameReveal`), the record's.
        Ask::Maps { kind, price, .. } => Ask::Maps {
            kind,
            price,
            reveal: Reveal::Tiles(visit.revealed.first().cloned().unwrap_or_default()),
        },
        Ask::Monster {
            stack, offer, room, ..
        } => {
            let joined = after.iter().find_map(|e| match e {
                GameEvent::Adventure {
                    event:
                        AdventureEvent::CreaturesJoined {
                            creature, count, ..
                        },
                } => Some(Stack {
                    creature: *creature,
                    count: i32::try_from(*count).unwrap_or(i32::MAX),
                }),
                _ => None,
            });
            Ask::Monster {
                stack: joined.unwrap_or(stack),
                offer,
                room: room && joined.is_some(),
                flees: false,
            }
        }
        ask => ask,
    };
    Some(Told::Ask {
        ask,
        answered,
        flag,
    })
}

/// What the answers of a record's visit give, by what the panels show (E24 M5 T4b: a live
/// answer's reply the client does not build, the Tree of Knowledge's experience and price).
#[must_use]
pub fn answered_gain(visit: &Visit) -> Gain {
    let after: Vec<GameEvent> = visit
        .answers
        .iter()
        .flat_map(|(_, events)| events.iter().cloned())
        .collect();
    gain(&after)
}

/// Whether `events` take an object off the map (`ObjectRemoved`; since E24 M5 T4b a live answer's
/// object taken, `h3-map-client`'s).
#[must_use]
pub fn removes(events: &[GameEvent]) -> bool {
    removed(events).is_some()
}

/// The anchor of the first object `events` take off the map (`ObjectRemoved`; E24 M5 T4c, design
/// §5.29: a live answer's object taken, the record's), none without one or off the map's tiles.
#[must_use]
pub fn removed(events: &[GameEvent]) -> Option<Pos> {
    events.iter().find_map(|e| match e {
        GameEvent::Adventure {
            event: AdventureEvent::ObjectRemoved { anchor, .. },
        } => Some(Pos {
            x: u8::try_from(anchor.x).ok()?,
            y: u8::try_from(anchor.y).ok()?,
            z: anchor.level,
        }),
        _ => None,
    })
}

/// What [`told`] builds of `visit` beyond its flag alone.
fn shown(visit: &Visit, [class, subclass]: [u32; 2], hero: &str) -> Option<Told> {
    let events = &visit.events;
    // What a battle's end took off the map: the object (`ObjectRemoved`) or the hero
    // (`HeroRemoved`).
    let removed = |ended: &[GameEvent], hero: bool| {
        ended.iter().any(|e| match e {
            GameEvent::Adventure {
                event: AdventureEvent::ObjectRemoved { .. },
            } => !hero,
            GameEvent::Adventure {
                event: AdventureEvent::HeroRemoved { .. },
            } => hero,
            _ => false,
        })
    };
    // A wandering monster's battle begun by the step (E24 M4 T5b2), once it ended.
    if class == MONSTER
        && let Some(battle) = &visit.battle
        && let Some(GameEvent::BattleEnded { result, .. }) = battle.ended.first()
    {
        let removed = |hero: bool| removed(&battle.ended, hero);
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
        return Some(Told::Guarded {
            battle: battle.started,
            winner: winner(result),
            lost: removed(&battle.ended, true),
            flag: visit.flag,
            resource: u8::try_from(subclass).unwrap_or(u8::MAX),
            experience: gain(&battle.ended).experience,
        });
    }
    let flag = visit.flag;
    let gained = gain(events);
    let taken = removes(events);
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
    // An observatory's reveal told (E24 M5 T4c: `TilesRevealed`, at every visit).
    let revealed = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::Adventure {
                event: AdventureEvent::TilesRevealed { .. }
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
        OBSERVATORY | PILLAR if revealed => find(Find::Observatory {
            fire: class == PILLAR,
        }),
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
        ARTIFACT | RANDOM_ARTIFACT..=RANDOM_RELIC => events.iter().find_map(|e| match e {
            GameEvent::HeroChanged {
                change: HeroChange::AddArtifact { artifact },
                ..
            } => find(Find::Artifact {
                artifact: artifact.artifact,
            }),
            _ => None,
        }),
        ARENA | PANDORA | CHEST | SCHOOL => asked(visit, class, hero),
        MINE | LIGHTHOUSE => {
            let find = match class {
                // stub: MCL-Q55 (a record's mine's income: the tables', SRV-STUB-10)
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
        DWELLING => hired(visit, subclass, flag),
        // A class-20 dwelling's hire (E24 M4 T6, R2-5): not built (ADVEVENT 36 and its names,
        // `crate::asks`' *Not built*), told as nothing.
        DWELLINGS => None,
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
            // Its guards fought (not built: ADVEVENT 16, the battle, then its rewards).
            if visit.battle.is_some() {
                return None;
            }
            let rewards = rewards(visit);
            let answered = match consent {
                Consent::Accept => Answered::Yes,
                Consent::Decline => Answered::No,
            };
            let hero = String::from(hero);
            (Ask::OpenBox { rewards, hero }, answered)
        }
        (SCHOOL, Answer::School { choice }) => {
            // A skill bought (E24 M4 T6, R2-2): its gain and price the record's; left, neither
            // is told.
            // stub: MCL-Q41 (a choice's parts not in the record: a School left tells nothing)
            let answered = match choice {
                SchoolChoice::First => Answered::Yes,
                SchoolChoice::Second => Answered::Second,
                SchoolChoice::Leave => return None,
            };
            let price = after.iter().find_map(|e| match *e {
                GameEvent::ResourcePaid {
                    resource, amount, ..
                } => Some([
                    i32::from(resource),
                    i32::try_from(amount).unwrap_or(i32::MAX),
                ]),
                _ => None,
            })?;
            let gain = gained.skills.iter().copied().max().unwrap_or(0);
            (Ask::School { gain, price }, answered)
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

/// Pandora's Box's rewards as the record tells them from its answers' events (E24 M4 T6, the
/// reviewer's R2-4; `grant_rewards` 0x49FA90's parts, Part A's [`crate::finds::rewards`] draws
/// them): the experience (given after the army dialog of the creatures with no room, when there
/// is one), the primary skills, each secondary skill at the level it reaches (`RaiseSkill`), the
/// spell points and the morale and luck no once-mark names changed (a record tells them as they
/// are after: against what the hero held as its walk began, brought on by the command's events,
/// [`Held`]), the resources gained and lost, the artifacts added, the spells learned and the
/// creatures joined, then those with no room (the army dialog's group, `ArmyDialogOpened`).
// stub: MCL-Q54 (what the record does not tell: the parts the hero does not get)
fn rewards(visit: &Visit) -> Rewards {
    let after: Vec<GameEvent> = visit
        .answers
        .iter()
        .flat_map(|(_, events)| events.iter().cloned())
        .collect();
    let gained = gain(&after);
    let before = held(visit.held, &visit.events);
    let now = held(before, &after);
    let mut rewards = Rewards {
        experience: u32::try_from(gained.experience).unwrap_or(0),
        mana: i32::from(now.mana) - i32::from(before.mana),
        morale: now.morale.saturating_sub(before.morale),
        luck: now.luck.saturating_sub(before.luck),
        resources: gained.resources,
        primary: gained.skills.map(|s| u8::try_from(s).unwrap_or(0)),
        ..Rewards::default()
    };
    let mut group = Vec::new();
    for event in &after {
        match event {
            GameEvent::ArmyDialogOpened { stack, .. } => group.push(h3_map::model::Stack {
                creature: stack.creature,
                count: u16::try_from(stack.count).unwrap_or(u16::MAX),
            }),
            GameEvent::HeroChanged { change, .. } => match *change {
                HeroChange::RaiseSkill { skill, level } => {
                    if let Ok(skill) = u8::try_from(skill) {
                        rewards.skills.push((skill, level));
                    }
                }
                HeroChange::AddArtifact { artifact } => rewards.artifacts.push(artifact.artifact),
                HeroChange::LearnSpell { spell } => {
                    if let Ok(spell) = u8::try_from(spell) {
                        rewards.spells.push(spell);
                    }
                }
                _ => {}
            },
            GameEvent::Adventure {
                event:
                    AdventureEvent::CreaturesJoined {
                        creature, count, ..
                    },
            } => rewards.creatures.push(h3_map::model::Stack {
                creature: *creature,
                count: u16::try_from(*count).unwrap_or(u16::MAX),
            }),
            _ => {}
        }
    }
    rewards.creatures.extend(group);
    rewards
}

/// What a hero holding `held` holds after `events` (E24 M4 T6, R2-4): its spell points as the
/// last `SetSpellPoints` sets them, its morale and luck as the last `BonusChanged` tells them.
fn held(mut held: Held, events: &[GameEvent]) -> Held {
    for event in events {
        match event {
            GameEvent::HeroChanged {
                change: HeroChange::SetSpellPoints { points },
                ..
            } => held.mana = *points,
            GameEvent::Adventure {
                event: AdventureEvent::BonusChanged { morale, luck, .. },
            } => {
                held.morale = *morale;
                held.luck = *luck;
            }
            _ => {}
        }
    }
    held
}

/// One of a record's visits as the timeline's find lays it out (E24 M4 T6, the reviewers' R2-1 and
/// R2-8; the module's documentation): its tile, its object, its walk, what it shows, the answers
/// its question keeps, whether its object blocks its tile and its level-ups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Laid {
    /// The walk's last tile (the visited object's entrance, or the tile it is visited from beside
    /// on).
    pub at: Pos,
    /// The object (its index in the map's order).
    pub object: usize,
    /// The walk's place among the record's walks ([`Visit::walk`]): the visit is that walk's
    /// end's.
    pub walk: usize,
    /// What it shows.
    pub told: Told,
    /// The answers its question keeps.
    pub kept: Kept,
    /// Its object blocks its tile: a key's step or a path's last node visits it from beside (a
    /// monster fought, a find taken, a question of [`Ask::beside`]).
    pub blocks: bool,
    /// Its level-ups' places among the record's ([`Visit::levels`]).
    pub levels: Vec<usize>,
}

/// The answers a record's question keeps (E24 M4 T6, R2-8): the record's alone (another is
/// refused in the timeline, `Unplayable::Answer`), the Arena's Cancel with its Attack (MCL-Q33),
/// a guarded mine's OK (its battle).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Kept {
    /// Its OK, or a choice's first picture.
    pub yes: bool,
    /// A choice's second picture.
    pub second: bool,
    /// Its Cancel.
    pub no: bool,
}

impl Kept {
    /// What `told`'s question keeps (none for a find or a fight).
    #[must_use]
    pub const fn of(told: &Told) -> Self {
        let none = Self {
            yes: false,
            second: false,
            no: false,
        };
        match told {
            Told::Ask {
                ask,
                answered: Answered::Yes,
                ..
            } => Self {
                yes: true,
                no: matches!(ask, Ask::Arena(_)),
                ..none
            },
            Told::Ask {
                answered: Answered::Second,
                ..
            } => Self {
                second: true,
                ..none
            },
            Told::Ask {
                answered: Answered::No,
                ..
            } => Self { no: true, ..none },
            Told::Ask {
                answered: Answered::Open,
                ..
            } => none,
            Told::Guarded { .. } => Self { yes: true, ..none },
            Told::Find { .. }
            | Told::Fight { .. }
            | Told::Flagged { .. }
            | Told::Taken
            | Told::Levels { .. }
            | Told::Notice(_) => none,
        }
    }
}

/// `visits` laid out (E24 M4 T6, the reviewers' R2-1, R2-6 and R2-8): each on the object whose
/// entrance `object_at` finds on its tile (its index and its class and subclass; since E24 M5 T4c
/// `object_at` is given the visit), told
/// ([`told`]) with its hero named by `name` (by its type), in the record's order, one a visit; an
/// artifact's pickup with the artifact's primary skills (`art`'s `artifact_skills`, taken as
/// worn: SRV-STUB-7); a visit on no object, or that tells nothing, left out. Since E24 M5 T4b
/// (live, design §5.27: `open` given, none for a record) a visit whose walk `open` names asks the
/// question of the decision the host held open there ([`live`]): with no answer yet every answer
/// the host's ([`Answered::Open`]), else the record's; one that tells nothing else but opens
/// level-ups shows them ([`Told::Levels`]); `object_at` gives its object's word too (the view's:
/// a monster's first count of its word). Since E24 M5 T4c (design §5.28) `object_at` places the
/// visit ([`Placed`]): the tile it is told on and whether the engine's walk stops one short of it
/// too; live, a question's visit is from beside where it does (the engine's walk), a meeting's
/// always, its tile the visited hero's (no object's entrance there: its object [`NO_OBJECT`]); a
/// decision the view does not ask is its notice ([`Ask::Notice`]); a live visit to a town that
/// tells nothing else shows the town screen's notice ([`Told::Notice`]).
#[must_use]
pub fn lay(
    visits: &[Visit],
    open: Option<&[(usize, Decision)]>,
    object_at: impl Fn(&Visit) -> Option<Placed>,
    art: &FindArt,
    name: impl Fn(Option<u8>) -> String,
) -> Vec<Laid> {
    visits
        .iter()
        .filter_map(|visit| laid(visit, open, object_at(visit), art, &name(visit.hero_type)))
        .collect()
}

/// Where a visit is told and on what object ([`lay`]'s `object_at`, E24 M5 T4c,
/// *provisional*): a record's on the object of its walk's last tile; live, a decision the engine
/// opened on the way (a monster's zone entered: its offer) where the engine's walk ended, on the
/// decision's object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placed {
    /// The tile it is told on: the walk's last (the object's entrance, or the tile it is visited
    /// from beside on), or where the engine's walk ended.
    pub at: Pos,
    /// The object (its index in the map's order).
    pub object: usize,
    /// Its class and subclass.
    pub class: [u32; 2],
    /// Its word (the view's: a monster's first count of its word).
    pub word: Option<u32>,
    /// The engine's walk stops one short of the tile: its object visited from beside.
    pub beside: bool,
}

/// `visit` laid out as [`lay`] lays it ([`lay`]'s body, not generic: E24 M5 T4c, each
/// instance of a generic function's closures counted on its own): as `found` places it, its hero
/// named `hero`.
fn laid(
    visit: &Visit,
    open: Option<&[(usize, Decision)]>,
    found: Option<Placed>,
    art: &FindArt,
    hero: &str,
) -> Option<Laid> {
    let Placed {
        at,
        object,
        class,
        word,
        beside,
    } = match found {
        Some(found) => found,
        // Live, a notice on no object's entrance (a meeting: the visited hero's tile).
        None if notice_of(visit, open).is_some() => Placed {
            at: visit.at,
            object: NO_OBJECT,
            class: [0, 0],
            word: None,
            beside: false,
        },
        None => return None,
    };
    let mut told = laid_live(visit, open, (class, word), hero)?;
    if let Told::Find {
        find: Find::Artifact { artifact },
        gain,
        ..
    } = &mut told
    {
        for (skill, more) in gain.skills.iter_mut().zip(art.skills_of(*artifact)) {
            *skill = skill.saturating_add(more);
        }
    }
    let blocks = match &told {
        Told::Fight { .. } | Told::Taken => true,
        Told::Guarded { .. } | Told::Flagged { .. } | Told::Levels { .. } | Told::Notice(_) => {
            false
        }
        Told::Find { taken, .. } => *taken,
        Told::Ask {
            ask: Ask::Notice(Notice::Meeting),
            ..
        } => true,
        // Live, from beside where the engine's walk stops one short of the tile (E24 M5 T4c).
        Told::Ask { .. } if open.is_some() => beside,
        Told::Ask { ask, .. } => ask.beside(),
    };
    Some(Laid {
        at,
        object,
        walk: visit.walk,
        kept: Kept::of(&told),
        told,
        blocks,
        levels: visit.levels.clone(),
    })
}

/// What `visit` tells as [`lay`] lays it out (its object of `class` with its `word`, its hero
/// named `hero`): live (`open` given), the question of the decision the host held open at its walk
/// ([`live`]), else what it tells ([`told`]), else its level-ups alone ([`Told::Levels`]), else at
/// a town the town screen's notice (E24 M5 T4c: [`Told::Notice`]); a record's, what it tells.
fn laid_live(
    visit: &Visit,
    open: Option<&[(usize, Decision)]>,
    (class, word): ([u32; 2], Option<u32>),
    hero: &str,
) -> Option<Told> {
    let Some(open) = open else {
        return told(visit, class, hero);
    };
    let asked = open
        .iter()
        .find(|(walk, _)| *walk == visit.walk)
        .and_then(|(_, decision)| live(visit, decision, (class, word), hero));
    asked
        .or_else(|| told(visit, class, hero))
        .or_else(|| {
            (!visit.levels.is_empty()).then(|| Told::Levels {
                gain: gain(&visit.events),
            })
        })
        .or_else(|| (class[0] == TOWN).then_some(Told::Notice(Notice::Town)))
}

/// The notice of the decision the host held open at `visit`'s walk (live, `open` given; E24 M5
/// T4c), none without one.
fn notice_of(visit: &Visit, open: Option<&[(usize, Decision)]>) -> Option<Notice> {
    open?
        .iter()
        .find(|(walk, _)| *walk == visit.walk)
        .and_then(|(_, decision)| Notice::of(decision))
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
/// experience. Since E24 M5 T4c (design §5.29, the facts audit) the resources paid and lost
/// taken off (a Tree of Knowledge's price, a Cartographer's, a monster's join for gold) and the
/// hero's army's slots as the record tells them ([`Gain::slotted`]: `StackChanged`).
#[must_use]
pub fn gain(events: &[GameEvent]) -> Gain {
    let mut gain = Gain::default();
    // A resource's amount added (a gain) or taken off (a price paid, a loss).
    let changed = |resources: &mut [i32; 7], resource: u16, amount: u32, more: bool| {
        if let Some(r) = resources.get_mut(usize::from(resource)) {
            let amount = i32::try_from(amount).unwrap_or(i32::MAX);
            *r = if more {
                r.saturating_add(amount)
            } else {
                r.saturating_sub(amount)
            };
        }
    };
    for event in events {
        match event {
            GameEvent::ResourceGained {
                resource, amount, ..
            } => changed(&mut gain.resources, *resource, *amount, true),
            GameEvent::ResourcePaid {
                resource, amount, ..
            }
            | GameEvent::ResourceLost {
                resource, amount, ..
            } => changed(&mut gain.resources, *resource, *amount, false),
            GameEvent::StackChanged {
                at:
                    ArmySlot {
                        army: ArmyRef::Hero { .. },
                        slot,
                    },
                stack: Some(stack),
                ..
            } => {
                if let Some(held) = gain.army.get_mut(usize::from(*slot)) {
                    *held = Some(Stack {
                        creature: stack.creature,
                        count: i32::try_from(stack.count).unwrap_or(i32::MAX),
                    });
                    gain.slotted = true;
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
