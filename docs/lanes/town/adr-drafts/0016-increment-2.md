# ADR-0016 amendment (draft): E09 increment 2, construction with the town events (the map side)

- Status: **Accepted as written** at T13's checkpoint (`TWN-Q10`, 2026-11-07; E09 decision 48),
  to be applied by T15 into ADR-0016 with batch TWN-2's outcome (scored by T14c: nothing here
  changes). Drafted by E09 T13 (2026-11-07).
- Amends: ADR-0016 points 1, 4 and 6 (each text below goes in after the point's E09 increment 1
  text, or replaces the sentence it names); its consequences and revision history.
- Choices: the epic's decisions 39 (T11), 46 and 47 (`TWN-Q9`). Detail: the design note
  `docs/epics/E09-design.md` Part D (§28 the water, §31 the views); the town context's side
  `0028-increment-2.md`, the game's `0017-increment-2.md`.

## Context

A Shipyard needs water: the original fixes a town's boat spot once at the start, the first of the
tiles one column left and right of the entrance, two rows below it, that is on the map, water, not
blocked and holds no object but a boat (`town_boat_spot` 0x5C0C90, survey `E09-construction.md`
§2.5), and forbids the Shipyard without one (`town_buildings_setup` 0x5C08C0, survey
`E09-towns.md` §5.3: a custom set's built Shipyard is not kept). A map's Shipyard object has the
same kind of tile (ADV-SHY-001). Increment 1 left both to increment 4 (`TWN-STUB-2`); the user
moved the town's water into increment 2, the boat itself staying increment 4's (decision 46).
E09 T11 resolved the random dwellings tied to a town, which point 4 still calls blockers.

## Decision

**Point 1** (storage), after its E09 item: **increment 2** gives a town object's static part its
**boat spot**, fixed when the game is built with the Shipyards' boat tiles (ADV-SHY-001's step,
after every object is placed) and never moved: from the town's entrance, the first of the kind
`Town`'s offsets (the adventure part: the original's (−1, +2), then (+1, +2), column and row) on
the entrance's level that is a tile of the map on water that no object blocks or is visited from
(a boat's tile allowed, the state's boats being no objects); none: the town has no water. It is
held for every town, whatever its faction; the game reads it at the build (ADR-0017 point 4's
draft: a Shipyard forbidden without it) and for the town events' Shipyard; increment 4's town boat
is built on it. No other town state becomes the adventure's.

**Point 4** (kinds), in its increment 10 random dwellings: the sentence "A random dwelling tied to
a town is a blocker until E09's random towns resolve it (ADV-STUB-13)" becomes: **A random
dwelling tied to a town** (216 or 217 whose identifier a town of the map has; the last such town
in object order, as the original searches its list from the end) is typed with that town's
`TownId`, takes the town's faction (drawn first, ADR-0017 point 4) and draws only its level (E09
T11, decision 39). In the kind `Town`, after its increment 1 items: **increment 2 adds no visit
rule**: a build is the town command (ADR-0017 point 1's draft), never a step, and needs no hero
in the town (the original builds from the town list).

**Point 6** (information), after its E09 text: **from E09 increment 2** (decision 47, the
original's views): a town's **buildings** (its ever-built and forbidden sets, so its standing
ones) are its **owner's**, as the town screen opens from the town list at any time; a player one
of whose heroes enters an **ally's** town learns them at that visit, as the original's town screen
opens for an allied town (survey `E09-towns.md` §7.2), told by the entering step to that player
alone (with E10's teams: until then no town is an ally's). **Whether a town was built in today**
is public where the town is seen (the popup's portrait, survey §12.3); its fort and Capitol stay
public as the picture shows them, changing with a build. The boat spot is told to no one in
increment 2 (whether the town may build a Shipyard is in its owner's buildings). The game composes
these views (ADR-0017 point 2's draft); `docs/06-security.md` lists them.

## Consequences

- The adventure ruleset part bumps (the kind `Town`'s offsets); the static map's hash holds the
  town's boat spot as it holds a Shipyard's (its encoding unchanged; the states of maps with towns
  move, so the replays are re-recorded with the game's bumps).
- A shipped town whose faction has a Shipyard and no water loses a custom set's built Shipyard at
  the build (the census of the build task counts them, design §32).

## Alternatives considered

- The offsets in the town part and the spot computed by the game from the static map's tiles: the
  map's terrain and objects are the adventure's facts, as a Shipyard's tile is.
- A spot only for the factions with a Shipyard: the adventure would read the town part.
- An ally's visitor seeing the buildings in its observation while it stands on the entrance: more
  than the original's town screen shows (it closes, and an ally cannot open it from the town list).

## Revision history

- 2026-11-07: Drafted (E09 T13) with the user's answers at `TWN-Q9` (decisions 46, 47); accepted as
  written at `TWN-Q10` (decision 48).
