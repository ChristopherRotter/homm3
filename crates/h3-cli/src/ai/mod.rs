//! `h3 ai …`: the AI lane's commands (E05, ADR-0025). The binary hands them everything after
//! `ai`; they parse their own arguments.
//!
//! ```text
//! h3 ai bench [--scenario <file>] [--packs-dir packs] [--envs 64] [--threads 0] [--calls 1000]
//!             [--seed 1] [--max-steps 0]
//! ```

// The bench times itself with the wall clock (an adapter).
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

pub mod bench;

use std::path::PathBuf;

use bench::BenchOptions;

/// The AI commands' usage.
pub const USAGE: &str = "usage:
  h3 ai bench [--scenario <file>] [--packs-dir packs] [--envs 64] [--threads 0] [--calls 1000]
              [--seed 1] [--max-steps 0]";

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
    ])?;
    let options = BenchOptions {
        scenario: args.path("scenario", "packs/synthetic/scenarios/full.toml"),
        packs_dir: args.path("packs-dir", "packs"),
        envs: args.num("envs", 64)?,
        threads: args.num("threads", 0)?,
        calls: args.num("calls", 1000)?,
        seed: args.num("seed", 1)?,
        max_steps: args.num("max-steps", 0)?,
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

#[cfg(test)]
mod tests;
