# E09 batch TWN-1 — results

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
| RF1 castle (7, 7) |   |   |   |
| RF2 rampart (15, 7) |   |   |   |
| RF3 tower (23, 7) |   |   |   |
| RF4 inferno (31, 7) |   |   |   |
| RF5 necropolis (39, 7) |   |   |   |
| RF6 dungeon (47, 7) |   |   |   |
| RF7 stronghold (55, 7) |   |   |   |
| RF8 fortress (63, 7) |   |   |   |
| BF1 conflux (7, 15) |   |   |   |
| BF2 castle (15, 15) |   |   |   |
| BF3 rampart (23, 15) |   |   |   |
| BF4 tower (31, 15) |   |   |   |
| BF5 inferno (39, 15) |   |   |   |
| BF6 necropolis (47, 15) |   |   |   |
| BF7 dungeon (55, 15) |   |   |   |
| BF8 stronghold (63, 15) |   |   |   |
| R1 fortress (7, 23) |   |   |   |
| R2 conflux (15, 23) |   |   |   |
| R3 castle (23, 23) |   |   |   |
| R4 rampart (31, 23) |   |   |   |
| B1 tower (39, 23) |   |   |   |
| B2 inferno (47, 23) |   |   |   |
| B3 necropolis (55, 23) |   |   |   |
| B4 dungeon (63, 23) |   |   |   |
| NF1 stronghold (7, 31) |   |   |   |
| NF2 fortress (15, 31) |   |   |   |
| NF3 conflux (23, 31) |   |   |   |
| NF4 castle (31, 31) |   |   |   |
| N1 rampart (39, 31) |   |   |   |
| N2 tower (47, 31) |   |   |   |
| N3 inferno (55, 31) |   |   |   |
| N4 necropolis (63, 31) |   |   |   |
| NEF dungeon (7, 39) |   |   |   |
| NE stronghold (15, 39) |   |   |   |

- every-built (+0x158) different from built anywhere (which town, which ids): 
- anything odd: 

## TWN-1-2 — A neutral town's explicitly empty custom garrison at day 1, and the drawn garrisons beside it (TD-5, TD-4, TWN-SET-004)

- Red hero **TWN-1-2** (Sylvia, specialty skill Navigation): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn, before any hero moves, in each of the three games: read the garrison (+0xE0: slot, creature, count) of the ten neutral towns, and whether every owned town's garrison is empty. This row's Red hero stands at (64, 66) and never moves.

**Record**

| town | game 1: garrison (slot: creature × count) | game 2 | game 3 |
|---|---|---|---|
| NF1 stronghold (7, 31) |   |   |   |
| NF2 fortress (15, 31) |   |   |   |
| NF3 conflux (23, 31) |   |   |   |
| NF4 castle (31, 31) |   |   |   |
| N1 rampart (39, 31) |   |   |   |
| N2 tower (47, 31) |   |   |   |
| N3 inferno (55, 31) |   |   |   |
| N4 necropolis (63, 31) |   |   |   |
| NEF dungeon (7, 39) |   |   |   |
| NE stronghold (15, 39) |   |   |   |

- the owned towns' garrisons (all 24 empty? which not): 
- anything odd: 

## TWN-1-3 — The garrison's merge: Blue's town with a lone visitor of seven stacks and a garrison of three; Red besieges (TD-7, TWN-SIE-001, TWN-SIE-002, TWN-CAP-003)

- Red hero **TWN-1-3** (Caitlin, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 20 Archangel
- Opponent: Blue Castle town (Village Hall, fort); garrison 1 Cavalier, 16 Halberdier, 10 Marksman; visiting hero: Blue hero **Visitor** (Octavia, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 4 Swordsman, 4 Centaur, 5 Archer, 5 Griffin, 5 Pikeman, 3 Monk, 10 Pikeman

1. Blue's Castle (a Fort only: no towers) has its entrance at (20, 61); Blue's hero Visitor visits it (it stands on the entrance) with seven stacks, and the town's garrison holds three. Red's hero TWN-1-3 (twenty Archangels) stands south of it at (20, 62). Before the step, read Red's hero's movement points, the town's garrison and Blue's hero's army (slot, creature, count).
2. 3a: step north onto the entrance (a siege). At the battle's opening read Blue's stacks (each one's army slot or hex, creature, count). Every Blue stack defends; Red retreats at its first stack's turn. After the battle read the town's garrison and Blue's hero's army.
3. 3b: load save 1 and step north again. At the opening read Blue's stacks again; then Red wins (auto combat, or the Archangels strike stack after stack while Blue's stacks defend; take any level-up's first offer). Right after the battle read: Red's hero's tile and movement points, the town's owner, whether a town screen opened, the town's garrison.

**Record**

| slot | before: garrison | before: Blue's army | 3a opening: Blue's stacks | 3a after: garrison | 3a after: Blue's army | 3b opening: Blue's stacks |
|---|---|---|---|---|---|---|
| 0 |   |   |   |   |   |   |
| 1 |   |   |   |   |   |   |
| 2 |   |   |   |   |   |   |
| 3 |   |   |   |   |   |   |
| 4 |   |   |   |   |   |   |
| 5 |   |   |   |   |   |   |
| 6 |   |   |   |   |   |   |

- Red's hero before the step: tile, movement points: 
- 3b after the battle: Red's hero's tile, movement points; the town's owner; a town screen (yes/no); the garrison: 
- anything odd: 

## TWN-1-4 — After a siege won: a defended neutral town, then a later step onto the gate (TWN-CAP-001, TWN-CAP-003, TWN-VIS-001)

- Red hero **TWN-1-4** (Jenova, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Archangel
- Opponent: Neutral Rampart town (the standard buildings with a fort); garrison 5 Centaur

1. After 3b, in the same turn. A neutral Rampart (the editor's standard set with a Fort; a custom garrison of 5 Centaurs) has its entrance at (40, 61); Red's hero TWN-1-4 (five Archangels) stands south of it at (40, 62). Read its movement points, then step north onto the entrance (a siege). Red wins (auto combat, or the Archangels strike).
2. Right after the battle read: Red's hero's tile and movement points, the town's owner, whether a town screen opened, the town's garrison.
3. Then step north onto the entrance again. Read the hero's tile and movement points, whether a town screen opened (close it), the town's visiting hero (+0x10).

**Record**

- Red's hero before the step: tile, movement points: 
- after the battle: Red's hero's tile and movement points; the owner; a town screen (yes/no); the garrison: 
- after the later step: the hero's tile and movement points; a town screen (yes/no); the town's visiting hero: 
- anything odd: 
