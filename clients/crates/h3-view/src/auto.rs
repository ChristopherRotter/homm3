//! Auto Combat (E17 M5 T4c1; `docs/epics/E17-design.md` §7.22). Its button (or its key, whose
//! release clicks it) toggles the battle's flag +0x132C4 on a human's turn (`battle_buttons`
//! 0x474E5A): from then the computer plays every stack of the battle's human players
//! (`stack_automatic` 0x474BF0, with the setting `Combat Auto Creatures`; the war machines by
//! their own settings), the turn under way at once. While it plays, each turn's start reads the
//! input queue up to a key or a mouse button (`battle_loop_turn` 0x4741F9: the moves and the
//! releases read and dropped) and, if one waits there, stops it (0x474251) and drops the rest:
//! a key or a button pressed while it plays stops it as the next turn starts, and does nothing
//! else. This module says which turns it plays, from when. E17 M6 T4b1 (design §12.13): a human's
//! stack plays itself only when its box is set ([`plays_itself`]); over one that does not, the
//! turn is the human's while the flag stays on, its click switching it off (a toggle). E17 M6
//! T5b1 (design §12.18): with Combat Auto Spells set the computer is asked for a human's hero's
//! cast as each turn starts under it ([`Auto::asks_cast`], by the boxes then, [`boxes_at`]),
//! whether the stack plays itself or not. E17 M6 T5b2b (design §12.20, CLT-Q61: copied): such a
//! cast is an action of its own, dispatched at the loop's next pass (0x474477), which reads the
//! input queue again: a key or a button pressed during the computer's cast on a turn Auto Combat
//! plays stops it at the cast's end, the stack's action the human's ([`Auto::casting`]).

use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::Side;
use h3_render::Bugs;

use crate::battle_log::Logged;
use crate::hero::Action;
use crate::mouse::track::Input;

#[cfg(test)]
mod tests;

/// The original ids of the war machines with a box of their own (`stack_automatic` 0x474BF0, its
/// case table 0x474D64): the Catapult, the Ballista, the First Aid Tent, the arrow tower.
const CATAPULT: u16 = 145;
const BALLISTA: u16 = 146;
const FIRST_AID_TENT: u16 = 147;
const ARROW_TOWER: u16 = 149;

/// Combat Auto Creatures' place in `Settings::auto` (`crate::options::Settings`).
pub const CREATURES: usize = 0;
/// Combat Auto Spells' place: no stack's, the computer's casts for a human's hero
/// ([`Auto::asks_cast`]).
pub const SPELLS: usize = 1;
/// Combat Catapult's place.
pub const CATAPULT_BOX: usize = 2;
/// Combat Ballista's place.
pub const BALLISTA_BOX: usize = 3;
/// Combat First Aid Tent's place.
pub const TENT_BOX: usize = 4;

/// Whether a human player's stack whose creature has the original id `creature` (none: not
/// known, a creature's) plays itself under Auto Combat with the boxes `boxes` (`Settings::auto`;
/// E17 M6 T4b1, design §12.13: `stack_automatic` 0x474BF0 by its case table 0x474D64): the
/// Catapult by Combat Catapult, the Ballista and an arrow tower by Combat Ballista (the original's
/// oddity 3, copied), the First Aid Tent by Combat First Aid Tent, any other by Combat Auto
/// Creatures. A war machine whose turn waits for a human is a commanded one (its hero has its
/// skill): the engine plays the others itself (BAT-WAR-002, -003), so no skill is tested here.
/// Combat Auto Spells (`boxes[SPELLS]`) is no stack's: it rules the computer's casts for a
/// human's hero (`ai_hero_cast_turn` 0x422DA0; E17 M6 T5b1, [`Auto::asks_cast`]).
#[must_use]
pub const fn plays_itself(creature: Option<u16>, boxes: [bool; 5]) -> bool {
    match creature {
        Some(CATAPULT) => boxes[CATAPULT_BOX],
        Some(BALLISTA | ARROW_TOWER) => boxes[BALLISTA_BOX],
        Some(FIRST_AID_TENT) => boxes[TENT_BOX],
        _ => boxes[CREATURES],
    }
}

