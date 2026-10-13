//! Wall-time benchmarks of the battle hot path (criterion; local only, not a CI gate).
//! The deterministic regression gate is `cargo xtask bench-check` (instruction counts).
//!
//! Workload: the "full" synthetic scenario (7 stacks per side), random legal play.

use std::hint::black_box;
use std::sync::Arc;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use h3_battle::{BattleState, Command, LegalCommands, NoopSink, apply, legal_commands, observe};
use h3_kernel::{Rng, StreamId};
use h3_sim::action::HERO_CAST;
use h3_sim::{ACTION_COUNT, Action, BattleEnv, OBSERVATION_LEN, observation};

fn mid_game_states() -> (Arc<h3_content_battle::BattleRuleset>, Vec<BattleState>) {
    let (ruleset, setup) = h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_FULL).unwrap();
    let mut states = Vec::new();
    let mut legal = LegalCommands::new();
    for seed in 0..32u64 {
        let mut s = BattleState::new(&ruleset, &setup, seed).unwrap();
        let mut rng = Rng::for_stream(seed, StreamId::AGENT);
        for _ in 0..(seed * 7 % 60) {
            let Some(side) = s.side_to_act() else { break };
            legal_commands(&s, &ruleset, side, &mut legal);
            let c = legal.as_slice()[rng.below(legal.len() as u32) as usize];
            apply(&mut s, &ruleset, side, c, &mut NoopSink).unwrap();
        }
        if !s.is_over() {
            states.push(s);
        }
    }
    (Arc::new(ruleset), states)
}

fn benches(c: &mut Criterion) {
    let (ruleset, states) = mid_game_states();
    let mut legal = LegalCommands::new();

    c.bench_function("legal_commands (full armies, mid-game)", |b| {
        let mut i = 0;
        b.iter(|| {
            let s = &states[i % states.len()];
            i += 1;
            legal_commands(black_box(s), &ruleset, s.side_to_act().unwrap(), &mut legal);
            black_box(legal.len())
        })
    });

    c.bench_function("step = legal_commands + pick + apply", |b| {
        let mut rng = Rng::from_seed(1);
        b.iter_batched_ref(
            || states[rng.below(states.len() as u32) as usize].clone(),
            |s| {
                let side = s.side_to_act().unwrap();
                legal_commands(s, &ruleset, side, &mut legal);
                let cmd: Command = legal.as_slice()[(legal.len() / 2).min(legal.len() - 1)];
                apply(s, &ruleset, side, cmd, &mut NoopSink).unwrap();
            },
            BatchSize::SmallInput,
        )
    });

    c.bench_function("clone BattleState", |b| {
        let s = &states[0];
        b.iter(|| black_box(black_box(s).clone()))
    });

    // The agent's view of a step (E04b T9b): the structured observation of the side to act, then
    // the flat encoding an RL agent reads (encoding v13, OBSERVATION_LEN floats).
    c.bench_function("observe (side to act, mid-game)", |b| {
        let mut i = 0;
        b.iter(|| {
            let s = &states[i % states.len()];
            i += 1;
            black_box(observe(black_box(s), &ruleset, s.side_to_act().unwrap()))
        })
    });

    c.bench_function("observe + encode (side to act, mid-game)", |b| {
        let width = ruleset.battle().field.width;
        let mut out = vec![0.0f32; OBSERVATION_LEN];
        let mut i = 0;
        b.iter(|| {
            let s = &states[i % states.len()];
            i += 1;
            let obs = observe(black_box(s), &ruleset, s.side_to_act().unwrap());
            observation::encode(&obs, width, &mut out).unwrap();
            black_box(out[0])
        })
    });

    c.bench_function("state_hash", |b| {
        let s = &states[0];
        b.iter(|| black_box(black_box(s).state_hash()))
    });

    let (r, setup) = h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_FULL).unwrap();
    let env = BattleEnv::new(Arc::new(r), Arc::new(setup), 1, 0).unwrap();
    c.bench_function("BattleEnv::fork", |b| {
        b.iter(|| black_box(black_box(&env).fork()))
    });

    c.bench_function("BattleEnv step via action mask", |b| {
        let mut mask = [false; ACTION_COUNT];
        let mut rng = Rng::from_seed(2);
        b.iter_batched_ref(
            || {
                let mut e = env.fork();
                e.reset(rng.next_u64()).unwrap();
                e
            },
            |e| {
                let side = e.to_move().unwrap();
                e.legal_mask(side, &mut legal, &mut mask);
                // The last legal non-cast entry (a cast needs its heads).
                let a = mask[..HERO_CAST].iter().rposition(|m| *m).unwrap();
                black_box(e.step(side, Action::entry(a)).unwrap())
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(battle, benches);
criterion_main!(battle);
