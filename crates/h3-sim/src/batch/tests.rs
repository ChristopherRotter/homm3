//! E05 T3b, T3c: the batch against N single envs built from its public pieces (the determinism
//! property for any thread count, ADR-0025 points 3 and 4), its seats, escapes, truncation,
//! skipped episodes and errors.

use std::sync::Arc;

use h3_battle::LegalCommands;
use h3_contracts::{BattleSetup, BattleSide, RetreatFacts};
use h3_kernel::{Rng, StreamId};
use proptest::prelude::*;

use super::*;
use crate::action::{CREATURE_CAST, HERO_CAST, PREVIEW_BASE, PREVIEW_ENTRIES, RETREAT, SURRENDER};
use crate::encounter::Kind;
use crate::reward::{self, Currency, RewardConfig};
use crate::rollout;

/// The synthetic scenarios a batch draws from in these tests.
const TEXTS: [&str; 4] = [
    h3_pack_battle::SYNTHETIC_DUEL,
    h3_pack_battle::SYNTHETIC_SKIRMISH,
    h3_pack_battle::SYNTHETIC_FULL,
    h3_pack_battle::SYNTHETIC_SIEGE,
];

/// The scenario of `text` with `weight` and `seats`.
fn scenario(text: &str, weight: u32, seats: [Seat; 2]) -> Scenario {
    let random_obstacles = h3_pack_battle::parse_scenario(text)
        .unwrap()
        .random_obstacles;
    let (ruleset, setup) = h3_pack_battle::synthetic(text).unwrap();
    Scenario::new(Arc::new(ruleset), &setup, random_obstacles, weight, seats).unwrap()
}

const EXTERNAL: [Seat; 2] = [Seat::External, Seat::External];
const RANDOM: Seat = Seat::Internal(Policy::Random);
const SYSTEM: Seat = Seat::Internal(Policy::System);

/// The output arrays of a batch of `n` envs.
struct Arrays {
    obs: Vec<f32>,
    mask: Vec<u8>,
    final_obs: Vec<f32>,
    preview: Vec<f32>,
    potential: Vec<f32>,
    reports: Vec<EnvReport>,
}

impl Arrays {
    fn new(n: usize) -> Self {
        Self {
            obs: vec![0.0; n * OBSERVATION_LEN],
            mask: vec![0; n * ACTION_COUNT],
            final_obs: vec![0.0; n * 2 * OBSERVATION_LEN],
            preview: vec![0.0; n * PREVIEW_LEN],
            potential: vec![0.0; n * 2],
            reports: vec![EnvReport::default(); n],
        }
    }

    fn outputs(&mut self) -> Outputs<'_> {
        Outputs {
            obs: Some(&mut self.obs),
            entry_mask: Some(&mut self.mask),
            final_obs: Some(&mut self.final_obs),
            preview: Some(&mut self.preview),
            potential: Some(&mut self.potential),
            reports: Some(&mut self.reports),
        }
    }

    fn preview(&self, i: usize) -> &[f32] {
        &self.preview[i * PREVIEW_LEN..(i + 1) * PREVIEW_LEN]
    }

    fn mask(&self, i: usize) -> &[u8] {
        &self.mask[i * ACTION_COUNT..(i + 1) * ACTION_COUNT]
    }
}

/// A uniformly random index among the set bytes of `mask` (0 if none is set).
fn pick(mask: &[u8], rng: &mut Rng) -> usize {
    let legal: Vec<usize> = (0..mask.len()).filter(|&i| mask[i] == 1).collect();
    if legal.is_empty() {
        return 0;
    }
    legal[rng.below(legal.len() as u32) as usize]
}

/// Random legal actions for every env, each head drawn from the batch's own masks.
fn actions(batch: &mut BattleBatch, arrays: &Arrays, rng: &mut Rng) -> Vec<Action> {
    let n = batch.len();
    let entries: Vec<u32> = (0..n).map(|i| pick(arrays.mask(i), rng) as u32).collect();
    let mut spells = vec![0u8; n * SPELL_COUNT];
    batch.spell_mask(&entries, &mut spells).unwrap();
    let spell: Vec<u16> = spells
        .chunks(SPELL_COUNT)
        .map(|m| pick(m, rng) as u16)
        .collect();
    let mut targets = vec![0u8; n * TARGET_COUNT];
    batch.target_mask(&entries, &spell, &mut targets).unwrap();
    let target: Vec<u16> = targets
        .chunks(TARGET_COUNT)
        .map(|m| pick(m, rng) as u16)
        .collect();
    batch
        .second_mask(&entries, &spell, &target, &mut targets)
        .unwrap();
    let second: Vec<u16> = targets
        .chunks(TARGET_COUNT)
        .map(|m| pick(m, rng) as u16)
        .collect();
    (0..n)
        .map(|i| Action {
            entry: entries[i],
            spell: spell[i],
            target: target[i],
            second: second[i],
        })
        .collect()
}

/// One env of the batch played as a single env from the batch's public pieces: the episode seeds
/// of [`episode_seed`], the scenario of [`draw_scenario`], [`Scenario::env`] and the seats'
/// policies on their streams.
struct Single {
    index: u32,
    started: u32,
    seed: u64,
    scenario: usize,
    env: Option<BattleEnv>,
    seats: [Seat; 2],
    kind: Option<Kind>,
    rngs: [Rng; 2],
    legal: LegalCommands,
}

/// What a single env shows: its report, observation, mask, previewed kills and final
/// observations.
#[derive(Debug, PartialEq)]
struct Shown {
    report: EnvReport,
    obs: Vec<u32>,
    mask: Vec<u8>,
    preview: Vec<u32>,
    final_obs: Option<Vec<u32>>,
    hash: Option<u64>,
}

/// The engine's preview of each legal attack and shot entry of `mask` (`h3_battle::preview` of
/// the entry's command), as the batch lays it out; 0 elsewhere.
fn engine_preview(env: &BattleEnv, side: BattleSide, mask: &ActionMask) -> Vec<f32> {
    let mut preview = vec![0.0; PREVIEW_LEN];
    for j in 0..PREVIEW_ENTRIES {
        let entry = PREVIEW_BASE + j;
        if !mask[entry] {
            continue;
        }
        let state = env.state();
        let command = crate::action::decode(state, env.ruleset(), Action::entry(entry)).unwrap();
        assert!(matches!(
            command,
            Command::Attack { .. } | Command::Shoot { .. }
        ));
        let p = h3_battle::preview(state, env.ruleset(), side, command).unwrap();
        preview[2 * j] = p.kills[0] as f32;
        preview[2 * j + 1] = p.kills[1] as f32;
    }
    preview
}

