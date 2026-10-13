# E04c batch 3b — results

Map file: `E04c-batch-3b.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `936d73fd`.** The map's description in the scenario list starts with "Stamp 936d73fd.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Run by an agent** (E04c T3, 2026-10-13) on the client lane's capture harness: the same Complete
4.0 `Heroes3.exe`, under Wine in the cloud sandbox, without the HD mod; one map per encounter
(`h3-experiment … --only <id>`), the battle opened by `experiments/agent/opening.py` and every
value read from the game's memory at the opening, before anything moved (the stacks' +0x4E8
morale, +0x4EC luck, +0xC4 speed and +0x4D8 native flag; the battle manager's +0x53C0 field and
+0x5394 terrain). A single scenario starts with a random starting bonus, sometimes an artifact for
the hero: a run whose hero wore anything beyond the spec's artifacts and the Catapult was played
again (the first run's TM showed morale +2 so; the kept runs wear nothing else). The pictures
were checked on the screen dumps for CS and SHM. Cross-check: batch 1a's R1 replayed on the same
harness gave the user's 105, 210, 315, 420, 525 and 630.

1. Copy `E04c-batch-3b.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps` and start it as a single scenario (Red human). Save right away.
2. Each Red hero stands next to a Peasant. Play the encounters in any order: select the hero and step east into the Peasant.
3. At the battle's start, before anything moves, note the battlefield's picture and right-click Red's stack: its luck and speed. Then end the battle as you like (win or retreat) and load the save for the next encounter.
4. Fill in this file (anything unexpected goes into the last line of an encounter) and keep it in the repo.

## TM — Clover Field under the Peasant only (BAT-TER-002, E04c-battlefields §4)

- Red hero **TM** (Caitlin, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Goblin
- Opponent: 1 Peasant (wandering monster, never flees)

1. Attack the Peasant (it stands on the Clover Field, the hero beside it does not). At the battle's start, note the battlefield's picture and the Goblins' luck.
2. End the battle as you like.

**Record**

- the battlefield (its picture): Clover Field (field 5, terrain 2, grass)
- Goblins' luck: +2 (morale +1, speed 5)
- anything odd: nothing (the Peasant: morale +1, luck 0; attempt 1)

## TH — Clover Field under the hero only (BAT-TER-002, E04c-battlefields §4)

- Red hero **TH** (Jenova, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Goblin
- Opponent: 1 Peasant (wandering monster, never flees)

1. Attack the Peasant (the hero stands on the Clover Field, the Peasant does not). At the battle's start, note the battlefield's picture and the Goblins' luck.
2. End the battle as you like.

**Record**

- the battlefield (its picture): the ordinary grass battlefield (field -1, terrain 2)
- Goblins' luck: 0 (morale +1, speed 5)
- anything odd: nothing (the Peasant: morale +1, luck 0; attempt 1)

## SH — Both next to water: the shore (BAT-TER-002, E04c-battlefields §4)

- Red hero **SH** (Rissa, specialty resource mercury): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman
- Opponent: 1 Peasant (wandering monster, never flees)

1. Attack the Peasant (both stand next to the water above them). At the battle's start, note the battlefield's picture and the Pikemen's speed.
2. End the battle as you like.

**Record**

- the battlefield (its picture): the shore (field 0, terrain 1, sand)
- Pikemen's speed: 4 (not native; morale +1, luck 0)
- anything odd: nothing (the Peasant: morale +1, luck 0; attempt 2)

## SHH — Only the hero next to water (BAT-TER-002, E04c-battlefields §4)

- Red hero **SHH** (Aine, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman
- Opponent: 1 Peasant (wandering monster, never flees)

1. Attack the Peasant (the water is west of the hero; the Peasant is two tiles from it). At the battle's start, note the battlefield's picture and the Pikemen's speed.
2. End the battle as you like.

**Record**

- the battlefield (its picture): the ordinary grass battlefield (field -1, terrain 2)
- Pikemen's speed: 5 (native; morale +1, luck 0)
- anything odd: nothing (the Peasant: morale +1, luck 0; attempt 2)

## SHM — Only the Peasant next to water (BAT-TER-002, E04c-battlefields §4)

- Red hero **SHM** (Octavia, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman
- Opponent: 1 Peasant (wandering monster, never flees)

1. Attack the Peasant (the water is east of it; the hero is two tiles from it). At the battle's start, note the battlefield's picture and the Pikemen's speed.
2. End the battle as you like.

**Record**

- the battlefield (its picture): the shore (field 0, terrain 1, sand)
- Pikemen's speed: 4 (not native; morale +1, luck 0)
- anything odd: nothing (the Peasant: morale +1, luck 0; attempt 1)
