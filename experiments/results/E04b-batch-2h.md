# E04b batch 2h — results

**Played by an agent** (2026-10-24) on the capture harness (`experiments/agent/ai.py`, recipe `experiments/agent/E04b-batch-2h.toml`): the Complete 4.0 `Heroes3.exe` under Wine without the HD mod, one map per run (`--only`), hot seat on Normal, good and bad morale forced to fail but where a run forces one draw; each run played again until neither hero wore a starting bonus's artifact. Every value below was read from the game's memory (`timeline.jsonl`, `controls.jsonl`: the battle window's buttons, `rolls`: xdd.dll's forced draws).

Map file: `E04b-batch-2h.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `9abbdcc6`.** The map's description in the scenario list starts with "Stamp 9abbdcc6.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

1. Played by an agent (experiments/agent/ai.py with experiments/agent/E04b-batch-2h.toml): one map per encounter, `h3-experiment E04b-batch-2h.toml --only <id>`. By hand: copy the map into `Maps`, start it in hot seat (Red and Blue human), select the Red hero and step east into the Blue hero; you command both sides.
2. Every stack defends unless the procedure says otherwise; a stack that gets an extra turn from good morale defends.
3. "Up-right", "up-left", "down-left", "down-right" mean the neighbouring hex in that direction; rows are counted from the top (row 0) to the bottom (row 10), columns from the left (0) to the right (16).
4. Fill in this file (anything unexpected goes into the last line of an encounter) and keep it in the repo.

## HM — A hypnotized stack's morale and luck: its controller's or its owner's (BAT-MAG-026, BAT-SIE-025, BAT-MOR-002)

- Red hero **HM** (Clavius, specialty resource gold): attack 0, defense 0, power 2, knowledge 2; Expert Leadership, Expert Luck; Spell Book (spell book); spells Hypnotize; army 5 Wolf Rider
- Opponent: Blue hero **HMb** (Kyrre, specialty skill Logistics): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Swordsman, 10 Dwarf

1. Attack the Blue hero. Right-click Blue's Swordsman: morale, luck.
2. Round 1, the Wolf Riders' turn: Red casts Hypnotize on the Swordsman; right-click it: morale, luck. The Wolf Riders Defend. The Swordsman (Red's now) Defends; its morale and luck again. Blue's Dwarves Defend.
3. Round 2: every stack Defends; the Swordsman's morale and luck at its turn. Retreat with Red.

**Record**

- the Swordsman's morale and luck: before, after the Hypnotize, at its turn, in round 2: 0, 0; 4, 3 (Red's: the Wolf Riders' record holds morale 4 unclamped, luck 3) from the read after the cast; 4, 3 at its turn; 0, 0 once Hypnotize had ended (power 2: two rounds)
- anything odd: nothing

## WP — Elementals summoned in the wait phase: who acts next, and is Wait offered (BAT-Q28, BAT-TRN-002, BAT-MAG-039)

- Red hero **WP** (Nagash, specialty resource gold): attack 0, defense 0, power 2, knowledge 3; no skills; Spell Book (spell book); spells Air Elemental; army 10 Peasant, 10 Pikeman
- Opponent: Blue hero **WPb** (Sylvia, specialty skill Navigation): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Swordsman

1. Attack the Blue hero. Round 1: Blue's Swordsmen Defend; Red's Pikemen Wait, then the Peasants Wait.
2. The wait phase: the Peasants' turn: Red casts Air Elemental; the Peasants Defend. Which stack acts next, the Pikemen or the elementals? On the elementals' turn: is the Wait button lit? They Defend (the WK run: press W).
3. Retreat with Red at its next turn.

**Record**

- after the summon in the wait phase: the next stack (Pikemen/elementals); the Wait button on the elementals' turn (lit/disabled); with W pressed (WK): the Pikemen, then the elementals (four Air Elementals); disabled: the Wait button's (2009) state word gains the bit 0x8 at the wait phase's first turn (0x0006 to 0x000E) for every stack; WK: the W key changed nothing within 1.5 s, the elementals then defended (the D key)
- anything odd: nothing

## FT — A stack frozen by bad morale, then a speed tie across the sides (BAT-Q28, BAT-TRN-001, BAT-MOR-004)

- Red hero **FT** (Saurug, specialty resource gems): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Wolf Rider, 5 Demon, 5 Skeleton
- Opponent: Blue hero **FTb** (Sephinroth, specialty resource crystal): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 2 Royal Griffin, 5 Medusa

1. Attack the Blue hero. Round 1: Blue's Royal Griffins Defend. Red's Wolf Riders freeze from bad morale (the harness forces the roll). Then the Demons (Red) and the Medusas (Blue), both of speed 5: which acts first? Every stack Defends.
2. Retreat with Red at its next turn.

**Record**

- the Wolf Riders froze (yes/no); then first: Demons/Medusas: yes (the forced draw; the bad-morale animation 30 over them); the Demons, then the Medusas
- anything odd: nothing

## TS — Wights wounded after they waited: do they regenerate in the wait phase (BAT-Q28, BAT-TRN-002, BAT-TRN-005)

- Red hero **TS** (Rissa, specialty resource mercury): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Wight
- Opponent: Blue hero **TSb** (Halon, specialty skill Mysticism): attack 0, defense 0, power 1, knowledge 1; no skills; Spell Book (spell book); spells Magic Arrow; army 10 Dwarf

1. Attack the Blue hero. Round 1: the Wights Wait. Blue's Dwarves' turn: Blue casts Magic Arrow at the Wights (20: one dies, the top one loses 2); the Dwarves Defend.
2. The wait phase, the Wights' turn: right-click them: health of the top one. They Defend.
3. Round 2, the Wights' turn: the top one's health again. Retreat with Red.

**Record**

- the top Wight's health: after the Magic Arrow, at its wait-phase turn, at round 2's turn: 4 Wights at 70 of 72 (one killed, the top at 16); 70 (no regeneration); 72 at round 2's first activation
- anything odd: nothing

## MR — Archers that shoot from a Fire Wall and burn to death: is good morale rolled first (BAT-MOR-003, BAT-MAG-028)

- Red hero **MR** (Voy, specialty skill Navigation): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Archer
- Opponent: Blue hero **MRb** (Axsis, specialty skill Mysticism): attack 0, defense 0, power 2, knowledge 1; no skills; Spell Book (spell book); spells Fire Wall; army 5 Wolf Rider

1. Attack the Blue hero. Round 1: Blue's Wolf Riders' turn: Blue casts Fire Wall at (2, 5), the hex right of the Archers (it burns on (2, 5) and (1, 4)); the Wolf Riders Defend. The Archers walk onto (2, 5) and burn (30: 2 of 5 left).
2. Round 2: the Wolf Riders Defend. The Archers shoot them; they burn again where they stand and die. (The harness forces the next good-morale draw to fall just before the shot.) The battle ends.
3. The MRc run: no Fire Wall; the Archers Defend in round 1 and shoot in round 2: the control, where the forced draw gives them another turn.

**Record**

- the Archers' count after the walk; after the shot (dead?); the forced draw taken (yes/no); the good-morale bit: 2 (20 of 50); dead (the second burn, 20 to 0); yes: the pending draw fell to 0 with the shot, before the burn; set (0x1000000) on the dying Archers. MRc: the draw taken, the good-morale bit and another turn
- anything odd: nothing
