//! `cargo xtask determinism` and `cargo xtask ffi-smoke`.
//!
//! Determinism matrix (ADR-0003 §5, docs/05-testing.md level 3): the golden replay fixtures must
//! reproduce their hashes in debug, in release, and in release with `-C target-cpu=native`
//! (separate target directory so the flag does not invalidate the normal build cache). Without
//! CI (OPEN-QUESTIONS Q3) the cross-platform half is run by hand on each OS.

use std::path::Path;

use crate::util::{cargo, run_cmd, workspace_root};
use crate::verify::Status;

/// The `h3-cli` test targets that re-run golden replays: the battle's, and the adventure's
/// (`tests/adventure_replays.rs`, adventure-owned in lanes.toml) once it exists (E08 T9).
const REPLAY_TESTS: [&str; 2] = ["replay_fixtures", "adventure_replays"];

/// `-p h3-cli` and `--test <name>` for each replay test target that exists.
fn fixture_tests(root: &Path) -> Vec<String> {
    let tests = root.join("crates/h3-cli/tests");
    let mut args = vec!["-p".to_owned(), "h3-cli".to_owned()];
    for name in REPLAY_TESTS {
        if tests.join(format!("{name}.rs")).is_file() {
            args.extend(["--test".to_owned(), name.to_owned()]);
        }
    }
    args
}

pub fn run(_args: &[String]) -> Result<(), String> {
    let fixture_test = fixture_tests(&workspace_root());
    println!("determinism: debug");
    let mut debug = cargo();
    debug.arg("test").args(&fixture_test).arg("--quiet");
    run_cmd(debug)?;

    println!("determinism: release");
    let mut release = cargo();
    release
        .args(["test", "--release"])
        .args(&fixture_test)
        .arg("--quiet");
    run_cmd(release)?;

    println!("determinism: release + target-cpu=native");
    let mut native = cargo();
    native
        .args(["test", "--release"])
        .args(&fixture_test)
        .arg("--quiet")
        .arg("--target-dir")
        .arg(crate::util::target_dir()?.join("native"))
        .env("RUSTFLAGS", "-C target-cpu=native");
    run_cmd(native)
}

/// Builds the static library and the C smoke program (`crates/h3-ffi/c/smoke.c`) and runs it.
pub fn ffi_smoke(_args: &[String]) -> Result<(), String> {
    match ffi_smoke_status()? {
        Status::Passed => Ok(()),
        Status::NotApplicable(why) => {
            println!("ffi-smoke: {why}");
            Ok(())
        }
    }
}

/// [`ffi_smoke`] for the verify pipeline. On Windows (MSVC) there is no `cc`; the Rust-side ABI
/// tests in `h3-ffi` still run there.
pub fn ffi_smoke_status() -> Result<Status, String> {
    if cfg!(windows) {
        return Ok(Status::NotApplicable(
            "no cc on Windows; the Rust-side ABI tests in h3-ffi cover the functions".to_owned(),
        ));
    }
    let root = workspace_root();
    let mut build = cargo();
    build.args(["build", "--quiet", "-p", "h3-ffi"]);
    run_cmd(build)?;
    let cc = std::env::var("CC").unwrap_or_else(|_| "cc".to_owned());
    let target = crate::util::target_dir()?;
    let lib = target.join("debug/libh3_ffi.a");
    let exe = target.join("debug/h3-ffi-smoke");
    let mut compile = std::process::Command::new(&cc);
    compile
        .args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-I"])
        .arg(root.join("crates/h3-ffi/include"))
        .arg(root.join("crates/h3-ffi/c/smoke.c"))
        .arg(&lib)
        .args(["-lpthread", "-ldl", "-lm", "-o"])
        .arg(&exe);
    run_cmd(compile).map_err(|e| {
        format!("{e} (a C compiler is required; set CC or pass `--skip ffi-smoke`)")
    })?;
    run_cmd(std::process::Command::new(&exe)).map(|()| Status::Passed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_tests_are_the_existing_targets() {
        let args = fixture_tests(&workspace_root());
        assert_eq!(&args[..4], ["-p", "h3-cli", "--test", "replay_fixtures"]);
        assert_eq!(fixture_tests(Path::new("no-such-root")), ["-p", "h3-cli"]);
    }
}
