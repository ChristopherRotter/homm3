# E23 — the original's battle AI: survey (T1b)

The survey behind E23's seam (T1b, 2026-10-20): what the original's computer does in a battle, as
read; where the wiki and VCMI stand; the readings at risk with a planned batch row each; the
original's slips in its own estimates; and the questions ADR-0027
(`docs/adr/0027-original-battle-ai.md`, accepted 2026-10-20) settles. The detail of every routine stays
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
anti-magic garrison; then the spell context, the retreat test and the cast choice (§7.55). Then
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

**The hero's cast** (§7.55, T1a6). Over the spells the hero has, can pay and may cast there
(Cursed Ground, Recanter's Cloak), each valued by its evaluator (§7.46) or a case of its own (the
area spells at every hex, Chain Lightning's first enemy, the mass damage spells, the raises and
Sacrifice, Earthquake for a besieger, Teleport as the walker's gain, the summons by AI value), ×
2.5 with mana for 7 casts, else × √(casts left), then a 75–100 % draw: the best is cast only when
its "now" says it is due at this activation. Quicksand, Land Mine, Force Field, Fire Wall, Berserk
and Remove Obstacle are never cast. With the enemy beaten and no own creature at risk, only the
raises; when the side would retreat, a damage spell first, always, in the desperate mode.

**The retreat test** (§7.55). Never without a hero, on Easy, on half the Normal draws, with the
Shackles of War on either side, for the victory condition's hero, without a tavern town to hire
the hero back from (a town's defender only through the Escape Tunnel). Then a besieger walled
out (gate closed, the four walls up, no flyer, Cyclops or shooter, every defender inside)
retreats; a hero worth less than 1000 in artifacts with under 2000 experience does not; a side
the predictions leave without a creature retreats; one without gold for the surrender price and
2500 does not; else it retreats when its share of both sides' Fight Value (× 1.2 for the stacks
yet to act, the other side × 1.1, a town's defender × 1.1) is below about a tenth to a sixth (the
artifacts, the difficulty, 200 000 experience and the attacker's role move it, capped at 0.16).
Part A, the adventure AI's value of the hero's artifacts, decides only the worth's thresholds;
the test reads the adventure's state, which no seam carries (BAT-Q61). **The neutral side's map is
complete**: a side without a hero never reaches either.

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
the retreat test (0x41E5BC, Normal only) and the cast choice (0x43CA60, per spell with a value;
§7.55); the predicted walkers draw
their candidates' values too (§7.42), and the rules' own draws inside the casts (the drawn spells,
Chain Lightning's next target).

## 2. The sources compared

| Area | The reading (code) | Wiki | VCMI 1.7.5 | Verdict |
|---|---|---|---|---|
| Surrender | a computer side never surrenders; its retreat test's yes is a retreat (0x47815A, §7.34) | "Same applies for enemy AI heroes, which however never surrender." (Combat) | the adventure AI's answer only retreats (`AIGateway::makeSurrenderRetreatDecision`) | **agree**: no row |
| Retreat | 0x41E570 (§7.55): never on Easy, a 50 % draw on Normal, never for the victory condition's hero or without a tavern town, a walled-out besieger always (gate, four walls, no flyer, catapult or shooter, defenders inside); the hero's worth (artifacts, experience), the gold and the share of Fight Value | "In most cases, AI will retreat based on its estimation of state of the battle, and value of the hero"; never on Easy, "only 50% chance to consider retreating on medium"; never for a "defeat hero X" hero or without a tavern town ("or besieged town is the last one with the tavern"); "AI will always retreat from failed siege if: AI is the attacker, and town has Fort; Drawbridge is up and there are no destroyed wall sections; AI has no flyers, shooters, or catapult; All defending units are inside town walls" (Combat, "AI Retreat", 2026-10-22) | strength thresholds of the adventure AI | **agree** on every term, the siege's four conditions word for word (the wiki's shooters are those able to shoot now); the threshold, its cap and float steps at risk (R1) |
| Wait | none below Hard (0x421F80's end; the walk's hold-back Hard and up) | Easy: "do not wait" (Difficulty level); Normal not mentioned | waits whenever its evaluation prefers it, at every difficulty | Easy agrees; **Normal at risk** (row W1) |
| Enemy reach | the enemies' reach in the scores from Normal; holding back from Hard | Hard: "takes into account the range of the enemy troops… does not run into the range of the movement if he knows that the enemy creature is attacking him" | a threat map (`ThreatMap.cpp`) at every difficulty | Hard agrees; **Normal at risk** (W2) |
| Tactics phase | a computer's shooter is placed only above Easy; walkers take the walker's choice inside the zone | Easy: "do not use creature tactics" | ends its tactics phase at once (`yourTacticPhase`) | Easy agrees; the placement a row for the hero's side (H1) |
| Target choice | value per turn from the computer's worth (Fight Value based, §7.31), a 75–100 % draw per candidate | stack value "used by the AI, to an extent, to determine a target to attack" (AI/Fight Value) | simulated exchanges, no draw | consistent in kind; the draw at risk (W3) |
| Creature casters | five casters; the elementals never cast | the Genie casts "a random, beneficial spell" and "will not cast spells that have no effect" (the rules' candidate set, §7.21); nothing on the computer's choice | every caster creature evaluates its spell (`findBestCreatureSpell`), elementals included; the Genie's spell drawn by weight | **the elementals at risk** (C1); the casts' values (C2 to C5) |
| The hero's spells | the cast choice 0x43C800 (§7.55): never Quicksand, Land Mine, Force Field, Fire Wall, Berserk or Remove Obstacle; a cast only when due now | Force Field, Quicksand: "AI cannot cast this spell" | casts any spell by simulation | agree on the two; **Berserk, Land Mine, Fire Wall and Remove Obstacle at risk** (H4) |
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
| 4a C1 | the elementals never cast (§7.42; built in both profiles; slip 25 lets `strict-fixed` cast, BAT-Q57) | neutral Storm Elementals beside an own stack they could protect, an enemy shooter on the field | no Protection cast | a cast |
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
| 4b H3 | the hero's cast considered at every activation, cast only when due now (§7.34, §7.55) | a computer hero whose best spell helps a stack that acts later in the round (Haste aside) | a cast at that stack's activation, not at the first | at the first |
| 4b H4 | the hero never casts Berserk, Land Mine, Fire Wall or Remove Obstacle (§7.55) | a computer hero with only Berserk (then only Fire Wall) and the mana, enemies in a cluster, ten seeds | no cast | a cast |
| 4b R1 | the retreat's share of Fight Value, its cap at 0.16 and the attacker's 0.06 (§7.55) | a computer hero defending on Hard at 15 % and at 17 % of both sides' Fight Value (artifacts worth over 1000, gold enough, a tavern town) | retreats at 15 %, fights at 17 % | other thresholds |

**Played** (E23 T7, 2026-10-23; run by an agent, `experiments/agent/ai.py`; batches 4a and 4c for
the neutral side, 4b for a computer player's hero, Tan; the results in `experiments/results/`):
W1 agrees (a Citadel's walker holding its walls: Defend on Normal, Wait on Hard, four rounds
each); W3 agrees (a Gold Dragon between two equal stacks: both taken across ten seeds, 6 and 4);
C1 agrees (no elemental's cast in five rounds); C2 agrees (a cast at all 8 dragon turns, Fireball
and Frost Ring at the free hex between two stacks 4 of 4); C3 agrees (ten dead Peasants left for
an attack, two Champions raised); C4 agrees (on Normal no walker's value: a cast at 6 of 6 turns
as the port; on Easy 3 skips in 21 turns at one field across seeds, every cast at a 10-Champion
stack, the later record first); S4 agrees (a destroyed mid-upper wall leaves the garrison holding
to round 9 in three runs, a destroyed mid-lower wall sends it out at once; slip 2); H1 **against
the port, not the rule** (the Archers go to hex 15, the corner of column 15: the original's
neighbour table, battle manager +0x13468, has no neighbours in the side columns 0 and 16; the
port counted them and went to hex 9; E23 T7 aligned the policy's neighbours with the table,
BAT-Q62 the rules' one use that differs, a breath's second hex); H3 agrees (Bless at the Archers' own activation, none at the
Wolf Riders' before); H4 agrees (none of six never-cast spells in four rounds); R1 agrees (a
retreat at 0.143; a fight at 0.172, then a retreat at the second activation at 0.158, the first
stack no longer × 1.2). S1 and S2 were not reached (batch 4c SC, Red in auto combat: Red's
Catapult breached a wall before the Cyclops' first turn; afterwards the Cyclops shot the
Archers, as read). S-25 (BAT-Q33): a neutral side's and a computer player's stacks are let off
one freeze in four (28 of 105, 14 of 60), a human's never. Not staged: W2, W4, W5 and P1
(positions the formations and the tile's obstacles do not give), C5 (the Genie's mean on the
original's state), S3 (an RoE map), H2 (no single choice shows it).

Not rows: the order of more than 16 stacks (the library's sort, §7.37: ported as read, ADR draft);
the tie orders (ported; batch 3c's M2 and G4 agree); everything the property tests and the
replays of the built rules already cover (the walks' paths, the automatic machines).

## 4. The original's slips in its own estimates

Places where the computer's estimate reads a wrong record, argument, spell, side, mastery or test
compared with the rule it estimates. They decide the computer's choices, never a rule's outcome: a
command the policy picks is validated and played by the engine's rules. A simplification (an
expectation in place of a roll, a resistance as a factor, the draws) is no slip. **The user
(2026-10-20, BAT-Q55): one fix-optional flag for all of them**, BAT-QUIRK-25
(`quirk.battle.original_ai_slips`): on in `faithful`, the slips as read; off in `strict-fixed`,
each estimate as the rule it estimates:

| # | The slip (where) | Fixed |
|---|---|---|
| 1 | the area shot's estimate ignores fire immunity and counts a Lich's cloud on the lifeless stacks that are not undead (§7.35; built, BAT-AI-003, E23 T2b) | the shot's rule's immunities (BAT-RNG-004) |
| 2 | the walls test reads the wrong records for the walls and the gate (§7.35; built, BAT-AI-011, E23 T3c: the Upper Wall's record reads a display byte, §7.49) | each part's own hex |
| 3 | the multi-headed reach's value is subtracted with its sign untested (§7.36; built, BAT-AI-007, E23 T3b) | only a positive value, as the other branch |
| 4 | the desperate kills count the top creature's remaining hit points where its lost ones belong (§7.38; built, BAT-AI-009, E23 T3c) | the lost hit points |
| 5 | the destination map marks a two-hex enemy's position hex twice, its other hex never (§7.38; E23 T5b2a: no choice reads a mark but at a position, §7.58, so nothing is built) | both hexes |
| 6 | the hero's value takes the mastery on the hero's adventure tile (§7.39; built, BAT-AI-008, E23 T6: the attacking hero's own ground `BattleSetup.attacker_ground`, ADR-0027 point 14, BAT-Q56; the defender's tile is the battle's) | the battlefield's |
| 7 | the Faerie Dragon's estimate: Expert on its school's field, the side's hero's bonuses on its bolts (§7.43; built, BAT-AI-013, E23 T4a) | the rules' mastery for a creature, no hero bonus |
| 8 | the Ogre Mage's Bloodlust valued at Expert on Fiery Fields, the Master Genie's spells on their school's field (§7.44, §7.51, §7.53; built, BAT-AI-014, E23 T4b1; BAT-AI-016, E23 T4c) | Advanced, as the rules cast it |
| 9 | the protections take the caster bonuses of the context's own hero and the caster's own army for the cost (§7.46; built, BAT-AI-016, E23 T4c) | the casting hero's, the target side's army |
| 10 | Magic Mirror keeps `2 × (50 − c)` % (§7.46; built, BAT-AI-016, E23 T4c) | `100 − c` % |
| 11 | Anti-Magic's estimate removes helpful spells too and counts level `v` (§7.46; built, BAT-AI-016, E23 T4c) | harmful spells below `v`; spells below `v` |
| 12 | a morale or luck rise from below 0 is valued as if it crossed 0 (§7.46; built, BAT-AI-016, E23 T4c) | `−c × N` while below 0 (`N` < 0) |
| 13 | Paralyze is valued with Blind's record and resistance (§7.46; built, BAT-AI-016, E23 T4c) | its own (74) |
| 14 | Disease asks the resistance to Weakness (§7.46; built, BAT-AI-016, E23 T4c) | its own (73) |
| 15 | Frenzy is valued as a second strike, without its value (§7.46; built, BAT-AI-016, E23 T4c) | 0x437800 with its attack gain less 0x438910 with the defense it loses |
| 16 | the exchange gives B's attack and kill values A's shot flag (§7.46; built, BAT-AI-016, E23 T4c) | B's own |
| 17 | the removals count Disrupting Ray, which none ends, and Cure removes every harmful spell (§7.46; built, BAT-AI-016, E23 T4c) | the spells each rule ends (§7.10, §7.13) |
| 18 | the Fire Wall's hex scores value its damage without the spell's immunity test (§7.48; built, BAT-AI-007, E23 T3b; found in T3b) | only a wall whose spell hurts the stack (BAT-MAG-028's test) |
| 19 | the Fire Wall's and the moat's hex scores value the damage with the least attack bonus for both bonuses (§7.48; built, BAT-AI-007, E23 T3b; found in T3b) | the least attack and the least defense bonus, as every other value |
| 20 | the odds value the turrets at creature 2's AI value (the Archers), not the turret's own (§7.38, §7.49; built, BAT-AI-008, E23 T3c; the user's call, 2026-10-21) | the turret creature's AI value |
| 21 | guarding a shooter defends when the walker stands on the guard hex of the last shooter asked, not the best's (§7.49; built, BAT-AI-010, E23 T3c; found by the blind re-read) | the best's guard hex |
| 22 | after a creature cast entry built a spell context and cast nothing, the besieger's walk and the fallbacks read the pathfinder as the context left it: the last Berserk or Hypnotize value's search, else the threat records' last (§7.52, §7.53; built, BAT-AI-005, BAT-AI-010, E23 T4c; BAT-Q59; a walk there without a path keeps the last action, BAT-STUB-11) | the walker's own map |
| 23 | the threat tables' seed never moves to the next record: every stack's melee entry gets the side's first record's seed (§7.53; built, BAT-AI-016, E23 T4c; found in T4c) | each record's own |
| 24 | a removal values each ended spell on a lone copy of the stack's record, which the evaluators walking the battle's records never meet by pointer: Disrupting Ray 0, no first strike for Slow, the battle's record of the stack (every spell on it) the nearest of Berserk's targets and one of Hypnotize's, a two-hex shooter's copy blocked by its own record where either is berserk or the copy hypnotized, an ended Mirth, Sorrow, Fortune or Misfortune still in the copy's morale or luck (§7.53, §7.54; built, BAT-AI-016, E23 T4d; found by T4c's blind check, BAT-Q60) | the stack itself |
| 25 | the computer's Ice, Magma, Storm and Energy Elementals never cast their protection (§7.42, §7.50; the user, 2026-10-22, BAT-Q57: a slip; built, BAT-AI-002, BAT-AI-012, BAT-AI-014, E23 slip 25) | they take the Ogre Mage's scan with the protections' estimates (§7.46); the Storm and Ice Elementals, shooters, also before their shot, against its value (the user, 2026-10-22: our own design, the original has no casting shooter) |
| 26 | the hero's cast choice reads the planned damage and the threat records as the retreat test's predictions left them, when the test got that far: both sides' runs, each side's records as the last predicted turn that wrote them left them (a walker's of its own side, a shot's of the other side, §7.33), until Teleport's evaluation fills them again; the threat tables and "the enemy beaten" keep the context's, filled at its build (§7.55; T1a6; built, BAT-AI-024, E23 T6) | the context's own (the side's run, the records it filled) |
| 27 | the hero's Expert Dispel is valued at the other side's sum alone: its case sums both sides, then falls into the stacks' case, whose mass evaluations overwrite the value with the side's sum and then the other side's (§7.56; built, BAT-AI-017, E23 T5a; found by T5a's blind check) | the side's sum and the other side's |
| 28 | Sacrifice values the raised target's worth with the victim's shot flag (0x43B06F asks 0x4428F0 of the victim's record; §7.57; built, BAT-AI-019, E23 T5b1; found in T5b1) | the target's own, as the raises' |
| 29 | Earthquake tests whether the walls halve a shooter's shot from the shooter's position, where the rule's halving tests from the hex it turns to: a two-hex shooter's differs (§7.57; built, BAT-AI-019, E23 T5b1; found in T5b1) | the rule's hex (BAT-SIE-008) |
| 30 | Teleport's destination map writes no stops, so its plan counts a turn more at a stop of the pathfinder's last search, another stack's or a spell context's that the stack's first turn's cast entry built: a destination in the moat or on a pit that stack's controller sees (§7.58, §7.59; built, BAT-AI-020, E23 T5b2a, the cast entry's context E23 T5b2b; found in T5b2a) | none: a teleported stack enters no hex and acts from it on its next turn (BAT-MAG-029, BAT-SIE-004; batch 3g TV, which the user asked for, 2026-10-22) |
| 31 | on the destination map the melee value's jousting run-up is the plan's hex's cost there, 1, where a teleported stack strikes from where it stands (§7.58; built, BAT-AI-020, E23 T5b2a; found by T5b2a's blind check) | no run-up |
| 32 | the Archangel's raise test takes the active record's hit points (its count × 100 for an Archangel, else × 50), so in the hero's Teleport turns of another Archangel the active stack's (§7.59; built, BAT-AI-012, E23 T5b2b; found by T5b2b's blind check) | the raiser's own |
| 33 | Teleport's walker turns are real ones: a creature caster among its candidates may build a spell context in its cast entry (0x421280, §7.59), whose predictions replace the planned damage on every stack, and the hero's later spells read that context's (0x443080 with flag 1; found in E23 T6 from §7.42 and §7.59; built in E23 T7, BAT-AI-020) | the choice's own context's |

The ledger entry lists them; a slip found later joins the same flag. The tie orders, the library
sort (§7.37) and the draws are the original's behaviour, not slips: no flag. Not modelled, no
slip either: a Faerie Dragon's spell before its first draw in a battle, which the original reads
from its record as an earlier battle of the session left it (+0x4E0 is never initialised,
§7.59); the engine's battles start afresh, with none (E23 T5b2b).

## 5. For the ADR (ADR-0027)

The crate and its layer; the surface `h3-battle` gives it (one implementation of the computer's
numbers the rules already use); the function, its stream and its fallback; the walker's path (a
move carries a destination: the engine's walk is the original's path, T1c); the cost budget; the
rule ids; the data the policy needs beyond today's packs (the spells' AI values from `SPTRAITS.TXT`
for the hero's value, which the neutral side reads for the enemy hero: §7.38, §7.39).
