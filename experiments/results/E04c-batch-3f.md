# E04c batch 3f — results

Map file: `E04c-batch-3f.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `b5e5ea95`.** The map's description in the scenario list starts with "Stamp b5e5ea95.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Run by an agent** (E04c T7d, 2026-10-14) on the client lane's capture harness, as batch 3e: the
same Complete 4.0 `Heroes3.exe`, under Wine in the cloud sandbox, without the HD mod; hot seat
(Red and Blue human), one map per encounter (`h3-experiment … --only <id>`), played by
`experiments/agent/turrets.py` (every stack defends): the clock stepped 10 ms at a time and the
game's memory read after each step. A keep's shot is a drop of its target's hit points while the
keep is the last activated stack (+0x132B8/+0x132BC); records are the side's battle records
(+0x54CC), slots the army's. Each battle was played into round 3 (its shot given as extra). BA's
first attempt drew a starting-bonus artifact (118) and was played again. A run took about 23 s of
real time.

1. Copy `E04c-batch-3f.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps` and start it in hot seat (Red and Blue human). Save right away.
2. Each Red hero stands south of a Blue town's entrance. Play the encounters in any order: select the hero and step north into the town; every stack defends, the keep and the Catapult act on their own.
3. Note the keep's shot in rounds 1 and 2: the stack it hit (slot, or the Ballista or Catapult) and the hit points it lost. Then end the battle as you like and load the save for the next encounter.
4. Fill in this file (anything unexpected goes into the last line of an encounter) and keep it in the repo.

## FV — Magogs or Medusas: the Fight Value against the AI value (BAT-SIE-020, BAT-SIE-021)

- Red hero **FV** (Caitlin, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Magog, 5 Medusa
- Opponent: Blue Castle town (Village Hall, fort, citadel); garrison 50 Pikeman

1. Every stack defends; the keep shoots on its own.
2. Play to round 2, noting the keep's targets, then end the battle as you like.

**Record**

- FV1: the keep's count; the keep's shot in rounds 1 and 2: the stack it hit (slot) and the hit points it lost: the keep's count 5 (record 1, position 254). Red's records: 5 Magogs record 0 (slot 0, hex 35), 5 Medusas record 1 (slot 1, hex 137), the Catapult record 2. Round 1: the Magogs (slot 0): 16 (65 → 49); round 2: the Magogs: 16 (49 → 33); (extra, round 3: the Magogs: 14). The Medusas were never hit.
- anything odd: none: the order every round: the keep, the Catapult, the Magogs, the Pikemen, the Medusas.

## TU — Royal Griffins or Giants: a flyer two turns away against a walker behind the walls (BAT-SIE-020, BAT-SIE-022)

- Red hero **TU** (Jenova, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Royal Griffin, 1 Giant
- Opponent: Blue Castle town (Village Hall, fort, citadel); garrison 50 Pikeman

1. Every stack defends; the keep shoots on its own.
2. Play to round 2, noting the keep's targets, then end the battle as you like.

**Record**

- TU1: the keep's count; the keep's shot in rounds 1 and 2: the stack it hit (slot) and the hit points it lost: the keep's count 5. Red's records: 5 Royal Griffins record 0 (slot 0, hex 35), 1 Giant record 1 (slot 1, hex 137), the Catapult record 2. Round 1: the Royal Griffins (slot 0): 13 (125 → 112); round 2: the Royal Griffins: 17 (112 → 95); (extra, round 3: the Royal Griffins: 15). The Giant was never hit.
- anything odd: none: the order every round: the keep, the Catapult, the Royal Griffins, the Giant, the Pikemen.

## DE — A weak defender: Master Gremlins or Magogs (BAT-SIE-020, BAT-SIE-023)

- Red hero **DE** (Aine, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Master Gremlin, 10 Magog
- Opponent: Blue Castle town (Village Hall, fort, citadel); garrison 1 Pikeman

1. Every stack defends; the keep shoots on its own.
2. Play to round 2, noting the keep's targets, then end the battle as you like.

**Record**

- DE1: the keep's count; the keep's shot in rounds 1 and 2: the stack it hit (slot) and the hit points it lost: the keep's count 5. Red's records: 10 Master Gremlins record 0 (slot 0, hex 35), 10 Magogs record 1 (slot 1, hex 137), the Catapult record 2; Blue's: 1 Pikeman record 0 (hex 100). Round 1: the Master Gremlins (slot 0): 14 (40 → 26, 10 → 7); round 2: the Master Gremlins: 19 (26 → 7, 7 → 2); (extra, round 3: the Master Gremlins: their last 7). The Magogs were never hit.
- anything odd: none: the order: the keep, the Catapult, the Magogs, the Pikeman, the Master Gremlins.

## DN — A strong defender: Master Gremlins or Magogs (DE's control) (BAT-SIE-020, BAT-SIE-023)

- Red hero **DN** (Octavia, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Master Gremlin, 10 Magog
- Opponent: Blue Castle town (Village Hall, fort, citadel); garrison 50 Pikeman

1. Every stack defends; the keep shoots on its own.
2. Play to round 2, noting the keep's targets, then end the battle as you like.

**Record**

- DN1: the keep's count; the keep's shot in rounds 1 and 2: the stack it hit (slot) and the hit points it lost: the keep's count 5. Red's records as in DE; Blue's: 50 Pikemen record 0 (hex 100). Round 1: the Magogs (slot 1): 17 (130 → 113); round 2: the Magogs: 13 (113 → 100); (extra, round 3: the Magogs: 15). The Master Gremlins were never hit.
- anything odd: none: the order as in DE.

## BA — The attacker's Ballista or a hundred Zombies (BAT-SIE-020, BAT-SIE-021, BAT-SIE-022)

- Red hero **BA** (Clavius, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; Ballista (war machine 1); no spells; army 100 Zombie
- Opponent: Blue Castle town (Village Hall, fort, citadel); garrison 30 Swordsman

1. Every stack defends; the keep shoots on its own (Red has no Artillery: its Ballista shoots on its own too).
2. Play to round 2, noting the keep's targets, then end the battle as you like.

**Record**

- BA1: the keep's count; the keep's shot in rounds 1 and 2: the stack it hit (slot, the Ballista or the Catapult) and the hit points it lost: the keep's count 5. Red's records: 100 Zombies record 0 (slot 0, hex 86), the Catapult record 1 (hex 119), the Ballista record 2 (hex 51); Blue's: 30 Swordsmen record 0 (hex 100). Round 1: the Ballista: 17 (250 → 233); round 2: the Ballista: 18 (233 → 215); (extra, round 3: the Ballista: 14). The Zombies were never hit.
- anything odd: the original's records put the siege's Catapult (record 1) before the hero's Ballista (record 2); the engine numbers the Ballista first (war machines in kind order, the Catapult last): no effect here (no tie between them). The order every round: the keep, the Catapult, the Swordsmen, the Zombies, the Ballista.
