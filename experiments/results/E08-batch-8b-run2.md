# E08 batch E8b — results

> Played by the capture harness on 2026-10-29 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-8b.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus random, Show Route on; read from the game's memory.

> The second recorded run (E08 T68b), a new game right after the first (`E08-batch-8b.md`): its draws add to the first's for the scoring's counts; every row's outcome reads as the first's; the tiles visited, the gold and the artifacts follow the draws.

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
| (28, 6); Sea Chest: artifact Ring of Conjuring, 1000 gold | 0xffffe26a | ADVEVENT:118; ADVEVENT:118: 1500 gold | 64; 64 | 30350; 31850 | none |
| (34, 12); Sea Chest: artifact Necklace of Swiftness, 1000 gold | 0xffffe30a | ADVEVENT:118; ADVEVENT:118: 1500 gold | 64; 64 | 31850; 33350 | none |
| (32, 10); Sea Chest: 1500 gold | 0xfffffff9 | ADVEVENT:118; ADVEVENT:118: 1500 gold | 64; 64 | 33350; 34850 | none |
| (30, 8); Sea Chest: nothing | 0xfffffff8 | ADVEVENT:116; none | 64; 64 | 34850; 34850 | none |
| (24, 16); Survivor: artifact Targ of the Rampaging Ogre | 0x00000010 | ADVEVENT:126; none | 64; 64 | 34850; 34850 | none |
| (27, 16); Survivor: artifact Sword of Hellfire | 0x0000000b | ADVEVENT:126; none | 64; 64 | 34850; 34850 | none |
| (30, 16); Survivor: artifact Eversmoking Ring of Sulfur | 0x00000071 | ADVEVENT:126; none | 64; 64 | 34850; 34850 | none |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |
|   |   |   |   |   |   |

- E8-6's movement points at day 2's start: 5250
- anything odd: (24, 4) at the start: 0xfffffff9; Sea Chest: 1500 gold; (26, 4) at the start: 0xfffffff8; Sea Chest: nothing; (28, 4) at the start: 0xfffffff9; Sea Chest: 1500 gold; (30, 4) at the start: 0xfffffff9; Sea Chest: 1500 gold; (32, 4) at the start: 0xfffffff9; Sea Chest: 1500 gold; (34, 4) at the start: 0xfffffff9; Sea Chest: 1500 gold; (24, 6) at the start: 0xfffffff8; Sea Chest: nothing; (26, 6) at the start: 0xfffffff9; Sea Chest: 1500 gold; (28, 6) at the start: 0xffffe26a; Sea Chest: artifact Ring of Conjuring, 1000 gold; (30, 6) at the start: 0xfffffff9; Sea Chest: 1500 gold; (32, 6) at the start: 0xfffffff9; Sea Chest: 1500 gold; (34, 6) at the start: 0xfffffff9; Sea Chest: 1500 gold; (24, 8) at the start: 0xfffffff9; Sea Chest: 1500 gold; (26, 8) at the start: 0xfffffff9; Sea Chest: 1500 gold; (28, 8) at the start: 0xfffffff9; Sea Chest: 1500 gold; (30, 8) at the start: 0xfffffff8; Sea Chest: nothing; (32, 8) at the start: 0xfffffff8; Sea Chest: nothing; (34, 8) at the start: 0xfffffff8; Sea Chest: nothing; (24, 10) at the start: 0xfffffff9; Sea Chest: 1500 gold; (26, 10) at the start: 0xfffffff9; Sea Chest: 1500 gold; (28, 10) at the start: 0xfffffff9; Sea Chest: 1500 gold; (30, 10) at the start: 0xfffffff9; Sea Chest: 1500 gold; (32, 10) at the start: 0xfffffff9; Sea Chest: 1500 gold; (34, 10) at the start: 0xfffffff8; Sea Chest: nothing; (24, 12) at the start: 0xfffffff9; Sea Chest: 1500 gold; (26, 12) at the start: 0xfffffff9; Sea Chest: 1500 gold; (28, 12) at the start: 0xfffffff8; Sea Chest: nothing; (30, 12) at the start: 0xfffffff9; Sea Chest: 1500 gold; (32, 12) at the start: 0xfffffff8; Sea Chest: nothing; (34, 12) at the start: 0xffffe30a; Sea Chest: artifact Necklace of Swiftness, 1000 gold; (24, 16) at the start: 0x00000010; Survivor: artifact Targ of the Rampaging Ogre; (27, 16) at the start: 0x0000000b; Survivor: artifact Sword of Hellfire; (30, 16) at the start: 0x00000071; Survivor: artifact Eversmoking Ring of Sulfur; worn at the start (the starting bonus): Sea Captain's Hat (head), Necklace of Ocean Guidance (neck); backpack count at the start: 64; E8-6's boarding (arrived, tile, points, status, boat): yes; 20,10,0; 0; 0x40000; tile 20,10,0; in use 1; type 0; facing 2; owner red; captain E8-6; aboard 1; E8-6 at day 2's start (day's maximum, status): 5250; 0x40000; E8-6 after its visits (tile, points, backpack): 25,15,0; 1712; 64; E8-6 after its visits (tile, points, backpack): 26,15,0; 1471; 64; E8-6 after its visits (tile, points, backpack): 29,15,0; 1030; 64

