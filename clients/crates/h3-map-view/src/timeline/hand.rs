//! A hot seat's hand-over (E24 M5 T5, *provisional*; `docs/epics/E24-design.md` §5.31): a
//! human's turn of a game begun with more than one human starts covered (`next_player_turn`
//! 0x4C6FE0's human start 0x4C7710, `covered_view_new_week` 0x4CA840): the input queue emptied, the
//! new player shown, the turn's clock stopped, the arrow, the music faded and the cover's sound,
//! the view drawn unexplored from the map's corner with the date's window asked (built when
//! another shows, MCL-Q74), the lists empty, the resource bar blank but the date, the shield;
//! then the box (GENRLTXT 14 with the human's name, its crest), whose OK waits for the cover's
//! sound (`wait_for_sample` 0x59A4C0: at most [`TurnArt::wait`], none without sound); then the
//! minimap drawn over the shield, four buttons enabled, the clock started, the timed events'
//! boxes over the covered picture (MCL-Q71) and the turn's start ([`Loop::day_start`]), its
//! date's window built once more than a day's (MCL-Q72: recorded). The game's first turn
//! (a [`NextTurn::first`] made of the live part, [`super::Cover::opening`]) is covered so too, its
//! kingdom and selection kept.
//!
//! [`TurnArt::wait`]: crate::turn::TurnArt::wait

use alloc::sync::Arc;

use crate::hover::Hint;
use crate::live::{Covered, Selected, Status};

use super::{
    Action, CURSOR_TYPE, Choice, DEFAULT_CURSOR, Handing, Loop, NextTurn, Told, Unplayable,
    VecDeque,
};

impl Loop {
    /// Whether `next` starts covered: a hot seat's human's turn ([`super::Start::seat`],
    /// [`NextTurn::cover`]).
    pub(super) fn covers(&self, next: &NextTurn) -> bool {
        self.seat.is_some() && next.cover.is_some()
    }

    /// The hand-over to `next`'s human at `t` (design §5.31's order: 0x4C7710 to 0x4C781E), its
    /// turn's selection `choice` kept for its start. A turn after another's takes the new
    /// player's kingdom with no selection yet (its start chooses), the colour shown, the four
    /// buttons a computer's turn disabled ([`super::Cover::computers`]); the game's start keeps
    /// its own ([`super::Cover::opening`]).
    pub(super) fn hand_over(&mut self, t: u32, next: NextTurn, choice: Choice) {
        let cover = next.cover.clone().unwrap_or_default();
        if !cover.opening {
            let mut kingdom = next.turn.kingdom.clone();
            kingdom.selected = Selected::None;
            kingdom.selection = None;
            kingdom.status = self.kingdom.status.take();
            kingdom.realm = self.kingdom.realm.take();
            self.kingdom = kingdom;
            self.player = next.turn.player;
            (self.hero, self.index, self.selected_id) = (None, None, None);
            self.drawn.computer = cover.computers;
        }
        // (2) the input queue emptied; (3) the new player shown; the turn's clock stopped at the
        // turn's end (0x4C708B).
        self.flush = true;
        self.shown = next.turn.player;
        self.timer = false;
        // The cover (0x4CA87D to 0x4CA962): the arrow, the music faded, its sound, the view
        // drawn covered with the date's window asked (built when another kind shows: the new
        // date's, at its first frame with its sound), the status bar's hint gone.
        self.set_cursor(DEFAULT_CURSOR, 0);
        if let Some(sound) = &mut self.sound {
            sound.fade(t);
            sound.cover(t, &self.rules.turn.cover);
        }
        let sample_end = self
            .seat
            .as_ref()
            .and_then(|s| s.sample)
            .map(|length| t.saturating_add(length));
        self.covered = Some(Covered {
            frame: self.framed,
            date: self.kingdom.date,
            quests: self.kingdom.quests,
            resources: None,
            started: false,
        });
        self.seat_shown = cover.seat.or(self.seat_shown);
        self.drawn.hint = Hint::Empty;
        self.asked = 1;
        self.end = t.saturating_add(self.rules.turn.covered);
        // The window built now names the new date; one kept steps on with its own (MCL-Q74).
        let builds = !matches!(self.kingdom.status, Some(Status::Date(_)));
        self.draw(t);
        if builds {
            Arc::make_mut(&mut self.drawn.kingdom).date = self.kingdom.date;
        }
        let message = self
            .seat
            .as_ref()
            .and_then(|s| s.boxes.iter().find(|(c, _)| *c == self.shown))
            .map(|(_, m)| Arc::clone(m));
        self.handing = Some(Handing {
            next,
            choice,
            sample_end,
            until: None,
            queued: alloc::vec::Vec::new(),
            events: cover.events.into_iter().collect::<VecDeque<_>>(),
        });
        match message {
            Some(message) => self.open_box(message, Told::Cover),
            // No box laid out for the colour (*provisional*): the start at once.
            None => {
                if let Some(handing) = &mut self.handing {
                    handing.until = Some(t);
                }
            }
        }
    }

