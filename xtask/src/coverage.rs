//! `cargo xtask coverage`: 100% line and region coverage on logic crates (ADR-0010,
//! docs/05-testing.md level 1), minus the reviewed `coverage-exclusions.toml`.
//!
//! Coverage is measured with LLVM source-based coverage via `cargo llvm-cov`. Region misses are
//! read from the merged per-file segment view (the one `llvm-cov show` renders), so a region of a
//! generic function counts as covered when any instantiation executes it.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::arch::{self, report};
use crate::util::{cargo, has_cargo_subcommand, rel, run_cmd, workspace_root};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Exclusions {
    #[serde(default)]
    exclude: Vec<Exclusion>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Exclusion {
    path: String,
    reason: String,
}

/// Per-file coverage result.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileCoverage {
    pub file: String,
    /// Lines with an execution count of zero (from the lcov line records, merged over
    /// instantiations by taking the maximum count).
    pub missed_lines: Vec<u64>,
    /// `(line, column)` of region entries that were never executed.
    pub missed_regions: Vec<(u64, u64)>,
}

/// Extracts never-executed region entries per file from an `llvm-cov export` JSON document.
pub fn parse_regions(json: &str, root: &Path) -> Result<BTreeMap<String, Vec<(u64, u64)>>, String> {
    let v: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("coverage json: {e}"))?;
    let files = v["data"][0]["files"]
        .as_array()
        .ok_or("coverage json: no data[0].files")?;
    let mut out = BTreeMap::new();
    for f in files {
        let name = f["filename"]
            .as_str()
            .ok_or("coverage json: file without name")?;
        let mut missed = BTreeSet::new();
        for seg in f["segments"].as_array().into_iter().flatten() {
            // [line, col, count, hasCount, isRegionEntry, isGapRegion]
            let s = seg.as_array().ok_or("coverage json: bad segment")?;
            let num = |i: usize| s.get(i).and_then(serde_json::Value::as_u64).unwrap_or(0);
            let flag = |i: usize| {
                s.get(i)
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
            };
            if flag(3) && flag(4) && !flag(5) && num(2) == 0 {
                missed.insert((num(0), num(1)));
            }
        }
        out.insert(rel(root, Path::new(name)), missed.into_iter().collect());
    }
    Ok(out)
}

/// Extracts never-executed lines per file from an lcov document (`SF:` / `DA:line,count`).
pub fn parse_lcov(lcov: &str, root: &Path) -> BTreeMap<String, Vec<u64>> {
    let mut out = BTreeMap::new();
    let mut current: Option<(String, BTreeMap<u64, u64>)> = None;
    for line in lcov.lines() {
        if let Some(path) = line.strip_prefix("SF:") {
            current = Some((rel(root, Path::new(path)), BTreeMap::new()));
        } else if let Some(da) = line.strip_prefix("DA:") {
            let mut parts = da.split(',');
            let (Some(Ok(l)), Some(Ok(c))) = (
                parts.next().map(str::parse::<u64>),
                parts.next().map(str::parse::<u64>),
            ) else {
                continue;
            };
            if let Some((_, counts)) = current.as_mut() {
                let e = counts.entry(l).or_insert(0);
                *e = (*e).max(c);
            }
        } else if line == "end_of_record"
            && let Some((file, counts)) = current.take()
        {
            let missed = counts
                .into_iter()
                .filter(|&(_, c)| c == 0)
                .map(|(l, _)| l)
                .collect();
            out.insert(file, missed);
        }
    }
    out
}

/// Combines region and line results.
pub fn combine(
    regions: BTreeMap<String, Vec<(u64, u64)>>,
    lines: &BTreeMap<String, Vec<u64>>,
) -> Vec<FileCoverage> {
    let mut files: BTreeSet<&String> = regions.keys().collect();
    files.extend(lines.keys());
    files
        .into_iter()
        .map(|f| FileCoverage {
            file: f.clone(),
            missed_lines: lines.get(f).cloned().unwrap_or_default(),
            missed_regions: regions.get(f).cloned().unwrap_or_default(),
        })
        .collect()
}

/// Checks gated files (under one of `gated_dirs`, not excluded) for 100% line and region coverage.
pub fn check(
    files: &[FileCoverage],
    gated_dirs: &[String],
    excluded: &BTreeSet<String>,
) -> Vec<String> {
    let mut v = Vec::new();
    for f in files {
        if !gated_dirs.iter().any(|d| f.file.starts_with(d.as_str())) || excluded.contains(&f.file)
        {
            continue;
        }
        for line in &f.missed_lines {
            v.push(format!("{}:{line}: line never executed", f.file));
        }
        for (line, col) in &f.missed_regions {
            v.push(format!("{}:{line}:{col}: region never executed", f.file));
        }
    }
    v
}

