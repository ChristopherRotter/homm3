# Fix notes of worker `b` (E04b T11b, list B)

Worktree `/home/claude/wt-b`, branch `battle/e04b-t11b-b` (pushed after every commit), HEAD
`417f58e`. All ten fixes are done. B1 and B2 (and the commits of B3 to B5) are the first worker
`b`'s, before the sandbox restarted at 08:27 on 2026-10-12; the notes of B3 to B5, B6 to B10 and
everything from "Rule texts…" on are the second worker's (a fresh context). The parts: one
section per fix, the golden replays, the wording proposed for the docs, notes for the
coordinator, the closing checks.

## B1. An aged Phoenix is reborn whole — done, commit `abc6c04`

- Test first: `tests/rebirth.rs::an_aged_phoenix_comes_back_whole` failed on the unchanged code
  with `(count, max_hp, top_hp) = (1, 10, 10)` instead of `(2, 10, 10)`: the reason of the
  findings.
- Change: `raise::rebirths` reads `max_hp` after `effects::remove_dead`.
- Golden replay: `full-s3-a955` changes (see "Golden replays").

## B2. A dead stack keeps its permanent effects — done, commit `956b5db`

- Tests first (all four failed on the unchanged code for the findings' reason: the dead stack
  had lost its entry, the raised or reborn stack had its full defense):
  `tests/raise.rs::a_raised_stack_keeps_the_permanent_effects_it_died_with` (Expert Disrupting
  Ray and Haste, death, Resurrection: the ray's entry and −5 stay, Haste is gone; the entry is in
  the observation while the stack is dead),
  `tests/strike.rs::acid_breath_stays_on_a_stack_that_dies_and_is_raised`,
  `tests/rebirth.rs::a_reborn_phoenix_keeps_the_permanent_effects_it_died_with`,
  `tests/created.rs::a_reused_record_starts_without_the_dead_stacks_permanent_effects` (this one
  also fails when `created::add` does not clear the record: checked by taking the call out).
- Change: `effects::remove_dead` keeps the entries with `rounds == None` and recomputes only
  the stacks that lost an entry; new `effects::forget(state, id)` removes every entry of one
  stack; `created::add` calls it for the record it reuses and no longer calls `remove_dead`
  (in play no stack is dead with timed effects when a stack is created: every creation is the
  first thing its command does; the call only mattered to unit tests that kill by hand).
- Callers of `remove_dead` checked: `command::apply` (end of every command: unchanged call),
  `raise::rebirths` (unchanged call; the reborn stack is recomputed from what it kept by
  `revive`), `created::add` (replaced as above).
- Places that assumed "a dead stack has no effects":
  - `tests/properties.rs`, `every_step_preserves_the_invariants`: the invariant is now "an entry
    is on a living stack or has no duration"; I also widened "each stack's values are those of
    its entries" from living stacks to all stacks (it holds: 3 runs × 96 games before the commit).
  - The observation (`observe_as`: `effects: state.effects().to_vec()`), `h3-sim`'s flat
    encoding (per stack: number of effects and the first entries, dead stacks included) and
    `h3-protocol`'s mapping (`stack(..)` lists the effects of every stack) take the list as it
    is: a dead stack's permanent entry now shows there, and its `Stack.spells.defense` stays
    lowered. No field, encoding or version changed.
  - `magic::admits`, `accepts`, `skips_hypnotized`, `casters` (candidates), `after_strike`,
    `combat::specialist_terms`, `magic::reflection` read the list only for living stacks (or
    for the struck stack within its command, as before).
  - `effects::count_down` already skips entries of dead stacks; `round_retaliations` walks every
    entry as before (a permanent Counterstrike exists in no pack).
  - `raise::eligible` leaves Anti-Magic out for a dead stack ("a dead one's effects ended at
    its death"): unchanged, the comment now says "timed effects". A pack with a *permanent*
    Anti-Magic would keep it on the dead stack and this test would not see it; no pack has one.
  - `binds::materialize` binds only a living stack: a pack with a permanent Bind would bind the
    stack again when it is raised (the harvested Bind has 255 rounds and ends at the death).
- Comments updated: `effects::remove_dead`, `raise::raise`, `raise::revive`, `raise::rebirths`,
  `raise::eligible`, `binds::died`, `created::add`.
- Golden replays: six change (see "Golden replays").

## B3. Poison is reported when Age ends at the same round start — done, commit `67433be`

(Notes written by the second worker `b` from the commit, after the sandbox restarted.)

- Test: `tests/strike.rs::poison_is_reported_when_age_ends_at_the_same_round_start` (next to
  BAT-MAG-044's test): 10 hit points, Poison for three rounds (9) and Age in its last round (5);
  the round start ends Age (9 again), Poison takes its tenth (8) and `Poisoned { 1, 8 }` is
  emitted. On the old code the 8 was compared with the aged 5 and nothing was reported.
- Change: `effects::count_down` recomputes a stack whose effect ended *before* Poison's tenth is
  taken and compared (it recomputed only stacks without Poison, afterwards). The state does not
  change (the top creature loses the same hit points either way); only the event is new.
- Source in the commit: the round start 0x447120 counts down first (Age's end recomputes the hit
  points), Poison's tenth is compared with that value (0x44720B, 0x4472A2).
- Golden replays: none changes (recording `b3` is identical to `b2`).

## B4. Morale and luck are clamped once, at the roll — done, commit `6da9dab`

- Tests: `tests/timed.rs::morale_and_luck_beyond_the_range_absorb_the_spells_and_are_clamped_once`
  (new, next to BAT-MAG-011's test: base morale 4 + 1 with Expert Sorrow −2 → 3; base luck 5 with
  Expert Misfortune −2 → 3; base morale −6 + 1 with Expert Mirth +2 → −3; the observation shows
  3 and 3; a lucky strike and a good-morale turn that only +3 gives);
  `morale_and_luck_spells_join_the_sum_before_the_minimums` (renamed from
  `…_apply_between_the_clamp_and_the_minimums`; it pinned 2 for morale 4 with Sorrow −1, now 3).
- Change: `morale::materialize` sums the side's changes and the spells without a clamp
  (`evaluate(spells, mods.chain(minimum))`), applies the caps and the minimums as before, then
  clamps once: morale to `[−bad levels, good levels]`, luck to `[0, good levels]`.
- What the observation and the events show: `Stack::morale` and `Stack::luck` were the clamped
  values before and still are (the rolls, `observe_as` and the flat encoding read these fields;
  no event carries a morale or luck number). Nothing unclamped is stored or shown.
- Order kept from the old code: the creature's minimum, then the side's cap
  (`h3_kernel::modifier::evaluate`: raised to the largest `AtLeast`, lowered to the smallest
  `AtMost`), then the one clamp. The only difference to before is that the side's sum is no
  longer clamped to ±3 before the spells join it.
- Golden replays: none changes (recording `b4` is identical to `b3`).

## B5. The Behemoths' defense reduction under quirk Q-0005 — done, commit `59389b0`

- Tests: `tests/abilities.rs::defense_reduction_ignores_a_share_of_the_defense_rounded_down`
  (the pinned values became those of the list: (17, 40) → 10, (17, 80) → 3, (7, 40) → 4,
  (7, 80) → 1, (10, 40) → 5, (5, 80) → 0, (1, 40) → 0, (−3, 40) → −1; VCMI's
  `floor(D × p / 100) + 1`, at most D, as an independent vector for D = 1 to 300 at both
  percentages and both signs; a strike at a defending Tank in profile `t:single`: 24 leave 14,
  damage 18, was 17); `tests/rage.rs::frenzy_against_a_behemoth_counts_its_reduction_with_quirk_q0009`
  (new test profile `t:frenzy_single` in `tests/support.rs` with Q-0009 and Q-0005: 25, was 26).
- Change: `damage::reduced_defense(…, staged = true)` computes
  `trunc(f32(|D|) − f32(|D|) × f32(p / 100))` in the x87 emulation and gives it the defense's
  sign; the exact variant (Q19) is unchanged.
- Golden replays: none changes (recording `b5` is identical to `b4`; the synthetic pack has no
  creature that ignores a share of the defense).

## B6. Spell damage under quirk Q-0005: the stores, and no overflow — done, commits `be311a3` (battle), `55ae779` (pack tests)

- **The earlier worker's numbers, checked before use.** I re-read 0x4E5760 and 0x5A78E0
  (`scripts/exe.py`). The stores are as r1 F-2 and B6 say: the value is loaded and stored as a
  single; with an Orb the product with the single constant 1.5 is stored as a single; the
  Sorcery value (grown for a specialist, itself stored as a single) plus 1, times the value, is
  stored as a single; only with a target stack the value is truncated, the specialist routine's
  whole number is stored as a single, added to the *stored* (untruncated) product and stored
  again; the last store is truncated. `stage1.py` models exactly that. I recomputed every
  expected number a second, independent way (`<SCRATCH>/b-logs/w2/check1.py` to `check3.py`:
  IEEE doubles as the x87 at 53 bits with stores to singles, and my own exact-rational rounding):
  58 (old engine 57); 245 (old 244); 250; 972 against the exact rates' 973 (the old engine also
  had 972: this vector tells two stores from one store at the end, which would give 973); 9995
  against 9994 (the bonus joins the stored value; the sum is a tie that rounds to even). All
  agree. r1 F-2's other examples hold too (level 15 Advanced, power 3: 47 against 46; level 2
  Advanced, power 9: 111 against 110).
