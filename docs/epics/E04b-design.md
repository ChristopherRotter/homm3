# E04b design note (working detail for ADR-0014)

A handover, not binding: proposals and sizes for the tasks of ADR-0014 (accepted). The T2
sections are implemented and deleted (code docs, `packs/README.md`, HER-BAT-011 to 013,
BAT-MAG-001 to BAT-MAG-008, `h3_sim::action`); so is T3's effect list (T3a: `h3_battle::effects`,
BAT-MAG-009 to 011, HER-BAT-014). Each task
settles its section and **deletes it** when the code documentation, the rule catalog and the pack
docs hold the detail; the file is deleted when E04b closes (ADR index rule). Names are provisional.
Sources: [`docs/rules/surveys/E04b-magic.md`](../rules/surveys/E04b-magic.md).

## T5 — battlefield objects

Force Field, Fire Wall, Teleport, Expert Dispel's clearing, the hidden obstacles, Quicksand and
Land Mine are implemented (T5a, T5c1, T5c2: `h3_battle::obstacles`, BAT-MAG-027 to 033). Left:
T5b (Remove Obstacle and the natural obstacles as pieces with their anchors): waits for the
user's class and seam option for Q55 (`docs/OPEN-QUESTIONS.md`); its target test must not show
the other side's hidden pits and mines (Q-0002).

## T6 — created stacks

- The creation limit (20, the original's) becomes a battle parameter checked by Clone and Summon
  Elemental; Summon Elemental also checks the side's first elemental type. The 21 slots per side
  exist since T2c (`SIDE_STACK_SLOTS`).

- `StackOrigin` gains `Summoned`, `Clone(of)` and `Raised`. Pit Lords' demons replace the dead
  stack they rise from in its slot. A per-stack count of temporarily raised creatures (Basic
  Resurrection) is subtracted before the outcome. Summoned stacks and clones are reported with
  their origin and never survive. Open: whether a vanished clone frees its slot (§7.7; an
  experiment can clone and kill repeatedly until the limit message appears).