## E8-8 — The sea pickups' draws at a new game: forty Flotsams, forty Sea Chests, forty Survivors; one of each visited (ADV-PIK-014, ADV-PIK-015, ADV-PIK-016)

- Red hero **E8-8** (Sanya, specialty skill Eagle Eye): attack 1, defense 1, power 1, knowledge 1; Expert Navigation; Sea Captain's Hat (head), Necklace of Ocean Guidance (neck); no spells; army 10 Pikeman

1. At the start (save 1), before anything is played: forty Flotsams stand at (24, 24) to (38, 32), forty Sea Chests at (44, 24) to (58, 32) and forty Shipwreck Survivors at (24, 40) to (38, 48), every second column and row. Read each one's word on its tile, row by row.
2. Day 1, Red's turn: E8-8 stands on a one-tile island at (41, 36), between the three groups, with Expert Navigation, the Necklace of Ocean Guidance and the Sea Captain's Hat; its boat at (42, 36). Click the boat: E8-8 boards it.
3. Day 2, Red's turn: visit the nearest Flotsam whose word is 3, then the nearest Survivor, then the nearest Sea Chest of kind 2, in legs over the water where a target is farther than four tiles. At each: read Red's wood and gold, E8-8's backpack count and worn positions; click the object (twice); note the message (first words; the artifact named); read them again.

**Record**

| Flotsam (tile) | its word |
|---|---|
| (24, 24) | 0x00000000; Flotsam: nothing |
| (26, 24) | 0x00000003; Flotsam: 10 wood, 500 gold |
| (28, 24) | 0x00000000; Flotsam: nothing |
| (30, 24) | 0x00000001; Flotsam: 5 wood |
| (32, 24) | 0x00000001; Flotsam: 5 wood |
| (34, 24) | 0x00000000; Flotsam: nothing |
| (36, 24) | 0x00000001; Flotsam: 5 wood |
| (38, 24) | 0x00000003; Flotsam: 10 wood, 500 gold |
| (24, 26) | 0x00000003; Flotsam: 10 wood, 500 gold |
| (26, 26) | 0x00000000; Flotsam: nothing |
| (28, 26) | 0x00000000; Flotsam: nothing |
| (30, 26) | 0x00000000; Flotsam: nothing |
| (32, 26) | 0x00000003; Flotsam: 10 wood, 500 gold |
| (34, 26) | 0x00000002; Flotsam: 5 wood, 200 gold |
| (36, 26) | 0x00000003; Flotsam: 10 wood, 500 gold |
| (38, 26) | 0x00000001; Flotsam: 5 wood |
| (24, 28) | 0x00000001; Flotsam: 5 wood |
| (26, 28) | 0x00000003; Flotsam: 10 wood, 500 gold |
| (28, 28) | 0x00000003; Flotsam: 10 wood, 500 gold |
| (30, 28) | 0x00000003; Flotsam: 10 wood, 500 gold |
| (32, 28) | 0x00000001; Flotsam: 5 wood |
| (34, 28) | 0x00000003; Flotsam: 10 wood, 500 gold |
| (36, 28) | 0x00000003; Flotsam: 10 wood, 500 gold |
| (38, 28) | 0x00000002; Flotsam: 5 wood, 200 gold |
| (24, 30) | 0x00000002; Flotsam: 5 wood, 200 gold |
| (26, 30) | 0x00000000; Flotsam: nothing |
| (28, 30) | 0x00000002; Flotsam: 5 wood, 200 gold |
| (30, 30) | 0x00000000; Flotsam: nothing |
| (32, 30) | 0x00000001; Flotsam: 5 wood |
| (34, 30) | 0x00000002; Flotsam: 5 wood, 200 gold |
| (36, 30) | 0x00000003; Flotsam: 10 wood, 500 gold |
| (38, 30) | 0x00000001; Flotsam: 5 wood |
| (24, 32) | 0x00000000; Flotsam: nothing |
| (26, 32) | 0x00000003; Flotsam: 10 wood, 500 gold |
| (28, 32) | 0x00000001; Flotsam: 5 wood |
| (30, 32) | 0x00000001; Flotsam: 5 wood |
| (32, 32) | 0x00000003; Flotsam: 10 wood, 500 gold |
| (34, 32) | 0x00000002; Flotsam: 5 wood, 200 gold |
| (36, 32) | 0x00000000; Flotsam: nothing |
| (38, 32) | 0x00000000; Flotsam: nothing |

