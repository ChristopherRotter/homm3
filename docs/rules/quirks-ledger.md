# Quirks ledger

Every known difference between the original game's behavior and a clean design. See `docs/04-rules-and-sources.md` for classes, the decision guide and profiles. This is the battle lane's ledger; new entries take the id `BAT-QUIRK-<n>` (the `Q-NNNN` ids stay). Other lanes keep their own `docs/rules/quirks-<lane>.md` (`docs/08-work-process.md`, "Lanes").

Classes: `exploit-fix` (always fixed, never toggleable) · `preserve` (accepted, flag on in `faithful`) · `fix-optional` (flag; default per profile).

## Entry template

```
### Q-NNNN — Short title
- Class: exploit-fix | preserve | fix-optional
- Context / rules affected: <context>, rule IDs
- Original behavior: precise description (conditions, numbers)
- Intended/expected behavior (if known):
- Impact: game-breaking | exploit | gameplay-altering | cosmetic
- Community status: e.g. "well known, accepted in tournaments" / "unknown"
- Sources: (same labelling as rule sources, with URLs)
- Flag: `quirk.<context>.<name>` (none for exploit-fix)
- Defaults: faithful = on/off, strict-fixed = on/off
- Regression test: test name (required for exploit-fix)
- Decided by / date:
```

## Entries

### Q-0001 — Spending another player's resources (exploit class)
- Class: exploit-fix
- Context / rules affected: town/economy, game (command validation); rule IDs to be assigned when the economy epic starts
- Original behavior: reported by the user: in the original game it is possible to buy things in your town using another player's resources. Exact mechanism and preconditions to be documented from sources when the economy epic begins.
- Intended behavior: a player can only spend from their own treasury.
- Impact: exploit
- Community status: known exploit
- Sources: user report (2026-09-30); details to be collected
- Flag: none (always fixed)
- Defaults: n/a
- Regression test: to be written with the first command that spends resources (must attempt the exploit through the command API and assert rejection, state unchanged)
- Decided by / date: user requirement, 2026-09-30

