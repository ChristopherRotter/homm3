//! `h3 ai eval` (E05 T7, `docs/epics/E05-design.md` §9): the engine's policies against each other
//! on held-out encounters, the baselines' table.
//!
//! Every ordered pair of distinct policies plays the same episodes, the first attacking and the
//! second defending: the encounter distribution v1 (its default kinds, or `--encounters`) of the
//! episode seeds `episode_seed(seed, e, i)` for `e < envs`, `i < ceil(episodes / envs)`. With the
//! default seed these are the held-out episodes of the Python evaluation (`h3env.train.evaluate`,
//! `EVAL_SEED`), so a trained network's rates against the same seats sit beside this table. A
//! policy draws from stream `seat(side)` of the episode seed, as an internal seat of the batch does.
//!
//! Reported per pair: each policy's win rate (both sides weighed equally; a draw or a truncation
//! is no win) and value efficiency (its mean reward v1), and every policy's Glicko-2 rating
//! (`rating`), each game one rating period, episode by episode in a fixed order of the pairs.

use std::fmt::Write as _;

use h3_battle::LegalCommands;
use h3_contracts::{BattleResult, BattleSide};
use h3_kernel::{Rng, StreamId};
use h3_sim::batch::{Policy, Scenario, episode_seed};
use h3_sim::{RewardConfig, rollout};

use super::rating::{self, Rating, Score};

/// The held-out episodes' batch seed (`h3env.train.evaluate.EVAL_SEED`).
pub const EVAL_SEED: u64 = 0x5EED_E7A1_0000_0001;

/// The policies `--policies` names.
pub const POLICIES: [(&str, Policy); 3] = [
    ("random", Policy::Random),
    ("aggressor", Policy::Aggressor),
    ("greedy", Policy::Greedy),
];

/// What `h3 ai eval` plays.
#[derive(Debug, Clone)]
pub struct EvalOptions {
    /// The policies, at least two.
    pub policies: Vec<(String, Policy)>,
    /// Episodes per ordered pair (rounded up to a multiple of `envs`).
    pub episodes: u32,
    /// The envs the episode seeds are spread over, as the Python evaluation's batch.
    pub envs: u32,
    /// The batch seed of the episode seeds.
    pub seed: u64,
    /// Accepted steps of both sides before a battle is truncated.
    pub max_steps: u32,
    /// The encounter kinds' weights.
    pub weights: [u32; 6],
}

/// One ordered pair's results: the first policy attacks.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PairResult {
    /// Episodes played.
    pub episodes: u32,
    /// Won by the attacker, by the defender.
    pub wins: [u32; 2],
    /// The sum of each side's reward v1.
    pub reward: [f64; 2],
}

/// The runner's results.
#[derive(Debug, Clone, PartialEq)]
pub struct EvalReport {
    /// The policies' names, in order.
    pub names: Vec<String>,
    /// `pairs[a][b]`: policy `a` attacking policy `b` (empty on the diagonal).
    pub pairs: Vec<Vec<PairResult>>,
    /// Every policy's rating after all games.
    pub ratings: Vec<Rating>,
}

impl EvalReport {
    /// Policy `a`'s win rate against `b`, both sides weighed equally.
    pub fn win_rate(&self, a: usize, b: usize) -> f64 {
        let (attacking, defending) = (&self.pairs[a][b], &self.pairs[b][a]);
        let rate = |wins: u32, n: u32| f64::from(wins) / f64::from(n.max(1));
        (rate(attacking.wins[0], attacking.episodes) + rate(defending.wins[1], defending.episodes))
            / 2.0
    }

    /// Policy `a`'s mean reward v1 against `b`, both sides weighed equally.
    pub fn value(&self, a: usize, b: usize) -> f64 {
        let (attacking, defending) = (&self.pairs[a][b], &self.pairs[b][a]);
        let mean = |sum: f64, n: u32| sum / f64::from(n.max(1));
        (mean(attacking.reward[0], attacking.episodes)
            + mean(defending.reward[1], defending.episodes))
            / 2.0
    }

