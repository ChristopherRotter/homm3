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

## Amendment — E02 content pipeline (2026-09-30, user decisions Q8, Q13, Q14)

- **Format confirmed: TOML** (Q14). Packs stay strict (unknown fields are errors); comments next to values carry sources and rule ids.
- **Expansions are packs in a dependency graph** (Q8): `base` (RoE), `ab` (Armageddon's Blade, depends on `base`) and `sod` (Shadow of Death, depends on `ab`). A configuration selects which packs are enabled; enabling a pack whose dependencies are not enabled is a configuration error (no silent auto-enable), as are unknown packs and cycles. Load order is a topological order of the enabled packs with a deterministic tie-break (pack id).
- **Overrides:** a pack may change fields of entities defined by packs in its dependency closure (e.g. `sod` changes the Angel's cost); overrides are partial (only the listed fields change), applied in load order, and may not target a pack outside the closure or an entity that does not exist. **References** (a creature's faction, cost resources) are restricted to the closure the same way, so a pack never silently depends on a pack it does not declare.
- **Implemented in E02 T3:** resources and factions are content; creatures carry the non-battle fields (names, faction, level, cost, growth, horde growth, fight/AI value, shots, casts, adventure-map count) and flags from a closed set in code (`CreatureFlag`, unknown names are errors). Non-battle fields live in `CreatureInfo` beside the hot-path `CreatureType`. Ruleset hash encoding v2. Format reference: `packs/README.md`.
- **Configuration source:** the committed packs are generated once from the original data by the harvesting tool and then maintained as source (Q13, ADR-0009 amendment 2).

## Amendment — E03 battle core (2026-10-01, T2)

- **Abilities in packs** (Tier 3, ADR-0012): creatures carry `abilities`, a list of typed
  primitives with parameters from the closed set `h3_content::Ability` (format in
  `packs/README.md`). Creature references inside abilities resolve like other references
  (dependency closure) and may point forward within a pack. The compiler checks parameters,
  duplicates, exclusive kinds and the consistency with the executable's flags (`multi_headed`
  needs an attack pattern, `area_attack` an `area_shot`). The harvester emits them from sourced
  gap-fill entries.
- **Hot-path creature data:** `CreatureType` now also holds `shots` and the flags (moved from
  `CreatureInfo`), because battle rules read them; abilities stay a per-creature list in the
  ruleset, read by handle at the decision point.
- Ruleset hash encoding **v3**.

## Amendment — E08 ruleset parts (2026-10-10, ADR-0015 point 2)

- **A game's rules are the core ruleset plus one compiled part per context** (the adventure's
  first, in `h3-adventure`): each part is immutable, `Arc`-shared, with its own canonical hash
  and version, references core entities by handle and its source by key. `h3-game`'s
  `GameRuleset` bundles them; replay headers and the server's manifest record every part's hash.
- **One pack file per context:** `adventure.toml` beside the pack's existing file, hand-written
  source the harvester never touches (E08 decision 10), parsed by `h3-pack-adventure`. The split
  of `h3-content` into a core and a battle part, with `core.toml` and `battle.toml`, waits for
  E04b to close (ADR-0015 point 3).

## Revision history

- 2026-09-30: Accepted.
- 2026-09-30: Amended by E01 (pack format, compiler rules, quirk registry, stats copied into battle state).
- 2026-09-30: Amended by E02 T1 (TOML confirmed, expansion dependency graph, overrides, harvested packs).
- 2026-10-01: Amended by E03 T2 (abilities in packs, flags/shots on the hot-path creature type, hash v3).
- 2026-10-10: Amended by E08 (ruleset parts per context, one pack file per context; ADR-0015).
