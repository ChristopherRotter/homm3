# E23 batch 4c — results

Map file: `E23-batch-4c.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `1885c096`.** The map's description in the scenario list starts with "Stamp 1885c096.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Run by an agent** (E23 T7, 2026-10-23) on the client lane's capture harness, as batch 4a: one map per encounter, hot seat with Red and Blue human, played by `experiments/agent/ai.py` with `experiments/agent/E23-batch-4c.toml`; every value read from the game's memory. W3, TP and SC were played on the spec's earlier builds (stamps 80f6e208 and 0a64ed11: C4f3's, C4f2's and TP2's piles not yet on the map, far from their battles), C4f3 and C4f2 on stamp 296b2a8c, TP2 (with Death Ripple) on d567fb54, TP3 on this build: each of these maps is this build's but for the stamp. SC played Red by auto combat.

1. Played by an agent (experiments/agent/ai.py): one map per encounter, `h3-experiment E23-batch-4c.toml --only <id>`. By hand: copy the map into `Maps`, start it in hot seat (Red and Blue human) at the difficulty the run names, select the Red hero and step east into the guards (north into a town).
2. Every Red stack defends unless the procedure says otherwise. Note what the encounter asks, then end the battle (retreat) as the procedure says.
3. Fill in this file (anything unexpected goes into the last line of an encounter) and keep it in the repo.

## W3 — W3: a neutral Gold Dragon's choice between two equal targets follows its draw (E23 survey row 4a W3)

- Red hero **W3** (Sephinroth, specialty resource crystal): attack 0, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Archangel, 5 Archangel
- Blue hero **Ryland** (Ryland, specialty creature Dendroid Guard): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Step east into the guard (one Gold Dragon, row 5). Red's two stacks of 5 Archangels (rows 2 and 8, the same distance) act first: the clock idles the run's seed, then both defend.
2. Note which stack the dragon attacks in round 1; ten runs with different seeds; after round 1 Red retreats.

**Record**

- W3: per run, the stack the dragon attacked (row 2 or 8) and its damage: Seeds 0 to 5: the row-8 stack (hex 137), hit points lost 20, 20, 20, 20, 23, 19; seeds 6 to 9: the row-2 stack (hex 35), 20, 20, 22, 22. Both targets; at least five distinct streams by the damage (two per target).
- anything odd: The field held 12 blocked hexes (the tile's obstacles); both targets one turn away for the flyer.

## TP — TP: Red's automatic First Aid Tent heals the stack whose top creature lost the most (BAT-Q51, BAT-WAR-003, BAT-AI-022)

- Red hero **TP** (Saurug, specialty resource gems): attack 0, defense 10, power 1, knowledge 1; no skills; First Aid Tent (war machine 3); no spells; army 30 Swordsman, 10 Champion
- Blue hero **Clancy** (Clancy, specialty creature Unicorn): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Step east into the guards (three shooter stacks). Red's hero has a First Aid Tent and no First Aid: the tent acts on its own at the end of each round. Red's Swordsmen and Champions defend.
2. Note at each tent turn the top creature's lost hit points of both Red stacks and the stack healed (or a Defend); after round 8 Red retreats.

**Record**

- TP: per tent turn: both stacks' top lost hit points, the stack healed (or Defend): Rounds 1 to 8: only the Swordsmen were hurt (the shooters never hit the Champions: top lost 0): the tent healed the Swordsmen at every turn (action 11 at hex 35), e.g. 12 to 9, 22 to 10, 29 to 21. Not decisive: BAT-WAR-003 draws among hurt stacks only (TP2).
- anything odd: none

## SC — S1, S2: Red's Cyclops in auto combat batter the weakest wall while Red is locked out (E23 survey rows 4b S1, S2, BAT-AI-004, BAT-AI-005)

- Red hero **SC** (Gelare, specialty resource gold): attack 0, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Cyclops, 200 Pikeman
- Opponent: Neutral Castle town (Village Hall, fort); garrison 100 Archer
- Blue hero **Aeris** (Aeris, specialty creature Pegasus): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Step north into the neutral Castle (a Fort: walls and gate, no moat or turrets; 100 Archers inside, which shoot and never sally). At Red's first turn auto combat is switched on: the computer plays Red's Cyclops and Pikemen through the same stack turn as a computer side's (installation.md §7.34), with Red's human flag set; Red's Catapult shoots on its own.
2. Note each Cyclops turn's action (a wall shot and its wall, or a shot at the Archers) with the walls' hit points before it, and each Pikemen turn's move, over 5 rounds; then auto combat off and Red retreats.

**Record**

- SC: per Cyclops turn, the walls' hit points and its action (wall shot at which wall, or a shot): Round 1: Red's Catapult broke the upper wall (2 to 0) with its first shot before the Cyclops' turn; the Cyclops then shot the Archers in rounds 1 to 5 (the Catapult went on: mid-upper, mid-lower, lower). The locked-out case (every wall standing at the Cyclops' turn) was not reached.
- SC: per Pikemen turn with every wall standing, its move's hex: None with every wall standing; through the breach: hex 55, 60, 81, then an attack on the Archers in round 4.
- anything odd: The port replayed in the engine (three streams) breached in one of three before the Cyclops' turn and shot the Archers then too.

## C4f3 — C4f3: neutral Ogre Mages never bless a walker weaker than themselves while their walker has a value (E23 survey row 4a C4)

- Red hero **C4f3** (Grindan, specialty resource gold): attack 0, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant, 100 Pikeman
- Blue hero **Josephine** (Josephine, specialty creature Stone Golem): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Easy. Step east into the guards (three stacks of 3 Ogre Mages, three of 20 Archers, one Champion). Red's Peasant and Pikemen defend.
2. Note each Ogre Mage turn's action in round 1; after round 1 Red retreats.

**Record**

- C4f3: each Ogre Mage turn's action in round 1: No cast: all three moved (to 10, 44, and towards 137); no Bloodlust on any stack.
- anything odd: The field held 21 blocked hexes.

## C4f2 — C4f2: the same with Ogre Mages weaker than the lone Champion: it is blessed (E23 survey row 4a C4)

- Red hero **C4f2** (Nagash, specialty resource gold): attack 0, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant, 100 Pikeman
- Blue hero **Fafner** (Fafner, specialty creature Naga): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Easy. Step east into the guards (three stacks of 2 Ogre Mages, three of 20 Archers, one Champion). Red's Peasant and Pikemen defend.
2. Note each Ogre Mage turn's action in round 1; after round 1 Red retreats.

**Record**

- C4f2: each Ogre Mage turn's action in round 1: The first: Bloodlust on the lone Champion (record 6, at hex 142); the second and third: moves (their only candidate blessed).
- anything odd: The field held 6 blocked hexes (another tile than C4f3's).

## TP2 — TP2: Red's automatic First Aid Tent between two hurt stacks, and with none hurt (BAT-Q51, BAT-WAR-003, BAT-AI-022)

- Red hero **TP2** (Clavius, specialty resource gold): attack 0, defense 10, power 6, knowledge 10; no skills; Spell Book (spell book), First Aid Tent (war machine 3); spells Death Ripple; army 30 Swordsman, 10 Champion
- Blue hero **Piquedram** (Piquedram, specialty creature Stone Gargoyle): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Step east into the guards (Diamond Golems: slow, a twentieth of spell damage). Round 1: no Red stack is hurt (the golems are far): the tent's turn has no patient. Rounds 2 to 5, Red's first turn: the hero casts Death Ripple (its book's only spell; power 6: 10 + 5 x 6 = 40 to every living stack, the tent spared): both Red stacks are hurt, the Champions' top creature (100 hit points) 40, the Swordsmen's (35) 5. A first run with Armageddon destroyed the tent with its first cast (80 against 75 hit points).
2. Note at each tent turn both stacks' top lost hit points and the stack healed (or Defend); after round 8 Red retreats.

**Record**

- TP2: per tent turn: both stacks' top lost hit points, the stack healed (or Defend): Round 1 (no stack hurt): no heal, no action of the tent's read, no defending bit on it (a war machine's Defend gives no bonus, 0x479722: no proof either way). Round 2, after Death Ripple: Swordsmen 5, Champions 40: the Champions healed (40 to 29). Round 3: the golems destroyed the tent.
- anything odd: A first run with Armageddon (power 1: 80) destroyed the tent with the first cast (75 hit points); replayed with Death Ripple, which spares it.

## TP3 — TP3: the automatic tent between two hurt stacks, round after round, outside a held Citadel (BAT-Q51, BAT-WAR-003, BAT-AI-022)

- Red hero **TP3** (Caitlin, specialty resource gold): attack 0, defense 10, power 6, knowledge 10; no skills; Spell Book (spell book), First Aid Tent (war machine 3); spells Death Ripple; army 30 Swordsman, 10 Champion
- Opponent: Neutral Castle town (Village Hall, fort, citadel); garrison 20 Swordsman
- Blue hero **Iona** (Iona, specialty creature Genie): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Step north into the neutral Citadel (W1's: its Swordsmen hold the walls and stay inside while the mid-lower wall and the gate stand). Every round from round 1, Red's first turn: the hero casts Death Ripple (power 6: 40 to every living stack, the garrison's too): both of Red's stacks are hurt at every tent turn (the keep's arrows add more). TP2's golems destroyed the tent in round 3.
2. Note at each tent turn both stacks' top lost hit points and the stack healed; after round 8 Red retreats.

**Record**

- TP3: per tent turn: both stacks' top lost hit points, the stack healed: Rounds 1 to 6 (Swordsmen, Champions): 19, 40; 3, 63; 25, 81; 9, 99; 14, 44; 33, 82: the Champions healed at every turn (6 of 6, the most lost); the tent fell in round 7.
- anything odd: The garrison held its walls throughout (Death Ripple hurt it too); Red's hero cast Death Ripple in all 8 rounds.
