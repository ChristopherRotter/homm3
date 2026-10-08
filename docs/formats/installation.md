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

## 6. Hero, skill, artifact and spell tables (E04a T1, 2026-10-03)

Same syntax as §4. "AB vs SoD" compares the `H3ab_bmp.lod` copy (AB era) with the `H3bitmap.lod`
copy (SoD). Details per table, with the parsing traps, are in the E04 surveys
(`docs/rules/surveys/`).

| Table | Rows | Columns | AB vs SoD |
|---|---|---|---|
| `SPTRAITS.TXT` | 2 header rows; group rows "Adventure Spells" (10 spells), "Combat Spells" (60), "Creature Abilities" (11): 81 spells, ids 0–80 in row order | 34: name, abbreviated name, level, school flags (Earth, Water, Fire, Air), mana cost ×4 (none/basic/advanced/expert), power factor, effect value ×4, chance to appear per town ×9, AI value ×4, description ×4, attributes ("reference only") | one row differs: id 57 is "Fear (disabled)" in AB and Titan's Lightning Bolt (level 5, Air, cost 0, effect 600) in SoD; descriptions otherwise equal |
| `ARTRAITS.TXT` | 2 header rows + 144 artifacts (ids 0–143 in row order) + padding rows | 23: name, cost, 19 slot flags (`x`), class letter (S/T/N/J/R), description | AB copy: 129 artifacts and 22 columns (no Misc 5 slot); SoD adds the 12 combination artifacts (129–140), 3 unused rows (141–143) and the Misc 5 column |
| `ArtSlots.txt` | 19 slot names (SoD), 18 (AB) | 1 | SoD appends Misc 5 |
| `HCTRAITS.TXT` | 2 header rows + 18 classes | 51: name, aggression, 4 initial primary skills, gain probabilities for levels 2–9 (4) and 10+ (4), 28 secondary-skill probabilities, 9 tavern weights | identical |
| `HOTRAITS.TXT` | 2 header rows + 156 heroes | 10: name, 3 × (low, high, creature name for reference) | identical |
| `HeroSpec.txt` | 2 header rows + 156 | 3: specialty short name, long name, text | identical |
| `SSTRAITS.TXT` | 2 header rows + 28 skills | 4: name, description per level (the numbers are only in the texts) | identical |
| `ballist.txt` | 2 header rows + 4 levels | 11: per Ballistics level the hit chances on keep/tower/gate/wall, shots, damage distribution | identical |
| `PriSkill.txt`, `SkillLev.txt` | 4 / 6 lines | names of the primary skills; level names and abbreviations | identical |

Effects of artifacts, skills and specialties are **not** in any text table. Most of them are in
static tables of the executable (§7); the rest are code.

## 7. `Heroes3.exe`: further static tables (E04a T1, 2026-10-03)

Same executable as §5 (`.data` raw offset = RVA, so file offset = VA − 0x400000 there; `.rdata`
likewise). Found by searching for known values and checked against the text tables and VCMI. As in
§5, a harvester must refuse other builds.

### 7.1 Hero table — VA 0x679DD0 (file 0x279DD0), 156 × 92 bytes

| Offset | Field |
|---:|---|
| 0x00 | i32 female |
| 0x04 | i32 unknown (7 for most heroes) |
| 0x08 | i32 class (0–17, HCTRAITS order) |
| 0x0C | 2 × (i32 secondary skill, i32 level 1–3): starting skills (`-1` = none) |
| 0x1C | u8 has a spell book (93 heroes) |
| 0x20 | i32 starting spell (`-1` = none) |
| 0x24 | 3 × i32 starting creature types |
| 0x30 | 2 pointers: small and large portrait names |
| 0x38 | 4 flag bytes: byte 0 = RoE hero (0–127), byte 1 = available with the expansions (all but id 4), byte 2 = campaign hero (145–155) (meaning inferred from the values) |
| 0x3C | pointer to the name (zero in the file; filled from `HOTRAITS.TXT`) |
| 0x40 | 3 × (i32 low, i32 high) starting stack sizes (zero in the file; filled from `HOTRAITS.TXT`) |

A 157th record follows the table; it is not a hero of the text tables and is ignored. Nine heroes
have a war machine as their third starting creature (six the Ballista, three the First Aid Tent,
count 1–1 in `HOTRAITS.TXT`); the harvester writes them as `war_machines`. Read by
`h3_formats::exe::hero_data` together with §7.2, §7.4 and §7.6 (E04a T3).

### 7.2 Hero specialty table — VA 0x678420 (file 0x278420), 156 × 40 bytes

Pointer to it at VA 0x679C80. Record: i32 kind, i32 id, i32 attack, i32 defense, i32 damage,
i32 upgrade 1, i32 upgrade 2, 3 text pointers (zero in the file; filled from `HeroSpec.txt`).
Kinds: 0 secondary skill (37 heroes), 1 creature with level scaling (55, among them 6 Ballista
heroes, creature 146), 2 resource (13), 3 spell (34), 4 fixed creature stats (11), 5 speed (1,
Sir Mullich +2), 6 creature upgrade (2, Dracon and Gelu), 7 dragons (2, Mutare and Mutare Drake),
−1 none (Adrienne). Per-kind formulas: `docs/rules/surveys/E04-heroes.md` §3.

### 7.3 Artifact primary skills — VA 0x63E758 (file 0x23E758), 144 × 4 bytes

Signed bytes attack, defense, power, knowledge per artifact id. Matches VCMI's hero-wide
`PRIMARY_SKILL` bonuses for all 144 artifacts (script check); creature-limited bonuses (Vial of
Dragon Blood) and per-copy bonuses (Quiet Eye of the Dragon, +1/+1 here) are where VCMI models
them differently. Example: Armageddon's Blade (128) is +3/+3/+3/+6, which settles the wiki
template's +6 power (artifact survey, D5). The artifact setup table itself (pointer at VA
0x660B68) is uninitialised in the file and filled from `ARTRAITS.TXT`.

Read by `h3_formats::exe::hero_data` (`HeroData::artifact_primary`, E04a T3b); the harvester writes
the non-zero rows as `primary_skills` effects. The table holds only the combinations' own values
(Power of the Dragon Father +6/+6/+6/+6, Angelic Alliance 0: its +21 comes from its parts), and 0
for Vial of Dragon Blood, whose dragon-only +5/+5 is code and comes from the gap-fill file. No
static table of combination parts was found (searched as bitsets and as id lists; E04a T3b): the
parts are gap-filled from VCMI and the wiki.

**Slot kinds (harvester, E04a T3b).** Every artifact row flags either no position (the Grail) or
exactly all positions of one kind: both rings, all misc positions (four in the AB copy, five in
SoD), or a single position. The harvester groups the `ArtSlots.txt` positions into these kinds
(`h3_harvest::artifacts::SLOT_KINDS`) and rejects a row that flags anything else.

### 7.4 Secondary skill values — VA 0x63E998–0x63EAA7 (file 0x23E998), 17 × 4 values

One four-entry table per skill, indexed by level 0–3; each is read by a small getter that also
applies the hero's skill specialty (× (1 + 0.05 × level), constant 0.05 at VA 0x63EAE4). The skill
was identified from the hero field each getter reads (the secondary-skill levels are bytes at
hero +0xC9 + skill id).

