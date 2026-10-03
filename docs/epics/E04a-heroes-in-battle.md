# E04a — Heroes in battle (without magic)

Status: **in progress** (refined 2026-10-01; T1 done and ADR-0013 accepted 2026-10-03; T2: batch 1a evaluated 2026-10-03, batch 1b next; T3 done 2026-10-03 (T3a, T3b); T4 next) · Follows E03 (closed 2026-10-01) · The former E04 was split by the user into three epics (OPEN-QUESTIONS Q27): **E04a** (this one), **E04b** battle magic, **E04c** sieges and special battlefields.

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

## Starting point (E01–E03)

- **Seam today** (`h3-contracts`): `BattleSetup { attacker, defender, obstacles, led_by_hero: [bool; 2], terrain, war_machines: [Vec<CreatureTypeId>; 2], formations }`, `BattleOutcome { result: BattleResult, survivors, war_machines }`, `BattleResult { Victory(side), Draw }`. `led_by_hero` and the war-machine fields carry comments "the hero itself arrives in E04".
- **Modifiers** (ADR-0012, accepted): pool with bearer `Side` only, stats morale and luck, one expiry, capacity 20. E04a adds side-wide stat sources (hero attack and defense, Leadership, Luck, artifacts); E04b adds stack bearers and timed expiry. ADR-0013 decides the shape and revisits the capacity.
- **Content**: the committed packs hold creatures, factions, resources, terrains, obstacles and battle parameters. No heroes, classes, skills, artifacts or spells yet.
- **War machines** act automatically without a hero's skill (E03 T8); with the skill the hero chooses the target (E04a T7).
- **Versions now:** action encoding v4, observation encoding v3, C ABI v2, replay format v2, state hash v8, ruleset hash v8 (v9 since T3a, v10 since T3b).
- **Open experiments:** Q21 (death stare), Q22 (jousting distance), Q23 (exact vs float damage rounding), Q26 (First Aid Tent heal). Q23 matters much more in E04a: Archery, Offense, Armorer and artifact bonuses stack on the damage formula. *(2026-10-03: Q21 and Q23 answered by batch 1a, Q26 by code reading; Q22 remains.)*
- **Quirk flags:** `quirk.battle.no_luck_without_hero` (Q-0003) reads `led_by_hero` today and must read the real hero afterwards.

## What the installation holds (checked 2026-10-01 on the copies in `original-data/`)

| Table | Contents | AB copy vs SoD copy |
|---|---|---|
| `SPTRAITS.TXT` | 81 spell rows (10 adventure, 60 combat, 11 creature abilities): level, school flags, mana cost per mastery, power factor, effect value per mastery, chance per town, AI value, target attributes, texts | differs |
| `ARTRAITS.TXT` | 144 artifacts: cost, slot flags (including war machine slots), class letter, text. No effects | differs |
| `HCTRAITS.TXT` | 18 hero classes: initial primary skills, primary-skill gain probabilities, secondary-skill probabilities, selection probability per town | identical |
| `HOTRAITS.TXT` | 156 heroes: starting army only | identical |
| `HeroSpec.txt` | 156 specialty rows: names and texts; parameters partly in the texts | identical |
| `SSTRAITS.TXT` | 28 secondary skills: texts only; the numbers are inside the descriptions | identical |
| `ballist.txt`, `PriSkill.txt`, `SkillLev.txt`, `ArtSlots.txt` | small tables (Ballistics effects, names, slots) | |

Consequences: effect semantics and most skill and specialty numbers need sourced gap-fill (wiki/VCMI), as with the E03 abilities. The RoE-era versions of all of these (RoE had fewer heroes and artifacts and older values; exact membership is to be sourced in T1) are not in the installation and need membership and difference data from other sources (Q15). Whether `Heroes3.exe` holds static tables for heroes, artifacts or spells (as it does for creatures) is an open question for T1.

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

