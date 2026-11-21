//! `h3 ai ceiling` (E25 T42, increment 10, AI-Q20; `docs/epics/E25-log.md`): a lower bound on the
//! play possible against the fair system actor, by rollout policy improvement.
//!
//! The improved seat is the fair system actor ([`Policy::System`]) improved by one step of policy
//! iteration: at each of its decisions every command it may play ([`candidates`]: the legal
//! commands but the escapes, every hero cast with each target and second target, the active
//! stack's cast with each target) is played in copies of the battle to its end (rollouts), the
//! fair actor on both sides, and the command of the best mean reward v1 is played; ties go to the
//! fair actor's own choice, which the seat draws from its stream as the fair actor would. The
//! decision's budget, `--budget` rollouts a candidate on average, is spent by sequential halving
//! (rounds that keep the better half, the fair actor's command always kept), and rollout `r` of a
//! decision plays every candidate still in on the same world with the same streams (common random
//! numbers: the commands' differences are measured on the same luck). The worlds (`--worlds`) are
//! the true battle's copies (`true`: a trusted diagnostic that reads the other hero's spells and
//! the hidden obstacles) or the improved side's worlds (`side`: [`BattleEnv::world`] with the
//! other hero guessed as `BattleBatch::determinize` guesses it, a world per rollout: what a
//! legitimate agent may know). Such a policy does at least as well as the fair actor up to the
//! rollouts' noise, so its results against the fair actor bound the ceiling from below.
//!
//! It plays `h3 ai eval`'s held-out sets: on the report set v1 the improved seat on either side
//! against each opponent (`--opponents`: the fair and the all-seeing system actor), on the neutral
//! set v1 the attacker against the fixed defender (the fair actor), the fair actor's own row
//! beside it. The record: per opponent `h3 ai eval`'s report of the pair (the rates, intervals and
//! values by kind and side), and per category of the fair actor's choice (`CATEGORIES`, the
//! cloning bench's) the improved decisions, how often the improved command differs from it, what
//! it was changed into, and the mean advantage of the played command over it (the rollouts'
//! means). Games run on `--threads`; a game's result depends on its seed alone and the record
//! sums the games in order. Each finished game is appended to `--out` (TOML: the run's settings,
//! then a `[[game]]` per game); a run started again with the same `--out` and settings continues
//! it, skipping the games it holds (a last game a stop cut short is dropped and played again):
//! Ctrl-C loses only the games in progress. `--leave-out` takes games out of the plan (a game
//! whose rollouts stall, E25 T43); the record names them.
//!
//! **A network's games valued** (`--commands`, E25 T44): the seat replays a network's recorded
//! games instead (`h3env.train.inspect --commands`: each game of a held-out set against one
//! opponent, the network's actions in order, from the batch whose episodes `h3 ai eval`'s are bit
//! for bit), checking that each ends as recorded. At each of the network's decisions the fair
//! actor's choice is drawn as the improved seat draws it, and every candidate is valued as
//! above, the network's command kept through the halving beside the fair actor's; the best by the
//! halving, the network's and the fair actor's are then valued again on `--confirm` fresh
//! rollouts (stream `AGENT` of the decision's seed, common to the three), which the halving's
//! choice of the best does not bias. The record adds, by the category of the network's command,
//! of the best and of the fair actor's, what the network's and the fair actor's choices leave
//! against the best (the confirmation's means; the halving's without one), and the `Game`
//! fields of the improved seat hold the network's command against the fair actor's.

use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;

use h3_battle::{
    CastTargets, Command, LegalCommands, cast_targets, castable_spells, creature_cast_spell,
    creature_cast_targets, legal_commands, second_targets,
};
use h3_contracts::{BattleResult, BattleSide};
use h3_kernel::{Rng, StreamId};
use h3_sim::action::{ACTION_ENCODING_VERSION, Action};
use h3_sim::batch::{GUESS_CANDIDATES, Policy, Scenario, episode_seed};
use h3_sim::{BattleEnv, RewardConfig, Sampler, rollout};
use serde::{Deserialize, Serialize};

use super::bench;
use super::eval::{self, EvalReport, Identity, PairResult, Stratum};
use super::rating::{self, Rating, Score};

/// The decision categories (the cloning bench's, `h3env.train.distill.CATEGORIES`): a decision
/// of the tactics phase, then by the command.
pub const CATEGORIES: [&str; 8] = [
    "tactics",
    "wait_defend",
    "move",
    "melee",
    "shot",
    "heal",
    "cast",
    "other",
];

/// The improved seat's name in the reports.
pub const IMPROVED: &str = "rollout-system";

/// Which battle the rollouts play (the module's documentation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Worlds {
    /// The true battle's copies: a trusted diagnostic.
    True,
    /// The improved side's worlds: only what it may know.
    Side,
}

impl Worlds {
    /// `--worlds`' value.
    ///
    /// # Errors
    /// A message for another word.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "true" => Ok(Self::True),
            "side" => Ok(Self::Side),
            _ => Err(format!("--worlds: `{value}` is neither true nor side")),
        }
    }

    /// The value's word.
    pub const fn name(self) -> &'static str {
        match self {
            Self::True => "true",
            Self::Side => "side",
        }
    }
}

/// A run's settings: the header of `--out`, which a continued run must match.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// `report-v1` or `neutral-v1`.
    pub set: String,
    /// The episodes of each kind.
    pub episodes: u32,
    /// The opponents' names (`h3 ai eval`'s policies); the neutral set's is its defender.
    pub opponents: Vec<String>,
    /// The rollouts a candidate gets on average at a decision, spread by sequential halving
    /// (0: the fair actor's own choice).
    pub budget: u32,
    /// The rollouts' worlds.
    pub worlds: Worlds,
    /// The battles' step limit.
    pub max_steps: u32,
    /// Mixed into every decision's rollout seeds.
    pub seed: u64,
    /// The engine's identity (`h3 ai eval`'s: the encounters, the rulesets' hashes, the system
    /// policy's version); filled by [`run`].
    #[serde(default)]
    pub identity: String,
    /// The replayed network's games ([`Recorded::name`]: the network and its file's hash); empty:
    /// the improved seat plays (E25 T44).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub commands: String,
    /// The fresh rollouts a replayed decision's best, network's and fair actor's commands get
    /// (0: none; `--commands` only).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub confirm: u32,
}

/// `skip_serializing_if`'s test of a zero (serde passes a reference).
const fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// A network's recorded games (`--commands`; `h3env.train.inspect --commands`, E25 T44): its
/// actions in each game of a held-out set against one opponent, in order, and how each ended.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Recorded {
    pub record: RecordedHeader,
    #[serde(default)]
    pub game: Vec<RecordedGame>,
    /// The file's hash (FNV-1a; filled by [`Recorded::parse`]).
    #[serde(skip)]
    pub hash: u64,
}

