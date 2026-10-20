# ADR-0013: Heroes in battle — entities, the `BattleHero` seam and the hero's battle rules

- Status: Accepted (user, 2026-10-03; OPEN-QUESTIONS Q30)
- Date: 2026-10-03
- Deciders: user; agent session (E04a T1)
- Working detail (field lists, formulas, sizes): the code documentation (`h3_contracts` `hero.rs`, `h3_heroes`, `h3_battle`), the rule catalog and [`docs/epics/E04a-log.md`](../epics/E04a-log.md) ("The design note at closure"; the design note `E04a-design.md` was deleted when E04a closed, 2026-10-05)

## Context

E04a lets a hero lead a battle without magic; E04b adds magic, E04c sieges (Q27). Heroes in battle
and on the adventure map are distinct entities, with no "god" hero type (user, Q27); derived values
are computed before the seam (03-domain). The seam must change once for E04a and E04b.

## Decision

1. **Entities.** Hero type (content, `h3_content::HeroType`), hero details (`h3_heroes::HeroDetails`:
   level, experience, primary and secondary skills, artifacts, spell book, mana), hero on the map
   (E08, own type), battle hero (`h3_contracts::BattleHero`, all derived). No type is named `Hero`;
   contexts meet through ids and explicit pure translations. A cross-context `HeroId` names a
   record of the game's **store of heroes** (ADR-0017 point 1): the contexts read a hero's details
   there by reference and change them by returning **hero changes** that the game applies through
   `h3-heroes`. The army is not part of `HeroDetails`. The details hold the level-up's two
   counters (the levels at which Wisdom and a magic school were last offered; E08 increment 4);
   their level is the levels given, at most the experience's level, the difference the hero's
   pending level-ups. A hero's level seed is not part of them: it is the game's server-only state
   (ADR-0017 point 1).
2. **Crates.** `h3-heroes` is the **hero model**: a logic crate in the shared layer beneath the
   contexts (kernel, content, contracts only) with `HeroDetails`, its validation, the translations
   (`derive_battle_hero`, `war_machines`) and the typed hero changes (`HeroChange`: experience,
   spell points, a spell, an artifact; from E08 increment 4 a level-up: the level, its primary
   skill, the learned skill, the counters) with their checks; no commands, no map, game or battle state. The adventure, the game and later the
   towns depend on it; `h3-battle` never does and sees only `BattleHero` (points 4 and 5). **What
   a hero has from its details in every context is derived here once** (BAT-Q19): its available
   spells (learned, worn scrolls, the spell-giving artifacts) and its mastery of a spell
   (`h3_heroes::available_spells`, `spell_masteries`), from E08 increment 3 the adventure's Attack
   and Defense, Diplomacy's join bonus, the maximum spell points, Necromancy and Learning, from
   increment 4 progression (point 12), later movement and scouting (E08 decision 48); no context
   keeps its own copy, and no adventure profile
   seam copies them (E08 decision 5 withdrawn).
3. **Effects are data.** Artifacts, skill levels and specialties carry a closed `HeroEffect` enum in
   content (Tier 3, like `Ability`, ADR-0012); the translation resolves them into a closed
   `BattleEffect` enum in contracts. Percentages are **basis points** in content and seam.
   Progression is data too (Tier 2, ADR-0006): the hero parameters hold the experience of levels
   1 to 12, the highest level and the skills a level-up forces (Wisdom, the magic schools); a
   hero class holds its primary chances, skill weights and two forcing intervals.
