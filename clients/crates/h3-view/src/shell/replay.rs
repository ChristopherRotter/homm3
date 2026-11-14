//! A capture recipe's steps played on the shell's own clock (E27 M2 T4; `clients/capture/
//! h3shell.py drawn`, the runbook's "The shell's screens"): the harness's step words read as
//! `h3shell.py` reads them, and played as its runs pass the original's frozen clock: the time from
//! the first screen (the original's from the game's first read; the main menu shows at 0 ms with
//! `Show Intro` off), an input then [`INPUT_STEP`] (a recipe's `input_step`), `until` and a clip
//! in steps of [`UNTIL_STEP`]. Each shot and each clip's frame is handed to the caller, which
//! draws the shell as it is then ([`Replay::screen`], [`Replay::pointer`], [`Replay::time`]).
//!
//! The start (E27 M2 T5a) plays on the same clock: each step's reading of it runs the start's
//! timing ([`super::start::Start`]), whose ops the caller carries out ([`Replay::ops`]); an input
//! that stops a clip is the start's ([`Start::skip`]); its end shows the main menu.
//!
//! What only the harness can do stops the replay ([`Stop`]): a step on the original's memory or
//! its probes (`poke`, `press`, `point`, `until-button`, `games-time`, `freeze`), a screen name the
//! viewer does not know, a screen not built yet ([`Screen::built`]: nothing on it changes by
//! itself, and its shots would be a stand-in's), a press or a key on a screen built only at rest
//! (the main menu until T6: the pointer's moves and the releases with nothing held are taken,
//! its hover not drawn). The real-time
//! pauses (`settle`, `wait`) pass no time on the frozen clock.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::start::{Op, Start};
use super::{Built, Ended, Game, Screen, Shell};

/// The clock's step after each input (h3shell's `INPUT_STEP`: the game draws its cursor at the
/// pointer only on a later tick).
pub const INPUT_STEP: u32 = 100;
/// The clock's step while waiting for a screen, and a clip's default step (`UNTIL_STEP`).
pub const UNTIL_STEP: u32 = 10;
/// How long `until` and `clip-until` wait by default (`UNTIL_LIMIT`).
pub const UNTIL_LIMIT: u32 = 60_000;

/// A recipe's step (h3shell's `STEPS`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// `until:<screen>[|<screen>...][:<limit>]`: the clock stepped until a screen shows.
    Until {
        /// The screens' names (the harness's probes').
        screens: Vec<String>,
        /// The most milliseconds waited.
        limit: u32,
    },
    /// `check:<screen>`: the screen shows now.
    Check(String),
    /// `step:<ms>[:<by>]`: the clock moved on, `by` at a time.
    Step {
        /// The milliseconds.
        ms: u32,
        /// The step (`ms` when not given or 0).
        by: u32,
    },
    /// An input, then the input step.
    Input(Input),
    /// `shot:<name>`.
    Shot(String),
    /// `clip:<name>:<ms>[:<step>]` or `clip-until:<name>:<screen>[:<step>][:<limit>]`: the
    /// screen kept at each step.
    Clip {
        /// The clip's name.
        name: String,
        /// When it ends.
        end: ClipEnd,
        /// The clock's step.
        step: u32,
    },
    /// A real-time pause (`settle`, `wait`): no time passes on the frozen clock.
    Pause,
    /// A step only the harness takes (its verb).
    Harness(String),
}

/// When a clip ends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipEnd {
    /// After so many milliseconds.
    After(u32),
    /// Once a screen shows (kept), at most `limit` milliseconds.
    Shown {
        /// The screens' names.
        screens: Vec<String>,
        /// The most milliseconds.
        limit: u32,
    },
}

/// An input (the harness's mouse and keys).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    /// `move:<x>,<y>`.
    Move([i32; 2]),
    /// `click:<x>,<y>` (and `click-wait`'s click: its wait is real time).
    Click([i32; 2]),
    /// `down:<x>,<y>`: the left button pressed there and held.
    Down([i32; 2]),
    /// `up`.
    Up,
    /// `right:<x>,<y>`: the right button pressed there and held.
    Right([i32; 2]),
    /// `right-up`.
    RightUp,
    /// `key:<name>` (xdotool's names).
    Key(String),
    /// `key-down:<name>`.
    KeyDown(String),
    /// `key-up:<name>`.
    KeyUp(String),
    /// `type:<text>`.
    Type(String),
}

