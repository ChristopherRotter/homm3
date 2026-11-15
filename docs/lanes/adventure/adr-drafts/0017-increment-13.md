# ADR-0017 amendment (draft): increment 13, creature banks (the game side)

- Status: **Draft** (E08 T118b2, 2026-11-15), for the checkpoint after T118b2 (decision 334: the
  user accepts the drafts before anything is built). Drafted from the survey
  `docs/rules/surveys/E08-banks.md` (§6 as corrected by T118b2) and the user's answers at ADV-Q52
  and ADV-Q53 (E08 decisions 333, 335 to 339) and at T118b2 (341, 342), then into ADR-0017
  through the round's `shared/` pull request with batch E13's outcome.
- Amends: ADR-0017 points 1, 2, 4, 6, 9, 10 and 11 (each text below goes in after the point's
  increment 12 part, and after E09's where a point has one); its consequences, alternatives and
  revision history. Points 3, 5, 7 and 8 are unchanged: no bank is refreshed by a day or a week,
  no seed is new, nothing simultaneous, and the env gains no action.
- Detail: `docs/epics/E08-design.md` part D (§13 to §21); the map side is `0016-increment-13.md`,
  the battle seam `0022-increment-13.md`.

## Context

Increment 13 types the creature banks (2 137 objects on 148 shipped maps, eleven kinds). A visit
asks to attack or to search; a yes fights the bank's guards at once in a battle of the original's
own setup (no tactics phase, no obstacles, no war machines on the field, a fixed layout); a won
bank gives its reward and stays looted. T118a read the reward as coming before the winner's
levels (D-67) and the user chose that order as the original's (decision 337); T118b2's re-reading
found that for a human the original gives the levels inside the battle's call, before the reward
(survey §6's correction), so the user replaced it (decision 342): the reward is a won battle's
rest, as every guarded object's.

## Decision

**Point 1** (one entry point), after increment 12's text: **Increment 13** (creature banks; E08
decision 333) adds **no command**: a bank is visited by `MoveHero`'s last step onto its entrance
(the ten entered kinds) or toward it (the Shipwreck, from the coast too) and by `Revisit` on an
entered one; its question is answered with the offers' answers, its battle with the battle's
commands, its reward's army dialog with `Close` and the management commands.

**Point 2** (actors and viewers), after increment 12's text: from increment 13 a viewer sees a
**creature bank**'s type and, by its team's visit mark, the original's hint, the adventure
observation's object view (ADR-0016 point 6); no game-level view is added. A team's new mark is
told with the hint after it.

**Point 4** (streams), after increment 12's text: **Increment 13's draws** are setup placeholders,
drawn from the setup stream when the game is built, in the second walk with the chests, the
gifts, the Shrines, the finds and the Pyramid, in object order: per bank its level by the
chances, its artifacts by class (relics first) from the one pool of the random artifacts, then
its upgrade's chance where the upgrade's slot holds a creature with an upgrade. No visit, battle
or reward of increment 13 draws. **A bank battle's seeds** are every battle's: its own seed the
attacking player's stream's next draw, its obstacles' seed as point 4 says, the generator giving
none; the original's seed of the bank's tile (170 101 x + 223 252 y + 250 037 z + 82 037) has no
counterpart, since the engine's battle draws are its own.

**Point 6** (pending decisions), after increment 12's text: **Increment 13 adds no decision kind.**
A bank's question is an offer of the visitor's owner, `Fight` for the attack kinds and `Search`
for the search kinds, routed by the object's kind when accepted (as `Fight` routes a mine's); a
yes opens the battle at once (point 11), or with no guards standing gives the reward at once. A
won bank's reward is **a won battle's rest** (GAM-DEC-007's third, `Rest::Won` of the entrance),
given once the winner's levels are answered: the creature onto the army or into the army dialog
(GAM-DEC-004), the artifacts read when given, the resources, then the loot. This is the
original's order for a human, whose levels come inside the battle's call (`adventure_battle`
0x4AE324) before `bank_battle_reward`'s reward (E08 decision 342, replacing 337; survey
`E08-banks.md` §6, D-67); every player answers as a human (point 1).

**Point 9** (versions), a line: increment 13 bumps the adventure state hash (v26: the banks'
draws and guards), the adventure ruleset part (v21: the kind) and the map import (v24: the five
classes); the game state hash, the adventure replay format (v16) and the env's action encoding
(v8) stay. The battle lane's battle state hash and battle part and the shared pack format move
with the bank battle (`0022-increment-13.md`), in its `cross-lane` task.

**Point 10** (outcomes), after increment 12's text: **Increment 13's outcomes:** a won bank's end
holds its reward as the visit's rest (`reward_after`); the reward is a reward record's outcome
(ADV-RWD-001): the creature's join or its group without room, the artifacts as hero changes, the
resources as gains, the loot the adventure's own change; a looted search's once-bonus is the
adventure's own change of the hero on the map (ADR-0013 point 1 as amended for increment 12).

**Point 11** (battles through the game), after increment 12's text: **Increment 13's battles**
(GAM-BAT-001 amended): a bank's guards defend as **`Defender::Bank`**, as they stand, unowned and
without a hero (the neutral seat's side), at the entrance's tile; the game sets the setup's
**`creature_bank`** from that defender, so the battle is a bank's by its cause, never by its tile
(a hero met on a bank's entrance fights an ordinary battle: of the original's 14 calls of
`adventure_battle`, only the bank's passes its flag). The attacker's war machines are listed as
always and set up by no one (the battle's rule, ADR-0022 point 8). A bank of a deck kind (the
Shipwreck, the Derelict Ship) fights on the sea terrain's deck, from the coast too. Its end: lost
or retreated, the survivors stay with the bank; won, the reward after the winner's levels (point
6).

**Consequences**, a line: increment 13 adds a defender kind and the setup's bank flag (no
decision kind, no rest kind, no command; the game state hash stays); its battles change the
battle lane's setup (ADR-0022 point 8, `cross-lane`); nothing changes on the wire (the events, the
offers and the views are existing ones), in the replay format or in the env's encoding.

**Alternatives considered**, a line: increment 13 (ADV-Q53, T118b2): the reward before the
winner's levels, a bank's exception to GAM-DEC-007 point 3 with the levels held until its army
dialog closes (decision 337, on the survey's first reading; replaced by 342 when the re-reading
found the original's human order the rest's); `BattleStart::bank`, a field (every literal, the
town lane's siege among them, would change); the bank battle marked by its tile; a bank offer kind
beside `Fight` and `Search`; the original's tile seed for the bank battle.

**Revision history**, a line: points 1, 2, 4, 6, 9, 10 and 11 amended for E08 increment 13,
creature banks: no command; the banks' hint in the observation; the banks' draws when the game
is built; no decision kind, the reward a won battle's rest after the levels (D-67 corrected); the
versions; the outcomes; `Defender::Bank` and the setup's bank flag by the battle's cause, the
deck. Drafted in E08 T118b2 (survey `E08-banks.md`, corrected); batch E13's outcome folded in.
