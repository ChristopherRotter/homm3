# E03 — Battle core (RoE rules)

Status: todo · Refined: 2026-09-30 (end of E01)

## Goal

Replace the walking skeleton's simplified battle with faithful RoE battle rules on real data (from E02), keeping the E01 architecture: validated commands, events with visibility, observations, legal-command consistency, determinism, ≥ 100 k steps/s/core.

## Starting point (from E01)

`h3-battle` implements BAT-GEO-001, BAT-SET-001 (invented formation data), BAT-TRN-001..004, BAT-MOV-001, BAT-ATK-001, BAT-DMG-001..003, BAT-RET-001, BAT-END-001, each with "Skeleton scope" notes in `docs/rules/catalog/battle.toml` listing what E03 must add. Open source conflicts: Q11 (attack cap), Q12 (large-stack damage roll).

## Scope

1. **Bonus/modifier system** in the kernel (design ADR first: Tier-3 friendly, no per-step allocation, deterministic ordering). Everything below builds on it.
2. **Creature abilities** as a closed set of typed primitives referenced from data (flying, shooting, two-hex, no retaliation, unlimited/extra retaliation, double attack, …) — the RoE set, AB/SoD later (E11).
3. **Battlefield:** obstacles (data), two-hex creatures, flyers, original formations (loose/tight) from imported data, tactics phase if in scope of RoE rules.
4. **Ranged combat:** shots, range and obstacle/wall penalties, melee penalty for shooters, shooter retaliation rules.
5. **Morale and luck:** RNG draws on the BATTLE stream, morale phase in the wait ordering, luck in the damage formula (I4).
6. **Damage formula completeness:** all I/R factors that do not need heroes/spells; decide float vs. integer semantics per factor with observation fixtures (Armorer-style quirks → ledger).
7. **War machines** (ballista, first aid tent, ammo cart; catapult with sieges later) including their turn-order phases.
8. **End of battle** details: summoned/war-machine stacks not counting as living, simultaneous wipe-out (draw).
9. **Observation fixtures** from the original game for disputed, high-impact rules (Q11, Q12, damage rounding), recorded per `docs/04-rules-and-sources.md` §2.
10. **Oracle harness (optional):** run VCMI as a separate process on generated scenarios and compare outcome distributions (ADR-0009 allows it; code may be ported with attribution).

## Non-goals

Heroes, spells and artifacts (E04), sieges beyond what war machines need, AB/SoD specifics (E11), AI (E05/E06).

## Exit criteria

1. Every battle rule in scope has a catalog entry with labelled sources and tests; disputed ones have observation fixtures or an OPEN-QUESTIONS entry.
2. Quirks found are in the ledger with flags where applicable; `faithful` and `strict-fixed` profiles differ exactly by them.
3. All E01 invariants still property-tested; golden replays re-recorded with a note; performance ≥ 100 k steps/s/core on the full scenario (re-baseline bench-check with the reason).
4. `cargo xtask verify` green (Linux + Windows).

## Tasks (sketch — split further when the epic starts)

- T1 Modifier-system ADR + kernel implementation.
- T2 Ability primitives v1 + data wiring (needs E02 pack format v2).
- T3 Geometry: obstacles, two-hex units, flyers (behind the existing `Field` API).
- T4 Ranged combat.
- T5 Morale/luck + turn-order phases.
- T6 Damage formula completion + observation fixtures.
- T7 War machines.
- T8 End-of-battle details, formations/tactics, closure.

## Risks

- The modifier system is the load-bearing design of E03–E04; do it first, keep it small, benchmark it.
- State size and clone cost will grow (effects per stack): keep them measured (`h3 bench`, `bench-check`).
