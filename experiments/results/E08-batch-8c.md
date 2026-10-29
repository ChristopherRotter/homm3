# E08 batch E8c — results

> Played by the capture harness on 2026-10-29 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-8c.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus random, Show Route on; read from the game's memory.

Map file: `E08-batch-8c.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `acfc5755`.** The map's description in the scenario list starts with "Stamp acfc5755.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-8c.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1).
4. Most records are the game's memory: a hero's tile and movement points (+0x4D), an object's word on its tile, a tile explored, the players' treasuries, the path marks (batch E1a's). The capture harness reads them (`clients/capture/batches/E08-batch-8c.toml`, E08 T69b). Played by hand, read the screens instead and leave the memory-only lines empty.
5. Tiles are (column, row) from 0 at the top left. A walk: click the target once (the path's marks show; note them, or that none show), then again (the hero goes, if a path showed).
6. Order: day 1, Red's turn: E8-17a, E8-17b, E8-17c, E8-17d. Then the map is done (Blue does nothing).
7. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E8-17 — A flyer's way east across water: a Flotsam from land, a Flotsam from water, plain water, an empty boat from land, an empty boat from water (D-46, ADV-MOV-009)

- Red hero **E8-17a** (Adela, specialty spell Bless): attack 1, defense 1, power 1, knowledge 1; no skills; Angel Wings (shoulders); no spells; army 10 Pikeman
- Red hero **E8-17b** (Cuthbert, specialty spell Weakness): attack 1, defense 1, power 1, knowledge 1; no skills; Angel Wings (shoulders); no spells; army 10 Pikeman
- Red hero **E8-17c** (Adelaide, specialty spell Frost Ring): attack 1, defense 1, power 1, knowledge 1; no skills; Angel Wings (shoulders); no spells; army 10 Pikeman
- Red hero **E8-17d** (Loynis, specialty spell Prayer): attack 1, defense 1, power 1, knowledge 1; no skills; Angel Wings (shoulders); no spells; army 10 Pikeman
- Red hero **E8-17e** (Orrin, specialty skill Archery): attack 1, defense 1, power 1, knowledge 1; no skills; Angel Wings (shoulders); no spells; army 10 Pikeman
- Blue hero **E8-17x** (Valeska, specialty creature Archer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. Day 1, Red's turn: five Red heroes wearing Angel Wings stand at the west end of five pockets closed by rock: E8-17a at (11, 4), a Flotsam on the water tile (12, 4) east of it; E8-17b at (11, 9), water on (12, 9) to (14, 9) and a Flotsam on (13, 9); E8-17c at (11, 14), plain water on (12, 14) to (14, 14); E8-17d at (11, 19), an empty boat on the water tile (12, 19); E8-17e at (11, 24), water on (12, 24) to (14, 24) and an empty boat on (13, 24). Each pocket's only way east crosses its water.
2. Read each hero's movement points, whether the far side (E8-17a and E8-17d: the tile just past the water, (13, 4) and (13, 19); E8-17b, E8-17c and E8-17e: (15, 9), (15, 14) and (15, 24)) is explored, each Flotsam's word, the boats' records and Red's treasury (wood and gold).
3. E8-17a: click (13, 4) once (note the marks, or that none show), then again. Read E8-17a's tile and movement points, Red's wood and gold, the Flotsam's word; note any message.
4. E8-17b: click (15, 9) the same way. The same reads.
5. E8-17c: click (15, 14) the same way. The same reads (no Flotsam).
6. E8-17d: click (13, 19) the same way. The same reads, and the boat's record (no Flotsam).
7. E8-17e: click (15, 24) the same way. The same reads, and the boat's record (no Flotsam).

**Record**

| hero | the far side explored | the Flotsam's word before | the marks (or none) | tile after | movement points before / after | wood, gold before / after | the Flotsam after (word, or gone) |
|---|---|---|---|---|---|---|---|
| E8-17a (a Flotsam, from land) | yes | 0x00000000; Flotsam: nothing | none; none | 11,4,0 | 1560; 1560 | 37; 30000; 37; 30000 | 0x00000000; Flotsam: nothing |
| E8-17b (a Flotsam, from water) | yes | 0x00000000; Flotsam: nothing | none; none | 11,9,0 | 1560; 1560 | 37; 30000; 37; 30000 | 0x00000000; Flotsam: nothing |
| E8-17c (plain water) | yes | no Flotsam | 12,14 13,14 14,14 15,14; green | 15,14,0 | 1560; 1160 | 37; 30000; 37; 30000 | no Flotsam |
| E8-17d (an empty boat, from land) | yes | no Flotsam | 12,19 13,19; green | 13,19,0 | 1560; 1360 | 37; 30000; 37; 30000 | no Flotsam |
| E8-17e (an empty boat, from water) | yes | no Flotsam | 12,24 13,24 14,24 15,24; green | 15,24,0 | 1560; 1160 | 37; 30000; 37; 30000 | no Flotsam |

- anything odd: E8-17a worn at the start (the starting bonus): Angel Wings (shoulders); E8-17b worn at the start: Angel Wings (shoulders); E8-17c worn at the start: Angel Wings (shoulders); E8-17d worn at the start: Angel Wings (shoulders); E8-17e worn at the start: Angel Wings (shoulders); the boats' records at the start (E8-17d's, E8-17e's): tile 12,19,0; in use 1; type 0; facing 2; owner none; captain none; aboard 0; tile 13,24,0; in use 1; type 1; facing 2; owner none; captain none; aboard 0; boats in use at the start: 2; E8-17d's status flags after (hex; bit 18 the boat): 0x0; the boat's record after E8-17d: tile 12,19,0; in use 1; type 0; facing 2; owner none; captain none; aboard 0; boats in use after E8-17d: 2; E8-17e's status flags after (hex; bit 18 the boat): 0x0; the boat's record after E8-17e: tile 13,24,0; in use 1; type 1; facing 2; owner none; captain none; aboard 0; boats in use after E8-17e: 2; the screen at the first step (`dump`, looked at by the agent, T69b): the pockets drawn as the spec writes them, rock around each, the Flotsams on (12, 4) and (13, 9); one run (no random draw decides a path)
