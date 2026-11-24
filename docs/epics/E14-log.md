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

## T1b2 — the phase oracle (2026-11-23 and 24, increment 1)

- **Sandbox:** a new chat (T1b1's sandbox was gone): the client lane's set staged again from the
  PC (60 files: the four LODs, `shell-videos.vid`, the executable and DLLs, sounds, videos,
  `maps.tar`, `extracted.tar`, the saves, the 40 non-campaign MP3s; `Heroes3.exe`'s SHA-256
  checked); stacked on `rmg/e14-t1b1` (#1138, open and green).
- **Built:** `xdd.c` section 10 (`H3RMG`): `rand`'s tail (0x617E70) jumps to `rmg_rand_tail`,
  which logs the generator thread's draws (the call site, the value) between its `srand` and
  `rmg_write_map`'s return; 18 calls through `rmg_thunk_jmp` and `rmg_thunk_call` (the srand,
  `rmg_generate_map`'s call, the 15 steps', `rmg_write_map`'s) mark the phases and dump the
  cells (format 1: per cell the objects' count and first three indices, then its words +0x10 to
  +0x2C; the objects' table: template index, type, subtype, position; the places past the
  third); the ack's eighteenth field (19 hooks; `h3drive.ACK_FIELDS` 18). `h3rmg.py`: `one` and
  `corpus --phases` (the log's end awaited, digests against `rmg/phases.txt`, `--again`'s cell
  differences named and the memory bits told apart), `phases` (the logs per phase against §2.6,
  `--runs`). Five map entries for the painters' draw functions (`inferred`).
- **Agent decisions (change freely):** `rand` hooked at its tail, not at its call sites: every
  draw of the stream is seen, those of callees outside the range too (the call-site table of
  section 3 would have missed them). The hooks run on their own 256 kB stack and read the thread
  id from `fs:0x24`: a first version ran its C code on the game's stack, where step 2 reads
  uninitialised bytes; its first maps and `l1-4p-b`'s draw count were the same, the own stack
  makes it hold by construction.
  The dumps replace each cell's object vector (pointers) by indices; `--phases` is opt-in (the
  plain oracle stays T1b1's).
- **Observed** (ORACLE.md, "The phase oracle"): the corpus with `--phases --again --twice` in
  about 20 minutes; 21 of 21 first maps as `kept.txt`, logs and dumps the same from fresh starts;
  21 454 to 500 780 draws a map; 1.03 of the corpus's 3.13 million draws from the terrain painter
  (steps 3, 6, 11) and the road and river painter (14, 15), outside §2.6; listed sites never
  drawn (water Random, step 4's far sites, 0x546154 in steps 6 and 9); 0x532A5B in step 2 in 18
  of 21. `l1-4p-b`'s second map in one game differs (with and without the hooks; 5 draws of
  0x532A5B instead of 6), the same with a 40-character variable added to the environment,
  different with 2 000. The cell bits `rmg_map_cell_reset` leaves (+0x14, +0x18, +0x24 bits
  30-31, +0x28 bit 24, +0x2C bits 1-31) differ in S and M second maps and change no map.
- **Checks:** the harness's own check (`harness-check.toml`); `check-exe-map`; the scoped verify.
- **Not done (T1c):** the painters' draws read; the memory-dependent quirks classified.

## T1c1 — the painters outside the range (2026-11-24, increment 1)

- **The cut** (the user, at the start; AskUserQuestion): T1c in three, T1c1 the painters, T1c2
  survey §3 and the quirk ledger with the user, T1c3 the ADR draft, the cut of increments 2 to 6
  and RMG-Q2; T1c1 "3 parts, 2 subagents, you read one third and finalize". Measured first: the
  terrain painter about 8 000 instructions (21.6 kB), the road and river painter about 1 000
  (2.5 kB). T1c1a the runner (the terrain painter's frame and picture sets, the road and river
  painter; §2.7.1, §2.7.4), T1c1b a worker (adding cells, the point sets; §2.7.2, about 3 250
  instructions), T1c1c a worker after it (the pinched cells and the last pass; §2.7.3, about
  2 700); a common brief (`docs/exe/README.md`'s rules, entries as a fragment the runner merged
  by address, a section and a short report); the runner sent the frame's facts to T1c1b while it
  read.
- **Sandbox:** a new chat: the client lane's set staged from the PC (61 files with Complete's two
  generated maps; SHA-256 against the PC's); the PC's repo folder was behind GitHub (its HANDOFF
  still at T1a1), so the clone is GitHub's. The VM restarted once (T1c1b's report lost, its files
  kept; the Wine setup run again).
- **Read** (survey §2.7): the painter's record and the handle's loop; ten picture sets by terrain
  (shape and flag tables, the flagged share, rock's variants); two point sets ordered by (y, x),
  unique keys: cells of the painter's terrain pinched west-east or north-south (water and rock:
  a split ring) and foreign neighbours to re-test; the pinched cells resolved by a fixed priority
  and the ring's gaps filled cheapest first; the last pass over the whole map: border classes
  (none, a dirt border, a sand border by the sets' byte +4), ten rule groups under four
  mirrorings, two refinements, slot 5 or 4 per cell; the road and river painter: one-cell
  strokes, a visit draws the cell and its same-type neighbours, the shape by the four orthogonal
  neighbours and the corners' diagonals, rivers without end pieces or diagonal corners.
- **Checks:** three corpus cases run again with `--phases` (digests as kept); the painters' sites
  per step; a script over the step dumps: every terrain cell after steps 3, 6, 11 and 15, every
  road cell after 14 and every river cell after 15 as the rules give (0 exceptions);
  `scripts/exe.py check` and `cited`; the scoped verify.
- **For T1c2** (quirk candidates in §2.7.1 to §2.7.4, unclassified): the cell words' fill takes the
  map's address (never read before a load); `queue_neighbours`' else-if per axis; `set_cell`
  never erasing its own cell from set 1; every painted cell re-drawn; the last pass over the whole
  map per finish; group 3's inverted mirroring; shapes a set lacks (water 24, rock 16 and 24: a
  zero divisor if reached); `road_river_rect_clear`'s short right column; a new road cell of an
  unmirrored shape 4 kept without a draw.
- **Agent decisions** (change freely): the painters' names `terrain_painter_*`,
  `terrain_picture_*`, `rock_picture_*`, `road_river_*`; `terrain_picture_draw_b` and `_kept_b`
  renamed `rock_picture_draw` and `_kept`; the check scripts stay out of the repo (increment 3's
  port compares with the log and dumps itself).
