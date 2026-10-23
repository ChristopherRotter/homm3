//! `h3 adventure bench` (E08 T9b, decision 11): the adventure's benchmark workloads, timed here
//! and counted in instructions by `cargo xtask bench-check` (`benches/adventure/`), each inside
//! one function that is never inlined:
//!
//! * `play`: random legal play with the env's random agent, inside
//!   `h3_sim::adventure::random_rollout` (choosing, the reachable-tiles query, `apply`).
//! * `reach`: the reachable-tiles query of every hero of the acting player at each step of
//!   random play (the play around it is not counted): the walking heroes' inside
//!   [`reach_queries`], the flying and water-walking heroes' inside [`lifted_queries`] (E08 T42:
//!   two `bench-check` workloads of the same games, `reach-xl-144` and `reach-lifted-xl-144`).
//! * `fork`: forks of the env after `days` days of random play, inside [`fork_workload`]; with
//!   `--battle 1` (E08 T19b1) the heroes seek each other instead and the play stops at the step
//!   that opens a battle, so the forks carry an open battle.
//!
//! Two variants measure what an agent adds (wall time only, for the docs): `--observe 1`, the
//! acting player's observation before each step; `--masks 1`, the entry mask and the chosen
//! entry's tile mask before each step, as an agent samples the factorized action.

use std::hint::black_box;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use h3_adventure::Pos;
use h3_battle::LegalCommands;
use h3_contracts::{HeroId, PlayerId};
use h3_game::{Actor, legal_moves, movement};
use h3_kernel::{Rng, StreamId};
use h3_sim::action::{ACTION_COUNT, SPELL_COUNT, TARGET_COUNT};
use h3_sim::adventure::action::{self, hero_in_slot, tile_index};
use h3_sim::adventure::rollout::choose;
use h3_sim::adventure::{Action, ENTRY_COUNT, HERO_SLOTS, Phase, Seat};
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
    /// `fork`: the play stops when a battle opens (within the day limit).
    pub battle: bool,
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
    /// `fork`: the games whose forks carried an open battle.
    pub battles: u64,
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
        battles: 0,
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
                let steps = if options.battle {
                    until_battle(&mut env)
                } else {
                    u64::from(random_rollout(&mut env, g, 0).steps)
                };
                let open = env.to_move().any(|p| env.phase(p) == Phase::Battle);
                result.battles += u64::from(open);
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

/// Play until a battle opens or the episode ends (`fork` with `battle`): the steps played. Random
/// play seldom brings two players' heroes together on a large map, so here the heroes seek each
/// other ([`seek`]).
fn until_battle(env: &mut AdventureEnv) -> u64 {
    let mut steps = 0;
    loop {
        let Some(seat) = env.to_move().next() else {
            break;
        };
        // A battle (the neutral seat acts in battles only) ends the walk.
        let (Seat::Player(player), Phase::Adventure) = (seat, env.phase(seat)) else {
            break;
        };
        // A legal action is always accepted (h3-sim's tests); a refusal would repeat.
        if env.step(player, seek(env, player)).is_err() {
            break;
        }
        steps += 1;
    }
    steps
}

/// The acting player's step toward another player's hero: of its heroes' legal moves the one that
/// ends nearest (in steps, on the same level) to another player's hero, the first slot's and
/// tile's on a tie, the hero's tile itself (a battle) nearest of all; `EndTurn` when no move gets
/// nearer than its hero stands, `Decline` with an open decision.
fn seek(env: &AdventureEnv, player: PlayerId) -> Action {
    let state = env.state();
    if state.decision(player).is_some() {
        return Action::decline();
    }
    let others: Vec<Pos> = state
        .adventure()
        .heroes()
        .iter()
        .filter(|hero| hero.owner != player)
        .map(|hero| hero.pos)
        .collect();
    let distance = |pos: Pos| {
        others
            .iter()
            .filter(|other| other.level == pos.level)
            .map(|other| other.x.abs_diff(pos.x).max(other.y.abs_diff(pos.y)))
            .min()
    };
    let mut best: Option<(u16, Action)> = None;
    for slot in 0..HERO_SLOTS {
        let (Some(hero), Some(reach)) = (
            hero_in_slot(state, player, slot).and_then(|id| state.adventure().hero(id)),
            action::moves(state, env.rules(), player, slot),
        ) else {
            continue;
        };
        let Some(here) = distance(hero.pos) else {
            continue;
        };
        for (pos, _) in reach.tiles() {
            let (Some(d), Some(tile)) = (distance(pos), tile_index(state, pos)) else {
                continue;
            };
            if d < here && best.is_none_or(|(b, _)| d < b) {
                best = Some((d, Action::move_to(slot, tile)));
            }
        }
    }
    best.map_or_else(Action::end_turn, |(_, action)| action)
}

