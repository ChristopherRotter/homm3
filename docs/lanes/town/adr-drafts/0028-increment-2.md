# ADR-0028 amendment (draft): E09 increment 2, construction with the town events (the town context)

- Status: **Accepted as written** at T13's checkpoint (`TWN-Q10`, 2026-11-07; E09 decision 48),
  to be applied by the round's `shared/` task T15 into ADR-0028, with batch TWN-2's outcome
  (decisions 41, 42; scored by T14c: both stand, survey `E09-construction.md` §13). Drafted by
  E09 T13 (2026-11-07).
- Amends: ADR-0028 points 2, 5, 6 and 9 (each text below goes in after the point's increment 1
  text); its consequences, alternatives and revision history. ADR-0015 and ADR-0023 are not
  amended: the crates, the import and the siege's facts stand as they are (a build changes the
  record the derivations read, ADR-0023 point 2's amendment).
- Choices: the epic's decisions 37 and 41–44 (TWN-Q7, TWN-Q8) and 45–47 (`TWN-Q9`, asked before
  this draft). Detail: the design note `docs/epics/E09-design.md` Part D (§23 to §36); the
  survey `docs/rules/surveys/E09-construction.md`; the map side `0016-increment-2.md`, the game
  side `0017-increment-2.md`.

## Context

Increment 2 is construction with the town events (decision 37). The original's human builds
through the town screen's hall, opened from the town list without a visit (survey §1.1): one
building a town a day (§3) whose requirements stand in the ever-built set, not forbidden, the
Shipyard only with water below the entrance, the Capitol only while its player holds none (§2),
paid from the treasury (§1.4, decision 42: the owner's). The map's town events build, give and
add at their owner's turn start with none of those tests but what stands, what the faction lacks
and the Shipyard's water (§8). Increment 1's record holds a town's faction, its ever-built and its
forbidden buildings (point 2), its part the buildings without requirements or costs (point 5).

## Decision

**Point 2** (a town split between two contexts), after its `h3-towns` item: **increment 2**
adds to the town's record **the day's mark** (whether a building was bought in it today, the
original's +0x2, decision 43) and to its rules, each a pure function of a town, the part and
plain inputs the game gives: **buildable now** (not ever built, every requirement in the
ever-built set, not forbidden, not of role `grail`, a hall-4 building only while the player holds
no Capitol, the day's mark clear; the hall's offer: a horde whose level's upgraded dwelling is
ever built is offered as its upgraded horde), **the build** (the building, what it supersedes
and what then builds automatically into the ever-built set; the mark set or not), **an event's
buildings** (the event's set less what is ever built, what another of the event's buildings
supersedes and a `shipyard` without water; in the part's order) and **the mark's life** (cleared
at every day's start; at a capture kept, but cleared when the old owner is not human and the new
one is). The record's map events are applied from increment 2 (resources and buildings; their
creatures with increment 3's recruits, decision 37). The adventure holds the town's **boat spot**
(ADR-0016 point 1's draft); "whether the town has water" is the game's input to `h3-towns`, never
a second copy. Every derivation of increment 1 (hall, income, fort, siege, reveal, Tavern,
Capitol) reads the record as a build leaves it, so a build changes them at once.

**Point 5** (the town ruleset part), after its increment 1 text: **increment 2** gives each
building of a town record its **direct requirements** (buildings of the same record: the
executable's lists, survey §6) and its **cost** (resources of the core, `Building.txt`'s rows,
§5); the compiler closes the requirements (the original's `building_requirements` 0x697798) and
refuses a requirement outside the record, a cycle, a building requiring itself and a resource the
core lacks. A new role **`shipyard`** names the building that needs water (decision 46). The
content is harvested (decision 23's route: `h3-harvest`'s gap-fill, `cross-lane`): the costs from
`Building.txt`, the requirements from the executable's lists; TD-9 the executable's (decision 41),
as batch TWN-2 observed it (no gap-fill override).

**Point 6** (seams), after its increment 1 text: **increment 2** adds the **town command**,
`h3_towns::TownCommand` (decision 45; increment 2's one variant `Build { town: TownId, building:
BuildingId }`), which the game wraps as `GameCommand::Town` (ADR-0017 point 1's draft) and whose
later variants are the town's own actions (increment 3's recruitment, increment 4's services).
`h3-towns` validates the town's part (buildable now) and names the price (the part's cost), each
refusal typed (`TownCommandError`); the game checks the turn, the owner and the treasury, pays
and applies the build infallibly (ADR-0017 point 10). No new contract type: the command and its
errors are `h3-towns`' types, which `h3-game` re-exports. The town events cross the same way:
`h3-towns` says which buildings an event builds, the game pays its resources and writes the
store.

**Point 9** (versions), after its text: **increment 2** bumps the town ruleset part (v4: the
requirements, the costs, the role `shipyard`) and the town state hash (v2: the day's mark); the
adventure lane's versions it touches, each `cross-lane`: the adventure ruleset part (the kind
`Town`'s boat-spot offsets), the game state hash (it folds the store's hash in), the adventure
replay format (the town command); the env's action encoding is the AI lane's (ADR-0017 point 8's
draft).

## Consequences

- One `shared/` pull request after acceptance and batch TWN-2 (design §35's round 3): this
  amendment and the two beside it into `docs/adr/`, `docs/03-domain.md`'s terms (the day's build,
  the hall's offer, a town event), `docs/06-security.md`'s town section (ADR-0017 point 2's
  draft).
- `cross-lane` after it: the harvest (`h3-harvest`, the battle lane's), the kind `Town`'s offsets
  and the town's boat spot (the adventure lane's static map), `GameCommand` and the replay format,
  the turn start's and the day's calls, the env (the AI lane's); each adventure golden replay
  re-recorded (skill `homm3-rerecord-replays`).
- A town's day's mark makes a fork's first build in a town copy that town (the store's rule);
  the day's start writes only the towns built in the day before.

## Alternatives considered

- A game command `GameCommand::Build { town, building }`, the rules as `h3-towns` functions, each
  later town action a variant of its own (TWN-Q9): the game's command type grows with every town
  service, and the town context's validation has no command of its own to name.
- A pending decision opened by entering an own town, its answers building (TWN-Q9): the original
  builds from the town list without a visit.
- The Shipyard refused until increment 4, or buildable without water until then (TWN-Q9): the
  Castle's Lighthouse needs the Shipyard; a Shipyard without water is a deviation the start
  already removes in the original.
- The requirements stored closed in the pack (the closure written by the harvester): two sources
  of one fact; the executable's lists are direct.
- The day's mark as the day of the town's last build (no write at the day's start): two states
  that play alike would hash apart; the original's flag is kept.

## Revision history

- 2026-11-07: Drafted (E09 T13) with the user's answers at `TWN-Q9` (decisions 45–47); accepted as
  written at `TWN-Q10` (decision 48).
