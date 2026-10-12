# Fix notes of worker `c` (E04b T11b, second round, list C)

Worktree `/home/claude/wt-a`, branch `battle/e04b-t11b-c` (pushed after every commit), base `64fe5cf`.
Logs and scripts: `<SCRATCH>/wc/` (`run.sh` background cargo, `rec.sh` / `cmp.py` replay recordings into
`wc/rep/<label>`, never into the repository; `rules.py` reads this worktree's catalog).

## C1. A spell without a target is cast only if it would affect some stack (r1 F-3) — done, `6eeff08`

- Tests first, both failing on the unchanged code for the finding's two reasons:
  - `tests/timed.rs::a_spell_without_a_target_needs_a_stack_it_would_affect` (next to BAT-MAG-003's
    mass test): failed with `castable == [storm, sloth]` where nothing may be castable (reach only).
    Cases: the enemy's hero resists 100 % and the caster's own stack is a level-1-immune wyrm: mass
    Slow (`sloth`) and the battle-wide `storm` are not castable, have no target, are refused
    (`InvalidSpellTarget`), state equal, mana and casts untouched (100, 0); with a `fixed` stack of
    the caster's own the storm is castable and strikes it while both enemy stacks resist (95, 1);
    with 99 % resistance (a chance of 1 %) both are castable and over 300 seeds the 600 rolls give
    some slowed stacks (1..=20) and the rest resisted.
  - `tests/control.rs::a_mass_version_is_cast_when_only_hypnotized_stacks_would_be_affected` (after
    A11's test): failed with no target where `[None]` is expected (hypnotized stacks left out). The
    attacker's mass Slow with the defender's only stack hypnotized by the attacker: legal, paid
    (196, 1), one event (`SpellCast`), nobody reached; with the attacker's hero resisting 100 %
    (the hero that controls the stack) it is refused.
- Code (`magic.rs`): `no_target` asks `in_reach(..) && has_chance(..)` of some stack; `has_chance`
  (new, private) is the second half of `accepts` (positive, or ignores immunity, or `may_affect`),
  which now calls it. `castable_spells` (`has_target`), `cast_targets` and `apply`
  (`first_target`) all go through `no_target`, so they agree by construction; the property
  `every_step_preserves_the_invariants` passed.
- **Decision 1: hypnotized stacks count (`in_reach`, not `reachable`).** The list says both
  "hypnotized stacks count there" and "the legality test must count exactly the stacks the
  resolution would reach". They differ in one case: a mass version whose only stacks with a chance
  are hypnotized ones it passes over. I followed the original (the finding, the digest's
  `any(in_reach && accepts)`, and my own look: the book's loop from 0x59ED5A calls 0x5A39C0 per hex,
  that calls 0x5A8090 for the stack on the hex, and 0x5A8090 reads the Hypnotize duration once, at
  0x5A82C0, for the controlling hero): the cast is legal, paid for and reaches nobody. The opening
  spells already work so (`magic::opening`, `tests/opening.rs` "is cast and reaches none"). Every
  stack the resolution reaches is counted (`reachable` implies `in_reach`), so no cast that would
  affect a stack is refused. If the other reading is wanted, it is one word in `no_target`
  (`reachable` for `in_reach`) and the first half of the control test flips to "refused".
- **Decision 2: not the sketch's `accepts`.** `accepts` also asks `needs_effect` (Dispel aimed at a
  stack). With it, Expert Dispel would be refused while no stack has an effect, so it could no
  longer clear spell obstacles; the original asks for an active spell below Expert only (BAT-MAG-016's
  source: 0x5A8090 for spell 35). Tried it: `dispel::dispel_reaches_the_enemy_from_advanced_and_every_
  stack_at_expert_without_immunity` (line 169) and `pits::living_natives_see_the_other_sides_pits_
  without_bat_quirk_1` fail with the sketch as written. So the mass test is `has_chance` only.
- Seen on the way, not changed (not in the list): `magic::opening` asks `accepts`, so an opening
  spell that needs an effect when aimed (an Expert Dispel as an opening spell; no pack has one) is
  cast only if some stack has an effect, unlike a hero's Expert Dispel.
- A share above zero that rounds to a chance of 0 % (resistance between 99 and 100 %) counts as "may
  be affected" here as it does for an aimed spell (`may_affect`, the original's factor above 0):
  castable, and every stack resists. Unchanged, the same as `aimable`.
- Cost: for a negative or neutral spell without a target `castable_spells` now computes one
  `may_affect` (the stack's share and the aura walk over the stacks) for the first stack in reach.
  If `bench-check` moves on the `full` workload, this is one place to look.
- Golden replays: none changes (recording `c1` is identical to `base`, the recording at `64fe5cf`:
  commands, checkpoints, final hashes of all eight).

## C2. Quicksand's and Land Mine's draws avoid only living stacks and corpses (r3 F-8) — done, `4b2cd76`

- Test first: `tests/pits.rs::pits_and_mines_may_lie_where_a_dead_stack_left_no_corpse` (after the
  draw's existing test; both spells through real casts, every hex blocked but the stacks' and one
  other): a dead army stack (marked by `mark_corpses`, a corpse) keeps its hex free: `{(5,5)}`; a
  dead stack with the trait SUMMONED (marked: `NO_CORPSE`) and a dead stack whose corpse is gone
  (`raised.corpse = NO_CORPSE` by hand: a sacrifice, the Pit Lords) do not: `{(5,5), (8,2)}`. On the
  unchanged code it failed at the summoned stack (its hex took no pit).
- Code (`obstacles::random_hexes`): the footprint of a stack is taken only if it is alive or its
  corpse mark is not `NO_CORPSE`.
- A dead stack whose death is not marked yet (mark 0) still counts, as a corpse would: in play every
  death is marked before a draw (after every command and before every activation; a cast draws
  first thing in its command), and the existing test kills by hand without marking.
- Not touched (not in the finding): `obstacles::free` (an aimed Force Field or Fire Wall: living
  stacks only, corpses do not count there already).
- **Golden replay `full-s3-a303` changes** from command 469 on (542 commands become 531, same
  winner; checkpoints differ from "after 475"). Why: command 468 is `D cast 32 none` (spell 32 of the
  synthetic pack is `mire`, its Quicksand); at that moment the attacker's created stack 18 lies dead
  on hex 161 with the corpse mark `NO_CORPSE` (a throwaway probe, `wc/zz_probe.rs`, printed the dead
  stacks before that command), so hex 161 is now among the draw's hexes and the eight pits fall
  elsewhere. The other seven recordings are identical to C1's.

## C3. An expired created stack's record is not reused (r4a F-4) — done, `1e9e4ab`

- Checked first: 0x47A100's search (0x47A12B to 0x47A155, my own look): a record is taken if its
  creature is −1, or if its count is 0 and it has the dead and the summoned flag; `installation.md`
  §7.19 says "with no creatures" already. What B2 did in `created::add`: it calls
  `effects::forget(state, id)` for the record it reuses (instead of `remove_dead`); unchanged, and
  still right (a killed created stack may carry a permanent effect).
- Tests first, each failing on the unchanged code at the reused record:
  - `tests/created.rs::a_created_stack_whose_rounds_ran_out_keeps_its_record` (failed: the next
    stack got id 2, the expired clone's, where 4 is expected). After the fix: clone (2), sprites (3),
    the clone expires, the next sprites are stack 4; of the two sprite stacks (equal speed) 3 acts
    before 4 in two rounds (`[3, 4, 3, 4]`); the killed sprites' record 3 is still reused (the
    clone's is passed over); the outcome is `Army(0) serf 10→10, Created serf 10→10` (the expired
    clone, not killed), `Created sprite 6→6` twice.
  - `tests/created.rs::created_stacks_whose_rounds_ran_out_count_towards_the_limit` (limit 3; failed:
    the second clone got id 2, not 3): after two expired clones the side is at its limit, `copy` has
    no target, is not castable and is refused.
  - `tests/created.rs::with_quirk_bat_quirk_7_a_vanished_original_keeps_its_record` (profile
    `t:clone_on_original`; failed: the next sprites got id 0, the army stack's own record): the
    ogres turned, lost one, vanished with two; the next sprites are stack 3; the outcome reports
    `Created ogre 3→2` and the two sprite stacks (Draw: only created stacks left).
  - `tests/raisers.rs::a_pit_lord_finds_no_slot_in_a_clone_whose_rounds_ran_out` (limit 4; failed:
    the fiend had a target): full side with an expired clone: no slot, the raise is refused; with
    a clone killed by damage: the imps take its record 4.
- Code (`created.rs`): `reusable(state, side)` (new, private): the side's first stack that is
  created, dead and has `Created::vanished == 0` (a killed one; a stack whose rounds ran out has its
  count there). `add` uses it; `has_slot` uses it too (it had its own "any dead created stack", which
  would have promised the Pit Lords a slot that `add` no longer takes: a push past the side's limit).
  Comments in `casters.rs` and `event.rs` ("a dead created stack's id may be reused") now say
  "killed".
- **The consequences the finding names, checked:**
  1. The limit: `has_room` counts the side's stacks, dead ones included, as before; expired stacks
     now stay in that count, so a side that keeps cloning reaches `side_stack_limit` (the test).
     Every creation is still checked by `has_room` (Clone, Summon) or `has_slot` (the Pit Lords)
     before `add`, so `add` never pushes past the limit, and the limit is at most 21 per side
     (`STACK_CAPACITY` 42): no capacity changes.
  2. The id: a later created stack gets a new id after every existing one; the turn order's
     tie-break (speed, side, slot, id) puts it after the earlier created stacks (the test).
  3. The outcome: one `StackRecord` more per expired stack (origin `Created`, start, end = the
     creatures it vanished with); with BAT-QUIRK-7 the turned army stack's record is no longer
     overwritten. `BattleOutcome.stacks` is a list: no type changes.
  No state field, event, error, wire type, encoding or version changed. State hashes change only
  where a creation follows an expiry on the same side.
- Golden replays: none changes (recording `c3` is identical to `c2`: no replay has a creation
  after an expiry on the same side).
- For the coordinator: long battles with many expired clones now carry up to the limit of dead
  records per side (the stack loops walk them); `bench-check` workloads were not run.

## Part 2: the two stale comments (r6 F-8) — done, `f0f2caa`

- `crates/h3-content/src/ability.rs`, `Ability::IgnoresMagicBlock`, and
  `crates/h3-battle/src/casters.rs`, `ready`: both now say the ability is one for packs that no
  harvested creature has, and that the Orb stops the Faerie Dragon's cast too (the code reading and
  batch 2e's observation of the cast by the mouse). The ability's comment adds "the cast by the key
  was not tried" and does not call the question closed (no question id but BAT-Q21, which the
  comment had; BAT-Q32 is not in this branch's `OPEN-QUESTIONS.md`, so I did not cite it).
- Comments only. `packs/README.md` and the gap-fill file already said it right.

## Part 3: tests of behaviour that is right today

No engine code changed for any of them. Each passed on the unchanged engine unless said otherwise.

### T1 (r1 F-10) — done, `949f001`

- h3-battle `tests/magic.rs::mastery_is_the_best_school_and_selects_cost_and_effect` (BAT-MAG-002's
  test, beside the refusal with 3 against 4): mana 4 at cost 4: castable, accepted, `(mana, casts)
  == (0, 1)`.
- h3-pack `tests/harvested_spell_damage.rs::batch_2a_r1_r2_chain_lightning_by_the_screen_distance_
  and_the_army_slot`: R1 (Caitlin against Nagash) and R2 (Jenova against Damacon), heroes derived
  by `h3-heroes` (power 1, 30 spell points; Blue with Expert Tactics, zone 7 columns), the
  Peasants placed through the tactics phase's real commands on the hexes as played (R1 one hex left
  of the plan: (11,4), (9,4), (11,6); R2 one hex down-left: (12,5), (14,5), (10,5); column, row,
  0-based). Engine: R1 `[(1,65), (3,32), (2,16), (0,8)]`, counts 35 / 85 / 70 (A, B, C) and 10
  Pikemen; R2 `[(1,65), (2,32), (3,16), (0,8)]`, counts 35 / 69 / 86. The batch's numbers exactly.
  The targets are the three Peasant stacks only (R1 round 2: not the own Pikemen).
- h3-pack `tests/harvested_mana.rs::batch_2a_r6_the_hero_casts_in_his_ballistas_turn`: Calid derived
  with Basic Artillery and the worn Ballista (the war machine from `h3_heroes::war_machines`),
  Grindan with Ice Bolt. Red 20 → 15, Blue 10 → 2 and Red 16, Red 16 → 11 in the Ballista's turn of
  round 2 (castable, Pegasus alive), Red 11 → 6 in round 3 (Pegasus dead). **One thing the test
  adds to the record:** the batch says the Pegasus was alive at the Ballista's turn and was killed
  by the Mages in round 2; in the engine the Mages (speed 5) act before the Ballista, so the test
  lets them Wait and shoot after it. The batch does not say whether the player waited.

### T2 (r2b F-6, items 2 to 4) — done, `3c89533`

- Item 2, h3-battle `tests/rage.rs::a_frenzied_stack_that_waits_is_counted_once_a_round` (before
  BAT-MAG-019's last test): real commands only (the stacks placed by hand, no forced activation). Ten
  Nagas (no retaliation against them) beside a Titan that always waits first; the hero has attack
  and defense 0. Cast on the active Nagas (count 1): Wait, strike in the wait phase 20 (frenzied;
  unfrenzied 15), count still 1; round 2's first turn start ends it: 15. Cast on Nagas that have
  waited (count 2): strike in the wait phase 20, count 2; round 2: count 1, strike 20; round 3:
  ended, 15. A count at the turn after waiting would give 15 in the wait phase.
- Item 3, h3-battle `tests/defences.rs::a_mirror_draws_the_new_target_among_the_casters_war_machines_
  too`: seeds 0 to 399, every cast sent away (Expert mirror, 100 %); each of the caster's Tanks,
  Ballista, Ammo Cart and First Aid Tent is the new target between 70 and 130 times.
- Item 4, h3-pack `tests/harvested_timed_spells.rs` (new file),
  `batch_2b_r3_the_fire_shield_reflects_a_fifth_of_the_first_stage`: Aine (attack 0) with ten Naga
  Queens against Sephinroth (defense 35, power 5, Fire Shield at no mastery, 20 spell points) with
  1000 Peasants; the batch's steps (walk; the cast in the Peasants' turn, 20 → 4 spell points; one
  hex; the attack in round 2). `base:faithful`: strike 150 with 150 kills, reflection 60, Naga
  Queens 10 with the top one at 50 of 110: the batch's numbers. `base:strict-fixed` (Q-0010 off):
  30, top at 80.

### T3 (r3 F-10) — done, `8fddc3c`

- h3-pack `tests/harvested_control_and_field.rs` (new file; heroes derived by `h3-heroes`, every step
  a real command, positions through the stacks' own moves):
  - `batch_2c_f1_fire_wall_burns_on_entering_on_wait_and_on_defend`: Aine (power 1) with 50 Pikemen
    and a Peasant against Gelare with ten Wolf Riders and a Peasant. Engine: Blue's Force Field =
    clicked hex + its up-right neighbour; Red's Fire Wall = (3,2) + up-right; the walk into (3,2)
    `SpellDamage 20, 2 kills` (48); Blue's Fire Wall = clicked hex + its up-left neighbour; Wait 20
    / 2 (46); Defend in the wait phase 20 / 2 (44); in round 3 no fire obstacle of Red's, no burn.
    The batch's numbers exactly.
  - `batch_2c_f2_a_walker_two_rows_up_passes_its_up_right_neighbour`: Red's Pikemen from (3,2) to
    (3,0) with the wall on (4,1) and (4,0): burnt 20 / 2 (8 left); Blue's from (13,2) to (13,0) with
    the wall on (13,1) and (12,0): not burnt (10). As the batch.
  - `batch_2c_h1_hypnotize_counts_full_hit_points_and_expert_cure_passes_over_it`: targets of
    Hypnotize `[the one Swordsman]` in round 1 and again in round 2 after the Magic Arrow (30; two
    Swordsmen left, top one at 5); Expert Cure: top one at 35; `base:faithful`: Hypnotize stays and
    Red commands the Swordsman in rounds 2 and 3 (the batch's record); `base:strict-fixed`: the Cure
    frees it and Blue commands it (not observed: the fixed variant).
- h3-battle `tests/obstacles.rs::of_two_shortest_paths_a_walker_takes_the_one_by_its_ends_first_
  neighbour`: `movement::path` from (5,5) to (5,3) is `[(5,4), (5,3)]`, to (5,7) is `[(4,6), (5,7)]`;
  the defender's walker from (12,5) alike.
- h3-battle `tests/control.rs::hypnotize_needs_full_hit_points_within_its_value_and_no_mirror`: power
  2 at Basic (60): 6 × 10 hit points a target, a stack of 61 hit points (set by hand) none, of 60
  one, 7 × 10 none.

---

**Takeover (after the restart at 18:00).** A second worker `c` continues from here: T4's uncommitted draft
(three files) was read against the list, run, completed and committed; T5 to T9 and the closing parts follow.
New helper scripts in `<SCRATCH>/wc/`: `seq.sh` (several cargo commands one after the other, in the
background), `mut.sh` (a throwaway mutation of one engine file: apply, run one test, restore from a saved
copy, check `git diff --quiet`; used only to see that a new test can fail; the engine files are unchanged
after each run, and `git status` is checked before every commit).

### T4 (r4a F-10) — done, `2a3aafd` and `a030d30`

All of it passed on the unchanged engine (the draft as found, and what was added).

- BAT-MAG-035's boundary, h3-battle `tests/raise.rs::resurrection_from_advanced_raises_for_good_and_needs_
  one_creatures_health` (extended): power 0 at Basic raises 40: an ogre stack (health 40) is a target and
  gets one ogre back for the battle (`SpellRaised 40, 1, temporary`); the second ogre stack with `max_hp` 41
  (set by hand) is none. Mutation `<=` to `<` in `raise::eligible`: the test fails (no target).
- BAT-MAG-038's skips, `tests/created.rs`:
  - `a_clone_goes_to_the_first_free_hex_along_the_rays` (one case more): top right, bottom right and bottom
    left taken: the clone goes top left, not left. Mutation (left no longer skipped in ring 1): fails.
  - `the_search_from_a_two_hex_stacks_head_skips_the_way_to_its_tail_in_rings_1_and_2` (new; the draft had
    its first half, I added the second): (a) only the ogres' hexes and a place three hexes left of them are
    free: the clone stands there in `t:faithful` and in `t:clone_on_original` (with the quirk an unskipped
    way from the head would find the ogres' own position and turn them); (b) the place four hexes away:
    ring 3 comes to the ogres' own position first, from the head: with the quirk `TurnedSummoned (0, 3)`,
    without it the clone on the place. Mutations `ring <= 1`, `ring == 2` (ring 1 not skipped) and
    `ring <= 3`: each fails, the first two at (a), the third at (b).
- BAT-MAG-039, `tests/created.rs::a_summoned_stack_takes_a_random_fitting_row_and_none_without_a_place`
  (new): rows 0 to 2 of column 1 blocked, seeds 0 to 319: every other row 20 to 60 times, column 1 always;
  mutation (always the first fitting row): fails with `[0, 0, 0, 320, 0, …]`. No place in any column:
  nothing created, 2 stacks, mana − 4, the observation's `summoned == [sprite, None]`, `sprites` still
  castable and `serfs` not.
- Batch 2d on the harvested packs, h3-pack `tests/harvested_life_created.rs` (new file; SoD + AB + base,
  `base:faithful`; heroes derived by `h3-heroes`, every step a real command):
  - `batch_2d_ra2_…`: Gelare (power 1, Basic Earth) against Grindan (power 7). Engine: Lightning Bolt
    `SpellDamage 185, 5 kills` → (5 Swordsmen, top 25); Resurrection `SpellRaised 90, 3, temporary` → 8,
    `temporary` 3; second bolt 185, 6 kills → 2; the Swordsmen walk over and kill the Peasants; outcome
    `Victory(Attacker)`, record `Army(0) 10 → 1`, survivors `[1]`. The batch's numbers exactly.
  - `batch_2d_cl1_…`: Aine (power 3) against Damacon (power 2). Engine: price 800 before and after Red's
    Clone; Blue's clone (stack 4, 10) on the up-left neighbour of its Pikemen, Red's (stack 5, 10) on the
    up-right neighbour of the Archers; the turn order of round 1 is the batch's (Archers, Blue's Pikemen,
    Red's Pikemen), then stacks 3, 4, 5 all act; Archers 11 shots after one shot, their clone 12 and 11;
    round 2, Blue's first turn: Magic Arrow `SpellDamage 30, 10 kills` on the clone, nothing left; Blue's
    clone alive in round 2, count 0 and no corpse when round 3 starts. The batch's record exactly.
  - `batch_2d_su_…` (variant a, on grass): Octavia (power 1, 350 spell points, Ammo Cart, First Aid Tent,
    seven single Peasants) against Calid (power 1, ten Skeletons). Engine: price 910 before and after E1;
    E1 = stack 10, 2 Air Elementals, column 1, attack / defense / speed 10 / 10 / 8, acts in round 1 right
    after the first Peasant; E2 = stack 11 in column 1; E1's and E2's rows are {3, 7} (the batch: 7 then
    3; the engine with seed 1: 3 then 7); Death Ripple 15 kills the seven Peasants; Implosion 175 kills E1;
    E3 takes record 10 and acts first in round 4; one summon a round until round 12; the refusal in round
    13 (`InvalidSpellTarget`, not castable) with 20 stacks of Red's; still refused in round 14 after
    Implosion killed the last elemental stack; mana 350 − 12 × 25. Columns `[1 × 10, 2, 2]`; the ten rows of
    column 1 are every row but the war machines' (the Ammo Cart at (1, 1), the Tent at (1, 9)) plus E1's row
    a second time, which is the batch's multiset (7, 3, 2, 0, 6, 8, 7, 5, 10, 4); the two rows of column 2
    differ. End: `Draw`, no army survivors on either side, Red's 20 records = 7 Peasants `1 → 0`, the two
    war machines `1 → 1`, ten created `2 → 2`, one created `2 → 0`; Blue's Skeletons `10 → 0`: the result
    window's casualties (7 Peasants, 2 Air Elementals, 10 Skeletons).
- **Seen in SU, not in the batch's record (no deviation from the rule):** Death Ripple also strikes the two
  elemental stacks (15 each, nobody lost; top one at 10 of 25). That is BAT-MAG-005 as written (Death
  Ripple spares undead and war machines only: the pack's `excludes = ["undead", "war_machine"]`, the code
  reading 0x44A4D0); the batch did not look at the elementals after the Ripple. The test asserts it with
  that comment. If someone replays SU, a right-click on E1 after the Ripple (10 of 25?) would observe it.
- **SU b and c too, `a030d30`** (`batch_2d_su_a_side_of_elementals_loses_too_when_the_enemy_leaves`; SU's
  first two rounds moved into a helper the two tests share; `battle()` gives each player 30 000 gold, SU
  c's record): in round 3, after the elementals Defend, Blue's price is 300 and Blue retreats or surrenders.
  Engine: `Retreat(Defender)` with no cost, `Surrender(Defender)` with `surrender_cost` 300; Red's army has
  no survivors both times (its records: seven Peasants `1 → 0`, two war machines `1 → 1`, two created
  `2 → 2`); Blue's Skeletons `10 → 10`. The batch's record (Red's hero gone both times, 300 paid to Red).
  I added it because the clause of BAT-END-001 and BAT-END-003 it pins ("a side whose opponent retreated or
  surrendered but which has no army survivors has lost as well, and still receives the price") had no
  test at all, on either pack.
- Rows of batch 2d **not** replayed (the finding names RA2, CL1 and SU): CL2, RA1 and RA4 (cursor and
  corpse rows; their rules are pinned on the synthetic pack in `tests/raise.rs` and `tests/created.rs`),
  RA3 (the original's result is the bug BAT-QUIRK-6 fixes), NE (Necromancy and experience are the
  orchestrator's). Say so if they are wanted too.

### T5 (r4b F-6) — done, `70f3977`

Three new tests in h3-battle `tests/strike.rs`, each beside its rule's tests; all passed on the unchanged engine.

- (a) `poison_and_age_follow_the_originals_single_precision_values_with_quirk_q_0005` (BAT-MAG-044): the
  serf has 9 hit points in this test's ruleset (`ruleset_from` with a pack patch, the helper for review
  repros: no creature of the test pack is living with 9, 29, 49 or 3, 23, 43 hit points, the only values at
  which the variants can differ, and the shared creature table stays as it is). Five wyverns strike ten
  serfs (10 damage; Poison): `(max_hp, top_hp, count) == (9, 8, 9)` in both profiles; five ancients strike
  (10; Age): `(5, 3, 8)` in `t:faithful`, `(4, 2, 8)` in `t:single`. Mutations of `vitality::materialize`
  (the flag constant false, constant true): the test fails in `t:single`, in `t:faithful`.
- (b) `a_shot_at_a_petrified_stack_is_halved_too` (BAT-MAG-043): five gazers petrify ten `fixed`; ten archers
  (damage set to 2 by hand, as `tests/ranged.rs` does; attack 6 against defense 10: 18) shoot for 9, which
  ends the petrification, then for 18; `t:faithful` and `t:single` alike. Mutation (petrified halves melee
  only): fails with 18. I did not take `chanter` as the shooter (the test pack's only one with a fixed
  damage, 2 to 2): an Enchanter casts at its activation, which would put a second thing into the test.
- (c) `bind_ends_after_its_rounds_and_leaves_the_stack_without_binders` (BAT-MAG-045): after 254 round
  starts `tether` has `Some(1)` and no end was reported; the 255th reports `EffectEnded`, the stack is not
  bound, has no effect and has moves among its legal commands; the binder's next strike casts `tether` again
  (255 rounds). Mutation (an ended Bind keeps its binders): fails at that strike.

### T6 (r5a F-7) — done, `86a40dd`

Three new tests in h3-battle, each beside its rule's tests; all passed on the unchanged engine.

- (a) `tests/raisers.rs::an_archangels_hit_points_equal_to_the_health_reach_it` (BAT-MAG-051): two heralds
  (200 hit points) and two seraph stacks (health 200) that lost one each, the second with `max_hp` 201 by
  hand: targets `[1]`, the raise of stack 2 is refused (`InvalidSpellTarget`), stack 1 gets one seraph back
  for good. Mutation `<=` to `<` in `raise::eligible`: fails (no target).
- (b) `tests/casters.rs::every_stack_in_a_creatures_area_rolls_for_resistance` (BAT-MAG-048, BAT-MAG-049):
  one novaling casts `nova` (70) at the hex of 30 dwarves (20 % resistance), with 30 dwarves and 30 `fixed`
  beside it, seeds 0 to 199: each dwarf stack either resists (reported) or takes 70; the dwarves on the
  aimed hex resisted 37 times, those beside it 47 times (asserted: 20 to 60); `fixed` always takes 70.
  Mutation (a creature's area never rolls): fails with `[0, 0]`.
- (c) `tests/casters.rs::a_creatures_chain_strikes_its_first_stack_without_a_roll_and_rolls_at_every_jump`
  (BAT-MAG-006 for a creature's cast): one chainling casts `chain` (70) at 30 dwarves with 30 dwarves next
  to them (47 pixels) and `fixed` two rows up (84 pixels), seeds 0 to 199: the first struck stack is always
  the aimed one with 70 and it never resists; the second is the dwarves beside it with 35, or, when they
  are passed over, `fixed` with 35: 37 times (asserted: 20 to 60). Mutations: the first stack rolls for a
  creature's cast: fails (no stack struck in some seed); no roll at the jumps: fails with 0.
- Seen while reading, **not a deviation, for the event's doc only**: a stack that resists when the chain
  looks for its next stack is passed over without a `SpellResisted` event (`magic::next_in_chain` has no
  sink); `EventKind::SpellResisted`'s doc does not say so. A hero's chain is the same.

### T7 (r6 F-5) — done, `7536b2d`

- **The reviewer's numbers are right: the engine gives exactly them.** Checked before the test was written,
  with a throwaway probe (deleted) over every skill level × the eight artifact sets × a non-specialist
  (Orrin) and the specialist Serena at levels 1 to 40, in `base:faithful` and `base:strict-fixed` (984
  values), against my own model of the arithmetic `installation.md` §7.24 records for 0x4E4420, in exact
  fractions (`<SCRATCH>/wc/t7/oracle.py`; the probe's output is `wc/t7-probe.log`): **0 mismatches**. The
  two profiles differ in 42 of the 984 cases, always by one percent less with Q-0005; without the
  intermediate `f32` stores the model gives the exact values everywhere, so the stores are the cause, as
  the reviewer says.
- h3-pack `tests/harvested_eagle_eye.rs` (new file),
  `eagle_eyes_chance_is_the_originals_single_precision_sum_with_quirk_q_0005` (HER-SPL-004, BAT-MAG-069):
  the percent in the own hero's observation of a battle the derived hero leads, `(faithful, strict-fixed)`:
  Orrin Advanced + Emblem of Cognizance `(64, 65)`; Basic + Stoic Watchman + Emblem `(64, 65)`; Basic + Bird
  of Perception `(45, 45)`; Serena (specialist) Basic at level 2 `(43, 44)`, at level 6 `(51, 52)`, Advanced
  at level 6 `(64, 65)`; no difference: Basic 40, Advanced 50, Expert 60, Expert with all three 90, Serena
  at level 40 with everything 100.
- Mutations in `h3-heroes` `RateSum`: the sum not stored as an `f32` at each step: fails at Basic +
  Watchman + Emblem (65); the seam never carries the single-precision value: fails at Basic + Bird (44).
- For the docs workers (the finding's rule-text part, not mine): both rules still have `quirks = []` in
  this branch's catalog.

### T8 (r6 F-6) — done, `29d4ed0`

Two new tests in h3-battle `tests/opening.rs`, after the test of a blind first stack.

- `a_side_whose_first_stack_freezes_casts_at_the_first_stack_that_takes_its_turn` (BAT-MAG-068 with
  BAT-MOR-004 and BAT-MAG-060), seeds 0 to 39 for each of two setups, only the defender with a hero (Slow
  and Curse as opening spells), the attacker's `fast` first, then the defender's dragon and flyers:
  - bad morale: the defender's hero has `Morale −3` on its own stacks, the ruleset's bad-morale chances are
    8 of 24: in 12 of the 40 battles the dragon freezes and the flyers do not: `BadMorale(dragon)` <
    `TurnStarted(flyers)` < `OpeningSpell(Defender)`, the flyers active;
  - Fear: a `spook` of the attacker's, its fear set to 50 % by the test's pack patch: 13 of the 40 alike with
    `Frightened`;
  - in the other battles: both froze: no opening spell in that command; the dragon took its turn: the cast
    follows its turn start.
  **First run: my test failed, and the test was wrong, not the engine.** With the file's `heroes()` the
  attacker's Prayer (+4 speed at Expert) made the spook as fast as the dragon, so the spook acted before it
  and no defender's stack was picked in that command. With no attacker's hero the order is as intended.
  Mutations of `turn::activate` (the activation before both checks; inside the Fear branch only; inside
  the bad-morale branch only): each fails (an opening spell although both stacks froze).
- `the_opening_spells_come_before_the_enchanters_own_cast` (BAT-MAG-068, BAT-MAG-053), seeds 0 to 59, five
  chanters against ten `fixed`: with a hero whose opening spell is Slow the `OpeningSpell` event precedes
  `PeriodicCast` and the periodic spell is never Slow (0 of 60); without the hero Slow is drawn 16 times.
  Mutation (the periodic cast first in `turn::activation`): fails.

### T9 (r6 F-7) — done, `2047c8b`

- h3-battle `tests/hero_defences.rs::the_hero_of_the_side_that_controls_a_striker_negates_immunities_for_
  its_strikes_spell` (BAT-MAG-066, BAT-MAG-040; after the negation's first test, before the file's last
  one). Passed on the unchanged engine.
  - Five `blinder` of the attacker's strike ten `statue` of the defender's: no `StrikeSpell`, no effect;
    with `NegateImmunities` on the attacker's hero: `StrikeSpell blind`, Blind on the statue; with it on
    the defender's hero only: the same (the statue's own hero).
  - Beyond the sketch, to pin "the side that controls the striker" and not the striker's own side: the
    defender's blinders, hypnotized (controlled by the attacker), strike the defender's statue: nothing
    without a negation; Blind with the attacker's hero's.
  - Mutations of `after_strike::spell_after_strike` (the casting side = the striker's own side: fails at
    the hypnotized case; = the struck stack's controller: fails at the plain case with the attacker's Orb).

---

## Closing parts (list C complete; branch `battle/e04b-t11b-c`, head `2047c8b`, pushed)

### State of every item

| item | state | commit(s) |
|---|---|---|
| C1 a spell without a target (r1 F-3) | done | `6eeff08` |
| C2 pits and mines against corpses (r3 F-8) | done | `4b2cd76` |
| C3 an expired created stack's record (r4a F-4) | done | `1e9e4ab` |
| Part 2 two comments (r6 F-8) | done | `f0f2caa` |
| T1 (r1 F-10) | done | `949f001` |
| T2 (r2b F-6, items 2 to 4) | done | `3c89533` |
| T3 (r3 F-10) | done | `8fddc3c` |
| T4 (r4a F-10) | done | `2a3aafd`, `a030d30` |
| T5 (r4b F-6) | done | `70f3977` |
| T6 (r5a F-7) | done | `86a40dd` |
| T7 (r6 F-5) | done | `7536b2d` |
| T8 (r6 F-6) | done | `29d4ed0` |
| T9 (r6 F-7) | done | `2047c8b` |

T4 to T9 changed no engine code (nine test files, 1 261 lines). The whole branch against `64fe5cf` touches
nothing under `h3-kernel`, `h3-contracts`, `Cargo.toml`, `xtask`, CI or `docs/`.

### Closing checks (at `2047c8b`)

- `cargo fmt --all -- --check`: clean. `cargo clippy -p h3-battle -p h3-content -p h3-pack --all-targets --
  -D warnings`: clean.
- `cargo test --no-fail-fast -p h3-battle -p h3-content -p h3-heroes -p h3-pack -p h3-sim -p h3-cli -p h3-ffi
  -p h3-protocol`: everything passes (h3-battle 500, h3-content 79, h3-heroes 23, h3-pack 17 + 26 in its
  nine harvested files, h3-protocol 68, h3-sim 35, h3-ffi 5, h3-cli 3 + 1) except the one expected failure,
  h3-cli `golden_replays_reproduce_their_hashes` (it stops at its first replay, `duel-s1`, which differs
  from its golden file at the base `64fe5cf` already).
- The battle crate's tests three times (once in that run, twice alone): 500 passed each time.
- `cargo xtask check-rules`: ok (183 rules, 409 rule markers, 929 covers markers). `check-arch`: ok.
  `check-stubs`: ok (1 registered stub). Also `check-exe-map`: ok (341 entries, every cited address
  covered; the tests and C1 to C3's comments cite addresses).
- Not run, as the brief says: coverage and `cargo xtask verify`.

### Golden replays

- Recorded again at the head (`wc/rep/final`, never into the repository): identical to the recording after
  C3 for all eight, so T4 to T9 change none.
- Against the base `64fe5cf` list C changes one recording: **`full-s3-a303`**, from command 469 on (542
  commands become 531, same winner), by C2 (the cause is in C2's notes above). C1 and C3 change none.
- Against the committed golden files the same seven differ as at the base (all but `full-s3-a1367`);
  `full-s3-a303` was one of the seven before C2 (from the checkpoint after 175).

### Rule texts, ledger entries and `installation.md` sentences that C1 to C3 make wrong, with proposed wording

Every "old" clause below is verbatim in this branch's docs (checked by script; line numbers are this
branch's). Tests that add no behaviour (T1 to T9) make no text wrong.

**C1 (`6eeff08`)**

1. BAT-MAG-003, summary (`battle.toml` line 1894).
   - old: "A battle-wide spell (Armageddon, Death Ripple, Destroy Undead) takes no target
     (CastTarget::None) and needs at least one stack it can reach."
   - new: "A battle-wide spell (Armageddon, Death Ripple, Destroy Undead) takes no target
     (CastTarget::None) and is cast only if it would affect some stack: a living stack in its reach that
     it can affect (BAT-MAG-005) and that does not always resist (its share after resistance above zero,
     unless the spell is positive or ignores immunity: the test of a stack spell's target). Without one
     the cast is refused (CommandError::InvalidSpellTarget) before the mana or the round's cast is
     spent, as the original's spell book refuses it."
2. BAT-MAG-003, summary, the mass version.
   - old: "from it takes no target and reaches every stack of its side (own for positive spells, enemy for
     negative ones) that it can affect, like a battle-wide spell, except hypnotized stacks (BAT-MAG-026;
     Forgetfulness's mass version from Advanced under quirk Q-0011, BAT-MAG-023)."
   - new: "from it takes no target, is cast only if it would affect some stack of its side (own for
     positive spells, enemy for negative ones) as a battle-wide spell is, and reaches every stack of
     that side that it can affect except hypnotized stacks (BAT-MAG-026; Forgetfulness's mass version from
     Advanced under quirk Q-0011, BAT-MAG-023). A hypnotized stack it passes over still counts for
     that condition (the original's test reads a stack's Hypnotize only for the hero that controls it):
     with no other stack the spell is cast, paid for and reaches nobody. A stack without an active
     effect counts for the mass version of a spell that needs one when aimed (Expert Dispel)."
   - "Magic resistance is rolled when the spell takes effect, not checked here" stays right: only a share
     of zero is checked, never a roll. (A share above zero that rounds to a chance of 0 %, resistance
     between 99 and 100 %, counts, as it does for an aimed spell.)
3. BAT-MAG-003, sources: the rule has no source for this clause yet. Proposed original-data ref: "the
   spell book's handler (from 0x59ED5A) asks the target test 0x5A39C0 of every hex outside the side
   columns before anything is paid and refuses the spell with a message when none passes; for these
   spells the test wants a stack whose factor is above zero and reads its Hypnotize only for the
   controlling hero (E04b T11b review; an experiment candidate: the code reading stands alone)". The
   further addresses of the reviewer and of my predecessor (0x59EE03, 0x5A82C0) have no map entry of
   their own; they lie inside mapped functions, and `cargo xtask check-exe-map` is ok on this branch
   with the code comments that cite them (341 entries, every cited address covered).
4. `installation.md` §7.8 (line 422).
   - old: "The single-target spells and the battle-wide ones (24–26) need at least one stack the target
     test accepts before the cursor opens; the area spells do not."
   - new: "The single-target spells, their mass versions and the battle-wide ones (24–26) need at least
     one hex whose stack the target test accepts (`f > 0`) before the spell is armed: without one the
     book shows a message and nothing is cast or paid; a spell that takes no target stack at its mastery
     is then cast without a cursor. The test reads a stack's Hypnotize only for the hero that controls
     it, so a hypnotized stack the mass handler later passes over counts. The area spells have no such
     test."
5. BAT-MAG-068 ("each spell, in order, is cast only if some living stack would be affected by it (the
   target test, hypnotized stacks counted)") already says for the opening spells what C1 now does for a
   hero's cast: nothing to change. (They differ in one corner my predecessor noted: `magic::opening` also
   asks for an active effect when the spell needs one aimed; no pack has such an opening spell.)

**C2 (`4b2cd76`)**

1. BAT-MAG-032, summary (`battle.toml` line 2596).
   - old: "without a natural obstacle, an anchor hex (quirk BAT-QUIRK-3, BAT-GEO-003), a spell obstacle
     and a stack, living or dead; with fewer such hexes, fewer pits."
   - new: "without a natural obstacle, an anchor hex (quirk BAT-QUIRK-3, BAT-GEO-003), a spell obstacle,
     a living stack and a corpse (BattleState::corpse; a dead stack that left none does not keep its
     hexes free: a created stack, BAT-MAG-037, a sacrificed one, BAT-MAG-036, one whose corpse the Pit
     Lords used, BAT-MAG-052); with fewer such hexes, fewer pits."
2. BAT-MAG-033 follows through its reference to BAT-MAG-032. The rule's original-data ref and
   `installation.md` §7.16 already say "a living stack (hex record +0x18) or a corpse (+0x1C)": nothing
   to change there.

**C3 (`1e9e4ab`)**

1. BAT-MAG-037, summary (line 2726).
   - old: "A created stack takes the slot of its side's first (lowest id) dead created stack, else a new
     id after every existing one;"
   - new: "A created stack takes the slot of its side's first (lowest id) killed created stack (dead
     with no creatures left: killed by damage, or a clone that died with its original), else a new id
     after every existing one; a created stack that vanished when its rounds ran out (a clone,
     BAT-MAG-038; an original turned into one, BAT-QUIRK-7) keeps its creatures (BAT-END-003) and with
     them its slot: no later stack takes it, and it still counts towards side_stack_limit;"
2. BAT-MAG-037, original-data ref (line 2731).
   - old: "0x47A100 takes the first of records 0-19 that is empty or holds a dead summoned stack and sets
     it up as at the battle's start (0x43D8B0)"
   - new: "0x47A100 takes the first of records 0-19 that is empty or holds a dead summoned stack with no
     creatures (a stack whose rounds ran out keeps its count, so its record is never taken again) and
     sets it up as at the battle's start (0x43D8B0)"
3. BAT-MAG-052, summary (line 3004).
   - old: "the side must have a slot for the new stack: a dead created stack's or a new one below the side
     stack limit"
   - new: "the side must have a slot for the new stack: a killed created stack's (BAT-MAG-037; not one
     whose rounds ran out) or a new one below the side stack limit"
4. BAT-MAG-044, summary (line 2871).
   - old: "a stack created in a dead created stack's slot starts afresh."
   - new: "a stack created in a killed created stack's slot starts afresh."
5. BAT-END-003, summary (line 1796), a precision only.
   - old: "a slot a later created stack reused reports only that one, as the original's records (its
     earlier stack's losses are gone, BAT-QUIRK-11)"
   - new: "a killed created stack's slot that a later created stack reused reports only that one, as the
     original's records (its earlier stack's losses are gone, BAT-QUIRK-11); a created stack that
     vanished at the end of its rounds keeps its slot and is always reported"
6. Ledger BAT-QUIRK-11, "Original behavior" (`quirks-ledger.md` line 313).
   - old: "a new created stack takes the record of the side's first dead summoned stack (0x47A100) and is
     set up afresh there"
   - new: "a new created stack takes the record of the side's first dead summoned stack with no creatures
     (0x47A100: one that was killed; a clone or a turned original whose rounds ran out keeps its count
     and so its record) and is set up afresh there"
   - "Regression test": add `created.rs` `a_created_stack_whose_rounds_ran_out_keeps_its_record`,
     `created_stacks_whose_rounds_ran_out_count_towards_the_limit` and
     `with_quirk_bat_quirk_7_a_vanished_original_keeps_its_record`.
7. Ledger BAT-QUIRK-12, "Original behavior" (line 325).
   - old: "which takes the first of the side's records 0–19 that is empty or holds a dead summoned stack;
     with 20 records in use and none of them a dead summoned stack (summoned stacks and clones alive, the
     army and war machines)"
   - new: "which takes the first of the side's records 0–19 that is empty or holds a dead summoned stack
     with no creatures; with 20 records in use and none of them such (summoned stacks and clones alive
     or vanished at the end of their rounds, the army and war machines)"
   - "Regression test": add `raisers.rs` `a_pit_lord_finds_no_slot_in_a_clone_whose_rounds_ran_out`.
8. `installation.md` §7.7 (line 332).
   - old: "A dead clone's or summoned stack's record goes to the next stack the side creates (§7.19)."
   - new: "A killed clone's or summoned stack's record (dead with no creatures) goes to the next stack the
     side creates; one whose rounds ran out keeps its record (§7.19)."
9. `installation.md` §7.19 (line 1041; the paragraph's first sentence, line 1035, is right already: "with
   no creatures").
   - old: "So a new stack acts in the round it appears, and a dead clone's or summoned stack's record (and
     so its place in the turn order's tie-break) goes to the next stack the side creates."
   - new: "So a new stack acts in the round it appears, and a killed clone's or summoned stack's record
     (and so its place in the turn order's tie-break) goes to the next stack the side creates. A clone
     whose rounds ran out, and an original turned into one, dies with its count (\"A clone's end\") and
     keeps its record: the side's records in use never fall for it."
10. `installation.md` §7.21 (line 1387).
    - old: "With 20 records in use and no dead summoned one, 0x47A100 returns 0"
    - new: "With 20 records in use and no dead summoned one without creatures, 0x47A100 returns 0"
11. ADR-0014 point 6 (`docs/adr/0014-battle-magic.md` line 69; a shared path, wording only, no decision
    changes).
    - old: "A Pit Lord's Demons are a new stack in a dead created stack's slot or a new one"
    - new: "A Pit Lord's Demons are a new stack in a killed created stack's slot or a new one"
12. The wiki check the reviewer asked for stays open ("Clone: do clones that ran out still count towards
    the 20 stacks of a side?"), and the experiment of r4a F-4 (Clone, a summon E1, the clone runs out,
    a second summon E2: with equal speed E1 acts first in the original by this reading).

**Part 2 (`f0f2caa`)**: comments only; no rule text. BAT-Q32 (the F-key path) is not in this branch's
`OPEN-QUESTIONS.md`; the comments cite BAT-Q21, which they had.

### Where the engine differed from its rule or from a batch's numbers

- **Nowhere in T4 to T9.** Every test of behaviour that is right today passed on the unchanged engine. One
  test failed at its first run by its own setup, not by the engine (T8: the attacker's Prayer changed the
  turn order I had assumed; see T8).
- T7: the reviewer's unrun numbers (64 / 65, 43 / 44 and the others) are the engine's.
- Three things seen that are by the rule and worth a line each elsewhere: Death Ripple strikes summoned
  elementals (T4, SU; BAT-MAG-005: only undead and war machines are spared; the batch did not look); a
  stack that resists while a chain looks for its next stack is passed over without a `SpellResisted`
  event (T6; the event's doc does not say so); BAT-MAG-069 and HER-SPL-004 still have `quirks = []` (T7;
  the finding's doc part).

### For the coordinator

- **Disk:** I cleaned once. After the full test run `target` was 5.3 GB; I removed 23 stale test
  executables in `target/debug/deps` (first-round and single-crate builds that the last runs no longer
  use, and two throwaway probes; about 0.8 GB): 4.6 GB now. Nothing else was removed; no `cargo clean`.
- **Mutation checks:** for each new unit test I changed the engine line it pins in the worktree, saw the
  test fail, and restored the file (`wc/mut.sh`, logs `wc/mut-t*.log`); `git status` was clean before every
  commit, and the branch's diff shows no engine change after `1e9e4ab`.
- New files in `crates/h3-pack/tests/` from this list: `harvested_timed_spells.rs`,
  `harvested_control_and_field.rs` (first worker), `harvested_life_created.rs`, `harvested_eagle_eye.rs`
  (second). Each is its own test binary (about 34 MB in `target`).
- Two decisions of C1 by my predecessor wait for a word: hypnotized stacks count for the "would affect
  some stack" test (`in_reach`, as the original; the other reading is one word in `no_target`), and the
  test is `has_chance`, not the sketch's `accepts` (which would refuse Expert Dispel without an effect).
- Cost notes of my predecessor stand: `castable_spells` computes one `may_affect` more for a spell
  without a target (C1), and expired clones keep their records to the side's limit (C3); `bench-check` was
  not run.
- T4 added two things beyond the list: SU's variants b and c (a clause of BAT-END-001 and BAT-END-003 that
  had no test), and ring 3 of Clone's place search (the skip's upper bound). T9 added the hypnotized
  striker. Batch 2d's rows CL2, RA1, RA4, RA3 and NE are not replayed on the harvested packs.
