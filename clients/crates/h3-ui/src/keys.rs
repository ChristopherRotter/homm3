//! A window's keys (E24 M2 T4a, *provisional*; the user, 2026-10-26: no key hard-coded in the
//! client): every key a viewer's window takes goes through one table of that viewer's commands
//! ([`Command`]: the commands, their names in a user's file and their default keys are the
//! viewer's own; the map's: `h3-map-view`'s `keys`). A user's file overrides the defaults command
//! by command ([`Keys::with_file`]). A key is named as the adapter names it (`h3-sdl`'s
//! `Key::name`: `Up`, `PageUp`, `F5`, …), `Ctrl+` holding either Control key with it; the adapter
//! checks the names ([`Keys::chords`]). A chord matches exactly: a key with Ctrl held is not the
//! key alone. Common to both viewers and generic over their commands since E24 M3 T1a (the
//! battle's window through it from T1b); a screen to rebind them comes with the options dialog.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

/// A viewer's commands: what its keys do.
pub trait Command: Copy + Ord {
    /// Every command, in the table's order.
    fn all() -> Vec<Self>;

    /// Its name in a user's file (`pause`, `scroll-north`, …).
    fn name(self) -> String;

    /// The default keys: each command bound by default with its chords, in the table's order.
    fn defaults() -> Vec<(Self, Vec<Chord>)>;

    /// The command a name gives.
    #[must_use]
    fn from_name(name: &str) -> Option<Self> {
        Self::all().into_iter().find(|c| c.name() == name)
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
pub struct Keys<C> {
    /// Each command's chords, in the table's order (a command without one left out).
    bound: BTreeMap<C, Vec<Chord>>,
    /// The command each chord gives.
    by_chord: BTreeMap<Chord, C>,
}

impl<C: Command> Keys<C> {
    /// The defaults ([`Command::defaults`]).
    #[must_use]
    pub fn original() -> Self {
        let bound: BTreeMap<C, Vec<Chord>> = C::defaults().into_iter().collect();
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
        let mut by_chord: BTreeMap<Chord, C> = BTreeMap::new();
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
    pub fn command(&self, ctrl: bool, key: &str) -> Option<C> {
        self.by_chord
            .get(&Chord {
                ctrl,
                key: key.to_string(),
            })
            .copied()
    }

    /// Every chord bound with its command, in the table's order.
    #[must_use]
    pub fn chords(&self) -> Vec<(C, &Chord)> {
        self.bound
            .iter()
            .flat_map(|(&c, chords)| chords.iter().map(move |k| (c, k)))
            .collect()
    }
}

#[cfg(test)]
mod tests;
