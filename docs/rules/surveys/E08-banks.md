# E08 creature banks survey (E08 T118a)

Date: 2026-11-15. Scope: E08 increment 13 (decisions 333, 334), input for T118b's ADR amendment
drafts, the design note and the cut. **Whole** (the user at T118a's start, decision 335, after the
reading was measured: the content and the draw about 2 KB of code, the visit, the battle and the
reward about 3 KB). Input, not rules (`README.md`): a fact becomes binding when a task writes it
into the catalog with its sources and tests.

**Sources** (retrieved 2026-11-15):

- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete); every address
  cited has an entry in `docs/exe/map.toml` or `structs.toml`. The reading follows the human
  player's path. **Read whole:** `crbanks_load` 0x47AB30, `bank_draw` 0x47AD90,
  `setup_map_objects` 0x4C0CC0 at its five bank cases (0x4C0F0C, 0x4C11A6, 0x4C1256, 0x4C1306,
  0x4C13B6), `visit_object` 0x4A84F0 at its five bank cases (0x4A8CCF, 0x4A8D63, 0x4A8DA9,
  0x4A9E30, 0x4A9E76), `visit_creature_bank` 0x4A15A0, `visit_dragon_utopia` 0x4A2140,
  `visit_search_bank` 0x4AC490, `hero_owner_is_human` 0x4D9050, `bank_record` 0x405DF0,
  `bank_battle_reward` 0x4ABDC0 (the rules; its dialog's text and picture lists by four helpers
  not read), `battle_war_machine_stacks` 0x463600 from its start to the war machines (0x463600 to
  0x463760). **Read in part:** `adventure_battle` 0x4AD470 (the arguments it hands
  `battle_side_setup` 0x4639F0, 0x4AE238 to 0x4AE27C), `battle_side_setup` (the flag's store
  0x463C48), `battle_start` 0x462A20 (the tactics test 0x462C5D), `battle_start_setup` 0x463C60
  (0x46400D), `bank_vector_insert` 0x4D2290 (its start). **Read before** and used here:
  `creature_bank_hint` 0x40D3F0 and `object_popup_text` 0x4137C0's calls of it (E24 M3 T6a2b),
  `battle_obstacles` 0x466290, `battle_setup_battlefield` 0x4643B0 (E08-sea.md §6),
  `hero_step_allowed` 0x481BE0 and `pathfinder_expand` 0x4B2300 (class 85, E08-sea.md §4),
  `random_artifact_of_level` 0x4C94D0, `army_add_stack` 0x44ACE0, `army_full_dialog` 0x5D11D0,
  `hero_give_artifact` 0x4E3070, `give_resource` 0x4E3600, `hero_level_up` 0x4DA720,
  `tile_mark_visited_by_team` 0x4FBF90, `creature_is_unupgraded` 0x47B120, `creature_upgrade`
  0x47B1A0. **Not followed:** the computer player (`ai_fight_answer` 0x52BD10,
  `ai_join_without_room` 0x52BC60, E10's), network play (a branch of `adventure_battle` after
  0x4AD7D7 reuses the flag's stack slot), the pictures, the sounds and the cursor (E24's).
- `vcmi`: VCMI 1.7.5 (tag `1.7.5`): `config/objects/creatureBanks.json` (the eleven kinds as
  rewardable objects with guards), `config/gameConfig.json` (`combat.layouts`:
  `creatureBankNarrow`, `creatureBankWide`), `lib/battle/BattleLayout.cpp`,
  `lib/mapObjects/CRewardableObject.cpp` (the visit, the battle's end, the reward after the
  levels), `lib/mapping/MapFormatH3M.cpp` (`readBank`).
- `wiki`: <https://heroes.thelazy.net/index.php/Crypt> (the layout, the war machines, the morale),
  the search results' pages of the Cyclops Stockpile, the Imp Cache and the Naga Bank
  (<https://heroes.thelazy.net/index.php/Cyclops_stockpile>,
  <https://heroes.thelazy.net/index.php/Imp_cache>,
  <https://heroes.thelazy.net/index.php/Naga_Bank>) and the mirror's list
  (<https://heroes.shoutwiki.com/wiki/Creature_Bank>; read through a summarising tool: re-check a
  quote against the page before a rule rests on it).
- The census (§1): a throwaway example over `h3_map::read` on the 160 shipped maps (not
  committed). The comparison of CrBanks.txt with VCMI's config (§8): a throwaway script (not
  committed).

## 0. Findings in short

- **The census** (§1): 2 137 banks on 148 of the 160 shipped maps, eleven kinds: class 16 by its
  subclass 0 to 6 (1 215 on 143 maps), the Shipwreck 85 (401 / 69), the Crypt 84 (277 / 97), the
  Derelict Ship 24 (170 / 58), Dragon Utopia 25 (74 / 55). Every kind on maps of all three
  versions; one entrance each (the Shipwreck visited from a neighbour, the ten others entered:
  the class lists, corrected by T118b1); 24 and 85 on water (183 of the
  401 Shipwrecks beside land, 11 of the 170 Derelict Ships), the others on land. No body in the map
  file (SoD and before; only HotA's format has one).
- **The content** (§2): CrBanks.txt holds per kind four levels with their chance, up to four
  guard counts, an upgrade chance, seven resources, a creature count and four artifact counts by
  class. The guards' and the reward's **creature types come from the executable's tables**
  (0x6702A0, 0x67037C), not from the text's names (which differ from the creature names for
  Medusae and Dragonflies).
