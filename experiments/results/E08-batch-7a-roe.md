# E08 batch E7a RoE — results

> Played by the capture harness on 2026-10-26 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-7a-roe.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus random, Show Route on; read from the game's memory.

Map file: `E08-batch-7a-roe.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `35da79fc`.** The map's description in the scenario list starts with "Stamp 35da79fc.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-7a-roe.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy** (the scenario list shows it as a Restoration of Erathia map). Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1).
4. The record is the game's memory: each Witch Hut's word on its tile (bits 13 to 19: the skill). The capture harness reads it (`clients/capture/batches/E08-batch-7a-roe.toml`, E08 T57). Played by hand, visit the huts with a hero instead (each tells its skill) and note them.
5. Nothing is played: the row reads the new game's draws. Then the map is done.
6. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E7-7 — Sixty Witch Huts on an RoE map: their skills (D-33, A-21, ADV-TCH-005)

- Red hero **E7-7** (Sylvia, specialty skill Navigation): everything its hero type's own (army, skills, artifacts, spells)
- Blue hero **Voy** (Voy, specialty skill Navigation): everything its hero type's own (army, skills, artifacts, spells)

1. At the start (save 1): sixty Witch Huts stand on (4, 5) to (31, 20), ten to a row 3 columns apart (each on the tile west of its picture's corner, the tile it blocks and is visited on). Read each one's word on that tile (bits 13 to 19: the skill, `SSTRAITS.TXT` order: 0 Pathfinding … 27 First Aid).
2. The heroes E7-7 (Red) and Blue's stand at (3, 30) and (30, 30) only to make the map playable; they play nothing.

**Record**

| Witch Hut | the word (hex) | the skill |
|---|---|---|
| (4, 5) | 0x00000000 | Pathfinding |
| (7, 5) | 0x00006000 | Scouting |
| (10, 5) | 0x0001a000 | Estates |
| (13, 5) | 0x00012000 | Luck |
| (16, 5) | 0x00010000 | Mysticism |
| (19, 5) | 0x0001c000 | Fire Magic |
| (22, 5) | 0x0001c000 | Fire Magic |
| (25, 5) | 0x00000000 | Pathfinding |
| (28, 5) | 0x0001c000 | Fire Magic |
| (31, 5) | 0x0001c000 | Fire Magic |
| (4, 8) | 0x00008000 | Diplomacy |
| (7, 8) | 0x00016000 | Eagle Eye |
| (10, 8) | 0x0001a000 | Estates |
| (13, 8) | 0x00016000 | Eagle Eye |
| (16, 8) | 0x00002000 | Archery |
| (19, 8) | 0x00010000 | Mysticism |
| (22, 8) | 0x00014000 | Ballistics |
| (25, 8) | 0x00000000 | Pathfinding |
| (28, 8) | 0x00002000 | Archery |
| (31, 8) | 0x0001e000 | Air Magic |
| (4, 11) | 0x00002000 | Archery |
| (7, 11) | 0x0001c000 | Fire Magic |
| (10, 11) | 0x00000000 | Pathfinding |
| (13, 11) | 0x00002000 | Archery |
| (16, 11) | 0x00000000 | Pathfinding |
| (19, 11) | 0x00004000 | Logistics |
| (22, 11) | 0x00000000 | Pathfinding |
| (25, 11) | 0x00012000 | Luck |
| (28, 11) | 0x00014000 | Ballistics |
| (31, 11) | 0x0000e000 | Wisdom |
| (4, 14) | 0x0000a000 | Navigation |
| (7, 14) | 0x0001a000 | Estates |
| (10, 14) | 0x00008000 | Diplomacy |
| (13, 14) | 0x0001a000 | Estates |
| (16, 14) | 0x00006000 | Scouting |
| (19, 14) | 0x0001a000 | Estates |
| (22, 14) | 0x00006000 | Scouting |
| (25, 14) | 0x00012000 | Luck |
| (28, 14) | 0x00014000 | Ballistics |
| (31, 14) | 0x00012000 | Luck |
| (4, 17) | 0x00004000 | Logistics |
| (7, 17) | 0x0000e000 | Wisdom |
| (10, 17) | 0x00008000 | Diplomacy |
| (13, 17) | 0x0000e000 | Wisdom |
| (16, 17) | 0x0001a000 | Estates |
| (19, 17) | 0x00014000 | Ballistics |
| (22, 17) | 0x00000000 | Pathfinding |
| (25, 17) | 0x0001c000 | Fire Magic |
| (28, 17) | 0x00010000 | Mysticism |
| (31, 17) | 0x00012000 | Luck |
| (4, 20) | 0x00014000 | Ballistics |
| (7, 20) | 0x00014000 | Ballistics |
| (10, 20) | 0x00010000 | Mysticism |
| (13, 20) | 0x00008000 | Diplomacy |
| (16, 20) | 0x0001e000 | Air Magic |
| (19, 20) | 0x00016000 | Eagle Eye |
| (22, 20) | 0x0001a000 | Estates |
| (25, 20) | 0x0001a000 | Estates |
| (28, 20) | 0x00016000 | Eagle Eye |
| (31, 20) | 0x00010000 | Mysticism |

- how many teach a skill of 16 to 27 (Water Magic to First Aid), Leadership or Necromancy: 
- anything odd: the game's version (game +0x1F698): SoD
