# Fix list B: effects of dead stacks, raises, and the numbers

Worker name: `b`. Worktree: `/home/claude/wt-b`. Branch: `battle/e04b-t11b-b`.
Reports are in `<SCRATCH>/review/`; "r2a" = `r2a-timed-effects.md`, "r4b" =
`r4b-strike-magic.md`, "robust" = `robustness.md`, and so on.

Do them in this order.

## B1. An aged Phoenix is reborn whole (r4b F-2, r5b F-1)

`raise::rebirths` reads `max_hp` before `effects::remove_dead` ends Age; the original's death ends
Age first. Read the health after the dead stack's effects are removed. Test
`an_aged_phoenix_comes_back_whole` from r5b F-1.

## B2. A dead stack keeps its permanent effects (r2a F-3, r4b F-1, r5b F-2)

`effects::remove_dead` drops every entry of a dead stack. In the original the death ends the
timed spells but not Disrupting Ray (the end routine returns at once for spell 47), and Acid
Breath lowers the defense directly; a raise or rebirth never restores the defense. Intended: a dead
stack keeps its entries without a duration (`rounds == None`: Disrupting Ray, Acid Breath), so a
stack that is raised or reborn comes back with them; a record that is **reused for a newly created
stack** starts with no effects at all (the original sets a new record up from scratch). Check every
caller of `remove_dead` and every place that assumes a dead stack has no effects (invariants in
`tests/properties.rs`, the observation, `created::add`). Tests from the three sketches (the
defense of a raised stack after Disrupting Ray and after Acid Breath; a reborn Phoenix; a reused
record).

## B3. Poison is reported when Age ends at the same round start (r4b F-8 part b)

`effects::count_down` compares the hit points after Poison's tenth with a value taken before the
stack was recomputed. Recompute first, as the original does. Test: a stack with Poison and Age
whose Age ends at a round start gets its `Poisoned` event.

## B4. Morale and luck are clamped once, at the roll (r2a F-1)

`morale.rs` about lines 314–318 clamps the base to ±3 and then adds Mirth, Sorrow, Fortune and
Misfortune. The original's stack routines ask the base routines not to clamp (0x43DF6D, 0x43E038;
the callees clamp only when their last argument is set: 0x44B380, 0x44B0B0; checked again by the
coordinator) and clamp only where the value is used (the good-morale roll 0x4649AA, bad morale,
the luck rolls); VCMI clamps the total once. Intended: the stack's morale and luck are the
unclamped sum of the base and the spells, then the minimums and caps (Minotaurs, Halflings, Spirit
of Oppression, Hourglass) as today, and the ±3 clamp applies where the value is used. Find out
what the observation and the events show today (clamped or not) and keep what they show within
±3 (the original's stack window shows the clamped value). Scenarios: base +5 with Expert Sorrow
(−2) rolls at +3 (the engine: +1); base luck +5 with Expert Misfortune: +3; base −5 with Expert
Mirth: −3. `tests/timed.rs` about line 456 pins the wrong number (2; it becomes 3).

## B5. The Behemoths' defense reduction under quirk Q-0005 (r2b F-1)

`damage::reduced_defense(…, staged = true)` truncates the share taken. The original (0x4425C8–
0x4425F5, checked again by the coordinator) computes the defense left in single precision and
truncates that toward zero: `trunc(f32(D) − f32(D) × f32(p))`, with the single-precision constants
0.4 and 0.8. Values: (17, 40 %) → 10, (17, 80 %) → 3, (7, 40 %) → 4, (7, 80 %) → 1, (10, 40 %) → 5,
(5, 80 %) → 0, (1, 40 %) → 0, (−3, 40 %) → −1. The variant without the quirk (exact, Q19) stays.
Change the tests in `tests/abilities.rs` (about lines 471–481) and add the case of quirks Q-0009
and Q-0005 together (Frenzy at Basic on a stack with defense 10 against Behemoths adds 5).

## B6. Spell damage under quirk Q-0005: the original's stores, and no overflow (r1 F-2, F-5; robust F-robust-3)

Two things in `magic::spell_damage` (about lines 1700–1780):

- **Overflow.** The products at lines 1710–1711 overflow `u128` for extreme but sound input
  (panic). The damage must saturate for every `u32` value and every rate the seam can carry
  (`Rate` with any basis points and any `single` bit pattern), also through Fire Shield's
  reflection, Fire Wall and mines. Tests: the five of the robustness patch for F-robust-3.
- **The stores.** With the quirk the original keeps the value as a single-precision number
  through stage 1 (0x4E5760): the value is stored as `f32`; with a school bonus the product is
  stored as `f32`; `(1 + every-spell bonus) × v` is stored as `f32`; only then it is truncated
  (the specialist's bonus is added to the untruncated value, the sum stored as `f32` and
  truncated: read r1 F-2 and `docs/formats/installation.md` §7.8, step 1). A store rounds to the
  nearest 24-bit value, so 57.99999982 becomes 58. The engine floors the exact product. Follow
  the stores with `h3_kernel::arith::X87`. Test from r1 F-2 (58, not 57); the 244 at
  `tests/magic.rs` about line 524 becomes 245. Check batch 2a's numbers still hold:
  `cargo test -p h3-pack --test harvested_spell_damage` (119, 115, 17, 230; 173, 25).

Without the quirk (exact rates) nothing changes except the saturation.

## B7. Sacrifice's raised stack passes Sacrifice's own test too (r4a F-1)

With quirk BAT-QUIRK-5 on, `raise::test_for` tests the raised stack as Resurrection would and
nothing else. The original asks Resurrection's test to find the stack and then Sacrifice's own
(the stack-to-raise variant: alive creature, not a summoned stack, fewer creatures than at the
start, the general immunities, so fire immunity, and the spell's level). Intended: under the
quirk both tests apply. Test from r4a F-1's sketch.

## B8. No Anti-Magic on a clone, whoever casts (r2b F-3)

The clone test of a spell that grants immunity sits only in the hero's aim (`stack_target`,
`grants_immunity`). In the original it is in the shared effect test (case 34 of 0x5A8090, checked
again by the coordinator), so a Master Genie never draws Anti-Magic for a clone. Move the test
into the shared stack test. Test from r2b F-3's sketch.

## B9. The Harpies' return lands on its hexes (r4b F-3, r3 F-6)

`combat::melee` sets the returning stack's hex directly. The original's return is a flight that
enters the hexes it lands on (Fire Wall, then Land Mine), and VCMI returns with `moveStack`.
Intended: the return goes through the same landing as a flyer's move, with whatever follows from
it (damage, a death, the battle's end, a rebirth). Test from r4b F-3's sketch.

## B10. A clone is not "raised" (r4a F-8)

`created.rs` about line 236: `StackTraits::with(RAISED, false)` clears nothing. Clear the trait
(without touching the others). Test: clone the Pit Lords' Demons and assert the clone has no
`RAISED` trait.

## Not yours

Hidden pits, Remove Obstacle, Slayer's addition, pack validation, the round start's end check,
the tactics phase, the sim's `step`, the bolt of 0, relations to hypnotized and berserk stacks and
Expert Dispel are worker `a`'s. The Defend bonus, a dead stack's round flags, the surrender price,
the Faerie Dragon's spells and everything else in the reports wait for the user.
