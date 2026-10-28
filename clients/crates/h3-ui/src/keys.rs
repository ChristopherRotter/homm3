//! A window's keys (E24 M2 T4a, *provisional*; the user, 2026-10-26: no key hard-coded in the
//! client): every key a viewer's window takes goes through one table of that viewer's commands
//! ([`Command`]: the commands, their names in a user's file and their default keys are the
//! viewer's own; the map's: `h3-map-view`'s `keys`, the battle's: `h3-view`'s). A user's file
//! overrides the defaults command by command ([`Keys::with_file`]), each viewer's in a table of its
//! own ([`VIEWERS`]). A key is named as the adapter names it (`h3-sdl`'s `Key::name`: `Up`,
//! `PageUp`, `F5`, …), `Ctrl+` holding either Control key with it; the adapter checks the names
//! ([`Keys::check`]). A chord matches exactly: a key with Ctrl held is not the key alone. Both
//! windows' own keys are the same ([`Window`]: the function keys and Esc, the user at E24 M3 T0).
//! A viewer with dialogs over its screen takes some keys only while one is open
//! ([`Command::context`]: the battle's spell book, message box and aim), so a key may give one
//! command in each. Common to both viewers and generic over their commands since E24 M3 T1a, the
//! battle's window through it since T1b; a screen to rebind them comes with the options dialog.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

/// The viewers' tables in a user's file, by name: `[map]` the adventure map's, `[battle]` the
/// battle's (E24 M3 T1b).
pub const VIEWERS: [&str; 2] = ["map", "battle"];

/// The adapter's names of the Control keys, held with a key (`Ctrl+`) and never bound alone.
pub const CONTROL: [&str; 2] = ["LeftCtrl", "RightCtrl"];

/// A viewer's commands: what its keys do.
pub trait Command: Copy + Ord {
    /// Where the viewer takes its keys: what is open over its screen (the battle's: nothing, the
    /// spell book, a message box, an aim); `()` for a viewer with one.
    type Context: Copy + Eq;

    /// Its table's name in a user's file (one of [`VIEWERS`]).
    const TABLE: &'static str;

    /// Every command, in the table's order (its `Ord`'s).
    fn all() -> Vec<Self>;

    /// Its name in a user's file (`pause`, `scroll-north`, …).
    fn name(self) -> String;

    /// The default keys: each command bound by default with its chords, in the table's order.
    fn defaults() -> Vec<(Self, Vec<Chord>)>;

    /// The context it is taken in; none for every one (the window's own keys that work whatever
    /// is open). Two commands that can be taken in one context never share a chord.
    fn context(self) -> Option<Self::Context>;

    /// The command a name gives.
    #[must_use]
    fn from_name(name: &str) -> Option<Self> {
        Self::all().into_iter().find(|c| c.name() == name)
    }
}

/// A window's own commands, the same in both viewers (the user, E24 M3 T0): the music, the
/// effects, pause, a frame on, restart, vsync, the cursor's kind, fullscreen, quit, on the
/// function keys and Esc, keys the original's screens leave to nothing else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Window {
    /// The music heard or not.
    Music,
    /// The effects (the map's steps and ambient loops, the battle's creatures) heard or not.
    Effects,
    /// The clock held, or run on.
    Pause,
    /// While paused, the next frame.
    Frame,
    /// The start again.
    Restart,
    /// Vsync on or off.
    Vsync,
    /// The hardware cursor or the drawn one.
    Cursor,
    /// Fullscreen on or off.
    Fullscreen,
    /// The window closed.
    Quit,
}

impl Window {
    /// Every one, in the table's order.
    pub const ALL: [Self; 9] = [
        Self::Music,
        Self::Effects,
        Self::Pause,
        Self::Frame,
        Self::Restart,
        Self::Vsync,
        Self::Cursor,
        Self::Fullscreen,
        Self::Quit,
    ];

    /// Its name in a user's file.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Music => "music",
            Self::Effects => "effects",
            Self::Pause => "pause",
            Self::Frame => "frame",
            Self::Restart => "restart",
            Self::Vsync => "vsync",
            Self::Cursor => "cursor",
            Self::Fullscreen => "fullscreen",
            Self::Quit => "quit",
        }
    }

    /// Its default key: F2 the music, F3 the effects, F5 pause, F6 a frame on, F7 restart, F8
    /// vsync, F9 the cursor's kind, F11 fullscreen, Esc quit.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Music => "F2",
            Self::Effects => "F3",
            Self::Pause => "F5",
            Self::Frame => "F6",
            Self::Restart => "F7",
            Self::Vsync => "F8",
            Self::Cursor => "F9",
            Self::Fullscreen => "F11",
            Self::Quit => "Escape",
        }
    }

    /// Whether it works whatever is open over the viewer's screen: all but quit, a frame on and
    /// restart, which the screen takes only with nothing over it (the battle's Esc is a dialog's
    /// Cancel there).
    #[must_use]
    pub const fn everywhere(self) -> bool {
        !matches!(self, Self::Quit | Self::Frame | Self::Restart)
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
    /// The key `key`, with Ctrl held or not.
    #[must_use]
    pub fn new(ctrl: bool, key: &str) -> Self {
        Self {
            ctrl,
            key: key.to_string(),
        }
    }

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
        Ok(Self::new(ctrl, key))
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

/// The table: each command's chords; a chord bound to two commands only where no context takes
/// both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keys<C> {
    /// Each command's chords, in the table's order (a command without one left out).
    bound: BTreeMap<C, Vec<Chord>>,
    /// The commands each chord gives, in the table's order.
    by_chord: BTreeMap<Chord, Vec<C>>,
}

