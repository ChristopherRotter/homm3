//! Adventure replays (ADR-0004 point 9, ADR-0015 "a replay names its map", E08 T9b): the rules'
//! identity, the map by content hash and path plus the import version (ADV-Q2), the seed and the
//! game's difficulty (format v2, E08 T14b), the game commands in the authoritative order with the
//! acting player (answers to decisions included), and state hashes.
//!
//! A replay records **game commands**, not the env's actions (the user, 2026-10-11): what the game
//! accepts from any source (the env, a server game, a client with a path of its own), so the
//! same verifier serves them all and a replay survives action-encoding bumps. Verifying re-imports
//! the map (the caller's job: this crate does not read maps) and re-applies every command through
//! `h3-game`, the single validated mutation path. The on-disk format is the CLI's business.

use std::fmt;
use std::sync::Arc;

use h3_contracts::PlayerId;
use h3_game::{
    Actor, GAME_STATE_HASH_VERSION, GameCommand, GameCommandError, GameRuleset, GameSetup,
    GameSetupError, GameState, NoopSink, apply,
};
use h3_kernel::hash::{HASH_ALGORITHM, HASH_VERSION, StateHasher};
use h3_kernel::rng::{RNG_ALGORITHM, RNG_VERSION};
use h3_kernel::{Rng, StreamId};

use super::env::{AdventureEnv, StepError};
use super::rollout::choose;

/// Version of the adventure replay model (bump when fields change meaning). Owned by the
/// adventure lane (`docs/08-work-process.md`, the versions table). Version 2 (E08 T14b): the
/// game's difficulty and the answers to decisions; a replay of another version is refused
/// ([`ReplayHeader::check`]).
pub const REPLAY_FORMAT_VERSION: u32 = 2;

/// Identity of the rules and algorithms a replay depends on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayHeader {
    /// [`REPLAY_FORMAT_VERSION`] at recording time.
    pub format_version: u32,
    /// Engine version at recording time (informational).
    pub engine_version: String,
    /// Packs as `(id, version)`, in load order.
    pub packs: Vec<(String, String)>,
    /// Profile key.
    pub profile: String,
    /// The core ruleset's content hash.
    pub ruleset_hash: u64,
    /// The adventure part's content hash (ADR-0015 point 2: every part's hash).
    pub adventure_ruleset_hash: u64,
    /// PRNG algorithm id.
    pub rng_algorithm: String,
    /// PRNG version.
    pub rng_version: u32,
    /// State-hash algorithm id.
    pub hash_algorithm: String,
    /// State-hash scheme version.
    pub hash_version: u32,
    /// [`GAME_STATE_HASH_VERSION`].
    pub game_state_hash_version: u32,
    /// [`h3_adventure::ADVENTURE_STATE_HASH_VERSION`].
    pub adventure_state_hash_version: u32,
}

impl ReplayHeader {
    /// The header for recording with `rules` in this build.
    pub fn current(rules: &GameRuleset) -> Self {
        Self {
            format_version: REPLAY_FORMAT_VERSION,
            engine_version: env!("CARGO_PKG_VERSION").to_owned(),
            packs: rules.core().packs().to_vec(),
            profile: rules.core().profile().as_str().to_owned(),
            ruleset_hash: rules.core().content_hash(),
            adventure_ruleset_hash: rules.adventure().content_hash(),
            rng_algorithm: RNG_ALGORITHM.to_owned(),
            rng_version: RNG_VERSION,
            hash_algorithm: HASH_ALGORITHM.to_owned(),
            hash_version: HASH_VERSION,
            game_state_hash_version: GAME_STATE_HASH_VERSION,
            adventure_state_hash_version: h3_adventure::ADVENTURE_STATE_HASH_VERSION,
        }
    }

