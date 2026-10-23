//! `cargo xtask verify`: the single quality gate (docs/05-testing.md, ADR-0010).
//!
//! - `--scope <lane>` is the lane's fast local loop (lanes.toml): lint, test and cover only the
//!   lane's crates, run the lane's own gates (golden replays, C ABI smoke test, its benchmark
//!   workloads) and `check-lane`, and skip what belongs to shared paths (`deny`). A scoped run
//!   never counts as green; CI runs the full verify on every pull request (once per code state).
//! - `--summary` prints one line per gate and the end of the first failure's output; the full
//!   output goes to `<target>/verify/verify.log`. Each gate then runs in a child process
//!   (`cargo xtask verify-step`), so that everything it prints lands in the log.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

use crate::lanes::Config;
use crate::util::{cargo, has_cargo_subcommand, rel, run_cmd, workspace_root};

/// How a gate ended when it did not fail.
#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    /// The gate ran and passed.
    Passed,
    /// The gate cannot run on this platform (e.g. valgrind on Windows); the reason is printed.
    /// Only for platform limitations, never for missing tools that could be installed.
    NotApplicable(String),
}

/// The lane a scoped run is restricted to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneScope {
    pub lane: String,
    /// Workspace packages the lane owns.
    pub packages: Vec<String>,
    /// Whether the lane owns golden replays of its own under `fixtures/replays/` (a sub-path of
    /// the replay crate's owner, such as the adventure's `fixtures/replays/adventure/`).
    pub replays: bool,
}

/// How a gate behaves under `--scope <lane>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InScope {
    /// Runs as in a full verify (fast, repository-wide checks).
    Whole,
    /// Runs on the lane's packages; skipped when the lane has none.
    Packages,
    /// Runs when the lane owns this package (the gate exercises it).
    Owner(&'static str),
    /// Runs when the lane owns the replay crate `h3-cli` or golden replays of its own.
    Replays,
    /// Runs on the lane's benchmark workloads; skipped when it has none.
    Workloads,
    /// Skipped: the gate guards shared paths only.
    Shared,
    /// Runs only in a scoped verify (CI runs it before the verify).
    ScopedOnly,
}

/// One gate of the verify pipeline.
struct Step {
    name: &'static str,
    /// Slow steps are skipped by `--quick`.
    slow: bool,
    in_scope: InScope,
    action: fn(Option<&LaneScope>) -> Result<Status, String>,
}

const fn step(
    name: &'static str,
    slow: bool,
    in_scope: InScope,
    action: fn(Option<&LaneScope>) -> Result<Status, String>,
) -> Step {
    Step {
        name,
        slow,
        in_scope,
        action,
    }
}

fn passed(r: Result<(), String>) -> Result<Status, String> {
    r.map(|()| Status::Passed)
}

/// The gates, in order (docs/05-testing.md "Gates").
const STEPS: &[Step] = &[
    step("fmt", false, InScope::Whole, |_| passed(fmt())),
    step("clippy", false, InScope::Packages, |s| passed(clippy(s))),
    step("deny", false, InScope::Shared, |_| passed(deny())),
    step("check-arch", false, InScope::Whole, |_| {
        passed(crate::arch::run(&[]))
    }),
    step("check-rules", false, InScope::Whole, |_| {
        passed(crate::rules::run(&[]))
    }),
    step("check-stubs", false, InScope::Whole, |_| {
        passed(crate::stubs::run(&[]))
    }),
    step("check-docs", false, InScope::Whole, |_| {
        passed(crate::docs::run(&[]))
    }),
    step("check-exe-map", false, InScope::Whole, |_| {
        passed(crate::exemap::run(&[]))
    }),
    step("clients-lock", false, InScope::Whole, |_| {
        passed(clients_lock())
    }),
    step("check-lane", false, InScope::ScopedOnly, |s| {
        passed(crate::lanes::check(s.map(|s| s.lane.as_str()), None, false))
    }),
    step("test", false, InScope::Packages, |s| passed(test(s))),
    step("coverage", true, InScope::Packages, |s| {
        passed(crate::coverage::gate(s.map(|s| s.packages.as_slice())))
    }),
    step("determinism", true, InScope::Replays, |_| {
        passed(crate::determinism::run(&[]))
    }),
    step("ffi-smoke", false, InScope::Owner("h3-ffi"), |_| {
        crate::determinism::ffi_smoke_status()
    }),
    step("bench-check", true, InScope::Workloads, |s| {
        crate::bench::check(
            false,
            s.map_or(crate::bench::Selection::All, |s| {
                crate::bench::Selection::Lane(&s.lane)
            }),
        )
    }),
    step("doc-drift", false, InScope::Whole, |_| {
        crate::drift::status()
    }),
];

