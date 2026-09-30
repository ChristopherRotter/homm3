//! Project automation for the h3 workspace. Run as `cargo xtask <command>`.
//!
//! `cargo xtask verify` is the single quality gate (docs/05-testing.md): it must be green before a
//! task counts as done.
// xtask is tooling, not game logic: it may use the clock, environment, processes and threads.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use std::process::ExitCode;

mod arch;
mod coverage;
mod rules;
mod scan;
mod stubs;
mod util;
mod verify;

const USAGE: &str = "\
usage: cargo xtask <command> [options]

commands:
  verify [--quick] [--skip <step>]...   run every gate (see docs/05-testing.md)
  check-arch                            crate graph + logic-crate purity (arch.toml)
  check-rules                           rule catalog + rule:/covers: traceability
  check-stubs                           stub registry vs todo!/#[ignore]/TODO markers
  coverage                              100% line+region coverage on logic crates
  help                                  show this message

`--quick` skips the slow gates (determinism matrix, coverage, benchmark regression) and is for
inner-loop development only; it never counts as a green verify.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (command, rest) = match args.split_first() {
        Some((c, rest)) => (c.as_str(), rest),
        None => ("help", &[][..]),
    };
    let result = match command {
        "verify" => verify::run(rest),
        "check-arch" => arch::run(rest),
        "check-rules" => rules::run(rest),
        "check-stubs" => stubs::run(rest),
        "coverage" => coverage::run(rest),
        "help" | "--help" | "-h" => {
            println!("{USAGE}");
            Ok(())
        }
        other => Err(format!("unknown command `{other}`\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}
