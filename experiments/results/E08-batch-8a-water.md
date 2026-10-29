# E08 batch E8a water — results

> Played by the capture harness on 2026-10-28 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-8a-water.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus random, Show Route on; read from the game's memory.

Map file: `E08-batch-8a-water.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `843348c3`.** The map's description in the scenario list starts with "Stamp 843348c3.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-8a-water.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. If the game refuses the map, stops or crashes, note where and stop: that is this row's result. Save right away (save 1).
4. Most records are the game's memory: a hero's tile, movement points (+0x4D), day's maximum (+0x49) and status flags (+0x105; bit 18, 0x40000, is the hero in a boat), the boats' records (game +0x4E3BC, `docs/exe/structs.toml` "boat"), the path marks (batch E1a's). The capture harness reads them (`clients/capture/batches/E08-batch-8a-water.toml`, E08 T68). Played by hand, read the screens instead and leave the memory-only lines empty.
5. Tiles are (column, row) from 0 at the top left. A walk: click the target once (the path's marks show; note them), then again (the hero goes).
6. Order: day 1, Red's turn: E8-5. Then the map is done (Blue does nothing).
7. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E8-5 — A hero written on a water tile, no boat there (D-44, ADV-SEA-007)

- Red hero **E8-5** (Orrin, specialty skill Archery): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel
- Blue hero **E8-5b** (Valeska, specialty creature Archer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. Day 1, Red's turn: E8-5 stands on the water tile (10, 10), one Angel in its army (2 000 points on land, 1 500 at sea). Note what the screen shows under it (a boat or not).
2. Read E8-5's tile, movement points, day's maximum and status flags, and the boats' records (how many in use).
3. Click the water tile (9, 10) west of it once (note the marks), then again. Read its tile and movement points.
4. Click the land tile east of it, (11, 10) (from (9, 10) if it moved: the tile two east), once (note the marks), then again. Read its tile, movement points, day's maximum and status flags.

**Record**

- what the screen shows under E8-5 (a boat or not): on foot
- day 1 start: tile, movement points, day's maximum, status flags; boats in use: 10,10,0; 2000; 2000; 0x0; 0
- the click on (9, 10): marks; tile and movement points after: none; 10,10,0; 2000
- the click on (11, 10): marks; tile, movement points, day's maximum, status flags after: 11,10; 11,10,0; 1900; 2000; 0x0
- anything odd: boats in use after (11, 10): 0; the screen at the first step (`dump`, looked at by the agent, T68a): E8-5 drawn on horseback on the water tile (10, 10), no boat under it; the game took the map, set it up and played it without a message (two runs, the same tiles, points and flags)
