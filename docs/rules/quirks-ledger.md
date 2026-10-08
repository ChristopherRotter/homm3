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
- Context / rules affected: battle, BAT-DMG-002 and every damage factor with a rate that is not a binary fraction (attack and defense points, skills, specialties, artifacts; E04a T6); since batch 1b also the surrender price's discount (BAT-FLE-002, E04a T9b); since E04b the spell damage bonuses (Sorcery; BAT-MAG-004, T2b/T2d), magic resistance (BAT-MAG-008, HER-BAT-013, T2d) and Slow's speed share (BAT-MAG-011, T3a)
- Original behavior (**since E04b T3b1, Q50**: physical damage after the executable, `docs/formats/installation.md` §7.10): damage is computed in three truncated stages in x87 arithmetic (bonuses as an integer, each fractional one truncated as it is added; times the defense and penalty factor; times the protection factor of Shield or Air Shield and Armorer, at least 1); the attack and defense rates are doubles (0.05 and 0.025 slightly above the decimals, the defense factor stored as a double), skill rates and the spells' stored factors single precision; the x87 rounds every operation to 53 bits (Windows' default; batch 2b R1: 195 and 390, at 64 bits 194 and 389). Both this and the earlier model below fit R1–R3; they differ by one or two points in about half the strikes that combine a bonus or defense with Armorer (59 base, one attack point, Basic Armorer: 57, rounded once 58). *Earlier model (E04a T6a, superseded for physical damage):* the damage factors use the binary single-precision values of their rates: 0.05 per attack point is 13421773/2^28 (slightly more than 5 %), 0.025 per defense point 13421773/2^29, Basic Armorer's 0.05 likewise, and the product is rounded down once. So a damage that is an exact integer with decimal rates can come out one lower (Angels with 24 defense points against them: 19 instead of 20; 100 base against Basic Armorer: 94 instead of 95) or stay exact (attack bonuses err upwards). Observed in R1–R3 (18 values, one model). The surrender price does the same with its discount (batch 1b R4): half of 10 Pikemen (300 gold) with Basic Diplomacy, Statesman's Medal and Diplomat's Ring (40 %) costs 179, because 0.4 is 0.4000000059604645 in single precision (exact: 180).
- Intended/expected behavior: exact decimal rates, rounded once (damage: our BAT-DMG-002 with the flag off; surrender: 180 in the example).
- Impact: gameplay-altering (tiny: at most one damage point per strike)
- Community status: the Armorer case is on the wiki (Damage page); the cause was unknown; the wiki's Armorer example (2002 → 2001) is not explained by the single-precision rates alone (noted at batch 1a; not reproduced, not staged)
- Sources: observation, `experiments/results/E04a-batch-1a.md` R1–R3 (SoD 4.0 Complete, 2026-10-03), `experiments/results/E04a-batch-1b.md` R4 (2026-10-04) and `experiments/results/E04b-batch-2b.md` R1, R4, R6 (plain HD, 2026-10-07: the staged damage at 53 bits, Shield and Armorer in one product, Armorer's single-precision rate, a specialist's grown rate); original-data, the executable's skill tables are f32 (`docs/formats/installation.md` §7.4); wiki, Damage (https://heroes.thelazy.net/index.php/Damage, retrieved 2026-10-01)
- Flag: `quirk.battle.single_precision_rates` (registered by E04a T6a; computed exactly in integers, ADR-0003: `h3_kernel::arith::single_precision` gives the binary32 value of a rate as an exact fraction over 2^40, and the damage product is exact with one final floor). Every rate and cap of a damage factor is converted, counts multiply the converted rate (22 points: 22 × 13421773/2^28); not observed and chosen for consistency: the 70 % defense cap (0.69999999), jousting's 5 % per hex; a hero's summed skill, artifact and specialty bonus enters as one rate (E04a T6b); the summed surrender discount enters as one rate (E04a T9; the observation cannot tell per-source from summed conversion, both give 179). Spells (E04b T2d, from the code, `docs/formats/installation.md` §7.8; user, 2026-10-06, Q44): the Orb and Sorcery factors are single precision and their product is rounded down once; the hero's resistance sources are added as single-precision values, each sum stored in single precision, and subtracted from the creature's share (stored likewise), so 10 % hero resistance leaves an 89 % chance to be affected and 5 % leaves 94 % (exact: 90 % and 95 %); not observed yet (batch 2a); Slow's speed share (E04b T3a, `installation.md` §7.9) is stored as a single-precision value and the slowed speed rounded down (Slow's 75 % and 50 % are binary fractions, so only a data spell with another share differs: 70 % of speed 10 gives 6, exact 7; BAT-MAG-011)
- Defaults: faithful = on, strict-fixed = off
- Regression test: R1–R3 as fixtures (`crates/h3-battle/src/tests/rules.rs`, `the_staged_damage_reproduces_batch_1a`; batch 2b R1, R3, R4, R6: `the_staged_damage_reproduces_batch_2b`, Mephala's rate from the harvested packs: `crates/h3-pack/src/lib.rs`, `harvested_magic_derives`; the stages: `the_stages_truncate_each_bonus_and_factor`; the flag in a battle: `the_quirk_flag_selects_the_staged_damage`; the x87 arithmetic: `crates/h3-kernel/src/arith.rs`, `x87_operations`, and `crates/h3-kernel/src/num.rs`, `x87_at_53_bits_is_double_arithmetic`; Shield and the Protections: `crates/h3-battle/src/tests/protect.rs`; the surrender discount: `crates/h3-battle/src/tests/flee.rs`, `the_discount_enters_as_a_single_precision_rate_under_q_0005`; spells: `crates/h3-battle/src/tests/magic.rs`, `the_casters_bonuses_apply_in_the_originals_order`, and `crates/h3-battle/src/tests/spells.rs`, `resistance_adds_the_heros_to_the_creatures_and_the_aura_multiplies`; Slow: `crates/h3-battle/src/tests/timed.rs`, `slow_rounds_down_and_leaves_at_least_one`)
- Decided by / date: confirmed by experiment batch 1a (Q23), 2026-10-03; class fix-optional confirmed by the user, 2026-10-03; extended to the surrender discount by the user (batch 1b, Q32), 2026-10-04; extended to magic resistance by the user (Q44), 2026-10-06; physical damage staged after the executable's code by the user (Q50), 2026-10-06; the grown skill rates (specialists) and the skill getters' artifact sums cross the hero seam in the original's single precision, computed by the derivation (Q45 option (b), the user 2026-10-06; built in E04b T3c: `h3_contracts::Rate`, HER-BAT-004/005/012/013, the tent's heal BAT-WAR-007)

