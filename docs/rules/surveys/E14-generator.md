# E14 generator survey: the original's random map generator

Date: 2026-11-23. Scope: E14 increment 1 (`docs/epics/E14-original-map-generator.md`). §1 the
frame (T1a1); §2 every function placed, the draws and floats per phase (T1a2), §2.7 the painters
outside the range (T1c1); §3 the template format and the two tables, each phase's rules, the
objects against the engine's reader and writer (T1c2); the candidates' classification is the
ledger `docs/rules/quirks-rmg.md` (T1c2). Input, not rules (`README.md`): a fact
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

`rmg_run` opens the file gzip-compressed (mode `wb6`) and calls `rmg_generate_and_write`; its one
try block (over the open and `rmg_generate_and_write`) catches only `TGzFile::TOpenFailure` (its
FuncInfo 0x651F98: one handler, a `const &` of that class; funclet 0x54C0FE) and returns 1. Any
other exception, a C++ throw or a structured one such as an integer division by zero, leaves
`rmg_run` uncaught: `random_map_generate` 0x5860E0 has no try block, nor have its two callers
(E14 T3b). So code 1 means only that the file could not be opened. `rmg_generate_and_write`: the generator's monster strength is the
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
limits by object type (`rmg_type_limit_map` 0x69CE4C, `rmg_type_limit_zone` 0x69D1F4) to 32000 and
then from the pairs at 0x640718 (30 types: 32, 48, 64 or 200) and 0x640808 (24 types: 1; type
113: 3), and clears the human flags. No other draw happens before `rmg_generate_map`.

### 1.4 The files read

