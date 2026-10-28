//! The map played forward as the window asks (E24 M2 T4a, *provisional*): the timeline's loop
//! ([`Loop`]) run from a start and the actions at their milliseconds, as far as the window's clock
//! needs, giving the changes of what the screen shows (the view's draw, the cursor and, since E24
//! M3 T2a, move hero's and next hero's states) with their
//! times. The window composes a change's frame ahead of its time and shows it when it falls due,
//! a change overtaken by a later one passed over (`h3-view window`'s way, E17 design §8.1).
//! `sequences` plays the same loop ([`crate::timeline::play`]), so each change is a frame the
//! headless check draws; past the actions the map stands still on, the loop's redraws and palette
//! steps going on. Actions at 0 are not played, as `play` leaves them.
//!
//! The player's input (M2 T4b) is given as it comes ([`Replay::give`]), at a millisecond after the
//! change shown: the loop may have run past it to find the next change, so each change keeps the
//! loop as it stood at its millisecond and the loop runs again from the latest one before the
//! action. Given so, the changes are the ones a headless play of every action gives.
//!
//! The sound's cues (E24 M2 T5) are kept apart from the changes, by their milliseconds, until
//! the window takes them as its clock reaches them ([`Replay::take_cues`]); the loop run again
//! from a change gives the cues after it again, and the ones handed out already are not given
//! twice.

use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec::Vec;
use core::iter;

use crate::live::Buttons;
use crate::sound::Cue;
use crate::timeline::{Action, Cursor, Draw, Loop, Rules, Start, Unplayable};

/// What the screen shows from a millisecond on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// The millisecond.
    pub t: u32,
    /// The view's draw.
    pub drawn: Draw,
    /// The cursor.
    pub cursor: Cursor,
    /// Move hero's and next hero's states (E24 M3 T2a).
    pub buttons: Buttons,
}

/// The loop run forward on demand, the changes kept from the one shown on, each with the loop as
/// of its millisecond.
#[derive(Debug, Clone)]
pub struct Replay {
    /// The loop after 0, for a restart.
    first: Loop,
    actions: BTreeMap<u32, Vec<Action>>,
    l: Loop,
    /// The last millisecond run.
    t: u32,
    /// The changes before `now` still kept, in time order.
    kept: VecDeque<(Change, Loop)>,
    /// The latest change, as of `t`.
    now: (Change, Loop),
    /// The cues at 0 (the map's screen's start), given again at each restart.
    first_cues: Vec<(u32, Cue)>,
    /// The cues not taken yet, in time order.
    cues: VecDeque<(u32, Cue)>,
    /// The cues taken: up to this millisecond, that many at it.
    taken: Option<(u32, usize)>,
    /// The cues the loop gave again at the last taken millisecond since it last ran again.
    again: usize,
}

/// The change the loop shows now, with the loop.
fn shown(l: &Loop, t: u32) -> (Change, Loop) {
    (
        Change {
            t,
            drawn: l.drawn().clone(),
            cursor: l.cursor(),
            buttons: l.buttons(),
        },
        l.clone(),
    )
}

impl Replay {
    /// The map from `start`, with the actions at their milliseconds.
    ///
    /// # Errors
    /// Why the loop cannot start (the pointer over the view).
    pub fn new(
        rules: Rules,
        start: &Start,
        actions: BTreeMap<u32, Vec<Action>>,
    ) -> Result<Self, Unplayable> {
        let mut first = Loop::start(rules, start)?;
        let first_cues = first.take_cues();
        Ok(Self {
            l: first.clone(),
            now: shown(&first, 0),
            first,
            actions,
            t: 0,
            kept: VecDeque::new(),
            cues: first_cues.iter().cloned().collect(),
            first_cues,
            taken: None,
            again: 0,
        })
    }

    /// Back to 0 (the changes kept forgotten; the actions given kept), the sound from its start:
    /// the cues at 0 given again.
    pub fn restart(&mut self) {
        self.taken = None;
        self.rewind();
    }

    /// The loop from 0 again, the cues not taken forgotten and the ones at 0 given again but the
    /// ones taken.
    fn rewind(&mut self) {
        self.l = self.first.clone();
        self.t = 0;
        self.kept.clear();
        self.now = shown(&self.first, 0);
        self.cues.clear();
        self.again = 0;
        let first = self.first_cues.clone();
        self.keep_cues(first);
    }

