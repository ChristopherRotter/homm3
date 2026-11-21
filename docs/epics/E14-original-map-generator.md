# E14 — The original's random map generator

Status: **refined** 2026-11-21 (RMG-Q1); increment 1 refined, increments 2 to 6 sketched; next
T1a1 · Lane: **rmg** (opened at this refinement) · Prerequisite of E30 (the custom generator) ·
Consumers: the client lane (the shell's Begin, CLT-STUB-12), the server lane (a game offered on a
random map), the adventure lane (the import of a generated map, the writer)

## Goal

A port of the random map generator the original added with AB (the scenario screen's Random Map
tab; GOG Complete's `Heroes3.exe`, SoD's code), so that the same template table, choices and
seed give **the same map as the original, byte for byte**. It is the base the custom,
objective-driven generator (E30) builds on: the template model, the zone placement, the
objects' values and the guards' strengths read from the original rather than guessed.

## What the first look found (2026-11-21, the refinement; `scripts/exe.py`, H3API's port)

- **Entry:** `scenario_begin` 0x58BCE0 → `random_map_generate` 0x5860E0 (its Random choices
  drawn with `crt_rand` at 0x586168 before the generation; a worker thread and a progress
  window) → 0x54BF00 and 0x54C090 (unnamed). H3API names the generator's record
  `H3RmgRandomMapGenerator`: its constructor 0x537B10 (12 arguments: width, height,
  underground, the humans, their teams, the computers, …), `GenerateMap` 0x549930,
  `WriteMap` 0x54ABF0, `H3RmgMap::GetMapItem` 0x5378E0 (H3API's `H3RMG/` headers hold the
  records; pointers where to read, never a rule's source).
- **The seed:** the generator's base constructor 0x536070 calls the C runtime's `time()`
  (0x6198E0, `GetLocalTime`/`GetSystemTime`), keeps it at +4 and seeds `srand` with it
  (0x536148): one seed per generated map, in seconds. The generation then draws from the C
  runtime's `rand` (MSVC's linear congruential generator).
- **Size:** the call closure of 0x54BF00 and 0x54C090 is 704 functions: **156 in
  0x530EE0–0x54D0F0 (about 98 KB of code)**, the generator's own; 250 other game functions
  (about 60 KB: text tables, `template_file_read`, `msk_load`, object tables, the archives);
  298 of the C runtime and library. 76 x87 instructions in the generator's range, with
  `crt_sqrt` and `crt_ftol`.
- **Its data:** `rmg.txt` in both bitmap archives (`H3ab_bmp.lod` the AB copy, `H3bitmap.lod`
  the SoD copy; `docs/formats/installation.md` lists it among the 23 tables that differ), the
  objects' table, masks.

## Decisions at the refinement (the user, 2026-11-21, RMG-Q1)

1. **Same map per seed.** The port reproduces the original's output for the same template
   table, choices and seed: the C runtime's `rand` ported, every draw in the original's
   order, the floats as the original computes them. The oracle is the original generating
   under Wine with its clock fixed, compared `.h3m` byte for byte. This amends the vision's
   non-goal (`docs/01-vision.md`) for the generator only; the game's own generator stays
   logic-equivalent (ADR-0003).
2. **A lane of its own**, `rmg` (code `RMG`), in parallel with the adventure lane, owning a new
   pure crate `h3-rmg` (created at increment 2, a shared change) and the packs' `rmg.toml`. The
   generator writes no file: it returns a map the adventure lane's writer (`h3_map::writer`)
   serialises; the exact seam is T1c's ADR draft.
3. **Scope:** SoD's generator with both template tables harvested (`ab` from `H3ab_bmp`, `sod`
   from `H3bitmap`); what AB's executable does differently is E11's. The epic ends at a game set
   up on a generated map in the engine (the CLI too); the shell's Begin (CLT-STUB-12 lifted) and
   the server's game offer go to those lanes' inboxes.
4. **The original's bugs: a quirk profile now.** `faithful` (the default; the one the oracle
   checks) reproduces them; `fixed` corrects them. Each bug is an entry `RMG-QUIRK-<n>` in the
   lane's ledger with its classification asked; a crash, a hang or a corrupt map is fixed in
   both profiles (AGENTS.md).
5. **Two epics.** E14 is the port; E30, the custom objective-driven generator (zone balance, few
   teleports, …), is refined after E14's reading and built on `h3-rmg`. The user's existing
   generator project is E30's input, named at its refinement.
