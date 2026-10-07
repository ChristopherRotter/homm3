//! `cargo xtask bench` (criterion wall-time benchmarks, local only) and
//! `cargo xtask bench-check [--update]` (deterministic instruction-count regression gate).
//!
//! The gate runs fixed workloads of the release `h3` binary under valgrind/callgrind, counting
//! instructions (`Ir`) only inside the hot function selected by `collect`, and compares them
//! with `benches/baseline.toml`. Counts are deterministic for a given toolchain, target and
//! source, so the baseline records the toolchain and target and is only compared when both
//! match (a mismatch fails: re-baseline deliberately). Changes beyond the threshold fail in both
//! directions. valgrind exists on Linux only; elsewhere the gate is not applicable.

use std::fs;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::util::{cargo, run_cmd, workspace_root};
use crate::verify::Status;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Baseline {
    pub toolchain: String,
    pub target: String,
    pub workload: Vec<Workload>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Workload {
    pub name: String,
    pub description: String,
    pub args: Vec<String>,
    pub collect: String,
    pub instructions: u64,
    pub threshold_percent: u64,
}

const HEADER: &str = "\
# Instruction-count baselines for `cargo xtask bench-check` (valgrind/callgrind, event Ir).
# Counts are deterministic for a given toolchain + target + source and are compared only when
# both match this file. A change above `threshold_percent` fails verify. Update intentionally
# with `cargo xtask bench-check --update` and state the reason in the commit message.
";

/// Verdict for one workload.
#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    Ok,
    Regressed,
    Improved,
}

/// Compares a measurement with its baseline (`threshold_percent` in whole percent).
pub fn judge(baseline: u64, measured: u64, threshold_percent: u64) -> Verdict {
    let limit = u128::from(baseline) * u128::from(100 + threshold_percent) / 100;
    let floor = u128::from(baseline) * u128::from(100u64.saturating_sub(threshold_percent)) / 100;
    if u128::from(measured) > limit {
        Verdict::Regressed
    } else if u128::from(measured) < floor {
        Verdict::Improved
    } else {
        Verdict::Ok
    }
}

/// Parses the `summary:` line of a callgrind output file.
pub fn parse_callgrind(text: &str) -> Option<u64> {
    text.lines()
        .find_map(|l| l.strip_prefix("summary:"))
        .and_then(|v| v.trim().parse().ok())
}

