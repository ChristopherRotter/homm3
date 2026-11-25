# E09 batch TWN-6c — results

> Played by the capture harness on 2026-11-25 (`clients/capture/h3adv.py batch clients/capture/batches/E09-batch-6c.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

Map file: `E09-batch-6c.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `182c5f36`.** The map's description in the scenario list starts with "Stamp 182c5f36.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E09-batch-6c.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, Red and Blue both human, difficulty **Easy**, both players' starting bonus Gold. Before you start it, compare the stamp at the start of the map's description with the stamp above.
4. The records are the game's screens and memory (the capture harness, `clients/capture/batches/E09-batch-6c.toml`, E09 T54): the Artifact Merchants' seven (game +0x1F664, i32[7], -1 an entry bought), the trade window's entries and price, the players' treasuries, a hero's artifacts, the date. Played by hand the seven show at T's merchants.
5. Tiles are (column, row) from 0 at the top left; a town's entrance is the tile its gate stands on. A step: click the target once (the path's marks show), then again (the hero goes).
6. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## TWN-6-8 — The Artifact Merchants' stock on day 1 and day 29, 18 of the 21 minors placed (§3.1, §3.2, D-32) (D-32, ADV-OBJ-006, TWN-STK-001, TWN-STK-002)

- Red hero **TWN-6-8** (Orrin, specialty skill Archery): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Red's turn, day 1, before any hero moves. Note the merchants' seven by class (the window at T shows them).
2. Set Red's gold to 100 000. TWN-6-8 stands south of T at (7, 9): walk onto the entrance (7, 7). On the town screen click the Artifact Merchants; pick the first entry and gold, note its price; press Trade. Note the entries, Red's treasury and TWN-6-8's artifacts. Close the window and the town.
3. End both players' turns until Red's day 29 (month 2, week 1, day 1). Note the date and the merchants' seven by class.

**Record**

- day 1: the seven (name; class): Still Eye of the Dragon (treasure), Speculum (treasure), Hourglass of the Evil Hour (treasure), Ring of Life (minor), Rib Cage (minor), Boots of Speed (minor), Necklace of Ocean Guidance (major); classes: 3 treasures, 3 minors, 1 major; the date: month 1, week 1, day 1
- day 1: the first entry with gold: the price; after Trade: the entries; Red's treasury; TWN-6-8's artifacts: the pair: gold for entry 0 (Still Eye of the Dragon); price: 10000 gold for 1 Still Eye of the Dragon; count: 1; after Trade: entries: empty, Speculum, Hourglass of the Evil Hour, Ring of Life, Rib Cage, Boots of Speed, Necklace of Ocean Guidance; stock: empty, Speculum (treasure), Hourglass of the Evil Hour (treasure), Ring of Life (minor), Rib Cage (minor), Boots of Speed (minor), Necklace of Ocean Guidance (major); treasury: 20 wood, 10 mercury, 20 ore, 10 sulfur, 10 crystal, 10 gems, 90000 gold; worn: 6: Still Eye of the Dragon, 16: Catapult; backpack: none
- day 29: the date; the seven (name; class): month 2, week 1, day 1; Ring of Conjuring (treasure), Shield of the Dwarven Lords (treasure), Spyglass (treasure), Torso of Legion (minor), Crown of the Supreme Magi (minor), Blackshard of the Dead Knight (minor), Targ of the Rampaging Ogre (major); classes: 3 treasures, 3 minors, 1 major
- anything odd: the game's version; the difficulty; the Black Market records: SoD; easy; 0