/// `-p <package>` for each package of the scope, or `--workspace`.
fn package_args(scope: Option<&LaneScope>) -> Vec<String> {
    match scope {
        None => vec!["--workspace".to_owned()],
        Some(s) => s
            .packages
            .iter()
            .flat_map(|p| ["-p".to_owned(), p.clone()])
            .collect(),
    }
}

fn fmt() -> Result<(), String> {
    let mut cmd = cargo();
    cmd.args(["fmt", "--all", "--", "--check"]);
    run_cmd(cmd)
}

fn clippy(scope: Option<&LaneScope>) -> Result<(), String> {
    let mut cmd = cargo();
    cmd.arg("clippy")
        .args(package_args(scope))
        .args(["--all-targets", "--", "-D", "warnings"]);
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

/// The client workspace's own lockfile is current (`clients/README.md`): an engine crate the
/// client builds (a path dependency, `.github/workflows/clients.yml`) that changes its
/// dependencies changes `clients/Cargo.lock` too, in the same pull request (a common path,
/// `lanes.toml`). `cargo update --workspace --locked` resolves the workspace's own packages with
/// the registry's index only: no download, no build; it fails when the lockfile would change.
fn clients_lock() -> Result<(), String> {
    let manifest = workspace_root().join("clients").join("Cargo.toml");
    let mut cmd = cargo();
    cmd.args([
        "update",
        "--workspace",
        "--locked",
        "--quiet",
        "--manifest-path",
    ])
    .arg(&manifest);
    run_cmd(cmd).map_err(|e| {
        format!(
            "{e}\n`clients/Cargo.lock` is stale: an engine crate the client workspace builds \
             changed its dependencies. Update it in the same pull request (a common path): \
             `cargo update --workspace --manifest-path clients/Cargo.toml`"
        )
    })
}

fn test(scope: Option<&LaneScope>) -> Result<(), String> {
    let mut cmd = cargo();
    cmd.arg("test")
        .args(package_args(scope))
        .args(["--all-targets", "--quiet"]);
    run_cmd(cmd)?;
    // Doc tests are not part of --all-targets.
    let mut doc = cargo();
    doc.arg("test")
        .args(package_args(scope))
        .args(["--doc", "--quiet"]);
    run_cmd(doc)
}

/// The `test` gate when `coverage` runs in the same verify: its instrumented run is a run of every
/// test of the same packages (`--all-targets`; a failing test fails it), so `test` runs only what
/// coverage leaves out: the doc tests and, in a full verify, `xtask`'s own tests.
const TEST_REST: &str = "test-rest";

fn test_rest(scope: Option<&LaneScope>) -> Result<(), String> {
    if scope.is_none() {
        let mut xtask = cargo();
        xtask.args(["test", "-p", "xtask", "--all-targets", "--quiet"]);
        run_cmd(xtask)?;
    }
    let mut doc = cargo();
    doc.arg("test")
        .args(package_args(scope))
        .args(["--doc", "--quiet"]);
    run_cmd(doc)
}

/// The `determinism` gate when the verify has run the golden replays in a debug build already
/// (in `test` or `coverage`): only its release and `target-cpu=native` runs.
const DETERMINISM_OPTIMIZED: &str = "determinism-optimized";

#[derive(Debug, PartialEq, Eq, Default)]
struct Options {
    quick: bool,
    summary: bool,
    scope: Option<String>,
    skip: Vec<String>,
    /// `--with-original-data [path]`: also re-harvest from the user's installation copy and
    /// require the committed packs to match (local only, never CI; E02 T5).
    original_data: Option<String>,
}

/// Default location of the installation copy (gitignored, ADR-0009).
const DEFAULT_ORIGINAL_DATA: &str = "original-data/install";

/// Name of the `--with-original-data` step.
const ORIGINAL_DATA: &str = "original-data";

/// Re-harvests from `install` and compares with the committed packs (read-only on `install`).
fn harvest_check(install: &str) -> Result<(), String> {
    let mut cmd = cargo();
    cmd.args([
        "run",
        "--quiet",
        "-p",
        "h3-harvest",
        "--",
        "--install",
        install,
        "--out",
        "packs",
        "--check",
    ]);
    run_cmd(cmd)
}

/// Name of the example a crate provides to check itself against the original data.
const ORIGINAL_DATA_EXAMPLE: &str = "original_data";

/// Crates (by directory name, sorted) that carry `examples/original_data.rs`: their checks against
/// the original data, such as `h3-map` reading every shipped map (E08 T3).
fn original_data_examples(root: &Path) -> Vec<String> {
    let mut crates: Vec<String> = fs::read_dir(root.join("crates"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            e.path()
                .join("examples")
                .join(format!("{ORIGINAL_DATA_EXAMPLE}.rs"))
                .is_file()
        })
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    crates.sort();
    crates
}

/// Runs each crate's `original_data` example with the installation copy as its one argument
/// (read-only on `install`); a crate's example fails the gate with a non-zero exit.
fn crate_checks(root: &Path, install: &str) -> Result<(), String> {
    for name in original_data_examples(root) {
        let mut cmd = cargo();
        cmd.args([
            "run",
            "--quiet",
            "-p",
            &name,
            "--example",
            ORIGINAL_DATA_EXAMPLE,
            "--",
            install,
        ]);
        run_cmd(cmd)?;
    }
    Ok(())
}

/// The executable the code-reading tool reads (docs/exe/README.md).
const EXE: &str = "original-data/install/Heroes3.exe";

/// `scripts/exe.py check`: the address map's function starts, sizes and H3API tags against the
/// staged executable, which `check-exe-map` (no executable in CI) cannot check. The tool's exit
/// code 1 means the map disagrees and fails the gate; without the executable, or where the tool
/// cannot run (no Python 3.11 with capstone and pefile: exit code 3), the check is skipped with
/// a note, since `--with-original-data` is also the user's local Windows run.
fn exe_check(root: &Path) -> Result<(), String> {
    if !root.join(EXE).is_file() {
        println!("exe check: skipped ({EXE} is not staged)");
        return Ok(());
    }
    let interpreters: &[&str] = if cfg!(windows) {
        &["python", "python3"]
    } else {
        &["python3", "python"]
    };
    for python in interpreters {
        let status = Command::new(python)
            .current_dir(root)
            .args(["-I", "scripts/exe.py", "check"])
            .status();
        match status {
            Err(_) => continue,
            Ok(s) if s.success() => return Ok(()),
            Ok(s) if s.code() == Some(1) => {
                return Err(format!(
                    "scripts/exe.py check: the map disagrees with {EXE}"
                ));
            }
            Ok(s) => {
                println!("exe check: skipped ({python} scripts/exe.py check ended with {s})");
                return Ok(());
            }
        }
    }
    println!("exe check: skipped (no python3 or python on PATH)");
    Ok(())
}

fn parse(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--quick" => options.quick = true,
            "--summary" => options.summary = true,
            "--scope" => {
                options.scope = Some(iter.next().ok_or("--scope needs a lane")?.clone());
            }
            "--with-original-data" => {
                let path = match iter.as_slice().first() {
                    Some(next) if !next.starts_with("--") => {
                        iter.next();
                        next.clone()
                    }
                    _ => DEFAULT_ORIGINAL_DATA.to_owned(),
                };
                options.original_data = Some(path);
            }
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

/// Error text used to signal "passed with skipped steps" (exit code 2, see main.rs).
pub const NOT_GREEN: &str = "verify incomplete: steps were skipped or the run was scoped";

/// Exit code of `verify-step` for a gate that is not applicable on this platform.
pub const EXIT_NOT_APPLICABLE: u8 = 3;
/// Prefix of the reason line `verify-step` prints for a gate that is not applicable.
const NOT_APPLICABLE_LINE: &str = "verify-step: N/A: ";

enum Outcome {
    Passed,
    NotApplicable(String),
    Failed(String),
    Skipped(String),
}

/// The lane scope for `lane`: its packages from `cargo metadata` and lanes.toml.
pub fn lane_scope(lane: &str) -> Result<LaneScope, String> {
    let root = workspace_root();
    let config = Config::load(&root)?;
    if config.lane(lane).is_none() {
        return Err(format!(
            "unknown lane `{lane}`; lanes: {}",
            config.lane_names().join(", ")
        ));
    }
    let packages: Vec<(String, String)> = crate::arch::load_packages()?
        .into_iter()
        .map(|p| (p.name, rel(&root, &p.dir)))
        .collect();
    let owned = config
        .lane_packages(lane, &packages)
        .into_iter()
        .map(str::to_owned)
        .collect();
    let replays = config.lane(lane).is_some_and(|l| {
        l.paths
            .iter()
            .any(|p| p.starts_with(REPLAY_FIXTURES) && has_files(&root.join(p)))
    });
    Ok(LaneScope {
        lane: lane.to_owned(),
        packages: owned,
        replays,
    })
}

/// The crate whose test re-runs the golden replays (`crate::determinism`).
const REPLAY_CRATE: &str = "h3-cli";
/// Where golden replays live; a lane may own a folder below it (lanes.toml).
const REPLAY_FIXTURES: &str = "fixtures/replays/";

/// Whether `dir` is a directory with at least one entry.
fn has_files(dir: &Path) -> bool {
    fs::read_dir(dir).is_ok_and(|mut entries| entries.next().is_some())
}

/// Why a gate does not run under `scope`, if it does not.
fn out_of_scope(step: &Step, scope: Option<&LaneScope>) -> Option<String> {
    let s = scope?;
    match step.in_scope {
        InScope::Whole | InScope::ScopedOnly => None,
        InScope::Packages if s.packages.is_empty() => {
            Some(format!("lane `{}` has no crates yet", s.lane))
        }
        InScope::Packages => None,
        InScope::Owner(package) if s.packages.iter().any(|p| p == package) => None,
        InScope::Owner(package) => Some(format!("`{package}` is not lane `{}`'s", s.lane)),
        InScope::Replays if s.replays || s.packages.iter().any(|p| p == REPLAY_CRATE) => None,
        InScope::Replays => Some(format!("lane `{}` has no golden replays yet", s.lane)),
        InScope::Workloads => {
            let has = crate::bench::load(&workspace_root())
                .is_ok_and(|entries| entries.iter().any(|e| e.lane == s.lane));
            (!has).then(|| format!("lane `{}` has no benchmark workload", s.lane))
        }
        InScope::Shared => Some("shared paths only; CI runs it".to_owned()),
    }
}

/// Runs one gate in this process.
fn run_step(
    name: &str,
    scope: Option<&LaneScope>,
    install: Option<&str>,
) -> Result<Status, String> {
    if name == TEST_REST {
        return passed(test_rest(scope));
    }
    if name == DETERMINISM_OPTIMIZED {
        return passed(crate::determinism::run_optimized());
    }
    if name == ORIGINAL_DATA {
        let install = install.unwrap_or(DEFAULT_ORIGINAL_DATA);
        return harvest_check(install)
            .and_then(|()| exe_check(&workspace_root()))
            .and_then(|()| crate_checks(&workspace_root(), install))
            .map(|()| Status::Passed);
    }
    let step = STEPS
        .iter()
        .find(|s| s.name == name)
        .ok_or_else(|| format!("unknown step `{name}`"))?;
    (step.action)(scope)
}

/// `cargo xtask verify-step <step> [--scope <lane>] [--install <path>]`: one gate, for
/// `verify --summary`. Exit code 0 passed, 1 failed, [`EXIT_NOT_APPLICABLE`] not applicable.
pub fn run_one(args: &[String]) -> Result<Status, String> {
    let mut iter = args.iter();
    let name = iter.next().ok_or("verify-step needs a step name")?.clone();
    let mut scope = None;
    let mut install = None;
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--scope" => scope = Some(lane_scope(iter.next().ok_or("--scope needs a lane")?)?),
            "--install" => install = Some(iter.next().ok_or("--install needs a path")?.clone()),
            other => return Err(format!("unknown verify-step option `{other}`")),
        }
    }
    let status = run_step(&name, scope.as_ref(), install.as_deref())?;
    if let Status::NotApplicable(why) = &status {
        println!("{NOT_APPLICABLE_LINE}{why}");
    }
    Ok(status)
}

