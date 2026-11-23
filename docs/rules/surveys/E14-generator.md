# E14 generator survey: the original's random map generator

Date: 2026-11-23. Scope: E14 increment 1 (`docs/epics/E14-original-map-generator.md`). §1 the
frame (T1a1); §2 every function placed, the draws and floats per phase (T1a2); §3 the template
format, each phase's rules and the quirk candidates (T1c). Input, not rules (`README.md`): a fact
becomes binding when a task writes it into the catalog with its sources and tests.

**Sources:** the code reading of GOG Complete's `Heroes3.exe` (SHA-256 `aac39e8c…`,
`docs/exe/README.md`; the names are in `docs/exe/map.toml`, the records in
`docs/exe/structs.toml`); H3API's `H3RMG` headers (commit `92255ab`, 2024-11-11; names and
layouts, a pointer only); two maps the original generated on the user's machine in May 2026
(Complete's `random_maps`, observation, never committed: `Untitled.h3m`, `dbg.h3m`).

**Terms.** *Players* are the dialog's human-or-computer players (H3API's `humanCount`),
*computers* its computer-only players. A *position* is a template's start zone number (zone
+0x1C, 0-based). A *draw* is a call of the C runtime's `rand` (`crt_rand` 0x617E5C, MSVC's
linear congruential generator; its state is per thread).

## 1 The frame (T1a1)

Path read: Begin (`scenario_begin` 0x58BCE0) → `random_map_generate` 0x5860E0 → `rmg_run`
0x54C090 → `rmg_generate_and_write` 0x54BF60 → `rmg_generator_create` 0x537B10,
`rmg_generate_map` 0x549930, `rmg_write_map` 0x54ABF0. Not followed: the other caller of
`random_map_generate`, `scenario_dialog_click`'s call at 0x586D7B (the dialog's own; T1b sees
which button). Everything runs on Begin's thread; the busy cursor's thread only animates.

### 1.1 The dialog's choices and their Random draws

`random_map_generate` reads the setup's choices and draws each that is Random (-1), in this
order, from the game's own `rand` state (before the generator's seed, so an oracle case with a
Random choice depends on what the game drew before):

| # | Choice | Draw (site) | Afterwards |
|---|---|---|---|
| 1 | players | `h + rand() % (9 - h)`, h the slots marked human (0x586168) | |
| 2 | their teams | `rand() % players + 1` (0x586183) | 0 or above the players: the players |
| 3 | computers | `rand() % (9 - players)` (0x5861B1); none with 8 players | at most `8 - players` |
| 4 | their teams | `rand() % computers + 1` (0x5861E4), only with computers | 0: the computers; above the computers: **the players** (as read; a quirk candidate if the dialog can give it) |
| 5 | monster strength | `rand() % 3 - 1` (0x58621D); the dialog's 0..2 less 1 | -1, 0, 1 |

Water's Random is not drawn here: the value 3 goes on to the constructor (§1.3). The version
is 0 RoE, 1 AB, 2 SoD (the setup's +0x1898: 1 AB, 2 or 3 SoD, else RoE). The parameters
(`rmg_parameters`, 0x50 bytes; `rmg_parameters_init` 0x54BF00: no humans, towns -1, 2 players,
2 teams, no computers, computer teams 8, water 3, monsters 0, version 2) get each colour's human
flag and town choice from the setup's slots. The file is `random_maps\` + the name.

### 1.2 The run and its codes

`rmg_run` opens the file gzip-compressed (mode `wb6`) and calls `rmg_generate_and_write`; a
catch-all handler (its funclet at 0x54C0FE) returns 1 for an exception anywhere in the open,
the generation or the writing. `rmg_generate_and_write`: the generator's monster strength is the
parameters' + 3 clamped to 1..5 (the dialog's weak, normal, strong are 2, 3, 4); fewer than 2
players and computers together become 1 and 1; the generator record lives on its stack; the
human flags and towns are copied in after construction; `rmg_generate_map` false gives 3,
`rmg_write_map` false 2, else 0; `rmg_generator_destroy` 0x537DF0 either way. The codes show
GENRLTXT rows (records from 0 with the header) 750 "Cannot create Random_Map.H3M in data
directory." (1), 751 "Error writing random map to disk." (2), 752 "Could not create a random map
that fits current choices." (3); code 0 returns true.

### 1.3 The generator record and the seed

