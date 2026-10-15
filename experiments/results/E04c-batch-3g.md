# E04c batch 3g — results

Map file: `E04c-batch-3g.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `83cb4073`.** The map's description in the scenario list starts with "Stamp 83cb4073.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Run by an agent** (E04c T8, 2026-10-15) on the client lane's capture harness, as batches 3e and
3f: the same Complete 4.0 `Heroes3.exe`, under Wine in the cloud sandbox, without the HD mod; hot
seat (Red and Blue human), one map per encounter (`h3-experiment … --only <id>`), played by
`experiments/agent/town.py`: the clock stepped 10 ms at a time and the game's memory read after
each step; the buttons are the battle window's items 2001 (Surrender) and 2002 (Retreat), their
word +0x16 (0x0006 enabled, 0x000E with the bit 0x8 disabled). A Teleport or Force Field target
counts as refused when a click starts nothing within 1.5 s of game time. Attempts whose heroes
drew a starting-bonus artifact were played again. A run took 17 to 86 s of real time.

1. Copy `E04c-batch-3g.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps` and start it in hot seat (Red and Blue human). Save right away.
2. Each Red hero stands south of a Blue town's entrance. Play the encounters in any order: select the hero and step north into the town; command both armies as the encounter says (every stack without a step defends).
3. Then end the battle as you like and load the save for the next encounter.
4. Fill in this file (anything unexpected goes into the last line of an encounter) and keep it in the repo.

## CM — A Castle with a Tavern and the Brotherhood of the Sword: the defender's buttons and morale (BAT-SIE-024, BAT-SIE-025)

- Red hero **CM** (Caitlin, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 3 Titan
- Opponent: Blue Castle town (Village Hall, fort, citadel, tavern, brotherhood of the sword); garrison 10 Pikeman, 10 Gnoll, 10 Goblin; visiting hero: Blue hero **CM** (Gelare, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman, 10 Gnoll, 10 Goblin

1. Every stack defends.
2. At the first turn of each side, note whether the Surrender and Retreat buttons are enabled.
3. Note every stack's morale at the opening. Play to round 2, then end the battle as you like.

**Record**

- CM1: the buttons at the attacker's and the defender's turn: both enabled at the attacker's turns (the buttons' word +0x16 = 0x0006, rounds 1 and 2); **both disabled at every defender's turn** (0x000E: the bit 0x8 that 0x4782D0's state 0x1000 sets), as the code.
- CM2: every stack's morale at the opening: the Titans +1. The defender's Pikemen, Gnolls and Goblins (each 20: the garrison joined the visiting hero's stacks) **+1 each, not +2**: three alignments (−1) and the town's **+2**. Read back from the town record (a probe): the built set +0x150 holds the Citadel (8), the Village Hall (10) and the Brotherhood (22) but neither the Fort (7) nor the Tavern (5): an upgrade replaces its predecessor there, and the Brotherhood is the Tavern's; the second set +0x158 holds all five. The Tavern's +1 reads +0x150, so with the Brotherhood a Castle gives +2 in all (D-21). The hero has no skill, no morale bonus (+0x11A 0) and only the Catapult.
- anything odd: run twice (the first run's Blue hero drew starting-bonus artifacts in five attempts; a later run): the same values.

## ET — A Stronghold with the Escape Tunnel: the defender may retreat but not surrender (BAT-SIE-024)

- Red hero **ET** (Jenova, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 3 Titan
- Opponent: Blue Stronghold town (Village Hall, fort, escape tunnel); garrison 10 Goblin; visiting hero: Blue hero **ET** (Grindan, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Goblin

1. Every stack defends.
2. At the first turn of each side, note whether the Surrender and Retreat buttons are enabled. Then end the battle as you like.

**Record**

- ET1: the buttons at the attacker's and the defender's turn: both enabled at the attacker's turn; at the defender's turn **Retreat enabled (0x0006), Surrender disabled (0x000E)**, as the code (the Escape Tunnel, building 17).
- anything odd: none.

## RL — A Rampart with the Fountain of Fortune: the defender's luck (BAT-SIE-025)

- Red hero **RL** (Rissa, specialty resource mercury): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 3 Titan
- Opponent: Blue Rampart town (Village Hall, fort, fountain of fortune); garrison 10 Centaur; visiting hero: Blue hero **RL** (Sylvia, specialty skill Navigation): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Centaur

1. Every stack defends.
2. Note every stack's luck at the opening, then end the battle as you like.

**Record**

- RL1: every stack's morale and luck at the opening: the Titans morale +1, luck 0; the defender's Centaurs (20) **morale +1, luck +2** (the Fountain of Fortune), as the code; no Tavern, so no more morale.
- anything odd: at the defender's turn both buttons disabled (a Rampart's defender: no tunnel), as the code.

## TB — Basic Teleport at a Citadel: neither across the moat nor through the walls (BAT-SIE-026, D-20)

- Red hero **TB** (Octavia, specialty resource gold): attack 0, defense 0, power 1, knowledge 3; Basic Water Magic; Spell Book (spell book); spells Teleport; army 5 Swordsman
- Opponent: Blue Castle town (Village Hall, fort, citadel); garrison 10 Peasant

1. Round 1, at the Swordsmen's turn: the Red hero casts Teleport on them and tries, in this order, hex 64 (inside), hex 61 (the moat), hex 91 (outside), until one is accepted.
2. Every other turn defends. Play to round 2, then end the battle as you like.

**Record**

- TB1: each destination tried: refused or accepted; where the Swordsmen stood afterwards: **64 refused, 61 refused, 91 accepted**: the Swordsmen moved from 86 to 91, as the code (the moat hex 61 counts as inside: VCMI, which tests only across the walls, would accept it; D-20).
- anything odd: the defender has no hero: at its turns both buttons disabled (retreat needs a hero); at the attacker's turn Surrender disabled (no enemy hero) and Retreat enabled, as the code. The keep shot the Swordsmen first (175 → 162).

## TV — Advanced Teleport at a Citadel: across the moat, not through the walls (BAT-SIE-026, D-20, BAT-MAG-029)

- Red hero **TV** (Aine, specialty resource gold): attack 0, defense 0, power 1, knowledge 3; Advanced Water Magic; Spell Book (spell book); spells Teleport; army 5 Swordsman
- Opponent: Blue Castle town (Village Hall, fort, citadel); garrison 10 Peasant

1. Round 1, at the Swordsmen's turn: the Red hero casts Teleport on them and tries, in this order, hex 64 (inside), hex 61 (the moat), hex 91 (outside), until one is accepted.
2. Every other turn defends. Play to round 2, then end the battle as you like.

**Record**

- TV1: each destination tried: refused or accepted; the Swordsmen's hit points on landing and after their turns: **64 refused, 61 accepted**: the Swordsmen landed on the moat hex 61 with **no damage** (160 → 160: a teleport enters no hex), then lost the moat's 70 when they defended there (160 → 90) and again in round 2 (75 → 5, after the keep's 15), as the code.
- anything odd: the first run timed out in round 3 (the Swordsmen died in the moat: the battle ended); its timeline gave the same values; the script now ends a battle that ends early.

## TE — Expert Teleport at a Citadel: through the walls (BAT-SIE-026)

- Red hero **TE** (Calid, specialty resource sulfur): attack 0, defense 0, power 1, knowledge 3; Expert Water Magic; Spell Book (spell book); spells Teleport; army 5 Swordsman
- Opponent: Blue Castle town (Village Hall, fort, citadel); garrison 10 Peasant

1. Round 1, at the Swordsmen's turn: the Red hero casts Teleport on them and tries, in this order, hex 64 (inside), hex 61 (the moat), hex 91 (outside), until one is accepted.
2. Every other turn defends. Play to round 2, then end the battle as you like.

**Record**

- TE1: each destination tried: refused or accepted; where the Swordsmen stood afterwards: **64 accepted**: the Swordsmen stand inside the walls (86 → 64), as the code.
- anything odd: two attempts drew starting-bonus artifacts (51, 45) and were played again.

## FF — Force Field beside the walls: never on a wall's hex or the gate's (BAT-SIE-027, BAT-MAG-027)

- Red hero **FF** (Clavius, specialty resource gold): attack 0, defense 0, power 1, knowledge 3; Expert Earth Magic; Spell Book (spell book); spells Force Field; army 5 Swordsman
- Opponent: Blue Castle town (Village Hall, fort); garrison 10 Peasant

1. Round 1, at the Swordsmen's turn: the Red hero casts Force Field and tries, in this order, hex 95 (its shape reaches the wall hex 78), hex 96 (the gate), hex 91 (outside), until one is accepted.
2. Every other turn defends. Then end the battle as you like.

**Record**

- FF1: each target tried: refused or accepted; the Force Field's hexes: **95 refused, 96 refused, 91 accepted**: one Force Field object anchored at 91 for 2 rounds (the battle's objects 0 → 1), as the code: 95's shape reaches the standing wall 78, 96 is the gate.
- anything odd: none.
