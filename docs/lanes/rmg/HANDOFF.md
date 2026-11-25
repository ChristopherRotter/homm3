# RMG lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`.

## Current state

- **E14** (the original's random map generator, ported to give the same map per seed): in progress since 2026-11-23. **Increment 1 closed** (2026-11-24, RMG-Q2; `docs/epics/E14-archive.md`): the survey `docs/rules/surveys/E14-generator.md`, the ledger `docs/rules/quirks-rmg.md`, the oracle and the phase oracle `ORACLE.md`, **ADR-0029** (`docs/adr/0029-original-map-generator.md`). **Increments 2 to 7 cut** in the run's order (the epic's table, decision 8); increment 2 refined (T2a to T2c). **T2a done** (2026-11-24): the crates, `CrtRand`, the catalog context `RMG`. **T2b done** (2026-11-25): the rmg part `h3_rmg::RmgRuleset`, `h3-pack-rmg`'s parser, `packs/ab/rmg.toml` and `packs/sod/rmg.toml` harvested (format: `packs/README.md`, "The map generator's tables"), `quirk.content.rmg_8xm8_zone_ids` with 8XM8's gated overrides (RMG-QUIRK-22), the `original_data` check.
- **Next:** **T2c** (`QUEUE.md`): the settings, `resolve_random`, the templates kept and drawn, `generate`'s frame, the catalog's first rules; checked on the 21 cases, so it needs the oracle's set staged from the PC (the client lane's, ORACLE.md; a new chat stages it). Then increment 2's checkpoint, with the registry's capacity (52 of 64 positions; the epic's risks).
- **Open pull requests:** **#1166** (`shared/e14-t2b`, T2b, `cross-lane`) and **#1170** (`shared/ci-jobs`, the CI rework's phase 1: the verify as parallel jobs, 60 minutes per job at most, never raised; phases 2 and 3 planned in `docs/05-testing.md`, "CI"). Both green; watch them until they merge.
- **Owns** (`lanes.toml`): `crates/h3-rmg/`, `crates/h3-pack-rmg/` (`h3-map`'s `src/generated*` joins at increment 7), the packs' `rmg.toml`, the RMG rules (`docs/rules/catalog/rmg.toml`, made by T2c's first rules) and quirks, E14's epic, surveys and experiments, this folder.
- **Versions:** `RMG_RULESET_HASH_VERSION` 1 (T2b; `h3-rmg`); `RMG_GENERATOR_VERSION` comes with T2c (ADR-0029 point 7). `CrtRand` has none (its sequence is the original's).
- **Ids:** questions `RMG-Q<n>` (next 3; none open), quirks `RMG-QUIRK-<n>` (next 24; two candidates open in the ledger), stubs `RMG-STUB-<n>` (next 1), decisions next 9.
- **Waiting on other lanes:** the writer's seven gaps (survey §3.3.3) before increment 7, in the adventure (1 to 6) and town (7) lanes' inboxes; the town lane's answer on a neutral main town (RMG-QUIRK-23).
- **Working agreement:** a checkpoint with the user per increment (decision 6); tokens first; read the executable in the cloud sandbox (`docs/exe/README.md`, "Setup"); a big task cut at a seam with the user, worker sub-agents allowed for its parts (2026-11-23); a couple of small tasks or one bigger per session (2026-11-24).

## Recent sessions

### 2026-11-25 — T2b the three tables, the rmg part, 8XM8's flag (shared, `cross-lane`)

A new chat; staged from the PC both LODs, `Heroes3.exe`, `extracted.tar` and the six tables before its link closed. Asked for a cut (the part, the harvest, the flag); the user: no cut, a worker for a part. Built the part, the parser, the harvest and the check; the flag registered and set in the profiles, the golden replays re-hashed by one worker. #1166's CI: cancelled once at the 55-minute timeout (a slow runner), then failed on `h3-map`'s view property test (a random case, failing on `master` too: the seed in the adventure lane's inbox), green the third time. The user then asked for the CI rework (60 minutes per job, never raised): phase 1, #1170, built by one worker.

### 2026-11-24 — T2a the crates, `CrtRand`, ADR-0029 (shared, `cross-lane`)

A new chat; staged from the PC only `Heroes3.exe` (its `master` behind GitHub's; the clone GitHub's). Kept whole. The two crates empty (`h3-game → h3-rmg` forbidden), `CrtRand` with the code reading's vectors (`crt_srand` 0x617E4F, `crt_rand` 0x617E5C, a thread's state 1 at 0x61D2FB): the phase oracle's draw logs were never kept, so the oracle checks the stream from T2c on. The domain row, the catalog context `RMG`, ADR-0029 numbered with ADR-0003's history line; the adventure lane told `TileRand` and `OriginalDraws` may wrap `CrtRand`.

### 2026-11-24 — T1c3 the ADR draft, the cut, RMG-Q2 (increment 1 closed)

A new chat; staged from the PC both `Heroes3.exe` and the six tables (no oracle run). The draft (eight points: crates, seam, `CrtRand` in the kernel, binary64 floats and integer square roots, pack data, the game's profiles, versions, the trace) and the cut (increments 2 to 7 in the run's order) written, then asked: the user chose the experiment writer with its seven gaps closed over the recommended `RawMap` writer (the draft and both inbox notes rewritten), the picture names harvested, the game's profiles, six increments. Increment 1 archived; T2a to T2c queued. The VM restarted once (the setup's baseline lost, the files kept).
