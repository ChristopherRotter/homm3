# E01 — Walking skeleton

Status: done on Linux (2026-09-30); the Windows run of `cargo xtask verify` is pending (see "Status" at the end) · Refined: 2026-09-30

## Goal

A minimal but **complete vertical slice** that exercises every upfront architectural decision: a tiny battle runs from config to command validation to events to observation to a simulation env, deterministically, fast, with all quality gates green. The slice is deliberately small and ugly-in-content; it is beautiful in *structure*.

## Why

To find out early whether the determinism model, command/event model, security shape (actor + validation + observation), state layout (cheap fork), compiled ruleset, rule traceability, gates and performance targets actually work together — before investing in breadth.

## Scope

- Cargo workspace, toolchain pin, lint policy, `xtask` (verify, check-arch, check-rules, check-stubs, coverage), CI skeleton.
- `kernel`: ids/handles, fixed-width numerics, in-repo PRNG with stream splitting, stable state hash.
- `content`: minimal definition model + TOML loader (adapter side) + `Ruleset` compiler (pure) for **synthetic** creature definitions (3 invented creatures) and a synthetic profile.
- `battle`: small hex (or simplified grid behind a geometry trait) battlefield, stacks, initiative order, commands `Move`, `Attack` (melee), `Wait`, `Defend`; base damage rule with one or two modifiers; retaliation; end-of-battle outcome. Two players/actors.
- `contracts`: `BattleSetup`, `BattleOutcome` (minimal).
- `apply` with actor validation, atomicity; `legal_commands` mask; `observe(viewer)` (trivial projection, but the API and its property test exist); event visibility scope field.
- `sim`: `BattleEnv` (reset/step/mask/fork/state_hash) + random-agent rollout; `ffi` minimal C ABI smoke (create env, step, destroy).
- `cli`: run a random-vs-random battle, write a replay (seed + commands + hashes), verify a replay.
- Rule catalog for the few rules involved, with sources labelled; quirks ledger has the structure (no entries required).
- Benchmarks: steps/s for random rollouts, clone cost.

## Non-goals

Heroes, spells, artifacts, abilities beyond the trivial, obstacles, war machines, real content import, adventure map, AI beyond random/scripted, real hex-rule fidelity. No original data; nothing that needs the game files.

## Exit criteria

1. `cargo xtask verify` green on Linux and Windows CI; 100% line+region coverage on core crates.
2. Replay fixtures produce identical state hashes on both platforms.
3. Property tests exist and pass for: atomicity, legality consistency, ownership isolation, observation hygiene, clone equivalence, determinism.
4. Architecture check enforces the dependency graph and forbidden APIs; a deliberately violating commit is demonstrably rejected (proved by a test in `xtask`).
5. Rule traceability works end to end (a rule without test fails; a marker for an unknown rule fails).
6. Benchmarks recorded; provisional performance targets in `02-architecture.md` replaced by measured numbers and initial regression thresholds.
7. ADR-0002/0003/0004/0011 updated with lessons (e.g., chosen PRNG, hashing, geometry abstraction, bench tooling).
8. HANDOFF, BACKLOG, stub registry updated; E02/E03 refined.

## Tasks (each sized for one agent session; reorder freely)

