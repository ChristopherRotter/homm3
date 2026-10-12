# Independent check of the 24 fixes (E04b T11b review, lists A, B and C)

State checked: `/home/claude/homm3`, branch `battle/e04b-t11b`, head `bc2d951`, against
`origin/master`. Read-only: no file of the repository was changed and no cargo command was run.
Method: the whole engine diff (26 files of `h3-battle`, `h3-content`, `h3-sim`, `h3-ffi`) read line
by line with every caller of each changed function, the fixes' tests, and the catalog entries of the
rules they touch. The original's behaviour is taken from the fix lists and findings as given.
One thing was computed: the arithmetic of B5 and B6 with an independent exact-rational model that
uses no code of the repository (`<SCRATCH>/review/verify-stage1.py`, Python, outside the repo).

Result in short: I found no fix whose code does something other than its list item asks. Two
defects are consequences of list C that nobody took up (the observation, the rule texts); four
doubts are corners or decisions; none of them is in a hot path of the harvested packs.

## Verdicts

| Fix | Verdict | Basis (file:line at `bc2d951`) |
|---|---|---|
| A1 | ok | `war.rs:109` asks `movement::reachable_in_public` (`movement.rs:101-110`), which stops a walker only at `obstacles::public_pits` (`obstacles.rs:309-316`: pits the other side of the pit's side sees). The commanded reach (`movement.rs:83-94`) is unchanged. Nothing else in `can_attack_next_turn` reads a hidden obstacle. Test `tests/war.rs:362`. |
| A2 | ok | `natural.rs:168-172` is equivalent to "some hex has `removable_at`" (`natural.rs:178-185` with `remove` and `refresh`, `natural.rs:191-226`: `anchors` never holds a cleared hex). The observation's `removable_at` (`observe.rs:147-154`) is the same test. |
| A3 | ok | `combat.rs:246-251`, two saturating additions; test `tests/specialist.rs:218`. |
| A4 | ok | `h3-content/src/spell.rs:981-989` refuses a permanent duration with a change of `TimedChange::takes_turns` (`spell.rs:278`); a permanent entry has no other source than that duration (`effects.rs:352-359`). Fear is 1..=99 (`ability.rs:722-726`). `effects::stops_turns` (`effects.rs:664-666`) now reads the same list. The harvested Blind, Stone Gaze and Paralyze are not permanent. |
| A5 | ok | `h3-content/src/ruleset.rs:271` (`MAX_USUAL_OBSTACLE_HEXES = 8`), used by `validate_obstacle` (`ruleset.rs:1332-1335`) and by `h3-battle/src/natural.rs:26`. |
| A6 | ok, doubt Q1 | `turn.rs:97-98` checks the end after the clones' countdown; `pick` (`turn.rs:183-188`) returns no stack once the result is set, so `activate` leaves `active` at `None` (`turn.rs:258`) and no loop goes on. Test `tests/created.rs:346` goes through real commands and fails without the check (the result stays `None` after its twelve commands). |
| A7 | ok | `command.rs:374-375` pass the actor; the listing already used it (`legal.rs:65-70`). |
| A8 | ok | `h3-sim/src/env.rs:225-230`: `to_move()` is `None` exactly when `is_done()` (`env.rs:153-164`), then the actor is compared before `action::decode`. `decode` has no other production caller (grep: only `env.rs:232`); `h3_env_step` goes through `step` (test in `h3-ffi/src/lib.rs:789-811`). The masks were per actor already (`action.rs:315-462`). |
| A9 | ok | `after_strike.rs:236-246`: at 0 damage no event and no further effect; the value is computed without a draw, so no random draw moves. |
| A10 | ok, two notes | `Stack::may_attack` and the narrowed `Stack::is_hostile_to` (`state.rs:727-740`). All call sites match the list: `legal.rs:99`, `command.rs:175` (attack, shoot), `war.rs:178` (the automatic Ballista's candidates), `war.rs:113` (next turn: `berserk || may_attack`), `combat.rs:43` and `combat.rs:632` (blocking, multi-headed strikes), `magic.rs:2021` (aura: `n.controller() == stack.side`), `war.rs:63-68` (tent, automatic and commanded through `command.rs:307` and `legal.rs:132-135`), `combat.rs:852` (Ammo Cart by the shooter's side). The berserk stack's own turn takes any stack as before (`control.rs:55-57`). A stack that is neither hypnotized nor berserk behaves exactly as before. BAT-MAG-025, BAT-MAG-026, BAT-WAR-002/003/004, BAT-RNG-001, BAT-ATK-003 and BAT-MAG-008 say what the code does. |
| A11 | ok | `magic.rs:870-873`. The harvested Dispel's effects are exactly `[dispel]` (`packs/base/pack.toml:3449`), Cure's are `[dispel, heal]` (`:3481`), so only Dispel is exempt. |
| B1 | ok | `raise.rs:372-373`: the health is read after `effects::remove_dead`. |
| B2 | ok | `effects.rs:933-951` keeps a dead stack's entries with `rounds == None`; `created.rs:423-428` empties a reused record with `effects::forget` (`effects.rs:956-958`) before the stack is replaced; a raise recomputes from what is left (`raise.rs` `revive`). The two callers of `remove_dead` (`command.rs:480`, `raise.rs:372`) need nothing else. |
| B3 | ok | `effects.rs:871-890`: the state after a round start is the same as before the fix, only the value Poison's tenth is compared with changed. |
| B4 | ok | `morale.rs:303-318`; `h3_kernel::modifier::evaluate` does not depend on the order of its pairs (`h3-kernel/src/modifier.rs:79-99`), so the sum is base + spells, then caps and minimums, then one clamp. |
| B5 | ok | `damage.rs:311-323`. My model gives the list's eight values (10, 3, 4, 1, 5, 0, 0, -1) and agrees for every defense 1..300 at 40 % and 80 %; `tests/abilities.rs:476-494` pins them. |
| B6 | ok, one note | `magic.rs:1730-1809`. My model gives the tests' numbers: 58; 972 (exact 973); 9995; 245, 245, 250; 472 238 801 289 216; 281 495 612 686 336; r1 F-2's 47 and 111. No product can overflow (`X87` saturates at 2^64 - 1, `h3-kernel/src/arith.rs:160-205`; the exact arm uses `checked_mul`). A stack's cast and a side without a hero get the value unchanged, as before. Hand check of batch 2a's 119, 115, 17, 230, 173, 25 under the new stores: unchanged. |
| B7 | ok for what the list asks, doubt Q2 | `raise.rs:87-116` (`own` set only under the quirk), `raise.rs:165-173`, `raise.rs:186-198`. The listing and `apply` share `raisable`. |
| B8 | ok | The clone test is in `magic::affects_stack` (`magic.rs:770-771`), which every caster's path uses; it is gone from `stack_target`, which reaches it through `aimable`. |
| B9 | ok for flyers (the Harpies), doubt Q3 | `combat.rs:760-776`, `obstacles.rs:748-763`, `enter` and `explode` with `ends` (`obstacles.rs:641-711`). The end of the battle is checked once, at `combat.rs:776`. Tests `tests/obstacles.rs:442` (burn, order of events, lost, won, draw) and `tests/mines.rs:448`. |
| B10 | ok | `created.rs:247-251` with `StackTraits::without` (`state.rs:214-216`). |
| C1 | code ok; defect D2 (rule text); decision Q4 | `magic.rs:1674-1695`: some stack `in_reach` with `has_chance` (`magic.rs:918-928`). `castable_spells`, `cast_targets` and `apply` all go through `no_target`. |
| C2 | code ok; defect D2 (rule text) | `obstacles.rs:403-408`: a footprint is taken if the stack lives or its corpse mark is not `NO_CORPSE`. |
| C3 | defect D1 and D2 | `created.rs:120-128` (`reusable`), used by `add` (`created.rs:423`) and `has_slot` (`created.rs:110`). The code does what the list asks; its consequences were not all taken up. |

## Defects

### D1 (low to medium). C3 makes a rule read a state field that the observation leaves out

- Where: `crates/h3-battle/src/created.rs:120-128` reads `state.created.vanished`;
  `created.rs:110` (`has_slot`) and `crates/h3-battle/src/casters.rs:602` make it decide a legal
  command; `crates/h3-battle/src/observe.rs:55-141` has no such field, and `observe.rs:17-20`
  still lists "the creatures a vanished clone took along" as the outcome's bookkeeping that the
  observation leaves out on purpose.
- Intended: the lane's standing rule, `docs/lanes/battle/HANDOFF.md:39`: "The observation is
  complete since T9a: a new state field a rule reads goes into `observe_as` [...], the flat
  encoding (version and ABI bump) and the hygiene property." `h3-sim/src/observation.rs:11-13`
  claims the same of the encoding. FIX-c, C3: "if a consequence needs a state field or changes a
  wire type, stop this fix and report."
- What the code does: before C3 `vanished` was read by `outcome()` only (`state.rs:1182-1206`).
  Now it decides (a) whether a Pit Lord may raise when its side is at the stack limit (BAT-MAG-052)
  and (b) which id the next created stack gets (BAT-MAG-037, and with it the turn order's
  tie-break, BAT-TRN-001). A killed clone and a clone whose rounds ran out look the same in the
  observation: count 0, `clone_rounds` 0, no link, corpse `NO_CORPSE`.
- Smallest scenario: the two halves of `tests/raisers.rs:387`
  (`a_pit_lord_finds_no_slot_in_a_clone_whose_rounds_ran_out`): limit 4, four stacks, the dead
  clone is stack 4. Killed: the fiend may raise from stack 1. Expired: it may not. A viewer cannot
  tell which from an observation, only from the events it remembers.
- Weight: both sides saw how the clone went, so nothing hidden leaks, and the legal masks come from
  the state, so no illegal action results. What breaks is the stated contract (an agent's
  observation no longer determines its legal actions in this corner). There are two ways out, and
  both are the user's call: one bit per stack ("its record stays in use") in `Observation`, with
  an encoding and ABI bump and the server's mapping; or the contract restated in `observe.rs`,
  `observation.rs` and the HANDOFF so that it names this bit as left out. The worker's note says
  "No state field, event, error, wire type, encoding or version changed", which is true and is
  the gap.
- Confidence: high that the dependence exists and the observation lacks it (read from the code).
- Test that shows it (in `tests/raisers.rs`, with the `start` closure of the test at line 387; it
  fails today at the last assertion):

  ```rust
  let mut killed = start();
  hurt(&mut killed, &r, 4, 1);
  let mut expired = killed.clone();
  expired.created.vanished[4] = 3; // what `created::count_down` leaves when the rounds run out
  assert_eq!(targets(&killed, &r, BattleSide::Attacker), vec![1]);
  assert!(targets(&expired, &r, BattleSide::Attacker).is_empty());
  for side in BattleSide::BOTH {
      assert_ne!(crate::observe(&killed, &r, side), crate::observe(&expired, &r, side));
  }
  ```

### D2 (medium for the catalog, no behaviour). The rule texts of C1, C2 and C3 were never written

The docs commits on the branch cover lists A and B only (`23480a1`, `4ee9b7a`); neither docs brief
(`DOCS-d.md`, `DOCS-q.md`) nor their notes mention list C. Worker c's wording proposals are in
`<SCRATCH>/review/fix-notes-c.md`, lines 436 to 569 (its line numbers are of its own branch); I
compared them with the code and they say what the code does. The clauses below are still in the
repository and contradict the code:

| Fix | Where | Stale clause | The code |
|---|---|---|---|
| C1 | `docs/rules/catalog/battle.toml:1920` (BAT-MAG-003) | "takes no target (CastTarget::None) and needs at least one stack it can reach"; the mass version "reaches every stack of its side [...] like a battle-wide spell" | needs a stack in reach with a chance above zero; a hypnotized stack the mass version passes over counts (`magic.rs:1688-1691`). The rule has no source for this clause. |
| C2 | `battle.toml:2653` (BAT-MAG-032; BAT-MAG-033 refers to it) | "a spell obstacle and a stack, living or dead" | a living stack or a corpse; a dead stack without a corpse keeps no hex free (`obstacles.rs:405`). |
| C3 | `battle.toml:2783` (BAT-MAG-037 summary), `:2788` (its original-data ref) | "first (lowest id) dead created stack"; "holds a dead summoned stack and sets" | the first killed one; one whose rounds ran out keeps its record and counts towards the limit (`created.rs:120-128`). |
| C3 | `battle.toml:3132` (BAT-MAG-052) | "a dead created stack's or a new one below the side stack limit" | a killed created stack's (`created.rs:110`). |
| C3 | `battle.toml:2956` (BAT-MAG-044), `:1817` (BAT-END-003) | "a stack created in a dead created stack's slot"; "a slot a later created stack reused" | wording only: a killed one's. |
| C3 | `docs/rules/quirks-ledger.md:316` (BAT-QUIRK-11), `:328` (BAT-QUIRK-12) | "the side's first dead summoned stack"; "empty or holds a dead summoned stack" | with no creatures; the new tests are not listed as regression tests. |
| C3 | `docs/formats/installation.md:332`, `:1286`, `:1765`; `docs/adr/0014-battle-magic.md:69` (a shared path) | "A dead clone's or summoned stack's record goes to the next stack"; "no dead summoned one"; "in a dead created stack's slot" | as above. |
| C3 | `crates/h3-battle/src/state.rs:1073-1074` (doc of the public `stacks()`) | "a new created stack takes a dead created stack's" | a killed one's. |

Confidence: high (each clause grepped at `bc2d951`). No run decides it.

## Doubts

### Q1 (low). A6: the round start goes on after `BattleEnded`

- Where: `crates/h3-battle/src/turn.rs:98-99`: `check_end` runs before `obstacles::count_down`.
- Scenario: profile `t:clone_on_original`; the attacker's only stack turns into its own clone in
  round 1 (three rounds, gone at the start of round 4); its hero casts Force Field (`wall`) in
  round 2, which also ends at the start of round 4. Events of that round start: `RoundStarted`,
  `Vanished`, `BattleEnded`, `ObstacleRemoved`.
- Intended: the list says only "the end of the battle is checked there". No doc promises that
  `BattleEnded` is the last event (`event.rs:466-470`), and I found no consumer that relies on it
  (`h3-protocol` maps event by event), but where a strike or a shot ends the battle it is the
  last event of its command, and tests say so (`tests/rules.rs:915-918`, `tests/war.rs:256-259`).
  So this is a tidiness doubt; moving line 98 below line 99 would remove it. The fix's test pins
  `BattleEnded` as the last event only without an obstacle.
- Also untested: a draw at a round start (both sides' last stacks are turned originals that
  vanish together); `check_end` handles it by reading (`combat.rs:915-920`).
- Test to run: `tests/created.rs:346` with the hero also holding `wall`, one cast of it in round 2
  on two free hexes, then `assert!(matches!(events.last().unwrap().kind,
  EventKind::BattleEnded { .. }))`. I expect it to fail today.

### Q2 (low). B7: without quirk BAT-QUIRK-5 a created stack is still a target of Sacrifice

- Where: `crates/h3-battle/src/raise.rs:97-101` (`own: None`), so `eligible` (`raise.rs:124-139`)
  is the whole test and has no "not created" clause; `sacrifice_admits` (`raise.rs:165-173`) runs
  only under the quirk.
- Intended: the list asks for both tests "under the quirk", and BAT-MAG-036 says the same
  ("Without the quirk the target is tested by Sacrifice's own conditions and level alone"), so
  code and rule agree. But the original's own test of Sacrifice, as B7 reads it, refuses a
  summoned stack; the arm without the quirk is "Sacrifice's own test" without that part. Worker b
  left this open in its notes.
- Reach: none in the harvested profiles (`faithful` has the quirk; in `strict-fixed` no created
  stack can pass: elementals fail `requires = ["alive"]`, a clone never has losses while it
  lives). It needs BAT-QUIRK-7 without BAT-QUIRK-5 (the test profile `t:clone_on_original`) or a
  pack that summons living creatures.
- Test to run: profile `t:clone_on_original`; the attacker has the crowded ogres of `walled` and
  a second army stack as the victim; `copy` on the ogres (they turn, trait SUMMONED), one ogre
  killed; `cast_targets` of `offering` lists stack 0 today. Whether it should is a decision.

### Q3 (low, packs only). B9: the return of a walker that strikes and returns

No harvested creature is one (the Harpies fly), and the tests cover flyers only. For a walker
`travel_through` takes `movement::path` from where it struck (`obstacles.rs:802-803`), and three
corners are untested:
- `path` is empty when the start is out of the stack's reach now (`movement.rs:192-209`), for
  example after a retaliation's spell lowered its speed: the loop does not run and the stack is
  put back with one `Moved`, entering nothing (`obstacles.rs:842-844`), unlike a flyer.
- The path is computed once. A fire or mine on it calls `rebirths` (`obstacles.rs:661`, `:706`),
  which can bring back a stack the strike killed on a hex the rest of the path crosses.
- The route goes by the pits the stack's controller sees and may differ from the way there.
Test to run: a synthetic walker with `strike_and_return`, speed 3, three hexes from its target, a
Fire Wall on its start hex and a retaliation that slows it: I expect no burn on its return today.

### Q4 (a decision, not a defect). C1 counts hypnotized stacks that the mass version passes over

`magic.rs:1688-1691` uses `in_reach`, not `reachable`: with the enemy's only stack hypnotized by
the caster, mass Slow is legal, paid for and reaches nobody (pinned by `tests/control.rs:734`).
The list says both "hypnotized stacks count there" and "the legality test must count exactly the
stacks the resolution would reach"; the worker followed the first and says so (`fix-notes-c.md`,
Decision 1). The other reading is one word. The catalog says neither (D2).

