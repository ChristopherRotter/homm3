# RMG lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`.

## Current state

- **E14** (the original's random map generator, ported to give the same map per seed): in progress since 2026-11-23. **Increment 1 closed** (2026-11-24, RMG-Q2; `docs/epics/E14-archive.md`): the survey `docs/rules/surveys/E14-generator.md`, the ledger `docs/rules/quirks-rmg.md`, the oracle and the phase oracle `ORACLE.md`, **ADR-0029** (`docs/adr/0029-original-map-generator.md`, numbered by T2a). **Increments 2 to 7 cut** in the run's order (the epic's table, decision 8); increment 2 refined (T2a to T2c). **T2a done** (2026-11-24): `h3-rmg` and `h3-pack-rmg` exist empty, `h3_kernel::rng::CrtRand` (KER-RNG-004), the catalog context `RMG`.
- **Next:** **T2b** (`QUEUE.md`; `cross-lane`, `h3-harvest`): the three tables into `packs/ab/rmg.toml` and `packs/sod/rmg.toml`, `h3-pack-rmg`'s parser, the rmg part. It needs `H3bitmap.lod`, `H3ab_bmp.lod` (or `extracted.tar`'s six tables) staged from the PC. T2c (the settings and the template) follows; increment 3 needs the oracle's set staged from the PC (a new chat stages it).
- **Open pull requests:** `shared/e14-t2a` (T2a, `cross-lane`). Watch it until it merges.
- **Owns** (`lanes.toml`): `crates/h3-rmg/`, `crates/h3-pack-rmg/` (`h3-map`'s `src/generated*` joins at increment 7), the packs' `rmg.toml`, the RMG rules (`docs/rules/catalog/rmg.toml`, made by T2c's first rules) and quirks, E14's epic, surveys and experiments, this folder.
- **Versions:** none yet (`RMG_GENERATOR_VERSION`, `RMG_RULESET_HASH_VERSION` come with T2b, T2c: ADR-0029 point 7). `CrtRand` has none (its sequence is the original's).
- **Ids:** questions `RMG-Q<n>` (next 3; none open), quirks `RMG-QUIRK-<n>` (next 24; two candidates open in the ledger), stubs `RMG-STUB-<n>` (next 1), decisions next 9.
- **Waiting on other lanes:** the writer's seven gaps (survey §3.3.3) before increment 7, in the adventure (1 to 6) and town (7) lanes' inboxes; the town lane's answer on a neutral main town (RMG-QUIRK-23).
- **Working agreement:** a checkpoint with the user per increment (decision 6); tokens first; read the executable in the cloud sandbox (`docs/exe/README.md`, "Setup"); a big task cut at a seam with the user, worker sub-agents allowed for its parts (2026-11-23); a couple of small tasks or one bigger per session (2026-11-24).

## Recent sessions

### 2026-11-24 — T2a the crates, `CrtRand`, ADR-0029 (shared, `cross-lane`)

A new chat; staged from the PC only `Heroes3.exe` (its `master` behind GitHub's; the clone GitHub's). Kept whole. The two crates empty (`h3-game → h3-rmg` forbidden), `CrtRand` with the code reading's vectors (`crt_srand` 0x617E4F, `crt_rand` 0x617E5C, a thread's state 1 at 0x61D2FB): the phase oracle's draw logs were never kept, so the oracle checks the stream from T2c on. The domain row, the catalog context `RMG`, ADR-0029 numbered with ADR-0003's history line; the adventure lane told `TileRand` and `OriginalDraws` may wrap `CrtRand`.

### 2026-11-24 — T1c3 the ADR draft, the cut, RMG-Q2 (increment 1 closed)

A new chat; staged from the PC both `Heroes3.exe` and the six tables (no oracle run). The draft (eight points: crates, seam, `CrtRand` in the kernel, binary64 floats and integer square roots, pack data, the game's profiles, versions, the trace) and the cut (increments 2 to 7 in the run's order) written, then asked: the user chose the experiment writer with its seven gaps closed over the recommended `RawMap` writer (the draft and both inbox notes rewritten), the picture names harvested, the game's profiles, six increments. Increment 1 archived; T2a to T2c queued. The VM restarted once (the setup's baseline lost, the files kept).

### 2026-11-24 — T1c2 survey §3 and the ledger (no cut, the user)

A new chat: the client lane's set staged from the PC (66 files; the SoD install's three tables are Complete's). Three workers wrote survey §3 as fragments (§3.1 the tables, §3.2 136 rules by phase, §3.3 the objects, the reader and the writer, with Wine and the corpus); the runner classified about 60 candidates with the user into RMG-QUIRK-1 to -23 and merged. TWN-SET-001's refusal is Dragons' neutral main town (RMG-QUIRK-23, the town lane asked); the writer's gaps in the adventure lane's inbox; `m2-4p`'s first map varied once (ORACLE.md).
