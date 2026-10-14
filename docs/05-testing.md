# 05 — Testing strategy

## Levels

1. **Unit tests** — every core crate: 100% line **and** region coverage (LLVM source-based coverage, `cargo llvm-cov`). Exclusions only via the single reviewed file `coverage-exclusions.toml` (graphics/sound/OS glue; never domain logic). Adding an exclusion requires a justification line and is called out in HANDOFF.
2. **Property-based tests** (`proptest`) — invariants, see below.
3. **Determinism replay tests** — recorded games (seed + commands) replay to identical events and state hashes; golden hashes compared per platform in CI (Linux, Windows; WASM once it exists), in debug and release, with and without `target-cpu=native` (ADR-0003). Any rule using floats or transcendentals adds a fixture exercising it, plus test vectors for `kernel::num` functions.
4. **Conformance tests** — worked examples from sources; oracle comparisons (ADR-0009); original-data tests (local only, `--with-original-data`).
5. **Statistical tests** — distributions of random mechanics across many seeds.
6. **Fuzzing** (`cargo-fuzz`) — all parsers of untrusted input (map, archive, save, network/command decoding) and the command validator. Not set up yet: it needs a nightly toolchain and `libfuzzer-sys` (NCSA, not in `deny.toml`'s list), so it waits until a deep parser (maps, archives) wants coverage guidance. Meanwhile the network decoder has mutation tests on the stable toolchain (E16 M2 T5d, SRV-Q14: valid messages mutated in their bytes and values, `h3-protocol`'s `tests/mutation.rs`).
7. **Mutation testing** (`cargo-mutants`) — periodic (not per-commit) on core crates; surviving mutants are triaged; a threshold is calibrated after the walking skeleton.
8. **Benchmarks** — `criterion` (local wall time) and an instruction-count tool for deterministic CI regression checks.
9. **Acceptance tests (BDD)** — later; Gherkin features under `features/`, each scenario referencing rule IDs. Not specified further now.

## Invariants every context must provide property tests for

- **Atomicity:** if `apply` returns an error, state (and its hash) is unchanged.
- **Legality consistency:** every command listed by `legal_commands` is accepted by `apply`; commands sampled from outside the set are rejected.
- **Ownership isolation:** a random command sequence issued by actor A never changes anything owned by actor B except through explicitly modelled interactions (battle, trade, …).
- **Conservation:** resources/creatures are conserved except through modelled sources and sinks.
- **Observation hygiene:** `observe(state, ruleset, viewer)` is a function of only what `viewer` may know (two states differing only in hidden info yield equal observations).
- **Clone equivalence:** cloned states evolve identically under the same commands/seed.
- **Determinism:** same inputs ⇒ same outputs, across runs.

## Gates: `cargo xtask verify`

One entry point, used locally and in CI; all must pass:

1. `cargo fmt --check`
2. `cargo clippy` with the workspace lint policy (deny: `unwrap_used`/`expect_used`/`panic` in logic crates; `disallowed_types` for `HashMap`/`HashSet`/time/thread in logic crates; `disallowed_methods` for platform transcendentals (`f32/f64::sin`, `cos`, `exp`, `ln`, `powf`, …) and `mul_add`; direct `f32`/`f64` use only via `kernel::num` or a module-level `allow` tied to a rule ID; `forbid(unsafe_code)` in logic crates)
3. `cargo deny` (licenses: only GPLv2-compatible dependencies per ADR-0009 and `deny.toml`; advisories; bans)
4. `xtask check-arch` — crate dependency graph matches `arch.toml`; forbidden APIs absent from logic crates
5. `xtask check-rules` — rule-catalog traceability (see `04-rules-and-sources.md`)
6. `xtask check-stubs` — stub registry consistency (see `08-work-process.md`)
7. Unit + property + replay tests
8. Coverage gate (100% line and region on core crates, minus the reviewed exclusions)
9. Benchmark regression check vs. stored baselines (thresholds per benchmark; intentional changes update the baseline with a note in the commit)

Optional flags: `--with-original-data`, `--mutants`, `--fuzz-smoke`.

**As implemented in E01** (`cargo xtask verify`; `--quick` skips the slow steps and never counts as green; `--skip <step>` makes the run explicitly not green; `--scope <lane>` and `--summary` since Phase 0, below):

