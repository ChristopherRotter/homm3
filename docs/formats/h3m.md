# Map files (`.h3m`)

Read by `h3-map` (E08 T3, adventure lane) and, for staged experiments, written by `h3_formats::h3m`
(E04a T2, OPEN-QUESTIONS Q28; the writer moves into `h3-map` after E04b). Sources: the field order
and sizes of VCMI 1.7.5's map reader (`lib/mapping/MapFormatH3M.cpp`, `MapReaderH3M.cpp`,
`MapFeaturesH3M.cpp`, `lib/mapObjects/ObjectTemplate.cpp`;
https://github.com/vcmi/vcmi/blob/1.7.5/lib/mapping/MapFormatH3M.cpp), checked against the maps
that ship with the user's installation and maps the user made in the editor. Everything is
little-endian; a string is a `u32` length plus Windows-1252 bytes; a bool is a byte (non-zero =
true); a position is x, y, level (one byte each); a map file is one gzip member (RFC 1952) of the
bytes below.

## Reading: the three versions (E08 T3a, up to the terrain)

| Part | RoE `0x0E` | AB `0x15` | SoD `0x1C` |
|---|---|---|---|
| version, any players, size, underground | u32, u8, u32, u8 | same | same |
| name, description, difficulty | string, string, u8 | same | same |
| level limit | — | u8 | u8 |
| 8 players | below | below | below |
| victory, loss | below | same | same |
| teams | u8 count; if > 0, a team byte per colour | same | same |
| allowed heroes (bit = allowed) | 16 bytes | 20 bytes | 20 bytes |
| campaign placeholders | — | u32 count, a hero byte each | same |
| disposed heroes | — | — | u8 count; hero, portrait, name, players mask |
| map options | 31 bytes (zero) | same | same |
| banned artifacts (bit = banned) | — | 17 bytes | 18 bytes |
| banned spells, skills (bit = banned) | — | — | 9, 4 bytes |
| rumors | u32 count; name, text | same | same |
| customised hero types | — | — | 156 × (bool; if set, the body below) |
| terrain | 7 bytes per tile, level by level, row by row | same | same |

**Player.** Human, computer (bools). Nobody plays the colour: 6 (RoE), 12 (AB) or 13 (SoD) unused
bytes follow. Otherwise: AI tactic (i8); SoD only, the town-choice byte; allowed towns (u8 in RoE,
u16 from AB on); random town (bool); main town (bool; if set, from AB on a generate-hero bool and a
town-type byte, then its position; RoE always generates the hero); random hero (bool); main hero
type (`0xFF` none; else portrait byte and name); from AB on an unknown byte and the hero list
(u32 count; hero type byte and name each).

**Victory** (u8; `0xFF` standard): allows standard victory, applies to AI (bools), then by code:
0 artifact (u8 in RoE, u16 from AB), 1 creature (same width) and an i32 count, 2 resource u8 and
an i32 amount, 3 town position, hall level, castle level, 4 grail town, 5 hero, 6 town, 7 monster
(positions), 8 dwellings, 9 mines (nothing), 10 artifact u8 and town position. Codes 11 and 12
are HotA's and rejected. **Loss** (u8; `0xFF` standard): 0 town, 1 hero (positions), 2 days (u16).

**Customised hero type** (SoD; also the core of the hero object's body, T3b): experience (bool +
u32), secondary skills (bool + u32 count + skill and level bytes), artifacts (bool; one code per
slot, 18 before SoD, 19 in SoD, "none" `0xFF` or `0xFFFF`; u16 backpack count and codes), biography
(bool + string), sex (byte, `0xFF` default), spells (bool + 9 bytes), primary skills (bool + 4
bytes).

**Tile:** terrain (0 dirt … 9 rock), picture, river (0 none … 4 lava), river picture, road (0 none
… 3 cobblestone), road picture, flags. The reader keeps every code raw; the import maps them.

**Input bounds** (`h3_map::Limits`; above one the file is rejected before anything is
allocated, and every count must also fit the bytes left). Peaks in the 164 maps shipped with the
user's installation (50 RoE, 55 AB, 59 SoD, `Tutorial.tut` included; 2026-10-10,
`cargo xtask verify --with-original-data`, which runs `crates/h3-map/examples/original_data.rs`):

