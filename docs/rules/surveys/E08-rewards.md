# E08 rewards survey (E08 T55a): treasure chests, artifacts and scrolls as pickups, the monsters' treasures, the objects that teach, the rewards that give creatures

The original's rules for the objects that hand a visiting hero something of the hero's own
(increment 7, E08 decisions 121 and 122), by code reading (`docs/exe/README.md`) against VCMI
1.7.5 and the wiki, with a census of the 160 shipped maps. Cut in four parts (decision 125):
**T55a1** the census and the pickups (§1 to §5, done), **T55a2** the objects that teach
(experience, spells, skills; §6), **T55a3** the rewards that give creatures, Pandora's Box and
events (§7), **T55a4** the common rules, batch E7's plan, the rule ids, the second reading and the
checkpoint's questions (§8 on). Sections of a later part are added by that part; §9 and §10 below
collect what each part leaves for T55a4.

Sources (labels as in `docs/04-rules-and-sources.md`):

- `original-data` (read): `Heroes3.exe` (SHA-256 `aac39e8c…`) with `scripts/exe.py`; every address
  cited is in `docs/exe/map.toml`. The human player's path is read (a visit's last argument, "the
  hero's owner is the local human"); the computer player's branches are named, not read, unless
  said.
- `vcmi`: VCMI 1.7.5 (`git clone --depth 1 --branch 1.7.5`): `lib/mapObjects/MiscObjects.cpp`
  (`CGArtifact::pickRandomObject`, `initObj`, `onHeroVisit`, `pick`, `battleFinished`,
  `blockingDialogAnswered`), `lib/callback/GameRandomizer.cpp` (`rollArtifact`),
  `lib/entities/artifact/CArtHandler.cpp` (`legalArtifact`), `config/objects/rewardablePickable.json`
  (`treasureChest`), `lib/rewardable/Interface.cpp` and `lib/mapObjects/CGHeroInstance.cpp`
  (`calculateXp`), `lib/mapObjects/CGCreature.cpp` (`giveReward`), `config/gameConfig.json`
  (`backpackSize` -1); links `https://github.com/vcmi/vcmi/blob/1.7.5/<file>`.
- `wiki`: read through the summarising fetch tool, `https://heroes.thelazy.net/index.php/<Page>`:
  Treasure_chest, Artifact, Spell_scroll (the pages found through the search tool; a direct fetch
  of a page not linked from a search is refused in this session). Quotes are short.
- `original-data` (the census): the 160 shipped maps (`original-data/maps.tar`) read with `h3-map`'s
  reader and imported, by a throwaway example (not committed), §1.

## 0. Findings in short (T55a1)

- **Today** the import keeps every object of §2 to §4 as an inert blocker (classes 5, 65 to 69, 93,
  101; `docs/formats/h3m.md`, "Import") and refuses 8 maps for a monster with an artifact (§5).
- **Treasure chest (101, §2):** drawn at a new game: 32 % 1 000 gold, 32 % 1 500, 31 % 2 000, 5 % a
  random treasure artifact (in the tutorial always 1 500 gold). A visit with gold asks gold or
  experience (gold − 500, times the hero's Learning factor and truncated, as battle experience);
  an artifact goes to the hero, or with a full backpack (64) the chest becomes the 1 000 gold or
  500 experience choice. Removed either way. Code, VCMI and the wiki agree.
- **Placed artifacts (5) and random ones (65 to 69, §3):** a random one becomes a placed one at a new
  game (65 any of treasure, minor or major, no relic; 66 to 69 one class each), never one already
  on the map or banned until the pool is exhausted. A visit with a full backpack refuses (nothing
  happens, even when a body position is free; the wiki says the same); else the artifact goes to
  the hero and the object is removed. A guarded artifact **always asks** (the map's text, else a
  built "guarded by …" question), a win gives it. The visit's code also holds six pickup
  conditions (paying gold, Wisdom, Leadership, a fight) that no map can reach:
  the loader and the new game clear them (§3.3).
- **Spell Scrolls (93, §4):** an artifact pickup of the Spell Scroll with the map's spell; a
  guarded scroll asks only with a text and fights at once without one (as guarded resource piles,
  unlike guarded artifacts).
- **Monsters' treasures with an artifact (§5):** after a win or a join the artifact goes to the hero,
  then the resources to the owner; with a full backpack the artifact is lost.
- **Disputes for the checkpoint** (§9): D-29 a guarded artifact without a text (code and VCMI ask;
  the wiki says it fights at once), D-30 where a reward's artifact goes (code: the first body
  position that takes it; the wiki: the backpack), D-31 which classes a random artifact (65) may be
  (code: no relic; VCMI: any), D-32 which artifacts a random one may repeat (code: none on the map
  or given; VCMI: the least used).