/// The log of a `--summary` run.
fn summary_log() -> Result<PathBuf, String> {
    let dir = crate::util::target_dir()?.join("verify");
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    Ok(dir.join("verify.log"))
}

/// Runs one gate as a child process whose output is appended to `log`.
fn run_logged(
    log: &PathBuf,
    name: &str,
    scope: Option<&LaneScope>,
    install: Option<&str>,
) -> Result<Outcome, String> {
    let mut file = OpenOptions::new()
        .append(true)
        .open(log)
        .map_err(|e| format!("{}: {e}", log.display()))?;
    let start = file.seek(SeekFrom::End(0)).map_err(|e| e.to_string())?;
    let header = format!("\n==> {name}\n");
    std::io::Write::write_all(&mut file, header.as_bytes()).map_err(|e| e.to_string())?;
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let mut cmd = Command::new(exe);
    cmd.current_dir(workspace_root())
        .args(["verify-step", name]);
    if let Some(s) = scope {
        cmd.args(["--scope", &s.lane]);
    }
    if let Some(path) = install {
        cmd.args(["--install", path]);
    }
    let out = file.try_clone().map_err(|e| e.to_string())?;
    let err = file.try_clone().map_err(|e| e.to_string())?;
    let status = cmd
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(err)
        .status()
        .map_err(|e| format!("could not start verify-step {name}: {e}"))?;
    let mut section = String::new();
    let mut reader = File::open(log).map_err(|e| e.to_string())?;
    reader
        .seek(SeekFrom::Start(start))
        .map_err(|e| e.to_string())?;
    reader
        .read_to_string(&mut section)
        .map_err(|e| e.to_string())?;
    Ok(match status.code() {
        Some(0) => Outcome::Passed,
        Some(code) if code == i32::from(EXIT_NOT_APPLICABLE) => Outcome::NotApplicable(
            section
                .lines()
                .find_map(|l| l.strip_prefix(NOT_APPLICABLE_LINE))
                .unwrap_or("not applicable")
                .to_owned(),
        ),
        _ => Outcome::Failed(section),
    })
}

