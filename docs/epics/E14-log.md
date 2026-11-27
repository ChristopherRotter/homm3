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

## T1c2 — survey §3 and the quirk ledger (2026-11-24, increment 1)

- **No cut** (the user, at the start: "no cut, use subagents efficiently"). The runner kept the
  ledger with the user; three worker sub-agents wrote §3's parts as fragments the runner merged:
  §3.1 (the tables, small) and §3.2 (the rules, large) at once, §3.3 (the objects, the reader and
  the writer, with the oracle's corpus) after §3.1.
- **Sandbox:** a new chat: the client lane's set staged from the PC (66 files with Complete's two
  generated maps; SHA-256 as `installation.md` lists them); the PC's repo folder 12 commits behind
  GitHub, so the clone is GitHub's. The standalone SoD installation (`D:\Heroes3\HoMM 3 SoD`) has
  another `Heroes3.exe` and `H3bitmap.lod`, but its `rmg.txt`, `objects.txt` and `rand_trn.txt`
  are Complete's byte for byte (extracted into `original-data/sod-install/`).
- **§3.1** (the template format; AB's and SoD's tables): the row kinds and columns as the code
  reads them (all nine connection columns: §1.4's "read in part" corrected), the space rule, what
  a harvester keeps (row order, templates by index: two are named 6LM10); AB's file a byte prefix
  of SoD's (SoD adds Golems Aplenty, Monk's Retreat, Rise of Phoenix; the 50 common templates
  identical); counts (AB 50 templates, 616 zones, 833 connections; SoD 53, 646, 869); six table
  candidates (8XM8's decimal ids, never-kept connections, a self-connection, Ownership 0 starts,
  owned treasure zones, lesser).