/// The recording's settings.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RecordedHeader {
    /// The network (its file as the recorder was given it).
    pub network: String,
    /// `report-v1` or `neutral-v1`, its episodes a kind, the opponent and the step limit.
    pub set: String,
    pub episodes: u32,
    pub opponent: String,
    pub max_steps: u32,
    /// The action encoding of the actions.
    pub action_encoding: u32,
}

/// A recorded game.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RecordedGame {
    /// Its stratum's kind (`h3 ai eval`'s name), its episode's env, the network's side (0 the
    /// attacker, 1 the defender).
    pub kind: String,
    pub env: u32,
    pub side: u8,
    /// How it ended, as [`Game`]'s fields.
    pub winner: i8,
    pub truncated: bool,
    pub reward: [f64; 2],
    /// The network's actions (entry, spell, target, second), in order.
    pub actions: Vec<[u32; 4]>,
}

impl Recorded {
    /// Reads the recording `path`.
    ///
    /// # Errors
    /// A message: the file does not read, or [`Recorded::parse`]'s.
    pub fn read(path: &Path) -> Result<Self, String> {
        let shown = path.display();
        let text = std::fs::read_to_string(path).map_err(|e| format!("{shown}: {e}"))?;
        Self::parse(&text).map_err(|e| format!("{shown}: {e}"))
    }

    /// A recording's text.
    ///
    /// # Errors
    /// A message: not a recording, or of another action encoding.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut recorded: Self = toml::from_str(text).map_err(|e| e.to_string())?;
        let network = &recorded.record.network;
        if network.is_empty()
            || network
                .chars()
                .any(|c| c == '"' || c == '\\' || c.is_control())
        {
            // The name goes into the records' JSON as it is (the recorder writes a POSIX path).
            return Err(format!(
                "the network's name {network:?} is empty or needs escaping"
            ));
        }
        let version = recorded.record.action_encoding;
        if version != ACTION_ENCODING_VERSION {
            return Err(format!(
                "recorded in action encoding v{version}, the engine's is v{ACTION_ENCODING_VERSION}"
            ));
        }
        recorded.hash = text.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
        });
        Ok(recorded)
    }

    /// The name [`Settings::commands`] holds: the network and the file's hash.
    pub fn name(&self) -> String {
        format!("{} {:016x}", self.record.network, self.hash)
    }
}

/// A game of the run: its opponent, stratum, episode and the improved seat's side (0 the
/// attacker, 1 the defender, 2 none: the neutral set's own row of its defender).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameKey {
    pub opponent: u32,
    pub stratum: u32,
    pub env: u32,
    pub improved: u8,
}

/// A finished game: its result and the improved seat's decisions by category of the fair actor's
/// choice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Game {
    #[serde(flatten)]
    pub key: GameKey,
    /// The winner (0 the attacker, 1 the defender; -1 none: a draw or a cut battle).
    pub winner: i8,
    pub truncated: bool,
    /// Each side's reward v1 (0 when cut).
    pub reward: [f64; 2],
    /// The improved decisions by category.
    pub decisions: [u32; 8],
    /// Those whose command differs from the fair actor's.
    pub differ: [u32; 8],
    /// The differing commands by their own category.
    pub into: [u32; 8],
    /// The sum of the played command's advantage over the fair actor's (the rollouts' means).
    pub advantage: [f64; 8],
    /// The candidates over all improved decisions.
    pub candidates: u64,
    /// The rollouts over all improved decisions.
    pub rollouts: u64,
    /// Candidates a world refused (scored as a loss there).
    pub refused: u32,
    /// A replayed network's decisions valued (`--commands`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valued: Option<Valued>,
}

impl Game {
    fn new(key: GameKey) -> Self {
        Self {
            key,
            winner: -1,
            truncated: false,
            reward: [0.0; 2],
            decisions: [0; 8],
            differ: [0; 8],
            into: [0; 8],
            advantage: [0.0; 8],
            candidates: 0,
            rollouts: 0,
            refused: 0,
            valued: None,
        }
    }
}

/// A replayed network's decisions valued (`--commands`, the module's documentation): "the best"
/// is the halving's choice, a command's value its mean over the confirmation's rollouts (the
/// halving's without one), a regret the best's value less the command's, summed over the
/// decisions.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Valued {
    /// By the network's command's category: the decisions, those where it is the fair actor's,
    /// those where it is the best, its regret and the fair actor's at the same decisions.
    pub net: [u32; 8],
    pub same_base: [u32; 8],
    pub same_best: [u32; 8],
    pub regret: [f64; 8],
    pub base_regret: [f64; 8],
    /// By the best command's category: the decisions and the network's regret.
    pub best: [u32; 8],
    pub missed: [f64; 8],
    /// By the fair actor's command's category: its regret.
    pub own_regret: [f64; 8],
    /// The network's regret by the halving's own means (optimistic: they chose the best).
    pub sample_regret: f64,
    /// The confirmation's rollouts in which the best, the network's and the fair actor's
    /// command were refused (each a loss there, which raises a regret: BAT-STUB-19's casts).
    #[serde(default)]
    pub refused: [u32; 3],
}

impl Valued {
    fn add(&mut self, other: &Self) {
        for c in 0..CATEGORIES.len() {
            self.net[c] += other.net[c];
            self.same_base[c] += other.same_base[c];
            self.same_best[c] += other.same_best[c];
            self.regret[c] += other.regret[c];
            self.base_regret[c] += other.base_regret[c];
            self.best[c] += other.best[c];
            self.missed[c] += other.missed[c];
            self.own_regret[c] += other.own_regret[c];
        }
        self.sample_regret += other.sample_regret;
        for k in 0..3 {
            self.refused[k] += other.refused[k];
        }
    }
}

/// The record of a run.
#[derive(Debug, Clone, PartialEq)]
pub struct CeilingReport {
    pub settings: Settings,
    /// Per opponent the pair's report: the improved seat (index 0) against it.
    pub reports: Vec<EvalReport>,
    /// Per opponent the improved seat's decisions summed ([`Game`]'s fields).
    pub decisions: Vec<Game>,
    pub games: usize,
    /// The games left out (`--leave-out`'s keys).
    pub left_out: Vec<String>,
    /// The seat's name: [`IMPROVED`], or the replayed network's.
    pub seat: String,
    /// Wall-clock seconds of this part of the run.
    pub seconds: f64,
}

/// A run's options.
#[derive(Debug, Clone)]
pub struct CeilingOptions {
    pub settings: Settings,
    /// Worker threads (at least 1).
    pub threads: usize,
    /// The games' file (`None`: none kept).
    pub out: Option<PathBuf>,
    /// The network's games to replay (`--commands`; [`Settings::commands`] names them).
    pub commands: Option<Recorded>,
    /// Games taken out of the plan, as `kind:env:side` (`attacker`, `defender`, or `none`: the
    /// neutral set's own row), for every opponent.
    pub leave_out: Vec<String>,
}

