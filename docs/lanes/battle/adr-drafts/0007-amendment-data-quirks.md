# Draft: ADR-0007 amendment — data quirks (E04b T8e)

- Status: Proposed (battle lane, 2026-10-10, BAT-Q25)
- Amends: ADR-0007 (profiles select quirk flags); lifts ADR-0006's deferral of profile-level
  overrides for data quirks only
- On acceptance: a `shared/` pull request appends the section below to ADR-0007, adds one line
  to ADR-0006 and deletes this draft

## Context

Two confirmed quirks live in the content data, not in behaviour: the executable names the
Crusader for Roland's specialty, so it raises only Crusaders (Q-0006, fix-optional), and its
creature table gives the Ice, Magma and Energy Elementals the wrong level (S-05, fix-optional,
BAT-Q25). The packs hold the executable's values, so `strict-fixed` cannot fix them yet: today
the two profiles differ only by the flags that rule code reads (E04b exit criterion 3).
Levels are read by spell specialties and Clone in battle, and later by the adventure (growth,
random monsters), so a fix belongs in the data every context reads, not in each reader.

## Amendment — E04b T8e: data quirks (proposed)

1. **Data quirks get flags.** A quirk whose wrong value is content data has a flag in the same
   closed registry, named `quirk.content.<name>`, with a ledger entry and defaults like any
   other. Profiles stay a selection of flags; `faithful` and `strict-fixed` differ exactly by
   their flags. Rule code never reads a `content` flag; only the compiler does.
2. **The pack keeps the original's value; the fix is a gated override.** The entity keeps the
   harvested (faithful) value. An override in a pack whose dependency closure holds the entity
   carries the fixed values and names its flag (`fixes = "quirk.content.<name>"`). The compiler
   applies it in load order, as any override, only when the active profile turns that flag off.
   It must name a registered `content` flag. Ungated overrides are unchanged.
3. **In the core compile.** Gated overrides are applied while `h3-content` compiles the core
   ruleset, before any ruleset part reads the entity. Every context sees one profile's values.
   The ruleset hash covers the resulting data (and the profile, as before), so a replay's
   header still names its rules exactly.
4. **Scope now:** gating for `[[creature_override]]`, and a new `[[hero_override]]` whose only
   field for now is `specialty` (it replaces the whole list). The harvester writes both
   overrides from its gap-fill file. Per-profile Tier-2 *parameters* stay deferred (ADR-0006);
   none is needed yet.

## Consequences

The ledger, the profiles and the replay header need no new concept, and a fixed data quirk
costs one flag and one override in the pack that defines the entity. A ruleset is compiled per
profile, as today. Cost: two flags of the 64 the registry allows, and the compiler resolves
the profile before it applies overrides.

## Alternatives considered

- **Overrides inside the profile** (`[[profile.creature_override]]`): `base` defines the
  profiles but cannot reach `ab`'s elementals or Roland (closure rule). Each pack would need its
  own profiles, and a profile would become a data patch rather than a selection of flags.
- **A second "level for calculations" field** (survey S-05): fixes only levels, and every
  reader in every context must pick the right field. Q-0006 would still need something else.
- **Rule code reading the flag** (as behaviour quirks do): spreads a data fact over every rule
  that reads a level or a specialty, where a missed reader goes unnoticed.
