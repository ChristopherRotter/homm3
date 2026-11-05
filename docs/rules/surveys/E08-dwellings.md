# E08 dwellings and hiring survey (E08 T88a): creature dwellings, random dwellings, the Refugee Camp, the hire dialog

Date: 2026-11-03. Scope: E08 increment 10 (decisions 231, 232), input for T88b's ADR amendment
drafts, the design note and the cut. Done whole in one session (the user at T88a's start, 2026-11-03:
not the queue's sketch of two parts, not three). Input, not rules (`README.md`): a fact becomes
binding when a task writes it into the catalog with its sources and tests.

**Batch E10** (run by T90) is scored in §16 (T91, 2026-11-03): every row as read; D-52 confirmed
(the original); no `ADV-Q`.

**Sources** (retrieved 2026-11-03):
- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete); every address
  cited has an entry in `docs/exe/map.toml` or `structs.toml`. The reading follows the human
  player's path. **Read whole:** `visit_dwelling` 0x4A18B0, `dwelling_offer_hire` 0x4ABA50,
  `visit_refugee_camp` 0x4A4600, `set_dwelling_owner` 0x4C67B0, `dwelling_owner_count_add`
  0x4B87A0, `dwelling_record_init` 0x4B8860, `dwelling_week_refill` 0x4B8A60, `dwelling_at`
  0x4BB900, `army_can_take` 0x44AC50, `H3RecruitMgr::initialize` 0x551350 (the dwellings'
  overload). **Read in part:** `recruit_dialog_select` 0x5503A0 (the costs and the most a player may
  buy, the buttons' states), `recruit_dialog_proc` 0x550940 (the purchase, the room test, the Max
  and Recruit buttons; not the slider's keys, the war machines' branch read only as far as it
  differs), `load_dwelling_body` 0x4FED40 (the record's fields), `random_dwellings_resolve`
  0x502B60 (the linked town's lookup, re-read here; the rest read by E24 M1 T5b1),
  `setup_map_objects` 0x4C0CC0 at its cases 17 to 20 and 78, `new_day_week_events` 0x4C8780 at
  the dwellings' loop and case 78, `object_popup_text` 0x4137C0 at its cases 17 and 20, `map_hint`
  at its cases (E24 M3 T6a2b's reading), the tables `dwelling_creatures` 0x63D570,
  `dwelling4_creatures` 0x677938 and `town_creatures` 0x6747B4. **Not followed:** the computer
  player (`ai_dwelling_visit` 0x527B00, `ai_refugee_camp_hire` 0x527E10, `ai_fight_answer`),
  network play (`map_message_send` 0x482390), the scenario's test after a flag (0x5F2600,
  `scenario_condition_met`: E10), the towns' external dwellings counts (`town_dwelling_count_add`:
  E09's, read by its T1b1), the Portal of Summoning (E09), the War Machine Factory (class 106, not
  in decision 231), the dialog's drawing.
- `vcmi`: VCMI 1.7.5 (`git clone --depth 1 --branch 1.7.5`): `lib/mapObjects/CGDwelling.cpp`
  (`randomizeFaction`, `randomizeLevel`, `pickRandomObject`, `onHeroVisit`, `newTurn`,
  `getPopupComponents`, `updateGuards`, `heroAcceptsCreatures`, `battleFinished`,
  `blockingDialogAnswered`), `lib/mapObjectConstructors/DwellingInstanceConstructor.cpp`
  (`randomizeObject`: the guards), `lib/mapObjects/CGObjectInstance.cpp` (`setType`: the offset
  kept), `lib/mapping/MapFormatH3M.cpp` (`readDwelling`, `readDwellingRandom`, the victory
  `TAKEDWELLINGS`), `server/CGameHandler.cpp` (`recruitCreatures`, `onNewTurn`),
  `lib/callback/GameRandomizer.cpp` (`rollCreature`), `config/objects/dwellings.json`,
  `config/gameConfig.json` ("dwellings": `accumulateWhenNeutral`, `accumulateWhenOwned`,
  `mergeOnRecruit`); links `https://github.com/vcmi/vcmi/blob/1.7.5/<file>`.
- `wiki`: read through the search tool's extracts (the site is not fetched from the sandbox),
  `https://heroes.thelazy.net/index.php/<Page>`: Creature dwelling, Refugee camp, Growth,
  Portal of summoning. Re-check a quote before a rule cites it with `high` confidence.
- `original-data` (the census): the installation's 160 shipped maps (`original-data/maps.tar`)
  read with `h3-map`'s reader and a throwaway example (not committed), §1; the text tables
  `CrGen1.txt`, `CrGen4.txt`, `GENRLTXT.TXT`, `ADVEVENT.TXT` and `Objects.txt` (`extracted.tar`).
  Text rows are counted from 0 with the file's first record, as `general_text` says (VCMI's
  `GENERAL_TXT` numbers are one less: it skips that record; its `ADVOB_TXT` ones are the same).

## 0. Findings in short