/// The last `n` lines of `text`.
fn tail(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(n)..].join("\n")
}

/// The one-line reason of a failed gate's output: its `error:` line, else its last line.
fn failure_line(section: &str) -> String {
    section
        .lines()
        .rev()
        .find_map(|l| l.strip_prefix("error: "))
        .or_else(|| section.lines().rev().find(|l| !l.trim().is_empty()))
        .unwrap_or("failed")
        .to_owned()
}

/// Which variant of `test` and `determinism` a verify runs, so that each test runs once in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Plan {
    /// `test` runs only what coverage leaves out ([`TEST_REST`]).
    test_rest: bool,
    /// `determinism` runs only its release legs ([`DETERMINISM_OPTIMIZED`]).
    determinism_optimized: bool,
}

impl Plan {
    /// `test_runs`: the `test` gate is not skipped; `coverage_tests`: the `coverage` gate runs and
    /// runs the tests (it has logic crates to measure); `replay_crate`: the tested packages
    /// include the replay crate, whose debug test run is determinism's debug leg.
    fn new(test_runs: bool, coverage_tests: bool, replay_crate: bool) -> Self {
        Plan {
            test_rest: coverage_tests,
            determinism_optimized: replay_crate && (test_runs || coverage_tests),
        }
    }

    /// The step `name` runs as.
    fn variant(self, name: &str) -> &str {
        match name {
            "test" if self.test_rest => TEST_REST,
            "determinism" if self.determinism_optimized => DETERMINISM_OPTIMIZED,
            _ => name,
        }
    }

