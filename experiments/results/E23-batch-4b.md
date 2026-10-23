# E23 batch 4b — results

Map file: `E23-batch-4b.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `d32ff7f5`.** The map's description in the scenario list starts with "Stamp d32ff7f5.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

1. Played by an agent (experiments/agent/ai.py): one map per encounter, `h3-experiment E23-batch-4b.toml --only <id>`. By hand: copy the map into `Maps`, start it in hot seat (Red and Blue human; Tan is the computer's) at the difficulty the run names, select the Red hero and step east into Tan's hero.
2. Every Red stack defends unless the procedure says otherwise. Note what the encounter asks, then end the battle (retreat) as the procedure says.
3. Fill in this file (anything unexpected goes into the last line of an encounter) and keep it in the repo.

## LC — BAT-Q33: a computer player's stack frozen by bad morale is let off one time in four (BAT-Q33, S-25, BAT-MOR-004)

- Red hero **LC** (Caitlin, specialty resource gold): attack 0, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 20 Bone Dragon, 100 Pikeman
- Opponent: Tan (the computer) hero **LC** (Sylvia, specialty skill Navigation): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archer, 10 Wood Elf, 10 Gog, 10 Orc, 10 Lizardman
- Blue hero **Gunnar** (Gunnar, specialty skill Logistics): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Step east into Tan's hero. Red's Bone Dragons defend every turn; Red's Pikemen freeze (forced).
2. At the opening the harness forces every bad-morale draw to fall and every good-morale draw to fail. Count, for each of Tan's picks after the battle's first, whether the stack froze or took its turn; the same for Red's Pikemen. After round 12 Red retreats.

**Record**

- LC: the stacks' morale at the opening (Red's and Tan's): 
- LC: Tan's picks after the first: frozen, let off: 
- LC: Red's Pikemen's picks: frozen, took their turn: 
- anything odd: 

## H1 — H1: a computer hero's Archers in the tactics phase go to the first hex with the fewest free neighbours (E23 survey row 4b H1, BAT-AI-023)

- Red hero **H1** (Jenova, specialty resource gold): attack 0, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 20 Pikeman
- Opponent: Tan (the computer) hero **H1** (Sanya, specialty skill Eagle Eye): attack 0, defense 0, power 1, knowledge 1; Expert Tactics; no artifacts; no spells; army 20 Archer
- Blue hero **Geon** (Geon, specialty skill Eagle Eye): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Step east into Tan's hero (Expert Tactics; Red's hero has none): Tan places its stacks in the tactics phase.
2. Note the Archers' hex before and after the tactics phase, and the field's blocked hexes; after round 1 Red retreats.

**Record**

- H1: the Archers' hex before and after the tactics phase: 
- anything odd: 

## H3 — H3: a computer hero blesses its Archers at their own turn, not at the round's first (E23 survey row 4b H3, BAT-AI-017)

- Red hero **H3** (Rissa, specialty resource mercury): attack 0, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 30 Pikeman
- Opponent: Tan (the computer) hero **H3** (Kyrre, specialty skill Logistics): attack 0, defense 0, power 5, knowledge 5; no skills; Spell Book (spell book); spells Bless; army 10 Wolf Rider, 20 Archer
- Blue hero **Dessa** (Dessa, specialty skill Logistics): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Step east into Tan's hero (only Bless in its book, the mana for ten casts). Tan's Wolf Riders (speed 6) act before its Archers (speed 4); Red's Pikemen defend.
2. Note at whose activation Tan's hero casts (the action 1, its spell and hex), in rounds 1 and 2; after round 2 Red retreats.

**Record**

- H3: per round, the activation at which Tan's hero cast, its spell and target: 
- anything odd: 

## H4 — H4: a computer hero never casts Berserk, Fire Wall, Land Mine, Quicksand, Force Field or Remove Obstacle (E23 survey row 4b H4, BAT-AI-017)

- Red hero **H4** (Aine, specialty resource gold): attack 0, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman, 10 Pikeman, 10 Pikeman, 10 Pikeman, 10 Pikeman, 10 Pikeman, 10 Pikeman
- Opponent: Tan (the computer) hero **H4** (Malcom, specialty skill Eagle Eye): attack 0, defense 0, power 5, knowledge 10; no skills; Spell Book (spell book); spells Berserk, Fire Wall, Land Mine, Quicksand, Force Field, Remove Obstacle; army 10 Swordsman
- Blue hero **Oris** (Oris, specialty skill Eagle Eye): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Step east into Tan's hero (a book of those six spells only, mana for many casts). Red's seven stacks of Pikemen (three of them side by side on rows 4 to 6) defend.
2. Note every cast of Tan's hero over 4 rounds; after round 4 Red retreats.

**Record**

- H4: Tan's hero's casts over 4 rounds: 
- anything odd: 

## R1a — R1a: a computer hero defending on Hard retreats at a 14 % share of Fight Value (E23 survey row 4b R1, BAT-AI-024, slip 26)

- Red hero **R1a** (Octavia, specialty resource gold): attack 0, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 3 Archangel
- Opponent: Tan (the computer) hero **R1a** (Serena, specialty skill Eagle Eye, 2000 experience): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 2 Dendroid Guard, 2 Dendroid Guard
- Blue hero **Voy** (Voy, specialty skill Navigation): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Hard. Step east into Tan's hero (2000 experience; Tan owns a Tavern town). Red's 3 Archangels act first and defend.
2. Note Tan's first stack's action (a retreat, 4, or a move), Tan's gold at the opening; after round 2 Red retreats.

**Record**

- R1a: Tan's gold; Tan's first action (retreat or not): 
- anything odd: 

## R1b — R1b: a computer hero defending on Hard fights at a 17 % share of Fight Value (E23 survey row 4b R1, BAT-AI-024)

- Red hero **R1b** (Calid, specialty resource sulfur): attack 0, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 3 Archangel
- Opponent: Tan (the computer) hero **R1b** (Nimbus, specialty skill Eagle Eye, 2000 experience): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 3 Dendroid Guard, 2 Dendroid Guard
- Blue hero **Tiva** (Tiva, specialty skill Eagle Eye): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Peasant

1. Hard. Step east into Tan's hero (2000 experience; Tan owns a Tavern town). Red's 3 Archangels act first and defend.
2. Note Tan's first stack's action (a retreat, 4, or a move), Tan's gold at the opening; after round 2 Red retreats.

**Record**

- R1b: Tan's gold; Tan's first action (retreat or not): 
- anything odd: 
