# Draft: ADR-0016 amended for E09 increment 5 (a town's visit named in its outcome)

- Status: **Draft**, written by E09 T43 (2026-11-20), **accepted** at its checkpoint (`TWN-Q33`,
  decisions 178 to 181); then moved into `docs/adr/0016-adventure-state.md` by the increment's `shared/`
  task.
- Decisions: the epic's 170 (TD-27: the original's triggers), 175 (the opening), 177.
- Working detail: survey `docs/rules/surveys/E09-guild.md` §12; design note
  `docs/epics/E09-design.md` Part G §89.

## Point 4, E09's kind `Town`, after "**E09's increment 2 adds no visit rule** …"

- **E09's increment 5** (TWN-GLD-005; TD-27, E09 decision 170): the step that **enters** a town
  (one of the team, or an undefended one it captures) and a `Revisit` of a town of the team name the
  town in the visit's outcome (`AdventureOutcome::town_entered`, beside `capture`), which the game
  composes as the town's visit (ADR-0017 point 10: the Vortex, then the town's heroes learn; the
  original's `town_visit` 0x4AAFD0 opens the town screen and teaches at its end). The adventure
  gives nothing itself and holds no guild fact; the outcome is not state (no hash moves). A siege
  names no town entered (the attacker stays outside); the game teaches after its battle.

## Revision history, a new item (when moved)

- 2026-11-xx (E09 increment 5's `shared/` task): point 4's kind `Town` (the town entered named in
  the visit's outcome).
