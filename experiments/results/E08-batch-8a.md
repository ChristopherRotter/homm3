# E08 batch E8a — results

> Played by the capture harness on 2026-10-28 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-8a.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus random, Show Route on; read from the game's memory.
>
> The third of three runs (E08 T68a), each played whole without a stop. The first played the procedure as written; its records are these but for the random draws (the Flotsams' words; the starting bonus: gold, in the second run Spirit of Oppression on E8-7a) and E8-4's last two clicks (below). The second added the controls of E8-7 and clicked E8-4's diagonal once only; its E8-7d marks were the stale arrows of E8-7c's control (the harness then read a path while none was shown; fixed), the rest as here. Every line but those reads the same in all three.

Map file: `E08-batch-8a.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `dbdf491f`.** The map's description in the scenario list starts with "Stamp dbdf491f.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-8a.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1), and at the start of each later turn (save 2 Blue's day 1, save 3 Red's day 2, save 4 Blue's day 2, save 5 Red's day 3).
4. Most records are the game's memory: a hero's tile, movement points (+0x4D) and day's maximum (+0x49), its status flags (+0x105; bit 18, 0x40000, is the hero in a boat), the boats' records (game +0x4E3BC, 0x28 bytes each: tile, in use +0x18, type +0x1A, facing +0x1B, owner +0x1C, captain +0x20, aboard +0x24; `docs/exe/structs.toml` "boat"), an object's word on its tile, the players' treasuries, a battle's terrain, special field, ship flag (`battle_manager` +0x53C6) and blocked hexes at its opening, the path marks (batch E1a's). The capture harness reads them (`clients/capture/batches/E08-batch-8a.toml`, E08 T68). Played by hand, read the screens instead and leave the memory-only lines empty.
5. Tiles are (column, row) from 0 at the top left. A walk: click the target once (the path's marks show; note them where asked), then again (the hero goes). A walk the procedure splits into legs goes leg by leg, each leg's target clicked twice: the original draws no path onto a tile not yet explored.
6. A boat stands on water beside the hero's tile; a click on it from the land beside it boards it (the walk ends there). A click on a land tile beside a hero's boat lands it (the walk ends there).
7. Each battle by auto combat; a tactics phase is ended at once (its Start button).
8. Order: day 1, Red's turn: E8-1 (both heroes board), E8-3, E8-4, E8-10 (both board), E8-15 (boards); end the turn. Blue's turn: E8-7; end the turn. Day 2, Red's turn: E8-1's day 2 (both land), E8-3, E8-4, E8-10, E8-15; end the turn. Blue's turn: E8-16; end the turn. Day 3, Red's turn: E8-10's last reads. Then the map is done.
9. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E8-1 — The Admiral's Hat at boarding (row E8-1) and landing (row E8-2): about 50 points left after the step, and a control (D-41, ADV-SEA-002, ADV-SEA-003, HER-ADV-009)

- Red hero **E8-1** (Orrin, specialty skill Archery): attack 1, defense 1, power 1, knowledge 1; no skills; Admiral's Hat (head); no spells; army 1 Angel
- Red hero **E8-1c** (Valeska, specialty creature Archer): attack 1, defense 1, power 1, knowledge 1; no skills; Admiral's Hat (head); no spells; army 1 Angel

1. Day 1, Red's turn: E8-1 stands at (9, 4) on a swamp corridor with the Admiral's Hat and one Angel (2 000 points on land); its boat stands at (20, 4), the corridor's east end. E8-1c, the control, stands at (11, 6) the same way, its boat at (20, 6).
2. Read E8-1's tile, movement points, day's maximum and status flags. Walk it east in three legs: (13, 4), (17, 4), then the boat (20, 4). Read the points after each leg; after boarding read the tile, movement points, day's maximum, status flags and the boat's record.
3. The same for E8-1c: legs (15, 6), (19, 6), then the boat (20, 6).
4. Day 2, Red's turn: read both heroes' movement points and day's maximum at the turn's start.
5. E8-1 (row E8-2): one step south-east to (21, 5); then east along row 5 in legs: (26, 5), (31, 5), (36, 5), (41, 5), (46, 5), then the coast tile (49, 5), which lands it. Read the points after each leg; after the landing read the tile, movement points, day's maximum, status flags and its boat's record.
6. E8-1c: east along row 6 in legs: (25, 6), (30, 6), (35, 6), (40, 6), then the coast tile (45, 6), which lands it. The same reads. Leave its boat where it is: E8-16 boards it.

**Record**

| hero, moment | tile | movement points | day's maximum | status flags (hex) |
|---|---|---|---|---|
| E8-1, day 1 start | 9,4,0 | 2000 | 2000 | 0x0 |
| E8-1, after (13, 4) | 13,4,0 | 1300 | 2000 | 0x0 |
| E8-1, after (17, 4) | 17,4,0 | 600 | 2000 | 0x0 |
| E8-1, after boarding | 20,4,0 | 0 | 3000 | 0x40000 |
| E8-1c, day 1 start | 11,6,0 | 2000 | 2000 | 0x0 |
| E8-1c, after (15, 6) | 15,6,0 | 1300 | 2000 | 0x0 |
| E8-1c, after (19, 6) | 19,6,0 | 600 | 2000 | 0x0 |
| E8-1c, after boarding | 20,6,0 | 637 | 3000 | 0x40000 |
| E8-1, day 2 start | 20,4,0 | 3000 | 3000 | 0x40000 |
| E8-1, after (21, 5) | 21,5,0 | 2859 | 3000 | 0x40000 |
| E8-1, after (46, 5) | 46,5,0 | 359 | 3000 | 0x40000 |
| E8-1, after the landing | 49,5,0 | 0 | 2000 | 0x0 |
| E8-1c, day 2 start | 20,6,0 | 3000 | 3000 | 0x40000 |
| E8-1c, after (40, 6) | 40,6,0 | 1000 | 3000 | 0x40000 |
| E8-1c, after the landing | 45,6,0 | 333 | 2000 | 0x0 |

- the boats' records after boarding (E8-1's, E8-1c's: tile, in use, type, facing, owner, captain, aboard): tile 20,4,0; in use 1; type 0; facing 2; owner red; captain E8-1; aboard 1; tile 20,6,0; in use 1; type 1; facing 2; owner red; captain E8-1c; aboard 1
- the boats' records after the landings: tile 48,5,0; in use 1; type 0; facing 2; owner red; captain E8-1; aboard 0; tile 44,6,0; in use 1; type 1; facing 2; owner red; captain E8-1c; aboard 0
- anything odd: worn at the start (the starting bonus): Admiral's Hat (head); boats in use at the start: 7; E8-1's boarding: arrived on the boat's tile: yes; worn at the start (the starting bonus): Admiral's Hat (head); E8-1c's boarding: arrived on the boat's tile: yes; E8-1 after (26, 5): tile, points: 26,5,0; 2359; E8-1 after (31, 5): tile, points: 31,5,0; 1859; E8-1 after (36, 5): tile, points: 36,5,0; 1359; E8-1 after (41, 5): tile, points: 41,5,0; 859; E8-1's landing: the marks: 47,5 48,5 49,5; E8-1's landing: arrived: yes; E8-1c after (25, 6): tile, points: 25,6,0; 2500; E8-1c after (30, 6): tile, points: 30,6,0; 2000; E8-1c after (35, 6): tile, points: 35,6,0; 1500; E8-1c's landing: the marks: 41,6 42,6 43,6 44,6 45,6; E8-1c's landing: arrived: yes

## E8-3 — Where a boat lands: an event's tile, a hole's tile and a free coastal tile (D-42, ADV-SEA-003)

- Red hero **E8-3** (Edric, specialty creature Griffin): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel

1. Day 1, Red's turn: E8-3 stands at (29, 14); its boat at (30, 14), the west end of a small sea whose notch (33, 14) has three coastal tiles: (33, 13) north (a hidden event of Red's, 100 gold), (34, 14) east (a hole) and (33, 15) south (free). Click the boat: E8-3 boards it (and has no points left).
2. Day 2, Red's turn: read E8-3's tile and movement points and Red's gold.
3. Click the hole's tile (34, 14) once and note the path's marks (none, or the tiles); click it again. Read E8-3's tile and movement points and Red's gold; note any message.
4. Then the event's tile (33, 13) the same way, then the free tile (33, 15) (each only if E8-3 is still in its boat).

**Record**

| target | the path's marks (tiles, or none) | tile after | movement points after | gold after | message |
|---|---|---|---|---|---|
| (34, 14) the hole | none | 30,14,0 | 1500 | 30850 | none |
| (33, 13) the event | none | 30,14,0 | 1500 | 30850 | none |
| (33, 15) free | 31,14 32,14 33,15 | 33,15,0 | 0 | 30850 | none |

- day 2 start: tile, movement points, gold: 30,14,0; 1500; 30850
- anything odd: E8-3's boarding (tile, points, status): 30,14,0; 0; 0x40000; E8-3's status after (34, 14) the hole: 0x40000; E8-3's status after (33, 13) the event: 0x40000; E8-3's status after (33, 15) free: 0x0

## E8-4 — 80 points on a Favorable Winds tile: a diagonal step (93), then a straight one (66) (D-43, ADV-SEA-004)

- Red hero **E8-4** (Sorsha, specialty creature Swordsman): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel

1. Day 1, Red's turn: E8-4 stands at (19, 26); its boat at (20, 26), on open sea. Favorable Winds cover columns 31 to 36, rows 27 to 32. Click the boat: E8-4 boards it.
2. Day 2, Red's turn: read E8-4's tile and movement points (1 500 expected). Sail in legs: (25, 26), (30, 26), then the winds tiles (32, 28) and (34, 30). Read the points after each leg.
3. On (34, 30): click (35, 31) (south-east) once and note the path's marks and their colour (today's or the next day's; the tiles). Click it again **only if** the marks are the single diagonal step; otherwise leave it. Read the tile and movement points.
4. Then click (35, 30) (east) the same way. Read the tile and movement points.

**Record**

| moment | tile | movement points |
|---|---|---|
| day 2 start | 20,26,0 | 1500 |
| after (25, 26) | 25,26,0 | 1000 |
| after (30, 26) | 30,26,0 | 500 |
| after (32, 28) | 32,28,0 | 266 |
| after (34, 30) | 34,30,0 | 80 |
| after the click on (35, 31) | 34,30,0 | 80 |
| after the click on (35, 30) | 35,30,0 | 0 |

- the marks of the click on (35, 31) (tiles, colour; clicked again or not), and of the click on (35, 30): 35,31; red; no; 35,30; green; yes
- anything odd: E8-4's boarding (tile, points, status): 20,26,0; 0; 0x40000; E8-4's day's maximum at day 2's start: 1500; the first run clicked the diagonal (35, 31) again as the procedure says (its marks the single step, red): E8-4 stayed on (34, 30) and its 80 points became 0; the click on (35, 30) then showed the single step in red, clicked again, nothing moved (0 points); this run clicks the diagonal once only, so the straight step is tried with 80

## E8-10 — The day's sea points: Advanced Navigation and the Necklace of Ocean Guidance; the Sea Captain's Hat (HER-ADV-009, ADV-MOV-001, ADV-MOV-002, A-28)

- Red hero **E8-10** (Christian, specialty creature Ballista): attack 1, defense 1, power 1, knowledge 1; Advanced Navigation; Necklace of Ocean Guidance (neck); no spells; army 10 Pikeman
- Red hero **E8-10b** (Tyris, specialty creature Cavalier): attack 1, defense 1, power 1, knowledge 1; no skills; Sea Captain's Hat (head); no spells; army 10 Pikeman

1. Day 1, Red's turn: E8-10 stands at (39, 12) with Advanced Navigation and the Necklace of Ocean Guidance, its boat at (40, 12); E8-10b at (39, 16) with the Sea Captain's Hat, its boat at (40, 16). Both armies are ten Pikemen (1 560 points on land). Click each one's boat: both board.
2. Day 2, Red's turn: read both heroes' movement points, day's maximum and status flags at the turn's start.
3. Land each one south of its boat: E8-10 on (40, 13), E8-10b on (40, 17). Read the movement points, day's maximum and status flags after each landing.
4. Day 3, Red's turn: read both heroes' movement points and day's maximum at the turn's start.

**Record**

| hero, moment | tile | movement points | day's maximum | status flags (hex) |
|---|---|---|---|---|
| E8-10, day 2 start | 40,12,0 | 4000 | 4000 | 0x40000 |
| E8-10, after the landing | 40,13,0 | 0 | 4000 | 0x0 |
| E8-10, day 3 start | 40,13,0 | 1560 | 1560 | 0x0 |
| E8-10b, day 2 start | 40,16,0 | 2000 | 2000 | 0x40000 |
| E8-10b, after the landing | 40,17,0 | 0 | 2000 | 0x0 |
| E8-10b, day 3 start | 40,17,0 | 1560 | 1560 | 0x0 |

- anything odd: worn at the start (the starting bonus): Necklace of Ocean Guidance (neck); E8-10's boarding (tile, points, day's maximum, status): 40,12,0; 0; 1560; 0x40000; worn at the start (the starting bonus): Sea Captain's Hat (head); E8-10b's boarding (tile, points, day's maximum, status): 40,16,0; 0; 1560; 0x40000; E8-10's landing: arrived: yes; E8-10b's landing: arrived: yes

## E8-15 — A landing on a free coastal tile a monster guards (ADV-SEA-003, GAM-BAT-001)

- Red hero **E8-15** (Caitlin, specialty resource gold): attack 5, defense 5, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Angel
- Opponent: 10 Pikeman (wandering monster, never flees)

1. Day 1, Red's turn: E8-15 stands at (49, 14) with ten Angels; its boat at (50, 14). Ten savage Pikemen stand on the land at (50, 16): they guard the coastal tile (50, 15) south of the boat. Click the boat: E8-15 boards it.
2. Day 2, Red's turn: click (50, 15) once (note the marks), then again: E8-15 lands. Note what follows (a message, the battle) and the battle's opening: terrain, special field, ship flag, blocked hexes.
3. After the battle (auto combat): read E8-15's tile, movement points and status flags, and its boat's record.

**Record**

- the path's marks to (50, 15): 1; 50,15
- what follows the landing (a message, the battle): none; yes; attacker
- the opening: terrain, special field, ship flag, blocked hexes: terrain grass, special field none, ship flag 0, blocked hexes 21, 22, 23, 76, 77, 126, 127, 128, 129, 143, 144, 145, 146
- after: E8-15's tile, movement points, status flags: 50,15,0; 0; 0x0
- after: the boat's record (tile, in use, facing, owner, captain, aboard): tile 50,14,0; in use 1; type 2; facing 4; owner red; captain E8-15; aboard 0
- anything odd: E8-15's boarding (tile, points, status): 50,14,0; 0; 0x40000; E8-15's army after the battle: 10 Angels

## E8-7 — A Flotsam on the water beside the coast: a flyer's click, a flyer's arrow key, a water walker, a walker (D-45, ADV-PIK-018)

- Blue hero **E8-7a** (Adela, specialty spell Bless): attack 1, defense 1, power 1, knowledge 1; no skills; Angel Wings (shoulders); no spells; army 10 Pikeman
- Blue hero **E8-7b** (Cuthbert, specialty spell Weakness): attack 1, defense 1, power 1, knowledge 1; no skills; Angel Wings (shoulders); no spells; army 10 Pikeman
- Blue hero **E8-7c** (Adelaide, specialty spell Frost Ring): attack 1, defense 1, power 1, knowledge 1; no skills; Boots of Levitation (feet); no spells; army 10 Pikeman
- Blue hero **E8-7d** (Ingham, specialty creature Monk): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. Day 1, Blue's turn: four Blue heroes stand on the coast at (30, 41), (30, 43), (30, 45) and (30, 47), each with a Flotsam on the water east of it: E8-7a and E8-7b wear Angel Wings, E8-7c Boots of Levitation, E8-7d nothing.
2. Read each Flotsam's word on its tile and Blue's treasury (wood and gold).
3. E8-7a: click its Flotsam (31, 41) once (note the marks), then again. Read E8-7a's tile and movement points, Blue's wood and gold, the Flotsam's word; note any message.
4. E8-7b: select it and press the key that steps the selected hero east (the **right arrow**) once, no click; note whether it stepped, scrolled the view or did nothing (a scroll leaves the row open, not "no move"). The same reads.
5. E8-7c and E8-7d: click their Flotsams as E8-7a did. The same reads.

**Record**

| hero | the Flotsam's word before | the marks (or none) | tile after | movement points before / after | wood, gold before / after | the Flotsam after (word, or gone) |
|---|---|---|---|---|---|---|
| E8-7a (wings, click) | 0x00000001; Flotsam: 5 wood | none | 30,41,0 | 1560; 1560 | 30; 30600; 30; 30600 | 0x00000001; Flotsam: 5 wood |
| E8-7b (wings, arrow key) | 0x00000001; Flotsam: 5 wood | nothing | 30,43,0 | 1560; 1560 | 30; 30600; 30; 30600 | 0x00000001; Flotsam: 5 wood |
| E8-7c (levitation, click) | 0x00000002; Flotsam: 5 wood, 200 gold | none | 30,45,0 | 1560; 1560 | 30; 30600; 30; 30600 | 0x00000002; Flotsam: 5 wood, 200 gold |
| E8-7d (walker, click) | 0x00000001; Flotsam: 5 wood | none | 30,47,0 | 1560; 1560 | 30; 30600; 30; 30600 | 0x00000001; Flotsam: 5 wood |

- anything odd: worn at the start (the starting bonus): Angel Wings (shoulders); control: E8-7a's path to (32, 39) at a first click (over (31, 40) if it flies): tiles, mark: 32,39 31,40; green; worn at the start (the starting bonus): Angel Wings (shoulders); E8-7b's right arrow (stepped, scrolled or nothing): nothing; control: E8-7b's key Left (stepped, scrolled or nothing; tile after): stepped; 29,43,0; control: E8-7b's key KP_Left (stepped, scrolled or nothing; tile after): stepped; 28,43,0; worn at the start (the starting bonus): Boots of Levitation (feet); control: E8-7c's path to (32, 49) at a first click (over the water if it walks on it): tiles, mark: 31,46 32,47 32,48 32,49; green; control: E8-7d's path to (32, 49) at a first click (on land: (30, 48), (31, 49)): tiles, mark: 30,48 31,49 32,49; green

## E8-16 — Blue's hero boards the boat Red's E8-1c left at its landing (ADV-SEA-001)

- Blue hero **E8-16** (Loynis, specialty spell Prayer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. Day 2, Blue's turn (after E8-1c landed on (45, 6)): E8-16 stands at (44, 7); the boat E8-1c left lies north of it on (44, 6). Read the boat's record.
2. Click the boat: E8-16 boards it. Read the boat's record, E8-16's movement points, day's maximum and status flags.

**Record**

- the boat's record before (tile, in use, type, facing, owner, captain, aboard): tile 44,6,0; in use 1; type 1; facing 2; owner red; captain E8-1c; aboard 0
- the boat's record after: tile 44,6,0; in use 1; type 1; facing 2; owner blue; captain E8-16; aboard 1
- E8-16 after: movement points, day's maximum, status flags: 0; 1560; 0x40000
- anything odd: E8-16's boarding: arrived on the boat's tile: yes