    /// The report as text: the win-rate and value tables and the ratings.
    pub fn to_text(&self) -> String {
        let n = self.names.len();
        let width = self.names.iter().map(String::len).max().unwrap_or(0).max(8);
        let mut out = String::new();
        for (title, cell) in [
            (
                "win rate (row against column)",
                Self::win_rate as fn(&Self, usize, usize) -> f64,
            ),
            ("value efficiency (row's mean reward v1)", Self::value),
        ] {
            let _ = writeln!(out, "{title}:");
            let _ = write!(out, "{:width$}", "");
            for name in &self.names {
                let _ = write!(out, " {name:>width$}");
            }
            out.push('\n');
            for a in 0..n {
                let _ = write!(out, "{:width$}", self.names[a]);
                for b in 0..n {
                    if a == b {
                        let _ = write!(out, " {:>width$}", "-");
                    } else {
                        let _ = write!(out, " {:>width$.3}", cell(self, a, b));
                    }
                }
                out.push('\n');
            }
        }
        let _ = writeln!(out, "Glicko-2 ratings:");
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| {
            self.ratings[b]
                .rating
                .total_cmp(&self.ratings[a].rating)
                .then(a.cmp(&b))
        });
        for a in order {
            let r = self.ratings[a];
            let _ = writeln!(
                out,
                "{:width$} {:7.1} ± {:5.1}",
                self.names[a],
                r.rating,
                2.0 * r.deviation
            );
        }
        out
    }
}

/// Plays every ordered pair of `options.policies` on the held-out episodes of `source` (the
/// encounter distribution).
///
/// # Errors
///
/// A message: fewer than two policies, or an episode that is no valid battle.
pub fn run(source: &Scenario, options: &EvalOptions) -> Result<EvalReport, String> {
    let n = options.policies.len();
    if n < 2 {
        return Err("eval needs at least two policies".to_owned());
    }
    let envs = options.envs.max(1);
    let per_env = options.episodes.div_ceil(envs);
    let mut pairs = vec![vec![PairResult::default(); n]; n];
    let mut ratings = vec![Rating::default(); n];
    let mut legal = LegalCommands::new();
    let reward = RewardConfig::default();
    for i in 0..per_env {
        for e in 0..envs {
            let seed = episode_seed(options.seed, e, i);
            for a in 0..n {
                for b in (0..n).filter(|&b| b != a) {
                    let seats = [options.policies[a].1, options.policies[b].1];
                    let (result, rewards) =
                        play(source, seed, options.max_steps, seats, reward, &mut legal)?;
                    let pair = &mut pairs[a][b];
                    pair.episodes += 1;
                    pair.reward[0] += f64::from(rewards[0]);
                    pair.reward[1] += f64::from(rewards[1]);
                    let score = match result.and_then(BattleResult::winner) {
                        Some(BattleSide::Attacker) => {
                            pair.wins[0] += 1;
                            Score::Win
                        }
                        Some(BattleSide::Defender) => {
                            pair.wins[1] += 1;
                            Score::Loss
                        }
                        None => Score::Draw,
                    };
                    (ratings[a], ratings[b]) = rating::game(ratings[a], ratings[b], score);
                }
            }
        }
    }
    Ok(EvalReport {
        names: options
            .policies
            .iter()
            .map(|(name, _)| name.clone())
            .collect(),
        pairs,
        ratings,
    })
}

/// One battle: the encounter of `seed` with `seats[side]`, its result and each side's reward v1
/// (0 when truncated).
fn play(
    source: &Scenario,
    seed: u64,
    max_steps: u32,
    seats: [Policy; 2],
    reward: RewardConfig,
    legal: &mut LegalCommands,
) -> Result<(Option<BattleResult>, [f32; 2]), String> {
    let mut env = source
        .episode(seed, max_steps)
        .map_err(|e| format!("episode {seed:#x}: {e:?}"))?
        .env;
    let mut rngs = [0, 1].map(|s| Rng::for_stream(seed, StreamId::seat(s)));
    while let Some(side) = env.to_move() {
        let i = side.index();
        let command = seats[i].choose(&env, side, &mut rngs[i], legal);
        // A policy plays legal commands, which always apply (property-tested in h3-battle).
        let _ = rollout::step_command(&mut env, side, command);
    }
    let rewards = env.state().outcome().map_or([0.0; 2], |outcome| {
        h3_sim::reward::rewards(reward, env.ruleset(), env.setup(), &outcome)
    });
    Ok((env.state().result(), rewards))
}

/// The policies `--policies` names, in order (`random,aggressor,greedy` by default).
///
/// # Errors
///
/// A message naming an unknown policy.
pub fn policies(value: &str) -> Result<Vec<(String, Policy)>, String> {
    value
        .split(',')
        .map(|name| {
            let name = name.trim();
            POLICIES
                .iter()
                .find(|(n, _)| *n == name)
                .map(|&(n, p)| (n.to_owned(), p))
                .ok_or_else(|| {
                    format!("--policies: unknown policy `{name}` (random, aggressor, greedy)")
                })
        })
        .collect()
}
