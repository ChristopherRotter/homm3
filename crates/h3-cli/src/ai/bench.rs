//! `h3 ai bench` (E05 T3c; `docs/epics/E05-log.md` §7): a batch of N envs of one scenario (or,
//! since T5b2, of the encounter distribution v1 on the original packs), the caller's seats
//! external, called K times with random legal actions chosen from the batch's own
//! masks (a cast through the head calls, made only when an env chose one), observations and entry
//! masks written. It reports the steps per second of wall time, inside `BattleBatch::step` and
//! with the head calls and the choice. `cargo xtask bench-check` counts the gated workloads'
//! instructions (`benches/ai/`, the caller's thread) only inside
//! `h3_sim::batch::BattleBatch::step`.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use h3_kernel::{Rng, StreamId};
use h3_sim::action::{CREATURE_CAST, HERO_CAST};
use h3_sim::{
    ACTION_COUNT, Action, BatchConfig, BattleBatch, EncounterTable, EnvReport, OBSERVATION_LEN,
    Outputs, RewardConfig, SPELL_COUNT, Sampler, Scenario, Seat, TARGET_COUNT,
};

use crate::load_scenario;

/// What [`run`] plays.
#[derive(Debug, Clone)]
pub struct BenchOptions {
    /// The scenario file.
    pub scenario: PathBuf,
    /// The encounter distribution v1 instead, its kinds drawn by these weights (in
    /// `h3_sim::Kind::ALL`'s order); `scenario` and `packs_dir` are then unused.
    pub encounters: Option<[u32; 6]>,
    /// The packs' directory.
    pub packs_dir: PathBuf,
    /// The batch's envs, N.
    pub envs: u32,
    /// Its worker threads, T (0 = the caller's thread).
    pub threads: u32,
    /// The calls of `step`, K.
    pub calls: u32,
    /// The batch's seed and the random choice's (its stream `AGENT`).
    pub seed: u64,
    /// The step limit per episode (0 = none).
    pub max_steps: u32,
}

/// What [`run`] measured.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BenchReport {
    /// Accepted steps of the caller's seats.
    pub steps: u64,
    /// Episodes that ended.
    pub episodes: u64,
    /// Actions refused (an env that gave up refuses every action).
    pub refused: u64,
    /// Wall time of the K calls with the choice and the head calls.
    pub secs: f64,
    /// Wall time inside `BattleBatch::step`.
    pub step_secs: f64,
}

/// The arrays a call reads and writes.
struct Arrays {
    obs: Vec<f32>,
    mask: Vec<u8>,
    reports: Vec<EnvReport>,
    heads: Vec<u8>,
    entries: Vec<u32>,
    spells: Vec<u16>,
    targets: Vec<u16>,
    seconds: Vec<u16>,
    actions: Vec<Action>,
}

impl Arrays {
    fn new(n: usize) -> Self {
        Self {
            obs: vec![0.0; n * OBSERVATION_LEN],
            mask: vec![0; n * ACTION_COUNT],
            reports: vec![EnvReport::default(); n],
            heads: vec![0; n * SPELL_COUNT.max(TARGET_COUNT)],
            entries: vec![0; n],
            spells: vec![0; n],
            targets: vec![0; n],
            seconds: vec![0; n],
            actions: vec![Action::default(); n],
        }
    }

    fn outputs(&mut self) -> Outputs<'_> {
        Outputs {
            obs: Some(&mut self.obs),
            entry_mask: Some(&mut self.mask),
            final_obs: None,
            reports: Some(&mut self.reports),
        }
    }
}

/// A uniformly random index among the set bytes of `mask` (0 if none is set).
fn pick(mask: &[u8], rng: &mut Rng) -> usize {
    let legal = mask.iter().filter(|m| **m != 0).count() as u32;
    if legal == 0 {
        return 0;
    }
    let chosen = rng.below(legal) as usize;
    mask.iter()
        .enumerate()
        .filter(|(_, m)| **m != 0)
        .nth(chosen)
        .map_or(0, |(i, _)| i)
}

