//! The training batch (E05 T3, ADR-0025 points 1 to 7; `docs/epics/E05-design.md` §1, §2): N
//! battle environments stepped together for training.
//!
//! * Each env's episodes come from a weighted list of [`Scenario`]s; episode `k` of env `i` has
//!   the seed [`episode_seed`]`(batch seed, i, k)` (stream [`StreamId::env`] of the batch seed),
//!   and its scenario is drawn from that seed's stream `SETUP`. A single [`BattleEnv`] from
//!   [`Scenario::env`] with that seed plays the same battle bit for bit.
//! * A scenario names a [`Seat`] per side: `External` (the caller acts) or `Internal` (a
//!   [`Policy`] plays inside the call, drawing from stream [`StreamId::seat`] of the episode
//!   seed). Every side the batch reports to move is external.
//! * **Same-step automatic reset:** an env whose episode ends in a call starts its next episode
//!   at once; the call reports how the ended one ended and shows the next one's first state. An
//!   episode that ends before an external seat could act is skipped (it holds no decision).
//! * Every side plays as a human player's (`human = [true, true]`, ADR-0025 point 6), and
//!   Retreat and Surrender are never legal and refused as invalid actions (point 7).
//! * A refused action leaves its env unchanged; the other envs step.
//! * The reward is [`reward`](crate::reward) v1 at the end of a decided episode, 0 otherwise (a
//!   truncation included).
//! * With `threads` T > 0 the envs run in T contiguous shards on parked std threads (`pool`);
//!   what an env does never depends on T.
//!
//! Trusted tooling may read an env's full state ([`BattleBatch::env`]); agents see only the
//! observations the batch writes (ADR-0025 point 12).

use std::ops::Range;
use std::sync::Arc;

use h3_battle::{Command, LegalCommands, SetupError};
use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleResult, BattleSetup, BattleSide};
use h3_kernel::{Rng, StreamId};

use crate::action::{
    ACTION_COUNT, Action, ActionMask, RETREAT, SPELL_COUNT, SURRENDER, SpellMask, TARGET_COUNT,
    TargetMask,
};
use crate::env::{BattleEnv, StepError};
use crate::observation::OBSERVATION_LEN;
use crate::reward::{self, RewardConfig};
use crate::rollout;

mod pool;

use pool::{Pool, shard_ranges, split, split_optional};

/// How many episodes in a row an env skips (each ended before an external seat could act)
/// before it gives up: then it shows no side to move until the next [`BattleBatch::reset`].
pub const MAX_SKIPPED_EPISODES: u32 = 64;

/// A policy played inside the engine for a seat (ADR-0025 point 5): a closed list, so a move
/// costs no allocation and no dynamic dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    /// Uniformly random legal commands that fight the battle out ([`rollout::choose`]): for
    /// tests and benchmarks only.
    Random,
}

impl Policy {
    /// The command this policy plays for `side` of `env`, drawing from `rng` (the seat's stream).
    pub fn choose(
        self,
        env: &BattleEnv,
        side: BattleSide,
        rng: &mut Rng,
        legal: &mut LegalCommands,
    ) -> Command {
        match self {
            Self::Random => rollout::pick(env, side, rng, legal),
        }
    }
}

/// Who plays a side of a scenario.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seat {
    /// The caller: the batch reports this side to move and takes its actions.
    External,
    /// A policy inside the engine, played within the call.
    Internal(Policy),
}

/// Why a batch refused a call or a scenario. A refused call changes nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchError {
    /// A batch needs at least one env.
    NoEnvs,
    /// A scenario's seats are both internal: nobody would act for the caller.
    NoExternalSeat,
    /// A scenario's weight is 0.
    ZeroWeight,
    /// The scenarios' weights add up to more than `u32::MAX`.
    WeightOverflow,
    /// The scenario's setup is not a valid battle.
    Setup(SetupError),
    /// The batch has no scenario to draw an episode from.
    NoScenario,
    /// The batch was not reset yet.
    NotReset,
    /// The system refused to start one of the batch's threads.
    Threads(String),
    /// An input or output array is shorter than the batch needs.
    Buffer {
        /// The array.
        name: &'static str,
        /// Its length.
        len: usize,
        /// The length needed.
        needed: usize,
    },
}

