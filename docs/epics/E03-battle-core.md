# E03 — Battle core (RoE rules)

Status: in progress (branch `e03-battle-core`) · Refined: 2026-09-30 (end of E01) · Updated: 2026-10-01 (E02 data available; scope decisions Q16 at E03 start)

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

## Scope decisions at E03 start (2026-10-01, user, OPEN-QUESTIONS Q16)

- **In E03:** war machines without heroes (Ballista, First Aid Tent, Ammo Cart with their own
  stats, turn phases and automatic control; hero skills plug in with E04); **native-terrain
  bonuses and random obstacle generation** per battlefield type (terrain content and obstacle
  catalogues).
- **Deferred to E04:** spell-like creature abilities (blind, petrify, bind, curse, dispel,
  weakness, aging, poison, …), spell immunities and resistances, spellcasting creatures, and the
  tactics phase (it needs a hero's Tactics skill).
- **Sources:** VCMI 1.7.5 (cloned in the sandbox) and the Heroes 3 wiki (fetches approved by the
  user). **No observation experiments** in the original game for now: uncertain rules get an
  OPEN-QUESTIONS entry with a ready-made experiment.
- **ADR-0012** (modifiers and abilities) is written as Proposed; the user reviews it with the E03
  report.

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

Heroes, spells and artifacts (E04), spell-like creature abilities and the tactics phase (E04, Q16), sieges beyond what war machines need, AB/SoD specifics (E11), AI (E05/E06).

## Exit criteria

1. Every battle rule in scope has a catalog entry with labelled sources and tests; disputed ones have observation fixtures or an OPEN-QUESTIONS entry.
2. Quirks found are in the ledger with flags where applicable; `faithful` and `strict-fixed` profiles differ exactly by them.
3. All E01 invariants still property-tested; golden replays re-recorded with a note; performance ≥ 100 k steps/s/core on the full scenario (re-baseline bench-check with the reason).
4. `cargo xtask verify` green (Linux + Windows).

## Tasks (refined at E03 start; status kept current)

- [x] T1 Modifier-system ADR (ADR-0012) + kernel implementation (`h3_kernel::modifier`, KER-MOD-001).
- [x] T2 Ability primitives v1 + data wiring: closed `Ability` set in `h3-content` (20 kinds),
  pack format, compiler checks, ruleset hash v3, harvester gap-fill with sourced abilities for
  `base`/`ab` (attribute-column cross-check), flags and shots on `CreatureType`. Rule code reads
  them in T3–T8 (each task lists the kinds it implements).
- [x] T3 Geometry: footprints of two-hex stacks (BAT-GEO-002), obstacles in the setup
  (BAT-GEO-003), placement around them (BAT-SET-001), walking and flying/teleporting
  (BAT-MOV-001/002; ability `teleports`), attack positions on footprints (BAT-ATK-001); action
  encoding v2, replay format v2; bench-check re-baselined (+18 % Ir on the changed workload:
  synthetic wisps now fly and brutes are two-hex; wall time per step about the same).
- [x] T4 Ranged combat: `Shoot { target, at }`, shots on stacks, blocked shooters, unblocked
  shooters may not melee (BAT-RNG-001), double shots, distance and melee penalties
  (BAT-RNG-002/003, Tier-2 parameters), area shots (BAT-RNG-004, abilities `area_shot`,
  `no_distance_penalty`); damage factors generalised (bonuses add, reductions multiply, one
  final floor); `Struck` events carry `StrikeFlags`; action encoding v3; bench re-baselined
  (+13 % Ir, synthetic wisps shoot; ≈ 225 k steps/s).
- [ ] T5 Melee abilities and damage factors: double attack, no/extra/unlimited retaliation,
  multi-hex attacks (hydra, Cerberus, breath), return after strike, jousting/charge immunity, hate,
  defense reduction, death blow, life drain, regeneration, death stare; exact combination of the
  damage factors (integer vs. float decision recorded).
- [ ] T6 Morale, luck and turn-order phases (modifiers in battle state; good morale extra action,
  bad morale freeze, wait-morale phase; luck in damage).
- [ ] T7 Terrain: battlefield types, native-terrain bonus, obstacle catalogue and random obstacle
  generation.
- [ ] T8 War machines (hero-free): data, Ballista, First Aid Tent, Ammo Cart, automatic control,
  turn order.
- [ ] T9 End-of-battle details (draw, war machines not counting as living), tight formation,
  golden replays, benchmark re-baseline, docs, review pass, closure.

## Risks

- The modifier system is the load-bearing design of E03–E04; do it first, keep it small, benchmark it.
- State size and clone cost will grow (effects per stack): keep them measured (`h3 bench`, `bench-check`).
