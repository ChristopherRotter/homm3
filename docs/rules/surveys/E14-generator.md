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
with it as 9 dwords; the code indexes position + 1); the object generators go to H3API's
`objectGenerators` +0x10F0 (T1a1 read +0x1100; §2.1); the cell's +0x20 is H3API's zone data.

### 1.8 Draws and floats so far; what T1a1 did not read

- **Draws, in stream order:** the dialog's Random choices (§1.1, the game's state); `srand(time)`;
  water's Random; the template; the 15 steps (44 sites in their closures, §1.5); the teams in the
  header. One generator, one thread, one stream.
- **Floats:** x87 in steps 1, 2, 5, 8, 10 (`crt_sqrt`, `crt_ftol`); T1a2 lists each site.
- **Not read:** the 15 steps' bodies (but steps 4, 8 and 15's own code, step 1's in part), the header before
  its end, the cell and template writers, the per-object writers (the setup's readers and the
  destructor: §2.1). The limits by
  type are set but where they are read was not found.
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
