//! `h3` — headless runner and replay tool.
//!
//! ```text
//! h3 run --scenario <file> [--packs-dir packs] [--seed N] [--agent-seed N] [--max-steps N]
//!        [--checkpoint-every N] [--record <out.toml>]
//! h3 verify [--packs-dir packs] <replay.toml>...
//! h3 bench [--scenario <file>] [--packs-dir packs] [--games N] [--seed N] [--observe 0|1]
//! ```
// The CLI is an adapter: it may use the clock (bench timing) and the environment.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use std::hint::black_box;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Instant;

use h3_battle::LegalCommands;
use h3_cli::{CliError, RunOptions, load_scenario, run_random, verify_file};
use h3_kernel::{Rng, StreamId};
use h3_sim::rollout::{pick, step_command};
use h3_sim::{BattleEnv, OBSERVATION_LEN, random_rollout};

const USAGE: &str = "usage:
  h3 run --scenario <file> [--packs-dir packs] [--seed N] [--agent-seed N] [--max-steps N]
         [--checkpoint-every N] [--record <out.toml>]
  h3 verify [--packs-dir packs] <replay.toml>...
  h3 bench [--scenario <file>] [--packs-dir packs] [--games N] [--seed N] [--observe 0|1]";

struct Args {
    positional: Vec<String>,
    options: Vec<(String, String)>,
}

impl Args {
    fn parse(raw: &[String]) -> Result<Self, String> {
        let mut positional = Vec::new();
        let mut options = Vec::new();
        let mut it = raw.iter();
        while let Some(a) = it.next() {
            if let Some(name) = a.strip_prefix("--") {
                let value = it.next().ok_or_else(|| format!("--{name} needs a value"))?;
                options.push((name.to_owned(), value.clone()));
            } else {
                positional.push(a.clone());
            }
        }
        Ok(Self {
            positional,
            options,
        })
    }

    fn get(&self, name: &str) -> Option<&str> {
        self.options
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    fn num<T: std::str::FromStr>(&self, name: &str, default: T) -> Result<T, String> {
        match self.get(name) {
            Some(v) => v
                .parse()
                .map_err(|_| format!("--{name}: `{v}` is not a valid number")),
            None => Ok(default),
        }
    }

    fn check_known(&self, known: &[&str]) -> Result<(), String> {
        match self
            .options
            .iter()
            .find(|(n, _)| !known.contains(&n.as_str()))
        {
            Some((n, _)) => Err(format!("unknown option --{n}")),
            None => Ok(()),
        }
    }
}

fn run(args: &Args) -> Result<(), String> {
    args.check_known(&[
        "scenario",
        "packs-dir",
        "seed",
        "agent-seed",
        "max-steps",
        "checkpoint-every",
        "record",
    ])?;
    let scenario = PathBuf::from(args.get("scenario").ok_or("--scenario is required")?);
    let packs_dir = PathBuf::from(args.get("packs-dir").unwrap_or("packs"));
    let options = RunOptions {
        seed: args.num("seed", 1)?,
        agent_seed: args.num("agent-seed", 1)?,
        max_steps: args.num("max-steps", 1000)?,
        checkpoint_every: args.num("checkpoint-every", 25)?,
    };
    let (replay, text) = run_random(&scenario, &packs_dir, &options).map_err(|e| e.to_string())?;
    let winner = match replay.result {
        None => "none (step limit)",
        Some(r) => h3_cli::result_text(Some(r)),
    };
    println!("commands: {}", replay.commands.len());
    println!("winner:   {winner}");
    println!("hash:     {:#018x}", replay.final_hash);
    if let Some(out) = args.get("record") {
        std::fs::write(out, text).map_err(|e| format!("{out}: {e}"))?;
        println!("replay written to {out}");
    }
    Ok(())
}

fn verify(args: &Args) -> Result<(), String> {
    args.check_known(&["packs-dir"])?;
    if args.positional.is_empty() {
        return Err("verify needs at least one replay file".to_owned());
    }
    let packs_dir = PathBuf::from(args.get("packs-dir").unwrap_or("packs"));
    let mut failed = 0;
    for file in &args.positional {
        match verify_file(&PathBuf::from(file), &packs_dir) {
            Ok(v) => println!(
                "ok   {file}: {} ({} commands, final hash {:#018x})",
                v.scenario, v.commands, v.final_hash
            ),
            Err(e) => {
                failed += 1;
                println!("FAIL {file}: {e}");
            }
        }
    }
    if failed == 0 {
        Ok(())
    } else {
        Err(format!("{failed} replay(s) failed to verify"))
    }
}

fn bench(args: &Args) -> Result<(), String> {
    args.check_known(&["scenario", "packs-dir", "games", "seed", "observe"])?;
    let packs_dir = PathBuf::from(args.get("packs-dir").unwrap_or("packs"));
    let scenario = PathBuf::from(
        args.get("scenario")
            .unwrap_or("packs/synthetic/scenarios/full.toml"),
    );
    let games: u64 = args.num("games", 2000)?;
    let seed: u64 = args.num("seed", 1)?;
    let observe = args.num("observe", 0u8)? != 0;
    let (_, ruleset, setup) =
        load_scenario(&scenario, &packs_dir, seed).map_err(|e: CliError| e.to_string())?;
    let mut env =
        BattleEnv::new(Arc::new(ruleset), Arc::new(setup), seed, 0).map_err(|e| e.to_string())?;
    let mut out = vec![0.0f32; OBSERVATION_LEN];
    let start = Instant::now();
    let mut steps: u64 = 0;
    for g in 0..games {
        env.reset(seed.wrapping_add(g)).map_err(|e| e.to_string())?;
        steps += u64::from(if observe {
            observed_rollout(&mut env, g, 5_000, &mut out)
        } else {
            random_rollout(&mut env, g, 5_000).steps
        });
    }
    let secs = start.elapsed().as_secs_f64();
    println!("games: {games}, steps: {steps}, time: {secs:.3}s");
    println!(
        "size of BattleState: {} bytes, BattleEnv: {} bytes",
        size_of::<h3_battle::BattleState>(),
        size_of::<BattleEnv>()
    );
    println!(
        "steps/s (random legal agent incl. legal-command generation{}): {:.0}",
        if observe {
            ", observation encoded before each step"
        } else {
            ""
        },
        steps as f64 / secs
    );
    Ok(())
}

/// [`random_rollout`] with the flat observation of the side to act encoded before each step, as
/// an agent reads it (E04b T9b, the encoding's cost): the same agent stream, so the same steps.
fn observed_rollout(env: &mut BattleEnv, agent_seed: u64, max_steps: u32, out: &mut [f32]) -> u32 {
    let mut rng = Rng::for_stream(agent_seed, StreamId::AGENT);
    let mut legal = LegalCommands::new();
    let mut steps = 0;
    while let Some(side) = env.to_move() {
        if max_steps != 0 && steps >= max_steps {
            break;
        }
        // The buffer has the encoding's length.
        let _ = env.encode_observation(side, out);
        black_box(&*out);
        let command = pick(env, side, &mut rng, &mut legal);
        let _ = step_command(env, side, command);
        steps += 1;
    }
    steps
}

fn main() -> ExitCode {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let Some((command, rest)) = raw.split_first() else {
        eprintln!("{USAGE}");
        return ExitCode::FAILURE;
    };
    let result = Args::parse(rest).and_then(|args| match command.as_str() {
        "run" => run(&args),
        "verify" => verify(&args),
        "bench" => bench(&args),
        _ => Err(format!("unknown command `{command}`\n{USAGE}")),
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
