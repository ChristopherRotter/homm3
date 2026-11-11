# Draft: ADR-0017 amended for E09 increment 3 (growth, recruitment and the army rows)

- Status: **Accepted** at T22b's checkpoint (`TWN-Q14`, 2026-11-11; E09 decisions 80 to 83, each
  as recommended); drafted by E09 T22a (growth) and T22b (recruitment, the rows), to be moved into
  `docs/adr/0017-game-orchestration.md` by the `shared/` task after batch TWN-3 (T24).
- Decisions: the epic's 65, 69 to 73, 74 to 77 (`TWN-Q13`: the towns' stream, decision 75; the
  week's kind in the game, decision 77), 78 (the agent's calls), 79, 80 to 83 (`TWN-Q14`); E08's 305.
- Working detail: survey `docs/rules/surveys/E09-growth.md` §2 to §10; design note
  `docs/epics/E09-design.md` Part E §41 to §46 (growth), §49 to §54 (recruitment, the rows).

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
order); **at a capture** that finds a standing Portal empty, from the capturer's stream after the
capture's own draws (decision 82: a human's capture opens the town screen, which draws at once).
No other command of growth draws: the counts, the first creatures and the events'
creatures are draw-free. The towns' stream is server-only (point 5) and hashed with the state.

## Point 1, the commands: after E09 increment 2's `GameCommand::Town`

**E09 increment 3** adds **`TownCommand::Recruit { town, building }`** (ADR-0028 point 6: opens
the hire dialog, point 6) and the adventure's **`AdventureCommand::TownHeroes { town, change }`**
(`Enter`, `Leave`, `Swap`: the garrison hero's moves, ADR-0016 point 3), each its town's owner's,
in its turn, with no decision open (GAM-DEC-001), a typed refusal changing nothing.

## Point 3, the turn's start: after "the maximum and the points left of each of its heroes"

**From E09 increment 3** (ADV-MOV-002 amended; TWN-ROW-005) each of the player's **garrison
heroes** gets its day's maximum and points too, after its heroes on the map, in `TownId` order
(the original's `next_player_turn` 0x4C7486: the towns' +0xC heroes); the day's spell points
reach every hero as now; the adventure's day and week starts drop a garrison hero's day and week
marks as a hero on the map's (E08's increment 12; the original's every hero record, `new_day`
0x4C80CF, 0x4C8D2A). A garrison hero's **income** (`h3_heroes::daily_income`: its Estates and its
worn resource artifacts) and its army's **weekly resources** (GAM-RES-005: the Crystal Dragons in
an owned town's upper bar, a garrison hero's army or else the town's garrison) count with its
owner's (GAM-RES-003 amended), once: the original takes a garrison hero out of its owner's list
(`town_visitor_to_garrison`), so nothing counts twice.

## Point 2 (viewers), a new sentence

**E09 increment 3:** the week's kind is public (`WeekStarted`, the observation's calendar, as the
month's); a town's available creatures and its Portal are its owner's (the town view's owner
part, TWN-VEW-004's level; `docs/06-security.md`'s town section).
A town's **garrison hero** is seen in its town's view at the level that shows the garrison's army
(TWN-VEW-001, level 3) and by its owner among its own heroes (ADR-0016 point 6); the hire
dialog's view is its owner's, as on the map.

## Point 6, pending decisions: after increment 10's hire dialog ("E09's recruitment reuses the kind …")

**E09 increment 3** opens the hire dialog from `TownCommand::Recruit` with **a town's source**
(an index's dwelling or horde, or the Portal of Summoning) and **the town's upper bar as its
target** (the garrison hero's army when one stands, else the town's garrison: the original's
`town_garrison_army`; never the visitor's, survey `E09-growth.md` §8), opened whatever the count
(0 too, the original's dialog); a Portal with nothing summoned is refused instead. The dialog's
rules are the map's: the legal answers each creature's most, `Close`; it stays open while it
offers two creatures (an upgraded index: the upgrade and its base from one count) and closes
after the purchase of a one-creature source. Its view and `HireOpened` name the town and the
index or the Portal in place of an object's type and anchor.
**The town's two rows open no decision** (TWN-ROW-001; decision 80): in its turn with
no decision open, the owner's army commands (`Manage::Army`) may name an own town's upper bar
alone or with that town's visitor's army, as the original's town screen, which opens at will and
needs no answer; the game admits the pair as it admits a meeting's two heroes (GAM-MAN-001
amended), by E08's one owner rule. E08's `Exchange` (GAM-DEC-009) stays the map garrison's,
opened by a visit. Neither the hero moves (point 1) nor the rows' commands draw.

## Point 8, the env: after E09 increment 2's `BUILD`

**E09 increment 3** appends two entries' groups (`cross-lane`, the AI lane's paths, E09 decision
7): **recruitment** (one entry: the town by its entrance in the tile head, the building in the
building head, one action per `Recruit` the legal commands list; encoding v10), whose purchases
are the hire dialog's four entries unchanged (a town's source offers at most two creatures), and
**the hero moves** (`Enter`, `Leave`, `Swap`, each the town by its entrance in the tile head;
encoding v11, or with v10 when they land together). The env plays no army management (no entry
before or after), so the rows' stack moves are a client's.

## Point 9, after E09's rows

**E09 increment 3's growth** bumps the game state hash (the week's kind, the towns' stream, the
store's v3; `cross-lane`), the protocol (`WeekStarted`'s kind, the observation's week, the owner's
town view; the server lane's, `cross-lane`) and re-records the adventure replays; no command of
growth, so neither the replay format nor the env's action encoding moves.
**Its recruitment and rows** bump the game state hash (the hire dialog's town sources and target),
the adventure state hash (a town's garrison hero; ADR-0016 point 1), the adventure replay format
(`town recruit`, the hero moves, management lines naming a town's bar), the env's action
encoding (point 8) and the protocol (the commands, `HireOpened`'s and the view's source, the
garrison hero's events and views), each `cross-lane` and in merge order with E08's increment 12
(the numbers are the next at merge; the second to merge bumps again).

## Point 10, a new sentence

**E09 increment 3:** a neutral town's garrison week returns the garrison as stacks with the
town's lowered counts; the game writes both, the army into the adventure (ADR-0016 point 1) and
the counts into the store, in one infallible application.

**E09 increment 3's recruitment** (TWN-REC-001): a purchase's application is infallible and in
this order: the price paid, the source's count lowered in the store, the creatures joined into
the target army by the adventure (its `StackChanged` to the owner, the town's viewers told the
upper bar's change at their level). **The hero moves** are the adventure's own changes (ADR-0016
point 3), told as heroes leaving and entering the map are, and the town's change to its viewers.

## Point 11, E09's sieges: after "A siege of an ally's visitor is refused by the step"

**From E09 increment 3** (GAM-BAT-001, TWN-SIE-001 amended): a town with a garrison hero and a
visitor is defended by **the visitor in a battle that is not a siege** (`siege: None`, the
gate tile's battlefield as any hero's battle, no town bonuses), after which the town stays its
owner's whatever the outcome; a garrison hero alone defends **the siege** with its own army and
its town bonuses (ADR-0023 point 3), and a beaten one goes back to the pool as a beaten hero; an
empty garrison hero yields the town without a battle (T-5, `TWN-QUIRK-11`).

## Revision history, a new item (when moved)

- 2026-11-xx (E09 T24 `shared/`; decisions 65 to 83): points 1 to 4, 6, 8 to 11
  amended for E09 increment 3. Drafted in E09 T22a and T22b.
