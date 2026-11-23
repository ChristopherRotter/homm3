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
limits by object type (`rmg_type_limit_map` 0x69CE4C, `rmg_type_limit_zone` 0x69D1F4) to 32000 and
then from the pairs at 0x640718 (30 types: 32, 48, 64 or 200) and 0x640808 (24 types: 1; type
113: 3), and clears the human flags. No other draw happens before `rmg_generate_map`.

### 1.4 The files read

| File | Reader | What is kept |
|---|---|---|
| `objects.txt` | `rmg_objects_read` 0x536200 (`template_file_read` 0x514D80) | the templates at +0x24, 0x4C bytes each (the stride of the division here; `template_file_read`'s entry says 0x44, from its other caller); a 0xE8-byte prototype for each with type 0..231 the version allows (RoE below 165, AB below 222, before SoD types 43 to 45 only subtypes 0 to 2), in the vector of its type's object settings +8; the monsters' vector (54) sorted by subtype with an exchange sort that swaps only when greater |
| `rand_trn.txt` | `rmg_rand_trn_read` 0x536560 | the obstacles, each prototype's row (§2.1) |
| `rmg.txt` | `rmg_templates_read` 0x537FF0 | from row 3, a template per row of two or more columns (name, min and max size) up to the next; kept when the range holds the map's size (width × height × levels / 1296; islands: halved, at least 1) and its zones fit the players, computers and version; connections from column 76 (read in part) |
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
with it as 9 dwords; the code indexes position + 1); the object generators go to H3API's
`objectGenerators` +0x10F0 (T1a1 read +0x1100; §2.1); the cell's +0x20 is H3API's zone data.

### 1.8 Draws and floats so far; what T1a1 did not read

- **Draws, in stream order:** the dialog's Random choices (§1.1, the game's state); `srand(time)`;
  water's Random; the template; the 15 steps (44 sites in their closures, §1.5); the teams in the
  header. One generator, one thread, one stream.
- **Floats:** x87 in steps 1, 2, 5, 8, 10 (`crt_sqrt`, `crt_ftol`); T1a2 lists each site.
- **Not read:** the 15 steps' bodies (but steps 4, 8 and 15's own code, step 1's in part; steps 1 to 5: §2.2), the header before
  its end, the cell and template writers, the per-object writers (the setup's readers and the
  destructor: §2.1). The limits by
  type are read only by step 10's object choice: one per map, one per zone (§2.4).
- **For T1b:** a generated map names its template and seed in its description, and the
  original's choices are reproducible only without Random ones (§1.1); the oracle can read the
  seed back from the file to check a fixed clock.

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
`rmg_prototype_draw` returns null without candidates (how the callers treat a null was not read);
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

**Quirk candidates (as read, unclassified):**

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

**Not read:** the growth code of the four `inferred` library inserts; how `template_file_read`
fills the masks +0x4 and +0xC (H3API's passable and enterable; the flags they set fit H3API's bit
names only loosely); who calls the map vtable's slots 1-6 and the generators' constant test; who
sets +0xF58, +0xF5C and +0x10B4 and reads +0x1100, a prototype's +4 and +0x10, the obstacles
and the distances; the 13 object vtables' methods (§2.5).

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
| 0x53C390 `rmg_zone_borders_draw` | 0x53E050 | a zone's polygon clipped, its borders | inferred |
| 0x53CAC0 `rmg_segment_clip` | 0x53C390, 0x53D380, 0x53DAD0 | a segment's first point on the map | inferred |
| 0x53C220 `rmg_line_draw`, 0x53BFF0 `rmg_jagged_line_draw` | 0x53C390 | straight and randomly displaced borders (0x53C0F2) | read |
| 0x53D380 `rmg_zone_flood_fill` (tail 0x53D640) | 0x53E050 | a zone's cells from its centre | inferred |
| 0x53DAD0 `rmg_water_zones_connect`, 0x53D8E0 `rmg_zone_graph_distances` | 0x53E050; 0x53DAD0 | the new zones' connections; the zone graph's distances | inferred |
| 0x54CBA0 `point_pair_vector_insert`, 0x54CD70 `point_pair_vector_erase`, 0x54C1E0 `u16_vector_insert`, 0x54C3D0 `u16_vector_size` | the line and distance code | the library's vectors | inferred; read; inferred; read |
| 0x53E6A0 `rmg_terrain_paint` | step 3 | boxes, centroids, islands, the painting (below) | read |
| 0x53BBB0 `rmg_zone_bounds_compute`, 0x53D0D0 `rmg_zone_centre_to_centroid` | 0x53E6A0 | the zones' boxes; their centres moved to their cells' centroid | read |
| 0x53D1C0 `rmg_island_shape`, 0x53CD30 `rmg_coast_line_draw`, 0x53CF50 `rmg_island_fill` | 0x53E6A0 (islands) | an island: polygon shrunk, coast (0x53CE1D), inside | read; read; inferred |
| 0x544A50 `rmg_zone_first_town` | step 4 | at most one town (below) | read |
| 0x545250 `rmg_town_place_central` | 0x544A50, 0x544D90 | a zone's first town, nearest its centre (0x545492) | read |
| 0x531CF0 `rmg_map_can_place` | 0x545250, 0x544D90; five functions of later steps | whether a prototype fits at a cell in a zone | read |
| 0x5318B0 `rmg_map_footprint_blocked`, 0x532C80 `rmg_prototype_outline`, 0x531170 `rmg_map_outline_clear` | 0x531CF0 (and functions of later steps) | its footprint, outline and ring tests | inferred |
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

