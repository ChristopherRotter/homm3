# ADR-0017: Game orchestration: turn groups, RNG streams per player, pending decisions

- Status: Accepted (2026-10-10)
- Date: 2026-10-09
- Deciders: user + adventure lane (refinement 2026-10-07/08, decisions 4, 8 and 9 of the epic)

<!-- Decisions only. The reasoning is in docs/epics/E08-adventure-map.md. Point 2 answers the
     server lane's SRV-Q6 (what h3-game exposes for the whole game, E16 M5). Longer than 80 lines
     because it covers decisions 4, 8 and 9 and the env at once, and increments 2 and 3. -->

## Context

`h3-game` holds what no domain context may own: players, turns, the calendar, randomness and
the wiring between contexts (ADR-0015). Simultaneous turns are an HD/HotA
feature and a non-goal of the vision, but nothing built now may make them a rewrite (ADR-0006
guardrail; the user, 2026-10-08). The server lane needs a player actor, per-viewer projections,
pending decisions as state and a trusted system actor (SRV-Q6). RL needs an `AdventureEnv`
(`docs/07` §1, §3) while `h3-sim` still belongs to the battle lane.

## Decision

1. **One entry point for the whole game** (ADR-0004): `apply(&mut GameState, &GameRuleset,
   Actor, GameCommand, &mut impl Sink)`, `legal_commands`, `observe(.., Viewer)` and
   `project(&GameEvent, Viewer)`. `GameState` holds the players, the schedule and calendar, the
   RNG streams, the pending decisions and the contexts' states (adventure now; the store of
   heroes and battles in progress from increment 3; the store of towns from E09, below). A game command wraps a context command (`Adventure(
   MoveHero { .. })`) or is the game's own (`EndTurn`). The game checks turn, phase and pending
   decisions; the context then validates its rules with the player as actor. A command touching
   several contexts validates in all of them before mutating any. Steps only the game triggers
   (start of a day, start of a turn) are infallible functions of the contexts, never commands.
   `GameState` holds each player's **treasury**: an amount per resource kind of the core ruleset
   (dense by `ResourceId`; kinds stay data, ADR-0006), never negative; a gain saturates at
   2³¹ − 1 (the original wraps to 0, `A-5`, a state-corrupting bug fixed without a flag). The
   starting treasury is the row of the setup's difficulty (`GameSetup`), every player the human
   row (E08 decision 16); the rows are data compiled into `GameRuleset`, which from increment 3
   also bundles the battle part (ADR-0015 point 2).
   **The store of heroes** (increment 3): `GameState` holds every hero's details
   (`h3_heroes::HeroDetails`) by `HeroId`, each shared between forks and copied on its first
   write, built with the state from the setup's customisations or the hero type's and class's
   defaults (a default experience is a setup placeholder from the setup stream), spell points
   full, validated by `h3-heroes`; a setup the increment cannot represent fails the import
   (`Unsupported`). From increment 4 the store keeps beside each hero's details its **level
   seed** (1–255, drawn from the setup stream when the store is built, never changed), and the
   game the scenario's **level limit** and **banned skills** (static, from the setup). A custom
   experience is no longer refused: the build gives a placed hero its **levels at load**, after
   its custom skills, army, spells and artifacts and before its spell points, from experience 0
   with the limit, each two-offer level chosen by the original's choice (ADR-0013 point 12) with
   the setup's defaults for the army. From increment 5 the levels at load also come before the
   hero's **day's points** and the **reveal around it**, which read its details as the levels
   left them (the original's order, `placed_hero_setup` 0x4D8B30, `placed_heroes_setup`
   0x4CAE10): the adventure places the heroes, the game gives the levels, then the adventure's
   start of the game sets the points and reveals, reading the store. A hero that leaves the map
   keeps its details (from E09 its type goes back to the hero pool, below; the Tavern is E09's
   increment 3). The contexts **read a hero's details
   by reference**: the game passes its store to the adventure's commands and queries (later the
   towns'), which read through `h3-heroes`' derivations (ADR-0013 point 2); what a command
   changes in a hero comes back as typed **hero changes** in its outcome (point 10), which the
   game applies through `h3-heroes`. No adventure profile seam (E08 decision 25).
   **Management commands** (increment 6): `GameCommand`'s new variant names its player's heroes
   and places: a stack's move, split or dismissal; the formation; the tactics flag; an
   artifact's move; an assembly; a disassembly. The game checks the actor, the turn and the open
   decision, then the armies' rules in the adventure (ADR-0016 point 3) and the artifacts' in
   `h3-heroes` through the store (ADR-0013 point 14), both before either applies; Scholar's
   spells are hero changes the game applies when it opens a meeting. **The expansions are the
   game's configuration** (E08 decision 104): the packs its ruleset is compiled from (base, AB,
   SoD). A rule that differs by expansion reads the ruleset's data (the combinations and the fifth
   misc position come with the SoD pack), never the map's version (the original's `mapKind`,
   which follows the map); the moat and Conflux are the packs' data from E09 (below), so nothing
   in the game reads a version.
   **The setup** (increment 7): `GameSetup` holds each player's **human flag** (E08 T54: a map's
   import sets it from the map's player kinds, a colour a human may play is human; a session may
   change it before the game is built; an env's agent counts as whatever the setup says), and
   beside the banned skills the scenario's **banned spells and artifacts** (static, from the map),
   which the game passes to the adventure's build.
   **Increment 8** adds no command and no decision: boarding, sailing and landing are steps of
   `MoveHero`'s path, and the sea's pickups give at once. The setup gains the map's **boats**
   (ADR-0016 point 1), which the import fills; a hero placed on water is refused (`Unsupported`,
   D-44). The turn start's points follow the hero's surface (ADR-0016 point 3).
   **Increment 9** adds one command, **`BuildBoat`** (an adventure command naming a Shipyard by
   its type and anchor, as the observation names it): a boat built from an own or allied
   Shipyard without a visit (ADV-SHY-003; the original's click on the Shipyard, VCMI's and the
   wiki's). No hero is named; the player's turn must be running with no open decision; the
   Shipyard must lie on a tile the player has explored, its boat tile must exist and be free,
   and the treasury must pay the kind's price, which the game checks before the adventure
   applies. Flagging, a mine's visit, a Lighthouse's and the weekly objects' are steps of
   `MoveHero`; the day's start is `EndTurn`'s (the last group's), no command of its own. The
   setup gains the ownable objects' **owners** (ADR-0016 point 1, from the map) and the map's
   **timed events** (the game's: for each its players, whether it applies to humans and to
   computers, its first day, its interval, its seven resources and its message by index); the
   import refuses an owner colour nobody plays.
   **E09's increment 1** (towns on the map; E09 decisions 6, 8 to 12, 14, 17, 18) adds no command
   and no decision kind (point 10). **The state:** `GameState` holds the **store of towns**
   (`h3-towns`' `Towns`, ADR-0028 point 4) beside the adventure's state, and the **hero pool**
   beside the store of heroes: per hero type of the core ruleset whether it is free, taken (by a
   colour, or as a prisoner) or out of the game (the map does not allow it), and the colours
   that may take it (SoD's disposed heroes' masks); one block shared by forks and copied on a
   write. **The pool's life:** built when the game is built from the setup's allowed heroes and
   masks; every placed hero's and prisoner's type taken, then each player's **pick reserved**,
   before any draw (T-2 and T-3 fixed: the original marks neither in time, E09 decision 12); a
   drawn or generated hero's type taken; a hero that leaves the map in play goes back free (the
   original's 0x4D9EC0; nothing draws from the pool in play before E09 increment 4's Tavern,
   which also takes `ADV-STUB-9`'s retreat facts).
   **The setup:** `GameSetup` gains the towns' setups (`h3-towns`', joined to the adventure's
   town objects by `TownId`) and, per player, its **allowed factions**, its **faction choice** (a
   faction or Random), its **starting-hero pick** (a hero type or none), its **main town** with
   whether it generates a hero, and its **random-hero flag**; the game also gains the **allowed
   heroes** and the **colour masks** (ADR-0015 point 4). The game validates (one check per
   invariant, ADR-0015 point 4): a choice is a faction of the player's allowed set that has a
   town record in the ruleset; a pick is in **the original's offer** (E09 decision 9: for a
   player whose choice is a faction, not Random, and whose main town generates a hero or who has
   the random-hero flag, the first 16 in type order of the free heroes of the faction's two
   classes it may take), and no two players pick one hero. A random hero's owner is a player of
   the game.
   **The expansions:** the moat and Conflux are data of the packs (ADR-0028 point 7); nothing in
   the game reads a version. The factions a Random choice or a neutral random town takes are
   the ruleset's with a town record, a player's within its allowed set (E09 decision 14: with AB
   or SoD enabled, an RoE-format map's player whose eight bits are all allowed may have Conflux;
   one restricted to fewer stays restricted).
   **E09's increment 2** (construction with the town events; E09 decisions 37, 45) adds the
   game's first **town command**: `GameCommand::Town(TownCommand::Build { town, building })`.
   Legal for a player in its turn with no open decision (as every map command) that **owns the
   town** (TD-11 the owner, E09 decision 42; the original's build dialog refuses any other player,
   batch TWN-2, survey `E09-construction.md` §13), for a building `h3-towns` finds buildable now
   in it (ADR-0028 point 2: the hall's tests, the day's mark, one Capitol a player over the
   player's towns) whose cost the treasury pays; no hero needs to be in the town. Each refusal is
   typed and changes nothing. **The setup gains nothing**: the requirements and costs are the
   part's, the forbidden sets and the town events the map's (already in the towns' setups), the
   boat spot the adventure's (ADR-0016 point 1).
   **E09's increment 3** (growth, recruitment and the army rows; E09 decisions 65 to 83) adds
   **`TownCommand::Recruit { town, building }`** (ADR-0028 point 6: opens the hire dialog, point
   6) and the adventure's **`AdventureCommand::TownHeroes { town, change }`** (`Enter`, `Leave`,
   `Swap`: the garrison hero's moves, ADR-0016 point 3), each its town's owner's, in its turn,
   with no decision open (GAM-DEC-001), a typed refusal changing nothing. **The state:**
   `GameState` holds **the week's kind** beside the month's (E09 decision 77): normal, a
   creature's week (one creature, the parameters' bonus) or the week of a `week_of` building
   (its level's two creatures of its faction, the base creature's growth as the bonus), and
   **the towns' stream** (point 4). **The setup** gains **whether the map is RoE's** (the
   import's format), which only the week's draw reads.
   **Increment 10** (dwellings and hiring; E08 decision 231) adds no command: a dwelling's or a
   Refugee Camp's visit is a step of `MoveHero` (both kinds are entered), and the free creatures
   and the purchases are answers (point 6). The setup gains nothing beyond ADR-0016's (the
   dwellings' owners and the random dwellings' faction sets and level ranges come with the map's
   objects).
   **Increment 11** (the teleporters; E08 decision 260) adds one command, **`Revisit { hero }`**
   (an adventure command; E08 decision 268, the original's space key): the entered object under
   the player's hero visited again, as the step onto it visits it, without points spent or
   required, without a step and without a guard test (batch E11's E11-19: no battle at a revisit
   beside a guard). The player's turn must be running with no open decision; the hero must be the
   player's and stand on the visitable tile of an entered object whose kind has a built visit (one
   typed refusal for every other case). A jump is a step of `MoveHero` (its last step's visit) and
   needs no command; no command names an exit (D-53, confirmed by E11-15). The setup gains
   nothing: the channels are the static map's, built with the game from its objects (ADR-0016
   point 1).
   **Increment 12** (garrisons and the bonus visitables; E08 decision 295) adds **no command**: a
   garrison is visited by `MoveHero`'s last step or `Revisit` and crossed by a path of its team
   (batch E12's E12-1: no dialog on the way); its exchange is the management commands'
   (`Manage::Army`), naming the hero's army and **`ArmyRef::MapGarrison(object)`**, a sibling of a
   town's `ArmyRef::Garrison(town)`. The adventure's command application takes **the day of the
   week** (1 to 7, the calendar's) beside the acting player's stream: every caller passes it.
2. **Actors and viewers.** `Actor::Player(PlayerId)` for every player command, also a bot's;
   `Actor::System` for the trusted layer that hosts the game only (the server's session layer,
   the env), never for a player, with its own closed set of commands (end a player's turn,
   answer a pending decision with its default, remove a player) that no player can issue. A
   battle side without a player (wandering monsters, an object's guards; increment 3) is played
   by `Actor::System`, which gains battle commands for an **unowned side** only: no player can
   command it, and the system actor commands no owned side. Its policy stays outside the state,
   as an agent's choices (point 4): the server's or the env's bot. The faithful policy is the
   original's battle AI, which the battle lane ports (E08 decision 27); it may read the whole
   battle state, as the original's does, since it runs in the trusted host; until then the env
   plays the side at random. Its other commands stay the server lane's (M5), which also picks
   its default for a battle whose player is gone. `Viewer::Player(PlayerId)` and `Viewer::Spectator`, which sees
   public facts only. Increment 1 builds the player variants; the system actor and the spectator
   come with the server's M5 as additive variants. Contexts never see these types: a context's
   `apply` takes the `PlayerId`, its `observe` and `project` a viewer of its own (a player, or
   public facts only), and `h3-game` maps onto them. From E09 the observation adds to each town
   a viewer sees the public facts and the garrison by the viewer's level (ADR-0016 point 6),
   which the game derives: the viewer's towns with a Tavern from the store and the owners, the
   owner's team, a hero within reach with expert Visions or a Rogue in its army. From E09's
   increment 2 the observation gives the **owner** of a town its buildings (ever built,
   forbidden, standing) and its day's mark, and every viewer that sees the town whether it was
   built in today (E09 decision 47, ADR-0016 point 6); a player whose hero enters an ally's town
   is told its buildings by the step (E10's teams). **E09's increment 3:** the week's kind is
   public (`WeekStarted`, the observation's calendar, as the month's); a town's available
   creatures and its Portal are its owner's (the town view's owner part, TWN-VEW-004's level;
   `docs/06-security.md`'s town section). A town's **garrison hero** is seen in its town's view
   at the level that shows the garrison's army (TWN-VEW-001, level 3) and by its owner among its
   own heroes (ADR-0016 point 6); the hire dialog's view is its owner's, as on the map.
   From increment 12 the observation adds to each **map garrison** a viewer sees (by object: type
   and anchor) its army at the viewer's level, which the game derives as a town's garrison's (the
   owner's team, a hero with Visions at level 3 within reach of the entrance, else the viewer's
   towns with a Tavern, at most 2), and its removable flag to the owner's team; and the types
   whose kind the viewer's team has marked (ADR-0016 point 6).
3. **Turn groups.** A day is a schedule: an ordered list of turn groups, each a set of players
   who act together. A group's turn starts with each context's start-of-turn step for its
   players and ends when each of them has ended its turn or left the game. After the last group
   the day advances: the calendar, then each context's start-of-day step, in a fixed order of
   contexts and players. Building the state ends with day 1's start-of-day step and the first
   group's start-of-turn step, so the first day starts as every later one. Which rule runs at
   which hook is a rule question (survey `E08-adventure-core.md` §3: in increment 1 every rule
   runs at the start of the day; ADV-Q4). From increment 5 the adventure's start-of-turn step reads
   the store of heroes (the day's points with each hero's bonuses, ADV-MOV-002), and its
   start-of-day step has its first rule: in `strict-fixed` (`ADV-QUIRK-18` off) every hero on the
   map reveals around it to its owner with its radius, after the calendar and before the first
   group's turn; `faithful` reveals nothing there until a rule hides tiles (the Cover of Darkness,
   E09). Sequential play is one player per group in player
   order, the only schedule of increment 1. Commands interleave in one authoritative order, the
   server's log, which is the replay; logic stays single-threaded per game (ADR-0003).
   From increment 9 the **day's start** has the original's order (`new_day` 0x4C7FE0;
   GAM-CAL-002), on every day advance, so from day 2 (the game's start runs none of it: day 1
   gives no income, grows and resets nothing): the calendar; on the first day of a week (days 8,
   15, …: weeks of seven days) **the adventure's week step** (ADR-0016 point 4: the weekly
   objects, then the monsters' growth); on the first day of a month (days 29, 57, …: months of
   four weeks) **the month's kind** drawn, then **the adventure's month step** (a creature
   month's monsters); then **the day's income** of every player (GAM-RES-003 to -005; every
   player not defeated, E10's defeat) added to its treasury; then **every hero's spell points**
   (HER-MAN-002: every hero of the store); then the adventure's start-of-day step (increment 5's
   reveal under `strict-fixed`). Then the first group's turn starts. At **each player's turn
   start**, after the contexts' start-of-turn steps, the map's **timed events** fire for that
   player (GAM-CAL-004): in the map's order, each whose players include it, which applies to its
   kind (human or computer, the setup's flag) and whose day it is (its first day, the file's day
   + 1, or every interval after it); its resources are added to the treasury, a loss floored at
   0. Day 1's fire when the game is built, with the first group's turn. The week and the month
   are derived from the day (no state of their own; the original's three words are one number
   here).
   The **month's kind** is game state: normal (the first month's), a creature's month (one of the
   parameters' twelve creatures) or the Plague; drawn at each new month as the parameters' odds
   (5, 4 and 1 in 10); its and the week's effects on the towns are E09 increment 3's (below). The
   month's and the week's names (the original draws one for its message) are not drawn: a name
   is the client's, and no draw of ours is the original's.
   **From E09 increment 3** (GAM-CAL-002 amended; E09 decisions 69 to 77): on the first day of a
   week, before `WeekStarted`, **the week's kind** is drawn (TWN-GRO-002: on a month's first day
   no creature's week; else 1 in 4 a creature's, uniform over the town records' creatures, on an
   RoE map those of the records with `roe`; then a standing `week_of` building in any town, the
   first in `TownId` order, makes it that level's week whatever was drawn) and `WeekStarted`
   carries it; then **every town's week** in `TownId` order (TWN-GRO-003); then the adventure's
   week step as before; then **each neutral town's garrison week** (TWN-GRO-006) and **each
   owned town's Portal** (TWN-GRO-007), each in `TownId` order (the original's: after the map's
   weekly objects). On the first day of a month the month's kind is drawn as before but, in a
   month whose first week is a `week_of` week, it becomes that level's base creature's month
   without the creature's draw (GAM-CAL-003 amended; `new_month` 0x4C8F63: its doubling skipped,
   its monsters spawned); then **every town's month** in `TownId` order (TWN-GRO-004: a
   creature's month doubles the exact creature's counts, the Plague's neutral subtraction, with
   their flags, `TWN-QUIRK-10` and `-9`), before the adventure's month step. The game's first
   week is normal. The towns' day (the marks, the views told) follows, as increment 2's, and
   tells each owner the towns whose counts or Portal the week or the month moved. Batch TWN-3
   confirmed the readings (E09 T23: TWN-3-1 to -5).
   From E09 each town pays its owner at the day's start, from day 2, with the income
   (GAM-RES-004's path): the day's income `h3-towns` derives (the gold by its highest hall and
   the Grail's, the Resource Silo's resources; E09 decision 10); a neutral town pays nobody.
   From E09's increment 2 **the day's start** clears every town's day's mark (the original's
   `new_day` towns' loop; GAM-CAL-002 amended; one a town a day for every player, E09 decision
   43: the computer's every other day on Easy and Normal comes with the computer player). **At
   each player's turn start**, right after the map's timed events, **the town events** fire (E09
   decision 44; GAM-CAL-004's test): over the towns in `TownId` order (the map's objects' order)
   and each town's events in the map's order, every event whose players include the player,
   whose kind (human or computer) applies and whose day it is (its first day or every interval
   after it: an interval of 1 from its first day, TD-12 the original's), **whose town the player
   owns**. Each fired event's resources are added as a timed event's (the gains, then the losses
   floored at 0), then its buildings are built (`h3-towns`' event buildings, in the part's order,
   with the boat spot's water, without the day's mark, no requirement, forbidden, Capitol, Grail
   or cost test; T-8's second Capitol stands), then its creatures are kept for increment 3's
   recruits (`TWN-STUB-10`: lost until then). Day 1's fire with the first group's turn inside the
   build, as the timed events'.
   From increment 12 the adventure's **day step** drops every hero on the map's day marks, its
   **week step** every hero's week marks (before the turn's points), refills every Magic Spring's
   entrance and draws each Fountain of Fortune's value (in the weekly objects' pass below, point
   4). Batch E12 read each so: day 2 the Well's bit clear (E12-14), day 8 the Stables' bit clear
   and the day's maximum without its 400 (E12-10), both Spring entrances full (E12-9), the
   Fountains' values new and unchanged within a week (E12-12).
   **From E09 increment 3** (ADV-MOV-002 amended; TWN-ROW-005) at its owner's turn start each
   **garrison hero** gets its day's maximum and points too, after the player's heroes on the map,
   in `TownId` order (the original's `next_player_turn` 0x4C7486: the towns' +0xC heroes; batch
   TWN-3's TWN-3-6); the day's spell points reach every hero as now; the adventure's day and week
   starts drop a garrison hero's day and week marks as a hero on the map's (E08's increment 12;
   the original's every hero record, `new_day` 0x4C80CF, 0x4C8D2A). A garrison hero's **income**
   (`h3_heroes::daily_income`: its Estates and its worn resource artifacts) and its army's
   **weekly resources** (GAM-RES-005: the Crystal Dragons in an owned town's upper bar, a
   garrison hero's army or else the town's garrison) count with its owner's (GAM-RES-003
   amended), once: the original takes a garrison hero out of its owner's list
   (`town_visitor_to_garrison`), so nothing counts twice.
   From increment 10 **the adventure's week step** first refills **every dwelling**, owned or not
   (each count its creature's growth, set; the guards the kind's full guards; no week's or month's
   kind), then draws the weekly objects' contents and **the camps' creatures** in one pass in
   object order, then grows the monsters (GAM-CAL-002 amended; ADR-0016 point 4; batch E10's E10-8
   and E10-9).
4. **An RNG stream per player** (kernel streams, ADR-0003 point 7). Randomness caused by a
   player's command draws only from that player's stream; the day advance draws from a game
   stream, and the random placeholders of a setup are resolved from a setup stream when the
   state is built (in increment 1 one: the starting army of a placed hero without stacks of its
   own, E08 T6a; increment 2 adds random resources, random pile amounts and campfires, ADR-0016
   point 4; no command of increment 2 draws, so the streams' ownership isolation is not yet
   exercised by commands). Agents' and bots' own choices stay outside the game
   state (the kernel's `AGENT` stream, in the env). The kernel's stream ids gain the per-player
   form (a shared change in T2). **A battle's randomness** (increment 3): its obstacles come
   from the battle lane's generator (`h3_battle::generate_setup_obstacles`, until it ports the
   original's) with a kernel `Rng` seeded by the profile (`ADV-QUIRK-5`, fix-optional, E08
   decision 26): in `faithful` the original's tile seed `110 291 x + 167 801 y + 81 569` (32-bit;
   the defender's tile, the level not included), so a tile's obstacles are predictable as in
   the original; in `strict-fixed` a second draw from the attacking player's stream. The battle's
   **own seed** (every later draw) is drawn from the attacking player's stream when it starts
   and lives in the battle's state, server-only as every stream (point 5). A monster's split
   (ADR-0016 point 4) takes its two draws from the tile in `faithful` (`ADV-QUIRK-6`) and from
   the attacking player's stream in `strict-fixed`. Increment 3's setup placeholders: a random
   monster's creature and count, every monster's aggression, a hero's default experience.
   **A level's draws** (increment 4) come from the original's source, the hero's level seed and
   the new level, in `faithful` (`ADV-QUIRK-9`, fix-optional, E08 decision 55); in
   `strict-fixed` from the owner's stream in play and from the setup stream at load. A level is
   drawn when it is given or its decision opens, never earlier.
   Increment 7's random contents are setup placeholders drawn from the setup stream when the
   state is built (ADR-0016 point 4); no visit of increment 7 draws. The Pyramid's monster battle
   splits its guards as a monster's (`ADV-QUIRK-6`: the tile under `faithful`, the attacking
   player's stream under `strict-fixed`).
   Increment 9's **setup placeholders**, drawn from the setup stream when the state is built, in
   one walk after increment 7's and 8's in object order: an abandoned mine's resource then its
   guards' count, a Windmill's resource then amount, a Garden's gift (the first week's contents
   with the other placeholders: a game's build draws nothing from the game's stream). **The
   day's start draws from the game's stream** (the day advance's): at a new week the Windmills'
   resource and amount and the Gardens' gift, in object order; at a new month its kind, a
   creature month's creature, then the spawn's walk, each candidate tile one draw and each
   monster placed its count and its aggression right after its tile's draw. E09's week of a
   creature draws from a stream of its own (E08 decision 197), so adding it shifts none of these.
   **E09's setup placeholders** (increment 1), drawn when the game is built **before every
   existing one** (a drawn hero's army and experience read its type; a random dwelling of E08's
   increment 10 reads its town's faction), in this order:
   1. each player's Random choice, in colour order: uniform over its allowed factions;
   2. in object order, each random town's faction (survey `E09-towns.md` §2.3, TD-1 by the
      reading: an alignment to a colour that plays gives that colour's faction, else its owner's,
      else uniform over the ruleset's factions) and each random hero's type (its owner's pick for
      the owner's first random hero, else the draw below);
   3. each main town's hero, per player in colour order when its main town generates one: a
      human player's pick, else the draw; it stands on the town's entrance as its visitor;
   4. the towns in `TownId` order: a standard set's level-2 dwelling (the part's chance, 30 in
      100, survey §5.3, TD-3 by the reading), a neutral town's garrison without one of the map's
      (four rolls by the part's odds, TD-4, TD-5 by the reading), a garrison's random stacks by
      the town's faction (TD-3 and TD-5 confirmed by batch TWN-1, survey §15);
   then the existing placeholders in their order (the starting armies, the drawn heroes' with
   them, the default experiences, the objects' contents, the level seeds last).
   **The draw** (`hero_draw_of_faction`): uniform over the free types of the faction's two
   classes the player may take, else over every free type it may take, else a typed failure of
   the build (T-4, E09 decision 12: no shipped map reaches it); no tavern weights.
   **E09's increment 2 draws nothing**: the build, the day's mark, the events and the Shipyard's
   water are draw-free. At the build, after the adventure's static map is built, a town whose
   faction has a building of role `shipyard` and whose object has no boat spot gets that building
   forbidden and, if the setup built it, unbuilt (TWN-SET-011; E09 decision 46; `TWN-STUB-2`'s
   Shipyard half lifted; its Grail half stays with increment 6 and E10's victory conditions): a
   write of the store before the first turn, after every draw, so no stream position moves.
   **E09's increment 3** draws from **the towns' stream** (`StreamId::TOWNS`, a named kernel
   stream, E09 decision 75): at a new week its kind (one draw of four unless it is a month's
   first week, then the creature's), then each neutral town's garrison week (its odds; when it
   grows, three for the level, then for a base creature that found room one for the upgrade),
   then each owned Portal's (a dwelling, then one of its creatures), in that order. The month's
   kind stays the game's stream's; after a `week_of` week its creature is not drawn. **The
   Portal's other draws** (E09 decision 71): at its build by a command from the builder's
   stream, after the build is checked (a refused build draws nothing); at a town event's build
   from the towns' stream; at the setup from the setup stream after every existing placeholder
   (a town that starts with a Portal and an owner); when its owner flags a dwelling while it has
   none, from the visitor's stream after the visit's own draws (each such Portal in `TownId`
   order); **at a capture** that finds a standing Portal empty, from the capturer's stream after
   the capture's own draws (E09 decision 82: a human's capture opens the town screen, which
   draws at once). No other command of growth draws: the counts, the first creatures and the
   events' creatures are draw-free. The towns' stream is server-only (point 5) and hashed with
   the state.
   **Increment 10's setup placeholders**: in the static map's placement walk, in object order,
   each **random dwelling's faction** (uniform among its set) then **level** (uniform in its
   range), after E09's placeholders and **before every other existing one** (the drawn type's
   footprint fixes the tiles the later draws read: a Shipyard's boat tile, the free tiles); a map
   without random dwellings draws nothing more, so nothing shifts. **A Refugee Camp's creature**
   (uniform among the random monsters of its levels, ADV-MON-003's set; its count the creature's
   growth, no draw) in the first week's walk with the weekly objects, in object order. **At a new
   week** each camp's creature from the game's stream in the weekly objects' pass, in object
   order. No visit, question, battle's end or purchase of increment 10 draws.
   **Increment 11's jump** is the first draw a command causes inside the adventure: a monolith's
   or a Whirlpool's exit is one uniform draw among its candidates from **the acting player's
   stream** (the visitor's, ADR-0016 point 4), none for a single candidate (the original's), none
   for a gate. `AdventureState::apply` takes that stream from the game for every adventure command
   and draws only after the whole command is checked, so a rejected command leaves the stream
   untouched. In a command the jump's draw comes before the draws of the battle it opens (the
   battle's seed, then under `strict-fixed` its obstacles). The answers and the rests draw nothing
   (a held jump's exit was drawn at its visit). Both profiles draw alike: the original's exit comes
   from the C runtime's one `rand()` stream, which no profile reproduces (batch E11's E11-15 and
   E11-17: uniform over the candidates, the sequence the game's). No setup placeholder is added
   and none moves (the channels and the gates' pairs are built without a draw).
   **Increment 12's draws**: a Fountain of Fortune's value is drawn when the game is built from
   the setup stream in the first week's walk, and at each week's start from the game's stream in
   the weekly objects' pass, each in object order with the Windmills, the Gardens and the camps
   (one uniform draw among the kind's values each). No visit of increment 12 draws. Batch E12's
   E12-12 saw the original's values new only at a week's start; its later weeks alike in every
   run, so the original's draw does not follow the new game's seed, which no profile reproduces
   (as E11-15's `rand()`): the engine draws from its own streams.
5. **Seeds are server-only** (E08 decision 9). The master seed and the stream positions are
   authoritative state: in the state hash (which never reaches a client either), never in an
   observation or event, never seeded from client data; the observation-hygiene and
   ownership-isolation tests cover them. A replay holds the seed and is released only after
   the game. The heroes' level seeds are server-only too; a level-up's offers are its owner's.
6. **No blocking prompts: pending decisions.** What needs a player's answer (a level-up, an
   object's question, monsters offering to join, a battle) is a pending decision in the state,
   owned by the players it involves (a battle against another player's hero: both). Its owners
   may answer outside their turn; while a player has one, its only legal commands are the
   decision's answers (in a battle, the battle's commands) and it cannot end its turn; it
   blocks no one else, but for a level-up (below). The engine never waits and never calls back. Increment 2 has two kinds,
   both offers answered yes or no: the Cartographer's (a price, E08 decision 15) and the border
   guard's passage (no price; ADV-Q10, D-7). A player's open decisions form a queue (at most one
   in increment 2); `GameCommand::Answer` answers the first (`Accept` or `Decline`, additive
   variants for later kinds; typed by decision from increment 7, below), and a decline only closes it. While one is open, its owner's legal
   commands are its answers only: no move, no `EndTurn`. The system actor's default answer is
   `Decline`. Increment 3's offers (yes or no): **join for free**, **join for gold**, **pursue
   a fleeing monster**, **fight an object's guards**. Answering an offer may open the next
   decision of the same visit in the same command (a declined join goes on to the pursue offer
   or a battle; an accepted pursuit or fight opens the battle): a player still has at most one
   open decision. A **battle** is a pending decision owned by the players of its sides (the
   visitor's; against another player's hero the defender's too, who plays outside its turn).
   While it is open its owners' only legal commands are `GameCommand::Battle(h3_battle::Command)`
   by the owner of the side to act (the system actor for an unowned side); neither owner can
   end its turn; it blocks no one else. The command that ends the battle applies its outcome
   (point 11) and closes the decision. **A level-up** (increment 4) with two offers is a pending
   decision of the hero's owner, answered **`First`** or **`Second`** (the original's left and
   right offer). A hero's pending levels are given in order, one at a time: a level with one offer
   learns it and one with none gives the primary skill only, at once, with an event to the
   owner; a level with two opens the decision, and its answer gives the level and draws the next.
   Unlike the other kinds, an open level-up **holds the game**, as the original's modal dialog
   does: until its owner answers, every other actor's command is rejected and lists no legal
   command (a battle's defender that won outside its turn answers before the player whose turn
   runs goes on), and the game names the player it waits for. The hosting layer handles the
   wait: in **hot seat** the client hands the screen to the owner, hiding everything until the
   owner confirms it is in control, then shows the dialog alone, and hands back the same way;
   **online** the server pauses the waiting players' turn timers and gives the owner a limited
   time, then answers with the system actor's default. Other players learn only that the game
   waits for the owner, never the level-up itself.
   The system actor's default answer to a level-up is the original's choice (ADR-0013 point 12),
   as the original's when the dialog's time runs out; until E10 gives the engine the computer
   player's state that choice reads with the army counted, it chooses with the setup's inputs, as
   at load (ADV-Q17).
   Increment 6 adds two kinds, each its player's alone, holding no one else and blocking its
   owner's `EndTurn` like an offer:
   - **A meeting** of two of its heroes, opened by the step toward the visited one after Scholar
     (ADR-0013 point 14): while it is open its owner's legal commands are the management commands
     on either hero (between them or within one) and `Decline`, which closes it. The next meeting
     needs another step.
   - **An army dialog** of a hero and a joining group, opened by an accepted join that finds no
     room (after the price and the treasure, the monster gone): the management commands between
     the hero and the group's small army (ADR-0016 point 3) and on the hero alone, and `Decline`,
     which closes it; the group's rest is lost (`ADV-STUB-2` closed).
   Outside a decision the management commands are legal in the owner's turn on any of its heroes
   on the map, but for the moves between two heroes, which need the meeting. They are listed
   **per hero**, as moves are (the reachable-tiles query): `legal_commands` keeps `EndTurn` and
   the answers, and a query lists one hero's management commands (with a meeting or an army
   dialog open, those toward the other hero or the group too) but the splits, whose counts a
   second query gives for a source and a target.
   **An answer is typed by its decision** (increment 7, E08 decision 138; it replaces the
   untyped `Accept`, `Decline`, `First` and `Second` above): `GameCommand::Answer` answers the
   actor's first open decision with an `Answer` whose variant names the kind of decision and
   carries that kind's own type; an answer of another kind is rejected (`WrongAnswer`) and
   changes nothing. Every decision's answers, increments 2 to 6's migrated with increment 7's
   (one replay format bump), the system actor's default in brackets:
   - **`Offer(Consent)`** (`Accept`, `Decline`) [`Decline`]: every yes-or-no offer, each the same
     question with an optional price (point 10): the Cartographer's, the border guard's, a
     monster's join, join for gold and pursuit, a guarded object's fight (a guarded artifact
     always, a guarded scroll with a text), and increment 7's: open a Pandora's Box, search a
     Pyramid (a looted one too) or a Warrior's Tomb, pay a Tree of Knowledge (its drawn price,
     not opened when the owner cannot pay).
   - **`LevelUp(SkillOffer)`** (`First`, `Second`: the original's left and right offer) [the
     original's choice, ADR-0013 point 12].
   - **`Chest(ChestChoice)`** (`Gold`, `Experience`) [`Gold`, the original's plain OK]; the view
     shows both amounts.
   - **`Arena(ArenaChoice)`** (`Attack`, `Defense`; no leaving) [`Attack`, the original's Cancel].
   - **`School(SchoolChoice)`** (the first or the second of the School's two skills, or `Leave`
     without paying) [`Leave`]; opened only when the owner can pay its price, which the view
     shows.
   - **`University(UniversityAnswer)`** (`Buy(slot)`, `Leave`) [`Leave`]: the University is a
     decision like a meeting, open until `Leave`; each `Buy` buys an open slot's skill for the
     type's price (checked against the slot's state and the treasury) and keeps it open. Its view
     shows the four skills and each slot's state.
   - **`Close`** [`Close`]: ends a meeting or an army dialog (increment 6's `Decline` for them).
     The army dialog also opens over **up to seven leftover stacks** of a box's or an event's
     creatures, with the same management commands; the visit's rest (the experience, then its
     level-ups) comes after it closes.
   - **The visit's rest after a won guards' battle** (a pickup, the rewards, the Pyramid's spell)
     is held by the game and given once the winner's level-ups are answered (at once when there
     are none), as the original gives the levels first (E08 decision 141).

   Each of increment 7's questions is its owner's alone, holds no one else and blocks its owner's
   `EndTurn` like an offer.

   Increment 9's decisions reuse increment 2's to 6's kinds: **a guarded mine's question** is a
   guarded pile's (`Fight`, accepted: the battle; declined: nothing); **the Leave Guards dialog**
   is the army dialog (GAM-DEC-004) over the mine's guards as its group, which its owner's
   `Close` keeps on the mine (a join's group is lost at its close, as before); **the Shipyard's
   boat offer** at a visit is an offer with the kind's price, made only when the treasury can pay
   (else refused, as every price), accepted: the price paid and the boat built. Nothing at the
   day's start asks.

   Increment 10's decisions:
   - **A guarded dwelling's question** (another team's or a neutral dwelling whose guards stand):
     an offer naming the guards' first creature and the size word of their total
     (`Offer(Consent)`, [`Decline`]); accepted, the guards' battle (point 11); declined, nothing.
   - **The hire question** (a dwelling of the visitor's team, so its own once flagged; a camp with
     creatures, for any visitor): an offer naming the creatures (a camp's hidden until then).
     Accepted, on one's own dwelling the creatures of the part's free level or less join free, all
     of each or none (no room: none, the count kept), then the hire dialog when a creature is
     above that level.
   - **The hire dialog** (GAM-DEC-008; a new kind, its owner's alone, holding no one else,
     blocking its owner's `EndTurn` like an offer), answered **`Hire(HireAnswer)`**: `Buy {
     creature, amount }` and `Close` [`Close`]. A purchase is refused (typed, nothing changed)
     unless the creature is the source's, the amount from 1 to its count, the army has room for it
     (a stack of the type within the stack's most, else a free slot) and the treasury pays its
     cost times the amount; then the price is paid, the count lowered and the creatures join. The
     dialog stays open until `Close`; a dialog of **one** creature closes after its purchase, as
     the original's (batch E10's E10-4, E10-8, E10-9: after a purchase of part of the count too).
     Its legal answers are each creature's purchase of **the most** it may buy (the count, the
     treasury's bound and the room's), then `Close`; a query gives the range of amounts from 1 to
     the most, as a split's counts. Its view (its owner's) shows the source by type and anchor and
     each creature's count and cost. An ally's dwelling (E10's teams) opens it **read-only**,
     `Close` alone (E10-6). E09's recruitment reuses the kind, with a town's dwelling as its
     source and the hero's or the garrison's army as its target.
     **E09 increment 3** opens the hire dialog from `TownCommand::Recruit` with **a town's
     source** (an index's dwelling or horde, or the Portal of Summoning) and **the town's upper
     bar as its target** (the garrison hero's army when one stands, else the town's garrison:
     the original's `town_garrison_army`; never the visitor's, survey `E09-growth.md` §8; batch
     TWN-3's TWN-3-6), opened whatever the count (0 too, the original's dialog); a Portal with
     nothing summoned is refused instead. The dialog's rules are the map's: the legal answers
     each creature's most, `Close`; it stays open while it offers two creatures (an upgraded
     index: the upgrade and its base from one count) and closes after the purchase of a
     one-creature source. Its view and `HireOpened` name the town and the index or the Portal in
     place of an object's type and anchor.
     **The town's two rows open no decision** (TWN-ROW-001; E09 decision 80): in its turn with no
     decision open, the owner's army commands (`Manage::Army`) may name an own town's upper bar
     alone or with that town's visitor's army, as the original's town screen, which opens at
     will and needs no answer; the game admits the pair as it admits a meeting's two heroes
     (GAM-MAN-001 amended), by E08's one owner rule. E08's `Exchange` (GAM-DEC-009) stays the map
     garrison's, opened by a visit. Neither the hero moves (point 1) nor the rows' commands draw.

   **Increment 11 adds no decision kind.** A battle at a teleporter's exit is a hero battle
   (point 11), a hero of the team on a gate's pair is increment 6's **meeting** (its owner's
   alone; the visitor stays on the entrance and the two heroes are not neighbours; batch E11's
   E11-6), and `Revisit`'s visit opens what the object's visit opens. **The jump after a won
   battle at an exit** is a fifth **visit's rest** (GAM-DEC-007): held by the game when the
   battle opens and given once the winner's levels are answered, as the original moves the hero
   after the levels (E11-7); nothing when the mover left the map. A visit that jumps, or holds a
   jump, holds no guard's visit of the entrance: **a guard is met first** (batch E11's E11-19, E08
   decision 279, for every entered object): a step onto a guarded tile opens the guard's visit,
   and the tile's own visit (a jump among them) is a visit's rest after it, given to a hero still
   on the tile, as an event's on a guarded tile.

   **E09's increment 2 adds no decision kind**: a build is a command (point 1), the town events
   need no answer (their messages are events).

   **Increment 12 adds one decision kind, the exchange** (GAM-DEC-009; the original's
   `garrison_exchange_dialog` 0x5D1130): the visiting hero and the garrison, its owner's alone,
   holding no one else, closed by its owner's `Close` and blocking its `EndTurn` until then. While
   it stands its owner's management commands may name the hero's army and the garrison's (a third
   pair of armies held together, beside a meeting's two heroes and an army dialog's hero and
   group); each command changes the armies at once, by the adventure's one owner rule (ADR-0016
   point 3: a non-removable garrison takes stacks only into its empty slots; batch E12's E12-3,
   D-61 the original), so the close writes nothing back. It opens at a visit of a team's garrison
   and at the capture of one, and **after a won garrison battle as a sixth visit's rest**
   (GAM-DEC-007), given once the winner's levels are answered, as the original's order; nothing
   when the winner left the map (E12-1, E12-2, E12-6: the exchange at a move's end, after a flag
   and after a won battle). Its view (the owner's): the garrison by type and anchor, its army by
   slot, its removable flag. The town's rows reuse its armies held together, not the kind (E09
   decisions 67 and 80, the town's two rows above), with `ArmyRef::Garrison(town)` or the
   garrison hero's army as the stationary bar. The bonus visitables ask nothing.
7. **Simultaneous play is designed for, not built.** Built now: turn groups, streams per player,
   pending decisions. **Not decided** (deferred to the increment that builds it): the target
   that commands of the players of one group commute (any interleaving, the same state hash; a
   property test), which needs a contact rule that ends simultaneous play for players who could
   interact (HD's and HotA's surveyed in T1b) and shared state (the tavern pool, global
   counters) partitioned or accepted as order-dependent; under it, a command that conflicts with
   the authoritative order is rejected and leaves the state untouched. Until then, each piece of
   state that commands of two players can both write is listed in the epic file when it is
   added: the inventory for that decision. Increment 2 adds an object two players can remove
   (a pile, a border guard) and the per-player object state (purchases, keys), which only its
   owner's commands write. E09's increment 2: a town's record is written by its **owner's**
   build, by the town events at its owner's turn start and by a capture (increment 1); its day's
   mark by a build, the day's start and a capture. Two players' commands can write one town only
   through a capture.
8. **`AdventureEnv`** (`crates/h3-sim/src/adventure/`; the lanes below) drives `h3-game`,
   not `h3-adventure`, so it sees turns, days and projections as a player does: reset (a map, a
   seed), step, mask, fork, state hash. Its action is a destination among the reachable tiles of
   a hero (the env builds the path from the core's reachable-tiles query, an ADR-0006
   guardrail) or `EndTurn`, provisional until E13's macro-actions. No C ABI for it in E08 (E05,
   with the batch API). Action encoding v2: the entry head gains the answers after `EndTurn` and
   the eight hero slots (`Accept`, `Decline`; the tile head is ignored for them); while a
   decision is open the mask allows its answers only. The observation (`h3_game`'s view) shows
   its owner the treasury and the open decision (kind, price). Increment 3's action encoding v3
   has two phases: while the player to move is in a battle its action is the battle lane's
   encoding (`h3_sim::action`) and every adventure entry is masked; otherwise v2's entries.
   `to_move` yields seats: an acting player without a battle and, for a battle, the owner of its
   side to act or the **neutral seat** for an unowned side, which the caller plays with a policy
   of its choice. The observation adds the open battle as its owner's side sees it
   (`h3_battle::observe_as`); the adventure replay format v3 records each command's actor.
   Increment 4's action encoding v4 appends two entries, the first and the second offer, legal
   only while a level-up is the player's open decision (`Accept` and `Decline` only while an offer
   is); while a level-up holds the game (point 6) `to_move` yields its owner alone, in its turn
   or not; the observation shows its owner the open level-up (the hero, the new level, the
   primary skill gained, the two offers).
   **The adventure's RL encoding belongs to the AI lane** (increment 6; E08 decision 99): the
   env's action and observation encodings and masks, the rollout and their tests; the adventure
   lane keeps the game's commands, `legal_commands` and `legal_moves`, the decisions' views, the
   replay model of game commands and the CLI's replays and benchmarks. In `lanes.toml` (E08
   decision 103): `crates/h3-sim/src/adventure/` the AI lane's but its replay model
   (`replay.rs`), which stays the adventure lane's with `crates/h3-cli/src/adventure/` and
   `fixtures/replays/adventure/` (longest match). Increment 6 adds no entry to the action encoding
   (v4); its masks allow `Decline` while a meeting or an army dialog is open, so an agent that
   meets a hero or fills its army closes the decision and plays on. Increment 7's decisions reach
   agents through `legal_commands`; the env's action encoding maps its entries onto the typed
   answers (its `Accept`, `Decline`, first and second offer entries onto `Offer`, `LevelUp`,
   `Close` and the new kinds' two-way choices), and its masks must allow at least one answer of
   each decision, so a rollout never stalls (the system actor's defaults, point 6). Whether a
   University's purchase gets a head of the encoding is the AI lane's call; the mapping's change
   in `h3-sim` comes with the typed answers, `cross-lane` with the user's consent (E08 decision
   138). Increment 8 adds no entry to the action encoding: a boarding or a landing is a
   destination among the reachable tiles, so the env's masks offer them as any other. The
   observation the env returns gains the boats on the viewer's explored tiles (tile, type,
   facing) and each seen hero's boat: `h3_game::GameObservation` as it is, since the adventure
   env has no flat encoding, so nothing of the AI lane's moves (E08 decision 187, replacing
   decision 172's `cross-lane` change). Increment 9 adds `BuildBoat` to the action encoding (one
   entry per Shipyard the viewer may use, by its anchor), masked by its legality; the
   observation the env returns gains the owners, the weekly marks and the month's kind, which no
   flat encoding carries yet (decision 187). The action encoding's change is the AI lane's
   (point 8's ownership), made `cross-lane` in the game's task with the user's consent, as
   increment 7's typed answers were (E08 decision 138).
   Increment 10's **action encoding v7** appends four entries, each buying the most of the open
   hire dialog's creature in that place (the original's four), masked by its legality; the
   dialog's `Close` is the closing entry, the two questions the offer's entries. The observation
   the env returns gains the dialog's view (`GameObservation` as it is, E08 decision 187). The
   change is the AI lane's (point 8's ownership), made `cross-lane` in the game's task with the
   user's consent, as increments 7's and 9's.
   Increment 11's **action encoding v8** appends eight entries, the `Revisit` of the hero in each
   of the eight hero slots (the move entries' slots), masked by its legality; a jump needs no
   entry (a destination among the reach's tiles). The observation the env returns gains the
   teleporters' types in the objects' views (`GameObservation` as it is, E08 decision 187). The
   change is the AI lane's (point 8's ownership), made `cross-lane` in the game's task with the
   user's consent, as increments 7's, 9's and 10's. A revisit at no points can repeat within a
   turn: the rollout's step cap is the AI lane's.
   E09's increment 2: the env plays the build (E09 decision 7): the action encoding gains one
   entry, **build**, with the tile head naming the town's entrance and a **building head** over
   the record's buildings (`h3_towns::MAX_BUILDINGS`), masked by the legal builds; the AI lane's
   version and layout, `cross-lane` in its paths.
   **E09's increment 3** appends two entries' groups (`cross-lane`, the AI lane's paths, E09
   decision 7): **recruitment** (one entry: the town by its entrance in the tile head, the
   building in the building head, one action per `Recruit` the legal commands list; encoding
   v10), whose purchases are the hire dialog's four entries unchanged (a town's source offers at
   most two creatures), and **the hero moves** (`Enter`, `Leave`, `Swap`, each the town by its
   entrance in the tile head; encoding v11, or with v10 when they land together). The env plays
   no army management (no entry before or after), so the rows' stack moves are a client's.
9. **Versions owned by the adventure lane** (T2 adds the rows): the game state hash (`h3-game`),
   the adventure state and ruleset-part hashes (`h3-adventure`), the import version (`h3-map`),
   the adventure replay format (`h3-sim`'s `replay.rs`), and until increment 6 the action
   encoding (the AI lane's from then on, point 8).
   Increment 2 bumps the game state hash to v2 (treasuries, decisions), the action encoding to
   v2 and, if the answer command needs it, the adventure replay format (E08 T14). Increment 3:
   the game state hash v5, the adventure state hash v7, the adventure ruleset part v6, the map
   import v4, the action encoding v3, the adventure replay format v3. Increment 4: the game state
   hash v9, the map import v8, the action encoding v4, the adventure replay format v5. Increment
   5: the core ruleset hash and the pack format (shared: six effect kinds, three flags), the
   adventure ruleset part v10 (the fly and water-walk prices, Nomads' terrain; Cursed Ground's
   parameter if batch E5 confirms it); the game and adventure state hashes, the map import, the
   action encoding (v4) and the adventure replay format (v5) do not change: a flying hero's
   destinations are tiles of the same mask. Increment 6: the game state hash (the decisions), the
   adventure state hash (the flag), the details' hash (positions), the core ruleset hash and the
   pack format (shared: two effect kinds), the map import (positions), the adventure replay
   format v6 (the management commands); the action encoding's version is the AI lane's.
   Increment 7: the core ruleset hash and the pack format (shared: the mana ceiling, the quirk
   flags), the adventure ruleset part (the kinds), the map import (the classes, their bodies, the
   bans), the adventure state hash (the visits' block, the army left), the game state hash (the
   decisions, the details' bonuses), the adventure replay format v7 (the typed answers); the
   action encoding's version is the AI lane's.
   E09: the town lane's rows (ADR-0028 point 9). E09's increment 1 bumps the game state hash
   (the store of towns, the pool), the adventure state hash and ruleset part (the town type and
   kind), the map import and the adventure replay format (the town part's hash in its header),
   each `cross-lane`; the env's encodings are the AI lane's. **E09's increment 2** bumps the game
   state hash (the store's v2 folded in, the town command), the adventure ruleset part (the kind
   `Town`'s offsets) and the adventure replay format (the town command), each `cross-lane`; the
   town lane's part v4 and state hash v2 (ADR-0028 point 9); the action encoding the AI lane's.
   **E09's increment 3's growth** bumps the game state hash (the week's kind, the towns' stream,
   the store's v3; `cross-lane`), the protocol (`WeekStarted`'s kind, the observation's week, the
   owner's town view; the server lane's, `cross-lane`) and re-records the adventure replays; no
   command of growth, so neither the replay format nor the env's action encoding moves. **Its
   recruitment and rows** bump the game state hash (the hire dialog's town sources and target),
   the adventure state hash (a town's garrison hero; ADR-0016 point 1), the adventure replay
   format (`town recruit`, the hero moves, management lines naming a town's bar), the env's
   action encoding (point 8) and the protocol (the commands, `HireOpened`'s and the view's
   source, the garrison hero's events and views), each `cross-lane` and in merge order with
   E08's increment 12 (the numbers are the next at merge; the second to merge bumps again).
   **Increment 12** bumps the game state hash (v28: the exchange, its rest, the battle bonuses
   leaving the details), the adventure state hash (v22), the adventure ruleset part (v20), the
   map import (v22) and the adventure replay format (v13: the map garrison's reference in a
   management line); the env's action encoding stays v8 (the exchange is answered by the close
   role). The numbers are the next at merge, after E09's increment 3 where it merges first.
10. **Gains and costs cross by outcome.** A context's `apply` returns an outcome: the resources
    its command gains for the actor and the offers it opens, each with its price. The context
    never sees a treasury. The game adds the gains after the context applied (gains cannot fail,
    so atomicity holds). An offer whose price the payer cannot pay is not opened (an event says
    so to the payer); otherwise it becomes a pending decision. Accepting an offer: the context
    checks it still stands, the game checks the price against the treasury, then pays and calls
    the context's infallible acceptance; a rejected answer changes nothing. Towns (E09) use the
    same outcome and the same check. Increment 3: an outcome may also name hero changes
    (`h3_heroes::HeroChange`, checked by the context against the details it read), a hero's
    army change (a group joining) and a battle to start (the visitor, the defender: a monster,
    an object's guards or another player's hero, the tile). A join for gold the payer cannot
    pay is not opened and counts as declined, so the visit's chain goes on, without the refusal
    event (the original asks only when the player can pay). Increment 7: a visit's outcome may
    also name the reward's hero changes (ADR-0013 point 15), **resources lost** (a reward
    record's negative amounts: the treasury goes to 0 at the least, never refusing), a choice
    with its amounts, a group of up to seven stacks without room, the visit's rest after a battle
    or an army dialog, an event's firing (to its visitor's owner) and the move's stop. A price (a
    School's, a Tree's, a University's) is the treasury's and the object type's, never the
    command's. Increment 8's outcomes reuse increment 7's: a sea pickup's resources and artifact
    (with the full backpack's turn or loss), a removal; a boarding or a landing reports the
    hero's boat and its points after the conversion (to its owner); a bottle's message (to its
    visitor's owner). Increment 9's outcomes: a weekly object's gain (as a pickup's resources); a
    flag (the object and its new owner); a boat offer accepted or `BuildBoat` (the boat and the
    price, which the game pays: for `BuildBoat` checked before the adventure applies, so a
    rejection leaves everything untouched). **The day's income** is no outcome but the game's own
    step: the game asks the adventure for each player's production (its mines' daily amounts; on
    a week's first day, for each creature type with a **`weekly_resource` ability** in one of the
    player's heroes' armies, its amount once, however many: the Crystal Dragons' 3 crystal) and
    `h3-heroes` for each of its heroes' income on the map (ADR-0013 point 16), adds them and pays
    them, saturating (GAM-RES-001), one `ResourceGained` per resource and player in player order.
    **A timed event's** resources are added the same way, its losses taken down to 0 at most
    (`ResourceLost`, the amount actually taken).
    **E09's increment 1** adds **no command and no decision kind**: entering, capture and the
    siege are steps of `MoveHero` (ADR-0016 point 4). The step's outcome names the town entered,
    captured, or the siege to start (the town and its defender); the game applies a **capture**
    in both contexts at once and infallibly: the adventure's (the owner, the garrison emptied,
    the reveal by the town's radius, which `h3-towns` gives) and the store's (the **Capitol's
    limit**: a captured town with a Capitol whose new owner holds another becomes a City Hall,
    survey `E09-towns.md` §8.4; the start keeps every Capitol the map gives). A town's day's
    income is the day's income step's, as a mine's (point 3).
    **Increment 10's outcomes**: a visit's free creatures (the adventure's own change of the
    hero's army, told by its events); **the hire dialog to open** (the hero and the source,
    read-only or not); a won guards' battle's rest (the flag, then the hire question, after the
    winner's levels, GAM-DEC-007). **A purchase:** the adventure checks the source, the amount and
    the room and names the price (the creature's cost times the amount, by resource); the game
    checks the treasury before and pays after, then calls the adventure's infallible purchase; a
    rejection leaves everything untouched. The flag is increment 9's.
    **Increment 11's outcomes:** a Whirlpool's loss is the adventure's own change of the hero's
    army (told by its event to the owner alone), no loss of resources; **the jump to hold** (the
    hero and the exit) beside the battle at an occupied exit, given by the adventure's infallible
    arrival when the rest runs; a gate's meeting is the outcome's meeting. No price, no gain.
    **E09's increment 2: a build**: `h3-towns` checks the town and the building and names the
    price (the building's cost); the game checks the owner and the treasury before anything
    changes, then pays the price (`ResourceLost`'s payment, as `BuildBoat`'s) and calls the
    store's infallible build with the day's mark. Its events: the building built, to the town's
    owner alone; the town's public facts (fort level, Capitol, built today) to every viewer that
    sees the town, and its top level's facts (hall, income) to the viewers at that level (point
    2), when they change (the original redraws the picture at a first fort or a Capitol). **A
    capture** (increment 1's application) also keeps the day's mark, clearing it when the old
    owner is not human and the new one is (survey `E09-construction.md` §3, E09 decision 43). **A
    town event's** resources are a timed event's (gains saturating, losses floored at 0, each told
    to the player); its buildings are told as a build's, its firing by the town and the event's
    place in the town's list (the message is the client's).
    **E09's increment 3:** a neutral town's garrison week returns the garrison as stacks with
    the town's lowered counts; the game writes both, the army into the adventure (ADR-0016 point
    1) and the counts into the store, in one infallible application. **Its recruitment**
    (TWN-REC-001): a purchase's application is infallible and in this order: the price paid, the
    source's count lowered in the store, the creatures joined into the target army by the
    adventure (its `StackChanged` to the owner, the town's viewers told the upper bar's change at
    their level). **The hero moves** are the adventure's own changes (ADR-0016 point 3), told as
    heroes leaving and entering the map are, and the town's change to its viewers.
    **Increment 12's outcomes:** the exchange is an outcome the game opens (or holds after a
    battle); a movement gain, the Sirens' losses and the Stables' upgrade, the battle bonuses and
    the timed marks are the adventure's own changes of the hero on the map and its army (E08
    decision 307: the bonuses no longer a hero change, ADR-0013 point 2); the spell points and the
    Sirens' experience are hero changes the game applies through `h3-heroes`, the experience's
    levels given as every gain's (batch E12's E12-7: 310 experience for the lost hit points). No
    price, no gain of resources.
11. **Battles through the game** (increment 3; ADR-0013 point 5). The game builds the
    `BattleSetup`: the armies from the adventure (a monster's stacks by its split), each hero's
    `BattleHero` and war machines from `h3-heroes`, the battlefield from the tile in the
    original's order (ADR-0022), the obstacles (point 4), the formations, the funds. It starts
    the battle with the seed of point 4, keeps it in the decision (shared between forks, copied
    on the first write), routes its commands and maps its events to the owners of the sides
    they are visible to (an unowned side's to no player). When it ends the game applies the
    outcome at once and infallibly, in every context (the heroes' part as hero changes):
    casualties (a monster's or guards' survivors stay with the object), experience for the
    winner's hero, Necromancy, Eagle Eye's spells, spell
    points, destroyed war machines' artifacts, the loser's artifacts to the winner (the first
    fitting slot, else the backpack), a surrender's gold from the loser's owner to the
    winner's, a hero that lost, retreated or surrendered leaves the map, a beaten monster leaves
    it and its treasure's resources go to the winner's owner, a won guarded pile is picked up; a
    monster that survives keeps every stack's survivors (A-6 fixed). Then (increment 4) the
    winner's owner gets its hero's level-ups (point 6), as the original gives a human player's
    levels after the battle's screen.
    **The tactics flag in battle** (increment 6): when the game builds a battle's setup it gives
    each hero's `BattleHero` its Tactics as `h3-heroes` derives it, unless the side whose hero has
    the greater Tactics level belongs to a hero on the map whose flag is off (ADR-0016 point 3):
    then **both** heroes' Tactics go as 0, so no side gets the tactics phase and the two levels'
    cancelling (BAT-TAC-001) stays as the experiments confirmed it (the user's condition at
    ADV-Q24). No seam changes.
    **The setup's facts from the game** (E08 T54, decision 124; GAM-BAT-010 to -013): the game's
    difficulty; each side's `human` flag from its player's setup flag (false for an unowned
    side), which the battle's bad-morale let-off reads (BAT-QUIRK-27); the sides' colours; each
    hero's sex (a placed hero's from the map); the background (the scenery of the tiles around
    the battle's, a tile's topmost object the last in map order); the attacker's ground (the
    special ground of the attacking hero's own tile). The retreat facts stay at their defaults
    (`ADV-STUB-9`: a Tavern to hire the hero again, E09; the victory condition and the computer
    player's reading, E10). **Increment 7's battles:** a box's, an event's, a guarded artifact's
    and a guarded scroll's guards fight as a guarded pile's (unsplit; their survivors stay with
    the object); the Pyramid's as a monster battle with a second creature (ADR-0016 point 4),
    whose losses are not kept (the next fight is against the full guards). When a battle ends
    every hero in it loses its battle bonuses (ADR-0013 point 15.1), as the original's
    `battle_end` (from increment 12 cleared by the adventure on the hero on the map, below).
    **Increment 8's battles** (GAM-BAT-001 amended): before the underground, a battle in which
    **either** hero is in a boat is fought on water's battle terrain with the ship's **deck**;
    when **both** are, on the **two boats** (their fixed blocked hexes, the battle part's); a
    special ground on the battle's tile then replaces the battlefield as on land (no shipped map
    has one on water). Nothing else in a battle reads a boat (the original's three tests). A
    monster or an event's guards on water fight a boat's hero on the deck; a landing's guards
    fight at the monster's tile, the hero out of its boat (batch E8's E8-15); no hero but a
    boat's reaches a monster on the water (ADR-0016 point 4; D-45 reverted). A hero in a boat
    that loses, retreats or surrenders leaves the map **with its boat** (GAM-BAT-003 amended,
    ADV-SEA-006); a winner in a boat stays in it.
    **Increment 9's battles** (GAM-BAT-001 amended): a mine's guards defend on the mine's
    entrance. A neutral mine's single stack is a wandering monster's defender (split as a
    monster's, `ADV-QUIRK-6`'s draws; D-47, confirmed by batch E9's E9-1); any other guards
    defend as an army without a hero: a neutral mine's played by the neutral seat, an owned
    mine's by its **owner** (a side with an owner is its owner's, as a hero's), who answers
    outside its turn as a hero battle's defender (the original hands its human owner the guards'
    side, each stack waiting for its order: batch E9's E9-2). The battle terrain is the mine
    type's when it has one (the subterranean of the crystal, gold and abandoned mines), else the
    tile's. The guards' survivors stay on the mine; a win flags it for the attacker.
    **E09's sieges** (increment 1; E09 decision 16): the game builds a siege's `BattleSetup` as
    any battle's, with `siege: Some(..)` from `h3-towns`' derivation (ADR-0023 point 2) and the
    defending hero's town bonuses (ADR-0023 point 3) from the same derivation. The defender is
    the visitor with the garrison merged into its army first (ADR-0016 point 4, TD-7, confirmed
    by batch TWN-1's row 3), else the garrison as `Defender::Garrison`, played by the town's
    owner (who answers outside its turn, as a mine's owner) or the neutral seat. An attacker's
    win captures the town (point 10) with the attacker outside; the survivors stay in the armies
    that fought; a beaten hero leaves the map and goes back to the pool. A siege of an ally's
    visitor is refused by the step (a typed refusal, a stub until E10).
    **From E09 increment 3** (GAM-BAT-001, TWN-SIE-001 amended): a town with a garrison hero and
    a visitor is defended by **the visitor in a battle that is not a siege** (`siege: None`, the
    gate tile's battlefield as any hero's battle, no town bonuses), after which the town stays
    its owner's whatever the outcome; a garrison hero alone defends **the siege** with its own
    army and its town bonuses (ADR-0023 point 3), and a beaten one goes back to the pool as a
    beaten hero; an empty garrison hero yields the town without a battle (T-5, `TWN-QUIRK-11`).
    **Increment 10's battles** (GAM-BAT-001 amended): a dwelling's guards defend on its entrance
    as an army without a hero, unsplit and **unowned**, played by the neutral seat whoever owns
    the dwelling (D-51 as the original, E08 decision 238; an owned mine's guards stay their
    owner's), on the entrance tile's battlefield (no terrain of the type; batch E10's E10-1: the
    defending side unowned, the tile's grass, no special field). Their survivors stay the
    dwelling's guards until the week's step (none after a draw); a loss or a retreat changes
    nothing else; a win flags the dwelling for the attacker after its levels, then the hire
    question.
    **Increment 11's battles** (GAM-BAT-004 amended): an enemy hero on a teleporter's exit (a
    monolith's, a gate's, a Whirlpool's drawn tile: A-37 fixed so, E08 decision 265; batch E11's
    E11-16 showed the original's two heroes on one tile, state-corrupting) defends as any hero
    against the mover, who attacks from the entrance: the battle at the defender's tile, the exit,
    with its terrain, special ground and background, the attacker's ground the entrance's (E11-7:
    a gate's battle underground); at a Whirlpool both heroes are in boats, so on the two boats.
    The tiles within 1 of the exit are revealed to the mover's owner first (ADR-0016 point 5).
    Won: the defender leaves the map, the winner's levels, then the held jump (point 6). Lost,
    retreated or surrendered: the mover leaves the map (in a boat with it), the jump dropped; a
    draw, both leave.
    **Increment 12's battles** (GAM-BAT-001 amended): a garrison's army defends on its entrance
    unsplit, as an owned mine's guards (its owner's side without a hero, played outside its turn)
    or a guarded object's (unowned, the neutral seat's); its end writes the survivors back, a win
    flags it and holds the exchange (batch E12's E12-6: no question, one stack, unowned). **A
    battle whose tile is an anti-magic garrison's entrance** (the garrison's own, or a hero's
    standing in it attacked) sets the setup's anti-magic flag (BAT-TER-002; D-60 the original:
    E12-4, the flag set for the defender's tile, clear for an attack out of it). A hero standing
    in a garrison is fought as any hero, the garrison's army not in its side (E12-4). The battle's
    setup derives each hero's side with **its bonuses on the map** as `derive_battle_hero`'s
    input (ADR-0013 point 15.1); the battle's end has the adventure clear both heroes' battle
    bonuses but the once-marks the part names as outliving a battle under the profile
    (GAM-BAT-014 amended; the Sirens' with `ADV-QUIRK-46` off: A-44, decision 308; E12-8).

## Consequences

- The server builds M5 on points 1, 2 and 6 without engine changes beyond the additive variants;
  its turn timers pause while the game waits for another player's level-up, whose time limit
  and default answer are the server's (point 6).
- One player's choices never move another player's dice, in sequential play too: for RL a
  cleaner signal, for simultaneous play the precondition of commuting.
- Every game command needs a turn check and a pending-decision check; `legal_commands` and
  `apply` share them (property-tested). The env and the CLI go through the game like a server.
- Conservation (increment 2): an army changes by no accepted command; a treasury only by a
  visit's gain or an accepted offer's price. Observation hygiene covers treasuries and decisions.
  From increment 3 an army changes only by a battle's outcome or a joining group, a treasury
  also by a won battle's treasure or pile and a surrender; legality, atomicity, ownership
  isolation (a battle's draws from its attacker's stream only), observation hygiene (a battle's
  private side) and clone equivalence are property-tested with battles open.
- Battles against monsters played at random by the env are easier than the original's: training
  results against them are provisional until the original's battle AI is ported.
- The simultaneous-play inventory (point 7) gains a monster two players can fight and a battle
  writing two players' heroes, armies and treasuries. Battles reach the wire through `h3-game`
  only when the server hosts whole games (ADR-0018).
- Increment 4: faithful level-ups are predictable from the level seed, which stays server-only,
  so an agent learns no more than a reloading player; `strict-fixed` hides them behind the
  owner's stream. The observation's open decision becomes a choice of kinds (an offer or a
  level-up). Level-ups write only their owner's hero: nothing for point 7's inventory. The
  golden replays, fixtures and property plans answer the levels a battle gives.
- Increment 5: the golden replays' commands stay valid as data; where a fixture's hero has a
  movement or scouting skill or artifact, its play and its state hashes change and the replay is
  re-recorded (its header's core ruleset hash moves in any case). The first rule at a day's start
  exists in one profile only: both profiles' properties run the day advance.
- Increment 6: clients and servers send the management commands; the server lane maps them when
  it serves adventure games. An open meeting or army dialog is a state a fork shares; its views
  are the owner's. The AI lane takes the env with an encoding that ignores increment 6's
  commands until it extends it; the replays record them from v6 on. A battle's observation
  shows the zeroed Tactics of both heroes in a battle the flag cancelled.
- Increment 7: `Answer` becomes an enum by decision kind: every caller that builds one changes
  (the env's action mapping, the CLI's replay files, the properties' plans, the tests); the
  replay format bumps once (v7); `h3-protocol` maps no adventure command yet (the server lane's,
  when it does). A wrong answer is a type error at the caller where it can be, `WrongAnswer`
  where the decision is only known at run time. The game holds a visit's rest across a battle
  and its level-ups, and across an army dialog: one more kind of pending state, its owner's, in
  the game state hash. Simultaneous play's inventory (point 7) grows by what two players'
  commands can both write: a find emptied, a chest, artifact, scroll, Scholar, box or event
  removed, an event's guards (the objects of increment 2's kind); a player's knowledge and a
  hero's marks are written by their owner's commands only.
- Increment 8: no command, decision or answer is new, so the adventure replay format keeps its
  version; the game state hash follows the adventure's. The battle's setup reads the heroes'
  boats (no seam change: `BattleSetup` already names its terrain and battlefield). The env's
  observation encoding changes (the AI lane's, `cross-lane`). Simultaneous play's inventory
  (point 7) grows by the empty boats (two players' heroes may board one) and the sea's pickups.
- Increment 9: the core creature gains **abilities** outside battle (the user at ADV-Q38, E08
  decision 201: the Crystal Dragons' crystal is an innate creature ability), a closed core kind
  beside its growth and flags, whose first is `weekly_resource { resource, amount }`; the battle
  part's abilities (ADR-0012) stay the battle's, and ADR-0015 point 2's core row gains "a
  creature's abilities outside battle". One command (`BuildBoat`), so the adventure replay format
  moves; the game state hash gains the month's kind and the timed events; the env's action
  encoding changes (the AI lane's, `cross-lane`). The day's start becomes `EndTurn`'s costliest
  step on a new week and a new month (the scan over the weekly objects and monsters; the spawn's
  walk over every tile).
- E09's increment 1: `GameState::new` grows the pool and the town draws; their order is part of
  the game state hash and the replays (each adventure golden replay re-recorded with the bump).
  E10's scenario setup keeps the rest of the setup (teams, conditions, the computer's choices);
  the faction choice and the pick are its seam with E09 (E09 decision 6).
- E09's increment 2: `GameCommand`, `GameCommandError` and `GameEvent` gain the town's variants
  (the adventure lane's `command.rs`, `cross-lane`; the server's wire maps no game command or
  event yet, so no `h3-protocol` change); `legal_commands` lists the builds after the boats, in
  `TownId` and building order. The day's start and the turn start each gain one call into the
  game's town module (the adventure lane's `state/day.rs`, `cross-lane`). Increment 3 gives the
  town events' creatures their recruits (`TWN-STUB-10` lifted) and a dwelling's first creatures
  at its build; increment 5 the heroes' visit after a build (the services, the guild's spells)
  and the Tower's reveals; increment 6 the Grail's building.
- Increment 10: one new decision kind and its answer, so the game state hash (v23) and the
  adventure replay format bump (E09's T5a2 took the replay format's v9 first, so increment 10's is
  v10; E09 bumps the game state hash too, the second to merge bumps again). The dialog's purchase
  rule is one, the map's (ADR-0016 point 4), checked by the game's treasury: E09's recruitment
  adds a source and a target, not a second dialog. The random dwellings' draws come first among
  the static map's: a map that holds one draws every later placeholder differently than before; no
  recorded game holds one. An agent of v7 buys the most of a creature, never less; a finer amount
  is the AI lane's.
- Increment 11: `AdventureState::apply` takes the acting player's stream (every caller passes
  it; a rejected command leaves it untouched), the game a fifth rest (game state hash v26), one
  command (adventure replay format v11) and eight env entries (action encoding v8, the AI
  lane's); the space key's revisit repeats at no points, so the env's rollout needs a step cap.
  The guard first (decision 279) moves every entered kind's visit on a guarded tile to a rest
  after the guard's battle, so the golden replays with such a visit change with it.
- Increment 12: one decision kind and its rest (game state hash v28), the exchange's direct army
  reference (adventure replay format v13), the day of the week in the adventure's command
  application (every caller), the Fountains' draws in the week's pass, the battle bonuses read
  from the hero on the map for a battle's setup and cleared by the adventure; the env's encoding
  stays v8. Each new event, the decision's view, the reference and the fields moved from the
  details to the hero on the map break `h3-protocol`'s build until mapped (`cross-lane`).

## Alternatives considered

- One global stream: a command of one player shifts every later roll of the others;
  simultaneous commands could never commute.
- Modal prompts as callbacks or blocking calls: a prompt would hold the whole game, and a
  state waiting inside a call cannot be forked.
- Lending the treasury to the context's `apply`, or the game checking a command's cost before
  passing it on: the adventure would own a game concept, or the game would know each object's
  prices (point 10).
- Opening every offer and refusing an unpayable acceptance: the original refuses at the visit
  (survey `E08-objects.md` §4); a decision that can only be declined is noise for an agent.
- An env on `h3-adventure` directly: no turns, no day, and a second path beside the game's.
- Obstacles and the split from the tile in every profile: exploitable, if mildly (a defender
  picks its tile, an attacker tunes its army before it commits; the user, ADV-Q13); the battle's
  own seed from the tile too: every roll predictable, against decision 9.
- The unowned side played inside the engine (a bot in the core; E05 and E06 own bots), or by
  the attacking player (it would throw the monsters' side); a dedicated `Actor::Neutral`: a
  second trusted actor for one job.
- One flat action space with the battle's entries appended: couples two encodings and their
  versions; the phase keeps the battle lane's encoding as it is.
- A separate adventure sim crate: a second env stack (batching, replay header) for the AI lane
  to merge at E05.
- Increment 4 (ADV-Q16): `Accept` and `Decline` read as left and right (an answer whose meaning
  depends on the decision's kind); the owner's stream at load in `strict-fixed` (the setup's
  randomness is the setup stream's); all of a battle's levels drawn at once (the original draws
  each after the previous answer, whose skill enters the next draw); the first offer as the
  system actor's default (not the original's; a fallback if the army's path proves costly).
- Increment 5 (ADV-Q21): the points and the reveal at the placement, before the levels at load
  (a Scouting or Logistics gained at load would show only from the next step or turn, unlike the
  original's); the levels at load before the adventure's state is built (the evaluation needs the
  static map's terrain shares, which the adventure's build computes).
- Increment 6 (ADV-Q24): a field of the battle's setup for the tactics flag (`cross-lane`, mapped
  on the wire; the zeroing gives the same battle); zeroing only the flagged hero's Tactics (it
  would hand the other side the phase when its level is lower but above 0, which the original
  never does); an answer of its own to close a meeting or an army dialog (`Decline` already
  closes a decision without carrying anything out); a subset of the management commands in the
  adventure lane's encoding now. At T51's start: every management command in `legal_commands`
  (each call in a player's turn would build up to a few thousand of them, and the env's random
  rollout, which picks among its entries, would play differently).
- Increment 7 (ADV-Q29): one answer type for every decision (today's, with a `Buy(u8)` added; the
  user: "there should be specific types tailored to the choice to be made"), or typed answers
  for increment 7's decisions alone beside the old ones; a command of its own for the
  University's purchase (a purchase answers the open decision and keeps it open, as a meeting's
  management does: no new routing, E08 decision 140); the rewards given at the battle's end,
  before the level-ups (a level-up's Wisdom or Learning would not count for a box's spells and
  experience, nor Expert Wisdom for the Pyramid's spell, unlike the original); the army dialog's
  rest given before it opens (VCMI's order, D-39: batch E7 showed the code's side, E08 T58).
- Increment 8 (ADV-Q33): the env's observation of boats left to the AI lane's inbox after the
  build (its agents would see a hero in a boat as on its tile until then).
- Increment 9 (ADV-Q36 to ADV-Q38): the week and the month as state of their own (they follow
  from the day); the first week's weekly contents drawn from the game's stream (a build draws
  nothing from it); the neutral seat playing an owned mine's guards (batch E9's E9-2 showed the
  owner playing them).
- E09: the original's unreserved pick and late marks (T-2, T-3: one hero could stand twice on
  the map; E09 decision 12: fixed, not a flag); the town draws after the existing placeholders
  (the drawn heroes' armies and experiences would need a second walk); every starting hero drawn
  until E10, or the pick for every colour (TWN-Q2, E09 decision 9).
- E09's increment 2: a visitor building at an ally's town from its own treasury (T12a's reading
  of the hall's path, TD-11): refuted by batch TWN-2 (the build dialog greys Build for a town the
  current player does not own; E09 decision 42); the computer's every other day on Easy and
  Normal now (E09 decision 43: no computer player yet); the town events in the player's order of
  its towns (VCMI's: the original's list is the map's); firing an every-day event from the day
  before its first (VCMI's, TD-12: the original's day).
- Increment 10 (ADV-Q41 to ADV-Q43): every amount in the legal set (each creature's 1 to its most:
  up to the count per creature, where the most and a query of the range serve, as a split's
  counts); an ally's dialog as an event (VCMI shows none; the original shows the disabled dialog,
  batch E10's E10-6, so a client draws one view; built with E10's teams); the dialog open after a
  one-creature purchase (the original closes it: survey `E08-dwellings.md` §5.3, E10-4);
  `Answer::Close` for the dialog (the exchange dialogs'; the kind's own answer instead, as the
  University's `Leave`, so an answer names its decision); the random dwellings drawn after the
  existing placeholders (their footprints fix the tiles the later draws read); the guards on the
  owner's side (VCMI; rejected by the user, E08 decision 238).
- Increment 11 (ADV-Q45 to ADV-Q47): the exit named by the command (VCMI's, D-53), so no draw;
  the exit's draw from the game's stream or a stream inside the adventure state; the jump kept in
  the open battle and given at its end, before the winner's levels; the revisit for the
  teleporters only, or later as a stub (decision 268); the revisit with the guard's test; one env
  entry for the selected hero instead of one per slot; a jump dropping the entrance's guard, or
  the guard first for the teleporters only (decision 279).
- Increment 12 (ADV-Q50): the exchange as the Leave Guards dialog's copied group, written back at
  the close (one rule fewer to share with the town's rows, but views stale until the close); the
  exchange without a decision (any management command naming a garrison the hero stands in,
  which the original's modal dialog does not allow); a new action for the exchange in the env;
  the day of the week kept in the adventure's state; the day's and the week's clears for every
  hero of the store (the marks in the details).

## Revision history

- 2026-10-10: Accepted (E08 T1a draft).
- 2026-10-10: point 4: increment 1 has one setup placeholder, a placed hero's starting army,
  drawn in T6b (the user's choice at the E08 T4 checkpoint over an `Unsupported` import).
- 2026-10-10: point 3: a day's movement points are set at the start of each player's turn
  (ADV-MOV-002, E08 T6b: the original's code, the user's interim choice in ADV-Q4), so not every
  rule of increment 1 runs at the start of the day; the hooks themselves are unchanged.
- 2026-10-12: points 1, 4 and 6–9 extended and point 10 added for increment 2: the treasury,
  gains and costs by outcome, the first pending decisions (the Cartographer, the border guard),
  the env's answers (E08 T11b, the user at ADV-Q10; survey `E08-objects.md`).
- 2026-10-14 (E08 T18a; the user at ADV-Q12 and ADV-Q13): points 1, 4, 6 and 8–10 extended, point
  2 amended and point 11 added for increment 3: the store of heroes read by reference and changed
  through hero changes, the system actor for an unowned battle side (the original's battle AI its
  faithful policy), a battle's obstacles and a monster's split by the profile (`ADV-QUIRK-5`,
  `-6`) and its seed from the attacker's stream, chained offers and battles as pending
  decisions, the env's battle phase and neutral seat, battles through the game (drafted in E08
  T17b, #181; survey `E08-battles.md`).
- 2026-10-20 (E08 T26a; the user at ADV-Q15 and ADV-Q16): points 1, 4, 5, 6, 8, 9 and 11 amended
  for increment 4, hero progression: the level seed beside each hero and the scenario's limit
  and banned skills, levels at load instead of the import's refusal, a level's draws by the
  profile (`ADV-QUIRK-9`), the seeds server-only, a two-offer level-up as the owner's pending
  decision answered `First` or `Second` one level at a time with the original's choice as the
  system default (with the setup's inputs until E10, ADV-Q17), action encoding v4 and the
  observation's level-up, the versions, the battle's winner levelling after the outcome. Drafted
  in E08 T24b2 (#321; survey `E08-progression.md` §10 for the evaluation).
- 2026-10-21 (E08 T30; the user at T30's checkpoint, E08 decision 65): point 6 amended: an open
  level-up holds the game until its owner answers (every other actor's command rejected, the
  game naming the player it waits for), as the original's modal dialog; the hot-seat handover is
  the client's, the paused turn timer, the owner's time limit and the default answer online the
  server's. Point 8: `to_move` yields the owner alone meanwhile. T30's first build let the other
  players act on, which allowed a battle against a hero whose level-up was still open.
- 2026-10-22 (E08 T35a; the user at ADV-Q20 and ADV-Q21): points 1, 3 and 9 amended for E08
  increment 5, the hero's movement and scouting: the levels at load before a placed hero's day's
  points and the reveal around it (the original's order); the start-of-turn step reads the store,
  the start-of-day step's first rule (`strict-fixed`'s reveal around every hero, `ADV-QUIRK-18`);
  the versions. Drafted in E08 T34b (#375; survey `E08-hero-movement.md` §1.4, §4.2).
- 2026-10-23 (E08 T45; the user at ADV-Q24 and ADV-Q25): points 1, 6, 8, 9 and 11 amended for E08
  increment 6, army and hero management: the management commands and the expansions as the
  game's configuration (E08 decision 104); a meeting and an army dialog as pending decisions
  closed by `Decline`; the adventure's RL encoding to the AI lane, the replay model staying the
  adventure lane's (E08 decisions 99, 103); the versions; the tactics flag zeroing both heroes'
  Tactics. Drafted in E08 T44b (#412; survey `E08-army.md` §1, §2, §4, §6).
- 2026-10-24 (E08 T51; the user at T51's start, E08 decision 117): point 6 amended: the
  management commands are listed per hero by a query, as moves are, not by `legal_commands`,
  which keeps `EndTurn` and the answers (the design note's fallback for the list's size, taken
  before the build: `E08-design.md` §16).
- 2026-10-27 (E08 T59; the user at ADV-Q28 and ADV-Q29): points 1, 4, 6, 8, 9, 10 and 11 amended
  for E08 increment 7, rewards on the map: the setup's human flag and the scenario's spell and
  artifact bans; increment 7's draws at the build; answers typed by decision (E08 decision 138),
  increments 2 to 6's migrated, the University a decision open until `Leave`, the army dialog
  over a box's or an event's leftover stacks, the visit's rest after the level-ups (E08 decision
  141); the env's mapping onto the typed answers; the versions; the outcome's lost resources,
  choices and the move's stop; the battle setup's facts from the game (E08 T54, decision 124),
  increment 7's guards and the battle bonuses cleared at a battle's end. Drafted in E08 T55b
  (#484; survey `E08-rewards.md` §8.1, §8.6).
- 2026-10-29 (E08 T70 `shared/`; the user at ADV-Q32 and ADV-Q33): points 1, 8, 10 and 11
  amended for E08 increment 8, boats and the sea: no new command or decision, the boats in the
  setup and a hero on water refused, the env's observation with the boats, the battle at sea (the
  deck by either hero's boat, the two boats by both) and a boat leaving with its beaten hero.
  Drafted in E08 T66b (survey `E08-sea.md`), batch E8's outcome folded in.
- 2026-10-30 (E08 T81a `shared/`; the user at ADV-Q36 to ADV-Q38, E08 decisions 192–204):
  points 1, 3, 4, 6, 8, 10 and 11 amended for E08 increment 9, mines, the week and owned
  objects: `BuildBoat`; the owners and the timed events in the setup; the day's start in the
  original's order from day 2 (the week's and the month's steps, the income, the spell points)
  and the timed events at each player's turn start; the month's kind; the streams of the setup's
  and the day's draws; Leave Guards as a kept army dialog, the boat offer; the env's
  `BuildBoat`; a mine's guards in battle, an owned mine's played by its owner; the core
  creature's abilities outside battle. Drafted in E08 T77b (survey `E08-economy.md`), batch E9's
  outcome (T80, §19: D-47 confirmed, E9-2 the owner's guards) folded in.
- 2026-11-03 (E09 T4a `shared/`; the user at TWN-Q1 to TWN-Q4, E09 decisions 6, 8 to 12, 14, 16
  to 18, 21): points 1, 2, 3, 4, 9, 10 and 11 amended for E09 increment 1, towns on the map: the
  store of towns and the hero pool (its life, the pick reserved before any draw, T-2 and T-3
  fixed); the setup's towns, factions, choices, picks, main towns and random-hero flags with
  their checks (the original's offer of 16); the expansions by the packs ("the moat, E09"
  closed); the observation's towns; a town's day's income; the build's town draws before every
  existing placeholder (T-4 a typed failure); the town lane's versions; capture with the
  Capitol's limit, no command; a siege's setup and defender. Drafted in E09 T2a (survey
  `E09-towns.md` §2, §4, §5, §8, §9, §11); batch TWN-1 (T3c, survey §15) confirmed TD-3, TD-5,
  TD-7 and the siege's aftermath, its building 23 involving no draw (E09 decision 25).
- 2026-11-03 (E08 T92 `shared/`; the user at ADV-Q41 to ADV-Q43, E08 decisions 231, 235–238, 241,
  243): points 1, 3, 4, 6, 8, 10 and 11 amended for E08 increment 10, dwellings and hiring: no
  command; the week's refill and the camps' draws; the random dwellings' and camps' draws; the
  guards' and the hire questions, the hire dialog; the env's v7; the purchase's outcome; the
  guards' battle unowned. Drafted in E08 T88b2 (survey `E08-dwellings.md` §4, §5, §6, §7, §12);
  batch E10's outcome (T91, §16: every row as read) folded in.
- 2026-11-07 (E08 T102 `shared/`; the user at ADV-Q45 to ADV-Q47, E08 decisions 260, 264–268,
  273, 279): points 1, 4, 6, 8, 10 and 11 amended for E08 increment 11, the teleporters: the
  command `Revisit`; the jump's draw from the acting player's stream, passed to the adventure's
  `apply`; no decision kind, the jump after a won battle at an exit a visit's rest, a gate's
  meeting, the guard met first for every entered object; action encoding v8; the jump held as an
  outcome; the battle at an exit. Drafted in E08 T98b2 (design part C); batch E11's outcome
  (T101, survey `E08-teleporters.md` §15: every row as read but E11-19, the guard first) folded
  in.
- 2026-11-07 (E09 T15 `shared/`; the user at `TWN-Q7` to `TWN-Q10`, E09 decisions 37, 41–48):
  points 1, 2, 3, 4, 6, 7, 8, 9 and 10 amended for E09 increment 2, construction with the town
  events: the town command `GameCommand::Town(TownCommand::Build)`, its owner's; the owner's
  views of its buildings, the built-today mark public; the day's start clears the marks, the
  town events at the turn start after the timed events; no draw, the Shipyard forbidden without
  water at the build (`TWN-STUB-2`'s Shipyard half); no decision kind; the inventory; the env's
  build entry; the versions; a build's and a town event's outcome, the capture's mark. Drafted in
  E09 T13 (design note `E09-design.md` §29 to §32); batch TWN-2 (T14c, survey
  `E09-construction.md` §13) confirmed point 1's owner test (TD-11) and decision 41 (TD-9).
- 2026-11-12 (E09 T24 `shared/`; E09 decisions 65 to 83, `TWN-Q13`, `TWN-Q14`; E08 decision 305):
  points 1 to 4, 6, 8 to 11 amended for E09 increment 3: the week's kind and the towns' stream
  (`StreamId::TOWNS`), the day's order with the towns' week and month, the garrison heroes' turn
  start, income and weekly resources, `TownCommand::Recruit` and the garrison hero's moves, the
  hire dialog's town sources and the rows without a decision, the env's entries, the versions,
  the applications, a town's battle with a garrison hero. Drafted in E09 T22a and T22b; batch
  TWN-3's verdicts (E09 T23) cited.
- 2026-11-12 (E08 T112 `shared/`; the user at ADV-Q48 to ADV-Q50, E08 decisions 295, 299–303,
  305–309): points 1, 2, 3, 4, 6, 9, 10 and 11 amended for E08 increment 12, garrisons and the
  bonus visitables: no command, `ArmyRef::MapGarrison` and the day of the week in the adventure's
  application; a map garrison's view by the viewer's level and the kind marks in the
  observation; the day's and the week's drops, the Springs' refill and the Fountains' draws (in
  point 3 before E09's garrison heroes' turn start); the exchange as a decision kind and a
  visit's rest, the town's rows reusing its armies held together; the versions; the outcomes; a
  garrison's battle and the anti-magic flag, the bonuses from the hero on the map, the clear's
  kept marks. Drafted in E08 T108b (survey `E08-garrisons-bonuses.md`); batch E12's outcome
  (T111, §15: every row as read) folded in.