4. **`BattleHero` is a hybrid:** typed fields for what the battle reads by name (primary skills,
   Tactics level, damage bonuses, war-machine control as derived numbers (the Catapult's: the
   Ballistics level, ADR-0023 point 6; the Ballista's also the Artillery level), surrender
   discount), a bounded list of resolved
   `BattleEffect`s for everything else, and E04b's magic fields reserved
   and validated from E04a on. The battle checks only what its rules need to run soundly
   (references, bounds); keeping a hero within the game's limits is the derivation's job (Q31).
5. **Seam changes.** `BattleSetup.heroes: [Option<BattleHero>; 2]` replaces `led_by_hero` (Q-0003
   reads hero presence); `BattleSetup.funds` for surrenders. `BattleResult` gains `Retreat(side)`
   and `Surrender(side)`. `BattleOutcome` reports facts — per-stack start and end counts, the
   surrender cost, per-hero results, rounds — and the orchestrator applies the consequences
   (casualties, captured artifacts, experience, Necromancy). Later epics add only enum variants;
   E04c adds the reserved optional fields `siege` on setup and outcome and a battlefield kind.
6. **Battle-long hero effects are applied at setup** to the stacks' base stats; morale and luck
   effects become side modifiers (ADR-0012 pool, capacity sized by construction).
7. **Phases and commands.** A tactics phase (round 0) reuses `Move` with cycling activations plus
   `EndTactics`; `Retreat`; `Surrender`, which ends the battle at once (no consent step); a
   hero-controlled Ballista uses `Shoot`, a hero-controlled First Aid Tent a new `Heal`.
8. **The battle prices a surrender**, because every input is in the battle context (living stacks,
   the ruleset's costs, the hero's discount); funds come in through the setup. Should a rule ever
   need information outside the battle, the price becomes an injected input, never an import.
9. **Visibility.** What the original shows in battle (hero identity, primary skills, mana, war
   machines and which of them the hero commands, tactics zone, retreat/surrender options) is
   public (but at a town the defender's option to retreat, which only the defender sees: ADR-0023
   point 2); skill levels, bonuses, effects, spell book, funds and the surrender price are private
   to the hero's side, which sees them in its own observation (`observe` reads the ruleset, Q37).
   Hidden battlefield objects (E04b) use the same owner-side projection (ADR-0004).
10. **Encodings.** E04a extends the dense action encoding (v8: heals, `EndTactics`, Retreat,
    Surrender) and the observation encoding (v4: a hero block per side, the opponent's without
    its private part); E04b designs a factorized one for casting together with E05.
11. **Rule ids.** Battle rules keep `BAT-` (new areas `BAT-TAC`, `BAT-FLE`, `BAT-HER`); derivation
    rules are `HER-` rules in `docs/rules/catalog/heroes.toml`, progression's `HER-EXP-` and
    `HER-LVL-` (E08 increment 4).
12. **Progression is `h3-heroes`' rules** (E08 increment 4; pure functions of the details, the
    ruleset and explicit inputs). No context writes a hero: an object or a building describes its
    effect, a context reads the details through `h3-heroes` and returns typed hero changes, and
    the game applies them and gives the levels that follow (point 1, ADR-0017 point 10); later
    increments' experience, skill and primary-skill gains, the levels at load of prisons and
    tavern heroes, and E10's computer players reuse the same functions.
    1. **Experience to levels:** levels 1 to 12 from the table, each further step the previous
       × 1.2 truncated (the original's); the experience is bounded at the highest level's range,
       so **level 74 is the highest** (A-7, always fixed, as HotA's fix).
    2. **The scenario's level limit** is an input: a gain caps the experience at the limit's
       threshold and lowers experience above it (D-19); a limit of 0 or above the highest level
       is none (A-8, always fixed).
    3. **A level-up** is drawn one level at a time through a **draw source its caller gives**:
       the primary skill from the class's chances (the second row from the level-up to level 10,
       D-15), +1 saturating at `primary_max` (A-9, always fixed); two offers by the original's
       procedure (an advance, then a new skill; Wisdom and a school forced after the class's
       intervals; the class's weights, a known school weighing 1 when one is due, a banned skill
       0, or 1 if known: D-17, D-18), the map's banned skills an input. The Elementalist's
       intervals are the original's 6 and 4, the magic classes' 3 and 3 by a data quirk off in
       `strict-fixed` (A-10, `ADV-QUIRK-10`).
    4. **The original's source** is provided here: its C runtime generator reseeded per level
       from the hero's seed and the new level. Which source a game uses is ADR-0017's (point 4).
    5. **The answer** is the level-up's hero change; who answers is the game's (ADR-0017 point 6).
    6. **The original's choice between two offers** (the computer's evaluation, 0x52BBD0 with
       0x524690 and 0x524DD0) is ported as a rule here, read first: it chooses at load (D-20) and
       for the system actor's default, and will for E10's computer players; what it needs beyond
       the details (the army's worth, the setup's defaults) is an argument.

## Consequences

- The battle stays ignorant of hero management; a skill's numbers are a data change, a new effect
  kind is one variant in content, one in contracts and one rule.
- Every caller of the seam changes in E04a (T4, T9, T11); state hash v9, ruleset hash v9.
- Enforced by `check-arch` (no `h3-battle` → `h3-heroes` edge: a forbidden edge in `arch.toml`,
  since the layers alone allow it from E08 T18a), property tests (hero isolation, observation
  hygiene, conservation of creatures) and `SetupError` tests for every validation.
- One store of heroes, as the original keeps one record per hero: heroes off the map (the tavern
  pool, prisons; E09) need no second home. Rejections stay atomic: a context checks a hero's
  details before it returns changes, and the game applies them infallibly (point 2).
- `h3-heroes` moves to the adventure lane once the battle lane is done with it (the user,
  ADV-Q13; a `shared/` change of `lanes.toml`); until then the adventure's additions to it are
  `cross-lane`.
- This revises the epic's non-goal "the orchestrator computes the surrender cost" (point 8).
- Progression (point 12): faithful level-ups are a function of the seed, the level and the hero's
  skills, as the original's; the data adds hero parameters, a class field and a class override
  kind (the pack format, the core ruleset hash); the details' counters move the game state hash;
  the evaluation's port needs a code reading first (E08 T25).

## Alternatives considered

