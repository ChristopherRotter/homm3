# HANDOFF

Living state. The last agent updates this at the end of every session; the next agent reads it first. Newest entry on top; keep old entries short (or prune) once superseded.

---

## 2026-09-30 — E01 walking skeleton (one session, with the user present at the start)

**State of the repo:** branch `e01-walking-skeleton` (one commit per task plus a review-fix commit, on top of `master`), all E01 tasks T1–T10 done. An independent review pass at the end found no rule or validation bugs but several gate loopholes (test-code classification, header check by substring, skipped steps exiting 0, fail-open coverage/bench checks); all fixed in the last commit. `cargo xtask verify` is **green on x86_64 Linux** (all 12 steps, none skipped). The Windows run is the only open E01 item (see "Next").

**What was done:**
- Workspace: `h3-kernel`, `h3-content`, `h3-contracts`, `h3-battle` (logic, `no_std` + `alloc`, 100 % line+region coverage); `h3-pack`, `h3-sim`, `h3-ffi`, `h3-cli` (adapters); `xtask`. Graph and purity rules in `arch.toml`.
- Gates: `cargo xtask verify` = fmt, clippy, deny, check-arch, check-rules, check-stubs, test, coverage, determinism, ffi-smoke, bench-check, doc-drift (table in `docs/05-testing.md`). Single steps: `cargo xtask <step>`; `cargo xtask help`.
- Kernel: xoshiro256\*\* + SplitMix64 streams, FNV-1a canonical hashing, `Real` float surface, `FixedVec`, `Handle`, total-order sort helper.
- Battle: 13 sourced rules (`docs/rules/catalog/battle.toml`): geometry, placement, turn order with wait phase and speed ties, Defend, walking, melee, damage (roll, attack/defense modifier, top-creature-first), retaliation, end of battle. Commands validated before mutation; `legal_commands`; `observe`/`project`; events with visibility; property tests for every mandatory invariant.
- Sim/FFI/CLI: `BattleEnv` (449-action encoding, 612-float observation, truncation, rewards, fork), random legal agent, replay model; C ABI v1 with header and C smoke program; `h3 run --record`, `h3 verify`, `h3 bench`; golden replays in `fixtures/replays/`.
- Performance: ≈ 245 k random-legal steps/s/core, 944-byte state, ≈ 90 ns clone; instruction-count baseline in `benches/baseline.toml` (+2 %).

**Decisions taken (user):** Q1 → GPL-2.0-or-later (ADR-0009 accepted, `LICENSE`); Q3 → no CI config yet, feature branch per epic. **Decisions taken (agent, recorded as ADR amendments):** toolchain 1.95.0; `no_std` logic crates; `h3-pack` adapter crate; PRNG/hash algorithms; release overflow checks; geometry seam as a concrete type; step limits in the sim, not the domain; `design` source kind; valgrind-based benchmark gate; dev-dependencies excluded from the license check.

**Next:**
1. **User:** fast-forward `master` to `e01-walking-skeleton` if happy, and run `cargo xtask verify` once on Windows (needs `cargo install cargo-deny cargo-llvm-cov --locked` and `rustup component add llvm-tools-preview`; ffi-smoke and bench-check report N/A there). That closes E01 exit criteria 1–2.
2. **E02** (content pipeline) needs the user's installation and answers to Q8/Q9 (and Q11 for a parameter value). E03 is refined too.

**Open questions added:** Q11 (attack-bonus cap 300 % vs 400 %), Q12 (large-stack damage roll: wiki vs VCMI).

**Traps / notes for the next agent:**
- **Coverage is strict about dead branches:** LLVM counts the implicit `else` of `if let` and the error arm of `?` as regions. After validation, use infallible accessors (`BattleState::at`/`at_mut`) or combinators; do not add defensive branches that cannot run.
- `cargo fmt` reflows code: when patching files with scripts, match against the formatted text (and check that the patch applied).
- Coverage tooling: `cargo llvm-cov` needs `llvm-tools-preview` or `LLVM_COV`/`LLVM_PROFDATA` pointing to LLVM tools that read rustc's profile format (in the E01 sandbox: Ubuntu's LLVM 20 worked with rustc 1.95's LLVM 22).
- Golden replays and `bench-check` baselines change whenever behavior, the synthetic pack, an algorithm/encoding version or (for instruction counts) the toolchain changes. Regenerate deliberately (`fixtures/README.md`, `cargo xtask bench-check --update`) and say why in the commit.
- Wiki/VCMI conflicts: the source hierarchy ranks VCMI above wikis, but E01 followed the wiki where it describes the original explicitly (Q11, Q12). Get the user's decision or an observation fixture before the base pack.
- The battle has no round limit (faithful). Random agents mostly walk around; `h3 bench`/rollouts use step limits from the sim layer.
- `Claude outputs/AGENTS.md` in the repo is an older copy of `AGENTS.md` (not updated by this session).

## 2026-09-30 — E00 foundations (planning session with the user) — superseded by the E01 entry above

**State of the repo (then):** documentation only.

**What was done:**
- Defined vision, architecture principles, bounded contexts, source-of-truth and provenance approach, quirks/profiles, testing gates, security model, simulation/AI strategy, work process.
- Recorded ADR-0001 … ADR-0011.
- Wrote the backlog (coarse) and refined E01 (walking skeleton).

**What is next:** Start E01 (`docs/epics/E01-walking-skeleton.md`), task T1 (workspace scaffold + `xtask verify` + CI skeleton).

**Waiting on the user (see OPEN-QUESTIONS):**
- Q1 license choice and VCMI policy — interim: do **not** copy/translate VCMI code. No `LICENSE` file has been added yet; add one once decided (E01 T1 may proceed without it, but note it).
- Q2 project/crate naming (working prefix `h3-`).
- Q8 which game version is the "original" baseline; Q9 whether base content tables may be committed.
- Others are lower priority and do not block E01.

**Traps / notes for the next agent:**
- Do not assume original data file names or layouts from memory; catalogue them from the files the user supplies (E02).
- Keep the skeleton *minimal*: its purpose is to exercise the architecture, not to be a nice battle system. Resist adding spells, heroes, or abilities.
- All tests in CI must run on synthetic content packs (neutral invented names).
- Determinism/numeric policy in ADR-0003 is strict: integers by default, floats only via `kernel::num` (no platform transcendentals, no fast-math), no `HashMap`/`HashSet` in logic crates, every sort has a unique tie-breaker.
- The Rust toolchain/versions are not pinned yet; pin in `rust-toolchain.toml` during T1 and record the MSRV policy in ADR-0002 if it needs refining.
