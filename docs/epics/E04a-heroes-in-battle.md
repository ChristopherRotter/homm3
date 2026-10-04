# E04a — Heroes in battle (without magic)

Status: **in progress** (refined 2026-10-01; T1–T9 done by 2026-10-04; T10a and T10b done, T10c next) · Follows E03 (closed 2026-10-01) · The former E04 was split by the user into three epics (OPEN-QUESTIONS Q27): **E04a** (this one), **E04b** battle magic, **E04c** sieges and special battlefields.

## Goal

Let a battle be led by a hero. A hero adds its primary skills, secondary skills, artifacts and specialty to its army, runs the war machines it brings, may use tactics before the first round, and may retreat or surrender; the outcome tells the orchestrator what happened. No spells (E04b). The architecture stays that of E01–E03: pure logic, validated commands, events with visibility, observations, determinism, at least 100 k steps/s/core.

The epic also fixes, once, the boundaries between the different things called "hero" (below), so that E04b, E04c and the adventure map (E08) build on them instead of growing one "god" hero type.

## The former E04 and where its parts went (Q27)

| Epic | Contents |
|---|---|
| **E04a** (this epic) | hero content (classes, heroes, secondary skills, artifacts, specialties; spell *data rows*, see T3), the `h3-heroes` crate and the `BattleHero` seam (ADR-0013), primary skills, secondary skills that need no spells, artifacts and specialties without spells, hero-run Ballista and First Aid Tent, tactics, retreat and surrender, a richer outcome, the experiment pipeline |
| **E04b** battle magic | spell effects and their rules, spell book and mana in battle, casting, timed effects on stacks, summons and clones, obstacle spells, resistance, spell-like creature abilities, spell immunities, spellcasting creatures, spell-related skills, artifacts and specialties, hidden-information rules, casting in the sim encodings |
| **E04c** sieges and special battlefields | siege setup derived from the town, walls, gate, moat, towers, Catapult, Ballistics, Earthquake, shooting at and through walls, magic terrains, coast and ship battles |

Carried over from E03 (HANDOFF, "Moved to E04"): special battlefields → E04c; spell-like abilities and spell immunities → E04b; tactics → E04a; sieges, Catapult and towers → E04c; hero control of war machines → E04a (Ballista and First Aid Tent; the Catapult is E04c). E03 review leftovers: checked arithmetic for the Ballista ranking → E04a T7; the war-machine overlap check → E04a T4; the others stay in the BACKLOG inbox.

## Decisions at the start of the refinement (2026-10-01, user)

Recorded in OPEN-QUESTIONS Q27 (scope), Q28 (experiments) and Q6 (quirk policy).

1. **Three epics**, allocated as in the table above.
2. **Heroes are battle-oriented in this milestone:** a hero value type, loadout validation and the pure derivation of what a battle needs. No equip, learn or level-up commands, no experience or level-up algorithm; those wait for the adventure and game epics (E08–E10), where they have callers.
3. **Heroes in battle and on the adventure map are distinct entities; no "god" hero type.** A domain crate `h3-heroes` holds only the hero's lasting details and the pure translation to the battle's own type. See the next section.
4. **All battle spells** are in scope for E04b (Earthquake goes with the sieges, E04c).
5. **The seam is decided by ADR-0013 in T1, and work stops there until the user has accepted it.** `BattleSetup`, `BattleOutcome` and `BattleResult` change only through that review.
6. **Experiments in the original game are batched and automated** (Q28). The user can currently verify only SoD behavior, and staging more than a handful of battles by hand in the map editor is too tedious. The engine therefore compiles a spec of staged battles into an `.h3m` map that the user loads and plays (T2).
7. **Quirks are surveyed up front** (T1): the user classifies the documented bugs of the whole E04 scope in one sitting. Q6 is confirmed: `faithful` is the default profile, `exploit-fix` is always fixed and never toggleable.

## The things called "hero"

Decided in [ADR-0013](../adr/0013-heroes-in-battle.md) (accepted 2026-10-03), points 1–2: hero type (content), `HeroDetails` (`h3-heroes`), the hero on the map (E08), `BattleHero` (contracts); no type named `Hero`; `h3-battle` never imports `h3-heroes`. Working detail for the tasks: [`E04a-design.md`](E04a-design.md), pruned as tasks implement it.

## Scope

