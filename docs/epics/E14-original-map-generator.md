# E14 — The original's random map generator

Status: **in progress** since 2026-11-23; increments 1 and 2 **closed** (2026-11-24, RMG-Q2;
2026-11-25, decision 10; `E14-archive.md`); increments 3 to 7 cut, increment 3 refined (T3a to
T3e; T3a to T3d done); next T3e · Lane: **rmg** (opened
at this refinement) · Prerequisite of E30 (the custom generator) · Consumers: the client lane (the shell's Begin, CLT-STUB-12), the server lane (a game offered on a
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
   serialises; the exact seam is ADR-0029 (T1c3's draft).
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
   original's own phase order (done: decision 8).
8. **The checkpoint after increment 1** (the user, 2026-11-24, RMG-Q2): the ADR draft (now
   ADR-0029, `docs/adr/0029-original-map-generator.md`) confirmed, with its seam the experiment writer
   `h3_map::writer` (its seven gaps closed by their owners before increment 7, the game reading
   the written bytes; not the session's recommended writer of `RawMap`); `objects.txt`'s picture
   names harvested with the tables (Q13's names); the 18 flags in the game's profiles
   (`faithful` on, `strict-fixed` off); six increments, 2 to 7, in the run's order (below).
9. **T2c's start** (the user, 2026-11-25): T2c whole in one session, a sub-agent where useful; the
   registry's set widened (64 to 128 flags) in T2c's shared change, so the replays are re-hashed
   once for it and the flags; RMG-QUIRK-24 (a start zone without Ownership marks a phantom human
   start 7, found by T2c's reading) `fix-optional` with a flag.
10. **The checkpoint after increment 2** (the user, 2026-11-25): increment 2 closed; RMG-QUIRK-11
   and -24 stand on two readings (Complete's generator writes SoD maps only, so no oracle case can
   show -11; T3a adds corpus cases drawing 2SM2i(2) and 2SM0k for -24); increment 3 cut into T3a to
   T3e (below); its four flags (-1 `quirk.rmg.zone_memory_model`, -12 `terrain_towns_count`, -14
   `partition_units`, -21 `cosmetic`) registered at T3c in one re-hash, each read by the task that
   ports its step.
11. **The checkpoint after increment 3** (the user, 2026-11-27): increment 3 closed; RMG-QUIRK-1's
   form left open (the constants stay provisional until increment 4, where the water zones' town
   types reach their guards); T3b's clip of an off-map side harmless and accepted (no flag); a zone
   that fills nothing (7 land zones in the corpus) fix-optional, RMG-QUIRK-25, its fix designed in
   increment 4; increment 4 cut into T4a to T4e (below), as proposed.
12. **T4b and T4a's monoliths** (the user, 2026-11-27): T4b kept whole, worker sub-agents for its
   parts; the objects' `.msk` harvest a `cross-lane` change in its branch, as T3e's; the one-way
   monoliths' shared channel (T4a) fix-optional, **RMG-QUIRK-26**, its flag registered when T4d
   ports RMG-CON-017.
13. **T4c and the Ownership 0 starts** (the user, 2026-11-27): T4c kept whole, worker sub-agents
   in turn; the start zones with Ownership 0 (no town of their colour; T4b) accepted for now, every
   profile as the original, no flag, to be compared with VCMI's and HotA's generators later (the
   ledger's "Considered, not quirks").

**Agent decisions** (change freely):

- The generator's `rand` lives in `h3-kernel` (`CrtRand`, KER-RNG-004), not in `h3-rmg` (ADR-0029
  point 3: the third use after `h3-adventure`'s and `h3-heroes`' copies).
- The oracle's maps are never committed (original output, ADR-0009); their hashes and the
  choices that made them are.
- Sources, in order: the code reading (primary); the oracle (observation); `rmg.txt` (original
  data, harvested under Q13); H3API (names and record layouts, a pointer only); HotA's template
  documentation for the fields' meanings and its list of the original's bugs (pointers, never a
  rule's source); VCMI's generator is a different design and is not ported (its `rmg.txt`
  converter only for the fields' names).

## Increments 2 to 7 (cut at T1c3, 2026-11-24; RMG-Q2, decision 8)

Cut in the run's own order (decision 7): a missed draw shifts every later one and the phase
oracle compares after each step, so each increment ports the next steps of the run whole (the
sketch's "connections before objects" would port step 6 before the towns of steps 4 and 5 it
reads). Rule ids are survey §3.2's (provisional until an increment writes them); ledger entries
`docs/rules/quirks-rmg.md`; the design in ADR-0029 (`docs/adr/0029-original-map-generator.md`).
Every increment needs the oracle's set staged from the PC (the client lane's, ORACLE.md).

**Every increment's exit criteria**, beside the Definition of Done: (a) its rules in the catalog,
implemented, each with a test; (b) on the corpus's cases (24 since T3a) in `faithful`, the port's draws and its
cells after each of its steps equal the phase oracle's (`clients/capture/rmg/phases.txt`'s
digests in CI, the whole logs and dumps beside the oracle; a divergence named by step and draw);
(c) its ledger entries confirmed by (b) or re-read, each flag's `strict-fixed` behaviour tested,
its `exploit-fix` cases typed errors; (d) a property test: arbitrary settings, seeds and the packs'
tables run its steps without a panic; (e) its "not read" items (survey §3.2.15) read first, into
the survey; (f) the checkpoint with the user.

| Inc | Scope | Rules (survey §3.2) | Ledger | Not read, read first | Its own exit |
|---|---|---|---|---|---|
| 2 (closed) | **Templates, the stream, the start:** the crates, `CrtRand`, the three tables harvested, the rmg part, the settings, the template's choice, the positions | SET-001 to -009, TPL-001 to -008 (17) | -6; -11 (the tables' half); -22 | none | the 21 cases' kept and drawn templates (`kept.txt`'s names) and the draws before step 1 equal; the harvested tables equal the text tables (with the original data) |
| 3 (closed) | **Zones and terrain:** steps 1 to 3, the terrain painter (§2.7.1 to §2.7.3) | ZON-001 to -014, TER-001 to -003, TER-005 to -018 (31) | -1 (the measurement, the model); -3 (its steps' cases); -12; -14; -21 (the painter's) | ZON-009's triangulation, ZON-012's clip corners, ZON-013's fill tail, ZON-014's water test; whether a divide by zero reaches code 1 | first the harness records each step's draw count and digest and RMG-QUIRK-1's bytes (`phases.txt` again); the control word read; the cells after steps 1 to 3 (the terrain layer) equal |
| 4 | **Towns and connections:** steps 4 to 7 (the guards' creation, the gates, monoliths, shipyards and Border Guards) | TWN-001 to -005, OBJ-009 to -011, CON-001 to -021, OBS-001, -002 (31) | -1 (the form, decision 11); -3, -4, -5 (their cases); -7; -8; -9; -10; -11 (the guards' half); -13; -19; -23 (the towns' half); -25 (the fix); -26 (decision 12); the ledger's two open items (6LM10's self-connection, Ownership 0 starts) | OBJ-011's `rmg_map_can_place` branches (tents on water), CON-008's heightmap quadrants, CON-017's one-way monolith pairing, OBJ-008's null prototype, OBJ-009's masks | the cells after steps 4 to 7 equal |
| 5 | **Mines and treasures:** steps 8 to 10, the object generators | OBJ-001 to -008, MIN-001 to -006, CON-022, TRS-001 to -012 (27) | -3 (the density filter); -15; -18; -20 | the pile ring (§2.4) | the cells after steps 8 to 10 equal; the generators' constants in `packs/sod/rmg.toml` |
| 6 | **Rock, shores, obstacles, roads, rivers:** steps 11 to 15, the road and river painter (§2.7.4), the teams' draw | TER-004, OBS-003 to -007, RDR-001 to -014, WRT-004 (21) | -2; -17; -21 (the roads' and rivers') | OBS-006's depth grid, RDR-010's river strokes | the cells after steps 11 to 15 and the whole stream (`draws=`, `log=`) equal |
| 7 | **The file and the game:** the writer (`h3_map::generated`), the census, `strict-fixed`, a game on a generated map | WRT-001 to -003, WRT-005 to -010 (9) | -16; -21 (the description); -23 (the player block) | WRT-007's settings +0xC types | the uncompressed `.h3m` equal on 21 of 21 (`kept.txt`) through `h3_map::writer` (survey §3.3.3's seven gaps closed), the game from the bytes read and imported; a census over more seeds and choices (RoE and AB formats among them); `strict-fixed` over the census without a failure; a game set up on a generated map (CLI, `GameSetup`); the benchmark; the client's and server's inboxes; the closure |

**Increment 4's tasks** (refined at increment 3's checkpoint, decision 11; in this order, each
about one session, the oracle's set staged from the PC for each):

| Task | What | Out |
|---|---|---|
| T4a | Exit (e), the items not read, into the survey (code reading only): `rmg_map_can_place`'s outline, ring and water branches (OBJ-011; tents on water), the islets' heightmap quadrants (CON-008), the one-way monoliths' pairing (CON-017), the null prototype (OBJ-008), the objects' masks (OBJ-009) | the survey, the rules rewritten, `map.toml` |
| T4b | Steps 4 and 5, the towns (RMG-TWN-001 to -005), with the object placing they need (RMG-OBJ-009 to -011: `rmg_map_place_object`, `rmg_map_can_place`, the town prototypes); RMG-QUIRK-13, -23's towns' half, the open table's Ownership 0 starts; RMG-QUIRK-25's fix designed and its flag registered (`cross-lane`, a re-hash) | the cells after steps 4 and 5 and the draws to them equal |
| T4c | Step 6's obstacle plan (RMG-CON-001 to -012): the lanes, the jagged segments and branches, the walls, the cells of no zone, the islets (the terrain painter's flush: its water and rock branches against the oracle), the joining, the cost flood, the carving, the shores sealed; RMG-QUIRK-5 | its draws equal the oracle log's prefix of step 6 (an oracle run; step 6 has one dump, its cells checked with T4d's) |
| T4d | Step 6's connections (RMG-CON-013 to -021): by land, by shipyard, by gates, by monoliths, the guards' values, Border Guards and monsters; RMG-QUIRK-7 to -10, -11's guards' half, -19; the open table's 6LM10 self-connection | the cells after step 6 and the draws to it equal |
| T4e | Step 7, the junction zones (RMG-OBS-001, -002); RMG-QUIRK-1's form (decision 11); the increment's exit (a) to (e) | the cells after step 7 and the draws to it equal |

## Risks

- **Size:** about 160 KB of code to read and port draw for draw; a missed draw shifts every
  later one. Mitigation: the phase oracle (T1b) and comparing phase by phase.
- **The oracle:** the harness must run the original's generation (E27 M3 T5b recorded its
  failure without a `random_maps` folder) with its clock fixed; T1b finds the way or says
  what the fallback is (a debugger-set seed).
- **Floats:** x87 at the thread's 53-bit control word, so binary64 (ADR-0029 point 4);
  confirmed by T3a's memory record: 0x027F at the generator's start and at every step.
- **Time limits or retries** in the generation would make the output depend on the machine;
  T1a2 looks for them.
- **The writer's coverage:** seven gaps (survey §3.3.3) the adventure and town lanes close
  from their inboxes before increment 7 (RMG-Q2); one still open then is the rmg lane's,
  `cross-lane` with the user's consent.
- **The oracle model** (RMG-QUIRK-1): measured by T3a (ORACLE.md, "The memory record and the
  fixed environment"): the temporary zone's stack bytes are stable, the water zones' heap bytes
  follow the game's environment block's size (fixed for the oracle since); the model's form left open at
  increment 3's checkpoint (decision 11) for increment 4.
- **The registry's flags** (`h3_content::MAX_QUIRKS`): widened from 64 to 128 by T2c (decision 9);
  60 positions are taken; of the ledger's 17 `quirk.rmg.*` flags (ADR-0029 point 6, RMG-QUIRK-25
  since decision 11), increment 3's four at T3c (decision 10), the other 13 with their steps.

## Tasks done

The full records: `E14-log.md`. Increment 1's tasks (T1a1 to T1c3), increment 2's (T2a to T2c,
its checkpoint) and increment 3's (T3a to T3e, its checkpoint): `E14-archive.md`.

- Increment 3's checkpoint (2026-11-27, decision 11): increment 3 closed, increment 4 cut into T4a
  to T4e; log `## Increment 3's checkpoint`.
- T4a (2026-11-27): the items not read, read (the masks, the placing tests, the islets' height map,
  the monoliths' pairing, the null prototype); log `## T4a`.
- T4b (2026-11-27, whole, decision 12): steps 4 and 5, the towns, with the objects' data and
  model; RMG-QUIRK-13's and -25's flags; the corpus to step 5 equal, 24 of 24; log `## T4b`.