- [x] **T1 Survey and ADR-0013 — stop for the user.** *Done 2026-10-03; ADR-0013 accepted by the user and cut to its decisions (detail in `E04a-design.md`); quirks: confirm before flagging (Q30); code reading allowed as a last resort (Q29).* Results: tables and exe tables in `docs/formats/installation.md` §6–§7 (hero table with RoE/expansion flags, specialty table, per-level skill values, artifact primary skills, spell table; one code reading pending Q29); surveys in `docs/rules/surveys/` (skills, artifacts, heroes and specialties, battle mechanics, 22 quirks to classify, the batch-1 dispute list); ADR-0013 Proposed; glossary in 03-domain. Findings that change later tasks: percentages need basis points (specialty terms such as 17.5 %); the creature upgrade relation is missing from content (T3); Ice, Magma and Energy Elementals have bugged levels in the exe table that our packs inherited (quirk S-05); the battle should price surrenders itself (ADR-0013 §7, revises the non-goal below); the Ballista may have unlimited shots in SoD (S-01, against E03's BAT-WAR-002).
  - Catalogue the hero, skill, artifact and spell tables (`docs/formats/installation.md`) and look for static tables in `Heroes3.exe`.
  - Source survey (VCMI 1.7.5 and the wiki, labelled) for E04a: secondary skill effects per level, artifact effect kinds, specialty kinds, tactics, retreat and surrender, hero-run war machines, experience inputs; in outline for E04b/E04c so the seam covers them (spell effect kinds, siege inputs). Check that wiki fetches are still approved.
  - **Quirk survey** for the whole E04 scope with sources (`docs/rules/`), for the user to classify in one sitting; then ledger entries and flags.
  - List the disputed rules that need an observation (input to T2).
  - **ADR-0013 (Proposed):** the entities table, the crate and type names, `BattleHero`, the effect vocabulary, modifiers on sides and stacks and the pool capacity, `BattleSetup`/`BattleOutcome`/`BattleResult` changes, visibility, the tactics model, the encoding policy. See "Open design points".
  - **Stop:** the user accepts ADR-0013 and classifies the quirks. Nothing later builds on a Proposed seam.
- [ ] **T2 Experiment pipeline v0** (can overlap T3; the user's runs are asynchronous). *2026-10-03: smoke map works in the user's game. Spec format, compiler (`crates/h3-experiment`: spec → `.h3m` + results template; names checked against the installation's tables) and **batch 1a** (`experiments/specs/E04a-batch-1a.toml`, results `experiments/results/E04a-batch-1a.md`) delivered, played by the user and **evaluated** (2026-10-03): Q21 → wiki, Q23 → single-precision rates (quirk Q-0005), quirks Q-0004 (unlimited Ballista shots) and Q-0006 (Roland), S-04 not reproduced; consequences per task in `E04a-design.md`, "Batch 1a results". Next: design batch 1b (T8–T10 disputes, plus the First Aid Tent with the skill). Q22 is not staged (see `docs/experiments/README.md`). Running specs in our engine waits for T11.*
  - Spec format: a battle setup (armies, heroes with skills, artifacts and spells, terrain) plus a *procedure* (what to click, in what order) and a *record* section (what to write down). Running it in our engine, so that prediction and observation can be compared automatically, waits for heroes in scenarios (T11); until then predictions are written into the spec.
  - Writer: a generator for the SoD `.h3m` format that stages N encounters on one small map (your hero next to each opposing hero or neutral stack, custom armies, skills, artifacts). References: VCMI's map reader (GPL, may be ported with attribution) and the community format notes; libraries found so far only read maps. The object templates come from the installation's tables (`original-data/`).
  - **Smoke map first:** one encounter, loaded by the user to prove the game accepts our file, before any batch is built.
  - Results: lean templates generated from the spec, filled in by the user (user's choice, Q30). Results become `observation` sources, version "SoD 4.0 (Complete)".
  - Batch 1, split (Q30): **1a** for T5–T7 (Q23, Q21, war machines, quirks S-01–S-04), **1b** for T8–T10. Q26 and First Aid are settled by code reading; Q22 is not staged.
  - Placement (T2 decides): an adapter. The h3m module probably belongs in `h3-formats` (E07 adds the reader there); no logic crate. The map goes into a gitignored output folder and the user copies it into the game's Maps folder; **agents never write into the installation**.
  - Limits to design around: damage rolls, morale and luck are random and obstacles are generated by the game, so prefer experiments with deterministic outcomes (creatures with a single-value damage range, no morale or luck sources, as sketched in Q23) or enough repetitions; many sequences need manual steps, hence the procedure.
- [x] **T3 Content** — pack format v3: types for hero classes, heroes, secondary skills, artifacts, specialties and spells (the spell *data rows*: level, schools, mana per mastery, power factor, effect values; effects are E04b, but heroes and artifacts reference spells, and the harvester reads the tables once). Harvester reads the tables and the exe tables T1 finds; gap-fill file for effect parameters with sources on every entry; RoE/AB/SoD membership and overrides; compiler checks; ruleset hash v9; synthetic packs get an invented hero, class, skill and artifact set (test-data policy). No flavor text is committed. **Split with the user (2026-10-03) at the artifacts:**
  - [x] **T3a** *(done 2026-10-03)*: spell schools, spells, secondary skills with per-level effects, hero classes, hero types with specialties, the creature upgrade relation; `h3_content::HeroEffect` with 13 kinds (`packs/README.md`, "Hero effects"); `h3_formats::exe::hero_data`; harvester module `heroes` (+ gap-fill `[heroes]`); synthetic hero set using every effect kind; ruleset hash v9; golden replays re-recorded (hash only). Refinements (agent, change freely): per-town columns (spell chances, tavern weights), class aggression, spell AI values and the spell flags/targets are **not** harvested (no consumer before E04b/E09; RoE has no Conflux column, so they would need override kinds); hero availability (campaign heroes, Lord Haart the Knight) waits for E08–E10. Batch 1a confirmed two T3a values: one Ballista shot at Basic Artillery (D1) and the creature specialties as the executable's table has them (H-1: Catherine Swordsmen and Crusaders, Roland Crusaders only); the gap-fill facts taken from the surveys were re-checked against the wiki's raw text (Hero specialty, Artillery).
  - [x] **T3b** *(done 2026-10-03; scope chosen by the user: T3b alone)*: artifact slots as kinds of positions with a count (`[[artifact_slot]]`, `[[artifact_slot_override]]`; 14 kinds, misc 4 before SoD and 5 with it), `[[artifact]]` with class, cost, slot (none: backpack only), war machine, `needs_skill`, effects and combination parts (locks derived by the compiler), nine new `HeroEffect` kinds (`primary_skills`, `morale_at_most`, `luck_at_most`, `alignment_group`, `creature_stat_percent`, `regeneration`, `no_range_penalty`, `shoot_when_adjacent`, `no_retreat`; `HeroEffect` gained a faction parameter); `h3_formats::exe` reads the artifact primary-skill table (§7.3); harvester module `artifacts` with the gap-fill `[artifacts]` section (membership, war machines, the 12 combinations, effects, pending lists checked for coverage); packs regenerated (base 127, ab 2, sod 12 artifacts; the three unused SoD rows are skipped); synthetic artifacts use every kind; ruleset hash v10; golden replays re-recorded (hash only). Agent decisions (change freely): `primary_skills` is an effect kind that counts per copy (the original sums its table), every other kind once per artifact type; `needs_skill` is a property of the artifact, not of single effects (true for every original artifact); Necromancy artifacts stay pending with the Necromancy skill (E08–E10); combination costs are left out (blank in the original). Pending lists and what is left out: `packs/README.md`.
- [ ] **T4 `h3-heroes` and the seam** — crate (domain, logic, `arch.toml` entry, rule catalog `heroes.toml` with prefix `HER`), `HeroDetails` and its validation (slots, class and skill limits), `derive_battle_hero`; `BattleHero` in contracts; `BattleSetup.heroes` replaces `led_by_hero` and Q-0003 reads the real hero; battle validates the setup; compiler check for overlapping war-machine positions (E03 review); state hash v9; observation features for heroes.
- [ ] **T5 Primary skills and hero morale/luck** — the hero's attack and defense add to its creatures, Leadership and Luck and artifacts feed morale and luck (side modifiers, ADR-0012 pool capacity revisited), rules with catalog ids. Also the death-stare change from batch 1a (Q21: the new top creature has full health, BAT-DMG-009).
- [ ] **T6 Skills that change damage** — Archery, Offense, Armorer as damage factors. Q23 is answered (batch 1a): implement quirk Q-0005 (single-precision rates, exactly in integers) with R1–R3 as fixtures.
- [ ] **T7 Hero-run war machines** — with Artillery or First Aid the hero chooses the target (new commands or activation), without the skill the automatic behavior of E03 stays; war machines come from the hero's artifacts; checked arithmetic for the Ballista ranking (E03 review). Batch 1a: the hero's attack counts in the Ballista's damage range and its attack advantage; Basic Artillery one shot, Expert two; a commanded Ballista may Wait and Defend; quirk Q-0004 (unlimited Ballista shots).
- [ ] **T8 Tactics** — the placement phase before round 1: who gets it, how far stacks may move, commands and events, interplay with the formations (BAT-SET-001).
- [ ] **T9 Retreat, surrender and the richer outcome** — new `BattleResult` variants, casualties per side and creature type, hero fate, the inputs for experience, Necromancy and artifact capture; no application of them.
- [ ] **T10 Artifacts and specialties without spells** — the effect primitives from the survey, stat and morale/luck artifacts, creature-stat and speed effects, war-machine artifacts, combination artifacts (assembly rules belong to the hero details), specialties of creature, skill and stat kinds. Spell-related ones are E04b.
- [ ] **T11 Sim, FFI, CLI, scenarios** — heroes in the scenario and replay formats, action encoding v5 (retreat, surrender, tactics, hero-run war machines), observation encoding v4, C ABI v3, golden replays re-recorded with notes, benchmark baseline re-measured with the reason, hero-bearing scenarios in the `full` workload.
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
