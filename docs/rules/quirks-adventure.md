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
- Original behavior: the map file gives each placed hero a formation (loose or tight), but the placed hero's setup (`placed_hero_setup` 0x4D8B30; read as 0x486590, `carried_hero_setup`, a campaign's carried-over hero's, until E08 T24a2a: survey `E08-progression.md` §3.1, which found the same on the placed hero's path), which fills every placed hero from the map's record, never writes the hero's formation bit (bit 0 of +0x48), so every hero starts loose, whoever owns it, until its player changes it on the hero screen (whose buttons clear and set the bit, 0x4DD577, 0x4DD587). Observed in batch E3c (E3-9): Blue's human hero, tight in the map, defended loose (hexes 15, 49, 100, 151, 185). Besides, the battle reads the bit only for a human player's side (0x463679): a computer player's hero always fights loose.
- Intended/expected behavior: the hero keeps the map's formation and fights in it (the engine's import and battle setup, unchanged).
- Impact: gameplay-altering (small): a map maker's tight hero fights loose until its player changes it; one shipped hero is tight (survey `E08-battles.md` §6.4).
- Community status: not found on the wiki; VCMI 1.7.5 resets every hero to loose at the start (`CGHeroInstance::initHero`, `setFormation(LOOSE)`), as the original behaves.
- Sources: code reading 0x4D8B30 (survey `E08-progression.md` §3.1; first read at 0x486590), 0x463600 (0x463652 to 0x463740), 0x4DD2D0 (E08 T22b, survey `E08-battles.md` §6.4); observation `experiments/results/E08-batch-3c.md` E3-8 (E3-9's formation); VCMI 1.7.5 `lib/mapObjects/CGHeroInstance.cpp` `initHero`.
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, strict-fixed = fixed
- Regression test: `crates/h3-map/src/tests/import.rs`, `a_placed_heros_customisations_and_formation_carry_over`; `crates/h3-adventure/src/command/tests.rs`, `a_step_toward_another_players_hero_visits_it_and_starts_a_battle` (the defender's tight formation in the battle's start)
- Decided by / date: the user, 2026-10-20 (decision 46; the reading E08 T22b)
- Open with it: whether a computer player's side should fight loose whatever its hero's formation (0x463679) is decided when the game has computer players (E10) or a formation command (army management, E08 decision 48).

### ADV-QUIRK-9 — A level's draws follow the hero's seed byte and the level
- Class: fix-optional (the user, 2026-10-20, ADV-Q15, D-14: `faithful` reseeds the original's generator per level, `strict-fixed` draws from the owner's stream; the epic's decision 55)
- Context / rules affected: game, GAM-HER-002 (the level seed and where a level's draws come from; E08 T29); heroes, HER-LVL-002 (the original's generator, E08 T27); the levels at load (GAM-SET-002) and a level-up in play (GAM-DEC-002, E08 T30)
- Original behavior: `hero_level_up` (0x4DA720) reseeds the C runtime's `rand` for every level with 214 013 L + 156 823 S + 154 079 (0x4DA801 to 0x4DA830), L the new level and S the hero's seed byte (+0x8F), which `new_game_heroes` draws once per hero with `random_between(1, 255)` (0x4BFEC2) and no code of the level-up writes. A level's primary skill and offers are a function of S, L, the class, the hero's skills and the two offer counters: reloading a game, or reaching the level by another gain, gives the same offers, and whoever knows S predicts every level.
- Intended/expected behavior: unclear; a fixed sequence per hero is how the wiki states the rule. For self-play it makes the offers predictable from a hidden byte: `strict-fixed` draws a level from the owner's stream in play and from the setup stream at load.
- Impact: gameplay-altering in what can be predicted (the offers follow S); harmless to the state. S is server-only in every profile (ADR-0017 point 5): never in an observation or event.
- Community status: known: the wiki ("The random seed for offering skills is separate for every hero and is fixed at the start of the scenario", https://heroes.thelazy.net/index.php/Secondary_skill; survey `E08-progression.md` §5.1); VCMI 1.7.5 draws from a randomizer per hero type seeded from the game's generator at its first use, which repeats on a reload but not when other draws change its order (survey §2.6).
- Sources: code reading (E08 T24a1, confirmed by T24a2b2; survey `E08-progression.md` §2.2, §9); the wiki and VCMI as above. Observed in SoD by batch E4 (E08 T33; `experiments/results/E08-batch-4a.md` and its runs, `-4b.md`): the seed bytes read from the game's memory gave every level of 66 heroes, at load and in play.
- Flag: `quirk.adventure.level_up_from_seed` (flag 27 in `h3_content`'s registry, E08 T26b), read by the game (E08 T29)
- Defaults: faithful = on (the original's draws), strict-fixed = off (the streams)
- Regression test: `crates/h3-heroes/src/tests/level.rs`, `batch_e3c_e3_6_levels_a_death_knight_from_its_seed` (the generator); `crates/h3-game/src/tests/levels.rs`, `in_faithful_the_levels_at_load_follow_the_heros_seed` and `in_strict_fixed_the_levels_at_load_draw_from_the_setup_stream` (the game's source, E08 T29)
- Decided by / date: the user 2026-10-20 (ADV-Q15; the epic's decision 55)

### ADV-QUIRK-10 — The Elementalist levels as a might class
- Class: fix-optional (the user, 2026-10-20, ADV-Q15: `faithful` keeps SoD's 6 and 4, `strict-fixed` gives the magic classes' 3 and 3; the epic's decision 55)
- Context / rules affected: heroes, HER-LVL-004 (Wisdom and a magic school due after the class's intervals; built with E08 T27); a data quirk (ADR-0007, amendment E04b T8e)
- Original behavior: `hero_secondary_offer` (0x4DAD00) takes a class's intervals from the list at 0x4DAD74: Wisdom 3 and schools 3 for the classes 1, 3, 5, 7, 9, 11, 13 and 15 (Cleric to Witch), else 6 and 4. The list stops at the RoE classes, so the Elementalist (17), a magic class, is offered Wisdom and the schools on the might classes' intervals; the Planeswalker (16) is might either way.
- Intended/expected behavior: the Elementalist levels as the magic class it is, 3 and 3 (as VCMI and HotA).
- Impact: gameplay-altering (small): an Elementalist is offered Wisdom and the schools less often; no shipped map levels an Elementalist at load (survey `E08-progression.md` §6.1).
- Community status: known: the wiki ("The only exception is the Elementalist class, to which are considered Might heroes by the levelling system"); HotA made it level as a magic class; VCMI 1.7.5 gives it 3 and 3 by its class's affinity (D-16).
- Sources: code reading 0x4DAD74 (E08 T24a1, survey `E08-progression.md` §2.4), confirmed by the second reading (§9); observed in SoD by batch E4's E4-3 (E08 T33): twelve Elementalists levelled to 3 in play, none offered Wisdom outright; wiki Secondary skill (https://heroes.thelazy.net/index.php/Secondary_skill), Hero class (https://heroes.thelazy.net/index.php/Hero_class); VCMI 1.7.5 `lib/mapObjects/CGHeroInstance.cpp` (`maxlevelsToWisdom`, `maxlevelsToMagicSchool`).
- Flag: `quirk.content.elementalist_offer_intervals` (flag 28 in `h3_content`'s registry, E08 T26b), a data quirk: the `ab` pack keeps the executable's `offer_intervals = [6, 4]` and a `[[hero_class_override]]` gated by the flag gives `[3, 3]` (the harvester's gap-fill `data_quirks.class_intervals`)
- Defaults: faithful = on (6 and 4), strict-fixed = off (3 and 3)
- Regression test: `crates/h3-pack/src/tests.rs`, `the_harvested_level_data_and_the_elementalists_intervals`; `crates/h3-harvest/src/tests.rs`, `harvests_data_quirk_fixes`; the offers that read the intervals (E08 T27): `crates/h3-heroes/src/tests/level.rs`, `wisdom_and_a_school_are_offered_after_the_class_intervals`
- Decided by / date: classified by the user 2026-10-20 (ADV-Q15; the epic's decision 55)

### ADV-QUIRK-11 — Levels past 74 wrap, and the experience overflows
- Class: exploit-fix (no flag: the user, 2026-10-20, ADV-Q15, A-7 always fixed as HotA's fix; the epic's decision 55)
- Context / rules affected: heroes, HER-EXP-002 (E08 T27)
- Original behavior: the thresholds beyond level 12 grow by a truncated × 1.2 in 32-bit sums: level 76's wraps negative, so from level 75's threshold (1 810 034 207) a hero is level 88 at once, then 100, 108, … 6 424; at 2³¹ − 1 experience no threshold exceeds it and the level loop never ends (the game hangs); the gain adds without a bound, so past 2³¹ − 1 the experience turns negative and the level-up lowers the hero to level 0.
- Intended/expected behavior: level 74 is the highest; the experience is at most level 75's threshold − 1 (1 810 034 206), where every gain saturates.
- Impact: state-corrupting (wrapped levels, a hang, negative experience); reachable only by huge experience (no shipped map's hero starts near it, survey §6.4).
- Community status: known: the wiki (Experience: level 88 after 75, "the final usable level is level 6424"); HotA fixed it at 74.
- Sources: code reading (E08 T24a1, confirmed by the second reading T24a2b2; survey `E08-progression.md` §1.2, §9); wiki Experience (https://heroes.thelazy.net/index.php/Experience).
- Flag: none (always fixed)
- Defaults: fixed in every profile
- Regression test: `crates/h3-heroes/src/tests/level.rs`, `experience_gives_the_originals_levels_up_to_74`, `a_gain_stops_at_the_bound_and_the_scenarios_level_limit`
- Decided by / date: the user 2026-10-20 (ADV-Q15; the epic's decision 55)

### ADV-QUIRK-12 — A level limit above 75 caps at a wrapped threshold
- Class: exploit-fix (no flag: the user, 2026-10-20, ADV-Q15, A-8 always fixed; the epic's decision 55)
- Context / rules affected: heroes, HER-EXP-003 (E08 T27)
- Original behavior: `hero_gain_experience` (0x4E33B0) computes the cap of the scenario's level limit as the level loop does; for a limit above 75 the 32-bit sums wrap, so 92 of the limits 76 to 255 give a negative cap (any gain then sets the experience to that negative number and gives no level) and the others a threshold unrelated to the limit (80: level 63's).
- Intended/expected behavior: a limit of 0 or above the highest level (74) is no limit; the cap is then the bound (ADV-QUIRK-11).
- Impact: state-corrupting (negative experience); no shipped map has a level limit (survey §6.2).
- Community status: not documented (the wiki has no page on the level limit).
- Sources: code reading (E08 T24a1, confirmed by T24a2b2; survey `E08-progression.md` §1.3, §9).
- Flag: none (always fixed)
- Defaults: fixed in every profile
- Regression test: `crates/h3-heroes/src/tests/level.rs`, `a_gain_stops_at_the_bound_and_the_scenarios_level_limit`
- Decided by / date: the user 2026-10-20 (ADV-Q15; the epic's decision 55)

### ADV-QUIRK-13 — A primary skill wraps at 128
- Class: exploit-fix (no flag: the user, 2026-10-20, ADV-Q15, A-9 always fixed; the epic's decision 55)
- Context / rules affected: heroes, HER-LVL-001 (E08 T27)
- Original behavior: a level-up adds 1 to the signed byte of the primary skill (+0x476 + skill, 0x4DA8F7), which holds the worn artifacts' bonuses too and has no upper bound: 127 + 1 becomes −128; the readers clamp to 0–99.
- Intended/expected behavior: the gain saturates at the ruleset's `primary_max` (99) on the base the engine keeps without artifacts; the effective skill (base + artifacts, clamped by its readers) is the original's at every value below the wrap.
- Impact: state-corrupting past 127 (a skill turning negative); reachable only past 99 with artifacts (no shipped map's hero starts near it, survey §6).
- Community status: known in part: the wiki (Primary skill: "The upper limit is 99. At 129 the skill value is reset to 0", which does not fit the byte's wrap at 128, survey §5.1).
- Sources: code reading (E08 T24a1, confirmed by T24a2b2; survey `E08-progression.md` §2.3, §9); wiki Primary skill (https://heroes.thelazy.net/index.php/Primary_skill).
- Flag: none (always fixed)
- Defaults: fixed in every profile
- Regression test: `crates/h3-heroes/src/tests/level.rs`, `a_level_is_checked_and_applied`
- Decided by / date: the user 2026-10-20 (ADV-Q15; the epic's decision 55)

### ADV-QUIRK-14 — Withdrawn (was: the computer's terrain shares count only the coast)
- **Withdrawn** (the user, 2026-10-22, ADV-Q22; the epic's decision 85; removed from the rules by E08 T38b). T25 read the tile flag `0x40` (+0xC) that `ai_map_terrain_shares` (0x52B9C0) tests as the map file's coast flag, so the shares would count the coast alone (ADV-Q17, decision 60: reproduced). E08 T38 found it means "no object blocks the tile": `map_tiles_load` (0x4FE220) sets it on every tile but rock, `tile_refresh_object` (0x505610) clears it where an object blocks the tile without an entrance there (survey `E08-progression.md` §10.3's correction). The original counts every tile no object blocks but rock, water included, which is no quirk: ADV-MAP-002 counts so (`crates/h3-adventure/src/state/tests.rs`, `the_terrain_shares_count_the_tiles_no_object_blocks`). The id is not reused.

### ADV-QUIRK-15 — A flight crosses unexplored tiles
- Class: always fixed for now, **provisional** (no flag: the user, 2026-10-22, ADV-Q20 and ADV-Q21, A-13: "probably fix, but gameplay impacts have to be considered (wide obstacle ranges become impassable); perhaps deserves a compromise, like allowing to route to the closest unexplored, but free hex; TBD later anyway"; the epic's decisions 75, 78). The compromise, and with it this class, is designed later.
- Context / rules affected: adventure, ADV-MOV-009 with ADV-MOV-008 (E08 T41)
- Original behavior: the pathfinder treats a tile the human player has not explored as one a flight passes over (`pathfinder_expand` 0x4B2300, the fog test at 0x4B25D4 with the blocked tiles, survey `E08-hero-movement.md` §3.1 point 3), so a flying hero's path may cross the unexplored, and its route and price depend on what lies hidden there (rock, water, the coast).
- Intended/expected behavior: moves stay on explored tiles for every hero (ADV-MOV-008): a flight never crosses an unexplored tile, and the legal moves never depend on a hidden tile (ADR-0016 point 4, ADR-0017 point 8). `AdventureState::passage` holds the row, so the compromise changes it alone.
- Impact: information (the original's flights tell a player what hidden tiles are not); gameplay: a flight over a wide obstacle range stops at its unexplored edge.
- Community status: not documented; VCMI 1.7.5 never paths through unexplored tiles (`evaluateAccessibility`).
- Sources: code reading (E08 T34a, survey §3.1, §3.2); batch E5's E5-9 (`experiments/results/E08-batch-5b.md`: the Angel Wings hero flew 13 tiles straight through an unexplored strip and arrived; the walker had no path; survey §13).
- Flag: none (fixed in every profile)
- Defaults: fixed in every profile
- Regression test: `crates/h3-adventure/src/state/tests/flight.rs`, `a_flight_never_crosses_an_unexplored_tile`
- Decided by / date: the user 2026-10-22 (ADV-Q20, ADV-Q21; the epic's decisions 75, 78); built by E08 T41

### ADV-QUIRK-16 — Withdrawn (was: a flight from an inland tile costs the fly price)
- **Withdrawn** (the user, 2026-10-22, ADV-Q22; the epic's decision 85). T34a read the tile bit +0xC 0x40 that `step_cost` (0x4B1740) tests as the map file's coast flag, so a flight would cost the fly price from every inland tile, roads included (D-24, the flag `quirk.adventure.flying_inland_price`). Batch E5's E5-4 refuted it and T38 found that the bit means "no object blocks the tile" (survey `E08-hero-movement.md` §2.1 point 5): the lesser of the fly and walking prices from every free land tile is the original's rule, which ADV-MOV-009 builds in every profile. The flag's registry position 30 was retired in E08 T41's shared part; the id is not reused.

### ADV-QUIRK-17 — The drop to 0 forgets the artifacts' flying and water walking
- Class: fix-optional (the user, 2026-10-22, ADV-Q20, A-12; the epic's decision 75)
- Context / rules affected: adventure, ADV-MOV-006 with ADV-MOV-009, ADV-MOV-010 (E08 T41)
- Original behavior: after each step the move drops the points left to 0 when they are below the cheapest straight step from the new tile (`hero_move_step` 0x48084E → `cheapest_step_from` 0x4B1880), which it prices with the hero's own fly and water-walk fields (+0x112, +0x116: the spells'), not the level 3 that `hero_step_cost` (0x4B18C0) forces for Angel Wings and Boots of Levitation. So the drop prices a walking step for a hero with the artifacts: on Cursed Ground's swamp an Expert flyer with 100 to 174 points left loses them, though its next step costs 100 (survey `E08-hero-movement.md` §2.5).
- Intended/expected behavior: the drop prices the cheapest step with the hero's flying and water-walking levels, as its steps are priced.
- Impact: gameplay, rare: only where the walking price is above the artifacts' (Cursed Ground; a blocked tile or water flown over, where the walking cheapest is 100 and the fly price at the spells' levels above it, comes with the spells); no shipped hero wears Angel Wings (survey §7).
- Community status: not documented; VCMI has its own last-step rule (D-3).
- Sources: code reading (E08 T34a, survey §2.5).
- Flag: `quirk.adventure.drop_ignores_artifacts`
- Defaults: faithful = on (the walking drop), strict-fixed = off (`Pricing::left_after` with the hero's fly and water-walk prices)
- Regression test: `crates/h3-adventure/src/movement/tests.rs`, `the_drop_forgets_flying_and_water_walking_under_adv_quirk_17`
- Decided by / date: the user 2026-10-22 (ADV-Q20; the epic's decision 75); built by E08 T41

### ADV-QUIRK-18 — A grown scouting radius waits for the hero's next step
- Class: fix-optional (the user, 2026-10-22, at T34b's checkpoint: the original's way as a fix-optional quirk; the epic's decision 77, ADV-Q21)
- Context / rules affected: adventure, ADV-FOG-003 (E08 T40), with ADV-FOG-001
- Original behavior: a hero's scouting radius grown in the day (a level-up's Scouting, the Speculum or the Spyglass put on) reveals nothing until the hero's next step, which reveals with the new radius; a hero that does not move shows nothing new at later days either. No turn start reveals: `new_day` (0x4C7FE0) calls `reveal_owned` (0x4C7C50, around everything a player owns) only after `cover_of_darkness_hide` (0x4C7BA0) hid tiles, and `next_player_turn` (0x4C6FE0) reaches no reveal; `hero_level_up`, `hero_learn_secondary_skill` and the artifact moves call none (survey `E08-hero-movement.md` §1.4, §4.2).
- Intended/expected behavior: a grown radius shows by the next day's start: the adventure's start-of-day step reveals around every hero to its owner with its radius (VCMI reveals at once for a skill, nothing for an artifact; the wiki is silent: D-26).
- Impact: information: a few tiles at the edge of a hero's view revealed one day earlier for a hero that waits.
- Community status: not documented.
- Sources: code reading (E08 T34a, corrected by T34b's re-read of `new_day`; survey `E08-hero-movement.md` §4.2); batch E5's E5-10 (`experiments/results/E08-batch-5e.md`: no tile revealed after the battle's Scouting nor at day 2's start, the radius 6's tile after the first step; the Spyglass the same; survey §13); VCMI 1.7.5 `CGameHandler::changeSecSkill`.
- Flag: `quirk.adventure.radius_waits_for_a_step`
- Defaults: faithful = on (no start-of-day reveal), strict-fixed = off (`AdventureState::start_day` reveals around every hero at each new day's start, after the calendar; on day 1 it adds nothing)
- Regression test: `crates/h3-adventure/src/state/tests/day.rs`, `a_gain_in_the_day_shows_at_the_next_turn_start_and_step`
- Decided by / date: the user 2026-10-22 (the epic's decision 77; confirmed by batch E5's E5-10, T38); built by E08 T40

### ADV-QUIRK-19 — A count of 32 768 or more passes through 16 bits
- Class: exploit-fix (no flag: state-corrupting, fixed in every profile as AGENTS.md requires; A-15, E08 decision 98; the user's cap, decisions 109, 114)
- Context / rules affected: adventure, ADV-ARM-005 and ADV-ARM-007 (E08 T50); the import's counts (`docs/formats/h3m.md`, "Import")
- Original behavior: `army_swap_slots` (0x44AD60) swaps two slots' creatures and counts but moves one of the two counts through a 16-bit register (`mov di, word ptr`, then `movsx`): from the hero screen (0x4DDB53) the emptied slot's 0, from the exchange (0x5B0ED5) the moved or the clicked stack's count, which arrives sign-extended at 32 768 or more (40 000 as −25 536). The map's 16-bit count of 32 768 or more loads as an empty slot (batch E6's T47a). Merges and splits keep 32 bits, so a long game can grow such a stack (Necromancy, joins).
- Intended/expected behavior: every count keeps its full width up to a stack's most (1 000 000 in the base pack, ADV-ARM-007): a move, a swap and the import keep it whole.
- Impact: state-corrupting (a negative or wrong count); no shipped map places such a stack (survey `E08-army.md` §7).
- Community status: not documented (the wiki's Troop stack: "unlimited quantity in game before overflow"); VCMI keeps 32-bit counts.
- Sources: code reading (E08 T44a, survey `E08-army.md` §3.5; T46's correction of the roles); batch E6's E6-5 dropped (decision 109) after T47a found the map's count of 40 000 loading as no creature.
- Flag: none (always fixed)
- Defaults: fixed in every profile
- Regression test: `crates/h3-adventure/src/army/tests.rs`, `between_two_armies_a_hero_keeps_its_last_stack_and_counts_stay_whole`
- Decided by / date: AGENTS.md (state-corrupting bugs are always fixed); the user 2026-10-23 and 2026-10-24 (decisions 98, 109, 114); built by E08 T50

### ADV-QUIRK-20 — The exchange's split keeps a creature for a single-stack hero
- Class: preserve, no flag (the user, 2026-10-23, ADV-Q24: the original's bounds in every profile; the epic's decision 98)
- Context / rules affected: adventure, ADV-ARM-003 (E08 T50)
- Original behavior: the split dialog (0x449E90) from the exchange (both flags 1) holds the target's new count to [m₂, t − m₁], m₁ = 1 when the source hero or the target hero has a single stack, m₂ = 1 when the target hero has one: so when the target hero has a single stack the source keeps one creature even if the source hero has other stacks (A-16), though a plain click moves that stack whole (survey `E08-army.md` §3.2, §3.3).
- Intended/expected behavior: as the original's outcomes: the engine's split takes the target's count from 1 to t − 1 (within a stack's most) and never empties a slot, the move covering the whole stack where the original's click allows it, so every outcome the original reaches and only those are reached. VCMI's split allows the whole stack into a single-stack target hero (D-27).
- Impact: none beyond the original's: a split never empties a slot in either.
- Community status: not documented; VCMI differs (D-27).
- Sources: code reading (E08 T44a, survey §3.3); batch E6's E6-4 (`experiments/results/E08-batch-6.md`: at the slider's maximum 1 left and 9 given, not VCMI's 10; survey §13).
- Flag: none (the original's bounds in every profile)
- Defaults: faithful = the original's, strict-fixed = the original's
- Regression test: `crates/h3-adventure/src/army/tests.rs`, `a_split_leaves_both_stacks_within_one_and_the_most`
- Decided by / date: the user 2026-10-23 (ADV-Q24; decision 98); confirmed by batch E6's E6-4 (T48); built by E08 T50

### ADV-QUIRK-21 — An RoE map's Witch Hut teaches only 14 skills
- Class: fix-optional (the user, 2026-10-26, ADV-Q28, A-21; the epic's decision 130)
- Context / rules affected: adventure, the Witch Hut's skill drawn at the build (ADV-TCH-005, E08 design §8, §9; E08 T61b)
- Original behavior: an RoE map's file holds no Witch Hut mask, and the load (0x5034BD) gives every such hut the fixed mask 0xEFBF, the editor's default (every skill but Leadership and Necromancy, 0x0FFFEFBF) cut to 16 bits: the skills 0 to 15 but Leadership and Necromancy, so an RoE hut never teaches Water Magic, Earth Magic, Scholar, Tactics, Artillery, Learning, Offense, Armorer, Intelligence, Sorcery, Resistance or First Aid (survey `E08-rewards.md` §9, D-33).
- Intended/expected behavior: every skill but Leadership and Necromancy, the editor's default and VCMI's 26.
- Impact: gameplay: 191 huts on the shipped RoE maps draw from 14 skills instead of 26.
- Community status: not documented (the wiki names the editor's default; RoE not addressed); VCMI uses the default.
- Sources: code reading (E08 T55a2, the second reading T55a4; survey §9, §13); batch E7's E7-7 (`experiments/results/E08-batch-7a-roe.md`: the fixed mask on an RoE map; survey §15, T58).
- Flag: `quirk.adventure.roe_witch_hut_mask` (registry position 37, E08 T59)
- Defaults: faithful = on (the mask 0xEFBF), strict-fixed = off (every skill but Leadership and Necromancy)
- Regression test: `h3-adventure`'s `a_witch_hut_draws_from_its_mask_less_the_bans_and_an_roe_hut_by_the_profile` (`src/state/tests/draws.rs`, E08 T61b: the build's draw in both profiles, ADV-TCH-005)
- Decided by / date: the user 2026-10-26 (ADV-Q28; decision 130); confirmed by batch E7's E7-7 (T58); the flag added by E08 T59

### ADV-QUIRK-22 — A full backpack refuses an artifact whatever the free positions
- Class: fix-optional (the user, 2026-10-27, ADV-Q30, A-25: "sounds like a bug to me"; the epic's decision 145)
- Context / rules affected: `h3-heroes`' receiving an artifact (ADR-0013 point 15.4, HER-CHG-002, HER-CHG-003; E08 T60) and the adventure's visits that give one (a chest, a Corpse, a Wagon, a Tomb, a box, an event, a monster's treasure; ADR-0016 point 4; E08 T62)
- Original behavior: the visits test the backpack's count (64 or more) before `hero_give_artifact` places anything, so a hero with a full backpack receives no artifact even with a free body position: the artifact is lost, or turned into a chest's gold, a Corpse's gold or a Wagon's resources, or left in a Tomb (survey `E08-rewards.md` §2.2, §5, §6.9, §7.4, §8.2).
- Intended/expected behavior: the artifact goes to the first position that takes it, else the backpack (HER-CHG-002); only an artifact that finds no room is lost or turned into gold or resources.
- Impact: gameplay, rare: only a hero carrying 64 artifacts.
- Community status: not documented.
- Sources: code reading (E08 T55a1 to T55a4, survey §2.2, §5, §6.9, §7.4); batch E7's E7-5, E7-6, E7-10 and E7-19 (`experiments/results/E08-batch-7a.md`, `-7b.md`: heroes with free positions and a full backpack lost or converted every artifact; survey §15, T58).
- Flag: `quirk.adventure.full_backpack_refuses` (registry position 38, E08 T59)
- Defaults: faithful = on (the count first), strict-fixed = off (the placement first)
- Regression test: `h3-heroes`' `a_full_backpack_refuses_an_artifact_in_faithful_only` (`src/tests/grant.rs`, E08 T60b: `can_receive_artifact` and `grant` in both profiles); the visits' with E08 T62
- Decided by / date: the user 2026-10-27 (ADV-Q30; decision 145); the flag added by E08 T59

### ADV-QUIRK-23 — A Shrine's or a Scholar's spell of −1 reads outside the tables
- Class: exploit-fix (no flag: an out-of-bounds read, fixed in every profile as AGENTS.md requires; A-18, E08 decision 133)
- Context / rules affected: adventure, ADV-TCH-004 and ADV-TCH-006 (the draws when the game is built, E08 T61b; the visits, T62)
- Original behavior: a Shrine's random spell is drawn by `random_shrine_spell` (0x4C95A0), which gives −1 when its level has no spell left after the marks' restart; a Scholar's placed spell may be 255; neither visit (`visit_shrine` 0x4A5610, `visit_scholar` 0x4A4DC0) tests it, so the visit reads the spell tables outside their bounds (survey `E08-rewards.md` §6.5, §9).
- Intended/expected behavior: a Shrine whose level has no spell to give holds none and refuses every visit; a Scholar whose skill or spell pool is empty gives a primary skill.
- Impact: unreachable on the shipped maps (no map bans a whole level); state-corrupting where reached.
- Community status: not documented; VCMI draws only allowed spells.
- Sources: code reading (E08 T55a2, survey §6.5, §9).
- Flag: none (always fixed)
- Defaults: fixed in every profile
- Regression test: `crates/h3-adventure/src/state/tests/draws.rs`, `random_shrines_repeat_no_spell_until_their_level_is_out` (a level without a spell) and `a_scholar_holds_its_gift_and_the_primary_skill_it_gives_instead` (every skill and spell banned); the visit's: `crates/h3-adventure/src/state/tests/rewards.rs`, `a_shrine_teaches_its_spell_to_a_hero_who_can_learn_it` (a Shrine without a spell refuses, E08 T62a)
- Decided by / date: the user 2026-10-26 (ADV-Q28; decision 133); built by E08 T61b

### ADV-QUIRK-24 — A Pyramid with every spell of its level banned reads outside its list
- Class: exploit-fix (no flag: an out-of-bounds read, fixed in every profile as AGENTS.md requires; A-20, E08 decision 133)
- Context / rules affected: adventure, ADV-TCH-008 (the draw when the game is built, E08 T61b)
- Original behavior: at a new game (`setup_map_objects` case 63, 0x4C19DE) the Pyramid draws `random_between(0, n − 1)` over the unbanned level-5 spells with a school; with all nine banned the pool is empty and the draw reads outside the list (survey `E08-rewards.md` §7.5, §9).
- Intended/expected behavior: the Pyramid holds no spell: its guards still fight, a win gives nothing but the looted mark.
- Impact: unreachable on the shipped maps (at most three of the nine banned).
- Community status: not documented.
- Sources: code reading (E08 T55a3, survey §7.5, §9).
- Flag: none (always fixed)
- Defaults: fixed in every profile
- Regression test: `crates/h3-adventure/src/state/tests/draws.rs`, `the_pyramid_holds_an_unbanned_spell_of_its_level`
- Decided by / date: the user 2026-10-26 (ADV-Q28; decision 133); built by E08 T61b

### ADV-QUIRK-25 — A University with fewer than four unbanned skills draws from a negative range
- Class: exploit-fix (no flag: an out-of-range draw, fixed in every profile as AGENTS.md requires; A-22, E08 decision 133)
- Context / rules affected: adventure, ADV-TCH-007 (the draw when the game is built, E08 T61b)
- Original behavior: `university_setup` (0x4C06F0) draws its four distinct skills each as `random_between(0, n − 1)` among the unbanned skills left; a map that bans more than 24 skills leaves the last draws a negative range (inferred, survey `E08-rewards.md` §9, §13).
- Intended/expected behavior: the University offers every unbanned skill left, fewer than four.
- Impact: unreachable on the shipped maps (at most 13 skills banned).
- Community status: not documented.
- Sources: code reading (E08 T55a2, the second reading T55a4; survey §6.8, §13).
- Flag: none (always fixed)
- Defaults: fixed in every profile
- Regression test: `crates/h3-adventure/src/state/tests/draws.rs`, `a_university_draws_distinct_unbanned_skills_and_all_when_fewer_are_left`
- Decided by / date: the user 2026-10-26 (ADV-Q28; decision 133); built by E08 T61b

### ADV-QUIRK-26 — A reward of more than seven creature stacks writes past its record
- Class: exploit-fix (no flag: an out-of-bounds write, state-corrupting, fixed in every profile as AGENTS.md requires; A-24, E08 decision 133)
- Context / rules affected: the map's import (`docs/formats/h3m.md`, "Import"; ADV-RWD-001, E08 T61a)
- Original behavior: the loader reads a Pandora's Box's or an event's creature list into a record of seven stacks without a bound, so a longer list writes past the record (survey `E08-rewards.md` §7.4, §13).
- Intended/expected behavior: the import refuses such a map (`Unsupported::RewardArmy`).
- Impact: unreachable on the shipped maps (none gives more than seven stacks).
- Community status: not documented.
- Sources: code reading (E08 T55a4, the second reading; survey §7.4, §13).
- Flag: none (always fixed)
- Defaults: fixed in every profile
- Regression test: `crates/h3-map/src/tests/import.rs`, `increment_7_bodies_import_with_their_guards_and_records` (E08 T61a: `Unsupported::RewardArmy`)
- Decided by / date: the user 2026-10-26 (ADV-Q28; decision 133); built by E08 T61a, recorded by E08 T61b

### ADV-QUIRK-27 — The 33rd once-per-hero object of a class shares the first one's mark
- Class: exploit-fix (no flag: the engine keeps no bound of the original's on visits' state, E08 decision 133; A-17)
- Context / rules affected: adventure, ADV-TCH-001 to ADV-TCH-003 (the once-per-hero gifts' marks, E08 T62a)
- Original behavior: at a new game each of the ten once-per-hero classes numbers its objects from 0 in the walk's order and the hero holds one u32 of marks per class; a visit tests and sets the bit `1 << number`, and the x86 shift counts mod 32 (the Tree masks its number to 5 bits outright, 0x4C1E6D), so the 33rd object of a class shares the first one's bit: a hero who visited one is refused by the other (survey `E08-rewards.md` §6.1, §9).
- Intended/expected behavior: a mark per object, by its index in the static map (`h3_adventure::Visits`, design §10), with no bound on how many objects of a class a map holds or a hero marks.
- Impact: unreachable on the shipped maps (at most 30 of a class, 30 Learning Stones in Back For Revenge; the wiki reads 32 as the editor's limit); a made map with more would refuse the later ones.
- Community status: the wiki (Learning stone): "There can be up to 32 learning stones in any map"; VCMI keeps a set of visitors per object.
- Sources: code reading (E08 T55a2, survey §6.1).
- Flag: none (always fixed)
- Defaults: fixed in every profile
- Regression test: `crates/h3-adventure/src/state/tests/rewards.rs`, `a_once_per_hero_gift_gives_once_and_marks_the_hero` (the mark per object and hero); the design's measurement of marks on every object, E08 T55b (design §10)
- Decided by / date: the user 2026-10-26 (ADV-Q28; decision 133); built and recorded by E08 T62a

### ADV-QUIRK-28 — A 401st box or event reads the first one's reward record
- Class: exploit-fix (no flag: the engine keeps no bound of the original's on visits' state, E08 decision 133; A-19)
- Context / rules affected: adventure, ADV-RWD-001 (the reward record), ADV-PIK-013 (Pandora's Box), ADV-OBJ-007 (events)
- Original behavior: Pandora's Boxes and events append their 228-byte reward records to one list of the map (+0x50), at most 400; a 401st is not stored and its tile's word keeps the index 0, so it gives the first record (survey `E08-rewards.md` §7.1, §9).
- Intended/expected behavior: a record per object (`Holds::Record`, `EventSetup::record` on the object, design §7), with no bound on how many boxes or events a map holds.
- Impact: unreachable on the shipped maps (at most 99 records, Myth and Legend); a made map with more would repeat the first record.
- Community status: not documented; VCMI keeps the rewards on each object.
- Sources: code reading (E08 T55a3, survey §7.1).
- Flag: none (always fixed)
- Defaults: fixed in every profile
- Regression test: `crates/h3-adventure/src/state/tests/records.rs`, `every_box_holds_its_own_record`
- Decided by / date: the user 2026-10-26 (ADV-Q28; decision 133); built and recorded by E08 T62b1