| File | Reader | What is kept |
|---|---|---|
| `objects.txt` | `rmg_objects_read` 0x536200 (`template_file_read` 0x514D80) | the templates at +0x24, 0x4C bytes each (the stride of the division here; `template_file_read`'s entry says 0x44, from its other caller); a 0xE8-byte prototype for each with type 0..231 the version allows (RoE below 165, AB below 222, before SoD types 43 to 45 only subtypes 0 to 2), in the vector of its type's object settings +8; the monsters' vector (54) sorted by subtype with an exchange sort that swaps only when greater |
| `rand_trn.txt` | `rmg_rand_trn_read` 0x536560 | the obstacles, each prototype's row (§2.1) |
| `rmg.txt` | `rmg_templates_read` 0x537FF0 | from row 3, a template per row of two or more columns (name, min and max size) up to the next; kept when the range holds the map's size (width × height × levels / 1296; islands: halved, at least 1) and its zones fit the players, computers and version; connections from column 76 (all nine columns read: §2.1, §3.1) |
| — | `rmg_object_generators_create` 0x538B10 | the object generators, built in code (0x14- to 0x20-byte records into +0x10F0), no file (§2.1) |

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
| 1 | `rmg_zones_lay_out` 0x53BCB0 | once (the template) | a zone generator (0x414 bytes) per template zone; a layout of three passes scaled to the map; each zone's terrain and town types (the count of `rmg_terrain_towns` 0x682450 is always 4, so -1 and the padding can be drawn: settled in §2.2) | 10, 5.5 kB, 4 |
| 2 | `rmg_level_zones_partition` 0x53E050 | per level (template, z) | a triangulation of the level's zone centres, water zones (level 0 with water) and empty points (level 1), the borders drawn and the zones filled (§2.2) | 12, 8.6 kB, 1 |
| 3 | `rmg_terrain_paint` 0x53E6A0 | once | the zones' boxes and centroids, the islands, rock and water painted, then each zone's terrain, by the game's terrain painter (§2.2) | 8, 2.8 kB, 1 |
| 4 | `rmg_zone_first_town` 0x544A50 | per zone | at most one town, nearest the zone's centre: the first kind asked for that is placed (castle and town of its colour, neutral castle and town; §2.2) | 7, 2.8 kB, 1 |
| 5 | `rmg_zone_towns_place` 0x544AE0 | per zone | the remaining least towns, then towns by density until none fits, each farthest from the objects placed (§2.2) | 3, 1.9 kB, 3 |
| 6 | `rmg_paths_and_connections` 0x544920 | once | which cells take obstacles and which stay free (lanes, zone walls, islets in water zones, each zone's free cells joined, unconnected shores sealed), then the template's connections: land passages, shipyards, subterranean gates, monoliths, with their guards (§2.3) | 35, 23.9 kB, 21 |
| 7 | `rmg_junction_zone_paths` 0x5446A0 | per junction zone (type 3) not on water | every empty cell an obstacle, then jagged free paths joining its passage points (§2.4) | 2, 1.4 kB, 1 |
| 8 | `rmg_mines_place` 0x545F60 | once (per zone inside) | each resource's minimum mines (zone +0x4C; the first wood and ore of a start zone with a town near it), then mines by density; each guarded, up to 3 resources beside it (§2.4) | 5, 2.7 kB, 3 |
| — | inline | once | the towns counted, in all and by type | — |
| 9 | `rmg_zone_paths_connect` 0x5405D0 | once (also in step 6, and per monolith border guard) | the zone floods reset; per zone a flood from a free cell and a path carved to every free cell it did not reach (§2.3) | (in step 6's) |
| 10 | `rmg_zone_treasures_place` 0x547360 | per zone | treasure piles by band density: objects chosen by value and density on a 16 × 16 pile, guarded, placed farthest from the objects; a placed Keymaster's Tent or quest artifact sends its Border Guard or seer hut to a near zone (§2.4) | 22 and 8 reached through the objects' vtables, 9.9 + 3.7 kB, 5 + 3 |
| 11 | `rmg_underground_rock_paint` 0x5439E0 | with two levels | level 1's cells neither free nor holding an object painted rock, the zones' free rock cells given back their terrain (§2.4) | 1, 0.6 kB, 0 |
| 12 | `rmg_map_shores_mark` 0x531310 | once, on the map | land beside water flagged (bit 21, §2.4) | 1, 0.3 kB, 0 |
| 13 | `rmg_obstacles_fill` 0x537970 | once | obstacles grown over the cells marked for them (bit 26), chosen by rand_trn.txt's scores; every cell left made free (§2.4) | 4, 3.7 kB, 1 |
| 14 | `rmg_roads_lay` 0x548290 | once | the road type drawn, then every pair of road targets (towns' and shipyards' entrances) joined by a road along a cost flood, later roads reusing earlier ones (§2.5) | 3, 3.2 kB, 1 |
| 15 | `rmg_rivers_lay` 0x549870 | once | rivers: sources (lakes 126, mountains 134, mines of subtype 5) and mouths (the coast, the map's border) marked, then per Water Wheel (109, not a shipyard) a river up to a source and one down to a mouth, with a river delta at the sea (§2.5) | 6, 5.1 kB, 2 |

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
   bytes) and skills (28 bits, 4 bytes) and 156 zero bytes. Just before, the banned artifacts:
   the 17-byte block of 129 bits is AB's, SoD writes 18 bytes as the reader expects (§2.5).
2. A u32 0. The reader's order is the rumours' count, then SoD's 156 hero bytes: the generator
   writes the 156 bytes first, the same 160 zero bytes.
3. Every cell (`rmg_write_cell` 0x532890), level by level, row by row.
4. The templates: those prototypes in use numbered from 2, their count + 2, then the first
   prototypes of types 71 (random monster) and 124 (hole) as templates 0 and 1 (as in the
   generated maps: `AVWmrnd0`, `AVLholg0`), then each in use by its type's object settings group, not sorted by type (§2.5; `rmg_write_template`
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
with it as 9 dwords; the code indexes position + 1); the object generators go to H3API's
`objectGenerators` +0x10F0 (T1a1 read +0x1100; §2.1); the cell's +0x20 is H3API's zone data.

### 1.8 Draws and floats so far; what T1a1 did not read

- **Draws, in stream order:** the dialog's Random choices (§1.1, the game's state); `srand(time)`;
  water's Random; the template; the 15 steps (47 sites in their closures, §1.5 and §2.2 to §2.5; T1a1 counted 44); the teams in the
  header. One generator, one thread, one stream.
- **Floats:** x87 in steps 1, 2, 5, 8, 10 (`crt_sqrt`, `crt_ftol`), and in steps 6 and 7 through
  `vector_length_int` outside the range (§2.6); T1a2 lists each site.
- **Not read in T1a1, read in T1a2:** every function of the range now has an entry (§2.1 to §2.5);
  §2.6 gives the stream's draws in order, the floats and the limits.
- **For T1b:** a generated map names its template and seed in its description, and the
  original's choices are reproducible only without Random ones (§1.1); the oracle can read the
  seed back from the file to check a fixed clock. Two risks found by T1a2: draws that depend on
  uninitialised memory (§2.2, §2.5), and `Untitled.h3m` differing from the plain executable's
  writer in three points (§2.5; the user's HD mod, presumably), so the oracle generates with the
  plain executable.

## 2 Every function placed (T1a2)

Cut by the user (2026-11-23) into five parts of about 22 kB of code each, read one after the
other by worker sub-agents: §2.1 the setup, §2.2 steps 1 to 5, §2.3 step 6, §2.4 steps 7, 8 and
10 to 13, §2.5 steps 14 and 15 and the writer, then §2.6 the stream's draws in order. Each part
names its functions in `docs/exe/map.toml` (at least `inferred`), classifies the game functions
they call, and lists every draw site, every float site and any clock, retry or iteration limit.

### 2.1 The setup (T1a2a)

Path read: `rmg_generator_create` 0x537B10 → `rmg_generator_base_init` 0x536070 (`rmg_map_init`
0x530FB0, `rmg_objects_read` 0x536200 → `rmg_rand_trn_read` 0x536560) → `rmg_templates_read`
0x537FF0 → `rmg_object_generators_create` 0x538B10; `rmg_generator_destroy` 0x537DF0 at the end;
and the methods reached only through the vtables the setup stores (the map's, the generator's, the
object generators'). 72 functions of the range (69 named here, 3 by T1a1) and 2 outside it.

| Function | Called from | What it does | Conf. |
|---|---|---|---|
| 0x537B10 `rmg_generator_create`, 0x536070 `rmg_generator_base_init`, 0x536200 `rmg_objects_read` | T1a1 (§1.3, §1.4) | the constructor, its base, objects.txt | read |
| 0x530FB0 `rmg_map_init` | base init; temporary maps of step 10 and two later functions | the map: vtable `rmg_map_vtable` 0x6409CC, width, height, levels, cells (0x30 bytes) by the C++ array constructor | read |
| 0x530E90 `rmg_map_cell_init` (outside the range), 0x530F10 `rmg_map_cell_reset` | the array constructor; reset also from three steps' functions | a cell's empty list and initial values (`structs.toml`, `rmg_map_cell`) | read |
| 0x530EE0 `rmg_map_cell_destroy` (was `rmg_range_first`), 0x531050 `rmg_map_cell_delete` | array destructors; the steps' temporary maps | frees a cell's list | read |
| 0x536560 `rmg_rand_trn_read` | 0x536200 | the obstacles (below); each prototype's first terrain (+4) and obstacle row (+0x10) | read |
| 0x536BA0 `vector_init_empty`, 0x54C730 `entry4c_vector_insert`, 0x54C910 `entry4c_vector_size`, 0x54CDB0 `dword_vector_erase` | 0x536560 | the library's vector code | read; 0x54C730 inferred |
| 0x537FF0 `rmg_templates_read` | 0x537B10 | the templates, their connections, which are kept (below) | read |
| 0x538480 `rmg_template_zones_read` | 0x537FF0 | a template's zones (the column map below) | read |
| 0x5330A0 `rmg_template_zone_find` | 0x537FF0 | a zone by id | read |
| 0x54C970 `entry1c_vector_insert`, 0x54C1B0 `entry1c_vector_size` | 0x537FF0 | the connections' vector | 0x54C970 inferred; read |
| 0x532FE0 `rmg_template_destroy` | 0x537FF0 (a template refused), 0x537DF0 | zones, connections, name | read |
| 0x538B10 `rmg_object_generators_create` | 0x537B10 | the object generators (below), into +0x10F0 | read |
| 0x534160, 0x534250, 0x5349D0, 0x534EA0, 0x534A60 (`rmg_object_generator_init`, `rmg_creature_generator_init`, `rmg_shrine_generator_init`, `rmg_scroll_generator_init`, `rmg_witch_hut_generator_init`) | 0x538B10 | the generators' constructors | read |
| 0x54CDF0 `byte_vector_insert`, 0x54CFD0 `byte_vector_erase` | 0x538B10 | the byte vector +0x1100 | inferred; read |
| 0x537DF0 `rmg_generator_destroy` | `rmg_generate_and_write` 0x54BF60 | zone generators, templates, generators (plain frees), the vectors, then the base | read |
| 0x532B50 `rmg_zone_generator_destroy`, 0x5363B0 `rmg_generator_base_destroy` | 0x537DF0 | three vectors; placed objects (virtual delete), prototypes, obstacles, templates, map | read |
| 0x536B60 `rmg_obstacle_destroy`, 0x54C940 `rmg_obstacles_destroy` | 0x5363B0; the obstacles' insert | an obstacle's two vectors | read |
| 0x537DC0 `rmg_generator_delete`, 0x536170 `rmg_generator_base_delete` | slot 0 of `rmg_generator_vtable` 0x640C44, `rmg_generator_base_vtable` 0x640C3C | deleting destructors | read |
| 0x5402A0 `rmg_generator_add_object` | slot 1 of 0x640C44: every step's placing of an object | place it, list it (+0xEC4), count its type (+0x1110, and its entrance zone's), flood the distances from its entrance (below) | read |
| 0x5371C0 `rmg_generator_base_add_object` | slot 1 of 0x640C3C (overridden, not reached) | place and list only | read |
| 0x531EA0 `rmg_map_place_object` | both add-object methods; four steps' functions | the object's position and its footprint's cell flags and lists | read |
| 0x54C3F0 `entry12_vector_insert_one`, 0x54C610 `entry12_vector_erase` | 0x5402A0's queue (0x54C3F0 also outside) | the library's vector code | inferred; read |
| 0x530F80 `rmg_map_delete`, 0x5310B0 `rmg_map_destroy`, 0x5361B0 `rmg_map_base_delete` | `rmg_map_vtable`, `rmg_map_base_vtable` 0x6409E8 (six purecall slots), the steps | the map's destructors | read |
| 0x532190 `rmg_map_set_tile`, 0x532200 `rmg_map_set_tile_picture`, 0x532240 `rmg_map_size`, 0x532260 `rmg_map_get_tile`, 0x5322C0 `rmg_map_get_land`, 0x5322F0 `rmg_map_get_tile_picture` | slots 1-6 of 0x6409CC (callers not traced) | land (bits 0-5) and picture (bits 6-13) of a cell of level 0 | read |
| 0x530DA0 `rmg_neighbours_init` (outside the range) | the C runtime's start-up | `rmg_neighbours` 0x69CDC0: E, SE, S, SW, W, NW, N, NE (y down) | read |
| the 23 generator methods (0x534190 to 0x534FD0), 0x546040 `rmg_prototype_draw`, 0x54B100 `rmg_hero_draw` | the 18 generator vtables (below); their callers are the steps' | create an object, value a generator | read |

**`rand_trn.txt`** (one file in both archives, 109 obstacles): from row 3 to the first row whose
column 0 is empty or begins with a space; columns 3 (object type), 4 (subtype), 6 (terrain), 7-15
(the modifiers by terrain, dirt to water; a tenth, -5000, added), 16 to 124 (Adjacent, one per
obstacle), 125 to 233 (Overlap); 0, 1, 2, 5 not read. `rmg_obstacle` in `structs.toml`. A
prototype gets the last row whose type is its type's object settings +8 group
(`object_settings_pointer` 0x660428), whose terrain is its template's first terrain and whose
subtype is its template's; where that row is used was not traced.

**`rmg.txt`'s columns** (from 0; *set*: not empty and not beginning with a space). A template
starts at a row of 2 or more columns; a zone is a row of 76 or more columns with column 3 set; a
connection a row of 85 or more with column 76 set and 77 not empty. Fields in `structs.toml`
(`rmg_template`, `rmg_zone`, `rmg_zone_connection`, all now read).

| Columns | Into |
|---|---|
| 0, 1, 2 | the template's name, least and greatest size (its first row) |
| 3; 4-7; 8 | zone id; type (4 human start 0, 5 computer start 1, 6 treasure 2, 7 junction 3; the last set wins; none: 2); base size |
| 9-12 | the zone's human and total positions, least and most: the zone is dropped unless the players and players + computers fit |
| 13 | the start position, less 1 |
| 14-17, 18-21 | the colour's least towns, castles and their densities; the same for neutral ones |
| 22, 23-31 | towns of the same type; the town types Castle to Conflux (RoE reads 8, clears the ninth) |
| 32-38, 39-45 | least mines and mine densities by resource |
| 46, 47-54 | terrain matches town; terrains dirt to lava (none set: dirt) |
| 55 | monster strength by its first letter: n 0, w 2, s 4, else (empty too) 3 |
| 56, 57, 58-66 | monsters match town, neutral, towns Castle to Forge (RoE clears +0x9D, column 65's) |
| 67-75 | three treasure bands: low, high, density |
| 76-84 | connection: zone ids 76 and 77, value 78, wide 79, border guard 80, positions 81-84 (dropped unless the players and players + computers fit) |

A template is kept when the map's size (width × height × levels / 1296, islands halved, at least 1)
lies in its range and, of the zones kept, the human starts number at least the players and the
start zones (types 0 and 1) at least players + computers; its zones' ids then become 0, 1, … in
row order. A connection naming a dropped zone is dropped.

**The object generators** (`rmg_object_generators_create`; record `rmg_object_generator`: vtable,
type, subtype, value, density, kind's extra from +0x14), appended to +0x10F0 in this order, every
number a constant in the code (subtype 0 unless given; *value/density*):

| Order | Types | Kind (vtable) | Values / density, extra |
|---|---|---|---|
| 1-2 | 2, 4 | base 0x640B64 | 100/20; 3000/50 |
| 3 | 6 | creature Pandora's Box 0x640B7C | one per creature of level ≥ 0, ids downward (118 RoE, 145 AB and SoD); count = `rmg_creature_reward_budget` 0x6824E0[level] / AI value, rounded (over 50 to tens, over 12 to fives, over 5 up to even); value computed (`rmg_creature_generator_value`), density 3 |
| 4 | 6 | experience Pandora 0x640B88 | 5000, 10000, 15000, 20000 valued 6000, 12000, 18000, 24000 / 20 |
| 5 | 6 | gold Pandora 0x640B94 | 5000 to 20000 valued the same / 5 |
| 6 | 6 | spell Pandora 0x640BA0 | levels 1..1 to 5..5 all schools: 5000, 7500, 10000, 12500, 15000; levels 1-5 of one school (bits 1, 2, 4, 8): 15000 each; levels 1-5 all: 30000 / 2 |
| 7 | 10 | Keymaster's Tent 0x640C30 | per tent prototype (subtype, from the last), 5000, 7500, 10000, 15000, 20000 / 10; the byte vector +0x1100 sized to them, cleared |
| 8-12 | 7; 11; 12; 13 (subtypes 0, 1, 2); 14 | base | 8000/20; 100/100; 2000/500; 5000, 10000, 7500 / 20; 100/100 |
| 13 | 16, subtypes 0-6 | base | 3000, 2000, 2000, 5000, 1500, 3000, 9000 / 100 |
| 14 | 17 | dwelling 0x640BB8 (0x640BAC stored first) | subtypes 79 down to 0 (57 for RoE), value computed (`rmg_dwelling_generator_value`), density 40 |
| 15 | 22, 23, 24, 25, 28, 29, 30, 31, 32, 35, 38, 39, 41, 47, 48, 49, 51, 52, 55, 56, 57, 58, 60, 61 | base | 500/100, 1500/100, 4000/20, 10000/100, 100/100, 500/1000, 100/100, 100/50, 1500/100, 7000/20, 100/100, 500/100, 12000/20, 1000/50, 500/50, 250/100, 1500/100, 100/100, 500/50, 100/50, 3500/200, 750/100, 750/100, 1500/100 |
| 16 | 62 | prison 0x640BD0 | experience 0, 5000, 15000, 90000, 500000 valued 2500, 5000, 10000, 20000, 30000 / 30 |
| 17 | 63, 64 | base | 5000/20, 100/100 |
| 18 | 66, 67, 68, 69 | random artifact 0x640B70 | 2000, 5000, 10000, 20000 / 150 |
| 19-21 | 76; 78; 79 (subtypes 0, 2, 1, 3, 4, 5, 6) | resource 0x640BC4; base; resource | 1500/2000; 5000/20; 1400, 1400, 2000, 2000, 2000, 2000, 750 / 300 |
| 22 | 80, 81, 82 | base, Scholar 0x640BDC, base | 100/50, 1500/100, 1500/500 |
| 23 | 83 | seer huts 0x640C00, 0x640C0C, 0x640C18 | per seer prototype (subtype, upward): a creature reward per creature as in row 3 (density 3, value computed), then experience and gold 5000, 10000, 15000, 20000 valued 2000, 5333, 8666, 12000 / 10 |
| 24 | 84, 85, 86 | base | 1000/100, 2000/100, 1500/50 |
| 25 | 88, 89, 90 | shrine 0x640BE8 | 500, 2000, 3000 / 100 |
| 26 | 92 | base | 100/20 |
| 27 | 93 | spell scroll 0x640C24 | levels 1-5 valued 500, 2000, 3000, 4000, 5000 / 30 |
| 28 | 94, 95, 96, 97, 99, 100, 101, 102, 104, 105, 106, 107, 108, 109, 110, 112 | base | 200/40, 100/20, 100/100, 100/100, 100/100, 1500/200, 1500/1000, 2500/50, 2500/20, 500/50, 1500/50, 1000/50, 6000/20, 750/50, 500/50, 2500/150 |
| 29 | 113 | Witch Hut 0x640BF4 | 1500/80 |

The 18 vtables (0x640B64 to 0x640C30, 12 bytes each, map entries `rmg_*_vtable`) hold three slots:
**create** (prototype, generator, an unused third) returns a new object of that prototype (its use
count +8 raised) with the kind's object vtable (0x640A74 to 0x640B54, 13 entries; their methods are
§2.5's writer's); **value** (zone generator, generator) returns the value, or -1 to refuse:
creatures and dwellings of another town than the zone generator's town type (+8) are refused, and
their value is AI value × count (dwellings: × growth), raised by the share of towns of their town
(+0xF64 / +0xF60; dwellings also + AI value × that count / 2); a seer hut's generators only while
the generator's +0xF58 is their prototype and +0x10B4 is clear, a creature reward valued
(2v − 4000) / 3; a Keymaster's Tent only while +0xF5C is its colour; and a **constant test**
(true for seer huts and Keymaster's Tents, false for the rest; its caller not traced). What the
create methods add: a Pandora's Box its creature and count, experience (+0x1C), gold (seventh
resource) or every spell of its levels and schools (in level order downward, spells without flag
0x2000); a prison a hero (`rmg_hero_draw`) or nothing when all are banned, its experience and the
generator's serial +0xF44; a scroll a spell drawn; a seer hut its reward and a quest artifact of
type 65 (subtype 0, for dirt) drawn by `rmg_prototype_draw`: the artifact is the object returned,
the hut hangs on it.

**Placing an object** (`rmg_generator_add_object`, slot 1 of 0x640C44, the steps' only way):
`rmg_map_place_object` writes x, y, z and marks the footprint (the template's +0x38 rows by +0x34
columns, up and left of the position): a cell in the template's +0xC mask gets bit 22 of its +0x28
and, unless its +0x2C bit 0, bit 26 cleared and 27 set; a cell outside the +0x4 mask gets bit 25
cleared; both get the object in their list. Then the object is listed and its type counted; with an
entrance (+0x29) the entrance cell's zone counts it too, and a flood from the entrance writes into
bits 0-15 of every cell's +0x20 on that level the least distance (2 per orthogonal, 3 per diagonal
step, over every cell whatever it holds; the nearest cell first, ties first in, first out).

**The template's masks as read (T4a; RMG-OBJ-009).** `template_file_read` 0x514D80 reads each
`objects.txt` line into a 0x4C-byte template (`rmg_object_template`; its stride is 0x4C for both
callers) through the record reader 0x514B80: the picture's name, two 48-character masks, two
9-character terrain marks, the class, subclass, group and overlay field. A mask is read as a
`std::bitset<48>` (0x515560: the line's character k is bit 47 − k), so character 8r + c is the
cell c columns left of and r rows above the anchor, the bit 47 − 8r − c the placing code tests
(`h3-rmg`'s bit k; `docs/formats/h3m.md`, "Template masks"; VCMI 1.7.5 `ObjectTemplate::readTxt`
reads character 0 as offset (0, 0) too); a terrain mark likewise (character 0 is bit 8, water).
`msk_load` 0x514610 then gives the template the picture's `.msk`: the width and height (+0x34,
+0x38, the footprint; `default.msk`, 1 × 1, without one) and its colour and shadow masks (+0x3C,
+0x44). The two masks the generator tests come from both: **+0x4, passable** = the passability text
OR NOT the colour mask (0x515510, 0x516770: a cell without body pixels is passable whatever the
text says; VCMI's `readMsk` takes the size alone, and 8 × 6 without a `.msk`); **+0xC, the
entrances** = the actions text AND NOT +0x4 (0x514A60): the visitable cells that are not passable.
+0x29 is set when +0xC has a bit; then +0x2C and +0x30 are the first entrance's columns left and
rows up, rows scanned from the anchor's upward, each from the anchor's column leftward (none: 8
and 6, 0x640278). +0x14 holds the third column's terrains, +0x18 the fourth's: every terrain test
of the generator reads +0x18 (`rmg_prototype_draw`, `rmg_mine_place`, the footprint's water test
below), the writer both (VCMI's allowed terrains are the fourth column as well); +0x1C the
class, +0x20 the subclass, +0x24 the group (the seventh column), +0x28 the overlay field not 0.
Placing (`rmg_map_place_object`) is one or the other per footprint cell: an entrance cell gets bit
22 (bit 26 cleared and 27 set unless +0x2C bit 0) and keeps bit 25; any other cell outside +0x4
loses bit 25; both get the object in their list; passable cells are left alone. So bit 22 marks an
object's entrance and a clear bit 25 its blocked body. For the port: the footprint's size and +0x4
depend on each picture's `.msk`, which the rmg packs hold since T4b: each `objects.txt` line's
`width`, `height` and `colour` (48 marks in the passability's order: mark k is the game's bit
47 − k, the `.msk`'s byte (47 − k) / 8, bit (47 − k) % 8), found as `msk_load` finds it (the
picture's name from its last `.` replaced by `.msk`, looked up by `sprite_archive_find` 0x55CF50 in
`H3sprite.lod`, then `H3ab_spr.lod`, the first holding it winning; `default.msk`, 1 × 1 with its
anchor coloured, without one). Every line of both tables has its own `.msk` (AB's 1 221, SoD's
1 326), and no line's passability text blocks a cell without a body pixel, so on the shipped data
+0x4 is the passability text. `h3-rmg`'s `ObjectTemplate` names the third column `editor_terrains`
and the fourth `terrains` (the other way round before T4b). The object settings the generator
reads, +0x0, +0x1, +0x2 and +0x8 (`object_settings_init` 0x41B500: a class no table names keeps
its own class as +0x8 and the flags clear), are harvested into SoD's `[generator]` too
(`GeneratorTables::object_settings`).

**The objects as ported (T4b).** `h3-rmg`'s `objects` module (`Objects`, `Prototype`, `Shape`,
`Object`; each cell's list in `Cells`) ports `rmg_objects_read`, `rmg_map_place_object`,
`rmg_generator_add_object`, `rmg_map_can_place`, `rmg_map_footprint_blocked`,
`rmg_prototype_outline` and `rmg_map_outline_clear`, all read again whole. As read: the filter's
numbers are signed (`c_atoi`'s: a subclass of 2^31 or more passes the monoliths' and gates'
test); the monsters' exchange sort swaps the two prototypes' template pointers when the earlier
subtype is greater, which can reverse equal subtypes (2a 2b 1 gives 1 2b 2a); the part's merged
object table is SoD's `Objects.txt` in line order (`h3-pack-rmg`'s `original_data` example
compares them), the index the phase oracle's dump gives. The add counts the type, then the
entrance's zone (its zone byte an index into +0x10E0, unchecked), then floods: two vectors, the
cells and their distances falling, the last entry taken and the cell's own distance read again
(the queued one unused), a cell lowered strictly below its value (the d + 2 or d + 3 of its
neighbour) inserted by a binary search before the first entry of a distance at most its own.
Precisions: placing tests the map's columns and rows per cell, not the level; the footprint test
reads every footprint cell once its bounds pass (no per-cell test); `rmg_map_can_place` tests only
the row of the cell below the entrance against the height, not its column, its row's lower bound
or its level (the column stays on the map only while the entrance lies within the footprint's
width, true of every shipped line); the outline's four refusals never happen (the walker stands
only on cells it may enter, its start below the footprint), and the walk stays within one cell
around the footprint's box. A bit position past 47 (a footprint past 8 × 6) is `std::bitset`'s
range error `bitset_range_error` 0x41B410, which the original throws (code 1). The port's faults
(`ObjectFault`, kept until the step ends: `GenerateError::Objects`) are the empty outline's
division (RMG-QUIRK-3), that range error, an unchecked cell off the map, an entrance zone byte
past the zone generators and an entrance below without an object listed (RMG-QUIRK-4); none is
reached with the shipped tables.

**Draw sites** (none runs during the setup but water's; the others run when a step creates an
object through these methods):

| Site | Function | Drawn | Per | Condition |
|---|---|---|---|---|
| 0x537C89 `rmg_water_draw` | `rmg_generator_create` | rand() % 3 (T1a1) | once | water Random |
| 0x54B136 `rmg_prison_hero_draw` | `rmg_hero_draw` | rand() % the heroes not banned among 0-127 (RoE) or 0-144 (AB, SoD); the r-th counted from the top; then banned | prison created | one left |
| 0x534F06 `rmg_scroll_spell_draw` | `rmg_scroll_generator_create` | rand() % the spells of the level with a school and without flag 0x2000; the r-th by id | scroll created | always |
| 0x546154 `rmg_prototype_choice_draw` | `rmg_prototype_draw` | rand() % the candidate prototypes (unsigned) | call: here each seer hut's quest artifact; also three steps' functions | one candidate or more |

**Floats:** none (no x87 or SSE instruction in any function of this part).

**Clocks, retries, limits, failures:** the only clock is `time()` for the seed (0x536148,
`rmg_seed_srand`, T1a1). No retry and no iteration cap: every loop runs over a file's rows, a
table or a vector, the flood until its queue is empty. Failure paths: no template kept makes `rmg_generate_map` fail (code 3, §1.5); a
zone or connection outside the player counts is dropped silently; a template too small, too large
or short of start zones is destroyed; a prison is not created when every hero is banned and
`rmg_prototype_draw` returns null without candidates (its callers, T4a: `rmg_mine_place`'s
resources and `rmg_object_choose` test it, the mine then placed without resources and the
generator not kept; the three seer huts' create methods and `rmg_path_carve` use it unchecked,
adding to its +8 count through null: RMG-QUIRK-4's class, reachable only with tables that lack a
quest artifact of type 65 for dirt or a Border Guard of a colour);
the bitset range exceptions (a terrain of 10 or more) cannot be reached from the callers read.
`rmg_scroll_generator_create` and `rmg_creature_generator_init` divide by a count and an AI value
that the game's tables never make 0.

**Outside callees** (43): I/O and files 2 (the text table loader for `rand_trn.txt` and `rmg.txt`,
released through its vtable; `template_file_read` 0x514D80 for `objects.txt`); the C runtime 11
(operator new, free, the array constructor and destructor, the stack probe, the exception throw,
`c_atoi` 0x617F09, `char_to_lower` 0x6172B6, `crt_rand` 0x617E5C, srand, `crt_time` 0x6198E0); the
C++ library 30 (vector inserts, erases, copies and sizes, among them `pointer_vector_insert`
0x54D120, `entry12_vector_insert` 0x4347A0, `vector_insert_u32` 0x404200, `dwords_copy` 0x5093C0,
`empty_ret_8` 0x404140; string construction and assignment; the bitset's range error and its
exception objects; nine vector helpers just past the range's end). No UI call besides the
progress window's vtable (T1a1) and no game object helper: the tables are read in place
(`object_settings_pointer`, `creature_table_pointer` 0x6747B0, `spell_table_pointer` 0x687F58,
`dwelling_creatures` 0x63D570).

**Quirk candidates (as read; classified in `docs/rules/quirks-rmg.md`, T1c2):**

- `rmg_template_zones_read` clears for RoE the monster town byte +0x9D (column 65, Fortress)
  where the town types clear the ninth (Conflux): an off-by-one if the intent was Forge's +0x9E.
  T1c reads what the monster step does with it; T1b can test it with an RoE map of a Fortress zone.
- A seer hut's creature reward of another town is valued -1334, not -1: equivalent only if the
  callers refuse every negative value (not read).
- `rmg_hero_draw` counts 145 heroes for both AB and SoD: SoD's heroes 145-155 never come from a
  prison whatever their version bytes.
- A zone generator's town type can be -1 (§1.5, step 1's candidate): then the neutral creatures
  (town -1) would pass the creature and dwelling value test.
- The distance flood ignores what the cells hold (a reading, not yet a quirk: what reads the
  distance decides).

**Not read:** the growth code of the four `inferred` library inserts; who calls the map vtable's slots 1-6; who reads +0x1100, a prototype's +4
and +0x10 and the distances; the 13 object vtables' methods (§2.5). Answered later (§2.4): the
constant test is applied by the replacements, `rand_trn.txt`'s rows by step 13, +0xF58, +0xF5C
and +0x10B4 set by steps 6 and 10.

**For the next parts:** objects are placed only through the generator's vtable slot 1 (a call
through `[vtable + 4]` with the object and x, y, z); a generator's value of -1 refuses it for the
zone; the steps set +0xF58 (the seer hut prototype allowed), +0xF5C (the Keymaster colour) and
+0x10B4 (seer huts blocked); `rmg_prototype_draw` 0x546040 and `rmg_map_cell_reset` 0x530F10 are
also called from steps' functions; `rmg_neighbours` 0x69CDC0 is read by 18 functions.

### 2.2 Steps 1 to 5: the zones and the towns (T1a2b)

Path read: `rmg_generate_map` 0x549930's calls of steps 1 to 5 and everything they reach by direct
calls; the only indirect calls are the generator's slot 1 (`rmg_generator_add_object`, §2.1) and
the progress window's. 42 functions of the range (37 named here: 32 new, the 5 steps renamed; 5
by §2.1) and 10 outside it.

| Function | Called from | What it does | Conf. |
|---|---|---|---|
| 0x53BCB0 `rmg_zones_lay_out` | step 1 | the zone generators, the layout, the scale, terrains, town types (below) | read |
| 0x5329E0 `rmg_zone_generator_init` | steps 1 and 2 | a zone generator; its town type +4 drawn among the zone's (0x532A5B) | read |
| 0x53B970 `rmg_zone_place` | 0x53BCB0 | one placement: candidates, filter, draw (0x53BB5F) | read |
| 0x53AD60 `rmg_zone_position_fits` | 0x53B970, 0x53AE80, step 2 | the start zones' level and the spacing | read |
| 0x53AE80 `rmg_zone_candidates_around` | 0x53B970 | the points around a placed zone | read |
| 0x53B2F0 `rmg_zone_candidates_filter` | 0x53B970 | a free level first, the most touching, the least span | read |
| 0x532BD0 `rmg_zones_touch`, 0x53B1F0 `rmg_zones_bounds` | 0x53B2F0; 0x53BCB0 | touching; the layout's box | read |
| 0x532AB0 `rmg_zone_terrain_draw` | 0x53BCB0 | a zone's terrain (0x532B00) | read |
| 0x53E050 `rmg_level_zones_partition` | step 2, per level | the triangulation, water zones, borders, fills (below) | read |
| 0x53C390 `rmg_zone_borders_draw` | 0x53E050 | a zone's Voronoi cell on the map: its borders and polygon | read |
| 0x53CAC0 `rmg_segment_clip` | 0x53C390, 0x53D380, 0x53DAD0 | the first point on the map of a segment's line | read |
| 0x53C220 `rmg_line_draw`, 0x53BFF0 `rmg_jagged_line_draw` | 0x53C390 | straight and randomly displaced borders (0x53C0F2) | read |
| 0x53D380 `rmg_zone_flood_fill` (tail 0x53D640) | 0x53E050 | a zone's cells from its centre | read |
| 0x53DAD0 `rmg_water_zones_connect`, 0x53D8E0 `rmg_zone_graph_distances` | 0x53E050; 0x53DAD0 | the new zones' connections; the zone graph's distances | read |
| 0x54CBA0 `point_pair_vector_insert`, 0x54CD70 `point_pair_vector_erase`, 0x54C1E0 `u16_vector_resize`, 0x54C3D0 `u16_vector_size` | the line and distance code | the library's vectors | inferred; read; read; read |
| 0x53E6A0 `rmg_terrain_paint` | step 3 | boxes, centroids, islands, the painting (below) | read |
| 0x53BBB0 `rmg_zone_bounds_compute`, 0x53D0D0 `rmg_zone_centre_to_centroid` | 0x53E6A0 | the zones' boxes; their centres moved to their cells' centroid | read |
| 0x53D1C0 `rmg_island_shape`, 0x53CD30 `rmg_coast_line_draw`, 0x53CF50 `rmg_island_fill` | 0x53E6A0 (islands) | an island: polygon shrunk, coast (0x53CE1D), inside | read (the fill since T3e) |
| 0x544A50 `rmg_zone_first_town` | step 4 | at most one town (below) | read |
| 0x545250 `rmg_town_place_central` | 0x544A50, 0x544D90 | a zone's first town, nearest its centre (0x545492) | read |
| 0x531CF0 `rmg_map_can_place` | 0x545250, 0x544D90; five functions of later steps | whether a prototype fits at a cell in a zone | read |
| 0x5318B0 `rmg_map_footprint_blocked`, 0x532C80 `rmg_prototype_outline`, 0x531170 `rmg_map_outline_clear` | 0x531CF0 (and functions of later steps) | its footprint, outline and ring tests | read (T4a) |
| 0x5330E0 `rmg_object_init` | 0x545250; two functions of later steps | a placed object's base | read |
| 0x544AE0 `rmg_zone_towns_place` | step 5 | least towns, then by density (below) | read |
| 0x544D90 `rmg_town_place_far`, 0x533150 `rmg_object_clear_flags` | 0x544AE0; 0x544D90 | one more town, farthest from the objects (0x544DEC, 0x544E22, 0x545174) | read |

**Step 1, `rmg_zones_lay_out`.** d = the least base size × min(width, height) / 5, 6 or 7 (water
none, normal, islands). Per template zone in order a zone generator: +4 drawn among the zone's
town types (none: -1); a start zone with colour towns or castles whose position's colour has a
town choice takes that choice; placed by `rmg_zone_place` and appended. Then two more passes of
`rmg_zone_place` over all zones, in order. **The layout is an iteration of exactly three passes**
(the creation and two re-placements), no convergence threshold; positions are integers in
template units (base sizes). A placement: no zone yet, (0,0,0) or, with two levels and if it
fits, (0,0,1); else the candidates around each connected zone already created (none: around
every created zone): 32 points (trunc(r × cos k·11.25° + ox), trunc(r × sin + oy), oz) with
r = the two bases' sum, and with two levels also (ox, oy, 1 − oz) and 32 points of radius
max(the two bases) on the other level, each kept if it fits: a start zone (type 0 or 1) on level
1 needs town type +4 Inferno, Necropolis or Dungeon, and every other zone on that level must have
10 × trunc(dist) ≥ 8 × (the two bases). The filter: with two levels and a level holding no other
zone, the candidates on it alone (when one exists at an index above 0); then those touching the
most connected created zones (same level: 11 s ≥ 10 t; across levels s ≥ t and s − t > the smaller
base / 2, t = trunc(dist), s = the bases' sum); then those giving the least max(d, the layout box's
width, height) (the box: x ± base, the far edges + 1, from 0). The zone takes `rand() % count`
of the rest. **Scaling:** the box over all zones, span = its larger side, m = max(width, height),
origin = (min + max − span) / 2 per axis; x, y = (p − origin) × m / span and the size
+0x1C = base × m / span, integer and truncating; z kept. **Terrain per zone:** terrain matches
town with a type +4 → `rmg_town_native_terrain` 0x6408C8 (grass, grass, snow, lava, dirt, dirt,
rough, swamp, grass for Castle to Conflux); else `rand() %` its allowed terrains (subterranean
counted only on level 1; none: dirt); on level 1 every terrain but lava becomes subterranean.
**Town type +8:** +4 when set; else `rmg_terrain_towns`[terrain][`rand() % n`] and **n is always 4**
(re-read: the counting loop stops only at an entry that is -1 and 8 at once, with a version test
`version ≥ 0` that every version passes), so the terminator -1 and the padding 0 (Castle) are
drawn: dirt Castle, Rampart, Necropolis, -1; sand Stronghold, -1, Castle, Castle; grass Castle,
Rampart, -1, Castle; snow Tower, -1, Castle, Castle; swamp Fortress, Necropolis, -1, Castle;
rough Stronghold, Conflux (also in RoE), -1, Castle; subterranean Dungeon, Inferno, Necropolis,
-1; lava Inferno, -1, Castle, Castle; water -1, Castle, Castle, Castle. It applies only to zones
whose +4 is -1 (no town type set and no colour choice), and steps 4 and 5 place towns of +4, not +8.

**Step 1 re-read for the port (T3c; the oracle's `layout.txt` agrees on the corpus).** The zone
generator's constructor draws its town type before the colour's choice replaces it (the draw
happens either way); it writes +0x1C (the base), the box (32000, -32000), +0x3C and the counts,
and leaves +8, +0xC, +0x10..+0x18, +0x30..+0x38 and +0x40 for its callers. A placement writes each
point it tests into the zone's +0x10..+0x18 before `rmg_zone_position_fits` (and each candidate
before its touch count), so the all-zones fallback of passes 2 and 3 starts the points around the
zone itself from the last point tested; `fits` skips the zone itself by its id, the filter's box
and free levels by the pointer. The box filter keeps the least of max(d, h, w) from 32000: a
value above it everywhere (d past 32000) would drop every candidate. The distances are 32-bit
(`imul`): a square past 2^31 wraps, its root is NaN and `crt_ftol` leaves 0 in the low word, so
nothing fits around a zone of base 50 000 (RMG-QUIRK-3's step 1 case without other data). The
second town type's row is the terrain's after the level-1 change: subterranean's or lava's on
level 1.

**Step 2, `rmg_level_zones_partition`, per level z.** The level's zone centres go into a
triangulation (`triangulation_init` 0x5FD010: integer, a frame with corners at -200 and 400;
`triangulation_insert` 0x5FD790 with the zone generator as data; Delaunay, below). On level 1 always, on level 0
only with water: for each zone present at the start, 8 directions (every fourth entry of the
tables): p = (trunc(2 × (s × cos) + x), trunc(2 × (s × sin) + y)), s its scaled size; p off the
map is refused when x + s cos (y + s sin) is off it too; p must fit (`rmg_zone_position_fits` on
a temporary zone of base s); on level 0 a **water zone** at p (a new template zone: type 3,
terrain 8, base s, treasure bands 100-1000 density 5 and 2000-6000 density 1, no towns, mines or
monster settings, position -1), appended to the template's zones and the zone generators; on level
1 a point without a zone (no data). Each p goes into the triangulation. Then
`triangulation_finish` 0x5FDB40 (the Voronoi vertices); per zone of the level, in order, its
borders (`rmg_zone_borders_draw` on `triangulation_find`'s record at its centre: the sides of its
Voronoi cell, clipped to the map, drawn in the lower zone id's index: jagged for the zones present
at the start, limit the smaller of the two sizes, unless islands on level 0; straight otherwise;
its polygon kept in +0x3F4); after every zone's borders, per zone its fill
(`rmg_zone_flood_fill` from its centre over cells of no zone). Cells get the zone index in +0x20
bits 16-23 and +0x28 bit 28 (land), except on the islands' level 0. Then
`rmg_water_zones_connect` (below). On level 1, the points without a zone leave cells of no zone:
rock in step 3.

**Step 2's triangulation (T3b).** D. Lischinski's incremental Delaunay triangulation (Graphics
Gems IV, 1994) step for step, in integers and on half-edges instead of quad-edges. A record
(0x24 bytes, made in pairs) is a half-edge: +0 x, +4 y, +8 data of its origin (a zone generator;
none for the frame's corners and level 1's points), +0xC its twin, +0x10 Onext and +0x14 Oprev
(the ring around its origin, counter-clockwise with y up), +0x18 a mark, +0x1C, +0x20 the
circumcentre of the triangle on its left (-1 until the finish). The triangulation (at the
caller's ebp-0x7C) holds the starting edge +0 and a vector +8..+0x10 of every record in creation
order: each pair as made, the edge before its twin; a deleted pair erased (the order kept); a flip
reuses its pair in place. **Init:** corners a (-200, -200), b (400, -200), c (400, 400),
d (-200, 400), no data; edges a→b, b→c, c→d, d→a (Splice(eₖ.Sym, eₖ₊₁) closes the ring), then
Connect(d→a, c→d) the diagonal a→c; the starting edge a→b. **Predicates:** TriArea(a, b, c) =
(b.x − a.x)(c.y − a.y) − (b.y − a.y)(c.x − a.x) in 32 bits (0x5FDAE0); RightOf(p, e) =
TriArea(p, e.dest, e.org) > 0; InCircle(a, b, c, d) = |a|²·T(b, c, d) − |b|²·T(a, c, d) +
|c|²·T(a, b, d) − |d|²·T(a, b, c) > 0, the squares and areas in 32 bits, each product and the sum
in 64; squared distances in 32 bits (0x5FDB10); Lnext = twin.Oprev, Lprev = Onext.twin,
Dprev = twin.Oprev.twin.
**Find** (Locate) from the starting edge: p at e's origin gives e, at its destination e's twin;
RightOf(p, e): e = twin; else not RightOf(p, Onext): e = Onext; else not RightOf(p, Dprev):
e = Dprev; else e. **Insert** (InsertSite): e = find(p); **a point already a vertex is not
inserted** (its data is lost); p on e (TriArea 0, both squared distances to e's ends at most e's
squared length): e = e.Oprev and its Onext deleted (0x5FD5B0); base = MakeEdge(e.org, p)
(0x5FD390 appends the pair), Splice(base, e), starting edge = base; repeat base = Connect(e,
base.Sym), e = base.Oprev until e.Lnext is the starting edge; then the flips: t = e.Oprev; if
RightOf(t.dest, e) and InCircle(e.org, t.dest, e.dest, p): Swap(e) (0x5FCFA0 detaches it; its
ends become the two far corners), e = e.Oprev; else if e.Onext is the starting edge, done; else
e = e.Onext.Lprev. Both tests are strict: four cocircular points keep their diagonal. **Finish:**
per record of the vector in order, with data and no mark: its left triangle A = its origin,
B = its destination, C = its Onext's destination; u = B − A, v = C − B, w = A − C,
r = (−u.y, u.x)·dot(v, w) / cross(u, w) per axis, centre = A + (u + r) / 2 per axis (both
divisions `idiv`, toward zero); written to the record, its Onext's twin and that one's Onext's
twin (the triangle's three records), each marked. The truncations make the centre depend on
which corner is A, so on the vector's order; a triangle without a zone corner keeps -1 (no
zone's cell uses it). cross(u, w) is 0 only for a flat triangle, which the insertion never
leaves (a point on an edge splits it; a flip needs the strict in-circle test). A port keeps the
records, their order and the starting edge as the original: `find`'s walk, the circumcentres,
the borders' first side and the fill's ties all follow them.

**Step 2's borders (T3b).** `rmg_zone_borders_draw` (the record r at the zone's centre, the
jagged flag): G = r's data (the record's zone, not the caller's), the box [0, W) × [0, H). The
cell's sides in Onext order: side k runs from the centre of rₖ's left triangle to that of
rₖ.Onext's, its neighbour rₖ.Onext's destination's data. **The first side:** from r on, the first
whose a = clip(start → end) lies in the box and differs from b = clip(end → start). None (a cell
holding the whole map, or one that never reaches it): straight lines (W−1, H−1) → (W−1, 0) →
(0, 0) → (0, H−1) → (W−1, H−1) in G's id, those four points appended to +0x3F4, done. **Each
side from the first:** a and b as above; a appended to +0x3F4; a → b drawn in G's id when the
neighbour is none or of a higher id: jagged with the flag (limit the smaller of the two sizes,
G's alone without a neighbour), else straight. If b is the side's unclipped end, the next side.
Else the cell leaves the map at b: the next sides are passed over until one whose clip from its
start, c, lies in the box; then the map's edge from b to c, the current point q = b: if q.x = c.x
or q.y = c.y, a straight line q → c, q appended, and the walk goes on from that side (its a is c);
else a straight line to the next corner and q appended, the corner by q's side: (0, 0) when
q.x = 0, (W−1, 0) when q.y = 0, (W−1, H−1) when q.x = W−1, else (0, H−1); a corner goes on to the
next one in that order. These edge lines are drawn whatever the neighbour, in G's id. The walk ends back at the
first side. **Corner cases as read:** the shortcut takes any shared coordinate, also between two
opposite edges: a cell whose only side on the map is axis-aligned (two centres of equal x or y)
leaves and re-enters at that side's two ends, so the "edge" drawn is a straight chord over the
side (also in the higher id's index, when that zone's turn comes), not the map's edge; b and c on
one edge give the short segment even when the cell goes round the other way. `rmg_line_draw`
writes whatever a cell holds, so a later zone's lines overwrite earlier ones; between a jagged
side and such a straight chord no centre lies, so no fill reaches those cells (no zone: water on
level 0, rock on level 1). The polygon then lacks the map's corners. T3d's step 2 cells against
the oracle's dumps are the check. A zone whose point was not inserted (another vertex at it)
draws that vertex's cell again in that zone's id, with its own flag.

**The clip (T3b).** `rmg_segment_clip` (from p toward q, the box [x0, x1) × [y0, y1)): p in the
box gives p. Else d = q − p and a point s = p moved, in this order, only while beyond each line
and with a divisor not 0: left (s.x < x0, d.x ≠ 0): k = x0 − s.x, s.x += d.x·k / d.x, s.y +=
d.y·k / d.x; top (s.y < y0, d.y ≠ 0): k = y0 − s.y, s.x += d.x·k / d.y, s.y += d.y·k / d.y;
right (s.x ≥ x1): k = x1 − 1 − s.x, as left; bottom (s.y ≥ y1): k = y1 − 1 − s.y, as top
(32-bit products, divisions toward zero; d.x·k / d.x is k unless the product overflows). After a
left or right move a miss when p.y ≥ y0 and s.y < y0, or p.y < y1 and s.y ≥ y1; after a top or
bottom move the same with x; a miss returns p itself. Else s, which may still lie outside (every
caller tests it). It clips the **line** through p and q, both ways from p, never testing q: a side
wholly off the map whose line crosses the map gives a point on the map's edge (the borders' first
side needs a ≠ b, which such a side mostly fails; the edge walk and the water connections take it).

**Step 2's fill (T3b).** `rmg_zone_flood_fill` (G, r): it starts at G's centre when that lies in
the box; else among the destinations of r's ring (r.Onext first, r last), the vertices themselves,
those in [1, W−2] × [1, H−2] scored by min(x, W−1−x, y, H−1−y), the first deepest (none, no fill),
and from the clip from the centre toward it (T3d: T3b had each destination clipped and scored; the
clip's result is used untested: off the map it would index cells outside the level; not reached as
read). A stack of 12-byte seeds (x, y, z), the start first: pop the
last; move left while the left cell is free (zone byte 0xFF); then rightwards while the cell is
free: zone byte = G's id, bit 28 unless islands on level 0; above and below, each run of free
cells gives one seed (its first cell), pushed when the run ends (a cell not free) or at the
span's end (the one above first). The cells filled are the 4-connected free region of the start,
whatever the order (of its left neighbour's when the start is not free and that one is; none
when neither is: a zone whose centre and its left cell lie on borders gets no cell). Zones fill
in generator order after all borders, so a free region joined to two centres goes to the earlier
zone.

**Step 2 as ported (T3d).** The port's step 2 draws and cells equal the phase oracle's on the 24
corpus cases (step 2's bytes as `zones.txt` measured them), its water zones `water.txt`'s. The
borders as read exactly: b is never tested against the box, and a later side's a neither (only the
first side's); "higher id" is strictly higher, signed; the edge walk's re-entry search takes the
first side whose clip from its start lies in the box (no a ≠ b test, no end of the ring: the first
side stops it); the corner rule is (0, 0) when q.x = 0 and q.y ≠ 0, (W−1, 0) when q.y = 0 and
q.x ≠ W−1, (W−1, H−1) when q.x = W−1 and q.y ≠ H−1, else (0, H−1) (also a q on no edge); b and each
corner passed are kept in the polygon, c not (it is the next side's a); q = c draws one cell. So
straight lines can get points off the map, which `rmg_line_draw` indexes unchecked (another row's
cell, or memory past the cells): never in the corpus, a typed error (RMG-QUIRK-4). The jagged
drawer marks the current point clamped to the map and moves on, so a jagged line's last end is not
marked. T3b's corner cases on the corpus (counted in the port, which equals the oracle): a clip
from a side wholly off the map lands on the map 104 times in 19 cases; the edge walk's chord, the
whole frame and a dropped point never occur; 20 zones in 6 cases (13 water, 7 land) fill nothing,
their start and its left cell on borders (the ledger's open table). The temporary zone's town type
is never read (type 3), so only whether it draws counts; a water zone's +0x8 stays the heap's.

**Step 2's water connections (T3b).** `rmg_water_zones_connect` (n, the triangulation), n the
zones before this level's step (on level 1 it counts level 0's water zones; nothing is new
there): (1) per new zone i ≥ n in order, per later zone j on its level: when j's vertex is in i's
ring (the first such record from i's Onext) and that edge's side, clipped from its left
triangle's centre toward its Oprev's, lies in the box, a connection both ways, i's first
(0x1C bytes: the other zone, value 0, wide 1, border guard 0, created 1: step 6 makes no way for
it). (2) Every zone's +0x3E4 resized to n columns (`u16_vector_resize` 0x54C1E0), each 32000,
its own column 0 when its id is below n; `rmg_zone_graph_distances` from each zone below n in
order. (3) Per new zone W ≥ n in order, per zone Z < n in order on its level whose vertex is in
W's ring (no map test): refused when for some column c other than Z's id Z.dist[c] > W.dist[c] + 1
(the connection would bring Z nearer to zone c); else a connection both ways, W's first (wide 1,
created 0: step 6 makes the way), then `rmg_zone_graph_distances` from Z. A water zone's first
neighbour is always taken (all its columns are 32000). `rmg_zone_graph_distances` (S): per column
k, a search from S over the template zones' connections (+0xC4), +1 each, a zone taken only when
strictly nearer, its work list kept sorted by distance (descending, popped from the back, a new
entry before equal ones); its values are the hop counts from zone k whatever the order: no draws,
so a port needs only those values (signed 16-bit).

**Step 3, `rmg_terrain_paint`.** The zones' boxes (+0x20..+0x2C) from the cells; each zone's centre
moved to its cells' centroid; with islands each level-0 zone's polygon shrunk toward the centre
by min(L/2, max(L/4, 4)), its coast drawn jagged (limit size / 2) and its inside filled (bit 28).
Then the game's terrain painter (`terrain_painter_init` 0x5B7250 with a map interface, a terrain
and 4; `terrain_painter_add_rect` 0x5B7690; `terrain_painter_finish` 0x5B72F0, its destructor,
does the work): level 1 all rock (9), level 0 all water (8), then per zone not of water a
painter of its terrain over each of its cells with bit 28. The painter reads cells through the map
vtable's slot 4: the map vtable's slots (§2.1) are its interface. Step 11 calls the
same painter and `terrain_painter_flush` 0x5B7520.
same painter and `terrain_painter_flush` 0x5B7520.

**Step 3 as ported (T3e).** Read again for the port: the boxes start at the constructor's (32000,
32000, -32000, -32000) and every cell of the map with a zone byte of 0 or more widens its zone
generator's (the byte indexes the generators); the centroid compares the signed byte with the
zone's id. An island: the first corner moved, then from the last corner down to the first each
moved corner's coast drawn to the one after it (the first's to the last's); the coast is the
jagged walk of step 2 with k = min(size / 2, L / 2) and a cell reached gets bit 28 only when of
the zone; `rmg_island_fill` (read since T3e) floods from the centre over the zone's cells without
bit 28 through E, S, W and N (`rmg_neighbours`' even entries). Water zones on level 0 get islands
too. The painting: rock only with two levels, then water, then the zones by index (the signed byte
against the index), each painter finished even without cells. On the 24 corpus cases the port's
draws and cells after step 3 equal the oracle's at the first run (the four islands cases too);
RMG-QUIRK-3's and -4's step 3 cases are typed errors (`TerrainFault`, `PaintFault`).

**The placing tests as read (T4a; RMG-OBJ-011).** `rmg_map_can_place` (prototype, x, y, z, zone
generator) is true when, in order: (1) `rmg_map_footprint_blocked` (prototype, x, y, z, the zone's
id, flag 0) is false; (2) `rmg_map_outline_clear` over the prototype's outline (built once by
`rmg_prototype_outline`) is true, its object flag set when the type's object settings +1 and +2
are both set; (3) a template without an entrance passes here; else the cell below the entrance,
(x − +0x2C, y − +0x30 + 1), lies on the map (only its row is tested: the column cannot leave it), is
free (bit 25), not rock, of this zone (its zone byte not negative and equal to the id), without
bit 22 unless the first object of its list has its type's settings +2, and is water exactly when
the zone's terrain is water.

- **The footprint** (`rmg_map_footprint_blocked`, flag the sixth argument: 0 from
  `rmg_map_can_place`, others pass 1): blocked when x < width − 1, x ≥ the map's width, y <
  height − 1 or y ≥ its height; else per cell (x − c, y − r), rows r from 0, columns c from 0 within
  them: an entrance cell (+0xC) is blocked unless free (bit 25), not rock, without bit 22, of the
  zone and, with the flag, without bit 26; then any cell outside +0x4 (the entrances too) is blocked
  unless free, not rock, without bit 22, of the zone and, **on water, the template's group (+0x24)
  is 0 and its fourth-column terrains (+0x18) hold water; on land, not both** (a group-0 template
  with water among its terrains is water-only). Passable cells are not tested. So only group-0
  templates with water in that column put blocked cells on water (the sea objects, whose two
  columns are water alone); every other template's body stays on land.
- **The outline** (`rmg_prototype_outline`, once per prototype, into its +0x14 vector of (dx, dy)
  i32 pairs relative to the position): on the anchor's row, from the anchor leftward, the first
  cell outside +0x4; none (the whole row passable within the width): the outline stays empty. Else
  from the cell below it (dx, 1) a walk of orthogonal steps keeping the blocked cells on its right
  hand: from the direction it came, it tries a right turn, straight on, a left turn, then back
  (`rmg_neighbours`' even entries; a cell may be entered when it lies outside the footprint or is
  passable in it; after four refusals it steps the last way tried), recording each cell before its
  step, until it returns to the first: a clockwise ring (on the screen) of the cells around the
  blocked group the start touches, passable cells inside the footprint included.
- **The ring** (`rmg_map_outline_clear` (outline, x, y, z, object flag, zone generator, kept
  flag): `rmg_map_can_place` passes kept 0): a cell is open when on the map, free, not rock,
  without bit 22, with the kept flag also bit 27, water exactly when the zone is, and of the zone;
  without the object flag a ring cell with bit 22 makes the test false at once. The ring is walked
  cyclically, its first cell twice (n + 1 cells, the state before the first counted closed); a
  second open-to-closed change makes it false; at the end it is true when it saw one such change,
  or none and its first cell is open (the whole ring open), false when none and the ring is all
  closed. So the open cells around the object must form **one unbroken arc, or the whole ring**:
  an object never splits the free cells around it, nor stands walled in. An empty outline divides
  by its count of 0 (the index is i mod n): a fault (RMG-QUIRK-3's class) that no template of the
  classes `rmg_map_can_place` places reaches with the shipped `objects.txt` (every town, shipyard,
  gate, monolith, tent, mine and resource line has a blocked cell on its anchor's row in the text,
  and its `.msk` keeps it blocked: no line blocks a cell without a body pixel, T4b); the piles' own
  outline (`rmg_pile_fits_at`, increment 5's pile ring) is not read.
- **Tents on water** (§2.3's shipyard guards): the Keymaster's Tents are group 0 without water in
  the fourth column, and their two blocked cells (the anchor, which is the entrance, and the cell
  left of it, as the text gives them) refuse water; in a water zone a tent fits only on land cells of that zone (an
  islet's, a sealed shore's: RMG-CON-008, -012) with water below its entrance and an unbroken arc
  of open water around it, else nowhere, and the guard falls back to its monster (RMG-CON-019).

**Steps 4 and 5, the towns.** Step 4 tries, in this order and only if asked for, a castle of the
zone's colour, a town of it, a neutral castle, a neutral town, with town type +4, and **stops at
the first placed: at most one town per zone** (T1a1 read "one of each"). `rmg_town_place_central`
refuses type -1; it takes the type's town prototype (type 98) and, among the zone's cells where
`rmg_map_can_place` allows it, those nearest the zone's centre (squared distance), `rand() %` them;
the town's entrance goes into the zone generator's +0x30 and the road targets +0x14B0, +0x3C is
set. Step 5 places the least counts not yet placed (no spacing) in the same kind order, the first
of the first kind asked for skipped as step 4's; then the densities d (colour castle +0x2C, colour
town +0x28, neutral castle +0x3C, neutral town +0x38): S and P the sum and product of the positive
ones, spacing r = trunc(sqrt(82944 / S)), each kind's step P / d and key step × its least count;
repeatedly the kind not exhausted with the least key (the first on ties) adds its step and places
one more town at spacing r; a kind is exhausted at its first failure; the step ends when all are.
`rmg_town_place_far` redraws the type for a neutral town without the zone's same-type byte or when
+4 is -1 (among the zone's types, else among 8 in RoE, 9 otherwise); a zone without a town yet
goes to `rmg_town_place_central`; else the positions whose entrance cell is in the zone with an
object distance (+0x20 bits 0-15, §2.1's flood) of at least r, the 3 × 3 cells around the entrance
in the zone and `rmg_map_can_place` true, and of those the farthest, `rand() %` them.

**Steps 4 and 5 as ported (T4b).** Read again whole: `rmg_zone_first_town`, `rmg_zone_towns_place`
(its jump table's four cases), `rmg_town_place_central`, `rmg_town_place_far`, `rmg_object_init`,
`rmg_object_clear_flags`, `rmg_zone_generator_init` (+0x3C and the counts +0x44 cleared, +0x30 to
+0x38 left), `rmg_generate_map`'s calls (every zone generator in order, step 2's water zones too)
and its colours' clear. On the 24 corpus cases the port's draws in steps 4 and 5 and its dumps 4
and 5 (the cells and the towns' objects) equal the phase oracle's at the first run. As read,
sharper than above: **the town prototype is the entry at the town type's index** of class 98's
vector (+0x658), no search by subtype (SoD's `objects.txt` files the nine lines in subtype order,
so the index is the subtype, for RoE too: Conflux's line stays filed); the colour is +0xEE4 + 4p
for the zone's position p, and **p = -1 reads +0xEE0**, which `rmg_generate_map` clears to -1 with
the colours (9 dwords from 0x5499FB) and nothing writes: a zone without Ownership gets neutral
towns; the central placement keeps only cells within a squared distance of 32000 of the centre
(its least starts there), a strictly nearer cell clearing those kept; the far placement draws its
redrawn type before it tests for a first town (so a zone whose first town failed draws it and then
takes the central placement), starts at the box's least corner plus the footprint's width and
height, compares each entrance's distance with the greatest kept so far (from the spacing; the
zone byte first, then the distance, then `rmg_map_can_place`, then the 3 × 3 clamped to the map,
a byte not negative and the id), and writes no +0x30; both create the town (0x28 bytes: the serial
+0xF44 taken and raised, the prototype's use count +8 raised, +0x20 owner, +0x24 castle) before
the draw, add it through slot 1, push its entrance to the road targets and mark the cell below it
(+0x28 bit 27 set, 26 cleared, unless +0x2C bit 0). Step 5's kinds read their counts each loop
(constant); the densities' kinds not above 0 keep an unset step and are exhausted from the start;
the least key is taken with `jge` (the first on ties). Step 4 places at the first kind asked for
or at none (the castle flag and owner do not change the fit), so RMG-QUIRK-13's skip differs from
the fix only for a zone whose first town failed: the corpus's two (below) fail step 5 either way,
so no corpus case tells them apart. The port: `towns.rs` (`Placed`: the towns with object, zone,
serial, owner, castle; the road targets), `ZoneGenerator.town` (+0x30 to +0x3C), `Objects::serial`;
`ObjectFault::Prototype` (an index past class 98's vector) and `ObjectFault::Colour` (a position
outside -1 to 7) for RMG-QUIRK-4, with `Outside` for a town line without an entrance (its offsets
8, 6). **What the corpus shows:** RMG-QUIRK-25's two land start zones (`xl1-4p-seed1`'s zone 3,
`xl2-8p-t4-islands`' zone 4, 133 and 225 border cells) get no town in the original: two human
players without a town. The start zones without Ownership (2SM2i(2)'s human starts 3 and 5, 2SM0k's
computer starts 3 to 8) ask for no town of their colour: 2SM2i(2)'s start 3 and 2SM0k's 3 and 4 get
nothing, the others their one least neutral town each (a village of their drawn town type).
RMG-QUIRK-25's fix (`fixed`, `quirk.rmg.fill_start_on_border` off): a fill whose start and the cell
left of it are not free starts at the free cell of the level nearest the zone's centre (squared
distance, the first row by row) outside the free regions in which the later zones' fills start
(their start or its left cell); none, no fill. In `strict-fixed` both start zones then get their
region and their colour's town.

**Draw sites** (10; steps 1 and 2's constructor site is one):

| Site | Function | Drawn | Per | Condition |
|---|---|---|---|---|
| 0x532A5B `rmg_zone_town_type_draw` | `rmg_zone_generator_init` | `rand() %` the zone's set town types, the r-th | zone generator (step 1's, step 2's temporary, each water zone) | one set or more |
| 0x53BB5F `rmg_zone_position_draw` | `rmg_zone_place` | `rand() %` candidates (unsigned) | zone per pass (3 per zone) | always (0 divides by zero) |
| 0x532B00 `rmg_zone_terrain_choice_draw` | `rmg_zone_terrain_draw` | `rand() %` allowed terrains | zone | not following its town; one allowed or more |
| 0x53BFC0 `rmg_zone_terrain_town_draw` | `rmg_zones_lay_out` | `rand() % 4` of the terrain's row | zone | +4 is -1 |
| 0x53C0F2 `rmg_border_jitter_draw` | `rmg_jagged_line_draw` | `rand() % min(limit, L)`, less half | segment split, L > 1 | step 2's jagged borders |
| 0x53CE1D `rmg_coast_jitter_draw` | `rmg_coast_line_draw` | `rand() % min(size/2, L/2)`, less half | segment split, L > 1 | islands, level 0 |
| 0x545492 `rmg_town_central_cell_draw` | `rmg_town_place_central` | `rand() %` nearest fitting cells | first town of a zone | one fits |
| 0x544DEC `rmg_town_type_zone_draw` | `rmg_town_place_far` | `rand() %` the zone's set town types | step 5's town | type redrawn, one set |
| 0x544E22 `rmg_town_type_any_draw` | `rmg_town_place_far` | `rand() % 8` (RoE) or 9 | step 5's town | type redrawn, none set |
| 0x545174 `rmg_town_far_cell_draw` | `rmg_town_place_far` | `rand() %` farthest fitting cells | further town | one fits |

**Float sites** (x87 only; no SSE, no `fistp`; every conversion is `crt_ftol`, truncating toward
zero):

| Function | Computed | Constants |
|---|---|---|
| `rmg_zone_position_fits` (2), `rmg_zones_touch` (2), `vector_length_int` 0x5FCEB0 | trunc(sqrt(f64(dx² + dy²))), the integer fild exact | none |
| `rmg_zone_candidates_around` (20) | trunc(f64(r) × T[k] + f64(o)) in the x87 stack at the control word's precision | `rmg_layout_cos` 0x682500, `rmg_layout_sin` 0x682600: 32 f64 each, cos and sin of k × 11.25° written to four decimals (0.9807 where rounding gives 0.9808) |
| `rmg_level_zones_partition` (44) | s × T[k] stored as f64, doubled, + f64(x), trunc; the bounds f64(p) < s × T and W + s × T ≤ f64(p) by `fcomp` | every fourth entry of the same tables |
| `rmg_zone_towns_place` (2) | trunc(sqrt(f64(82944 / S))), the division integer | none |

The triangulation's functions have no float instruction; the borders and fills are
integer.

**Clocks, retries, limits, failures:** no clock. No retry and no convergence test: the layout's
three passes, 32 (65 with two levels) candidates per placed zone, 8 directions per zone in step 2,
jagged lines split until segments of length 1, step 5's loop until every kind has failed once. A
town that does not fit is skipped without a retry (step 4 moves to the next kind). Failure paths:
an empty candidate list in `rmg_zone_place` and a jitter limit of 0 (a zone of scaled size 0 on a
jagged border, an island zone of size 1) divide by zero, and nothing catches it (§1.2: `rmg_run`
takes only the file's open failure); the terrain painter throws on a failed allocation, also
uncaught. Step 2's own divisions are guarded: the clip tests its divisors, and no triangle of the
triangulation is flat. Step 2's loops end: the insertion's walks (a Delaunay triangulation), the
borders' walk at the first side (one exists, or the whole frame is drawn), the edge walk within
four corners (each move puts one coordinate on an edge line).

**Outside callees** (41): mapped already 7 (`crt_rand`, `crt_sqrt` 0x617B94, `crt_ftol` 0x6179D4,
`pointer_vector_insert`, `entry12_vector_insert`, `vector_insert_u32`, `empty_ret_8`); the
triangulation 6 (0x5FD010, 0x5FD790, 0x5FDB40, 0x5FD6B0 `triangulation_find`, 0x5FD330
`triangulation_destroy`, 0x5FCEB0; only the generator calls them; read with their helpers in
T3b, above); the terrain painter 3 (0x5B7250,
0x5B7690, 0x5B72F0; with 0x5B7520 for step 11); the C runtime 3 (operator new, free, the array
destructor); the C++ library 22 by their callers (vector inserts,
erases, sizes and copies, the 0x1C-byte connection insert, the bitset's range error;
not read). UI: the progress window's +8 twice in step 3 (12500, then 15800 / zones per
zone). No file and no object table besides `object_settings_pointer`.

**Quirk candidates (as read; classified in `docs/rules/quirks-rmg.md`, T1c2):**

- `rmg_terrain_towns`' count is always 4: a zone with no town type gets -1 or Castle often
  (sand, snow, lava half Castle; water three quarters), Conflux on rough also in RoE.
- Uninitialised memory: step 2's temporary zone (on the stack) and each water zone (operator new,
  0xD4 bytes) have town type bytes +0x41..+0x49 that nothing writes before
  `rmg_zone_generator_init` counts them, so its draw 0x532A5B happens or not by what memory
  holds, and the stream after it shifts; a water zone's +0x40, +0x84, +0x94 and +0xC..+0x18 and
  its zone generator's +8 (town type) are not written either.
- Step 2 compares map distances with the existing zones' base sizes in template units
  (`rmg_zone_position_fits`), the temporary zone carrying a scaled size: mixed units.
- Step 4 places at most one town; step 5 skips the first of the first kind asked for even when
  step 4 failed, so a zone without a town type set (+4 -1, refused by `rmg_town_place_central`)
  ends one least town short.
- `rmg_zone_candidates_filter`'s free-level test ignores a candidate at index 0 (no effect found:
  index 0 always lies on an occupied level).
- The divisions by zero above.
- T3b: the clip works on a side's line; the edge walk's shortcut draws chords (cells no fill
  reaches); a cell with no side on the map gets the whole frame; a point at an existing vertex is
  dropped; a zone whose centre and left cell lie on borders gets no cell (above; the ledger's
  open table).

**Not read:** (the placing tests read in T4a, above) the painter's algorithm (its constructor, its rectangle code, the tests inside
`terrain_painter_finish`) and its third value 4; the 22 library helpers.

**For the next parts:** a cell's zone is +0x20 bits 16-23 (the zone generator's index, also its
zone id; -1 none) and +0x28 bit 28 marks the zone's land; level 0 cells without it are water,
level 1 cells of no zone rock. A zone generator's box is +0x20..+0x2C and its centre the
centroid after step 3. Water zones (type 3, terrain 8) follow the template's zones. Placement
tests are `rmg_map_can_place` 0x531CF0 (five later functions call it), objects start with
`rmg_object_init` 0x5330E0, towns fill the road targets +0x14B0. Step 11 uses the terrain painter.

### 2.3 Step 6: the obstacle plan and the connections (T1a2c)

Path read: `rmg_paths_and_connections` 0x544920 (was `rmg_phase_06`) and everything it reaches by
direct calls, `rmg_zone_paths_connect` 0x5405D0 (was `rmg_phase_09`, also step 9) with it; the only
indirect calls are the generator's slot 1 (`rmg_generator_add_object`, §2.1), an object's deleting
destructor and the progress window's. 35 functions of the range (33 named here, 2 by §2.1), one
static initializer, one helper just past the range, six tables, 20 draw sites.

**What step 6 does.** It decides, cell by cell, where obstacles go and what stays free, then
makes the template's connections. Two bits of a cell's +0x28 carry the plan: **bit 26, to be filled
with obstacles** (step 13 counts and fills these cells), and **bit 27, kept free**; a cell with
neither is left to later steps. The markings of bits 26 and 27 skip a cell with +0x2C bit 0 (a
pending border guard) unless they place the guard. H3API's names for bits 25 to 27 do not fit (`structs.toml`, `rmg_map_cell`).

| Function | Called from | What it does | Conf. |
|---|---|---|---|
| 0x544920 `rmg_paths_and_connections` | step 6 | the order below | read |
| 0x543E20 `rmg_free_lanes_draw` | 0x544920 | objects' cells free, empty cells obstacles; one jagged lane per level with side branches (below) | read |
| 0x543C70 `rmg_lane_ray_cast`, 0x531AD0 `rmg_map_mark_free_3x3` | 0x543E20 | a branch's ray to the next free cell or the edge; a 3×3 made free | read |
| 0x531BD0 `rmg_map_mark_obstacle` | 0x543E20, 0x544920 | a cell made an obstacle, its passable empty land neighbours lose bit 27 | read |
| 0x54D0F0 `point_list_node_new` (was `rmg_range_last`, past the range), 0x54D000 `point_list_insert`, 0x54D060 `point_list_erase`, 0x54D0A0 `point_list_erase_range` | 0x543E20 | the branch queue, a linked list of points | read |
| 0x53F880 `rmg_zone_borders_close` | 0x544920 | zone walls (below) | read |
| 0x53F470 `rmg_water_zone_islets`, 0x53F1A0 `rmg_zone_distance_flood` | 0x544920 per zone generator; 0x53F470 | islets in water zones far from their edge (below); the distance flood | read |
| 0x53EFA0 `rmg_islet_draw`, 0x53ED00 `rmg_islet_heightmap`, 0x53E9E0 `rmg_heightmap_quadrants_push`, 0x54C670 `entry24_vector_size` | 0x53F470 down | an islet's terrain, shape and obstacles | read; read; read (T4a); read |
| 0x5405D0 `rmg_zone_paths_connect` | 0x544920, step 9, 0x542B00 | the floods reset; per zone the free cells joined (below) | read |
| 0x531460 `rmg_zone_cost_flood` | 0x5405D0; step 7's `rmg_junction_zone_paths` | a zone's cost flood (below) | read |
| 0x5408E0 `rmg_path_carve` | 0x5405D0, 0x541140 | a path back to the zone's free cells; pending border guards placed | read |
| 0x5378E0 `rmg_map_item` | the connection functions, 0x5408E0, three of later steps | a cell's address (H3API's `GetMapItem`) | read |
| 0x53FCB0 `rmg_water_shores_seal` | 0x544920 | the shores of unconnected water sealed (below) | read |
| 0x543240 `rmg_zone_connections_make` | 0x544920 | the connections, two passes (below) | read |
| 0x5355C0 `point3_init` | 0x543240, 0x541AD0, five of later steps | a point | read |
| 0x541140 `rmg_connection_by_land` | 0x543240 | passages between neighbouring land zones | read |
| 0x541AD0 `rmg_connection_by_shipyard`, 0x541960 `rmg_shipyard_site_fits`, 0x541780 `rmg_boat_reach_flood` | 0x543240; 0x541AD0; both | a shipyard toward the other zone; its site test; the cells a boat reaches (bit 24) | read |
| 0x542080 `rmg_connection_by_gate` | 0x543240 | subterranean gates between levels | read |
| 0x542CE0 `rmg_connection_by_monoliths`, 0x542B00 `rmg_monolith_border_guard` | 0x543240; 0x542CE0 | the last resort: monoliths; a border guard before one | read |
| 0x540D60 `rmg_border_guard_place`, 0x540FC0 `rmg_border_guard_ring_mark`, 0x542930 `rmg_object_place_in_zone` | the connection functions | a Border Guard and its Keymaster's Tent; the ring around it; an object anywhere in a zone | read |
| 0x540B20 `rmg_guard_monster_create` | the connection functions; two of later steps | a guard monster for a value | read |
| 0x541910 `rmg_side_steps_init` | the C runtime's start-up | `rmg_side_steps` 0x69CE00: (-3, 0), (1, 0), (-3, 1), (1, 1), beside a 3-wide footprint | read |

**The order of step 6:** (1) `rmg_free_lanes_draw`; (2) `rmg_zone_borders_close`; (3) every land
cell of no zone, passable, empty, not rock, not yet an obstacle, made one; (4) per zone generator
`rmg_water_zone_islets`; (5) `rmg_zone_paths_connect`; (6) `rmg_water_shores_seal`; (7)
`rmg_zone_connections_make`.

**The lanes.** Every cell holding an object is made free, every empty cell an obstacle. Per level a
line drawn `rand() % 4`: (0,0)–(W−1,H−1), (W/2,0)–(W/2,H−1), (W−1,0)–(0,H−1), (0,H/2)–(W−1,H/2).
A stack of segments: a segment's midpoint ((ax+bx+1)/2, (ay+by+1)/2, truncating) is moved, when
L = trunc(|b − a|) > 1, by (r × nx / L, r × ny / L) with r = `rand() % L` − L/2 and the normal
n = (−dy, dx); a segment whose midpoint is one of its ends marks one end's 3×3 free; one with L ≥ 8
and its midpoint on the map queues two branches, from the midpoint toward ± the normal. When the
stack is empty the next branch is cast (Bresenham, past its target, until it would leave the map's
interior or, from its third step, meet a free 3×3) and drawn as a segment when 5 cells or longer.
Water and rock are then made free, and every obstacle cell takes bit 27 from its passable empty land
neighbours, so a 3-wide lane keeps its middle free and its sides neither.

**The zone walls.** A cell of a zone, not water, becomes an obstacle (even holding an object) when
its 3×3 holds a water cell of no zone, or a cell of another zone to which its zone has no
connection, or a connection that is not wide, or when it lies on level 1: **zones are walled off
except along wide connections on level 0**; the empty cells of its 3×3 lose bit 27.

**The islets.** Per water zone: the distance from the zone's edge (2 orthogonal, 3 diagonal, seeds
every cell around and inside its box not of the zone); while a cell of the box (shrunk to
3..W−4, 3..H−4) lies 20 or more away, one is drawn, a half-size r = 3 + `rand() %` (d/3 − 5), at
most 6, and an islet painted in [x−r, x+r) × [y−r, y+r): terrain `rand() % 6` (dirt to rough), shape
the positive cells of a midpoint-displacement height map (corners 0, amplitude (w+h)/4 + 1 then
(a−1)/2 + 1 per level, five displacements `rand() % a − a/2` per rectangle split while a > 1, the
quadrants last in, first out, the low-low one first), painted by the terrain painter
(`terrain_painter_init` 0x5B7250, `terrain_painter_add_rect` 0x5B7690, `terrain_painter_finish`
0x5B72F0); **every land cell of the box then becomes an obstacle**; the distance flooded again
from the islet's centre.

**The height map as read (T4a; RMG-CON-008).** `rmg_islet_draw` sizes a byte mask to the box (W =
x1 − x0 columns, H = y1 − y0 rows) and `rmg_islet_heightmap` fills it row-major (index row × W +
column). Its records (0x24 bytes, `rmg_heightmap_rect`): the first row, the first column, the row
end, the column end (ends exclusive), the corner heights top-left, top-right, bottom-left,
bottom-right, the amplitude. The whole box is never a record itself: with a = (H + W) / 4 + 1 its
four quadrants are pushed at once, with corners 0, edge midpoints 0, the centre a / 2 and amplitude
a, and no draw. A popped record of one cell writes its top-left height clamped to 0..255 into that
cell (when inside the mask); a larger one takes the means of its edges' two corners (top:
top-left and top-right; left; bottom; right) and of all four, and, while its amplitude a > 1, adds
`rand() % a − a / 2` to each in the order top, left, bottom, right, centre (the sites 0x53EEFF to
0x53EF43); its quadrants get amplitude (a − 1) / 2 + 1. Every division here is C's, toward zero.
`rmg_heightmap_quadrants_push` cuts at the middle row (first + end) / 2 and the middle column, gives
each quadrant the heights at its own corners, and appends them bottom-right, bottom-left,
top-right, top-left, skipping one of no row or no column; the stack is last in, first out, so the
top-left one is split first (the fourth append inlines the vector's growth, capacity size +
max(size, 1): no effect on the order). A cell belongs to the islet when its height is above 0; the
cells go to the painter row by row.

**Joining each zone's free cells** (`rmg_zone_paths_connect`, also GenerateMap's step 9). Every
cell's costs reset (+0x1C 32000 in both halves, the previous cell +0x10 −1, +0x20 bits 24-31 −1,
+0x28 bits 12-14 0). Per zone generator in order: the first cell of its box (row by row) in the zone,
empty, free, passable, not rock (water only in a water zone) seeds `rmg_zone_cost_flood` (none:
the last empty cell of the zone tried is made free and seeds it). The flood: least cost first
(a sorted vector); 8 steps, or only E, SE, S, SW, W out of an object's cell whose type's object
settings byte +1 is clear; into a cell of a zone, passable, not rock, an object's cell only as its
settings bytes +0 and +2 allow and not by SE, S, SW when +1 is clear. Inside the seed's zone +1 per
step, +10 onto water, and **0 from a 0-cost cell onto a free cell** (water only for a water zone):
cost 0 marks the free cells joined to the seed. Into a neighbouring zone +10 per step, in +0x1C bits
16-31, with +0x20 bits 24-31 = the seed's zone and the direction back in +0x28 bits 12-14 (from the
seed's zone or inside that zone only). Then every free, passable cell of the zone, not rock or water, whose
cost is not 0 gets `rmg_path_carve`: back along the previous cells while the cost is above 0 (none below
30000 at the start), each made free, the obstacles of its 3×3 in the zone cleared (without the
border-guard flag), a pending Border Guard placed where one is marked; then a new flood from that
cell.

