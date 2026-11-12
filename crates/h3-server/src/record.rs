//! `h3-server record` (E16 M5 T6; the map client lane's E24 M4 T3, T4, ADR-0020 point 8): an
//! adventure replay run through the whole game's session in-process
//! (`h3_protocol::session::game`), so that each player's record is what a client hosted by the
//! server would have been sent. A record file is the record format's (v4): its first line the
//! [`GameRecordHeader`](h3_protocol::wire::game::GameRecordHeader) (the game's manifest, the map
//! by its file's hash and import version, the player), then one numbered server message a line,
//! every one the player was sent from its `GameSnapshot` on.
//!
//! - **The replay is checked as `h3 adventure verify` checks it**: its header against the rules,
//!   its map by its hash before the file is parsed (a replay is untrusted input), then every
//!   command through the session as its actor's: each checkpoint, the final hash and day equal
//!   to the replay's, or there is no record.
//! - **The players are the host's seats**: a player's command is played as that player's
//!   (`GameSession::play`), numbered by its own counter from 1, as a client numbers its
//!   commands; the system actor's goes through `GameSession::system`.
//! - **A record holds every message** (unbounded): it is the operator's own file, of a replay
//!   that was played to its end.
//!
//! Provisional (E16 M5 T6; the epic's review list): the command and its options, the files'
//! names (`<stem>.player-<k>.jsonl`), the game's id 0 by default, the player's own command
//! numbers, the unbounded records.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use h3_contracts::PlayerId;
use h3_game::{Actor, GameSetup};
use h3_protocol::map::game::{command, manifest};
use h3_protocol::session::game::{GameEntry, GameOptions, GameSession};
use h3_protocol::wire::WireU64;
use h3_protocol::wire::game::MapIdentity;
use h3_sim::adventure::replay::ReplayError;

use crate::Error;

/// A replay run through the session: what it was and each player's record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    /// The map's path, as the replay names it.
    pub map: String,
    /// The replay's commands.
    pub commands: usize,
    /// The day after the last command.
    pub day: u32,
    /// The state's hash after the last command.
    pub final_hash: u64,
    /// Each player's record, by player index: its header line, then every message it was sent,
    /// each line ending in a line break.
    pub records: Vec<String>,
}

/// Runs the adventure replay at `path` through the whole game's session as game `game`, its
/// packs from `packs_dir`, its map below `root`, and gives each player's record.
///
/// # Errors
///
/// The replay's file, its packs or its map cannot be read; its header or map is not this build's
/// or not the file's; a command is rejected; a checkpoint, the final hash or the final day is not
/// the session's.
pub fn record_replay(
    path: &Path,
    packs_dir: &Path,
    root: &Path,
    game: u32,
) -> Result<Recorded, Error> {
    let failed = |e: &dyn std::fmt::Display| Error(format!("{}: {e}", path.display()));
    let text = fs::read_to_string(path).map_err(|e| failed(&e))?;
    let replay = h3_files::adventure::parse_replay(&text).map_err(|e| failed(&e))?;
    let ids: Vec<String> = replay
        .header
        .packs
        .iter()
        .map(|(id, _)| id.clone())
        .collect();
    let rules = h3_files::adventure::load_rules(packs_dir, &ids, &replay.header.profile)
        .map_err(|e| failed(&e))?;
    // The header first: a replay of other rules is refused before its map is read.
    replay.header.check(&rules).map_err(|e| failed(&e))?;
    // The map by its hash before its bytes are parsed: an untrusted file is read only when it is
    // the one the replay names.
    let (bytes, map) =
        h3_files::adventure::read_map(root, &replay.map.path).map_err(|e| failed(&e))?;
    for (field, same) in [
        ("map.hash", map.content_hash == replay.map.content_hash),
        (
            "map.import_version",
            map.import_version == replay.map.import_version,
        ),
    ] {
        if !same {
            return Err(failed(&ReplayError::Incompatible(field)));
        }
    }
    let every = usize::try_from(replay.checkpoint_every).unwrap_or(usize::MAX);
    if replay.checkpoints.len() != replay.commands.len().checked_div(every).unwrap_or(0) {
        return Err(failed(&ReplayError::CheckpointCount));
    }
    let map_failed = |e: &dyn std::fmt::Display| failed(&format_args!("map `{}`: {e}", map.path));
    let raw = h3_map::read(&bytes, &h3_map::Limits::default()).map_err(|e| map_failed(&e))?;
    let setup = h3_map::import(&raw, &rules).map_err(|e| map_failed(&e))?;
    let setup = GameSetup {
        difficulty: replay.difficulty,
        ..setup
    };
    let identity = MapIdentity {
        content_hash: WireU64(map.content_hash),
        import_version: map.import_version,
    };
    let options = GameOptions {
        game,
        checkpoint_every: replay.checkpoint_every,
        record_bytes: usize::MAX,
    };
    // Every frame stays in the session's records; the host delivers none here.
    let mut out = Vec::new();
    let mut session = GameSession::start(
        &rules,
        &setup,
        replay.seed,
        manifest(&rules, identity),
        options,
        &mut out,
    )
    .map_err(|e| failed(&ReplayError::Setup(e)))?;
    let players = session.state().players();
    let mut seqs = vec![0_u32; players];
    let mut checkpoints = replay.checkpoints.iter().enumerate();
    for (i, (actor, engine)) in replay.commands.iter().enumerate() {
        out.clear();
        let wire = command(engine);
        let played = match *actor {
            Actor::Player(player) => {
                let seq = seqs.get_mut(player.index()).map_or(0, |seq| {
                    *seq = seq.saturating_add(1);
                    *seq
                });
                session.play(player, seq, wire, &mut out)
            }
            Actor::System => session.system(wire, &mut out),
        };
        played.map_err(|code| failed(&format_args!("command {i} rejected: {code:?}")))?;
        for entry in session.drain_entries() {
            if let GameEntry::Checkpoint(actual) = entry {
                let (k, &expected) = checkpoints.next().unwrap_or((0, &0));
                if actual != expected {
                    return Err(failed(&format_args!(
                        "checkpoint {k} after command {i}: the replay's {expected:#018x}, the session's {actual:#018x}"
                    )));
                }
            }
        }
    }
    let (final_hash, day) = (session.state().state_hash(), session.state().day());
    if final_hash != replay.final_hash {
        return Err(failed(&ReplayError::HashMismatch {
            step: replay.commands.len(),
            expected: replay.final_hash,
            actual: final_hash,
        }));
    }
    if day != replay.final_day {
        return Err(failed(&ReplayError::DayMismatch {
            expected: replay.final_day,
            actual: day,
        }));
    }
    let records = (0..=u8::MAX)
        .take(players)
        .filter_map(|p| session.record(PlayerId::new(p)))
        .collect();
    Ok(Recorded {
        map: replay.map.path,
        commands: replay.commands.len(),
        day,
        final_hash,
        records,
    })
}

