# Backlog

Coarse epics in rough priority order. Only epics marked **Refined** have an epic file. Everything else is a sketch that will be refined when it becomes next. Reordering is cheap; update this file when priorities change.

Status: `todo` · `next` · `in-progress` · `partial` · `done`

| ID | Epic | Status | Detail level |
|---|---|---|---|
| E00 | Foundations: vision, architecture, ADRs, process docs | done (2026-09-30) | — |
| E01 | **Walking skeleton**: minimal battle end to end (workspace, gates, kernel, tiny content pack, battle core, commands + validation, observation, sim env, CLI, replay + hash, benchmark) | done (2026-09-30; verified on Linux and Windows) | **Refined** → [epics/E01-walking-skeleton.md](epics/E01-walking-skeleton.md) |
| E02 | Content pipeline: catalogue the user's game archives/data tables, harvest them once into committed `base`/`ab`/`sod` packs (toggleable expansions), pack format v2, synthetic test packs | done (2026-10-01; verified on Linux and Windows incl. `--with-original-data`) | **Refined** → [epics/E02-content-pipeline.md](epics/E02-content-pipeline.md) |
| E03 | Battle core (RoE rules): modifier system, abilities via Tier-3 primitives, obstacles, two-hex units, ranged, morale/luck, damage, war machines | done (2026-10-01; verified on Linux and Windows incl. `--with-original-data`) | **Refined** → [epics/E03-battle-core.md](epics/E03-battle-core.md) |
| E04a | **Heroes in battle (no magic)** — the first third of the former E04 (split by the user, Q27): hero content (classes, heroes, secondary skills, artifacts, specialties, spell data rows), `h3-heroes` and the `BattleHero` seam (ADR-0013, reviewed by the user before work continues), primary skills, damage skills, artifacts and specialties without spells, hero-run Ballista and First Aid Tent, tactics, retreat/surrender, richer outcome, the experiment pipeline (spec → `.h3m`, SoD only) | done (2026-10-05; verified on Linux and Windows incl. `--with-original-data`) | **Refined** → [epics/E04a-heroes-in-battle.md](epics/E04a-heroes-in-battle.md) |
| E04b | **Battle magic** — spell effects and rules, spell book and mana in battle, casting, timed effects on stacks, summons and clones, obstacle spells, resistance, spell-like creature abilities, spell immunities, spellcasting creatures, spell-related skills/artifacts/specialties, hidden-information rules, casting in the sim encodings (needs a factorized action design, flag to E05) | in-progress (increment 1 done: T1, T2a–T2e, batch 2a evaluated; increment 2 (Q46, Q48): T3a, T3b1a, T3b1b, T3b2, T3c1, T3c2, T3c3 (batch 2b compiled), T4a, T4b, T4c done (T4 in three parts, Q52, Q53); increment 3 (Q54, Q56): T5a done, batch 2b folded in, batch 2c compiled, T5b waits for it (Q55), T5c next) | **Refined** (increment 1: T1–T2; increment 2: T3; Q41, Q46) → [epics/E04b-battle-magic.md](epics/E04b-battle-magic.md) |
| E04c | **Sieges and special battlefields** — siege setup derived from the town, walls, gate, moat, towers, Catapult, Ballistics, Earthquake, shooting at and through walls, magic terrains (Holy Ground, Cursed Ground, …), coast and ship battles | todo | sketch (scope fixed by Q27) |
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

*(2026-10-01)* The former E04 is now E04a, E04b and E04c, in that order: magic builds on the hero seam, and the special battlefields' school-based effects build on magic. Comments in code and older docs that say "E04" mean this group; the tasks they refer to are listed in the epic files and in HANDOFF.

1. **E01 first** validates determinism, the command/event model, security shape, performance, gates and the sim API on the smallest possible slice.
2. **E02–E03 before AI**: the battle must be faithful before it is worth training on.
3. **E05–E06 early**: the AI is the motivator; a working battle training loop de-risks the performance and API decisions sooner than a large adventure map would.
4. Adventure/town/game (E07–E10) follow; security projections (E10) build on the skeleton's observation API.
5. Map generator, long-tail objects, clients and server come when their dependencies exist.

## E01 summary (2026-09-30)

Walking skeleton delivered on branch `e01-walking-skeleton`: Cargo workspace (4 `no_std` logic crates, 4 adapters, xtask), `cargo xtask verify` with 12 gates (fmt, clippy, deny, check-arch, check-rules, check-stubs, tests, 100 % coverage, determinism matrix, C ABI smoke, instruction-count benchmark gate, doc-drift warning), deterministic kernel (xoshiro256\*\* streams, FNV-1a state hash), content compiler + TOML packs, a small but sourced battle (13 rules), `BattleEnv` + C ABI + CLI replays with golden fixtures, ≈ 245 k steps/s/core. Decisions: ADR-0009 accepted (GPL-2.0-or-later); amendments to ADR-0002/0003/0004/0005/0006/0007/0011. Details: [HANDOFF](HANDOFF.md), [epic file](epics/E01-walking-skeleton.md).

## E02 summary (2026-10-01)

Content pipeline delivered on branch `e02-content-pipeline`: the installation is catalogued (GOG Complete 4.0, `docs/formats/installation.md`); `h3-formats` reads LOD archives, text tables and the exe's creature table; pack format v2 adds a dependency graph with closures, overrides, resources, factions, creature info and flags; `h3-harvest` with a sourced gap-fill file produces the committed packs `base` (RoE, latest patch), `ab` and `sod`; `cargo xtask verify --with-original-data` re-harvests and compares. Decisions Q8, Q13 (revises Q9), Q14, Q15; ADR-0005/0006/0009 amended. Details: [HANDOFF](HANDOFF.md), [epic file](epics/E02-content-pipeline.md).

