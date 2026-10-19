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
pub mod encounter;
pub mod env;
pub mod observation;
pub mod replay;
pub mod reward;
pub mod rollout;

pub use action::{
    ACTION_COUNT, ACTION_ENCODING_VERSION, Action, ActionMask, SPELL_COUNT, SpellMask,
    TARGET_COUNT, TargetMask,
};
pub use batch::{
    BatchConfig, BatchError, BattleBatch, EnvReport, Episode, Outputs, Policy, Scenario, Seat,
};
pub use encounter::{Encounter, EncounterTable, Kind, Sampler};
pub use env::{BattleEnv, StepError, StepResult};
pub use observation::{OBSERVATION_ENCODING_VERSION, OBSERVATION_LEN};
pub use replay::{Replay, ReplayError, ReplayHeader};
pub use reward::{Currency, RewardConfig};
pub use rollout::{RolloutResult, random_rollout};