## Notes

- A10: `Stack::may_attack` is a new public method of `h3-battle` and the public
  `Stack::is_hostile_to` changed its meaning (no user outside the crate, by grep). Batch 2c row
  H1's last line (Blue cannot attack its own hypnotized stack), the one observation behind the
  first point, is not replayed on the harvested packs: `crates/h3-pack/tests/
  harvested_control_and_field.rs:286` covers Hypnotize's limit and Expert Cure only. The rule
  itself is tested on the synthetic pack (`tests/control.rs:528`, `:589`, `:221`).
- B6: "saturates" is loose in one place that the fix did not touch: `Specialist::bonus` clamps to
  `i64::MAX` (`magic.rs:1856`), so a value near 2^62 with a huge specialist rate comes to less
  than `u64::MAX`. No panic, and far beyond any game value.
- C1: a share above zero that rounds to a chance of 0 % (resistance between 99 and 100 %) makes
  the spell castable while every stack resists, as for an aimed spell (`magic.rs:2001` against
  `magic.rs:2061-2078`). Worker c notes it; it is the original's "factor above zero" as read.
- C3: a side that keeps cloning now runs out of records for good (tested,
  `tests/created.rs:820`). This rests on the one code reading, as the list says.

## What I could not judge

- Whether the original behaves as the lists say (not re-read, as asked): in particular 0x47A100's
  record test (C3), the book's test at 0x59ED5A (C1), the draw's corpse test (C2), 0x442880 and
  0x476490 (A10), the stores of 0x4E5760 (B6).
- Whether the tests pass: nothing was run. The numbers of B5 and B6 are checked by my own model
  only; property tests, golden replays, coverage and the benchmark gate are not judged.
- The clients and the server: whether one of them assumes that a dead stack has no effects (B2
  keeps the permanent ones on it, without an event) or that `BattleEnded` is the last event (Q1).
  I found no such code in `h3-protocol`; I did not read `clients/`.
- Lists P and F are not part of this check.
