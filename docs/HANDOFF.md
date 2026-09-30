# HANDOFF

Living state. The last agent updates this at the end of every session; the next agent reads it first. Newest entry on top; keep old entries short (or prune) once superseded.

---

## 2026-09-30 — E02 in progress (session 1)

**State:** E02 T1 done; E02 re-planned after user decisions (see the epic file). Work happens on branch **`e02-content-pipeline`** (from `master` at `8e913f6`, which the user had already fast-forwarded to E01).

**Decisions (user, this session):**
- Q8: installation = **GOG Complete 4.0** (evidence: `docs/formats/installation.md`). Support **RoE (base), AB and SoD**; AB and SoD are toggleable expansions in a dependency graph (SoD → AB → base). Packs `base`, `ab`, `sod`.
- Q13 (revises Q9): harvest the original tables **once** into a clean config and **commit** it (gameplay data + names; no flavor text/assets/original files). Harvest tooling lives in the repo; gaps from labelled wiki/VCMI sources.
- Q14: keep **TOML**.
- Recorded in ADR-0006 (E02 amendment: dependency graph, enabled-pack config, overrides) and ADR-0009 (amendment 2).

**Key findings (T1):** LOD layout verified; `H3bitmap.lod` = SoD tables, `H3ab_bmp.lod` = AB-era tables (same 71 TXT names, 23 differ; the diff is the AB→SoD change set). Level/town/flags are only in `Heroes3.exe` (static 150 × 116-byte table at file offset `0x2703B8`); the agent decided to read it (original data ranks above wiki/VCMI) — noted in Q13, not an explicit user decision. RoE-era values and AB membership are not in the installation → gap-fill file with sources (T4).

**Environment notes (cloud sandbox):** `static.rust-lang.org` is blocked by the egress proxy, so `rustup` cannot install 1.95.0; the sandbox's `stable` *is* 1.95.0 and was symlinked as `~/.rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu`. crates.io works. No `llvm-tools-preview`; Ubuntu's `llvm-cov` is LLVM 18.

**Next:** T2 `h3-formats` (LOD + text table + exe table readers), then T3 pack v2, T4 harvester, T5 conformance.