- **T1 Scaffold & gates.** Workspace, toolchain pin, lint policy (ADR-0003: float confinement to `kernel::num`, disallowed transcendentals, `HashMap`/`HashSet`, unwrap in logic crates), CI matrix for determinism (debug/release, with/without `target-cpu=native`), `xtask verify` skeleton running fmt/clippy/deny/test, CI config, PR template with DoD. Empty crates with `arch.toml`. LICENSE placeholder pending Q1.
- **T2 Arch & rule checks.** `check-arch`, `check-rules`, `check-stubs`, `docs/rules/catalog/` parsing, `docs/stubs.toml`.
- **T3 Kernel.** IDs/handles, PRNG (choose algorithm; document; test vectors), stream splitting, canonical state hashing, checked-arithmetic helpers, the `kernel::num` module skeleton (float surface with rounding helpers; transcendentals only when a rule needs them, YAGNI), total-order sort helper.
- **T4 Content & ruleset.** Definition model, synthetic pack loader (TOML adapter), `Ruleset` compile, dense handles, profile/quirk flag plumbing (even if no quirks yet).
- **T5 Battle state & geometry.** Field, stacks, placement, initiative order; fixed-capacity structures; clone/fork cost measured.
- **T6 Commands & rules.** `Move`, `Attack`, `Wait`, `Defend`; validation (actor, turn, ownership, bounds); damage rule(s) with catalog entries; retaliation; outcome. Events with visibility scope.
- **T7 Legal commands & observation.** Mask generation consistent with `apply`; `observe`; property tests.
- **T8 Sim & FFI & CLI.** `BattleEnv`, random rollout, C ABI smoke, replay write/verify, cross-platform hash fixtures.
- **T9 Benchmarks & calibration.** criterion + instruction-count tool; measured targets; write results into architecture doc; update ADRs.
- **T10 Coverage gate & closure.** Coverage with exclusion file, final verify, handoff, refine E02/E03.

## Risks

- Over-building the battle: keep the content trivial; the skeleton tests structure, not fidelity.
- PRNG/hash choices locked in too casually: they become part of replay compatibility. Version them (replay header carries algorithm + version ids).
- Hex geometry may bias the state layout; isolate behind a geometry interface so the real hex rules (E03) can replace it.

## Questions for the user during this epic

Only if needed; Q1/Q2/Q3 (license/name/CI host) are the likely ones.

## Status (2026-09-30, end of the E01 session)

All tasks were implemented in one session on branch `e01-walking-skeleton`.

| Task | State | Notes |
|---|---|---|
| T1 Scaffold & gates | done | toolchain 1.95.0; GPL-2.0-or-later `LICENSE`; no CI config (Q3); PR checklist in `docs/pull_request_template.md` |
| T2 Arch & rule checks | done | `check-arch`, `check-rules`, `check-stubs` with rejection tests |
| T3 Kernel | done | xoshiro256\*\*/SplitMix64 streams, FNV-1a state hash, `Real`, `FixedVec`, `Handle`, `order`, `arith` |
| T4 Content & ruleset | done | `h3-content` compiler, `h3-pack` TOML adapter, synthetic pack + 3 scenarios |
| T5 Battle state & geometry | done | 17×11 hex field, 944-byte state, ≈ 90 ns clone |
| T6 Commands & rules | done | 13 battle rules with sources (`docs/rules/catalog/battle.toml`) |
| T7 Legal commands & observation | done | property tests for all mandatory invariants |
| T8 Sim & FFI & CLI | done | `BattleEnv`, C ABI v1 + C smoke, `h3 run/verify/bench`, 3 golden replays |
| T9 Benchmarks & calibration | done | ≈ 245 k steps/s/core; instruction-count gate; numbers in `02-architecture.md` §8 |
| T10 Coverage gate & closure | done | 100 % line + region on logic crates; docs, ADR amendments, E02/E03 refined |

Exit criteria:

1. `cargo xtask verify` green on Linux, 100 % line+region coverage — **met**. Windows: **not run** (no CI per Q3, and this session had no Windows toolchain). To close: run `cargo xtask verify` on the user's Windows machine (ffi-smoke and bench-check report N/A there).
2. Replay fixtures identical across platforms — **met for x86_64 Linux** in debug, release and release + native. Windows run pending (same command as 1).
3. Property tests (atomicity, legality consistency, ownership isolation, observation hygiene, clone equivalence, determinism, plus conservation) — **met**.
4. A violating change is rejected, proved by xtask tests — **met** (`arch::tests::rejects_*`).
5. Rule traceability end to end — **met** (`rules::tests::implemented_rule_without_test_fails`, `unknown_markers_fail`).
6. Benchmarks recorded, provisional targets replaced — **met**.
7. ADRs updated — **met** (amendments to ADR-0002/0003/0004/0005/0006/0007/0011; ADR-0009 accepted).
8. HANDOFF, BACKLOG, stub registry, E02/E03 refined — **met**.