## E03 summary (2026-10-01)

Battle core delivered on branch `e03-battle-core` (T1–T9): ADR-0012 (modifiers and abilities, accepted by the user at closure) with `h3_kernel::modifier`; 22 typed creature abilities as pack data; two-hex footprints, obstacles, flying/teleporting, 24 attack positions; ranged combat with distance/melee penalties and area shots; melee abilities (double attack, attack patterns, strike and return, retaliation variants, hate, jousting, defense reduction, death blow, life drain, death stare, regeneration); morale and luck with the wait-morale phase and quirk Q-0003; terrain (native bonus, random obstacles); war machines without heroes; draw on simultaneous wipe-out; loose and tight formations. 45 sourced battle rules (49 with the kernel rules). Seam changes approved by the user: `led_by_hero` (Q17), `BattleResult`/draw (Q24), `formations` (Q25); T8's war-machine fields were covered by Q16. Encodings: action v4, observation v3, C ABI v2, state hash v8, ruleset hash v8. Open for experiments: Q21–Q23; Q26 decided (wiki). Details: [HANDOFF](HANDOFF.md), [epic file](epics/E03-battle-core.md).

## E04 refinement summary (2026-10-01)

Planning session with the user, nothing implemented. The old E04 was split into **E04a** (heroes in battle without magic), **E04b** (battle magic) and **E04c** (sieges and special battlefields) (Q27). Heroes in battle and on the adventure map are distinct entities: a domain crate `h3-heroes` will hold only the hero's lasting details (`HeroDetails`, provisional) and the pure translation to the battle's own `BattleHero` seam type; no plain `Hero` type anywhere. The seam is decided by ADR-0013 in E04a T1, and work stops until the user accepts it. Experiments in the original game become batched and automated (Q28): the engine compiles a spec of staged battles into an `.h3m` for the user's SoD installation (the user can verify SoD only). Quirks are surveyed up front and classified by the user in one sitting; Q6 confirmed. Details: [epic file](epics/E04a-heroes-in-battle.md), [HANDOFF](HANDOFF.md).

## E04a summary (2026-10-05)

Heroes in battle without magic, delivered on branch `e04a-heroes-in-battle` (T1–T12): ADR-0013 (entities, the `BattleHero` seam, the hero's battle rules; accepted by the user, Q30) with the domain crate `h3-heroes` (`HeroDetails`, validation, `derive_battle_hero`, `war_machines`); pack format v3 with hero classes, heroes and specialties, secondary skills, artifacts and spell data rows harvested into `base`/`ab`/`sod`; primary skills, Leadership/Luck and artifact morale and luck, Archery/Offense/Armorer (quirk Q-0005, single-precision rates), hero-run Ballista and First Aid Tent (Q-0004), tactics, retreat, surrender with its price, the richer outcome, stat and trait effects and alignment groups; observation encoding v4 and C ABI v7; the experiment pipeline (spec → `.h3m`) with batches 1a and 1b played by the user in SoD, 21 rules `verified` by observation; the death stare's work bound (Q39, KER-RNG-003) and random obstacles per episode. 16 HER rules, 15 new battle rules and one kernel rule; new quirks Q-0004 to Q-0006 (Q-0003 now reads the real hero). Decisions Q27–Q39. Versions: state hash v17, ruleset hash v13, replay format v13, action encoding v8, observation encoding v4, C ABI v7; about 229–237 k steps/s on the `full` workload with heroes. Closed by an independent review pass (T12c: no defect; robustness tests; five unobserved rules for the next batch, one of them OPEN-QUESTIONS Q40, the tactics hand-over). Details: [epic file](epics/E04a-heroes-in-battle.md), [log](epics/E04a-log.md), [HANDOFF](HANDOFF.md).

## E04b refinement summary (2026-10-05)

Planning session with the user, nothing implemented (Q41). E04b stays one epic, delivered in increments: a coarse outline T1–T11, only the current increment refined, a user checkpoint at the end of each. **Increment 1** is T1 (a just-in-time magic survey and ADR-0014, then a stop for the user's review) and T2 (the casting core with the 13 direct-damage spells, in four checkpoints: content and the hero's magic, casting in battle, the factorized casting action in the sim and the C ABI, area spells with resistance and immunities; then experiment batch 2a for the user). Casting enters the sim with the core, so every later spell is exercised by random rollouts. Details: [epic file](epics/E04b-battle-magic.md), [HANDOFF](HANDOFF.md).

## Stub registry

See `docs/stubs.toml` (empty: no registered stubs after E01).

## Discovered work inbox

Cross-cutting items only (tooling, gates, kernel). Lane work goes into the lane's `docs/lanes/<lane>/QUEUE.md`, "Inbox" (Phase 0, 2026-10-07: the earlier lane items moved there; done items were dropped, git history keeps them). Agents: one line each, with date and epic ID; this file is shared, so add items in a `shared/` pull request or note them in the lane HANDOFF for the next one.

- 2026-09-30 (E01): turn the doc-drift warning into an error in CI for large changes (the Linux + Windows CI running `cargo xtask verify` exists since 2026-10-06, Q3).
- 2026-09-30 (E01): WASM build check of the logic crates (`wasm32` target could not be installed in the E01 environment); add a `wasm` verify step, ideally running the golden replays under a WASI runtime.
- 2026-09-30 (E01): machine check for minimal `pub` surface of seam/contract crates (ADR-0005 §4 is not enforced yet).
- 2026-09-30 (E01): mutation testing (`cargo-mutants`) and fuzzing (`cargo-fuzz`) set-up; first fuzz targets come with E02's binary parsers.
- 2026-09-30 (E01): faster state hash if search needs per-node hashing (FNV-1a ≈ 1 ns/byte; bump `HASH_VERSION`).
