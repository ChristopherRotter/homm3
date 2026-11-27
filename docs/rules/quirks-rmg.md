# Quirks ledger: rmg lane

Known differences between the original's random map generator (GOG Complete's `Heroes3.exe`, SoD's code; E14) and a clean design. Classes and the decision guide are in `docs/04-rules-and-sources.md` §4; the entry template in `docs/rules/quirks-ledger.md`. Ids are `RMG-QUIRK-<n>` (`docs/08-work-process.md`, "Lanes").

**Profiles** (E14 decision 4): `faithful`, the default and the one the oracle checks byte for byte, reproduces every entry; `fixed` corrects the `fix-optional` ones; `exploit-fix` entries (a crash, a hang, a corrupt map) are fixed in both. How the generator's two profiles sit in the game's profiles (`faithful`, `strict-fixed`, ADR-0007) is T1c3's ADR draft; below, "fixed" is the generator's.

**Classified by the user, 2026-11-24 (E14 T1c2)**, from the quirk candidates of the survey `docs/rules/surveys/E14-generator.md` (§1, §2.1 to §2.5, §2.7, §3.1) and `docs/lanes/rmg/ORACLE.md`. **Confirmation** (`docs/04-rules-and-sources.md` §4, "Confirm before flagging"): every entry rests on one code reading; it is confirmed when the port's step reproduces the phase oracle's step dump and draw log on the corpus (the step named in the entry), `exploit-fix` entries need none. Until then the entry says "unconfirmed"; a step that does not match re-reads the entry first.

## Entries

