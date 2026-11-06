# E08 batch E11b — results

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

- before: points; day's maximum: 
- the step onto S: the question (table, line); E11-14's tile; points: 
- Space on S: what is shown (table, line); tile; points: 
- the legs: points at (10, 4); at (15, 4): 
- the step onto A: messages; tile after; points; day's maximum: 
- Space 1: messages; tile; points: 
- Space 2: messages; tile; points: 
- anything odd: 

## E11-15 — A two-way channel of four, pressed 60 times: the exits' spread (D-53, ADV-TEL-003, ADV-TEL-004, ADV-TEL-008)

- Red hero **E11-15** (Valeska, specialty creature Archer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel

1. Day 1, Red's turn, after E11-14: the orange two-way channel 1 has four monoliths: M1 (26, 4), M2 (32, 4), M3 (26, 9), M4 (32, 9). E11-15 stands at (25, 4), west of M1, with one Angel.
2. Click M1 (26, 4) twice: E11-15 steps onto it. Note any message, then its tile (which monolith) and points.
3. Press Space 60 times. After each press note E11-15's tile (which monolith). After the last, note its points.

**Record**

- the step onto M1: messages; tile after (M1 to M4); points: 

| jump | from | to (M1 to M4) |
|---|---|---|
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |

- after the 60th press: points: 
- anything odd: 

## E11-17 — Three Whirlpools pressed 60 times: a hero of the team on W2's visitable tile, an empty boat on W3 (D-57, ADV-TEL-002, ADV-TEL-003, ADV-TEL-006, ADV-TEL-008)

- Red hero **E11-17** (Tyris, specialty creature Cavalier): attack 1, defense 1, power 1, knowledge 1; no skills; Admiral's Hat (head); no spells; army 5 Archangel
- Red hero **E11-17b** (Christian, specialty creature Ballista): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel

1. Day 1, Red's turn, after E11-15. The sea: water from (14, 14) to (35, 35), a grass island (19, 22) to (21, 24). The Whirlpools: W1 (22, 15) to (24, 16), W2 (16, 23) to (18, 24), W3 (28, 19) to (30, 20). An empty boat lies on W2's (18, 24), its bottom-right tile (VCMI's visitable tile of W2), another on W3's (29, 20). Red's E11-17b stands on the island at (19, 24), east of W2's boat, with one Angel. Red's E11-17 stands on the shore at (22, 13) with five Archangels and the Admiral's Hat; its boat lies at (22, 14), south of it. Read the Whirlpools' list (its positions in order) and the word of each of the eighteen Whirlpool tiles; and the class of (18, 24) and of (29, 20).
2. E11-17b: click the boat on W2's (18, 24) twice. Note what is shown, then E11-17b's tile, points, army and whether it is in a boat, and the class of (18, 24). **If E11-17b is not in a boat on (18, 24)**, note where it is, and go on all the same.
3. E11-17: click the boat (22, 14) twice: it boards. Note its points, day's maximum and whether it is in a boat. Click W1's (22, 15) twice: it sails onto it. Note any message, then its tile (which Whirlpool; column, row), army and points.
4. Press Space 60 times. After each press note E11-17's tile (which Whirlpool; column, row). After the last, note its army and points, and E11-17b's tile.

**Record**

- at the start: the Whirlpools' list (positions in order); each Whirlpool tile's word; the class of (18, 24) and of (29, 20): 
- E11-17b's click: what is shown; tile; points; army; in a boat; the class of (18, 24) after: 
- E11-17 after the boarding: points; day's maximum; in a boat: 
- the step onto W1's (22, 15): messages; tile after (Whirlpool; column, row); army; points: 

| jump | from (Whirlpool) | to (Whirlpool) | tile |
|---|---|---|---|
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
|   |   |   |   |
|   |   |   |   |

- after the 60th press: army; points; E11-17b's tile: 
- anything odd: 

## E11-16 — A Whirlpool's draw lands on an enemy hero's tile (Blue onto E11-17); the next turns (A-37, ADV-TEL-003, ADV-TEL-006, GAM-BAT-004)

- Blue hero **E11-16** (Clancy, specialty creature Unicorn): attack 1, defense 1, power 1, knowledge 1; no skills; Admiral's Hat (head); no spells; army 1 Pikeman

1. Day 1, Blue's turn: Blue's E11-16 stands on the shore at (24, 13) with one Pikeman and the Admiral's Hat; its boat lies at (24, 14), south of it. Read E11-17's tile (Red's; where its presses left it: by hand, E11-17's last table row) and E11-17b's.
2. Click the boat (24, 14) twice: E11-16 boards it. Note its points and day's maximum. Click W1's (24, 15) twice, or (23, 15) if a Red hero stands on (24, 15): it sails onto it. Note any message, dialog or battle, then its tile (which Whirlpool; column, row).
3. Unless E11-16 now stands on a Red hero's tile, press Space until it does, at most 60 times; after each press note E11-16's tile. On the Red hero's tile: note any message, dialog or battle; both heroes' tiles; the tile's class, word and the hero it names; which hero the map draws there (a screenshot); E11-16's army and points. **If it lands on none in the 60 presses** (or E11-17 stood on no Whirlpool tile at the start), note that, leave out the shared tile's reads below and still do each step off. End Blue's turn.
4. Day 2, Red's turn: note anything shown at the turn's start; both heroes' tiles; the shared tile's class, word and hero; which hero is drawn there. Select E11-17 in the hero list: note the tile's class, word and hero then, and what the map draws there. Click a plain water tile next to E11-17 (no Whirlpool's, no boat, no hero; name it) twice: note any battle, meeting or message, then both heroes' tiles, the old tile's class, word and hero, and which hero is drawn there. End Red's turn.
5. Day 2, Blue's turn: note E11-16's tile, whether it is drawn and whether it can be selected; selected, the tile's class, word and hero. Click a plain water tile next to it (name it) twice: note any battle, meeting or message, then its tile, and the old tile's class, word and hero and what the map draws there. Then the map is done.

**Record**

- day 1, Blue, first: E11-17's tile; E11-17b's tile: 
- after the boarding: points; day's maximum: 
- the step onto W1 (which tile): messages, dialogs, battles; tile after: 

| jump | from (Whirlpool) | to (Whirlpool) | tile |
|---|---|---|---|
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
|   |   |   |   |
|   |   |   |   |

- on the Red hero's tile (or none in 60): messages, dialogs, battles; both heroes' tiles; the tile's class, word and hero; the hero drawn (a screenshot); E11-16's army; points: 
- day 2, Red, at the start: anything shown; both tiles; the tile's class, word and hero; the hero drawn: 
- E11-17 selected: the tile's class, word and hero; the hero drawn: 
- E11-17's step off (to which tile): battles, meetings, messages; both tiles after; the old tile's class, word and hero; the hero drawn there: 
- day 2, Blue: E11-16's tile; drawn; selectable; selected, the tile's class, word and hero: 
- E11-16's step off (to which tile): battles, meetings, messages; its tile after; the old tile's class, word and hero; the hero drawn there: 
- anything odd: 
