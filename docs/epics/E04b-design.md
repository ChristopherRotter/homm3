# E04b design note (working detail for ADR-0014)

A handover, not binding: proposals and sizes for the tasks of ADR-0014 (accepted). The T2
sections are implemented and deleted (code docs, `packs/README.md`, HER-BAT-011 to 013,
BAT-MAG-001 to BAT-MAG-008, `h3_sim::action`); so is T3's effect list (T3a: `h3_battle::effects`,
BAT-MAG-009 to 011, HER-BAT-014). Each task
settles its section and **deletes it** when the code documentation, the rule catalog and the pack
docs hold the detail; the file is deleted when E04b closes (ADR index rule). Names are provisional.
Sources: [`docs/rules/surveys/E04b-magic.md`](../rules/surveys/E04b-magic.md).

## T5 — battlefield objects

The spell obstacle list, Force Field, Fire Wall, Teleport and Expert Dispel's clearing are
implemented (T5a: `h3_battle::obstacles`, BAT-MAG-027 to 030, BAT-MOV-003), so are the hidden
obstacles and Quicksand (T5c1: BAT-MAG-031/032, quirks BAT-QUIRK-1/2). For T5c2 (Land Mine):

- A mine is a hidden obstacle kind of its own (pack format: an obstacle kind `mine`) with the
  spell's value in `SpellObstacle::value`; its placement as Quicksand's (a random shape), its
  cast test (the spell book refuses it while the other side had natives at the battle's start,
  0x59ECEE: the faithful sight of BAT-QUIRK-1; the fix needs a rule of its own), its trigger
  (0x59FB40, between Fire Wall's and the moat's when a hex is entered) and who crosses safely:
  read them first (`installation.md` §7.15 lists the addresses).
- `obstacles::travel_through` enters the hexes in order and already handles a stop and a death
  on the way; a mine that goes off goes (ObstacleRemoved) and its reveal follows BAT-MAG-031.

## T6 — created stacks

- The creation limit (20, the original's) becomes a battle parameter checked by Clone and Summon
  Elemental; Summon Elemental also checks the side's first elemental type. The 21 slots per side
  exist since T2c (`SIDE_STACK_SLOTS`).

- `StackOrigin` gains `Summoned`, `Clone(of)` and `Raised`. Pit Lords' demons replace the dead
  stack they rise from in its slot. A per-stack count of temporarily raised creatures (Basic
  Resurrection) is subtracted before the outcome. Summoned stacks and clones are reported with
  their origin and never survive. Open: whether a vanished clone frees its slot (§7.7; an
  experiment can clone and kill repeatedly until the limit message appears).
