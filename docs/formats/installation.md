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
0.05 into the skill specialties' 500 bp per level; Logistics is not read. E04b adds Sorcery,
Resistance and Eagle Eye, E08 T18b Necromancy (`necromancy`), Learning (`experience_gain`) and
Intelligence (`max_mana`).

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

Not read: why the 21st record stays free (Pit Lords' Demons: §7.21, "The raisers"). A killed clone's or summoned stack's record (dead with no creatures) goes to the next stack the side creates; one whose rounds ran out keeps its record (§7.19).

### 7.8 Code reading: damage spells, immunities and magic resistance (E04b T2d, 2026-10-06)

Read for T2d (Q29: insight only, nothing copied; disassembly of the SoD executable with
`capstone`). Floats are x87: intermediate values are extended precision, stores to memory are
single precision (`f32`); `_ftol` (0x6179D4) truncates; 0x618CC7 is `ceil` (rounding mode up).

**Effect handlers.** The battle manager's cast routine switches on the spell id through a byte
table at 0x5A2838 (targets at 0x5A27A0): one handler per damage spell 15–19 and 24–26, the four
area spells 20–23 share 0x5A4970, Titan's Lightning Bolt (57) shares Lightning Bolt's. A
single-target spell's value is `power × power factor + effect[mastery]`, passed through the
damage-modifier function 0x5A78E0, which applies in this order (step 1 only when the routine is
given a hero, 0x5A790D):

1. **The caster's bonuses** (0x4E5760, on the hero): `v = f32(value)`; `× 1.5` if the hero has
   the Orb of one of the spell's schools (checked air, earth, fire, water; at most once); `× (1 +
   Sorcery)` (the §7.4 table, grown by `× (1 + 0.05 × level)` for a Sorcery specialist); both
   products are stored as `f32`, **not truncated**; then, if the spell has a target stack, the
   specialty bonus (0x4E5FF0) computed from `t = trunc(v)`, a whole number, is stored as an `f32`
   and added to the stored `v` itself (not to `t`), and the sum, stored as an `f32` once more, is
   truncated (0x4E5932 to 0x4E5944); without a target stack `v` is truncated as it is. A store
   rounds to 24 bits, to nearest, so a product a hair below a whole number becomes that number
   before the truncation (a Sorcery specialist's `f32` rate of 0.15999999642 on 50: 57.99999982 is
   stored as 58). Specialty bonus, for a hero whose specialty is this spell: by default
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
first, then the attacker's (the review's re-read below has the attacker's first), each by
stack record, keeping the first on ties (strict `<`); each
candidate is first tested like a spell target (below): when casting for real, a candidate that
fails its resistance roll is skipped for that jump (it can be chosen at a later jump); an immune
one is never chosen. `dx`, `dy` come from getters 0x446660/0x446630: the stack's hex record's
screen position, x shifted by ±22 for a two-hex stack (towards its second hex), y lifted by
half of `267 − top offset of the creature's sprite frame` (a graphics value; inference: screen
pixels, as the mouse-to-hex code at 0x464870 uses 44 × 42 pixel hexes and the bolt's animation
length is derived from the same distance). Jumps per mastery from the table at 0x642274.

**Review re-read (E04b T11b, 2026-10-12): the chain's search order.** The search 0x5A61F0 starts at
the first side's data, the struck marks at +0x547C, the stack records at +0x54CC and the record
count at +0x54BC (0x5A6223 to 0x5A622F), and after its loop over that side's records moves all three
on by one side (0x5A6328 to 0x5A6331). The first side is side 0, the attacker's: the handler sets a
struck stack's mark at +0x547C + 20 × its side (+0xF4) + its record index (+0xF8) (0x5A657E to
0x5A659A). So **the attacker's stacks are searched first**, then the defender's, each by record, and
the strict `<` keeps the earlier of two equally near (0x5A62F7): on a tie between a stack of each
side the attacker's is struck first, where the sentence above has the defender's, and the two
stacks' shares of the damage change places. The counter beside the loop, which runs from 1 to 0
(0x5A6235, 0x5A6334) and which the first reading took for the side searched, is used once, as the
caster's side of the candidate's test (the next paragraph). Not observed: batch 2a R2's tie is
between two stacks of one side, and R1 has none. The reading stands alone: the wiki has a random
choice among equally near stacks, VCMI 1.7.5 a tie-break by position
(`lib/spells/effects/UnitEffect.cpp`, `BattleHex::getClosestTile`). On a later target that resists
VCMI differs from this § in three more points (whether it resists is drawn once per cast; a skipped
unit is out for the rest of the chain; the next target is the one nearest to the unit just
examined): there the re-read confirmed this § (a candidate that fails its roll is passed for that
jump only, 0x5A6272 to 0x5A6295; marks are written for struck stacks only) and the engine follows
it; that conflict is on record in BAT-Q28 and in BAT-MAG-006's VCMI source. The engine searches the
defender's stacks first (BAT-MAG-006): it does not follow this yet: BAT-Q28.

**Review re-read (E04b T11b, 2026-10-12): which side a chain candidate is tested for.** That counter
is the second argument of the candidate's test 0x5A8090, the caster's side (0x5A6269): 1 while side
0's records are walked, 0 for side 1's. So every later candidate is tested as if the side opposite
to its record had cast the spell, whoever cast it: the test's casting hero (+0x53CC by that
argument, 0x5A80A1) is the hero opposite to the candidate, its controlling hero the candidate's own
(0x442690), and 0x44A4D0 gets both. Chain Lightning is neutral, so the side tests do not notice
(below); what changes is whose Orb of Vulnerability counts, since 0x44A746 asks both heroes it is
given (§7.23): by this reading either hero's Orb opens every later target, the caster's own stacks
included (the defender's hero wears the Orb: the attacker's chain strikes the attacker's own Black
Dragons), where the first target is tested for the acting side (the roll at 0x5A02EA). Unsure, by
the report (confidence medium): the consequence is traced only as far as that Orb test, readings
about this Orb have needed an experiment before (BAT-Q21), and nothing was observed; neither VCMI
nor the wiki was compared. The engine tests a candidate with the caster's hero and the hero of the
candidate's controller, so at the caster's own stacks only the caster's Orb counts: it does not
follow this yet: BAT-Q28.

