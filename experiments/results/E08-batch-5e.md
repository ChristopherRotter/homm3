# E08 batch E5e — results

Map file: `E08-batch-5e.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `bf513e0e`.** The map's description in the scenario list starts with "Stamp bf513e0e.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-5e.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1).
4. The heroes' seed bytes (hero +0x8F), the levels Wisdom (+0x90) and a magic school (+0x3F) were last offered, the movement points and whether a tile is explored for a player (its vision word) show in the game's memory: the capture harness reads them (clients/capture/batches/E08-batch-5e.toml). Played by hand, leave the seed bytes and the two levels empty, read the points from the hero's movement bar and a tile's state from the map (black: unexplored).
5. The map bans every secondary skill but Logistics and Scouting (the scenario's information does not show it), so each level-up should offer those two; another skill, or one alone, goes under the encounter's "anything odd".
6. Each battle by auto combat. The level-up dialog: note the new level, the primary skill it raises, the skill offered left and right and the one taken; take the skill the encounter names, on whichever side it stands.
7. Order: day 1, Red's turn: E5-3a, then E5-3b, each to its step of day 1; end the turn. Blue's turn: E5-10a, then E5-10b, each to its step of day 1; end the turn. Day 2, Red's turn: E5-3a's and E5-3b's points at the turn start; end the turn. Blue's turn: E5-10a's and E5-10b's steps of day 2. Then the map is done.
8. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E5-3a — A level-up's Logistics in the day: the points left, then day 2 (ADV-MOV-002, ADV-MOV-001, HER-ADV-003, HER-LVL-003)

- Red hero **E5-3a** (Orrin, specialty skill Archery, 999 experience): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman
- Opponent: 1 Pikeman (wandering monster, never flees)

1. Day 1, Red's turn, before anything moves: read E5-3a's seed byte and the two levels last offered (the harness) and its movement points.
2. Click the Pikeman east of E5-3a twice; the battle by auto combat. The level-up dialog: note it and take **Logistics**. Read the hero's experience and its movement points.
3. Walk one step east onto (7, 26), where the Pikeman stood (two clicks on it). Read its movement points.
4. Day 2, Red's turn start: read its movement points.

**Record**

- seed byte (hero +0x8F): 
- Wisdom last offered (hero +0x90): 
- a school last offered (hero +0x3F): 
- points at the start: 
- dialog 1, level: 
- dialog 1, primary skill raised: 
- dialog 1, left offer (or the only one): 
- dialog 1, right offer: 
- dialog 1, taken: 
- experience after the battle: 
- points after the battle: 
- points after the step: 
- day 2 points: 
- anything odd: 

## E5-3b — Boots of Speed put on in the day: the points left, then day 2 (ADV-MOV-002, ADV-MOV-001, HER-ADV-003)

- Red hero **E5-3b** (Sylvia, specialty skill Navigation): attack 1, defense 1, power 1, knowledge 1; no skills; Boots of Speed (backpack); no spells; army 10 Pikeman

1. Day 1, Red's turn, after E5-3a: read E5-3b's movement points (Boots of Speed are in its backpack, nothing on its feet).
2. Open its hero screen, put the Boots of Speed from the backpack on its feet and close the screen. Read its movement points (and the movement bar's length, if it changed).
3. Walk one step east onto (17, 26) (two clicks on it). Read its movement points.
4. Day 2, Red's turn start: read its movement points.

**Record**

- points at the start: 
- points with the boots on: 
- points after the step: 
- day 2 points: 
- anything odd: 

## E5-10a — A level-up's Scouting in the day: the tiles at distance 6, at day 2 and after a step (D-26, ADV-FOG-003, ADV-FOG-001, HER-ADV-006, HER-LVL-003)

- Blue hero **E5-10a** (Edric, specialty creature Griffin, 999 experience): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman
- Opponent: 1 Pikeman (wandering monster, never flees)

1. Day 1, Blue's turn, before anything moves: read E5-10a's seed byte and the two levels last offered (the harness) and its movement points. It stands on (8, 10). Read whether Blue has explored the probe tiles W (2, 10), E (15, 10) and C (16, 10).
2. Click the Pikeman east of E5-10a twice; the battle by auto combat. The level-up dialog: note it and take **Scouting**. Read the hero's movement points and the three probes again. No step today.
3. Day 2, Blue's turn start: read the three probes.
4. Walk one step east onto (9, 10), where the Pikeman stood (two clicks on it). Read the three probes.

**Record**

- seed byte (hero +0x8F): 
- Wisdom last offered (hero +0x90): 
- a school last offered (hero +0x3F): 
- points at the start: 

| probe | at the start | after the battle | day 2 start | after the step |
|---|---|---|---|---|
| W (2, 10) |   |   |   |   |
| E (15, 10) |   |   |   |   |
| C (16, 10) |   |   |   |   |

- dialog 1, level: 
- dialog 1, primary skill raised: 
- dialog 1, left offer (or the only one): 
- dialog 1, right offer: 
- dialog 1, taken: 
- points after the battle: 
- anything odd: 

## E5-10b — The Spyglass put on in the day: the tiles at distance 6, at day 2 and after a step (D-26, ADV-FOG-003, ADV-FOG-001, HER-ADV-006)

- Blue hero **E5-10b** (Christian, specialty creature Ballista): attack 1, defense 1, power 1, knowledge 1; no skills; Spyglass (backpack); no spells; army 10 Pikeman

1. Day 1, Blue's turn, after E5-10a: E5-10b stands on (24, 10), the Spyglass in its backpack. Read whether Blue has explored the probe tiles W (18, 10), E (31, 10) and C (32, 10).
2. Open its hero screen, put the Spyglass from the backpack into a free misc slot and close the screen. Read the three probes again. No step today.
3. Day 2, Blue's turn start: read the three probes.
4. Walk one step east onto (25, 10) (two clicks on it). Read the three probes.

**Record**

| probe | at the start | with the Spyglass on | day 2 start | after the step |
|---|---|---|---|---|
| W (18, 10) |   |   |   |   |
| E (31, 10) |   |   |   |   |
| C (32, 10) |   |   |   |   |

- anything odd: 