/// The decision category of `command` played in `env` ([`CATEGORIES`]).
pub fn category(env: &BattleEnv, command: Command) -> usize {
    if env.state().round() == 0 {
        return 0;
    }
    match command {
        Command::Wait | Command::Defend => 1,
        Command::Move { .. } => 2,
        Command::Attack { .. } => 3,
        Command::Shoot { .. } => 4,
        Command::Heal { .. } => 5,
        Command::Cast { .. } | Command::CreatureCast { .. } => 6,
        _ => 7,
    }
}

/// Every command `side` may play in `env` into `out`: the legal commands but the escapes (the
/// batch masks Retreat and Surrender out, ADR-0025 point 7), every castable hero spell with each
/// target and each second target, the active stack's cast with each target; only those the
/// action encoding expresses (`legal` is scratch).
pub fn candidates(
    env: &BattleEnv,
    side: BattleSide,
    legal: &mut LegalCommands,
    out: &mut Vec<Command>,
) {
    let (state, ruleset) = (env.state(), env.ruleset());
    out.clear();
    legal_commands(state, ruleset, side, legal);
    out.extend(legal.iter().copied().filter(|c| !c.leaves_battle()));
    let mut targets = CastTargets::new();
    let mut seconds = CastTargets::new();
    for spell in castable_spells(state, ruleset, side).iter() {
        cast_targets(state, ruleset, side, spell, &mut targets);
        for &target in targets.iter() {
            second_targets(state, ruleset, side, (spell, target), &mut seconds);
            if seconds.is_empty() {
                out.push(Command::Cast {
                    spell,
                    target,
                    second: Default::default(),
                });
            }
            out.extend(seconds.iter().map(|&second| Command::Cast {
                spell,
                target,
                second,
            }));
        }
    }
    if creature_cast_spell(state, ruleset, side).is_some() {
        creature_cast_targets(state, ruleset, side, &mut targets);
        out.extend(
            targets
                .iter()
                .map(|&target| Command::CreatureCast { target }),
        );
    }
    out.retain(|&c| h3_sim::action::encode(state, ruleset, c).is_some());
}

/// An improved decision.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Decision {
    /// The command played.
    pub command: Command,
    /// The fair actor's own.
    pub base: Command,
    /// The commands valued.
    pub candidates: u32,
    /// The rollouts played (each a candidate's).
    pub rollouts: u32,
    /// The played command's mean over the rollouts minus the fair actor's.
    pub advantage: f64,
    /// Candidates a world refused.
    pub refused: u32,
}

/// A replayed decision valued (`--commands`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Valuation {
    /// The best command by the halving, the network's and the fair actor's.
    pub commands: [Command; 3],
    /// The commands valued.
    pub candidates: u32,
    /// The rollouts played (each a command's), the confirmation's included.
    pub rollouts: u32,
    /// The three's means over the halving's rollouts (each of them has all).
    pub sample: [f64; 3],
    /// Their means over the confirmation's (0 without one).
    pub confirm: [f64; 3],
    /// Rollouts a world refused.
    pub refused: u32,
    /// The confirmation's rollouts in which each of the three was refused (a loss there).
    pub confirm_refused: [u32; 3],
}

/// What the halving found (`halve`): the best candidate (an index into the scratch's commands,
/// its sums there), the rollout seeds every kept candidate and the best played.
#[derive(Debug, Clone, Copy)]
struct Halved {
    best: usize,
    seeds: u32,
    rollouts: u32,
    refused: u32,
}

/// Scratch buffers of a worker.
#[derive(Default)]
struct Scratch {
    legal: LegalCommands,
    commands: Vec<Command>,
    sums: Vec<f64>,
    active: Vec<usize>,
    /// The kept commands' indices into `commands`.
    kept: Vec<usize>,
}

/// The world rollout `seed` plays (the module's documentation): a true copy or `side`'s world,
/// its stream `seed`'s.
fn world_of(
    env: &BattleEnv,
    side: BattleSide,
    sampler: &Sampler,
    worlds: Worlds,
    seed: u64,
) -> Result<BattleEnv, String> {
    let mut world = match worlds {
        Worlds::True => env.fork(),
        Worlds::Side => {
            // As `BattleBatch::determinize` guesses the other hero.
            let guess = env.state().hero(side.opponent()).and_then(|view| {
                let mut rng = Rng::for_stream(seed, StreamId::ENCOUNTER);
                let siege = env.setup().siege.is_some();
                sampler.guess(&mut rng, &view, siege, GUESS_CANDIDATES)
            });
            env.world(side, guess.as_ref(), seed)
                .map_err(|e| format!("a world refused its guess: {e:?}"))?
        }
    };
    world.reseed(seed);
    Ok(world)
}

/// `side`'s reward v1 for `command` played in `world`, then the fair actor on both sides to the
/// battle's end, its seats drawing from the streams of `seed`; `None` if the world refuses it (a
/// side world can: the other hero's spell defences are guessed there, though the side's own cast
/// candidates show them, BAT-STUB-19; a refused candidate scores a loss in that rollout).
fn value(
    world: &BattleEnv,
    side: BattleSide,
    command: Command,
    seed: u64,
    reward: RewardConfig,
    legal: &mut LegalCommands,
) -> Result<Option<f64>, String> {
    let mut w = world.clone();
    if rollout::step_command(&mut w, side, command).is_err() {
        return Ok(None);
    }
    let mut rngs = [0, 1].map(|s| Rng::for_stream(seed, StreamId::seat(s)));
    while let Some(s) = w.to_move() {
        let c = Policy::System.choose(&w, s, &mut rngs[s.index()], legal);
        // The fair actor plays legal commands, which always apply (property-tested in h3-battle).
        rollout::step_command(&mut w, s, c)
            .map_err(|e| format!("a rollout's {c:?} refused: {e:?}"))?;
    }
    Ok(Some(w.state().outcome().map_or(0.0, |outcome| {
        f64::from(h3_sim::reward::rewards(reward, w.ruleset(), w.setup(), &outcome)[side.index()])
    })))
}

/// The improved choice of `side` in `env` (the module's documentation), the fair actor's own
/// being `base`, its rollouts' seeds from `seed`: the decision's budget (`budget` rollouts a
/// candidate on average) spent by sequential halving. Each round gives every candidate still in
/// the same rollouts (common random numbers) and keeps the better half by mean, the fair
/// actor's command always among them; the last round's best mean is played, ties to the fair
/// actor's command, and its advantage is the paired difference of the two means on the same
/// rollouts.
fn decide(
    env: &BattleEnv,
    side: BattleSide,
    sampler: &Sampler,
    settings: &Settings,
    seed: u64,
    base: Command,
    scratch: &mut Scratch,
) -> Result<Decision, String> {
    let h = halve(env, side, sampler, settings, seed, &[base], scratch)?;
    let at_base = scratch.kept[0];
    Ok(Decision {
        command: scratch.commands[h.best],
        base,
        candidates: u32::try_from(scratch.commands.len()).unwrap_or(u32::MAX),
        rollouts: h.rollouts,
        advantage: if h.seeds == 0 {
            0.0
        } else {
            (scratch.sums[h.best] - scratch.sums[at_base]) / f64::from(h.seeds)
        },
        refused: h.refused,
    })
}

