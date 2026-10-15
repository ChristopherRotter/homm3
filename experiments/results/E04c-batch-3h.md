# E04c batch 3h — results

Map file: `E04c-batch-3h.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `a8deac4e`.** The map's description in the scenario list starts with "Stamp a8deac4e.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Run by an agent** (E04c T7e, 2026-10-15) on the client lane's capture harness, as batches 3e and
3f: the same Complete 4.0 `Heroes3.exe`, under Wine in the cloud sandbox, without the HD mod; hot
seat (Red and Blue human), one map per encounter (`h3-experiment … --only <id>`, the build of
stamp a8deac4e), played by `experiments/agent/ballista.py` (every stack defends, Blue's commanded
Ballista in AR too): the Red hero steps east into the Blue hero, the clock is stepped 10 ms at a
time and the game's memory read after each step. A Ballista's shot is a drop of its target's hit
points while it is the last activated stack (+0x132B8/+0x132BC); records are the side's battle
records (+0x54CC), slots the army's. Each battle was played into round 3. DE's, DN's and AR's
first attempts drew a starting-bonus artifact for a hero (13, 20 and 49, 64) and were played
again. A run took 16 to 23 s of real time.

**Evaluation** (the agent, 2026-10-15): every shot as the code reading of `installation.md` §7.33
once the estimate starts from the Ballista's record (2..3), not from the range its hero's attack
makes of them in a strike. The spec's first predictions (from the engine's first port, which
estimated with that multiplied range) said the Behemoths in TU and the Titans in DN; both rounds 1
shot the small stack, which the record's 2..3 predicts (TU: the Archer 24 against 13; DN: the
Archers 24 against 16): the reading confirmed (0x442780 reads +0xD0/+0xD4; 0x443160 multiplies
only at 0x4431AC), the port fixed before the commit. VCMI's rule (the engine's until T7e) is
refuted by TI (the later stack on a tie), DE (desperate mode), DN (the value) and AR (an enemy
Ballista before creatures); AR also shows the Artillery factor in the enemy Ballista's worth
(without it the Zombies score more).

1. Copy `E04c-batch-3h.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps` and start it in hot seat (Red and Blue human). Save right away.
2. Each Red hero stands west of a Blue hero. Play the encounters in any order: select the Red hero and step east into the Blue hero; every stack defends (a Ballista commanded with Artillery too), Red's Ballista shoots on its own at the end of each round.
3. Note Red's Ballista's shot in rounds 1 and 2: the stack it hit (Blue's slot, or Blue's Ballista) and the hit points it lost. Then end the battle as you like and load the save for the next encounter.
4. Fill in this file (anything unexpected goes into the last line of an encounter) and keep it in the repo.

## TI — Two like stacks of Archers: the later or the lower slot (BAT-WAR-008)

- Red hero **TI** (Caitlin, specialty resource gold): attack 10, defense 0, power 1, knowledge 1; no skills; Ballista (war machine 1); no spells; army 20 Pikeman
- Opponent: Blue hero **TIb** (Nagash, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archer, 10 Archer

1. Every stack defends; Red's Ballista shoots on its own.
2. Play to round 2, noting the Ballista's targets, then end the battle as you like.

**Record**

- TI: the Ballista's shot in rounds 1 and 2: the stack it hit (Blue's slot) and the hit points it lost: Red's records: 20 Pikemen record 0 (hex 86), the Ballista record 1 (hex 51); Blue's: the Archers record 0 (slot 0, hex 49) and record 1 (slot 1, hex 151). Round 1: **slot 1**: 23 (100 → 77, 10 → 8 Archers); round 2: **slot 1**: 19 (77 → 58, 8 → 6). Slot 0 was never hit.
- anything odd: none: the order every round: the Pikemen, Archers slot 0, Archers slot 1, the Ballista (round 2: the Archers slot 0 first).

## TU — One Archer or three Behemoths: the turns divide, a shooter is not first (BAT-WAR-008, BAT-SIE-022)

- Red hero **TU** (Damacon, specialty resource gold): attack 30, defense 0, power 1, knowledge 1; no skills; Ballista (war machine 1); no spells; army 100 Pikeman
- Opponent: Blue hero **TUb** (Octavia, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Archer, 3 Behemoth

1. Every stack defends; Red's Ballista shoots on its own.
2. Play to round 2, noting the Ballista's targets, then end the battle as you like.

**Record**

- TU: the Ballista's shot in rounds 1 and 2: the stack it hit (Blue's slot) and the hit points it lost: Blue's records: the Archer record 0 (slot 0, hex 49), the Behemoths record 1 (slot 1, hex 151). Round 1: **the Archer (slot 0)**: 10 (10 → 0, killed); round 2: **the Behemoths (slot 1)**: 87 (480 → 393).
- anything odd: none: the order: the Behemoths, the Pikemen, the Archer, the Ballista.

## DE — Desperate: one Pikeman against five Archers and two Titans (BAT-WAR-008, BAT-SIE-023)

- Red hero **DE** (Clavius, specialty resource gold): attack 30, defense 0, power 1, knowledge 1; no skills; Ballista (war machine 1); no spells; army 1 Pikeman
- Opponent: Blue hero **DEb** (Aine, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Archer, 2 Titan

1. Every stack defends; Red's Ballista shoots on its own.
2. Play to round 2, noting the Ballista's targets, then end the battle as you like.

**Record**

- DE: the Ballista's shot in rounds 1 and 2: the stack it hit (Blue's slot) and the hit points it lost: Blue's records: the Archers record 0 (slot 0, hex 49), the Titans record 1 (slot 1, hex 151). Round 1: **the Archers (slot 0)**: 50 (50 → 0, all five killed); round 2: **the Titans (slot 1)**: 60 (600 → 540).
- anything odd: none (attempt 2; attempt 1 drew artifact 13 for Red).

## DN — Not desperate: a hundred Pikemen against five Archers and two Titans (BAT-WAR-008, BAT-SIE-023)

- Red hero **DN** (Gelare, specialty resource gold): attack 30, defense 0, power 1, knowledge 1; no skills; Ballista (war machine 1); no spells; army 100 Pikeman
- Opponent: Blue hero **DNb** (Jenova, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Archer, 2 Titan

1. Every stack defends; Red's Ballista shoots on its own.
2. Play to round 2, noting the Ballista's targets, then end the battle as you like.

**Record**

- DN: the Ballista's shot in rounds 1 and 2: the stack it hit (Blue's slot) and the hit points it lost: Blue's records as DE's. Round 1: **the Archers (slot 0)**: 50 (50 → 0, all five killed); round 2: **the Titans (slot 1)**: 68 (600 → 532).
- anything odd: round 1 shot the Archers, not the Titans the first predictions named (see the evaluation above); attempt 1 drew artifacts 20 (Red) and 49 (Blue).

## AR — Blue's Ballista under Expert Artillery or thirty Zombies (BAT-WAR-008, BAT-SIE-021)

- Red hero **AR** (Grindan, specialty resource gold): attack 10, defense 0, power 1, knowledge 1; no skills; Ballista (war machine 1); no spells; army 50 Pikeman
- Opponent: Blue hero **ARb** (Sephinroth, specialty resource crystal): attack 0, defense 0, power 1, knowledge 1; Expert Artillery; Ballista (war machine 1); no spells; army 30 Zombie

1. Every stack defends, Blue's commanded Ballista too (Defend at its turn); Red's Ballista shoots on its own.
2. Play to round 2, noting Red's Ballista's targets, then end the battle as you like.

**Record**

- AR: Red's Ballista's shot in rounds 1 and 2: the stack it hit (Blue's slot, or Blue's Ballista) and the hit points it lost: Blue's records: the Zombies record 0 (slot 0, hex 100), Blue's Ballista record 1 (hex 67). Round 1: **Blue's Ballista**: 20 (250 → 230); round 2: **Blue's Ballista**: 24 (230 → 206). The Zombies were never hit.
- anything odd: none: Blue's commanded Ballista waited for a command at its turn (3 s of game time) and accepted Defend both rounds; the order: the Pikemen, the Zombies, Red's Ballista, Blue's Ballista (attempt 2; attempt 1 drew artifact 64 for Blue).

## AN — Blue's Ballista without Artillery or thirty Zombies (BAT-WAR-008, BAT-SIE-021)

- Red hero **AN** (Rissa, specialty resource mercury): attack 10, defense 0, power 1, knowledge 1; no skills; Ballista (war machine 1); no spells; army 50 Pikeman
- Opponent: Blue hero **ANb** (Saurug, specialty resource gems): attack 0, defense 0, power 1, knowledge 1; no skills; Ballista (war machine 1); no spells; army 30 Zombie

1. Every stack defends; both Ballistas shoot on their own.
2. Play to round 2, noting Red's Ballista's targets, then end the battle as you like.

**Record**

- AN: Red's Ballista's shot in rounds 1 and 2: the stack it hit (Blue's slot, or Blue's Ballista) and the hit points it lost: Blue's records as AR's. Round 1: **the Zombies (slot 0)**: 26 (600 → 574, 30 → 29); round 2: **the Zombies**: 22 (574 → 552, 29 → 28).
- anything odd: none: Blue's automatic Ballista shot Red's Pikemen each round (1 hit point).
