# Review r1-casting-core: BAT-MAG-001..008 and the Heroes context's magic rules (HER-SPL-001..003, HER-ART-003, HER-BAT-011..014)

Method: every rule read (`rules.py`), the implementing code read completely (`h3-battle/src/magic.rs`
whole, `command.rs`, `hero.rs`, `mana.rs`, `turn.rs` round start, `h3-heroes/src/spells.rs`,
`derive.rs`, `details.rs`, `h3-contracts/src/hero.rs`, `h3-kernel/src/arith.rs` X87 and
single-precision helpers), `installation.md` §7.6, §7.8, §7.22, batch 2a, and the executable re-read
with `scripts/exe.py` at the addresses listed per rule. Nothing in the repository was changed, no
cargo run. Two brute-force comparisons of the engine's formula with the executable's arithmetic
(Python, IEEE doubles and `f32` stores) are in `<SCRATCH>/r1/stage1.py` and `stage1b.py`.

Executable addresses re-read (function, or range inside a long one): `cast_spell` 0x59FE30
(0x59FE30–0x5A03C8 entry, mastery, cost, cast mark, the aimed stack's roll; 0x5A097D–0x5A0C81 the
single-target handlers; 0x5A0C81–0x5A0FA0 Death Ripple and Destroy Undead), 0x5A4BC0 (Armageddon,
0x5A4BC0–0x5A4CE7; the map still calls it "inferred"), 0x5A4970, 0x5A46F0, 0x5A4170 (area),
0x5A6360, 0x5A61F0 (chain), 0x446660, 0x446630, 0x5A78E0 (to 0x5A7AA2), 0x4E5760, 0x4E5FF0,
0x44B4B0, 0x5A8090 (whole), 0x44A4D0 (whole), 0x4E46E0, 0x442690, 0x43EA70 (aura lists), 0x59E060,
0x5A39C0 (to 0x5A3B33), 0x5A3950, 0x59F700, `enter_player_cast` 0x59EC50 (entry, 0x59ED5A–0x59F046),
0x59E900 (to 0x59EAC2), 0x41F890, 0x4D92D0, `round_start` 0x475ED0 (0x476060–0x476180), the
activation's acting side 0x465351–0x4653CB, the menu toggle of +0x13D74 at 0x4F4603, 0x50B230,
0x444090, 0x4E5080, 0x4E5100, 0x4E5240, 0x4D95D0, 0x4D9350, 0x4E4DB0. Data read: 0x642274 (chain
jumps 4, 4, 5, 5), 0x63EA78 (Sorcery f32 0, 0.05, 0.10, 0.15), 0x63EA48 (Resistance f32 0, 0.05,
0.10, 0.20), 0x63B998 (the nine spell-giving artifacts), creature levels at 0x6703B8 for ids
112–115, 123, 125, 127, 129.

## Coverage

