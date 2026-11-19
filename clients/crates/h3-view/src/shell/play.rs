//! Play's start as far as the client goes before E24 M5 hosts the game (E27 M3 T6;
//! `docs/epics/E27-design.md` §3.17, §32; CLT-Q135 (a), decision 44): the loading screen, its bar
//! filled by the client's own steps, the fade out at play's start, then the client's stub (a
//! notice over the main menu, its OK the main menu).
//!
//! - **The loading screen** (`loading_screen_open` 0x4ED350, from Begin's tail 0x58C255 and again
//!   from the adventure manager's start 0x407517): `loadbar.pcx` copied whole to (0, 0), then the
//!   bar's pieces drawn so far, frames 0 to count − 1 of `loadprog.def` by the plain drawer at
//!   (395 + 18 *i*, 548). Its first opening loads both and sets the count 0; play frees them
//!   (`game_main` 0x4EFDBA), so every begin starts empty.
//! - **The bar's steps** (`load_progress_step` 0x4ED2A0, 39 callers): the count one more, at most
//!   20, the pieces drawn again. The original steps through the begin (`new_game_start` 4 times,
//!   the map's load and the adventure manager's start: more than 20), so the bar shows full
//!   before play; it fills in real time, the clock not read (the harness sees it full only). The
//!   client steps it as its own work goes ([`Play::step`]: the adapter's), then fills it
//!   ([`Play::loaded`]).
//! - **The fade** (the adventure manager's start 0x407526: `window_manager_state` with 1, 4, 1 →
//!   `screen_fade_out` 0x6030E0, §10): the loading screen as it shows, the mouse hidden; each
//!   channel shifted down 0, 1 and 2 bits, each step 50 ms of the clock after the last began
//!   ([`super::start::FADE_STEP`]); then black, where the original fades the map in (0x6032E0).
//!   On the harness's frozen clock: the steps at Begin's release, + 100 ms (the input's step) and
//!   + 150 (`adv-begin`'s `bar` clip: 1 100 and 1 150).
//! - **The stub** (CLT-STUB-13, decision 44): at the black the client's notice over the main menu
//!   ([`super::menu::STUBS`]: `shell.stub.play`, or the adapter's refusal when the engine refused
//!   the setup, [`Play::refused`]); its OK ends play with the main menu. A replay stops under it
//!   ([`Play::noticing`]).
//!
//! The busy cursor of Begin's tail (0x58C21C) is not drawn: the loading screen shows no cursor
//! (*provisional*: no recording shows the screen before the fade, whose mouse is hidden).

use alloc::string::String;
use alloc::vec::Vec;

use h3_ui::ui::Draw;
use serde::Deserialize;

use super::keys::{Command, Context};
use super::menu::{Did, Hovers, Menu};
use super::start::FADE_STEP;
use crate::message::MessageBox;

/// The bar's most pieces (`load_progress_step`'s 0x14).
pub const STEPS: u8 = 20;

/// The loading screen's look (the shell's layout `[loading]`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoadingLook {
    /// The picture copied whole to (0, 0): `loadbar.pcx`.
    pub background: String,
    /// The bar's pieces: `loadprog.def`, frame *i* the *i*-th piece.
    pub bar: String,
    /// The first piece's corner: (395, 548).
    pub at: [i32; 2],
    /// From a piece to the next: 18.
    pub step: i32,
}

/// Where play's start is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// The bar stepping.
    Loading,
    /// The fade's step showing (its channels shifted down so many bits), begun at a time.
    Fading {
        /// The shift: 0 to 2.
        shift: u8,
        /// When it began.
        at: u32,
    },
    /// The client's notice over the main menu.
    Notice,
}

/// Play's start: the loading screen, then the stub's menu under its notice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Play {
    count: u8,
    state: State,
    /// The main menu under the stub's notice, shown at the fade's black.
    then: Menu,
}

impl Play {
    /// The loading screen opened (its bar empty), `then` the stub's menu under its notice.
    #[must_use]
    pub const fn new(then: Menu) -> Self {
        Self {
            count: 0,
            state: State::Loading,
            then,
        }
    }

    /// A step of the bar (`load_progress_step`): one piece more, at most [`STEPS`]; none once
    /// loaded.
    pub fn step(&mut self) {
        if self.state == State::Loading {
            self.count = (self.count + 1).min(STEPS);
        }
    }

