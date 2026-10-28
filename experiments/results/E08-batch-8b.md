# E08 batch E8b — results

Map file: `E08-batch-8b.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `18e9bac9`.** The map's description in the scenario list starts with "Stamp 18e9bac9.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-8b.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1), and at the start of each later turn (save 2 Blue's day 1, save 3 Red's day 2).
4. At the start (save 1) read every Flotsam's, Sea Chest's and Survivor's word (E8-6's and E8-8's): if none of E8-6's thirty chests holds an artifact (bits 0 to 2 equal 2), or none of E8-8's forty, restart the scenario (a new game draws again) and save again, at most 20 times. After a restart check that the words changed; if they did not, start the new game from the main menu instead.
5. Most records are the game's memory: an object's word on its tile, a hero's tile, movement points (+0x4D), day's maximum (+0x49), status flags (+0x105; bit 18, 0x40000, is the hero in a boat), its backpack's count (+0x3D4) and worn positions (+0x12D), the boats' records (game +0x4E3BC, 0x28 bytes each: tile, in use +0x18, type +0x1A, facing +0x1B, owner +0x1C, captain +0x20, aboard +0x24; `docs/exe/structs.toml` "boat"), the players' treasuries, a battle's terrain, special field, ship flag (`battle_manager` +0x53C6) and blocked hexes at its opening, the path marks (batch E1a's). The capture harness reads them (`clients/capture/batches/E08-batch-8b.toml`, E08 T68). Played by hand, read the screens instead and leave the memory-only lines empty.
6. Tiles are (column, row) from 0 at the top left. A walk: click the target once (the path's marks show; note them where asked), then again (the hero goes). The original draws no path onto a tile not yet explored (the scouting radius is 5): a target farther away is reached in legs of at most four tiles over the water toward it, each leg's target clicked twice, then the target itself. A hero out of movement points goes on the next day (Red's day 3). A boat stands on water beside the hero's tile; a click on it from the land beside it boards it. A pickup, a bottle or another hero in a boat is visited by a click on it: the hero sails to a water tile beside it and visits it from there.
7. Battles by auto combat unless the encounter says otherwise; a tactics phase is ended at once (its Start button).
8. Order: day 1, Red's turn: E8-6, E8-8, E8-9, E8-12, E8-13 and E8-14 board their boats (E8-11 does nothing); end the turn. Blue's turn: E8-11b and E8-12b board theirs; end the turn. Day 2, Red's turn: E8-6, E8-8, E8-9, E8-13, E8-14, then E8-11 and E8-12 (Blue loses two heroes there; E8-0b, Blue's spare at (66, 66), keeps Blue in the game). A row left unfinished goes on in Red's day 3. Then the map is done.
9. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E8-6 — A full backpack at sea: Sea Chests that hold an artifact, and Shipwreck Survivors (ADV-PIK-015, ADV-PIK-016, ADV-QUIRK-22)

- Red hero **E8-6** (Rion, specialty skill First Aid): attack 1, defense 1, power 1, knowledge 1; Expert Navigation; Sea Captain's Hat (head), Necklace of Ocean Guidance (neck), the backpack filled to 64 with Centaurs Axe; no spells; army 10 Pikeman

1. Day 1, Red's turn: E8-6 stands at (19, 10) with 64 Centaurs Axes in its backpack, the Necklace of Ocean Guidance and the Sea Captain's Hat worn and Expert Navigation; its boat at (20, 10). Thirty Sea Chests stand on the water at (24, 4) to (34, 12), every second column and row, and three Shipwreck Survivors at (24, 16), (27, 16) and (30, 16). Click the boat: E8-6 boards it.
2. Day 2, Red's turn: visit **every chest whose word holds an artifact** (bits 0 to 2 equal 2), then one chest of kind 1 and one of kind 0 (the nearest), then the three Survivors, in legs over the water where a target is farther than four tiles. At each: read E8-6's backpack count and Red's gold; click the object (twice); note the message (first words, the pictures shown); read the backpack count, Red's gold and the object's word (or that it is gone) again.

**Record**

| object (tile) | its word (hex) | the message (first words; pictures) | backpack before / after | gold before / after | the object after |
|---|---|---|---|---|---|
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |

- E8-6's movement points at day 2's start: 
- anything odd: 

## E8-8 — The sea pickups' draws at a new game: forty Flotsams, forty Sea Chests, forty Survivors; one of each visited (ADV-PIK-014, ADV-PIK-015, ADV-PIK-016)

- Red hero **E8-8** (Sanya, specialty skill Eagle Eye): attack 1, defense 1, power 1, knowledge 1; Expert Navigation; Sea Captain's Hat (head), Necklace of Ocean Guidance (neck); no spells; army 10 Pikeman

1. At the start (save 1), before anything is played: forty Flotsams stand at (24, 24) to (38, 32), forty Sea Chests at (44, 24) to (58, 32) and forty Shipwreck Survivors at (24, 40) to (38, 48), every second column and row. Read each one's word on its tile, row by row.
2. Day 1, Red's turn: E8-8 stands on a one-tile island at (41, 36), between the three groups, with Expert Navigation, the Necklace of Ocean Guidance and the Sea Captain's Hat; its boat at (42, 36). Click the boat: E8-8 boards it.
3. Day 2, Red's turn: visit the nearest Flotsam whose word is 3, then the nearest Survivor, then the nearest Sea Chest of kind 2, in legs over the water where a target is farther than four tiles. At each: read Red's wood and gold, E8-8's backpack count and worn positions; click the object (twice); note the message (first words; the artifact named); read them again.

**Record**

| Flotsam (tile) | its word |
|---|---|
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |

| Sea Chest (tile) | its word (hex) |
|---|---|
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |

| Survivor (tile) | its word (the artifact) |
|---|---|
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |
|   |   |

| object visited (tile) | its word | the message (first words) | wood, gold before / after | backpack count before / after; the artifact |
|---|---|---|---|---|
|   |   |   |   |   |
|   |   |   |   |   |
|   |   |   |   |   |

- anything odd: 

## E8-9 — Four Ocean Bottles: three without a text at columns 24, 25 and 49, one with the map's text (ADV-PIK-017)

- Red hero **E8-9** (Ivor, specialty creature Wood Elf): attack 1, defense 1, power 1, knowledge 1; Expert Navigation; Necklace of Ocean Guidance (neck); no spells; army 10 Pikeman

1. Day 1, Red's turn: E8-9 stands at (19, 57) with Expert Navigation and the Necklace of Ocean Guidance; its boat at (20, 57). Four Ocean Bottles float on row 55: at (24, 55), (25, 55) and (49, 55) without a text, at (34, 55) with one. Click the boat: E8-9 boards it.
2. Day 2, Red's turn: visit the bottles in the order (24, 55), (25, 55), (34, 55), (49, 55), sailing along row 56 in legs: (24, 55) and (25, 55) from the start; then (28, 56), (32, 56) and the bottle (34, 55); then (38, 56), (42, 56), (46, 56) and the bottle (49, 55). At each bottle: note the message's whole text; read Red's treasury and whether the bottle is gone.

**Record**

| bottle (tile) | the message's text | the treasury changed (yes/no) | the bottle after |
|---|---|---|---|
| (24, 55) |   |   |   |
| (25, 55) |   |   |   |
| (34, 55) |   |   |   |
| (49, 55) |   |   |   |

- anything odd: 

## E8-11 — Red's hero on the shore attacks Blue's hero in a boat and wins (GAM-BAT-001, GAM-BAT-003, ADV-SEA-005, ADV-SEA-006)

- Red hero **E8-11** (Caitlin, specialty resource gold): attack 5, defense 5, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel
- Blue hero **E8-11b** (Mephala, specialty skill Armorer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Blue hero **E8-0b** (Clancy, specialty creature Unicorn): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. Day 1, Blue's turn: E8-11b stands at (60, 4) with one Pikeman; its boat at (61, 4). Click the boat: E8-11b boards it.
2. Day 2, Red's turn: E8-11 stands at (61, 5), south of the boat, with ten Archangels. Read the boat's record. Click E8-11b once (note the marks), then again: the battle. Note its opening: terrain, special field, ship flag, blocked hexes. Auto combat.
3. After the battle: read E8-11's tile and status flags, the boat's record and whether anything stands on (61, 4).

**Record**

- the boat's record before (tile, in use, owner, captain, aboard): 
- the marks of the click on E8-11b: 
- the opening: terrain, special field, ship flag, blocked hexes: 
- after: E8-11's tile and status flags; the boat's record; what stands on (61, 4): 
- anything odd: 

## E8-12 — Two heroes in boats side by side: Red attacks, Blue retreats (GAM-BAT-001, GAM-BAT-003, ADV-SEA-005, ADV-SEA-006)

- Red hero **E8-12** (Ufretin, specialty creature Dwarf): attack 5, defense 5, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel
- Blue hero **E8-12b** (Jenova, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn: E8-12 stands at (60, 10) with ten Archangels; its boat at (61, 10). Click the boat: E8-12 boards it.
2. Day 1, Blue's turn: E8-12b stands at (62, 11) with one Pikeman; its boat at (62, 10), beside Red's. Click the boat: E8-12b boards it.
3. Day 2, Red's turn: click E8-12b once (note the marks), then again: the battle. Note its opening: terrain, special field, ship flag, blocked hexes. Fight by hand: each Archangel stack **defends** until Blue's Pikeman acts; then Blue **retreats** (the Retreat button, confirmed).
4. After the battle: read E8-12's tile and status flags, both boats' records and whether anything stands on (62, 10).

**Record**

- the marks of the click on E8-12b: 
- the opening: terrain, special field, ship flag, blocked hexes (all of them): 
- after: E8-12's tile and status flags: 
- after: Red's boat's record; Blue's boat's record; what stands on (62, 10): 
- anything odd: 

## E8-13 — A boat and Pikemen on the water: a click past their guarded tiles, then onto one (GAM-BAT-001, ADV-SEA-005, ADV-MON-001)

- Red hero **E8-13** (Ryland, specialty creature Dendroid Guard): attack 5, defense 5, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Archangel
- Opponent: 10 Pikeman (wandering monster, never flees)

1. Day 1, Red's turn: E8-13 stands at (4, 63) with five Archangels; its boat at (5, 63), at the west end of a channel three tiles wide (rows 62 to 64). Ten savage Pikemen stand on the water at (8, 63): they guard columns 7 to 9 of the channel. Click the boat: E8-13 boards it.
2. Day 2, Red's turn: click (10, 63), past the guards, once; note the marks (or none); click it again. Read E8-13's tile.
3. Click (7, 63), a guarded tile, once (note the marks), then again. Note where the battle starts and its opening: terrain, special field, ship flag, blocked hexes. Auto combat.
4. After the battle: read E8-13's tile and status flags.

**Record**

- the click on (10, 63): the marks (or none); E8-13's tile after: 
- the click on (7, 63): the marks; the tile where the battle starts: 
- the opening: terrain, special field, ship flag, blocked hexes: 
- after: E8-13's tile and status flags: 
- anything odd: 

## E8-14 — Two of Red's events on a channel: 100 gold, then guards (ADV-OBJ-007, ADV-MOV-011, ADV-SEA-005, GAM-BAT-001)

- Red hero **E8-14** (Thorgrim, specialty skill Resistance): attack 5, defense 5, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Archangel

1. Day 1, Red's turn: E8-14 stands at (4, 68) with five Archangels; its boat at (5, 68), at the west end of a channel one tile wide (row 68). Hidden events of Red's lie on the water at (8, 68) (100 gold) and (11, 68) (five Pikemen, no text). Click the boat: E8-14 boards it.
2. Day 2, Red's turn: read Red's gold. Click (10, 68) once (note the marks), then again. Read E8-14's tile and movement points and Red's gold; note any message.
3. Click (13, 68) once (note the marks), then again. Note the message and where the battle starts, its opening: terrain, special field, ship flag, blocked hexes. Auto combat.
4. After the battle: read E8-14's tile and status flags.

**Record**

- gold at day 2's start: 
- the click on (10, 68): the marks; tile, movement points and gold after; the message: 
- the click on (13, 68): the marks; the message; the tile where the battle starts: 
- the opening: terrain, special field, ship flag, blocked hexes: 
- after: E8-14's tile and status flags: 
- anything odd: 
