# E01 — Walking skeleton

Status: next · Refined: 2026-09-30

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
