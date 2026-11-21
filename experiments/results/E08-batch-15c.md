# E08 batch E15c — results

> Played by the capture harness on 2026-11-22 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-15c.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory. Saved and loaded again in the batch by the recipe's `save` and `load` steps (saved E15c2, loaded E15c2).

Map file: `E08-batch-15c.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `0ebfde6e`.** The map's description in the scenario list starts with "Stamp 0ebfde6e.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-15c.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1) and at the start of Red's day 2 (save 2).
4. Most records are the game's memory: a hero's tile and level, movement points (hero +0x4D), spell points (+0x18), status flags (+0x105; bit 18, 0x40000, the hero in a boat), Fly (+0x112) and Water Walk (+0x116) levels; the boats' records (game +0x4E3BC, 0x28 bytes each: tile, in use +0x18, type +0x1A, facing +0x1B, owner +0x1C, captain +0x20, aboard +0x24; `docs/exe/structs.toml` "boat"); whether a tile is explored for Red; the mouse cursor's frame over a tile in a target dialog; the messages in their order; the path's marks after one click. The capture harness reads them (`clients/capture/batches/E08-batch-15c.toml`, E08 T139). Played by hand, read the screens instead (an unexplored tile is black) and leave the memory-only lines empty.
5. Tiles are (column, row) from 0 at the top left; the underground's tiles carry "level 1". A walk: click the target once (the path's marks show), then again (the hero goes). **A cast:** select the hero, click the cast button (the spell book opens), click the spell named; Scuttle Boat then waits for a tile: move the pointer over the tiles the procedure names (note the cursor over each: the spell's cursor or none) and left-click the target named. Close every message with its OK button.
6. Order: day 1, Red's turn: E15-10 (first: nothing else moves before it), E15-4, E15-11 (with E15-12), E15-9's day 1 (E15-9r, then E15-9a); end the turn. Blue's turn: E15-9's Blue step; end the turn. Day 2, Red's turn: save 2, E15-9a, then load save 2 and E15-9b. Then the map is done.
7. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E15-10 — Scuttle Boat's cursor over a hidden empty boat, a seen one, an occupied one and a hidden tree; the hidden boat sunk at Expert (A-50, ADV-SPL-004, ADV-FOG-002)

- Red hero **E15-10** (Orrin, specialty skill Archery): attack 1, defense 1, power 1, knowledge 2; Expert Water Magic; Spell Book (spell book); spells Scuttle Boat; army 10 Pikeman
- Red hero **E15-10o** (Edric, specialty creature Griffin): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. Day 1, Red's turn, **first**: E15-10 stands at (12, 12) with Expert Water Magic, west of a lake (columns 14 to 20, rows 6 to 18); E15-10o stands at (13, 17). Empty boats lie at (19, 12) (HB, unexplored for Red), (15, 8) (CB, explored) and (14, 17) (O); a tree at (21, 12) is unexplored. Read whether (15, 8), (19, 12) and (21, 12) are explored for Red, and the boats' records.
2. Select E15-10o and click O (14, 17) twice: it boards O. Read E15-10o's tile, points and status flags, O's record, and whether (19, 12) and (21, 12) are still unexplored.
3. Select E15-10. Note its spell points. Cast **Scuttle Boat**: the target dialog opens (the view centred on E15-10). Move the pointer over CB (15, 8), O (14, 17), HB (19, 12) and the tree (21, 12), in that order: note the cursor over each.
4. Left-click the tree (21, 12): note what happens (does the dialog close? a message?) and E15-10's spell points. If the dialog closed, cast Scuttle Boat again (note any message).
5. Left-click HB (19, 12). Note every message, E15-10's spell points, HB's record and the boats at (19, 12); read whether (19, 12) and (21, 12) are explored now.

**Record**

- explored before: (15, 8); (19, 12); (21, 12); the boats' records (HB, CB, O): (15, 8): yes; (19, 12): no; (21, 12): no; HB: tile 19,12,0; in use 1; type 0; facing 2; owner none; captain none; aboard 0; CB: tile 15,8,0; in use 1; type 1; facing 2; owner none; captain none; aboard 0; O: tile 14,17,0; in use 1; type 2; facing 2; owner none; captain none; aboard 0
- after boarding: E15-10o's tile; points; status flags; O's record; (19, 12) and (21, 12) explored: tile: 14,17,0; points: 0; status: 0x40000; O: tile 14,17,0; in use 1; type 2; facing 2; owner red; captain E15-10o; aboard 1; (19, 12): no; (21, 12): no
- before the cast: E15-10's spell points: 20

| the pointer over | explored | the cursor |
|---|---|---|
| CB (15, 8), seen, empty | yes | cursor 1/42, picks 15,8 (cell 12,4), accept 1 |
| O (14, 17), occupied | yes | cursor 1/0, picks 14,17 (cell 11,13), accept 0 |
| HB (19, 12), hidden, empty | no | cursor 1/42, picks 19,12 (cell 16,8), accept 1 |
| the tree (21, 12), hidden | no | cursor 1/0, picks 21,12 (cell 18,8), accept 0 |

- the click on the tree: what happens; spell points: 20,12: cursor 1/0, picks 21,12 (cell 18,8), accept 0; the dialog stays open; no message; spell points: 20; HB: tile 19,12,0; in use 1; type 0; facing 2; owner none; captain none; aboard 0
- the click on HB: messages; spell points; HB's record; what lies on (19, 12): 19,12: cursor 1/42, picks 19,12 (cell 16,8), accept 1; the dialog closed; no message; the dialog: scuttle (0x63dbd8); messages: none; spell points: 20 then 14; HB: tile 19,12,0; in use 0; type 0; facing 2; owner none; captain none; aboard 0; records on (19, 12): tile 19,12,0; in use 0; type 0; facing 2; owner none; captain none; aboard 0; the tile: 0x00000000 class 0.-1
- explored after: (19, 12); (21, 12): (19, 12): no; (21, 12): no
- anything odd: the date at the start: month 1, week 1, day 1; the game's version: SoD; the difficulty: easy; the boats at the start: HB: tile 19,12,0; in use 1; type 0; facing 2; owner none; captain none; aboard 0 | CB: tile 15,8,0; in use 1; type 1; facing 2; owner none; captain none; aboard 0 | O: tile 14,17,0; in use 1; type 2; facing 2; owner none; captain none; aboard 0 | B4: tile 52,7,0; in use 1; type 0; facing 2; owner none; captain none; aboard 0 | B5: tile 40,35,0; in use 1; type 1; facing 2; owner none; captain none; aboard 0 | B3: tile 4,50,0; in use 1; type 2; facing 2; owner none; captain none; aboard 0 | B2: tile 16,56,0; in use 1; type 0; facing 2; owner none; captain none; aboard 0 | B1: tile 38,55,1; in use 1; type 1; facing 2; owner none; captain none; aboard 0

## E15-4 — Fly and Summon Boat cast on Cursed Ground, then both in an anti-magic garrison (ADV-SPL-001, GAM-SPL-001, ADV-SPL-003)

- Red hero **E15-4** (Sorsha, specialty creature Swordsman): attack 1, defense 1, power 1, knowledge 4; Expert Water Magic; Spell Book (spell book); spells Fly, Summon Boat; army 10 Pikeman

1. Day 1, Red's turn, after E15-10: E15-4 stands at (49, 7) on Cursed Ground (columns 44 to 49, rows 6 to 9) with Expert Water Magic, beside a lake (columns 50 to 56, rows 4 to 10); an empty boat B4 lies at (52, 7). Note its points and spell points and B4's record.
2. Cast **Fly**: note every message, the spell points and the Fly level (+0x112).
3. Cast **Summon Boat**: note every message, the spell points and where B4 is (its record) and any other boat beside E15-4.
4. Walk E15-4 to Red's anti-magic garrison's entrance (49, 14) in two legs, each clicked twice: (49, 13), then (49, 14). Note the window that opens and close it. Note E15-4's tile and points.
5. In the garrison: cast **Fly**, then **Summon Boat**: after each note every message and the spell points.

**Record**

- before: points; spell points; B4's record: points: 1560; spell points: 40; B4: tile 52,7,0; in use 1; type 0; facing 2; owner none; captain none; aboard 0
- Fly on Cursed Ground: messages; spell points; the Fly level: messages: GENRLTXT:748; spell points: 40 then 40; Fly level before: -1; after: -1; the book: opened; listed: Summon Boat (place 0), Fly (place 1); after the click: message
- Summon Boat on Cursed Ground: messages; spell points; B4's record; the boats beside E15-4: messages: none; spell points: 40 then 33; B4: tile 50,6,0; in use 1; type 0; facing 2; owner none; captain none; aboard 0; beside E15-4: B4: tile 50,6,0; in use 1; type 0; facing 2; owner none; captain none; aboard 0; the book: opened; listed: Summon Boat (place 0), Fly (place 1)
- at the garrison: the window; E15-4's tile; points: after (49, 13), tile: 49,13,0; points: 960; the step onto (49, 14): dialogs: garrison; the bars as it opened: at 49,14: upper empty; lower 0: 10 Pikemen; messages: none; tile: 49,14,0; points: 860
- Fly in the garrison: messages; spell points: messages: GENRLTXT:685; spell points: 33 then 33; the book: opened; Fly level: -1; tile: 49,14,0
- Summon Boat in the garrison: messages; spell points: messages: GENRLTXT:685; spell points: 33 then 33; the book: opened; B4: tile 50,6,0; in use 1; type 0; facing 2; owner none; captain none; aboard 0
- anything odd: 

## E15-11 — Fly cast twice, Fly with Angel Wings, Fly in a boat (E15-11); Water Walk cast in a boat, then the landing and a walk on water (E15-12) (ADV-SPL-002, A-49, A-51, ADV-MOV-010)

- Red hero **E15-11** (Christian, specialty creature Ballista): attack 1, defense 1, power 1, knowledge 4; no skills; Admiral's Hat (head), Spell Book (spell book); spells Fly, Water Walk; army 10 Pikeman
- Red hero **E15-11w** (Tyris, specialty creature Cavalier): attack 1, defense 1, power 1, knowledge 2; no skills; Angel Wings (shoulders), Spell Book (spell book); spells Fly; army 10 Pikeman

1. **E15-11.** Day 1, Red's turn, after E15-4: E15-11 stands at (39, 35) with the Admiral's Hat, west of a strait (columns 40 and 41, rows 30 to 40) and of an empty boat B5 on (40, 35). Note its points, spell points, Fly and Water Walk levels.
2. Cast **Fly**: note every message, the spell points and the Fly level. Cast **Fly** again: note every message (does the book close at once?), the spell points and the level.
3. Select E15-11w (at (36, 30), wearing Angel Wings): note its spell points; cast **Fly**: note every message, the spell points and the Fly level.
4. Select E15-11 and click B5 (40, 35) twice: it boards. Note its tile, points, status flags, Fly level. Cast **Fly**: note every message, the spell points and the level.
5. **E15-12.** Still in the boat: cast **Water Walk**: note every message, the spell points and the Water Walk level.
6. Click (39, 35) twice: E15-11 lands. Note its tile, points, status flags, Fly and Water Walk levels.
7. Click (42, 36) once: note the target's mark and the marks (the tiles, in order). Click it again: note where E15-11 arrives and its points.

**Record**

- before: E15-11's points; spell points; Fly and Water Walk levels: points: 1560; spell points: 40; Fly: -1; Water Walk: -1
- Fly 1: messages; spell points; Fly level: messages: none; spell points: 40 then 20; Fly level: 0; the book: opened; listed: Summon Boat (place 0), Scuttle Boat (place 1), Water Walk (place 2), Fly (place 3); after the click: map
- Fly 2: messages; spell points; Fly level: messages: none; spell points: 20 then 20; Fly level: 0; the book: opened; after the click: map
- E15-11w's Fly (Angel Wings): spell points before; messages; after; Fly level: before: 20; messages: none; after: 20; Fly level: -1; the book: opened; after the click: map
- after boarding: tile; points; status flags; Fly level: tile: 40,35,0; points: 2807; status: 0x40000; Fly level: -1; B5: tile 40,35,0; in use 1; type 1; facing 2; owner red; captain E15-11; aboard 1
- Fly in the boat: messages; spell points; Fly level: messages: GENRLTXT:193; spell points: 20 then 20; Fly level: -1; the book: opened; after the click: message
- E15-12: Water Walk in the boat: messages; spell points; Water Walk level: messages: none; spell points: 20 then 8; Water Walk level: 0; the book: opened; after the click: map
- after the landing: tile; points; status flags; Fly and Water Walk levels: tile: 39,35,0; points: 1407; status: 0x0; Fly: -1; Water Walk: 0; B5: tile 40,35,0; in use 1; type 1; facing 6; owner red; captain E15-11; aboard 0
- the click on (42, 36): the target's mark; the marks: mark: green; marks: 3; tiles: 40,36 41,36 42,36
- after the walk: tile; points: 42,36,0; points: 986
- anything odd: E15-11 worn at the start (the starting bonus): Admiral's Hat (head); E15-11w's: Angel Wings (shoulders); E15-11's book page (the spec lists Fly and Water Walk): Summon Boat (place 0), Scuttle Boat (place 1), Water Walk (place 2), Fly (place 3)

## E15-9a — Summon Boat by a hero whose last boat lies far away underground, with Red's and Blue's empty boats nearer (D-75, ADV-SPL-003)

- Red hero **E15-9a** (Valeska, specialty creature Archer): attack 1, defense 1, power 1, knowledge 1; Expert Water Magic; Admiral's Hat (head), Spell Book (spell book); spells Summon Boat; army 10 Pikeman
- On the underground level.

1. **Day 1**, Red's turn, after E15-9r's step (E15-9b's encounter): E15-9a stands underground at (37, 55, level 1) with the Admiral's Hat and Expert Water Magic, beside an empty boat B1 on (38, 55, level 1). Click B1 twice: E15-9a boards it. Note its points. Click (37, 55) twice: it lands. Note its points and B1's record (owner, captain, tile).
2. Walk E15-9a in two legs, each clicked twice: (37, 59, level 1), then the Subterranean Gate's entrance (35, 60, level 1): it goes through to the surface's gate at (9, 62). Note its points after the first leg, then its tile and points. End Red's turn.
3. **Day 2**, Red's turn, after save 2: read every boat's record. Walk E15-9a to (4, 57) (the south shore), clicked twice. Note its tile and spell points. Cast **Summon Boat**: note every message, the spell points, which boat now lies beside E15-9a (its record: tile, owner, captain) and the other boats' records. Then load save 2 for E15-9b.

**Record**

- day 1: points after boarding; after the landing; B1's record (tile; owner; captain): after boarding: 2807; tile: 38,55,1; status: 0x40000; after the landing: 1407; tile: 37,55,1; B1: tile 38,55,1; in use 1; type 1; facing 6; owner red; captain E15-9a; aboard 0
- day 1: points after (37, 59); after the gate: tile; points: after (37, 59), tile: 37,59,1; points: 1007; after the gate, tile: 9,62,0; points: 625; messages: none

| boat, day 2 before the cast | tile (level) | in use | owner | captain | aboard |
|---|---|---|---|---|---|
| B1 | 38,55,1 | 1 | red | E15-9a | 0 |
| B2 | 16,56 | 1 | red | E15-9r | 0 |
| B3 | 4,50 | 1 | blue | E15-9blue | 0 |
| B4 | 50,6 | 1 | none | none | 0 |
| B5 | 40,35 | 1 | red | E15-11 | 0 |
| CB | 15,8 | 1 | none | none | 0 |
| HB | 19,12 | 0 | none | none | 0 |
| O | 14,17 | 1 | red | E15-10o | 1 |

- day 2: E15-9a's tile; spell points before: 4,57,0; spell points: 10; points: 655; the path: 8,63 7,62 7,61 7,60 6,59 5,58 4,57
- Summon Boat: messages; spell points; the boat beside E15-9a (which; its tile; owner; captain): messages: none; spell points: 10 then 3; beside E15-9a: B1: tile 4,56,0; in use 1; type 1; facing 6; owner red; captain E15-9a; aboard 0; the book: opened; listed: Summon Boat (place 0), Scuttle Boat (place 1)
- the other boats' tiles after: HB: tile 19,12,0; in use 0; type 0; facing 2; owner none; captain none; aboard 0 | CB: tile 15,8,0; in use 1; type 1; facing 2; owner none; captain none; aboard 0 | O: tile 14,17,0; in use 1; type 2; facing 2; owner red; captain E15-10o; aboard 1 | B4: tile 50,6,0; in use 1; type 0; facing 2; owner none; captain none; aboard 0 | B5: tile 40,35,0; in use 1; type 1; facing 6; owner red; captain E15-11; aboard 0 | B3: tile 4,50,0; in use 1; type 2; facing 0; owner blue; captain E15-9blue; aboard 0 | B2: tile 16,56,0; in use 1; type 0; facing 4; owner red; captain E15-9r; aboard 0 | B1: tile 4,56,0; in use 1; type 1; facing 6; owner red; captain E15-9a; aboard 0
- anything odd: E15-9a worn at the start (the starting bonus): Admiral's Hat (head); the date after Blue's turn: month 1, week 1, day 2; save 2: E15c2.GM2 (GENRLTXT:351); day 2's start: E15-9a's tile; points: 9,62,0; points: 1560; every boat record on day 2 before the cast: HB: tile 19,12,0; in use 0; type 0; facing 2; owner none; captain none; aboard 0 | CB: tile 15,8,0; in use 1; type 1; facing 2; owner none; captain none; aboard 0 | O: tile 14,17,0; in use 1; type 2; facing 2; owner red; captain E15-10o; aboard 1 | B4: tile 50,6,0; in use 1; type 0; facing 2; owner none; captain none; aboard 0 | B5: tile 40,35,0; in use 1; type 1; facing 6; owner red; captain E15-11; aboard 0 | B3: tile 4,50,0; in use 1; type 2; facing 0; owner blue; captain E15-9blue; aboard 0 | B2: tile 16,56,0; in use 1; type 0; facing 4; owner red; captain E15-9r; aboard 0 | B1: tile 38,55,1; in use 1; type 1; facing 6; owner red; captain E15-9a; aboard 0; E15-9a's book page (the spec lists Summon Boat): Summon Boat (place 0), Scuttle Boat (place 1)

## E15-9b — Summon Boat by a hero that never sailed: Red's nearest boat or Blue's nearer one (the boats reset); the boats' owners set by sailing first (D-75, ADV-SPL-003, ADV-SEA-003)

- Red hero **E15-9b** (Rion, specialty skill First Aid): attack 1, defense 1, power 1, knowledge 1; Expert Water Magic; Spell Book (spell book); spells Summon Boat; army 10 Pikeman
- Red hero **E15-9r** (Adela, specialty spell Bless): attack 1, defense 1, power 1, knowledge 1; no skills; Admiral's Hat (head); no spells; army 10 Pikeman
- Blue hero **E15-9blue** (Mephala, specialty skill Armorer): attack 1, defense 1, power 1, knowledge 1; no skills; Admiral's Hat (head); no spells; army 10 Pikeman

1. **Day 1**, Red's turn, after E15-11 and before E15-9a: E15-9r stands at (16, 57) with the Admiral's Hat, beside an empty boat B2 on (16, 56). Click B2 twice: it boards. Click (16, 57) twice: it lands. Note B2's record (owner, captain, tile).
2. **Blue's turn**, day 1: E15-9blue stands at (4, 49) with the Admiral's Hat, beside an empty boat B3 on (4, 50). Click B3 twice: it boards. Click (4, 49) twice: it lands. Note B3's record. End Blue's turn.
3. **Day 2**, Red's turn, after E15-9a's cast and **load save 2** (the boats as before it): E15-9b stands at (6, 57) with Expert Water Magic (it never sailed). Read the boats' records. Note its spell points. Cast **Summon Boat**: note every message, the spell points, which boat now lies beside E15-9b (its record: tile, owner, captain) and the other boats' tiles.

**Record**

- day 1: B2's record after E15-9r's landing (tile; owner; captain): after boarding, points: 2807; status: 0x40000; after the landing, tile: 16,57,0; points: 1407; B2: tile 16,56,0; in use 1; type 0; facing 4; owner red; captain E15-9r; aboard 0
- Blue's day 1: B3's record after E15-9blue's landing (tile; owner; captain): after boarding, points: 2807; status: 0x40000; after the landing, tile: 4,49,0; points: 1407; B3: tile 4,50,0; in use 1; type 2; facing 0; owner blue; captain E15-9blue; aboard 0
- day 2 after the load: the boats' records as in E15-9a's table (the same?): loaded: E15c2.GM2; the boats' records as saved: yes; the records: HB: tile 19,12,0; in use 0; type 0; facing 2; owner none; captain none; aboard 0 | CB: tile 15,8,0; in use 1; type 1; facing 2; owner none; captain none; aboard 0 | O: tile 14,17,0; in use 1; type 2; facing 2; owner red; captain E15-10o; aboard 1 | B4: tile 50,6,0; in use 1; type 0; facing 2; owner none; captain none; aboard 0 | B5: tile 40,35,0; in use 1; type 1; facing 6; owner red; captain E15-11; aboard 0 | B3: tile 4,50,0; in use 1; type 2; facing 0; owner blue; captain E15-9blue; aboard 0 | B2: tile 16,56,0; in use 1; type 0; facing 4; owner red; captain E15-9r; aboard 0 | B1: tile 38,55,1; in use 1; type 1; facing 6; owner red; captain E15-9a; aboard 0
- E15-9b: spell points before: 10; tile: 6,57,0
- Summon Boat: messages; spell points; the boat beside E15-9b (which; its tile; owner; captain): messages: none; spell points: 10 then 3; beside E15-9b: B2: tile 6,56,0; in use 1; type 0; facing 4; owner red; captain E15-9r; aboard 0; the book: opened; listed: Summon Boat (place 0)
- the other boats' tiles after: HB: tile 19,12,0; in use 0; type 0; facing 2; owner none; captain none; aboard 0 | CB: tile 15,8,0; in use 1; type 1; facing 2; owner none; captain none; aboard 0 | O: tile 14,17,0; in use 1; type 2; facing 2; owner red; captain E15-10o; aboard 1 | B4: tile 50,6,0; in use 1; type 0; facing 2; owner none; captain none; aboard 0 | B5: tile 40,35,0; in use 1; type 1; facing 6; owner red; captain E15-11; aboard 0 | B3: tile 4,50,0; in use 1; type 2; facing 0; owner blue; captain E15-9blue; aboard 0 | B2: tile 6,56,0; in use 1; type 0; facing 4; owner red; captain E15-9r; aboard 0 | B1: tile 38,55,1; in use 1; type 1; facing 6; owner red; captain E15-9a; aboard 0
- anything odd: E15-9r worn at the start (the starting bonus): Admiral's Hat (head); E15-9blue worn at the start (the starting bonus): Admiral's Hat (head); the date after the load: month 1, week 1, day 2
