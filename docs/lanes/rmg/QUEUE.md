# RMG lane — QUEUE

Pre-refined tasks of the rmg lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E14-original-map-generator.md` (increments 2 to 7 cut 2026-11-24, RMG-Q2; increments 2 and 3 closed, increment 4 refined 2026-11-27, decision 11; T4a done 2026-11-27). Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`) and the increment's exit criteria (a) to (f) (the epic, "Increments 2 to 7"). The design: ADR-0029 (`docs/adr/0029-original-map-generator.md`). Each task's scope in full: the epic's "Increment 4's tasks".

- **T4b — steps 4 and 5, the towns.** Scope: RMG-TWN-001 to -005, RMG-OBJ-009 to -011; RMG-QUIRK-13, -23's towns' half, the Ownership 0 starts; RMG-QUIRK-25's fix designed, its flag registered (`cross-lane`). **Needs, found by T4a** (survey §2.1, "The template's masks as read"): each picture's `.msk` (width, height, colour mask) in the rmg packs, since the footprint's size and the passable mask come from it (a harvest from the sprite archives: `h3-harvest`, `cross-lane`, a re-hash); `ObjectTemplate`'s two terrain columns named as the generator uses them (it tests the fourth, `editor_terrains` today). Acceptance: the cells after steps 4 and 5 and the draws to them equal on the corpus. Status: `ready` (next).
- **T4c — step 6's obstacle plan.** Scope: RMG-CON-001 to -012, RMG-QUIRK-5; the painter's flush, water and rock branches against the oracle. Acceptance: its draws equal the oracle log's prefix of step 6 (an oracle run). Depends on: T4b. Status: `ready`.
- **T4d — step 6's connections.** Scope: RMG-CON-013 to -021; RMG-QUIRK-7 to -10, -11's guards' half, -19; 6LM10's self-connection; the one-way monoliths' shared channel (the ledger's open table, T4a: the user classifies it). Acceptance: the cells after step 6 and the draws to it equal. Depends on: T4c. Status: `ready`.
- **T4e — step 7, the junctions.** Scope: RMG-OBS-001, -002; RMG-QUIRK-1's form (decision 11); the increment's exit (a) to (e), then increment 4's checkpoint with the user. Depends on: T4d. Status: `ready`.

## Inbox

Nothing yet.
