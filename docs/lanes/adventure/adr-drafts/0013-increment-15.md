# Draft: ADR-0013 amended for E08 increment 15 (a spell's mastery on a ground, the cost on the map)

Status: **Proposed** (E08 T137b, 2026-11-21; decision 405, ADV-Q63). Applied by the `shared/` task
(T140) with batch E15's outcome. Design note: `docs/epics/E08-design.md` §5, §6.

## Point 2 (Crates), amended

After "its available spells (learned, worn scrolls, the spell-giving artifacts) and its mastery of
a spell (`h3_heroes::available_spells`, `spell_masteries`)" insert:

> **on a ground that raises it** (from E08 increment 15, point 17: `spell_masteries_on` with the
> ground's raise, which the caller derives from the battlefield; `spell_masteries` is the
> ordinary ground's),

## Point 17 (new): a spell on the map is `h3-heroes`' derivation

17. **A spell's mastery on a ground and its cost on the map are `h3-heroes`' derivations** (E08
    increment 15; pure functions of the details, the core ruleset and an explicit raise):
    1. **The raise** (`SchoolRaise`: none, every school, or a set of schools) is an input. Each
       caller derives it from the ground's battlefield of the battle part
       (`Battlefield::expert_schools`, BAT-TER-005's data): the battle from its battlefield, the
       game from the special ground of the hero's tile (`AdventureState::special_ground`). A raised
       school gives Expert to every spell among its schools; the rest is HER-SPL-003's (the best
       school mastery, the `spell_mastery` effects, at most Expert). The original's
       `hero_school_mastery` 0x4E5100 with the tile's field (survey `E08-spells.md` §4.2).
       `h3-heroes` reads no battle content; the battle may call it for its heroes' casts (its
       choice of when, `cross-lane`), so one rule serves both contexts.
    2. **The cost on the map** (`map_spell_cost`, HER-SPL-008): the core's cost at that mastery,
       at least 1, without the armies' ±2 (battle-only: `hero_spell_cost` 0x4E5240 adds them only
       with an enemy army). Spent by the existing spell points change.
    3. **Who may cast on the map** (`can_cast_on_map`): the spell among the available spells and a
       spell book worn (HER-BAT-011's test), until batch E15's row E15-15 reads the original's
       bookless scroll (E08 decision 404; ADV-STUB-21).

## Consequences (added)

- The adventure's casts read their mastery from the same derivation as the battle's, with the
  ground as an input: no copy of the raise in the adventure part.
- `h3-game` derives the raise and the casting limit from the battle part it already bundles
  (ADR-0015 point 2), so no crate gains a dependency.

## Alternatives considered (added)

- An adventure-only helper (`h3-heroes` gains a map function, the battle's raise stays in
  `h3-battle`): two homes for one rule; not chosen (decision 405).
- The raise as core data (the special grounds' schools in `h3-content`): moves battle content to
  the core for one reader; not chosen.

## Revision history (line to add)

- 2026-11-?? (E08 T140 `shared/`; E08 decision 405, ADV-Q63): point 2 amended and point 17 added
  for E08 increment 15, the adventure spells: the mastery on a ground by an explicit raise, the
  cost on the map, the cast's book test. Drafted in E08 T137b (design note `E08-design.md` §5,
  §6); batch E15's outcome (T139) folded in.
