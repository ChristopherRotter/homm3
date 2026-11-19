# E08 batch E14 — results

> Played by the capture harness on 2026-11-19 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-14.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

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

- before: E14-1's points; Red's treasury: points: 1560; treasury: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30600 gold
- at the window: its screen; E14-1's tile; points: the path: 5,8 5,9; the window: 4 (creatures sold); kind: 3 (a Freelancer's Guild); count: 5; buttons: screens none; trade disabled; at the window: 5,8 with 1360 points; in order: trade window (screen 4, creatures sold; kind 3, a Freelancer's Guild)
- the Pixies for gold: the slider's most; what one gives; Red's treasury after 10; the army: the pair: slot 0 (10 Pixies) for gold; the most: 10; counts: given; one gives: 1 Pixie for 17 gold; amount: 10; the trade's change: +170 gold; treasury after: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30770 gold; army: 1: 2 Vampires, 2: 10 Skeletons; points: 1360
- the Vampires for wood: the slider's most; what one gives; Red's treasury after 2; the army: the pair: slot 1 (2 Vampires) for wood; the most: 2; counts: given; one gives: 1 Vampire for 1 wood; amount: 2; the trade's change: +2 wood; treasury after: 32 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30770 gold; army: 2: 10 Skeletons
- E14-2: the Skeletons' most (gold): the pair: slot 2 (10 Skeletons) for gold; the most: 9; counts: given; one gives: 1 Skeleton for 42 gold; army: 2: 10 Skeletons
- after closing: E14-1's tile; points: tile: 5,8,0; points: 1360; treasury: 32 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30770 gold
- Space: what opens; E14-1's tile: the key: nothing; in order: trade window (screen 4, creatures sold; kind 3, a Freelancer's Guild); at the window: 5,8 with 1360 points; tile after: 5,8,0; points: 1360
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
| M1 (12, 8), day 1 | Stoic Watchman (treasure), Breastplate of Petrified Wood (treasure), Bird of Perception (treasure), Crown of the Supreme Magi (minor), Blackshard of the Dead Knight (minor), Shield of the Yawning Dead (minor), Sword of Hellfire (major); classes: 3 treasures, 3 minors, 1 major | record: 0; word: 0x00000000 |
| M2 (18, 8), day 1 | Helm of the Alabaster Unicorn (treasure), Spyglass (treasure), Shield of the Dwarven Lords (treasure), Greater Gnoll's Flail (minor), Rib Cage (minor), Red Dragon Flame Tongue (minor), Shield of the Damned (major); classes: 3 treasures, 3 minors, 1 major | record: 1; word: 0x00000001 |
| M1, day 29 | empty, Breastplate of Petrified Wood (treasure), Bird of Perception (treasure), Crown of the Supreme Magi (minor), Blackshard of the Dead Knight (minor), Shield of the Yawning Dead (minor), Sword of Hellfire (major); classes: 2 treasures, 3 minors, 1 major | record: 0; word: 0x00000000 |
| M2, day 29 | Helm of the Alabaster Unicorn (treasure), Spyglass (treasure), Shield of the Dwarven Lords (treasure), Greater Gnoll's Flail (minor), Rib Cage (minor), Red Dragon Flame Tongue (minor), Shield of the Damned (major); classes: 3 treasures, 3 minors, 1 major | record: 1; word: 0x00000001 |

