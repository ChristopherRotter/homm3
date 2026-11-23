# RMG lane — QUEUE

Pre-refined tasks of the rmg lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E14-original-map-generator.md` (increment 1, refined 2026-11-21, RMG-Q1). Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

- **T1b2 — the phase oracle** (T1b's second part, cut by the user 2026-11-23; T1b1 done: `ORACLE.md`). Every draw of a generation logged with its site and value: `xdd.dll`'s logged sites (`xdd.c` section 4) over survey §2.6's sites (the C library's `rand` calls of the generator's range and the writer's), written per case beside the map; the log's order against §2.6 (which sites draw per step, how often); whether 0x532A5B draws in step 2 (the memory-dependent draw). The generator's cells (`rmg_map`, 0x30 bytes each) dumped after each of the 15 steps of `rmg_generate_map` (a hook on each step's call), so increments 3 to 5 compare before the file is whole. Out: `h3rmg.py` options, ORACLE.md's "The phase oracle", the corpus's logs' digests. Staging: the client lane's set (this chat's sandbox has it).
- **T1c — survey, ADR draft, cut, checkpoint.** The survey's rules and quirk candidates, the writer's gaps; the ADR draft (crate, seam, `rand`, floats, pack data, profiles, version); increments 2 to 6 cut with exit criteria; the checkpoint RMG-Q2 with the user.

## Inbox

Nothing yet.