| Limit | Default | Shipped peak |
|---|---|---|
| file (compressed) | 4 MiB | 123 343 bytes |
| map (inflated) | 16 MiB | 525 124 bytes |
| map side | 144 (two levels) | 144 |
| string | 65 536 bytes | 300 (T3a's strings only) |
| heroes per player | 64 | 6 |
| campaign placeholders | 156 | 0 |
| rumors | 256 | 19 |
| secondary skills per hero | 28 | 0 (customised hero types) |
| backpack | 256 | 0 (customised hero types) |

Versions the reader rejects: Heroes Chronicles (`0x1D`), HotA (`0x20`), WoG (`0x33`) and every
other code.

## The subset we write (E04a T2)

### Order of a SoD map (version `0x1C`), as we write it

| Part | Bytes | Our value |
|---|---|---|
| version | u32 | `0x1C` |
| any players, size, underground | u8, u32, u8 | 1, 36/72/108/144, 0 |
| name, description | string, string | |
| difficulty, level limit | u8, u8 | 1 (normal), 0 |
| 8 players | see below | |
| victory, loss condition | u8, u8 | `0xFF`, `0xFF` (standard) |
| teams | u8 | 0 |
| allowed heroes | 20 bytes, bit = allowed | the editor's default: all but hero 4 and the campaign heroes 145–155 |
| campaign placeholders | u32 | 0 |
| disposed heroes | u8 | 0 |
| map options | 31 bytes | zero |
| banned artifacts, spells, skills | 18, 9, 4 bytes, bit = banned | the editor's default (artifacts 127–143), none, none |
| rumors | u32 | 0 |
| customised hero types | 156 × bool | all 0 |
| terrain | 7 bytes per tile, row by row | type, picture, river ×2, road ×2, flags |
| templates | u32 count + records | name, 6 bytes passable, 6 bytes visitable, u16 terrains, u16 editor terrains, u32 class, u32 subclass, u8 group, u8 priority, 16 zero bytes |
| objects | u32 count + records | x, y, z (u8 each), u32 template index, 5 zero bytes, body |
| timed events | u32 | 0 |

**Player** (playable): human, computer (bools), AI tactic (i8), town choice offered (u8), allowed
towns (u16 bit mask), random town (bool), has main town (bool; 0 here), random hero (bool), main
hero type (u8, `0xFF` none; if set: portrait u8 `0xFF` and a name string), one unknown byte (0), and
the list of the player's heroes (u32 count, then hero type u8 + name string). A colour nobody plays
is 15 bytes (two zero bools and 13 bytes the game skips; the editor writes an empty player there).

**Template masks:** `Objects.txt` gives 48 characters per mask (6 rows of 8, top left first); in
the file, bit `j` of byte `i` is character `47 − (8i + j)`. Hero templates are not in `Objects.txt`:
the editor writes `ahNN_e.def` (NN = class), class 34, subclass = class, group 3, terrains `0xFF`,
masks blocked/visitable at one tile, which sits one column *left* of the object's position — so a
hero standing on (x, y) is written at (x + 1, y).

**Hero body** (class 34): u32 identifier, owner u8, hero type u8, then optional parts each behind a
bool: name (string), experience (u32), portrait (u8), secondary skills (u32 count, then skill u8 +
level u8), army (7 × (u16 creature, u16 count), `0xFFFF` empty); formation u8; artifacts (19 × u16
by slot, `0xFFFF` empty, then u16 backpack count and u16 ids); patrol radius u8 (`0xFF` none);
biography (bool + string); sex i8 (`-1` default); spells (bool + 9-byte mask); primary skills (bool
+ 4 × u8); 16 zero bytes.

**Monster body** (classes 54, 71, 162–164): u32 identifier, u16 count (0 = random), disposition u8
(0 compliant … 4 savage), message and treasure (bool; 0 here), never flees (bool), does not grow
(bool), 2 zero bytes.

**Terrain pictures** observed for interior tiles: dirt and water 21–28, grass 49–56. Other terrains
are rejected until a map shows their numbers.
