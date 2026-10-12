# List P: the property tests play what E04b added (sec F-4, the hygiene gaps, r4a F-3's invariant)

Worker name: `p`. Worktree: `/home/claude/wt-b`. Branch: `battle/e04b-t11b-p`.

The security reviewer found that the invariant and hygiene properties that `docs/06-security.md`,
ADR-0014 and the epic's exit criterion 4 rest on never play the cases where E04b made them matter.
You widen the generators and restate the assertions; you change no engine code.

## Read first

- `<SCRATCH>/review/security.md`: the table "What is hidden from whom", finding **F-sec-4**, and
  the section **"Gaps in the hygiene properties"** (all of it).
- `<SCRATCH>/review/robustness.md`: "What the existing tests exercise of E04b (and what they do
  not)" and "Not exercised".
- `docs/05-testing.md` (how property tests are seeded, their case counts, regression files) and
  `crates/h3-battle/src/tests/properties.rs` with its helpers in `tests/support.rs`.
- `<SCRATCH>/review/fix-notes-a.md`, items A1, A6 and A10 (what changed for hidden pits, the
  battle's end at a round start, and who is an enemy of a hypnotized or berserk stack), and
  `fix-notes-b.md`, item B2 (the invariant on a dead stack's effects) and B6 (the bounded test it
  widened).

## What to do, in this order

### P1. Controllers that differ from sides

`make_setup`'s spell book has no `t:berserk`, `t:hypnotize` or `t:blind`, so in every generated
battle a stack's controller is its side. Add the three spells (with masteries that let them land)
to the heroes the generators give books to, in `every_step_preserves_the_invariants`, the
legal-and-apply agreement property and the observation hygiene properties. Restate every assertion
that silently assumes "controller == side" (who may command, whose turn it is, which side's private
part an observation shows) by the trusted actor and the controller, as the rules say.

### P2. The heroes' hidden magic in force

No generated hero has `NoCasting`, `SpellLevelLimit`, the spell immunities, `NegateImmunities`, an
opening spell or Eagle Eye. Give the generators these effects (each present in a share of the
cases), so that the hygiene properties check that none of them reaches the other side's
observation or its events except through what the rules make public (BAT-Q26: the other side
infers them only from casts its own commands may not make).

### P3. Hidden obstacles with everything that walks, casts or aims

`hidden_obstacles_do_not_show` never runs the profile `t:pits` (quirk BAT-QUIRK-2) and its
`arb_command` has no `CreatureCast`. Add both. **One known exception:** a berserk stack's
automatic turn follows its own side's hidden pits and so can show them; the original does the same
by the reviewers' reading, and what the engine should do is an open question for the user
(BAT-Q31; findings F-sec-1 part a and r3 F-11). Keep Berserk out of this one property with a
comment that names BAT-Q31; keep it in all the others.

### P4. The battle's end as an invariant (r4a F-3)

The invariant "while the battle is not over, each side has a living stack that is no war machine"
is not asserted, because the invariant property runs a profile without quirk BAT-QUIRK-7. Add the
invariant, and run the invariant property in a profile that has BAT-QUIRK-7 on as well (A6's note
says where this stopped).

### P5. The two bounded robustness tests

`extreme_casting_heroes_play_without_failures` (h3-battle) and
`random_legal_sod_heroes_cast_and_play` (`crates/h3-pack/tests/harvested_heroes.rs`) were bounded
because of defects the first round fixed (F-robust-2, F-robust-3; A2 and B6 already widened them
in part). Read both and their doc comments: take out every remaining bound whose reason is fixed,
and correct the comments.

## Rules for this list

- **No engine change.** If a widened property fails, that is a new finding. Shrink it, write it
  into your notes (the property, the smallest case, what the rule says, what the engine does, your
  judgement: a defect, a wrong assertion, or a gap in a rule's text), and then commit the widening
  **without** the part that fails (no `#[ignore]`, no saved failing case in a regression file, no
  weakened assertion that hides it): say exactly what you left out and why. If the failure is in an
  assertion you restated wrongly, fix the assertion.
- Run each changed property many times before you trust it (the properties are randomly seeded):
  at least 20 runs of each, and one long run with the case count raised tenfold through the
  environment variable `docs/05-testing.md` names. Give the counts in your notes.
- Keep the suite's time: `cargo test -p h3-battle` takes about 90 seconds on this machine today;
  it must stay below 120. Measure before and after and give both numbers.
- Coverage is gated at 100 % for `h3-battle`: property tests may not be the only tests that reach
  a branch, and you add no engine code, so nothing changes there; but do not delete a fixed test.
- One commit per item P1 to P5.

## Not yours

Rule tests outside `tests/properties.rs` and its generators are worker `c`'s (it adds tests to
`control.rs`, `casters.rs`, `opening.rs`, `raise.rs`, `strike.rs` and others, and fixes three
defects in `magic.rs`, `obstacles.rs`, `created.rs`). Everything under `docs/` is workers `d` and
`q`'s.