**Area spells** (0x5A4970): the stacks on the area's hexes (one entry per stack), each tested
and rolled separately, then damaged with steps 1–3. The handler takes step 1's hero from the
acting side itself, whoever cast the spell, and so does the chain's (§7.21, the review re-read
on a creature's area spell).

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
19) the cursor, the book's "any target" check and the opening spells' (0x5A40D0, §7.24) also
refuse a stack of the caster's side (0x59F7BF; batch 2a R1). The single-target spells, their mass versions and the battle-wide ones (24–26) need at least one hex whose stack the target test accepts (`f > 0`) before the spell is armed: without one the book shows a message and nothing is cast or paid; a spell that takes no target stack at its mastery is then cast without a cursor. The test reads a stack's Hypnotize only for the hero that controls it, so a hypnotized stack the mass handler later passes over counts. The area spells have no such test. 0x5A8090 also returns 0 for spells above level 1 on battlefield type 2 (Cursed
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
− v)`, else the roll; a Ballista's range is already the hero-scaled one. Luck (0x43DF20): the
base luck, **not clamped** (the routine pushes 0 for the base routine's clamp argument,
0x43DF6D; 0x44B2D0 clamps to ±3 only when that argument is set, 0x44B380, and passes it on to the
hero's routine), + Fortune's `v` − Misfortune's `v`, then the Halfling minimum; morale
(0x43E000): 0 for creatures without morale, else the base morale, not clamped either (0x43E038;
0x44AE60, 0x44B0B0), + Mirth's `v` − Sorrow's `v`, then the Minotaurs' minimum and Spirit of
Oppression; the value is clamped to ±3 only where it is used: the good-morale roll (0x4649AA) and
the luck rolls (0x43F936, 0x441818). So a base beyond ±3 absorbs the spells: +5 under Expert
Sorrow is +3. VCMI 1.7.5 (`lib/BasicTypes.cpp`) clamps the total once as well.

**Review re-read (E04b T11b, 2026-10-12): the Defend bonus sits inside the defense.** The Defend
action (the dispatcher's case at 0x479706) computes `max(1, defense × 20 / 100)` in whole numbers
from the defense field +0xCC once (0x47974A to 0x479791), stores it at +0x4DC, adds it to +0xCC
(0x4798C9 to 0x4798D7) and sets the stack's defending bit `0x8000000`; the stack's next turn start
subtracts the stored amount again and clears the bit (0x446E74 to 0x446E8D). In the tactics phase
and for a war machine the stored amount is 0 and the bit stays clear (0x479722 to 0x479735,
0x4798C3). Nothing recomputes the amount: Stone Skin, Prayer, their ends and Disrupting Ray write
+0xCC itself (above), so a defense change between the Defend and the stack's next turn does not move
the bonus, and whatever reads or floors the defense in that time meets the bonus inside it: a
strike's defense (0x442590), Frenzy (§7.11), Disrupting Ray's floor at 0, and Acid Breath, whose
test wants +0xCC above 0 (0x4407F8) and whose handler lowers +0xCC by 3, to 0 at least (0x5A1E70,
§7.20). In numbers: defense 12, Defend (+2), then Expert Stone Skin (+6) gives 20, where a bonus
taken from the defense of the moment gives 18 + 3 = 21; defense 0, Defend (+1): the breath lands (1
to 0) and the next turn start leaves −1. Not observed. VCMI 1.7.5 adds 20 % of the unit's defense as
a bonus of its own until the unit's next turn (`server/battles/BattleActionProcessor.cpp`
`doDefendAction`), which follows the defense as the engine's does; the wiki's wording fits both, by
the report. The engine takes the bonus from the defense at each strike and tests and lowers the
defense without it (BAT-TRN-004, BAT-MAG-046): it does not follow this yet: BAT-Q28.

**Ending.** At the start of every round (0x4760xx calls 0x447120 for every stack record of both
sides, attacker's first; skipped in the tactics phase) each active spell of a living stack, in
spell order, except Frenzy, either ends (duration 1) or loses a round. A stack that dies has
every active spell ended (0x444120 calls 0x444510 for each, the loop 0x444228 to 0x444244): all
but Disrupting Ray (below), whose duration and lowered defense stay on the dead record; no raise
restores it (§7.18). Ending a spell (0x444510) restores what it changed: Weakness adds
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
stack of both sides that it can affect (0x5A8090 above 0, **no resistance roll**; its own loop
from 0x5A160F, two passes over the sides' records, reads no Hypnotize: hypnotized stacks
included, unlike the mass handler of the timed spells, §7.14), then removes
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

**Review re-read (E04b T11b, 2026-10-12): what counts as an active spell for Dispel's aim.** Below
Expert the test above walks the stack's durations of the spell ids 10 to 80 and wants one that is
not 0 (0x5A81A2 to 0x5A81B9). Acid Breath lowers the defense in its own handler, which never calls
the apply routine 0x4448F0 (0x5A1E70 to 0x5A1F84, §7.20), so it leaves no duration: a stack whose
only change is Acid Breath is no target for a Dispel below Expert. Disrupting Ray keeps its duration
(§7.9), so a stack with only the Ray is one, though Dispel ends nothing on it. Not observed (the
report: high for the code; rare in play, it takes Rust Dragons); neither VCMI nor the wiki was
compared. The engine keeps Acid Breath as an effect and accepts such a stack, a cast that does
nothing (BAT-MAG-016): it does not follow this yet: BAT-Q28.

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
struck stack lives (0x441DD4; for a striker the reflection killed see the review re-read
below).

**Review re-read (E04b T11b, 2026-10-12): the retaliation at a striker the reflection killed.**
After the first strike (0x441DC5), inside which the reflection is applied, the melee routine skips
the retaliation only if the struck stack has no creatures left (0x441DD4), the attacker's creature
allows none (flag `0x10000`, 0x441DDF), the struck stack is petrified (0x441DED) or has no
retaliation left (0x441DF7), or the strike answered "no retaliation" (0x441E01). It does not ask
whether the attacker has creatures left, though the second strike does (0x441EAF). So the struck
stack strikes back at an attacker the reflection killed (0x441E3D) and one retaliation is taken
(0x441E58 to 0x441E5F): the round's next attacker is not struck back. What that strike does was not
followed beyond two points: the strike routine wants only a target pointer (0x4419EA), and the
damage routine takes nothing from a stack without creatures (0x4440DE). Not observed (the report:
medium to high for the path). VCMI 1.7.5 retaliates only against a living attacker
(`server/battles/BattleActionProcessor.cpp` `doAttackAction`), as the engine; the wiki was not
checked. If an experiment confirms it, it is an oddity of the original whose class is the user's to
decide. The engine does not retaliate at a dead striker and spends no retaliation (BAT-RET-001): it
does not follow this yet: BAT-Q28.

**Frenzy** (spell 56; one target, a mind spell): 0x4448F0 sets its duration to 1 if the target is
the active stack, else 2 (whatever the caster's power), and stores `v / 100` as `f32` (+0x498).
The round start's countdown skips it (§7.9); the stack's turn start 0x446E30 (once per round:
the flag +0x4F0, cleared at the round start 0x447120) counts it down: above 1 it loses a round,
else it ends. So it lasts through the stack's next turn (re-read in the E04b T11b review; VCMI
1.7.5 instead ends it at the stack's first strike of any kind, a retaliation included, however
many turns pass: `UNTIL_ATTACK` in `config/spells/timed.json`; the conflict is on record in
BAT-Q28). **Attack** (0x442410, last, after
Slayer): `trunc(f32(attack) + f32(d) × factor)` with `d` the frenzied stack's own defense as the
struck stack sees it (0x442590 with the struck stack as the other side, flag 0): +0xCC (the
Defend bonus included) reduced by a Behemoth's or Ancient Behemoth's share when the **struck**
stack is one (quirk S-16, confirmed), and the siege moat's −3 (E04c). **Defense** when struck
(0x442590 flag 1, used by the damage stages 0x4433A8 and 0x443BBD): 0 while frenzied.

**Behemoths** (0x442590, read for S-16): against a Behemoth (creature 96) the defense is
`trunc(f32(D) − f32(D) × f32(0.4))`, against an Ancient Behemoth (97) `trunc(f32(D) − f32(D) ×
f32(0.8))` (0x4425C8 to 0x4425F5; the Ancient Behemoth's twin at 0x4425DE; the constants at
0x63B8B0 and 0x63B8AC): the defense left is computed in single precision and only then
truncated; the share taken is never rounded on its own. The result is Q19's except where D × 0.4
(D × 0.8) is whole, where the stored constants, a hair above the decimals, leave one point less
(D = 7: 4 and 1; D = 10: 5 against a Behemoth; D = 5: 0 against an Ancient Behemoth). VCMI's
`floor(share × D) + 1` ignored points, at most D (`lib/battle/DamageCalculator.cpp`,
`getTargetDefenseIgnored`), is the same number for every D from 1 to 300. The user (2026-10-07):
follow the executable under quirk Q-0005, keep Q19 in strict-fixed.

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
T7). The redirect is decided where a player's cast is entered (0x59EC50, from the spell book and
for a Faerie Dragon's spell from its key and its dialog, at 0x59EE7A; a creature's action then
drops it: §7.21, the review re-read): for a cast that takes a target stack (0x59E060: the
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
  (+0x84 bits 21, 25, 26; the comparison and the wait phase: §7.21, the review re-reads) and, for
  a stack whose turn has started this round (+0x4F0), only if it
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
- beside another stack it counts as an enemy either way (0x442880 answers yes when either stack is
  berserk; its only callers are the blocked-shooter test 0x469600 and the neighbour test of
  multi-headed strikes, 0x523B20 through 0x523BB0). Who may attack or shoot it is the cursor's
  test (0x476490: the target's own side against the acting side), which has no Berserk test, so
  its own side cannot;
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
0x4448F0 ends Berserk first and rebuilds the stack's Unicorn aura lists (+0x514/+0x524: the
adjacent Unicorns whose controller is the stack's real side, and for a Unicorn the neighbours
whose real side is its controller; 0x43EA70, the Unicorn's side flipped by its own Hypnotize at
0x43EAEE and 0x43EB9C, the other stack's +0xF4 as it is), as its end does. While active, the
stack's **controller** is the other side (0x442690 and the inline copies `side ^ hypnotized`):
the controller commands its turns (the battle loop sets the acting side to it, 0x465389, so the
controller's hero may cast during that turn), and these use the controller and its hero: the
Offense or Archery and the Bless specialist of its strikes and the Armorer of strikes on it
(stage 1 and 3, 0x443898, 0x443DE6), the hero's resistance and the Pendants against spells on it
(0x5A8090 → 0x44A4D0), Dispel's Sphere test (§7.10), the target's hero in spell damage and Fire
Shield (0x5A0D1C, 0x4224AF). Relations between two stacks take the controller of the one and the
real side (+0xF4) of the other: 0x442880 (the first stack's controller, the second's side; asked
for the blocked shooter, 0x469600 with the shooter first, and for multi-headed strikes, 0x523B20
with the striker first), the cursor 0x476490 (the acting side +0x132C0 against the target's side
for an attack or a shot, equal sides for the tent's patient; the pathfinder's attack targets
alike, 0x4B3801) and the aura lists (above). These keep the real side: the stack's statistics set
at the battle's start (its hero's primary skills, morale and luck), the turn order's tie-break
(0x46502A), the automatic tent's loop over its side's records (0x473EA0: the stack with the most
lost hit points that is neither dead nor a war machine, no Hypnotize test), the shot's Ammo Cart
test (0x43F9E1: the hero of the shooter's +0xF4 side) and the side tests of spells (0x5A82F8): the
owner may cast its positive spells on its hypnotized stack (Cure ends Hypnotize, §7.10) but no
negative ones, the controller the other way round. So the side that hypnotized a stack may attack
and shoot it and its own side may not (batch 2c H1's remark), it blocks the shooters of the side
that controls it, and its own side's Unicorns, tent and Ammo Cart go on serving it. The mass
versions of the timed spells (0x5A66D0, §7.9) skip hypnotized stacks, and so does Expert Cure,
which loops over the caster's own stacks (0x5A1847, +0x5754 = Hypnotize's duration): an Expert
Cure never reaches them (quirk S-13), a single Cure does. Dispel's Expert loop (§7.10, from
0x5A160F: two passes, one per side, over the side's records) reads no Hypnotize duration: an
Expert Dispel reaches them and ends their Hypnotize. VCMI 1.7.5 limits Hypnotize by the stack's
remaining health (`config/spells/timed.json`, `lib/spells/TargetCondition.cpp`
`HealthValueCondition`) and agrees on who may attack and shoot whom (`battleMatchOwner`: the
attacker's owner through hypnosis against the defender's own side), the tent, the Ammo Cart and a
hypnotized stack's aura; it differs for hypnotized Unicorns (they go on protecting their own
side), for a hypnotized shooter (blocked by the stacks of the side that controls it:
`battleIsUnitBlocked` compares initial owners, and counts Berserk only on the shooter itself) and
for multi-headed strikes (`getAttackedCreatures` compares both owners through hypnosis and has no
Berserk term). On these three relations and on Berserk's term the engine follows this reading,
not VCMI; the conflict is on record in BAT-Q28 and in the VCMI sources of BAT-MAG-025,
BAT-MAG-026, BAT-RNG-001 and BAT-ATK-003.

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
moved and then attacked is not (0x478900 skips it). The Harpies' return (0x4789F8, §7.20) is the
move routine again: the marks are cleared, the hexes the stack stands on are marked, and the
flight enters the hexes it lands on (Fire Wall, then Land Mine; the start hex is no longer
marked). Nothing tests the battle's end before the return. After a shot the battle's end is checked first
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
(the engine's rule in every profile since BAT-QUIRK-1 was retired, BAT-Q34). VCMI 1.7.5 asks each time whether the side has a native stack, dead ones included
(`battleHasNativeStack`, `lib/battle/CBattleInfoEssentials.cpp`: every stack that is no ghost): as
the original for natives that die, unlike it for natives that appear later.

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
rest of the move). The walker (0x441FA0), at each step of its path, before it moves: if its new
front hex or, for a two-hex stack, its new rear hex (its position) is a stop, this step is the last; else if it
has Quicksand, the walker reveals that pit (+0xA = 1) and this step is the last; then it moves and
enters the hexes (§7.15: Fire Wall, Land Mine, the moat). Either way the walk returns false, and a move and attack (0x478971) then skips the attack; a walk that ends in a seen pit is only ever a plain move, as no attack is offered from there (the review re-read below). A flyer (0x4B49C0) and a teleporting creature (0x4B4E90) never test the pits;
Teleport places the stack without entering. Standing on a pit has no further effect: the stack
leaves it freely on its next turn (but for a two-hex stack walking on along its row: the review
re-read below).

**Review re-read (E04b T11b, 2026-10-12): a two-hex walker leaving a pit along its row.** The
walker's tests differ in what they leave out. The stops array leaves out the hexes the stack stands
on when the walk is planned (0x4B33D6 to 0x4B33F5), and the pathfinder does not count a stop that
the stack already covers at the position it steps from (0x4B36ED to 0x4B3725). The walker's own
Quicksand test leaves out nothing: at every step it reads the Quicksand bit of the hex its position
moves to and, for a two-hex stack, of that position's other hex (the position ± 1 by the facing,
0x4422C6 to 0x4422E8), and either bit reveals the pit, makes the step the last and the walk return
false (0x442281 to 0x4422B8, 0x4422FD to 0x442334). After a step along its row a two-hex stack still
covers one of its two former hexes. So a two-hex stack with one hex on a pit that walks on along its
row with that hex leading is stopped again after one step, its other half now on the pit, and a move
and attack ordered with that step loses its attack (0x478971); the turn after it is free. Walking
the other way along the row, or leaving the row, it is free at once. The path and the cursor allow
the whole move, since they go by the stops. Example: Cavaliers whose front hex stands on a pit are
ordered three hexes forward: they advance one hex and stop, so the pit holds them two turns, not
one. Not observed (the report: confidence medium, the code is plain but has no second source). VCMI
1.7.5 passes the hexes a stack starts on (`server/battles/BattleActionProcessor.cpp` `moveStack`)
and has a setting, off by default, that lets a two-hex unit moving one hex forward or back trigger
obstacles (`oneHexTriggersObstacles`, `config/gameConfig.json`); the wiki was not checked. The
engine leaves the start hexes out for the whole walk (BAT-MAG-032): it does not follow this yet:
BAT-Q28.

**Review re-read (E04b T11b, 2026-10-12): no attack is offered from a seen pit.** The interface's
path, which the ledger's BAT-QUIRK-2 left unread. With the cursor on a stack to attack, 0x474690
fills the active stack's twelve attack slots around it: a neighbouring position is taken only if the
reach map marks it, the stack fits there (0x446C40) and neither the position's hex nor, for a
two-hex stack, its other hex is in the stops array (0x4747BE to 0x474810; the same for a two-hex
stack's shifted position, 0x47485F to 0x4748B5). 0x474A00 then takes the attack's from-hex from the
slot the mouse points into, and from nowhere else (0x474B4F). The pathfinder's own marking of attack
targets passes over a position with a hex on a stop in the same way when the range asked is within
one move, as the cursor's is (0x4B378D to 0x4B37D1; for a longer range such a position counts, one
move dearer, 0x4B378A). The stops are the pits the stack's side sees (above), so the original offers
a player no attack that ends on a pit his side sees; the attack skipped after a stop ("The walker's
stops", above) is reached only through a pit the side does not see, or through a two-hex stack's own
pit hex (the re-read before this one). By the report the computer player's query refuses such an
attack too (not re-read here); the wiki and VCMI were not compared. The engine follows this since the user's decision of 2026-10-12 (BAT-Q30): it lists and accepts no attack whose walk stops in a pit the stack's side sees, in any profile, and the flag `quirk.battle.attack_from_seen_pit` of BAT-QUIRK-2 is retired (BAT-MAG-032).

**Information the original gives away** (fixed, Q-0002): Fire Wall's and Force Field's target test
(0x5A39C0) refuses a hex with the other side's hidden pit, so the cursor shows it; the stops of a
hypnotized stack are its real side's, so its controller sees where the owner's pits lie.

**Review re-read (E04b T11b, 2026-10-12): a berserk stack's turn and its own side's pits.** The
pathfinder marks the stops for every caller (0x4B353E calls 0x4B3290), the berserk turn's two among
them: the map over the whole field from which the turn takes its candidates (0x4B2DA0, §7.14) and
the walk towards the chosen one (0x4222C0). The stops are those of the stack's real side (0x4B3384
to 0x4B33C4: the pit's owner byte against the stack's +0xF4, or the revealed byte), its side's
hidden pits included. So the original's berserk stack ranks its targets by paths around its own
side's hidden pits and walks around them, in the open: the other side, who knows the rule and the
field, can tell from a detour, a farther target or a turn without a move where such a pit lies. The
review's security report did not read the original; the wiki and VCMI were not compared. The engine
does the same (BAT-MAG-025 plans with the pits the stack's side sees), so here it follows the
original. One corner was not read to its end. The berserk turn asks the pathfinder with a range
above the stack's speed (127; 0x445582, 0x42232C), and for such a query a position on a stop does
count as a place to attack from, one whole move dearer (0x4B378A to 0x4B3792), where the cursor's
query within one move counts none (the re-read above). The walk then attacks only if the target's
cost is within the stack's speed (0x4223E1) and otherwise walks towards it (0x41F580, not read): a berserk stack whose attack position lies on a pit its side sees does not strike that turn, and whether it ends its walk on that pit is not known; the engine's takes no such position and goes for another stack or passes (BAT-MAG-025; the flag of BAT-QUIRK-2, under which it walked into the pit, is retired, BAT-Q30).
The user decided on 2026-10-12 to keep the engine as the original here, in every profile (BAT-Q31; noted in the ledger beside BAT-QUIRK-4).

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
Resurrection's hit points at the caster's Earth mastery for the health test; and it must pass
spell 40's own test too (below).

**Review re-read (E04b T11b, 2026-10-12): the hero's raise spells search twice.** The cast routine
finds the stack at the aimed hex as above (0x5A008D calls 0x5A3950), but the handler of Resurrection
and Animate Dead does not raise that stack: it takes its position hex (+0x38) and runs the search
again from there (0x5A1907 to 0x5A1928: 0x5A3F50 for Animate Dead, else 0x5A3CC0, with the acting
side and that hex) and raises what the second search returns. A two-hex stack's position is one of
its two hexes (the rear one, by the report's reading of 0x444297), and each hex keeps its own corpse
list. So for a two-hex corpse A with a later corpse B of the same side on A's position hex, B
passing the test: aimed at A's other hex, the first search finds A and the second B, and B is
raised; A cannot be raised while B lies there. The rule that follows: a hero's raise spell raises a
dead stack only if it is the topmost corpse on its own position hex that passes the test. Sacrifice
(0x5A1A49) and the Archangel (0x4485BE, §7.21) raise the stack their one search found. Not observed
(the report: medium to high); neither VCMI nor the wiki was compared. If an experiment confirms it,
it is an oddity of the original whose class is the user's to decide. The engine raises the corpse
aimed at through either of its hexes (BAT-MAG-035): it does not follow this yet: BAT-Q28.

**Resurrection and Animate Dead** (0x5A1907): hit points `power × factor + effect[mastery]` plus
the specialty bonus 0x4E5FF0 with the target's level (§7.12: `ceil((hero level div (c + 1)) × t ×
0.03)`); temporary when the caster's mastery is below Advanced (Resurrection only). The raise
(0x5A7560, stack, hit points, temporary): the stack's total health (0 if dead) plus the hit points
gives the count `ceil(total / health)` and the top creature's loss, both capped at the start count
(full health then); a temporary raise adds the creatures gained to the counter +0x54, which is
capped at the start count and written nowhere else (0 at setup, 0x43D861; no loss lowers it). A
dead stack comes back on its own hex (0x43EA70 revives it, 0x4687C0 places it) and leaves the
corpse lists; at the end the raise clears the dead bit `0x200000` (0x5A7867) and touches no other
flag, so a stack that had acted this round before it died stays done (and through later rounds
while it lies dead: the review re-read below). Its spells ended at its
death (0x444120 ends every active spell, then sets the dead bit), Disrupting Ray excepted (§7.9);
the raise writes no defense, so Disrupting Ray's and Acid Breath's (§7.20) defense loss stays on a
raised stack.

**Review re-read (E04b T11b, 2026-10-12): a dead stack's round state is frozen.** The round start
calls 0x447120 for every record with a creature type (0x476107), and 0x447120 returns at once for a
record without creatures (0x44712D), before anything it does: clearing the bits good morale, waited
and acted (`0x7000000`, 0x44713F) and the turn-started byte +0x4F0, and setting the retaliations
(§7.10). The raise clears only the dead bit (above), and the pick passes over a record with the
acted bit (0x46511C, §7.14). So a stack killed after it had acted keeps "acted" and the retaliations
it died with through every round start while it lies dead: raised in a later round it gets no turn
in that round, and acts again from the round start after it. A stack killed before it acted, and
raised later, does act. (The start of a round's wait phase clears the waited bit of every record,
dead ones too, 0x4652CD.) By the report nothing but the good-morale roll (0x4649E6) clears a battle
stack's acted bit otherwise; its search of the whole code for the masks was not repeated here. Not
observed (the report: confidence medium for want of an observation, the effect large in ordinary
play if it holds). VCMI 1.7.5 resets every unit at a new round, dead or alive
(`lib/battle/BattleInfo.cpp` `nextRound`, `lib/battle/CUnitState.cpp` `afterNewRound`), as the
engine; the wiki was not checked. The engine resets dead stacks at every round start too, so a stack
raised in a later round acts in it (BAT-MAG-035): it does not follow this yet: BAT-Q28.

**Sacrifice** (0x5A199C): the victim is the living stack on the cast's second hex. 0x5A8090 (case
0x5A8523) wants it living (`0x10`), not summoned (`0x400000`), not undead, with creatures, and of
the caster's side by both its controller (the other side while hypnotized) and its real side; the
raised stack takes the immunity factor of spell 38 as above (the finder) and then that of spell
40 too: the target test 0x5A39C0 asks 0x5A8090 about the spell itself for the stack found, with
its fifth argument set (the spell book 0x59F1FC, Sacrifice's first cursor 0x5A2D00 and the cast
0x5A02EA all pass 1 for the stack to raise), which skips the controller's test (0x5A82B9) and in
case 0x5A8523 wants the creature flag `0x10`, no flag `0x400000` and fewer creatures than at the
start, then the general tests (the Fire school against fire immunity `0x4000`, 0x44A7BA; level
5): Efreet, living or dead, and summoned stacks are never raised by Sacrifice. The victim takes
the factor of spell 40 with the argument 0 (level 5: no creature's spell-level immunity reaches
it). The hit points are `(the victim's creature's
health from the creature table + effect[mastery] + power) × the victim's creatures`: the base
health, not the stack's (+0xC0). The victim takes its whole health as damage (0x444090), is flagged
sacrificed (no corpse) and the raised stack gets the hit points as a permanent raise. The victim
is compared with the raised stack in Sacrifice's second cursor (0x5A2E90, a callback the spell
book sets at 0x59F349, as the first cursor 0x5A2D00 at 0x59F32A): on a mouse move it asks the
target test with the victim's argument (0x5A2F11: 0 where the first cursor passes 1, 0x5A2D7D)
and then refuses the hex when the stack on it is the stack on the first target's hex (+0x44;
0x5A2F3B to 0x5A2F63). With a dead first target no stack stands there, and every victim passes.
Batch 2d RA1 observed the refusal.

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
record was reused. So a new stack acts in the round it appears, and a killed clone's or summoned stack's record (and so its place in the turn order's tie-break) goes to the next stack the side creates. A clone whose rounds ran out, and an original turned into one, dies with its count ("A clone's end") and keeps its record: the side's records in use never fall for it. 0x43D730 also clears the town of creatures 112–115 unless a game setting
(`[0x6994E8]+0x1F698`, not read) is set, which takes their native-terrain bonus away; this
applies to army stacks too and is left for BAT-TER-001 (experiment list). Batch 2d (NE, SU; SoD 4.0 Complete, plain HD) showed Air Elementals in an army and summoned Air and Earth Elementals with the native bonus on grass: in that setup the setting is on, or this reading is wrong; the engine keeps them natives.

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
sides lose"), also when the other side retreated or surrendered. Batch 2d confirmed the clone's place (up-right for the attacker, up-left for the defender), its full shots, its end by any damage and by its rounds, a clone on its own crowded original (CL2: the original vanishes), the summons' rows, the reused record and the limit, the end against only summoned stacks by elimination, retreat and surrender (the surrender's gold is still paid), and Necromancy counting created stacks (NE: 7); the surrender price (read at the E04b T11b review, below) leaves created stacks out. The experience of the battle
(0x46A350) counts no killed summoned stack, clone or war machine; Necromancy (0x4775E1) counts the
killed creatures of every record, summoned stacks and clones included.

**Review re-read (E04b T11b, 2026-10-12): the surrender price.** Read for the first time, in the
battle window's handler of its surrender button (control 2001, 0x474FE9 to 0x47507F). Over the
acting side's records 0 to 19 it passes over a record without a creature type or without creatures,
one with the summoned bit `0x400000` (0x47501F; a clone has it too, above) and one with no more
creatures than its temporary counter +0x54 (0x47502A), and for the rest adds `the creature's gold
cost (the creature table's +0x38) × (creatures − temporary counter)`. Nothing asks for the
war-machine flag: war machines count. The sum is halved as a whole number, towards 0, turned into a
single-precision value, multiplied by the hero's factor and truncated (0x47506B to 0x47507F). The
factor (0x4E4580) is `1 − discount`: the Diplomacy value of §7.4 (0, 0.2, 0.4, 0.6; grown `× (1 +
0.05 × level)` for a Diplomacy specialist) plus 0.1 for each of the artifacts 66, 67 and 68 the hero
wears (also as a part of a worn combination), every sum stored in single precision, at most 0.9. Two
differences from the engine follow. Creatures raised for the battle only (Basic Resurrection, §7.18)
are left out of the price: 5 of 10 Swordsmen at 300 gold cost 750, and after a raise to 8 still 750,
where the full count gives 1 200. And the halving comes before the factor: one Pixie (25 gold) at a
discount of 10 % costs `trunc(12 × 0.9)` = 10, where one division at the end gives 11 (the report
found no such case with discounts of 20, 40 and 60 %). Batch 2d showed created stacks left out and
war machines counted; the rest is not observed. VCMI 1.7.5 takes cost × count of the units from the
army's slots (`lib/battle/Unit.cpp` `getRawSurrenderCost`); the wiki is silent, by the report. The
engine counts every living creature of an army stack and divides once at the end (BAT-FLE-002): it
does not follow this yet: BAT-Q28.

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
  entry in the active spells; each Rust Dragon strike lowers it again). The defense it tests and
  lowers has a defending stack's bonus inside it (§7.9, the review re-read).
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
above 0; Bind's (+0x2B8) is not tested. The return is a move like any other: 0x445950 needs a hex
where the stack fits (0x446C40), clears the entered-hex marks and marks the hexes it stands on
(0x46A520), and the flight (0x4B49C0) enters the hexes it lands on (§7.15), each damage followed
by the rebirth check (§7.21). So a Harpy bound by the Dendroids' retaliation flies back
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
  factors (Air and Storm Elementals double, the golems) and Protection from Air; then, if the
  result is above 0 (else the case leaves without a report, 0x4411FD), the hit points are taken
  (0x444090) and the kills reported.
- *Serpent Fly, Dragon Fly*: if the test for Dispel Helpful Spells (78) passes (0x5A8090's own case:
  no if T's controller's hero wears the Sphere of Permanence, T8; yes if T has an active spell of
  positive type; no immunity, resistance or aura), it is cast at once (creature cast, mastery 0,
  power 3). Then a Dragon Fly, if T is not under Weakness (+0x24C) and passes the test for Weakness
  (45), casts Weakness at once **at mastery 2** (Advanced: `v` = 6) with power 3.
- *Rust Dragon* (0x44149F): if T lives, a 20 % draw, then `random(min, max damage) × S's creatures
  / 2` (truncated; S's +0xD0, +0xD4) taken through 0x444090 if above 0, with no test and no
  reduction (after Acid Breath, step 2); a message, then the pending spells are cast again
  (0x468990).

**The other sources** (compared at the E04b T11b review, 2026-10-12; the wiki was not consulted).
VCMI 1.7.5 (`server/battles/BattleActionProcessor.cpp` `attackCasting`, `config/creatures/*.json`,
`config/spells/ability.json`) agrees on the cast after an attack or a retaliation on a living
target, with the immunity test before the draw and no Magic Mirror; on the chances of the
Unicorns, Medusas, Basilisks, Scorpicores, Zombies, Ghost Dragons and Thunderbirds (20 %), the
Wyvern Monarchs (30 %), the flies, the Dendroids and the Rust Dragon's defense loss (100 %); on 10
damage per Thunderbird and 3 rounds for a creature's cast; on a petrified stack's halved damage,
a paralysed stack's retaliation at 25 %, Acid Breath's permanent −3 and Dispel Helpful Spells'
positive effects. It differs on ten points, on each of which the review's re-read confirmed this
reading (reading / VCMI): the Curse of Black and Dread Knights 25 % / 20 % and of Mummies 25 % /
50 % (0x440711); the Rust Dragon's extra damage 20 % / 30 % (0x4414BC); the Unicorns' Blind at
mastery 0, so the blinded stack retaliates at half / at level 3, no retaliation; the Dragon Flies'
Weakness at mastery 2, −6 / at level 0, −3 (0x44147C); the Thunderbirds' bolt tested and dealt as
Lightning Bolt (0x4411D4, 0x4411F5) / a spell of its own, which Lightning Bolt's immunities and
Protection from Air do not meet; Stone Gaze resisted as a negative spell / no resistance roll;
Poison a tenth at the cast and at each round start down to half, kept until a Cure / 10 % per own
turn down to 30 %, given back at its end; Age's `trunc(H / 2 + 0.95)` (25 → 13) / a whole-number
−50 % (25 → 12, not run); Bind ended at once when the last binder moves, dies or is teleported,
or after 255 rounds / at the bound stack's turn start when no binder is adjacent, never by its
rounds. VCMI also rolls magic resistance against Dispel Helpful Spells. The engine follows this
reading on all of them; the conflict is on record in BAT-Q28 and in the VCMI sources of
BAT-MAG-040 to BAT-MAG-047.

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
E04c) raise its mastery to 3. A creature's cast has no hero bonuses in the handlers inside the
cast routine (its area spells and Chain Lightning: the review re-read below), and the aimed
stack never rolls for resistance (§7.8; area spells still roll per stack).

**Review re-read (E04b T11b, 2026-10-12): the hero's damage bonuses in a creature's area spell and
chain.** The cast routine keeps a casting hero that is null for caster kind 1 (0x59FE63 to
0x59FE7B), and the damage handlers written inside it pass that one to the damage modifiers 0x5A78E0
(Magic Arrow at 0x5A09DC; by the report Ice Bolt, Lightning Bolt, Implosion, Death Ripple and
Destroy Undead alike), which apply the caster's bonuses 0x4E5760 only when given a hero (0x5A790D):
a creature's bolt has none, as above. The handlers that are functions of their own are not told the
caster kind and fetch a hero themselves: the area handler 0x5A4970 takes the hero of the acting side
(+0x53CC by +0x132C0, 0x5A4A05 to 0x5A4A1F) and passes it as the casting hero for every stack it
damages (0x5A4AB0 to 0x5A4AD2), Chain Lightning's handler 0x5A6360 does so for every target
(0x5A6557 to 0x5A656E), and so does Armageddon's (0x5A4C5A to 0x5A4C90; no creature casts it).
During a creature's cast the acting side is its controller's (0x465389, §7.14). So by this reading a
Faerie Dragon's Fireball, Frost Ring, Inferno, Meteor Shower and Chain Lightning (46 of its 100
draws) take the Orb, Sorcery and specialty bonuses of its side's hero (§7.8, step 1), and its Ice
Bolt, Lightning Bolt and Magic Arrow do not: one dragon's Fireball under the Orb of Tempestuous Fire
deals 120, not 80 (`5 × 10 + 30`, × 1.5), its Magic Arrow 70 either way. Not observed. VCMI 1.7.5
gives a unit's cast no such bonus (`lib/battle/CUnitState.cpp` `getSpellBonus`); the wiki was not
checked; the wiki agrees (Sorcery: "Sorcery also boosts damage spells cast by creatures under the
hero's control"; Faerie Dragon: the bolts take neither Sorcery nor the orbs; the wiki checks,
2026-10-13). The engine follows the reading since the user's decision of 2026-10-13 (BAT-Q28 row
2; BAT-MAG-048, BAT-MAG-049).

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

