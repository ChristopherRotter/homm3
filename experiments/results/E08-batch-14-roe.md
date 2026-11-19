# E08 batch E14 RoE — results

> Played by the capture harness on 2026-11-19 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-14-roe.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

Map file: `E08-batch-14-roe.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `a88dd771`.** The map's description in the scenario list starts with "Stamp a88dd771.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-14-roe.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy** (the scenario list shows it as a Restoration of Erathia map). If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1).
4. The records are the game's memory: the hero's tile, army and points (hero +0x4D), Red's treasury, the game's version (game +0x1F698) and the Hill Fort's rows (prices, states, the whole +0x298). The capture harness reads them (`clients/capture/batches/E08-batch-14.toml`, E08 T129). Played by hand, read the screens instead and leave the memory-only lines empty.
5. Tiles are (column, row) from 0 at the top left. A walk: click the target once (the path's marks show), then again (the hero goes).
6. Order: day 1, Red's turn: E14-14. Then the map is done.
7. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E14-14 — An RoE map's Hill Fort: Air Elementals and Pikemen (ADV-HFT-001, ADV-HFT-002)

- Red hero **E14-14** (Orrin, specialty skill Archery): its hero type's primary skills; no skills; no artifacts; no spells; army 5 Air Elemental, 5 Pikeman
- Blue hero **Valeska** (Valeska, specialty creature Archer): everything its hero type's own (army, skills, artifacts, spells)

1. **E14-14.** Day 1, Red's turn: read the game's version. E14-14 stands at (10, 12) with 5 Air Elementals (slot 0) and 5 Pikemen (slot 1). Note Red's treasury. Click the Hill Fort's entrance (10, 10) twice: E14-14 walks onto it and the dialog opens. Note E14-14's tile and points.
2. Note both rows (price or text, state) and the whole's state. Click the Air Elementals' row button: note what happens. Click the Pikemen's row button: note what happens. Note the rows again, E14-14's army and Red's treasury. Close the dialog.
3. Blue's hero at (30, 30) only makes the map playable; it plays nothing.

**Record**

- the game's version; before: Red's treasury; at the dialog: E14-14's tile; points: the game's version: SoD; treasury: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30700 gold; army: 0: 5 Air Elementals, 1: 5 Pikemen; at the dialog: 10,10 with 1360 points; in order: Hill Fort's dialog (the whole 1, affordable)

| slot | stack | price | state |
|---|---|---|---|
| 0 Air Elementals | 5 Air Elementals | 31 gold | 1, affordable |
| 1 Pikemen | 5 Pikemen | 0 (shown GENRLTXT:345) | 1, affordable |

- the whole's state: 31 gold, state 1, affordable; buttons: all enabled, row 0 enabled, row 1 enabled, row 2 disabled, row 3 disabled, row 4 disabled, row 5 disabled, row 6 disabled
- the Air Elementals' button: what happens; the Pikemen's: what happens: the Air Elementals: none; the change: -31 gold; army: 0: 5 Storm Elementals, 1: 5 Pikemen; the Pikemen: none; the change: none; army: 0: 5 Storm Elementals, 1: 5 Halberdiers
- after: the rows; army; Red's treasury: the rows: 0: 5 Storm Elementals: 0 (shown GENRLTXT:346), state 0, no upgrade; 1: 5 Halberdiers: 0 (shown GENRLTXT:346), state 0, no upgrade; the whole: 0, state 0, no upgrade; army: 0: 5 Storm Elementals, 1: 5 Halberdiers; treasury: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30669 gold
- anything odd: the date at the start: month 1, week 1, day 1; the difficulty: easy; the game's version reads SoD (game +0x1F698) on this map, whose header is the RoE format (0x0E; as batch E7a's RoE map, `E08-batch-7a-roe.md`): so the Air Elementals were upgradable (31 gold, no message) and became Storm Elementals, against the reading's game version 0.