impl std::fmt::Display for BatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoEnvs => f.write_str("a batch needs at least one env"),
            Self::NoExternalSeat => f.write_str("a scenario needs an external seat"),
            Self::ZeroWeight => f.write_str("a scenario's weight must be positive"),
            Self::WeightOverflow => f.write_str("the scenarios' weights exceed u32::MAX"),
            Self::Setup(e) => write!(f, "setup: {e}"),
            Self::NoScenario => f.write_str("the batch has no scenario"),
            Self::NotReset => f.write_str("the batch was not reset"),
            Self::Threads(e) => write!(f, "a thread did not start: {e}"),
            Self::Buffer { name, len, needed } => {
                write!(f, "`{name}` has {len} elements, {needed} needed")
            }
        }
    }
}

impl std::error::Error for BatchError {}

/// An episode source entry: a battle, its weight among the batch's scenarios and its seats.
#[derive(Debug, Clone)]
pub struct Scenario {
    ruleset: Arc<BattleRuleset>,
    setup: Arc<BattleSetup>,
    random_obstacles: bool,
    weight: u32,
    seats: [Seat; 2],
}

impl Scenario {
    /// A scenario of `setup` on `ruleset`, drawn with `weight`, its sides played by `seats`
    /// (indexed by [`BattleSide::index`]). Obstacles are generated for each episode from its seed
    /// when `random_obstacles` is set (as [`BattleEnv::with_random_obstacles`]). Both sides play
    /// as human players' whatever `setup.human` says (ADR-0025 point 6).
    ///
    /// # Errors
    /// [`BatchError::NoExternalSeat`], [`BatchError::ZeroWeight`], or [`BatchError::Setup`] when
    /// `setup` is not a valid battle.
    pub fn new(
        ruleset: Arc<BattleRuleset>,
        setup: &BattleSetup,
        random_obstacles: bool,
        weight: u32,
        seats: [Seat; 2],
    ) -> Result<Self, BatchError> {
        if !seats.contains(&Seat::External) {
            return Err(BatchError::NoExternalSeat);
        }
        if weight == 0 {
            return Err(BatchError::ZeroWeight);
        }
        let setup = Arc::new(BattleSetup {
            human: [true, true],
            ..setup.clone()
        });
        let scenario = Self {
            ruleset,
            setup,
            random_obstacles,
            weight,
            seats,
        };
        scenario.env(0, 0).map_err(BatchError::Setup)?;
        Ok(scenario)
    }

    /// The setup every episode starts from (`human = [true, true]`).
    pub fn setup(&self) -> &Arc<BattleSetup> {
        &self.setup
    }

    /// The scenario's weight.
    pub fn weight(&self) -> u32 {
        self.weight
    }

    /// The seats, indexed by [`BattleSide::index`].
    pub fn seats(&self) -> [Seat; 2] {
        self.seats
    }

    /// A single environment playing this scenario's battle with `seed` (truncated after
    /// `max_steps` accepted steps of both sides, 0 = no limit): the battle a batch plays for an
    /// episode with that seed.
    ///
    /// # Errors
    /// The battle's [`SetupError`] (checked once by [`new`](Self::new) with seed 0; generated
    /// obstacles never make a valid setup invalid).
    pub fn env(&self, seed: u64, max_steps: u32) -> Result<BattleEnv, SetupError> {
        let create = if self.random_obstacles {
            BattleEnv::with_random_obstacles
        } else {
            BattleEnv::new
        };
        create(self.ruleset.clone(), self.setup.clone(), seed, max_steps)
    }
}

/// The seed of episode `episode` (0 the first) of env `env` in a batch with seed `batch_seed`:
/// draw `episode + 1` of stream [`StreamId::env`]`(env)` of the batch seed (ADR-0025 point 4).
pub fn episode_seed(batch_seed: u64, env: u32, episode: u32) -> u64 {
    let mut rng = Rng::for_stream(batch_seed, StreamId::env(env));
    let mut seed = rng.next_u64();
    for _ in 0..episode {
        seed = rng.next_u64();
    }
    seed
}

/// The index of the scenario an episode with `seed` plays among `scenarios`, whose weights add
/// up to `total`: drawn from the seed's stream `SETUP` by weight.
pub fn draw_scenario(scenarios: &[Scenario], total: u32, seed: u64) -> usize {
    let mut point = Rng::for_stream(seed, StreamId::SETUP).below(total);
    scenarios
        .iter()
        .position(|s| {
            let inside = point < s.weight;
            point = point.saturating_sub(s.weight);
            inside
        })
        .unwrap_or(0)
}

