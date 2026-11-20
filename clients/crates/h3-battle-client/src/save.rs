//! A saved game (E27 M4 T3b; decisions 69 and 71, CLT-Q156 (a); `docs/epics/E27-design.md`
//! §37): the client's own file, the engine's adventure replay (`h3-files`' format v20: the map by
//! its file and content hash, the seed, the difficulty, the players' choices, the commands and the
//! hashes) followed by the client's table (the save's name, the turn's duration, each colour's
//! human's name, starting bonus and handicap: what the engine's setup has no field for,
//! [`Game`]), with its own [`VERSION`]. The original's save file (`game_save` 0x4BEEA0, the
//! game's whole memory) is not this one: the original's saves stay the original's (decision 8).
//!
//! - **The file** ([`Save::write`], [`Save::read`]; *provisional*): the replay's text as
//!   `h3_files::adventure::replay_to_toml` writes it, then, from the line `[save]` on, the client's
//!   table and its eight `[[save.colours]]`, red first. The replay part is the engine's file
//!   whole, read by `h3_files::adventure::parse_replay`; the replay names its map by the file's
//!   name in the installation's `Maps`, the folder it is resolved against ([`map`]).
//! - **The load** ([`load`]): the replay's header checked against the rules before anything is
//!   read; its map read again from the installation (`h3_files::adventure::read_map`'s checks: a
//!   plain name below `Maps`, within `h3-map`'s size limit) and refused when its content hash or
//!   the import's version moved; imported as [`crate::begin::setup`] reads a map; the replay's
//!   choices set over the import (`Replay::setup`), a human only where the import lets one play
//!   (as the begin refuses it, E27 M3 T7) and a name only for a human; then the commands
//!   replayed through the engine (`h3_game::apply`, each as its actor's), with `Replay::verify`'s
//!   checks (the checkpoints, the final hash, the final day), the state kept ([`Loaded`]).
//!
//! Where a save is kept, its file's name and the list of them are Load Game's (T4) and Save's
//! (M5); a save is made from play (M5's Save, its host's replay).

use std::fs;
use std::path::Path;

use h3_game::{GameRuleset, GameSetup, GameState, NoopSink, apply};
use h3_map::Limits;
use h3_sim::adventure::replay::{MapRef, Replay, ReplayError};
use h3_view::shell::scenario::{Bonus, UNLIMITED};
use serde::Deserialize;

use crate::begin::Game;

/// The version of the client's table (`[save]`'s `version`, *provisional*): a file of another, or
/// of none, is refused. The replay part has its own (`h3_sim`'s `REPLAY_FORMAT_VERSION`), checked
/// at the load.
pub const VERSION: i64 = 1;

/// The line that begins the client's table: the replay's text ends before it.
const TABLE: &str = "[save]";

/// The colours a table holds, red first.
const COLOURS: usize = 8;

/// The handicaps (the record's: 0 none, 1 mild, 2 severe).
const HANDICAPS: u8 = 3;

/// The comment a save's file begins with.
const HEADER: &str = "# A saved game of the h3-view shell (E27 M4 T3b): the engine's adventure replay\n\
                      # (h3-files' format), then the client's table from the line [save] on.\n";

/// A saved game: the engine's replay and the client's table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Save {
    /// The save's name (the scenario screen's edit in its save mode, design §3.6).
    pub name: String,
    /// Each colour's human's name, red first; none for a computer's ([`Game::names`]).
    pub names: [Option<String>; 8],
    /// Each colour's starting bonus, red first ([`Game::bonuses`]).
    pub bonuses: [Bonus; 8],
    /// Each colour's handicap, red first ([`Game::handicaps`]).
    pub handicaps: [u8; 8],
    /// The turn's duration (`turndur.txt`'s record, [`Game::duration`]).
    pub duration: u8,
    /// The engine's record of the game: its map, seed, difficulty, players' choices, commands and
    /// hashes.
    pub replay: Replay,
}

impl Save {
    /// The save named `name` of `game` and its engine's `replay` (the record of the game `game`
    /// began, its map named by [`map`]).
    #[must_use]
    pub fn of(name: &str, game: &Game, replay: Replay) -> Self {
        Self {
            name: name.to_owned(),
            names: game.names.clone(),
            bonuses: game.bonuses,
            handicaps: game.handicaps,
            duration: game.duration,
            replay,
        }
    }

