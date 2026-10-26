# E08 batch E7a — results

Map file: `E08-batch-7a.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `a01503a7`.** The map's description in the scenario list starts with "Stamp a01503a7.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-7a.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1).
4. Most records are the game's memory: an object's word on its tile, a hero's experience, primary and secondary skills, spell points, morale and luck bonuses (+0x11A, +0x11B) and status flags, its worn positions (+0x12D, 8 bytes each), its backpack in order (+0x1D4, the count +0x3D4), its learned spells (+0x3EA), its army (creature ids +0x91, counts +0xAD), the players' treasuries, the game's Universities (+0x4E3CC). The capture harness reads them (`clients/capture/batches/E08-batch-7a.toml`, E08 T57). Played by hand, read the screens instead and leave the memory-only lines empty.
5. Positions are the original's numbers: head 0, shoulders 1, neck 2, right hand 3, left hand 4, torso 5, rings 6 and 7, feet 8, misc 9 to 12 (the hero screen's misc 1 to 4) and 18 (misc 5), war machines 13 to 16, spell book 17. Army slots count from 0 at the left. Tiles are (column, row) from 0 at the top left.
6. Answer questions as the encounter says; write each question's or message's text (or its first words) down where the encounter asks for it. A level-up: take the offer the encounter names, else the left one.
7. Each battle by auto combat unless the encounter says otherwise; a tactics phase is ended at once (its Start button).
8. Order: at the start (save 1) rows E7-3, E7-8 and E7-9 read the map; nothing is played for them. Day 1, Red's turn: E7-1, E7-2, E7-4, E7-6, E7-5 (a hero out of movement points goes on the next day); end the turn. Blue's turn (save 2 at its start): E7-10, E7-11. Further days only for E7-5 and E7-10 if their heroes need them. Then the map is done.
9. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E7-1 — A guarded artifact without a text: a question or the battle (D-29, ADV-PIK-007, D-30)

- Red hero **E7-1** (Sylvia, specialty skill Navigation): attack 2, defense 2, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Angel

1. Day 1, Red's turn: E7-1 stands at (4, 4) wearing and carrying nothing. A Centaurs Axe lies at (7, 4), guarded by 10 Pikemen; its message record has no text.
2. Click the axe once (the path), then once more: the hero walks to (6, 4).
3. Note what follows the step: a question (its text or first words) or the battle at once. A question: answer **No**.
4. After No: note whether the axe still lies at (7, 4) and read E7-1's worn positions and backpack. Click the axe again and answer **Yes**: the battle, auto combat.
5. After the battle: read E7-1's army, worn positions and backpack, and whether the axe still lies at (7, 4).

**Record**

- after the step (the question's text, or: the battle at once): 
- after No: the axe on (7, 4) (yes or no); E7-1's artifacts: 
- the battle's result; E7-1's army after: 
- worn positions after (position: artifact): 
- backpack after: 
- the axe on (7, 4) after (yes or no): 
- anything odd: 

## E7-2 — Where a pickup's artifact goes: a Centaurs Axe, then a Spell Scroll (D-30, ADV-PIK-006, ADV-PIK-008, HER-CHG-002)

- Red hero **E7-2** (Sorsha, specialty creature Swordsman): attack 2, defense 2, power 1, knowledge 1; no skills; no artifacts; no spells; army 2 Angel

1. Day 1, Red's turn: E7-2 stands at (4, 8) wearing and carrying nothing (no Spell Book). A Centaurs Axe lies at (7, 8), a Spell Scroll (Magic Arrow) at (10, 8), neither guarded.
2. Click the axe twice; note the message. Read E7-2's worn positions and backpack.
3. Click the scroll twice; note the message. Read E7-2's worn positions, backpack and learned spells.

**Record**

- the axe's message (first words): 
- worn positions after the axe (position: artifact): 
- backpack after the axe: 
- the scroll's message (first words): 
- worn positions after the scroll: 
- backpack after the scroll: 
- learned spells after the scroll: 
- anything odd: 

## E7-3 — Forty random artifacts (class 65): their artifacts at the new game (D-31, D-32, ADV-OBJ-006)

- Red hero **E7-3** (Valeska, specialty creature Archer): attack 0, defense 0, power 0, knowledge 0; no skills; no artifacts; no spells; army 1 Pikeman

1. At the start (save 1), before anything is played: forty random artifacts (class 65, any class) stand at (4, 40) to (13, 43), ten to a row. Read each one's word on its tile (bits 0 to 9: the artifact).
2. E7-3's hero stands at (2, 44) only to name the row; it plays nothing.

**Record**

| tile | the word (hex) | the artifact |
|---|---|---|
| (4, 40) |   |   |
| (5, 40) |   |   |
| (6, 40) |   |   |
| (7, 40) |   |   |
| (8, 40) |   |   |
| (9, 40) |   |   |
| (10, 40) |   |   |
| (11, 40) |   |   |
| (12, 40) |   |   |
| (13, 40) |   |   |
| (4, 41) |   |   |
| (5, 41) |   |   |
| (6, 41) |   |   |
| (7, 41) |   |   |
| (8, 41) |   |   |
| (9, 41) |   |   |
| (10, 41) |   |   |
| (11, 41) |   |   |
| (12, 41) |   |   |
| (13, 41) |   |   |
| (4, 42) |   |   |
| (5, 42) |   |   |
| (6, 42) |   |   |
| (7, 42) |   |   |
| (8, 42) |   |   |
| (9, 42) |   |   |
| (10, 42) |   |   |
| (11, 42) |   |   |
| (12, 42) |   |   |
| (13, 42) |   |   |
| (4, 43) |   |   |
| (5, 43) |   |   |
| (6, 43) |   |   |
| (7, 43) |   |   |
| (8, 43) |   |   |
| (9, 43) |   |   |
| (10, 43) |   |   |
| (11, 43) |   |   |
| (12, 43) |   |   |
| (13, 43) |   |   |

- anything odd: 

## E7-4 — Six chests and a Learning Stone with Basic Learning: the experience a choice gives (ADV-PIK-005, ADV-TCH-003, HER-EXP-001)

- Red hero **E7-4** (Caitlin, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; Basic Learning; no artifacts; no spells; army 3 Angel

1. Day 1, Red's turn: E7-4 stands at (4, 12) with Basic Learning (and no Learning specialty). Six chests stand at (7, 12) to (17, 12), every second tile; a Learning Stone at (20, 12). Read each chest's word first (bit 10: an artifact; bits 11 to 14: the gold in 500s).
2. For each chest from west to east: read E7-4's experience, click the chest twice, note the two pictures' values (gold and experience; or the artifact) and **take the experience** (the right picture); read the experience again. A level-up between: take the offer that is **not Learning** (the left one if neither is).
3. Then the Learning Stone: read the experience, click it twice, note the message, read the experience again (a level-up as above).
4. Read E7-4's secondary skills at the end (Learning must still be Basic).

**Record**

| chest | its word (hex) | gold shown | experience shown | experience before | experience after |
|---|---|---|---|---|---|
| (7, 12) |   |   |   |   |   |
| (9, 12) |   |   |   |   |   |
| (11, 12) |   |   |   |   |   |
| (13, 12) |   |   |   |   |   |
| (15, 12) |   |   |   |   |   |
| (17, 12) |   |   |   |   |   |

- the Learning Stone: experience before, after, its message: 
- level-ups (level: the two offers, the one taken): 
- secondary skills at the end: 
- anything odd: 

## E7-5 — Chests with a full backpack: an artifact chest's offer (ADV-PIK-005)

- Red hero **E7-5** (Edric, specialty creature Griffin): attack 1, defense 1, power 1, knowledge 1; no skills; the backpack filled to 64 with Centaurs Axe; no spells; army 3 Angel

1. Day 1, Red's turn (after E7-6): E7-5 stands at (24, 6) with 64 Centaurs Axes in its backpack, nothing worn. Forty chests stand at (26, 4) to (44, 10), every second column and row. Read every chest's word at the start (save 1): bit 10 marks an artifact (bits 0 to 9 the artifact), else bits 11 to 14 the gold in 500s.
2. Visit **every chest whose word holds an artifact**, then the three nearest gold chests. At each: read E7-5's backpack count and Red's gold, click the chest twice, note what is shown (the pictures' values, or the artifact) and take the **gold** (the left picture, or OK); read the backpack count and Red's gold again. A hero out of movement points goes on the next day.
3. If no chest's word holds an artifact, write so and visit three gold chests.

**Record**

| chest | its word (hex) | what is shown | backpack before / after | gold before / after |
|---|---|---|---|---|
| (26, 4) |   |   |   |   |
| (28, 4) |   |   |   |   |
| (30, 4) |   |   |   |   |
| (32, 4) |   |   |   |   |
| (34, 4) |   |   |   |   |
| (36, 4) |   |   |   |   |
| (38, 4) |   |   |   |   |
| (40, 4) |   |   |   |   |
| (42, 4) |   |   |   |   |
| (44, 4) |   |   |   |   |
| (26, 6) |   |   |   |   |
| (28, 6) |   |   |   |   |
| (30, 6) |   |   |   |   |
| (32, 6) |   |   |   |   |
| (34, 6) |   |   |   |   |
| (36, 6) |   |   |   |   |
| (38, 6) |   |   |   |   |
| (40, 6) |   |   |   |   |
| (42, 6) |   |   |   |   |
| (44, 6) |   |   |   |   |
| (26, 8) |   |   |   |   |
| (28, 8) |   |   |   |   |
| (30, 8) |   |   |   |   |
| (32, 8) |   |   |   |   |
| (34, 8) |   |   |   |   |
| (36, 8) |   |   |   |   |
| (38, 8) |   |   |   |   |
| (40, 8) |   |   |   |   |
| (42, 8) |   |   |   |   |
| (44, 8) |   |   |   |   |
| (26, 10) |   |   |   |   |
| (28, 10) |   |   |   |   |
| (30, 10) |   |   |   |   |
| (32, 10) |   |   |   |   |
| (34, 10) |   |   |   |   |
| (36, 10) |   |   |   |   |
| (38, 10) |   |   |   |   |
| (40, 10) |   |   |   |   |
| (42, 10) |   |   |   |   |
| (44, 10) |   |   |   |   |

- anything odd: 

## E7-6 — A monster's treasure artifact with a full backpack (ADV-MON-011)

- Red hero **E7-6** (Tyris, specialty creature Cavalier): attack 2, defense 2, power 1, knowledge 1; no skills; the backpack filled to 64 with Centaurs Axe; no spells; army 5 Angel
- Opponent: 10 Pikeman (wandering monster, never flees, its treasure Centaurs Axe)

1. Day 1, Red's turn: E7-6 stands at (4, 16) with 64 Centaurs Axes in its backpack, nothing worn. Ten savage Pikemen east of it hold a treasure: 1 000 gold and a Centaurs Axe.
2. Read E7-6's backpack count and worn positions and Red's gold. Attack the Pikemen (auto combat).
3. Note every message after the battle (first words). Read E7-6's backpack count, worn positions and Red's gold.

**Record**

- before: backpack count, worn positions, gold: 
- the messages after the battle (first words, in order): 
- after: backpack count, worn positions, gold: 
- anything odd: 

## E7-8 — Fifteen random Shrines of the first circle: their spells (D-34, ADV-TCH-004)

- Red hero **E7-8** (Christian, specialty creature Ballista): attack 0, defense 0, power 0, knowledge 0; no skills; no artifacts; no spells; army 1 Pikeman

1. At the start (save 1): fifteen Shrines of the first circle stand at (4, 48) to (32, 48), every second column, their spells drawn at the new game. Read each one's word on its tile (bits 13 to 22: the spell).
2. E7-8's hero stands at (2, 50) only to name the row; it plays nothing.

**Record**

| shrine | the word (hex) | the spell |
|---|---|---|
| (4, 48) |   |   |
| (6, 48) |   |   |
| (8, 48) |   |   |
| (10, 48) |   |   |
| (12, 48) |   |   |
| (14, 48) |   |   |
| (16, 48) |   |   |
| (18, 48) |   |   |
| (20, 48) |   |   |
| (22, 48) |   |   |
| (24, 48) |   |   |
| (26, 48) |   |   |
| (28, 48) |   |   |
| (30, 48) |   |   |
| (32, 48) |   |   |

- anything odd: 

## E7-9 — Thirty Universities: Necromancy among their skills (D-36, ADV-TCH-007)

- Red hero **E7-9** (Ingham, specialty creature Monk): attack 0, defense 0, power 0, knowledge 0; no skills; no artifacts; no spells; army 1 Pikeman

1. At the start (save 1): thirty Universities stand with their bottom-right corners at (40, 40) to (65, 56), six to a row 5 columns apart, the rows 4 apart. Read the game's Universities (+0x4E3CC): each one's four skills.
2. E7-9's hero stands at (36, 58) only to name the row; it plays nothing.

**Record**

| University | its four skills |
|---|---|
| (40, 40) |   |
| (45, 40) |   |
| (50, 40) |   |
| (55, 40) |   |
| (60, 40) |   |
| (65, 40) |   |
| (40, 44) |   |
| (45, 44) |   |
| (50, 44) |   |
| (55, 44) |   |
| (60, 44) |   |
| (65, 44) |   |
| (40, 48) |   |
| (45, 48) |   |
| (50, 48) |   |
| (55, 48) |   |
| (60, 48) |   |
| (65, 48) |   |
| (40, 52) |   |
| (45, 52) |   |
| (50, 52) |   |
| (55, 52) |   |
| (60, 52) |   |
| (65, 52) |   |
| (40, 56) |   |
| (45, 56) |   |
| (50, 56) |   |
| (55, 56) |   |
| (60, 56) |   |
| (65, 56) |   |

- how many offer Necromancy: 
- anything odd: 

## E7-10 — Corpses, Wagons and Warrior's Tombs with a full backpack (ADV-PIK-009, ADV-PIK-010, ADV-PIK-011)

- Blue hero **E7-10** (Rissa, specialty resource mercury): attack 1, defense 1, power 1, knowledge 1; no skills; the backpack filled to 64 with Centaurs Axe; no spells; army 3 Angel

1. Blue's turn (save 2 at its start): E7-10 stands at (24, 18) with 64 Centaurs Axes in its backpack, nothing worn. Twenty Corpses stand at (26, 16) to (44, 18), every second column and row; ten Wagons at (26, 22) to (44, 22), every second column; two Warrior's Tombs with their bottom-right corners at (48, 16) and (51, 16). Read every Corpse's, Wagon's and Tomb's word at the start (save 1).
2. Visit **every Corpse and every Wagon whose word holds an artifact**, then one Corpse and one Wagon without one. At each: read Blue's treasury and E7-10's backpack count, click it twice, note the message (first words), read the treasury, the backpack count and its word again. A hero out of movement points goes on the next day.
3. Then both Tombs: at each, read E7-10's morale bonus (+0x11A) and backpack count, click it twice, answer the question **Yes**, note the messages, read the morale bonus, the backpack count and the Tomb's word again.

**Record**

| Corpse | its word before | message | treasury before / after | backpack before / after | its word after |
|---|---|---|---|---|---|
| (26, 16) |   |   |   |   |   |
| (28, 16) |   |   |   |   |   |
| (30, 16) |   |   |   |   |   |
| (32, 16) |   |   |   |   |   |
| (34, 16) |   |   |   |   |   |
| (36, 16) |   |   |   |   |   |
| (38, 16) |   |   |   |   |   |
| (40, 16) |   |   |   |   |   |
| (42, 16) |   |   |   |   |   |
| (44, 16) |   |   |   |   |   |
| (26, 18) |   |   |   |   |   |
| (28, 18) |   |   |   |   |   |
| (30, 18) |   |   |   |   |   |
| (32, 18) |   |   |   |   |   |
| (34, 18) |   |   |   |   |   |
| (36, 18) |   |   |   |   |   |
| (38, 18) |   |   |   |   |   |
| (40, 18) |   |   |   |   |   |
| (42, 18) |   |   |   |   |   |
| (44, 18) |   |   |   |   |   |

| Wagon | its word before | message | treasury before / after | backpack before / after | its word after |
|---|---|---|---|---|---|
| (26, 22) |   |   |   |   |   |
| (28, 22) |   |   |   |   |   |
| (30, 22) |   |   |   |   |   |
| (32, 22) |   |   |   |   |   |
| (34, 22) |   |   |   |   |   |
| (36, 22) |   |   |   |   |   |
| (38, 22) |   |   |   |   |   |
| (40, 22) |   |   |   |   |   |
| (42, 22) |   |   |   |   |   |
| (44, 22) |   |   |   |   |   |

| Tomb | its word before | the question | messages | morale bonus before / after | backpack before / after | its word after |
|---|---|---|---|---|---|---|
| (48, 16) |   |   |   |   |   |   |
| (51, 16) |   |   |   |   |   |   |

- anything odd: 

## E7-11 — Two Warrior's Tombs, then a battle: the morale bonus (ADV-PIK-011, HER-ADV-008, GAM-BAT-014)

- Blue hero **E7-11** (Calid, specialty resource sulfur): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 20 Pikeman, 10 Centaur
- Opponent: 5 Pikeman (wandering monster, never flees)

1. Blue's turn, after E7-10: E7-11 stands at (24, 28) with Pikemen and Centaurs (two factions: no morale from its army's mix beyond 0), no skills, no artifacts. Five savage Pikemen stand east of it at (25, 28). Two Warrior's Tombs stand west, their bottom-right corners at (21, 28) and (21, 31).
2. Read E7-11's morale bonus (+0x11A) and status flags. Walk west (away from the Pikemen) and visit the first Tomb (click it twice, answer **Yes**); note the messages; read the morale bonus and status flags.
3. Visit the second Tomb the same way; read the morale bonus and status flags.
4. Attack the Pikemen. At the battle's opening read the stacks' morale (the capture harness's `opening.py`, or the creature windows); then auto combat.
5. After the battle: read E7-11's morale bonus and status flags.

**Record**

- before: morale bonus, status flags (hex): 
- first Tomb: messages; morale bonus, status flags after: 
- second Tomb: messages; morale bonus, status flags after: 
- the opening: each of E7-11's stacks' morale: 
- after the battle: morale bonus, status flags: 
- anything odd: 