- **Tests first.** On the unchanged code: `a_grown_spell_bonus_…` 244 ≠ 245;
  `the_casters_products_are_stored_…` 57 ≠ 58; the panics at `magic.rs:1710:18` or `1711:27` of
  `huge_spell_damage_rates_do_not_overflow_the_spell_damage`,
  `spell_damage_rates_of_millions_of_percent_do_not_overflow`,
  `saturated_single_precision_rates_do_not_overflow_the_factor`,
  `the_spell_damage_saturates_for_every_value_and_rate` and
  `rage::a_fire_shield_reflects_a_huge_strike_without_overflow`; the control
  `huge_spell_damage_rates_cast_with_exact_rates` passed. The two harvested tests failed alike
  (57 ≠ 58 for Sandro; the panic for Implosion), and the widened property test panicked at
  `magic.rs:1711:27`.
- **Change** (`magic.rs`): the caster's stage is its own function `caster_stage`. With the quirk
  it follows the stores with `X87` (`round(SINGLE_BITS)` after the load and after each product;
  the bonus, one whole number, stored and added to the stored product; that sum stored and
  truncated); everything saturates. With exact rates the formula is the old one, and a product
  beyond `u128` gives `u64::MAX` (it would exceed `u64` after the division by 10^8).
- **A decision the coordinator should know: no hero, no stage.** The original runs 0x4E5760 only
  when the damage-modifier routine 0x5A78E0 is given a hero (so §7.11, §7.15, §7.17, §7.20: the
  Thunderbirds' bolt "without a hero"; shield, wall and mine with the owner's hero, if any). A
  store of the value to a single rounds whole numbers above 2^24, so applying the stage without
  a hero would have rounded a hero-less value above 16 777 216 where the original and the old
  engine keep it exact. `spell_damage` therefore takes the hero's bonuses as
  `Option<&SpellBonuses>`: `None` (the Thunderbirds' bolt, a stack's cast: `cast.by` is some,
  a shield, wall or mine of a side without a hero) leaves the value as it is. Call sites:
  `after_strike::thunderbolt`, `magic::reflection`, `magic::affect`, `magic::burn`. No behaviour
  changes below 2^24. (r1 F-12, a creature's area spell taking the side's hero's bonuses, is not
  mine and unchanged: a stack's cast passes `None`.)
- **Generalisations beyond the original's data** (the pack decides the rates): the Orb's factor
  is `1 + f32(rate)` computed at 53 bits (1.5 exactly for the harvested 50 %); the spell's own
  bonus (`damage_bp`, Ciele's and Luna's) and the specialist's are summed to the one whole number
  the original's routine returns; a specialist rule that lowers the value (`Total`) is subtracted
  from the stored product.
- **Saturation.** For every `u64` value and every `Rate` (any basis points, any `single` bit
  pattern: NaN, infinities, negative and subnormal values count as 0, as `X87::from_f32_bits` has
  it; a value of 2^24 or more is 2^24 − 2^-40, `hero_rate`'s fixed point) nothing panics; the
  largest value at the largest rates is `u64::MAX`. Not changed, and to be known: the target's
  spell damage reduction and the protection without the quirk compute
  `damage.saturating_mul(k) / 100`, so above `u64::MAX / 100` they level off at `u64::MAX / 100`
  (monotone, no panic; not in the findings).
- **Tests** (`tests/magic.rs` next to BAT-MAG-004's, `tests/rage.rs` next to BAT-MAG-018's):
  `the_casters_products_are_stored_as_singles_with_quirk_q0005`,
  `a_grown_spell_bonus_counts_with_its_single_precision_value_with_quirk_q0005` (244 → 245, and a
  rate whose own single value is not its basis points': 250 with the quirk, 245 without), the
  robustness patch's five (with the exact damage pinned where the rates are basis points:
  472 238 801 289 216 with the quirk, 472 238 847 092 877 exact, 281 495 612 686 336),
  `the_spell_damage_saturates_for_every_value_and_rate` (also: no hero keeps every value; a hero
  without bonuses rounds above 2^24 with the quirk, ties to even).
  `crates/h3-pack/tests/harvested_spell_damage.rs`:
  `a_sorcery_specialists_product_is_stored_as_a_single_before_it_is_rounded_down` (Sandro,
  Advanced Sorcery, Magic Arrow: 58, 47, 111 in `base:faithful` and `base:strict-fixed`; the
  hero is derived by `h3-heroes`, so the whole chain is pinned) and
  `implosion_with_huge_spell_damage_rates_does_not_overflow` (the robustness patch's harvested
  test, with the damage pinned in both profiles). Batch 2a's numbers hold (119, 115, 17, 230;
  173, 25).
- **The bounded property test is widened** (`tests/properties.rs`,
  `extreme_casting_heroes_play_without_failures`, commit `15ae951`'s "widen them once it
  saturates"): `extreme_caster` lost its `exact` switch, the Q-0005 half now has the extreme
  spell damage rates and armies of up to `u32::MAX`. The obstacle bound (F-robust-2, worker `a`)
  is untouched; I rewrote the doc paragraph that names both findings, so it may conflict
  textually with a change of `a`'s there (at 09:37 `a` had not touched this file). The test
  now counts 1 217 casts of the heroes and 373 of the stacks (its floor is 500 and 200).
- With the stores modelled, a rate's own single value (`Rate::single`) changed no bolt value in
  the earlier worker's sweeps (`stage1b.py`: 22.5 % as 0x3E666666 against 0x3E666667, values 1 to
  60 000; `stage1c.py`: every derived Sorcery specialist rate up to level 74 against the single
  of its basis points, values 1 to 5 000). The seam still carries it and the battle still reads
  it (the test with 0.25 against 2 250 bp).
- **A differential check outside the repository** (after the commit; a throwaway test printed
  the stage's result over a grid and was removed again: `<SCRATCH>/b-logs/w2/grid.log`,
  `grid_check.py`): 112 064 cases (values 1 to 400 and twelve larger ones up to 2^31 − 1; the
  every-spell rate from 13 basis-point values and four `single` patterns; a school rate of 0, 1,
  3 900 and 5 000 bp; the spell's own bonus 0, 33.33, 50 and 100 %), the engine's `X87` stage
  against IEEE doubles with stores to singles, and every 97th case against exact rationals:
  no mismatch. Against the old formula, of the 28 016 cases without an own bonus the 27 609
  with a result below 2^24 are equal (27 462) or one higher (147), never lower; of the 407 with
  a result of 2^24 or more 315 differ (a single cannot hold such values).
- Golden replays: none changes (recording `b6` is identical to `b5`).

## B7. Sacrifice's raised stack passes Sacrifice's own test too — done, commit `6c3ddec`

- Re-read before the fix (the list does not say "checked again"): the target test 0x5A39C0
  finds the stack (0x5A3950) and, if one is found, asks 0x5A8090 about the spell itself with its
  own fifth argument passed on; in 0x5A8090 spell 40 with that argument set skips the
  controller's test (0x5A82B9) and case 0x5A8523 asks: creature flag `0x10` (living), not flag
  `0x400000` (summoned), count below the start count; then the general tail (0x44A4D0 with
  spell 40). With the argument 0 it is the victim's test (not undead, creatures left). As r4a
  F-1 says. (The three callers that pass 1 for the raised stack I took from the finding.)
- Test first: `tests/raise.rs::with_bat_quirk_5_the_raised_stack_passes_sacrifices_own_test_too`
  failed on the unchanged code with `[1, 2]` instead of `[2]` (the fire-immune embers were a
  target of `offering`).
- Change (`raise.rs`): `RaiseTest` has a third field `own`, set only for a sacrifice under the
  quirk; `raisable` finds the stack by the test as before and then asks `sacrifice_admits` of
  the stack found (not `is_created()`, and the own spell passes `can_affect`: the spell level
  limit, conditions, immunities with the heroes' part, Anti-Magic on a living stack; this is
  the block `eligible` already had, now a function of its own).
- **The order matters for corpses** and is the original's: the own test is not part of what
  makes a corpse "on top". Dead Efreet above dead Pikemen on one hex: the finder returns the
  Efreet (they pass Resurrection's test), Sacrifice's own test refuses them, and the Pikemen
  below are not a target. The test pins this (and that without the quirk the own test alone
  picks the corpse: the serfs).
- **Not changed, to be decided by the coordinator:** without the quirk (`strict-fixed`) the
  raised stack's own test still has no "not created" clause (`test_for`'s `(true, None)` arm):
  a created stack of *living* creatures with losses is a target there. The original's own test
  refuses it in every case. With the harvested packs it cannot happen in `strict-fixed` (a clone
  has no losses, a dead created stack no corpse, elementals are not living, and turned originals
  need BAT-QUIRK-7, which is off there). One line (`own: Some(..)` in that arm, or the clause
  in `eligible`) if it is wanted; B7 asks for the quirk's side only.
- Golden replays: none changes (recording `b7` is identical to `b6`).

## B8. No Anti-Magic on a clone, whoever casts — done, commit `d5a7051`

- Checked at 0x5A83FF: Anti-Magic's own case of 0x5A8090 gives 0 for a stack with the clone flag.
- Test first: `tests/casters.rs::anti_magic_is_never_useful_on_a_clone` failed on the unchanged
  code at `!is_useful(.., "antimagic", <the clone>)`.
- Change (`magic.rs`): the clause `!(stack.has(CLONE) && grants_immunity(spell))` moved from
  `stack_target` (the hero's aim) into `affects_stack`, the shared stack test. Every path asks
  it: the hero's aim and mass versions (`in_reach`), a creature's named spell (`first_target`),
  a drawn spell and a helpful caster's draw (`affectable`), the candidates of the observation
  (`casters::candidates` through `useful_spells`), strike spells, the chain, a mirrored target,
  obstacles. The hero's aim is unchanged (its test in `tests/created.rs` still passes).
- No doc is wrong here: `installation.md` §7.13 already says the per-spell case refuses
  Anti-Magic on a clone; BAT-MAG-021's "No clone may be aimed at" could say "whoever casts".
- Golden replays: none changes (recording `b8` is identical to `b7`).

## B9. The Harpies' return lands on its hexes — done, commit `9076b08`

- Re-read before the fix: the move-and-attack routine 0x478900 (the return's conditions at
  0x4789F8: the stack moved, is creature 72 or 73, is not dead, and Blind, Stone Gaze and
  Paralyze are not active; **no test of the battle's end**; then the move routine 0x445D10 with
  the start hex) and the move handler 0x445950 (the destination must be a hex where the stack
  fits; the entered-hex marks are cleared and the current hexes marked; then a flight for a
  flyer, the teleporting move for creatures 54 and 55, a walk for the others). As the two
  findings say.
- Tests first: `tests/obstacles.rs::a_stack_that_strikes_and_returns_burns_where_it_lands` and
  `tests/mines.rs::a_stack_that_strikes_and_returns_sets_off_the_mine_it_lands_on` failed on the
  unchanged code with no burn and no blast (`[]` instead of `[(0, Hex(91), 40, 4)]`).
- Change: `combat::melee` calls the new `obstacles::travel_back` instead of setting the hex. It
  is `travel`'s move (the binder's stacks released, the hexes entered: a flyer's landing, a
  walker's path with its pits) with one difference: the battle's end is not checked inside it.
  `enter`, `explode` and `travel_through` got a parameter `ends` for that (`true` from `travel`
  and `stand`, which behave exactly as before).
- **Why the end check is deferred** (a decision for the coordinator to confirm). `check_end`
  emits `BattleEnded` every time it finds the battle over, and `melee` checks at its end. Inside
  the return the strike may already have killed the other side's last stack (not checked yet):
  a check after the burn would report an end that a mine on the same hex, or the final check,
  reports again, and with a result that may be stale (the Harpies die of the mine afterwards).
  The original checks nothing before the return, so the engine now checks once, after the whole
  action, on the final state. Consequences, each pinned by the test:
  - Harpies that die where they land as their side's last stack end the battle (one event);
  - Harpies whose strike killed the last enemy stack **still land and burn** (casualties in the
    outcome), and win;
  - if that burn kills them too, both sides are without stacks: `Draw` (BAT-END-002).
  This differs from the engine's rule for a stack that attacks *from where it stands*
  (BAT-MAG-028, `installation.md` §7.15: "no such check was found" in the original, "the engine
  skips the burn once the battle is over there too"): there the engine skips, here it does not.
  If the same choice is wanted for the return, it is the other order in `melee` (end check
  first, then a plain move when the battle is over); say so and I would not have guessed it from
  B9's "with whatever follows from it".
- **Follows from "a move like any other"** and is new behaviour for packs only (no creature of
  the original has both abilities): a binder that strikes and returns releases the stacks it
  binds (BAT-MAG-045's "when its last binder moves"; the original's handler releases at every
  move). Test `tests/strike.rs::a_binder_that_strikes_and_returns_releases_the_stack_it_bound`
  (the trait set by hand). A walker with the ability would walk back along `movement::path`
  (burning on each hex, stopping in a pit); if the start is out of its reach the stack is put
  back directly, as before.
- The rebirth: `tests/rebirth.rs::a_damage_spell_a_shot_and_a_mine_bring_the_rebirth_too` now
  also has firebirds (the trait set by hand) that walk back onto a mine, die and are reborn
  there once, having acted.
- Golden replays: none changes (recording `b9` is identical to `b8`).

## B10. A clone is not "raised" — done, commit `417f58e`

- Test first: `tests/raisers.rs::a_clone_of_raised_imps_is_not_raised_from_a_corpse` failed on
  the unchanged code at `!clone.has(StackTraits::RAISED)`.
- Change: `StackTraits::without` (new, `pub(crate)`: the public `with(_, false)` adds nothing by
  its contract and stays as it is; so does `Status::with`) and `created::clone` uses it. The
  test also pins that the clone's other traits are its original's plus SUMMONED and CLONE.
- Effect outside the battle crate: the sim's observation and the protocol's view no longer show
  such a clone as raised from a corpse; the state hash of a state with a clone of raised
  Demons changes (no golden replay has one).
- Golden replays: none changes (recording `b10` is identical to `b9`).

## Golden replays (not regenerated)

`cargo test -p h3-cli` fails in `golden_replays_reproduce_their_hashes`. Checked per fix by
recording the eight scenarios with the same seeds into the scratchpad
(`<SCRATCH>/b-logs/rec.sh`, `cmp.py`; nothing written into the repository):

| after | replay | what changed | why |
|---|---|---|---|
| B1 | `full-s3-a955` | command 133 (`A heal 1`; `h3 verify` counts from 0 and says 132) is rejected ("the target is not a hurt allied stack"); a new recording has 472 commands instead of 458, same winner | at command 127 the attacker's Brutes (stack 1, rebirth) die of a retaliation whose strike spells include the synthetic Age (3 rounds); they are now reborn whole (3 creatures at 18 of 18 hit points), so the First Aid Tent has no patient in them |
| B2 | `duel-s1` | final hash only (371 commands and all 14 checkpoints agree) | the dead Spearmen keep the permanent entry of the Brutes' strike spell (synthetic Acid Breath, spell handle 45, defense −3) in the hashed effect list |
| B2 | `skirmish-s7`, `full-s3`, `full-s3-a1766`, `full-s3-a2378`, `full-s3-a303` | the same commands (399, 519, 282, 535, 542) and winners; the state hashes differ from the checkpoints after 225, 225, 125, 375, 175 commands on | the same: from the first death of a stack that carries a permanent entry |
| B2 | `full-s3-a1367` | nothing | |
| B3 | all eight | nothing (recording `b3` is identical to `b2`: commands, checkpoints, final hashes) | B3 changes no state, only an event |
| B4 | all eight | nothing (`b4` identical to `b3`) | it takes a side's sum beyond ±3 together with Mirth, Sorrow, Fortune or Misfortune; no game shows one |
| B5 | all eight | nothing (`b5` identical to `b4`) | the scenarios run in `synthetic:faithful` (Q-0005 on), but the synthetic pack has no creature that ignores a share of the defense |
| B6 | all eight | nothing (`b6` identical to `b5`) | the scenarios' heroes have no spell damage bonus (no `spell_damage` effect), and no value there is above 2^24: the stores change nothing |
| B7 | all eight | nothing (`b7` identical to `b6`) | Brea has the synthetic Sacrifice (`synthetic:offer`) and the quirk is on; the recordings show no difference (it takes a stack that Resurrection's test admits and Sacrifice's own refuses) |
| B8 | all eight | nothing (`b8` identical to `b7`) | |
| B9 | all eight | nothing (`b9` identical to `b8`) | the synthetic pack has no creature that strikes and returns |
| B10 | all eight | nothing (`b10` identical to `b9`) | |

The benchmark baselines (`benches/battle/`) follow the replays' workloads: not run, not touched.

## Rule texts, ledger entries and `installation.md` that are now wrong, with the wording proposed

I read these files and did not edit them. "Old" is the clause as it stands on the branch; "new"
is my proposal. Rule summaries are in `docs/rules/catalog/battle.toml` unless said otherwise.
Where a text is not wrong but would mislead after the fix, it says "(not wrong; clearer)".

### B1 (an aged Phoenix is reborn whole)

- **BAT-MAG-059**, summary. Old: "uses one cast and comes back for good, on its hexes and
  without the effects it died with, with its count at the battle's start div the divisor".
  New: "uses one cast and comes back for good, on its hexes, without the timed effects it died
  with (a permanent one stays, BAT-MAG-010) and at its hit points per creature without Age (its
  death ended Age before the creatures are restored, BAT-MAG-044: an aged Phoenix comes back
  whole), with its count at the battle's start div the divisor". (The first half is B2's.)
- **`installation.md` §7.21**, "The Phoenix's rebirth" (not wrong; clearer). Old: "the raise
  0x5A7560 with `n × health (+0xC0)` hit points and its last argument 0". New: "the raise
  0x5A7560 with `n × health (+0xC0)` hit points (0x46959E to 0x4695AD; the health as the death
  routine left it: 0x444120 has ended every timed spell, and Age's end 0x444740 gave the half
  back, so `n` whole creatures come back) and its last argument 0".

### B2 (a dead stack keeps its permanent effects)

- **BAT-MAG-010**, summary. Old: "A stack that dies loses its effects (without events)."
  New: "A stack that dies loses its timed effects (without events) and keeps its permanent ones
  (Disrupting Ray: the original's death routine 0x444120 ends each spell through 0x444510, which
  returns at once for it; Acid Breath, BAT-MAG-046, lowers the defense directly): they stay in
  the effect list and in the dead stack's values, and a stack that is raised (BAT-MAG-035,
  BAT-MAG-036, BAT-MAG-051) or reborn (BAT-MAG-059) comes back with them. A record reused for a
  newly created stack starts without any effect (BAT-MAG-037)."
- **BAT-MAG-035**, summary. Old: "a dead stack comes back on its hexes with no effects (they
  ended at its death, BAT-MAG-010) and keeps its round's flags". New: "a dead stack comes back on
  its hexes without its timed effects (they ended at its death, BAT-MAG-010), with the permanent
  ones it died with (Disrupting Ray's and Acid Breath's defense loss: the original's raise
  0x5A7560 writes no defense) and keeps its round's flags".
- **BAT-MAG-044**, summary. Old: "a stack that dies loses it with its other effects
  (BAT-MAG-010; the original's death routine 0x444120 ends every spell)". New: "a stack that dies
  loses it with its other timed effects (BAT-MAG-010; the original's death routine 0x444120 ends
  every spell but Disrupting Ray)".
- **BAT-MAG-046**, summary. Old: "a permanent, cumulative defense_loss change (as Disrupting Ray,
  BAT-MAG-009), which no dispel ends." New: "a permanent, cumulative defense_loss change (as
  Disrupting Ray, BAT-MAG-009), which no dispel ends, nor the stack's death: a stack raised or
  reborn comes back with the lowered defense (BAT-MAG-010)."
- **BAT-MAG-037**, summary. Old: "it starts as a stack of its side at the battle's start would
  (the side's hero's attack, defense and stack effects, the native terrain, morale and luck)".
  New: the same, then: "and without any effect of the dead stack whose slot it takes (a dead
  stack keeps its permanent effects, BAT-MAG-010; the original sets a reused record up from
  scratch, 0x47A100)".
