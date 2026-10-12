# Fix list A: hidden information, legal commands, robustness, relations to controlled stacks

Worker name: `a`. Worktree: `/home/claude/wt-a`. Branch: `battle/e04b-t11b-a`.
Reports are in `<SCRATCH>/review/`; "robust" = `robustness.md`, "sec" = `security.md`,
"r3" = `r3-control-battlefield.md`, and so on.

Do them in this order.

## A1. The automatic Ballista must not show pits its side does not see (robust F-robust-1, sec F-sec-1 part b)

`war::can_attack_next_turn` computes a candidate's reach with the pits the *candidate's*
controller sees (`movement::reachable` → `obstacles::seen_pits(state, mover.controller())`), so the
public choice of target shows hidden Quicksand, and the existing property
`hidden_obstacles_do_not_show` fails in about 1 of 83 runs.

Intended: this rule-driven choice reads only what both sides see: the reach for "could attack next
turn" stops at revealed pits only (neither side's hidden ones; using the Ballista's own side's
hidden pits would turn the leak round). A commanded stack's reach is unchanged.

Tests: `the_automatic_ballista_does_not_show_hidden_pits` from the robustness patch; add the
saved case of `robustness-hidden-obstacles-regression.patch` to the property's regression file.
Do **not** change the berserk stack's automatic turn (sec F-sec-1 part a, r3 F-11): the user
decides that one.

## A2. Remove Obstacle is castable only while some hex leads to an obstacle (robust F-robust-2, r3 F-7)

`NaturalObstacles::any_removable` (`natural.rs`) must agree with `removable_at` for every field:
under quirk BAT-QUIRK-3 a removal clears the removed obstacle's anchor hex, which can orphan
another obstacle. Tests: `a_castable_remove_obstacle_has_a_target` and the harvested-pack test
`remove_obstacle_keeps_a_target_while_it_is_castable` from the robustness patch.

## A3. Slayer's specialist bonus saturates (robust F-robust-6, r2b F-7 point 1)

`combat.rs` about line 246: a saturating addition. Test from the robustness patch
(`a_huge_slayer_specialty_does_not_overflow_the_attack`).

## A4. A pack cannot make a command that never returns (robust F-robust-4, r2b F-7 point 2)

The content compiler refuses (a) a permanent duration on a timed change that takes a stack's turns
away (the blind, petrify and paralyze kinds) and (b) a `fear` chance of 100, as it already refuses
a certain bad-morale freeze (`crates/h3-content/src/ruleset.rs`, near line 1378;
`ability.rs` near 718). Tests in `h3-content` (and `h3-pack` if the check sits there). The test
`a_command_returns_when_no_stack_can_ever_act` of the patch becomes a compile refusal.
If the harvested or synthetic packs trip the new checks, stop and report.

## A5. An obstacle type the battle cannot hold does not compile (robust F-robust-5)

The compiler refuses a usual obstacle type that blocks more hexes than the battle's
`MAX_REMOVABLE_HEXES` (8, `h3-battle/src/natural.rs`). `h3-content` cannot depend on `h3-battle`:
put the bound where both can read it (a `pub const` in `h3-content`, used by `natural.rs`).

## A6. The battle ends when a round start takes a side's last stack (r4a F-3)

With quirk BAT-QUIRK-7 a turned original can vanish at a round start (`turn::start_round` →
`created::count_down`) as its side's last stack; nothing checks the end, and the battle can go on
for ever. Intended: the end of the battle is checked there (and no stack is active once it is
over). Test from r4a F-3's sketch, through real commands (the existing test calls `check_end` by
hand). Correct the comment near `turn.rs:69`.

## A7. In the tactics phase Retreat and Surrender act for the trusted actor (sec F-sec-3)

`command.rs` about lines 372–373 use `mover.side`; use the actor, as outside the tactics phase.
Test from the sketch.

## A8. The sim's `step` refuses the wrong side before it decodes (sec F-sec-2)

`h3-sim/src/env.rs`: `BattleEnv::step` answers `Rejected(CommandError::NotYourTurn)` when the
actor is not the side to move, before decoding the action, so the kind of refusal no longer shows
a Faerie Dragon's drawn spell. Check that the C ABI's step goes the same way. Test from the sketch.

## A9. A strike's damage spell of 0 damage does nothing (r4b F-8 part a)

`after_strike::thunderbolt`: when the damage comes to 0, no event and no further effect, as
`extra_damage` does (the original leaves at 0x4411FD when the modified damage is not above 0).
Test: one Thunderbird-like striker against a creature whose spell damage reduction brings 10 to 0.

## A10. Relations to hypnotized and berserk stacks (r3 F-1, F-2; r1 F-4; r6 F-1)

Read r3 F-1 and F-2 in full first, then r1 F-4 and r6 F-1. The engine's `Stack::is_hostile_to`
compares both stacks' controllers. The original (re-read by three reviewers and by the coordinator:
0x442880; the cursor 0x476490; the aura lists 0x43EA70; VCMI's `battleMatchOwner` agrees; batch 2c
row H1's last line observed the first point) uses the *acting* stack's controller against the
*other* stack's real side:

- **May attack or shoot it** (a commanded `Attack` or `Shoot`: `legal.rs`, `command.rs`; the
  automatic Ballista's candidates): `mover.controller() != target.side`. No Berserk term: a side
  cannot attack or shoot its own berserk stack; the other side can as before. A berserk stack's own
  automatic turn still takes any stack.
- **Counts as an enemy** for a shooter's blocking and for the secondary targets of multi-headed
  strikes (the two callers of 0x442880): either stack berserk, or
  `me.controller() != other.side`.
- **The Unicorns' aura** protects a neighbour when `unicorn.controller() == neighbour.side`.
- **The First Aid Tent** (automatic and commanded) heals the stacks of its own side:
  `patient.side == tent.side`, hypnotized or not.
- **The Ammo Cart** that supplies a shooter is the one of the shooter's own side (today: of its
  controller); keep today's test of the cart itself.

Scenarios (a) to (e) of r3 F-1 are the tests; flip `tests/control.rs` about line 152 (a side
attacking its own berserk stack is now refused). Look at every other caller of `is_hostile_to`
and of `controller()` in friend-or-foe tests: where the reports give no reading (morale, luck,
retaliation, breath and area shots, spells' own-side tests, the heroes' bonuses), keep today's
behaviour and list the caller in your notes. The reviewers checked that the heroes' Offense,
Armorer, resistance and artifact defences go by the controller: leave those.

## A11. Expert Dispel reaches hypnotized stacks (r2a F-2, r3 F-3)

`magic::skips_hypnotized` applies quirk Q-0012 (`quirk.battle.mass_cure_skips_hypnotized`) to
every mass version. The original skips hypnotized stacks in the mass handler of the timed spells
and in Expert Cure's loop, but Dispel's Expert loop has no such test (0x5A160F–0x5A16D0). Intended:
a mass *dispel* (Dispel) reaches hypnotized stacks in every profile; Cure follows the quirk as
today; the other mass versions pass over them as today. Tests from both sketches (both profiles).

## Not yours

The spell damage arithmetic, effects of dead stacks, the Phoenix, morale, the Behemoths, Sacrifice,
Clone and the Harpies' return are worker `b`'s. The berserk walker's hidden pits, BAT-QUIRK-2,
BAT-QUIRK-9, the Defend bonus, the Faerie Dragon's F key and everything else in the reports wait
for the user.
