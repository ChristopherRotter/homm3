# ADR-0012: Modifiers and abilities

- Status: Accepted (2026-10-01, by the user after reviewing it with the E03 report; proposed in E03 T1)
- Date: 2026-10-01
- Deciders: agent session (E03), within the scope the user set for E03; accepted by the user

## Context

Battles need stat changes that come and go (morale and luck auras, floors, terrain bonuses; from
E04 on heroes, artifacts and spells with durations) through one mechanism that is Tier-3 friendly
(ADR-0006), allocation-free and not evaluated on hot-path reads (ADR-0004 §8), and order-independent
(ADR-0003). Most creature abilities are not stat changes but conditional behaviour at one decision
point (jousting, hate, death blow). VCMI's general bonus tree (propagators, selectors, limiters,
updaters) is flexible but allocates and evaluates on reads; we keep only its observations that a
bonus has a source, a duration, a stacking group and a value type.

## Decision

### 1. Two concepts

- **Abilities** are static, typed behaviour primitives of a creature type: a closed enum in
  `h3-content` with parameters, validated by the compiler, read by rule code at the point where
  the rule applies. New behaviour is a new variant plus the rule that reads it.
- **Modifiers** are numeric changes `(bearer, stat, op, amount, source, expiry)`. The kernel owns
  the primitive; each context defines bearers, stats, sources and expiry events. Abilities that
  are plain stat changes (an aura, a floor) are turned into modifiers by the context that runs
  them; conditional behaviour is not.

### 2. Kernel primitive (`h3_kernel::modifier`, rule KER-MOD-001)

- Operations `Add`, `AddPercent`, `AtLeast`, `AtMost`; a fixed-capacity, inline pool.
- **Stacking:** a source contributes at most one modifier per bearer and stat; re-inserting
  replaces it. Different sources stack.
- **Evaluation** in fixed stages (sum, percentage, lower bound, upper bound wins, saturate) with
  only `+`, `max` and `min`, so the result is independent of order. The stages are specified in
  KER-MOD-001.
- **Expiry** is a context tag; the context removes modifiers when the matching event happens.

### 3. Use in a context

- The pool lives in the context state and is part of its canonical hash.
- **Effective stats are materialised:** recomputed when a modifier or something it depends on
  changes; the hot path reads plain fields.
- **Capacity is sized by construction:** each context documents and tests why its pool cannot
  overflow; a full pool is an invariant violation, not a command error.
- Context rules stay rules: the Defend bonus and rule clamps (morale −3…+3) are applied by the rule
  that reads the stat.

## Consequences

- Reads are free; writes cost one pass over the pool and the affected stacks.
- Conditional behaviour lives in rule code with a rule id, not in data-driven predicates.
- Enforced by kernel unit and property tests (order independence, stacking, staging), the state
  hash covering the pool and the coverage gate. The battle's pool (bearers, stats, sources,
  capacity) is documented in `h3_battle::morale`; heroes feed it per ADR-0013.

## Alternatives considered

- A VCMI-style bonus tree: allocation-heavy, evaluated on reads, hard to audit.
- Recompute on every read: turn order and damage would evaluate the pool on every step.
- Per-stack modifier arrays: side-wide effects duplicated into every stack.
- Abilities as modifiers with predicates: a limiter language, i.e. Tier 4 by the back door.

## Revision history

- 2026-10-01: Proposed (E03 T1); implementation notes added (E03 T6); accepted by the user.
- 2026-10-03: Cut to its decisions (ADR size rule, `docs/adr/README.md`). The evaluation stages
  are in KER-MOD-001, the battle pool's details in `h3_battle::morale`; the longer text is in git
  history (this file at commit `22b6bc0`).
