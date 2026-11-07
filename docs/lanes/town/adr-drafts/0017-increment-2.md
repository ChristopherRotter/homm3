# ADR-0017 amendment (draft): E09 increment 2, construction with the town events (the game side)

- Status: **Accepted as written** at T13's checkpoint (`TWN-Q10`, 2026-11-07; E09 decision 48),
  to be applied by T15 into ADR-0017 with batch TWN-2's outcome (decisions 41, 42: TD-11's row may
  revisit point 1's owner test). Drafted by E09 T13 (2026-11-07).
- Amends: ADR-0017 points 1, 2, 3, 4, 6, 7, 8, 9 and 10 (each text below goes in after the point's
  E09 increment 1 text, or after its increment 10 text where that comes later); its consequences,
  alternatives and revision history. Point 11 is not amended (a build changes the siege's facts
  through the record, ADR-0023 point 2's amendment).
- Choices: the epic's decisions 37, 41–44 and 45–47 (`TWN-Q9`). Detail: the design note
  `docs/epics/E09-design.md` Part D (§29 to §32); the town context's side `0028-increment-2.md`,
  the map's `0016-increment-2.md`.

## Context

The town context gains a command, the day's mark and the events' buildings (ADR-0028's draft);
the game composes them: the turn and the owner (the adventure's), the treasury, the day's start,
each player's turn start (the map's timed events, GAM-CAL-004), the capture, the views. Survey
`docs/rules/surveys/E09-construction.md` §1 to §4 (the build), §8 (the town events).

## Decision

**Point 1** (one entry point), after its E09 text: **E09 increment 2** (construction; decisions
37, 45) adds the game's first **town command**: `GameCommand::Town(TownCommand::Build { town,
building })`. Legal for a player in its turn with no open decision (as every map command) that
**owns the town** (TD-11 the owner, decision 42; batch TWN-2's row may revisit it), for a building
`h3-towns` finds buildable now in it (ADR-0028 point 2's draft: the hall's tests, the day's mark,
one Capitol a player over the player's towns) whose cost the treasury pays; no hero needs to be
in the town. Each refusal is typed and changes nothing. **The setup gains nothing**: the
requirements and costs are the part's, the forbidden sets and the town events the map's (already
in the towns' setups), the boat spot the adventure's (ADR-0016 point 1's draft).

**Point 2** (actors and viewers), after its E09 text: from increment 2 the observation gives the
**owner** of a town its buildings (ever built, forbidden, standing) and its day's mark, and every
viewer that sees the town whether it was built in today (decision 47, ADR-0016 point 6's draft); a
player whose hero enters an ally's town is told its buildings by the step (E10's teams).

**Point 3** (turn groups), after its E09 text: **the day's start** clears every town's day's mark
(the original's `new_day` towns' loop; GAM-CAL-002 amended; one a town a day for every player,
decision 43: the computer's every other day on Easy and Normal comes with the computer player).
**At each player's turn start**, right after the map's timed events, **the town events** fire
(decision 44; GAM-CAL-004's test): over the towns in `TownId` order (the map's objects' order) and
each town's events in the map's order, every event whose players include the player, whose kind
(human or computer) applies and whose day it is (its first day or every interval after it: an
interval of 1 from its first day, TD-12 the original's), **whose town the player owns**. Each
fired event's resources are added as a timed event's (the gains, then the losses floored at 0),
then its buildings are built (`h3-towns`' event buildings, in the part's order, with the boat
spot's water, without the day's mark, no requirement, forbidden, Capitol, Grail or cost test;
T-8's second Capitol stands), then its creatures are kept for increment 3's recruits
(`TWN-STUB-10`: lost until then). Day 1's fire with the first group's turn inside the build, as
the timed events'.

**Point 4** (streams), after its E09 text: increment 2 **draws nothing**: the build, the day's
mark, the events and the Shipyard's water are draw-free. At the build, after the adventure's static
map is built, a town whose faction has a building of role `shipyard` and whose object has no boat
spot gets that building forbidden and, if the setup built it, unbuilt (TWN-SET-011; `TWN-STUB-2`'s
Shipyard half lifted; its Grail half stays with increment 5 and E10's victory conditions): a write
of the store before the first turn, after every draw, so no stream position moves.

**Point 6** (pending decisions), after its E09 text: **increment 2 adds no decision kind**: a build
is a command, the town events need no answer (their messages are events).

**Point 7** (simultaneous play), the inventory: a town's record is written by its **owner's**
build, by the town events at its owner's turn start and by a capture (increment 1); its day's mark
by a build, the day's start and a capture. Two players' commands can write one town only through
a capture.

**Point 8** (`AdventureEnv`), after its latest increment's text: the env plays the build (E09 decision
7): the action encoding gains one entry, **build**, with the tile head naming the town's entrance
and a **building head** over the record's buildings (`h3_towns::MAX_BUILDINGS`), masked by the
legal builds; the AI lane's version and layout, `cross-lane` in its paths.

**Point 9** (versions), after its E09 text: **E09's increment 2** bumps the game state hash (the
store's v2 folded in, the town command), the adventure ruleset part (the kind `Town`'s offsets)
and the adventure replay format (the town command), each `cross-lane`; the town lane's part v4 and
state hash v2 (ADR-0028 point 9's draft); the action encoding the AI lane's.

**Point 10** (gains and costs by outcome), after its increment 10 text: **a build**: `h3-towns`
checks the town and the building and names the price (the building's cost); the game checks the
owner and the treasury before anything changes, then pays the price (`ResourceLost`'s payment, as
`BuildBoat`'s) and calls the store's infallible build with the day's mark. Its events: the
building built, to the town's owner alone; the town's public facts (fort level, Capitol, built
today) to every viewer that sees the town, and its top level's facts (hall, income) to the viewers
at that level (point 2), when they change (the original redraws the picture at a first fort or a
Capitol). **A capture** (increment 1's
application) also keeps the day's mark, clearing it when the old owner is not human and the new
one is (survey §3, decision 43). **A town event's** resources are a timed event's (gains saturating,
losses floored at 0, each told to the player); its buildings are told as a build's, its firing
by the town and the event's place in the town's list (the message is the client's).

## Consequences

- `GameCommand`, `GameCommandError` and `GameEvent` gain the town's variants (the adventure lane's
  `command.rs`, `cross-lane`; the server's wire maps no game command or event yet, so no
  `h3-protocol` change); `legal_commands` lists the builds after the boats, in `TownId` and
  building order.
- The day's start and the turn start each gain one call into the game's town module (the adventure
  lane's `state/day.rs`, `cross-lane`).
- Increment 3 gives the town events' creatures their recruits (`TWN-STUB-10` lifted) and a
  dwelling's first creatures at its build; increment 4 the heroes' visit after a build (the
  services, the guild's spells) and the Tower's reveals; increment 5 the Grail's building.

## Alternatives considered

- A visitor building at an ally's town from its own treasury (the reading's path, TD-11): kept for
  batch TWN-2's verdict (decision 42).
- The computer's every other day on Easy and Normal now (decision 43): no computer player yet.
- The town events in the player's order of its towns (VCMI's): the original's list is the map's.
- Firing an every-day event from the day before its first (VCMI's, TD-12): the original's day.

## Revision history

- 2026-11-07: Drafted (E09 T13) with the user's answers at `TWN-Q9` (decisions 45–47); accepted as
  written at `TWN-Q10` (decision 48).
