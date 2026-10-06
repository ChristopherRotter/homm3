# E04b — Battle magic

Status: **in progress** (refined 2026-10-05; increment 1 = T1–T2; T1 done, ADR-0014 accepted; T2a–T2c done) · Follows E04a (closed 2026-10-05) · Part of the former E04 (OPEN-QUESTIONS Q27): E04a heroes in battle, **E04b** battle magic (this one), E04c sieges and special battlefields.

## Goal

Let heroes and creatures use magic in battle: the hero's spell book, mana and one cast per round; every battle spell of the Complete edition except Earthquake (E04c); timed effects on stacks; summoned, cloned and raised stacks; obstacle spells with hidden information; resistance and immunities; spell-like creature abilities and spellcasting creatures; the spell-related skills, artifacts and specialties. Casting is reachable from the sim and the C ABI through a factorized action encoding. The architecture stays that of E01–E04a: pure logic, validated commands, events with visibility, observations, determinism, at least 100 k steps/s/core.

## Decisions at the refinement (2026-10-05, user, OPEN-QUESTIONS Q41)

1. **One epic, delivered in increments.** This file keeps a coarse outline of all tasks; only the current increment's tasks are refined. Each increment ends at a checkpoint where the user confirms the next cut. **Increment 1 is T1 + T2**; T1 also stops for the review of ADR-0014.
2. **Casting enters the sim with the core (T2).** ADR-0014 fixes a factorized action shape (action kind → spell → target); T2 builds it in `h3-sim` and the C ABI, so random rollouts, benchmarks and golden replays exercise every later spell. E05 builds its batch API on it (BACKLOG inbox, 2026-10-01).
3. **The survey is just in time.** T1 surveys what ADR-0014 decides (effect kinds, targeting, durations, the resistance and immunity order, casting rules, new stacks, hidden information); each later task surveys the numbers and disputes of its own spells and abilities.

Unchanged from earlier decisions: all battle spells are in E04b, Earthquake goes with the sieges (Q27); a quirk is confirmed for SoD before it gets a flag (Q30), S-09 to S-21 are to confirm in E04b batches, S-14 and S-20 are exploit fixes and never implemented (`docs/rules/surveys/E04-quirks.md`); experiment maps are for SoD (Q28); token efficiency beats speed (AGENTS.md).

## Starting point (from E04a)