**Sealing water shores** (`rmg_water_shores_seal`). A water cell of a water zone reached by a
land zone's flood, with a passable non-obstacle land cell in its 3×3, when the water zone has no
connection to that land zone: its 3×3 becomes obstacles, its 5×5 empties lose bit 27, and its water
cells are painted to that land's terrain: water cannot be used to reach a zone it is not connected
to.

**The connections** (`rmg_zone_connections_make`). The crossing cells: every land cell, passable,
entered by another zone's flood, whose cell back is land of another zone. **Pass 1**, per land zone
generator in order (the boat reach, bit 24, cleared on its level first), per connection of its zone in order not yet made (+0xA): by land, else by a
shipyard (only this end marked), else, the other zone being land, by gates. **Pass 2**, per land
zone with one left: the boat reach (bit 24) cleared on its level and flooded from the water beside
each of its shipyards; per connection left: by a shipyard, else, the other zone being land, by
monoliths, **both ends marked made whatever was placed**. The ways:

- **By land** (same level, both land): the crossing cells P of this zone entered by the other's
  flood, P and its cell back Q empty, n of them; the candidates are those of least entering cost
  (at most 100); none: refused. k = min(candidates, (n + 39) / 40) passages, each a candidate drawn:
  paths carved from P and from Q, P and Q into the zones' +0x404, the candidate removed.
- **By a shipyard** (same level): a shipyard prototype drawn first; this zone's cells entered by
  the other's flood: one already reached by boat ends it (made); else per land cell the positions
  x..x+2 of its row where the shipyard fits (`rmg_map_can_place` 0x531CF0 and the site test: a
  3 × 2 land block, a free water cell beside it by `rmg_side_steps`, land on the other side), one
  drawn; the shipyard placed (`rmg_object_init` 0x5330E0, `rmg_dwelling_object_vtable` 0x640AA4),
  its entrance a road target (+0x14B0), the row below it free, the boat reach flooded.
- **By gates** (different levels, this zone land, their boxes overlapping): a gate prototype
  drawn first; the (x, y) of the overlap in this zone on its level and in the other on its, both
  fitting, of greatest sum of the two cells' object distances (§2.1's flood); one drawn; a gate on
  each level, the cells below the entrances free.
- **By monoliths** (both land): an index cycling over the two-way monolith prototypes (type 45),
  then the one-way entrance and exit pairs (types 43, 44); an entrance placed at random in each zone
  (`rmg_object_place_in_zone`), for one-way also an exit in each; listed in +0x14D0 or +0x14C0.
  **The pairing as read (T4a; RMG-CON-017):** nothing pairs them at write. Each monolith is
  written as its template (class and subclass, the channel), and the game sends an entrance to one
  of its channel's exits (ADV-TEL-002, -003). The index i starts at 0 and becomes (i + 1) mod (n45
  + n43) after each monolith connection (the two prototype lists' sizes, 0x543957); i < n45 takes
  two-way prototype i for both ends, else entrance prototype j = i − n45 (type 43) and exit
  prototype j (type 44): the j-th lines of their classes in `objects.txt` (the version filter keeps
  subclasses 0 to 2 before SoD), whose subclasses agree in the shipped tables (AB 0 to 2, SoD 0 to
  7). The order: the entrance in this zone, then its Border Guard or the monster below it; the
  entrance in the other zone, the same; then for one-way an exit in this zone and one in the
  other, unguarded, both listed in +0x14C0; every monolith placed puts its position into its zone's
  +0x404. So a one-way connection is one channel with an entrance and an exit in each zone, each
  entrance leading to either exit (its own zone's among them), and from the (n45 + n43 + 1)-th
  monolith connection on a channel is used again, its monoliths shared with an earlier
  connection's.

**The guards.** Value g = 0 for a wide connection, else from the connection's value v and the
generator's monster strength s (+0x10BC): g = (v − `rmg_guard_threshold_low`[s]) ×
`rmg_guard_factor_low`[s] / 4 when v is above that threshold, plus (v −
`rmg_guard_threshold_high`[s]) × `rmg_guard_factor_high`[s] / 4 when above that one, and 0 when
under 2000 (tables 0x6823F0, 0x682408, 0x682420, 0x682438 by s = 0..5: thresholds 50000, 2500,
1500, 1000, 500, 0 and 50000, 7500, 7500, 7500, 5000, 5000; factors 0, 2, 3, 4, 6, 6 and 0, 2, 3, 4,
4, 6; T1c compares them with VCMI's). With the connection's border guard flag: `rmg_border_guard_place`
on this zone's side with the tent in the other zone, then on the other side with the tent in this
one (a land passage: at P then at Q; gates: below each; a shipyard: three guards before its
entrance, the tent in the water zone; a monolith: before it, after `rmg_zone_paths_connect` runs
again); a guard takes the next Keymaster colour (+0xF5C, marked in +0x1100), its tent is placed at
random in the given zone, its 3×3 (gates: the two cells beside; monoliths: five cells) becomes
obstacles marked with the colour, and once one is placed g = 0. Else, when g > 0, a guard monster
(`rmg_guard_monster_create`): a land passage one, at P or Q by `rand() & 1`; a shipyard one, below
its entrance; gates one below each; monoliths one below each entrance. The monster: the creatures
of an allowed town (the zone's town type when its monsters match it, else its neutral and town bytes
+0x95..+0x9E), level 0 or more, with AI value × (map low + high) / 2 ≤ g ≤ AI value × 100, one
drawn; count n = (g + AI/2) / AI, then n + `rand() % s` − `rand() % s` with s = n/4 + 1 when s > 1;
the object takes the serial +0xF44.

**Step 6's plan as ported (T4c).** `h3-rmg`'s `plan` module, parts (1) to (6) of the order (the
lanes, the walls, the cells of no zone; the islets, `plan/islets.rs`; the joining, `plan/join.rs`;
the shores' seal, `plan/seal.rs`): on the 24 corpus cases the port's draws after step 5 are
`plan.txt`'s (count and digest: the lanes, 101 islets in 4 cases, the sealed shores' painters in
11), and after the plan its floods (+0x10 to +0x18, +0x1C, +0x20 bits 24-31, +0x28 bits 12-14),
terrain and pictures equal step 6's dump on every cell (the connections leave them). Read again,
beyond the above:

- The lanes' first pass tests a cell's object list (+0x4 to +0x8 not empty), not bit 22: an
  object's passable cells off its list stay obstacles.
- The segment stack holds points in pairs (p, q), q on top; a split pushes q, m, m, p, so (m, p) is
  drawn before (q, m), and each child has its parent's end in the other place. A terminal segment
  (m = p or m = q) marks p's 3 × 3 only, and only when p lies on the map: a lane's end whose last
  segment starts off the map (a midpoint displaced past the edge) can stay an obstacle.
- A segment of length 1 whose midpoint is neither end ((1, 0)–(0, 1): the midpoint (1, 1)) splits
  without a draw (L ≤ 1); the split's tests use the displaced midpoint (on the map, for the
  branches).
- The branch queue is a `std::list` filled at its back and taken from its front, two points at a
  time: the branches run in the order queued, (m, m + n) before (m, m − n), and only once the
  segment stack is empty; a cast branch's segment is pushed as (end, from).
- `rmg_lane_ray_cast` 0x543C70: the major axis is |dy| on ties; the diagonal step's signs are
  + for a delta above 0, else − (on both axes); the interior is 1 ≤ x < W − 1, 1 ≤ y < H − 1 (the
  map's +0xC, +0x10); a first step out returns the start (no segment: distance 0).
- `rmg_map_mark_free_3x3` writes the cell, then the 3 × 3 clamped to the map with it, each unless
  +0x2C bit 0; `rmg_map_mark_obstacle` indexes its cell unchecked (every caller passes one on the
  map) and clamps its 3 × 3 (the cell itself among them).
- The lanes' last pass is one sweep: per cell, water or rock made free, then a bit-26 cell marked
  an obstacle; the two never meet (the mark skips water and rock).
- The walls take the zone generator by the cell's zone byte (+0x10E4, unchecked: RMG-QUIRK-4;
  step 2 writes no byte past them, the port leaves such a cell unwalled) and its template zone's
  list +0xC4, the first connection whose far end's id (+0x0's +0x0) is the neighbour's zone byte.
  On level 1 any other zone's cell walls, connected or not (RMG-QUIRK-9). A neighbour of no zone
  walls only when it is water. The 3 × 3's
  bit 27 is cleared on every cell without an object and without +0x2C bit 0, water, rock and other
  zones' cells included (no land test, unlike `rmg_map_mark_obstacle`); a second pass over
  [y, y + 1) × [x, x + 1) re-marks the cell alone when empty, which changes nothing (not ported).
- The cells of no zone (0x544937 to 0x5449F7): bit 26 clear, bit 25 set, land not 9, bit 22
  clear, no object, +0x20 bit 23 set (the zone byte negative), land not 8; marked by
  `rmg_map_mark_obstacle`.
- The islets' reset of the box also clears +0x28 bits 12-14 and +0x20 bits 24-31. The seeds are
  the cells of the box widened by one and clamped to [0, W) × [0, H), row by row, whose zone byte
  is not the zone's id (the template zone's +0x0; the kept zones are renumbered from 0 in the
  generators' order, so it is the generator's index): one `rmg_zone_distance_flood` call each, a
  later one entering a cell only below its cost. The candidates: columns max(x0, 3) to
  min(x1, W − 4), rows alike, ends exclusive; rebuilt after every islet. d / 3 is unsigned, the
  modulus (d / 3 − 5) signed; the islet's box is [max(x − r, 0), min(x + r, W)) and alike in y.
- `rmg_zone_distance_flood` keeps two vectors (the points, their costs) sorted from the highest:
  a binary search puts a new entry before its equals (equal costs are taken first listed first),
  the last is taken and expands at the cell's current cost, so a cell entered twice is listed
  twice (harmless). The seed's zone is not tested; its +0x1C high half, bits 12-14 and +0x20's
  top byte are cleared. A step leaves the map (tested) or the level never; a cell of another zone
  byte is skipped before its cost is read.