- before: E14-4's points; worn artifacts; the backpack's count; Red's treasury: points: 1630; worn: 0: Helm of Chaos, 1: Dragon Wing Tabard, 2: Pendant of Dispassion, 3: Ogre's Club of Havoc, 4: Targ of the Rampaging Ogre, 5: Scales of the Greater Basilisk, 6: Diplomat's Ring, 7: Ring of Conjuring, 8: Dragonbone Greaves, 9: Clover of Fortune, 10: Cards of Prophecy, 11: Ladybird of Luck, 12: Badge of Courage, 16: Catapult, 18: Crest of Valor; backpack: 64; treasury: 32 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30770 gold
- at the window: the entries shown; E14-4's tile; points: the path: 12,8 12,9; the window: 2 (artifacts bought); kind: 2 (a Black Market); entries: Stoic Watchman, Breastplate of Petrified Wood, Bird of Perception, Crown of the Supreme Magi, Blackshard of the Dead Knight, Shield of the Yawning Dead, Sword of Hellfire; buttons: screens none; trade disabled; at the window: 12,8 with 1430 points; in order: trade window (screen 2, artifacts bought; kind 2, a Black Market)
- M1's first entry: its name and gold price: the pair: gold for entry 0 (Stoic Watchman); the price: 5000 gold for 1 Stoic Watchman; the most: 6; amount: 1; buttons: screens none; trade enabled
- after Trade: messages and questions (in order); Red's treasury: questions: none; messages: none; the trade's change: -5000 gold; treasury: 32 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 25770 gold
- after Trade: E14-4's worn artifacts; the backpack's count and last entry; the window's entries: worn: 0: Helm of Chaos, 1: Dragon Wing Tabard, 2: Pendant of Dispassion, 3: Ogre's Club of Havoc, 4: Targ of the Rampaging Ogre, 5: Scales of the Greater Basilisk, 6: Diplomat's Ring, 7: Ring of Conjuring, 8: Dragonbone Greaves, 9: Clover of Fortune, 10: Cards of Prophecy, 11: Ladybird of Luck, 12: Badge of Courage, 16: Catapult, 18: Crest of Valor; backpack: 64; last: Glyph of Gallantry; the window's entries: empty, Breastplate of Petrified Wood, Bird of Perception, Crown of the Supreme Magi, Blackshard of the Dead Knight, Shield of the Yawning Dead, Sword of Hellfire; the window: 2 (artifacts bought)
- M1's record after the purchase: record 0; empty, Breastplate of Petrified Wood (treasure), Bird of Perception (treasure), Crown of the Supreme Magi (minor), Blackshard of the Dead Knight (minor), Shield of the Yawning Dead (minor), Sword of Hellfire (major); word 0x00000000
- after closing: E14-4's tile; points; Space: what opens; the tile: tile: 12,8,0; points: 1430; Space: nothing; in order: trade window (screen 2, artifacts bought; kind 2, a Black Market); entries: empty, Breastplate of Petrified Wood, Bird of Perception, Crown of the Supreme Magi, Blackshard of the Dead Knight, Shield of the Yawning Dead, Sword of Hellfire; tile after: 12,8,0
- day 29: the date: month 2, week 1, day 1; the Black Market records: 2
- anything odd: the date at the start: month 1, week 1, day 1; the game's version: SoD; the difficulty: easy; E14-4's backpack's last entry at the start: Glyph of Gallantry

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

