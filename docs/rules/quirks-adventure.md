# Quirks ledger: adventure lane

Known differences between the original game's behaviour and a clean design, in the adventure lane's rules (map, movement, fog of war, objects, towns; the game's turns and calendar). Classes, the decision guide and profiles are in `docs/04-rules-and-sources.md` §4; the entry template is in `docs/rules/quirks-ledger.md` (the battle lane's ledger). Ids are `ADV-QUIRK-<n>` (`docs/08-work-process.md`, "Lanes").

A candidate becomes an entry here only once it is confirmed for SoD (by an experiment, by original data, or by a code reading that a second look confirms). Until then it waits in its survey and the engine implements the clean behaviour. `exploit-fix` entries need no confirmation.

## Entries

### ADV-QUIRK-1 — The fog's fringe shows what is drawn on hidden tiles
- Class: fix-optional (the survey's class, E08 decision 7; Q30's rule: confirmed 2026-10-11, the user may object at the T8a checkpoint)
- Context / rules affected: adventure, ADV-FOG-005 (the projection, E08 T8b)
- Original behavior: the hidden tiles next to explored ones (the fringe, drawn half dark) show their terrain and the objects and creatures standing on them, though the player has not explored them; no path goes onto them (ADV-MOV-008).
- Intended/expected behavior: a hidden tile shows nothing of its content; the fringe's half-dark edge shows terrain, roads and rivers only (E08 decision 7).
- Impact: information: a player learns of objects and monsters (and presumably heroes, not asked) one tile beyond its explored area.
- Community status: known; VCMI's pathfinder notes that objects are usually visible on the fog's border.
- Sources: the user's look in SoD (2026-10-11, survey `E08-adventure-core.md` §4: terrain, objects and creatures visible, no path onto a dark tile).
- Flag: with E08 T8b (the projection), in `h3_content`'s registry (a `cross-lane` change)
- Defaults: faithful = on (terrain, roads, rivers and object appearance on the fringe), strict-fixed = off (terrain, roads and rivers only)
- Regression test: with E08 T8b
- Decided by / date: confirmed by the user 2026-10-11 (E08 T8a checkpoint answers); class as suggested

### ADV-QUIRK-2 — The diagonal exception ignores roads in its check but not in its cost
- Class: fix-optional (the user, 2026-10-11, ADV-Q3)
- Context / rules affected: adventure, ADV-MOV-006 (T7b)
- Original behavior: a diagonal step's cost depends on the hero's points left, compared with the source tile's plain terrain cost T (with Pathfinding), not with the step's own straight cost: with T ≤ points < T × √2 the diagonal costs only its straight cost, with points ≥ T × √2 it costs trunc(straight × √2). On a road the straight cost is the road's (50–75), so a road diagonal costs the road's straight cost with 100–141 points on grass but 70–106 with more, and the points left may still pay further road steps.
- Intended/expected behavior: the check compares with the step's own straight cost (VCMI `CPathfinderHelper::getMovementCost`), so the exception only helps when a full diagonal is unaffordable.
- Impact: gameplay-altering (small): an extra cheap road step at the end of a day.
- Community status: documented on the wiki (Movement, "one notable, complex and buggy exception").
- Sources: code reading 0x4B1740 (E08 T7a; `docs/rules/surveys/E08-adventure-core.md` §1.3), confirmed by an independent second reading (a fresh agent, 2026-10-10); wiki Movement (https://heroes.thelazy.net/index.php/Movement).
- Flag: `quirk.adventure.diagonal_check_ignores_roads`
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-adventure/src/movement/tests.rs` (E08 T7b)
- Decided by / date: confirmed 2026-10-10 (E08 T7a); classified by the user 2026-10-11

### ADV-QUIRK-3 — The diagonal exception forgets a native army's missing penalty
- Class: fix-optional (the user, 2026-10-11, ADV-Q3)
- Context / rules affected: adventure, ADV-MOV-004, ADV-MOV-006 (T7b)
- Original behavior: the exception's check uses the terrain's penalised cost T even when the whole army is native and the straight step costs 100; a native army on swamp (T = 175) with 100–140 points cannot make a diagonal step, which a clean check (against 100) would allow at the straight cost.
- Intended/expected behavior: the check compares with the step's own straight cost, native terrain included (VCMI).
- Impact: gameplay-altering (small): native swamp, snow, sand and rough armies lose the last diagonal of a day.
- Community status: documented on the wiki (Movement).
- Sources: as ADV-QUIRK-2.
- Flag: `quirk.adventure.diagonal_check_ignores_native`
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-adventure/src/movement/tests.rs` (E08 T7b)
- Decided by / date: confirmed 2026-10-10 (E08 T7a); classified by the user 2026-10-11

### ADV-QUIRK-4 — Neutral stacks before the first faction stack keep the native terrain
- Class: fix-optional (the user, 2026-10-11, ADV-Q3)
- Context / rules affected: adventure, ADV-MOV-004 (and the diagonal exception's check, ADV-MOV-006)
- Original behavior: the army's native terrain (0x44C590) walks the seven slots in order; each creature's native terrain is its faction's (0x643698), and a neutral creature reads the entry before the table, -1, which is also the "none yet" marker. So neutral stacks in slots before the first faction stack are skipped, and the first faction stack sets the terrain; any later stack whose terrain differs, a neutral one included, makes it none. [Peasants, Pikemen] walks grass at the native cost, [Pikemen, Peasants] does not.
- Intended/expected behavior: the penalty goes only when every creature is native (wiki Movement); a neutral stack anywhere cancels it (VCMI 1.7.5 `CGHeroInstance::getNativeTerrain`, neutral creatures native nowhere).
- Impact: gameplay-altering (small): the order of the slots decides a whole army's movement on its native terrain.
- Community status: not on the wiki's Movement page; VCMI does the clean rule.
- Sources: code reading 0x44C590 (E08 T7b; survey §1.2), confirmed by an independent second reading (a fresh agent, 2026-10-11); wiki Movement (https://heroes.thelazy.net/index.php/Movement); VCMI 1.7.5 `lib/mapObjects/CGHeroInstance.cpp` and `config/factions/neutral.json`.
- Flag: `quirk.adventure.native_skips_leading_neutrals`
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-adventure/src/movement/tests.rs` (E08 T7b)
- Decided by / date: confirmed and classified 2026-10-11 (the user, E08 T7b)
