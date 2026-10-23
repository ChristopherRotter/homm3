# E23 batch 4a — results

Map file: `E23-batch-4a.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `5d7e1701`.** The map's description in the scenario list starts with "Stamp 5d7e1701.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Run by an agent** (E23 T7, 2026-10-23) on the client lane's capture harness: the same Complete 4.0 `Heroes3.exe` under Wine in the cloud sandbox, without the HD mod; hot seat (Red and Blue human), one map per encounter (`h3-experiment … --only <id>`, compiled when each run started), played by `experiments/agent/ai.py` with the recipe `experiments/agent/E23-batch-4a.toml` on the stepped clock; every value below is read from the game's memory (`ai_turns.py <run> --actions`). Maps: LO was played on the spec's first version (stamp 0a2cf351: the batch's other piles not yet on it), S4 on stamp 8027ee14, W1, C1 to C3 on 57e172cf, C4 on Easy on this build; each encounter's own map is this build's but for the stamp (and LO's missing piles, far from its battle). C4's first runs (on Normal, stamp 57e172cf) had Red's Archangels alone; this build adds Red's Pikemen and plays it on Easy. A random starting bonus that armed Red's hero was played again.

1. Played by an agent (experiments/agent/ai.py): one map per encounter, `h3-experiment E23-batch-4a.toml --only <id>`. By hand: copy the map into `Maps`, start it in hot seat (Red and Blue human) at the difficulty the run names, select the Red hero and step east into the guards (north into a town).
2. Every Red stack defends unless the procedure says otherwise. Note what the encounter asks, then end the battle (retreat) as the procedure says.
3. Fill in this file (anything unexpected goes into the last line of an encounter) and keep it in the repo.

## LO — S-25: a neutral stack frozen by bad morale is let off one time in four (BAT-Q33, S-25, BAT-MOR-004)

- Red hero **LO** (Clavius, specialty resource gold): attack 0, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 20 Bone Dragon, 100 Pikeman
- Blue hero **Nagash** (Nagash, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Red's Bone Dragons defend every turn; Red's Pikemen freeze (forced).
2. At the opening the harness forces every bad-morale draw to fall (`rolls bad_morale 1`) and every good-morale draw to fail: each stack with morale below 0 is frozen at every pick, unless a second draw lets it off.
3. Count, for each neutral pick after the battle's first, whether the stack froze (the bad-morale animation 30 over it) or took its turn; the same for Red's Pikemen. After round 15 Red retreats.

**Record**

- LO: the stacks' morale at the opening (Red's and the neutrals'): Red's Bone Dragons 0 (no morale), Pikemen -1; the seven neutral stacks -6 (seven towns and the Bone Dragons; the check clamps it to -3); setup bytes +0x54A4 = [1, 0].
- LO: neutral picks after the first: frozen, let off (took their turn): 77 frozen (animation 30), 28 let off (26.7 %), 15 rounds; every let-off took its turn (a shot).
- LO: Red's Pikemen's picks: frozen, took their turn: 15 frozen, 0 took their turn.
- anything odd: none; Red retreated after round 15 (the neutral shooters' let-off turns shot Red's Pikemen).

## W1 — W1: a walker that chooses nothing defends on Normal, waits on Hard (E23 survey row 4a W1)

- Red hero **W1** (Caitlin, specialty resource gold): attack 0, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Champion, 30 Dwarf
- Opponent: Neutral Castle town (Village Hall, fort, citadel); garrison 20 Swordsman
- Blue hero **Sylvia** (Sylvia, specialty skill Navigation): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Step north into the neutral Citadel. Red's Champions and Dwarves defend (no shooter, no spell book): Red is the weaker at range, the keep's turret makes the garrison the stronger, so its Swordsmen hold the walls with no enemy inside them. Red's Catapult shoots on its own.
2. Played twice: on Normal and on Hard. At each turn of Red's Dwarves (speed 3, after the Swordsmen's first turn and before the wait phase; ten Peasants died to the keep's first arrow in the first runs) note whether the Swordsmen defend or wait (their flags), and the log's lines; while every wall stands. Red retreats after round 4.

**Record**

- W1: the Swordsmen's actions in rounds 1 and 2, on Normal: Rounds 1 to 4: at Red's Dwarves' turn the Swordsmen show acted and defending (Defend) every round, at hex 100; walls at that time: the lower wall 1 then 0, the mid-upper 1 then 0 by round 4 (Red's Catapult), never a counted breach.
- W1: the Swordsmen's actions in rounds 1 and 2, on Hard: Rounds 1 to 4: at Red's Dwarves' turn the Swordsmen show waited every round (Wait, then Defend in the wait phase), at hex 100; the same walls as on Normal.
- anything odd: The first runs (Red's Champions and 10 Peasants: the keep's first arrow killed the Peasants) held four rounds behind standing walls on both difficulties and sallied right after the mid-lower wall fell (1 to 0 hit points), as S4's control.

## S4 — S4: a garrison holds the walls behind a destroyed mid-upper wall, not behind the mid-lower (E23 survey row 4a S4, slip 2)

- Red hero **S4** (Jenova, specialty resource gold): attack 0, defense 10, power 1, knowledge 1; Basic Ballistics; no artifacts; no spells; army 10 Champion
- Opponent: Neutral Castle town (Village Hall, fort, citadel); garrison 20 Swordsman
- Blue hero **Piquedram** (Piquedram, specialty creature Stone Gargoyle): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Step north into the neutral Citadel. Red's hero has Basic Ballistics (one aimed shot a turn; Expert's second shot broke a second wall in the first runs): the agent aims the Catapult at one wall until it falls (the mid-upper in one run, the mid-lower in the other); Red's Champions defend.
2. Note the Swordsmen's action at each turn and the walls' hit points before it; after round 6 Red retreats.

**Record**

- S4: the mid-upper wall's run: the round it fell; the Swordsmen's actions before and after: Three runs (seeds 0, 3000, 6000 ms): the mid-upper wall fell in round 1, 2 and 2 (the upper wall at 1 in two); the Swordsmen defended at hex 100 before and after, to round 9 in every run: they hold.
- S4: the mid-lower wall's run: the round it fell; the Swordsmen's actions before and after: Round 1 (the only wall down): the Swordsmen moved out at their round-1 turn (hex 100 to 94 through the gate) and attacked Red's Champions in round 2: they sally.
- anything odd: The first runs with Expert Ballistics: its second shot broke a second wall (mid-upper and mid-lower; mid-lower and lower): the garrison sallied in both, as the mid-lower breach predicts; replayed with Basic Ballistics.

## C1 — C1: neutral Storm, Ice, Energy and Magma Elementals never cast their protection (E23 survey row 4a C1, slip 25, BAT-Q57)

- Red hero **C1** (Rissa, specialty resource mercury): attack 0, defense 5, power 5, knowledge 10; no skills; Spell Book (spell book); spells Lightning Bolt, Fireball, Ice Bolt, Implosion; army 30 Archer, 50 Pikeman
- Blue hero **Sanya** (Sanya, specialty skill Eagle Eye): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Step east into the guards. Red's hero has Lightning Bolt, Fireball, Ice Bolt and Implosion and the mana for them (never cast: Red defends); Red's Archers shoot nothing.
2. Note every creature cast (action 10) of the four elementals over 5 rounds, then Red retreats.

**Record**

- C1: the elementals' turns and actions over 5 rounds (casts?): No creature cast (action 10) and no spell on any stack in 5 rounds; the Storm and Ice Elementals shot, the Energy Elementals flew and attacked, the Magma Elementals walked and attacked.
- anything odd: none

## C2 — C2: neutral Faerie Dragons cast every damage spell they draw, an area spell at a free hex (E23 survey row 4a C2, BAT-Q32, BAT-STUB-8)

- Red hero **C2** (Aine, specialty resource gold): attack 0, defense 5, power 1, knowledge 1; no skills; no artifacts; no spells; army 50 Pikeman, 50 Pikeman, 50 Pikeman, 50 Pikeman
- Blue hero **Kyrre** (Kyrre, specialty skill Logistics): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Step east into the guards (three stacks of two Faerie Dragons). Red's four stacks of 50 Pikemen stand on rows 0, 4, 6 and 10 (hexes 1, 69, 103, 171) and defend.
2. Note each dragon's drawn spell (+0x4E0) and its action at each turn (a cast, its hex) over 4 rounds, then Red retreats.

**Record**

- C2: per dragon turn: the drawn spell, the action and its hex: Round 1: Fireball, cast at 86; Fireball, at 86; Ice Bolt, at 1; Lightning Bolt (good morale's turn), at 1. Round 2: Frost Ring, at 86; Fireball, at 86 (both stacks at 69 and 103 die); Lightning Bolt, at 171. Round 3: Lightning Bolt, at 171; Ice Bolt, at 1 (Red's last stack). A cast at 8 of 8 turns.
- anything odd: Every bolt went to the first stack of the best kill value (hex 1 over 171 while both were alike, 171 once hex 1 held one Pikeman).

## C3a — C3a: neutral Archangels with ten dead Peasants attack rather than raise them (E23 survey row 4a C3)

- Red hero **C3a** (Octavia, specialty resource gold): attack 0, defense 5, power 1, knowledge 2; no skills; Spell Book (spell book); spells Lightning Bolt; army 30 Marksman, 100 Pikeman
- Blue hero **Malcom** (Malcom, specialty skill Eagle Eye): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Step east into the guards (2 Archangels, 10 Peasants). Round 1, Red's first turn: the hero casts Lightning Bolt (its book's only spell) at the Peasants; every Red stack defends.
2. Note the Archangels' actions in rounds 1 to 3 (a cast at the Peasants' corpses, or an attack), then Red retreats.

**Record**

- C3a: the Archangels' actions in rounds 1 to 3: Round 1: move and attack on the Marksmen (30 to 9); Red's Lightning Bolt killed the 10 Peasants; round 2: attack on the Pikemen; round 3: attack on the Marksmen; round 4: attack on the Pikemen. Never a raise.
- anything odd: none

## C3b — C3b: neutral Archangels raise two dead Champions rather than attack weak Peasants (E23 survey row 4a C3)

- Red hero **C3b** (Calid, specialty resource sulfur): attack 0, defense 5, power 5, knowledge 3; no skills; Spell Book (spell book); spells Implosion; army 300 Peasant
- Blue hero **Edric** (Edric, specialty creature Griffin): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Step east into the guards (2 Archangels, 3 Champions). Round 1, Red's Peasants' turn: the hero casts Implosion (its book's only spell) at the Champions; the Peasants defend.
2. Note the Archangels' actions in rounds 1 to 3, then Red retreats.

**Record**

- C3b: the Archangels' actions in rounds 1 to 3: Round 1: move and attack on the Peasants (300 to 75); Red's Implosion killed the 3 Champions at 141; round 2: creature cast at 141, 2 Champions raised; round 3: attack on the Peasants.
- anything odd: none

## C4 — C4: neutral Ogre Mages with a walker's value to beat skip three casts in ten (E23 survey row 4a C4)

- Red hero **C4** (Damacon, specialty resource gold): attack 0, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Archangel, 100 Pikeman
- Blue hero **Ufretin** (Ufretin, specialty creature Dwarf): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Easy (the first runs on Normal: below). Step east into the guards (three stacks of 4 Ogre Mages, three of 10 Champions, one Champion). Red's Archangels act first: the clock idles the run's seed, then they and Red's Pikemen defend.
2. Note each Ogre Mage turn's action in round 1 (Bloodlust at which stack, or none); seven runs with different seeds; after round 1 Red retreats.

**Record**

- C4: per run, each Ogre Mage turn's action in round 1 (Bloodlust's target or none): Easy, seven seeds (the Ogre Mages records 0 to 2; Bloodlust's target by record): 5, 4, 3 | none, 5, 4 | 5, 4, 3 | 5, 4, 3 | 5, none, 4 | none, 5, 4 | 5, 4, 3: 18 casts, 3 turns without one (a move), each at a field the same as a casting run's.
- anything odd: The first runs on Normal (Red's 5 Archangels alone, seeds 0 and 3000): Bloodlust at records 5, 4, 3 at all 6 turns; round 2 the lone Champion's (record 6) Bloodlust once its strong stacks had theirs.
