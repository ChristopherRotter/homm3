//! A record played without a recording (E24 M4 T4c, `docs/epics/E24-design.md` §5.17,
//! *provisional*): the viewer's commands given to the map as its player's input, each once the map
//! has waited for input a while, so that a whole game's record shows to its end as the original
//! would show that game to the record's player. The map's loop ([`Loop`]) is run millisecond by
//! millisecond as `timeline::play` runs it; the input it is given is kept, so that the window and
//! the headless play run the same loop on the same input.
//!
//! - **A walk** ([`Command::Walk`]): its hero chosen ([`Action::Choose`], its place in the
//!   kingdom's list), then after a wait its path walked ([`Action::Walk`]).
//! - **End Turn**: its hotkey pressed and released.
//! - **A box** (a find, a question, End Turn's question, the week's message, a level-up): its
//!   answer when the record's next command is one (an offer's yes Enter, its no Escape, a level-up
//!   its offer's key; the other choices Enter, *provisional*), else Enter (OK).
//! - **A command of a screen the map has not** ([`Command::Other`]: a town's, a hero's, an
//!   army's; M5's): nothing shown, its events folded with the others'.
//!
//! - **A battle an answer began** ([`Command::Battle`], E24 M4 T5b1, design §5.20): handed over
//!   where the answer's box closes (its key's release), or where the answer was passed over with
//!   no box open ([`Directed::battles`]; a walk's battle is the map's own, at the walk's end:
//!   `Loop::battles`).
//!
//! The record has played to its end once its last command has been given and the map waits again.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use h3_map::model::Pos;
use h3_protocol::wire::game::{Answer, Consent, SkillOffer};
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
    /// The battles the record's answers began (E24 M4 T5b1, [`Command::Battle`]), in order: each
    /// at the millisecond the map hands it over, with the record's number of the `GameEvents`
    /// that began it.
    pub battles: Vec<(u32, u32)>,
}

/// The input that plays `commands` (the viewer's, in order) on the map from `start` under
/// `rules`, each hero's place in the kingdom's list by `slots` (its record id, its place), at
/// `pace`; at most until `limit` ms.
///
/// # Errors
/// What the map cannot play, with its millisecond; a walk's hero `slots` does not hold, or a
/// record not at its end by `limit` ([`Unplayable::Record`]).
pub fn direct(
    rules: Rules,
    start: &Start,
    commands: &[Command],
    slots: &[(u16, usize)],
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
            let answer = match commands.get(next) {
                Some(Command::Answer(answer)) => {
                    next += 1;
                    Some(*answer)
                }
                _ => None,
            };
            match answer {
                Some(Answer::LevelUp { offer }) => {
                    give(at, Action::Offer(usize::from(offer == SkillOffer::Second)));
                    give(at.saturating_add(pace.hold), Action::Up);
                }
                answer => {
                    let key = match answer {
                        Some(Answer::Offer {
                            consent: Consent::Decline,
                        }) => Key::Cancel,
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
                let slot = slots
                    .iter()
                    .find(|(id, _)| *id == hero)
                    .map(|&(_, slot)| slot)
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

/// A tile as the timeline's `[x, y, level]`.
fn at_of(at: Pos) -> [i32; 3] {
    [i32::from(at.x), i32::from(at.y), i32::from(at.z)]
}