    /// The cover's box closed by its OK at `t` (0x4CA994): the start once the cover's sound ends,
    /// at most [`crate::turn::TurnArt::wait`] from now (an OK in its last part waits for its end,
    /// one after it none); the turn's start at once without a wait.
    pub(super) fn covered_ok(&mut self, t: u32) -> Result<(), Unplayable> {
        let wait = self.rules.turn.wait;
        let until = match self.handing.as_ref().and_then(|h| h.sample_end) {
            Some(end) if end > t => t.saturating_add((end - t).min(wait)),
            _ => t,
        };
        if let Some(handing) = &mut self.handing {
            handing.until = Some(until);
        }
        if until <= t {
            return self.hand_start(t);
        }
        Ok(())
    }

    /// A hand-over's wait at `t` (`wait_for_sample`: the messages pumped, the tick run, nothing
    /// drawn): whether it holds the loop's turn; at its end the start, the loop's turn then run.
    pub(super) fn waiting(&mut self, t: u32) -> Result<bool, Unplayable> {
        let Some(until) = self.handing.as_ref().and_then(|h| h.until) else {
            return Ok(false);
        };
        if t < until {
            return Ok(true);
        }
        self.hand_start(t)?;
        Ok(false)
    }

    /// An action given during a hand-over's wait: the mouse's point read at once, the message
    /// kept for the start. Whether it was.
    pub(super) fn wait_input(&mut self, action: Action) -> bool {
        let Some(handing) = self.handing.as_mut().filter(|h| h.until.is_some()) else {
            return false;
        };
        if let Some(at) = action.point() {
            self.pointer = at;
        }
        handing.queued.push(action);
        true
    }

    /// The start after the wait (0x4C7862 to 0x4C78F0): the minimap drawn over the shield (the
    /// view's corner as it stands), sleep, move hero, cast and End Turn enabled, the clock
    /// started; then the timed events and the turn's start ([`Loop::hand_events`]).
    fn hand_start(&mut self, t: u32) -> Result<(), Unplayable> {
        if let Some(handing) = &mut self.handing {
            handing.until = None;
        }
        if let Some(covered) = &mut self.covered {
            covered.started = true;
        }
        self.drawn.computer = false;
        self.draw_minimap();
        self.drawn.cover = self.covered;
        self.timer = self.seat.as_ref().is_some_and(|s| s.timed);
        self.hand_events(t)
    }

    /// The next timed event's box over the covered picture (MCL-Q71: its resources added and the
    /// resource bar drawn with them as it opens), or after the last the turn's start: the cover
    /// gone, the turn's end its human left, the start's date window built once more (MCL-Q72),
    /// [`Loop::day_start`], then the actions given during the wait.
    pub(super) fn hand_events(&mut self, t: u32) -> Result<(), Unplayable> {
        let Some(mut handing) = self.handing.take() else {
            self.idle_message(t);
            return Ok(());
        };
        if let Some((message, resources)) = handing.events.pop_front() {
            for (have, add) in self.kingdom.resources.iter_mut().zip(resources) {
                *have = have.saturating_add(add);
            }
            if let Some(covered) = &mut self.covered {
                covered.resources = Some(self.kingdom.resources);
            }
            self.drawn.cover = self.covered;
            self.handing = Some(handing);
            self.open_box(message, Told::Event);
            return Ok(());
        }
        let Handing {
            mut next,
            choice,
            queued,
            ..
        } = handing;
        next.turn.kingdom.resources = self.kingdom.resources;
        self.covered = None;
        // The adventure's cursor again before the start's boxes (recorded: `seat-new-week`'s
        // week's box gives it back at its close; `adventure_mouse_again` 0x419570 last).
        self.set_cursor(CURSOR_TYPE, self.rules.arrow);
        let ended = self.ends.remove(&next.turn.player).unwrap_or_default();
        // The start's full redraw builds the date's window once more than a day's (MCL-Q72:
        // recorded, three `newday.wav` in the start's millisecond; not read).
        self.choose(t, true);
        self.day_start(t, next, choice, ended);
        for action in queued {
            self.act(t, action, false)?;
        }
        Ok(())
    }
}