### Q-0006 — Roland's specialty affects only Crusaders
- Class: fix-optional (survey S-03, suggested class; confirmed for Roland only)
- Context / rules affected: content (hero specialties), the creature specialty rule (E04a T10)
- Original behavior: the executable's specialty table names the Crusader (creature 7) for Roland, so his specialty raises only Crusaders, although it is presented as a Swordsman specialty. Observed at hero level 4: Roland's Crusaders get +1/+1/+1, his Swordsmen nothing; Catherine's and Sorsha's specialty (the Swordsman) raises both. The wiki's claim that Catherine's is bugged as well does not hold in SoD.
- Intended/expected behavior: Swordsmen and Crusaders (HotA fixed it).
- Impact: gameplay-altering (a campaign hero)
- Community status: on the wiki (Swordsman, Hero specialty pages; Catherine mis-attributed), fixed in Horn of the Abyss
- Sources: observation, `experiments/results/E04a-batch-1a.md` B4–B6 (SoD 4.0 Complete, 2026-10-03); original-data, the specialty table (`docs/formats/installation.md` §7.2); wiki, Hero specialty (https://heroes.thelazy.net/index.php/Hero_specialty, retrieved 2026-10-03)
- Flag: none yet; a quirk in data, so it needs the same mechanism as the elementals' levels (survey S-05): a per-profile data override (profile-level Tier-2 overrides, deferred by ADR-0006), built with E04b (user, OPEN-QUESTIONS Q35). Until then the packs hold the executable's value (faithful).
- Defaults: faithful = on (Crusader), strict-fixed = off (Swordsman), effective once the override exists (E04b); until then both profiles keep the Crusader, the one difference of the profiles not yet built (E04a exit criterion 3)
- Regression test: `crates/h3-pack/src/lib.rs`, `harvested_hero_effects_reproduce_the_experiments` (B4–B6 with the harvested packs; E04a T10a)
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
- Sources: original-data, `docs/formats/installation.md` §7.11; VCMI 1.7.5 (`lib/battle/DamageCalculator.cpp`, the damage after all reductions); observation, `experiments/results/E04b-batch-2b.md` R3 (SoD 4.0 Complete, plain HD, 2026-10-07): 10 Naga Queens deal 150 (first stage 300) to Fire-Shielded Peasants, "The Fire Shield does 60 damage." (of the damage dealt: 30)
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
- Context / rules affected: battle, BAT-MAG-026, BAT-MAG-003
- Original behavior: Expert Cure loops over the caster's own stacks and skips those under Hypnotize (0x5A1847), as the mass handler of every other mass version does (0x5A66D0), so it never frees a hypnotized stack; a single Cure from its owner reaches it (the side tests use the real side) and ends Hypnotize.
- Intended/expected behavior: Expert Cure also cures the caster's hypnotized stacks and ends their Hypnotize (strict-fixed: a mass version whose dispel ends the stack's Hypnotize reaches it; the other mass versions still pass over hypnotized stacks).
- Impact: gameplay-altering (small)
- Community status: unknown ("may/may not be a bug", wiki Cure)
- Sources: original-data, `docs/formats/installation.md` §7.14; wiki Cure (survey S-13); observation `experiments/results/E04b-batch-2c.md` H1 (SoD 4.0 Complete, plain HD, 2026-10-07: Hypnotize still listed after Expert Cure, which healed the caster's other Swordsmen to 35) and C1 (a Basic Cure ends it)
- Flag: `quirk.battle.mass_cure_skips_hypnotized` (E04b T4c)
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-battle/src/tests/control.rs`, `mass_versions_pass_over_hypnotized_stacks_and_expert_cure_only_with_q0012`
- Decided by / date: user, 2026-10-07 (E04b T4a checkpoint, Q53)

### BAT-QUIRK-1 — The natives' sight of hidden obstacles is fixed at the battle's start
- Class: fix-optional (user, 2026-10-07, BAT-Q1)
- Context / rules affected: battle, BAT-MAG-031 (ADR-0014 point 7, amended)
- Original behavior: at the battle's start the original sets one byte per side (+0x1329C, 0x46402C): whether any of its stacks fights on its native terrain. When Quicksand or a Land Mine is placed, the object is revealed to both sides if the other side's byte is set (0x5A0442). Nothing writes the byte again: a side whose natives die keeps seeing what is placed later, and stacks created later change nothing.
- Intended/expected behavior: a side sees the other side's hidden obstacles while it has a living stack on its native terrain (VCMI's `nativeVisible`; strict-fixed), so its sight comes and goes with its natives.
- Impact: gameplay-altering (small)
- Community status: unknown (the wiki describes the natives' sight without timing)
- Sources: original-data, `docs/formats/installation.md` §7.16; VCMI 1.7.5 `lib/battle/CObstacleInstance.cpp` (survey E04b-magic)
- Flag: `quirk.battle.natives_sight_at_start` (E04b T5c1)
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-battle/src/tests/pits.rs`, `natives_at_the_start_reveal_the_other_sides_pits_with_bat_quirk_1`, `living_natives_see_the_other_sides_pits_without_bat_quirk_1`
- Decided by / date: user, 2026-10-07 (E04b checkpoint before T5c, BAT-Q1)

### BAT-QUIRK-2 — An attack from a seen Quicksand pit is accepted and skipped
- Class: fix-optional (user, 2026-10-07, BAT-Q1)
- Context / rules affected: battle, BAT-MAG-032
- Original behavior: the walker (0x441FA0) stops at a step into a hex its side's stops mark (a pit it sees), also when that hex is the move's destination, and returns false; a move and attack (0x478971) then skips the attack. A command to attack from a seen pit therefore moves the stack into the pit and does nothing else.
- Intended/expected behavior: such an attack is not legal (strict-fixed), as it can never happen; the plain move into the pit stays legal.
- Impact: cosmetic for players (the original's interface may not offer it; not read), relevant for agents (an action that never does what it names)
- Community status: unknown
- Sources: original-data, `docs/formats/installation.md` §7.16
- Flag: `quirk.battle.attack_from_seen_pit` (E04b T5c1)
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-battle/src/tests/pits.rs`, `an_attack_from_a_seen_pit_is_legal_only_with_bat_quirk_2`
- Decided by / date: user, 2026-10-07 (E04b checkpoint before T5c, BAT-Q1)

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

### BAT-QUIRK-5 — Sacrifice tests the stack it raises as Resurrection would
- Class: fix-optional (user, 2026-10-08)
- Context / rules affected: battle, BAT-MAG-036 (with BAT-MAG-035)
- Original behavior: Sacrifice finds and tests the stack it raises through Resurrection's routine (0x5A3950 calls 0x5A3CC0, which asks 0x5A8090 about spell 38): Resurrection's conditions, its level 4 against spell-level immunities and Anti-Magic (Gold Dragons, immune to levels 1–4, are never raised by Sacrifice; survey S-15), and Resurrection's hit points at the caster's Earth mastery, which must reach one creature's health, although Sacrifice raises its own amount.
- Intended/expected behavior: the target is tested with Sacrifice's own conditions and level 5, without a hit-point minimum (strict-fixed).
- Impact: gameplay-altering (small)
- Community status: the wiki's Gold Dragon trivia notes the Gold Dragons' half (S-15); the hit-point test is undocumented
- Sources: original-data, `docs/formats/installation.md` §7.18; wiki, Gold Dragon page (https://heroes.thelazy.net/index.php/Gold_Dragon, retrieved 2026-10-03)
- Flag: `quirk.battle.sacrifice_tests_as_resurrection` (E04b T6a)
- Defaults: faithful = on, strict-fixed = off
- Regression test: `crates/h3-battle/src/tests/raise.rs`, `with_bat_quirk_5_sacrifice_tests_its_target_as_resurrection`
- Decided by / date: user, 2026-10-08 (E04b T6a)

### BAT-QUIRK-6 — A winner left without an army (survey S-20)
- Class: exploit-fix
- Context / rules affected: battle, BAT-MAG-035, BAT-END-003
- Original behavior: creatures raised by Basic Resurrection fight on and may win the battle, then vanish. The end of the battle (0x4774E2, `docs/formats/installation.md` §7.18) keeps one creature in the last stack that had any when no creatures are left, but its total counts war machines and summoned stacks or clones, which the army does not keep: a hero with a Ballista (or a living elemental) whose stacks were all raised so is left without an army (HotA changelog: "a defending hero could be left without an army after winning a combat if all his stacks had been resurrected with a Basic-level Resurrection spell").
- Intended/expected behavior: the total counts only army stacks: an army whose living creatures are all temporary keeps one in its last living stack whatever else survives (BAT-END-003, BAT-MAG-035; the user, 2026-10-08).
- Impact: exploit (a hero without an army breaks an invariant)
- Community status: fixed in Horn of the Abyss
- Sources: original-data, `docs/formats/installation.md` §7.18; HotA changelog (survey `docs/rules/surveys/E04-quirks.md` S-20)
- Flag: none
- Regression test: `crates/h3-battle/src/tests/raise.rs`, `a_side_left_with_creatures_raised_for_the_battle_only_keeps_one`; the property `outcomes_account_for_every_stack`
- Decided by / date: user, 2026-10-08 (E04b T6a)

### BAT-QUIRK-7 — A clone placed on its own original (E04b T6b)
- Class: exploit-fix
- Context / rules affected: battle, BAT-MAG-038
- Original behavior: Clone's place search (0x5A6C70, `docs/formats/installation.md` §7.19) tests each hex with 0x446C40, which counts the original's own hexes as free. Around a crowded two-hex original (every place in ring 1 taken), ring 2 can then pick the original's head as the clone's position: the stack creation (0x47A100) refuses the occupied hex and creates nothing, and the cast goes on to flag *the original* as summoned with the clone's rounds and as its own clone, so it vanishes when the rounds run out and its army slot is lost after the battle; or it picks a position whose front hex is the original's, and the two stacks share a hex.
- Intended/expected behavior: the original's hexes are taken like any living stack's; the search goes on (BAT-MAG-038).
- Impact: game-breaking (a stack lost from the army, two stacks on one hex)
- Community status: not documented (found by code reading)
- Sources: original-data, `docs/formats/installation.md` §7.19
- Flag: none
- Regression test: `crates/h3-battle/src/tests/created.rs`, `a_crowded_two_hex_stack_is_never_covered_by_its_clone`; the property invariant that living stacks never share a hex (`tests/properties.rs`)
- Decided by / date: agent, 2026-10-08 (E04b T6b; state-corrupting bugs are always fixed, AGENTS.md), provisionally; the user wants the original's behaviour confirmed by experiment first (2026-10-08, T6 checkpoint; staged in batch 2d, CL2). The engine keeps the fix until then.
