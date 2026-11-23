# E14 — log

The record of E14's tasks: what each read, built and decided, beyond the one line in the epic file.

## T1a1 — the generator's frame (2026-11-23, increment 1)

- **Kept whole** (the agent's sizing, the user's rule of 2026-11-23: a couple of small tasks or one
  bigger one per session, a big one cut at a seam with the user): the frame is about 5 kB of
  code to read plus the records.
- **Staged:** `Heroes3.exe` (SHA-256 checked), `h3maped.exe`, `extracted.tar`, `maps.tar`, both
  `Rmg.txt`, the three manuals, `HEROES3.HLP`, the readme files, and the two maps found in
  Complete's `random_maps` (`Untitled.h3m` 144 × 144 × 2 of template 8MM6, `dbg.h3m` 72 × 72 ×
  2 with three templates only, its origin unknown); the repo cloned from GitHub (the PC's
  shell worked this time; read in the cloud sandbox as `docs/exe/README.md` says).
- **Read** (survey `docs/rules/surveys/E14-generator.md` §1): `random_map_generate` whole (the
  five Random draws and their order, the parameters, the codes 1 to 3 and their GENRLTXT rows);
  `rmg_parameters_init`, `rmg_run` (a catch-all returning 1), `rmg_generate_and_write` (the
  monster strength + 3 clamped, the codes); the base constructor (the seed after the map and
  the objects' vectors) and the constructor (water's Random drawn after the seed; the heroes'
  bans; the two limit tables); `rmg_objects_read` (the version filter, the prototypes, the
  monsters' sort); `rmg_generate_map` whole (the template draw, the positions, the 15 steps);
  steps 4, 8 and 15 and step 1 in part; `rmg_write_map` whole, `rmg_teams_draw` whole, the
  header's end.
- **Measured:** each step's closure inside the generator's range (functions, bytes, draw call
  sites) with a scratch script over `scripts/exe.py`'s `Code` (not committed): 44 draw sites in
  the 15 steps, 1 in the writer's header, the template's, water's; the steps' closures and the
  constructor's readers cover about 148 of the 156 functions.
- **Ported:** H3API's `H3RMG` records into `docs/exe/structs.toml` as twelve structs under our
  names (fields read where the frame showed them, H3API's as `guess` otherwise); differences
  from H3API in §1.7.
- **Map:** 46 entries written or rewritten (`random_map_generate` and the frame now `read`;
  0x54BF00 renamed `rmg_parameters_init`; the 15 steps under neutral `rmg_phase_NN` names, for
  T1a2 to rename from their reading; five sites of the dialog's draws, water's and the
  template's; four tables).
- **Found on the way** (survey §1.5, §1.6; quirk candidates for T1c, none classified): the
  computers' teams above the computers become the players; a neutral zone's town type drawn
  among four entries even past the -1 (re-read in T1a2); the writer's 156 hero bytes before the
  rumours' count (the same zero bytes); a 17-byte artifact block where the reader's SoD block
  is 18 (T1b's map shows which); a generated map records its template and seed in its
  description (for T1b's oracle).
- **Agent decisions** (change freely): the steps' names are neutral until read; the structs take
  our names with `h3api` tags; the closures' sizes count a function for the first step that
  reaches it.
