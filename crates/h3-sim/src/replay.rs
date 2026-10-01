//! Replays (ADR-0004 §9): ruleset identity + scenario + seed + command log + state hashes.
//!
//! A replay verifies only against the exact ruleset it was recorded with (same content hash) and
//! the same algorithm versions (RNG, hashing, battle-state encoding): the header records them and
//! [`ReplayHeader::check`] refuses anything else. The on-disk format is the CLI's business.

use std::fmt;

use h3_battle::{
    BATTLE_STATE_HASH_VERSION, BattleState, Command, CommandError, LegalCommands, NoopSink,
    SetupError, apply,
};
use h3_content::Ruleset;
use h3_contracts::{BattleSetup, BattleSide};
use h3_kernel::hash::{HASH_ALGORITHM, HASH_VERSION};
use h3_kernel::rng::{RNG_ALGORITHM, RNG_VERSION};
use h3_kernel::{Rng, StreamId};

/// Version of the replay model (bump when fields change meaning).
pub const REPLAY_FORMAT_VERSION: u32 = 2;

/// Identity of everything a replay depends on.
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
    /// `Ruleset::content_hash` of the compiled ruleset.
    pub ruleset_hash: u64,
    /// PRNG algorithm id.
    pub rng_algorithm: String,
    /// PRNG version.
    pub rng_version: u32,
    /// State-hash algorithm id.
    pub hash_algorithm: String,
    /// State-hash scheme version.
    pub hash_version: u32,
    /// Battle-state hash encoding version.
    pub battle_state_hash_version: u32,
}

impl ReplayHeader {
    /// The header for recording with `ruleset` in this build.
    pub fn current(ruleset: &Ruleset) -> Self {
        Self {
            format_version: REPLAY_FORMAT_VERSION,
            engine_version: env!("CARGO_PKG_VERSION").to_owned(),
            packs: ruleset.packs().to_vec(),
            profile: ruleset.profile().as_str().to_owned(),
            ruleset_hash: ruleset.content_hash(),
            rng_algorithm: RNG_ALGORITHM.to_owned(),
            rng_version: RNG_VERSION,
            hash_algorithm: HASH_ALGORITHM.to_owned(),
            hash_version: HASH_VERSION,
            battle_state_hash_version: BATTLE_STATE_HASH_VERSION,
        }
    }

    /// Checks that this build and `ruleset` can verify a replay with this header.
    pub fn check(&self, ruleset: &Ruleset) -> Result<(), ReplayError> {
        let now = Self::current(ruleset);
        let fields: [(&'static str, bool); 8] = [
            ("format_version", self.format_version == now.format_version),
            ("packs", self.packs == now.packs),
            ("profile", self.profile == now.profile),
            ("ruleset_hash", self.ruleset_hash == now.ruleset_hash),
            (
                "rng",
                self.rng_algorithm == now.rng_algorithm && self.rng_version == now.rng_version,
            ),
            (
                "hash",
                self.hash_algorithm == now.hash_algorithm && self.hash_version == now.hash_version,
            ),
            (
                "battle_state_hash_version",
                self.battle_state_hash_version == now.battle_state_hash_version,
            ),
            ("engine_version", true),
        ];
        match fields.iter().find(|(_, ok)| !ok) {
            Some((field, _)) => Err(ReplayError::Incompatible(field)),
            None => Ok(()),
        }
    }
}

/// A recorded battle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replay {
    /// What the replay depends on.
    pub header: ReplayHeader,
    /// Battle seed.
    pub seed: u64,
    /// The battle setup.
    pub setup: BattleSetup,
    /// Accepted commands in order, with the acting side.
    pub commands: Vec<(BattleSide, Command)>,
    /// A checkpoint hash is recorded after every `checkpoint_every` commands (0 = none).
    pub checkpoint_every: u32,
    /// State hashes after commands `k · checkpoint_every`, k = 1, 2, …
    pub checkpoints: Vec<u64>,
    /// State hash after the last command.
    pub final_hash: u64,
    /// Winner at the end, if decided.
    pub winner: Option<BattleSide>,
}