/// [`decide`]'s sequential halving over `side`'s candidates in `env` and the `kept` commands
/// (the first the fair actor's), which stay in every round: into the scratch's commands, their
/// sums over the rollouts and the kept ones' indices (`kept`'s order). With no budget or a
/// single candidate no rollout is played and the best is `kept[0]`.
fn halve(
    env: &BattleEnv,
    side: BattleSide,
    sampler: &Sampler,
    settings: &Settings,
    seed: u64,
    kept: &[Command],
    scratch: &mut Scratch,
) -> Result<Halved, String> {
    let Scratch {
        legal,
        commands,
        sums,
        active,
        kept: at,
    } = scratch;
    candidates(env, side, legal, commands);
    at.clear();
    for &k in kept {
        let i = commands.iter().position(|&c| c == k).unwrap_or_else(|| {
            commands.push(k);
            commands.len() - 1
        });
        at.push(i);
    }
    let n = commands.len();
    sums.clear();
    sums.resize(n, 0.0);
    let mut halved = Halved {
        best: at[0],
        seeds: 0,
        rollouts: 0,
        refused: 0,
    };
    if settings.budget == 0 || n < 2 {
        return Ok(halved);
    }
    active.clear();
    active.extend(0..n);
    let rounds = usize::BITS - (n - 1).leading_zeros();
    let total = u64::from(settings.budget) * n as u64;
    let reward = RewardConfig::default();
    let mut r = 0u32;
    for round in 0..rounds {
        let per = (total / (u64::from(rounds) * active.len() as u64)).max(1);
        for _ in 0..per {
            let rseed = Rng::for_stream(seed, StreamId::env(r)).next_u64();
            r += 1;
            let world = world_of(env, side, sampler, settings.worlds, rseed)?;
            for &c in active.iter() {
                sums[c] += match value(&world, side, commands[c], rseed, reward, legal)? {
                    Some(v) => v,
                    None => {
                        halved.refused += 1;
                        -1.0
                    }
                };
            }
            halved.rollouts += u32::try_from(active.len()).unwrap_or(u32::MAX);
        }
        if round + 1 < rounds {
            // Every candidate still in has the same rollouts: the sums rank as the means. The
            // better half by sum (ties to the earlier candidate), then the kept commands.
            active.sort_by(|&a, &b| sums[b].total_cmp(&sums[a]).then(a.cmp(&b)));
            active.truncate(active.len().div_ceil(2));
            for &k in at.iter() {
                if !active.contains(&k) {
                    active.push(k);
                }
            }
            active.sort_unstable();
        }
    }
    for &c in active.iter() {
        if sums[c] > sums[halved.best] {
            halved.best = c;
        }
    }
    halved.seeds = r;
    Ok(halved)
}

/// The means of `commands` over `n` rollouts of their own (stream `AGENT` of `seed`, apart from
/// the halving's), common to the three; a command equal to an earlier one shares its rollout.
/// Returns the means, the rollouts played and per command those refused (each a loss).
fn confirmation(
    env: &BattleEnv,
    side: BattleSide,
    sampler: &Sampler,
    worlds: Worlds,
    (seed, n): (u64, u32),
    commands: [Command; 3],
    legal: &mut LegalCommands,
) -> Result<([f64; 3], u32, [u32; 3]), String> {
    let (mut sums, mut rollouts, mut refused) = ([0.0; 3], 0, [0; 3]);
    if n == 0 || commands.iter().all(|&c| c == commands[0]) {
        return Ok((sums, rollouts, refused));
    }
    let reward = RewardConfig::default();
    let mut seeds = Rng::for_stream(seed, StreamId::AGENT);
    for _ in 0..n {
        let rseed = seeds.next_u64();
        let world = world_of(env, side, sampler, worlds, rseed)?;
        let mut v: [Option<f64>; 3] = [None; 3];
        for k in 0..3 {
            v[k] = match (0..k).find(|&j| commands[j] == commands[k]) {
                Some(j) => v[j],
                None => {
                    rollouts += 1;
                    value(&world, side, commands[k], rseed, reward, legal)?
                }
            };
            refused[k] += u32::from(v[k].is_none());
            sums[k] += v[k].unwrap_or(-1.0);
        }
    }
    Ok((sums.map(|s| s / f64::from(n)), rollouts, refused))
}

/// The network's command `net` of `side` in `env` valued (the module's documentation), the
/// fair actor's being `base`: the halving with both kept, then the confirmation.
fn value_replayed(
    env: &BattleEnv,
    side: BattleSide,
    sampler: &Sampler,
    settings: &Settings,
    seed: u64,
    (base, net): (Command, Command),
    scratch: &mut Scratch,
) -> Result<Valuation, String> {
    let h = halve(env, side, sampler, settings, seed, &[base, net], scratch)?;
    let at = [h.best, scratch.kept[1], scratch.kept[0]];
    let commands = at.map(|i| scratch.commands[i]);
    let sample = at.map(|i| {
        if h.seeds == 0 {
            0.0
        } else {
            scratch.sums[i] / f64::from(h.seeds)
        }
    });
    let (confirm, rollouts, confirm_refused) = confirmation(
        env,
        side,
        sampler,
        settings.worlds,
        (seed, settings.confirm),
        commands,
        &mut scratch.legal,
    )?;
    Ok(Valuation {
        commands,
        candidates: u32::try_from(scratch.commands.len()).unwrap_or(u32::MAX),
        rollouts: h.rollouts + rollouts,
        sample,
        confirm,
        refused: h.refused + confirm_refused.iter().sum::<u32>(),
        confirm_refused,
    })
}

/// The seed of the rollouts of the decision at step `steps` of `side` in the episode of
/// `episode` (the run's `seed` mixed in).
fn decision_seed(run: u64, episode: u64, steps: u32, side: BattleSide) -> u64 {
    let stream = steps
        .wrapping_mul(2)
        .wrapping_add(u32::from(side == BattleSide::Defender));
    Rng::for_stream(episode ^ run, StreamId::env(stream)).next_u64()
}

/// A seat of a game.
#[derive(Debug, Clone, Copy)]
enum Player {
    Improved,
    /// The recorded network's (`--commands`).
    Replayed,
    Engine(Policy),
}

/// The run's strata, opponents and games, in order; with `--commands` each game's recording
/// (its index in the file) and the games are the recording's.
#[derive(Debug)]
struct Plan {
    strata: Vec<Stratum>,
    opponents: Vec<(String, Policy)>,
    neutral: bool,
    games: Vec<GameKey>,
    recorded: Vec<usize>,
    /// The games `--leave-out` took out.
    left_out: Vec<GameKey>,
}

/// The name of the improved seat's side `improved` ([`GameKey`]'s) in `--leave-out`'s keys.
const SEATS: [&str; 3] = ["attacker", "defender", "none"];

