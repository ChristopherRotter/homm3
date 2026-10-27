//! The map played forward as the window asks (E24 M2 T4a, *provisional*): the timeline's loop
//! ([`Loop`]) run from a start and the actions at their milliseconds, as far as the window's clock
//! needs, giving the changes of what the screen shows (the view's draw and the cursor) with their
//! times. The window composes a change's frame ahead of its time and shows it when it falls due,
//! a change overtaken by a later one passed over (`h3-view window`'s way, E17 design §8.1).
//! `sequences` plays the same loop ([`crate::timeline::play`]), so each change is a frame the
//! headless check draws; past the actions the map stands still on, the loop's redraws and palette
//! steps going on. Actions at 0 are not played, as `play` leaves them.

use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec::Vec;
use core::iter;

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
}

/// The loop run forward on demand, the changes kept from the one shown on.
#[derive(Debug, Clone)]
pub struct Replay {
    /// The loop after 0, for a restart.
    first: Loop,
    actions: BTreeMap<u32, Vec<Action>>,
    l: Loop,
    /// The last millisecond run.
    t: u32,
    /// The changes before `now` still kept, in time order.
    kept: VecDeque<Change>,
    /// The latest change, as of `t`.
    now: Change,
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
        let first = Loop::start(rules, start)?;
        let now = Change {
            t: 0,
            drawn: first.drawn().clone(),
            cursor: first.cursor(),
        };
        Ok(Self {
            l: first.clone(),
            first,
            actions,
            t: 0,
            kept: VecDeque::new(),
            now,
        })
    }

    /// Back to 0 (the changes kept forgotten).
    pub fn restart(&mut self) {
        self.l = self.first.clone();
        self.t = 0;
        self.kept.clear();
        self.now = Change {
            t: 0,
            drawn: self.l.drawn().clone(),
            cursor: self.l.cursor(),
        };
    }

    /// The last millisecond run.
    #[must_use]
    pub fn ran(&self) -> u32 {
        self.t
    }

    /// What the screen shows at `t`: the latest change at or before it, the loop run to `t` (the
    /// earliest change kept for a time before it).
    ///
    /// # Errors
    /// The millisecond of the first action the loop cannot play yet, and why; the replay is spent
    /// then (restart it).
    pub fn at(&mut self, t: u32) -> Result<Change, (u32, Unplayable)> {
        self.run_to(t)?;
        let mut found = &self.now;
        let mut earlier = self.kept.iter().rev();
        while found.t > t {
            match earlier.next() {
                Some(c) => found = c,
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
        if self.now.t <= t {
            self.kept.clear();
        } else {
            while self.kept.get(1).is_some_and(|c| c.t <= t) {
                self.kept.pop_front();
            }
        }
        let until = t.saturating_add(limit);
        while self.now.t <= t && self.t < until {
            self.run_to(self.t.saturating_add(1))?;
        }
        Ok(self
            .kept
            .iter()
            .chain(iter::once(&self.now))
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
            let (drawn, cursor) = (self.l.drawn(), self.l.cursor());
            if *drawn != self.now.drawn || cursor != self.now.cursor {
                let change = Change {
                    t: next,
                    drawn: drawn.clone(),
                    cursor,
                };
                self.kept
                    .push_back(core::mem::replace(&mut self.now, change));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
