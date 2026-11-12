# Quirks ledger: town lane

Known differences between the original game's behaviour and a clean design, in the town lane's rules (a town's record and setup, the derivations over it, the town on the map; E09). Classes, the decision guide and profiles are in `docs/04-rules-and-sources.md` §4; the entry template is in `docs/rules/quirks-ledger.md` (the battle lane's ledger). Ids are `TWN-QUIRK-<n>` (`docs/08-work-process.md`, "Lanes"); 1 to 4 are the quirk candidates T-1 to T-4 (E09 design note §19): E09 T7a1 entered 1 and 4, E09 T7b1 2 and 3 with the pool.

A candidate becomes an entry here only once it is confirmed for SoD (by an experiment, by original data, or by a code reading that a second look confirms). Until then it waits in its survey and the engine implements the clean behaviour. `exploit-fix` entries and those not reproduced need no confirmation.

## Entries

### TWN-QUIRK-1 — A random town's alignment byte past the colours
- Class: fix, no flag (the user, 2026-11-02, TWN-Q2, T-1: refused, typed; the epic's decision 12)
- Context / rules affected: town, the import of a random town (class 77; E09 T7a1), TWN-SET-002 (a random town's faction; E09 T7b)
- Original behavior: `load_town_body` (0x5019F0) reads SoD's alignment byte after the events and, for class 77, takes the chosen faction of the colour it names when that colour plays (0x5020AB). The byte is a signed index into the playable flags: a byte from 8 to 254 reads past the eight colours (up to 127 forward, 128 to 254 backward into other fields) and may take a faction from another table entry.
- Intended/expected behavior: the import refuses such a town with a typed error (`h3_map::ImportError::Alignment`, the object's index); `0xFF` is no alignment and 0 to 7 a colour.
- Impact: none on the shipped maps (the editor writes 0 to 7 or 255; no shipped map has another byte); a hand-edited map is refused rather than read past the table.
- Community status: VCMI 1.7.5's reader warns on 8 to 15 ("not as player", HotA) and on the rest ("corrupted") and keeps the town as the owner's or random (survey §2.5).
- Sources: code reading of `load_town_body` 0x5019F0 (E09 T1a1; survey `docs/rules/surveys/E09-towns.md` §2.3, §3 T-1).
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, strict-fixed = fixed
- Regression test: `crates/h3-map/src/tests/towns.rs`, `a_random_towns_alignment_is_a_colour_or_none`
- Decided by / date: the user, 2026-11-02 (TWN-Q2; the epic's decision 12); entered by E09 T7a1, 2026-11-04

### TWN-QUIRK-2 — The starting-hero pick is not reserved
- Class: fix, no flag (the user, 2026-11-02, TWN-Q2, T-2: a state-corrupting bug; the epic's decision 12)
- Context / rules affected: town, TWN-SET-005 (the pool; E09 T7b1), TWN-SET-006 (the pick), TWN-SET-008 (a random hero)
- Original behavior: `new_game_heroes` marks a human's chosen hero (0x4BFF83), the map read's `hero_pool_init` (0x4C4E30) frees it again (0x4C4E6B), and neither `load_hero_body` nor `main_town_heroes` checks a choice before placing it. Another colour's random hero, or an earlier colour's main-town draw, can take the chosen hero, which is then placed a second time for its chooser: one hero type on two tiles and in two players' lists.
- Intended/expected behavior: each player's pick is taken by its colour in the pool before any draw (`h3_game`'s `HeroPool::new`), so no other colour's draw gives it; its first random hero takes it, and a pick no random hero took goes back free after the draws.
- Impact: none on the shipped maps' imports (the import picks nothing); a start where a colour picks and another colour can draw its faction's heroes no longer places one type twice.
- Community status: VCMI 1.7.5 reserves the choices before any draw (survey §4.7).
- Sources: code reading of `new_game_heroes`, `hero_pool_init` 0x4C4E30, `load_hero_body` and `main_town_heroes` 0x4CA040 (E09 T1a2; survey `docs/rules/surveys/E09-towns.md` §4.2, §4.3, §6 T-2).
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, strict-fixed = fixed
- Regression test: `crates/h3-game/src/town/tests/pool.rs`, `twn_quirk_2_a_pick_is_reserved_before_any_draw`
- Decided by / date: the user, 2026-11-02 (TWN-Q2; the epic's decision 12); entered by E09 T7b1, 2026-11-04

### TWN-QUIRK-3 — A random hero can take a later hero's type
- Class: fix, no flag (the user, 2026-11-02, TWN-Q2, T-3; the epic's decision 12)
- Context / rules affected: town, TWN-SET-005 (the pool; E09 T7b1), TWN-SET-008 (a random hero)
- Original behavior: a placed hero or a prisoner marks its type in the pool only when its body is read (`load_hero_body`), so a random hero earlier in the file can draw it when the map allows it; the later body then overwrites the type's record (`hero_setup` by type).
- Intended/expected behavior: every placed hero's and prisoner's type is taken in the pool before the random heroes are drawn (`h3_game`'s `HeroPool::new`, whatever the allowed set), as VCMI does.
- Impact: none on the shipped maps (no placed hero's or prisoner's type is allowed on its map, survey §4.2); a map that allows one never gets that type twice.
- Community status: VCMI 1.7.5 draws the random heroes after every object is read (survey §4.7).
- Sources: code reading of `load_hero_body` and `hero_pool_init` 0x4C4E30 (E09 T1a2; survey `docs/rules/surveys/E09-towns.md` §4.2, §4.4, §6 T-3).
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, strict-fixed = fixed
- Regression test: `crates/h3-game/src/town/tests/pool.rs`, `twn_quirk_3_a_random_hero_draws_no_later_placed_heros_type`
- Decided by / date: the user, 2026-11-02 (TWN-Q2; the epic's decision 12); entered by E09 T7b1, 2026-11-04

### TWN-QUIRK-4 — The start's draws' empty cases
- Class: fix, no flag (the user, 2026-11-02, TWN-Q2, T-4: typed; the epic's decision 12)
- Context / rules affected: town, TWN-SET-001 (the player's main town; E09 T7a1), TWN-SET-007 and TWN-SET-009 (the draw and the main town's hero; E09 T7b)
- Original behavior: `main_town_heroes` (0x4CA040) looks for the town whose position equals the main town's; when none matches, the town pointer stays null and is used. A draw with no free hero at all (`hero_draw_of_faction` 0x4BB400) calls `random_between(1, 0)` and reads below the list. A faction outside 0 to 8 draws from Castle's classes.
- Intended/expected behavior: each case is typed. The import refuses a main town whose position is no town's entrance (`h3_map::ImportError::MainTown`, the colour; E09 T7a1); a draw with nothing free fails typed (E09 T7b); the faction is always one of the ruleset's with a town record (the setup refuses a player's set holding another, `h3_game::GameSetupError::Choice`; E09 T7a2).
- Impact: none on the shipped maps (all 553 main towns match a town; 156 heroes); a malformed map or an exhausted pool fails typed instead of reading memory it should not.
- Community status: VCMI 1.7.5 throws when no hero is left (survey §4.7).
- Sources: code reading of `main_town_heroes` 0x4CA040 (read whole) and `hero_draw_of_faction` 0x4BB400 (E09 T1a2; survey `docs/rules/surveys/E09-towns.md` §4.4, §4.5, §6 T-4).
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, strict-fixed = fixed
- Regression test: `crates/h3-map/src/tests/towns.rs`, `a_main_town_is_the_town_whose_entrance_the_header_names` (the import's case); `crates/h3-game/src/town/tests/pool.rs`, `the_draw_falls_back_to_every_free_type_then_fails` (the draw's, `GameSetupError::NoFreeHero`; E09 T7b1)
- Decided by / date: the user, 2026-11-02 (TWN-Q2; the epic's decision 12); entered by E09 T7a1, 2026-11-04

### TWN-QUIRK-5 — A custom set's horde leaves its dwelling standing
- Class: fix, no flag (the user, 2026-11-03, TWN-Q6, T-6; the epic's decision 29)
- Context / rules affected: town, TWN-SET-003 (the buildings at the start; E09 T6c), TWN-SIE-003 (the keep's count of a siege's turrets)
- Original behavior: `town_buildings_setup` (0x5C08C0) builds a custom set's buildings in id order through `town_build` (0x5BE930), which takes out of the standing set only what is standing when it builds. A horde (18 or 24) comes before the dwelling it supersedes (30 + its level, brought in by the horde's supersede set), so the dwelling's later build removes nothing: both stand. The same buildings built in play (the dwelling first, the horde then replacing it) leave the horde alone. The keep (`town_turret_count` 0x5BF4F0) counts one more structure in such a town, and the town screen draws both.
- Intended/expected behavior: the standing set is derived from the buildings ever built (`h3_towns::Town::standing`: those less every building an ever-built one supersedes), so a horde never leaves its dwelling standing, however it came to be built.
- Impact: gameplay-altering (small): a siege's keep has one structure more in a town whose map's custom set builds a horde (T7a's census counts the shipped maps' cases).
- Community status: unknown.
- Sources: code reading of `town_buildings_setup` 0x5C08C0 and `town_build` 0x5BE930, read whole (E09 T5b1; survey `docs/rules/surveys/E09-towns.md` §16.2, §16.5); VCMI 1.7.5 keeps one set of built buildings and no standing set (survey §16.8).
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, strict-fixed = fixed
- Regression test: `crates/h3-towns/src/setup/tests.rs`, `twn_quirk_5_a_custom_sets_horde_never_leaves_its_dwelling_standing`
- Decided by / date: the user, 2026-11-03 (TWN-Q6; decision 29)

### TWN-QUIRK-6 — A main town's hero is drawn over the town's visitor
- Class: fix, no flag (state-corrupting: two heroes on one tile, which AGENTS.md always fixes; quirk candidate T-7)
- Context / rules affected: town, TWN-SET-009 (the main town's hero; E09 T7b2b), TWN-SET-010 (a hero right of a town's entrance starts on it)
- Original behavior: `main_town_heroes` (0x4CA040) puts the main town's hero on the town's entrance (`town_hero_place` 0x5C13B0) without testing for a hero already there; a placed hero that `placed_heroes_setup` (0x4CAE10) moved onto the same entrance stays the town's visitor (+0x10, `hero_show`), the new hero is drawn over it on the same tile, and the guild's spells (0x5BE030) then go to that visitor, not to the new hero.
- Intended/expected behavior: the build refuses the setup: the adventure's check of the heroes' tiles refuses the main town's hero, whose id follows the setup's heroes, on a tile another hero stands on (`h3_game::GameSetupError::Adventure` with `SetupError::HeroTile` and `StandError::Occupied`).
- Impact: none on the shipped maps (no hero moved onto an entrance stands in a main town that generates a hero, survey §4.6; the census: all 156 maps that import build); a hand-made map with one is refused instead of starting with two heroes on one tile.
- Community status: VCMI 1.7.5 asserts that the tile is free when it places a starting hero.
- Sources: code reading of `main_town_heroes` 0x4CA040 and `placed_heroes_setup` 0x4CAE10 (E09 T1a2, read again in T7b2b; survey `docs/rules/surveys/E09-towns.md` §4.5, §4.6; `docs/exe/map.toml`).
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, strict-fixed = fixed
- Regression test: `crates/h3-game/src/town/tests/heroes.rs`, `twn_quirk_6_a_main_towns_hero_is_refused_on_a_visitors_entrance`
- Decided by / date: the agent, 2026-11-05 (state-corrupting, so fixed; the refusal proposed to the user at the task's start, the user told of it after the reading); entered by E09 T7b2b, 2026-11-05

### TWN-QUIRK-7 — A town event's bits past the map's slots
- Class: fix, no flag (a read past a table, which the engine never reproduces; quirk candidate T-9; the user, 2026-11-07, TWN-Q10, the epic's decision 51)
- Context / rules affected: town, TWN-EVT-003 (a town event's buildings; E09 T17b), TWN-SET-003 (the map's slots resolved through the faction)
- Original behavior: `town_events_apply` (0x5BFEB0) loops over the 44 masks of `building_bits` (0x66CD98, 8 bytes a building id, the loop's end its 0x160 bytes) and maps each set bit of the event through `map_building_ids` (0x6888C0) at faction × 41 + bit, a table of 41 entries a faction: the bits 41 to 43 would read the next faction's entries 0 to 2 (past the table for the Conflux, the last faction). The map has 41 slots; no shipped event sets those bits (survey `E09-construction.md` §10).
- Intended/expected behavior: a town event's buildings are its slots below the map's 41 (`h3_towns::SlotSet` drops the higher bits, `SlotSet::from_bits`, the import's reading of the six bytes), each resolved through the town's own faction (`SlotSet::buildings`); the bits 41 to 47 build nothing.
- Impact: none on the shipped maps; a hand-made map setting those bits builds nothing instead of another faction's building.
- Community status: unknown; VCMI 1.7.5 reads the event's bits through the town's faction alone (survey §9).
- Sources: code reading of `town_events_apply` 0x5BFEB0, read whole (E09 T12b; survey `docs/rules/surveys/E09-construction.md` §8.3), the loop's bound and index read again by E09 T17b (0x5BFEF9 to 0x5BFF4C).
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, strict-fixed = fixed
- Regression test: `crates/h3-towns/src/build.rs`, `twn_quirk_7_an_events_bits_past_the_maps_slots_build_nothing`
- Decided by / date: the user, 2026-11-07 (TWN-Q10; decision 51); entered by E09 T17b, 2026-11-09

### TWN-QUIRK-8 — A town's 16-bit counts wrap
- Class: fix, no flag (state-corrupting: a count past 32 767 turns negative; quirk candidate T-10; fixed by proposal, the epic's decision 73, accepted at TWN-Q12 and TWN-Q14)
- Context / rules affected: town, TWN-GRO-001 (a dwelling's growth), TWN-GRO-005 (the first creatures); the week's growth, the month's doubling, the town events' creatures and the Portal's count to come (TWN-GRO-003, -004, -007, TWN-EVT-004; E09 T25b, T26a)
- Original behavior: a town's available creatures (+0x16, fourteen i16) are added to by the week (`town_week_growth` 0x5BFDD0) and the events, and doubled by a creature's month (`new_month` 0x4C8F40), without a cap: past 32 767 they wrap negative, and the town screen and the hire read the wrapped count. `town_dwelling_growth` (0x5BFB60) sums on an i16 too. Reachable only in very long games (survey `E09-growth.md` §13: a neutral town's level 1 with a Castle grows 14 a week).
- Intended/expected behavior: every count and every growth is held at `h3_towns::MAX_AVAILABLE` (32 767): sums in wider integers, then saturated; below the bound nothing differs. The Portal's count too.
- Impact: none below the bound (every shipped map's first months); a very long game keeps its counts at the bound instead of wrapping.
- Community status: unknown (survey §13 found no report; VCMI 1.7.5 and the wiki say nothing of the bound).
- Sources: code reading of `town_dwelling_growth` 0x5BFB60 and `town_week_growth` 0x5BFDD0 (E09 T21; survey `docs/rules/surveys/E09-growth.md` §1, §2, §13).
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, strict-fixed = fixed
- Regression test: `crates/h3-towns/src/record.rs`, `twn_quirk_8_a_town_s_counts_saturate_at_the_bound`; `crates/h3-towns/src/growth/tests.rs`, `a_growth_is_held_at_the_bound`; `crates/h3-towns/src/week/tests.rs`, `a_week_s_count_is_held_at_the_bound_and_a_town_without_a_record_grows_nothing` and the month's doubling in `a_creature_s_month_doubles_its_creature_s_counts_after_the_week` (E09 T25b1)
- Decided by / date: the agent's proposal (decision 73), the user at TWN-Q12 and TWN-Q14, 2026-11-11; entered by E09 T25a, 2026-11-12

### TWN-QUIRK-9 — The Plague takes a neutral town's whole growth
- Class: fix-optional (the user, 2026-11-11, TWN-Q12, TD-14 and T-12: "fix-optional"; the epic's decision 70)
- Context / rules affected: town, TWN-GRO-004 (a town's month of growth; E09 T25b1), TWN-GRO-003 (the week a neutral town grows half)
- Original behavior: `new_month` (0x4C8F40) in the Plague takes each growing index's whole growth g (`town_dwelling_growth` 0x5BFB60, unhalved) off the count, at least 0, then halves it; the week before (`town_week_growth` 0x5BFDD0) added only g ÷ 2 to a town without an owner, so a neutral town keeps max(0, before − ⌈g ÷ 2⌉) ÷ 2 of its count before the week, an owned one half of it.
- Intended/expected behavior: with the flag off a town without an owner loses g ÷ 2 (what its week added), so every town keeps half of its count before the week.
- Impact: gameplay, small: a neutral town's creatures for the month after a Plague (a smaller population to fight or, once captured, to hire).
- Community status: VCMI 1.7.5 and the fandom page halve the count after the week for every town (refuted for the original by batch TWN-3's row 3).
- Sources: code reading of `new_month` 0x4C8F40 and `town_week_growth` 0x5BFDD0 (E09 T21; survey `docs/rules/surveys/E09-growth.md` §2, §3, §13 TD-14, T-12); batch TWN-3's row TWN-3-3 (`experiments/results/E09-batch-3a.md`, `-3a-run2.md`: N1 17 and 10 from 42 and 25 before the week; survey §17).
- Flag: `quirk.town.neutral_plague_full_growth` (registry position 40, E09 T24)
- Defaults: faithful = on (the whole growth), strict-fixed = off (half of the count before the week)
- Regression test: `crates/h3-towns/src/week/tests.rs`, `the_plague_takes_the_growth_then_halves` (on) and `twn_quirk_9_off_every_town_keeps_half_its_count_before_the_week` (off)
- Decided by / date: the user, 2026-11-11 (TWN-Q12; decision 70); the flag added by E09 T24; entered by E09 T25b1, 2026-11-12

### TWN-QUIRK-10 — A creature's month doubles the exact creature only
- Class: fix-optional (the user, 2026-11-11, TWN-Q12, TD-13 and T-13: "fix-optional flag"; the epic's decision 69)
- Context / rules affected: town, TWN-GRO-004 (a town's month of growth; E09 T25b1)
- Original behavior: `new_month` (0x4C8F40) in a creature's month doubles a growing index's count only when the index's creature is the month's: a base creature's month misses every town whose dwelling of that level is upgraded (its base index grows nothing, its upgrade's creature is another), and an upgrade's month doubles only upgraded dwellings.
- Intended/expected behavior: with the flag off the count doubles when the month's creature is either of the index's level's creatures (VCMI 1.7.5's rule): a Griffin's month doubles the Royal Griffins too.
- Impact: gameplay: every creature month with an upgraded dwelling of the month's base creature (most shipped maps' mid-game; survey §13).
- Community status: the wiki's growth page and VCMI's forum say the exact creature (an upgraded dwelling gets no bonus for its base creature's month); VCMI 1.7.5's code doubles the level.
- Sources: code reading of `new_month` 0x4C8F40 (E09 T21; survey `docs/rules/surveys/E09-growth.md` §3, §11, §13 TD-13); batch TWN-3's row TWN-3-1 (`experiments/results/E09-batch-3a.md`, `-3a-run2.md`: K1's Royal Griffins 47 kept, K2's Griffins 47 doubled to 94; survey §17).
- Flag: `quirk.town.creature_month_exact` (registry position 39, E09 T24)
- Defaults: faithful = on (the exact creature), strict-fixed = off (the level)
- Regression test: `crates/h3-towns/src/week/tests.rs`, `a_creature_s_month_doubles_its_creature_s_counts_after_the_week` (on) and `twn_quirk_10_off_a_creature_s_month_doubles_either_form_of_its_level` (off)
- Decided by / date: the user, 2026-11-11 (TWN-Q12; decision 69); the flag added by E09 T24; entered by E09 T25b1, 2026-11-12