**Step 2, `rmg_level_zones_partition`, per level z.** The level's zone centres go into a
triangulation (`triangulation_init` 0x5FD010: integer, a frame with corners at -200 and 400;
`triangulation_insert` 0x5FD790 with the zone generator as data). On level 1 always, on level 0
only with water: for each zone present at the start, 8 directions (every fourth entry of the
tables): p = (trunc(2 × (s × cos) + x), trunc(2 × (s × sin) + y)), s its scaled size; p off the
map is refused when x + s cos (y + s sin) is off it too; p must fit (`rmg_zone_position_fits` on
a temporary zone of base s); on level 0 a **water zone** at p (a new template zone: type 3,
terrain 8, base s, treasure bands 100-1000 density 5 and 2000-6000 density 1, no towns, mines or
monster settings, position -1), appended to the template's zones and the zone generators; on level
1 a point without a zone. Each p goes into the triangulation. Then `triangulation_finish`
0x5FDB40; per zone of the level its borders (`rmg_zone_borders_draw`: the sides of its cell in the
triangulation, clipped to the map, drawn in the lower zone id's index: jagged for the zones present
at the start, limit the smaller of the two sizes, unless islands on level 0; straight otherwise;
its clipped corners kept in +0x3F4) and its fill (`rmg_zone_flood_fill` from its centre over
cells of no zone). Cells get the zone index in +0x20 bits 16-23 and +0x28 bit 28 (land), except
on the islands' level 0. Then `rmg_water_zones_connect`: the new zones that are neighbours with
a shared side on the map are connected (value 0), the zones' graph distances computed, and each new
zone connected to its neighbouring earlier zones unless a distance test refuses (read in part).
On level 1, the points without a zone leave cells of no zone: rock in step 3.

**Step 3, `rmg_terrain_paint`.** The zones' boxes (+0x20..+0x2C) from the cells; each zone's centre
moved to its cells' centroid; with islands each level-0 zone's polygon shrunk toward the centre
by min(L/2, max(L/4, 4)), its coast drawn jagged (limit size / 2) and its inside filled (bit 28).
Then the game's terrain painter (`terrain_painter_init` 0x5B7250 with a map interface, a terrain
and 4; `terrain_painter_add_rect` 0x5B7690; `terrain_painter_finish` 0x5B72F0, its destructor,
does the work): level 1 all rock (9), level 0 all water (8), then per zone not of water a
painter of its terrain over each of its cells with bit 28. The painter reads cells through the map
vtable's slot 4: the map vtable's slots (§2.1) are its interface. Step 11 calls the
same painter and `terrain_painter_flush` 0x5B7520.

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
jagged border, an island zone of size 1) divide by zero (whether `rmg_run`'s catch-all takes that
was not checked); the terrain painter throws on a failed allocation.

