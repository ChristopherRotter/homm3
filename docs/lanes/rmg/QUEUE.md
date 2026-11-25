# RMG lane — QUEUE

Pre-refined tasks of the rmg lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E14-original-map-generator.md` (increments 2 to 7 cut 2026-11-24, RMG-Q2; increment 2 closed and increment 3 refined 2026-11-25, decision 10). Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`) and the increment's exit criteria (a) to (f) (the epic, "Increments 2 to 7"). The design: ADR-0029 (`docs/adr/0029-original-map-generator.md`). Each task's scope in full: the epic's "Increment 3's tasks"; every one needs the oracle's set staged from the PC (ORACLE.md).

- **T3a — the phase oracle's per-step record.** Scope: `phases.txt` with each step's draw count and log digest; RMG-QUIRK-1's unwritten zone bytes per case and what moves them; the x87 control word; corpus cases drawing 2SM2i(2) (three players) and 2SM0k for RMG-QUIRK-24. Acceptance: the corpus recorded again with the new format and cases (`--twice` reproducible), ORACLE.md and the ledger's -1 and -24 updated, the T2c corpus test covering the new cases. Depends on: nothing. Status: `ready`.
- **T3b — the items not read** (exit (e)). Scope: the triangulation's insertion and finish, the fill's tail 0x53D640, the water zones' distance test, the borders' clip corners, a division by zero and code 1 (RMG-QUIRK-3). Acceptance: survey §2.2 and §3.2.4 complete for steps 1 to 3 (ZON-009, -012 to -014 rewritten), `map.toml` entries. Depends on: nothing (after T3a in the order). Status: `ready`.
- **T3c — step 1, the cell record, the comparison seam.** Scope: RMG-ZON-001 to -008, the layout tables, the dump format 1 and the per-step digests against `phases.txt` in CI, RMG-QUIRK-12, -3's step 1 case; the four flags in one re-hash (`cross-lane`, decision 10; skill `homm3-rerecord-replays`). Acceptance: the cells after step 1 and the draws to it equal on the corpus. Depends on: T3a (the per-step counts), T3b. Status: `ready` after them.
- **T3d — step 2.** Scope: RMG-ZON-009 to -014; RMG-QUIRK-1's oracle model, -14, -3's step 2 cases. Acceptance: the cells after step 2 and the draws to it equal on the corpus. Depends on: T3c. Status: `ready` after it.
- **T3e — step 3 and the terrain painter.** Scope: RMG-TER-001 to -003, -005 to -018, RMG-QUIRK-21's painters' half. Acceptance: the cells after step 3 equal; the increment's exit (a) to (e); then increment 3's checkpoint with the user (the oracle model's form). Depends on: T3d. Status: `ready` after it.

## Inbox

Nothing yet.