`rmg_generator_create` (11 arguments: width, height, underground, players, their teams,
computers, their teams, water, monster strength, progress, version; `rmg_generator`, 0x14E0
bytes) calls `rmg_generator_base_init` 0x536070, which builds the map (`rmg_map_init`
0x530FB0: cells of 0x30 bytes), the object vectors and the progress, keeps the version at +8,
then **`time()` (seconds) into +4 and `srand` with it** (0x536148), then reads the objects
(§1.4). Back in the constructor: the choices into the record, then **water Random (3) is drawn
`rand() % 3`** (0x537C89): when water is Random, the generation's first draw. Then the templates
are read (§1.4); with none kept the constructor stops there and `rmg_generate_map` fails (code
3). Else it clears the banned heroes and artifacts and the counts by object type, builds the
object generators, bans the heroes (a hero is banned when its table byte +0x3A is set, or when
its byte for the version is clear: +0x39 for AB and SoD, +0x38 for RoE), sets two tables of 232
limits by object type (`rmg_type_limit_a` 0x69CE4C, `rmg_type_limit_b` 0x69D1F4) to 32000 and
then from the pairs at 0x640718 (30 types: 32, 48, 64 or 200) and 0x640808 (24 types: 1; type
113: 3), and clears the human flags. No other draw happens before `rmg_generate_map`.

### 1.4 The files read