1. **Survey and ADR-0013** (T1).
2. **Experiment pipeline** (T2): spec → `.h3m` for the user's SoD installation, results template, first batch.
3. **Content** (T3): hero classes, heroes, secondary skills, artifacts, specialties and the spell data rows, harvested and gap-filled, in the packs `base`/`ab`/`sod`; pack format v3.
4. **`h3-heroes` and the seam** (T4): `HeroDetails`, loadout validation, `BattleHero`, `BattleSetup.heroes` replacing `led_by_hero`.
5. **Primary skills and hero morale/luck** (T5).
6. **Skills that change damage** (T6): Archery, Offense, Armorer.
7. **Hero-run war machines** (T7): Artillery (Ballista), First Aid (Tent), war machines from artifacts.
8. **Tactics** (T8).
9. **Retreat, surrender and the richer outcome** (T9).
10. **Artifacts and specialties without spells** (T10).
11. **Sim, FFI, CLI, scenarios** (T11): heroes in scenarios and replays, new encodings, ABI v3, benchmarks.
12. **Closure** (T12): fold the experiment results in, review, quirk ledger, docs.

## Non-goals

Spells, mana in play, spell-like abilities, immunities, spellcasting creatures (E04b). Sieges, Catapult, Ballistics' effect, towers, Earthquake, special battlefields (E04c). Experience, level-ups, skill offers, equip/unequip/learn/trade commands, army management, the hero on the map (E08–E10). Applying outcomes to heroes (experience, Necromancy raising, artifact capture): the outcome carries the inputs, applying them waits for the epic that owns hero progression. AI (E05/E06). The VCMI oracle harness stays deferred (E03 scope item 10 was never done; revisit with the evaluation needs of E05/E06).

## Tasks (proposed; refine at the start of each; keep status current)

Starting point, installation tables and the full record of done tasks (refinements, agent decisions, measurements): [`E04a-log.md`](E04a-log.md).

- [x] **T1 Survey and ADR-0013** — done 2026-10-03: surveys in `docs/rules/surveys/`, ADR-0013 accepted (Q30), the executable's tables in `docs/formats/installation.md` §6–§7.
- [x] **T2 Experiment pipeline** — done 2026-10-04: `crates/h3-experiment`; batches 1a and 1b played by the user and evaluated (`E04a-design.md`, "Batch 1a results" and "Batch 1b results"; `docs/experiments/README.md`). Running specs in our engine: T11.
- [x] **T3 Content** — done 2026-10-03 (T3a heroes, skills, spell rows; T3b artifacts): pack format v3 (`packs/README.md`), ruleset hash v10.
- [x] **T4 `h3-heroes` and the seam** — done 2026-10-04 (T4a, T4b1, T4b2): `HeroDetails`, `BattleHero`, `derive_battle_hero`, `BattleSetup.heroes` (Q31); HER-DET/SEC/ART/SPL/WAR, HER-BAT-001/002, BAT-HER-001; state hash v9, ruleset hash v11.
- [x] **T5 Primary skills, hero morale and luck** — done 2026-10-04 (T5a, T5b): BAT-HER-002/003, HER-BAT-003, BAT-DMG-009 (Q21); replay format v4.
- [x] **T6 Skills that change damage** — done 2026-10-04 (T6a, T6b): quirk Q-0005, HER-BAT-004, BAT-HER-004; state hash v10, replay format v5.
- [x] **T7 Hero-run war machines** — done 2026-10-04 (T7a, T7b): BAT-WAR-005/006/007, HER-BAT-005, quirk Q-0004, `Command::Heal`; state hash v12, replay format v7, action encoding v5, C ABI v3.
- [x] **T8 Tactics** — done 2026-10-04 (one checkpoint, Q33): BAT-TAC-001/002, `tactics_zone_columns` (ruleset hash v12), `Command::EndTactics`, `Observation.tactics`; state hash v13, replay format v8, action encoding v6, C ABI v4.
- [x] **T9 Retreat, surrender and the richer outcome** — done 2026-10-04 in three checkpoints (T9a Retreat, T9b Surrender, T9c the outcome; split with the user, Q34): BAT-FLE-001/002, BAT-END-003, HER-BAT-006/007; `BattleResult::Retreat`/`Surrender`, `BattleSetup.funds`, the outcome's stack records; ruleset hash v13, state hash v16, replay format v10, action encoding v8, C ABI v6.
- [ ] **T10 Artifacts and specialties without spells** — the effect primitives from the survey, stat and morale/luck artifacts, creature-stat and speed effects, war-machine artifacts, combination artifacts (assembly rules belong to the hero details), specialties of creature, skill and stat kinds. Spell-related ones are E04b. Split with the user (Q35): **T10a** stat effects on stacks (`creature_stat`, `creature_stat_percent`, `creature_specialty`), **T10b** stack traits (regeneration, no range penalty, shooting while adjacent), **T10c** Angelic Alliance's alignment group and the T10 closure (measurements, design note). Already done earlier: war-machine artifacts (T7), combination parts (T4b1), the skill specialties without spells (T6b, T7b); Q-0006 stays data until E04b. **T10a done** 2026-10-04: HER-BAT-008, BAT-HER-005, `StatOp::PercentBpDown`, effect kind `stack_stat` in scenarios and replays (replay format v11). **T10b done** 2026-10-04: HER-BAT-009, BAT-HER-006, trait `SHOOTS_WHEN_ADJACENT`, `Stack.regeneration` (state hash v17), every `BattleEffect` in scenarios and replays (replay format v12).
- [ ] **T11 Sim, FFI, CLI, scenarios** — heroes in the scenario and replay formats, the action encoding complete for retreat and surrender (T7b added heals as v5 with C ABI v3, T8 `EndTactics` as v6 with C ABI v4; T9 appends its commands), observation encoding v4 and its ABI bump, golden replays re-recorded with notes, benchmark baseline re-measured with the reason, hero-bearing scenarios in the `full` workload.
- [ ] **T12 Closure** — fold the experiment results into the catalog (rules `verified` where observed), quirk ledger and profiles, independent review pass (rules against sources, robustness and security), HANDOFF, backlog, ADR-0013 revisions; the E03 hardening items that fit (work bound for death stare, random obstacles in the sim and C ABI).