impl Plan {
    /// The plan of `settings`, its games `commands`' when given, but those `leave_out` names.
    fn of(
        settings: &Settings,
        commands: Option<&Recorded>,
        leave_out: &[String],
    ) -> Result<Self, String> {
        let mut plan = Self::all(settings)?;
        if let Some(recorded) = commands {
            plan.replay(settings, recorded)?;
        } else if !settings.commands.is_empty() || settings.confirm != 0 {
            return Err("--confirm needs --commands, the games it values".to_owned());
        }
        for key in leave_out {
            let parts: Vec<&str> = key.split(':').collect();
            let found = match parts[..] {
                [kind, env, seat] => plan
                    .strata
                    .iter()
                    .position(|s| s.name == kind)
                    .zip(env.parse::<u32>().ok())
                    .zip(SEATS.iter().position(|&s| s == seat)),
                _ => None,
            };
            let Some(((stratum, env), seat)) = found else {
                return Err(format!(
                    "--leave-out: `{key}` is not kind:env:side (attacker, defender or none)"
                ));
            };
            let out = |g: &GameKey| {
                g.stratum as usize == stratum && g.env == env && usize::from(g.improved) == seat
            };
            if !plan.games.iter().any(out) {
                return Err(format!("--leave-out: `{key}` is no game of the run"));
            }
            let mut i = 0;
            while i < plan.games.len() {
                if out(&plan.games[i]) {
                    plan.left_out.push(plan.games.remove(i));
                    if !plan.recorded.is_empty() {
                        plan.recorded.remove(i);
                    }
                } else {
                    i += 1;
                }
            }
        }
        Ok(plan)
    }

    /// The plan's games become `recorded`'s, ordered as the plan's own (by stratum, env and
    /// side), after checking that it is of these settings.
    fn replay(&mut self, settings: &Settings, recorded: &Recorded) -> Result<(), String> {
        let r = &recorded.record;
        if settings.commands != recorded.name() {
            return Err(format!(
                "the settings name the recording `{}`, not `{}`",
                settings.commands,
                recorded.name()
            ));
        }
        let ours = (
            settings.set.as_str(),
            settings.episodes,
            settings.opponents.as_slice(),
            settings.max_steps,
        );
        if ours
            != (
                r.set.as_str(),
                r.episodes,
                [r.opponent.clone()].as_slice(),
                r.max_steps,
            )
        {
            return Err(format!(
                "--commands: recorded on {} ({} a kind, against {}, step limit {}), not {ours:?}",
                r.set, r.episodes, r.opponent, r.max_steps
            ));
        }
        let mut games = Vec::with_capacity(recorded.game.len());
        for (i, g) in recorded.game.iter().enumerate() {
            let stratum = self.strata.iter().position(|s| s.name == g.kind);
            let key = stratum.map(|s| GameKey {
                opponent: 0,
                stratum: u32::try_from(s).unwrap_or(u32::MAX),
                env: g.env,
                improved: g.side,
            });
            let Some(key) = key.filter(|k| self.games.contains(k)) else {
                return Err(format!(
                    "--commands: game {i} ({} env {} side {}) is not of the set",
                    g.kind, g.env, g.side
                ));
            };
            games.push((key, i));
        }
        games.sort_by_key(|&(k, _)| (k.stratum, k.env, k.improved));
        if let Some(w) = games.windows(2).find(|w| w[0].0 == w[1].0) {
            return Err(format!("--commands: game {:?} recorded twice", w[0].0));
        }
        (self.games, self.recorded) = games.into_iter().unzip();
        Ok(())
    }

    /// Every game of `settings`' set and opponents.
    fn all(settings: &Settings) -> Result<Self, String> {
        let neutral = match settings.set.as_str() {
            "report-v1" => false,
            "neutral-v1" => true,
            set => {
                return Err(format!(
                    "--set: unknown set `{set}` (report-v1, neutral-v1)"
                ));
            }
        };
        let strata = if neutral {
            Stratum::neutral_v1(settings.episodes)?
        } else {
            Stratum::report_v1(settings.episodes)?
        };
        let opponents = eval::policies(&settings.opponents.join(","))?;
        if neutral
            && opponents
                != [(
                    eval::NEUTRAL_DEFENDER.0.to_owned(),
                    eval::NEUTRAL_DEFENDER.1,
                )]
        {
            return Err(
                "--opponents: the neutral set's opponent is its defender, system".to_owned(),
            );
        }
        let seats: &[u8] = if neutral { &[0, 2] } else { &[0, 1] };
        let mut games = Vec::new();
        for opponent in 0..opponents.len() {
            for (stratum, s) in strata.iter().enumerate() {
                for env in 0..s.envs {
                    for &improved in seats {
                        games.push(GameKey {
                            opponent: u32::try_from(opponent).unwrap_or(u32::MAX),
                            stratum: u32::try_from(stratum).unwrap_or(u32::MAX),
                            env,
                            improved,
                        });
                    }
                }
            }
        }
        Ok(Self {
            strata,
            opponents,
            neutral,
            games,
            recorded: Vec::new(),
            left_out: Vec::new(),
        })
    }

    /// `key` as `--leave-out` names it.
    fn name(&self, key: GameKey) -> String {
        format!(
            "{}:{}:{}",
            self.strata[key.stratum as usize].name,
            key.env,
            SEATS[usize::from(key.improved)]
        )
    }
}

