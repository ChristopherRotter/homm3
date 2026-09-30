//! Small helpers shared by xtask commands.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Root of the Cargo workspace (the parent of the `xtask` directory).
pub fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map_or_else(|| manifest_dir.clone(), Path::to_path_buf)
}

/// A `cargo` command using the same cargo binary that runs xtask.
pub fn cargo() -> Command {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let mut cmd = Command::new(cargo);
    cmd.current_dir(workspace_root());
    cmd
}

/// Runs a command, inheriting stdio, and maps a non-zero exit into an error message.
pub fn run_cmd(mut cmd: Command) -> Result<(), String> {
    let shown = format!("{cmd:?}");
    let status = cmd
        .status()
        .map_err(|e| format!("could not start {shown}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{shown} failed with {status}"))
    }
}

/// Whether a cargo subcommand (e.g. `deny`, `llvm-cov`) is installed.
pub fn has_cargo_subcommand(name: &str) -> bool {
    cargo()
        .arg(name)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}