/// Reads a step as `h3shell.py` does (`<verb>[:<arguments>]`).
///
/// # Errors
/// An unknown verb, or arguments that are not the verb's.
pub fn parse(step: &str) -> Result<Step, String> {
    let (verb, arg) = step.split_once(':').unwrap_or((step, ""));
    let bad = || format!("the step `{step}`: not `{verb}`'s arguments");
    let number = |s: &str| s.parse::<u32>().map_err(|_| bad());
    let point = |s: &str| -> Result<[i32; 2], String> {
        let (x, y) = s.split_once(',').ok_or_else(bad)?;
        Ok([x.parse().map_err(|_| bad())?, y.parse().map_err(|_| bad())?])
    };
    let screens = |s: &str| -> Result<Vec<String>, String> {
        if s.is_empty() {
            return Err(bad());
        }
        Ok(s.split('|').map(String::from).collect())
    };
    let name = |s: &str| -> Result<String, String> {
        if s.is_empty() {
            Err(bad())
        } else {
            Ok(s.to_string())
        }
    };
    let parts: Vec<&str> = arg.split(':').collect();
    let at = |i: usize| parts.get(i).copied().unwrap_or("");
    Ok(match verb {
        "until" => Step::Until {
            screens: screens(at(0))?,
            limit: optional(at(1), UNTIL_LIMIT, number)?,
        },
        "check" => Step::Check(name(arg)?),
        "step" => {
            let ms = number(at(0))?;
            let by = optional(at(1), ms, number)?;
            Step::Step {
                ms,
                by: if by == 0 { ms } else { by },
            }
        }
        "move" => Step::Input(Input::Move(point(arg)?)),
        "click" => Step::Input(Input::Click(point(arg)?)),
        "click-wait" => Step::Input(Input::Click(point(at(0))?)),
        "down" => Step::Input(Input::Down(point(arg)?)),
        "up" => Step::Input(Input::Up),
        "right" => Step::Input(Input::Right(point(arg)?)),
        "right-up" => Step::Input(Input::RightUp),
        "key" => Step::Input(Input::Key(name(arg)?)),
        "key-down" => Step::Input(Input::KeyDown(name(arg)?)),
        "key-up" => Step::Input(Input::KeyUp(name(arg)?)),
        "type" => Step::Input(Input::Type(name(arg)?)),
        "shot" => Step::Shot(name(arg)?),
        "clip" => Step::Clip {
            name: name(at(0))?,
            end: ClipEnd::After(number(at(1))?),
            step: clip_step(at(2), number)?,
        },
        "clip-until" => Step::Clip {
            name: name(at(0))?,
            end: ClipEnd::Shown {
                screens: screens(at(1))?,
                limit: optional(at(3), UNTIL_LIMIT, number)?,
            },
            step: clip_step(at(2), number)?,
        },
        "settle" | "wait" => Step::Pause,
        "freeze" | "press" | "point" | "until-button" | "games-time" | "poke" => {
            Step::Harness(verb.to_string())
        }
        _ => return Err(format!("the step `{step}`: an unknown verb")),
    })
}

/// An optional number: `default` when empty.
fn optional(
    s: &str,
    default: u32,
    number: impl Fn(&str) -> Result<u32, String>,
) -> Result<u32, String> {
    if s.is_empty() { Ok(default) } else { number(s) }
}

/// A clip's step: [`UNTIL_STEP`] when empty, never 0 (the clip would not end).
fn clip_step(s: &str, number: impl Fn(&str) -> Result<u32, String>) -> Result<u32, String> {
    match optional(s, UNTIL_STEP, number)? {
        0 => Err(String::from("a clip's step of 0 ms")),
        step => Ok(step),
    }
}

/// Whether `screen` is the one the harness's probes call `name`; none for a name the viewer does
/// not know.
#[must_use]
pub fn shown(screen: Screen, name: &str) -> Option<bool> {
    Some(match name {
        "main_menu" => screen == Screen::MainMenu,
        "new_game_menu" => screen == Screen::GameType(Game::New),
        "load_game_menu" => screen == Screen::GameType(Game::Load),
        "multiplayer" => matches!(screen, Screen::Multiplayer(_)),
        "scenario_list" => matches!(screen, Screen::Scenario(..)),
        "high_scores" => screen == Screen::HighScores,
        _ => return None,
    })
}

