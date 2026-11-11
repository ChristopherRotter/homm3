# Draft: ADR-0017 amended for E09 increment 3 (growth, recruitment and the army rows)

- Status: **Draft**, growth's half (E09 T22a, 2026-11-11); T22b adds recruitment's and the
  rows' half (the hire, the army reference, a garrison hero) and puts the whole to the user at
  its checkpoint (`TWN-Q14`); moved into `docs/adr/0017-game-orchestration.md` by the `shared/`
  task after batch TWN-3.
- Decisions: the epic's 65, 69 to 73, 74 to 77 (`TWN-Q13`: the towns' stream, decision 75; the
  week's kind in the game, decision 77), 78 (the agent's calls).
- Working detail: survey `docs/rules/surveys/E09-growth.md` §2 to §6; design note
  `docs/epics/E09-design.md` Part E §41 to §46.

## Point 1, after E09 increment 2's sentences on the state

**E09 increment 3:** `GameState` holds **the week's kind** beside the month's (decision 77):
normal, a creature's week (one creature, the parameters' bonus) or the week of a `week_of`
building (its level's two creatures of its faction, the base creature's growth as the bonus),
and **the towns' stream** (point 4). The setup gains **whether the map is RoE's** (the import's
format), which only the week's draw reads.

## Point 3, the day's start: replaces "The week's kind (a week of a creature) and its towns' effects come with E09 (E08 decision 197)" and the two sentences on the Plague and the week's creature

**From E09 increment 3** (GAM-CAL-002 amended): on the first day of a week, before
`WeekStarted`, **the week's kind** is drawn (TWN-GRO-002: on a month's first day no creature's
week; else 1 in 4 a creature's, uniform over the town records' creatures, on an RoE map those of
the records with `roe`; then a standing `week_of` building in any town, the first in `TownId`
order, makes it that level's week whatever was drawn) and `WeekStarted` carries it; then **every
town's week** in `TownId` order (TWN-GRO-003); then the adventure's week step as before; then
**each neutral town's garrison week** (TWN-GRO-006) and **each owned town's Portal** (TWN-GRO-007),
each in `TownId` order (the original's: after the map's weekly objects). On the first day of a
month the month's kind is drawn as before but, in a month whose first week is a `week_of` week,
it becomes that level's base creature's month without the creature's draw (GAM-CAL-003 amended;
`new_month` 0x4C8F63: its doubling skipped, its monsters spawned); then **every town's month** in
`TownId` order (TWN-GRO-004), before the adventure's month step. The game's first week is
normal. The towns' day (the marks, the views told) follows, as increment 2's, and tells each
owner the towns whose counts or Portal the week or the month moved.

## Point 4, after "**E09's increment 2 draws nothing**: …"

**E09's increment 3** draws from **the towns' stream** (a named kernel stream, decision 75): at a
new week its kind (one draw of four unless it is a month's first week, then the creature's), then
each neutral town's garrison week (its odds; when it grows, three for the level, then for a base
creature that found room one for the upgrade), then each owned Portal's (a dwelling, then one of its creatures),
in that order. The month's kind stays the game's stream's; after a `week_of` week its creature is
not drawn. **The Portal's other draws** (decision 71): at its build by a command from the
builder's stream, after the build is checked (a refused build draws nothing); at a town event's
build from the towns' stream; at the setup from the setup stream after every existing
placeholder (a town that starts with a Portal and an owner); when its owner flags a dwelling while
it has none, from the visitor's stream after the visit's own draws (each such Portal in `TownId`
order). No other command of growth draws: the counts, the first creatures and the events'
creatures are draw-free. The towns' stream is server-only (point 5) and hashed with the state.

## Point 2 (viewers), a new sentence

**E09 increment 3:** the week's kind is public (`WeekStarted`, the observation's calendar, as the
month's); a town's available creatures and its Portal are its owner's (the town view's owner
part, TWN-VEW-004's level; `docs/06-security.md`'s town section).

## Point 9, after E09's rows

**E09 increment 3's growth** bumps the game state hash (the week's kind, the towns' stream, the
store's v3; `cross-lane`), the protocol (`WeekStarted`'s kind, the observation's week, the owner's
town view; the server lane's, `cross-lane`) and re-records the adventure replays; no command of
growth, so neither the replay format nor the env's action encoding moves.

## Point 10, a new sentence

**E09 increment 3:** a neutral town's garrison week returns the garrison as stacks with the
town's lowered counts; the game writes both, the army into the adventure (ADR-0016 point 1) and
the counts into the store, in one infallible application.

## Revision history, a new item (when moved)

- 2026-11-xx (E09 T2x `shared/`; decisions 65 to 78 and T22b's): points 1 to 4, 9 and 10
  amended for E09 increment 3. Drafted in E09 T22a and T22b.