- **BAT-MAG-059**: see B1.
- **`installation.md` §7.9**, "Ending". Old: "A stack that dies loses all its spells
  (0x444120)." New: "A stack that dies has every active spell ended (0x444120 calls 0x444510 for
  each): all but Disrupting Ray (below), whose lowered defense stays on the dead record; no
  raise restores it (§7.18)."
- **`installation.md` §7.18**, "Resurrection and Animate Dead". Old: "Its spells ended at its
  death (0x444120 ends every active spell, then sets the dead bit)." New: "Its spells ended at
  its death (0x444120 ends every active spell, then sets the dead bit), Disrupting Ray excepted
  (§7.9); the raise writes no defense, so Disrupting Ray's and Acid Breath's (§7.20) defense loss
  stays on a raised stack."
- **`docs/lanes/battle/HANDOFF.md`**, standing trap (not a rule text, but the next agent reads
  it first). Old: "A dead stack's effects stay until the command's end
  (`effects::remove_dead`): code that brings a stack back within a command removes them first
  (the rebirth, T7c2b review)." New: "A dead stack's timed effects stay until the command's end
  (`effects::remove_dead`), its permanent ones (`rounds == None`: Disrupting Ray, Acid Breath)
  for good: code that brings a stack back within a command removes the timed ones first (the
  rebirth, T7c2b review), and a record reused for a new stack is cleared with `effects::forget`
  (`created::add`). An effect entry is on a living stack or has no duration."
