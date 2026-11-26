# RMG lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`.

## Current state

- **E14** (the original's random map generator, ported to give the same map per seed): in progress since 2026-11-23. **Increments 1 and 2 closed** (2026-11-24, RMG-Q2; 2026-11-25, decision 10; `docs/epics/E14-archive.md`): the survey `docs/rules/surveys/E14-generator.md`, the ledger `docs/rules/quirks-rmg.md`, the oracle and the phase oracle `ORACLE.md`, **ADR-0029** (`docs/adr/0029-original-map-generator.md`); `h3-rmg`'s start, the rmg part, the catalog `docs/rules/catalog/rmg.toml`. **Increment 3 refined** (steps 1 to 3 and the terrain painter; T3a to T3e, the epic's "Increment 3's tasks"): **T3a** (the memory record, the fixed environment, the corpus at 24) and **T3b** (step 2 read whole) done; **T3c done** (2026-11-26): step 1 ported (RMG-ZON-001 to -008, `layout.rs`), the generator's map (`Cells`, RMG-SET-010), the comparison seam (`Trace::step_end`'s `StepState`, dump format 1, the draws' digests against `phases.txt` in CI), the oracle's zone generators after each step (`layout.txt`: 388 of 388 the port's), the four flags registered (56 to 59).
- **Next:** **T3d** (step 2 with T3b's five corner cases; RMG-QUIRK-1's model, -14; the zone generators after step 2 against a record like `layout.txt`), then T3e, then increment 3's checkpoint with the user (RMG-QUIRK-1's model's form). Every oracle run needs the clone at `/home/claude/homm3` (the fixed environment's size, `TRAPS.md`).
- **Open pull requests:** the session's `shared/e14-t3c` (T3c, `cross-lane`), named in the newest session entry. Watch it until it merges.
- **Owns** (`lanes.toml`): `crates/h3-rmg/`, `crates/h3-pack-rmg/` (`h3-map`'s `src/generated*` joins at increment 7), the packs' `rmg.toml`, the RMG rules (`docs/rules/catalog/rmg.toml`) and quirks, E14's epic, surveys and experiments, this folder.
- **Versions:** `RMG_RULESET_HASH_VERSION` 3 (T3c: step 1's tables), `RMG_GENERATOR_VERSION` 1 (no map written before increment 7; `h3-rmg`); the core's `RULESET_HASH_VERSION` 54 (T2c, shared). `CrtRand` has none (its sequence is the original's).
- **Ids:** questions `RMG-Q<n>` (next 3; none open), quirks `RMG-QUIRK-<n>` (next 25; seven candidates open in the ledger), stubs `RMG-STUB-<n>` (next 2; RMG-STUB-1 open: steps 2 to 15), decisions next 11. Registry: 60 of 128 positions taken (52, 53 and 56 to 59 the rmg lane's `quirk.rmg` flags; 12 of the ledger's still to come).
- **Waiting on other lanes:** the writer's seven gaps (survey §3.3.3) before increment 7, in the adventure (1 to 6) and town (7) lanes' inboxes; the town lane's answer on a neutral main town (RMG-QUIRK-23).
- **Working agreement:** a checkpoint with the user per increment (decision 6); tokens first; read the executable in the cloud sandbox (`docs/exe/README.md`, "Setup"); a big task cut at a seam with the user, worker sub-agents allowed for its parts (2026-11-23); a couple of small tasks or one bigger per session (2026-11-24); T2c and T3c kept whole (decision 9; the user, 2026-11-26).

## Recent sessions

### 2026-11-26 — T3c step 1, the generator's map, the comparison seam (`shared/e14-t3c`)

A new chat; staged the oracle's set from the PC (58 files, SHA-256 checked), SoD's tables, Complete's two maps; #1187 had merged; baseline green (CI's `master` run, the scoped verify). Asked for a cut; the user: T3c whole, plus an oracle record of the zone generators after step 1 (dumps 0 and 1 are only the reset cells). One agent registered the four flags in a worktree; step 1's draws, dumps and layout matched the oracle on all 24 cases at the first run.

### 2026-11-26 — T3b step 2's items not read (code reading; docs and the exe map)

A new chat; staged only `Heroes3.exe` from the PC (code reading, no oracle run); #1185 had merged. Read the triangulation, the borders, the clip, the fill, the water connections and the run's exception data in the sandbox; stopped the setup's full baseline (docs only; CI on `master` stands in). Found the edge walk's chords (a corner case T3d checks against the oracle's cells) and that no handler catches a division by zero.

### 2026-11-25 — T3a the memory record, the fixed environment, the corpus at 24

A new chat; staged the client lane's set from the PC (58 files, SHA-256 checked), SoD's tables and Complete's two maps; baseline green. Asked for a cut; the user: none, sub-agents used efficiently. The first hooked run showed the sandbox's environment moving step 2's heap bytes (`m2-4p`'s variant), so the game's environment was fixed before the record; one agent measured what moves the bytes (the block's size). RMG-QUIRK-24 confirmed by the oracle. `l1-4p-b` and `xl2-4p-seed0` re-kept under the fixed environment.