| VA | Skill | Values (level 0–3) |
|---|---|---|
| 0x63E998 | Luck | i32 0, 1, 2, 3 |
| 0x63E9A8 | Leadership | i32 0, 1, 2, 3 |
| 0x63E9B8 | Necromancy | f32 0, 0.10, 0.20, 0.30 |
| 0x63E9C8 | Mysticism | i32 1, 2, 3, 4 (spell points per day) |
| 0x63E9D8 | Scouting | i32 5, 6, 7, 8 (radius) |
| 0x63E9E8 | Archery | f32 0, 0.10, 0.25, 0.50 |
| 0x63E9F8 | Offense | f32 0, 0.10, 0.20, 0.30 |
| 0x63EA08 | Armorer | f32 0, 0.05, 0.10, 0.15 |
| 0x63EA18 | Estates | i32 0, 125, 250, 500 |
| 0x63EA28 | Eagle Eye | f32 0, 0.40, 0.50, 0.60 |
| 0x63EA38 | Diplomacy | f32 0, 0.20, 0.40, 0.60 |
| 0x63EA48 | Resistance | f32 0, 0.05, 0.10, 0.20 |
| 0x63EA58 | Learning | f32 0, 0.05, 0.10, 0.15 |
| 0x63EA68 | Logistics (getter not yet confirmed) | f32 0, 0.10, 0.20, 0.30 |
| 0x63EA78 | Sorcery | f32 0, 0.05, 0.10, 0.15 |
| 0x63EA88 | Intelligence | f32 0, 0.25, 0.50, 1.00 |
| 0x63EA98 | First Aid | f32 0, 1, 2, 3 (the tent's maximum heal is 25 × (1 + value), §7.5) |

These are original data and rank above VCMI and the wiki; all agree with the skill texts and
with VCMI except where noted in the skills survey. The harvester (E04a T3) turns the tables of
Luck, Leadership, Archery, Offense, Armorer, Diplomacy and First Aid into the skills' hero effects
(fractions as basis points, each checked to be a whole number of them) and the growth constant
0.05 into the skill specialties' 500 bp per level; Logistics is not read.

### 7.5 Code reading: First Aid Tent heal

Identifying the First Aid getter (VA 0x4E4920) led to its battle caller (VA 0x478BEC): the tent
heals `Random(1, trunc(25.0 × getter))` hit points, where the getter returns
`1 + table[level] × (1 + 0.05 × hero level if First Aid specialist)` and `Random(a, b)`
(VA 0x50B230) is `a + rand() % (b − a + 1)`; the result is capped by the missing hit points. So
without the skill the tent heals 1–25 (confirms Q26) and with Basic/Advanced/Expert First Aid
1–50/1–75/1–100 (the wiki's reading; VCMI's fixed 50/75/100 is wrong). This is a reading of the
game's code, not of a data table; such readings are allowed as a last resort, for insight only
(user, OPEN-QUESTIONS Q29: findings in our own words and numbers, no code or listings copied).

### 7.6 Spell table — VA 0x685450 (file 0x285450), 81 × 0x88 bytes

Static fields per spell id: i32 type (−1 negative, 0 neutral, 1 positive) at +0, pointer to the
sound name at +4, i32 animation at +8, u32 flag bits at +0xC. All other fields (name, level,
schools, costs, power, effects, chances, AI values, texts) are zero in the file and filled from
`SPTRAITS.TXT`. The harvester (E04a T3) writes the type as the spell's `polarity`; since E04b T2a
it also reads the flag word (`h3_formats::exe::HeroData::spell_flags`): it checks the category bits
(`0x1`, `0x2`, `0x8`) against the table's group and writes the damage spells' `target` (`0x10` a
stack, `0x80` a hex, neither: none) and `damage` effect (`0x200`).

**Flag bits** (decoded in E04b T1, 2026-10-05, by matching the 81 values against the spells' known
behaviour and VCMI 1.7.5's `config/spells/*.json`; no code was read). *Confirmed*: consistent with
every row and with VCMI; *probable*: a consistent pattern whose meaning is not proven.

| Bit | Meaning | Set on (spell ids) | Confidence |
|---|---|---|---|
| `0x1` | combat spell | 10–69 | confirmed |
| `0x2` | adventure spell | 0–9 | confirmed |
| `0x4` | timed effect on the target stack | 27–34, 36, 41–46, 48–62 except 57, 70, 71, 73–75 | confirmed (absent on Dispel, Cure, Disrupting Ray (permanent), Bind, Clone (its duration belongs to the new stack)) |
| `0x8` | creature ability, never in a spell book | 70–80 | confirmed |
| `0x10` | one target stack, no mass version | 15–19, 29, 34, 36, 38–40, 47, 55–57, 60, 62, 63, 65, 70–80 | confirmed |
| `0x20` | mass version from Advanced | 61 (Forgetfulness) | **confirmed** by code reading (§7.8, E04b T2e): 0x59E060 gives the spell a target stack only below Advanced; agrees with quirk S-10 |
| `0x40` | mass version at Expert | 27, 28, 30–33, 35, 37, 41–46, 48–54, 58 | confirmed (VCMI also makes Forgetfulness mass at Expert; see quirk S-10) |
| `0x80` | targets a hex or area | 10–13, 20–23, 59 (Berserk) | confirmed |
| `0x100` | targets an obstacle | 64 (Remove Obstacle) | probable (one row) |
| `0x200` | deals damage | 15–26, 57, 77 (Thunderbolt) | confirmed |
| `0x400` | mind spell | 49, 50, 56, 59–62 (Mirth, Sorrow, Frenzy, Berserk, Hypnotize, Forgetfulness, Blind) | confirmed |
| `0x800` | beneficial effect on an own stack | 27–34, 36, 37, 41, 43, 44, 46, 48, 49, 51, 53, 55, 56, 58 | probable |
| `0x1000` | war machines are not affected | 18, 24, 38–40, 43, 53, 54, 58–63, 65, 70–75 | **confirmed** by code reading (§7.8, E04b T2d): the spell-effect test returns 0 for a war machine against a spell with this bit. VCMI also spares war machines from Bless, Curse, Mirth, Sorrow and Frenzy, which lack the bit; Bloodlust (43), Haste (53), Slow (54) and Counterstrike (58) have it, as in VCMI (list corrected in E04b T3b1 prep, 2026-10-06: it had left out 43, 53, 54, 58; the values below and the packs were right) |
| `0x2000` | granted by an artifact | 57 (Titan's Lightning Bolt) | probable (one row) |
| `0x4000` | protective spell | 27–34, 36, 46, 58 | probable |
| `0x8000`–`0x100000` | AI categories: `0x8000` damage to one target or area, `0x10000` damage that can hit own stacks, `0x20000` damage to every stack, `0x40000` other battle spells, `0x80000` new or raised stacks, `0x100000` special (map movement spells, Earthquake, Titan's Lightning Bolt) | see the values | probable; no rule reads them |

Values (`id:flags`, hexadecimal; SoD copy of the executable):

```text
 0:100002  1:2  2:2  3:2  4:2  5:2  6:100002  7:100002  8:100002  9:100002
10:81 11:81 12:81 13:81 14:100001 15:8211 16:8211 17:8211 18:9211 19:10211
20:8281 21:8281 22:10281 23:8281 24:21201 25:20201 26:20201 27:44845 28:44845 29:44815
30:44845 31:44845 32:44845 33:44845 34:44815 35:40041 36:44815 37:40841 38:81011 39:81011
40:1011 41:40845 42:40045 43:41845 44:40845 45:40045 46:44845 47:40011 48:40845 49:40c45
50:40445 51:40845 52:40045 53:41845 54:41045 55:40815 56:40c15 57:10a211 58:45845 59:41485
60:81415 61:1425 62:41415 63:41011 64:101 65:41011 66:80001 67:80001 68:80001 69:80001
70:101c 71:101c 72:1018 73:101c 74:101c 75:101c 76:18 77:218 78:18 79:18 80:18
```

The AB and SoD copies of `SPTRAITS.TXT` differ only in row 57 (AB: "Fear (disabled)", level 4;
SoD: Titan's Lightning Bolt, level 5) among the numeric columns; RoE's table is not in the
installation (Q15).

### 7.7 Code reading: the battle's stack slots and their limit (E04b T1, 2026-10-05)

Read for OPEN-QUESTIONS Q42 (Q29: insight only, nothing copied). The battle manager keeps, per
side, an array of **21 stack records** of 0x548 bytes at offset +0x54CC (the side stride 0x6EE8 =
21 × 0x548 occurs in 61 instructions) and the number of stacks per side at +0x54BC. The spell
dispatcher (VA 0x59ECC4) maps spell ids 10–69 to handlers through a byte table at 0x59F6AC:

- **Clone** (spell 65, handler at 0x59F43E) is refused when the caster's side already has
  **20 or more** stacks; the game then shows `GENRLTXT.TXT` line 185 (the side "already has %d
  creature groups" and cannot take more).
- **Summon Elemental** (spells 66–69, handler at 0x59F55D; the same test as the helper at
  0x5A93A0, which the game calls at 0x43BE2F) needs fewer than 20 stacks on the side **and**
  either no elemental summoned yet by that side or the same creature (per side at +0x132A8, −1 =
  none; spells 66, 67, 68, 69 summon creatures 114, 113, 115, 112: Fire, Earth, Water, Air).
- No other comparison of the count with 20 or 21 was found. Resurrection, Animate Dead and
  Sacrifice bring back a dead stack in its own record, so they never need a new one.

Not read: where Pit Lords' demons go, and why the 21st record stays free. A dead clone's or summoned
stack's record goes to the next stack the side creates (§7.19).

### 7.8 Code reading: damage spells, immunities and magic resistance (E04b T2d, 2026-10-06)

Read for T2d (Q29: insight only, nothing copied; disassembly of the SoD executable with
`capstone`). Floats are x87: intermediate values are extended precision, stores to memory are
single precision (`f32`); `_ftol` (0x6179D4) truncates; 0x618CC7 is `ceil` (rounding mode up).

**Effect handlers.** The battle manager's cast routine switches on the spell id through a byte
table at 0x5A2838 (targets at 0x5A27A0): one handler per damage spell 15–19 and 24–26, the four
area spells 20–23 share 0x5A4970, Titan's Lightning Bolt (57) shares Lightning Bolt's. A
single-target spell's value is `power × power factor + effect[mastery]`, passed through the
damage-modifier function 0x5A78E0, which applies in this order:

1. **The caster's bonuses** (0x4E5760, on the hero): `v = f32(value)`; `× 1.5` if the hero has
   the Orb of one of the spell's schools (checked air, earth, fire, water; at most once); `× (1 +
   Sorcery)` (the §7.4 table, grown by `× (1 + 0.05 × level)` for a Sorcery specialist); both
   products are stored as `f32`, **not truncated**; then, if the spell has a target stack, the
   specialty bonus (0x4E5FF0) computed from `t = trunc(v)` is added as an integer and the result
   truncated. Specialty bonus, for a hero whose specialty is this spell: by default
   `ceil((hero level div creature level) × t × 0.03)` (integer division first; the creature
   level is the stack's 0-based level + 1); Magic Arrow `t >> 1` (Ciele, +50 %); Fire Wall `t`
   (Luna, +100 %); a few timed spells use their own tables (§7.12).
2. **The target creature** (0x44B4B0, integer): Air and Storm Elementals double damage from
   Lightning Bolt, Chain Lightning, Armageddon and Titan's Lightning Bolt; Fire and Energy
   Elementals from Ice Bolt and Frost Ring; Earth and Magma Elementals from Meteor Shower; Water
   and Ice Elementals from Fireball, Inferno and Armageddon. Stone Golem `/ 2`, Iron Golem `/ 4`,
   Gold Golem `× 15 / 100`, Diamond Golem `/ 20` (C division, truncating).
3. **Protection spells on the target** (0x5A7BB0): the first of the spell's schools, checked
   earth, air, fire, water, for which the stack has its protection active multiplies by the
   stored `f32` factor and truncates (quirk S-19 for Magic Arrow; T3).

**Chain Lightning** (0x5A6360): the first target takes the full value; each later target takes
the *value before the caster's bonuses* halved by integer division at every jump, then step 1–3
for that target (so the specialty divides by each target's level). The next target (0x5A61F0) is
the stack not yet hit with the smallest `trunc(sqrt(dx² + dy²))`, searched defender's stacks
first, then the attacker's, each by stack record, keeping the first on ties (strict `<`); each
candidate is first tested like a spell target (below): when casting for real, a candidate that
fails its resistance roll is skipped for that jump (it can be chosen at a later jump); an immune
one is never chosen. `dx`, `dy` come from getters 0x446660/0x446630: the stack's hex record's
screen position, x shifted by ±22 for a two-hex stack (towards its second hex), y lifted by
half of `267 − top offset of the creature's sprite frame` (a graphics value; inference: screen
pixels, as the mouse-to-hex code at 0x464870 uses 44 × 42 pixel hexes and the bolt's animation
length is derived from the same distance). Jumps per mastery from the table at 0x642274.

**Area spells** (0x5A4970): the stacks on the area's hexes (one entry per stack), each tested
and rolled separately, then damaged with steps 1–3.

**Can a spell affect a stack** (0x5A8090 → 0x44A4D0; returns a factor `f` in [0, 1], 0 =
immune): absolute cases first: the stack's hero wears Power of the Dragon Father and the spell
is level 1–4; war machines (creature flag `0x40`) against spells with flag `0x1000` (Implosion,
Death Ripple, …; settles dispute M-1: the bit means "war machines are not affected"); creature
149 (arrow tower); per-spell hero artifacts (the Pendants, T8); Death Ripple spares undead
(creature flag `0x40000`), Destroy Undead affects only undead. Then, unless either hero has the
Orb of Vulnerability: mind spells (flag `0x400`) against mind-immune creatures (`0x400`); fire
spells (school bit 2) against fire-immune creatures (`0x4000`); per creature: Dwarf and Crystal
Dragon `f = 0.8`, Battle Dwarf `0.6`; Air and Storm Elementals immune to Meteor Shower (and
Blind); Earth and Magma Elementals to Lightning Bolt, Chain Lightning, Armageddon and Titan's
Lightning Bolt; Water and Ice Elementals to Ice Bolt and Frost Ring; Green, Red and Azure Dragons
to spells of level ≤ 3, Gold Dragon ≤ 4, Black Dragon and Magic Elemental to all. Then `f =
f32(f − r)` with `r` the resistance of the stack's own hero (0x4E46E0: the §7.4 Resistance
value, grown `× (1 + 0.05 × level)` for a Resistance specialist, `+ 0.05` Garniture of
Interference, `+ 0.10` Surcoat of Counterpoise, `+ 0.15` Boots of Polarity, each stored as
`f32`, at most 1); `f < 0` becomes 0. A positive spell ignores `f` (only immunity counts). For a
neutral or negative spell, a stack with an adjacent Unicorn aura (a list on the stack record)
gets `f × 0.8` (once, however many Unicorns). The cast rolls `random(1, 100) ≤ trunc(f × 100)`
to affect the stack (the call site at 0x5A02EA and those in the area and chain handlers), so the hero
resistance adds to the creature's and the aura multiplies (dispute M-2), and single-precision
values make 10 % hero resistance a 89 % chance to be affected and 5 % a 94 % chance (quirk
Q-0005). A negative spell never affects a stack of the caster's side through this test, a
positive one never an enemy (Chain Lightning and the area spells are neutral).

**Targets and the aimed stack's roll** (re-read in E04b T2e, 2026-10-06, after batch 2a refuted
the first reading). The cast routine (0x59FE30; arguments spell, target hex, caster kind (0 a
hero, 1 a creature), second hex) rolls at 0x5A02EA only for the stack on the target hex of a
spell that *takes a target stack*: 0x59E060 (spell, mastery) says so for flag `0x10`, for `0x40`
below Expert and for `0x20` below Advanced (Forgetfulness, quirk S-10); never for a creature's
cast. If that stack resists, a message names it and the spell ends. An area spell (`0x80`) has no
such roll: each stack in its area rolls once in the area handler, silently (batch 2a R3: the log
names no resisting stack). The second hex is Teleport's destination, Sacrifice's victim, or the
stack a Magic Mirror (0x4487F0, chance per stack) redirects a spell to, chosen at random among
the caster's own stacks except arrow towers (that stack rolls at the same call site). The
targeting cursor (0x59F700, used by the hero's spell book at 0x59ED5A–0x59F04B) accepts a hex
outside the two side columns (`hex mod 17` not 0 or 16) that the target test 0x5A39C0 accepts:
for a spell that takes a target stack (flags `0x20070`), a living stack on the hex with `f > 0`
(so a stack that would always resist is no target); Remove Obstacle, Fire Wall and Force Field
have their own hex tests (T5); every other hex is accepted, so an area spell may be aimed at an
empty hex, an obstacle or an immune stack's hex (batch 2a R3b, R3c). For Chain Lightning (spell
19) the cursor, the book's "any target" check and the creature casters' (0x5A40D0) also refuse a
stack of the caster's side (0x59F7BF; batch 2a R1). The single-target spells and the battle-wide
ones (24–26) need at least one stack the target test accepts before the cursor opens; the area
spells do not. 0x5A8090 also returns 0 for spells above level 1 on battlefield type 2 (Cursed
Ground, E04c) and checks a worn artifact 0x53 (Recanter's Cloak, T8) for spells above level 2.

### 7.9 Code reading: timed spells (E04b T3a, 2026-10-06)

Read for T3a (Q29: insight only, nothing copied). Offsets are in the 0x548-byte stack record
(§7.7): +0x4C creatures alive, +0x84 creature flags (`0x40` war machine), +0xC4 speed, +0xC8
attack, +0xCC defense, +0xD0/+0xD4 minimum and maximum damage, +0x194 the number of active
spells, +0x198 `i32 duration[81]` (a spell is active while its duration is above 0), +0x2DC
`i32 mastery[81]`, from +0x450 one stored value per spell.

**Casting.** For a hero's cast the routine (0x59FE30) takes the side's spell power and, for a
spell with flag `0x4`, adds 0x4E4DB0: +1 Collar of Conjuring, +2 Ring of Conjuring, +3 Cape of
Conjuring, +50 Ring of the Magi, each counted once and a part also while it is joined in the
combination (so the Ring of the Magi gives 56). That sum is the **duration**. The single-target
handler (case 17 of the switch at 0x5A2838, shared by the timed spells; Bloodlust's own handler
differs only in its animation) applies the spell to the target stack through
0x4448F0 (spell, duration, mastery, hero); the mass handler 0x5A66D0 does it for every stack of
both sides, attacker's first, by record, skipping a stack under Hypnotize (+0x288, T4), each
after the target test and a roll `random(1, 100) ≤ trunc(f × 100)` (§7.8; so positive spells
also draw once per stack).

**Applying a spell to a stack** (0x4448F0). Durations are fixed for Disrupting Ray, Berserk and
Bind (255) and Frenzy (1 or 2, by whether the stack is the active one); otherwise the given
duration. If the spell is already active, the duration becomes the larger of the two, the
stored mastery the higher, and **nothing else changes** (the value of the first cast stays).
Otherwise the active count rises, duration and mastery are stored, and the value `v =
effect[mastery]` (the spell table, from `SPTRAITS.TXT`; plus the hero's specialty bonus 0x4E5FF0
for Bloodlust, Precision, Weakness, Stone Skin, Prayer, Fortune and Haste, §7.12) is used per
spell: Bless removes Curse and stores `v`; Curse removes Bless and stores `v`; Bloodlust and
Precision store `v`; Weakness lowers `v` to the current attack, stores it and subtracts it from
the attack; Stone Skin stores `v` and adds it to the defense; Prayer stores `v` and adds it to
attack, defense and, unless a war machine, speed; Mirth, Sorrow, Fortune and Misfortune store
`v`; Haste (not on a war machine) removes Slow, stores `v` and adds it to the speed; Slow (not on
a war machine) removes Haste and stores `v / 100` as `f32`. No other pair removes each other:
Bloodlust and Weakness, Mirth and Sorrow, Fortune and Misfortune can be active together.
Disrupting Ray has its own cast handler (0x5A1A53): defense − `v` (and the specialty bonus),
at least 0, at every cast, then 0x4448F0 marks it active.

**Reading the values.** Attack (0x442410): +0xC8, plus Precision's `v` on a ranged attack or
Bloodlust's `v` on a melee one while active (Slayer, T3b). Defense (0x442590): +0xCC (Frenzy
and the Behemoths, T3b). Speed (0x448CD0, used for movement and the turn order): +0xC4; while
slowed `max(1, trunc(f32(speed) × factor))` (0 for a war machine). Base damage per creature
(0x443180): while blessed `max + v` for every creature (no roll), else while cursed `max(1, min
− v)`, else the roll; a Ballista's range is already the hero-scaled one. Luck (0x43DF30): the
base luck, clamped to ±3 by its own function, + Fortune's `v` − Misfortune's `v`, then the
Halfling minimum; morale (0x43E003): 0 for creatures without morale, else the base morale clamped
to ±3 + Mirth's `v` − Sorrow's `v`, then the Minotaurs' minimum and Spirit of Oppression; the
rolls clamp both to ±3 again (0x4649AA, 0x43F936).

**Ending.** At the start of every round (0x4760xx calls 0x447120 for every stack record of both
sides, attacker's first; skipped in the tactics phase) each active spell of a living stack, in
spell order, except Frenzy, either ends (duration 1) or loses a round. A stack that dies loses
all its spells (0x444120). Ending a spell (0x444510) restores what it changed: Weakness adds
its stored value back, Stone Skin subtracts its value from the defense (not clamped, so the
defense can go below 0 after a Disrupting Ray), Prayer subtracts from attack, defense and
(unless a war machine) speed, Haste from speed; the others only stop being read. **Disrupting
Ray is never ended**: 0x444510 returns at once for it, so its duration stops at 1 and the
defense stays lowered for the rest of the battle.

### 7.10 Code reading: physical damage and the T3b1 spells (E04b T3b1, 2026-10-06)

Read for T3b1 (Q29: insight only, nothing copied; survey §15's reading list). Offsets as in §7.9.

**Physical damage** (0x443F40; the shot routine 0x43F900 calls it for shots, 0x440310 for melee
strikes and retaliations; 0x442A50 and 0x443D90 are the AI's estimates of the same). Three
stages, each **truncated**, the last at least 1:

1. *Bonuses as an integer* (0x443840 around 0x443320; base `b` from 0x443160, Bless and Curse
   included): `B = 0`; jousting (Cavalier, Champion; not against Pikemen and Halberdiers)
   `B = trunc(b × hexes × 0.05)`; a lucky strike (+0x70, set by the luck roll) `B += b`; if attack
   ≥ defense (0x442410, 0x442590), `r = min((attack − defense) × 0.05, 3.0)` stored as a double
   and `B = trunc(b × r + B)`; elemental pairs (Air/Storm against Earth/Magma, Earth/Magma
   against Air/Storm, Fire/Energy against Water/Ice, Water/Ice against Fire/Energy) `B += b`;
   hate (Angels/Archangels and Devils/Arch Devils, Genies/Master Genies and Efreet/Efreet
   Sultans, Titans and Black Dragons, each way) `B += b / 2` (integer); with a hero on the
   striker's side (the other side's while it is hypnotized), `B = trunc(o × b + B)` with `o`
   Archery for a shot or Offense for a melee strike (0x4E4160, 0x4E42B0; single-precision rates;
   then a Bless specialist's bonus, §7.12); then a Dread Knight's death blow (20 %) `B += b` and the
   Ballista's double damage (by the hero's Artillery) `B += b`. The damage so far is `b + B`.
2. *Times the defense and penalty factor* (0x443B90): `f = 1`; if defense > attack, `f = max(1 −
   (defense − attack) × 0.025, 0.3)` stored as a double; × 0.5 for a Psychic Elemental against a
   mind-immune creature and for a Magic Elemental against a Magic Elemental or Black Dragon; for
   a shot × 0.5 behind a wall (sieges, E04c) and × 0.5 beyond the full-damage distance (0x467600);
   for a shooter's melee strike × 0.5 unless it has no melee penalty; then the Blind and Paralyze
   retaliation factors (+0x4C0/+0x4C1, T4/T7). Truncated.
3. *Times the target's protection factor*: `g = 1`; Air Shield's stored factor for a shot,
   Shield's for a melee strike (retaliations included); × 0.5 while petrified (spell 70); × `(1 −
   Armorer)` of the target's hero (0x4E4310: the single-precision rate, grown for a specialist,
   at most 1). Truncated, at least 1.

The constants 0.05, 0.025, 0.3 and 3.0 are doubles (0.05 and 0.025 slightly above the
decimals); only the hero rates and the spells' stored factors are single precision. The x87
precision control is never set by the game's own code (only the runtime's `_ftol` changes the
rounding mode), so operations round to the thread's default, which on Windows is 53 bits
(inference; 24 bits is refuted by batch 1a R3, 53 and 64 bits both fit batch 1a and differ in
other cases, e.g. 4 Angels against one more defense point: 195 at 53 bits, 194 at 64).

**Applying the T3b1 spells** (0x4448F0, cases by spell): Shield, Air Shield and the four
Protections store `v / 100` as `f32` (Shield +0x4B8, Air Shield +0x4BC, Protection from Air,
Fire, Water, Earth +0x4A8, +0x4AC, +0x4B0, +0x4B4; Fire Shield +0x4A0, T3b2); Slayer stores the
caster's **mastery** (+0x48C), not a value; Counterstrike stores `v` (+0x494) and adds it to the
retaliations left this round (+0x454) at once. As in §7.9, a re-cast changes only the duration
and the stored mastery, so Slayer's creature set and Counterstrike's count stay those of the
first cast. Ending (0x444510) gives nothing back for any of them: Counterstrike's extra
retaliations stay until the next round start.

**Slayer** (0x442410, the attack of a strike against a target): while active, +8 attack
(a constant in the code; the table's value is 8 at every mastery) against a creature with flag
`0x80` (king 1) at any mastery, **none included**, with `0x100` (king 2) from Advanced, with
`0x200` (king 3) at Expert; melee and shots alike; then a Slayer specialist's bonus (T3c).

**Retaliations** (0x447120 at every round start, 0x43D970 at the battle start): 1, Griffins 2,
Royal Griffins 5000 (unlimited); plus Counterstrike's `v` while it is active; 0 for war machines.
The round start does this **before** the durations fall, so in the round in which Counterstrike
runs out the stack still has its extra retaliations. A retaliation uses one (0x441DF7; a
petrified stack does not retaliate).

**Protections** (0x5A7BB0, confirmed): the damage as `f32` times the stored factor, truncated,
for the first of the spell's schools, checked earth, air, fire, water, that has its protection
active; a spell without a school is unchanged. So with `f32(0.7)` slightly below 0.7, Basic
protection turns a damage of 100 into 69.

**Dispel** (case at 0x5A15CE): on a target stack (below Expert) it ends every active spell but
Poison (71), each through 0x444510, so Disrupting Ray stays; at Expert it does the same for every
stack of both sides that it can affect (0x5A8090 above 0, **no resistance roll**), then removes
every battlefield object whose type has one of the bits `0x3C` (the spell obstacles: Fire Wall,
Force Field, Quicksand, Land Mine; T5). Which stacks it can affect (0x5A8090 for spell 35, re-read
in E04b T3b1b): 0 if the stack's controlling hero (0x442690: the other side's while it is
hypnotized) wears the Sphere of Permanence (artifact 92); below Advanced (0x4E5080) 0 for a stack
of the other side; below Expert 0 for a stack without an active spell (a duration above 0 among
ids 10–80); otherwise **1, returned before the immunity, resistance and aura tests**: no creature
is immune to Dispel and none resists it (the aimed stack's roll at 0x5A02EA always passes; the
mass version does not roll). Magic Mirror never redirects Dispel (0x59EE99). Dispel Helpful
Spells (78) has a twin test (the Sphere, then a stack with an active spell of positive type) and
handler (0x5A177E: ends the active spells whose table type is positive; T7).

**Cure** (case at 0x5A17E6; 0x446500 per stack): first resets Slow's stored factor and the
hit points (an aged stack's are recomputed, T7), then ends Curse, Weakness, Sorrow, Misfortune,
Slow, Berserk, Hypnotize, Forgetfulness, Blind, Stone Gaze, Poison, Disease, Paralyze and Age
(not Bind, not Disrupting Ray, not Frenzy), then heals the top creature only (lost hit points
− (power × factor + effect[mastery] + a Cure specialist's bonus), at least 0; no creature comes
back).
At Expert it does so for every own stack it can affect that is not hypnotized. No test of the
target's health or spells: Cure may be cast on a healthy stack without negative spells.

### 7.11 Code reading: Fire Shield and Frenzy (E04b T3b2, 2026-10-07)

Read for T3b2 (Q29: insight only, nothing copied). Offsets as in §7.9.

**Fire Shield** (spell 29; one target, no mass version): 0x4448F0 stores `v / 100` as `f32`
(+0x4A0); ending gives nothing back. The factor getter 0x443130 returns that value while the
spell is active, else 0.2 for an Efreet Sultan (creature 53, its own shield; T7), else 0. The
physical damage routine 0x443F40 computes the reflection right after stage 1 (§7.10), when its
caller passes an out pointer: 0x422440 returns 0 if the target has neither the spell nor the
Sultan's shield or if the striker is fire-immune (creature flag `0x4000`); otherwise `x =
min(stage 1, the target's hit points before the strike)` (1 for a stack with the stack flag
`0x800000`, a clone, T6), `r = trunc(f32(x) × factor)`, then `r` passes the spell-damage
modifiers 0x5A78E0 for spell 29 with the **target's** controlling hero as the caster (the Orb of
Fire, Sorcery and a Fire Shield specialty, §7.8) and the striker as the stack (the creature's
own factors, the striker's Protection from Fire). No immunity, resistance or aura test. The shot
routines pass the out pointer but drop the value: **shots are never reflected**. The melee
strike routine 0x441610 (attacks, retaliations and second strikes alike) keeps the value of the
attacked stack only (a breath attack's second stack reflects nothing); for a multi-headed
striker (creature flag `0x80000`, the attack-all-adjacent loop 0x440310) it sums the values of
every stack struck. After the strike and its kills (0x440BC0), a positive sum damages the
striker (0x4409C0 → 0x444090) before the retaliation (0x441DC5), which follows only if the
struck stack lives (0x441DD4; whether a striker killed by the reflection is still struck back
was not read: the engine never retaliates against a dead stack).

**Frenzy** (spell 56; one target, a mind spell): 0x4448F0 sets its duration to 1 if the target is
the active stack, else 2 (whatever the caster's power), and stores `v / 100` as `f32` (+0x498).
The round start's countdown skips it (§7.9); the stack's turn start 0x446E30 (once per round:
the flag +0x4F0, cleared at the round start 0x447120) counts it down: above 1 it loses a round,
else it ends. So it lasts through the stack's next turn. **Attack** (0x442410, last, after
Slayer): `trunc(f32(attack) + f32(d) × factor)` with `d` the frenzied stack's own defense as the
struck stack sees it (0x442590 with the struck stack as the other side, flag 0): +0xCC (the
Defend bonus included) reduced by a Behemoth's or Ancient Behemoth's share when the **struck**
stack is one (quirk S-16, confirmed), and the siege moat's −3 (E04c). **Defense** when struck
(0x442590 flag 1, used by the damage stages 0x4433A8 and 0x443BBD): 0 while frenzied.

**Behemoths** (0x442590, read for S-16): against a Behemoth (creature 96) the defense is `D −
trunc(f32(D) × f32(0.4))`, against an Ancient Behemoth (97) `D − trunc(f32(D) × f32(0.8))`: the
share taken is truncated, so the defense left rounds up (D = 7: 5 and 2), not down as Q19 chose
(4 and 1). The user (2026-10-07): follow it under quirk Q-0005, keep Q19 in strict-fixed.

### 7.12 Code reading: spell specialists (E04b T3c, 2026-10-07)

Read for T3c (Q29: insight only, nothing copied). The specialty bonus 0x4E5FF0 (on a hero; its
arguments are the spell, the 0-based creature level `c` of a stack (+0x78) and a value `t`)
returns 0 unless the hero's specialty (§7.2) is of kind 3, a spell, and names this spell. Then
it switches on the spell (a byte table at 0x4E60E8 for spells 13–55, jump table 0x4E60CC):

| Spell | Bonus |
|---|---|
| Fire Wall (13) | `t` (Luna) |
| Magic Arrow (15) | `t >> 1` (Ciele) |
| Bloodlust, Precision, Weakness, Stone Skin, Prayer, Haste (43–46, 48, 53) | by creature level, the `i32` table at 0x63EAA8: 3, 3, 2, 2, 1, 1, 0 for levels 1–7 |
| Disrupting Ray (47) | 2 |
| Fortune (51) | `3 − t`: the value becomes 3 |
| Slayer (55) | by creature level, the `i32` table at 0x63EAC4: 4, 3, 2, 1, 0, 0, 0 |
| every other spell | `ceil((hero level div (c + 1)) × t × 0.03)`: the division and the product in integers, then the double 0.03 (§7.8) |

The callers decide which value, which hero and which stack:

- **Applying a timed spell** (0x4448F0, §7.9), with the casting hero (none for a creature's
  cast) and the target: Bloodlust, Precision, Weakness (before the cut to the current attack),
  Stone Skin, Prayer, Fortune and Haste add the bonus of `t = v` to the stored value. No other
  spell calls it there: Bless, Curse, Mirth, Sorrow, Misfortune, Slow and the spells of §7.10 and
  §7.11 get nothing at the cast.
- **Disrupting Ray** (0x5A1A53): defense − `v`, then − the bonus (2), then at least 0.
- **Cure** (0x446500, §7.10): the heal `t = power × factor + effect[mastery]` plus the bonus of
  the caster for the target.
- **Bless** (stage 1 of physical damage, 0x443320 at 0x443776, §7.10): with a hero on the
  striker's side (the other side while the striker is hypnotized) and Bless active on the
  striker (its duration +0x23C), `B += bonus(41, the striker's level, b)` right after the
  Offense or Archery term. The hero is the striker's controller at the strike, not the caster.
- **Slayer** (the attack, 0x442410 at 0x4424F7): when Slayer counts against the struck stack,
  attack + 8 + `bonus(55, the striker's level, 8)` with the striker's side's hero (the other side
  while hypnotized).
- Elsewhere: the damage spells (0x4E5760, §7.8); 0x5A196B and 0x5A84FB, the cast values of the
  raising spells (T6); 0x5A839F, Hypnotize's hit-point limit (T4); 0x423D93 outside battle (not
  read).

VCMI 1.7.5 agrees on the creature-level table (`SPECIAL_PECULIAR_ENCHANT`,
`lib/spells/effects/Timed.cpp`), Slayer's `max(5 − level, 0)` (`DamageCalculator.cpp`),
Disrupting Ray's 2 and Fortune's fixed 3 (`config/heroes`); it gives a Bless specialist a
damage premium of 3 % per hero level divided by the stack's level among the additive bonuses
(`GENERAL_DAMAGE_PREMY`) instead of a separate term rounded up.

### 7.13 Code reading: Anti-Magic and Magic Mirror (E04b T4a, 2026-10-07)

Read for T4a (Q29: insight only, nothing copied). Both spells go through the timed handler
0x5A107F like Shield (one target, no mass version), so their duration is the caster's power plus
the duration bonus and a re-cast keeps the first value (§7.9).

**Anti-Magic** (spell 34; effect 4, 4, 5, 6 by mastery). 0x4448F0 stores `v` as an `i32`
(+0x460) and then, for every spell of the table whose type is negative (−1), whose level is
below `v` and which is not a creature ability (flag `0x8`), calls the end routine 0x444510 for
the target: the active negative spells of levels 1–3 end (1–4 at Advanced, 1–5 at Expert),
Disrupting Ray excepted (it never ends, §7.9); Sorrow and Berserk (level 4) survive a Basic
Anti-Magic. Positive spells stay. While the spell is active (its duration +0x220 above 0), the
spell-effect test 0x5A8090 returns 0 for every spell whose level is below the stored `v` and
which is not a creature ability, **whatever its polarity**: the stack is immune to its own
side's spells of those levels too (Bless, Cure, a second Anti-Magic). The test comes after
Cursed Ground and Recanter's Cloak and after Dispel's and Dispel Helpful's own tests (they
return first, so Dispel ends Anti-Magic), before the dead-stack, polarity, per-spell and
creature tests (§7.8), so the Orb of Vulnerability does not lift it. The per-spell case of
0x5A8090 refuses Anti-Magic on a clone (stack flag `0x800000`, T6). 0x5A298E words the
message of a spell that fails on it. VCMI 1.7.5 (`config/spells/timed.json`) agrees on the
levels (`LEVEL_SPELL_IMMUNITY` 3, 3, 4, 5) and dispels every negative and neutral effect
whatever its level.

**Magic Mirror** (spell 36; effect 20, 20, 30, 40). 0x4448F0 stores `v` as an `i32` (+0x4E4).
The chance getter 0x4487F0 returns `v` while the spell is active (duration +0x228), else 0; for
a Faerie Dragon (creature 134) the larger of that and the table's `effect[0]` of spell 36 (20;
T7). The redirect is decided where a player's cast is entered (0x59EC50, the spell book and the
creature casters' button, at 0x59EE7A): for a cast that takes a target stack (0x59E060: the
single-target spells and the mass spells below their mass mastery, §7.8) whose target stack is
not Dispel's, belongs to the other side and is alive, the game draws `r = random(1, 100)`
(drawn for every such cast, with or without a mirror) and redirects when `r ≤ chance`. The new
target is drawn uniformly (0x50C6E0/0x50C740: draws without replacement until one fits) among
the caster's side's stack records that are alive and not an arrow tower (creature 149): war
machines, summoned and cloned stacks count. Its hex is the cast's second hex. The cast routine
0x59FE30 then shows the mirror's animation on the first target, takes the stack on the second
hex (looked up for the other side, 0x5A3950) as the target and rolls it at 0x5A02EA with the
reflected flag set: 0x5A8090 skips its polarity tests for it (a negative spell may affect a
stack of the caster's side), but every immunity, Anti-Magic, the hero resistance of that stack's
own side and the Unicorn aura apply; if it resists, the spell ends. The spell's value and the
caster's bonuses are unchanged; Chain Lightning starts its jumps at the new target. The first
target never rolls. Area spells, mass versions and battle-wide spells are never redirected,
nor is anything cast at an own stack. 0x5A8090's case for Hypnotize (60) returns 0 for a stack
with Magic Mirror active: such a stack cannot be hypnotized (T4c).

The computer player's casts are entered without 0x59EC50 (0x43CAD1 writes the action itself
with the candidate's second hex); its evaluation 0x43A78F only scales a candidate's value by
`(50 − chance) / 50`, so the computer player's spells are never redirected. This concerns the
original's AI only: every cast here is a player's command (ADR-0004) and follows the player's
path.

VCMI 1.7.5 (`lib/spells/BattleSpellMechanics.cpp`, `isReflected`, `getRandomUnit`) redirects a
hero's direct negative spell with `random(0, 99) < chance` to a random valid unit of the
caster's side other than war machines (`SIEGE_WEAPON`), and marks an immune new target as
resisting.

### 7.14 Code reading: Forgetfulness, Blind, Berserk and Hypnotize (E04b T4b/T4c, 2026-10-07)

Read for T4b and T4c (Q29: insight only, nothing copied). Offsets as in §7.9; a spell is active
while its duration `+0x198 + 4 × id` is above 0 (Berserk +0x284, Hypnotize +0x288, Forgetfulness
+0x28C, Blind +0x290; Stone Gaze +0x2B0 and Paralyze +0x2C0, T7, share Blind's code). All four
are mind spells (flag `0x400`) that spare war machines (`0x1000`). The creature test 0x44A4D0
(§7.8) has absolute cases for them, before the Orb of Vulnerability: a stack whose controlling
hero wears a Pendant (Berserk artifact 100, Hypnotize 105, Forgetfulness 107, Blind 101; T8) is
immune; **Forgetfulness affects only shooters** (creature flag `0x4`); **Blind spares the
Troglodytes (creatures 70, 71) and every undead** (`0x40000`), even under the Orb.

**Forgetfulness** (spell 61; effect 0 at every mastery; flag `0x20`: one target below Advanced,
the mass version from Advanced, §7.8, quirk S-10). Both go through the timed handler 0x5A107F
(the mass one through 0x5A66D0, which skips hypnotized stacks, each stack tested and rolled),
duration as §7.9. 0x4448F0 stores the caster's **mastery** (+0x4C4); as for every spell a re-cast
keeps it (the general mastery array +0x2DC takes the higher one, nothing reads it here), so a
stack under a Basic Forgetfulness can still shoot after an Advanced one. While active:
- the creatures a strike counts (0x443160, the base damage of every strike: shots, melee
  attacks, retaliations): `max(1, n / 2)` instead of `n` (`n / 2` truncated) for a stack with the
  shooter flag, at any mastery. The in-game text of None and Basic says half the stack forgets
  to shoot; the melee and retaliation part is quirk S-09 (the rule in every profile, Q53);
- from a stored mastery of 2 (Advanced) the stack cannot shoot: every can-shoot test (0x447D92,
  0x4429B2, 0x445556 for the berserk turn, 0x448A47 and 0x448C90 for the cursor) answers no, so
  it fights in melee with the shooter's melee penalty (§7.10) and the halved count.
Ending gives nothing back.

**Blind** (spell 62; effect 50, 50, 25, 0; one target). 0x4448F0 stores `v / 100` as `f32`
(+0x49C). Duration as §7.9. While active:
- *No turns.* The battle loop (0x465080) picks the next stack by speed among those not done
  (+0x84 bits 21, 25, 26) and, for a stack whose turn has started this round (+0x4F0), only if it
  is not blind, petrified or paralysed; it then runs the stack's turn start 0x446E30 once per
  round (+0x4F0 set; the Defend bonus ends, regeneration, Frenzy's countdown, §7.11) and, if the
  stack is blind, petrified or paralysed, picks again without marking it done. No morale roll, no
  action. A stack whose blindness ends later in the round (it is struck) has its turn started and
  is not done, so it can still act that round.
- *Any damage ends it* (and Stone Gaze and Paralyze): the routine that takes hit points 0x444090
  calls the end routine for spells 62, 70 and 74 every time, whoever deals the damage: strikes,
  shots, damage spells, Fire Shield's reflection.
- *Retaliation.* Before a melee attack (0x441D9C) the struck stack is marked if it is blind
  (+0x4C0) or paralysed (+0x4C1); the strike then ends the blindness. If the mark is set and the
  stored factor is 0 (Expert), the strike routine answers "no retaliation" (0x441C85): it does
  not retaliate and keeps its retaliation. Otherwise it retaliates as usual (if alive, with a
  retaliation left, the attacker not exempt, not petrified), and stage 2 of the damage (0x443CD5,
  §7.10) multiplies the factor by the stored `f32` (blind) or by Blind's Advanced effect / 100
  (paralysed; the smaller of the two if both). The marks are cleared after the retaliation.
Ending gives nothing back. Shots and spells end it without a retaliation.

**Berserk** (spell 59; effect 0, 0, 1, 2; flags: a hex target, `0x80`). The handler 0x5A1D80
collects the stacks on the hexes within the radius of the table 0x642264 (0, 0, 1, 2 by mastery)
around the target hex (0x5A4810, one entry per living stack), tests and rolls each (0x5A8090,
`random(1, 100) ≤ trunc(f × 100)`) and applies it to those that fail. The polarity test of
0x5A8090 (which, for a negative spell, refuses a stack of the caster's side) **makes an exception
for spell 59**: Berserk also affects the caster's own stacks in its area. 0x4448F0 fixes the
duration at 255 (quirk S-17) and ends Hypnotize first. While active:
- every other stack is its enemy and it is every stack's enemy (0x442880 answers yes when either
  stack is berserk);
- its turn is automatic (0x4745DF → 0x4456D0): the candidates (0x445490) are the other living
  stacks except arrow towers: for a stack that can shoot (as above, the Forgetfulness test
  included), by hex distance; otherwise, from the AI pathfinder's map over the whole field
  (0x4B2DA0 → 0x4B3400 with side −1, so every stack is an enemy to reach), the stacks whose hex
  has an attack position the stack can walk to, by the path cost to that position. The nearest
  ones are kept and one is drawn with equal chances (0x50B230). None: the stack does nothing
  (action 12, done without the Defend bonus). A shooter shoots at it (action 7); otherwise
  0x4222C0 walks the path towards it and attacks from the last hex if the attack position is
  within its speed, else only walks as far as it can (action 6, or a plain move);
- after its own attack (the melee attack routine 0x441F55, the shot routine 0x4402F7) it ends; a
  retaliation does not end it. Cure, Dispel and an Anti-Magic of level 5 or more end it.
The wiki's pathing bug (S-11) was not read: it would sit in 0x4B3400. Q53: the engine uses its
own pathfinding; S-11 stays unconfirmed.

**Hypnotize** (spell 60; power factor 25, effect 10, 10, 20, 50; one target). Target test
(0x5A8090 case 60, after the general tests): 0 if the stack has Magic Mirror active (§7.13);
else the limit is `power × 25 + effect[mastery] + the specialist bonus (0x4E5FF0, by the stack's
level, of the limit)`, and the stack qualifies if `hit points per creature × creatures` (the full
hit points, the top creature's wounds not counted) is at most the limit. Duration as §7.9.
0x4448F0 ends Berserk first and rebuilds the stack's Unicorn aura lists (+0x514/+0x524, the
adjacent Unicorns of the same controller; 0x43EA70), as its end does. While active, the stack's
**controller** is the other side (0x442690 and the inline copies `side ^ hypnotized`): the
controller commands its turns (the battle loop sets the acting side to it, 0x465389, so the
controller's hero may cast during that turn), and these use the controller and its hero:
friend and enemy (0x442880), the Offense or Archery and the Bless specialist of its strikes and
the Armorer of strikes on it (stage 1 and 3, 0x443898, 0x443DE6), the hero's resistance and the
Pendants against spells on it (0x5A8090 → 0x44A4D0), Dispel's Sphere test (§7.10), the target's
hero in spell damage and Fire Shield (0x5A0D1C, 0x4224AF), the Unicorn aura. These keep the
real side (+0xF4): the stack's statistics set at the battle's start (its hero's primary skills,
morale and luck), the turn order's tie-break (0x46502A), and the side tests of spells (0x5A82F8):
the owner may cast its positive spells on its hypnotized stack (Cure ends Hypnotize, §7.10) but no
negative ones, the controller the other way round. Mass versions (0x5A66D0, §7.9) skip
hypnotized stacks, and so does Expert Cure, which loops over the caster's own stacks (0x5A1847,
+0x5754 = Hypnotize's duration): an Expert Cure never reaches them (quirk S-13), a single Cure
does. VCMI 1.7.5 (`config/spells/ability.json`, `BattleInfo`) limits Hypnotize by the stack's
remaining health.

### 7.15 Code reading: battlefield objects, Force Field, Fire Wall, Teleport, Remove Obstacle (E04b T5a/T5b, 2026-10-07)

Read for T5a and T5b (Q29: insight only, nothing copied). The cast routine's switch (§7.8) sends
Quicksand (10) to 0x5A0354, Land Mine (11) to 0x5A0536, Force Field (12) to 0x5A0759, Fire Wall
(13) to 0x5A0821, Teleport (63) to 0x5A1B4D and Remove Obstacle (64) to 0x5A1C04; the spell
book's handler 0x59EC50 picks the cursor mode per spell (Quicksand casts at once, Land Mine first
tests the native terrain, T5c).

**Battlefield objects.** The battle manager keeps a vector of objects at +0x13D58 (begin +0x13D5C,
end +0x13D60), 0x18 bytes each: +0 the graphic (0 once removed), +4 the type descriptor, +8 the
anchor hex, +9 the owner side (−1 for the battlefield's own obstacles), +0xA revealed (shown to
both sides), +0xC a value (Fire Wall's and Land Mine's damage), +0x10 rounds left (0: lasts the
battle), +0x14 the animation played when it goes (−1: fade out). A descriptor (0x14 bytes) holds
a count of blocked hexes at +6 and their offsets from the anchor at +8 (`hex + offset`, one hex
to the left when the anchor's row is odd and the result's even: VCMI's rule, BAT-GEO-004).
Placing an object (0x4669B0) sets, on each hex record (stride 0x70 from +0x1C4), flag bits at
+0x10 and the object's index at +0x14; the anchor also gets bit `0x1` and the index, even where
the descriptor does not block it (28 of the 55 usual obstacle types in the harvested `base` pack
leave out the offset (0, 0)). Read so, such an anchor hex may be stood on and walked through, but
takes no Fire Wall or Force Field (their test wants none of the bits `0x3F`) and Remove Obstacle
aimed at it removes its obstacle (the test wants an index); batch 2c A2 confirmed it (quirk
BAT-QUIRK-3, fix-optional; OPEN-QUESTIONS Q55, BAT-GEO-003, BAT-MAG-034). The flag bits:
`0x2` blocked (natural obstacles and Force Field), `0x4` Quicksand, `0x8` Land Mine, `0x10` Fire
Wall, `0x20` Force Field; `0x3C` are the spell objects. Only `0x2` blocks movement and standing
(0x469A10); Quicksand, Land Mine and Fire Wall hexes may be entered and stood on. The usual
obstacles (0x466010) become objects (side −1, revealed, permanent); the absolute ones (0x466939)
only set `0x2` on their hexes and have no object, so nothing can remove them. Removing an object
(0x466B30, re-read in T5b) clears all six bits `0x3F` and the index (−1) of each hex its
descriptor names, then the bit `0x1` and the index of its anchor hex (the bit `0x2` stays), and
its graphic: a hex the removed object shared with another (a later object blocking an earlier
one's anchor, or two objects with one anchor hex) leads to no object afterwards. At every round start
(0x476152, after the stacks' effects, §7.9) each object with rounds left loses one; at 0 it is
removed.

**Force Field** (12): one object over 2 hexes (offsets 0, −16) below Advanced, 3 (0, −16, −34)
from Advanced (descriptors 0x63CF18, 0x63CF2C), for both sides alike: the target hex, its upper
right neighbour, and that one's upper left; revealed, 2 rounds, flags `0x22`. **Fire Wall** (13):
one object per hex, 2 hexes below Advanced and 3 from it: the target hex, its upper right
neighbour for the attacker's hero or upper left for the defender's, and two rows above the
target (the upper left, resp. upper right, of the second); each revealed, 2 rounds, flag `0x10`,
with the value `power × power factor + effect[mastery]` (0x5A78E0 called without a hero or a
stack: no bonus yet). The target test 0x5A39C0 wants every hex of either inside the field,
outside the side columns, without any object bit (`0x3F`) and without a stack.

**Entering a hex** (0x46A570, for a stack's hex and, if it is a two-hex stack, its second hex,
each at most once per move: a per-hex array cleared at the start of a move with the starting
hexes already marked, 0x46A520) calls in turn the Fire Wall trigger 0x59FCD0, the Land Mine
trigger 0x59FB40 and the moat's damage 0x469E50 (sieges, E04c; Quicksand acts in the walker,
§7.16). A walker (0x441FA0) enters each hex of the path its pathfinder chose (0x5239D0 →
0x4B3400, the pathfinder of survey S-11, which avoids Quicksand its side can see and nothing
else, §7.16) and stops when it dies; a flyer (0x4B49C0) and a
teleporting creature (0x4B4E90) enter only the hexes they land on. After an action that does not
move the stack (defend 0x4798F3, wait 0x479AD6, a shot or melee attack from where it stands
0x4792D7, 0x4789C7) the array is cleared and the stack's own hexes are entered again; a stack that
moved and then attacked is not (0x478900 skips it). After a shot the battle's end is checked first
(0x4762F0 → 0x465830) and nothing is entered once it is over; after a melee attack from where the
stack stands no such check was found (the engine skips the burn once the battle is over there
too). Arrow towers (creature 149) never enter.

**Fire Wall's damage** (0x59FCD0): for a living stack on a Fire Wall hex, if 0x5A8090 (spell 13,
the object's owner as the caster's side) gives `f > 0` (Fire Wall is neutral: either side's
stacks; immunities, Anti-Magic and the stack's hero's resistance count only through `f > 0`:
**no resistance roll**), 0x5A78E0 with the owner's hero as the caster, the stack's controlling
hero as the target's, and the stack: the caster's bonuses (the Orb of Fire, Sorcery, Luna's +100 %,
§7.12), the creature's factor, Protection from Fire (§7.8), then the hit points (0x444090, which
also ends Blind and the like, §7.14). The object stays.

**Teleport** (63): a positive stack spell (flag `0x1000`: no war machine). The handler moves the
stack to the second hex (0x5A1B4D: off its hexes, the destination adjusted for a two-hex stack by
0x446C40, placed) and enters no hex: no trigger fires. The destination test 0x5A3700: not the
stack's own hex, a position where it fits (0x446C40: inside the field, outside the side columns,
not blocked, no other stack, for a two-hex stack also its second hex); in a siege, below Advanced
not across the wall and below Expert only where 0x467840 allows (E04c); outside sieges every
mastery is alike.

**Remove Obstacle** (64; flag `0x100`): the target test wants a hex with an object (index ≥ 0)
whose flags allow it at the caster's mastery (table 0x5A3C64): below Advanced no spell object
(`0x3C`); Advanced also Fire Wall (`0x10`); Expert any object. The handler removes the whole
object (0x466B30), a single Fire Wall hex being an object of its own.

**Expert Dispel** (§7.10) removes every object with a spell bit (`0x3C`), whoever cast it.

### 7.16 Code reading: Quicksand, the natives' sight, the walker's stops (E04b T5c1, 2026-10-07)

Read for T5c1 (Q29: insight only, nothing copied).

**The natives' sight.** When a stack is set up (0x43D7DD), its byte +0x4D8 says whether it fights
on its native terrain: its faction's (+0x74; the table 0x643698 maps factions to terrains, −1 to
none) is the battlefield's (+0x5394), except on special battlefield 2 (+0x53C0, E04c), where no
stack is native; while +0x1F698 of the game object 0x6994E8 is 0 (likely the RoE game version, not
read) the elementals (creatures 112–115) have no faction. At the battle's
start (0x46402C) the battle sets one byte per side at +0x1329C: whether any of the side's stacks has
that byte. Nothing writes it again; it is read only when Quicksand or a Land Mine is placed and by
Land Mine's cast test (T5c2): an object placed by side `s` gets the revealed flag (+0xA, shown to
both sides) if the other side's byte is set. Natives that die or appear later change nothing
(BAT-QUIRK-1; VCMI tests the living natives each time).

