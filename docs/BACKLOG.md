# Backlog

Coarse epics in rough priority order. Only epics marked **Refined** have an epic file. Everything else is a sketch that will be refined when it becomes next. Reordering is cheap; update this file when priorities change.

Status: `todo` · `next` · `in-progress` · `partial` · `done`

| ID | Epic | Status | Detail level |
|---|---|---|---|
| E00 | Foundations: vision, architecture, ADRs, process docs | done (2026-09-30) | — |
| E01 | **Walking skeleton**: minimal battle end to end (workspace, gates, kernel, tiny content pack, battle core, commands + validation, observation, sim env, CLI, replay + hash, benchmark) | done (2026-09-30; verified on Linux and Windows) | **Refined** → [epics/E01-walking-skeleton.md](epics/E01-walking-skeleton.md) |
| E02 | Content pipeline: catalogue the user's game archives/data tables, harvest them once into committed `base`/`ab`/`sod` packs (toggleable expansions), pack format v2, synthetic test packs | done (2026-10-01; verified on Linux and Windows incl. `--with-original-data`) | **Refined** → [epics/E02-content-pipeline.md](epics/E02-content-pipeline.md) |
| E03 | Battle core (RoE rules): modifier system, abilities via Tier-3 primitives, obstacles, two-hex units, ranged, morale/luck, damage, war machines | done (2026-10-01; verified on Linux incl. `--with-original-data`; Windows run by the user pending) | **Refined** → [epics/E03-battle-core.md](epics/E03-battle-core.md) |
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

## E02 summary (2026-10-01)

Content pipeline delivered on branch `e02-content-pipeline`: the installation is catalogued (GOG Complete 4.0, `docs/formats/installation.md`); `h3-formats` reads LOD archives, text tables and the exe's creature table; pack format v2 adds a dependency graph with closures, overrides, resources, factions, creature info and flags; `h3-harvest` with a sourced gap-fill file produces the committed packs `base` (RoE, latest patch), `ab` and `sod`; `cargo xtask verify --with-original-data` re-harvests and compares. Decisions Q8, Q13 (revises Q9), Q14, Q15; ADR-0005/0006/0009 amended. Details: [HANDOFF](HANDOFF.md), [epic file](epics/E02-content-pipeline.md).

## E03 summary (2026-10-01)

