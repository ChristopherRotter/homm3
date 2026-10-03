# Map files (`.h3m`): the subset we write (E04a T2)

Written by `h3_formats::h3m` for staged experiments (OPEN-QUESTIONS Q28). Sources: the field order
and sizes of VCMI 1.7.5's map reader (`lib/mapping/MapFormatH3M.cpp`, `MapReaderH3M.cpp`,
`MapFeaturesH3M.cpp`, `lib/mapObjects/ObjectTemplate.cpp`;
https://github.com/vcmi/vcmi/blob/1.7.5/lib/mapping/MapFormatH3M.cpp), checked on 2026-10-03 by
parsing SoD maps that ship with the user's installation (`Deluge.h3m`) and two maps the user made in
the editor. Everything is little-endian; a string is a `u32` length plus Windows-1252 bytes; a map
file is a gzip stream of the bytes below.

## Order of a SoD map (version `0x1C`), as we write it

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