/// `h3-server record`'s options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordOptions {
    /// The replays.
    pub replays: Vec<PathBuf>,
    /// The packs' directory.
    pub packs_dir: PathBuf,
    /// The directory the replays' map paths are below.
    pub root: PathBuf,
    /// Where the records go.
    pub out: PathBuf,
    /// The game's id in the records' headers.
    pub game: u32,
}

/// `h3-server record`'s usage.
pub const RECORD_USAGE: &str =
    "h3-server record [--packs-dir packs] [--root .] [--out .] [--game 0] <replay.toml>...";

impl RecordOptions {
    /// Parses `[--packs-dir packs] [--root .] [--out .] [--game 0] <replay.toml>...` (the
    /// arguments after `record`).
    ///
    /// # Errors
    ///
    /// An unknown option, one without a value, a game id that is not a number, no replay, two
    /// replays of one file name (their records would share files; E16 M5 T7b's review).
    pub fn parse(args: &[OsString]) -> Result<Self, Error> {
        let mut options = Self {
            replays: Vec::new(),
            packs_dir: PathBuf::from("packs"),
            root: PathBuf::from("."),
            out: PathBuf::from("."),
            game: 0,
        };
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            // A path as given, also one that is not UTF-8.
            let Some(name) = arg.to_str().and_then(|a| a.strip_prefix("--")) else {
                options.replays.push(PathBuf::from(arg));
                continue;
            };
            let value = rest
                .next()
                .ok_or_else(|| Error(format!("`--{name}` needs a value")))?;
            match name {
                "packs-dir" => options.packs_dir = PathBuf::from(value),
                "root" => options.root = PathBuf::from(value),
                "out" => options.out = PathBuf::from(value),
                "game" => {
                    options.game =
                        value.to_str().and_then(|v| v.parse().ok()).ok_or_else(|| {
                            Error(format!("bad game id `{}`", value.to_string_lossy()))
                        })?;
                }
                _ => return Err(Error(format!("unknown option `--{name}`"))),
            }
        }
        if options.replays.is_empty() {
            return Err(Error("no replay given".to_owned()));
        }
        let mut stems = BTreeSet::new();
        for replay in &options.replays {
            if !stems.insert(stem(replay)) {
                return Err(Error(format!(
                    "two replays named `{}`: their records would share files",
                    stem(replay)
                )));
            }
        }
        Ok(options)
    }
}

/// Records each replay of `options` and writes each player's record to
/// `<out>/<stem>.player-<k>.jsonl`; gives one line per replay for standard output.
///
/// # Errors
///
/// The first replay that does not record ([`record_replay`]) or a record that cannot be written.
pub fn record(options: &RecordOptions) -> Result<Vec<String>, Error> {
    let mut lines = Vec::new();
    for replay in &options.replays {
        let recorded = record_replay(replay, &options.packs_dir, &options.root, options.game)?;
        let stem = stem(replay);
        let mut files = Vec::new();
        for (k, text) in recorded.records.iter().enumerate() {
            let file = options.out.join(format!("{stem}.player-{k}.jsonl"));
            fs::write(&file, text).map_err(|e| Error(format!("{}: {e}", file.display())))?;
            files.push(file.display().to_string());
        }
        lines.push(format!(
            "{}: {} ({} commands, day {}, final hash {:#018x}): {}",
            replay.display(),
            recorded.map,
            recorded.commands,
            recorded.day,
            recorded.final_hash,
            files.join(", ")
        ));
    }
    Ok(lines)
}

/// The name a replay's records take after it: its file's stem.
fn stem(replay: &Path) -> String {
    replay
        .file_stem()
        .map_or_else(|| "replay".to_owned(), |s| s.to_string_lossy().into_owned())
}
