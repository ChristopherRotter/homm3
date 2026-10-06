# E04b design note (working detail for ADR-0014)

A handover, not binding: proposals and sizes for the tasks of ADR-0014 (accepted). The T2
sections are implemented and deleted (code docs, `packs/README.md`, HER-BAT-011 to 013,
BAT-MAG-001 to BAT-MAG-008, `h3_sim::action`); so is T3's effect list (T3a: `h3_battle::effects`,
BAT-MAG-009 to 011, HER-BAT-014). Each task
settles its section and **deletes it** when the code documentation, the rule catalog and the pack
docs hold the detail; the file is deleted when E04b closes (ADR index rule). Names are provisional.
Sources: [`docs/rules/surveys/E04b-magic.md`](../rules/surveys/E04b-magic.md).

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