    fn note(variant: &str) -> &'static str {
        if variant == TEST_REST {
            "doc tests (and xtask's); coverage runs every other test"
        } else {
            "release and target-cpu=native; the debug run was test's or coverage's"
        }
    }
}

pub fn run_steps(args: &[String]) -> Result<(), String> {
    let options = parse(args)?;
    let scope = options.scope.as_deref().map(lane_scope).transpose()?;
    let log = if options.summary {
        let path = summary_log()?;
        File::create(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        println!("verify --summary: full log in {}", path.display());
        Some(path)
    } else {
        None
    };
    if let Some(s) = &scope {
        println!(
            "verify scoped to lane `{}` (crates: {})",
            s.lane,
            if s.packages.is_empty() {
                "none yet".to_owned()
            } else {
                s.packages.join(", ")
            }
        );
    }
    let mut names: Vec<&str> = STEPS
        .iter()
        .filter(|s| scope.is_some() || s.in_scope != InScope::ScopedOnly)
        .map(|s| s.name)
        .collect();
    if options.original_data.is_some() {
        names.push(ORIGINAL_DATA);
    }
    let skip_reason = |name: &str| {
        let step = STEPS.iter().find(|s| s.name == name);
        if options.skip.iter().any(|s| s == name) {
            Some("--skip".to_owned())
        } else if options.quick && step.is_some_and(|s| s.slow) {
            Some("--quick".to_owned())
        } else {
            step.and_then(|s| out_of_scope(s, scope.as_ref()))
                .map(|why| format!("--scope: {why}"))
        }
    };
    let plan = Plan::new(
        skip_reason("test").is_none(),
        skip_reason("coverage").is_none()
            && crate::coverage::runs_tests(scope.as_ref().map(|s| s.packages.as_slice()))?,
        scope
            .as_ref()
            .is_none_or(|s| s.packages.iter().any(|p| p == REPLAY_CRATE)),
    );
    let mut results = Vec::new();
    let mut first_failure: Option<(&str, String)> = None;
    for name in names {
        let skipped = skip_reason(name);
        let run_name = plan.variant(name);
        let start = Instant::now();
        let outcome = if let Some(why) = skipped {
            Outcome::Skipped(why)
        } else if let Some(log) = &log {
            if run_name != name {
                println!("  ...   {name}: {}", Plan::note(run_name));
            }
            run_logged(
                log,
                run_name,
                scope.as_ref(),
                options.original_data.as_deref(),
            )?
        } else {
            println!("\n==> {name}");
            if run_name != name {
                println!("{name}: {}", Plan::note(run_name));
            }
            let result = run_step(run_name, scope.as_ref(), options.original_data.as_deref());
            println!("<== {name} ({:.1}s)", start.elapsed().as_secs_f64());
            match result {
                Ok(Status::Passed) => Outcome::Passed,
                Ok(Status::NotApplicable(why)) => Outcome::NotApplicable(why),
                Err(e) => Outcome::Failed(e),
            }
        };
        if log.is_some() {
            let secs = start.elapsed().as_secs_f64();
            match &outcome {
                Outcome::Passed => println!("  PASS  {name} ({secs:.1}s)"),
                Outcome::NotApplicable(why) => println!("  N/A   {name}: {why}"),
                Outcome::Failed(section) => {
                    println!("  FAIL  {name} ({secs:.1}s): {}", failure_line(section));
                }
                Outcome::Skipped(why) => println!("  SKIP  {name} ({why})"),
            }
        }
        if let (Outcome::Failed(text), None) = (&outcome, &first_failure) {
            first_failure = Some((name, text.clone()));
        }
        results.push((name, outcome));
    }

    let failed = results
        .iter()
        .filter(|(_, o)| matches!(o, Outcome::Failed(_)))
        .count();
    let skipped = results
        .iter()
        .filter(|(_, o)| matches!(o, Outcome::Skipped(_)))
        .count();
    if let Some(log) = &log {
        if let Some((name, section)) = &first_failure {
            println!(
                "\nfirst failure: {name} (end of its output; all of it in {}):",
                log.display()
            );
            println!("{}", tail(section, 25));
        }
    } else {
        println!("\nverify summary:");
        for (name, outcome) in &results {
            match outcome {
                Outcome::Passed => println!("  PASS  {name}"),
                Outcome::NotApplicable(why) => println!("  N/A   {name}: {why}"),
                Outcome::Failed(e) => println!("  FAIL  {name}: {e}"),
                Outcome::Skipped(why) => println!("  SKIP  {name} ({why})"),
            }
        }
    }
    if failed > 0 {
        return Err(format!("{failed} verify step(s) failed"));
    }
    if skipped > 0 || scope.is_some() {
        if let Some(s) = &scope {
            println!(
                "\nverify passed for lane `{}` with {skipped} skipped step(s): a scoped run is NOT \
                 a green verify (CI runs the full one).",
                s.lane
            );
        } else {
            println!("\nverify passed with {skipped} skipped step(s): this is NOT a green verify.");
        }
        // Exit code 2: nothing failed, but the run does not count as green.
        return Err(NOT_GREEN.to_owned());
    }
    println!("\nverify is green.");
    Ok(())
}

pub fn run(args: &[String]) -> Result<(), String> {
    run_steps(args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::test_support::TempDir;

    fn strings(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn each_test_runs_once_in_a_verify() {
        // A full verify: coverage runs every test, its debug run includes the replays.
        let full = Plan::new(true, true, true);
        assert_eq!(full.variant("test"), TEST_REST);
        assert_eq!(full.variant("determinism"), DETERMINISM_OPTIMIZED);
        assert_eq!(full.variant("clippy"), "clippy");
        // Coverage skipped: test runs whole, and its debug run is determinism's.
        let no_coverage = Plan::new(true, false, true);
        assert_eq!(no_coverage.variant("test"), "test");
        assert_eq!(no_coverage.variant("determinism"), DETERMINISM_OPTIMIZED);
        // A lane whose tested packages leave out the replay crate keeps the debug leg.
        let without_replays = Plan::new(true, true, false);
        assert_eq!(without_replays.variant("determinism"), "determinism");
        // Neither test nor coverage runs: determinism runs whole.
        let untested = Plan::new(false, false, true);
        assert_eq!(untested.variant("test"), "test");
        assert_eq!(untested.variant("determinism"), "determinism");
    }

    #[test]
    fn finds_the_crates_with_an_original_data_example() {
        let dir = TempDir::new("original-data-examples");
        dir.write("crates/h3-b/examples/original_data.rs", "fn main() {}\n");
        dir.write("crates/h3-a/examples/original_data.rs", "fn main() {}\n");
        dir.write("crates/h3-c/examples/other.rs", "fn main() {}\n");
        dir.write("crates/h3-d/src/lib.rs", "");
        assert_eq!(original_data_examples(dir.path()), ["h3-a", "h3-b"]);
        let empty = TempDir::new("original-data-examples-none");
        assert!(original_data_examples(empty.path()).is_empty());
        assert_eq!(crate_checks(empty.path(), "original-data/install"), Ok(()));
    }

    #[test]
    fn exe_check_is_skipped_without_the_executable() {
        let dir = TempDir::new("exe-check");
        dir.write("scripts/exe.py", "import sys\nsys.exit(1)\n");
        assert_eq!(exe_check(dir.path()), Ok(()));
    }

    #[test]
    fn parses_quick_skip_summary_and_scope() {
        let options = parse(&strings(&[
            "--quick",
            "--skip",
            "deny",
            "--summary",
            "--scope",
            "battle",
        ]))
        .unwrap();
        assert_eq!(
            options,
            Options {
                quick: true,
                summary: true,
                scope: Some("battle".to_owned()),
                skip: vec!["deny".to_owned()],
                original_data: None,
            }
        );
    }

    #[test]
    fn parses_with_original_data() {
        let default = parse(&strings(&["--with-original-data", "--quick"])).unwrap();
        assert_eq!(
            default.original_data.as_deref(),
            Some(DEFAULT_ORIGINAL_DATA)
        );
        assert!(default.quick);
        let path = parse(&strings(&["--with-original-data", "D:/H3"])).unwrap();
        assert_eq!(path.original_data.as_deref(), Some("D:/H3"));
    }

    #[test]
    fn rejects_unknown_step_and_option() {
        assert!(parse(&strings(&["--skip", "nope"])).is_err());
        assert!(parse(&strings(&["--skip"])).is_err());
        assert!(parse(&strings(&["--scope"])).is_err());
        assert!(parse(&strings(&["--frobnicate"])).is_err());
        assert!(run_one(&strings(&[])).is_err());
        assert!(run_one(&strings(&["nope"])).is_err());
        assert!(run_one(&strings(&["fmt", "--what"])).is_err());
    }

    #[test]
    fn scopes_steps() {
        let step = |name| STEPS.iter().find(|s| s.name == name).unwrap();
        let battle = LaneScope {
            lane: "battle".to_owned(),
            packages: vec!["h3-cli".to_owned(), "h3-battle".to_owned()],
            replays: false,
        };
        let client = LaneScope {
            lane: "client".to_owned(),
            packages: Vec::new(),
            replays: false,
        };
        let adventure = |replays| LaneScope {
            lane: "adventure".to_owned(),
            packages: vec!["h3-adventure".to_owned()],
            replays,
        };
        assert_eq!(out_of_scope(step("fmt"), Some(&client)), None);
        assert_eq!(
            out_of_scope(step("clients-lock"), Some(&adventure(true))),
            None
        );
        assert_eq!(out_of_scope(step("test"), None), None);
        assert_eq!(out_of_scope(step("test"), Some(&battle)), None);
        assert!(out_of_scope(step("test"), Some(&client)).is_some());
        assert_eq!(out_of_scope(step("determinism"), Some(&battle)), None);
        assert!(out_of_scope(step("determinism"), Some(&adventure(false))).is_some());
        assert_eq!(
            out_of_scope(step("determinism"), Some(&adventure(true))),
            None
        );
        assert!(out_of_scope(step("ffi-smoke"), Some(&battle)).is_some());
        assert!(out_of_scope(step("deny"), Some(&battle)).is_some());
        assert_eq!(out_of_scope(step("check-lane"), Some(&client)), None);
        assert_eq!(out_of_scope(step("bench-check"), Some(&battle)), None);
        assert!(out_of_scope(step("bench-check"), Some(&client)).is_some());
        assert_eq!(
            package_args(Some(&battle)),
            strings(&["-p", "h3-cli", "-p", "h3-battle"])
        );
        assert_eq!(package_args(None), strings(&["--workspace"]));
    }

    #[test]
    fn lane_scopes_come_from_lanes_toml() {
        let battle = lane_scope("battle").unwrap();
        assert!(battle.packages.iter().any(|p| p == "h3-battle"));
        assert!(
            !battle
                .packages
                .iter()
                .any(|p| p == "h3-kernel" || p == "xtask")
        );
        assert!(lane_scope("client").unwrap().packages.is_empty());
        assert!(lane_scope("map-client").unwrap().packages.is_empty());
        let adventure = lane_scope("adventure").unwrap();
        assert!(adventure.packages.iter().any(|p| p == "h3-game"));
        assert!(!adventure.packages.iter().any(|p| p == "h3-sim"));
        assert!(!battle.replays);
        assert!(!has_files(
            &workspace_root().join("fixtures/no-such-folder")
        ));
        assert!(has_files(&workspace_root().join("fixtures/replays")));
        assert!(lane_scope("nope").is_err());
    }

    #[test]
    fn summarises_failures() {
        let section = "==> test\ncompiling\nerror: `cargo test` failed\n";
        assert_eq!(failure_line(section), "`cargo test` failed");
        assert_eq!(failure_line("a\nlast line\n\n"), "last line");
        assert_eq!(failure_line(""), "failed");
        assert_eq!(tail("1\n2\n3", 2), "2\n3");
        assert_eq!(tail("1", 5), "1");
    }
}
