# E08 batch E11a — results

Map file: `E08-batch-11a.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `444b7900`.** The map's description in the scenario list starts with "Stamp 444b7900.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-11a.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1), and at the start of each later turn (save 2 Blue's day 1, save 3 Red's day 2).
4. Most records are the game's memory: a hero's tile (column, row, level), its movement points (hero +0x4D), the day's maximum (+0x49) and army, a tile's explored bit for Red, the hero's path after a first click (its tiles, or none), the messages and dialogs shown (by their text's table and line), the meeting dialog, a battle's opening (terrain, sides, stacks), the adventure map's view during Replay Opponent Turn. The capture harness reads them (`clients/capture/batches/E08-batch-11a.toml`, E08 T100). Played by hand, read the screens instead (the level button under the minimap shows the underground; a tile is explored when it is not black) and leave the memory-only lines empty.
5. Tiles are (column, row) from 0 at the top left; underground tiles say so. A walk: click the target once (the path's marks show), then again (the hero goes). A step onto a monolith or a gate: click its tile from the neighbouring tile named (twice). **Space:** with the hero selected, press the space bar once.
6. Each battle against a monster by auto combat. E11-7's battle is between two human players, so auto combat is refused: play it, Red's stacks attacking, Blue's stack only waiting or defending (never a retreat or a surrender). A level-up's dialog after a battle (none is expected): take the left skill.
7. Order: day 1, Red's turn: E11-1 (rows E11-1 to E11-3), E11-4 (rows E11-5, then E11-4), E11-6, E11-9 (rows E11-9, then E11-8), E11-19; E11-7's and E11-18's first reads; end the turn. Blue's turn: E11-7's Blue step, then E11-18's; end the turn. Day 2, Red's turn: E11-18's replay and reads first, then E11-7's step and battle; end the turn. Blue's turn: end the turn. Then the map is done.
8. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E11-1 — A path past a two-way monolith (E11-1), the step onto it (E11-2), a monster beside its exit (E11-3) (D-58, ADV-OBJ-003, ADV-TEL-004, ADV-TEL-007, ADV-MOV-008, ADV-MON-005)

- Red hero **E11-1** (Edric, specialty creature Griffin): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel

1. Day 1, Red's turn, first: E11-1 stands at (8, 10) with ten Archangels. East of it the two-way monolith A (9, 10), green; a rock wall fills columns 10 and 11 from row 1 to row 19; east of the wall the green monolith B (12, 10) and a Peasant P at (13, 11), south-east of B. Read E11-1's points and day's maximum and the explored bits of the tiles (12, 11), (16, 13), (16, 14), (17, 10) and (18, 10).
2. Click B (12, 10) once: note the path shown (its tiles, or that none shows). If none shows, do not click again and go on with the next step. **If a path shows**, click B again: E11-1 walks it. Note every step, any message or battle, then E11-1's tile, points and day's maximum, whether it moved on after a jump, and the explored bits of the five tiles of step 1; then skip the next step.
3. Click A (9, 10) once: note the path shown. Click it again: E11-1 steps onto A. Note any message or battle, then E11-1's tile, points and day's maximum, and the explored bits of the five tiles of step 1.
4. If E11-1 stands on B (12, 10): click (13, 10) (east of B, beside P) twice. Note the battle (auto combat) or that none came, and E11-1's tile after.

**Record**

- E11-1 before: points; day's maximum; explored (12, 11), (16, 13), (16, 14), (17, 10), (18, 10): 
- E11-1: the path to B (12, 10) (its tiles, or none); if one showed, the walk: its steps, messages, battles: 
- E11-1: the path to A (its tiles): 
- E11-2: after the step onto A: messages; battles; E11-1's tile, points and day's maximum; moved on after the jump?: 
- E11-2: explored after: (12, 11), (16, 13), (16, 14), (17, 10), (18, 10): 
- E11-3: the battle on arrival, if any (the defender, the result): 
- E11-3's control: the step to (13, 10): the battle or none; E11-1's tile after: 
- anything odd: 

## E11-4 — A one-way exit visited (E11-5); then its entrance, whose only exit holds Red's hero (E11-4) (ADV-TEL-003, ADV-TEL-004)

- Red hero **E11-4** (Valeska, specialty creature Archer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel
- Red hero **E11-5** (Christian, specialty creature Ballista): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel

1. Day 1, Red's turn, after E11-1: one-way channel 0 (blue) has one entrance E4 (32, 10) and one exit X4 (40, 10). E11-5 stands at (39, 10), west of X4; E11-4 at (31, 10), west of E4; each with one Angel.
2. E11-5: click X4 (40, 10) twice: E11-5 steps onto it. Note the message (and close it), then E11-5's tile and points.
3. E11-4: click E4 (32, 10) twice: E11-4 steps onto it. Note any message, sound or battle, then E11-4's tile and points, and E11-5's tile.

**Record**

- E11-5: the message after the step onto X4 (its text): 
- E11-5 after: tile; points: 
- E11-4: messages or battles after the step onto E4: 
- E11-4 after: tile; points. E11-5's tile: 
- anything odd: 

## E11-6 — A Subterranean Gate whose exit holds Red's other hero: the meeting, the mover staying (ADV-TEL-005, ADV-TEL-007, GAM-MET-001)

- Red hero **E11-6** (Cuthbert, specialty spell Weakness): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel
- Red hero **E11-6b** (Ingham, specialty creature Monk): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel

1. Day 1, Red's turn, after E11-4: the gate G6 (entrance (51, 10); it covers (50, 10) to (52, 10) and (51, 9)) leads to the underground gate U6 (entrance (51, 10) underground). E11-6b stands at (52, 11), E11-6 at (50, 11), each with one Angel.
2. E11-6b: click G6's entrance (51, 10) twice: E11-6b steps onto it. Note any message or dialog, then E11-6b's tile (level too), points and day's maximum.
3. E11-6: read the explored bits of the underground tiles (50, 9) to (52, 11) and (53, 10). Click G6's entrance (51, 10) twice: E11-6 steps onto it. Note the dialog that opens (which heroes it shows); close it at once without changing anything. Then read both heroes' tiles (level too) and E11-6's points, and the same explored bits again.

**Record**

- E11-6b: messages or dialogs after the step onto G6: 
- E11-6b after: tile (level); points; day's maximum: 
- E11-6 before its step: explored underground (50, 9) to (52, 11), (53, 10): 
- E11-6: the dialog after its step onto G6 (its kind; the heroes it shows): 
- after: E11-6's tile (level) and points; E11-6b's tile (level): 
- after: explored underground (50, 9) to (52, 11), (53, 10): 
- anything odd: 

## E11-9 — The gates' pairing at a truncated tie (E11-9); then the gate left without a pair (E11-8) (D-56, ADV-TEL-002, ADV-TEL-005)

- Red hero **E11-9** (Adela, specialty spell Bless): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel

1. Day 1, Red's turn, after E11-6: the gate G9 (entrance (20, 30); it covers (19, 30) to (21, 30) and (20, 29)). Underground two gates: Uf (entrance (17, 27); it covers (16, 27) to (18, 27) and (17, 26)) and Un (entrance (16, 30); it covers (15, 30) to (17, 30) and (16, 29)). E11-9 stands at (20, 31) with one Angel.
2. Click G9's entrance (20, 30) twice: E11-9 steps onto it. Note any message, then E11-9's tile (level too), points and day's maximum.
3. E11-8: E11-9 now stands on one of the two underground gates. Click the other one's entrance twice (Un (16, 30) if E11-9 stands on Uf, else Uf (17, 27)): E11-9 walks there and steps onto it. Note the path's tiles, the message (close it), then E11-9's tile (level too) and points.

**Record**

- E11-9: messages after the step onto G9: 
- E11-9 after: tile (level); points; day's maximum; which gate (Uf or Un): 
- E11-8: the gate stepped onto; the path's tiles; the message after the step (its text): 
- E11-8 after: tile (level); points: 
- anything odd: 

## E11-19 — Guards at both ends of a two-way pair: the step onto A19, the arrival at B19, the revisit by Space (ADV-MON-005, ADV-MON-006, ADV-TEL-007, ADV-TEL-008)

- Red hero **E11-19** (Rion, specialty skill First Aid): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel

1. Day 1, Red's turn, after E11-9: the two-way monoliths A19 (10, 50) and B19 (30, 50), orange (channel 1, no other member). A Peasant M at (9, 51), south-west of A19, a Peasant N at (31, 50), east of B19. E11-19 stands at (11, 50), east of A19, with ten Archangels. Read its points.
2. Click A19 (10, 50) twice: E11-19 steps onto it. Note every battle in order (by auto combat; which Peasant: M at (9, 51) or N at (31, 50)) and every message, then E11-19's tile and points.
3. If E11-19 stands on B19 (30, 50): press **Space** once. Note every battle and message, then E11-19's tile and points.
4. The control: if E11-19 stands on A19 (10, 50) and M is still there, click (10, 51) (south of A19, beside M) twice. Note the battle or that none came, and E11-19's tile after.

**Record**

- E11-19 before: points: 
- the step onto A19: battles in order (which Peasant; the result); messages: 
- after it: E11-19's tile; points: 
- Space on B19: battles (which Peasant); messages: 
- after Space: E11-19's tile; points: 
- the control: the step to (10, 51): the battle or none; E11-19's tile after: 
- anything odd: 

## E11-7 — A Subterranean Gate whose exit holds Blue's hero: the reveal, the battle, the move (ADV-TEL-005, GAM-BAT-001, GAM-BAT-004, ADV-FOG-002)

- Red hero **E11-7** (Tyris, specialty creature Cavalier): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Archangel
- Blue hero **E11-7b** (Loynis, specialty spell Prayer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn, last: the gate G7 (entrance (44, 30); it covers (43, 30) to (45, 30) and (44, 29)) leads to U7 (entrance (44, 30) underground). Red's E11-7 stands at (45, 31) with five Archangels, Blue's E11-7b at (43, 31) with one Pikeman. E11-7 does not move today. Read the explored bits of the underground tiles (43, 29) to (45, 31), (46, 30), (49, 30) and (50, 30).
2. Day 1, Blue's turn, first: E11-7b: click G7's entrance (44, 30) twice: E11-7b steps onto it. Note any message, then E11-7b's tile (level too).
3. Day 2, Red's turn, after E11-18: read the same explored bits again. E11-7: click G7's entrance (44, 30) twice: E11-7 steps onto it. At the battle's opening (before any command) note the terrain and battlefield, the attacker and the defender (heroes, stacks), and read the same explored bits. Play the battle: Red's stacks attack; Blue's Pikeman only waits or defends, never retreats or surrenders (two human sides: no auto combat). After it note the messages, then E11-7's tile (level too) and points, and read the explored bits once more.

**Record**

- day 1: explored underground (43, 29) to (45, 31), (46, 30), (49, 30), (50, 30): 
- E11-7b: messages after its step onto G7; its tile (level) after: 
- day 2 before the step: the same explored bits: 
- E11-7: messages or dialogs after its step onto G7: 
- the battle's opening: terrain, battlefield; attacker; defender (stacks): 
- at the opening: the same explored bits: 
- the result; the messages after the battle: 
- after: E11-7's tile (level); points; the same explored bits: 
- anything odd: 

## E11-18 — Blue's hero enters a one-way monolith Red sees, its exit in Red's shroud; Red's Replay Opponent Turn (A-41, ADV-FOG-004, ADV-TEL-007)

- Blue hero **E11-18** (Clancy, specialty creature Unicorn): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn: one-way channel 1 (red) has one entrance M18 (48, 33), which Red's E11-7 at (45, 31) sees, and one exit X18 (66, 60), far in Red's shroud. Blue's E11-18 stands at (47, 33), west of M18, with one Pikeman. Read Red's explored bits of (66, 60), (65, 60), (67, 60), (66, 59), (66, 61).
2. Day 1, Blue's turn, after E11-7b's step: E11-18: click M18 (48, 33) twice: E11-18 steps onto it. Note any message, then E11-18's tile.
3. Day 2, Red's turn, first: read the five explored bits again. Open the Adventure Options and choose **Replay Opponent Turn**. While it plays, note where the view goes (its top-left tile and level each time it moves) and what it shows (each Blue hero's moves). When it ends, read the five explored bits again, and Red's explored bits of the underground (43, 29) to (45, 31) (E11-7's). If the item is greyed out or plays nothing, note that and still read the bits.

**Record**

- day 1, Red: explored (66, 60), (65, 60), (67, 60), (66, 59), (66, 61): 
- E11-18: messages after its step onto M18; its tile after: 
- day 2, before the replay: the same explored bits: 
- the replay: the view's corners (tile, level) in order; what each part shows: 
- after the replay: the same explored bits; E11-7's underground bits: 
- anything odd: 
