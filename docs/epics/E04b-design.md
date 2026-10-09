# E04b design note (working detail for ADR-0014)

A handover, not binding: proposals and sizes for the tasks of ADR-0014 (accepted). The T2
sections are implemented and deleted (code docs, `packs/README.md`, HER-BAT-011 to 013,
BAT-MAG-001 to BAT-MAG-008, `h3_sim::action`); so is T3's effect list (T3a: `h3_battle::effects`,
BAT-MAG-009 to 011, HER-BAT-014), T5's battlefield objects (`h3_battle::obstacles`,
`h3_battle::natural`, BAT-MAG-027 to 034), T6a's raising (`h3_battle::raise`, BAT-MAG-035/036),
T6b's created stacks (`h3_battle::created`, BAT-MAG-037 to 039) and T7's creature casters
(`h3_battle::casters`, BAT-MAG-048 to 052; the Pit Lord's Demons as `StackOrigin::Raised`, ADR-0014
point 6). No section is open.
Each task
settles its section and **deletes it** when the code documentation, the rule catalog and the pack
docs hold the detail; the file is deleted when E04b closes (ADR index rule). Names are provisional.
Sources: [`docs/rules/surveys/E04b-magic.md`](../rules/surveys/E04b-magic.md).