### RMG-QUIRK-1 — Zones whose town-type bytes nobody writes
- Class: fix-optional (the user, 2026-11-24, T1c2: "model the oracle")
- Context / rules affected: step 2, `rmg_level_zones_partition` 0x53E050; `rmg_zone_generator_init` 0x5329E0 and its draw `rmg_zone_town_type_draw` 0x532A5B; survey §2.2, §3.2 (ZON)
- Original behavior: step 2 builds a temporary zone on the stack for each level's fit tests, and a water zone through operator new (0xD4 bytes) for each water point. Neither writes the town-type bytes +0x41..+0x49 before `rmg_zone_generator_init` counts them: the count decides whether 0x532A5B draws (`rand() %` the count) and which town type the zone generator gets. A water zone's +0x40, +0x84, +0x94, +0xC..+0x18 and its zone generator's +8 are not written either. Under Wine (ORACLE.md, "The phase oracle") the temporary zone drew in every corpus case that runs step 2 (18 of 21; the other three have one level and no water); water zones drew 11 or 17 times in the water-heavy cases; `l1-4p-b`'s second map in one game drew once less and differs, moved by the environment's size. **Measured** (E14 T3a, the memory record, `clients/capture/rmg/zones.txt`; ORACLE.md, "The memory record and the fixed environment"): the temporary zone's bytes are the stack's, the same in every start and environment tried, 5 to 7 town types set in each of the corpus's 29 (it draws every time); a water zone's are the heap's (pointer-like values), set in 42 of 44; the water zones' follow the size of the game's environment block in 16-byte steps, and a map moves only where a water zone's set count crosses 0 (one draw fewer); the bytes' values (not whether each is 0) and the zone generator's +8 differ between fresh starts.
- Intended/expected behavior: zeroed bytes: no town type set, no draw.
- Impact: gameplay-altering (the stream shifts from step 2 on: a different map for the same seed); the drawn town type of a water zone reaches its guards' creatures.
- Community status: unknown
- Sources: code reading (survey §2.2, quirk candidates); observation (ORACLE.md, the phase oracle's draw log; E14 T3a, the memory record)
- Flag: `quirk.rmg.zone_memory_model` (registry position 56, E14 T3c)
- Defaults: faithful = on (the **oracle model**: the bytes as a fresh start's first map holds them under the oracle, measured by the phase hooks in T3a, `zones.txt`, in the form fresh starts agree on; the model's form left open at increment 3's checkpoint for increment 4, E14 decision 11), fixed = off (zeroed, no draw). **Provisional form (E14 T3d):** whether each byte is 0, as constants in `h3-rmg` (`ZoneMemory::TEMPORARY`: town types 0BB, +0x40, +0x84 and +0x94 set; `ZoneMemory::WATER`: all nine, likewise), a trace's measured bytes in their place (`Trace::zone_memory`; the corpus test feeds `zones.txt`'s). The temporary zone's town type is never read (type 3): only its draw counts, and it draws in all 29 records. With the measured bytes step 2 equals the oracle's on 24 of 24; with the model's own on 22 (`l1-4p-b`'s and `xl2-4p-seed0`'s water zone with none set draws once less); the water zones' town types differ in 4 cases (masks 1FE, 187, 080), which reaches their guards in later steps. A water zone's +0x8 (its second town type) stays unmodelled (-1; the heap's value differs between fresh starts).
- Limits (documented, not a defect of the port): a second map in one game, another size of the game's environment and Windows' memory are out of reach; the byte-for-byte claim holds for a fresh start's first map under the oracle's fixed environment (`h3drive.py`'s `H3_FIXED_ENV` with the record's paths; `docs/lanes/rmg/TRAPS.md`, "The oracle"). The variation T1c2's and T2c's sandboxes saw in `m2-4p`'s first map was their environment's size (T3a).
- Regression test: since E14 T3d, `h3-pack-rmg`'s `the_corpus_draws_and_dumps_as_the_phase_oracle_to_step_5` (to step 2 until T3e, to step 3 until T4b) and `the_corpus_appends_the_originals_water_zones` (the measured bytes), `the_oracle_models_own_bytes_give_step_2_on_all_but_two`, `strict_fixed_zeroes_step_2s_unwritten_bytes`; `h3-rmg`'s `the_unwritten_bytes_follow_the_flag_and_the_trace`
- Decided by / date: the user, 2026-11-24 (T1c2); measured (E14 T3a); ported with the provisional form (E14 T3d); the form left open at increment 3's checkpoint (the user, 2026-11-27, E14 decision 11: the constants stay provisional until increment 4, where the water zones' town types reach their guards)

### RMG-QUIRK-2 — A river delta's x bound reads the next cell
- Class: fix-optional (the user, 2026-11-24, T1c2: the oracle model, as RMG-QUIRK-1)
- Context / rules affected: step 15, `rmg_river_delta_site_mark` 0x548A40; survey §2.5, §3.2 (RDR)
- Original behavior: the site's x is bounded by `x > width`, so x = width is accepted: the test reads the next row's first cell (deterministic) and, at the map's last cell, one cell past the cell array, whose bytes decide whether that delta site is marked.
- Intended/expected behavior: x < width; nothing past the row.
- Impact: gameplay-altering, rare (a delta site at a row's end)
- Sources: code reading (survey §2.5)
- Flag: `quirk.rmg.delta_bound`
- Defaults: faithful = on (the next row's cell as read; the cell past the array as the oracle model gives it), fixed = off
- Regression test: when step 15 is ported
- Decided by / date: the user, 2026-11-24 (T1c2); unconfirmed

### RMG-QUIRK-3 — Divisions and moduli by zero
- Class: exploit-fix (the user, 2026-11-24, T1c2)
- Context / rules affected: `rmg_zone_place` 0x53B970 with no candidate (its draw 0x53BB5F); a jitter limit of 0 (`rmg_border_jitter_draw` 0x53C0F2: a zone of scaled size 0 on a jagged border, an island zone of size 1); `rmg_junction_path_draw` for a zone of size 0; `rmg_object_choose` 0x546190's density filter for a template with no blocking or enterable cell; a terrain picture set missing a shape (water 24, rock 16 and 24) reached by slot 5; `rand() % 0` without shipyard or gate prototypes and a modulus 0 without monolith prototypes (step 6); `rmg_map_outline_clear` 0x531170 over an empty outline (a prototype whose anchor row has no blocked cell: its ring index is i mod 0; no line of the classes `rmg_map_can_place` places reaches it, E14 T4a); survey §2.2 to §2.4, §2.7.3; the dialog's teams draw `rand() % players` (0x586183) with no players (E14 T2c: `resolve_random` refuses them, RMG-SET-001)
- Original behavior: a division by zero, which nothing catches: `rmg_run` catches only the file's open failure (code 1), `random_map_generate` and its callers nothing (E14 T3b, survey §1.2); the game fails there, no map.
- Intended/expected behavior: the generation fails with a typed error naming the cause; no map.
- Impact: game-breaking (a crash) where reached; as read none is reachable with the shipped tables and objects (increment 6's census confirms it).
- Sources: code reading (survey §2.2 "Clocks, retries, limits, failures", §2.3 and §2.4 "Faults reachable only with other data", §2.7.3)
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, fixed = fixed
- Regression test: each case when its step is ported (a table or template that reaches it gives the typed error); step 1's since E14 T3c (`h3-pack-rmg`'s `step_1s_divisions_by_zero_are_refused`: `GenerateError::NoPosition`, a zone of base 50 000 whose points' squared distances wrap; `GenerateError::LayoutSpan`, two zones of base -1); step 2's jagged limit 0 since E14 T3d (`step_2s_jagged_limit_of_0_is_refused`: a zone of base 0, `GenerateError::Partition` with `PartitionFault::Limit`); step 3's since E14 T3e: an island's coast of limit 0 (`rmg_coast_jitter_draw` 0x53CE1D, a zone of scaled size below 2: `step_3s_coast_limit_of_0_is_refused`, `GenerateError::Terrain` with `TerrainFault::CoastLimit`), a shape missing from a picture set (`h3-rmg`'s `a_missing_shape_is_refused`, `PaintFault::NoPicture`; never with the shipped sets in the corpus). Step 2's other divisions cannot divide by zero: the clip tests its divisors, and no triangle is flat (the triangulation keeps -1 for one)
- Decided by / date: the user, 2026-11-24 (T1c2)

### RMG-QUIRK-4 — Unchecked reads, writes and nulls
- Class: exploit-fix (the user, 2026-11-24, T1c2)
- Context / rules affected: `rmg_border_guard_place` 0x540D60 returning colour 0 when a tent exists but no Border Guard of its colour; `rmg_path_carve` 0x5408E0 and the seer huts' create methods (0x534B90, 0x534CC0, 0x534DB0: a quest artifact of type 65 for dirt) with a null prototype (`rmg_prototype_draw` 0x546040 returns null without candidates; E14 T4a read its callers); `rmg_monolith_border_guard` 0x542B00 marking five cells without a bounds check (a monolith on the map's edge wraps to the next row or level); `rmg_connection_by_monoliths` 0x542CE0 reading the exit list at the entrance's index unchecked; `rmg_seer_hut_place` 0x54B490 reading past the artifact prototypes when none has the quest's id; survey §2.3, §2.4; `rmg_generate_map`'s positions (0x5499BF to 0x549ACD, E14 T2c): a start zone's position below -1 or above 7 indexes its two eight-byte arrays outside the frame, and starts running out (fewer distinct starts than the players, or than the players and computers) write past them, a ninth player clearing the drawn template's index at ebp-4 and every one writing a colour past +0xF00 (AB's 2SM2i(2) with four players; `PositionError`, RMG-TPL-008)
- Original behavior: memory read or written out of bounds, or a null used; reachable only with other data as read.
- Intended/expected behavior: the generation fails with a typed error.
- Impact: game-breaking (memory corruption or a crash) where reached
- Sources: code reading (survey §2.3, §2.4, "Faults reachable only with other data")
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, fixed = fixed
- Regression test: each case when its step is ported; the positions' cases since E14 T2c (`h3-rmg`'s `positions_outside_the_arrays_are_refused`, `h3-pack-rmg`'s `two_sm2i_2_gives_the_third_player_a_seventh_start`); step 2's two since E14 T3d (`PartitionFault::Outside`, `h3-rmg`'s `straight_lines_are_bresenhams_from_the_lower_x` and `a_fills_start_clipped_off_the_map_is_a_fault`): `rmg_line_draw` 0x53C220 indexes the cells unchecked, and the borders can give it a point off the map (a side's clipped end b and a later side's clipped start are never tested; survey §2.2, "Step 2 as ported"); `rmg_zone_flood_fill` 0x53D380 uses its clipped start untested. Neither occurs in the corpus. Step 3's two since E14 T3e (`h3-rmg`'s `step_3s_faults_are_refused`, `a_pinched_cell_that_cannot_be_mended_is_refused`): an island without a polygon (`rmg_island_shape` 0x53D1C0 reads its first corner from the vector's start: `TerrainFault::NoPolygon`), a pinched cell whose resolution changes nothing (`terrain_painter_finish` 0x5B72F0 would loop for ever: `PaintFault::Stuck`; as read every resolution changes a cell)
- Decided by / date: the user, 2026-11-24 (T1c2)

### RMG-QUIRK-5 — A zone without an empty cell seeds its flood with another zone's cell
- Class: exploit-fix (the user, 2026-11-24, T1c2)
- Context / rules affected: `rmg_zone_paths_connect` 0x5405D0 (steps 6 and 9); survey §2.3
- Original behavior: the fallback seed keeps the previous zone's coordinates when a zone has no empty cell, so the zone's paths are carved from a cell of another zone; reachable only with other data as read.
- Intended/expected behavior: the generation fails with a typed error (a zone that cannot hold a path).
- Impact: corrupt map (paths through a foreign zone) where reached
- Sources: code reading (survey §2.3)
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, fixed = fixed
- Regression test: when step 6 is ported
- Decided by / date: the user, 2026-11-24 (T1c2)

### RMG-QUIRK-6 — Computer teams above the computers
- Class: exploit-fix (the user, 2026-11-24, T1c2: input validation)
- Context / rules affected: `random_map_generate` 0x5860E0, the dialog's computer-teams choice (survey §1.1, row 4; §3.2 RMG-SET-002)
- Original behavior: a computer-teams count above the computers becomes the players' count (as read; whether the dialog can give it was not read).
- Intended/expected behavior: the port takes explicit choices (a Random choice is the caller's, drawn from the game's own state) and refuses a teams count outside 0..computers with a typed error.
- Impact: none known (the dialog's range not read)
- Sources: code reading (survey §1.1)
- Flag: none (validated in every profile)
- Defaults: faithful = fixed, fixed = fixed
- Regression test: E14 T2c, `SettingsError::ComputerTeams` from `resolve_random` and `Settings::check` (`h3-rmg`'s `choices_the_original_would_go_wrong_on_are_refused`, `settings_the_tab_cannot_give_are_refused`)
- Decided by / date: the user, 2026-11-24 (T1c2)

### RMG-QUIRK-7 — Border Guards on both sides with crossed keys
- Class: fix-optional (the user, 2026-11-24, T1c2)
- Context / rules affected: step 6, a connection with the border-guard flag (`rmg.txt` column 80): `rmg_border_guard_place` 0x540D60 from the connection functions; survey §2.3
- Original behavior: the guard is tried at P with its Keymaster's Tent in the other zone, then at Q with its tent in this one (gates likewise). When both succeed the passage holds two guards in a row, each one's key lying beyond the other guard: neither side passes unless another route reaches a tent.
- Intended/expected behavior: one guard per passage (the second try only when the first fails).
- Impact: gameplay-altering, possibly game-breaking (a connection shut from both sides); frequency on the corpus measured when step 6 is ported
- Sources: code reading (survey §2.3)
- Flag: `quirk.rmg.border_guard_both_sides`
- Defaults: faithful = on, fixed = off
- Regression test: when step 6 is ported
- Decided by / date: the user, 2026-11-24 (T1c2); unconfirmed

### RMG-QUIRK-8 — A shipyard's Border Guards seek their tent in the water zone
- Class: fix-optional (the user, 2026-11-24, T1c2)
- Context / rules affected: step 6, a water connection with the border-guard flag (three guards before the shipyard's entrance); `rmg_border_guard_place` 0x540D60; survey §2.3
- Original behavior: the guards' Keymaster's Tents are placed in the water zone; what a tent on water does was not read, and a guard that cannot be placed falls back to the monster.
- Intended/expected behavior: the tent in the land zone across the connection.
- Impact: gameplay-altering
- Sources: code reading (survey §2.3)
- Flag: `quirk.rmg.shipyard_guard_tent_on_water`
- Defaults: faithful = on, fixed = off
- Regression test: when step 6 is ported
- Decided by / date: the user, 2026-11-24 (T1c2); unconfirmed

### RMG-QUIRK-9 — Underground zones walled along wide connections
- Class: fix-optional (the user, 2026-11-24, T1c2)
- Context / rules affected: step 6, the zone walls (`rmg_zone_borders_close` 0x53F880); survey §2.3
- Original behavior: on level 1 every zone is walled off from its neighbours, even along a wide connection (`rmg.txt` column 79), which on level 0 leaves the border open.
- Intended/expected behavior: wide connections open on both levels.
- Impact: gameplay-altering
- Sources: code reading (survey §2.3, "The zone walls")
- Flag: `quirk.rmg.underground_wide_walled`
- Defaults: faithful = on, fixed = off
- Regression test: when step 6 is ported
- Decided by / date: the user, 2026-11-24 (T1c2); unconfirmed

### RMG-QUIRK-10 — RoE guards: creature 117 drawn uncounted
- Class: fix-optional (the user, 2026-11-24, T1c2)
- Context / rules affected: `rmg_guard_monster_create` 0x540B20 (steps 6, 8, 10); survey §2.3
- Original behavior: for RoE the creatures 0-116 are filtered and counted, but the draw walks down from 144, so creature 117 (when it has a prototype) can be drawn without being counted, shifting the choice; a creature that passes the tests without a monster prototype is counted but cannot be walked to.
- Intended/expected behavior: the draw walks the counted creatures only.
- Impact: gameplay-altering (RoE maps only)
- Sources: code reading (survey §2.3)
- Flag: `quirk.rmg.roe_guard_walk`
- Defaults: faithful = on, fixed = off
- Regression test: when the guards are ported (an RoE case)
- Decided by / date: the user, 2026-11-24 (T1c2); unconfirmed

### RMG-QUIRK-11 — RoE clears Fortress, not Conflux, from the monster towns
- Class: fix-optional (the user, 2026-11-24, T1c2)
- Context / rules affected: `rmg_template_zones_read` 0x538480; the guards' creatures (`rmg_guard_monster_create` 0x540B20); survey §2.1, §3.2 RMG-TPL-003
- Original behavior: for RoE the town types clear the ninth (Conflux), but the monster towns clear byte +0x9D (column 65, Fortress) instead of +0x9E: an RoE zone's Fortress monster setting is lost.
- Intended/expected behavior: clear +0x9E (Conflux, AB's town) for RoE.
- Impact: gameplay-altering (RoE maps: no Fortress guards from a zone's monster setting)
- Sources: code reading (survey §2.1)
- Flag: `quirk.rmg.roe_monster_town_byte`
- Defaults: faithful = on, fixed = off
- Regression test: the templates since E14 T2c (`h3-rmg`'s `a_kept_templates_zones_are_renumbered_and_its_connections_filtered`, `h3-pack-rmg`'s `roe_zones_lose_a_monster_town_by_the_flag`, both profiles); the guards when step 6 is ported (an RoE case)
- Decided by / date: the user, 2026-11-24 (T1c2); flag at registry position 52 since E14 T2c; the reading confirmed by a second look (E14 T2c, a fresh agent's re-read of 0x5386C4 to 0x538889); unconfirmed by the oracle (no RoE case yet)

### RMG-QUIRK-12 — A zone without a town type draws among four entries
- Class: fix-optional (the user, 2026-11-24, T1c2)
- Context / rules affected: step 1, the town type of a zone whose +4 is -1 (no town type set, no colour choice), from `rmg_terrain_towns` 0x682450 (four entries per terrain); survey §1.5, §2.2
- Original behavior: the count is always 4, so the padding (-1) and repeated entries are drawn: sand, snow and lava half Castle, water three quarters Castle, Conflux on rough also in RoE.
- Intended/expected behavior: the draw among the terrain's own town types only (and none of another version).
- Impact: gameplay-altering (Castle over-represented)
- Sources: code reading (survey §2.2)
- Flag: `quirk.rmg.terrain_towns_count` (registry position 57, E14 T3c)
- Defaults: faithful = on, fixed = off
- Regression test: `h3-pack-rmg`'s `a_zone_without_a_town_type_draws_its_second_by_the_flag` (both profiles, RoE's Conflux); the corpus's `the_corpus_lays_the_zones_out_as_the_original`
- Decided by / date: the user, 2026-11-24 (T1c2); confirmed by the phase oracle (E14 T3c, `layout.txt`: of the corpus's 201 zones without a town type, 55 drew -1 and 62 Castle)

### RMG-QUIRK-13 — Step 5 skips a least town step 4 never placed
- Class: fix-optional (the user, 2026-11-24, T1c2)
- Context / rules affected: steps 4 and 5, `rmg_zone_first_town` 0x544A50 and `rmg_zone_towns_place` 0x544AE0; survey §2.2
- Original behavior: step 4 places at most one town; step 5 skips the first of the first kind asked for even when step 4 failed, so a zone (for example one without a town type set, refused by the central placement) ends one least town short.
- Intended/expected behavior: step 5 skips only the town step 4 placed.
- Impact: gameplay-altering
- Sources: code reading (survey §2.2)
- Flag: `quirk.rmg.least_town_skipped` (registry position 60, E14 T4b)
- Defaults: faithful = on, fixed = off
- Regression test: `h3-rmg`'s `step_5_skips_the_least_town_step_4_never_placed_by_the_flag` (both sides, SoD's and RoE's type draws) and `h3-pack-rmg`'s `a_least_town_step_4_never_placed_comes_in_step_5_by_the_flag` (both profiles); the corpus's steps 4 and 5 (`the_corpus_draws_and_dumps_as_the_phase_oracle_to_step_5`) with the flag on
- Decided by / date: the user, 2026-11-24 (T1c2); unconfirmed: the corpus agrees with the flag on, but no corpus zone tells the two sides apart (step 4 places at the first kind asked for or at none; the corpus's two zones whose step 4 failed, RMG-QUIRK-25's, fail step 5 either way, with the same draws; E14 T4b)

### RMG-QUIRK-14 — Step 2 mixes map and template units
- Class: fix-optional (the user, 2026-11-24, T1c2)
- Context / rules affected: step 2, `rmg_zone_position_fits` 0x53AD60; survey §2.2
- Original behavior: map distances are compared with the existing zones' base sizes in template units, the temporary zone carrying a scaled size.
- Intended/expected behavior: every size scaled to the map.
- Impact: gameplay-altering (where water zones and empty points may lie)
- Sources: code reading (survey §2.2)
- Flag: `quirk.rmg.partition_units` (registry position 58, E14 T3c)
- Defaults: faithful = on, fixed = off
- Regression test: since E14 T3d, `h3-rmg`'s `the_fit_compares_template_units_by_the_flag`; the corpus's step 2 (`h3-pack-rmg`)
- Decided by / date: the user, 2026-11-24 (T1c2); confirmed by the oracle (E14 T3d: with scaled sizes `s1-2p` alone would get 12 water zones, the original none)

### RMG-QUIRK-15 — A pile without a guard monster stays unguarded
- Class: fix-optional (the user, 2026-11-24, T1c2)
- Context / rules affected: step 10, `rmg_pile_build` 0x5466E0 and `rmg_guard_monster_create` 0x540B20; survey §2.4
- Original behavior: a treasure pile whose value calls for a guard (2000 or more) but finds no monster in range is placed unguarded.
- Intended/expected behavior: the strongest monster in range below the value, or the pile not placed.
- Impact: gameplay-altering (valuable treasure unguarded)
- Sources: code reading (survey §2.4)
- Flag: `quirk.rmg.pile_unguarded`
- Defaults: faithful = on, fixed = off
- Regression test: when step 10 is ported
- Decided by / date: the user, 2026-11-24 (T1c2); unconfirmed

### RMG-QUIRK-16 — Generated Witch Huts' 16-bit skill mask
- Class: fix-optional (the user, 2026-11-24, T1c2)
- Context / rules affected: the writer, `rmg_witch_hut_object_write` 0x5340C0; survey §2.5
- Original behavior: every generated Witch Hut is written with the allowed-skills mask 0xEFDF: skills 0-4, 6-11 and 13-15 only, never Navigation (5), Necromancy (12) or any of 16-27 (`Untitled.h3m`'s eleven agree). The mask looks cut to 16 bits.
- Intended/expected behavior: every skill but Navigation and Necromancy (0x0FFFEFDF).
- Impact: gameplay-altering
- Sources: code reading (survey §2.5); observation (`Untitled.h3m`, Complete's `random_maps`)
- Flag: `quirk.rmg.witch_hut_mask`
- Defaults: faithful = on, fixed = off
- Regression test: when the writer is ported (increment 6)
- Decided by / date: the user, 2026-11-24 (T1c2); unconfirmed

### RMG-QUIRK-17 — Roads cross one-way monoliths both ways
- Class: fix-optional (the user, 2026-11-24, T1c2)
- Context / rules affected: step 14, `rmg_roads_lay` 0x548290; survey §2.5
- Original behavior: the road flood treats a one-way monolith as two-way and jumps to the far monolith's position.
- Intended/expected behavior: one-way monoliths followed in their direction only.
- Impact: gameplay-altering (minor: where roads run)
- Sources: code reading (survey §2.5)
- Flag: `quirk.rmg.roads_one_way_monoliths`
- Defaults: faithful = on, fixed = off
- Regression test: when step 14 is ported
- Decided by / date: the user, 2026-11-24 (T1c2); unconfirmed

### RMG-QUIRK-18 — A zone of town type -1 values neutral creatures
- Class: fix-optional (the user, 2026-11-24, T1c2)
- Context / rules affected: `rmg_creature_generator_value` 0x534310 (the creature and dwelling rewards' value test); survey §2.1
- Original behavior: a zone generator's town type can be -1 (step 1's draw, RMG-QUIRK-12); the neutral creatures (town -1) then pass the creature and dwelling value test as the zone's own town.
- Intended/expected behavior: neutral creatures valued as of another town.
- Impact: gameplay-altering
- Sources: code reading (survey §2.1)
- Flag: `quirk.rmg.neutral_town_value`
- Defaults: faithful = on, fixed = off
- Regression test: when step 10 is ported
- Decided by / date: the user, 2026-11-24 (T1c2); unconfirmed

### RMG-QUIRK-19 — Shipyard sites counted more than once
- Class: fix-optional (the user, 2026-11-24, T1c2)
- Context / rules affected: step 6, a water connection: `rmg_shipyard_site_fits` 0x541960 and the draw `rmg_shipyard_site_draw` 0x541D6C; survey §2.3
- Original behavior: a site found from several neighbouring cells enters the list once per cell, so it is drawn by that weight.
- Intended/expected behavior: each site once.
- Impact: gameplay-altering (minor)
- Sources: code reading (survey §2.3)
- Flag: `quirk.rmg.shipyard_sites_repeated`
- Defaults: faithful = on, fixed = off
- Regression test: when step 6 is ported
- Decided by / date: the user, 2026-11-24 (T1c2); unconfirmed

### RMG-QUIRK-20 — A pile's box one cell past its zone
- Class: fix-optional (the user, 2026-11-24, T1c2)
- Context / rules affected: step 10, `rmg_pile_place_in_zone` 0x5470D0; survey §2.4
- Original behavior: the pile's offsets run to the zone's maximum + 1 − the pile's size, so its box may end one cell past the zone's; the fit test still applies.
- Intended/expected behavior: the box inside the zone's.
- Impact: gameplay-altering (minor)
- Sources: code reading (survey §2.4)
- Flag: `quirk.rmg.pile_box_past_zone`
- Defaults: faithful = on, fixed = off
- Regression test: when step 10 is ported
- Decided by / date: the user, 2026-11-24 (T1c2); unconfirmed

### RMG-QUIRK-21 — Cosmetic: the painters' pictures, rivers under objects, the description
- Class: fix-optional, **one flag for all** (the user, 2026-11-24, T1c2)
- Context / rules affected: the terrain painter (steps 3, 6, 11, 15), the road and river painter (14, 15), step 15's rivers, the writer's header; survey §2.5, §2.7.2 to §2.7.4
- Original behavior:
  1. `terrain_painter_queue_neighbours` 0x5B50F0 tests the axes with an else-if: when N is queued S is not re-tested (E after W likewise), so a foreign neighbour the new cell has pinched may stay a 1-wide strip.
  2. The last pass's group 3 gives sand-corner shape 8 with the mirroring inverted where one side is a dirt border.
  3. `road_river_rect_clear` 0x4FA080's right column stops one row short when the cleared cell lies on the map's second-last row: that corner neighbour keeps a stale picture.
  4. Rivers test no objects or obstacles: a river can run under them.
  5. The description prints the town of the colour's index, not the town chosen, and "humans" counts the computers too.
- Intended/expected behavior: the pictures as the neighbours give them, rivers on free cells only, the description's town and count as chosen.
- Impact: cosmetic
- Sources: code reading (survey §2.5, §2.7.2, §2.7.3, §2.7.4)
- Flag: `quirk.rmg.cosmetic` (registry position 59, E14 T3c)
- Defaults: faithful = on, fixed = off
- Regression test: the painter's half (1, 2) since E14 T3e: `h3-rmg`'s `the_cosmetic_flag_queues_both_and_keeps_group_3s_mirroring` (both behaviours), the corpus's step 3 with the flag on (`faithful`); 3 to 5 when steps 14, 15 and the writer are ported
- Decided by / date: the user, 2026-11-24 (T1c2); unconfirmed

### RMG-QUIRK-22 — 8XM8's decimal zone ids (a data quirk)
- Class: fix-optional, data (the user, 2026-11-24, T1c2)
- Context / rules affected: both `rmg.txt` tables (AB's and SoD's), template 8XM8; the harvest (increment 2, `packs/ab/rmg.toml`, `packs/sod/rmg.toml`); survey §3.1
- Original behavior: six connection rows give their first zone as 5.9, 6.3, 6.7, 7.1, 7.5 and 7.9 (a filled series); `c_atoi` reads 5, 6, 6, 7, 7, 7, so human start 8 has no connection, starts 5 and 7 three, 6 two.
- Intended/expected behavior (a documented guess): every start zone has two rows to its own treasure zones in the table (1 to 17 and 18, 2 to 23 and 24, … 5 to 35 and 36), and the six rows continue it: 6, 6, 7, 7, 8, 8 (6 to 37 and 38, 7 to 41 and 42, 8 to 47 and 48).
- Impact: game-breaking for one player (a start zone without any connection) on that template
- Sources: original data (`rmg.txt`, both archives; survey §3.1)
- Flag: `quirk.content.rmg_8xm8_zone_ids` (the pack keeps the values as read and an override gated by the flag, ADR-0007 amendment E04b T8e)
- Defaults: faithful = on, fixed = off
- Regression test: `h3-pack-rmg`'s `eight_xm8s_zone_ids_follow_the_flag` (the packs' rmg part in `faithful` and `strict-fixed`, AB's tables and SoD's) and its `original_data` check (`--with-original-data`: `strict-fixed` differs from the text tables by these six ids only); the override is the gap-fill's `data_quirks.rmg_zone_ids`, harvested into `packs/ab/rmg.toml` (E14 T2b, 2026-11-25)
- Decided by / date: the user, 2026-11-24 (T1c2); original data, confirmed

### RMG-QUIRK-23 — A start zone without a colour town takes a neutral main town
- Class: no flag (the user, 2026-11-24, T1c2: as the original, in every profile)
- Context / rules affected: steps 4 and 5 (`rmg_zone_first_town` 0x544A50, `rmg_zone_towns_place` 0x544AE0) and the writer's player block (`rmg_write_header` 0x549CB0); the game's start (`main_town_heroes` 0x4CA040); TWN-SET-001 (the player's main town, the town lane); survey §3.3.4
- Original behavior: a colour's main town is its start zone's first town placed (zone generator +0x30), written at the town's entrance as the editor writes it. When the zone asks for no town of its colour (SoD's and AB's Dragons, zone 32: purple's computer start with neutral densities only), that town is neutral: purple's main town is a neutral town. The original starts such a map (observed on `l2-2p-c4-islands`, T1c2): the town stays neutral, purple's generated hero stands on its entrance as its visitor, purple owns no town; `main_town_heroes` matches the town by its entrance without an owner test.
- Intended/expected behavior: as the original (no fix chosen: a `fixed` variant would be a map the original never makes, and the map is playable as written).
- Impact: gameplay-altering (that colour starts without a town, a hero at a neutral town's gate)
- Community status: unknown
- Sources: code reading (survey §2.5, §3.3.4); observation (the oracle's `l2-2p-c4-islands`, the game's memory after the start; T1c2)
- Flag: none (the rule in every profile)
- Defaults: faithful = original, fixed = original
- Regression test: steps 4 and 5 (E14 T4b): `h3-pack-rmg`'s `a_start_zone_without_colour_towns_takes_a_neutral_first_town` (`l2-2p-c4-islands`: purple's first town a neutral castle, by step 5's densities, the corpus's comparison); the writer's half when it is ported (increment 6); the game's start on such a map waits for the town lane (its inbox: whether `GameSetup::validate` accepts a main town of another owner, as the original does)
- Decided by / date: the user, 2026-11-24 (T1c2); confirmed (observation)

### RMG-QUIRK-24 — A start zone without Ownership marks a phantom human start 7
- Class: fix-optional (the user, 2026-11-25, E14 T2c)
- Context / rules affected: `rmg_generate_map` 0x549930's positions (0x5499BF to 0x5499F9, RMG-TPL-008); AB's tables (also SoD's, which keep them): 2SM2i(2)'s human starts 3 and 5, 2SM0k's computer starts 3 to 8, whose Ownership (column 13) is empty; survey §2.1, §3.1
- Original behavior: a start zone's position is its Ownership less 1, -1 for these. The zones' pass marks `human[p]` and `start[p]` (two eight-byte arrays at ebp-0x14 and ebp-0xC); for -1 its `start[-1]` is `human[7]` and its `human[-1]` a byte of the colours' list, which is written over next. So any such zone, human or computer start, adds a human start at position 7, which no zone has: with 2SM2i(2) and three players the third player's colour goes to position 7 and the two zones without Ownership stay without a colour (position -1 reads +0xEE0, -1); with four the starts run out (RMG-QUIRK-4).
- Intended/expected behavior: a zone without Ownership marks no start.
- Impact: gameplay-altering (a player without a start zone; what steps 4 and 5 and the writer then give that colour and those zones is read with their steps)
- Community status: unknown
- Sources: code reading (E14 T2c, `rmg_generate_map` 0x5499A1 to 0x549ACD; a second look by a fresh agent's re-read); observation (E14 T3a, the oracle's corpus cases `s1-3p-2sm2i2`, `s1-2p-2sm0k`, `s1-1p-c1-2sm0k`, `docs/lanes/rmg/ORACLE.md`): with 2SM2i(2) and three players the original writes a map whose description says three humans and whose player block has two, red and blue with their main towns, tan (the third) neither playable nor with a town; 2SM0k with two players, and with one and a computer, gives both their main towns (the phantom start 7 unused there)
- Flag: `quirk.rmg.unowned_start_seventh` (registry position 53)
- Defaults: faithful = on, fixed = off
- Regression test: `h3-rmg`'s `a_start_without_ownership_marks_the_eighth_human_start`, `h3-pack-rmg`'s `two_sm2i_2_gives_the_third_player_a_seventh_start` (both profiles)
- Decided by / date: the user, 2026-11-25 (E14 T2c); confirmed by the oracle (E14 T3a: the third player without a start zone is left out of the written map)

### RMG-QUIRK-25 — A zone whose fill finds no free cell keeps only its border cells
- Class: fix-optional (the user, 2026-11-27, increment 3's checkpoint, E14 decision 11)
- Context / rules affected: step 2's fill, `rmg_zone_flood_fill` 0x53D380 and its tail 0x53D640 (RMG-ZON-013); steps 4 on, which place a zone's towns, mines and treasures in its cells; survey §2.2
- Original behavior: the fill starts at the zone's centre (or the cell left of it when the centre is not free); when both lie on borders, the zone gets no cell from its fill, only its border cells (T3b's corner case). In the corpus 20 zones in 6 of the 24 cases (T3d's count): 13 water zones and 7 land zones (`xl1-4p-seed1`'s zone 3, `xl2-8p-t4-islands`' zone 4, five of `xl2-4p-seed0`'s).
- Intended/expected behavior: a zone's region filled from a free cell of it, so that its objects have room.
- Impact: gameplay-altering for a land zone (its towns, mines and treasures on its border cells only); a water zone's cells are painted water either way. **T4b:** both corpus land zones that are start zones get no town in the original (steps 4 and 5 equal the oracle's): `xl1-4p-seed1`'s zone 3 (human start 4, 133 border cells) and `xl2-8p-t4-islands`' zone 4 (human start 5, 225), two human players without a town
- Sources: code reading (survey §2.2, T3b); observation (the phase oracle: the port, equal to the oracle on step 2, counted the cases, T3d; on steps 4 and 5, T4b)
- Fix (E14 T4b, survey §2.2 "Steps 4 and 5 as ported (T4b)"): with the flag off, a fill whose start and the cell left of it are not free starts at the free cell of the level nearest the zone's centre (squared distance, the first row by row) outside the free regions in which the later zones' fills start (their start, or its left cell); none, no fill. The regions of the zones filled before are no longer free, so a zone never takes another zone's region; an orphan region (no zone's fill starts in it) goes to the nearest such zone
- Flag: `quirk.rmg.fill_start_on_border` (registry position 61, E14 T4b)
- Defaults: faithful = on (the original's border cells only), fixed = off (the fix above)
- Regression test: `h3-rmg`'s `a_fill_whose_start_lies_on_borders_starts_from_a_free_cell_by_the_flag` (both sides, a claimed region left alone, no free cell left), `h3-pack-rmg`'s `a_zone_whose_start_lies_on_borders_gets_its_region_by_the_flag` (both profiles: `strict-fixed` gives both start zones their region and colour's town); the faithful side is the corpus's steps 2 to 5 (`the_corpus_draws_and_dumps_as_the_phase_oracle_to_step_5`)
- Decided by / date: the user, 2026-11-27 (increment 3's checkpoint)

### RMG-QUIRK-26 — A one-way monolith connection's entrances share their channel's exits
- Class: fix-optional (the user, 2026-11-27, E14 decision 12)
- Context / rules affected: step 6, `rmg_connection_by_monoliths` 0x542CE0 and `rmg_zone_connections_make` 0x543240's index (RMG-CON-017); the game's channels (ADV-TEL-002, -003); survey §2.3
- Original behavior: a one-way connection takes entrance and exit prototypes of one index, so one channel, and places an entrance and an exit in each zone: each entrance leads to either exit, its own zone's among them. The index wraps after the two-way and one-way prototypes (AB 6, SoD 16), so a later connection shares a channel's monoliths with an earlier one.
- Intended/expected behavior: each entrance leads to the other zone.
- Impact: gameplay-altering (a hero may come out in the zone he left; channels mixed across connections)
- Sources: code reading (survey §2.3, E14 T4a)
- Flag: to be registered with its port (`quirk.rmg.*`, T4d)
- Defaults: faithful = on (as read), fixed = off (the fix designed in T4d: each direction its own channel where one is free)
- Regression test: when RMG-CON-017 is ported (T4d)
- Decided by / date: the user, 2026-11-27 (E14 decision 12)

## Considered, not quirks

Reproduced in every profile, no flag (the user, 2026-11-24, T1c2: group E of the classification, and the oracle's memory bits). A later reading that shows one changes a map's gameplay re-opens it here.

| Candidate | Why not a quirk | Survey |
|---|---|---|
| `rmg_map_cell_reset` 0x530F10 leaves +0x14, +0x18, bits of +0x24, +0x28, +0x2C to memory | no map changed by them; the port zeroes them (a fresh start's first map holds 0) | ORACLE.md |
| The terrain painter's cell-word fill takes the map interface's address | never read before a load | §2.7.1 |
| `rmg_object_choose` draws a prototype per candidate generator before the choice | the stream's order, no other effect | §2.4 |
| The river floods draw per neighbour examined, a cell popped again drawing again | the stream's order | §2.5 |
| Shipyard and gate prototypes drawn before any site is sought | the stream's order | §2.3 |
| An unmirrored shape 4 keeps picture 0 without a draw | the stream and the stored picture, as read | §2.7.4 |
| Repainting redraws every painted cell; a pinched cell drawn twice; each finish re-fits the whole map; a ring gap's own-terrain diagonal redrawn | the stream's order; the pictures as the rules give | §2.7.2, §2.7.3 |
| `terrain_painter_set_cell` 0x5B4B20 never erases its own cell from set 1 | no other effect found | §2.7.2 |
| Mine sites' and the density filter's running best (the scan order decides) | the selection rule as it is | §2.4 |
| `rmg_connection_by_land`'s shortcut for a least cost of 1 | dead (entering costs are 10 or more) | §2.3 |
| `rmg_zone_candidates_filter`'s free-level test skips index 0 | no effect (index 0 lies on an occupied level) | §2.2 |
| Islet candidates not restricted to the zone's cells | harmless (the others are seeds at 0) | §2.3 |
| The distance flood ignores what the cells hold | the distance as defined | §2.1 |
| A tent or quest artifact taken off without delete leaves its template written | an unused template in the file | §2.4 |
| `rmg_obstacle_fill_from`'s SoD test refuses none of the 45 types | no effect | §2.4 |
| A seer hut's creature reward of another town valued -1334, not -1 | equivalent: every band's minimum is 0 or more in both tables (§3.1) | §2.1 |
| Guards per way: one per land passage and shipyard, two for gates and monoliths, each of the full value | design | §2.3 |
| Mines and treasures index the guard tables by the zone's strength plus the map's, connections by the map's alone; strength none gives no guard | design | §2.4 |
| Seer hut quests: treasure-class artifacts 0-143 only; under 20 left, no seer hut | design | §2.4 |
| A Border Guard's pile and a seer hut never go to a neighbouring zone while a farther one fits | design | §2.4 |
| Underground, every cell neither free nor holding an object becomes rock (no obstacles there) | design | §2.4 |
| Every map bans artifacts 127 and 128 besides the combination artifacts | design | §2.5 |
| Prisons draw heroes below 145 for AB and SoD (SoD's campaign heroes never) | design | §2.1 |
| A main town written at the town's entrance (`Untitled.h3m`'s towns two cells right of it) | the format's convention: the editor writes the same (553 of 553 shipped main towns), the reader expects it | §2.5, §3.3.4 |
| `rmg.txt`: connections whose four positions are empty (never kept), treasure zones with an Ownership and colour towns, zones without terrain, monster town or bands | the tables' data as the game reads them | §3.1 |
| T3b: `rmg_segment_clip` clips a side's line, not the side: a side wholly off the map whose line crosses it gives a point on the map's edge (the borders' first side and edge walk stop at it; step 2's water–water connection takes it); T3d: 104 times in 19 of the 24 corpus cases | the original's geometry, harmless and accepted (the user, 2026-11-27, increment 3's checkpoint, E14 decision 11): every profile follows it, no flag | §2.2 |

## Open (classified when read)

| Candidate | What decides | Survey |
|---|---|---|
| Both 6LM10 templates connect zone 13 to itself | what step 6 makes of a self-connection (a fault would be RMG-QUIRK-3's or -4's class; T4d) | §3.1 |
| Start zones with Ownership 0 (position −1: 2SM2i(2)'s human starts 3 and 5, 2SM0k's computer starts 3-8; their positions' mark is RMG-QUIRK-24): the colour table's slot 0. **T4b (ported, equal to the oracle):** steps 4 and 5 read the colour at +0xEE0 for position −1, which `rmg_generate_map` clears to −1 and nothing writes, so a colour town there would be neutral; but none of these rows asks for a town of its colour: 2SM2i(2)'s start 3 and 2SM0k's 3 and 4 get no town, 2SM2i(2)'s start 5 and 2SM0k's 5 to 8 their one least neutral town each (a village of their drawn town type) | what steps 4 and 5 give such a zone (the writer leaves the colour at position 7 out of the map: T3a's `s1-3p-2sm2i2`, RMG-QUIRK-24): the user's classification | §3.1 |
| T3b: the borders' edge walk draws straight to the re-entry point whenever it shares x or y with the current point, also across the map: a cell whose one side on the map is axis-aligned gets a straight chord over its jagged side, in both zones' ids; the cells between are reached by no fill (water on level 0, rock on level 1, as read). **T3d: never in the corpus** | cosmetic or a fix, once a census shows it (increment 7's census, E14 decision 11) | §2.2 |
| T3b: a cell with no side on the map (meant for one holding the whole map) gets the whole frame drawn in its id, also a water zone's cell wholly off the map. **T3d: never in the corpus** | whether such a cell occurs (a census) (increment 7's census, E14 decision 11) | §2.2 |
| T3b: a point inserted where a vertex already is is dropped: the second zone at one point draws the first's cell again and has none of its own. **T3d: never in the corpus** | whether two of step 2's points can coincide (a census) (increment 7's census, E14 decision 11) | §2.2 |
