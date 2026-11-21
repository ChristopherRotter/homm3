# RMG lane — QUEUE

Pre-refined tasks of the rmg lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E14-original-map-generator.md` (increment 1, refined 2026-11-21, RMG-Q1). Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

- **T1a1 — the generator's frame.** Read `random_map_generate` 0x5860E0, 0x54BF00, 0x54C090, the constructor 0x537B10 with its base 0x536070 (the seed), `GenerateMap` 0x549930's phases in order and `WriteMap` 0x54ABF0's outline; port H3API's `H3RMG` records into `docs/exe/structs.toml` and check them against the code. Out: `map.toml` entries, the survey's §1 (`docs/rules/surveys/E14-generator.md`: phases, records, files read). Staging: `Heroes3.exe` only.
- **T1a2 — every function placed.** The closure's 156 functions in 0x530EE0–0x54D0F0 named at least `inferred` and placed in a phase; the 250 outside classified; every `rand` and float site per phase; any clock or iteration limit. Out: `map.toml`, the survey's §2.
- **T1b — the oracle.** The original generating under Wine with `time()` fixed and the choices given; reproducible bytes; the reader reads each map; a per-phase dump if a debugger allows; the corpus's spec. Out: `docs/lanes/rmg/ORACLE.md`, a script, the corpus's hashes. Staging: the client lane's set (its HANDOFF, "Staging").
- **T1c — survey, ADR draft, cut, checkpoint.** The survey's rules and quirk candidates, the writer's gaps; the ADR draft (crate, seam, `rand`, floats, pack data, profiles, version); increments 2 to 6 cut with exit criteria; the checkpoint RMG-Q2 with the user.

## Inbox

Nothing yet.
