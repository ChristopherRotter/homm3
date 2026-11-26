# RMG lane — QUEUE

Pre-refined tasks of the rmg lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E14-original-map-generator.md` (increments 2 to 7 cut 2026-11-24, RMG-Q2; increment 2 closed and increment 3 refined 2026-11-25, decision 10). Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`) and the increment's exit criteria (a) to (f) (the epic, "Increments 2 to 7"). The design: ADR-0029 (`docs/adr/0029-original-map-generator.md`). Each task's scope in full: the epic's "Increment 3's tasks"; every one needs the oracle's set staged from the PC (ORACLE.md).

- **T3d — step 2.** Scope: RMG-ZON-009 to -014; RMG-QUIRK-1's oracle model (its flag `quirk.rmg.zone_memory_model`, registered), -14 (`partition_units`), -3's step 2 cases; T3b's five corner cases (the ledger's open table). Acceptance: the cells after step 2, the zone generators after it (the oracle's `generators 2` lines, a `layout.txt`-like record to add) and the draws to it equal on the corpus. Depends on: T3c (done: the seam, `Trace`'s `StepState`, `phases.txt` per phase). Status: `ready` (next).
- **T3e — step 3 and the terrain painter.** Scope: RMG-TER-001 to -003, -005 to -018, RMG-QUIRK-21's painters' half. Acceptance: the cells after step 3 equal; the increment's exit (a) to (e); then increment 3's checkpoint with the user (the oracle model's form). Depends on: T3d. Status: `ready` after it.

## Inbox

Nothing yet.
