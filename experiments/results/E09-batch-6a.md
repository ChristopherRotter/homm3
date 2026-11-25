# E09 batch TWN-6a — results

> Played by the capture harness on 2026-11-25 (`clients/capture/h3adv.py batch clients/capture/batches/E09-batch-6a.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

Map file: `E09-batch-6a.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `1655a41f`.** The map's description in the scenario list starts with "Stamp 1655a41f.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E09-batch-6a.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, Red and Blue both human, difficulty **Easy**, both players' starting bonus Gold. Before you start it, compare the stamp at the start of the map's description with the stamp above.
4. The records are the game's screens and memory (the capture harness, `clients/capture/batches/E09-batch-6a.toml`, E09 T54): the trade window's state (its screen, the count of Marketplaces 0x6AAA98, the pair, the rate, the buttons shown), a town's record (game +0x21614, 0x168 bytes each, `docs/exe/structs.toml` "town"), a hero's backpack, the players' treasuries, the message a click brings (as the record it was written from: `GENRLTXT:274`, never its text). Played by hand they show on the screens.
5. Tiles are (column, row) from 0 at the top left; a town's entrance is the tile its gate stands on. A step: click the target once (the path's marks show), then again (the hero goes).
6. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## TWN-6-1 — The Marketplaces' count at a town (D-70): the gold offered for 1 wood and for 1 ore at 2, 4 and 8 Marketplaces (D-70, TWN-MKT-001)

- Red hero **TWN-6-1** (Orrin, specialty skill Archery): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Red's turn, day 1, before any hero moves. Open T1 (7, 7) from the town list (no visitor) and click its Marketplace: the trade window opens on resources for resources. Pick wood to give and gold to want: note the rate (one wood for so many gold) and the count the window uses; then ore for gold the same. Close the window and the town.
2. Set Red's resources to 50 wood, 10 mercury, 50 ore, 10 sulfur, 10 crystal, 10 gems and 50 000 gold. Open C1 (23, 7), its hall, the Marketplace and Build; the same in C2 (31, 7). Open T1's Marketplace again and note the two rates and the count.
3. Build the Marketplace in C3 (7, 15), C4 (15, 15), C5 (23, 15) and C6 (31, 15) the same way; open T1's Marketplace again and note the two rates and the count.

**Record**

- 2 Marketplaces: the count; 1 wood for (gold); 1 ore for (gold): screen: 0 (resources for resources); count: 2; wood: 1 wood for 37 gold; ore: 1 ore for 37 gold
- the builds in C1 and C2: each town's buildings after: C1: 10, 14, 23; C2: 10, 14, 23
- 4 Marketplaces: the count; 1 wood for (gold); 1 ore for (gold): screen: 0 (resources for resources); count: 4; wood: 1 wood for 62 gold; ore: 1 ore for 62 gold
- the builds in C3 to C6: each town's buildings after: C3: 10, 14, 23; C4: 10, 14, 23; C5: 10, 14, 23; C6: 10, 14, 23; Red's treasury: 20 wood, 10 mercury, 50 ore, 10 sulfur, 10 crystal, 10 gems, 47000 gold
- 8 Marketplaces: the count; 1 wood for (gold); 1 ore for (gold): screen: 0 (resources for resources); count: 8; wood: 1 wood for 112 gold; ore: 1 ore for 112 gold
- anything odd: the game's version; the difficulty: SoD; easy

## TWN-6-3 — An artifact sold from the backpack after its arrow (T-20): the price shown and paid, the artifact removed (T-20, TWN-MKT-005)

- Red hero **TWN-6-3** (Edric, specialty creature Griffin): attack 1, defense 1, power 1, knowledge 1; no skills; Titan's Gladius (backpack), Badge of Courage (backpack), Crest of Valor (backpack), Glyph of Gallantry (backpack), Speculum (backpack), Spyglass (backpack); no spells; army 1 Pikeman

1. Red's turn, day 1, after TWN-6-1 (Red's count 8). Note TWN-6-3's backpack (six artifacts, Titan's Gladius first).
2. TWN-6-3 stands south of T1 at (7, 9): walk onto the entrance (7, 7). On the town screen click the Artifact Merchants: the window opens on the artifacts bought. Press its button to the artifacts sold. Note the five backpack places shown.
3. Press the backpack's right arrow once. Note the five places shown. Pick the first of them, then gold: note the offer (the gold the window says it gives).
4. Press Trade. Note Red's gold gained, TWN-6-3's backpack and the five places shown. Close the window and the town.

**Record**

