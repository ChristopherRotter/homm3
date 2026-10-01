# ADR-0012: Modifiers and abilities

- Status: Accepted (2026-10-01, by the user after reviewing it with the E03 report; proposed in E03 T1)
- Date: 2026-10-01
- Deciders: agent session (E03), within the scope the user set for E03; accepted by the user

## Context

E03 brings the first stat changes that come and go during a battle: morale and luck auras of
creatures (Angels, Bone Dragons, Devils), floors on morale and luck (Minotaurs, Halflings), army
composition morale, native-terrain bonuses. E04 adds many more: hero primary skills, secondary
skills, artifacts, specialties and spells with durations. They all need one mechanism
(03-domain, "Open domain design questions"), and it must be:

- **Tier-3 friendly** (ADR-0006): behaviour comes from a closed set of typed primitives that
  content refers to, not from scripts;
- **fast**: no allocation per step (ADR-0004 §8), no per-read evaluation on the hot path (turn
  order reads every living stack's speed before every activation; damage reads attack and
  defense);
- **deterministic** (ADR-0003): integer arithmetic, no order dependence.

VCMI solves this with a general bonus system: bonuses attached to a tree of bearers, propagated
by propagators, filtered by selectors and limiters, changed by updaters, cached by string keys.
It is very flexible, allocates, and evaluates selectors on reads. We do not copy it (03-domain:
"do not copy another project's design uncritically"), but we keep its useful observations: a
bonus has a *source*, a *duration*, a *stacking* group, and a value type
(additive, percent, minimum/maximum).

Most creature abilities of the original are not stat changes at all. Jousting, hate, death blow,
double attack, no retaliation, breath attacks and ranged penalties are *conditional behaviour at
one decision point* ("when this stack strikes that stack after walking n hexes"). Expressing them
as generic modifiers with predicate limiters is where bonus systems become slow and opaque.

## Decision

### 1. Two concepts

- **Abilities** are static, typed behaviour primitives of a creature type: a closed Rust enum in
  `h3-content` (variants with parameters, e.g. `Hate { targets }`, `Jousting { percent_per_hex }`,
  `ExtraRetaliations { count }`). Packs refer to them by name with parameters; the compiler
  validates them; the ruleset stores them per creature type. Rule code reads them at the point
  where the rule applies (the damage computation asks whether the attacker hates the target).
  New abilities are new variants plus the rule code that reads them (E03 T2 and later tasks).
- **Modifiers** are numeric changes to a *stat* of a *bearer*: `(bearer, stat, op, amount,
  source, expiry)`. The kernel owns the primitive and its evaluation; each context defines its
  bearers, stats, sources and expiry events. Abilities whose effect is a plain stat change (a
  morale aura, a luck floor) are turned into modifiers by the context that runs them; abilities
  with conditional behaviour are not.

### 2. Kernel primitive (`h3_kernel::modifier`, rule KER-MOD-001)

- `Modifier<B, S, E>` with a context-defined bearer `B`, stat `S` and expiry tag `E`, an
  operation `ModifierOp ∈ {Add, AddPercent, AtLeast, AtMost}`, an `i32` amount and a
  `SourceKey` (`u32`).
- `ModifierPool<B, S, E, N>`: fixed capacity `N`, inline storage (`FixedVec`), no allocation.
- **Stacking:** a source contributes at most one modifier per bearer and stat. Inserting a
  modifier whose `(bearer, stat, source)` already exists *replaces* it (a re-cast spell refreshes,
  two stacks with the same aura give it once). Different sources stack. Content that must not
  stack across creature types (Angels and Archangels) uses the same source key.
