//! The map window's commands (E24 M2 T4a, *provisional*; the user, 2026-10-26: no key hard-coded
//! in the client), whose keys go through the common table (`h3_ui::keys`, E24 M3 T1a: the chords,
//! a user's file, the lookup; its `[map]` table since T1b). Their defaults are the original's keys
//! on the adventure map (`docs/epics/E24-design.md` §3.4: Ctrl and a direction key scroll the view,
//! a direction key alone steps the selected hero; the keypad's eight give the same scan codes in
//! either NumLock state, §3.10) and the window's own (`h3_ui::keys::Window`, the battle's too: the
//! music, the effects, pause, a frame on, restart, vsync, the cursor's kind, fullscreen, quit),
//! which take the function keys and Esc, keys the original's map leaves to nothing else.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use h3_ui::keys::{self, Chord, Window};

use crate::timeline::{Action, Direction};

/// The map window's table of keys.
pub type Keys = keys::Keys<Command>;

/// What a key does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Command {
    /// The view one scroll step that way (the original's Ctrl and a direction key).
    Scroll(Direction),
    /// The selected hero one step that way (the original's direction key alone).
    Hero(Direction),
    /// The window's own (the music and the effects since E24 M2 T5, the cursor's kind since T4b).
    Window(Window),
}

/// The original's direction keys by the adapter's names, north clockwise to north-west: the scan
/// codes 72, 73, 77, 81, 80, 79, 75, 71 (`adventure_key_down` 0x408C40).
const DIRECTION_KEYS: [&str; 8] = [
    "Up", "PageUp", "Right", "PageDown", "Down", "End", "Left", "Home",
];

/// The keypad's keys giving the same scan codes, in the same order: the message's code is
/// lParam's bits 16 to 23 (`input_key_message` 0x4EC0E0), the keypad's in either NumLock state.
const KEYPAD_KEYS: [&str; 8] = [
    "Keypad8", "Keypad9", "Keypad6", "Keypad3", "Keypad2", "Keypad1", "Keypad4", "Keypad7",
];

impl Command {
    /// The map's action (the timeline's) a scroll or a hero's step gives; none for the window's
    /// own commands.
    #[must_use]
    pub fn action(self) -> Option<Action> {
        match self {
            Self::Scroll(d) => Some(Action::Scroll(d)),
            Self::Hero(d) => Some(Action::Step(d)),
            _ => None,
        }
    }
}

impl keys::Command for Command {
    /// The map takes every key alike (no dialog over it yet).
    type Context = ();
    const TABLE: &'static str = "map";

    /// Every command, in the table's order: the scroll's eight directions, the hero's eight, the
    /// window's own.
    fn all() -> Vec<Self> {
        let mut all: Vec<Self> = Direction::ALL.into_iter().map(Self::Scroll).collect();
        all.extend(Direction::ALL.into_iter().map(Self::Hero));
        all.extend(Window::ALL.map(Self::Window));
        all
    }

    /// Its name in a user's file: `scroll-north`, `hero-south-west`, `pause`, …
    fn name(self) -> String {
        match self {
            Self::Scroll(d) => format!("scroll-{}", direction_name(d)),
            Self::Hero(d) => format!("hero-{}", direction_name(d)),
            Self::Window(w) => w.name().into(),
        }
    }

    /// The original's keys on the map (a direction's key and the keypad's, with Ctrl for the
    /// scroll) and the window's own.
    fn defaults() -> Vec<(Self, Vec<Chord>)> {
        let mut defaults = Vec::new();
        for ((d, key), pad) in Direction::ALL
            .into_iter()
            .zip(DIRECTION_KEYS)
            .zip(KEYPAD_KEYS)
        {
            let chords = |ctrl| Vec::from([key, pad].map(|k| Chord::new(ctrl, k)));
            defaults.push((Self::Scroll(d), chords(true)));
            defaults.push((Self::Hero(d), chords(false)));
        }
        for w in Window::ALL {
            defaults.push((Self::Window(w), Vec::from([Chord::new(false, w.key())])));
        }
        defaults
    }

    /// Every key in the one context.
    fn context(self) -> Option<()> {
        None
    }
}

/// A direction's part of a command's name.
fn direction_name(d: Direction) -> &'static str {
    match d {
        Direction::North => "north",
        Direction::NorthEast => "north-east",
        Direction::East => "east",
        Direction::SouthEast => "south-east",
        Direction::South => "south",
        Direction::SouthWest => "south-west",
        Direction::West => "west",
        Direction::NorthWest => "north-west",
    }
}

#[cfg(test)]
mod tests;
