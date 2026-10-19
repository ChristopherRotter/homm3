//! The bots against Random on the encounter distribution (E05 T7): legal commands only, every
//! battle to its end, and both bots ahead of Random from either side.

use std::sync::Arc;

use h3_battle::LegalCommands;
use h3_contracts::{BattleResult, BattleSide};
use h3_kernel::{Rng, StreamId};

use crate::batch::{Policy, Scenario};
use crate::encounter::tests::SAMPLER;
use crate::rollout;

/// Plays the encounter of `seed` (the distribution's default kinds) with `seats[side]`; returns
/// how it ended (`None`: truncated after 2 000 steps).
fn play(seats: [Policy; 2], seed: u64) -> Option<BattleResult> {
    let source = Scenario::encounters(Arc::new(SAMPLER.clone()), 1, None).unwrap();
    let mut env = source.episode(seed, 2000).unwrap().env;
    let mut rngs = [0, 1].map(|s| Rng::for_stream(seed, StreamId::seat(s)));
    let mut legal = LegalCommands::new();
    while let Some(side) = env.to_move() {
        let i = side.index();
        let command = seats[i].choose(&env, side, &mut rngs[i], &mut legal);
        let step = rollout::step_command(&mut env, side, command);
        assert!(step.is_ok(), "{:?} chose {command:?}: {step:?}", seats[i]);
    }
    env.state().result()
}

/// Of `episodes` encounters played by `bot` against Random from each side, the share `bot` won.
fn rate(bot: Policy, episodes: u64) -> u64 {
    let mut won = 0;
    for seed in 0..episodes {
        for (seats, side) in [
            ([bot, Policy::Random], BattleSide::Attacker),
            ([Policy::Random, bot], BattleSide::Defender),
        ] {
            let result = play(seats, 7_000 + seed);
            won += u64::from(result.is_some_and(|r| r.winner() == Some(side)));
        }
    }
    100 * won / (2 * episodes)
}

#[test]
fn both_bots_play_legal_commands_and_beat_random() {
    let (aggressor, greedy) = (rate(Policy::Aggressor, 30), rate(Policy::Greedy, 30));
    assert!(
        aggressor >= 70 && greedy >= 70,
        "aggressor {aggressor} %, greedy {greedy} %"
    );
}

#[test]
fn the_bots_play_each_other_to_the_end() {
    for seed in 0..10 {
        play([Policy::Aggressor, Policy::Greedy], seed);
        play([Policy::Greedy, Policy::Aggressor], seed);
    }
}
