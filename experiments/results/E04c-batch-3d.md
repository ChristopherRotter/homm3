# E04c batch 3d — results

Map file: `E04c-batch-3d.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `0ee2dfc1`.** The map's description in the scenario list starts with "Stamp 0ee2dfc1.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Run by an agent** (E04c T6d, 2026-10-14) on the client lane's capture harness: the same Complete
4.0 `Heroes3.exe`, under Wine in the cloud sandbox, without the HD mod; hot seat (Red and Blue
human, the agent commanding both armies and, in B, aiming the Catapult), one map per encounter
(`h3-experiment … --only <id>`), played by `experiments/agent/catapult.py` (built on `siege.py`):
the clock stepped 10 ms at a time and the game's memory read after each step (the 18 wall parts'
hit points +0x13F60, target t being part 5, 6, 8, 9, 10, 12, 13, 14; the gate +0x53A4; each
stack's hex, count, lost hit points and flags +0x84). Each change of the parts' hit points is one
shot or one cast; the aim of an automatic shot is not in memory, so A's landings are checked
against the code's possible aims and miss landings. Every encounter was played twice (A and E
first from a build of the spec whose encounter B differed): the game's draws repeated exactly, so
the two runs agree in every value. B's earlier build (Basic Ballistics, aiming at the gate first)
gave: the gate destroyed by the first shot (2), then aimed at 1: 1, 1 (destroyed); at 5: 1, a miss
on 4 (1), 1 (destroyed). Bonus artifacts: Blue's hero in E's first run wore the Hourglass of the
Evil Hour (luck only); B's first attempt wore a bonus artifact and was played again.

1. Copy `E04c-batch-3d.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps` and start it in hot seat (Red and Blue human). Save right away.
2. Each Red hero stands south of a Blue town's entrance. Play the encounters in any order: select the hero and step north into the town; command both armies as the encounter says (every stack without a step defends).
3. Note the walls' state after every shot of the Catapult and every Earthquake (which part lost how much). Then end the battle as you like and load the save for the next encounter.
4. Fill in this file (anything unexpected goes into the last line of an encounter) and keep it in the repo.

## A — The automatic Catapult at a Castle with a Citadel; a destroyed wall and a destroyed gate (BAT-SIE-010, BAT-SIE-011, BAT-SIE-012, BAT-SIE-014, D-11, D-12, D-13)

- Red hero **A** (Caitlin, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Titan
- Opponent: Blue Castle town (Village Hall, fort, citadel); garrison 10 Peasant, 5 Centaur Captain, 5 Centaur Captain

1. Every stack defends unless a step below says otherwise; the Catapult shoots on its own (the hero has no Ballistics), one shot each round. Note each shot: the part it damaged and by how much.
2. Round 1: the Centaur Captains in slot 2 walk to hex 97 (row 5, column 12, inside right of the gate).
3. Once the mid-upper wall (the wall over the gate) is destroyed: the Centaur Captains in slot 1 try to walk onto hex 62 (row 3, column 11), then onto hex 78 (row 4, column 10).
4. Once the gate is destroyed: the Centaur Captains on 97 walk out to hex 94 (row 5, column 9) over the gate's hex 96 and the drawbridge's 95.
5. Play on to round 16, then end the battle as you like.

**Record**

- A1: each shot of the Catapult: the round, the part it damaged (target index) and the damage: round 1: 2, 2 (destroyed; every wall at 2, so any wall was the aim); 2: 5, 2 (destroyed); 3: 4, 1; 4: 1, 2 (destroyed; the weakest wall was 4 at 1 hit point: a miss, landing on the only other standing wall); 5: 4, 1 (destroyed); 6: no part changed (a 0-damage shot); 7: the gate (3), 2 (destroyed; every wall down); 8: the keep (7), 1; 9: the keep, 1 (destroyed; its cover, part 15, 1 -> 0, and its arrow tower's stack marked dead); rounds 10 to 16: no shot (nothing standing; the towers are absent at a Citadel). Every shot landed on the code's aim (the weakest standing wall, then the gate, then the keep) or on the miss list's nearest other wall to it.
- A2: could the Centaur Captains walk onto 62, onto 78, once the mid-upper wall was destroyed?: 62 no (round 1, the click refused: nothing started); 78 yes (round 2: from 100 to position 79, the two-hex Centaur Captains' rear hex, their head on 78).
- A3: the gate's state once destroyed; where the walk out from 97 ended and its lost hit points: the gate 3 -> 0 at round 7's shot and 0 to the end; the walk (round 7, the stack at position 98, head on 97) ended at position 95 with its head on 94, over 96 and 95 without a stop and without damage (50 hit points before and after; the moat does 70); the gate stayed 0.
- anything odd: the Centaur Captains are two-hex: their recorded position (+0x38) is the rear hex, one right of the hex clicked for their head (97 -> 98, 78 -> 79, 94 -> 95).

## B — The Catapult aimed under Advanced Ballistics at a Castle with a Fort: where the misses land (BAT-SIE-011, BAT-SIE-014, HER-BAT-018, D-12)

- Red hero **B** (Jenova, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; Advanced Ballistics; no artifacts; no spells; army 2 Titan
- Opponent: Blue Castle town (Village Hall, fort); garrison 10 Peasant

1. Every stack defends. At each of its turns the Catapult (Advanced Ballistics: the player aims; two shots) aims at the upper wall (hex 29) while it stands, then at the lower wall (hex 182), the gate (96), the mid-upper wall (its aim hex 62) and the mid-lower wall (130). Note each shot: the target aimed at, the part it damaged and by how much.
2. Play on to round 10, then end the battle as you like.

**Record**

- B1: each shot of the Catapult: the round, the target aimed at, the part it damaged and the damage: round 1: aimed at 1: 1, 2 (destroyed), then 2, 1 (the second shot's aim moved to 2, the nearest standing target: the upper tower is absent at a Fort); round 2: aimed at 5: 5, 1, then 4, 1 (a miss onto the nearest other wall); round 3: aimed at 5: 5, 1 (destroyed), then 2, 1 (destroyed; the aim moved to 4, the nearest standing target, and the shot missed onto 2, the only other standing wall); round 4: aimed at the gate (3): 3, 2 (destroyed, the gate 3 -> 0), then 4, 1 (destroyed; the aim moved to 4); rounds 5 to 10: no target standing, the Catapult's turns passed.
- anything odd: under Ballistics the game waits for the player's aim at the Catapult's turn (a click on a standing target's aim hex starts the shots); Defend does nothing there.

## E — Earthquake by both sides at a Castle with a Citadel (BAT-SIE-015, D-14, D-15, S-08)

- Red hero **E** (Rissa, specialty resource mercury): attack 0, defense 0, power 1, knowledge 5; Expert Earth Magic; Spell Book (spell book); spells Earthquake; army 2 Titan
- Opponent: Blue Castle town (Village Hall, fort, citadel); garrison 10 Peasant, 10 Pikeman; visiting hero: Blue hero **E** (Octavia, specialty resource gold): attack 0, defense 0, power 1, knowledge 5; Expert Earth Magic; Spell Book (spell book); spells Earthquake; army 10 Peasant, 10 Pikeman

1. Every stack defends unless a step below says otherwise.
2. Round 1: at the attacker's first stack's turn, the Red hero casts Earthquake (Expert Earth Magic: four hits). At the defender's first stack's turn, the Blue hero casts Earthquake too.
3. Round 2: the Red hero casts Earthquake again; so does the Blue hero. Note after each cast which parts lost hit points, and whether a defending stack vanished.
4. Round 3: end the battle as you like.

**Record**

- E1: after each cast (side, round): which parts lost how many hit points: the attacker, round 1: the upper tower (0, absent) 1, the mid-lower wall (4) 2 (destroyed), the lower tower (6, absent) 1; the defender, round 1: the upper tower 1 (0 left), the gate (3) 1, the lower tower 1 (0 left), the keep (7) 1, and the parts 16 and 17 (the towers' covers) 1 -> 0; the attacker, round 2: the upper wall (1) 1 (destroyed), the gate 1 (destroyed: 3 -> 0), the lower wall (5) 1: three hit points for four hits (the upper wall and the gate had 1 each). The Catapult (no Ballistics) shot too: round 1 the upper wall 1, round 2 the mid-upper wall 2 (destroyed; the weakest wall was the upper wall at 1: a miss onto its nearest), round 3 the lower wall 1.
- E2: did a defending stack vanish (which, after which cast)?: yes: after the defender's own cast in round 1, which brought both absent towers to 0, the defender's first stack (20 Peasants, its count unchanged) carries the dead flag 0x200000 and never acts again (the defender's hero had no stack left to cast with in round 2).
- anything odd: the visiting hero's army and the garrison joined: the defender had 20 Peasants and 20 Pikemen (and the keep).
