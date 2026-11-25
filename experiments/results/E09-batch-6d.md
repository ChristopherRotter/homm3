# E09 batch TWN-6d — results

Map file: `E09-batch-6d.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `d22528b3`.** The map's description in the scenario list starts with "Stamp d22528b3.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E09-batch-6d.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, Red and Blue both human, difficulty **Easy**, both players' starting bonus Gold. Before you start it, compare the stamp at the start of the map's description with the stamp above.
4. The scenario bans the secondary skill Fire Magic (the scenario's information does not show it).
5. The records are the game's screens and memory (the capture harness, `clients/capture/batches/E09-batch-6d.toml`, E09 T54): a town's record (game +0x21614, 0x168 bytes each, `docs/exe/structs.toml` "town": the boat spot +0x8, +0x9, the buildings standing +0x150), the boats (the game's boat records: tile, owner, in use), the players' treasuries, the University's four slots (their states and texts as the records they were written from), a hero's secondary skills. Played by hand they show on the screens and the map.
6. Tiles are (column, row) from 0 at the top left; a town's entrance is the tile its gate stands on. A step: click the target once (the path's marks show), then again (the hero goes).
7. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## TWN-6-6 — A Fortress's boat bought over the docked one (TD-31, T-21): the ship clicked, the Shipyard's uncovered pixel, the boats (TD-31, T-21, TWN-SHY-001, TWN-SET-011)

- Red hero **TWN-6-6** (Orrin, specialty skill Archery): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Red's turn, day 1, before any hero moves. Note F's boat spot and the boats on the map.
2. Set Red's resources to 50 wood, 10 mercury, 50 ore, 10 sulfur, 10 crystal, 10 gems and 50 000 gold. Open F (10, 9) from the town list (no visitor). Click the Shipyard: the boat dialog opens; press Build. Note the boats, Red's treasury and F's buildings standing. Close the town.
3. Open F again: the ship (building 20) is drawn over the dock. Click the ship: note whether anything opens. Then click the screen at (233, 341) (the dock's one pixel the ship does not cover): note what opens; press Build. Note the boats, Red's treasury and F's buildings standing. Close the town.

**Record**

- before: F's boat spot; the boats; Red's treasury: 
- the first boat: what the Shipyard's click opened; the boats; Red's treasury; F's buildings standing: 
- F reopened: F's buildings standing; the ship clicked: what opens: 
- at (233, 341): what opens; after Build: the boats; Red's treasury; F's buildings standing: 
- anything odd: 

## TWN-6-7 — The Conflux's Magic University on a map banning Fire Magic (TD-30, T-23): the four slots, a school bought (TD-30, T-23, TWN-UNI-001)

- Red hero **TWN-6-7** (Adela, specialty spell Bless): attack 1, defense 1, power 1, knowledge 1; Basic Wisdom, Basic Air Magic; no artifacts; no spells; army 1 Pikeman

1. Red's turn, day 1. Note TWN-6-7's secondary skills (Wisdom, Air Magic) and Red's treasury.
2. Set Red's gold to 5 000. TWN-6-7 (a Cleric) stands south of C at (26, 10): walk onto the entrance (26, 9). On the town screen click the Magic University. Note each of the four slots' state (learnable, known, cannot learn, no more skills) and the text it shows.
3. Click the Fire Magic slot: note what opens and whether its buy button is enabled; press it. Note TWN-6-7's secondary skills, Red's treasury and the four slots' states. Close the University and the town.

**Record**

- before: TWN-6-7's secondary skills; Red's treasury: 
- the University: the four slots (skill: state, text): 
- the Fire Magic slot clicked: what opens; the buy button: 
- after the purchase: TWN-6-7's secondary skills; Red's treasury; the four slots: 
- anything odd: 