struct Reference<'a> {
    config: BatchConfig,
    scenarios: &'a [Scenario],
    total: u32,
    singles: Vec<Single>,
}

fn bits(values: &[f32]) -> Vec<u32> {
    values.iter().map(|v| v.to_bits()).collect()
}

impl<'a> Reference<'a> {
    fn new(config: BatchConfig, scenarios: &'a [Scenario]) -> Self {
        let total = scenarios.iter().map(Scenario::weight).sum();
        let mut reference = Self {
            config,
            scenarios,
            total,
            singles: Vec::new(),
        };
        for index in 0..config.envs {
            let mut single = Single {
                index,
                started: 0,
                seed: 0,
                scenario: 0,
                env: None,
                seats: EXTERNAL,
                kind: None,
                rngs: [Rng::from_seed(0), Rng::from_seed(0)],
                legal: LegalCommands::new(),
            };
            reference.begin(&mut single);
            reference.singles.push(single);
        }
        reference
    }

    fn internal(&self, single: &mut Single) -> Option<crate::StepResult> {
        let seats = single.seats;
        let env = single.env.as_mut().unwrap();
        let mut last = None;
        while let Some(side) = env.to_move() {
            let Seat::Internal(policy) = seats[side.index()] else {
                break;
            };
            let command =
                policy.choose(env, side, &mut single.rngs[side.index()], &mut single.legal);
            last = Some(rollout::step_command(env, side, command).unwrap());
        }
        last
    }

    fn begin(&self, single: &mut Single) {
        for _ in 0..MAX_SKIPPED_EPISODES {
            single.seed = episode_seed(self.config.seed, single.index, single.started);
            single.started += 1;
            single.scenario = draw_scenario(self.scenarios, self.total, single.seed);
            single.rngs = [0, 1].map(|s| Rng::for_stream(single.seed, StreamId::seat(s)));
            let episode = self.scenarios[single.scenario]
                .episode(single.seed, self.config.max_steps)
                .unwrap();
            single.seats = episode.seats;
            single.kind = episode.kind;
            single.env = Some(episode.env);
            self.internal(single);
            if !single.env.as_ref().unwrap().is_done() {
                return;
            }
        }
        single.env = None;
    }

    fn step(&mut self, actions: &[Action]) -> Vec<Shown> {
        let mut singles = std::mem::take(&mut self.singles);
        let shown = singles
            .iter_mut()
            .zip(actions)
            .map(|(single, action)| self.step_one(single, *action))
            .collect();
        self.singles = singles;
        shown
    }

    fn step_one(&self, single: &mut Single, action: Action) -> Shown {
        let mut report = EnvReport::default();
        let mut final_obs = None;
        match single.env.as_mut() {
            None => report.error = Some(StepError::EpisodeOver),
            Some(_) if [RETREAT, SURRENDER].contains(&(action.entry as usize)) => {
                report.error = Some(StepError::InvalidAction);
            }
            Some(env) => {
                let side = env.to_move().unwrap();
                match env.step(side, action) {
                    Err(e) => report.error = Some(e),
                    Ok(first) => {
                        let last = self.internal(single).unwrap_or(first);
                        let env = single.env.as_ref().unwrap();
                        if env.is_done() {
                            report.done = true;
                            report.truncated = last.truncated;
                            report.result = last.result;
                            report.reward = env.state().outcome().map_or([0.0; 2], |o| {
                                reward::rewards(self.config.reward, env.ruleset(), env.setup(), &o)
                            });
                            if last.truncated {
                                let mut both = vec![0.0; 2 * OBSERVATION_LEN];
                                let (a, d) = both.split_at_mut(OBSERVATION_LEN);
                                env.encode_observation(BattleSide::Attacker, a).unwrap();
                                env.encode_observation(BattleSide::Defender, d).unwrap();
                                final_obs = Some(bits(&both));
                            }
                            self.begin(single);
                        }
                    }
                }
            }
        }
        self.show(single, report, final_obs)
    }

    fn shown(&mut self) -> Vec<Shown> {
        let singles = std::mem::take(&mut self.singles);
        let shown = singles
            .iter()
            .map(|s| self.show(s, EnvReport::default(), None))
            .collect();
        self.singles = singles;
        shown
    }

    fn show(&self, single: &Single, mut report: EnvReport, final_obs: Option<Vec<u32>>) -> Shown {
        let mut obs = vec![0.0; OBSERVATION_LEN];
        let mut mask = [false; ACTION_COUNT];
        let mut preview = vec![0.0; PREVIEW_LEN];
        let env = single.env.as_ref();
        let to_move = env.and_then(BattleEnv::to_move);
        if let (Some(env), Some(side)) = (env, to_move) {
            env.encode_observation(side, &mut obs).unwrap();
            env.legal_mask(side, &mut LegalCommands::new(), &mut mask);
            mask[RETREAT] = false;
            mask[SURRENDER] = false;
            preview = engine_preview(env, side, &mask);
        }
        report.to_move = to_move;
        report.episode = single.started - 1;
        report.scenario = single.scenario as u32;
        report.encounter = single.kind;
        report.episode_seed = single.seed;
        Shown {
            report,
            obs: bits(&obs),
            mask: mask.iter().map(|m| u8::from(*m)).collect(),
            preview: bits(&preview),
            final_obs,
            hash: env.map(BattleEnv::state_hash),
        }
    }
}

/// What the batch shows for env `i` (`final_obs` only if its report says it was truncated).
fn batch_shown(batch: &BattleBatch, arrays: &Arrays, i: usize) -> Shown {
    let report = arrays.reports[i];
    let rows = 2 * OBSERVATION_LEN;
    Shown {
        report,
        obs: bits(&arrays.obs[i * OBSERVATION_LEN..(i + 1) * OBSERVATION_LEN]),
        mask: arrays.mask(i).to_vec(),
        preview: bits(arrays.preview(i)),
        final_obs: report
            .truncated
            .then(|| bits(&arrays.final_obs[i * rows..(i + 1) * rows])),
        hash: batch.env(i).map(BattleEnv::state_hash),
    }
}

