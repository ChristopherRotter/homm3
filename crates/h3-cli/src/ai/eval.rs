//! `h3 ai eval` (E05 T7, `docs/epics/E05-log.md` §9; E06 T3a, `docs/epics/E06-log.md`): the
//! engine's policies against each other on a held-out set of encounters, the baselines' table.
//!
//! A held-out set is a list of strata, each a batch of episodes: the encounter distribution v1
//! with the stratum's kinds' weights, the episode seeds `episode_seed(seed, e, i)` for `e < envs`,
//! `i < per_env`. The report set v1 (`--set report-v1`, the default; E06 design §5) has one
//! stratum per kind with a player on both sides (a hero battle, a siege a hero defends, a
//! garrison's siege): batch seed `REPORT_V1_SEED + k` for kind k, the kind's weight one-hot, the
//! first episode of each of 500 envs (or of the first `--episodes`), weighed in the overall rates
//! as the distribution weighs the kinds. The neutral set v1 (`--set neutral-v1`; E06 T7c, design
//! §12) is built alike from the neutral kinds (a wandering monster, a pile's or a garrison's
//! guards, a neutral town; `NEUTRAL_V1_SEED + k`), weighed by the census's shares (A1), and has a
//! fixed defender, the system actor: every policy plays only the attacker's seat against it (the
//! system actor too, its own row). The set v0 (`--set v0`, E05's) is one stratum: the default
//! kinds, `EVAL_SEED`, 1 000 episodes over 100 envs. All are the Python evaluation's
//! (`h3env.train.evaluate`), so a trained network's rates against the same seats sit beside this
//! table. Without a fixed defender every ordered pair of distinct policies plays every episode,
//! the first attacking and the second defending; a policy draws from stream `seat(side)` of the
//! episode seed, as an internal seat of the batch does.
//!
//! Reported per pair: each policy's win rate (both sides weighed equally, or the attacker's alone
//! against a fixed defender, the strata by their weights; a draw or a truncation is no win) with
//! its 95 % interval (normal, the stratified variance `Σ_s Σ_side w_s² r (1 − r) / n` over the
//! sides' count squared), per stratum and side, its value efficiency (its mean reward v1, weighed
//! alike), and without a fixed defender every policy's Glicko-2 rating (`rating`): the whole run
//! one rating period, every game against the opponents' ratings at its start (the defaults), so
//! that the ratings sum up every game and not the last few (E05 T9b, the user; one period per game
//! held the deviation near 64 and remembered about 40 games).

use std::fmt::Write as _;

use h3_battle::LegalCommands;
use h3_contracts::{BattleResult, BattleSide};
use h3_kernel::{Rng, StreamId};
use h3_sim::batch::{Policy, Scenario, episode_seed};
use h3_sim::{
    ENCOUNTERS_VERSION, EncounterTable, Kind, RewardConfig, SYSTEM_POLICY_VERSION, Sampler, rollout,
};

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

/// The neutral set v1's base seed: kind k's batch seed is `NEUTRAL_V1_SEED + k`
/// (`h3env.train.evaluate.NEUTRAL_V1_SEED`).
pub const NEUTRAL_V1_SEED: u64 = 0x5EED_E7A1_0003_0000;

/// The neutral set v1's envs per kind, the first episode of each.
pub const NEUTRAL_V1_ENVS: u32 = 500;

/// The kinds the neutral set v1 is stratified by (their indices in `Kind::ALL`): those whose
/// defender is the system actor.
pub const NEUTRAL_V1_KINDS: [usize; 3] = [0, 1, 5];

/// The two-sided 95 % quantile of the normal distribution.
const Z95: f64 = 1.96;

/// The policies `--policies` names: the bots and the system actor (E06 T7).
pub const POLICIES: [(&str, Policy); 4] = [
    ("random", Policy::Random),
    ("aggressor", Policy::Aggressor),
    ("greedy", Policy::Greedy),
    ("system", Policy::System),
];

/// `--policies`' default: every policy (the system actor since E06 T7c, decision 12).
pub const DEFAULT_POLICIES: &str = "random,aggressor,greedy,system";