/// Plays the game `key` (its episode's seed `h3 ai eval`'s), replaying `recorded` when given.
fn play(
    key: GameKey,
    plan: &Plan,
    sources: &[Scenario],
    sampler: &Sampler,
    settings: &Settings,
    recorded: Option<&RecordedGame>,
) -> Result<Game, String> {
    let stratum = &plan.strata[key.stratum as usize];
    let seed = episode_seed(stratum.seed, key.env, 0);
    let opponent = plan.opponents[key.opponent as usize].1;
    let seat = if recorded.is_some() {
        Player::Replayed
    } else {
        Player::Improved
    };
    let players = match key.improved {
        0 => [seat, Player::Engine(opponent)],
        1 => [Player::Engine(opponent), seat],
        _ => [Player::Engine(Policy::System), Player::Engine(opponent)],
    };
    let mut env = sources[key.stratum as usize]
        .episode(seed, settings.max_steps)
        .map_err(|e| format!("episode {seed:#x}: {e:?}"))?
        .env;
    let mut rngs = [0, 1].map(|s| Rng::for_stream(seed, StreamId::seat(s)));
    let mut scratch = Scratch::default();
    let mut game = Game::new(key);
    let mut actions = recorded.map_or(&[][..], |r| &r.actions[..]).iter();
    while let Some(side) = env.to_move() {
        let i = side.index();
        let command = match players[i] {
            Player::Engine(policy) => policy.choose(&env, side, &mut rngs[i], &mut scratch.legal),
            Player::Replayed => {
                let at = format!("{} at step {}", plan.name(key), env.steps());
                let &[entry, spell, target, second] = actions
                    .next()
                    .ok_or_else(|| format!("{at}: the recorded actions end before the battle"))?;
                let narrow = |x: u32| u16::try_from(x).map_err(|_| format!("{at}: head {x}"));
                let action = Action {
                    entry,
                    spell: narrow(spell)?,
                    target: narrow(target)?,
                    second: narrow(second)?,
                };
                let net = h3_sim::action::decode(env.state(), env.ruleset(), action)
                    .ok_or_else(|| format!("{at}: the recorded {action:?} decodes to nothing"))?;
                let base = Policy::System.choose(&env, side, &mut rngs[i], &mut scratch.legal);
                let seed = decision_seed(settings.seed, seed, env.steps(), side);
                let v = value_replayed(
                    &env,
                    side,
                    sampler,
                    settings,
                    seed,
                    (base, net),
                    &mut scratch,
                )?;
                tally(&mut game, &env, &v, settings.confirm != 0);
                net
            }
            Player::Improved => {
                let base = Policy::System.choose(&env, side, &mut rngs[i], &mut scratch.legal);
                let at = decision_seed(settings.seed, seed, env.steps(), side);
                let d = decide(&env, side, sampler, settings, at, base, &mut scratch)?;
                let from = category(&env, base);
                game.decisions[from] += 1;
                game.advantage[from] += d.advantage;
                game.candidates += u64::from(d.candidates);
                game.rollouts += u64::from(d.rollouts);
                game.refused += d.refused;
                if d.command != base {
                    game.differ[from] += 1;
                    game.into[category(&env, d.command)] += 1;
                }
                d.command
            }
        };
        rollout::step_command(&mut env, side, command)
            .map_err(|e| format!("episode {seed:#x}: {command:?} refused: {e:?}"))?;
    }
    let result = env.state().result();
    game.truncated = result.is_none();
    game.winner = match result.and_then(BattleResult::winner) {
        Some(BattleSide::Attacker) => 0,
        Some(BattleSide::Defender) => 1,
        None => -1,
    };
    if let Some(outcome) = env.state().outcome() {
        let r = h3_sim::reward::rewards(
            RewardConfig::default(),
            env.ruleset(),
            env.setup(),
            &outcome,
        );
        game.reward = r.map(f64::from);
    }
    if let Some(r) = recorded {
        // The replay is the recorded game only if it ends as recorded, every action played.
        let ended = (game.winner, game.truncated, game.reward);
        if actions.next().is_some() || ended != (r.winner, r.truncated, r.reward) {
            return Err(format!(
                "{}: the replay ends {ended:?} with {} recorded actions left, the recording {:?}: \
                 recorded on another engine?",
                plan.name(key),
                actions.len() + 1,
                (r.winner, r.truncated, r.reward)
            ));
        }
    }
    Ok(game)
}

/// A replayed decision `v` in `env` into `game` (the [`Valued`] values by the confirmation's
/// means if `confirmed`, else by the halving's; [`Game`]'s own fields the network's command
/// against the fair actor's by the halving's).
fn tally(game: &mut Game, env: &BattleEnv, v: &Valuation, confirmed: bool) {
    let [best, net, base] = v.commands;
    let [c_best, c_net, c_base] = v.commands.map(|c| category(env, c));
    let means = if confirmed { v.confirm } else { v.sample };
    let (regret, base_regret) = (means[0] - means[1], means[0] - means[2]);
    game.decisions[c_base] += 1;
    game.advantage[c_base] += v.sample[1] - v.sample[2];
    game.candidates += u64::from(v.candidates);
    game.rollouts += u64::from(v.rollouts);
    game.refused += v.refused;
    if net != base {
        game.differ[c_base] += 1;
        game.into[c_net] += 1;
    }
    let valued = game.valued.get_or_insert_with(Valued::default);
    valued.net[c_net] += 1;
    valued.same_base[c_net] += u32::from(net == base);
    valued.same_best[c_net] += u32::from(net == best);
    valued.regret[c_net] += regret;
    valued.base_regret[c_net] += base_regret;
    valued.best[c_best] += 1;
    valued.missed[c_best] += regret;
    valued.own_regret[c_base] += base_regret;
    valued.sample_regret += v.sample[0] - v.sample[1];
    for k in 0..3 {
        valued.refused[k] += v.confirm_refused[k];
    }
}

/// `--out`'s contents.
#[derive(Serialize, Deserialize)]
struct File {
    settings: Settings,
    #[serde(default)]
    game: Vec<Game>,
}

/// `--out`'s header (an empty `game` list there would clash with the `[[game]]` appended).
#[derive(Serialize)]
struct Header<'a> {
    settings: &'a Settings,
}

/// A game as `--out` appends it.
#[derive(Serialize)]
struct Appended<'a> {
    game: [&'a Game; 1],
}

/// Runs `options` (the module's documentation; `log` takes the progress lines), continuing the
/// games of `--out` if it holds them.
///
/// # Errors
/// A message: unknown settings, an `--out` of other settings or not readable, an episode that is
/// no valid battle.
pub fn run(
    sampler: &Sampler,
    options: &CeilingOptions,
    log: &mut dyn FnMut(&str),
) -> Result<CeilingReport, String> {
    let start = std::time::Instant::now();
    let mut settings = options.settings.clone();
    settings.identity = format!("{:?}", Identity::of(sampler));
    let plan = Plan::of(&settings, options.commands.as_ref(), &options.leave_out)?;
    let mut done: Vec<Option<Game>> = vec![None; plan.games.len()];
    let mut file = match &options.out {
        Some(path) => Some(open(path, &settings, &plan, &mut done)?),
        None => None,
    };
    let todo: Vec<usize> = (0..done.len()).filter(|&i| done[i].is_none()).collect();
    log(&format!(
        "{} games, {} to play on {} threads ({} held)",
        done.len(),
        todo.len(),
        options.threads.max(1),
        done.len() - todo.len()
    ));
    let sources = plan
        .strata
        .iter()
        .map(|s| bench::encounters_of(sampler, s.kinds))
        .collect::<Result<Vec<_>, _>>()?;
    let next = AtomicUsize::new(0);
    let failed = AtomicBool::new(false);
    let (tx, rx) = mpsc::channel();
    let mut error = None;
    std::thread::scope(|scope| {
        for _ in 0..options.threads.max(1) {
            let tx = tx.clone();
            let (next, failed, todo, plan, sources, settings) =
                (&next, &failed, &todo, &plan, &sources, &settings);
            let commands = options.commands.as_ref();
            scope.spawn(move || {
                while !failed.load(Ordering::Relaxed) {
                    let Some(&i) = todo.get(next.fetch_add(1, Ordering::Relaxed)) else {
                        break;
                    };
                    let recorded = commands.map(|c| &c.game[plan.recorded[i]]);
                    let game = play(plan.games[i], plan, sources, sampler, settings, recorded);
                    if tx.send((i, game)).is_err() {
                        break;
                    }
                }
            });
        }
        drop(tx);
        let every = (todo.len() / 50).max(1);
        for (n, (i, game)) in rx.into_iter().enumerate() {
            let kept = game.and_then(|game| {
                if let Some(f) = file.as_mut() {
                    append(f, &game)?;
                }
                done[i] = Some(game);
                Ok(())
            });
            if let Err(e) = kept {
                failed.store(true, Ordering::Relaxed);
                if error.is_none() {
                    error = Some(e);
                }
            }
            if (n + 1) % every == 0 || n + 1 == todo.len() {
                let seconds = start.elapsed().as_secs_f64();
                log(&format!(
                    "{}/{} games, {seconds:.0} s, about {:.0} s left",
                    n + 1,
                    todo.len(),
                    seconds / (n + 1) as f64 * (todo.len() - n - 1) as f64
                ));
            }
        }
    });
    if let Some(e) = error {
        return Err(e);
    }
    let games: Vec<Game> = done.into_iter().flatten().collect();
    Ok(report(
        sampler,
        settings,
        &plan,
        &games,
        start.elapsed().as_secs_f64(),
    ))
}