- **Evaluation** of a stat for a bearer, in fixed stages, with `i64` intermediates:
  1. `flat = base + Σ Add`
  2. `value = flat + floor(flat · Σ AddPercent / 100)`
  3. `value = max(value, max AtLeast)` if any `AtLeast` applies
  4. `value = min(value, min AtMost)` if any `AtMost` applies (an upper bound wins over a lower
     bound)
  5. saturate to `i32`.
  Every stage aggregates with `+`, `max` or `min`, so the result does not depend on the order of
  the modifiers. Which modifiers apply to a bearer (its own, its side's, …) is a predicate the
  context passes in.
- **Expiry** is a context-defined tag; the context removes modifiers when the matching event
  happens (`ModifierPool::remove_if`, order-preserving). The kernel never interprets tags.

### 3. Use in a context (the battle, from E03 T6 on)

- The pool lives in the context state (`BattleState`) and is part of its canonical hash.
- **Effective stats are materialised.** A stack keeps its base stats (copied from the creature
  at setup, ADR-0006 E01 amendment) and its effective stats. Whenever a modifier is inserted or
  removed, or something a modifier depends on changes (a source stack dies), the affected stacks'
  effective stats are recomputed. The hot path reads plain fields; evaluation runs only on
  changes.
- **Capacity is sized by construction:** each context documents why its pool cannot overflow
  (bounded sources per bearer) and tests the bound; a full pool is an invariant violation, not a
  reachable command error. E04 revisits the capacity when spells arrive.
- Context-specific rules stay rules: the Defend bonus (BAT-TRN-004) is computed from the
  materialised defense when the stack is hit, as in E01; rule clamps (morale range −3…+3) are
  applied by the rule that reads the stat.

### 4. What this does not decide

Stat sets beyond the battle, how heroes/artifacts/spells feed modifiers across the seam (E04:
derived values are computed before the `BattleSetup` seam, 03-domain), and the `Trace` sink that
will list modifier contributions (backlog).

## Consequences

- Reads are free; writes cost one pass over the pool and the affected stacks. State grows by
  `N` small entries (measured in `bench-check` when the battle starts using the pool).
- Conditional behaviour lives in rule code next to the rule it changes, with a rule id, instead
  of in data-driven predicates; adding such behaviour means adding an ability variant and code
  (ADR-0006 Tier 3: "the set grows by adding variants in code").
- Order independence is structural (sums, maxima, minima), so no sort or tie-breaker is needed
  for evaluation (ADR-0003 §9).
- Enforced by: kernel unit and property tests (order independence, stacking, staging), the state
  hash covering the pool, the coverage gate.

## Alternatives considered

- **A VCMI-style bonus tree** (propagators, selectors, limiters, updaters): general, but
  allocation-heavy, evaluated on reads and hard to audit; rejected.
- **Recompute on every read** (no materialised stats): simpler state, but turn order and damage
  would evaluate the pool on every step; rejected for the hot path.
- **Per-stack fixed arrays of modifiers**: no bearer field, but side-wide effects would be
  duplicated into every stack and the state would grow by stacks × slots; rejected.
- **Abilities as modifiers with predicates**: one mechanism, but conditional behaviour (hate,
  jousting) would need a limiter language; rejected (Tier 4 by the back door).

## Implementation notes — E03 T6 (2026-10-01)

- The battle's pool (`h3_battle::Modifiers`) has bearers `Bearer::Side(side)` only, stats
  `Morale` and `Luck`, and one expiry, `EndOfBattle`. Sources: army alignment and undead
  (fixed keys) and auras (key = `Ability::aura_key`, equal for equal aura values, so equal
  auras replace each other — Angels and Archangels give +1 once).
- Capacity by construction: per side 2 army modifiers plus one per distinct aura value; the
  compiler caps distinct aura values at `MAX_AURA_VALUES` = 8, so `MODIFIER_CAPACITY` = 20.
- Per-creature floors (`minimum_morale`, `minimum_luck`) are turned into `AtLeast` modifiers
  when the stats are materialised, read from the creature's abilities rather than stored in the
  pool (they never change during a battle; stack bearers arrive with E04 spells).
- Materialisation happens once at setup; in E03 nothing changes the pool during a battle
  (auras persist after their bearers die). Rule clamps (morale −3…+3, luck 0…3 from the roll
  tables) are applied by the morale/luck rules, as §3 says.

## Revision history

- 2026-10-01: Proposed (E03 T1).
- 2026-10-01: Implementation notes added (E03 T6, battle pool).
- 2026-10-01: Accepted by the user at the E03 closure.