- One hero type for all contexts, or `HeroDetails` across the seam: couples battle to hero
  management (rejected by the user, Q27).
- `h3-heroes` as a peer domain crate the adventure sees through a profile seam (E08 decision 5):
  every read a copied field, every write an outcome variant and code in `h3-game`; the details
  with the hero on the map in the adventure: heroes off the map need a second home; the hero rules
  in `h3-adventure`: the battle's pack tests and the towns would depend on the adventure (E08
  T17b, ADV-Q13).
- Typed fields only (seam grows per artifact kind) or an effects list only (list searches on the hot
  path, legality depends on list contents).
- Hero attack and defense as pool modifiers: recomputation for values fixed for the whole battle.
- Whole percents: cannot express 17.5 % or 0.25 % steps; floats are banned (ADR-0003).
- A `Place { stack, hex }` tactics command: about 3 500 extra actions for little gain.
- The orchestrator pricing surrenders: the action mask could not tell whether surrender is
  affordable.
- Progression (E08 ADV-Q16): the first offer at load (VCMI, the wiki; the code reading twice says
  otherwise, D-20); VCMI's table to level 197 (diverges from SoD's levels, A-7); the intervals by a
  might/magic affinity of the class (no other rule reads one); progression in `h3-game` (towns,
  objects and computer players would each need the game).

## Revision history

- 2026-10-03: Proposed (E04a T1); revised after an internal consistency review.
- 2026-10-03: Accepted by the user. Cut to its decisions (ADR size rule, `docs/adr/README.md`); the
  working detail moved to `docs/epics/E04a-design.md`. Point 8 records the user's condition.
- 2026-10-04: point 7 — the optional consent step for a surrender is dropped (user, OPEN-QUESTIONS
  Q32): in SoD neither a human nor the computer opponent is asked (experiment batch 1b R4), and in
  HotA the enemy cannot refuse either (wiki, Combat). Point 9 no longer lists offers.
- 2026-10-04 (E04a T12a): what E04a's tasks decided, folded in — point 4: the battle checks only
  soundness (Q31); point 9: which war machines a hero commands is public, the own side's private
  part is in its observation (T11a, Q37); point 10: the encodings E04a ended with (T11b). Detail
  that stays out of the ADR: `FactionGroup` (Q36) and `StatOp` in the crate docs.
- 2026-10-05 (E04a closure): no decision changed; the header points to the code documentation and
  the epic's log, because the design note was deleted.
- 2026-10-10 (E04b T8a; the user, BAT-Q19): point 2 — hero-level magic (available spells, the
  mastery of a spell) is derived once in `h3-heroes` for every context, as the original keeps it
  in the hero record (`installation.md` §7.22); the battle's own best-school code went, and the
  seam carries a mastery per spell (ADR-0014 point 2).
- 2026-10-13 (E04c T5a; the user, BAT-Q40): point 5's reserved `siege` fields are decided by
  ADR-0023: the setup's is `BattleSetup.siege`, the outcome's is dropped (the original writes
  nothing about the walls back to the town). The battlefield kind is ADR-0022's.
- 2026-10-14 (E08 T18a; the user, ADV-Q13): points 1 and 2 amended: `h3-heroes` is the hero
  model in the shared layer beneath the contexts, read by them by reference and changed through
  typed hero changes; the store of heroes is the game's (ADR-0017 point 1); E08 decision 5's
  adventure profile withdrawn (drafted in E08 T17b, #181).
- 2026-10-14 (E04c T6; the user, BAT-Q43): point 4 — the Catapult's control carries the hero's
  Ballistics level, not derived numbers (ADR-0023 point 6): the battle's own Ballistics table also
  serves the uncontrolled Catapult and the Cyclops.
- 2026-10-15 (E04c T7e, BAT-STUB-3): point 4 — the Ballista's control carries the hero's Artillery
  level beside its derived shots and chance: the computer's worth of a Ballista grows with it
  (`installation.md` §7.33), and the turrets and the automatic Ballista read that worth. Private,
  as the rest of the control (point 9).
- 2026-10-15 (E04c T10b; the user, BAT-Q50): point 9 — at a town the defender's option to retreat
  (the town's Escape Tunnel) is private to the defender: the original enables the retreat control
  for the acting side only (0x478573), and a town's buildings are no public fact (ADR-0023 point 2).
- 2026-10-20 (E08 T26a; the user at ADV-Q15 and ADV-Q16): points 1, 2, 3 and 11 amended and point
  12 added for E08 increment 4, hero progression: the details' counters and the level's meaning,
  the level-up's hero change, progression's data, `HER-EXP-` and `HER-LVL-`, and progression as
  `h3-heroes`' rules (experience to levels bounded at 74, the level limit as an input, a level-up
  through a draw source its caller gives, the original's offers, the original's choice ported);
  no context writes a hero (the user's note at ADV-Q16). Drafted in E08 T24b1 (#318; survey
  `E08-progression.md`).