/// The neutral set's fixed defender (E06 T7c): the system actor, as the batch seats it.
pub const NEUTRAL_DEFENDER: (&str, Policy) = ("system", Policy::System);

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
        let table = EncounterTable::v1();
        let set = ("report set v1", REPORT_V1_SEED, REPORT_V1_ENVS);
        Self::by_kind(set, &REPORT_V1_KINDS, per_kind, |k| table.kinds[k].weight)
    }

    /// The neutral set v1 (E06 T7c), the first `per_kind` of its 500 envs of each neutral kind,
    /// weighed by the census's shares (the distribution draws them only by a run's weights).
    ///
    /// # Errors
    ///
    /// A message: `per_kind` is 0 or above 500.
    pub fn neutral_v1(per_kind: u32) -> Result<Vec<Self>, String> {
        let table = EncounterTable::v1();
        let set = ("neutral set v1", NEUTRAL_V1_SEED, NEUTRAL_V1_ENVS);
        Self::by_kind(set, &NEUTRAL_V1_KINDS, per_kind, |k| table.kinds[k].share)
    }

    /// One stratum per kind of `kinds` of the set `(name, seed, envs)`: batch seed `seed + k`, the
    /// kind one-hot, the first episode of each of `per_kind` envs (at most `envs`), weighed by
    /// `weight(k)`.
    fn by_kind(
        (set, seed, envs): (&str, u64, u32),
        kinds: &[usize],
        per_kind: u32,
        weight: impl Fn(usize) -> u32,
    ) -> Result<Vec<Self>, String> {
        if !(1..=envs).contains(&per_kind) {
            return Err(format!(
                "--episodes: the {set} has 1 to {envs} episodes per kind"
            ));
        }
        Ok(kinds
            .iter()
            .map(|&k| {
                let mut one_hot = [0; 6];
                one_hot[k] = 1;
                Self {
                    name: Kind::ALL[k].key().to_owned(),
                    weight: weight(k),
                    kinds: one_hot,
                    seed: seed + k as u64,
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
    /// The held-out set's name (`report-v1`, `neutral-v1`, `v0`).
    pub set: String,
    /// The policies: at least two, or one with a fixed defender.
    pub policies: Vec<(String, Policy)>,
    /// None: every ordered pair plays. A fixed defender (the neutral set's system actor): every
    /// policy attacks it, and it attacks itself (added to the policies when they lack it).
    pub defender: Option<(String, Policy)>,
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
/// T2c): the encounter distribution's version, the rulesets' content hashes and the system actor's
/// policy version (E06 T7c).
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
    /// `h3_battle_ai::POLICY_VERSION` (`h3_sim::SYSTEM_POLICY_VERSION`).
    pub system_policy: u32,
}

impl Identity {
    /// The identity of the distribution `sampler` draws.
    pub fn of(sampler: &Sampler) -> Self {
        Self {
            encounters: ENCOUNTERS_VERSION,
            core: sampler.battle().core_hash(),
            battle: sampler.battle().content_hash(),
            adventure: sampler.adventure().content_hash(),
            system_policy: SYSTEM_POLICY_VERSION,
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
    /// The policies' names, in order (a fixed defender among them).
    pub names: Vec<String>,
    /// The fixed defender's index in `names` (the neutral set's system actor), or None.
    pub defender: Option<usize>,
    /// `pairs[a][b][s]`: policy `a` attacking policy `b` in stratum `s` (played: `matchups`).
    pub pairs: Vec<Vec<Vec<PairResult>>>,
    /// Every policy's rating after all games (none with a fixed defender).
    pub ratings: Vec<Rating>,
}

/// The ordered pairs `n` policies play, the attacker first: every pair of distinct ones, or every
/// policy against a fixed defender (itself too).
fn matchups(n: usize, defender: Option<usize>) -> Vec<(usize, usize)> {
    match defender {
        Some(d) => (0..n).map(|a| (a, d)).collect(),
        None => (0..n)
            .flat_map(|a| (0..n).filter(move |&b| b != a).map(move |b| (a, b)))
            .collect(),
    }
}

impl EvalReport {
    /// The ordered pairs played, the attacker first ([`EvalReport::pairs`]).
    pub fn matchups(&self) -> Vec<(usize, usize)> {
        matchups(self.names.len(), self.defender)
    }

    /// The sides a policy's rates count: both, or the attacker's alone against a fixed defender.
    fn sides(&self) -> usize {
        if self.defender.is_some() { 1 } else { 2 }
    }

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

    /// Policy `a`'s win rate against `b` in stratum `s`, both sides weighed equally (against a
    /// fixed defender: attacking).
    pub fn stratum_rate(&self, a: usize, b: usize, s: usize) -> f64 {
        let sides = self.sides();
        (0..sides)
            .map(|side| self.side_rate(a, b, s, side))
            .sum::<f64>()
            / sides as f64
    }

    /// Policy `a`'s win rate against `b`: both sides weighed equally (against a fixed defender:
    /// attacking), the strata by their weights.
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
                (0..self.sides())
                    .map(|side| {
                        let (n, wins, _) = self.side(a, b, s, side);
                        let r = wins / n;
                        w * w * r * (1.0 - r) / n
                    })
                    .sum::<f64>()
            })
            .sum();
        let sides = self.sides() as f64;
        Z95 * (variance / (sides * sides)).sqrt()
    }

    /// Policy `a`'s mean reward v1 against `b`, weighed as [`Self::win_rate`].
    pub fn value(&self, a: usize, b: usize) -> f64 {
        self.shares()
            .iter()
            .enumerate()
            .map(|(s, w)| {
                let sides = self.sides();
                let mean: f64 = (0..sides)
                    .map(|side| {
                        let (n, _, reward) = self.side(a, b, s, side);
                        reward / n
                    })
                    .sum();
                w * mean / sides as f64
            })
            .sum()
    }

    /// The episodes each ordered pair played.
    pub fn episodes_per_pair(&self) -> u32 {
        self.strata.iter().map(|s| s.envs * s.per_env).sum()
    }

    /// The report as text: the strata, the win-rate tables (overall with its interval, then per
    /// stratum and side), the value table and the ratings; against a fixed defender one table, a
    /// row per policy.
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
        let against = self.defender.map_or(String::new(), |d| {
            format!(", each attacking {}", self.names[d])
        });
        let _ = writeln!(
            out,
            "set {}: {} episodes per ordered pair{against}, at most {} steps",
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
        if let Some(d) = self.defender {
            self.seat_table(&mut out, d, width);
            return out;
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

    /// The table against the fixed defender `d`: per policy its win rate with the interval, its
    /// value efficiency and its win rate per stratum.
    fn seat_table(&self, out: &mut String, d: usize, width: usize) {
        let _ = writeln!(
            out,
            "attacking {} (win rate ± its 95 % interval, value efficiency, win rate per stratum):",
            self.names[d]
        );
        let _ = write!(out, "{:width$} {:>13} {:>6}", "", "win rate", "value");
        for s in &self.strata {
            let _ = write!(out, " {:>w$}", s.name, w = s.name.len().max(5));
        }
        out.push('\n');
        for a in 0..self.names.len() {
            let rate = format!("{:.3} ± {:.3}", self.win_rate(a, d), self.interval(a, d));
            let _ = write!(
                out,
                "{:width$} {rate:>13} {:>+6.3}",
                self.names[a],
                self.value(a, d)
            );
            for (s, stratum) in self.strata.iter().enumerate() {
                let w = stratum.name.len().max(5);
                let _ = write!(out, " {:>w$.3}", self.side_rate(a, d, s, 0));
            }
            out.push('\n');
        }
    }

    /// The report as JSON (`--json`; the bots' tables `python/records/bots-report-v1.json` and
    /// `bots-neutral-v1.json`, read by `h3env.train.evaluate.bots_table`): schema 2 (E06 T7c: the
    /// fixed `defender`, else null, and the engine's `system_policy`), one line per stratum, pair,
    /// rate and rating; against a fixed defender a rate's strata name the attacker's side alone
    /// and there are no ratings.
    pub fn to_json(&self) -> String {
        let list = |items: Vec<String>| {
            if items.is_empty() {
                "[]".to_owned()
            } else {
                format!("[\n    {}\n  ]", items.join(",\n    "))
            }
        };
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
        for (a, b) in self.matchups() {
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
                    let defending = if self.defender.is_some() {
                        String::new()
                    } else {
                        format!(", \"defender\": {:.4}", self.side_rate(a, b, s, 1))
                    };
                    format!(
                        "\"{}\": {{\"attacker\": {:.4}{defending}}}",
                        stratum.name,
                        self.side_rate(a, b, s, 0)
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
        let ratings = self
            .ratings
            .iter()
            .zip(&self.names)
            .map(|(r, name)| {
                format!(
                    "{{\"policy\": \"{name}\", \"rating\": {:.1}, \"deviation\": {:.1}}}",
                    r.rating, r.deviation
                )
            })
            .collect();
        let names = self
            .names
            .iter()
            .map(|name| format!("\"{name}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let defender = self
            .defender
            .map_or("null".to_owned(), |d| format!("\"{}\"", self.names[d]));
        let id = self.identity;
        format!(
            "{{\n  \"schema\": 2,\n  \"set\": \"{}\",\n  \"max_steps\": {},\n  \"engine\": \
             {{\"encounters\": {}, \"core\": \"{:016x}\", \"battle\": \"{:016x}\", \
             \"adventure\": \"{:016x}\", \"system_policy\": {}}},\n  \"policies\": [{names}],\n  \
             \"defender\": {defender},\n  \"strata\": {},\n  \"pairs\": {},\n  \"rates\": {},\n  \
             \"ratings\": {}\n}}\n",
            self.set,
            self.max_steps,
            id.encounters,
            id.core,
            id.battle,
            id.adventure,
            id.system_policy,
            list(strata),
            list(pairs),
            list(rates),
            list(ratings)
        )
    }
}

/// Plays every ordered pair of `options.policies` (or each against the fixed defender) on every
/// stratum's episodes of the encounter distribution `sampler` draws (its kinds' weights the
/// stratum's).
///
/// # Errors
///
/// A message: fewer than two policies without a fixed defender, a stratum's weights the sampler
/// refuses, or an episode that is no valid battle.
pub fn run(sampler: &Sampler, options: &EvalOptions) -> Result<EvalReport, String> {
    let mut players = options.policies.clone();
    let defender = options.defender.as_ref().map(|(name, policy)| {
        players
            .iter()
            .position(|(n, _)| n == name)
            .unwrap_or_else(|| {
                players.push((name.clone(), *policy));
                players.len() - 1
            })
    });
    let n = players.len();
    if defender.is_none() && n < 2 {
        return Err("eval needs at least two policies".to_owned());
    }
    let played = matchups(n, defender);
    let mut pairs = vec![vec![vec![PairResult::default(); options.strata.len()]; n]; n];
    let mut games: Vec<Vec<(Rating, Score)>> = vec![Vec::new(); n];
    let mut legal = LegalCommands::new();
    let reward = RewardConfig::default();
    for (s, stratum) in options.strata.iter().enumerate() {
        let source = bench::encounters_of(sampler, stratum.kinds)?;
        for i in 0..stratum.per_env {
            for e in 0..stratum.envs {
                let seed = episode_seed(stratum.seed, e, i);
                for &(a, b) in &played {
                    let seats = [players[a].1, players[b].1];
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
    // Against a fixed defender every game has the same opponent: a rating says nothing more than
    // the win rate.
    let ratings = if defender.is_some() {
        Vec::new()
    } else {
        games
            .iter()
            .map(|games| rating::period(Rating::default(), games))
            .collect()
    };
    Ok(EvalReport {
        set: options.set.clone(),
        strata: options.strata.clone(),
        max_steps: options.max_steps,
        identity: Identity::of(sampler),
        names: players.into_iter().map(|(name, _)| name).collect(),
        defender,
        pairs,
        ratings,
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

/// The policies `--policies` names, in order (`DEFAULT_POLICIES` by default).
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