/// A turn's acting stack to Auto Combat (E17 M6 T4b1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Acting {
    /// It plays itself whenever Auto Combat is on: the computer's (`stack_automatic`'s player's
    /// test), or one the engine plays itself (a war machine without its hero's skill, BAT-WAR-002,
    /// -003: no command waits for it).
    Itself,
    /// A human player's whose command the battle waits for, its creature's original id (none:
    /// not known): it plays itself under Auto Combat by its box ([`plays_itself`]).
    Human(Option<u16>),
}

impl Acting {
    /// Whether it plays itself under Auto Combat with the boxes `boxes`.
    #[must_use]
    pub const fn plays_itself(self, boxes: [bool; 5]) -> bool {
        match self {
            Self::Itself => true,
            Self::Human(creature) => plays_itself(creature, boxes),
        }
    }
}

/// A box change: the turn whose dialog's close made it (0 the opening's), its time, the boxes
/// after it.
pub type BoxChange = (usize, u32, [bool; 5]);

/// From when each turn's stack plays itself under Auto Combat ([`Auto::of`]'s `itself`; E17 M6
/// T4b1, *provisional*): `stacks` the turns' (in order, the opening's first), `first` the boxes
/// the battle began with, `changes` the combat options dialog's closes that changed them (in
/// order). A turn plays itself from its start (0) by the boxes then, or from the last close in
/// it that set its stack's box (the original's oddity 4: the close's next pass hands the turn
/// under way to the computer, 0x4744C0), never after one that cleared it; a change counts for the
/// turns after it too.
#[must_use]
pub fn itself(stacks: &[Acting], first: [bool; 5], changes: &[BoxChange]) -> Vec<Option<u32>> {
    let mut boxes = first;
    let mut changes = changes.iter().peekable();
    stacks
        .iter()
        .enumerate()
        .map(|(k, stack)| {
            // The changes in order: those of the turns before were taken in their own turns.
            let mut from = stack.plays_itself(boxes).then_some(0);
            while let Some(&(_, at, set)) = changes.next_if(|&&(turn, _, _)| turn == k) {
                boxes = set;
                from = match (from, stack.plays_itself(boxes)) {
                    (_, false) => None,
                    (None, true) => Some(at),
                    (kept, true) => kept,
                };
            }
            from
        })
        .collect()
}

/// The boxes as turn `k` starts (0 the opening's; E17 M6 T5b1, *provisional*): `first` the boxes
/// the battle began with, `changes` the combat options dialog's closes since (in order, each with
/// its turn): the last made in a turn before `k`, else `first`. A dialog opens only on a human's
/// turn waiting, so a change in turn `k` comes after its start.
#[must_use]
pub fn boxes_at(first: [bool; 5], changes: &[BoxChange], k: usize) -> [bool; 5] {
    (changes.iter())
        .take_while(|&&(turn, _, _)| turn < k)
        .last()
        .map_or(first, |&(_, _, boxes)| boxes)
}

/// A human's hero's cast in a turn (E17 M6 T5b2b; [`casts`]): its action's index among the
/// timeline's actions, its start and its end (the hero standing).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cast {
    /// The action's index.
    pub action: usize,
    /// Its start.
    pub start: u32,
    /// Its end.
    pub end: u32,
}

/// Each turn's first hero's cast by a side `human` names a human's (E17 M6 T5b2b; the opening's
/// turn first, then one per `Logged::Turn` of `log`), `actions` the timeline's (one per
/// `Logged::Action`): under Auto Combat the computer's cast for the human's hero, whose end
/// reads a stop ([`Auto::casting`]); none in a turn without one.
#[must_use]
pub fn casts(log: &[(u32, Logged)], actions: &[Action], human: [bool; 2]) -> Vec<Option<Cast>> {
    let mut out = vec![None];
    let mut next = 0;
    for (_, entry) in log {
        match entry {
            Logged::Turn(_) => out.push(None),
            Logged::Action => {
                let action = next;
                next += 1;
                let cast = (actions.get(action)).filter(|a| {
                    a.cast
                        .is_some_and(|s| human[usize::from(s == Side::Defender)])
                });
                if let (Some(a), Some(slot @ None)) = (cast, out.last_mut()) {
                    *slot = Some(Cast {
                        action,
                        start: a.start,
                        end: a.end,
                    });
                }
            }
            _ => {}
        }
    }
    out
}