| Sea Chest (tile) | its word (hex) |
|---|---|
| (44, 24) | 0xffffe34a; Sea Chest: artifact Pendant of Free Will, 1000 gold |
| (46, 24) | 0xfffffff8; Sea Chest: nothing |
| (48, 24) | 0xfffffff9; Sea Chest: 1500 gold |
| (50, 24) | 0xfffffff9; Sea Chest: 1500 gold |
| (52, 24) | 0xfffffff8; Sea Chest: nothing |
| (54, 24) | 0xfffffff9; Sea Chest: 1500 gold |
| (56, 24) | 0xfffffff9; Sea Chest: 1500 gold |
| (58, 24) | 0xfffffff9; Sea Chest: 1500 gold |
| (44, 26) | 0xffffe12a; Sea Chest: artifact Quiet Eye of the Dragon, 1000 gold |
| (46, 26) | 0xfffffff9; Sea Chest: 1500 gold |
| (48, 26) | 0xfffffff9; Sea Chest: 1500 gold |
| (50, 26) | 0xfffffff9; Sea Chest: 1500 gold |
| (52, 26) | 0xfffffff8; Sea Chest: nothing |
| (54, 26) | 0xfffffff9; Sea Chest: 1500 gold |
| (56, 26) | 0xffffe25a; Sea Chest: artifact Mystic Orb of Mana, 1000 gold |
| (58, 26) | 0xfffffff8; Sea Chest: nothing |
| (44, 28) | 0xfffffff8; Sea Chest: nothing |
| (46, 28) | 0xfffffff9; Sea Chest: 1500 gold |
| (48, 28) | 0xfffffff9; Sea Chest: 1500 gold |
| (50, 28) | 0xfffffff8; Sea Chest: nothing |
| (52, 28) | 0xfffffff8; Sea Chest: nothing |
| (54, 28) | 0xfffffff9; Sea Chest: 1500 gold |
| (56, 28) | 0xfffffff8; Sea Chest: nothing |
| (58, 28) | 0xfffffff9; Sea Chest: 1500 gold |
| (44, 30) | 0xffffe17a; Sea Chest: artifact Cards of Prophecy, 1000 gold |
| (46, 30) | 0xfffffff9; Sea Chest: 1500 gold |
| (48, 30) | 0xfffffff9; Sea Chest: 1500 gold |
| (50, 30) | 0xffffe192; Sea Chest: artifact Crest of Valor, 1000 gold |
| (52, 30) | 0xfffffff9; Sea Chest: 1500 gold |
| (54, 30) | 0xfffffff9; Sea Chest: 1500 gold |
| (56, 30) | 0xfffffff9; Sea Chest: 1500 gold |
| (58, 30) | 0xfffffff9; Sea Chest: 1500 gold |
| (44, 32) | 0xfffffff8; Sea Chest: nothing |
| (46, 32) | 0xfffffff9; Sea Chest: 1500 gold |
| (48, 32) | 0xfffffff9; Sea Chest: 1500 gold |
| (50, 32) | 0xfffffff9; Sea Chest: 1500 gold |
| (52, 32) | 0xfffffff9; Sea Chest: 1500 gold |
| (54, 32) | 0xfffffff9; Sea Chest: 1500 gold |
| (56, 32) | 0xfffffff8; Sea Chest: nothing |
| (58, 32) | 0xfffffff9; Sea Chest: 1500 gold |

