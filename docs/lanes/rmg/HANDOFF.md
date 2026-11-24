# RMG lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`.

## Current state

- **E14** (the original's random map generator, ported to give the same map per seed): in progress since 2026-11-23. **Increment 1 closed** (2026-11-24, RMG-Q2; `docs/epics/E14-archive.md`): the survey `docs/rules/surveys/E14-generator.md`, the ledger `docs/rules/quirks-rmg.md`, the oracle and the phase oracle `ORACLE.md`, the ADR draft `adr-drafts/h3-rmg-generator.md` (confirmed; numbered by T2a). **Increments 2 to 7 cut** in the run's order (the epic's table, decision 8); increment 2 refined (T2a to T2c). No engine code yet.
- **Next:** **T2a** (`QUEUE.md`): the `shared/` change: `h3-rmg` and `h3-pack-rmg` created, `h3_kernel::rng::CrtRand`, the domain context, the catalog context `RMG`, the ADR numbered. T2b (the harvest, `cross-lane`) and T2c (the settings and the template) follow. Increment 3 needs the oracle's set staged from the PC (a new chat stages it).
- **Open pull requests:** `rmg/e14-t1c3` (T1c3) and **#1155** (`shared/e14-increment-1`, the README's E14 row). Watch them until they merge.
- **Owns** (`lanes.toml`): `crates/h3-rmg/` (created by T2a, a shared change; `h3-pack-rmg` and `h3-map`'s `src/generated*` join it there and at increment 7), the packs' `rmg.toml`, the RMG rules and quirks, E14's epic, surveys and experiments, this folder.
- **Versions:** none yet (`RMG_GENERATOR_VERSION`, `RMG_RULESET_HASH_VERSION` come with T2b, T2c: ADR draft point 7).
- **Ids:** questions `RMG-Q<n>` (next 3; none open), quirks `RMG-QUIRK-<n>` (next 24; two candidates open in the ledger), stubs `RMG-STUB-<n>` (next 1), decisions next 9.
- **Waiting on other lanes:** the writer's seven gaps (survey §3.3.3) before increment 7, in the adventure (1 to 6) and town (7) lanes' inboxes; the town lane's answer on a neutral main town (RMG-QUIRK-23).
- **Working agreement:** a checkpoint with the user per increment (decision 6); tokens first; read the executable in the cloud sandbox (`docs/exe/README.md`, "Setup"); a big task cut at a seam with the user, worker sub-agents allowed for its parts (2026-11-23); a couple of small tasks or one bigger per session (2026-11-24).

## Recent sessions

### 2026-11-24 — T1c3 the ADR draft, the cut, RMG-Q2 (increment 1 closed)

A new chat; staged from the PC both `Heroes3.exe` and the six tables (no oracle run). The draft (eight points: crates, seam, `CrtRand` in the kernel, binary64 floats and integer square roots, pack data, the game's profiles, versions, the trace) and the cut (increments 2 to 7 in the run's order) written, then asked: the user chose the experiment writer with its seven gaps closed over the recommended `RawMap` writer (the draft and both inbox notes rewritten), the picture names harvested, the game's profiles, six increments. Increment 1 archived; T2a to T2c queued. The VM restarted once (the setup's baseline lost, the files kept).

### 2026-11-24 — T1c2 survey §3 and the ledger (no cut, the user)

A new chat: the client lane's set staged from the PC (66 files; the SoD install's three tables are Complete's). Three workers wrote survey §3 as fragments (§3.1 the tables, §3.2 136 rules by phase, §3.3 the objects, the reader and the writer, with Wine and the corpus); the runner classified about 60 candidates with the user into RMG-QUIRK-1 to -23 and merged. TWN-SET-001's refusal is Dragons' neutral main town (RMG-QUIRK-23, the town lane asked); the writer's gaps in the adventure lane's inbox; `m2-4p`'s first map varied once (ORACLE.md).

### 2026-11-24 — T1c1 the painters (T1c cut in three by the user)

A new chat: the client lane's set staged from the PC (61 files); the PC's repo folder was behind GitHub, so the clone is GitHub's. T1c1 read in thirds (the runner one, two workers one after the other): the terrain painter (picture sets, the row-major point sets of pinched cells, the last pass's transition rule over the whole map) and the road and river painter (one-cell strokes, the shape by the neighbours); survey §2.7, 148 map entries. Checked with three corpus cases run again: every terrain, road and river cell's picture in the step dumps as the rules give. The VM restarted once (a worker's report lost, its files kept).
