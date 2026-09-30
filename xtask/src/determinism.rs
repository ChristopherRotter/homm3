//! `cargo xtask determinism` and `cargo xtask ffi-smoke`.
//!
//! Determinism matrix (ADR-0003 §5, docs/05-testing.md level 3): the golden replay fixtures must
//! reproduce their hashes in debug, in release, and in release with `-C target-cpu=native`
//! (separate target directory so the flag does not invalidate the normal build cache). Without
//! CI (OPEN-QUESTIONS Q3) the cross-platform half is run by hand on each OS.

use crate::util::{cargo, run_cmd, workspace_root};
use crate::verify::Status;

const FIXTURE_TEST: [&str; 4] = ["-p", "h3-cli", "--test", "replay_fixtures"];

pub fn run(_args: &[String]) -> Result<(), String> {
    println!("determinism: debug");
    let mut debug = cargo();
    debug.arg("test").args(FIXTURE_TEST).arg("--quiet");
    run_cmd(debug)?;

    println!("determinism: release");
    let mut release = cargo();
    release
        .args(["test", "--release"])
        .args(FIXTURE_TEST)
        .arg("--quiet");
    run_cmd(release)?;

    println!("determinism: release + target-cpu=native");
    let mut native = cargo();
    native
        .args(["test", "--release"])
        .args(FIXTURE_TEST)
        .arg("--quiet")
        .arg("--target-dir")
        .arg(workspace_root().join("target").join("native"))
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
    let lib = root.join("target/debug/libh3_ffi.a");
    let exe = root.join("target/debug/h3-ffi-smoke");
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