/// Plays `calls` calls of random legal actions through `batch` and the reference and checks that
/// every env shows the same after each, that every action the masks allowed applied, and that the
/// rewards are zero-sum. Returns how many episodes ended, and how many of them with a reward.
fn same_as_single_envs(
    config: BatchConfig,
    scenarios: Vec<Scenario>,
    calls: u32,
    play: u64,
) -> (u32, u32) {
    let mut batch = BattleBatch::new(config).unwrap();
    for s in &scenarios {
        batch.add_scenario(s.clone()).unwrap();
    }
    let n = batch.len();
    let mut arrays = Arrays::new(n);
    batch.reset(&mut arrays.outputs()).unwrap();
    let mut reference = Reference::new(config, &scenarios);
    let expected = reference.shown();
    for (i, e) in expected.iter().enumerate() {
        assert_eq!(
            &batch_shown(&batch, &arrays, i),
            e,
            "env {i} after the reset"
        );
    }
    let mut rng = Rng::for_stream(play, StreamId::AGENT);
    let (mut ended, mut rewarded) = (0, 0);
    for call in 0..calls {
        let live: Vec<bool> = arrays.reports.iter().map(|r| r.to_move.is_some()).collect();
        let actions = actions(&mut batch, &arrays, &mut rng);
        batch.step(&actions, &mut arrays.outputs()).unwrap();
        let expected = reference.step(&actions);
        for (i, e) in expected.iter().enumerate() {
            assert_eq!(&batch_shown(&batch, &arrays, i), e, "env {i}, call {call}");
            // Every action the masks allowed applies; an env that gave up refuses any.
            let error = if live[i] {
                None
            } else {
                Some(StepError::EpisodeOver)
            };
            assert_eq!(e.report.error, error, "env {i}, call {call}");
            ended += u32::from(e.report.done);
            // Zero-sum, and 0 unless the episode was decided in this call (ADR-0025 point 8).
            let r = e.report.reward;
            assert_eq!(r[0], -r[1], "env {i}, call {call}");
            if e.report.result.is_none() {
                assert_eq!(r, [0.0; 2], "env {i}, call {call}");
            }
            rewarded += u32::from(r[0] != 0.0);
        }
    }
    (ended, rewarded)
}

#[test]
fn a_batch_steps_as_single_envs() {
    let scenarios = vec![
        scenario(TEXTS[0], 3, EXTERNAL),
        scenario(TEXTS[1], 2, [Seat::External, RANDOM]),
        scenario(TEXTS[2], 2, [RANDOM, Seat::External]),
        scenario(TEXTS[3], 1, EXTERNAL),
    ];
    let config = BatchConfig {
        envs: 4,
        threads: 3,
        seed: 11,
        max_steps: 120,
        reward: RewardConfig::default(),
    };
    let (ended, rewarded) = same_as_single_envs(config, scenarios, 400, 5);
    assert!(ended > 4, "{ended}");
    assert!(rewarded > 0, "{rewarded} of {ended}");
}

#[test]
fn the_preview_shows_the_kills_of_attacks_and_shots() {
    // AI-Q7 (b): `same_as_single_envs` compares each env's preview with the engine's; here both
    // kinds of strike show kills, and a strike never kills fewer at most than at least.
    let config = BatchConfig {
        envs: 8,
        threads: 2,
        seed: 3,
        max_steps: 200,
        reward: RewardConfig::default(),
    };
    let mut batch = BattleBatch::new(config).unwrap();
    batch.add_scenario(scenario(TEXTS[2], 1, EXTERNAL)).unwrap();
    let mut arrays = Arrays::new(8);
    batch.reset(&mut arrays.outputs()).unwrap();
    let mut rng = Rng::for_stream(4, StreamId::AGENT);
    let shots = (PREVIEW_ENTRIES - 2 * h3_battle::STACK_CAPACITY)..PREVIEW_ENTRIES;
    let (mut attacks, mut shot) = (0, 0);
    for _ in 0..200 {
        for (j, kills) in arrays.preview.chunks_exact(2).enumerate() {
            assert!(kills[0] <= kills[1]);
            if kills[1] > 0.0 {
                *if shots.contains(&(j % PREVIEW_ENTRIES)) {
                    &mut shot
                } else {
                    &mut attacks
                } += 1;
            }
        }
        let actions = actions(&mut batch, &arrays, &mut rng);
        batch.step(&actions, &mut arrays.outputs()).unwrap();
    }
    assert!(attacks > 0 && shot > 0, "{attacks} attacks, {shot} shots");
}

#[test]
fn a_reset_replays_the_same_episodes() {
    let config = BatchConfig {
        envs: 3,
        threads: 0,
        seed: 2,
        max_steps: 0,
        reward: RewardConfig::default(),
    };
    let mut batch = BattleBatch::new(config).unwrap();
    batch
        .add_scenario(scenario(TEXTS[1], 1, [Seat::External, RANDOM]))
        .unwrap();
    batch.add_scenario(scenario(TEXTS[0], 1, EXTERNAL)).unwrap();
    let mut first = Arrays::new(3);
    batch.reset(&mut first.outputs()).unwrap();
    let mut rng = Rng::from_seed(1);
    for _ in 0..20 {
        let actions = actions(&mut batch, &first, &mut rng);
        batch.step(&actions, &mut first.outputs()).unwrap();
    }
    let mut again = Arrays::new(3);
    batch.reset(&mut again.outputs()).unwrap();
    let mut fresh = BattleBatch::new(config).unwrap();
    fresh
        .add_scenario(scenario(TEXTS[1], 1, [Seat::External, RANDOM]))
        .unwrap();
    fresh.add_scenario(scenario(TEXTS[0], 1, EXTERNAL)).unwrap();
    let mut new = Arrays::new(3);
    fresh.reset(&mut new.outputs()).unwrap();
    assert_eq!(again.reports, new.reports);
    assert_eq!(bits(&again.obs), bits(&new.obs));
    for (i, r) in new.reports.iter().enumerate() {
        assert_eq!(r.episode, 0);
        assert_eq!(r.episode_seed, episode_seed(2, i as u32, 0));
    }
}

#[test]
fn escapes_are_never_legal_and_refused() {
    // Both heroes of the full scenario may retreat and surrender (BAT-FLE-001, BAT-FLE-002); the
    // batch masks both out and refuses them as invalid actions (ADR-0025 point 7).
    let config = BatchConfig {
        envs: 2,
        threads: 0,
        seed: 3,
        max_steps: 0,
        reward: RewardConfig::default(),
    };
    let mut batch = BattleBatch::new(config).unwrap();
    batch.add_scenario(scenario(TEXTS[2], 1, EXTERNAL)).unwrap();
    let mut arrays = Arrays::new(2);
    batch.reset(&mut arrays.outputs()).unwrap();
    for i in 0..2 {
        let env = batch.env(i).unwrap();
        let side = env.to_move().unwrap();
        assert!(env.state().may_retreat(side));
    }
    let mut rng = Rng::from_seed(4);
    for _ in 0..30 {
        for i in 0..2 {
            assert_eq!(arrays.mask(i)[RETREAT], 0);
            assert_eq!(arrays.mask(i)[SURRENDER], 0);
        }
        let hashes = [0, 1].map(|i| batch.env(i).unwrap().state_hash());
        let obs = arrays.obs.clone();
        let escapes = [RETREAT, SURRENDER].map(|e| Action {
            entry: e as u32,
            ..Action::default()
        });
        batch.step(&escapes, &mut arrays.outputs()).unwrap();
        for (i, hash) in hashes.iter().enumerate() {
            assert_eq!(arrays.reports[i].error, Some(StepError::InvalidAction));
            assert_eq!(batch.env(i).unwrap().state_hash(), *hash);
        }
        assert_eq!(bits(&arrays.obs), bits(&obs));
        let actions = actions(&mut batch, &arrays, &mut rng);
        batch.step(&actions, &mut arrays.outputs()).unwrap();
        assert!(arrays.reports.iter().all(|r| r.error.is_none()));
    }
}

