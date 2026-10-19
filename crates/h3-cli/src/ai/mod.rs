//! `h3 ai …`: the AI lane's commands (E05, ADR-0025). The binary hands them everything after
//! `ai`; they parse their own arguments.
//!
//! ```text
//! h3 ai bench [--scenario <file>] [--packs-dir packs] [--envs 64] [--threads 0] [--calls 1000]
//!             [--seed 1] [--max-steps 0] [--encounters default|<six weights, a,b,c,d,e,f>]
//! h3 ai census <maps folder or file>... [--out <file>]
//! h3 ai eval [--policies random,aggressor,greedy] [--episodes 1000] [--envs 100] [--seed <s>]
//!            [--max-steps 2000] [--encounters default|<six weights, a,b,c,d,e,f>]
//! ```

// The bench times itself with the wall clock (an adapter).
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

pub mod bench;
pub mod census;
pub mod eval;
pub mod rating;

use std::path::PathBuf;

use bench::BenchOptions;
use h3_sim::EncounterTable;

/// The AI commands' usage.
pub const USAGE: &str = "usage:
  h3 ai bench [--scenario <file>] [--packs-dir packs] [--envs 64] [--threads 0] [--calls 1000]
              [--seed 1] [--max-steps 0] [--encounters default|<six weights, a,b,c,d,e,f>]
  h3 ai census <maps folder or file>... [--out <file>]
  h3 ai eval [--policies random,aggressor,greedy] [--episodes 1000] [--envs 100] [--seed <s>]
             [--max-steps 2000] [--encounters default|<six weights, a,b,c,d,e,f>]";

/// Parsed arguments: `--name value` options and positional words.
struct Args {
    positional: Vec<String>,
    options: Vec<(String, String)>,
}

impl Args {
    fn parse(raw: &[String]) -> Result<Self, String> {
        let (mut positional, mut options) = (Vec::new(), Vec::new());
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
        self.get(name).map_or(Ok(default), |v| {
            v.parse()
                .map_err(|_| format!("--{name}: `{v}` is not a valid number"))
        })
    }

    fn check(&self, known: &[&str]) -> Result<(), String> {
        if let Some(word) = self.positional.first() {
            return Err(format!("unexpected argument `{word}`"));
        }
        match self
            .options
            .iter()
            .find(|(n, _)| !known.contains(&n.as_str()))
        {
            Some((n, _)) => Err(format!("unknown option --{n}")),
            None => Ok(()),
        }
    }

    fn path(&self, name: &str, default: &str) -> PathBuf {
        PathBuf::from(self.get(name).unwrap_or(default))
    }
}

/// Runs `h3 ai <command> …` with everything after `ai`.
///
/// # Errors
///
/// A message for the user.
pub fn main(raw: &[String]) -> Result<(), String> {
    let Some((command, rest)) = raw.split_first() else {
        return Err(USAGE.to_owned());
    };
    let args = Args::parse(rest)?;
    match command.as_str() {
        "bench" => bench(&args),
        "census" => census(&args),
        "eval" => eval(&args),
        _ => Err(format!("unknown ai command `{command}`\n{USAGE}")),
    }
}

fn bench(args: &Args) -> Result<(), String> {
    args.check(&[
        "scenario",
        "packs-dir",
        "envs",
        "threads",
        "calls",
        "seed",
        "max-steps",
        "encounters",
    ])?;
    let options = BenchOptions {
        scenario: args.path("scenario", "packs/synthetic/scenarios/full.toml"),
        packs_dir: args.path("packs-dir", "packs"),
        envs: args.num("envs", 64)?,
        threads: args.num("threads", 0)?,
        calls: args.num("calls", 1000)?,
        seed: args.num("seed", 1)?,
        max_steps: args.num("max-steps", 0)?,
        encounters: args.get("encounters").map(encounter_weights).transpose()?,
    };
    let report = bench::run(&options)?;
    println!(
        "envs: {}, threads: {}, calls: {}, steps: {}, episodes: {}, refused: {}",
        options.envs, options.threads, options.calls, report.steps, report.episodes, report.refused
    );
    println!(
        "time: {:.3}s, of it in BattleBatch::step: {:.3}s",
        report.secs, report.step_secs
    );
    println!(
        "steps/s inside BattleBatch::step (observations and entry masks written): {:.0}",
        report.steps as f64 / report.step_secs
    );
    println!(
        "steps/s with the head calls and the random choice: {:.0}",
        report.steps as f64 / report.secs
    );
    Ok(())
}

/// `h3 ai eval`: the engine's policies against each other on the held-out encounters (E05 T7).
fn eval(args: &Args) -> Result<(), String> {
    args.check(&[
        "policies",
        "episodes",
        "envs",
        "seed",
        "max-steps",
        "encounters",
    ])?;
    let seed = match args.get("seed") {
        None => eval::EVAL_SEED,
        Some(v) => v
            .strip_prefix("0x")
            .map_or_else(|| v.parse(), |hex| u64::from_str_radix(hex, 16))
            .map_err(|_| format!("--seed: `{v}` is not a number"))?,
    };
    let weights = encounter_weights(args.get("encounters").unwrap_or("default"))?;
    let options = eval::EvalOptions {
        policies: eval::policies(args.get("policies").unwrap_or("random,aggressor,greedy"))?,
        episodes: args.num("episodes", 1000)?,
        envs: args.num("envs", 100)?,
        seed,
        max_steps: args.num("max-steps", 2000)?,
        weights,
    };
    let source = bench::encounters(options.weights)?;
    let start = std::time::Instant::now();
    let report = eval::run(&source, &options)?;
    print!("{}", report.to_text());
    println!(
        "{} episodes per ordered pair, seed {:#x}, {:.1}s",
        report.pairs[0][1].episodes,
        options.seed,
        start.elapsed().as_secs_f64()
    );
    Ok(())
}

/// `--encounters`: `default` (the distribution's own weights) or six weights `a,b,c,d,e,f` (the
/// kinds of `h3_sim::Kind::ALL`, in order).
fn encounter_weights(value: &str) -> Result<[u32; 6], String> {
    if value == "default" {
        return Ok(EncounterTable::v1().kinds.map(|k| k.weight));
    }
    let weights = value
        .split(',')
        .map(|w| w.trim().parse::<u32>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| format!("--encounters: `{value}` is not six weights"))?;
    weights
        .try_into()
        .map_err(|_| format!("--encounters: `{value}` is not six weights"))
}

/// `h3 ai census`: the census of the maps given (a folder's `.h3m` files, or files), as TOML on
/// standard output or into `--out`; the maps it skipped and why on standard error.
fn census(args: &Args) -> Result<(), String> {
    if let Some((n, _)) = args.options.iter().find(|(n, _)| n != "out") {
        return Err(format!("unknown option --{n}"));
    }
    if args.positional.is_empty() {
        return Err(format!("census needs a maps folder or file\n{USAGE}"));
    }
    let paths: Vec<PathBuf> = args.positional.iter().map(PathBuf::from).collect();
    let files = census::map_files(&paths)?;
    let rules = census::original_rules()?;
    let (counted, skipped) = census::run(&files, &rules)?;
    for s in &skipped {
        eprintln!("skipped {}: {}", s.path.display(), s.reason);
    }
    eprintln!(
        "census: {} of {} maps counted",
        counted.census.maps,
        files.len()
    );
    let text = census::to_toml(&counted)?;
    match args.get("out") {
        Some(out) => std::fs::write(out, text).map_err(|e| format!("{out}: {e}")),
        None => {
            print!("{text}");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests;