- `rmg_islet_draw` draws the terrain before the height map; its painter has a view of the level
  (one level, `rmg_map_vtable`) made before the height map (no draw), is finished, never flushed.
  The mask is `operator new`'s, unset, but every cell is written by exactly one one-cell record;
  the height map's bounds tests (a one-cell record inside the mask, a larger one meeting it)
  never fail (not ported). The quadrants' corners (top-left, top-right, bottom-left,
  bottom-right): bottom-right (centre, right, bottom, the record's), bottom-left (left, centre,
  the record's, bottom), top-right (top, the record's, centre, right), top-left (the record's,
  top, left, centre); a one-row record's top quadrants are empty, so its cells take its
  displaced left and centre, not its corners. The obstacles' loop tests the land byte
  (+0x24 & 0x3F) against 8 and +0x2C bit 0, per cell of the box.
- The islets' painters are of land: `terrain_painter_set_cell`'s water and rock branch and the
  split ring's mending are not reached (they are the painter's own terrain's), only the water
  cells' ring test in the pinch and the diagonal queue; they draw as the original's (no fault in
  `painter.rs`). A painter's fault there (a shape its set lacks) is `GenerateError::Paint` with
  step 6 after the joining (RMG-QUIRK-3).
- `rmg_zone_paths_connect`'s reset writes all three words +0x10 to +0x18 as -1 and +0x1C's low
  half as 32000 too. A zone generator's cells are those of zone byte k, its index (the generator's
  +0x10E4 order), over its box (+0x20 to +0x2C) on the level of its +0x18. The seed's loop keeps
  the coordinates of every cell of the zone holding no object (water only for a water zone
  generator: its +0xC), before testing bits 27 and 25 and rock: the fallback is the last of those,
  made kept free unless +0x2C bit 0 (bits 25 and the land left), and the flood's water flag is the
  zone generator's terrain being water. The carving loop reads the box again, row by row, at each
  cell's current state (the carves and floods before it moved them).
