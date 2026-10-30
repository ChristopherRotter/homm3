//! The client's own pace (E17 M5 review R1-1, *provisional*; design §11.2): a server closes a
//! connection that sends more than its limit of messages a second (`h3-server`'s
//! `messages_per_second`, 20 by default) with `Goodbye(RateLimited)`, and the limit is not on the
//! wire. A turn's queries come in a burst (one per castable spell and one more), so the client
//! sends at most [`PACE_MOST`] messages within any [`PACE_WINDOW_MS`] of its clock, half the
//! server's default: the rest wait their turn, in order.

use alloc::collections::VecDeque;
use alloc::vec::Vec;

/// The most messages sent within [`PACE_WINDOW_MS`] (*provisional*: half `h3-server`'s default
/// `messages_per_second`, which leaves room for the network's jitter and the pongs it counts too).
pub const PACE_MOST: usize = 10;

/// The window [`PACE_MOST`] counts in, in milliseconds of the client's clock.
pub const PACE_WINDOW_MS: u64 = 1_000;

/// The messages a client sends, at its pace (the module's text).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pace<T> {
    /// When the latest sends went (at most [`PACE_MOST`]), oldest first, by the client's clock in
    /// milliseconds; none: before the clock was read (the window's opening waits without one),
    /// stamped at its next reading.
    sent: VecDeque<Option<u64>>,
    /// The messages waiting their turn, in order.
    waiting: VecDeque<T>,
}

impl<T> Default for Pace<T> {
    fn default() -> Self {
        Self {
            sent: VecDeque::new(),
            waiting: VecDeque::new(),
        }
    }
}

impl<T> Pace<T> {
    /// Queues `message` behind those waiting.
    pub fn push(&mut self, message: T) {
        self.waiting.push_back(message);
    }

    /// The messages that may go at `now` (the client's clock in milliseconds; none: not read, so
    /// every send so far counts as within the window), in order: at most [`PACE_MOST`] within any
    /// [`PACE_WINDOW_MS`].
    pub fn release(&mut self, now: Option<u64>) -> Vec<T> {
        if let Some(now) = now {
            for at in &mut self.sent {
                at.get_or_insert(now);
            }
            while let Some(&Some(at)) = self.sent.front()
                && now.saturating_sub(at) >= PACE_WINDOW_MS
            {
                self.sent.pop_front();
            }
        }
        let room = PACE_MOST.saturating_sub(self.sent.len());
        let ready: Vec<T> = self.waiting.drain(..room.min(self.waiting.len())).collect();
        self.sent.extend(ready.iter().map(|_| now));
        ready
    }

    /// How many messages wait.
    pub fn waiting(&self) -> usize {
        self.waiting.len()
    }

    /// Drops the messages waiting (a connection's end: they were its own); the sends stay counted.
    pub fn clear(&mut self) {
        self.waiting.clear();
    }
}
