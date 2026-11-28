//! The turn's clock as shown (E24 M5 T5 part five, *provisional*; `docs/epics/E24-design.md`
//! §5.32): a timed turn's warnings (`turn_timer_poll` 0x557AF0 from the tick every 60 ms: their
//! schedule from the turn's start and limit, the text by the time left), the screen chat they go
//! into (`screen_chat` 0x69D7B0: its ring of entries, each line's life from its first view draw,
//! `chat_expire` 0x553DF0) and the expiry's box (GENRLTXT 202). The timeline (`crate::timeline`)
//! runs them; the adapter lays the box out and the scene draws the chat's text item (the
//! adventure dialog's 37) with the texts given here. *Not built:* a player's chat line (the
//! network's typing, §4.1: its 10 s more), the chat frozen by a battle (`chat_freeze` 0x554050),
//! the wrapped lines' count of the text built (`chat_text_build` 0x553EE0: each warning one line).

use alloc::collections::VecDeque;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use h3_ui::text::{Args, Catalogue};
use serde::Deserialize;

/// The clock's numbers and sound (the tables' `[map.clock]`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClockArt {
    /// The tick's period for the warnings' poll, ms (60: `tick_60ms_time` 0x6989AC, set to now +
    /// 60 when it runs).
    pub tick: u32,
    /// A chat line's life from its first view draw, ms (20 000: `chat_expire` 0x553DF0, one line
    /// removed a draw once the oldest is older).
    pub life: u32,
    /// The chat's entries (20: its ring, a 21st dropping the oldest, `chat_add_line` 0x553877).
    pub entries: usize,
    /// A warning's sound (`timeover.wav`, the chat's +0x3C: `chat_sounds_load` 0x5537B0).
    pub sound: String,
    /// A box's deadline when opened with less than it left in the turn, ms (15 000:
    /// `message_box_build` 0x4F6990, `turn_timer_ends_within(15 000)`; the expiry's box so).
    pub deadline: u32,
}

impl Default for ClockArt {
    /// No clock's numbers: a tick every millisecond, lines that live for ever, no sound.
    fn default() -> Self {
        Self {
            tick: 1,
            life: u32::MAX,
            entries: 0,
            sound: String::new(),
            deadline: 0,
        }
    }
}

/// The string catalogue's keys the clock shows (`clients/text/keys.toml`).
pub const KEYS: [&str; 5] = [
    "map.clock.system",
    "map.clock.seconds",
    "map.clock.minute",
    "map.clock.minutes",
    "map.clock.expired",
];

/// A warning's text by the time left (`turn_timer_poll`'s GENRLTXT 627 to 630).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Warning {
    /// Whole seconds (627 for one, 628 `%d` else).
    Seconds(u32),
    /// A minute (629).
    Minute,
    /// Minutes in tenths (630, `%0.1f`).
    Minutes(u32),
}

/// The first warning's interval from a turn's start (`turn_timer_start` 0x557D90): half of a limit
/// of a minute or less, else a quarter of it.
#[must_use]
pub const fn first(limit: u32) -> u32 {
    if limit <= 60_000 {
        limit / 2
    } else {
        limit / 4
    }
}

/// The warning `left` ms before the end shows (0x557B6F to 0x557C5B): above a minute the minutes
/// (629 below 1.2 of them, *provisional*: "0.8 to 1.2 minutes", the f32's test not built; no
/// schedule of the original's shows it, survey §3), else the whole seconds, 29 shown as 30.
#[must_use]
pub fn warning(left: u32) -> Warning {
    if left > 60_000 {
        if left < 72_000 {
            Warning::Minute
        } else {
            Warning::Minutes(tenths(left))
        }
    } else {
        match left / 1_000 {
            29 => Warning::Seconds(30),
            s => Warning::Seconds(s),
        }
    }
}

/// The next warning's interval after one `left` ms before the end of a turn of `limit` ms
/// (0x557C9A to 0x557CF7): half the limit while more than half is left and the limit is above two
/// minutes, else to a minute left, else to 10 s left, then a second while 2 s or more are left;
/// 0: none.
#[must_use]
pub const fn next(left: u32, limit: u32) -> u32 {
    if left > limit / 2 && limit > 120_000 {
        limit / 2
    } else if left > 60_000 {
        left - 60_000
    } else if left > 10_000 {
        left - 10_000
    } else if left >= 2_000 {
        1_000
    } else {
        0
    }
}

