# E23 — the original's battle AI: survey (T1b)

The survey behind E23's seam (T1b, 2026-10-20): what the original's computer does in a battle, as
read; where the wiki and VCMI stand; the readings at risk with a planned batch row each; the
original's slips in its own estimates; and the questions the ADR draft
(`docs/lanes/battle/adr-drafts/original-battle-ai.md`) settles. The detail of every routine stays
in its reading; this file points there.

Sources: `code` (the executable, `docs/formats/installation.md` §7.34 to §7.46, E23 T1a1 to T1a5e;
the automatic machines §7.29, §7.31, §7.33; insight only, Q29); `wiki` (heroes.thelazy.net, read
2026-10-20 through the summarising fetch: `Combat` (retreating and surrendering), `Difficulty
level`, `AI/Fight Value`, `Genie and Master Genie`, `Faerie Dragon`, the full-text search "battle
AI"); `vcmi` (VCMI 1.7.5: `AI/BattleAI/`, `AI/StupidAI/`, `AI/Nullkiller/AIGateway.cpp`,
`config/schemas/settings.json`, `lib/battle/CBattleInfoCallback.cpp`). VCMI's battle AI is its own
design (the epic's non-goals): it serves to show which behaviours an independent reimplementation
chose, not as a source of the original's choices.

## 1. The original's computer in a battle (the reading)

**Two decisions per activation** (§7.34). At every activation of a stack of a side the computer
plays (a side without a human player, or a human's in auto-combat or a hidden battle; 0x474BF0),
first **the hero's cast** (0x422DA0): unless the hero cast this round, the tactics phase, or a war
machine is active; the side's hero needs a spell book, neither hero the Orb of Inhibition, no
anti-magic garrison; then the spell context, the retreat test and the cast choice (T1a6). Then
**the stack's action** (0x477EE0): the retreat test again (outside the tactics phase); yes means
retreat for a computer side (never surrender: the surrender branch is a human's only); no means
the stack's turn. The Catapult and the First Aid Tent have turns of their own (§7.29, §7.14); the
automatic Ballista and the turrets are built (BAT-WAR-008, BAT-SIE-020 to 023).

**The stack's turn** (§7.34). Default Defend. A stack that can shoot now (or the Ballista) is a
shooter: in the tactics phase it may move to the hex with the fewest free neighbours (above Easy;
0x422060), else it shoots (0x41F060, §7.33, §7.35: the best value per turn, the area shooters'
value at a hex with friendly losses subtracted, the Cyclops's wall shot). Any other stack is a
walker (0x421F80): the walker's choice (0x421680); if it chose nothing, the creature's cast entry
(0x421280); if neither acted, **Wait on Hard and above** (outside the wait phase), else Defend.

**The walker's choice** (§7.35 to §7.38). Hex scores (Fire Walls, the moat under SoD rules, from
Normal the enemies' reach, from Hard the area threat and the breath lines; floored at the stack's
whole value); the pathfinder's map (the original's 0x4B3400, built in T1c); per enemy record a
value: the plan (the best hex to strike it from, its turns), the melee value with the jousting
run-up, the last retaliation's gain (Hard and up), **a draw of 75 to 100 %**; the best by
ability, fewer turns, value per turn, the top creature's wounds, the path's cost, the later
record. Then: the creature's cast first (outside a prediction); an attack one turn away (move and
attack); a walk towards the plan's hex (on Hard and up **holding back** from a worse-scored hex and
waiting); a besieger without a target walks to its row's wall; a defender may hold the walls; the
fallbacks guard a shooter or step to safety (Hard and up). "The stronger side" in three branches
means the stronger **at range**: shooters, the hero's value from its maximum mana and its best
damage spell (§7.39, up to four times the side's whole strength), a fort's turrets.

**The creature casts** (§7.21, §7.42 to §7.46). Only five creatures cast for the computer: the
Archangel and the Pit Lord (the raise: the side's dead or raisable stack worth the most, against
the walker's value, doubled when overwhelmingly stronger), the Master Genie and the Ogre Mage (the
scan: against the walker's value; with one to beat, `random(1, 100) ≤ 30` skips the cast and only
a stack at least as strong as the caster is a target; the Genie's value the mean over the spells
it may draw), the Faerie Dragon (it casts whenever its drawn spell is worth anything, an area
spell at a free hex too). **The computer's Ice, Magma, Storm and Energy Elementals never cast.**
The values come from the spell context (the threats each stack meets this turn, the predicted run
of the side) and 36 per-spell evaluators (§7.46): mostly `(√g − 1) × strength × the rounds'
share`, with the balance class as the battle's expected length.

**Not read yet** (T1a6, before T5): the hero's cast choice 0x43C800 with its candidates and the
retreat test 0x41E570; part A, the adventure AI's routines they call (6.4 KB), as far as the
battle needs them. **The neutral side's map is complete**: a side without a hero never reaches
them (no hero's cast; the retreat test answers for a hero's side only).

**Difficulty and `human`** (the side's player is human: auto-combat, a hidden battle):

| From | What it adds (a `human` side behaves as Hard in each branch marked *) |
|---|---|
| Easy | no shooter placement in the tactics phase (Wait); no Wait; no retaliation in the predictions for a computer side; a negative value never drops an enemy; the plan's value always added |
| Normal | the enemies' reach in the hex scores |
| Hard* | Wait when nothing else acts; the walk holds back from a worse-scored hex; the area threat and breath lines in the scores; the last retaliation's gain; the step to safety; the multi-headed reach for a computer side only |
| Expert, Impossible | nothing further found |

**What it reads:** the whole battle (the enemy hero's spells, mana and power for the protections'
and the odds' values; the stacks' wounds, effects and retaliations left; the walls, the moat, the
Fire Walls and the side's own stops). It runs in the trusted host (ADR-0017 point 4).

**Its draws** (all the battle's generator, 0x50B230): the walker's 75–100 % per candidate
(0x421B02), the wall shot's choice among the weakest walls (0x41EFFB), the scan's skip (0x420D98),
the retreat test (0x41E5BC) and the cast choice (0x43CA60; both T1a6); the predicted walkers draw
their candidates' values too (§7.42), and the rules' own draws inside the casts (the drawn spells,
Chain Lightning's next target).

## 2. The sources compared

| Area | The reading (code) | Wiki | VCMI 1.7.5 | Verdict |
|---|---|---|---|---|
| Surrender | a computer side never surrenders; its retreat test's yes is a retreat (0x47815A, §7.34) | "Same applies for enemy AI heroes, which however never surrender." (Combat) | the adventure AI's answer only retreats (`AIGateway::makeSurrenderRetreatDecision`) | **agree**: no row |
| Retreat | the test 0x41E570 is unread (T1a6) | "In most cases, AI will retreat based on its estimation of state of the battle, and value of the hero"; never on Easy, "only 50% chance to consider retreating on medium"; never for a "defeat hero X" hero or without a tavern town; always from a hopeless siege (four conditions) (Combat) | strength thresholds of the adventure AI | T1a6 reads the test against these |
| Wait | none below Hard (0x421F80's end; the walk's hold-back Hard and up) | Easy: "do not wait" (Difficulty level); Normal not mentioned | waits whenever its evaluation prefers it, at every difficulty | Easy agrees; **Normal at risk** (row W1) |
| Enemy reach | the enemies' reach in the scores from Normal; holding back from Hard | Hard: "takes into account the range of the enemy troops… does not run into the range of the movement if he knows that the enemy creature is attacking him" | a threat map (`ThreatMap.cpp`) at every difficulty | Hard agrees; **Normal at risk** (W2) |
| Tactics phase | a computer's shooter is placed only above Easy; walkers take the walker's choice inside the zone | Easy: "do not use creature tactics" | ends its tactics phase at once (`yourTacticPhase`) | Easy agrees; the placement a row for the hero's side (H1) |
| Target choice | value per turn from the computer's worth (Fight Value based, §7.31), a 75–100 % draw per candidate | stack value "used by the AI, to an extent, to determine a target to attack" (AI/Fight Value) | simulated exchanges, no draw | consistent in kind; the draw at risk (W3) |
| Creature casters | five casters; the elementals never cast | the Genie casts "a random, beneficial spell" and "will not cast spells that have no effect" (the rules' candidate set, §7.21); nothing on the computer's choice | every caster creature evaluates its spell (`findBestCreatureSpell`), elementals included; the Genie's spell drawn by weight | **the elementals at risk** (C1); the casts' values (C2 to C5) |
| The hero's spells | every spell outside the 36 evaluators takes the empty one (§7.44), Force Field and Quicksand among them; how the cast choice values the damage spells is T1a6's | Force Field, Quicksand: "AI cannot cast this spell" | casts any spell by simulation | agree so far (T1a6 confirms) |
| The moat | the scores avoid it under SoD rules only (BAT-SIE-022) | AB and RoE readmes: AI-controlled monsters now avoid the moat | — | agree; the rules version at risk (S3) |
| Sieges | the Cyclops batters the weakest wall when locked-out strength outweighs its best shot; a besieger without a target walks to its wall; a defender may hold the walls | "It is very rare that the AI stays in the castle entirely" (Ballistics, fan text) | — | rows S1, S2, S4 |
| Paths | the original's pathfinder (built, BAT-MOV-003, T1c) | — | its own | done (batch 3c's M2, G4) |
| War machines, turrets | built (BAT-WAR-008, BAT-SIE-020 to 023; batches 3f, 3h) | War Machine, Arrow tower pages | — | done |

## 3. Readings at risk and the planned batch rows

A reading is at risk against the wiki or VCMI, at a float boundary, or on run-time `.TXT` values
(Q57). Each gets one row; the specs come with the tasks that build them (batch **4a** for the
neutral side, T2 to T4; **4b** for a computer hero's side, T5 and T6). Rows are played on Wine by
an agent where the harness allows (`experiments/README.md`), else by the user.

| Row | The reading (where) | Set-up | Under the reading | Otherwise |
|---|---|---|---|---|
| 4a W1 | no Wait below Hard (§7.34) | a neutral walker walled in by obstacles, no enemy it can reach, on Normal and on Hard (two maps) | Defend on Normal, Wait on Hard | a Wait on Normal |
| 4a W2 | the enemies' reach from Normal (§7.36) | a neutral walker between two hexes, one inside an enemy's reach, on Easy and on Normal | Normal avoids the reached hex, Easy does not | both alike |
| 4a W3 | the walker's 75–100 % draw (§7.35) | two targets whose values differ by under a quarter, the same battle replayed with ten seeds | both targets chosen across the seeds | always the same |
| 4a W4 | holding back on Hard (§7.38) | a neutral walker two turns from its target whose path crosses an enemy's reach, Hard | it stops short and waits | it walks into the reach |
| 4a W5 | the several-turn comparison undivided (§7.35) and the Harpies' larger value | a near weak and a far strong target; Harpies beside it | as the reading's value per turn | another order |
| 4a C1 | the elementals never cast (§7.42) | neutral Storm Elementals beside an own stack they could protect, an enemy shooter on the field | no Protection cast | a cast |
| 4a C2 | the Faerie Dragon casts whenever its spell is worth anything; an area spell at a free hex (§7.43) | neutral Faerie Dragons beside an attackable enemy, ten seeds | a cast at every damage spell drawn, a free hex when it pays | an attack instead |
| 4a C3 | the raise against the walker's value (§7.43) | neutral Archangels with a dead own stack and an enemy in reach | as the values compare | always raise |
| 4a C4 | the Ogre Mages' skip and strength floor (§7.44) | neutral Ogre Mages beside a weaker and a stronger own stack, an enemy in reach, twenty seeds | about three in ten skip; only the stronger stack is blessed | other rates or targets |
| 4a C5 | the Genie's mean (§7.44, §7.46) | neutral Master Genies with a strong own stack and an enemy in reach, twenty seeds | casts at the rate the mean predicts | another rate |
| 4a S3 | the moat's scores under SoD rules only (§7.35) | a neutral town's garrison sallying past the moat, on an RoE and an SoD map | the SoD map's walkers avoid the moat | both or neither |
| 4a S4 | the walls test's wrong records (§7.35) | a neutral garrison behind a destroyed mid-upper wall | it holds the walls | it sallies |
| 4a P1 | S-11: the berserk walk ends on the target's position only (§7.41) | a berserk walker whose two-hex target is reachable only by its other hex | as BAT-QUIRK-24 on | the other hex |
| 4b H1 | the shooter's tactics placement (§7.34) | a computer hero with Tactics and Archers, Normal | the hex with the fewest free neighbours | no move |
| 4b S1 | the wall shot's 3.0 and its draw (§7.35) | a computer Cyclops besieger with much of its side locked out | the weakest wall, drawn among equals | a stack |
| 4b S2 | the besieger walks to its row's wall (§7.35) | a computer besieger without a target in reach | before the moat in its row | elsewhere |
| 4b H2 | the hero's value in the odds: the table's thresholds, the maximum mana, the run-time AI values (§7.39) | after T1a6: a computer hero at a threshold | as read | another branch |
| 4b H3 | the hero's cast considered at every activation (§7.34) | after T1a6: a computer hero who declines at the first stack and casts later in the round | a cast after the first activation | only at the first |

Not rows: the order of more than 16 stacks (the library's sort, §7.37: ported as read, ADR draft);
the tie orders (ported; batch 3c's M2 and G4 agree); everything the property tests and the
replays of the built rules already cover (the walks' paths, the automatic machines).

## 4. The original's slips in its own estimates

The reading found places where the computer's estimate departs from the rule it estimates: the
walls test's wrong records (§7.35); the multi-headed reach's untested sign (§7.36); the desperate
kills' top-creature term (§7.38); the protections' caster bonuses from the own hero and the cost
from the caster's own army, Magic Mirror's doubled chance, Anti-Magic's helpful spells and level,
the morale and luck rise from below 0, Paralyze's Blind record and resistance, Disease's
resistance to Weakness, Frenzy without its value, the exchange's shot flag, the removals'
Disrupting Ray (§7.46). They decide the computer's choices, never a rule's outcome: a command the
policy picks is validated and played by the engine's rules. The ADR draft proposes porting them as
read, each named in its rule's text, without quirk flags (the user decides at the review).

## 5. For the ADR draft

The crate and its layer; the surface `h3-battle` gives it (one implementation of the computer's
numbers the rules already use); the function, its stream and its fallback; the walker's path (a
move carries a destination: the engine's walk is the original's path, T1c); the cost budget; the
rule ids; the data the policy needs beyond today's packs (the spells' AI values from `SPTRAITS.TXT`
for the hero's value, which the neutral side reads for the enemy hero: §7.38, §7.39).