/// When Auto Combat plays a battle's turns: the opening's first, then each one started, in the
/// timeline's order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Auto {
    turns: Vec<Turn>,
}

/// Auto Combat over a turn.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Turn {
    /// The flag (+0x132C4) as the turn starts, its start's stop read.
    on: bool,
    /// It plays the turn from its start (switched on in an earlier turn, its stack playing
    /// itself).
    from_start: bool,
    /// When it plays the turn, in order: from a time until one (none: to the turn's end); none
    /// at all: never. E17 M6 T5b2b: a stop read at the computer's cast's end ends the first, a
    /// switch after it may start a second.
    plays: Vec<(u32, Option<u32>)>,
    /// The times of the switches the turn took (each toggles the flag; E17 M6 T5b2b: a stop read
    /// at a cast's end too).
    switches: Vec<u32>,
    /// A key or a button pressed while it was on waits for the next turn's start.
    stopped: bool,
}

impl Auto {
    /// Auto Combat over the turns starting at `starts` (in order, the opening's at 0 first) by
    /// the switches and stops among `inputs` (the others do nothing here), each in the turn it
    /// names (T8b2, R5c-1: not by its time, which a command that takes no time shares with the
    /// next turn's start; one naming an earlier turn than the input before it, or a turn past the
    /// last, is in that one's), each turn's stack playing itself under it from the time `itself`
    /// says ([`itself`]; none: never, it is the human's; a turn past it from its start: E17 M6
    /// T4b1, design §12.13).
    ///
    /// [`Input::Auto`] toggles the flag (0x474E5A) on a turn it does not play: on over a stack
    /// that plays itself, it plays the turn from then; off, the turns after it are the human's.
    /// A box set while it is on hands the turn to it from then. [`Input::Stop`] while it is on
    /// stops it as the next turn it plays starts (0x474251: the stop is read only there); the
    /// turn's start of one it does not play reads none (0x4741F0) and the press is gone (the
    /// original's acts on that turn, the client's drops it: CLT-Q45; the flag stays on either
    /// way). The battle takes a switch only on a human's turn waiting for a command, with the
    /// button enabled (`Battle::switch_auto`); a sequence's `key:a` is trusted to be one (the
    /// harness pressed it on such a turn).
    #[must_use]
    pub fn of(inputs: &[Input], starts: &[u32], itself: &[Option<u32>]) -> Self {
        Self::casting(inputs, starts, itself, &[])
    }

    /// [`Auto::of`] with each turn's human's hero's cast `casts` ([`casts`]; E17 M6 T5b2b, design
    /// §12.20, CLT-Q61: copied in both profiles). A cast that started while Auto Combat played
    /// the turn (as it started, or at a switch: the computer's for the human's hero, §12.18) is
    /// an action of its own, the stack's action dispatched at the loop's next pass (0x474477),
    /// which reads the input queue again (0x4741F9): a stop pressed in the turn before the cast's
    /// end stops Auto Combat there (0x474251), the turn the human's from then and the flag off,
    /// not as the next turn starts. One at or after the cast's end, or in a turn without such a
    /// cast, waits for the next turn's start as before (*provisional*: a press in the cast's
    /// last millisecond, the original's action in that tick, is the next turn's).
    #[must_use]
    pub fn casting(
        inputs: &[Input],
        starts: &[u32],
        itself: &[Option<u32>],
        casts: &[Option<Cast>],
    ) -> Self {
        let mut turns = Vec::with_capacity(starts.len());
        let (mut on, mut stopped) = (false, false);
        let mut inputs = inputs
            .iter()
            .filter_map(|input| match *input {
                Input::Auto { at, turn } => Some((turn, Ok(at))),
                Input::Stop { at, turn } => Some((turn, Err(at))),
                _ => None,
            })
            .peekable();
        for (k, &start) in starts.iter().enumerate() {
            let itself = itself.get(k).copied().unwrap_or(Some(0));
            let plays_at = |t: u32| itself.is_some_and(|from| from <= t);
            // 0x474251: the press found as a turn it plays starts stops it.
            if on && plays_at(start) {
                on &= !stopped;
            }
            stopped = false;
            let mut turn = Turn {
                on,
                from_start: on && plays_at(start),
                ..Turn::default()
            };
            // From when it plays the turn now (none: it does not).
            let mut from = turn.from_start.then_some(start);
            let cast = casts.get(k).copied().flatten();
            let last = k + 1 == starts.len();
            while let Some((_, input)) = inputs.next_if(|&(turn, _)| turn <= k || last) {
                match input {
                    Ok(at) => {
                        // A box set before it, while on, handed the turn over.
                        if from.is_none() && on {
                            from = itself.filter(|&from| from <= at).map(|f| f.max(start));
                        }
                        if from.is_some() {
                            continue;
                        }
                        on = !on;
                        turn.switches.push(at);
                        from = (on && plays_at(at)).then_some(at);
                    }
                    // The computer's cast it played: read at its end (CLT-Q61).
                    Err(at) => match (from, cast) {
                        (Some(f), Some(c)) if on && f <= c.start && at < c.end => {
                            turn.plays.push((f, Some(c.end)));
                            turn.switches.push(c.end);
                            (from, on) = (None, false);
                        }
                        _ => stopped |= on,
                    },
                }
            }
            if from.is_none() && on {
                from = itself.map(|from| from.max(start));
            }
            turn.plays.extend(from.map(|f| (f, None)));
            turn.stopped = stopped;
            turns.push(turn);
        }
        Self { turns }
    }