**Quicksand** (10; the spell book casts it at once, no target, 0x59ECDF): the pits by mastery come
from the table 0x642214 (4, 4, 6, 8), not from the spell's effect values. A sampler over the hexes
0..186 (0x50C6E0, 0x50C740) draws each hex at most once, uniformly among those not yet drawn; a
drawn hex takes a pit if it is outside the side columns, has none of the object bits `0x3F`
(natural obstacles, anchors and every spell object, hidden ones included), no living stack (hex
record +0x18) and no corpse (+0x1C counts the dead stacks on it; a dead two-hex stack counts on both
hexes, 0x444380); otherwise the next hex is drawn, until the pits are placed or the hexes run out.
Each pit is an object of its own (descriptor 0x63CEE8: one hex), owner the casting side, revealed
as above, value 0, lasting the battle (rounds 0), flag `0x4` (0x4669B0).

**The walker's stops.** Before a walker moves (0x4B3290, called from the pathfinder 0x4B3400) the
battle marks the hexes where it must stop: in a siege the moat, and every Quicksand hex whose
object belongs to the stack's side (+0xF4, the real side even when hypnotized) or is revealed; the
stack's own hexes are left out. Flyers (stack flag `0x2`) and Devils and Arch Devils (creatures 54,
55) get no stops. The pathfinder lets a path end on a stop but not go on from it (a stop costs the
rest of the move). The walker (0x441FA0), at each step of its path, before it moves: if the new
hex of its head or, for a two-hex stack, of its tail is a stop, this step is the last; else if it
has Quicksand, the walker reveals that pit (+0xA = 1) and this step is the last; then it moves and
enters the hexes (§7.15: Fire Wall, Land Mine, the moat). Either way the walk returns false, and a
move and attack (0x478971) then skips the attack, also when the pit was the move's destination and
seen (BAT-QUIRK-2). A flyer (0x4B49C0) and a teleporting creature (0x4B4E90) never test the pits;
Teleport places the stack without entering. Standing on a pit has no further effect: the stack
leaves it freely on its next turn.

