//! Adapter: reinforcement-learning simulation environments over the battle context
//! (docs/07-simulation-and-ai.md).
//!
//! * [`BattleEnv`]: reset / step / legal mask / fork / state hash for one battle.
//! * [`batch`]: [`BattleBatch`], N battles stepped together for training, with seats and automatic
//!   reset (ADR-0025).
//! * [`action`]: the versioned action encoding, dense entries with a factorized cast (owned by
//!   the sim layer, not the domain).
//! * [`observation`]: the flat, versioned observation encoding of a per-viewer [`Observation`].
//! * [`replay`]: recording and verifying replays (seed + commands + state hashes).
//! * [`encounter`]: the encounter distribution v1, a sampler of battles as the adventure brings
//!   them (E05 T5b, decision 7).
//! * [`reward`]: the batch's reward v1 (value killed, value lost, mana spent; ADR-0025 point 8).
//! * [`adventure`]: [`adventure::AdventureEnv`] for a game on an adventure map, with its own action
//!   encoding (the adventure lane's, ADR-0017 point 8).
//!
//! The sim layer is where rewards, episode truncation and encodings live; the domain knows
//! nothing about them. Floats appear only in encodings and rewards and never feed back into game
//! state (ADR-0003).
//!
//! [`Observation`]: h3_battle::Observation

pub mod action;
pub mod adventure;
pub mod batch;
pub mod bots;
pub mod encounter;
pub mod env;
pub mod observation;
pub mod replay;
pub mod reward;
pub mod rollout;

pub use action::{
    ACTION_COUNT, ACTION_ENCODING_VERSION, Action, ActionMask, PREVIEW_BASE, PREVIEW_ENTRIES,
    PREVIEW_LEN, SPELL_COUNT, SpellMask, TARGET_COUNT, TargetMask,
};
pub use batch::{
    BatchConfig, BatchError, BattleBatch, EnvReport, Episode, GUESS_CANDIDATES, MAX_ENVS, Outputs,
    Policy, Scenario, Seat,
};
pub use encounter::{ENCOUNTERS_VERSION, Encounter, EncounterTable, Kind, Sampler};
pub use env::{BattleEnv, StepError, StepResult};
/// The behaviour version of the system actor's policy, [`Policy::System`] (ADR-0027 point 12): a
/// training run's record names it with its opponents (E06 T7).
pub use h3_battle_ai::POLICY_VERSION as SYSTEM_POLICY_VERSION;
pub use observation::{OBSERVATION_ENCODING_VERSION, OBSERVATION_LEN};
pub use replay::{Replay, ReplayError, ReplayHeader};
pub use reward::{Currency, RewardConfig};
pub use rollout::{RolloutResult, random_rollout};

/// A literal constant of the entity network's layout (`python/h3env/train/layout.py`, E06 T5a),
/// for the tests that pin it to the encodings.
#[cfg(test)]
pub(crate) fn python_layout(name: &str) -> usize {
    let prefix = format!("{name} = ");
    include_str!("../../../python/h3env/train/layout.py")
        .lines()
        .find_map(|l| l.strip_prefix(prefix.as_str()))
        .unwrap()
        .parse()
        .unwrap()
}
