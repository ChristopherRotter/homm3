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

## T1a2 — every function placed (2026-11-23, increment 1)

- **The cut** (the user, 2026-11-23): "4 or 5 parts of roughly equal size if sensible", worker
  sub-agents in the same session, then the session finishes up. Five parts of about 22 kB each
  (the range's 255 function starts: 154 in the direct-call closure, 101 reached only through
  vtables, the C runtime's initializer table, the array constructor or unwind funclets):
  T1a2a the setup (§2.1), T1a2b steps 1 to 5 (§2.2), T1a2c step 6 (§2.3), T1a2d steps 7, 8 and
  10 to 13 (§2.4), T1a2e steps 14 and 15 and the writer, then the stream (§2.5, §2.6). One worker
  each, in order, each reading the earlier parts' sections; a common brief (the rules of
  `docs/exe/README.md`, an entries helper applied by script, a site entry per draw, a short
  report). Commits 9ff8d12, 361b758, 06e07c8, ad8f2e4, 826b036.
- **Result:** 254 of the 255 starts with a function entry (237 read, 17 inferred; the last a
  false start inside `rmg_zone_flood_fill`, a site entry); about 340 entries in all with the
  sites and tables (the object generators' 18 and the objects' 13 vtables, the guard tables,
  the layout's cosine and sine tables, the obstacle types); the records read and two added
  (`rmg_obstacle`, `rmg_treasure_pile`); the steps renamed from their reading (survey §1.5):
  `rmg_zones_lay_out`, `rmg_level_zones_partition`, `rmg_terrain_paint`, `rmg_zone_first_town`,
  `rmg_zone_towns_place`, `rmg_paths_and_connections`, `rmg_junction_zone_paths`,
  `rmg_mines_place`, `rmg_zone_paths_connect`, `rmg_zone_treasures_place`,
  `rmg_underground_rock_paint`, `rmg_map_shores_mark`, `rmg_obstacles_fill`, `rmg_roads_lay`,
  `rmg_rivers_lay`.
- **Corrections to T1a1:** the object generators are at +0x10F0 (H3API was right; +0x1100 holds
  the Keymaster's Tents); step 4 places one town per zone, not one of each kind; step 15 lays
  rivers (type 109 is the Water Wheel, not a shipyard); the 17-byte artifact block is AB's (SoD
  writes 18, as the reader expects); the writer writes no 124-byte padding.
- **What T1b and T1c must know** (survey §2.6): one stream, no clock but the seed; x87 only,
  every conversion truncating, with two f64 tables; one retry loop (step 10: six tries per pick,
  three per object), fixed iterations elsewhere, every other give-up silent; **two draws depend
  on memory** (step 2's uninitialised town-type bytes decide whether a zone generator's draw
  happens; step 15 reads one cell past the array at the map's last column); `Untitled.h3m`
  differs from the plain executable's writer in three points (a main town's type byte, 124
  trailing zero bytes, no colour named human in the description), likely the user's HD mod
  patching the game at run time: the oracle generates with the plain executable. Quirk
  candidates (unclassified) are listed per part (§2.1 to §2.5).
- **Agent decisions** (change freely): five equal parts rather than three by theme; each worker
  wrote its own section and entries and committed on the lane branch; the runner kept only the
  reports.

## T1b1 — the oracle (2026-11-23, increment 1)

- **Cut** by the user at the start (AskUserQuestion): T1b1 the oracle, T1b2 the phase oracle (the
  draw log and the step dumps), T1b2 later in the same chat on this sandbox's staged files.
- **Staged** (the PC's shell worked): the client lane's set, 64 files (the four LODs each in its
  own call, `shell-videos.vid` alone, the 19 others, the 40 non-campaign MP3s), the four manuals
  and three help files, and Complete's two generated maps; the LODs' and the executables' SHA-256
  as `docs/formats/installation.md` lists them. The VM restarted once (about 21:07): the setup's
  baseline was lost, CI's green run on `master` (7ad3b1d) stands for it; Wine and the run
  directory were kept.
- **Built:** `xdd.c` section 9 (`H3TIME`: `Heroes3.exe`'s `GetLocalTime` and `GetSystemTime`
  imports return a fixed date; `crt_time`'s one caller is the generator's seeding; the ack's
  seventeenth field, `h3drive.ACK_FIELDS` 17); `clients/capture/h3rmg.py` (`one`, `corpus`,
  `read`, `templates`) on `h3shell.Run`; `rmg/corpus.toml` (21 cases) and `rmg/kept.txt`; the
  runbook's "The random map's oracle" and two coordinates (System Options, Main Menu).
- **Observed** (ORACLE.md, "What it showed"): the seed printed is the date given; 2 to 3 s a
  generation; 21 of 21 the same from fresh starts in two corpus runs, in a second generation in
  one game and (3 of 3) after the Load Game menu first; 21 distinct digests. SoD's `rmg.txt` is
  read (the templates predicted by the survey's rule and the first `rand()` after `srand`: 21 of
  21 from SoD's table, 13 from AB's). One template name holds a line break ("Meeting⏎in Muzgob").
  The plain writer as read (no padding, main town types 0xFF, red human named). `h3_map` reads
  all 21 (the padding's absence accepted: no adventure inbox item); 20 import and validate,
  `l2-2p-c4-islands` refused by TWN-SET-001 (a main town not its player's own town).
- **The tab:** two levels at its first opening and again after a game (CLT's catalogue had not
  said which).
- **Not done (T1b2):** the draw log per site and the step dumps; whether 0x532A5B draws in step 2.
