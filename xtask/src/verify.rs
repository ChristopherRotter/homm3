//! `cargo xtask verify`: the single quality gate (docs/05-testing.md, ADR-0010).

use std::time::Instant;

use crate::util::{cargo, has_cargo_subcommand, run_cmd};

/// One gate of the verify pipeline.
struct Step {
    name: &'static str,
    /// Slow steps are skipped by `--quick`.
    slow: bool,
    action: fn() -> Result<(), String>,
}

const fn step(name: &'static str, slow: bool, action: fn() -> Result<(), String>) -> Step {
    Step { name, slow, action }
}

/// The gates, in order (docs/05-testing.md "Gates").
const STEPS: &[Step] = &[
    step("fmt", false, fmt),
    step("clippy", false, clippy),
    step("deny", false, deny),
    step("check-arch", false, || crate::arch::run(&[])),
    step("check-rules", false, || crate::rules::run(&[])),
    step("check-stubs", false, || crate::stubs::run(&[])),
    step("test", false, test),
    step("coverage", true, || crate::coverage::run(&[])),
    step("determinism", true, || crate::determinism::run(&[])),
    step("ffi-smoke", false, || crate::determinism::ffi_smoke(&[])),
];

fn fmt() -> Result<(), String> {
    let mut cmd = cargo();
    cmd.args(["fmt", "--all", "--", "--check"]);
    run_cmd(cmd)
}

fn clippy() -> Result<(), String> {
    let mut cmd = cargo();
    cmd.args([
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ]);
    run_cmd(cmd)
}

fn deny() -> Result<(), String> {
    if !has_cargo_subcommand("deny") {
        return Err(
            "cargo-deny is not installed: `cargo install cargo-deny --locked` \
                    (or pass `--skip deny`, which makes this run not count as green)"
                .to_owned(),
        );
    }
    let mut cmd = cargo();
    cmd.args(["deny", "--all-features", "check"]);
    run_cmd(cmd)
}

fn test() -> Result<(), String> {
    let mut cmd = cargo();
    cmd.args(["test", "--workspace", "--all-targets", "--quiet"]);
    run_cmd(cmd)?;
    // Doc tests are not part of --all-targets.
    let mut doc = cargo();
    doc.args(["test", "--workspace", "--doc", "--quiet"]);
    run_cmd(doc)
}

#[derive(Debug, PartialEq, Eq)]
struct Options {
    quick: bool,
    skip: Vec<String>,
}

fn parse(args: &[String]) -> Result<Options, String> {
    let mut options = Options {
        quick: false,
        skip: Vec::new(),
    };
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--quick" => options.quick = true,
            "--skip" => {
                let step = iter.next().ok_or("--skip needs a step name")?;
                if !STEPS.iter().any(|s| s.name == step) {
                    let names: Vec<_> = STEPS.iter().map(|s| s.name).collect();
                    return Err(format!(
                        "unknown step `{step}`; steps: {}",
                        names.join(", ")
                    ));
                }
                options.skip.push(step.clone());
            }
            other => return Err(format!("unknown verify option `{other}`")),
        }
    }
    Ok(options)
}

enum Outcome {
    Passed,
    Failed(String),
    Skipped(&'static str),
}

pub fn run_steps(args: &[String]) -> Result<(), String> {
    let options = parse(args)?;
    let mut results = Vec::new();
    for step in STEPS {
        let outcome = if options.skip.iter().any(|s| s == step.name) {
            Outcome::Skipped("--skip")
        } else if options.quick && step.slow {
            Outcome::Skipped("--quick")
        } else {
            println!("\n==> {}", step.name);
            let start = Instant::now();
            let result = (step.action)();
            let elapsed = start.elapsed();
            println!("<== {} ({:.1}s)", step.name, elapsed.as_secs_f64());
            match result {
                Ok(()) => Outcome::Passed,
                Err(e) => Outcome::Failed(e),
            }
        };
        results.push((step.name, outcome));
    }

    println!("\nverify summary:");
    let mut failed = 0;
    let mut skipped = 0;
    for (name, outcome) in &results {
        match outcome {
            Outcome::Passed => println!("  PASS  {name}"),
            Outcome::Failed(e) => {
                failed += 1;
                println!("  FAIL  {name}: {e}");
            }
            Outcome::Skipped(why) => {
                skipped += 1;
                println!("  SKIP  {name} ({why})");
            }
        }
    }
    if failed > 0 {
        return Err(format!("{failed} verify step(s) failed"));
    }
    if skipped > 0 {
        println!("\nverify passed with {skipped} skipped step(s): this is NOT a green verify.");
    } else {
        println!("\nverify is green.");
    }
    Ok(())
}

pub fn run(args: &[String]) -> Result<(), String> {
    run_steps(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn parses_quick_and_skip() {
        let options = parse(&strings(&["--quick", "--skip", "deny"])).unwrap();
        assert_eq!(
            options,
            Options {
                quick: true,
                skip: vec!["deny".to_owned()]
            }
        );
    }

    #[test]
    fn rejects_unknown_step_and_option() {
        assert!(parse(&strings(&["--skip", "nope"])).is_err());
        assert!(parse(&strings(&["--skip"])).is_err());
        assert!(parse(&strings(&["--frobnicate"])).is_err());
    }
}
