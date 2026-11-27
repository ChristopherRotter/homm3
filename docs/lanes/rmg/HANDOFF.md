# RMG lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`.

## Current state

- **E14** (the original's random map generator, ported to give the same map per seed): in progress since 2026-11-23. **Increments 1 to 3 closed** (2026-11-24, RMG-Q2; 2026-11-25, decision 10; 2026-11-27, decision 11; `docs/epics/E14-archive.md`): the survey `docs/rules/surveys/E14-generator.md`, the ledger `docs/rules/quirks-rmg.md`, the oracle and the phase oracle `ORACLE.md`, **ADR-0029** (`docs/adr/0029-original-map-generator.md`); `h3-rmg` (the rmg part, the frame, steps 1 to 3 and the game's terrain painter: the draws and cells equal the oracle's on 24 of 24), the catalog `docs/rules/catalog/rmg.toml`. **Increment 4 refined** (towns and connections, steps 4 to 7; T4a to T4e, the epic's "Increment 4's tasks"); **T4a and T4b done** (2026-11-27: the items not read, read; steps 4 and 5, the towns, with the objects' data and model: the draws and cells with objects equal the oracle's to step 5 on 24 of 24).
- **Next:** **T4c** (step 6's obstacle plan; an oracle run: the oracle's set staged from the PC). Asked of the user at T4b's end: the Ownership 0 starts' classification (the ledger's open table) and a look at RMG-QUIRK-25's fix. RMG-QUIRK-1's form is open until T4e (decision 11). Every oracle run needs the clone at `/home/claude/homm3` (`TRAPS.md`).
- **Open pull requests:** **#1210** (`shared/e14-t4b`, T4b, `cross-lane`). Watch it until it merges. (#1206 merged 2026-11-27.)
- **Owns** (`lanes.toml`): `crates/h3-rmg/`, `crates/h3-pack-rmg/` (`h3-map`'s `src/generated*` joins at increment 7), the packs' `rmg.toml`, the RMG rules (`docs/rules/catalog/rmg.toml`) and quirks, E14's epic, surveys and experiments, this folder.
- **Versions:** `RMG_RULESET_HASH_VERSION` 5 (T4b), `RMG_GENERATOR_VERSION` 1 (no map written before increment 7; `h3-rmg`); the core's `RULESET_HASH_VERSION` 54 (T2c, shared). `CrtRand` has none (its sequence is the original's).
- **Ids:** questions `RMG-Q<n>` (next 3; none open), quirks `RMG-QUIRK-<n>` (next 27; RMG-QUIRK-26 decided, its flag at T4d; five candidates open in the ledger, two for increment 4 (the Ownership 0 starts, for the user; 6LM10's self-connection), three for increment 7's census), stubs `RMG-STUB-<n>` (next 2; RMG-STUB-1 open: steps 4 to 15), decisions next 13. Registry: 62 of 128 positions taken (52, 53, 56 to 61 the rmg lane's `quirk.rmg` flags; -26's at T4d, the rest of the ledger's with their steps); `faithful`'s core hash `0x810a2c4eb5a0373f` (T4b).
- **Waiting on other lanes:** the writer's seven gaps (survey §3.3.3) before increment 7, in the adventure (1 to 6) and town (7) lanes' inboxes; the town lane's answer on a neutral main town (RMG-QUIRK-23).
- **Working agreement:** a checkpoint with the user per increment (decision 6); tokens first; read the executable in the cloud sandbox (`docs/exe/README.md`, "Setup"); a big task cut at a seam with the user, worker sub-agents allowed for its parts (2026-11-23); a couple of small tasks or one bigger per session (2026-11-24); T2c, T3c, T3d, T3e and T4b kept whole (decision 9; the user, 2026-11-26; decision 12).

## Recent sessions

### 2026-11-27 — T4c step 6's obstacle plan (`rmg/e14-t4c`, in progress)

A new chat; staged the oracle's set from the PC (67 files, 58 sums checked); #1210 had merged. The user kept T4c whole (worker sub-agents in turn, as T4b) and classed the Ownership 0 starts accepted for now, to compare with VCMI and HotA later. One oracle run (`corpus --phases`: 24 of 24 as `phases.txt`); `h3rmg.py plan --record` wrote `rmg/plan.txt` (each case's plan prefix of step 6, ending at a connection's first draw). **In progress:** worker 1 (lanes, walls, cells of no zone), then 2 (islets), 3 (joining, carving, sealing, RMG-QUIRK-5); the logs in `clients/out/rmg/` (this sandbox only).

### 2026-11-27 — T4b steps 4 and 5, the towns (`shared/e14-t4b`, #1210, `cross-lane`)

Same chat as T4a. Staged the oracle's set from the PC (67 files, 58 sums checked); the user kept T4b whole with worker sub-agents and agreed the `.msk` harvest cross-lane (decision 12; RMG-QUIRK-26 fix-optional). Three workers in turn: the objects' data (every line's `.msk`, the 232 classes' settings, `RMG_RULESET_HASH_VERSION` 5), the object model (`objects.rs`), steps 4 and 5 (`towns.rs`, two flags, `faithful`'s core hash re-pinned). The corpus equals the phase oracle to step 5 on 24 of 24 at the first run. Asked: the Ownership 0 starts' class, a look at RMG-QUIRK-25's fix.

### 2026-11-27 — T4a the items not read (`rmg/e14-t4a`, #1206)

A new chat; staged `Heroes3.exe` from the PC (SHA-256 checked); #1204 had merged, `master`'s CI green. Code reading only: the objects' masks through `objects.txt`'s reader and `msk_load`, `rmg_map_can_place`'s footprint, outline and ring, the null prototype's six callers, the islets' height map, the monoliths' pairing; VCMI 1.7.5's `ObjectTemplate` compared. Into the survey (§2.1 to §2.3, §3.2), `map.toml`, `structs.toml`, the ledger (two faults joined RMG-QUIRK-3 and -4; the one-way monoliths an open candidate). Found for T4b: the `.msk` data the packs lack.