- **The census** (§1): 1 981 dwellings of class 17 on 116 maps (79 of the 80 subclasses; 60, the
  Altar of Thought, never), 80 of class 20 on 38 (the Elemental Conflux 24, the Golem Factory 56),
  none of 18 or 19; 321 Refugee Camps on 101 maps; random dwellings 216 (276 on 22 maps), 217
  (1 326 on 75), 218 (1 on 1). 15 dwellings are owned at load on 5 maps, all by a colour that
  plays; no random dwelling is owned. Of the random ones 1 397 are tied to a random town (77,
  E09's) and **206 are in scope**: 79 of 216 and 126 of 217 with a faction set, the 218. No map
  holds an empty set, a level past 6, a lowest level above the highest, or a town identifier that
  names no town. 5 maps have the victory "flag all creature dwellings" (E10).
- **The record** (§2): each dwelling of 17 and 20 is a 0x5C-byte record (H3API's `H3Generator`):
  four creatures and their counts, the guards' army, the entrance, the owner. Class 17 has one
  creature (`dwelling_creatures`, by subclass), class 20 four (`dwelling4_creatures`: the Conflux's
  Air, Fire, Earth and Water Elementals; the Factory's Stone, Iron, Gold and Diamond Golems). At
  load each count is its creature's **growth** and the guards are, for each creature of level 5 or
  more, a stack of **3 × its growth** (the Golem Factory 9 Gold and 6 Diamond Golems, the Conflux
  12 Earth Elementals, a Dragon Cliffs 3 Green Dragons). Classes 18 and 19 stop the setup with an
  error box; a subclass past the tables reads past them.
- **The random dwellings** (§3): at the map's load, faction from the linked town, else a uniform
  draw over the set (an **empty set gives Castle**, A-34; a **dangling identifier reads before the
  towns' list**, A-35); level uniform between the two bytes; the creature `town_creatures[faction ×
  14 + level]`; the dwelling the last class-17 subclass listing it, or the Golem Factory (20, 1) for
  the Tower's level 3; **the object takes that dwelling's template** (`Objects.txt`'s, one per
  subclass) and the Angel Portal's anchor moves one tile right. **Every outcome's footprint differs
  from the random dwelling's 2 × 2**: two blocked cells on the bottom row, its entrance one of them
  (45 subclasses at the anchor, 13 one tile left); so the entrance tile depends on the draw.
- **The visit** (§4): an own or allied (team) dwelling skips the guards and the flag. Else the
  guards, when any stand, are asked (GENRLTXT 422) and fought **unsplit and unowned** (the
  defending side's owner −1, though the dwelling is a player's: **D-51**, VCMI the owner's side);
  lost: the survivors stay, nothing else; won or unguarded: **flagged** (radius 3 revealed for the
  team), then the hire question (ADVEVENT 35, or 36 for four creatures). Yes: on the visitor's
  own dwelling each **level-1 creature joins free**, all of it (none without room: GENRLTXT 426,
  **D-52**, VCMI merges two stacks first); any creature of level 2 or more opens **the hire
  dialog**; on an ally's dwelling the dialog opens with every button disabled (H3API's
  `teamDwelling`): an ally neither flags nor hires (the wiki and VCMI agree; VCMI shows nothing).
- **The hire dialog** (§5): per creature, the most a player may buy is
  `min(count, gold ÷ gold cost, its other resource ÷ that cost)` (the creature's cost holds gold
  and at most one other resource); Recruit adds the amount to the army (a slot of the type or the
  first empty; none: GENRLTXT 426, nothing bought), takes gold × amount and the resource × amount
  from the treasury, and lowers the dwelling's count; a single-creature dialog closes, a
  four-creature one stays open. The purchase itself checks neither the treasury nor the count: the
  slider's bound does (the engine validates both).
- **The week** (§6): at each new week (never day 1), **every** dwelling's counts are **set** to the
  growth (not added; owned or neutral; the week's and the month's kinds are not applied) and its
  guards **replaced** by 3 × growth of the level-5+ creatures, whoever owns it. VCMI and the wiki
  agree.
- **The Refugee Camp** (§7): no record, no owner, not flaggable, no guards; its creature and count
  live on its entrance tile's word and subclass. At a new game and at each new week its creature
  is drawn as a random monster of levels 1 to 7 is (ADV-MON-003's set: AB and SoD leave out
  132–137) and its count set to that creature's growth. A visit with none left says it is
  deserted (ADVEVENT 44); else asks (ADVEVENT 112) and opens the hire dialog for anyone, with no
  free creatures.
- **A defeat** (§8) unflags the player's dwellings (their guards stay). The flag and the defeat
  also move the towns' external dwellings counts (E09's growth bonus, not built here).
- **What a player sees** (§9): the hint and the right-click popup show a dwelling's name (so its
  creatures) and its owner; never its counts or guards. A Refugee Camp shows its name only; its
  creature is named only by its visit's question. The counts are seen in the dialog; the guards
  in the question (a size word of their total count and the first stack's creature).
- **Disputes and candidates** (§11): **D-51** (the guards' side), **D-52** (no merge for the free
  creatures); **A-34** (an empty faction set gives Castle), **A-35** (a dangling town identifier),
  **A-36** (classes 18 and 19, subclasses past the tables): all three refused at the import, as
  A-31. Batch E10's plan (§13): ten rows in two runs (E10-2 dropped), run by T90 and **scored in
  §16 (T91): every row as read**. The proposed ids (§14): the new families
  ADV-DWL and ADV-HIR, ADV-RFG-001, GAM-DEC-008 and amendments. The checkpoint's questions for
  T88b (§15): ADV-Q42.

## 1. The census

`original-data`, the 160 shipped maps, read with `h3-map` (a throwaway example):

| Class | Objects | Maps | By version (objects / maps) | Notes |
|---|---|---|---|---|
| 17 dwelling | 1 981 | 116 | RoE 695 / 44, AB 654 / 41, SoD 632 / 31 | 79 subclasses (60 never; 59, 61, 68, 70, 71 on 2 maps each); most on one map 99 (`Pandora's Box `) |
| 18, 19 | 0 | 0 | | the original refuses them at setup (§2.4) |
| 20 multi-creature dwelling | 80 | 38 | RoE 14 / 10, AB 30 / 18, SoD 36 / 10 | subclass 0 (`AVGelem0`, the Elemental Conflux) 24 on 15 maps, 1 (`AVGgolm0`, the Golem Factory) 56 on 26; most on one map 14 (`Golems Aplenty Allied`) |
| 78 Refugee Camp | 321 | 101 | RoE 56 / 25, AB 144 / 38, SoD 121 / 38 | subclass 0 only; most on one map 9 (`Battle of the Sexes Allied`) |
| 216 random dwelling | 276 | 22 | AB 66 / 7, SoD 210 / 15 | 197 tied to a random town (167 a player's, 30 neutral); 79 with the set of all nine factions (12 maps) |
| 217 random dwelling by level | 1 326 | 75 | AB 766 / 41, SoD 560 / 34 | subclass = level 0–6 (446, 286, 217, 124, 116, 53, 84); 1 200 tied to a random town (940 a player's, 260 neutral); 126 with a set: 109 all nine, 17 one faction (`War of the Mighty` and its Allies version one per faction 0–7, `The Newcomers` faction 3) |
| 218 random dwelling by faction | 1 | 1 | AB 1 / 1 (`The Gauntlet`) | subclass 3 (Inferno), levels 0–2 |

- **Owners at load:** 14 of class 17 on 5 maps (`Adventures of Jared Haret`, `Dragon Orb`,
  `Myth and Legend`, `Unholy Quest`, `Wings of War`) and 1 of class 20 (`Adventures of Jared
  Haret`, a Golem Factory); every owner a colour that plays. No random dwelling is owned; no
  owner past colour 7.
- **The random dwellings' bodies:** no empty faction set, no level past 6, no lowest level above
  the highest, no town identifier that names no town (A-34, A-35 stay input bounds); every set is
  two bytes (AB, SoD; RoE has no random dwelling) within bits 0–8. 216's ranges: 0–6 (89), 0–4
  (78), 0–3 (53), 0–5 (32), 0–2 (12), 0–1 (8), 4–6 (2), 1–4 and 3–6 (1 each).
- **In scope** (decision 231: not tied to a random town): 206 objects, 216 on 12 maps, 217 on 33,
  218 on 1. Of them 96 (23 maps) may become the Golem Factory (the Tower in the set, level 2 in
  the range) and 76 (15 maps) the Angel Portal (Castle, level 6).
- **Victory "flag all creature dwellings"** (goal 8): 5 maps (`Goblins in the Pantry`, `Hatchet Axe
  and Saw`, `Noahs Ark`, `Too Many Monsters`, `When Dragons Clash`): E10's.
- **Templates** (`Objects.txt`): one template for each of the 80 + 2 subclasses of 17 and 20, so
  "the last template of the subclass" (§3.3) is the only one. The random dwellings' templates
  (`avrcgen*`, `avrcgn0*`) and the Refugee Camp's (`AVGrefg0`) are 2 × 2 with both bottom cells
  visitable (the camp: one, the anchor).

## 2. The dwellings and their records

### 2.1 The record

`original-data` (`structs.toml` "dwelling_record", 0x5C bytes, the vector at game +0x4E39C; H3API
`H3Generator`): +0x0 the class (17 or 20), +0x1 the subclass (the name's row in `CrGen1.txt` or
`CrGen4.txt`), +0x4 four creature ids (−1 none), +0x14 four i16 counts, +0x1C the guards' army
(seven creature ids and seven counts), +0x54 to +0x56 the entrance (column, row, level), +0x57
the owner (−1 none), +0x58 a byte set to −1 at init and not read further. The entrance tile's
word is the record's index (`dwelling_at` 0x4BB900 finds a record by its entrance instead).

### 2.2 At load

`load_dwelling_body` 0x4FED40 (from the object loader): the record takes the template's class and
subclass and the entrance tile (`object_entrance_tile` 0x4FEC30), and `dwelling_record_init`
0x4B8860 (read whole) with the body's owner (its u32 read as a signed byte: 0xFF is −1):

1. the four creatures −1, counts 0, the guards emptied (`army_reset`);
2. class 17: one creature, `dwelling_creatures` 0x63D570[subclass] (80 entries); class 20: four,
   `dwelling4_creatures` 0x677938[subclass] (subclass 0: 112, 114, 113, 115, the Air, Fire, Earth
   and Water Elementals; 1: 32, 33, 116, 117, the Stone, Iron, Gold and Diamond Golems; the table
   holds two rows, a subclass of 2 or more reads the following data);
3. `dwelling_week_refill` (§6.1): the counts and the guards;
4. the owner set and, unless it is −1, the owner's towns' external dwellings counts raised
   (`town_dwelling_count_add`, E09's; under RoE rules creatures 112–115 count for no faction).

`setup_map_objects` (the new game) then calls, for each dwelling's entrance,
`set_dwelling_owner(index, its own owner)`: the same owner, so only the reveal of radius 3 for an
owned one and the scenario's test (§4.4).

**So at the start** a dwelling holds one week's growth of each creature, and its guards are 3 ×
growth of each creature of level 5 or more (zero-based level 4 or more), one stack each in
creature order: a Dragon Cliffs 3 Green Dragons, a Pyre 6 Firebirds (growth 2), the Golem Factory
9 Gold and 6 Diamond Golems, the Elemental Conflux 12 Earth Elementals (level 5), any level-1 to
level-4 dwelling none. `wiki` (Creature dwelling): dwellings of level 5, 6 and 7 are guarded by
"growth times three" of their creature, a Pyre by 6 Firebirds; a dwelling owned at the start is
guarded "for the enemy at the first week". `vcmi`: `randomizeObject` puts 3 × growth of each
creature of level 5 or more, the Conflux's and the Factory's from `dwellings.json` (12 Earth; 9
Gold and 6 Diamond: the same numbers); `newTurn` runs for every object on the first day too, so
the counts start at the growth. Three sources agree.

### 2.3 The creature tables

`dwelling_creatures` 0x63D570 lists five creatures twice: the Unicorn (subclasses 51, 68) and the
four elementals (7, 13, 16, 47 the RoE "Elemental Conflux" dwellings of one element; 69 to 72 the
AB Altars). The levels: zero-based in the creature record (+0x4), so "level 1" below is 0 there.
The neutral dwellings 73 to 79 (Halfling to Troll) and the AB special ones 62 to 67 (the four
dragons, the Enchanter, the Sharpshooter) are in it; 60 (the Psychic Elemental's Altar of Thought)
too, never placed on a shipped map.

### 2.4 Invalid classes and subclasses

Classes 18 and 19 get no record: `load_object_body` sends them to the case of the classes it reads
no body for (where `docs/formats/h3m.md` gives them an owner, so such a map would be misread; that
case was not read further), and `setup_map_objects` cases 18 and 19 show a message box
("CREATURE_GENERATOR_2 found at …", "Invalid Generator") and go on. A class-17 subclass of 80 or more, or a class-20
one of 2 or more, reads past its table (the names' too: `dwelling1_names`, `dwelling4_names`).
None of the shipped maps has one; the map client's MCL-Q21 (b) asks the import to refuse "a
dwelling's kind past its table". **A-36** (§11).

## 3. The random dwellings

### 3.1 The bodies

`docs/formats/h3m.md` and `h3-map`'s reader (`parse/objects.rs` `random_dwelling`): a u32 owner;
216 and 217 a u32 town identifier and, when it is 0, the faction set (one byte in RoE, two from AB
on; the census finds only AB and SoD maps); 216 and 218 the lowest and highest level bytes. 217's
level is its subclass, 218's faction its subclass. `vcmi` `readDwellingRandom`: the same fields;
it clamps the highest level to 6 and keeps levels one-based. They agree.

### 3.2 The resolution (`random_dwellings_resolve` 0x502B60)

At the map's load, after the objects' bodies and before their tiles are placed (E24 design §2.9;
re-read here at the linked town), for each random dwelling of the map's list (16-byte records: +0x0
the identifier, +0x4 the set, +0x7 and +0x8 the levels, +0xC the object):

1. **The faction:** with an identifier, the town whose identifier it is, searched from the end of
   the game's town list (+0x98, 0x88-byte entries) — the town's faction (+0x68), its random town
   already resolved (E09 survey §2.1, §2.3). **When no town has it** the search ends at index −1
   and reads the entry before the list (A-35). Without an identifier, `random_allowed_faction`
   0x5132D0 over the set: uniform over its set bits; **an empty set gives faction 0, Castle**
   (A-34). 218: its subclass (the loader puts it in the set).
2. **The level:** `random_between(lowest, highest)`, uniform (a lowest above the highest divides by
   zero or worse: an input bound). 217: its subclass both.
3. **The creature:** `town_creatures` 0x6747B4[faction × 14 + level] (the unupgraded one).
4. **The dwelling:** a Stone Golem (32, the Tower's level 3) makes class 20 subclass 1 (the Golem
   Factory: all four golems and its guards); any other creature class 17 with **the last**
   subclass `dwelling_creatures` lists it at (so 68 for the Unicorn, 69 to 72 for the elementals).
   Every faction's seven creatures have one (checked over the table: the Tower's level 3 is the
   only one without a class-17 dwelling).
5. **The template:** `object_set_class_template` 0x505F20 gives the object that class's last
   template of the subclass from the game's templates (`Objects.txt`; appended to the map's), and
   the anchor's column grows by the new template's bottom-row cells from the anchor leftwards that
   are passable and not visitable: only the Angel Portal (17, 8) moves, by one (E24 M1 T5b1,
   captured exact by the map client's `owners-random-dwellings`).
6. **The record:** `dwelling_record_init` with the random dwelling's owner (§2.2).

**The footprints** (`Objects.txt`, the 62 outcomes of the 9 factions' 63 creatures): every outcome
blocks two cells of the bottom row (the anchor and the tile left of it) and frees the upper row,
where the random dwelling's template blocks the 2 × 2 and makes both bottom cells visitable: 45
outcomes have their entrance at the anchor, 13 one tile left (the Crusader's, Dwarf's, Elf's,
Gog's, Efreet's, Wight's, Vampire's, Bone Dragon's, Beholder's, Minotaur's, Red Dragon's, Wyvern's
and Hydra's), 3 also block the cell above the left one (Pikeman, Titan, Wolf Rider); the Naga and
the Ogre keep a blocked 2 × 2 with one entrance. The Angel Portal's template is three cells wide
with its entrance one left of its anchor: its move puts that entrance on the random dwelling's
anchor and its third cell on the random dwelling's left one. **So after the draw a player's paths
and the entrance tile depend on the drawn creature.** *Corrected by E08 T93a* (the packs'
check against `Objects.txt` and the executable's tables, `h3-pack-adventure`'s `original_data`):
**63** outcomes, one per creature (a class-17 subclass lists one creature, the Golem Factory's
outcome is the Stone Golem alone); 44 entrances at the anchor besides the Angel Portal's (45 after
its move); the one-left list has the Archer's (`AVGcros0.def`), not the Crusader's (the Barracks'
entrance is its anchor); the three blocking the cell above the left one are the Pikeman's, the
Giant's (the Cloud Temple) and the Wolf Rider's.

`vcmi` (`pickRandomObject`): the same faction (the linked town's, else the set, else any faction
with a town), level and creature; the dwelling is the first one not `bannedForRandomDwelling` that
produces it, and `dwellings.json` bans exactly the duplicates the original's "last" skips (7, 13,
16, 47, 51) and the Elemental Conflux, so both pick the same subclasses (the Golem Factory too,
through `CREATURE_GENERATOR4`); `setType` takes the new type's template for the tile's terrain and
moves the position by the templates' offsets. The faction, level, creature and dwelling agree; the
template's tiles were not compared tile by tile (VCMI's own templates; the map client's captures
are the reference for the original's). An empty set and a dangling identifier differ (VCMI: any
faction) — both refused at the import (A-34, A-35).

### 3.3 What the engine needs for them

The draw is the game's (decision 17's order, as the random monsters', ADV-MON-003): when the game
is built, from the setup stream. Its result needs, as content, **each dwelling kind's footprint**:
the 82 templates' blocked and visitable cells (and the Angel Portal's move), since the map file
holds only the random dwelling's template. The 206 in scope come from 46 maps; the 1 397 tied to a
random town wait for E09's random towns. A random dwelling tied to a fixed town (98) appears on no
shipped map (its faction would be the town's, known at load).

## 4. The visit (`visit_object` case 17 and 20 → `visit_dwelling` 0x4A18B0, read whole)

Arguments: the hero, the entrance tile, the position, human. The dwelling is entered (its
entrance is a visitable cell of its template, as a mine's).

### 4.1 An own or allied dwelling

When the record's owner and the current player are of one team (the team bytes, game +0x1F879):
no guards' question, no flag; on to the hire question (§4.4).

### 4.2 The guards

When the guards' army holds a creature: a human is asked with a two-button message box, GENRLTXT
422 ("Much to your dismay, the %s is guarded by %s %s …"): the dwelling's name, the size word
(`creature_quantity_word`, its plural form) of **the guards' total count**, and the creature of
**the first occupied slot**. No: the visit ends. The computer answers through `ai_fight_answer`.

Yes (or the computer's yes): `adventure_battle` with the hero's army and **the guards' army as it
stands, unsplit, with owner −1 and no hero** (where `visit_mine` passes the mine's owner). The
battlefield is the tile's (no special field). Lost, retreated or surrendered: the visit ends; the
guards' losses stay in the record (their army is the battle's) until the week replaces them (§6.1).
Won: the guards are gone; on to the flag.

`wiki`: "If the attacking hero retreats or is defeated, only the surviving guards will be there if
another hero attacks in the same week." `vcmi`: the same question (its text index 421, our 422),
its size word from the first stack's count (the original's from the total: a text difference);
`startBattle(hero, dwelling)`, the guards unsplit, **the defending side the dwelling's owner**
(D-51); after a win `battleFinished` visits again (the flag, then the question), as the original
goes on.

### 4.3 The flag

The guards gone or none: unless the owner is the visitor's team, `set_dwelling_owner(index,
current player)` (§4.5) — the flag needs no answer and comes before the hire question. `vcmi`:
`setOwner` for an enemy's or a neutral dwelling (relations ENEMIES), not for an ally's or one's
own. `wiki`: an allied player's dwelling is not re-flagged. They agree.

### 4.4 The hire question and the free creatures

A human is asked (two buttons): ADVEVENT 35 ("{%s} Would you like to recruit %s?") with the name
and the creature's plural for class 17, ADVEVENT 36 with the four plurals for class 20. No: the
visit ends (the flag stays). Yes:

- **The visitor's own dwelling** (owner = current player): for each of its creatures **of level 1**
  (zero-based 0): a count of 0 adds GENRLTXT 423 ("There are no %s here to recruit."); else
  `army_add_stack(hero's army, creature, count, −1)` (a slot of the type, else the first empty):
  added, the count set to 0 and GENRLTXT 424 ("%d %s join your army.", singular for 1); no room,
  GENRLTXT 426 ("The %s would join your hero, but there aren't enough provisions …") and the count
  kept. The lines show in one message box. If none of its creatures is of level 2 or more the
  visit ends; else `dwelling_offer_hire` 0x4ABA50.
- **An ally's dwelling** (owner ≠ current player: the visitor's team): straight to
  `dwelling_offer_hire`, with no free creatures.

`dwelling_offer_hire` (read whole): on the visitor's own dwelling it runs the free creatures' loop
again (it would matter only for a dwelling holding creatures of level 1 and of level 2 or more,
which neither class has) and returns when no creature of level 2 or
more is there; then builds the hire dialog (`H3RecruitMgr::initialize` 0x551350: the hero's army,
not the Portal, the four creatures and **pointers to the record's four counts**) and sets its
+0x4C (H3API `teamDwelling`) when the owner is another player: every button of an ally's dialog
is disabled (§5.2). So **an ally's visit shows the offer and buys nothing**.

The computer (`visit_dwelling` 0x4A201F): its own or a neutral record → `ai_dwelling_visit`
0x527B00 (not read, E10); an ally's: nothing.

`wiki` (Creature dwelling): "level 1 creatures, which are recruited for free"; "A hero visiting an
adventure map dwelling (or town) that belongs to an allied player doesn't re-flag it and cannot
hire creatures from it." `vcmi` `heroAcceptsCreatures`: a first creature of level 1 joins free
(only the first creature is tested; class 20 has none of level 1, so the same), the count to 0,
GENERAL_TXT 423 and 422 and 425 (ours 424, 423, 426); with `mergeOnRecruit` (on by default) a full
army first **merges two stacks of one type** to free a slot (D-52); any other opens the
recruitment window. For an ally VCMI returns before any message (no dialog): what a player sees
differs, the rule does not.

### 4.5 `set_dwelling_owner` 0x4C67B0 (read whole)

Arguments: the record's index, the new owner. A message record for the other screens and the
network (not followed). A different owner: the old owner's towns' counts lowered for the first
creature's faction (E09's), the owner +0x57 written, the new owner's raised
(`dwelling_owner_count_add` 0x4B87A0). Then, for an owner other than −1, `reveal_disk` radius 3
around the entrance for the owner's team (as `set_mine_owner`); then the scenario's test
(0x5F2600 on game +0x1F89C, then `scenario_condition_met`: E10). The guards are untouched (they
were fought before). Callers: the visit, the setup (§2.2), `player_defeated` (§8), the computer.

## 5. The hire dialog (`H3RecruitMgr`, vtable 0x640C70)

### 5.1 Its state

`H3RecruitMgr::initialize` 0x551350 (read whole; H3API's overload with an army): the army (+0x98),
the Portal flag (+0x9C), the four types (+0x5C …) with pointers to their counts (+0x6C …; the
dwelling's own i16 counts: a purchase lowers them directly), the selected type (+0x50) and its
count's pointer (+0x54), the selected index (+0x58), the gold cost (+0x84) and the first other
resource the creature costs (+0x88 its index below 6, +0x8C its amount; −1 none): the dialog
handles gold and one other resource, and every creature costs at most one (the creature tables).
The town's recruitment (E09) uses H3API's other overload (with a hero), the Refugee Camp and
the Portal of Summoning this one.

### 5.2 The most a player may buy (`recruit_dialog_select` 0x5503A0, read in part)

On each selection: the costs read again; **the most** (+0xAC, H3API `amountAvailable`) =
`min(gold ÷ gold cost, resource ÷ its cost (if any), the count)` from the current player's
treasury (+0xB4 gold, +0x9C + 4 × index), integer division; the amount to buy (+0xB8) clamped to
it. The Recruit, Max and slider buttons are enabled only when the most is above 0 and +0x4C (the
ally's dialog) is clear; Recruit also needs an amount above 0.

### 5.3 The purchase (`recruit_dialog_proc` 0x550940, read in part)

- **Recruit** (item 30722): an amount of 0 with a single creature closes the dialog. Else (a
  creature, not a war machine) `army_can_take` 0x44AC50 (a slot of the type or an empty one);
  none: GENRLTXT 426 (18 for the Portal), nothing bought, the dialog stays. Room:
  `army_add_stack(army, type, amount, −1)`, then gold −= gold cost × amount, the other resource −=
  its cost × amount, the dwelling's count −= amount, the amount 0. A single creature's dialog then
  closes; with several the selection is refreshed and the dialog stays open for more purchases
  until closed.
- **Max** (item 532): the amount = the most. **Close** (item 30721) ends the dialog.
- The purchase does not test the treasury or the count again: the bound of §5.2 is the only test.
  The engine validates the amount against both (ADR-0004).

`vcmi` `recruitCreatures`: refuses a dwelling not the player's (an unowned Refugee Camp passes as
UNFLAGGABLE) or a hero not visiting it, reduces the amount to the count, refuses more than the
treasury pays for (`maxAmount`), 0 or less, or no slot; takes `getFullRecruitCost() × amount` and
adds to the slot. The same rule. `wiki`: "The cost of creatures is the same in towns and on
adventure map, except level 1 creatures". Three sources agree.

## 6. The week

### 6.1 The dwellings (`dwelling_week_refill` 0x4B8A60, read whole)

`new_day_week_events` (survey `E08-economy.md` §7.3; after the towns' growth, the week's count and
the taverns, before the map's scan) calls it for **every** record, owned or not:

1. the guards' army emptied;
2. each creature's count **set** to its growth (creature record +0x44);
3. for each creature of level 5 or more (zero-based 4), `army_add_stack(guards, creature, 3 ×
   growth, −1)`.

So the counts never accumulate, unbought creatures are lost, the guards are back each week
whoever owns the dwelling (an owner's own visit never meets them, §4.1), and neither the week's
creature nor the month's doubling or Plague reaches an external dwelling (the refill reads no
kind). `wiki` (Creature dwelling, Growth): "In adventure map dwellings the number of creatures
cannot exceed the weekly growth"; "The guards are renewed at the beginning of each week". `vcmi`
`newTurn` on a week's first day: the count = growth (`accumulateWhenNeutral` and
`accumulateWhenOwned` false; its growth bonuses on the creature type are empty in the base game),
then `updateGuards` (3 × growth for level 5 and more). Three sources agree.

### 6.2 The Refugee Camp (the map's scan, `new_day_week_events` case 78)

`random_creature_by_level(0, 6)` → the tile's subclass word (+0x22) = the creature, its word
(+0x0) = the creature's growth; unbought creatures are lost. §7.

## 7. The Refugee Camp (78)

- **At a new game** (`setup_map_objects` case 78): the same draw as the week's (§6.2). No body in
  the map file, no record, no owner.
- **The draw:** `random_creature_by_level` 0x4C92C0 with levels 0 to 6: uniform over the creature
  ids 0–144 of those levels, AB and SoD leaving out 132–137 (the four AB dragons, the Enchanter,
  the Sharpshooter), RoE allowing only 0–117; ids 122, 124, 126 and 128 have level −1 (the random
  monsters' rule, ADV-MON-003; survey `E08-battles.md` §1.4).
- **The visit** (`visit_refugee_camp` 0x4A4600, read whole): a count of 0: a human sees ADVEVENT 44
  ("{%s} … The camp is deserted. Perhaps you should try next week."), nothing else. Else a human
  is asked (ADVEVENT 112, the camp's name and the creature's plural); no: nothing; yes: the hire
  dialog (§5) for the hero's army with the camp's creature and its count (a copy, written back to
  the tile's word when the dialog closes). No flag, no guards, no free level-1 creatures, any
  player's hero. The computer: `ai_refugee_camp_hire` 0x527E10 (E10).
- `wiki` (Refugee camp): random creatures "including neutral creatures", the type changes every
  week, "the number available is their basic growth"; no free first-level units; cannot be
  flagged; never Sharpshooters, Enchanters or the four AB dragons. `vcmi`: `rollCreature()` (every
  creature with a monster object, not `excludeFromRandomization`), the count = growth each week
  (the camp never accumulates), the same messages; its popup shows the creature to a player who
  visited it. Three sources agree on the rule.

## 8. A player's defeat, the scenario, the towns

- `player_defeated` 0x4F11A0 (survey `E08-economy.md` §2.5) calls `set_dwelling_owner(index, −1)`
  for each of the player's dwellings: neutral again, the guards kept (they are the week's anyway).
  VCMI: a defeated player's objects become neutral. E10 builds the defeat.
- "Flag all creature dwellings" (victory goal 8; 5 shipped maps): every flag runs the scenario's
  test (E10). VCMI counts classes 17 and 20.
- The flag moves the towns' external dwellings counts (`town_dwelling_count_add`, E09 survey
  §8.3): +1 growth a week per owned dwelling in each of the owner's towns of its creature's
  faction (`wiki` Growth: "Flagged external dwelling: +1 creature per corresponding dwelling");
  E09's, with the town's growth.

## 9. What a player sees

- **The hint** (`map_hint` cases 17 and 20; E24 design §4.12) and **the right-click popup**
  (`object_popup_text` cases 17 and 20, 0x413EDB, 0x413F21): the dwelling's name (`CrGen1`,
  `CrGen4` by the record's subclass) and, owned, "Owned by …" (ARRAYTXT 23–30). Nothing of its
  counts or guards, for anyone. So its kind (the creatures) and owner are public; its counts and
  guards are not shown.
- **The Refugee Camp**: its name only (no case in either switch); its creature is named by its
  visit's question and seen with its count in the dialog.
- **At the visit** the visitor sees the guards' first creature and a size word of their total
  (the question), the counts (the dialog, its own or an ally's), what joined.
- `vcmi` (`getPopupComponents`): a dwelling's creature types to everyone, its count to its owner;
  a Refugee Camp's creature and count to a player who visited it this week. More than the
  original shows; the engine follows the original (ADR-0016 point 6's rule: what the original's
  screens show).

## 10. VCMI and the wiki at a glance

| Point | Original | VCMI 1.7.5 | Wiki |
|---|---|---|---|
| Guards at the start and each week | 3 × growth of level 5+ | the same | the same |
| Counts at the start and each week | growth, set | growth, set | cannot exceed the growth |
| The guards' battle | unsplit, owner −1 | unsplit, the owner's side | — |
| An ally's visit | question, a disabled dialog | nothing | no flag, no hiring |
| Level-1 creatures | free, own dwelling only | free | free |
| A full army and free creatures | refused | merges two stacks first | — |
| The most a player may buy | count, gold, one resource | the same | — |
| Random dwellings' dwelling | the last listing it | the first not banned (same) | — |
| The Refugee Camp | levels 1–7 less 132–137, growth, weekly | the same set, growth, weekly | the same |
| A dwelling's popup | name, owner | types to all, count to the owner | — |

## 11. Disputes, quirk candidates and open points

- **D-51 (the guards' side).** The original fights a dwelling's guards with owner −1, unowned,
  whoever owns the dwelling (`visit_dwelling` 0x4A1AEB pushes −1 where `visit_mine` pushes the
  owner); VCMI's battle puts them on the owner's side. Gameplay: in a game of players, who commands
  an owned dwelling's guards (the neutral seat or the owner, as increment 9's owned mines,
  ADV-MIN-002's `Defender::Garrison`). Recommended: **the original** (unowned, the neutral seat),
  checked by batch E10's E10-2. *Decided (ADV-Q42, decision 234):* a `fix-optional` quirk, either
  behaviour by a flag. *Revised (T88b's start, decision 238):* **the original, not a quirk**: the
  guards are the dwelling's own creatures, renewed weekly (§6.1), never its owner's troops (no
  Leave Guards, §4.1); no flag; E10-2 dropped.
- **D-52 (free creatures with a full army).** The original refuses (GENRLTXT 426, the count kept);
  VCMI with `mergeOnRecruit` merges two of the hero's stacks of one type first. Gameplay: a hero
  with seven slots, two of one type, gets the free creatures in VCMI. Recommended: **the
  original**, checked by E10-3. *Decided (decision 235):* the original. *Confirmed* by E10-3
  (§16, T91).
- **A-34 (an empty faction set gives Castle).** `random_allowed_faction` over an empty mask gives
  faction 0. VCMI draws any faction with a town. No shipped map has one; the editor never writes
  one. Recommended: **refused at the import** (an input bound, as A-31), no quirk flag.
- **A-35 (a dangling town identifier).** No town with the identifier: the search reads the entry
  before the towns' list (a garbage faction). VCMI draws any faction. None on the shipped maps.
  Recommended: **refused at the import**.
- **A-36 (classes 18 and 19, kinds past the tables).** The original stops the setup's case with an
  error box (18, 19) or reads past a table (17 subclass 80+, 20 subclass 2+). None on the shipped
  maps. Recommended: **refused at the import** (MCL-Q21 (b)), with a 217 subclass past 6 and a 218
  subclass past 8 and levels past 6 or the lowest above the highest.
- **Not disputes** (texts and screens, recorded): the guards' question names the first stack's
  creature with the total's size word (VCMI the first stack's); an ally sees the offer read-only
  (VCMI nothing); the Elemental Conflux's and Golem Factory's creature order (the original's
  Air, Fire, Earth, Water and Stone, Iron, Gold, Diamond; VCMI Air, Water, Fire, Earth and Iron,
  Stone, Gold, Diamond: the dialog's order and the guards' slot order).
- **Open:** whether an ally's dialog can be reached for a Refugee Camp (no owner: never an ally's);
  the slider's key handling (not read; the engine's command carries an amount); the war machines'
  branch of the dialog (the War Machine Factory, not in scope); the computer's hiring (E10). No
  fresh agent re-read this survey (the token rule; T77a2 did): every point that rests on the
  reading alone (D-51, D-52, the ally's disabled dialog, the purchase's bound, the template's
  move) has a row in batch E10 (each as read, §16; D-51's owned case dropped with E10-2).

## 12. Commands, decisions, events, the observation, replays and benchmarks

For T88b's drafts and design note (ADR-0015 to -0017, the GAM-DEC rules); a sketch, not decided:

- **The guards' question** reuses increment 9's fight offer (`OfferKind::Fight`, ADV-MIN-002), the
  battle a dwelling's (`Defender` unowned, D-51), the visit's rest (GAM-DEC-007) carrying the flag
  and the hire question after a win.
- **The hire question** is a yes/no offer (a new `OfferKind`); yes on one's own dwelling joins the
  level-1 creatures (an event per creature: joined, or no room) and, with a creature of level 2 or
  more, opens **the hire dialog**: a pending decision open for purchases until closed (as the
  University's, GAM-DEC-006): a command `Hire { creature, amount }` validated against the
  decision, the dwelling's count, the treasury (gold and one resource) and the army's room, and
  `Close`. An ally's visit: the offer shown read-only (an event or the decision with no legal
  purchase; the agent's call at T88b). The Refugee Camp: the same dialog for any visitor. E09
  reuses the dialog for towns (its increment 3; E09 decision 4).
- **Events:** the flag (increment 9's owner change, radius 3), joined, hired (creature, amount,
  cost), the guards' change (increment 9's `GuardsChanged` to the players that see it? nothing
  shows a dwelling's guards: none needed), the week's refill (no public event: nothing shows the
  counts).
- **The observation:** a dwelling's kind and owner public (ADV-OWN-003 amended); its counts and
  guards hidden; during the dialog the visitor sees the offer's counts and costs; a Refugee Camp's
  creature hidden but to its visitor during the question and the dialog.
- **State:** the dwellings' counts and guards and the camps' creatures and counts change weekly and
  on hiring: per-object data in the persistent state (the adventure state hash bumps); the random
  dwellings' draws when the game is built (the static map's footprint resolved per game).
- **Replays and benchmarks:** the adventure replay format bumps with the new commands; the week's
  refill over 99 dwellings (`Pandora's Box `) and 1 981 on the shipped maps is cheap; a workload
  with hiring in random play (the economy XL map's dwellings) when built.

## 13. Batch E10's plan

Run by T90, scored in §16 (T91). Increment 10's batch (named after the increment as batches E3 to E9; not the epic E10), **batch
E10**, runs on the client lane's harness in SoD on Easy, hot seat, from maps `h3-experiment` writes
(`docs/experiments/README.md`), as before (decision 232's agent's call). Its spec and recipe are
tasks of T88b's cut; this is the plan. **Two runs**: **E10a** the visits (E10-1, E10-3 to E10-7, day
1 and 2; E10-2 dropped by decision 238), **E10b** the week (E10-8 to E10-10, to day 8 or 15).

| Row | Settles | Setup | Record | The reading predicts |
|---|---|---|---|---|
| E10-1 | §2.2, §4.2, §4.3 | Red's hero (strong) visits a neutral Dragon Cliffs (Green Dragon), a neutral Golem Factory and a neutral Elemental Conflux | each question's text; each battle's defending stacks and side owner; the records' counts, guards and owner after | 3 Green Dragons; 9 Gold + 6 Diamond Golems; 12 Earth Elementals; unowned; flagged after each win, counts = growth |
| ~~E10-2~~ | D-51 | *Dropped (decision 238).* Blue owns a Dragon Cliffs at load (Blue human); Red's hero attacks it | the battle's sides: owner and controller | the defending side unowned (owner −1), not Blue's (VCMI Blue's) |
| E10-3 | §4.4, D-52 | Red's hero A visits a neutral Guardhouse (Pikemen) twice; hero B with seven stacks, two of Archers, none of Pikemen, visits another | A's army and the record after each visit; B's army and the record | +14 Pikemen then "no Pikemen"; B refused, the count kept (VCMI merges the Archers) |
| E10-4 | §5.2, §5.3 | Red with 1 000 gold owns a Barracks at load (Swordsmen: 300 gold, growth 4) and buys the most | the dialog's most; the treasury, the army and the count after | the most 3 (min(4, 1 000 ÷ 300)); 900 paid, the count 1 |
| E10-5 | §5.1, §5.2 | Red owns a Dragon Cliffs at load (Green Dragon: gold and 1 crystal), with 10 000 gold and no crystal; then with 1 crystal | the dialog's most each time | 0, then 1 (the crystal bounds it; no guards for its owner) |
| E10-6 | §4.4 | Red and Blue allied; Blue's Barracks visited by Red's hero | owner after; the dialog's buttons | not flagged; the dialog disabled |
| E10-7 | §3.2 | random dwellings: 216 with the set {Castle}, levels 6–6; 217 level 2 with {Tower}; 216 with {Rampart} levels 1–1 (the Dwarf: entrance left) | each object's class, subclass, anchor, entrance and blocked tiles; the Golem Factory's record | the Angel Portal moved one right; the Golem Factory with its four golems and guards; the Dwarf Cottage's entrance one left of the anchor |
| E10-8 | §6.1 | E10-1's dwellings on day 8 (one flagged by Red, its guards beaten; one never visited); a Barracks Red bought from on day 1 | counts and guards on day 8 | counts = growth (not added); guards back on both |
| E10-9 | §7 | a Refugee Camp: visited on day 1 (buy part), again on day 2, again on day 8 | its creature and count (tile words +0x22, +0x0) each time; the messages | a creature of levels 1–7 with its growth; the rest on day 2; a new creature and growth on day 8 |
| E10-10 | §9 | right-click and hover a neutral and an owned dwelling and a Refugee Camp (another player's turn) | the texts | name; name and owner; the camp's name only |

**Not in the batch:** A-34, A-35, A-36 (refused at the import), a defeat (E10), the scenario's test
(E10), the towns' counts (E09), the computer's hiring (E10). E10-6 needs teams in the hot-seat map
(the spec's alliances: check at the batch's task).

**What the harness lacks** (`clients/capture/` is a common path: built with the harness check, the
client and map-client lanes' inboxes told): reads of the dwelling records' creatures, counts and
guards (game +0x4E39C; the owner is read already, `h3mem` 0x57), a Refugee Camp's tile words, the
hire dialog (its most +0xAC, the amount +0xB8, the Max and Recruit items 532 and 30722, Close
30721), the battle's defending side's owner (E9-2 read the sides' owners and controllers already).
**What the spec lacks** (`h3-experiment`, battle-owned: `cross-lane`; the writer
`h3_map::writer` is this lane's): dwellings by subclass with an owner (`[[map.ownable]]` takes a
picture: check classes 17 and 20), random dwellings with a set and levels (the writer writes them,
`writer/extras.rs`), Refugee Camps, a hero's full army of named stacks, teams in a hot-seat map.

## 14. The proposed rule ids

New families **ADV-DWL** (dwellings) and **ADV-HIR** (hiring); the T88b drafts fix titles and
cuts.

| Id | Proposed rule | Rests on |
|---|---|---|
| ADV-DWL-001 | The dwelling kinds: 17 (one creature by subclass, 80 kinds) and 20 (four, 2 kinds) as content; their names; the import refuses 18, 19 and kinds past the tables (A-36) | §2.1, §2.3, §2.4 |
| ADV-DWL-002 | A dwelling when the game is built: each creature's count its growth; the guards 3 × growth of each creature of level 5+, one stack each in creature order | §2.2 |
| ADV-DWL-003 | The week: every dwelling's counts set to the growth, its guards replaced (owned or not; no week's or month's kind) | §6.1 |
| ADV-DWL-004 | The visit: own or ally's (no guards, no flag); another's or neutral: the guards' question and battle (unsplit, unowned: D-51), the survivors kept; won or unguarded: flagged; then the hire question | §4.1–§4.3 |
| ADV-DWL-005 | The random dwellings: faction, level, creature, dwelling kind, footprint and owner drawn when the game is built; the import's bounds (A-34, A-35, A-36's levels and subclasses); those tied to a town: E09 | §3 |
| ADV-HIR-001 | Free level-1 creatures on one's own dwelling: all join (a slot of the type or the first empty), the count to 0; no room: none join (D-52) | §4.4 |
| ADV-HIR-002 | The hire dialog: per creature at most min(count, gold ÷ cost, resource ÷ cost) and army room; paid and counted down; open until closed; an ally's read-only | §5 |
| ADV-RFG-001 | The Refugee Camp: its creature (levels 1–7, ADV-MON-003's set) and count (growth) when the game is built and each week; visited by anyone, hired through ADV-HIR-002, never flagged, no free creatures | §7 |
| GAM-DEC-008 | The hire dialog as a pending decision open for purchases until closed (as GAM-DEC-006) | §5.3 |
| ADV-OWN-001 (amended) | Dwellings 17 and 20 flagged by a visit after their guards; radius 3 for the team | §4.3, §4.5 |
| ADV-OWN-002 (amended) | Dwellings' owners at the start (and a random dwelling's) | §1, §2.2 |
| ADV-OWN-003 (amended) | A dwelling's kind and owner public, its counts and guards hidden; a Refugee Camp's creature hidden | §9 |
| ADV-OBJ-002 (amended) | The import's classes 17, 20, 78 and 216–218 and its refusals | §1, §2.4, §3.1 |
| GAM-CAL-002 (amended) | The new week's order with the dwellings' refill and the camps' draw | §6 |
| GAM-BAT (an amendment of the battle's surroundings or a new id) | A dwelling's guards fight unowned, without a hero, on the entrance tile's terrain | §4.2 |
| ADV-MON-003 (amended) | Its creature set also the Refugee Camp's draw (levels 1–7) | §7 |

E09 keeps the towns' external dwellings counts (growth +1 per owned dwelling) and the town's
recruitment through ADV-HIR-002's dialog (E09 decision 4; its inbox told when it lands).

## 15. For T88b: the checkpoint's questions (ADV-Q42)

Asked at T88a's end (`docs/OPEN-QUESTIONS.md`, ADV-Q42; the answers in the epic as decisions from
233):

1. **D-51**, the guards' side in a battle at an owned dwelling: the original (unowned, the neutral
   seat; recommended) or VCMI (the owner's, as an owned mine's).
2. **D-52**, free creatures with a full army: the original (refused; recommended) or VCMI (two
   stacks of one type merged first).
3. **The random dwellings' footprints**: the 206 in scope need each dwelling kind's template as
   content (82 footprints and the Angel Portal's move; recommended, in the increment, harvested
   from `Objects.txt`), or keep a resolved dwelling on the random dwelling's own 2 × 2 (not the
   original's tiles), or leave the random dwellings to E09 with those tied to random towns.
4. **Batch E10** before the build (recommended, as batch E9), after it, or not at all.

**Answered** (2026-11-03, `AskUserQuestion`; the epic's decisions 234–237): (1) D-51: asked how
the mines were (the owner plays an owned mine's guards: `visit_mine` passes the owner, and batch
E9's E9-2 saw Blue order each of its mine's stacks), the user made it **a `fix-optional` quirk**,
either behaviour by a flag (on in `faithful`: unowned; off: the owner's side, as a mine's), its
ledger entry once E10-2 confirms; (2) D-52 **as the original**; (3) **the original's footprints**
in the increment; (4) **batch E10 before the build**. *Revised* at T88b's start (decision 238): the user, on how
the guards come to an owned dwelling, made D-51 **the original's rule, not a quirk** (no flag) and
dropped E10-2.


## 16. Batch E10 scored (T91)

Scored on 2026-11-03 (E08 T91; decision 247: whole) from the recorded runs: E10a
`experiments/results/E08-batch-10a.md` (T90: two runs, 72 of 74 records equal; the differences
the starting bonus and a level-up's offer) and E10b `E08-batch-10b.md` (T90: five runs; its last
two differ in the starting gold and the camp's day-1 creature), played by the capture harness
(`clients/capture/batches/E08-batch-10a.toml`, `-10b.toml`) in SoD 4.0 on Easy, hot seat, and read
from the game's memory. Each row against §13's prediction (§13's levels are zero-based, the
specs' one-based: "levels 6–6" is the spec's "7 to 7"; the texts by record, counted from 0).

| Row | Predicted | Observed | Verdict |
|---|---|---|---|
| E10-1 (§2.2, §4.2, §4.3) | 3 Green Dragons; 9 Gold + 6 Diamond Golems; 12 Earth Elementals; unowned; flagged after each win, counts = growth; the size word the guards' total's (design §16) | the records before: D 1 Green Dragon, F 6, 6, 3, 2 (Stone, Iron, Gold, Diamond), X 6, 5, 4, 6 (Air, Fire, Earth, Water), the guards as predicted, one stack each in creature order; GENRLTXT 422 with ARRAYTXT 176 "a few" (D, 3) and 182 "a pack of" (F 15, its first stack 9 would be 179 "several"; X 12); each battle on the tile's grass, no special field, the defending stacks the guards unsplit by slot, the defending side's owner none and not human; won; then the hire question (ADVEVENT 35 at D, 36 at F and X), answered no; each record Red's, its guards empty, its counts kept | confirmed (the defender unowned as `visit_dwelling` pushes −1; the size word the total's) |
| E10-3 (§4.4, D-52) | +14 Pikemen then "no Pikemen"; B refused, the count kept (VCMI merges the Archers) | P (neutral, unguarded) flagged Red's, ADVEVENT 35, yes: GENRLTXT 424, 14 Pikemen in E10-3's first empty slot, the count 0; again: ADVEVENT 35, yes: GENRLTXT 423, nothing; Q with E10-3b's seven stacks (two of Archers): flagged Red's, ADVEVENT 35, yes: GENRLTXT 426, the army as it was, the count 14 kept; no hire dialog (a level-1 dwelling), no battle | **D-52 confirmed**: the original's refusal; VCMI's merge is not the original's |
| E10-4 (§5.2, §5.3) | the most 3 (min(4, 1 000 ÷ 300)); 900 paid, the count 1; the dialog closed after the purchase (design §16) | no guards' question; ADVEVENT 35; the dialog opened with the most 3, the amount 0, Max and the slider enabled, Recruit disabled (amount 0); Max, Recruit: 3 Swordsmen in slot 1, 1 000 → 100 gold, the count 4 → 1; the dialog closed | confirmed |
| E10-5 (§5.1, §5.2) | 0, then 1 (the crystal bounds it; no guards for its owner) | owned by Red at load and guarded (3 Green Dragons, kept in the record throughout); no guards' question; with 10 000 gold and no crystal the most 0, Max and the slider disabled; after the crystal (1) the most 1; Max, Recruit: 1 Green Dragon for 2 400 gold and 1 crystal (10 000 → 7 600, 1 → 0), the count 0, the dialog closed | confirmed: an owned dwelling holds its guards, its owner never meets them (§4.1, §6.1) |
| E10-6 (§4.4) | not flagged; the dialog disabled | the team bytes red 0, blue 0, tan 1 (the hot seat keeps the map's teams; design §16's fallback not needed); U (Blue's Barracks): ADVEVENT 35, yes: the dialog with +0x4C set, the most 3 from Red's 1 000 gold, Max, Recruit and the slider disabled, Close enabled; Max changed nothing; V (Blue's, guarded): no guards' question, ADVEVENT 35, the dialog with the most 0, the same buttons; both records Blue's, their counts and V's guards unchanged, Red's treasury unchanged, at Blue's turn and on day 2 too | confirmed: an ally skips the guards and the flag, sees the offer and buys nothing |
| E10-7 (§3.2) | the Angel Portal moved one right; the Golem Factory with its four golems and guards; the Dwarf Cottage's entrance one left of the anchor | R1 (216, {Castle}, level 7) class 17 subclass 8, anchor (13, 30): one right of the spec's (12, 30), its entrance (12, 30), blocked (11, 30); 1 Angel, 3 Angels' guards. R2 (217, {Tower}, level 3) class 20 subclass 1 at (16, 30), the entrance the anchor, blocked (15, 30); 6, 6, 3, 2 golems, 9 Gold and 6 Diamond Golems' guards. R3 (216, {Rampart}, level 2) class 17 subclass 12 at (21, 30), the entrance (20, 30), blocked (21, 30); 8 Dwarves, no guards. All three unowned; row 29 free | confirmed (T90's note that the anchors differ from the spec's `at` is this move) |
| E10-8 (§6.1) | counts = growth (not added); guards back on both | S: 2 of 4 bought on day 1 (the most 4; the dialog closed), 2 on days 2 and 7, 4 on day 8 (not 6); F, never visited: 6, 6, 3, 2 and 9 and 6 guards throughout (not added on day 8); D, flagged by Red on day 1 (its guards beaten): no guards on days 2 and 7, 3 Green Dragons again on day 8, still Red's; day 8 the week's first day (week 2, day 1; the week's message) | confirmed (the week was the Beholder's: no week's creature reached these, untested as read) |
| E10-9 (§7) | a creature of levels 1–7 with its growth; the rest on day 2; a new creature and growth on day 8 | the camp's entrance word (19, 10) +0x22 creature 131 (Phoenix), +0x0 count 2 (its growth), its other tile −1; no owner; the question (ADVEVENT 112's text, see below), the dialog's most 2; 1 bought for 2 000 gold and 1 mercury, the dialog closed, the count 1; day 2: the most 1, bought, 0; the next visit ADVEVENT 44 alone; day 7 still 0; day 8 Obsidian Gargoyles (31) × 9, the most 9 | confirmed |
| E10-10 (§9) | name; name and owner; the camp's name only | the hint: D's and S's name with " - " and ARRAYTXT 23 ("Owned by red player"), F's name, the camp's name; the popup the same lines (the name, an empty line, the owner's); the camp's popup its name only to Blue and to Red, who had bought there that day (VCMI shows a visitor the creature) | confirmed |

**Beyond the predictions:**
- **The camp's question** is ADVEVENT 112 by the reading; its text is ADVEVENT 35's, so the screen
  cannot tell them apart (the harness names the first record). A client's choice of record, no
  rule.
- **A one-creature dialog closes after any purchase**, of part of the count too (E10-9's 1 of 2,
  E10-8's 2 of 4), as §5.3 reads; **an ally's dialog computes the most** and only disables the
  buttons (E10-6's U: 3), as §5.2.
- **The flag needs no join:** Q was flagged although nothing joined (the flag before the question,
  §4.3).
- **The draws** (the game's seed, no rule): the camp's day-1 creature differed between E10b's
  last two runs, its week-2 creature was the same in all; the engine draws from its own stream
  (decision 17).
- **A player without towns** (E10a's Red) saw GENRLTXT 7 ("… lost your last town …") after D's
  battle, the harness's known message (`clients/capture/README.md`): E10's defeat, not this
  increment's. **Tan's turn** (the computer, walled in) changed nothing read (its treasury, its
  hero's tile and points): the computer's hiring stays E10's (§11).
- **Not read:** the scenario's information on the teams (the harness reads no screen text there);
  the team bytes stand in.

**The rules** (§14): the new ones cite these rows as `observation` sources and are written
`verified` when T93 to T95 build them: E10-1 ADV-DWL-002, ADV-DWL-004, ADV-OWN-001 (amended) and
the guards' battle (GAM-BAT-001 amended, design §12); E10-3 ADV-HIR-001 (D-52), ADV-DWL-004; E10-4
ADV-HIR-002, GAM-DEC-008; E10-5 ADV-HIR-002, ADV-DWL-002, ADV-DWL-004; E10-6 ADV-DWL-004,
ADV-HIR-002, ADV-OWN-001; E10-7 ADV-DWL-005, ADV-DWL-002; E10-8 ADV-DWL-003, GAM-CAL-002
(amended), ADV-DWL-002; E10-9 ADV-RFG-001, ADV-HIR-002, ADV-MON-003 (amended); E10-10 ADV-OWN-003
(amended). No rule in the catalog gains a row now: each row tests what the increment adds or
amends. No `ADV-Q`; D-51 stays as decided (decision 238; E10-1 shows the reading's owner −1 at
neutral dwellings, the owned case dropped with E10-2); A-34 to A-36 are refused at the import and
not in the batch.

## 17. A dwelling entered from water (T97b1)

Read 2026-11-05 (E08 T97b1, decision 259; the inbox's note of 2026-11-04 and decision 256: four
shipped maps were refused as `Unsupported::OwnedOnWater` for a dwelling whose entrance lies on
water). The human player's path, a hero in a boat; the computer's (`ai_dwelling_visit`) not
followed.

- **The step.** A boat's search takes another entrance on water as a normal step (`E08-sea.md`
  §4.1 point 3). The move: `hero_move_step`'s class switch (0x480C12) sends 17, 20 and 216 to 218
  (the random ones are dwellings by the game's start) to its default case 0x480F27, where a class
  whose `object_settings` +0 is clear (entered, not on the "cannot be entered" list) goes on as a
  step (0x480F34 to 0x480CB5); after the step the tile's entrance bit (+0xD bit 0x10) sends a hero
  in a boat to the visit without the flyer's test (0x48143B, 0x48145B). The hero stays in its boat
  on the entrance.
- **The visit and the hire.** `visit_dwelling` 0x4A18B0, `dwelling_offer_hire` 0x4ABA50 and
  `visit_refugee_camp` 0x4A4600 (all read whole by T88a) hold no test of the hero's in-boat bit
  (+0x105 bit 18, as 0x40000 or as a byte of +0x105 to +0x107) and no call to `hero_boat`
  0x4CE900: the guards' question, the flag, the hire question, the free creatures and the hire
  dialog (into the hero's army, `recruit_dialog_init` with it) are §4's and §5's.
- **The guards' battle.** `adventure_battle` with the visitor attacking at the entrance;
  `battle_setup_battlefield` 0x4643B0 tests the attacker's hero for the in-boat bit first, so it
  is fought on water and the deck (`E08-sea.md` §7.1); a special ground on the tile would replace
  the picture (none lies on water). After a win the hero stays in its boat; a loss or a retreat
  frees its boat with it (`E08-sea.md` §7.3).
- **VCMI 1.7.5:** on the sail layer a visitable object that is no boat, hero, town, garrison or
  border gate is a `VISIT` (`lib/pathfinder/PathfindingRules.cpp`, `DestinationActionRule`);
  `CGDwelling::onHeroVisit` tests no boat; the guards' battle (`startBattle` at the dwelling's
  visitable tile) takes water's battlefield, the deck (`CGameState::battleGetBattlefieldType`,
  `E08-sea.md` §7.4). **Wiki:** the Creature dwelling page says nothing of water or boats; the
  Combat page's "Boat (Deck)" battlefield covers "monsters or locations on a boat" (§7.4).
  **Consensus**: a dwelling entered from water is visited from a boat as on land.
- **The engine** already behaves so (ADV-SEA-005: an entered object's tile on water is entered
  from a boat; GAM-BAT-001: a battle with the attacker in a boat on the deck; `visit_dwelling`,
  the hire question and `check_hire` know no boat), so only the import's refusal goes: a dwelling
  or a random dwelling entered from water imports (map import v19, ADV-OWN-002, ADV-DWL-004); the
  other ownable kinds (mines, abandoned mines, Lighthouses, Shipyards, towns) keep
  `OwnedOnWater`, unread. Only a hero in a boat reaches such an entrance in the engine (an
  object's tile on water is closed to a walk, to a flight, D-46, and to a water walk).
- **The census** (map import v19, `original_data`, seed 0): all 160 shipped maps import, validate
  and build (v18: 156); the 9 dwellings entered from water are typed, all on the four maps
  refused before (Adventures of Jared Haret, Rise of the Phoenix and its Allied version, Serpents
  Treasure). With the same bump T97a's finding: a random dwelling tied to a town (1 397, all to
  random towns) has its levels and owner checked before it stays a blocker; every one passes.