/// Why a replay does not verify.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayError {
    /// The header does not match this build or ruleset (field name).
    Incompatible(&'static str),
    /// The setup does not start a battle.
    Setup(SetupError),
    /// A recorded command was rejected.
    Rejected {
        /// Index of the command.
        step: usize,
        /// Why.
        error: CommandError,
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
    /// The winner differs.
    WinnerMismatch,
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
            } => {
                write!(
                    f,
                    "state hash after {step} commands: expected {expected:#018x}, got {actual:#018x}"
                )
            }
            Self::CheckpointCount => {
                f.write_str("checkpoint count does not match the command count")
            }
            Self::WinnerMismatch => f.write_str("winner differs"),
        }
    }
}

impl std::error::Error for ReplayError {}

impl Replay {
    /// Records a game played by the uniformly random legal agent (AGENT stream of `agent_seed`)
    /// for both sides, for at most `max_steps` commands (0 = no limit, as everywhere in the sim
    /// layer; a random game can take long).
    pub fn record_random(
        ruleset: &Ruleset,
        setup: BattleSetup,
        seed: u64,
        agent_seed: u64,
        max_steps: u32,
        checkpoint_every: u32,
    ) -> Result<Self, SetupError> {
        let mut state = BattleState::new(ruleset, &setup, seed)?;
        let mut agent = Rng::for_stream(agent_seed, StreamId::AGENT);
        let mut legal = LegalCommands::new();
        let mut commands = Vec::new();
        let mut checkpoints = Vec::new();
        while let Some(side) = state.side_to_act() {
            if max_steps != 0 && commands.len() >= max_steps as usize {
                break;
            }
            h3_battle::legal_commands(&state, ruleset, side, &mut legal);
            let n = u32::try_from(legal.len()).unwrap_or(u32::MAX);
            let command = legal
                .get(agent.below(n) as usize)
                .copied()
                .unwrap_or(Command::Defend);
            // Legal commands are accepted by construction (property-tested in h3-battle).
            let _ = apply(&mut state, ruleset, side, command, &mut NoopSink);
            commands.push((side, command));
            if checkpoint_every != 0 && commands.len() % checkpoint_every as usize == 0 {
                checkpoints.push(state.state_hash());
            }
        }
        Ok(Self {
            header: ReplayHeader::current(ruleset),
            seed,
            setup,
            commands,
            checkpoint_every,
            checkpoints,
            final_hash: state.state_hash(),
            winner: state.winner(),
        })
    }

