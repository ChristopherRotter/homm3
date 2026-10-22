# E08 batch E5a — results

> Played by the capture harness on 2026-10-22 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-5a.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus random, Show Route on; read from the game's memory.

Map file: `E08-batch-5a.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `5e02703d`.** The map's description in the scenario list starts with "Stamp 5e02703d.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-5a.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1).
4. A click on a tile shows the hero's path (arrows and a cross on the tile clicked); a second click on the same tile makes the hero go. Each walk below is one such pair of clicks on the tile named; note the colour of the target's mark, the number of marks and the hero's movement points after it (the harness reads them from memory: clients/capture/batches/E08-batch-5a.toml). A walk that does not happen: note it and go on with the next.
5. Order: day 1, Red's turn: E5-4a, E5-4b, E5-5, E5-6a, E5-6b, each as its encounter says; end the turn. Blue's turn: read E5-1's points (no hero moves); end the turn. Day 2, Red's turn: end the turn. Blue's turn: read E5-1's points again. Then the map is done: two days, no battle.
6. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E5-1 — The day's points: Logistics, Boots of Speed, Equestrian's Gloves, a creature specialist (ADV-MOV-001, ADV-MOV-002, HER-ADV-003, HER-ADV-004)

- Blue hero **E5-1a** (Orrin, specialty skill Archery): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman
- Blue hero **E5-1b** (Edric, specialty creature Griffin): attack 1, defense 1, power 1, knowledge 1; Basic Logistics; no artifacts; no spells; army 10 Pikeman
- Blue hero **E5-1c** (Sorsha, specialty creature Swordsman): attack 1, defense 1, power 1, knowledge 1; Advanced Logistics; no artifacts; no spells; army 10 Pikeman
- Blue hero **E5-1d** (Tyris, specialty creature Cavalier): attack 1, defense 1, power 1, knowledge 1; Expert Logistics; no artifacts; no spells; army 10 Pikeman
- Blue hero **E5-1e** (Rion, specialty skill First Aid): attack 1, defense 1, power 1, knowledge 1; no skills; Boots of Speed (feet); no spells; army 10 Pikeman
- Blue hero **E5-1f** (Adela, specialty spell Bless): attack 1, defense 1, power 1, knowledge 1; no skills; Equestrian's Gloves (right ring); no spells; army 10 Pikeman
- Blue hero **E5-1g** (Cuthbert, specialty spell Weakness): attack 1, defense 1, power 1, knowledge 1; Expert Logistics; Boots of Speed (feet), Equestrian's Gloves (right ring); no spells; army 10 Pikeman
- Blue hero **E5-1h** (Valeska, specialty creature Archer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archer

1. Day 1, Blue's turn, before anything moves: read the movement points of Blue's eight heroes E5-1a to E5-1h (row 32; the harness, from memory). None of them moves. End the turn.
2. Day 2, Blue's turn start: read the eight heroes' points again.
3. Note each hero's worn artifacts if any is not the one its line names (a starting bonus).

**Record**

| hero | day 1 points | day 2 points |
|---|---|---|
| E5-1a (no Logistics) | 1560 | 1560 |
| E5-1b (Basic Logistics) | 1716 | 1716 |
| E5-1c (Advanced Logistics) | 1872 | 1872 |
| E5-1d (Expert Logistics) | 2028 | 2028 |
| E5-1e (Boots of Speed) | 2160 | 2160 |
| E5-1f (Equestrian's Gloves) | 1860 | 1860 |
| E5-1g (both, Expert Logistics) | 2928 | 2928 |
| E5-1h (Valeska, Archers) | 1630 | 1630 |

- anything odd: E5-1e's (the spec: Boots of Speed): Boots of Speed (feet); E5-1f's (the spec: Equestrian's Gloves): Equestrian's Gloves (right ring); E5-1g's (the spec: both): Equestrian's Gloves (right ring), Boots of Speed (feet)

## E5-4a — Angel Wings inland: a cobblestone road, grass, swamp (D-24, ADV-MOV-009, HER-ADV-005)

- Red hero **E5-4a** (Adelaide, specialty spell Frost Ring): attack 1, defense 1, power 1, knowledge 1; no skills; Angel Wings (shoulders); no spells; army 10 Pikeman

1. Day 1, Red's turn: select E5-4a (Angel Wings worn). It stands on (3, 4), the west end of a cobblestone road (columns 3 to 5 of row 4); east of the road grass, then swamp from column 8. No water is near.
2. Read its movement points (before any walk).
3. Walk 1: one step east onto (4, 4), road to road. Walk 2: onto (5, 4), road to road. Walk 3: onto (6, 4), off the road onto grass. Walk 4: onto (7, 4), grass to grass. Walk 5: onto (8, 4), grass to swamp. Walk 6: onto (9, 4), swamp to swamp. After each walk read its points.

**Record**

- points before the walks: 1560

| walk | mark | marks | points after |
|---|---|---|---|
| 1 (3, 4) to (4, 4), road | green | 1 | 1510 |
| 2 (4, 4) to (5, 4), road | green | 1 | 1460 |
| 3 (5, 4) to (6, 4), road to grass | green | 1 | 1360 |
| 4 (6, 4) to (7, 4), grass | green | 1 | 1260 |
| 5 (7, 4) to (8, 4), grass to swamp | green | 1 | 1160 |
| 6 (8, 4) to (9, 4), swamp | green | 1 | 1060 |

- anything odd: worn at the start but the war machines (the spec: Angel Wings): Angel Wings (shoulders)

## E5-4b — Angel Wings on the coast: a cobblestone road along the shore (D-24, ADV-MOV-009)

- Red hero **E5-4b** (Sanya, specialty skill Eagle Eye): attack 1, defense 1, power 1, knowledge 1; no skills; Angel Wings (shoulders); no spells; army 10 Pikeman

1. Day 1, Red's turn: select E5-4b (Angel Wings worn). It stands on (3, 8), the west end of a cobblestone road along the lake's north shore (columns 3 to 6 of row 8; the water starts on row 9).
2. Read its points. Walk 1: one step east onto (4, 8). Walk 2: onto (5, 8). After each walk read its points.

**Record**

- points before the walks: 1560

| walk | mark | marks | points after |
|---|---|---|---|
| 1 (3, 8) to (4, 8), coastal road | green | 1 | 1510 |
| 2 (4, 8) to (5, 8), coastal road | green | 1 | 1460 |

- anything odd: worn at the start but the war machines (the spec: Angel Wings): Angel Wings (shoulders)

## E5-5 — Boots of Levitation: across the lake and back with a diagonal (ADV-MOV-010, HER-ADV-005)

- Red hero **E5-5** (Loynis, specialty spell Prayer): attack 1, defense 1, power 1, knowledge 1; no skills; Boots of Levitation (feet); no spells; army 10 Pikeman

1. Day 1, Red's turn: select E5-5 (Boots of Levitation worn). It stands on sand at (8, 12), the lake's south shore; the lake covers rows 9 to 11, and (8, 8) north of it is sand too.
2. Read its points. Walk 1: click (8, 8), across the lake straight north. Read its points and the number of marks.
3. Walk 2: click (9, 12), back across the lake to the sand one column east of the start. Read its points and the number of marks.

**Record**

- points before the walks: 1560

| walk | mark | marks | points after |
|---|---|---|---|
| 1 (8, 12) to (8, 8) | green | 4 | 1110 |
| 2 (8, 8) to (9, 12) | green | 4 | 619 |

- anything odd: worn at the start but the war machines (the spec: Boots of Levitation): Boots of Levitation (feet)

## E5-6a — A Fortress army on swamp under the RoE Cursed Ground (class 21) (D-23, ADV-MOV-003, ADV-MOV-004)

- Red hero **E5-6a** (Tazar, specialty skill Armorer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Gnoll

1. Day 1, Red's turn: select E5-6a (10 Gnolls, native to swamp). It stands on (7, 18), swamp under a Cursed Ground (columns 3 to 8, rows 16 to 19); east of column 8 plain swamp.
2. Read its points. Walk 1: one step east onto (8, 18), still on the Cursed Ground. Walk 2: onto (9, 18), off it. Walk 3: onto (10, 18), plain swamp to plain swamp. After each walk read its points.

**Record**

- points before the walks: 1560

| walk | mark | marks | points after |
|---|---|---|---|
| 1 (7, 18) to (8, 18), cursed to cursed | green | 1 | 1385 |
| 2 (8, 18) to (9, 18), cursed to plain | green | 1 | 1210 |
| 3 (9, 18) to (10, 18), plain swamp | green | 1 | 1110 |

- anything odd: 

## E5-6b — The same under the SoD Cursed Ground (class 223) (D-23, ADV-MOV-003)

- Red hero **E5-6b** (Alkin, specialty creature Gorgon): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Gnoll

1. Day 1, Red's turn: select E5-6b (10 Gnolls). It stands on (7, 25), swamp under the SoD's Cursed Ground (columns 6 to 8, rows 24 to 26); east of column 8 plain swamp.
2. Read its points. Walk 1: one step east onto (8, 25), still on the Cursed Ground. Walk 2: onto (9, 25), off it. Walk 3: onto (10, 25). After each walk read its points.

**Record**

- points before the walks: 1560

| walk | mark | marks | points after |
|---|---|---|---|
| 1 (7, 25) to (8, 25), cursed to cursed | green | 1 | 1385 |
| 2 (8, 25) to (9, 25), cursed to plain | green | 1 | 1210 |
| 3 (9, 25) to (10, 25), plain swamp | green | 1 | 1110 |

- anything odd: 
