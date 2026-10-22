# E08 batch E5b — results

> Played by the capture harness on 2026-10-22 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-5b.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus random, Show Route on; read from the game's memory.

Map file: `E08-batch-5b.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `7f27dfab`.** The map's description in the scenario list starts with "Stamp 7f27dfab.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-5b.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1).
4. A click on a tile shows the hero's path (arrows and a cross on the tile clicked); a second click on the same tile makes the hero go. Each walk below is one such pair of clicks on the tile named; note the colour of the target's mark (green, red, or none when no path shows), the number of marks, whether the hero arrived and its movement points after it (the harness reads them from memory: clients/capture/batches/E08-batch-5b.toml). Whether a tile is explored for Red shows only in the game's memory (the harness reads it); by hand, whether the tile is black.
5. Order: day 1, Red's turn: E5-9 first (nothing else may move before it: its strip must stay unexplored), then E5-7a, E5-7b and E5-11, each as its encounter says. Then the map is done: one day, no battle. Blue's hero never moves.
6. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E5-9 — A flight over a strip of unexplored land (A-13, ADV-MOV-008, ADV-MOV-009)

- Red hero **E5-9** (Sorsha, specialty creature Swordsman): attack 1, defense 1, power 1, knowledge 1; no skills; Angel Wings (shoulders); no spells; army 10 Pikeman
- Red hero **E5-9w** (Tyris, specialty creature Cavalier): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman
- Red hero **E5-9s** (Rion, specialty skill First Aid): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. Day 1, Red's turn, before any other hero moves: E5-9 (Angel Wings worn) stands on (5, 24), the walker E5-9w (no artifact) on (5, 26), the scout E5-9s on (21, 24). Each explored around itself at the start (radius 5); between them columns 11 to 15 stay unexplored (black) on rows 19 to 31. Nothing else of Red's is near.
2. Read whether (13, 24), in the strip's middle, and (18, 24), near the scout, are explored for Red (the harness, from memory).
3. Control first: select E5-9w and click (18, 26) (east of the strip, explored by the scout): note the target's mark and the number of marks; click it again and note whether E5-9w moved.
4. Then select E5-9 and click (18, 24): note the target's mark and the number of marks; click it again and note where it arrived and its points.

**Record**

- (13, 24) explored before the clicks (yes/no): no
- (18, 24) explored before the clicks (yes/no): yes
- E5-9w's mark at (18, 26): none
- E5-9w's marks: 0
- did E5-9w move (yes/no): no
- E5-9's mark at (18, 24): green
- E5-9's marks: 13
- did E5-9 arrive (yes/no): yes
- E5-9's points after: 260
- anything odd: E5-9's worn artifacts (the spec: Angel Wings): Angel Wings (shoulders); E5-9's path (its marks' tiles): 6,24 7,24 8,24 9,24 10,24 11,24 12,24 13,24 14,24 15,24 16,24 17,24 18,24

## E5-7a — Angel Wings: a tree beyond a tree line, then the free tile before it (D-25, ADV-MOV-009)

- Red hero **E5-7a** (Orrin, specialty skill Archery): attack 1, defense 1, power 1, knowledge 1; no skills; Angel Wings (shoulders); no spells; army 10 Pikeman

1. Day 1, Red's turn, after E5-9: select E5-7a (Angel Wings worn) on (7, 8). Two rows north of it a line of trees (row 6, columns 3 to 11); beyond it the free tile (7, 5) and a tree on (7, 4).
2. Walk 1: click the tree on (7, 4): note the target's mark and the number of marks; click it again and note whether the hero moved.
3. Walk 2: click (7, 5): note the mark and the number of marks; click it again, note where the hero arrived and its points.

**Record**

- points before the walks: 1560

| walk | mark | marks | arrived | hero's tile | points after |
|---|---|---|---|---|---|
| 1 the tree on (7, 4) | none | 0 | no | 7,8,0 | 1560 |
| 2 the free tile (7, 5) | green | 3 | yes | 7,5,0 | 1260 |

- anything odd: worn at the start but the war machines (the spec: Angel Wings): Angel Wings (shoulders); walk 2's path (its marks' tiles): 7,5 7,6 7,7

## E5-7b — Boots of Levitation: a water tile, then the shore beyond the lake (D-25, ADV-MOV-010)

- Red hero **E5-7b** (Edric, specialty creature Griffin): attack 1, defense 1, power 1, knowledge 1; no skills; Boots of Levitation (feet); no spells; army 10 Pikeman

1. Day 1, Red's turn: select E5-7b (Boots of Levitation worn) on (25, 7), the lake's south shore (the lake covers rows 4 to 6, columns 20 to 30).
2. Walk 1: click (25, 5), water in the lake's middle: note the target's mark and the number of marks; click it again and note whether the hero moved.
3. Walk 2: click (25, 3), land beyond the lake: note the mark and the number of marks; click it again, note where the hero arrived and its points.

**Record**

- points before the walks: 1560

| walk | mark | marks | arrived | hero's tile | points after |
|---|---|---|---|---|---|
| 1 the water tile (25, 5) | none | 0 | no | 25,7,0 | 1560 |
| 2 the shore (25, 3) | green | 4 | yes | 25,3,0 | 1160 |

- anything odd: worn at the start but the war machines (the spec: Boots of Levitation): Boots of Levitation (feet); walk 2's path (its marks' tiles): 25,3 25,4 25,5 25,6

## E5-11 — Angel Wings over a light blue Border Gate whose key Red lacks (E5-11, ADV-MOV-009, ADV-VIS-005)

- Red hero **E5-11** (Adela, specialty spell Bless): attack 1, defense 1, power 1, knowledge 1; no skills; Angel Wings (shoulders); no spells; army 10 Pikeman

1. Day 1, Red's turn: select E5-11 (Angel Wings worn) on (31, 16). Two rows north of it the light blue Border Gate: its entrance (31, 14) between its two blocked sides (30, 14) and (32, 14). Red holds no key.
2. Click (31, 12), the free tile two rows beyond the gate: note the target's mark, the number of marks and the tiles the path crosses on row 14; click it again, note where the hero arrived, its points and any message (a locked gate's).

**Record**

- points before the walk: 1560
- the mark at (31, 12): green
- marks: 4
- the path's tile on row 14: 31,14
- arrived (yes/no): yes
- hero's tile after: 31,12,0
- points after: 1160
- a message (its text): none
- anything odd: worn at the start but the war machines (the spec: Angel Wings): Angel Wings (shoulders); the path (its marks' tiles): 31,12 31,13 31,14 31,15

## E5-b — Blue's hero (never moves)

- Blue hero **E5-b** (Cuthbert, specialty spell Weakness): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. Blue's hero stands far from Red's; it never moves.

**Record**

- anything odd: E5-b's worn artifacts (the starting bonus): Pendant of Free Will (neck)
