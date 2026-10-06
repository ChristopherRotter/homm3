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

Not read: whether a vanished clone frees its record, where Pit Lords' demons go, and why the 21st
record stays free.

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
   (Luna, +100 %); a few timed spells use their own tables (T3).
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
for Bloodlust, Precision, Weakness, Stone Skin, Prayer, Fortune and Haste, T3c) is used per
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
   then a Bless specialist's bonus, T3c); then a Dread Knight's death blow (20 %) `B += b` and the
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
Force Field, Quicksand, Land Mine; T5). Which stacks it can affect (0x5A8090 for spell 35):
none if the stack's hero wears the Sphere of Permanence (artifact 92); below Advanced only the
caster's own stacks; below Expert only a stack with at least one active spell (ids 10–80); then
the usual immunity tests. Magic Mirror never redirects Dispel (0x59EE99).

**Cure** (case at 0x5A17E6; 0x446500 per stack): heals the top creature only (lost hit points
− (power × factor + effect[mastery] + a Cure specialist's bonus), at least 0; no creature comes
back), and ends Curse, Weakness, Sorrow, Misfortune, Slow, Berserk, Hypnotize, Forgetfulness,
Blind, Stone Gaze, Poison, Disease, Paralyze and Age (not Bind, not Disrupting Ray, not Frenzy).
At Expert it does so for every own stack it can affect that is not hypnotized. No test of the
target's health or spells: Cure may be cast on a healthy stack without negative spells.
