//! `h3 ai eval` (E05 T7, `docs/epics/E05-log.md` §9; E06 T3a, `docs/epics/E06-log.md`): the
//! engine's policies against each other on a held-out set of encounters, the baselines' table.
//!
//! A held-out set is a list of strata, each a batch of episodes: the encounter distribution v1
//! with the stratum's kinds' weights, the episode seeds `episode_seed(seed, e, i)` for `e < envs`,
//! `i < per_env`. The report set v1 (`--set report-v1`, the default; E06 design §5) has one
//! stratum per kind with a player on both sides (a hero battle, a siege a hero defends, a
//! garrison's siege): batch seed `REPORT_V1_SEED + k` for kind k, the kind's weight one-hot, the
//! first episode of each of 500 envs (or of the first `--episodes`), weighed in the overall rates
//! as the distribution weighs the kinds. The set v0 (`--set v0`, E05's) is one stratum: the
//! default kinds, `EVAL_SEED`, 1 000 episodes over 100 envs. Both are the Python evaluation's
//! (`h3env.train.evaluate`), so a trained network's rates against the same seats sit beside this
//! table. Every ordered pair of distinct policies plays every episode, the first attacking and the
//! second defending; a policy draws from stream `seat(side)` of the episode seed, as an internal
//! seat of the batch does.
//!
//! Reported per pair: each policy's win rate (both sides weighed equally, the strata by their
//! weights; a draw or a truncation is no win) with its 95 % interval (normal, the stratified
//! variance `¼ Σ_s Σ_side w_s² r (1 − r) / n`), per stratum and side, its value efficiency (its
//! mean reward v1, weighed alike), and every policy's Glicko-2 rating (`rating`): the whole run one
//! rating period, every game against the opponents' ratings at its start (the defaults), so that
//! the ratings sum up every game and not the last few (E05 T9b, the user; one period per game held
//! the deviation near 64 and remembered about 40 games).

use std::fmt::Write as _;

use h3_battle::LegalCommands;
use h3_contracts::{BattleResult, BattleSide};
use h3_kernel::{Rng, StreamId};
use h3_sim::batch::{Policy, Scenario, episode_seed};
use h3_sim::{ENCOUNTERS_VERSION, EncounterTable, Kind, RewardConfig, Sampler, rollout};

use super::bench;
use super::rating::{self, Rating, Score};

/// The held-out set v0's batch seed (`h3env.train.evaluate.EVAL_SEED`).
pub const EVAL_SEED: u64 = 0x5EED_E7A1_0000_0001;

/// The report set v1's base seed: kind k's batch seed is `REPORT_V1_SEED + k`
/// (`h3env.train.evaluate.REPORT_V1_SEED`).
pub const REPORT_V1_SEED: u64 = 0x5EED_E7A1_0001_0000;

/// The report set v1's envs per kind, the first episode of each.
pub const REPORT_V1_ENVS: u32 = 500;

/// The kinds the report set v1 is stratified by (their indices in `Kind::ALL`): those with a
/// player on both sides.
pub const REPORT_V1_KINDS: [usize; 3] = [2, 3, 4];

/// The two-sided 95 % quantile of the normal distribution.
const Z95: f64 = 1.96;

/// The policies `--policies` names: the bots and the system actor (E06 T7), which the default list
/// leaves out until the ladder takes it in.
pub const POLICIES: [(&str, Policy); 4] = [
    ("random", Policy::Random),
    ("aggressor", Policy::Aggressor),
    ("greedy", Policy::Greedy),
    ("system", Policy::System),
];

/// One stratum of a held-out set: a batch of episodes, weighed in the overall rates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stratum {
    /// Its name: its kind's key, or `default`.
    pub name: String,
    /// Its weight in the overall rates.
    pub weight: u32,
    /// The encounter kinds' weights its episodes are drawn with (in `Kind::ALL`'s order).
    pub kinds: [u32; 6],
    /// Its batch seed.
    pub seed: u64,
    /// Its envs.
    pub envs: u32,
    /// The episodes played of each env, its first.
    pub per_env: u32,
}

impl Stratum {
    /// The held-out set v0 (E05's): `episodes` (rounded up to a multiple of `envs`) of the
    /// distribution with the kinds' weights `kinds`, from batch seed `seed`.
    pub fn v0(episodes: u32, envs: u32, seed: u64, kinds: [u32; 6]) -> Vec<Self> {
        let envs = envs.max(1);
        vec![Self {
            name: "default".to_owned(),
            weight: 1,
            kinds,
            seed,
            envs,
            per_env: episodes.div_ceil(envs),
        }]
    }