| rule | text↔code | sources checked (exe addresses re-read; VCMI files; batch rows) | tests | verdict |
|---|---|---|---|---|
| BAT-MAG-001 | every clause traced (`check_spell`, `plan`, `apply_cast`, `start_round`); summary carries history wording | exe: cast mark +0x54B4 set 0x59FFF6 (hero casts only), tested 0x59E948 and 0x59EC76 (unless the byte +0x13D74 is set, which only a menu command toggles, 0x4F4603: not a rule), cleared per side at the round start 0x4760EE; book needs artifact 0 (0x41F890); VCMI `CBattleInfoCallback.cpp` L116–159, `BattleInfo.cpp` L651–670, `BattleActionProcessor.cpp` L1705–1748: as cited; batch 2a R6: casts in the Ballista's turn, yes; a refused second cast is not in R6 | once per round, tactics, war machine turn, killed active stack: pinned (synthetic pack) | F-r1-8, F-r1-9 (both low) |
| BAT-MAG-002 | ok | exe 0x4E5080, 0x4E5100, 0x4E5240, 0x4D92D0 agree with §7.22 and the summary; VCMI `CGHeroInstance.cpp` L792–813, `CBattleInfoCallback.cpp` L2111–2136: as cited | mastery selects cost and effect, mana below cost: pinned; mana equal to cost: not pinned | ok; F-r1-10 (low) |
| BAT-MAG-003 | stack, hex and side clauses ok; the no-target clause differs from the book's test | exe 0x59E060, 0x5A39C0, 0x5A3950, 0x59F700 (side columns, Chain Lightning refuses the acting side's stacks), 0x59ED5A–0x59EE28; VCMI `UnitEffect.cpp` L288–293, `offensive.json`: as cited; batch 2a R1, R3, R4: show what is claimed | R1 (own Pikemen), R3 a–c, R4 replayed (R3, R4 on harvested packs) | F-r1-3 (low) |
| BAT-MAG-004 | order of stages ok; under Q-0005 the `f32` stores of the products are missing; "at least 1" misreads | exe 0x5A78E0 (hero, then creature, then protection), 0x4E5760, 0x4E5FF0 (hero level div (0-based creature level + 1) first, × t, × 0.03 as f64, ceil; Magic Arrow t >> 1; Fire Wall t), 0x44B4B0 (elementals × 2, Golems / 2, / 4, × 15 / 100, / 20); VCMI `CSpellHandler.cpp` L378–424, `CGHeroInstance.cpp` L815–837: as cited, VCMI's numbers 118/116/16/231 and 172/24 recomputed; batch 2a R4, R5: 119, 115, 17, 230, 173, 25 recomputed by hand from the exe's formula | R4, R5 replayed on harvested packs with the same numbers; order test in the synthetic pack | F-r1-2 (medium), F-r1-5 (robustness), F-r1-7 (low) |
| BAT-MAG-005 | ok (conditions, level, school, spell, absolute, mind; Dispel ignores); two sentences stale | exe 0x5A8090 and 0x44A4D0 whole: both switch tables decoded and compared with the summary and with the packs' `excludes`/`requires` for the spells of this slice (Blind excludes undead, Poison requires alive: present); +0xEA (killed, death pending) is the engine's `is_alive`; VCMI `TargetCondition.cpp` L130–160, L222–240: as cited; batch 2a R3, R4 | replayed on harvested packs | ok; F-r1-8 (low) |
| BAT-MAG-006 | halving, count, first target, pass-over for one jump, strict `<`: ok; the side order is reversed | exe 0x5A6360, 0x5A61F0, 0x446660, 0x446630, table 0x642274; VCMI `UnitEffect.cpp` L190–250 (resisting later target skipped, but dropped for good there), `Damage.cpp` L155–165, `offensive.json`: as cited; batch 2a R1, R2: geometry and the tie inside one side, counts 35/70/85 and 35/69/86 recomputed (65, 32, 16, 8) | geometry, two-hex shift, tie by id inside a side: pinned; a tie between sides: never tested; R1/R2 not replayed on harvested packs | F-r1-1 (medium), F-r1-12 (question), F-r1-11 (low) |
| BAT-MAG-007 | ok | exe 0x5A4970 (radius 2 for Inferno, 1 else, centre left out for Frost Ring; each stack tested and rolled; no first roll), 0x5A46F0, 0x5A4170 (hex metric identical to `Field::distance`; the list keeps to columns 1–15, which no stack hex of the engine leaves), Death Ripple 0x5A0C81, Destroy Undead 0x5A0E7E, Armageddon 0x5A4BC0: every stack of both sides, attacker's first, tested and rolled; VCMI `offensive.json` ranges, `BattleSpellMechanics.cpp` L446–463: as cited; batch 2a R3 | pinned, R3 statistics in the synthetic pack | ok for a hero's cast; F-r1-12 (question) for a creature's |
| BAT-MAG-008 | ok for stacks that are not hypnotized | exe 0x44A7F5–0x44A810 (`f = f32(f − r)`, only with a controlling hero), 0x4E46E0 (skill, grown for specialty 26, + 0.05, + 0.10, + 0.15 in artifact id order, each sum an `f32`, at most 1), 0x5A8592–0x5A85CF (aura × 0.8f for neutral and negative spells), rolls `random(1, 100) ≤ trunc(f × 100.0f)` at 0x5A02EA, 0x5A4A54, 0x5A6272, 0x5A0CE6, 0x5A0EE3, 0x5A4C2E; 0x50B230 inclusive; 89 % and 94 % recomputed (also 15 % → 85 % by the tie to even, 20 % → 80 %); VCMI `CStack.cpp` L92–107, `BasicTypes.cpp` L41–46: as cited. No observation: the single-precision chances rest on the reading; this re-read is the second look | exact and single variants, aura, positive spell: pinned | ok; F-r1-4 (low) |
| HER-SPL-001 | ok (`validate_spells`) | SPTRAITS sections (pack categories) | pinned | ok |
| HER-SPL-002 | ok (`available`, `SpellGrant::gives`; the compiler refuses a grant naming an ability) | exe 0x4D95D0 (learned copied; scroll's second word; own set and parts' sets of a worn combination; backpack not read), 0x4D9350 (Tomes by school bit over spells 0–69, Hat level 5, 128 → 26, 123 → 0 and 1, 135 → 57; every other set clears 57), list 0x63B998 | synthetic and harvested | ok |
| HER-SPL-003 | ok (`masteries`) | exe 0x4E5080, 0x4E5100 (best of Air +0xD8, Fire +0xD7, Earth +0xDA, Water +0xD9 among the school bits; battlefield raise left to E04c) | synthetic and harvested | ok |
| HER-ART-003 | ok (`validate_scroll`) | exe 0x4D95D0 (artifact 1) | pinned | ok |
| HER-BAT-011 | ok (`magic`) | exe 0x41F890 (a hero's cast needs worn artifact 0); VCMI `CBattleInfoCallback.cpp` L146 | pinned | ok |
| HER-BAT-012 | ok for what crosses the seam (`spell_bonuses`, `Growth::single`, `RateSum`); the battle's use of the single value: F-r1-2 | exe 0x4E58AE–0x4E58FE (Sorcery value, grown `f32((f32(level) × 0.05f + 1) × s)` for specialty skill 25), 0x4E578A–0x4E58AB (Orbs 79 air, 80 earth, 81 fire, 82 water, × 1.5 once), 0x4E5FF0; 0x3E666667 for level 10 Expert recomputed; VCMI `skills.json`, `artifacts.json` values checked (5/10/15; 50) | derivation pinned, also under Q-0005 | ok (see F-r1-2) |
| HER-BAT-013 | ok (`magic_resistance`, `HeroMagic::of`) | exe 0x4E46E0; VCMI `skills.json` 5/10/20, `artifacts.json` 5/10/15 checked | pinned | ok |
| HER-BAT-014 | ok (`spell_duration`) | exe 0x4E4DB0 (76 → 1, 77 → 2, 78 → 3, 139 → 50, each also as a part of a worn combination); VCMI `artifacts.json` 1/2/3/50 checked | pinned | ok |
| BAT-QUIRK-19 | data quirk, both variants in the packs | creature table read: Ice 4, Magma 3, Energy 5 against Water 3, Earth 5, Fire 4 (Storm 2 = Air 2); the specialty divisor reads the level from the stack's creature copy (+0x78, 0x4E5923) | `the_data_quirks_are_fixed_in_strict_fixed` asserts each profile's levels (each fails for the other); no battle test of a specialist's bonus on these creatures (not needed) | ok; class fix-optional fits §4 |
| Q-0011 and BAT-QUIRK-9 (other reviewers' rules; by-product of the paths read here) | — | Q-0011: 0x59E060 gives a spell with flag 0x20 a target stack only up to Basic, and spell 61 alone has the flag (§7.6 values): Forgetfulness is mass from Advanced, confirmed. BAT-QUIRK-9: 0x5A02CB–0x5A02CF skips both the effect test and the roll of the aimed stack for caster kind 1, confirmed | not checked here | second look done for the deciding path only |
| Q-0005 (as far as BAT-MAG-004, BAT-MAG-008, HER-BAT-012, HER-BAT-013 use it) | resistance: both variants right; spell damage: the single variant is not the original's arithmetic for rates below their decimals | this re-read (0x4E5760, 0x44A4D0, 0x4E46E0) | resistance 89/94 against 90/95: pinned both ways; spell damage: the pinned single value (244) is the engine's model, not the original's | F-r1-2 |

## Findings

By weight: F-r1-1, F-r1-2, F-r1-12 (question), F-r1-5 are medium; the rest low.

### F-r1-1: Chain Lightning searches the attacker's stacks first in the original; the engine, the rule and §7.8 have the defender's first

- rules: BAT-MAG-006
- kind: behaviour-bug (with it: rule-text, source, test-gap)
- severity: medium (needs a tie in truncated pixel distance between a stack of each side; symmetric
  positions around the last stack struck give one, e.g. two hexes left and right in one row)
- confidence: high for the code (structure of one short function, checked against three other
  facts); it stands alone against the wiki ("randomly chosen") and VCMI (a positional tie-break,
  `UnitEffect.cpp` L253 `getClosestTile`), so an experiment should confirm it: first target an enemy
  stack A; one own stack and one enemy stack of the *same creature type* (so the sprite lift of
  Q43 cancels) each two hexes from A in A's row; the second strike hits the own stack (reading)
  or the enemy stack (engine).
- evidence:
  - Engine: `crates/h3-battle/src/magic.rs:2106` `for side in [BattleSide::Defender,
    BattleSide::Attacker]`, first of equals kept (`:2116`); doc comment `:2091–2094`. Rule summary:
    "The defender's stacks are searched first, then the attacker's". `installation.md:368–369`,
    `docs/rules/surveys/E04b-magic.md:158`, `docs/exe/map.toml` 0x5A61F0 ("defenders first") say
    the same.
  - Executable, 0x5A61F0: at 0x5A6223–0x5A622F the function points at the *first* side's data:
    struck marks at manager +0x547C, stack records at +0x54CC, the record count at +0x54BC
    (`structs.toml`: `stacks battle_stack[2][21]` from +0x54CC, `stack_count i32[2]` at +0x54BC).
    After the inner loop over that side's records each pointer moves on by one side (0x5A6328–
    0x5A6331: + 4, + 0x6EE8, + 20). The counter that starts at 1 and ends at 0 (0x5A6235, 0x5A6334)
    is never used as an index: its one use is the second argument of the effect test (0x5A6269),
    the side taken as the caster's. So the first pass tests side 0's stacks "as cast by side 1",
    the second pass side 1's "as cast by side 0". The earlier reading took the counter for the side
    searched.
  - Side 0 is the attacker: the handler marks a struck stack at +0x547C + side × 20 + index with
    side and index from the stack's +0xF4 and +0xF8 (0x5A657E–0x5A659A; +0xF4 is "0 attacker, 1
    defender" in `structs.toml`), the same marks the search reads first; and the cast routine
    computes the missile's origin for a hero of side 0 near the left edge of the screen (a table
    value − 43, 0x5A00C4–0x5A0106) and for side 1 mirrored near the right edge (… + 693,
    0x5A0173–0x5A01B3), where the attacker's and the defender's hero stand.
    Death Ripple (0x5A0CA5), Destroy Undead (0x5A0EA2) and Armageddon (0x5A4BEC) walk the same
    arrays from the same base, attacker first.
  - The comparison keeps the earlier of equals (0x5A62F7–0x5A62FA), so on a tie the attacker's
    stack wins in the original and the defender's in the engine.
  - Batch 2a does not tell: R1 has no tie, R2's tie is between two defender stacks. No test tells
    either: `tests/spells.rs:386–400` announces "the defender's stacks come first" but its tie is
    between two attacker stacks (ids 0 and 1; the defender's stack 3 is 176 pixels away).
  - Scenario: attacker's hero casts at defender stack A on (8,5); attacker stack on (6,5), defender
    stack on (10,5), all one-hex, 88 pixels each. Original: A, the attacker's own stack, then the
    defender's. Engine: A, the defender's, then the attacker's (the halves are swapped).
  - The same argument has a second, smaller consequence: F-r1-6.
- fix or test sketch: swap the order in `next_in_chain`; correct the summary, §7.8, the survey, the
  map entry and the comment at `spells.rs:386`. Test (needs-run, in `tests/spells.rs`, expects the
  original's order and fails today):
  ```rust
  let mut s = arena(&r, [Some(mage(&r, &["chain"])), None],
      &[("tank", 5, 6, 5)], &[("tank", 5, 8, 5), ("tank", 5, 10, 5)], 1);
  // Ids: 0 (6,5) attacker; 1 (8,5), 2 (10,5) defender; row 5: x = 264, 352, 440.
  assert_eq!(chain(&mut s, 1), [(1, 40), (0, 20), (2, 10)]); // today: [(1, 40), (2, 20), (0, 10)]
  ```

### F-r1-2: Under Q-0005 the original stores each caster product as an `f32` before truncating; the engine floors the exact product, one point low for some Sorcery specialists

- rules: BAT-MAG-004, HER-BAT-012; quirk Q-0005 (ledger text, flag section "Spells")
- kind: behaviour-bug (the faithful profile only; strict-fixed is right)
- severity: medium (five heroes: Zydar, Sandro, Malekith, Gird, Styg; some levels and values; one
  damage point)
- confidence: high for the reading (the stores are plain in the code and §7.8 itself records them);
  never observed (batch 2a had no Sorcery specialist). Experiment: Sandro at level 12 with Advanced
  Sorcery, power 4, no magic school, Magic Arrow at a stack without resistance or reduction: 58
  (reading, and exact rates) or 57 (engine, faithful).
- evidence:
  - Executable, 0x4E5760: the value is stored as an `f32` (0x4E5774–0x4E577C); the Orb product is
    stored as an `f32` (0x4E58A2–0x4E58AB); `(s + 1) × v` is stored as an `f32` (0x4E5901–0x4E5912);
    only then is it truncated (0x4E5917–0x4E591A). The specialty bonus is added to the
    *untruncated* `f32`, the sum stored as an `f32` and truncated (0x4E5932–0x4E5944).
    `installation.md` §7.8 step 1 says "both products are stored as `f32`, not truncated".
  - A store rounds to the nearest 24-bit value, so a product a hair below a whole number becomes
    that number. Engine: `magic.rs:1710–1711` multiplies the exact rates and divides with a floor;
    BAT-MAG-004: "the product kept fractional and rounded down once"; ledger Q-0005: "their product
    is rounded down once".
  - Plain Sorcery never differs: f32(0.05), f32(0.10), f32(0.15) lie above the decimals; brute
    force over values 1–40 000, with and without an Orb: no difference (`stage1.py`). A Sorcery
    specialist's grown rate can be an `f32` *below* its decimal: level 12, Advanced:
    `f32((12 × 0.05f + 1) × 0.1f)` = 0x3E23D70A = 0.15999999642 (it equals f32(0.16), so the seam
    carries `Rate::bp(1600)` with no value of its own). Magic Arrow at no mastery and power 4:
    value 50; original 1.15999999642 × 50 = 57.99999982 → stored 58.0 → 58; engine 57; exact rates
    58. More (`stage1b.py`, levels 1–30, powers 1–30, the twelve damage spells): levels 2, 6, 9,
    12, 15, 18, 21, 24, 27 with Basic or Advanced Sorcery, e.g. level 15 Advanced, power 3: 47
    against 46; level 2 Advanced, power 9: 111 against 110.
  - `tests/magic.rs:504–526` asserts 244 for the rate f32(0.225) at value 200 and calls that the
    single-precision result; by the executable's arithmetic it is 245 too (244.9999988 is stored as
    245.0). OPEN-QUESTIONS Q45's worked example has the same slip. With the stores modelled, the
    rate's own single value (0x3E666667 against 0x3E666666) no longer changes this example.
- fix or test sketch: with the flag, compute stage 1 as the code does, with `X87`:
  `v = int(value).round(24)`; with a school bonus `v = v.times(1 + school).round(24)`;
  `v = (ONE.plus(all)).times(v).round(24)`; `t = v.trunc()` (the specialty added to `v`, not `t`,
  then `round(24)` and `trunc`, which only matters at absurd values). Test (needs-run,
  `tests/magic.rs`, fails today with 57):
  ```rust
  let r = ruleset_with("t:single", |_| {});
  let mut h = mage(&r, &["bolt"]);
  h.power = 4; // 4 × 10 + 10 = 50
  h.effects = vec![BattleEffect::SpellDamage { scope: SpellScope::All, rate: Rate::bp(1600) }];
  let mut s = led(&r, [Some(h), None], &[(0, "fast", 1)], &[(0, "titan", 5)]);
  assert_eq!(spell_damage(&act(&mut s, &r, cast(&r, "bolt", 1)).unwrap())[0].1, 58);
  ```
  and the 244 at `magic.rs:524` becomes 245; reword the ledger and BAT-MAG-004.

### F-r1-3: A spell without a target is refused by the original unless some stack would be affected with a chance above zero; `no_target` asks only for reach, and leaves out hypnotized stacks the original counts

- rules: BAT-MAG-003 (also BAT-MAG-026 for the hypnotized part)
- kind: behaviour-bug and rule-text
- severity: low (needs a hero whose resistance empties every share, or only hypnotized stacks in reach)
- confidence: high for the code path (the hero's book), medium that nothing else gates the cast
- evidence:
  - Executable, 0x59ED5A–0x59EE03: for every book spell of that handler (15–19, 24–39, 41–58,
    60–62: single-target spells, mass versions and the three battle-wide spells), before anything
    is paid, every hex outside the side columns is tried with the target test 0x5A39C0, which for
    these spells wants a stack on the hex whose factor is above zero (0x5A39F8–0x5A3A33; the
    factor includes the hero's resistance, 0x44A7F5); if no hex passes: a message, and the cast
    mode (+0x3C, cleared at 0x59ECA3) stays 0. Only after a hex passed is the spell armed
    (0x59EE08–0x59EE1C); a spell that takes no target stack at this mastery then returns without a
    cursor (0x59EE1F–0x59EE28; the caller's dispatch of the armed spell was not read). The test
    does not look at Hypnotize (the mass handler skips such stacks later).
  - Engine: `no_target` (`magic.rs:1645–1660`) asks `reachable` only. `aimable` (`:880–888`) and
    the opening spells (`:952–955`, "in_reach && accepts") do ask `accepts`, i.e. the share above
    zero; the battle-wide and mass casts do not. `reachable` also drops hypnotized stacks a mass
    version passes over (`:783–785`), which the original's test counts.
  - Scenario: defender's hero with 100 % resistance (the test hero `warded` of
    `tests/spells.rs:437`), attacker casts a mass negative spell: original refuses ("will affect
    no one"), engine casts, takes the mana and the round's cast, every stack resists.
- fix or test sketch: `no_target` for the non-summon, non-obstacle branch:
  `stack_ids(state).any(|s| in_reach(…) && accepts(…))`; say so in the summary.

### F-r1-4: The Unicorns' aura protects by the Unicorn's controller against the *real side* of the protected stack; the engine compares the two controllers

- rules: BAT-MAG-008 (and BAT-MAG-026; `installation.md` §7.14 "the adjacent Unicorns of the same controller")
- kind: behaviour-bug (source: the note is right only for stacks that are not hypnotized)
- severity: low (a hypnotized stack beside Unicorns, struck by a neutral or negative spell)
- confidence: medium to high (one function, both branches agree); no other source speaks of it
- evidence:
  - Executable, 0x43EA70 (rebuilds a stack's lists +0x514/+0x524; the factor multiplies by 0.8
    when +0x524 is not empty, 0x5A8599–0x5A85C3). Branch "the neighbour is a Unicorn" (0x43EAEE–
    0x43EB13): the Unicorn's side, flipped if the Unicorn is hypnotized, is compared with the
    rebuilt stack's +0xF4 (its real side). Branch "the rebuilt stack is a Unicorn" (0x43EB9C–
    0x43EBC3): its side, flipped if hypnotized, against the neighbour's +0xF4. Neither reads the
    protected stack's Hypnotize.
  - Engine: `magic.rs:1936–1940` `n.controller() == controller` with `controller =
    stack.controller()`.
  - Scenario: side A's Unicorns stand beside A's Swordsmen; B hypnotizes the Swordsmen and casts
    Fireball on them. Original: the aura still applies (Unicorn's controller A = Swordsmen's real
    side A), chance 80 %. Engine: no aura, 100 %. With B's Unicorns beside them it is the other
    way round.
- fix or test sketch: compare `n.controller()` with `stack.side`; a test with a hypnotized stack
  beside a Unicorn of each side (`affect_chance` 80 and 100).

### F-r1-5: `spell_damage` can overflow `u128` in the single-precision profile (a panic)

- rules: BAT-MAG-004 (the function also serves BAT-MAG-018, BAT-MAG-028)
- kind: robustness
- severity: medium (a panic, but only from a hero given directly at the seam or from extreme
  synthetic content)
- confidence: high by arithmetic; needs-run to see the panic
- evidence: `magic.rs:1710` `(one + school) * (one + all)` and `:1711` `u128::from(value) *
  factor`, with `one = 2^40` under Q-0005, so `factor ≥ 2^80`.
  - (a) `hero::check` (`hero.rs:289–452`) bounds `damage.reduction` and the Eagle Eye chance but
    no `SpellDamage` rate. A `BattleHero` with `SpellDamage { All }` and `SpellDamage { School }`
    whose `Rate.single` is a huge `f32` (e.g. bits 0x7F00_0000) is accepted by `BattleState::new`;
    `add_rates` keeps 2^24 for each, `hero_rate` returns `u64::MAX` for each, and the first product
    is (2^40 + 2^64 − 1)² > `u128::MAX`: the first damage spell of that school panics. The
    derivation cannot produce this (it caps at 65 535 basis points).
  - (b) Without any bonus the second product overflows for `value ≥ 2^48`. A hero's or a
    creature's cast stays below (u32 power × u16 factor + u16 effect < 2^48), but Fire Shield's
    reflection (`magic.rs:1841`) passes a share of a strike capped only by the target's total hit
    points (u32 × u32): by my arithmetic (not run) `u32::MAX` creatures of damage 65 535 with
    + 300 % attack against a 30 % shield on a stack with enough hit points give about 3.4 × 10^14,
    above 2^48 ≈ 2.8 × 10^14. Out of reach for the harvested packs.
- fix or test sketch: bound the rates in `hero::check` (as the reduction is) and compute `t` with
  `WideUint` or a saturating two-step division. Test (needs-run, `tests/magic.rs`):
  ```rust
  let r = ruleset_with("t:single", |_| {});
  let huge = Rate { bp: 0, single: core::num::NonZeroU32::new(0x7f00_0000) };
  let mut h = mage(&r, &["bolt"]);
  h.effects = vec![
      BattleEffect::SpellDamage { scope: SpellScope::All, rate: huge },
      BattleEffect::SpellDamage { scope: SpellScope::School(SchoolId::from_raw(0)), rate: huge },
  ];
  let mut s = led(&r, [Some(h), None], &[(0, "fast", 1)], &[(0, "titan", 5)]);
  let _ = act(&mut s, &r, cast(&r, "bolt", 1)); // must not panic
  ```

### F-r1-6: The chain tests each later candidate as if cast by the side opposite to the candidate's; the engine passes the real caster (it shows only with an Orb of Vulnerability)

- rules: BAT-MAG-006, BAT-MAG-066
- kind: question
- severity: low
- confidence: medium (the argument is plain at 0x5A6269; what follows from it was traced only to
  the casting hero's Orb test at 0x44A746, and BAT-Q21 showed that readings about that Orb need an
  experiment)
- evidence: 0x5A61F0 passes 1 as the caster's side for side 0's records and 0 for side 1's (see
  F-r1-1), so the "casting hero" of the effect test is the hero opposite to the candidate, and the
  controlling hero is the candidate's own. Both heroes' Orbs of Vulnerability are therefore asked
  for every candidate. Engine: `next_in_chain` → `affects_stack(state, ruleset, caster, …)` →
  `Guard::of(state, caster, s)` asks the caster's and the controller's hero, which are the same
  hero for the caster's own stacks. Scenario: the defender's hero wears the Orb; the attacker's
  Chain Lightning reaches the attacker's own Black Dragons. Original (by this reading): they can be
  jumped to and struck. Engine: they stay immune.
- fix or test sketch: record the argument in §7.8; decide with BAT-MAG-066 whether to follow it.

### F-r1-7: BAT-MAG-004's "(hero level div the stack's creature level, at least 1)" reads as a quotient of at least 1; the code and batch 2a R4 have a quotient of 0

- rules: BAT-MAG-004
- kind: rule-text
- severity: low
- confidence: high
- evidence: summary: "a spell specialist's ⌈per-level rate × (hero level div the stack's creature
  level, at least 1) × t⌉". Code: `magic.rs:1759–1762` puts the floor of 1 on the creature's level,
  `:1774` divides without a floor. R4: Alagar at level 1 deals 115 to Wolf Riders (level 2), i.e.
  no bonus; a quotient of at least 1 would give 119. The executable divides by the 0-based level
  + 1 with no floor on the quotient (0x4E608D–0x4E609D). HER-BAT-012's text is right.
- fix or test sketch: "(hero level div the stack's creature level, which is at least 1)".

### F-r1-8: Summaries describe delivered work as still to come

- rules: BAT-MAG-005, BAT-MAG-001
- kind: rule-text
- severity: low
- confidence: high
- evidence: BAT-MAG-005: "Power of the Dragon Father, the Pendants and the Orb of Vulnerability
  join with T8", "the Sphere of Permanence … joins with the Pendants in E04b T8", "a SoD pack
  override once it is castable, E04b T8", "and later Berserk, Hypnotize, …": all delivered
  (`affects`, `magic.rs:704–739`, BAT-MAG-063 to 066; `packs/sod/pack.toml` overrides). BAT-MAG-001:
  "the damage spells since E04b T2d …, other effects with their tasks": every effect kind is
  implemented now (`implemented`, `magic.rs:582–587`; only spells without effects are refused).
- fix or test sketch: state the present rule; move the history to the log.

### F-r1-9: BAT-MAG-001's observation source claims "once per round" from R6, which never tries a second cast

- rules: BAT-MAG-001
- kind: source
- severity: low
- confidence: high
- evidence: catalog: "R6 …: Calid with Basic Artillery casts Magic Arrow in his Ballista's turn (16
  to 11 spell points), and once per round". `experiments/results/E04b-batch-2a.md` R6 records one
  cast per round in three rounds and the answer "can R6 cast in the Ballista's turn: yes"; no
  attempt at a second cast in a round. The limit itself is well supported otherwise: wiki, VCMI,
  and the cast mark +0x54B4 (set 0x59FFF6, tested 0x59E948 and 0x59EC76, cleared at 0x4760EE).
