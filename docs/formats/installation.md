# Installation catalogue (E02 T1)

What the user's game installation actually contains, as observed on 2026-09-30 from copies in the
gitignored `original-data/` directory. The installation itself was only read (AGENTS.md). Names,
sizes, offsets and layouts below are observations of these files, not assumptions; where a
statement is an inference it says so.

## 1. Edition and version (Q8)

| Evidence | Observed |
|---|---|
| Distribution | GOG, `goggame-1207658787.info`: name "Heroes of Might and Magic 3 Complete", language English (`en-US`), buildId `58018312193614395` |
| `README.TXT` | "Heroes of Might and Magic III Complete, Version 4.0, September 2000" |
| `Heroes3.exe` | PE32 (i386), link timestamp 2000-09-08 00:52:05 UTC, VERSIONINFO FileVersion/ProductVersion **4.0**, ProductName "Heroes of Might and Magic III: The Shadow of Death", OriginalFilename `HEROES3.EXE`, 2 732 032 bytes |
| `h3maped.exe` | Map editor, FileVersion 3.00, link timestamp 2000-09-05 |
| Campaign files | `H3bitmap.lod` carries the SoD campaigns, `H3ab_bmp.lod` the AB campaigns (see §3) |

**Conclusion:** the installation is the **Complete edition, version 4.0** (RoE + AB + SoD in one
executable; SoD rules and balance). It contains no RoE-only or AB-only executable or data set.
User decision 2026-09-30: the engine supports RoE (base), AB and SoD, with AB and SoD as expansions
that can be switched off; SoD requires AB (OPEN-QUESTIONS, answered).

SHA-256 of the copied files (to recognise the same build later):

| File | Bytes | SHA-256 |
|---|---:|---|
| `Heroes3.exe` | 2 732 032 | `aac39e8cf7ff46fb009057bd53716f4be46a88a49b857165b8d19e04df9d3d7d` |
| `h3maped.exe` | 2 134 016 | `4480fba145c9f885942cc668d4bce430fe39c0fa482d1a6e58f96318ab857a37` |
| `Data/H3bitmap.lod` | 105 724 954 | `774cbfa7cfed6713ec80c48f8619a258dac14260e4f85faeebb29c053950a5a8` |
| `Data/H3ab_bmp.lod` | 44 019 475 | `3cc4256e8844b3cd0af2706431dee320848bd1630b9f6c6022f6d4c78780f347` |
| `Data/H3sprite.lod` | 64 973 159 | `57caf2c50573f33a0d91e4222e51d3a73c136d44decf59dde21cacad88fe5d66` |
| `Data/H3ab_spr.lod` | 12 131 193 | `e0d5003742c8602827ef409966784483dece6eedde76aa2cfeee26cb12d25d67` |

## 2. Top-level layout

291 files, about 981 MB. Relevant for the engine: `Heroes3.exe` and the four `Data/*.lod`
archives. Not relevant (non-goals: graphics, sound, video, tooling): `Data/*.snd`, `Data/*.vid`,
`Mp3/`, DLLs (`BINKW32`, `SMACKW32`, `MSS32`, `IFC20`, `MP3DEC.ASI`, `xdd.dll`), help files, PDFs,
GOG launcher files, `Maps/`, `Games/` (saves) and `random_maps/` (maps are E07).

Copied into `original-data/install/`: `Heroes3.exe`, `h3maped.exe`, `README.TXT`, the GOG `.info`
and file list, and `Data/{H3bitmap,H3ab_bmp,H3sprite,H3ab_spr}.lod`.

## 3. LOD archives

Little-endian container, verified on all four archives:

| Offset | Size | Field |
|---:|---:|---|
| 0 | 4 | magic `LOD\0` |
| 4 | 4 | u32 archive kind: `200` (`H3bitmap`, `H3sprite`) or `500` (`H3ab_bmp`, `H3ab_spr`) |
| 8 | 4 | u32 number of entries `n` |
| 12 | 80 | unknown; all zero in the kind-500 archives, a few non-zero bytes in the kind-200 archives |
| 92 | 32 × 10 000 | entry table; the first `n` slots are used, the rest are zero |
| 320 092 | … | file data (the first entry's data starts here in all four archives) |

Entry (32 bytes): name (16 bytes, NUL-terminated ASCII, mixed case), u32 data offset (absolute),
u32 uncompressed size, u32 type tag, u32 compressed size. A compressed size of `0` means the data
is stored; otherwise the data is a zlib stream that inflates to exactly the uncompressed size.

| Archive | Entries | Compressed | Content by extension (type tag) |
|---|---:|---:|---|
| `H3bitmap.lod` | 4 908 | 4 887 | PCX (16, 17), **TXT (2) × 71**, H3C campaigns (1) × 14, FNT (80), PAL (96), XMI, IFR |
| `H3ab_bmp.lod` | 2 491 | 2 469 | PCX (16, 17), **TXT (2) × 71**, H3C (1) × 14 |
| `H3sprite.lod` | 4 013 | 2 575 | DEF (64–73), MSK (79) |
| `H3ab_spr.lod` | 569 | 344 | DEF, MSK |

Only the TXT entries are game data. Both bitmap archives contain the **same 71 table names**
(different letter case); 48 are byte-identical, 23 differ: `advevent`, `arraytxt`, `artevent`,
`artraits`, `artslots`, `camptext`, `cmpedcmd`, `cmpeditr`, `cmpmovie`, `cmpmusic`, `credits`,
`crtraits`, `editor`, `editrcmd`, `genrltxt`, `help`, `herobios`, `monolith`, `objects`, `objnames`,
`rmg`, `seerhut`, `sptraits`.

**Which copy is SoD, which is AB (inference, strong):** `H3bitmap.lod` holds the SoD campaigns
(`Sandro`, `Yog`, `Gem`, `Crag`, `Gelu`, `Final`, `Secret`), `H3ab_bmp.lod` the AB ones (`ab`,
`blood`, `festival`, `fire`, `fool`, `slayer`). In `CRTRAITS`, the `H3bitmap` copy has the SoD
balance (Angel 1 gem, Archangel 3 gems, Lizardman growth 9, stronger Dragon Fly, Cerberus 2–7) and
the `H3ab_bmp` copy the older values. So `H3ab_bmp.lod` is the **AB-era table set**, `H3bitmap.lod`
the **SoD** one. The diff between the two is the AB→SoD change set; RoE-era values are not in the
installation and have to come from other sources (wiki/VCMI, labelled).

## 4. Text tables

Common syntax (observed in `CRTRAITS.TXT`; to be confirmed per table as each is harvested):
Windows-1252 bytes, CRLF line ends, TAB-separated fields, fields may be wrapped in double quotes and
then contain CRLF (multi-line ability texts). Header rows are part of the file.

### `CRTRAITS.TXT` (creatures)

- Rows 1–2: headers (group names, then 25 column names). 183 data rows, each with 25 fields.
- Columns: singular name, plural name, cost in seven resources (wood, mercury, ore, sulfur,
  crystal, gems, gold), fight value, AI value, growth, horde growth, hit points, speed, attack,
  defense, damage low, damage high, shots, spells (casts), adventure-map count low/high, ability
  text, attributes (header says "Reference only, do not change these values").
- Blank rows separate the groups; rows 3–16 are Castle, 20–33 Rampart and so on (groups of 14,
  then a 3-row gap). The non-blank rows in order are the **150 creature ids 0–149** used by the
  executable (§5): 8 towns × 14, 6 neutral elementals/golems, 14 Conflux rows (including 4 rows
  named `NOT USED (n)`), 13 AB neutrals, 5 war machines.
- The attribute column uses names such as `DOUBLE_WIDE`, `FLYING_ARMY`, `SHOOTING_ARMY`,
  `IS_UNDEAD`, `KING_1..3`, `const_two_attacks`, `const_free_attack`, `const_no_melee_penalty`,
  `IMMUNE_TO_MIND_SPELLS`, `SIEGE_WEAPON`, `CATAPULT`; it is incomplete (e.g. no multi-head flag on
  hydras) and inconsistently spaced. Not authoritative; the executable's flags are (§5).

`RESTYPES.TXT` has 8 rows: the seven resources in the `CRTRAITS` cost-column order, then
`Abandoned` (not a resource; not harvested). `TownType.txt` has the 9 town names in town-index
order (Castle … Conflux).

Other tables (artifacts `ARTRAITS`, spells `SPTRAITS`, heroes `HOTRAITS`, classes `HCTRAITS`,
secondary skills `SSTRAITS`, buildings, objects, …) are listed in §3 and catalogued when an epic
needs them.

## 5. `Heroes3.exe`: static creature table

Level, town and ability flags are **not** in any text table. They are in a static array in the
executable's `.data` section:

- File offset `0x2703B8` (VA `0x006703B8`, image base `0x400000`, `.data` raw offset = RVA), 150
  records × 116 bytes.
- Record: i32 town (0–8, `-1` = neutral), i32 level (0–6, `-1` for unused ids), pointer to sound
  prefix, pointer to DEF name, u32 flags, then 3 pointers (names, ability text) and 21 i32 values
  (cost, fight/AI value, growth, …, adventure-map count), which are **zero** in the file: the game
  fills them from `CRTRAITS.TXT` at start-up.
- Town indices (inference from the rows): 0 Castle, 1 Rampart, 2 Tower, 3 Inferno, 4 Necropolis,
  5 Dungeon, 6 Stronghold, 7 Fortress, 8 Conflux. The four classic elementals (ids 112–115) have
  town 8 in this build; the golems (116–117), dragons and other AB neutrals (132–144) and war
  machines (145–149) are `-1`.
- Flag bits (inference, cross-checked against the attribute column and known creature properties;
  the harvester re-checks each one against `CRTRAITS`): `0x1` two-hex, `0x2` flying, `0x4` shooter,
  `0x8` two-hex (breath) attack, `0x10` alive, `0x20` catapult, `0x40` war machine, `0x80`/`0x100`/
  `0x200` king 1/2/3, `0x400` mind immunity, `0x1000` no melee penalty, `0x4000` fire immunity,
  `0x8000` strikes twice, `0x10000` no enemy retaliation, `0x20000` no morale, `0x40000` undead,
  `0x80000` multi-headed attack (hydras, Cerberus, psychic/magic elementals), `0x100000` area (ball) ranged attack (ids 45, 64, 65),
  `0x80000000` dragon. `0x800` is set on ids 35, 74, 75 and not identified yet; `0x2000` and bits
  21–30 are unused by all 150 records.

The harvester cross-checks the flags with the `CRTRAITS` attribute column. Where they disagree
the executable matches known gameplay and wins: the column marks Hell Hound, Cerberus, Devil, Arch
Devil and Crystal Dragon as flying and Psychic/Magic Elemental as two-hex; none of them is.

Only this exact executable (SHA-256 above) is known to have the table at this offset; a harvester
must refuse other builds rather than read garbage.