**Information the original gives away** (fixed, Q-0002): Fire Wall's and Force Field's target test
(0x5A39C0) refuses a hex with the other side's hidden pit, so the cursor shows it; the stops of a
hypnotized stack are its real side's, so its controller sees where the owner's pits lie.

### 7.17 Code reading: Land Mine (E04b T5c2, 2026-10-07)

Read for T5c2 (Q29: insight only, nothing copied).

**The spell book** (0x59ECEE) refuses Land Mine (11) while the other side's natives byte
(+0x1329C, §7.16) is set: it shows a message (0x4F6570) and sets no cast mode, so nothing is cast
or paid. Otherwise it casts at once, as Quicksand (0x59ECDF: no target). No other code reads the
byte for Land Mine; the AI never casts it (wiki). A side whose stacks on their native terrain are
all immune to the spell still has the byte set (0x46402C tests only the stacks' native flag
+0x4D8, set at 0x43D7DD without an immunity test), so the book refuses Land Mine against it too;
the wiki says the opposite (experiment candidate, `docs/experiments/README.md`).

**The cast** (0x5A0536): the mines by mastery come from the table 0x642224 (4, 4, 6, 8), the
damage from the spell's values: `power × power factor + effect[mastery]`, passed through 0x5A78E0
without a hero or a stack (no bonus yet, as Fire Wall's, §7.15). The hexes are drawn as
Quicksand's pits (§7.16: the same sampler and test, outside the side columns, none of the object
bits `0x3F`, no living stack, no corpse). Each mine is an object of its own (descriptor 0x63CF00:
one hex), owner the casting side, revealed if the other side's natives byte is set (never, as the
book refuses then), with the damage as its value, lasting the battle (rounds 0), flag `0x8`.

