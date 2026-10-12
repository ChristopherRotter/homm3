# List C: three fixes by the code reading, two stale comments, and the rule tests the review asks for

Worker name: `c`. Worktree: `/home/claude/wt-a`. Branch: `battle/e04b-t11b-c`.
Reports are in `<SCRATCH>/review/`; "r1" = `r1-casting-core.md`, "r3" = `r3-control-battlefield.md`,
"r4a" = `r4a-life-created.md`, and so on. Read each finding's section before its item (it has the
evidence, the scenario and usually a test sketch). Do the items in this order.

## Part 1: fixes

For these three the code reading stands alone (the wiki and VCMI are silent), which by
`docs/04-rules-and-sources.md` §2 ("Consensus before trust") carries the rule; each stays an
experiment candidate (someone else records that).

### C1. A spell without a target is castable only when it would affect some stack (r1 F-3)

`magic::no_target` asks only whether a stack is in the spell's reach, and leaves out hypnotized
stacks. The original refuses the cast, before mana or the round's cast is spent, unless some stack
would be affected with a chance above zero (0x59ED5A–0x59EE03 with the factor of 0x5A39C0;
hypnotized stacks count there). Intended: a mass or battle-wide spell is legal only while at least
one stack in its reach accepts it (the same stack test the spell's resolution uses, with the
casting side; a stack that can only resist by a roll counts, one that cannot be affected at all
does not); `castable_spells`, `legal_commands` and `apply` agree (the property "legal casts and
apply agree" must hold). Check what the first round's A11 did to `skips_hypnotized`: the legality
test must count exactly the stacks the resolution would reach. Tests from the sketch (an enemy
whose every stack is immune: refused, nothing spent; one stack that accepts: legal; the hypnotized
case).

### C2. Quicksand's and Land Mine's draws avoid only living stacks and corpses (r3 F-8)

`obstacles.rs` about lines 388–390 skips every stack's footprint. The original's draw skips a hex
with a living stack or a corpse (0x5A03E3–0x5A03F3, 0x5A05FB–0x5A060D); a dead stack that left no
corpse (trait `NO_CORPSE`) does not keep its hexes free of pits and mines. Test from the sketch.

### C3. An expired created stack's record is not reused (r4a F-4)

`created::add` reuses any dead created stack. The original (0x47A100, 0x47A131–0x47A14A) reuses
only an empty record or a dead summoned one whose count is 0; a created stack whose rounds ran out
keeps its count, so its record is never taken again (`docs/formats/installation.md` §7.19 says so
already). Intended: `add` passes over a record that vanished at the end of its rounds. Read what
the first round's B2 did in `created::add` (`effects::forget`). Check the consequences the finding
names (the stack limit, a later created stack's id, the outcome's record) and test them; if a
consequence needs a state field or changes a wire type, stop this fix and report.

## Part 2: two stale comments (r6 F-8)

`crates/h3-content/src/ability.rs` about lines 370–372 and `crates/h3-battle/src/casters.rs` about
line 381 still say the Faerie Dragon ignores the Orb of Inhibition. Batch 2e settled the opposite
for the mouse path (the engine refuses the cast). Reword both to what the code does: the ability
exists for packs and no harvested creature has it. Do not write that the question is closed for
every path (the F-key path is an open question, BAT-Q32).

## Part 3: tests of behaviour that is right today

One commit per finding (or per test file where that is cleaner). Put each test next to the
existing tests of its rule. Harvested-pack tests go to `crates/h3-pack/tests/` beside
`harvested_spell_damage.rs` and `harvested_heroes.rs` (read one first for the conventions and for
how the batches' setups are built).

- **T1 (r1 F-10):** a cast with mana exactly equal to the cost (legal, leaves 0); batch 2a's R1 and
  R2 (Chain Lightning's geometry and tie order) and R6's cast in the Ballista's turn, on the
  harvested packs with the batch's numbers (`experiments/results/` and `docs/epics/E04b-log.md`
  have them).
- **T2 (r2b F-6, items 2 to 4):** Frenzy's count for a stack that waits, in play; Magic Mirror's
  new target among war machines; batch 2b R3's 60 with its numbers. (Item 1 came with B5. Item 5,
  the striker killed by a Fire Shield's reflection, waits for a decision: no test.)
- **T3 (r3 F-10):** batch 2c's rows F1 and H1 with their numbers on the harvested packs; the tie
  rule of BAT-MOV-003 that row F2 supports; Hypnotize's limit at exactly the limit (a stack of
  exactly 60 in `tests/control.rs`, as the sketch says).
- **T4 (r4a F-10):** BAT-MAG-035's boundary at the health; BAT-MAG-039's row draw and its "no
  place" case; the two place skips of BAT-MAG-038 the report names; batch 2d's rows with their
  numbers (a base-pack test for RA2). (A created stack's end through real commands came with A6.)
- **T5 (r4b F-6):** quirk Q-0005's Poison and Age variants in a battle (the sketch: a creature of 9
  hit points, poisoned and aged: `max_hp` 5, and 4 with `t:single`); a shot at a petrified stack is
  halved; Bind ends after its rounds.
- **T6 (r5a F-7):** the Archangel's raise at a health equal to 100 × creatures (a target) and one
  above (none); a resisting stack in a creature's area spell; a creature's chain whose first stack
  does not roll while a later jump does.
- **T7 (r6 F-5):** Eagle Eye's chance under quirk Q-0005 on the harvested packs, as the sketch
  (Orrin with the Emblem at Advanced: 64 in `base:faithful`, 65 in `base:strict-fixed`; the Basic
  specialist at level 2: 43 and 44). The reviewer computed these and did not run them: if the
  engine gives other numbers, find out which side is wrong before you write the test; report
  either way.
- **T8 (r6 F-6):** the opening spells' timing: after a bad-morale freeze of the side's first stack
  the cast comes at the first stack that takes its turn (`BadMorale`, then `TurnStarted`, then
  `OpeningSpell`); the opening cast comes before the Enchanters' periodic cast.
- **T9 (r6 F-7):** a strike's spell and the negation of immunities: the sketch's `blinder` at the
  mind-immune `statue`, with and without `NegateImmunities` on the striker's side's hero.

## Not yours

The property tests (`tests/properties.rs` and the generators) are worker `p`'s: do not edit them.
Rule texts, the ledger, `installation.md` and every other file under `docs/` are workers `d` and
`q`'s: write your wording proposals for C1 to C3 into your notes (rule id, old clause, new
clause). BAT-QUIRK-2, BAT-QUIRK-9, the berserk stack's hidden pits, the Defend bonus, a dead
stack's round state, the wait phase's order and everything else in the reports wait for the user.
