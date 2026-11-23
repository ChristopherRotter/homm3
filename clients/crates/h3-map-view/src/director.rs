//! A record played without a recording (E24 M4 T4c, `docs/epics/E24-design.md` §5.17,
//! *provisional*): the viewer's commands given to the map as its player's input, each once the map
//! has waited for input a while, so that a whole game's record shows to its end as the original
//! would show that game to the record's player. The map's loop ([`Loop`]) is run millisecond by
//! millisecond as `timeline::play` runs it; the input it is given is kept, so that the window and
//! the headless play run the same loop on the same input.
//!
//! - **A walk** ([`Command::Walk`]): its hero chosen ([`Action::Choose`], its place in the
//!   kingdom's list as the map holds it, `Loop::slot_of`), then after a wait its path walked
//!   ([`Action::Walk`]).
//! - **End Turn**: its hotkey pressed and released.
//! - **A box** (a find, a question, End Turn's question, the week's message, a level-up): its
//!   answer when the record's next command is one (an offer's yes Enter, its no Escape, a level-up
//!   its offer's key, *provisional*), else Enter (OK). Since E24 M4 T6 (the reviewer's R2-2) only
//!   a box that takes an answer takes the record's (a question, a choice, a level-up, End Turn's
//!   question: `Loop::asks`; a find's box before its question, Pandora's Box's map text, takes
//!   Enter and leaves it), and a choice's answer is its picture clicked, then Enter (a chest's gold
//!   or experience, the Arena's Attack or Defense, a School's first or second skill, as a player's
//!   mouse would choose it: the click at the picture's middle, `Loop::choice_at`, the pointer back
//!   where it was in that millisecond, MCL-Q37), a School left Escape; the commands before the
//!   answer that the map does not show passed over (an army dialog's and its close, after a
//!   reward's creatures with no room: GAM-DEC-004, M5's).
//! - **A command of a screen the map has not** ([`Command::Other`]: a town's, a hero's, an
//!   army's; M5's): nothing shown, its events folded with the others'.
//!
//! - **A battle an answer began** ([`Command::Battle`], E24 M4 T5b1, design §5.20): handed over
//!   where the answer's box closes (its key's release), or where the answer was passed over with
//!   no box open ([`Directed::battles`]; a walk's battle is the map's own, at the walk's end:
//!   `Loop::battles`). Since E24 M4 T6 (the reviewer's R5-2) one the map's switch hands over
//!   (a mine's guards', T5b2) is the map's own too, listed once.
//! - **A battle's switch** (E24 M4 T5b2, design §5.21; since E24 M4 T6, the reviewer's R5-3):
//!   the map waits through it, and the battle is closed ([`Action::BattleClosed`]) in the
//!   millisecond its switch shows it whole (`Loop::battle_whole_at`): the battle takes none of
//!   the map's time, and that millisecond's change is its fade out's first (R5-7). A host shows
//!   the battle before that millisecond's frame.
//! - **A hero a battle took off the map** (E24 M4 T6, the reviewer's R5-6): each later hero's
//!   place in the kingdom's list one less from then on; since E24 M4 T6 (R1-4) the map's own
//!   list's, which each day's start takes from the record (a hero lost or hired).
//!
//! The record has played to its end once its last command has been given and the map waits again.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use h3_map::model::Pos;
use h3_protocol::wire::game::{
    Answer, ArenaChoice, ChestChoice, Consent, SchoolChoice, SkillOffer,
};
use h3_ui::message::Key;

use crate::engine::Command;
use crate::timeline::{Action, Hotkey, Loop, Rules, Start, Unplayable};

/// The waits of the record player (*provisional*: the client's).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pace {
    /// How long the map waits for input before a command (ms).
    pub wait: u32,
    /// How long a key is held (ms).
    pub hold: u32,
}

impl Default for Pace {
    fn default() -> Self {
        Self {
            wait: 400,
            hold: 60,
        }
    }
}

/// A record's commands as the map's input (the module's documentation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Directed {
    /// The input, by millisecond.
    pub actions: BTreeMap<u32, Vec<Action>>,
    /// The millisecond the map waits again after the last command: the record's end.
    pub end: u32,
    /// The battles the record's answers began (E24 M4 T5b1, [`Command::Battle`]) that the map
    /// does not hand over itself (since E24 M4 T6, R5-2: a switch's is `Loop::battles`'), in
    /// order: each at the millisecond the map hands it over (a plain cut), with the record's
    /// number of the `GameEvents` that began it.
    pub battles: Vec<(u32, u32)>,
}