- before: TWN-6-3's backpack; Red's treasury: backpack: Titan's Gladius, Badge of Courage, Crest of Valor, Glyph of Gallantry, Speculum, Spyglass; treasury: 20 wood, 10 mercury, 50 ore, 10 sulfur, 10 crystal, 10 gems, 47000 gold
- screen 3 as it opened: the places shown; the count: screen: 3 (artifacts sold); places: 18: Titan's Gladius, 19: Badge of Courage, 20: Crest of Valor, 21: Glyph of Gallantry, 22: Speculum; offset: 0; count: 8; buttons: screens 16, 17, 18; trade disabled
- after the right arrow: the places shown; the first place picked with gold: the offer: places: 18: Badge of Courage, 19: Crest of Valor, 20: Glyph of Gallantry, 21: Speculum, 22: Spyglass; offset: 1; the pair: position 18 (Badge of Courage) for gold; the offer: 1 Badge of Courage for 5500 gold; most: 1; buttons: screens 16, 17, 18; trade enabled
- after Trade: the gold gained; TWN-6-3's backpack; the places shown: gained: +5500 gold; treasury: 20 wood, 10 mercury, 50 ore, 10 sulfur, 10 crystal, 10 gems, 52500 gold; backpack: Titan's Gladius, Crest of Valor, Glyph of Gallantry, Speculum, Spyglass; places: 18: Titan's Gladius, 19: Crest of Valor, 20: Glyph of Gallantry, 21: Speculum, 22: Spyglass; offset: 1
- anything odd: TWN-6-3's visit: a town screen; how it closed; the messages: town screen: yes; closed: click; messages: none

## TWN-6-5 — A garrison hero and no visitor (TD-28): the merchants, the Freelancer's Guild, the Blacksmith, the Ballista Yard and the Marketplace clicked (TD-28, T-28, TWN-MKT-002, TWN-SMT-001)

- Red hero **TWN-6-5** (Tyris, specialty creature Cavalier): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Red's turn, day 1, after TWN-6-3: TWN-6-3 is T1's visitor. Open T1 from the town list. Move TWN-6-3 into the garrison (its portrait, then the garrison's): T1 has a garrison hero and no visitor.
2. Click the Artifact Merchants: note the message (its text) and whether a window opens; close it. Click the Marketplace: note the window's screen and its buttons to the other screens. Close the window and the town.
3. TWN-6-5 stands south of S1 at (15, 9): walk onto the entrance (15, 7). On the town screen move TWN-6-5 into the garrison. Click the Freelancer's Guild, the Blacksmith and the Ballista Yard in turn: note each message and whether a window or dialog opens. Click the Marketplace: note the window's screen and its buttons. Close the window and the town.

**Record**

- T1: the visitor and the garrison hero after the move: visitor: none; garrison hero: TWN-6-3
- T1: the Artifact Merchants clicked: the message; a window: messages: GENRLTXT:274; after: town
- T1: the Marketplace clicked: the screen; its buttons: screen: 0 (resources for resources); buttons: screens 17; trade disabled; count: 8
- S1: the visitor and the garrison hero after the move: visitor: none; garrison hero: TWN-6-5
- S1: the Freelancer's Guild clicked: the message; a window: messages: GENRLTXT:274; after: town
- S1: the Blacksmith clicked: the message; a dialog: messages: GENRLTXT:274; after: town
- S1: the Ballista Yard clicked: the message; a dialog: messages: GENRLTXT:274; after: town
- S1: the Marketplace clicked: the screen; its buttons: screen: 0 (resources for resources); buttons: screens 17; trade disabled; count: 8
- anything odd: TWN-6-5's visit: a town screen; how it closed: town screen: yes; closed: click

## TWN-6-4 — Ring of the Magi sold at 1 Marketplace (§3.4): the price of a combination artifact (HER-ART-008, TWN-MKT-005)

- Blue hero **TWN-6-4** (Valeska, specialty creature Archer): attack 1, defense 1, power 1, knowledge 1; no skills; Ring of the Magi (backpack); no spells; army 1 Pikeman

1. Blue's turn, day 1. Note TWN-6-4's backpack (Ring of the Magi alone).
2. TWN-6-4 stands south of B at (23, 32): walk onto the entrance (23, 30). On the town screen click the Artifact Merchants, then the window's button to the artifacts sold. Pick Ring of the Magi (the first backpack place), then gold: note the offer and the count. Close the window and the town without trading.

**Record**

- before: TWN-6-4's backpack; Blue's treasury: backpack: Ring of the Magi; treasury: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30800 gold
- Ring of the Magi with gold: the offer; the count: places: 18: Ring of the Magi, 19: empty, 20: empty, 21: empty, 22: empty; the pair: position 18 (Ring of the Magi) for gold; the offer: 1 Ring of the Magi for 600 gold; count: 1
- anything odd: TWN-6-4's visit: a town screen; how it closed: town screen: yes; closed: click
