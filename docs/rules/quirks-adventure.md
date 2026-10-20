# Quirks ledger: adventure lane

Known differences between the original game's behaviour and a clean design, in the adventure lane's rules (map, movement, fog of war, objects, towns; the game's turns and calendar). Classes, the decision guide and profiles are in `docs/04-rules-and-sources.md` §4; the entry template is in `docs/rules/quirks-ledger.md` (the battle lane's ledger). Ids are `ADV-QUIRK-<n>` (`docs/08-work-process.md`, "Lanes").

A candidate becomes an entry here only once it is confirmed for SoD (by an experiment, by original data, or by a code reading that a second look confirms). Until then it waits in its survey and the engine implements the clean behaviour. `exploit-fix` entries need no confirmation.

## Entries

### ADV-QUIRK-1 — The fog's fringe shows what is drawn on hidden tiles
- Class: fix-optional (the survey's class, E08 decision 7; Q30's rule: confirmed 2026-10-11, not objected to at the T8a checkpoint)
- Context / rules affected: adventure, ADV-FOG-005 (the observation and the steps' viewers, E08 T8b1)
- Original behavior: the hidden tiles next to explored ones (the fringe, drawn half dark) show their terrain and the objects and creatures standing on them, though the player has not explored them; no path goes onto them (ADV-MOV-008).
- Intended/expected behavior: a hidden tile shows nothing of its content; the fringe's half-dark edge shows terrain, roads and rivers only (E08 decision 7).
- Impact: information: a player learns of objects, monsters and heroes one tile beyond its explored area, and sees a hero's step there.
- Community status: known; VCMI's pathfinder notes that objects are usually visible on the fog's border.
- Sources: the user's look in SoD (2026-10-11, survey `E08-adventure-core.md` §4: terrain, objects and creatures visible, no path onto a dark tile); heroes on the fringe show as creatures do (the user's choice for E08 T8b1, 2026-10-11, not looked at).
- Flag: `quirk.adventure.fringe_shows_objects`
- Defaults: faithful = on (terrain, roads, rivers, what objects cover there and the heroes standing there), strict-fixed = off (terrain, roads and rivers only, but a wandering monster: it guards the explored tiles beside it, and the guard shapes the player's paths there, so hiding it would leak it through the legal moves; E08 decision 40, the user, 2026-10-15, after the plans' view property found the leak in T20b3b1)
- Regression test: `crates/h3-adventure/src/observe/tests.rs` (E08 T8b1)
- Decided by / date: confirmed by the user 2026-10-11 (E08 T8a checkpoint answers); class as suggested; the flag with E08 T8b1

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