**The trigger** (0x59FB40), called by 0x46A570 for each hex a stack enters, after Fire Wall's
(§7.15): nothing for a dead stack or a hex without bit `0x8`; nothing for a stack whose side
(+0xF4, the real side even when hypnotized) owns the mine, nor when the mine is revealed; nothing
when 0x5A8090 (spell 11, the owner as the caster's side) gives `f ≤ 0`, so an immune stack crosses
and the mine stays as it was. Otherwise the damage is Fire Wall's with spell 11 (0x5A78E0 with
the owner's hero as the caster, the stack's controlling hero and the stack: the caster's bonuses,
the creature's factor, Protection from Fire; no resistance roll), 0x444090 applies it, the mine
is removed (0x466B30) and, as after Fire Wall's damage, the Phoenixes' rebirth is checked
(0x469440, creature 131; E04b T7). The walker ignores the trigger's result (0x44237E): a stack
that survives a mine walks on, and a move and attack still attacks. A stack standing on a mine
it did not trigger (Teleport places it without entering; or it was immune then) triggers it when
it acts without moving (§7.15).

### 7.18 Code reading: Resurrection, Animate Dead and Sacrifice (E04b T6a, 2026-10-08)

Read for T6a (Q29: insight only, nothing copied). The cast routine's switch (§7.8) sends
Resurrection (38) and Animate Dead (39) to 0x5A1907 and Sacrifice (40) to 0x5A199C; the spell
book (0x59ED5A for 38 and 39, 0x59F196 for 40) opens the cursor only if some hex passes the target
test 0x5A39C0 (Sacrifice: some hex for the raised stack and some hex for the victim).

**Corpses.** A hex record (§7.15) lists the dead stacks lying on it: their number at +0x1C (at most
14), their sides at +0x20, their record indices at +0x2E and, per corpse, the other hex of a
two-hex one at +0x3C (0 the next hex, 1 the previous, otherwise none). A stack's death (0x444120)
appends it to the list of each hex it covers, unless the hex already holds 14; a stack flagged
sacrificed (`0x10000000`) or summoned (`0x400000`) leaves no corpse. Raising removes it from them
(0x5A7320).

**Which stack a cast raises.** 0x5A3950 finds the stack for spells 38 and 39 (and for Sacrifice's
raised stack) by hex, through 0x5A3CC0 (Resurrection) and 0x5A3F50 (Animate Dead):

- a living stack on the hex (+0x18) is the only candidate: it must be of the caster's side (+0xF4,
  the real side), have the creature flag `0x10` (living; Animate Dead: `0x40000`, undead), fewer
  creatures (+0x4C) than at the battle's start (+0x60), and pass the spell test 0x5A8090;