| Survivor (tile) | its word (the artifact) |
|---|---|
| (24, 40) | 0x00000019; Survivor: artifact Breastplate of Petrified Wood |
| (26, 40) | 0x00000064; Survivor: artifact Pendant of Dispassion |
| (28, 40) | 0x0000006a; Survivor: artifact Pendant of Negativity |
| (30, 40) | 0x0000006f; Survivor: artifact Everpouring Vial of Mercury |
| (32, 40) | 0x0000005c; Survivor: artifact Sphere of Permanence |
| (34, 40) | 0x0000006c; Survivor: artifact Pendant of Courage |
| (36, 40) | 0x0000003e; Survivor: artifact Angel Feather Arrows |
| (38, 40) | 0x00000043; Survivor: artifact Diplomat's Ring |
| (24, 42) | 0x00000022; Survivor: artifact Lion's Shield of Courage |
| (26, 42) | 0x00000015; Survivor: artifact Helm of Chaos |
| (28, 42) | 0x00000053; Survivor: artifact Recanter's Cloak |
| (30, 42) | 0x00000009; Survivor: artifact Greater Gnoll's Flail |
| (32, 42) | 0x0000003a; Survivor: artifact Surcoat of Counterpoise |
| (34, 42) | 0x00000062; Survivor: artifact Boots of Speed |
| (36, 42) | 0x0000000d; Survivor: artifact Shield of the Dwarven Lords |
| (38, 42) | 0x00000033; Survivor: artifact Glyph of Gallantry |
| (24, 44) | 0x0000002d; Survivor: artifact Still Eye of the Dragon |
| (26, 44) | 0x00000008; Survivor: artifact Blackshard of the Dead Knight |
| (28, 44) | 0x00000026; Survivor: artifact Red Dragon Flame Tongue |
| (30, 44) | 0x0000004e; Survivor: artifact Cape of Conjuring |
| (32, 44) | 0x00000078; Survivor: artifact Torso of Legion |
| (34, 44) | 0x00000007; Survivor: artifact Centaurs Axe |
| (36, 44) | 0x0000001b; Survivor: artifact Scales of the Greater Basilisk |
| (38, 44) | 0x00000077; Survivor: artifact Loins of Legion |
| (24, 46) | 0x00000075; Survivor: artifact Endless Purse of Gold |
| (26, 46) | 0x00000068; Survivor: artifact Pendant of Death |
| (28, 46) | 0x00000044; Survivor: artifact Ambassador's Sash |
| (30, 46) | 0x0000001d; Survivor: artifact Breastplate of Brimstone |
| (32, 46) | 0x0000005f; Survivor: artifact Ring of Life |
| (34, 46) | 0x00000031; Survivor: artifact Badge of Courage |
| (36, 46) | 0x0000001a; Survivor: artifact Rib Cage |
| (38, 46) | 0x00000013; Survivor: artifact Helm of the Alabaster Unicorn |
| (24, 48) | 0x00000016; Survivor: artifact Crown of the Supreme Magi |
| (26, 48) | 0x00000040; Survivor: artifact Stoic Watchman |
| (28, 48) | 0x00000035; Survivor: artifact Spyglass |
| (30, 48) | 0x0000003c; Survivor: artifact Bow of Elven Cherrywood |
| (32, 48) | 0x0000007d; Survivor: artifact Shackles of War |
| (34, 48) | 0x0000001f; Survivor: artifact Armor of Wonder |
| (36, 48) | 0x0000000a; Survivor: artifact Ogre's Club of Havoc |
| (38, 48) | 0x0000006d; Survivor: artifact Everflowing Crystal Cloak |

