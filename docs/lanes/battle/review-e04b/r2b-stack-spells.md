# Review r2b-stack-spells: BAT-MAG-018 to BAT-MAG-024, quirks Q-0009, Q-0010, Q-0011

Reviewer `r2b-stack-spells`, 2026-10-11. Nothing in the repository was changed, nothing was built
or run. Executable readings are in my own words with addresses (`scripts/exe.py fn|switch|callers|data`).

## Coverage

| rule | text↔code | sources checked (exe addresses re-read; VCMI files; batch rows) | tests | verdict |
|---|---|---|---|---|
| BAT-MAG-018 Fire Shield | every clause traced (`combat::strike`, `attack`, `reflect`, `magic::reflection`, `damage::Strike::stage_one`); the summary is silent on what follows when the striker dies of the reflection | exe: 0x422440 (whole), 0x443130, 0x443F40 (0x443F81–0x443FC0: stage 1, the clone's 1, the hit-point cap), 0x441610 (0x4419BE–0x441A79 the two strikes, 0x441B70–0x441CA7 the order: strike spell, striker effects, reflection), 0x440310 (the sum), 0x4409C0, 0x441CB0 (retaliation conditions), 0x444090, 0x5A78E0 (head), 0x4E5760; VCMI `server/battles/BattleActionProcessor.cpp` L1183–1233, L1602–1616; batch 2b R3 (60 of a first stage of 300; 10 Naga Queens, top one at 50: correct) | `tests/rage.rs` L105–249, `tests/created.rs` L638, `tests/rules.rs` L609–614 | F-2 (question), F-5, F-6; else ok |
| BAT-MAG-019 Frenzy | traced (`effects::apply` OwnTurn, `effects::own_turn`, `count_down`, `turn::activate`, `combat::strike_inputs`, `frenzy_defense`, `damage::frenzied`) | exe: 0x4448F0 (0x4448FD–0x444995 duration 1 or 2 and the re-cast return; case 56 at 0x444D5B), 0x446E30 (flag +0x4F0, countdown 0x4470D1–0x447101), 0x447120 (0x4471C6 skips spell 56), 0x465080 (turn start only outside the wait phase), 0x442410 (whole), 0x442590 (whole); VCMI `config/spells/timed.json` L1112–1160, `lib/bonuses/BonusEnum.h` L265, `lib/gameState/GameStatePackVisitor.cpp` L1314 | `tests/rage.rs` L267–354 | F-1 (through Q-0009), F-4, F-6 |
| BAT-MAG-020 Spell specialists | traced (`magic::Specialist`, `specialist_bonus`, `effects::apply`, `combat::specialist_terms`, `Strike::blessed`, `magic::affect`, `h3_content::SpecialistBonus`) | exe: 0x4E5FF0 (whole; byte table 0x4E60E8, default: the hero's level divided by c + 1 in whole numbers, × t, × the double 0.03, rounded up), tables 0x63EAA8 and 0x63EAC4 (data), callers list (16 sites), 0x4448F0 cases 43–46, 48, 51, 53 (0x444B11–0x444CFC), 0x4436F0–0x44377D (Bless, stage 1), 0x4424A6–0x4424FE (Slayer), 0x4465DD–0x44660D (Cure); VCMI `lib/spells/effects/Timed.cpp` L144–185, `lib/battle/DamageCalculator.cpp` L156–186, `config/heroes/{castle,conflux,rampart,tower}.json` | `tests/specialist.rs` (whole) | ok (F-7 note on an unchecked addition) |
| BAT-MAG-021 Anti-Magic | traced (`effects::apply` L412–421, `magic::kept_off`, `affects_stack`, `stack_target`, `raise.rs` L133/236) | exe: 0x444A9E–0x444AE4 (the cast's loop), 0x444510 (head: only spell 47 never ends), 0x5A8090 (0x5A8090–0x5A82F8: Cursed Ground, Recanter's Cloak, Dispel, Dispel Helpful, then Anti-Magic at 0x5A8240; switch case 34 at 0x5A83FF: no clone), 0x447A80 → 0x5A3C80 → 0x5A8090; VCMI `config/spells/timed.json` L262–322 | `tests/defences.rs` L85–197, `tests/created.rs` L638, `tests/raise.rs` L420 | F-3; else ok |
| BAT-MAG-022 Magic Mirror | traced (`magic::mirror`, `mirror_chance`, `reach_stacks`, `living`, `affects_stack`, `admits`, `resists`) | exe: 0x59EC50 (0x59EE1F–0x59EF96: target stack, not Dispel, other side by +0xF4, alive, random(1, 100) ≤ chance, the sampler loop skipping dead stacks and creature 149, the second hex), 0x4487F0, 0x50C6E0/0x50C740 (uniform without replacement), 0x59FE30 (0x5A0275–0x5A0318: reflected flag, creature casts skip the roll), 0x5A8090 (0x5A82F8–0x5A8345: the polarity tests skipped for a reflected cast; case 60), 0x59E060; the computer player's path: 0x43C800 (0x43CACB–0x43CAF5 writes the candidate's own second hex), 0x43A76D–0x43A7B1 (only scales the value), the dispatcher 0x479036–0x479056 (passes the action's second hex on): no draw there, as §7.13 says; VCMI `lib/spells/BattleSpellMechanics.cpp` L482–550 | `tests/defences.rs` L199–314, `tests/casters.rs` L648, `tests/strike.rs` L256 | ok (F-6: no war machine as the new target) |
| BAT-MAG-023 Forgetfulness | traced (`combat::striking_count`, `can_shoot`, `effects::recompute` L745–751, `magic::target_at`, `skips_hypnotized`) | exe: 0x443160 (0x443166–0x4431A7: `max(1, n / 2)` for a stack with the shooter bit while spell 61 is active, at any mastery), 0x4429B2 and the inline copies 0x447D92, 0x445556, 0x448A47, 0x448C90 (all: stored mastery ≥ 2), 0x445036 (the mastery stored only at a first application), 0x44A614 (only shooters), 0x59E060 (flag 0x20: a target only up to Basic), 0x5A66D0 (the mass handler: hypnotized stacks skipped, every other stack tested and rolled); VCMI `lib/battle/DamageCalculator.cpp` L421–441, `config/spells/timed.json` L1323–1388 | `tests/actions.rs` L111–231 | ok |
| BAT-MAG-024 Blind | traced (`turn::activate`, `next_active`, `passed_without_turns`, `effects::damaged`, `retaliation_share`, `combat::melee`, `retaliate`, `Strike` stage 2 and exact) | exe: 0x444EB3 (factor stored for spell 62 only), 0x465080 (whole: the pick, turn start, re-pick without the done bit, no morale check), 0x446E30, 0x444090 (ends 62, 70, 74 on any damage), 0x441D9C–0x441E6C (marks set before the strike, cleared after), 0x441C85–0x441CA7 (no retaliation at a factor of 0), 0x443CD5–0x443D8B (stage 2 reads the striker's marks), 0x44A59F–0x44A5D8 (Pendant, creatures 70/71, undead: absolute); VCMI `config/spells/timed.json` L1390–1450 (not cited by the rule) | `tests/actions.rs` L233–341, `tests/morale.rs` L477/547, `tests/strike.rs` L781 | ok (F-5: no VCMI or wiki source cited) |
| BAT-DMG-006 Defense reduction (outside the slice; Q-0009 and BAT-MAG-019 rest on it) | faithful variant `damage::reduced_defense` against the executable | exe: 0x442590 (0x4425C0–0x4425F5); VCMI `lib/battle/DamageCalculator.cpp` L198–208 | `tests/abilities.rs` L463–481 | F-1 |
| Q-0009 Frenzy counts a Behemoth's reduction | both variants in `combat::frenzy_defense`; test `rage.rs` L293–302 (30 without, 26 with: each fails for the other) | direction confirmed by my re-read: 0x442410 calls 0x442590 at 0x442514 with the struck stack and flag 0; 0x442590 applies the share whenever the other stack is creature 96 or 97. The arithmetic of that share is misread (F-1). Class fix-optional fits (HotA fixed it) | only with Q-0005 off; the shipped faithful pair (Q-0005 and Q-0009) is untested | F-1 |
| Q-0010 Fire Shield's base | both variants in `combat::strike` L158–166; test `rage.rs` L117–130 (3 without, 4 with) | confirmed three ways: batch 2b R3, my re-read (0x443F81–0x443FC0), and VCMI's own remark at `BattleActionProcessor.cpp` L1613 that the base should be the damage "with bonus but without penalties". Class fix-optional (user) fits | ok | ok |
| Q-0011 Forgetfulness mass from Advanced | both variants in `SpellMagic::target_at`; test `actions.rs` L193–231 | confirmed by my re-read of 0x59E060 and by the wiki quote in the catalog; VCMI has the fixed variant (Expert). Class fix-optional fits | ok | ok |

## Findings

### F-r2b-stack-spells-1: under Q-0005 the Behemoths' defense reduction is one point too weak for every positive defense: the executable truncates the defense left, not the share taken

- rules: BAT-DMG-006 (faithful variant), BAT-MAG-019 and quirk Q-0009 (the frenzied striker's own
  defense goes through the same function), `installation.md` §7.11 "Behemoths", OPEN-QUESTIONS Q19's
  amendment
- kind: behaviour-bug (and source: the reading in §7.11 is wrong; "VCMI not followed" was the right
  formula for the original)
- severity: high (the default profile `faithful` has Q-0005 on: every strike and retaliation of a
  Behemoth or Ancient Behemoth on a stack with defense above 0 sees one defense point too many, 5 %
  or 2.5 % of the base damage; the same point in a frenzied striker's attack against Behemoths)
- confidence: high. Raised further by Q19's own experiment (Behemoths at a stack with defense 10:
  the original must treat it as 5, the engine's faithful profile has 6).
- evidence:
  - The executable, 0x4425C8–0x4425F5 in 0x442590 (Behemoth; the Ancient Behemoth's twin at
    0x4425DE): the defense D is loaded as an integer, stored and reloaded as a single, multiplied by
    the single constant at 0x63B8B0 (0.4000000059604645; 0x63B8AC = 0.800000011920929 for creature
    97); the instruction at 0x4425ED (a reverse subtraction with the single D as its memory
    operand: operand minus register) then subtracts that product from the single D, and only after
    it the truncation routine is called (0x4425F0) and its result becomes the defense (0x4425F5).
    So the defense left is `trunc(f32(D) − f32(D) × f32(p))`; the share taken is never truncated
    on its own. (All values are exact at the x87's 53 bits: 24-bit operands.)
    `installation.md` L605–608 says `D − trunc(f32(D) × f32(0.4))` ("the share taken is truncated,
    so the defense left rounds up (D = 7: 5 and 2)"); `docs/exe/map.toml` L59 repeats it
    ("truncated, rounds up").
  - Values (exact rational arithmetic on the two single constants):

    | D | original 40 % | engine faithful | engine strict-fixed (Q19) | original 80 % | engine faithful | strict-fixed |
    |---|---|---|---|---|---|---|
    | 1 | 0 | 1 | 0 | 0 | 1 | 0 |
    | 5 | 2 | 3 | 3 | 0 | 1 | 1 |
    | 7 | 4 | 5 | 4 | 1 | 2 | 1 |
    | 10 | 5 | 6 | 6 | 1 | 2 | 2 |
    | 12 | 7 | 8 | 7 | 2 | 3 | 2 |
    | 17 | 10 | 11 | 10 | 3 | 4 | 3 |
    | 20 | 11 | 12 | 12 | 3 | 4 | 4 |

    For every D from 1 to 300 the engine's staged value is exactly the original's plus 1. The
    original equals Q19's floor except where 0.6 × D (0.2 × D) is whole, where the constant's excess
    over 0.4 (0.8) costs one more point: a Q-0005-class effect, as one would expect.
  - VCMI 1.7.5 `lib/battle/DamageCalculator.cpp` L198–208 (`getTargetDefenseIgnored`): ignored =
    `floor(p × D) + 1`, capped at D. That is identical to my reading for every D from 1 to 300 at
    both percentages: an independent source for the reading. The "+ 1" that Q19 and the catalog
    (`battle.toml` L828: "not followed") set aside is the original's: the defense left is
    truncated (so the share rounds up), and where p × D is whole the constant's single-precision
    excess takes the extra point.
  - Code: `crates/h3-battle/src/damage.rs` L304–321 `reduced_defense(…, staged = true)` computes
    `defense − sign × trunc(f32(|defense|) × f32(percent / 100))`; used by `combat.rs` L224–227
    (the strike) and L339–349 (`frenzy_defense`, Q-0009). Tests pin the wrong values:
    `tests/abilities.rs` L471–481 (17 → 11 and 4, 7 → 5 and 2, 10 → 6, −3 → −2; the original: 10
    and 3, 4 and 1, 5, −1). The catalog summary (`battle.toml` L816: "7 leaves 5 … 2") and the
    T3b2 record (`E04b-log.md` L396–398) carry the misreading; the user's decision of 2026-10-07
    ("follow the executable under Q-0005") was taken on it.
  - Scenario: 10 Behemoths (attack 17) strike Swordsmen (defense 12, no hero). Original: defense 7,
    +50 %. Engine, profile `faithful`: defense 8, +45 %. Strict-fixed: 7. Frenzy at Basic on a stack
    with defense 10 attacking Behemoths: the original adds 5 to the attack, the engine (faithful) 6.
- fix or test sketch: in `reduced_defense`'s staged branch `let d = stored(|defense|, SINGLE_BITS);
  let left = d.minus(d.times(single(percent, 100))).trunc(); defense.signum() × left`. Tests:
  `reduced_defense(17, 40, true) == 10`, `(17, 80) == 3`, `(7, 40) == 4`, `(7, 80) == 1`,
  `(10, 40) == 5`, `(5, 80) == 0`, `(1, 40) == 0`, `(−3, 40) == −1`; a Q-0009 case in a profile with
  Q-0005 and Q-0009 both on (10 + 5 at Basic against a Behemoth). Correct §7.11, `map.toml` L59,
  BAT-DMG-006's summary and VCMI note (VCMI's formula is the original's), Q19's amendment
  (`OPEN-QUESTIONS.md` L142); name BAT-DMG-006 in the ledger's Q-0005 entry and in the rule's
  `quirks` field (both miss it today, though the Q-0005 flag selects the rule's variant); tell the
  user. Q19's experiment stays the cheap confirmation.

### F-r2b-stack-spells-2: in the original the struck stack still retaliates, and spends its retaliation, when the striker died of the Fire Shield reflection; the engine skips the retaliation and keeps it

- rules: BAT-MAG-018 (the summary only says "the striker may die of it"), BAT-RET-001 ("if both
  stacks are still alive", from the wiki and VCMI: before E04b's Fire Shield, and a clone that
  kills its original, the attacker could not die before the retaliation, so E03 never had to
  ask), BAT-RET-003
- kind: question (the executable against VCMI and the engine; no ledger entry, no user decision)
- severity: medium (a rare setup: the reflection must kill the whole attacking stack, and the
  retaliation matters only if the shielded stack is attacked again in the round)
- confidence: medium to high on the code path; an observation would settle it (below)
- evidence:
  - `installation.md` L589–592: "whether a striker killed by the reflection is still struck back
    was not read: the engine never retaliates against a dead stack". I read it. 0x441CB0 after the
    first strike (0x441DC5): the retaliation is skipped only if the struck stack has no creatures
    (0x441DD4), the attacker has the no-retaliation flag (0x441DDF), the struck stack is petrified
    (0x441DED), it has no retaliation left (0x441DF7) or the strike answered "no retaliation"
    (0x441E01). The attacker's count is not tested here, although the second strike tests it at
    0x441EAF. So 0x441E3D strikes back at the dead attacker and 0x441E58–0x441E5F take one
    retaliation. The strike routine itself tests only that the target pointer is not null
    (0x4419EA); the damage routine 0x444090 on a stack without creatures kills nothing
    (0x4440DE–0x4440F5). The reflection is applied by 0x4409C0 (called at 0x441C80) through
    0x444090 and changes nothing these tests read.
  - Engine: `combat.rs` L508–510 (`reflect`), then `melee` L746–748 calls `retaliate`, which returns
    at L793–802 because `!a.is_alive()`, before `retaliations_left` is touched.
  - VCMI 1.7.5 `server/battles/BattleActionProcessor.cpp` L324–334: the counterattack needs
    `stack->alive()` (the attacker): VCMI agrees with the engine, not with the code reading.
  - Scenario: one Archangel with 5 hit points left attacks 20 Efreet Sultans (first stage 90, 18
    reflected) and dies of the reflection; a second stack then attacks the Sultans in the same
    round. Original (by the code): the Sultans swung at the dead Archangel and do not strike the
    second attacker back. Engine: they do.
- fix or test sketch: decide with the user: a fix-optional quirk (the wasted retaliation) or the
  clean behaviour in every profile; either way the ledger needs the entry ("every known
  difference", `docs/04` §4) and BAT-MAG-018 the clause. Experiment: the scenario above in hot
  seat. Engine test once decided (`rage.rs` style): the Tank's hero with `SpellDamage { All, 10 000
  bp }`, a Fire Shield at mastery 3 on the Tank, 10 `fixed` with `max_hp = top_hp = 1` set in the
  state (stage 1 is 20, reflection 6 × 2 = 12 ≥ 10): assert the `Reflected` kills and the Tank's
  `retaliations_left` (1 today; 0 under the original's variant).

### F-r2b-stack-spells-3: a creature's draw (Master Genie) can put Anti-Magic on a clone; the original's clone test for Anti-Magic sits in the shared effect test, the engine has it only in the hero's aim

- rules: BAT-MAG-021 ("No clone may be aimed at"), BAT-MAG-050, BAT-MAG-038
- kind: behaviour-bug
- severity: medium (rare setup: a Master Genie, a cloned own stack, an enemy hero with a spell book)
- confidence: high for both code paths; `needs-run` only to show it in a test
- evidence:
  - Original: the clone test is case 34 of the per-spell switch in 0x5A8090 (0x5A83FF: a stack
    with the clone bit gives 0). Every caller of the effect test shares it; the Genie's usefulness
    test 0x447A80 calls 0x5A3C80 (0x447AAB), which is "0x5A8090 above 0". So Anti-Magic is never
    useful on a clone and is never drawn for one.
  - Engine: `magic.rs` L1074–1081 `grants_immunity` is used only in `stack_target` (L1103), the
    hero's aim. The creature path is `casters::useful` (`casters.rs` L452–453) → `magic::affectable`
    (L1334–1350) → `affects_stack` (L755–767): no clone test. `packs/base/pack.toml` L961: the Master
    Genie draws `base:anti_magic` (weight 1). So `helpful_target` accepts the clone, `useful_spells`
    lists Anti-Magic for it, and `draw_useful` can cast it: one more spell in the draw and a clone
    under Anti-Magic, neither possible in the original.
- fix or test sketch: move the test into `affects_stack` (or `affectable`). Test in
  `tests/casters.rs` with its helper: an arena with `jinn` and `serf` for the attacker and a
  defender hero with a spell book, `crate::created::clone(&mut s, &r, (StackId(1), 3), …)`, then
  `assert!(is_useful(&s, &r, "antimagic", 1)); assert!(!is_useful(&s, &r, "antimagic", <clone id>));`
  (the second fails today).

### F-r2b-stack-spells-4: BAT-MAG-019's VCMI source is misquoted: VCMI ends Frenzy at the stack's first strike, not at its next turn, so the duration rests on the code reading against VCMI

- rules: BAT-MAG-019
- kind: source
- severity: low (docs; the reading itself holds on a second, independent read)
- confidence: high
- evidence: `battle.toml` L2327 cites "config/spells/timed.json frenzy: … until the stack's next
  turn". VCMI 1.7.5 `config/spells/timed.json` L1130 has `"duration" : "UNTIL_ATTACK"`, which
  `lib/bonuses/BonusEnum.h` L265 defines as removed after each attack or counterattack and
  `lib/gameState/GameStatePackVisitor.cpp` L1314 removes from the attacker of every `BattleAttack`,
  a retaliation included. The survey (`E04b-magic.md` L40) has it right ("until it attacks
  (Frenzy)"). The two differ in play: in VCMI a frenzied stack stays frenzied, however many of its
  turns pass, until its first strike of any kind (an attack or a retaliation) and then loses the
  effect; in the original (0x4448F0: 1 for the
  active stack, else 2; 0x446E30 counts once per round at the stack's turn start; 0x447120 skips
  spell 56 at the round start: all re-read, all as §7.11 says) it lasts by the stack's turns
  whatever it strikes. By `docs/04` §2 ("Consensus before trust") a reading that VCMI contradicts
  is a conflict to record, with the wiki as the third voice; none is recorded.
- fix or test sketch: correct the reference (VCMI: until its first strike; not followed), cite the
  wiki's Frenzy page (see "Wiki checks wanted") and, if the wiki is silent, queue the experiment:
  Frenzy on a stack that defends twice, then attacks.

### F-r2b-stack-spells-5: citations and catalog fields of the slice that do not hold as written

- rules: BAT-MAG-018, BAT-MAG-019, BAT-MAG-022, BAT-MAG-024, Q-0010
- kind: source / rule-text
- severity: low
- confidence: high
- evidence:
  - BAT-MAG-018 (`battle.toml` L2300) and the ledger's Q-0010 (L150) cite
    `lib/battle/DamageCalculator.cpp` for VCMI's reflection. That file has no Fire Shield code; it is
    in `server/battles/BattleActionProcessor.cpp` L1602–1616 (base `min(defender's health, damage
    dealt)`, with the remark that it should be the damage "with bonus but without penalties", which
    supports Q-0010) and L1183–1233 (the struck stack's side's hero through `adjustRawDamage`).
  - The catalog's `quirks` field ("ids … that affect this rule", `docs/04` §3) is missing on
    BAT-MAG-018 (Q-0010, Q-0005) and BAT-MAG-019 (Q-0009, Q-0005) and lacks BAT-QUIRK-9 on
    BAT-MAG-022, although each summary names them; BAT-MAG-023 lists its quirk. `check-rules` cannot
    see the gap.
  - BAT-MAG-024 cites only the reading and the spell table's text. `docs/04` §2 asks for the wiki
    and VCMI beside a reading. VCMI agrees and can be cited: `config/spells/timed.json` L1390–1450
    (no turns until attacked; the attack reduced by 50 %, 75 % at Advanced; no retaliation at
    Expert).
- fix or test sketch: correct the two VCMI references, fill the `quirks` fields, add the VCMI
  source to BAT-MAG-024.

### F-r2b-stack-spells-6: clauses with an effect in play that no test pins

- rules: BAT-MAG-018, BAT-MAG-019, BAT-MAG-022, Q-0009
- kind: test-gap
- severity: low
- confidence: high (read `tests/rage.rs`, `defences.rs`, `actions.rs`, `specialist.rs`, the greps
  for the other files)
- evidence:
  1. Q-0009 together with Q-0005, the pair the faithful profile ships: `rage.rs` L293–302 uses
     `t:frenzy` (Q-0005 off). F-1 lives exactly there.
  2. Frenzy's count "at the stack's first turn start of a round (not after waiting)" is pinned only
     by direct calls of `effects::own_turn` (`rage.rs` L333–337). No play test has a frenzied stack
     wait: cast on the active stack (count 1), Wait, act in the wait phase still frenzied; or cast
     on a stack that has waited (count 2), which then strikes frenzied in the wait phase and again
     in its next turn. No rule test would notice a call of `own_turn` at the second activation
     (`actions.rs` L263–268 pins only that regeneration is not repeated).
  3. Magic Mirror's new target among war machines ("war machines included", the point where the
     engine follows the reading against VCMI's `getRandomUnit`): `defences.rs` L288–314 draws among
     three plain stacks, one dead.
  4. Batch 2b R3's 60 is not reproduced with its numbers: `rules.rs` L609–614 pins the strike (first
     stage 300, 150 dealt) and says 60 in a comment; the reflection is tested only with the
     synthetic 20 → 4 (`rage.rs` L117–130). A `verified` rule should carry its row.
  5. The striker killed by the reflection (F-2): no test either way.
- fix or test sketch: as named per item; for 4, `magic::reflection` on the R3 strike (first stage
  300, 1000 hit points, 20 %) equal to 60 and the Naga Queens' top creature at 50 of 110.

### F-r2b-stack-spells-7: two places that only data bounds

- rules: BAT-MAG-020 (Slayer's term), BAT-MAG-024 (the pick loop)
- kind: robustness
- severity: low (neither is reachable with the committed packs)
- confidence: medium (traced; not run)
- evidence:
  1. `combat.rs` L246: `attack += i32::from(striker.spells.slayer.0) + slayer_bonus` is a checked
     `i32` addition (overflow checks are on in every profile). `slayer_bonus` is clamped to the
     `i32` range at L325–326, so a bonus near `i32::MAX` panics here. It needs a pack whose Slayer
     change has no specialist rule of its own (the scaled rule then applies) and a hero whose
     `SpellSpecialty` effects for that spell sum high (`SpellBonuses::of` adds `bp_per_level`
     without a bound on the number of effects, `magic.rs` L199–202). The base Slayer has a
     creature-level rule (at most 255): no panic there.
  2. `turn::activate` (L207–252) picks again after every passed turn, and `pick` starts new rounds
     without limit; the loop ends only because a turn-stopping effect runs out. The pack compiler
     accepts `duration = "permanent"` for a timed change that stops turns (`h3-pack/src/lib.rs`
     L944, no check). With such a spell on every stack that can act, one command never returns. The
     base packs' Blind, Stone Gaze and Paralyze all count rounds.
- fix or test sketch: `saturating_add` at L246; reject a permanent or `fixed` duration beyond a
  bound for `blind`, `petrify` and `paralyze` changes in the compiler, or bound the loop.

## Wiki checks wanted

1. Behemoth and Ancient Behemoth: does the page (or Damage) give the rounding of the defense
   reduction, or an example (defense 10 against a Behemoth: 5 or 6)? (F-1)
2. Frenzy: how long it lasts (the stack's next turn, its next attack, one round) and whether a
   retaliation ends it. (F-4)
3. Fire Shield: anything on a retaliation against an attacker that the shield killed, and the HotA
   changelog for a fix of it. (F-2)
4. Magic Mirror: whether the reflected spell can go to the caster's war machines; and whether the
   page or the HotA changelog says that the computer player's spells are never reflected (the
   code reading says so, §7.13; I read the same).
5. Anti-Magic: which active spells the cast removes (only negative ones below its level, or all
   negative ones) and that the stack's own side's spells of those levels are blocked too.
6. Adela (Bless specialty): "3 % per n hero levels" with the levels divided first, as the code has
   it (VCMI multiplies first).

## Not reviewed, left open

- Stage 1 beyond the Bless specialist's term (0x443320 before 0x4436F0, 0x443840) and the whole of
  0x5A78E0 past its three calls: BAT-DMG-002's and BAT-MAG-004's reviewers.
- BAT-MAG-058's detail that the engine takes the larger of a cast Fire Shield and the creature's own
  (the original: the cast one first): no effect with the base values (the spell is at least 20 %).
- What the original does with an active stack that a redirected Blind hits during its own turn: I
  found no test of it in the loop around 0x4745DF (only Berserk is tested there); the engine lets
  the stack finish its turn. Not followed further.
- The wiki quotes in the catalog (BAT-MAG-023) could not be checked (no wiki access).
- No test was run; F-3's test sketch is `needs-run`.
