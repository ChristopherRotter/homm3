//! The adventure side of the `h3` tool (E08 T9b; adventure-owned, `lanes.toml`): recording random
//! and scripted games and verifying replays. The commands themselves are in [`cli`]. The files
//! (the packs with their adventure files, the `.h3m` maps and the replay file format v8) are
//! `h3_files::adventure`'s, re-exported here (moved there with the user's file crate,
//! 2026-11-09).

#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

pub mod bench;
pub mod cli;

use std::path::Path;
use std::sync::Arc;

use h3_game::{Actor, GameCommand};
use h3_sim::adventure::replay::{RecordOptions, Replay};

/// The adventure's files (`h3-files`): its loaders, the replay file and the commands' text form.
pub use h3_files::adventure::{AdventureFileError as AdventureCliError, *};

/// The packs an adventure game loads by default: the original three, RoE to SoD.
pub const DEFAULT_PACKS: [&str; 3] = ["base", "ab", "sod"];

/// The default profile.
pub const DEFAULT_PROFILE: &str = "base:faithful";

type Result<T> = std::result::Result<T, AdventureCliError>;

fn read_text(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|e| AdventureCliError::Io(path.to_path_buf(), e))
}

/// What [`run_random`] plays.
#[derive(Debug, Clone)]
pub struct RunOptions {
    /// The map's path, relative to the root.
    pub map: String,
    /// The profile key.
    pub profile: String,
    /// The game's difficulty (setup input, checked by the game's build: an unknown one is the
    /// setup's error).
    pub difficulty: u8,
    /// The game and the agent ([`RecordOptions`]).
    pub record: RecordOptions,
}

/// Plays a game of uniformly random legal agents on a map at the options' difficulty and returns
/// the replay and its TOML.
///
/// # Errors
///
/// The first [`AdventureCliError`].
pub fn run_random(packs_dir: &Path, root: &Path, options: &RunOptions) -> Result<(Replay, String)> {
    let ids: Vec<String> = DEFAULT_PACKS.iter().map(|&s| s.to_owned()).collect();
    let rules = load_rules(packs_dir, &ids, &options.profile)?;
    let (map, mut setup) = load_map(root, &options.map, &rules)?;
    setup.difficulty = options.difficulty;
    let replay = Replay::record_random(&rules, Arc::new(setup), map, options.record)
        .map_err(AdventureCliError::Record)?;
    let text = replay_to_toml(&replay)?;
    Ok((replay, text))
}

/// What [`run_script`] plays (E08 T52).
#[derive(Debug, Clone)]
pub struct ScriptOptions {
    /// The map's path, relative to the root.
    pub map: String,
    /// The profile key.
    pub profile: String,
    /// The game's difficulty (setup input, checked by the game's build).
    pub difficulty: u8,
    /// The game's seed.
    pub seed: u64,
    /// Checkpoint interval (0 = none).
    pub checkpoint_every: u32,
}

/// Reads a script (E08 T52): a replay file's commands (a path ending in `.toml`), or a text file
/// of commands, one per line as a replay writes them ([`format_command`]), blank lines and lines
/// starting with `#` skipped.
///
/// # Errors
///
/// The file's, or [`AdventureCliError::Format`] naming a malformed line or replay.
pub fn read_script(path: &Path) -> Result<Vec<(Actor, GameCommand)>> {
    let text = read_text(path)?;
    if path.extension().is_some_and(|e| e == "toml") {
        return Ok(parse_replay(&text)?.commands);
    }
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(parse_command)
        .collect()
}

/// Plays `commands` on a map at the options' difficulty and returns the replay and its TOML (E08
/// T52: the scripted golden replays, of what the random agent does not play; a replay's own
/// commands re-record it when a hash moves).
///
/// # Errors
///
/// The first [`AdventureCliError`]: the rules', the map's, the setup's or a rejected command's.
pub fn run_script(
    packs_dir: &Path,
    root: &Path,
    options: &ScriptOptions,
    commands: Vec<(Actor, GameCommand)>,
) -> Result<(Replay, String)> {
    let ids: Vec<String> = DEFAULT_PACKS.iter().map(|&s| s.to_owned()).collect();
    let rules = load_rules(packs_dir, &ids, &options.profile)?;
    let (map, mut setup) = load_map(root, &options.map, &rules)?;
    setup.difficulty = options.difficulty;
    let replay = Replay::record_commands(
        &rules,
        (&setup, map),
        options.seed,
        commands,
        options.checkpoint_every,
    )
    .map_err(AdventureCliError::Record)?;
    let text = replay_to_toml(&replay)?;
    Ok((replay, text))
}

#[cfg(test)]
mod tests;