- History only: `docs/rules/surveys/E04b-magic.md` (about line 235), "A stack that dies loses
  its effects."

### B3 (Poison is reported when Age ends at the same round start)

- **BAT-MAG-044**, summary (not wrong; clearer). Old: "and at every round start while it lasts,
  after the effects count down (a Poisoned event reports the hit points)". New: "and at every
  round start while it lasts, after the effects count down (a Poisoned event reports the hit
  points whenever they fell below what they were after the countdown: an Age that ended at that
  round start has given its half back first)".

### B4 (morale and luck are clamped once)

- **BAT-MAG-011**, summary, last sentence. Old: "Morale and luck: the changes on the side summed
  and clamped to ±3, then Mirth's and Fortune's v added and Sorrow's and Misfortune's
  subtracted, then the side's caps and the creature's minimums, then clamped again (luck to
  0..3)." New: "Morale and luck: the changes on the side and the spells' (Mirth's and Fortune's v
  added, Sorrow's and Misfortune's subtracted) are summed without a clamp (the original's stack
  routines 0x43DF20 and 0x43E000 ask the base routines not to clamp), then raised to the
  creature's minimum and lowered to the side's cap, then clamped once to the range the rolls
  read (morale ±3, luck 0..3), which the observation shows too: a base of +5 under Expert Sorrow
  (−2) is +3, not +1."