/// The input that plays `commands` (the viewer's, in order) on the map from `start` under
/// `rules`, each hero's map hero by `heroes` (its record id, its index among the map's heroes:
/// `turn::record_heroes`), chosen by its place in the kingdom's list as the map holds it then
/// (`Loop::slot_of`, E24 M4 T6, R1-4), at `pace`; at most until `limit` ms.
///
/// # Errors
/// What the map cannot play, with its millisecond; a walk's hero `heroes` does not hold or the
/// map's list does not, or a record not at its end by `limit` ([`Unplayable::Record`]).
pub fn direct(
    rules: Rules,
    start: &Start,
    commands: &[Command],
    heroes: &[(u16, usize)],
    pace: Pace,
    limit: u32,
) -> Result<Directed, (u32, Unplayable)> {
    let mut map = Loop::start(rules, start).map_err(|why| (0, why))?;
    let mut actions: BTreeMap<u32, Vec<Action>> = BTreeMap::new();
    let mut battles = Vec::new();
    let mut next = 0;
    // The millisecond the map last started to wait, and the last one with input to come.
    let mut waiting: Option<u32> = None;
    let mut given_until = 0;
    for t in 1..=limit {
        let given = actions.get(&t).cloned().unwrap_or_default();
        map.run(t, &given).map_err(|why| (t, why))?;
        // A battle's switch: closed where it shows the battle whole (R5-3, R5-7).
        let at = t.saturating_add(1);
        if map.battle_whole_at() == Some(at) {
            actions.entry(at).or_default().push(Action::BattleClosed);
            given_until = given_until.max(at);
        }
        if t <= given_until || !map.idle() && !map.boxed() {
            waiting = None;
            continue;
        }
        let since = *waiting.get_or_insert(t);
        if t.saturating_sub(since) < pace.wait {
            continue;
        }
        waiting = None;
        let mut give = |at: u32, action: Action| {
            actions.entry(at).or_default().push(action);
            given_until = given_until.max(at);
        };
        let at = t.saturating_add(1);
        if map.boxed() {
            // A box that takes an answer takes the record's next one, the commands before it that
            // the map does not show passed over (R2-4: an army dialog's after a reward's
            // creatures with no room, its close; a screen's).
            let mut answer = None;
            if map.asks() {
                let shows =
                    |c: &&Command| !matches!(c, Command::Other | Command::Answer(Answer::Close {}));
                let k = (next..commands.len())
                    .find(|&k| commands.get(k).as_ref().is_some_and(shows))
                    .unwrap_or(commands.len());
                if let Some(&Command::Answer(given)) = commands.get(k) {
                    next = k + 1;
                    answer = Some(given);
                }
            }
            // A choice's picture (R2-2): clicked where the box shows it, the pointer back, then
            // the OK its choice enabled.
            let picked = answer
                .and_then(picture)
                .and_then(|n| map.choice_at(n))
                .map(|point| {
                    give(at, Action::Click(point));
                    give(at, Action::Pointer(start.pointer));
                });
            let at = if picked.is_some() {
                at.saturating_add(pace.hold)
            } else {
                at
            };
            match answer {
                Some(Answer::LevelUp { offer }) => {
                    give(at, Action::Offer(usize::from(offer == SkillOffer::Second)));
                    give(at.saturating_add(pace.hold), Action::Up);
                }
                answer => {
                    let key = match answer {
                        Some(
                            Answer::Offer {
                                consent: Consent::Decline,
                            }
                            | Answer::School {
                                choice: SchoolChoice::Leave,
                            },
                        ) => Key::Cancel,
                        _ => Key::Ok,
                    };
                    give(at, Action::BoxKey(key));
                    give(at.saturating_add(pace.hold), Action::BoxKeyUp(key));
                }
            }
            // The battle the answer began: where its box closes.
            if let Some(&Command::Battle(n)) = commands.get(next) {
                next += 1;
                battles.push((at.saturating_add(pace.hold), n));
            }
            continue;
        }
        let Some(&command) = commands.get(next) else {
            // An answer's battle the map's switch handed over is the map's own (R5-2).
            battles.retain(|&(_, n)| map.battles().iter().all(|&(_, m)| m != n));
            return match map.walks_left() {
                0 => Ok(Directed {
                    actions,
                    end: t,
                    battles,
                }),
                n => Err((t, Unplayable::Untaken(n))),
            };
        };
        next += 1;
        match command {
            Command::Walk { hero, to } => {
                let slot = heroes
                    .iter()
                    .find(|(id, _)| *id == hero)
                    .and_then(|&(_, index)| map.slot_of(index))
                    .ok_or((t, Unplayable::Record(at_of(to))))?;
                give(at, Action::Choose(slot));
                give(at.saturating_add(pace.wait), Action::Walk(to));
            }
            Command::EndTurn => {
                give(at, Action::Key(Hotkey::EndTurn));
                give(at.saturating_add(pace.hold), Action::KeyUp(Hotkey::EndTurn));
            }
            // An answer with no box open: nothing on the map, its battle handed over here.
            Command::Answer(_) => {
                if let Some(&Command::Battle(n)) = commands.get(next) {
                    next += 1;
                    battles.push((at, n));
                }
            }
            // A battle no answer began (*provisional*: none in a record), a screen's command:
            // nothing on the map.
            Command::Battle(_) | Command::Other => {}
        }
    }
    Err((limit, Unplayable::Record([-1, -1, -1])))
}

/// The picture a choice's `answer` clicks (E24 M4 T6, R2-2): a chest's gold the first, its
/// experience the second; the Arena's Attack and Defense; a School's first and second skill; none
/// for another answer (a School left: its Cancel).
const fn picture(answer: Answer) -> Option<usize> {
    match answer {
        Answer::Chest {
            choice: ChestChoice::Gold,
        }
        | Answer::Arena {
            choice: ArenaChoice::Attack,
        }
        | Answer::School {
            choice: SchoolChoice::First,
        } => Some(0),
        Answer::Chest {
            choice: ChestChoice::Experience,
        }
        | Answer::Arena {
            choice: ArenaChoice::Defense,
        }
        | Answer::School {
            choice: SchoolChoice::Second,
        } => Some(1),
        _ => None,
    }
}

/// A tile as the timeline's `[x, y, level]`.
fn at_of(at: Pos) -> [i32; 3] {
    [i32::from(at.x), i32::from(at.y), i32::from(at.z)]
}