- fix or test sketch: drop "and once per round" from the observation's ref, or cite the code
  reading for it.

### F-r1-10: Small test gaps

- rules: BAT-MAG-001, BAT-MAG-002, BAT-MAG-006
- kind: test-gap
- severity: low
- confidence: high
- evidence: (a) no test casts with mana equal to the cost (`magic.rs:632` `<`): the casts in
  `tests/magic.rs` start from 15 and 10 spell points at cost 5, the refusals have less than the
  cost (3 against 4, 6 against 7 in `tests/mana.rs`); (b) batch
  2a R1 and R2 are reproduced by geometry in the synthetic pack only; the harvested replay
  (`h3-pack/tests/harvested_spell_damage.rs`) covers R3 to R5, so the packs' chain data (power 40,
  effect 25, targets 4/4/5/5, factor 0.5) is not held against the observed counts 35/70/85 and
  35/69/86; (c) the tie between sides, see F-r1-1; (d) R6's cast in the Ballista's turn (BAT-MAG-001)
  is reproduced only in the synthetic pack (`tests/magic.rs:316–360`); the harvested replay of R6
  (`h3-pack/tests/harvested_mana.rs`) has no Ballista and another order of casts.
- fix or test sketch: (a) a third cast in `a_hero_casts_once_per_round…` (15 → 0); (b) a harvested
  replay of R1 with Caitlin, power 1, three Peasant stacks on the batch's hexes.

