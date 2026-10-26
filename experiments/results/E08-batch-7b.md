# E08 batch E7b — results

Map file: `E08-batch-7b.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `02112650`.** The map's description in the scenario list starts with "Stamp 02112650.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-7b.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1).
4. Most records are the game's memory: an object's word on its tile, a hero's experience, primary and secondary skills, spell points, morale and luck bonuses (+0x11A, +0x11B) and status flags, its worn positions (+0x12D, 8 bytes each), its backpack in order (+0x1D4, the count +0x3D4), its learned spells (+0x3EA), its army (creature ids +0x91, counts +0xAD), its tile and movement points, the players' treasuries, a battle's stacks at its opening (`opening.py`), the path marks (batch E1a's). The capture harness reads them (`clients/capture/batches/E08-batch-7b.toml`, E08 T57). Played by hand, read the screens instead and leave the memory-only lines empty.
5. Positions are the original's numbers: head 0, shoulders 1, neck 2, right hand 3, left hand 4, torso 5, rings 6 and 7, feet 8, misc 9 to 12 (the hero screen's misc 1 to 4) and 18 (misc 5), war machines 13 to 16, spell book 17. Army slots count from 0 at the left. Tiles are (column, row) from 0 at the top left.
6. Answer questions as the encounter says; write each question's or message's text (or its first words) down where the encounter asks for it. The map bans every skill but Wisdom and Learning: a level-up offers those two; take the one the encounter names.
7. Each battle by auto combat unless the encounter says otherwise; a tactics phase is ended at once (its Start button).
8. Order: day 1, Red's turn: E7-12, E7-13, E7-14, E7-15, E7-17, E7-18; end the turn. Blue's turn (save 2 at its start): E7-16, E7-22, E7-21, E7-19, E7-20, E7-23. Then the map is done.
9. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E7-12 — A box's Advanced Wisdom for a hero with Basic Wisdom (D-37, ADV-RWD-001, HER-CHG-003)

- Red hero **E7-12** (Sir Mullich, specialty speed): attack 1, defense 1, power 1, knowledge 1; Basic Wisdom; no artifacts; no spells; army 10 Pikeman

1. Day 1, Red's turn: E7-12 stands at (3, 3) with Basic Wisdom. A Pandora's Box at (6, 3) gives Advanced Wisdom and nothing else.
2. Read E7-12's secondary skills. Click the box twice; note the question and answer **Yes**; note the messages.
3. Read E7-12's secondary skills and whether the box still stands at (6, 3).

**Record**

- skills before: 
- the question; the messages (first words): 
- skills after: 
- the box on (6, 3) after (yes or no): 
- anything odd: 

## E7-13 — A box's 500 mana for a hero at its maximum of 100 (D-38, ADV-RWD-001, HER-CHG-003)

- Red hero **E7-13** (Adrienne, specialty none): attack 0, defense 0, power 0, knowledge 10; no skills; no artifacts; no spells; army 10 Pikeman

1. Day 1, Red's turn: E7-13 stands at (3, 6) with Knowledge 10 and no Intelligence: 100 spell points, its maximum. A Pandora's Box at (6, 6) gives 500 mana and nothing else.
2. Read E7-13's spell points. Click the box twice; answer **Yes**; note the messages.
3. Read E7-13's spell points.

**Record**

- spell points before: 
- the messages (first words): 
- spell points after: 
- anything odd: 

## E7-14 — A box's Expert Learning and 1 000 experience for a hero without Learning (D-39, ADV-RWD-001, HER-EXP-001)

- Red hero **E7-14** (Gelare, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. Day 1, Red's turn: E7-14 stands at (3, 9) without secondary skills and with 0 experience. A Pandora's Box at (6, 9) gives Expert Learning and 1 000 experience.
2. Read E7-14's experience and skills. Click the box twice; answer **Yes**; note the messages (the parts shown and their amounts).
3. A level-up (level 2): note the offers and take **Wisdom**.
4. Read E7-14's experience and skills.

**Record**

- experience and skills before: 
- the messages (the experience shown): 
- the level-up's offers: 
- experience and skills after: 
- anything odd: 

## E7-15 — A box's level-3 spell with the experience of a level that offers Wisdom (D-39, ADV-RWD-001, HER-CHG-003)

- Red hero **E7-15** (Grindan, specialty resource gold, 999 experience): attack 1, defense 1, power 1, knowledge 1; no skills; Spell Book (spell book); no spells; army 10 Pikeman

1. Day 1, Red's turn: E7-15 stands at (3, 12) with a Spell Book, no spells, no secondary skills (no Wisdom: spells up to level 2) and 999 experience (level 1; level 2 at 1 000). A Pandora's Box at (6, 12) gives 1 experience and Fireball (level 3).
2. Read E7-15's spells, skills and experience. Click the box twice; answer **Yes**; note the messages (the parts shown, in order).
3. The level-up (level 2): note the offers and take **Wisdom**.
4. Read E7-15's spells, skills and experience.

**Record**

- before: spells, skills, experience: 
- the messages (the parts shown, in order): 
- the level-up's offers; the one taken: 
- after: spells, skills, experience: 
- anything odd: 

## E7-17 — An event's stop: Red's fires and ends the move on it; Blue's does nothing (ADV-OBJ-007, ADV-MOV-011)

- Red hero **E7-17** (Valeska, specialty creature Archer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman
- Red hero **E7-17b** (Edric, specialty creature Griffin): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. Day 1, Red's turn: E7-17 stands at (12, 3), its second hero E7-17b at (12, 6). Hidden events: at (16, 3) Red's (100 gold, removed after it fires), at (16, 6) Blue's only (100 gold).
2. Read E7-17's tile and movement points and Red's gold. Click (16, 3) once (note the path's tiles), then again: the hero walks. Note any message.
3. Read E7-17's tile and movement points and Red's gold.
4. Read E7-17b's tile and movement points and Red's gold. Click (16, 6) once (note the path's tiles), then again. Note any message.
5. Read E7-17b's tile and movement points and Red's gold.

**Record**

- E7-17 before: tile, movement points, gold: 
- E7-17's path (tiles); the message: 
- E7-17 after: tile, movement points, gold: 
- E7-17b before: tile, movement points, gold: 
- E7-17b's path (tiles); the message (or none): 
- E7-17b after: tile, movement points, gold: 
- anything odd: 

## E7-18 — An event without the removal flag: it fires at every entry (ADV-OBJ-007)

- Red hero **E7-18** (Sorsha, specialty creature Swordsman): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. Day 1, Red's turn: E7-18 stands at (12, 9). A hidden event of Red's at (14, 9) gives 100 gold and stays after it fires.
2. Read Red's gold. Click (14, 9) twice: the hero enters it. Note the message; read Red's gold and E7-18's tile.
3. Click (15, 9) twice (step off east). Then click (14, 9) twice again; note the message; read Red's gold and E7-18's tile.

**Record**

- gold before: 
- first entry: the message; gold; tile: 
- second entry: the message; gold; tile: 
- anything odd: 

## E7-16 — The Pyramid's golems against a weak hero (and E7-22's strong one) (D-40, ADV-MON-012, ADV-TCH-008)

- Blue hero **E7-16** (Jenova, specialty resource gold): attack 0, defense 0, power 0, knowledge 0; no skills; no artifacts; no spells; army 1 Pikeman

1. Blue's turn (save 2 at its start): E7-16 stands at (29, 20) with one Pikeman. The Pyramid's bottom-right corner is at (26, 20) (it covers (25, 20) too). Read the Pyramid's word.
2. Click the Pyramid twice; note the question and answer **Yes**; note the message.
3. At the battle's opening read every stack (the capture harness's `opening.py`: side, slot, hex, creature, count). Then **retreat** (the Retreat button, confirm).
4. E7-22 then fights the same Pyramid with a strong army: its opening is recorded there.

**Record**

- the Pyramid's word before (hex): 
- the question; the message: 

| side | slot | hex | creature | count |
|---|---|---|---|---|
|   |   |   |   |   |
|   |   |   |   |   |
|   |   |   |   |   |
|   |   |   |   |   |
|   |   |   |   |   |
|   |   |   |   |   |
|   |   |   |   |   |
|   |   |   |   |   |
|   |   |   |   |   |

- after the retreat: the Pyramid's word: 
- anything odd: 

## E7-22 — The Pyramid won by a hero with Advanced Wisdom: its level-5 spell (ADV-TCH-008, D-40, GAM-DEC-007)

- Blue hero **E7-22** (Octavia, specialty resource gold): attack 2, defense 2, power 1, knowledge 1; Advanced Wisdom; Spell Book (spell book); no spells; army 20 Archangel, 10 Titan

1. Blue's turn, after E7-16: E7-22 stands at (29, 22) with a Spell Book, no spells, Advanced Wisdom (spells up to level 4) and a strong army. Read its spells and the Pyramid's word (bits 13 to 20: the spell).
2. Click the Pyramid twice; answer **Yes**. At the battle's opening read every stack (`opening.py`); then auto combat.
3. The level-ups after the battle: note each one's offers and take **Learning** every time (never Wisdom: Expert Wisdom would teach the level-5 spell). Note the messages after them.
4. Read E7-22's spells, skills and the Pyramid's word.

**Record**

- before: spells; the Pyramid's word (hex): 

| side | slot | hex | creature | count |
|---|---|---|---|---|
|   |   |   |   |   |
|   |   |   |   |   |
|   |   |   |   |   |
|   |   |   |   |   |
|   |   |   |   |   |
|   |   |   |   |   |
|   |   |   |   |   |
|   |   |   |   |   |
|   |   |   |   |   |

- the battle's result; the level-ups (offers, the one taken): 
- the messages after the level-ups (first words): 
- after: spells, skills; the Pyramid's word (hex): 
- anything odd: 

## E7-21 — A looted Pyramid visited twice, then a battle: the luck bonus (ADV-TCH-008, HER-ADV-008, GAM-BAT-014)

- Blue hero **E7-21** (Aine, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 20 Pikeman, 10 Centaur
- Opponent: 5 Pikeman (wandering monster, never flees)

1. Blue's turn, after E7-22 (the Pyramid looted): E7-21 stands at (29, 18) with Pikemen and Centaurs, no skills, no artifacts. Five savage Pikemen stand east of it at (30, 18).
2. Read E7-21's luck bonus (+0x11B) and status flags. Walk west (away from the Pikemen) and visit the Pyramid: click it twice, answer **Yes**; note the messages; read the luck bonus and status flags.
3. Visit it again the same way; read the luck bonus and status flags.
4. Attack the Pikemen. At the battle's opening read the stacks' luck (`opening.py`, or the creature windows); then auto combat.
5. After the battle: read E7-21's luck bonus and status flags.

**Record**

- before: luck bonus, status flags (hex): 
- first visit: the question, messages; luck bonus, status flags after: 
- second visit: messages; luck bonus, status flags after: 
- the opening: each of E7-21's stacks' luck: 
- after the battle: luck bonus, status flags: 
- anything odd: 

## E7-19 — A box's artifact with a full backpack (ADV-RWD-001, ADV-PIK-013)

- Blue hero **E7-19** (Damacon, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; the backpack filled to 64 with Centaurs Axe; no spells; army 10 Pikeman

1. Blue's turn: E7-19 stands at (20, 3) with 64 Centaurs Axes in its backpack, nothing worn. A Pandora's Box at (23, 3) gives a Centaurs Axe.
2. Read E7-19's backpack count and worn positions. Click the box twice; answer **Yes**; note the messages.
3. Read E7-19's backpack count and worn positions and whether the box still stands at (23, 3).

**Record**

- before: backpack count, worn positions: 
- the messages (first words): 
- after: backpack count, worn positions: 
- the box on (23, 3) after (yes or no): 
- anything odd: 

## E7-20 — A box's three stacks for a hero with seven: the army dialog (ADV-RWD-001, GAM-DEC-004, ADV-PIK-013)

- Blue hero **E7-20** (Clavius, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman, 10 Archer, 5 Griffin, 5 Swordsman, 3 Monk, 2 Cavalier, 1 Angel

1. Blue's turn: E7-20 stands at (20, 6) with seven stacks. A Pandora's Box at (23, 6) gives 5 Wood Elves, 3 Dendroid Guards and 2 Unicorns.
2. Read E7-20's army. Click the box twice; answer **Yes**; note the messages.
3. If a dialog with two armies opens: note its title and the upper army; click E7-20's Monks (slot 4, the lower bar) twice, dismiss them (**Yes**), then click the Wood Elves in the upper bar and the emptied slot 4; press **OK**.
4. Read E7-20's army and whether the box still stands at (23, 6).

**Record**

- army before: 
- the messages; the dialog's title (or: no dialog): 
- the dialog's upper army: 
- army after: 
- the box on (23, 6) after (yes or no): 
- anything odd: 

## E7-23 — A path and a hidden event: around it in the open, past it in a dead-end corridor (A-23, ADV-MOV-011)

- Blue hero **E7-23** (Nagash, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. Blue's turn: E7-23 stands at (8, 30). East of it a corridor one tile wide, (10, 30) to (20, 30), rock on every side but the west, holds a hidden event of Red's at (14, 30) (it does not fire for Blue). West of it, in the open, a hidden event of Blue's at (4, 30).
2. Click (15, 30) **once**: note the path's tiles (the marks), or that no path is drawn.
3. Click (14, 30) **once**: note the path's tiles.
4. Click (3, 30) **once**: note the path's tiles.
5. Click (4, 30) **once**: note the path's tiles. Never click a tile twice: E7-23 must not move.

**Record**

- to (15, 30): the path's tiles (or: no path): 
- to (14, 30): the path's tiles: 
- to (3, 30): the path's tiles: 
- to (4, 30): the path's tiles: 
- anything odd: 
