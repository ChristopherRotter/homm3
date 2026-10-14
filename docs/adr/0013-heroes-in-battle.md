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
   `h3-heroes`. The army is not part of `HeroDetails`.
2. **Crates.** `h3-heroes` is the **hero model**: a logic crate in the shared layer beneath the
   contexts (kernel, content, contracts only) with `HeroDetails`, its validation, the translations
   (`derive_battle_hero`, `war_machines`) and the typed hero changes (`HeroChange`: experience,
   spell points, a spell, an artifact; increment 4 adds skills, primary skills and levels) with
   their checks; no commands, no map, game or battle state. The adventure, the game and later the
   towns depend on it; `h3-battle` never does and sees only `BattleHero` (points 4 and 5). **What
   a hero has from its details in every context is derived here once** (BAT-Q19): its available
   spells (learned, worn scrolls, the spell-giving artifacts) and its mastery of a spell
   (`h3_heroes::available_spells`, `spell_masteries`), from E08 increment 3 the adventure's Attack
   and Defense, Diplomacy's join bonus, the maximum spell points, Necromancy and Learning, from
   increment 4 movement and scouting; no context keeps its own copy, and no adventure profile
   seam copies them (E08 decision 5 withdrawn).
3. **Effects are data.** Artifacts, skill levels and specialties carry a closed `HeroEffect` enum in
   content (Tier 3, like `Ability`, ADR-0012); the translation resolves them into a closed
   `BattleEffect` enum in contracts. Percentages are **basis points** in content and seam.
4. **`BattleHero` is a hybrid:** typed fields for what the battle reads by name (primary skills,
   Tactics level, damage bonuses, war-machine control as derived numbers (the Catapult's: the
   Ballistics level, ADR-0023 point 6), surrender discount), a bounded list of resolved
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
   public; skill levels, bonuses, effects, spell book, funds and the surrender price are private
   to the hero's side, which sees them in its own observation (`observe` reads the ruleset, Q37).
   Hidden battlefield objects (E04b) use the same owner-side projection (ADR-0004).
10. **Encodings.** E04a extends the dense action encoding (v8: heals, `EndTactics`, Retreat,
    Surrender) and the observation encoding (v4: a hero block per side, the opponent's without
    its private part); E04b designs a factorized one for casting together with E05.
11. **Rule ids.** Battle rules keep `BAT-` (new areas `BAT-TAC`, `BAT-FLE`, `BAT-HER`); derivation
    rules are `HER-` rules in `docs/rules/catalog/heroes.toml`.

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