    /// Checks that this build and `rules` can verify a replay with this header.
    ///
    /// # Errors
    ///
    /// [`ReplayError::Incompatible`] naming the first field that differs.
    pub fn check(&self, rules: &GameRuleset) -> Result<(), ReplayError> {
        let now = Self::current(rules);
        let fields: [(&'static str, bool); 8] = [
            ("format_version", self.format_version == now.format_version),
            ("packs", self.packs == now.packs),
            ("profile", self.profile == now.profile),
            ("ruleset_hash", self.ruleset_hash == now.ruleset_hash),
            (
                "adventure_ruleset_hash",
                self.adventure_ruleset_hash == now.adventure_ruleset_hash,
            ),
            (
                "rng",
                self.rng_algorithm == now.rng_algorithm && self.rng_version == now.rng_version,
            ),
            (
                "hash",
                self.hash_algorithm == now.hash_algorithm && self.hash_version == now.hash_version,
            ),
            (
                "state_hash_versions",
                self.game_state_hash_version == now.game_state_hash_version
                    && self.adventure_state_hash_version == now.adventure_state_hash_version,
            ),
        ];
        match fields.iter().find(|(_, ok)| !ok) {
            Some((field, _)) => Err(ReplayError::Incompatible(field)),
            None => Ok(()),
        }
    }
}

/// How a replay names its map (ADV-Q2): the file's path (a committed fixture, or the server's
/// copy beside its journal), the file's content hash ([`map_content_hash`]) and `h3-map`'s
/// import version. The map itself is never embedded (ADR-0015, alternatives).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapRef {
    /// Where the file is, relative to the directory the tool resolves it against.
    pub path: String,
    /// [`map_content_hash`] of the file's bytes.
    pub content_hash: u64,
    /// `h3_map::MAP_IMPORT_VERSION` at recording time.
    pub import_version: u32,
}

/// The content hash of a map file's bytes (the kernel's state hash over the raw bytes, so the
/// header's `hash` fields name its algorithm too).
pub fn map_content_hash(bytes: &[u8]) -> u64 {
    let mut h = StateHasher::new();
    h.write_len(bytes.len());
    h.write_bytes(bytes);
    h.finish()
}

/// A recorded adventure game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replay {
    /// What the replay depends on.
    pub header: ReplayHeader,
    /// The map.
    pub map: MapRef,
    /// The game's seed.
    pub seed: u64,
    /// The game's difficulty (`GameSetup::difficulty`, E08 T14b): verifying builds the game at
    /// it, whatever the map's import sets.
    pub difficulty: u8,
    /// Accepted commands in order, with the acting player.
    pub commands: Vec<(PlayerId, GameCommand)>,
    /// A checkpoint hash is recorded after every `checkpoint_every` commands (0 = none).
    pub checkpoint_every: u32,
    /// State hashes after commands `k · checkpoint_every`, k = 1, 2, …
    pub checkpoints: Vec<u64>,
    /// The day after the last command.
    pub final_day: u32,
    /// State hash after the last command.
    pub final_hash: u64,
}

/// Why a replay does not verify.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayError {
    /// The header or the map does not match this build, the rules or the map file (field name).
    Incompatible(&'static str),
    /// The setup does not start a game.
    Setup(GameSetupError),
    /// A recorded command was rejected.
    Rejected {
        /// Index of the command.
        step: usize,
        /// Why.
        error: GameCommandError,
    },
    /// A state hash differs.
    HashMismatch {
        /// Commands applied when the hash was taken.
        step: usize,
        /// Recorded hash.
        expected: u64,
        /// Recomputed hash.
        actual: u64,
    },
    /// The number of checkpoints does not match the command count.
    CheckpointCount,
    /// The final day differs.
    DayMismatch {
        /// Recorded day.
        expected: u32,
        /// Replayed day.
        actual: u32,
    },
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Incompatible(field) => write!(f, "replay header mismatch: {field}"),
            Self::Setup(e) => write!(f, "setup: {e}"),
            Self::Rejected { step, error } => write!(f, "command {step} rejected: {error}"),
            Self::HashMismatch {
                step,
                expected,
                actual,
            } => write!(
                f,
                "state hash after {step} commands: expected {expected:#018x}, got {actual:#018x}"
            ),
            Self::CheckpointCount => {
                f.write_str("checkpoint count does not match the command count")
            }
            Self::DayMismatch { expected, actual } => {
                write!(f, "final day: expected {expected}, got {actual}")
            }
        }
    }
}

impl std::error::Error for ReplayError {}

/// Why [`Replay::record_random`] recorded nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordError {
    /// The setup does not start a game.
    Setup(GameSetupError),
    /// Neither a day limit nor a step limit: increment 1's games never end on their own.
    NoLimit,
    /// The game refused the agent's legal action (a bug: never a truncated record).
    Refused {
        /// Commands recorded before it.
        step: usize,
        /// Why.
        error: StepError,
    },
}

impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Setup(e) => write!(f, "setup: {e}"),
            Self::NoLimit => f.write_str("a game ends only by a limit: give a day or step limit"),
            Self::Refused { step, error } => {
                write!(f, "the random agent's action {step} was refused: {error}")
            }
        }
    }
}

impl std::error::Error for RecordError {}