### Q-0002 — Information exposure beyond player knowledge (exploit class)
- Class: exploit-fix
- Context: game/projection (security)
- Original behavior: the original client holds full game state; information a player should not see can be read by a modified client (user-reported concern).
- Intended behavior: clients receive only projected observations/events.
- Instances found in the original's code (fixed): Fire Wall's and Force Field's target test refuses a hex with the other side's hidden Quicksand, so the cursor shows it; a hypnotized stack's stops are its owner's pits, so its controller sees where they lie (E04b T5c1, `docs/formats/installation.md` §7.16, BAT-MAG-031); Expert Remove Obstacle's target test finds the other side's hidden pits and mines (the wiki's Remove Obstacle page; E04b T5b, §7.15, BAT-MAG-034). Quicksand's and Land Mine's random draws avoiding the other side's hidden obstacles are a leak of the same kind that the user classified fix-optional (BAT-QUIRK-4).
- Impact: exploit
- Sources: user requirement (2026-09-30)
- Flag: none
- Regression test: observation-hygiene property tests per context
- Decided by / date: user requirement, 2026-09-30

### Q-0003 — No luck without a hero
- Class: fix-optional
- Context / rules affected: battle, BAT-LCK-002
- Original behavior: creatures of a side without a hero (neutral monsters, garrisons and town defenders without a hero) never get lucky strikes, although their luck is shown (e.g. Halflings' +1). Morale is not affected.
- Intended/expected behavior: luck works like for any other stack (HotA fixed it).
- Impact: gameplay-altering (small)
- Community status: documented as an original bug on the wiki; fixed in Horn of the Abyss
- Sources: wiki, Luck page ("originally, luck never worked for creatures without a hero", https://heroes.thelazy.net/index.php/Luck, retrieved 2026-10-01); wiki, Halfling page (https://heroes.thelazy.net/index.php/Halfling, retrieved 2026-10-01)
- Flag: `quirk.battle.no_luck_without_hero`
- Defaults: faithful = on, strict-fixed = off
- Regression test: `tests::morale::without_a_hero_there_is_no_luck_under_quirk_q0003` (h3-battle)
- Decided by / date: user, 2026-10-01 (OPEN-QUESTIONS Q17)


### Q-0004 — The Ballista has unlimited shots
- Class: fix-optional (survey S-01, suggested class; confirmed by experiment, Q30 rule)
- Context / rules affected: battle, BAT-WAR-002, BAT-WAR-006 (E04a T7a)
- Original behavior: the Ballista shoots on after its 24 shots without an Ammo Cart; the displayed shot count goes negative. Observed with Expert Artillery: two shots per round, 22 and 20 shots shown after rounds 1 and 2, 0 after round 12, then two shots per round with negative counts.
- Intended/expected behavior: the Ballista uses its 24 shots unless an Ammo Cart supplies it (HotA fixed it; VCMI and our E03 rule).
- Impact: gameplay-altering (small)
- Community status: documented on the wiki (War Machine page), fixed in Horn of the Abyss
- Sources: observation, `experiments/results/E04a-batch-1a.md` R7 (SoD 4.0 Complete, 2026-10-03); wiki, War Machine (https://heroes.thelazy.net/index.php/War_Machine, retrieved 2026-10-03)
- Flag: `quirk.battle.unlimited_ballista_shots` (registered by E04a T7a, index 2): the shooting war machine (stack trait `UNLIMITED_SHOTS`, set at setup) shoots without shots left, automatic or commanded; its count stays at 0 (we do not show negative counts). Agent decision: also the automatic Ballista without Artillery (only observed with Expert Artillery; an automatic Ballista needs 25 rounds to run dry)
- Defaults: faithful = on, strict-fixed = off
- Regression test: `h3-battle` `tests/war.rs` `under_quirk_q_0004_the_ballista_shoots_on_without_shots` (both profiles)
- Decided by / date: suggested class S-01, confirmed by experiment batch 1a, 2026-10-03

### Q-0005 — The original's rounding: staged damage and single-precision rates
- Class: fix-optional (proposed at the batch-1a report and confirmed by the user, 2026-10-03; survey S-02 said "decide with Q23")
- Context / rules affected: battle, BAT-DMG-002 and every damage factor with a rate that is not a binary fraction (attack and defense points, skills, specialties, artifacts; E04a T6); since batch 1b also the surrender price's discount (BAT-FLE-002, E04a T9b); since E04b the spell damage bonuses (Sorcery; BAT-MAG-004, T2b/T2d), magic resistance (BAT-MAG-008, HER-BAT-013, T2d) and Slow's speed share (BAT-MAG-011, T3a); since E04b T7a2 Poison's and Age's hit points per creature (BAT-MAG-044); the Behemoths' defense reduction (BAT-DMG-006, E04b T3b2); Eagle Eye's chance (BAT-MAG-069, HER-SPL-004, E04b T8d); a hero's maximum spell points and the experience it gains, grown by Intelligence and Learning (HER-MAN-001, E08 T18b; HER-EXP-001, E08 T19a2a); its Necromancy share and the count it raises per killed record, the x87's quotient truncated (HER-NEC-001, GAM-BAT-006, E08 T19a2b)
- Original behavior (**since E04b T3b1, Q50**: physical damage after the executable, `docs/formats/installation.md` §7.10): damage is computed in three truncated stages in x87 arithmetic (bonuses as an integer, each fractional one truncated as it is added; times the defense and penalty factor; times the protection factor of Shield or Air Shield and Armorer, at least 1); the attack and defense rates are doubles (0.05 and 0.025 slightly above the decimals, the defense factor stored as a double), skill rates and the spells' stored factors single precision; the x87 rounds every operation to 53 bits (Windows' default; batch 2b R1: 195 and 390, at 64 bits 194 and 389). Both this and the earlier model below fit R1–R3; they differ by one or two points in about half the strikes that combine a bonus or defense with Armorer (59 base, one attack point, Basic Armorer: 57, rounded once 58). *Earlier model (E04a T6a, superseded for physical damage):* the damage factors use the binary single-precision values of their rates: 0.05 per attack point is 13421773/2^28 (slightly more than 5 %), 0.025 per defense point 13421773/2^29, Basic Armorer's 0.05 likewise, and the product is rounded down once. So a damage that is an exact integer with decimal rates can come out one lower (Angels with 24 defense points against them: 19 instead of 20; 100 base against Basic Armorer: 94 instead of 95) or stay exact (attack bonuses err upwards). Observed in R1–R3 (18 values, one model). The surrender price does the same with its discount (batch 1b R4): half of 10 Pikemen (300 gold) with Basic Diplomacy, Statesman's Medal and Diplomat's Ring (40 %) costs 179, because 0.4 is 0.4000000059604645 in single precision (exact: 180).
- Intended/expected behavior: exact decimal rates, rounded once (damage: our BAT-DMG-002 with the flag off; surrender: 180 in the example).
- Impact: gameplay-altering (tiny: at most one damage point per strike)
- Community status: the Armorer case is on the wiki (Damage page); the cause was unknown; the wiki's Armorer example (2002 → 2001) is not explained by the single-precision rates alone (noted at batch 1a; not reproduced, not staged)
- Sources: observation, `experiments/results/E04a-batch-1a.md` R1–R3 (SoD 4.0 Complete, 2026-10-03), `experiments/results/E04a-batch-1b.md` R4 (2026-10-04) and `experiments/results/E04b-batch-2b.md` R1, R4, R6 (plain HD, 2026-10-07: the staged damage at 53 bits, Shield and Armorer in one product, Armorer's single-precision rate, a specialist's grown rate); original-data, the executable's skill tables are f32 (`docs/formats/installation.md` §7.4); wiki, Damage (https://heroes.thelazy.net/index.php/Damage, retrieved 2026-10-01)
- Flag: `quirk.battle.single_precision_rates` (registered by E04a T6a; computed exactly in integers, ADR-0003: `h3_kernel::arith::single_precision` gives the binary32 value of a rate as an exact fraction over 2^40, and the damage product is exact with one final floor). Every rate and cap of a damage factor is converted, counts multiply the converted rate (22 points: 22 × 13421773/2^28); not observed and chosen for consistency: the 70 % defense cap (0.69999999), jousting's 5 % per hex; a hero's summed skill, artifact and specialty bonus enters as one rate (E04a T6b); the summed surrender discount enters as one rate (E04a T9; the observation cannot tell per-source from summed conversion, both give 179). Spells (E04b T2d, from the code, `docs/formats/installation.md` §7.8; user, 2026-10-06, Q44): the Orb and Sorcery factors are single precision, and the value is kept as a single-precision number through the caster's stage as the code keeps it (0x4E5760; E04b T11b): the value, its product with the Orb's factor and that one's with 1 + Sorcery are each stored in single precision, the specialist's bonus is added to the stored product and the sum, stored once more, is rounded down, so a product a hair below a whole number becomes that number (a Sorcery specialist's 16 % on 50 deals 58; the exact product's floor is 57); the hero's resistance sources are added as single-precision values, each sum stored in single precision, and subtracted from the creature's share (stored likewise), so 10 % hero resistance leaves an 89 % chance to be affected and 5 % leaves 94 % (exact: 90 % and 95 %); not observed yet (batch 2a); Slow's speed share (E04b T3a, `installation.md` §7.9) is stored as a single-precision value and the slowed speed rounded down (Slow's 75 % and 50 % are binary fractions, so only a data spell with another share differs: 70 % of speed 10 gives 6, exact 7; BAT-MAG-011); Poison's factor and Age (E04b T7a2, `installation.md` §7.20) as the code stores them: a factor falling by f32(0.1) and stored as an f32, times H as an f32, + f32(0.95), truncated, so a value at .05 stays below the next whole number (9 hit points poisoned once and aged: 4, exact 5; BAT-MAG-044); the Behemoths' defense reduction (`installation.md` §7.11, 0x4425C8 to 0x4425F5) as the code computes it: the defense left from single-precision values, `trunc(f32(D) − f32(D) × f32(share))`, so where D × the share is whole the stored 0.4 or 0.8 costs one more point (10 against a Behemoth leaves 5, exact 6; BAT-DMG-006); Eagle Eye's chance (E04b T8d, `installation.md` §7.24) as the code sums it: the skill's single-precision value, grown for a specialist, plus each artifact's single-precision share, each sum stored in single precision, then `trunc(p × 100)` (a Basic specialist at level 2: 43 %, exact 44 %; by the same arithmetic Advanced with the Emblem of Cognizance gives 64 %, exact 65 %, which no test pins yet; HER-SPL-004, BAT-MAG-069)
- Defaults: faithful = on, strict-fixed = off
- Regression test: R1–R3 as fixtures (`crates/h3-battle/src/tests/rules.rs`, `the_staged_damage_reproduces_batch_1a`; batch 2b R1, R3, R4, R6: `the_staged_damage_reproduces_batch_2b`, Mephala's rate from the harvested packs: `crates/h3-pack/src/lib.rs`, `harvested_magic_derives`; the stages: `the_stages_truncate_each_bonus_and_factor`; the flag in a battle: `the_quirk_flag_selects_the_staged_damage`; the x87 arithmetic: `crates/h3-kernel/src/arith.rs`, `x87_operations`, and `crates/h3-kernel/src/num.rs`, `x87_at_53_bits_is_double_arithmetic`; Shield and the Protections: `crates/h3-battle/src/tests/protect.rs`; the surrender discount: `crates/h3-battle/src/tests/flee.rs`, `the_discount_enters_as_a_single_precision_rate_under_q_0005`; spells: `crates/h3-battle/src/tests/magic.rs`, `the_casters_bonuses_apply_in_the_originals_order` and `the_casters_products_are_stored_as_singles_with_quirk_q0005`, `crates/h3-pack/tests/harvested_spell_damage.rs`, `a_sorcery_specialists_product_is_stored_as_a_single_before_it_is_rounded_down`, and `crates/h3-battle/src/tests/spells.rs`, `resistance_adds_the_heros_to_the_creatures_and_the_aura_multiplies`; Slow: `crates/h3-battle/src/tests/timed.rs`, `slow_rounds_down_and_leaves_at_least_one`; Poison and Age: `crates/h3-battle/src/vitality.rs`, `hit_points_round_up_from_0_05_and_stop_at_half`; the Behemoths: `crates/h3-battle/src/tests/abilities.rs`, `defense_reduction_ignores_a_share_of_the_defense_rounded_down`; Eagle Eye: `crates/h3-battle/src/tests/eagle.rs`, `the_chance_is_truncated_to_whole_percent`)
- Decided by / date: confirmed by experiment batch 1a (Q23), 2026-10-03; class fix-optional confirmed by the user, 2026-10-03; extended to the surrender discount by the user (batch 1b, Q32), 2026-10-04; extended to magic resistance by the user (Q44), 2026-10-06; physical damage staged after the executable's code by the user (Q50), 2026-10-06; the grown skill rates (specialists) and the skill getters' artifact sums cross the hero seam in the original's single precision, computed by the derivation (Q45 option (b), the user 2026-10-06; built in E04b T3c: `h3_contracts::Rate`, HER-BAT-004/005/012/013, the tent's heal BAT-WAR-007); extended to Poison's and Age's hit points per creature (E04b T7a2) by the user at the T7a checkpoint, 2026-10-08

### Q-0006 — Roland's specialty affects only Crusaders
- Class: fix-optional (survey S-03, suggested class; confirmed for Roland only)
- Context / rules affected: content (hero specialties), the creature specialty rule (E04a T10)
- Original behavior: the executable's specialty table names the Crusader (creature 7) for Roland, so his specialty raises only Crusaders, although it is presented as a Swordsman specialty. Observed at hero level 4: Roland's Crusaders get +1/+1/+1, his Swordsmen nothing; Catherine's and Sorsha's specialty (the Swordsman) raises both. The wiki's claim that Catherine's is bugged as well does not hold in SoD.
- Intended/expected behavior: Swordsmen and Crusaders (HotA fixed it).
- Impact: gameplay-altering (a campaign hero)
- Community status: on the wiki (Swordsman, Hero specialty pages; Catherine mis-attributed), fixed in Horn of the Abyss
- Sources: observation, `experiments/results/E04a-batch-1a.md` B4–B6 (SoD 4.0 Complete, 2026-10-03); original-data, the specialty table (`docs/formats/installation.md` §7.2); wiki, Hero specialty (https://heroes.thelazy.net/index.php/Hero_specialty, retrieved 2026-10-03)
- Flag: `quirk.content.roland_crusader_specialty`, a data quirk (ADR-0007, amendment E04b T8e): the `ab` pack keeps the executable's Crusader and a `[[hero_override]]` gated by the flag names the Swordsman (E04b T8e, BAT-Q25; the harvester's gap-fill `data_quirks.specialty_creature`)
- Defaults: faithful = on (Crusader), strict-fixed = off (Swordsman)
- Regression test: `crates/h3-pack/src/lib.rs`, `harvested_hero_effects_reproduce_the_experiments` (B4–B6 with the harvested packs; E04a T10a) and `the_data_quirks_are_fixed_in_strict_fixed` (E04b T8e)
- Decided by / date: suggested class S-03, confirmed by experiment batch 1a, 2026-10-03

### Q-0007 — A spell of several schools meets the first protection, not the strongest
- Class: fix-optional (survey S-19, suggested class)
- Context / rules affected: battle, BAT-MAG-013
- Original behavior: a damage spell of several schools (Magic Arrow) checks the target's Protections in the order earth, air, fire, water and applies the first that is active, even when a later one is stronger: a stack with Basic Protection from Earth (70 %) and Expert Protection from Air (50 %) takes 70 % of a Magic Arrow.
- Intended/expected behavior: the strongest protection (HotA's fix).
- Impact: gameplay-altering (tiny)
- Community status: fixed in Horn of the Abyss (changelog: "now the most effective of defenses is used")
- Sources: original-data, `docs/formats/installation.md` §7.8 and §7.10 (0x5A7BB0); HotA changelog (survey S-19)
- Flag: `quirk.battle.first_protection` (E04b T3b1). The order is data: each spell school's `protection_rank` (pack format v6; the base pack: earth 0, air 1, fire 2, water 3)
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-battle/src/tests/protect.rs`, `a_spell_of_two_schools_meets_one_protection`
- Decided by / date: suggested class S-19; confirmed for SoD by code reading, 2026-10-06 (E04b T3b1)

### Q-0008 — Counterstrike's extra retaliations last into the round in which it runs out
- Class: fix-optional (user, 2026-10-07)
- Context / rules affected: battle, BAT-MAG-015
- Original behavior: every round start first resets each stack's retaliations (adding Counterstrike's `v` while the spell is active) and only then counts the timed effects down, so in the round at whose start Counterstrike runs out the stack still has its extra retaliations, although the spell is gone.
- Intended/expected behavior: the extra retaliations last exactly as long as the spell (strict-fixed: the round start gives no extra for an effect that ends at it).
- Impact: gameplay-altering (tiny)
- Community status: unknown
- Sources: original-data, `docs/formats/installation.md` §7.10 (0x447120 before the countdown); observation, `experiments/results/E04b-batch-2b.md` R2 (SoD 4.0 Complete, plain HD, 2026-10-07): Counterstrike at power 1 cast in round 2, both Wolf Rider attacks struck back in round 3 while the spell is no longer listed, only the first in round 4
- Flag: `quirk.battle.counterstrike_extra_round` (E04b T3b1b)
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-battle/src/tests/protect.rs`, `counterstrike_adds_retaliations_now_and_at_each_round_start`
- Decided by / date: user, 2026-10-07 (E04b T3b1a checkpoint); found by code reading, 2026-10-06 (E04b T3b1a); confirmed by experiment batch 2b, 2026-10-07

### Q-0009 — Frenzy's attack bonus counts a Behemoth's defense reduction
- Class: fix-optional (survey S-16, suggested class; confirmed for SoD by code reading)
- Context / rules affected: battle, BAT-MAG-019, BAT-DMG-006
- Original behavior: a frenzied stack adds its defense × v % to its attack, computing that defense as the struck stack would see it: against a Behemoth (Ancient Behemoth) it is first reduced by 40 % (80 %), so a frenzied stack attacking Behemoths gains less (0x442410 calls 0x442590 with the target).
- Intended/expected behavior: the frenzied stack's own defense, unreduced (HotA's fix).
- Impact: gameplay-altering (small)
- Community status: fixed in Horn of the Abyss (changelog, survey S-16)
- Sources: original-data, `docs/formats/installation.md` §7.11; HotA changelog (survey S-16)
- Flag: `quirk.battle.frenzy_reduced_defense` (E04b T3b2)
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-battle/src/tests/rage.rs`, `frenzy_against_a_behemoth_counts_its_reduction_with_quirk_q0009`
- Decided by / date: suggested class S-16 (user, Q30); confirmed for SoD by code reading, 2026-10-07 (E04b T3b2)

### Q-0010 — Fire Shield reflects a share of the strike before the defense and protections
- Class: fix-optional (user, 2026-10-07)
- Context / rules affected: battle, BAT-MAG-018
- Original behavior: the reflected damage is `v %` of the strike's first damage stage (the base damage with its bonuses: luck, attack advantage, Offense, …), capped at the struck stack's hit points, taken before the struck stack's defense, the penalties, Shield, Air Shield, the Protections and Armorer reduce the strike; a well-defended shielded stack reflects more than `v %` of what it lost (0x443F40 passes stage 1 to 0x422440)
- Intended/expected behavior: `v %` of the damage dealt, as the spell's text says (VCMI; strict-fixed: the hit points the strike took).
- Impact: gameplay-altering (small)
- Community status: unknown
- Sources: original-data, `docs/formats/installation.md` §7.11; VCMI 1.7.5 (`server/battles/BattleActionProcessor.cpp` L1603–L1616: a share of the damage dealt after all reductions, capped at the struck stack's health; its own remark there says the base should be the damage with bonus but without penalties); observation, `experiments/results/E04b-batch-2b.md` R3 (SoD 4.0 Complete, plain HD, 2026-10-07): 10 Naga Queens deal 150 (first stage 300) to Fire-Shielded Peasants, "The Fire Shield does 60 damage." (of the damage dealt: 30)
- Flag: `quirk.battle.fire_shield_first_stage` (E04b T3c)
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-battle/src/tests/rage.rs`, `fire_shield_reflects_a_share_of_the_first_stage_capped_at_the_targets_health_with_quirk_q0010`
- Decided by / date: user, 2026-10-07 (E04b T3b2 checkpoint); found by code reading, 2026-10-07 (E04b T3b2); confirmed by experiment batch 2b, 2026-10-07. Considered at the same checkpoint and not a quirk: a breath attack's second stack reflects nothing, in every profile (the rule, BAT-MAG-018).

### Q-0011 — Forgetfulness is the mass version from Advanced
- Class: fix-optional (survey S-10, suggested class; confirmed for SoD by code reading; user, 2026-10-07, Q53)
- Context / rules affected: battle, BAT-MAG-023, BAT-MAG-003
- Original behavior: the spell table's flag `0x20` gives Forgetfulness a target stack only below Advanced (0x59E060), so from Advanced it reaches every enemy shooter it can affect, against its in-game text (Advanced: "the selected enemy unit").
- Intended/expected behavior: one target at Advanced, every enemy shooter at Expert (the text; VCMI 1.7.5, `config/spells/timed.json`; HotA's fix).
- Impact: gameplay-altering
- Community status: fixed in Horn of the Abyss (wiki Forgetfulness)
- Sources: original-data, `docs/formats/installation.md` §7.6, §7.8, §7.14; wiki Forgetfulness (survey S-10)
- Flag: `quirk.battle.early_mass_version` (E04b T4b): a spell's mass version starts at `mass` (the original's flag word) with the flag, at its `fixed_mass` without it (pack format v11)
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-battle/src/tests/actions.rs`, `forgetfulness_reaches_enemy_shooters_and_is_mass_from_advanced_with_its_quirk`
- Decided by / date: user, 2026-10-07 (E04b T4a checkpoint, Q53). Considered at the same checkpoint and not a quirk: a forgetful shooter's melee strikes and retaliations count half its creatures too (survey S-09), the rule in every profile (BAT-MAG-023).

### Q-0012 — Expert Cure skips hypnotized stacks
- Class: fix-optional (survey S-13, suggested class; confirmed for SoD by code reading; user, 2026-10-07, Q53)
- Context / rules affected: battle, BAT-MAG-026, BAT-MAG-003, BAT-MAG-016
- Original behavior: Expert Cure loops over the caster's own stacks and skips those under Hypnotize (0x5A1847), as the mass handler of the timed spells does (0x5A66D0), so it never frees a hypnotized stack; a single Cure from its owner reaches it (the side tests use the real side) and ends Hypnotize. Dispel's Expert loop (from 0x5A160F) has no such test, so Expert Dispel frees it in every profile (BAT-MAG-016).
- Intended/expected behavior: Expert Cure also cures the caster's hypnotized stacks and ends their Hypnotize (strict-fixed: a mass version whose dispel ends the stack's Hypnotize reaches it; the mass version of a spell that only dispels, Dispel, reaches hypnotized stacks with and without the flag; the mass versions of the timed spells still pass over them).
- Impact: gameplay-altering (small)
- Community status: unknown ("may/may not be a bug", wiki Cure)
- Sources: original-data, `docs/formats/installation.md` §7.14; wiki Cure (survey S-13); observation `experiments/results/E04b-batch-2c.md` H1 (SoD 4.0 Complete, plain HD, 2026-10-07: Hypnotize still listed after Expert Cure, which healed the caster's other Swordsmen to 35) and C1 (a Basic Cure ends it)
- Flag: `quirk.battle.mass_cure_skips_hypnotized` (E04b T4c)
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-battle/src/tests/control.rs`, `mass_versions_pass_over_hypnotized_stacks_and_expert_cure_only_with_q0012`; Expert Dispel with and without the flag: `expert_dispel_reaches_hypnotized_stacks_in_every_profile`
- Decided by / date: user, 2026-10-07 (E04b T4a checkpoint, Q53)

### BAT-QUIRK-1 — The natives' sight of hidden obstacles is fixed at the battle's start (retired: not a quirk)
- Class: none: considered, not a quirk (retired by the user, 2026-10-13, BAT-Q34; before: fix-optional, user, 2026-10-07, BAT-Q1)
- Context / rules affected: battle, BAT-MAG-031, BAT-MAG-033 (ADR-0014 point 7, amended)
- Original behavior: at the battle's start the original sets one byte per side (+0x1329C, 0x46402C): whether any of its stacks fights on its native terrain. When Quicksand or a Land Mine is placed, the object is revealed to both sides if the other side's byte is set (0x5A0442), and Land Mine is refused while it is (0x59ECEE). Nothing writes the byte again: a side whose natives die keeps seeing what is placed later, and stacks created later change nothing.
- Engine behavior: the original's, in every profile (BAT-MAG-031).
- Why it is no quirk: the flag's other variant (a side sees while it has a living native stack) was the engine's own design, kept as the "fix" when the code was read. No source describes it: VCMI 1.7.5 asks each time but counts dead natives too (`lib/battle/CBattleInfoEssentials.cpp` L107–L126), the wiki's Land Mine page says "started the battle with" (the original's timing) and its Quicksand page names no timing ("unless the hero has a creature fighting on native terrain"; the wiki checks, 2026-10-13).
- Impact: none (the engine follows the original)
- Community status: the wiki describes the original's timing (Land Mine)
- Sources: original-data, `docs/formats/installation.md` §7.16; VCMI 1.7.5 `lib/battle/CObstacleInstance.cpp` L152–L162 and `lib/battle/CBattleInfoEssentials.cpp` L107–L126; the wiki's Land Mine and Quicksand pages
- Flag: none; retired: `quirk.battle.natives_sight_at_start` (E04b T5c1 to 2026-10-13). Its position 9 in `h3_content::REGISTRY` stays reserved and a profile that names it does not compile (`CompileError::UnknownQuirk`)
- Defaults: none (no flag)
- Regression test: `crates/h3-battle/src/tests/pits.rs`, `natives_at_the_start_reveal_the_other_sides_pits`, `without_natives_at_the_start_the_other_sides_pits_stay_hidden`; `crates/h3-battle/src/tests/mines.rs`, `land_mine_is_refused_while_the_other_sides_natives_would_see_the_mines`; the retired name: `crates/h3-content/src/quirks.rs`, `a_retired_flag_keeps_its_position_and_has_no_name`
- Decided by / date: user, 2026-10-07 (E04b checkpoint before T5c, BAT-Q1); review (E04b T11b, 2026-10-12): at that decision this entry described VCMI as testing the living natives, which it does not; retired: user, 2026-10-13 (BAT-Q34)

### BAT-QUIRK-2 — An attack from a seen Quicksand pit (retired: not a quirk)
- Class: none: considered, not a quirk (retired by the user, 2026-10-12, BAT-Q30; before: fix-optional, user, 2026-10-07, BAT-Q1)
- Context / rules affected: battle, BAT-MAG-032
- Original behavior: the walker (0x441FA0) stops at a step into a hex its side's stops mark (a pit it sees), also when that hex is the move's destination, and returns false; a move and attack (0x478971) then skips the attack. No path of the original issues such a command: the cursor's attack positions (0x474690, 0x4747BE to 0x474810; the from-hex only from them, 0x474A00) and the pathfinder's marking of attack targets within one move (0x4B378D to 0x4B37D1) leave out every position with a hex on one of the stack's stops, so the original offers no attack from a pit the stack's side sees.
- Engine behavior: such an attack is not legal in any profile (CommandError::Unreachable, not listed), as in the original; the plain move into the pit stays legal.
- Impact: none (the engine follows the original)
- Community status: unknown
- Sources: original-data, `docs/formats/installation.md` §7.16
- Flag: none; retired: `quirk.battle.attack_from_seen_pit` (E04b T5c1 to 2026-10-12). Its position 10 in `h3_content::REGISTRY` stays reserved and a profile that names it does not compile (`CompileError::UnknownQuirk`)
- Defaults: none (no flag)
- Regression test: `crates/h3-battle/src/tests/pits.rs`, `an_attack_whose_walk_stops_in_a_seen_pit_is_neither_listed_nor_accepted`; `crates/h3-battle/src/tests/control.rs`, `a_berserk_stack_does_not_attack_from_a_pit_its_side_sees`; the retired name: `crates/h3-content/src/quirks.rs`, `a_retired_flag_keeps_its_position_and_has_no_name`
- Decided by / date: user, 2026-10-07 (E04b checkpoint before T5c, BAT-Q1); retired: user, 2026-10-12 (BAT-Q30)
- Review (E04b T11b, 2026-10-12): the re-read finds that the original's interface never issues such a command (the part this entry marks "not read"): the cursor's attack slots take a neighbouring position only if neither its hex nor, for a two-hex stack, its other hex is among the stack's stops (0x474690, 0x4747BE to 0x474810), the attack's from-hex comes only from these slots (0x474A00), and the pathfinder's own marking of attack targets passes over a position on a stop for a query within one move, as the cursor's is (0x4B378D to 0x4B37D1). So no attack from a seen pit is offered to a player, and the flag's "on" variant added commands that no path of the original's interface issues (`docs/formats/installation.md` §7.16). The user retired the flag (BAT-Q30): the engine offers and accepts no such attack in any profile.

### BAT-QUIRK-3 — A natural obstacle's anchor hex belongs to it
- Class: fix-optional (user, 2026-10-07, OPEN-QUESTIONS Q55)
- Context / rules affected: battle, BAT-GEO-003, BAT-MAG-027, BAT-MAG-032, BAT-MAG-034
- Original behavior: a usual obstacle is an object whose anchor hex (its bottom-left hex) gets bit `0x1` and the object's index even where the obstacle does not block it (28 of the 55 usual types; 0x4669B0). Such a hex may be stood on and walked through, but the spell obstacles' target test and Quicksand's and Land Mine's draws want none of the bits `0x3F`, so it takes no Fire Wall, Force Field, pit or mine, and Remove Obstacle aimed at it removes the obstacle.
- Intended/expected behavior: the anchor is no part of the obstacle: a hex the obstacle does not block is a free hex, and Remove Obstacle is aimed at the obstacle's blocked hexes (strict-fixed).
- Impact: gameplay-altering (small: one hex beside some obstacles)
- Community status: unknown (the Q55 survey: the wiki, HotA's changelog, VCMI's issues and df2.ru describe nothing of it; VCMI has no anchors)
- Sources: original-data, `docs/formats/installation.md` §7.15; observation, `experiments/results/E04b-batch-2c.md` A1, A2
- Flag: `quirk.battle.obstacle_anchors` (E04b T5b)
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-battle/src/tests/remove.rs`, `anchors_belong_to_their_obstacles_with_bat_quirk_3`
- Decided by / date: user, 2026-10-07 (E04b T5c2 checkpoint, Q55)

### BAT-QUIRK-4 — Quicksand's and Land Mine's random hexes avoid the other side's hidden obstacles
- Class: fix-optional (user, 2026-10-07, BAT-Q4)
- Context / rules affected: battle, BAT-MAG-032, BAT-MAG-033
- Original behavior: Quicksand (0x5A0354) and Land Mine (0x5A0536) draw their hexes among those without any object bit `0x3F`, the other side's hidden pits and mines included, so the caster learns where none of them lies.
- Intended/expected behavior: the draw avoids only the obstacles the caster's side sees; a new pit or mine may go over a hidden one of the other side, and both stay (strict-fixed, as Q-0002 fixes the cursor leaks).
- Impact: gameplay-altering (small information leak)
- Community status: unknown
- Sources: original-data, `docs/formats/installation.md` §7.16, §7.17; found by the T5c2 review
- Flag: `quirk.battle.draws_avoid_hidden` (E04b T5b)
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-battle/src/tests/remove.rs`, `random_draws_avoid_hidden_obstacles_only_with_bat_quirk_4`; the property test `hidden_obstacles_do_not_show` covers casts without it
- Decided by / date: user, 2026-10-07 (E04b T5c2 checkpoint, BAT-Q4)
- Review (E04b T11b, 2026-10-12): a leak of the same kind that no flag covers: a berserk stack's automatic turn (BAT-MAG-025) plans and walks by the pits its own side sees, hidden ones included, and its moves and strikes are public, so a detour, a farther target or a passed turn can show the other side where a hidden pit lies. The re-read finds the same in the original (its pathfinder takes the stops of the stack's side for every caller, 0x4B3384 to 0x4B33C4, 0x4B353E; `docs/formats/installation.md` §7.16); what its walk does when the only attack position lies on a pit its side sees was not read to the end. The engine is as the original here, and stays so in every profile (the user, 2026-10-12, BAT-Q31: kept as the original and noted here; no flag). The property test `hidden_obstacles_do_not_show` therefore leaves Berserk out of its heroes' books.
- Security review (E04c T10b, 2026-10-15): another leak no flag covers: the automatic turrets' and Ballista's aim (BAT-SIE-020 to BAT-SIE-023, BAT-WAR-008) reads the other side's private facts, its hero's Armorer reduction and Artillery level, and the target it shoots is public, so the choice can tell the other side something of them. The original's computer reads the same (`docs/formats/installation.md` §7.31, §7.33); the engine is as the original in every profile (the user, 2026-10-15, BAT-Q50 (5): kept as the original and documented, as BAT-Q31's berserk walk; no flag).

