# ADR-0006: Configuration tiers and compiled ruleset

- Status: Accepted
- Date: 2026-09-30
- Deciders: user + planning session

## Context

Everything gameplay-relevant should be configurable/moddable — but the user explicitly ranks architecture and performance above configurability when they conflict. Extensions (AB, SoD; later WoG, HotA) must fit without a rewrite.

## Decision

**Tiers**

- **Tier 1 — data:** definitions of creatures, heroes/classes, spells, artifacts, skills, towns/buildings, terrain, object types, costs, which content packs are active.
- **Tier 2 — parameters:** formula constants, limits (army slots, hero level cap, map size limits, …), quirk flags, profile selection.
- **Tier 3 — composable behavior:** abilities/effects built from a **closed set of typed primitives** (the set grows by adding variants in code), referenced from data.
- Tier 4 (arbitrary code/scripting) is out of scope for the base game and official extensions. Rule modules for future extensions are compiled in (Cargo features/registration), not dynamically loaded, until a later ADR says otherwise.

**Compiled ruleset.** Human-editable text (TOML by default; format confirmed in E02) → validated by adapters → compiled once into an immutable, `Arc`-shared `Ruleset` with dense integer handles. IDs in data are namespaced strings (`pack:name`). The hot path never parses, hashes strings, or does generic lookups.

**Architecture wins.** If a configurability feature makes the architecture needlessly complex or measurably slows the hot path without immediate gain, it is dropped or deferred; note it in the relevant ADR/HANDOFF.

**Guardrails so later extensions and fan features don't force a rewrite** (applied from E01):
- No magic numbers in logic for: map dimensions, army slot count, hero level cap, number of players/teams, number of town types, resource kinds, skill counts, spell levels. They are ruleset parameters or data.
- All entities are referenced by handle/ID from content; no `match` over "the nine towns".
- Content packs declare dependencies, load order and override rules; AB, SoD are packs like any other.
- Queries a UI would need (path preview, damage preview, legal moves) are first-class core functions, not client-side reimplementations.
- Profiles/quirk flags are part of the ruleset and replay header.

## Consequences

Moddability and extension room without runtime interpretation cost; a clear place (the closed primitive set) where "new ability" requirements are debated. Some extension needs may require a new primitive or rule-module hook (expected and acceptable).

## Alternatives considered

Runtime scripting for all rules (flexible, slow, hard to test/secure); hard-coded rules with constants (fast, unmoddable, painful for extensions); generic data-interpreting engine everywhere (over-general and slow).

## Amendment — E01 walking skeleton (2026-09-30)

- **Format:** TOML packs parsed by the `h3-pack` adapter (strict: unknown fields and out-of-range numbers are errors); the pure compiler in `h3-content` validates everything again. Keys are `pack:name`; a pack may only define keys in its own namespace; packs are compiled in load order and must list dependencies loaded before them. Duplicate keys are errors (override rules are deferred, backlog).
- **Tier 2 in the skeleton:** battle parameters (field size, side columns, army slots, damage roll samples, attack/defense rates and caps, Defend bonus, formation rows) come from exactly one pack.
- **Profiles and quirks:** a profile selects quirk flags from a closed registry in code (`h3_content::REGISTRY`, empty in E01 because no quirk-affected rule exists yet); unknown flags are compile errors. Profile-level Tier-2 overrides are deferred (backlog).
- **Hot path:** creature stats are copied into each battle stack at setup, so the battle never looks anything up in the ruleset during play (and later per-stack effects have a place to live). The compiled ruleset has a canonical content hash recorded in replays.
- **Guardrails applied:** army slots and battlefield dimensions are parameters (engine capacity limits: 16 slots per side, 255 hexes), no content is matched by name in code.

## Revision history

- 2026-09-30: Accepted.
- 2026-09-30: Amended by E01 (pack format, compiler rules, quirk registry, stats copied into battle state).
