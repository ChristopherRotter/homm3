# ADR-0028: The town context: `h3-towns`, the store of towns, its ruleset part and seams

- Status: Accepted (user, 2026-11-02, TWN-Q4: the draft accepted as written; moved here by E09
  T4a, 2026-11-03, with batch TWN-1's outcome, survey §15). Its choices were made by the user:
  the crate and the split of a town at E09's refinement (the epic's decisions 2 and 3, TWN-Q1),
  the store, the pool, the part and the moat's data at the drafts (decisions 17 to 20, TWN-Q3).
- Date: 2026-11-02
- Deciders: user; town lane session (E09 T2a)
- Amends with it: ADR-0015, ADR-0016, ADR-0017, ADR-0023 (E09 increment 1's amendments, each in
  its revision history or status line); ADR-0016 and ADR-0017 again with increment 2's (E09 T15,
  2026-11-07; `TWN-Q10`, decision 48); ADR-0013, ADR-0016, ADR-0017 and ADR-0023 with increment
  3's (E09 T24, 2026-11-12; `TWN-Q14`, decision 83).
- Working detail: the survey `docs/rules/surveys/E09-towns.md` (§0's findings, §15 batch TWN-1),
  the design note `docs/epics/E09-design.md` (T2b); increment 2's the survey
  `docs/rules/surveys/E09-construction.md` (§13 batch TWN-2) and the design note's Part D (T13).

<!-- Decisions only; the readings are the survey's, the build's detail the design note's. -->

## Context

E09 builds towns on E08's adventure map. `docs/03-domain.md` gives the town context buildings,
construction, growth, recruitment and the town's services; decision 2 keeps the town as a map
object (its tiles and entrance, owner, garrison, visiting hero) in the adventure, as a mine and
its guards, and has `h3-game` compose the visit, recruitment, the town's part of the day and the
siege's facts. ADR-0005 forbids edges between domain crates; ADR-0015 point 5 makes identities
that cross contexts contract types; ADR-0017's store of heroes is the shape of per-entity state
that forks share. Increment 1 reads from a town its faction, its buildings at the start and the
forbidden ones, its hall (the day's gold), its fort (the siege, the views), its Resource Silo, its
Tavern (another player's view) and the Capitol's limit; every later increment adds state to the
same record (the day's construction, the available creatures, the guild's spells).

## Decision

1. **Two new crates.**

   | Crate | Layer | Logic | Holds |
   |---|---|---|---|
   | `h3-towns` | domain | yes | the town context: every town's own record, its rules and derivations, its ruleset part and that part's compiler |
   | `h3-pack-towns` | adapter | no | `packs/*/towns.toml` → `h3-towns`' definitions |

   `h3-towns` is a logic crate (`no_std`, `check-arch` purity, the 100 % coverage gate). It
   depends on the kernel, the core content, `h3-contracts` and the hero model `h3-heroes` (the
   shared layer, for the services that change a hero from increment 5); no domain crate depends
   on it, `h3-game` does. Both crates and the pack files `packs/*/towns.toml` are the town
   lane's (`lanes.toml`).
2. **A town is split between two contexts, each fact held once** (decision 2):
   - **`h3-towns`** holds the town's own record: its **faction**, the buildings **built** and
     **forbidden**, its map **events** (kept from the import for the increments that apply them;
     increment 1 applies none), the map's name if it gives one (an unnamed town's name is the
     client's: the original's lists are text); later increments add the day's construction, the
     available creatures, the guild's spells. Its rules are pure functions of a town, the part
     and plain inputs: the **hall level** and the **day's income** (the gold by the highest hall,
     the Grail's, the silo's resources), the **fort level**, the **siege's facts** (point 7), the
     **reveal radius** of a capture, whether it has a **Tavern** (another player's view level),
     the **Capitol's limit** (given the owner's other towns).
     **Increment 2** (construction with the town events; decisions 37, 41–47) adds to the town's
     record **the day's mark** (whether a building was bought in it today, the original's +0x2,
     decision 43) and to its rules, each a pure function of a town, the part and plain inputs the
     game gives: **buildable now** (not ever built, every requirement in the ever-built set, not
     forbidden, not of role `grail`, a hall-4 building only while the player holds no Capitol,
     the day's mark clear; the hall's offer: a horde whose level's upgraded dwelling is ever built
     is offered as its upgraded horde), **the build** (the building, what it supersedes and what
     then builds automatically into the ever-built set; the mark set or not), **an event's
     buildings** (the event's set less what is ever built, what another of the event's buildings
     supersedes and a `shipyard` without water; in the part's order) and **the mark's life**
     (cleared at every day's start; at a capture kept, but cleared when the old owner is not human
     and the new one is). The record's map events are applied from increment 2 (resources and
     buildings; their creatures with increment 3's recruits, decision 37). The adventure holds
     the town's **boat spot** (ADR-0016 point 1); "whether the town has water" is the game's
     input to `h3-towns`, never a second copy. Every derivation of increment 1 (hall, income,
     fort, siege, reveal, Tavern, Capitol) reads the record as a build leaves it, so a build
     changes them at once.
     **Increment 3** (growth; E09 decisions 65, 69 to 78) adds to the town's record **the available
     creatures**, a count by the faction's **index** (0 to 6 the levels' creatures, 7 to 13 their
     upgrades; the original's i16 counts +0x16), each at most 32 767 (T-10 fixed: a count saturates
     where the original's wraps), and **the Portal of Summoning's** creature and count (+0x3C,
     +0x40; none until one is drawn). Its rules, each a pure function of a town, the part, the core
     and plain inputs the game gives:
     - **A dwelling's growth** at an index (TWN-GRO-001; survey §1): none without the level's
       dwelling ever built, none at a base index whose upgraded dwelling is ever built; else the
       creature's growth (the core's), plus the castle's (a standing fort level 3 adds the growth
       again, level 2 half of it, truncated); for an owned town plus the owner's share of the two
       (the Statue of Legion's half, truncated) and the Legion's amount at the index's level (the
       inputs); plus the first ever-built horde of the index (its creature's horde growth, the
       core's); plus the owner's dwellings of the index's creature (an input); then, with a building
       of role `grail` ever built, plus half of the sum, truncated.
     - **The week's growth** (TWN-GRO-003; §2): each index whose growth is above 0 gains it, plus
       the week's bonus when its creature is one of the week's, halved (truncated) in a town without
       an owner, saturating.
     - **The month** (TWN-GRO-004; §3): in a creature's month each index whose growth is above 0 and
       whose creature is the month's doubles its count (the exact creature, T-13 `fix-optional`:
       with its flag off, both indexes of the level holding it), not in a month whose first week is
       a `week_of` week; in the Plague each such index loses its growth (at least 0), then halves
       (T-12 `fix-optional`: with its flag off, a neutral town loses the half growth it got).
     - **A dwelling's first creatures** (TWN-GRO-005; §6): a dwelling built sets its index's count
       to its creature's growth; an upgraded dwelling built moves the base index's count to its own.
       The setup's, a command's and an event's builds alike.
     - **A town event's creatures** (TWN-EVT-004 amended; §6): each level's count to the upgraded
       dwelling's index when it is ever built, else to the dwelling's, else none; saturating.
     - **A neutral town's garrison week** (TWN-GRO-006; §4, TD-15 by the reading): with draws from
       the stream the game passes (the towns'), whether its garrison grows (the part's odds by a
       standing fort), the level (three draws, widening with the weeks), the creature (the level's,
       its upgrade when the garrison holds the upgrade), its room (a stack of it, an empty slot,
       else the stack of least AI value times count below the new one's, replaced; else nothing),
       the town's two counts of the level lowered by it, then a 5-in-100 upgrade of every stack of a
       base creature. It reads and returns the garrison as `h3_contracts::ArmyStack`s; the adventure
       still holds the army.
     - **The Portal of Summoning** (TWN-GRO-007; §5, TD-16 by the reading, T-11 by decision 71):
       given the owner's dwellings' creatures in object order, one dwelling drawn, then one of its
       creatures; the count set to that creature's growth (not added; no castle, Grail, week, month
       or Plague); no dwelling: none. The same draw when the Portal is built, when its owner flags a
       dwelling while it has none and at a capture that finds it empty (decision 82).
     Every derivation reads the record as the last write left it, so a build, an event and a week
     change the counts at once. Batch TWN-3 confirmed the readings (E09 T23, survey `E09-growth.md`
     §17: TWN-3-1 to -5; T-12 and T-13 stand `fix-optional`).
     **Increment 3's recruitment** (TWN-REC-001, -002; survey §5, §8) adds three rules over the same
     record: **the source** of a standing building (ever built, no built building superseding it) of
     role `dwelling` (its index), `horde` (its horde's index) or `portal` (the Portal), else a typed
     refusal, as is a base index whose upgraded dwelling is ever built (its count moved, its growth
     none); **the stock** of a source: an index below 7 its creature with the index's count, an
     upgraded index its upgrade then its base creature, **both with the index's one count** (the
     original's second creature), the Portal its creature and count (none while nothing is
     summoned); **the purchase**: a creature of the stock, an amount from 1 to the count, typed
     refusals otherwise, and its write, the source's count lowered by the amount. The army the
     creatures join, its room and the price are not the town's (point 6). A town holds **no garrison
     hero**: the hero in the garrison and its army are the adventure's (ADR-0016 point 3; batch
     TWN-3's TWN-3-6: a purchase joins the garrison hero's army).
   - **The adventure** holds the map object of kind `Town` (ADR-0016 points 1, 3 and 4, E09's amendment): its static
     part (footprint, entrance, the `TownId`), its **owner**, its **garrison** (the armies on
     objects, as a mine's guards) and its **visiting hero** (the hero on its entrance).
   - **`h3-game`** composes them (ADR-0017 points 1, 3, 4, 10 and 11, E09's amendment): the build (a random town's faction, the
     starting heroes and the pool), the visit and the capture, the siege's setup, the day's
     income, the views.
3. **`TownId`** (`h3-contracts`, ADR-0015 point 5): a town's index among the map's towns (classes
   77 and 98) in object order, dense from 0, fixed when the game is built (no rule creates or
   removes a town). The adventure's town object holds it, `h3-towns`' records are by it, and the
   commands, events and observations name a town by it.
4. **The store of towns** (decision 17): `GameState` holds `h3-towns`' `Towns` beside the
   adventure's state: every town's record by `TownId`, each shared between forks and copied on
   its first write, the list shared too (a fork costs one reference count; a write copies the
   list of references and the one town), as the store of heroes (ADR-0017 point 1). Nothing sees
   the sharing (ADR-0016 point 2's rule): `h3-towns` has its own canonical state hash with a
   version of the town lane's, which the game state hash folds in, in `TownId` order. A map
   without towns has an empty store.
5. **The town ruleset part** (decision 19): `TownRuleset`, compiled on the core as the adventure
   and battle parts (ADR-0015 point 2, its R01 amendment point 2): its own canonical hash and
   version, the core's recorded beside it; core entities (factions, creatures, resources) by
   handle, by key in its source. Per faction a **town record**: its buildings (identities, the
   hall and fort level each gives, which it supersedes), the silo's resources, the moat and the
   siege's natural obstacles (point 7); the part's parameters (the gold by hall level and the
   Grail's, the neutral garrison's odds by level, the standard set's level-2 dwelling chance, the
   hall a second Capitol becomes). Only what an increment reads; increment 2 adds requirements
   and costs. One file per pack folder, `towns.toml` (base: the eight factions; AB: Conflux;
   SoD: its overrides), overrides as ADR-0006's. `GameRuleset` bundles the part; a replay's
   header and the server's manifest record its hash. Content is harvested where the original
   has it as data; a constant of the executable is tier-2 data whose source is the reading
   (decision 10).
   **Increment 2** gives each building of a town record its **direct requirements** (buildings
   of the same record: the executable's lists, survey `E09-construction.md` §6) and its **cost**
   (resources of the core, `Building.txt`'s rows, §5); the compiler closes the requirements (the
   original's `building_requirements` 0x697798) and refuses a requirement outside the record, a
   cycle, a building requiring itself and a resource the core lacks. A new role **`shipyard`**
   names the building that needs water (decision 46). The content is harvested (decision 23's
   route: `h3-harvest`'s gap-fill, `cross-lane`): the costs from `Building.txt`, the requirements
   from the executable's lists; TD-9 the executable's (decision 41), as batch TWN-2 observed it
   (survey §13; no gap-fill override).
   **Increment 3** gives buildings three roles: **`horde`** (a level and whether upgraded, as
   `dwelling`: its growth is the core's `horde_growth` of that index's creature, the figures the
   executable's `town_hordes` 0x6887A0 copies), **`portal`** (the Portal of Summoning) and
   **`week_of`** (a level: while the building stands in any town, every week is the week of that
   level's two creatures of its faction, the Deity of Fire's week of the Imp). Each town record
   gains **`roe`** (whether its creatures may be the week's on an RoE map; false for the AB pack's
   Conflux). The parameters gain the week's creature chance (1 in 4) and bonus (5) and a neutral
   town's garrison week (the odds 40 and 80 in 101 by a standing fort, the level weights 2, 3, 4, 5,
   4, 3, the weeks' cap 8, the upgrade's 5 in 100). The creatures' growths stay the core's. The
   content: `horde` harvested from the executable's horde table, `portal` and `week_of` the
   harvest's gap-fill roles (the Dungeon's and the Inferno's specials), the parameters tier-2 data
   of the reading.
6. **Seams.** `h3-towns` and `h3-adventure` never import each other; `h3-game` reads both. The
   only new contract type of increment 1 is `TownId`. The siege's facts cross to the battle as
   ADR-0023's `Siege` (point 7). From increment 5 a town's service reads a hero's details by
   reference and returns `h3_heroes::HeroChange`s, which the game applies (ADR-0017 point 10), as
   a visit of the adventure does; recruitment returns the creatures for the game to place in an
   army the adventure holds.
   **Increment 2** adds the **town command**, `h3_towns::TownCommand` (decision 45; increment 2's
   one variant `Build { town: TownId, building: BuildingId }`), which the game wraps as
   `GameCommand::Town` (ADR-0017 point 1) and whose later variants are the town's own actions
   (increment 3's recruitment, increment 5's services). `h3-towns` validates the town's part
   (buildable now) and names the price (the part's cost), each refusal typed
   (`TownCommandError`); the game checks the turn, the owner and the treasury, pays and applies
   the build infallibly (ADR-0017 point 10). No new contract type: the command and its errors are
   `h3-towns`' types, which `h3-game` re-exports. The town events cross the same way: `h3-towns`
   says which buildings an event builds, the game pays its resources and writes the store.
   **Increment 3's growth** crosses as plain inputs and the store's writes: the game gives each rule
   whether the town is owned, the owner's share (`h3-heroes`' town growth of the owner's heroes,
   ADR-0013 point 16.4), the Legion's amounts (the garrison hero's and the visitor's), the owner's
   dwellings' creatures (the adventure's owners and kinds), the week's kind and the stream; it
   writes the counts and the Portal into the store and the returned garrison into the adventure
   (ADR-0016 point 1) in one infallible application. No new contract type.
   **Increment 3's recruitment** adds the town command's second variant, **`TownCommand::Recruit {
   town, building }`** (the original's click paths name a building: a dwelling, a horde, the
   Portal): `h3-towns` gives the source or its typed refusal; the game checks the turn, the owner
   and that no decision is open, and opens E08's **hire dialog** (ADR-0017 point 6) with the town's
   source and the garrison as its target, its creatures and counts read from the store at each view.
   **A purchase crosses three contexts**, as E08's purchase rule (ADV-HIR-002) parts it: `h3-towns`
   checks the creature and the amount against the stock, the adventure the room in the target army
   (the garrison hero's army when one stands, else the town's garrison; ADR-0016 point 3) and the
   game the treasury against the core's price; then the game pays, writes the lowered count into the
   store and calls the adventure's join, in one infallible application. The town's refusals reach
   the player as the hire dialog's (`HireError`'s offer and amount), one vocabulary for both
   sources. No new contract type: the adventure's `HireSource` and `HireTarget` name the town by
   `TownId` and the index as a plain number.
7. **The siege's facts and the editions by the packs** (decisions 14 and 20): `h3-towns` derives
   `Siege` (ADR-0023 point 2, E09's amendment) from the town and its faction's record; no game version exists.
   The faction's record holds the moat's kind (absent, harmless, damaging) and whether a siege
   there draws natural obstacles: the base pack gives them as RoE and AB play them (harmless; the
   Stronghold none, with natural obstacles), the SoD pack overrides them as SoD plays them
   (damaging; the Stronghold's moat, no natural obstacles); a Tower's kind decides its land
   mines' damage, as ADR-0023 point 2 has it.
   Conflux is the AB pack's faction, so the factions a random town or a Random choice may take
   are the ruleset's factions with a town record (eight with the base pack alone, nine with AB),
   within the map's allowed set for a player (ADR-0017 point 1).
8. **The hero pool is the game's** (decision 18; ADR-0017 point 1): `h3-towns` holds no hero
   and no pool; the Tavern (increment 4) is a town's building whose hire the game composes from
   the pool and the store.
9. **Versions owned by the town lane:** `h3-towns`' state hash and the town part's hash (v1 when
   the crate is created). Its changes bump the adventure lane's versions it touches (the game
   state hash, the import, the adventure replay format) in `cross-lane` pull requests.
   **Increment 2** bumps the town ruleset part (v4: the requirements, the costs, the role
   `shipyard`) and the town state hash (v2: the day's mark); the adventure lane's versions it
   touches, each `cross-lane`: the adventure ruleset part (the kind `Town`'s boat-spot offsets),
   the game state hash (it folds the store's hash in), the adventure replay format (the town
   command); the env's action encoding is the AI lane's (ADR-0017 point 8).
   **Increment 3's growth** bumps the town state hash (v3: the counts, the Portal) and the town part
   (v5: the roles, `roe`, the parameters); `cross-lane`: the game state hash (the week's kind, the
   towns' stream, the store), the protocol (the week's kind, the owner's town view), the adventure
   replays re-recorded; `shared/`: the kernel's stream id, `h3-content`'s two hero effects and two
   quirk flags (pack format v52, core ruleset hash v53; E09 T24). **Increment 3's recruitment and
   rows** move no version of the town's own (the counts are growth's v3; no new role or parameter);
   `cross-lane`: the game state hash (the hire dialog's town sources and target), the adventure
   state hash (a town's garrison hero, ADR-0016 point 3), the adventure replay format (`town
   recruit`, the hero moves), the env's action encoding (ADR-0017 point 8) and the protocol.

## Consequences

- Two `shared/` pull requests after acceptance (E09 T4, cut in two at its start, the user): T4a
  this ADR and the four amendments into `docs/adr/`, `docs/03`'s terms; T4b the two crate
  skeletons, `arch.toml`'s edges (`h3-game` → `h3-towns`; `h3-towns` → kernel, content,
  contracts, heroes; `h3-pack-towns` → `h3-towns`, `h3-content`, `h3-pack`, as
  `h3-pack-adventure`), `lanes.toml` (the adapter and `packs/*/towns.toml` to the town lane),
  `TownId`, the versions rows; `docs/02` §3 follows.
- `cross-lane` after it: `GameState` and its hash, `GameRuleset`, the adventure replay's header
  (the part's hash), each adventure golden replay re-recorded (skill `homm3-rerecord-replays`);
  the server's manifest (the server lane's inbox) and the env (the AI lane's) as their notes.
- A fork costs one more reference count (the XL fork benchmark, `bench-check`).
- A town is never half in one context: a capture writes the owner in the adventure and the
  Capitol's limit in the store in one infallible application (ADR-0017 point 11's rule).
- Increment 2: one `shared/` pull request after acceptance and batch TWN-2 (E09 T15, design
  §35's round 3): its amendments into this ADR, ADR-0016 and ADR-0017, `docs/03-domain.md`'s
  terms (the day's build, the hall's offer, a town event), `docs/06-security.md`'s town section.
  `cross-lane` after it: the harvest (`h3-harvest`, the battle lane's), the kind `Town`'s offsets
  and the town's boat spot (the adventure lane's static map), `GameCommand` and the replay
  format, the turn start's and the day's calls, the env (the AI lane's); each adventure golden
  replay re-recorded (skill `homm3-rerecord-replays`). A town's day's mark makes a fork's first
  build in a town copy that town (the store's rule); the day's start writes only the towns built
  in the day before.
- Increment 3: each week writes every town with a dwelling (a fork's first week copies most towns);
  the day's start computes each town's growth twice at a month's first day (the week and the month),
  measured against `day-start-builds-36`.
- Increment 3's recruitment reads the counts from the store at each view and purchase (no copy in
  the dialog); a purchase writes one town and one army.

## Alternatives considered

- The town records inside the adventure state, `h3-towns` a library of rules over types passed
  in (TWN-Q3): the adventure lane's structs would change with every town feature.
- The same store with an id local to `h3-game` (TWN-Q3): a mapping in every projection and seam,
  as ADR-0015 found for local ids.
- The pool in `h3-towns` (TWN-Q3): the random hero is drawn with no town involved, and the store
  of heroes is the game's.
- The town data as a section of `adventure.toml`, or in the core content (TWN-Q3): the adventure
  lane's adapter, or a `shared/` pull request, for every town content change.
- The moat in the battle part's siege record, or an edition derived from the packs' names
  (TWN-Q3): each change `cross-lane`; a rule keyed by names, against ADR-0006.
- A module inside `h3-adventure` split out later; `h3-towns` owning the town's armies (TWN-Q1,
  decision 2).
- Increment 2 (`TWN-Q9`): a game command `GameCommand::Build { town, building }`, the rules as
  `h3-towns` functions, each later town action a variant of its own (the game's command type
  grows with every town service, and the town context's validation has no command of its own to
  name); a pending decision opened by entering an own town, its answers building (the original
  builds from the town list without a visit); the Shipyard refused until increment 4, or
  buildable without water until then (the Castle's Lighthouse needs the Shipyard; a Shipyard
  without water is a deviation the start already removes in the original).
- Increment 2: the requirements stored closed in the pack (the closure written by the harvester:
  two sources of one fact; the executable's lists are direct); the day's mark as the day of the
  town's last build (no write at the day's start: two states that play alike would hash apart;
  the original's flag is kept).

## Revision history

- 2026-11-02: Drafted (E09 T2a); accepted as written at T2b's checkpoint (the user, TWN-Q4,
  decision 21).
- 2026-11-03 (E09 T4a `shared/`): moved here as ADR-0028 (the next free number) with the four
  amendments; batch TWN-1 (T3c, survey §15) confirmed TD-3, TD-5, TD-7 and the siege's
  aftermath, so no point changed; its unpredicted building 23 changes no point (decision 25:
  re-read in T5b). The Consequences' `shared/` work split in two (T4a, T4b).
- 2026-11-07 (E09 T15 `shared/`; the user at `TWN-Q7` to `TWN-Q10`, decisions 37, 41–48): points
  2, 5, 6 and 9 amended for increment 2, construction with the town events (with ADR-0016 points
  1, 4, 6 and ADR-0017 points 1 to 4 and 6 to 10): the day's mark and the rules buildable now,
  the build, an event's buildings and the mark's life; the part's requirements, costs and role
  `shipyard`; the town command `TownCommand::Build`; the versions. Drafted in E09 T13 (design
  note `E09-design.md` Part D, §23 to §36); batch TWN-2 (T14c, survey `E09-construction.md` §13)
  confirmed decisions 41 (TD-9, the executable's list) and 42 (TD-11, the owner builds), so no
  point changed.
- 2026-11-12 (E09 T24 `shared/`; `TWN-Q12` to `TWN-Q14`, decisions 65 to 83): points 2, 5, 6
  and 9 amended for increment 3, growth, recruitment and the army rows (with ADR-0013 point 16.4,
  ADR-0016 points 1, 3, 4, 6, ADR-0017 points 1 to 4, 6, 8 to 11 and ADR-0023 points 1 and 3):
  the available creatures and the Portal, the growth rules, the recruitment's source, stock and
  purchase; the roles `horde`, `portal`, `week_of`, `roe` and the week's parameters; the inputs
  and `TownCommand::Recruit`; the versions; the Consequences. Drafted in E09 T22a and T22b
  (design note `E09-design.md` Part E); batch TWN-3 (T23, survey `E09-growth.md` §17) confirmed
  every row, so no point changed.