**Who may cast, and at what** (re-read for T7b1, BAT-MAG-048). 0x4476C0(hex), the test behind the
cursor (0x47670B, 0x476736) and the AI's loop (0x420E15): the casts left are not 0; magic is
allowed for the caster's side, the controller's (+0x288 flips the side of a hypnotized stack):
0x41F890(side, 0) refuses on battlefield type 2 (Cursed Ground; a hero's cast, flag 1, passes it
there and needs a spell book instead, artifact 0), at an anti-magic garrison (+0x53C4, set at the
battle's start for object 33 with subtype 1, 0x4645B1) and while either hero wears the Orb of
Inhibition (126); then the hex is on the field and, by creature (table 0x447A00), the Ogre Mage
and the four elementals need a stack on the hex (0x4E7170) that 0x5A3C80(spell, side, stack, 1,
1) accepts (0x5A8090 above 0: immunities, Anti-Magic, the spell's side test). The cursor
(0x4766EA) offers the cast only over a living stack that is not the caster itself (cursor 5)
and, but for a Faerie Dragon, of the acting side (+0x132C0): **no caster casts at itself**. The
dispatcher (0x47936E) calls 0x448260, sets the stack's bit `0x4000000`, rolls good morale
(0x464920, as after a walk, an attack or a shot; not after Wait or Defend) and does not re-enter
the stack's hexes (no 0x441F70, which Wait, Defend and a shot or attack from where it stands
call), so a caster standing in a Fire Wall does not burn for casting. A stack created in battle
is set up from its creature's table entry (0x47A100, §7.19), so a clone of a caster has its
creature's full casts.

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

**The Faerie Dragon, re-read for T7b2a** (BAT-MAG-049, 2026-10-08). *The draw* (0x447510, called
from the activation 0x465330 for creature 134): only while the casts left are not 0; the weights
of 0x63B850 (pairs of spell and weight, ended by a negative spell) are summed, `rand() mod sum` is
walked down the pairs in table order (Ice Bolt, Lightning Bolt, Fireball, Magic Arrow, Frost
Ring, Chain Lightning, Meteor Shower, Inferno) and the first pair whose weight takes it below 0 is
stored at +0x4E0. Unless 0x46A4A0 answers (not read), a combat-log line names the creature and
the spell, but only if the controller's side (+0xF4, flipped by +0x288) is a human player's
(+0x54A4[side]; the review re-read on bad morale, below). The activation returns at once in the
tactics phase (+0x13D68), and the battle
loop calls it again for a waited stack and after good morale (0x464920 clears the done bit
`0x4000000` and sets `0x1000000`), so the dragon draws anew each time. +0x4E0 is read only for the
active stack: its cast (0x448260), the F key's case (0x475CD8 → 0x59EC50(spell, 1); the earlier
readings took it for a button), the stats dialog's cast button 0x468954 and the status line
0x492FD3; nothing shows another stack's drawn spell. *The target*
(0x4476C0, case 7 of the table 0x447A00 indexed by creature − 13: 13 Archangel and 51 Pit Lord
case 0, 37 Master Genie 1, 91 Ogre Mage 2, 123, 125, 127, 129 the elementals 3 to 6, 134 the
Faerie Dragon 7): a stack on the hex (0x4E7170) that 0x5A3C80(drawn spell, the controller's side,
stack, 1, 1) accepts, which is 0x5A8090(…, reflected 0, …) above 0: its polarity test refuses an
own stack for a negative spell only, so Chain Lightning and the area spells (neutral) may be aimed
at the dragon's own stacks, unlike a hero's Chain Lightning (0x59F7BF). The cursor (0x4766EA)
refuses the caster's own hexes and, for creature 134 only, calls 0x4476C0 over a stack of either
side (0x4766FC); over an empty hex it goes to 0x47684B, whose cast test 0x4473D0 answers only for
the Archangel and the Pit Lord: by the cursor **the dragon never casts at an empty hex**, its
area spells too, and a Frost Ring must be centred on a stack it can affect, which the ring then
spares (the F key aims otherwise: the review re-read below). *The cast*
(0x448260, case 134): 0x59FE30(+0x4E0, hex, 1, −1, 2, 5 × creatures). *Magic Mirror*
(corrected at the E04b T11b review, 2026-10-12: the reading of T7b2a had the redirect arrive at
the cast): the key enters the cast through 0x59EC50 with action 10 (+0x3C = 10 for its second
argument 1), which draws the redirect as for a hero's cast (§7.13), but the creature's action
hands the new hex to nobody (the review re-read below). 0x59FE30 skips the test and roll at
0x5A02CB for caster kind 1, and the damage handlers (Magic Arrow 0x5A098A) have no immunity
test of their own; both only ever meet the aimed stack. So in the original no own Black Dragon or Earth Elemental takes the dragon's mirrored Magic Arrow or Lightning Bolt, nor in the engine since BAT-QUIRK-9's flag was retired (2026-10-12, BAT-Q29). *Survey S-18*: a
Magic Arrow from one dragon is 5 × 10 + 20 = 70 before the target's factors (§7.8), a hero's at
least 20 (power ≥ 1, HER-DET-001), so neither falls below Gold Golems' 7 or Diamond Golems' 20;
in SoD only a protection on a Diamond Golem cuts a power-1 Magic Arrow's 1 to 0, the truncation
every spell takes (the user: not reproducible, no flag, BAT-Q11).

