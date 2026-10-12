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