    /// The report set v1, the first `per_kind` of its 500 envs of each kind.
    ///
    /// # Errors
    ///
    /// A message: `per_kind` is 0 or above 500.
    pub fn report_v1(per_kind: u32) -> Result<Vec<Self>, String> {
        if !(1..=REPORT_V1_ENVS).contains(&per_kind) {
            return Err(format!(
                "--episodes: the report set v1 has 1 to {REPORT_V1_ENVS} episodes per kind"
            ));
        }
        let table = EncounterTable::v1();
        Ok(REPORT_V1_KINDS
            .iter()
            .map(|&k| {
                let mut kinds = [0; 6];
                kinds[k] = 1;
                Self {
                    name: Kind::ALL[k].key().to_owned(),
                    weight: table.kinds[k].weight,
                    kinds,
                    seed: REPORT_V1_SEED + k as u64,
                    envs: per_kind,
                    per_env: 1,
                }
            })
            .collect())
    }
}

/// What `h3 ai eval` plays.
#[derive(Debug, Clone)]
pub struct EvalOptions {
    /// The held-out set's name (`report-v1`, `v0`).
    pub set: String,
    /// The policies, at least two.
    pub policies: Vec<(String, Policy)>,
    /// The held-out set's strata.
    pub strata: Vec<Stratum>,
    /// Accepted steps of both sides before a battle is truncated.
    pub max_steps: u32,
}

/// One ordered pair's results in one stratum: the first policy attacks.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PairResult {
    /// Episodes played.
    pub episodes: u32,
    /// Won by the attacker, by the defender.
    pub wins: [u32; 2],
    /// Truncated at the step limit.
    pub truncated: u32,
    /// The sum of each side's reward v1.
    pub reward: [f64; 2],
}

/// The engine's identity the numbers depend on beside its code (as a run's record has it, E06
/// T2c): the encounter distribution's version and the rulesets' content hashes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Identity {
    /// `ENCOUNTERS_VERSION`.
    pub encounters: u32,
    /// The core ruleset's content hash.
    pub core: u64,
    /// The battle ruleset's.
    pub battle: u64,
    /// The adventure ruleset's.
    pub adventure: u64,
}

impl Identity {
    /// The identity of the distribution `sampler` draws.
    pub fn of(sampler: &Sampler) -> Self {
        Self {
            encounters: ENCOUNTERS_VERSION,
            core: sampler.battle().core_hash(),
            battle: sampler.battle().content_hash(),
            adventure: sampler.adventure().content_hash(),
        }
    }
}

/// The runner's results.
#[derive(Debug, Clone, PartialEq)]
pub struct EvalReport {
    /// The held-out set's name.
    pub set: String,
    /// The strata played.
    pub strata: Vec<Stratum>,
    /// The step limit.
    pub max_steps: u32,
    /// The engine's identity.
    pub identity: Identity,
    /// The policies' names, in order.
    pub names: Vec<String>,
    /// `pairs[a][b][s]`: policy `a` attacking policy `b` in stratum `s` (empty on the diagonal).
    pub pairs: Vec<Vec<Vec<PairResult>>>,
    /// Every policy's rating after all games.
    pub ratings: Vec<Rating>,
}

impl EvalReport {
    /// The strata's shares of the overall rates (their weights, summing to 1).
    fn shares(&self) -> Vec<f64> {
        let total: u32 = self.strata.iter().map(|s| s.weight).sum();
        self.strata
            .iter()
            .map(|s| f64::from(s.weight) / f64::from(total.max(1)))
            .collect()
    }

    /// `a`'s results against `b` in stratum `s` from `side` (0: attacking): the games (at least
    /// one), the wins and the reward's sum.
    fn side(&self, a: usize, b: usize, s: usize, side: usize) -> (f64, f64, f64) {
        let pair = if side == 0 {
            &self.pairs[a][b][s]
        } else {
            &self.pairs[b][a][s]
        };
        (
            f64::from(pair.episodes.max(1)),
            f64::from(pair.wins[side]),
            pair.reward[side],
        )
    }

    /// Policy `a`'s win rate against `b` in stratum `s` from `side` (0: attacking).
    pub fn side_rate(&self, a: usize, b: usize, s: usize, side: usize) -> f64 {
        let (n, wins, _) = self.side(a, b, s, side);
        wins / n
    }

    /// Policy `a`'s win rate against `b` in stratum `s`, both sides weighed equally.
    pub fn stratum_rate(&self, a: usize, b: usize, s: usize) -> f64 {
        (self.side_rate(a, b, s, 0) + self.side_rate(a, b, s, 1)) / 2.0
    }

    /// Policy `a`'s win rate against `b`: both sides weighed equally, the strata by their weights.
    pub fn win_rate(&self, a: usize, b: usize) -> f64 {
        self.shares()
            .iter()
            .enumerate()
            .map(|(s, w)| w * self.stratum_rate(a, b, s))
            .sum()
    }