    /// Whether Auto Combat plays turn `k` (0 the opening's) at `t`, a time in it.
    #[must_use]
    pub fn plays(&self, k: usize, t: u32) -> bool {
        self.turns.get(k).is_some_and(|turn| {
            (turn.plays.iter()).any(|&(from, until)| from <= t && until.is_none_or(|u| t < u))
        })
    }

    /// Whether Auto Combat plays turn `k` at `t` from a switch in the turn (its click or A
    /// selecting its button), not from the turn's start (E17 M6 T5b2b, the panel's
    /// [`crate::panel::Autoplay::switched`]); so from a box set in the turn while it is on (the
    /// options dialog's, handing the turn over: shown as a switch, *provisional*, not recorded).
    #[must_use]
    pub fn switched(&self, k: usize, t: u32) -> bool {
        self.turns.get(k).is_some_and(|turn| {
            (turn.plays.iter().enumerate()).any(|(i, &(from, until))| {
                from <= t && until.is_none_or(|u| t < u) && (i > 0 || !turn.from_start)
            })
        })
    }

    /// Whether Auto Combat plays turn `k` at its end, having played it from a time in it (its
    /// button shows selected since: 0x4784F9; E17 M6 T5b2b: a stop read at the computer's cast's
    /// end runs the turn's start again without it, which unselects it, 0x47864A).
    #[must_use]
    pub fn played(&self, k: usize) -> bool {
        (self.turns.get(k)).is_some_and(|turn| turn.plays.last().is_some_and(|p| p.1.is_none()))
    }

    /// When Auto Combat stopped in turn `k`, at the end of the computer's cast for the human's
    /// hero (E17 M6 T5b2b, CLT-Q61; [`Auto::casting`]): from then the turn is the human's,
    /// waiting for its command as after a stop read as it started. None: no such stop.
    #[must_use]
    pub fn stopped_at(&self, k: usize) -> Option<u32> {
        let turn = self.turns.get(k)?;
        turn.plays.first().and_then(|&(_, until)| until)
    }

    /// The index of the turn the stop read last in turn `k` comes at (E17 M5 T4c1; E17 M6 T5b2b,
    /// CLT-Q61): `k` itself when that stop ended Auto Combat's play at the computer's cast's end
    /// ([`Auto::stopped_at`]; not one after a switch back on, which waits as any), else the next
    /// turn's start, `k + 1`.
    #[must_use]
    pub fn stop_comes(&self, k: usize) -> usize {
        if self.stopped_at(k).is_some() && !self.stopped(k) {
            k
        } else {
            k + 1
        }
    }