6. **A checkpoint with the user per increment** (the domain lanes' way): the cut, the seams and
   the ADR drafts confirmed at each.
7. **The increments** as below; only increment 1 is refined. The reading may re-cut 2 to 6 to the
   original's own phase order.

**Agent decisions** (change freely):

- The generator's `rand` lives in `h3-rmg`, not in `h3-kernel`: it is the generator's own
  stream, not the game's (if T1c finds a reason for the kernel, its ADR draft says so).
- The oracle's maps are never committed (original output, ADR-0009); their hashes and the
  choices that made them are.
- Sources, in order: the code reading (primary); the oracle (observation); `rmg.txt` (original
  data, harvested under Q13); H3API (names and record layouts, a pointer only); HotA's template
  documentation for the fields' meanings and its list of the original's bugs (pointers, never a
  rule's source); VCMI's generator is a different design and is not ported (its `rmg.txt`
  converter only for the fields' names).

## Increment 1 — the reading's map, the oracle, the survey (refined)

| Task | What | Out |
|---|---|---|
| T1a1 | The generator's frame: `random_map_generate`, 0x54BF00, 0x54C090, the constructor and its settings, `GenerateMap`'s phases in order, `WriteMap`'s outline; H3API's `H3RMG` records ported into `structs.toml` and checked against the code | `map.toml` entries; the survey's §1 (phases, records, files read) |
| T1a2 | Every function of the closure inside 0x530EE0–0x54D0F0 named at least `inferred` and placed in a phase; the 250 outside classified (mapped already, table and object helpers, I/O); every `rand` draw site and every float site listed per phase; any clock or iteration limit found (a wall-clock limit would break the oracle) | `map.toml`; the survey's §2 (per phase: reads, draws, floats, size) |
| T1b | The oracle: the original generating under Wine with `time()` fixed (the capture harness, `clients/capture/`; a fake clock or a breakpoint), the choices given (no Random); twice the same seed gives the same bytes, another seed other bytes; `h3_map`'s reader reads each map; if a debugger can stop at the phases' ends, a state dump per phase (an oracle for increments 3 to 5 before the file is complete); the corpus's spec (sizes, levels, players, teams, water, monsters, seeds) | `docs/lanes/rmg/ORACLE.md`; a script; the corpus's hashes |
| T1c | The survey `docs/rules/surveys/E14-generator.md` (the template format and both tables' differences, each phase's rules, the objects placed against the engine's and the writer's coverage, the quirk candidates); the ADR draft (the crate, the seam to `h3-map`, the `rand`, the floats under `kernel::num`, the pack data, the profiles, the generator's version); the cut of increments 2 to 6 with exit criteria; the checkpoint RMG-Q2 | survey, ADR draft, this file |

**Exit criteria:** (1) every function of the generator's range in the map, at least `inferred`,
with its phase; (2) every draw and float site listed; (3) the oracle generates a map for a given
seed and choices, reproducibly, and the reader reads it; (4) the survey and the ADR draft
written; (5) increments 2 to 6 cut with exit criteria and confirmed at the checkpoint.

## Later increments (sketched)

2. **Templates and the start:** both `rmg.txt` harvested into `packs/ab/rmg.toml` and
   `packs/sod/rmg.toml` (`h3-harvest`, `cross-lane`), `h3-rmg` created, the C runtime's `rand`,
   the settings and the template's choice: the same template as the original for each oracle
   case.
3. **Zones and terrain:** the zones placed and grown, their terrain; the terrain layer equal to
   the oracle's.
4. **Connections, roads, guards, obstacles:** the connections made (gates, monoliths,
   subterranean gates), the roads, the border guards' strengths, the obstacles.
5. **Objects:** towns, mines, dwellings, treasures and their guards, the monsters, by value and
   density.
6. **The file and the game:** the map written byte for byte (`h3_map::writer`'s gaps the
   adventure lane's inbox), a census over many seeds and choices against the oracle, the
   `fixed` profile, a game set up on a generated map (CLI, `GameSetup`), a benchmark; the
   inboxes of the client and server lanes; the closure.

## Risks

- **Size:** about 160 KB of code to read and port draw for draw; a missed draw shifts every
  later one. Mitigation: the phase oracle (T1b) and comparing phase by phase.
- **The oracle:** the harness must run the original's generation (E27 M3 T5b recorded its
  failure without a `random_maps` folder) with its clock fixed; T1b finds the way or says
  what the fallback is (a debugger-set seed).
- **Floats:** x87 with the original's precision control; T1a2 lists the sites, T1c decides
  their port under ADR-0003.
- **Time limits or retries** in the generation would make the output depend on the machine;
  T1a2 looks for them.
- **The writer's coverage:** objects the generator places that `h3_map::writer` cannot yet
  write; T1c lists them for the adventure lane.

## Tasks done

None yet.