fn toolchain_and_target() -> Result<(String, String), String> {
    let out = Command::new("rustc")
        .current_dir(workspace_root())
        .arg("-vV")
        .output()
        .map_err(|e| format!("rustc -vV: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout);
    let toolchain = text.lines().next().unwrap_or_default().trim().to_owned();
    let target = text
        .lines()
        .find_map(|l| l.strip_prefix("host:"))
        .unwrap_or_default()
        .trim()
        .to_owned();
    Ok((toolchain, target))
}

fn measure(root: &Path, w: &Workload) -> Result<u64, String> {
    let target = crate::util::target_dir()?;
    let out_file = target.join(format!("callgrind.{}.out", w.name));
    let mut cmd = Command::new("valgrind");
    cmd.current_dir(root)
        .arg("--tool=callgrind")
        .arg(format!("--callgrind-out-file={}", out_file.display()))
        .arg(format!("--toggle-collect={}", w.collect))
        .arg(target.join("release/h3"))
        .args(&w.args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    run_cmd(cmd)?;
    let text = fs::read_to_string(&out_file).map_err(|e| format!("{}: {e}", out_file.display()))?;
    parse_callgrind(&text).ok_or_else(|| format!("{}: no `summary:` line", out_file.display()))
}

fn load(root: &Path) -> Result<Baseline, String> {
    let path = root.join("benches/baseline.toml");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("benches/baseline.toml: {e}"))
}

/// The gate. `update` rewrites the baseline with the current toolchain and measurements.
pub fn check(update: bool) -> Result<Status, String> {
    if !cfg!(target_os = "linux") {
        return Ok(Status::NotApplicable(
            "instruction counting needs valgrind (Linux only)".to_owned(),
        ));
    }
    if Command::new("valgrind").arg("--version").output().is_err() {
        return Err(
            "valgrind is not installed (apt install valgrind), or pass `--skip bench-check`"
                .to_owned(),
        );
    }
    let root = workspace_root();
    let mut baseline = load(&root)?;
    let (toolchain, target) = toolchain_and_target()?;
    if !update && (baseline.toolchain != toolchain || baseline.target != target) {
        return Err(format!(
            "baseline recorded with `{}` on {}, this is `{toolchain}` on {target}; \
             re-baseline with `cargo xtask bench-check --update` and say why in the commit",
            baseline.toolchain, baseline.target
        ));
    }
    // The binary is linked without RELRO, a single writable segment: valgrind 3.22 reads no
    // symbols from an lld-linked executable whose two writable segments the kernel maps as
    // one (the RELRO end falling on a page boundary), and the gate would then measure nothing
    // (E04b T3b2). Only the final link differs; the code measured is the same.
    let mut build = cargo();
    build.args([
        "rustc",
        "--release",
        "--quiet",
        "-p",
        "h3-cli",
        "--bin",
        "h3",
        "--",
        "-C",
        "link-arg=-Wl,-z,norelro",
    ]);
    run_cmd(build)?;
    let mut failures = Vec::new();
    for w in &mut baseline.workload {
        let measured = measure(&root, w)?;
        if measured == 0 {
            return Err(format!(
                "{}: 0 instructions measured; the collect pattern `{}` matched nothing",
                w.name, w.collect
            ));
        }
        let delta =
            (measured as f64 - w.instructions as f64) / w.instructions.max(1) as f64 * 100.0;
        println!(
            "bench-check: {}: {measured} Ir (baseline {}, {delta:+.2}%)",
            w.name, w.instructions
        );
        match judge(w.instructions, measured, w.threshold_percent) {
            Verdict::Ok => {}
            Verdict::Improved => failures.push(format!(
                "{} dropped by {delta:+.2}% (threshold {}%): if the improvement is real, record it \
                 with `cargo xtask bench-check --update`; a large drop can also mean the workload \
                 no longer measures what it should",
                w.name, w.threshold_percent
            )),
            Verdict::Regressed => failures.push(format!(
                "{} regressed by {delta:+.2}% (threshold {}%)",
                w.name, w.threshold_percent
            )),
        }
        if update {
            w.instructions = measured;
        }
    }
    if update {
        baseline.toolchain = toolchain;
        baseline.target = target;
        let body = toml::to_string_pretty(&baseline).map_err(|e| e.to_string())?;
        fs::write(
            root.join("benches/baseline.toml"),
            format!("{HEADER}\n{body}"),
        )
        .map_err(|e| e.to_string())?;
        println!("bench-check: baseline updated");
        return Ok(Status::Passed);
    }
    if failures.is_empty() {
        Ok(Status::Passed)
    } else {
        Err(failures.join("; "))
    }
}

/// [`check`] for the verify pipeline.
pub fn check_status() -> Result<Status, String> {
    check(false)
}

/// `cargo xtask bench-check [--update]`.
pub fn run_check(args: &[String]) -> Result<(), String> {
    let update = args.iter().any(|a| a == "--update");
    match check(update)? {
        Status::Passed => Ok(()),
        Status::NotApplicable(why) => {
            println!("bench-check: not applicable: {why}");
            Ok(())
        }
    }
}

/// `cargo xtask bench`: criterion wall-time benchmarks (noisy; for local investigation).
pub fn run_criterion(args: &[String]) -> Result<(), String> {
    let mut cmd = cargo();
    cmd.args(["bench", "-p", "h3-sim", "--bench", "battle", "--"])
        .args(args);
    run_cmd(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn judging() {
        assert_eq!(judge(1000, 1000, 2), Verdict::Ok);
        assert_eq!(judge(1000, 1020, 2), Verdict::Ok);
        assert_eq!(judge(1000, 1021, 2), Verdict::Regressed);
        assert_eq!(judge(1000, 980, 2), Verdict::Ok);
        assert_eq!(judge(1000, 979, 2), Verdict::Improved);
        assert_eq!(judge(1000, 0, 200), Verdict::Ok);
    }

    #[test]
    fn parses_callgrind_summary() {
        assert_eq!(
            parse_callgrind("events: Ir\nsummary: 12345\ntotals: 12345\n"),
            Some(12345)
        );
        assert_eq!(parse_callgrind("events: Ir\n"), None);
    }

    #[test]
    fn baseline_file_parses() {
        let b = load(&workspace_root()).unwrap();
        assert!(!b.workload.is_empty());
        assert!(
            b.workload
                .iter()
                .all(|w| w.instructions > 0 && w.threshold_percent > 0)
        );
    }
}