- `BattleHero.power`, `.knowledge` and `.magic` (`BattleMagic`: spell book, mana, maximum mana, school levels) are carried and checked since E04a and read by no rule (`h3_contracts` `hero.rs`). `derive_battle_hero` passes the mana through and leaves the rest neutral. The observation already shows each hero's power, knowledge and mana (public, ADR-0013 point 9).
- Packs (format v3): 4 spell schools and 81 spell data rows (`base` 80, `sod` Titan's Lightning Bolt): 10 adventure, 60 combat (59 without Earthquake), 11 creature abilities (Stone Gaze, Poison, Bind, Disease, Paralyze, Age, Death Cloud, Thunderbolt, Dispel Helpful Spells, Death Stare, Acid Breath), each with level, schools, cost and effect by mastery, power factor and polarity. The executable's spell flag bits are decoded since T1 (`docs/formats/installation.md` §7.6) but not harvested. The magic skills, Sorcery, Resistance and Eagle Eye exist without levels; 43 artifacts carry effects marked `# pending: E04b`; spell specialties carry only the spell (`packs/README.md`).
- Creature abilities (ADR-0012) cover E03's behaviour; no spell-like ability, immunity or resistance exists. The battle's modifier pool has side bearers only (`h3_battle::morale`, capacity 28); battle-long hero effects are applied once at setup (`Stack::new`).
- Versions: state hash v17, ruleset hash v13, replay format v13, action encoding v8 (1286 actions), observation encoding v4, C ABI v7.
- Notes left for E04b: `docs/epics/E04a-log.md`, "The design note at closure" (last bullet); the outline in `docs/rules/surveys/E04-battle.md` §7; the quirk entries S-09 to S-21; the BACKLOG inbox (profile-level overrides for Q-0006 and S-05, spell scrolls as artifact instances, the spell flags, the experiment-spec runner). The `.h3m` writer already writes a hero's spells (`h3_formats::h3m`); the experiment spec does not expose them yet.

## Non-goals

Earthquake, sieges and the moat (E04c). The special battlefields' magic (Magic Plains, Cursed Ground, the elemental terrains; E04c builds on E04b's mastery and spell-level rules). Adventure spells and learning spells (Mage Guild, Wisdom, Eagle Eye's learning, scrolls on the map; E08–E10): the battle reports what Eagle Eye needs, the orchestrator applies it. Maximum mana and its regeneration (Intelligence, Mysticism, wells): derived before the seam. Hero commands and progression (E08–E10). AI (E05/E06) and E05's batch API. The VCMI oracle harness stays deferred.

## Tasks (outline; refine at the start of each increment; keep status current)

**Increment 1:** T1 (done), T2 (refined below). Later increments are cut with the user at each checkpoint; order and boundaries of T3–T11 may change.

- [x] **T1 Magic survey and ADR-0014** — done 2026-10-05: survey `docs/rules/surveys/E04b-magic.md`, spell flags and stack records (`docs/formats/installation.md` §7.6, §7.7), ADR-0014 accepted after one revision (Q42; ADR-0004 point 8 amended). Record: [`E04b-log.md`](E04b-log.md).
- [ ] **T2 Casting core and direct-damage spells** — increment 1; split T2a–T2d confirmed by the user (2026-10-05), delivered as T2a + T2b, then T2c + T2d, each pair ending at a user checkpoint. T2a and T2b done 2026-10-05, T2c 2026-10-06 ([log](E04b-log.md)); the user confirmed T2c + T2d with batch 2a at the checkpoint (Q44).
- [ ] T3 Timed effects on stacks: buffs and debuffs, Mass versions at Expert, durations and the artifacts that change them, Dispel, Cure (S-16, S-17, S-19).
- [ ] T4 Control spells and spell defences: Berserk, Hypnotize, Forgetfulness, Blind; Anti-Magic, Magic Mirror (S-09 to S-11, S-13).
- [ ] T5 Battlefield spells: Quicksand and Land Mine (hidden, owner-side visibility), Force Field, Fire Wall, Remove Obstacle, Teleport.
- [ ] T6 Life and summons: Resurrection, Animate Dead, Sacrifice, Clone, the four Elementals; stacks created in battle (`StackOrigin`), temporary resurrection in the outcome (S-15, S-20, S-21).
- [ ] T7 Creature magic: spell-like attack effects (curse, blind, petrify, bind, dispel, weakness, age, poison, …), spellcasting creatures, creature effects on mana and costs (S-12, S-14).
- [ ] T8 Hero-side long tail: the remaining magic artifacts (opening spells, spell-level limits, the Tomes, spell scrolls as instances, "known spells need a worn spell book"), Eagle Eye's outcome inputs, profile-level overrides for data quirks (Q-0006, S-05).
- [ ] T9 Workload and measurements: casting heroes with the full spell range in the `full` scenario, golden replays, `bench-check`, state size and clone cost, observation completeness.
- [ ] T10 Experiments: batch 2a compiled at the end of increment 1 (direct-damage spells and the E04a leftovers, Q40 among them); later batches with the tasks they check; the experiment-spec runner (BACKLOG, Q37) when a batch needs predictions.
- [ ] T11 Closure: fold-in, independent review in a fresh session, docs.

Order: T1 first and alone; T2 after the review; T3 before T4–T7 (they use timed effects); T5–T8 independent of each other; T9 and T11 last; T10's batches follow the tasks they check.

### T2 — Casting core and direct-damage spells (refined; the split is confirmed at its start)

The 13 direct-damage spells: Magic Arrow, Ice Bolt, Lightning Bolt, Titan's Lightning Bolt (SoD), Implosion, Chain Lightning, Frost Ring, Fireball, Inferno, Meteor Shower, Death Ripple, Destroy Undead, Armageddon. Proposed checkpoints:

- [x] **T2a Content and the hero's magic** — done 2026-10-05: pack format v4 (target, area, chain, conditions, `damage` for the 13 spells), the flag word harvested, `school_mastery`, `spell_damage`, `spell_book`, `spell_specialty` growth; HER-BAT-011/012; ruleset hash v14. Record: [`E04b-log.md`](E04b-log.md). *As planned:* Pack format v4: effect kind and targeting of the 13 spells; the spell flags they need (harvested where §7.6 is decoded, otherwise gap-fill with sources). Hero effects for the four magic schools' levels, Sorcery, the spell specialties of these spells and the spell-damage artifacts (the Orbs); `derive_battle_hero` fills `BattleMagic` and the spell-damage bonuses. HER- rules; ruleset hash.
- [x] **T2b Casting in battle** — done 2026-10-05: `Command::Cast`, legality (BAT-MAG-001 to 003), single-target damage with the caster's bonuses (BAT-MAG-004), `castable_spells`/`cast_targets`, events, observation, outcome, the scenario and replay formats; state hash v18, ruleset hash v15, replay format v14 (action encoding unchanged until T2c). Record: [`E04b-log.md`](E04b-log.md). *As planned:* The hero's mana and "cast this round" in the state; the cast command (ADR-0014); legality (the side's turn, spell in the book, a combat spell, mana for the cost at the hero's mastery, at most once per round and never required, not in tactics, a target valid for the spell); the single-target spells with the damage formula (power × factor + effect by mastery; Sorcery, specialty and artifact bonuses; rounding as Q-0005 until batch 2a says otherwise); events with visibility; the observation (cast this round public, the spell book private); the outcome's mana per hero; replay format, state hash.
- [x] **T2c The sim** — done 2026-10-06: 21 stack slots per side (`SIDE_STACK_SLOTS`), the factorized action (`h3_sim::Action`, entries plus spell and target heads with conditioned masks; action encoding v9), C ABI v8 (`H3Action`, mask per head), observation encoding v5 (casts, masteries, spell book); both heroes of `full` cast, `full-s3` re-recorded, `bench-check` re-baselined. Record: [`E04b-log.md`](E04b-log.md). *As planned:* The factorized casting action in `h3-sim` and the C ABI (action encoding v9, C ABI v8), observation encoding v5; a casting hero in the `full` scenario, its golden replay re-recorded, `bench-check` re-baselined with the reason; state size and clone cost measured. Every later spell is then exercised by random rollouts.
- **T2d Area, chain and battle-wide damage; resistance and immunities.** Area shapes, Chain Lightning, Death Ripple, Destroy Undead, Armageddon; the resolution order (ADR-0014) with the Resistance skill and the creature abilities these spells meet (immunities by level, school or to all spells; resistance chances; damage reductions; S-18), as new `Ability` variants with BAT rules. Then experiment batch 2a compiled for the user (T10; the spec gains spells and mana), so its results are in before increment 2 builds on the damage formula.

**Checkpoint after T2d:** the user plays batch 2a and confirms the cut of increment 2 (proposed: T3 and T4).

## Design

[ADR-0014](../adr/0014-battle-magic.md) (accepted 2026-10-05, Q42) with its design note [`E04b-design.md`](E04b-design.md), pruned task by task (ADR index rule); ADR-0012 (modifiers and abilities) and ADR-0013 (the hero seam, points 4, 5, 9, 10) still apply.

## Exit criteria

1. ~~ADR-0014 accepted by the user.~~ (done 2026-10-05, Q42.)
2. Every battle spell in scope and every spell-like creature ability has catalog rules with labelled sources and tests; disputed ones have an observation fixture (SoD) or an OPEN-QUESTIONS entry with the experiment.
3. Confirmed quirks are in the ledger with flags; `faithful` and `strict-fixed` differ exactly by them; the data quirks Q-0006 and S-05 work through profile overrides; S-14 and S-20 are never reproduced.
4. Earlier invariants still property-tested, plus: legal casts and `apply` agree, mana never goes negative and a hero casts at most once per round, hidden objects and spell books never reach the other side's observation, stacks created in battle are accounted for in the outcome.
5. Casting is reachable from the sim and the C ABI through the factorized encoding; golden replays with casting; at least 100 k steps/s/core on the `full` scenario with casting heroes; `bench-check` re-baselined with the reason; state size and clone cost recorded.
6. At least one E04b experiment batch played by the user and recorded as fixtures.
7. `cargo xtask verify` green on Linux and Windows, including `--with-original-data`.

## What E04c needs from E04b

Battlefield-wide magic rules that a battlefield kind can set (spells at Expert by school, a spell-level limit); an effect kind for Earthquake; the hidden-obstacle mechanism for the Tower moat's land mines; spells against arrow towers and the S-06 interaction (Air Shield).

## Risks

- **Breadth:** about 60 hero spells, 11 ability spells, some 30 creature magic abilities, 43 artifacts. Mitigation: increments with user checkpoints, just-in-time surveys, effect kinds as data so most spells become data once their kind exists.
- **State size and clone cost** from per-stack effects and new stacks (`BattleState` 2192 bytes, clone ≈ 183 ns at E04a's end). Mitigation: ADR-0014 sizes the storage; measured at T2c, T3, T6 and T9.
- **The factorized action space is new** and must suit E05. Mitigation: ADR-0014 point 7 reviewed by the user; property test "every legal action applies, every applied action was legal".
- **Automatic behaviour** (Berserk, Blind, Hypnotize, casting creatures' target choice) is poorly documented. Mitigation: VCMI plus experiments; quirks S-11, S-12.
- **Hidden information** (Land Mine, Quicksand, spell books): hygiene property tests extended per task.
- **Rounding:** spell damage bonuses may use single-precision rates like Q-0005. Mitigation: batch 2a before T3 builds on the formula.
- **Sources:** the wiki is summarised and needs approval per session; only SoD can be observed; RoE differences come from other sources (Q15).
