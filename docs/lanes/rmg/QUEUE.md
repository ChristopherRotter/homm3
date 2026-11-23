# RMG lane — QUEUE

Pre-refined tasks of the rmg lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E14-original-map-generator.md` (increment 1, refined 2026-11-21, RMG-Q1). Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

- **T1a2 — every function placed**, cut by the user (2026-11-23) into five parts of about 22 kB each, each a worker sub-agent in the session that did T1a1. Common to each: its functions named at least `inferred` and placed in their step (`map.toml`), the game functions they call classified (mapped already, table and object helpers, I/O, the C runtime), every `rand` and float site listed, any clock, retry or iteration limit; out: the survey's §2.x (`docs/rules/surveys/E14-generator.md`). Staging: `Heroes3.exe` only. The steps and names: the survey's §1.5.
  - **T1a2a — the setup** (§2.1): `rmg_objects_read`'s rest, `rmg_rand_trn_read`, `rmg_templates_read` with its zones' reader, `rmg_object_generators_create` and the object generators' vtable methods, `rmg_map_init`, the destructor (about 22 kB).
  - **T1a2b — steps 1 to 5** (§2.2): the zones, the per-level layout, step 3, the towns, step 5 (about 22 kB).
  - **T1a2c — step 6** (§2.3), with step 9's function (about 24 kB).
  - **T1a2d — steps 7, 8 and 10 to 13** (§2.4): the junctions, the mines, step 10, the levels, steps 12 and 13 (about 19 kB).
  - **T1a2e — steps 14 and 15 and the writer** (§2.5): the roads, the shipyards, the header, the cells, the templates, the placed objects' vtable methods, and every function of the range still without an entry (about 22 kB); then §2.6, the stream's draws in order.
- **T1b — the oracle.** The original generating under Wine with `time()` fixed and the choices given; reproducible bytes; the reader reads each map; a per-phase dump if a debugger allows; the corpus's spec. Out: `docs/lanes/rmg/ORACLE.md`, a script, the corpus's hashes. Staging: the client lane's set (its HANDOFF, "Staging").
- **T1c — survey, ADR draft, cut, checkpoint.** The survey's rules and quirk candidates, the writer's gaps; the ADR draft (crate, seam, `rand`, floats, pack data, profiles, version); increments 2 to 6 cut with exit criteria; the checkpoint RMG-Q2 with the user.

## Inbox

Nothing yet.