    /// The save's file: the replay's text, then the client's table ([`Save::read`]).
    ///
    /// # Errors
    /// The replay's writer's (none for a replay the engine recorded).
    pub fn write(&self) -> Result<String, String> {
        let replay =
            h3_files::adventure::replay_to_toml(&self.replay).map_err(|e| e.to_string())?;
        let mut text = String::from(HEADER);
        text.push_str(&replay);
        if !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&format!(
            "\n{TABLE}\nversion = {VERSION}\nname = {}\nduration = {}\n",
            quoted(&self.name),
            self.duration
        ));
        for colour in 0..COLOURS {
            text.push_str("\n[[save.colours]]\n");
            if let Some(name) = &self.names[colour] {
                text.push_str(&format!("name = {}\n", quoted(name)));
            }
            text.push_str(&format!(
                "bonus = \"{}\"\nhandicap = {}\n",
                word(self.bonuses[colour]),
                self.handicaps[colour]
            ));
        }
        Ok(text)
    }

    /// The save of a file's `text` ([`Save::write`]'s format): the replay's text up to the line
    /// `[save]`, the client's table from it on: its `version` the [`VERSION`], the `name`, the
    /// `duration` (at most unlimited's), eight `colours`, each with a `bonus` (`artifact`, `gold`,
    /// `resource` or `random`), a `handicap` (0 to 2) and, for a human's, its `name`.
    ///
    /// # Errors
    /// No line `[save]`; the table not TOML, of another version or none, not as above; the
    /// replay's text not the engine's format.
    pub fn read(text: &str) -> Result<Self, String> {
        let at = table_start(text)
            .ok_or_else(|| format!("no line `{TABLE}`: not a saved game of this client"))?;
        let (replay, table) = text.split_at(at);
        let file: toml::Table = toml::from_str(table).map_err(|e| format!("[save]: {e}"))?;
        let version = file
            .get("save")
            .and_then(toml::Value::as_table)
            .and_then(|t| t.get("version"))
            .map(toml::Value::as_integer);
        match version {
            Some(Some(VERSION)) => {}
            Some(Some(v)) => {
                return Err(format!(
                    "[save]: version {v}: this client reads version {VERSION}"
                ));
            }
            _ => return Err(String::from("[save]: no `version` number")),
        }
        let file: File = toml::from_str(table).map_err(|e| format!("[save]: {e}"))?;
        let table = file.save;
        if table.duration > UNLIMITED {
            return Err(format!(
                "[save]: duration {}, past unlimited's {UNLIMITED}",
                table.duration
            ));
        }
        let colours: [Colour; COLOURS] = table
            .colours
            .try_into()
            .map_err(|c: Vec<Colour>| format!("[save]: {} colours, not {COLOURS}", c.len()))?;
        let mut bonuses = [Bonus::Random; COLOURS];
        let mut handicaps = [0; COLOURS];
        for (colour, entry) in colours.iter().enumerate() {
            bonuses[colour] = bonus(&entry.bonus)
                .ok_or_else(|| format!("[save]: colour {colour}: bonus `{}`", entry.bonus))?;
            if entry.handicap >= HANDICAPS {
                return Err(format!(
                    "[save]: colour {colour}: handicap {}",
                    entry.handicap
                ));
            }
            handicaps[colour] = entry.handicap;
        }
        let replay = h3_files::adventure::parse_replay(replay).map_err(|e| e.to_string())?;
        Ok(Self {
            name: table.name,
            names: colours.map(|c| c.name),
            bonuses,
            handicaps,
            duration: table.duration,
            replay,
        })
    }
}

/// The byte where the line `[save]` begins in `text`, if it has one.
fn table_start(text: &str) -> Option<usize> {
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        if line.trim_end() == TABLE {
            return Some(at);
        }
        at += line.len();
    }
    None
}

/// The client's table as [`Save::read`] takes it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    save: Table,
}

/// The `[save]` table.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Table {
    /// Checked before (a number of any version).
    #[serde(rename = "version")]
    _version: i64,
    name: String,
    duration: u8,
    colours: Vec<Colour>,
}

/// A colour's entry of the table.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Colour {
    #[serde(default)]
    name: Option<String>,
    bonus: String,
    handicap: u8,
}

/// A bonus's word in the file.
const fn word(bonus: Bonus) -> &'static str {
    match bonus {
        Bonus::Artifact => "artifact",
        Bonus::Gold => "gold",
        Bonus::Resource => "resource",
        Bonus::Random => "random",
    }
}

/// The bonus of a file's `text`, none for another word.
fn bonus(text: &str) -> Option<Bonus> {
    [Bonus::Artifact, Bonus::Gold, Bonus::Resource, Bonus::Random]
        .into_iter()
        .find(|b| word(*b) == text)
}

/// `text` as a TOML basic string: `"` and `\` escaped, each control character as `\uXXXX`.
fn quoted(text: &str) -> String {
    let mut quoted = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            c if c.is_control() => quoted.push_str(&format!("\\u{:04X}", u32::from(c))),
            c => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}