**Review re-read (E04b T11b, 2026-10-12): Magic Mirror never redirects a creature's spell.** The
redirect is drawn in one place, the player's targeting routine 0x59EC50 after its cursor loop
(0x59EE7A to 0x59EF8B, §7.13), which stores the new target's hex in the manager's +0x48 (reset to −1
at each entry, 0x59ECB0). Only the hero's action hands that field on: the dispatcher's case for
action 1 passes +0x40, +0x44 and +0x48 to the cast routine as spell, hex and second hex (0x479036 to
0x479056), and the cast routine looks for a redirected stack only when its second hex is not −1
(0x5A0275 to 0x5A027E). The creature's action (action 10) passes the target hex +0x44 alone to
0x448260 (0x47936E to 0x479382); every cast of 0x448260 gives the cast routine the second hex −1
(the Faerie Dragon's at 0x448656), as do the Master Genie's (0x447FC4) and the Enchanters' at their
activation (0x448249); 0x448260 has no other caller, and neither it nor the cast routine reads the
manager's +0x48. So a redirect drawn on the Faerie Dragon's key path (the next paragraph) is
dropped, the mouse path draws none (its click handler queues action 10 with +0x44, by the report),
and no creature's spell is ever sent away: the skip at 0x5A02CB only ever meets the aimed stack, and
the original has no behaviour of the kind BAT-QUIRK-9 describes. VCMI 1.7.5 agrees: only a hero's
spell is reflected (`lib/spells/BattleSpellMechanics.cpp` `isReflected`); the wiki was not checked.
The engine follows this since the user's decision of 2026-10-12 (BAT-Q29): only a hero's spell is redirected, by the spell or by the Faerie Dragons' own 20 %, in every profile, and the flag `quirk.battle.creature_mirror_untested` of BAT-QUIRK-9 is retired (BAT-MAG-022, BAT-MAG-049, BAT-MAG-057). The wiki agrees ("Spells cast by creatures … will never be reflected", the Magic Mirror page).

**Review re-read (E04b T11b, 2026-10-12): the Faerie Dragon's key path and its targets.** The
original casts the drawn spell in two ways, and the reading above followed the mouse cursor alone.
The other is the F key (the draw's log line names it, by the report): the key handler's case for key
code 33 (0x475C17), outside the tactics phase and for an active stack of creature 134 with casts
left (0x475C87 to 0x475CD2), calls the targeting routine 0x59EC50 with the drawn spell (0x475CE3)
and, when it comes back with an action, makes it action 10 (0x475CF3); the stats dialog's cast
button does the same (0x468954). On this path the spell is aimed as a hero's (§7.8): the area spells
20 to 23 go to the handler 0x59EF9B and its cursor loop (0x59F700), which takes every hex outside
the two side columns that 0x5A39C0 accepts, and 0x5A39C0 accepts every hex for them. The cast
(0x448260's case) then asks only that the hex is on the field (0x448645). So by the key a Fireball,
Frost Ring, Meteor Shower or Inferno (41 of 100 draws) may be aimed at an empty hex or an obstacle
(and at the dragon's own hex, by the report); the mouse path gives the hexes of stacks the spell can
affect, as above. For the single-target spells both paths ask the same stack test; for Chain
Lightning the key refuses the acting side's stacks (0x59F7BF) and the mouse path does not. Neither
path takes a hex in the side columns, by the report (the engine takes a two-hex target's hex there).
VCMI 1.7.5 casts a unit's spell at the action's target, a location for an area spell
(`server/battles/BattleActionProcessor.cpp` `doUnitSpellAction`); the wiki was not checked. Two
corners are not settled: 0x59EC50 returns at once while the acting side's hero has cast this round
(the mark +0x54B4, 0x59EC76; unless the byte +0x13D74 is set, which another report of the review
found toggled by a menu command only), so then the mouse path alone is left; and it asks the acting
side's hero for its mastery without a test that there is one (0x59ECB4), so what the key does for a
side without a hero is not known (the report: probably a crash). Not observed. The engine takes the
cursor's rule for every drawn spell (BAT-MAG-049): it does not follow this yet: BAT-Q32.

**Review re-read (E04b T11b, 2026-10-12): the key path asks no "magic allowed" test.** The test
0x41F890 (§7.23) has six call sites: the cursor's test 0x4476C0 (two), the empty-hex test 0x4473D0,
the Enchanters' 0x447FE0, the computer player's 0x4396E0 and the cast routine, which asks it for
caster kind 0 only (0x59FED5 skips it otherwise). The Faerie Dragon's key path (0x475C17, 0x59EC50
and its cursor loop, action 10, 0x448260, the cast routine as kind 1) passes none of them, and by
the report no other test of artifact 126 lies on it. So by this reading the Orb of Inhibition, an
anti-magic garrison and Cursed Ground do not stop a Faerie Dragon's cast by the key, as the wiki
says of the Orb and the garrison (§7.23); on Cursed Ground the effect test still answers 0 for every
spell above level 1 (§7.8), so such a spell is cast and affects nothing. Batch 2e's row OI asked for
a spell cursor over a stack, which is 0x4476C0's answer; the key was not pressed. Not observed (the
report: medium to high for the reading). The wiki agrees (the wiki checks, 2026-10-13), and the
engine follows the reading since the user's decision of 2026-10-13 (BAT-Q32 (2)): the Faerie
Dragon has `ignores_magic_block` (BAT-MAG-061). The key path's own targets are not modelled
(BAT-Q32 (1)): under the Orb the engine still offers the cursor's.

**The Master Genie, re-read for T7b2b** (BAT-MAG-050, 2026-10-08). The target test 0x4476C0
switches through the byte table 0x447A00 (creature − 13) to eight cases: 0 Archangel and Pit Lord,
1 Master Genie, 2 Ogre Mage, 3 to 6 the Ice, Magma, Storm and Energy Elementals, 7 the Faerie
Dragon. *Case 1*: a stack on the hex for which 0x447A80 accepts at least one spell of 10–69
with flag `0x800` (Shield, Air Shield, Fire Shield, the four Protections, Anti-Magic, Magic Mirror,
Cure, Bless, Bloodlust, Precision, Stone Skin, Prayer, Mirth, Fortune, Haste, Slayer, Frenzy,
Counterstrike: 21); the cursor (0x4766EA) offers only stacks the acting side owns, not the caster.
*The cast* (0x448260 case 37 → 0x447EE0): the same count n, `k = random(1, n)`, the k-th in spell
order through 0x59FE30(spell, hex, 1, −1, 2, 6). *0x447A80(spell, stack)*: 0 if the spell is active
on the stack (+0x198 + 4 × spell) or 0x5A3C80(spell, the acting side, stack, 1, 1) refuses it,
then a switch on spell − 27 (byte table 0x447E8C, targets 0x447E68); every spell not named here is
useful. Shield and Fire Shield: over the records of the side opposite the acting one (by owner),
skipping the dead (bit 21), the blind (+0x290), petrified (+0x2B0) and paralysed (+0x2C0) and
creatures 147 and 148 (First Aid Tent, Ammo Cart), one that cannot shoot now; Air Shield: one that
can. "Can shoot now" is 0x4428F0: the Ballista and the arrow tower always; otherwise a shooter
(flag `0x4`) with shots, not blocked at either hex (0x469600) unless its controller's hero wears
artifact 137, and not forgetful at mastery 2 or more (+0x28C, +0x4C4). The Protections,
Anti-Magic and Magic Mirror: the opposite hero wears artifact 0 (a spell book). Cure: the top
creature lost hit points (+0x58 above 0). Bloodlust: the stack cannot shoot now (the Ballista and
the tower never qualify); Precision: it can. Prayer: the stack's bit `0x4000000` (acted this round)
is clear. Slayer: a record of the opposite side with creature flag `0x80` (king 1), whether dead
or alive: dead stacks count and king 2 or 3 alone does not (the user: the rule in every profile,
BAT-Q12; ledger BAT-QUIRK-10). The Enchanter (0x447FE0, T7b4) uses the same test.

**The raisers and the end of the battle, re-read for T6c and T7b3** (BAT-Q13, 2026-10-08).
*The Pit Lord* (0x448260's case → 0x5A3E40, 0x5A7390). The side is the caster's controller (+0x288
flips it); the cursor (0x4477BF) and the cast both call 0x5A3E40(side, hex): the hex must be
unblocked (hex byte +0x10 bit `0x2`) and hold no living stack (+0x18 below 0); its corpses from the
last added to the first: the first of that side whose creature is living (flag `0x10`) and, for a
two-hex corpse, whose other hex is unblocked with no living stack. There is no spell test (no
immunity, Anti-Magic or resistance). The cursor then needs 0x447330 above 0. *0x447330(caster,
target)*: for an Archangel (13) `min(start − count, 100 × Archangels div health)` of the target
(+0x60, +0x4C, +0xC0), the creatures its raise brings back; otherwise (the Pit Lord) `min(start,
min(50 × Pit Lords, start × health) div 35)` with the corpse's start count and
health (35: the Demon's health, creature 48 of the creature table). *0x5A7390(caster, corpse)*: removes the corpse from
its hexes' lists (0x5A7320, both hexes), then 0x47A100(controller's side, 48, the count, the
corpse's hex, flags **0**, animation 1) and sets the new record's +0x5C (army slot) to the
corpse's. The Demons are no summoned stack: they count for the winner (0x4658B0), leave a corpse,
give experience and count for Necromancy, and the army rebuild puts them into that slot. With 20 records in use and no dead summoned one without creatures, 0x47A100 returns 0 and 0x5A7390 writes through it: the
game crashes (the engine refuses the raise then). *The Archangel*: the cursor's case 0 calls
0x5A3CC0(side, hex, 1) (Resurrection's search and test, §7.18) and needs 0x447330 above 0.
*Clone on its original* (BAT-QUIRK-7): 0x5A6C70 calls 0x47A100 at the clone's position, then takes
the stack on that hex (0x4E7170) as the clone, whether 0x47A100 created it or not: if the
position was the original's, the original gets `0x400000` (not `0x800000`), the clone's rounds
(+0x2C) and itself as both links (+0x24, +0x28).
*The end of the battle.* 0x47A100 sets a created record up through 0x43D8B0 with army slot −1
(+0x5C) and 0x43D730 (count +0x4C, temporary +0x54 = 0, start +0x60 = the count), so a reused
record starts afresh. The rebuild 0x4641F0(side) empties the army's seven slots, then for each
record in use: creatures above 0, not dead, not summoned (only while the side has a hero,
+0x54A8 not −1), not a clone, not a war machine, army slot 0–6: that slot gets its creature and
count (a later record wins a shared slot). Necromancy (0x4775E1, the winner's hero) sums over the
loser's records 0–19 with a creature: `killed = start − count`, and if above 0, `min(killed,
trunc(f32(min(health, raised health) × killed) × rate / f32(raised health)))`, at least 1 in all
if any record had kills; summoned stacks, clones and war machines count. The experience
(0x46A350) sums `killed × health` over the opposite side's records that are neither summoned nor
war machines (then hero and game-setting terms, not read here). The death 0x444120 does not touch the dying stack's count
(a clone whose rounds ran out keeps it), but sets an original's clone to 0 before killing it.

**The Enchanter, re-read for T7b4** (BAT-MAG-053, BAT-Q14, 2026-10-09). *When it casts.* The
activation 0x465330 sets the acting side to the stack's controller (+0x288 flips +0xF4) and
returns at once in the tactics phase. The battle loop's pick 0x465080 does not activate a blind,
petrified or paralysed stack (0x465222), nor one that bad morale or a second test (0x465283, not
read) stops; a berserk stack is activated (its automatic action comes after), and so is a stack
that waited or got good morale, again. For creature 136 the activation calls 0x447FE0 when the
acting side's counter +0x132A0 is above 2, and sets that counter to 0 when it cast (0x465503).
The counters rise by 1 for both sides at every round start (0x4760F8), which runs when a round
ends (0x479C7E) and when the tactics phase ends (0x474559, 0x47525F), not before a battle's first
round. Nothing sets them when a battle starts: the battle manager is made once per session
(0x4EDCF5; its constructor 0x462760 leaves +0x132A0 alone), so the counters carry over from
battle to battle, by side (BAT-QUIRK-14; the engine starts every battle ready, BAT-Q14). *What it
casts* (0x447FE0). Magic must be allowed for the acting side (0x41F890(side, 0): Cursed Ground,
an anti-magic garrison, the Orb of Inhibition; E04c, T8). For each pair of the table 0x6608B8 the
side tested is the acting side, or the other one when the spell's record field +0 is below 0
(Slow, Weakness; 0x448042); the loop calls 0x447A80(spell, the side's record 0) once per record
in use without advancing the record (0x44807C), so a spell counts when the side's first record
accepts it (survey S-12, BAT-QUIRK-13). A dead first record accepts nothing: 0x5A8090 refuses a
dead stack (flag `0x200000`) for every spell but 38 to 40. If the counted weights sum above 0,
`rand() mod sum` (0x4480B8, the C library's generator) is walked down the table again with the
same test, each counted weight subtracted until the remainder is below 0. Unless the battle is
hidden (0x46A4A0), the controller's living Enchanters that are not blind, petrified or paralysed
then show the cast (0x4481B9; animation only), and 0x59FE30(spell, hex −1, caster kind 1, −1,
mastery 3, power 3) casts it (0x448250): the Expert mass version over the side the spell aims at,
every stack rolling for resistance as in an area spell, the timed spells for 3 rounds. No cast is
used (+0xDC untouched) and the stack's turn goes on as usual. If nothing counts, nothing is cast
and the counter stays, so the next activation of an Enchanter of that side tries again.

**The mana creatures, re-read for T7c1** (BAT-MAG-054 to BAT-MAG-056, BAT-Q15, 2026-10-09).
*The cost* (0x4E5240: hero, spell, enemy army, battlefield). Titan's Lightning Bolt (57) costs 0
and nothing else applies. Otherwise the cost is the spell record's at the hero's mastery (0x4E5100;
Armageddon at Expert while the hero wears Armageddon's Blade, T8); then, only if an enemy army is
given (0x4E52DA; a cast always gives one, +0x54C4 of the side opposite the acting one, 0x59FFCF),
**+2** if that army holds creature 20 or 21 (Pegasi, Silver Pegasi) and **−2** if the hero's own
army (hero +0x91) holds 34 or 35 (Mages, Arch Mages), once each; **at least 1**. 0x44AB80 compares
the seven slots' creature types only, and the armies stay as they were until the battle's end
(the rebuild 0x4641F0), so a stack that died still counts and a stack created in the battle never
does (batch 2a R6 round 3: 5 with the Pegasus dead; BAT-Q15: the rule, ledger BAT-QUIRK-15). The
AI (0x43C996) and 24 other call sites (the spell book's among them, not read) ask the same
routine; the cast pays it (0x59FFDF, then 0x4D92D0: spell points less the cost, at least 0) after
magic was allowed for the acting side (0x59FEEF).
*The channel* (0x5A2170). The 35 spell handlers end in a common tail (0x5A2058: every record's
cast animation, a hero's casting animation) that reaches it; it acts only for a hero's cast
(caster kind 0) when the side opposite the acting one has a hero (+0x53CC) and the paid cost
(0 for every other cast) is 5 or more. It sums the creatures (+0x4C; a dead stack has none) of
the records of creature 43 (Familiar) whose controller (+0xF4, flipped by +0x288) is not the
acting side; if the sum is above 0, that hero's spell points (+0x18, a word) rise by `cost div 5`,
once whatever the number of Familiars and with no cap at its maximum, and a combat-log line
follows. The loop walks both sides' records, but each only up to the record count of the side
opposite the acting one (+0x54BC read once, 0x5A21B2; 0x5A21CC): a Familiar of the caster's side
that the enemy hypnotized is missed when its record lies beyond that count, and when the caster's
side has fewer records, records past its count are read (BAT-Q15: always fixed, ledger
BAT-QUIRK-16). A clone of a Familiar counts (a record of its own).
*The drain* (0x465548, the activation's case for creature 61, Wraith). Nothing in the tactics
phase (the activation returns before, 0x4653B6) or in the round's wait phase (+0x13DE4, H3API's
`waitPhase`: the pick 0x465080 sets it when no stack is left to act, clearing every record's
waited bit `0x2000000` so the waited stacks can act, 0x4652A7; the round start clears it,
0x4760AB). Otherwise the hero of the side opposite the Wraith's controller, if there is one and
its spell points are above 0, loses 2, at least 0, and a log line follows: once per activation of
a Wraith stack, whatever its count. So a Wraith drains at its first activation of a round, at a
good-morale turn of the normal phase and, berserk, before its automatic action, but not when it
acts after waiting nor at any later activation of that round; a blind, petrified or paralysed
Wraith, or one bad morale stops, is not activated (0x465222, 0x465283). A hypnotized Wraith drains
its owner's hero.

**The creature defences, re-read for T7c2a** (BAT-MAG-057, BAT-MAG-058, BAT-DMG-010, BAT-Q16,
2026-10-09). *The Faerie Dragon's mirror* (0x4487F0, Magic Mirror's chance): the spell's stored
`v` (+0x4E4) while its duration (+0x228) is above 0, else 0; for creature 134 the larger of that
and spell 36's table value at no mastery (spell table +0x1354: 20). Only the cast's entry
(0x59EC50, §7.13) and the AI's spell evaluation (0x43A670) ask it; Hypnotize's refusal (0x5A8090,
case 60, 0x5A83A6) reads the spell's duration, so a Faerie Dragon can be hypnotized. The dragon's
own mirror thus sends a hero's single-target cast at it to a random living stack of the caster's
side as the spell does; a creature's cast is never sent away (the review re-read above; nor in the engine since 2026-10-12, BAT-Q29). *The Efreet
Sultan's shield* (0x443130, Fire Shield's factor): the stored `f32` (+0x4A0) while the spell's
duration (+0x20C) is not 0, else 0.2 (an `f32`) for creature 53, else 0. The reflection 0x422440
tests the same (the duration, else creature 53, 0x42244C) and runs as for the spell (§7.11): melee
only, no fire-immune striker, `trunc(f32(min(stage 1, hit points)) × factor)`, then spell 29's
damage modifiers with the Sultan's controlling hero as the caster, so that hero's Orb of Fire,
Sorcery or Fire Shield specialty raise the Sultan's own shield too. No pack value of the spell is
below 20 % (20, 20, 25, 30), so "the spell's value while active" and "the larger of the two" never
differ. *The elementals' factors* (stage 2, 0x443B90; `this` the striker, the argument the
target): right after the defense factor, × 0.5 (a double, exact) when the striker is creature 120
(Psychic Elemental) and the target stack has flag `0x400` (mind immunity, 0x443C0B), and × 0.5
when the striker is creature 121 (Magic Elemental) and the target is creature 121 or 83 (Black
Dragon, 0x443C2D); then the wall, distance and melee penalties (§7.10). One striker meets at most
one of the two.

**The Phoenix's rebirth, read for T7c2b** (BAT-MAG-059, BAT-Q18, 2026-10-09). *The check*
0x469440 walks the records of side 0, then side 1. For every record of creature 131 with the dead
bit `0x200000`, casts left (+0xDC) above 0 and not a clone (`0x800000`): one cast less, count
(+0x4C) and lost hit points (+0x58) set to 0, `n = start (+0x60) div 5` plus one for each of
`start mod 5` draws with `rand() mod 5 = 0` (the C library's). Only if `n > 0`: the acted bit
`0x4000000`, a sound, and the raise 0x5A7560 with `n × health (+0xC0)` hit points (0x46959E to
0x4695AD; the health as the death routine left it: 0x444120 has ended every timed spell, and
Age's end 0x444740 gave the half back, so `n` whole creatures come back) and its last argument 0,
the *temporary* flag (§7.18): a permanent raise, so the creatures count as the stack's own at the
battle's end (no `+0x54`). The raise revives the dead stack (0x43EA70), places
it on its hex +0x38 (0x4687C0 writes the hex records' stack fields, whatever stack they named) and
removes its corpse from its hexes. A stack whose draws give 0 stays dead with its cast used.
*Where it runs.* At the end of the strike action 0x445A60 (it shoots, for the Ballista (146), an
arrow tower (149) or a shooter that may shoot, through 0x440160, else picks the side to strike
from and calls the melee 0x441CB0 with its retaliation and second strike; then it sets +0x132C8
and calls the check), which the attack 0x478900 and the dispatcher (0x47927A) call; then the
dispatcher sets the active stack's acted bit, tests the end (0x4762F0) and rolls good morale
(0x464920, which clears the acted bit: a Phoenix killed by the retaliation and reborn may get its
morale turn). In the cast routine 0x59FE30 after the damage of Magic Arrow, Ice Bolt, Lightning
Bolt and Titan's Lightning Bolt, Implosion, Chain Lightning and Death Ripple (0x5A0A26 to
0x5A0E74), in the area handler 0x5A4970 and the mass handlers 0x5A4BC0 and 0x5A67C0; after Land
Mine's (0x59FB40) and Fire Wall's (0x59FCD0) damage on a hex a stack enters (0x46A570), and after
the moat's (0x469E50, E04c). **Not after Sacrifice** (0x5A199C): its victim dies through 0x468990
with its casts left and no corpse, so the next check, at any damaging action of either side,
brings a sacrificed Phoenix back onto its old hexes (BAT-QUIRK-17, fixed: never reborn). Every
other death of a Phoenix is followed by a check in the same action (clones are excluded), so
outside Sacrifice a dead Phoenix with a cast left died in the action just checked.

**Fear, read for T7c3** (BAT-MAG-060, BAT-Q18, 2026-10-09). The battle loop's pick 0x465080
(§7.14) runs, for the stack it picked and outside the tactics phase (+0x13D68) and the round's
wait phase (+0x13DE4), and only when called with argument 1, bad morale 0x464B40 and then Fear
0x464D40 (the site 0x465283); either answering skips the stack and picks again. Fear spares a
stack with creature flag `0x20000` (no morale: the undead, the elementals, the golems, the war
machines) or of creature 132 (the Azure Dragon), sums the counts (+0x4C) of the records of creature
132 of the side opposite the stack's controller (+0xF4, flipped by +0x288; the records by real
side, so a hypnotized Azure Dragon frightens the side that controls it, BAT-QUIRK-18) and, when
the sum is above 0, freezes the stack if `rand() mod 10 = 0` (the C library's): the acted bit
`0x4000000`, `Fear.wav`, a combat-log line and an animation. The callers with argument 1 are the
dispatcher after an action (0x479C73, 0x479C87) and the round start after a tactics phase
(0x474562, 0x475268); the battle start 0x462A20 picks the first stack with argument 0, so neither
bad morale nor Fear can freeze the battle's first stack when no tactics phase runs (BAT-MOR-004
had it checked until T7c3). *Good morale* (0x464920) clears the stack's done bit and sets
`0x1000000`; the dispatcher's next pick then finds the stack among those not done: its checks run
again (bad morale cannot hit a stack with good morale; Fear can, outside the wait phase), and a
stack its own action left blind, petrified or paralysed is passed over (0x465222, its turn having
started) until that ends, when it may act later in the round (BAT-MOR-003, the T7b4 review's
finding). Such a stack picked in the wait phase is not checked for bad morale either, though it
never waited: BAT-MOR-004 tested the stack's own waited bit (VCMI's test) until E04b T11a.

**Review re-read (E04b T11b, 2026-10-12): bad morale and who commands the side.** The bad-morale
check 0x464B40 (side, record), read in full for the first time: the stack's morale (+0x4E8) is
clamped to ±3 and `random(1, 12)` is drawn; a draw above −morale ends the check (so 1, 2 or 3 of 12
freeze: the catalog's 2, 4 and 6 of 24). Then the manager's byte +0x54A4 of the stack's real side is
read (0x464BDB): if it is 0, a second draw `random(1, 4)` follows, and a 1 ends the check without a
freeze (0x464BE6 to 0x464BF8). Only after that the acted bit is set (the sound and the log line
follow, by the report). The battle's setup fills the byte per side (0x4639F0): for a side with a
player, whether the byte +0xE2 of the player's record is set (0x4CE940, written at 0x463AA8), and 0
for a side without a player (0x463AED). +0xE2 is the record's human flag by the adventure lane's own
reading (0x4BADA0) and by H3API (`IsHuman`; the manager's field `isNotAI`); the byte beside it,
+0x54A6, wants the record's +0xE2 and +0xE1 both set (0x463ABD; H3API has both as human flags, the
report takes +0x54A6 for the local human). So by this reading the stacks of a computer player and of
a neutral army escape one freeze in four: 6.25 %, 12.5 % and 18.75 % at morale −1, −2 and −3, where
a human's freeze at 8.3 %, 16.7 % and 25 %. Good morale (0x464920) and Fear (0x464D40) have no such
branch, by the report. Unsure: what the two record bytes mean rests on those names and on the
setter, not on a reading of how the game fills them; not observed. VCMI 1.7.5 rolls alike for every
army (`server/battles/BattleFlowProcessor.cpp` `tryActivateMoralePenalty`), and the wiki, as the
catalog quotes it, has no such rule. The engine rolls once for every side and is not told who
commands one (BAT-MOR-004): it does not follow this yet: BAT-Q33.

**Review re-read (E04b T11b, 2026-10-12): the order in the wait phase.** When the pick 0x465080
finds no stack for the normal phase, it sets +0x13DE4, clears the waited bit of every record in use,
of both sides, dead or alive (0x4652A7 to 0x4652E5), and picks again; from then on every stack that
is not done (acted or dead) is a candidate alike, whether it had waited or not. The comparison
0x464F50 puts a stack with the good-morale bit `0x1000000` first in either phase (0x464F60 to
0x464F82), then the arrow tower and the Catapult (sieges), and of two speeds the higher first in the
normal phase and the lower first while +0x13DE4 is set (0x464FDB to 0x465007). So a stack that comes
to its first turn of the round in the wait phase (summoned or cloned in it, or seeing again in it)
takes its place among the waited stacks by its speed, the slower first: stacks of speed 2 and 5
wait, the slower acts in the wait phase and its hero summons elementals of speed 7: the stack of
speed 5 acts before them. The pick runs no turn start (0x446E30) in the wait phase (0x465211 to
0x46521D), so a stack first picked there gets none that round. The Wait button (control 2009, whose
handler queues action 8, 0x4751B8) is given the control state 0x1000 while +0x13DE4 or the tactics
phase is on (0x478755 to 0x478790: read as disabled; the state's meaning is H3API's guess). The
dispatcher's case for action 8 (0x479906), which the report did not read, has no test of the phase
at its head: it sets the waited bit, and nothing clears that bit again before the round ends, so a
Wait that did arrive there in the wait phase would cost the stack its turn; whether the keyboard can
send one was not read. Not observed (the report: confidence medium); neither VCMI nor the wiki was
compared. The engine puts a stack that has not waited before every waited one, fastest first, in the
wait phase too, and lets it Wait there and act once more (BAT-TRN-002): it does not follow this yet:
BAT-Q28.

**Review re-read (E04b T11b, 2026-10-12): a freeze and the side that acted last.** For two stacks of
equal speed and different sides the comparison prefers the one whose side is not the manager's
+0x132B8 (0x46502A to 0x46504C; with equal sides the lower record). +0x132B8 and +0x132BC are the
side and the record of the stack activated last: the activation writes them (0x465362), and the
cursor and the key handler read them as the active stack (0x4746A7, 0x475C87). Besides the battle's
setup the side is written only at the end of the tactics phase (0x474539, 0x475233, 0x475F6D: 1, so
that on a tie the attacker's stack goes first), in the pick's Catapult case (0x4650BB, sieges) and,
saved and restored, in the melee routine (0x441D54 and 0x441D87, 0x441F13 and 0x441F44). A stack
that bad morale or Fear freezes goes back to the pick's loop without an activation (0x465283 to
0x46529C), as a blind stack does, so its side is not recorded: after a freeze the tie-break still
goes by the side of the stack that acted before it. Example: B's stack of speed 9 acts, A's stack of
speed 6 freezes, and one stack of each side with speed 5 is left: A's goes first, B having acted
last, where counting the freeze as A's action gives B's. Not observed (the report: confidence
medium). VCMI 1.7.5 makes a frozen unit the active one (`server/battles/BattleFlowProcessor.cpp`
`makeAutomaticAction`), as the engine; the wiki's wording fits both, by the report. The engine
counts a frozen stack's side as the side that acted last (BAT-TRN-001): it does not follow this yet:
BAT-Q28.

**The other sources for the casting creatures** (compared at the E04b T11b review, 2026-10-12; the
wiki was not consulted). VCMI 1.7.5 agrees on the Archangel (Expert Resurrection, 100 hit points
per creature, one cast), the Faerie Dragon's weights but Magic Arrow's, the rebirth's fifth and
its draws, Fear's 10 % after bad morale and the Enchanters' counter. It differs from the readings
above, which the review's re-read confirmed, in: the Ogre Mage's Bloodlust lasting 3 rounds (the
reading: power 6, 0x4486A7); obstacles acting on a unit after its spell (the reading: no re-entry
after the cast, 0x47936E); a resistance roll for every unit against a negative spell whatever the
caster (the reading: none for the aimed stack of a creature's cast, 0x5A02CB); the Master Genie's
20 spells without Frenzy, lasting 5 rounds, Prayer without a condition and Slayer for a living
enemy of any king level (`getRandomBeneficialSpell`); the Enchanters' six spells without weights
(no Bloodlust, no Cure); the Phoenix's cast used only when a creature comes back, the reborn stack
without retaliations and back at the moment of its death (`lib/CStack.cpp`); Fear only for living
creatures (so not the Gargoyles, which 0x464D4C to 0x464D57 frighten) and rolled in the wait phase
too; the Familiars' channel computed before the cast's effect, so Familiars the cast kills still
channel. The engine follows the readings; the conflict is on record in BAT-Q28 and in the rules'
VCMI sources (BAT-MAG-048 to BAT-MAG-060).

### 7.22 Code reading: the hero's available spells and its mastery of a spell (E04b T8a, 2026-10-10)

Read for E04b T8a (BAT-Q19). Both are functions of the hero record, run outside the battle and
read by every caller: the adventure map's spell book, the battle's book, its cast check and its
creature code all read the hero's **available spells** (`+0x430`, 70 bytes, one per spell id
0–69), which only 0x4D95D0 writes; the **learned spells** (`+0x3EA`, the same shape) are what the
hero keeps without artifacts, read by learning (Mage Guild, scrolls on the map) and by Eagle Eye's
test in the cast routine (0x59FF6A; T8d).

**Available spells** (0x4D95D0, run when the hero's artifacts change): copy the learned spells,
then for each of the 19 body positions (`+0x12D`, 8 bytes each: artifact id, then a second word;
the backpack is never read): a Spell Scroll (artifact 1) adds the spell its second word names; an
artifact whose setup record has `+0x1D` set (H3API's `newSpell`) adds the spell set 0x4D9350
returns for it; a combination (`+0x14` not −1) adds the sets of those of its parts that have
`+0x1D`. The loader 0x44CE61 sets `+0x1D` from the list 0x63B998 (nine ids: 1, 128, 123, 124, 86,
87, 88, 89, 135) and `+0x1C` ("disabled") from 0x63B98C (141–143, the unused SoD rows). Sets add,
never remove: an available spell stays available whatever else is worn.

**The artifact spell sets** (0x4D9350, a switch on the artifact id): the four Tomes give every
spell 0–69 with their school's bit (Fire 86: bit 2, Air 87: bit 1, Water 88: bit 4, Earth 89:
bit 8; the adventure spells included, the creature abilities 70–80 never); Spellbinder's Hat (124)
every spell 0–69 of level 5; Armageddon's Blade (128) Armageddon (26); the Sea Captain's Hat (123)
Summon Boat and Scuttle Boat (0, 1; the Admiral's Hat gives them as its part); Titan's Thunder
(135) Titan's Lightning Bolt (57). Every set but Titan's Thunder's then **clears spell 57**
(0x4D9532): Titan's Lightning Bolt is an air spell of level 5, so the Tome of Air and the Hat
would give it otherwise. The spell table's flag `0x2000` (§7.6, "granted by an artifact") is set
on spell 57 alone, so the packs carry the exception as a spell attribute harvested from that flag
(`granted_by_name_only`): a grant by school or level skips such a spell, a grant naming it gives
it. Equipping Titan's Thunder also puts a Spell Book into an empty book position (0x4E2A64 in
0x4E2A00, H3API's `GiveArtifact`): an equipping rule for the adventure epics (E08–E10); the
details a battle gets from then on simply hold the book.

**Mastery of a spell** (0x4E5080, H3API's `GetSpellExpertise`; arguments spell and battlefield):
Armageddon (26) is cast at Expert while the hero wears Armageddon's Blade (its 19 positions, then
a combination holding it); otherwise 0x4E5100 takes the spell's school bits and the battlefield:
the battlefield value 1 (Magic Plains, `+0x53C0`) gives Expert to every spell, 6, 7, 8 and 9 to
the water, fire, earth and air spells (the elemental battlefields; E04c names them); else the best of
the hero's Air (`+0xD8`), Fire (`+0xD7`), Earth (`+0xDA`) and Water (`+0xD9`) Magic levels among
the spell's schools, 0 without one. No specialty, artifact or other bonus enters it. The spell's
cost (0x4E5240) takes its mastery from here. The engine splits the function at its argument: the
Heroes context derives each available spell's mastery on ordinary ground (HER-SPL-003); the
battlefield's raise to Expert is the battle's (E04c, which passes it here as the original does).

### 7.23 Code reading: the casting limits and the artifacts' spell defences (E04b T8b, 2026-10-10)

Read for E04b T8b (BAT-Q20). Every test below asks the hero record with 0x4D91F0, which finds an
artifact in the 19 body positions or as a part of a worn combination (the backpack never counts).

**Who may cast** (T8b1, BAT-MAG-061). 0x41F890(side, hero cast) answers 0 on Cursed Ground for
a creature's cast (a hero's needs its spell book there instead; E04c), at an anti-magic garrison
(+0x53C4; E04c) and **while either hero** (+0x53CC, +0x53D0) **wears the Orb of Inhibition**
(126). Its callers are every cast there is but the Faerie Dragon's by the F key, which passes
none of them (§7.21, the review re-read): the hero's cast in the cast routine (0x59FEEF: a
message, no cast), the creature cast and raise test 0x4476C0 (twice: before the by-creature switch
and in the Archangel's and the Pit Lord's case), the empty-hex test 0x4473D0, the Enchanters'
cast 0x447FE0 (it returns 0 then, so the activation 0x465330 keeps the side's count; nothing is
drawn) and the computer player's evaluation 0x4396E0. The Faerie Dragon's draw at its activation
(0x447510) asks nothing: the dragon draws and may not cast. The strike's creature magic (§7.20)
is no cast and goes on. The hero's spell book button (0x59E900, the human player's path) refuses
earlier, with the artifact's message, when the side has no hero, has cast this round, at an
anti-magic garrison or under the Orb on either hero, before it opens the book.

**The spell level limit** (T8b1, BAT-MAG-062). After the book returns a spell, 0x59E900 refuses
one above level 1 on Cursed Ground (E04c) and one above level 2 **while either hero wears
Recanter's Cloak** (83), each with a message. The effect test 0x5A8090 (§7.8) answers 0 for a
spell above level 2 under the Cloak on either hero (0x5A80F6), right after its Cursed Ground test
and before every other one (Dispel's and the Sphere's, Anti-Magic, the polarity tests, 0x44A4D0):
positive spells and the creature abilities too. So under the Cloak no creature cast of such a
spell finds a target (0x5A3C80, §7.21), the Enchanters' table counts no such spell, the strike's
Stone Gaze (level 3), Paralyze (4) and Age (5) do nothing (0x5A8640, §7.20) and an Archangel
raises nothing, its search 0x5A3CC0 testing every stack with Resurrection (38, level 4; §7.18).
The Pit Lord's corpse search 0x5A3E40 calls no effect test: Pit Lords still raise.

**Whose defences** (T8b2). 0x5A8090 takes the hero controlling the stack (0x442690: a hypnotized
stack's controller) and the hero of the casting side, and passes both to 0x44A4D0. The
controlling hero's artifacts decide, in this order (each answer 0 returns at once):

1. 0x5A8090, before Anti-Magic: the Sphere of Permanence (92) for Dispel (35, before Dispel's own
   tests, which then return 1 without 0x44A4D0) and for Dispel Helpful Spells (78); after
   Anti-Magic and the dead and clone tests, Armageddon's Blade (128) for Armageddon (26), then the
   polarity tests.
2. 0x44A4D0, first: **Power of the Dragon Father** (134) for every spell of level 1 to 4 (+0x18 at
   most 4; level-0 creature abilities too), whatever its polarity; then Dispel's Sphere again; war
   machines against flag `0x1000`; the arrow tower.
3. Its per-spell switch (spell − 17, table 0x44A92C): the **Pendants** (Negativity 106: Lightning
   Bolt 17 and Chain Lightning 19, not Titan's; Life 103: Death Ripple 24, which also spares the
   undead; Death 104: Destroy Undead 25, only undead; Holiness 102: Curse 42; Dispassion 100:
   Berserk 59; Free Will 105: Hypnotize 60; Total Recall 107: Forgetfulness 61, which also needs a
   shooter; Second Sight 101: Blind 62, which also spares the undead and the Troglodytes 70, 71),
   Stone Gaze (70) spares the Troglodytes, Poison (71) non-living creatures and the Gargoyles (30,
   31). The same switch holds per-spell conditions (re-read for T8b3, BAT-Q22; pack format v32,
   BAT-MAG-067): Bless (41) and Curse (42) spare the undead and creatures with maximum damage 0,
   Precision (44) needs a shooter (`0x04`), Mirth and Sorrow (49, 50) spare creatures without
   morale (`0x20000`), Fortune, Misfortune and Slayer (51, 52, 55) creatures with maximum damage
   0. The values are the creature type's (the table behind 0x6747B0, 116 bytes per creature: flags
   +0x10, maximum damage +0x60), so the Ballista (2–3) takes Bless and Curse and the First Aid
   Tent and the Ammo Cart (0) take none of the five.
4. 0x44A746: while the **casting side's hero or the controlling hero** wears the **Orb of
   Vulnerability** (93) it returns 1 (a positive spell's 1 too): no mind, fire or per-creature
   immunity (the creature table 0x44A988: the elementals, the dragons' levels, the Black Dragon
   and Magic Elemental), no Dwarf or Crystal Dragon factor and no hero resistance (0x4E46E0), all
   tested after it. 0x5A8090 still multiplies the Unicorns' aura (× 0.8) into what it returns.
5. Otherwise a mind spell (flag `0x400`) returns 0 for a mind-immune creature or while the
   controlling hero wears **Badge of Courage** (49); then the rest of §7.8.

Titan's Lightning Bolt (57) is among the Earth and Magma Elementals' immunities in the creature
table (case 6 with 17, 19, 26) and among the Air and Storm Elementals' doubled spells in damage
stage 2 (0x44B4B0, §7.10): a `sod` override with each one's full ability list (the spell is
`sod`'s).

**The other sources** (compared 2026-10-10 after the user's remark; BAT-Q21). The wiki's artifact
pages and VCMI 1.7.5 agree with this reading on the Pendants, the Sphere, the Blade, the Badge,
the Dragon Father's levels 1–4 (the hero's own spells included), the Orb of Vulnerability's sides
and what it lifts and keeps, and on both heroes under the Orb of Inhibition and Recanter's Cloak.
They disagree on five points, open in BAT-Q21: Faerie Dragons under the Orb of Inhibition (the
wiki: unaffected, as by the anti-magic garrison; VCMI blocks no creature cast; this reading found
no path around 0x41F890, neither the human player's cursor 0x47670B nor the computer player's
0x421280; the review's re-read found one, the F key: §7.21); the Unicorn aura under the Orb of
Vulnerability (the wiki and VCMI negate it); the
Troglodytes' and Gargoyles' immunities under it (VCMI lifts them); the Cloak against the strike's
abilities and the Archangel (the wiki names the heroes only; VCMI bars creature casts, not
abilities); the Dragon Father against level-0 abilities (VCMI never bars level 0). Until an
experiment settles them, the engine follows the wiki and VCMI on the first two (the Faerie Dragon's
`ignores_magic_block`, the aura lifted) and this reading on the other three (the user, BAT-Q21).
A sixth point, found at the E04b T11b review (2026-10-12): VCMI does not agree on what the Orb of
Vulnerability keeps. In `lib/spells/TargetCondition.cpp` only immunities marked absolute (`addInfo`
1: in `config/artifacts.json` the Sphere of Permanence's two alone) stand before the Orb's negation;
the Pendants', Armageddon's Blade's and the Dragon Father's are plain bonuses there, which the Orb
lifts. This reading has them before the Orb's test (the per-spell switch at 0x44A5F6 and the
Dragon Father at 0x44A50F, both before 0x44A746) and the wiki keeps "artifact immunities"; no batch
observed it. The engine follows the reading (BAT-Q28). T8b3's
per-spell conditions (BAT-Q22): the wiki (the spell pages, `War Machine`, the `Spell Immunity Table`
template) and VCMI (`config/spells/timed.json`) agree on the undead under Bless and Curse, also with
the Orb, on Precision's shooters, on the Tent and the Ammo Cart under Bless and Curse (VCMI spares
every war machine) and on Mirth's and Sorrow's creatures without the Orb; VCMI differs on the
Ballista under Bless and Curse (the wiki: affected), on golems and elementals under the Orb for
Mirth and Sorrow (VCMI: cast, no effect; the wiki's table: golems keep them) and on Fortune,
Misfortune and Slayer at the Tent and the Ammo Cart (VCMI: cast, no effect; the wiki ambiguous).
The engine follows this reading on all three until batch 2e (the user, BAT-Q22). The rendered table
also lists the Troglodytes' Blind among the immunities the Orb keeps (BAT-Q21 point 3).

**Batch 2e** (the user, 2026-10-11, SoD with plain HD; `experiments/results/E04b-batch-2e.md`)
observed this reading on BAT-Q22's three points (WM, OV) and on BAT-Q21's points 1, 3, 4 and 5: the
Orb of Inhibition stops the Faerie Dragons (OI: no spell cursor in two rounds; the cursor's path
only, the F key was not pressed: §7.21, BAT-Q32), the Troglodytes keep
Blind and the Gargoyles Poison under the Orb of Vulnerability (OV: 0 of 12 attacks), the Cloak bars
the Archangel (RC) and, weakly, the Medusas' Stone Gaze (0 of 8) and the Dragon Father the bolt and
Poison (DF: 0 of 3 and 0 of 4; the bolt is the strike's Lightning Bolt, a level-2 spell, §7.20, so
for point 5, the level-0 abilities, only Poison counts: 0 of 4 at 30 % happens in 24 % of the
tries without the immunity). On point 2 it observed the wiki and VCMI: under the Orb of
Vulnerability no stack beside the Unicorns resisted Expert Slow (UA: 12 of 12 slowed; 7 % to see
that with the aura's × 0.8 kept), so this reading's aura in 0x5A8090 is either skipped on a path not
read or the factor is applied elsewhere; the engine keeps the aura lifted.

**Review re-read (E04b T11b, 2026-10-12): the aura under the Orb of Vulnerability, a second
reading.** An independent re-read finds what point 4 says and no path around it. At 0x5A8592 to
0x5A85CF, for a spell that is not positive at a stack whose aura list (+0x524) is not empty, the
result of 0x44A4D0 is multiplied by the single-precision 0.8, with no artifact test; under the Orb
0x44A4D0 returns 1 (0x44A746), so such a stack is affected with 80 %, and Expert Slow's mass handler
0x5A66D0 rolls `random(1, 100)` against that. By the report the executable tests artifact 93 in two
places only, both in 0x44A4D0, and 0x43EA70 fills the aura lists without an artifact test, at a
walk's end too (0x4423BE), so the three stacks of batch 2e's row UA had theirs. Against the reading
stands that row alone: 12 of 12 rolls slowed, which the reading allows in `0.8^12` = 6.9 % of the
tries, in a battle without a control (the same set-up without the Orb, to show that it produces
resists at all). The user decided with the wiki and VCMI (BAT-Q27). The engine lifts the aura under
the Orb (BAT-MAG-066): it does not follow this reading: BAT-Q28.

### 7.24 Code reading: the opening spells and Eagle Eye (E04b T8c, T8d, 2026-10-10)

Read for E04b increment 17 (BAT-Q23). Both are casts the hero does not choose: the opening spells
are cast for the hero by its artifacts, and Eagle Eye watches the enemy hero's casts.

**The opening spells** (T8c, BAT-MAG-068). The battle setup (0x463B05, in the loop over both
sides) sets a flag per side (+0x54B0 + side). A stack's activation (0x465330) first sets the
acting side (+0x132C0) to the stack's controller (a hypnotized stack's is the other side) and then
(0x4653A8) does nothing more in the tactics phase (+0x13D68): neither the opening spells nor the
Faerie Dragon's draw, the Wraith's drain or the Enchanters' cast (§7.21). Otherwise, while the
acting side's flag is set, it takes that side's hero (+0x53CC + 4 × side) and, if the hero wears
**Angelic Alliance** (129; 0x4D91F0, §7.23), casts **Prayer** (48), then, if it wears **Armor of
the Damned** (132), **Slow** (54), **Curse** (42), **Weakness** (45) and **Misfortune** (52), in
this order, and clears the flag (0x4654E5; also without a hero). The activation runs only for a
stack the pick 0x465080 lets act (after the blind, petrified and paralysed stacks, bad morale and
Fear, 0x465283), so a side whose first stack loses its turn casts at its next stack's. Each spell
goes through 0x5A40D0 first (its five calls are its only callers; the address map had named it
the creature casters' target check): it asks the target test 0x5A39C0 (§7.8) at every hex outside
the two side columns, at mastery 3 for the acting side, and casts only if one passes, so only if
some living stack would be affected (`f > 0`; a hypnotized stack counts there, though the mass
handler then passes over it). The cast is the cast routine 0x59FE30 with target hex −1, **caster
kind 2**, mastery 3 (Expert) and power 10 (Prayer) or 50 (the Armor's four).

**Caster kind 2 in the cast routine.** The casting hero is the acting side's (as for kind 0;
kind 1 has none). The mastery is the argument (Expert on Magic Plains too, 0x59FEBC); the hero's
own mastery is not asked. Only kind 0 takes the path at 0x59FEE4: the Orb of Inhibition, an
anti-magic garrison and Cursed Ground's ban (0x41F890, §7.23), Eagle Eye's roll (below), the cost
and its payment, the side's "cast this round" mark (+0x54B4). The power stays the argument and
is also the duration: the duration bonus of the Conjuring artifacts (0x4E4DB0, §7.9) is added
for kind 0 only (0x5A0098). The hero's casting animation plays as for kind 0. Target hex −1 has
no aimed stack (no roll at 0x5A02EA, no Magic Mirror); the five spells take the timed spells'
case (0x5A10AB), which with no target stack calls the mass handler 0x5A66D0 (§7.9) with the
casting hero and kind 2: each stack of both sides, attacker's first, skipping hypnotized ones,
is tested by 0x5A8090 (kind 2 changes nothing there; it matters only for the raising spells' kind
1) and rolls, and 0x4448F0 applies the spell with the hero, so the hero's specialty bonus counts
(Prayer and Weakness are among its spells, §7.12). The cost is 0, so no Familiar channel
(0x5A2170). So the opening spells are the hero's Expert mass casts without mana, cast mark,
duration bonus, Orb or garrison test, and per-stack tests and rolls as usual: the level limits
(Recanter's Cloak, Cursed Ground's level 1) and the defences of §7.23 apply per stack.

**Eagle Eye** (T8d, BAT-MAG-069). In the cast routine, for caster kind 0 only and after the Orb
test passed (0x59FF52): if the hero of the side opposite the acting one exists, has Eagle Eye
(+0xD4, the secondary skill 11, above 0), has not learned the spell (+0x3EA, the learned spells,
§7.22: a spell only an artifact gives does not count as learned) and the spell's level (+0x18 of
its record) is at most its Eagle Eye level + 1, it draws `random(1, 100)` from the battle's
generator (0x50B230) and, if the draw is at most `trunc(p × 100.0f)` (0x6179D4 truncates), puts
the spell into that side's set (+0x545C + 16 × side; 0x5A9450 is the C++ library's unique set
insert, so a spell is kept once, while each cast of it draws again). The draw comes before the
cost and before any roll of the cast itself (a Magic Mirror's redirection too); creature casts and
the opening spells draw nothing.

**The chance** `p` (0x4E4420, H3API's `GetEagleEyePower`): 0 without the skill (returned at
once); otherwise the table 0x63EA28 by level (`f32` 0.40, 0.50, 0.60), multiplied for a hero
whose specialty is the skill (specialty type 0, subtype 11) by `f32(level) × 0.05f + 1` (the
x87's 53 bits, stored as `f32`, as the other skill getters, §7.4); then `+ 0.05f` while it wears
the **Bird of Perception** (63), `+ 0.10f` the **Stoic Watchman** (64), `+ 0.15f` the **Emblem of
Cognizance** (65) (each a body position or a worn combination holding it; each sum stored as
`f32`); at most 1.0.

**After the battle** (the orchestrator's; BAT-Q24 answered as this code, confirmed by batch 2f). The battle's end 0x477470 does this for
the winning side only (0x47772F; none in a draw): 0x469FE0 has the winner's hero learn each spell
of that side's set (0x4D9330: learned and available) if it wears a Spell Book (artifact 0) and the
spell's level is at most its Wisdom (+0xD0) + 2. The side's experience (0x4E33B0) comes before
that unless the side's flag +0x54A6 is set (set at the setup, not read); then it comes after
it, in the results dialog's branch (0x477948), which also shows the "learned" message (0x476FE0,
with a pick-up sound drawn by `random(1, 7)`, sound only). The loser's set is dropped.

**The other sources** (2026-10-10, BAT-Q23). The wiki (the artifact pages; Eagle Eye's SoD text,
the second argument of its `swh` template) agrees with every point of the opening spells (cast
"once an allied creature is able to take its turn", later after a bad-morale turn; immunities,
Cursed Ground and "anti-magic artifacts" mitigate; it works in an anti-magic garrison; Loynis's
and the Weakness specialists' specialties improve it) and with Eagle Eye's chances, levels,
artifacts and specialty, the win and the Wisdom. VCMI 1.7.5 (`BattleFlowProcessor::castOpeningSpells`,
`BattleResultProcessor`) agrees on the spells, Expert, the durations, the specialty, the resistance
rolls and the Orb (its passive mode skips the magic block), but casts both sides' opening spells at
the end of the tactics phase, before round 1 (attacker first); it rolls Eagle Eye after the battle
over the loser's recorded hero casts (one roll per cast: the same odds), for a winner with units
left, without a book or Wisdom test. The engine follows the code and the wiki on the timing until
an experiment confirms it (BAT-Q23); the learning after the battle is the code's (BAT-Q24, batch 2f).

**Batch 2f** (the user, 2026-10-11, SoD with plain HD; `experiments/results/E04b-batch-2f.md`)
observed this reading: Armor of the Damned cast its four spells at the defender's first stack that
took its turn, its blinded first stack passed over (AD), and under an Orb of Inhibition (AO); a
winner learned only with a worn Spell Book (EB: none from the backpack) and only up to Wisdom + 2
(EL: Bless, not Air Shield, without Wisdom; EW: Air Shield with Basic Wisdom), as 0x469FE0 reads.
The message after the battle names every pick, the unlearned ones too.

### 7.25 Code reading: a stack's facing and a shot's full-damage distance (2026-10-13)

Read at the user's question after #150 (does the original measure a shot's distance from the front
hex, also for a shooter that walked past its target?). Insight only, nothing copied (Q29). Caller
path: the human player's shot action (0x440160), the damage's stage 2 (0x443B90) and the cursor
(0x476490); not the computer player's own routines.

**Facing.** A stack's facing +0x44 is 1 (right) or 0 (left); the setup sets 1 − side (0x43D846).
A stack's other hex (0x4466A0) is its position +0x38 for a one-hex stack, else the position + 1
facing right and − 1 facing left. **Turning around** (0x446720) flips the facing and, for a two-hex
stack, moves the position onto its other hex: the stack keeps its two hexes, its position stays the
hex behind it and its other hex the one ahead (the hexes' draw parts +0x1A are rewritten, the turn
animated on request). Twelve routines turn stacks: a walk's step faces the step's direction
(0x43F0B0: right for directions 0 to 2, left for 3 to 5; the walker 0x441FA0 turns back at its end
to the facing it started with, when its flag argument is set, 0x4423A2); the strike action turns
its attacker to face the target (0x445C6A) and back after it (0x445C9B); the melee attack turns the
struck stack to face its attacker (0x441D79) and back at its end unless the stack's flag bit 21 is
set (0x441F36); the creature's cast, the attack after a move, the flight, the teleporting move and
the raise call it too (not read whether they turn back).

**The shot** (0x440160). An arrow tower takes its own path (0x465AD0). Any other shooter first
compares the screen x (the hex record's first word) of the target's position hex with that of its
own other hex: a target to the right turns a shooter facing left around, a target to the left one
facing right; equal x, no turn. Then it shoots (0x43F900, which computes the damage), and at the
end turns back if its facing changed (0x4402DC).

**The full-damage distance** (0x467600, shooter and target; called by the damage's stage 2 at
0x443C96 and by the cursor at 0x4767E6). No halving when the shooter's side's hero (the other
side's while the shooter is hypnotized, +0x288) wears artifact 91 (Golden Bow) or 137 (Bow of the
Sharpshooter), nor for creatures 149 (arrow tower) and 137 (Sharpshooter). Otherwise the hex
distance (the usual hex metric) from the shooter's other hex to the target's position; at most 10
is full damage; else, for a two-hex target, the distance to its other hex decides the same way;
both above 10, the shot is halved.

**Consequence.** The damage is computed after the turn, so the distance runs from the shooter's
hex on the target's side to the target's nearest hex. A brute force over every placement of a
two-hex shooter of either side and a one- or two-hex target of either side on the 17 × 11 field,
not overlapping (252 120 pairs), gives exactly the smallest hex distance between the two stacks
(the turn's screen-x test never picks the farther hex). The engine had measured from the front hex
by side (VCMI's head, never turned): it differed on 7 084 pairs, a target behind the shooter 10
hexes from its rear hex and 11 from its front hex, which it halved. The cursor tests without
turning: its half-damage cursor (0xF) reads the shooter's current facing, so it can show the broken
arrow over a target that the shot will strike in full (presentation; not modelled). Two-hex
shooters: Medusas and Medusa Queens, Ice Elementals (AB) and the Ballista (whose targets are never
behind it).

**Sources.** VCMI 1.7.5 measures from the shooter's position (its head) to the target's nearest
hex (`CBattleInfoCallback::battleHasDistancePenalty`, `isEnemyUnitWithinSpecifiedRange`): as the
engine did. The wiki is silent on the hex ("Ranged attack": beyond 10 hexes). The reading stands
against VCMI; the user decided on 2026-10-13 (BAT-Q36) that the engine follows it (BAT-RNG-002),
and a confirmation row is queued (`docs/experiments/README.md`, batch 2g). Not read: whether the
other routines that turn a stack measure anything at its turned front hex before turning back (the
melee attack's breath and attack patterns: leads for a later reading; the engine measures those at
the front hex by side, BAT-GEO-002). **Second look** (a fresh agent's independent re-read of
0x4466A0, 0x446720, 0x440160 and 0x467600, 2026-10-13): it agrees on every point above; it traced
the damage path after the turn (0x43F900 → 0x443F40 → 0x443B90 → 0x443C96; the wall test 0x467510
at 0x443C79 gets the turned front hex too) and found 0x443B90 also reached without a turn through
0x443E30 from 0x437800 and 0x438910, likely the computer player's damage estimates (not read).

### 7.26 Code reading: the battlefield (E04c T1, 2026-10-13)

Read for E04c T1 (the battlefields; `docs/rules/surveys/E04c-battlefields.md`). Insight only,
nothing copied (Q29). Caller path: the battle's start (0x462A20) for the setup, the obstacles and
the background; the stack routines for morale and luck; the human player's casts (the book button,
the cast routine) for magic. The computer player's routines (0x4396E0, 0x43C330 to 0x43C944) read
the same fields and were not followed.

**The battle manager's battlefield fields**, set by 0x4643B0 at the battle's start: the terrain
+0x5394, the special field +0x53C0 (−1 none, 0 the shore, 1–9 below), the anti-magic garrison
+0x53C4, a battle on a ship +0x53C6. The routine starts with field −1 and both flags clear, then
takes the **terrain** from the first case that applies:

1. A siege (the town +0x53C8): the town's faction's native terrain (0x5C1440, table 0x643698).
2. Either hero (+0x53CC, +0x53D0) with the flag 0x40000 at +0x105 (in a boat: two of them are the
   "boats" picture below), or the battle's map tile (+0x53BC) is the entrance (tile +0xD bit 0x10)
   of a Shipwreck (class 85) or a Derelict Ship (24): **water** (8), and the ship flag +0x53C6.
3. The battle's map position (+0x13FF0) on the underground level: **subterranean** (6). This is
   survey S-22: every underground battle, whatever the tile's terrain.
4. The entrance of a mine (class 53) whose record (0x4FCC30, the game's mine list at +0x4E38C, 0x40
   bytes each) has type 4 (crystal) or 6 (gold) at +1, or a nonzero byte +2 (the abandoned mine):
   **subterranean**.
5. A tile with bit 0x2 of +0xD (next to water: the shore's use names it): **sand** (1), and field
   **0** (the shore).
6. Otherwise the tile's own terrain (+0x4, signed byte).

Then, in every case, the tile's **special ground** (0x4FCE20) by a switch on the class (0x46453E):
Cursed Ground (21) field 2, Magic Plains (46) field 1, Clover Field (222) 5, Evil Fog (224) 4,
Fiery Fields (226) 7, Holy Ground (227) 3, Lucid Pools (228) 6, Magic Clouds (229) 9, Rocklands
(231) 8; a garrison (33) sets +0x53C4 when its subtype is 1 (anti-magic); anything else changes
nothing (Favorable Winds 225 among them). So a special ground replaces the shore's field 0 and
keeps the terrain the cases gave (a Cursed Ground on the shore fights on sand), and a garrison is
never a special field. 0x4FCE20 answers 33 for an anti-magic garrison's tile or a hero standing in
one; otherwise it walks the tile's objects from the last to the first and answers the first class
among 21, 46, 222, 224–229 and 231; Cursed Ground (223) and Magic Plains (230), the SoD objects,
are not in its list (the map loader is not read: it probably gives them the classes 21 and 46).

**The picture** (0x4646D0, `docs/epics/E17-design.md` §5): a siege the town's; a field 1–9 its
own; both heroes in boats `CmBkBoat.pcx`; the ship flag `CmBkDeck.pcx`; field 0 `CmBkBch.pcx`;
else the terrain's by the surroundings' variant.

**Morale and luck** (0x469750 passes +0x53C0 to both routines, their only caller; they store the
results in the stack's +0x4E8 and +0x4EC; 0x469750 runs at the battle's start, 0x462E19, and after
every action, 0x479D16: E04c T3 read them from memory in batch 3).
- 0x43E000, a stack's morale: on field 2 (Cursed Ground) the morale is **0**, whatever its sources
  (0x43E00F, before anything else); a creature without morale 0. Otherwise the base (army, hero),
  + Mirth − Sorrow, then **by the creature's town** +0x74: field 3 (Holy Ground) +1 for towns 0–2
  (Castle, Rampart, Tower) and −1 for 3–5 (Inferno, Necropolis, Dungeon); field 4 (Evil Fog) the
  opposite; towns 6–8 and creatures without a town (above 8 unsigned) nothing (byte tables
  0x43E118, 0x43E130). Then the Minotaurs' minimum 1 and Spirit of Oppression (84) on either hero.
- 0x43DF20, a stack's luck: on field 2 the luck is **0**, as with Hourglass of the Evil Hour (85)
  on either hero (0x43DF2B to 0x43DF5E: no Fortune, no Halfling minimum). Otherwise the base,
  + Fortune − Misfortune, then field 5 (Clover Field) **+2 for towns 6–8** (Stronghold, Fortress,
  Conflux; table 0x43DFF4), creatures without a town nothing; then the Halflings' minimum 1.
- The game's own texts agree (`ARRAYTXT.TXT`): "No morale effects on cursed ground", "No luck
  effects on cursed ground", "Creature of neutral town alignment on Clover Field +2", "Creature of
  good town alignment on Holy Ground +1" and the other three.

**Native terrain** (0x43D730, the stack's setup): no stack is native on field 2 (0x43D7C1); else
native when its town's native terrain is +0x5394 (+1 speed, attack and defense, §7.16).

**Magic** (the readings of §7.21 to §7.24, with the field):
- The hero's mastery of a spell is 0x4E5080 with the field: Expert for every school on field 1
  (Magic Plains), for water, fire, earth and air on fields 6, 7, 8 and 9 (Lucid Pools, Fiery
  Fields, Rocklands, Magic Clouds), else its school levels (0x4E5100). Every reader passes +0x53C0:
  the cost (§7.21), the cast routine (0x59FEA2), Dispel's test (0x5A8167), the cursors.
- A cast that is not a hero's (caster kind 1, a creature's, also the strike's spells of §7.20; kind
  2, an opening spell) takes the mastery argument, **Expert on field 1 only** (0x59FEBC).
- Field 2 (Cursed Ground): no creature's cast (0x41F890 with flag 0, §7.23: the creature casts,
  raises and periodic casts); the hero's book refuses a spell above level 1 (0x59EB11); the effect
  test 0x5A8090 answers 0 for a spell above level 1 (0x5A80D1), so above level 1 no strike spell,
  creature cast or opening spell takes effect. A hero's cast needs its worn spell book on every
  field (0x41F8C3; the review's r6 point).
- The anti-magic garrison (+0x53C4): 0x41F890 answers 0 for every cast, the hero's and the
  creatures' (as the Orb of Inhibition; the Faerie Dragon's key path asks nothing, §7.21).

**Obstacles** (0x466290, the battle's start; a siege takes its own path, 0x4662DD, not read here;
+0x53C5 set skips all obstacles, not read which battles set it).
- **Both heroes in boats** on water: the 32 hexes of the table 0x63D368 get the blocked bit 0x2,
  as an absolute obstacle's (0x466710 to 0x46673D), and nothing else is placed: 6–9, 24–26, 58–60,
  75–77, 92–94, 109–111, 126–128, 159–163, 176–180 (VCMI's `ship_to_ship` lists the same).
- Otherwise a count of hexes to block, `random(5, 12)` (a siege of a town of type 6 can reach
  this path with another range; sieges are not read here), and a **mask**: the field's bit
  `1 << field` when +0x53C0 is not −1 (the shore included), else the terrain's bit
  `1 << terrain`. With `random(1, 100) ≤ 40` the absolute obstacles (0x4668A0) by the mask, the
  count reduced by half what they blocked; then usual obstacle types drawn without repetition
  (the sampler 0x50C6E0 over the types 0 to 90) until one has the mask's bit among its terrain
  bits (+0x0) or its field bits (+0x2), placed by 0x466010, until the blocked hexes reach the
  count. So on a special field **only the field's obstacles** appear, and a ship's deck (field −1,
  terrain water) takes the types with the water bit (type 55 alone).
- The two tables (usual types 0x63C7C8, absolute obstacles 0x63BEC0) equal VCMI 1.7.5's
  `config/obstacles.json` in every type's terrains, fields, size and blocked hexes (checked
  2026-10-13, 91 + 34 records); VCMI calls the water bit of type 55 the battlefield `ship`.
- Not read: how 0x466010 and 0x4668A0 choose positions (the engine keeps VCMI's, BAT-GEO-004).

### 7.27 Code reading: the siege's setup (E04c T4, 2026-10-13)

Read for E04c T4 (the siege survey and seam; `docs/rules/surveys/E04c-sieges.md`): what the battle
takes from the town at its start, and what it gives back at its end. Insight only, nothing copied
(Q29). Caller path: the battle's start (0x462A20 → the side setup 0x4639F0, 0x463C60, the war
machines 0x463600, the turret records 0x466C50) and the controls 0x4782D0. Not read here: the
walls' sections and hit points, the gate, the Catapult's shot, the turrets' shot and target choice,
the shot penalty through walls (0x467510), the Tower's land mines (0x4663BE), Earthquake: each
with its increment (T5–T8).

**The town and the fort level.** The side setup stores the town (+0x53C8) for every battle at a
town, with or without a fort (0x463C1A), and the fort level +0x132F4 from the town's built buildings
(`+0x150`, the per-building masks 0x66CD98), testing 7, 8 and 9 in that order: **1** Fort, **2**
Citadel, **3** Castle, 0 without one (so a town holds one of the three bits: an upgrade replaces its
predecessor's, not read). The town's special buildings (ids 17, 21, 22) are tested in a second
set, `+0x158` (H3API's `IsBuildingBuilt` with its flag reads it; what it holds beyond the built
special buildings is not read); the Tavern, the fort and the Grail in `+0x150`. Every reader of the town then tests the pointer, some the level too.
The town alone (any level): the battlefield's terrain is its faction's native one (§7.26 case 1),
the defending hero's bonuses below, the morale and luck below, the controls and the experience
below. A fort level above 0: the siege picture (0x4646D0), the attacker's Catapult, no defender's
Ballista, the siege obstacles (0x466290's own path).

**The moat** (two flags). With a Citadel or a Castle, +0x53A8 (a moat) is set unless the town is
a Tower (type 2) or a Stronghold (6) in a game under RoE or AB rules (the game's +0x1F698, H3API's
`mapKind`: 0 RoE, 1 AB, 2 SoD; **the game's version**, BAT-Q41: the saved-game loader 0x4BCDA0
restores it, the new game's setter is not read, probably from the map's format); +0x53A9 marks a Fortress (7). The moat's hexes (0x469DC0): the 11 of
0x63BCE8 (11, 28, 44, 61, 77, 95, 111, 129, 146, 164, 181) and for the Fortress also the 11 of
0x63BCF4 (10, 27, 43, 60, 76, 94, 110, 128, 145, 163, 180); hex 95 (94 for the Fortress's second
row) only while +0x53A4 is 3, the closed gate (§7.28). The
damage on entering a moat hex (0x469E50) is applied only under SoD rules (`mapKind` 2), straight to the
stack's hit points (0x444090, no defense, no resistance), by the town type (0x63BD18): Castle 70,
Rampart 70, Tower 150, Inferno 90, Necropolis 70, Dungeon 90, Stronghold 70, Fortress 90, Conflux
70; Tower's 150 is never used here (no moat flag: its land mines take the moat's place).

**The siege's obstacles** (0x466290 with a fort level above 0, from 0x4662E3). The walls' 18
part values +0x13F60 come from a per-town table (0x66D868, records of 36 bytes: `WALLS.TXT`,
§7.28), four of them raised by 1 with a Castle, and are copied to +0x13FA8; the 11 wall
hexes (0x63BD00: 12, 29, 45, 62, 78, 96, 112, 130, 147, 165, 182) get the blocked bit 0x2. With a
Citadel or Castle in a Tower, the land mines go onto moat hexes with the Tower's 150 (0x63BD20)
and the defending hero's mastery of Land Mine (0x4663BE; §7.28). Then no natural obstacle
under SoD rules; under older rules only a Stronghold with a Citadel or Castle (no moat there) goes on
to the usual obstacles.

**The war machines and the turrets** (0x463600, per side). In a battle with a fort level above 0:
the attacker's Catapult (creature 145) on hex 119 (every hero has one); the defender gets no
Ballista; the First Aid Tent and the Ammo Cart as elsewhere. With a Citadel or Castle the defender
gets the **keep**: an Arrow Tower stack (creature 149) at the off-field position 254, whose count
is 0x5BF4F0's result: 4 + the number of building ids 0–43 the town has built among its mask +0x160
(which buildings that mask holds is not read); with a Castle also the **upper and lower towers** at
255 and 251, each with half the keep's count rounded up. The turret records +0x13D78 (three of
0x24 bytes; 0x466C50) hold each turret's picture and position from the town type's record
(0x63CF88, 32 bytes). `CRTRAITS.TXT`: the Arrow Tower 100 hit points, attack 10, defense 5, damage
2–4, 99 shots (the harvest, E04c T7); the Catapult 1 000 hit points, attack 10, defense 10, no damage.
**The records' order** (re-read in E04c T10a): after the army's stacks (slots 0–6) the war
machines' records follow in one fixed order, each only where it is set up: the Catapult (side 0
with a fort level above 0, 0x46376D), the Ballista (artifact 4, 0x4637AB; not for side 1 with a
fort), the First Aid Tent (artifact 6, creature 147, 0x463819), the Ammo Cart (artifact 5,
creature 148, 0x46387B); then, for side 1, the turrets. VCMI numbers the war machines before the
army (Ballista, Ammo Cart, Tent, Catapult): the engine follows the code (survey D-23).

**The defending hero** (0x463C60, a battle at a town with a defending hero; the fort level is not
tested). The hero's attack, defense, power and mana are kept (+0x13DE8 to +0x13DF4), then raised by
the town's buildings, by the town type: Tower with its Grail (building 26) +150 mana; Inferno with
building 21 (Brimstone Stormclouds) +2 power; Dungeon with its Grail +12 power; Stronghold with its
Grail +20 attack; Fortress with building 21 (Blood Obelisk) +2 attack, building 22 (Glyphs of Fear)
+2 defense and its Grail +10 attack and +10 defense. The battle's end (0x4776E1 to 0x477719)
restores the attack, defense and power, and sets the mana to the smaller of the mana left and the
mana before the battle.

**Morale and luck.** 0x469750 passes the town to the defender's stacks only. The base morale
(0x44B065 to 0x44B0AD): +1 with a Tavern (building 5), +2 more in a Castle with building 22 (the
Brotherhood of the Sword; E04c T8: the Tavern is read in the set +0x150, where the Brotherhood
replaces it, so +2 in all, §7.32). The base luck 0x44B2D0 takes the town too (not read; Rampart's Fountain
of Fortune by VCMI and the wiki).

**The controls** (0x478573 to 0x478635). Surrender needs a hero on both sides and is refused to
the defender of a battle at a town. Retreat needs the side's hero and is refused to the defender at
a town unless the town is a Stronghold with building 17 (the Escape Tunnel).

**Experience** (0x46A405): the attacker's side gets +500 in a battle at a town (any fort level),
beside the +500 for a defeated hero (0x46A3DF; the −500 of 0x46A3FC, with two globals, not read).

### 7.28 Code reading: the walls, the gate and the moat (E04c T5, 2026-10-13)

Read for E04c T5 (walls, gate and moat; the survey `docs/rules/surveys/E04c-sieges.md`): the walls'
parts and their hexes, the gate's states and who passes it, the moat's hexes, stops, damage and
defense, the Tower's land mines, and the shot's wall test. Insight only, nothing copied (Q29).
Not read here: which part a destroyed hex belongs to and what destroys it (0x465990, the Catapult
and Earthquake: T6, §7.29), the turrets (T7).

**The walls' parts** come from `WALLS.TXT` (`H3bitmap.lod`, loaded at start by 0x462990 into
0x66D868: per town type 18 records of 36 bytes, each its hit points, its screen position, the hex
after which it is drawn and six pictures). The 18 parts, the same in every town (Rampart's keep is
its "Main Building"): Drawbridge, Drawbridge Rope, Moat, Moat Lip, Back Wall, Upper Tower, Upper
Wall, Upper Buttress, Mid-Upper Wall, Gate, Mid-Lower Wall, Lower Buttress, Lower Wall, Lower Tower,
Keep, and the covers of the keep and the two towers. The file gives 2 hit points to the two towers,
the four walls (Upper, Mid-Upper, Mid-Lower, Lower), the gate and the keep, 0 to the others (its
header: "max is 3, only walls, towers, the gate, and the main building are used"). At a siege's
start (0x4662E3) the 18 values go to +0x13F60, the three covers are set to 1, a Castle adds 1 to
the four walls (3 hit points), and the whole is copied to +0x13FA8. The 11 wall hexes 0x63BD00, one
per row (12, 29, 45, 62, 78, 96, 112, 130, 147, 165, 182), get the blocked bit `0x2`; a destroyed
part clears it on its hex (0x465990, T6). **The wall line** of a row is its wall hex: a hex left of
it is outside the walls, the wall hex and the hexes right of it inside.

**The gate** (+0x53A4): **3 closed** (the setup's value, 0x463C10), **1 open**, 2 between them (the
drawbridge's animation, 0x4671C0 and 0x4672E0 step through it). The gate's hexes are 96 (the gate,
a wall hex) and 95 (the moat before it, the drawbridge's hex); the Fortress's second moat row adds
94. `hex_blocked` (0x469A10) treats 95 and 96 apart from the blocked bit, in every siege: while the
gate is not closed neither blocks; while it is closed both block, except for the side acting as
the defender (+0x132C0 = 1) when no living stack and no corpse lies on 95 nor on 94, **94 tested in
every town** (it is outside the moat but for the Fortress). A stack its controller commands for the
defender that enters 94, 95 or 96 (either hex of a two-hex stack) while the gate is closed opens it
(0x467130 → 0x4671C0: the walk at 0x442186, the flight at 0x4B4A8C, an elemental summoned there at
0x5A71D7). After a walk, a flight, a teleporting move and every action (0x4423F2, 0x4B4A29,
0x4B4EF9, 0x479C65) an open gate closes (0x4672E0) unless a living stack or a corpse lies on 96 or
95, or for the Fortress on 94.

**The moat** (with the moat flag +0x53A8: a Citadel or Castle, not a Tower, not a Stronghold under
RoE or AB rules; §7.27). `is_moat_hex` (0x469DC0): the 11 hexes 0x63BCE8 (11, 28, 44, 61, 77, 95,
111, 129, 146, 164, 181) and for the Fortress (+0x53A9) also the 11 of 0x63BCF4 (10, 27, 43, 60,
76, 94, 110, 128, 145, 163, 180); 95 (94) only while the gate is closed (the lowered drawbridge
covers it). Three effects:
- *A stop* (0x4B3290): every moat hex stops a walker (not a flyer, not the teleporting Devils and
  Arch Devils, 0x4B32BC), except 95 (94) while the gate is not closed or for a stack its controller
  commands for the defender; the stack's own hexes are no stop.
- *Damage on entering* (0x469E50 from `enter_hex` 0x46A570, at most once per move for a two-hex
  stack): only under SoD rules (`mapKind` 2, §7.27), by town type (0x63BD18: 70 Castle,
  Rampart, Necropolis, Stronghold, Conflux; 90 Inferno, Dungeon, Fortress), straight to the hit
  points (0x444090), to either side's stacks, then the Phoenix's rebirth check. A stack re-enters
  its own hexes after Wait, Defend, a shot or an attack from where it stands (0x441F70) and after a
  flight or teleporting move (0x4B4A1E, 0x4B4EEE), so it takes the damage again each time.
- *Defense −3* (0x442590): a struck stack with either hex on a moat hex has 3 defense less, after
  the Behemoths' reduction, not floored at 0; with the moat flag only (a harmless moat too).

**The Tower's land mines** (0x4663BE, a Tower with a Citadel or Castle; the Tower has no moat
flag): a Land Mine (spell 11) object on each moat hex but 95 (ten), owned by the defender, hidden
(+0xA 0), lasting the battle, its damage: under SoD rules 150 (0x63BD20), or the defending hero's
Land Mine damage if that is larger; under older rules the defending hero's, or without a hero the
spell's at power 3 and no mastery. The hero's damage is the spell's base for the hero's mastery
(0x4E5080 with the battlefield, so Magic Plains raise it) plus its power times the spell's factor
(0x5A7890, with the hero's power at +0x53D4 per side, 1 to 99), without the hero's damage
bonuses (0x5A78E0 gets no hero). The mines are the Land Mine
spell's objects from then on (§7.15: they go off for the other side's stacks and are removed).

**The shot's wall test** (0x467510, from the damage's stage 2 at 0x443C79 with the turned shooter's
hex, and the cursor): a shot is halved when the shooter's hex is outside the walls on its row
(left of its wall hex) and the target's hex is inside on its row, unless the shooter is a Mage,
Arch Mage, Enchanter, Sharpshooter or Arrow Tower (creatures 34, 35, 136, 137, 149) or its
controller's hero wears the Golden Bow (91) or the Bow of the Sharpshooter (137), and the line
test 0x467840 finds a wall in the way. **The line test** walks a straight line on the column and
row grid (not the hexes' geometry) from the shooter's (column, row) to the target's in 17 × the
larger of the two differences steps of 1/17 of a cell along the longer axis (single-precision
steps, the position truncated to a hex each step); the line is blocked by the gate's hex 96 while
the gate is closed, and by any wall hex that still has its blocked bit. The halving multiplies
with the distance's (§7.10 stage 2): a shot from beyond the full-damage distance through a wall
does a quarter. 0x467840 also decides Teleport below Expert in sieges (0x5A3764, T8).

**Harvested** (E04c T5d, `h3_formats::exe::siege_data`): the tables 0x63BD00, 0x63BCE8, 0x63BCF4
and 0x63BD18, and six one-byte operands, each read after its instruction's opcode bytes are
checked: the gate's hex (`cmp eax, 0x60` at 0x469A20), the moat's defense penalty (`sub ebx, 3`
at 0x442646), the mines' power (`push 3` at 0x466460), the second row's town type (`cmp al, 7`
at 0x463BAE), the mines' town type (`cmp byte [eax+4], 2` at 0x4663C4) and spell (`push 0xB` at
0x466462). The harvester writes the siege's field into `[battle]` and a `[[siege]]` record per
town type.

**Observed** (E04c T5e, batch 3c, 2026-10-14; `experiments/results/E04c-batch-3c.md`): every point
above that the batch staged behaves as read: the moat's stop (a walk routes around a moat hex),
its damage on entering, landing and defending in place, its −3 defense below 0 (D-10), the gate's
passage, opening and closing, the shot's halving through the closed gate and not through the open
one (D-8), the Tower's ten hidden mines of 150. Two timings seen in memory: the gate opens (+0x53A4
3 → 2 → 1) while the walker still stands before the gate's hex, as it begins the step onto it; and
at a round's start the stacks' turn flags (+0x4F0) are cleared, then the keep shoots, then the
Catapult (the keep's flag stayed set throughout, and so did the Catapult's in G's town with a Fort only,
though it shot every round: not read).

**Review re-read (E04c T10b, 2026-10-15): the gate's hexes as the records hold them.** `hex_blocked`
(0x469A10) reads the records of 95 and 94 as they are (their stack byte and corpse count), with no
exception for the stack that moves: a defender's stack standing on 94 bars itself from 95 and 96
and must go round through the moat (the review found no call that takes the mover off its hexes
before the pathfinder's test; not re-read here). The closing 0x4672E0 reads the same records, and
the death routine 0x444120 appends no corpse for a summoned stack: one killed on 95 holds nothing
from its death on, so the gate closes at that action's end (0x479C65). The engine had counted a
dead created stack as a corpse until the command's end marked it none, which kept the gate open
through the killer's good-morale turn; it now counts it nowhere (BAT-SIE-003). Not read: whether
the dispatcher reaches 0x479C65 before a good-morale turn (the engine closes the gate at every
turn's end, before the morale roll).

**Review re-read (E04c T10b, 2026-10-15): a frenzied striker in the moat.** For a frenzied
striker `stack_attack_against` (0x442410) asks 0x442590 for the striker's own defense (0x442514,
the other stack as the argument, the struck flag 0), scales it by the frenzy's factor and adds it
to the attack. With the flag 0 only the "frenzied when struck: 0" branch is skipped: the Behemoths'
share and the moat's 3 (0x442646, either hex of the striker on a moat hex) apply, without a floor.
So a frenzied stack in the moat adds 3 × the factor less; with a defense below 3 the sum lowers its
attack. VCMI agrees on the 3 (its moat's defence bonus feeds the frenzy's share) but floors the
attack at 0. The engine had added the full defense (BAT-SIE-006, BAT-MAG-019). Not observed.

### 7.29 Code reading: the Catapult, Ballistics and Earthquake (E04c T6, 2026-10-14)

Read for E04c T6 (the survey `docs/rules/surveys/E04c-sieges.md`, D-11 to D-15; BAT-Q43): the
Catapult's stack and turn, its automatic aim, a wall shot (the Catapult's and the Cyclops'), the
parts' destruction, and Earthquake. Insight only, nothing copied (Q29). Not read here: the
turrets' shots and what a destroyed turret's stack does beyond its dead bit (T7), Teleport's wall
test (T8), the computer player's choices beyond the Catapult's aim.

**The targets** (0x63BE60, eight records of 12 bytes: the aim hex, an index into the wall hexes
0x63BD00 or −1, a screen position, the wall part): **0** the Upper Tower (aim 255, part 5), **1**
the Upper Wall (aim 29, part 6, wall hex 29), **2** the Mid-Upper Wall (aim 62, part 8, wall hex
**78**: index 4), **3** the Gate (96, part 9), **4** the Mid-Lower Wall (130, part 10, wall hex
130), **5** the Lower Wall (182, part 12, wall hex 182), **6** the Lower Tower (183, part 13), **7**
the Keep (254, part 14). The aim hex is only where the cursor takes the target; the destruction
opens the wall hex. The other wall hexes (12, 45, 62, 112, 147, 165) belong to no target and stay
blocked. A
target is **standing** (0x476440) while its part's hit points (+0x13F60) are above 0; the towers
(0 and 6) only with a Castle, the keep (7) only with a Citadel or Castle. The hit points come from
`WALLS.TXT` (2 for each of the eight parts in every town) at the siege's start, a Castle's four
walls +1 (0x466348; §7.28), whatever the fort level: at a Fort the keep's and towers' parts hold
2 as well. +0x13FA8 is the parts' picture state (copied before the Castle's +1, then 1 after a hit
and 0 once destroyed), not their hit points.

**The Catapult's stack** (0x463600): creature 145 (`CRTRAITS.TXT`: two hexes, shooter, catapult,
war machine; 1 000 hit points, attack 10, defense 10, damage 0, speed 0, 24 shots) at position 119
(its other hex 120) for an attacker with a hero in a battle with a fort level above 0; the hero's
Catapult artifact is not tested (every hero has one). It never shoots at stacks (the cursor sends
it to the wall test, 0x4766B2). In the pick (0x464F50) it comes after the arrow towers and before
every other stack.

**Its turn** (0x473C00, from the battle loop 0x4740D0): with no standing target (or no fort) the
action is 12 (the turn passes, 0x479BC2). Otherwise the hero's Ballistics (+0xD3) decides: a
player's hero with Ballistics aims the Catapult itself (0x474BF0 false: the cursor 0x476490 takes
the aim hexes of the standing targets, the towers' and keep's off-field ones included); without
Ballistics, or for the computer player or a hidden battle, the aim is automatic:
- with Ballistics (the computer's or a hidden battle only) the gate while it stands;
- the weakest standing wall of 0x63D54C (targets 1, 2, 4, 5: the fewest hit points, ties drawn
  with `random(1, n)` in that order), without Ballistics whenever one stands, with it only while
  all four stand;
- else the first standing of 0x670198 (3 the gate, 7 the keep, 0 and 6 the towers);
- else the first standing target in index order.
The action is 9 with the target's aim hex.

**A wall shot** (action 9, 0x479AE9 → 0x445D30, then the acted bit and the good-morale roll
0x464920). The shooter's level: the Cyclops (94) 1, the Cyclops King (95) 2, the Catapult (145) its
controlling hero's Ballistics level. The record of `BALLIST.TXT` (loaded at [0x679C84], 8 bytes:
the hit chances on keep, tower, gate and wall, the shots, the chances of 0, 1 and 2 damage; none
5/10/25/50 %, 1 shot, 10/60/30 %; Basic 7/15/30/60 %, 1, 0/50/50 %; Advanced as Basic with 2
shots; Expert 10/20/40/75 %, 2, 0/0/100 %). Per shot (0x445EC0, at the aimed target):
1. the damage: `random(1, 100)` less the 0-chance ≤ 0 gives 0, less the 1-chance ≤ 0 gives 1,
   else 2;
2. the hit: `random(1, 100)` above the aimed target's chance by kind (the switch 0x445EFF: 0 and 6
   tower, 3 gate, 7 keep, the walls wall) misses: the shot lands on the **nearest other standing
   wall** of 0x63B840 (1, 2, 5, 4) by the difference of the target indices, a tie replaced with
   `random(1, 100)` ≤ 50; with no other wall standing it lands on the aimed target;
3. 0x445FD0 animates (WallMiss for 0 damage, else WallHit) and calls 0x465990 with the target
   landed on and the damage.
After a shot, while shots are left: if the aimed target no longer stands, the aim moves to the
nearest standing target of all eight (0x63B820 in index order, ties as above); with none the
action ends. The shot uses no ammunition, and the Cyclops' cursor offers it on an empty aim hex
of a standing target (0x47684B) whenever the creature has the catapult flag, the battle a fort
and the acting side is the attacker (+0x132C0 0), whether the stack could shoot or not.

**A part's damage** (0x465990, target and damage; nothing for damage 0): the part's hit points
less the damage, at least 0. When they reach 0: a wall (1, 2, 4, 5) clears the blocked bit of its
wall hex; the gate sets +0x53A4 to **0** (destroyed: not 3, so 95 and 96 never block, 95 is no
moat hex, the line test passes 96, and neither 0x467130 opens nor 0x4672E0 closes it again); a
tower or the keep sets the dead bit 0x200000 on the stack its turret record names (+0x13D98,
+0x13DBC, +0x13DE0) and zeroes its cover's part. The picture state +0x13FA8 becomes 1 or 0.

**Earthquake** (0x5A7C80, from `cast_spell` 0x5A0980 with the mastery). The book allows it only in
a battle with a fort level above 0 (0x59ED29: else a message), for either side, cast at once
without a target and whatever stands. Its hits are the spell's value for the mastery
(`SPTRAITS.TXT`: 2, 2, 3, 4). For each hit: `n` = the targets whose hit points exceed the hits
already drawn on them; none ends the draw; `random(1, n)` picks the n-th target, in index order,
whose hit points are above 0 (**not** less the hits drawn: a target already doomed can be drawn
again, and the last ones in the order can be spared). No fort test: at a Fort or Citadel the
absent towers and keep (2 hit points from `WALLS.TXT`) are drawn too. Then each drawn target takes
its hits as damage through 0x465990 (1 per hit, no roll). **S-08:** an absent turret destroyed so
marks dead the stack its record names, an index among the defender's stacks: without a turret set
up there (the record's index not read, presumably 0) the defender's first stack, which disappears
and stops acting (the wiki's bug, fixed in HotA). The engine keeps the draw and the soaked hits
and harms no stack (exploit-fix, BAT-Q43).

**Observed** (batch 3d, `experiments/results/E04c-batch-3d.md`, run by an agent under Wine,
2026-10-14): the Catapult at 119; every part at 2 hit points, the absent turrets' too; without
Ballistics every landing was the weakest standing wall or its miss list's nearest, the gate only
after the four walls, the keep after the gate (D-13); under Advanced Ballistics the game waited
for the aim, misses landed on the nearest other wall and a second shot whose aim had fallen went to
the nearest standing target (D-12); a destroyed mid-upper wall opened 78, not 62 (D-11); a
destroyed gate stayed 0 and let a walker pass 95 without a stop or damage; Earthquake cast by both
sides (D-15) drew the absent towers, wasted a hit on a doomed part (D-14), and the absent towers'
destruction marked the defender's first stack dead (S-08).

**Review re-read (E04c T10b, 2026-10-15): the side after the Catapult's turn.** The pick 0x465080
first looks at the record activated last (+0x132B8, +0x132BC; §7.21): when it is the attacker's
and its creature is 145, the Catapult, it writes the defender's record 0 there (0x46508B to
0x4650C5) before it compares the candidates. The comparison then prefers at equal speeds the
stack whose side is not the defender's (0x46502A), so after the Catapult's turn, automatic,
commanded or passed (action 12 follows an activation too), the attacker's stack wins a tie. The
engine counts the defender as the side that acted last after the Catapult's turn (BAT-SIE-011,
BAT-TRN-001). Not observed. Action 12 (the dispatcher's case 0x479BC2) only sets the acted bit:
a passed turn gives no Defend bonus. The engine offers the commanded Catapult no Wait and no
Defend (BAT-Q50: batch 3d B saw Defend do nothing, Wait was not tried), and a hero's Earthquake
that brings down the last standing target during the commanded turn passes the turn at once, as
the Catapult's turn 0x473C00 (called by the battle loop at 0x4743A3) does without a standing
target; whether the loop calls it again after a cast was not read.

**Review re-read (E04c T10b, 2026-10-15): a living stack on the gate's hex.** Before the wall test
the cursor 0x476490 reads the hovered hex's record (+0x18, the living stack on it): it goes on to
the empty-hex path with the wall test (0x47684B) only when no living stack stands there or the
acting creature is 145, the Catapult (0x4766AA to 0x4766B9); any other creature's click on a hex
with a living stack goes to that stack. Of the targets' aim hexes only the gate's (96) can hold a
stack (the walls' are wall hexes, the towers' and the keep's lie off the field), so while a
living stack stands in the open gate the Cyclops has no shot at the gate. The engine neither
lists nor accepts it then (BAT-SIE-013); a corpse there does not count. Not observed.

### 7.30 Code reading: the arrow towers (E04c T7, 2026-10-14)

Read for E04c T7 (the survey `docs/rules/surveys/E04c-sieges.md`, D-16 to D-19; quirk candidates
S-06 and S-07; BAT-Q44): the turrets' stacks, turn and control, their shot, and what reaches
them. Insight only, nothing copied (Q29). Not read here: the automatic turret's target (the
computer player's battle AI, reached from the battle loop 0x4740D0 after 0x474BF0 says "on its
own"; as the automatic Ballista's), the battle buttons a commanded turret offers (Wait, Defend).

**The stacks** (§7.27: 0x463600). With a Citadel or Castle the defender's records after its war
machines hold the **keep** (position 254, count `4 + structures` from 0x5BF4F0), then with a
Castle the **upper tower** (255) and the **lower tower** (251), each with half the keep's count
rounded up; with or without a defending hero. The turret records +0x13D78 (0x24 bytes each: the
keep, the lower tower, the upper tower) name each turret's stack by its index at +0x20 (+0x13D98,
+0x13DBC, +0x13DE0). The positions are off the field (0–186). Creature 149's flags `0x20444`:
shooter, war machine, mind-immune, no morale.

**The turn** (0x464F50, the pick's comparison). After a pending good-morale turn, an arrow tower
goes before every other stack, then the Catapult, then by speed; two turrets (one side, one speed)
by the lower record: the keep, the upper tower, the lower tower. The same holds in the wait phase.
After a turret's action the dispatcher rolls no good morale (0x4792A3 skips 0x464920 for creature
149), and its flags give it no bad morale either.

**Control** (0x474BF0): creature 149 takes the Ballista's case (the switch's table at 0x474D64):
a side whose stack's controlling hero (0x442690) has Artillery (+0xDD) and whose player is human
commands it; without a hero or Artillery, for a computer player and in a hidden battle the turret
acts on its own.

**Can it shoot** (0x4428F0): the Ballista and an arrow tower always (no shots, no blocking
neighbour, no Forgetfulness test).

**The shot** (the strike action 0x445A60 → 0x440160, which sends creature 149 to **0x465AD0**;
then the strike action's Phoenix check as after any shot, §7.21). The turret index comes from the
active stack's position (251 → 1, 254 → 0, 255 → 2); after the arrow's animation (0x469880) the
damage (0x465D23 to 0x465D7F): for each creature of the turret's stack (+0x4C) one
`random(2, 4)` (constants of the code, not the creature's damage), summed; the sum as a double
times the target's **protection factor** 0x443D90 with its shot flag 1, the same factor as
physical damage's stage 3 (§7.10, 0x443FF5 to 0x44406D): Air Shield's stored factor (+0x4BC,
`v / 100` as `f32`) while it lasts, × 0.5 while petrified (spell 70), × `(1 − Armorer)` of the
target's controller's hero (0x4E4310); truncated (`_ftol`), at least 1; then 0x444090 takes it
off the target. Nothing else: no attack or defense, luck, Bless or Curse, no hero's Archery or
Artillery, no wall or range penalty, no shot used, no retaliation or Fire Shield, one shot per
activation (0x440160 returns at once, before the Ballista's second shot). So Armorer, Air Shield
and petrification **lower** a turret's damage as any shot's, and Archery does not touch it: the
code contradicts the wiki's S-06 (they raise it) and agrees with its Arrow tower page on S-07
(the Archery page: Archery lowers it).

**What reaches a turret.** Creature 149 is immune to every spell (0x44A4D0 at 0x44A577, before
the per-spell cases; only Dispel's own branch comes earlier), so the mass handler 0x5A66D0 and
Armageddon's 0x5A4BC0, which walk every record, pass over it; its position is off the field, so
no strike, shot, area or breath reaches it; a berserk stack never picks it (0x4455D1); a Magic
Mirror never redirects to it (§7.8). It counts among its side's records (the stack limit, §7.7).
**A destroyed turret** (§7.29, 0x465990): its part at 0 hit points sets the dead bit 0x200000 on
its stack, which then no longer acts.

**VCMI** (1.7.5): the turrets are units at the special hexes of the keep and the towers, never a
valid target (`CUnitState::isValidTarget`); first in the round's siege phase, then the Catapult
(`CMP_stack`, phase 0, by creature index); the keep's damage range `(10 + 2n)..(20 + 4n)`, a
tower's `(6 + n)..(12 + 2n)` with `n` the town level, **one uniform draw** for the range,
through the full damage formula with attack 0, the defense ignored, no luck, no wall or range
penalty (`config/creatures/special.json`, `DamageCalculator::getBaseDamageSingle`); so the
target's Air Shield, Armorer and petrification lower it. Artillery gives control of the towers
(`config/skills.json`: `MANUAL_CONTROL` for `creature.arrowTower`); without it the automatic
shot as the Ballista's (`tryMakeAutomaticActionOfRangedUnit`).

**Observed** (batch 3e, `experiments/results/E04c-batch-3e.md`, run by an agent under Wine,
2026-10-14; Castle towns with a Fort and a Citadel, with a Castle in K: N = 5 in every one, one
structure of the writer's town counted, and 3 creatures in a tower): every round the keep, the upper and the
lower tower shot first, then the Catapult; the keep's shots within 2N..4N (10..20), the towers'
within 6..12; under Expert Air Shield the keep's shots 9, 7, 7, 7, 8 (the code's N..2N; the
wiki's raised damage would be 20 or more); under the attacker's Expert Armorer ten shots of 10 to
14, mean 12.3 (the code's 0.85; 10 is below any raised value); with the defender's Expert Archery
ten shots of 10 to 17, mean 15.2 (unchanged); a defender with Expert Artillery commanded the
keep, one shot a turn, a Wait accepted (the keep acted again after every other stack); a keep the
Catapult destroyed shot no more; the counts and the 99 shots never changed. **The automatic
keep's target** (D-19): of ten Marksmen and one, the keep shot the single Marksman first (the
wiki's computer rule; the engine's, VCMI's, picks the ten), then the ten.


### 7.31 Code reading: the automatic turret's target (E04c T7d, 2026-10-14)

Read for E04c T7d (BAT-Q45 (b), BAT-Q46; the survey `docs/rules/surveys/E04c-sieges.md`, D-19):
the target an arrow tower picks on its own. Insight only, nothing copied (Q29). Not read here:
the computer's general shooter (0x41F060 → 0x41EB80 → 0x435CB0 → 0x435B90), which the automatic
Ballista and every computer-controlled shooter use (below, for T7e).

**The turret's own routine.** When an arrow tower's turn comes and it acts on its own (0x474381:
0x474BF0 says so, or the battle is hidden), the battle loop calls 0x465F20, not the computer's
shooter. 0x465F20 takes the keep's count from the town (0x5BF4F0: 4 + structures), halves it
rounded up for a tower (any position but 254), and calls **0x41E190** with the side 0 (the
attacker's records) and the damage `3 × count`, the mean of the turret's per-creature
`random(2, 4)` (§7.30; not the wiki's "tower damage counted as 1"). A record index ≥ 0 becomes a
shot at that stack's position (action 7); −1 an action 0xC (nothing to shoot).

**0x41E190, the score.** It first builds the computer's battle context (0x435EC0, below) for the
turret's side and the attacker's threat records (0x422B20, below). Then, for every attacker record
whose dead bit `0x200000` is clear (any controller, war machines included):

- `D = trunc(3n × p)` with `p` the target's protection factor for a shot (0x443D90: Air Shield's
  stored `f32`, × 0.5 while petrified, × (1 − Armorer) of its controller's hero), each product
  rounded to the double; no "at least 1".
- the worth `v` of one of its creatures (0x442A50 with the context's two minima and the shot flag
  1; below); then 0x442FD0: for a clone (`0x800000`) `W = trunc(count × v / 5.0)`; else
  `E = D + l` if `D mod h + l ≥ h`, else `E = D` (`h` its hit points per creature +0xC0, `l` its
  top creature's lost hit points +0x58), and `W = trunc(E × v' / h)`, where `v' = 1000.0` in the
  context's desperate mode and `v` otherwise.
- the divisor: 5 if the stack is blind (+0x290), petrified (+0x2B0) or paralysed (+0x2C0), a
  First Aid Tent or an Ammo Cart (147, 148), or has no threat record (+0x538 = 0); else its turns
  `t` (0x448BD0: 1 for the Ballista, an arrow tower and a stack that can shoot now; else
  `ceil(+0x540 / s)` with `s` its speed now (0x448CD0, slowed included), 1 for speed 0), and 5 if
  `t > 5`. The score is `W / divisor` (integer division).
- the best score wins, starting from 0 with no stack; a score equal to the best replaces it
  (`>=`), so **a tie goes to the later record**. In desperate mode, if no stack scored above 0,
  the loop runs again with `v' = v`, keeping the best so far.

Batch 3e's AI encounter follows: the keep (5 creatures, `D = 15`) scores the ten Marksmen and the
single one alike (the same `E`, worth and hit points, both shooters: divisor 1), and the single
one is the later record.

**The worth of a creature** (0x442A50, the computer's estimate; called with the stack, the
context's minima `a₀`, `d₀`, a shot flag and 0): `o = A − a − a₀` with `A` the stack's attack
against no target with the shot flag (0x442410: +0xC8, Precision for a shot or Bloodlust for a
strike, no Slayer, Frenzy's share of its own defense) and `a` its creature's base attack
(table +0x54); `δ = Δ − b − d₀` with `Δ` its defense against no striker for a shot (0x442590 with
flag 1: +0xCC, 0 while frenzied, the moat's −3) and `b` its base defense (+0x58). The defense
factor `f_d = (1 + 0.05 δ) × p` (`p` as above for a shot). The shot flag stays only for the
Ballista and an arrow tower, or a shooter with shots that is not blocked (or whose controller's
hero wears the Bow of the Sharpshooter, artifact 137) and not forgetful at mastery 2 or more; the
offense factor `f_o = 1 + 0.05 o`, halved for a shooter whose shot flag fell; for the Ballista
with a hero, doubled when the hero's Artillery is above Basic and then times 1.0, 1.5, 1.5 or 2.0
by its level (0x63B7F0); while blessed times `(max + bonus) / ((min + max) / 2)`, while cursed
times `max(1, min − penalty) / ((min + max) / 2)` (the stack's damage +0xD0, +0xD4 and the
spells' values +0x458, +0x45C); doubled for a creature that shoots twice (flag `0x8000`) with the
shot flag. Then `v = sqrt(f_o × f_d) × F` with `F` the creature's Fight Value (table +0x3C), each
step rounded to the double. For a war machine (`0x40`) or a summoned stack (`0x400000`, clones
included) `v = v × O / (S + O)`, with `S` its own total hit points (1 for a clone) and `O` the
total hit points of its side's other records that are neither war machines, dead nor summoned;
0.1 if `S = 0`.

**The context** (0x435EC0, side = the turret's): `a₀` and `d₀` are the least attack and defense
bonus (0x442550: attack against no target, with the shot flag if the stack can shoot now, minus
the base attack; 0x442660: defense against no striker with flag 1 minus the base defense) over the
records of both sides with creatures, arrow towers left out. The four strengths (0x41EAC0) sum,
over a side's living records but arrow towers, `trunc(T × v / h)` (a clone: `trunc(count × v /
5.0)`; 0x442E60, `T` its total hit points, `v` its worth with its own shot flag); flag 1 counts
every such record, flag 0 leaves out the blind, petrified and paralysed ones and the tent and the
cart. **Desperate mode** (+8): the side's strength (flag 1) doubled is below the other side's, and
the game's difficulty (`[game]+0x1F6D8`) is above Easy or the side's player is human (+0x54A4).
The context's other fields (+9, +0x14, +0x18, +0x1C) are not used here.

**The threat records** (0x422B20 with the attacker's side, no stack left out, flag 0): each
attacker record is cleared (+0x538, +0x53C, +0x540, +0x544), then skipped if dead, a war machine
other than the Ballista, petrified or blind for more than one round, hypnotized or berserk. A stack
that can shoot now records every defender record but the arrow towers at distance 1. Any other
runs the computer's pathfinder (0x4B2DA0 → 0x4B3400) with range 127 and its speed: a walker steps
between free positions, entering a stop (the moat, a pit its side knows, §7.16) costs the rest of
the turn's move (`s − c mod s`) and it may go on from there; a flyer or a Devil (0x4B39C4) may
cross occupied hexes. An enemy next to a reached position gets its cost (one more turn, `+ s`, from
a stop): the pathfinder marks it (+0x4A of its hexes, cleared at the start) and stores the cost
in its cell (+0x18). Every defender record but the arrow towers whose position hex is so marked,
or (a dead one's) is a free position within the stack's speed, is recorded (0x448840): the one
with the fewest turns `ceil(cost / s)` (then the higher value, 0x435C70) gives +0x538 (the enemy)
and +0x540 (its cost). A stack of speed 0 records no enemy at a distance above 0.

**VCMI** (1.7.5) has no turret routine: the towers shoot as the automatic Ballista
(`tryMakeAutomaticActionOfRangedUnit`: no war machine first, then a stack that can attack next
turn, then the highest expected damage value; BAT-WAR-002). **The wiki** (Arrow tower) describes
the computer's choice by classes A to C (attack this turn, next turn, later) weighted 100 %, 50 %,
33 %, each stack's priority from its AI value and health, ties to the rightmost slot: the code's
turns divisor (1 to 5) and tie agree; its worth is the Fight Value estimate above, not the AI
value; and in this executable the damage is `3n`, not 1.

**The general shooter, for T7e** (not built): 0x41F060 builds the same context and threat records
for the other side, then 0x41EB80 takes, over the enemy's living records but arrow towers, the
highest 0x435CB0 value (an area shooter's own estimate for Magogs, Liches and Power Liches),
preferring a stack that is not blind, petrified or paralysed, ties to the later record.
0x435CB0 is the expected damage value of 0x435B90 (the shooter's own damage estimate, then
0x442FD0's kills times worth) divided as above, but by 10 for a blind, petrified or paralysed
target, and not at all on Easy for a computer side.

**Review re-read (E04c T10b, 2026-10-15): the stop a step enters.** 0x4B3400 tests a step's stop
at 0x4B3701–0x4B3725: for a one-hex stack the new hex; for a two-hex stack the new position's hex
unless it is the current other hex, and the new other hex unless it is the current position's
hex. So a stop is entered only by a hex the stack does not cover before the step: a two-hex
walker whose front entered the moat and steps on, its rear now on that moat hex, pays nothing
more (BAT-SIE-022). Its starting hexes are no stop at all (0x4B3290 clears them).

### 7.32 Code reading: the siege's other rules (E04c T8, 2026-10-15)

Read for E04c T8 (the survey `docs/rules/surveys/E04c-sieges.md`, §2's rows "Surrender, retreat",
"Experience", "Teleport, summoning, spell cost in sieges" and "Defender's bonuses"; BAT-STUB-2).
Insight only, nothing copied (Q29).

**The controls** (0x4782D0, re-read from 0x478573 to 0x478635). For the acting side (+0x132C0),
the surrender button (control 2001) is enabled when the acting side's hero (+0x53CC + 4 × side)
and the other side's are both there and the battle is not at a town (+0x53C8) with the acting
side the defender; the retreat button (2002) when the acting side's hero is there and the
acting side is the attacker, or the battle is at no town, or the town is a Stronghold (type 6,
+0x4) with building 17 (the Escape Tunnel) in its second set +0x158 (the bit at 0x66CE20). So a
town's defender never surrenders, with the Escape Tunnel neither; it retreats only through the
tunnel; the attacker of a town keeps both. `vcmi` (1.7.5, `battleCanFlee`,
`battleCanSurrender`): the same; the `wiki` (Combat: "a hero defending a town cannot surrender
(or flee)"; Escape Tunnel: "retreat (but not surrender)").

**Morale and luck** (0x469750, re-read whole; 0x44AE60, 0x44B2D0). At the battle's start
(0x462E19) and after every action (0x479D16) 0x469750 recomputes, for every record in use of both
sides, the stack's luck (0x43DF20) and morale (0x43E000) from its **controller** (+0xF4, the other
side while +0x288 is set: hypnotized): the controller's hero and army, the other side's hero and
army, the special field +0x53C0, and the town only when the controller is the defender. The base
morale (0x44AE60) adds the town's terms last, after the hero's, the alignments' and the undead's
and before its optional clamp: +1 with a Tavern (building 5 in the set +0x150), +2 in a Castle
(type 0) with building 22 (the Brotherhood of the Sword, in +0x158). **The two do not add up:**
+0x150 holds the buildings with an upgrade's predecessor left out (the Citadel without the Fort,
the Brotherhood, the Tavern's upgrade, without the Tavern), +0x158 every built one (batch 3g CM,
read back from the town record: +0x150 the Citadel, the Village Hall, the Brotherhood; +0x158 also
the Fort and the Tavern; the defenders' morale −1 + 2). So a Castle with the Brotherhood gives +2,
any other town with a Tavern +1 (`vcmi`: the Brotherhood `upgrades` the Tavern with
`upgradeReplacesBonuses`; the `wiki`'s "+2 (in addition to Tavern)" is refuted; survey D-21). The base luck
(0x44B2D0) is 0 when either hero wears the Hourglass of the Evil Hour (85); else the controller's
hero's luck, −1 if the other army holds Devils or Arch Devils (54, 55), then +2 in a Rampart
(type 1) with building 21 (the Fountain of Fortune, +0x158). The town context derives these as
`Siege.morale` and `Siege.luck` (ADR-0023 point 2); the battle adds them to the defender's stacks
as side terms. A stack another side controls takes that side's terms, the town's included, from
the next action on: the engine keeps a hypnotized stack's own side's (BAT-MAG-026; the battle
queue's inbox). `vcmi`: the buildings' bonuses (Tavern +1, Brotherhood +2, Fountain +2 luck) to
the town's defender; the `wiki` (Morale, Luck: "when defending against siege").

**Experience** (0x46A350, re-read from 0x46A3D1 to 0x46A41C). After the kills (the other side's
records that are neither summoned, `0x400000`, nor war machines, `0x40`: start − end counts times
hit points), +500 if the other side has a hero, **−500 if the battle ended in a retreat
(0x6985A3) or a surrender (0x697744)**, **+500 if the battle is at a town (+0x53C8, any fort level)
and the receiving side is the attacker**; then times the hero's Learning (0x4E4840). The battle
reports facts and leaves experience to the orchestrator (BAT-END-003): it knows the battle was at a
town (the adventure lane's inbox). The `wiki` (Experience): "if an enemy hero is defeated or an
enemy town is captured during a siege, the hero receives 500 extra experience points … not … if
the enemy hero escapes or surrenders".

**Tactics** (0x462A20, 0x462C16 to 0x462D0D). The difference of the heroes' Tactics levels (+0xDC)
goes to +0x13D70; the tactics phase +0x13D68 is on when it is not 0 and the battle's bank flag
+0x53C5 is clear (an argument of the battle's start, 0x463C48; it also skips the obstacles,
0x4662B3, and goes to the war machines' setup, 0x46400D); the rest (from 0x462C82: the game's
settings, the players, the hero's formation bit +0x48 & 2) can only turn it off again. No test of
the town or the fort: **a siege has its tactics phase** (`vcmi`: `layout.tacticsAllowed`, off in
creature banks only; the `wiki`, Tactics: "especially vital when sieging a town").

**Teleport in a siege** (0x5A3700, re-read). After the fit test (§7.15) the mastery is the acting
side's hero's of Teleport on the battlefield (0x4E5080 with +0x53C0), 0 without a hero. With the
moat flag +0x53A8 (a Citadel or a Castle, not a Tower, not a Stronghold under RoE or AB rules,
§7.27), a move **crosses the moat** when 0x467490 differs for the stack's position (+0x38) and the
destination: 0x467490(hex) is whether the hex lies left of its row's moat hex in 0x63BCE8 (so the
moat hex counts as inside, the Fortress's outer row, 0x63BCF4, as outside). Below Advanced
(mastery < 2) a crossing refuses the hex; below Expert (< 3) the line test 0x467840 (§7.28) from
the position to the destination must meet no standing wall and not the closed gate (it passes
without a fort). So Basic and no mastery: neither across the moat nor through the walls; Advanced:
across the moat, not through the walls; Expert: wherever the stack fits. The `wiki` (Teleport):
"cannot teleport over walls or moats" (Basic), "over moats (but not walls)" (Advanced), "no
restrictions" (Expert); `vcmi` the same masteries (`config/spells/other.json`: `isMoatPassable`
from Advanced, `isWallPassable` at Expert), its test (`Teleport::isReceptive`,
`battleHasPenaltyOnLine`) only while a wall stands, only for two hexes on different sides of the
walls, along the shortest hex path (survey D-20).

**Placing a created stack** (0x5A7080, 0x5A6C70, 0x469A10, re-read). The summon's place test
0x446C40 runs for a stack of the acting side (its +0xF4 set to it, 0x5A7109), the clone's for its
original; `hex_blocked` (0x469A10) lets the acting side through the closed gate as the defender
(§7.28: 95 and 96, while no living stack and no corpse lies on 95 nor on 94). So the defender's
summoned elemental and clone may be placed on the gate's or the drawbridge's hex while the gate is
closed. A summoned stack placed by the defender on 94, 95 or 96 opens the gate (0x5A71D7 →
0x467130, 0x4671C0); a clone does not (0x5A6C70 calls neither), nor does Teleport (§7.15).

**Spell obstacles and the walls** (0x5A39C0, 0x5A0354, 0x5A0536, re-read). Fire Wall's and Force
Field's target test and Quicksand's and Land Mine's draws refuse a hex with any of the object bits
`0x3F` (§7.15, §7.16): the wall hexes' blocked bit `0x2` (0x4662E3: the 11 wall hexes, the gate's
96 included) is one. A destroyed wall clears its hex's bit, the destroyed gate does not (§7.29):
no spell obstacle ever lies on a standing wall's hex or on the gate's; the moat's hexes have no
bit.

**The spell cost in a siege** (0x59FFDF, 0x463A7F). The cost's army tests (§7.21) read the armies
the battle's start was given (+0x54C4 per side, set at 0x463A7F from its arguments), as every
battle does: which army defends a town is whoever starts the battle's choice (the town context,
E09); the battle reads the setup's armies (BAT-MAG-054), nothing siege-specific.

### 7.33 Code reading: the automatic Ballista's target (E04c T7e, 2026-10-15)

Read for E04c T7e (the battle queue; §7.31's "general shooter"; BAT-STUB-3): the target the
Ballista picks on its own. Insight only, nothing copied (Q29).

**The path.** When no command arrives for the active stack and the battle is hidden or the stack
is a war machine acting on its own (0x474BF0, §7.29), the battle loop (0x4744C0) calls the
computer's action 0x477EE0. For a human side it first asks the computer's retreat test 0x41E570
(the result only matters in auto-combat, +0x1402F); then 0x4221F0 takes the active stack: one that
can shoot now (0x4428F0, the Ballista always) or the Ballista (creature 146) goes to the shooter
0x41F060 with the acting side (+0x132C0), outside the tactics phase (+0x13D68). So the automatic
Ballista and every computer-controlled shooter share it; the arrow towers do not (§7.31).

**0x41F060, the shot.** The context (0x435EC0) for the acting side, the threat records (0x422B20)
of the other side, both as for the turret (§7.31); then the target (0x41EB80). A shooter that can
shoot walls (flag `0x20`, the Cyclops) may shoot them instead (0x41EEA0, not read). In the
context's desperate mode (+8), if the best value is 0, the target is chosen again without it. A
target hex gives action 7 there (the target record's position +0x38, the rear hex of a two-hex
stack); none gives action 3 (Defend).

**0x41EB80, the target.** Over the other side's records (+0x24 of the context), skipping the
dead (`0x200000`) and the arrow towers (149), each scores 0x435CB0 (the Magogs, Liches and Power
Liches score an area estimate instead, 0x41EDA0, not read). The first such record is taken; a
later one replaces the best when it is not blind (+0x290), petrified (+0x2B0) or paralysed
(+0x2C0) and the best is, or when both are alike in that and its value is at least the best's.
So the target is the best by (not disabled, value, record order): a disabled stack only when
every candidate is disabled, ties to the later record. No preference for creatures over war
machines (VCMI's first key, BAT-WAR-002), no controller test: a stack of the other side that the
shooter's side hypnotized is a candidate, its own side's is not.

**0x435CB0, a shot's value.** `V` = 0x435B90 (below) with the shot flag. On Easy
(`[game]+0x1F6D8` = 0) for a computer side (+0x54A4 of the context's side 0) the value is `V`.
Otherwise `V / 10` for a blind, petrified or paralysed target; `V / 5` for a dead one, a First
Aid Tent or Ammo Cart (147, 148) or one without a threat record (+0x538 = 0: a war machine but
the Ballista, a hypnotized or berserk stack, one that reaches no enemy within 127); else
`V / t` with `t` its turns (0x448BD0, §7.31: 1 for the Ballista and a stack that can shoot now),
and `V / 5` above 5 turns. Integer divisions, truncated.

**0x435B90, the value.** With the attacker `A` and the target `T` and their hit points left
(0x443080 flag 0: the total hit points, 1 for a clone): the shot flag stays if `A` can shoot now;
0x4359B0 estimates the damage and takes it from `T`'s hit points (at least 0); the value is
0x442FD0 for `T` (§7.31: its kills times the worth of one of its creatures with the shot flag,
1000 per creature in desperate mode, a clone's count × worth / 5) of the damage dealt, less
0x442FD0 for `A` of the hit points `A` lost (a strike's Fire Shield and retaliation; none for a
shot).

**0x4359B0 and 0x442780, the damage estimate.** For a shot: `n` = `A`'s hit points left over its
hit points per creature, rounded up (its count); the damage per creature `d` is
`max + bonus` while blessed (+0x23C; +0xD4, +0x458), `max(1, min − penalty)` while cursed
(+0x240; +0xD0, +0x45C), else `(min + max) / 2.0`, a double, from the record's +0xD0 and +0xD4. For
the Ballista these hold its creature's 2 and 3: the hero's multiplier (its attack + 1, BAT-WAR-005)
applies only in a strike's base damage (0x443160 from 0x4431AC: creature 146, the controlling
side's hero's attack +0x476 clamped to 0..99, plus 1, times +0xD0 and +0xD4), so the estimate of
one Ballista is `b = 2` whatever its hero's attack, which still counts in its attack (10 + it, the
stage 1 below). Batch 3h confirmed it (TU, DN: a whole small stack before a larger one that the
multiplied range would have chosen). The base `b = trunc(n × d)`; then
the physical damage (0x443F40, §7.10) of `b` with the estimate flag: every stage as a real shot
(attack and defense, the elemental pairs and hate, the hero's Archery, the wall and distance
halves, the protections), no draw (0x443840 rolls the death blow and the Ballista's double damage
only without the flag) and no luck (the lucky field +0x70 is 0 outside a strike: the shot and the
strike clear it at their end); at least 1; doubled for a creature that shoots twice (`0x8000`);
at most `T`'s hit points left. A melee estimate would also reflect Fire Shield and add the
retaliation (0x4359B0's second half).

**The Ballista's worth** (0x442A50 from 0x442C0F; BAT-STUB-3). For the Ballista (146) whose
controlling side has a hero, the offense factor is doubled when the hero's Artillery (+0xDD) is
above Basic, then multiplied by 0x63B7F0's factor for its level (1.0, 1.5, 1.5, 2.0 for none to
Expert), each a double: ×1, ×1.5, ×3, ×4. It counts wherever a Ballista's worth does: as a
target here and of the turrets, and in both sides' strengths (the desperate mode). The
Artillery's double-damage chances are 0x63B810 (0, 50, 75, 100 by level, the pack's values).

**VCMI** (1.7.5, `tryMakeAutomaticActionOfRangedUnit`, BAT-WAR-002): creatures before war
machines, then a stack that can attack next turn, then the highest expected damage times the AI
value; ties to the lower id. **The wiki** (Ballista): the Ballista without Artillery fires at a
target the computer chooses (not re-read this session). **Batch 3h** (an agent under Wine, six
encounters, two rounds each): every target as this reading; VCMI's refuted by the tie (TI), the
desperate mode (DE), the value (DN) and an enemy Ballista before creatures (AR).