- RMG-QUIRK-5 as read: a zone generator with no such cell (a box of no row: a water zone of step 2
  without a cell, `l1-4p-c`'s ninth of 18, at x = −14) keeps the coordinates last tried, the seed
  of the last zone generator that had one; the fallback's mark (that cell is kept free already)
  and its flood (a seed of cost 0 whose zone's floods are done: every cell was expanded at its last
  cost, the conditions unchanged, so its first expansion enters nothing) change nothing; the
  carving loop runs as for any zone. Only with none before (the first zone generator) are they the
  stack's bytes. The port makes no seed in the first case and refuses the second
  (`GenerateError::NoEmptyCell`); the ledger's "paths through a foreign zone" does not occur: the
  carving reads the zone's own cells. `l1-4p-c`'s floods equal step 6's dump on every cell.
- `rmg_zone_cost_flood` reads the seed's zone byte (signed) as Z and clears only the seed's
  +0x1C low half and +0x10 to +0x18 (its high half, bits 12-14 and +0x20's top byte kept). Its two
  vectors (the points, 12 bytes; the costs) are kept as `rmg_zone_distance_flood`'s; the listed
  cost is never read. A taken cell's directions run from `rmg_neighbours`' index 7 down to 0, or
  from 4 when it is an entrance (+0x28 bit 22: the first object of its list, its prototype's
  template's class +0x1C, `object_settings_pointer` 0x660428's 16-byte row) with byte +1 clear;
  a neighbour's tests in order: on the map's columns and rows (the level unchecked), +0x20 bit 23
  clear, bit 25 set, land not 9, then for an entrance +0 set with +2 clear refused and +1 clear
  with index 1 to 3 refused. Into another zone the cost is the taken cell's + 10, the test
  unsigned against the high half (`jbe`), the step written ((k << 12) − 0x3001)'s bits 12-14, that
  is (k + 4) mod 8, and +0x20's top byte Z; no previous cell. Into Z (only from Z): + 1, + 10
  onto land 8, the test against the low half before the zero rule (a cell at 1 beside a cell of 0
  stays 1, re-flooded or not), the zero rule's water test the land's low byte against 8 with the
  flag. Every cost written is below the 32000 it started at.
- `rmg_path_carve` reads its start's zone byte and cost first (none below 1 or from 30000 on: no
  walk); per cell: a pending guard placed (not ported, RMG-STUB-1: +0x2C bit 0 is set only by
  the connections' Border Guards, so none is pending in step 6's joining; step 9's and
  0x542B00's need it), else the cell kept free without a +0x2C test; then with the flag clear its
  3 × 3 clamped to [0, W) × [0, H) (the generator's +0x18, +0x1C), cells of the start's zone byte
  without +0x2C bit 0 losing bit 26 (bit 27 not set); the next cell is the previous one, indexed
  unchecked, the walk going on while its low half is above 0 (`ja`). Every cost along a chain of
  previous cells is below the one before (a cell's cost and previous cell are written together,
  from a cell then cheaper whose cost only fell since), so the walk ends at a cost of 0.
- `rmg_water_shores_seal` walks the cells with a pointer from the first (+0x14), level by level,
  row by row; per water cell: the zone byte, the land's low byte against 8, +0x20's top byte as a
  signed F (any zone's flood, a water zone's too; the reset's −1 none), the 3 × 3's first land
  (not 8 or 9, bit 26 clear, bit 25 set) row by row, its land sign-extended from 6 bits; then the
  zone generator by the zone byte (+0x10E4, unchecked: RMG-QUIRK-4; the port leaves a byte past
  them unsealed) and its zone's +0xC4 list, 0x1C bytes an entry, its first word the far zone,
  whose +0x0 is compared with F. A seal marks its 3 × 3's cells (any land, objects or not) unless
  +0x2C bit 0 and lists its water cells (pending ones too) with the land; its 5 × 5 tests the
  object list (+0x4 to +0x8) and +0x2C bit 0. The level's painter is made after the level's last
  row (a view of the level, `rmg_map_vtable`), with the first listed land; per listed cell a flush
  `(land, 4)` when its land differs from the current one, then `terrain_painter_add_rect(x, y, 1,
  1)` (its z overwrites the level counter with the same level); finished, freed, then the next
  level. So the marks all precede the painter: its 16-bit copy (TRAPS) equals the map throughout.
- `terrain_painter_flush` 0x5B7520 drains the sets as `terrain_painter_finish` does (set 1's
  first, `std::set`'s leftmost; set 2's first erased by key), with the set-2 word load inlined
  (`terrain_painter_cell_word` loads as well: the same), runs the last pass, and sets +4 and +8; the
  sets are empty after it, the cell words kept.

**Draw sites** (20 here and one of §2.1; in the order they run):

| Site | Function | Drawn | Per | Condition |
|---|---|---|---|---|
| 0x543EB8 `rmg_lane_orientation_draw` | `rmg_free_lanes_draw` | `rand() % 4` (signed form) | level | always |
| 0x54403E `rmg_lane_jitter_draw` | `rmg_free_lanes_draw` | `rand() % L` | segment split | L > 1 |
| 0x53F745 `rmg_islet_centre_draw` | `rmg_water_zone_islets` | `rand() %` cells at distance ≥ 20 (unsigned) | islet | water zone, one such cell |
| 0x53F794 `rmg_islet_radius_draw` | `rmg_water_zone_islets` | `rand() % (d/3 − 5)` | islet | always after it |
| 0x53EFE4 `rmg_islet_terrain_draw` | `rmg_islet_draw` | `rand() % 6` | islet | always |
| 0x53EEFF, 0x53EF0C, 0x53EF19, 0x53EF2E, 0x53EF43 `rmg_islet_height_draw_1` to `_5` | `rmg_islet_heightmap` | `rand() % a` each: edge midpoints 0-1, 0-2, 2-3, 1-3, the centre | rectangle split | a > 1 |
| 0x546154 `rmg_prototype_choice_draw` (§2.1) | `rmg_prototype_draw` from `rmg_path_carve` | a Border Guard prototype of the colour | pending guard crossed | a path through a marked cell |
| 0x54146B `rmg_passage_cell_draw` | `rmg_connection_by_land` | `rand() %` least-cost crossings (unsigned) | passage | always |
| 0x542A8F `rmg_zone_object_cell_draw` | `rmg_object_place_in_zone` | `rand() %` fitting positions (unsigned) | tent or monolith | one fits |
| 0x541673 `rmg_passage_guard_side_draw` | `rmg_connection_by_land` | `rand() & 1` | passage | g > 0, no border guard |
| 0x540C61 `rmg_guard_creature_draw` | `rmg_guard_monster_create` | `rand() %` fitting creatures (signed), the r-th from the top | monster | one fits |
| 0x540CCE, 0x540CD8 `rmg_guard_count_plus_draw`, `_minus_draw` | `rmg_guard_monster_create` | `rand() % s` twice | monster | s > 1 |
| 0x541B78 `rmg_shipyard_prototype_draw` | `rmg_connection_by_shipyard` | `rand() %` shipyard prototypes (unsigned) | call | same level, before any test |
| 0x541D6C `rmg_shipyard_site_draw` | `rmg_connection_by_shipyard` | `rand() %` sites (unsigned, duplicates counted) | shipyard | a site, no boat reach yet |
| 0x5421BC `rmg_gate_prototype_draw` | `rmg_connection_by_gate` | `rand() %` gate prototypes (unsigned) | call | boxes overlap, before any site |
| 0x542387 `rmg_gate_site_draw` | `rmg_connection_by_gate` | `rand() %` best gate positions (unsigned) | gate pair | a position |

Within a connection the order is: the way's draws (prototype, site, or per passage the cell), then
per guard the tent's position, or the side and the monster's three draws.

**Float sites:** none in the 35 functions (no x87 or SSE instruction); the lanes' length is
`vector_length_int` 0x5FCEB0 (§2.2's float table, trunc(sqrt)) once per segment split.

**Clocks, retries, limits, failures:** no clock. No retry, no cap: the lanes run until the stack and
the branch queue are empty (segments shrink, branches stop at free cells); the islets until no
cell is 20 away (each islet's flood zeroes its centre); `rmg_zone_paths_connect` one pass per zone,
one carve and flood per free cell not joined; a land connection makes ceil(n/40) passages at most;
each connection is tried once per pass. **Give-ups:** a connection with no way is left silently (a
water connection without a shipyard site; a land one whose monoliths find no room is still marked
made); a border guard that cannot be placed falls back to the monster; a guard under 2000 is none.
Faults reachable only with other data: `rand() % 0` with no shipyard or gate prototype, a modulus
0 in the monolith index without monolith prototypes, `rmg_border_guard_place` returning 0 (colour
0) when a tent exists but no Border Guard of its colour, `rmg_path_carve` using a null prototype,
`rmg_monolith_border_guard` marking five cells without a bounds check (a monolith on the map's edge
wraps to the next row or level), `rmg_connection_by_monoliths` reading the exit list at the
entrance's index unchecked, `rmg_zone_paths_connect`'s fallback seed keeping the previous zone's
coordinates when a zone has no empty cell (reached with the shipped data, T4c: `l1-4p-c`, and
harmless there; the stack's bytes only for a first zone generator without one). Progress: +1600 (the walls), +1000 per islet, +20 per
row (the shores), +6400 (the connections).

**Outside callees** (23): mapped already 10 (`crt_rand` 0x617E5C, `vector_length_int`, the terrain
painter's four with `terrain_painter_flush` 0x5B7520, `entry12_vector_insert` 0x4347A0,
`pointer_vector_insert` 0x54D120, `vector_insert_u32` 0x404200, `empty_ret_8` 0x404140); named here
1 (`point_list_node_new`); the C runtime 3 (operator new, free, the array destructor); the C++
library 9 by their callers (a u32 vector's insert and erase, a 12-byte vector's range erase,
destructor and copy, an 8-byte point vector's insert, a 0x24-byte vector's insert and two helpers;
not read). UI: the progress window's +8 only. No file. Tables read in place:
`object_settings_pointer` 0x660428 (the flood), `creature_table_pointer` 0x6747B0 (the monster).

**Quirk candidates (as read; classified in `docs/rules/quirks-rmg.md`, T1c2):**

- Border guards are tried on both sides: at P with the tent in the other zone, then at Q with the
  tent in this one (gates likewise); when both succeed the passage holds two guards in a row, the key
  of the guard on this side lying in the other zone and the other's key in this one, so neither side
  passes. Once one guard is placed, g = 0 for the connection's later
  passages too.
- A shipyard's border guards seek their tent in the water zone.
- Level 1 zones are walled from each other even along wide connections.
- RoE guard monsters: creatures 0-116 are filtered and counted, but the draw walks down from 144,
  so creature 117 (when it has a prototype) can be drawn without being counted, shifting the choice;
  a creature that passes the tests without a monster prototype is counted but cannot be walked to.
- The guard counts per way differ: one monster per land passage, one per shipyard, two for gates and
  for monoliths, each of the full value.
- The shipyard and gate prototypes are drawn before any site is sought, so a refused way still
  draws; shipyard sites found from neighbouring cells are counted more than once.
- `rmg_connection_by_land`'s shortcut for a least cost of 1 is dead (entering costs are 10 or more).
- The islet candidates are not restricted to the zone's cells (harmless: the others are seeds at 0).
- T4a: a one-way connection puts both its entrances and both its exits on one channel, so an
  entrance may lead back into its own zone; and the monolith index wraps after n45 + n43
  connections (AB 6, SoD 16), so later connections share a channel's monoliths with earlier ones.

**Not read:** (read in T4a: the heightmap's records and growth, the tents on water in §2.2's
placing tests, the monoliths' pairing) the 9 library helpers; what reads the zones' +0x404; what
later steps do with cells that are neither obstacles nor free; the monster object's +0x24 = 3
(H3API unknown; §2.5's writer).

**For the next parts:** +0x28 bit 26 = obstacles to place (step 13), 27 = kept free, 24 = boat
reach, +0x2C bit 0 a pending Border Guard of colour bits 1-4; the floods' costs (+0x1C halves,
+0x10, +0x20 bits 24-31) are reset and recomputed by `rmg_zone_paths_connect` (step 9 runs it
again); `rmg_guard_monster_create` 0x540B20 and the four guard tables are also used by three
functions of steps 10 and on; `rmg_zone_cost_flood` by step 7; shipyards join the road targets
(+0x14B0); the Keymaster colour +0xF5C advances with each border guard; monoliths are in +0x14C0
and +0x14D0.

### 2.4 Steps 7, 8 and 10 to 13: junctions, mines, treasures, rock, shores, obstacles (T1a2d)

Path read: `rmg_generate_map`'s calls of steps 7, 8 and 10 to 13 and everything they reach by
direct calls, with the two object methods step 10 reaches through the placed objects' vtables
(slot 2, "placed", of the Keymaster's Tent and the quest artifact, and their callees) and slot 1,
"release", of the prison. Other indirect calls: the generator's slot 1 (`rmg_generator_add_object`),
the object generators' slots (create, value, constant test), the objects' deleting destructors and
the progress window's +8. 39 functions named here (38 read, 1 inferred), 4 of §2.1 used
(`rmg_map_init`, `rmg_map_cell_reset`, `rmg_map_place_object`, `rmg_map_destroy`), one table
added and the two limits by type settled; one new record, `rmg_treasure_pile` (`structs.toml`).
Earlier parts' functions used: `rmg_zone_cost_flood`, `rmg_guard_monster_create` 0x540B20,
`rmg_prototype_draw` 0x546040, `rmg_map_can_place` 0x531CF0, `rmg_map_footprint_blocked`
0x5318B0, `rmg_map_outline_clear` 0x531170, `rmg_prototype_outline` 0x532C80, `rmg_object_init`
0x5330E0, `rmg_map_item`, `point3_init` and the library vectors.

| Function | Called from | What it does | Conf. |
|---|---|---|---|
| 0x5446A0 `rmg_junction_zone_paths` (was `rmg_phase_07_junctions`) | step 7 | a junction zone's empty cells made obstacles, its passage points joined (below) | read |
| 0x5443A0 `rmg_junction_path_draw` | 0x5446A0 | a jagged free path between two cells | read |
| 0x545F60 `rmg_mines_place` (was `rmg_phase_08_mines`) | step 8 | per zone the least mines, then the mines by density | read |
| 0x545990 `rmg_mine_place`, 0x545580 `rmg_mine_site_place` | 0x545F60, 0x545E60; 0x545990 | a mine with its guard and resources; its position (below) | read |
| 0x545E00 `rmg_guard_value` | 0x545990 | the guard value of a value at a strength index (§2.3's formula) | read |
| 0x545E60 `rmg_zone_mines_by_density` | 0x545F60 | mines by density, spaced as step 5's towns | read |
| 0x547360 `rmg_zone_treasures_place` (was `rmg_phase_10`) | step 10, per zone | the treasure bands' piles (below) | read |
| 0x535040 `rmg_pile_reset`, 0x531140 `rmg_map_cells_reset`, 0x546990 `rmg_map_cell_at`, 0x546940 `rmg_map_cell_set_land` | 0x547360, 0x5466E0 and the hooks | a pile emptied; every cell reset; a first-level cell; a cell's land | read |
| 0x5466E0 `rmg_pile_build` | 0x547360 | a band's pile: target, fill, guard, outline | read |
| 0x546520 `rmg_pile_fill` | 0x5466E0, 0x54B8C0 | objects up to a target value | read |
| 0x546190 `rmg_object_choose` | 0x546520, 0x5338E0, 0x54B490 | an object by value and density (below) | read |
| 0x535970 `rmg_pile_object_add`, 0x5355E0 `rmg_pile_site_fits` | 0x546520; 0x535970, 0x535110 | an object beside the pile's entrances; the fit on the pile | read |
| 0x535110 `rmg_pile_guard_place` | 0x5466E0, 0x54B8C0 | the guard before the pile's entrances | read |
| 0x535DF0 `rmg_pile_bounds_compute`, 0x535EE0 `rmg_pile_outline_trace` | the pile functions | the pile's box; the ring of free cells around its objects | read |
| 0x54C6F0 `point_vector_erase`, 0x54C120 `point_vector_clear` | 0x5466E0, 0x547360 | the library's 8-byte vector code | read |
| 0x5470D0 `rmg_pile_place_in_zone`, 0x546C70 `rmg_pile_fits_at`, 0x5469B0 `rmg_pile_commit` | 0x547360, 0x54B300 | the pile's offset in the zone; its fit there; the copy onto the map and the placed hooks | read |
| 0x5338E0 `rmg_keymaster_object_placed`, 0x533A50 `rmg_quest_artifact_object_placed` | slot 2 of `rmg_keymaster_object_vtable` 0x640AE4 and `rmg_quest_artifact_object_vtable` 0x640AF4 (from 0x5469B0) | a placed tent's Border Guard; a placed quest artifact's seer hut | read |
| 0x533C70 `rmg_prison_object_release` | slot 1 of `rmg_prison_object_vtable` 0x640B14 (the steps' cleanups) | the prison's hero may be drawn again | read |
| 0x54B8C0 `rmg_keymaster_guard_pile_place`, 0x54B490 `rmg_seer_hut_place` | 0x5338E0; 0x533A50 | a pile of 1.5 × the tent's value behind its Border Guard; the quest drawn and the hut, each in a near zone | read |
| 0x54B300 `rmg_pile_place_in_near_zone`, 0x54B180 `rmg_zone_hop_distances` | 0x54B8C0, 0x54B490; 0x54B300 | a pile tried in the zones by connection hops | read |
| 0x54BC50 `rmg_generator_remove_object` | 0x5338E0, 0x54B490 | an object taken off the map (add-object's inverse) | read |
| 0x5439E0 `rmg_underground_rock_paint` (was `rmg_phase_11_levels`) | step 11, with two levels | level 1's leftover cells made rock (below) | read |
| 0x531310 `rmg_map_shores_mark` (was `rmg_phase_12`) | step 12 | every land cell (not rock) beside water gets +0x28 bit 21 (H3API's shore) | read |
| 0x537970 `rmg_obstacles_fill` (was `rmg_phase_13`) | step 13 | obstacles over the cells of bit 26, then every leftover cell made free | read |
| 0x5373A0 `rmg_obstacle_fill_from`, 0x536BC0 `rmg_obstacle_score`, 0x532E40 `rmg_prototype_depth_grid` | 0x537970 down | obstacles grown from a cell; a placement's score; a prototype's depth grid | read; read; inferred |
| 0x6408EC `rmg_obstacle_types` | 0x5373A0 | the 45 obstacle types: 114-123, 125-138, 147-161, 206-211 | read |

**Step 7, junctions** (per zone of type 3 whose terrain is not water). Every cell of the zone in its
box, not water, gets its own cost reset (32000, previous cell -1) and, when empty and without a
pending guard, becomes an obstacle (+0x28 bit 26): **a junction zone starts full**. Then a cost
flood (`rmg_zone_cost_flood`, §2.3) from its first passage point (+0x404, the cells step 6's land
passages left in it), and per later point whose cost is 1 to 30000: back along the previous cells
to the first cell of cost 0 (a cell already joined), a jagged path from the point to that cell
(`rmg_junction_path_draw`: step 6's lane splitting, the midpoint moved by r × (−dy, dx) / L with
r = `rand() %` min(L, the zone's size) − min / 2; each step's cell made free and its 3 × 3 in the
zone cleared of bit 26), then a flood from the point. Points of cost 0 or unreached are skipped.

**Step 8, mines.** Per zone generator in order, per resource 0 to 6 (wood, mercury, ore, sulfur,
crystal, gems, gold): its least mines (zone +0x4C), each by `rmg_mine_place` with spacing 0, until
the resource's first failure; the first wood and the first ore of a start zone (type 0 or 1) with a
town are placed **near the town**. Then `rmg_zone_mines_by_density`: the resources of positive density
(+0x68), S and P their sum and product, spacing r = trunc(sqrt(82944 / S)), step P / d, key step ×
least count; repeatedly the resource not exhausted with the least key (the first on ties) adds its
step and places one more mine at spacing r; a resource is exhausted at its first failure.
`rmg_mine_place`: the mine prototypes (type 53) of the resource allowed on the zone's terrain, else
all of that resource, one drawn; `rmg_mine_site_place` scans the zone's box (row by row) for positions
`rmg_map_can_place` allows; near the town, only entrances 4 to 200 cells from the town's entrance
(squared distance 16 to 40000), every squared distance up to 144 counted as 144, the least kept;
then a position is skipped when its object distance (§2.1's flood) is below the greatest kept so far
(first the spacing) or when its count of outline cells that are obstacles (bit 26, passable, not
rock; on or above its row; at most 5) is below the greatest kept so far; a greater count or distance
empties the list; one drawn. The cell below the entrance is made free; **its guard** is
`rmg_guard_value`(1500 wood and ore, 7000 gold, 3500 the others; index s = the zone's monster strength
+ the map's (+0x10BC) − 3, clamped 0 to 5), none when the zone's strength is 0 (n) or the value under
2000, by `rmg_guard_monster_create` on the cell below the entrance when it is empty. Then a resource
prototype of the same resource (type 79) and, per cell of the row below the footprint from x − width
to x + 1, while at most 2 are placed, `rand() & 1`: on 0 a resource where `rmg_map_can_place` allows
(at most 3). After all zones: progress +3900; then the towns are counted (§1.5) and step 9 runs.

**Step 10, treasures** (per zone generator, water zones included; progress 6900 / zones each). The
zone's three bands (low, high, density: columns 67-75) count when high ≥ 100 and density > 0;
spacing r = trunc(sqrt(800 / S)) (1600 for a water zone; S the densities' sum, the division
integer); each band's step P / d, keys from 0. Repeatedly the band not exhausted with the least key
(first on ties) adds its step and tries: **three times `rmg_pile_build` without the density filter,
then three times with it**; a built pile goes to `rmg_pile_place_in_zone` (spacing r); a pile that
cannot be placed is released (its objects' slot 1, then deleted) and the pile cleared. A band is
exhausted when one pick's six tries all fail; the step ends when every band is.

- **A pile** (`rmg_treasure_pile`: a 16 × 16 one-level map, its objects, its outline): target t =
  low + `rand() %` (high − low) (high when high ≤ low); `rmg_pile_fill`: a first object worth t/4 to t
  (any type; three tries) at the pile's centre, then while the total is below t and the rest is 1500
  or more or at least half the total, an object worth rest/4 to 5 × rest/4 (not a type whose object
  settings +0 is set and +2 clear) beside the others: its entrance on a neighbour of an earlier
  object's entrance (all 8 when that type's settings +1 is set, else SE, S, SW), kept off the pile's
  edges and allowed by `rmg_pile_site_fits`, one drawn; three null choices or three failed additions
  end the fill.
- **Choosing an object** (`rmg_object_choose`). Over the object generators (+0x10F0, §2.1's order),
  a generator is a candidate when its type's count on the map (+0x1110) is below
  `rmg_type_limit_map` 0x69CE4C (**per map**: 32000 unless listed; 200 for types 6 and 26, 64 for 8, 48
  for 57 and 83, 32 for 25 others) and its count in the zone (zone generator +0x44, the objects whose
  entrance is in it) below `rmg_type_limit_zone` 0x69D1F4 (**per zone**: 1 for 23 types, 3 for the
  Witch Hut 113, else 32000), its value
  (slot 1; −1 refuses) lies in [min, max], `rmg_prototype_draw` finds a prototype of its type and
  subtype for the zone's terrain (**a draw per candidate**), the footprint is free when a point is
  given, and, with the density filter, its value per occupied footprint cell (cells not passable or
  enterable) is at least 3/4 of the best so far (one above 4/3 of the best empties the list and
  becomes the best). The choice is `rand() %` the candidates' densities summed (+0x10), by weight;
  the object is created by the generator's slot 0 with the drawn prototype and its value read again.
  The constant test (§2.1) is applied only when the caller asks: the replacements below, which so
  refuse seer huts and Keymaster's Tents. Nowhere else are the limits by type read.
- **The guard** (`rmg_pile_build`): none when the zone's strength is 0; else the guard value of the
  pile's total (index as for mines) and, when 2000 or more, `rmg_guard_monster_create`; no monster:
  the pile stays unguarded. `rmg_pile_guard_place` marks the empty cells before each entrance (bit 26;
  their 3 × 3 loses bit 27), keeps the outline cells among them where the monster fits (one of its
  entrance's neighbours open), draws one, places the monster and frees its entrance's 8 neighbours (not
  those of bit 26 for a Border Guard) and one or three cells beyond each when unmarked; none: the pile
  is refused.
- **Placing it** (`rmg_pile_place_in_zone`): per offset that keeps the pile's box in the zone's (rows
  outer), the zone cell under the pile's box centre must be in the zone with an object distance of at
  least r, and `rmg_pile_fits_at` true: every footprint free on the map, the guard's 3 × 3 holding no
  monster, the last object's entrance (the guard's) beside a free outline cell over a map cell kept
  free (bit 27; water exactly in a water zone), `rmg_map_outline_clear` over the outline, no pile cell
  that is not free over a map object. **The farthest such offset from the objects** (ties: one drawn).
  `rmg_pile_commit` places the objects through slot 1, makes map obstacles under the pile's cells of
  bit 26 (the marked entrance cells the guard does not cover: **the guard is the only way in**), takes
  bit 27 off the map cells under the pile's non-free empty cells, then calls each object's slot 2.
- **The placed hooks.** A Keymaster's Tent (`rmg_keymaster_object_placed`): its colour's Border Guard
  prototype, the colour marked (+0x1100) and +0xF5C moved to the next unmarked colour, a new pile filled
  to 1.5 × the tent's value in the tent's zone and guarded by that Border Guard, placed by
  `rmg_pile_place_in_near_zone`; on failure the colour is unmarked, the tent removed
  (`rmg_generator_remove_object`) and an object worth v to 1.5 v (not a seer hut or tent) put at its
  position. A quest artifact (`rmg_quest_artifact_object_placed` → `rmg_seer_hut_place`): the treasure
  artifacts (class 2) among 0-143 not disabled and not used, under 20 of them blocking further seer
  huts (+0x10B4), one drawn as the quest, the artifact object given that artifact's prototype, the hut
  alone in a pile placed by `rmg_pile_place_in_near_zone`; success marks the artifact used and moves
  +0xF58 to the next seer hut prototype; failure removes the artifact and puts an object of the hut
  generator's value v to 1.5 v at its position, the hut deleted. `rmg_pile_place_in_near_zone`: the
  connection hops from the zone (`rmg_zone_hop_distances`), keys 10 × hops + `rand() % 10`, **a
  neighbour (one hop) 1000 + `rand() % 10`**, the zones other than this one, not junctions, not
  water, keyed at most 2000, tried in key order (stable): neighbours only after every farther reachable
  zone. The new pile is committed in turn, so its own tents and quest artifacts chain the same way
  (a tent of the next colour can sit behind a Border Guard).

**Step 11, rock underground** (two levels). Through a view of level 1 the terrain painter makes rock
every cell that is passable, empty, not rock and not kept free (the obstacle cells and the cells
marked neither), then gives each level-1 zone's rock cells that are kept free or hold an object its
terrain back; progress +1200 twice. Step 13 then skips them (rock): **underground obstacles are rock**.

**Step 13, obstacles.** n = the cells with bit 26; per such cell (level, row, column order) that is
passable and not rock, `rmg_obstacle_fill_from`; then every cell neither kept free nor rock, passable
and without a pending guard is made free. A fill: a stack of cells from the given one; per cell popped,
per type of `rmg_obstacle_types` in order, per prototype of it with a `rand_trn.txt` row (§2.1) whose
modifier for the cell's land is above −5000 (types 165 and on refused in RoE; the test for 222 and on
below SoD refuses none), per position of its footprint over the cell with a blocking cell there:
`rmg_obstacle_score`, kept when positive. The score: −5000 when a blocking cell of its visible mask
(+0x3C) lies on a land its template does not allow or on a kept-free cell; else the sum of its row's
modifiers for the lands under those cells (returned when below −1000; −1 when none is positive); plus,
per object on a blocked cell of the footprint widened by one, its row's Overlap value for that object's
row when a blocking cell lies on it (−5000 when that object has no row), else its Adjacent value when
beside it; −5000 when their depths on shared visible cells compare both ways, the two interleaving
(their prototypes' depth grids, `rmg_prototype_depth_grid`). One candidate drawn by weight `rand() %` the scores' sum; a plain object
placed through slot 1; the cells of bit 26 in its footprint widened by one lose the bit and are pushed.
Progress 276300 / n per cell (a cell popped counts again).

**Draw sites** (13 here, and those of §2.1 and §2.3 that these steps reach; in the order they run):

| Site | Function | Drawn | Per | Condition |
|---|---|---|---|---|
| 0x5444B4 `rmg_junction_jitter_draw` | `rmg_junction_path_draw` | `rand() %` min(L, zone size) (signed) | segment split | L > 1 |
| 0x545AA8 `rmg_mine_prototype_draw` | `rmg_mine_place` | `rand() %` mine prototypes (unsigned) | mine tried | a prototype |
| 0x54591A `rmg_mine_site_draw` | `rmg_mine_site_place` | `rand() %` positions kept (unsigned) | mine | a position |
| 0x540C61, 0x540CCE, 0x540CD8 (§2.3) | `rmg_guard_monster_create` | the guard's creature, count + and − | mine | guard value ≥ 2000, the cell below the entrance empty |
| 0x546154 (§2.1) | `rmg_prototype_draw` | the resources' prototype | mine | always after the guard |
| 0x545D0C `rmg_mine_resource_cell_draw` | `rmg_mine_place` | `rand() & 1` (signed form) | cell of the row below | a resource prototype, at most 2 placed |
| 0x546154 (§2.1) | `rmg_prototype_draw` from `rmg_path_carve` | step 9: a Border Guard prototype (§2.3) | pending guard crossed | a path through a marked cell |
| 0x546771 `rmg_pile_target_draw` | `rmg_pile_build` | `rand() %` (high − low) (signed) | pile built | high > low |
| 0x546154 (§2.1) | `rmg_prototype_draw` from `rmg_object_choose` | a prototype of the generator | candidate generator | its count, limits and value pass |
| 0x546489 `rmg_object_weight_draw` | `rmg_object_choose` | `rand() %` densities summed (signed) | object chosen | a candidate |
| 0x535B31 `rmg_pile_object_cell_draw` | `rmg_pile_object_add` | `rand() %` positions (unsigned) | object after the first | a position |
| 0x540C61, 0x540CCE, 0x540CD8 (§2.3) | `rmg_guard_monster_create` | the pile's guard | pile | guard value ≥ 2000, zone strength not 0 |
| 0x5353B9 `rmg_pile_guard_cell_draw` | `rmg_pile_guard_place` | `rand() %` guard cells (unsigned) | pile guarded | a cell |
| 0x5472F0 `rmg_pile_offset_draw` | `rmg_pile_place_in_zone` | `rand() %` farthest offsets (unsigned) | pile placed | an offset |
| 0x54B511 `rmg_quest_artifact_draw` | `rmg_seer_hut_place` | `rand() %` free treasure artifacts (signed) | quest artifact committed | one left |
| 0x54B366 `rmg_near_zone_neighbour_draw`, 0x54B382 `rmg_near_zone_order_draw` | `rmg_pile_place_in_near_zone` | `rand() % 10` (signed) | zone generator (every one, in order) | a tent's or seer hut's pile |
| 0x5376AA `rmg_obstacle_weight_draw` | `rmg_obstacle_fill_from` | `rand() %` scores summed (signed) | cell popped | a candidate |

Within a pick of step 10: the target, then per object the candidates' prototype draws and the weight
draw (and for later objects the cell), then the guard's three draws and its cell, then the offset,
then per committed object in pile order its hook's draws: a tent's pile (fill, guard cell, the
near-zone keys, the offset in the first zone that fits) or its replacement (prototype draws, weight);
a quest artifact's quest, the near-zone keys, the offset, or the replacement. A failed try's draws
are not undone.

**Float sites** (x87 only, every conversion `crt_ftol`, truncating):

| Function | Computed | Constants |
|---|---|---|
| `rmg_zone_mines_by_density` (2) | trunc(sqrt(f64(82944 / S))), the integer quotient converted by `fild` | none (82944 integer) |
| `rmg_zone_treasures_place` (4) | trunc(sqrt(f64(800 / S))), 1600 / S for water, the same way | none |
| `rmg_junction_path_draw` | the segment length by `vector_length_int` 0x5FCEB0 (§2.2's table) | none |

**Clocks, retries, limits, failures:** no clock. **Step 10 retries:** each pick tries six times
(three without, three with the density filter) and a band ends at a pick that fails all six; the
first object of a pile gets three tries, each later one three null choices and three placements; the
step's loop has no cap: it ends when the zone stops taking piles. Elsewhere one pass: step 7 per
passage point, step 8 until each resource's first failure, step 13 until its stack is empty.
**Give-ups:** a mine or pile with no position is dropped silently; a pile whose guard finds no
monster is placed unguarded; a tent or quest artifact whose pile cannot be placed is swapped for an
object of about its value, or left out when none fits; a cell of bit 26 without a candidate obstacle is
made free. Faults reachable only with other data: `rand() % 0` in `rmg_junction_path_draw` for a zone
of size 0, a division by zero in `rmg_object_choose` for a template with no blocking or enterable cell
(density filter), `rmg_seer_hut_place` reading past the artifact prototypes when none has the quest's
id. Progress: +3900 (mines), 6900 / zones per zone (treasures), +1200 twice (rock), 276300 / n per
obstacle cell.

**Outside callees** (24): mapped already 11 (`crt_rand`, `crt_sqrt`, `crt_ftol`, `vector_length_int`,
the terrain painter's four, `pointer_vector_insert` 0x54D120, `entry12_vector_insert` 0x4347A0,
`empty_ret_8` 0x404140); the C runtime 3 (operator new, free, the array destructor); the C++ library
10 by their callers (a pointer vector's insert, erase, clear, construction and destruction, an 8-byte
point vector's insert, a 12-byte element copy, the 48-bit and 10-bit bitset tests and the bitset's
range error; not read). UI: the progress window's +8 only. No file. Tables read in place:
`object_settings_pointer` 0x660428 (bytes +0, +1, +2), `artifact_setup_pointer` 0x660B68 (+0xC
class, +0x1C disabled), `rmg_neighbours` 0x69CDC0, the guard tables (§2.3).

**Quirk candidates (as read; classified in `docs/rules/quirks-rmg.md`, T1c2):**

- `rmg_mine_site_place` and `rmg_object_choose`'s density filter keep a running best and test each
  candidate against it, so the result depends on the scan order: a position with more obstacles
  around it but a smaller distance than one kept earlier is skipped; a ratio above 4/3 of the best
  drops earlier ones that would pass 3/4 of the new best.
- `rmg_object_choose` draws a prototype for every candidate generator before the weighted choice (and
  before the density filter): the stream advances by the number of candidates, not of objects.
- A treasure pile whose value calls for a guard but finds no monster in range is placed unguarded.
- Mines and treasures index the guard tables with the zone's strength plus the map's (− 3, clamped),
  connections (§2.3) with the map's alone; a zone of strength none (n) gets no mine or treasure guard.
- Seer hut quests are treasure-class artifacts only, counted over 0-143 for every version; under 20
  left, no seer hut generator is valued again.
- A Border Guard's pile and a seer hut go to the nearest zone by connections but **never a neighbour
  while a farther zone fits**; the pile can chain (its tents and quest artifacts run the same hooks).
- A tent or quest artifact whose partner cannot be placed is taken off the map but not deleted: its
  prototype's use count stays raised, so its template may be written (§1.6) with no object using it.
- Underground, every cell neither free nor holding an object becomes rock, including those step 13
  would fill with obstacles.
- `rmg_obstacle_fill_from`'s SoD test (types 222 and on) refuses none of the 45 types.
- `rmg_pile_place_in_zone` lets the pile's box end one cell past the zone's (its offsets run to the
  zone's max + 1 − the pile's); the fit test still applies.

**Not read:** the increments of `rmg_prototype_depth_grid`'s values (only that they order two
obstacles); what the object settings bytes +0, +1, +2 mean beyond the tests above; the library
helpers; the objects' write methods (§2.5); whether `rmg_map_outline_clear` (§2.2, read in part)
refuses what the pile's ring needs.

**For the next parts:** step 14's roads start after every object is placed; the cells now carry bit 21
(shore; bit 23 is set only on the piles' own maps); after step 13 every passable cell that is not rock
and has no pending guard is kept free (bit 27). The placed objects'
slots: 0 delete, 1 release (the prison's frees its hero), 2 placed hook (tent, quest artifact; true
otherwise), 3 write. The zone generators' +0x40 holds the last near-zone keys; the generator's +0xF58,
+0x10B4, +0x1024 and +0x1100 are final after step 10.

### 2.5 Steps 14 and 15 and the writer (T1a2e)

Path read: `rmg_generate_map`'s steps 14 and 15 and what they reach by direct calls, with the
road and river views the game's painters call back through; `rmg_write_map` 0x54ABF0 →
`rmg_write_header` 0x549CB0 (whole), `rmg_teams_draw` 0x54AB40, `rmg_write_cell` 0x532890,
`rmg_write_template` 0x54AE30, and slots 3 (write) and 0 (delete) of the 15 object vtables; then
every function start of the range still without an entry. 66 functions named or re-read here (all
read; 61 new), 4 draw sites, 10 tables; two steps renamed. **The range is closed:** of its 255
function starts, 254 have a function entry (237 read, 17 inferred) and one, 0x53D640
`rmg_zone_flood_fill_tail`, a site (§2.2: a continuation the tool takes for a start).

| Function | Called from | What it does | Conf. |
|---|---|---|---|
| 0x548290 `rmg_roads_lay` (was `rmg_phase_14_roads`) | step 14 | the road type drawn; every pair of road targets joined (below) | read |
| 0x547880 `rmg_road_cost_flood`, 0x548040 `rmg_road_trace` | 0x548290 | the flood over the map; a road laid back along it | read |
| 0x549870 `rmg_rivers_lay` (was `rmg_phase_15_shipyards`) | step 15 | the marks, then two rivers per Water Wheel (below) | read |
| 0x5497A0 `rmg_river_sources_mark`; 0x548C70 `rmg_river_mouths_mark` → 0x548A40 `rmg_river_delta_site_mark` | 0x549870 | bit 29 on the sources; bit 30 on the map's border and the delta sites | read |
| 0x548500 `rmg_river_to_source`, 0x548DF0 `rmg_river_to_mouth`, 0x549790 `rmg_river_delta_offsets_destroy` | 0x549870; an exit function | the upper and the lower course; the empty destructor of `rmg_river_delta_offsets` 0x69CE28 | read |
| 0x532320 to 0x5324B0 `rmg_road_view_*`, 0x5324E0 to 0x532860 `rmg_river_view_*`, 0x532790 `rmg_tile_view_size`, 0x537910, 0x537940 (the bases' deletes) | `rmg_road_view_vtable` 0x640A04, `rmg_river_view_vtable` 0x640A3C (bases 0x640A20, 0x640A58): the game's road and river painters | a level of the map seen as roads or rivers: set, set type, size, get, get type, get land | read |
| 0x549CB0 `rmg_write_header` | 0x54ABF0 | the header (below) | read |
| 0x532890 `rmg_write_cell`, 0x54AE30 `rmg_write_template` | 0x54ABF0 | a cell's 7 bytes; a template in the reader's layout | read |
| 0x533170 `rmg_object_write`, 0x5331F0, 0x5332F0, 0x533460, 0x533500, 0x5335C0, 0x5336F0, 0x533A90, 0x533C80, 0x533E70, 0x533F40, 0x533FF0, 0x5340C0 (`rmg_<kind>_object_write`) | slot 3 of the object vtables (0x54ABF0) | an object's header and body (below) | read |
| 0x533120 `rmg_object_delete`, 0x533590 `rmg_object_derived_delete`, 0x533680, 0x5339C0 (Pandora's, quest artifact's); 0x5338D0 `rmg_object_destroy`, 0x5336B0, 0x5339F0 | slot 0; the steps' cleanups and the destructor | the prototype's use count less 1; a Pandora's spells freed; **a quest artifact deletes its seer hut** | read |
| 0x5357E0, 0x535840, 0x5358A0, 0x535910, 0x535BB0, 0x535C10, 0x535C70, 0x535CD0, 0x535D30, 0x535D90 `rmg_static_terrain_set_init_1` to `_10` | the C runtime's initializer table (not reached otherwise) | a terrain set holding dirt alone into a global no code reads | read |
| 0x5361E0 `rmg_prototype_vector_init` | the array constructor in `rmg_generator_base_init` | an empty prototype vector | read |
| 0x5361A0 `rmg_map_base_destroy`, 0x5329A0 `rmg_zone_destroy`, 0x5477C0 `rmg_pile_destroy`, 0x54C170 `rmg_obstacle_vector_free`, 0x54C6A0 `rmg_list_clear` | unwind funclets only (an exception inside a step) | destructors | read |

**Step 14, roads.** One draw, `rand() % 3 + 1`: the type of every road (1 dirt, 2 gravel, 3
cobblestone). For each road target i but the last (+0x14B0 in order: the towns' entrances of steps
4 and 5, then the shipyards' of step 6), every cell's cost is reset (32000, previous −1) and
`rmg_road_cost_flood` runs from it **over the whole map**: least cost first (a sorted vector, ties
first in, first out); a step costs 2 from a road cell onto a road cell, else 20, ×3 diagonally;
off the map, water, rock and impassable cells (+0x28 bit 25 clear: the obstacles) are refused, and
so are objects whose type's settings +0 is set and +2 clear; an object whose settings +1 and +2
are clear is left only toward its five lower and side neighbours and entered only not moving SE,
S or SW; a one-way monolith also reaches every one-way monolith of its subtype (+0x14C0, exits
and entrances alike) at +50, a two-way one those of +0x14D0, a subterranean gate the same x, y on
the other level at +1. Per later target j whose cost is at most 30000, `rmg_road_trace` walks back
from j: the stretch at j's end already of this type is skipped, then every cell to the origin goes
to the game's road painter (outside the range), a new stroke after each diagonal step and level
change; when it painted and j is not the last target, the flood from i runs again, so later roads
follow earlier ones (cost 2). An unreachable target gets no road. Progress +1000 per re-flood and
per i.

**Step 15, rivers** (T1a1's shipyards are Water Wheels, type 109). Bit 29 (a river or a source) on
one cell of every lake (126), mountain (134) and Gem Pond (53, subtype 5): its entrance, else the
cell half its footprint up and left of its position. Bit 30 (a river's end) on the map's border
cells and on the **delta sites**: for each water cell and orthogonal direction d, when the three
cells across it are water, the three beyond them land and the next four cells in d land, all
without objects, the fourth (4 cells inland) gets bit 30 and d in bits 8-11. Then per Water Wheel
in placing order `rmg_river_to_source` from its entrance and `rmg_river_to_mouth` from three
cells: two left of its entrance, the one above it and the one right of that. Both are least-cost floods over E, S, W, N that refuse
water, rock and the other snowness (on snow an icy river, type 2, else a clear one, type 1: a
river never passes between snow and other land) and **test no object, obstacle or passability**;
a step costs **(rand() & 31) + 1, a draw per neighbour examined**, +30 into a road cell; each stops
at the first neighbour it improves that carries its target bit (29 upward, 30 downward) and paints
the river from there back to its start through the river view. The lower course also refuses cells
beside an earlier river (bit 31: the river view marks the 3 × 3 of every river cell) and enters a
delta site only along its direction; at a delta site it places a **river delta** (type 143: the
n-th of its prototypes allowed on that land, n by direction from 0x6409A0 for a clear river,
0x6409B0 for an icy one; none: no delta) through slot 1 at the site + `rmg_river_delta_offsets`
0x69CE28, and the river gets one cell more toward the sea. Its cells get bit 30, so a later wheel's
lower course can end on it; the river view clears bit 30 in the 5 × 5 around each new river cell
that holds no river. A flood that reaches nothing paints nothing. Progress +1000 per mark and per
wheel.

**The file** (settles §1.6; `h3m.md`'s reader table; ✓ = as the reader expects):

1. **Header, in write order:** format (0x0E, 0x15, 0x1C by the version), 1, the width (one side),
   levels > 1 ✓; the name "Random Map", the description, difficulty 1, AB's level limit 0 ✓. Per
   colour: the bools from its zones that hold a town (a human start: human and computer; a
   computer start: computer only), and human-start colours beyond the players that are not
   human-flagged turned computer-only, pink first; AI tactic 0, SoD's town-choice byte 0, the
   allowed towns = those zones' town types (u8 RoE, u16 from AB), random town 0, a main town with
   AB's generate-hero 1 and town type 0xFF at the zone's first town's **entrance** (zone generator
   +0x30), no random hero, hero type 0xFF, AB's byte 0 and no heroes; a colour without such a zone
   fills the reader's 6, 12 or 13 unused bytes (zeros and the hero type 0xFF) ✓. Standard victory and loss ✓; the teams ✓
   (below); the allowed heroes (not banned at +0xF88: 16 bytes of 128 in RoE, 20 of 156) ✓; AB's
   u32 0 placeholders, SoD's u8 0 disposed heroes ✓; 31 zero bytes ✓; **the banned artifacts**:
   those whose setup record +0x14 is not −1 (H3API's combination index; 129-140 in `Untitled.h3m`)
   and 127 and 128: **AB 17 bytes of 129 bits, SoD 18 of 144** ✓ (T1a1's 17-byte block was AB's
   branch); SoD's 9 and 4 zero bytes (no spell or skill banned) ✓, then 156 zero bytes. The
   reader's order is the rumours' count, then the 156 hero bytes: `rmg_write_map`'s u32 0 after
   the header is the rumours' count, the same 160 zero bytes in the other order.
2. **Teams:** a computers' team count of 0 becomes the computers, a team count of 0 the players,
   with no computers at least 2; teams ≥ players and computers' teams ≥ computers: no teams (count
   0); else both clamped to 1 .. their colours (recounted from the bools above), the players spread
   from team 0, the computers from the players' team count on, one draw per flagged colour.
3. **The rest:** every cell (7 bytes: land, picture, river, picture, road, picture, flags = the
   three mirror pairs and the shore 0x40) ✓; the templates (count + 2, the first prototypes of
   groups 71 and 124 (random monster, hole), then every prototype in use by its type's object settings group, so
   `Untitled.h3m`'s classes are not sorted; name, the two 6-byte masks repacked, two u16 terrain
   masks from +0x14 and +0x18, class, subclass, the bytes +0x24 and +0x28, 16 zero) ✓; the
   objects: their count, in placing order **first those whose type's object settings +0xC is set,
   then the rest**, each through its slot 3 with the version: x, y, z, the template index, 5 zero
   bytes, the body ✓: monster (AB's identifier, count, disposition +0x24, no treasure, 0, 0, 2
   zero), town (identifier, owner, no name or garrison, formation 0, no building list, has fort,
   AB's 9 + 9 zero spell bytes, no events, SoD's 0xFF, 3 zero), dwelling and shipyard (owner 0xFF),
   random and quest artifact (no message), resource (no message, amount 0, 4 zero), Pandora's Box
   (its rewards: experience, resources, spells, one creature), seer hut (RoE the artifact; AB a
   quest of one artifact, no deadline, empty texts; the reward experience, creature or resource),
   prison (identifier, hero, experience, the rest default), Scholar (bonus 0xFF), shrine (spell
   0xFF), scroll (its spell), Witch Hut (AB's skills 0xEFDF), the plain kind (a Keymaster's Tent
   among others) the header alone; then the timed events' u32 0. **No 124 padding bytes are
   written.**
4. **The description:** "Map created by the Random Map Generator.  Template was …" with, in order,
   the template's name, the seed (+4), the width, the levels, "humans" = the dialog's players
   (humans and computers, +0xF48), the computers, water (None, normal, islands), the monster
   strength 1 to 5; then the version (original, first expansion, second expansion map); then per
   colour in order ", <colour> is human" when its human flag is set and ", <colour> town choice is
   <town>" when it has a town choice, **the town's name taken by the colour's index** (`rmg_town_names`
   0x68272C[colour]: red always reads castle). **Nothing in the file depends on the time or the
   machine but the seed** (no date or name; `Untitled.h3m`'s gzip header has time 0, OS 0x0B).
5. **Draws in the writer:** only the teams' (0x54ABA6), after every draw of the generation.

**`Untitled.h3m` against this reading** (observation): it matches in all but three points: each
main town's type byte holds that town's type where the code writes 0xFF; the file ends with 124
zero bytes after the events' count, which `rmg_write_map` does not write (its last objects are
bodiless 12-byte ones, then 128 zero bytes); its description names no colour as human, which a run
with a human player gives as read. The user plays with the HD mod (`docs/epics/E17-design.md`): a
runtime patch of the writer, and possibly of the generation, is the likely cause. Its 11 Witch Huts
carry 0xEFDF as read.

**Draw sites** (4):

| Site | Function | Drawn | Per | Condition |
|---|---|---|---|---|
| 0x54829B `rmg_road_type_draw` | `rmg_roads_lay` | `rand() % 3 + 1` | run | always |
| 0x54876F `rmg_river_source_cost_draw` | `rmg_river_to_source` | `(rand() & 31) + 1` | neighbour examined, per cell popped (a cell can pop more than once) | on the map, not water or rock, same snowness |
| 0x549153 `rmg_river_mouth_cost_draw` | `rmg_river_to_mouth` | the same | the same | also not beside a river |
| 0x54ABA6 `rmg_team_draw` | `rmg_teams_draw` | `rand() %` teams with room left (signed) | flagged colour, players then computers | the map has teams |

**Float sites:** none (no x87 or SSE instruction in the 66 functions).

**Clocks, retries, limits, failures:** no clock (the seed is only printed). No retry and no cap: the
floods run until their queue is empty; step 14 floods once per target and once more per road laid
(one road per pair of targets at most), step 15 twice per wheel. Give-ups, all silent: a target of cost above
30000 gets no road; a wheel whose flood reaches no source or end gets no course; a delta without its
n-th prototype is not placed. `rmg_river_delta_site_mark` bounds x by `x > width` (x = width
accepted): it reads the next row's first cell, and at the map's last cell **one cell past the
array**, whose bytes decide whether that delta site is marked (memory-dependent).

**Outside callees** (32): mapped already 6 (`empty_ret_8`, `vector_insert_u32`,
`entry12_vector_insert`, `c_sprintf`, two H3API guesses: string init, free); the game's road and
river painters 5 (a road and a river painter's start and end, a stroke's next cell, shared); a
template's name 1; the C runtime 4 more (operator new, the exception throw, atexit, the array
destructor); the C++ library 16 (strings, vectors, bitsets of 10, 48, 128, 129,
144 and 156 bits and their range errors, exception objects). I/O: the stream's write (+8) only, no
file opened here; UI: the progress window's +8.

**Quirk candidates (as read; classified in `docs/rules/quirks-rmg.md`, T1c2):**

- The description's town choice prints the town of the colour's index, not the choice; "humans"
  counts the computers too.
- Rivers test no objects or obstacles: a river can run under them.
- The river floods draw per neighbour examined, a cell popped again drawing again: the stream
  depends on the queue's details, not only on the map.
- Generated Witch Huts offer skills 0-4, 6-11 and 13-15 only (never Navigation, Necromancy or any
  of 16-27; `Untitled.h3m` agrees).
- Every map bans artifacts 127 and 128 besides the combination artifacts.
- Roads treat one-way monoliths both ways and jump to the far monolith's position.
- `rmg_river_delta_site_mark`'s x bound (above).
- The main town's position is the town's entrance (`Untitled.h3m`'s towns lie two cells right of
  it); T1c compares with the reader's convention.

**Not read:** the game's road and river painters (which picture they choose; they write through the
views' slots 1 and 2); which types set object settings +0xC (the first pass); where the river
view's bits 29-31 are read beyond step 15 (nowhere else in the range).

**For the next parts:** every draw site of the run is now listed (§2.6); the oracle (T1b) must
generate with the plain executable or record which patch ran, since `Untitled.h3m` differs from the
code read; a generated file names its seed, template, choices and version, and its 124 trailing
bytes, the town type bytes and the human list show whether the HD mod wrote it.

### 2.6 The stream's draws in order

One stream (`crt_rand`, Begin's thread), in execution order; the sites from §1.1 to §2.5.

| Phase | Sites in order | Per |
|---|---|---|
| dialog | 0x586168, 0x586183, 0x5861B1, 0x5861E4, 0x58621D | Random choice, on the game's earlier state |
| setup | `srand(time())` 0x536148; water 0x537C89 | once; water Random |
| template | 0x549975 | once |
| step 1 | per template zone 0x532A5B (its town types), 0x53BB5F; two more passes of 0x53BB5F; per zone 0x532B00 or none, 0x53BFC0 when +4 is -1 | zone, pass |
| step 2 | 0x532A5B per temporary and water zone (by uninitialised bytes); 0x53C0F2 | level; segment split |
| step 3 | 0x53CE1D | split (islands) |
| steps 4, 5 | 0x545492; 0x544DEC or 0x544E22, 0x545174 | town |
| step 6 | 0x543EB8, 0x54403E; per water zone 0x53F745, 0x53F794, 0x53EFE4, 0x53EEFF to 0x53EF43; 0x546154 (joining, again per monolith Border Guard); per connection the way (0x541B78, 0x541D6C; 0x5421BC, 0x542387; 0x54146B per passage), then its guards (0x542A8F, or 0x541673 and 0x540C61, 0x540CCE, 0x540CD8) | level, split; islet; guard crossed; connection |
| step 7 | 0x5444B4 | split |
| step 8 | 0x545AA8, 0x54591A, the guard's three, 0x546154, 0x545D0C | mine; cell |
| step 9 | 0x546154 | guard crossed |
| step 10 | 0x546771; per object 0x546154 per candidate, 0x546489, 0x535B31; the guard's three, 0x5353B9; 0x5472F0; the hooks' 0x54B511, 0x54B366, 0x54B382 and their piles' draws | pile, per pick and try |
| any object made | 0x54B136 (prison), 0x534F06 (scroll), 0x546154 (seer's artifact) | object created |
| step 13 | 0x5376AA | cell popped |
| step 14 | 0x54829B | once |
| step 15 | 0x54876F, then 0x549153 | neighbour examined, per wheel |
| writer | 0x54ABA6 | flagged colour, with teams |

**Observed (T1b2's phase oracle, `docs/lanes/rmg/ORACLE.md`):** the range's sites draw in the
table's order and phases. The stream also holds draws of callees outside the range that this
table leaves out, a third of the corpus's draws: the terrain painter's pictures
(`terrain_picture_draw` 0x5B3890 and its kin, one or two per painted cell) in steps 3, 6 and 11,
the road and river painter's (`road_river_picture_draw` 0x4F9F00) in steps 14 and 15. §2.7 reads
them; §2.7.5 gives their rows.

**The run's floats:** x87 only, every conversion `crt_ftol` (truncation): step 1's candidates
trunc(f64(r) × T[k] + f64(o)) with `rmg_layout_cos` 0x682500 and `rmg_layout_sin` 0x682600 (f64,
four decimals) at the control word's precision; step 2's points (s × T[k] doubled + x, trunc;
`fcomp` bounds); trunc(sqrt(f64(dx² + dy²))) in `rmg_zone_position_fits`, `rmg_zones_touch` and
`vector_length_int` 0x5FCEB0 (steps 1, 2, 6, 7); the spacings trunc(sqrt(f64(q))) of an integer
quotient: 82944 / S (steps 5, 8), 800 / S or 1600 / S (step 10). None elsewhere.

**Clocks, retries, limits:** the clock: `time()` in seconds, once (the seed, printed in the file).
Memory-dependent: step 2's uninitialised town-type bytes (whether 0x532A5B draws; T1b2: it drew
in 18 of 21 first maps, and a second map in one game drew it once less and differed) and step
15's read past the cells (§2.5). Fixed iterations: step 1's three passes, 32 (65 with two levels)
candidates per placement. Retries: step 10 only (six per pick, three per object). Limits: a land
connection's passages ≤ ceil(n / 40) of ≤ 100 candidates; roads to costs ≤ 30000; the limits by
type in step 10's choice. Every other give-up is silent; the run's only reported failure is no
template (code 3), an exception anywhere code 1.

### 2.7 The painters outside the range (T1c1)

Cut by the user (2026-11-24) into three parts read in one session: T1c1a (the runner: the terrain
painter's frame and its picture sets, the road and river painter; §2.7.1, §2.7.4), T1c1b (a
worker: adding cells and the work sets; §2.7.2), T1c1c (a worker: the finishing passes; §2.7.3);
§2.7.5 the draws they add to the stream and the oracle's check. About a third of a generation's
draws come from here (T1b2). The painters are the game's own code outside the generator's range, reached from steps 3, 6 and 11 (terrain) and 14 and 15 (roads and rivers) through
small interfaces: the generator's map (`rmg_map_vtable` 0x6409CC, §2.1) and its road and river
views (§2.5).

#### 2.7.1 The terrain painter's frame and the picture sets (T1c1a)

Path read: `terrain_painter_init` 0x5B7250 and its constructor, `terrain_painter_add_rect`,
`terrain_painter_finish` and `terrain_painter_flush` (their own bodies: their callees are §2.7.2
and §2.7.3), the ten picture sets with their constructors, vtables and tables. 26 functions named
or re-read (all read), 5 draw sites, 9 tables and globals. The C++ library helpers the frame calls
(strings, the exception's text, vectors' copy and fill, the sets' head) are not named.

| Function | Called from | What it does | Conf. |
|---|---|---|---|
| 0x5B7250 `terrain_painter_init` → 0x5B45F0 `terrain_painter_construct` | steps 3, 6 (`rmg_islet_draw`, `rmg_water_shores_seal`), 11 | the handle and its 0x44-byte painter (below) | read |
| 0x5B7690 `terrain_painter_add_rect` | the same | a rectangle to `terrain_painter_paint_rect` (§2.7.2) | read |
| 0x5B72F0 `terrain_painter_finish`, 0x5B7520 `terrain_painter_flush` | the handle's destructor (and unwinding); `rmg_water_shores_seal`, step 11 | the sets drained, the last pass (below) | read |
| 0x5B3780 `terrain_picture_set_init`; 0x5B3840, 0x5B3850, 0x5B3860, 0x5B3880, 0x5B3890, 0x5B38F0, 0x5B3A50 | the static initializers; `terrain_picture_set_vtable` 0x642C98 | a picture set of the first kind and its slots 0 to 5 | read |
| 0x5B3940 `rock_picture_ranges_init` (from 0x5B3A10), 0x5B3A20 `rock_picture_set_init`; 0x5B3A40, 0x5543F0 `false_ret_4`, 0x5B3A80, 0x5B3AA0, 0x5B3AE0 | `rock_picture_set_vtable` 0x642CB0 | the rock set and its slots | read |
| 0x5B3B60 to 0x5B3D60 (nine), 0x5B3DA0 `<terrain>_picture_set_static_init` | the C runtime's initializer table | the ten sets built at start-up | read |

**The painter** (0x44 bytes): +0 the map interface, +4 the terrain (0-9), +8 the *weight* (4 in
every generator call), +0xC and +0x10 the map's width and height (the interface's slot 3), +0x14
set 1 and +0x24 set 2 (`std::set` of points, §2.7.2), +0x34 a vector of one 16-bit cell word per
cell. **The words' fill is memory-dependent:** the constructor keeps the fill value in a reused
argument slot and clears only its bit 0, so every word starts with bits 1-15 equal to the high
half of the map interface's address. T1c1b found no read of those bits before a load (§2.7.2):
the fill never reaches a result.

**The handle's use.** `terrain_painter_init(map, terrain, 4)`, then rectangles
(`terrain_painter_add_rect`), then the handle's destructor (`terrain_painter_finish`), which runs
until both sets are empty: set 1's cells one by one, smallest (y, x) first, through 0x5B5440
(§2.7.3); then set 2's, each erased and, when pinched west-east or north-south or (for a set whose
byte +5 is clear: water and rock) its ring split, made the painter's terrain
(`terrain_painter_set_cell`, §2.7.2); repeated while either set holds a cell; then the last pass
0x5B5A70 (§2.7.3) once. `terrain_painter_flush(terrain', weight')` does the same without freeing,
with one difference: a set-2 cell whose word is not loaded is loaded (`terrain_painter_cell_load`)
before its set's byte +5 is read; then the painter's +4 and +8 take the two arguments and the
painter goes on with another terrain (`rmg_water_shores_seal`, step 11 twice).

**The picture sets** (`terrain_picture_sets` 0x642BD8: one per terrain). A picture has a *shape*
(0 the plain inside, 2-28 the transitions) and, in the first kind, a *flag* (the decorated plain
pictures). The first kind (`terrain_picture_set_init`): bytes +4 and +5, the flagged *share* in
percent (+8), the picture count and table (8 bytes a picture: shape u32, flag u8), and 58 (first,
count) ranges by shape × 2 + flag, built in one pass that assumes each (shape, flag) contiguous
(it is, in all five tables):

| Terrain | Set's static init | Bytes +4, +5 | Share | Pictures (table) | Shapes |
|---|---|---|---|---|---|
| 0 dirt | 0x5B3B60 | 1, 1 | 50 | 46 (`dirt_picture_shapes` 0x6428A0) | 8-13, 16, 0 (8 plain, 16 flagged), 24 |
| 1 sand | 0x5B3BA0 | 0, 1 | 70 | 24 (`sand_picture_shapes` 0x642A10) | 0 only (8 plain, 16 flagged) |
| 2 grass, 3 snow, 4 swamp, 5 rough, 6 subterranean, 7 lava | 0x5B3BE0 to 0x5B3D20 | 1, 1 | 50, 80, 80, 80, 60, 80 | 79 (`land_picture_shapes` 0x642628) | 2-22, 0 (8 plain, 16 flagged), 23-28 |
| 8 water | 0x5B3D60 | 0, 0 | 0 | 33 (`water_picture_shapes` 0x642AD0) | 8-13, 16, 0 (12 plain, none flagged) |
| 9 rock | 0x5B3DA0 (second kind) | 0, 0 | — | 48 (`rock_picture_shapes` 0x6424A8) | 0 (8), 8-13 by two variant bytes a, b |

The slots: 1 whether the flagged plain range is non-empty, 2 a picture's flag, 3 its shape, 4 a
plain picture, 5 a picture of a given shape. **Slot 4** (`terrain_picture_draw`, the weight w and
the current picture): a current picture of shape 0 is kept, no draw; else, when the flagged range
is non-empty, `rand() % 100 < (share × w) / 8` picks the flagged range, otherwise the plain one,
and `rand() % count + first` the picture: **two draws on land, one on water** (no flagged range);
with w = 4 the flagged share is half the set's share. **Slot 5** (`terrain_picture_draw_kept`,
the shape, a 16-bit word of two mirror flags, an out word, the current picture): a current
picture of that shape is kept, no draw; else `rand() % count + first` of the shape's plain range;
the word goes to the out pointer either way. The rock set (`rock_picture_ranges` 0x6A4158, 116
ranges by (shape × 2 + a) × 2 + b) has no flagged pictures and no mirroring: slot 4 keeps a shape-0
picture, else draws from the first range (one draw, the weight unused); slot 5 matches shape, a
and b, else draws from that range, and writes a zero word.

**Draws, floats, memory, limits.** Five draw sites: 0x5B38B9 (the flag roll) and 0x5B38D3 (the
picture) in slot 4, 0x5B3912 in slot 5, 0x5B3AB6 and 0x5B3B2D in rock's slots 4 and 5; who calls
them and how often: §2.7.2, §2.7.3. No floats; the cell words' fill (above); allocation failures
throw ('Allocation failure.'), the sets' nodes are unchecked (§2.7.2).

**Quirk candidates (as read; classified in `docs/rules/quirks-rmg.md`, T1c2):**

- The cell words' fill takes the map interface's address (above): harmless as far as read.

**Not read:** the C++ library helpers; 0x5B76D0 and 0x5B76F0 (reached only from
`terrain_painter_init`'s unwind funclet 0x6349D0).

#### 2.7.2 Adding cells: the rectangle and the work sets (T1c1b)

Path read: `terrain_painter_add_rect` 0x5B7690 → 0x5B4960 and everything it reaches by direct
calls, down to the two point sets' library code; the map's slots 1 and 4 (`rmg_map_set_tile`,
`rmg_map_get_tile`) and the picture set's slot 4 (`terrain_picture_draw` 0x5B3890, T1c1a's) only
as far as their arguments and results. 29 functions named (27 read, 2 inferred: the tree's two
rebalancing functions), 5 sites, no table of its own (it uses `0x642BD8`, the picture sets by
terrain, and the offsets table 0x6A80A8 with 0x5BC910: T1c1a's). No `crt_rand` call site: the
part draws through two calls of a picture set's slot 4.

| Function | Called from | What it does | Conf. |
|---|---|---|---|
| 0x5B4960 `terrain_painter_paint_rect` | `terrain_painter_add_rect` | the rectangle, row-major: a cell of another terrain to `set_cell`; a cell already of the terrain gets a new picture only (0x5B4A4F, written 0x5B4A6F) | read |
| 0x5B4B20 `terrain_painter_set_cell` | paint_rect; finish, flush, 0x5B5440, 0x5B76F0 | the cell made the terrain (picture 0x5B4B44, written 0x5B4B61), its neighbours' pinches re-tested, the sets updated (below) | read |
| 0x5B6FD0 `terrain_painter_decor_weight` | paint_rect, set_cell, 0x5B5A70 | the weight given to slot 4 | read |
| 0x5B50F0 `terrain_painter_queue_neighbours` | set_cell | foreign neighbours into set 2 | read |
| 0x5B6320 `terrain_painter_pinched_we`, 0x5B6430 `terrain_painter_pinched_ns` | set_cell; finish, flush, 0x5B5440, 0x5B76F0 | the two strip tests | read |
| 0x5B6810 `terrain_painter_ring_split`, 0x5B6540 `terrain_painter_ring_same` | set_cell; finish, flush, 0x5B5440 | the ring test (water, rock) | read |
| 0x5B48D0 `terrain_painter_cell_word`, 0x5B3DD0 `terrain_painter_cell_load` | everywhere in the painter | the per-cell cache of the map's tiles | read |
| 0x5B7CD0 `_set_insert`, 0x5B7FC0 `_set_find`, 0x5B7F60 `_set_erase_key`, 0x5B85C0 `_set_equal_range`, 0x5B85A0 `_set_lower_bound`, 0x5B8A20 `_set_upper_bound`, 0x5B8A40 `_set_lower_node`, 0x5B8B60 `_set_upper_node`, 0x5B8BC0 `_set_next`, 0x5B8AA0 `_set_prev`, 0x5B7E30 `_set_erase_range`, 0x5B85F0 `_set_free_subtree`, 0x5B8C70 `_set_distance`, 0x5B8CD0 `_set_distance_b`, 0x5B8CA0 `terrain_painter_point_less` (all `terrain_painter_`) | set_cell, queue_neighbours; finish, flush | MSVC's `std::set` of points | read |
| 0x5B8090 `terrain_painter_set_erase`, 0x5B8720 `terrain_painter_set_insert_node` | erase_range; insert | node out, node in, with the red-black rebalancing | inferred |
| 0x5B8CC0 `point_copy_construct`, 0x51B150 `iterator_bool_pair_init` | node insert; insert's callers (shared with other code) | library constructors | read |

**The sets.** Both work sets (+0x14 and +0x24) are MSVC 6's `std::set` of 8-byte points (x, y):
a red-black tree, nodes of 0x18 bytes (left, parent, right, the point at +0xC, colour +0x14),
the nil node the global 0x6A52C4, the size at +0xC of the set, unique keys (+8 false). The order
is `terrain_painter_point_less`: y first, then x, unsigned: **row-major, top to bottom, left to
right.** A key is the whole point, so there are no ties, and **nothing is ordered by address**: the
in-order sequence does not depend on the tree's shape or the nodes' addresses. A port can use any
ordered set of (y, x).

**The cell cache.** One u16 per cell (+0x38): bit 0 loaded, bits 1-4 terrain, bits 5-11 picture,
bits 12 and 13 the two mirror flags (map cell +0x28 bits 15 and 16), bits 14-15 untouched. A word
is loaded from the map (slot 4) whenever bit 0 is clear before any read of bits 1-13; the painter's
own writes update it. So the memory-dependent initial fill (T1c1a: the high half of the map
interface's pointer) never reaches a result in this part: bits 1-13 are always loaded or written
first, and bits 14-15 are only kept (mask 0xC001), never read (no read found in the terrain painter's code).
Neighbours outside the rectangle are read, from the map.

**The tests.** *Pinched W-E* (0x5B6320): 0 < x < width-1 and the W and E neighbours both of
another terrain than the cell's (they need not be equal). *Pinched N-S* (0x5B6430): the same
vertically. *Ring split* (0x5B6810): the 8 neighbours of the cell's own terrain (0x5B6540; order
N, NE, E, SE, S, SW, W, NW; off the map the clamped coordinate, so the edge counts as the same
terrain; a diagonal counts only when one of its two adjacent orthogonals does) fall into two or
more separate arcs of the ring. A cell is *pinched* when W-E or N-S holds, or, for a terrain whose
picture set has byte +5 clear (water 8 and rock 9, T1c1a's sets), the ring is split. Set 1
(+0x14) holds pinched cells of the painter's terrain; set 2 (+0x24) holds cells of other terrains
to re-test (finish absorbs a pinched one into the painter's terrain through `set_cell`).

**`set_cell` (0x5B4B20), in order.** (1) The weight, a picture drawn, the cell written (terrain,
picture, mirror flags cleared) and cached; the cell erased from set 2. (2) Land terrains (set byte
+5 set): the N and S neighbours that are in set 1 and no longer pinched W-E, and the W and E
neighbours in set 1 no longer pinched N-S, are erased from set 1 and queued (the new cell has
broken their other pinch); nothing is added. Water and rock: each of the 8 neighbours in the map
(0x5BC910) of the painter's terrain is re-tested in the ring order: in set 1 and no longer pinched →
erased and queued; not in set 1 and now pinched → inserted. (3) The cell itself: pinched → into
set 1; else queued. **`queue_neighbours`** (0x5B50F0) inserts into set 2: the first of N, S that
is in the map and of another terrain (S only when N is not, 0x5B5159), the same for W, E, then
each diagonal of another terrain whose set has byte +5 clear.

**`paint_rect` (0x5B4960).** For y from y0 to y0+h-1, x from x0 to x0+w-1 (no clamp): another
terrain → `set_cell`; the same terrain → a new picture with the same weight rule, written with the
mirror flags cleared, no set change. **Every cell of the rectangle is drawn a picture, once.**

**The weight (0x5B6FD0).** w = the painter's +8 (4 in the generator's calls), halved for each of
the W, N, E, S neighbours in the map of the painter's terrain whose current picture has the
set's flag (slot 2): 4, 2, 1 or 0, the order of the halvings irrelevant. Slot 4 is called with
(w, -1): with -1 it always draws (T1c1a: `rand() % 100 < (share × w) >> 3` picks the flagged
range when that range is non-empty, then `rand() % count + first`). The weight reads the
neighbours' pictures as they are: in the rectangle's order the W and N ones are already redrawn,
the E and S ones still hold the map's earlier pictures (a port must keep them, e.g. the water fill's).

**The draws in order.** No `crt_rand` site of its own. Per `paint_rect`, one slot-4 call per cell
in row-major order (0x5B4A4F for a cell already of the terrain, 0x5B4B44 inside `set_cell`
otherwise), each one or two draws (two when the set has a non-empty flagged range, the share
then decides). Later, one more slot-4 call (0x5B4B44) for every `set_cell` of finish, flush or
0x5B5440, in their order: sets taken smallest (y, x) first. The set work itself draws nothing.

**Floats, memory, limits.** No floats. Memory: none here (above). Allocation: one 0x18-byte
node per insert through operator new, unchecked. No iteration limit in this part (the sets drain
in finish, T1c1a); `_set_free_subtree` recurses to the tree's height. The rectangle is not
clamped: a rectangle past the map's width would wrap into the next row, past its height would
read past the cache (the generator's callers pass rectangles inside the map, not checked here).

**Quirk candidates (as read; classified in `docs/rules/quirks-rmg.md`, T1c2).**
- `queue_neighbours` is an else-if per axis: when N is queued, S is not re-tested (likewise E
  after W), so a foreign S (E) neighbour that the new cell has just pinched may stay a 1-wide strip.
- `set_cell` never erases its own cell from set 1: a cell already in set 1 passed to it stays
  there even when no longer pinched (whether reachable depends on 0x5B5440, T1c1c).
- Repainting redraws: `paint_rect` gives every cell already of the terrain a new picture and
  clears its mirror flags, so each painted cell costs draws even when nothing changes; a pinched
  cell is drawn before it enters set 1 and may be drawn again when 0x5B5440 resolves it.

**Not read.** The red-black rebalancing in 0x5B8090 and 0x5B8720 (it cannot change the in-order
sequence); the picture sets' slots (T1c1a); finish, flush, 0x5B5440, 0x5B5A70, 0x5B76D0/0x5B76F0
(a third caller of `set_cell`), 0x5B4860 (calls `_set_erase_range`, likely the destructor),
0x5B8670 (a set's head node) beyond their calls into this part; which terrains reach the
water/rock branch other than by the sets' byte +5 as T1c1a gives it.

#### 2.7.3 The finishing passes: pinched cells and the transitions (T1c1c)

Path read: `terrain_painter_finish` 0x5B72F0 and `terrain_painter_flush` 0x5B7520 (T1c1a's) →
0x5B5440 `terrain_painter_resolve_pinch` and 0x5B5A70 `terrain_painter_transition_pass`, whole,
and everything they reach by direct calls; the picture sets' slots 4 and 5 (T1c1a's) and
`terrain_painter_set_cell`, `_decor_weight` and the pinch and ring tests (T1c1b's) only as far as
their arguments and results. 15 functions named (14 read; 0x5B76F0 inferred, 0x5B76D0 read: the unwind path), 8 sites, 4 tables (one `.rdata`, three
built on first use). No `crt_rand` call site of its own: the draws are the picture sets' (slot 4
through `set_cell` 0x5B4B44 and in the last pass; slot 5 in the last pass).

| Function | Called from | What it does | Conf. |
|---|---|---|---|
| 0x5B5440 `terrain_painter_resolve_pinch` | finish, flush, 0x5B76F0 | one set-1 cell: its N-S pinch, then W-E, then (water, rock) its ring mended through `set_cell` | read |
| 0x5B76B0 `terrain_painter_point_make`, 0x47A670 `std_tree_begin` | 0x5B5440; finish, 0x5B76F0 (shared) | a point from two coordinates; a tree's smallest node | read |
| 0x5B5A70 `terrain_painter_transition_pass` | finish, flush, 0x5B76F0 | every map cell's picture and mirror flags from its neighbours | read |
| 0x5B68A0 `terrain_painter_border_classes`, 0x5B3E40 `terrain_painter_border_class` | 0x5B5A70 | the 8 neighbours' classes 0, 1, 2 | read |
| 0x5B3E80 `terrain_painter_transition_shape` | 0x5B5A70 | classes → shape 0-28 and mirroring | read |
| 0x5B6BA0 `terrain_painter_corner_arm_same`, 0x5B6E00 `terrain_painter_corner_far_differs` | 0x5B5A70 | the refinements 2/8 → 6/12 and 5/11 → 7/13 | read |
| 0x5B8060 `terrain_painter_bytes_fill`; 0x5B45E0, 0x5B6DF0, 0x5B6FC0 (`_destroy`) | 0x5B5A70; at exit | a byte fill; three empty exit functions of the static tables | read |

**Resolving a pinched cell (0x5B5440).** For cell p of the painter's terrain, in this order:
(1) *Pinched N-S* (N and S of other terrains): one of N, S is given to `set_cell` (0x5B5687):
N if N is itself pinched (W-E or N-S against its own terrain, or, when its own terrain's set has
byte +5 clear, its ring split); else S if S is; else N if x = 0 or x = width-1; else N if NW or
NE is the painter's terrain; else S if SW or SE is; else N. (2) p re-tested *pinched W-E*: the
same with W for N, E for S, y for x, and NW/SW, NE/SE for the diagonal pairs (0x5B58C7).
(3) Only when the painter's terrain's set has byte +5 clear (water, rock) and p's ring is split:
p's same-terrain flags (`ring_same`) are read once; the *gaps* (maximal runs of unflagged
neighbours) are listed in ring order starting after the first flagged one, each with a cost of 2
per orthogonal and 1 per diagonal. While more than one gap is left, the cheapest (strictly less
wins, so the first listed on ties) has each of its on-map cells (`neighbours_on_map`) given to
`set_cell` in ring order from its first cell (0x5B5A15) and is struck off; the costliest gap (the
last listed on ties) stays. Nothing is recomputed between gaps. It never touches the sets:
p leaves set 1 through `set_cell`'s re-test of the changed cells' neighbours (T1c1b); finish
calls it again on the smallest cell until it has. Each call converts at least one cell (a gap's
discounted diagonal comes with its two orthogonals), so the loop ends.

**The last pass (0x5B5A70).** Runs once per finish or flush, over **the whole map**, not the
painted area. First a byte per cell (new, unchecked; filled 0) counts its on-map 8-neighbours of
another terrain (each differing pair counted once for both). Then every cell, rows top to bottom,
each left to right:
- *Classes* (0x5B68A0): for each neighbour, coordinates clamped to the map (off the map the edge
  cell, possibly p itself), class 0 when its terrain is p's or p is sand; else, when both
  terrains' sets have **byte +4** (dirt, grass … lava: the terrains that meet through dirt), 1 if p
  is not dirt and 0 if it is; else 2. So sand never borders, dirt borders only sand, water and rock
  (class 2), the other lands take dirt borders (1) from dirt and other lands and sand borders (2)
  from sand, water and rock; water and rock border everything (2).
- *Shape* (0x5B3E80): ten groups tried in order; within a group each mirroring in the order none,
  N-S, W-E, both (`terrain_painter_mirror_words` 0x6A52B8; a role names the neighbour that
  `terrain_painter_mirror_roles` 0x642C00 maps to it), the conditions in the order listed; the
  first that holds returns the shape and that mirroring (word byte 0 = W-E swap → cell bit 12,
  byte 1 = N-S swap → bit 13):

| Group | Conditions → shape |
|---|---|
| 1 | E=1, S=1: NE=2 and SW=2 → 28, else SE=2 → 27 |
| 2 | N=1, W=1, SE≠0 → 23 (SE=1), 25 (SE=2); N=2, W=2, SE≠0 → 24 (SE=2), 26 (SE=1) |
| 3 | E=2, S=1 → 8 with both mirror bytes inverted if SW=2, else 21; E=1, S=2 → 8 inverted if NE=2, else 22 |
| 4 | E=1, S=1: SW=2 → 17, NE=2 → 18 |
| 5 | N=1, W=1 → 2; N=2, W=2 → 8 |
| 6 | E=1, SW=2 → 17; S=1, NE=2 → 18; E=2, SW=1 → 21; S=2, NE=1 → 22; W=1, NE=1 → 2; N=1, SW=1 → 2; W=2, NE=2 → 8; N=2, SW=2 → 8 |
| 7 | E=1, SE=2 → 19; S=1, SE=2 → 20 |
| 8 | N=1 → 4; N=2 → 10; W=1 → 3; W=2 → 9 |
| 9 | NW=1, SE=1 → 14; NW=1, SE=2 → 15; NW=2, SE=2 → 16 |
| 10 | SE=1 → 5; SE=2 → 11 |
| — | none: shape 0, no mirroring |

- *Refinements:* 2 → 6 and 8 → 12 when either cell past the corner's arms (unmirrored SW and NE;
  `terrain_painter_corner_arm_offsets` 0x6A5260), clamped, has p's terrain; 5 → 7 and 11 → 13 when
  the cell two steps along x or two along y toward the bordering diagonal (unmirrored (x+2, y),
  (x, y+2); 0x6A3D68), clamped, has another terrain.
- *Picture,* from p's own terrain's set: shape ≠ 0 → slot 5 (0x5B60B6; draw kept: the current
  picture kept when it has that shape, for rock that shape and variants (a, b) = the mirror bytes;
  the first kind passes the mirroring on, rock clears it). Shape 0, bordered or not → slot 4 with
  `terrain_painter_decor_weight` of p's terrain (0x5B60D4, 0x5B6231; kept when the current picture
  has shape 0). The count only picks one of two paths with the same
  effect: a port may run the shape rule on every cell.
- *Write* through the map's slot 1 (0x5B6104, 0x5B6262) only when the picture or a mirror flag
  changes. Terrains are never changed here.

The shapes each set holds cover what its classes can produce (dirt: 8-13, 16, 24; land 2-28), with
three gaps: water lacks 24, rock 16 and 24 (rock's 9 has variants (0,0), (1,0) and 10 (0,0), (0,1),
exactly what group 8 gives). Those patterns need a pinched or ring-split water or rock cell.

**The draws in order.** Finish/flush (T1c1a): while set 1 is not empty, `resolve_pinch` on its
smallest (y, x): its `set_cell` calls in the order N-S, W-E, gaps (cheapest first, ring order
inside), each one slot-4 draw with weight and -1, so always drawing: 2 `crt_rand` for dirt, sand
and grass … lava (a non-empty flagged range), 1 for water and rock. Then the last pass, row-major
over the whole map, per cell: slot 5 → 0 draws when the current picture already has the shape
(rock: and the variants), else 1 (`% count` of that shape's range); slot 4 → 0 when the current
picture has shape 0, else 2 (1 for water and rock). The weight reads the orthogonal neighbours'
current pictures: W and N already redone in this pass, E and S not yet. A port must keep this
order; nothing in it is ordered by address.

**Floats, memory, limits.** No floats; nothing memory-dependent. The count array: one unchecked
new per pass, filled 0. The three static tables: constants built on first use. The gap records
(at most 4) fit their stack array. No iteration limit: finish's loop ends as argued above.

**Quirk candidates (as read; classified in `docs/rules/quirks-rmg.md`, T1c2).**
- Each finish or flush re-fits the whole map's pictures, far from the painted area too.
- A ring gap can hold a diagonal of the painter's own terrain (discounted by `ring_same`): it goes
  to `set_cell` and is redrawn without changing.
- Group 3 gives sand-corner shape 8 with the mirroring inverted where one side is a dirt border.
- A shape missing from a set (water 24; rock 16, 24) would make slot 5 divide by a zero count.

**Not read.** The slots' internals (T1c1a); `set_cell`, the pinch, ring and weight functions
(T1c1b); 0x5B76F0 `terrain_painter_destroy` (inferred: finish's loop and last pass, then the
frees) past its loop, reached only through 0x5B76D0 from `terrain_painter_init`'s unwind funclet;
the C runtime's exit registration; whether a caller can leave a pinched water or rock cell, or a
picture outside its terrain's table, at a last pass.

#### 2.7.4 The road and river painter (T1c1a)

Path read: the three entries from `rmg_road_trace`, `rmg_river_to_source` and
`rmg_river_to_mouth` (`road_painter_init` 0x55F3B0 or `river_painter_init` 0x55EE50, then
`road_river_stroke_to` 0x4FA2B0 per step, the base destructor at a run's end) and all they reach,
with both painters' vtables and both picture sets. 31 functions named (all read), 1 draw site,
8 tables and globals.

| Function | Called from | What it does | Conf. |
|---|---|---|---|
| 0x55F3B0 `road_painter_init`, 0x55EE50 `river_painter_init`; 0x55F460, 0x55EDA0 (base destroy); 0x55F430, 0x55EED0 (delete) | steps 14, 15 | a painter on a road or river view (vtables 0x64120C, 0x641190) | read |
| 0x55F320, 0x55EDB0 (the picture set), 0x55EDC0 `road_river_painter_set_cell`, 0x55F330 `_set_type`, 0x55EE00 `_blocked`, 0x55F350 `_get_cell`, 0x55F390, 0x55EE30 (the type) | slots 0-5 | the view's slots forwarded; slot 3 is water or rock land | read |
| 0x4FA280 `road_river_stroke_start`, 0x4FA2B0 `road_river_stroke_to`, 0x4FA3C0 `road_river_stroke_visit` | the painters' init; the three walks | the stroke (below) | read |
| 0x4FA080 `road_river_rect_clear`, 0x4F9F00 `road_river_picture_draw`, 0x4F9CB0 `road_river_shape_choose` | the visit | another type cleared; a cell's picture; its shape | read |
| 0x4F9BE0 `road_river_picture_set_init` (from 0x55F2F0, 0x55ED70) | the C runtime's initializer table | `road_picture_set` 0x69E650, `river_picture_set` 0x69E5D0 | read |
| 0x5BC910 `neighbours_on_map`, 0x5BC890 `neighbour_offsets_init`; 0x4FA050, 0x4FA520, 0x4FA540 | shared with the terrain painter; helpers | which of the 8 neighbours lie on the map; `neighbour_offsets` 0x6A80A8 | read |

**The neighbours.** `neighbour_offsets` 0x6A80A8 orders the 8 neighbours clockwise from north:
N, NE, E, SE, S, SW, W, NW; both painters use that order. `neighbours_on_map` clears the row above
on the first row (else the row below on the last) and the column left on the first column (else
the column right on the last).

**The stroke.** A painter is made at a point with a type (road 1-3, river 1-4): it visits that
point. `road_river_stroke_to(t)` walks a line **from t back toward the current point**, one cell
per step of the longer axis (x on a tie), the shorter axis by an error sum, a step of the shorter
axis visiting its corner cell too (a staircase, never a diagonal move); the current point is
visited again only when the error sum reaches the longer length at the end; t becomes the
current point. The generator's walks pass one-cell steps (`rmg_road_trace` an orthogonal one, a
new painter after each diagonal step or level change, §2.5; the river walks' calls not traced
step by step), so each step visits its target alone. **A visit** (`road_river_stroke_visit`): nothing when the cell already has the type or its
land is water or rock; a cell of another type is cleared first (`road_river_rect_clear`: type 0,
picture 0, no flips; then the cells around it of a non-zero type redrawn: the column left from
the row above to the row below, the column right the same but without its last row's cell when
the cell lies on the second-last row, the row above, the row below); the type set, **the cell
drawn**, then **each of its 8 neighbours on the map that holds the type drawn again**, in the
neighbours' order.

**A cell's picture** (`road_river_picture_draw`): the same-type flags of its 8 neighbours (on the
map), the shape and two mirror flags by `road_river_shape_choose`, then: when the cell's stored
picture already has that shape (by the set's table) and the same flags, nothing; else `rand() %
count + first` of the shape's range, written with the flags. A new cell holds picture 0 (shape 4
in both sets) without flags, so a new cell whose shape is an unmirrored 4 is not drawn. **The
shape** (N, E, S, W the orthogonal flags): all four → 8; N and S → 6 with E, 6 mirrored
left-right with W, else 2; E and W → 7 with S, 7 mirrored top-bottom with N, else 3; else the
first corner of E+S, E+N, W+S, W+N (`road_river_corner_flips` 0x63FF1C, the neighbours mirrored
by `road_river_mirror_maps` 0x63FE9C) → 5 when the set has shape-5 pictures and the mirrored NE
or SW neighbour holds the type (a diagonal run), else 4, with the corner's flags; else, when the
set has shape-0 pictures, 1 for a lone E or W neighbour (mirrored left-right when W), 0 otherwise
(mirrored top-bottom unless the neighbour is S); without them 3 for E or W, else 2.

**The picture sets** (`road_river_picture_set_init`: the count, the shape per picture, nine
(first, count) ranges by shape, contiguous): roads 17 pictures (`road_picture_shapes` 0x6411AC:
two each for shapes 2, 3, 4, 6, 7, four for 5, one each for 0, 1, 8), rivers 13
(`river_picture_shapes` 0x641140: four for 4, one for 8, two each for 2, 3, 6, 7; no 0, 1 or 5,
so a river has no end pieces and no diagonal corners). Every shape the choice can give has a
range in its set (no division by zero).

**Draws, floats, memory, limits.** One draw site, 0x4FA00F, at most one draw per cell drawn: per
visit the cell (unless its picture already fits) and each same-type neighbour whose picture no
longer fits, plus a clearing's redraws. No floats, nothing memory-dependent, no limits; the views
bound nothing (the walks stay on the map).

**Checked against the oracle** (T1b2's harness; a script over the step dumps): every road cell
of `s1-2p`, `s2-2p` and `m1-4p` after step 14 (67, 128 and 229 cells) and every river cell of
`m1-4p` after step 15 (111) has the shape and mirror flags the choice gives (road picture +0x28
bits 0-7, flags 19 and 20; river picture +0x24 bits 18-25, flags +0x28 bits 17 and 18), the roads
still so after step 15; every town's entrance cell holds road (9 towns).

**Quirk candidates (as read; classified in `docs/rules/quirks-rmg.md`, T1c2):**

- `road_river_rect_clear`'s right column stops one row short when the cleared cell lies on the
  map's second-last row (its bound compares with height − 1, the left column's with height): that
  corner neighbour keeps a stale picture.
- A cell whose computed shape is an unmirrored 4 keeps picture 0 without a draw (picture 0 is the
  first of shape 4's range): the stream and the picture depend on the stored value, not only on
  the shape.

**Not read:** the road and river views' slots beyond what the painters call (§2.5 read them).

#### 2.7.5 The painters' draws in the stream; the oracle's check (T1c1)

The rows §2.6 leaves out, in the order they draw inside their steps (the painters' sites only; the
steps' own sites stay §2.6's):

| Step | Painter draws in order | Per |
|---|---|---|
| 3 | two levels: level 1 painted rock, one rectangle (0x5B3AB6 per cell), its finish (nothing to draw); level 0 painted water (0x5B38D3 per cell), its finish; then per zone not of water, in zone order, a painter of its terrain: its cells with bit 28, row by row over the zone's box, a one-cell rectangle each (0x5B38B9, 0x5B38D3), then its finish: the pinched cells (`set_cell`'s draws, §2.7.3) and the last pass over the whole map (0x5B3912 or 0x5B3B2D per bordered cell whose shape changed, slot 4's draws per plain cell whose current picture is not plain) | cell; painter |
| 6 | the islets (`rmg_islet_draw`) and sealed shores (`rmg_water_shores_seal`): their rectangles' cells row-major, then finish or flush as in step 3 | islet, shore |
| 11 | `rmg_underground_rock_paint`'s rectangles, flushes and finish, as in step 3 | painter |
| 14 | per road cell visited: 0x4FA00F for the cell unless its picture fits, then for each same-type neighbour whose picture no longer fits (N, NE, …, NW) | cell visited |
| 15 | as step 14 with the river set, interleaved with the floods' draws (§2.5: each course painted when its flood ends) | cell visited |

A slot-4 call draws twice on land (the flag roll, then the picture), once on water and rock; a
slot-5 call once; a call whose current picture already fits draws nothing. Every order is a
coordinate order (the sets row-major, the passes row-major, the neighbours clockwise from north):
nothing depends on addresses, so the stream follows from the map alone.

**Checked against the phase oracle** (`s1-2p`, `s2-2p`, `m1-4p` run again with `--phases`: the
digests as `kept.txt` and `phases.txt` keep them):

- *The sites:* steps 3, 6 and 11 draw only at the five picture sites and steps 14 and 15 only at
  0x4FA00F besides §2.6's. Step 3 of `s2-2p` begins with 1 296 draws at 0x5B3AB6 (the rock level),
  then 1 296 at 0x5B38D3 (the water level), then alternating 0x5B38B9, 0x5B38D3 (the zones' cells);
  in all three cases 0x5B38D3 counts exactly one draw per water cell more than 0x5B38B9 (1 296 + 1 391,
  1 296 + 2 780, 5 184 + 5 395): every land picture rolls its flag, no water picture does.
- *The pictures:* a script over the step dumps applied §2.7.3's transition rule and §2.7.4's shape
  rule to every cell: after steps 3, 6, 11 and 15 every cell's terrain picture has the shape and
  mirror flags the rule gives (rock: its variant bytes; 1 296, 2 × 1 296 and 5 184 cells a level),
  every road cell after step 14 (67, 128, 229) and every river cell after step 15 (111 in `m1-4p`)
  the shape and flags of its neighbours. The draw counts per cell and the stream's order beyond the
  sites are increment 3's comparison (the port against the log).

## 3 The template format, each phase's rules, the objects and the writer (T1c2)

Three parts written by worker sub-agents from §1, §2 and the tables (§3.1), the readings (§3.2)
and the oracle's corpus with `h3_map` (§3.3); the quirk candidates of §1 to §3 are classified in the
ledger `docs/rules/quirks-rmg.md` (RMG-QUIRK-1 to -23, the user, 2026-11-24), which also lists the
candidates considered and not quirks, and the two still open.

### 3.1 The template format; AB's and SoD's tables

**Inputs.** AB's table (`H3ab_bmp.lod`, 124 341 bytes, SHA-256 `0de02827…`, 50 templates) and
SoD's (`H3bitmap.lod`, 129 956 bytes, `feb881da…`, 53; the one the game reads, `ORACLE.md`; SoD's
standalone install holds the same bytes); original data, never committed. The reader as §1.4 and
§2.1 place it: `rmg_templates_read` 0x537FF0, `rmg_template_zones_read` 0x538480,
`rmg_template_zone_find` 0x5330A0; records `rmg_template`, `rmg_zone`, `rmg_zone_connection`.
HotA's template-format page (h3hota.com/en/template-format) was a pointer for Wide and Border
Guard only.

**The file.** Latin-1, CRLF rows, tab-separated cells. Four cells are quoted: template names that
hold a line break, an LF after a space (Ready⏎or Not, South⏎of Hell, Worlds⏎at War,
Meeting⏎in Muzgob). How the game's table loader treats the quotes is inferred
(`text_table_columns` 0x5BBE70, not read for this file); the observation agrees: a parser of
quoted cells predicts the template of all 21 observed cases, and a generated map's description
prints the break (`ORACLE.md`). Rows 0-2 are headers (group labels
and column names), not read. *Set* means not empty and not beginning with a space; numbers go
through `c_atoi` 0x617F09 (an empty or space cell gives 0, "5.9" gives 5).

**Row kinds** (from row 3; a row can be all three; the tests count cells):

| Kind | Test | AB / SoD |
|---|---|---|
| a template's first row | 2 or more cells, column 0 set; the template runs to the next such row | 50 / 53, each also a zone and a connection row |
| zone row | 76 or more cells, column 3 set | 616 / 646 |
| connection row | 85 or more cells, column 76 set, 77 not empty (map.toml: both set; no row of the tables tells the two apart) | 833 / 869, of which 611 / 641 are also zone rows |
| anything else | not read | 6 blank rows (in 2SM2f, 2SM2f(2)) |

Every row has 85 cells (the first 16 rows 183, columns 85-182 empty), so the thresholds never
bite on the originals; a port applies them for other tables.

**Columns** (from 0; full map and fields in §2.1 and `structs.toml`). Everything in 0-84 is read
on a row of its kind; nothing past 84; the headers not.

| Columns | Meaning | Encoding in the tables |
|---|---|---|
| 0-2 | name, least and greatest size index (w × h × levels / 1296, islands halved, ≥ 1: reachable 1, 2, 4, 8, 9, 16, 18, 32) | names unique but one (below); sizes 1..32, never min > max |
| 3 | zone id; connections find zones by it before the kept zones become 0, 1, … in row order | 1..n in every template, three not in row order |
| 4-7 | human start, computer start, treasure, junction: the last set wins, none treasure | marks always `x`; no zone has two |
| 8 | base size | 5..50 |
| 9-12 | least and most players, least and most players + computers: else the zone is dropped | numbers |
| 13 | Ownership: position = value − 1 | 0..8 |
| 14-17, 18-21 | least towns, castles, town density, castle density: colour's, neutral | numbers |
| 22; 23-31 | same type; Castle to Conflux (header: Elemental) | marks |
| 32-38; 39-45 | least mines; mine density (wood, mercury, ore, sulfur, crystal, gems, gold) | 0..2; 0..4 |
| 46; 47-54 | terrain matches town; dirt to lava (none set: dirt) | marks |
| 55 | strength by first letter: n 0, w 2, s 4, else 3 | avg, average, strong, weak, none; never empty |
| 56; 57; 58-66 | monsters match town; neutral; Castle to the ninth (header: Forge, a label; byte +0x9E) | marks |
| 67-75 | three bands: low, high, density | 0..40 000 (AB), 60 000 (SoD) |
| 76-84 | zone ids, value (guard), wide, border guard, the four positions: else the connection is dropped | values 0..45 000; wide 21, border guard 8 (Gauntlet, Ready⏎or Not, Worlds⏎at War), all of value 0 |

All nine connection columns are read (§2.1, `rmg_zone_connection`; §1.4's "read in part" is
T1a1's, before T1a2a read them); +0xA `created` is not from the file. The space rule decides
cells: SoD has 57 single-space cells (AB the same rows), in Gauntlet's zone 1 and in Worlds at
War, where zones 16 and 17 have column 6 `x` and column 7 a space: treasure, junctions if a space
counted.

**What a harvester must keep.** Template row order (the draw is `rand() %` the kept count over the
kept in row order, §1.5); zone row order (indices come from it, not from the ids: 2SM2b, 2SM2b(2)
and 2SM2c list ids 1, 2, 4, 5, 3, 6); connection row order (each is appended to both ends' vectors
in row order, which step 6 walks, §2.3); a template by its index, not its name (two are named
6LM10); a connection per row, not per pair (51 repeats of a pair in 9 templates are separate
connections); names with their break.

**AB's against SoD's.** AB's file is a byte prefix of SoD's: SoD adds 36 rows after the last
template (7SB0c) and changes nothing before them. The 50 common templates are byte for byte the
same, in the same order: no difference in sizes, zones, towns, mines, terrains, monsters, bands or
connections. SoD's three:

| Template | Sizes | Zones (starts / treasure / junction) | Positions (zones) | Connections, values | Bands' top |
|---|---|---|---|---|---|
| Golems Aplenty | 6-18 | 7 (4 / 3 / 0) | 1-4 players, exactly 4 in all | 8, 7000-10 000 (one never kept, below) | 60 000 |
| Monk's Retreat | 9-32 | 9 (6 / 3 / 0) | 1-6, exactly 6 | 14, 6000-18 000 | 30 000 |
| Rise of Phoenix | 9-18 | 14 (4 / 9 / 1) | 1-4, exactly 4 | 14, 9000-24 000 | 40 000 |

None has a computer start, a wide or a border-guard connection. With AB's table the game would
keep fewer templates for some setups and draw among another count (`ORACLE.md`: AB's table mispredicts
8 of the 21 observed cases).

**Quirk candidates (as read; in both tables unless said; classified in `docs/rules/quirks-rmg.md`, T1c2: 8XM8 RMG-QUIRK-22, the self-connection and Ownership 0 open, the others not quirks):**

- **8XM8's decimal ids.** Six connection rows give their first zone as 5.9, 6.3, 6.7, 7.1, 7.5,
  7.9 (a filled series); `c_atoi` makes them 5, 6, 6, 7, 7, 7. Human start 8 then has no
  connection, 5 and 7 three, 6 two; starts 1-4 have two each.
- **Connections never kept:** empty columns 81-84 read 0, below every player count. Newcomers 6
  of 20 (each the reverse of a kept one); Golems Aplenty (SoD) 5-7, the only one between them.
- **A self-connection:** both 6LM10s join zone 13 to itself (value 6000): appended to zone 13
  twice; what step 6 makes of it is not read here.
- **Start zones with Ownership 0** (position −1): 2SM2i(2)'s human starts 3 and 5, 2SM0k's
  computer starts 3-8, none asking a colour town (some a neutral one); the colour table is
  indexed position + 1 (§1.7): slot 0.
- **Treasure zones with Ownership** and colour towns: Newcomers 3, 4, 6, 8-11, Meeting in Muzgob
  7 and 9; which colour steps 4 and 5 give them is not read here.
- Lesser: Newcomers 12-14 have no terrain and no match (dirt by the reader's default); Dragons'
  computer start 32 asks no colour town (only neutral densities); South of Hell 8-10 and Worlds
  at War 9-11, 17 set no monster town, match or neutral (strength none); 3SM3d's junctions 11-14 have every band 0.

Nothing else odd: no zone without an id, no unknown mark, no id named by a connection but
missing, every template kept by some size and setup.

**For increment 2's harvest** (`packs/ab/rmg.toml`, `packs/sod/rmg.toml`). Keep, in row order:
per template the name, least and greatest size; per zone the id, the type as resolved (0-3), base
size, the four positions, Ownership, the eight town counts and densities, same type, the 9 town
types, the 7 + 7 mine numbers, terrain match, the 8 terrains, strength as resolved (0, 2, 3, 4),
monsters' match and neutral, the 9 monster towns, the 3 × 3 bands; per connection the two ids as
`c_atoi` reads them, value, wide, border guard, the four positions. Drop: the header rows, columns
85 on, blank rows, the mark text and strength words (only set-ness and the first letter count).
Keep the rows as read (8XM8's ids with RMG-QUIRK-22's override for `fixed`, the never-kept
connections, the self-connection). Sanity counts:

| Table | Templates | Zones (0 / 1 / 2 / 3) | Connections (never kept) |
|---|---|---|---|
| AB | 50 | 616 (222 / 30 / 308 / 56) | 833 (6) |
| SoD | 53 | 646 (236 / 30 / 323 / 57) | 869 (7) |

### 3.2 Each phase's rules

The rules a port must implement, phase by phase in the run's order, each the size of one catalog
entry (one testable behaviour: inputs → outcome, order, draws), condensed from §2. The ids
`RMG-<AREA>-<NNN>` are proposals (CTX `RMG` is not yet in the catalog's list,
`docs/04-rules-and-sources.md` §3); the increments that write the entries assign them. Areas:
SET choices, settings, seed; TPL files read, template, positions; OBJ the object generators and
placing (shared by the steps); ZON steps 1–2; TER step 3, step 11 and the terrain painter; TWN steps
4–5; CON steps 6 and 9; OBS steps 7, 12, 13; MIN step 8; TRS step 10; RDR steps 14–15 and the road
and river painter; WRT the writer. A draw is `site %modulus`; *(QC: …)* marks a quirk candidate of
§2 (classified in `docs/rules/quirks-rmg.md`, T1c2).

#### 3.2.1 The setup

| Id | Rule | Draws (sites, in order) | § |
|---|---|---|---|
| RMG-SET-001 | Players Random: h + rand % (9 − h), h the human slots; their teams Random: rand % players + 1 (0 or above the players: the players). From the game's `rand` state, before the seed. | 0x586168 %(9−h); 0x586183 %players | §1.1 |
| RMG-SET-002 | Computers Random: rand % (9 − players), none with 8 players; their teams, only with computers: rand % computers + 1 (0: the computers; above them: the players). *(QC: the fallback to the players)* | 0x5861B1 %(9−players); 0x5861E4 %computers | §1.1 |
| RMG-SET-003 | Monster strength Random: rand % 3 − 1. Water's Random goes on as 3. | 0x58621D %3 | §1.1 |
| RMG-SET-004 | The parameters' defaults; each colour's human flag and town from the setup; version 0 RoE, 1 AB, 2 SoD (setup 1 AB, 2–3 SoD, else RoE). | — | §1.1 |
| RMG-SET-005 | The run: monster strength = parameters' + 3 clamped 1..5; fewer than 2 players and computers become 1 and 1; codes 0 done, 1 exception, 2 writer false, 3 no template. | — | §1.2 |
| RMG-SET-006 | The seed: `time()` seconds at +4, `srand` with it before the objects are read; one C-runtime `rand` stream for the run. The port takes the seed as input. | — | §1.3 |
| RMG-SET-007 | Water Random drawn after the seed: then the generation's first draw. | 0x537C89 %3 | §1.3 |
| RMG-SET-008 | Heroes banned: table byte +0x3A set, or the version's byte clear (+0x39 AB, SoD; +0x38 RoE). | — | §1.3 |
| RMG-SET-009 | Limits by object type, per map and per zone: 32000, then the constant pairs (map: 200 types 6, 26; 64 type 8; 48 types 57, 83; 32 for 25 others; zone: 1 for 23 types, 3 for 113). | — | §1.3, §2.4 |
| RMG-TPL-001 | `rmg.txt`: a template at a row of ≥ 2 columns (name, least, greatest size), a zone at ≥ 76 with column 3 set, a connection at ≥ 85 with 76 set and 77 not empty (*set*: not empty, no leading space). | — | §2.1 |
| RMG-TPL-002 | A zone's fields by §2.1's column map: type the last of columns 4–7 set (none: treasure); terrains none: dirt; monster strength by first letter n 0, w 2, s 4, else 3. | — | §2.1 |
| RMG-TPL-003 | RoE reads 8 town types (clears Conflux) and clears monster-town byte +0x9D. *(QC: +0x9D is Fortress, Forge expected)* | — | §2.1 |
| RMG-TPL-004 | A zone or connection whose player ranges do not fit players and players + computers is dropped; a connection naming a dropped zone too. | — | §2.1 |
| RMG-TPL-005 | Kept: size w × h × levels / 1296 (islands halved, ≥ 1) in range, human starts ≥ players, start zones ≥ players + computers; zones renumbered 0, 1, … in row order. None kept: code 3. | — | §2.1 |
| RMG-TPL-006 | `rand_trn.txt`: rows from 3 to the first empty column 0; type, subtype, terrain, 9 modifiers (+ a tenth, −5000), Adjacent and Overlap per obstacle (109); a prototype takes the last row of its group, first terrain, subtype (used by step 13). | — | §2.1 |
| RMG-OBJ-001 | The object generators: 29 groups in a fixed order, type, subtype, value, density constants (§2.1's table); tents per tent prototype from the last, seer huts per seer prototype upward. | — | §2.1 |
| RMG-OBJ-002 | A creature reward: one per creature, ids downward (118 RoE, 145 AB, SoD); count budget[level] / AI value rounded (> 50 tens, > 12 fives, > 5 up to even); density 3. | — | §2.1 |
| RMG-OBJ-003 | A generator's value for a zone (−1 refuses): creatures and dwellings of another town than the zone's refused; AI × count (dwellings × growth) raised by their towns' share (dwellings + AI × count / 2). *(QC: town type −1 passes neutrals)* | — | §2.1 |
| RMG-OBJ-004 | Seer huts only while +0xF58 is their prototype and +0x10B4 clear, a creature reward (2v − 4000) / 3; a tent only while +0xF5C is its colour. *(QC: another town's reward −1334, not −1)* | — | §2.1 |
| RMG-OBJ-005 | Creating: Pandora's Box its creature and count, experience, gold or its levels' and schools' spells (level downward, no flag 0x2000); prison a hero (none when all banned), experience, serial +0xF44; scroll a spell; seer hut its reward and a quest artifact (type 65), returned as the object. | at creation: 0x54B136, 0x534F06, 0x546154 | §2.1 |
| RMG-OBJ-006 | The prison's hero: the r-th unbanned from the top among 0–127 (RoE) or 0–144, then banned. *(QC: SoD's 145–155 never)* | 0x54B136 %unbanned | §2.1 |
| RMG-OBJ-007 | The scroll's spell: the r-th by id of the level's spells with a school, without flag 0x2000. | 0x534F06 %spells | §2.1 |
| RMG-OBJ-008 | `rmg_prototype_draw`: the r-th candidate prototype; none: null. A mine's resources then none, `rmg_object_choose` skips the generator; the seer huts' creates and `rmg_path_carve` use it unchecked (RMG-QUIRK-4). | 0x546154 %candidates (unsigned) | §2.1 |
| RMG-OBJ-009 | The masks: character 8r + c of a 48-mark mask is the cell c left of, r above the anchor (bit 47 − 8r − c); footprint and colour mask from the picture's `.msk` (1 × 1 without); passable = the passability text or no body pixel; entrances = visitable and not passable; the first entrance from the anchor's row up, each row leftward (none: (8, 6)). Placing (the only way): position; per footprint cell an entrance bit 22 and, unless +0x2C bit 0, 26 cleared, 27 set; else one not passable bit 25 cleared; listed, type counted, the entrance's zone counts it. | — | §2.1 |
| RMG-OBJ-010 | The entrance's distance flood: bits 0–15 of +0x20 on its level, 2 orthogonal, 3 diagonal, over every cell, nearest first, ties FIFO. *(QC: ignores contents)* | — | §2.1 |

Reads: the setup's slots; `rmg.txt`, `rand_trn.txt`, `objects.txt`; hero, creature, spell, dwelling
tables; the reward budget; the limit pairs. Floats: none. Limits: none (loops over rows and tables,
floods to an empty queue); failures: no template (3), silent drops.

#### 3.2.2 The template and the positions

| Id | Rule | Draws (sites, in order) | § |
|---|---|---|---|
| RMG-TPL-007 | The template: the r-th kept, its name kept. | 0x549975 %kept | §1.5 |
| RMG-TPL-008 | Positions: colours humans first, then the others, each in colour order; the first `players` take the human starts' positions in order, then the computers the remaining starts; position p's colour at +0xEE4 + 4p (−1). | — | §1.5 |

Reads: the kept templates, the human flags. Floats, limits: none.

#### 3.2.3 Step 1: the zones laid out

Implemented (E14 T3c; `crates/h3-rmg/src/layout.rs`, the catalog's rows).

| Id | Rule | Draws (sites, in order) | § |
|---|---|---|---|
| RMG-ZON-001 | Per template zone a zone generator: town type +4 the r-th set type (none: −1, no draw); then a start zone with colour towns takes its position's town choice. | 0x532A5B %set types | §2.2 |
| RMG-ZON-002 | Exactly three passes of one placement per zone (creation, then two re-placements in order); integer positions in base-size units; no convergence test. | ZON-005's per zone and pass | §2.2 |
| RMG-ZON-003 | Candidates: first zone (0,0,0), or with two levels if it fits (0,0,1); else 32 points around each connected created zone (none: every created zone), (trunc(r cos k·11.25° + ox), trunc(r sin + oy)), r the bases' sum; two levels: also (ox, oy, 1 − oz) and 32 at max(bases) on the other level. | — | §2.2 |
| RMG-ZON-004 | Fits: a start zone on level 1 needs Inferno, Necropolis or Dungeon; every other zone on the level 10 × trunc(dist) ≥ 8 × bases' sum. | — | §2.2 |
| RMG-ZON-005 | Filter: two levels with an empty level, its candidates (if one at index > 0); most touching connected zones (same level 11s ≥ 10t; across s ≥ t, s − t > smaller base / 2); least max(d, layout box), d = least base × min(w, h) / 5, 6, 7 (no water, normal, islands); the r-th. *(QC: index 0 skipped; empty list divides by 0)* | 0x53BB5F %candidates (unsigned) | §2.2 |
| RMG-ZON-006 | Scaling: span the box's larger side, m = max(w, h), origin (min + max − span) / 2; x, y = (p − origin) × m / span, size = base × m / span, truncating; z kept. | — | §2.2 |
| RMG-ZON-007 | Then per zone, its terrain: the town's native one when matching (grass, grass, snow, lava, dirt, dirt, rough, swamp, grass); else the r-th allowed (subterranean only on level 1; none: dirt); level 1 all but lava → subterranean. | 0x532B00 %allowed (when not native) | §2.2 |
| RMG-ZON-008 | Town type +8: +4 when set, else terrain-towns[terrain][r], r of 4. *(QC: count always 4, −1 and Castle padding drawn)* | 0x53BFC0 %4 (+4 is −1) | §2.2 |

Reads: zones (base, type, town types, terrains), connections, town choices, two terrain tables.
Floats: trunc(f64(r) × T[k] + f64(o)), `rmg_layout_cos`/`_sin` (32 f64, four decimals, 0.9807);
trunc(sqrt) of integer squares; `crt_ftol` truncation. Limits: 3 passes, 32 (65) candidates.

#### 3.2.4 Step 2: each level partitioned

| Id | Rule | Draws (sites, in order) | § |
|---|---|---|---|
| RMG-ZON-009 | The level's centres (data: the zone) and step 2's points (a water zone; none on level 1) into Lischinski's incremental Delaunay triangulation in integers (frame (−200, −200) to (400, 400), diagonal a→c; strict 32-bit RightOf, 64-bit InCircle); a point at a vertex is not inserted; the finish gives each zone triangle its circumcentre (two truncating divisions, A the first record with data in creation order). *(QC: a dropped point)* | — | §2.2 |
| RMG-ZON-010 | Level 1 always, level 0 with water: per original zone 8 directions, p = (trunc(2(s cos) + x), trunc(2(s sin) + y)); off-map refused when x + s cos (y + s sin) is off too; p must fit a temporary zone of base s; p into the triangulation. *(QC: mixed units)* | 0x532A5B of the temporary zone *(QC: uninitialised town types decide it)* | §2.2 |
| RMG-ZON-011 | On level 0, p a water zone (type 3, terrain 8, base s, bands 100–1000/5 and 2000–6000/1, nothing else) appended; on level 1 a point without zone (rock). *(QC: fields uninitialised)* | its generator's 0x532A5B (QC as above) | §2.2 |
| RMG-ZON-012 | Borders, per zone in order: its Voronoi cell's sides in Onext order from the first on the map, each clipped both ways (the line's first point in the box), a→b drawn in its id when the neighbour is none or higher: jagged (limit the smaller size) for the zones present at the start unless islands on level 0, else straight; leaving the map, the map's edge to the next side's entry corner by corner, straight in its id, at once when the two share x or y; no side on the map, the whole frame; the points into +0x3F4. *(QC: the line clip; the shortcut's chords)* | per split, L > 1: 0x53C0F2 %min(limit, L) − half | §2.2 |
| RMG-ZON-013 | Fill after all borders, per zone in order: the 4-connected free region of its centre (off the map: of the clip toward its ring's destination vertex deepest in the map, T3d; a centre not free: its left cell's region, or none): zone index (+0x20 bits 16–23), land bit 28 (not on the islands' level 0). *(QC: no cell)* | — | §2.2 |
| RMG-ZON-014 | Water zones: new triangulation neighbours whose side's clip is on the map connected (value 0, wide, created); hop counts from each earlier zone; each new zone to its earlier neighbours in order unless Z.dist[c] > W.dist[c] + 1 for some other zone c (wide, not created), the counts updated after each. | — | §2.2 |

Reads: step 1's positions and sizes. Floats: 44 sites, s × T[k] as f64, doubled, + f64(x),
truncated; `fcomp` bounds. Limits: 8 directions; lines split to length 1.

#### 3.2.5 Step 3: the terrain painted

| Id | Rule | Draws (sites, in order) | § |
|---|---|---|---|
| RMG-TER-001 | Each zone's box from its cells; its centre moved to their centroid. | — | §2.2 |
| RMG-TER-002 | Islands: each level-0 zone's polygon shrunk by min(L/2, max(L/4, 4)), its coast jagged (limit size / 2), inside filled (bit 28; read T3e). | per split 0x53CE1D %min(size/2, L/2) − half | §2.2 |
| RMG-TER-003 | Level 1 rock (one rectangle), level 0 water, then per land zone in order a painter of its terrain, a one-cell rectangle per cell with bit 28 (box row by row), then its finish. | rock 0x5B3AB6, water 0x5B38D3 per cell; zone cells 0x5B38B9, 0x5B38D3; finish's (§3.2.14) | §2.2, §2.7.5 |

Reads: zone and land bits. Floats: none. Limits: none; allocation failure throws.

#### 3.2.6 Steps 4 and 5: the towns

| Id | Rule | Draws (sites, in order) | § |
|---|---|---|---|
| RMG-OBJ-011 | `rmg_map_can_place`: the footprint on the map, its entrances and blocked cells free, not rock, without an entrance, of the zone; a blocked cell on water only for a group-0 template with water in its fourth column, on land not for one; the outline (a right-hand walk around the anchor row's blocked group) open in one unbroken arc or whole (open: on the map, free, not rock, no entrance, of the zone, water as the zone; an entrance on it refuses unless the type's settings +1 and +2); the cell below the entrance free, not rock, of the zone, without an entrance unless removable, water as the zone. *(QC: an empty outline divides by 0)* | — | §2.2 |
| RMG-TWN-001 | Step 4 per zone: colour castle, colour town, neutral castle, neutral town (if asked, type +4); stops at the first placed. | TWN-002's | §2.2 |
| RMG-TWN-002 | Central town: type −1 refused; the town prototype at the type's index of class 98's vector, at the placeable cells nearest the centre (squared, at most 32000); entrance into +0x30 and the road targets. | 0x545492 %nearest | §2.2 |
| RMG-TWN-003 | Step 5: per kind the least count not yet placed, no spacing, the first of the first kind asked counted as step 4's. *(QC: even when step 4 placed none)* | TWN-005's | §2.2 |
| RMG-TWN-004 | Densities: S, P sum and product of the positive ones; r = trunc(sqrt(82944 / S)); step P / d, key step × least; least key (first on ties) steps and places one at r; exhausted at its first failure. | TWN-005's | §2.2 |
| RMG-TWN-005 | A further town: type redrawn (neutral without same-type byte, or +4 −1) among set types, else 8 (RoE) or 9; no town yet: TWN-002; else entrances in the zone at distance ≥ r, the 3 × 3 in the zone, placeable; the farthest, the r-th. | 0x544DEC %set or 0x544E22 %8/9; 0x545174 %farthest | §2.2 |

Reads: towns, densities, same-type byte, centres, the distance flood. Floats: trunc(sqrt(f64(82944 /
S))). Limits: until every kind has failed once; a town that does not fit is skipped.

#### 3.2.7 Step 6: the obstacle plan and the connections

Cell +0x28 bit 26 = obstacles to place (step 13), bit 27 = kept free; a pending border guard (+0x2C
bit 0) is skipped by both unless placing it.

| Id | Rule | Draws (sites, in order) | § |
|---|---|---|---|
| RMG-CON-001 | Order: lanes, zone walls, cells of no zone, islets, free cells joined, shores sealed, connections. | — | §2.3 |
| RMG-CON-002 | Lanes: object cells free, empty cells obstacles; per level one of four lines (diagonal, vertical middle, anti-diagonal, horizontal middle). | per level 0x543EB8 %4 | §2.3 |
| RMG-CON-003 | Segments (a stack): midpoint ((ax+bx+1)/2, (ay+by+1)/2) moved, L = trunc(length) > 1, by r × (−dy, dx) / L, r = rand % L − L/2; midpoint an end: that end's 3×3 free; L ≥ 8 on the map: two branches ± the normal queued. | per split 0x54403E %L | §2.3 |
| RMG-CON-004 | Branches cast (Bresenham past the target, to the interior's edge or, from step 3, a free 3×3), drawn when ≥ 5 cells; then water and rock free, obstacles' passable empty land neighbours lose bit 27. | CON-003's | §2.3 |
| RMG-CON-005 | Walls: a land zone cell is an obstacle when its 3×3 holds water of no zone, an unconnected zone, a not-wide connection, or it is on level 1; its 3×3 empties lose bit 27. *(QC: level 1 walled even along wide)* | — | §2.3 |
| RMG-CON-006 | Land cells of no zone, passable, empty, not rock: obstacles. | — | §2.3 |
| RMG-CON-007 | Islets per water zone: edge distance (2/3); while a box cell (3..W−4, 3..H−4) is ≥ 20 away, a centre and r = 3 + rand % (d/3 − 5) (≤ 6); after it the distance re-flooded from the centre. *(QC: candidates outside the zone)* | 0x53F745 %cells (unsigned); 0x53F794 %(d/3−5) | §2.3 |
| RMG-CON-008 | An islet: terrain dirt..rough; shape the positive cells of a row-major midpoint-displacement height map: the box split at once (corners and edges 0, centre a / 2, a = (h + w) / 4 + 1, no draw); a split record's edges top, left, bottom, right and centre displaced by rand % a − a/2 while a > 1, its quadrants at (a − 1) / 2 + 1, appended bottom-right, bottom-left, top-right, top-left, popped last first; a 1 × 1 its top-left height clamped to 0..255; painted; the box's land made obstacles. | 0x53EFE4 %6; per split 0x53EEFF…0x53EF43 %a; the painter's | §2.3, §2.7.5 |
| RMG-CON-009 | Joining (also step 9): costs reset; per zone in order the seed is the first box cell in the zone, empty, free, passable, not rock (water only in water zones); none: the last empty cell tried made free. *(QC: stale coordinates with no empty cell)* | — | §2.3 |
| RMG-CON-010 | Cost flood: least first; 8 steps (out of an object without settings +1: E, SE, S, SW, W); objects entered as settings +0, +2 allow; +1 a step, +10 onto water, 0 from cost 0 onto free; into a neighbouring zone +10, upper half, zone and direction back recorded. | — | §2.3 |
| RMG-CON-011 | Carve: each free passable land cell of cost ≠ 0 walks back while cost > 0, cells freed, their 3×3 cleared, a marked Border Guard placed; then re-flooded. | per guard crossed 0x546154 %guards | §2.3 |
| RMG-CON-012 | Seal: a water-zone cell reached by an unconnected land zone's flood, beside passable land: 3×3 obstacles, 5×5 lose bit 27, its water painted that land. | the painter's | §2.3 |
| RMG-CON-013 | Connections: pass 1 per land zone (boat reach cleared), its connections not made: land, else shipyard, else gates; pass 2 per zone with one left (boat reach from its shipyards): shipyard, else monoliths, both ends made. | — | §2.3 |
| RMG-CON-014 | By land: crossings P (cell back Q) empty, n; candidates of least cost (≤ 100); k = min(candidates, (n + 39) / 40) passages, each drawn and carved from P and Q. | per passage 0x54146B %candidates | §2.3 |
| RMG-CON-015 | By shipyard: a prototype drawn; a crossing already reached by boat makes it; else sites x..x+2 per land cell (3 × 2 land, free water beside, land beyond); placed, a road target, row below free, boat reach flooded. *(QC: drawn before any test; sites counted twice)* | 0x541B78 %prototypes; 0x541D6C %sites | §2.3 |
| RMG-CON-016 | By gates (other level, boxes overlap): a prototype drawn; overlap cells fitting both, greatest distance sum; a gate per level, below each free. *(QC: drawn before any site)* | 0x5421BC %prototypes; 0x542387 %positions | §2.3 |
| RMG-CON-017 | By monoliths: an index i from 0, (i + 1) mod (n45 + n43) per connection: two-way prototype i, else one-way entrance and exit prototypes i − n45 (the same channel); entrances at random in each zone (each with its guard, if any), then for one-way an exit in each; no pairing written: the channel pairs them in the game. *(QC: one channel per one-way connection, both exits; channels reused after n45 + n43)* | per object 0x542A8F %positions | §2.3 |
| RMG-CON-018 | Guard value: 0 when wide; (v − low[s]) × fl[s] / 4 above low plus (v − high[s]) × fh[s] / 4 above high; none under 2000; the four tables of §2.3. | — | §2.3 |
| RMG-CON-019 | Border Guard (the flag): this side, tent in the other zone, then the other side, tent here (shipyard: three before its entrance; monolith: after CON-009 again); next Keymaster colour; 3×3 obstacles marked; one placed: g = 0. Fails: monster. *(QC: two guards in a row, keys crossed; a shipyard's tent in the water zone, fitting only on its land; five cells unchecked)* | per tent 0x542A8F %positions | §2.3 |
| RMG-CON-020 | Guard monsters (g > 0, no Border Guard): land one at P or Q; shipyard one; gates and monoliths one per side, each full value. *(QC: counts differ by way)* | land 0x541673 &1; CON-021's | §2.3 |
| RMG-CON-021 | The monster: an allowed town's creatures, AI × (low + high) / 2 ≤ g ≤ AI × 100, the r-th from the top; n = (g + AI/2) / AI, ± rand % (n/4 + 1) when > 1; serial +0xF44. *(QC: RoE counts 0–116, walks from 144)* | 0x540C61 %fitting; 0x540CCE, 0x540CD8 %s | §2.3 |

Within a connection: the way's draws, then per guard the tent's or the side and the monster's.
Reads: connections (value, wide, guard flag), object settings +0..+2, creature table, guard
tables, `rmg_side_steps`. Floats: none (lengths by `vector_length_int`, trunc(sqrt)). Limits: none
capped; a connection with no way left silently; faults only with other data (modulus 0).

#### 3.2.8 Step 7: the junction zones

| Id | Rule | Draws (sites, in order) | § |
|---|---|---|---|
| RMG-OBS-001 | Per junction zone not of water: its cells' costs reset, empty ones without a pending guard get bit 26 (the zone starts full). | — | §2.4 |
| RMG-OBS-002 | A flood from its first passage point (+0x404); per later point of cost 1..30000, back to the first cost-0 cell, a jagged path (CON-003 with m = min(L, size)) freeing cells and clearing their 3×3's bit 26; re-flooded. | per split 0x5444B4 %m | §2.4 |

Reads: step 6's passage points. Floats: lengths. Limits: one pass per point.

#### 3.2.9 Steps 8 and 9: the mines

| Id | Rule | Draws (sites, in order) | § |
|---|---|---|---|
| RMG-MIN-001 | Per zone, resources 0–6: least mines at spacing 0 until a failure; a start zone's first wood and ore near its town. | MIN-003's | §2.4 |
| RMG-MIN-002 | Then by density as TWN-004 (82944 / S). | MIN-003's | §2.4 |
| RMG-MIN-003 | A mine: prototypes (53) of the resource allowed on the terrain, else all; one drawn; site, guard, resources. | 0x545AA8 %prototypes | §2.4 |
| RMG-MIN-004 | Site: box row by row, placeable; near the town entrances 4..200 away (squared ≤ 144 as 144), the least; skipped below the best distance so far (first the spacing) or best obstacle-outline count (≤ 5); a better one empties the list; cell below the entrance freed. *(QC: running best)* | 0x54591A %kept | §2.4 |
| RMG-MIN-005 | Guard: value 1500 (wood, ore), 7000 (gold), 3500, index zone + map strength − 3 (0..5); none at strength 0 or < 2000; monster below the entrance. *(QC: index differs from connections')* | CON-021's | §2.4 |
| RMG-MIN-006 | Resources: a prototype (79) drawn; per cell of the row below, x − width..x + 1, while ≤ 2 placed, a coin: 0 places one. | 0x546154; per cell 0x545D0C &1 | §2.4 |
| RMG-CON-022 | Then the towns counted; step 9 runs CON-009–011 once more. | CON-011's | §1.5, §2.3 |

Reads: least mines, densities, strengths, town entrances. Floats: trunc(sqrt(f64(82944 / S))).
Limits: each resource to its first failure; a mine without a site dropped.

#### 3.2.10 Step 10: the treasures

| Id | Rule | Draws (sites, in order) | § |
|---|---|---|---|
| RMG-TRS-001 | Per zone (water too): bands with high ≥ 100 and density > 0; r = trunc(sqrt(800 / S)) (1600 water); keys as TWN-004 from 0; each pick tries a pile 3× without, 3× with the density filter; built: placed, else released; a band ends when six tries fail. Draws of failed tries stand. | TRS-002–008's | §2.4 |
| RMG-TRS-002 | Pile target t = low + rand % (high − low) (high when high ≤ low), on a 16 × 16 map. | 0x546771 %(high−low) | §2.4 |
| RMG-TRS-003 | Fill: first object t/4..t at the centre (3 tries); while short and the rest ≥ 1500 or ≥ half, rest/4..5rest/4 beside an earlier entrance (8 neighbours, or SE, S, SW), off the edges; 3 null or 3 failed end it. | per object TRS-004, -005; then 0x535B31 %positions | §2.4 |
| RMG-TRS-004 | Candidates in generator order: map and zone counts under the limits, value in range, a prototype for type, subtype, terrain, footprint free; density filter ≥ 3/4 of the best (> 4/3 resets). *(QC: a draw per candidate; running best)* | per candidate 0x546154 | §2.4 |
| RMG-TRS-005 | Weighted by density; created with the drawn prototype (OBJ-005). Constant test (no huts, tents) only for TRS-009/010's replacements. | 0x546489 %densities; OBJ-005's | §2.4 |
| RMG-TRS-006 | Guard: none at zone strength 0; value of the total (MIN-005's index) ≥ 2000: a monster at an outline cell before an entrance, neighbours freed; no monster: unguarded; no cell: refused. *(QC: unguarded)* | CON-021's; 0x5353B9 %cells | §2.4 |
| RMG-TRS-007 | Placing: offsets keeping the box in the zone's; centre cell in the zone at distance ≥ r; the fit (§2.4); the farthest, ties drawn. *(QC: box may pass the zone's by one)* | 0x5472F0 %farthest | §2.4 |
| RMG-TRS-008 | Commit: objects placed, obstacles under the pile's bit-26 cells (only through the guard), bit 27 off under its non-free empties; each object's placed hook in order. | the hooks' | §2.4 |
| RMG-TRS-009 | A tent: its colour's Border Guard, next colour; a pile of 1.5 × its value behind it in a near zone; fails: tent removed, an object v..1.5v there. Chains. *(QC: removed, not deleted: template still written)* | fill, guard cell, TRS-011's, offset | §2.4 |
| RMG-TRS-010 | A quest artifact: treasure artifacts 0–143 free (< 20 left blocks huts), one drawn; the hut alone in a near zone; +0xF58 to the next seer prototype; fails: replaced, hut deleted. *(QC: 0–143 every version)* | 0x54B511 %free; TRS-011's; offset | §2.4 |
| RMG-TRS-011 | Near zones: keys 10 × hops + rand % 10, a neighbour 1000 + rand % 10, drawn per zone in order; not this, junctions, water; ≤ 2000; tried by key. *(QC: neighbours last)* | per zone 0x54B366 or 0x54B382 %10 | §2.4 |
| RMG-TRS-012 | A released prison frees its hero. | — | §2.4 |

Reads: bands, generators and counts, limits, artifact class and disabled, settings +0..+2. Floats:
trunc(sqrt(f64(800 or 1600 / S))). Limits: six tries per pick, three per object; no cap on the loop;
give-ups silent.

#### 3.2.11 Steps 11 to 13: rock, shores, obstacles

| Id | Rule | Draws (sites, in order) | § |
|---|---|---|---|
| RMG-TER-004 | Step 11 (two levels): level 1's passable empty cells not kept free painted rock; level-1 zones' rock cells kept free or holding an object get their terrain back. *(QC: obstacles become rock)* | the painter's rectangles, flushes, finish | §2.4, §2.7.5 |
| RMG-OBS-003 | Step 12: land beside water gets bit 21. | — | §2.4 |
| RMG-OBS-004 | Step 13: per bit-26 cell (level, row, column), passable, not rock, a fill (a stack); then every cell not kept free, not rock, passable, unguarded made free. | OBS-007's | §2.4 |
| RMG-OBS-005 | Candidates per popped cell: 45 obstacle types in order, prototypes whose row allows the land (> −5000; ≥ 165 refused in RoE), footprints over it with a blocking cell, positive score. *(QC: SoD test refuses none)* | — | §2.4 |
| RMG-OBS-006 | Score: −5000 for a blocking cell on a forbidden land or kept-free cell; else modifiers summed (< −1000 returned; −1 if none positive); + Overlap / Adjacent per object near (−5000 without row); −5000 when depths interleave. Depth grid values not read. | — | §2.4 |
| RMG-OBS-007 | One by score weight, placed; bit-26 cells under its widened footprint cleared and pushed. | per cell 0x5376AA %scores | §2.4 |

Reads: `rand_trn.txt`, the obstacle types. Floats: none. Limits: the stack until empty.

#### 3.2.12 Steps 14 and 15: roads and rivers

| Id | Rule | Draws (sites, in order) | § |
|---|---|---|---|
| RMG-RDR-001 | One road type, rand % 3 + 1 (dirt, gravel, cobblestone). | 0x54829B %3 | §2.5 |
| RMG-RDR-002 | Per road target i but the last (towns, then shipyards): costs reset, a flood over the map, ties FIFO; 2 road to road else 20, ×3 diagonal; refuses water, rock, bit 25 clear, blocking objects; settings +1, +2 clear: left by five neighbours only, entered not SE, S, SW. | — | §2.5 |
| RMG-RDR-003 | Teleports: one-way monoliths reach every one-way of their subtype at +50, two-way their own list, gates the other level at +1. *(QC: one-way used both ways)* | — | §2.5 |
| RMG-RDR-004 | Per later target of cost ≤ 30000, traced back (its end's stretch of the type skipped) to the painter, a new stroke after each diagonal step or level change; painted: i re-flooded. | RDR-011's | §2.5 |
| RMG-RDR-005 | Sources bit 29 on lakes (126), mountains (134), Gem Ponds (53/5): entrance, else half the footprint up-left. | — | §2.5 |
| RMG-RDR-006 | Ends bit 30 on the border and delta sites (3 water across, 3 land beyond, 4 land in d, no objects: the fourth, d in bits 8–11). *(QC: `x > width` bound reads past the cells)* | — | §2.5 |
| RMG-RDR-007 | Per Water Wheel (109) in order: a course up from its entrance, one down from three cells beside it. | RDR-008's | §2.5 |
| RMG-RDR-008 | Courses: least cost over E, S, W, N; refuse water, rock, the other snowness (icy 2, clear 1); no object test; cost (rand & 31) + 1 per neighbour examined, +30 into road; stop at the first improved target-bit cell, paint back. *(QC: under objects; draws per examination)* | per neighbour 0x54876F, then 0x549153 &31; painter's | §2.5 |
| RMG-RDR-009 | Lower course: not beside an earlier river (bit 31), deltas entered along d; a delta (143, the direction's prototype, none: none) at site + offset, one cell more; its cells get bit 30, 5×5 around cleared. | — | §2.5 |

Reads: road targets, monolith lists, gates, settings, terrain, objects. Floats: none. Limits: none;
unreachable targets and courses get nothing.

#### 3.2.13 The writer

| Id | Rule | Draws (sites, in order) | § |
|---|---|---|---|
| RMG-WRT-001 | Gzip (`wb6`); header: format by version, width, levels, "Random Map", description, difficulty 1; nothing but the seed depends on time or machine. | — | §1.2, §2.5 |
| RMG-WRT-002 | Per colour: bools from its town zones (human start: both; computer: computer), extra human starts computer-only (pink first); allowed towns; main town at the first town's entrance, type 0xFF; else the unused bytes. *(QC: position the entrance; HD mod writes the type)* | — | §2.5 |
| RMG-WRT-003 | Standard victory and loss; teams; allowed heroes (16 bytes of 128 RoE, 20 of 156); AB and SoD placeholders; 31 zeros; banned artifacts (combination + 127, 128; AB 17 bytes, SoD 18); SoD spell and skill zeros; 156 zeros; u32 0 (rumours). *(QC: 127, 128 banned)* | — | §2.5, §1.6 |
| RMG-WRT-004 | Teams: counts 0 become the colours (with no computers ≥ 2); both ≥ their colours: none; else clamped, players from team 0, computers after; per flagged colour a team with room. | per colour 0x54ABA6 %teams with room | §2.5 |
| RMG-WRT-005 | Cells level, row, column: 7 bytes (land, picture, river, picture, road, picture, mirrors + shore 0x40). | — | §2.5 |
| RMG-WRT-006 | Templates: count + 2, groups 71, 124 first, then every prototype in use by settings group (repacked masks, terrain masks, class, bytes). | — | §2.5 |
| RMG-WRT-007 | Objects: count; those whose settings +0xC is set first (which types: not read), then the rest, in placing order; x, y, z, template, 5 zeros, body. | — | §2.5 |
| RMG-WRT-008 | Bodies: monster (count, disposition +0x24), town (owner, fort), dwellings, artifacts, resource, Pandora, seer hut (RoE artifact; AB quest), prison, Scholar, shrine, scroll, Witch Hut 0xEFDF; plain: header only. *(QC: Witch Hut skills)* | — | §2.5 |
| RMG-WRT-009 | Ends with the events' u32 0; no 124 padding bytes. | — | §2.5 |
| RMG-WRT-010 | The description: template, seed, width, levels, "humans" (+0xF48), computers, water, strength, version; per colour human and town choice by the colour's index. *(QC: wrong town name; humans counts computers)* | — | §2.5 |

Reads: everything placed; the plain executable's writer (not `Untitled.h3m`'s HD mod). Floats:
none. Limits: none; false only when a write fails (code 2).

#### 3.2.14 The painters (reached from steps 3, 6, 11, 14, 15)

| Id | Rule | Draws (sites, in order) | § |
|---|---|---|---|
| RMG-TER-005 | Painter (map, terrain, weight 4), rectangles, finish: set 1's smallest (y, x) through TER-014, then set 2's erased, pinched ones absorbed, repeated; last pass once; flush the same, then a new terrain. Sets ordered by (y, x), nothing by address. | TER-007's | §2.7.1 |
| RMG-TER-006 | Picture sets per terrain (bytes +4, +5, flagged share, shapes; rock by variants a, b). | — | §2.7.1 |
| RMG-TER-007 | Slot 4: shape 0 kept unless called with −1; flagged non-empty: rand % 100 < share × w / 8 picks it; then rand % count + first (two draws on land, one on water; rock one). | 0x5B38B9 %100, 0x5B38D3 %count; rock 0x5B3AB6 | §2.7.1 |
| RMG-TER-008 | Slot 5: kept when of the shape (rock: and variants), else rand % count + first of the shape. | 0x5B3912; rock 0x5B3B2D %count | §2.7.1 |
| RMG-TER-009 | Rectangle row-major, unclamped: other terrain set_cell, same terrain a new picture; every cell drawn once. *(QC: repainting redraws)* | per cell TER-007's | §2.7.2 |
| RMG-TER-010 | Weight: 4 halved per W, N, E, S neighbour of the terrain with a flagged picture, as they are (W, N redone). | — | §2.7.2 |
| RMG-TER-011 | Tests: pinched W-E, N-S (both neighbours another terrain, not at the edge); ring split (water, rock: own-terrain neighbours in two arcs). | — | §2.7.2 |
| RMG-TER-012 | set_cell: draw, write, out of set 2; land: neighbours' broken pinches erased and queued; water/rock: 8 neighbours re-tested; itself pinched to set 1, else queued. *(QC: never leaves set 1 itself)* | TER-007's | §2.7.2 |
| RMG-TER-013 | Queue: first of N, S of another terrain, then of W, E; diagonals for water and rock. *(QC: else-if per axis)* | — | §2.7.2 |
| RMG-TER-014 | Resolve: N-S pinch gives N if pinched, else S if pinched, else N at the edge, else N if NW/NE, S if SW/SE, else N; then W-E alike; then the ring's gaps (2 orthogonal, 1 diagonal) cheapest first, the costliest kept. *(QC: own diagonal redrawn)* | per set_cell TER-007's | §2.7.3 |
| RMG-TER-015 | Last pass, whole map row-major: neighbour classes (clamped): 0 same or p sand; 1 both through dirt, p not dirt; else 2. *(QC: whole map)* | — | §2.7.3 |
| RMG-TER-016 | Shape: §2.7.3's ten groups, mirrorings none, N-S, W-E, both; first match; none 0. *(QC: group 3 inverted)* | — | §2.7.3 |
| RMG-TER-017 | Refinements 2 → 6, 8 → 12 (arm cells same), 5 → 7, 11 → 13 (two steps differ). | — | §2.7.3 |
| RMG-TER-018 | Picture: shape ≠ 0 slot 5, 0 slot 4; written only when changed; terrain never. *(QC: missing shape divides by 0)* | TER-007/008's | §2.7.3 |
| RMG-RDR-010 | Stroke: visit the start; stroke_to(t) walks from t back, longer axis, staircase corners. River walks' calls not traced. | — | §2.7.4 |
| RMG-RDR-011 | Visit: nothing on the type or water, rock; another type cleared (neighbours redrawn); set, drawn, 8 same-type neighbours redrawn N..NW. *(QC: clear's right column short)* | RDR-012's | §2.7.4 |
| RMG-RDR-012 | Picture: shape and flags from the 8 neighbours; kept when fitting, else rand % count + first. *(QC: unmirrored 4 not drawn)* | 0x4FA00F %count | §2.7.4 |
| RMG-RDR-013 | Shape: all four 8; N+S 6 (with E, W mirrored) else 2; E+W 7 (S, N mirrored) else 3; a corner 5 on a diagonal run, else 4; ends 1, 0 (rivers 3, 2). | — | §2.7.4 |
| RMG-RDR-014 | Sets: roads 17 pictures, rivers 13 (no 0, 1, 5). | — | §2.7.4 |

Reads: the map interface, the views, the picture tables. Floats: none. Limits: none; the cell words'
fill is memory-dependent but never read (harmless).

#### 3.2.15 Dependencies, oracle points, not read

Each step reads what the earlier ones leave: step 1 the template and positions; step 2 step 1's
scaled zones; step 3 step 2's zone and land bits; steps 4–5 step 3's centroids and the distance
flood; step 6 the towns, zones and connections (water zones' from step 2); step 7 step 6's passage
points and floods; step 8 the towns' entrances and bit 26; step 9 the mines; step 10 the generators'
hooks (+0xF58, +0xF5C, +0x10B4) and connection hops; step 11 bit 27 and objects; step 13 bits 26,
27 and the terrain; step 14 the road targets (steps 4–6), monoliths and gates; step 15 the objects
(wheels from step 10) and roads; the writer everything, then the teams' draws. The phase oracle
(ORACLE.md, "The phase oracle") logs every draw with its site and dumps the cells at each step's
first call and at `rmg_generate_map`'s return: the port compares its draws and cells after every
step (steps 7 and 11 have no dump when they do not run; steps 9 and 12 drew nothing in the corpus),
then the file with the plain executable's. Memory-dependent: ZON-010/011's town-type draw,
RDR-006's past-the-end read. Not read: the pile ring (§2.4, increment 5); depth grid (§2.4,
OBS-006); settings +0xC types (§2.5, WRT-007); river strokes (§2.7.4, RDR-010). Read in T3b: step
2's triangulation, borders, clip, fill and water connections (§2.2), code 1 (§1.2). Read in T4a
(increment 4's, exit (e)): the objects' masks and the null prototype's callers (§2.1, OBJ-009,
OBJ-008), `rmg_map_can_place`'s footprint, outline and ring with tents on water (§2.2, OBJ-011),
the islets' height map (§2.3, CON-008), the one-way monoliths' pairing (§2.3, CON-017).

### 3.3 The objects placed, the reader and the writer

(T1c2.) What the generator puts on a map against what `h3_map` reads, imports and writes. Sources:
§2.1 to §2.5 (the code read); the oracle's 21 maps (`h3rmg.py corpus` and `read` in this sandbox)
and a census of them by a scratch program over `h3_map::read` (not in the repo, no map kept); the
160 shipped maps for the main town; one observation of the original's start (h3mem). `h3-map` as
of 25397e78a.

#### 3.3.1 What the generator can place

A generator's *type* is its prototypes' object-settings group (§2.1; `object_class_values`
0x63A6E4), so a prototype of another class in that group comes in under it: in the corpus 199
(swamp trees, loaded as an obstacle of 143-161) and 221 (the snow Trading Post, as 99). The class
written is the prototype's own (`rmg_write_template`). Corpus: objects / maps of 21.

| Class | Subclasses | Placed by | Written by (slot 3) | Corpus |
|---|---|---|---|---|
| 98 Town | faction 0-8 | steps 4, 5 | `rmg_town_object_write` 0x5332F0 | 212; 87 owned, 137 with a fort |
| 54 Monster | creature | steps 6 (passages, shipyards, gates, monoliths), 8 (mines), 10 (piles): `rmg_guard_monster_create` | `rmg_monster_object_write` 0x5331F0 | 3 156 / 21, 135 creatures |
| 53 Mine | resource 0-6 | step 8 | `rmg_dwelling_object_write` 0x533460 | 757 / 21 |
| 79 Resource | 0-6 | step 8 (beside a mine); generator 21 | `rmg_resource_object_write` 0x5335C0 | 3 062 / 21 |
| 87 Shipyard | 0 | step 6 (`rmg_connection_by_shipyard`) | dwelling write | 61 / 7 |
| 103 Subterranean Gate; 43, 44, 45 Monoliths | 0; channel | step 6 (`_by_gate`, `_by_monoliths`) | `rmg_object_write` 0x533170 (header alone) | 170 / 8; 32, 32, 124 |
| 9 Border Guard; 10 Keymaster's Tent | colour 0-7 | step 6 (`rmg_border_guard_place`); step 10 (generator 7, the tent's placed hook) | header alone | none: no corpus template has a border-guard connection (3 of SoD's 53: Gauntlet, Ready or Not, Worlds at War) |
| 5 Artifact (quest), 83 Seer's Hut | artifact; 0-2 | step 10: generator 23, the artifact's placed hook (`rmg_seer_hut_place`) | `rmg_artifact_object_write` 0x533500; `rmg_seer_object_write` 0x533A90 | 115 and 115 / 18 |
| 62 Prison | 0 | generator 16 (`rmg_hero_draw`) | `rmg_prison_object_write` 0x533C80 | 77 / 17 |
| 6 Pandora's Box | 0 | generators 3-6 | `rmg_pandora_object_write` 0x5336F0 | 221 / 19: 51 creature, 118 experience, 21 gold, 31 spells |
| 17 Dwelling | 0-79 (RoE 0-57) | generator 14 | dwelling write | 244, 60 subclasses |
| 66-69 Random artifacts | 0 | generator 18 | artifact write | 548 |
| 76 Random resource | 0 | generator 19 | resource write | 1 232 / 20 |
| 81 Scholar; 88-90 Shrines; 93 Spell Scroll | 0 | generators 22; 25; 27 (spell drawn) | `rmg_scholar_`, `rmg_shrine_`, `rmg_scroll_object_write` | 66; 516; 351 |
| 113 Witch Hut | 0 | generator 29 | `rmg_witch_hut_object_write` 0x5340C0 | 35 / 12 |
| The bodiless visitables: 2, 4, 7, 11-14, 16, 22-25, 28-32, 35, 38, 39, 41, 47-49, 51, 52, 55-58, 60, 61, 63, 64, 78, 80, 82, 84-86, 92, 94-97, 99-102, 104-110, 112 | 13: 0-2; 16: 0-6; else 0 | generators 1, 2, 8-13, 15, 17, 20, 22, 24, 26, 28 | header alone | 3 471; 56 classes seen (not 24, 56) |
| 143 River delta | 0 | step 15 | header alone | 18 / 7 |
| Obstacles (lakes 126, mountains 134 among them) | 0 | step 13 (`rmg_obstacles_fill`: `rmg_obstacle_types`, `rand_trn.txt`) | header alone | 32 843 (69 %), 37 classes |
| 71 Random monster, 124 Hole | first prototype | none: the template table's entries 0 and 1 (`rmg_write_map`) | template only | 21 / 21, no object |

**Order and identifiers** (the code read, the corpus agreeing): first the objects whose type's
settings +0xC is set (in the corpus exactly the obstacles and deltas: 116-161, 199, 206-211, 143),
then the rest, each pass in placing order **across both levels** (34 to 162 level changes in a
two-level map). Towns', monsters' and prisons' identifiers are the generator's serial +0xF44
(`s1-2p-dry`'s towns 1 to 4), never the written index + 1 (3 445 of 3 445). The template table:
those two entries, then the prototypes in use by settings group: not in first-use order (21 of 21),
no two equal.

#### 3.3.2 The census

`setup.sh`, then `corpus`: 20 cases as kept; `m2-4p` **CHANGED** in the corpus run (113 282 bytes,
its towns elsewhere, the header differing from byte 235) and as kept run alone right after: a first
map of a fresh start still varies here (RMG-QUIRK-1's stack bytes the likely cause; not followed).
`read`: 21 of 21 read, 20 import and validate, `l2-2p-c4-islands` refused (§3.3.4). **Headline:**
47 458 objects of 115 classes (389 class and subclass pairs): obstacles 69 %, resources 4 294,
monsters 3 156, mines 757, towns 212, prisons 77; no Border Guard, Keymaster's Tent, hero, event
or random monster object.

As read (every map unless counted):

- **Header:** format 0x1C, difficulty 1, level limit 0, standard victory and loss; the allowed
  heroes the editor's default (0xEF, 0xFF … , 0x01, 0x00: 4 and 145-155 out) less each prison's
  hero (21 of 21 exact); **banned artifacts 127-140** (bytes 15-17 `80 FF 1F`); no spell or skill
  banned, no rumours, custom heroes or timed events, no 124 trailing bytes; gzip mtime 0, OS 0x0B.
- **Players:** a colour in play: tactic 0, choice 0, random town 0, main town with generate-hero 1
  and **type byte 0xFF**, no random hero, hero type 0xFF, the unknown byte 0, no heroes (88 of 88);
  an unused colour's 13 bytes 7 × 0, 0xFF, 5 × 0 (80).
- **Bodies:** monsters a count, disposition 3, no treasure, may flee, grow; towns no name, garrison
  or building list, formation 0, spells zero, no events, alignment 0xFF, 125 neutral; prisons owner,
  patrol and sex 0xFF, nothing customised, 15 without experience; seer huts SoD's one-artifact
  quest, deadline −1, no texts, rewards experience 37, creatures 42, gold 36; one reward kind per
  Pandora's Box; Witch Huts `DF EF 00 00`; shrines spell 0xFF, Scholars bonus −1, resources and
  random resources amount 0, artifacts no message, mines, dwellings and shipyards owner 0xFF.

#### 3.3.3 The engine's coverage

`h3_map::read` reads every class (21 of 21). The import (`import/codes.rs`, `object_type`) types a
class or leaves it a blocker (a footprint without a type, the import's default). The writer: its
typed `MapObject`s, `PLAIN_OBJECTS`, or `Extras::objects` (`AnyObject`: any template but 34, 62,
70, 214, with the editor's default body; SoD only).

| Classes | Import | Writer |
|---|---|---|
| 98 | typed (main town: §3.3.4) | `AnyTown` |
| 54 | typed | `Monster` (count, disposition, flees, growth) |
| 53, 17, 87 | typed by subclass | `OwnedObject` (no owner: 0xFF) |
| 79; 76 | typed | `Pile` (79 only); 76 by `Extras` (amount 0) |
| 5, 66-69; 6 | typed | `RewardBody::Artifact`; `Pandora` (the model's `Rewards`) |
| 62 | typed (prisoner) | `Prison` (its hero out of the allowed set, as the generator) |
| 88-90, 93, 113 | typed | `RewardBody::Shrine`, `Scroll`, `WitchHut(Some(mask))` |
| 81 | typed | `Extras` (its default body is the generator's) |
| 83 | **blocker** (quests: E12) | `Extras::quests`: the mission, no reward |
| 9, 10 | typed by colour | `Extras` |
| 57 Obelisk, 80 Sanctuary | **blockers** (E12) | `Extras` |
| 43-45, 103, the other bodiless visitables | typed | `Plain` or `Extras` |
| 143, obstacles | blockers (scenery) | `Plain` for 135, 137, 155; `Extras` |

Nothing of the corpus is refused or stubbed (no `Unsupported`, no `docs/stubs.toml` entry); 115
seer huts (18 maps), 278 obelisks (20) and 5 sanctuaries play as blockers until E12.

**The writer's gaps** for increment 6 (the uncompressed `.h3m` byte for byte; the gzip stream
cannot match: the game's zlib at level 6, OS 0x0B, against `writer::gzip`'s miniz at 9, OS 0xFF):

1. **One object list in the caller's order:** `write_all` writes the typed surface objects, the
   underground's, then `Extras`; the generator interleaves levels and kinds.
2. **Identifiers as given** (towns, monsters, prisons): the writer's are the written index + 1.
3. **The template table as given:** the generator's order and its two unused entries (71, 124); the
   writer's `templates` lists the objects' in first-use order.
4. **A Seer's Hut's reward** (experience, one creature stack or gold) with its one-artifact quest;
   `Extras::quests` writes none, and only in SoD.
5. **The banned artifacts as a set:** the generator bans 127-140; the writer ORs
   `Map::banned_artifacts` into the editor's 127-143.
6. **RoE and AB layouts** (the generator writes its version's 0x0E, 0x15 or 0x1C): the writer has
   `Format::RoE` (no towns, no `Extras`) and SoD, no AB; the corpus is SoD only, so oracle cases
   first.
7. **The main town** (the town lane's `writer/town.rs`): the type byte 0xFF where the writer writes
   the faction; a main town of a town not of its colour (`main_towns` takes the colour's own only).

Not gaps: the header's other fields, the unused colours' bytes, the allowed heroes, the tiles
(`GivenTile`, flags as given), the other bodies, the description (Latin-1, a template name's line
break), the absent padding. Each gap is one inbox line for the adventure lane (7: the town lane).

#### 3.3.4 TWN-SET-001 and `l2-2p-c4-islands`

- **The original writes** each colour's main town at its zone's first town's entrance (zone
  generator +0x30, §2.5): the town's anchor less two columns, 88 of 88 in the corpus;
  `Untitled.h3m`'s "two cells right" is exactly this. The editor's shipped maps write the same: 553
  of 553 main towns at the anchor less two of a town of that colour (160 maps).
- **The reader expects** the same (`docs/formats/h3m.md`, the player block; `import::town::starts`:
  the town one of whose entrances is the position, else `ImportError::MainTown`, TWN-QUIRK-4).
  §2.5's candidate is no quirk: the entrance is the format's convention.
- **Why only this map:** Dragons' zone 32 is a computer start of colour 5 (purple) that asks for no
  colour town or castle (columns 14-17 zero) and for neutral towns by density (18-21: 0, 0, 6, 6).
  Its first town, a neutral Inferno placed by step 5 (`rmg_town_place_far` through `_central`,
  which sets +0x30), becomes purple's main town; purple plays (its zone holds a town). The import
  finds the town; `GameSetup::validate` refuses it (`GameSetupError::MainTown`, "not a town of its
  own": the owner half the user's cut of 2026-11-04 added, E09 design §8, an engine decision
  without an original source). The corpus's other 87 main towns are towns of their colour.
- **The original accepts it** (observed after the start settled, the same bytes as kept): the town
  (entrance (9, 39, 0)) stays neutral, purple's hero (Marius, 50, of purple's one faction, Inferno)
  stands on its entrance as its visiting hero, purple owns no town. `main_town_heroes` 0x4CA040
  matches the town by entrance alone, no owner test; nothing read gives the town to the colour. The
  seven days without a town that follow (E10, MCL-STUB-4) were not played.

**Decided** (the user, 2026-11-24, T1c2): the entrance convention is settled (not a quirk);
the neutral main town is **RMG-QUIRK-23**, no flag (the original's map in every profile); the town
lane's inbox holds the question whether `GameSetup::validate` accepts a main town of another owner
as the original does (the hero generated on its entrance as the town's visitor, the town keeping its
owner, the colour starting without a town). Until then a game on such a generated map is refused
(1 of the 21).
