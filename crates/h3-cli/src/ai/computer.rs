//! `h3 ai computer`: the original's computer player (E31; ADR-0030) playing a map's games with
//! every player the computer (`h3_sim::computer::play`), each game to a day limit. Its gated
//! workload (`benches/ai/computer-turns-towns-36.toml`, ADR-0030 point 13) counts only inside
//! `h3_game::computer::choose`, the policy's own cost (the game's steps and the battles' seats
//! outside it).

use std::path::PathBuf;
use std::time::Instant;

use h3_game::GameState;
use h3_sim::computer::{Played, play};

use super::census;

/// What `h3 ai computer` plays.
#[derive(Debug, Clone)]
pub struct ComputerOptions {
    /// The map file.
    pub map: PathBuf,
    /// The games, of the seeds `seed` to `seed + games - 1`.
    pub games: u32,
    /// The day limit: each game is played to the end of this day.
    pub days: u32,
    /// The first game's seed.
    pub seed: u64,
    /// The game's difficulty, 0 (Easy) to 4 (Impossible).
    pub difficulty: u8,
    /// Whether each command is asked twice and each turn held to its bound (the property tests'
    /// checks; off for the benchmark, whose count it would double).
    pub checked: bool,
}

/// The games' sums.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComputerReport {
    /// The games' counts summed, the longest turn the longest of all; `day` and `hash` the last
    /// game's.
    pub played: Played,
    /// The wall-clock seconds of the games.
    pub secs: f64,
}

/// Plays the games of `options`.
///
/// # Errors
///
/// A map that cannot be read or imported, rules that do not compile, a game that stops.
pub fn run(options: &ComputerOptions) -> Result<ComputerReport, String> {
    let rules = census::original_rules()?;
    let bytes =
        std::fs::read(&options.map).map_err(|e| format!("{}: {e}", options.map.display()))?;
    let raw = h3_map::read(&bytes, &h3_map::Limits::default()).map_err(|e| format!("read: {e}"))?;
    let mut setup = h3_map::import(&raw, &rules).map_err(|e| format!("import: {e}"))?;
    setup.difficulty = options.difficulty;
    let mut report = ComputerReport::default();
    let start = Instant::now();
    for game in 0..options.games {
        let seed = options.seed + u64::from(game);
        let state = GameState::new(&rules, &setup, seed).map_err(|e| format!("build: {e}"))?;
        let played = play(&rules, state, seed, options.days, options.checked)
            .map_err(|e| format!("game {seed}: {e:?}"))?;
        let sum = &mut report.played;
        sum.turns += played.turns;
        sum.commands += played.commands;
        sum.answers += played.answers;
        sum.battle_commands += played.battle_commands;
        for (total, moves) in sum.garrison_moves.iter_mut().zip(played.garrison_moves) {
            *total += moves;
        }
        sum.longest_turn = sum.longest_turn.max(played.longest_turn);
        sum.day = played.day;
        sum.hash = played.hash;
    }
    report.secs = start.elapsed().as_secs_f64();
    Ok(report)
}
