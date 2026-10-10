//! Project automation for the h3 workspace. Run as `cargo xtask <command>`.
//!
//! `cargo xtask verify` is the single quality gate (docs/05-testing.md): it must be green before a
//! task counts as done.
// xtask is tooling, not game logic: it may use the clock, environment, processes and threads.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use std::process::ExitCode;

mod arch;
mod bench;
mod coverage;
mod determinism;
mod docs;
mod drift;
mod exemap;
mod lanes;
mod rules;
mod scan;
mod stubs;
mod util;
mod verify;

const USAGE: &str = "\
usage: cargo xtask <command> [options]

commands:
  verify [--quick] [--skip <step>]... [--scope <lane>] [--summary]
                                        run every gate (see docs/05-testing.md)
  check-arch                            crate graph + logic-crate purity (arch.toml)
  check-rules                           rule catalog + rule:/covers: traceability
  check-stubs                           stub registry vs todo!/#[ignore]/TODO markers
  check-docs                            HANDOFF size budgets (global and per lane), lane files
  check-exe-map                         the Heroes3.exe address map is well formed and covers every
                                        address the docs and code cite (docs/exe/README.md)
  check-lane [--lane <lane>] [--base <ref>] [--cross-lane]
                                        the branch changes only its lane's paths (lanes.toml)
  coverage                              100% line+region coverage on logic crates
  determinism                           golden replays in debug, release, release+native
  ffi-smoke                             build and run the C ABI smoke test (needs a C compiler)
  doc-drift                             warn if logic changed without docs (heuristic)
  bench [criterion args]                criterion wall-time benchmarks (local, noisy)
  bench-check [--update] [--lane <lane>] [--workload <name>]
                                        instruction-count regression gate (valgrind, Linux)
  help                                  show this message

`--quick` skips the slow gates (determinism matrix, coverage, benchmark regression) and is for
inner-loop development only; it never counts as a green verify (exit code 2 when steps were
skipped, 1 when a step failed, 0 only for a complete green run).

`--scope <lane>` is a lane's fast local loop: lint, test and cover the lane's crates, run its own
gates and `check-lane`, skip shared-only gates. A scoped run never counts as green either.

`--summary` prints one line per gate plus the end of the first failure's output and writes the
full output to `<target>/verify/verify.log`.

`verify --with-original-data [path]` adds a local-only step: re-harvest the packs from the
installation copy (default `original-data/install`, read-only) and require `packs/{base,ab,sod}`
to match byte for byte (E02), check the executable's address map against `Heroes3.exe`, and run
every crate's `examples/original_data.rs` with the installation path (E08: `h3-map` reads every
shipped map in `Maps/`).";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (command, rest) = match args.split_first() {
        Some((c, rest)) => (c.as_str(), rest),
        None => ("help", &[][..]),
    };
    if command == "verify-step" {
        // Internal: one gate for `verify --summary`, which reads the exit code.
        return match verify::run_one(rest) {
            Ok(verify::Status::Passed) => ExitCode::SUCCESS,
            Ok(verify::Status::NotApplicable(_)) => ExitCode::from(verify::EXIT_NOT_APPLICABLE),
            Err(message) => {
                eprintln!("error: {message}");
                ExitCode::FAILURE
            }
        };
    }
    let result = match command {
        "verify" => verify::run(rest),
        "check-arch" => arch::run(rest),
        "check-rules" => rules::run(rest),
        "check-stubs" => stubs::run(rest),
        "check-docs" => docs::run(rest),
        "check-exe-map" => exemap::run(rest),
        "check-lane" => lanes::run(rest),
        "coverage" => coverage::run(rest),
        "determinism" => determinism::run(rest),
        "ffi-smoke" => determinism::ffi_smoke(rest),
        "doc-drift" => drift::run(rest),
        "bench" => bench::run_criterion(rest),
        "bench-check" => bench::run_check(rest),
        "help" | "--help" | "-h" => {
            println!("{USAGE}");
            Ok(())
        }
        other => Err(format!("unknown command `{other}`\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) if message == verify::NOT_GREEN => {
            eprintln!("{message} (exit code 2)");
            ExitCode::from(2)
        }
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}
