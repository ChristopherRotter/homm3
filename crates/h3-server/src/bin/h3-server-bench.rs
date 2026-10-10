//! `h3-server-bench [--scenario <file>] [--packs-dir packs] [--games N] [--seed N]`: the session
//! core's workload (`h3_server::bench`; `benches/server/`, measured by `cargo xtask bench-check`).
//! Scripted clients play hosted battles frame by frame through the server's session core.
// An adapter: it may use the clock (the wall time it prints; the gate counts instructions).
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use h3_server::bench;

const USAGE: &str =
    "usage: h3-server-bench [--scenario <file>] [--packs-dir packs] [--games N] [--seed N]";

/// The longest game, in commands (as `h3 bench`'s rollouts).
const MAX_COMMANDS: u32 = 5_000;

fn run(args: &[String]) -> Result<(), String> {
    let mut scenario = PathBuf::from("packs/synthetic/scenarios/full.toml");
    let mut packs_dir = PathBuf::from("packs");
    let mut games: u64 = 20;
    let mut seed: u64 = 1;
    let mut iter = args.iter();
    while let Some(flag) = iter.next() {
        let value = iter
            .next()
            .ok_or_else(|| format!("{flag} needs a value\n{USAGE}"))?;
        let number = || value.parse().map_err(|e| format!("{flag} {value}: {e}"));
        match flag.as_str() {
            "--scenario" => scenario = PathBuf::from(value),
            "--packs-dir" => packs_dir = PathBuf::from(value),
            "--games" => games = number()?,
            "--seed" => seed = number()?,
            _ => return Err(format!("unknown option {flag}\n{USAGE}")),
        }
    }
    let (_, ruleset, setup) =
        h3_cli::load_scenario(&scenario, &packs_dir, seed).map_err(|e| e.to_string())?;
    let start = Instant::now();
    let totals = bench::run(&ruleset, &setup, seed, games, MAX_COMMANDS).map_err(|e| e.0)?;
    let secs = start.elapsed().as_secs_f64();
    println!(
        "games: {} ({} ended), frames in: {}, frames out: {}, bytes out: {}, time: {secs:.3}s",
        totals.games, totals.ended, totals.frames_in, totals.frames_out, totals.bytes_out
    );
    println!(
        "frames in per second (decode, validate, apply, project, encode): {:.0}",
        totals.frames_in as f64 / secs
    );
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("h3-server-bench: {e}");
            ExitCode::FAILURE
        }
    }
}
