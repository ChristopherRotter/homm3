# E08 batch E8a — results

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
| E8-1, day 1 start |   |   |   |   |
| E8-1, after (13, 4) |   |   |   |   |
| E8-1, after (17, 4) |   |   |   |   |
| E8-1, after boarding |   |   |   |   |
| E8-1c, day 1 start |   |   |   |   |
| E8-1c, after (15, 6) |   |   |   |   |
| E8-1c, after (19, 6) |   |   |   |   |
| E8-1c, after boarding |   |   |   |   |
| E8-1, day 2 start |   |   |   |   |
| E8-1, after (21, 5) |   |   |   |   |
| E8-1, after (46, 5) |   |   |   |   |
| E8-1, after the landing |   |   |   |   |
| E8-1c, day 2 start |   |   |   |   |
| E8-1c, after (40, 6) |   |   |   |   |
| E8-1c, after the landing |   |   |   |   |

- the boats' records after boarding (E8-1's, E8-1c's: tile, in use, type, facing, owner, captain, aboard): 
- the boats' records after the landings: 
- anything odd: 

## E8-3 — Where a boat lands: an event's tile, a hole's tile and a free coastal tile (D-42, ADV-SEA-003)

- Red hero **E8-3** (Edric, specialty creature Griffin): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel

1. Day 1, Red's turn: E8-3 stands at (29, 14); its boat at (30, 14), the west end of a small sea whose notch (33, 14) has three coastal tiles: (33, 13) north (a hidden event of Red's, 100 gold), (34, 14) east (a hole) and (33, 15) south (free). Click the boat: E8-3 boards it (and has no points left).
2. Day 2, Red's turn: read E8-3's tile and movement points and Red's gold.
3. Click the hole's tile (34, 14) once and note the path's marks (none, or the tiles); click it again. Read E8-3's tile and movement points and Red's gold; note any message.
4. Then the event's tile (33, 13) the same way, then the free tile (33, 15) (each only if E8-3 is still in its boat).

**Record**

| target | the path's marks (tiles, or none) | tile after | movement points after | gold after | message |
|---|---|---|---|---|---|
| (34, 14) the hole |   |   |   |   |   |
| (33, 13) the event |   |   |   |   |   |
| (33, 15) free |   |   |   |   |   |

- day 2 start: tile, movement points, gold: 
- anything odd: 

## E8-4 — 80 points on a Favorable Winds tile: a diagonal step (93), then a straight one (66) (D-43, ADV-SEA-004)

- Red hero **E8-4** (Sorsha, specialty creature Swordsman): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel

1. Day 1, Red's turn: E8-4 stands at (19, 26); its boat at (20, 26), on open sea. Favorable Winds cover columns 31 to 36, rows 27 to 32. Click the boat: E8-4 boards it.
2. Day 2, Red's turn: read E8-4's tile and movement points (1 500 expected). Sail in legs: (25, 26), (30, 26), then the winds tiles (32, 28) and (34, 30). Read the points after each leg.
3. On (34, 30): click (35, 31) (south-east) once and note the path's marks and their colour (today's or the next day's; the tiles). Click it again **only if** the marks are the single diagonal step; otherwise leave it. Read the tile and movement points.
4. Then click (35, 30) (east) the same way. Read the tile and movement points.

**Record**

| moment | tile | movement points |
|---|---|---|
| day 2 start |   |   |
| after (25, 26) |   |   |
| after (30, 26) |   |   |
| after (32, 28) |   |   |
| after (34, 30) |   |   |
| after the click on (35, 31) |   |   |
| after the click on (35, 30) |   |   |

- the marks of the click on (35, 31) (tiles, colour; clicked again or not), and of the click on (35, 30): 
- anything odd: 

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
| E8-10, day 2 start |   |   |   |   |
| E8-10, after the landing |   |   |   |   |
| E8-10, day 3 start |   |   |   |   |
| E8-10b, day 2 start |   |   |   |   |
| E8-10b, after the landing |   |   |   |   |
| E8-10b, day 3 start |   |   |   |   |

- anything odd: 

## E8-15 — A landing on a free coastal tile a monster guards (ADV-SEA-003, GAM-BAT-001)

- Red hero **E8-15** (Caitlin, specialty resource gold): attack 5, defense 5, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Angel
- Opponent: 10 Pikeman (wandering monster, never flees)

1. Day 1, Red's turn: E8-15 stands at (49, 14) with ten Angels; its boat at (50, 14). Ten savage Pikemen stand on the land at (50, 16): they guard the coastal tile (50, 15) south of the boat. Click the boat: E8-15 boards it.
2. Day 2, Red's turn: click (50, 15) once (note the marks), then again: E8-15 lands. Note what follows (a message, the battle) and the battle's opening: terrain, special field, ship flag, blocked hexes.
3. After the battle (auto combat): read E8-15's tile, movement points and status flags, and its boat's record.

**Record**

- the path's marks to (50, 15): 
- what follows the landing (a message, the battle): 
- the opening: terrain, special field, ship flag, blocked hexes: 
- after: E8-15's tile, movement points, status flags: 
- after: the boat's record (tile, in use, facing, owner, captain, aboard): 
- anything odd: 

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
| E8-7a (wings, click) |   |   |   |   |   |   |
| E8-7b (wings, arrow key) |   |   |   |   |   |   |
| E8-7c (levitation, click) |   |   |   |   |   |   |
| E8-7d (walker, click) |   |   |   |   |   |   |

- anything odd: 

## E8-16 — Blue's hero boards the boat Red's E8-1c left at its landing (ADV-SEA-001)

- Blue hero **E8-16** (Loynis, specialty spell Prayer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. Day 2, Blue's turn (after E8-1c landed on (45, 6)): E8-16 stands at (44, 7); the boat E8-1c left lies north of it on (44, 6). Read the boat's record.
2. Click the boat: E8-16 boards it. Read the boat's record, E8-16's movement points, day's maximum and status flags.

**Record**

- the boat's record before (tile, in use, type, facing, owner, captain, aboard): 
- the boat's record after: 
- E8-16 after: movement points, day's maximum, status flags: 
- anything odd: 