#[test]
fn every_side_plays_as_a_human_players() {
    // With no retreat facts either (E06 T9b): a Tavern would let the system actor retreat.
    let (ruleset, setup) = h3_pack_battle::synthetic(TEXTS[2]).unwrap();
    let facts = RetreatFacts {
        tavern: true,
        ..RetreatFacts::default()
    };
    let setup = BattleSetup {
        human: [false, false],
        retreat: [facts; 2],
        ..setup
    };
    let s = Scenario::new(Arc::new(ruleset), &setup, false, 1, EXTERNAL).unwrap();
    assert_eq!(s.setup().unwrap().human, [true, true]);
    assert_eq!(s.setup().unwrap().retreat, [RetreatFacts::default(); 2]);
    assert!(Arc::ptr_eq(s.ruleset(), s.env(0, 0).unwrap().ruleset()));
    assert_eq!(s.seats(), Some(EXTERNAL));
    assert!(s.sampler().is_none());
}

#[test]
fn a_hero_casts_through_the_head_calls() {
    let config = BatchConfig {
        envs: 1,
        threads: 0,
        seed: 3,
        max_steps: 0,
        reward: RewardConfig::default(),
    };
    let mut batch = BattleBatch::new(config).unwrap();
    batch.add_scenario(scenario(TEXTS[2], 1, EXTERNAL)).unwrap();
    let mut arrays = Arrays::new(1);
    batch.reset(&mut arrays.outputs()).unwrap();
    let mut casts = 0;
    let mut rng = Rng::from_seed(8);
    for _ in 0..200 {
        // Any other entry than a cast (a stack's cast would need its heads too).
        let mut others = arrays.mask(0).to_vec();
        others[HERO_CAST] = 0;
        others[CREATURE_CAST] = 0;
        let mut action = Action {
            entry: pick(&others, &mut rng) as u32,
            ..Action::default()
        };
        if arrays.mask(0)[HERO_CAST] == 1 {
            let entries = [HERO_CAST as u32];
            let mut spells = vec![0u8; SPELL_COUNT];
            batch.spell_mask(&entries, &mut spells).unwrap();
            let spell = spells.iter().position(|m| *m == 1).unwrap() as u16;
            let mut targets = vec![0u8; TARGET_COUNT];
            batch.target_mask(&entries, &[spell], &mut targets).unwrap();
            let target = targets.iter().position(|m| *m == 1).unwrap() as u16;
            batch
                .second_mask(&entries, &[spell], &[target], &mut targets)
                .unwrap();
            assert_eq!(targets.iter().position(|m| *m == 1), Some(0));
            action = Action {
                entry: HERO_CAST as u32,
                spell,
                target,
                second: 0,
            };
            casts += 1;
        }
        batch.step(&[action], &mut arrays.outputs()).unwrap();
        assert_eq!(arrays.reports[0].error, None);
    }
    assert!(casts > 0);
}

/// The single env's spell, target and second masks of env `i` for `entry` and the earlier heads
/// `(spell, target)`, as 0/1 bytes (all 0 for an env without a side to move).
fn single_heads(
    batch: &BattleBatch,
    i: usize,
    entry: usize,
    (spell, target): (u16, u16),
) -> Vec<u8> {
    let mut legal = LegalCommands::new();
    let mut spells = [false; SPELL_COUNT];
    let mut targets = [false; TARGET_COUNT];
    let mut seconds = [false; TARGET_COUNT];
    if let Some((env, side)) = acting(batch.env(i)) {
        env.spell_mask(side, entry, &mut legal, &mut spells);
        env.target_mask(side, (entry, spell), &mut legal, &mut targets);
        env.second_mask(side, (entry, spell, target), &mut legal, &mut seconds);
    }
    [&spells[..], &targets, &seconds]
        .concat()
        .into_iter()
        .map(u8::from)
        .collect()
}

#[test]
fn the_head_calls_answer_as_the_single_envs() {
    // A non-cast entry's heads come from the entry mask the batch kept at the last call, a cast's
    // from the env: both as the single env's, for every legal entry, the casts, illegal and
    // out-of-range entries, and earlier heads other than 0; an escape's are refused as the escape
    // (ADR-0025 point 7), where the single env allows it too.
    // On the caller's thread and on a worker per env.
    for threads in [0, 2] {
        head_calls_as_single_envs(threads);
    }
}

/// [`the_head_calls_answer_as_the_single_envs`] with `threads` workers.
fn head_calls_as_single_envs(threads: u32) {
    let config = BatchConfig {
        envs: 2,
        threads,
        seed: 4,
        max_steps: 0,
        reward: RewardConfig::default(),
    };
    let mut batch = BattleBatch::new(config).unwrap();
    batch.add_scenario(scenario(TEXTS[2], 1, EXTERNAL)).unwrap();
    batch.add_scenario(scenario(TEXTS[3], 1, EXTERNAL)).unwrap();
    let mut arrays = Arrays::new(2);
    batch.reset(&mut arrays.outputs()).unwrap();
    let mut rng = Rng::from_seed(2);
    let mut legal = LegalCommands::new();
    let mut checked = 0;
    let mut escapes = 0;
    for _ in 0..8 {
        let mut entries = std::collections::BTreeSet::from([
            0,
            RETREAT,
            SURRENDER,
            HERO_CAST,
            CREATURE_CAST,
            ACTION_COUNT - 1,
            ACTION_COUNT,
            u32::MAX as usize,
        ]);
        for i in 0..2 {
            if let Some((env, side)) = acting(batch.env(i)) {
                let mut mask = [false; ACTION_COUNT];
                env.legal_mask(side, &mut legal, &mut mask);
                entries.extend((0..ACTION_COUNT).filter(|&e| mask[e]));
            }
        }
        for &entry in &entries {
            for earlier in [(0, 0), (1, 0), (0, 1)] {
                let n = 2;
                let e = vec![entry as u32; n];
                let (s, t) = (vec![earlier.0; n], vec![earlier.1; n]);
                let mut spells = vec![0u8; n * SPELL_COUNT];
                let mut targets = vec![0u8; n * TARGET_COUNT];
                let mut seconds = vec![0u8; n * TARGET_COUNT];
                batch.spell_mask(&e, &mut spells).unwrap();
                batch.target_mask(&e, &s, &mut targets).unwrap();
                batch.second_mask(&e, &s, &t, &mut seconds).unwrap();
                for i in 0..n {
                    let rows = [
                        &spells[i * SPELL_COUNT..(i + 1) * SPELL_COUNT],
                        &targets[i * TARGET_COUNT..(i + 1) * TARGET_COUNT],
                        &seconds[i * TARGET_COUNT..(i + 1) * TARGET_COUNT],
                    ]
                    .concat();
                    let mut single = single_heads(&batch, i, entry, earlier);
                    if [RETREAT, SURRENDER].contains(&entry) {
                        escapes += usize::from(single.contains(&1));
                        single.fill(0);
                    }
                    assert_eq!(rows, single, "{entry} {earlier:?}");
                }
                checked += 1;
            }
        }
        let actions = actions(&mut batch, &arrays, &mut rng);
        batch.step(&actions, &mut arrays.outputs()).unwrap();
    }
    assert!(checked > 100, "{checked}");
    assert!(escapes > 0, "no escape the single env allows");
}

