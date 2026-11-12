# E08 batch E12a — results

> Played by the capture harness on 2026-11-12 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-12a.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

> E08 T110: four runs of the map; the second, the third (this one) and the fourth (the harness merged with `master`'s) agree in all 74 filled records, the first only in one label (the recipe's since).

Map file: `E08-batch-12a.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `d2c37714`.** The map's description in the scenario list starts with "Stamp d2c37714.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-12a.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1), and at the start of Blue's turn (save 2).
4. Most records are the game's memory: a hero's tile, its movement points (hero +0x4D) and army, the path's marks shown after one click, the dialogs and messages shown (the exchange: its two bars), a garrison's record (the game's vector at +0x4E3AC, 0x40 bytes each: the owner +0x0, the seven stacks +0x4, removable +0x3C, the entrance +0x3D), Red's explored tiles, and as a battle opens its anti-magic flag (+0x53C4) and each side's stacks. The capture harness reads them (`clients/capture/batches/E08-batch-12a.toml`, E08 T110). Played by hand, read the screens instead and leave the memory-only lines empty.
5. Tiles are (column, row) from 0 at the top left. A walk: click the target once (the path's marks show), then again (the hero goes).
6. **The exchange** (a garrison's dialog: the garrison's army in the upper bar, the hero's in the lower; slots 0 to 6 from the left): a move is a click on a stack (it is selected), then a click on the target slot. Close it with its OK button.
7. A battle against the neutral garrison by auto combat. **A battle between the two heroes** (E12-4; both sides human, so no auto combat): at each side's first stack's turn, click the hero's spell book button: note whether the book opens or a message shows (its text); if it opens, cast Bless on that stack (note whether the cast is made); then press Defend. Afterwards play the battle out (attack with every stack) to its end. A level-up's dialog: take the left skill.
8. Order: day 1, Red's turn: the first reads (E12-5), E12-1, E12-2, E12-3, E12-6, E12-4's Red part; end the turn. Blue's turn: E12-4's Blue part. Then the map is done.
9. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E12-6 — An owned garrison's reveal for its owner at the start (E12-5); a neutral garrison of twenty Pikemen won (E12-6) (ADV-GAR-002, ADV-OWN-001, ADV-FOG-002, GAM-BAT-001)

- Red hero **E12-6** (Ingham, specialty creature Monk): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel

1. **E12-5.** Day 1, Red's turn, first, before any hero moves: Red's garrison G5 covers (29, 31) to (31, 31), its entrance the middle tile (30, 31), empty; no Red hero is near it. Read Red's explored bits of the tiles (30, 28), (30, 27), (33, 31), (34, 31), (32, 33) and (33, 33).
2. Also read every garrison's record now (G5, and V1, G2, G3, G4, G6 of the other rows): owner, stacks, removable, entrance.
3. **E12-6.** Day 1, Red's turn, after E12-3: the neutral garrison G6 covers (22, 22) to (24, 22), its entrance the middle tile (23, 22), with twenty Pikemen. E12-6 stands at (23, 19), north of it, with ten Archangels. Read its points, experience and army and G6's record.
4. Click G6's entrance (23, 22) twice: E12-6 walks onto it. Note every question, dialog and message in order, and **as the battle opens** each side's stacks (the defender's: how many stacks, their counts) and the battle's anti-magic flag. The battle by auto combat. Then note every dialog after it (the exchange: its two bars; close it with OK), G6's record and E12-6's tile, points, experience and army.

**Record**

- E12-5: Red's explored bits: (30, 28); (30, 27); (33, 31); (34, 31); (32, 33); (33, 33): (30, 28): yes; (30, 27): no; (33, 31): yes; (34, 31): no; (32, 33): yes; (33, 33): no

| garrison | entrance | owner | stacks | removable |
|---|---|---|---|---|
| G5 | 30,31 | red | empty | 1 |
| V1 | 8,3 | red | 0: 2 Pikemen | 1 |
| G2 | 19,6 | none | empty | 1 |
| G3 | 29,6 | red | 0: 5 Pikemen | 0 |
| G4 | 9,22 | red | 0: 3 Pikemen | 1 |
| G6 | 23,22 | none | 0: 20 Pikemen | 1 |

- E12-6 before: points; experience; army; G6's record (owner, stacks): points: 2000; experience: 0; army: 0: 10 Archangels; G6: record 5; owner none; stacks 0: 20 Pikemen; removable 1; entrance 23,22
- E12-6, the walk: questions, dialogs and messages before the battle (in order): the path: 23,20 23,21 23,22; questions: none; messages: GENRLTXT:7; battles: 1. at 23,22,0: defender 20 Pikemen, winner attacker
- E12-6, as the battle opens: the defender's stacks (creature, count each); the attacker's; anti-magic flag (+0x53C4): defender: slot 0: 20 Pikemen; attacker: slot 0: 10 Archangels; anti-magic flag: clear (0); sides: attacker red (human 1, here 1); defender none (human 0, here 0)
- E12-6: the battle's result: winner: attacker; the attacker at the end: 10 Archangels; the defender at the end: none
- E12-6 after it: dialogs (the exchange's bars): dialogs: garrison; the bars as it opened: at 23,22: upper empty; lower 0: 10 Archangels; level-ups: none
- E12-6 after OK: G6's record (owner, stacks); E12-6's tile; points; experience; army: G6: record 5; owner red; stacks empty; removable 1; entrance 23,22; tile: 23,22,0; points: 1700; experience: 200; army: 0: 10 Archangels
- anything odd: the date at the start: month 1, week 1, day 1; the game's version: SoD; the difficulty: easy; E12-5: the record order (index by entrance): G5: 4; V1: 0; G2: 1; G3: 2; G4: 3; G6: 5; E12-6: the battlefield: grass

## E12-1 — A move through Red's own vertical garrison to a tile beyond it; then the garrison as the move's end (D-59, ADV-GAR-004, ADV-GAR-003, GAM-DEC-004)

- Red hero **E12-1** (Edric, specialty creature Griffin): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel

1. Day 1, Red's turn, after E12-5's reads: Red's vertical garrison V1 covers (8, 2) to (8, 4), its entrance the middle tile (8, 3), with two Pikemen; rock fills column 8 above it (rows 0 and 1) and below it (rows 5 to 13). E12-1 stands at (6, 3), west of it, with ten Archangels. Read its points and V1's record.
2. Click D1 (10, 3), east of V1, once: note the path shown (its tiles, or that none shows). If none shows, go on with the next step. **If a path shows**, click D1 again: E12-1 walks it. Note every dialog or message on the way and where the walk stops (if a dialog opens, note its bars and close it with OK; if E12-1 then stands short of D1, click D1 twice more), then E12-1's tile and points and V1's record.
3. Click V1's entrance (8, 3) twice: E12-1 steps onto it. Note every dialog and message (the exchange: its two bars), then E12-1's tile and points.
4. In the exchange: click V1's two Pikemen (upper bar, slot 0), then E12-1's empty slot 1 (lower bar). Note both bars after it, then close it with OK. Note V1's record and E12-1's army.

**Record**

- E12-1 before: points; V1's record (owner, stacks): points: 2000; V1: record 0; owner red; stacks 0: 2 Pikemen; removable 1; entrance 8,3
- the path to D1 (10, 3) (its tiles, or none): 7,3 8,3 9,3 10,3
- the walk to D1: dialogs and messages on the way; where it stopped; E12-1's tile; points; V1's record: clicked again: yes; dialogs: none; the exchanges' bars as they opened: none; messages: none; tile: 10,3,0; points: 1600; V1: record 0; owner red; stacks 0: 2 Pikemen; removable 1; entrance 8,3
- the step onto V1's entrance (8, 3): dialogs and messages (the exchange's bars); E12-1's tile; points: dialogs: garrison; the bars as it opened: at 8,3: upper 0: 2 Pikemen; lower 0: 10 Archangels; messages: none; the path: 8,3 9,3; tile: 8,3,0; points: 1400
- the take of V1's Pikemen into slot 1: both bars after: upper (V1's record): empty; lower (E12-1's army): 0: 10 Archangels, 1: 2 Pikemen; the upper bar shows: none; the lower bar shows: 0: 10, 1: 2; selected before: none; the clicks: upper:0: upper 0 selected, lower:1: none selected; messages: none; creature windows: none
- after OK: V1's record (owner, stacks); E12-1's army: V1: record 0; owner red; stacks empty; removable 1; entrance 8,3; E12-1's army: 0: 10 Archangels, 1: 2 Pikemen
- anything odd: the walk to D1: the marks' colours: green

## E12-2 — An empty neutral garrison on the way: a path beyond it, the path to it, its visit (D-59, ADV-GAR-002, ADV-GAR-004, ADV-MOV-008)

- Red hero **E12-2** (Christian, specialty creature Ballista): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel

1. Day 1, Red's turn, after E12-1: the empty neutral garrison G2 covers (18, 6) to (20, 6), its entrance the middle tile (19, 6); rock closes the box below it (columns 15 to 23, rows 6 to 10: inside, columns 16 to 22, rows 7 to 9), so the box's only way in is through G2. E12-2 stands at (19, 4), north of G2, with ten Archangels. Read its points and G2's record.
2. Click D2 (19, 8), inside the box, once: note the path shown (its tiles, or that none shows). If none shows, go on with the next step. **If a path shows**, click D2 again: E12-2 walks it. Note every dialog or message on the way (an exchange: its bars, then OK) and where the walk stops, E12-2's tile and points and G2's record; then skip the next step.
3. Click G2's entrance (19, 6) once: note the path shown. Click it again: E12-2 steps onto it. Note every battle, dialog and message in order (the exchange: its two bars), then close the exchange with OK. Note G2's record, E12-2's tile and points.

**Record**

- E12-2 before: points; G2's record (owner, stacks): points: 2000; G2: record 1; owner none; stacks empty; removable 1; entrance 19,6
- the path to D2 (19, 8) (its tiles, or none); if one showed, the walk: dialogs and messages, where it stopped: the path: none; clicked again: no; dialogs: none; the exchanges' bars as they opened: none; messages: none; tile: 19,4,0; points: 2000; G2: record 1; owner none; stacks empty; removable 1; entrance 19,6
- the path to G2's entrance (19, 6) (its tiles): 19,5 19,6
- the step onto (19, 6): battles, dialogs and messages in order (the exchange's bars): battles: none; dialogs: garrison; the bars as it opened: at 19,6: upper empty; lower 0: 10 Archangels; messages: none
- after OK: G2's record (owner, stacks); E12-2's tile; points: G2: record 1; owner red; stacks empty; removable 1; entrance 19,6; tile: 19,6,0; points: 1800
- anything odd: 

## E12-3 — Red's own garrison not removable: a move into an empty slot, a merge onto its stack, two takes (D-61, ADV-GAR-003, GAM-DEC-004)

- Red hero **E12-3** (Rion, specialty skill First Aid): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman, 5 Archer, 2 Griffin

1. Day 1, Red's turn, after E12-2: Red's garrison G3 covers (28, 6) to (30, 6), its entrance the middle tile (29, 6), with five Pikemen; it is **not removable** (the map's flag). E12-3 stands at (29, 4), north of it, with ten Pikemen, five Archers and two Griffins (slots 0, 1, 2). Read its points and G3's record.
2. Click G3's entrance (29, 6) twice: E12-3 steps onto it. Note every dialog and message (the exchange: its two bars), then E12-3's tile and points.
3. In the exchange, four tries in this order, noting before each which stack is highlighted and after each both bars (and any message; if a creature's window opens, note it, close it and go on): (a) **a move**: click E12-3's Archers (lower bar, slot 1), then G3's empty slot 1 (upper bar); (c) **a take**: click G3's Pikemen (upper slot 0), then E12-3's empty slot 3 (lower bar); (d) **a take back**: click the Archers in G3's slot 1 (if they are there), then E12-3's empty slot 4; (b) **a merge**: click E12-3's Pikemen (lower slot 0), then G3's Pikemen (upper slot 0). Close it with OK; note G3's record and E12-3's army.

**Record**

- E12-3 before: points; G3's record (owner, stacks, removable): points: 1560; G3: record 2; owner red; stacks 0: 5 Pikemen; removable 0; entrance 29,6
- the step onto (29, 6): dialogs and messages (the exchange's bars); E12-3's tile; points: dialogs: garrison; the bars as it opened: at 29,6: upper 0: 5 Pikemen; lower 0: 10 Pikemen, 1: 5 Archers, 2: 2 Griffins; messages: none; tile: 29,6,0; points: 1360

| try (a, c, d, b) | highlighted before | the upper bar after | the lower bar after | messages |
|---|---|---|---|---|
| (a) a move: E12-3's Archers (lower 1) to G3's empty slot 1 (upper 1) | none | G3: 0: 5 Pikemen, 1: 5 Archers; shows: 0: 5, 1: 5; the clicks: lower:1: lower 1 selected, upper:1: none selected | E12-3: 0: 10 Pikemen, 2: 2 Griffins; shows: 0: 10, 2: 2 | none; none |
| (c) a take: G3's Pikemen (upper 0) to E12-3's empty slot 3 (lower 3) | none | G3: 0: 5 Pikemen, 1: 5 Archers; shows: 0: 5, 1: 5; the clicks: upper:0: upper 0 selected, lower:3: upper 0 selected | E12-3: 0: 10 Pikemen, 2: 2 Griffins; shows: 0: 10, 2: 2 | none; none |
| (d) a take back: the Archers in G3's slot 1 (upper 1) to E12-3's empty slot 4 (lower 4) | upper 0 | G3: 0: 5 Pikemen, 1: 5 Archers; shows: 0: 5, 1: 5; the clicks: upper:1: upper 1 selected, lower:4: upper 1 selected | E12-3: 0: 10 Pikemen, 2: 2 Griffins; shows: 0: 10, 2: 2 | none; none |
| (b) a merge: E12-3's Pikemen (lower 0) onto G3's Pikemen (upper 0) | upper 1 | G3: 0: 5 Pikemen, 1: 5 Archers; shows: 0: 5, 1: 5; the clicks: lower:0: lower 0 selected, upper:0: upper 0 selected | E12-3: 0: 10 Pikemen, 2: 2 Griffins; shows: 0: 10, 2: 2 | none; none |

- after OK: G3's record (stacks); E12-3's army: G3: record 2; owner red; stacks 0: 5 Pikemen, 1: 5 Archers; removable 0; entrance 29,6; E12-3's army: 0: 10 Pikemen, 2: 2 Griffins
- anything odd: 

## E12-4 — A hero in Red's anti-magic garrison: attacking out of it (Red's turn), attacked in it (Blue's turn) (D-60, ADV-GAR-005, BAT-TER-002, GAM-BAT-004)

- Red hero **E12-4** (Sorsha, specialty creature Swordsman): attack 5, defense 5, power 1, knowledge 2; no skills; Spell Book (spell book); spells Bless; army 10 Archangel
- Blue hero **E12-4b** (Ryland, specialty creature Dendroid Guard): attack 1, defense 1, power 1, knowledge 1; no skills; Spell Book (spell book); spells Bless; army 1 Pikeman
- Blue hero **E12-4a** (Ufretin, specialty creature Dwarf): attack 1, defense 1, power 1, knowledge 1; no skills; Spell Book (spell book); spells Bless; army 1 Pikeman
- Blue hero **E12-4c** (Ivor, specialty creature Wood Elf): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn, after E12-6: Red's anti-magic garrison G4 covers (8, 22) to (10, 22), its entrance the middle tile (9, 22), with three Pikemen. Blue's hero E12-4b stands at (9, 21), north of the entrance, Blue's E12-4a at (9, 23), south of it; each has one Pikeman, a spell book with Bless and 10 spell points. E12-4 stands at (8, 20) with ten Archangels, a spell book with Bless and 20 spell points. Read E12-4's points and spell points and G4's record.
2. Click G4's entrance (9, 22) twice: E12-4 steps onto it. Note every dialog and message (the exchange: its bars), close the exchange with OK without a move; note E12-4's tile and points.
3. **Battle 2 (attacking out of it):** click E12-4b (9, 21) twice: E12-4 attacks it. As the battle opens note its anti-magic flag and each side's stacks. At each side's first stack's turn: the spell book (opens, or the message's text; if it opens, cast Bless on that stack: made or not), then Defend; then play it out. Note the result, then E12-4's tile, points and spell points. End Red's turn.
4. **Battle 1 (attacked in it), Blue's turn:** if E12-4 stands on (9, 22), click it with E12-4a (from (9, 23)) twice: E12-4a attacks it. As the battle opens note its anti-magic flag and each side's stacks (Red's: whether G4's Pikemen are among them). At each side's first stack's turn: the spell book as in battle 2, then Defend; then play it out. Note the result and G4's record after.

**Record**

- E12-4 before: points; spell points; G4's record (owner, stacks): points: 2000; spell points: 20; G4: record 3; owner red; stacks 0: 3 Pikemen; removable 1; entrance 9,22
- the step onto (9, 22): dialogs and messages (the exchange's bars); E12-4's tile; points: the path: 8,21 9,22; dialogs: garrison; the bars as it opened: at 9,22: upper 0: 3 Pikemen; lower 0: 10 Archangels; messages: none; tile: 9,22,0; points: 1759
- battle 2 as it opens: the anti-magic flag (+0x53C4); Red's stacks; Blue's stacks: anti-magic flag: clear (0); Red's (attacker): slot 0: 10 Archangels; Blue's (defender): slot 0: 1 Pikeman; the hero's tile as it opened: 9,22,0
- battle 2, Red's first turn: the spell book (opened, or the message); Bless cast?: 10 Archangels: the book opened (0x641dcc); cast made; spell points 20 then 15; on the stack after: Bless (1 round); then defends
- battle 2, Blue's first turn: the spell book (opened, or the message); Bless cast?: 1 Pikeman: the book opened (0x641dcc); cast made; spell points 10 then 5; on the stack after: Bless (1 round); then defends
- battle 2: the result; then E12-4's tile; points; spell points: winner: attacker; Red at the end: 10 Archangels; messages: none; level-ups: none; tile: 9,22,0; points: 1659; spell points: 15
- battle 1 as it opens: the anti-magic flag (+0x53C4); Red's stacks (G4's Pikemen among them?); Blue's stacks: anti-magic flag: set (1); Red's (defender): slot 0: 10 Archangels; Blue's (attacker): slot 0: 1 Pikeman; E12-4's tile: 9,22,0
- battle 1, Red's first turn: the spell book (opened, or the message); Bless cast?: 10 Archangels: the book refused: GENRLTXT:685; cast not tried; spell points 15 then 15; on the stack after: no spell; then defends
- battle 1, Blue's first turn: the spell book (opened, or the message); Bless cast?: 1 Pikeman: the book refused: GENRLTXT:685; cast not tried; spell points 10 then 10; on the stack after: no spell; then defends
- battle 1: the result; G4's record after (owner, stacks): winner: defender; Red at the end: 10 Archangels; messages: none; G4: record 3; owner red; stacks 0: 3 Pikemen; removable 1; entrance 9,22; E12-4's tile: 9,22,0; E12-4's spell points: 15
- anything odd: E12-4b's and E12-4a's spell points: E12-4b: 10; E12-4a: 10; battle 2: the battlefield: terrain grass, special field none, ship flag 0, blocked hexes 42, 43, 44, 45, 60, 61, 62, 63; Blue's turn: its messages: GENRLTXT:14, ?; battle 1: the battlefield: terrain grass, special field none, ship flag 0, blocked hexes 21, 22, 23, 24, 38, 39, 40, 41, 56, 57, 97, 98; E12-4a before: points; spell points: 1560; 10
