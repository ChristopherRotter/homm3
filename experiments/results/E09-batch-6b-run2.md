# E09 batch TWN-6b — results

> Played by the capture harness on 2026-11-25 (`clients/capture/h3adv.py batch clients/capture/batches/E09-batch-6b.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

Map file: `E09-batch-6b.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `011092b2`.** The map's description in the scenario list starts with "Stamp 011092b2.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

**Teams: red and blue.** The map makes them allies; any other player plays alone.

1. Copy `E09-batch-6b.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, Red and Blue both human, difficulty **Easy**, both players' starting bonus Gold; Tan is the computer's (the map lets no human play it). Red and Blue are allied. Before you start it, compare the stamp at the start of the map's description with the stamp above.
4. The records are the game's screens and memory (the capture harness, `clients/capture/batches/E09-batch-6b.toml`, E09 T54): the trade window's state (its screen, the count of Marketplaces 0x6AAA98, the pair, the price, the entries), the players' treasuries and teams, a hero's worn artifacts and backpack, the Artifact Merchants' seven (game +0x1F664). Played by hand they show on the screens.
5. Tiles are (column, row) from 0 at the top left; a town's entrance is the tile its gate stands on. A step: click the target once (the path's marks show), then again (the hero goes).
6. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## TWN-6-2 — An ally's Artifact Merchants for a player with no Marketplace (TD-29, T-19): the prices, the purchase, the treasury (TD-29, T-19, TWN-MKT-001, TWN-MKT-004)

- Red hero **TWN-6-2** (Edric, specialty creature Griffin): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Red's turn, day 1. Note the teams, Red's treasury and TWN-6-2's artifacts, and the merchants' seven.
2. TWN-6-2 stands south of B at (23, 9): walk onto B's entrance (23, 7). The town screen opens (an ally's town). Click the Marketplace: note whether anything opens.
3. Click the Artifact Merchants: note the window's screen, the count it uses and the entries. Pick the first entry, then gold: note its price; the same for the last entry (the major).
4. Press Trade (the last entry with gold). Note Red's treasury, TWN-6-2's artifacts and the entries. Close the window and the town.

**Record**

- before: the teams; Red's treasury; TWN-6-2's worn artifacts and backpack; the stock: teams: red 0, blue 0, tan 1, green 0, orange 0, purple 0, teal 0, pink 0; treasury: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30800 gold; worn: 16: Catapult; backpack: none; stock: Centaurs Axe (treasure), Cards of Prophecy (treasure), Spirit of Oppression (treasure), Equestrian's Gloves (minor), Scales of the Greater Basilisk (minor), Helm of Chaos (minor), Surcoat of Counterpoise (major)
- at B: the Marketplace clicked: what opens: after: town; messages: none
- the merchants clicked: the screen; the count; the entries: screen: 2 (artifacts bought); count: 0; entries: Centaurs Axe, Cards of Prophecy, Spirit of Oppression, Equestrian's Gloves, Scales of the Greater Basilisk, Helm of Chaos, Surcoat of Counterpoise; buttons: screens 16, 17, 19; trade disabled
- the first entry with gold: the price; the last entry with gold: the price: first: gold for entry 0 (Centaurs Axe); price: 0 gold for 1 Centaurs Axe; most: 0; amount: 1; last: gold for entry 6 (Surcoat of Counterpoise); price: 0 gold for 1 Surcoat of Counterpoise; most: 0; amount: 1; screens 16, 17, 19; trade enabled
- after Trade: Red's treasury; TWN-6-2's worn artifacts and backpack; the entries; the stock: treasury: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30800 gold; gained: none; worn: 1: Surcoat of Counterpoise, 16: Catapult; backpack: none; entries: Centaurs Axe, Cards of Prophecy, Spirit of Oppression, Equestrian's Gloves, Scales of the Greater Basilisk, Helm of Chaos, empty; stock: Centaurs Axe (treasure), Cards of Prophecy (treasure), Spirit of Oppression (treasure), Equestrian's Gloves (minor), Scales of the Greater Basilisk (minor), Helm of Chaos (minor), empty; questions: none; messages: none
- anything odd: the game's version; the difficulty: SoD; easy; TWN-6-2's visit: a town screen; how it closed; the town's visitor: town screen: yes; closed: click; visitor: none