| Step | What | Slow |
|---|---|---|
| fmt | `cargo fmt --check` | |
| clippy | `cargo clippy --workspace --all-targets -D warnings` (workspace lints + `clippy.toml` + logic-crate header) | |
| deny | `cargo deny check` (licenses per ADR-0009, advisories, bans, sources) | |
| check-arch | crate graph and logic-crate purity (`arch.toml`) | |
| check-rules | rule catalog and `rule:`/`covers:` traceability | |
| check-stubs | stub registry vs. `todo!`/`unimplemented!`/`#[ignore]`/`TODO`/`FIXME` | |
| check-docs | the global `docs/HANDOFF.md` within 6 000 bytes; each lane's `docs/lanes/<lane>/HANDOFF.md` within 10 000 bytes with at most three entries under "Recent sessions"; every lane in `lanes.toml` has a `HANDOFF.md` and a `QUEUE.md` (`08-work-process.md`, "Keeping the start-of-session reading small") | |
| check-exe-map | the `Heroes3.exe` address map (`docs/exe/`) is well formed (sorted, unique snake_case names, kinds, confidences, sizes, no overlapping functions), covers every image address cited in `docs/`, `crates/` and `clients/` (not its local output `clients/out/`), and every entry at an address H3API names carries `h3api = "…"` or disputes it (`docs/exe/README.md`); needs no executable. `--with-original-data` adds `scripts/exe.py check` (starts and sizes against the staged executable; skipped with a note where the tool cannot run) | |
| clients-lock | `clients/Cargo.lock` is current: `cargo update --workspace --locked` in `clients/` (the registry's index only, no download); an engine crate the client builds that changes its dependencies updates it in the same pull request (a common path, `lanes.toml`; user, 2026-10-14) | |
| check-lane | only under `--scope <lane>`: the branch changes only the lane's and common paths (`lanes.toml`); CI runs it on every pull request, before the verify | |
| test | all unit, property, integration (incl. golden replays) and doc tests; when coverage runs its tests in the same verify, only what coverage leaves out: the doc tests and, in a full verify, `xtask`'s tests (each test runs once, below) | |
| coverage | `cargo llvm-cov` (its instrumented run runs every test of the packages, `--all-targets`, and fails on a failing test): 100 % lines (lcov, merged over generic instantiations) and regions (merged segments) on logic crates, minus `coverage-exclusions.toml`; fails if a non-test logic file with functions is missing from the report (test files are left out of the report by cargo-llvm-cov) | yes |
| determinism | golden replays in debug, release, release + `target-cpu=native`; the debug run is test's or coverage's when the verify ran the replay crate's tests | yes |
| ffi-smoke | compile and run the C smoke program against the static library (N/A on Windows) | |
| bench-check | instruction counts (valgrind/callgrind) vs. one baseline file per workload, `benches/<lane>/<workload>.toml`, +2 % threshold each (`benches/README.md`; a `lane_only` workload only in its lane's scope) (N/A off Linux or for another toolchain/target); the measured binary is linked without RELRO, as valgrind 3.22 may read no symbols from an lld-linked binary otherwise | yes |
| doc-drift | warning only: logic crates changed since `master` without HANDOFF/lane docs/rules/ADR/epic changes | |

**Lane scope and summary (Phase 0).** `--scope <lane>` is the lane's fast local loop: clippy, test and coverage run on the lane's crates only (coverage gates the lane's logic crates), determinism and ffi-smoke run only for the lane that owns `h3-cli` and `h3-ffi`, bench-check measures the lane's workloads, `deny` is skipped (dependencies are shared), the repository-wide checks run as usual, and `check-lane` runs too. A scoped run always exits 2 (not green); CI runs the full verify. `--summary` prints one line per gate plus the last 25 lines of the first failure and writes everything to `<target>/verify/verify.log` (each gate runs as a child `cargo xtask verify-step <gate>`). Both combine with `--quick`.

**Each code state once** (user, 2026-10-14). A verify runs each test once: `coverage` already runs every test of its packages, so `test` runs only the rest and `determinism` only its release legs (the table above; `cargo xtask determinism` alone still runs all three). And each state of the code is verified once, locally as in CI ("CI" below): `scripts/verify-once.sh engine|clients [--base <ref>] [-- <verify options>]` hashes the files the heavy gates read with CI's fingerprint (the engine: every tracked file but `docs/`, `experiments/`, `clients/`, `*.md`; the clients: the paths `clients.yml` watches) and runs only the light gates (check-rules, check-stubs, check-docs, check-exe-map; the clients none) when a success status `code <fingerprint>` (CI's `verified (ubuntu-24.04)` or its own `verified (sandbox)`, `clients verified (sandbox)`) sits on HEAD, its last ten commits, `master`'s last ten or the heads of the pull requests those merged, or, with `--base origin/master`, when the branch changes none of those files; otherwise the full run (`scripts/sandbox-verify.sh`, `clients/scripts/check.sh`), whose success it records on HEAD when the files are committed and pushed. CI accepts only its own statuses (its runner's bench baselines, `--with-original-data` never). A gate or test that starts reading a left-out path takes it out of the fingerprint in `ci.yml` and the script alike.

Exit codes: 0 = complete and green; 1 = a step failed; 2 = nothing failed but steps were skipped (`--quick`/`--skip`), which is not green. A step may report **N/A** only for a platform limitation (not for a tool that could be installed; a bench-check baseline for another toolchain fails rather than being N/A). Local setup: `cargo install cargo-deny cargo-llvm-cov --locked`, `rustup component add llvm-tools-preview` (or set `LLVM_COV`/`LLVM_PROFDATA` to compatible LLVM tools), a C compiler, and valgrind on Linux.

**CI** (GitHub Actions, `.github/workflows/ci.yml`; Q3). Actions minutes are scarce (private repo on the free plan: 2 000 minutes a month; user, 2026-10-08), so CI verifies each state of the code once:

- **When.** The job `verify (ubuntu-24.04)` runs on every pull request event (opened, pushed, reopened, the label `cross-lane` added or removed), on pushes to `master` and on manual dispatch. On a pull request it first runs `cargo xtask check-lane` (the label passes `--cross-lane`). Pushing a branch without a pull request runs nothing, so a pull request is opened only when its branch is ready for merging (`08-work-process.md`, "Branches, pull requests and syncing"). Closing or merging a pull request cancels its running CI; `master` verifies the merged result (the concurrency group keys pull requests by number: a `closed` event carries the base branch as its ref, and a ref key cancelled `master`'s run until 2026-10-09).
- **Code fingerprint.** A hash of every tracked file except `docs/`, `experiments/`, `clients/` (its own workspace and job, below) and `*.md`, which no heavy gate (fmt, clippy, deny, test, coverage, determinism, ffi-smoke, bench-check) reads. Every successful run records it as the commit status `verified (<os>)` (description `code <fingerprint>`) on the commit it checked, a pull request's head. A run that finds its fingerprint on the pull request's head or previous head, on the heads of the pull requests a push merged, or on the last ten first-parent commits of the target branch skips the full verify and runs only the gates that read the left-out paths (check-rules, check-stubs, check-docs), about a minute: a docs-only pull request, a docs-only push to an open one, the merge of a pull request whose code `master` did not change in between. A pull request whose merge with its base changes only left-out paths runs only those gates even when no run has verified the base's code yet (the base branch's own run does; user, 2026-10-11), and records no fingerprint. Pushes to `master` that change only left-out paths start no run. Manual dispatch and re-running a run always run the full verify. A gate or test that starts reading a left-out path must take it out of both exclusion lists in `ci.yml`.
- **Weekly:** only `cargo deny check advisories` (new RustSec advisories without a code change; licenses, bans and sources change only with the code).
- **When CI cannot run** (user, 2026-10-12). The minutes are limited and run out, and GitHub also stops runs over billing: such a run fails within seconds without starting a step, and its job's annotation says "The job was not started because …" (`gh api repos/ChristopherRotter/homm3/check-runs/<job id>/annotations`). That is not a red build, and **it never blocks work**: for as long as it lasts, nobody relies on CI and the **full local verify stands in for it**. Before a pull request is opened, commit and push the branch merged with the current `master` and run `scripts/verify-once.sh engine --base origin/master -- --with-original-data` (without the option where the files are not staged; `clients` for a branch that changes the client workspace or an engine crate it builds, both when both), plus `cargo xtask check-lane --base origin/master` (`--cross-lane` with the label), and write the result, the commit and the date into the pull request's verify line; the pull request is then mergeable on that record. The script runs the full verify unless the code was verified already, and only the light gates for a pull request that changes only left-out paths ("Each code state once" above). At the start of a session the sandbox's baseline (`scripts/sandbox-setup.sh --lane <lane>`, the same script) replaces `master`'s run, and costs nothing when `master`'s code carries a verified fingerprint. When runs start again, nothing is re-run: `master`'s next run verifies everything merged meanwhile, and a failure there is fixed by whoever sees it.

A `windows-2025` job passed once (2026-10-06) and is disabled for being slow (about 14 minutes against 8; the workflow's header says how to re-enable it), so the Windows verify stays a local run by the user. CI is not enforced: the private repo on a free plan cannot enforce branch rules, so a red `master` must be fixed by whoever sees it. The bench-check baseline is compared on the Linux runner, so a re-baseline must produce counts that hold there (re-baseline from the CI runner if the sandbox and the runner disagree). `--with-original-data` stays local (ADR-0009). Only `master`'s full verifies write the cache; branches restore it. `target/native` is dropped before the verify, since its `target-cpu=native` proc macros from another runner's CPU die with SIGILL (the sandbox script retries the same way).

**The client job** (`.github/workflows/clients.yml`, E17): `clients/scripts/check.sh` (fmt, clippy, tests, deny, 100 % coverage of the client's pure crates) on Linux, for pull requests and `master` pushes that change `clients/`, an engine crate the client builds or a setting it inherits (the path filter; `check.sh` fails when it misses a crate); no fingerprint, so a merged pull request runs once more on `master`; weekly its own `cargo deny check advisories`.

**Coverage lesson (E01):** LLVM counts the implicit `else` of an `if let` and the error path of `?` as regions. Code must not contain branches that cannot execute: express invariants structurally (validated ids → infallible accessors; combinators such as `Option::inspect`/`or_else`), not as defensive dead branches.

## Mechanisms that keep agents from "forgetting" tests

1. **One command, required in CI.** Nothing merges/counts as done unless `verify` is green (in CI, or locally while CI cannot run: "CI" above).
2. **Rule traceability is mechanical.** A rule cannot reach `implemented` without a referencing test, and a marker for a nonexistent rule fails the build.
3. **No silent debt.** `#[ignore]`, `todo!()`, `unimplemented!()` and `// TODO` in non-test logic code fail `check-stubs` unless registered in `docs/stubs.toml` with an owner epic and reason. The registry is visible in the backlog.
4. **Definition of Done** (below) is in the PR template and in every epic file; the end-of-session checklist in `AGENTS.md` repeats it.
5. **Doc-drift check:** a change that touches logic crates but none of `docs/HANDOFF.md`, `docs/lanes/`, `docs/rules/`, `docs/adr/` or the epic file is flagged by `xtask verify` as a warning that becomes an error in CI for changes above a size threshold (heuristic; tune later).
6. **Coverage exclusions are centralized** and diffs to `coverage-exclusions.toml` are surfaced in the gate output.
7. **Bug rule:** every fixed bug gets a regression test first.

## Definition of Done (per task)

- [ ] Behavior implemented in the correct bounded context, in the ubiquitous language
- [ ] Rule catalog updated (new/changed rules with sources); code carries `rule:` markers; tests carry `covers:` markers
- [ ] Unit tests + relevant property/replay tests; 100% coverage for touched core code
- [ ] Quirks considered: any known original deviation has a ledger entry and a flag where applicable
- [ ] Security considered: commands validated against actor and state; hidden info not leaked (`06-security.md`)
- [ ] Performance considered: no allocation regressions in hot paths; benchmarks updated if relevant
- [ ] `cargo xtask verify` green (CI on the pull request, or the full local verify while CI cannot run; locally `verify --scope <lane>`); `check-lane` passes
- [ ] ADR added/updated for any architectural decision; OPEN-QUESTIONS updated for any open one
- [ ] Lane HANDOFF updated; epic file and lane QUEUE updated (BACKLOG status only when an epic starts or ends)

## Test data policy

- Unit tests of rules use **synthetic content packs** (invented creatures with neutral names and numbers chosen to exercise the rule). Tests may also load the committed harvested packs `base`/`ab`/`sod` (Q13), e.g. to check that they compile in every allowed expansion combination. No original files, assets or flavor text in the repo.
- Real-data conformance lives behind `cargo xtask verify --with-original-data [path]` (default path `original-data/install`): it re-harvests from the gitignored `original-data/` directory (copies from the user's installation, which itself is never written to; see AGENTS.md) or a user-provided path and compares with the committed packs. It also runs every crate's `examples/original_data.rs` with the installation path, the crate's own check against the original files (E08 T3: `h3-map` reads every shipped map in `Maps/`); a crate adds one without a shared change.
- Replay fixtures reference synthetic packs or ship self-contained minimal packs.