/// What [`Replay::record_random`] plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordOptions {
    /// The game's seed.
    pub seed: u64,
    /// The random agent's seed (its AGENT stream, never the game's streams).
    pub agent_seed: u64,
    /// The env's day limit: the game stops when day `max_days + 1` starts (0 = none).
    pub max_days: u32,
    /// The most commands to play (0 = none).
    pub max_steps: u32,
    /// Checkpoint interval (0 = none).
    pub checkpoint_every: u32,
}

impl Replay {
    /// Records a game on `map` played by the uniformly random legal agent
    /// ([`super::rollout::choose`]) for every player, as [`super::random_rollout`] plays it, at
    /// the setup's difficulty. Increment 1's games end only by a limit: give `max_days` or
    /// `max_steps`.
    ///
    /// # Errors
    ///
    /// A [`RecordError`]: the setup's, no limit, or a refused action.
    pub fn record_random(
        rules: &GameRuleset,
        setup: Arc<GameSetup>,
        map: MapRef,
        options: RecordOptions,
    ) -> Result<Self, RecordError> {
        if options.max_days == 0 && options.max_steps == 0 {
            return Err(RecordError::NoLimit);
        }
        let mut env = AdventureEnv::new(rules.clone(), setup, options.seed, options.max_days)
            .map_err(RecordError::Setup)?;
        let mut rng = Rng::for_stream(options.agent_seed, StreamId::AGENT);
        let (mut commands, mut checkpoints) = (Vec::new(), Vec::new());
        loop {
            let Some(player) = env.to_move().next() else {
                break;
            };
            if options.max_steps != 0 && commands.len() >= options.max_steps as usize {
                break;
            }
            let action = choose(&env, player, &mut rng);
            // A legal action is always accepted (`tests.rs`): a refusal is a bug, reported.
            let (_, command) =
                env.step_decoded(player, action, &mut NoopSink)
                    .map_err(|error| RecordError::Refused {
                        step: commands.len(),
                        error,
                    })?;
            commands.push((player, command));
            let every = options.checkpoint_every as usize;
            if every != 0 && commands.len() % every == 0 {
                checkpoints.push(env.state_hash());
            }
        }
        Ok(Self {
            header: ReplayHeader::current(rules),
            map,
            seed: options.seed,
            difficulty: env.difficulty(),
            commands,
            checkpoint_every: options.checkpoint_every,
            checkpoints,
            final_day: env.day(),
            final_hash: env.state_hash(),
        })
    }

    /// Re-runs the replay through `h3-game` and compares every recorded hash and the final day.
    /// `map` names the file the caller loaded (its content hash and this build's import version)
    /// and `setup` is that file's import; the game is built at the replay's difficulty.
    ///
    /// # Errors
    ///
    /// The first [`ReplayError`].
    pub fn verify(
        &self,
        rules: &GameRuleset,
        map: &MapRef,
        setup: &GameSetup,
    ) -> Result<(), ReplayError> {
        self.header.check(rules)?;
        if self.map.content_hash != map.content_hash {
            return Err(ReplayError::Incompatible("map.hash"));
        }
        if self.map.import_version != map.import_version {
            return Err(ReplayError::Incompatible("map.import_version"));
        }
        let every = self.checkpoint_every as usize;
        let expected_checkpoints = self.commands.len().checked_div(every).unwrap_or(0);
        if self.checkpoints.len() != expected_checkpoints {
            return Err(ReplayError::CheckpointCount);
        }
        let setup = GameSetup {
            difficulty: self.difficulty,
            ..setup.clone()
        };
        let mut state = GameState::new(rules, &setup, self.seed).map_err(ReplayError::Setup)?;
        let mut next_checkpoint = self.checkpoints.iter();
        for (i, (player, command)) in self.commands.iter().enumerate() {
            apply(
                &mut state,
                rules,
                Actor::Player(*player),
                command.clone(),
                &mut NoopSink,
            )
            .map_err(|error| ReplayError::Rejected { step: i, error })?;
            let step = i + 1;
            if every != 0 && step % every == 0 {
                let expected = next_checkpoint.next().copied().unwrap_or_default();
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
        if actual != self.final_hash {
            return Err(ReplayError::HashMismatch {
                step: self.commands.len(),
                expected: self.final_hash,
                actual,
            });
        }
        if state.day() != self.final_day {
            return Err(ReplayError::DayMismatch {
                expected: self.final_day,
                actual: state.day(),
            });
        }
        Ok(())
    }
}
