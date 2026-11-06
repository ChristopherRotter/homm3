# E08 batch E11a — results

> Played by the capture harness on 2026-11-06 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-11a.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

> E08 T100a: three runs of the map; the second and third (this one) agree in all 97 filled records; the first differs only in E11-8, whose two branches it played both (the recipe's order since fixed).

Map file: `E08-batch-11a.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `0b5df5f3`.** The map's description in the scenario list starts with "Stamp 0b5df5f3.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-11a.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1), and at the start of each later turn (save 2 Blue's day 1, save 3 Red's day 2, save 4 Blue's day 2).
4. Most records are the game's memory: a hero's tile (column, row, level), its movement points (hero +0x4D), the day's maximum (+0x49) and army (slot, creature, count), whether it is in a boat (its status +0x105, bit 18, 0x40000), a tile's explored bit for Red, the hero's path after a first click (its tiles, or none), the messages and dialogs shown (by their text's table and line), the meeting dialog, a battle's opening (terrain, sides, stacks), the adventure map's view during Replay Opponent Turn. The capture harness reads them (`clients/capture/batches/E08-batch-11a.toml`, E08 T100). Played by hand, read the screens instead (the level button under the minimap shows the underground; a tile is explored when it is not black) and leave the memory-only lines empty.
5. Tiles are (column, row) from 0 at the top left; underground tiles say so. A walk: click the target once (the path's marks show), then again (the hero goes). A step onto a monolith or a gate: click its tile from the neighbouring tile named (twice). A boat: click it from the shore beside it (twice): the hero boards it and its walk ends there. A Whirlpool: from a boat beside it, click the Whirlpool's tile named (twice). **Space:** with the hero selected, press the space bar once.
6. Each battle against a monster by auto combat. E11-7's battle is between two human players, so auto combat is refused: play it, Red's stacks attacking, Blue's stack only waiting or defending (never a retreat or a surrender). A level-up's dialog after a battle (none is expected): take the left skill.
7. Order: day 1, Red's turn: E11-1 (rows E11-1 to E11-3), E11-4 (rows E11-5, then E11-4), E11-6, E11-9 (rows E11-9, then E11-8), E11-19; E11-7's and E11-18's first reads; end the turn. Blue's turn: E11-7's Blue step, then E11-18's; then the boardings of E11-10, E11-11, E11-12, E11-12b, E11-12c and E11-13, in that order; end the turn. Day 2, Red's turn: E11-18's replay and reads first, then E11-7's step and battle; end the turn. Blue's turn: E11-10, E11-11, E11-12 (E11-12, E11-12b, E11-12c), E11-13 (E11-13, then E11-18's hero); end the turn. Then the map is done.
8. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E11-1 — A path past a two-way monolith (E11-1), the step onto it (E11-2), a monster beside its exit (E11-3) (D-58, ADV-OBJ-003, ADV-TEL-004, ADV-TEL-007, ADV-MOV-008, ADV-MON-005)

- Red hero **E11-1** (Edric, specialty creature Griffin): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel

1. Day 1, Red's turn, first: E11-1 stands at (8, 10) with ten Archangels. East of it the two-way monolith A (9, 10), green; a rock wall fills columns 10 and 11 from row 1 to row 19; east of the wall the green monolith B (12, 10) and a Peasant P at (13, 11), south-east of B. Read E11-1's points and day's maximum and the explored bits of the tiles (12, 11), (16, 13), (16, 14), (17, 10) and (18, 10).
2. Click B (12, 10) once: note the path shown (its tiles, or that none shows). If none shows, do not click again and go on with the next step. **If a path shows**, click B again: E11-1 walks it. Note every step, any message or battle, then E11-1's tile, points and day's maximum, whether it moved on after a jump, and the explored bits of the five tiles of step 1; then skip the next step.
3. Click A (9, 10) once: note the path shown. Click it again: E11-1 steps onto A. Note any message or battle, then E11-1's tile, points and day's maximum, and the explored bits of the five tiles of step 1.
4. If E11-1 stands on B (12, 10): click (13, 10) (east of B, beside P) twice. Note the battle (auto combat) or that none came, and E11-1's tile after.

**Record**

- E11-1 before: points; day's maximum; explored (12, 11), (16, 13), (16, 14), (17, 10), (18, 10): points: 2000; day's maximum: 2000; explored: 12,11 yes; 16,13 no, 16,14 no; 17,10 no, 18,10 no
- E11-1: the path to B (12, 10) (its tiles, or none); if one showed, the walk: its steps, messages, battles: the path: none; clicked again: no; messages: none; battles: none; tile: 8,10,0; points: 2000; day's maximum: 2000; explored: 12,11 yes; 16,13 no, 16,14 no; 17,10 no, 18,10 no
- E11-1: the path to A (its tiles): 9,10
- E11-2: after the step onto A: messages; battles; E11-1's tile, points and day's maximum; moved on after the jump?: messages: none; battles: none; tile: 12,10,0; points: 1900; day's maximum: 2000; on A (9, 10) after it: no
- E11-2: explored after: (12, 11), (16, 13), (16, 14), (17, 10), (18, 10): 12,11 yes; 16,13 yes, 16,14 no; 17,10 yes, 18,10 no
- E11-3: the battle on arrival, if any (the defender, the result): none
- E11-3's control: the step to (13, 10): the battle or none; E11-1's tile after: battles: 1. at 13,10,0: defender 1 Peasant, winner attacker; tile after: 13,10,0; points: 1800; P (13, 11) after: none
- anything odd: the date at the start: month 1, week 1, day 1; the game's version: SoD; the difficulty: easy; 1 dialog(s) after the walk to [13, 10]; the control's messages: GENRLTXT:7

## E11-4 — A one-way exit visited (E11-5); then its entrance, whose only exit holds Red's hero (E11-4) (ADV-TEL-003, ADV-TEL-004)

- Red hero **E11-4** (Valeska, specialty creature Archer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel
- Red hero **E11-5** (Christian, specialty creature Ballista): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel

1. Day 1, Red's turn, after E11-1: one-way channel 0 (blue) has one entrance E4 (32, 10) and one exit X4 (40, 10). E11-5 stands at (39, 10), west of X4; E11-4 at (31, 10), west of E4; each with one Angel.
2. E11-5: click X4 (40, 10) twice: E11-5 steps onto it. Note the message (and close it), then E11-5's tile and points.
3. E11-4: click E4 (32, 10) twice: E11-4 steps onto it. Note any message, sound or battle, then E11-4's tile and points, and E11-5's tile.

**Record**

- E11-5: the message after the step onto X4 (its text): ADVEVENT:70
- E11-5 after: tile; points: tile: 40,10,0; points: 1900
- E11-4: messages or battles after the step onto E4: messages: none; battles: none; screens: none; sound: not read (a run without sound)
- E11-4 after: tile; points. E11-5's tile: tile: 32,10,0; points: 1900; E11-5's tile: 40,10,0
- anything odd: 

## E11-6 — A Subterranean Gate whose exit holds Red's other hero: the meeting, the mover staying (ADV-TEL-005, ADV-TEL-007, GAM-MET-001)

- Red hero **E11-6** (Cuthbert, specialty spell Weakness): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel
- Red hero **E11-6b** (Ingham, specialty creature Monk): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel

1. Day 1, Red's turn, after E11-4: the gate G6 (entrance (51, 10); it covers (50, 10) to (52, 10) and (51, 9)) leads to the underground gate U6 (entrance (51, 10) underground). E11-6b stands at (52, 11), E11-6 at (50, 11), each with one Angel.
2. E11-6b: click G6's entrance (51, 10) twice: E11-6b steps onto it. Note any message or dialog, then E11-6b's tile (level too), points and day's maximum.
3. E11-6: read the explored bits of the underground tiles (50, 9) to (52, 11) and (53, 10). Click G6's entrance (51, 10) twice: E11-6 steps onto it. Note the dialog that opens (which heroes it shows); close it at once without changing anything. Then read both heroes' tiles (level too) and E11-6's points, and the same explored bits again.

**Record**

- E11-6b: messages or dialogs after the step onto G6: messages: none; screens: none; battles: none
- E11-6b after: tile (level); points; day's maximum: tile: 51,10,1; points: 1859; day's maximum: 2000
- E11-6 before its step: explored underground (50, 9) to (52, 11), (53, 10): 50,9 yes, 51,9 yes, 52,9 yes, 50,10 yes, 51,10 yes, 52,10 yes, 50,11 yes, 51,11 yes, 52,11 yes; 53,10 yes
- E11-6: the dialog after its step onto G6 (its kind; the heroes it shows): screens: exchange; shows: E11-6 (left, the visitor) and E11-6b (right); messages: none; battles: none
- after: E11-6's tile (level) and points; E11-6b's tile (level): E11-6's tile: 51,10,0; points: 1859; E11-6b's tile: 51,10,1
- after: explored underground (50, 9) to (52, 11), (53, 10): 50,9 yes, 51,9 yes, 52,9 yes, 50,10 yes, 51,10 yes, 52,10 yes, 50,11 yes, 51,11 yes, 52,11 yes; 53,10 yes
- anything odd: E11-6's army after the meeting: 0: 1 Angel; E11-6b's army after the meeting: 0: 1 Angel

## E11-9 — The gates' pairing at a truncated tie (E11-9); then the gate left without a pair (E11-8) (D-56, ADV-TEL-002, ADV-TEL-005)

- Red hero **E11-9** (Adela, specialty spell Bless): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel

1. Day 1, Red's turn, after E11-6: the gate G9 (entrance (20, 30); it covers (19, 30) to (21, 30) and (20, 29)). Underground two gates: Uf (entrance (17, 27); it covers (16, 27) to (18, 27) and (17, 26)) and Un (entrance (16, 30); it covers (15, 30) to (17, 30) and (16, 29)). E11-9 stands at (20, 31) with one Angel.
2. Click G9's entrance (20, 30) twice: E11-9 steps onto it. Note any message, then E11-9's tile (level too), points and day's maximum.
3. E11-8: E11-9 now stands on one of the two underground gates. Click the other one's entrance twice (Un (16, 30) if E11-9 stands on Uf, else Uf (17, 27)): E11-9 walks there and steps onto it. Note the path's tiles, the message (close it), then E11-9's tile (level too) and points.

**Record**

- E11-9: messages after the step onto G9: messages: none; screens: none; battles: none
- E11-9 after: tile (level); points; day's maximum; which gate (Uf or Un): tile: 17,27,1; points: 1900; day's maximum: 2000; gate: Uf is 17,27,1, Un 16,30,1
- E11-8: the gate stepped onto; the path's tiles; the message after the step (its text): the gate: Un (16, 30) underground; the path: 17,28,1 17,29,1 16,30,1 18,30,1 17,31,1; messages: ADVEVENT:153
- E11-8 after: tile (level); points: tile: 16,30,1; points: 1277
- anything odd: 1 dialog(s) after the walk to [16, 30, 1]

## E11-19 — Guards at both ends of a two-way pair: the step onto A19, the arrival at B19, the revisit by Space (ADV-MON-005, ADV-MON-006, ADV-TEL-007, ADV-TEL-008)

- Red hero **E11-19** (Rion, specialty skill First Aid): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel

1. Day 1, Red's turn, after E11-9: the two-way monoliths A19 (10, 50) and B19 (30, 50), orange (channel 1, no other member). A Peasant M at (9, 51), south-west of A19, a Peasant N at (31, 50), east of B19. E11-19 stands at (11, 50), east of A19, with ten Archangels. Read its points.
2. Click A19 (10, 50) twice: E11-19 steps onto it. Note every battle in order (by auto combat; which Peasant: M at (9, 51) or N at (31, 50)) and every message, then E11-19's tile and points.
3. If E11-19 stands on B19 (30, 50): press **Space** once. Note every battle and message, then E11-19's tile and points.
4. The control: if E11-19 stands on A19 (10, 50) and M is still there, click (10, 51) (south of A19, beside M) twice. Note the battle or that none came, and E11-19's tile after.

**Record**

- E11-19 before: points: 2000
- the step onto A19: battles in order (which Peasant; the result); messages: battles: 1. at 10,50,0: defender 1 Peasant, winner attacker; messages: none; M (9, 51) after: none; N (31, 50) after: 1 Peasant
- after it: E11-19's tile; points: tile: 30,50,0; points: 1900
- Space on B19: battles (which Peasant); messages: battles: none; messages: none; what the key did: stepped; M (9, 51) after: none; N (31, 50) after: 1 Peasant
- after Space: E11-19's tile; points: tile: 10,50,0; points: 1900
- the control: the step to (10, 51): the battle or none; E11-19's tile after: battles: none; tile after: 10,51,0; points: 1800; M (9, 51) after: none; N (31, 50) after: 1 Peasant
- anything odd: 

## E11-7 — A Subterranean Gate whose exit holds Blue's hero: the reveal, the battle, the move (ADV-TEL-005, GAM-BAT-001, GAM-BAT-004, ADV-FOG-002)

- Red hero **E11-7** (Tyris, specialty creature Cavalier): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Archangel
- Blue hero **E11-7b** (Loynis, specialty spell Prayer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn, last: the gate G7 (entrance (44, 30); it covers (43, 30) to (45, 30) and (44, 29)) leads to U7 (entrance (44, 30) underground). Red's E11-7 stands at (45, 31) with five Archangels, Blue's E11-7b at (43, 31) with one Pikeman. E11-7 does not move today. Read the explored bits of the underground tiles (43, 29) to (45, 31), (46, 30), (49, 30) and (50, 30).
2. Day 1, Blue's turn, first: E11-7b: click G7's entrance (44, 30) twice: E11-7b steps onto it. Note any message, then E11-7b's tile (level too).
3. Day 2, Red's turn, after E11-18: read the same explored bits again. E11-7: click G7's entrance (44, 30) twice: E11-7 steps onto it. At the battle's opening (before any command) note the terrain and battlefield, the attacker and the defender (heroes, stacks), and read the same explored bits. Play the battle: Red's stacks attack; Blue's Pikeman only waits or defends, never retreats or surrenders (two human sides: no auto combat). After it note the messages, then E11-7's tile (level too) and points, and read the explored bits once more.

**Record**

- day 1: explored underground (43, 29) to (45, 31), (46, 30), (49, 30), (50, 30): 43,29 no, 44,29 no, 45,29 no, 43,30 no, 44,30 no, 45,30 no, 43,31 no, 44,31 no, 45,31 no; 46,30 no; 49,30 no, 50,30 no
- E11-7b: messages after its step onto G7; its tile (level) after: messages: none; battles: none; tile: 44,30,1
- day 2 before the step: the same explored bits: 43,29 no, 44,29 no, 45,29 no, 43,30 no, 44,30 no, 45,30 no, 43,31 no, 44,31 no, 45,31 no; 46,30 no; 49,30 no, 50,30 no
- E11-7: messages or dialogs after its step onto G7: screens: none; questions: none
- the battle's opening: terrain, battlefield; attacker; defender (stacks): terrain subterranean, special field none, ship flag 0, blocked hexes 122, 123, 124, 128, 129, 130, 144, 145, 146, 147; special field: none; sides: attacker red (human 1, here 1); defender blue (human 1, here 1); stacks: attacker 5 Archangels | defender 1 Pikeman; E11-7's tile at the opening: 44,30,0
- at the opening: the same explored bits: 43,29 yes, 44,29 yes, 45,29 yes, 43,30 yes, 44,30 yes, 45,30 yes, 43,31 yes, 44,31 yes, 45,31 yes; 46,30 no; 49,30 no, 50,30 no
- the result; the messages after the battle: winner: attacker; stacks at the end: attacker 5 Archangels | defender none; messages: none; level-ups: none
- after: E11-7's tile (level); points; the same explored bits: tile: 44,30,1; points: 1859; explored: 43,29 yes, 44,29 yes, 45,29 yes, 43,30 yes, 44,30 yes, 45,30 yes, 43,31 yes, 44,31 yes, 45,31 yes; 46,30 yes; 49,30 yes, 50,30 no
- anything odd: E11-7's experience after the battle: 510; E11-7b on the map after it: no

## E11-18 — Blue's hero enters a one-way monolith Red sees, its exit in Red's shroud; Red's Replay Opponent Turn (A-41, ADV-FOG-004, ADV-TEL-007)

- Blue hero **E11-18** (Clancy, specialty creature Unicorn): attack 1, defense 1, power 1, knowledge 1; no skills; Admiral's Hat (head); no spells; army 10 Pikeman

1. Day 1, Red's turn: one-way channel 1 (red) has one entrance M18 (48, 33), which Red's E11-7 at (45, 31) sees, and one exit X18 (66, 60), far in Red's shroud. Blue's E11-18 stands at (47, 33), west of M18, with ten Pikemen and the Admiral's Hat (it plays E11-13's second part on day 2). Read Red's explored bits of (66, 60), (65, 60), (67, 60), (66, 59), (66, 61), and of the boardings' tiles (51, 61), (51, 62), (51, 66).
2. Day 1, Blue's turn, after E11-7b's step: E11-18: click M18 (48, 33) twice: E11-18 steps onto it. Note any message, then E11-18's tile. (Blue's six boardings at the strait follow, E11-10 to E11-13.)
3. Day 2, Red's turn, first: read the five explored bits and the boardings' three again. Note the adventure view's size in tiles (columns, rows) once. Open the Adventure Options and choose **Replay Opponent Turn**. While it plays, note where the view goes (its top-left tile and level each time it moves) and what it shows (each Blue hero's moves; whether a view holds any tile of the six boardings, (50, 61) to (52, 66), and draws the heroes and boats there: a screenshot or a human's eye). When it ends, read the five explored bits again, the boardings' three, and Red's explored bits of the underground (43, 29) to (45, 31) (E11-7's). If the item is greyed out or plays nothing, note that and still read the bits.

**Record**

- day 1, Red: explored (66, 60), (65, 60), (67, 60), (66, 59), (66, 61); the boardings' (51, 61), (51, 62), (51, 66): 65,60 no, 66,60 no, 67,60 no; 66,59 no; 66,61 no; the boardings': 51,61 no, 51,62 no; 51,66 no
- E11-18: messages after its step onto M18; its tile after: messages: none; battles: none; tile: 66,60,0; points: 1460
- day 2, before the replay: the same eight explored bits: 65,60 no, 66,60 no, 67,60 no; 66,59 no; 66,61 no; the boardings': 51,61 no, 51,62 no; 51,66 no
- the view's size in tiles (columns, rows): 18 x 17 (the map area's whole tiles)
- the replay: the view's corners (tile, level) in order; what each part shows: the replay: played (1020 ms of the game's time, from 0 ms after the click); the view before: 36,23,0; the views in order: 35,22,0 → 35,22,1 → 38,25,0 → 39,25,0 → 57,52,0; the view after: 36,23,0
- the replay: a view holding tiles of the boardings (50, 61) to (52, 66), or none; the heroes and boats drawn there (a screenshot): none
- after the replay: the same eight explored bits; E11-7's underground bits: 65,60 no, 66,60 no, 67,60 no; 66,59 no; 66,61 no; the boardings': 51,61 no, 51,62 no; 51,66 no; E11-7's underground: 43,29 no, 44,29 no, 45,29 no, 43,30 no, 44,30 no, 45,30 no, 43,31 no, 44,31 no, 45,31 no; 46,30 no; 49,30 no, 50,30 no
- anything odd: worn at the start (the starting bonus): Admiral's Hat (head); the date: month 1, week 1, day 2; the Adventure Options dialog: vtable 0x63a610, item 5 state 0x6, items 1 2 3 4 5 200 201 30722; the screen during the replay (read by the agent from the run's 51 dumps, one every 20 ms of the game's time; not committed, ADR-0009): 0 to 160 ms the view about Red's E11-7 with E11-7b stepping onto G7; from 180 ms (the view's corner already U7's, 35,22,1) the same surface picture fading to black, no underground drawn; 500 to 660 ms the surface again about M18, E11-7b gone, E11-18 stepping onto M18; from 680 ms (the corner X18's, 57,52,0) that picture fading out again, nothing of X18's surroundings or of a hero there drawn; at the end the view back at 36,23,0 without E11-18

## E11-10 — A Whirlpool takes from the weakest stack: by fight value or by AI value (D-54, D-55, ADV-TEL-006, ADV-TEL-007, ADV-SEA-002)

- Blue hero **E11-10** (Sorsha, specialty creature Swordsman): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 3 Pegasus, 3 Dendroid Guard, 2 Angel

1. The strait (water from (46, 62) to (71, 65)) and the sea south-east of it lie in Red's shroud. The Whirlpool W1 covers (50, 63) to (52, 64); W2 (59, 68) to (61, 69) and W3 (65, 68) to (67, 69) lie in the sea. Blue's E11-10 stands on the shore at (50, 61) with 3 Pegasi (slot 0), 3 Dendroid Guards (slot 1) and 2 Angels (slot 2); an empty boat lies at (50, 62), south of it.
2. Day 1, Blue's turn, after E11-18's step: click the boat (50, 62) twice: E11-10 boards it. Note its tile, points, day's maximum and whether it is in a boat.
3. Day 2, Blue's turn, first: read E11-10's points, day's maximum and army (slot, creature, count). Click W1's (50, 63) twice: E11-10 sails onto it. Note every message (its text; close it), then E11-10's army, its tile (which Whirlpool; column, row), points and day's maximum, and whether it is still in a boat.

**Record**

- day 1, after the boarding: tile; points; day's maximum; in a boat: tile: 50,62,0; points: 0; day's maximum: 1500; in a boat: yes
- day 2, before the step: points; day's maximum: points: 1500; day's maximum: 1500

| slot | creature | count before | count after |
|---|---|---|---|
| 0 | Pegasi | 3 | 1 |
| 1 | Dendroid Guards | 3 | 3 |
| 2 | Angels | 2 | 2 |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |

- the step onto W1: messages (their text): ADVEVENT:168
- after: tile (W2 or W3; column, row); points; day's maximum; in a boat: tile: 66,69,0; points: 1400; day's maximum: 1500; in a boat: yes
- anything odd: 1 dialog(s) after the walk to [50, 63]; the army slot by slot after the step: 0: 1 Pegasus, 1: 3 Dendroid Guards, 2: 2 Angels

## E11-11 — A Whirlpool's tie between two equally weak stacks: the first slot or the last (D-54, D-55, ADV-TEL-006)

- Blue hero **E11-11** (Mephala, specialty skill Armorer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 2 Angel, 5 Pikeman, 5 Pikeman

1. Blue's E11-11 stands on the shore at (51, 61) with 2 Angels (slot 0), 5 Pikemen (slot 1) and 5 Pikemen (slot 2); an empty boat lies at (51, 62), south of it.
2. Day 1, Blue's turn, after E11-10's boarding: click the boat (51, 62) twice: E11-11 boards it. Note its tile, points, day's maximum and whether it is in a boat.
3. Day 2, Blue's turn, after E11-10's step: read E11-11's points, day's maximum and army. Click W1's (51, 63) twice: E11-11 sails onto it. Note every message (close it), then E11-11's army, its tile (which Whirlpool; column, row), points and day's maximum.

**Record**

- day 1, after the boarding: tile; points; day's maximum; in a boat: tile: 51,62,0; points: 0; day's maximum: 1560; in a boat: yes
- day 2, before the step: points; day's maximum: points: 1500; day's maximum: 1500

| slot | creature | count before | count after |
|---|---|---|---|
| 0 | Angels | 2 | 2 |
| 1 | Pikemen | 5 | 2 |
| 2 | Pikemen | 5 | 5 |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |
|   |   |   |   |

- the step onto W1: messages (their text): ADVEVENT:168
- after: tile (W2 or W3; column, row); points; day's maximum; in a boat: tile: 66,68,0; points: 1400; day's maximum: 1500; in a boat: yes
- anything odd: 1 dialog(s) after the walk to [51, 63]; the army slot by slot after the step: 0: 2 Angels, 1: 2 Pikemen, 2: 5 Pikemen

## E11-12 — Lone stacks in a Whirlpool: one Pikeman; 1 775 Azure Dragons; 1 776, past the original's bound (D-55, A-38, ADV-TEL-006)

- Blue hero **E11-12** (Ufretin, specialty creature Dwarf): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Blue hero **E11-12b** (Thorgrim, specialty skill Resistance): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1775 Azure Dragon
- Blue hero **E11-12c** (Ivor, specialty creature Wood Elf): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1776 Azure Dragon

1. Three Blue heroes, each with one stack: E11-12 on the shore at (52, 61) with one Pikeman, its boat at (52, 62) south of it; E11-12b at (50, 66) with 1 775 Azure Dragons, its boat at (50, 65) north of it; E11-12c at (51, 66) with 1 776 Azure Dragons, its boat at (51, 65) north of it.
2. Day 1, Blue's turn, after E11-11's boarding: E11-12, then E11-12b, then E11-12c: click the hero's boat twice; it boards. Note each one's tile, points, day's maximum and whether it is in a boat.
3. Day 2, Blue's turn, after E11-11's step: E11-12: read its points, day's maximum and army; click W1's (52, 63) twice: it sails onto it. Note every message (close it), then its army, tile (which Whirlpool; column, row) and points. Then E11-12b the same way onto W1's (50, 64), then E11-12c onto W1's (51, 64).

**Record**

| hero | day 1 after the boarding: tile; points; day's maximum; in a boat |
|---|---|
| E11-12 | tile: 52,62,0; points: 0; day's maximum: 1560; in a boat: yes |
| E11-12b | tile: 50,65,0; points: 0; day's maximum: 2000; in a boat: yes |
| E11-12c | tile: 51,65,0; points: 0; day's maximum: 2000; in a boat: yes |

| hero | day 2 before: points; maximum; army | messages after the step onto W1 | army after | tile after (W2 or W3) | points after; in a boat |
|---|---|---|---|---|---|
| E11-12 | points: 1500; maximum: 1500; army: 0: 1 Pikeman | ADVEVENT:168 | 0: 1 Pikeman | 60,69,0 | 1400; yes |
| E11-12b | points: 1500; maximum: 1500; army: 0: 1775 Azure Dragons | ADVEVENT:168 | 0: 887 Azure Dragons | 67,68,0 | 1400; yes |
| E11-12c | points: 1500; maximum: 1500; army: 0: 1776 Azure Dragons | ADVEVENT:168 | 0: 1776 Azure Dragons | 61,69,0 | 1400; yes |

- anything odd: 1 dialog(s) after the walk to [52, 63]; 1 dialog(s) after the walk to [50, 64]; 1 dialog(s) after the walk to [51, 64]

## E11-13 — The Sea Captain's Hat and the Admiral's Hat in a Whirlpool (ADV-TEL-006, ADV-SEA-002, HER-ADV-009)

- Blue hero **E11-13** (Uland, specialty spell Cure): attack 1, defense 1, power 1, knowledge 1; no skills; Sea Captain's Hat (head); no spells; army 10 Pikeman

1. Blue's E11-13 stands on the shore at (52, 66) with ten Pikemen and the Sea Captain's Hat, its boat at (52, 65) north of it. E11-18's hero (Blue's, on X18 (66, 60) since its jump on day 1) has ten Pikemen and the Admiral's Hat; its boat lies at (66, 62).
2. Day 1, Blue's turn, after E11-12c's boarding: click the boat (52, 65) twice: E11-13 boards it. Note its tile, points, day's maximum and whether it is in a boat.
3. Day 2, Blue's turn, after E11-12c's step: E11-13: read its points, day's maximum and army; click W1's (52, 64) twice: it sails onto it. Note every message, then its army, tile (which Whirlpool; column, row) and points.
4. Then E11-18's hero: read its tile, points and day's maximum. Click (66, 61) twice (it walks one step south), then the boat (66, 62) twice: it boards. Note its points, day's maximum and whether it is in a boat. Click W1's (52, 63) once: note the path's tiles, or that none shows. If one shows, click again: it sails there. If none shows, click (61, 63) twice (it sails west; note its points there), then W1's (52, 63) twice (note the path's tiles). Note every message, then its army, tile (which Whirlpool; column, row), points, day's maximum and whether it is in a boat.

**Record**

- E11-13, day 1 after the boarding: tile; points; day's maximum; in a boat: tile: 52,65,0; points: 0; day's maximum: 1560; in a boat: yes
- E11-13, day 2 before: points; day's maximum; army: points: 2000; day's maximum: 2000; army: 0: 10 Pikemen
- E11-13: messages after the step onto W1; army after; tile after (W2 or W3); points after; in a boat: messages: none; army: 0: 10 Pikemen; tile: 67,69,0; points: 1900; in a boat: yes
- E11-18's hero, day 2 before: tile; points; day's maximum; army: tile: 66,60,0; points: 1560; day's maximum: 1560; army: 0: 10 Pikemen
- E11-18's hero after the boarding: points; day's maximum; in a boat: points: 2615; day's maximum: 3000; in a boat: yes; tile: 66,62,0
- E11-18's hero: the path to W1's (52, 63) from the boat (its tiles, or none); if none, after the first leg (tile; points) and the second leg's path: from the boat: 62,62 63,62 64,62 65,62 52,63 61,63 53,64 54,64 55,64 56,64 57,64 58,64 59,64 60,64; clicked again: yes; not played: E11-18 stands on 60,68,0, not 66,62,0; not played: E11-18 stands on 60,68,0, not 61,63,0
- E11-18's hero: messages after the step onto W1; army after; tile after (W2 or W3); points after; day's maximum; in a boat: messages: none; army: 0: 10 Pikemen; tile: 60,68,0; points: 1092; day's maximum: 3000; in a boat: yes
- anything odd: worn at the start (the starting bonus): Sea Captain's Hat (head); worn on day 2: Sea Captain's Hat (head); E11-18's hero worn on day 2: Admiral's Hat (head); E11-18's hero's step to (66, 61): tile; points: 66,61,0; 1460
