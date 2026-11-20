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
//! it, skipping the games it holds.

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

/// Scratch buffers of a worker.
#[derive(Default)]
struct Scratch {
    legal: LegalCommands,
    commands: Vec<Command>,
    sums: Vec<f64>,
    active: Vec<usize>,
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
    let Scratch {
        legal,
        commands,
        sums,
        active,
    } = scratch;
    candidates(env, side, legal, commands);
    if !commands.contains(&base) {
        commands.push(base);
    }
    let n = commands.len();
    let mut decision = Decision {
        command: base,
        base,
        candidates: u32::try_from(n).unwrap_or(u32::MAX),
        rollouts: 0,
        advantage: 0.0,
        refused: 0,
    };
    if settings.budget == 0 || n < 2 {
        return Ok(decision);
    }
    let at_base = commands.iter().position(|&c| c == base).unwrap_or(0);
    sums.clear();
    sums.resize(n, 0.0);
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
                        decision.refused += 1;
                        -1.0
                    }
                };
            }
            decision.rollouts += u32::try_from(active.len()).unwrap_or(u32::MAX);
        }
        if round + 1 < rounds {
            // Every candidate still in has the same rollouts: the sums rank as the means. The
            // better half by sum (ties to the earlier candidate), then the fair actor's command.
            active.sort_by(|&a, &b| sums[b].total_cmp(&sums[a]).then(a.cmp(&b)));
            active.truncate(active.len().div_ceil(2));
            if !active.contains(&at_base) {
                active.push(at_base);
            }
            active.sort_unstable();
        }
    }
    let mut best = at_base;
    for &c in active.iter() {
        if sums[c] > sums[best] {
            best = c;
        }
    }
    decision.command = commands[best];
    decision.advantage = (sums[best] - sums[at_base]) / f64::from(r);
    Ok(decision)
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
    Engine(Policy),
}

/// The run's strata, opponents and games, in order.
#[derive(Debug)]
struct Plan {
    strata: Vec<Stratum>,
    opponents: Vec<(String, Policy)>,
    neutral: bool,
    games: Vec<GameKey>,
}

impl Plan {
    fn of(settings: &Settings) -> Result<Self, String> {
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
        })
    }
}

/// Plays the game `key` (its episode's seed `h3 ai eval`'s).
fn play(
    key: GameKey,
    plan: &Plan,
    sources: &[Scenario],
    sampler: &Sampler,
    settings: &Settings,
) -> Result<Game, String> {
    let stratum = &plan.strata[key.stratum as usize];
    let seed = episode_seed(stratum.seed, key.env, 0);
    let opponent = plan.opponents[key.opponent as usize].1;
    let players = match key.improved {
        0 => [Player::Improved, Player::Engine(opponent)],
        1 => [Player::Engine(opponent), Player::Improved],
        _ => [Player::Engine(Policy::System), Player::Engine(opponent)],
    };
    let mut env = sources[key.stratum as usize]
        .episode(seed, settings.max_steps)
        .map_err(|e| format!("episode {seed:#x}: {e:?}"))?
        .env;
    let mut rngs = [0, 1].map(|s| Rng::for_stream(seed, StreamId::seat(s)));
    let mut scratch = Scratch::default();
    let mut game = Game::new(key);
    while let Some(side) = env.to_move() {
        let i = side.index();
        let command = match players[i] {
            Player::Engine(policy) => policy.choose(&env, side, &mut rngs[i], &mut scratch.legal),
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
    Ok(game)
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
    let plan = Plan::of(&settings)?;
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
            scope.spawn(move || {
                while !failed.load(Ordering::Relaxed) {
                    let Some(&i) = todo.get(next.fetch_add(1, Ordering::Relaxed)) else {
                        break;
                    };
                    let game = play(plan.games[i], plan, sources, sampler, settings);
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
        let held: File = toml::from_str(&text).map_err(|e| format!("{shown}: {e}"))?;
        if held.settings != *settings {
            return Err(format!(
                "{shown} holds a run of other settings ({:?}); start a new file",
                held.settings
            ));
        }
        for game in held.game {
            let i = plan
                .games
                .iter()
                .position(|k| *k == game.key)
                .ok_or_else(|| format!("{shown}: a game not of the run, {:?}", game.key))?;
            done[i] = Some(game);
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
            names: vec![IMPROVED.to_owned(), name.clone()],
            defender: plan.neutral.then_some(1),
            pairs,
            ratings,
        });
        decisions.push(sum);
    }
    CeilingReport {
        settings,
        reports,
        decisions,
        games: games.len(),
        seconds,
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
        for (report, sum) in self.reports.iter().zip(&self.decisions) {
            out.push_str(&report.to_text());
            let decided: u32 = sum.decisions.iter().sum();
            let _ = writeln!(
                out,
                "{IMPROVED} against {}: {decided} decisions, {:.1} candidates and {:.0} rollouts \
                 each, {} refused in a world",
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
        }
        out
    }

    /// The record as JSON: the settings, per opponent `h3 ai eval`'s report and the decisions.
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
                     \"candidates\": {}, \"rollouts\": {}, \"refused\": {}}}}}",
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
                    sum.refused
                )
            })
            .collect::<Vec<_>>()
            .join(",\n  ");
        format!(
            "{{\n \"tool\": \"h3 ai ceiling\",\n \"settings\": {{\"set\": \"{}\", \"episodes\": {}, \
             \"opponents\": [{}], \"budget\": {}, \"worlds\": \"{}\", \"max_steps\": {}, \
             \"seed\": {}, \"identity\": \"{}\"}},\n \"games\": {},\n \"seconds\": {:.1},\n \
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