### BAT-QUIRK-5 — Sacrifice tests the stack it raises as Resurrection would
- Class: fix-optional (user, 2026-10-08)
- Context / rules affected: battle, BAT-MAG-036 (with BAT-MAG-035)
- Original behavior: Sacrifice finds and tests the stack it raises through Resurrection's routine (0x5A3950 calls 0x5A3CC0, which asks 0x5A8090 about spell 38): Resurrection's conditions, its level 4 against spell-level immunities and Anti-Magic (Gold Dragons, immune to levels 1–4, are never raised by Sacrifice; survey S-15), and Resurrection's hit points at the caster's Earth mastery, which must reach one creature's health, although Sacrifice raises its own amount. The stack so found must pass Sacrifice's own test as well (0x5A39C0 asks 0x5A8090 about spell 40 for it as the stack to raise: a living creature, not a summoned stack, fewer creatures than at the start, Sacrifice's level 5 and Fire school against immunities and Anti-Magic), so Efreet are never raised by Sacrifice either.
- Intended/expected behavior: the target is tested with Sacrifice's own conditions and level 5, without a hit-point minimum (strict-fixed).
- Impact: gameplay-altering (small)
- Community status: the wiki's Gold Dragon trivia notes the Gold Dragons' half (S-15); the hit-point test is undocumented
- Sources: original-data, `docs/formats/installation.md` §7.18; wiki, Gold Dragon page (https://heroes.thelazy.net/index.php/Gold_Dragon, retrieved 2026-10-03); observation `experiments/results/E04b-batch-2d.md` RA1 (SoD 4.0 Complete, plain HD, 2026-10-08: Sacrifice refused on dead Titans, whose 300 hit points Resurrection's 290 do not reach, and on dead Gold Dragons)
- Flag: `quirk.battle.sacrifice_tests_as_resurrection` (E04b T6a)
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-battle/src/tests/raise.rs`, `with_bat_quirk_5_sacrifice_tests_its_target_as_resurrection` and `with_bat_quirk_5_the_raised_stack_passes_sacrifices_own_test_too`
- Decided by / date: user, 2026-10-08 (E04b T6a)

### BAT-QUIRK-6 — A winner left without an army (survey S-20)
- Class: exploit-fix
- Context / rules affected: battle, BAT-MAG-035, BAT-END-003
- Original behavior: creatures raised by Basic Resurrection fight on and may win the battle, then vanish. The end of the battle (0x4774E2, `docs/formats/installation.md` §7.18) keeps one creature in the last stack that had any when no creatures are left, but its total counts war machines and summoned stacks or clones, which the army does not keep: a hero with a Ballista (or a living elemental) whose stacks were all raised so is left without an army (HotA changelog: "a defending hero could be left without an army after winning a combat if all his stacks had been resurrected with a Basic-level Resurrection spell").
- Intended/expected behavior: the total counts only army stacks: an army whose living creatures are all temporary keeps one in its last living stack whatever else survives (BAT-END-003, BAT-MAG-035; the user, 2026-10-08).
- Impact: exploit (a hero without an army breaks an invariant)
- Community status: fixed in Horn of the Abyss
- Sources: original-data, `docs/formats/installation.md` §7.18; HotA changelog (survey `docs/rules/surveys/E04-quirks.md` S-20); observation `experiments/results/E04b-batch-2d.md` RA2, RA3 (SoD 4.0 Complete, plain HD, 2026-10-08: without a Ballista the army keeps 1 Swordsman, with one none; the hero without creatures still moves on the map and loses its next battle at once)
- Flag: none
- Regression test: `crates/h3-battle/src/tests/raise.rs`, `a_side_left_with_creatures_raised_for_the_battle_only_keeps_one`; the property `outcomes_account_for_every_stack`
- Decided by / date: user, 2026-10-08 (E04b T6a)

### BAT-QUIRK-7 — A clone placed on its own original makes the original vanish (E04b T6b)
- Class: fix-optional (user, 2026-10-08)
- Context / rules affected: battle, BAT-MAG-038
- Original behavior: Clone's place search (0x5A6C70, `docs/formats/installation.md` §7.19) tests each hex with 0x446C40, which counts the original's own hexes as free. Around a crowded two-hex original (every place in ring 1 taken), ring 2 can pick the original's front hex (its other hex, BAT-GEO-002) as the clone's position: the stack creation (0x47A100) refuses the occupied hex and creates nothing, and the cast goes on to flag *the original* (the stack on that hex, 0x4E7170) as summoned (`0x400000`, not the clone bit `0x800000`) with the clone's rounds and as its own clone (+0x24 and +0x28 both itself): no clone appears, the original vanishes without a corpse when the rounds run out, and its army slot is lost after the battle (observed, batch 2d CL2). The other outcome of the same search, a clone sharing a hex with its original, is BAT-QUIRK-8.
- Intended/expected behavior: the original's hexes are taken like any living stack's; the search goes on (BAT-MAG-038; HotA).
- Impact: gameplay-altering (the caster loses its own stack; Clone takes only stacks of the caster's real side, `installation.md` §7.14, so it cannot be turned against the enemy)
- Community status: known and treated as a bug: "Did you know? #136" (a clone cast on a completely surrounded stack does not appear and the original is removed; "fixed in Horn of the Abyss"); "Знаете ли Вы всё о… «Клоне»" (a wide creature's clone placed on the original's hex: no clone, the original disappears when the spell runs out or after the battle); HD+ lists "Fixed a bug with Clone spell"
- Sources: original-data, `docs/formats/installation.md` §7.19; observation `experiments/results/E04b-batch-2d.md` CL2 (SoD 4.0 Complete, plain HD, 2026-10-08: Clone on crowded two-hex Griffins and Wolf Riders, no clone, both gone at the start of round 3, the Griffins lost from Red's army); heroes3wog.net (https://heroes3wog.net/disappearing-clones-did-you-know-heroes-3-complete/, 2022, retrieved 2026-10-08); pikabu.ru (https://pikabu.ru/story/znaete_li_vyi_vsyo_o_klone_5870302, 2018, retrieved 2026-10-08); HD Mod (https://www.sites.google.com/site/heroes3hd/eng/original-game-bug-fixing, retrieved 2026-10-08)
- Flag: `quirk.battle.clone_on_its_original` (E04b T6c)
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-battle/src/tests/created.rs`, `with_quirk_bat_quirk_7_a_crowded_original_becomes_its_own_clone` (on) and `a_crowded_two_hex_stack_is_never_covered_by_its_clone` (off); a turned original that vanishes as its side's last stack ends the battle at that round start (BAT-END-001): `the_battle_ends_when_a_round_start_takes_a_sides_last_stack`
- Decided by / date: user, 2026-10-08 (batch 2d fold-in: a community search first; known, so fix-optional, as it would be always fixed if unknown). The agent had classified it exploit-fix in T6b.

### BAT-QUIRK-8 — A clone sharing a hex with its original (E04b batch 2d)
- Class: exploit-fix
- Context / rules affected: battle, BAT-MAG-038
- Original behavior: the same search as BAT-QUIRK-7 (0x446C40 counts the original's hexes as free) can place a two-hex clone whose front hex is one of the original's: two living stacks share a hex. The community describes the original as invisible and impossible to target while it stays there and does not act.
- Intended/expected behavior: the original's hexes are taken like any living stack's; the search goes on (BAT-MAG-038; HotA: a wide creature's clone never overlaps the cloned stack).
- Impact: game-breaking (two stacks on one hex; a stack nobody can target)
- Community status: known ("Знаете ли Вы всё о… «Клоне»", 2018); fixed in HotA
- Sources: original-data, `docs/formats/installation.md` §7.19; pikabu.ru (https://pikabu.ru/story/znaete_li_vyi_vsyo_o_klone_5870302, retrieved 2026-10-08). Not observed (batch 2d CL2 staged BAT-QUIRK-7's case)
- Flag: none
- Regression test: the property invariant that living stacks never share a hex (`crates/h3-battle/src/tests/properties.rs`); `crates/h3-battle/src/tests/created.rs`, `a_crowded_two_hex_stack_is_never_covered_by_its_clone` and, with BAT-QUIRK-7's flag on, `a_clone_never_shares_a_hex_with_its_original`
- Decided by / date: agent, 2026-10-08 (state-corrupting bugs are always fixed, AGENTS.md); split from BAT-QUIRK-7 at the batch 2d fold-in, which the user reclassified

### BAT-QUIRK-9 — A creature's spell that Magic Mirror sends away (retired: not a quirk; E04b T7b2a)
- Class: none: considered, not a quirk (retired by the user, 2026-10-12, BAT-Q29; before: fix-optional, user, 2026-10-08, BAT-Q11)
- Context / rules affected: battle, BAT-MAG-049 (with BAT-MAG-022)
- Original behavior: no creature's spell is ever sent away. The redirect is drawn only in the player's targeting routine (0x59EC50, 0x59EE7A to 0x59EF8B) and kept in the manager's +0x48, which only the hero's action hands to the cast routine (0x479036 to 0x479056); the creature's action passes the target hex alone (0x47936E to 0x479382) and every cast of 0x448260 gives the second hex -1. The cast routine's skipped test for caster kind 1 (0x5A02CB) therefore only ever meets the aimed stack. A hero's mirrored spell is tested and rolls (BAT-MAG-022).
- Engine behavior: only a hero's spell is redirected, by the spell or by a creature's own mirror (BAT-MAG-057), in every profile; a stack's cast always meets the stack it is aimed at, as in the original and in VCMI.
- Impact: none (the engine follows the original)
- Community status: unknown (no report found; not searched)
- Sources: original-data, `docs/formats/installation.md` §7.21 ("The Faerie Dragon, re-read for T7b2a" and the review re-read "Magic Mirror never redirects a creature's spell"); VCMI 1.7.5 `lib/spells/BattleSpellMechanics.cpp` L514–L523; the wiki's Magic Mirror page ("Spells cast by creatures … will never be reflected", https://heroes.thelazy.net/index.php/Magic_Mirror, retrieved 2026-10-12)
- Flag: none; retired: `quirk.battle.creature_mirror_untested` (E04b T7b2a to 2026-10-12). Its position 14 in `h3_content::REGISTRY` stays reserved and a profile that names it does not compile (`CompileError::UnknownQuirk`)
- Defaults: none (no flag)
- Regression test: `crates/h3-battle/src/tests/casters.rs`, `a_magic_mirror_never_sends_a_creatures_spell_away`; `crates/h3-battle/src/tests/creature_defences.rs`, `a_mirror_of_its_own_never_sends_a_creatures_spell_away`; `crates/h3-pack/tests/harvested_defences.rs`, `a_faerie_dragons_spell_is_never_sent_away_by_the_other_dragons_mirror`
- Decided by / date: user, 2026-10-08 (E04b T7b2a, BAT-Q11); retired: user, 2026-10-12 (BAT-Q29)
- Review (E04b T11b, 2026-10-12): the re-read finds that the redirect never reaches a creature's cast, so the behaviour described above does not occur in the original: the redirect is drawn only in the player's targeting routine (0x59EC50, 0x59EE7A to 0x59EF8B), which stores the new hex in the manager's +0x48; the hero's action passes it to the cast routine as the second hex (0x479036 to 0x479056), but the creature's action passes only the target hex (0x47936E to 0x479382) and every cast of 0x448260 gives the cast routine the second hex −1. The Faerie Dragon's key path draws a redirect that is dropped; the mouse path draws none. VCMI 1.7.5 agrees (only a hero's spell is reflected, `lib/spells/BattleSpellMechanics.cpp` `isReflected`). The wiki says the same. The user retired the flag (BAT-Q29): the engine redirects only a hero's spell, in every profile.

### BAT-QUIRK-10 — The Master Genie's Slayer test counts dead stacks and only the first king level (E04b T7b2b)
- Class: preserve, no flag: the rule in every profile (user, 2026-10-08, BAT-Q12)
- Context / rules affected: battle, BAT-MAG-050
- Original behavior: the usefulness test 0x447A80 counts Slayer as useful if any record of the opposite side has creature flag `0x80` (king 1); it does not test whether the stack is alive, and enemies of king level 2 or 3 alone (which Slayer at the Genie's Advanced mastery also affects at level 2) do not make it useful (`docs/formats/installation.md` §7.21).
- Intended/expected behavior: none decided; it only shapes which spell the Genie (and later the Enchanter) draws, not a rule players see.
- Impact: gameplay-altering (tiny)
- Community status: unknown
- Sources: original-data, `docs/formats/installation.md` §7.21 ("The Master Genie, re-read for T7b2b")
- Flag: none
- Defaults: n/a
- Regression test: `crates/h3-battle/src/tests/casters.rs`, `the_enemy_decides_the_shields_protections_and_slayer`
- Decided by / date: user, 2026-10-08 (E04b T7b2b, BAT-Q12)

### BAT-QUIRK-11 — A reused record loses its earlier created stack's losses (E04b T6c)
- Class: preserve, no flag: the rule in every profile (user, 2026-10-08, BAT-Q13)
- Context / rules affected: battle, BAT-END-003 (with BAT-MAG-037)
- Original behavior: a new created stack takes the record of the side's first dead summoned stack with no creatures (0x47A100: one that was killed; a clone or a turned original whose rounds ran out keeps its count and so its record) and is set up afresh there (0x43D730: count, start count, temporary creatures), so the dead stack's start count and losses are gone: Necromancy (0x4775E1) and the casualty window count only the later stack (batch 2d SU a: 2 Air Elementals of at least 4 killed); the experience never counts created stacks (`docs/formats/installation.md` §7.19, §7.21).
- Intended/expected behavior: every created stack's losses count for Necromancy, if they count at all (HotA treats Necromancy from clones as a bug; E08–E10 classify Necromancy from created stacks as a whole).
- Impact: gameplay-altering (small: fewer Skeletons after a battle against reused summons)
- Community status: unknown (no report found; not searched)
- Sources: original-data, `docs/formats/installation.md` §7.19; observation `experiments/results/E04b-batch-2d.md` SU a (SoD 4.0 Complete, plain HD, 2026-10-08)
- Flag: none (the outcome reports the stacks the battle's slots hold at its end, as the original's records)
- Regression test: `crates/h3-battle/src/tests/created.rs`, `the_outcome_reports_each_created_stack_with_its_losses`; the records that are not reused: `a_created_stack_whose_rounds_ran_out_keeps_its_record`, `created_stacks_whose_rounds_ran_out_count_towards_the_limit`, `with_quirk_bat_quirk_7_a_vanished_original_keeps_its_record`
- Decided by / date: user, 2026-10-08 (E04b increment 9, BAT-Q13)

### BAT-QUIRK-12 — A Pit Lord's raise with no free stack record crashes (E04b T7b3)
- Class: exploit-fix
- Context / rules affected: battle, BAT-MAG-052 (with BAT-MAG-037)
- Original behavior: the raise 0x5A7390 creates the Demons through 0x47A100, which takes the first of the side's records 0–19 that is empty or holds a dead summoned stack with no creatures; with 20 records in use and none of them such (summoned stacks and clones alive or vanished at the end of their rounds, the army and war machines), it creates nothing and returns 0, and 0x5A7390 then writes the army slot through that null pointer: the game crashes. The cursor (0x4473D0, 0x4476C0) does not test the side's records (`docs/formats/installation.md` §7.21).
- Intended/expected behavior: no raise without a record for the new stack; the engine refuses the raise then (its target test).
- Impact: game-breaking (a crash; reachable with many summoned stacks of one side)
- Community status: unknown (not searched)
- Sources: original-data, `docs/formats/installation.md` §7.21 ("The raisers and the end of the battle")
- Flag: none
- Regression test: `crates/h3-battle/src/tests/raisers.rs`, `a_pit_lord_needs_a_slot_for_the_new_stack`, `a_pit_lord_finds_no_slot_in_a_clone_whose_rounds_ran_out`
- Decided by / date: agent, 2026-10-08 (state-corrupting bugs are always fixed, AGENTS.md; the fix's shape, a refused raise, as Clone's and Summon Elemental's limit; BAT-Q13 noted it)

### BAT-QUIRK-13 — Enchanters judge a spell by the first stack of its side only (E04b T7b4; survey S-12)
- Class: fix-optional (user, 2026-10-09, BAT-Q14)
- Context / rules affected: battle, BAT-MAG-053 (with BAT-MAG-050's usefulness)
- Original behavior: before its cast an Enchanter counts each spell of its table that the Master Genie's usefulness test (0x447A80) accepts on a stack of the side the spell aims at (its own, the enemy's for Slow and Weakness); the loop calls the test once per record of that side but never advances past the first record (0x447FE0, `docs/formats/installation.md` §7.21), so only the side's first stack decides. A shooter there rules out Bloodlust, a full-health one Cure, and a dead one every spell aimed at that side: once the first stack dies its side's Enchanters cast nothing more on it.
- Intended/expected behavior: a spell counts if it is useful on any stack of that side (strict-fixed; HotA fixed it, wiki).
- Impact: gameplay-altering (small to moderate: whether and what Enchanters cast)
- Community status: known (wiki Enchanter: "the spell relevance is calculated based on stats of the first stack only", fixed in HotA)
- Sources: original-data, `docs/formats/installation.md` §7.21 ("The Enchanter, re-read for T7b4"); wiki https://heroes.thelazy.net/index.php/Enchanter
- Flag: `quirk.battle.periodic_first_stack` (E04b T7b4)
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-battle/src/tests/enchanters.rs`, `only_the_first_stack_decides_with_quirk_bat_quirk_13`
- Decided by / date: user, 2026-10-09 (E04b T7b4, BAT-Q14)

### BAT-QUIRK-14 — The Enchanters' counters carry over from battle to battle (E04b T7b4)
- Class: exploit-fix
- Context / rules affected: battle, BAT-MAG-053
- Original behavior: each side's counter (+0x132A0) rises by 1 at every round start and returns to 0 after its Enchanters' cast, which needs it above 2; nothing sets it when a battle starts, because the battle manager is made once per session (0x4EDCF5, 0x462760; `docs/formats/installation.md` §7.21). A battle's first Enchanter cast therefore depends on earlier battles: in round 1 after battles in which that side's Enchanters did not cast, later (up to round 4, when the round-start increments are the only ones) after one in which they did.
- Intended/expected behavior: a battle depends only on its own setup; every battle starts with both counters ready, so a side's Enchanters cast at their first activation (as after any battles without their cast, and as VCMI does), then once in three rounds.
- Impact: gameplay-altering (when Enchanters first cast); state leaking from one battle into the next
- Community status: unknown (not searched)
- Sources: original-data, `docs/formats/installation.md` §7.21 ("The Enchanter, re-read for T7b4"); VCMI 1.7.5 `server/battles/BattleFlowProcessor.cpp` (the counter starts at 0 and casts when it is 0)
- Flag: none
- Regression test: `crates/h3-battle/src/tests/enchanters.rs`, `every_battle_starts_with_the_enchanters_ready`
- Decided by / date: user, 2026-10-09 (E04b T7b4, BAT-Q14: ready at every battle's start)

### BAT-QUIRK-15 — The spell cost reads the armies, not the battle's stacks (E04b T7c1)
- Class: preserve, no flag: the rule in every profile (user, 2026-10-09, BAT-Q15)
- Context / rules affected: battle, BAT-MAG-054
- Original behavior: a hero's spell cost rises by 2 if the enemy's army holds Pegasi or Silver Pegasi and falls by 2 if the hero's own army holds Mages or Arch Mages (0x4E5240); the test (0x44AB80) reads the armies' seven slots, which keep their creature types until the battle's end, so a stack that died still counts and a stack created in the battle (a clone) never does (`docs/formats/installation.md` §7.21, "The mana creatures, re-read for T7c1").
- Intended/expected behavior: none decided; VCMI 1.7.5 counts the living stacks on the field (`CBattleInfoCallback.cpp`).
- Impact: gameplay-altering (small: a spell's cost by 2 after the stack died)
- Community status: unknown (not searched)
- Sources: original-data, `docs/formats/installation.md` §7.21; observation, batch 2a R6 round 3 (`experiments/results/E04b-batch-2a.md`: 5 with the Pegasus dead, VCMI 3)
- Flag: none
- Defaults: n/a
- Regression test: `crates/h3-battle/src/tests/mana.rs`, `a_dead_pegasus_still_raises_the_cost` (a created stack never counts: `mana::cost_change` reads `BattleSetup`'s armies only; the name was corrected at E04b T11a)
- Decided by / date: user, 2026-10-09 (E04b T7c1, BAT-Q15)

### BAT-QUIRK-16 — The Familiars' channel walks both sides' records up to one side's count (E04b T7c1)
- Class: exploit-fix
- Context / rules affected: battle, BAT-MAG-055
- Original behavior: after a hero's cast the channel (0x5A2170) counts the Familiars whose controller is not the caster's side, walking the records of both sides, each up to the record count of the side opposite the caster (+0x54BC read once, 0x5A21B2): a Familiar of the caster's side that the enemy hypnotized is missed when its record lies beyond that count, and when the caster's side has fewer records the loop reads records past its count, which may hold anything an earlier battle left there (`docs/formats/installation.md` §7.21).
- Intended/expected behavior: every living Familiar the other side controls counts.
- Impact: gameplay-altering (tiny: a hypnotized Familiar's channel); reads records not in use
- Community status: unknown (not searched; the user's rule for unknown bugs, BAT-Q15)
- Sources: original-data, `docs/formats/installation.md` §7.21
- Flag: none
- Defaults: n/a
- Regression test: `crates/h3-battle/src/tests/mana.rs`, `a_hypnotized_familiar_channels_for_its_controller_wherever_its_record`
- Decided by / date: user, 2026-10-09 (E04b T7c1, BAT-Q15)

### BAT-QUIRK-17 — A sacrificed Phoenix comes back at the next damaging action (E04b T7c2b)
- Class: exploit-fix (for now; the user, BAT-Q18: a later fix may rebirth the Phoenix right after the sacrifice)
- Context / rules affected: battle, BAT-MAG-059, BAT-MAG-036
- Original behavior: the rebirth check 0x469440 runs after every damaging action (strikes, damage spells, Land Mine, Fire Wall) and walks every record, but Sacrifice (0x5A199C) kills its victim without calling it and leaves its casts: a sacrificed Phoenix (dead, a cast left, no corpse) comes back at the next damaging action of either side, at a fifth of its count at the battle's start, placed by 0x4687C0 on its old hexes, which overwrites whatever stack stands there by now (`docs/formats/installation.md` §7.21).
- Intended/expected behavior: a sacrificed stack is consumed (Sacrifice's victim leaves no corpse and is not raised by anything else).
- Impact: exploit (the hit points of the sacrifice and a fifth of the Phoenixes back); state corruption (two stacks on one hex)
- Community status: unknown (not searched)
- Sources: original-data, `docs/formats/installation.md` §7.21 ("The Phoenix's rebirth, read for T7c2b")
- Flag: none (always fixed: a stack without a corpse is never reborn)
- Defaults: n/a
- Regression test: `crates/h3-battle/src/tests/rebirth.rs`, `a_sacrificed_phoenix_stays_dead_after_the_next_strike`, `clones_and_sacrificed_phoenixes_are_never_reborn`
- Decided by / date: user, 2026-10-09 (E04b T7c2b, BAT-Q18)

### BAT-QUIRK-18 — A hypnotized Azure Dragon frightens the side that controls it (E04b T7c3)
- Class: preserve, no flag: the rule in every profile (user, 2026-10-09, BAT-Q18)
- Context / rules affected: battle, BAT-MAG-060
- Original behavior: Fear (0x464D40) takes the frightened stack's side from its controller (+0xF4, flipped while hypnotized) but counts the Azure Dragons in the other side's records by real side: an Azure Dragon the enemy hypnotized still frightens the enemy's stacks, which now fight beside it, and a stack hypnotized by the enemy can be frightened by its own side's dragons (`docs/formats/installation.md` §7.21).
- Intended/expected behavior: none decided; arguably the dragons' controller should count.
- Impact: gameplay-altering (tiny)
- Community status: unknown
- Sources: original-data, `docs/formats/installation.md` §7.21 ("Fear, read for T7c3")
- Flag: none
- Defaults: n/a
- Regression test: `crates/h3-battle/src/tests/fear.rs`, `a_hypnotized_azure_frightens_the_side_that_controls_it`
- Decided by / date: user, 2026-10-09 (E04b T7c3, BAT-Q18)

### BAT-QUIRK-19 — The Ice, Magma and Energy Elementals have the wrong level (E04b T8e)
- Class: fix-optional (survey S-05; user, 2026-10-10, BAT-Q25)
- Context / rules affected: content (creature levels); BAT-MAG-004 and BAT-MAG-020 (a spell specialist's bonus by the target's creature level), later every level-based rule of the adventure (growth, random monsters)
- Original behavior: the executable's creature table gives the Ice Elemental level 4, the Magma Elemental level 3 and the Energy Elemental level 5, where each upgrade has its base creature's level (Water 3, Earth 5, Fire 4); the Storm Elemental is right. Level-based calculations use these levels.
- Intended/expected behavior: each upgrade has its base creature's level, 3, 5 and 4 (fixed in Horn of the Abyss).
- Impact: gameplay-altering (small)
- Community status: on the wiki (the Ice, Magma and Energy Elemental pages), fixed in Horn of the Abyss
- Sources: original-data, the executable's creature table (`docs/formats/installation.md` §5); wiki, Ice Elemental (https://heroes.thelazy.net/index.php/Ice_Elemental, retrieved 2026-10-03) and the Energy and Magma Elemental pages; survey `docs/rules/surveys/E04-quirks.md` S-05
- Flag: `quirk.content.elemental_levels`, a data quirk (ADR-0007, amendment E04b T8e): the `ab` pack keeps the executable's levels and three `[[creature_override]]` entries gated by the flag give the base creatures' (the harvester's gap-fill `data_quirks.base_level`)
- Defaults: faithful = on (4, 3, 5), strict-fixed = off (3, 5, 4)
- Regression test: `crates/h3-pack/src/lib.rs`, `the_data_quirks_are_fixed_in_strict_fixed`
- Decided by / date: user, 2026-10-10 (E04b T8e, BAT-Q25)

### BAT-QUIRK-20 — A bound Harpy flies back and stays bound (survey S-14; E04b T7a)
- Class: exploit-fix (never reproduced, no flag: the user, 2026-10-08, BAT-Q9)
- Context / rules affected: battle, BAT-ATK-004, BAT-MAG-045
- Original behavior: after its attack a Harpy or Harpy Hag that moved flies back to its start hex unless it is dead, blind, petrified or paralysed (0x4789F8); Bind is not tested, so a Harpy that the Dendroids' retaliation bound returns all the same. The return is a move, and the move's release routine (0x43EE10) empties the returning stack's own bound-by list without ending its Bind: no binder's move or death releases it any more, and it stays bound, away from its binder, until Teleport, Dispel or the 255 rounds end the Bind (`docs/formats/installation.md` §7.20).
- Intended/expected behavior: a bound stack does not return: it stays where it struck, beside its binder, and is released as any bound stack (as Horn of the Abyss from 1.3.3 and VCMI have it).
- Impact: gameplay-altering (a stack that cannot move for the rest of most battles); an inconsistent state (a Bind that no binder holds)
- Community status: known only as a bug (the wiki's Harpy and Dendroid pages: "bound down forever", bugged in Shadow of Death and until Horn of the Abyss 1.3.3); no player uses it on purpose (the survey for BAT-Q9)
- Sources: original-data, `docs/formats/installation.md` §7.20 ("Strike and return"); wiki, Harpy and Harpy Hag (https://heroes.thelazy.net/index.php/Harpy_and_Harpy_Hag, retrieved 2026-10-08); VCMI 1.7.5 `server/battles/BattleActionProcessor.cpp` L337–L351 (no return with a Bind)
- Flag: none
- Defaults: n/a
- Regression test: `crates/h3-battle/src/tests/strike.rs`, `a_harpy_bound_or_blinded_by_the_retaliation_stays_where_it_struck`
- Decided by / date: user, 2026-10-08 (E04b T7a review, BAT-Q9: no flag, a bound stack does not return, in every profile); entered here at the E04b T11b review, 2026-10-12

### BAT-QUIRK-21 — A binder is listed on a stack its strike killed (E04b T7a)
- Class: exploit-fix (never reproduced, no flag: the user, 2026-10-08, BAT-Q8)
- Context / rules affected: battle, BAT-MAG-045 (with BAT-MAG-040)
- Original behavior: the strike's spell routine (0x440500) puts a Dendroid on the list of binders of the stack it struck without asking whether that stack still lives; for a stack the strike killed no cast follows, but the entry stays until the striker moves or dies. A strike on a stack the striker already binds does nothing, so if the dead stack is raised meanwhile, that Dendroid's next strike does not bind it, although it carries no Bind (`docs/formats/installation.md` §7.20).
- Intended/expected behavior: only a living stack is bound: a strike that kills its target binds nothing, and a raised stack is bound by the next strike as any other.
- Impact: gameplay-altering (tiny: one strike's Bind on a raised stack); an inconsistent state (a binder list that names a stack without the effect)
- Community status: unknown (not searched)
- Sources: original-data, `docs/formats/installation.md` §7.20 ("Not reproduced (BAT-Q8)")
- Flag: none
- Defaults: n/a
- Regression test: `crates/h3-battle/src/tests/strike.rs`, `shots_cast_nothing_and_a_killed_target_takes_no_spell` (a killed target takes no strike spell) and `a_stack_a_binders_strike_killed_is_bound_by_its_next_strike_once_raised` (2026-10-19: the binder's strike kills a stack, which has no binders; raised by Resurrection, it is bound by that binder's next strike; the original's listing of the binder on the dead stack fails it)
- Decided by / date: user, 2026-10-08 (E04b T7a checkpoint, BAT-Q8: not reproduced, the engine binds only a living stack); entered here at the E04b T11b review, 2026-10-12

### BAT-QUIRK-22 — Earthquake disables the first defending stack (survey S-08; E04c T6)
- Class: exploit-fix (no flag: the user, 2026-10-14, BAT-Q43)
- Context / rules affected: battle, BAT-SIE-015 (with BAT-SIE-010)
- Original behavior: Earthquake's draw (0x5A7C80) has no fort test, so at a Fort or Citadel the towers and the keep the fort level does not set up (2 hit points each from `WALLS.TXT`) are drawn too; destroying such an absent turret (0x465990) marks dead the stack its turret record names, an index among the defender's stacks that no turret set up, so the defender's first stack disappears and stops acting (`docs/formats/installation.md` §7.29; the wiki's Earthquake bug, fixed in HotA).
- Intended/expected behavior: the absent parts stay in the draw and soak hits (so the real walls keep the original's odds), and their destruction harms no stack.
- Impact: gameplay-altering (a defending stack lost to a spell that hits walls); an exploit for the attacker
- Community status: known bug (the wiki's Earthquake page), fixed in HotA
- Sources: original-data (`installation.md` §7.29), wiki (Earthquake), observation (`experiments/results/E04c-batch-3d.md` E: the defender's Earthquake destroyed both absent towers of a Citadel and its first stack was marked dead)
- Flag: none
- Defaults: n/a
- Regression test: `crates/h3-battle/src/tests/walls.rs`, `an_absent_turret_quaked_down_harms_no_stack`; with the turrets set up (E04c T7: an absent target holds none, BAT-SIE-017), `crates/h3-battle/src/tests/turrets.rs`, `a_destroyed_tower_or_keep_takes_its_turret_down`
- Decided by / date: user, 2026-10-14 (BAT-Q43: the absent parts soak hits, no stack is harmed)

### BAT-QUIRK-23 — No Ballista for the defender at a town with a fort (E04c T9; survey D-22)
- Class: fix-optional (user, 2026-10-15, BAT-Q48)
- Context / rules affected: battle, BAT-SIE-029 (with BAT-WAR-001, BAT-SIE-009)
- Original behavior: the battle's setup 0x463600 sets up, in a battle with a fort level above 0, the attacker's Catapult and no Ballista for the defender, though the defending hero has one; its First Aid Tent and Ammo Cart are set up as elsewhere (`docs/formats/installation.md` §7.27). VCMI leaves it out at every town, with or without a fort ("bug #551").
- Intended/expected behavior: the defending hero's Ballista fights at every town, as it does everywhere else (strict-fixed).
- Impact: gameplay-altering (one war machine more or less for the defender of a fortified town)
- Community status: unknown (not searched); VCMI treats a defender's Ballista in a town as its bug #551 and leaves it out
- Sources: original-data (`installation.md` §7.27), vcmi (`lib/battle/BattleInfo.cpp` 1.7.5, `BattleInfo::setupBattle`)
- Flag: `quirk.battle.no_defender_ballista_at_fort` (registry position 25; E04c T9)
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-battle/src/tests/sieges.rs`, `with_bat_quirk_23_a_town_with_a_fort_sets_up_no_ballista_for_its_defender` (both settings); the invariant properties mix it into half their games
- Decided by / date: user, 2026-10-15 (E04c T9, BAT-Q48)

### BAT-QUIRK-24 — A berserk walk ends only beside the target's position hex (E23 T1c; survey S-11)
- Class: fix-optional (user, 2026-10-20, BAT-Q54)
- Context / rules affected: battle, BAT-MAG-025 (with BAT-MOV-003)
- Original behavior: the berserk turn (0x4222C0) searches with its pathfinder (0x4B3400) for the target's position hex, and the search ends only on a blocked step onto that hex (0x4B38A6; for a two-hex walker onto the hex beside its step, 0x4B39BB), though the candidates' marks count both hexes of a two-hex target (`docs/formats/installation.md` §7.41). So a two-hex target reachable only by its other hex is still chosen, but the walker goes round to a position beside its position hex, or does nothing: when no such position is free within the search's range, or when the target's mark (through its other hex) is within its speed and that position is not (the move to it fails, 0x445950).
- Intended/expected behavior: the walk ends next to either of the target's hexes, as HD and HotA do (strict-fixed).
- Impact: gameplay-altering (a berserk two-hex fight's turns)
- Community status: the wiki's Berserk page lists it ("two-hex units miscount positions and pass although a target is reachable"; fixed in the HD mod and HotA)
- Sources: original-data (`installation.md` §7.41), wiki (Berserk; survey S-11)
- Flag: `quirk.battle.berserk_walks_to_position_hex` (registry position 26; E23 T1c)
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-battle/src/tests/control.rs`, `with_bat_quirk_24_a_berserk_walker_attacks_a_two_hex_stack_only_from_beside_its_position` (both settings)
- Decided by / date: user, 2026-10-20 (E23 T1c, BAT-Q54)

### BAT-QUIRK-25 — The original computer's slips in its own estimates (E23 T1b; survey E23 §4)
- Class: fix-optional (user, 2026-10-20, BAT-Q55: one flag for all the fixes of the original's AI)
- Context / rules affected: battle, the computer's choices (ADR-0027 point 9; `BAT-AI-nnn`, E23 T2 onwards; the policy crate `h3-battle-ai` and the computer's numbers in `h3-battle`)
- Original behavior: where the computer estimates a rule's outcome, it sometimes reads a wrong record, argument, spell, side, mastery or test: the walls test's records, Magic Mirror's doubled chance, Paralyze with Blind's record, Disease with Weakness's resistance, the protections with the own hero's bonuses, and the rest of `docs/rules/surveys/E23-battle-ai.md` §4 (17 so far; `docs/formats/installation.md` §7.35 to §7.46). They change which command the computer picks, never a rule's outcome.
- Intended/expected behavior: each estimate as the rule it estimates (the survey's "Fixed" column; strict-fixed).
- Impact: gameplay-altering (the computer side's choices)
- Community status: unknown (the wiki describes none of them)
- Sources: original-data (`installation.md` §7.35 to §7.46; the blind re-reads of T1a5e)
- Flag: `quirk.battle.original_ai_slips` (registered with the first slip built, E23 T2 or later; a slip found later joins it)
- Defaults: faithful = on, strict-fixed = off
- Regression test: each slip's rule tests both settings (with its task)
- Decided by / date: user, 2026-10-20 (E23 T1b, BAT-Q55)