/// Fills `arrays.actions` with a random legal action per env from the batch's masks: the entry,
/// then, if any env chose a cast, its heads one call after another (an env without a cast keeps
/// the heads 0, the only legal value).
fn choose(batch: &mut BattleBatch, arrays: &mut Arrays, rng: &mut Rng) -> Result<(), String> {
    let n = batch.len();
    let mut casts = false;
    for (entry, mask) in arrays
        .entries
        .iter_mut()
        .zip(arrays.mask.chunks_exact(ACTION_COUNT))
    {
        *entry = pick(mask, rng) as u32;
        casts |= [HERO_CAST, CREATURE_CAST].contains(&(*entry as usize));
    }
    arrays.spells.fill(0);
    arrays.targets.fill(0);
    arrays.seconds.fill(0);
    if casts {
        let error = |e: h3_sim::BatchError| e.to_string();
        let heads = &mut arrays.heads;
        batch
            .spell_mask(&arrays.entries, &mut heads[..n * SPELL_COUNT])
            .map_err(error)?;
        for (s, m) in arrays
            .spells
            .iter_mut()
            .zip(heads.chunks_exact(SPELL_COUNT))
        {
            *s = pick(m, rng) as u16;
        }
        let out = &mut heads[..n * TARGET_COUNT];
        batch
            .target_mask(&arrays.entries, &arrays.spells, out)
            .map_err(error)?;
        for (t, m) in arrays
            .targets
            .iter_mut()
            .zip(out.chunks_exact(TARGET_COUNT))
        {
            *t = pick(m, rng) as u16;
        }
        batch
            .second_mask(&arrays.entries, &arrays.spells, &arrays.targets, out)
            .map_err(error)?;
        for (t, m) in arrays
            .seconds
            .iter_mut()
            .zip(out.chunks_exact(TARGET_COUNT))
        {
            *t = pick(m, rng) as u16;
        }
    }
    for (i, action) in arrays.actions.iter_mut().enumerate() {
        *action = Action {
            entry: arrays.entries[i],
            spell: arrays.spells[i],
            target: arrays.targets[i],
            second: arrays.seconds[i],
        };
    }
    Ok(())
}

/// The encounter distribution v1 on the original packs (`base:faithful`) with the kinds' `weights`.
pub(super) fn encounters(weights: [u32; 6]) -> Result<Scenario, String> {
    let profile = "base:faithful";
    let battle = h3_pack_battle::compile(
        &[
            h3_pack_battle::BASE_PACK,
            h3_pack_battle::AB_PACK,
            h3_pack_battle::SOD_PACK,
        ],
        profile,
    )
    .map_err(|e| e.to_string())?;
    let (_, adventure) = h3_pack_adventure::compile(&h3_pack_adventure::ORIGINAL_PACKS, profile)
        .map_err(|e| e.to_string())?;
    let sampler = Sampler::new(Arc::new(battle), Arc::new(adventure), &EncounterTable::v1())
        .and_then(|s| s.with_weights(weights))
        .map_err(|e| e.to_string())?;
    Scenario::encounters(Arc::new(sampler), 1, None).map_err(|e| e.to_string())
}

/// Plays the benchmark (see the module documentation).
///
/// # Errors
///
/// A message: the scenario does not load, or the batch refuses its settings.
pub fn run(options: &BenchOptions) -> Result<BenchReport, String> {
    let scenario = match options.encounters {
        Some(weights) => encounters(weights)?,
        None => {
            let (def, ruleset, setup) =
                load_scenario(&options.scenario, &options.packs_dir, options.seed)
                    .map_err(|e| e.to_string())?;
            Scenario::new(
                Arc::new(ruleset),
                &setup,
                def.random_obstacles,
                1,
                [Seat::External; 2],
            )
            .map_err(|e| e.to_string())?
        }
    };
    let mut batch = BattleBatch::new(BatchConfig {
        envs: options.envs,
        threads: options.threads,
        seed: options.seed,
        max_steps: options.max_steps,
        reward: RewardConfig::default(),
    })
    .map_err(|e| e.to_string())?;
    batch.add_scenario(scenario).map_err(|e| e.to_string())?;
    let mut arrays = Arrays::new(batch.len());
    batch
        .reset(&mut arrays.outputs())
        .map_err(|e| e.to_string())?;
    let mut rng = Rng::for_stream(options.seed, StreamId::AGENT);
    let mut report = BenchReport {
        steps: 0,
        episodes: 0,
        refused: 0,
        secs: 0.0,
        step_secs: 0.0,
    };
    let start = Instant::now();
    for _ in 0..options.calls {
        choose(&mut batch, &mut arrays, &mut rng)?;
        let step = Instant::now();
        let actions = std::mem::take(&mut arrays.actions);
        batch
            .step(&actions, &mut arrays.outputs())
            .map_err(|e| e.to_string())?;
        arrays.actions = actions;
        report.step_secs += step.elapsed().as_secs_f64();
        for r in &arrays.reports {
            match r.error {
                Some(_) => report.refused += 1,
                None => report.steps += 1,
            }
            report.episodes += u64::from(r.done);
        }
    }
    report.secs = start.elapsed().as_secs_f64();
    Ok(report)
}