fn load_exclusions(root: &Path) -> Result<Vec<Exclusion>, String> {
    let path = root.join("coverage-exclusions.toml");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let parsed: Exclusions =
        toml::from_str(&text).map_err(|e| format!("coverage-exclusions.toml: {e}"))?;
    Ok(parsed.exclude)
}

fn llvm_tools_available() -> bool {
    if std::env::var_os("LLVM_COV").is_some() && std::env::var_os("LLVM_PROFDATA").is_some() {
        return true;
    }
    std::process::Command::new("rustup")
        .args(["component", "list", "--installed"])
        .output()
        .is_ok_and(|o| String::from_utf8_lossy(&o.stdout).contains("llvm-tools"))
}

/// `cargo llvm-cov clean --workspace`: the workspace members' instrumented artifacts, their
/// profiles (`*.profraw`, `*.profdata`) and old reports; third-party dependencies stay.
fn clean_instrumented() -> Result<(), String> {
    let mut cmd = cargo();
    cmd.args(["llvm-cov", "clean", "--workspace"]);
    run_cmd(cmd)
}

/// Runs the instrumented tests of `packages` (all but `xtask` without a scope), writes the JSON and
/// lcov reports and reads them back.
fn measure(
    root: &Path,
    packages: Option<&[String]>,
    json_path: &Path,
    lcov_path: &Path,
) -> Result<Vec<FileCoverage>, String> {
    let mut test = cargo();
    test.arg("llvm-cov");
    match packages {
        None => {
            test.args(["--workspace", "--exclude", "xtask"]);
        }
        Some(list) => {
            for p in list {
                test.args(["-p", p]);
            }
        }
    }
    test.args(["--all-targets", "--no-report"]);
    run_cmd(test)?;
    let mut json_report = cargo();
    json_report
        .args(["llvm-cov", "report", "--json", "--output-path"])
        .arg(json_path);
    run_cmd(json_report)?;
    let mut lcov_report = cargo();
    lcov_report
        .args(["llvm-cov", "report", "--lcov", "--output-path"])
        .arg(lcov_path);
    run_cmd(lcov_report)?;
    let read = |p: &Path| fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()));
    Ok(combine(
        parse_regions(&read(json_path)?, root)?,
        &parse_lcov(&read(lcov_path)?, root),
    ))
}

pub fn run(_args: &[String]) -> Result<(), String> {
    gate(None)
}

