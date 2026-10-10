# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md` (refined 2026-10-08, reviewed with the user the same day; decisions 1–12 there; done tasks in `E08-log.md`). Increment 1 = T1a, T1b, T2–T10, with a checkpoint with the user at T1a (done: ADR-0015–0017), after T3 (done, 2026-10-10) and at T10. T1a, T1b, T2, T3 and T4 are done; the import moved from T3 to T5 and T6 (user, 2026-10-10). Refined into the entries below at the T3 checkpoint (2026-10-10): the user takes one or two tasks per round, each round ends at a checkpoint (the round after T3: T4 alone). Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

### E08 T5a — Static map and setup

- **Scope:** `AdventureSetup` (dimensions and levels, per tile terrain, road and river as T4's handles, object footprints with their blocked and visitable masks, heroes on the map with owner, tile and army) and its validation, one check per invariant (ADR-0015 point 4: a hero on a passable tile, not blocked, inside the map; sizes within the ruleset's bounds); `AdventureState` with the static map behind an `Arc` and the heroes in plain arrays; the canonical state hash (`ADVENTURE_STATE_HASH_VERSION`). `h3-map`'s import of terrain, roads, rivers and footprints into `AdventureSetup` (the format's codes → keys only there), run on every shipped map by `examples/original_data.rs`.
- **Acceptance:** setups built in code cover each validation error; every shipped map imports its static layer; equal content hashes equal.
- **Depends on:** T4 (done). **Status:** ready (the user picks the next round's tasks at the T4 checkpoint).

### E08 T5b — The persistent grid

- **Scope:** `src/persistent.rs` (an `Arc` table of `Arc` leaves, uniform leaves shared through a sentinel held by the grid, set and clear; the only place `ptr_eq`/`as_ptr` may appear), the two copy-on-write properties (ADR-0016 point 2), the leaf-size benchmark against a flat per-player copy, fork benchmarks on 36×36 and 144×144 with their first baseline in `benches/adventure/`.
- **Acceptance:** both properties pass; the measurements and the chosen leaf size in `docs/02-architecture.md` §8 and the log. A grid that loses on hot reads goes back to the user (ADR-0016).
- **Depends on:** T5a. **Status:** ready after T5a.

### E08 T6a — Game setup and the rest of the import

- **Scope:** `h3_game::GameSetup` (players, the turn schedule's input, the adventure setup) and its validation; `h3-map`'s import of players and placed heroes with the typed `Unsupported` errors (a random hero, a hero generated at a main town); `MAP_IMPORT_VERSION` documented in `docs/formats/h3m.md`; one property test on arbitrary bytes running parse, import and validation together.
- **Acceptance:** the shipped maps sort into imported and `Unsupported` with a count per reason in the log; no panic on arbitrary bytes.
- **Depends on:** T5a. **Status:** ready after T5a.

### E08 T6b — Players, turns and the day

- **Scope:** players, turn groups (sequential play only), the calendar, the per-player RNG streams, `EndTurn` and the day advance with the movement points of the day (ADV-Q4: the start of the day; read CR-2's call sites first), command routing across contexts (ADR-0017).
- **Acceptance:** rules `GAM-*` implemented with sources; a rejected `EndTurn` leaves the state untouched.
- **Depends on:** T6a. **Status:** ready after T6a.

### E08 T7 — Movement

- **Scope:** first CR-1 (`ADV-Q3`: D-2, D-3, A-2, A-3), then the user classifies A-2 and A-3; `MoveHero` with a full path validated step by step and rejected whole, the reachable-tiles query (costs and the chosen shortest path), the leavable own tile, rules `ADV-MOV-*`. May split into a reading and a build task once the reading is done.
- **Acceptance:** each step rule has a test from its source; the query and `MoveHero` agree (a property).
- **Depends on:** T4, T5a, T6b; the user's classification after CR-1. **Status:** waiting (T6b).

### E08 T8 — Fog of war and projection

- **Scope:** first CR-3 (`ADV-Q5`); reveal by scouting radius on the grid of T5b, the fringe quirk `ADV-QUIRK-1` if confirmed (A-1), events with their visibility, `observe` and `legal_commands`, the viewer knowledge list in `docs/06-security.md`, the property tests (atomicity, legality consistency, ownership isolation, observation hygiene, clone equivalence, determinism).
- **Acceptance:** the seven property tests pass on imported maps.
- **Depends on:** T5b, T7; the user's four answers on the fringe (A-1). **Status:** waiting (A-1, T7).

### E08 T9 — Env, CLI, gates

- **Scope:** `AdventureEnv` (reset, step, mask, fork, hash; the action a destination among the reachable tiles), a random agent, the CLI's run, replay verify and bench, golden replays naming their map by content hash, path and `MAP_IMPORT_VERSION` (ADV-Q2), the benchmark workload and its baseline (decision 11).
- **Acceptance:** the replays pass the determinism matrix; the +2 % gate has a baseline.
- **Depends on:** T8; the editor maps as committed fixtures (Inbox). **Status:** waiting (T8).

### E08 T10 — Closure

- **Scope:** the movement experiments' spec shape (cross-lane, decision 12) and a batch, the docs, the refinement of increment 2, the checkpoint.
- **Depends on:** T9. **Status:** waiting (T9).

## Inbox

- 2026-10-10 (T4): check `packs/base/adventure.toml`'s land table against the installation's `movement.txt` under `--with-original-data` (an `examples/original_data.rs` in `h3-pack-adventure` needs `h3-formats` as a dev-dependency to read the LOD: a shared `arch.toml` change), or harvest the table when adventure data is harvested.
- 2026-10-10 (T3b): fixtures in `fixtures/maps/` once the user's editor maps arrive (one per version: a customised hero, many object kinds, a special condition, a rumor, a timed event, one underground): commit them with a test that reads each and checks a few fields, so CI reads real editor output (the shipped maps stay local).

- 2026-10-10 (T2): the game context's rule ids use the existing prefix `GAM-` (`check-rules`, catalog file `docs/rules/catalog/game.toml`), not `GAME-` as the T1b survey proposed.
- 2026-10-10 (T2): already allowed by `arch.toml`, so T5–T9 need no shared change: `h3-sim` → `h3-adventure`, `h3-game` (dev: `h3-map`, `h3-pack-adventure`); `h3-cli` → the four crates; `h3-map` dev → `h3-pack`, `h3-pack-adventure`. Golden replays go to `fixtures/replays/adventure/` with the test `crates/h3-cli/tests/adventure_replays.rs` (both adventure-owned): `cargo xtask determinism` runs it once it exists, and `verify --scope adventure` runs the determinism step once the folder has files (T9).
- 2026-10-01 (E02): ~~tight formations~~ (done in E03 T9) / tactics (E04), creature-bank layouts (with creature banks), and other harvested tables (spells, artifacts, heroes, buildings) when their epics come; RoE/AB patch differences outside creatures need the same gap-fill treatment (Q15).
- 2026-10-01 (E04 refinement): the `.h3m` writer built for the experiment pipeline (E04a T2) overlaps with E07 (map reader) and E14 (map generator). ~~Keep it in the formats adapter so both can extend it~~: E08 decision 2 moves it into `h3-map` after E04b closes, beside the reader (2026-10-08).
- 2026-10-03 (E04a T3a): not harvested yet, each for the epic that reads it: spell town chances and hero-class tavern weights and aggression (E09 towns and taverns; RoE lacks the Conflux column, so the `ab` pack needs an override kind or a per-faction table), spell AI values (E05/E06 if wanted), the spell flags and target attributes (E04b T1/T2a, decode the executable's flag bits, installation.md §7.6), hero availability (campaign heroes from the executable's flag byte 2 and VCMI's `special`; Lord Haart the Knight retired by the expansions, flag byte 1; Sir Mullich: VCMI special, executable not campaign; E08–E10, check in the SoD map editor and taverns), specialty names (presentation).
- 2026-10-03 (E04a T3b): artifacts not modelled in E04a, each for its epic (the packs mark them `# pending`, `packs/README.md`): Necromancy artifacts with the Necromancy skill and the raised creature by level (dispute D8; E08–E10), combination costs (blank in the original; artifact merchants, E09), the artifact class of Vial of Dragon Blood and Armageddon's Blade for random placement (D4; E08/E14), backpack capacity (D7).
- 2026-10-01 (E04 refinement): hero progression (experience table, level-up algorithm with the class probabilities, skill offers) and the hero commands (equip, unequip, learn, trade) are deferred to the adventure and game epics (E08–E10).
- 2026-10-04 (E04a T9): applying a battle outcome (E08–E10) follows batch 1b (`experiments/results/E04a-batch-1b.md`): killed war machines give no experience; Learning multiplies the whole sum including the +500 for a defeated hero (R5: floor(550 × 1.15) = 632; D-3, OQ-6); the loser's surviving war machines are not captured (OQ-5); Necromancy rounds down per killed stack (R6: 6, 6, 6, 14, 17 at 10 % → 2 Skeletons; D-7), so it needs `BattleOutcome.stacks`. Also check whether a player without towns may retreat or surrender (not researched).