    /// The end of turn `k`'s cast `cast` ([`casts`]) while it is the battle's action `last` (its
    /// last played) on a turn Auto Combat played as it started (E17 M6 T5b2b, CLT-Q61): a key or
    /// a button pressed until then stops Auto Combat there, so the stack's action is asked only
    /// once the clock reaches that end (the window's). None: no such cast.
    #[must_use]
    pub fn holds(&self, k: usize, cast: Option<Cast>, last: usize) -> Option<u32> {
        cast.filter(|c| c.action == last && self.plays(k, c.start))
            .map(|c| c.end)
    }

    /// The actions after which Auto Combat stopped (E17 M6 T5b2b): each by its index among
    /// `casts`' (the ones [`Auto::casting`] took), with the stop's time, its end.
    #[must_use]
    pub fn handed(&self, casts: &[Option<Cast>]) -> Vec<(usize, u32)> {
        (casts.iter().enumerate())
            .filter_map(|(k, cast)| Some((cast.as_ref()?.action, self.stopped_at(k)?)))
            .collect()
    }

    /// Whether Auto Combat plays turn `k` from its start, switched on in an earlier turn: the
    /// computer's turn then (the mouse's moves are not taken, its start shows the pointer and
    /// unmarks the mouse's stack).
    #[must_use]
    pub fn from_start(&self, k: usize) -> bool {
        self.turns.get(k).is_some_and(|turn| turn.from_start)
    }

    /// Whether Auto Combat is on (the flag) at `t` in turn `k`, whether it plays the turn or not
    /// (E17 M6 T4b1: on over a stack it does not play, the turn is the human's).
    #[must_use]
    pub fn on(&self, k: usize, t: u32) -> bool {
        self.turns.get(k).is_some_and(|turn| {
            let switched = turn.switches.iter().filter(|&&at| at <= t).count();
            turn.on ^ (switched % 2 == 1)
        })
    }

    /// Whether Auto Combat was on as turn `k` started, its start's stop read.
    #[must_use]
    pub fn on_at_start(&self, k: usize) -> bool {
        self.turns.get(k).is_some_and(|turn| turn.on)
    }

    /// Whether the computer is asked for a human's hero's cast as turn `k` starts (E17 M6 T5b1,
    /// design §12.18: `ai_hero_cast_turn` 0x422DA0 at the end of `battle_controls_state`, its
    /// gate 0x422E42): Auto Combat on as the turn starts, its start's stop read (a stop found
    /// there runs `battle_controls_state` again without it, which clears the cast set: 0x47425B),
    /// and Combat Auto Spells set among `boxes`, the boxes then ([`boxes_at`]). The original's
    /// asks it whether the stack plays itself or not; the client's profile (`bugs`, CLT-Q62,
    /// the user 2026-11-05) only on a turn Auto Combat plays from its start, a turn the human
    /// plays keeping the hero's cast the human's. The hero's own tests (cast this round, a war
    /// machine's turn, the tactics phase, its book, the Orb, a garrison) are the engine's.
    #[must_use]
    pub fn asks_cast(&self, k: usize, boxes: [bool; 5], bugs: Bugs) -> bool {
        boxes[SPELLS] && self.on_at_start(k) && (bugs == Bugs::Original || self.from_start(k))
    }

    /// Whether Auto Combat's switch at `t` in turn `k` asks the computer for the hero's cast (E17
    /// M6 T5b1: the switch runs the turn's start again, 0x474E73): on at `t` and Combat Auto
    /// Spells set among `boxes`, the boxes now; in the client's profile (CLT-Q62) only when it
    /// then plays the turn.
    #[must_use]
    pub fn asks_cast_at(&self, k: usize, t: u32, boxes: [bool; 5], bugs: Bugs) -> bool {
        boxes[SPELLS] && self.on(k, t) && (bugs == Bugs::Original || self.plays(k, t))
    }

    /// Whether a key or a button pressed while it was on waits at turn `k`'s end for the next
    /// turn's start.
    #[must_use]
    pub fn stopped(&self, k: usize) -> bool {
        self.turns.get(k).is_some_and(|turn| turn.stopped)
    }
}

/// The turn started last by `at` (0 the opening's) when each later one starts as `turns` says
/// (in order: a timeline's `turns`); of two starting then, the later: the turn a sequence's
/// switch or stop made at `at` is in.
#[must_use]
pub fn turn_at(turns: &[(u32, u8)], at: u32) -> usize {
    turns.partition_point(|&(t, _)| t <= at)
}