/// The settings of a batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchConfig {
    /// The number of envs, N.
    pub envs: u32,
    /// The number of worker threads, T (at most N are started); 0 runs every call in the
    /// caller's thread.
    pub threads: u32,
    /// The batch's seed: every env's episode seeds derive from it ([`episode_seed`]).
    pub seed: u64,
    /// Accepted steps of both seats per episode before it is truncated; 0 = no limit.
    pub max_steps: u32,
    /// The reward's currency and mana price.
    pub reward: RewardConfig,
}

/// What a call reports for one env.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EnvReport {
    /// The side to act now (always an external seat), or `None` for an env that gave up
    /// ([`MAX_SKIPPED_EPISODES`]).
    pub to_move: Option<BattleSide>,
    /// An episode ended in this call (decided or truncated); the env shows its next episode.
    pub done: bool,
    /// The episode that ended was truncated by the step limit.
    pub truncated: bool,
    /// How the episode that ended in this call ended, if it was decided.
    pub result: Option<BattleResult>,
    /// This call's reward per side, indexed by [`BattleSide::index`]: reward v1
    /// ([`reward::rewards`]) of the episode that ended in this call if it was decided, else 0.
    pub reward: [f32; 2],
    /// The number of the env's current episode (0 the first; it grows at each reset).
    pub episode: u32,
    /// The index of the current episode's scenario among the batch's.
    pub scenario: u32,
    /// The current episode's seed.
    pub episode_seed: u64,
    /// Why the env refused this call's action (it is unchanged), if it did.
    pub error: Option<StepError>,
}

/// The caller's arrays a call writes into; `None` skips that output. An env's rows are
/// contiguous: `obs` holds N × [`OBSERVATION_LEN`] values (the acting seat's observation),
/// `entry_mask` N × [`ACTION_COUNT`] (0/1), `final_obs` N × 2 × [`OBSERVATION_LEN`] (both sides'
/// last observations, written only for envs whose episode was truncated in this call, for
/// bootstrapping), `reports` N.
#[derive(Debug, Default)]
pub struct Outputs<'a> {
    /// The acting seat's observation per env.
    pub obs: Option<&'a mut [f32]>,
    /// The acting seat's legal entries per env (0/1).
    pub entry_mask: Option<&'a mut [u8]>,
    /// Both sides' last observations of a truncated episode.
    pub final_obs: Option<&'a mut [f32]>,
    /// What happened per env.
    pub reports: Option<&'a mut [EnvReport]>,
}

/// One env of a batch.
#[derive(Debug)]
struct Slot {
    /// The env's index.
    index: u32,
    /// Its episode seeds: stream `env(index)` of the batch seed.
    seeds: Rng,
    /// Episodes started (the current one is `started - 1`).
    started: u32,
    /// The current episode's seed and scenario.
    episode_seed: u64,
    scenario: u32,
    /// The current episode, `None` before the first reset or after giving up.
    env: Option<BattleEnv>,
    /// The current scenario's seats and the internal seats' streams.
    seats: [Seat; 2],
    seat_rngs: [Rng; 2],
    legal: LegalCommands,
    mask: ActionMask,
    spells: SpellMask,
    targets: TargetMask,
}

/// One env's rows of the outputs.
#[derive(Debug, Default)]
struct Rows<'a> {
    obs: Option<&'a mut [f32]>,
    mask: Option<&'a mut [u8]>,
    final_obs: Option<&'a mut [f32]>,
    report: Option<&'a mut EnvReport>,
}

/// The episodes' source: the scenarios and their total weight.
#[derive(Debug, Clone, Copy)]
struct Source<'a> {
    scenarios: &'a [Scenario],
    total: u32,
    max_steps: u32,
    reward: RewardConfig,
}

/// N battle environments stepped together (see the module documentation).
#[derive(Debug)]
pub struct BattleBatch {
    config: BatchConfig,
    scenarios: Vec<Scenario>,
    total_weight: u32,
    slots: Vec<Slot>,
    /// Each shard's envs, in order (one shard per worker, or one).
    ranges: Vec<Range<usize>>,
    pool: Pool,
    reset: bool,
}