- otherwise, if the hex is not blocked (bit `0x2`), the corpses from the last added to the first:
  the first of the caster's side with the flag, whose other hex (a two-hex one) has no living
  stack and is not blocked, and which passes 0x5A8090.

0x5A8090 lets only spells 38–40 reach a dead stack (bit `0x200000`), and for 38 and 39 (case
0x5A8467) asks that the stack has fewer creatures than at the start and that the spell's hit
points (below) reach one creature's health (+0xC0), then applies the usual immunity factor
(0x44A4D0: Anti-Magic, the creature's spell-level immunity). A positive spell takes no enemy
stack. So a full stack that is only wounded is no target, and nor is a stack whose creatures
have more health than the spell raises. **0x5A3CC0 tests with spell 38 whichever spell asked**,
so Sacrifice's raised stack must be a valid Resurrection target: Resurrection's level 4 for the
immunities (Gold Dragons, immune to levels 1–4, cannot be raised by Sacrifice; survey S-15) and
Resurrection's hit points at the caster's Earth mastery for the health test.

**Resurrection and Animate Dead** (0x5A1907): hit points `power × factor + effect[mastery]` plus
the specialty bonus 0x4E5FF0 with the target's level (§7.12: `ceil((hero level div (c + 1)) × t ×
0.03)`); temporary when the caster's mastery is below Advanced (Resurrection only). The raise
(0x5A7560, stack, hit points, temporary): the stack's total health (0 if dead) plus the hit points
gives the count `ceil(total / health)` and the top creature's loss, both capped at the start count
(full health then); a temporary raise adds the creatures gained to the counter +0x54, which is
capped at the start count and written nowhere else (0 at setup, 0x43D861; no loss lowers it). A
dead stack comes back on its own hex (0x43EA70 revives it, 0x4687C0 places it) and leaves the
corpse lists; at the end the raise clears the dead bit `0x200000` (0x5A7867) and touches no other
flag, so a stack that had acted this round before it died stays done. Its spells ended at its
death (0x444120 ends every active spell, then sets the dead bit).

**Sacrifice** (0x5A199C): the victim is the living stack on the cast's second hex. 0x5A8090 (case
0x5A8523) wants it living (`0x10`), not summoned (`0x400000`), not undead, with creatures, and of
the caster's side by both its controller (the other side while hypnotized) and its real side; the
raised stack takes the immunity factor of spell 38 as above, the victim that of spell 40 (level
5: no creature's spell-level immunity reaches it). The hit points are `(the victim's creature's
health from the creature table + effect[mastery] + power) × the victim's creatures`: the base
health, not the stack's (+0xC0). The victim takes its whole health as damage (0x444090), is flagged
sacrificed (no corpse) and the raised stack gets the hit points as a permanent raise. No code
compares the victim with the raised stack (the second cursor, 0x59F700, was not read).

