//! `cargo xtask bench` (criterion wall-time benchmarks, local only) and
//! `cargo xtask bench-check [--update] [--lane <lane>] [--workload <name>]` (deterministic
//! instruction-count regression gate).
//!
//! The gate runs fixed workloads of a release binary (`h3-cli`'s `h3` unless the workload names
//! another) under valgrind/callgrind, counting instructions (`Ir`) only inside the hot function
//! selected by `collect`, and compares them with their baselines. Each workload has its own file,
//! `benches/<lane>/<workload>.toml`, so each lane's threshold is its own and two lanes never
//! re-baseline the same file (lanes.toml). A workload whose count depends on another lane's code
//! is `lane_only`: measured when its lane or the workload is selected (`--lane`, `--workload`,
//! `verify --scope <lane>`), never by the full verify, so that the other lane's changes cannot
//! fail it there. Counts are deterministic for a given toolchain, target and source, so a baseline
//! records the toolchain and target and is only compared when both match (a mismatch fails:
//! re-baseline deliberately). Changes beyond the threshold fail in both directions. valgrind
//! exists on Linux only; elsewhere the gate is not applicable.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::util::{cargo, run_cmd, workspace_root};
use crate::verify::Status;

/// One baseline file: `benches/<lane>/<workload>.toml`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Baseline {
    pub toolchain: String,
    pub target: String,
    pub workload: Workload,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Workload {
    pub description: String,
    pub args: Vec<String>,
    pub collect: String,
    pub instructions: u64,
    pub threshold_percent: u64,
    /// The package of the binary that runs the workload (default [`DEFAULT_PACKAGE`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// The binary (default [`DEFAULT_BIN`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bin: Option<String>,
    /// Measured only when its lane or the workload itself is selected, never by the full verify:
    /// for a count that depends on another lane's code (the server's session core runs the
    /// battle engine), which that lane's changes must not fail.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub lane_only: bool,
}

/// The package of the binary a workload runs unless it names another.
pub const DEFAULT_PACKAGE: &str = "h3-cli";
/// The binary a workload runs unless it names another.
pub const DEFAULT_BIN: &str = "h3";

impl Workload {
    /// The package and the binary that run the workload.
    pub fn binary(&self) -> (&str, &str) {
        (
            self.package.as_deref().unwrap_or(DEFAULT_PACKAGE),
            self.bin.as_deref().unwrap_or(DEFAULT_BIN),
        )
    }
}

/// A baseline with where it lives.
#[derive(Debug, Clone)]
pub struct Entry {
    /// The owning lane (the directory name).
    pub lane: String,
    /// The workload name (the file stem).
    pub name: String,
    pub path: PathBuf,
    pub baseline: Baseline,
}

/// Which workloads a run measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection<'a> {
    All,
    Lane(&'a str),
    Workload(&'a str),
}

impl Selection<'_> {
    fn selects(&self, e: &Entry) -> bool {
        match self {
            Selection::All => !e.baseline.workload.lane_only,
            Selection::Lane(lane) => e.lane == *lane,
            Selection::Workload(name) => e.name == *name,
        }
    }
}

const HEADER: &str = "\
# Instruction-count baseline of one `cargo xtask bench-check` workload (valgrind/callgrind, event
# Ir); the directory names the owning lane (lanes.toml). Counts are deterministic for a given
# toolchain + target + source and are compared only when both match this file. A change above
# `threshold_percent` fails verify. Update intentionally with
# `cargo xtask bench-check --update --workload <name>` and state the reason in the commit message.
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

