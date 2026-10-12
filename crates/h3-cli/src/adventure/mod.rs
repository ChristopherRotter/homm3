//! The adventure side of the `h3` tool (E08 T9b; adventure-owned, `lanes.toml`): loading packs
//! with their adventure files and `.h3m` maps from disk, recording random games and the adventure
//! replay file format (TOML). The commands themselves are in [`cli`].
//!
//! Replay file format v1 (`fixtures/replays/adventure/*.toml` are examples):
//!
//! ```toml
//! commands = [                        # the game commands in order, with the acting player:
//!     "0 move 1 5,7,0 6,8,0",         #   `<player> move <hero> <x>,<y>,<level> …` (the path
//!     "0 end-turn",                   #   without the hero's own tile), `<player> end-turn`
//!     "1 accept",                     #   `<player> accept` or `decline`: an answer (E08 T14b)
//! ]
//! checkpoints = ["0x…"]               # state hash after every `checkpoint_every` commands
//!
//! [replay]
//! format = 1
//! engine = "0.1.0"
//! seed = "1"                          # u64 as a decimal string
//! packs = [["base", "1.0.0"], ["ab", "1.0.0"], ["sod", "1.0.0"]]
//! profile = "base:faithful"
//! ruleset_hash = "0x…"                # the core ruleset's content hash
//! adventure_ruleset_hash = "0x…"      # the adventure part's
//! rng = "xoshiro256starstar+splitmix64"
//! rng_version = 1
//! hash = "fnv1a64-le"
//! hash_version = 1
//! game_state_hash_version = 1
//! adventure_state_hash_version = 4
//! checkpoint_every = 25
//!
//! [map]                               # how the replay names its map (ADV-Q2)
//! path = "fixtures/maps/e08-duel-36.h3m"  # relative, resolved against the tool's root
//! hash = "0x…"                        # the file's content hash
//! import_version = 3                  # h3_map::MAP_IMPORT_VERSION
//!
//! [result]
//! day = 29                            # the day after the last command
//! final_hash = "0x…"
//! ```

#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

pub mod bench;
pub mod cli;

use std::fmt;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use h3_adventure::{AdventureCommand, Pos};
use h3_contracts::{HeroId, PlayerId};
use h3_game::{Answer, GameCommand, GameRuleset, GameSetup};
use h3_sim::adventure::replay::{
    MapRef, RecordError, RecordOptions, Replay, ReplayError, ReplayHeader, map_content_hash,
};
use serde::{Deserialize, Serialize};

/// The packs an adventure game loads by default: the original three, RoE to SoD.
pub const DEFAULT_PACKS: [&str; 3] = ["base", "ab", "sod"];

/// The default profile.
pub const DEFAULT_PROFILE: &str = "base:faithful";

/// Errors of the adventure commands.
#[derive(Debug)]
pub enum AdventureCliError {
    /// File system error.
    Io(PathBuf, std::io::Error),
    /// A pack, its adventure file or the profile does not compile.
    Pack(PathBuf, String),
    /// The map file does not read or import.
    Map(PathBuf, String),
    /// The replay file is malformed.
    Format(String),
    /// The replay does not verify.
    Replay(ReplayError),
    /// The setup does not start a game.
    Setup(h3_game::GameSetupError),
    /// A game could not be recorded.
    Record(RecordError),
}

impl fmt::Display for AdventureCliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(p, e) => write!(f, "{}: {e}", p.display()),
            Self::Pack(p, e) | Self::Map(p, e) => write!(f, "{}: {e}", p.display()),
            Self::Format(e) => write!(f, "replay format: {e}"),
            Self::Replay(e) => write!(f, "{e}"),
            Self::Setup(e) => write!(f, "setup: {e}"),
            Self::Record(e) => write!(f, "record: {e}"),
        }
    }
}

impl std::error::Error for AdventureCliError {}

type Result<T> = std::result::Result<T, AdventureCliError>;

fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|e| AdventureCliError::Io(path.to_path_buf(), e))
}

fn read_text(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|e| AdventureCliError::Io(path.to_path_buf(), e))
}