| object visited (tile) | its word | the message (first words) | wood, gold before / after | backpack count before / after; the artifact |
|---|---|---|---|---|
| Flotsam (36, 30); Flotsam: 10 wood, 500 gold | 0x00000003 | ADVEVENT:53; ADVEVENT:53: 500 gold, 10 wood | 35; 34850; 45; 35350 | 0; 0; 0: Sea Captain's Hat, 2: Necklace of Ocean Guidance, 16: Catapult; none |
| Survivor (28, 40); Survivor: artifact Pendant of Negativity | 0x0000006a | ADVEVENT:125; ADVEVENT:125: Pendant of Negativity | 45; 35350; 45; 35350 | 0; 1; 0: Sea Captain's Hat, 2: Necklace of Ocean Guidance, 16: Catapult; Pendant of Negativity |
| Sea Chest (44, 24); Sea Chest: artifact Pendant of Free Will, 1000 gold | 0xffffe34a | ADVEVENT:117; ADVEVENT:117: 1000 (resource.def 6), Pendant of Free Will | 45; 35350; 45; 36350 | 1; 2; 0: Sea Captain's Hat, 2: Necklace of Ocean Guidance, 16: Catapult; Pendant of Negativity, Pendant of Free Will |

- anything odd: worn at the start (the starting bonus): Sea Captain's Hat (head), Necklace of Ocean Guidance (neck); E8-8's boarding (arrived, tile, points, status, boat): yes; 42,36,0; 0; 0x40000; tile 42,36,0; in use 1; type 1; facing 2; owner red; captain E8-8; aboard 1; E8-8 at day 2's start (points, day's maximum): 5250; 5250; Flotsam (36, 30) after the visit: its object; none; Survivor (28, 40) after the visit: its object; none; Sea Chest (44, 24) after the visit: its object; none

## E8-9 — Four Ocean Bottles: three without a text at columns 24, 25 and 49, one with the map's text (ADV-PIK-017)

- Red hero **E8-9** (Ivor, specialty creature Wood Elf): attack 1, defense 1, power 1, knowledge 1; Expert Navigation; Necklace of Ocean Guidance (neck); no spells; army 10 Pikeman

1. Day 1, Red's turn: E8-9 stands at (19, 57) with Expert Navigation and the Necklace of Ocean Guidance; its boat at (20, 57). Four Ocean Bottles float on row 55: at (24, 55), (25, 55) and (49, 55) without a text, at (34, 55) with one. Click the boat: E8-9 boards it.
2. Day 2, Red's turn: visit the bottles in the order (24, 55), (25, 55), (34, 55), (49, 55), sailing along row 56 in legs: (24, 55) and (25, 55) from the start; then (28, 56), (32, 56) and the bottle (34, 55); then (38, 56), (42, 56), (46, 56) and the bottle (49, 55). At each bottle: note the message's whole text; read Red's treasury and whether the bottle is gone.

**Record**

| bottle (tile) | the message's text | the treasury changed (yes/no) | the bottle after |
|---|---|---|---|
| (24, 55) | RANDSIGN:24 | no | none |
| (25, 55) | RANDSIGN:0 | no | none |
| (34, 55) | the map's text "E8-9: the map's own text." | no | none |
| (49, 55) | RANDSIGN:24 | no | none |

- anything odd: worn at the start (the starting bonus): Necklace of Ocean Guidance (neck); E8-9's boarding (arrived, tile, points, status, boat): yes; 20,57,0; 0; 0x40000; tile 20,57,0; in use 1; type 2; facing 2; owner red; captain E8-9; aboard 1; E8-9 at day 2's start (points, day's maximum): 4750; 4750; (24, 55) before (its record; the hero's tile after): Ocean Bottle: no text; 23,56,0; (25, 55) before (its record; the hero's tile after): Ocean Bottle: no text; 24,56,0; (34, 55) before (its record; the hero's tile after): Ocean Bottle: the map's text "E8-9: the map's own text."; 33,56,0; (49, 55) before (its record; the hero's tile after, points): Ocean Bottle: no text; 48,56,0; 1345

## E8-11 — Red's hero on the shore attacks Blue's hero in a boat and wins (GAM-BAT-001, GAM-BAT-003, ADV-SEA-005, ADV-SEA-006)

