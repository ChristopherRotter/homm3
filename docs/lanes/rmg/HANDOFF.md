# RMG lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`.

## Current state

- **E14** (the original's random map generator, ported to give the same map per seed): in progress since 2026-11-23. Increment 1 (the reading's map, the oracle, the survey): **T1a1, T1a2, T1b1, T1b2, T1c1 and T1c2 done** (survey `docs/rules/surveys/E14-generator.md` §1, §2 (§2.7 the painters), §3 the tables, the rules by phase, the objects and the writer; the ledger `docs/rules/quirks-rmg.md`; the oracle and the phase oracle `ORACLE.md`; log `docs/epics/E14-log.md`). Increments 2 to 6 sketched. No engine code yet.
- **Next:** **T1c3** (`QUEUE.md`): the ADR draft (crate, seam, `rand`, floats, pack data, the profiles and the ledger's 18 flags, version), increments 2 to 6 cut with exit criteria (survey §3.2's rules and "not read" list; RMG-QUIRK-1's measurement at step 2's port), the checkpoint RMG-Q2 with the user. Reading needs `Heroes3.exe` alone; an oracle run the client lane's set (a new chat stages it from the PC).
- **Open pull requests:** `rmg/e14-t1c2` (T1c2). Watch it until it merges.
- **Owns** (`lanes.toml`): `crates/h3-rmg/` (created at increment 2, a shared change), the packs' `rmg.toml`, the RMG rules and quirks, E14's epic, surveys and experiments, this folder.
- **Ids:** questions `RMG-Q<n>` (next 2; none open), quirks `RMG-QUIRK-<n>` (next 24; two candidates open in the ledger), stubs `RMG-STUB-<n>` (next 1), decisions next 8.
- **Working agreement:** a checkpoint with the user per increment (decision 6); tokens first; read the executable in the cloud sandbox (`docs/exe/README.md`, "Setup"); a big task cut at a seam with the user, worker sub-agents allowed for its parts (2026-11-23).

## Recent sessions

### 2026-11-24 — T1c2 survey §3 and the ledger (no cut, the user)

A new chat: the client lane's set staged from the PC (66 files; the SoD install's three tables are Complete's). Three workers wrote survey §3 as fragments (§3.1 the tables, §3.2 136 rules by phase, §3.3 the objects, the reader and the writer, with Wine and the corpus); the runner classified about 60 candidates with the user into RMG-QUIRK-1 to -23 and merged. TWN-SET-001's refusal is Dragons' neutral main town (RMG-QUIRK-23, the town lane asked); the writer's gaps in the adventure lane's inbox; `m2-4p`'s first map varied once (ORACLE.md).

### 2026-11-24 — T1c1 the painters (T1c cut in three by the user)

A new chat: the client lane's set staged from the PC (61 files); the PC's repo folder was behind GitHub, so the clone is GitHub's. T1c1 read in thirds (the runner one, two workers one after the other): the terrain painter (picture sets, the row-major point sets of pinched cells, the last pass's transition rule over the whole map) and the road and river painter (one-cell strokes, the shape by the neighbours); survey §2.7, 148 map entries. Checked with three corpus cases run again: every terrain, road and river cell's picture in the step dumps as the rules give. The VM restarted once (a worker's report lost, its files kept).

### 2026-11-23/24 — T1b2 the phase oracle

A new chat: the client lane's set staged from the PC again (60 files). `xdd.dll`'s section 10 (`H3RMG`) logs every draw of the generation from `rand`'s tail with its call site and dumps the cells after each step, its hooks on their own stack; `h3rmg.py --phases`, `phases`. The corpus reproducible, no map changed by the hooks; a third of the draws come from the terrain and road painters outside the range (survey §2.6 lacks them); 0x532A5B draws in step 2 in 18 of 21, and `l1-4p-b`'s second map in one game differs by it, moved by the environment's size (T1b1's "21 of 21" was its sandbox's).
