//! A timed turn's clock on the map (E24 M5 T5 part five, *provisional*; `docs/epics/E24-design.md`
//! §5.32): the turn timer started at a human's start and stopped at its turn's end
//! (`turn_timer_start` 0x557D90, `turn_timer_stop` 0x557D60), its warnings from the tick every 60
//! ms (`turn_timer_poll` 0x557AF0: a line in the screen chat, `chat_system_message` 0x553960, and
//! `timeover.wav` unless the chat's last sample still plays), the chat aged at each view's draw
//! ([`Chat::draw`]: nothing drawn while a box or another screen is up, so the lines written
//! meanwhile show together at the map's next draw), and the turn's expiry at the adventure
//! manager's first message once its time is out (0x4087F3: GENRLTXT 202's box, its OK or its own
//! close 15 s on, then the turn's end without End Turn's question; a walk under way finishes first,
//! its steps inside the message that began it). A box opened with less than 15 s left closes by
//! itself 15 s on, and one open at the end by its style (`message_box_build` 0x4F6990,
//! `message_box_proc` 0x4F0906; survey `E16-timer-survey.md` §5): its caller's else, a question's
//! No. In play ([`TurnClock::hosted`]) the host's session ends the turn ([`Loop::expire`]): the box
//! open then is closed as the session's system answered it, no other box gets a deadline.

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_ui::message::{self as boxes, Open};

use super::{ChatLine, Loop, Told, TurnTimer, Unplayable};
use crate::clock;

impl Loop {
    /// The turn timer started at `t` (`turn_timer_start`: its start and last warning now, the
    /// first warning a quarter of the limit on, half of one of a minute or less); the view's first
    /// start takes the host's time left ([`super::TurnClock::left`]).
    pub(super) fn timer_start(&mut self, t: u32) {
        let Some(clock) = &self.clock else {
            return;
        };
        let now = i64::from(t);
        let spent = match clock.left.filter(|_| !self.left_taken) {
            Some(left) => i64::from(clock.limit) - i64::from(left),
            None => 0,
        };
        self.left_taken = true;
        let start = now - spent;
        self.turn_timer = Some(TurnTimer {
            start,
            last: start,
            next: clock::first(clock.limit),
            at: t,
        });
    }

    /// The turn timer stopped (`turn_timer_stop`: the turn's end, the next player's turn).
    pub(super) fn timer_stop(&mut self) {
        self.turn_timer = None;
    }

    /// The warnings' poll at `t` (`turn_timer_poll`): with a next warning due (the clock its
    /// interval past the last), the time left `r`: negative, no more; else the warning `r` shows
    /// in the chat, the last warning now and the next by [`clock::next`]; its sound unless the
    /// chat's last sample still plays (`sample_query` 0x59A030 with 4: the warning's own sound, or
    /// the step sound looping on the same sample, category 0's: recorded, `seat-clock-walk`'s
    /// 58 199 silent under the walk's step sound).
    pub(super) fn poll_timer(&mut self, t: u32) {
        let (Some(clock), Some(timer)) = (&self.clock, &mut self.turn_timer) else {
            return;
        };
        let now = i64::from(t);
        if timer.next == 0 || now - timer.last <= i64::from(timer.next) {
            return;
        }
        let Ok(left) = u32::try_from(timer.start + i64::from(clock.limit) - now) else {
            timer.next = 0;
            return;
        };
        timer.last = now;
        timer.next = clock::next(left, clock.limit);
        self.warned = Some((t, timer.next));
        self.chat
            .add(clock::warning(left), self.rules.clock.entries);
        let stepping = self
            .sound
            .as_ref()
            .is_some_and(crate::sound::Sound::stepping);
        let plays = self.chat_sample.is_some_and(|end| t < end || stepping);
        if !plays && let Some(sound) = &mut self.sound {
            sound.effect(t, &self.rules.clock.sound);
            self.chat_sample = Some(t.saturating_add(clock.sample.unwrap_or(0)));
        }
    }

    /// The chat's lines by their first draws, −1 not drawn yet (the timeline's `chat`).
    pub(super) fn chat_drawn(&self) -> Vec<i64> {
        self.chat
            .lines()
            .iter()
            .map(|l| l.drawn.unwrap_or(-1))
            .collect()
    }

    /// The chat at a view's draw at `t` ([`Chat::draw`](crate::clock::Chat::draw)): the lines it
    /// shows, those of the last draw when none changed.
    pub(super) fn chat_draw(&mut self, t: u32) -> Arc<Vec<clock::Warning>> {
        if self.chat.draw(i64::from(t), self.rules.clock.life) {
            Arc::new(self.chat.shown())
        } else {
            Arc::clone(&self.drawn.chat)
        }
    }