**Outside callees** (41): mapped already 7 (`crt_rand`, `crt_sqrt` 0x617B94, `crt_ftol` 0x6179D4,
`pointer_vector_insert`, `entry12_vector_insert`, `vector_insert_u32`, `empty_ret_8`); the
triangulation 6 (0x5FD010, 0x5FD790, 0x5FDB40, 0x5FD6B0 `triangulation_find`, 0x5FD330
`triangulation_destroy`, 0x5FCEB0; only the generator calls them); the terrain painter 3 (0x5B7250,
0x5B7690, 0x5B72F0; with 0x5B7520 for step 11); the C runtime 3 (operator new, free, the array
destructor); the C++ library 22 by their callers (vector inserts,
erases, sizes and copies, the 0x1C-byte connection insert, the bitset's range error;
not read). UI: the progress window's +8 twice in step 3 (12500, then 15800 / zones per
zone). No file and no object table besides `object_settings_pointer`.

**Quirk candidates (as read, unclassified):**

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

**Not read:** the triangulation's insertion and finish (whether it is Delaunay; what +0x1C, +0x20
hold); `rmg_zone_flood_fill`'s second half (0x53D640); `rmg_water_zones_connect`'s distance test
and `rmg_zone_graph_distances`' order; the corner cases of `rmg_zone_borders_draw` and
`rmg_segment_clip`; the water branch of `rmg_map_footprint_blocked`; `rmg_prototype_outline` and
`rmg_map_outline_clear` in detail; the painter's algorithm (its constructor, its rectangle code, the tests inside
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
| 0x53EFA0 `rmg_islet_draw`, 0x53ED00 `rmg_islet_heightmap`, 0x53E9E0 `rmg_heightmap_quadrants_push`, 0x54C670 `entry24_vector_size` | 0x53F470 down | an islet's terrain, shape and obstacles | read; read; inferred; read |
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
coordinates when a zone has no empty cell. Progress: +1600 (the walls), +1000 per islet, +20 per
row (the shores), +6400 (the connections).

**Outside callees** (23): mapped already 10 (`crt_rand` 0x617E5C, `vector_length_int`, the terrain
painter's four with `terrain_painter_flush` 0x5B7520, `entry12_vector_insert` 0x4347A0,
`pointer_vector_insert` 0x54D120, `vector_insert_u32` 0x404200, `empty_ret_8` 0x404140); named here
1 (`point_list_node_new`); the C runtime 3 (operator new, free, the array destructor); the C++
library 9 by their callers (a u32 vector's insert and erase, a 12-byte vector's range erase,
destructor and copy, an 8-byte point vector's insert, a 0x24-byte vector's insert and two helpers;
not read). UI: the progress window's +8 only. No file. Tables read in place:
`object_settings_pointer` 0x660428 (the flood), `creature_table_pointer` 0x6747B0 (the monster).

**Quirk candidates (as read, unclassified):**

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

**Not read:** `rmg_heightmap_quadrants_push`'s growth code and which corner each record field is
(the draw order above follows the code); the 9 library helpers; whether `rmg_map_can_place` refuses
a tent on water; what reads the zones' +0x404 and how one-way monoliths pair at write time; what
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

**Quirk candidates (as read, unclassified):**

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

**Quirk candidates (as read, unclassified):**

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
| step 1 | per template zone 0x532A5B (its town types), 0x53BB5F; two more passes of 0x53BB5F; per zone 0x532B00 or none, 0x53BFC0 | zone, pass |
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

**The run's floats:** x87 only, every conversion `crt_ftol` (truncation): step 1's candidates
trunc(f64(r) × T[k] + f64(o)) with `rmg_layout_cos` 0x682500 and `rmg_layout_sin` 0x682600 (f64,
four decimals) at the control word's precision; step 2's points (s × T[k] doubled + x, trunc;
`fcomp` bounds); trunc(sqrt(f64(dx² + dy²))) in `rmg_zone_position_fits`, `rmg_zones_touch` and
`vector_length_int` 0x5FCEB0 (steps 1, 2, 6, 7); the spacings trunc(sqrt(f64(q))) of an integer
quotient: 82944 / S (steps 5, 8), 800 / S or 1600 / S (step 10). None elsewhere.

**Clocks, retries, limits:** the clock: `time()` in seconds, once (the seed, printed in the file).
Memory-dependent: step 2's uninitialised town-type bytes (whether 0x532A5B draws) and step 15's
read past the cells (§2.5). Fixed iterations: step 1's three passes, 32 (65 with two levels)
candidates per placement. Retries: step 10 only (six per pick, three per object). Limits: a land
connection's passages ≤ ceil(n / 40) of ≤ 100 candidates; roads to costs ≤ 30000; the limits by
type in step 10's choice. Every other give-up is silent; the run's only reported failure is no
template (code 3), an exception anywhere code 1.