- **BAT-MOR-001**, summary, end (not wrong; stale). Old: "Morale is materialised when the battle
  starts; no source changes during a battle before E04b (spells)." New: "Morale is materialised
  when the battle starts and whenever a timed spell changes it: Mirth and Sorrow join the sum
  before the minimum, the cap and the one clamp (BAT-MAG-011)."
- **BAT-LCK-001**, summary, end (not wrong; stale). Old: "Spells are E04b." New: "Fortune and
  Misfortune join the sum before the minimum, the cap and the one clamp (BAT-MAG-011)."
- **`installation.md` §7.9**, "Reading the values". Old: "Luck (0x43DF30): the base luck,
  clamped to ±3 by its own function, + Fortune's `v` − Misfortune's `v`, then the Halfling
  minimum; morale (0x43E003): 0 for creatures without morale, else the base morale clamped to ±3
  + Mirth's `v` − Sorrow's `v`, then the Minotaurs' minimum and Spirit of Oppression; the rolls
  clamp both to ±3 again (0x4649AA, 0x43F936)." New: "Luck (0x43DF20): the base luck, **not
  clamped** (the routine pushes 0 for the base routine's clamp argument, 0x43DF6D; 0x44B2D0
  clamps to ±3 only when that argument is set, 0x44B380, and passes it on to the hero's
  routine), + Fortune's `v` − Misfortune's `v`, then the Halfling minimum; morale (0x43E000): 0
  for creatures without morale, else the base morale, not clamped either (0x43E038; 0x44AE60,
  0x44B0B0), + Mirth's `v` − Sorrow's `v`, then the Minotaurs' minimum and Spirit of Oppression;
  the value is clamped to ±3 only where it is used: the good-morale roll (0x4649AA) and the luck
  rolls (0x43F936, 0x441818)." (I re-read the two pushes and the two clamps on 2026-10-12:
  both stack routines zero a register at their start and push it as the base routine's last
  argument, and each base routine returns its sum unclamped when that byte is 0. The three
  roll addresses are those of the commit `6da9dab`. 0x44B2D0 and 0x44AE60 have no entry in the
  address map yet: see "The address map" below.)

### B5 (the Behemoths' defense reduction under Q-0005)

- **BAT-DMG-006**, summary. Old: "Under quirk Q-0005 (faithful) as the executable computes it: D
  − trunc(f32(D) × f32(percent / 100)), the share taken truncated toward 0, so the defense left
  rounds up (7 leaves 5 against a Behemoth, 2 against an Ancient Behemoth; installation.md
  §7.11, the user 2026-10-07); without it (strict-fixed) floor(D × (100 − percent) / 100) toward
  0 (Q19; 7 leaves 4 and 1)." New: "Under quirk Q-0005 (faithful) as the executable computes it
  (0x4425C8 to 0x4425F5): trunc(f32(D) − f32(D) × f32(percent / 100)), the defense left computed
  from single-precision values and only then truncated toward 0; the stored 0.4 and 0.8 lie a
  hair above the decimals, so a whole D × percent leaves one point less than the exact formula
  (7 leaves 4 against a Behemoth and 1 against an Ancient Behemoth, as exact; 10 leaves 5, exact
  6; 5 leaves 0 against an Ancient Behemoth, exact 1; VCMI's `floor(D × percent / 100) + 1`
  ignored points, at most D, is the same number; installation.md §7.11, the user 2026-10-07);
  without it (strict-fixed) floor(D × (100 − percent) / 100) toward 0 (Q19; 7 leaves 4 and 1)."