**The end of the battle** (0x4774E2, before the armies are rebuilt by 0x4641F0): for each side,
every record 0–19 with a creature type and creatures loses its temporary creatures (`count −
+0x54`, at least 0) and the counts left are summed; if the sum is 0 while some record had
creatures, the last such record keeps **one creature**. The sum counts every record, war machines
and summoned stacks or clones included, which the rebuild 0x4641F0 then leaves out of the army
(alive, not dead, not summoned `0x400000`, not a clone `0x800000`, not a war machine `0x40`, an
army slot 0–6): a hero whose army was raised at Basic but who keeps a Ballista or an elemental is
left without an army (survey S-20). VCMI instead lowers its counter as creatures die
(`CHealth::damage`); the original never lowers +0x54 during the battle.

### 7.19 Code reading: Clone, Summon Elemental and the stacks a battle creates (E04b T6b, 2026-10-08)

Read for T6b (Q29: insight only, nothing copied). The cast routine's switch (§7.8) sends Clone (65)
to 0x5A1D1B, which calls 0x5A6C70 with the target hex, and the four Summon Elemental spells (66,
67, 68, 69) to 0x5A7080 with the creature (114 Fire, 113 Earth, 115 Water, 112 Air Elemental),
the caster's power and mastery. The spell book refuses both while the caster's side has 20 or more
records in use (§7.7).

**Creating a stack** (0x47A100: side, creature, count, hex, flags, animation): the first of the
side's records 0–19 that is empty (creature −1) or holds a dead (`0x200000`) summoned
(`0x400000`) stack with no creatures; none: nothing is created. The hex must hold no living stack,
else nothing is created either (and the caller is not told). The record is set up like any stack
at the battle's start (0x43D8B0, 0x43D730: the creature's table entry, so its flags and none of
the round's, then the side's hero's bonuses through 0x4E6120, the native-terrain bonus, morale and
luck), the given flags are added, and the side's count of records in use (+0x54BC) grows unless a
record was reused. So a new stack acts in the round it appears, and a dead clone's or summoned
stack's record (and so its place in the turn order's tie-break) goes to the next stack the side
creates. 0x43D730 also clears the town of creatures 112–115 unless a game setting
(`[0x6994E8]+0x1F698`, not read) is set, which takes their native-terrain bonus away; this
applies to army stacks too and is left for BAT-TER-001 (experiment list).

**Clone** (0x5A6C70). The target test (0x5A8090, case 0x5A841A) wants a stack that is not a clone
(`0x800000`), has no clone (+0x28 = −1) and whose creature level (+0x78, 0-based) + 1 is at most
Clone's effect[mastery] (5, 5, 6, 7), besides the usual tests of a positive spell (own side,
immunities, Anti-Magic; flag `0x1000` spares war machines). An elemental (summoned, not a clone)
qualifies. The clone's position is searched in rings 1 to 10 around the original: per ring, from
the original's position (+0x38) and, for a two-hex one, also from its other hex (position ± 1 by
the facing +0x44), in the directions 0 to 5 for a stack facing right (the attacker's) and 5 to 0
for one facing left (directions: 0 top right, 1 right, 2 bottom right, 3 bottom left, 4 left,
5 top left), the hexes at distances 1 to the ring's number along that direction, outside the side
columns; the first where the original's creature fits (0x446C40: the hex and, for a two-hex
creature, the hex in front of it by the facing are on the field, outside the side columns, not
blocked, and hold no living stack *or the original itself*) takes the clone. Skipped: right and
left at distance 1 from the position in ring 1, and from the other hex towards the position in
rings 1 and 2 (left for the attacker, right for the defender). No hex: a message, nothing else
(the mana is spent). The clone is created (0x47A100) with the original's creature and count and
flag `0x800000`, then gets `0x400000`, rounds left (+0x2C) = the caster's power + its duration
bonus (0x4E4DB0, as timed spells, §7.9) and the links original +0x28 = clone, clone +0x24 =
original.

Because 0x446C40 counts the original's own hexes as free, a crowded two-hex original can get a
clone on its own hexes: if the clone's position is the original's hex, 0x47A100 creates nothing
and the cast then flags *the original* as summoned with the clone's rounds (it vanishes when they
run out and its army slot is lost after the battle); if only the clone's front hex is the
original's, the two stacks share a hex. Only reachable when every place in ring 1 fails (quirk
ledger BAT-QUIRK-7).

**A clone's end.** At the start of each round (0x447120, per living stack after its spell
durations) rounds left above 0 lose one, and a stack whose rounds left are 0 dies (0x444120).
The damage routine (0x444090) kills every creature of a clone (flag `0x800000`) whatever the
damage. A stack's death (0x444120) resets its original's link if it is a clone and kills its clone
if it has one; a clone, as any summoned stack, leaves no corpse (§7.18).

**Summon Elemental** (0x5A7080). Count: effect[mastery] (2, 2, 3, 4) × the caster's power. Place:
the attacker tries columns 1, 2, …, 15, the defender 15, 14, …, 1; in each column a sampler
(0x50C6E0 with rows 0–10, 0x50C740) draws rows without repetition until one fits a stack of the
creature (0x446C40 for a stack of the side that is no stack on the field), so the row is uniform
among the fitting rows of the first column with one. The side's summoned creature (+0x132A8) is
set to the creature (even if no hex is found), and the stack is created (0x47A100) with flag
`0x400000`. Several casts of one elemental make several stacks.

**The battle's end.** The battle is over (0x465830) when a side has retreated or surrendered
(+0x132B0, +0x132B2) or has no living stack in records 0–19 apart from war machines (`0x40`); a
summoned stack or clone counts as living. The winner (0x4762F0 with 0x4658B0 for each side) is a
side that has a living stack that is neither summoned nor a war machine and whose opponent
retreated, surrendered or has no living stack apart from war machines; if neither side qualifies
nobody wins: a side left with only summoned stacks and clones loses as well (the wiki's "both
sides lose"), also when the other side retreated or surrendered. The experience of the battle
(0x46A350) counts no killed summoned stack, clone or war machine; Necromancy (0x4775E1) counts the
killed creatures of every record, summoned stacks and clones included.

### 7.20 Code reading: creature magic after a strike (E04b T7a, 2026-10-08)

Read for T7a (Q29: insight only, nothing copied). Offsets as in §7.9; creature ids as in §5 (22,
23 Dendroids, 24, 25 Unicorns, 59 Zombie, 66, 67 Black and Dread Knight, 69 Ghost Dragon, 76, 77
Medusas, 81 Scorpicore, 93 Thunderbird, 104, 105 Serpent and Dragon Fly, 106, 107 Basilisks, 109
Wyvern Monarch, 135 Rust Dragon, 141 Mummy).

**The melee attack** (0x441CB0, attacker A on target T): T is marked if blind or paralysed
(§7.14), A strikes (0x441610), then T retaliates if it lives, A's creature allows retaliation
(flag `0x10000` not set), T is **not petrified** (Stone Gaze's duration +0x2B0 is 0), T has a
retaliation left and the strike did not answer "no retaliation"; the marks are cleared. Then a
creature that strikes twice (`0x8000`) and is no shooter (`0x4`) strikes again, without a
retaliation, if A lives and is **not blind, petrified or paralysed** and T lives. Everything below
runs inside the strike, so for the first and second strike and for the retaliation alike, and only
in melee: the shot routine (0x43F900) has none of it.

**A strike** (0x441610, striker S on target T; a breath's second stack only takes damage): the
damage (0x443F40) and the hit points taken (0x444090, which ends Blind, Stone Gaze and Paralyze,
§7.14), then in this order:

1. *The strike's spell* (0x440500, a switch on S's creature) is put in T's pending spell (+0xEC,
   −1 none) by the table below.
2. *The pending spells are cast* at the end of the strike's animation (0x468990, also when combat
   is not animated): for every living stack of both sides with a pending spell, the spell is cast
   on that stack's hex as a creature's cast (0x59FE30, caster kind 1, §7.8) **at mastery 0 with
   power 3**, and the pending spell is cleared. So the spell takes effect before the retaliation.
3. *The striker's own effects* (0x440BC0, a switch on S's creature, on T only): Vampire Lords' life
   drain (63) and Mighty Gorgons' death stare (103) (E03), and the Thunderbird, flies and Rust
   Dragon below.
4. Fire Shield's reflection (0x4409C0, §7.11).

The strike answers "no retaliation" if T was marked blind with Blind's stored factor 0 (§7.14) or
if step 1 put Blind, Stone Gaze or Paralyze in T's pending spell.

| Striker | Spell put in T's pending spell | Draw | Also needs |
|---|---|---|---|
| Unicorn, War Unicorn | Blind (62) | 20 % | T alive, the test |
| Black Knight, Dread Knight, Mummy | Curse (42) | **25 %** | T alive, the test |
| Zombie | Disease (73) | 20 % | T's creature living (`0x10`) first, T alive, the test |
| Ghost Dragon | Age (75) | 20 % | as Disease |
| Wyvern Monarch | Poison (71) | 30 % | as Disease |
| Medusa, Medusa Queen, Basilisk, Greater Basilisk | Stone Gaze (70) | 20 % | T alive, the test |
| Scorpicore | Paralyze (74) | 20 % | T alive, the test |
| Dendroid Guard, Dendroid Soldier | Bind (72) | none | S not yet among T's binders; no test, T may be dead |
| Rust Dragon | Acid Breath (80) | none | T alive, T's defense above 0; no test |

The draw is `random(1, 100) ≤ chance`. *The test* (0x5A8640: spell, S's controller, T) is the
chance `f` of 0x5A8090 (§7.8) with its **side test skipped** (its fourth argument: a negative
spell may land on a stack of S's own side, a hypnotized or berserk striker's) and a second draw
`random(1, 100) ≤ trunc(f × 100)`. So the creature immunities (0x44A4D0), the magic resistance of
creatures (Dwarves, Crystal Dragons) and of T's controller's hero and the Unicorn aura apply as
to a hero's spell. The ability spells' levels (`SPTRAITS.TXT`: Stone Gaze 3, Disease 2, Paralyze
4, Age 5, the others 0) count for the dragons' level immunity (Green, Red and Azure Dragons ≤ 3,
Gold ≤ 4), for Cursed Ground (none above 1) and Recanter's Cloak on either hero (none above 2)
(E04c, T8); the Orb of Vulnerability on either hero lifts the normal immunities (T8). 0x44A4D0's
absolute cases for them: Stone Gaze spares the Troglodytes (70, 71); Poison spares creatures that
are not living (`0x10`) and the Gargoyles (30, 31); Blind as §7.14, and as a mind spell it spares
mind-immune creatures. Bind's binder lists: T keeps the stacks that bind it (+0x504), S the stacks
it binds (+0x4F4); a strike on a T that S already binds does nothing.