**In progress (checkpoint 2026-10-01, user-requested stop):**
- Done and committed on `e02-content-pipeline`: T1 (catalogue + decisions) and T2 (`crates/h3-formats`). This branch is green for fmt, clippy, deny, check-arch, check-rules, check-stubs, tests, determinism, ffi-smoke and bench-check in the cloud sandbox; **coverage was not run** there (no LLVM tools matching rustc 1.95's LLVM 22; `apt-get install llvm-20` was started and `LLVM_COV`/`LLVM_PROFDATA` can point to `/usr/lib/llvm-20/bin/*`, as in E01). `h3-formats` is an adapter crate, so the 100 % coverage gate does not apply to it; run `cargo xtask verify` on Windows once to confirm.
- **T3 half-done on branch `e02-t3-wip`** (one commit on top of this branch, **does not compile**): `h3-content` pack model v2 (`pack.rs`: resources, factions, `CreatureInfoDef`, `CreatureOverride`/`CreaturePatch`, `ContentPack::new`), `flags.rs` (closed `CreatureFlag` set, bitset), `ruleset.rs` (topological load order with pack-id tie-break, missing dependency / cycle errors, dependency closures, resources/factions/creature info, overrides restricted to the closure, references restricted to the closure, hash encoding v2).
- **To finish T3:** declare `mod flags` and export the new types in `h3-content/src/lib.rs`; add `CompileError` variants `DependencyCycle(String)`, `ReferenceOutsideDependencies { key, pack }`, `UnknownReference { kind, key }`, `UnknownFlag(String)`, `TooManyEntries(&'static str)` (+ Display; `MissingDependency` text becomes "…which is not enabled"); fix the struct literals (`CreatureDef { info: CreatureInfoDef::default(), .. }`, `ContentPack::new(..)`) in `h3-content` tests, `h3-battle/src/tests/support.rs`, `h3-pack`; tests for every new compiler rule (100 % coverage — watch for dead branches, e.g. the `get_mut(..).zip(..)` in the override loop and the `unwrap_or_default` in the cycle error); `h3-pack` DTOs (`[[resource]]`, `[[faction]]`, new creature fields with defaults, `[[creature_override]]` with `target`, refs without `:` resolve to the own pack); extend `packs/synthetic` with the new fields; update `GOLDEN_BASE_HASH` and regenerate the golden replays (`fixtures/README.md`; the ruleset hash encoding changed to v2); document format v2 in `packs/README.md`.
- Then T4 (`h3-harvest`, gap-fill file with labelled sources, committed `packs/{base,ab,sod}`) and T5 (conformance via `--with-original-data`).
- Open design points noted for T4: exe flag bit `0x800` (ids 35, 74, 75) is unidentified and will not be harvested; RoE-era values and AB membership need the gap-fill file.
---

## 2026-09-30 — E01 closed (final commit)

**State:** E01 is **done**. The user ran `cargo xtask verify` on Windows: green (ffi-smoke and bench-check N/A by design). Branch `e01-walking-skeleton` is ready for the user to fast-forward into `master`.

**Decisions recorded in this commit (user):**
- Q11: attack-bonus cap **+300 %** (BAT-DMG-002). Q12: large-stack damage roll per the **wiki** (BAT-DMG-001; for exactly 10 creatures both formulations coincide, test added).
- Q9: **import-only**. Copies of original files go to the repo's `original-data/` (gitignored; `check-arch` fails if anything there is tracked). ADR-0009 amended.
- Q8 (partly): the installation is available (on the user's machine at `D:\Heroes3\HoMM 3 Complete`, connected to sessions as an extra folder; it was not yet connected to the session that made this commit). Its version is identified in E02 T1.

**Rule for every agent from now on:** the installation folder is **read-only** (the user cannot enforce it). Never edit, create, move or delete anything in it, never run tools that write there, never request delete permission for it. Copy into `original-data/` and work on the copies (AGENTS.md, ADR-0009).

**Next:** E02 (content pipeline), starting with T1 (catalogue the installation, identify the version).

**Process addition (after closure):** `AGENTS.md` now has "Interruptions and resuming": sync every commit to the user's repo immediately, keep HANDOFF resumable, checkpoint when the user announces a stop, and rebuild from the user's repo when resuming (the user amended the closing commit in their repo; agents must build on the user's history, not on a stale sandbox copy).

**Trap:** on the user's machine, running `git status` from the agent's device shell without delete permission leaves a stale `.git/index.lock` (the shell may create but not delete files in connected folders). Prefer doing git work in the cloud workspace and syncing with a bundle; if you must run git on the device, get delete permission for the repo folder first.

---

## 2026-09-30 — E01 walking skeleton (one session, with the user present at the start)

**State of the repo:** branch `e01-walking-skeleton` (one commit per task plus a review-fix commit, on top of `master`), all E01 tasks T1–T10 done. An independent review pass at the end found no rule or validation bugs but several gate loopholes (test-code classification, header check by substring, skipped steps exiting 0, fail-open coverage/bench checks); all fixed in the last commit. `cargo xtask verify` is **green on x86_64 Linux** (all 12 steps, none skipped); the Windows run was done by the user afterwards (see the entry above).

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
2. **E02** (content pipeline) needs the user's installation and answers to Q8/Q9 (and Q11 for a parameter value). E03 is refined too. *(Superseded: see "E01 closed" above.)*

**Open questions added:** Q11 (attack-bonus cap 300 % vs 400 %), Q12 (large-stack damage roll: wiki vs VCMI) — both answered at closure.

**Traps / notes for the next agent:**
- **Coverage is strict about dead branches:** LLVM counts the implicit `else` of `if let` and the error arm of `?` as regions. After validation, use infallible accessors (`BattleState::at`/`at_mut`) or combinators; do not add defensive branches that cannot run.
- `cargo fmt` reflows code: when patching files with scripts, match against the formatted text (and check that the patch applied).
- Coverage tooling: `cargo llvm-cov` needs `llvm-tools-preview` or `LLVM_COV`/`LLVM_PROFDATA` pointing to LLVM tools that read rustc's profile format (in the E01 sandbox: Ubuntu's LLVM 20 worked with rustc 1.95's LLVM 22).
- Golden replays and `bench-check` baselines change whenever behavior, the synthetic pack, an algorithm/encoding version or (for instruction counts) the toolchain changes. Regenerate deliberately (`fixtures/README.md`, `cargo xtask bench-check --update`) and say why in the commit.
- Wiki/VCMI conflicts: the source hierarchy ranks VCMI above wikis; for real gameplay conflicts ask the user or record an observation fixture (Q11 and Q12 were settled that way: the user chose the wiki).
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