fn measure(root: &Path, name: &str, w: &Workload) -> Result<u64, String> {
    let target = crate::util::target_dir()?;
    let out_file = target.join(format!("callgrind.{name}.out"));
    let mut cmd = Command::new("valgrind");
    cmd.current_dir(root)
        .arg("--tool=callgrind")
        .arg(format!("--callgrind-out-file={}", out_file.display()))
        .arg(format!("--toggle-collect={}", w.collect))
        .arg(target.join("release").join(w.binary().1))
        .args(&w.args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    run_cmd(cmd)?;
    let text = fs::read_to_string(&out_file).map_err(|e| format!("{}: {e}", out_file.display()))?;
    parse_callgrind(&text).ok_or_else(|| format!("{}: no `summary:` line", out_file.display()))
}

/// Every baseline under `benches/<lane>/*.toml`, sorted by lane and name.
pub fn load(root: &Path) -> Result<Vec<Entry>, String> {
    let benches = root.join("benches");
    let mut entries = Vec::new();
    let dirs = fs::read_dir(&benches).map_err(|e| format!("{}: {e}", benches.display()))?;
    for dir in dirs {
        let dir = dir.map_err(|e| e.to_string())?.path();
        if !dir.is_dir() {
            continue;
        }
        let lane = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        for file in fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let path = file.map_err(|e| e.to_string())?.path();
            if path.extension().is_none_or(|e| e != "toml") {
                continue;
            }
            let name = path
                .file_stem()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let baseline =
                toml::from_str(&text).map_err(|e| format!("benches/{lane}/{name}.toml: {e}"))?;
            entries.push(Entry {
                lane: lane.clone(),
                name,
                path,
                baseline,
            });
        }
    }
    entries.sort_by(|a, b| (&a.lane, &a.name).cmp(&(&b.lane, &b.name)));
    Ok(entries)
}

/// The gate. `update` rewrites the selected baselines with the current toolchain and
/// measurements.
pub fn check(update: bool, selection: Selection<'_>) -> Result<Status, String> {
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
    let mut entries: Vec<Entry> = load(&root)?
        .into_iter()
        .filter(|e| selection.selects(e))
        .collect();
    if entries.is_empty() {
        return Err(format!("no bench-check workload matches {selection:?}"));
    }
    let (toolchain, target) = toolchain_and_target()?;
    if !update {
        for e in &entries {
            if e.baseline.toolchain != toolchain || e.baseline.target != target {
                return Err(format!(
                    "{}: baseline recorded with `{}` on {}, this is `{toolchain}` on {target}; \
                     re-baseline with `cargo xtask bench-check --update --workload {}` and say \
                     why in the commit",
                    e.name, e.baseline.toolchain, e.baseline.target, e.name
                ));
            }
        }
    }
    // The binaries are linked without RELRO, a single writable segment: valgrind 3.22 reads no
    // symbols from an lld-linked executable whose two writable segments the kernel maps as
    // one (the RELRO end falling on a page boundary), and the gate would then measure nothing
    // (E04b T3b2). Only the final link differs; the code measured is the same.
    let mut binaries: Vec<(&str, &str)> = entries
        .iter()
        .map(|e| e.baseline.workload.binary())
        .collect();
    binaries.sort_unstable();
    binaries.dedup();
    for (package, bin) in binaries {
        let mut build = cargo();
        build.args([
            "rustc",
            "--release",
            "--quiet",
            "-p",
            package,
            "--bin",
            bin,
            "--",
        ]);
        build.args(["-C", "link-arg=-Wl,-z,norelro"]);
        run_cmd(build)?;
    }
    let mut failures = Vec::new();
    for e in &mut entries {
        let w = &mut e.baseline.workload;
        let measured = measure(&root, &e.name, w)?;
        if measured == 0 {
            return Err(format!(
                "{}: 0 instructions measured; the collect pattern `{}` matched nothing",
                e.name, w.collect
            ));
        }
        let delta =
            (measured as f64 - w.instructions as f64) / w.instructions.max(1) as f64 * 100.0;
        println!(
            "bench-check: {}/{}: {measured} Ir (baseline {}, {delta:+.2}%)",
            e.lane, e.name, w.instructions
        );
        match judge(w.instructions, measured, w.threshold_percent) {
            Verdict::Ok => {}
            Verdict::Improved => failures.push(format!(
                "{} dropped by {delta:+.2}% (threshold {}%): if the improvement is real, record it \
                 with `cargo xtask bench-check --update --workload {}`; a large drop can also mean \
                 the workload no longer measures what it should",
                e.name, w.threshold_percent, e.name
            )),
            Verdict::Regressed => failures.push(format!(
                "{} regressed by {delta:+.2}% (threshold {}%)",
                e.name, w.threshold_percent
            )),
        }
        if update {
            w.instructions = measured;
        }
    }
    if update {
        for e in &mut entries {
            e.baseline.toolchain.clone_from(&toolchain);
            e.baseline.target.clone_from(&target);
            let body = toml::to_string_pretty(&e.baseline).map_err(|err| err.to_string())?;
            fs::write(&e.path, format!("{HEADER}\n{body}")).map_err(|err| err.to_string())?;
            println!("bench-check: {}/{}: baseline updated", e.lane, e.name);
        }
        return Ok(Status::Passed);
    }
    if failures.is_empty() {
        Ok(Status::Passed)
    } else {
        Err(failures.join("; "))
    }
}

