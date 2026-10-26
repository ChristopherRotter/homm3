# E08 rewards survey (E08 T55a): treasure chests, artifacts and scrolls as pickups, the monsters' treasures, the objects that teach, Pandora's Box, events and the Pyramid

The original's rules for the objects that hand a visiting hero something of the hero's own
(increment 7, E08 decisions 121 and 122), by code reading (`docs/exe/README.md`) against VCMI
1.7.5 and the wiki, with a census of the 160 shipped maps. Cut in four parts (decision 125):
**T55a1** the census and the pickups (§1 to §5, done), **T55a2** the objects that teach
(experience, spells, skills; §6, done), **T55a3** Pandora's Box, events and the Pyramid (§7, done;
decision 127 placed the banks, the Refugee Camp and the quests in later items), **T55a4** the common rules, batch E7's plan, the rule ids, the second reading and the
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
  (`backpackSize` -1); for T55a2 `config/objects/rewardableOncePerHero.json`, `shrine.json`,
  `witchHut.json`, `scholar.json`, `rewardableOnceVisitable.json`, `markets.json` (`university`),
  `lib/mapping/MapFormatH3M.cpp` (`readWitchHut`, `readScholar`, `readShrine`),
  `lib/json/JsonRandom.cpp` (`loadSpell`, `loadSecondary`), `CGHeroInstance.cpp` (`canLearnSkill`,
  `canLearnSpell`); for T55a3 `lib/mapObjects/CGPandoraBox.cpp` (`CGPandoraBox`, `CGEvent`),
  `MapFormatH3M.cpp` (`readBoxContent`, `readEvent`, `readPandora`), `lib/rewardable/Interface.cpp`
  (`grantRewardBeforeLevelup`, `grantRewardAfterLevelup`), `lib/rewardable/Reward.cpp`
  (`calculateManaPoints`), `server/CGameHandler.cpp` (`giveCreatures`, `tryJoiningArmy`),
  `config/objects/pyramid.json`; links `https://github.com/vcmi/vcmi/blob/1.7.5/<file>`.
- `wiki`: read through the summarising fetch tool, `https://heroes.thelazy.net/index.php/<Page>`:
  Treasure_chest, Artifact, Spell_scroll; for T55a2 Arena, Shrine, Learning_stone, Witch_Hut,
  Scholar_(adventure_map), University, Wagon, Warrior's_Tomb; for T55a3 Pandora's_Box, Event, Pyramid
  (the pages found through the search tool; a direct fetch
  of a page not linked from a search is refused in this session). Quotes are short.
- `original-data` (the census): the 160 shipped maps (`original-data/maps.tar`) read with `h3-map`'s
  reader and imported, by a throwaway example (not committed), §1.

## 0. Findings in short (T55a1 to T55a3)

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
- **The objects that teach (T55a2, §6):** ten classes give once per hero, marked in 32 bits per
  class on the hero (**A-17**: a 33rd object shares the first one's mark; at most 30 on a shipped
  map): Marletto Tower, Garden, Mercenary Camp, Star Axis +1 to a skill; the Arena +2 to Attack or
  Defense (no leaving without one); the Library +2 to all four from level 10 (2 less per
  Diplomacy level); the Schools +1 of two for 1 000 gold (or leave); the Learning Stone 1 000
  experience times Learning; the Tree of Knowledge one level, without Learning, at a price drawn at
  a new game (free, 2 000 gold or 10 gems). The Shrines and the Witch Hut teach at every visit what
  they drew at a new game (a Shrine's spell repeats none of its level until the level is used up;
  a hut's skill from the map's mask, all 28 on an RoE map). The Scholar draws at the map's load, falls
  back to +1 primary skill and is removed. The University offers four drawn skills for 2 000 gold
  each to a class that may learn them. The Corpse, Wagon, Warrior's Tomb and Lean To are drawn at
  a new game; a full backpack turns the Corpse's artifact into 1 000 gold and the Wagon's into its
  resources and leaves the Tomb's; the Tomb costs −3 morale once until the hero's next battle ends
  (new hero state).
- **Disputes (T55a2, §9):** D-33 an RoE map's Witch Hut (code: Leadership and Necromancy too; VCMI
  and the wiki: never), D-34 the Shrines' repeats (code: none until the level is used up; VCMI:
  uniform), D-35 the Corpse's and the Wagon's artifact class and the Wagon's amount, D-36 Necromancy
  at a University (code: offered; VCMI and the wiki: never).