impl BattleBatch {
    /// A batch of `config.envs` envs without scenarios; add them, then [`reset`](Self::reset).
    ///
    /// # Errors
    /// [`BatchError::NoEnvs`], or [`BatchError::Threads`].
    pub fn new(config: BatchConfig) -> Result<Self, BatchError> {
        if config.envs == 0 {
            return Err(BatchError::NoEnvs);
        }
        let envs = config.envs as usize;
        let pool = Pool::new((config.threads as usize).min(envs))
            .map_err(|e| BatchError::Threads(e.to_string()))?;
        let ranges = shard_ranges(envs, pool.shards());
        let slots = (0..config.envs)
            .map(|index| Slot {
                index,
                seeds: Rng::for_stream(config.seed, StreamId::env(index)),
                started: 0,
                episode_seed: 0,
                scenario: 0,
                env: None,
                seats: [Seat::External; 2],
                seat_rngs: [Rng::from_seed(0), Rng::from_seed(0)],
                legal: LegalCommands::new(),
                mask: [false; ACTION_COUNT],
                spells: [false; SPELL_COUNT],
                targets: [false; TARGET_COUNT],
            })
            .collect();
        Ok(Self {
            config,
            scenarios: Vec::new(),
            total_weight: 0,
            slots,
            ranges,
            pool,
            reset: false,
        })
    }

    /// The batch's settings.
    pub fn config(&self) -> BatchConfig {
        self.config
    }

    /// The number of envs.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// Always false: a batch has at least one env.
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// The scenarios, in the order they were added (their indices in [`EnvReport::scenario`]).
    pub fn scenarios(&self) -> &[Scenario] {
        &self.scenarios
    }

    /// Adds an episode source entry; episodes started from now on draw it by weight.
    ///
    /// # Errors
    /// [`BatchError::WeightOverflow`].
    pub fn add_scenario(&mut self, scenario: Scenario) -> Result<(), BatchError> {
        self.total_weight = self
            .total_weight
            .checked_add(scenario.weight)
            .ok_or(BatchError::WeightOverflow)?;
        self.scenarios.push(scenario);
        Ok(())
    }

    /// Env `index`'s current battle, for trusted tooling and tests (the full state: never an
    /// agent's view, ADR-0025 point 12); `None` before the first reset or for an env that gave up.
    pub fn env(&self, index: usize) -> Option<&BattleEnv> {
        self.slots.get(index).and_then(|s| s.env.as_ref())
    }

    /// Starts every env's first episode (and again from the start on a later call: the batch
    /// then replays the same episodes) and writes them into `out`.
    ///
    /// # Errors
    /// [`BatchError::NoScenario`], or [`BatchError::Buffer`] for a short output array.
    pub fn reset(&mut self, out: &mut Outputs<'_>) -> Result<(), BatchError> {
        if self.scenarios.is_empty() {
            return Err(BatchError::NoScenario);
        }
        self.check_outputs(out)?;
        let seed = self.config.seed;
        let source = Source {
            scenarios: &self.scenarios,
            total: self.total_weight,
            max_steps: self.config.max_steps,
            reward: self.config.reward,
        };
        let parts = parts(&mut self.slots, out, &self.ranges);
        self.pool.run(parts, |(slots, mut out)| {
            for (slot, rows) in slots.iter_mut().zip(rows(&mut out)) {
                slot.seeds = Rng::for_stream(seed, StreamId::env(slot.index));
                slot.started = 0;
                begin(slot, source);
                write_state(slot, rows, &mut EnvReport::default());
            }
        });
        self.reset = true;
        Ok(())
    }

    /// Plays `actions[i]` for env i's acting seat, then its internal seats until an external
    /// seat must act or the episode ends (then its next episode starts), and writes the result
    /// into `out`. An env that refuses its action is unchanged (its report's `error`).
    ///
    /// # Errors
    /// [`BatchError::NotReset`], or [`BatchError::Buffer`] for a short array.
    // Never inlined: the batch benchmarks count inside it (`benches/ai/`).
    #[inline(never)]
    pub fn step(&mut self, actions: &[Action], out: &mut Outputs<'_>) -> Result<(), BatchError> {
        self.check_ready()?;
        check("actions", actions.len(), self.len())?;
        self.check_outputs(out)?;
        let source = Source {
            scenarios: &self.scenarios,
            total: self.total_weight,
            max_steps: self.config.max_steps,
            reward: self.config.reward,
        };
        let parts: Vec<_> = parts(&mut self.slots, out, &self.ranges)
            .into_iter()
            .zip(&self.ranges)
            .map(|((slots, out), envs)| (slots, out, &actions[envs.clone()]))
            .collect();
        self.pool.run(parts, |(slots, mut out, actions)| {
            for ((slot, rows), action) in slots.iter_mut().zip(rows(&mut out)).zip(actions) {
                step_slot(slot, source, *action, rows);
            }
        });
        Ok(())
    }