#[test]
fn truncation_writes_both_last_observations_and_resets() {
    let config = BatchConfig {
        envs: 1,
        threads: 0,
        seed: 9,
        max_steps: 6,
        reward: RewardConfig::default(),
    };
    let mut batch = BattleBatch::new(config).unwrap();
    batch.add_scenario(scenario(TEXTS[1], 1, EXTERNAL)).unwrap();
    let mut arrays = Arrays::new(1);
    batch.reset(&mut arrays.outputs()).unwrap();
    let mut rng = Rng::from_seed(1);
    let mut calls = 0;
    while !arrays.reports[0].done {
        let actions = actions(&mut batch, &arrays, &mut rng);
        batch.step(&actions, &mut arrays.outputs()).unwrap();
        calls += 1;
    }
    let report = arrays.reports[0];
    assert_eq!(calls, 6);
    assert!(report.truncated);
    assert_eq!(report.result, None);
    assert_eq!(report.reward, [0.0, 0.0]);
    assert_eq!(report.episode, 1);
    assert_eq!(report.episode_seed, episode_seed(9, 0, 1));
    assert!(report.to_move.is_some());
    assert!(arrays.final_obs.iter().any(|v| *v != 0.0));
    // Only a truncation writes `final_obs`; without the array the call still steps.
    arrays.final_obs.fill(0.0);
    let actions = actions(&mut batch, &arrays, &mut rng);
    let mut out = Outputs {
        reports: Some(&mut arrays.reports),
        ..Outputs::default()
    };
    batch.step(&actions, &mut out).unwrap();
    assert!(!arrays.reports[0].done);
    assert!(arrays.final_obs.iter().all(|v| *v == 0.0));
}

#[test]
fn an_episode_without_a_decision_is_skipped() {
    // The defender's Drakes (speed 8) act before the attacker's Spearmen (4): with a step limit of
    // one, the internal seat's first step ends every episode before the caller could act. Mixed
    // with a scenario the caller acts in, the batch skips those episodes; alone, it gives up.
    let lopsided = "[scenario]\nid = \"lopsided\"\npacks = [\"synthetic\"]\nprofile = \"synthetic:faithful\"\n\
        [[attacker]]\nslot = 0\ncreature = \"synthetic:spearman\"\ncount = 5\n\
        [[defender]]\nslot = 0\ncreature = \"synthetic:drake\"\ncount = 5\n";
    let skipped = scenario(lopsided, 3, [Seat::External, RANDOM]);
    let decided = scenario(TEXTS[0], 1, EXTERNAL);
    let config = BatchConfig {
        envs: 3,
        threads: 0,
        seed: 5,
        max_steps: 1,
        reward: RewardConfig::default(),
    };
    same_as_single_envs(config, vec![skipped.clone(), decided], 30, 2);
    let mut batch = BattleBatch::new(config).unwrap();
    batch.add_scenario(skipped).unwrap();
    let mut arrays = Arrays::new(3);
    arrays.obs.fill(1.0);
    arrays.mask.fill(1);
    batch.reset(&mut arrays.outputs()).unwrap();
    for (i, r) in arrays.reports.iter().enumerate() {
        assert_eq!(r.to_move, None);
        assert_eq!(r.episode, MAX_SKIPPED_EPISODES - 1);
        assert!(batch.env(i).is_none());
    }
    assert!(arrays.obs.iter().all(|v| *v == 0.0));
    assert!(arrays.mask.iter().all(|m| *m == 0));
    batch
        .step(&[Action::default(); 3], &mut arrays.outputs())
        .unwrap();
    assert!(
        arrays
            .reports
            .iter()
            .all(|r| r.error == Some(StepError::EpisodeOver))
    );
    for entry in [0, HERO_CAST as u32, CREATURE_CAST as u32] {
        let mut spells = vec![1u8; 3 * SPELL_COUNT];
        batch.spell_mask(&[entry; 3], &mut spells).unwrap();
        let mut targets = vec![1u8; 3 * TARGET_COUNT];
        batch
            .target_mask(&[entry; 3], &[0; 3], &mut targets)
            .unwrap();
        let mut seconds = vec![1u8; 3 * TARGET_COUNT];
        batch
            .second_mask(&[entry; 3], &[0; 3], &[0; 3], &mut seconds)
            .unwrap();
        assert!(
            spells
                .iter()
                .chain(&targets)
                .chain(&seconds)
                .all(|m| *m == 0)
        );
    }
}

#[test]
fn refused_actions_leave_their_env_unchanged() {
    let config = BatchConfig {
        envs: 2,
        threads: 0,
        seed: 6,
        max_steps: 0,
        reward: RewardConfig::default(),
    };
    let mut batch = BattleBatch::new(config).unwrap();
    batch.add_scenario(scenario(TEXTS[1], 1, EXTERNAL)).unwrap();
    let mut arrays = Arrays::new(2);
    batch.reset(&mut arrays.outputs()).unwrap();
    let before = batch.env(0).unwrap().state_hash();
    let mut rng = Rng::from_seed(3);
    let mut actions = actions(&mut batch, &arrays, &mut rng);
    actions[0].entry = u32::MAX;
    batch.step(&actions, &mut arrays.outputs()).unwrap();
    assert_eq!(arrays.reports[0].error, Some(StepError::InvalidAction));
    assert_eq!(batch.env(0).unwrap().state_hash(), before);
    assert_eq!(arrays.reports[1].error, None);
}