- **Pandora's Box, events and the Pyramid (T55a3, §7):** boxes and events share one list of reward
  records (at most 400: **A-19**, unreachable, 99 at most on a shipped map). A box asks "open?"
  first, fights its guards after a warning, gives its rewards and is removed (empty: a message).
  An event fires when a hero of a colour in its mask enters its tile (a computer's only with its
  flag); it **ends the move there**, fights its guards at once (no question), gives its rewards,
  and is removed only with its flag (else it fires at every entry). **The rewards** come in a fixed
  order: primary skills (positive only), secondary skills (**raised to** the level, a new one below
  eight), mana (the total held in 0 to 999), morale and luck (bonuses until the next battle's end),
  resources, artifacts (lost with 64 in the backpack), spells (a book, level ≤ 2 + Wisdom),
  creatures (no room: one army dialog, the rest lost), and **experience last**, computed with the
  Learning before the box's skills. **The Pyramid** draws a level-5 spell at a new game (Titan's
  Lightning Bolt never: always banned). It asks at every visit and fights 40 Gold Golems (split
  into 1 to 5 stacks like a wandering monster's) and two stacks of 10 Diamond Golems. The spell is
  lost without a book or Expert Wisdom. A looted Pyramid costs −2 luck once until the next battle's
  end (**A-20**: an empty pool, unreachable).
- **Disputes (T55a3, §9):** D-37 a known skill (code: raised to the level; VCMI: the level added),
  D-38 mana above the maximum (code: up to 999; VCMI: up to the maximum), D-39 the order
  (experience last with the old Learning; VCMI: before the rest, with the new Learning), D-40 the
  Pyramid's Gold Golems (code: 1 to 5 stacks; VCMI: one).

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
| 6 | Pandora's Box | 46 | 20 | 0 | T55a3, §7 |
| 26 | Event | 486 | 68 | 48 | T55a3, §7 |
| 78 | Refugee Camp | 321 | 101 | 5 | not in increment 7: the sketch's item 9 (decision 127) |
| 16 | Creature Bank (subclasses 0–6) | 1 215 | 62–106 | 19 | not in increment 7: item 10 (decision 127) |
| 24 / 25 / 84 / 85 | Derelict Ship / Dragon Utopia / Crypt / Shipwreck | 170 / 74 / 277 / 401 | 58 / 55 / 97 / 69 | 2 / 1 / 4 / 10 | not in increment 7: item 10 (decision 127) |
| 63 | Pyramid | 60 | 41 | 2 | T55a3, §7 |
| 83 | Seer's Hut (subclasses 0–2) | 265 | 41–56 | 0 | not in increment 7: item 11, after E09 and E10 (decision 127) |
| 215 | Quest Guard | 114 | 31 | 0 | not in increment 7: item 11 (decision 127) |
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

The bodies and counts, for T55a2 (the same throwaway example):

- **No class above 32 on a map:** the once-per-hero classes at most 30 (Learning Stones, Back For
  Revenge), Scholars 31 (Battle of the Sexes), Shrines of level 1 29 (Race for Ardintinny).
- **Shrines (88 to 90):** 1 215 random, 28 with a spell (each of its shrine's level, none banned);
  no map bans every spell of a level.
- **Witch Huts (113):** 191 on RoE maps (no mask); 581 masks on AB and SoD maps, 16 of them
  allowing Leadership, none Necromancy, none empty or wholly banned.
- **Scholars (81):** 609 random, 43 primary skills, 34 secondary skills, 8 spells (levels 1, 1, 3,
  3, 4, 5, 5, 5); no placed id of 255.
- **Universities (104):** no map leaves fewer than four skills unbanned.

The bodies and counts, for T55a3 (a throwaway example of the same kind, deleted):

- **Pandora's Boxes (6):** 46 on 20 maps (35 SoD, 6 RoE, 5 AB), none on the five whole maps; 15
  message records, 11 with a text, 12 guarded (4 without a text). Every box gives something: 39 one
  kind, 3 two, 4 three. Creatures 20 (21 stacks, at most 2 in a box), experience 14 (at most
  25 000), spells 6 (104 spells, up to 19 in one box: levels 1 to 5 18, 42, 16, 20, 8; one box gives
  a spell its map bans), resources 6 (one takes), artifacts 5 (10, at most 4), morale 3 (all
  positive), primary skills 2 (+1), a secondary skill 1 (Expert).
- **Events (26):** 486 on 68 maps (201 RoE, 182 AB, 103 SoD); 48 on the whole maps (Faeries 22,
  Race for Ardintinny 12, Adventures of Jared Haret 11, the Tutorial 2, Knee Deep in the Dead 1:
  texts, morale and luck from −3 to +2, creatures, two artifacts, a skill, a primary skill, a
  loss of 10 gems and 1 000 gold; 3 guarded, all on Faeries). 482 message records, 470 with a text; 87
  guarded (12 without a text). **231 give nothing** (180 a text only, 35 a text and guards, 12
  guards only, 4 nothing at all: they only stop the hero); 222 give one kind, 33 more. Morale 76 (43
  negative), resources 73 (26 take), artifacts 47 (53, at most 7), creatures 40 (63 stacks, up to
  7), luck 29 (9 negative), experience 23 (at most 7 000), primary skills 7 (at most +5), skills 7
  (8 pairs, levels 1 to 3), mana 4 (one negative, at most 60), spells 2. Colours: 389 for all, 97
  for some; 60 for computers too; 460 removed after the visit, 26 repeat (none guarded; 10 give
  something every time: morale, luck or a loss of resources, as Shadow Valleys' four).
- **Out of range: none.** No morale or luck beyond ±3, no primary skill above 127, no experience of
  2³¹ or more, no random creature code, no count of 0, no Spell Book, Scroll, Grail or war machine
  among the artifacts. **The reward records per map:** at most 99 (Myth and Legend), then 79
  (Pandora's Box), against the cap of 400 (§7.1).
- **Pyramids (63):** 60 on 41 maps (at most 4: Land of Titans), 2 on the whole maps; four maps ban
  level-5 spells: Good Witch, Bad Witch three (Fly, Dimension Door, Summon Water Elemental),
  Gorlam's Tentacle Swampland and Island King with its allied version two (Fly, Dimension Door).

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

## 6. The objects that teach (T55a2)

The cut (decision 126): the 20 classes of the queue's list, the University's dialog read for its
rules only. `original-data` (read) follows the human player's path; the computer player's branches
are named, not read (E10): the Arena's pick (read, it is short), `ai_fight_answer` and
`ai_magic_school_choice` (0x527CD0) at the School of Magic, `ai_gold_or_experience` (0x527FE0) at
the School of War and the Tree of Knowledge, `ai_artifact_received` (0x433B40) after an artifact,
`ai_university` (0x524ED0). Every handler is called from `visit_object`'s case for its class
(`map.toml`, 0x4A84F0); the University's case is inline (0x4AA4DE). The draws named "at a new
game" are `setup_map_objects`' cases (0x4C0CC0, its tile walk: level, row, column).

| Class | Gives | Asks or costs | Refuses | How often |
|---|---|---|---|---|
| 23 Marletto Tower, 32 Garden of Revelation, 51 Mercenary Camp, 61 Star Axis | +1 Defense, Knowledge, Attack, Spell Power | — | — | once per hero |
| 4 Arena | +2 Attack or Defense | a choice | — | once per hero |
| 41 Library of Enlightenment | +2 to all four | — | level + 2 × Diplomacy below 10 | once per hero |
| 47 School of Magic, 107 School of War | +1 Spell Power or Knowledge, +1 Attack or Defense | 1 000 gold, a choice or leave | less gold | once per hero |
| 100 Learning Stone | 1 000 experience × Learning | — | — | once per hero |
| 102 Tree of Knowledge | one level | drawn: free, 2 000 gold or 10 gems; a question | too little | once per hero |
| 88, 89, 90 Shrines | a spell of level 1, 2, 3 (drawn) | — | known, no Spell Book, Wisdom too low | every visit |
| 113 Witch Hut | a secondary skill at Basic (drawn) | — | known, eight skills | every visit |
| 81 Scholar | a spell, a skill level or +1 primary | — | (falls back to the primary skill) | once (removed) |
| 104 University | up to four skills at Basic (drawn) | 2 000 gold each | known, the class's weight 0, eight skills, gold | every visit |
| 22 Corpse | 20 %: a treasure or minor artifact | — | (full backpack: 1 000 gold) | once (emptied) |
| 105 Wagon | an artifact (40 %) or 2 to 5 of a resource (50 %) | — | (full backpack: the resources) | once (emptied) |
| 108 Warrior's Tomb | an artifact of any class; −3 morale | a question | (full backpack: nothing, the artifact stays) | the artifact once |
| 39 Lean To | 1 to 5 of a resource | — | — | once (emptied) |

### 6.1 The marks of a visit

- **Once per hero.** At a new game each of the classes 4, 23, 32, 41, 47, 51, 61, 100, 102 and 107
  numbers its objects from 0 in the walk's order (counters set to 0 at 0x4C0CF6) and keeps the
  number in the tile word. The hero holds one u32 per class (hero +0x57 Learning Stones, +0x5B
  Marletto, +0x5F Garden, +0x63 Mercenary Camp, +0x67 Star Axis, +0x6B Tree, +0x6F Library, +0x73
  Arena, +0x77 School of Magic, +0x7B School of War; H3API's per-hero bitfields); a visit tests
  and sets the bit `1 << number`. The x86 shift counts mod 32 (the Tree masks its number to 5
  bits outright, 0x4C1E6D), so **the 33rd object of a class shares the first one's bit**: a hero
  who visited one is refused by the other (**A-17**). The census (§1) has at most 30 of a class on
  a map (30 Learning Stones in Back For Revenge); the wiki (Learning Stone): "There can be up to
  32 learning stones in any map", which reads as the editor's limit.
- **The team's marks** (presentation and knowledge, no rule reads them here): most handlers set,
  for every player of the visitor's team (`player_team` 0x4A5960), a bit in a byte per object kind
  (game +0x4E344, `kinds_visited`); the Tree, the Shrines, the Witch Hut, the Wagon and the Tomb
  also set the tile word's bits 5 to 12 (`tile_mark_visited_by_team` 0x4FBF90); the Corpse and
  the Lean To set a bit of the current player's record (+0xC0, +0xC4). Several handlers set them
  before a refusal (the Schools before the gold test, the Tree before its price, the Shrines and
  the Witch Hut before any test), so a refused visit still counts as seen. They feed what a player
  knows of the object (its right-click), §10.

### 6.2 Primary skills: Marletto Tower, Garden, Mercenary Camp, Star Axis, Arena, Library

`original-data` (read): **Marletto Tower** (`visit_marletto_tower` 0x4A2050): visited, a message;
else +1 Defense (hero +0x477), the marks. The **Garden of Revelation** (0x4A2710, +1 Knowledge),
**Mercenary Camp** (0x4A38B0, +1 Attack) and **Star Axis** (0x4A3DC0, +1 Spell Power) are the same
code with their fields. **Arena** (`visit_arena` 0x49E7D0, its marks 0x4E53C0 and 0x4E53E0):
visited, a message; else a human picks in a two-picture box (window result 0x7809 Attack, 0x780A
Defense, anything else Attack, also the box's Cancel: style 10 builds one, 0x7801 at 0x4F6E5B, so
there is no way to leave without a bonus), +2 to it. The computer takes the lower of the two
clamped to 0 to 99 (Attack on a tie). **Library of Enlightenment** (0x4A3280): visited, a message;
level (+0x55) + 2 × Diplomacy (+0xCD) below 10 refuses with a message and no mark (level 10;
8, 6, 4 with Basic, Advanced, Expert Diplomacy); else +2 to all four. Every increase is a byte's
`inc` or `add`, unclamped: past 127 a skill wraps (the level-up's **A-9**; the readers clamp to 0
to 99).

`vcmi` (`config/objects/rewardableOncePerHero.json`): the same gifts, the Arena's choice
(`selectPlayer`, no refusal), the Library's limiter (level 10, or 8, 6, 4 with Diplomacy 1, 2, 3).
`wiki` (Arena): "It is not possible to leave the arena without accepting one of these bonuses"; the
others not read (VCMI agrees). **Consensus.** VCMI keeps a set of visitors per object, so
A-17 is the original's alone.

### 6.3 The Schools of Magic and War

`original-data` (read): **School of Magic** (`visit_school_of_magic` 0x4A33E0): visited, a message;
else the team's kind mark, then the current player's gold (record +0xB4) below 1 000 refuses with a
message and no hero mark. A human picks Spell Power or Knowledge or leaves (window result 0x7801:
nothing paid, no mark; 0x7809 Power, 0x780A Knowledge, anything else Power); then +1, the hero's
mark, −1 000 gold. **School of War** (0x4A7A40): the same with Attack or Defense (anything else
Attack). The computer: at the School of War `ai_gold_or_experience` weighs the 1 000 gold against
the hero's experience to the next level and declines or takes the lower skill as at the Arena; at
the School of Magic `ai_fight_answer` then 0x527CD0 (not read).

`vcmi`: `schoolOfMagic`, `schoolOfWar`: two rewards each with the gold limiter and −1 000 gold,
`canRefuse`. **Consensus.**

### 6.4 Experience: the Learning Stone and the Tree of Knowledge

`original-data` (read): **Learning Stone** (`visit_learning_stone` 0x4A6330): visited, a message;
else x = trunc(`hero_learning` × 1 000.0f) (the chest's computation, §2.2: 1 000 before Learning),
shown, `hero_gain_experience(x, no levels, tell)`, the marks, then `hero_level_up`.
**Tree of Knowledge** (0x4A6710): at a new game its price is drawn, random_between(0, 2) in the
word's bits 13 to 15: free, 2 000 gold or 10 gems (record +0xB0). The visit: visited, a message;
else the marks, then too little gold or gems refuses (a message, no hero mark); with a price a human
is asked (the answer 0x7806 declines, no hero mark); the gift is `experience_to_next_level(level)`
(0x4DA420: the gap between the hero's level's threshold and the next's, the level-up's table and
its × 1.2 steps), so exactly one level, **without Learning**; then the hero's mark and
`hero_level_up`.

`vcmi`: `learningStone` gives `heroExperience` 1 000, which VCMI's rewards scale by the hero's
Learning (`calculateXp`, §2.3); `treeOfKnowledge`: `heroLevel` 1 at a price drawn by the map's dice
(below 34 free, 34 to 66 2 000 gold, 67 on 10 gems: 34/33/33 % against the code's thirds),
`canRefuse`. `wiki` (Learning stone): "1000 experience points the first time they visit it",
silent on Learning. **Consensus** (the Tree's third is a rounding of VCMI's dice, not a rule).

### 6.5 Spells: the Shrines

`original-data` (read): the body is the spell (a u32; 255 random, `h3-map`'s `Body::Shrine`). At a
new game a spell of 0x3FF (the word's bits 13 to 22; the cases 0x4C1CA8, 0x4C1CF5, 0x4C1D59) is
drawn by `random_shrine_spell` (0x4C95A0) among the spells 0 to 69 of the shrine's level that have
a school and carry no shrine mark (game +0x4, `shrine_spell_marks`), uniformly; the drawn spell
gets the mark. The marks start as a copy of the game's spell bans (game +0x4A; `new_game_bans`
0x4C2965). When a level's pool is used up, the marks of its unbanned spells are cleared and the
draw starts again. So **the random Shrines of a level repeat no spell until each of the level's
spells is out** (SoD: 15 spells at each of the levels 1 to 3). The visit (`visit_shrine` 0x4A5610):
the marks first, then a spell the hero knows (+0x3EA), no Spell Book (`hero_wears_artifact(0)`) or
a level above Wisdom (+0xD0) + 2 refuses with its message; else the hero learns it
(`hero_learn_spell`). No mark of the hero's own: a refused hero may come back. A spell of −1 (a
draw that found nothing) is not tested and would read outside the tables (**A-18**).

`vcmi` (`shrine.json`, `JsonRandom::loadSpell`): the map's spell, else a random allowed, not special
spell of the level, **uniform with repeats**; the limiter `canLearnSpells` (a book, the level at
most `maxSpellLevel`, not known, not banned). `wiki` (Shrine): level 1, 2, 3, Basic Wisdom for the
third, the three refusals' texts; silent on the draw. **D-34** (the repeats). The census: 1 215
random shrines, 28 placed with a spell (each of its shrine's level, none banned); no map bans a
whole level.

### 6.6 A secondary skill: the Witch Hut

`original-data` (read): the body is a mask of allowed skills (AB and SoD; none in RoE). At a new
game (case 0x4C207D) the mask (an empty one: all 28 skills) less the banned skills (game +0x4E658)
gives n candidates; the hut's skill is the r-th of them in id order, r = random_between(1, n), none
−1 (`witch_hut_set_skill` 0x4C23C0, bits 13 to 19). The visit (`visit_witch_hut` 0x4A8080): the
marks, then −1, a known skill or eight skills (+0x101) a message; else
`hero_learn_secondary_skill(skill, 1)`: Basic. No hero mark and no class test.

`vcmi` (`witchHut.json`, `CMapLoaderH3M::readWitchHut`): an AB or SoD map's mask is used; without
one (RoE) the default **excludes Leadership and Necromancy**; the limiters agree (a free slot, not
known). `wiki` (Witch Hut): the skill "cannot be Leadership, Necromancy" (the editor's default; the
map may allow them), Basic, a skill "only ... if the hero has not already learned it"; RoE maps not
addressed. The census: 191 huts on RoE maps (no mask: Leadership and Necromancy possible in the
code); of the 581 masks of AB and SoD maps, 16 allow Leadership and none Necromancy. **D-33.**

### 6.7 The Scholar

`original-data` (read): the body (`load_scholar_body` 0x500B30) is drawn **when the map is loaded**:
a kind of 0xFF becomes random_between(0, 2) (0 a primary skill, 1 a secondary skill, 2 a spell) and
its id is drawn: a spell uniformly among the spells 0 to 69 with a school that are not banned (any
level, no marks), a skill among the 28 not banned, a primary skill random_between(0, 3); a placed
kind takes the map's byte. At a new game a Scholar of kind 1 or 2 also gets a primary skill
random_between(0, 3) (case 0x4C1B29), its fallback. The visit (`visit_scholar` 0x4A4DC0): a spell
is learned when not known, of a level at most Wisdom + 2 and with a Spell Book; a skill is taken by
`hero_learn_secondary_skill(skill, 1)` (a new one at Basic below eight skills, a known one one level
up below Expert; no class test); when it cannot, and for kind 0, **+1 to the fallback primary
skill**. Then the Scholar is removed (`remove_map_object`).

`vcmi` (`scholar.json`, `readScholar`): the same three rewards in that order with the same limiters,
a primary skill as the last resort, `removeObject`; a random kind by the map's dice (33/33/34 %).
`wiki` (Scholar): "+1 to one of the primary skills, or one level of secondary skill, or a spell"; a
known skill rises "to the next level"; a spell it cannot learn or an expert skill gives a primary
skill instead; it teaches class-restricted skills; "disappears after that". **Consensus.** The
census: 609 random, 43 primary, 34 secondary, 8 spells (two of level 1, two of 3, one of 4, three of
5).

### 6.8 The University

`original-data` (read for its rules): at a new game (`university_setup` 0x4C06F0) four distinct
skills are drawn, each random_between(0, n − 1) among the unbanned skills left; they go to the
game's universities (game +0x4E3CC). A human's visit opens the dialog (0x5EF500; the computer
`ai_university`); the team's kind mark is set after it. Each of the four slots
(`university_slot_state` 0x5F0790) is disabled when the hero knows the skill, the hero's class
weighs it 0 (hero class +0x18, the level-up's weights: Necromancy for every class but Death Knight
and Necromancer, and e.g. Leadership for those two, Fire Magic for Rangers and Beastmasters) or the
hero has eight skills; a click (0x5F0AC0) gives the reason, else the purchase box, whose button
works only with 2 000 gold or more (0x5F0D18); buying (0x5F0F60) takes 2 000 gold from the owner
and teaches the skill at Basic, and the slots are re-evaluated, so several skills may be bought in
one visit. **Necromancy is among the skills drawn.**

`vcmi` (`markets.json` `university`, `CGHeroInstance::canLearnSkill`): four offers, each **"noneOf
necromancy"**; buying needs a free slot, an allowed and unknown skill and the class's probability
above 0. `wiki` (University): "learn up to four secondary skills to basic level for 2000" each,
"a disabled skill will not appear there", "**Necromancy can not appear in universities**", the
class restrictions ("Barbarians cannot learn Water magic"). **D-36** (the reading against both).

### 6.9 The finds: Corpse, Wagon, Warrior's Tomb and Lean To

`original-data` (read):

- **Corpse** (case 0x4C1131; `visit_corpse` 0x4A5480): random_between(0, 99) below 20 holds
  `random_artifact_of_level(6)` (treasure and minor drawn as one pool, so weighted by the classes'
  sizes: 37 and 21 records in the packs, about 64 % treasure). The visit: with the artifact and a
  full backpack (≥ 64) **1 000 gold** to the owner instead; else the artifact to the hero; emptied
  either way; without one a message.
- **Wagon** (0x4C1EA2; `visit_wagon` 0x4A69B0): every wagon draws a resource random_between(0, 5)
  (wood to gems, no gold) and an amount random_between(2, 5); then r = random_between(0, 99): below
  10 empty, below 50 `random_artifact_of_level(6)`, else the resources. The visit: an artifact to
  the hero, with a full backpack **the drawn resources** instead; resources to the owner; emptied.
- **Warrior's Tomb** (0x4C1F1C; `visit_warriors_tomb` 0x4A7C30): r below 30 a treasure artifact, 80
  minor, 95 major, else a relic. A human is asked first; no answer leaves (nothing, no penalty).
  With the artifact and room it goes to the hero; empty, or with a full backpack (**the artifact
  stays**), a message. Then, whether or not anything was found, the hero's morale bonus (+0x11A)
  takes −3 **once until the end of its next battle** (status flag bit 21; `hero_battle_bonuses_clear`
  0x4DA510, called by `battle_end`, zeroes the morale and luck bonuses and the flag).
- **Lean To** (0x4C157A; `visit_lean_to` 0x4A31A0): a resource random_between(0, 5) and an amount
  random_between(1, 5); the visit gives them to the owner and empties it.

`vcmi` (`rewardableOnceVisitable.json`): Corpse 10 % treasure, 10 % minor, 80 % nothing; Wagon 20 %
treasure, 20 % minor, 50 % **1 to 5** of a resource (no gold), 10 % nothing; Tomb 30/50/15/5 % by
class, −3 morale for one battle at every visit, also at a looted tomb (whether two stack is VCMI's
bonus system, not read); Lean To 1 to 5 of a resource (no gold); no full-backpack cases (VCMI's
backpack is unbounded, §3.4). `wiki`: Wagon 10 % nothing, 20 % treasure, 20 % minor, 50 % "2–5"
resources "(except gold)"; Warrior's Tomb 30/50/15/5 %, "Do you wish to search the tomb?", "-3 to
morale until the next battle", a searched tomb gives the penalty only; the Corpse's page was not
returned by a search. **D-35** (the artifact's class split, the Wagon's amount); the Tomb agrees;
the full-backpack cases are the code's alone (rows for batch E7, §10).

## 7. Pandora's Box, events and the Pyramid (T55a3)

Decision 127 cut T55a3 to these three classes: the creature banks (16, 24, 25, 84, 85) are the
sketch's item 10, the Refugee Camp (78) item 9, Seer's Huts and quest guards (83, 215) item 11. The
two other callers of the army dialog that decision 121 named are the banks' reward (0x4ABDC0) and
the Seer's Hut's (0x573C80), so they go with those items. Pandora's Box and events give creatures
through their own reward routine (§7.4). The human player's path is read. The computer's answers
(`ai_fight_answer` 0x52BD10, `ai_join_without_room` 0x52BC60, `ai_artifact_received` 0x433B40) are
named, not read (E10). The texts are ADVEVENT.TXT's lines, which the code reads through
`advevent_text_pointer` (0x696A18; line n at +4n of its +0x20 array).

### 7.1 The reward record

`original-data` (read): `load_object_body` reads a Pandora's Box (0x4FFBF0) and an event (0x5008B0)
through one reader of the format's message, guards and rewards (`load_rewards` 0x4FF6B0, skimmed).
Both append a 228-byte **reward record** to **one list** of the map (+0x50; begin +0x54, end
+0x58). The record's index goes into bits 0–9 of the tile's word (`reward_record` 0x4A0C20). An
event's word also holds the format's player byte in bits 10–17, the computer flag in bit 18 and
"remove after the visit" in bit 19. The record holds the text (a string at +0x0), the guards flag
(+0x10), the guards (+0x14) and the rewards (`structs.toml`, `reward_record`).

**At most 400 records:** a 401st box or event is not stored, and its word keeps index 0, so it reads
the first record (**A-19**). The census's largest list is 99 (Myth and Legend: events and boxes
together), so no shipped map reaches the cap.

### 7.2 Pandora's Box (6)

`original-data` (read): `visit_pandora_box` (0x4A0C50, `visit_object` case 6):

1. **A human** sees the map's text, when it has one, in an OK box. Then comes the question "Do you
   wish to open the box?" (ADVEVENT 14). Answering no does nothing, and the box stays. The computer
   asks `ai_fight_answer` instead.
2. **Guards** (the flag, and at least one stack): a human sees "you are attacked" (ADVEVENT 16),
   then the battle against the record's army as the map gives it. The call is the guarded pile's
   (`adventure_battle` 0x4AD470: no split, a stored seed; survey `E08-battles.md` §5.1).
   - **Lost or retreated:** return. The box stays with the guards' survivors (the record's army,
     passed by address).
   - **Won:** the levels from the battle (0x4DA720).
3. **The rewards** (`grant_rewards` 0x49FA90, §7.4), without the map's text, which was already
   shown. When nothing was given, a human sees "the box is empty" (ADVEVENT 15).
4. **The box is removed** (`remove_map_object`, with a pickup sound).

At a new game, `setup_map_objects` case 6 only fills the word (9 instructions).

### 7.3 Events (26)

`original-data` (read): an event is not a case of `visit_object`, and `place_object_tiles` gives it
no drawing, only its tile's class (§10).

**When it fires.** `hero_move_step` tests the tile a step enters (`event_fires_for` 0x4FCDC0, at
0x4811A3):

- a **human** player (`player_is_human_both` 0x4BAD80: both of the player record's human flags,
  +0xE1 and +0xE2) whose colour bit is in bits 10–17;
- a **computer** player only with bit 18 set and its colour in the mask.

When it fires, **the move ends on the event's tile**. The step sets its stop flag, as when the
movement points run out, and the global `move_interrupted` (0x69777C, at 0x4816B6), which ends the
move's loop (0x407F15). After the step, `event_fire` (0x4A0ED0, called at 0x481741) handles the
event. An event of another colour does not stop the hero.

**The fire** (`event_fire`):

1. **Guards:** a human sees the event's text, or when the text is empty "you are attacked"
   (ADVEVENT 16). There is **no question**: the battle starts at once, the same call as Pandora's.
   - **Lost:** return. The event stays with the survivors.
   - **Won:** the levels from the battle.
2. **The rewards** (`grant_rewards`, §7.4) with the event's text. If the guards already showed
   that text, the rewards get an empty one instead.
3. **Removal:** with bit 19 the tile's object is cleared: its word, its class (+0x1E) and bit 0x10
   of the tile's +0xD (not identified). Without bit 19 the event stays and **fires again** at every
   later entry by a hero of a qualifying colour.
4. **The end-of-game tests:** `scenario_condition_test` (0x5F1B10) and the second test (0x5F1C80)
   run on the game's victory condition (+0x1F89C). When either holds, `scenario_condition_met`
   (0x4F2CE0) is called. These
   are the same tests a battle and a turn run (not read, E10).

### 7.4 The rewards (0x49FA90)

`original-data` (read): `grant_rewards` (the text, the hero, the tile, the position, the human
flag, the record) applies the parts **in this order**:

| # | Part (record offset) | The rule |
|---|---|---|
| 1 | experience (+0x50, signed) | Only when above 0. The amount is computed **first**: amount × `hero_learning` (f32, truncated). The Learning level used is the one before this record's skills. The amount is **given last** (step 10). |
| 2 | primary skills (+0x78, 4 signed bytes) | Each value **above 0** is added to hero +0x476 + i at once (unclamped; the level-up's A-9 covers the class). 0 or a negative value is skipped. |
| 3 | secondary skills (+0x80, (skill, level) pairs) | A known skill below the given level **rises to it** (`hero_learn_secondary_skill` with the difference, at most Expert). A known skill at or above the level is unchanged. A new skill is learned at the level only below eight skills. There is no class or ban test. |
| 4 | mana (+0x54, signed) | Added. The total is held **between 0 and 999**: it may exceed the hero's maximum, and a loss stops at 0. |
| 5 | morale (+0x58), luck (+0x59) | Added to the **bonuses that last until the end of the hero's next battle** (+0x11A, +0x11B; cleared by `hero_battle_bonuses_clear`). They add up across objects. |
| 6 | resources (+0x5C, 7 signed) | `give_resource` per resource: 32-bit; a negative result becomes 0. |
| 7 | artifacts (+0x90) | Each is given only while the backpack holds **fewer than 64** (`hero_give_artifact`: the first body position that takes it, else the backpack, with SoD's combination question). Otherwise it is **lost**. |
| 8 | spells (+0xA0) | **Only with a Spell Book**. Each is learned (`hero_learn_spell`) when its level is at most **2 + Wisdom** and it is not known; there is no ban test (one shipped box gives a spell its map bans). Otherwise it is skipped without a message. Step 3's Wisdom and step 7's Spell Book count. |
| 9 | creatures (+0xAC, an army of 7 slots) | Slot by slot: `army_add_stack` (onto the slot of the same creature, else the first empty slot). For a human, what finds **no room** goes into **one army dialog** after the last box (`army_dialog_with` 0x5D12B0, flag 0: the "make room" dialog of `army_full_dialog`, survey `E08-army.md` §1.1). What is left in that dialog is lost. The computer: `ai_join_without_room`. |
| 10 | experience | Step 1's amount by `hero_gain_experience` (the levels, and the scenario's level limit with its message). |

It returns whether anything was given. Every non-zero part counts, and so does every creature slot
even when it found no room; a skipped artifact or spell does not. A human sees the map's text and
the parts as pictures, in boxes of up to eight: steps 1 to 5 first, then the resources, the
artifacts, the spells and the creatures. Each box after the first gets ADVEVENT 175 to 188 ("%s
learns something.", "%s has found treasure.", …) when the map's text is empty.

### 7.5 The Pyramid (63)

`original-data` (read):

**At a new game** (`setup_map_objects` case 63, 0x4C19DE): the pool is every spell 0 to 69 of
**level 5 with a school** that the game does not ban. The game's bans (+0x4A, `new_game_bans`) are
the map's, plus every spell with flag 0x2000. Only Titan's Lightning Bolt (57) has that flag, so it
never enters the pool. The pool is Fly, Dimension Door, Implosion, Magic Mirror, Sacrifice and the
four Summon Elementals, less the map's bans. One spell is drawn uniformly: random_between(0,
n − 1). `pyramid_word_set` (0x4C2330) sets the word's bit 0 (guarded) and the spell in bits 13–20;
bits 5–12 (the team marks) are cleared. An empty pool would read outside the list (**A-20**). The
census found at most 3 of the 9 banned (Good Witch, Bad Witch: Fly, Dimension Door, Water
Elemental), so no shipped map has an empty pool.

**The visit** (`visit_pyramid` 0x4A4230, `visit_object` case 63):

1. **A human** is asked first, at every visit (ADVEVENT 105: search it?). Answering no does
   nothing. The computer asks `ai_fight_answer`. Then the team's mark is set
   (`tile_mark_visited_by_team`).
2. **Looted** (bit 0 clear): a human sees ADVEVENT 107. The hero takes **−2 luck** (+0x11B)
   **once until the end of its next battle**: status flag bit 12, which `hero_battle_bonuses_clear`
   clears. This applies to the computer's heroes too.
3. **Guarded:** `monster_battle` (0x4AC580, at 0x4A434A) against **40 Gold Golems** (116) and **20
   Diamond Golems** (117) in 2 stacks, with no third creature:
   - The battle's seed comes from the tile, as for a wandering monster's.
   - The monster's strength sums both creatures' values. The number of stacks n comes from the army
     ratio and the roll (survey `E08-battles.md` §4.1, 2 to 7, then ±1), **less the 2 Diamond
     stacks**, and is at least 1. So the Gold Golems split evenly into **1 to 5 stacks** and the
     Diamond Golems into 2 of 10. No stack is upgraded.
   - The slots are ordered by the table 0x63DF94 (`monster_stack_order`, row 7n). Slot by slot, with
     G a Gold stack, D a Diamond stack and – an empty slot: n = 1 D D – G – – –; 2 D D G – G – –;
     3 D G D G – G –; 4 G D G D G – G; 5 G G D G D G G.
   - **Lost or retreated:** nothing changes. The counts are local, so the next fight is again
     against 40 and 20.
4. **Won:** the battle's levels. Bit 0 is cleared (looted) and the spell stays in the word. Then:
   - no Spell Book: the spell is **lost** (ADVEVENT 109);
   - level 5 above 2 + Wisdom (below Expert Wisdom): **lost** (108);
   - otherwise `hero_learn_spell`. There is no "known" test, and relearning a known spell changes
     nothing.

### 7.6 VCMI and the wiki

- **Pandora's Box.** `vcmi` (`CGPandoraBox`, `readBoxContent`): the question (ADVOB 14), guards
  with text 16 and the battle, an empty box 15, removed after the rewards. The map's text is not
  shown before the question (presentation). `wiki` (Pandora's Box): "Do you wish to open the box?",
  "combat ensues if there are guardians", the reward list (spells "the proper level of Wisdom skill
  is required", secondary skills "ignoring hero class restrictions"), "The box disappears after a
  hero successfully opens it". **Consensus** on the visit.
- **Events.** `vcmi` (`CGEvent`): fires for `availableFor` and `computerActivate`. The human flag
  is HotA's (`levelHOTA3`), and SoD reads it as always set, as the code does. With guards it shows
  the message (or 16) and fights at once. `removeAfterVisit`; a draw removes a non-repeatable
  event. `wiki` (Event): "triggered when a hero enters the square", "can be canceled after a hero
  first time triggers it, or it can be triggered unlimited times", guardians "will not be reset",
  skills "even if it is restricted from his hero class", spells need "a corresponding level of
  wisdom and a spell book". Silent on whether the hero stops. **Consensus.**
- **The rewards** (`Rewardable::Interface`; the wiki silent beyond the above):
  - **Skills:** VCMI **adds** the given level to the known one (Basic + Advanced = Expert); the
    code raises to it (Advanced). **D-37.**
  - **Mana:** VCMI gives mana up to the hero's **maximum** (`manaOverflowFactor` 0) and loses the
    rest; the code allows up to 999. **D-38.**
  - **Order:** VCMI gives the experience (computed after the box's skills, so with a new Learning)
    and its levels **before** the mana, morale, luck, artifacts, spells and creatures; the code
    computes it with the old Learning and gives it **last**. So a spell that needs the Wisdom of a
    level-up is refused in the code, and the Learning differs. **D-39.**
  - **Agree:** the spells' limits (`canLearnSpell`: a book, Wisdom, not known, bans allowed), the
    morale and luck for one battle, creatures without room through the garrison dialog. VCMI's
    unbounded backpack never loses an artifact (§3.4).
  - **Primary skills:** VCMI adds them as unsigned bytes; no shipped value is above 5.
- **The Pyramid.** `vcmi` (`pyramid.json`): a level-5 spell, question 105, guards **40 Gold Golems
  in one stack** and two stacks of 10 Diamond Golems (`guardsLayout` default). The spell is learned
  only by a hero who can learn it (108, 109 otherwise, lost). Looted: 107 and −2 luck for one
  battle at each visit (`onVisited`; whether two such bonuses add was not traced, as the Tomb's,
  §6.9). `wiki` (Pyramid): "guarded by 40 Gold Golems and two stacks of 10 Diamond Golems", "expert
  Wisdom and a spell book", "it is lost forever", "Searching an empty pyramid decreases hero's luck
  by 2", golems restored after a retreat. **D-40** (the Gold Golems' stacks); the rest agrees.

## 9. Disputes and quirk candidates so far (for T55a4)

| Id | What | Sources | Proposal |
|---|---|---|---|
| D-29 | a guarded artifact without a text | code (0x49F070) and VCMI: a question naming the guards; wiki: "no question is asked and the guards attack immediately" | the code's (the question); a batch E7 row |
| D-30 | where a reward's artifact goes | code: `hero_give_artifact`, the first body position that takes it, else the backpack (HER-CHG-002, implemented, not yet verified); wiki: a chest's artifact "is put in the hero's backpack", "Scrolls go into the backpack" | the code's; a batch E7 row (a scroll or an artifact picked up with its position free) |
| D-31 | the classes of a random artifact (65) | code: treasure, minor or major (0xE); VCMI: any legal class, relics too; wiki silent | the code's; a batch E7 row (many class 65 objects, no relic among them) |
| D-32 | what a random artifact may repeat | code: never a banned, placed or already drawn one until the pool is exhausted; VCMI: the least rolled, placed ones not counted; wiki silent | the code's; no row (rare: a pool exhausted) |
| D-33 | an RoE map's Witch Hut (no mask) | code (0x4C207D): any of the 28 skills not banned, Leadership and Necromancy among them (191 huts); VCMI: never Leadership or Necromancy; wiki: "cannot be Leadership, Necromancy" (the editor's default), RoE not addressed | the reading against both: the user's, after a batch E7 row (an RoE map with many huts) |
| D-34 | what the random Shrines repeat | code (0x4C95A0): no spell of the level until each is out (the bans never); VCMI: uniform, repeats; wiki silent | the code's; a batch E7 row (15 random Shrines of one level: 15 spells) |
| D-35 | the Corpse's and the Wagon's artifact; the Wagon's amount | code: treasure and minor as one pool (by the classes' sizes, about 64 % treasure), the Corpse 20 %, the Wagon 40 %; the amount 2 to 5; VCMI and the wiki (Wagon): half each class (10 + 10 %, 20 + 20 %); the amount: wiki 2 to 5, VCMI 1 to 5 | the code's (the amount with the wiki); no row (a distribution) |
| D-36 | Necromancy at a University | code (0x4C06F0): drawn like any unbanned skill, bought only by a class that weighs it (Death Knight, Necromancer); VCMI ("noneOf necromancy") and the wiki ("Necromancy can not appear in universities"): never offered | the reading against both: the user's, after a batch E7 row (many Universities) |
| D-37 | a box's or event's secondary skill the hero knows | code (0x49FA90): raised to the given level, never lowered (Basic + Advanced: Advanced); VCMI (`grantRewardBeforeLevelup`): the given level added, clamped (Expert); wiki silent | the code's; a batch E7 row (a box with Advanced Wisdom, a hero with Basic Wisdom) |
| D-38 | a box's or event's mana above the hero's maximum | code: the total held at 999; VCMI (`calculateManaPoints`, overflow factor 0): up to the maximum, the rest lost; wiki silent | the code's; a batch E7 row (a box with 500 mana, a hero of 100 at most) |
| D-39 | the order of a box's or event's parts | code: experience computed with the Learning before the box's skills, given last (after the spells and creatures); VCMI: the box's skills, then the experience (with the new Learning) and its levels, then mana, morale, luck, artifacts, spells, creatures; wiki silent | the code's; batch E7 rows (a box with Learning and experience; a box with a level's experience and a level-3 spell, a hero whose level-up offers Wisdom) |
| D-40 | the Pyramid's 40 Gold Golems | code (`monster_battle` with a second creature): split like a wandering monster's into 1 to 5 stacks by the army ratio, ordered by the table 0x63DF94; VCMI (`pyramid.json`): one stack of 40; wiki: "40 Gold Golems and two stacks of 10 Diamond Golems" (the split not stated) | the code's; a batch E7 row (a weak and a strong hero at one Pyramid) |

Quirk candidates (T55a2): **A-17** the once-per-hero marks are 32 bits per class (§6.1): the 33rd
object of a class shares the first one's mark. Unreachable on the shipped maps (at most 30) and
probably in the editor (the wiki's "up to 32 learning stones"). Proposal: not reproduced (a mark
per object), no flag. **A-18** a Shrine's or a Scholar's spell of −1 (a draw with nothing left, or
a placed id of 255) is not tested at the visit and reads outside the tables (§6.5); unreachable on
the shipped maps. Proposal: always fixed (nothing to learn: the Shrine refuses as for a known
spell, the Scholar falls back). Not quirks (T55a2): the unclamped primary skill increases (the
level-up's A-9 covers the class), the Arena's Cancel taken as Attack (the wiki: no leaving without
a bonus), the team's marks set before a refusal (knowledge, §10).

Quirk candidates (T55a3): **A-19** events and boxes share one list of at most 400 reward records
(§7.1): a 401st reads the first one's record. Unreachable on the shipped maps (at most 99).
Proposal: not reproduced (a record per object), no flag. **A-20** a Pyramid whose level-5 pool is
empty (the map bans all nine) reads outside the list at a new game (§7.5); unreachable (at most
three banned). Proposal: always fixed (no spell: the Pyramid fights and gives nothing, or the map
is refused; T55b decides). Not quirks (T55a3): the primary skills read as signed bytes (only
positive values given; no shipped value above 5), a box's spells not tested against the bans (VCMI
the same), a looted Pyramid that still asks first, the Pyramid's spell lost when it cannot be
learned (the wiki states it), an event that fires again at every entry (the editor's choice).

T55a1: none. Not quirks: the full backpack that refuses a wearable artifact (the
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
- **New hero state (T55a2):** a visited mark per object for the ten once-per-hero classes (§6.1,
  A-17); the bonus to morale until the end of the hero's next battle and its flag (the Tomb's −3,
  §6.9), which reaches the battle seam (the hero's morale, ADR-0013): T55b's drafts.
- **Decisions (T55a2):** the Arena's two options (no refusal); the Schools' two options or leave;
  the Tree's and the Tomb's yes or no; the University's purchases (any learnable slot while the
  gold lasts, then leave). The computer's choices are E10's.
- **What a player knows (T55a2):** the team's marks (§6.1) make a Shrine's spell, a Witch Hut's
  skill, a Tree's price and a Wagon's or a Tomb's state known to the visitor's team (the
  right-click), hidden from the others; the hero's own marks are its owner's.
- **Draws when the game is built (T55a2, decision 17's setup stream):** the Tree's price, the
  Shrines' spells (with their marks), the Witch Huts' skills, the Scholars' kinds, ids and
  fallbacks (the original draws the Scholar at the map's load), the Universities' four skills, the
  Corpse, the Wagon, the Tomb and the Lean To.
- **Batch E7's rows from T55a2:** D-33, D-34, D-36; the code's alone (VCMI and the wiki silent): the
  full backpack at a Corpse (1 000 gold), a Wagon (its resources) and a Tomb (the artifact stays),
  two Tombs before one battle (−3, not −6); the Learning Stone's 1 000 with Basic Learning (the
  chest's truncation).
- **The proposed rule ids (T55a2):** ADV-TCH-001 Marletto Tower, Garden, Mercenary Camp, Star Axis,
  Arena and Library (§6.2), ADV-TCH-002 the Schools (§6.3), ADV-TCH-003 the Learning Stone and the
  Tree (§6.4), ADV-TCH-004 the Shrines and their draw (§6.5, D-34), ADV-TCH-005 the Witch Hut (§6.6,
  D-33), ADV-TCH-006 the Scholar (§6.7), ADV-TCH-007 the University (§6.8, D-36), ADV-PIK-009 the
  Corpse, ADV-PIK-010 the Wagon, ADV-PIK-011 the Warrior's Tomb, ADV-PIK-012 the Lean To (§6.9,
  D-35), HER-ADV-008 the morale bonus until the next battle's end (§6.9).
- **The import's changes (T55a2):** the 20 classes typed (blockers today); the bodies `h3-map`
  already reads (the Shrine's spell, the Witch Hut's mask, the Scholar's kind and code); no new
  refusal.
- **New hero state (T55a3):** the luck bonus until the next battle's end (+0x11B) beside T55a2's
  morale bonus. Boxes and events add to both, cumulatively. The Pyramid's once flag (status bit 12,
  −2 luck) sits beside the Tomb's (bit 21). HER-ADV-008 covers all of them.
- **Decisions (T55a3):** a box's "open?" (yes or no); the Pyramid's "search?" (yes or no, at every
  visit, a looted one too); the army dialog for a reward's creatures without room. That dialog is
  one dialog over **up to seven** leftover stacks, unlike increment 6's joining group of one stack
  (survey `E08-army.md` §1.1). An event asks nothing. A guarded box's or event's battle is a
  guarded pile's. The computer's answers are E10's.
- **The move (T55a3):** an event that fires **ends the move on its tile**, and one of another colour
  does not. The stop depends on an object no player sees (below), so T55a4 must also settle what a
  move's outcome and the env's masks may reveal.
- **What a player knows (T55a3):**
  - **An event has no picture** (`place_object_tiles` gives it none). No player sees it until it
    fires, and the observation must never show it; when it fires, the visitor learns its text and
    rewards.
  - A box shows one picture for every box; its contents and guards are hidden, and the guards are
    seen only in the battle.
  - The Pyramid's spell is hidden. Whether it was looted is known to the visitor's team (the
    team's mark, `tile_mark_visited_by_team`).
- **Draws when the game is built (T55a3):** the Pyramid's spell (decision 17's setup stream). A box's
  and an event's contents are the map's, with nothing drawn.
- **The battles (T55a3):** a box's and an event's guards as a guarded pile's (no split, a stored
  seed). The Pyramid's goes through the monster battle with a second creature (§7.5: the split less
  the other stacks, the order table 0x63DF94), which the engine's monster battle does not have yet.
- **The end-of-game tests after an event** (0x5F1B10, 0x5F1C80): E10's victory and loss conditions.
- **Batch E7's rows from T55a3:**
  - the disputes D-37, D-38, D-39 and D-40;
  - the code's alone (VCMI and the wiki silent or different):
    - an event that stops a hero mid-move, and one of another colour that does not;
    - a repeatable event that fires at each entry;
    - a box's artifact with a full backpack (lost);
    - a box's creatures without room (one dialog for all; the rest lost);
    - a looted Pyramid twice before a battle (−2, not −4);
    - the Pyramid's spell without Expert Wisdom (lost, the Pyramid looted).
- **The proposed rule ids (T55a3):**
  - ADV-PIK-013 Pandora's Box (§7.2);
  - ADV-OBJ-007 events: firing, the stop, removal and repeats (§7.3);
  - ADV-RWD-001 the reward record's parts and their order (§7.4, D-37 to D-39);
  - ADV-TCH-008 the Pyramid (§7.5, D-40);
  - ADV-MON-012 the monster battle with a second creature (§7.5);
  - HER-ADV-008 widened to the luck bonus and the Pyramid's flag.
- **The import's changes (T55a3):** classes 6, 26 and 63 typed (blockers today). The bodies are the
  ones `h3-map` already reads (`Rewards`, `EventBody`: the players, computer and removal flags).
  The Pyramid's spell is drawn when the game is built. No new refusal: nothing on the shipped maps
  is out of range (§1). The map's timed events (the header's) stay out (decision 127).
