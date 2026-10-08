# Battle lane — QUEUE

Pre-refined tasks of the battle lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Increment 6 (the user, 2026-10-08): T7a in parts (T7a1, T7a2 done; T7a3 proposed); batch 2d is with the user and is folded in at the checkpoint after T7a.

### T7a3 — Bind and Acid Breath (E04b T7a3)

- **Scope:** the epic's T7a3; `docs/formats/installation.md` §7.20 (0x440500's Bind lists and Acid Breath without a test, 0x43EE10's release on a binder's move 0x4420A3, death 0x4441C7 and teleport 0x5A1B88, the pathfinder's no-movement test 0x4B2DE0/0x4B350B, 0x5A1E70, 0x440BC0's Rust Dragon case). Bind (fixed 255 rounds, no movement while bound, binder lists beside the stacks, released as above, ended by Dispel not Cure) and Acid Breath (permanent cumulative defense −3, only on a stack with defense above 0) on T7a1's hook without the spell's test (`ignores_immunity`); the Rust Dragon's 20 % extra damage of half a damage roll per creature, without a test (an ability of its own, its event); legal moves of a bound stack (attack in place only).
- **Acceptance:** the Definition of Done; rule tests (bind and release by move, death, teleport; Dispel ends it; a bound stack's legal commands; Acid Breath's floor at 0 and repetition; the extra damage).
- **Depends on:** T7a2 (done 2026-10-08); the user's go for the split (proposed with PR #12).
- **Status:** ready.
