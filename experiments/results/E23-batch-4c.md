# E23 batch 4c — results

Map file: `E23-batch-4c.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `80f6e208`.** The map's description in the scenario list starts with "Stamp 80f6e208.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

1. Played by an agent (experiments/agent/ai.py): one map per encounter, `h3-experiment E23-batch-4c.toml --only <id>`. By hand: copy the map into `Maps`, start it in hot seat (Red and Blue human) at the difficulty the run names, select the Red hero and step east into the guards (north into a town).
2. Every Red stack defends unless the procedure says otherwise. Note what the encounter asks, then end the battle (retreat) as the procedure says.
3. Fill in this file (anything unexpected goes into the last line of an encounter) and keep it in the repo.

## W3 — W3: a neutral Gold Dragon's choice between two equal targets follows its draw (E23 survey row 4a W3)

- Red hero **W3** (Sephinroth, specialty resource crystal): attack 0, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Archangel, 5 Archangel
- Blue hero **Ryland** (Ryland, specialty creature Dendroid Guard): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Step east into the guard (one Gold Dragon, row 5). Red's two stacks of 5 Archangels (rows 2 and 8, the same distance) act first: the clock idles the run's seed, then both defend.
2. Note which stack the dragon attacks in round 1; ten runs with different seeds; after round 1 Red retreats.

**Record**

- W3: per run, the stack the dragon attacked (row 2 or 8) and its damage: 
- anything odd: 

## TP — TP: Red's automatic First Aid Tent heals the stack whose top creature lost the most (BAT-Q51, BAT-WAR-003, BAT-AI-022)

- Red hero **TP** (Saurug, specialty resource gems): attack 0, defense 10, power 1, knowledge 1; no skills; First Aid Tent (war machine 3); no spells; army 30 Swordsman, 10 Champion
- Blue hero **Clancy** (Clancy, specialty creature Unicorn): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Step east into the guards (three shooter stacks). Red's hero has a First Aid Tent and no First Aid: the tent acts on its own at the end of each round. Red's Swordsmen and Champions defend.
2. Note at each tent turn the top creature's lost hit points of both Red stacks and the stack healed (or a Defend); after round 8 Red retreats.

**Record**

- TP: per tent turn: both stacks' top lost hit points, the stack healed (or Defend): 
- anything odd: 

## SC — S1, S2: Red's Cyclops in auto combat batter the weakest wall while Red is locked out (E23 survey rows 4b S1, S2, BAT-AI-004, BAT-AI-005)

- Red hero **SC** (Gelare, specialty resource gold): attack 0, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Cyclops, 200 Pikeman
- Opponent: Neutral Castle town (Village Hall, fort); garrison 100 Archer
- Blue hero **Aeris** (Aeris, specialty creature Pegasus): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Step north into the neutral Castle (a Fort: walls and gate, no moat or turrets; 100 Archers inside, which shoot and never sally). At Red's first turn auto combat is switched on: the computer plays Red's Cyclops and Pikemen through the same stack turn as a computer side's (installation.md §7.34), with Red's human flag set; Red's Catapult shoots on its own.
2. Note each Cyclops turn's action (a wall shot and its wall, or a shot at the Archers) with the walls' hit points before it, and each Pikemen turn's move, over 5 rounds; then auto combat off and Red retreats.

**Record**

- SC: per Cyclops turn, the walls' hit points and its action (wall shot at which wall, or a shot): 
- SC: per Pikemen turn with every wall standing, its move's hex: 
- anything odd: 