/// Loads `<packs_dir>/<id>/pack.toml` and, where it exists, `<id>/adventure.toml` for each id and
/// compiles them with `profile` into the game's rules. Ids come from replay files, so they must be
/// valid pack ids (no paths).
///
/// # Errors
///
/// An invalid id, an unreadable file, or the packs' compile error.
pub fn load_rules(packs_dir: &Path, ids: &[String], profile: &str) -> Result<GameRuleset> {
    let mut texts = Vec::new();
    for id in ids {
        h3_content::ContentKey::new(id, "pack")
            .map_err(|_| AdventureCliError::Format(format!("invalid pack id `{id}`")))?;
        let dir = packs_dir.join(id);
        let adventure = dir.join("adventure.toml");
        let adventure = if adventure.is_file() {
            read_text(&adventure)?
        } else {
            String::new()
        };
        texts.push((read_text(&dir.join("pack.toml"))?, adventure));
    }
    let refs: Vec<(&str, &str)> = texts
        .iter()
        .map(|(p, a)| (p.as_str(), a.as_str()))
        .collect();
    let pack_error = |e: String| AdventureCliError::Pack(packs_dir.to_path_buf(), e);
    let (core, part) =
        h3_pack_adventure::compile(&refs, profile).map_err(|e| pack_error(e.to_string()))?;
    GameRuleset::new(Arc::new(core), Arc::new(part)).map_err(|e| pack_error(e.to_string()))
}

/// Checks that a map path from the command line or a replay file is relative and stays below the
/// directory it is resolved against (a replay file is untrusted input).
fn check_map_path(path: &str) -> Result<()> {
    let ok = !path.is_empty()
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir));
    if ok {
        Ok(())
    } else {
        Err(AdventureCliError::Format(format!(
            "map path `{path}` must be relative, without `..`"
        )))
    }
}

/// A map file below `root`: its bytes and how a replay names it. The path must be relative and
/// stay below `root` once links are resolved, and the file within `h3-map`'s size limit, checked
/// before it is read (a replay file is untrusted input).
///
/// # Errors
///
/// A path that is not relative or leaves `root`, an unreadable or too large file.
pub fn read_map(root: &Path, path: &str) -> Result<(Vec<u8>, MapRef)> {
    check_map_path(path)?;
    let joined = root.join(path);
    let canonical = |p: &Path| {
        p.canonicalize()
            .map_err(|e| AdventureCliError::Io(p.to_path_buf(), e))
    };
    let file = canonical(&joined)?;
    if !file.starts_with(canonical(root)?) {
        return Err(AdventureCliError::Format(format!(
            "map path `{path}` leads outside {}",
            root.display()
        )));
    }
    let max = h3_map::Limits::default().max_file_bytes;
    let size = std::fs::metadata(&file)
        .map_err(|e| AdventureCliError::Io(file.clone(), e))?
        .len();
    if usize::try_from(size).map_or(true, |size| size > max) {
        return Err(AdventureCliError::Map(
            joined,
            format!("{size} bytes, over the limit of {max}"),
        ));
    }
    let bytes = read_bytes(&file)?;
    let map = MapRef {
        path: path.to_owned(),
        content_hash: map_content_hash(&bytes),
        import_version: h3_map::MAP_IMPORT_VERSION,
    };
    Ok((bytes, map))
}

/// Reads a map file's bytes ([`read_map`]) and imports them with `rules`.
fn import_map(root: &Path, path: &str, bytes: &[u8], rules: &GameRuleset) -> Result<GameSetup> {
    let map_error = |e: String| AdventureCliError::Map(root.join(path), e);
    let raw =
        h3_map::read(bytes, &h3_map::Limits::default()).map_err(|e| map_error(e.to_string()))?;
    h3_map::import(&raw, rules).map_err(|e| map_error(e.to_string()))
}

/// Reads the map at `path` (relative to `root`, [`read_map`]) and imports it with `rules`: how a
/// replay names it, and the game's setup.
///
/// # Errors
///
/// [`read_map`]'s errors, or the map's read or import error.
pub fn load_map(root: &Path, path: &str, rules: &GameRuleset) -> Result<(MapRef, GameSetup)> {
    let (bytes, map) = read_map(root, path)?;
    let setup = import_map(root, path, &bytes, rules)?;
    Ok((map, setup))
}

fn hex64(v: u64) -> String {
    format!("{v:#018x}")
}

fn parse_hex64(s: &str) -> Result<u64> {
    s.strip_prefix("0x")
        .and_then(|h| u64::from_str_radix(h, 16).ok())
        .ok_or_else(|| AdventureCliError::Format(format!("bad hash `{s}`")))
}