/// `left` ÷ 60 000 as the executable prints it (GENRLTXT 630's `%0.1f` of an f32), in tenths:
/// the quotient's nearest f32, then its exact value rounded half away from zero to a tenth (the C
/// library's), in integers (no float in the crate). The quotient never lies halfway between two
/// f32s: 60 000 holds 2^5, the mantissa's shift 7 or more.
#[must_use]
pub fn tenths(left: u32) -> u32 {
    const MINUTE: u128 = 60_000;
    if left == 0 {
        return 0;
    }
    let n = u128::from(left);
    // The shift that brings the quotient's integer part to 24 bits (2^23 to 2^24): the f32's
    // mantissa, its value m / 2^s.
    let mut s = 0_u32;
    while (n << s) / MINUTE < 1 << 23 {
        s += 1;
    }
    let (m, rem) = ((n << s) / MINUTE, (n << s) % MINUTE);
    let m = if rem * 2 > MINUTE { m + 1 } else { m };
    // Ten times m / 2^s, half up (s ≥ 7: a quotient below 2^17).
    let half = 1_u128 << s.saturating_sub(1);
    u32::try_from((10 * m + half) >> s).unwrap_or(u32::MAX)
}

/// The warning's chat line (0x5539F9 to 0x553A35): GENRLTXT 54 and the warning's text joined with
/// nothing between ("%s%s"), the numbers as shown.
#[must_use]
pub fn text(texts: &Catalogue, warning: Warning) -> String {
    let system = texts.text("map.clock.system", &Args::new());
    let said = match warning {
        Warning::Seconds(s) => texts.text("map.clock.seconds", &Args::new().count(i64::from(s))),
        Warning::Minute => texts.text("map.clock.minute", &Args::new()),
        Warning::Minutes(t) => texts.text(
            "map.clock.minutes",
            &Args::new().text("minutes", &format!("{}.{}", t / 10, t % 10)),
        ),
    };
    system + &said
}

/// The expiry's box's text (GENRLTXT 202, the adventure manager's 0x4087F3).
#[must_use]
pub fn expired(texts: &Catalogue) -> String {
    texts.text("map.clock.expired", &Args::new())
}

/// A line of the screen chat: a warning (a system line, +0x84 set) and the millisecond of its
/// first view draw (+0x80, 0 in memory until then: none), which may lie before the loop's 0 (play:
/// a line carried over from the view before, MCL-Q70).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChatLine {
    /// The warning.
    pub warning: Warning,
    /// Its first draw.
    pub drawn: Option<i64>,
}

/// The screen chat (`screen_chat` 0x69D7B0, E24 M5 T5 part five): its lines oldest first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Chat {
    lines: VecDeque<ChatLine>,
}

impl Chat {
    /// The chat with `lines`, oldest first.
    #[must_use]
    pub fn of(lines: &[ChatLine]) -> Self {
        Self {
            lines: lines.iter().copied().collect(),
        }
    }

    /// Its lines, oldest first.
    #[must_use]
    pub fn lines(&self) -> Vec<ChatLine> {
        self.lines.iter().copied().collect()
    }

    /// The warnings its lines show, oldest first.
    #[must_use]
    pub fn shown(&self) -> Vec<Warning> {
        self.lines.iter().map(|l| l.warning).collect()
    }

    /// A warning's line added (`chat_add_line` 0x553840), not drawn yet; past the ring's
    /// `entries` the oldest dropped.
    pub fn add(&mut self, warning: Warning, entries: usize) {
        self.lines.push_back(ChatLine {
            warning,
            drawn: None,
        });
        while self.lines.len() > entries {
            self.lines.pop_front();
        }
    }

    /// A view's draw at `t` (`chat_draw` 0x553D00 from `view_draw_chat` 0x40F833): each line not
    /// drawn yet stamped now, then the oldest removed once first drawn more than `life` ms before
    /// (one a draw). Whether a line changed.
    pub fn draw(&mut self, t: i64, life: u32) -> bool {
        let mut changed = false;
        for line in self.lines.iter_mut().filter(|l| l.drawn.is_none()) {
            line.drawn = Some(t);
            changed = true;
        }
        if self
            .lines
            .front()
            .and_then(|l| l.drawn)
            .is_some_and(|d| t.saturating_sub(d) > i64::from(life))
        {
            self.lines.pop_front();
            changed = true;
        }
        changed
    }
}

#[cfg(test)]
mod tests;