/// Random play with every hero's reachable-tiles query at each step (the counted and timed part):
/// the walking heroes' ([`reach_queries`]), then the flying and water-walking heroes'
/// ([`lifted_queries`], E08 T42): queries, steps, seconds.
fn reach_rollout(env: &mut AdventureEnv, agent_seed: u64) -> (u64, u64, f64) {
    let mut rng = Rng::for_stream(agent_seed, StreamId::AGENT);
    let (mut queries, mut steps, mut seconds) = (0, 0, 0.0);
    loop {
        let Some(seat) = env.to_move().next() else {
            break;
        };
        if let Some(player) = seat.player() {
            // Which heroes fly or walk on water, outside the counted and timed part.
            let (lifted, walkers) = heroes_by_lift(env, player);
            let start = Instant::now();
            queries += reach_queries(env, player, &walkers) + lifted_queries(env, player, &lifted);
            seconds += start.elapsed().as_secs_f64();
        }
        let action = choose(env, seat, &mut rng);
        // A legal action is always accepted (h3-sim's tests).
        let _ = env.step(seat, action);
        steps += 1;
    }
    (queries, steps, seconds)
}

/// `player`'s heroes that fly or walk on water (`h3_heroes`' levels, HER-ADV-005), then the
/// others, in id order.
pub(crate) fn heroes_by_lift(env: &AdventureEnv, player: PlayerId) -> (Vec<HeroId>, Vec<HeroId>) {
    let (state, rules) = (env.state(), env.rules());
    let own = state
        .adventure()
        .heroes()
        .iter()
        .filter(|h| h.owner == player);
    own.map(|hero| hero.id).partition(|&id| {
        state.hero(id).is_some_and(|details| {
            let levels = movement(details, rules.core());
            levels.flying.is_some() || levels.water_walking.is_some()
        })
    })
}

/// The reachable-tiles query of each of `heroes`, `player`'s walking heroes (neither flying nor
/// water-walking, E08 T42), as a client asks for its heroes' moves: the number of queries. The
/// `reach` workload counts the instructions inside this function: the walker's search.
#[inline(never)]
pub fn reach_queries(env: &AdventureEnv, player: PlayerId, heroes: &[HeroId]) -> u64 {
    queries(env, player, heroes)
}

/// The reachable-tiles query of each of `heroes`, `player`'s flying or water-walking heroes (E08
/// T42, the reach's two kinds of node): the number of queries. The `reach-lifted` workload counts
/// the instructions inside this function.
#[inline(never)]
pub fn lifted_queries(env: &AdventureEnv, player: PlayerId, heroes: &[HeroId]) -> u64 {
    queries(env, player, heroes)
}

/// The reachable-tiles query of each of `heroes` for `player`: the number of queries.
fn queries(env: &AdventureEnv, player: PlayerId, heroes: &[HeroId]) -> u64 {
    let (state, rules) = (env.state(), env.rules());
    for &hero in heroes {
        let reach = legal_moves(state, rules, Actor::Player(player), hero);
        black_box(reach.as_ref().map(|r| r.tiles().count()));
    }
    u64::try_from(heroes.len()).unwrap_or(u64::MAX)
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

/// Random play with what an agent reads before each step: the observation and the masks (in a
/// battle the battle's heads, E08 T19b1).
fn agent_rollout(env: &mut AdventureEnv, agent_seed: u64, observe: bool, masks: bool) -> u64 {
    let mut rng = Rng::for_stream(agent_seed, StreamId::AGENT);
    let mut entries = [false; ENTRY_COUNT];
    let mut tiles = Vec::new();
    let mut legal = LegalCommands::new();
    let mut battle_entries = [false; ACTION_COUNT];
    let mut spells = [false; SPELL_COUNT];
    let mut targets = [false; TARGET_COUNT];
    let mut steps = 0;
    loop {
        let Some(seat) = env.to_move().next() else {
            break;
        };
        if observe {
            match seat.player() {
                Some(player) => {
                    black_box(env.observation(player));
                }
                None => {
                    black_box(env.neutral_observation());
                }
            }
        }
        let action = choose(env, seat, &mut rng);
        match action {
            Action::Adventure(action) if masks => {
                env.legal_mask(seat, &mut entries);
                env.tile_mask(seat, action.entry, &mut tiles);
                black_box((&entries, &tiles));
            }
            Action::Battle(action) if masks => {
                let (entry, spell) = (action.entry as usize, action.spell);
                env.battle_mask(seat, &mut legal, &mut battle_entries);
                env.spell_mask(seat, entry, &mut legal, &mut spells);
                env.target_mask(seat, (entry, spell), &mut legal, &mut targets);
                black_box((&battle_entries, &spells, &targets));
                let prefix = (entry, spell, action.target);
                env.second_mask(seat, prefix, &mut legal, &mut targets);
                black_box(&targets);
            }
            _ => {}
        }
        let _ = env.step(seat, action);
        steps += 1;
    }
    steps
}