/// A moment the caller draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Take<'a> {
    /// A shot of this name.
    Shot(&'a str),
    /// A clip's frame (the caller keeps it when it changed): the clip's name, and whether it is
    /// the first.
    Clip {
        /// The clip's name.
        name: &'a str,
        /// The clip's first frame.
        first: bool,
    },
}

/// Why a replay stops before its recipe's end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stop {
    /// A step only the harness takes (its verb).
    Harness(String),
    /// A screen name the viewer does not know.
    Unknown(String),
    /// The screen on is not built yet: nothing on it changes, nothing is drawn.
    NotBuilt(Screen),
    /// A press or a key on a screen built at rest (none takes them yet: the main menu's with T6).
    AtRest(Screen),
    /// None of the screens shows (at the end of the wait, for `until` and `clip-until`).
    NotShown {
        /// The screens' names.
        screens: Vec<String>,
        /// The screen on.
        screen: Screen,
    },
}

/// A recipe's replay: the shell, the time and the pointer, the start's timing and its ops.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replay {
    shell: Shell,
    time: u32,
    pointer: [i32; 2],
    input_step: u32,
    start: Option<Start>,
    ops: Vec<Op>,
}

/// Whether an input stops a start's clip (`video_play_fullscreen`'s and `video_play_framed`'s
/// loops): a key down but F4 (scan code 0x3E), a left or a right press.
#[must_use]
pub fn skips(input: &Input) -> bool {
    match input {
        Input::Key(name) | Input::KeyDown(name) => !name.eq_ignore_ascii_case("F4"),
        Input::Type(text) => !text.is_empty(),
        Input::Click(_) | Input::Down(_) | Input::Right(_) => true,
        Input::Move(_) | Input::Up | Input::RightUp | Input::KeyUp(_) => false,
    }
}

impl Replay {
    /// A replay of `shell` from its first screen at 0 ms, the pointer at `pointer` (where the
    /// harness parks it before the game starts), each input followed by `input_step`; a shell at
    /// the start plays `start`'s clips (started at 0 ms: [`Start::new`] with its ops, given as
    /// `ops`), none ending the start at once.
    #[must_use]
    pub fn new(
        mut shell: Shell,
        pointer: [i32; 2],
        input_step: u32,
        start: Option<(Start, Vec<Op>)>,
    ) -> Self {
        let (start, ops) = match start {
            Some((start, ops)) if shell.screen() == Screen::Start => (Some(start), ops),
            _ => (None, Vec::new()),
        };
        if shell.screen() == Screen::Start && start.as_ref().is_none_or(Start::ended) {
            let _ = shell.end(Ended::Start);
        }
        Self {
            shell,
            time: 0,
            pointer,
            input_step,
            start,
            ops,
        }
    }

    /// The start's ops so far, in order (the caller carries out those it has not yet).
    #[must_use]
    pub fn ops(&self) -> &[Op] {
        &self.ops
    }

    /// The start's timing, while the start plays.
    #[must_use]
    pub const fn start(&self) -> Option<&Start> {
        self.start.as_ref()
    }

    /// The milliseconds since the first screen.
    #[must_use]
    pub const fn time(&self) -> u32 {
        self.time
    }

    /// The pointer.
    #[must_use]
    pub const fn pointer(&self) -> [i32; 2] {
        self.pointer
    }

    /// The screen on.
    #[must_use]
    pub const fn screen(&self) -> Screen {
        self.shell.screen()
    }

