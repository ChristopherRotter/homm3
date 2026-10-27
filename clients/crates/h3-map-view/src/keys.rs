//! The window's keys (E24 M2 T4a, *provisional*; the user, 2026-10-26: no key hard-coded in the
//! client): every key the window takes goes through this one table of commands. Its defaults are
//! the original's keys on the adventure map (`docs/epics/E24-design.md` §3.4: Ctrl and a direction
//! key scroll the view, a direction key alone steps the selected hero; the keypad's eight give the
//! same scan codes in either NumLock state, §3.10) and the window's own (the music, the effects,
//! pause, a frame on, restart, vsync, the cursor's kind, fullscreen, quit), which take the function
//! keys and Esc, keys the original's map leaves to nothing else. A user's file overrides the
//! defaults command by command ([`Keys::with_file`]). A key is named as the adapter names it
//! (`h3-sdl`'s `Key::name`: `Up`, `PageUp`, `F5`, …), `Ctrl+` holding either Control key with it;
//! the adapter checks the names ([`Keys::chords`]). A chord matches exactly: a key with Ctrl held is
//! not the key alone. M3 T0
//! moves the table into the common crate (the battle viewer's keys too); a screen to rebind them
//! comes with the options dialog.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use crate::timeline::{Action, Direction};

/// What a key does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Command {
    /// The view one scroll step that way (the original's Ctrl and a direction key).
    Scroll(Direction),
    /// The selected hero one step that way (the original's direction key alone).
    Hero(Direction),
    /// The music heard or not (E24 M2 T5).
    Music,
    /// The effects (the step and ambient sounds) heard or not (E24 M2 T5).
    Effects,
    /// The clock held, or run on.
    Pause,
    /// While paused, the next frame.
    Frame,
    /// The start again.
    Restart,
    /// Vsync on or off.
    Vsync,
    /// The hardware cursor or the drawn one (E24 M2 T4b).
    Cursor,
    /// Fullscreen on or off.
    Fullscreen,
    /// The window closed.
    Quit,
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

/// The window's own commands and their keys.
const WINDOW_KEYS: [(Command, &str); 9] = [
    (Command::Music, "F2"),
    (Command::Effects, "F3"),
    (Command::Pause, "F5"),
    (Command::Frame, "F6"),
    (Command::Restart, "F7"),
    (Command::Vsync, "F8"),
    (Command::Cursor, "F9"),
    (Command::Fullscreen, "F11"),
    (Command::Quit, "Escape"),
];

impl Command {
    /// Every command, in the table's order: the scroll's eight directions, the hero's eight, the
    /// window's own.
    #[must_use]
    pub fn all() -> Vec<Self> {
        let mut all: Vec<Self> = Direction::ALL.into_iter().map(Self::Scroll).collect();
        all.extend(Direction::ALL.into_iter().map(Self::Hero));
        all.extend(WINDOW_KEYS.into_iter().map(|(c, _)| c));
        all
    }

    /// Its name in a user's file: `scroll-north`, `hero-south-west`, `pause`, …
    #[must_use]
    pub fn name(self) -> String {
        match self {
            Self::Scroll(d) => format!("scroll-{}", direction_name(d)),
            Self::Hero(d) => format!("hero-{}", direction_name(d)),
            Self::Pause => "pause".into(),
            Self::Frame => "frame".into(),
            Self::Restart => "restart".into(),
            Self::Vsync => "vsync".into(),
            Self::Cursor => "cursor".into(),
            Self::Fullscreen => "fullscreen".into(),
            Self::Music => "music".into(),
            Self::Effects => "effects".into(),
            Self::Quit => "quit".into(),
        }
    }

    /// The command a name gives.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::all().into_iter().find(|c| c.name() == name)
    }

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

/// A key and whether Ctrl is held with it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Chord {
    /// Either Control key held.
    pub ctrl: bool,
    /// The key's name (the adapter's).
    pub key: String,
}

