# Backlog

Coarse epics in rough priority order. Only epics marked **Refined** have an epic file. Everything else is a sketch that will be refined when it becomes next. Reordering is cheap; update this file when priorities change.

Status: `todo` · `next` · `in-progress` · `partial` · `done`

| ID | Epic | Status | Detail level |
|---|---|---|---|
| E00 | Foundations: vision, architecture, ADRs, process docs | done (2026-09-30) | — |
| E01 | **Walking skeleton**: minimal battle end to end (workspace, gates, kernel, tiny content pack, battle core, commands + validation, observation, sim env, CLI, replay + hash, benchmark) | done (2026-09-30; verified on Linux and Windows) | **Refined** → [epics/E01-walking-skeleton.md](epics/E01-walking-skeleton.md) |
| E02 | Content pipeline: catalogue the user's game archives/data tables, harvest them once into committed `base`/`ab`/`sod` packs (toggleable expansions), pack format v2, synthetic test packs | done (2026-10-01; verified on Linux and Windows incl. `--with-original-data`) | **Refined** → [epics/E02-content-pipeline.md](epics/E02-content-pipeline.md) |
| E03 | Battle core (RoE rules): modifier system, abilities via Tier-3 primitives, obstacles, two-hex units, ranged, morale/luck, damage, war machines | done (2026-10-01; verified on Linux and Windows incl. `--with-original-data`) | **Refined** → [epics/E03-battle-core.md](epics/E03-battle-core.md) |
| E04a | **Heroes in battle (no magic)** — the first third of the former E04 (split by the user, Q27): hero content (classes, heroes, secondary skills, artifacts, specialties, spell data rows), `h3-heroes` and the `BattleHero` seam (ADR-0013, reviewed by the user before work continues), primary skills, damage skills, artifacts and specialties without spells, hero-run Ballista and First Aid Tent, tactics, retreat/surrender, richer outcome, the experiment pipeline (spec → `.h3m`, SoD only) | done (2026-10-05; Linux incl. `--with-original-data`; the Windows run is the user's before the merge) | **Refined** → [epics/E04a-heroes-in-battle.md](epics/E04a-heroes-in-battle.md) |
| E04b | **Battle magic** — spell effects and rules, spell book and mana in battle, casting, timed effects on stacks, summons and clones, obstacle spells, resistance, spell-like creature abilities, spell immunities, spellcasting creatures, spell-related skills/artifacts/specialties, hidden-information rules, casting in the sim encodings (needs a factorized action design, flag to E05) | next | sketch (scope fixed by Q27) |
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

Heroes in battle without magic, delivered on branch `e04a-heroes-in-battle` (T1–T12): ADR-0013 (entities, the `BattleHero` seam, the hero's battle rules; accepted by the user, Q30) with the domain crate `h3-heroes` (`HeroDetails`, validation, `derive_battle_hero`, `war_machines`); pack format v3 with hero classes, heroes and specialties, secondary skills, artifacts and spell data rows harvested into `base`/`ab`/`sod`; primary skills, Leadership/Luck and artifact morale and luck, Archery/Offense/Armorer (quirk Q-0005, single-precision rates), hero-run Ballista and First Aid Tent (Q-0004), tactics, retreat, surrender with its price, the richer outcome, stat and trait effects and alignment groups; observation encoding v4 and C ABI v7; the experiment pipeline (spec → `.h3m`) with batches 1a and 1b played by the user in SoD, 21 rules `verified` by observation; the death stare's work bound (Q39, KER-RNG-003) and random obstacles per episode. 16 HER rules, 15 new battle rules and one kernel rule; new quirks Q-0004 to Q-0006 (Q-0003 now reads the real hero). Decisions Q27–Q39. Versions: state hash v17, ruleset hash v13, replay format v13, action encoding v8, observation encoding v4, C ABI v7; about 229–237 k steps/s on the `full` workload with heroes. Closed by an independent review pass (T12c: no defect; robustness tests; five unobserved rules for the next batch). Details: [epic file](epics/E04a-heroes-in-battle.md), [log](epics/E04a-log.md), [HANDOFF](HANDOFF.md).

## Stub registry

See `docs/stubs.toml` (empty: no registered stubs after E01).

## Discovered work inbox

(Agents: add coarse items here, one line each, with date and your epic ID. Triage into the table at the next planning pass.)

- 2026-09-30 (E01): CI configuration once a host is chosen (Q3): Linux + Windows matrix running `cargo xtask verify`; turn the doc-drift warning into an error for large changes.
- 2026-09-30 (E01): WASM build check of the logic crates (`wasm32` target could not be installed in the E01 environment); add a `wasm` verify step, ideally running the golden replays under a WASI runtime.
- 2026-09-30 (E01): machine check for minimal `pub` surface of seam/contract crates (ADR-0005 §4 is not enforced yet).
- 2026-09-30 (E01): `Trace` sink for rule-level explanations ("explain this damage number", 02-architecture §10).
- 2026-09-30 (E01): mutation testing (`cargo-mutants`) and fuzzing (`cargo-fuzz`) set-up; first fuzz targets come with E02's binary parsers.
- 2026-09-30 (E01): profile-level Tier-2 overrides (still deferred after E02; content-pack override rules were done in E02 T3; E04b builds them for data quirks: Q-0006 (Roland's specialty) and survey S-05 (the elementals' levels), user Q35).
- 2026-09-30 (E01): faster state hash if search needs per-node hashing (FNV-1a ≈ 1 ns/byte; bump `HASH_VERSION`).
- 2026-10-01 (E02): war machines (Catapult, Ballista, First Aid Tent, Ammo Cart, Arrow Tower) are not harvested (speed 0, no damage for most): model them with heroes/sieges (E03/E04) and extend the harvester. *(E03 T8: Ballista, First Aid Tent and Ammo Cart harvested and implemented; Catapult and Arrow Tower remain for sieges, E04.)*
- 2026-10-01 (E02): abilities from the harvest report for E03 (jousting, morale ±, no wall penalty, Dragon Fly weakness from SoD/RoE 1.3, hate, …) and the unidentified exe flag bit `0x800` (Arch Mage, Beholder, Evil Eye).
- 2026-10-01 (E02): ~~tight formations~~ (done in E03 T9) / tactics (E04), creature-bank layouts (with creature banks), and other harvested tables (spells, artifacts, heroes, buildings) when their epics come; RoE/AB patch differences outside creatures need the same gap-fill treatment (Q15).
- 2026-10-01 (E02): `cargo-fuzz` targets for `h3-formats` (property tests cover arbitrary input today; the readers only run in the harvester on local files).
- 2026-10-01 (E03 T7): special battlefields for E04: magic terrains (Holy Ground, Evil Fog, Clover Field, Cursed Ground, Magic Plains, Fiery Fields, Rocklands, Magic Clouds, Lucid Pools), coast and ship battles — battlefield content, their obstacles (VCMI `specialBattlefields`, left out of the gap-fill file) and effects (morale/luck by alignment, spell effects).
- 2026-10-01 (E03 T7): harvest the obstacle table from `Heroes3.exe` (today from VCMI's `config/obstacles.json`), once its layout is known.
- ~~2026-10-01 (E03 T7): `h3-sim`/C ABI: random obstacles per episode (today scenarios fix them, or the CLI generates them once per run from the seed). *(E03 T9 review: `h3_pack::synthetic`, the C ABI and `BattleEnv::reset` ignore `random_obstacles`, so the FFI duel plays without obstacles, unlike `h3 run`.)*~~ *(Done in E04a T12b: `BattleEnv::with_random_obstacles`, the C ABI's duel.)*
- ~~2026-10-01 (E03 T9 review): bound the work of one attack: death stare rolls once per creature (`combat::death_stare`), so a stack of 4·10⁹ creatures takes seconds per attack. Options: a ruleset `max_stack_count` checked at setup (also bounds every other count-proportional cost) or an exact integer binomial sampler. Reachable only with crafted setups. *(Planned: E04a T12 hardening.)*~~ *(Done in E04a T12b, Q39: rolls per creature up to 65 536, above that one draw from the normal approximation, KER-RNG-003.)*
- 2026-10-01 (E03 T9 review): information hiding for server mode (E16): the tools derive the public random obstacles and the hidden battle RNG from the same seed (different streams), so a low-entropy seed can be guessed from the layout. Server mode needs a secret, uniformly random seed (or an independent obstacle seed); write it into docs/06-security.md with the server.
- ~~2026-10-01 (E03 T9 review): compiler check that the war-machine positions cannot overlap (two-hex footprints); today an overlap is a `NoRoom` setup error at battle start.~~ *(Done in E04a T4a: `compile_battle`, BAT-WAR-001.)*
- 2026-10-01 (E03 T9 review): the Ballista's exact value comparison (`war.rs`, u128 cross products) is safe for war machines (count 1, < 2^99); use checked arithmetic if E04 lets other stacks use it (hero Artillery, towers). *(Done E04a T7a: the cross products are compared in 256 bits, `h3_kernel::arith::cmp_fractions`, so any shooter is safe.)*
- 2026-09-30 (E01): batch stepping over many environments in `h3-sim`/C ABI, and loading packs/scenarios through the ABI (E05).
- 2026-10-01 (E04 refinement): the `.h3m` writer built for the experiment pipeline (E04a T2) overlaps with E07 (map reader) and E14 (map generator): keep it in the formats adapter so both can extend it.
- 2026-10-03 (E04a T2): the `.h3m` writer knows tile pictures for dirt, grass and water only; other terrains need their picture numbers from a game map first. (`h3-experiment` now reads the hero tables through `h3_formats::exe::hero_data`, E04a T3a.)
- 2026-10-03 (E04a T3a): not harvested yet, each for the epic that reads it: spell town chances and hero-class tavern weights and aggression (E09 towns and taverns; RoE lacks the Conflux column, so the `ab` pack needs an override kind or a per-faction table), spell AI values (E05/E06 if wanted), the spell flags and target attributes (E04b, decode the executable's flag bits, installation.md §7.6), hero availability (campaign heroes from the executable's flag byte 2 and VCMI's `special`; Lord Haart the Knight retired by the expansions, flag byte 1; Sir Mullich: VCMI special, executable not campaign; E08–E10, check in the SoD map editor and taverns), specialty names (presentation).
- 2026-10-03 (E04a T3b): artifacts not modelled in E04a, each for its epic (the packs mark them `# pending`, `packs/README.md`): Necromancy artifacts with the Necromancy skill and the raised creature by level (dispute D8; E08–E10), combination costs (blank in the original; artifact merchants, E09), the artifact class of Vial of Dragon Blood and Armageddon's Blade for random placement (D4; E08/E14), backpack capacity (D7).
- 2026-10-04 (E04a T4a): `HeroDetails` lists artifacts by type, but a Spell Scroll carries its spell per instance: E04b turns `worn`/`backpack` entries into instances (type + scroll spell) and adds "known spells need a worn spell book" once the book artifact has its effect; experience against level (the experience table) and level bounds come with hero progression (E08–E10).
- 2026-10-04 (E04a T5b): the instruction count of the `full` workload swings with codegen in the geometry hot path: with T5b's code (no change to geometry) `Field::neighbor` got inlined into `Field::neighbors`' array map and the same workload took +7 % Ir (callgrind per function; steps identical). A precomputed neighbour table in `Field` (or neighbours computed from one `coords`) would make `movement::reachable` cheaper and independent of inlining; a quick attempt at the latter was not enough on its own. E04a T6a: it swung back (−3.2 % Ir, `Field::neighbor` out of line again, 104 M → 83 M Ir in that path) while the damage code grew by about 1 % (wide product); re-baselined. E04a T10a: it swung again (+8.5 %, `neighbor` inlined into the array map's drain); `Field::neighbors` is now written out from one `coords`, out of line and cheaper (−12.5 % Ir); re-baselined. A precomputed neighbour table remains an option if it swings again.
- 2026-10-03 (E04a T2): run experiment specs in our engine to print predictions automatically (Q37: not in T11, revisit with the next experiment batches, E04b).
- 2026-10-01 (E04 refinement): E04b's casting (spells × targets) cannot use the flat dense action encoding of `h3-sim`; design a factorized or hierarchical action space with E05 (batch API, masks) before E04b's encoding task.
- 2026-10-01 (E04 refinement): the VCMI oracle harness (E03 scope item 10) was never built; with spells and skills the formula surface is large. Revisit with the evaluation needs of E05/E06; observation experiments (Q28) only cover SoD.
- 2026-10-01 (E04 refinement): hero progression (experience table, level-up algorithm with the class probabilities, skill offers) and the hero commands (equip, unequip, learn, trade) are deferred to the adventure and game epics (E08–E10).
- 2026-10-04 (E04a T9): applying a battle outcome (E08–E10) follows batch 1b (`experiments/results/E04a-batch-1b.md`): killed war machines give no experience; Learning multiplies the whole sum including the +500 for a defeated hero (R5: floor(550 × 1.15) = 632; D-3, OQ-6); the loser's surviving war machines are not captured (OQ-5); Necromancy rounds down per killed stack (R6: 6, 6, 6, 14, 17 at 10 % → 2 Skeletons; D-7), so it needs `BattleOutcome.stacks`. Also check whether a player without towns may retreat or surrender (not researched).
- ~~2026-10-04 (E04a T9b): the observation shows neither whether surrender is possible at all (public) nor the own price (private): `observe` has no ruleset; decide with the observation encoding v4 (E04a T11).~~ *(Done in E04a T11a: `observe` takes the ruleset, Q37.)*