    /// The adventure manager's test at a message at `t` (0x4087F3, before anything else): once
    /// the turn's time is out (the time since the start past the limit, or the host's expiry),
    /// the expiry's box opens and the message is not handled. Whether it opened.
    pub(super) fn expires(&mut self, t: u32) -> bool {
        let (Some(clock), Some(timer)) = (&self.clock, &self.turn_timer) else {
            return false;
        };
        let out = if clock.hosted {
            self.expired_at.is_some_and(|at| t >= at)
        } else {
            i64::from(t) - timer.start > i64::from(clock.limit)
        };
        if !out {
            return false;
        }
        let expired = Arc::clone(&clock.expired);
        self.open_box(expired, Told::Expired);
        true
    }

    /// A box's deadline as it opens now (`message_box_build`): the expiry's 15 s; another's when
    /// it opens with less than that left in a timed turn the loop times itself (not a level-up's,
    /// whose own the hot seat leaves to MCL-Q75).
    pub(super) fn deadline(&self, told: &Told) -> Option<u32> {
        let clock = self.clock.as_ref()?;
        let at = self.now.saturating_add(self.rules.clock.deadline);
        if *told == Told::Expired {
            return Some(at);
        }
        let timer = self.turn_timer.as_ref()?;
        let end = timer.start + i64::from(clock.limit);
        let late = !clock.hosted && !matches!(told, Told::Level(_)) && i64::from(at) > end;
        late.then_some(at)
    }

    /// The open box timed out at `t`: its deadline come (`dialog_default_proc` 0x4F0FC0: 9999,
    /// its caller's else), or the turn's time out with no deadline (`message_box_proc` 0x4F09D3:
    /// by its style); the button its close takes ([`timed_out`]). In play the host's expiry closes
    /// any box but the expiry's at once, as the session's system answered it (none asked of the
    /// host again), and opens the expiry's. Whether the loop's turn was taken.
    ///
    /// # Errors
    /// As [`Loop::close_box`]'s.
    pub(super) fn box_timeout(&mut self, t: u32) -> Result<bool, Unplayable> {
        let (Some(boxed), Some(clock)) = (&self.boxed, &self.clock) else {
            return Ok(false);
        };
        if boxed.told == Told::Expired || clock.hosted {
            let hosted_out = clock.hosted
                && boxed.told != Told::Expired
                && self.turn_timer.is_some()
                && self.expired_at.is_some_and(|at| t >= at);
            if hosted_out {
                self.dismiss_box();
                return Ok(self.expires(t));
            }
            if boxed.deadline.is_some_and(|d| t >= d) {
                return self.close_box(t, timed_out(&boxed.open)).map(|()| true);
            }
            return Ok(false);
        }
        let over = self
            .turn_timer
            .as_ref()
            .is_some_and(|timer| i64::from(t) - timer.start > i64::from(clock.limit));
        let due = match boxed.deadline {
            Some(d) => t >= d,
            None => over && !matches!(boxed.told, Told::Level(_)),
        };
        if !due {
            return Ok(false);
        }
        let id = timed_out(&boxed.open);
        self.close_box(t, id).map(|()| true)
    }

    /// The open box and those waiting gone, nothing they do done (play: the host's session
    /// answered them at the turn's end).
    fn dismiss_box(&mut self) {
        let before = [self.cursor.kind, self.cursor.frame];
        [self.cursor.kind, self.cursor.frame] = self.boxed.take().map_or(before, |b| b.cursor);
        self.drawn.message = None;
        self.waiting.clear();
    }

    /// The host's session ended the turn by its time at `t` (E24 M5 T5 part five: play's system
    /// actor): the expiry's box at the adventure manager's next message from then (the first
    /// expiry kept).
    pub fn expire(&mut self, t: u32) {
        self.expired_at.get_or_insert(t);
    }

    /// The screen chat's lines now, oldest first (play: carried over into the next human's view,
    /// MCL-Q70).
    #[must_use]
    pub fn chat(&self) -> Vec<ChatLine> {
        self.chat.lines()
    }

    /// The millisecond the turn's timer was started, while it runs (play: the host's clock, held
    /// since the turn passed, runs again from then).
    #[must_use]
    pub fn timer_at(&self) -> Option<u32> {
        self.turn_timer.map(|timer| timer.at)
    }
}

/// The button a box's timing out takes (survey §5: a question's No; an OK box's OK;
/// *provisional*: a choice's random picture, styles 7 and 10, not built: its Cancel, else its OK).
pub(super) fn timed_out(open: &Open) -> u16 {
    let dialog = &open.message.dialog;
    [boxes::NO, boxes::CANCEL]
        .into_iter()
        .find(|&id| dialog.item(id).is_some())
        .unwrap_or(boxes::OK)
}
