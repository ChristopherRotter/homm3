# E14 — The original's random map generator

Status: **in progress** since 2026-11-23; increment 1 **closed** (2026-11-24, RMG-Q2;
`E14-archive.md`); increments 2 to 7 cut, increment 2 refined; T2a done; next T2b · Lane: **rmg** (opened
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
implemented, each with a test; (b) on the 21 corpus cases in `faithful`, the port's draws and its
cells after each of its steps equal the phase oracle's (`clients/capture/rmg/phases.txt`'s
digests in CI, the whole logs and dumps beside the oracle; a divergence named by step and draw);
(c) its ledger entries confirmed by (b) or re-read, each flag's `strict-fixed` behaviour tested,
its `exploit-fix` cases typed errors; (d) a property test: arbitrary settings, seeds and the packs'
tables run its steps without a panic; (e) its "not read" items (survey §3.2.15) read first, into
the survey; (f) the checkpoint with the user.

| Inc | Scope | Rules (survey §3.2) | Ledger | Not read, read first | Its own exit |
|---|---|---|---|---|---|
| 2 | **Templates, the stream, the start:** the crates, `CrtRand`, the three tables harvested, the rmg part, the settings, the template's choice, the positions | SET-001 to -009, TPL-001 to -008 (17) | -6; -11 (the tables' half); -22 | none | the 21 cases' kept and drawn templates (`kept.txt`'s names) and the draws before step 1 equal; the harvested tables equal the text tables (with the original data) |
| 3 | **Zones and terrain:** steps 1 to 3, the terrain painter (§2.7.1 to §2.7.3) | ZON-001 to -014, TER-001 to -003, TER-005 to -018 (31) | -1 (the measurement, the model); -3 (its steps' cases); -12; -14; -21 (the painter's) | ZON-009's triangulation, ZON-012's clip corners, ZON-013's fill tail, ZON-014's water test; whether a divide by zero reaches code 1 | first the harness records each step's draw count and digest and RMG-QUIRK-1's bytes (`phases.txt` again); the control word read; the cells after steps 1 to 3 (the terrain layer) equal |
| 4 | **Towns and connections:** steps 4 to 7 (the guards' creation, the gates, monoliths, shipyards and Border Guards) | TWN-001 to -005, OBJ-009 to -011, CON-001 to -021, OBS-001, -002 (31) | -3, -4, -5 (their cases); -7; -8; -9; -10; -11 (the guards' half); -13; -19; -23 (the towns' half); the ledger's two open items (6LM10's self-connection, Ownership 0 starts) | OBJ-011's `rmg_map_can_place` branches (tents on water), CON-008's heightmap quadrants, CON-017's one-way monolith pairing, OBJ-008's null prototype, OBJ-009's masks | the cells after steps 4 to 7 equal |
| 5 | **Mines and treasures:** steps 8 to 10, the object generators | OBJ-001 to -008, MIN-001 to -006, CON-022, TRS-001 to -012 (27) | -3 (the density filter); -15; -18; -20 | the pile ring (§2.4) | the cells after steps 8 to 10 equal; the generators' constants in `packs/sod/rmg.toml` |
| 6 | **Rock, shores, obstacles, roads, rivers:** steps 11 to 15, the road and river painter (§2.7.4), the teams' draw | TER-004, OBS-003 to -007, RDR-001 to -014, WRT-004 (21) | -2; -17; -21 (the roads' and rivers') | OBS-006's depth grid, RDR-010's river strokes | the cells after steps 11 to 15 and the whole stream (`draws=`, `log=`) equal |
| 7 | **The file and the game:** the writer (`h3_map::generated`), the census, `strict-fixed`, a game on a generated map | WRT-001 to -003, WRT-005 to -010 (9) | -16; -21 (the description); -23 (the player block) | WRT-007's settings +0xC types | the uncompressed `.h3m` equal on 21 of 21 (`kept.txt`) through `h3_map::writer` (survey §3.3.3's seven gaps closed), the game from the bytes read and imported; a census over more seeds and choices (RoE and AB formats among them); `strict-fixed` over the census without a failure; a game set up on a generated map (CLI, `GameSetup`); the benchmark; the client's and server's inboxes; the closure |

**Increment 2's tasks** (refined with the cut):

| Task | What | Out |
|---|---|---|
| T2a | `shared/`: `h3-rmg` and `h3-pack-rmg` created (`Cargo.toml`, `arch.toml`, `lanes.toml`), `h3_kernel::rng::CrtRand` with test vectors, the context in `docs/03-domain.md`, the catalog context `RMG`, the ADR numbered | the crates, the kernel's generator, the ADR in `docs/adr/` |
| T2b | `cross-lane` (`h3-harvest`): `rmg.txt`, `objects.txt` and `rand_trn.txt` into `packs/ab/rmg.toml` and `packs/sod/rmg.toml`; `h3-pack-rmg`'s parser; the rmg part's compiler and hash; 8XM8's gated override and its flag; `packs/README.md` | the packs' `rmg.toml`, the rmg part |
| T2c | The settings and their checks (RMG-SET-001 to -009, RMG-QUIRK-6), `resolve_random`, the templates kept and drawn and the positions (RMG-TPL-001 to -008, RMG-QUIRK-11's flag), `Trace`; checked on the 21 cases | `h3-rmg`'s start, the increment's exit |

## Risks

- **Size:** about 160 KB of code to read and port draw for draw; a missed draw shifts every
  later one. Mitigation: the phase oracle (T1b) and comparing phase by phase.
- **The oracle:** the harness must run the original's generation (E27 M3 T5b recorded its
  failure without a `random_maps` folder) with its clock fixed; T1b finds the way or says
  what the fallback is (a debugger-set seed).
- **Floats:** x87 at the thread's 53-bit control word, so binary64 (ADR-0029 point 4);
  increment 3 reads the control word in the oracle to confirm.
- **Time limits or retries** in the generation would make the output depend on the machine;
  T1a2 looks for them.
- **The writer's coverage:** seven gaps (survey §3.3.3) the adventure and town lanes close
  from their inboxes before increment 7 (RMG-Q2); one still open then is the rmg lane's,
  `cross-lane` with the user's consent.
- **The oracle model** (RMG-QUIRK-1): a first map varied once (ORACLE.md); increment 3 measures
  what moves the stack bytes before the model is fixed.

## Tasks done

The full records: `E14-log.md`. Increment 1's tasks (T1a1 to T1c3): `E14-archive.md`.

- **T2a** (2026-11-24, `shared/e14-t2a`, `cross-lane`): `h3-rmg` and `h3-pack-rmg` created empty
  (`h3-game → h3-rmg` forbidden), `h3_kernel::rng::CrtRand` (KER-RNG-004, the code reading's
  vectors), the context *map generation*, the catalog context `RMG`, ADR-0029 numbered.