#[test]
fn errors() {
    let config = BatchConfig {
        envs: 2,
        threads: 0,
        seed: 1,
        max_steps: 0,
        reward: RewardConfig::default(),
    };
    assert_eq!(
        BattleBatch::new(BatchConfig { envs: 0, ..config }).unwrap_err(),
        BatchError::NoEnvs
    );
    let envs = MAX_ENVS + 1;
    assert_eq!(
        BattleBatch::new(BatchConfig { envs, ..config }).unwrap_err(),
        BatchError::TooManyEnvs(envs)
    );
    let (ruleset, setup) = h3_pack_battle::synthetic(TEXTS[1]).unwrap();
    let ruleset = Arc::new(ruleset);
    let new = |setup: &BattleSetup, weight, seats| {
        Scenario::new(ruleset.clone(), setup, false, weight, seats).map(|_| ())
    };
    assert_eq!(new(&setup, 1, [RANDOM; 2]), Err(BatchError::NoExternalSeat));
    assert_eq!(new(&setup, 0, EXTERNAL), Err(BatchError::ZeroWeight));
    let empty = BattleSetup {
        defender: h3_contracts::Army { stacks: Vec::new() },
        ..setup.clone()
    };
    assert!(matches!(
        new(&empty, 1, EXTERNAL),
        Err(BatchError::Setup(_))
    ));

    let mut batch = BattleBatch::new(config).unwrap();
    assert!(!batch.is_empty());
    assert_eq!(batch.config(), config);
    assert!(batch.env(0).is_none());
    assert_eq!(
        batch.reset(&mut Outputs::default()),
        Err(BatchError::NoScenario)
    );
    let heavy = scenario(TEXTS[1], u32::MAX, EXTERNAL);
    batch.add_scenario(heavy.clone()).unwrap();
    assert_eq!(batch.add_scenario(heavy), Err(BatchError::WeightOverflow));
    assert_eq!(batch.scenarios().len(), 1);
    let none = Outputs::default;
    assert_eq!(batch.step(&[], &mut none()), Err(BatchError::NotReset));
    assert_eq!(batch.spell_mask(&[], &mut []), Err(BatchError::NotReset));
    assert_eq!(
        batch.target_mask(&[], &[], &mut []),
        Err(BatchError::NotReset)
    );
    assert_eq!(
        batch.second_mask(&[], &[], &[], &mut []),
        Err(BatchError::NotReset)
    );
    batch.reset(&mut none()).unwrap();
    let buffer = |name, len, needed| Err(BatchError::Buffer { name, len, needed });
    let mut short = vec![0.0; 1];
    let mut bytes = vec![0u8; 1];
    let mut reports = vec![EnvReport::default(); 1];
    assert_eq!(
        batch.reset(&mut Outputs {
            obs: Some(&mut short),
            ..none()
        }),
        buffer("obs", 1, 2 * OBSERVATION_LEN)
    );
    assert_eq!(
        batch.reset(&mut Outputs {
            entry_mask: Some(&mut bytes),
            ..none()
        }),
        buffer("entry_mask", 1, 2 * ACTION_COUNT)
    );
    assert_eq!(
        batch.reset(&mut Outputs {
            final_obs: Some(&mut short),
            ..none()
        }),
        buffer("final_obs", 1, 4 * OBSERVATION_LEN)
    );
    assert_eq!(
        batch.reset(&mut Outputs {
            preview: Some(&mut short),
            ..none()
        }),
        buffer("preview", 1, 2 * PREVIEW_LEN)
    );
    assert_eq!(
        batch.reset(&mut Outputs {
            reports: Some(&mut reports),
            ..none()
        }),
        buffer("reports", 1, 2)
    );
    let two = [Action::default(); 2];
    assert_eq!(batch.step(&two[..1], &mut none()), buffer("actions", 1, 2));
    assert_eq!(
        batch.step(
            &two,
            &mut Outputs {
                obs: Some(&mut short),
                ..none()
            }
        ),
        buffer("obs", 1, 2 * OBSERVATION_LEN)
    );
    assert_eq!(batch.spell_mask(&[0], &mut []), buffer("entries", 1, 2));
    assert_eq!(
        batch.spell_mask(&[0, 0], &mut bytes),
        buffer("out", 1, 2 * SPELL_COUNT)
    );
    assert_eq!(
        batch.target_mask(&[0], &[0, 0], &mut []),
        buffer("entries", 1, 2)
    );
    assert_eq!(
        batch.target_mask(&[0, 0], &[0], &mut []),
        buffer("spells", 1, 2)
    );
    assert_eq!(
        batch.target_mask(&[0, 0], &[0, 0], &mut bytes),
        buffer("out", 1, 2 * TARGET_COUNT)
    );
    assert_eq!(
        batch.second_mask(&[0], &[0, 0], &[0, 0], &mut []),
        buffer("entries", 1, 2)
    );
    assert_eq!(
        batch.second_mask(&[0, 0], &[0], &[0, 0], &mut []),
        buffer("spells", 1, 2)
    );
    assert_eq!(
        batch.second_mask(&[0, 0], &[0, 0], &[0], &mut []),
        buffer("targets", 1, 2)
    );
    assert_eq!(
        batch.second_mask(&[0, 0], &[0, 0], &[0, 0], &mut bytes),
        buffer("out", 1, 2 * TARGET_COUNT)
    );
    let messages = [
        BatchError::NoEnvs,
        BatchError::TooManyEnvs(MAX_ENVS + 1),
        BatchError::NoExternalSeat,
        BatchError::ZeroWeight,
        BatchError::WeightOverflow,
        BatchError::Setup(SetupError::EmptyArmy(BattleSide::Defender)),
        BatchError::NoScenario,
        BatchError::NotReset,
        BatchError::Threads("refused".to_owned()),
        BatchError::Buffer {
            name: "obs",
            len: 1,
            needed: 2,
        },
    ]
    .map(|e| e.to_string());
    assert!(messages.iter().all(|m| !m.is_empty()));
    assert_eq!(messages[1], "65537 envs: a batch holds at most 65536");
    assert_eq!(messages[8], "a thread did not start: refused");
    assert_eq!(messages[9], "`obs` has 1 elements, 2 needed");
}