/// Opens `path` for the run: a new file with the settings' header, or an existing one of the
/// same settings whose games go into `done`.
fn open(
    path: &Path,
    settings: &Settings,
    plan: &Plan,
    done: &mut [Option<Game>],
) -> Result<std::fs::File, String> {
    let shown = path.display();
    if path.exists() {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{shown}: {e}"))?;
        let held: File = match toml::from_str(&text) {
            Ok(held) => held,
            Err(e) => {
                // A stop during an append can cut the last game short: drop it (it is played
                // again), as long as the rest reads.
                let cut = text
                    .rfind("\n[[game]]")
                    .ok_or_else(|| format!("{shown}: {e}"))?;
                let held = toml::from_str(&text[..cut]).map_err(|_| format!("{shown}: {e}"))?;
                std::fs::write(path, &text[..cut]).map_err(|e| format!("{shown}: {e}"))?;
                held
            }
        };
        if held.settings != *settings {
            return Err(format!(
                "{shown} holds a run of other settings ({:?}); start a new file",
                held.settings
            ));
        }
        for game in held.game {
            match plan.games.iter().position(|k| *k == game.key) {
                Some(i) => done[i] = Some(game),
                // A game `--leave-out` takes out of a continued run stays in the file, unread.
                None if plan.left_out.contains(&game.key) => {}
                None => return Err(format!("{shown}: a game not of the run, {:?}", game.key)),
            }
        }
        return std::fs::OpenOptions::new()
            .append(true)
            .open(path)
            .map_err(|e| format!("{shown}: {e}"));
    }
    let header = toml::to_string(&Header { settings }).map_err(|e| e.to_string())?;
    let mut f = std::fs::File::create(path).map_err(|e| format!("{shown}: {e}"))?;
    f.write_all(header.as_bytes())
        .and_then(|()| f.flush())
        .map_err(|e| format!("{shown}: {e}"))?;
    Ok(f)
}

/// Appends `game` to `--out`, flushed.
fn append(f: &mut std::fs::File, game: &Game) -> Result<(), String> {
    let text = toml::to_string(&Appended { game: [game] }).map_err(|e| e.to_string())?;
    f.write_all(format!("\n{text}").as_bytes())
        .and_then(|()| f.flush())
        .map_err(|e| format!("--out: {e}"))
}

/// The record of the run's `games` (every game of `plan`, in its order).
fn report(
    sampler: &Sampler,
    settings: Settings,
    plan: &Plan,
    games: &[Game],
    seconds: f64,
) -> CeilingReport {
    let strata = plan.strata.len();
    let mut reports = Vec::new();
    let mut decisions = Vec::new();
    for (o, (name, _)) in plan.opponents.iter().enumerate() {
        let mut pairs = vec![vec![vec![PairResult::default(); strata]; 2]; 2];
        let mut scores: [Vec<(Rating, Score)>; 2] = [Vec::new(), Vec::new()];
        let mut sum = Game::new(GameKey {
            opponent: u32::try_from(o).unwrap_or(u32::MAX),
            stratum: 0,
            env: 0,
            improved: 0,
        });
        for g in games.iter().filter(|g| g.key.opponent as usize == o) {
            // The pair (attacker, defender): the improved seat is player 0, the opponent 1.
            let (a, b) = match g.key.improved {
                0 => (0, 1),
                1 => (1, 0),
                _ => (1, 1),
            };
            let pair = &mut pairs[a][b][g.key.stratum as usize];
            pair.episodes += 1;
            pair.truncated += u32::from(g.truncated);
            pair.reward[0] += g.reward[0];
            pair.reward[1] += g.reward[1];
            let score = match g.winner {
                0 => {
                    pair.wins[0] += 1;
                    Score::Win
                }
                1 => {
                    pair.wins[1] += 1;
                    Score::Loss
                }
                _ => Score::Draw,
            };
            if a != b {
                scores[a].push((Rating::default(), score));
                scores[b].push((Rating::default(), score.reversed()));
            }
            for c in 0..CATEGORIES.len() {
                sum.decisions[c] += g.decisions[c];
                sum.differ[c] += g.differ[c];
                sum.into[c] += g.into[c];
                sum.advantage[c] += g.advantage[c];
            }
            sum.candidates += g.candidates;
            sum.rollouts += g.rollouts;
            sum.refused += g.refused;
            if let Some(v) = &g.valued {
                sum.valued.get_or_insert_with(Valued::default).add(v);
            }
        }
        let ratings = if plan.neutral {
            Vec::new()
        } else {
            scores
                .iter()
                .map(|s| rating::period(Rating::default(), s))
                .collect()
        };
        reports.push(EvalReport {
            set: settings.set.clone(),
            strata: plan.strata.clone(),
            max_steps: settings.max_steps,
            identity: Identity::of(sampler),
            names: vec![seat_name(settings.commands.as_str()), name.clone()],
            defender: plan.neutral.then_some(1),
            pairs,
            ratings,
        });
        decisions.push(sum);
    }
    CeilingReport {
        seat: seat_name(&settings.commands),
        settings,
        reports,
        decisions,
        games: games.len(),
        left_out: plan.left_out.iter().map(|&k| plan.name(k)).collect(),
        seconds,
    }
}

