# RMG lane — QUEUE

Pre-refined tasks of the rmg lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E14-original-map-generator.md` (increments 2 to 7 cut 2026-11-24, RMG-Q2; increment 2 closed and increment 3 refined 2026-11-25, decision 10). Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`) and the increment's exit criteria (a) to (f) (the epic, "Increments 2 to 7"). The design: ADR-0029 (`docs/adr/0029-original-map-generator.md`). Each task's scope in full: the epic's "Increment 3's tasks"; every one needs the oracle's set staged from the PC (ORACLE.md).

- **T3c — step 1, the cell record, the comparison seam.** Scope: RMG-ZON-001 to -008, the layout tables, the dump format 1 and the per-step digests against `phases.txt` in CI, RMG-QUIRK-12, -3's step 1 case; the four flags in one re-hash (`cross-lane`, decision 10; skill `homm3-rerecord-replays`). Acceptance: the cells after step 1 and the draws to it equal on the corpus. Depends on: T3a and T3b (done; the per-step counts in `phases.txt`, step 2's readings in survey §2.2). Status: `ready` (next).
- **T3d — step 2.** Scope: RMG-ZON-009 to -014; RMG-QUIRK-1's oracle model, -14, -3's step 2 cases; T3b's five corner cases (the ledger's open table). Acceptance: the cells after step 2 and the draws to it equal on the corpus. Depends on: T3c. Status: `ready` after it.
- **T3e — step 3 and the terrain painter.** Scope: RMG-TER-001 to -003, -005 to -018, RMG-QUIRK-21's painters' half. Acceptance: the cells after step 3 equal; the increment's exit (a) to (e); then increment 3's checkpoint with the user (the oracle model's form). Depends on: T3d. Status: `ready` after it.

## Inbox

Nothing yet.