| File | Reader | What is kept |
|---|---|---|
| `objects.txt` | `rmg_objects_read` 0x536200 (`template_file_read` 0x514D80) | the templates at +0x24, 0x4C bytes each (the stride of the division here; `template_file_read`'s entry says 0x44, from its other caller); a 0xE8-byte prototype for each with type 0..231 the version allows (RoE below 165, AB below 222, before SoD types 43 to 45 only subtypes 0 to 2), in the vector of its type's object settings +8; the monsters' vector (54) sorted by subtype with an exchange sort that swaps only when greater |
| `rand_trn.txt` | `rmg_rand_trn_read` 0x536560 | not read (T1a2) |
| `rmg.txt` | `rmg_templates_read` 0x537FF0 | from row 3, a template per row of two or more columns (name, min and max size) up to the next; kept when the range holds the map's size (width × height × levels / 1296; islands: halved, at least 1) and its zones fit the players, computers and version; connections from column 76 (read in part) |
| — | `rmg_object_generators_create` 0x538B10 | the object generators, built in code (many 0x14- to 0x28-byte records into +0x1100), no file; not read (T1a2) |

Which archive's `rmg.txt` the game loads (both `H3bitmap.lod` and `H3ab_bmp.lod` hold one) is
the archives' priority, not read here (T1b observes it, T1c records it).

### 1.5 `rmg_generate_map`'s steps in order

1. **The template:** `rand() % count` of the kept templates (0x549975), its name kept.
2. **The positions:** the colours listed humans first, then the others, each group in colour
   order; the first `players` colours take the positions of the human start zones (type 0) in
   position order, then the computers the remaining start positions (human or computer start)
   in order; position p's colour at +0xEE4 + 4p (-1 none). No draw.
3. Fifteen steps (the names are T1a1's, neutral where the code was not read; T1a2 names them
   from their reading). Closure: the step's functions inside the generator's range not counted
   for an earlier step, its size in thousands of bytes; draws are call sites in that closure, not
   draws per run.

| # | Function | Called | What T1a1 saw | Closure (functions, bytes, draw sites) |
|---|---|---|---|---|
| 1 | `rmg_phase_01_zones` 0x53BCB0 | once (the template) | a zone generator (0x414 bytes) per template zone; a start zone with towns gets its colour's town choice; a layout scaled to the map; each zone's size; a zone without a town type draws one of its terrain's towns from `rmg_terrain_towns` 0x682450 (as read the count is 4 whatever the entries: the test after an entry of -1 goes on counting, so -1 and the padding 0 can be drawn; a quirk candidate to re-read in T1a2 and check in T1b) | 10, 5.5 kB, 4 |
| 2 | `rmg_phase_02_level` 0x53E050 | per level (template, z) | x87 on the zones' positions with two f64 tables; new zone generators | 12, 8.6 kB, 1 |
| 3 | `rmg_phase_03` 0x53E6A0 | once | a temporary map record; game functions outside the range over the cells | 8, 2.8 kB, 1 |
| 4 | `rmg_phase_04_towns` 0x544A50 | per zone | one town of each kind the zone asks for, while each succeeds: castle and town of its colour, neutral castle and town | 7, 2.8 kB, 1 |
| 5 | `rmg_phase_05` 0x544AE0 | per zone | one helper eight times, `sqrt` | 3, 1.9 kB, 3 |
| 6 | `rmg_phase_06` 0x544920 | once | every free land cell, then per zone, then step 9's function and two more | 35, 23.9 kB, 21 |
| 7 | `rmg_phase_07_junctions` 0x5446A0 | per junction zone (type 3) not on water | | 2, 1.4 kB, 1 |
| 8 | `rmg_phase_08_mines` 0x545F60 | once (per zone inside) | each resource's minimum mines (zone +0x4C), the first wood and ore of a start zone with a town flagged | 5, 2.7 kB, 3 |
| — | inline | once | the towns counted, in all and by type | — |
| 9 | `rmg_phase_09` 0x5405D0 | once (also inside step 6) | cells marked impassable (cost 32000) or blocked by land and rock | (in step 6's) |
| 10 | `rmg_phase_10` 0x547360 | per zone | `sqrt`; the progress shared out | 21, 9.9 kB, 5 |
| 11 | `rmg_phase_11_levels` 0x5439E0 | with two levels | game functions outside the range | 1, 0.6 kB, 0 |
| 12 | `rmg_phase_12` 0x531310 | once, on the map | land cells not rock flagged | 1, 0.3 kB, 0 |
| 13 | `rmg_phase_13` 0x537970 | once | cells flagged blocked | 4, 3.7 kB, 1 |
| 14 | `rmg_phase_14_roads` 0x548290 | once | the road targets, costs up to 30000 | 3, 3.2 kB, 1 |
| 15 | `rmg_phase_15_shipyards` 0x549870 | once | cells near objects of types 5, 53, 126, 134 flagged; next to each shipyard (109) two helpers | 6, 5.1 kB, 2 |

It returns true. The only failure it reports is no template (code 3); an exception inside gives
code 1 (§1.2). Whether a step retries, loops to a limit or reads a clock is T1a2's (none of the
frame's functions does).

### 1.6 `rmg_write_map`'s outline

1. `rmg_write_header` 0x549CB0 (read only at its end): the header from the format word on, with
   the description "Map created by the Random Map Generator. Template was …, Random seed was …,
   size …, levels …, humans …, computers …, water …, monsters …, <version> map" (as
   `Untitled.h3m` shows it: the seed and template of every generated map are in its file); the
   teams drawn by `rmg_teams_draw` 0x54AB40 (twice: one draw per playing colour, **at write time,
   after every draw of the generation**: the players spread over the teams as evenly as possible,
   each colour drawing among the teams with room left); for SoD the banned spells (70 bits, 9
   bytes) and skills (28 bits, 4 bytes) and 156 zero bytes. Just before, a 17-byte block of 129 bits
   (the banned artifacts, presumably; the code before it not read); the reader's SoD block is
   18 bytes (`h3m.md`): T1b's map shows which.
2. A u32 0. The reader's order is the rumours' count, then SoD's 156 hero bytes: the generator
   writes the 156 bytes first, the same 160 zero bytes.
3. Every cell (`rmg_write_cell` 0x532890), level by level, row by row.
4. The templates: those prototypes in use numbered from 2, their count + 2, then the first
   prototypes of types 71 (random monster) and 124 (hole) as templates 0 and 1 (as in the
   generated maps: `AVWmrnd0`, `AVLholg0`), then each in use in type order (`rmg_write_template`
   0x54AE30).
5. The objects' count; first the objects whose type's object settings +0xC is set, then the
   others, each through its own vtable with the version.
6. A u32 0 (the timed events' count); true when it took 4 bytes.

### 1.7 The records

`docs/exe/structs.toml` holds the generator's records under our names, each field `read` where
T1a1's code showed it, else H3API's as `guess`: `rmg_parameters` (all read), `rmg_generator`,
`rmg_map`, `rmg_map_cell`, `rmg_template`, `rmg_zone`, `rmg_zone_connection`,
`rmg_zone_generator`, `rmg_object_template`, `rmg_prototype` (H3API has no layout),
`rmg_object`, `rmg_object_generator`. Against H3API: `H3RmgParameters` matches field for field;
the generator's colour per position starts at +0xEE4 (H3API's `playerOwner` at +0xEE0 is cleared
with it as 9 dwords; the code indexes position + 1); the object generators go to +0x1100, not
H3API's `objectGenerators` +0x10F0; the cell's +0x20 is H3API's zone data.

### 1.8 Draws and floats so far; what T1a1 did not read

- **Draws, in stream order:** the dialog's Random choices (§1.1, the game's state); `srand(time)`;
  water's Random; the template; the 15 steps (44 sites in their closures, §1.5); the teams in the
  header. One generator, one thread, one stream.
- **Floats:** x87 in steps 1, 2, 5, 8, 10 (`crt_sqrt`, `crt_ftol`); T1a2 lists each site.
- **Not read:** the 15 steps' bodies (but steps 4, 8 and 15's own code, step 1's in part), `rmg_rand_trn_read`,
  `rmg_object_generators_create`, the zones' reader of `rmg_templates_read`, the header before
  its end, the cell and template writers, the destructor, the per-object writers. The limits by
  type are set but where they are read was not found.
- **For T1b:** a generated map names its template and seed in its description, and the
  original's choices are reproducible only without Random ones (§1.1); the oracle can read the
  seed back from the file to check a fixed clock.