- before: E14-5's points; Red's treasury: points: 1560; treasury: 32 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 25770 gold
- P1's window: its screen; E14-5's tile; points: the path: 24,8 24,9; the window: 0 (resources for resources); kind: 1 (a Trading Post); count: 5; buttons: screens 17; trade disabled; at the window: 24,8 with 1360 points; in order: trade window (screen 0, resources for resources; kind 1, a Trading Post)
- wood → gold: the rate; the most; Red's treasury after 2: the rate: 1 wood for 75 gold; the most: 32; counts: given; the trade's change: -2 wood, +150 gold; treasury after: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 25920 gold
- gold → wood: the rate; the most; Red's treasury after 1: the rate: 833 gold for 1 wood; the most: 31; counts: wanted; the trade's change: +1 wood, -833 gold; treasury after: 31 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 25087 gold
- mercury → gems: the rate; the most; Red's treasury after 1: the rate: 3 mercury for 1 gems; the most: 5; counts: wanted; the trade's change: -3 mercury, +1 gems; treasury after: 31 wood, 12 mercury, 30 ore, 15 sulfur, 15 crystal, 16 gems, 25087 gold
- after closing P1: E14-5's tile; points; Space: what opens; the tile: tile: 24,8,0; points: 1360; Space: nothing; in order: trade window (screen 0, resources for resources; kind 1, a Trading Post); tile after: 24,8,0
- P2: the path's marks; at the window its screen; E14-5's tile; points: the path: 24,9 24,10 26,10 25,11; marks: 4; the window: 0 (resources for resources); kind: 1 (a Trading Post); buttons: screens 17; trade disabled; at the window: 26,10 with 878 points; in order: trade window (screen 0, resources for resources; kind 1, a Trading Post)
- before the transfer: Red's treasury; Blue's treasury; the recipients offered: red: 31 wood, 12 mercury, 30 ore, 15 sulfur, 15 crystal, 16 gems, 25087 gold; blue: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30800 gold; the screen: 1 (the transfer); the recipients offered: blue; buttons: screens 16; trade disabled; the pair: ore to blue; the rate: 1 ore for 1 to blue; the most: 30
- after the transfer: Red's treasury; Blue's treasury: red: 31 wood, 12 mercury, 25 ore, 15 sulfur, 15 crystal, 16 gems, 25087 gold; red's change: -5 ore; blue: 30 wood, 15 mercury, 35 ore, 15 sulfur, 15 crystal, 15 gems, 30800 gold; blue's change: +5 ore
- after closing P2: the tile; Space: what opens; the tile: tile: 26,10,0; points: 878; Space: nothing; in order: trade window (screen 0, resources for resources; kind 1, a Trading Post); tile after: 26,10,0
- Blue's turn: Blue's treasury: blue: 30 wood, 15 mercury, 35 ore, 15 sulfur, 15 crystal, 15 gems, 30800 gold; the date: month 1, week 1, day 1
- anything odd: 

## E14-8 — A step from above toward an Altar's entrance (E14-16); a Castle hero's Altar (E14-7); a treasure, a minor, a major and a relic sacrificed (E14-8) (ADV-ALT-001, ADV-ALT-002, ADV-OBJ-004, ADV-OBJ-003, HER-EXP-001)

- Red hero **E14-8** (Sorsha, specialty creature Swordsman, 68700 experience): attack 1, defense 1, power 1, knowledge 1; Expert Archery, Expert Armorer, Expert Offense, Expert Resistance, Expert Ballistics, Expert Artillery, Expert Tactics, Expert First Aid; Charm of Mana (backpack), Emblem of Cognizance (backpack), Garniture of Interference (backpack), Tome of Fire Magic (backpack); no spells; army 5 Pikeman

1. **E14-16.** Day 1, Red's turn, after E14-5: E14-8 stands at (6, 17), right above the Altar A1's entrance (6, 18). Note its points. Click the entrance (6, 18) **once**: note the path's marks (the tiles, in order). Click it again: E14-8 walks; the Altar's dialog opens. Note E14-8's tile and points.
2. **E14-7.** Note the screen the dialog opens on and whether its button to the other screen is enabled.
3. **E14-8.** Note E14-8's experience and level. Put the four backpack artifacts (a treasure, a minor, a major and a relic) on the altar (the backpack's button, or each by hand). Note the total and the experience the dialog says the hero lacks for its next level. Click Sacrifice. Note every message and dialog in order, E14-8's experience, level, worn artifacts and backpack, and whether the Altar is still open. Click Done.
4. **E14-16.** Note E14-8's tile and points. Press Space: note what opens and the tile; close it with Done.

**Record**

