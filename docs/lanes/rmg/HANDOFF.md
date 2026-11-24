# RMG lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`.

## Current state

- **E14** (the original's random map generator, ported to give the same map per seed): in progress since 2026-11-23. Increment 1 (the reading's map, the oracle, the survey): **T1a1, T1a2, T1b1, T1b2 and T1c1 done** (survey `docs/rules/surveys/E14-generator.md` §1, §2, §2.7 the painters; the oracle and the phase oracle `ORACLE.md`; log `docs/epics/E14-log.md`). T1c cut by the user into T1c1 (done), T1c2, T1c3. Increments 2 to 6 sketched. No engine code yet.
- **Next:** **T1c2** (`QUEUE.md`): survey §3 (the template format and both tables' differences, each phase's rules, the writer's gaps) and the quirk ledger, every candidate classified with the user (the memory-dependent ones among them). Then T1c3 (ADR draft, the cut of increments 2 to 6, RMG-Q2). Reading needs `Heroes3.exe` alone; an oracle run the client lane's set (a new chat stages it from the PC).
- **Open pull requests:** `rmg/e14-t1c1` (T1c1). Watch it until it merges.
- **Owns** (`lanes.toml`): `crates/h3-rmg/` (created at increment 2, a shared change), the packs' `rmg.toml`, the RMG rules and quirks, E14's epic, surveys and experiments, this folder.
- **Ids:** questions `RMG-Q<n>` (next 2; none open), quirks `RMG-QUIRK-<n>` (next 1; ledger `docs/rules/quirks-rmg.md`, made with the first; candidates in the survey's §1, §2 (§2.7 too) and ORACLE.md, unclassified until T1c2), stubs `RMG-STUB-<n>` (next 1), decisions next 8.
- **Working agreement:** a checkpoint with the user per increment (decision 6); tokens first; read the executable in the cloud sandbox (`docs/exe/README.md`, "Setup"); a big task cut at a seam with the user, worker sub-agents allowed for its parts (2026-11-23).

## Recent sessions

### 2026-11-24 — T1c1 the painters (T1c cut in three by the user)

A new chat: the client lane's set staged from the PC (61 files); the PC's repo folder was behind GitHub, so the clone is GitHub's. T1c1 read in thirds (the runner one, two workers one after the other): the terrain painter (picture sets, the row-major point sets of pinched cells, the last pass's transition rule over the whole map) and the road and river painter (one-cell strokes, the shape by the neighbours); survey §2.7, 148 map entries. Checked with three corpus cases run again: every terrain, road and river cell's picture in the step dumps as the rules give. The VM restarted once (a worker's report lost, its files kept).

### 2026-11-23/24 — T1b2 the phase oracle

A new chat: the client lane's set staged from the PC again (60 files). `xdd.dll`'s section 10 (`H3RMG`) logs every draw of the generation from `rand`'s tail with its call site and dumps the cells after each step, its hooks on their own stack; `h3rmg.py --phases`, `phases`. The corpus reproducible, no map changed by the hooks; a third of the draws come from the terrain and road painters outside the range (survey §2.6 lacks them); 0x532A5B draws in step 2 in 18 of 21, and `l1-4p-b`'s second map in one game differs by it, moved by the environment's size (T1b1's "21 of 21" was its sandbox's).

### 2026-11-23 — T1b1 the oracle (T1b cut by the user)

Staged the client lane's set (the PC's shell worked). `xdd.dll` fixes `time()`'s date (`H3TIME`), `h3rmg.py` sets the Random Map tab and keeps the map: 21 cases, each the same from fresh starts, in a second generation in one game and after another history (no memory-dependent variation under Wine); SoD's `rmg.txt` is read (the templates predicted 21 of 21); the plain writer as read; `h3_map` reads all, one refused at import (TWN-SET-001, for T1c). The VM restarted once (the baseline lost, CI's run on `master` stands for it).
