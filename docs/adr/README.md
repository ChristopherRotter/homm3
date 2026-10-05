# Architecture Decision Records

Binding decisions. One decision per file, numbered, never silently edited once Accepted (supersede instead). Template: `0000-template.md`.

**Keep ADRs short (user directive, 2026-10-03).** Agents read the ADRs that touch their task before working, so every line costs context in every later session.

- An ADR states *decisions*: short, checkable statements, with context, consequences and rejected alternatives in a few lines each. Target: about 80 lines; a longer one needs a reason.
- Working detail (field lists, formulas, sizes, derivations, step-by-step designs) goes into a **design note** next to the epic (`docs/epics/<epic>-design.md`). It is a handover, not binding: the task that implements a section deletes it, because the code documentation, the rule catalog and the pack docs then hold that detail. The file is deleted when the epic closes.
- Presentation-only edits of an accepted ADR (cutting detail that is now documented in code or other docs) are allowed and noted in its revision history; changing a decision still needs a new ADR or an amendment the user accepts.
- Read the "Read when" column and open only the ADRs that match your task.

| ADR | Title | Status | Read when |
|---|---|---|---|
| [0001](0001-record-architecture-decisions.md) | Record architecture decisions | Accepted | writing or changing an ADR |
| [0002](0002-rust-and-workspace.md) | Rust, Cargo workspace, unsafe policy | Accepted | adding a crate, a dependency or `unsafe`; toolchain bumps |
| [0003](0003-determinism-and-numeric-model.md) | Determinism and numeric model | Accepted | any logic code (numbers, randomness, ordering, floats) |
| [0004](0004-state-commands-events.md) | Server-authoritative state, commands and events | Accepted | adding commands, events, state or observation features |
| [0005](0005-bounded-contexts-and-boundaries.md) | Bounded contexts and enforced boundaries | Accepted | adding a crate or a dependency between contexts |
| [0006](0006-configuration-tiers-and-ruleset.md) | Configuration tiers and compiled ruleset | Accepted | adding content types, pack fields or battle parameters |
| [0007](0007-provenance-quirks-profiles.md) | Rule provenance, quirks ledger, profiles | Accepted | writing rules, quirk flags or profiles |
| [0008](0008-security-model.md) | Security model | Accepted | command validation, actor identity, hidden information |
| [0009](0009-license-and-reference-material.md) | License and reference-material policy (GPL-2.0-or-later) | Accepted | using VCMI code, wiki text or original game data |
| [0010](0010-testing-gates-and-traceability.md) | Testing gates and rule traceability | Accepted | adding gates, coverage exclusions, rule traceability |
| [0011](0011-portability-and-c-abi.md) | Portability and C ABI | Accepted | FFI, encodings, platform targets |
| [0012](0012-modifiers-and-abilities.md) | Modifiers and abilities | Accepted (E03) | modifiers (morale, luck, stats) and creature abilities |
| [0013](0013-heroes-in-battle.md) | Heroes in battle: entities, the `BattleHero` seam, the hero's battle rules | Accepted | hero content, `h3-heroes`, the `BattleHero` seam (E04a–E04c) |
| [0014](0014-battle-magic.md) | Battle magic: spells as data, casting, timed effects, created stacks, hidden objects | Proposed (E04b T1) | spells, casting, spell effects on stacks, stacks created in battle, hidden obstacles, the casting encoding (E04b, E04c) |
