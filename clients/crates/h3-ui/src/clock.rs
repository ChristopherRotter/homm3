//! A viewer's clock in the window (E17 M2 T5a, the battle's; the map's since E24 M2 T4a, common
//! since E24 M3 T1a): the adapter's clock in (nanoseconds from any origin), the viewer's time out
//! (a battle's, a map's), with a pause, a restart and steps. Pure: the window's adapter reads its
//! clock and passes the time in.

/// Nanoseconds in a millisecond.
const MS: u64 = 1_000_000;

/// The viewer's time as a function of the adapter's: running from `mark`, or held.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clock {
    /// The viewer's time at `mark`, in nanoseconds.
    at_mark: u64,
    /// The adapter's time it last ran from.
    mark: u64,
    /// The viewer's time while paused.
    held: Option<u64>,
}

impl Clock {
    /// A clock running from the viewer's `from_ms` at the adapter's `now`.
    #[must_use]
    pub fn new(now: u64, from_ms: u32) -> Self {
        Self {
            at_mark: u64::from(from_ms) * MS,
            mark: now,
            held: None,
        }
    }

    /// The viewer's time at the adapter's `now`, in nanoseconds.
    #[must_use]
    pub fn ns(&self, now: u64) -> u64 {
        self.held
            .unwrap_or_else(|| self.at_mark.saturating_add(now.saturating_sub(self.mark)))
    }

    /// The viewer's time at `now` in whole milliseconds (the original's clock).
    #[must_use]
    pub fn ms(&self, now: u64) -> u32 {
        u32::try_from(self.ns(now) / MS).unwrap_or(u32::MAX)
    }

    /// Whether the clock is held.
    #[must_use]
    pub fn paused(&self) -> bool {
        self.held.is_some()
    }

    /// Holds a running clock at `now`, or runs a held one on from where it stopped.
    pub fn toggle(&mut self, now: u64) {
        match self.held.take() {
            Some(held) => {
                self.at_mark = held;
                self.mark = now;
            }
            None => self.held = Some(self.ns(now)),
        }
    }

    /// Moves the viewer's time to `ms` at `now`, held or running as it was (a restart, a step).
    pub fn seek(&mut self, now: u64, ms: u32) {
        let to = u64::from(ms) * MS;
        if self.held.is_some() {
            self.held = Some(to);
        } else {
            self.at_mark = to;
            self.mark = now;
        }
    }

    /// The adapter's nanoseconds from `now` until the viewer's time reaches `ms` (0 if it has); `None`
    /// while held.
    #[must_use]
    pub fn until(&self, now: u64, ms: u32) -> Option<u64> {
        (!self.paused()).then(|| (u64::from(ms) * MS).saturating_sub(self.ns(now)))
    }

    /// How far the viewer's time at `now` is past `ms`, in nanoseconds (0 if not yet): a frame's delay.
    #[must_use]
    pub fn late(&self, now: u64, ms: u32) -> u64 {
        self.ns(now).saturating_sub(u64::from(ms) * MS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_runs_pauses_and_seeks() {
        let mut c = Clock::new(5 * MS, 100);
        assert_eq!(c.ms(5 * MS), 100);
        assert_eq!(c.ms(7 * MS + 999_999), 102);
        assert_eq!(c.ns(0), 100 * MS);
        assert!(!c.paused());
        assert_eq!(c.until(5 * MS, 110), Some(10 * MS));
        assert_eq!(c.until(50 * MS, 110), Some(0));
        assert_eq!(c.late(50 * MS, 110), 35 * MS);
        assert_eq!(c.late(5 * MS, 110), 0);
        // Held at 20 ms: 115 ms of the viewer's, whatever the adapter's time.
        c.toggle(20 * MS);
        assert!(c.paused());
        assert_eq!(c.ms(90 * MS), 115);
        assert_eq!(c.until(90 * MS, 200), None);
        // Stepped while held, then run on from there.
        c.seek(95 * MS, 150);
        assert_eq!(c.ms(1000 * MS), 150);
        c.toggle(1000 * MS);
        assert_eq!(c.ms(1010 * MS), 160);
        // Restarted while running.
        c.seek(2000 * MS, 0);
        assert_eq!(c.ms(2003 * MS), 3);
        assert_eq!(Clock::new(0, 0).ms(u64::MAX), u32::MAX);
    }
}
