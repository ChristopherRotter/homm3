# Fix notes of worker `a` (E04b T11b, list A)

Worktree `/home/claude/wt-a`, branch `battle/e04b-t11b-a` (pushed after every commit), base `68838df`.
(In progress: this file is rewritten as the fixes land.)

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
