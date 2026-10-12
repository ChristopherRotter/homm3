# Fix notes of worker `p` (E04b T11b, second round, list P)

Worktree `/home/claude/wt-b`, branch `battle/e04b-t11b-p`, base `64fe5cf`. Logs: `<SCRATCH>/wp/`.
No engine code is changed by this list: only `crates/h3-battle/src/tests/properties.rs` (P1 to P5)
and `crates/h3-pack/tests/harvested_heroes.rs` (P5).

Status (kept current; a restart continues from here):

- P1: code written; 20 runs of the invariant property under way (`wp/p1-inv-20.log`); not committed.
- P2 to P5: not started.

## Before anything (baseline at `64fe5cf`)

- `cargo test -p h3-battle`: 479 tests, 77.7 s (`wp/base-test-1.log`; the binary was built before).
- Each property alone, one test thread (`wp/base-props-time.log`): `every_step_preserves_the_invariants`
  95.5 s in that run (another worker was building; 77 s inside the whole suite), every other
  property 0.2 to 3.9 s. The suite's wall time is this one property's: two test threads, and the
  rest together takes about 20 s on the other.
- Where its time went (a throwaway harness over 96 generated games, `wp/p1-stats-probe-1.log`,
  with Berserk already in the book): 22.5 s for the listed legal commands (370 000 clone-and-apply
  probes), 11.3 s each for `nova`, `ring` and (new) `berserk` (a hex spell has 165 targets, each
  applied to a clone at every step), 6.4 s for Teleport (669 000 probes), 2 s each for the two
  wall spells, under 0.5 s for each other spell.
- `PROPTEST_CASES` does override the `cases` of `proptest_config` (proptest 1.11's `proptest!`
  passes the given config through `contextualize_config`), so the long runs below use it.
  `docs/05-testing.md` does not name the variable; `docs/lanes/server/HANDOFF.md` does.
- The ten saved seeds of `proptest-regressions/tests/properties.txt` no longer reproduce the
  cases they were saved for once the strategies change (a `cc` line is the seed of the whole
  strategy): they still run, as ten more random cases per property. The file is unchanged.

## P1. Controllers that differ from sides — in progress (not committed yet)

What is written (all in `tests/properties.rs`; the source as built for the runs is saved in
`wp/snap/properties-p1c.rs`, with a throwaway block at its end that is not committed):

- Generators. The heroes' book has `t:berserk`, `t:hypnotize`, `t:blind` (38 spells). A new step
  kind `Control(n)` (weight 1 of 11; `Legal` 6, `Arbitrary` 2, `Cast` 2) casts the n-th castable
  one of the three at its n-th target, Berserk at a hex a living stack stands on. Power and
  masteries from bits of the setup word rotated by 29: the attacker's hero has power 10 instead
  of 2 one time in four, the defender's 6 instead of 1, and the defender's hero has air and fire
  masteries (it had none). Seeds near 0 keep the old values.
- `arb_command` (sec F-4's fourth point): `Shoot`, any spell of the ruleset as a cast (it named
  only the first 12 of 61), stack ids up to 43 and sometimes any byte, a stack or a hex as a
  cast's second target.
- What a run of 192 generated games plays (`wp/p1-stats-1.log`, a throwaway harness with a fixed
  seed): 832 hero casts, of them 146 Berserk (101 stacks made berserk), 145 Blind, 40 Hypnotize;
  153 accepted commands for a hypnotized stack (9 strikes, 10 hero casts in its turn); 100
  automatic turns of berserk stacks with 93 strikes and 59 moves; 16 casts that made the active
  stack berserk. So one run of a 96-case property has about 75 commands of a controller that is
  not the stack's side and about 50 automatic berserk turns.
- Assertions restated in `every_step_preserves_the_invariants`:
  - exactly the side that controls the active stack has legal commands (was: "the opponent has
    none or it is its turn");
  - a `SpellCast` event appears exactly for an accepted `Cast`, with the actor's side;
  - a cast keeps the turn unless it killed the active stack **or made it berserk** (BAT-MAG-025);
  - one walk over the command's events replaces "the mover's strikes hit the commanded target,
    every other strike is a retaliation at the mover or a war machine's": the acting stack is the
    mover until a `TurnStarted`; a stack moves and strikes on its own turn only; the commanded
    strikes hit the commanded target, which is never of the actor's side; a retaliation is by the
    last struck stack at the acting stack; a turn after the command is automatic, a war
    machine's (its target of the other side) or a berserk stack's (any other stack); `Teleported`
    only by the actor's cast at a stack of the actor's side;
  - good morale per `GoodMorale` event (was: the mover's): the stack takes the turn (the next
    event is its `TurnStarted`, not followed by its `Frightened`) and then, unless it is still
    berserk, it is the active stack at the command's end and no round started after;
  - "the actor cannot move enemy stacks" became "a stack's hex changes only with a `Moved` or
    `Teleported` event that names it" (the walk above says when those may occur);
  - an enemy stack's count and hit points change only with an event that names it: `Reflected`
    and `LifeDrained` added (a stack of the other side that strikes on the actor's command while
    hypnotized, or on its own while berserk);
  - new state invariants: `Stack::controller()` is the other side exactly while a spell with the
    Hypnotize change is in the effect list for the stack (read from the list, not from the
    stack); `side_to_act()` is the active stack's controller by that rule; the stack that awaits a
    command is not berserk.
- `observations_do_not_depend_on_hidden_state`: the drawn spell is shown exactly to the side
  that commands the active stack, and only that side has cast candidates.
- `hidden_obstacles_do_not_show`: `t:berserk` is taken out of both books, with a comment that
  names BAT-Q31.
- **A change of how the invariant property probes casts (to pay for Berserk's 165 targets).**
  Every listed cast (spell, target, second target) is checked with `validate` at every step (the
  crate's public entry for exactly this, the validation `apply` runs before it changes anything);
  it is applied to a clone as before unless it is aimed at a hex: of those (the area spells, the
  wall spells, Remove Obstacle, Teleport's places) every second hex is applied, alternating with
  the step. Before, every one was applied at every step. Legal commands and creature casts are all
  applied as before.

### BAT-Q31, confirmed by runs (not a new finding; the exception P3 names)

- The security reviewer's sketch for F-sec-1 part a, run as a throwaway test (`zz_berserk_pits`,
  not committed; output in these notes only): the defender's berserk `fixed` at (8,5), the
  attacker's at (4,5), the defender's own hidden pits at (6,5) and (5,5).
  `t:faithful`: with the pits it ends on hex 106 and strikes from there, without them on hex 90:
  the attacker, who sees no pit, is told another `Moved`. `t:pits` (BAT-QUIRK-2): with the pits it
  walks to hex 90, stops there and does not strike; without them it strikes: the attacker reads
  the pit's hex. Exactly as the reviewer traced by hand.
- The property itself with Berserk in the book: 30 runs of 96 cases passed; of 10 runs of 960
  cases one failed (`wp/p1-hid-berserk-10x-fail-9.log`, seed line
  `cc c42b19a96a87c7b2cc77144d9e5378b8d3d1fcc53a2d1281be4ae4206a13a62f`, reproducible only with
  the binary `wp/bin/p1a`). There the berserk stack of the other side makes the same move and
  strikes the same target in both worlds, but its strike's damage differs (4 against 1): the
  number of nearest candidates differs with its side's hidden pits, so the tie draw
  (`control.rs:84`) happens in one world only and every later roll moves. A third form of the
  same exception, besides the two the reviewers describe (another position, no strike).
  With Berserk in this property's book the suite would fail about once in a hundred runs.