- **`installation.md` §7.11**, "Behemoths". Old: "against a Behemoth (creature 96) the defense is
  `D − trunc(f32(D) × f32(0.4))`, against an Ancient Behemoth (97) `D − trunc(f32(D) ×
  f32(0.8))`: the share taken is truncated, so the defense left rounds up (D = 7: 5 and 2), not
  down as Q19 chose (4 and 1)." New: "against a Behemoth (creature 96) the defense is
  `trunc(f32(D) − f32(D) × f32(0.4))`, against an Ancient Behemoth (97) `trunc(f32(D) − f32(D) ×
  f32(0.8))` (0x4425C8 to 0x4425F5; the Ancient Behemoth's twin at 0x4425DE): the defense left
  is computed in single precision and only then truncated; the share taken is never rounded on
  its own. The result is Q19's except where D × 0.4 (D × 0.8) is whole, where the stored
  constants leave one point less (D = 7: 4 and 1; D = 10: 5 against a Behemoth; D = 5: 0 against
  an Ancient Behemoth)."
- **Quirks ledger, Q-0005**: its text has no Behemoth clause. Proposed addition to "Context /
  rules affected": "the Behemoths' defense reduction (BAT-DMG-006)"; to "Regression test":
  `crates/h3-battle/src/tests/abilities.rs`, `defense_reduction_ignores_a_share_of_the_defense_rounded_down`.
- **Quirks ledger, Q-0009** and **BAT-MAG-019**: no number; nothing wrong. The regression test
  named there now also has the case of both quirks (profile `t:frenzy_single`: 25).

### B6 (spell damage under Q-0005: the stores; saturation)

- **BAT-MAG-004**, summary. Old: "first the caster's bonuses, v × (1 + the best bonus among the
  spell's schools) (the Orbs, +50 %) × (1 + the every-spell bonus) (Sorcery), the product kept
  fractional and rounded down once to t; then t + this spell's own bonus of t rounded down
  (Ciele +50 %, Luna +100 %) + a spell specialist's ⌈per-level rate × (hero level div the
  stack's creature level, at least 1) × t⌉ (3 % per level in SoD; HER-BAT-012); […] Rates are
  exact, or single precision under quirk Q-0005 (Sorcery's 5/10/15 %)." New: "first the bonuses
  of the caster's hero (none without a hero: the original runs this stage on the hero only,
  0x5A78E0, so a stack's cast, the Thunderbirds' bolt and a hero-less side's shield, wall or
  mine keep the value): v × (1 + the best bonus among the spell's schools) (the Orbs, +50 %) ×
  (1 + the every-spell bonus) (Sorcery), whose whole part is t; then this spell's own bonus of t
  rounded down (Ciele +50 %, Luna +100 %) and a spell specialist's ⌈per-level rate × (hero level
  div the stack's creature level, at least 1) × t⌉ (3 % per level in SoD; HER-BAT-012). With
  exact rates the product is kept fractional and rounded down once to t, and the bonuses are
  added to t. Under quirk Q-0005 the rates are single precision (Sorcery's 5/10/15 %) and the
  value is kept as the executable keeps it (0x4E5760): v, its product with the school factor and
  that one's with the every-spell factor are each stored as a single-precision number (rounded
  to 24 bits, to nearest), the bonuses of the last product's whole part t are added to the
  stored product itself, and that sum, stored once more, is rounded down; a product a hair below
  a whole number so becomes that number (a Sorcery specialist's 16 % on 50: 57.99999982 is
  stored as 58, the damage of the exact rates). […] The damage saturates for any value and
  rates." (The clause "at least 1" is r1 F-7's, not mine: left as it is.)
- **BAT-MAG-018**, summary (not wrong; clearer). Old: "then r is the shield spell's damage
  (BAT-MAG-004) with the bonuses of the struck stack's hero as the caster's". New: the same,
  then: "(without a hero r is the value as it is)". The same holds for BAT-MAG-028's and
  BAT-MAG-033's "the bonuses of the hero of the side that cast it".
- **Quirks ledger, Q-0005**, "Flag", the spells' clause. Old: "the Orb and Sorcery factors are
  single precision and their product is rounded down once;". New: "the Orb and Sorcery factors
  are single precision, and the value is kept as a single-precision number through the caster's
  stage as the code keeps it (0x4E5760; E04b T11b, review r1 F-2): the value, its product with
  the Orb's factor and that one's with 1 + Sorcery are each stored in single precision, the
  specialist's bonus is added to the stored product and the sum, stored once more, is rounded
  down, so a product a hair below a whole number becomes that number (a Sorcery specialist's 16 %
  on 50 deals 58; the exact product's floor is 57);". To "Regression test":
  `crates/h3-battle/src/tests/magic.rs`, `the_casters_products_are_stored_as_singles_with_quirk_q0005`,
  and `crates/h3-pack/tests/harvested_spell_damage.rs`,
  `a_sorcery_specialists_product_is_stored_as_a_single_before_it_is_rounded_down`.
- **`installation.md` §7.8**, step 1. Old: "both products are stored as `f32`, **not
  truncated**; then, if the spell has a target stack, the specialty bonus (0x4E5FF0) computed
  from `t = trunc(v)` is added as an integer and the result truncated." New: "both products are
  stored as `f32`, **not truncated**; then, if the spell has a target stack, the specialty bonus
  (0x4E5FF0) computed from `t = trunc(v)`, a whole number, is stored as an `f32` and added to
  the stored `v` itself (not to `t`), and the sum, stored as an `f32` once more, is truncated
  (0x4E5932 to 0x4E5944); without a target stack `v` is truncated as it is. A store rounds to 24
  bits, to nearest, so a product a hair below a whole number becomes that number before the
  truncation." And to the paragraph above the list ("passed through the damage-modifier
  function 0x5A78E0, which applies in this order"): "step 1 only when the routine is given a
  hero (0x5A790D)".
- **HER-BAT-012** (`heroes.toml`): nothing wrong (its example is the derivation's value). After
  the fix the two values of its example (0.2250000089… and f32(0.225)) give the same spell
  damage in every case tried (see B6's notes); the physical damage's example (`tests/rules.rs`,
  245 against 244) is another formula and unchanged.
- History only: `docs/epics/E04b-log.md` (about lines 201–203 and 494: "level 10, value 200: 245
  against our 244", "`tests/magic.rs` (the same through a cast)"): with the stores both rates
  deal 245 in the cast; the test now pins 245 for both and shows the rate's own value with 0.25.

### B7 (Sacrifice's raised stack passes Sacrifice's own test too)

- **BAT-MAG-036**, summary. Old: "With quirk BAT-QUIRK-5 (faithful) the target is tested as the
  tests_as spell's would be (Resurrection's, BAT-MAG-035: its conditions, its level 4 against
  immunities and Anti-Magic, so Gold Dragons are never a target (survey S-15), and its hit
  points at the caster's mastery in it, which must reach one creature's health); without it by
  Sacrifice's own conditions and level, with no hit-point minimum." New: "With quirk BAT-QUIRK-5
  (faithful) the target is found as the tests_as spell's would be (Resurrection's, BAT-MAG-035:
  its conditions, its level 4 against immunities and Anti-Magic, so Gold Dragons are never a
  target (survey S-15), and its hit points at the caster's mastery in it, which must reach one
  creature's health; a corpse is on top among those that pass this test) and must then pass
  Sacrifice's own test as well: not created in the battle (BAT-MAG-037), and Sacrifice's own
  conditions, level 5 and Fire school against immunities and Anti-Magic, so Efreet, living or
  dead, are no target; a corpse that fails this second test still covers the corpses below it.
  Without the quirk the target is tested by Sacrifice's own conditions and level alone, with no
  hit-point minimum."
- **BAT-MAG-036**, the `original-data` source's `ref`. Old: "0x5A3950 finds the raised stack
  through 0x5A3CC0, which tests spell 38 (Resurrection); 0x5A8090 case 0x5A8523 wants the victim
  living (0x10), not summoned (0x400000), not undead, with creatures, of the caster's side by
  controller and real side". New: the same, then: "; the target test 0x5A39C0 then asks 0x5A8090
  about spell 40 for the stack found, with its fifth argument set: the case skips the
  controller's test (0x5A82B9) and wants it living (0x10), not summoned (0x400000) and with
  fewer creatures than at the start, then the general immunities (0x44A4D0 with spell 40)".
- **Quirks ledger, BAT-QUIRK-5**, "Original behavior". Old: "[…] and Resurrection's hit points
  at the caster's Earth mastery, which must reach one creature's health, although Sacrifice
  raises its own amount." New: the same, then: "The stack so found must pass Sacrifice's own test
  as well (0x5A39C0 asks 0x5A8090 about spell 40 for it as the stack to raise: a living creature,
  not a summoned stack, fewer creatures than at the start, Sacrifice's level 5 and Fire school
  against immunities and Anti-Magic), so Efreet are never raised by Sacrifice either." To
  "Regression test": `with_bat_quirk_5_the_raised_stack_passes_sacrifices_own_test_too`.
- **`installation.md` §7.18**, "Sacrifice". Old: "the raised stack takes the immunity factor of
  spell 38 as above, the victim that of spell 40 (level 5: no creature's spell-level immunity
  reaches it)." New: "the raised stack takes the immunity factor of spell 38 as above (the
  finder) and then that of spell 40 too: the target test 0x5A39C0 asks 0x5A8090 about the spell
  itself for the stack found, with its fifth argument set, which skips the controller's test
  (0x5A82B9) and in case 0x5A8523 wants the creature flag `0x10`, no flag `0x400000` and fewer
  creatures than at the start, then the general tests (the Fire school against fire immunity
  `0x4000`, level 5); the victim takes that of spell 40 with the argument 0 (level 5: no
  creature's spell-level immunity reaches it)." The paragraph "Which stack a cast raises" ends
  "so Sacrifice's raised stack must be a valid Resurrection target": add "and pass spell 40's
  own test (below)".

### B8 (no Anti-Magic on a clone, whoever casts)

- **BAT-MAG-021**, summary, last sentence (not wrong; too narrow). Old: "No clone may be aimed at
  (the original's 0x5A8090, installation.md §7.13; E04b T6b, BAT-MAG-038)." New: "No clone takes
  it, whoever casts: a hero may not aim it at a clone, a mass version passes over one and it is
  never useful on a clone for a stack that draws its spell (BAT-MAG-050); the original's
  0x5A8090 has the test in the spell's own case (installation.md §7.13; E04b T6b, BAT-MAG-038)."
- **BAT-MAG-050**, summary (not wrong; clearer). Old: "can affect it (a positive spell an own
  stack; immunities, Anti-Magic)". New: "can affect it (a positive spell an own stack;
  immunities, Anti-Magic; Anti-Magic itself never a clone, BAT-MAG-021)".
- `installation.md` §7.13 is right as it stands.

### B9 (the Harpies' return lands on its hexes)

- **BAT-ATK-004**, summary. Old: "the return is a move event and costs nothing." New: "the
  return is a move like any other and costs nothing (the original returns through its move
  handler, 0x445950): the stacks the returning stack binds are released (BAT-MAG-045) and it
  enters the hexes it lands on (a walker: those of its path), where a Fire Wall burns it
  (BAT-MAG-028) and an unseen enemy mine goes off (BAT-MAG-033), each followed by the rebirths
  (BAT-MAG-059). The battle's end is checked after the whole action (the original checks
  nothing before the return): a stack that dies where it lands as its side's last ends the
  battle; one whose strike killed the last enemy stack still lands and burns, and if it dies
  there nobody wins (BAT-END-002)."
- **BAT-MAG-028**, summary. Old: "A stack that moved and then attacks is not burnt again after
  its attack." New: the same, then: "The return of a stack that strikes and returns is a move
  of its own: it burns on the hexes it lands on, also once its strike has left the other side
  without stacks (BAT-ATK-004)." (The clause before it, "and on each hex it stands on after an
  action that does not move it […], unless the battle is over", stays: the two cases differ, see
  B9's notes.)
- **BAT-MAG-033**, summary (not wrong; clearer). Old: "A mine goes off when a stack enters its
  hex (each hex a walker enters on its path, its head's and tail's; a flyer or teleporter only
  where it lands, BAT-MOV-002) or acts there without moving". New: the same with "a flyer or
  teleporter only where it lands, BAT-MOV-002, also when it lands after its strike, BAT-ATK-004".
- **BAT-MAG-045**, summary (not wrong; clearer). Old: "Its Bind ends (reported) when its last
  binder moves (walk, flight, teleporting move)". New: "Its Bind ends (reported) when its last
  binder moves (walk, flight, teleporting move, also its return after a strike, BAT-ATK-004)".
- **`installation.md` §7.15**, "Entering a hex" (not wrong; incomplete). Old: "a stack that moved
  and then attacked is not (0x478900 skips it)." New: the same, then: "The Harpies' return
  (0x4789F8, §7.20) is the move routine again: the marks are cleared, the hexes the stack stands
  on are marked, and the flight enters the hexes it lands on (Fire Wall, then Land Mine; the
  start hex is no longer marked). Nothing tests the battle's end before the return."
- **`installation.md` §7.20**, "Strike and return" (not wrong; incomplete). Old: "after the
  attack the action handler flies a Harpy or Harpy Hag (72, 73) that moved back to its start hex
  (0x4789F8, the move 0x445D10 → 0x445950) unless it is dead or its Blind (+0x290), Stone Gaze
  (+0x2B0) or Paralyze (+0x2C0) duration is above 0; Bind's (+0x2B8) is not tested." New: the
  same, then: "The return is a move like any other: 0x445950 needs a hex where the stack fits
  (0x446C40), clears the entered-hex marks and marks the hexes it stands on (0x46A520), and the
  flight (0x4B49C0) enters the hexes it lands on (§7.15), each damage followed by the rebirth
  check (§7.21)."

### B10 (a clone is not "raised")

- No text is wrong: BAT-MAG-038 says "with traits SUMMONED and CLONE", BAT-MAG-052 gives RAISED
  to the stack that rose from the corpse. Optional, BAT-MAG-038: "(a clone of a stack raised
  from a corpse, BAT-MAG-052, is not RAISED)".

### The address map (`docs/exe/map.toml`, summaries of entries)

`cargo xtask check-exe-map` passes on the branch (the code cites only covered addresses). These
summaries state the old readings:

- **0x43DF20 `stack_luck`** (B4). Old: "A stack's luck: base luck clamped to +-3, + Fortune's v
  - Misfortune's v, then the Halfling minimum (the docs cite 0x43DF30, inside it)." New: "A
  stack's luck: the base luck, not clamped (the base routine's clamp argument is pushed as 0,
  0x43DF6D), + Fortune's v - Misfortune's v, then the Halfling minimum (the docs cite 0x43DF30,
  inside it)."
- **0x43E000 `stack_morale`** (B4). Old: "A stack's morale: 0 without morale, else base clamped
  to +-3 + Mirth's v - Sorrow's v, then the Minotaurs' minimum and Spirit of Oppression (the
  docs cite 0x43E003)." New: "A stack's morale: 0 without morale, else the base, not clamped
  (the base routine's clamp argument is pushed as 0, 0x43E038), + Mirth's v - Sorrow's v, then
  the Minotaurs' minimum and Spirit of Oppression (the docs cite 0x43E003)."
- **New entries** if `installation.md` §7.9 takes the wording proposed under B4 (it cites them;
  the tool knows them as unnamed functions): 0x44B2D0, function, size 0xEB, "The base luck of a
  side's stack; clamps to +-3 only when its last argument is set (0x44B380). Read in part.";
  0x44AE60, function, size 0x29A, "The base morale of a side's stack; clamps to +-3 only when
  its last argument is set (0x44B0B0). Read in part."
- **0x442590 `stack_defense_against`** (B5). Old: "the Behemoths' share (truncated, rounds up)".
  New: "the Behemoths' reduction (the defense left computed in f32, then truncated: 0x4425C8 to
  0x4425F5)".
- **0x4E5760 `hero_spell_damage_bonus`** (B6; incomplete). Old: "x 1.5 Orb, x (1 + Sorcery), f32
  not truncated, then the specialty bonus on a target stack." New: "x 1.5 Orb, x (1 + Sorcery),
  each product stored as f32 and not truncated; on a target stack the specialty bonus of the
  truncated value is added to the stored f32 and the sum stored as f32; the last value is
  truncated."
- **0x5A8523 `spell_factor_sacrifice`** (B7; incomplete). Old: "Sacrifice's victim: living, not
  summoned, not undead, with creatures, of the caster's side by controller and real side." New:
  "Sacrifice's case: living, not summoned; as the stack to raise (fifth argument set, no
  controller test: 0x5A82B9) fewer creatures than at the start; as the victim not undead, with
  creatures, of the caster's side by controller and real side."

## For the coordinator

- **Decisions I took that go beyond the list's words** (each explained under its fix):
  B6, `spell_damage` takes the hero's bonuses as an option, and no hero means no caster stage;
  B6, the bounded property test is widened; B7, the order "found by Resurrection's test, then
  Sacrifice's own" for corpses, and `strict-fixed` left as it is; B9, the battle's end is
  checked once after the return, so the landing happens even when the strike has won; B9, a
  returning binder releases its stacks; B10, a crate-private `StackTraits::without`.
- **Merging with worker `a`.** I read `/home/claude/wt-a` (and changed nothing there), last at
  09:37 with its HEAD at `b118286` and uncommitted work. Files both branches change, with the
  places (line numbers of the common base `68838df`):
  - `after_strike.rs`: mine the import (22) and one line of `thunderbolt`'s call of
    `spell_damage` (237: `None` for `&SpellBonuses::default()`); `a`'s hunks in `thunderbolt`
    end 5 lines above it and start 3 lines below it;
  - `combat.rs`: `a` at 34, 243, 606, 821 to 851; mine 699 and 759 (the return in `melee`);
  - `effects.rs`: `a` at 176 and 658; mine 825 to 946 (`count_down`, `remove_dead`, `forget`);
  - `magic.rs`: `a` at 848 to 862 (`admits`, `skips_hypnotized`) and 1927 (`aura`); mine 751 to
    770 (`affects_stack`), 1071 to 1104 (`grants_immunity`, `stack_target`), 1684 to 1733
    (`spell_damage`, with the new `caster_stage` after it), 1835 (`reflection`), 2012 (`affect`),
    2454 (`burn`);
  - `obstacles.rs`: `a` at 303 and 539; mine 618 to 852 (`enter`, `explode`, `travel`,
    `travel_back`, `travel_through`, `stand_on`);
  - `state.rs`: `a` at 712; mine 209 (`StackTraits::without`);
  - tests: `created.rs` (`a` 298 to 323, mine 542), `strike.rs` (`a` 362; mine 604, 807, 909),
    `support.rs` (`a` 974 to 984 and 1394, 1420, 1594; mine 1478, the profile
    `t:frenzy_single`).
  Only mine: `binds.rs`, `created.rs`, `damage.rs`, `morale.rs`, `raise.rs` and the tests
  `abilities`, `casters`, `magic`, `mines`, `obstacles`, `properties`, `rage`, `raise`,
  `raisers`, `rebirth`, `timed`; `crates/h3-pack/tests/harvested_spell_damage.rs` (`a` has
  `harvested_heroes.rs`). If `a` adds a call of `magic::spell_damage`, it needs `Some(&bonuses)`
  or `None`. `a` changes the test creature `azure` to `fear=99`: the widened property test's
  battles will differ after the join (it has room: see B6).
- **Coverage was not run** (the brief). New branches and the fixed tests that reach both sides:
  `caster_stage`'s `if single` (`the_casters_products_are_stored_as_singles_with_quirk_q0005`);
  the exact branch's overflow (`the_spell_damage_saturates_for_every_value_and_rate`);
  `spell_damage` without a hero (the Thunderbirds' tests) and with one; `raisable`'s own test
  (`with_bat_quirk_5_the_raised_stack_passes_sacrifices_own_test_too`: created, fire-immune,
  admitted); `affects_stack`'s clone clause (`anti_magic_is_never_useful_on_a_clone`); `enter`'s
  and `explode`'s `if ends` (false: the two return tests; true: every older test);
  `travel_back`'s two paths (plain: `a_harpy_bound_or_blinded_by_the_retaliation_stays_where_it_struck`;
  through obstacles: the return tests).
- **Benchmarks** (`benches/battle/`): not run, not touched. B6 changes the arithmetic of a
  hero's damage spell under Q-0005 (X87 stores in place of two `u128` products and a division),
  B8 adds one bit test to `affects_stack`, B9 a call of `binds::release` and one test of the
  obstacle sets to a return; no field was added to `Stack` or the state.
- **No version, seam or wire type changed**: no new `Command`, `EventKind`, `CommandError`,
  `CastTarget`, `DamageRoll` or `ObstacleKind` variant, no `Observation` field, no state field,
  no quirk flag, nothing in `h3-kernel`, `h3-contracts`, `Cargo.toml`, `xtask` or `docs/`.
  State hashes change where behaviour changes (B1, B2: six golden replays; B10: only a state
  with a clone of raised Demons).

## Closing checks (after B10, HEAD `417f58e`, 2026-10-12)

- `cargo fmt --all --check`: clean. `cargo clippy -p h3-battle -p h3-pack --all-targets -- -D
  warnings`: clean (the two crates this branch touches).
- `cargo test --no-fail-fast -p h3-battle -p h3-content -p h3-heroes -p h3-pack -p h3-sim
  -p h3-cli -p h3-ffi -p h3-protocol`: all pass (battle 463, content 79, heroes 23, pack 17 + 3
  + 3 + 1 + 2 + 5, sim 29, ffi 5, protocol 61, cli 3 + 1) **except**
  `h3-cli --test replay_fixtures`, `golden_replays_reproduce_their_hashes`, which stops at the
  first changed replay (`duel-s1`: "state hash after 371 commands"): the golden replays of B1
  and B2, not regenerated (table above).
- `cargo test -p h3-battle` twice more: 463 passed each time (three runs in all).
- `cargo xtask check-rules`: ok (183 rules, 403 rule markers, 814 covers markers);
  `check-arch`: ok; `check-stubs`: ok (1 registered stub); also `check-exe-map`: ok (341
  entries, every cited address covered).
- Not run, as the brief says: coverage, `cargo xtask verify`, the benchmarks.
- The branch is pushed: `origin/battle/e04b-t11b-b` at `417f58e`. Logs: `<SCRATCH>/b-logs/w2/`
  (`final-1.log`, `final-2.log`; the arithmetic checks `check1.py` to `check3.py`); the replay
  recordings `<SCRATCH>/b-logs/rep/b2` … `b10`.
