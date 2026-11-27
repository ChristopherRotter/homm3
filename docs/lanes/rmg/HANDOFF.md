# RMG lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`.

## Current state

- **E14** (the original's random map generator, ported to give the same map per seed): in progress since 2026-11-23. **Increments 1 to 3 closed** (2026-11-24, RMG-Q2; 2026-11-25, decision 10; 2026-11-27, decision 11; `docs/epics/E14-archive.md`): the survey `docs/rules/surveys/E14-generator.md`, the ledger `docs/rules/quirks-rmg.md`, the oracle and the phase oracle `ORACLE.md`, **ADR-0029** (`docs/adr/0029-original-map-generator.md`); `h3-rmg` (the rmg part, the frame, steps 1 to 3 and the game's terrain painter: the draws and cells equal the oracle's on 24 of 24), the catalog `docs/rules/catalog/rmg.toml`. **Increment 4 refined** (towns and connections, steps 4 to 7; T4a to T4e, the epic's "Increment 4's tasks"); **T4a done** (2026-11-27: the items not read, read; docs only).
- **Next:** **T4b** (steps 4 and 5; RMG-QUIRK-25's fix and flag). Before its port, the rmg packs need each picture's `.msk` (width, height, colour mask: the footprint and the passable mask, survey §2.1), a `cross-lane` harvest to agree with the user; then the oracle's set staged. RMG-QUIRK-1's form is open until T4e (decision 11). Every oracle run needs the clone at `/home/claude/homm3` (`TRAPS.md`).
- **Open pull requests:** **#1206** (`rmg/e14-t4a`, T4a, docs only). Watch it until it merges. (#1204 merged 2026-11-27.)
- **Owns** (`lanes.toml`): `crates/h3-rmg/`, `crates/h3-pack-rmg/` (`h3-map`'s `src/generated*` joins at increment 7), the packs' `rmg.toml`, the RMG rules (`docs/rules/catalog/rmg.toml`) and quirks, E14's epic, surveys and experiments, this folder.
- **Versions:** `RMG_RULESET_HASH_VERSION` 4 (T3e), `RMG_GENERATOR_VERSION` 1 (no map written before increment 7; `h3-rmg`); the core's `RULESET_HASH_VERSION` 54 (T2c, shared). `CrtRand` has none (its sequence is the original's).
- **Ids:** questions `RMG-Q<n>` (next 3; none open), quirks `RMG-QUIRK-<n>` (next 26; six candidates open in the ledger, three for increment 4 (T4a's one-way monoliths among them, for the user), three for increment 7's census), stubs `RMG-STUB-<n>` (next 2; RMG-STUB-1 open: steps 4 to 15), decisions next 12. Registry: 60 of 128 positions taken (52, 53 and 56 to 59 the rmg lane's `quirk.rmg` flags; 13 of the ledger's still to come, -25's at T4b).
- **Waiting on other lanes:** the writer's seven gaps (survey §3.3.3) before increment 7, in the adventure (1 to 6) and town (7) lanes' inboxes; the town lane's answer on a neutral main town (RMG-QUIRK-23).
- **Working agreement:** a checkpoint with the user per increment (decision 6); tokens first; read the executable in the cloud sandbox (`docs/exe/README.md`, "Setup"); a big task cut at a seam with the user, worker sub-agents allowed for its parts (2026-11-23); a couple of small tasks or one bigger per session (2026-11-24); T2c, T3c, T3d and T3e kept whole (decision 9; the user, 2026-11-26).

## Recent sessions

### 2026-11-27 — T4a the items not read (`rmg/e14-t4a`, #1206)

A new chat; staged `Heroes3.exe` from the PC (SHA-256 checked); #1204 had merged, `master`'s CI green. Code reading only: the objects' masks through `objects.txt`'s reader and `msk_load`, `rmg_map_can_place`'s footprint, outline and ring, the null prototype's six callers, the islets' height map, the monoliths' pairing; VCMI 1.7.5's `ObjectTemplate` compared. Into the survey (§2.1 to §2.3, §3.2), `map.toml`, `structs.toml`, the ledger (two faults joined RMG-QUIRK-3 and -4; the one-way monoliths an open candidate). Found for T4b: the `.msk` data the packs lack.

### 2026-11-26 — T3e step 3 and the terrain painter (`shared/e14-t3e`, #1204)

A new chat; staged the oracle's set from the PC (58 files, SHA-256 checked), SoD's tables, Complete's two maps; #1199 had merged. Asked for a cut (at the islands, at the painter, whole); the user: whole, the picture sets' harvest `cross-lane`. Read step 3 and the painter's ambiguous parts, `rmg_island_fill` whole; one oracle run (`--phases`) for dumps, never needed: the port matched all 24 cases at the first run. The corpus never resolves a pinched water or rock cell: unit tests cover that branch until steps 6 and 11. Then increment 3's checkpoint (decision 11): increment 3 closed, increment 4 cut into T4a to T4e; its docs on #1204 too.

### 2026-11-26 — T3d step 2 (`rmg/e14-t3d`, #1199)

A new chat; staged the oracle's set from the PC (58 files, SHA-256 checked), SoD's tables, Complete's two maps; #1196 had merged. Asked for a cut (on water, on code, whole); the user: whole. One oracle run recorded `water.txt`; one agent read the borders, the clip, the fill and the water connections exactly. The step matched on 23 of 24 at the first run, 24 after the fill's start was corrected; the coverage gate drove the faults into a per-level record.