/// `cargo xtask bench-check [--update] [--lane <lane>] [--workload <name>]`.
pub fn run_check(args: &[String]) -> Result<(), String> {
    let mut update = false;
    let mut lane = None;
    let mut workload = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--update" => update = true,
            "--lane" => lane = Some(iter.next().ok_or("--lane needs a name")?.as_str()),
            "--workload" => {
                workload = Some(iter.next().ok_or("--workload needs a name")?.as_str());
            }
            other => return Err(format!("unknown bench-check option `{other}`")),
        }
    }
    let selection = match (lane, workload) {
        (_, Some(name)) => Selection::Workload(name),
        (Some(lane), None) => Selection::Lane(lane),
        (None, None) => Selection::All,
    };
    match check(update, selection)? {
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
    fn baseline_files_parse_and_belong_to_lanes() {
        let root = workspace_root();
        let entries = load(&root).unwrap();
        assert!(!entries.is_empty());
        let lanes = crate::lanes::Config::load(&root).unwrap();
        for e in &entries {
            let w = &e.baseline.workload;
            assert!(w.instructions > 0 && w.threshold_percent > 0, "{}", e.name);
            assert!(
                lanes.lane(&e.lane).is_some(),
                "benches/{}/ is not a lane",
                e.lane
            );
        }
        let all = Selection::All;
        let first = &entries[0];
        assert!(all.selects(first));
        assert!(Selection::Lane(&first.lane).selects(first));
        assert!(Selection::Workload(&first.name).selects(first));
        assert!(!Selection::Lane("no-such-lane").selects(first));
    }

    fn workload(extra: &str) -> Baseline {
        toml::from_str(&format!(
            "toolchain = \"t\"\ntarget = \"x\"\n[workload]\ndescription = \"d\"\nargs = []\n\
             collect = \"*f*\"\ninstructions = 1\nthreshold_percent = 2\n{extra}"
        ))
        .unwrap()
    }

    #[test]
    fn a_workload_runs_h3_unless_it_names_a_binary() {
        let plain = workload("");
        assert_eq!(plain.workload.binary(), (DEFAULT_PACKAGE, DEFAULT_BIN));
        assert!(!plain.workload.lane_only);
        // The defaults are not written back, so an update leaves a plain baseline as it was.
        let text = toml::to_string_pretty(&plain).unwrap();
        assert!(!text.contains("package") && !text.contains("bin") && !text.contains("lane_only"));
        let named = workload("package = \"p\"\nbin = \"b\"\nlane_only = true\n");
        assert_eq!(named.workload.binary(), ("p", "b"));
        assert_eq!(
            toml::from_str::<Baseline>(&toml::to_string_pretty(&named).unwrap()).unwrap(),
            named
        );
    }

    #[test]
    fn a_lane_only_workload_is_measured_only_for_its_lane() {
        let entry = |lane_only| Entry {
            lane: "server".to_owned(),
            name: "w".to_owned(),
            path: PathBuf::new(),
            baseline: workload(if lane_only { "lane_only = true\n" } else { "" }),
        };
        assert!(Selection::All.selects(&entry(false)));
        assert!(!Selection::All.selects(&entry(true)));
        assert!(Selection::Lane("server").selects(&entry(true)));
        assert!(Selection::Workload("w").selects(&entry(true)));
    }
}