- **The draw at a new game** (§3): the level by one `random_between(1, 100)` over the chances; the
  artifacts drawn at once (relics, majors, minors, treasures, in that order); the guards split
  into five stacks by a fixed rule (four or more types: no split); one more draw for the upgrade
  of one stack (slot 2 for one type). **Never drawn again**: no week or month touches a bank.
- **The visit** (§4): class 16 and Dragon Utopia mark the team's visit, tell a looted bank as
  such, else ask to attack; the Crypt and the two sea banks **ask first** (to search), mark the
  visit only after a yes, and a looted one then costs **−1 morale until the next battle**, once
  per kind (the hero's bits 9 to 11).
- **The battle** (§5): at once after the yes, the guards without a hero, at the bank's entrance
  (24 and 85 on the ship's deck), with the battle's **bank flag** (+0x53C5): no tactics phase, no
  obstacles, no war machines on the field, and both armies on a **fixed layout** (the hero's in the
  middle, the guards round it) by the order of their filled slots. Lost or retreated: the guards'
  survivors stay, the bank is not looted.
- **The reward** (§6): won, the creatures join (else the army dialog), the artifacts are given,
  the resources paid, the bank marked looted for good (never removed), **then** the winner's
  levels.
- **What a viewer sees** (§7): the kind's name and "(Not visited)", or for a visiting team
  "(Visited)" once looted or the guards' total as a size word (E24's reading).
- **Disputes** (§9): D-65 (VCMI's data: the Naga Bank's gems, the Imp Cache's first level without
  its 2 Mercury), D-66 (the Crypt's and the ships' question first, even when looted, and the mark
  after the yes), D-67 (the reward before the winner's levels), D-68 (the artifacts' order), D-69
  (a two-hex guard on the left corners). No quirk candidate. **The checkpoint** (§13, decisions
  336 to 339): the original in D-65, D-66, D-68 and D-67 (the reward before the levels); the bank
  battle the adventure lane's, `cross-lane`; batch E13 before the build.

## 1. The census (the five classes)

All 160 shipped maps read (`h3_map::read`, 159 `.h3m` and the tutorial); banks on 148.

| Kind (k) | Class, subclass | Objects | Maps | RoE / AB / SoD maps' objects | Underground | Entrance |
|---|---|---|---|---|---|---|
| Cyclops Stockpile (0) | 16, 0 | 168 | 90 | 32 / 64 / 72 | 28 | land |
| Dwarven Treasury (1) | 16, 1 | 251 | 106 | 47 / 99 / 105 | 57 | land |
| Griffin Conservatory (2) | 16, 2 | 128 | 62 | 39 / 56 / 33 | 7 | land |
| Imp Cache (3) | 16, 3 | 193 | 100 | 36 / 77 / 80 | 55 | land |
| Medusa Stores (4) | 16, 4 | 180 | 93 | 36 / 73 / 71 | 63 | land |
| Naga Bank (5) | 16, 5 | 150 | 87 | 30 / 53 / 67 | 26 | land |
| Dragon Fly Hive (6) | 16, 6 | 145 | 83 | 46 / 52 / 47 | 25 | land |
| Shipwreck (7) | 85, 0 | 401 | 69 | 63 / 246 / 92 | 14 | water (183 beside land) |
| Derelict Ship (8) | 24, 0 | 170 | 58 | 24 / 91 / 55 | 9 | water (11 beside land) |
| Crypt (9) | 84, 0 | 277 | 97 | 51 / 124 / 102 | 20 | land |
| Dragon Utopia (10) | 25, 0 | 74 | 55 | 8 / 43 / 23 | 16 | land |

Class 16: 1 215 objects on 143 maps. No subclass of 16 above 6 (the setup indexes the kinds' table
by it without a test, §3). Most banks on one map: *Thousand Islands* and its allied twin (106
each), *War of the Mighty* (53). **Templates** (`Objects.txt` and the maps agree): one picture per
kind but the Crypt's three (`AVXgyds0`, `AVXgyne0`, `AVXgysn0`, one per terrain group) and the
Shipwreck's two (`AVAwrek0`, `AVAwre20`); every template has **one visitable tile, which it also
blocks** (on the bottom row, at the template's right corner or one column left of it). *Corrected
by T118b1* (design note's correction; as decision 317 for increment 12): the visit mode is the
class lists', not the mask's: `object_classes_not_entered` 0x63AA58 and `object_classes_from_above`
0x63AB00 list 85 and none of the four others, so the Shipwreck is visited from a neighbour (and
from above), the ten others are entered, the hero standing on the entrance (VCMI 1.7.5:
`blockedVisitable` on the Shipwreck alone). The land kinds' entrances lie
on every land terrain, the subterranean included; 24's and 85's on water only.

**The map file:** no body for any of the five (`docs/formats/h3m.md`, the engine's
`Body::None`; VCMI's `readBank` reads a guard preset, an upgrade choice and an artifact list only
in HotA's format). The level, the guards and the reward are not the map maker's in RoE, AB or
SoD.

## 2. The content (CrBanks.txt and the executable's tables)

`crbanks_load` 0x47AB30 (the game's text tables' load) reads `CrBanks.txt` (H3bitmap and H3ab_bmp
hold the same file): at least 13 records (else the load fails), from record 2 four records per
kind, k = 0 to 10 (record 2 + 4k + l for level l), into the kinds' table `creature_bank_kinds`
0x695038 (0x190 bytes a kind: the name, an H3String at +0x0, from the first record's column 0;
the four levels at +0x10, 0x60 bytes each; H3API's `H3CreatureBankSetup`). Per level:

| Field | Level +off | From |
|---|---|---|
| the guards (an army: types +0x0, counts +0x1C) | +0x00 | types from `bank_guard_creatures` 0x6702A0 (five per kind, −1 ends); counts from columns 3, 6, 8, 10; a count of 0 empties the slot's type |
| the resources (wood, mercury, ore, sulfur, crystal, gems, gold) | +0x38 | columns 13 to 19 |
| the reward's creature | +0x54 | `bank_reward_creatures` 0x67037C by kind; −1 when the count is 0 |
| the reward's count (byte) | +0x58 | column 20 |
| the level's chance (byte, `atoi` of "30%") | +0x59 | column 2 |
| the upgrade chance (byte, "50%") | +0x5A | column 5 |
| artifacts: treasure, minor, major, relic (bytes) | +0x5B to +0x5E | columns 22 to 25 |

Columns 1 (the level), 4, 7, 9, 11 and 21 (the creatures' names), 12 (the "combat value") and 26
to 28 (the analysis) are not read. **The creature types** (the executable's tables, by our base
pack's ids): Cyclops Stockpile `cyclops` (94), Dwarven Treasury `dwarf` (16), Griffin Conservatory
`griffin` (4) with `angel` (12) as the reward, Imp Cache `imp` (42), Medusa Stores `medusa` (76),
Naga Bank `naga` (38), Dragon Fly Hive **`dragon_fly` (105, the upgraded fly)** with `wyvern`
(108), Shipwreck `wight` (60), Derelict Ship `water_elemental` (115), Crypt `skeleton` (56),
`walking_dead` (58), `wight` (60), `vampire` (62), Dragon Utopia `green_dragon` (26),
`red_dragon` (82), `gold_dragon` (27), `black_dragon` (83). The text's names agree with these
but two of them are not our creatures' plurals ("Medusae", "Dragonflies"): a harvest or pack
check takes the types from the executable's tables, not by name.

**The table** (the original's data; the guards before the split of §3):

| Kind | Level 1 (30 %) | Level 2 (30 %) | Level 3 (30 %) | Level 4 (10 %) | Upgrade |
|---|---|---|---|---|---|
| Cyclops Stockpile | 20; 4 of each but gold | 30; 6 each | 40; 8 each | 50; 10 each | 50 % |
| Dwarven Treasury | 50; 2 crystal, 2 500 | 75; 3, 4 000 | 100; 5, 5 000 | 150; 10, 7 500 | 50 % |
| Griffin Conservatory | 50; 1 Angel | 100; 2 | 150; 3 | 200; 4 | 50 % |
| Imp Cache | 100; 2 mercury, 1 000 | 150; 3, 1 500 | 200; 4, 2 000 | 300; 6, 3 000 | 50 % |
| Medusa Stores | 20; 5 sulfur, 2 000 | 30; 6, 3 000 | 40; 8, 4 000 | 50; 10, 5 000 | 50 % |
| Naga Bank | 10; 8 sulfur, 4 000 | 15; 12, 6 000 | 20; 16, 8 000 | 30; 24, 12 000 | 50 % |
| Dragon Fly Hive | 30; 4 Wyverns | 45; 6 | 60; 8 | 90; 12 | 50 % (no effect, §3) |
| Shipwreck | 10 Wights; 2 000 | 15; 3 000 | 25; 4 000, a treasure | 50; 5 000, a minor | 0 |
| Derelict Ship | 20 Water Elementals; 3 000 | 30; 3 000, a treasure | 40; 4 000, a treasure | 60; 6 000, a minor | 0 |
| Crypt | 30 Skeletons, 20 Walking Dead; 1 500 | 25, 20, 5 Wights; 2 000 | 20, 20, 10, 5 Vampires; 2 500, a treasure | 20, 20, 10, 10; 5 000, a treasure | 0 |
| Dragon Utopia | 8 Green, 5 Red, 2 Gold, 1 Black; 20 000; one of each class | 8, 6, 3, 2; 30 000; a minor, a major, 2 relics | 8, 6, 4, 3; 40 000; a major, 3 relics | 8, 7, 6, 5; 50 000; 4 relics | 0 |

(Gold unmarked; every chance column reads 30, 30, 30, 10.)

**The game's records** (the banks vector at game +0x4E3D8, begin +0x4E3DC, emptied at a map's
load by `map_objects_place_all` 0x4FD690): 0x6C bytes each (`creature_bank_record`): the guards
+0x0 (an army), the resources +0x38, the creature +0x54 and its count +0x58 (byte), the artifacts
+0x5C (a vector: begin +0x60, end +0x64). The object's word holds the record's index in bits 13
to 24 (`bank_record` 0x405DF0), the visited marks in bits 5 to 12 (one per player, as every
visitable's), the looted mark in bit 25.

## 3. The draw at a new game

`setup_map_objects` 0x4C0CC0 walks the tiles with an object's entrance (level, row, column) and
fills each class's word (its switch: 16 at 0x4C0F0C, 24 at 0x4C11A6, 84 at 0x4C1256, 85 at
0x4C1306, 25 at 0x4C13B6). A bank's case: the word's bits 5 to 12 (visited) and 13 to 25 cleared,
the next record's index written into bits 13 to 24, a record built by `bank_draw` 0x47AD90 with
the kind (16: the subclass; 85: 7, 24: 8, 84: 9, 25: 10) and appended (`bank_vector_insert`
0x4D2290). The draws are the setup's (`random_between`, the game's generator at the new game) in
the walk's order, three kinds per bank:

1. **The level:** r = `random_between(1, 100)`; for l = 0 to 2, r −= the chance of l, stop at the
   first l with r ≤ 0; else level 3 (with the original's chances: 1 to 30 level 1, 31 to 60
   level 2, 61 to 90 level 3, 91 to 100 level 4). The level's guards, resources and creature are
   copied into the record.
2. **The artifacts:** `random_artifact_of_level` per artifact, **the relics first** (0x10), then
   the majors (8), the minors (4), the treasures (2), appended in that order (the game's pool:
   artifacts not yet given in this game, refilled from the map's bans when one class runs out;
   E24's reading). Drawn when the game starts, so a bank's artifacts leave the pool then.
3. **The split** (by the level's number of guard stacks n, each new stack in the first empty
   slot, so slot order is the order below):
   - n = 1 (every kind of class 16, the Shipwreck, the Derelict Ship): the stack's count c is
     divided four times, by 5, 4, 3 and 2, each quotient of what is left becoming a new stack:
     slots 0 to 4 hold c − h₁ − h₂ − h₃ − h₄, h₁ = ⌊c / 5⌋, h₂ = ⌊(c − h₁) / 4⌋, … (every shipped
     count is a multiple of 5: five equal stacks). The upgrade's slot: 2.
   - n = 2 (the Crypt's level 1): slot 1 halved into slot 2; slot 0 divided by 3 into slot 3 and
     what is left halved into slot 4: 30 Skeletons and 20 Walking Dead become 10, 10, 10, 10, 10
     (Skeleton, Walking Dead, Walking Dead, Skeleton, Skeleton). The upgrade's slot: 3.
   - n = 3 (the Crypt's level 2): slot 1 halved into slot 3, slot 0 halved into slot 4: 13
     Skeletons, 10 Walking Dead, 5 Wights, 10 Walking Dead, 12 Skeletons. The upgrade's slot: 0.
   - n ≥ 4 (the Crypt's levels 3 and 4, Dragon Utopia): no split. The upgrade's slot: 0.
4. **The upgrade:** r = `random_between(1, 100)`, drawn for every bank; r ≤ the level's upgrade
   chance and the slot's creature a town's unupgraded creature (`creature_is_unupgraded`): the
   stack becomes its upgrade (`creature_upgrade`), same count. In RoE (game +0x1F698 = 0) the
   four elementals (112 to 115) are never upgraded (a second, dead branch would empty the slot).
   With the shipped data only class 16 upgrades (50 %): one of its five stacks, slot 2. **The
   Dragon Fly Hive's guards are already Dragon Flies**, so its 50 % never changes them (VCMI
   agrees).

No other code touches a bank's record but the visit (§4 to §6), the hint (§7), the computer's
estimate and the game's construction: **no weekly or monthly refill or redraw**.

## 4. The visit

`visit_object` 0x4A84F0's cases (16 at 0x4A8CCF, 25 at 0x4A8DA9; 24 at 0x4A8D63, 84 at 0x4A9E30,
85 at 0x4A9E76) hand the hero, the tile, the visited tile's packed position and whether the
player is human. The move ends there (as every visit). Two flows:

**Class 16 and Dragon Utopia** (`visit_creature_bank` 0x4A15A0, `visit_dragon_utopia` 0x4A2140):

1. The team's visited marks set at once (`tile_mark_visited_by_team`: bit 5 + p for every player
   p of the hero's team).
2. Looted (bit 25): a human is told so (ADVEVENT 33 with the kind's name; Dragon Utopia GENRLTXT
   425); nothing else.
3. Else a human is asked to attack (ADVEVENT 32 with the name; Dragon Utopia ADVEVENT 47); no:
   nothing more (the marks stay); yes: the battle (§5).

**The Crypt and the sea banks** (`visit_search_bank` 0x4AC490, called with three texts and a
bit: 24 ADVEVENT 41 to 43 and 0x800, 84 ADVEVENT 119 to 121 and 0x400, 85 ADVEVENT 122 to 124
and 0x200):

1. A human (`hero_owner_is_human` 0x4D9050) is **asked first** whether to search (the first
   text), looted or not; no: nothing, **not even the visited marks**.
2. Yes: the team's visited marks set.
3. Looted: a human is told that the search found nothing and the army's morale drops (the second
   text, "upon defeating ... you find nothing"); then, the hero's status bit not yet set (+0x105:
   the Shipwreck bit 9, the Crypt 10, the Derelict Ship 11), it is set and the hero's morale
   (+0x11A) lowered by 1: **−1 morale until the end of its next battle**, once per kind (a second
   looted Crypt changes nothing; a looted Crypt and a looted Shipwreck −2). The computer's hero
   takes the same.
4. Else the battle (§5). The third text ("... you find something!") is never shown: the reward's
   message is the common one (§6).

A hero on land visits a Shipwreck from the coast as it boards a boat (`hero_step_allowed`
0x481BE0's case for water: only a boat's or a class 85 entrance from land; `pathfinder_expand`
0x4B2300 alike; E08-sea.md §3.1, §4), a hero in a boat any of the two sea banks from the water. A
flight passes over a Shipwreck's water tile (the engine's ADV-STUB-12 until it is typed).

## 5. The battle

`bank_battle_reward` 0x4ABDC0 (the hero, the tile, a text, the packed position, human) finds the
record (`bank_record`), counts the guards (for the message, §6) and, for a human, the guard
creature of the highest AI value (the creature table's +0x40; the first on a tie). Then the hero
stops (`adventure_hero_stop`), the C runtime's generator is seeded with **s = 170 101 x + 223 252 y
+ 250 037 z + 82 037** (32 bits; the visited tile, the bank's entrance; a monster's coefficients
differ, E08-battles.md §4.1) and `adventure_battle` 0x4AD470 starts the battle at that tile: the
hero's army against **the record's army without a hero** (owner −1, no town), the stored seed s,
and its last argument, **the bank flag, 1** (a monster's battle passes 0). `adventure_battle`
hands the flag to `battle_side_setup` 0x4639F0 as its eleventh argument, which stores it in the
battle manager's +0x53C5 (0x463C48; H3API's `creatureBank`). The flag's effects (the battle's
sites, read for this):

- **No tactics phase** (`battle_start` 0x462C5D: the phase only when the Tactics difference is
  above 0 and the flag is clear; BAT-TAC-001's "no tactics in creature banks").
- **No obstacles** (`battle_obstacles` 0x4662B3: none with the flag; read before, installation.md
  §7.26).
- **No war machines on the field** (`battle_war_machine_stacks` 0x463600, called with the flag by
  `battle_start_setup` 0x46400D: 0x463758 skips the Catapult, Ballista, First Aid Tent and Ammo
  Cart). The Ammo Cart's passive test reads the hero's artifact, not the stack (BAT-WAR-004), so
  its unlimited shots stay (the wiki agrees).
- **The layout** (0x4636B7): each side's stacks, in slot order, take the hexes of
  `bank_battle_hexes` 0x63D0E0 by **their ordinal among the filled slots** (not the slot, not the
  formation): the attacker 57, 61, 90, 93, 96, 125, 129 (columns 6 and 10 of row 3, 5, 8 and 11 of
  row 5, 6 and 10 of row 7: the middle), the defender 15, 185, 172, 2, 100, 87, 8 (column 15 of
  rows 0 and 10, column 2 of rows 10 and 0, column 15 and 2 of row 5, column 8 of row 0). Five
  guard stacks stand in the four corners and the middle right; the upgraded slot 2 at 172, the
  lower left (the wiki's "C hex"); four stacks (the Crypt's levels 3 and 4, Dragon Utopia) in the
  four corners. The hero's tight formation plays no part.

**The battlefield:** the bank's tile's. The Shipwreck and the Derelict Ship set the ship flag
(+0x53C6) by their class: water and the deck (`CmBkDeck`; E08-sea.md §6), also for a hero
attacking a Shipwreck from the coast; the land banks the tile's terrain or special ground, as any
battle (GAM-BAT-001). After the battle the hero is mobilized again; **lost or retreated**
(`adventure_battle` returns another winner than the attacker): the function returns 0 at once,
the guards' survivors stay in the record (its army is the battle's, as a garrison's; the upgraded
survivors too, unlike a monster's A-6), the bank is not looted and is asked again at the next
visit; the hero leaves the map as after any lost battle. A neutral side cannot be surrendered to
(the battle lane's). The battle's experience is the battle's (GAM-BAT-002).

## 6. The reward

Won, in this order (0x4AC3C7 to 0x4AC473):

1. A human is shown the win (`message_box_pictures`, ADVEVENT 34: "Having defeated the %s, you
   discover %s.") with the strongest guard's name (singular when the guards counted 1), and the
   reward listed: the creatures ("%d %s", the picture kind 0x15), each artifact (ADVEVENT 45), each
   resource above 0 (gold's picture changed when artifacts come with it), joined by ", " and
   GENRLTXT 142 before the last.
2. **The creatures** (count > 0): `army_add_stack` with slot −1 (a stack of the type, else the
   first empty slot); no room: a human's **army dialog** (`army_full_dialog` 0x5D11D0, its seven
   local slots: what is left there when it closes is lost; E08-army.md §1), the computer's
   `ai_join_without_room`.
3. **The artifacts**, in the record's order (relics first): `hero_give_artifact` each (equipped at
   the first body position that takes it, else the backpack; neither: lost; SoD's combination
   question).
4. **The resources**, wood to gold, to the hero's owner (`give_resource`: 32-bit add, a negative
   result 0).
5. The bank **looted** (the word's bit 25), never removed.
6. **The winner's levels** (`hero_level_up` 0x4DA720) last.

No experience is given by the reward itself. VCMI gives its reward after the levels
(`heroLevelUpDone`), and so does the engine for every won battle's rest (GAM-DEC-007 point 3):
D-67.

## 7. What a viewer sees

E24 M3 T6a2b's reading (`creature_bank_hint` 0x40D3F0, for the map hint and the right-click popup
`object_popup_text` 0x4137C0 alike; E24-design.md §4.12), for the player the hint is shown to:
the kind's name, then **"(Not visited)"** (GENRLTXT 354) unless the word's bit 5 + player is set;
set, **"(Visited)"** (GENRLTXT 353) once looted or with no guards left, else "(" the guards ")":
their total count's size word and the only creature's plural name ("Lots of Cyclopes"), GENRLTXT
44 for several creatures. No entrance test. So a team learns a bank's guards (as a size word) by a
visit: class 16 and Dragon Utopia by any visit, a declined one too; the Crypt and the sea banks
only by a yes (a fight or a looted search). The real counts, the level and the reward stay
hidden until the battle (the battle shows the counts) and the win's message. The morale marks are
the owner's (its hero's status).

## 8. VCMI and the wiki

- **VCMI 1.7.5** models the eleven as rewardable objects with guards (`creatureBanks.json`):
  the same chances (an `appearChance` per level), the guards **already split** in its config (the
  original's split of §3 written out: the Crypt's 10, 10, 10, 10, 10 and 13, 10, 5, 10, 12; four
  stacks for the Crypt's levels 3 and 4 and Dragon Utopia), the upgrade chance on the third stack
  (`upgradeChance` 50; none for the Dragon Fly Hive's `fireDragonFly`, the original's Dragon
  Fly), the morale −1 for one battle on a looted Crypt, Shipwreck and Derelict Ship (`onVisited`,
  stacked once per kind), the Shipwreck `coastVisitable`. **Data that differ** (the throwaway
  comparison of every level): the Naga Bank gives **gems** (8, 12, 16, 24) where CrBanks.txt has
  sulfur (the HD mod's change, the wiki says), and the Imp Cache's first level **lacks its 2
  Mercury** (1 000 gold only); every other level agrees. **The flow:** every visit marks the
  bank scouted first; a guarded bank asks, a yes fights, a win gives the reward after the levels;
  a bank without guards (looted) gives its `onVisited` at once (no question). The layouts
  (`gameConfig.json`): `creatureBankNarrow` is the original's table; `creatureBankWide` (the
  Griffin Conservatory, Medusa Stores, Naga Bank, Derelict Ship, Dragon Utopia) moves the three
  left hexes one column out (171, 1, 86) for two-hex guards; no tactics, no obstacles, the war
  machines left out. Artifacts listed treasure first.
- **The wiki** (the Crypt's page; the bank pages): the five stacks and the upgraded stack's 50 %
  "in the lower left corner (C hex)"; the hero's stacks fill the hexes from the first "regardless
  of their slot order"; no war machines on the field but their passive effects stay; no tactics;
  slain guards do not come back after a retreat or a loss; a looted Crypt's search −1 morale, not
  again for another Crypt; the guards' preview by the hint after a visit "(not attacked)"; the
  Naga Bank's sulfur (gems under the HD mod); the Imp Cache's 1 000 gold and 2 Mercury. The
  wiki calls the Crypt's guards five stacks at every level (four at levels 3 and 4 in the code and
  in VCMI).

## 9. Disputes, quirk candidates and open points

- **D-65 (VCMI's data;** two rows): the Naga Bank's sulfur (the original's CrBanks.txt, the wiki)
  against VCMI's gems; the Imp Cache's level 1 with 2 Mercury (the original, the wiki) against
  VCMI's none. Data, settled by the original's table; batch E13 can confirm the Naga Bank's.
- **D-66 (the Crypt's and the sea banks' flow):** the original asks first, a looted one too, and
  marks the team's visit only after the yes (a declined Crypt teaches nothing, a declined Cyclops
  Stockpile its guards' size word); VCMI marks every visit at once and gives a looted one's −1
  morale without a question. The engine's Pyramid already follows the original's form (ask first,
  a looted one's mark after the yes: ADV-VIS-006, `OfferKind::SearchPyramid`).
- **D-67 (the reward and the levels):** the original gives the creatures (with their army dialog),
  the artifacts and the resources, then the winner's levels; VCMI and the engine's GAM-DEC-007
  point 3 (every won battle's rest: the Pyramid, the guarded piles, Pandora's Box, the garrisons)
  the levels first. Nothing the reward gives changes a level-up or the reverse (the levels' draws
  and offers do not read the army, the artifacts or the treasury), so only the dialogs' order
  differs. Batch E13 can record it.
- **D-68 (the artifacts' order):** relics first in the original (the record's vector, the order
  they are given, so the equipped positions and the backpack's order); VCMI treasure first.
  Visible only with Dragon Utopia (several classes) and a hero whose positions fill up.
- **D-69 (a two-hex guard on the left):** the original has one hex table for every creature (a
  two-hex guard's position 172, 2 or 87); VCMI keeps a second table for two-hex guards (171, 1,
  86). Which hexes such a stack covers is the battle's footprint rule (the battle lane's
  BAT-SET-001: a defender's position its rear hex, its front one column toward the attacker); the
  two may describe the same cells under different conventions. Batch E13 records the Griffins'
  hexes.
- **No quirk candidate.** Considered and not candidates: the third texts never shown (messages
  only), the Dragon Fly Hive's 50 % without effect (no rule changes), the RoE elementals' dead
  branch (unreachable), the visited mark only after a yes (D-66: the original's design, as the
  Pyramid's), a lost reward's creatures in a closed army dialog (the dialog's rule, E08-army.md).
  The reward's creature count is a signed byte (at most 12 shipped) and a guard count an i32: no
  wrap within the data.
- **Open points for T118b:** (1) where the eleven kinds' data live (recommended: the base pack's
  adventure part, every kind on RoE maps too; a check against CrBanks.txt and the executable's two
  tables, as `h3-pack-adventure`'s original-data check); (2) the draw's stream and order (the
  engine's setup stream when the game is built, in object order after the random artifacts and
  the Pyramid's spell (ADV-TCH-008); the level, the artifacts relics first, the upgrade; the
  original's walk is by entrance tile); (3) the record in the adventure state (the guards, the
  resources, the creature and count, the artifacts, the looted mark; the visited marks per player
  as ADV-VIS-006's knowledge); (4) the question as an offer of the visitor's owner (two kinds: to
  attack, class 16 and 25; to search, 24, 84, 85, as the Pyramid's), the battle at once after a
  yes; (5) the reward through the shared reward record or its own (ADV-RWD-001's creatures with
  their army dialog, artifacts, resources; the bank's order); (6) the three once-marks (−1
  morale, HER-ADV-008); (7) the bank battle's setup (§5, the battle lane's inbox: a `BattleSetup`
  field in `h3-contracts`, shared); (8) ADV-STUB-12's lift (the Shipwreck's water tile: a flight
  passes, a walker visits from the coast).

## 10. Commands, decisions, events, the observation, replays and benchmarks

For T118b's drafts and design note (ADR-0015 to -0017, ADR-0013, the battle setup seam); a sketch,
not decided.

- **Commands:** none new. A bank is visited by `MoveHero`'s last step onto its entrance (the ten
  entered kinds; *corrected by T118b1*, §1) or toward it (the Shipwreck, `Visit::Neighbour`), from
  land or from a boat (the sea banks), and again by a later move or `Revisit`; the question is answered by the offer's answer, the battle by its commands, the reward's
  army dialog by the dialog's (GAM-DEC-004).
- **Decisions:** the question, a pending offer of the visitor's owner (GAM-DEC-001: two kinds or
  one with the bank's kind), declining keeps nothing; the battle (GAM-BAT-001); the reward's army
  dialog when the army has no room; the winner's levels (GAM-DEC-002) after the reward (D-67 as
  the original, decision 337: an exception to GAM-DEC-007 point 3's order).
- **Events:** the offer and its answer; a looted bank told (`ObjectEmpty`'s use, or the Pyramid's
  `Known::Emptied` path); the morale mark (`HeroChange::Bonus` with its once-mark); the battle's
  events; the reward's (`CreaturesJoined`, the artifacts received, the resources gained, as a
  reward record's); the bank looted (to whom: the visitor's owner; the others by the hint's
  knowledge). A new variant of `h3-game`'s, `h3-adventure`'s events breaks `h3-protocol`'s build
  until mapped (the global trap: `cross-lane`, or the server lane first).
- **The observation:** a bank's kind to every viewer that sees its tile; per team, visited or not
  and, visited, looted ("(Visited)") or its guards' size word and creature ("several creatures"
  for more than one type): a knowledge by object (ADV-VIS-006 amended) that changes with a visit, a
  battle's survivors and the loot; the counts, the level and the reward private (server-only) until
  the battle. The hero's morale marks in the owner's view (`OwnHeroView`). No flat encoding
  (decision 187).
- **State and versions:** the records (adventure state hash), the kinds (adventure ruleset part,
  perhaps the pack format), the import of the five classes typed (map import version), the once-
  marks (game state hash through the hero's visit marks); the battle setup's field (the battle
  state, the core ruleset if the layout is content). Replays: no new command, the format stays;
  the golden replays' hash lines move (`homm3-rerecord-replays`).
- **The env** (the AI lane's): the offer is an existing decision kind for it unless T118b adds a
  kind; the observation's additions are theirs to encode.
- **Benchmarks:** the 2 137 objects stop being inert blockers on 148 shipped maps (their
  neighbours' steps toward them end paths, ADV-OBJ-003), so the shipped-map workloads' counts move
  once (re-baselined); ADV-STUB-12's lift touches the passage of a flight over water (the hot
  path: `homm3-bench-regression`); a bank-heavy XL map and a workload with bank battles (as
  T116c's). A test that uses one of the five classes as a plain blocker breaks (`TRAPS.md`).

## 11. Batch E13's plan

Increment 13's batch, **batch E13**, runs on the client lane's harness in SoD on Easy, hot seat,
from maps `h3-experiment` writes (`docs/experiments/README.md`), as batches E3 to E12; its spec
and recipe are tasks of T118b's cut. One run, days 1 to 3, two players (the second for nothing
but the hint's other team). The level and the upgrade are drawn at the start, so each row reads
the drawn record first (game +0x4E3DC, `creature_bank_record`) and predicts from it.

| Row | Settles | Setup | Record | The reading predicts |
|---|---|---|---|---|
| E13-1 | §3 | the eleven kinds, two of each, on one map | each record at day 1: the guards by slot, the resources, the creature, the artifacts' classes | a level's row of §2; five stacks (the Crypt's levels 3 and 4 and Dragon Utopia four) split as §3; at most slot 2 upgraded, class 16 only; Dragon Flies never upgraded |
| E13-2 | §4, §7 | Red declines a Cyclops Stockpile, then a Crypt | the hint of both for Red and for Blue; the words' bits 5 to 12 | the Stockpile "(Lots of Cyclopes)" for Red, "(Not visited)" for Blue; the Crypt "(Not visited)" for both (D-66; VCMI: scouted) |
| E13-3 | §5 | Red's hero with Expert Tactics, a Ballista and 3 stacks attacks a Dwarven Treasury | the battle's stacks and hexes; a tactics phase; obstacles; war machines | 57, 61, 90 for Red's (by ordinal), 15, 185, 172, 2, 100 for the guards; no tactics, no obstacles, no Ballista |
| E13-4 | §5, D-69 | Red attacks a Griffin Conservatory (two-hex guards) | each guard stack's two hexes | the hexes of 15, 185, 172, 2, 100 under the battle's footprint rule (VCMI's 171 for the third) |
| E13-5 | §5 | Red attacks the Shipwreck from the coast; a second hero from a boat a Derelict Ship | the battlefield | the deck both times |
| E13-6 | §6, D-67 | Red's hero, its army full and near a level, wins a Griffin Conservatory | the dialogs' order; the army after | the army dialog with the Angels before the level-up (VCMI: the level first) |
| E13-7 | §6, D-68 | Red's hero with full body positions wins a Dragon Utopia | the backpack's order | relics first (VCMI: treasure first) |
| E13-8 | §4 | after E13-6 Red revisits the looted Conservatory; another hero says yes at two looted Crypts and a looted Shipwreck, then fights | messages; +0x11A and the bits 9 to 11 before and after the battle | the deserted message, nothing else; −1, then −1 more only for the Shipwreck (−2); 0 after the battle |
| E13-9 | §5 | Red's weak hero attacks an Imp Cache and retreats | the record after; the hint | the survivors in the record, not looted; the hint by the survivors |
| E13-10 | §6, D-65 | Red wins a Naga Bank and an Imp Cache at level 1 (by the drawn records) | the treasury | sulfur (VCMI gems); 2 Mercury with the 1 000 gold (VCMI none) |

**Not in the batch:** the computer's visits (E10), network play, the seed's draws inside the
battle (unread, BAT's). **What the harness lacks** (`clients/capture/`): a read of the banks'
records (game +0x4E3DC, 0x6C each) and of a hero's +0x105 and +0x11A (E12's reads may serve), the
battle's stack hexes at the start (the battle capture has them), the dialogs' order. **What the
spec lacks** (`h3-experiment`, battle-owned: `cross-lane`; the writer is this lane's): the five
classes by template (no body), a hero's secondary skills and war machines at the start, a boat
for E13-5.

## 12. The proposed rule ids

A family **ADV-BNK**:

- **ADV-BNK-001** the objects: the five classes, eleven kinds (16 by subclass 0 to 6, 85, 24, 84,
  25), one entrance each, the ten entered and the Shipwreck visited from a neighbour (the
  class lists, §1 as corrected; the sea banks from water, the Shipwreck from the coast too); no
  body; the kinds' four levels as data (§2).
- **ADV-BNK-002** the draw when the game is built: the level, the artifacts (relics first), the
  split, the upgrade (§3); never again.
- **ADV-BNK-003** the visit: the two flows of §4, the question as an offer, declining; a looted
  bank (told; the three's −1 morale once per kind, HER-ADV-008 amended with three once-marks).
- **ADV-BNK-004** the battle: at once after a yes, the record's army without a hero at the
  entrance's tile (the deck for 24 and 85), the bank battle's setup (with the battle lane's rule:
  no tactics, no obstacles, no war machines on the field, the layout); lost or retreated: the
  survivors stay, not looted.
- **ADV-BNK-005** the reward: the creatures (room, else the army dialog), the artifacts in the
  record's order, the resources; looted for good; then the winner's levels (D-67, decision 337).
- **ADV-BNK-006** what a viewer sees: the kind, the team's visit, looted or the guards' size word
  (ADV-VIS-006 amended); the marks to the owner.

Also amended: GAM-BAT-001 (the sea banks' deck; the bank battle's setup), GAM-DEC-007 (a bank's
reward as a won battle's rest), ADV-OBJ-003 (the five's entrances end a path), ADV-MOV-009 and
ADV-SEA-002 (ADV-STUB-12 lifted: the Shipwreck's water tile), ADV-VIS-006; the battle lane's
BAT-SET-001 (the bank layout), BAT-TAC-001, BAT-WAR-001, BAT-GEO-004 (none in a bank).

## 13. For T118b: the checkpoint's questions

Asked at T118a's end (`docs/OPEN-QUESTIONS.md`, ADV-Q53; the answers in the epic as decisions
from 336):

1. **D-65, D-66, D-68:** the original in each (recommended): the original's table (sulfur, the
   Imp Cache's Mercury); the Crypt's and the sea banks' question first, the mark after a yes; the
   artifacts relics first.
2. **D-67:** the reward after the winner's levels (recommended: GAM-DEC-007 point 3 as for every
   won battle's rest; only the dialogs' order differs), or before them as the original.
3. **The bank battle** (§5; `BattleSetup` lacks it, a shared seam): the adventure lane builds it
   in increment 13, `cross-lane`, after a `shared/` change of the seam (recommended), or the
   battle lane first from its inbox.
4. **Batch E13** before the build (recommended, as batches E9 to E12), after it, or not at all.

D-69 is left to batch E13 (the battle's footprint rule decides the cells; no question). The open
points of §9 are T118b's calls.

**Answered** (2026-11-15, `AskUserQuestion`, one round; the epic's decisions 336 to 339; ADV-Q53):
(1) D-65, D-66 and D-68 **the original** in each (as recommended); (2) D-67 **the reward first, as
the original** (not as recommended): the creatures with their army dialog, the artifacts and the
resources before the winner's levels, a bank's exception to GAM-DEC-007 point 3 for T118b to
design; (3) the bank battle **built by the adventure lane, `cross-lane`**, after a `shared/` change
of the seam, the battle lane's inbox told (as recommended); (4) **batch E13 before the build** (as
recommended).