impl<C: Command> Keys<C> {
    /// The defaults ([`Command::defaults`]).
    #[must_use]
    pub fn original() -> Self {
        let bound: BTreeMap<C, Vec<Chord>> = C::defaults().into_iter().collect();
        let mut by_chord: BTreeMap<Chord, Vec<C>> = BTreeMap::new();
        for (&command, chords) in &bound {
            for chord in chords {
                by_chord.entry(chord.clone()).or_default().push(command);
            }
        }
        Self { bound, by_chord }
    }

    /// The defaults with a user's file over them: a TOML file of the viewers' tables
    /// ([`VIEWERS`]), this viewer's ([`Command::TABLE`]) of commands by their names, each with the
    /// list of its chords (`[map]`, then `pause = ["F5", "Space"]`); a command the table leaves
    /// out keeps its default keys, an empty list unbinds it; a file without the viewer's table is
    /// the defaults.
    ///
    /// # Errors
    /// The parser's message, a name that is no viewer's table or no command, a value that is no
    /// table or no list of chords, a chord that does not read or is bound to two commands one
    /// context takes.
    pub fn with_file(text: &str) -> Result<Self, String> {
        let file: toml::Table = toml::from_str(text).map_err(|e| e.to_string())?;
        for (name, value) in &file {
            if !VIEWERS.contains(&name.as_str()) {
                return Err(format!(
                    "`{name}` is no viewer's table; the tables: {}",
                    VIEWERS.join(", ")
                ));
            }
            if !value.is_table() {
                return Err(format!("`{name}` is not a table of commands"));
            }
        }
        let empty = toml::Table::new();
        let table = file
            .get(C::TABLE)
            .and_then(toml::Value::as_table)
            .unwrap_or(&empty);
        Self::with_table(table).map_err(|e| format!("[{}] {e}", C::TABLE))
    }

    /// The defaults with a viewer's table over them.
    fn with_table(table: &toml::Table) -> Result<Self, String> {
        let mut bound = Self::original().bound;
        for (name, value) in table {
            let command = C::from_name(name).ok_or_else(|| {
                let names: Vec<String> = C::all().into_iter().map(C::name).collect();
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
        let mut by_chord: BTreeMap<Chord, Vec<C>> = BTreeMap::new();
        for (&command, chords) in &bound {
            for chord in chords {
                let commands = by_chord.entry(chord.clone()).or_default();
                if commands.contains(&command) {
                    continue;
                }
                if let Some(&other) = commands.iter().find(|&&o| together(o, command)) {
                    return Err(format!(
                        "{chord} is bound to both `{}` and `{}`",
                        other.name(),
                        command.name()
                    ));
                }
                commands.push(command);
            }
        }
        Ok(Self { bound, by_chord })
    }

    /// The command a key gives in `context`, with Ctrl held or not.
    #[must_use]
    pub fn command(&self, context: C::Context, ctrl: bool, key: &str) -> Option<C> {
        self.by_chord
            .get(&Chord::new(ctrl, key))?
            .iter()
            .copied()
            .find(|c| c.context().is_none_or(|x| x == context))
    }

    /// Every chord bound with its command, in the table's order.
    #[must_use]
    pub fn chords(&self) -> Vec<(C, &Chord)> {
        self.bound
            .iter()
            .flat_map(|(&c, chords)| chords.iter().map(move |k| (c, k)))
            .collect()
    }

    /// Every key the table binds is one the adapter knows (`known`, its names) and not Control
    /// alone ([`CONTROL`]: Ctrl+ holds it).
    ///
    /// # Errors
    /// The first chord that is not, with its command; for an unknown key the keys there are.
    pub fn check(&self, known: &[&str]) -> Result<(), String> {
        for (command, chord) in self.chords() {
            let key = chord.key.as_str();
            if CONTROL.contains(&key) {
                return Err(format!(
                    "`{}`: {chord}: Control is held with a key (`Ctrl+`), not bound alone",
                    command.name()
                ));
            }
            if !known.contains(&key) {
                let names: Vec<&str> = known
                    .iter()
                    .copied()
                    .filter(|k| !CONTROL.contains(k))
                    .collect();
                return Err(format!(
                    "`{}`: {chord}: no such key; the keys: {}",
                    command.name(),
                    names.join(", ")
                ));
            }
        }
        Ok(())
    }
}

/// Whether one context can take both commands: neither has one of its own, or they have the same.
fn together<C: Command>(a: C, b: C) -> bool {
    match (a.context(), b.context()) {
        (Some(x), Some(y)) => x == y,
        _ => true,
    }
}

#[cfg(test)]
mod tests;
