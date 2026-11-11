# E08 batch E12b — results

> Played by the capture harness on 2026-11-12 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-12b.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

> E08 T110: four runs of the map; the fourth (the harness merged with `master`'s) agrees with the third (this one) in all 144 filled records; the second and the third agree in every filled record but the Fountains' first-week values, a new game's draws (F1 and F2: run 1 +3 and +1, run 2 +2 and +3, run 3 +2 and +2), and two reads the harness fixed before the third (the hint over S2 before, read only since; the Mermaids' entrance before, unexplored: the bar kept the Buoy's text); the values of weeks 2 to 4 the same in all four runs. The first run's labels differ (the recipe's since).

Map file: `E08-batch-12b.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `216b91af`.** The map's description in the scenario list starts with "Stamp 216b91af.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-12b.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1), and at the start of Red's turn on days 2, 6, 8 and 15 (saves 2 to 5).
4. Day 1 opens with a message for Red (a tax collector takes its gold): close it.
5. Most records are the game's memory: the date (game +0x1F63E the day of the week, +0x1F640 the week), a hero's tile, status bits (hero +0x105), its visits' morale and luck (+0x11A, +0x11B), the day's maximum movement (+0x49), the points left (+0x4D), the spell points (+0x18), experience and army; a tile's word; the map hint's text; every message's text. The capture harness reads them (`clients/capture/batches/E08-batch-12b.toml`, E08 T110). Played by hand, read the screens instead (the hero screen's morale and luck lists, the status bar's hint) and leave the memory-only lines empty.
6. Tiles are (column, row) from 0 at the top left. A walk: click the target once (the path's marks show), then again (the hero goes). A visit of an object that is not entered: click it twice from where the hero stands. **A hint:** with the hero named selected, hold the mouse over the tile named (an object's entrance) and note the status bar's text. **Another hero's hint:** select that hero in the hero list, centre the view on the tile with a click on the minimap, then hold the mouse over it.
7. **A cast:** open the hero's spell book on the adventure map, cast View Earth, close its view.
8. **The exchange between two heroes** (a click on a hero of one's own beside the hero): a move is a click on a stack, then a click on an empty slot of the other hero's bar (slots 0 to 6 from the left); close it with OK.
9. Each battle by auto combat. A level-up's dialog: take the left skill.
10. Order, Red's turns (Blue ends each of its turns at once): **day 1**: E12-12's reads, E12-17, E12-14, E12-9, E12-10, E12-11, E12-13, E12-7; **day 2**: E12-12, E12-14, E12-10, E12-11, E12-13 (E12-16's land part); **days 3 to 5**: E12-12; **day 6**: E12-12, E12-11; **day 7**: E12-12, E12-11; **day 8**: E12-12, E12-9, E12-10's reads; then end the turns to **day 15** and **day 22**: E12-12 on each. Then the map is done.
11. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E12-17 — Two Fountains of Fortune through the weeks (E12-12); a School of War visited without gold: refused, the kind mark set before the gold test, the hint over a second School (E12-17) (ADV-BON-003, GAM-CAL-002, ADV-TCH-002, ADV-BON-009, ADV-VIS-006)

- Red hero **E12-17** (Calh, specialty creature Gog): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Archangel

1. **E12-12.** Every day of Red's from day 1 to day 8, then on day 15 and day 22, first in Red's turn: read the date and the tile words of the Fountains of Fortune F1 (26, 4) and F2 (30, 4) (their value, bits 13 to 16 signed; their team bits 5 to 12). No hero visits them.
2. **E12-17.** Day 1, Red's turn, after E12-12's reads (Red's treasury: 0 gold after the day's message): the Schools of War S1 (entered at (51, 22)) and S2 (entered at (51, 26)). E12-17 stands at (54, 24) with one Archangel. Read Red's gold, E12-17's points and its School of War bits (hero +0x7B).
3. With E12-17 selected, the hint over S2's entrance (51, 26); note its text.
4. Click S1's entrance (51, 22) twice: E12-17 steps onto it. Note every message and dialog (their texts; answer a question with its leave or cancel button), then E12-17's tile, points, attack, defense, +0x7B, and Red's gold.
5. Click (52, 23) twice: E12-17 steps off S1, south-east (a hero on an entrance hides its object from the hint). With E12-17 selected, the hints over S2's entrance (51, 26) and S1's (51, 22); note each text and E12-17's points.

**Record**

| day | F1 (26, 4): value; team bits | F2 (30, 4): value; team bits |
|---|---|---|
| month 1, week 1, day 1 | value +2; team bits none (word 0xfffe401f) | value +2; team bits none (word 0xfffe401f) |
| month 1, week 1, day 2 | value +2; team bits none (word 0xfffe401f) | value +2; team bits none (word 0xfffe401f) |
| month 1, week 1, day 3 | value +2; team bits none (word 0xfffe401f) | value +2; team bits none (word 0xfffe401f) |
| month 1, week 1, day 4 | value +2; team bits none (word 0xfffe401f) | value +2; team bits none (word 0xfffe401f) |
| month 1, week 1, day 5 | value +2; team bits none (word 0xfffe401f) | value +2; team bits none (word 0xfffe401f) |
| month 1, week 1, day 6 | value +2; team bits none (word 0xfffe401f) | value +2; team bits none (word 0xfffe401f) |
| month 1, week 1, day 7 | value +2; team bits none (word 0xfffe401f) | value +2; team bits none (word 0xfffe401f) |
| month 1, week 2, day 1 | value +3; team bits none (word 0xfffe601f) | value +3; team bits none (word 0xfffe601f) |
| month 1, week 3, day 1 | value +2; team bits none (word 0xfffe401f) | value +2; team bits none (word 0xfffe401f) |
| month 1, week 4, day 1 | value +1; team bits none (word 0xfffe201f) | value +3; team bits none (word 0xfffe601f) |

- E12-17 before: Red's gold; E12-17's points; +0x7B: gold: 0; points: 2000; +0x7B: 0x00000000
- the hint over S2 (51, 26) before: 51,26: [class 107 name] [GENRLTXT 354]
- the step onto S1 (51, 22): messages and dialogs (texts): questions: none; messages: ADVEVENT:160; pictures: none; dialogs: none
- after it: E12-17's tile; points; attack; defense; +0x7B; Red's gold: tile: 51,22,0; points: 1618; primary: 1/1/1/1; +0x7B: 0x00000000; gold: 0
- the hints after (E12-17 on (52, 23)): over S2 (51, 26); over S1 (51, 22); E12-17's points: 51,26: [class 107 name] [XTRAINFO 10] [GENRLTXT 354]; 51,22: [class 107 name] [XTRAINFO 10] [GENRLTXT 354]; points: 1477
- anything odd: the step off S1 to (52, 23): tile: 52,23,0; the game's version: SoD; the difficulty: easy; the turns' messages to day 2: GENRLTXT:14, GENRLTXT:14; the turns' messages to day 8 (the new week): GENRLTXT:14, GENRLTXT:14, GENRLTXT:16 [other: 76 characters]

## E12-14 — Two Magic Wells: a full hero, a refill after a cast, another Well the same day, the next day (ADV-BON-006, HER-MAN-001, HER-MAN-002)

- Red hero **E12-14** (Thane, specialty creature Genie): attack 1, defense 1, power 1, knowledge 2; no skills; Spell Book (spell book); spells View Earth; army 1 Archangel

1. Day 1, Red's turn, after E12-17: the Magic Wells W1 (6, 5) and W2 (10, 5) (each visited from a neighbour). E12-14 stands at (6, 6), south of W1, with one Archangel, a spell book with View Earth, knowledge 2 (its maximum 20, full). Read its spell points and status bits (+0x105).
2. Click W1 (6, 5) twice: note every message, the spell points and bit 0.
3. Cast View Earth. Read the spell points. Click W1 twice: note every message, the spell points and bit 0.
4. Cast View Earth. Read the spell points. Click W2 (10, 5) twice (E12-14 walks to a tile beside it): note every message, the tile, the spell points and bit 0.
5. Day 2, Red's turn: read E12-14's spell points and bit 0. Click W2 twice: note every message, the spell points and bit 0.

**Record**

- day 1 before: spell points; +0x105: spell points: 20; +0x105: 0x0
- W1, full: messages; spell points; bit 0: messages: ADVEVENT:79; spell points: 20; bit 0: clear; tile: 6,5,0
- the cast: spell points; W1 again: messages; spell points; bit 0: the cast: the book opened; View Earth; the view 0x643c54; spell points 20 then 18; W1 again (standing on it: Space): messages: ADVEVENT:77; spell points: 20; bit 0: set
- the cast: spell points; W2: messages; E12-14's tile; spell points; bit 0: the cast: the book opened; View Earth; the view 0x643c54; spell points 20 then 18; W2: messages: ADVEVENT:78; tile: 10,5,0; spell points: 18; bit 0: set
- day 2 before: spell points; bit 0: spell points: 19; bit 0: clear; tile: 10,5,0
- W2 on day 2: messages; spell points; bit 0: standing on W2: Space: messages: ADVEVENT:77; spell points: 20; bit 0: set
- anything odd: the agent (E08 T110): E12-14 stood on W1 (6, 5) after its first visit (its record's tile; in the first run a click on (6, 5) opened the hero screen), so W1 again was Space there (the recipe's branch); the walk to W2 (10, 5) ended on it, and day 2's visit again was Space too.

## E12-9 — A Magic Spring's two entrances: each refills once a week on its own; both again at the next week (D-62, ADV-BON-007, HER-MAN-001, HER-MAN-002)

- Red hero **E12-9** (Ivor, specialty creature Wood Elf): attack 1, defense 1, power 1, knowledge 2; no skills; Spell Book (spell book); spells View Earth; army 1 Archangel

1. Day 1, Red's turn, after E12-14: the Magic Spring's entrances L (17, 5) and R (18, 5). E12-9 stands at (16, 6) with one Archangel, a spell book with View Earth, knowledge 2 (its maximum 20, full). Read its spell points and the words of L and R (bit 6).
2. Click L (17, 5) twice: E12-9 steps onto it. Note every message, the spell points and R's word.
3. Cast View Earth; read the spell points. Click R (18, 5) twice: E12-9 steps onto it. Note every message, the spell points and L's word.
4. Cast View Earth; read the spell points. Click L (17, 5) twice: E12-9 steps onto it. Note every message, the spell points and R's word. Then click (17, 6) twice (E12-9 steps off, south) and read L's and R's words.
5. Day 8, Red's turn (the second week's first day): read E12-9's tile, spell points and the words of L and R. Cast View Earth; read the spell points. Click R (18, 5) twice: note every message, the spell points and L's word. Cast View Earth; read the spell points. Click L (17, 5) twice: note every message, the spell points and R's word.

**Record**

- day 1 before: spell points; L's word (bit 6); R's word (bit 6): spell points: 20; L: full (bit 6 set; word 0xffffffe0); R: full (bit 6 set; word 0xffffffe0)
- onto L: messages; spell points; R's bit 6: messages: ADVEVENT:74; tile: 17,5,0; spell points: 40; R: full (bit 6 set; word 0xffffffe0); L (under the hero): emptied (bit 6 clear; word 0xffffffa0)
- the cast: spell points; onto R: messages; spell points; L's bit 6: the cast: the book opened; View Earth; the view 0x643c54; spell points 40 then 38; onto R: messages: ADVEVENT:74; tile: 18,5,0; spell points: 40; L: emptied (bit 6 clear; word 0xffffffa0); R (under the hero): emptied (bit 6 clear; word 0xffffffa0)
- the cast: spell points; onto L again: messages; spell points; R's bit 6: the cast: the book opened; View Earth; the view 0x643c54; spell points 40 then 38; onto L again: messages: ADVEVENT:75; tile: 17,5,0; spell points: 38; R: emptied (bit 6 clear; word 0xffffffa0); L (under the hero): emptied (bit 6 clear; word 0xffffffa0)
- off to (17, 6): L's bit 6; R's bit 6: tile: 17,6,0; L: emptied (bit 6 clear; word 0xffffffa0); R: emptied (bit 6 clear; word 0xffffffa0)
- day 8 before: E12-9's tile; spell points; L's bit 6; R's bit 6: tile: 17,6,0; spell points: 38; L: full (bit 6 set; word 0xffffffe0); R: full (bit 6 set; word 0xffffffe0)
- day 8, the cast: spell points; onto R: messages; spell points; L's bit 6: the cast: the book opened; View Earth; the view 0x643c54; spell points 38 then 36; onto R: messages: ADVEVENT:74; tile: 18,5,0; spell points: 40; L: full (bit 6 set; word 0xffffffe0); R (under the hero): emptied (bit 6 clear; word 0xffffffa0)
- day 8, the cast: spell points; onto L: messages; spell points; R's bit 6: the cast: the book opened; View Earth; the view 0x643c54; spell points 40 then 38; onto L: messages: ADVEVENT:74; tile: 17,5,0; spell points: 40; R: emptied (bit 6 clear; word 0xffffffa0); L (under the hero): emptied (bit 6 clear; word 0xffffffa0)
- anything odd: 

## E12-10 — The Stables: a first visit without Cavaliers, Cavaliers taken from a second hero, its own first visit, the revisits on day 2 (D-63, ADV-BON-005, HER-ADV-003, ADV-MOV-001)

- Red hero **E12-10** (Thorgrim, specialty skill Resistance): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Pikeman
- Red hero **E12-10b** (Mephala, specialty skill Armorer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 3 Cavalier, 3 Cavalier, 5 Pikeman

1. Day 1, Red's turn, after E12-9: the Stables ST, entered at (38, 5). E12-10 stands at (39, 7) with five Pikemen; E12-10b at (40, 7) with two stacks of three Cavaliers and five Pikemen (slots 0, 1, 2). Read both heroes' points, day's maximum, status bits (+0x105) and armies.
2. Click ST's entrance (38, 5) with E12-10 twice: E12-10 steps onto it. Note every message, its points, day's maximum, bit 1 and army.
3. Click E12-10b (40, 7) with E12-10 twice: E12-10 walks beside it and the exchange opens. Move both of E12-10b's Cavalier stacks into E12-10's empty slots 1 and 2; close it. Note both armies, E12-10's tile and points.
4. Click ST's entrance (38, 5) with E12-10b twice: E12-10b steps onto it. Note every message, its points, day's maximum, bit 1 and army. Then click (37, 6) with E12-10b twice (it steps off, south-west).
5. Day 2, Red's turn: read both heroes' points, day's maximum, bit 1 and armies.
6. Click E12-10 (39, 6) with E12-10b twice: the exchange opens. Move E12-10's Cavaliers of slot 2 into E12-10b's empty slot 0; close it. Note both armies.
7. Click ST's entrance (38, 5) with E12-10 twice: note every message, its points, day's maximum and army. Then click (39, 5) with E12-10 twice (it steps off, east).
8. Click ST's entrance (38, 5) with E12-10b twice: note every message, its points, day's maximum and army.
9. Day 8, Red's turn (the second week's first day): read both heroes' day's maximum, points, bit 1 and armies.

**Record**

- day 1 before: E12-10's points, maximum, +0x105, army; E12-10b's the same: E12-10's points: 1560; maximum: 1560; +0x105: 0x0; army: 0: 5 Pikemen; E12-10b's points: 1560; maximum: 1560; +0x105: 0x0; army: 0: 3 Cavaliers, 1: 3 Cavaliers, 2: 5 Pikemen
- E12-10 onto ST: messages; points; maximum; bit 1; army: messages: ADVEVENT:137; tile: 38,5,0; points: 1719; maximum: 1960; bit 1: set; army: 0: 5 Pikemen
- the exchange: both armies after; E12-10's tile; points: E12-10: 0: 5 Pikemen, 1: 3 Cavaliers, 2: 3 Cavaliers; E12-10b: 2: 5 Pikemen; the meeting: E12-10 (left, the visitor) and E12-10b (right); E12-10's tile: 39,6,0; points: 1437
- E12-10b onto ST: messages; points; maximum; bit 1; army; its tile after stepping off: messages: ADVEVENT:137; tile: 38,5,0; points: 1619; maximum: 1960; bit 1: set; army: 2: 5 Pikemen; stepped off: tile: 37,6,0; points: 1478
- day 2 before: E12-10's points, maximum, bit 1, army; E12-10b's the same: E12-10's points: 1960; maximum: 1960; bit 1: set; army: 0: 5 Pikemen, 1: 3 Cavaliers, 2: 3 Cavaliers; tile: 39,6,0; E12-10b's points: 1960; maximum: 1960; bit 1: set; army: 2: 5 Pikemen; tile: 37,6,0
- the exchange back: both armies after: E12-10: 0: 5 Pikemen, 1: 3 Cavaliers; E12-10b: 0: 3 Cavaliers, 2: 5 Pikemen; the meeting: E12-10b (left, the visitor) and E12-10 (right); E12-10b's tile: 38,6,0; points: 1760
- E12-10 onto ST again: messages; points; maximum; army: messages: ADVEVENT:138; tile: 38,5,0; points: 1819; maximum: 1960; army: 0: 5 Pikemen, 1: 3 Champions
- E12-10b onto ST again: messages; points; maximum; army: messages: ADVEVENT:138; tile: 38,5,0; points: 1660; maximum: 1960; army: 0: 3 Champions, 2: 5 Pikemen
- day 8: E12-10's maximum, points, bit 1, army; E12-10b's the same: E12-10's maximum: 1560; points: 1560; bit 1: clear; army: 0: 5 Pikemen, 1: 3 Champions; E12-10b's maximum: 1560; points: 1560; bit 1: clear; army: 0: 3 Champions, 2: 5 Pikemen
- anything odd: E12-10 stepped off east: tile: 39,5,0; points: 1719

## E12-11 — The Idol of Fortune on days 1, 2 and 7, the Temple on days 6 and 7, a battle between (the marks cleared) (ADV-BON-002, GAM-BAT-014, HER-ADV-008)

- Red hero **E12-11** (Josephine, specialty creature Stone Golem): attack 5, defense 5, power 1, knowledge 1; no skills; no artifacts; no spells; army 2 Archangel

1. Day 1, Red's turn, after E12-10: the Idol of Fortune ID (48, 5) (visited from a neighbour), the Temple TE entered at (53, 5); three Peasants P1 (46, 9), P2 (51, 9), P3 (56, 9). E12-11 stands at (48, 6), south of ID, with two Archangels. Read the date, E12-11's +0x105, +0x11A, +0x11B.
2. Click ID (48, 5) twice: note every message, then +0x105, +0x11A, +0x11B. Click P1 (46, 9) twice: E12-11 walks towards it and fights it (auto combat). Note the battle, E12-11's tile, then +0x105, +0x11A, +0x11B.
3. Day 2, Red's turn: read the date. Click ID (48, 5) twice (E12-11 walks beside it from below or the side): note every message, then +0x105, +0x11A, +0x11B. Click P2 (51, 9) twice: the battle; note E12-11's tile, then +0x105, +0x11A, +0x11B.
4. Day 6, Red's turn: read the date. Click TE's entrance (53, 5) twice: note every message, then +0x105, +0x11A, +0x11B. Click P3 (56, 9) twice: the battle; note E12-11's tile, then +0x105, +0x11A, +0x11B.
5. Day 7, Red's turn: read the date. Click ID (48, 5) twice: note every message, then +0x105, +0x11A, +0x11B. Click TE's entrance (53, 5) twice: note every message, then +0x105, +0x11A, +0x11B, and the hero screen's morale and luck lists.

**Record**

- day 1 before: date (day of the week); +0x105; +0x11A; +0x11B: month 1, week 1, day 1; +0x105: 0x0; +0x11A: 0; +0x11B: 0

| day | visit or battle | messages | +0x105 after | +0x11A (morale) after | +0x11B (luck) after |
|---|---|---|---|---|---|
| month 1, week 1, day 1 | the Idol (48, 5) | ADVEVENT:62 | 0x10 | 0 | 1 |
| month 1, week 1, day 1 | the battle with P1: 1. at 47,8,0: defender 1 Peasant, winner attacker; tile: 47,8,0 | none | 0x0 | 0 | 0 |
| month 1, week 1, day 2 | the Idol (48, 5); tile: 48,5,0 | ADVEVENT:62 | 0x2000000 | 1 | 0 |
| month 1, week 1, day 2 | the battle with P2: 1. at 50,8,0: defender 1 Peasant, winner attacker; tile: 50,8,0 | none | 0x0 | 0 | 0 |
| month 1, week 1, day 6 | the Temple (53, 5); tile: 53,5,0 | ADVEVENT:140 | 0x100 | 1 | 0 |
| month 1, week 1, day 6 | the battle with P3: 1. at 55,8,0: defender 1 Peasant, winner attacker; tile: 55,8,0 | none | 0x0 | 0 | 0 |
| month 1, week 1, day 7 | the Idol (48, 5); tile: 48,5,0 | ADVEVENT:62 | 0x2000010 | 1 | 1 |
| month 1, week 1, day 7 | the Temple (53, 5); tile: 53,5,0 | ADVEVENT:140 | 0x6000010 | 3 | 1 |

- day 7: the hero screen's morale and luck lists: morale: the picture (id:state:frame): 116:0x6:6; the lines: [other: 13 characters] / [other: 57 characters] / [other: 27 characters] / ARRAYTXT:68|93 / ARRAYTXT:97; luck: the picture (id:state:frame): 117:0x6:4; the lines: [other: 11 characters] / [other: 81 characters] / [other: 25 characters] / ARRAYTXT:68|93
- anything odd: the agent (E08 T110): E12-11 stood on the Idol (48, 5) after each visit (the rows' tiles), as on the Temple's entrance (53, 5).

## E12-13 — An Oasis, a Watering Hole, a Swan Pond (the points and the day's maximum); on day 2 a Faerie Ring, a Rally Flag, a Fountain of Youth and the hints over their twins (E12-16) (ADV-BON-004, ADV-BON-002, ADV-BON-009, ADV-VIS-006, HER-ADV-008)

- Red hero **E12-13** (Piquedram, specialty creature Stone Gargoyle): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 2 Archangel

1. Day 1, Red's turn, after E12-11: the Oasis O1 (entered at (43, 16), (44, 16)), the Watering Hole WH (entered at (47, 16) to (50, 16)), the Swan Pond SW (entered at (55, 16)). E12-13 stands at (42, 17) with two Archangels. Read its points (+0x4D), day's maximum (+0x49), +0x105, +0x11A, +0x11B.
2. Click O1's entrance (43, 16) twice. Note every message, then +0x4D, +0x49, +0x105, +0x11A, +0x11B.
3. Click WH's entrance (47, 16) twice. Note every message, then +0x4D, +0x49, +0x105, +0x11A, +0x11B.
4. Click (51, 17) twice (a walk east), then SW's entrance (55, 16) twice. Note every message, then +0x4D, +0x49, +0x105, +0x11A, +0x11B.
5. **E12-16, land.** Day 2, Red's turn: the Faerie Ring F1 (entered at (57, 20)), the Rally Flag R1 (at (61, 20)), the Fountain of Youth Y1 (at (64, 20)); their twins F2 (57, 24), R2 (61, 24), Y2 (64, 24). Read E12-13's points, day's maximum and +0x105. Click (58, 19) twice, then (60, 22) twice (two walks). With E12-13 selected, the hints over F2 (57, 24), R2 (61, 24) and Y2 (64, 24): note each text.
6. Click F1's entrance (57, 20) twice; then R1's (61, 20) twice; then Y1's (64, 20) twice. After each note every message, +0x4D, +0x49, +0x105, +0x11A, +0x11B.
7. With E12-13 selected, the hints over F2 (57, 24), R2 (61, 24) and Y2 (64, 24); then with E12-17 selected (the hero list; centre the view on them with the minimap) the same three. Note each text.

**Record**

- day 1 before: +0x4D; +0x49; +0x105; +0x11A; +0x11B: +0x4D: 2000; +0x49: 2000; +0x105: 0x0; +0x11A: 0; +0x11B: 0

| visit | messages | +0x4D after | +0x49 after | +0x105 after | +0x11A after | +0x11B after |
|---|---|---|---|---|---|---|
| O1 (43, 16), day 1; tile: 43,16,0 | ADVEVENT:95 | 2659 | 2800 | 0x80 | 1 | 0 |
| WH (47, 16), day 1; tile: 47,16,0 | ADVEVENT:166 | 2577 | 3200 | 0xc0 | 2 | 0 |
| SW (55, 16), day 1; tile: 55,16,0 | ADVEVENT:29 | 0 | 3200 | 0xc8 | 2 | 2 |
| F1 (57, 20), day 2; tile: 57,20,0 | ADVEVENT:49 | 813 | 2000 | 0x20c8 | 2 | 3 |
| R1 (61, 20), day 2; tile: 61,20,0 | ADVEVENT:111 | 731 | 2400 | 0x120c8 | 3 | 4 |
| Y1 (64, 20), day 2; tile: 64,20,0 | ADVEVENT:57 | 749 | 2800 | 0x160c8 | 4 | 4 |

- day 2 before: +0x4D; +0x49; +0x105: +0x4D: 2000; +0x49: 2000; +0x105: 0xc8; tile: 55,16,0
- day 2, the hints before (E12-13 selected): F2; R2; Y2: 57,24: [class 28 name] [GENRLTXT 354]; 61,24: [class 64 name] [GENRLTXT 354]; 64,24: [class 31 name] [GENRLTXT 354]
- the hints after, E12-13 selected: F2; R2; Y2: 57,24: [class 28 name] [XTRAINFO 16] [GENRLTXT 353]; 61,24: [class 64 name] [XTRAINFO 17] [GENRLTXT 353]; 64,24: [class 31 name] [XTRAINFO 13] [GENRLTXT 353]
- the hints after, E12-17 selected: F2; R2; Y2: 57,24: [class 28 name] [XTRAINFO 16] [GENRLTXT 354]; 61,24: [class 64 name] [XTRAINFO 17] [GENRLTXT 354]; 64,24: [class 31 name] [XTRAINFO 13] [GENRLTXT 354]
- anything odd: the walk to (51, 17): points: 2136; day 2's walks: to (58, 19): points: 1577; to (60, 22): points: 1195

## E12-7 — The Admiral's Hat's boarding after an Oasis (E12-15); the Sirens (E12-7), a battle, the Sirens again (E12-8); a Buoy, the Mermaids and the hints over their twins (E12-16) (D-64, A-44, ADV-BON-008, ADV-SEA-002, HER-EXP-001, ADV-BON-009)

- Red hero **E12-7** (Clancy, specialty creature Unicorn): attack 1, defense 1, power 1, knowledge 1; no skills; Admiral's Hat (head); no spells; army 90 Pikeman, 10 Archer, 1 Griffin

1. **E12-15.** Day 1, Red's turn, after E12-13: the Oasis O2 (entered at (41, 27), (42, 27)) on the shore; an empty boat on (43, 30); the Sirens SI, entered at (47, 33); a Peasant M8 at sea (47, 37). E12-7 stands at (40, 28) with ninety Pikemen, ten Archers and one Griffin (slots 0, 1, 2), the Admiral's Hat, no skills. Read its points, day's maximum, experience, army and +0x105.
2. Click O2's entrance (41, 27) twice: note every message, the points and the day's maximum. Click (43, 29) twice (the shore beside the boat): note the points. Click the boat (43, 30) twice: E12-7 boards it. Note the points and the day's maximum.
3. **E12-7.** Click SI's entrance (47, 33) twice: E12-7 sails onto it. Note every message (their texts), the army, experience, level, +0x105 and the points.
4. **E12-8.** Click M8 (47, 37) twice: E12-7 sails towards it and fights it (auto combat). Note the battle, the tile, the army, experience and +0x105 after it. With E12-7 selected, the hints over the Buoy B2 (51, 38) and the Mermaids M2's entrance (52, 37): note each text (E12-16's before). Click SI's entrance (47, 33) twice: note every message, the army, experience, level and +0x105.
5. **E12-16, water.** The Buoy B1 (51, 33) (visited from a neighbour) and the Mermaids M1, entered at (53, 33). Click B1 twice: note every message, the tile, the points, +0x105 and +0x11A. Click M1's entrance (53, 33) twice: note every message, the points, +0x105 and +0x11B.
6. With E12-7 selected, the hints over B2 (51, 38) and M2's entrance (52, 37); then with E12-17 selected (the hero list; centre the view on them with the minimap) the same two. Note each text.

**Record**

- E12-15 before: points; day's maximum; experience; army; +0x105: points: 1560; maximum: 1560; experience: 0; army: 0: 90 Pikemen, 1: 10 Archers, 2: 1 Griffin; +0x105: 0x0
- E12-15, O2: messages; points; day's maximum: messages: ADVEVENT:95; tile: 41,27,0; points: 2219; maximum: 2360
- E12-15, on the shore (43, 29): points: tile: 43,29,0; points: 1937
- E12-15, after boarding: points; day's maximum: in a boat: yes; tile: 43,30,0; points: 2335; maximum: 3000
- E12-7, the Sirens: messages (texts); army; experience; level; +0x105; points: messages: ADVEVENT:132; tile: 46,34,0; army: 0: 62 Pikemen, 1: 7 Archers, 2: 1 Griffin; experience: 310; level: 1; +0x105: 0x140080; points: 1671
- E12-8, the battle: result; E12-7's tile; army; experience; +0x105: battles: 1. at 46,36,0: defender 1 Peasant, winner attacker; messages: none; tile: 46,36,0; army: 0: 62 Pikemen, 1: 7 Archers, 2: 1 Griffin; experience: 311; +0x105: 0x40000
- E12-16, the hints before (E12-7 selected): B2; M2: 51,38: [class 11 name] [GENRLTXT 354]; 52,37: the tile is not explored: no hint (the bar kept the text before: [other: 4 characters][GENRLTXT 354])
- E12-8, the Sirens again: messages (texts); army; experience; level; +0x105: messages: ADVEVENT:132; tile: 46,34,0; army: 0: 43 Pikemen, 1: 4 Archers, 2: 1 Griffin; experience: 531; level: 1; +0x105: 0x140000; points: 1130
- E12-16, B1: messages; tile; points; +0x105; +0x11A: messages: ADVEVENT:21; tile: 50,34,0; points: 589; +0x105: 0x140004; +0x11A: 1
- E12-16, M1: messages; points; +0x105; +0x11B: messages: ADVEVENT:83; tile: 52,34,0; points: 248; +0x105: 0x148004; +0x11B: 1
- E12-16, the hints after, E12-7 selected: B2; M2: 51,38: [class 11 name] [XTRAINFO 0] [GENRLTXT 353]; 52,37: [class 52 name] [XTRAINFO 16] [GENRLTXT 353]
- E12-16, the hints after, E12-17 selected: B2; M2: 51,38: [class 11 name] [XTRAINFO 0] [GENRLTXT 354]; 52,37: [class 52 name] [XTRAINFO 16] [GENRLTXT 354]
- anything odd: worn at the start (the Admiral's Hat; the starting bonus): Admiral's Hat (head); the agent (E08 T110): the Sirens, B1 and M1 were visited from the neighbouring water tiles (46, 34), (50, 34), (52, 34), the hero never on them; M2's entrance (52, 37) was not explored by Red at the hints before (the bar kept B2's text: no hint over it), explored after B1's visit.