- Red hero **E8-11** (Caitlin, specialty resource gold): attack 5, defense 5, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel
- Blue hero **E8-11b** (Mephala, specialty skill Armorer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Blue hero **E8-0b** (Clancy, specialty creature Unicorn): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. Day 1, Blue's turn: E8-11b stands at (60, 4) with one Pikeman; its boat at (61, 4). Click the boat: E8-11b boards it.
2. Day 2, Red's turn: E8-11 stands at (61, 5), south of the boat, with ten Archangels. Read the boat's record. Click E8-11b once (note the marks), then again: the battle. Note its opening: terrain, special field, ship flag, blocked hexes. Auto combat.
3. After the battle: read E8-11's tile and status flags, the boat's record and whether anything stands on (61, 4).

**Record**

- the boat's record before (tile, in use, owner, captain, aboard): tile 61,4,0; in use 1; type 0; facing 2; owner blue; captain E8-11b; aboard 1
- the marks of the click on E8-11b: 1; green; 61,4
- the opening: terrain, special field, ship flag, blocked hexes: terrain water, special field none, ship flag 1, blocked hexes 98, 114, 115
- after: E8-11's tile and status flags; the boat's record; what stands on (61, 4): 61,5,0; 0x0; tile 61,4,0; in use 0; type 0; facing 2; owner blue; captain E8-11b; aboard 1; none (class 3: the coast)
- anything odd: E8-11b's boarding (arrived, tile, status, boat): yes; 61,4,0; 0x40000; tile 61,4,0; in use 1; type 0; facing 2; owner blue; captain E8-11b; aboard 1; E8-11b before (tile, status): 61,4,0; 0x40000; boats in use before: 8; 1 dialog(s) after the walk to [61, 4]; the battle (winner; stacks at the start): attacker; attacker 10 Archangels | defender 1 Pikeman; boats in use after: 7; messages: GENRLTXT:31

## E8-12 — Two heroes in boats side by side: Red attacks, Blue retreats (GAM-BAT-001, GAM-BAT-003, ADV-SEA-005, ADV-SEA-006)

- Red hero **E8-12** (Ufretin, specialty creature Dwarf): attack 5, defense 5, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel
- Blue hero **E8-12b** (Jenova, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn: E8-12 stands at (60, 10) with ten Archangels; its boat at (61, 10). Click the boat: E8-12 boards it.
2. Day 1, Blue's turn: E8-12b stands at (62, 11) with one Pikeman; its boat at (62, 10), beside Red's. Click the boat: E8-12b boards it.
3. Day 2, Red's turn: click E8-12b once (note the marks), then again: the battle. Note its opening: terrain, special field, ship flag, blocked hexes. Fight by hand: each Archangel stack **defends** until Blue's Pikeman acts; then Blue **retreats** (the Retreat button, confirmed).
4. After the battle: read E8-12's tile and status flags, both boats' records and whether anything stands on (62, 10).

**Record**

- the marks of the click on E8-12b: 1; green; 62,10
- the opening: terrain, special field, ship flag, blocked hexes (all of them): terrain water, special field none, ship flag 1, blocked hexes 6, 7, 8, 9, 24, 25, 26, 58, 59, 60, 75, 76, 77, 92, 93, 94, 109, 110, 111, 126, 127, 128, 159, 160, 161, 162, 163, 176, 177, 178, 179, 180
- after: E8-12's tile and status flags: 61,10,0; 0x40000; in a boat (type 1, flag 0)
- after: Red's boat's record; Blue's boat's record; what stands on (62, 10): tile 61,10,0; in use 1; type 1; facing 2; owner red; captain E8-12; aboard 1; tile 62,10,0; in use 0; type 2; facing 2; owner blue; captain E8-12b; aboard 1; none
- anything odd: E8-12's boarding (arrived, tile, points, status, boat): yes; 61,10,0; 0; 0x40000; tile 61,10,0; in use 1; type 1; facing 2; owner red; captain E8-12; aboard 1; E8-12b's boarding (arrived, tile, status, boat): yes; 62,10,0; 0x40000; tile 62,10,0; in use 1; type 2; facing 2; owner blue; captain E8-12b; aboard 1; the boats' records before (Red's, Blue's): tile 61,10,0; in use 1; type 1; facing 2; owner red; captain E8-12; aboard 1; tile 62,10,0; in use 1; type 2; facing 2; owner blue; captain E8-12b; aboard 1; the battle (winner, retreated; stacks at the start, at the end): attacker; defender; attacker 10 Archangels | defender 1 Pikeman; attacker 10 Archangels | defender 1 Pikeman; boats in use after: 6

## E8-13 — A boat and Pikemen on the water: a click past their guarded tiles, then onto one (GAM-BAT-001, ADV-SEA-005, ADV-MON-001)

- Red hero **E8-13** (Ryland, specialty creature Dendroid Guard): attack 5, defense 5, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Archangel
- Opponent: 10 Pikeman (wandering monster, never flees)

1. Day 1, Red's turn: E8-13 stands at (4, 63) with five Archangels; its boat at (5, 63), at the west end of a channel three tiles wide (rows 62 to 64). Ten savage Pikemen stand on the water at (8, 63): they guard columns 7 to 9 of the channel. Click the boat: E8-13 boards it.
2. Day 2, Red's turn: click (10, 63), past the guards, once; note the marks (or none); click it again. Read E8-13's tile.
3. Click (7, 63), a guarded tile, once (note the marks), then again. Note where the battle starts and its opening: terrain, special field, ship flag, blocked hexes. Auto combat.
4. After the battle: read E8-13's tile and status flags.

**Record**

- the click on (10, 63): the marks (or none); E8-13's tile after: 0; none; 5,63,0
- the click on (7, 63): the marks; the tile where the battle starts: 2; 6,63 7,63; 7,63,0
- the opening: terrain, special field, ship flag, blocked hexes: terrain water, special field none, ship flag 1, blocked hexes 79, 96, 97
- after: E8-13's tile and status flags: 7,63,0; 0x40000; in a boat (type 0, flag 0); tile 5,63,0; in use 1; type 0; facing 2; owner red; captain E8-13; aboard 1
- anything odd: E8-13's boarding (arrived, tile, points, status, boat): yes; 5,63,0; 0; 0x40000; tile 5,63,0; in use 1; type 0; facing 2; owner red; captain E8-13; aboard 1; (10, 63) explored before the click; E8-13's points before: yes; 1500; the guards before (8, 63): 10 Pikemen; the battle (winner; stacks at the start): attacker; attacker 5 Archangels | defender 4 Pikemen, 3 Halberdiers, 3 Pikemen

## E8-14 — Two of Red's events on a channel: 100 gold, then guards (ADV-OBJ-007, ADV-MOV-011, ADV-SEA-005, GAM-BAT-001)

- Red hero **E8-14** (Thorgrim, specialty skill Resistance): attack 5, defense 5, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Archangel

1. Day 1, Red's turn: E8-14 stands at (4, 68) with five Archangels; its boat at (5, 68), at the west end of a channel one tile wide (row 68). Hidden events of Red's lie on the water at (8, 68) (100 gold) and (11, 68) (five Pikemen, no text). Click the boat: E8-14 boards it.
2. Day 2, Red's turn: read Red's gold. Click (10, 68) once (note the marks), then again. Read E8-14's tile and movement points and Red's gold; note any message.
3. Click (13, 68) once (note the marks), then again. Note the message and where the battle starts, its opening: terrain, special field, ship flag, blocked hexes. Auto combat.
4. After the battle: read E8-14's tile and status flags.

**Record**

- gold at day 2's start: 30350
- the click on (10, 68): the marks; tile, movement points and gold after; the message: 5; 6,68 7,68 8,68 9,68 10,68; 8,68,0; 1200; 36450; ADVEVENT:183; ADVEVENT:183: 100 gold
- the click on (13, 68): the marks; the message; the tile where the battle starts: 5; 9,68 10,68 11,68 12,68 13,68; ADVEVENT:16; 11,68,0
- the opening: terrain, special field, ship flag, blocked hexes: terrain water, special field none, ship flag 1, blocked hexes 90, 106, 107
- after: E8-14's tile and status flags: 11,68,0; 0x40000; in a boat (type 1, flag 0); tile 5,68,0; in use 1; type 1; facing 2; owner red; captain E8-14; aboard 1
- anything odd: E8-14's boarding (arrived, tile, points, status, boat): yes; 5,68,0; 0; 0x40000; tile 5,68,0; in use 1; type 1; facing 2; owner red; captain E8-14; aboard 1; E8-14 at day 2's start (points, day's maximum): 1500; 1500; gold and points right before the click: 36350; 1500; the battle (winner; stacks at the start): attacker; attacker 5 Archangels | defender 5 Pikemen