/// The gate. With `packages` (a lane scope, `verify --scope`), only those packages' tests run and
/// only their logic crates are gated; coverage of other crates is left to the full verify.
pub fn gate(packages: Option<&[String]>) -> Result<(), String> {
    if !has_cargo_subcommand("llvm-cov") {
        return Err(
            "cargo-llvm-cov is not installed: `cargo install cargo-llvm-cov --locked`".to_owned(),
        );
    }
    if !llvm_tools_available() {
        return Err(
            "LLVM tools not found: `rustup component add llvm-tools-preview`, or set \
                    LLVM_COV and LLVM_PROFDATA to llvm-cov/llvm-profdata binaries whose LLVM \
                    version is compatible with rustc's (scripts/sandbox-verify.sh sets them)"
                .to_owned(),
        );
    }
    let root = workspace_root();
    let target = crate::util::target_dir()?;
    let json_path = target.join("coverage.json");
    let lcov_path = target.join("coverage.lcov");
    let config = arch::load_config(&root)?;
    let in_scope = |name: &str| packages.is_none_or(|p| p.iter().any(|q| q == name));
    let logic: Vec<arch::Package> = arch::load_packages()?
        .into_iter()
        .filter(|p| config.crates.get(&p.name).is_some_and(|c| c.logic) && in_scope(&p.name))
        .collect();
    if packages.is_some() && logic.is_empty() {
        println!("coverage: no logic crate in scope");
        return Ok(());
    }
    // The instrumented build of the workspace's own crates is removed before the run and again
    // after it (docs/08-work-process.md, "Cloud sandbox" 5). Before: `cargo llvm-cov` cleans only
    // the packages it builds, while `report` reads every test binary under
    // `target/llvm-cov-target`, so a scoped run after a full one, or a run after a branch switch,
    // took stale binaries' counts as misses at lines that no longer match. After: those artifacts
    // and the profiles take several GB and are rebuilt by the next run anyway; the third-party
    // dependencies stay cached, and the reports (`coverage.json`, `coverage.lcov`) stay.
    clean_instrumented()?;
    let measured = measure(&root, packages, &json_path, &lcov_path);
    let cleaned = clean_instrumented();
    let files = measured?;
    cleaned?;

    let gated: Vec<String> = logic
        .iter()
        .map(|p| format!("{}/src/", rel(&root, &p.dir)))
        .collect();
    let exclusions = load_exclusions(&root)?;
    println!("coverage exclusions ({}):", exclusions.len());
    for e in &exclusions {
        println!("  {} — {}", e.path, e.reason);
    }
    let excluded: BTreeSet<String> = exclusions.iter().map(|e| e.path.clone()).collect();
    // Every non-excluded, non-test source file of a logic crate that defines a function must
    // appear in the report; otherwise a path mismatch (symlinks, Windows prefixes) or an
    // uncompiled file would pass silently. (Declaration-only files have no regions, and
    // cargo-llvm-cov leaves test files out of the report by default.)
    let reported: BTreeSet<&str> = files.iter().map(|f| f.file.as_str()).collect();
    let mut missing = Vec::new();
    // Whether any gated file defines a function: logic crates that are still skeletons
    // (declarations only, as in a lane's first increment) have nothing to cover.
    let mut has_code = false;
    for p in &logic {
        for (file, is_test) in crate::scan::classify_crate(&p.dir)? {
            let r = rel(&root, &file);
            if is_test || excluded.contains(&r) {
                continue;
            }
            let text = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
            if crate::scan::find_pattern(&crate::scan::scan(&text, false), "fn").is_empty() {
                continue;
            }
            has_code = true;
            if !reported.contains(r.as_str()) {
                missing.push(format!(
                    "{r}: defines functions but is not in the coverage report"
                ));
            }
        }
    }
    let gated_files = files
        .iter()
        .filter(|f| gated.iter().any(|d| f.file.starts_with(d.as_str())))
        .count();
    println!(
        "coverage: {gated_files} gated file(s) in {} logic crate(s)",
        gated.len()
    );
    if gated_files == 0 && has_code {
        missing.push("no logic-crate file found in the coverage report".to_owned());
    }
    let mut violations = missing;
    violations.extend(check(&files, &gated, &excluded));
    report("coverage", &violations)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPORT: &str = r#"{"data":[{"files":[
      {"filename":"/w/crates/a/src/lib.rs",
       "segments":[[1,1,5,true,true,false],[2,5,0,true,true,false],[3,1,0,true,false,false],[4,1,0,true,true,true]]},
      {"filename":"/w/crates/b/src/lib.rs","segments":[[1,1,0,true,true,false]]}
    ]}]}"#;

    const LCOV: &str = "SF:/w/crates/a/src/lib.rs\nDA:1,5\nDA:2,0\nDA:2,3\nDA:7,0\nDA:x,1\nend_of_record\n\
                        SF:/w/crates/a/src/excluded.rs\nDA:1,0\nend_of_record\n";

    #[test]
    fn parses_and_gates_only_logic_files() {
        let root = Path::new("/w");
        let files = combine(
            parse_regions(EXPORT, root).unwrap(),
            &parse_lcov(LCOV, root),
        );
        assert_eq!(files.len(), 3);
        let a = files
            .iter()
            .find(|f| f.file == "crates/a/src/lib.rs")
            .unwrap();
        assert_eq!(a.missed_regions, vec![(2, 5)]);
        assert_eq!(
            a.missed_lines,
            vec![7],
            "line 2 is covered by another instantiation"
        );
        let excluded: BTreeSet<String> = ["crates/a/src/excluded.rs".to_owned()].into();
        let v = check(&files, &["crates/a/src/".to_owned()], &excluded);
        assert_eq!(
            v,
            vec![
                "crates/a/src/lib.rs:7: line never executed".to_owned(),
                "crates/a/src/lib.rs:2:5: region never executed".to_owned(),
            ]
        );
        assert!(parse_regions("{}", root).is_err());
    }

    #[test]
    fn exclusions_file_parses() {
        let e = load_exclusions(&workspace_root()).unwrap();
        assert!(e.iter().all(|x| !x.reason.trim().is_empty()));
    }
}
