# RMG lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`.

## Current state

- **E14** (the original's random map generator, ported to give the same map per seed): in progress since 2026-11-23. **Increments 1 and 2 closed** (2026-11-24, RMG-Q2; 2026-11-25, decision 10; `docs/epics/E14-archive.md`): the survey `docs/rules/surveys/E14-generator.md`, the ledger `docs/rules/quirks-rmg.md`, the oracle and the phase oracle `ORACLE.md`, **ADR-0029** (`docs/adr/0029-original-map-generator.md`); `h3-rmg`'s start, the rmg part, the catalog `docs/rules/catalog/rmg.toml`. **Increment 3 refined** (steps 1 to 3 and the terrain painter; T3a to T3e, the epic's "Increment 3's tasks"): T3a, T3b, T3c done; **T3d done** (2026-11-26): step 2 ported (RMG-ZON-009 to -014, `triangulation.rs`, `partition.rs`), its draws and cells equal the oracle's on 24 of 24 with `zones.txt`'s bytes, the water zones `water.txt`'s (new oracle record); RMG-QUIRK-1's provisional form (constants, `Trace::zone_memory` for measured bytes; 22 of 24 with the constants alone), -14 confirmed.
- **Next:** **T3e** (step 3 and the terrain painter), then **increment 3's checkpoint** with the user: RMG-QUIRK-1's model's form, and the ledger's open table (T3d's counts: the clip of an off-map side 104 times in 19 cases; 20 zones, 7 of land, that fill nothing). Every oracle run needs the clone at `/home/claude/homm3` (the fixed environment's size, `TRAPS.md`).
- **Open pull requests:** **#PR** (`rmg/e14-t3d`, T3d). Watch it until it merges.
- **Owns** (`lanes.toml`): `crates/h3-rmg/`, `crates/h3-pack-rmg/` (`h3-map`'s `src/generated*` joins at increment 7), the packs' `rmg.toml`, the RMG rules (`docs/rules/catalog/rmg.toml`) and quirks, E14's epic, surveys and experiments, this folder.
- **Versions:** `RMG_RULESET_HASH_VERSION` 3 (T3c), `RMG_GENERATOR_VERSION` 1 (no map written before increment 7; `h3-rmg`); the core's `RULESET_HASH_VERSION` 54 (T2c, shared). `CrtRand` has none (its sequence is the original's).
- **Ids:** questions `RMG-Q<n>` (next 3; none open), quirks `RMG-QUIRK-<n>` (next 25; seven candidates open in the ledger), stubs `RMG-STUB-<n>` (next 2; RMG-STUB-1 open: steps 3 to 15), decisions next 11. Registry: 60 of 128 positions taken (52, 53 and 56 to 59 the rmg lane's `quirk.rmg` flags; 12 of the ledger's still to come).
- **Waiting on other lanes:** the writer's seven gaps (survey §3.3.3) before increment 7, in the adventure (1 to 6) and town (7) lanes' inboxes; the town lane's answer on a neutral main town (RMG-QUIRK-23).
- **Working agreement:** a checkpoint with the user per increment (decision 6); tokens first; read the executable in the cloud sandbox (`docs/exe/README.md`, "Setup"); a big task cut at a seam with the user, worker sub-agents allowed for its parts (2026-11-23); a couple of small tasks or one bigger per session (2026-11-24); T2c, T3c and T3d kept whole (decision 9; the user, 2026-11-26).

## Recent sessions

### 2026-11-26 — T3d step 2 (`rmg/e14-t3d`)

A new chat; staged the oracle's set from the PC (58 files, SHA-256 checked), SoD's tables, Complete's two maps; #1196 had merged. Asked for a cut (on water, on code, whole); the user: whole. One oracle run recorded `water.txt`; one agent read the borders, the clip, the fill and the water connections exactly. The step matched on 23 of 24 at the first run, 24 after the fill's start was corrected; the coverage gate drove the faults into a per-level record.

### 2026-11-26 — T3c step 1, the generator's map, the comparison seam (`shared/e14-t3c`)

#1196. A new chat; staged the oracle's set from the PC (58 files, SHA-256 checked), SoD's tables, Complete's two maps; #1187 had merged; baseline green (CI's `master` run, the scoped verify). Asked for a cut; the user: T3c whole, plus an oracle record of the zone generators after step 1 (dumps 0 and 1 are only the reset cells). One agent registered the four flags in a worktree; step 1's draws, dumps and layout matched the oracle on all 24 cases at the first run.

### 2026-11-26 — T3b step 2's items not read (code reading; docs and the exe map)

A new chat; staged only `Heroes3.exe` from the PC (code reading, no oracle run); #1185 had merged. Read the triangulation, the borders, the clip, the fill, the water connections and the run's exception data in the sandbox; stopped the setup's full baseline (docs only; CI on `master` stands in). Found the edge walk's chords (a corner case T3d checks against the oracle's cells) and that no handler catches a division by zero.