    /// Writes each env's spell-head mask after choosing `entries[i]` into
    /// `out[i * SPELL_COUNT..]` (0/1; "0 only" for an entry that is no cast, as the single env's).
    ///
    /// # Errors
    /// [`BatchError::NotReset`], or [`BatchError::Buffer`] for a short array.
    pub fn spell_mask(&mut self, entries: &[u32], out: &mut [u8]) -> Result<(), BatchError> {
        self.check_ready()?;
        check("entries", entries.len(), self.len())?;
        check("out", out.len(), self.len() * SPELL_COUNT)?;
        self.each_row(out, SPELL_COUNT, |slot, i, row| {
            slot.spells.fill(false);
            if let Some((env, side)) = acting(slot.env.as_ref()) {
                env.spell_mask(side, entries[i] as usize, &mut slot.legal, &mut slot.spells);
            }
            write_mask(&slot.spells, row);
        });
        Ok(())
    }

    /// Writes each env's target-head mask after choosing `entries[i]` and `spells[i]` into
    /// `out[i * TARGET_COUNT..]` (0/1).
    ///
    /// # Errors
    /// [`BatchError::NotReset`], or [`BatchError::Buffer`] for a short array.
    pub fn target_mask(
        &mut self,
        entries: &[u32],
        spells: &[u16],
        out: &mut [u8],
    ) -> Result<(), BatchError> {
        self.check_ready()?;
        check("entries", entries.len(), self.len())?;
        check("spells", spells.len(), self.len())?;
        check("out", out.len(), self.len() * TARGET_COUNT)?;
        self.each_row(out, TARGET_COUNT, |slot, i, row| {
            slot.targets.fill(false);
            if let Some((env, side)) = acting(slot.env.as_ref()) {
                let prefix = (entries[i] as usize, spells[i]);
                env.target_mask(side, prefix, &mut slot.legal, &mut slot.targets);
            }
            write_mask(&slot.targets, row);
        });
        Ok(())
    }

    /// Writes each env's second-target mask after choosing `entries[i]`, `spells[i]` and
    /// `targets[i]` into `out[i * TARGET_COUNT..]` (0/1).
    ///
    /// # Errors
    /// [`BatchError::NotReset`], or [`BatchError::Buffer`] for a short array.
    pub fn second_mask(
        &mut self,
        entries: &[u32],
        spells: &[u16],
        targets: &[u16],
        out: &mut [u8],
    ) -> Result<(), BatchError> {
        self.check_ready()?;
        check("entries", entries.len(), self.len())?;
        check("spells", spells.len(), self.len())?;
        check("targets", targets.len(), self.len())?;
        check("out", out.len(), self.len() * TARGET_COUNT)?;
        self.each_row(out, TARGET_COUNT, |slot, i, row| {
            slot.targets.fill(false);
            if let Some((env, side)) = acting(slot.env.as_ref()) {
                let prefix = (entries[i] as usize, spells[i], targets[i]);
                env.second_mask(side, prefix, &mut slot.legal, &mut slot.targets);
            }
            write_mask(&slot.targets, row);
        });
        Ok(())
    }

    /// Runs `f(slot, i, row)` for every env i with its row of `out` (`width` values per env, the
    /// array checked to hold every env), shard by shard on the pool.
    fn each_row(
        &mut self,
        out: &mut [u8],
        width: usize,
        f: impl Fn(&mut Slot, usize, &mut [u8]) + Sync,
    ) {
        let parts: Vec<_> = split(&mut self.slots, 1, &self.ranges)
            .into_iter()
            .zip(split(out, width, &self.ranges))
            .zip(self.ranges.iter().cloned())
            .collect();
        self.pool.run(parts, |((slots, out), envs)| {
            for ((slot, row), i) in slots.iter_mut().zip(out.chunks_exact_mut(width)).zip(envs) {
                f(slot, i, row);
            }
        });
    }

