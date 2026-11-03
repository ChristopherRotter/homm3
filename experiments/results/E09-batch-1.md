# E09 batch TWN-1 — results

> Played by the capture harness on 2026-11-03 (`clients/capture/h3adv.py batch clients/capture/batches/E09-batch-1.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory. The map started anew 2 more time(s) by the recipe's `new_game` steps (new games, no save loaded).

Map file: `E09-batch-1.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `7902bb9e`.** The map's description in the scenario list starts with "Stamp 7902bb9e.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E09-batch-1.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1).
4. The records are the game's memory: a town's record (game +0x21614, 0x168 bytes each, `docs/exe/structs.toml`; its fields from docs/rules/surveys/E09-towns.md §5, §8, §9): its owner +0x1 (−1 neutral), its visiting hero +0x10, its garrison +0xE0 (seven creature ids, then seven counts), its built set +0x150 and every-built set +0x158 (a bit per building id); the battle's stacks at a siege's opening (E04c's reads); a hero's tile and movement points (E08's). The capture harness reads them (`clients/capture/batches/E09-batch-1.toml`, E09 T3b). Played by hand, only rows 3 and 4 can be read from the screens.
5. Tiles are (column, row) from 0 at the top left; a town's entrance is the tile its gate stands on. A step: click the target once (the path's marks show), then again (the hero goes).
6. Order: game 1, day 1, Red's turn, before any hero moves: TWN-1-1's and TWN-1-2's reads; save 1. TWN-1-3 twice from save 1 (3a, then load save 1 and 3b); TWN-1-4 after 3b, in the same turn. Then start the map anew (a new game, not a load) twice more, games 2 and 3: TWN-1-1's and TWN-1-2's reads at day 1, nothing else.
7. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## TWN-1-1 — The buildings of the editor's standard sets at day 1, owned and neutral, with and without a Fort (TD-3, TWN-SET-003)

- Red hero **TWN-1-1** (Orrin, specialty skill Archery): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn, before any hero moves, in each of the three games: read every town's built set (+0x150) and every-built set (+0x158) as building ids. The towns are listed below with their entrances; this row's Red hero stands at (60, 66) and never moves.

**Record**

| town | game 1: built (ids) | game 2: built | game 3: built |
|---|---|---|---|
| RF1 castle (7, 7) | 5, 7, 10, 30 | 5, 7, 10, 30, 31 | 5, 7, 10, 30 |
| RF2 rampart (15, 7) | 5, 7, 10, 23, 30 | 5, 7, 10, 23, 30 | 5, 7, 10, 23, 30 |
| RF3 tower (23, 7) | 5, 7, 10, 30, 31 | 5, 7, 10, 30 | 5, 7, 10, 30 |
| RF4 inferno (31, 7) | 5, 7, 10, 30 | 5, 7, 10, 30 | 5, 7, 10, 30 |
| RF5 necropolis (39, 7) | 5, 7, 10, 23, 30, 31 | 5, 7, 10, 23, 30 | 5, 7, 10, 23, 30 |
| RF6 dungeon (47, 7) | 5, 7, 10, 30 | 5, 7, 10, 30 | 5, 7, 10, 30, 31 |
| RF7 stronghold (55, 7) | 5, 7, 10, 30 | 5, 7, 10, 30, 31 | 5, 7, 10, 30 |
| RF8 fortress (63, 7) | 5, 7, 10, 30 | 5, 7, 10, 30 | 5, 7, 10, 30 |
| BF1 conflux (7, 15) | 5, 7, 10, 23, 30 | 5, 7, 10, 23, 30 | 5, 7, 10, 23, 30 |
| BF2 castle (15, 15) | 5, 7, 10, 30 | 5, 7, 10, 30 | 5, 7, 10, 30 |
| BF3 rampart (23, 15) | 5, 7, 10, 23, 30, 31 | 5, 7, 10, 23, 30 | 5, 7, 10, 23, 30 |
| BF4 tower (31, 15) | 5, 7, 10, 30 | 5, 7, 10, 30 | 5, 7, 10, 30 |
| BF5 inferno (39, 15) | 5, 7, 10, 30 | 5, 7, 10, 30 | 5, 7, 10, 30 |
| BF6 necropolis (47, 15) | 5, 7, 10, 23, 30 | 5, 7, 10, 23, 30 | 5, 7, 10, 23, 30 |
| BF7 dungeon (55, 15) | 5, 7, 10, 30, 31 | 5, 7, 10, 30, 31 | 5, 7, 10, 30 |
| BF8 stronghold (63, 15) | 5, 7, 10, 30 | 5, 7, 10, 30, 31 | 5, 7, 10, 30, 31 |
| R1 fortress (7, 23) | 5, 10 | 5, 10 | 5, 10 |
| R2 conflux (15, 23) | 5, 10, 23 | 5, 10, 23 | 5, 10, 23 |
| R3 castle (23, 23) | 5, 10 | 5, 10 | 5, 10 |
| R4 rampart (31, 23) | 5, 10, 23 | 5, 10, 23 | 5, 10, 23 |
| B1 tower (39, 23) | 5, 10 | 5, 10 | 5, 10 |
| B2 inferno (47, 23) | 5, 10 | 5, 10 | 5, 10 |
| B3 necropolis (55, 23) | 5, 10, 23 | 5, 10, 23 | 5, 10, 23 |
| B4 dungeon (63, 23) | 5, 10 | 5, 10 | 5, 10 |
| NF1 stronghold (7, 31) | 7, 10, 30, 31 | 7, 10, 30, 31 | 7, 10, 30, 31 |
| NF2 fortress (15, 31) | 7, 10, 30, 31 | 7, 10, 30, 31 | 7, 10, 30, 31 |
| NF3 conflux (23, 31) | 7, 10, 23, 30, 31 | 7, 10, 23, 30, 31 | 7, 10, 23, 30, 31 |
| NF4 castle (31, 31) | 7, 10, 30, 31 | 7, 10, 30, 31 | 7, 10, 30, 31 |
| N1 rampart (39, 31) | 10, 23, 30, 31 | 10, 23, 30, 31 | 10, 23, 30, 31 |
| N2 tower (47, 31) | 10, 30, 31 | 10, 30, 31 | 10, 30, 31 |
| N3 inferno (55, 31) | 10, 30, 31 | 10, 30, 31 | 10, 30, 31 |
| N4 necropolis (63, 31) | 10, 23, 30, 31 | 10, 23, 30, 31 | 10, 23, 30, 31 |
| NEF dungeon (7, 39) | 7, 10, 30, 31 | 7, 10, 30, 31 | 7, 10, 30, 31 |
| NE stronghold (15, 39) | 10, 30, 31 | 10, 30, 31 | 10, 30, 31 |

- every-built (+0x158) different from built anywhere (which town, which ids): game 1: none; game 2: none; game 3: none
- anything odd: 

## TWN-1-2 — A neutral town's explicitly empty custom garrison at day 1, and the drawn garrisons beside it (TD-5, TD-4, TWN-SET-004)

- Red hero **TWN-1-2** (Sylvia, specialty skill Navigation): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn, before any hero moves, in each of the three games: read the garrison (+0xE0: slot, creature, count) of the ten neutral towns, and whether every owned town's garrison is empty. This row's Red hero stands at (64, 66) and never moves.

**Record**

| town | game 1: garrison (slot: creature × count) | game 2 | game 3 |
|---|---|---|---|
| NF1 stronghold (7, 31) | slot 0: 7 Wolf Riders, slot 1: 3 Ogres | slot 0: 5 Orcs | empty |
| NF2 fortress (15, 31) | slot 0: 12 Gnolls, slot 1: 2 Basilisks | slot 0: 8 Gnolls, slot 1: 7 Lizardmen | slot 0: 13 Gnolls, slot 1: 7 Lizardmen |
| NF3 conflux (23, 31) | slot 0: 15 Pixies | slot 0: 5 Air Elementals | slot 0: 3 Water Elementals |
| NF4 castle (31, 31) | slot 0: 6 Archers, slot 1: 2 Swordsmen | slot 0: 13 Pikemen, slot 1: 6 Archers | empty |
| N1 rampart (39, 31) | slot 0: 14 Centaurs, slot 1: 6 Dwarves | empty | slot 0: 5 Wood Elves |
| N2 tower (47, 31) | slot 0: 5 Stone Golems, slot 1: 1 Mage | empty | slot 0: 5 Stone Golems |
| N3 inferno (55, 31) | slot 0: 6 Gogs | slot 0: 7 Gogs | empty |
| N4 necropolis (63, 31) | slot 0: 5 Walking Dead | slot 0: 6 Walking Dead, slot 1: 3 Wights | empty |
| NEF dungeon (7, 39) | empty | empty | empty |
| NE stronghold (15, 39) | empty | empty | empty |

- the owned towns' garrisons (all 24 empty? which not): game 1: 1 of 25 with a stack: (20, 61): slot 0: 1 Cavalier, slot 1: 16 Halberdiers, slot 2: 10 Marksmen; game 2: 1 of 25 with a stack: (20, 61): slot 0: 1 Cavalier, slot 1: 16 Halberdiers, slot 2: 10 Marksmen; game 3: 1 of 25 with a stack: (20, 61): slot 0: 1 Cavalier, slot 1: 16 Halberdiers, slot 2: 10 Marksmen
- anything odd: the game's version: SoD

## TWN-1-3 — The garrison's merge: Blue's town with a lone visitor of seven stacks and a garrison of three; Red besieges (TD-7, TWN-SIE-001, TWN-SIE-002, TWN-CAP-003)

- Red hero **TWN-1-3** (Caitlin, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 20 Archangel
- Opponent: Blue Castle town (Village Hall, fort); garrison 1 Cavalier, 16 Halberdier, 10 Marksman; visiting hero: Blue hero **Visitor** (Octavia, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 4 Swordsman, 4 Centaur, 5 Archer, 5 Griffin, 5 Pikeman, 3 Monk, 10 Pikeman

1. Blue's Castle (a Fort only: no towers) has its entrance at (20, 61); Blue's hero Visitor visits it (it stands on the entrance) with seven stacks, and the town's garrison holds three. Red's hero TWN-1-3 (twenty Archangels) stands south of it at (20, 62). Before the step, read Red's hero's movement points, the town's garrison and Blue's hero's army (slot, creature, count).
2. 3a: step north onto the entrance (a siege). At the battle's opening read Blue's stacks (each one's army slot or hex, creature, count). Every Blue stack defends; Red retreats at its first stack's turn. After the battle read the town's garrison and Blue's hero's army.
3. 3b: load save 1 and step north again. At the opening read Blue's stacks again; then Red wins (auto combat, or the Archangels strike stack after stack while Blue's stacks defend; take any level-up's first offer). Right after the battle read: Red's hero's tile and movement points, the town's owner, whether a town screen opened, the town's garrison.

**Record**

| slot | before: garrison | before: Blue's army | 3a opening: Blue's stacks | 3a after: garrison | 3a after: Blue's army | 3b opening: Blue's stacks |
|---|---|---|---|---|---|---|
| 0 | 1 Cavalier | 4 Swordsmen | 4 Swordsmen | empty | 4 Swordsmen | 4 Swordsmen |
| 1 | 16 Halberdiers | 4 Centaurs | 16 Halberdiers | 4 Centaurs | 16 Halberdiers | 16 Halberdiers |
| 2 | 10 Marksmen | 5 Archers | 10 Marksmen | 5 Archers | 10 Marksmen | 10 Marksmen |
| 3 | empty | 5 Griffins | 5 Griffins | empty | 5 Griffins | 5 Griffins |
| 4 | empty | 5 Pikemen | 1 Cavalier | empty | 1 Cavalier | 1 Cavalier |
| 5 | empty | 3 Monks | 3 Monks | empty | 3 Monks | 3 Monks |
| 6 | empty | 10 Pikemen | 15 Pikemen | empty | 15 Pikemen | 15 Pikemen |

- Red's hero before the step: tile, movement points: 20,62,0; 2000
- 3b after the battle: Red's hero's tile, movement points; the town's owner; a town screen (yes/no); the garrison: 20,62,0; 1900; red; no; empty
- anything odd: game 1, the town (20, 61) before 3a: owner; visitor; built; every built: blue; Visitor; 7, 10; 7, 10; 3a: the winner; who retreated; Red's hero on the map after: defender; attacker; no; 3a opening: the sides; Blue's stacks' hexes; Red's stacks: attacker red (human 1, here 1); defender blue (human 1, here 1); 15, 49, 83, 100, 117, 151, 185; slot 0: 20 Archangels, slot -1: 1 Catapult; 3a after: the town's owner; its visitor: blue; Visitor; 3b (game 2) before the step: Red's hero's tile, points; the garrison; Blue's army: 20,62,0; 2000; slot 0: 1 Cavalier, slot 1: 16 Halberdiers, slot 2: 10 Marksmen; 0: 4 Swordsmen, 1: 4 Centaurs, 2: 5 Archers, 3: 5 Griffins, 4: 5 Pikemen, 5: 3 Monks, 6: 10 Pikemen; 3b: the winner; the level-ups; the messages: attacker; level 2: Scouting or Wisdom, took left (Scouting); none; 3b after: the town's visitor; built; Visitor on the map: none; 7, 10; no

## TWN-1-4 — After a siege won: a defended neutral town, then a later step onto the gate (TWN-CAP-001, TWN-CAP-003, TWN-VIS-001)

- Red hero **TWN-1-4** (Jenova, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Archangel
- Opponent: Neutral Rampart town (the standard buildings with a fort); garrison 5 Centaur

1. After 3b, in the same turn. A neutral Rampart (the editor's standard set with a Fort; a custom garrison of 5 Centaurs) has its entrance at (40, 61); Red's hero TWN-1-4 (five Archangels) stands south of it at (40, 62). Read its movement points, then step north onto the entrance (a siege). Red wins (auto combat, or the Archangels strike).
2. Right after the battle read: Red's hero's tile and movement points, the town's owner, whether a town screen opened, the town's garrison.
3. Then step north onto the entrance again. Read the hero's tile and movement points, whether a town screen opened (close it), the town's visiting hero (+0x10).

**Record**

- Red's hero before the step: tile, movement points: 40,62,0; 2000
- after the battle: Red's hero's tile and movement points; the owner; a town screen (yes/no); the garrison: 40,62,0; 1900; red; no; empty
- after the later step: the hero's tile and movement points; a town screen (yes/no); the town's visiting hero: 40,61,0; 1800; yes; none
- anything odd: the town (40, 61) before: owner; visitor; garrison; built: neutral; none; slot 0: 5 Centaurs; 7, 10, 23, 30, 31; the siege: the winner; the defenders at the opening; the level-ups; the messages: attacker; slot 0: 5 Centaurs; none; none; the later step: its path's marks, the target's mark; the town's owner: 1; green; red