    /// The load done at `now`: the bar's last steps (full), the fade begun (its first step, the
    /// screen as it is); nothing once loaded.
    pub fn loaded(&mut self, now: u32) {
        if self.state == State::Loading {
            self.count = STEPS;
            self.state = State::Fading { shift: 0, at: now };
        }
    }

    /// The engine refused the setup (the adapter's [`MessageBox`] for it): the notice shows it in
    /// place of the stub's.
    pub fn refused(&mut self, notice: MessageBox) {
        let screen = self.then.screen();
        let menu = self.then.clone();
        self.then = menu.stub(screen, notice);
    }

    /// The shell's clock is `now`: the fade's steps due run, each 50 ms after the last began (the
    /// first reading at or past it); after the third the black and the notice. Whether what shows
    /// changed.
    pub fn clock(&mut self, now: u32) -> bool {
        let before = self.state;
        while let State::Fading { shift, at } = self.state {
            if now.wrapping_sub(at) as i32 >= FADE_STEP as i32 {
                self.state = if shift < 2 {
                    State::Fading {
                        shift: shift + 1,
                        at: now,
                    }
                } else {
                    State::Notice
                };
            } else {
                break;
            }
        }
        self.state != before
    }

    /// The pieces of the bar drawn.
    #[must_use]
    pub const fn count(&self) -> u8 {
        self.count
    }

    /// The fade's shift showing (0 while loading or under the notice).
    #[must_use]
    pub const fn shift(&self) -> u8 {
        match self.state {
            State::Fading { shift, .. } => shift,
            _ => 0,
        }
    }

    /// Whether the client's notice shows (the fade's black past).
    #[must_use]
    pub const fn noticing(&self) -> bool {
        matches!(self.state, State::Notice)
    }

    /// The stub's menu under its notice.
    #[must_use]
    pub const fn menu(&self) -> &Menu {
        &self.then
    }

    /// The loading screen's draws over its background (`look`'s): the bar's pieces so far.
    #[must_use]
    pub fn draws(&self, look: &LoadingLook) -> Vec<Draw> {
        (0..self.count)
            .map(|i| Draw::Button {
                def: look.bar.clone(),
                frame: u32::from(i),
                x: look.at[0] + look.step * i32::from(i),
                y: look.at[1],
                mirrored: false,
                player: None,
            })
            .collect()
    }

    /// The menus' last items: the menu's.
    #[must_use]
    pub fn hovers(&self) -> Hovers {
        self.then.hovers()
    }

    /// Where it takes its keys: the notice's (the keys go nowhere before it shows).
    #[must_use]
    pub const fn context(&self) -> Context {
        self.then.context()
    }

    /// The menu under the notice, once it shows.
    const fn taking(&mut self) -> Option<&mut Menu> {
        if self.noticing() {
            Some(&mut self.then)
        } else {
            None
        }
    }

    /// The mouse moved to `point`.
    pub fn moved(&mut self, point: [i32; 2]) -> Did {
        self.taking().map_or_else(Did::default, |m| m.moved(point))
    }

    /// The left button pressed at `point`.
    pub fn press(&mut self, point: [i32; 2]) -> Did {
        self.taking().map_or_else(Did::default, |m| m.press(point))
    }

    /// The left button released.
    pub fn release(&mut self) -> Did {
        self.taking().map_or_else(Did::default, Menu::release)
    }

    /// The right button pressed at `point`.
    pub fn right(&mut self, point: [i32; 2]) -> Did {
        self.taking().map_or_else(Did::default, |m| m.right(point))
    }

    /// The right button released.
    pub fn right_release(&mut self) -> Did {
        self.taking().map_or_else(Did::default, Menu::right_release)
    }

    /// A key down: its command `command` (the notice's), its name `by`.
    pub fn key(&mut self, command: Option<Command>, by: &str) -> Did {
        match (self.taking(), command) {
            (Some(m), Some(c)) => m.key(c, by),
            _ => Did::default(),
        }
    }

    /// The key `by` released.
    pub fn key_up(&mut self, by: &str) -> Did {
        self.taking().map_or_else(Did::default, |m| m.key_up(by))
    }
}

#[cfg(test)]
mod tests;