## 1. The census

`original-data` (the 160 shipped maps: 49 RoE, 55 AB, 56 SoD; today's import takes 5 whole:
Adventures of Jared Haret, Faeries, Knee Deep in the Dead, Race for Ardintinny, the Tutorial; 128
are refused for a hero generated at a main town, 18 for a random hero, 8 for a monster with an
artifact, 1 for a hero in a town). Every class that may hand a visiting hero something, with the
part of the survey that reads it. "Maps" counts the maps that hold the class; "whole" the objects
on the five maps that import whole.

| Class | Object | Objects | Maps | Whole | Part |
|---|---|---|---|---|---|
| 101 | Treasure Chest | 11 720 | 157 | 212 | T55a1, §2 |
| 5 | Artifact | 362 | 68 | 33 | T55a1, §3 |
| 65 / 66 / 67 / 68 / 69 | Random artifact (any, treasure, minor, major, relic) | 70 / 1 253 / 1 216 / 574 / 258 | 23 / 132 / 138 / 118 / 90 | 8 / 12 / 9 / 6 / 2 | T55a1, §3 |
| 93 | Spell Scroll | 100 | 21 | 2 | T55a1, §4 |
| 54, 71–75, 162–164 | Monster with a treasure (with an artifact) | 355 (53) | — (18) | — | T55a1, §5 |
| 4 | Arena | 248 | 111 | 1 | T55a2 |
| 23 | Marletto Tower | 461 | 143 | 6 | T55a2 |
| 32 | Garden of Revelation | 432 | 136 | 10 | T55a2 |
| 41 | Library of Enlightenment | 135 | 84 | 7 | T55a2 |
| 47 | School of Magic | 430 | 131 | 10 | T55a2 |
| 51 | Mercenary Camp | 500 | 136 | 12 | T55a2 |
| 61 | Star Axis | 527 | 140 | 14 | T55a2 |
| 81 | Scholar | 694 | 131 | 9 | T55a2 |
| 88 / 89 / 90 | Shrine of Magic Incantation / Gesture / Thought | 530 / 375 / 338 | 126 / 113 / 109 | 32 / 30 / 27 | T55a2 |
| 100 | Learning Stone | 1 072 | 155 | 21 | T55a2 |
| 102 | Tree of Knowledge | 341 | 119 | 6 | T55a2 |
| 107 | School of War | 302 | 116 | 10 | T55a2 |
| 113 | Witch Hut | 772 | 144 | 26 | T55a2 |
| 22 | Corpse | 321 | 82 | 7 | T55a2 (an artifact) |
| 105 | Wagon | 163 | 74 | 5 | T55a2 (an artifact or resources) |
| 108 | Warrior's Tomb | 261 | 82 | 1 | T55a2 (an artifact) |
| 39 | Lean To | 101 | 57 | 7 | T55a2 (resources; the list's edge) |
| 104 | University | 143 | 75 | 0 | T55a2 (skills for gold) |
| 6 | Pandora's Box | 46 | 20 | 0 | T55a3 |
| 26 | Event | 486 | 68 | 48 | T55a3 |
| 78 | Refugee Camp | 321 | 101 | 5 | T55a3 |
| 16 | Creature Bank (subclasses 0–6) | 1 215 | 62–106 | 19 | T55a3 decides in or out |
| 24 / 25 / 84 / 85 | Derelict Ship / Dragon Utopia / Crypt / Shipwreck | 170 / 74 / 277 / 401 | 58 / 55 / 97 / 69 | 2 / 1 / 4 / 10 | T55a3 decides in or out |
| 63 | Pyramid | 60 | 41 | 2 | T55a3 decides in or out |
| 83 | Seer's Hut (subclasses 0–2) | 265 | 41–56 | 0 | T55a3 (where it belongs) |
| 215 | Quest Guard | 114 | 31 | 0 | T55a3 (where it belongs) |
| 62 | Prison | 89 | 41 | 3 | not in increment 7 (a hero; the queue's inbox, 2026-10-20) |
| 82 / 86 / 29 / 59 | Sea Chest / Shipwreck Survivor / Flotsam / Ocean Bottle | 826 / 362 / 2 019 / 29 | 80 / 72 / 90 / 15 | 37 / 7 / 92 / 3 | not in it: the sea (decision 121, the sketch's item 8) |

The bodies, for the parts of T55a1 (the rest with their parts):

- **Placed artifacts (5):** 99 distinct artifacts: 101 treasure, 77 minor, 93 major, 91 relic
  objects; none is the Spell Book, the Spell Scroll, the Grail or a war machine; one combination
  (Admiral's Hat, SoD), no other combination. 28 have a message record: 14 with guards (8 of them
  without a text), 14 without guards (a text only).
- **Random artifacts (65 to 69):** 3 371; 23 with a message record, 19 of them guarded (8 without a
  text).
- **Scrolls (93):** 100 with 30 spells, 33 of them adventure spells (Summon Boat 4, View Earth 8,
  View Air 10, Fly 4, Water Walk 1, Dimension Door 2, Town Portal 4); 10 with a message record: 3
  guarded (all with a text), 7 without guards (2 with a text).
- **Chests (101):** no body (the contents are drawn, §2.1). Up to 330 on one map (Tale of two
  lands); none on 3 maps; the Tutorial has 1.
- **Monsters' treasures:** 355 monsters have a treasure record (253 with a text, 97 with
  resources, none below 0), 53 an artifact (19 treasure, 11 minor, 17 major, 6 relic; 9 with
  resources too) on 18 maps; today's import refuses Brave New World and its allied version,
  Hoard and its allied version, Myth and Legend, Pandora's Box, Treasure Hunt and Unholy Quest for
  it (the other 10 maps are refused first for another reason).

## 2. Treasure chests (class 101)

### 2.1 The draw at a new game

`original-data` (read): `setup_map_objects` (0x4C0CC0, case 101 at 0x4C1DD8) draws once per chest,
in its tile walk (level, row, column), r = `random_between(0, 99)`; in the tutorial (game
+0x1F69D set) r is taken as 60 after the draw. r < 32: 1 000 gold; r < 64: 1 500; r < 95: 2 000;
else an artifact `random_artifact_of_level(2)` (0x4C94D0, a treasure artifact, §3.1's pool). The
tile's word holds it: bit 10 set and bits 0 to 9 the artifact, or bits 11 to 14 the gold in 500s
(2, 3 or 4). The chests are drawn after the random artifacts are resolved (§3.1), so a chest's
artifact is never one placed, banned or drawn before it (until the pool is exhausted).

### 2.2 The visit

`original-data` (read): `visit_treasure_chest` (0x4A6520, from `visit_object`'s case 101):

1. **An artifact** (bit 10): with a full backpack (`hero_backpack_count` 0x4D90C0 ≥ 64) the chest
   is treated as a 1 000 gold chest (step 2 with 1 000, whatever was drawn). Else a human sees the
   artifact (a message with its picture), and the artifact goes to the hero
   (`hero_give_artifact` 0x4E3070: worn at the first body position that takes it, else the
   backpack; in SoD a received part may complete a combination, survey `E08-army.md` §5.3).
2. **Gold** (bits 11 to 14, × 500), `chest_gold_or_experience` (0x4A6440): the experience is
   x = trunc(f32(gold − 500) × the hero's Learning factor) (`hero_learning` 0x4E4840, the f32
   product of HER-EXP-001's battle experience): 500, 1 000 or 1 500 before Learning. A human
   chooses in a two-picture dialog (gold first, experience second; the pictures' items are
   0x7809 + their index, 0x4F731B in 0x4F6990): the gold picture or a plain OK gives the gold to
   the owner (`give_resource` 0x4E3600), the experience picture the experience
   (`hero_gain_experience` 0x4E33B0, its levels given at once). The computer player (0x527FE0,
   read; E10) takes the gold when trunc(f64(gold) × the player's value of gold) exceeds
   trunc(f32(x) × the hero's value of experience, hero +0x109).
3. The chest is removed (`remove_map_object` 0x4AABB0; a pickup sound drawn from 1 to 7 is
   presentation), then `hero_level_up` (0x4DA720) runs any level pending.

Nothing is refused: a chest is always taken, whatever the hero carries. The visit itself draws
nothing that is not presentation.

### 2.3 VCMI and the wiki

`vcmi` (`treasureChest`): the same four outcomes and chances (`appearChance` 32, 32, 31, 5; the
experience 500, 1 000, 1 500 through `calculateXp`, the hero's experience bonus percent), the
artifact a treasure one drawn at the map's start (`gainedArtifact` of class `TREASURE`), removed
after the visit; no full-backpack case (its backpack is unlimited, `backpackSize` -1); nothing for
the tutorial. `wiki` (Treasure chest): the same table ("32%: 1000 gold or 500 experience", 1 500
or 1 000, 2 000 or 1 500, "5%: an artifact"); "a random treasure class artifact is put in the
hero's backpack" (D-30: the code wears it when a position takes it); "If the hero's backpack is
full, the chest offers gold and experience". **Consensus** on the chances and the choice; the
1 000 gold of the full-backpack case is the code's alone (the wiki is silent on the amount);
D-30.

## 3. Artifacts as pickups (classes 5 and 65 to 69)

### 3.1 At load and at a new game

`original-data` (read):

- **The body** (`load_artifact_body` 0x4FF120, from `load_object_body` 0x502E00's case for 5 and
  65 to 69): a flag; when set, a message record (a text and up to seven guard stacks, 0x4FEE50,
  appended to the map's records, at most 4 000). The tile's word gets bit
  31 and the record's index in bits 19 to 30; the low bits stay 0. The artifact is the template's
  subclass (tile +0x22).
- **The pool** (`artifacts_given_init` 0x4C0AA0, from `new_game_heroes` 0x4BFE70 at 0x4BFFF0): the
  game's "given" marks (+0x4E224, one byte per artifact) start as a copy of the map's banned
  artifacts (+0x4E2B4), then every placed artifact (a class 5 entrance tile) marks its own.
- **Random artifacts** (`resolve_random_objects` 0x4C9DD0, next, at 0x4BFFF7), in the tile walk
  (level, row, column): 65 draws from the treasure, minor and major classes together (bits 0xE),
  66 treasure (2), 67 minor (4), 68 major (8), 69 relic (0x10), by `random_artifact_of_level`
  (0x4C94D0, read at E24 M1 T4b): uniform among the artifacts of those classes not disabled and
  not marked; none left, the marks restart from the bans, and with still none any class; the
  drawn one is marked. The object becomes a placed artifact (class 5). Then the chests (§2.1).
- **The new game's pass** (`setup_map_objects` case 5, 0x4C0DC8): an artifact without a record
  costs a draw `random_between(0, 99)` whose result is dropped, and its word's low 4 bits are
  cleared (§3.3). The draw moves the original's one generator only; the engine draws from its own
  setup stream in an order of its own (decision 17).

### 3.2 The visit

`original-data` (read): `visit_artifact` (0x49F7E0, from `visit_object`'s case 5):

1. **A full backpack** (≥ 64, 0x4D90C0): a human is told the backpack is full; nothing changes and
   the object stays, even when a body position would take the artifact.
2. **A message record** (bit 31): `visit_guarded_artifact` (0x49F070):
   - **no guards** (the record's flag +0x10 clear, or no stack): a human sees the record's text
     (or, when empty, the artifact's own pickup text), then the pickup (step 4);
   - **guards:** a human is asked a yes-or-no question: the record's text, or when it has none a
     built one naming the guards ("guarded by …", their creatures and quantity words, a general
     text); the computer player asks `ai_fight_answer` (0x52BD10; E10). No: nothing happens. Yes:
     the battle with the guards (`adventure_battle` 0x4AD470, called as for a guarded pile, survey
     `E08-battles.md` §5.1). Lost: as any lost battle. Won: the levels the
     battle gave (0x4DA720), a human is told the artifact is won, the hero gets it, the object is
     removed.
3. **The pickup conditions** (the word's low 4 bits as a signed nibble; §3.3): 0 the plain pickup
   (step 4); 1 to 6 the dead branches.
4. **The plain pickup:** a human sees the artifact's pickup text with its picture; the hero gets it
   (`hero_give_artifact`); the object is removed (0x49E170, with the pickup sound); 0x4DA720.

### 3.3 The dead pickup conditions

`original-data` (read: the dispatch at 0x49F862 to 0x49FA5C and its table at 0x49FA6C; the
callees 0x49EA40, 0x49ED50 and 0x49E8F0 by their arguments and calls only): nibbles 1, 4 and 5
ask to pay 2 000, 2 500 or 3 000 gold with a second value 0, 3 or 5 (0x49ED50, not read: a
resource or its amount), 2 asks for Wisdom (hero +0xD0, skill 7), 3 for Leadership (+0xCF, skill
6), 6 a fight (0x49EA40, through `monster_battle` 0x4AC580); each with its own text. **No map
reaches them:** the loader leaves the nibble 0 (§3.1) and the new game's pass clears it for every
artifact without a record; nothing else read writes it. Not built, not a quirk: no player can
meet them.

### 3.4 VCMI and the wiki

`vcmi` (`CGArtifact`): random artifacts rolled at the start (`pickRandomObject`): 65 with
`rollArtifact()` over every legal artifact (no combination, any class from treasure to relic:
**D-31**), 66 to 69 by class; the pick among the allowed ones least used so far (`allocatedArtifacts`
counts only rolls: **D-32**), none left the Grail. The visit: no guards, the artifact if
`canBePutAt` the hero, else "ADVOB_TXT 2" (the full-backpack text), then `pick`
(`FIRST_AVAILABLE`); guards: a question with the map's text, else GENERAL_TXT 420 with the guards'
description ("Guard text is more complex in H3"), yes fights, a win picks. No pickup conditions.
`wiki` (Artifact): "The backpack can hold up to 64 unequipped artifacts"; when full, "no more
artifacts can be picked up even if its slot is empty" (**consensus** with the code's step 1);
guarded artifacts: "text appears asking whether or not the hero wants to attack the guardians",
and without that text "no question is asked and the guards attack immediately" (**D-29**: the
code and VCMI ask); random artifacts by class ("Treasure, Minor, Major, and Relic"; silent on 65);
nothing on pickup conditions or repeats.

## 4. Spell Scrolls (class 93)

`original-data` (read): the body (`load_scroll_body` 0x4FF2F0) is an artifact's (the record flag,
the record) and then the spell: one byte into the word's low byte, three more bytes skipped (the
format's u32; the spell is its low byte). No draw at a new game (93 is not among
`setup_map_objects`' cases). The visit (`visit_spell_scroll` 0x4A5EA0): a full backpack refuses as
§3.2 step 1; a record goes to `visit_guarded_scroll` (0x4A5A80); else a human sees the spell's
picture, the hero gets the Spell Scroll (artifact 1) with the spell (`hero_give_artifact`), the
object is removed. No level-up call at its end (an artifact's has one; nothing is pending
anyway). **A guarded scroll** (0x4A5A80): the spell is the word's low byte; guards with a text ask
it; guards **without a text fight at once** (0x4A5AFE); no guards show the text and pick it up; a
win picks it up and removes the object. So the guarded scroll is the guarded pile's shape
(ADV-PIK-004), not the guarded artifact's.

`vcmi`: the same (`SPELL_SCROLL`: "if(!message.empty())" a question, else
`blockingDialogAnswered(…, true)`); the scroll keeps its spell (`getScrollSpellID`). `wiki` (Spell
scroll): silent on the map's scrolls ("find[s] a spell scroll inscribed with the (...) spell");
"Scrolls go into the backpack when picked up" (**D-30**); while worn the scroll's spell is
available, also without a book or Wisdom (the hero's derived spells: T55a4). **Consensus** but
D-30. The census has no guarded scroll without a text (§1), so the asymmetry meets only made maps.

The engine already carries a scroll's spell on the artifact (`ArtifactInstance::scroll`,
`h3-heroes`); 33 of the 100 shipped scrolls hold adventure spells the engine does not cast yet
(the epic's "not placed yet").

## 5. The monsters' treasures with an artifact

`original-data` (read): `monster_treasure` (0x4A6B30; after a won battle, `monster_fight` 0x4A6C60,
or a join, 0x4A7000 and 0x4A7250; survey `E08-battles.md` §3.3): the monster's word bit 31 and its
record's index in bits 19 to 26 (records of 0x30 bytes). The artifact (record +0x2C, -1 none)
first: with a full backpack (≥ 64) a human is told so and **the artifact is lost**; else a human
sees it and the hero gets it (`hero_give_artifact`). Then each resource of the seven that is not 0
(+0x10, i32 each) goes to the owner (`give_resource`, a human told one message each). The text is
shown before (not read here; presentation).

`vcmi` (`CGCreature::giveReward`): the resources, then the artifact (`giveHeroNewArtifact`,
`FIRST_AVAILABLE`), one window "% has found treasure"; no full-backpack case. `wiki` (Artifact):
artifacts from creature banks or defeated heroes that find no room "will just disappear"; silent on
a monster's own treasure. **Consensus** (the order of the two gifts is presentation; the loss on a
full backpack is the code's, in line with the wiki's rule for banks and heroes).

The import then takes the 53 artifacts (the 8 refused maps move on to their next refusal or import
whole); `MonsterTreasure.artifact` is already read by `h3-map`.

## 9. Disputes and quirk candidates so far (for T55a4)

| Id | What | Sources | Proposal |
|---|---|---|---|
| D-29 | a guarded artifact without a text | code (0x49F070) and VCMI: a question naming the guards; wiki: "no question is asked and the guards attack immediately" | the code's (the question); a batch E7 row |
| D-30 | where a reward's artifact goes | code: `hero_give_artifact`, the first body position that takes it, else the backpack (HER-CHG-002, implemented, not yet verified); wiki: a chest's artifact "is put in the hero's backpack", "Scrolls go into the backpack" | the code's; a batch E7 row (a scroll or an artifact picked up with its position free) |
| D-31 | the classes of a random artifact (65) | code: treasure, minor or major (0xE); VCMI: any legal class, relics too; wiki silent | the code's; a batch E7 row (many class 65 objects, no relic among them) |
| D-32 | what a random artifact may repeat | code: never a banned, placed or already drawn one until the pool is exhausted; VCMI: the least rolled, placed ones not counted; wiki silent | the code's; no row (rare: a pool exhausted) |

Quirk candidates: none so far. Not quirks: the full backpack that refuses a wearable artifact (the
wiki states it), the chest's 1 000 gold with a full backpack, the monster's artifact lost on a full
backpack (the wiki's rule for banks), the dead pickup conditions (§3.3).

**Questions for the checkpoint (T55a4 writes them as an `ADV-Q`):** the tutorial's chests (1 500
gold always): a setup flag for the tutorial (game +0x1F69D also changes its starting resources,
`E08-objects.md` §7.2, and some battle steps) or not modelled (one chest; the agent's
recommendation: not modelled, the Tutorial is a teaching map).

## 10. For T55a4 (the common rules, batch E7, the rule ids)

- **The combination question on a received part** (SoD, `hero_give_artifact` asks the local human
  once per player and combination, survey `E08-army.md` §5.3): every reward artifact meets it; the
  engine's `Manage::Assemble` is a command, the question is not built.
- **What another player sees:** a placed artifact's picture names it (public appearance); a chest's
  contents and a scroll's spell (one picture for every scroll) are hidden until taken; a guarded
  object's guards are hidden, and its question names their creatures and quantity words to the
  visitor only.
- **Batch E7's rows from T55a1:** D-29, D-30, D-31; the chest's experience with Learning (a
  truncation, the Q-0005 class); the chest's artifact with a full backpack (the 1 000 gold choice);
  a monster's artifact with a full backpack (lost).
- **The proposed rule ids** (written when built; the catalog's last: ADV-PIK-004, ADV-MON-010):
  ADV-PIK-005 the treasure chest (§2), ADV-PIK-006 an artifact's pickup and the full backpack
  (§3.2), ADV-PIK-007 a guarded artifact (§3.2, D-29), ADV-PIK-008 a Spell Scroll (§4),
  ADV-OBJ-006 random artifacts at a new game and their pool (§3.1, D-31, D-32), ADV-MON-011 a
  monster's treasure artifact (§5).
- **The import's changes:** classes 5, 65 to 69, 93 and 101 typed; the record's guards and text flag
  as a guarded pile's; the scroll's spell (a byte); the random artifacts and the chests drawn when
  the game is built (setup stream, decision 17); the monster's artifact.
