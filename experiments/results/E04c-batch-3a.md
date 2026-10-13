# E04c batch 3a — results

Map file: `E04c-batch-3a.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `adbe3948`.** The map's description in the scenario list starts with "Stamp adbe3948.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

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

1. Copy `E04c-batch-3a.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps` and start it as a single scenario (Red human). Save right away.
2. Each Red hero stands next to a Peasant on its encounter's ground. Play the encounters in any order: select the hero and step east into the Peasant.
3. At the battle's start, before anything moves, right-click each stack (Red's and the Peasant) and note its morale and luck (the numbers beside the two icons of its window) and its speed. Then end the battle as you like (win or retreat) and load the save for the next encounter.
4. Fill in this file (anything unexpected goes into the last line of an encounter) and keep it in the repo.

## G — Control: CG's army and hero on plain grass (BAT-MOR-002, BAT-LCK-001, BAT-TER-001, D-1)

- Red hero **G** (Caitlin, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; Clover of Fortune (misc 1); no spells; army 10 Pikeman, 10 Centaur, 10 Gremlin, 10 Skeleton
- Opponent: 1 Peasant (wandering monster, never flees)

1. Attack the Peasant. At the battle's start, note each stack's morale, luck and speed (right-click).
2. End the battle as you like.

**Record**

| stack | morale | luck | speed |
|---|---|---|---|
| Pikemen | -3 | +1 | 5 (native) |
| Centaurs | -3 | +1 | 7 (native) |
| Gremlins | -3 | +1 | 4 |
| Skeletons | 0 | +1 | 4 |
| Peasant | +1 | 0 | 3 |

- the battlefield (its picture: grass, a magic field, …): the ordinary grass battlefield (field -1, terrain 2)
- anything odd: nothing (the Peasant: morale +1, luck 0; attempt 1)

## CG — Cursed Ground: morale and luck 0 for every stack, negative morale included; no native stack (D-1, S-28, BAT-TER-003, BAT-TER-006)

- Red hero **CG** (Jenova, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; Clover of Fortune (misc 1); no spells; army 10 Pikeman, 10 Centaur, 10 Gremlin, 10 Skeleton
- Opponent: 1 Peasant (wandering monster, never flees)

1. Attack the Peasant. At the battle's start, note each stack's morale, luck and speed (right-click).
2. End the battle as you like.

**Record**

| stack | morale | luck | speed |
|---|---|---|---|
| Pikemen | 0 | 0 | 4 |
| Centaurs | 0 | 0 | 6 |
| Gremlins | 0 | 0 | 4 |
| Skeletons | 0 | 0 | 4 |
| Peasant | 0 | 0 | 3 |

- the battlefield (its picture): Cursed Ground (field 2, terrain 2, grass)
- anything odd: nothing (the Peasant: morale 0, luck 0; attempt 1)

## CS — The SoD picture of Cursed Ground (object 223): a Cursed Ground battle or none (D-4, D-1)

- Red hero **CS** (Rissa, specialty resource mercury): attack 1, defense 1, power 1, knowledge 1; no skills; Clover of Fortune (misc 1); no spells; army 10 Pikeman, 10 Centaur, 10 Gremlin, 10 Skeleton
- Opponent: 1 Peasant (wandering monster, never flees)

1. Attack the Peasant. At the battle's start, note each stack's morale, luck and speed (right-click).
2. End the battle as you like.

**Record**

| stack | morale | luck | speed |
|---|---|---|---|
| Pikemen | 0 | 0 | 4 |
| Centaurs | 0 | 0 | 6 |
| Gremlins | 0 | 0 | 4 |
| Skeletons | 0 | 0 | 4 |
| Peasant | 0 | 0 | 3 |

- the battlefield (its picture): Cursed Ground (field 2, terrain 2, grass)
- anything odd: nothing (the Peasant: morale 0, luck 0; attempt 1)

## PL — Control: Magic Plains (object 46) (D-4, BAT-TER-002)

- Red hero **PL** (Aine, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman
- Opponent: 1 Peasant (wandering monster, never flees)

1. Attack the Peasant. At the battle's start, note the battlefield's picture and the Pikemen's morale, luck and speed.
2. End the battle as you like.

**Record**

| stack | morale | luck | speed |
|---|---|---|---|
| Pikemen | +1 | 0 | 5 (native) |
| Peasant | +1 | 0 | 3 |

- the battlefield (its picture): Magic Plains (field 1, terrain 2, grass)
- anything odd: nothing (the Peasant: morale +1, luck 0; attempt 1)

## MS — The SoD picture of Magic Plains (object 230): a Magic Plains battle or none (D-4)

- Red hero **MS** (Octavia, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman
- Opponent: 1 Peasant (wandering monster, never flees)

1. Attack the Peasant. At the battle's start, note the battlefield's picture and the Pikemen's morale, luck and speed.
2. End the battle as you like.

**Record**

| stack | morale | luck | speed |
|---|---|---|---|
| Pikemen | +1 | 0 | 5 (native) |
| Peasant | +1 | 0 | 3 |

- the battlefield (its picture): Magic Plains (field 1, terrain 2, grass)
- anything odd: nothing (the Peasant: morale +1, luck 0; attempt 1)

## CF — Clover Field: +2 luck for Stronghold, Fortress and Conflux creatures; creatures without a town (D-2, BAT-TER-003)

- Red hero **CF** (Calid, specialty resource sulfur): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Goblin, 10 Gnoll, 10 Pixie, 10 Peasant, 10 Pikeman
- Opponent: 1 Peasant (wandering monster, never flees)

1. Attack the Peasant. At the battle's start, note each stack's morale, luck and speed (right-click).
2. End the battle as you like.

**Record**

| stack | morale | luck | speed |
|---|---|---|---|
| Goblins | -3 | +2 | 5 |
| Gnolls | -3 | +2 | 4 |
| Pixies | -3 | +2 | 8 (native) |
| Peasants (Red's) | -3 | 0 | 3 |
| Pikemen | -3 | 0 | 5 (native) |
| Peasant (the monster) | +1 | 0 | 3 |

- the battlefield (its picture): Clover Field (field 5, terrain 2, grass)
- anything odd: nothing (the Peasant: morale +1, luck 0; attempt 1)

## HG — Control: Holy Ground's morale by the creature's town (BAT-TER-003)

- Red hero **HG** (Clavius, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman, 10 Imp, 10 Goblin, 10 Peasant
- Opponent: 1 Peasant (wandering monster, never flees)

1. Attack the Peasant. At the battle's start, note each stack's morale, luck and speed (right-click).
2. End the battle as you like.

**Record**

| stack | morale | luck | speed |
|---|---|---|---|
| Pikemen | -1 | 0 | 5 (native) |
| Imps | -3 | 0 | 5 |
| Goblins | -2 | 0 | 5 |
| Peasants (Red's) | -2 | 0 | 3 |
| Peasant (the monster) | +1 | 0 | 3 |

- the battlefield (its picture): Holy Ground (field 3, terrain 2, grass)
- anything odd: nothing (the Peasant: morale +1, luck 0; attempt 1)

## LP — Control: Lucid Pools (an elemental field) as the battlefield (BAT-TER-002)

- Red hero **LP** (Damacon, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman
- Opponent: 1 Peasant (wandering monster, never flees)

1. Attack the Peasant. At the battle's start, note the battlefield's picture and the Pikemen's morale, luck and speed.
2. End the battle as you like.

**Record**

| stack | morale | luck | speed |
|---|---|---|---|
| Pikemen | +1 | 0 | 5 (native) |
| Peasant | +1 | 0 | 3 |

- the battlefield (its picture): Lucid Pools (field 6, terrain 2, grass)
- anything odd: nothing (the Peasant: morale +1, luck 0; attempt 3)
