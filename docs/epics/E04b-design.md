# E04b design note (working detail for ADR-0014)

A handover, not binding: proposals and sizes for the tasks of ADR-0014 (Proposed). Each task
settles its section and **deletes it** when the code documentation, the rule catalog and the pack
docs hold the detail; the file is deleted when E04b closes (ADR index rule). Names are provisional.
Sources: [`docs/rules/surveys/E04b-magic.md`](../rules/surveys/E04b-magic.md).

## T2a — content and the hero's magic

- **Pack format v4, `[[spell]]`:** `target` (`none`, `stack`, `hex`, `obstacle`, `stack_and_hex`
  for Teleport, `dead_and_stack` for Sacrifice); `flags` harvested from the executable's flag word
  (timed, single, mass at Expert, area, damage, mind; the unclear bit `0x1000` is not read until
  dispute M-1 is settled); `effects` per mastery, T2 only `{ kind = "damage" }`, whose value is
  `power × power factor + effect[mastery]`; target conditions in creature-flag vocabulary
  (Death Ripple: not `undead`; Destroy Undead: only `undead`; Implosion: not `war_machine`).
- **Hero effects** (content `HeroEffect`, resolved to `BattleEffect` or typed fields): the magic
  skills set `BattleMagic.schools`; Sorcery and the Orbs a spell-damage bonus in basis points (all
  schools, or one school); a spell specialty of a damage spell scales by
  `1 + 0.03 × hero level / target creature level` (`E04-heroes.md` §3, SoD). Other specialties
  come with their spells' tasks.
- **Derivation:** `spell_book = Some(spells)` when the hero has a spell book (the full "worn spell
  book" rule and the Tomes are T8); `max_mana` stays informative. Rules continue `HER-BAT`.

## T2b — casting in battle

- **State:** per side the hero's mana (`u16`, from `BattleMagic.mana`), casts this round, and the
  distinct spells cast in first-cast order (fixed list, capacity the content's spell limit; the
  `HeroOutcome.spells_cast` doc changes to "distinct, in first-cast order"). Counters reset when a
  round starts.
- **Command:** `HeroCast { spell, target, second }` with targets `None | Hex | Stack`. Legality
  (`BAT-MAG-001`): a stack of the actor's side awaits a command; not the tactics phase; no cast yet
  this round (a battle parameter, default 1, as VCMI's bonus); the spell in the book and a combat
  spell; mana at least the cost; a valid target. The creature cost modifiers (Mages, Pegasi,
  Familiar's channelling, Wraiths' drain) arrive with their abilities in T7; the cost rule
  (`BAT-MAG-002`) names them in its scope until then.
- **Damage** (`BAT-MAG-003`): VCMI's order is protection by school (T3), the general reduction
  (Golems, T2d), more damage from a spell (elementals, T2d), then the caster's Sorcery, specific
  spell bonus, best school bonus (Orbs) and the level-scaled specialty, truncating after each
  factor. Rounding in SoD: batch 2a X-1 (Q-0005 used single precision for physical damage).
  Damage kills like physical damage; no luck, morale or retaliation.
- **Events:** a public `SpellCast` (side, caster, spell, targets); damage per stack; resisted and
  reflected stacks (T2d).
- **Observation:** casts this round public; the spell book in `own_hero` (private). Mana is
  already public.

## T2c — the sim and the extra stack slots

- **Extra stack slots:** proposal 8 per side (clones of a seven-slot army plus one elemental
  stack; demons share them); `STACK_CAPACITY` = 2 × (16 + 3 + 8) = 54. The original's limit is
  unknown (M-5); a cast that needs a slot when none is free is not legal.
- **Action encoding v9:** the existing blocks in their order, the stack-indexed ones grown to 54
  slots, then two entries `HERO_CAST` and `CREATURE_CAST`. Spell head: `MAX_SPELLS` (content
  limit, proposal 128; the compiler rejects more). Target heads: `0` none, `1 + h` for hex
  `h < 255`, `256 + s` for stack slot `s` (310 entries); the second target has the same layout.
- **Masks:** per head, conditioned on the earlier choices (entry; entry and spell; entry, spell
  and first target). A property test samples full tuples: a tuple the masks allow applies, and
  `apply` accepts nothing the masks forbid.
- **C ABI v8:** an action struct `{ entry: u32, spell: u16, target: u16, second: u16 }`; one mask
  query per head. The random agent picks an entry among the legal ones, then spell and targets.
- **Observation encoding v5:** casts this round per hero, the own spell book as a bit set; per-stack
  effects follow in T3. The `full` workload gives both heroes a spell book and mana.

## T2d — area, chain and battle-wide spells; resistance and immunities

- **Shapes:** Frost Ring the hexes at distance 1 without the centre; Fireball and Meteor Shower
  radius 1; Inferno radius 2; Death Ripple, Destroy Undead, Armageddon every stack meeting the
  condition. Chain Lightning's targets and falloff are surveyed in T2d (X-6).
- **Ability variants** (ADR-0012): immunity to spells up to a level (Green, Red, Azure, Gold,
  Black Dragons, Magic Elemental), to a school (fire), to one spell; magic resistance (Dwarves,
  Crystal Dragon); a resistance aura for adjacent allies (Unicorns); spell damage reduction
  (Golems); more damage from a spell (elementals). The executable's `mind_immune` and
  `fire_immune` flags are reconciled with VCMI's lists.
- **Hero resistance:** the Resistance skill and artifacts as a `BattleEffect` applied at setup to
  the side's stacks; adding or multiplying with the creature's own is M-2 (VCMI adds, the aura
  multiplies).
- Quirk S-18 (Magic Arrow can deal 0 damage to Gold and Diamond Golems) is checked by X-1.

## T3 — effect slots

- **Layout options:** (a) an array per stack indexed by a dense timed-spell index the compiler
  assigns (limit `MAX_TIMED_SPELLS`, proposal 48; SoD has about 34 including abilities); an entry is
  rounds left (`u8`; 0 inactive, 255 permanent), mastery and end-condition bits, and a `u16` value,
  so 4 bytes × 48 = 192 bytes per stack and about 10 KB for 54 stacks (today's state: 2 192 bytes,
  clone ≈ 183 ns). (b) The same capacity as a list of active entries. Measure clone cost and
  steps/s before choosing; the original keeps one 8-bit duration per spell and stack (S-17).
- **Materialisation:** base stats (setup) → slot effects in fixed stages (add, percent, bounds, like
  KER-MOD-001) → rule clamps; recomputed when a slot changes.
- **Expiry:** one fixed point of the round for all stacks (M-3); end conditions checked by the
  rules that cause them (attacked, attacks).

## T5 — battlefield objects

- A spell obstacle: kind, owner side, hexes, revealed, rounds left, value (mine damage). Capacity
  by construction: at most one spell obstacle of a kind per hex, so kinds × field hexes.
- Visibility as ADR-0014 point 7; legality and reachability use the acting side's view; triggering
  reveals and emits a public event.

## T6 — created stacks

- `StackOrigin` gains `Summoned`, `Clone(of)` and `Raised`. A per-stack count of temporarily raised
  creatures (Basic Resurrection) is subtracted before the outcome. Summoned stacks and clones are
  reported with their origin and never survive; Pit Lord demons are reported as new stacks for the
  orchestrator to place.
