# E04b design note (working detail for ADR-0014)

A handover, not binding: proposals and sizes for the tasks of ADR-0014 (accepted). The T2
sections are implemented and deleted (code docs, `packs/README.md`, HER-BAT-011 to 013,
BAT-MAG-001 to BAT-MAG-008, `h3_sim::action`); so is T3's effect list (T3a: `h3_battle::effects`,
BAT-MAG-009 to 011, HER-BAT-014), T5's battlefield objects (`h3_battle::obstacles`,
`h3_battle::natural`, BAT-MAG-027 to 034) and T6a's raising (`h3_battle::raise`, BAT-MAG-035/036).
Each task
settles its section and **deletes it** when the code documentation, the rule catalog and the pack
docs hold the detail; the file is deleted when E04b closes (ADR index rule). Names are provisional.
Sources: [`docs/rules/surveys/E04b-magic.md`](../rules/surveys/E04b-magic.md).

## T6b — created stacks

T6a (raising: Resurrection, Animate Dead, Sacrifice) is implemented (`h3_battle::raise`,
BAT-MAG-035/036). What T6b needs:

- The creation limit (20, the original's) becomes a battle parameter checked by Clone and Summon
  Elemental; Summon Elemental also checks the side's first elemental type. The 21 slots per side
  exist since T2c (`SIDE_STACK_SLOTS`).
- Nothing new crosses the seam (the user, 2026-10-08; ADR-0014 points 6 and 10 amended in a
  `shared/` pull request): summoned stacks and clones never survive and the battle leaves them out
  of the outcome; no `StackOrigin` variant. Read in T6b whether the original's end of battle
  counts killed summoned or cloned stacks for experience (VCMI does not). A summoned stack leaves
  no corpse (the original's flag `0x400000`, `installation.md` §7.18), as a sacrificed one.
  Open: whether a vanished clone frees its slot (§7.7; an experiment can clone and kill
  repeatedly until the limit message appears).
- Pit Lords' demons (T7) replace a dead stack for good: the one case that may cross the seam.
