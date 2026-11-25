//! `h3-sim`'s check against the original data (`cargo xtask verify --with-original-data`; E31
//! T2a, ADR-0030 point 3): the original's computer player (`h3_game::computer::choose`, hosted by
//! `h3_sim::computer::play` with every player the computer) plays every shipped map in
//! `<install>/Maps` that imports, at each of the five difficulties, to the end of day 7, and five
//! of them (a small, a medium, a large and two extra-large maps of two to six players) to the end
//! of day 28: every command legal, each turn within its bound, each command the same on copies of
//! its state, memory and stream. Prints the games' counts; fails on a map that does not import, a
//! named map missing, or a game that stops.

use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

use h3_game::{GameRuleset, GameState};
use h3_sim::computer::{Played, play};

/// The maps played for four weeks.
const MONTH: [&str; 5] = [
    "Arrogance.h3m",
    "Dead and Buried.h3m",
    "Crimson and Clover.h3m",
    "The Gauntlet.h3m",
    "War of the Mighty.h3m",
];

/// The original packs (RoE to SoD) under the faithful profile, as the maps are imported.
fn rules() -> Result<GameRuleset, String> {
    let profile = "base:faithful";
    let (core, part) = h3_pack_adventure::compile(&h3_pack_adventure::ORIGINAL_PACKS, profile)
        .map_err(|e| format!("adventure packs: {e}"))?;
    let battle = h3_pack_battle::compile(
        &[
            h3_pack_battle::BASE_PACK,
            h3_pack_battle::AB_PACK,
            h3_pack_battle::SOD_PACK,
        ],
        profile,
    )
    .map_err(|e| format!("battle packs: {e}"))?;
    let towns = h3_pack_towns::compile(&h3_pack_towns::ORIGINAL_PACKS, profile)
        .map_err(|e| format!("town packs: {e}"))?;
    GameRuleset::new(
        Arc::new(core),
        Arc::new(part),
        Arc::new(battle),
        Arc::new(towns),
    )
    .map_err(|e| format!("rules: {e}"))
}

/// The game of the map in `bytes` at `difficulty` played to the end of day `days`.
fn played(rules: &GameRuleset, bytes: &[u8], difficulty: u8, days: u32) -> Result<Played, String> {
    let raw = h3_map::read(bytes, &h3_map::Limits::default()).map_err(|e| format!("read: {e}"))?;
    let mut setup = h3_map::import(&raw, rules).map_err(|e| format!("import: {e}"))?;
    setup.difficulty = difficulty;
    let seed = u64::from(difficulty) + 1;
    let state = GameState::new(rules, &setup, seed).map_err(|e| format!("build: {e}"))?;
    play(rules, state, seed, days, true).map_err(|e| format!("play: {e:?}"))
}

fn main() -> ExitCode {
    let Some(install) = std::env::args().nth(1) else {
        eprintln!("usage: original_data <installation copy>");
        return ExitCode::FAILURE;
    };
    let dir = Path::new(&install).join("Maps");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        println!("h3-sim: skipped ({} is not there)", dir.display());
        return ExitCode::SUCCESS;
    };
    let mut files: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("h3m") || e.eq_ignore_ascii_case("tut"))
        })
        .collect();
    files.sort();
    let rules = match rules() {
        Ok(rules) => rules,
        Err(e) => {
            eprintln!("h3-sim: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut failures = Vec::new();
    let (mut games, mut total) = (0_u32, Played::default());
    let mut month = 0;
    for path in &files {
        let name = path
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        let Ok(bytes) = std::fs::read(path) else {
            failures.push(format!("{name}: unreadable"));
            continue;
        };
        let days = if MONTH.contains(&name.as_str()) {
            month += 1;
            28
        } else {
            7
        };
        for difficulty in 0..5 {
            match played(&rules, &bytes, difficulty, days) {
                Ok(game) => {
                    games += 1;
                    total.turns += game.turns;
                    total.commands += game.commands;
                    total.answers += game.answers;
                    total.battle_commands += game.battle_commands;
                    for (sum, moves) in total.garrison_moves.iter_mut().zip(game.garrison_moves) {
                        *sum += moves;
                    }
                    total.longest_turn = total.longest_turn.max(game.longest_turn);
                }
                Err(e) => failures.push(format!("{name} at difficulty {difficulty}: {e}")),
            }
        }
    }
    println!(
        "h3-sim: the computer played {games} games of {} maps ({month} of {} to day 28, the rest to \
         day 7; difficulties 0 to 4): {} turns, {} commands ({} answers, {} battle commands; the \
         garrison moves Enter {}, Leave {}, Swap {}), the longest turn {} commands",
        files.len(),
        MONTH.len(),
        total.turns,
        total.commands,
        total.answers,
        total.battle_commands,
        total.garrison_moves[0],
        total.garrison_moves[1],
        total.garrison_moves[2],
        total.longest_turn,
    );
    if month != MONTH.len() {
        failures.push(format!("{} of the {} month maps found", month, MONTH.len()));
    }
    if failures.is_empty() {
        ExitCode::SUCCESS
    } else {
        for failure in &failures {
            eprintln!("h3-sim: {failure}");
        }
        ExitCode::FAILURE
    }
}