Order: T1 first and alone; T2 and T3 independent; T4 after T1 and T3; T5–T10 after T4 (T6 wants the batch-1 result, T10 after T5); T11 and T12 last.

## Design

Settled by ADR-0013 (accepted by the user 2026-10-03, Q30). The working detail per task (field lists, formulas, sizes) is in [`E04a-design.md`](E04a-design.md); each task deletes the section it implements.

## Exit criteria

1. ~~ADR-0013 accepted by the user; quirk survey classified~~ (done 2026-10-03, Q30: quirks are confirmed before they are flagged).
2. Every rule in scope has a catalog entry with labelled sources and tests; disputed ones have an observation fixture (SoD) or an OPEN-QUESTIONS entry with the experiment.
3. Quirks are in the ledger with flags; `faithful` and `strict-fixed` differ exactly by them.
4. All earlier invariants still property-tested, plus: hero isolation (a side's hero never changes the other side's stacks except through modelled events), observation hygiene for hero data, conservation of creatures with casualties accounted.
5. Golden replays re-recorded with notes; at least 100 k steps/s/core on the `full` scenario with heroes (E03 closed at about 197 k); `bench-check` re-baselined with the reason; state size and clone cost measured and recorded.
6. The first experiment batch has been run by the user and its results are recorded as fixtures.
7. `cargo xtask verify` green on Linux and Windows, including `--with-original-data`.

## What E04b and E04c need from E04a

- **E04b:** `BattleHero` with spell fields reserved and validated; a stat/effect mechanism that can take stack bearers and expiries; hero data already in the observation; the quirk list for spells already classified; the experiment pipeline able to stage heroes with spell books; spell data rows already in the packs.
- **E04c:** the setup seam can take a siege description without another `BattleSetup` redesign (ADR-0013 reserves the extension point); `Ballistics` level present in `BattleHero`; the Catapult's place in the war-machine model decided.

## Risks

- **Seam churn.** Mitigation: one ADR, reserved fields, the stop after T1.
- **Data volume and gap-fill:** 144 artifacts, 156 specialties and 28 skills whose numbers sit in description texts; RoE-era values are invisible in the installation. Mitigation: gap-fill with a source on every entry, the same discipline as E03's abilities.
- **The `.h3m` writer is new and checkable only by the user.** Mitigation: smoke map first; small scope; the writer stays an adapter.
- **Experiment results hold for SoD only.** Rules that differ in RoE or AB cannot be verified; the catalog says which rules were observed in which version.
- **State and observation growth** (heroes, effects): measure state size, clone cost and steps/s at T4, T10 and T11.
- **Vocabulary drift:** "hero" used loosely again. Mitigation: the entities table, no plain `Hero` type, glossary entries.
