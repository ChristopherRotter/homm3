# Adventure lane — HANDOFF

The adventure lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E08 increment 1** (epic `docs/epics/E08-adventure-map.md`, decisions 1–12; done tasks in `E08-log.md`). T1a, T1b, T2, T3, T4, T5a, T6a, T6b and T7a done. The user takes one or two tasks per round with a checkpoint after each (user, 2026-10-10); next **the user's checkpoint after T6b/T7a**: ADV-Q3's D-2 and D-3, any objection to fix-optional for `ADV-QUIRK-2`/`-3`; then the round they pick (the queue's next: T7b, waiting for that checkpoint; T5b ready).
- **Map reader and import** (`h3-map`): `read`/`parse` into `model::RawMap`; `import` into `h3_game::GameSetup`, `import_map` its static layer (`MAP_IMPORT_VERSION` 1, `docs/formats/h3m.md` "Import"). Every shipped map's static layer imports and validates; 10 maps import whole, the rest are typed `Unsupported`. `verify --with-original-data` runs `crates/h3-map/examples/original_data.rs` on `original-data/install/Maps` (staged 50 per call).
- **Setups and state:** `h3_adventure::{AdventureSetup, AdventureState}` (static map behind an `Arc`, heroes by id with their movement points, state hash v2; `check_stand` is ADV-MAP-001, `day_points` ADV-MOV-001, `start_turn` ADV-MOV-002); `h3_game::{GameRuleset, GameSetup, GameState}`: the starting armies drawn from the setup stream (GAM-SET-001), a stream per player, sequential turn groups, day 1, `apply` with `EndTurn` (GAM-TRN-001/002, GAM-CAL-001), game state hash v1. `GameSetup::validate` builds with seed 0.
- **Adventure ruleset part** (v3 since T6b): terrains, roads, rivers, native cost, land table, base scouting radius, the setup bounds and the starting stack chances. ADV-MOV-003/004/007 and ADV-FOG-001 stay `specified` until T7b/T8.
- **Readings this increment** (survey `docs/rules/surveys/E08-adventure-core.md`, map entries): CR-2's callers (movement points at the start of the owner's turn), the starting army (0x4C9730), CR-1 (the step cost 0x4B1740, the move 0x4805E0). Conflicts with the wiki or VCMI go to the user (policy "consensus before trust"): ADV-Q4 and ADV-Q6 have the user's interim choices and confirmation experiments for T10; ADV-Q3 waits for the checkpoint.
- **ADRs accepted** (2026-10-10): ADR-0015, ADR-0016, ADR-0017 (point 3's "every rule at the start of the day" no longer holds: ADV-MOV-002 runs at the start of a turn; revision note in the shared PR).
- **Waiting for the user:** the checkpoint above; three editor maps for the fixtures (one per version: a customised hero, many object kinds, a special condition, a rumor, a timed event, one underground); A-1, the fog fringe: four answers after looking at a map in SoD (survey §4), needed by T8. Open: `ADV-Q3`–`ADV-Q6`.
- **Fuzzing is stable-only for now** (user, 2026-10-10): property tests on arbitrary bytes; `cargo-fuzz` later.
- **Not before E04b closes:** the content split and the writer's move into `h3-map` (BACKLOG R01). Until then adventure crates use only the core items of `h3-content`.
- **Ownership:** the four crates, `src/adventure/` in `h3-sim` and `h3-cli`, `crates/h3-cli/tests/adventure_replays.rs`, `fixtures/replays/adventure/`, `fixtures/maps/`, each pack's `adventure.toml`, `docs/formats/h3m.md`, `docs/rules/catalog/{adventure,game,town}.toml`. `h3-heroes` moves to this lane when hero progression starts (increment 3).
- **Ids:** questions `ADV-Q<n>`, quirks `ADV-QUIRK-<n>` (1 reserved for the fog fringe; 2, 3 the diagonal exception), stubs `ADV-STUB-<n>`, rules `ADV-*` and `GAM-*`. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** increments of a couple of tasks with a user checkpoint; the user confirms seams and ADRs; token efficiency beats speed (AGENTS.md).

## Standing traps

- Cheap forks are a hard requirement (decision 6): no per-fork deep copy of map-sized data; pointer identity never in logic or the hash.
- RNG seeds and stream positions are server-only state (decision 9): never in an observation, never seeded from client data.
- Template masks: bit `j` of byte `i` is the tile `7 − j` columns left of and `5 − i` rows above the anchor (`h3m.md` had it mirrored until T5a); check a geometric reading of the format against the shipped maps (towns' gates, heroes' tiles) before building on it.
- A shared change the lane needs (Cargo members, `arch.toml`, `lanes.toml`, contracts, kernel, xtask) goes in a `shared/` pull request; one that must land with lane paths (a new crate) needs the user's consent and the label `cross-lane`. A `shared/` or cross-lane branch runs the full verify (its scoped `check-lane` fails by design).

## Recent sessions

### 2026-10-10 — T5a/T6a checkpoint and T6b + T7a: the game state, the readings

The user took T6b with T7's reading and moved T5b's gated benchmark to T9 (T5b measures on its own example; `h3-cli` is battle-owned). Readings: the original sets a day's movement points at the start of the owner's turn (0x4C6FE0), against the wiki and VCMI (the user chose the reading, ADV-Q4); the starting army rolls the second and third slots apart, the war machine with the second (the user expected it guaranteed; ADV-Q6 for increment 3); the step cost 0x4B1740 (D-2 VCMI's way, D-3 differs from VCMI, A-2 and A-3 confirmed by an independent re-read: `ADV-QUIRK-2`/`-3`). Built: `GameState`, `apply`/`EndTurn`, the draw, movement points per turn. Trap: H3API's hero +0x49 is the maximum, +0x4D the current points. Detail: `E08-log.md`.

### 2026-10-10 — T4 checkpoint and T5a + T6a: the setups and the import

At the checkpoint the user took T5a and T6a as one round and chose a starting-army placeholder for placed heroes without stacks (58 of 116), drawn in T6b. Built: `AdventureSetup`/`AdventureState` (static map behind an `Arc`, ADV-OBJ-001, ADV-MAP-001, state hash v1), the setup bounds in the ruleset part (v2), `GameRuleset`/`GameSetup` with validation, `h3_map::import`/`import_map` with typed `Unsupported` errors and code tables by the packs' original ids. Found: the masks' geometry was mirrored in `h3m.md` since T3 (VCMI `readMap`/`readTxt`; towns' gates showed it); anchors may lie past the right and bottom edges; towns' gates are blocked and visitable (visitable wins). Shipped maps: 10 import whole, 154 unsupported. Trap: the VM restarted during the baseline verify (rerun the scoped verify with `LLVM_COV`/`LLVM_PROFDATA` set as `scripts/sandbox-verify.sh` does). Detail: `E08-log.md`.

### 2026-10-10 — T3 checkpoint and T4: the adventure content

At the checkpoint the user chose T4 alone for this round; T4–T10 went into queue entries first (T5a/T5b, T6a/T6b). T4: the ruleset part compiled on the core ruleset from one adventure pack per core pack (the core's order and closures), the strict `adventure.toml` format, base values with sources (wiki and VCMI agree on the costs; VCMI's config files fetched raw from GitHub). D-4 settled by original data: the original reads the land table from `movement.txt` (0x4D7240 → 0x698A98; 0x4E4990 takes the slowest of the seven slots from 20), the wiki's row; increment-3 findings (specialty +1 speed, Logistics, sea by Navigation) in the log and the map. Cross-lane: one paragraph in `packs/README.md`. Trap: `git push --delete` and the REST ref delete are refused through this session's proxy. Detail: `E08-log.md`.