    fn check_ready(&self) -> Result<(), BatchError> {
        if self.reset {
            Ok(())
        } else {
            Err(BatchError::NotReset)
        }
    }

    fn check_outputs(&self, out: &Outputs<'_>) -> Result<(), BatchError> {
        let n = self.len();
        if let Some(obs) = &out.obs {
            check("obs", obs.len(), n * OBSERVATION_LEN)?;
        }
        if let Some(mask) = &out.entry_mask {
            check("entry_mask", mask.len(), n * ACTION_COUNT)?;
        }
        if let Some(final_obs) = &out.final_obs {
            check("final_obs", final_obs.len(), n * 2 * OBSERVATION_LEN)?;
        }
        if let Some(reports) = &out.reports {
            check("reports", reports.len(), n)?;
        }
        Ok(())
    }
}

/// `Err` unless `len >= needed`.
fn check(name: &'static str, len: usize, needed: usize) -> Result<(), BatchError> {
    if len >= needed {
        Ok(())
    } else {
        Err(BatchError::Buffer { name, len, needed })
    }
}

/// Each shard's envs and their rows of `out` (checked to hold every env).
fn parts<'a>(
    slots: &'a mut [Slot],
    out: &'a mut Outputs<'_>,
    ranges: &[Range<usize>],
) -> Vec<(&'a mut [Slot], Outputs<'a>)> {
    let obs = split_optional(out.obs.as_deref_mut(), OBSERVATION_LEN, ranges);
    let mask = split_optional(out.entry_mask.as_deref_mut(), ACTION_COUNT, ranges);
    let final_obs = split_optional(out.final_obs.as_deref_mut(), 2 * OBSERVATION_LEN, ranges);
    let reports = split_optional(out.reports.as_deref_mut(), 1, ranges);
    split(slots, 1, ranges)
        .into_iter()
        .zip(obs)
        .zip(mask)
        .zip(final_obs)
        .zip(reports)
        .map(|((((slots, obs), entry_mask), final_obs), reports)| {
            let out = Outputs {
                obs,
                entry_mask,
                final_obs,
                reports,
            };
            (slots, out)
        })
        .collect()
}

/// Each env's rows of `out`, in env order (as many as `out`'s shortest present array allows;
/// the arrays were checked to hold every env).
fn rows<'a>(out: &'a mut Outputs<'_>) -> impl Iterator<Item = Rows<'a>> {
    let mut obs = out
        .obs
        .as_deref_mut()
        .map(|o| o.chunks_exact_mut(OBSERVATION_LEN));
    let mut mask = out
        .entry_mask
        .as_deref_mut()
        .map(|m| m.chunks_exact_mut(ACTION_COUNT));
    let mut final_obs = out
        .final_obs
        .as_deref_mut()
        .map(|o| o.chunks_exact_mut(2 * OBSERVATION_LEN));
    let mut reports = out.reports.as_deref_mut().map(|r| r.iter_mut());
    std::iter::from_fn(move || {
        Some(Rows {
            obs: obs.as_mut().and_then(Iterator::next),
            mask: mask.as_mut().and_then(Iterator::next),
            final_obs: final_obs.as_mut().and_then(Iterator::next),
            report: reports.as_mut().and_then(Iterator::next),
        })
    })
}

/// The env and its side to move, if it has one.
fn acting(env: Option<&BattleEnv>) -> Option<(&BattleEnv, BattleSide)> {
    env.and_then(|e| e.to_move().map(|side| (e, side)))
}

/// Converts `bools` into 0/1 bytes in `out`.
fn write_mask(bools: &[bool], out: &mut [u8]) {
    for (o, m) in out.iter_mut().zip(bools) {
        *o = u8::from(*m);
    }
}

/// Plays the internal seats of `env` until an external seat must act or the episode ends; the
/// reward and result of the last accepted step if one ended the episode.
fn play_internal(
    env: &mut BattleEnv,
    seats: [Seat; 2],
    rngs: &mut [Rng; 2],
    legal: &mut LegalCommands,
) -> Option<crate::StepResult> {
    let mut last = None;
    while let Some(side) = env.to_move() {
        let Seat::Internal(policy) = seats[side.index()] else {
            break;
        };
        let command = policy.choose(env, side, &mut rngs[side.index()], legal);
        // A policy plays legal commands, which always apply (property-tested in h3-battle).
        last = rollout::step_command(env, side, command).ok().or(last);
    }
    last
}

