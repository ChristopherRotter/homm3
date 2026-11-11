# Draft: ADR-0016 amended for E09 increment 3 (growth, recruitment and the army rows)

- Status: **Draft**, growth's half (E09 T22a, 2026-11-11); T22b adds the rows' half (the army
  reference agreed with E08's increment 12, a garrison hero) and puts the whole to the user at
  its checkpoint (`TWN-Q14`); moved into `docs/adr/0016-adventure-state.md` by the `shared/`
  task after batch TWN-3.
- Decisions: the epic's 65, 72 (TD-15 by the reading), 75, 78.
- Working detail: survey `docs/rules/surveys/E09-growth.md` §4, §5; design note
  `docs/epics/E09-design.md` Part E §44, §45.

## Point 1, E09's towns: replaces "the setup's (its random stacks resolved when the game is built, ADR-0017 point 4) until a rule writes it"

the setup's (its random stacks resolved when the game is built, ADR-0017 point 4) until a rule
writes it: **from E09's increment 3** a neutral town's garrison week (the game writes the stacks
`h3-towns`' rule returns, through the town module's writer; TWN-GRO-006).

## Point 1, E09's towns, a new sentence after it

The game reads, for a town's growth and its Portal, **each dwelling's owner and its kind's
creatures** in object order (`StaticMap::dwellings`, `AdventureState::owner`; ADR-0028 point 6):
the external dwellings count by their first creature (the original's record +4,
`town_change_owner` 0x4C62B5), the Portal draws one of the owner's dwellings (classes 17 and
20; never a Refugee Camp), then one of its kind's creatures. No adventure state is added for
growth.

## Revision history, a new item (when moved)

- 2026-11-xx (E09 T2x `shared/`): point 1 amended for E09 increment 3. Drafted in E09 T22a and
  T22b.
