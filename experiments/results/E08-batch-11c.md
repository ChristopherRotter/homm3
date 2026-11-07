# E08 batch E11c — results

> Played by the capture harness on 2026-11-07 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-11c.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

> E08 T103c: three runs of the map (87 s each); they agree in every record but Red's starting gold (31 000, 30 800, 30 500: the starting bonus) and E11-22's level-up offers (Estates or Logistics, Archery or Tactics, Leadership or Diplomacy: the game's draws); this file is the third run.

Map file: `E08-batch-11c.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `e12e6373`.** The map's description in the scenario list starts with "Stamp e12e6373.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-11c.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1).
4. Most records are the game's memory: a hero's tile, its movement points (hero +0x4D), experience and level, a mine's record (owner, resource), Red's treasury, the messages and dialogs shown (by their text's table and line), the battles (the defender, the hero's tile as each opens, the result) and what the memory holds as the first battle of a step opens. The capture harness reads them (`clients/capture/batches/E08-batch-11c.toml`, E08 T103c). Played by hand, read the screens instead and leave the memory-only lines empty (a battle's opening: note whether the mine's flag or the stone's message came before it).
5. Tiles are (column, row) from 0 at the top left. A step: click the target once (the path's marks show), then again (the hero goes). **Space:** with the hero selected, press the space bar once.
6. Each battle against a monster by auto combat. A level-up's dialog: take the left skill.
7. Order: day 1, Red's turn: E11-20, E11-21, E11-22, E11-23. Then the map is done (Blue never plays).
8. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E11-20 — A one-way monolith's exit beside a monster: the arrival, Space on the exit, the step off into the monster's zone (ADV-MON-005, ADV-MON-006, ADV-TEL-004, ADV-TEL-008)

- Red hero **E11-20** (Edric, specialty creature Griffin): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel

1. Day 1, Red's turn, first: E11-20 stands at (3, 4) with ten Archangels. East of it the one-way monolith's entrance E20 (4, 4), blue; its one exit X20 (14, 4), blue, with a Peasant P20 at (15, 4), east of X20. Read E11-20's points.
2. Click E20 (4, 4) twice: E11-20 steps onto it. Note every battle (by auto combat; the defender) and every message, then E11-20's tile and points.
3. If E11-20 stands on X20 (14, 4): press **Space** once. Note every battle and message, then E11-20's tile and points.
4. If E11-20 stands on X20 (14, 4) and P20 is still there: click (14, 5) (south of X20, beside P20) twice: the step off the exit. Note the battle or that none came, every message, then E11-20's tile and points.

**Record**

- E11-20 before: points: 2000
- the step onto E20: battles (the defender; the result); messages: battles: none; messages: none
- after the step onto E20: E11-20's tile; points; P20 (15, 4): tile: 14,4,0; points: 1900; P20: 1 Peasant
- Space on X20: battles; messages: battles: none; messages: ADVEVENT:70; what the key did: nothing
- after Space: E11-20's tile; points: tile: 14,4,0; points: 1900
- the step off X20 to (14, 5): the battle or none (the defender; the result); messages: battles: 1. at 14,5,0: defender 1 Peasant, winner attacker; messages: GENRLTXT:7
- after the step off: E11-20's tile; points; P20 (15, 4): tile: 14,5,0; points: 1800; P20: none
- anything odd: the date at the start: month 1, week 1, day 1; the game's version: SoD; the difficulty: easy

## E11-21 — A neutral mine whose entrance a monster guards: which comes first, the battle or the flag (ADV-MON-005, ADV-MON-006, GAM-DEC-007)

- Red hero **E11-21** (Valeska, specialty creature Archer): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel

1. Day 1, Red's turn, after E11-20: a neutral crystal cavern covers (4, 12) to (6, 12), its entrance the middle tile (5, 12); a Peasant Q21 stands at (4, 13), south-west of the entrance. E11-21 stands at (6, 13), south-east of the entrance, with ten Archangels. Read its points, the mine's record (owner, resource) and Red's treasury.
2. Click the entrance (5, 12) twice: E11-21 steps onto it (a diagonal step). Note every battle in order (by auto combat; the defender) and, **as the battle opens**, the mine's owner (played by hand: whether the mine's message came before the battle); then every message after it, the mine's record, Red's treasury, and E11-21's tile and points.

**Record**

- before: E11-21's points; the mine's record (owner, resource); Red's treasury: points: 2000; the mine: record 0; owner none; resource crystal; abandoned 0; guards none; treasury: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30500 gold
- the step onto the entrance: battles in order (the defender; the hero's tile as it opens; the result): 1. at 5,12,0: defender 1 Peasant, winner attacker
- as the battle opens: the mine's record (owner): record 0; owner none; resource crystal; abandoned 0; guards none
- after it: messages; the mine's record (owner); Red's treasury: messages: MINEEVNT:4; pictures: MINEEVNT:4: 1/day; the mine: record 0; owner red; resource crystal; abandoned 0; guards none; treasury: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30500 gold
- after it: E11-21's tile; points; Q21 (4, 13): tile: 5,12,0; points: 1859; Q21: none
- anything odd: 

## E11-22 — A Learning Stone a monster guards: which comes first, the battle or the stone's experience (ADV-MON-005, ADV-MON-006, GAM-DEC-007, HER-EXP-002)

- Red hero **E11-22** (Christian, specialty creature Ballista): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel

1. Day 1, Red's turn, after E11-21: a Learning Stone at (6, 20); a Peasant R22 at (7, 20), east of it. E11-22 stands at (5, 20), west of the stone, with ten Archangels, no experience (level 1). Read its points, experience and level.
2. Click the stone (6, 20) twice: E11-22 steps onto it. Note every battle in order (by auto combat; the defender) and, **as the battle opens**, E11-22's experience and level (played by hand: whether the stone's message came before the battle); then every message and level-up after it (a level-up: the left skill), and E11-22's experience, level, tile and points.

**Record**

- before: E11-22's points; experience; level: points: 2000; experience: 0; level: 1
- the step onto the stone: battles in order (the defender; the hero's tile as it opens; the result): 1. at 6,20,0: defender 1 Peasant, winner attacker
- as the battle opens: E11-22's experience; level: experience: 0; level: 1
- after it: messages (their pictures); level-ups (the level, the two offers, the one taken): messages: ADVEVENT:143; pictures: ADVEVENT:143: 1000 experience; level-ups: level 2: Leadership or Diplomacy, took left (Leadership)
- after it: E11-22's experience; level; tile; points; R22 (7, 20): experience: 1001; level: 2; tile: 6,20,0; points: 1900; R22: none
- anything odd: 

## E11-23 — Space back and forth on a two-way pair whose exit a monster guards (ADV-MON-005, ADV-MON-006, ADV-TEL-004, ADV-TEL-008)

- Red hero **E11-23** (Rion, specialty skill First Aid): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel
- Blue hero **E11-23b** (Loynis, specialty spell Prayer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn, after E11-22: the two-way monoliths A23 (4, 28) and B23 (14, 28), green (channel 0, no other member); a Peasant N23 at (15, 28), east of B23. E11-23 stands at (3, 28), west of A23, with ten Archangels. Read its points.
2. Click A23 (4, 28) twice: E11-23 steps onto it. Note every battle and message, then E11-23's tile and points.
3. Press **Space** four times, one press at a time. After each press note every battle and message, then E11-23's tile and points.
4. The control: if E11-23 stands on B23 (14, 28) and N23 is still there, click (14, 29) (south of B23, beside N23) twice. Note the battle or that none came, then E11-23's tile and points.

**Record**

- E11-23 before: points: 2000
- the step onto A23: battles; messages; E11-23's tile; points: battles: none; messages: none; tile: 14,28,0; points: 1900

| press | battles | messages | tile after | points after |
|---|---|---|---|---|
| 1 | none | none | 4,28,0 | 1900 |
| 2 | none | none | 14,28,0 | 1900 |
| 3 | none | none | 4,28,0 | 1900 |
| 4 | none | none | 14,28,0 | 1900 |

- the control: the step to (14, 29): the battle or none; messages; E11-23's tile; points; N23 (15, 28): battles: 1. at 14,29,0: defender 1 Peasant, winner attacker; messages: none; tile: 14,29,0; points: 1800; N23: none
- anything odd: 
