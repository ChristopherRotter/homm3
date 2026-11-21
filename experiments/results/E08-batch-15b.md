# E08 batch E15b — results

Map file: `E08-batch-15b.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `24eb46ac`.** The map's description in the scenario list starts with "Stamp 24eb46ac.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-15b.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1).
4. Most records are the game's memory: a hero's tile, movement points (hero +0x4D), spell points (+0x18), the day's Dimension Doors (+0x10D), Fly (+0x112) and Water Walk (+0x116) levels, worn artifacts; whether a tile is explored for Red; the mouse cursor's frame over a tile in a target dialog; the messages in their order; the path's marks after one click. The capture harness reads them (`clients/capture/batches/E08-batch-15b.toml`, E08 T139). Played by hand, read the screens instead (an unexplored tile is black) and leave the memory-only lines empty.
5. Tiles are (column, row) from 0 at the top left. A walk: click the target once (the path's marks show), then again (the hero goes). **A cast:** select the hero, click the cast button (the spell book opens), click the spell named; a target spell (Dimension Door) then waits for a tile: move the pointer over the tiles the procedure names (note the cursor over each: the spell's cursor or none) and left-click the target named; a right click or Escape cancels it. Close every message with its OK button.
6. Order: day 1, Red's turn: E15-14 (one click, first: nothing may move before it), E15-13, E15-8, E15-16, then E15-5, E15-6, E15-7, E15-15. Then the map is done: one day.
7. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E15-14 — Water Walk (None) facing unexplored water between two explored land tiles: the path (D-78, ADV-MOV-008, ADV-MOV-010)

- Red hero **E15-14** (Rion, specialty skill First Aid): attack 1, defense 1, power 1, knowledge 2; no skills; Spell Book (spell book); spells Water Walk; army 10 Pikeman

1. Day 1, Red's turn, **first** (nothing else moves before it): E15-14 stands at (4, 27); a water strip lies on columns 10 to 13, rows 26 to 28, unexplored (black) for Red; (9, 27) west of it is explored (E15-14's reveal) and (14, 27) east of it too (the scout E15-13s at (19, 25)). Read whether (9, 27), (10, 27), (13, 27) and (14, 27) are explored for Red.
2. Note E15-14's points and spell points. Cast **Water Walk**: note every message, the spell points and the Water Walk level (+0x116).
3. Click (14, 27) **once**: note the target's mark (green, red or none) and the marks (the tiles, in order). **Do not click again**: cancel the path (click E15-14 itself or press Escape). Note E15-14's tile.

**Record**

- explored before: (9, 27); (10, 27); (13, 27); (14, 27): 
- before the cast: points; spell points: 
- the cast: messages; spell points; the Water Walk level: 
- the click on (14, 27): the target's mark; the marks: 
- E15-14's tile after: 
- anything odd: 

## E15-13 — Fly (None, level 0) over a strip of unexplored land with a hidden tree: the path and its price (as E5-9) (A-13, ADV-QUIRK-15, ADV-MOV-008, ADV-MOV-009)

- Red hero **E15-13** (Valeska, specialty creature Archer): attack 1, defense 1, power 1, knowledge 2; no skills; Spell Book (spell book); spells Fly; army 10 Pikeman
- Red hero **E15-13s** (Adela, specialty spell Bless): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. Day 1, Red's turn, after E15-14: E15-13 stands at (4, 24), the scout E15-13s at (19, 25); columns 10 to 13 are unexplored for Red on rows 20 to 31, a tree hidden at (12, 24). Read whether (9, 24), (10, 24), (12, 24), (13, 24) and (15, 24) are explored for Red.
2. Note E15-13's points and spell points. Cast **Fly**: note every message, the spell points and the Fly level (+0x112).
3. Click (15, 24) once: note the target's mark and the marks (the tiles, in order). Click it again: note where E15-13 arrives and its points. Read which of (10, 24) to (13, 24) and (12, 23), (12, 25) are explored after.

**Record**

- explored before: (9, 24); (10, 24); (12, 24); (13, 24); (15, 24): 
- before the cast: points; spell points: 
- the cast: messages; spell points; the Fly level: 
- the click on (15, 24): the target's mark; the marks: 
- after the walk: E15-13's tile; points: 
- explored after: (10, 24) to (13, 24); (12, 23); (12, 25): 
- anything odd: 

## E15-8 — Dimension Door's cursor over explored and unexplored tiles, free and blocked; a click on a hidden tree; the door onto a hidden free tile and what it reveals (E15-8); the second door at dy +9 (E15-16) (A-53, A-54, D-76, ADV-FOG-002)

- Red hero **E15-8** (Orrin, specialty skill Archery): attack 1, defense 1, power 1, knowledge 6; Advanced Air Magic; Spell Book (spell book); spells Dimension Door; army 10 Pikeman

1. **E15-8.** Day 1, Red's turn, after E15-13: E15-8 stands at (10, 10) with Advanced Air Magic. Columns 16 to 19 are unexplored for Red on rows 6 to 14; a tree is hidden at (17, 12), a seen one at (13, 12). Read whether (13, 10), (13, 12), (17, 10) and (17, 12) are explored. Note E15-8's points, spell points and the day's doors (+0x10D).
2. Cast **Dimension Door**: the target dialog opens (the view centred on E15-8). Move the pointer over (13, 10) (seen, free), (13, 12) (the seen tree), (17, 10) (hidden, free) and (17, 12) (the hidden tree), in that order: note the cursor over each (the door's cursor or none).
3. **The hidden tree:** left-click (17, 12): note what happens (does the dialog close? a message?), E15-8's tile, points, spell points and the day's doors, and which of (16, 10) to (19, 14) are explored now.
4. If the dialog is still open: left-click (17, 10). If it closed: cast Dimension Door again (note any message) and left-click (17, 10). Note every message, E15-8's tile, points, spell points and the day's doors. Read which tiles around (17, 10) are explored: (17, 5), (22, 10), (17, 15), (16, 14), (21, 13), (22, 13).
5. **E15-16.** From (17, 10), cast **Dimension Door** again: the view centres on E15-8 at (17, 10), its map area's last cell row (y 544 to 551 on the screen, the 8 pixel rows below the last drawn row) picks the row dy +9, (17, 19). Put the pointer at x = the column of (17, 10) on the screen, y = 548: note the cursor and the tile the game picks (the harness: the dialog's tile). Left-click there. Note every message, E15-8's tile, points, spell points and the day's doors.

**Record**

- explored before: (13, 10); (13, 12); (17, 10); (17, 12): 
- before: points; spell points; the day's doors: 

| the pointer over | explored | the cursor |
|---|---|---|
| (13, 10) seen, free |   |   |
| (13, 12) the seen tree |   |   |
| (17, 10) hidden, free |   |   |
| (17, 12) the hidden tree |   |   |

- the click on the hidden tree: what happens; tile; points; spell points; the day's doors; explored in (16, 10) to (19, 14): 
- the door onto (17, 10): messages; tile; points; spell points; the day's doors: 
- explored after: (17, 5); (22, 10); (17, 15); (16, 14); (21, 13); (22, 13): 
- E15-16: (17, 19) explored before the cast; the cursor at y 548; the tile picked: 
- E15-16 after the click: messages; tile; points; spell points; the day's doors: 
- anything odd: 

## E15-5 — A hero without Air Magic on Magic Plains casts Dimension Door five times (HER-SPL-003, ADV-SPL-001, D-77)

- Red hero **E15-5** (Edric, specialty creature Griffin): attack 1, defense 1, power 1, knowledge 10; no skills; Spell Book (spell book); spells Dimension Door; army 10 Pikeman

1. Day 1, Red's turn, after E15-8 (with E15-16): E15-5 stands at (26, 3) on Magic Plains (columns 25 to 30, rows 2 to 5), with no Air Magic. Note its points, spell points and the day's doors.
2. Cast **Dimension Door** and click (29, 3). Note every message, E15-5's tile, points, spell points and the day's doors.
3. Cast it again and click (26, 3); again and click (29, 3); again and click (26, 3). After each note the same.
4. Cast it a fifth time: note every message (does the target dialog open?), the points, spell points and the day's doors. If the dialog opens, click (29, 3) and note the same.

**Record**

- before: points; spell points; the day's doors: 

| cast | messages | tile | points | spell points | the day's doors |
|---|---|---|---|---|---|
| 1 (29, 3) |   |   |   |   |   |
| 2 (26, 3) |   |   |   |   |   |
| 3 (29, 3) |   |   |   |   |   |
| 4 (26, 3) |   |   |   |   |   |
| 5 |   |   |   |   |   |

- anything odd: 

## E15-6 — Dimension Door with 1 point left, then with 0 (D-77, ADV-SPL-001)

- Red hero **E15-6** (Tyris, specialty creature Cavalier): attack 1, defense 1, power 1, knowledge 6; Advanced Air Magic; Spell Book (spell book); spells Dimension Door; army 10 Pikeman

1. Day 1, Red's turn, after E15-5: E15-6 stands at (23, 30) on a swamp tile with Advanced Air Magic. Note its points and spell points.
2. Walk in four legs, each target clicked twice: (24, 31) (the sand tile, one diagonal step), (25, 32) (one diagonal step), (30, 32) (five steps east), then (33, 32) (three steps east; the original draws no path onto a tile not yet explored). Note the points after each leg.
3. Cast **Dimension Door** and click (33, 28). Note every message, E15-6's tile, points, spell points and the day's doors.
4. Cast it again and click (33, 24). Note the same.
5. Cast it a third time: note every message (does the target dialog open?), the points, spell points and the day's doors.

**Record**

- before: points; spell points: 
- the points after the legs: (24, 31); (25, 32); (30, 32); (33, 32): 
- door 1 (33, 28): messages; tile; points; spell points; the day's doors: 
- door 2 (33, 24): messages; tile; points; spell points; the day's doors: 
- door 3: messages; the dialog; points; spell points; the day's doors: 
- anything odd: 

## E15-7 — Dimension Door from land onto a water tile: the failure and what it spends (A-52, ADV-SPL-001)

- Red hero **E15-7** (Christian, specialty creature Ballista): attack 1, defense 1, power 1, knowledge 5; no skills; Spell Book (spell book); spells Dimension Door; army 10 Pikeman

1. Day 1, Red's turn, after E15-6: E15-7 stands at (29, 10), west of a pond (columns 30 to 33, rows 9 to 11). Note its points, spell points and the day's doors.
2. Cast **Dimension Door**: note the cursor over (31, 10) (water). Left-click (31, 10). Note every message, E15-7's tile, points, spell points and the day's doors.
3. Cast it again: note every message (does the target dialog open?); if it opens, click (27, 10) (land) and note the same.

**Record**

- before: points; spell points; the day's doors: 
- the cursor over (31, 10): 
- after the click: messages; tile; points; spell points; the day's doors: 
- the second cast: messages; the dialog; tile; points; spell points; the day's doors: 
- anything odd: 

## E15-15 — A hero without a spell book wearing a Dimension Door scroll: the cast button and the book (D-79, O-1, ADV-STUB-21, HER-BAT-011)

- Red hero **E15-15** (Sorsha, specialty creature Swordsman): attack 1, defense 1, power 1, knowledge 3; no skills; no artifacts; no spells; army 10 Pikeman

1. Day 1, Red's turn, after E15-7: E15-15 stands at (4, 4) with no spell book and no artifact; a Spell Scroll of Dimension Door lies at (5, 4). Note its points, spell points and worn artifacts. Click (5, 4) twice: E15-15 takes the scroll (from its tile). Note every message, its tile, points and worn artifacts (where the scroll is).
2. Note whether the cast button is enabled. Click it: note what opens (a book? which spells it lists? an empty book? a message?). If Dimension Door is listed, click it, then click (4, 8): note every message, E15-15's tile, points, spell points and the day's doors. Close whatever is open.

**Record**

- before: points; spell points; worn artifacts: 
- the scroll: messages; tile; points; where the scroll is worn: 
- the cast button: enabled?; what opens; the spells listed: 
- the cast (if listed): messages; tile; points; spell points; the day's doors: 
- anything odd: 
