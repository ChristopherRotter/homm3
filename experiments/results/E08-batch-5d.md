# E08 batch E5d — results

> Played by the capture harness on 2026-10-22 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-5d.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus random, Show Route on; read from the game's memory.

Map file: `E08-batch-5d.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `1255306f`.** The map's description in the scenario list starts with "Stamp 1255306f.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-5d.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1).
4. Every hero's seed byte (hero +0x8F) and the levels Wisdom (+0x90) and a magic school (+0x3F) were last offered show only in the game's memory, the movement points too: the capture harness reads them (clients/capture/batches/E08-batch-5d.toml). Played by hand, leave the seed byte and the two levels empty and read the points from the hero's movement bar.
5. No hero moves. Order: day 1, Red's turn: read E5-2d and E5-2e; end the turn. Blue's turn: read E5-2f; end the turn. Day 2, Red's turn: read E5-2d's and E5-2e's points again; end the turn. Blue's turn: read E5-2f's points again. Then the map is done: two days, no battle.
6. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E5-2d — Kyrre, Basic Logistics, level 2 at load (1 000 experience), 10 Dwarves (D-22, HER-ADV-003, ADV-MOV-001, D-20, GAM-SET-002)

- Red hero **E5-2d** (Kyrre, specialty skill Logistics, 1000 experience): attack 1, defense 1, power 1, knowledge 1; Basic Logistics; no artifacts; no spells; army 10 Dwarf

1. Day 1, Red's turn, before anything moves: read E5-2d's seed byte and the two levels last offered (the harness), its experience, level, primary and secondary skills (its portrait's window) and its movement points. It never moves.
2. Day 2, Red's turn start: read its movement points again.
3. Note any worn artifact (a starting bonus) under "anything odd".

**Record**

- seed byte (hero +0x8F): 196
- Wisdom last offered (hero +0x90): 0
- a school last offered (hero +0x3F): 0
- experience at the start: 1000
- level at the start: 2
- primary skills at the start (attack/defense/power/knowledge): 1/2/1/1
- secondary skills at the start (in the hero's order, each with its level): Logistics advanced
- day 1 points: 1829
- day 2 points: 1829
- anything odd: 

## E5-2e — Gunnar, Basic Logistics, level 3 at load (2 000 experience), 10 Dwarves (D-22, HER-ADV-003, ADV-MOV-001, D-20, GAM-SET-002)

- Red hero **E5-2e** (Gunnar, specialty skill Logistics, 2000 experience): attack 1, defense 1, power 1, knowledge 1; Basic Logistics; no artifacts; no spells; army 10 Dwarf

1. Day 1, Red's turn, before anything moves: read E5-2e as E5-2d. It never moves.
2. Day 2, Red's turn start: read its movement points again.
3. Note any worn artifact (a starting bonus) under "anything odd".

**Record**

- seed byte (hero +0x8F): 167
- Wisdom last offered (hero +0x90): 0
- a school last offered (hero +0x3F): 0
- experience at the start: 2000
- level at the start: 3
- primary skills at the start (attack/defense/power/knowledge): 1/2/1/2
- secondary skills at the start (in the hero's order, each with its level): Logistics advanced, First Aid basic
- day 1 points: 1845
- day 2 points: 1845
- anything odd: 

## E5-2f — Dessa, Basic Logistics and seven other skills, level 6 at load (6 200 experience), 10 Dwarves (D-22, HER-ADV-003, ADV-MOV-001, D-20, GAM-SET-002)

- Blue hero **E5-2f** (Dessa, specialty skill Logistics, 6200 experience): attack 1, defense 1, power 1, knowledge 1; Basic Logistics, Basic Archery, Basic Armorer, Basic Offense, Basic Leadership, Basic Luck, Basic Resistance, Basic First Aid; no artifacts; no spells; army 10 Dwarf

1. Day 1, Blue's turn, before anything moves: read E5-2f as E5-2d. It never moves.
2. Day 2, Blue's turn start: read its movement points again.
3. Note any worn artifact (a starting bonus) under "anything odd".

**Record**

- seed byte (hero +0x8F): 11
- Wisdom last offered (hero +0x90): 0
- a school last offered (hero +0x3F): 0
- experience at the start: 6200
- level at the start: 6
- primary skills at the start (attack/defense/power/knowledge): 2/2/4/1
- secondary skills at the start (in the hero's order, each with its level): Logistics expert, Archery advanced, Armorer basic, Offense advanced, Leadership basic, Luck basic, Resistance basic, First Aid advanced
- day 1 points: 2085
- day 2 points: 2085
- anything odd: 
