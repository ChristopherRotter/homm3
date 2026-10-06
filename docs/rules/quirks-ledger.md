# Quirks ledger

Every known difference between the original game's behavior and a clean design. See `docs/04-rules-and-sources.md` for classes, the decision guide and profiles.

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

### Q-0005 — Damage and surrender rates are single-precision values
- Class: fix-optional (proposed at the batch-1a report and confirmed by the user, 2026-10-03; survey S-02 said "decide with Q23")
- Context / rules affected: battle, BAT-DMG-002 and every damage factor with a rate that is not a binary fraction (attack and defense points, skills, specialties, artifacts; E04a T6); since batch 1b also the surrender price's discount (BAT-FLE-002, E04a T9b); since E04b the spell damage bonuses (Sorcery; BAT-MAG-004, T2b/T2d) and magic resistance (BAT-MAG-008, HER-BAT-013, T2d)
- Original behavior: the damage factors use the binary single-precision values of their rates: 0.05 per attack point is 13421773/2^28 (slightly more than 5 %), 0.025 per defense point 13421773/2^29, Basic Armorer's 0.05 likewise, and the product is rounded down once. So a damage that is an exact integer with decimal rates can come out one lower (Angels with 24 defense points against them: 19 instead of 20; 100 base against Basic Armorer: 94 instead of 95) or stay exact (attack bonuses err upwards). Observed in R1–R3 (18 values, one model). The surrender price does the same with its discount (batch 1b R4): half of 10 Pikemen (300 gold) with Basic Diplomacy, Statesman's Medal and Diplomat's Ring (40 %) costs 179, because 0.4 is 0.4000000059604645 in single precision (exact: 180).
- Intended/expected behavior: exact decimal rates (damage: our BAT-DMG-002 with the flag off; surrender: 180 in the example).
- Impact: gameplay-altering (tiny: at most one damage point per strike)
- Community status: the Armorer case is on the wiki (Damage page); the cause was unknown; the wiki's Armorer example (2002 → 2001) is not explained by the single-precision rates alone (noted at batch 1a; not reproduced, not staged)
- Sources: observation, `experiments/results/E04a-batch-1a.md` R1–R3 (SoD 4.0 Complete, 2026-10-03) and `experiments/results/E04a-batch-1b.md` R4 (2026-10-04); original-data, the executable's skill tables are f32 (`docs/formats/installation.md` §7.4); wiki, Damage (https://heroes.thelazy.net/index.php/Damage, retrieved 2026-10-01)
- Flag: `quirk.battle.single_precision_rates` (registered by E04a T6a; computed exactly in integers, ADR-0003: `h3_kernel::arith::single_precision` gives the binary32 value of a rate as an exact fraction over 2^40, and the damage product is exact with one final floor). Every rate and cap of a damage factor is converted, counts multiply the converted rate (22 points: 22 × 13421773/2^28); not observed and chosen for consistency: the 70 % defense cap (0.69999999), jousting's 5 % per hex; a hero's summed skill, artifact and specialty bonus enters as one rate (E04a T6b); the summed surrender discount enters as one rate (E04a T9; the observation cannot tell per-source from summed conversion, both give 179). Spells (E04b T2d, from the code, `docs/formats/installation.md` §7.8; user, 2026-10-06, Q44): the Orb and Sorcery factors are single precision and their product is rounded down once; the hero's resistance sources are added as single-precision values, each sum stored in single precision, and subtracted from the creature's share (stored likewise), so 10 % hero resistance leaves an 89 % chance to be affected and 5 % leaves 94 % (exact: 90 % and 95 %); not observed yet (batch 2a)
- Defaults: faithful = on, strict-fixed = off
- Regression test: R1–R3 as fixtures (`crates/h3-battle/src/tests/rules.rs`, `single_precision_rates_reproduce_batch_1a`; the flag in a battle: `the_quirk_flag_selects_the_single_precision_rates`; the surrender discount: `crates/h3-battle/src/tests/flee.rs`, `the_discount_enters_as_a_single_precision_rate_under_q_0005`; spells: `crates/h3-battle/src/tests/magic.rs`, `the_casters_bonuses_apply_in_the_originals_order`, and `crates/h3-battle/src/tests/spells.rs`, `resistance_adds_the_heros_to_the_creatures_and_the_aura_multiplies`)
- Decided by / date: confirmed by experiment batch 1a (Q23), 2026-10-03; class fix-optional confirmed by the user, 2026-10-03; extended to the surrender discount by the user (batch 1b, Q32), 2026-10-04; extended to magic resistance by the user (Q44), 2026-10-06

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