    /// The half-width of [`Self::win_rate`]'s 95 % interval (normal, the stratified variance).
    pub fn interval(&self, a: usize, b: usize) -> f64 {
        let variance: f64 = self
            .shares()
            .iter()
            .enumerate()
            .map(|(s, w)| {
                (0..2)
                    .map(|side| {
                        let (n, wins, _) = self.side(a, b, s, side);
                        let r = wins / n;
                        w * w * r * (1.0 - r) / n
                    })
                    .sum::<f64>()
            })
            .sum();
        Z95 * (variance / 4.0).sqrt()
    }

    /// Policy `a`'s mean reward v1 against `b`, weighed as [`Self::win_rate`].
    pub fn value(&self, a: usize, b: usize) -> f64 {
        self.shares()
            .iter()
            .enumerate()
            .map(|(s, w)| {
                let mean = |side| {
                    let (n, _, reward) = self.side(a, b, s, side);
                    reward / n
                };
                w * (mean(0) + mean(1)) / 2.0
            })
            .sum()
    }

    /// The episodes each ordered pair played.
    pub fn episodes_per_pair(&self) -> u32 {
        self.strata.iter().map(|s| s.envs * s.per_env).sum()
    }

    /// The report as text: the strata, the win-rate tables (overall with its interval, then per
    /// stratum and side), the value table and the ratings.
    pub fn to_text(&self) -> String {
        let n = self.names.len();
        let width = self
            .names
            .iter()
            .map(String::len)
            .max()
            .unwrap_or(0)
            .max(13);
        let mut out = String::new();
        let _ = writeln!(
            out,
            "set {}: {} episodes per ordered pair, at most {} steps",
            self.set,
            self.episodes_per_pair(),
            self.max_steps
        );
        for s in &self.strata {
            let _ = writeln!(
                out,
                "  {} (weight {}): seed {:#x}, {} envs × {} episode(s)",
                s.name, s.weight, s.seed, s.envs, s.per_env
            );
        }
        let mut table = |title: &str, cell: &dyn Fn(usize, usize) -> String| {
            let _ = writeln!(out, "{title}:");
            let _ = write!(out, "{:width$}", "");
            for name in &self.names {
                let _ = write!(out, " {name:>width$}");
            }
            out.push('\n');
            for a in 0..n {
                let _ = write!(out, "{:width$}", self.names[a]);
                for b in 0..n {
                    let text = if a == b { "-".to_owned() } else { cell(a, b) };
                    let _ = write!(out, " {text:>width$}");
                }
                out.push('\n');
            }
        };
        table(
            "win rate (row against column, ± its 95 % interval)",
            &|a, b| format!("{:.3} ± {:.3}", self.win_rate(a, b), self.interval(a, b)),
        );
        if self.strata.len() > 1 {
            for (s, stratum) in self.strata.iter().enumerate() {
                table(
                    &format!("win rate in {} (attacking / defending)", stratum.name),
                    &|a, b| {
                        format!(
                            "{:.3} / {:.3}",
                            self.side_rate(a, b, s, 0),
                            self.side_rate(a, b, s, 1)
                        )
                    },
                );
            }
        }
        table("value efficiency (row's mean reward v1)", &|a, b| {
            format!("{:+.3}", self.value(a, b))
        });
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

    /// The report as JSON (`--json`; the bots' table `python/records/bots-report-v1.json`, read by
    /// `h3env.train.evaluate.bots_table`): schema 1, one line per stratum, pair, rate and rating.
    pub fn to_json(&self) -> String {
        let n = self.names.len();
        let list = |items: Vec<String>| items.join(",\n    ");
        let strata = self
            .strata
            .iter()
            .map(|s| {
                format!(
                    "{{\"name\": \"{}\", \"weight\": {}, \"seed\": \"{:#018x}\", \"envs\": {}, \
                     \"episodes_per_env\": {}}}",
                    s.name, s.weight, s.seed, s.envs, s.per_env
                )
            })
            .collect();
        let (mut pairs, mut rates) = (Vec::new(), Vec::new());
        for a in 0..n {
            for b in (0..n).filter(|&b| b != a) {
                for (s, stratum) in self.strata.iter().enumerate() {
                    let p = &self.pairs[a][b][s];
                    let mean = |side: usize| p.reward[side] / f64::from(p.episodes.max(1));
                    pairs.push(format!(
                        "{{\"attacker\": \"{}\", \"defender\": \"{}\", \"stratum\": \"{}\", \
                         \"episodes\": {}, \"wins\": [{}, {}], \"truncated\": {}, \
                         \"reward\": [{:.4}, {:.4}]}}",
                        self.names[a],
                        self.names[b],
                        stratum.name,
                        p.episodes,
                        p.wins[0],
                        p.wins[1],
                        p.truncated,
                        mean(0),
                        mean(1)
                    ));
                }
                let by_stratum = self
                    .strata
                    .iter()
                    .enumerate()
                    .map(|(s, stratum)| {
                        format!(
                            "\"{}\": {{\"attacker\": {:.4}, \"defender\": {:.4}}}",
                            stratum.name,
                            self.side_rate(a, b, s, 0),
                            self.side_rate(a, b, s, 1)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                rates.push(format!(
                    "{{\"policy\": \"{}\", \"opponent\": \"{}\", \"win_rate\": {:.4}, \
                     \"interval\": {:.4}, \"value\": {:.4}, \"by_stratum\": {{{by_stratum}}}}}",
                    self.names[a],
                    self.names[b],
                    self.win_rate(a, b),
                    self.interval(a, b),
                    self.value(a, b)
                ));
            }
        }
        let ratings = (0..n)
            .map(|a| {
                format!(
                    "{{\"policy\": \"{}\", \"rating\": {:.1}, \"deviation\": {:.1}}}",
                    self.names[a], self.ratings[a].rating, self.ratings[a].deviation
                )
            })
            .collect();
        let names = self
            .names
            .iter()
            .map(|name| format!("\"{name}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let id = self.identity;
        format!(
            "{{\n  \"schema\": 1,\n  \"set\": \"{}\",\n  \"max_steps\": {},\n  \"engine\": \
             {{\"encounters\": {}, \"core\": \"{:016x}\", \"battle\": \"{:016x}\", \
             \"adventure\": \"{:016x}\"}},\n  \"policies\": [{names}],\n  \"strata\": [\n    \
             {}\n  ],\n  \"pairs\": [\n    {}\n  ],\n  \"rates\": [\n    {}\n  ],\n  \
             \"ratings\": [\n    {}\n  ]\n}}\n",
            self.set,
            self.max_steps,
            id.encounters,
            id.core,
            id.battle,
            id.adventure,
            list(strata),
            list(pairs),
            list(rates),
            list(ratings)
        )
    }
}

/// Plays every ordered pair of `options.policies` on every stratum's episodes of the encounter
/// distribution `sampler` draws (its kinds' weights the stratum's).
///
/// # Errors
///
/// A message: fewer than two policies, a stratum's weights the sampler refuses, or an episode
/// that is no valid battle.
pub fn run(sampler: &Sampler, options: &EvalOptions) -> Result<EvalReport, String> {
    let n = options.policies.len();
    if n < 2 {
        return Err("eval needs at least two policies".to_owned());
    }
    let mut pairs = vec![vec![vec![PairResult::default(); options.strata.len()]; n]; n];
    let mut games: Vec<Vec<(Rating, Score)>> = vec![Vec::new(); n];
    let mut legal = LegalCommands::new();
    let reward = RewardConfig::default();
    for (s, stratum) in options.strata.iter().enumerate() {
        let source = bench::encounters_of(sampler, stratum.kinds)?;
        for i in 0..stratum.per_env {
            for e in 0..stratum.envs {
                let seed = episode_seed(stratum.seed, e, i);
                for a in 0..n {
                    for b in (0..n).filter(|&b| b != a) {
                        let seats = [options.policies[a].1, options.policies[b].1];
                        let (result, rewards) =
                            play(&source, seed, options.max_steps, seats, reward, &mut legal)?;
                        let pair = &mut pairs[a][b][s];
                        pair.episodes += 1;
                        pair.truncated += u32::from(result.is_none());
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
                        games[a].push((Rating::default(), score));
                        games[b].push((Rating::default(), score.reversed()));
                    }
                }
            }
        }
    }
    Ok(EvalReport {
        set: options.set.clone(),
        strata: options.strata.clone(),
        max_steps: options.max_steps,
        identity: Identity::of(sampler),
        names: options
            .policies
            .iter()
            .map(|(name, _)| name.clone())
            .collect(),
        pairs,
        ratings: games
            .iter()
            .map(|games| rating::period(Rating::default(), games))
            .collect(),
    })
}

/// One battle: the encounter of `seed` with `seats[side]`, its result (`None`: truncated) and each
/// side's reward v1 (0 when truncated).
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
        // A policy plays legal commands, which always apply (property-tested in h3-battle); a
        // refusal would repeat itself forever.
        rollout::step_command(&mut env, side, command)
            .map_err(|e| format!("episode {seed:#x}: {command:?} refused: {e:?}"))?;
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
                    format!(
                        "--policies: unknown policy `{name}` (random, aggressor, greedy, system)"
                    )
                })
        })
        .collect()
}
