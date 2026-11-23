# RMG lane — QUEUE

Pre-refined tasks of the rmg lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E14-original-map-generator.md` (increment 1, refined 2026-11-21, RMG-Q1). Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

- **T1b — the oracle.** The original generating under Wine with `time()` fixed and the choices given; reproducible bytes; the reader reads each map; a per-phase dump if a debugger allows; the corpus's spec. Out: `docs/lanes/rmg/ORACLE.md`, a script, the corpus's hashes. Staging: the client lane's set (its HANDOFF, "Staging"). From T1a2 (survey §1.8, §2.6): the seed is printed in each map's description (a check of the fixed clock); generate with the plain executable (no HD mod: `Untitled.h3m` shows three differences, §2.5); two draws depend on memory (§2.2 step 2, §2.5 step 15): say whether the oracle's output repeats run to run and, if not, which zones or cells vary; the writer writes no 124-byte padding (whether `h3_map`'s reader accepts that: the adventure lane's inbox if not).
- **T1c — survey, ADR draft, cut, checkpoint.** The survey's rules and quirk candidates, the writer's gaps; the ADR draft (crate, seam, `rand`, floats, pack data, profiles, version); increments 2 to 6 cut with exit criteria; the checkpoint RMG-Q2 with the user.

## Inbox

Nothing yet.
