# E08 batch E14 — results

Map file: `E08-batch-14.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `7945e78d`.** The map's description in the scenario list starts with "Stamp 7945e78d.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-14.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1), at the start of Blue's turn (save 2) and at the start of Red's day 29 (save 3).
4. Most records are the game's memory: a hero's tile, its movement points (hero +0x4D), army, experience, level, worn artifacts and backpack; both players' treasuries; the Black Markets' records (game +0x1F680, seven artifacts each) and each market's object word; the trade window's screen, pair and slider's most (`market_screen` 0x6AAAA4, `market_kind` 0x6AAAC4, `market_count` 0x6AAA98); the Altar's dialog (+0x78 the total, its places, the slider's most); the Hill Fort's rows (prices, states, the whole +0x298); the questions, messages and dialogs in their order; the path's marks after one click. The capture harness reads them (`clients/capture/batches/E08-batch-14.toml`, E08 T129). Played by hand, read the screens instead and leave the memory-only lines empty.
5. Tiles are (column, row) from 0 at the top left. A walk: click the target once (the path's marks show), then again (the hero goes). Where the procedure says so (E14-16): note the hero's tile while the window (or question) is open, close it as the procedure says, note the tile again, press Space (the visit of the tile the hero stands on), note what opens and the tile, and close it again without a trade.
6. A level-up's dialog: take the left skill (E14-17's offers one). Close every window with its Done or OK button unless the procedure says otherwise.
7. Order: day 1, Red's turn: E14-4's first reads (both Black Markets, before any move), E14-1, E14-4, E14-5, E14-8, E14-9, E14-10, E14-12, E14-15; end the turn. Blue's turn: E14-5's Blue read, E14-13, E14-17; end the turn. Then end both players' turns until Red's day 29 (no hero moves; answer each turn's prompt to end it) and read E14-4's day-29 lines. Then the map is done.
8. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E14-1 — A Freelancer's Guild: 10 Pixies for gold, 2 Vampires for wood (E14-1); the only stack's most (E14-2); where the hero stands (E14-16) (ADV-MKT-001, ADV-MKT-002, ADV-MKT-005, D-71, ADV-OBJ-003)

- Red hero **E14-1** (Orrin, specialty skill Archery): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pixie, 2 Vampire, 10 Skeleton

1. **E14-1.** Day 1, Red's turn, after E14-4's first reads: E14-1 stands at (5, 10) with 10 Pixies (slot 0), 2 Vampires (slot 1) and 10 Skeletons (slot 2). Note its points and Red's treasury. Click the guild G's entrance (5, 8) twice: E14-1 walks onto it and the window opens. Note the window (its screen), E14-1's tile and points.
2. Select the Pixies' slot and gold. Note the slider's most and what one Pixie gives. Set the slider to **10** and Trade. Note Red's treasury and E14-1's army.
3. Select the Vampires' slot and wood. Note the slider's most and what one Vampire gives. Set the slider to **2** and Trade. Note Red's treasury and E14-1's army.
4. **E14-2.** Select the Skeletons' slot (now the army's only stack) and gold. Note the slider's most. Close the window without a trade.
5. **E14-16.** Note E14-1's tile and points. Press Space: note what opens and E14-1's tile; close it.

**Record**

- before: E14-1's points; Red's treasury: 
- at the window: its screen; E14-1's tile; points: 
- the Pixies for gold: the slider's most; what one gives; Red's treasury after 10; the army: 
- the Vampires for wood: the slider's most; what one gives; Red's treasury after 2; the army: 
- E14-2: the Skeletons' most (gold): 
- after closing: E14-1's tile; points: 
- Space: what opens; E14-1's tile: 
- anything odd: 

## E14-4 — The two Black Markets' seven at day 1 and day 29 on a map with bans (E14-3); a purchase by a hero with no room for it (E14-4); where the hero stands (E14-16) (ADV-MKT-006, ADV-MKT-007, A-46, GAM-SET-003, ADV-OBJ-003)

- Red hero **E14-4** (Valeska, specialty creature Archer): attack 1, defense 1, power 1, knowledge 1; no skills; Dragonbone Greaves (feet), Helm of Chaos (head), Targ of the Rampaging Ogre (left hand), Ring of Conjuring (left ring), Clover of Fortune (misc 1), Cards of Prophecy (misc 2), Ladybird of Luck (misc 3), Badge of Courage (misc 4), Crest of Valor (misc 5), Pendant of Dispassion (neck), Ogre's Club of Havoc (right hand), Diplomat's Ring (right ring), Dragon Wing Tabard (shoulders), Scales of the Greater Basilisk (torso), the backpack filled to 64 with Glyph of Gallantry; no spells; army 5 Archer

1. **E14-3.** Day 1, Red's turn, first, before any hero moves: read both Black Markets' records (M1, entrance (12, 8); M2, entrance (18, 8)): the seven artifacts in order (name; class) and each object's word (the record's index).
2. **E14-4.** E14-4 stands at (12, 10) wearing an artifact in every body position (head to feet, the four misc and the fifth) with 64 in the backpack. Note its points, every worn artifact, the backpack's count and Red's treasury. Click M1's entrance (12, 8) twice: E14-4 walks onto it and the window opens. Note the window's entries, E14-4's tile and points.
3. Select M1's **first** entry and gold. Note its price. Click Trade once. Note every message and question in order, Red's treasury, E14-4's worn artifacts and backpack (its count and its last entry) and the window's entries. Close the window. Read M1's record.
4. **E14-16.** Note E14-4's tile and points. Press Space: note what opens and the tile; close it.
5. **Day 29**, Red's turn, before any move (after both players' turns were ended 28 times): read both markets' records again, and the date.

**Record**

| market and day | the seven (name; class) | the word's record index |
|---|---|---|
| M1 (12, 8), day 1 |   |   |
| M2 (18, 8), day 1 |   |   |
| M1, day 29 |   |   |
| M2, day 29 |   |   |

- before: E14-4's points; worn artifacts; the backpack's count; Red's treasury: 
- at the window: the entries shown; E14-4's tile; points: 
- M1's first entry: its name and gold price: 
- after Trade: messages and questions (in order); Red's treasury: 
- after Trade: E14-4's worn artifacts; the backpack's count and last entry; the window's entries: 
- M1's record after the purchase: 
- after closing: E14-4's tile; points; Space: what opens; the tile: 
- day 29: the date: 
- anything odd: 

## E14-5 — A Trading Post's trades (E14-5); a transfer to Blue at the snow Trading Post (E14-6); where the hero stands at both (E14-16) (ADV-MKT-001, ADV-MKT-002, ADV-MKT-003, ADV-MKT-004, ADV-OBJ-003)

- Red hero **E14-5** (Edric, specialty creature Griffin): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. **E14-5.** Day 1, Red's turn, after E14-4: E14-5 stands at (24, 10). Note its points and Red's treasury. Click the Trading Post P1's entrance (24, 8) twice: E14-5 walks onto it and the window opens. Note the window (its screen), E14-5's tile and points.
2. Give **wood**, want **gold**: note the rate shown and the slider's most; set **2** (wood) and Trade; note Red's treasury.
3. Give **gold**, want **wood**: note the rate and the slider's most; set **1** (wood) and Trade; note Red's treasury.
4. Give **mercury**, want **gems**: note the rate and the slider's most; set **1** (gem) and Trade; note Red's treasury. Close the window.
5. **E14-16.** Note E14-5's tile and points. Press Space: note what opens and the tile; close it.
6. **E14-6.** Click the snow Trading Post P2's entrance (26, 10) twice: E14-5 walks onto it (note the path's marks after the first click) and the window opens. Note E14-5's tile and points. Read **both** treasuries (Red's and Blue's). Open the transfer's screen: note the recipients offered. Choose **ore**, the recipient **Blue**, the amount **5** and Trade. Read both treasuries again. Close the window; note the tile; press Space: note what opens and the tile; close it.
7. **Blue's turn** (day 1), first: read Blue's treasury.

**Record**

- before: E14-5's points; Red's treasury: 
- P1's window: its screen; E14-5's tile; points: 
- wood → gold: the rate; the most; Red's treasury after 2: 
- gold → wood: the rate; the most; Red's treasury after 1: 
- mercury → gems: the rate; the most; Red's treasury after 1: 
- after closing P1: E14-5's tile; points; Space: what opens; the tile: 
- P2: the path's marks; at the window its screen; E14-5's tile; points: 
- before the transfer: Red's treasury; Blue's treasury; the recipients offered: 
- after the transfer: Red's treasury; Blue's treasury: 
- after closing P2: the tile; Space: what opens; the tile: 
- Blue's turn: Blue's treasury: 
- anything odd: 

## E14-8 — A step from above toward an Altar's entrance (E14-16); a Castle hero's Altar (E14-7); a treasure, a minor, a major and a relic sacrificed (E14-8) (ADV-ALT-001, ADV-ALT-002, ADV-OBJ-004, ADV-OBJ-003, HER-EXP-001)

- Red hero **E14-8** (Sorsha, specialty creature Swordsman, 68700 experience): attack 1, defense 1, power 1, knowledge 1; Expert Archery, Expert Armorer, Expert Offense, Expert Resistance, Expert Ballistics, Expert Artillery, Expert Tactics, Expert First Aid; Charm of Mana (backpack), Emblem of Cognizance (backpack), Garniture of Interference (backpack), Tome of Fire Magic (backpack); no spells; army 5 Pikeman

1. **E14-16.** Day 1, Red's turn, after E14-5: E14-8 stands at (6, 17), right above the Altar A1's entrance (6, 18). Note its points. Click the entrance (6, 18) **once**: note the path's marks (the tiles, in order). Click it again: E14-8 walks; the Altar's dialog opens. Note E14-8's tile and points.
2. **E14-7.** Note the screen the dialog opens on and whether its button to the other screen is enabled.
3. **E14-8.** Note E14-8's experience and level. Put the four backpack artifacts (a treasure, a minor, a major and a relic) on the altar (the backpack's button, or each by hand). Note the total and the experience the dialog says the hero lacks for its next level. Click Sacrifice. Note every message and dialog in order, E14-8's experience, level, worn artifacts and backpack, and whether the Altar is still open. Click Done.
4. **E14-16.** Note E14-8's tile and points. Press Space: note what opens and the tile; close it with Done.

**Record**

- before: E14-8's points: 
- the first click's path (the marked tiles in order): 
- at the dialog: E14-8's tile; points: 
- E14-7: the first screen; the other screen's button (enabled or not): 
- before: experience; level; on the altar: the total; the experience lacking: 
- after Sacrifice: messages and dialogs (in order); experience; level; artifacts; the Altar still open: 
- after Done: E14-8's tile; points; Space: what opens; the tile: 
- anything odd: 

## E14-9 — A Fortress hero's Altar (E14-7); the Ballista, a Spell Scroll, the Grail and a treasure sacrificed, the Catapult clicked (E14-9) (ADV-ALT-001, ADV-ALT-002, D-72, A-47)

- Red hero **E14-9** (Bron, specialty creature Basilisk, 2100 experience): attack 1, defense 1, power 1, knowledge 1; Expert Archery, Expert Armorer, Expert Offense, Expert Resistance, Expert Ballistics, Expert Artillery, Expert Tactics, Expert First Aid; Ballista (war machine 1), Catapult (war machine 4), The Grail (backpack), Talisman of Mana (backpack); no spells; army 5 Gnoll

1. **E14-9.** Day 1, Red's turn, after E14-8: E14-9 stands at (12, 20) wearing a Ballista and a Catapult, the Grail and a Talisman of Mana in its backpack. Note its points. Click the Spell Scroll at (13, 20) twice: E14-9 picks it up from its tile. Note every message, E14-9's tile, points, worn artifacts and backpack.
2. Click the Altar A2's entrance (12, 18) twice: E14-9 walks onto it and the dialog opens. **E14-7:** note the screen it opens on and whether its button to the other screen is enabled. Note E14-9's tile and experience.
3. Take the **Ballista** (from its position), the **Spell Scroll**, the **Grail** and the **Talisman of Mana** onto the altar. Note the total. Click the **Catapult**: note what happens (a message?) and where the Catapult is. Click Sacrifice. Note every message, E14-9's experience, level, worn artifacts and backpack. Click Done; note E14-9's tile.

**Record**

- before: E14-9's points; the scroll's messages; after it: tile; points; worn artifacts; backpack: 
- at the dialog: E14-9's tile; points; experience: 
- E14-7: the first screen; the other screen's button (enabled or not): 
- the four on the altar: the total: 
- the Catapult's click: what happens; the Catapult after: 
- after Sacrifice: messages; experience; level; worn artifacts; backpack: 
- after Done: E14-9's tile: 
- anything odd: 

## E14-10 — A Necropolis hero's Altar (E14-7); 3 and 1 Skeletons sacrificed with Advanced Learning (E14-10); the only stack's most and All (E14-11) (ADV-ALT-001, ADV-ALT-003, D-73, HER-EXP-001)

- Red hero **E14-10** (Straker, specialty creature Walking Dead): attack 1, defense 1, power 1, knowledge 1; Advanced Learning; no artifacts; no spells; army 3 Skeleton, 1 Skeleton, 10 Skeleton

1. **E14-10.** Day 1, Red's turn, after E14-9: E14-10 stands at (18, 20) with Advanced Learning and Skeletons in three slots: 3 (slot 0), 1 (slot 1), 10 (slot 2). Click the Altar A3's entrance (18, 18) twice: E14-10 walks onto it and the dialog opens. **E14-7:** note the screen it opens on and whether its button to the other screen is enabled. Note E14-10's tile and experience.
2. Select slot 0 and set its amount to **3**; select slot 1 and set **1** (note its slider's most). Note the total. Click Sacrifice. Note E14-10's experience and army.
3. **E14-11.** Select slot 2 (10 Skeletons, now the only stack): note the slider's most. Click **All**: note every slot's amount and the total. Click Done (no sacrifice). Note E14-10's army and tile.

**Record**

- at the dialog: E14-10's tile; experience: 
- E14-7: the first screen; the other screen's button (enabled or not): 
- slot 1's most; the total of 3 and 1: 
- after Sacrifice: experience; army: 
- E14-11: slot 2's most; All's amounts; the total: 
- after Done: army; tile: 
- anything odd: 

## E14-12 — A Hill Fort's prices and All (E14-12, with four Air Elementals); where the hero stands (E14-16) (ADV-HFT-001, ADV-HFT-002, D-74, ADV-OBJ-003)

- Red hero **E14-12** (Tyris, specialty creature Cavalier): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 20 Pikeman, 10 Archer, 3 Wood Elf, 3 Pegasus, 1 Angel, 4 Air Elemental

1. **E14-12.** Day 1, Red's turn, after E14-10: E14-12 stands at (24, 20) with 20 Pikemen, 10 Archers, 3 Wood Elves, 3 Pegasi, 1 Angel and 4 Air Elementals (slots 0 to 5). Note Red's treasury. Click the Hill Fort H1's entrance (24, 18) twice: E14-12 walks onto it and the dialog opens. Note E14-12's tile and points.
2. Note every row: its price (gold and other resources, or the text) and its state; and the whole's state. Click **All**. Note every message, Red's treasury, E14-12's army and the rows again. Close the dialog.
3. **E14-16.** Note E14-12's tile and points. Press Space: note what opens (and its rows) and the tile; close it.

**Record**

- before: Red's treasury; at the dialog: E14-12's tile; points: 

| slot | stack | price | state |
|---|---|---|---|
| 0 Pikemen |   |   |   |
| 1 Archers |   |   |   |
| 2 Wood Elves |   |   |   |
| 3 Pegasi |   |   |   |
| 4 Angel |   |   |   |
| 5 Air Elementals |   |   |   |

- the whole's state: 
- after All: messages; Red's treasury; army; the rows' states: 
- after closing: tile; points; Space: what opens; the tile: 
- anything odd: 

## E14-15 — A War Machine Factory bought from by a hero with a Ballista, then visited again (E14-15); where the hero stands (E14-16) (ADV-WMF-001, GAM-DEC-008, ADV-HIR-002, ADV-OBJ-003)

- Red hero **E14-15** (Adela, specialty spell Bless): attack 1, defense 1, power 1, knowledge 1; no skills; Ballista (war machine 1); no spells; army 5 Pikeman

1. **E14-15.** Day 1, Red's turn, after E14-12: E14-15 stands at (29, 20) wearing a Ballista. Note Red's treasury. Click the factory F's entrance (29, 18) twice: E14-15 walks onto it. Note every question and E14-15's tile and points at it; answer **yes**.
2. In the dialog note each machine (in order) with its count and price. Buy every machine whose count is 1 (one each). Note Red's treasury and E14-15's war machine positions. Close the dialog.
3. **E14-16.** Note E14-15's tile. Press Space: note the question; answer yes; note each machine's count; close it. Note the tile. Then end Red's turn.

**Record**

- before: Red's treasury; at the question: E14-15's tile; points: 

| machine | count | price |
|---|---|---|
|   |   |   |
|   |   |   |
|   |   |   |

- after the purchases: Red's treasury; the war machine positions: 
- after closing: tile; Space: the question; the counts; the tile after: 
- anything odd: 

## E14-13 — A Hill Fort for a hero too poor for its one upgradable stack (E14-13) (ADV-HFT-001, ADV-HFT-002)

- Blue hero **E14-13** (Mephala, specialty skill Armorer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Rogue, 5 Marksman, 3000 Pegasus

1. **E14-13.** Day 1, Blue's turn, after E14-5's Blue read: E14-13 stands at (24, 28) with 5 Rogues, 5 Marksmen and 3 000 Pegasi. Note Blue's treasury. Click the Hill Fort H2's entrance (24, 26) twice: E14-13 walks onto it and the dialog opens. Note E14-13's tile.
2. Note every row's price and state and the whole's state. Click the Pegasi's row button: note what happens. Click **All**: note what happens. Note Blue's treasury and E14-13's army. Close the dialog.

**Record**

- before: Blue's treasury; at the dialog: E14-13's tile: 

| slot | stack | price | state |
|---|---|---|---|
| 0 Rogues |   |   |   |
| 1 Marksmen |   |   |   |
| 2 Pegasi |   |   |   |

- the whole's state: 
- the Pegasi's button: what happens; All: what happens: 
- after: Blue's treasury; army: 
- anything odd: 

## E14-17 — A treasure sacrificed for a level: the level-up over the Altar, the Altar still open, the creatures' total with the new Learning (E14-17) (ADV-ALT-002, ADV-ALT-003, GAM-DEC-012, GAM-DEC-007, HER-EXP-001)

- Blue hero **E14-17** (Alkin, specialty creature Gorgon): attack 1, defense 1, power 1, knowledge 1; Expert Archery, Expert Armorer, Expert Offense, Expert Resistance, Expert Ballistics, Expert Artillery, Expert Tactics, Basic Learning; Mystic Orb of Mana (backpack); no spells; army 10 Gnoll, 5 Lizardman

1. **E14-17.** Day 1, Blue's turn, after E14-13: E14-17 stands at (12, 28), level 1 with eight secondary skills (seven at Expert, Basic Learning), a Mystic Orb of Mana in its backpack, 10 Gnolls (slot 0) and 5 Lizardmen (slot 1). Note its experience, level and skills. Click the Altar A4's entrance (12, 26) twice: E14-17 walks onto it and the dialog opens. Note the screen and whether the creatures' button is enabled.
2. Put the **Mystic Orb of Mana** on the altar. Note the total and the experience lacking. Click Sacrifice. Note every dialog in order and what is drawn around it (is the Altar's dialog still drawn under the level-up?); at the level-up note the offered skills and take the left one. After it note whether the Altar is still open, E14-17's experience, level and skills.
3. Open the **creatures'** screen. Set the Gnolls' (slot 0) amount to **10**. Note the total. Click Sacrifice. Note E14-17's experience, level and army. Click Done. Then end Blue's turn.

**Record**

- before: experience; level; skills; at the dialog: E14-17's tile; the first screen; the creatures' button: 
- the Mystic Orb: the total; the experience lacking: 
- after Sacrifice: the dialogs (in order) and what is drawn under them; the offered skills: 
- after the level-up: the Altar still open; experience; level; skills: 
- 10 Gnolls: the total; after Sacrifice: experience; level; army: 
- anything odd: 
