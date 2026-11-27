# RMG lane — QUEUE

Pre-refined tasks of the rmg lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E14-original-map-generator.md` (increments 2 to 7 cut 2026-11-24, RMG-Q2; increments 2 and 3 closed, increment 4 refined 2026-11-27, decision 11; T4a to T4c done 2026-11-27). Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`) and the increment's exit criteria (a) to (f) (the epic, "Increments 2 to 7"). The design: ADR-0029 (`docs/adr/0029-original-map-generator.md`). Each task's scope in full: the epic's "Increment 4's tasks".

- **T4d — step 6's connections.** Scope: RMG-CON-013 to -021; RMG-QUIRK-7 to -10, -11's guards' half, -19; 6LM10's self-connection; RMG-QUIRK-26 (the one-way monoliths' shared channel, fix-optional, decision 12): both sides and its flag. Acceptance: the cells after step 6 and the draws to it equal. Also, left by T4c (`plan/join.rs`, RMG-STUB-1): the carve's pending-Border-Guard branch (site 0x546154; step 9's joining needs it too) and its flag-set form (no 3×3 cleared); `PlanInput.objects` then `&mut`; the floods' outputs the connections read: survey §2.3, "Step 6's plan as ported (T4c)". Depends on: T4c (done). Status: `ready` (next).
- **T4e — step 7, the junctions.** Scope: RMG-OBS-001, -002; RMG-QUIRK-1's form (decision 11); the increment's exit (a) to (e), then increment 4's checkpoint with the user. Depends on: T4d. Also, found in T4b: step 1 gives no town choice for a start position of 8 or more, where the original reads past the colours (RMG-QUIRK-4's class: type it). Status: `ready`.

## Inbox

- **The corpus tests' time** (found in T4c, `TRAPS.md`, "The port's code"): step 6's seal makes the islands maps' generation slow in debug and coverage builds (the transition pass per flush over the whole level). Two ways, either measured on the corpus: the tests that check steps up to 5 stop there (a `Trace` hook), or the transition pass visits only the cells whose 5×5 changed since the last pass, its draws and writes unchanged (the corpus proves it). Before T4d if CI's coverage job nears its 60 minutes. Status: `ready`.