    /// Re-runs the replay and compares every recorded hash and the winner.
    pub fn verify(&self, ruleset: &Ruleset) -> Result<(), ReplayError> {
        self.header.check(ruleset)?;
        let every = self.checkpoint_every as usize;
        let expected_checkpoints = self.commands.len().checked_div(every).unwrap_or(0);
        if self.checkpoints.len() != expected_checkpoints {
            return Err(ReplayError::CheckpointCount);
        }
        let mut state =
            BattleState::new(ruleset, &self.setup, self.seed).map_err(ReplayError::Setup)?;
        let mut next_checkpoint = self.checkpoints.iter();
        for (i, (side, command)) in self.commands.iter().enumerate() {
            apply(&mut state, ruleset, *side, *command, &mut NoopSink)
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
        if state.winner() != self.winner {
            return Err(ReplayError::WinnerMismatch);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recorded() -> (Ruleset, Replay) {
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_SKIRMISH).unwrap();
        let replay = Replay::record_random(&r, setup, 11, 12, 400, 25).unwrap();
        (r, replay)
    }

    #[test]
    fn recorded_replays_verify() {
        let (r, replay) = recorded();
        assert_eq!(replay.checkpoints.len(), replay.commands.len() / 25);
        assert_eq!(replay.verify(&r), Ok(()));
        let (r2, again) = recorded();
        assert_eq!(replay, again);
        assert_eq!(again.verify(&r2), Ok(()));
    }

    #[test]
    fn tampering_is_detected() {
        let (r, replay) = recorded();
        let mut bad = replay.clone();
        bad.final_hash ^= 1;
        assert!(matches!(
            bad.verify(&r),
            Err(ReplayError::HashMismatch { .. })
        ));
        let mut bad = replay.clone();
        bad.checkpoints[0] ^= 1;
        assert_eq!(
            bad.verify(&r),
            Err(ReplayError::HashMismatch {
                step: 25,
                expected: replay.checkpoints[0] ^ 1,
                actual: replay.checkpoints[0]
            })
        );
        let mut bad = replay.clone();
        bad.checkpoints.pop();
        assert_eq!(bad.verify(&r), Err(ReplayError::CheckpointCount));
        let mut bad = replay.clone();
        bad.seed += 1;
        assert!(bad.verify(&r).is_err());
        let mut bad = replay.clone();
        bad.winner = bad
            .winner
            .map(BattleSide::opponent)
            .or(Some(BattleSide::Attacker));
        assert!(bad.verify(&r).is_err());
        let mut bad = replay.clone();
        bad.commands
            .insert(0, (BattleSide::Defender, Command::Wait));
        assert!(matches!(
            bad.verify(&r),
            Err(ReplayError::Rejected { step: 0, .. })
        ));
        let mut bad = replay.clone();
        bad.setup.attacker.stacks.clear();
        bad.checkpoints.clear();
        bad.checkpoint_every = 0;
        assert_eq!(
            bad.verify(&r),
            Err(ReplayError::Setup(SetupError::EmptyArmy(
                BattleSide::Attacker
            )))
        );
    }

    #[test]
    fn headers_must_match() {
        let (r, replay) = recorded();
        let other = h3_pack::compile(&[h3_pack::SYNTHETIC_PACK], "synthetic:strict-fixed").unwrap();
        assert_eq!(
            replay.verify(&other),
            Err(ReplayError::Incompatible("profile"))
        );
        for (field, edit) in [
            (
                "format_version",
                (|h: &mut ReplayHeader| h.format_version += 1) as fn(&mut ReplayHeader),
            ),
            ("packs", |h| h.packs.clear()),
            ("ruleset_hash", |h| h.ruleset_hash ^= 1),
            ("rng", |h| h.rng_version += 1),
            ("hash", |h| h.hash_algorithm.push('x')),
            ("battle_state_hash_version", |h| {
                h.battle_state_hash_version += 1
            }),
        ] {
            let mut bad = replay.clone();
            edit(&mut bad.header);
            assert_eq!(bad.verify(&r), Err(ReplayError::Incompatible(field)));
        }
        let mut newer = replay.clone();
        newer.header.engine_version = "9.9.9".into();
        assert_eq!(newer.verify(&r), Ok(()), "engine version is informational");
    }

    #[test]
    fn error_messages() {
        assert_eq!(
            ReplayError::Incompatible("x").to_string(),
            "replay header mismatch: x"
        );
        assert_eq!(
            ReplayError::CheckpointCount.to_string(),
            "checkpoint count does not match the command count"
        );
        assert_eq!(ReplayError::WinnerMismatch.to_string(), "winner differs");
        assert!(
            ReplayError::Setup(SetupError::EmptyArmy(BattleSide::Defender))
                .to_string()
                .starts_with("setup: ")
        );
        let r = ReplayError::Rejected {
            step: 3,
            error: CommandError::NotYourTurn,
        };
        assert_eq!(r.to_string(), "command 3 rejected: it is not your turn");
        let h = ReplayError::HashMismatch {
            step: 1,
            expected: 1,
            actual: 2,
        };
        assert_eq!(
            h.to_string(),
            "state hash after 1 commands: expected 0x0000000000000001, got 0x0000000000000002"
        );
    }
}