### F-r1-11: VCMI treats a chain's resisting later target differently from the rule in three points; the catalog's VCMI ref reads as agreement

- rules: BAT-MAG-006
- kind: source
- severity: low
- confidence: high (VCMI as read; the executable re-read supports the rule's side)
- evidence: catalog, VCMI source of BAT-MAG-006: "a resisting later target is skipped, the next by
  the closest hex"; its note names only the hex distance and the factor. VCMI 1.7.5: (1) whether a
  unit resists is drawn once per cast (`BattleSpellMechanics.cpp` L455–462, read back by
  `wouldResist`, `UnitEffect.cpp` L227), not once per jump; (2) a skipped unit's hexes leave the
  candidates (L246–248), so no later jump strikes it; (3) the next target is the one closest to
  the *skipped* unit (L253). The rule: each candidate rolls at every jump, a stack that resisted
  one jump may be struck by a later one, and the distance is from the last stack struck. The
  executable agrees with the rule: in 0x5A61F0 a candidate that fails its roll is just passed
  (0x5A6272–0x5A6295), no mark is written for it; marks are written only for struck stacks
  (0x5A659A), and the distance is taken from the stack the handler passes in, the last one struck
  (0x5A6594–0x5A65A2). Effect in play: Dwarves beside the first target of a four-target chain are
  struck with about 99 % (three rolls at 80 %) by the rule, 80 % in VCMI. `docs/04` §2 asks that
  such a contradiction is named with the source (and, where it counts, queued as an experiment);
  the entry reads as if VCMI agreed.
- fix or test sketch: extend the VCMI note of BAT-MAG-006; an experiment candidate if wanted
  (Battle Dwarves beside the first target, the other stacks far: how often they are struck).

### F-r1-12: The area spells' handler and Chain Lightning's fetch the acting side's hero for the damage bonuses whatever the caster; a creature's area spell or chain gets them in the original, none in the engine

- rules: found in the handlers of BAT-MAG-006 and BAT-MAG-007; the statement it contradicts is in
  BAT-MAG-048 and BAT-MAG-049 ("no hero bonuses") and `installation.md` §7.21 ("A creature's cast
  has no hero bonuses"); BAT-MAG-004 for the bonuses
- kind: question (a reading against VCMI; the wiki not checked: `docs/04` §2 makes that a conflict
  of sources for the user) with a source defect (§7.21 generalises from one path)
- severity: medium (Faerie Dragons led by a hero with Sorcery, an Orb or a specialty in the spell:
  five of their eight spells, 46 of 100 draws by weight)
- confidence: high for the code (three handlers show the same pattern, no branch on the caster
  kind in them); what settles it: the wiki's Faerie Dragon and Sorcery pages, or an experiment:
  one Faerie Dragon under a hero wearing the Orb of Tempestuous Fire: its Fireball deals
  5 × 10 + 30 = 80 (engine, VCMI) or 120 (reading); its Magic Arrow 70 either way.
- evidence:
  - The cast routine keeps a "casting hero" that is null for caster kind 1 (0x59FE63–0x59FE7B),
    and the handlers written inside it pass that one to the damage modifiers: Magic Arrow
    (0x5A09DC), Ice Bolt (0x5A0A6B), Lightning Bolt and Titan's (0x5A0B49), Implosion (0x5A0B9E),
    Death Ripple (0x5A0D3F), Destroy Undead (0x5A0F3C). So a creature's bolt has no hero bonuses,
    as §7.21 says.
  - The three handlers that are functions of their own do not get the caster kind or that hero:
    the area handler 0x5A4970 (arguments hex, spell, mastery, power) reads the hero of the acting
    side (+0x132C0 into +0x53CC, 0x5A4A05–0x5A4A1F) and passes it as the casting hero for every
    stack struck (0x5A4AB0–0x5A4AD2); Chain Lightning's 0x5A6360 does the same for every target
    (0x5A6557–0x5A656E); Armageddon's 0x5A4BC0 too (0x5A4C5A–0x5A4C90; no creature casts it). The
    modifiers 0x5A78E0 apply 0x4E5760 whenever that hero is not null (0x5A790D–0x5A7916): the Orb,
    Sorcery and, with the target, the specialty.
  - During a creature's cast the acting side is its controller's (§7.14, 0x465389), so a Faerie
    Dragon's Fireball, Frost Ring, Inferno, Meteor Shower and Chain Lightning take its hero's
    bonuses; its Ice Bolt, Lightning Bolt and Magic Arrow do not.
  - Engine: `plan_creature_cast` sets `bonuses: SpellBonuses::default()` for every creature cast
    (`magic.rs:1467`); no test gives the hero of a casting creature a spell-damage bonus
    (`tests/casters.rs` has no `BattleEffect::SpellDamage`), so nothing pins either behaviour.
  - VCMI 1.7.5: a unit caster has no such bonuses (`lib/battle/CUnitState.cpp` L423–427 returns
    the base value), so VCMI contradicts the reading.
- fix or test sketch: none before the sources are compared; then either `plan_creature_cast`
  takes the controller's hero's bonuses when the spell has an area or a chain, or §7.21 records
  why not. An OPEN-QUESTIONS entry with the experiment above.

## Wiki checks wanted

1. Chain Lightning: what the page says about equal distances beyond "randomly chosen", and
   whether it says anything about own stacks being preferred (F-r1-1).
2. Sorcery and Hero specialty (Sorcery specialists): any worked number for a specialist's damage
   that could tell 58 from 57 in the case of F-r1-2.
3. Unicorn, "Aura of Magic Resistance": whether it mentions hypnotized stacks (F-r1-4).
4. Combat or Spell book: whether a battle-wide or mass spell that would affect nobody can be cast
   (F-r1-3).
5. Faerie Dragon and Sorcery (also the Orbs' pages): whether the hero's Sorcery, Orbs or spell
   specialty change the damage of the Faerie Dragons' spells, and of which spells (F-r1-12).

## Not reviewed, left open

- The book dialog 0x59BDF0 was not read: where the original tests the mana against the cost and
  which spells it offers. "Less mana than the cost cannot cast" rests on the wiki and VCMI;
  0x4D92D0 only shows that the spell points never go below zero.
- Why the tactics phase allows no cast in the original was not read (wiki and VCMI carry it).
- 0x4E7170 (the stack on a hex), 0x523DF0 (neighbour hex by direction) and 0x5A7BB0 (protections,
  BAT-MAG-013) were taken as described; 0x5A78E0 was read to 0x5A7AA2 (the rest writes the log).
- The hex records' screen coordinates (+0x1C4, +0x1C6) are filled at run time; the 44 × 42 grid
  rests on batch 2a. Q43 (the sprite lift in the chain's distance) stays open as recorded; the
  sign of the two-hex shift follows the stack's +0x44, read as in §7.19, not observed.
- BAT-MAG-001 with BAT-MAG-059: an active Phoenix killed by its own hero's cast and reborn stays
  the active stack in the engine (`command.rs:458`, alive again, with its acted mark set by the
  rebirth). What the original does then was not read.
- The original rolls `random(1, 100)` also when the chance is 0 or 100 and walks an area's stacks
  from the last list entry to the first; the engine draws nothing at 0 and 100 and goes by stack
  id. Only the random stream differs, no outcome.
- The wiki's statements were taken from the catalog and the survey (no access).
- Security: nothing found in this slice. `Command::Cast` takes the spell from the actor's own
  book, every stack id and hex of the payload is checked before it is indexed (`first_target`,
  `second_target`, `raise::victim`), a refused cast changes nothing, and the public `SpellDamage`
  values match what the original's log shows both players.