/// A command as a replay file writes it (format v1, the module's docs).
pub fn format_command(player: PlayerId, command: &GameCommand) -> String {
    match command {
        GameCommand::EndTurn => format!("{} end-turn", player.raw()),
        // E08 T14b: an answer to a pending decision; the format's version follows with the env's
        // action encoding v2, which first records one.
        GameCommand::Answer(Answer::Accept) => format!("{} accept", player.raw()),
        GameCommand::Answer(Answer::Decline) => format!("{} decline", player.raw()),
        GameCommand::Adventure(AdventureCommand::MoveHero { hero, path }) => {
            let mut text = format!("{} move {}", player.raw(), hero.raw());
            for pos in path {
                text.push_str(&format!(" {},{},{}", pos.x, pos.y, pos.level));
            }
            text
        }
    }
}

/// Parses a command written by [`format_command`].
///
/// # Errors
///
/// [`AdventureCliError::Format`] naming the text.
pub fn parse_command(text: &str) -> Result<(PlayerId, GameCommand)> {
    let bad = || AdventureCliError::Format(format!("bad command `{text}`"));
    let mut words = text.split_whitespace();
    let player = words
        .next()
        .and_then(|w| w.parse().ok())
        .map(PlayerId::new)
        .ok_or_else(bad)?;
    let command = match words.next() {
        Some("end-turn") => GameCommand::EndTurn,
        Some("accept") => GameCommand::Answer(Answer::Accept),
        Some("decline") => GameCommand::Answer(Answer::Decline),
        Some("move") => {
            let hero = words
                .next()
                .and_then(|w| w.parse().ok())
                .map(HeroId::new)
                .ok_or_else(bad)?;
            let path = words
                .by_ref()
                .map(|w| {
                    let mut n = w.split(',');
                    let (x, y, level) = (n.next()?, n.next()?, n.next()?);
                    if n.next().is_some() {
                        return None;
                    }
                    Some(Pos::new(
                        x.parse().ok()?,
                        y.parse().ok()?,
                        level.parse().ok()?,
                    ))
                })
                .collect::<Option<Vec<_>>>()
                .ok_or_else(bad)?;
            GameCommand::Adventure(AdventureCommand::MoveHero { hero, path })
        }
        _ => return Err(bad()),
    };
    if words.next().is_some() {
        return Err(bad());
    }
    Ok((player, command))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplayFile {
    commands: Vec<String>,
    checkpoints: Vec<String>,
    replay: Meta,
    map: MapEntry,
    result: ResultEntry,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Meta {
    format: u32,
    engine: String,
    seed: String,
    packs: Vec<[String; 2]>,
    profile: String,
    ruleset_hash: String,
    adventure_ruleset_hash: String,
    rng: String,
    rng_version: u32,
    hash: String,
    hash_version: u32,
    game_state_hash_version: u32,
    adventure_state_hash_version: u32,
    checkpoint_every: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MapEntry {
    path: String,
    hash: String,
    import_version: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResultEntry {
    day: u32,
    final_hash: String,
}

/// The replay as TOML (format v1).
///
/// # Errors
///
/// The serializer's error (none for a replay this tool recorded).
pub fn replay_to_toml(replay: &Replay) -> Result<String> {
    let h = &replay.header;
    let file = ReplayFile {
        commands: replay
            .commands
            .iter()
            .map(|(p, c)| format_command(*p, c))
            .collect(),
        checkpoints: replay.checkpoints.iter().map(|h| hex64(*h)).collect(),
        replay: Meta {
            format: h.format_version,
            engine: h.engine_version.clone(),
            seed: replay.seed.to_string(),
            packs: h
                .packs
                .iter()
                .map(|(id, v)| [id.clone(), v.clone()])
                .collect(),
            profile: h.profile.clone(),
            ruleset_hash: hex64(h.ruleset_hash),
            adventure_ruleset_hash: hex64(h.adventure_ruleset_hash),
            rng: h.rng_algorithm.clone(),
            rng_version: h.rng_version,
            hash: h.hash_algorithm.clone(),
            hash_version: h.hash_version,
            game_state_hash_version: h.game_state_hash_version,
            adventure_state_hash_version: h.adventure_state_hash_version,
            checkpoint_every: replay.checkpoint_every,
        },
        map: MapEntry {
            path: replay.map.path.clone(),
            hash: hex64(replay.map.content_hash),
            import_version: replay.map.import_version,
        },
        result: ResultEntry {
            day: replay.final_day,
            final_hash: hex64(replay.final_hash),
        },
    };
    toml::to_string_pretty(&file).map_err(|e| AdventureCliError::Format(e.to_string()))
}

/// Parses a replay file (format v1). Its rules and map are loaded by [`verify_file`].
///
/// # Errors
///
/// [`AdventureCliError::Format`]: malformed TOML, an unknown key, a bad number, hash or command.
pub fn parse_replay(text: &str) -> Result<Replay> {
    let f: ReplayFile =
        toml::from_str(text).map_err(|e| AdventureCliError::Format(e.to_string()))?;
    let m = f.replay;
    Ok(Replay {
        header: ReplayHeader {
            format_version: m.format,
            engine_version: m.engine,
            packs: m.packs.into_iter().map(|[id, v]| (id, v)).collect(),
            profile: m.profile,
            ruleset_hash: parse_hex64(&m.ruleset_hash)?,
            adventure_ruleset_hash: parse_hex64(&m.adventure_ruleset_hash)?,
            rng_algorithm: m.rng,
            rng_version: m.rng_version,
            hash_algorithm: m.hash,
            hash_version: m.hash_version,
            game_state_hash_version: m.game_state_hash_version,
            adventure_state_hash_version: m.adventure_state_hash_version,
        },
        map: MapRef {
            path: f.map.path,
            content_hash: parse_hex64(&f.map.hash)?,
            import_version: f.map.import_version,
        },
        seed: m
            .seed
            .parse()
            .map_err(|_| AdventureCliError::Format(format!("bad seed `{}`", m.seed)))?,
        commands: f
            .commands
            .iter()
            .map(|c| parse_command(c))
            .collect::<Result<_>>()?,
        checkpoint_every: m.checkpoint_every,
        checkpoints: f
            .checkpoints
            .iter()
            .map(|h| parse_hex64(h))
            .collect::<Result<_>>()?,
        final_day: f.result.day,
        final_hash: parse_hex64(&f.result.final_hash)?,
    })
}

/// Result of verifying a replay file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verified {
    /// The map's path.
    pub map: String,
    /// Number of commands.
    pub commands: usize,
    /// The final day.
    pub day: u32,
    /// Final state hash.
    pub final_hash: u64,
}

/// Loads a replay file, its packs from `packs_dir` and its map below `root`, and verifies it.
///
/// # Errors
///
/// The first [`AdventureCliError`].
pub fn verify_file(path: &Path, packs_dir: &Path, root: &Path) -> Result<Verified> {
    let replay = parse_replay(&read_text(path)?)?;
    let ids: Vec<String> = replay
        .header
        .packs
        .iter()
        .map(|(id, _)| id.clone())
        .collect();
    let rules = load_rules(packs_dir, &ids, &replay.header.profile)?;
    // The header first: a replay of other rules is refused before its map is read.
    replay
        .header
        .check(&rules)
        .map_err(AdventureCliError::Replay)?;
    // The map by its hash before its bytes are parsed: an untrusted file is read only when it is
    // the one the replay names.
    let (bytes, map) = read_map(root, &replay.map.path)?;
    if map.content_hash != replay.map.content_hash {
        return Err(AdventureCliError::Replay(ReplayError::Incompatible(
            "map.hash",
        )));
    }
    let setup = import_map(root, &replay.map.path, &bytes, &rules)?;
    replay
        .verify(&rules, &map, &setup)
        .map_err(AdventureCliError::Replay)?;
    Ok(Verified {
        map: replay.map.path,
        commands: replay.commands.len(),
        day: replay.final_day,
        final_hash: replay.final_hash,
    })
}

/// What [`run_random`] plays.
#[derive(Debug, Clone)]
pub struct RunOptions {
    /// The map's path, relative to the root.
    pub map: String,
    /// The profile key.
    pub profile: String,
    /// The game and the agent ([`RecordOptions`]).
    pub record: RecordOptions,
}

/// Plays a game of uniformly random legal agents on a map and returns the replay and its TOML.
///
/// # Errors
///
/// The first [`AdventureCliError`].
pub fn run_random(packs_dir: &Path, root: &Path, options: &RunOptions) -> Result<(Replay, String)> {
    let ids: Vec<String> = DEFAULT_PACKS.iter().map(|&s| s.to_owned()).collect();
    let rules = load_rules(packs_dir, &ids, &options.profile)?;
    let (map, setup) = load_map(root, &options.map, &rules)?;
    let replay = Replay::record_random(&rules, Arc::new(setup), map, options.record)
        .map_err(AdventureCliError::Record)?;
    let text = replay_to_toml(&replay)?;
    Ok((replay, text))
}

#[cfg(test)]
mod tests;
