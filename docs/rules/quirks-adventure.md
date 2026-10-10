# Quirks ledger: adventure lane

Known differences between the original game's behaviour and a clean design, in the adventure lane's rules (map, movement, fog of war, objects, towns; the game's turns and calendar). Classes, the decision guide and profiles are in `docs/04-rules-and-sources.md` §4; the entry template is in `docs/rules/quirks-ledger.md` (the battle lane's ledger). Ids are `ADV-QUIRK-<n>` (`docs/08-work-process.md`, "Lanes").

A candidate becomes an entry here only once it is confirmed for SoD (by an experiment, by original data, or by a code reading that a second look confirms). Until then it waits in its survey and the engine implements the clean behaviour. `exploit-fix` entries need no confirmation.

## Entries

`ADV-QUIRK-1` stays reserved for the fog's fringe (candidate A-1, `docs/rules/surveys/E08-adventure-core.md` §4) until it is confirmed.

### ADV-QUIRK-2 — The diagonal exception ignores roads in its check but not in its cost
- Class: fix-optional (suggested in the survey, candidate A-2; stands unless the user objects at the T6b/T7a checkpoint)
- Context / rules affected: adventure, ADV-MOV-006 (T7b)
- Original behavior: a diagonal step's cost depends on the hero's points left, compared with the source tile's plain terrain cost T (with Pathfinding), not with the step's own straight cost: with T ≤ points < T × √2 the diagonal costs only its straight cost, with points ≥ T × √2 it costs trunc(straight × √2). On a road the straight cost is the road's (50–75), so a road diagonal costs the road's straight cost with 100–141 points on grass but 70–106 with more, and the points left may still pay further road steps.
- Intended/expected behavior: the check compares with the step's own straight cost (VCMI `CPathfinderHelper::getMovementCost`), so the exception only helps when a full diagonal is unaffordable.
- Impact: gameplay-altering (small): an extra cheap road step at the end of a day.
- Community status: documented on the wiki (Movement, "one notable, complex and buggy exception").
- Sources: code reading 0x4B1740 (E08 T7a; `docs/rules/surveys/E08-adventure-core.md` §1.3), confirmed by an independent second reading (a fresh agent, 2026-10-10); wiki Movement (https://heroes.thelazy.net/index.php/Movement).
- Flag: named when T7b implements it (`quirk.adventure.<name>`)
- Defaults: faithful = on, strict-fixed = off
- Regression test: T7b
- Decided by / date: confirmed 2026-10-10 (E08 T7a); class pending the user's checkpoint

### ADV-QUIRK-3 — The diagonal exception forgets a native army's missing penalty
- Class: fix-optional (suggested in the survey, candidate A-3; stands unless the user objects at the T6b/T7a checkpoint)
- Context / rules affected: adventure, ADV-MOV-004, ADV-MOV-006 (T7b)
- Original behavior: the exception's check uses the terrain's penalised cost T even when the whole army is native and the straight step costs 100; a native army on swamp (T = 175) with 100–140 points cannot make a diagonal step, which a clean check (against 100) would allow at the straight cost.
- Intended/expected behavior: the check compares with the step's own straight cost, native terrain included (VCMI).
- Impact: gameplay-altering (small): native swamp, snow, sand and rough armies lose the last diagonal of a day.
- Community status: documented on the wiki (Movement).
- Sources: as ADV-QUIRK-2.
- Flag: named when T7b implements it
- Defaults: faithful = on, strict-fixed = off
- Regression test: T7b
- Decided by / date: confirmed 2026-10-10 (E08 T7a); class pending the user's checkpoint
