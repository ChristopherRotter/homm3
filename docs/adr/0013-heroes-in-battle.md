# ADR-0013: Heroes in battle — entities, the `BattleHero` seam and the hero's battle rules

- Status: Accepted (user, 2026-10-03; OPEN-QUESTIONS Q30)
- Date: 2026-10-03
- Deciders: user; agent session (E04a T1)
- Working detail (field lists, formulas, sizes): [`docs/epics/E04a-design.md`](../epics/E04a-design.md), pruned as E04a's tasks implement it

## Context

E04a lets a hero lead a battle without magic; E04b adds magic, E04c sieges (Q27). Heroes in battle
and on the adventure map are distinct entities, with no "god" hero type (user, Q27); derived values
are computed before the seam (03-domain). The seam must change once for E04a and E04b.

## Decision

1. **Entities.** Hero type (content, `h3_content::HeroType`), hero details (`h3_heroes::HeroDetails`:
   level, experience, primary and secondary skills, artifacts, spell book, mana), hero on the map
   (E08, own type), battle hero (`h3_contracts::BattleHero`, all derived). No type is named `Hero`;
   contexts meet through ids and explicit pure translations. A cross-context `HeroId` arrives with a
   store of heroes (E08/E10). The army is not part of `HeroDetails`.
2. **Crates.** `h3-heroes` is a domain logic crate (kernel, content, contracts only) with
   `HeroDetails`, its validation, `derive_battle_hero` and `war_machines`; no commands, no map or
   battle state. `h3-battle` never depends on it.
3. **Effects are data.** Artifacts, skill levels and specialties carry a closed `HeroEffect` enum in
   content (Tier 3, like `Ability`, ADR-0012); the translation resolves them into a closed
   `BattleEffect` enum in contracts. Percentages are **basis points** in content and seam.
4. **`BattleHero` is a hybrid:** typed fields for what the battle reads by name (primary skills,
   Tactics level, damage bonuses, war-machine control as derived numbers, surrender discount), a
   bounded list of resolved `BattleEffect`s for everything else, and E04b's magic fields reserved
   and validated from E04a on. The battle validates it like the rest of the setup.
5. **Seam changes.** `BattleSetup.heroes: [Option<BattleHero>; 2]` replaces `led_by_hero` (Q-0003
   reads hero presence); `BattleSetup.funds` for surrenders. `BattleResult` gains `Retreat(side)`
   and `Surrender(side)`. `BattleOutcome` reports facts — per-stack start and end counts, the
   surrender cost, per-hero results, rounds — and the orchestrator applies the consequences
   (casualties, captured artifacts, experience, Necromancy). Later epics add only enum variants;
   E04c adds the reserved optional fields `siege` on setup and outcome and a battlefield kind.
6. **Battle-long hero effects are applied at setup** to the stacks' base stats; morale and luck
   effects become side modifiers (ADR-0012 pool, capacity sized by construction).
7. **Phases and commands.** A tactics phase (round 0) reuses `Move` with cycling activations plus
   `EndTactics`; `Retreat`; `Surrender` with an optional consent step (ruleset parameter); a
   hero-controlled Ballista uses `Shoot`, a hero-controlled First Aid Tent a new `Heal`.
8. **The battle prices a surrender**, because every input is in the battle context (living stacks,
   the ruleset's costs, the hero's discount); funds come in through the setup. Should a rule ever
   need information outside the battle, the price becomes an injected input, never an import.
9. **Visibility.** What the original shows in battle (hero identity, primary skills, mana, war
   machines, tactics zone, retreat/surrender options and offers) is public; skill levels, bonuses,
   effects, spell book and funds are private to the hero's side. Hidden battlefield objects (E04b)
   use the same owner-side projection (ADR-0004).
10. **Encodings.** E04a extends the dense action encoding (v5); E04b designs a factorized one for
    casting together with E05.
11. **Rule ids.** Battle rules keep `BAT-` (new areas `BAT-TAC`, `BAT-FLE`, `BAT-HER`); derivation
    rules are `HER-` rules in `docs/rules/catalog/heroes.toml`.

## Consequences

- The battle stays ignorant of hero management; a skill's numbers are a data change, a new effect
  kind is one variant in content, one in contracts and one rule.
- Every caller of the seam changes in E04a (T4, T9, T11); state hash v9, ruleset hash v9.
- Enforced by `check-arch` (no `h3-battle` → `h3-heroes` edge), property tests (hero isolation,
  observation hygiene, conservation of creatures) and `SetupError` tests for every validation.
- This revises the epic's non-goal "the orchestrator computes the surrender cost" (point 8).

## Alternatives considered

- One hero type for all contexts, or `HeroDetails` across the seam: couples battle to hero
  management (rejected by the user, Q27).
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