### ADV-QUIRK-5 — A battle's obstacles follow from its tile
- Class: fix-optional (the user, 2026-10-13, ADV-Q13: exploitable, if mildly)
- Context / rules affected: game, GAM-BAT-001 (the battle's setup from the adventure map; E08 T19a)
- Original behavior: before the battlefield and its obstacles are set up, `battle_side_setup` seeds the C runtime's generator with 110 291 x + 167 801 y + 81 569 (32-bit, the battle's tile, the level not included; 0x463A08, the coordinates from the position word at 0x4AE27C), so every battle on a tile gets the same obstacles whoever fights there and however often.
- Intended/expected behavior: the obstacles are not known before the battle starts: drawn from the attacking player's stream, server-only (E08 decision 9).
- Impact: exploit (mild): a defending hero can pick the tile it stands on for a layout that suits its army; an attacker, or an agent that has learned the tile's layout over many episodes, knows the battlefield before it commits. In the original the same is learnable by reloading, which the tile seed was probably meant to make useless.
- Community status: not on the wiki's pages read (Wandering creature, Diplomacy); VCMI reproduces it (`BattleInfo::setupBattle` seeds its `RandGen` with the same expression).
- Sources: code reading 0x463A08 (E08 T17a, survey `E08-battles.md` §4.2), re-read with the call site's arguments 2026-10-13 (E08 T17b); VCMI 1.7.5 `lib/battle/BattleInfo.cpp`.
- Flag: `quirk.adventure.obstacles_from_tile` (flag 23 in `h3_content`'s registry, E08 T19a)
- Defaults: faithful = on (from the tile; the layouts match the original's once the battle lane ports its generator), strict-fixed = off (the attacker's stream)
- Regression test: `crates/h3-game/src/tests/battles.rs` (`the_obstacles_follow_the_tile_under_adv_quirk_5`; the strict draw in `a_visit_of_another_players_hero_opens_a_battle_both_own`), E08 T19a
- Decided by / date: classified by the user 2026-10-13 (ADV-Q13; the epic's decision 26)

### ADV-QUIRK-6 — A monster's split follows from its tile
- Class: fix-optional (the user, 2026-10-13, ADV-Q13; its own flag, apart from ADV-QUIRK-5)
- Context / rules affected: adventure, ADV-MON-009 (the split into stacks; E08 T20b)
- Original behavior: `monster_battle` seeds the C runtime's generator with 248 071 x + 276 589 y + 310 111 z + 155 303 (the monster's visitable tile, 0x4AC580); the first draw moves the number of stacks by one (at most 20: one fewer; at least 80: one more), the second (at most 50) upgrades the middle stack of a town's unupgraded creature. With the armies' ratio, which picks the base number from a table, the split is a function of the tile and the two armies.
- Intended/expected behavior: the two draws come from the attacking player's stream, server-only; the table by the armies' ratio stays.
- Impact: exploit (mild): a player can work out the exact stacks before it attacks and tune its army (the ratio) to get a split it likes; a re-visit always splits the same way.
- Community status: known: the wiki says the split varies but "is not random"; VCMI reproduces the first draw exactly (`hashByPosition`) and approximates the second (D-11).
- Sources: code reading 0x4AC580 (E08 T17a, survey `E08-battles.md` §4.1), confirmed by an independent second reading (a fresh agent, 2026-10-13); wiki Wandering creature (https://heroes.thelazy.net/index.php/Wandering_creature); VCMI 1.7.5 `lib/mapObjects/CGCreature.cpp`.
- Flag: `quirk.adventure.split_from_tile` (flag 24 in `h3_content`'s registry, E08 T20b2)
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-adventure/src/split/tests.rs` (the tile's generator), `crates/h3-game/src/tests/monsters.rs`, `a_monster_fights_split_by_its_tile_and_the_system_plays_its_side`
- Decided by / date: classified by the user 2026-10-13 (ADV-Q13; the epic's decision 26)

### ADV-QUIRK-7 — A monster that survives a battle loses its upgraded stack's survivors
- Class: fix, no flag (the user, 2026-10-13, ADV-Q12; the epic's decision 23: "A-6 always fixed")
- Context / rules affected: adventure, ADV-MON-010 (after a monster's battle; E08 T20b2)
- Original behavior: after the battle `monster_battle` counts the survivors of the monster's own creature only (`army_count_of_type`, 0x44ADC0, written back at 0x4AC905), so the survivors of the stack the split upgraded (ADV-MON-009) vanish when the hero loses or retreats.
- Intended/expected behavior: every survivor counts as the monster's creature (as VCMI turns upgraded survivors back into the base creature and keeps them); the next battle splits the whole count again.
- Impact: gameplay-altering (small): a lost or fled battle shrinks the monster by the upgraded stack's survivors.
- Community status: known: the wiki ("A bug removes troops equal to the upgraded stack's size from the wandering army when a hero is defeated or retreats").
- Sources: code reading 0x4AC580 (E08 T17a, survey `E08-battles.md` §4.3, A-6); observation `experiments/results/E08-batch-3b.md` E3-4, E3-4c, E3-5 (E08 T22b: 30 Pikemen split with 7 Halberdiers left 23 Pikemen after a retreat, three times; the next battle split the 23 into 12 Pikemen and 11 Halberdiers); wiki Wandering creature (https://heroes.thelazy.net/index.php/Wandering_creature); VCMI 1.7.5 `lib/mapObjects/CGCreature.cpp` `battleFinished`.
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, strict-fixed = fixed
- Regression test: `crates/h3-adventure/src/state/tests/answers.rs`, `a_monsters_battle_leaves_its_survivors_or_removes_it_with_its_treasure`
- Decided by / date: the user, 2026-10-13 (ADV-Q12; decision 23)

### ADV-QUIRK-8 — A map's tight formation reaches no hero
- Class: fix, no flag (the user, 2026-10-20, at E08 T22a3's end, decision 46: once a code reading confirms the reset for every hero; confirmed by E08 T22b's reading)
- Context / rules affected: game, GAM-BAT-001 (each side's formation in a battle's setup); the import's formation (`docs/formats/h3m.md`, "Import"); BAT-SET-001 places the armies by it
- Original behavior: the map file gives each placed hero a formation (loose or tight), but `map_hero_setup` (0x486590), which fills every placed hero from the map's record, never writes the hero's formation bit (bit 0 of +0x48), so every hero starts loose, whoever owns it, until its player changes it on the hero screen (whose buttons clear and set the bit, 0x4DD577, 0x4DD587). Observed in batch E3c (E3-9): Blue's human hero, tight in the map, defended loose (hexes 15, 49, 100, 151, 185). Besides, the battle reads the bit only for a human player's side (0x463679): a computer player's hero always fights loose.
- Intended/expected behavior: the hero keeps the map's formation and fights in it (the engine's import and battle setup, unchanged).
- Impact: gameplay-altering (small): a map maker's tight hero fights loose until its player changes it; one shipped hero is tight (survey `E08-battles.md` §6.4).
- Community status: not found on the wiki; VCMI 1.7.5 resets every hero to loose at the start (`CGHeroInstance::initHero`, `setFormation(LOOSE)`), as the original behaves.
- Sources: code reading 0x486590, 0x463600 (0x463652 to 0x463740), 0x4DD2D0 (E08 T22b, survey `E08-battles.md` §6.4); observation `experiments/results/E08-batch-3c.md` E3-8 (E3-9's formation); VCMI 1.7.5 `lib/mapObjects/CGHeroInstance.cpp` `initHero`.
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, strict-fixed = fixed
- Regression test: `crates/h3-map/src/tests/import.rs`, `a_placed_heros_customisations_and_formation_carry_over`; `crates/h3-adventure/src/command/tests.rs`, `a_step_toward_another_players_hero_visits_it_and_starts_a_battle` (the defender's tight formation in the battle's start)
- Decided by / date: the user, 2026-10-20 (decision 46; the reading E08 T22b)
- Open with it: whether a computer player's side should fight loose whatever its hero's formation (0x463679) is decided when the game has computer players (E10) or a formation command (army management, E08 decision 48).

### ADV-QUIRK-10 — The Elementalist levels as a might class
- Class: fix-optional (the user, 2026-10-20, ADV-Q15: `faithful` keeps SoD's 6 and 4, `strict-fixed` gives the magic classes' 3 and 3; the epic's decision 55)
- Context / rules affected: heroes, HER-LVL-004 (Wisdom and a magic school due after the class's intervals; built with E08 T27); a data quirk (ADR-0007, amendment E04b T8e)
- Original behavior: `hero_secondary_offer` (0x4DAD00) takes a class's intervals from the list at 0x4DAD74: Wisdom 3 and schools 3 for the classes 1, 3, 5, 7, 9, 11, 13 and 15 (Cleric to Witch), else 6 and 4. The list stops at the RoE classes, so the Elementalist (17), a magic class, is offered Wisdom and the schools on the might classes' intervals; the Planeswalker (16) is might either way.
- Intended/expected behavior: the Elementalist levels as the magic class it is, 3 and 3 (as VCMI and HotA).
- Impact: gameplay-altering (small): an Elementalist is offered Wisdom and the schools less often; no shipped map levels an Elementalist at load (survey `E08-progression.md` §6.1).
- Community status: known: the wiki ("The only exception is the Elementalist class, to which are considered Might heroes by the levelling system"); HotA made it level as a magic class; VCMI 1.7.5 gives it 3 and 3 by its class's affinity (D-16).
- Sources: code reading 0x4DAD74 (E08 T24a1, survey `E08-progression.md` §2.4), confirmed by the second reading (§9); wiki Secondary skill (https://heroes.thelazy.net/index.php/Secondary_skill), Hero class (https://heroes.thelazy.net/index.php/Hero_class); VCMI 1.7.5 `lib/mapObjects/CGHeroInstance.cpp` (`maxlevelsToWisdom`, `maxlevelsToMagicSchool`).
- Flag: `quirk.content.elementalist_offer_intervals` (flag 28 in `h3_content`'s registry, E08 T26b), a data quirk: the `ab` pack keeps the executable's `offer_intervals = [6, 4]` and a `[[hero_class_override]]` gated by the flag gives `[3, 3]` (the harvester's gap-fill `data_quirks.class_intervals`)
- Defaults: faithful = on (6 and 4), strict-fixed = off (3 and 3)
- Regression test: `crates/h3-pack/src/tests.rs`, `the_harvested_level_data_and_the_elementalists_intervals`; `crates/h3-harvest/src/tests.rs`, `harvests_data_quirk_fixes`; the offers that read the intervals with E08 T27
- Decided by / date: classified by the user 2026-10-20 (ADV-Q15; the epic's decision 55)