#[test]
fn scenarios_are_drawn_by_weight() {
    let scenarios = [
        scenario(TEXTS[0], 1, EXTERNAL),
        scenario(TEXTS[1], 3, EXTERNAL),
    ];
    let mut counts = [0u32; 2];
    for seed in 0..4000 {
        counts[draw_scenario(&scenarios, 4, seed)] += 1;
    }
    // 1 : 3 within a few percent.
    assert!((900..1100).contains(&counts[0]), "{counts:?}");
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 8, ..ProptestConfig::default() })]

    /// The determinism property (ADR-0025 points 3 and 4): for random batches and thread counts, a
    /// batch steps as N single envs with the derived seeds, output by output and state hash by
    /// state hash, and every action its masks allow applies.
    #[test]
    fn a_batch_is_n_single_envs(
        envs in 1u32..7,
        threads in 0u32..8,
        seed in any::<u64>(),
        max_steps in prop::sample::select(vec![0u32, 3, 40]),
        currency in prop::sample::select(vec![Currency::AiValue, Currency::Gold]),
        mana_price in 0u32..30,
        picks in prop::collection::vec((0usize..4, 1u32..4, 0usize..5), 1..4),
        play in any::<u64>(),
    ) {
        // The system actor (E06 T7) on either side, a hero's included: its commands always apply.
        let seats = [
            EXTERNAL,
            [Seat::External, RANDOM],
            [RANDOM, Seat::External],
            [Seat::External, SYSTEM],
            [SYSTEM, Seat::External],
        ];
        let scenarios = picks
            .iter()
            .map(|&(text, weight, seat)| scenario(TEXTS[text], weight, seats[seat]))
            .collect();
        let reward = RewardConfig { currency, mana_price };
        let config = BatchConfig { envs, threads, seed, max_steps, reward };
        same_as_single_envs(config, scenarios, 40, play);
    }
}

/// E05 T5b2: the encounter distribution as an episode source, mixed with a fixed scenario: the
/// batch plays its envs bit for bit as single envs of [`Scenario::episode`], with each
/// encounter's seats (the neutral kinds' defender inside the call) and kind.
#[test]
fn encounters_play_as_single_envs() {
    let sampler = Arc::new(crate::encounter::tests::SAMPLER.clone().with_shares());
    assert_eq!(
        Scenario::encounters(Arc::clone(&sampler), 0, None).err(),
        Some(BatchError::ZeroWeight)
    );
    let encounters = Scenario::encounters(Arc::clone(&sampler), 3, None).unwrap();
    assert!(encounters.setup().is_none() && encounters.seats().is_none());
    assert!(Arc::ptr_eq(encounters.ruleset(), sampler.battle()));
    assert!(Arc::ptr_eq(encounters.sampler().unwrap(), &sampler));
    // The episode of a seed is the sampler's encounter of that seed, as a human players' battle.
    let episode = encounters.episode(9, 0).unwrap();
    let encounter = sampler.sample(9);
    assert_eq!(
        (episode.seats, episode.kind),
        (encounter.seats, Some(encounter.kind))
    );
    assert_eq!(episode.env.setup().attacker, encounter.setup.attacker);
    assert_eq!(episode.env.setup().human, [true, true]);
    let fixed = scenario(h3_pack_battle::SYNTHETIC_DUEL, 1, EXTERNAL);
    for threads in [0, 2] {
        let config = BatchConfig {
            envs: 6,
            threads,
            seed: 11,
            max_steps: 300,
            reward: RewardConfig::default(),
        };
        let (ended, _) =
            same_as_single_envs(config, vec![encounters.clone(), fixed.clone()], 150, 5);
        assert!(ended > 0, "threads {threads}");
    }
    // The reports name each encounter's kind, and nothing for the fixed scenario.
    let mut batch = BattleBatch::new(BatchConfig {
        envs: 16,
        threads: 0,
        seed: 3,
        max_steps: 0,
        reward: RewardConfig::default(),
    })
    .unwrap();
    batch.add_scenario(encounters).unwrap();
    batch.add_scenario(fixed).unwrap();
    let mut arrays = Arrays::new(16);
    batch.reset(&mut arrays.outputs()).unwrap();
    for report in &arrays.reports {
        assert_eq!(
            report.encounter.is_some(),
            report.scenario == 0,
            "{report:?}"
        );
    }
    assert!(arrays.reports.iter().any(|r| r.encounter.is_some()));
}

/// E05 T8b: seats given to the encounter distribution replace each encounter's own for every
/// episode (an evaluation against an internal policy), a neutral kind's defender included; both
/// internal are refused. The batch still plays its envs as single envs of [`Scenario::episode`].
#[test]
fn encounter_seats_replace_the_encounters_own() {
    let sampler = Arc::new(crate::encounter::tests::SAMPLER.clone().with_shares());
    assert_eq!(
        Scenario::encounters(Arc::clone(&sampler), 1, Some([RANDOM, RANDOM])).err(),
        Some(BatchError::NoExternalSeat)
    );
    for seats in [[Seat::External, RANDOM], [RANDOM, Seat::External]] {
        let encounters = Scenario::encounters(Arc::clone(&sampler), 1, Some(seats)).unwrap();
        assert_eq!(encounters.seats(), Some(seats));
        let mut kinds = Vec::new();
        for seed in 0..40 {
            let episode = encounters.episode(seed, 0).unwrap();
            assert_eq!(episode.seats, seats);
            kinds.push(episode.kind);
        }
        // Neutral kinds (an internal defender of their own) are among the episodes replaced.
        assert!(kinds.contains(&Some(crate::encounter::Kind::Wandering)));
        let config = BatchConfig {
            envs: 4,
            threads: 0,
            seed: 5,
            max_steps: 200,
            reward: RewardConfig::default(),
        };
        let (ended, _) = same_as_single_envs(config, vec![encounters], 120, 7);
        assert!(ended > 0, "{seats:?}");
    }
}

