# Backlog

Coarse epics in rough priority order. Only epics marked **Refined** have an epic file. Everything else is a sketch that will be refined when it becomes next. Reordering is cheap; update this file when priorities change.

Status: `todo` · `next` · `in-progress` · `partial` · `done`

| ID | Epic | Status | Detail level |
|---|---|---|---|
| E00 | Foundations: vision, architecture, ADRs, process docs | done (2026-09-30) | — |
| E01 | **Walking skeleton**: minimal battle end to end (workspace, gates, kernel, tiny content pack, battle core, commands + validation, observation, sim env, CLI, replay + hash, benchmark) | partial — all done and green on Linux; only the Windows run of `cargo xtask verify` is pending (user's machine, no CI per Q3) | **Refined** → [epics/E01-walking-skeleton.md](epics/E01-walking-skeleton.md) |
| E02 | Content pipeline: catalogue the user's game archives/data tables, import into content packs, pack format v2, synthetic test packs | next (needs Q8/Q9 answers and the user's files) | **Refined** → [epics/E02-content-pipeline.md](epics/E02-content-pipeline.md) |
| E03 | Battle core (RoE rules): modifier system, abilities via Tier-3 primitives, obstacles, two-hex units, ranged, morale/luck, damage, war machines | todo | **Refined** → [epics/E03-battle-core.md](epics/E03-battle-core.md) |
| E04 | Heroes & army, artifacts, skills, specialties, spell book; battle spells (magic) | todo | sketch |
| E05 | Simulation env v1 + baselines: batch API, forkable envs, masks, random + heuristic bots, FFI, Python smoke binding | todo | sketch |
| E06 | Battle AI v1 + local training pipeline v0 (evaluation gate, opponent ladder, experiment records) | todo | sketch |
| E07 | Formats: map (.h3m) reader, scenario loading, validation/fuzzing | todo | sketch |
| E08 | Adventure map core: tiles, movement, pathing, day/week/month, resources, a first subset of objects, fog of war | todo | sketch |
| E09 | Town & economy: buildings, construction, recruitment, growth | todo | sketch |
| E10 | Game orchestration: players/teams, turns, battle integration via contracts, victory/defeat, projections/visibility (security), scenario setup | todo | sketch |
| E11 | AB and SoD completeness (content + rules), expanded from E02–E04 as needed | todo | sketch |
| E12 | Map objects long tail | todo | sketch |
| E13 | Adventure AI (hierarchical) + training on generated maps | todo | sketch |
| E14 | Map generator (seeded from user's existing project; objective-driven: zone balance, few teleports, …) | todo | sketch |
| E15 | Acceptance tests (BDD) suite — grows alongside features | todo | parallel/ongoing |
| E16 | Server, protocol, auth hooks, rate limiting, fuzzing/hardening | todo | sketch |
| E17 | Clients & asset pipeline (web/WASM, desktop) — outside the core | todo | sketch |
| E18 | Extension API spike (WoG/HotA-shaped requirements), modding tooling | todo | sketch |
| E19 | Original save-file import (low priority) | todo | future |
| E20 | Campaigns | todo | future |

## Ordering rationale

1. **E01 first** validates determinism, the command/event model, security shape, performance, gates and the sim API on the smallest possible slice.
2. **E02–E03 before AI**: the battle must be faithful before it is worth training on.
3. **E05–E06 early**: the AI is the motivator; a working battle training loop de-risks the performance and API decisions sooner than a large adventure map would.
4. Adventure/town/game (E07–E10) follow; security projections (E10) build on the skeleton's observation API.
5. Map generator, long-tail objects, clients and server come when their dependencies exist.

## E01 summary (2026-09-30)

Walking skeleton delivered on branch `e01-walking-skeleton`: Cargo workspace (4 `no_std` logic crates, 4 adapters, xtask), `cargo xtask verify` with 12 gates (fmt, clippy, deny, check-arch, check-rules, check-stubs, tests, 100 % coverage, determinism matrix, C ABI smoke, instruction-count benchmark gate, doc-drift warning), deterministic kernel (xoshiro256\*\* streams, FNV-1a state hash), content compiler + TOML packs, a small but sourced battle (13 rules), `BattleEnv` + C ABI + CLI replays with golden fixtures, ≈ 245 k steps/s/core. Decisions: ADR-0009 accepted (GPL-2.0-or-later); amendments to ADR-0002/0003/0004/0005/0006/0007/0011. Details: [HANDOFF](HANDOFF.md), [epic file](epics/E01-walking-skeleton.md).

## Stub registry

See `docs/stubs.toml` (empty: no registered stubs after E01).

## Discovered work inbox

(Agents: add coarse items here, one line each, with date and your epic ID. Triage into the table at the next planning pass.)

- 2026-09-30 (E01): CI configuration once a host is chosen (Q3): Linux + Windows matrix running `cargo xtask verify`; turn the doc-drift warning into an error for large changes.
- 2026-09-30 (E01): WASM build check of the logic crates (`wasm32` target could not be installed in the E01 environment); add a `wasm` verify step, ideally running the golden replays under a WASI runtime.
- 2026-09-30 (E01): machine check for minimal `pub` surface of seam/contract crates (ADR-0005 §4 is not enforced yet).
- 2026-09-30 (E01): `Trace` sink for rule-level explanations ("explain this damage number", 02-architecture §10).
- 2026-09-30 (E01): mutation testing (`cargo-mutants`) and fuzzing (`cargo-fuzz`) set-up; first fuzz targets come with E02's binary parsers.
- 2026-09-30 (E01): content-pack override rules and profile-level Tier-2 overrides (deferred; E02 T4).
- 2026-09-30 (E01): faster state hash if search needs per-node hashing (FNV-1a ≈ 1 ns/byte; bump `HASH_VERSION`).
- 2026-09-30 (E01): bump the pinned toolchain (1.95.0) to the current stable when the build environment allows; re-baseline bench-check.
- 2026-09-30 (E01): batch stepping over many environments in `h3-sim`/C ABI, and loading packs/scenarios through the ABI (E05).