    /// The cues the loop gave, but the ones taken already (the loop run again gives them again).
    fn keep_cues(&mut self, cues: Vec<(u32, Cue)>) {
        for (t, cue) in cues {
            if let Some((taken, n)) = self.taken {
                if t < taken {
                    continue;
                }
                if t == taken && self.again < n {
                    self.again += 1;
                    continue;
                }
            }
            self.cues.push_back((t, cue));
        }
    }

    /// The cues up to `t` not taken yet, in order, the loop run to `t` first.
    ///
    /// # Errors
    /// As [`Replay::at`].
    pub fn take_cues(&mut self, t: u32) -> Result<Vec<(u32, Cue)>, (u32, Unplayable)> {
        self.run_to(t)?;
        let due = self.cues.iter().take_while(|(c, _)| *c <= t).count();
        let out: Vec<(u32, Cue)> = self.cues.drain(..due).collect();
        let at_t = out.iter().filter(|(c, _)| *c == t).count();
        self.taken = match self.taken {
            Some((taken, n)) if taken == t => Some((t, n + at_t)),
            Some((taken, n)) if taken > t => Some((taken, n)),
            _ => Some((t, at_t)),
        };
        Ok(out)
    }

    /// The last millisecond run.
    #[must_use]
    pub fn ran(&self) -> u32 {
        self.t
    }

    /// The actions at their milliseconds, the ones given among them.
    #[must_use]
    pub fn actions(&self) -> &BTreeMap<u32, Vec<Action>> {
        &self.actions
    }

    /// An action given at `t` (the player's, live: at least 1, after the change shown), after the
    /// ones at `t` already: when the loop ran `t`, it runs again from the latest change kept
    /// before `t` (from 0 when none is: a time not after the change shown), the changes from `t`
    /// on forgotten.
    pub fn give(&mut self, t: u32, action: Action) {
        let t = t.max(1);
        self.actions.entry(t).or_default().push(action);
        if t > self.t {
            return;
        }
        while self.now.0.t >= t {
            match self.kept.pop_back() {
                Some(earlier) => self.now = earlier,
                None => {
                    self.rewind();
                    return;
                }
            }
        }
        self.l = self.now.1.clone();
        self.t = self.now.0.t;
        let from = self.t;
        self.cues.retain(|(c, _)| *c <= from);
        self.again = 0;
    }

    /// What the screen shows at `t`: the latest change at or before it, the loop run to `t` (the
    /// earliest change kept for a time before it).
    ///
    /// # Errors
    /// The millisecond of the first action the loop cannot play yet, and why; the replay is spent
    /// then (restart it).
    pub fn at(&mut self, t: u32) -> Result<Change, (u32, Unplayable)> {
        self.run_to(t)?;
        let mut found = &self.now.0;
        let mut earlier = self.kept.iter().rev();
        while found.t > t {
            match earlier.next() {
                Some((c, _)) => found = c,
                None => break,
            }
        }
        Ok(found.clone())
    }

    /// The first change after `t`, the loop run until one comes or `limit` milliseconds past `t`
    /// (none then); the changes before the one shown at `t` forgotten.
    ///
    /// # Errors
    /// As [`Replay::at`].
    pub fn after(&mut self, t: u32, limit: u32) -> Result<Option<Change>, (u32, Unplayable)> {
        if self.now.0.t <= t {
            self.kept.clear();
        } else {
            while self.kept.get(1).is_some_and(|(c, _)| c.t <= t) {
                self.kept.pop_front();
            }
        }
        let until = t.saturating_add(limit);
        while self.now.0.t <= t && self.t < until {
            self.run_to(self.t.saturating_add(1))?;
        }
        Ok(self
            .kept
            .iter()
            .chain(iter::once(&self.now))
            .map(|(c, _)| c)
            .find(|c| c.t > t)
            .cloned())
    }

    /// Runs the loop to `t` (nothing if it ran that far), keeping each change.
    fn run_to(&mut self, t: u32) -> Result<(), (u32, Unplayable)> {
        while self.t < t {
            let next = self.t.saturating_add(1);
            let given = self.actions.get(&next).map_or(&[][..], Vec::as_slice);
            self.l.run(next, given).map_err(|why| (next, why))?;
            self.t = next;
            let cues = self.l.take_cues();
            self.keep_cues(cues);
            if *self.l.drawn() != self.now.0.drawn
                || self.l.cursor() != self.now.0.cursor
                || self.l.buttons() != self.now.0.buttons
            {
                let change = shown(&self.l, next);
                self.kept
                    .push_back(core::mem::replace(&mut self.now, change));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