- **§3.2** (each phase's rules): 136 rules at one catalog entry each, provisional ids
  `RMG-<AREA>-<NNN>` in 12 areas (SET 9, TPL 8, OBJ 11, ZON 14, TER 18, TWN 5, CON 22, OBS 7, MIN
  6, TRS 12, RDR 14, WRT 10), their draws, the quirk candidates marked; the phases' dependencies,
  the oracle's comparison points and the "not read" list for T1c3. Five stale points of §1 and §2
  corrected (§1.6's template order, §1.8's site count and floats, §2.1's "Not read", §2.6's step 1
  row).
- **The ledger** `docs/rules/quirks-rmg.md` (the user, by AskUserQuestion): about 60 candidates in
  five groups. Memory-dependent: the oracle model in `faithful`, zeroed in `fixed` (RMG-QUIRK-1,
  -2); the cell bits and the cell-word fill not quirks. Faults: `exploit-fix`, a typed failure
  (RMG-QUIRK-3 to -6). Map-content bugs: `fix-optional`, a flag each (RMG-QUIRK-7 to -20; the
  crossed Border Guards' keys too, RMG-QUIRK-7). Cosmetic: one flag for all (RMG-QUIRK-21, the
  user's own option). 8XM8's ids: a data quirk with a flag (RMG-QUIRK-22; the fix 6, 6, 7, 7, 8, 8
  from the table's own pattern). The rest "considered, not quirks"; two open until read (a
  self-connection, Ownership 0 starts).
- **§3.3** (the objects placed, the reader and the writer): the classes the generator places (its
  generators by settings group, the steps' objects, the writers' slots); Wine set up and the
  corpus run again (20 of 21 as kept; `m2-4p` another map once, as kept run alone right after:
  ORACLE.md); a census of the 21 maps (47 458 objects of 115 classes, 69 % obstacles; no Border
  Guard or tent: no corpus template has a border-guard connection); the header, players and
  bodies as written; the import's and the writer's coverage; the writer's seven gaps for
  increment 6 (the adventure lane's inbox; the main town's two, the town lane's). TWN-SET-001's
  refusal of `l2-2p-c4-islands`: the entrance is the format's convention (553 of 553 shipped
  main towns), the cause Dragons' zone 32 (a neutral town as purple's main town), which the
  original starts (observed: the town stays neutral, purple's hero on its entrance):
  RMG-QUIRK-23, no flag (the user), and a question in the town lane's inbox.
- **Checks:** `check-rules`, `check-exe-map` (every cited address mapped), the scoped verify.
- **Agent decisions** (change freely): §3.2's area codes and provisional ids (the increments
  assign the catalog's); the ledger groups the faults by kind (RMG-QUIRK-3 to -6) and the cosmetic
  ones in one entry with the user's one flag; the scratch scripts (table parser, census) stay out
  of the repo.

## T1c3 — the ADR draft, the cut, the checkpoint RMG-Q2 (2026-11-24, increment 1)

- **No cut** (the user: "a couple of small tasks or a bigger one at a time"; this one small, the
  runner alone, no workers). Staged from the PC: both `Heroes3.exe` (Complete's and the standalone
  SoD's, the latter a 345 KB launcher) and the six tables of `original-data/extracted/`
  (`Rmg.txt`, `Objects.txt`, `Rand_trn.txt` of `H3bitmap` and `H3ab_bmp`); no oracle run.
- **Findings for the draft:** AB's `Rand_trn.txt` equals SoD's; SoD's `Objects.txt` holds 1 326
  template lines against AB's 1 221 (105 added, three changed); AB's `Rmg.txt` is a byte prefix of
  SoD's (T1c2). The C runtime's `rand` is already ported twice (`h3_adventure::split::TileRand`,
  `h3_heroes::level_up::OriginalDraws`), so the draft puts the generator's in the kernel
  (`CrtRand`; the epic's agent decision changed). The game's control word is 0x27F (53 bits,
  `docs/formats/installation.md`), so the generator's x87 sites are binary64: `trunc(sqrt(n))` of
  an integer is the integer square root for every `n < 2^52` (the domain's at most 82 944), the
  layout products go through `kernel::num::Real` with the tables as `Real(k) / Real(10 000)`.
  `phases.txt` keeps each step's cells' digest but one digest of the whole draw log: increment 3
  first records each step's draw count and digest.
- **The draft** (`docs/lanes/rmg/adr-drafts/h3-rmg-generator.md`, eight points): the crates
  (`h3-rmg` a domain logic crate on the kernel and the core content, `h3-pack-rmg`, an rmg-lane
  module `h3_map::generated`), the seam (`generate` → `GeneratedMap` in the writer's order;
  `resolve_random` for the dialog's Random choices), the stream, the floats, the pack data (`ab`
  AB's three tables, `sod` SoD's additions and the executable's constants; 8XM8's gated override
  in the rmg part's compiler; the part outside `GameRuleset`; no generator without `sod`), the
  profiles (the 18 flags in the game's registry), the versions (`RMG_GENERATOR_VERSION`,
  `RMG_RULESET_HASH_VERSION`, the provenance) and the trace (draws with their sites, the cells in
  the dump format 1, CI on the digests).
- **The cut** (the epic's table): increments 2 to 7 in the run's order, 17, 31, 31, 27, 21 and 9
  of §3.2's 136 rules; each increment's ledger entries and "not read" items; the shared exit
  criteria (a) to (f) and each increment's own; increment 2's tasks T2a to T2c.
- **The checkpoint RMG-Q2** (one round of four): the seam **the experiment writer with its seven
  gaps closed** (the session recommended a writer of `RawMap`; the draft rewritten to the user's
  choice, the adventure and town lanes' inbox notes updated: their gaps before increment 7, the
  rmg lane `cross-lane` for any left then); the picture names harvested; the game's profiles; six
  increments. Increment 1 closed: its section and tasks moved to `E14-archive.md`.

## T2a — the crates, `CrtRand`, the contexts, ADR-0029 (2026-11-24, increment 2)

- **Kept whole** (a small shared change, the user's "a couple of small tasks or a bigger one").
  Branch `shared/e14-t2a`, `cross-lane` (the rmg lane's crates and docs ride along, as E09 T4b's).
  Staged from the PC: Complete's `Heroes3.exe` alone (the code reading). The PC's repo folder was
  behind GitHub (its `master` at 7b4f02c); the clone is GitHub's.
- **The crates:** `h3-rmg` (domain, logic: the lint header, `no_std`; `arch.toml` allows the kernel
  and the core content, the manifest none yet, as `h3-towns`' skeleton) and `h3-pack-rmg`
  (adapter: the kernel, the core content, `h3-rmg`, `h3-pack`, serde and toml; proptest for exit
  criterion (d)'s property test over the packs' tables, which live there, not in `h3-rmg`). The
  edge `h3-game → h3-rmg` is a `forbidden_edge` (point 2: the game reads the bytes); the edge
  `h3-map → h3-rmg` waits for increment 7. `lanes.toml` gives the rmg lane `crates/h3-pack-rmg/`.
- **`CrtRand`** (`h3_kernel::rng`, KER-RNG-004): `new(seed)` (`srand`), `rand()`, `state()`,
  `MAX`. Read again: `crt_srand` 0x617E4F sets the thread data's +0x14; `crt_rand` 0x617E5C
  multiplies it by 0x343FD, adds 0x269EC3, stores it (0x617E70) and returns bits 16 to 30; the
  thread data (0x61D303, `_getptd`) starts at 1 (0x61D2F0, at 0x61D2FB). The three entries in
  `docs/exe/map.toml`. **The test vectors are the code reading's** (srand of 1, 0, 2^32 − 1 and
  `s1-2p`'s 1 000 000 000, six draws and the state each, computed from the formula): the phase
  oracle's draw logs were never kept (`clients/out/rmg/` lived in an earlier sandbox; the PC has
  none), so the stream is checked against the oracle from T2c on (the draws before step 1) and
  whole at increment 6 (`log=`). ADR-0029 point 3 says so.
- **Docs:** `docs/03-domain.md`'s row *Map generation*; the catalog context `RMG` (`rmg`, xtask's
  `CONTEXTS`, the catalog README, `04-rules-and-sources.md` §3); ADR-0029 (the draft moved, its
  status, the Consequences' edges, a revision history) with its index row, ADR-0003's revision
  history and AGENTS.md's rmg reading list; `02-architecture.md` §3, the README's crate table, the
  work process's rmg lane, the global HANDOFF's row. The adventure lane's inbox: `TileRand` and
  `OriginalDraws` may wrap `CrtRand`.
- **The coverage gate:** a scope whose logic crates are skeletons (no function: `h3-rmg`) has no
  coverage data, and `llvm-cov report` failed the scoped verify; the gate and `runs_tests` skip
  such a scope now (the test step then runs its tests; `docs/05-testing.md`), with a unit test.
- **Checked:** the scoped verify (`--scope rmg`) passes but `check-lane` (a shared branch, by
  design; `check-lane --cross-lane` passes); `cargo check --workspace --all-targets`, the kernel's
  and xtask's tests and clippy, `scripts/exe.py check`.

## T2b — the three tables, the rmg part, 8XM8's flag (2026-11-25, increment 2)

- **No cut** (the user, asked with the three parts T2b1 the part, T2b2 the harvest, T2b3 the flag:
  "No cut, use a subagent for a part"): one branch `shared/e14-t2b`, `cross-lane` (`h3-content`'s
  registry is shared; `h3-harvest`, `packs/README.md`, `packs/base/core.toml` and the fixtures are
  the battle lane's), the golden replays' re-hash given to one worker. Staged from the PC (before
  its link closed): `H3bitmap.lod`, `H3ab_bmp.lod`, `Heroes3.exe`, `extracted.tar` and the six
  tables.
- **The rmg part** (`crates/h3-rmg/src/part.rs`; agent decisions, change freely): the records
  `Template`, `Zone`, `Connection`, `ObjectTemplate`, `Obstacle` hold the values as read (`i32` as
  `c_atoi` gives them, the records' widths in `structs.toml`; marks as bit sets, bit k the k-th
  column or character); `ZoneKind` and `Strength` with the original's numbers (0-3; 0, 2, 3, 4). It
  refers to no core content (the tables index town types, terrains and classes by position, as
  the code does). `RmgRuleset::compile(core, packs)` in the core's load order: a pack's
  `objects.txt` lines first (inserted `before` a line of the earlier packs' table, appended, or
  replacing a `line`; the indices are of the table the earlier packs left, so they do not shift
  each other), then its templates appended, then its connection overrides (by template index in
  the table as it then stands, checked in every profile, applied when `h3_content::override_applies`
  says so), then its obstacles (a whole table, replacing). Every index must be of the pack's
  closure. The hash (FNV over the tables, the core's hash apart, as the town part's) with
  `RMG_RULESET_HASH_VERSION` 1. `RMG_GENERATOR_VERSION` comes with T2c's `generate`.
- **The format** (`packs/README.md`, "The map generator's tables", as ADR-0029 point 7 says):
  top-level `object_templates` and `object_template_overrides` (one inline table per line),
  `[[template]]` with `zones` and `connections` arrays (one inline table per row), `[[connection_override]]`,
  `[[obstacle]]`. `packs/synthetic/rmg.toml` (invented: two templates, three lines, two obstacles)
  exercises every field. `h3-pack-rmg`: `parse_rmg`, `compile`, `ORIGINAL_PACKS` (`base` has no
  file), strict like the other adapters (unknown fields, mark strings' lengths and characters).
- **The harvest** (`crates/h3-harvest/src/rmg.rs`): renders TOML from the cells (no edge to
  `h3-pack-rmg`: `arch.toml` unchanged). AB's pack from `H3ab_bmp.lod`'s tables; SoD's holds what
  `H3bitmap.lod`'s add: the templates after AB's (AB's must be a prefix, block for block), the
  `objects.txt` edits (a SoD line equal to the next AB line keeps it, one with its picture, class
  and subclass replaces it, any other is inserted before it; every AB line kept or replaced), the
  obstacles only if they differ (they do not). Result: AB 50 templates, 1 221 lines, 109
  obstacles; SoD 3 templates, 105 lines added (12 before AB's line 124, 5 each before 1 141, 1 144 and
  1 147, 7 before 1 187, 71 before 1 188, lines from 0 without the count line), 2 changed (1 186, 1 187: AVXcrsd0 and AVXplns0's editor terrains; the count line is derived). Survey §3.1's
  sanity counts hold (zones 616 and 646 by kind, connections 833 and 869, never kept 6 and 7:
  `the_original_tables_are_the_surveys`). `packs/ab/rmg.toml` is 854 kB (1 221 lines with two
  48-character masks, 616 zones of 19 fields, a 109 × 109 score table), `packs/sod/rmg.toml` 49 kB.
- **8XM8** (RMG-QUIRK-22): the gap-fill's `data_quirks.rmg_zone_ids` (template, the fixed first
  zones 6, 6, 7, 7, 8, 8); the harvester finds the template's connection rows whose first zone is
  a decimal (template 43, connections 10 to 15) and writes six `[[connection_override]]` with
  `fixes = "quirk.content.rmg_8xm8_zone_ids"` into `packs/ab/rmg.toml` (zones [6, 37], [6, 38],
  [7, 41], [7, 42], [8, 47], [8, 48]). The flag at registry position 51 (`RMG_8XM8_ZONE_IDS`), in
  the harvested profiles (`faithful` on, `strict-fixed` off): the original packs' core hash moved,
  so the golden replays and the tests pinning it were re-hashed by the worker (only hash lines):
  `base:faithful`'s core hash 0x0ee6066a4c802530 → 0x52a9516f14766fb8 (`strict-fixed`'s, the flag
  off, unchanged; the adventure and town parts' unchanged): the 115 `base:faithful` adventure
  replays' `ruleset_hash` headers (their checkpoints and final hashes reproduce), `h3-server`'s
  `record.rs` expectation, the two bot reports' `engine` line (regenerated in release by the
  runner, the same bytes). `python/records/ceiling/e25-structure-1-side-4.json` keeps the old
  hash (an E25 run's record; no test reads it).
- **The original-data check** (`crates/h3-pack-rmg/examples/original_data.rs`, run by
  `--with-original-data`): its own reader of the six text tables (survey §3.1's rules) against the
  compiled part, AB's (`base`, `ab`) and SoD's: equal in `faithful`; `strict-fixed` differs by
  8XM8's six ids only.
- **The registry's capacity:** 52 of `MAX_QUIRKS` 64 positions are taken; ADR-0029's remaining 17
  flags need 69 (the epic's risks).

## T2c — the settings, the template, the positions, `generate`'s frame (2026-11-25, increment 2)

- **Whole** (decision 9; asked with the cut T2c1 the lane's own paths, T2c2 the hash-moving part:
  "one session, a sub-agent if needed"): one branch `shared/e14-t2c`, `cross-lane` (`h3-content`'s
  registry and hash, `packs/base/core.toml`, `h3-harvest`, `h3-formats`, `h3-pack-battle`'s profile
  test, the replays' headers). Staged from the PC at the start: the client lane's set (the four
  LODs, `shell-videos.vid`, the executable and five DLLs, both `.snd`, `H3ab_ahd.vid`,
  `battle-videos.vid`, both tars, `games/`, SoD's three tables, Complete's two generated maps, the
  40 MP3s), SHA-256 checked against the PC's. The VM restarted once (the setup's baseline lost).
- **Read** (`scripts/exe.py`; survey §1.1 to §1.5, §2.1 confirmed and sharpened): the dialog's
  draws and clamps (0x586103 to 0x58622C: teams 0 or above the players the players, computers
  clamped to 8 less the players, computer teams 0 the computers, none drawn without computers or
  with 8 players); the run (0x54BF60: under two players in all one and one); the constructor
  (0x537B10: water's draw, the templates, the hero bans from the table 0x679DD0's bytes +0x38 to
  +0x3A, the limits from the last pair to the first); the templates (0x537FF0, 0x538480: the
  ranges inclusive, a connection's zones the first kept zone of each id, the kept count of human
  starts then of all starts, RoE's eight town types and its monster towns' +0x9D, Fortress); the
  draw and the positions (0x549930 to 0x549ACD). **New, RMG-QUIRK-24** (the user: fix-optional,
  flag): a start zone without Ownership (position -1; AB's 2SM2i(2), 2SM0k) writes `start[-1]`,
  the human starts' eighth byte, a phantom human start 7; starts running out write past the arrays
  (a ninth player clears the drawn template's index), added to RMG-QUIRK-4 (typed error). A fresh
  agent re-read 0x549930 to 0x549AD0 and 0x5386C4 to 0x538895 without the conclusions and found
  the same (RMG-QUIRK-11's and -24's second look).
- **The port** (`crates/h3-rmg`; agent decisions, change freely): `settings.rs` (`Settings` with
  every choice resolved but water's Random, `Settings::check`'s typed `SettingsError`s, `Choices`
  and `resolve_random` over a caller's `CrtRand`, re-exported by `h3-rmg`); `template.rs` (`Fit`,
  `kept_templates`, `KeptTemplate` with its zones renumbered, its connections as zone indices and
  rows, RoE's masks; `positions` with `PositionError`); `generate.rs` (`generate(core, part,
  settings, seed, trace)`: the part's core hash checked, `NoGenerator` without the generator's
  tables, the run's one and one, `srand`, water, the templates, the bans, the draw, the
  positions, `step_end(0)`; `GeneratedMap` so far the settings as run, the seed, the kept
  template and its name, the positions and the bans; RMG-STUB-1 the steps and the writer's
  content). `Trace` takes each draw's call site and value (the phase log's line) and each step's
  end (0 before step 1, the oracle's dump 0).
- **The executable's constants** (ADR-0029 point 5): `h3-formats`' `RmgTables` and `rmg_data`
  (the limit pairs, types checked below 232) and `hero_data`'s three bytes, harvested into
  `packs/sod/rmg.toml`'s `[generator]` (marks by hero id, pairs in order; `packs/README.md`);
  `GeneratorTables` (`banned_heroes`, `type_limits`) in the part, its hash with them
  (`RMG_RULESET_HASH_VERSION` 2); the synthetic pack's `[generator]` invented.
- **The registry** (`h3_content`): `MAX_QUIRKS` 128, `Quirks` a `u128`, the ruleset's hash
  writing its low and high 64 bits (`RULESET_HASH_VERSION` 54; golden `0xa4a7d9691b62641f`);
  `quirk.rmg.roe_monster_town_byte` (52, RMG-QUIRK-11) and `quirk.rmg.unowned_start_seventh` (53,
  RMG-QUIRK-24) on in `faithful`, off in `strict-fixed` (`h3-harvest`'s profile text and
  `packs/base/core.toml`). Every profile's core hash moved: `base:faithful` 0x52a9516f14766fb8 →
  0x9078e1983efcc013, `base:strict-fixed` 0xb0c378a75ff70a5b → 0xc67d550d834033d8,
  `synthetic:faithful` 0x876dc0d92f71fae2 → 0xa57f7ac71303fa43: the 128 replays' `ruleset_hash`
  headers, `h3-server`'s `record.rs`, the two bot reports' `core` (only those lines; the part
  hashes, checkpoints and final hashes unchanged: `replay_fixtures`, `adventure_replays` pass).
- **Rules** (`docs/rules/catalog/rmg.toml`, the catalog's first): RMG-SET-001 to -009, RMG-TPL-001
  to -008, all implemented; RMG-TPL-001, -002 and -006 marked in the harvest and its tests.
- **Tests:** `h3-rmg`'s unit tests (the settings, the draws from `srand(1)`, the size index, the
  kept templates, RoE's masks, the positions with and without the flag, the bans, the limits);
  `h3-pack-rmg`'s `generate` tests: the 21 corpus cases draw `kept.txt`'s templates with the one
  draw `(0x549975, the first rand())` and `step_end(0)`; water's Random first; one and one; the
  original bans (SoD and AB: 4 and 145 to 155; RoE: 128 to 155); RMG-QUIRK-11 and -24 in both
  profiles (2SM2i(2) with three players: position 7 the third colour; strict-fixed and four players
  run out); every error; a property test over arbitrary choices, seeds and both profiles.
- **The oracle** (`h3rmg.py corpus --phases`, a ballast for the game's disk check): the 21 first
  maps' draws before step 1 are the port's (21 of 21: `549975` and the first `rand()` after
  `srand(seed)`); 20 maps as kept; `m2-4p` the other map T1c2 once saw (113 282 bytes) in the corpus
  and alone, its dumps 0 and 1 as kept and step 2's not (RMG-QUIRK-1's draw 0x532A5B; increment 3).

## Increment 2's checkpoint — the exit, increment 3's cut (2026-11-25, increment 2)

- **Sandbox:** a new chat; staged from the PC the client lane's set (the four LODs,
  `shell-videos.vid`, the executable and five DLLs, both `.snd`, `H3ab_ahd.vid`,
  `battle-videos.vid`, both tars, `games/`, the 40 MP3s: 58 files, SHA-256 equal to the PC's
  `sync/staged-sums-2026-11-25.txt`), SoD's three tables and Complete's two generated maps; the
  clone GitHub's (the PC's repo folder one commit behind). The setup's baseline lost to a VM
  restart in its coverage gate, its test gate passed; CI green on `master` after #1174
  (`529bb617b`) and after #1175 (`8fa27115f`) stood in.
- **The exit, as checked:** (a) the catalog's 17 rules (RMG-SET-001 to -009, RMG-TPL-001 to -008)
  `implemented`, each named by a test (`h3-rmg`'s unit tests, `h3-pack-rmg`'s `generate` tests,
  `h3-harvest`'s `rmg` tests); (b) the 21 cases draw `kept.txt`'s templates with the original's one
  draw before step 1 (`the_corpus_draws_the_originals_templates`, in CI; the phase oracle 21 of 21
  in T2c); (c) RMG-QUIRK-6 a typed error in every profile, -11's and -24's flags tested in both
  profiles and each re-read by a fresh agent, -22 the `original_data` check (run again: AB's and
  SoD's tables equal their text tables, 1262 zones in both parts, `strict-fixed` only 8XM8's ids);
  (d) `arbitrary_settings_never_panic`; (e) none. **Not seen by the oracle:** -11 (Complete's
  generator writes SoD maps only: no RoE case) and -24 (no corpus case draws 2SM2i(2) or 2SM0k).
- **The user** (decision 10): increment 2 closed; increment 3 cut into T3a (the phase oracle's
  per-step record, RMG-QUIRK-1's bytes, the control word, the cases for -24), T3b (the items not
  read), T3c (step 1, the cell record and the comparison seam), T3d (step 2), T3e (step 3 and the
  terrain painter), as proposed; the increment's four flags registered at T3c in one re-hash (the
  other choice, each with its task, three re-hashes); the session docs only.

## T3a — the phase oracle's memory record, the fixed environment, the corpus at 24 (2026-11-25, increment 3)

- **Whole** (asked with a cut, the record and hooks then the measurement; the user: "no cut, use
  subagents efficiently"). Staged from the PC at the start: the client lane's set (58 files,
  SHA-256 checked), `games/`, SoD's three tables, Complete's two generated maps. Baseline green
  (`verify-once: verified (sandbox) code f4dc00b2… on ad1c91a7…`), CI on `master` green.
- **Read** (`scripts/exe.py`): `rmg_zone_generator_init` 0x5329E0 has three callers, each with the
  zone generator as `this` and the zone as its one argument: 0x53BDA1 (step 1, the template
  zones), 0x53E149 (step 2, the temporary zone at the caller's ebp-0x164, whose +0, +4 and +8 alone
  are written, its generator at ebp-0x578) and 0x53E45C (step 2, a water zone from operator new).
  The constructor writes the generator's +4 (the drawn town type) and never its +8.
- **Built** (agent decisions, change freely): `xdd.c` 10. the memory record `rmg<g>.memory` (the
  control word by `fnstcw` at `srand` and each step's first call; per constructor call the zone's
  0xD4 bytes and the generator's first 0x44, read only; 22 hooks); `h3rmg.py`: `phase_digests`
  (per phase its draws and the digest of its lines as logged), `read_memory`, `zone_fields` (the
  form two fresh starts agree on: the set mask of +0x41..+0x49, +0x40, +0x84, +0x94 as 0 or 1; the
  values, the limits and the generator's +8 left in the record after the measurement showed them
  vary), `zones.txt`, `phases.txt`'s `cw=`, `zones=`, `steps=`, the record checked against the
  log's 0x532A5B draws per step, `memory [--against]`, zone differences failing only a fresh start;
  `h3drive.py`'s `H3_FIXED_ENV` (an allowlist of six variables and `H3*`, `PATH` fixed, sorted, the
  clock folder through `original-data/wine/clock`), set by `h3rmg.py` (`--sandbox-env` the old
  way; `FIXED_PATHS` refuses another clone folder, home or prefix); `test_h3rmg.py` +3 tests;
  `corpus.toml` +3 cases (seeds by `drawn_template`: 1 350 000 003 draws 2SM2i(2) of 12 with three
  players, 1 350 000 000 and 1 450 000 002 2SM0k of 21); `h3-pack-rmg`'s corpus at 24 and
  `the_corpus_starts_without_ownership_place_the_colours_as_the_original`.
- **Found and changed course:** the first hooked run gave `m2-4p` the variant map (`45ae078f…`):
  the game got the sandbox's whole environment (17 KB, a session folder in `PATH`) and a clock path
  whose length differs between `one` and `corpus`. With the fixed environment it gave `kept.txt`'s
  map and T1b2's log in both; so the environment was fixed before the record, and the measurement
  ran on it.
- **The record** (`corpus --phases --twice --again --record`, about 32 minutes): the 24 first maps
  reproducible; 19 of the old 21 as kept with T1b2's logs and dumps; `l1-4p-b`'s first and second
  maps swapped against T1b's, `xl2-4p-seed0` another map (both re-kept); the three RMG-QUIRK-24
  cases generate; the control word 0x027F in all 72 records; 73 step 2 zones (ORACLE.md).
  `phases.txt` and `zones.txt` were rewritten twice from the record's first maps with `h3rmg`'s own
  functions after the zone form changed (each line's draws, log, steps and cells asserted equal).
- **The measurement** (one agent, about 50 minutes, 31 starts and about 60 environment sizes; its
  table in ORACLE.md): independent of variables outside `H3*`; the water zones' bytes flip at a few
  block sizes, each at N ≡ 2 (mod 8) characters; the temporary zone never changed; a map moved only
  where a water zone's set count crossed 0; one water zone's +8 (`l1-4p-b`) differs between fresh
  starts about half the time. Not run: another history before the generation (no documented
  `--before` string; T1b1 had found the bytes unmoved by the Load Game menu).
- **RMG-QUIRK-24 confirmed:** `s1-3p-2sm2i2`'s map has two players in its player block (tan, on the
  port's position 7, left out); the 2SM0k maps give both players their main towns. `h3_map` reads
  24 of 24, 23 import and validate.

## T3b — step 2's items not read, and code 1 (2026-11-26, increment 3)

- **Code reading only** (the user: "it is code reading only"): `Heroes3.exe` staged alone from the
  PC (SHA-256 `aac39e8c…` checked by the tool); no oracle run, no build of the lane's crates.
  Started on `master` after #1185 (T3a) merged; CI on `master` running then.
- **Read** (`scripts/exe.py`, about 30 functions and sites): the triangulation 0x5FCEF0 to 0x5FDB40
  (init, find, insert, finish and their helpers), `rmg_level_zones_partition`'s calls (the data
  passed: the zone, the water zone, none on level 1), `rmg_zone_borders_draw` 0x53C390 whole,
  `rmg_segment_clip` 0x53CAC0 whole, `rmg_zone_flood_fill` and its tail 0x53D640,
  `rmg_water_zones_connect` 0x53DAD0 and `rmg_zone_graph_distances` 0x53D8E0 whole, `u16_vector_resize`
  (was `_insert`), `rmg_run`'s, `random_map_generate`'s and its two callers' exception data.
- **Found:** the triangulation is D. Lischinski's incremental Delaunay triangulation step for step
  (Locate, InsertSite with the on-edge case, Lawson flips), in integers (32-bit TriArea, 64-bit
  InCircle, both strict) on half-edges that store their origin; a square frame (-200..400) split by
  a diagonal; a point at an existing vertex is dropped. `triangulation_finish` writes each zone
  triangle's circumcentre (two truncating divisions, from the first record with data in the
  vector's creation order) into its three records' +0x1C, +0x20. The borders walk the zone's
  Voronoi cell; the clip works on a side's line (both ways from its start, never testing the end);
  the edge walk's shortcut (a shared x or y) also fires across the map, so an axis-aligned single
  side gets a straight chord over its jagged line, and as read no fill reaches the cells between.
  The fill is a 4-connected scanline fill (its result order-free). The water connections: new
  neighbours whose side's clip is on the map (wide, created), then each new zone to its earlier
  neighbours unless Z.dist[c] > W.dist[c] + 1 for another zone c (wide, not created). **Code 1 is
  only `TGzFile::TOpenFailure`:** `rmg_run`'s one handler catches that class; no frame up to the
  scenario dialog catches anything else, so the divisions by zero (RMG-QUIRK-3) end the game.
- **Docs:** survey §1.2, §2.2 (four paragraphs, the table, the failures, the not-read list),
  §3.2.4 (ZON-009, -012 to -014), §3.2.15; `map.toml` 12 entries re-read, 9 new; `structs.toml`
  `triangulation`, `triangulation_edge`, the zone generator's +0x3E4 and +0x3F4; the ledger's
  RMG-QUIRK-3 corrected and five open candidates (classified when T3d's cells compare); T3d's scope
  names them.

## T3c — step 1, the generator's map, the comparison seam (2026-11-26, increment 3)

- **Whole** (the user, asked with the cut T3c1 flags and seam, T3c2 step 1: "T3c whole", and "yes"
  to an oracle hook recording the zone generators after step 1, since dumps 0 and 1 are only the
  reset cells). One branch `shared/e14-t3c`, `cross-lane` (`h3-content`'s registry, `core.toml`,
  `h3-harvest`, `h3-formats`, the replays' and bot reports' core hashes). Staged from the PC at the
  start: the oracle's set (58 files, SHA-256 checked), SoD's three tables, Complete's two maps; #1187
  had merged.
- **The flags** (one agent, its own worktree): `quirk.rmg.zone_memory_model` 56,
  `terrain_towns_count` 57, `partition_units` 58, `cosmetic` 59, on in `faithful`, off in
  `strict-fixed`; only `faithful`'s core hash moved (`0x3f1f68842dab4e0c` → `0x7d6081fbb0552a2f`;
  a flag off leaves the bits as they were): 115 adventure replays' `ruleset_hash`, `h3-server`'s
  `record.rs`, the two bot reports' `core`; checkpoints and final hashes unchanged.
- **Read** (`scripts/exe.py`): `rmg_zones_lay_out` 0x53BCB0, `rmg_zone_generator_init`,
  `rmg_zone_place`, `rmg_zone_position_fits`, `rmg_zone_candidates_around`,
  `rmg_zone_candidates_filter`, `rmg_zones_touch`, `rmg_zones_bounds`, `rmg_zone_terrain_draw`,
  `rmg_map_cell_reset`, the connection lists in `rmg_templates_read`; survey §2.2's step 1
  confirmed and sharpened (the tested points written to the zone, the box filter from 32000, the
  32-bit squares).
- **The port** (agent decisions, change freely): `cells.rs` (`Cell` the record's words +0x10 to
  +0x2C, `Cell::RESET`, `Cells::dump` the oracle's format 1); `layout.rs` (`ZoneGenerator`, the
  three passes, `trunc_sqrt` the integer root binary64's gives, the points through `Real` with the
  tables' ten-thousandths as the nearest doubles); `Trace::step_end` takes a `StepState` (the
  cells, the zone generators); `GeneratedMap` holds both; `GenerateError::NoPosition` and
  `LayoutSpan` (RMG-QUIRK-3). The tables (`layout_cos`, `layout_sin`, `terrain_towns`,
  `native_terrains`) read by `h3-formats`' `rmg_data` (a double kept only when its shortest
  decimal has at most four places) into SoD's `[generator]`; the synthetic pack's invented rows.
- **The oracle** (`xdd.c` section 10: the `generators` lines; `h3rmg.py`: `read_generators`,
  `layout.txt`): `corpus --phases --record`, 24 of 24 maps, logs and dumps as kept, `layout.txt`
  new (388 zone generators). The disk ballast set to 1.5 GiB modulo 4 GiB for the game's check.
- **Tests:** the corpus's draws (setup, template, step 1) and dumps 0 and 1 against `phases.txt`
  (a test SHA-256), its layout against `layout.txt`, both 24 of 24 at the first run; RMG-QUIRK-12 in
  both profiles and RoE; the colour's town choice; RMG-QUIRK-3's two cases; the property test's
  layout invariants; the root, the dump's bytes. `RMG_GENERATOR_VERSION` stays 1: no map is
  written before increment 7.

## T3d — step 2, the triangulation, the borders, the fills, the water zones (2026-11-26, increment 3)

- **Whole** (the user, asked with the cut split on water / split on code / whole: "T3d whole").
  Branch `rmg/e14-t3d`, the lane's paths and the capture harness (`h3rmg.py`, `water.txt`).
  Staged from the PC at the start: the oracle's set (58 files, SHA-256 checked), SoD's three
  tables, Complete's two maps; #1196 had merged. The setup's baseline stopped early (no build ran
  during the oracle's corpus); CI on `master` stood in.
- **The oracle:** `corpus --phases --record` once, `water.txt` added (the memory record's zone
  generators after step 2, `town2` left out: a water zone's +0x8 is the heap's); `kept.txt`,
  `phases.txt`, `zones.txt`, `layout.txt` unchanged on 24 of 24.
- **Read** (`scripts/exe.py`): `rmg_level_zones_partition` 0x53E050 whole (the points' f64
  compares, the water zone's written fields, the borders' and fills' loops); `rmg_jagged_line_draw`
  and `rmg_line_draw`; `triangulation_finish` and its helpers 0x5FDCB0 to 0x5FDD40 (32-bit, two
  `idiv`). One agent read `rmg_zone_borders_draw`, `rmg_segment_clip`, `rmg_zone_flood_fill` with
  its tail and `rmg_water_zones_connect` exactly (pseudo-code, survey §2.2 "Step 2 as ported"):
  the fill's start scores the ring's destination vertices and clips once (T3b had each clipped),
  the borders' b and later a are never tested, the corner rule's exclusions.
- **Ported:** `triangulation.rs` (Lischinski's on half-edges, the original's vector order, total:
  a point outside the frame not inserted, a walk capped, a flat triangle -1, none reachable);
  `partition.rs` (the points, the water zones, the clip, both line drawers, the borders, the fill,
  the connections and hop counts); `Cells::get`, the zone byte and land bit; `KeptConnection`'s
  `created`; `GeneratedMap.partition` (polygons, distances); `Trace::zone_memory` (RMG-QUIRK-1's
  measured bytes in place of the model's); `GenerateError::Partition` (RMG-QUIRK-3's jagged limit
  0, -4's straight line off the map and fill start off the map, the level's first fault).
- **Against the oracle:** with `zones.txt`'s bytes the step 2 draws (per phase) and dump 2 equal on
  24 of 24 once the fill's start was corrected (`l1-4p-b`'s zone 14 at (119, 30)); the water zones
  44 of 44; no straight line left the map. The model's own bytes: 22 of 24 (`l1-4p-b`,
  `xl2-4p-seed0`). RMG-QUIRK-14 confirmed: with scaled sizes `s1-2p` would get 12 water zones.
- **T3b's corner cases** (counted in the port, equal to the oracle): the clip of a side wholly off
  the map on the map 104 times in 19 cases; the chord, the whole frame, a dropped point never; 20
  zones (13 water, 7 land) in 6 cases fill nothing. Into the ledger's open table for the user.
- **Tests:** the corpus to step 2, the water zones, the model's 22, `strict-fixed`'s zeroed bytes,
  step 2's jagged limit 0; `h3-rmg`'s triangulation (Delaunay over seeded points, Euler, the edge
  and vertex cases, the circumcentres, the fallbacks) and partition units (the clip, the lines,
  the units flag, the memory choice, the whole frame, the faults, an off-map water side); the
  property test's step 2 invariants. Coverage 100 % after the faults became a per-level record
  (the unreachable `?` paths gone). `RMG_GENERATOR_VERSION` stays 1.

## T3e — step 3, the terrain painter, the picture sets (2026-11-26, increment 3)

- **Whole** (the user, asked with a cut at the islands or at the painter: "Whole"; the harvest
  `cross-lane`: yes). Branch `shared/e14-t3e`, `cross-lane` (`h3-formats`, `h3-harvest`,
  `packs/README.md` beside the lane's paths). Staged from the PC at the start: the oracle's set
  (58 files, SHA-256 checked), SoD's three tables, Complete's two maps; #1199 had merged.
- **The oracle:** `corpus --phases` once (no `--record`: `phases.txt` already held step 3's draws
  and dump digests since T3a); 24 of 24 reproduced; the step 3 dumps kept for a cell diff, never
  needed.
- **Read** (`scripts/exe.py`): `rmg_terrain_paint` 0x53E6A0 whole (rock only with two levels,
  the zones' cells by their signed byte against the index, a painter finished even without
  cells), `rmg_zone_bounds_compute`, `rmg_zone_centre_to_centroid` (the byte against the zone's
  id), `rmg_island_shape` (the corners from the last down, each coast to the one after it),
  `rmg_coast_line_draw` (k = min(limit, L / 2)), **`rmg_island_fill` (inferred until now): a
  4-connected stack flood through E, S, W, N**; the map's slots 1 and 4 (`rmg_map_set_tile`,
  `rmg_map_get_tile`: bits 0-5, 6-13, +0x28 bits 15-16); the painter's `finish` loop, `set_cell`,
  `queue_neighbours` (the diagonals by their own set's byte +5), `ring_split`, `ring_same`'s
  per-axis clamp, the weight, the slots 4 and 5 of both kinds, `border_class`,
  `corner_far_differs`; the sets' static initializers (the arguments are immediates).
- **Harvested:** the ten picture sets (`h3_formats::exe::rmg_data`: each initializer's pushes,
  its set checked against `terrain_picture_sets`, the tables' shapes and flags; rock's 48 by
  variants) into SoD's `[generator]` as runs (`picture_sets`, `rock_pictures`); the synthetic
  pack's invented; `RMG_RULESET_HASH_VERSION` 4; `GeneratorTables` checks nine sets, shares up to
  100, at most 127 pictures of shapes up to 28.
- **Ported:** `painter.rs` (RMG-TER-005 to -018: the sets' ranges as the initializer builds them,
  slots 4 and 5, the weight, the pinch and ring tests, `set_cell`, the queue, the resolution and
  the ring's gaps, the finish loop, the last pass's classes, ten shape groups under four
  mirrorings and the refinements); `terrain.rs` (RMG-TER-001 to -003: boxes, centroids, islands,
  rock, water, each zone); `ZoneGenerator.bounds`; `Cells::clamped`; `GenerateError::Terrain`
  (`TerrainFault::CoastLimit`, `NoPolygon`, `Paint(NoPicture | Stuck)`). The painter reads the map
  itself, not a copy (every write goes to both; nothing else writes while it works). RMG-QUIRK-21's
  painter's half on `quirk.rmg.cosmetic`: off, both neighbours of an axis queued and group 3's 8s
  mirrored as matched.
- **Against the oracle:** the step 3 draws and dump 3 equal `phases.txt`'s on **24 of 24 at the
  first run**, the four islands cases included. The corpus never resolves a pinched water or rock
  cell (water is painted over water, rock in one rectangle): the ring's gaps and the water branch
  of `set_cell` are checked by unit tests until steps 6 and 11 reach them.
- **Tests:** the corpus to step 3; step 3's coast limit 0 (a zone of base 0 with islands); the
  property test's step 3 invariants (a terrain and one of its set's pictures per cell, rock
  unmirrored, a zone centred in its box); `h3-rmg`'s painter units (the ranges, the slots' draws,
  a split water ring mended, water neighbours entering and leaving set 1, the cosmetic flag's two
  halves, the classes and refinements, a missing shape, a cell that cannot be mended) and step 3
  units (boxes, centroids and levels, an island, the faults). `RMG_GENERATOR_VERSION` stays 1.

## Increment 3's checkpoint — the exit, increment 4's cut (2026-11-27, increment 3)

- **The exit, as checked:** (a) the catalog's 31 rules (RMG-ZON-001 to -014, RMG-TER-001 to -003,
  -005 to -018) `implemented`, each named by a test; (b) on the 24 corpus cases in `faithful` the
  draws and dumps after steps 1, 2 and 3 equal `phases.txt`'s (with `zones.txt`'s measured bytes;
  the model's own give step 2 on 22), in CI; (c) RMG-QUIRK-12, -14 confirmed by (b), -21's
  painter's half and -1's flag tested in both profiles, -3's and -4's cases of steps 1 to 3 typed
  errors; (d) `arbitrary_settings_never_panic` over steps 1 to 3; (e) T3b's items and
  `rmg_island_fill` read. #1204 (T3e) open, CI running.
- **The user** (decision 11): increment 3 closed; RMG-QUIRK-1's form left open until increment 4
  (the constants stay provisional); T3b's clip of an off-map side harmless and accepted (the
  ledger's "Considered, not quirks"); a zone that fills nothing fix-optional as RMG-QUIRK-25 (its
  fix and flag in T4b); the three corner cases never seen wait for increment 7's census;
  increment 4 cut into T4a (the items not read), T4b (steps 4 and 5, object placing), T4c (step
  6's obstacle plan), T4d (step 6's connections), T4e (step 7), as proposed. Docs only.

## T4a — the items not read: the masks, the placing tests, the islets, the monoliths (2026-11-27, increment 4)

- **Read** (code only, `Heroes3.exe` staged; survey §2.1 "The template's masks as read", §2.2
  "The placing tests as read", §2.3 "The height map as read" and the monoliths' pairing; §3.2's
  rules rewritten; §3.2.15's list holds no item of increment 4's now):
  - RMG-OBJ-009: `objects.txt`'s masks are `std::bitset<48>`s (character 8r + c is the cell c
    left of, r above the anchor, VCMI alike); the footprint's size and colour mask come from the
    picture's `.msk`; +0x4 = passability OR NOT colour; +0xC = actions AND NOT +0x4 (the
    entrances); placing marks an entrance bit 22 (bit 25 kept), else a blocked cell bit 25 clear.
  - RMG-OBJ-011: the footprint (with its water branch: a blocked cell on water only for a group-0
    template with water in the fourth column), the outline (a right-hand walk from below the
    anchor row's first blocked cell), the ring (its open cells one unbroken arc, or all), the cell
    below the entrance. Tents on water: refused; a water zone's tent fits only on its land.
  - RMG-OBJ-008: the six callers of `rmg_prototype_draw`, two testing a null.
  - RMG-CON-008: the height map's records (rows, columns, corners top-left to bottom-right, the
    amplitude), the first split without a draw (centre a / 2), the quadrants' order.
  - RMG-CON-017: no pairing at write; entrance and exit prototypes share the index, one channel
    per one-way connection with an exit in each zone; the index wraps after n45 + n43.
- **Found:** an empty outline divides by 0 (RMG-QUIRK-3's class, not reached by the shipped
  lines); the seer huts' creates use a null unchecked (RMG-QUIRK-4's class); the one-way
  monoliths' shared channel, a new open candidate for the user; **for T4b** the rmg packs lack
  each picture's `.msk` (width, height, colour mask), which the footprint and +0x4 need, and
  `h3-rmg`'s `ObjectTemplate` names the two terrain columns the other way round from the
  generator's use.
- **Docs:** the survey, `map.toml` (ten entries read again or corrected, seven added),
  `structs.toml` (`rmg_object_template`'s fields, `rmg_prototype`'s outline, `rmg_heightmap_rect`),
  the ledger. No code. The gates: `exe.py check`, `cited`, `check-exe-map`, the light verify.
