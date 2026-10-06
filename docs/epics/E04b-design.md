# E04b design note (working detail for ADR-0014)

A handover, not binding: proposals and sizes for the tasks of ADR-0014 (accepted). T2a's, T2b's
and T2c's sections are implemented and deleted (code docs, `packs/README.md`, HER-BAT-011/012,
BAT-MAG-001 to BAT-MAG-004, `h3_sim::action`). Each task
settles its section and **deletes it** when the code documentation, the rule catalog and the pack
docs hold the detail; the file is deleted when E04b closes (ADR index rule). Names are provisional.
Sources: [`docs/rules/surveys/E04b-magic.md`](../rules/surveys/E04b-magic.md).

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

## T3 — the effect list

- **Layout:** one `Vec` of active entries per battle (stack, spell, rounds left or end condition,
  mastery, a value), sorted by (stack, spell); reserved at the start (proposal: 32 entries, measure
  real battles) so steady play does not allocate. An entry of about 8 bytes; a stack's entries are
  a contiguous run found by binary search. A fork is one allocation plus the active entries (today:
  2 192 bytes inline, clone ≈ 183 ns).
- **Materialisation:** a stack's effective stats and status bits are recomputed from its base
  stats (setup) and its entries in fixed stages (add, percent, bounds, like KER-MOD-001), then the
  rule clamps, whenever one of its entries changes; hot-path reads use the inline result.
- **Expiry:** one fixed point of the round for all stacks (M-3); end conditions are checked by the
  rules that cause them (attacked, attacks). The original keeps one 8-bit duration per spell and
  stack (S-17).

## T5 — battlefield objects

- A spell obstacle: kind, owner side, hexes, revealed, rounds left, value (mine damage), in a
  growable list like the effects (ADR-0014 point 5).
- Visibility as ADR-0014 point 7; legality and reachability use the acting side's view; a move
  that meets an unseen pit stops there (an attack ordered with it does not happen); triggering
  reveals and emits a public event.

## T6 — created stacks

- The creation limit (20, the original's) becomes a battle parameter checked by Clone and Summon
  Elemental; Summon Elemental also checks the side's first elemental type. The 21 slots per side
  exist since T2c (`SIDE_STACK_SLOTS`).

- `StackOrigin` gains `Summoned`, `Clone(of)` and `Raised`. Pit Lords' demons replace the dead
  stack they rise from in its slot. A per-stack count of temporarily raised creatures (Basic
  Resurrection) is subtracted before the outcome. Summoned stacks and clones are reported with
  their origin and never survive. Open: whether a vanished clone frees its slot (§7.7; an
  experiment can clone and kill repeatedly until the limit message appears).