    /// Plays `step`, handing each moment to draw to `take` with the replay as it is then.
    /// `Ok(Some(stop))`: the recipe cannot go on in the viewer.
    ///
    /// # Errors
    /// `take`'s.
    pub fn play(
        &mut self,
        step: &Step,
        take: &mut dyn FnMut(&Self, Take<'_>) -> Result<(), String>,
    ) -> Result<Option<Stop>, String> {
        match step {
            Step::Until { screens, limit } => Ok(self.until(screens, *limit)),
            Step::Check(name) => Ok(self.known(core::slice::from_ref(name)).or_else(|| {
                (shown(self.screen(), name) != Some(true)).then(|| Stop::NotShown {
                    screens: alloc::vec![name.clone()],
                    screen: self.screen(),
                })
            })),
            Step::Step { ms, by } => {
                let mut left = *ms;
                while left > 0 {
                    self.tick(left.min(*by));
                    left = left.saturating_sub(*by);
                }
                Ok(None)
            }
            Step::Input(Input::Move(at)) => {
                self.pointer = *at;
                self.tick(self.input_step);
                Ok(None)
            }
            Step::Input(input) if self.screen() == Screen::Start => {
                if skips(input) {
                    let (time, ops) = (self.time, &mut self.ops);
                    self.start
                        .iter_mut()
                        .for_each(|start| start.skip(time, ops));
                    self.ended();
                }
                self.tick(self.input_step);
                Ok(None)
            }
            // A release with nothing held (a press would have stopped the replay before; the
            // start's skip took its own): nothing on a screen at rest.
            Step::Input(Input::Up | Input::RightUp | Input::KeyUp(_))
                if self.screen().built() == Built::AtRest =>
            {
                self.tick(self.input_step);
                Ok(None)
            }
            // A press or a key: no other screen takes them yet (T6 builds the main menu's).
            Step::Input(_) => Ok(Some(match self.screen().built() {
                Built::Not => Stop::NotBuilt(self.screen()),
                Built::AtRest | Built::Whole => Stop::AtRest(self.screen()),
            })),
            Step::Shot(name) => {
                if let Some(stop) = self.unbuilt() {
                    return Ok(Some(stop));
                }
                take(self, Take::Shot(name))?;
                Ok(None)
            }
            Step::Clip { name, end, step } => self.clip(name, end, *step, take),
            Step::Pause => Ok(None),
            Step::Harness(verb) => Ok(Some(Stop::Harness(verb.clone()))),
        }
    }

    /// The clock moved on: the start's reading of it.
    fn tick(&mut self, ms: u32) {
        self.time = self.time.saturating_add(ms);
        if let Some(start) = self.start.as_mut() {
            start.read(self.time, &mut self.ops);
        }
        self.ended();
    }

    /// The start's end taken: the main menu.
    fn ended(&mut self) {
        if self.start.as_ref().is_some_and(Start::ended) {
            self.start = None;
            let _ = self.shell.end(Ended::Start);
        }
    }

    /// A name the viewer does not know among `screens`.
    fn known(&self, screens: &[String]) -> Option<Stop> {
        screens
            .iter()
            .find(|name| shown(self.screen(), name).is_none())
            .map(|name| Stop::Unknown(name.clone()))
    }

    /// The screen on, when it is not built.
    fn unbuilt(&self) -> Option<Stop> {
        (self.screen().built() == Built::Not).then(|| Stop::NotBuilt(self.screen()))
    }

    fn any_shown(&self, screens: &[String]) -> bool {
        screens
            .iter()
            .any(|name| shown(self.screen(), name) == Some(true))
    }

    /// `until`: as the harness, checked then stepped, `limit / UNTIL_STEP + 1` times.
    fn until(&mut self, screens: &[String], limit: u32) -> Option<Stop> {
        if let Some(stop) = self.known(screens).or_else(|| self.unbuilt()) {
            return Some(stop);
        }
        for _ in 0..=limit / UNTIL_STEP {
            if self.any_shown(screens) {
                return None;
            }
            self.tick(UNTIL_STEP);
        }
        Some(Stop::NotShown {
            screens: screens.to_vec(),
            screen: self.screen(),
        })
    }

    /// A clip: its first frame now, then one each step until its end (a screen's: kept, then
    /// the end).
    fn clip(
        &mut self,
        name: &str,
        end: &ClipEnd,
        step: u32,
        take: &mut dyn FnMut(&Self, Take<'_>) -> Result<(), String>,
    ) -> Result<Option<Stop>, String> {
        let (until, screens) = match end {
            ClipEnd::After(ms) => (*ms, None),
            ClipEnd::Shown { screens, limit } => (*limit, Some(screens.as_slice())),
        };
        if let Some(stop) = screens
            .and_then(|s| self.known(s))
            .or_else(|| self.unbuilt())
        {
            return Ok(Some(stop));
        }
        take(self, Take::Clip { name, first: true })?;
        let end = self.time.saturating_add(until);
        while self.time < end {
            self.tick(step);
            take(self, Take::Clip { name, first: false })?;
            if screens.is_some_and(|s| self.any_shown(s)) {
                return Ok(None);
            }
        }
        Ok(screens.map(|s| Stop::NotShown {
            screens: s.to_vec(),
            screen: self.screen(),
        }))
    }
}

#[cfg(test)]
mod tests;