/// The save file at `path`, read whole ([`Save::read`]).
///
/// # Errors
/// The file does not read, or is not a save.
pub fn read_file(path: &Path) -> Result<Save, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Save::read(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Writes `save` to `path` whole or not at all (as the settings file: a sibling renamed over it).
///
/// # Errors
/// The save's writer's, or the file does not write.
pub fn write_file(path: &Path, save: &Save) -> Result<(), String> {
    let text = save.write()?;
    crate::window::write_whole(path, &text).map_err(|e| format!("{}: {e}", path.display()))
}

/// How a save's replay names the installation's map `file` (the file's name in `Maps`, the folder
/// [`load`] resolves it against): the name, the file's content hash, the import's version.
///
/// # Errors
/// Not a plain name of a file in `install`'s `Maps`, the file does not read, or is over
/// `h3-map`'s size limit.
pub fn map(install: &Path, file: &str) -> Result<MapRef, String> {
    h3_files::adventure::read_map(&install.join(crate::maps::MAPS), file)
        .map(|(_, map)| map)
        .map_err(|e| e.to_string())
}

/// A game loaded: begun again as the save's, its state after the save's commands.
#[derive(Debug)]
pub struct Loaded {
    /// The game as [`crate::begin::game`] makes one: the map's file, the setup rebuilt and
    /// validated, the client's table.
    pub game: Game,
    /// The game's state after the save's commands, replayed by the engine.
    pub state: GameState,
}

/// `save`'s game under `rules` over the installation `install`'s map (the module's docs).
///
/// # Errors
/// The save's replay does not fit the rules, its map is missing or not the one it names, the
/// import fails, a choice is refused (a human where the map lets none play, a name for a
/// computer's colour, the engine's refusal of the setup), a command is rejected, or a hash or the
/// day differs.
pub fn load(save: &Save, install: &Path, rules: &GameRuleset) -> Result<Loaded, String> {
    let replay = &save.replay;
    let refused = |e: ReplayError| format!("the save's replay: {e}");
    // The rules first: a save of other rules is refused before its map is read (as the engine's
    // `verify_file`).
    replay.header.check(rules).map_err(refused)?;
    // The map by its name and hash before its bytes are parsed.
    let (bytes, map) =
        h3_files::adventure::read_map(&install.join(crate::maps::MAPS), &replay.map.path)
            .map_err(|e| e.to_string())?;
    if map.content_hash != replay.map.content_hash {
        return Err(refused(ReplayError::Incompatible("map.hash")));
    }
    if map.import_version != replay.map.import_version {
        return Err(refused(ReplayError::Incompatible("map.import_version")));
    }
    let raw = h3_map::read(&bytes, &Limits::default())
        .map_err(|e| format!("{}: {e}", replay.map.path))?;
    let import = h3_map::import(&raw, rules).map_err(|e| format!("the import: {e}"))?;
    // The import's `human` the only may-be-human mark, which the engine cannot check again (as
    // `begin::setup`).
    for choices in replay.players.iter().filter(|c| c.human) {
        if !import
            .players
            .iter()
            .any(|p| p.colour == choices.colour && p.human)
        {
            return Err(format!(
                "the save makes colour {} human, which the map gives the computer alone",
                choices.colour
            ));
        }
    }
    let setup = replay.setup(&import).map_err(refused)?;
    for (colour, name) in save.names.iter().enumerate() {
        let human = setup
            .players
            .iter()
            .any(|p| usize::from(p.colour & 7) == colour && p.human);
        if name.is_some() && !human {
            return Err(format!(
                "the save names colour {colour}, which no human holds"
            ));
        }
    }
    let state = replayed(replay, rules, &setup).map_err(refused)?;
    Ok(Loaded {
        game: Game {
            file: replay.map.path.clone(),
            setup,
            names: save.names.clone(),
            bonuses: save.bonuses,
            handicaps: save.handicaps,
            duration: save.duration,
        },
        state,
    })
}

/// The state after `replay`'s commands from `setup`, each applied through the engine as its
/// actor's, with `Replay::verify`'s checks in its order (the checkpoints' count, each checkpoint,
/// the final hash, the final day; *provisional*: the engine's `verify` keeps no state, M4's review
/// list).
fn replayed(
    replay: &Replay,
    rules: &GameRuleset,
    setup: &GameSetup,
) -> Result<GameState, ReplayError> {
    let every = replay.checkpoint_every as usize;
    if replay.checkpoints.len() != replay.commands.len().checked_div(every).unwrap_or(0) {
        return Err(ReplayError::CheckpointCount);
    }
    let mut state = GameState::new(rules, setup, replay.seed).map_err(ReplayError::Setup)?;
    let mut checkpoints = replay.checkpoints.iter();
    for (i, (actor, command)) in replay.commands.iter().enumerate() {
        apply(&mut state, rules, *actor, command.clone(), &mut NoopSink)
            .map_err(|error| ReplayError::Rejected { step: i, error })?;
        let step = i + 1;
        if every != 0 && step % every == 0 {
            let expected = checkpoints.next().copied().unwrap_or_default();
            let actual = state.state_hash();
            if expected != actual {
                return Err(ReplayError::HashMismatch {
                    step,
                    expected,
                    actual,
                });
            }
        }
    }
    let actual = state.state_hash();
    if actual != replay.final_hash {
        return Err(ReplayError::HashMismatch {
            step: replay.commands.len(),
            expected: replay.final_hash,
            actual,
        });
    }
    if state.day() != replay.final_day {
        return Err(ReplayError::DayMismatch {
            expected: replay.final_day,
            actual: state.day(),
        });
    }
    Ok(state)
}

#[cfg(test)]
mod tests;
