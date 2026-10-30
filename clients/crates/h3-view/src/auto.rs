//! Auto Combat (E17 M5 T4c1; `docs/epics/E17-design.md` §7.22). Its button (or its key, whose
//! release clicks it) toggles the battle's flag +0x132C4 on a human's turn (`battle_buttons`
//! 0x474E5A): from then the computer plays every stack of the battle's human players
//! (`stack_automatic` 0x474BF0, with the setting `Combat Auto Creatures`; the war machines by
//! their own settings), the turn under way at once. While it plays, each turn's start reads the
//! input queue up to a key or a mouse button (`battle_loop_turn` 0x4741F9: the moves and the
//! releases read and dropped) and, if one waits there, stops it (0x474251) and drops the rest:
//! a key or a button pressed while it plays stops it as the next turn starts, and does nothing
//! else. This module says which turns it plays, from when.

use alloc::vec::Vec;

use crate::mouse::track::Input;

#[cfg(test)]
mod tests;

/// When Auto Combat plays a battle's turns: the opening's first, then each one started, in the
/// timeline's order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Auto {
    /// Per turn: whether Auto Combat plays it from its start (switched on in an earlier turn),
    /// and from when it plays it (none: never).
    turns: Vec<(bool, Option<u32>)>,
}

impl Auto {
    /// Auto Combat over the turns starting at `starts` (in order, the opening's at 0 first) by
    /// the switches and stops among `inputs` (the others do nothing here), each in the turn it
    /// names (T8b2, R5c-1: not by its time, which a command that takes no time shares with the
    /// next turn's start; one naming an earlier turn than the input before it, or a turn past the
    /// last, is in that one's): [`Input::Auto`] switches it on unless it plays; [`Input::Stop`]
    /// while it plays stops it as the next turn starts. The battle takes a switch only on a
    /// human's turn waiting for a command, with the button enabled (`Battle::switch_auto`); a
    /// sequence's `key:a` is trusted to be one (the harness pressed it on such a turn).
    #[must_use]
    pub fn of(inputs: &[Input], starts: &[u32]) -> Self {
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
            // 0x474251: the press found as this turn starts stops it.
            on &= !core::mem::take(&mut stopped);
            let on_at_start = on;
            let mut from = on.then_some(start);
            let last = k + 1 == starts.len();
            while let Some((_, switch)) = inputs.next_if(|&(turn, _)| turn <= k || last) {
                match switch {
                    Some(at) if !on => {
                        on = true;
                        from = Some(at);
                    }
                    Some(_) => {}
                    None => stopped |= on,
                }
            }
            turns.push((on_at_start, from));
        }
        Self { turns }
    }

    /// Whether Auto Combat plays turn `k` (0 the opening's) at `t`, a time in it.
    #[must_use]
    pub fn plays(&self, k: usize, t: u32) -> bool {
        self.turns
            .get(k)
            .and_then(|&(_, from)| from)
            .is_some_and(|from| from <= t)
    }

    /// Whether Auto Combat played turn `k` at any time in it (its button shows selected since:
    /// 0x4784F9).
    #[must_use]
    pub fn played(&self, k: usize) -> bool {
        self.turns.get(k).is_some_and(|(_, from)| from.is_some())
    }

    /// Whether Auto Combat plays turn `k` from its start, switched on in an earlier turn: the
    /// computer's turn then (the mouse's moves are not taken, its start shows the pointer and
    /// unmarks the mouse's stack).
    #[must_use]
    pub fn from_start(&self, k: usize) -> bool {
        self.turns.get(k).is_some_and(|&(at_start, _)| at_start)
    }
}

/// The turn started last by `at` (0 the opening's) when each later one starts as `turns` says
/// (in order: a timeline's `turns`); of two starting then, the later: the turn a sequence's
/// switch or stop made at `at` is in.
#[must_use]
pub fn turn_at(turns: &[(u32, u8)], at: u32) -> usize {
    turns.partition_point(|&(t, _)| t <= at)
}