- before: E14-8's points: tile: 6,17,0; points: 1560
- the first click's path (the marked tiles in order): the path: 6,18 7,18; marks: 2
- at the dialog: E14-8's tile; points: at the dialog: 6,18 with 1319 points; in order: Altar's dialog (the artifacts' screen; allowed artifacts)
- E14-7: the first screen; the other screen's button (enabled or not): the screen: artifacts; allowed: artifacts; the other: the creatures' button disabled
- before: experience; level; on the altar: the total; the experience lacking: experience: 68700; level: 19; backpack: Charm of Mana, Emblem of Cognizance, Garniture of Interference, Tome of Fire Magic; on the altar: 0: Charm of Mana (from the backpack, 1000), 1: Emblem of Cognizance (from the backpack, 1500), 2: Garniture of Interference (from the backpack, 3000), 3: Tome of Fire Magic (from the backpack, 6000); the total: 11500; lacking: 13261; Sacrifice: enabled
- after Sacrifice: messages and dialogs (in order); experience; level; artifacts; the Altar still open: in order: none; experience: 80200; level: 19; worn: Catapult (war machine 4); backpack: none; the screen: altar; the Altar: the artifacts' screen (allowed artifacts; the creatures' button disabled); total 0; lacking 1761; on the altar none; held none; Sacrifice disabled
- after Done: E14-8's tile; points; Space: what opens; the tile: tile: 6,18,0; points: 1319; the Altar after its actions: 0; Space: nothing; in order: Altar's dialog (the artifacts' screen; allowed artifacts); at the dialog: 6,18 with 1319 points; tile after: 6,18,0
- anything odd: the path's order (the record lists its tiles by row, then column): (7, 18), then the entrance (6, 18), 241 points (a diagonal step and a straight one); none from above (E14-16).

## E14-9 — A Fortress hero's Altar (E14-7); the Ballista, a Spell Scroll, the Grail and a treasure sacrificed, the Catapult clicked (E14-9) (ADV-ALT-001, ADV-ALT-002, D-72, A-47)

- Red hero **E14-9** (Bron, specialty creature Basilisk, 2100 experience): attack 1, defense 1, power 1, knowledge 1; Expert Archery, Expert Armorer, Expert Offense, Expert Resistance, Expert Ballistics, Expert Artillery, Expert Tactics, Expert First Aid; Ballista (war machine 1), Catapult (war machine 4), The Grail (backpack), Talisman of Mana (backpack); no spells; army 5 Gnoll

1. **E14-9.** Day 1, Red's turn, after E14-8: E14-9 stands at (12, 20) wearing a Ballista and a Catapult, the Grail and a Talisman of Mana in its backpack. Note its points. Click the Spell Scroll at (13, 20) twice: E14-9 picks it up from its tile. Note every message, E14-9's tile, points, worn artifacts and backpack.
2. Click the Altar A2's entrance (12, 18) twice: E14-9 walks onto it and the dialog opens. **E14-7:** note the screen it opens on and whether its button to the other screen is enabled. Note E14-9's tile and experience.
3. Take the **Ballista** (from its position), the **Spell Scroll**, the **Grail** and the **Talisman of Mana** onto the altar. Note the total. Click the **Catapult**: note what happens (a message?) and where the Catapult is. Click Sacrifice. Note every message, E14-9's experience, level, worn artifacts and backpack. Click Done; note E14-9's tile.

**Record**

- before: E14-9's points; the scroll's messages; after it: tile; points; worn artifacts; backpack: before: points: 1560; the scroll's messages: message ADVEVENT:135; tile: 12,20,0; points: 1460; worn: 9: Spell Scroll, 13: Ballista, 16: Catapult; backpack: The Grail, Talisman of Mana
- at the dialog: E14-9's tile; points; experience: at the dialog: 12,18 with 1260 points; experience: 2100; in order: Altar's dialog (the artifacts' screen; allowed artifacts, creatures)
- E14-7: the first screen; the other screen's button (enabled or not): the screen: artifacts; allowed: artifacts, creatures; the other: the creatures' button enabled
- the four on the altar: the total: on the altar: 0: Ballista (from worn 13, 0), 1: Spell Scroll (Haste) (from worn 9, 0), 2: The Grail (from the backpack, 0), 3: Talisman of Mana (from the backpack, 1000); the total: 1000; lacking: 1100
- the Catapult's click: what happens; the Catapult after: in order: message GENRLTXT:483; held: none; worn: 16: Catapult; on the altar: 0: Ballista (from worn 13, 0), 1: Spell Scroll (Haste) (from worn 9, 0), 2: The Grail (from the backpack, 0), 3: Talisman of Mana (from the backpack, 1000)
- after Sacrifice: messages; experience; level; worn artifacts; backpack: in order: none; experience: 3100; level: 3; worn: Catapult (war machine 4); backpack: none; the Altar: the artifacts' screen (allowed artifacts, creatures; the creatures' button enabled); total 0; lacking 100; on the altar none; held none; Sacrifice disabled
- after Done: E14-9's tile: tile: 12,18,0; points: 1260
- anything odd: 

## E14-10 — A Necropolis hero's Altar (E14-7); 3 and 1 Skeletons sacrificed with Advanced Learning (E14-10); the only stack's most and All (E14-11) (ADV-ALT-001, ADV-ALT-003, D-73, HER-EXP-001)

- Red hero **E14-10** (Straker, specialty creature Walking Dead): attack 1, defense 1, power 1, knowledge 1; Advanced Learning; no artifacts; no spells; army 3 Skeleton, 1 Skeleton, 10 Skeleton

1. **E14-10.** Day 1, Red's turn, after E14-9: E14-10 stands at (18, 20) with Advanced Learning and Skeletons in three slots: 3 (slot 0), 1 (slot 1), 10 (slot 2). Click the Altar A3's entrance (18, 18) twice: E14-10 walks onto it and the dialog opens. **E14-7:** note the screen it opens on and whether its button to the other screen is enabled. Note E14-10's tile and experience.
2. Select slot 0 and set its amount to **3**; select slot 1 and set **1** (note its slider's most). Note the total. Click Sacrifice. Note E14-10's experience and army.
3. **E14-11.** Select slot 2 (10 Skeletons, now the only stack): note the slider's most. Click **All**: note every slot's amount and the total. Click Done (no sacrifice). Note E14-10's army and tile.

**Record**

- at the dialog: E14-10's tile; experience: at the dialog: 18,18 with 1360 points; experience: 0; in order: Altar's dialog (the creatures' screen; allowed creatures)
- E14-7: the first screen; the other screen's button (enabled or not): the screen: creatures; allowed: creatures; the other: the artifacts' button disabled
- slot 1's most; the total of 3 and 1: slot 1's most: 1; amounts: 0: 3, 1: 1; the total: 21; lacking: 1000
- after Sacrifice: experience; army: in order: none; experience: 21; army: 2: 10 Skeletons; the Altar: the creatures' screen (allowed creatures; the artifacts' button disabled); total 0; lacking 979; amounts none; selected none (most none); Sacrifice disabled
- E14-11: slot 2's most; All's amounts; the total: slot 2's most: 9; All's amounts: 2: 9; the total: 49
- after Done: army; tile: army: 2: 10 Skeletons; tile: 18,18,0; experience: 21
- anything odd: 

## E14-12 — A Hill Fort's prices and All (E14-12, with four Air Elementals); where the hero stands (E14-16) (ADV-HFT-001, ADV-HFT-002, D-74, ADV-OBJ-003)

- Red hero **E14-12** (Tyris, specialty creature Cavalier): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 20 Pikeman, 10 Archer, 3 Wood Elf, 3 Pegasus, 1 Angel, 4 Air Elemental

1. **E14-12.** Day 1, Red's turn, after E14-10: E14-12 stands at (24, 20) with 20 Pikemen, 10 Archers, 3 Wood Elves, 3 Pegasi, 1 Angel and 4 Air Elementals (slots 0 to 5). Note Red's treasury. Click the Hill Fort H1's entrance (24, 18) twice: E14-12 walks onto it and the dialog opens. Note E14-12's tile and points.
2. Note every row: its price (gold and other resources, or the text) and its state; and the whole's state. Click **All**. Note every message, Red's treasury, E14-12's army and the rows again. Close the dialog.
3. **E14-16.** Note E14-12's tile and points. Press Space: note what opens (and its rows) and the tile; close it.

**Record**

- before: Red's treasury; at the dialog: E14-12's tile; points: treasury: 31 wood, 12 mercury, 25 ore, 15 sulfur, 15 crystal, 16 gems, 25087 gold; at the dialog: 24,18 with 1360 points; in order: Hill Fort's dialog (the whole 1, affordable)

| slot | stack | price | state |
|---|---|---|---|
| 0 Pikemen | 20 Pikemen | 0 (shown GENRLTXT:345) | 1, affordable |
| 1 Archers | 10 Archers | 125 gold | 1, affordable |
| 2 Wood Elves | 3 Wood Elves | 37 gold | 1, affordable |
| 3 Pegasi | 3 Pegasi | 56 gold | 1, affordable |
| 4 Angel | 1 Angel | 2 gems, 2000 gold | 1, affordable |
| 5 Air Elementals | 4 Air Elementals | 25 gold | 1, affordable |

- the whole's state: 2 gems, 2243 gold, state 1, affordable; buttons: all enabled, row 0 enabled, row 1 enabled, row 2 enabled, row 3 enabled, row 4 enabled, row 5 enabled, row 6 disabled
- after All: messages; Red's treasury; army; the rows' states: in order: none; the change: -2 gems, -2243 gold; treasury: 31 wood, 12 mercury, 25 ore, 15 sulfur, 15 crystal, 14 gems, 22844 gold; army: 0: 20 Halberdiers, 1: 10 Marksmen, 2: 3 Grand Elves, 3: 3 Silver Pegasi, 4: 1 Archangel, 5: 4 Storm Elementals; rows: 0: 20 Halberdiers: 0 (shown GENRLTXT:346), state 0, no upgrade; 1: 10 Marksmen: 0 (shown GENRLTXT:346), state 0, no upgrade; 2: 3 Grand Elves: 0 (shown GENRLTXT:346), state 0, no upgrade; 3: 3 Silver Pegasi: 0 (shown GENRLTXT:346), state 0, no upgrade; 4: 1 Archangel: 0 (shown GENRLTXT:346), state 0, no upgrade; 5: 4 Storm Elementals: 0 (shown GENRLTXT:346), state 0, no upgrade; the whole: 0, state 0, no upgrade
- after closing: tile; points; Space: what opens; the tile: tile: 24,18,0; points: 1360; Space: nothing; in order: Hill Fort's dialog (the whole 0, no upgrade); its rows: 0: 20 Halberdiers: 0 (shown GENRLTXT:346), state 0, no upgrade; 1: 10 Marksmen: 0 (shown GENRLTXT:346), state 0, no upgrade; 2: 3 Grand Elves: 0 (shown GENRLTXT:346), state 0, no upgrade; 3: 3 Silver Pegasi: 0 (shown GENRLTXT:346), state 0, no upgrade; 4: 1 Archangel: 0 (shown GENRLTXT:346), state 0, no upgrade; 5: 4 Storm Elementals: 0 (shown GENRLTXT:346), state 0, no upgrade; tile after: 24,18,0
- anything odd: 

## E14-15 — A War Machine Factory bought from by a hero with a Ballista, then visited again (E14-15); where the hero stands (E14-16) (ADV-WMF-001, GAM-DEC-008, ADV-HIR-002, ADV-OBJ-003)

- Red hero **E14-15** (Adela, specialty spell Bless): attack 1, defense 1, power 1, knowledge 1; no skills; Ballista (war machine 1); no spells; army 5 Pikeman

1. **E14-15.** Day 1, Red's turn, after E14-12: E14-15 stands at (29, 20) wearing a Ballista. Note Red's treasury. Click the factory F's entrance (29, 18) twice: E14-15 walks onto it. Note every question and E14-15's tile and points at it; answer **yes**.
2. In the dialog note each machine (in order) with its count and price. Buy every machine whose count is 1 (one each). Note Red's treasury and E14-15's war machine positions. Close the dialog.
3. **E14-16.** Note E14-15's tile. Press Space: note the question; answer yes; note each machine's count; close it. Note the tile. Then end Red's turn.

**Record**

- before: Red's treasury; at the question: E14-15's tile; points: treasury: 31 wood, 12 mercury, 25 ore, 15 sulfur, 15 crystal, 14 gems, 22844 gold; worn: 13: Ballista, 16: Catapult; at the question: ADVEVENT:157 (yes) on 29,18 with 1360; in order: question ADVEVENT:157 (yes); hire screen

| machine | count | price |
|---|---|---|
| Ballista | 0 | 2500 |
| First Aid Tent | 1 | 750 |
| Ammo Cart | 1 | 1000 |

- after the purchases: Red's treasury; the war machine positions: treasury: 31 wood, 12 mercury, 25 ore, 15 sulfur, 15 crystal, 14 gems, 21094 gold; worn: 13: Ballista, 14: Ammo Cart, 15: First Aid Tent, 16: Catapult; the dialog: most 0; amount 0; ally 0; selected 2 (counts 0, 0, 0); max disabled, recruit disabled, close enabled, slider disabled
- after closing: tile; Space: the question; the counts; the tile after: tile: 29,18,0; the change: -1750 gold; Space: Ballista; available: 0; First Aid Tent; available: 0; Ammo Cart; available: 0; in order: question ADVEVENT:157 (yes); hire screen; tile after: 29,18,0
- anything odd: 

## E14-13 — A Hill Fort for a hero too poor for its one upgradable stack (E14-13) (ADV-HFT-001, ADV-HFT-002)

- Blue hero **E14-13** (Mephala, specialty skill Armorer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Rogue, 5 Marksman, 3000 Pegasus

1. **E14-13.** Day 1, Blue's turn, after E14-5's Blue read: E14-13 stands at (24, 28) with 5 Rogues, 5 Marksmen and 3 000 Pegasi. Note Blue's treasury. Click the Hill Fort H2's entrance (24, 26) twice: E14-13 walks onto it and the dialog opens. Note E14-13's tile.
2. Note every row's price and state and the whole's state. Click the Pegasi's row button: note what happens. Click **All**: note what happens. Note Blue's treasury and E14-13's army. Close the dialog.

**Record**

- before: Blue's treasury; at the dialog: E14-13's tile: treasury: 30 wood, 15 mercury, 35 ore, 15 sulfur, 15 crystal, 15 gems, 30800 gold; at the dialog: 24,26 with 1500 points; in order: Hill Fort's dialog (the whole 2, not affordable)

| slot | stack | price | state |
|---|---|---|---|
| 0 Rogues | 5 Rogues | 0 (shown GENRLTXT:346) | 0, no upgrade |
| 1 Marksmen | 5 Marksmen | 0 (shown GENRLTXT:346) | 0, no upgrade |
| 2 Pegasi | 3000 Pegasi | 56250 gold | 2, not affordable |

- the whole's state: 56250 gold, state 2, not affordable; buttons: all enabled, row 0 enabled, row 1 enabled, row 2 enabled, row 3 disabled, row 4 disabled, row 5 disabled, row 6 disabled
- the Pegasi's button: what happens; All: what happens: the Pegasi: message GENRLTXT:315; the change: none; army: 0: 5 Rogues, 1: 5 Marksmen, 2: 3000 Pegasi; All: message GENRLTXT:317; the change: none; the rows: 0: 5 Rogues: 0 (shown GENRLTXT:346), state 0, no upgrade; 1: 5 Marksmen: 0 (shown GENRLTXT:346), state 0, no upgrade; 2: 3000 Pegasi: 56250 gold, state 2, not affordable
- after: Blue's treasury; army: treasury: 30 wood, 15 mercury, 35 ore, 15 sulfur, 15 crystal, 15 gems, 30800 gold; army: 0: 5 Rogues, 1: 5 Marksmen, 2: 3000 Pegasi
- anything odd: 

## E14-17 — A treasure sacrificed for a level: the level-up over the Altar, the Altar still open, the creatures' total with the new Learning (E14-17) (ADV-ALT-002, ADV-ALT-003, GAM-DEC-012, GAM-DEC-007, HER-EXP-001)

- Blue hero **E14-17** (Alkin, specialty creature Gorgon): attack 1, defense 1, power 1, knowledge 1; Expert Archery, Expert Armorer, Expert Offense, Expert Resistance, Expert Ballistics, Expert Artillery, Expert Tactics, Basic Learning; Mystic Orb of Mana (backpack); no spells; army 10 Gnoll, 5 Lizardman

1. **E14-17.** Day 1, Blue's turn, after E14-13: E14-17 stands at (12, 28), level 1 with eight secondary skills (seven at Expert, Basic Learning), a Mystic Orb of Mana in its backpack, 10 Gnolls (slot 0) and 5 Lizardmen (slot 1). Note its experience, level and skills. Click the Altar A4's entrance (12, 26) twice: E14-17 walks onto it and the dialog opens. Note the screen and whether the creatures' button is enabled.
2. Put the **Mystic Orb of Mana** on the altar. Note the total and the experience lacking. Click Sacrifice. Note every dialog in order and what is drawn around it (is the Altar's dialog still drawn under the level-up?); at the level-up note the offered skills and take the left one. After it note whether the Altar is still open, E14-17's experience, level and skills.
3. Open the **creatures'** screen. Set the Gnolls' (slot 0) amount to **10**. Note the total. Click Sacrifice. Note E14-17's experience, level and army. Click Done. Then end Blue's turn.

**Record**

- before: experience; level; skills; at the dialog: E14-17's tile; the first screen; the creatures' button: experience: 0; level: 1; skills: Archery expert, Armorer expert, Offense expert, Resistance expert, Ballistics expert, Artillery expert, Tactics expert, Learning basic; backpack: Mystic Orb of Mana; army: 0: 10 Gnolls, 1: 5 Lizardmen; at the dialog: 12,26 with 1360 points; the screen: artifacts; allowed: artifacts, creatures; the other: the creatures' button enabled; in order: Altar's dialog (the artifacts' screen; allowed artifacts, creatures)
- the Mystic Orb: the total; the experience lacking: on the altar: 0: Mystic Orb of Mana (from the backpack, 1050); the total: 1050; lacking: 1000
- after Sacrifice: the dialogs (in order) and what is drawn under them; the offered skills: in order: level-up (E14-17, level 2) over the altar dialog; the level-ups: level 2: Learning or none, took Learning
- after the level-up: the Altar still open; experience; level; skills: the screen: altar; the Altar: the artifacts' screen (allowed artifacts, creatures; the creatures' button enabled); total 0; lacking 950; on the altar none; held none; Sacrifice disabled; experience: 1050; level: 2; skills: Archery expert, Armorer expert, Offense expert, Resistance expert, Ballistics expert, Artillery expert, Tactics expert, Learning advanced
- 10 Gnolls: the total; after Sacrifice: experience; level; army: slot 0's most: 10; amounts: 0: 10; the total: 55; after Sacrifice: none; experience: 1105; level: 2; army: 1: 5 Lizardmen; the Altar: the creatures' screen (allowed artifacts, creatures; the artifacts' button enabled); total 0; lacking 895; amounts none; selected none (most none); Sacrifice disabled
- anything odd: the screen under the level-up (its dump, `level-up-E14-17-2.xwd`, looked at): the Altar's dialog around it, its places already empty (the orb gone) while its texts still read the total 1,050 and 1,000 lacking (the Altar redrawn after the level-up's OK).
