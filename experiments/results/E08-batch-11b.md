# E08 batch E11b — results

> Played by the capture harness on 2026-11-07 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-11b.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

> E08 T100b: three runs of the map (the second and third with the `look` steps' reads before the scroll); this is the third. Every record but the draws agrees in all three; the draws differ from run to run (the game runs in real time between the presses), the other runs' sequences are in T100b's record in `docs/epics/E08-log.md`. The dumps are described in E11-16's last line (the three runs alike).

Map file: `E08-batch-11b.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `547aeb9d`.** The map's description in the scenario list starts with "Stamp 547aeb9d.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-11b.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1), and at the start of each later turn (save 2 Blue's day 1, save 3 Red's day 2, save 4 Blue's day 2).
4. Most records are the game's memory: a hero's tile (column, row), its movement points (hero +0x4D), the day's maximum (+0x49) and army (slot, creature, count), whether it is in a boat (its status +0x105, bit 18, 0x40000), a tile's class (its map_item record +0x1E) and the hero there, the Whirlpools' list (game +0x4E77C: its packed positions in order) and each Whirlpool tile's word (map_item +0x0), the messages and dialogs shown (by their text's table and line), a battle's opening (terrain, sides, stacks). The capture harness reads them (`clients/capture/batches/E08-batch-11b.toml`, E08 T100). Played by hand, read the screens instead and leave the memory-only lines empty.
5. Tiles are (column, row) from 0 at the top left. A walk: click the target once (the path's marks show), then again (the hero goes). A step onto a monolith, a dwelling or a Whirlpool: click its tile from the neighbouring tile named (twice). A boat: click it from the shore beside it (twice): the hero boards it and its walk ends there. **Space:** with the hero selected, press the space bar once; a row's "presses" are that, one at a time, with the reads after each.
6. A battle between Red's and Blue's heroes (none is expected): play it, Red's stacks attacking, Blue's Pikeman only waiting or defending (never a retreat or a surrender). A level-up's dialog after a battle: take the left skill.
7. Order: day 1, Red's turn: E11-14, E11-15, E11-17 (E11-17b's boarding, then E11-17's presses); end the turn. Blue's turn: E11-16's steps 1 to 3; end the turn. Day 2, Red's turn: E11-16's step 4; end the turn. Blue's turn: E11-16's step 5. Then the map is done.
8. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E11-14 — The space key at an owned dwelling, then at a two-way monolith with 0 points (ADV-TEL-008, ADV-DWL-004, ADV-TEL-004, ADV-MOV-006)

- Red hero **E11-14** (Edric, specialty creature Griffin): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Dendroid Guard

1. Day 1, Red's turn, first: Red owns the Barracks S (entrance (5, 4); it covers (4, 4) and (5, 4)). E11-14 stands at (5, 5), south of S, with five Dendroid Guards. The green two-way channel 0 has two monoliths only: A (19, 4) and B (12, 9). Read E11-14's points and day's maximum.
2. Click S's entrance (5, 4) twice: E11-14 steps onto it. Note the question (its text's table and line); answer **No**. Note E11-14's tile and points. Press Space once: note what is shown (the question again, another, or nothing); if a question, answer **No**. Note E11-14's tile and points.
3. Walk east along row 4 in three legs: click (10, 4) twice (note the points there), then (15, 4) twice (note the points), then A (19, 4) twice: E11-14 steps onto A. Note any message, then E11-14's tile, points and day's maximum.
4. Press Space once: note any message, E11-14's tile and points. Press Space again: the same.

**Record**

- before: points; day's maximum: points: 1500; day's maximum: 1500
- the step onto S: the question (table, line); E11-14's tile; points: the question and the answer: ADVEVENT:35: no; messages: none; screens: none; tile: 5,4,0; points: 1400
- Space on S: what is shown (table, line); tile; points: the question and the answer: ADVEVENT:35: no; messages: none; screens: none; what the key did: nothing; tile: 5,4,0; points: 1400
- the legs: points at (10, 4); at (15, 4): at (10, 4): 900; tile: 10,4,0; at (15, 4): 400; tile: 15,4,0
- the step onto A: messages; tile after; points; day's maximum: messages: none; battles: none; tile: 12,9,0; place: B; points: 0; day's maximum: 1500
- Space 1: messages; tile; points: messages: none; battles: none; what the key did: stepped; tile: 19,4,0; place: A; points: 0
- Space 2: messages; tile; points: messages: none; battles: none; what the key did: stepped; tile: 12,9,0; place: B; points: 0
- anything odd: the date at the start: month 1, week 1, day 1; the game's version: SoD; the difficulty: easy; the third leg's path: 16,4 17,4 18,4 19,4

## E11-15 — A two-way channel of four, pressed 60 times: the exits' spread (D-53, ADV-TEL-003, ADV-TEL-004, ADV-TEL-008)

- Red hero **E11-15** (Valeska, specialty creature Archer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel

1. Day 1, Red's turn, after E11-14: the orange two-way channel 1 has four monoliths: M1 (26, 4), M2 (32, 4), M3 (26, 9), M4 (32, 9). E11-15 stands at (25, 4), west of M1, with one Angel.
2. Click M1 (26, 4) twice: E11-15 steps onto it. Note any message, then its tile (which monolith) and points.
3. Press Space 60 times. After each press note E11-15's tile (which monolith). After the last, note its points.

**Record**

- the step onto M1: messages; tile after (M1 to M4); points: messages: none; battles: none; tile: 32,4,0; monolith: M2; points: 1900

| jump | from | to (M1 to M4) |
|---|---|---|
| 1 | M1 | M2 |
| 2 | M2 | M3 |
| 3 | M3 | M2 |
| 4 | M2 | M4 |
| 5 | M4 | M1 |
| 6 | M1 | M4 |
| 7 | M4 | M3 |
| 8 | M3 | M4 |
| 9 | M4 | M3 |
| 10 | M3 | M4 |
| 11 | M4 | M3 |
| 12 | M3 | M4 |
| 13 | M4 | M2 |
| 14 | M2 | M3 |
| 15 | M3 | M4 |
| 16 | M4 | M2 |
| 17 | M2 | M3 |
| 18 | M3 | M1 |
| 19 | M1 | M3 |
| 20 | M3 | M1 |
| 21 | M1 | M2 |
| 22 | M2 | M1 |
| 23 | M1 | M3 |
| 24 | M3 | M2 |
| 25 | M2 | M4 |
| 26 | M4 | M3 |
| 27 | M3 | M1 |
| 28 | M1 | M2 |
| 29 | M2 | M4 |
| 30 | M4 | M1 |
| 31 | M1 | M2 |
| 32 | M2 | M4 |
| 33 | M4 | M2 |
| 34 | M2 | M1 |
| 35 | M1 | M4 |
| 36 | M4 | M2 |
| 37 | M2 | M4 |
| 38 | M4 | M2 |
| 39 | M2 | M1 |
| 40 | M1 | M4 |
| 41 | M4 | M3 |
| 42 | M3 | M2 |
| 43 | M2 | M4 |
| 44 | M4 | M1 |
| 45 | M1 | M2 |
| 46 | M2 | M4 |
| 47 | M4 | M2 |
| 48 | M2 | M3 |
| 49 | M3 | M2 |
| 50 | M2 | M1 |
| 51 | M1 | M3 |
| 52 | M3 | M1 |
| 53 | M1 | M4 |
| 54 | M4 | M2 |
| 55 | M2 | M1 |
| 56 | M1 | M2 |
| 57 | M2 | M1 |
| 58 | M1 | M2 |
| 59 | M2 | M3 |
| 60 | M3 | M1 |
| 61 | M1 | M3 |

- after the 60th press: points: points: 1900; day's maximum: 2000; presses: 60 presses
- anything odd: 

## E11-17 — Three Whirlpools pressed 60 times: a hero of the team on W2's visitable tile, an empty boat on W3 (D-57, ADV-TEL-002, ADV-TEL-003, ADV-TEL-006, ADV-TEL-008)

- Red hero **E11-17** (Tyris, specialty creature Cavalier): attack 1, defense 1, power 1, knowledge 1; no skills; Admiral's Hat (head); no spells; army 5 Archangel
- Red hero **E11-17b** (Christian, specialty creature Ballista): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel

1. Day 1, Red's turn, after E11-15. The sea: water from (14, 14) to (35, 35), a grass island (19, 22) to (21, 24). The Whirlpools: W1 (22, 15) to (24, 16), W2 (16, 23) to (18, 24), W3 (28, 19) to (30, 20). An empty boat lies on W2's (18, 24), its bottom-right tile (VCMI's visitable tile of W2), another on W3's (29, 20). Red's E11-17b stands on the island at (19, 24), east of W2's boat, with one Angel. Red's E11-17 stands on the shore at (22, 13) with five Archangels and the Admiral's Hat; its boat lies at (22, 14), south of it. Read the Whirlpools' list (its positions in order) and the word of each of the eighteen Whirlpool tiles; and the class of (18, 24) and of (29, 20).
2. E11-17b: click the boat on W2's (18, 24) twice. Note what is shown, then E11-17b's tile, points, army and whether it is in a boat, and the class of (18, 24). **If E11-17b is not in a boat on (18, 24)**, note where it is, and go on all the same.
3. E11-17: click the boat (22, 14) twice: it boards. Note its points, day's maximum and whether it is in a boat. Click W1's (22, 15) twice: it sails onto it. Note any message, then its tile (which Whirlpool; column, row), army and points.
4. Press Space 60 times. After each press note E11-17's tile (which Whirlpool; column, row). After the last, note its army and points, and E11-17b's tile.

**Record**

- at the start: the Whirlpools' list (positions in order); each Whirlpool tile's word; the class of (18, 24) and of (29, 20): the list: 16 tiles: 22,15 23,15 24,15 22,16 23,16 24,16 28,19 29,19 30,19 28,20 30,20 16,23 17,23 18,23 16,24 17,24; W1: 22,15 0x00000000 class 111.0, 23,15 0x00000000 class 111.0, 24,15 0x00000000 class 111.0, 22,16 0x00000000 class 111.0, 23,16 0x00000000 class 111.0, 24,16 0x00000000 class 111.0; W2: 16,23 0x00000002 class 111.0, 17,23 0x00000002 class 111.0, 18,23 0x00000002 class 111.0, 16,24 0x00000002 class 111.0, 17,24 0x00000002 class 111.0, 18,24 0x00000002 class 8.0; W3: 28,19 0x00000001 class 111.0, 29,19 0x00000001 class 111.0, 30,19 0x00000001 class 111.0, 28,20 0x00000001 class 111.0, 29,20 0x00000003 class 8.0, 30,20 0x00000001 class 111.0; (18, 24): 0x00000002 class 8.0; (29, 20): 0x00000003 class 8.0
- E11-17b's click: what is shown; tile; points; army; in a boat; the class of (18, 24) after: the path: 18,24; clicked again: yes; messages: none; questions: none; screens: none; battles: none; tile: 18,24,0; points: 0; army: 0: 1 Angel; in a boat: yes; (18, 24): 0xffffffff class 111.0
- E11-17 after the boarding: points; day's maximum; in a boat: points: 2850; day's maximum: 3000; in a boat: yes; tile: 22,14,0
- the step onto W1's (22, 15): messages; tile after (Whirlpool; column, row); army; points: messages: none; battles: none; tile: W2; 17,23; army: 0: 5 Archangels; points: 2750

| jump | from (Whirlpool) | to (Whirlpool) | tile |
|---|---|---|---|
| 1 | W1 | W2 | 17,23 |
| 2 | W2 | W1 | 24,16 |
| 3 | W1 | W2 | 16,24 |
| 4 | W2 | W3 | 28,20 |
| 5 | W3 | W1 | 24,16 |
| 6 | W1 | W2 | 17,23 |
| 7 | W2 | W3 | 30,20 |
| 8 | W3 | W2 | 17,24 |
| 9 | W2 | W1 | 24,15 |
| 10 | W1 | W3 | 28,19 |
| 11 | W3 | W1 | 24,15 |
| 12 | W1 | W3 | 30,19 |
| 13 | W3 | W1 | 23,16 |
| 14 | W1 | W2 | 16,24 |
| 15 | W2 | W3 | 30,19 |
| 16 | W3 | W1 | 22,15 |
| 17 | W1 | W3 | 30,19 |
| 18 | W3 | W1 | 23,15 |
| 19 | W1 | W3 | 30,20 |
| 20 | W3 | W1 | 24,16 |
| 21 | W1 | W2 | 17,23 |
| 22 | W2 | W3 | 30,19 |
| 23 | W3 | W1 | 22,16 |
| 24 | W1 | W3 | 30,19 |
| 25 | W3 | W1 | 22,15 |
| 26 | W1 | W3 | 30,19 |
| 27 | W3 | W2 | 17,23 |
| 28 | W2 | W1 | 24,15 |
| 29 | W1 | W2 | 17,24 |
| 30 | W2 | W3 | 28,19 |
| 31 | W3 | W1 | 22,16 |
| 32 | W1 | W2 | 17,23 |
| 33 | W2 | W1 | 24,15 |
| 34 | W1 | W3 | 29,19 |
| 35 | W3 | W1 | 24,15 |
| 36 | W1 | W2 | 17,24 |
| 37 | W2 | W3 | 28,20 |
| 38 | W3 | W2 | 17,24 |
| 39 | W2 | W3 | 28,19 |
| 40 | W3 | W2 | 17,23 |
| 41 | W2 | W1 | 23,16 |
| 42 | W1 | W2 | 16,24 |
| 43 | W2 | W1 | 24,15 |
| 44 | W1 | W3 | 29,19 |
| 45 | W3 | W2 | 16,23 |
| 46 | W2 | W1 | 22,16 |
| 47 | W1 | W2 | 16,23 |
| 48 | W2 | W3 | 28,19 |
| 49 | W3 | W1 | 24,16 |
| 50 | W1 | W2 | 17,23 |
| 51 | W2 | W1 | 22,15 |
| 52 | W1 | W3 | 30,19 |
| 53 | W3 | W2 | 17,23 |
| 54 | W2 | W3 | 30,20 |
| 55 | W3 | W1 | 24,16 |
| 56 | W1 | W2 | 18,23 |
| 57 | W2 | W1 | 24,16 |
| 58 | W1 | W2 | 17,23 |
| 59 | W2 | W1 | 24,16 |
| 60 | W1 | W3 | 28,20 |
| 61 | W3 | W2 | 17,23 |

- after the 60th press: army; points; E11-17b's tile: army: 0: 5 Archangels; points: 2750; day's maximum: 3000; E11-17b's tile: 18,24,0; presses: 60 presses
- anything odd: E11-17 worn at the start: Admiral's Hat (head); E11-17b's tile after the presses: 18,24: 0x00000006 class 34.0; hero E11-17b, under it 0xffffffff class 111

## E11-16 — A Whirlpool's draw lands on an enemy hero's tile (Blue onto E11-17); the next turns (A-37, ADV-TEL-003, ADV-TEL-006, GAM-BAT-004)

- Blue hero **E11-16** (Clancy, specialty creature Unicorn): attack 1, defense 1, power 1, knowledge 1; no skills; Admiral's Hat (head); no spells; army 1 Pikeman

1. Day 1, Blue's turn: Blue's E11-16 stands on the shore at (24, 13) with one Pikeman and the Admiral's Hat; its boat lies at (24, 14), south of it. Read E11-17's tile (Red's; where its presses left it: by hand, E11-17's last table row) and E11-17b's.
2. Click the boat (24, 14) twice: E11-16 boards it. Note its points and day's maximum. Click W1's (24, 15) twice, or (23, 15) if a Red hero stands on (24, 15): it sails onto it. Note any message, dialog or battle, then its tile (which Whirlpool; column, row).
3. Unless E11-16 now stands on a Red hero's tile, press Space until it does, at most 60 times; after each press note E11-16's tile. On the Red hero's tile: note any message, dialog or battle; both heroes' tiles; the tile's class, word and the hero it names; which hero the map draws there (a screenshot); E11-16's army and points. **If it lands on none in the 60 presses** (or E11-17 stood on no Whirlpool tile at the start), note that, leave out the shared tile's reads below and still do each step off. End Blue's turn.
4. Day 2, Red's turn: note anything shown at the turn's start; both heroes' tiles; the shared tile's class, word and hero; which hero is drawn there. Select E11-17 in the hero list: note the tile's class, word and hero then, and what the map draws there. Click a plain water tile next to E11-17 (no Whirlpool's, no boat, no hero; name it) twice: note any battle, meeting or message, then both heroes' tiles, the old tile's class, word and hero, and which hero is drawn there. End Red's turn.
5. Day 2, Blue's turn: note E11-16's tile, whether it is drawn and whether it can be selected; selected, the tile's class, word and hero. Click a plain water tile next to it (name it) twice: note any battle, meeting or message, then its tile, and the old tile's class, word and hero and what the map draws there. Then the map is done.

**Record**

- day 1, Blue, first: E11-17's tile; E11-17b's tile: E11-17: W2; 17,23,0; E11-17's tile: 17,23: 0x00000007 class 34.0; hero E11-17, under it 0x00000002 class 111; E11-17b: 18,24,0
- after the boarding: points; day's maximum: points: 2807; day's maximum: 3000; in a boat: yes; tile: 24,14,0
- the step onto W1 (which tile): messages, dialogs, battles; tile after: onto: 24,15; messages: none; screens: none; battles: none; tile after: W3; 28,19,0; points: 2707

| jump | from (Whirlpool) | to (Whirlpool) | tile |
|---|---|---|---|
| 1 | W1 | W3 | 28,19 |
| 2 | W3 | W2 | 17,23 |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |

- on the Red hero's tile (or none in 60): messages, dialogs, battles; both heroes' tiles; the tile's class, word and hero; the hero drawn (a screenshot); E11-16's army; points: presses: on E11-17's tile after 1 presses; messages: none; screens: none; battles: none; E11-16: 17,23,0; E11-17: 17,23,0; the tile: 17,23: 0x00000007 class 34.0; hero E11-17, under it 0x00000002 class 111; army: 0: 1 Pikeman; points: 2707; a moment later, the tile: 17,23: 0x00000007 class 34.0; hero E11-17, under it 0x00000002 class 111; the view centred on E11-16, the tile: 17,23: 0x00000016 class 34.0; hero E11-16, under it 0x00000007 class 34; drawn: the dump blue-day1-landed (the log)
- day 2, Red, at the start: anything shown; both tiles; the tile's class, word and hero; the hero drawn: shown: GENRLTXT:14, ?; selected: E11-15; E11-17: 17,23,0; E11-16: 17,23,0; E11-17's tile as the dialogs closed: 17,23: 0x00000016 class 34.0; hero E11-16, under it 0x00000007 class 34; a moment later: 17,23: 0x00000016 class 34.0; hero E11-16, under it 0x00000007 class 34; the view centred on E11-17: 17,23: 0x00000016 class 34.0; hero E11-16, under it 0x00000007 class 34; drawn: the dump red-day2-start (the log)
- E11-17 selected: the tile's class, word and hero; the hero drawn: selected: E11-17; the tile: 17,23: 0x00000002 class 111.0; drawn: the dump red-day2-selected (the log)
- E11-17's step off (to which tile): battles, meetings, messages; both tiles after; the old tile's class, word and hero; the hero drawn there: to: 17,22; battles: none; meetings: none; screens: none; messages: none; E11-17: 17,22,0; E11-16: 17,23,0; the old tile: 17,23: 0x00000002 class 111.0; drawn: the dump red-day2-off (the log)
- day 2, Blue: E11-16's tile; drawn; selectable; selected, the tile's class, word and hero: at the start, E11-16's tile: 17,23,0; selected: E11-16; its tile as the dialogs closed: 17,23: 0x00000007 class 34.0; hero E11-17, under it 0x00000000 class 0; a moment later: 17,23: 0x00000007 class 34.0; hero E11-17, under it 0x00000000 class 0; the view centred on E11-16: 17,23: 0x00000016 class 34.0; hero E11-16, under it 0x00000007 class 34; drawn: the dump blue-day2-start (the log); selectable: E11-16; selected, its tile: 17,23: 0x00000016 class 34.0; hero E11-16, under it 0x00000007 class 34; drawn: the dump blue-day2-selected (the log)
- E11-16's step off (to which tile): battles, meetings, messages; its tile after; the old tile's class, word and hero; the hero drawn there: to: 18,22; battles: none; meetings: none; screens: none; messages: none; tile after: 18,22,0; E11-17: 17,22,0; the old tile: 17,23: 0x00000007 class 34.0; hero E11-17, under it 0x00000000 class 0; drawn: the dump blue-day2-off (the log)
- anything odd: Blue's turn's start: messages: GENRLTXT:14, ?; worn at the start (the starting bonus): Admiral's Hat (head); E11-17's step off: the path: 17,22; E11-17's points after the step off: 2900; Blue's day 2: messages at the start: GENRLTXT:14, ?; E11-16's step off: the path: 18,22; E11-16's points after the step off: 2859; the agent's reading of the dumps (the view centred on the shared tile 17,23; the three runs alike): blue-day1-landed one boat there with Blue's flag (E11-16), E11-17 not drawn; red-day2-start the same; red-day2-selected one boat there with Red's flag (E11-17), E11-16 not drawn; red-day2-off E11-17's boat on 17,22, the shared tile the Whirlpool alone (E11-16 not drawn); blue-day2-start and blue-day2-selected E11-16 (Blue) on 17,23 and E11-17 (Red) on 17,22; blue-day2-off E11-16 on 18,22, and on 17,23 a boat with Red's flag besides E11-17's on 17,22 (E11-17 drawn twice); each turn's second OK (`?`) is the townless player's countdown, ARRAYTXT 128 (7 days left, then 6; its text read in a local log, never committed)
