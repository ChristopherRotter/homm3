# 05 — Testing strategy

## Levels

1. **Unit tests** — every core crate: 100% line **and** region coverage (LLVM source-based coverage, `cargo llvm-cov`). Exclusions only via the single reviewed file `coverage-exclusions.toml` (graphics/sound/OS glue; never domain logic). Adding an exclusion requires a justification line and is called out in HANDOFF.
2. **Property-based tests** (`proptest`) — invariants, see below.
3. **Determinism replay tests** — recorded games (seed + commands) replay to identical events and state hashes; golden hashes compared per platform in CI (Linux, Windows; WASM once it exists), in debug and release, with and without `target-cpu=native` (ADR-0003). Any rule using floats or transcendentals adds a fixture exercising it, plus test vectors for `kernel::num` functions.
4. **Conformance tests** — worked examples from sources; oracle comparisons (ADR-0009); original-data tests (local only, `--with-original-data`).
5. **Statistical tests** — distributions of random mechanics across many seeds.
6. **Fuzzing** (`cargo-fuzz`) — all parsers of untrusted input (map, archive, save, network/command decoding) and the command validator.
7. **Mutation testing** (`cargo-mutants`) — periodic (not per-commit) on core crates; surviving mutants are triaged; a threshold is calibrated after the walking skeleton.
8. **Benchmarks** — `criterion` (local wall time) and an instruction-count tool for deterministic CI regression checks.
9. **Acceptance tests (BDD)** — later; Gherkin features under `features/`, each scenario referencing rule IDs. Not specified further now.

## Invariants every context must provide property tests for

- **Atomicity:** if `apply` returns an error, state (and its hash) is unchanged.
- **Legality consistency:** every command listed by `legal_commands` is accepted by `apply`; commands sampled from outside the set are rejected.
- **Ownership isolation:** a random command sequence issued by actor A never changes anything owned by actor B except through explicitly modelled interactions (battle, trade, …).
- **Conservation:** resources/creatures are conserved except through modelled sources and sinks.
- **Observation hygiene:** `observe(state, viewer)` is a function of only what `viewer` may know (two states differing only in hidden info yield equal observations).
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

## Mechanisms that keep agents from "forgetting" tests

1. **One command, required in CI.** Nothing merges/counts as done unless `verify` is green.
2. **Rule traceability is mechanical.** A rule cannot reach `implemented` without a referencing test, and a marker for a nonexistent rule fails the build.
3. **No silent debt.** `#[ignore]`, `todo!()`, `unimplemented!()` and `// TODO` in non-test logic code fail `check-stubs` unless registered in `docs/stubs.toml` with an owner epic and reason. The registry is visible in the backlog.
4. **Definition of Done** (below) is in the PR template and in every epic file; the end-of-session checklist in `AGENTS.md` repeats it.
5. **Doc-drift check:** a change that touches logic crates but none of `docs/HANDOFF.md`, `docs/rules/`, `docs/adr/` or the epic file is flagged by `xtask verify` as a warning that becomes an error in CI for changes above a size threshold (heuristic; tune later).
6. **Coverage exclusions are centralized** and diffs to `coverage-exclusions.toml` are surfaced in the gate output.
7. **Bug rule:** every fixed bug gets a regression test first.

## Definition of Done (per task)

- [ ] Behavior implemented in the correct bounded context, in the ubiquitous language
- [ ] Rule catalog updated (new/changed rules with sources); code carries `rule:` markers; tests carry `covers:` markers
- [ ] Unit tests + relevant property/replay tests; 100% coverage for touched core code
- [ ] Quirks considered: any known original deviation has a ledger entry and a flag where applicable
- [ ] Security considered: commands validated against actor and state; hidden info not leaked (`06-security.md`)
- [ ] Performance considered: no allocation regressions in hot paths; benchmarks updated if relevant
- [ ] `cargo xtask verify` green
- [ ] ADR added/updated for any architectural decision; OPEN-QUESTIONS updated for any open one
- [ ] HANDOFF updated; epic/backlog status updated

## Test data policy

- CI uses **synthetic content packs** (invented creatures/spells with neutral names). No original assets or text in the repo.
- Real-data conformance lives behind `--with-original-data` and reads from a user-provided path.
- Replay fixtures reference synthetic packs or ship self-contained minimal packs.