Battle core delivered on branch `e03-battle-core` (T1–T9): ADR-0012 (modifiers and abilities, still Proposed for the user's review) with `h3_kernel::modifier`; 22 typed creature abilities as pack data; two-hex footprints, obstacles, flying/teleporting, 24 attack positions; ranged combat with distance/melee penalties and area shots; melee abilities (double attack, attack patterns, strike and return, retaliation variants, hate, jousting, defense reduction, death blow, life drain, death stare, regeneration); morale and luck with the wait-morale phase and quirk Q-0003; terrain (native bonus, random obstacles); war machines without heroes; draw on simultaneous wipe-out; loose and tight formations. 45 sourced battle rules (49 with the kernel rules). Seam changes approved by the user: `led_by_hero` (Q17), `BattleResult`/draw (Q24), `formations` (Q25); T8's war-machine fields were covered by Q16. Encodings: action v4, observation v3, C ABI v2, state hash v8, ruleset hash v8. Open for experiments: Q21–Q23; Q26 decided (wiki). Details: [HANDOFF](HANDOFF.md), [epic file](epics/E03-battle-core.md).

## Stub registry

See `docs/stubs.toml` (empty: no registered stubs after E01).

## Discovered work inbox

(Agents: add coarse items here, one line each, with date and your epic ID. Triage into the table at the next planning pass.)

- 2026-09-30 (E01): CI configuration once a host is chosen (Q3): Linux + Windows matrix running `cargo xtask verify`; turn the doc-drift warning into an error for large changes.
- 2026-09-30 (E01): WASM build check of the logic crates (`wasm32` target could not be installed in the E01 environment); add a `wasm` verify step, ideally running the golden replays under a WASI runtime.
- 2026-09-30 (E01): machine check for minimal `pub` surface of seam/contract crates (ADR-0005 §4 is not enforced yet).
- 2026-09-30 (E01): `Trace` sink for rule-level explanations ("explain this damage number", 02-architecture §10).
- 2026-09-30 (E01): mutation testing (`cargo-mutants`) and fuzzing (`cargo-fuzz`) set-up; first fuzz targets come with E02's binary parsers.
- 2026-09-30 (E01): profile-level Tier-2 overrides (still deferred after E02; content-pack override rules were done in E02 T3).
- 2026-09-30 (E01): faster state hash if search needs per-node hashing (FNV-1a ≈ 1 ns/byte; bump `HASH_VERSION`).
- 2026-09-30 (E01): bump the pinned toolchain (1.95.0) to the current stable when the build environment allows; re-baseline bench-check.
- 2026-10-01 (E02): war machines (Catapult, Ballista, First Aid Tent, Ammo Cart, Arrow Tower) are not harvested (speed 0, no damage for most): model them with heroes/sieges (E03/E04) and extend the harvester. *(E03 T8: Ballista, First Aid Tent and Ammo Cart harvested and implemented; Catapult and Arrow Tower remain for sieges, E04.)*
- 2026-10-01 (E02): abilities from the harvest report for E03 (jousting, morale ±, no wall penalty, Dragon Fly weakness from SoD/RoE 1.3, hate, …) and the unidentified exe flag bit `0x800` (Arch Mage, Beholder, Evil Eye).
- 2026-10-01 (E02): ~~tight formations~~ (done in E03 T9) / tactics (E04), creature-bank layouts (with creature banks), and other harvested tables (spells, artifacts, heroes, buildings) when their epics come; RoE/AB patch differences outside creatures need the same gap-fill treatment (Q15).
- 2026-10-01 (E02): `cargo-fuzz` targets for `h3-formats` (property tests cover arbitrary input today; the readers only run in the harvester on local files).
- 2026-10-01 (E03 T7): special battlefields for E04: magic terrains (Holy Ground, Evil Fog, Clover Field, Cursed Ground, Magic Plains, Fiery Fields, Rocklands, Magic Clouds, Lucid Pools), coast and ship battles — battlefield content, their obstacles (VCMI `specialBattlefields`, left out of the gap-fill file) and effects (morale/luck by alignment, spell effects).
- 2026-10-01 (E03 T7): harvest the obstacle table from `Heroes3.exe` (today from VCMI's `config/obstacles.json`), once its layout is known.
- 2026-10-01 (E03 T7): `h3-sim`/C ABI: random obstacles per episode (today scenarios fix them, or the CLI generates them once per run from the seed). *(E03 T9 review: `h3_pack::synthetic`, the C ABI and `BattleEnv::reset` ignore `random_obstacles`, so the FFI duel plays without obstacles, unlike `h3 run`.)*
- 2026-10-01 (E03 T9 review): bound the work of one attack: death stare rolls once per creature (`combat::death_stare`), so a stack of 4·10⁹ creatures takes seconds per attack. Options: a ruleset `max_stack_count` checked at setup (also bounds every other count-proportional cost) or an exact integer binomial sampler. Reachable only with crafted setups.
- 2026-10-01 (E03 T9 review): information hiding for server mode (E16): the tools derive the public random obstacles and the hidden battle RNG from the same seed (different streams), so a low-entropy seed can be guessed from the layout. Server mode needs a secret, uniformly random seed (or an independent obstacle seed); write it into docs/06-security.md with the server.
- 2026-10-01 (E03 T9 review): compiler check that the war-machine positions cannot overlap (two-hex footprints); today an overlap is a `NoRoom` setup error at battle start.
- 2026-10-01 (E03 T9 review): the Ballista's exact value comparison (`war.rs`, u128 cross products) is safe for war machines (count 1, < 2^99); use checked arithmetic if E04 lets other stacks use it (hero Artillery, towers).
- 2026-09-30 (E01): batch stepping over many environments in `h3-sim`/C ABI, and loading packs/scenarios through the ABI (E05).
