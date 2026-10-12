# Fix notes of worker `a` (E04b T11b, list A)

Worktree `/home/claude/wt-a`, branch `battle/e04b-t11b-a` (pushed after every commit), base `68838df`.
A1 to A9: the first worker `a`. From the lint fix on (`a84ba1d`, A10, A11, the closing checks and the
closing parts of these notes for all of A1 to A11): the second worker `a`, after the sandbox restart of
2026-10-12. Head: `b118286`, pushed. Nothing was rebased, amended or reordered.

## A1. The automatic Ballista must not show pits its side does not see — done, `cf3b718`

- Test first: `tests/war.rs::the_automatic_ballista_does_not_show_hidden_pits` failed on the unchanged
  code with target 1 instead of 0 for the attacker's hidden pit (the reason of F-robust-1); the
  property `hidden_obstacles_do_not_show` failed every time with the saved case (now in
  `proptest-regressions/tests/properties.txt`).
- Code: `movement::reachable_in_public` (a walker stops only at `obstacles::public_pits`, the pits
  both sides see: `sees(state, o.side.opponent(), o)`, so with BAT-QUIRK-1 fixed the living natives'
  sight counts as in the observation's `revealed`); `war::can_attack_next_turn` uses it.
  `movement::reachable_within` keeps its behaviour; its body moved into the private generic
  `movement::reach(.., seen: impl FnOnce() -> HexSet)`, one instance per caller.
- Tests: the directed test also pins the Ballista's own side's hidden pit (no change of target) and a
  pit both sides see (the target changes).

## A2. Remove Obstacle is castable only while some hex leads to an obstacle — done, `c010257`

- Test first: `tests/remove.rs::remove_obstacle_is_castable_only_while_a_hex_leads_to_an_obstacle`
  failed at `!castable` (no target, still castable); the harvested
  `h3-pack/tests/harvested_heroes.rs::remove_obstacle_keeps_a_target_while_it_is_castable` failed the
  same way without the fix.
- Code: `NaturalObstacles::any_removable`: an obstacle with a blocked hex not in `cleared`, or with
  its anchor in `anchors`.
- Also: `random_legal_sod_heroes_cast_and_play` (the reviewer's bounded test) takes every generated
  obstacle now; its helper `apart` is gone. It passed with and without the fix (the layout is rare).

## A3. Slayer's specialist bonus saturates — done, `dc0c6d9`

- Test first: `tests/specialist.rs::a_huge_slayer_specialty_does_not_overflow_the_attack` panicked at
  `combat.rs:246` ("attempt to add with overflow").
- Code: two `saturating_add` in `combat::strike_inputs`.
- The test support has the reviewer's hook `ruleset_from` / `try_ruleset_from` now, verbatim from
  `robustness-failing-tests.patch` (if worker `b` took the same hunk, the two are identical).

## A4. A pack cannot make a command that never returns — done, `6fd9ba8`

- Test first: the reviewer's `a_command_returns_when_no_stack_can_ever_act` run as a throwaway on the
  unchanged code: "the command did not return within 10 seconds". The new h3-content cases failed
  too (fear 100 and a permanent Blind were accepted).
- Code (h3-content): `SpellMagic::error` refuses `duration = permanent` on a spell with a change
  of `TimedChange::takes_turns` (blind, petrify, paralyze); `Ability::parameter_error` refuses
  `fear` above 99. h3-battle's `effects::stops_turns` now calls `TimedChange::takes_turns` (one list).
- Tests: h3-content `spell::tests::targeting_is_validated`, `ability::tests::parameter_errors`;
  h3-battle `tests/fear.rs::a_pack_cannot_take_every_turn_away_for_ever` (the reviewer's pack as a
  compile refusal, both checks, exact errors).
- **The battle's in-code test pack tripped check (b)**: its `azure` had `fear=100`. It is neither the
  harvested nor the synthetic pack (those have 10 and compile), so I went on: `azure` is `fear=99`
  now (as tests may not make bad morale certain, HANDOFF "Standing traps"). One test changed:
  `fear_takes_the_largest_chance_of_the_living_frighteners_outside_the_wait_phase` counts the azure's
  rolls over its 1 000 seeds (970..1000) instead of asserting each. The other fear tests keep their
  seeds (the draws are the same; only a draw of 99 would differ).
- `packs/README.md` (not under `docs/`) now states both bounds (`fear` 1..=99; `permanent` not with
  blind, petrify or paralyze).
- Not changed: a `fixed` duration of up to 65 535 rounds on such a change still compiles (bounded:
  r2b F-7 asked for "a bound" there; the decided fix names the permanent duration only).

## A5. An obstacle type the battle cannot hold does not compile — done, `d713543`

- Test first: the reviewer's `the_battle_accepts_every_generated_battlefield` as a throwaway on the
  unchanged code: `Err(TooManyObstacles)`.
- Code: `h3_content::MAX_USUAL_OBSTACLE_HEXES` (8), checked in `validate_obstacle` for usual (not
  absolute) types by the length of `blocked`; `h3_battle::MAX_REMOVABLE_HEXES` is that constant.
- Tests: h3-content `ruleset::tests::terrains_native_terrains_and_obstacles` (9 refused, 8 accepted,
  an absolute one of 9 accepted); h3-battle `tests/terrain.rs::the_battle_accepts_every_generated_battlefield`.
- `packs/README.md` states the bound.

## A6. The battle ends when a round start takes a side's last stack — done, `9983a08`

- Test first: `tests/created.rs::the_battle_ends_when_a_round_start_takes_a_sides_last_stack` (the
  sketch through real commands) failed with `result() == None` after twelve Defends.
- Code: `turn::start_round` calls `combat::check_end` after `created::count_down` (every round
  start; it changes nothing while both sides have a living stack that is no war machine);
  `turn::pick` gives no stack once that ended the battle, so `state.active` is `None`. The comment
  near `turn.rs:69` is corrected.
- `with_quirk_bat_quirk_7_a_crowded_original_becomes_its_own_clone` no longer calls `check_end`.
- Timing against the original (for the rule text, not changed here): the engine ends the battle at
  the round start itself; the original at the winner test after the next action at the latest
  (r4a F-3), so there the other side could still act once.
- Not added: the sketch's property invariant "while the battle is not over each side has a living
  stack that is no war machine" (the property runs `t:quirky`, where BAT-QUIRK-7 is off; worth
  adding together with a profile that has it).

## A7. In the tactics phase Retreat and Surrender act for the trusted actor — done, `c24c322`

- Test first: `tests/flee.rs::in_a_tactics_phase_too_a_side_leaves_only_for_itself` failed with
  `Retreat(Attacker)` for the defender's command (the active stack hypnotized by hand).
- Code: `command::plan_tactics` gets the actor and uses it for both commands.

## A8. The sim's `step` refuses the wrong side before it decodes — done, `600b098`

- Test first: `h3-sim/src/env.rs::a_step_out_of_turn_is_refused_whatever_the_action` failed with
  `InvalidAction` for spell 0 (the oracle of F-sec-2).
- Code: `BattleEnv::step` answers `Rejected(CommandError::NotYourTurn)` for an actor that is not
  `to_move()`, before `action::decode`. The C ABI's `h3_env_step` only calls `BattleEnv::step`, so
  it goes the same way (`H3_ERR_REJECTED` for any action out of turn; pinned in
  `h3-ffi::tests::errors_are_codes_not_panics`). No ABI or encoding change.
- Not changed (same root, no leak today, sec F-sec-2's last evidence line): `action::encode` reads
  the active stack's spell whoever asks; `h3_battle::stack_spell` stays public without a viewer.

## A9. A strike's damage spell of 0 damage does nothing — done, `2d09edb`

- Test first: `tests/strike.rs::a_thunderbolt_of_no_damage_does_nothing` failed with a `StrikeSpell`
  event for a bolt of 0.
- Code: `after_strike::thunderbolt` returns before its events when the damage is 0.
- For the join: the edit moves the `StrikeSpell` emit below the `magic::spell_damage` call; if
  worker `b` changed that call's arguments, the two hunks touch.

## Lint fix after A9 (not a fix of the list) — `a84ba1d`

clippy refused A5's test (`crate::MAX_USUAL_OBSTACLE_HEXES`, `unused_qualifications`, in
`h3-content/src/ruleset/tests.rs`); the test names the constant without its path now. A1 to A9 are
otherwise as the first worker left them (not amended, not reordered).

## A10. Relations to hypnotized and berserk stacks — done, `97cf422`

The first worker's uncommitted draft (13 files) was complete for A10's five points; I checked it
against A10 and the four findings, proved its tests, added one pin and tidied comments.

- Code:
  - `Stack::may_attack` (new, `pub const fn`, `state.rs`): `self.controller() != other.side`. Used by
    `legal::turn_commands`, `command::enemy` (commanded Attack and Shoot) and
    `war::shooting_target` (the automatic Ballista's candidates). No Berserk term.
  - `Stack::is_hostile_to`: either stack berserk, or `may_attack`. Callers: `combat::is_blocked`
    (first = the shooter) and `combat::melee_secondaries` (first = the striker), the two callers
    of 0x442880.
  - `magic::aura`: `unicorn.controller() == neighbour.side`.
  - `war::is_patient`: `patient.side == tent.side` (automatic and commanded tent).
  - `combat::shoot`: `ammo_supplied(state, shooter.side)` (was `.controller()`); `ammo_supplied`
    itself (the cart's own `side`, alive) is unchanged.
  - `war::can_attack_next_turn` (the Ballista's second key, a VCMI port): `berserk ||
    me.may_attack(t)`. **A judgment call**: for a candidate that is not berserk it follows the new
    attack permission (VCMI's `getCanAttackNextTurn` asks `battleCanAttackUnit`); for a berserk
    candidate it keeps today's "any other stack" (its own turn takes any stack), where VCMI has no
    Berserk term. The alternative, plain `may_attack`, would be VCMI's.
  - Doc comments: `effects::Status::BERSERK`, h3-content `Ability::FirstAid`,
    `Ability::MagicResistanceAura`, `TimedChange::Berserk`; `packs/README.md` (`first_aid`,
    `magic_resistance_aura`, the `berserk` and `hypnotize` changes).
- Tests (h3-battle), each failing on the unchanged sources at its first changed assertion (non-test
  files stashed; log `wa/t10-nosrc.log`), all passing with the change:
  - `tests/control.rs::a_hypnotized_stack_is_attacked_by_the_side_that_controls_it_and_not_by_its_own`
    (r3 F-1 scenarios a and b: legal commands and refusals for Attack and Shoot, both sides; the
    hypnotized stack's own targets; the attack carried out).
  - `tests/control.rs::a_hypnotized_stack_blocks_and_is_struck_as_a_stack_of_its_own_side` (scenario c:
    blocking both ways, the hydra of each side).
  - `tests/spells.rs::the_aura_protects_the_stacks_of_the_side_that_controls_the_unicorns` (scenario d,
    r1 F-4, r6 F-1).
  - `tests/war.rs::the_tent_heals_its_own_sides_stacks_whoever_controls_them` and
    `::a_hypnotized_shooter_is_supplied_by_its_own_sides_ammo_cart` (scenario e).
  - `tests/war.rs::the_automatic_ballista_shoots_the_other_sides_stacks_whoever_controls_them` (the
    Ballista part of scenario a and of r3 F-2; I added the direct pins of
    `war::can_attack_next_turn` and `war::shooting_target` at its end).
  - `tests/control.rs::a_berserk_stack_is_no_target_of_its_own_side_and_an_enemy_next_to_any_stack`
    (r3 F-2).
  - `tests/control.rs::a_berserk_stack_attacks_the_nearest_stack_on_its_own_turn`: **the flipped
    test**. It had the attacker's brute attack the attacker's berserk stack; it now asserts that
    this is not legal and refused (`NotAnEnemy`), and the brute is the defender's, so that the
    rest of the test (the retaliation does not end Berserk, the berserk stack's own turn) stays.
- Per assertion (a throwaway run with the assertions made soft, `wa/probe/old.log`, `new.log`): of
  the 58 assertions of these eight tests, 31 fail on the unchanged sources and two `unwrap`s panic
  there; **25 pass either way**. Those pin geometry or behaviour that A10 keeps:
  - the unchanged rest of the flipped test (the other side attacks the berserk stack, the strikes,
    Berserk's end, the walk to its own stack: 8 assertions);
  - in the berserk test: a berserk stack blocks the archers of its own side (no Shoot listed,
    `CannotShoot`), the other side's archers beside it are blocked and strike it, those far away
    shoot it, a hydra strikes its own side's berserk neighbour (the Berserk term of
    `is_hostile_to`, kept: 6 assertions and one of geometry);
  - in the aura test: the chances before any Hypnotize (2), one of geometry, and hypnotized Unicorns
    protecting the side that commands them (the same before and after, as r6 F-1 says);
  - in the Ballista test: the targets without Hypnotize or Berserk, and "still berserk";
  - in the tent test: the controller check of the setup (twice) and "one heal" (the healer differs);
  - one of geometry in the blocking test.
- The other callers, kept as they are (A10 asks for the list):
  - `is_hostile_to` has no other caller. Through `combat::can_shoot` → `is_blocked` the new
    blocking relation also reaches: `legal`/`command` (Shoot or melee), `control::berserk` (does the
    berserk stack shoot), `war::is_commanded` and `war::act`, `war::can_attack_next_turn`,
    `casters::useful` (`EnemyMelee`, `EnemyShooter`, `Melee`, `Shooting`).
  - `controller()` in friend-or-foe tests, unchanged: `morale::frightened` (the frightened stack by
    controller, the Azure Dragons by `side`: BAT-QUIRK-18); `mana::channel` (Familiars count for
    their controller) and `mana::drain` (the hero opposite the Wraith's controller);
    `raise::victim` (a sacrifice victim: of the side and controlled by it); `magic::Guard::of`,
    `magic::share` and the Fire Shield's reflection (`magic.rs` about 1837): the controller's
    hero's defences, resistance and bonuses; `magic::opening` (the opening spells of the hero of
    the first stack's controller); `combat::strike_inputs` and `specialist_terms` (Offense,
    Archery, Armorer, the specialist's Bless and Slayer: the controller's hero);
    `after_strike` (a strike's spell is cast for the striker's controller, twice);
    `casters` (the drawn spell is its controller's secret, a periodic caster and a raiser act for
    their controller, the cast's authority).
  - `controller()` outside friend-or-foe tests: who commands (`legal_commands`, `command::plan`,
    `observe`, h3-sim's observation, h3-protocol's mapping), whose pits a walker sees
    (`movement::reachable_within`, `obstacles::stuck`).
  - Relations by `side` that no report reads otherwise, unchanged: retaliation (no relation test:
    whoever is struck in melee retaliates), breath and area shots (any stack), morale and luck
    (set at the start by side), the spells' side tests (`magic::in_reach` and the target tests:
    the stack's own side), `casters::useful`'s enemies (the other side's stacks), the battle's end,
    Land Mines and the sight of hidden obstacles, the berserk stack's automatic turn (any stack).
- Removed from the draft: nothing. Reworded: the doc comment of `can_attack_next_turn`, the module
  comment of `war.rs` ("a random damaged stack of its side"), two reflowed comments.
- For the coordinator:
  - `may_attack` is a new public method of `Stack` (like `is_hostile_to`, `controller`); no type on
    the wire changed.
  - `is_hostile_to` lost its fast path for stacks that are neither berserk nor hypnotized; the new
    body is one OR and one compare. It is in the hot path (`is_blocked`, `legal`): if `bench-check`
    moves, look here first.
  - `shooting_target` now always finds a candidate while the battle goes on (the other side has a
    living stack by `side`); before, a Ballista whose enemies were all hypnotized had none. The
    property `the_lazy_ballista_target_equals_the_reference` already took its reference candidates
    by `side` (`tests/properties.rs::reference_target`): engine and reference now use the same
    relation (I did not check whether that property's games ever have a hypnotized or berserk stack).

## A11. Expert Dispel reaches hypnotized stacks — done, `b118286`

- Test first: `tests/control.rs::expert_dispel_reaches_hypnotized_stacks_in_every_profile` (both
  sketches in one: profiles `t:faithful` and `t:hypnosis`; the owner's hero and the hero of the
  controlling side cast Expert Dispel at a stack with Hypnotize and Haste). On the unchanged code it
  failed for `t:hypnosis` with `[(["haste", "hypnotize"], Attacker), (["haste", "hypnotize"],
  Attacker)]` for the two casts (the stack skipped: the reason of r2a F-2 and r3 F-3); `t:faithful`
  (the test pack's profile without quirks) passed before and after.
- Code: `magic::skips_hypnotized` returns false for the mass version of a spell whose effects are a
  dispel and nothing else (`spell.magic.effects == [SpellEffect::Dispel]`): Dispel. Cure (a dispel
  and a heal) and the timed spells go on as before (`quirk || !ends_it`). No flag, profile or pack
  format change. I read the loop again with `scripts/exe.py` (the slice 0x5A15CE to 0x5A16D6 of the
  cast routine): two passes, one per side, over the side's records; per record the test 0x5A8090,
  then the end routine for every spell id but 71; no read of the Hypnotize duration.
- **The criterion is a choice** (the pack format has no field for it and may not get one here):
  "only dispels" tells Dispel from Cure by the effect list. The alternatives were the heal effect
  or `restores_health` (Cure follows the quirk because it heals) and "its dispel ends the stack's
  Hypnotize" alone (then a Dispel that spares Hypnotize, or a permanent Hypnotize, would still be
  passed over; the original's loop has no test at all). With the harvested packs all of them give
  the same result.
- Comments: the doc of `quirk.battle.mass_cure_skips_hypnotized` (`h3-content/src/quirks.rs`) says
  that the flag does not cover a spell that only dispels; the comment in
  `mass_versions_pass_over_hypnotized_stacks_and_expert_cure_only_with_q0012` no longer says "the
  other mass versions never do".
- Not done here (other findings): r1 F-3 (`no_target` leaves out hypnotized stacks the original's
  cast test counts) is not in list A.

## Closing checks (at `b118286`)

- `cargo fmt --all -- --check`: clean.
- `cargo clippy -p h3-battle -p h3-content -p h3-sim -p h3-ffi -p h3-pack --all-targets -- -D warnings`:
  exit 0 (after `a84ba1d`; before it h3-content's tests did not pass clippy).
- `cargo test --no-fail-fast -p h3-battle -p h3-content -p h3-heroes -p h3-pack -p h3-sim -p h3-cli -p h3-ffi
  -p h3-protocol` (log `wa/closing-tests-1.log`): h3-battle 459, h3-content 79, h3-heroes 23, h3-pack 17 and
  its five harvested test files (3, 4, 1, 2, 3), h3-sim 30, h3-ffi 5, h3-protocol 61, h3-cli's lib 3: all
  pass. **One failure: h3-cli `replay_fixtures::golden_replays_reproduce_their_hashes`** (below).
- h3-battle's tests three times at the final state: 459 passed each time (`wa/t11-all.log`,
  `closing-tests-1.log`, `closing-battle-3.log`); no property test failed in any run of this session.
- `cargo xtask check-rules` (183 rules, 401 rule markers, 808 covers markers), `check-arch`, `check-stubs`
  (1 registered stub): ok. Also `check-exe-map`: ok (341 entries; the new comments cite 0x476490,
  0x442880, 0x43EA70, 0x43F9E1, 0x5A160F, 0x5A66D0, all covered by existing entries).
- `cargo check --workspace --all-targets`: exit 0 (the other crates still build against the battle's API).
- Not run, as the brief says: coverage, `cargo xtask verify`, `bench-check`. Target directory below 4 GB.

## Golden replays

- **`fixtures/replays/full-s3-a955.toml` no longer verifies**: "command 219 rejected: the target is
  not an enemy". Cause: A10 (r3 F-2). Command 201 of the replay is `D cast 27 hex 141` (spell 27 of the
  synthetic pack is `rampage`, its Berserk); command 219 is `A attack 0 123`: the attacker's random agent
  attacks stack 0, a stack of its own side, which only Berserk allowed (no Charm, spell 28, is cast
  before it). A side may no longer attack its own berserk stack, so the recorded command is refused. The
  fixture must be recorded again (`fixtures/README.md`, agent seed 955); I did not regenerate it.
- The other seven verify at the final state with their recorded hashes (`h3 verify`: `duel-s1`,
  `skirmish-s7`, `full-s3`, `full-s3-a1367`, `full-s3-a1766`, `full-s3-a2378`, `full-s3-a303`).
- At `2d09edb` (after A1 to A9) all eight verify (`h3` built there and run on each file): A1 to A9 changed
  no golden replay. A11 changes none either (it cannot refuse an attack, and the seven still verify).
- Benchmark baselines: `bench-check` not run. Step counts of `h3 bench --games 20` (debug build, default
  seeds), as the lane's trap asks before a new baseline: `full` 7388 at the base `68838df` and after A9
  (`2d09edb`), **7297 after A10** (`97cf422`) and after A11; `duel` 3584 and `skirmish` 7789 at all four.
  So A1 to A9 and A11 leave the three workloads' step counts as they were, and A10 changes the `full` games
  (Berserk and Hypnotize are cast there): `benches/battle/random-rollouts-full` will need a new baseline
  for that reason, whatever the Ir per step does.

## Rule texts, ledger entries and `installation.md` sentences that are now wrong (A1 to A11)

Old clauses are quoted from the branch's docs (base `68838df`); the new wording is a proposal. Where a
fix made the code agree with a text that was already right, it says "none".

### A1 (the automatic Ballista and hidden pits)

- **BAT-WAR-002**, summary. Old: "then stacks that could attack next turn (can shoot, or can reach an
  attack position)". New: "then stacks that could attack next turn (can shoot, or can reach an attack
  position on a stack they may attack, BAT-MAG-026, a berserk one on any other stack; their reach as both
  sides can tell it: only the pits both sides see stop a walker here, so the choice, which both sides are
  told, shows no hidden Quicksand, BAT-MAG-031)". (The middle part is A10's, see there.)
- **BAT-MAG-031**, summary, after "What a side does not see never changes what it may do: its walkers'
  paths and stops use the pits their controller sees (a hypnotized stack's controller's), and an obstacle
  spell's target test ignores the hidden obstacles the caster's side does not see (…)." Add: "Nor does
  the automatic Ballista's choice of target read a pit hidden from either side (BAT-WAR-002)." The
  berserk stack's automatic turn still walks by the pits its controller sees (sec F-sec-1 part a,
  r3 F-11: not decided), so the sentence should name the Ballista only.
- `docs/lanes/battle/HANDOFF.md`, "Standing traps": "`hidden_obstacles_do_not_show` checks the acting
  side's commands and casts" could add that the Ballista's key reads `movement::reachable_in_public`.

### A2 (Remove Obstacle castable)

- None. BAT-MAG-034 already says "A hex with nothing to remove is no target
  (CommandError::InvalidSpellTarget), and the spell is not castable without one"; the code now does that
  for an obstacle orphaned by a removal under BAT-QUIRK-3.

### A3 (Slayer's specialist bonus)

- None (a saturating addition; no rule text speaks of the range).

### A4 (a pack cannot take every turn away)

Nothing is wrong, but two limits of a ruleset are new and unwritten in the catalog (BAT-MOR-004 has the
model sentence: "A ruleset must keep every bad-morale chance below morale_dice: a certain freeze could
leave no stack able to act.").

- **BAT-MAG-060** (Fear), add at the end: "A ruleset must keep every fear chance below 100 % (the
  compiler refuses `fear` above 99): a certain freeze could leave no stack able to act (as BAT-MOR-004)."
- **BAT-MAG-024** (Blind), add at the end, and a reference in **BAT-MAG-043**: "A spell whose change
  takes a stack's turns (blind, petrify, paralyze) cannot have a permanent duration (the compiler refuses
  it): stacks that never act again could leave a battle without an end."
- **BAT-MAG-010**, "a permanent spell (Disrupting Ray) lasts the rest of the battle": add "(not allowed
  for a change that takes a stack's turns, BAT-MAG-024)".

### A5 (obstacle types of more than eight hexes)

- **BAT-GEO-004**, summary. Old: "Obstacle types (terrains, width, height, blocked hexes, absolute flag)
  are content." New: "Obstacle types (terrains, width, height, blocked hexes, absolute flag) are
  content; a usual type blocks at most 8 hexes (the compiler refuses more: a removable piece of the
  battle holds 8, BAT-GEO-003), an absolute one any number."

### A6 (the battle's end at a round start)

- **BAT-END-001**, summary. Old: "After every action (a whole melee sequence with retaliation and second
  attack, or a whole shot sequence; as VCMI checks after each action), if one side has no living stacks
  the battle ends and the other side wins (BattleResult::Victory); no stack is active afterwards and
  every command is rejected." New: "After every action (…as VCMI checks after each action) and at every
  round's start after the created stacks' countdown (with quirk BAT-QUIRK-7 a turned original can vanish
  there as its side's last stack, BAT-MAG-038), if one side has no living stacks the battle ends and the
  other side wins (BattleResult::Victory); no stack is active afterwards and every command is rejected.
  The original asks for the winner after every action, so there the other side still acts once before
  the end (r4a F-3: the dispatcher's call at 0x479BDC); ending at the round start itself is the engine's
  choice."
- **BAT-MAG-038**, summary. Old: "linked to itself, so that it vanishes without a corpse when they run
  out;". New: "linked to itself, so that it vanishes without a corpse when they run out (as its side's
  last stack that ends the battle at that round start, BAT-END-001);".
- **Ledger BAT-QUIRK-7**, "Regression test": add `crates/h3-battle/src/tests/created.rs`,
  `the_battle_ends_when_a_round_start_takes_a_sides_last_stack`.

### A7 (Retreat and Surrender in the tactics phase)

- None is wrong: the case is not reachable today (no stack is hypnotized in a tactics phase). BAT-FLE-001's
  "when one of its stacks is active or its tactics phase runs" could say "when a stack it commands is
  active (BAT-MAG-026)" to match the code in both phases.

### A8 (the sim's step out of turn)

- None found in the catalog, the ledger, `installation.md`, `docs/06-security.md` (its "the seat not to
  act gets `NotYourTurn`" is the server's and now also the sim's) or `docs/07-simulation-and-ai.md`.

### A9 (a strike's damage spell of no damage)

- **BAT-MAG-041**, summary. Old: "then the damage without a hero's bonuses and with the target's extra
  damage, reduction and school protection (BAT-MAG-004); StrikeSpell and SpellDamage report it, and what
  damage ends ends (BAT-MAG-024, BAT-MAG-038)." New: "…(BAT-MAG-004); a damage that comes to 0 (one
  Thunderbird against Diamond Golems: 10 less 95 %) does nothing: no event, nothing ends; otherwise
  StrikeSpell and SpellDamage report it, and what damage ends ends (BAT-MAG-024, BAT-MAG-038)."
- **`installation.md` §7.20**, "The striker's effects", Thunderbird. Old: "then the hit points are taken
  (0x444090) and the kills reported." New: "then, if the result is above 0 (else the routine leaves
  without a report, 0x4411FD; r4b F-8), the hit points are taken (0x444090) and the kills reported."
  (0x4411FD is covered by the map's entry for 0x440BC0, the striker's effects; the first worker's
  comment in `after_strike.rs` cites it too. I did not read it myself.)

### A10 (relations to hypnotized and berserk stacks)

- **BAT-MAG-025**, summary. Old: "While active: every other stack is the berserk stack's enemy and it is
  every stack's (its own side may attack it; it blocks adjacent shooters and they block it, BAT-RNG-001; a
  multi-headed berserk stack strikes every adjacent stack, BAT-ATK-003; the automatic Ballista may shoot
  it, BAT-WAR-002)." New: "While active: beside another stack it counts as that stack's enemy and the
  stack as its, whatever their sides (it blocks adjacent shooters and they block it, BAT-RNG-001; a
  multi-headed berserk stack strikes every adjacent stack, and a multi-headed stack strikes a berserk
  one beside it, BAT-ATK-003). Who may attack or shoot it does not change: the other side's stacks may
  as ever, its own side's may not (the original's cursor has no Berserk test), and its own side's
  automatic Ballista does not shoot it (BAT-WAR-002)."
- **BAT-MAG-025**, source `original-data`. Old: "0x442880 makes a berserk stack every stack's enemy;". New:
  "0x442880, which only the blocked-shooter test (0x469600) and the neighbour test of multi-headed
  strikes ask, answers yes when either stack is berserk; the cursor's target test (0x476490: the target's
  own side against the acting side) has no Berserk test;". A `vcmi` source could be added:
  `lib/battle/CBattleInfoCallback.cpp` `battleCanAttackUnit` (no Berserk exception), `battleCanShoot`
  (an exception only for a berserk attacker), `server/battles/BattleFlowProcessor.cpp` (the automatic
  Ballista compares owners).
- **BAT-MAG-026**, summary. Old: "The stack is the controller's friend and the owner's enemy (attacks,
  retaliation partners, blocking, the berserk relation of BAT-MAG-025); these use the controller's hero:
  Offense or Archery of its strikes and Armorer against strikes on it (BAT-HER-004), a specialist's
  Bless and Slayer terms (BAT-MAG-020), the hero resistance and the Unicorn aura of the same controller
  against spells on it (BAT-MAG-008), the bonuses of its Fire Shield's reflection (BAT-MAG-018), its
  controller's Ammo Cart (BAT-WAR-004) and First Aid Tent (BAT-WAR-003)." New: "On its turns it fights
  for its controller: it may attack and shoot the stacks of its own side and none of its controller's.
  To other stacks it stays a stack of its own side: who may attack or shoot a stack is decided by the
  acting stack's controller against the other stack's own side, so the side that controls it may attack
  and shoot it and its own side may not (batch 2c H1), the automatic Ballista included (BAT-WAR-002);
  beside another stack it counts as an enemy when that stack's controller is not its own side (it blocks
  the shooters of the side that controls it and not those of its own side, BAT-RNG-001; a multi-headed
  strike of the side that controls it reaches it, BAT-ATK-003), or when either is berserk
  (BAT-MAG-025). Unicorns protect the adjacent stacks whose own side is the Unicorns' controller
  (BAT-MAG-008): hypnotized, a stack keeps its own side's aura, and hypnotized Unicorns protect the side
  that controls them. Its own side's First Aid Tent heals it, its controller's does not
  (BAT-WAR-003, BAT-WAR-007), and its own side's Ammo Cart supplies it (BAT-WAR-004). The controller's
  hero counts for: Offense or Archery of its strikes and Armorer against strikes on it (BAT-HER-004), a
  specialist's Bless and Slayer terms (BAT-MAG-020), the hero resistance against spells on it
  (BAT-MAG-008), the bonuses of its Fire Shield's reflection (BAT-MAG-018)."
- **BAT-MAG-026**, source `original-data`. Old: "the controller (0x442690, inline side ^ hypnotized) in
  0x442880, 0x465389, the damage stages (0x443898, 0x443DE6), 0x5A8090 → 0x44A4D0 (resistance,
  Pendants), 0x4224AF; the real side (+0xF4) in the side tests (0x5A82F8) and the turn order
  (0x46502A)". New: "the controller (0x442690, inline side ^ hypnotized) in 0x465389, the damage stages
  (0x443898, 0x443DE6), 0x5A8090 → 0x44A4D0 (resistance, Pendants), 0x4224AF; the controller of the
  one stack against the real side (+0xF4) of the other in 0x442880 (the shooter's or striker's
  controller, the neighbour's side), in the cursor 0x476490 (attack, shot and the tent's patient: the
  acting side against the target's side) and in the aura lists 0x43EA70 (the Unicorn's controller, the
  neighbour's side); the real side in the side tests (0x5A82F8), the turn order (0x46502A), the automatic
  tent's loop and the shot's Ammo Cart test (0x43F9E1)". Its `observation` source can add H1's last
  line ("While hypnotized, that stack can attack its original side, but Blue's stacks cannot attack the
  hypnotized stack"), which the fold-in left out; a `vcmi` source: `lib/battle/CBattleInfoEssentials.cpp`
  `battleMatchOwner` (the attacker's owner through hypnosis against the defender's initial side).
- **BAT-MAG-008**, summary. Old: "times 0.8 once if an adjacent allied stack (of the same controller) has
  a magic_resistance_aura (Unicorns 20 %; the strongest counts)". New: "times 0.8 once if an adjacent
  stack whose controller is the stack's own side has a magic_resistance_aura (Unicorns 20 %; the
  strongest counts; a hypnotized stack keeps its own side's aura, BAT-MAG-026)". The hero's part of the
  same sentence ("the resistance of the hero of the stack's controller") stays.
- **BAT-WAR-002**, summary. Old: "Target: among living enemies prefer stacks that are not war machines".
  New: "Target: among the living stacks of the other side, whoever controls them (its controller
  against the target's own side, BAT-MAG-026: not its own side's hypnotized or berserk stack, the other
  side's hypnotized stack is one), prefer stacks that are not war machines". With A1's clause for "could
  attack next turn" above; the source `vcmi` could say that the berserk candidate's "any stack" is ours.
- **BAT-WAR-003**, summary. Old: "a uniformly chosen allied stack that is alive, not a war machine and
  whose top creature is hurt". New: "a uniformly chosen stack of its own side (also one the other side
  controls, and no stack of the other side that its side controls, BAT-MAG-026) that is alive, not a war
  machine and whose top creature is hurt".
- **BAT-WAR-007**, summary. Old: "when the tent becomes active and some allied stack is a patient (alive,
  not a war machine, its top creature hurt; BAT-WAR-003)". New: "when the tent becomes active and some
  stack of its side is a patient (of its own side whoever controls it, alive, not a war machine, its top
  creature hurt; BAT-WAR-003)".
- **BAT-WAR-004**, summary. Old: "While a side has a living Ammo Cart (ability ammo_supply), its stacks do
  not use up shots." New: "While a side has a living Ammo Cart (ability ammo_supply), its own stacks do
  not use up shots, also those the other side controls; a stack of the other side that it controls is not
  supplied (BAT-MAG-026: the original asks the hero of the shooter's own side)."
- **BAT-RNG-001**, summary, "no living enemy stack touches its footprint (an adjacent enemy blocks it …)"
  and "Then it may shoot at any living enemy stack": "enemy" has two meanings now. Add: "An adjacent
  stack blocks it if either of the two is berserk or the adjacent stack's own side is not the shooter's
  controller; it may shoot at the living stacks whose own side is not its controller (BAT-MAG-025,
  BAT-MAG-026)."
- **BAT-ATK-003**, summary, "attacks_all_adjacent hits every living enemy stack whose footprint touches
  the attacker's; three_headed_attack hits enemy stacks on the two hexes beside the attacked hex". Add:
  "An enemy here is a stack whose own side is not the striker's controller, and any stack when either
  of the two is berserk (BAT-MAG-025, BAT-MAG-026)."
- **`installation.md` §7.14**, Berserk. Old: "- every other stack is its enemy and it is every stack's
  enemy (0x442880 answers yes when either stack is berserk);". New: "- beside another stack it counts as
  an enemy either way (0x442880 answers yes when either stack is berserk; its only callers are the
  blocked-shooter test and the neighbour test of multi-headed strikes); who may attack or shoot it is the
  cursor's test (0x476490: the target's own side against the acting side), which has no Berserk test, so
  its own side cannot;".
- **`installation.md` §7.14**, Hypnotize. Old: "rebuilds the stack's Unicorn aura lists (+0x514/+0x524,
  the adjacent Unicorns of the same controller; 0x43EA70)". New: "rebuilds the stack's Unicorn aura lists
  (+0x514/+0x524: the adjacent Unicorns whose controller is the stack's real side, and for a Unicorn the
  neighbours whose real side is its controller; 0x43EA70)".
- **`installation.md` §7.14**, Hypnotize. Old: "and these use the controller and its hero: friend and enemy
  (0x442880), the Offense or Archery and the Bless specialist of its strikes and the Armorer of strikes
  on it (stage 1 and 3, 0x443898, 0x443DE6), the hero's resistance and the Pendants against spells on
  it (0x5A8090 → 0x44A4D0), Dispel's Sphere test (§7.10), the target's hero in spell damage and Fire
  Shield (0x5A0D1C, 0x4224AF), the Unicorn aura. These keep the real side (+0xF4): the stack's
  statistics set at the battle's start (its hero's primary skills, morale and luck), the turn order's
  tie-break (0x46502A), and the side tests of spells (0x5A82F8)". New: "and these use the controller and
  its hero: the Offense or Archery and the Bless specialist of its strikes and the Armorer of strikes on
  it (…), the hero's resistance and the Pendants against spells on it (…), Dispel's Sphere test (§7.10),
  the target's hero in spell damage and Fire Shield (…). Relations between two stacks take the
  controller of the one and the real side (+0xF4) of the other: 0x442880 (the first stack's controller,
  the second's side; asked for the blocked shooter and for multi-headed strikes), the cursor 0x476490
  (the acting side against the target's side for an attack or a shot, equal sides for the tent's
  patient; the pathfinder's attack targets alike) and the aura lists (above). These keep the real side:
  the stack's statistics set at the battle's start (…), the turn order's tie-break (0x46502A), the side
  tests of spells (0x5A82F8), the automatic tent's loop over its side's records and the shot's Ammo Cart
  test (0x43F9E1)".
- **`docs/exe/map.toml`**, entry 0x442880 (`stacks_are_enemies`). Old summary: "Whether two stacks are
  enemies: by controller; yes when either is berserk." New: "Whether the second stack is the first's
  enemy: the first's controller against the second's own side; yes when either is berserk. Asked by the
  blocked-shooter test and the neighbour test of multi-headed strikes only." (The proposals above cite
  only addresses the map covers. 0x523B20, the neighbour test of multi-headed strikes, and 0x473EA0, the
  automatic tent, which the reviewers name, have no entry: citing them needs one.)
- `docs/epics/E04b-log.md`, "Batch 2c folded in": H1's last line was not taken up (r3 F-1).

### A11 (Expert Dispel and hypnotized stacks)

- **BAT-MAG-026**, summary. Old: "Mass versions pass over hypnotized stacks (BAT-MAG-003); with quirk
  Q-0012 (faithful) Expert Cure does too, as the original's own loop does (survey S-13), without it
  (strict-fixed) a mass version whose dispel ends the stack's Hypnotize reaches the caster's hypnotized
  stacks." New: "The mass versions of the timed spells pass over hypnotized stacks (BAT-MAG-003). Expert
  Dispel does not, in any profile: the mass version of a spell that only dispels reaches them and ends
  their Hypnotize, as the original's own loop, which tests none (BAT-MAG-016). With quirk Q-0012
  (faithful) Expert Cure passes over them, as the original's own loop does (survey S-13); without it
  (strict-fixed) a mass version whose dispel ends the stack's Hypnotize reaches the caster's hypnotized
  stacks."
- **BAT-MAG-026**, source `original-data`. Old: "the mass handler 0x5A66D0 and Expert Cure (0x5A1847) skip
  hypnotized stacks". New: "the mass handler 0x5A66D0 and Expert Cure (0x5A1847) skip hypnotized stacks;
  Dispel's Expert loop over both sides' records (from 0x5A160F) does not".
- **BAT-MAG-003**, summary. Old: "like a battle-wide spell, except hypnotized stacks (BAT-MAG-026;
  Forgetfulness's mass version from Advanced under quirk Q-0011, BAT-MAG-023)". New: "like a battle-wide
  spell, except hypnotized stacks (BAT-MAG-026; not so for a spell that only dispels: Expert Dispel
  reaches them, BAT-MAG-016; Forgetfulness's mass version from Advanced under quirk Q-0011,
  BAT-MAG-023)".
- **BAT-MAG-016**, summary. Old: "at Expert (mass) it reaches every living stack of both sides;". New: "at
  Expert (mass) it reaches every living stack of both sides, hypnotized ones too (its own loop in the
  original tests no Hypnotize, unlike the other mass versions, BAT-MAG-026);". Its `original-data`
  source, "at Expert on every stack whose test 0x5A8090 is above 0", can add ", hypnotized or not".
- **Ledger Q-0012**, "Original behavior". Old: "Expert Cure loops over the caster's own stacks and skips
  those under Hypnotize (0x5A1847), as the mass handler of every other mass version does (0x5A66D0), so
  it never frees a hypnotized stack;". New: "Expert Cure loops over the caster's own stacks and skips
  those under Hypnotize (0x5A1847), as the mass handler of the timed spells does (0x5A66D0), so it never
  frees a hypnotized stack; Dispel's Expert loop (from 0x5A160F) has no such test, so Expert Dispel frees
  it in every profile (BAT-MAG-016);".
- **Ledger Q-0012**, "Intended/expected behavior". Old: "(strict-fixed: a mass version whose dispel ends
  the stack's Hypnotize reaches it; the other mass versions still pass over hypnotized stacks)". New:
  "(strict-fixed: a mass version whose dispel ends the stack's Hypnotize reaches it; the mass version of
  a spell that only dispels, Dispel, reaches hypnotized stacks with and without the flag; the mass
  versions of the timed spells still pass over them)". "Regression test": add
  `expert_dispel_reaches_hypnotized_stacks_in_every_profile`.
- **`installation.md` §7.14**, Hypnotize. Old: "Mass versions (0x5A66D0, §7.9) skip hypnotized stacks, and
  so does Expert Cure, which loops over the caster's own stacks (0x5A1847, +0x5754 = Hypnotize's
  duration): an Expert Cure never reaches them (quirk S-13), a single Cure does." New: "The mass
  versions of the timed spells (0x5A66D0, §7.9) skip hypnotized stacks, and so does Expert Cure, which
  loops over the caster's own stacks (0x5A1847, +0x5754 = Hypnotize's duration): an Expert Cure never
  reaches them (quirk S-13), a single Cure does. Dispel's Expert loop (§7.10, from 0x5A160F: two passes,
  one per side, over the side's records) reads no Hypnotize duration: an Expert Dispel reaches them and
  ends their Hypnotize."
- **`installation.md` §7.10**, Dispel, "at Expert it does the same for every stack of both sides that it
  can affect (0x5A8090 above 0, **no resistance roll**)": not wrong; add "hypnotized stacks included".

## Open points and what the coordinator should know

- **The golden replay `full-s3-a955`** must be recorded again (A10), and the baseline of
  `random-rollouts-full` renewed for the same reason (the step counts under "Golden replays").
- **Two choices of mine inside the list**, both said above: `can_attack_next_turn` keeps "any stack" for a
  berserk candidate (A10); "a spell that only dispels" is what tells Dispel from Cure (A11).
- **The join with worker `b`**: A10 and A11 add tests in the middle of `tests/control.rs` (after
  `a_berserk_stack_attacks_the_nearest_stack_on_its_own_turn`, after
  `a_hypnotized_stack_fights_for_the_other_side`, after `mass_versions_pass_over_…`), `tests/spells.rs`
  (after `resistance_adds_the_heros_…`) and `tests/war.rs` (a helper `give` after `defend_round`, tests
  after `the_automatic_ballista_does_not_show_hidden_pits`, after `the_tent_heals_at_most_…`, before
  `the_ammo_cart_keeps_shots_full_…`). In sources A10 touches `combat.rs` (`is_blocked`,
  `melee_secondaries`: comments; `shoot`: the Ammo Cart line and its doc), `magic.rs` (`aura`), A11
  `magic.rs` (`skips_hypnotized`). A9's note on `after_strike::thunderbolt` stands.
- `Stack::may_attack` is new and public; `Stack::is_hostile_to` keeps its name with a narrower meaning
  (blocking and multi-headed strikes). No type on the wire, no version, no flag, no profile changed.
- Not done, seen on the way (not in list A): r1 F-3 (`no_target` and hypnotized stacks); the berserk
  stack's automatic turn and hidden pits (sec F-sec-1 part a); a `fixed` duration of up to 65 535 rounds
  on a turn-taking change still compiles (A4's note); the property invariant "each side has a living
  stack while the battle goes on" (A6's note).