impl Chord {
    /// `Ctrl+<key>` or `<key>`.
    ///
    /// # Errors
    /// An empty key, or one with a `+` or white space in it.
    pub fn parse(text: &str) -> Result<Self, String> {
        let (ctrl, key) = text
            .strip_prefix("Ctrl+")
            .map_or((false, text), |key| (true, key));
        if key.is_empty() || key.contains(|c: char| c == '+' || c.is_whitespace()) {
            return Err(format!("`{text}` is not a key, or Ctrl+ and a key"));
        }
        Ok(Self {
            ctrl,
            key: key.to_string(),
        })
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ctrl {
            write!(f, "Ctrl+{}", self.key)
        } else {
            f.write_str(&self.key)
        }
    }
}

/// The table: each command's chords, each chord bound once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keys {
    /// Each command's chords, in the table's order (a command without one left out).
    bound: BTreeMap<Command, Vec<Chord>>,
    /// The command each chord gives.
    by_chord: BTreeMap<Chord, Command>,
}

impl Keys {
    /// The defaults: the original's keys on the map and the window's own.
    #[must_use]
    pub fn original() -> Self {
        let mut bound = BTreeMap::new();
        for ((d, key), pad) in Direction::ALL
            .into_iter()
            .zip(DIRECTION_KEYS)
            .zip(KEYPAD_KEYS)
        {
            let chords = |ctrl| {
                [key, pad].map(|k| Chord {
                    ctrl,
                    key: k.to_string(),
                })
            };
            bound.insert(Command::Scroll(d), Vec::from(chords(true)));
            bound.insert(Command::Hero(d), Vec::from(chords(false)));
        }
        for (command, key) in WINDOW_KEYS {
            bound.insert(
                command,
                Vec::from([Chord {
                    ctrl: false,
                    key: key.to_string(),
                }]),
            );
        }
        let by_chord = bound
            .iter()
            .flat_map(|(&c, chords)| chords.iter().map(move |k| (k.clone(), c)))
            .collect();
        Self { bound, by_chord }
    }

    /// The defaults with a user's file over them: a TOML table of commands by their names, each
    /// with the list of its chords (`pause = ["F5", "Space"]`); a command the file leaves out
    /// keeps its default keys, an empty list unbinds it.
    ///
    /// # Errors
    /// The parser's message, a name that is no command, a value that is no list of chords, a chord
    /// that does not read or is bound to two commands.
    pub fn with_file(text: &str) -> Result<Self, String> {
        let table: toml::Table = toml::from_str(text).map_err(|e| e.to_string())?;
        let mut bound = Self::original().bound;
        for (name, value) in &table {
            let command = Command::from_name(name).ok_or_else(|| {
                let names: Vec<String> = Command::all().into_iter().map(Command::name).collect();
                format!("`{name}` is no command; the commands: {}", names.join(", "))
            })?;
            let texts: Option<Vec<&str>> = value
                .as_array()
                .and_then(|a| a.iter().map(toml::Value::as_str).collect());
            let texts = texts.ok_or_else(|| format!("`{name}` is not a list of keys"))?;
            let chords = texts
                .into_iter()
                .map(Chord::parse)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("`{name}`: {e}"))?;
            bound.insert(command, chords);
        }
        bound.retain(|_, chords| !chords.is_empty());
        let mut by_chord: BTreeMap<Chord, Command> = BTreeMap::new();
        for (&command, chords) in &bound {
            for chord in chords {
                if let Some(other) = by_chord.insert(chord.clone(), command)
                    && other != command
                {
                    return Err(format!(
                        "{chord} is bound to both `{}` and `{}`",
                        other.name(),
                        command.name()
                    ));
                }
            }
        }
        Ok(Self { bound, by_chord })
    }

    /// The command a key gives, with Ctrl held or not.
    #[must_use]
    pub fn command(&self, ctrl: bool, key: &str) -> Option<Command> {
        self.by_chord
            .get(&Chord {
                ctrl,
                key: key.to_string(),
            })
            .copied()
    }

    /// Every chord bound with its command, in the table's order.
    #[must_use]
    pub fn chords(&self) -> Vec<(Command, &Chord)> {
        self.bound
            .iter()
            .flat_map(|(&c, chords)| chords.iter().map(move |k| (c, k)))
            .collect()
    }
}

#[cfg(test)]
mod tests;
