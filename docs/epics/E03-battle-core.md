# E03 — Battle core (RoE rules)

Status: todo · Refined: 2026-09-30 (end of E01) · Updated: 2026-10-01 (E02 data available)

## Goal

Replace the walking skeleton's simplified battle with faithful RoE battle rules on real data (from E02), keeping the E01 architecture: validated commands, events with visibility, observations, legal-command consistency, determinism, ≥ 100 k steps/s/core.

## Starting point (from E01)

`h3-battle` implements BAT-GEO-001, BAT-SET-001 (invented formation data), BAT-TRN-001..004, BAT-MOV-001, BAT-ATK-001, BAT-DMG-001..003, BAT-RET-001, BAT-END-001, each with "Skeleton scope" notes in `docs/rules/catalog/battle.toml` listing what E03 must add. Source conflicts Q11 (attack cap: +300 %) and Q12 (large-stack damage roll: wiki) were decided by the user at E01 closure.

## Inputs from E02 (2026-10-01)

- **Real data:** committed packs `base` (RoE, latest patch), `ab`, `sod` (OPEN-QUESTIONS Q8/Q13/Q15;
  format `packs/README.md`). E03 targets `base`; AB/SoD-specific abilities stay E11, but the
  ability primitives must not assume RoE only (Conflux, Dragon Fly weakness from RoE 1.3, …).
- **Flags already in data** (`h3_content::CreatureFlag`, from the executable's table): `two_hex`,
  `flying`, `shooter`, `breath_attack`, `alive`, `catapult`, `war_machine`, `king_1..3`,
  `mind_immune`, `no_melee_penalty`, `fire_immune`, `strikes_twice`, `no_enemy_retaliation`,
  `no_morale`, `undead`, `multi_headed` (pattern per creature: hydras all around, Cerberi three
  hexes), `area_attack`, `dragon`. Today they live in `CreatureInfo` (not on the hot path); T2 decides
  which become battle-stack data. Shots are in `CreatureInfo.shots`.
- **Not in data yet — abilities from the harvest report** (hard-coded per creature in the
  original; need primitives + data): jousting (Cavalier/Champion), morale ± (Angels, Bone/Ghost
  Dragons), no obstacle/wall penalty (Arch Mage; also exe bit `0x800` on Arch Mage, Beholder, Evil Eye,
  unidentified), unlimited/extra retaliations (Griffins), hate, death stare, devil teleport (Devils
  walk in the exe table; they do not fly), Dragon Fly weakness and other spell-like attacks.
  Collect the full RoE list with sources at T2 start.
- **War machines are not harvested** (speed 0, no damage for most): T7 must extend the harvester
  and the pack format (or model them separately) — see BACKLOG.
- **Formations:** `base` carries the original loose formation (VCMI layout table, sourced in the
  gap-fill file); tight formation and tactics are T8.

## Scope

1. **Bonus/modifier system** in the kernel (design ADR first: Tier-3 friendly, no per-step allocation, deterministic ordering). Everything below builds on it.
2. **Creature abilities** as a closed set of typed primitives referenced from data (flying, shooting, two-hex, no retaliation, unlimited/extra retaliation, double attack, …) — the RoE set, AB/SoD later (E11).
3. **Battlefield:** obstacles (data), two-hex creatures, flyers, original formations (loose/tight) from imported data, tactics phase if in scope of RoE rules.
4. **Ranged combat:** shots, range and obstacle/wall penalties, melee penalty for shooters, shooter retaliation rules.
5. **Morale and luck:** RNG draws on the BATTLE stream, morale phase in the wait ordering, luck in the damage formula (I4).
6. **Damage formula completeness:** all I/R factors that do not need heroes/spells; decide float vs. integer semantics per factor with observation fixtures (Armorer-style quirks → ledger).
7. **War machines** (ballista, first aid tent, ammo cart; catapult with sieges later) including their turn-order phases.
8. **End of battle** details: summoned/war-machine stacks not counting as living, simultaneous wipe-out (draw).
9. **Observation fixtures** from the original game for disputed, high-impact rules (e.g. damage rounding when several float factors combine), recorded per `docs/04-rules-and-sources.md` §2.
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
- T2 Ability primitives v1 + data wiring (E02 pack format v2 is ready; extend the harvester to emit ability references).
- T3 Geometry: obstacles, two-hex units, flyers (behind the existing `Field` API).
- T4 Ranged combat.
- T5 Morale/luck + turn-order phases.
- T6 Damage formula completion + observation fixtures.
- T7 War machines.
- T8 End-of-battle details, formations/tactics, closure.

## Risks

- The modifier system is the load-bearing design of E03–E04; do it first, keep it small, benchmark it.
- State size and clone cost will grow (effects per stack): keep them measured (`h3 bench`, `bench-check`).