/// Starts the slot's next episode, skipping those that end before an external seat could act
/// (at most [`MAX_SKIPPED_EPISODES`] in a row; then the slot gives up: no env).
fn begin(slot: &mut Slot, source: Source<'_>) {
    slot.env = None;
    for _ in 0..=MAX_SKIPPED_EPISODES {
        slot.started = slot.started.saturating_add(1);
        slot.episode_seed = slot.seeds.next_u64();
        let index = draw_scenario(source.scenarios, source.total, slot.episode_seed);
        let scenario = &source.scenarios[index];
        slot.scenario = index as u32;
        slot.seats = scenario.seats;
        slot.seat_rngs = [0, 1].map(|s| Rng::for_stream(slot.episode_seed, StreamId::seat(s)));
        let Ok(mut env) = scenario.env(slot.episode_seed, source.max_steps) else {
            continue;
        };
        play_internal(&mut env, slot.seats, &mut slot.seat_rngs, &mut slot.legal);
        if !env.is_done() {
            slot.env = Some(env);
            return;
        }
    }
}

/// Plays one env's call (see [`BattleBatch::step`]).
fn step_slot(slot: &mut Slot, source: Source<'_>, action: Action, rows: Rows<'_>) {
    let Rows {
        obs,
        mask,
        final_obs,
        report: row,
    } = rows;
    let mut report = EnvReport::default();
    report.error = advance(slot, source, action, final_obs, &mut report).err();
    let rows = Rows {
        obs,
        mask,
        final_obs: None,
        report: row,
    };
    write_state(slot, rows, &mut report);
}

/// Plays the acting seat's `action` and the internal seats after it; when the episode ends,
/// reports it (with both sides' last observations into `final_obs` if it was truncated) and
/// starts the next one.
fn advance(
    slot: &mut Slot,
    source: Source<'_>,
    action: Action,
    final_obs: Option<&mut [f32]>,
    report: &mut EnvReport,
) -> Result<(), StepError> {
    let env = slot.env.as_mut().ok_or(StepError::EpisodeOver)?;
    let side = env.to_move().ok_or(StepError::EpisodeOver)?;
    if action.entry as usize == RETREAT || action.entry as usize == SURRENDER {
        return Err(StepError::InvalidAction);
    }
    let first = env.step(side, action)?;
    let last =
        play_internal(env, slot.seats, &mut slot.seat_rngs, &mut slot.legal).unwrap_or(first);
    if !env.is_done() {
        return Ok(());
    }
    report.done = true;
    report.truncated = last.truncated;
    report.result = last.result;
    // A decided battle has an outcome; a truncated one has none and gets 0.
    report.reward = env.state().outcome().map_or([0.0; 2], |outcome| {
        reward::rewards(source.reward, env.ruleset(), env.setup(), &outcome)
    });
    if let Some(final_obs) = final_obs.filter(|_| last.truncated) {
        let (attacker, defender) = final_obs.split_at_mut(OBSERVATION_LEN);
        let _ = env.encode_observation(BattleSide::Attacker, attacker);
        let _ = env.encode_observation(BattleSide::Defender, defender);
    }
    begin(slot, source);
    Ok(())
}

/// Writes the slot's current state (the acting seat's observation and entry mask, without the
/// escapes) and `report` with its episode into `rows`.
fn write_state(slot: &mut Slot, rows: Rows<'_>, report: &mut EnvReport) {
    slot.mask.fill(false);
    let acting = acting(slot.env.as_ref());
    if let Some((env, side)) = acting {
        env.legal_mask(side, &mut slot.legal, &mut slot.mask);
        slot.mask[RETREAT] = false;
        slot.mask[SURRENDER] = false;
    }
    if let Some(mask) = rows.mask {
        write_mask(&slot.mask, mask);
    }
    if let Some(obs) = rows.obs {
        match acting {
            Some((env, side)) => {
                let _ = env.encode_observation(side, obs);
            }
            None => obs.fill(0.0),
        }
    }
    report.to_move = acting.map(|(_, side)| side);
    report.episode = slot.started.saturating_sub(1);
    report.scenario = slot.scenario;
    report.episode_seed = slot.episode_seed;
    if let Some(r) = rows.report {
        *r = *report;
    }
}

#[cfg(test)]
mod tests;