/// A replay's table into `out` (`to_text`): by category the network's decisions, the shares of
/// them that are the fair actor's command and the best, its regret a decision and the fair
/// actor's at the same decisions; the decisions whose best is of the category and the network's
/// regret there; the fair actor's regret by its own command's category (`decisions` by it). Then
/// the means a decision and a game.
fn valued_text(out: &mut String, v: &Valued, decisions: &[u32; 8], games: usize) {
    let _ = writeln!(
        out,
        "{:12} {:>9} {:>6} {:>6} {:>8} {:>8} | {:>9} {:>8} | {:>8}",
        "category", "network", "=fair", "=best", "regret", "fair's", "the best", "missed", "own"
    );
    let per = |sum: f64, n: u32| sum / f64::from(n.max(1));
    for (c, name) in CATEGORIES.iter().enumerate() {
        let n = v.net[c];
        let _ = writeln!(
            out,
            "{name:12} {n:>9} {:>6.3} {:>6.3} {:>+8.4} {:>+8.4} | {:>9} {:>+8.4} | {:>+8.4}",
            per(f64::from(v.same_base[c]), n),
            per(f64::from(v.same_best[c]), n),
            per(v.regret[c], n),
            per(v.base_regret[c], n),
            v.best[c],
            per(v.missed[c], v.best[c]),
            per(v.own_regret[c], decisions[c]),
        );
    }
    let n: u32 = v.net.iter().sum();
    let (regret, base): (f64, f64) = (v.regret.iter().sum(), v.base_regret.iter().sum());
    let _ = writeln!(
        out,
        "a decision: the network's regret {:+.4} (by the halving's means {:+.4}), the fair \
         actor's {:+.4}; a game: {:+.4} and {:+.4}; refused in the confirmation: the best {}, \
         the network's {}, the fair actor's {}",
        per(regret, n),
        per(v.sample_regret, n),
        per(base, n),
        regret / games.max(1) as f64,
        base / games.max(1) as f64,
        v.refused[0],
        v.refused[1],
        v.refused[2],
    );
}

/// The seat's name in the reports: [`IMPROVED`], or the replayed network's ([`Settings::commands`]
/// without the hash).
fn seat_name(commands: &str) -> String {
    match commands.rsplit_once(' ') {
        Some((network, _)) => network.to_owned(),
        None => IMPROVED.to_owned(),
    }
}

impl CeilingReport {
    /// Each opponent's pair report, then the decision table.
    pub fn to_text(&self) -> String {
        let s = &self.settings;
        let mut out = format!(
            "ceiling: {} rollouts a candidate on the {} worlds, set {}, {} episodes a kind, {} \
             games ({:.0} s this part)\n",
            s.budget,
            s.worlds.name(),
            s.set,
            s.episodes,
            self.games,
            self.seconds
        );
        if !s.commands.is_empty() {
            let _ = writeln!(
                out,
                "replayed: {} ({} fresh rollouts confirm a decision)",
                s.commands, s.confirm
            );
        }
        if !self.left_out.is_empty() {
            let _ = writeln!(out, "left out: {}", self.left_out.join(", "));
        }
        for (report, sum) in self.reports.iter().zip(&self.decisions) {
            out.push_str(&report.to_text());
            let decided: u32 = sum.decisions.iter().sum();
            let _ = writeln!(
                out,
                "{} against {}: {decided} decisions, {:.1} candidates and {:.0} rollouts \
                 each, {} refused in a world",
                self.seat,
                report.names[1],
                sum.candidates as f64 / f64::from(decided.max(1)),
                sum.rollouts as f64 / f64::from(decided.max(1)),
                sum.refused
            );
            let _ = writeln!(
                out,
                "{:12} {:>9} {:>7} {:>10} {:>6}",
                "category", "decisions", "differ", "advantage", "into"
            );
            for (c, name) in CATEGORIES.iter().enumerate() {
                let n = f64::from(sum.decisions[c].max(1));
                let _ = writeln!(
                    out,
                    "{name:12} {:>9} {:>7.3} {:>+10.4} {:>6}",
                    sum.decisions[c],
                    f64::from(sum.differ[c]) / n,
                    sum.advantage[c] / n,
                    sum.into[c]
                );
            }
            if let Some(v) = &sum.valued {
                valued_text(&mut out, v, &sum.decisions, self.games);
            }
        }
        out
    }

    /// The record as JSON: the settings, per opponent `h3 ai eval`'s report and the decisions
    /// (with `--commands` their values, `valued`).
    pub fn to_json(&self) -> String {
        let s = &self.settings;
        let list = |v: &[u32]| v.iter().map(u32::to_string).collect::<Vec<_>>().join(", ");
        let floats = |v: &[f64]| {
            v.iter()
                .map(|x| format!("{x}"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let opponents = self
            .reports
            .iter()
            .zip(&self.decisions)
            .map(|(report, sum)| {
                format!(
                    "{{\"opponent\": \"{}\", \"report\": {}, \"decisions\": {{\"categories\": [{}], \
                     \"decisions\": [{}], \"differ\": [{}], \"into\": [{}], \"advantage\": [{}], \
                     \"candidates\": {}, \"rollouts\": {}, \"refused\": {}{}}}}}",
                    report.names[1],
                    report.to_json().trim_end(),
                    CATEGORIES
                        .iter()
                        .map(|c| format!("\"{c}\""))
                        .collect::<Vec<_>>()
                        .join(", "),
                    list(&sum.decisions),
                    list(&sum.differ),
                    list(&sum.into),
                    floats(&sum.advantage),
                    sum.candidates,
                    sum.rollouts,
                    sum.refused,
                    sum.valued.as_ref().map_or(String::new(), |v| format!(
                        ", \"valued\": {{\"net\": [{}], \"same_base\": [{}], \"same_best\": [{}], \
                         \"regret\": [{}], \"base_regret\": [{}], \"best\": [{}], \
                         \"missed\": [{}], \"own_regret\": [{}], \"sample_regret\": {}, \
                         \"refused\": [{}]}}",
                        list(&v.net),
                        list(&v.same_base),
                        list(&v.same_best),
                        floats(&v.regret),
                        floats(&v.base_regret),
                        list(&v.best),
                        floats(&v.missed),
                        floats(&v.own_regret),
                        v.sample_regret,
                        list(&v.refused)
                    ))
                )
            })
            .collect::<Vec<_>>()
            .join(",\n  ");
        let mut more = String::new();
        if !s.commands.is_empty() {
            let _ = write!(
                more,
                ", \"commands\": \"{}\", \"confirm\": {}",
                s.commands, s.confirm
            );
        }
        let mut top = String::new();
        if !s.commands.is_empty() {
            let _ = write!(top, "\n \"seat\": \"{}\",", self.seat);
        }
        if !self.left_out.is_empty() {
            let names: Vec<String> = self.left_out.iter().map(|k| format!("\"{k}\"")).collect();
            let _ = write!(top, "\n \"left_out\": [{}],", names.join(", "));
        }
        format!(
            "{{\n \"tool\": \"h3 ai ceiling\",\n \"settings\": {{\"set\": \"{}\", \"episodes\": {}, \
             \"opponents\": [{}], \"budget\": {}, \"worlds\": \"{}\", \"max_steps\": {}, \
             \"seed\": {}, \"identity\": \"{}\"{more}}},{top}\n \"games\": {},\n \"seconds\": {:.1},\n \
             \"opponents\": [\n  {opponents}\n ]\n}}\n",
            s.set,
            s.episodes,
            s.opponents
                .iter()
                .map(|o| format!("\"{o}\""))
                .collect::<Vec<_>>()
                .join(", "),
            s.budget,
            s.worlds.name(),
            s.max_steps,
            s.seed,
            s.identity,
            self.games,
            self.seconds
        )
    }
}

#[cfg(test)]
mod tests;
