//! `h3 adventure …`: the adventure's commands (E08 T9b). The binary hands them everything after
//! `adventure`; they parse their own arguments, so the battle's commands stay as they are.
//!
//! ```text
//! h3 adventure run --map <file.h3m> [--packs-dir packs] [--root .] [--profile base:faithful]
//!                  [--difficulty 1] [--seed N] [--agent-seed N] [--days N] [--max-steps N]
//!                  [--checkpoint-every N] [--record <out.toml>]
//! h3 adventure verify [--packs-dir packs] [--root .] <replay.toml>...
//! h3 adventure bench --map <file.h3m> [--workload play|reach|fork] [--packs-dir packs] [--root .]
//!                    [--profile base:faithful] [--difficulty 1] [--games N] [--days N]
//!                    [--seed N] [--forks N] [--observe 0|1] [--masks 0|1]
//! ```
//!
//! `--root` is the directory map paths are relative to (the repository's root for the fixtures);
//! a replay records the path as given. `--difficulty` is the game's (E08 T14b: a row of the
//! rules' starting resources, 0 easy to 4 impossible in the original packs; default 1, normal, as
//! a map's import sets it), checked by the game's build like the rest of the setup; a replay
//! records it and `verify` plays at it.

use std::path::PathBuf;

use h3_game::DEFAULT_DIFFICULTY;
use h3_sim::adventure::replay::RecordOptions;

use super::bench::{BenchOptions, Workload};
use super::{DEFAULT_PROFILE, RunOptions, run_random, verify_file};

/// The adventure commands' usage.
pub const USAGE: &str = "usage:
  h3 adventure run --map <file.h3m> [--packs-dir packs] [--root .] [--profile base:faithful]
                   [--difficulty 1] [--seed N] [--agent-seed N] [--days N] [--max-steps N]
                   [--checkpoint-every N] [--record <out.toml>]
  h3 adventure verify [--packs-dir packs] [--root .] <replay.toml>...
  h3 adventure bench --map <file.h3m> [--workload play|reach|fork] [--packs-dir packs] [--root .]
                     [--profile base:faithful] [--difficulty 1] [--games N] [--days N]
                     [--seed N] [--forks N] [--observe 0|1] [--masks 0|1] [--battle 0|1]";

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

    fn path(&self, name: &str, default: &str) -> PathBuf {
        PathBuf::from(self.get(name).unwrap_or(default))
    }
}

/// Runs `h3 adventure <command> …` with everything after `adventure`.
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
        "run" => run(&args),
        "verify" => verify(&args),
        "bench" => bench(&args),
        _ => Err(format!("unknown adventure command `{command}`\n{USAGE}")),
    }
}

fn run(args: &Args) -> Result<(), String> {
    args.check_known(&[
        "map",
        "packs-dir",
        "root",
        "profile",
        "difficulty",
        "seed",
        "agent-seed",
        "days",
        "max-steps",
        "checkpoint-every",
        "record",
    ])?;
    let options = RunOptions {
        map: args.get("map").ok_or("--map is required")?.to_owned(),
        profile: args.get("profile").unwrap_or(DEFAULT_PROFILE).to_owned(),
        difficulty: args.num("difficulty", DEFAULT_DIFFICULTY)?,
        record: RecordOptions {
            seed: args.num("seed", 1)?,
            agent_seed: args.num("agent-seed", 1)?,
            max_days: args.num("days", 28)?,
            max_steps: args.num("max-steps", 0)?,
            checkpoint_every: args.num("checkpoint-every", 25)?,
        },
    };
    if options.record.max_days == 0 && options.record.max_steps == 0 {
        return Err("a game ends only by a limit: give --days or --max-steps".to_owned());
    }
    let (replay, text) = run_random(
        &args.path("packs-dir", "packs"),
        &args.path("root", "."),
        &options,
    )
    .map_err(|e| e.to_string())?;
    println!("commands: {}", replay.commands.len());
    println!("day:      {}", replay.final_day);
    println!("hash:     {:#018x}", replay.final_hash);
    if let Some(out) = args.get("record") {
        std::fs::write(out, text).map_err(|e| format!("{out}: {e}"))?;
        println!("replay written to {out}");
    }
    Ok(())
}

fn verify(args: &Args) -> Result<(), String> {
    args.check_known(&["packs-dir", "root"])?;
    if args.positional.is_empty() {
        return Err("verify needs at least one replay file".to_owned());
    }
    let (packs_dir, root) = (args.path("packs-dir", "packs"), args.path("root", "."));
    let mut failed = 0;
    for file in &args.positional {
        match verify_file(&PathBuf::from(file), &packs_dir, &root) {
            Ok(v) => println!(
                "ok   {file}: {} ({} commands, day {}, final hash {:#018x})",
                v.map, v.commands, v.day, v.final_hash
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
    args.check_known(&[
        "map",
        "workload",
        "packs-dir",
        "root",
        "profile",
        "difficulty",
        "games",
        "days",
        "seed",
        "forks",
        "observe",
        "masks",
        "battle",
    ])?;
    let workload = match args.get("workload").unwrap_or("play") {
        "play" => Workload::Play,
        "reach" => Workload::Reach,
        "fork" => Workload::Fork,
        other => return Err(format!("--workload: `{other}` is not play, reach or fork")),
    };
    let options = BenchOptions {
        map: args.get("map").ok_or("--map is required")?.to_owned(),
        profile: args.get("profile").unwrap_or(DEFAULT_PROFILE).to_owned(),
        difficulty: args.num("difficulty", DEFAULT_DIFFICULTY)?,
        workload,
        games: args.num("games", 10)?,
        days: args.num("days", 28)?,
        seed: args.num("seed", 1)?,
        forks: args.num("forks", 10_000)?,
        observe: args.num("observe", 0u8)? != 0,
        masks: args.num("masks", 0u8)? != 0,
        battle: args.num("battle", 0u8)? != 0,
    };
    if options.days == 0 {
        return Err("a game ends only by its day limit: --days must be above 0".to_owned());
    }
    let r = super::bench::run(
        &args.path("packs-dir", "packs"),
        &args.path("root", "."),
        &options,
    )
    .map_err(|e| e.to_string())?;
    let what = match workload {
        Workload::Play => "steps",
        Workload::Reach => "queries",
        Workload::Fork => "forks",
    };
    let counted = if workload == Workload::Play {
        String::new()
    } else {
        format!(", {what}: {}", r.operations)
    };
    println!(
        "games: {}, steps: {}{counted}, time: {:.3}s",
        options.games, r.steps, r.seconds
    );
    if workload == Workload::Fork {
        println!(
            "forks with a battle open: {} of {} games",
            r.battles, options.games
        );
    }
    println!(
        "size of GameState: {} bytes, AdventureEnv: {} bytes",
        size_of::<h3_game::GameState>(),
        size_of::<h3_sim::adventure::AdventureEnv>()
    );
    let per_second = r.operations as f64 / r.seconds;
    println!(
        "{what}/s: {per_second:.0} ({:.2} µs each){}{}",
        1e6 / per_second,
        if options.observe {
            ", observation before each step"
        } else {
            ""
        },
        if options.masks {
            ", masks before each step"
        } else {
            ""
        },
    );
    Ok(())
}
