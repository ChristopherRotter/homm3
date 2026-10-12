//! `h3 adventure bench` (E08 T9b, decision 11): the adventure's benchmark workloads, timed here
//! and counted in instructions by `cargo xtask bench-check` (`benches/adventure/`), each inside
//! one function that is never inlined:
//!
//! * `play`: random legal play with the env's random agent, inside
//!   `h3_sim::adventure::random_rollout` (choosing, the reachable-tiles query, `apply`).
//! * `reach`: the reachable-tiles query of every hero of the acting player at each step of
//!   random play, inside [`reach_queries`] (the play around it is not counted).
//! * `fork`: forks of the env after `days` days of random play, inside [`fork_workload`].
//!
//! Two variants measure what an agent adds (wall time only, for the docs): `--observe 1`, the
//! acting player's observation before each step; `--masks 1`, the entry mask and the chosen
//! entry's tile mask before each step, as an agent samples the factorized action.

use std::hint::black_box;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use h3_contracts::PlayerId;
use h3_game::{Actor, legal_moves};
use h3_kernel::{Rng, StreamId};
use h3_sim::adventure::ENTRY_COUNT;
use h3_sim::adventure::rollout::choose;
use h3_sim::adventure::{AdventureEnv, random_rollout};

use super::{DEFAULT_PACKS, Result, load_map, load_rules};

/// Which workload [`run`] plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Workload {
    /// Random legal play.
    Play,
    /// The reachable-tiles queries.
    Reach,
    /// Forks.
    Fork,
}

/// What [`run`] plays.
#[derive(Debug, Clone)]
pub struct BenchOptions {
    /// The map's path, relative to the root.
    pub map: String,
    /// The profile key.
    pub profile: String,
    /// The games' difficulty (setup input, checked by the game's build).
    pub difficulty: u8,
    /// The workload.
    pub workload: Workload,
    /// Games (episodes); game `g` uses seed `seed + g` and agent seed `g`.
    pub games: u64,
    /// The day limit of each game (`fork`: the days played before forking).
    pub days: u32,
    /// The first game's seed.
    pub seed: u64,
    /// Forks per game (`fork`).
    pub forks: u32,
    /// `play`: the acting player's observation before each step.
    pub observe: bool,
    /// `play`: the entry and tile masks before each step.
    pub masks: bool,
}

/// What [`run`] measured.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BenchResult {
    /// Counted operations: steps (`play`), queries (`reach`) or forks (`fork`).
    pub operations: u64,
    /// Steps played in all.
    pub steps: u64,
    /// Seconds spent in the counted part.
    pub seconds: f64,
}

/// Runs a workload on a map below `root` with the packs of `packs_dir`.
///
/// # Errors
///
/// The rules', the map's or the setup's error.
pub fn run(packs_dir: &Path, root: &Path, options: &BenchOptions) -> Result<BenchResult> {
    let ids: Vec<String> = DEFAULT_PACKS.iter().map(|&s| s.to_owned()).collect();
    let rules = load_rules(packs_dir, &ids, &options.profile)?;
    let (_, mut setup) = load_map(root, &options.map, &rules)?;
    setup.difficulty = options.difficulty;
    let mut env = AdventureEnv::new(rules, Arc::new(setup), options.seed, options.days)
        .map_err(super::AdventureCliError::Setup)?;
    let mut result = BenchResult {
        operations: 0,
        steps: 0,
        seconds: 0.0,
    };
    for g in 0..options.games {
        // The setup built the first game: a reset cannot fail.
        let _ = env.reset(options.seed.wrapping_add(g));
        let (operations, steps, seconds) = match options.workload {
            Workload::Play if options.observe || options.masks => {
                let start = Instant::now();
                let steps = agent_rollout(&mut env, g, options.observe, options.masks);
                (steps, steps, start.elapsed().as_secs_f64())
            }
            Workload::Play => {
                let start = Instant::now();
                let steps = u64::from(random_rollout(&mut env, g, 0).steps);
                (steps, steps, start.elapsed().as_secs_f64())
            }
            Workload::Reach => reach_rollout(&mut env, g),
            Workload::Fork => {
                let steps = u64::from(random_rollout(&mut env, g, 0).steps);
                let start = Instant::now();
                black_box(fork_workload(&env, options.forks));
                (
                    u64::from(options.forks),
                    steps,
                    start.elapsed().as_secs_f64(),
                )
            }
        };
        result.operations += operations;
        result.steps += steps;
        result.seconds += seconds;
    }
    Ok(result)
}

/// Random play with every hero's reachable-tiles query at each step ([`reach_queries`], the
/// counted and timed part): queries, steps, seconds.
fn reach_rollout(env: &mut AdventureEnv, agent_seed: u64) -> (u64, u64, f64) {
    let mut rng = Rng::for_stream(agent_seed, StreamId::AGENT);
    let (mut queries, mut steps, mut seconds) = (0, 0, 0.0);
    loop {
        let Some(player) = env.to_move().next() else {
            break;
        };
        let start = Instant::now();
        queries += reach_queries(env, player);
        seconds += start.elapsed().as_secs_f64();
        let action = choose(env, player, &mut rng);
        // A legal action is always accepted (h3-sim's tests).
        let _ = env.step(player, action);
        steps += 1;
    }
    (queries, steps, seconds)
}

/// The reachable-tiles query of every hero `player` owns, as a client asks for its heroes' moves:
/// the number of queries. The `reach` workload counts the instructions inside this function.
#[inline(never)]
pub fn reach_queries(env: &AdventureEnv, player: PlayerId) -> u64 {
    let state = env.state();
    let mut queries = 0;
    for hero in state.adventure().heroes() {
        if hero.owner == player {
            let reach = legal_moves(state, env.rules(), Actor::Player(player), hero.id);
            black_box(reach.as_ref().map(|r| r.tiles().count()));
            queries += 1;
        }
    }
    queries
}

/// `forks` forks of `env`, each dropped at once: the `fork` workload counts the instructions
/// inside this function. Returns the last fork's hash so the forks are not optimized away.
#[inline(never)]
pub fn fork_workload(env: &AdventureEnv, forks: u32) -> u64 {
    let mut last = 0;
    for _ in 0..forks {
        let fork = black_box(env.fork());
        last = fork.steps().into();
    }
    last
}

/// Random play with what an agent reads before each step: the observation and the masks.
fn agent_rollout(env: &mut AdventureEnv, agent_seed: u64, observe: bool, masks: bool) -> u64 {
    let mut rng = Rng::for_stream(agent_seed, StreamId::AGENT);
    let mut entries = [false; ENTRY_COUNT];
    let mut tiles = Vec::new();
    let mut steps = 0;
    loop {
        let Some(player) = env.to_move().next() else {
            break;
        };
        if observe {
            black_box(env.observation(player));
        }
        let action = choose(env, player, &mut rng);
        if masks {
            env.legal_mask(player, &mut entries);
            env.tile_mask(player, action.entry, &mut tiles);
            black_box((&entries, &tiles));
        }
        let _ = env.step(player, action);
        steps += 1;
    }
    steps
}