**Applying them** (the cast routine's handlers, §7.8; 0x4448F0, §7.9). A creature's cast has no
hero: no specialty bonus, no duration bonus; its duration is the power, 3, unless fixed (Bind:
255); its mastery is 0 (Expert on battlefield type 1, Magic Plains, E04c); the stack on the hex is
the target, with no further draw (§7.8). As for every spell, a spell already active only takes the
longer duration and the higher mastery. Per spell:

- Blind, Curse, Disease, Paralyze (the timed handler 0x5A107F), Bind and Age (0x5A15C7, the same
  code), Stone Gaze (0x5A10F1, plus its animation), Poison (0x5A1590). Curse at mastery 0 has
  `v` = 0 (the minimum damage, §7.9); Blind `v` = 50 (a blind stack retaliates at half, §7.14);
  Stone Gaze, Paralyze and Bind store no value.
- *Disease* takes `min(2, attack)` and `min(2, defense)` and stores both (+0x4D4, +0x4D0).
- *Poison*: the stack's poison factor (+0x4A4, 1.0 at the battle's start) becomes `f32(max(factor
  − f32(0.1), 0.5))`; then the hit points per creature (+0xC0) become `trunc(f32(H) × factor +
  f32(0.95))`, the product times f32(0.5) while aged, at the x87's 53 bits (0x444F65: `H` and the
  factor are stored as f32, the product is not; corrected at the T7a review), with `H` the
  stack's hit points per creature at the battle's start (+0x6C), and the top creature's lost hit
  points (+0x58) become at most the new value − 1 (no creature dies).
- *Age*: the hit points per creature become `trunc(f32(H) × factor × f32(0.5) + f32(0.95))`
  (0x444FBE, as Poison's), with the same cap.
- *Acid Breath* (0x5A1E70): defense − 3, at least 0, for the rest of the battle (no duration, no
  entry in the active spells; each Rust Dragon strike lowers it again).
- *Dispel Helpful Spells* (0x5A177E): ends every active spell 10–69 whose table type is positive
  (0x444510).

**While active and ending.** Stone Gaze and Paralyze stop the stack's turns and end on any damage
as Blind does (§7.14); a petrified stack takes half damage (stage 3, §7.10) and does not
retaliate; a paralysed one retaliates with Blind's Advanced share (25 %, §7.14). A stack under
Bind cannot move (the pathfinder, 0x4B2DE0 and 0x4B350B, gives it no movement) but still attacks
next to it. At every round start (0x447120), after the durations count down, a stack still
poisoned loses another tenth (the factor and the hit points as above; a message when they fell),
so a poison of 3 rounds leaves 90 % at the strike, then 80 % and 70 %, and ends at the third round
start. The end routine (0x444510) gives Disease's points back, recomputes Age's hit points with the
poison factor and the same cap, and for Bind removes the stack from each binder's list and empties
its own; it gives nothing back for Poison, so **the poison factor and its lost hit points stay for
the battle** after Poison ends, until a Cure (0x446500 resets the factor to 1.0 and the hit points
before it ends the spells, §7.10). Bind is also released (0x43EE10) when a binder moves (the move
handler 0x445990: the walk 0x441FA0 at 0x4420A3, the flight 0x4B49C0 at 0x4B49E1, a Devil's
teleporting move 0x4B4E90 at 0x4B4EB1), dies (the death routine 0x444120 at 0x4441C7) or is
teleported by Teleport (0x5A1B88): the binder leaves the bound-by list of each stack it binds
(from its own list's end), a stack whose list becomes empty has its Bind ended (0x444510(72)),
and the binder's own list is emptied. Then 0x43EE10 does the same for the moving stack as a bound
one (0x43EEED): it leaves the list of each of its binders and its own bound-by list is emptied,
**without ending its Bind**. Teleport then ends the teleported stack's own Bind as well (0x5A1B91,
`push 0x48; call 0x444510`). The pathfinder's test reads Bind's duration (+0x2B8), not the lists,
so a bound stack moves only where the pathfinder is not asked: the Harpies' return.

**Strike and return** (read at the T7a review): after the attack the action handler flies a Harpy
or Harpy Hag (72, 73) that moved back to its start hex (0x4789F8, the move 0x445D10 → 0x445950)
unless it is dead or its Blind (+0x290), Stone Gaze (+0x2B0) or Paralyze (+0x2C0) duration is
above 0; Bind's (+0x2B8) is not tested. So a Harpy bound by the Dendroids' retaliation flies back
and the flight's 0x43EE10 empties its lists without ending its Bind: no binder's move or death
releases it any more, only Teleport, Dispel or its 255 rounds (survey S-14, the wiki's "bound down
forever"; fixed in every profile, the user 2026-10-08: a bound stack does not return, BAT-ATK-004). Bind is ended by Dispel, not by Cure (§7.10). Bind and Acid Breath skip the test, so
the flag word's war-machine bit (`0x1000`, set on Bind) does not keep them off a war machine.
*Not reproduced (BAT-Q8):* 0x440500 lists the striker among the binders of a T its strike killed
(no cast follows); the entry stays until the striker moves or dies, so a T raised meanwhile would
not be bound by that striker's next strike. The engine binds only a living T.

**The striker's effects** (0x440BC0):
- *Thunderbird*: a 20 % draw, then the test for **Lightning Bolt** (17) as above (so Earth and
  Magma Elementals are immune, the dragons' level immunity counts level 2); the damage is
  0x5A78E0 (§7.8) of `10 × S's creatures` for spell 17 without a hero: the target creature's
  factors (Air and Storm Elementals double, the golems) and Protection from Air; then the hit
  points are taken (0x444090) and the kills reported.
- *Serpent Fly, Dragon Fly*: if the test for Dispel Helpful Spells (78) passes (0x5A8090's own case:
  no if T's controller's hero wears the Sphere of Permanence, T8; yes if T has an active spell of
  positive type; no immunity, resistance or aura), it is cast at once (creature cast, mastery 0,
  power 3). Then a Dragon Fly, if T is not under Weakness (+0x24C) and passes the test for Weakness
  (45), casts Weakness at once **at mastery 2** (Advanced: `v` = 6) with power 3.
- *Rust Dragon* (0x44149F): if T lives, a 20 % draw, then `random(min, max damage) × S's creatures
  / 2` (truncated; S's +0xD0, +0xD4) taken through 0x444090 if above 0, with no test and no
  reduction (after Acid Breath, step 2); a message, then the pending spells are cast again
  (0x468990).

### 7.21 Code reading: casting creatures, the Enchanter and the spell cost (E04b T7b cut, 2026-10-08)

Read for the T7b/T7c cut (Q29: insight only, nothing copied); the parts each task builds are
re-read there. Creature ids as in §5 (13 Archangel, 34, 35 Mage, Arch Mage, 37 Master Genie, 20,
21 Pegasi, 43 Familiar, 48 Demon, 51 Pit Lord, 61 Wraith, 91 Ogre Mage, 123, 125, 127, 129 Ice,
Magma, Storm and Energy Elementals, 131 Phoenix, 134 Faerie Dragon, 136 Enchanter). Stack
offsets as in §7.9; **+0xDC is the casts left** (the creature's casts per battle, `casts` in the
packs), +0x4E0 a Faerie Dragon's spell.

**The cast routine's arguments** (0x59FE30, §7.8): spell, target hex, caster kind (0 hero, 1
creature), second hex, **mastery**, **power**. A hero's cast passes 0 and 3 and takes its own
mastery and power; a creature's cast takes the arguments, and Magic Plains (battlefield type 1,
E04c) raise its mastery to 3. A creature's cast has no hero bonuses, and the aimed stack never
rolls for resistance (§7.8; area spells still roll per stack).

**The creature's cast action** (the action dispatcher's case 10, 0x47936E → 0x448260, stack,
target hex; the action names no spell): it turns the stack towards the hex (animation only),
**lowers its casts left by 1**, then switches on the creature (0x448768/0x448740):

| Creature | What it casts (0x59FE30 arguments: spell, mastery, power) |
|---|---|
| Ogre Mage | Bloodlust (43), mastery 2, power 6 |
| Storm, Ice, Energy, Magma Elemental | Protection from Air (30), Water (32), Fire (31), Earth (33), mastery 2, power 6 |
| Faerie Dragon | its drawn spell (+0x4E0), mastery 2, power **5 × creatures** |
| Master Genie | 0x447EE0: a random spell for the stack on the hex (below), mastery 2, power 6 |
| Archangel | no cast routine: 0x5A3CC0(side, hex, 1) finds the stack (Resurrection's test, §7.18, with the creature's hit points), then 0x5A7560 raises it by **creatures × 100** hit points, permanently |
| Pit Lord | 0x5A3E40(side, hex) finds the topmost corpse on a free hex of the caster's side whose creature is living (`0x10`); 0x5A7390 raises **Demons** (48) there as a created stack (0x47A100, §7.19), which takes the corpse's +0x5C: `min(start count, min(50 × Pit Lords, start count × hit points) div Demon's hit points)` (0x447330) |

Every other creature does nothing there. With mastery 2 the timed spells take their Advanced
value and last 6 rounds (the power, §7.9).

**At a stack's activation** (0x465330, after the hero's opening spells, T8): a Faerie Dragon with
casts left draws its spell (0x447510) from the weighted table 0x63B850, Ice Bolt 22, Lightning
Bolt 22, Fireball 21, Magic Arrow 10, Frost Ring 10, Chain Lightning 5, Meteor Shower 5, Inferno 5
(of 100; the draw is the C library's `rand() mod 100`, not the battle's random generator; VCMI
gives Magic Arrow 22), and a message tells its controller; a Wraith lowers the enemy hero's mana
by 2, at least 0 (T7c); an Enchanter casts on its own (below). A Phoenix's rebirth also uses its
casts left (0x469480, T7c).

**Master Genie** (0x447EE0): over spells 10–69 with flag `0x800` (beneficial on an own stack,
§7.6: Shield, Air Shield, Fire Shield, the Protections, Anti-Magic, Magic Mirror, Cure, Bless,
Bloodlust, Precision, Stone Skin, Prayer, Mirth, Fortune, Haste, Slayer, Frenzy, Counterstrike), it
counts those 0x447A80 accepts for the target stack, draws one with the battle's `random(1, n)`
and casts it. 0x447A80(spell, stack): not active on it, 0x5A3C80 lets it affect the stack, then a
usefulness case (0x447E8C/0x447E68) for Shield and Fire Shield, Air Shield, the Protections
with Anti-Magic and Magic Mirror (an enemy hero with a spell book), Cure (a wounded top creature),
Bloodlust, Precision, Prayer (the stack has not acted this round), Slayer (an enemy stack with a
creature flag `0x80`); the cases not named here are read in T7b; every other spell is useful.

**Enchanter** (0x447FE0, at its activation if its side's counter is above 2): each side's counter
(+0x132A0) rises by 1 at every round start (0x4760F8) and returns to 0 after a cast, so a side's
Enchanters cast once in three rounds, the first of them to act. Over the weighted table 0x6608B8,
Haste 15, Air Shield 10, Slow 10, Stone Skin 15, Bloodlust 5, Bless 15, Cure 10, Weakness 4, a
spell counts if 0x447A80 accepts it on a stack of the side it is aimed at (the enemy's for a
negative spell); **the loop tests the side's first stack every time** (it never advances the
record: survey S-12, confirmed). It draws among the counted weights with `rand()` and casts the
spell on no hex at **mastery 3** (Expert: the mass versions) with **power 3**. If nothing counts,
nothing is cast and the counter stays.

**The hero's spell cost** (0x4E5240: hero, spell, the enemy's army, battlefield): Titan's
Lightning Bolt costs 0; otherwise the cost at the hero's mastery (Armageddon at Expert with
Armageddon's Blade, T8), **+2 if the enemy's army holds Pegasi or Silver Pegasi, −2 if the hero's
own army holds Mages or Arch Mages**, at least 1. Both tests read the armies (0x44AB80), not the
battle's stacks, so a dead Pegasus stack still counts (batch 2a R6 round 3: 5, VCMI 3) and a
summoned or cloned one does not. After a hero's cast of cost 5 or more, the other side's living
Familiars channel `cost div 5` (0x5A2170, T7c; batch 2a R6).
