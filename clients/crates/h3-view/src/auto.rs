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
//! turn is the human's while the flag stays on, its click switching it off (a toggle).

use alloc::vec::Vec;

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
/// Combat Auto Spells' place (kept only: T5's).
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
/// Combat Auto Spells (`boxes[SPELLS]`) is no stack's: it is kept only, the computer's casts for
/// a human's hero (`ai_hero_cast_turn` 0x422DA0) being T5's (the client's bot casts nothing).
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
            while let Some(&(_, _, set)) = changes.next_if(|&&(turn, _, _)| turn < k) {
                boxes = set;
            }
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
    /// From when it plays the turn (none: never).
    from: Option<u32>,
    /// The times of the switches the turn took (each toggles the flag).
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
        let mut turns = Vec::with_capacity(starts.len());
        let (mut on, mut stopped) = (false, false);
        let mut inputs = inputs
            .iter()
            .filter_map(|input| match *input {
                Input::Auto { at, turn } => Some((turn, Some(at))),
                Input::Stop { turn, .. } => Some((turn, None)),
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
            turn.from = turn.from_start.then_some(start);
            let last = k + 1 == starts.len();
            while let Some((_, switch)) = inputs.next_if(|&(turn, _)| turn <= k || last) {
                match switch {
                    Some(at) => {
                        // A box set before it, while on, handed the turn over.
                        if turn.from.is_none() && on {
                            turn.from = itself.filter(|&from| from <= at).map(|f| f.max(start));
                        }
                        if turn.from.is_some() {
                            continue;
                        }
                        on = !on;
                        turn.switches.push(at);
                        turn.from = (on && plays_at(at)).then_some(at);
                    }
                    None => stopped |= on,
                }
            }
            if turn.from.is_none() && on {
                turn.from = itself.map(|from| from.max(start));
            }
            turn.stopped = stopped;
            turns.push(turn);
        }
        Self { turns }
    }

    /// Whether Auto Combat plays turn `k` (0 the opening's) at `t`, a time in it.
    #[must_use]
    pub fn plays(&self, k: usize, t: u32) -> bool {
        self.turns
            .get(k)
            .and_then(|turn| turn.from)
            .is_some_and(|from| from <= t)
    }

    /// Whether Auto Combat played turn `k` at any time in it (its button shows selected since:
    /// 0x4784F9).
    #[must_use]
    pub fn played(&self, k: usize) -> bool {
        self.turns.get(k).is_some_and(|turn| turn.from.is_some())
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
