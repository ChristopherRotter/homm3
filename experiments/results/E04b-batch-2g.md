# E04b batch 2g — results

**Played by an agent** (2026-10-24) on the capture harness (`experiments/agent/ai.py`, recipe `experiments/agent/E04b-batch-2g.toml`): the Complete 4.0 `Heroes3.exe` under Wine without the HD mod, one map per run (`--only`), hot seat on Normal, good and bad morale forced to fail; each run played again until neither hero wore a starting bonus's artifact (FS: the sixth attempt). Every value below was read from the game's memory (`timeline.jsonl`, `prices.jsonl`), the prices from the surrender dialog's numbers.

Map file: `E04b-batch-2g.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `94220a7b`.** The map's description in the scenario list starts with "Stamp 94220a7b.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

1. Played by an agent (experiments/agent/ai.py with experiments/agent/E04b-batch-2g.toml): one map per encounter, `h3-experiment E04b-batch-2g.toml --only <id>`. By hand: copy the map into `Maps`, start it in hot seat (Red and Blue human), select the Red hero and step east into the Blue hero; you command both sides.
2. Every stack defends unless the procedure says otherwise; a stack that gets an extra turn from good morale defends.
3. "Up-right", "up-left", "down-left", "down-right" mean the neighbouring hex in that direction; rows are counted from the top (row 0) to the bottom (row 10), columns from the left (0) to the right (16).
4. Fill in this file (anything unexpected goes into the last line of an encounter) and keep it in the repo.

## BR — A Red Dragon's breath behind a stack in column 15 with Blue's Ballista behind it (BAT-Q62, BAT-ATK-003)

- Red hero **BR** (Aine, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Red Dragon
- Opponent: Blue hero **BRb** (Octavia, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; Ballista (war machine 1); no spells; army 1 Peasant, 30 Pikeman, 1 Peasant, 1 Peasant

1. Attack the Blue hero. Blue's stacks stand in column 15 on rows 0, 4, 6 and 10: the Pikemen on (15, 4); Blue's Ballista on (15, 3) and (16, 3).
2. Round 1: the Red Dragon flies to the middle (its head on (8, 5)). Blue's stacks Defend; the Ballista shoots on its own.
3. Round 2: the Red Dragon attacks the Pikemen from (15, 5), its head there and its tail on (14, 5) (the breath goes on up-right, onto (16, 3)). Note the Pikemen's losses and the Ballista's health before and after the attack.
4. Retreat with Red.
5. The control (the run BC, on this map): in round 1 the Pikemen also walk one hex left, onto (14, 4); in round 2 the Red Dragon attacks them from (14, 5), its head there: the breath goes on up-right onto (15, 3), the Ballista's front hex.

**Record**

- the Pikemen's count before and after the Red Dragon's attack; the Ballista's health before and after; the same in BC: BR: 30 to 24 (the dragon on 99, its head on (15, 5)); the Ballista 250 before and after (the breath stopped at the side column); BC: the Pikemen walked to (14, 3), the dragon attacked from their left (its head on (13, 3), position 63): 30 to 23, the Ballista 250 to 187 (the breath onto its front hex (15, 3))
- anything odd: BC took three tries: attacked from below, the original put the dragon's tail on the clicked hex (14, 5) and its head on (15, 5), so the breath went up-left, off the Ballista (twice, once after its head stood on (14, 5)); the recipe now attacks from the left. A two-hex attacker takes the clicked neighbour as its rear hex when its head then has room, else as its head

## SP — The surrender price with Basic Resurrection's temporary creatures (BAT-Q28, BAT-FLE-002)

- Red hero **SP** (Gelare, specialty resource gold): attack 0, defense 0, power 1, knowledge 2; Basic Earth Magic; Spell Book (spell book); spells Resurrection; army 10 Swordsman
- Opponent: Blue hero **SPb** (Grindan, specialty resource gold): attack 0, defense 0, power 7, knowledge 2; no skills; Spell Book (spell book); spells Lightning Bolt; army 10 Peasant

1. Attack the Blue hero. Round 1: the Swordsmen Defend. Blue's turn: Blue casts Lightning Bolt at the Swordsmen; the Peasants Defend.
2. Round 2, the Swordsmen's turn: open the surrender dialog and note its price; cancel. Red casts Resurrection on the Swordsmen; the surrender price again (cancel). The Swordsmen Defend. Blue's turn: Lightning Bolt at the Swordsmen again; the Peasants Defend.
3. Round 3, the Swordsmen's turn: the surrender price once more (cancel). Retreat with Red.

**Record**

- the Swordsmen after each Lightning Bolt and after Resurrection (number, top health): (5, 25); 8 (255 health in all); 2 (70): as batch 2d RA2, the user's (the harness's cross-check)
- the surrender price before Resurrection, after it, in round 3: 750; 750; 0
- anything odd: played on the map before BC's encounter left the spec (its place in the grid then differed; nothing in the row depends on the field)

## FS — A striker killed by the Efreet Sultans' Fire Shield, then a second attacker (BAT-Q28, S-23, BAT-RET-001, BAT-MAG-058)

- Red hero **FS** (Caitlin, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Archangel, 10 Royal Griffin
- Opponent: Blue hero **FSb** (Jenova, specialty resource gold): attack 0, defense 0, power 9, knowledge 2; Basic Air Magic; Spell Book (spell book); spells Lightning Bolt; army 20 Efreet Sultan

1. Attack the Blue hero. Round 1: the Archangel Defends. The Sultans' turn: Blue casts Lightning Bolt at the Archangel (245: 5 health left); the Sultans Defend. The Royal Griffins fly to the middle.
2. Round 2: the Archangel attacks the Sultans from their left; the Fire Shield kills it. Do the Sultans strike back at it? The Sultans Defend. The Royal Griffins attack the Sultans from up-left: are they struck back?
3. Retreat with Red.

**Record**

- the Archangel's health after the Lightning Bolt; its attack: damage, reflection, death; the Sultans' retaliation at it (yes/no): 15 (the bolt 235: Basic Air's effect is 10, as the pack's, not the spec's 20); 87 to the Sultans (1 800 to 1 713), the reflection killed it (15 to 0, dead); yes: the Sultans' retaliations left (+0x454) fell from 1 to 0 in the Archangel's action, after its death
- the Royal Griffins' attack: struck back (yes/no): no: 34 to the Sultans (one killed), the Griffins lost 8 (250 to 242, the reflection alone), the Sultans' retaliations still 0 until round 3
- anything odd: nothing

## RS — Resurrection at a two-hex corpse's front hex with a later corpse on its rear hex (BAT-Q28, S-24, BAT-MAG-035)

- Red hero **RS** (Dessa, specialty skill Logistics): attack 0, defense 0, power 5, knowledge 3; no skills; Spell Book (spell book); spells Resurrection; army 2 Wolf Rider, 1 Swordsman, 5 Archer
- Opponent: Blue hero **RSb** (Gunnar, specialty skill Logistics): attack 0, defense 0, power 10, knowledge 3; no skills; Spell Book (spell book); spells Lightning Bolt; army 1 Peasant

1. Attack the Blue hero. Red's Wolf Riders stand on row 2 (their rear hex (1, 2), their head (2, 2)), the Swordsman on (1, 5), the Archers on (1, 8).
2. Round 1: Red's stacks Defend. Blue's turn: Lightning Bolt at the Wolf Riders (they die); the Peasant Defends.
3. Round 2: the Swordsman walks onto the Wolf Riders' rear hex (1, 2). Blue's turn: Lightning Bolt at the Swordsman (it dies there); the Peasant Defends.
4. Round 3, the Archers' turn: Red casts Resurrection at the Wolf Riders' head (2, 2). Which stack comes back, and how many? The Archers Defend. Retreat with Red.

**Record**

- Resurrection at the Wolf Riders' head raises (Wolf Riders/Swordsman), how many: the Swordsman, 1 (cast at hex 36, the head; the Wolf Riders' record on 35 stayed dead); it died on 35 in round 2 after walking 86, 69, 53, 35
- anything odd: nothing
