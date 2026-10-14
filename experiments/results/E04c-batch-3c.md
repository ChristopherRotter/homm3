# E04c batch 3c — results

Map file: `E04c-batch-3c.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `1fb8059c`.** The map's description in the scenario list starts with "Stamp 1fb8059c.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Run by an agent** (E04c T5e, 2026-10-14) on the client lane's capture harness: the same Complete
4.0 `Heroes3.exe`, under Wine in the cloud sandbox, without the HD mod; hot seat (Red and Blue
human, the agent commanding both armies), one map per encounter (`h3-experiment … --only <id>`),
played by `experiments/agent/siege.py`: the clock stepped 10 ms at a time and the game's memory
read after each step (each stack's hex +0x38, count +0x4C, the top creature's lost hit points
+0x58 and turn flag +0x4F0; the gate +0x53A4, the walls +0x13FA8, the hexes' Land Mine bit and the
battlefield's objects). A step's hit points run from the stack's turn to the next turn the agent
commands; the keep's and the Catapult's shots at a round's start fall outside every step (the
Catapult hit walls in M and G, never the gate). A run whose hero wore a bonus artifact (the random
starting bonus) was played again: T on its fifth attempt; M and G on the first, wearing only the
Catapult. M and G gave the same values in three runs.

1. Copy `E04c-batch-3c.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps` and start it in hot seat (Red and Blue human). Save right away.
2. Each Red hero stands south of a Blue town's entrance. Play the encounters in any order: select the hero and step north into the town; command both armies as the encounter says (every stack without a step defends).
3. Note what the encounter's record asks right after each step: the hit points a stack lost are its count times its hit points before and after (right-click). The keep's and the Catapult's shots at a round's start belong to no step. Then end the battle as you like and load the save for the next encounter.
4. Fill in this file (anything unexpected goes into the last line of an encounter) and keep it in the repo.

## M — The moat at an Inferno with a Citadel: stop, damage, defense (BAT-SIE-004, BAT-SIE-005, BAT-SIE-006, D-10)

- Red hero **M** (Caitlin, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 200 Sprite, 10 Magic Elemental, 10 Peasant
- Opponent: Blue Inferno town (Village Hall, fort, citadel); garrison 1 Archangel, 50 Peasant, 10 Halfling

1. Round 1: the Archangel waits. The Sprites (column 1, row 2) fly into the moat at hex 44 (row 2, column 10). The Magic Elementals (column 1, row 5) walk into the moat at hex 111 (row 6, column 9). When the Archangel's turn comes back, it strikes the Sprites from the left (from hex 43).
2. Round 2: the Sprites defend where they stand (in the moat). The Magic Elementals walk to hex 146 (row 8, column 10, the moat), two hexes down the moat past hex 129.
3. Round 3: the Magic Elementals defend where they stand. Then end the battle as you like.

**Record**

- M1: the Sprites' lost hit points as they land on 44: 90 (600 -> 510, the damage as they landed)
- M2: the Magic Elementals' lost hit points as they walk into 111: 90 (800 -> 710; their walk 86 -> 87 … 94 -> 111, the damage on 111)
- M3: the Sprites' lost hit points from the Archangel's strike: 127 (510 -> 383; the Archangel stood on 42 and 43 to strike, outside the moat)
- M4: the Sprites' lost hit points as they defend in the moat: 90 (368 -> 278)
- M5: where the Magic Elementals end (hex) and their lost hit points: 146, 90 (710 -> 620; their walk 111 -> 128 -> 145 -> 146, the damage on 146 only)
- M6: the Magic Elementals' lost hit points as they defend in the moat: 90 (620 -> 530)
- anything odd: the walks' paths are the original's choice among shortest paths (M2 along row 5, not row 6 as our engine's; Q53's pathfinder): no moat hex on either. The Archangel had a good-morale turn after its strike (defended). At every round's start the keep's arrow tower shot first (round 1 the ten Peasants, 10; rounds 2 and 3 the Sprites, 15 each), then the Catapult (it destroyed the Upper Wall in round 1 and hit the Mid-Lower and Lower Walls later).

## G — The gate of a Castle with a Fort; shots through the walls and the open gate (BAT-SIE-002, BAT-SIE-003, BAT-SIE-008, D-8)

- Red hero **G** (Jenova, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 20 Sprite, 1 Titan, 10 Pixie
- Opponent: Blue Castle town (Village Hall, fort); garrison 5 Troll, 5 Minotaur King, 10 Peasant

1. Round 1: the Titan (column 1, row 5, hex 86) shoots the Minotaur Kings (hex 100, inside, behind the closed gate). The Sprites wait. The Minotaur Kings walk onto the gate's hex 96. The Trolls (hex 49) walk to hex 97, inside right of the gate.
2. Round 2: the Titan shoots the Trolls on 97 through the open gate. The Minotaur Kings walk out to hex 76 (row 4, column 8, outside).
3. Round 3: the Sprites fly to the outer bridge, hex 94 (row 5, column 9). The Trolls try to walk onto the gate's hex 96. Then end the battle as you like.

**Record**

- G1: the Minotaur Kings' lost hit points from the Titan's shot: 17 (250 -> 233)
- G2: the gate after the Minotaur Kings' walk onto 96 (open or closed): open (3 -> 2 -> 1 while they stood on 97, about to step onto 96; still 1 after the walk)
- G3: the Trolls' lost hit points from the Titan's shot: 46 (200 -> 154; the Minotaur Kings on 96, the gate open)
- G4: the gate after the Minotaur Kings' walk out to 76: closed (their walk 96 -> 95 -> 94 -> 76; 1 -> 2 -> 3 after it ended)
- G5: where the Sprites land: 94
- G6: could the Trolls walk onto 96?: no (the click started nothing; they stayed on 97 and defended)
- anything odd: the gate opened while the Minotaur Kings stood on 97, as they began the step onto 96 (the engine reports it after the step, BAT-SIE-003's event order; the same state). G4's walk passed 94, our engine's 77 (a tie, Q53). The Catapult destroyed the Upper Wall (round 2) and the Lower Wall (round 3), never the gate (2 hit points throughout). The Trolls regenerated 6 at their turn in round 2.

## T — The Tower's land mines (a Citadel, no defending hero) (BAT-SIE-007, BAT-MAG-033, D-9)

- Red hero **T** (Rissa, specialty resource mercury): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 3 Titan
- Opponent: Blue Tower town (Village Hall, fort, citadel); garrison 10 Gremlin

1. At the battle's start, note whether any mine shows (the attacker's view).
2. Round 1: the Titans (column 1, row 5, hex 86) walk to hex 77 (row 4, column 9, where a moat would be).
3. Round 2: the Titans walk up to hex 28 (row 1, column 11): the only shortest way passes 61 and 44. Then end the battle as you like.

**Record**

- T1: mines shown at the start (where, if any): none shown: ten Land Mine objects of the defender's (owner 1) on 11, 28, 44, 61, 77, 111, 129, 146, 164, 181, each hidden (+0xA 0), worth 150, lasting the battle (rounds 0); the hexes' Land Mine bit on those ten; no moat flag (+0x53A8 0)
- T2: the Titans' lost hit points as they walk onto 77; where they stand: 150 (884 -> 734), on 77; nine mines left (their walk 86 -> 69 -> 70 … -> 77)
- T3: the Titans' lost hit points on the walk to 28; where they stand: 450 (715 -> 265: 150 on 61, 44 and 28 each, the walk going on), on 28; six mines left
- anything odd: the keep shot the Titans at both rounds' start (16, 19). Bonus artifacts in attempts 1 to 4 (37, 75, 50, 103): played again.
