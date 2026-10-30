# E08 batch E9a — results

> Played by the capture harness on 2026-10-30 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-9a.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus random, Show Route on; read from the game's memory.

> The second recorded run (E08 T79a, the fourth played), a new game right after the first (`E08-batch-9a.md`): its draws (the abandoned mines' resource and Troglodytes, the starting bonus) add to the first's; every row's outcome reads as the first's.

Map file: `E08-batch-9a.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `bfafdcac`.** The map's description in the scenario list starts with "Stamp bfafdcac.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-9a.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1), and at the start of each later turn (save 2 Blue's day 1, save 3 Red's day 2, save 4 Blue's day 2).
4. Most records are the game's memory: the mine and Lighthouse records (game +0x4E38C, 0x40 bytes each: owner +0x0, resource +0x1, abandoned +0x2, the guards' creature types +0x4 and counts +0x20, the entrance +0x3C; survey §2.1), a Shipyard's tile word (owner, boat tile), the boats' records (game +0x4E3BC, `docs/exe/structs.toml` "boat"), a hero's tile, movement points (+0x4D) and day's maximum (+0x49), the players' treasuries, the explored tiles, a battle's terrain, special field and stacks at its opening and its sides' owners and controllers (`battle_manager`). The capture harness reads them (`clients/capture/batches/E08-batch-9a.toml`, E08 T79). Played by hand, read the screens instead and leave the memory-only lines empty.
5. Tiles are (column, row) from 0 at the top left. A walk: click the target once (the path's marks show), then again (the hero goes). A visit: walk to the tile named, then click the object's entrance from there (twice).
6. Each battle by auto combat; a tactics phase is ended at once (its Start button). A level-up's dialog after a battle: take the left skill.
7. Order: day 1, Red's turn: E9-4's first reads, E9-1, E9-3, E9-4 (boards), E9-5's visit and refusal; Red's treasury; end the turn. Blue's turn: Blue's treasury, E9-2 (Leave Guards), E9-3b; Blue's treasury; end the turn. Day 2, Red's turn: Red's treasury, E9-4's reads, E9-5's use without a visit, E9-2r's two battles; end the turn. Blue's turn: Blue's treasury. Then the map is done.
8. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E9-1 — A hero visits two neutral abandoned mines (D-47, ADV-MIN-002, ADV-MON-009)

- Red hero **E9-1** (Edric, specialty creature Griffin): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 6 Archangel

1. Day 1, Red's turn: E9-1 stands at (11, 11) with six Archangels. Two abandoned mines stand north of it: A at (8, 8) (its entrance; the mine covers (7, 8) to (9, 8)) and B at (14, 8) (covering (13, 8) to (15, 8)). Read both mines' records (owner, resource, guards).
2. Walk to (7, 9), then click A's entrance (8, 8). Note the question; answer yes. At the battle's opening note the defending stacks (creature and count, by slot). Auto combat. Note the messages after the battle; read A's record again.
3. Walk to (13, 9), then click B's entrance (14, 8). As for A: the question (yes), the defending stacks, the messages, B's record.

**Record**

- before: A's record (owner, resource, guards' creature and count by slot): record 0; owner none; resource crystal; abandoned 1; guards 0: 101 Troglodytes
- before: B's record (owner, resource, guards' creature and count by slot): record 1; owner none; resource gold; abandoned 1; guards 0: 187 Troglodytes
- A: the question's text: ADVEVENT:84

| slot | creature | count |
|---|---|---|
| 0 | Troglodytes | 34 |
| 1 | Troglodytes | 34 |
| 2 | Troglodytes | 33 |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |

- A: the battle's result; the messages after it: attacker; GENRLTXT:7, ADVEVENT:85, MINEEVNT:4; GENRLTXT:7: Red; MINEEVNT:4: 1/day
- after: A's record (owner, resource, guards): record 0; owner red; resource crystal; abandoned 1; guards none
- B: the question's text: ADVEVENT:84

| slot | creature | count |
|---|---|---|
| 0 | Troglodytes | 94 |
| 1 | Infernal Troglodytes | 93 |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |

- B: the battle's result; the messages after it: attacker; ADVEVENT:85, MINEEVNT:6; MINEEVNT:6: 1000/day
- after: B's record (owner, resource, guards): record 1; owner red; resource gold; abandoned 1; guards none
- anything odd: worn at the start (the starting bonus): Skull Helmet (head); every mine record at the start (Blue's two, N): record 2; owner blue; resource crystal; abandoned 0; guards none; record 3; owner blue; resource wood; abandoned 0; guards none; record 4; owner none; resource wood; abandoned 0; guards none; A: the opening (terrain, special field): terrain subterranean, special field none, ship flag 0, blocked hexes 91, 92, 93, 128, 129; A: the sides: attacker red (human 1, here 1); defender none (human 0, here 0); A: E9-1's tile after: 8,8,0; B: the opening (terrain, special field): terrain subterranean, special field none, ship flag 0, blocked hexes 41, 42, 43, 58, 59, 60, 61, 156, 157, 158; B: the sides: attacker red (human 1, here 1); defender none (human 0, here 0); B: the level-ups: level 2: Eagle Eye or Offense, took left (Eagle Eye)

## E9-2 — An owned mine's guards: Leave Guards, the question, the battle and its field (ADV-MIN-002, GAM-BAT-001)

- Blue hero **E9-2** (Sorsha, specialty creature Swordsman): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 2 Angel, 1 Archangel, 1 Angel
- Red hero **E9-2r** (Tyris, specialty creature Cavalier): attack 5, defense 5, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Angel

1. Day 1, Blue's turn: E9-2 stands at (10, 30) with two Angels, one Archangel and one Angel (three stacks). Blue owns the crystal cavern C (entrance (7, 26); it covers (6, 26) to (8, 26)) and the sawmill S (entrance (13, 26)). Walk to (7, 27), then click C's entrance (7, 26). Note the dialog that opens; move the two Angels (the hero's first stack) into the mine's army; close it. Read C's record.
2. Walk to (13, 27), then click S's entrance (13, 26). As for C: move the Archangel (the second stack) into the mine's army; close it. Read S's record. Walk to (15, 28) and end the turn there.
3. Day 2, Red's turn: E9-2r stands at (8, 31) with ten Angels. Walk to (7, 27), then click C's entrance (7, 26). Note the question; answer yes. At the battle's opening note the terrain, the special field, the defending stacks, the defending side's owner, and whether Blue is asked to play it: a hot-seat screen naming Blue, or the battle stopping for input at a guard's turn (memory: the side's human or computer controller). Auto combat (again at a stop). Note the messages after it; read C's record.
4. Walk to (11, 28), then to (13, 27) (a leg: the original draws no path onto a tile not yet explored), then click S's entrance (13, 26). As for C: the question (yes), the opening, who plays the guards, the messages, S's record.

**Record**

- C: the dialog at Blue's visit (its title): army; GENRLTXT:710
- after Blue's day 1: C's record (owner, guards): record 2; owner blue; resource crystal; abandoned 0; guards 0: 2 Angels
- after Blue's day 1: S's record (owner, guards): record 3; owner blue; resource wood; abandoned 0; guards 0: 1 Archangel
- C, day 2: the question's text: ADVEVENT:187
- C: terrain, special field: terrain subterranean, special field none, ship flag 0, blocked hexes 112, 113, 129, 130, 131, 143, 144, 145, 160, 161, 162, 163
- C: the defending stacks (creature, count, slot): slot 0: 2 Angels
- C: the defending side's owner; whether Blue plays it (how it showed): attacker red (human 1, here 1); defender blue (human 1, here 1); attacker 10 Angels (the key left auto combat off 3 times); attacker 10 Angels (waited for an order: moves to hex 98 (toward hex 100; speed 12)); defender 2 Angels (waited for an order: attacks Angel at hex 98 from the right (cursor frame 11))
- C: the result; the messages; the record after: attacker; MINEEVNT:4; MINEEVNT:4: 1/day; record 2; owner red; resource crystal; abandoned 0; guards none
- S, day 2: the question's text: ADVEVENT:187
- S: terrain, special field: terrain grass, special field none, ship flag 0, blocked hexes 77, 78, 110, 111, 143
- S: the defending stacks (creature, count, slot): slot 0: 1 Archangel
- S: the defending side's owner; whether Blue plays it (how it showed): attacker red (human 1, here 1); defender blue (human 1, here 1); defender 1 Archangel (the key left auto combat off 3 times); defender 1 Archangel (waited for an order: moves to hex 69 (toward hex 86; speed 19)); attacker 10 Angels (waited for an order: attacks Archangel at hex 69 from the down-left (cursor frame 7))
- S: the result; the messages; the record after: attacker; MINEEVNT:0; MINEEVNT:0: 2/day; record 3; owner red; resource wood; abandoned 0; guards none
- anything odd: C's dialog before the move: the counts shown (upper, lower): none; 0: 2, 1: 1, 2: 1; C's dialog after the move: the counts shown (upper, lower): 0: 2; 1: 1, 2: 1; C: the screens passed, messages; E9-2's army after: army; none; 1: 1 Archangel, 2: 1 Angel; S's dialog after the move: the counts shown (upper, lower): 0: 1; 2: 1; S: the screens passed, messages; E9-2's army after: army; none; 2: 1 Angel; C: the stacks at the end: attacker 10 Angels | defender none; S: the stacks at the end: attacker 10 Angels | defender none

## E9-3 — A neutral unguarded sawmill: Red flags it, visits it again, Blue flags it (ADV-OWN-001, ADV-MIN-002)

- Red hero **E9-3** (Valeska, specialty creature Archer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 2 Angel
- Blue hero **E9-3b** (Christian, specialty creature Ballista): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 2 Angel

1. Day 1, Red's turn: E9-3 stands at (41, 14) with two Angels. The neutral sawmill N's entrance is (41, 10) (it covers (39, 10) to (42, 10) and (41, 9), (42, 9)). Walk to (41, 11), then click (41, 10). Note the message; read N's owner.
2. Click (41, 11) (one step south, off the entrance), then (41, 10) again. Note what opens; close it without moving anything. Walk to (38, 13).
3. Day 1, Blue's turn: E9-3b stands at (45, 14) with two Angels. Walk to (41, 11), then click (41, 10). Note any question and the message; read N's owner.
4. Day 2: Blue's wood at its turn's start against its day 1 (E9-2's two mines produce too: see below).

**Record**

- Red's first visit: the message; N's owner after: none; MINEEVNT:0; MINEEVNT:0: 2/day; record 4; owner red; resource wood; abandoned 0; guards none
- Red's second visit: what opens: army; GENRLTXT:710; none; 0: 2
- Blue's visit: a question?; the message; N's owner after: none; MINEEVNT:0; MINEEVNT:0: 2/day; record 4; owner blue; resource wood; abandoned 0; guards none
- anything odd: Red's first visit: E9-3's tile after: 41,10,0; Red's second visit: the screens passed, messages and N after: army; none; record 4; owner red; resource wood; abandoned 0; guards none; Blue's visit: the screens passed, battles: none; no

## E9-4 — Two Lighthouses of Red's at load: the sea points, their surroundings at day 1 (ADV-OWN-005, HER-ADV-009, A-33)

- Red hero **E9-4** (Ingham, specialty creature Monk): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel

1. Day 1, Red's turn, before any hero moves: Red owns two Lighthouses, at (66, 6) and (6, 66), far from every hero. Read whether these tiles are explored for Red: (66, 6), (66, 8), (66, 9), (68, 8), (66, 10), (6, 64), (6, 63), (6, 62). Read both Lighthouses' records (owner, resource).
2. E9-4 stands at (55, 30) with one Angel; an empty boat at (56, 30) on the sea east of it. Click the boat: E9-4 boards it. Read its movement points and day's maximum. End the turn (after the others).
3. Day 2, Red's turn: read E9-4's movement points and day's maximum.

**Record**

| tile | explored for Red at day 1 |
|---|---|
| (66, 6) | yes |
| (66, 8) | yes |
| (66, 9) | yes |
| (68, 8) | yes |
| (66, 10) | no |
| (6, 64) | yes |
| (6, 63) | yes |
| (6, 62) | no |

- the Lighthouses' records (owner, resource): record 5; owner red; resource Lighthouse (100); abandoned 0; guards none; record 6; owner red; resource Lighthouse (100); abandoned 0; guards none
- day 1 after boarding: E9-4's movement points and maximum: 0; 2000
- day 2 at the turn's start: E9-4's movement points and maximum: 2500; 2500
- anything odd: the date at the start: month 1, week 1, day 1; the game's version: SoD; E9-4's boarding: arrived, status, boat: yes; 0x40000; tile 56,30,0; in use 1; type 1; facing 2; owner red; captain E9-4; aboard 1; day 2: E9-4's status, boat: 0x40000; tile 56,30,0; in use 1; type 1; facing 2; owner red; captain E9-4; aboard 1

## E9-5 — Red's Shipyard: a boat bought at a visit, the refusal with a hero on the boat tile, the use without a visit (ADV-SHY-001, ADV-SHY-002, ADV-SHY-003)

- Red hero **E9-5** (Adela, specialty spell Bless): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel
- Red hero **E9-5b** (Cuthbert, specialty spell Weakness): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn: the Shipyard Y covers (32, 50) to (34, 50), its entrance (33, 50); the lake lies west of it (columns 20 to 31). Read the three tiles' words (owner, boat tile; a hero on a tile: its +0x14) and Red's treasury.
2. E9-5 stands at (35, 53) with one Angel. Walk to (33, 51), then click Y's entrance (33, 50). Note what opens; build the boat. Note the treasury and the new boat's record (tile, type, facing, owner, captain).
3. Click (32, 51) (off the entrance), then click the boat on (31, 50): E9-5 boards it and stays there, on the boat tile (boarding leaves it no points).
4. Select E9-5b (at (40, 62), far away). Click Y's west tile (32, 50). Note what the cursor shows over it and what opens; close any dialog without building. Note the treasury and the three tiles' words.
5. Day 2, Red's turn, after the treasury's read: select E9-5; sail to (29, 50). Select E9-5b; click (32, 50) again. Note the cursor and what opens; build the boat. Note the treasury, the boats' records and the three tiles' words.

**Record**

- before: the words of (32, 50), (33, 50), (34, 50); Red's treasury (wood, gold): 0xff321f00 class 87.0 (owner red, boat tile 31,50); 0xff321f00 class 87.0 (owner red, boat tile 31,50); 0xff321f00 class 87.0 (owner red, boat tile 31,50); 30; 30000
- the visit: what opens: boat; GENRLTXT:596
- after building: Red's treasury; the boat's record: 20; 29000; tile 31,50,0; in use 1; type 1; facing 2; owner red; captain none; aboard 0
- E9-5 on the boat tile, the click from E9-5b: the cursor; what opens; the treasury; the three words: type 1, frame 6; none; GENRLTXT:52; 20; 29000; 0xff321f00 class 87.0 (owner red, boat tile 31,50); 0xff321f00 class 87.0 (owner red, boat tile 31,50); 0xff321f00 class 87.0 (owner red, boat tile 31,50)
- day 2, the boat tile free, the click: the cursor; what opens: boat; GENRLTXT:596; type 1, frame 6; boat; none
- after building: Red's treasury; both boats' records; the three words: 10; 29000; tile 31,50,0; in use 1; type 1; facing 2; owner red; captain E9-5; aboard 1; tile 31,50,0; in use 1; type 1; facing 2; owner red; captain none; aboard 0; 0xff321f00 class 87.0 (owner red, boat tile 31,50); 0xff321f00 class 87.0 (owner red, boat tile 31,50); 0xff321f00 class 87.0 (owner red, boat tile 31,50)
- anything odd: boats in use at the start: 1; the visit: the screens passed and messages: boat; none; the visit: E9-5's tile after: 33,50,0; boats in use after the build: 2; E9-5's boarding: arrived, points, status, boat: yes; 0; 0x40000; tile 31,50,0; in use 1; type 1; facing 2; owner red; captain E9-5; aboard 1; the click from E9-5b: the path's marks, E9-5b's tile after: 0; 40,62,0; day 2: E9-5's sail to (29, 50): arrived, points, boat: yes; 2300; tile 31,50,0; in use 1; type 1; facing 2; owner red; captain E9-5; aboard 1; day 2's click from E9-5b: the path's marks, E9-5b's tile after; boats in use: 0; 40,62,0; 3

## E9-0 — The treasuries (E9-1's and E9-3's income)

- Red hero **E9-0** (Loynis, specialty spell Prayer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Read each player's treasury (the seven resources) where the table says. E9-0 stands at (60, 64) and never moves.

**Record**

| when | wood | mercury | ore | sulfur | crystal | gems | gold |
|---|---|---|---|---|---|---|---|
| Red, day 1, the turn's end | 20 | 15 | 30 | 15 | 15 | 15 | 29000 |
| Red, day 2, the turn's start | 20 | 15 | 30 | 15 | 16 | 15 | 30000 |
| Blue, day 1, the turn's start | 30 | 15 | 30 | 15 | 15 | 15 | 30700 |
| Blue, day 1, the turn's end | 30 | 15 | 30 | 15 | 15 | 15 | 30700 |
| Blue, day 2, the turn's start | 34 | 15 | 30 | 15 | 16 | 15 | 30700 |

- anything odd: Blue: the date: month 1, week 1, day 1; Red, day 2: the date: month 1, week 1, day 2; Red, day 2: every mine record (A, B, C, S, N): record 0; owner red; resource crystal; abandoned 1; guards none; record 1; owner red; resource gold; abandoned 1; guards none; record 2; owner blue; resource crystal; abandoned 0; guards 0: 2 Angels; record 3; owner blue; resource wood; abandoned 0; guards 0: 1 Archangel; record 4; owner blue; resource wood; abandoned 0; guards none; Blue, day 2: the date; C, S and N: month 1, week 1, day 2; record 2; owner red; resource crystal; abandoned 0; guards none; record 3; owner red; resource wood; abandoned 0; guards none; record 4; owner blue; resource wood; abandoned 0; guards none