/// E06 T8b: a world of an env's battle (ADR-0025 point 12a, design §14): its viewer sees the
/// battle in it as in the env; the other side's hero is a guess with the observed public part
/// and a setup without what the viewer may not know; a seed gives the same world, another
/// another; a world plays to its end; a world of no playing env is refused.
#[test]
fn a_world_of_an_env_shows_its_viewer_the_battle_and_plays_to_its_end() {
    let sampler = crate::encounter::tests::SAMPLER.clone();
    let sampler = Arc::new(sampler.with_weights([0, 0, 2, 1, 0, 0]).unwrap());
    let mut batch = BattleBatch::new(BatchConfig {
        envs: 8,
        threads: 0,
        seed: 5,
        max_steps: 500,
        reward: RewardConfig::default(),
    })
    .unwrap();
    let viewer = BattleSide::Attacker;
    assert_eq!(
        batch.determinize(0, viewer, 1).err(),
        Some(BatchError::NotReset)
    );
    let encounters = Scenario::encounters(Arc::clone(&sampler), 1, None).unwrap();
    batch.add_scenario(encounters).unwrap();
    let mut arrays = Arrays::new(8);
    batch.reset(&mut arrays.outputs()).unwrap();
    assert_eq!(
        batch.determinize(8, viewer, 1).err(),
        Some(BatchError::NoEnv(8))
    );
    let mut guessed = 0;
    for i in 0..8 {
        let env = batch.env(i).unwrap();
        for viewer in BattleSide::BOTH {
            let other = viewer.opponent();
            let world = batch.determinize(i, viewer, 7).unwrap();
            assert_eq!(world.observation(viewer), env.observation(viewer));
            let again = batch.determinize(i, viewer, 7).unwrap();
            assert_eq!(world.state_hash(), again.state_hash());
            let another = batch.determinize(i, viewer, 8).unwrap();
            assert_ne!(world.state_hash(), another.state_hash());
            let (setup, guess) = (world.setup(), &world.setup().heroes[other.index()]);
            assert_eq!(setup.funds[other.index()], 0);
            if let Some(view) = env.state().hero(other) {
                // The guess is of the observed type; the world shows its public part, and its
                // setup the battle's starting mana (shown at the start; reward v1 reads it).
                assert_eq!(guess.as_ref().map(|h| h.hero_type), Some(view.hero_type));
                assert_eq!(world.state().hero(other), Some(view));
                let start = env.setup().heroes[other.index()].as_ref().unwrap();
                assert_eq!(guess.as_ref().unwrap().magic.mana, start.magic.mana);
                guessed += 1;
            }
            let mut w = world.fork();
            let result = rollout::random_rollout(&mut w, 3, 0);
            assert!(w.is_done(), "env {i}: {result:?}");
        }
    }
    assert!(guessed >= 8, "{guessed}");
    // A guess the battle refuses is an error; without one the other hero is its type alone.
    let (env, view) = (0..8)
        .find_map(|i| {
            let env = batch.env(i).unwrap();
            env.state().hero(BattleSide::Defender).map(|v| (env, v))
        })
        .unwrap();
    let mut bad = h3_contracts::BattleHero::new(view.hero_type);
    bad.tactics = 9;
    let refused = env.world(BattleSide::Attacker, Some(&bad), 1);
    assert!(matches!(refused, Err(SetupError::InvalidHero { .. })));
    let bare = env.world(BattleSide::Attacker, None, 1).unwrap();
    let hero = bare.setup().heroes[1].as_ref().unwrap();
    let mut alone = h3_contracts::BattleHero::new(view.hero_type);
    alone.magic.mana = env.setup().heroes[1].as_ref().unwrap().magic.mana;
    assert_eq!(*hero, alone);
    assert_eq!(
        bare.observation(BattleSide::Attacker),
        env.observation(BattleSide::Attacker)
    );
}

/// E25 T1a: a world scores a decided battle by the batch's reward v1 (its currency and mana
/// price), the scale of a value head trained on it, not ±1. A world of a battle that hides
/// nothing (the duel: no hero) on its episode's seed is the env's battle: played as the batch
/// plays the env, it ends at the same step with the batch's reward for each side (a single env
/// keeps ±1: `env::tests::rewards_on_decision`).
#[test]
fn a_world_scores_a_decided_battle_by_the_batchs_reward_v1() {
    let n = 6;
    let reward = RewardConfig {
        currency: Currency::AiValue,
        mana_price: 3,
    };
    let config = BatchConfig {
        envs: n as u32,
        threads: 0,
        seed: 11,
        max_steps: 0,
        reward,
    };
    let mut batch = BattleBatch::new(config).unwrap();
    let duel = scenario(h3_pack_battle::SYNTHETIC_DUEL, 1, EXTERNAL);
    batch.add_scenario(duel).unwrap();
    let mut arrays = Arrays::new(n);
    batch.reset(&mut arrays.outputs()).unwrap();
    let mut worlds: Vec<Option<BattleEnv>> = (0..n)
        .map(|i| {
            let r = &arrays.reports[i];
            let world = batch.determinize(i, r.to_move.unwrap(), r.episode_seed);
            let world = world.unwrap();
            assert_eq!(world.state_hash(), batch.env(i).unwrap().state_hash());
            Some(world)
        })
        .collect();
    let mut rng = Rng::for_stream(2, StreamId::AGENT);
    let mut decided = 0;
    for _ in 0..5_000 {
        let actions = actions(&mut batch, &arrays, &mut rng);
        let mut ends = vec![None; n];
        for (i, world) in worlds.iter_mut().enumerate() {
            if let Some(w) = world {
                let side = arrays.reports[i].to_move.unwrap();
                let step = w.step(side, actions[i]).unwrap();
                if !step.done {
                    assert_eq!(step.reward, [0.0; 2]);
                }
                ends[i] = Some(step).filter(|s| s.done);
            }
        }
        batch.step(&actions, &mut arrays.outputs()).unwrap();
        for (i, end) in ends.iter().enumerate() {
            if worlds[i].is_none() {
                continue;
            }
            let report = &arrays.reports[i];
            assert_eq!(report.done, end.is_some(), "env {i}");
            if let Some(step) = end {
                assert!(step.result.is_some() && !step.truncated, "env {i}");
                assert_eq!(step.reward, report.reward, "env {i}");
                assert_eq!(step.reward[0], -step.reward[1]);
                assert!(step.reward[0].abs() < 1.0, "env {i}: {:?}", step.reward);
                worlds[i] = None;
                decided += 1;
            }
        }
        if decided == n {
            break;
        }
    }
    assert_eq!(decided, n);
}

/// E25 T2a: the batch writes each env's potential (`reward::potential` at its reward settings:
/// each side's reward v1 as if the battle ended now) after every call, 0 at an episode's start.
#[test]
fn the_batch_writes_each_envs_potential() {
    let reward = RewardConfig {
        currency: Currency::AiValue,
        mana_price: 3,
    };
    let config = BatchConfig {
        envs: 4,
        threads: 2,
        seed: 9,
        max_steps: 0,
        reward,
    };
    let mut batch = BattleBatch::new(config).unwrap();
    let full = scenario(h3_pack_battle::SYNTHETIC_FULL, 1, EXTERNAL);
    batch.add_scenario(full).unwrap();
    let mut arrays = Arrays::new(4);
    batch.reset(&mut arrays.outputs()).unwrap();
    assert!(arrays.potential.iter().all(|&p| p == 0.0));
    let mut rng = Rng::for_stream(4, StreamId::AGENT);
    let mut moved = false;
    for _ in 0..300 {
        let actions = actions(&mut batch, &arrays, &mut rng);
        batch.step(&actions, &mut arrays.outputs()).unwrap();
        for i in 0..4 {
            let env = batch.env(i).unwrap();
            let expected = reward::potential(reward, env.ruleset(), env.setup(), env.state());
            assert_eq!(arrays.potential[2 * i..2 * i + 2], expected, "env {i}");
            moved |= expected[0] != 0.0;
        }
    }
    assert!(moved);
}
