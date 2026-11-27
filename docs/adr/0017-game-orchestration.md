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
   **E09's increment 4** (the Tavern; E09 decisions 99, 103 to 113) adds two commands and one
   decision kind (point 6). **`TownCommand::Hire { town, slot }`** (ADR-0028 point 6; TWN-TAV-003)
   hires the player's tavern hero of that slot in an own town with a standing Tavern and no visitor,
   for the part's price, while the player has fewer heroes on the map than the part's `most_heroes`
   (8; a garrison hero is not counted, TWN-ROW-002): the hero stands on the town's entrance as its
   visitor (batch TWN-4's TWN-4-2 and TWN-4-6, E09 T34: 2 500 gold a hire, the hero the visitor; the
   Recruit button disabled with a visitor and at 8 heroes on the map, enabled at 7 with a garrison
   hero). **`Manage::Dismiss { hero }`** (GAM-MAN-003; E09 decision 112): the player's hero on the
   map leaves the game (point 11), refused for a town's visitor and for the player's last hero on
   the map while it owns no town (the original's `hero_screen_refresh` 0x4E1A50, survey
   `E09-tavern.md` §12); a garrison hero is not on the map and is not dismissed (the town screen
   offers no dismissal); the loss condition's "lose this hero" waits for E10's scenarios
   (`TWN-STUB-16`). Both are their player's, in its turn, with no decision open (GAM-DEC-001), each
   refusal typed and changing nothing.
   **The state.** **The pool** (TWN-SET-005 amended) gains a status **offered** (a tavern hero's
   type: taken by no colour, out of every draw; the original's 0x40) and, per hero type, **its
   hero's id once it has a record** (E09 decision 109). A type's **record** (its hero id and its
   details in the store) is made the first time a draw offers the type; a placed or a generated
   hero's type has its record from the build. From then on the type keeps its id and its details
   whatever happens to it: a hero that leaves the game and is drawn again, by any player, comes back
   with its level, experience, skills, spells and artifacts (the original's 156 hero records, survey
   §4; batch TWN-4's TWN-4-5: a dismissed hero drawn again with its two artifacts). The store of
   heroes grows by one record at each type's first draw (the next id, so in the order of the draws)
   and never shrinks. A new record's details are GAM-HER-001's defaults under the map's **map-wide
   entry** of its type (SoD's customised hero types; `GameSetup` gains them for every type, the
   import reads them, `cross-lane`): the default experience's draw, or a custom experience's levels
   at load (GAM-SET-002), then its level seed (GAM-HER-002), each from the stream of the draw that
   made it (point 4), and its spell points full. Beside each recorded type off the map the pool
   keeps its **away record**, what the adventure keeps of a hero on the map and hands back when it
   leaves (ADR-0016 point 3): its points left and the day's maximum, its formation and tactics flag
   and, while it is a tavern hero, its army; its battle bonuses and timed marks are dropped at the
   leave (E08 decision 307). **Each player** gains **its faction** (the chosen faction:
   TWN-SET-001's draw kept; a Random choice no start reads is drawn at the first fill instead, from
   the taverns' stream, so the setup stream does not move; none without a choice) and **its two
   tavern heroes** (the survey's offers: slot 0 and slot 1, each a hero id or empty, and per slot
   the **returning mark**, the original's bit 17 of +0x105: the hero came back by a retreat or a
   surrender since its owner's turn last ended and the day last started, point 3). The pool, the
   away records, the factions and the slots are one block shared by forks and copied on a write (a
   fill, a hire, a leave, a turn's end or a day's start that clears a mark).
   **The draw** (TWN-TAV-001; `tavern_hero_draw` 0x4BB5E0): per hero class of the core, in class
   order, its weight the part's tavern weight of the player's faction (ADR-0028 point 5) when the
   class has a free type the player's colour may take (the masks), else 0; the other slot's class 0
   while another class has a free type; for slot 0 (own faction only) every class not of the
   player's faction 0 while one of its faction's weighs; a class by one draw over the total weight,
   then one of its free takeable types in type order, uniform. No free takeable type: the slot stays
   empty. A player without a faction weighs every class 1 and has no own faction (the agent's call:
   only setups without towns lack one). The SoD edition's Conflux rule is not modelled (dead in this
   build, E11's lead). Batch TWN-4 confirmed the draw (E09 T34; survey `E09-tavern.md` §14, the
   class draws logged with their ranges): slot 0 of the faction's two classes over their weights
   alone and slot 1 over 100 less the other slot's class's weight, each type among its class's free
   ones (TWN-4-1); the two Conflux classes weighing 5 in every slot-1 draw (TWN-4-8).
   **Increment 4's setup** gains, per hero type, the map's customised entry (above); nothing else:
   the masks, the allowed heroes and the prisoners are increment 1's.
   **E09's increment 5** (the mage guild; E09 decisions 162, 167 to 177) adds two town commands
   (ADR-0028 point 6). **`TownCommand::Open { town }`** (TWN-GLD-006; TD-27, E09 decision 170)
   stands for the original's town screen opened from the town list and its guild's window looked
   at: in an own town, in its turn, with no decision open, it gives the Mana Vortex (TWN-GLD-009)
   to the town's garrison hero, then to its visitor, then teaches the visitor, then the garrison
   hero (TWN-GLD-004); refused only for an unknown town or another's (`UnknownTown`, `NotOwner`),
   changing nothing; listed by `legal_commands` only where it would change something (E09 decision
   175). **`TownCommand::BuySpellBook { town }`** (TWN-GLD-007; E09 decision 176) buys the core's
   spell book for the town's visitor, else its garrison hero, in an own town with a standing Mage
   Guild, for the part's price (500 gold), while that hero wears none; then the town's heroes learn.
   Both are their player's, in its turn, with no decision open (GAM-DEC-001), each refusal typed
   and changing nothing.
   **The state.** Each town's record (ADR-0028 point 2) gains its guild's twenty slots, drawn when
   the game is built (TWN-GLD-002, the towns' stream, point 4), its Mana Vortex's week flag and,
   static, its Aurora set; the store of towns shares and hashes them as the rest (the town state
   hash v4).
   **The setup** gives each town its map's **required** and **banned** spells (`TownSetup`, the
   import: TWN-SET-012; an RoE map's required set empty, its banned set read), each the ruleset's
   (`GameSetupError`'s spell refusal, after the scenario's bans); the scenario's banned spells
   (GAM-SET-003) are read by the draw and, with `TWN-QUIRK-14`'s flag off, by the Aurora set, both
   at the build: the game keeps no ban after it.
   **The build** draws every town's guild from the towns' stream after the towns' records exist
   (each faction resolved, its buildings set; the original's `town_setup` 0x5C08B0 after the
   buildings), in `TownId` order; then the main towns' heroes learn (TD-10, E09 decision 167: the
   visitor, then the garrison hero; the original's `main_town_heroes` 0x4CA1A2), after every draw
   of the build. Batch TWN-5 confirmed the draw's exclusions and the start's learning (E09 T44,
   survey `E09-guild.md` §21: TWN-5-1, a spell both required and banned in no slot, TD-25;
   TWN-5-4, the main town's generated hero knowing the guild's shown spells on day 1, TD-10).
   **E09's increment 6** (the town's trades; E09 decisions 197, 209 to 213) adds five town commands
   (ADR-0028 point 6), each its player's, in its turn, with no decision open (GAM-DEC-001), the town
   the actor's (an ally's waits for E10's teams, `TWN-STUB-21`), each refusal typed and changing
   nothing: **`TownCommand::Market { town }`** (TWN-MKT-002; decision 209) opens E08's market
   decision with the town as its place (point 6); **`TownCommand::University { town, hero }`**
   (TWN-UNI-001; decision 210) opens E08's University decision for the named visitor or garrison
   hero; **`TownCommand::BuyWarMachine { town, building }`** (TWN-SMT-001; decision 211) buys the
   building's machine for the window's hero, the Stronghold's Blacksmith and Ballista Yard each by
   its own building; **`TownCommand::BuildBoat { town }`** (TWN-SHY-001; TWN-SET-011 amended)
   launches the part's boat at the town's spot, no hero needed, several a day while the spot frees;
   **`TownCommand::Transform { town, army, slot }`** (TWN-TRF-001) turns one whole stack of the
   visitor's army or the upper bar's into its target in place, any number a turn. **The answer**
   `MarketAnswer::SellArtifact { place, take }` (TWN-MKT-005; ADV-MKT-002 amended) is new, offered
   only by a town's window with the merchants (point 6). Batch TWN-6 (E09 T54, survey
   `E09-trades.md` §14) confirmed the window's hero (TWN-6-5: the visitor alone, T-28), the
   University's banned school (TWN-6-7: T-23) and the town's one spot (TWN-6-6: TD-31, T-21).
   **The state.** A town's record gains nothing (the town state hash stays v4); the adventure state
   holds the merchants' stock and the artifact pool's marks (ADR-0016 point 1; decision 212).
   **The build** draws the stock's seven (TWN-STK-001) last among its setup draws (point 4), so they
   avoid the map's artifacts, the random ones and the Black Markets' while the pools last
   (confirmed by batch TWN-6's TWN-6-8, D-32: the map's placed artifacts left out on day 1).
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
   **Increment 13** (creature banks; E08 decision 333) adds **no command**: a bank is visited by
   `MoveHero`'s last step onto its entrance (the ten entered kinds) or toward it (the Shipwreck,
   from the coast too) and by `Revisit` on an entered one; its question is answered with the
   offers' answers, its battle with the battle's commands, its reward's army dialog with `Close`
   and the management commands (batch E13's E13-11: the move ends on the entrance, the hero stays
   there through the question, and `Revisit` asks again).
   **Increment 14** (the trades and services; E08 decision 361) adds **no command**: the seven
   classes are visited by `MoveHero`'s last step onto their entrance and by `Revisit`; their
   windows are pending decisions answered with `Answer`'s new kinds (point 6), the War Machine
   Factory's question with the offers' answers and its dialog with the hire dialog's (batch E14's
   E14-16: the move ends on the entrance, the hero stays there through the window, and `Revisit`
   opens it again).
   **The cast** (increment 15): `GameCommand::Cast { hero, spell, target }`, in the actor's turn
   with no decision open. The game checks, in order, each refusal typed and changing nothing: the
   actor's hero on the map; the spell castable on the map (`h3_heroes::can_cast_on_map`: among
   the available spells, worn scrolls included, no book test, ADR-0013 point 17.3); one of the
   adventure part's spells; a target exactly for Scuttle Boat and Dimension Door; the cost
   (`map_spell_cost` at the mastery on the hero's tile's ground, the raise from the battle part)
   against the spell points; the ground (the special ground's battlefield's highest spell level,
   then an anti-magic garrison's tile); then the adventure's checks (ADR-0016 point 4). The cost
   before the ground is the original's order (its book tests the points, then
   `hero_special_ground`; E08 decision 414): both refuse free, only the reported error differs. The adventure applies, the game sets the spell points less the cost (at
   least 0). Batch E15 saw the refusals change nothing (E08 T139a, survey `E08-spells.md` §18:
   E15-4's ground and garrison, E15-5's fifth door, E15-6's door without movement points, E15-11's
   cast aboard).
   **The prisoners** (increment 15): the build stores each prisoner's details as a placed hero's
   (its levels at load, army, skills, artifacts, spells, then its spell points), in the setup's
   object order after the placed heroes, the levels at load in hero id order with the others
   (E08 decision 410: the original's loop by type levels each hero from its own seed, so no
   order is the original's), in no player's list, its type `Prisoner` in the pool (E09's). A
   prison's visit outcome names it; the game frees it unless the visitor's player has the town
   part's most heroes on the map (garrisons not counted, as the hire): its colour takes the type,
   the hero joins the end of the player's list with its day's land points (E15-1, E15-2).
   **The teams and the game's end** (E10 increment 1; E10 decisions 9 to 19; design note
   `E10-design.md` §1, §2, §8 to §14): `PlayerSetup` carries the player's team (the map's, every
   colour its own without; checked dense at the build); `GameState` holds per player its team,
   its defeat (the day and why: no hero and no town, or the seven days out) and its days without
   a town (none, or 7 to 0), and the game's **result** once over (the winning team or none, and
   why: a conquest, no human left). The game **checks the end** after every accepted command and
   system command but `EndTurn`, at each turn's end before the outgoing player's count-down and at
   a human's turn start (never the game's first; point 3): the eliminations in colour order, then
   the verdict. A defeat is the game's step over the contexts' infallible functions: the
   adventure frees the player's ownables and takes its heroes off the map, the game moves them
   into the pool (E09's leave) and frees its tavern offers. Once over, **every command is
   refused** (`GameOver`), changing nothing, and no decision is open.
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
   plays the side at random. **Its own commands** (E16 M5 T14, SRV-Q31; built by the server
   lane) are one closed variant, `GameCommand::System(SystemCommand)`, each naming the player it
   acts for and checked and applied as that player's own command: `EndTurn { player }` (its
   turn running, no open decision), `Answer { player }` (its first open decision but a battle,
   answered by `default_answer`, which the engine takes: the host never chooses an answer) and
   `Battle { player, command }` (the side the player owns while it is to act: the host's
   computer plays a timed-out player's side for the rest of the battle, so the system actor
   commands an owned side through this command alone). The game's hold (point 6) judges a
   system command as its player's: a level-up's owner's default answer passes it. A player's
   system command is refused (`SystemOnly`). They are not among the legal commands: the host
   issues them by its clock, which stays outside the state, as its battle policy does; every
   step is a journaled command. Removing a player comes with E10. `Viewer::Player(PlayerId)` and `Viewer::Spectator`, which sees
   public facts only. Increment 1 builds the player variants; the system actor and the spectator
   come with the server's M5 as additive variants. *Amended 2026-11-14 (E16 M5 T21a, SRV-Q35,
   SRV-Q36):* no spectator viewer is built: a whole game's spectator is the host's, shown every
   player's own view (each `Viewer::Player`'s observation and projected events, as its seat gets
   them; ADR-0018 point 7), so the engine projects for players only; a viewer that is no player
   of the game sees the public facts and no tile. Contexts never see these types: a context's
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
   **E09's increment 4** (the Tavern; TWN-TAV-002 to -009): **a player's two tavern heroes are its
   own** (as its treasury): its observation gives, by slot, each tavern hero's view (its id and
   type, its details as the tavern's held hero screen shows them, its army by slot, its formation
   and tactics flag: what the original's tavern dialog and the offer's hero screen, held while the
   right button is, show; no points, no returning mark; E09 decision 134 at T37a: its level,
   experience, primary skills, spell points, skills and worn artifacts, the backpack's first five
   places and its count, no learned spells and no offered levels), whether or not it owns
   a Tavern (the agent's call: they tell nothing of another player), and `TavernOffered` (the view
   without its lists, then `TavernPartSeen` per stack, skill and artifact: `GameEvent` stays
   `Copy`, E09 decision 135) tells it a slot's new view (or its emptying) after every change: a week's release and fill (in player, then
   slot order, after the towns' week), a hire's replacement, a hero's return; the game's start's
   fill is the first observation's. **A hire** is told as `HeroHired` (the hero, its type, its
   owner, its tile; its slot to the owner alone) to the owner and to every player that sees the tile
   (ADR-0016 point 6), after the price's `ResourcePaid` and before the replacement's
   `TavernOffered`; the owner's view of the hero moves from the slot to its heroes, completed by
   `HiredPartSeen`, to the owner alone, per part the slot's view did not show (E09 decision 141:
   the levels last offered with the worn positions' count, the backpack past its first five places,
   the learned spells), its points by `MovementChanged`. **The Thieves' Guild** (TWN-TAV-007 to -009; ADR-0028 point 8) is a **query**,
   not a part of the observation (the original's window is a snapshot when it opens; an
   observation's view would need every command to tell every viewer's changed ranks):
   `h3_game::thieves_guild(state, rules, viewer)` answers while the viewer's turn runs and it owns a
   town with a standing Tavern, or its open decision is a tavern decision, at its level (its towns
   with a Tavern, counted to 5; 0 at a map Tavern without one: two rows and no part); otherwise
   none. A **Den of Thieves' visit** tells the visitor's owner the guild at everything
   (`ThievesGuildSeen` with its level, then `GuildPartSeen` per part: each shown row's places in
   row then player order, then each player's best hero and creature; `GameEvent` stays `Copy`,
   E09 T39b2). **The guild's view:** the players in the game in id
   order; the rows its level shows (2 rows at 0 or 1, 4 at 2, 6 at 3, 8 at 4, all 9 from 5: towns,
   heroes, gold, wood and ore, the four rare resources, obelisks visited, artifacts, army strength,
   gold income), each as its places, from the highest value, equal values sharing a place (in player
   order within it), never a value; per player from level 1 its best hero's type, from 2 that hero's
   four primary skills (each 0 to 99), from 4 its best creature (survey `E09-tavern.md` §6, §13;
   TD-23 the reading, decision 118: the best hero by army power, the towns' shown armies in the army
   row and the best creature). The artifacts row ranks the backpacks alone under `TWN-QUIRK-13`
   (T-17, decision 117: `fix-optional`, on in `faithful`; off, the worn and the backpack's
   artifacts, VCMI's). The player's tactic (from 3) is the computer's play and not shown
   (`TWN-STUB-18`); the obelisks row reads 0 for every player until increment 6 places the obelisks
   (`TWN-STUB-17`). A defeated player's column is E10's (every player is in the game until then).
   The guild shows nothing a click opens: another player's best hero and creature open no screen
   (`thieves_guild_proc` 0x5C9930). Batch TWN-4's TWN-4-7 (E09 T34) read every level, row and part
   so: the best hero the strongest army's, not the most experienced; the artifacts by the backpacks
   alone; the army with a town's garrison counted and the best creature from it; VCMI's best hero,
   artifacts, army and best creature refuted.
   **E09's increment 5** (TWN-VEW-004 amended): **a town's guild is its owner's** (the original
   shows it in the owner's town screen only; `E09-guild.md` §6): the owner's town view gains, per
   level 1 to 5, the spells the guild shows (TWN-GLD-003: none above the guild level; the drawn
   spells but never the unshown slots), whether a building of role `aurora` teaches every spell
   there (derivable, given for the client's scroll), and the Mana Vortex's week flag (the owner
   knows whether it took the Vortex this week). No other viewer sees any of it, not even Visions'
   top level. A build that changes what the guild shows (a level, the Library), a capture and the
   Vortex's cleared flag are told as the town view's change (`TownChanged`, TWN-VEW-001's events),
   after the command's own events; every week's set flag in the week's views. **Learning** is
   told as `HeroChanged { change: LearnSpell(spell) }` per spell, in the order learned, to the
   hero's owner (as a meeting's, E08 T51b); **the Vortex** as `HeroChanged { change:
   SetSpellPoints(points) }` to the hero's owner, then the town's `TownChanged`; the **night's
   refill** as the day's regain is (HER-MAN-002: no event, the points in the owner's observation).
   The book's purchase is told as `ResourcePaid`, then `HeroChanged { change: AddArtifact(book) }`,
   then the learning's. Batch TWN-5 read what the guild shows (E09 T44, TWN-5-3: a Tower's Mage
   Guild 5 shows 5, 4, 3, 2, 1 spells by level, with the Library 6, 5, 4, 3, 2, the spares at once).
   From increment 12 the observation adds to each **map garrison** a viewer sees (by object: type
   and anchor) its army at the viewer's level, which the game derives as a town's garrison's (the
   owner's team, a hero with Visions at level 3 within reach of the entrance, else the viewer's
   towns with a Tavern, at most 2), and its removable flag to the owner's team; and the types
   whose kind the viewer's team has marked (ADR-0016 point 6).
   From increment 13 a viewer sees a **creature bank**'s type and, by its team's visit mark, the
   original's hint, the adventure observation's object view (ADR-0016 point 6); no game-level view
   is added. A team's new mark is told with the hint after it.
   **E10's increment 1** (design note `E10-design.md` §6, §14, §15): the system's closed set
   gains **`HandOver { player }`** (SRV-Q29's removal, E10 decision 15): the player's `human` flag
   cleared, told to every player, then the check; refused for a defeated player or a computer;
   its turn and pieces stay, the host plays the colour with its system commands (and, with E31,
   the original's computer player). No player command removes a player. **Views:** each player's
   team, defeat and whether it is human are public, its days without a town its own, the result
   public; a viewer sees a hero, town or garrison of its team at the popups' top level (ADR-0016
   point 6).
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
   **From E09 increment 4** (TWN-TAV-002, -005; GAM-CAL-002 amended) a new week's start, after the
   towns' week (the week's kind, the neutral garrisons, the Portals) and before the adventure's week
   step, gives **the taverns' week** (the original's `new_day_week_events` 0x4C8780: after the
   towns' growth, before the dwellings): every player's two slots, in player order, are **released**
   (each type free again, the slot empty) but a slot whose returning mark is set (TD-18, E09
   decision 103; batch TWN-4's TWN-4-3, E09 T34: a hero that came back in another player's turn
   stayed through the week while the other slot was released and drawn anew; VCMI's release of both
   refuted); then each player in player order **fills** its empty slots, slot 0 then slot 1
   (`tavern_new_heroes` 0x4C8450): each the draw of point 1, the type offered, its record made if it
   has none, then the type's **full starting army** (GAM-SET-001's draw, replacing whatever army its
   away record held; a rolled war machine gives its artifact where a position takes it, as
   GAM-HER-001), its backpack's artifacts equipped from the last where a free position takes each
   (`hero_equip_artifact` 0x4E2A00) and its spell points full (batch TWN-4's TWN-4-1: every offer of
   eight weeks a full starting army). A player defeated (E10) is not filled. **The game's start**
   fills every player the same way (nothing to release) after the build's last draw, before day 1's
   start-of-day step (`new_game_heroes`' taverns, 0x4C00C0, after the main towns' heroes).
   **Every day's start then clears every player's returning marks** (E09 decision 126; the
   original's `new_day` clears bit 17 of every hero record's +0x105 after the week's and the month's
   events, 0x4C80CF to 0x4C80FD, with the day marks above), after the taverns' week when one runs:
   the week's release still spares a hero that came back after its owner's turn ended (TD-18), and a
   mark lasts until the next day's start at most (survey `E09-tavern.md` §14). **The day's spell
   points** reach every hero record, a tavern hero and a hero out of the game too (ADR-0013 points
   16.3 and 16.5, HER-MAN-002; E09 decision 127, TD-20's spell-point half amended: the original's
   156 records, 0x4C8237).
   **At its owner's turn start** each tavern hero gets its day's maximum and points (TWN-TAV-005;
   ADV-MOV-002's derivation over its away record's army, as a hero on the map's; TD-20's movement
   half, E09 decision 104: batch TWN-4's TWN-4-3 saw a returning hero's points refilled at its
   owner's next turn), after the player's heroes on the map and before the garrison heroes (the
   original's `next_player_turn`). **At its owner's turn end** its two slots' returning marks are
   cleared too (`next_player_turn` 0x4C705D, after the victory tests), before the next group's turn
   starts: a hero that came back in its owner's own turn goes at the next week's release, and one
   that came back after that turn ended stays through it.
   **From E09 increment 5** (TWN-GLD-008, -009; GAM-CAL-002 amended) **every week's first day**
   sets every town's Mana Vortex flag with the towns' week (the original's `new_day_week_events`
   0x4C8A09), and **every new day**, after every hero's daily regain (HER-MAN-002), raises each
   town's visitor's, then garrison hero's spell points to their maximum (HER-MAN-001), never
   lowering them, for each town in `TownId` order with a building of role `guild` ever built (the
   original's `new_day` 0x4C82E4: no spell book, no owner test; `E09-guild.md` §14). Day 1 has no
   new day, so no refill. **No turn start teaches or gives the Vortex** (TD-27, E09 decision 170):
   a hero that gains Wisdom or a spell book while it stands in a town learns at the next trigger
   (point 10). Batch TWN-5 confirmed both (E09 T44, TWN-5-5: the garrison hero at 20, its maximum,
   at week 2's and week 3's turn starts, refilled and not doubled; the flag set again each week).
   **From E09 increment 6** (TWN-STK-002; GAM-CAL-002 amended) **the first day of every month from
   the second** (days 29, 57, …) ends the month's start with **the merchants' draw**: after the
   month's kind, the towns' month and, in a creature's month, the adventure's month step
   (ADV-MON-014), before the towns' day, as the original's `new_month` ends with it (survey
   `E09-trades.md` §3.2). Every entry is redrawn in order, sold or not, the old ones' marks kept (an
   unsold artifact stays out of the pool until its class resets), whether or not any merchants
   stand (decision 207); an entry stays empty where the pool gives none (T-24 fixed by
   construction, `TWN-QUIRK-21`: the original loops for ever). Between two draws a purchase empties
   its entry for every town and player; a sale never fills one; nothing else writes the stock.
   Batch TWN-6 confirmed it (E09 T54, TWN-6-8: on day 29 all seven redrawn, the bought entry
   refilled, sold or not; D-32, the marks reset free the placed artifacts).
   From increment 10 **the adventure's week step** first refills **every dwelling**, owned or not
   (each count its creature's growth, set; the guards the kind's full guards; no week's or month's
   kind), then draws the weekly objects' contents and **the camps' creatures** in one pass in
   object order, then grows the monsters (GAM-CAL-002 amended; ADR-0016 point 4; batch E10's E10-8
   and E10-9).
   **E10's increment 1** (design note `E10-design.md` §9 to §11): a **defeated** player leaves
   the schedule: its turn ends at its defeat (its group's ends when the others' have), it never
   acts again, and it leaves every step that walks the players (the day's income, the week's
   tavern heroes, the Thieves' Guild, the market's recipients; the original's readers of its
   defeated byte). At **each turn's end** the game checks, then lowers the outgoing player's days
   without a town by one when above 0 (a computer's too); at a **human's** turn start it checks
   before the turn's timed events (a computer's runs none), and every turn start tells a player
   with days its days left after them. Building the state checks nothing (A-57, `ADV-QUIRK-61`).
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
   **E09's increment 4** draws from **the taverns' stream** (`StreamId::TAVERNS`, a new named kernel
   stream beside `TOWNS`, a `shared/` change; E09 decisions 113 and 115: no existing draw moves, so
   the replays' outcomes keep but for the new state) at the game's start's fill and at every week's
   fill, per filled slot in this order: the class, the type, a new record's draws (its experience or
   its levels at load, then its level seed), then its army (the first count, then each later slot's
   roll and, when given, its count; the original's `draw_starting_army` 0x4C9730 order). A Random
   faction choice that no start read is drawn there at the player's first fill, before its slot 0.
   **A hire's replacement** (TWN-TAV-004) draws from **the hirer's stream** after the hire is
   checked (a refused hire draws nothing): the class, the type, a new record's draws; its army is
   one creature of the type's first starting stack, without a draw (batch TWN-4's TWN-4-2, E09 T34:
   one class and one type draw a replacement, the other slot's class avoided, any faction, one
   creature with its spell points full). **A hero's return** after a retreat or a surrender
   (TWN-TAV-006) draws its slot, one draw of two (the original's `random_between(0, 1)`, then the
   other slot when that one holds a returning hero; batch TWN-4's TWN-4-4), from **the stream of the
   player whose hero returns**: its own command (the flight, the surrender) ends the battle. A loss,
   a draw, a capture's leave and a dismissal draw nothing. The map Tavern's hire draws as the
   town's.
   **E09's increment 5** draws **the guilds from the towns' stream** (`StreamId::TOWNS`; E09
   decision 179, the user's over a new named stream: no `shared/` change, but every later draw of
   the towns' stream moves, so the replays that play a week re-record their outcomes): when the
   game is built, before any other draw of that stream, every town in `TownId` order, its levels 1
   to 5, each slot not given to a required spell and with a total weight above 0 one draw over the
   total (the original's `random_between(1, total)` and its running sums). Nothing in play draws a
   guild. The original's one generator interleaves these draws with the rest of its setup's; the
   engine's order is its own (E08 decision 17). Batch TWN-5 saw the guild drawn once at the start
   (E09 T44, TWN-5-3: the Library's spares as drawn at the start, shown when it is built; a
   Castle's level-5 slots empty, its weights 0 there).
   **E09's increment 6** draws **the merchants' stock** (TWN-STK-001, -002; design note
   `E09-design.md` Part H §101, call 4): **the new game's seven** from the setup stream, last among
   the build's setup draws (after the Portals' `town::portal::at_setup`, the setup stream's last
   today), so no earlier draw moves but every state hash does; **every later month's** on its first
   day (point 3) from **the towns' stream** (`StreamId::TOWNS`, as decision 179's guilds; E09
   decision 214, over a named stream of its own), so every replay that passes day 28 changes its
   later towns' draws (the census of growth to day 29, `h3-sim`'s town-map seeds), which T57
   re-records and re-seeds in one pull request. Entry by entry in the part's class order, equal
   chances among the class's unmarked artifacts; with none left the marks reset to the bans, then
   any class but `special` (relics and combinations through the fallback only); the drawn one
   marked. Batch TWN-6 saw the draw so (E09 T54, survey `E09-trades.md` §14, TWN-6-8, D-32: on day 1
   the three minors the map does not place, on day 29 the minors from all 21 once the marks reset).
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
   **Increment 13's draws** are setup placeholders, drawn from the setup stream when the game is
   built, in the second walk with the chests, the gifts, the Shrines, the finds and the Pyramid, in
   object order: per bank its level by the chances, its artifacts by class (relics first) from the
   one pool of the random artifacts, then its upgrade's chance where the upgrade's slot holds a
   creature with an upgrade. No visit, battle or reward of increment 13 draws. **A bank battle's
   seeds** are every battle's: its own seed the attacking player's stream's next draw, its
   obstacles' seed as above, the generator giving none; the original's seed of the bank's tile
   (170 101 x + 223 252 y + 250 037 z + 82 037) has no counterpart, since the engine's battle
   draws are its own. Batch E13's E13-1 found the original's records in the walk's order (each
   start drawing anew, the 27 levels against their chances with no evidence against them).
   **Increment 15's draws:** Summon Boat's and Scuttle Boat's chance draws come from the caster's
   stream, one draw each, in the cast's order (the draw made at 100 % too, as the original's).
   **Each player's list of heroes at the build** (E08 decision 413, GAM-SET-004): the game hands
   the adventure its heroes in the start list's order: each player's placed heroes by hero type
   (a random hero by its drawn type), then the main towns' generated heroes in player order (the
   original's `placed_heroes_setup` and `main_town_heroes`). The heroes' ids stay the setup's;
   the prisoners are in no list.
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
     **E09 increment 4** adds **the tavern decision** (GAM-DEC-010; E09 decisions 106, 111): a visit
     of the adventure map's Tavern (class 95, ADR-0016 point 4) opens it for the visitor's owner
     (`map_tavern_visit` 0x5D7E90); its answers are `Hire { slot }` and `Close`. A hire is checked
     as the town's (the slot's tavern hero, `most_heroes`, the price), without a visitor test, and
     is refused while a hero stands on the Tavern's visited tile (VCMI's; the original never reaches
     it, since a hero on the tile hides the Tavern from a step, ADR-0016 point 4); a refused answer
     is typed and leaves the decision open. A hire places the hero on the Tavern's visited tile,
     draws the slot's replacement and closes the decision (the original's dialog returns after one
     hire, `map_tavern_hire` 0x4D7890); no guild spells. Its view (`DecisionView::Tavern`) is the
     visitor and the Tavern by type and anchor, told whole by `TavernOpened` to the visitor's owner;
     the offers it hires from are that owner's tavern heroes (point 2), so no view of it waits on a
     later event. The town's tavern opens no decision (the town command, point 1), as the town's two
     rows.

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
   and at the capture of one, and **after a won garrison battle**, which the won battle's own
   rest gives (`Rest::Won`, GAM-DEC-007's third: the flag, then the exchange; no rest of its
   own), once the winner's levels are answered, as the original's order; nothing
   when the winner left the map (E12-1, E12-2, E12-6: the exchange at a move's end, after a flag
   and after a won battle). Its view (the owner's): the garrison by type and anchor, its army by
   slot, its removable flag. The town's rows reuse its army reference and the one owner rule,
   not the decision (E09 decisions 67 and 80, the town's two rows above; E09 T28a): an own
   town's two bars are admitted outside any decision, with `ArmyRef::Garrison(town)` or the
   garrison hero's army as the stationary bar. The bonus visitables ask nothing.

   **Increment 13 adds no decision kind.** A bank's question is an offer of the visitor's owner,
   `Fight` for the attack kinds and `Search` for the search kinds, routed by the object's kind
   when accepted (as `Fight` routes a mine's); a yes opens the battle at once (point 11), or with
   no guards standing gives the reward at once. A won bank's reward is **a won battle's rest**
   (GAM-DEC-007's third, `Rest::Won` of the entrance), given once the winner's levels are
   answered: the creature onto the army or into the army dialog (GAM-DEC-004), the artifacts read
   when given, the resources, then the loot. This is the original's order for a human, whose
   levels come inside the battle's call (`adventure_battle` 0x4AE324) before
   `bank_battle_reward`'s reward (E08 decision 342, replacing 337; survey `E08-banks.md` §6,
   D-67); every player answers as a human (point 1). Batch E13 confirmed it (E13-6: the level-ups,
   then the reward's message, then its army dialog; E13-7: the artifacts in the record's order).
   **Increment 14's decisions:** three new kinds, each its owner's alone, holding no one else and
   blocking its owner's `EndTurn` like an offer, open until its `Leave` (every other answer keeps it
   open, as the original's windows stay open for more trades), the system actor's default
   `Leave` (nothing is lost: a sacrifice names everything it takes in one answer, A-48 fixed,
   E08 decision 366):
   - **A market** (GAM-DEC-011), opened by a Trading Post's, a Freelancer's Guild's or a Black
     Market's visit, answered **`Market(MarketAnswer)`**: `Trade { give, take, lots }` (whole lots
     at the rate, from 1 to what the treasury holds), `Transfer { resource, amount, to }` (1 : 1 to
     another player still in the game, allies and enemies alike), `Sell { slot, take, lots }` (the
     army keeping one creature), `Buy { entry, pay }` (an unsold entry at its price; refused
     before paying when the hero cannot receive the artifact, A-46 fixed), `Leave`; an answer of a
     screen the object lacks is refused. Its view shows the screens with their rates, the stock
     and the recipients.
   - **An Altar of Sacrifice** (GAM-DEC-012), answered **`Altar(AltarAnswer)`**:
     `Artifacts(places)` (at most 23, none kept), `Creatures(amounts by slot)` (one creature kept),
     each with a total above 0 and only where the hero's class's faction allows it, and `Leave`.
     The total is gained as experience with its levels: levels without a decision at once, the
     Altar staying open; when a level-up opens, **the Altar closes before it and its visit is the
     visit's rest** (GAM-DEC-007, `Rest::Visit`): once the level-ups are answered the Altar opens
     again and a later sacrifice reads the new level (the original's dialog beneath its level-up
     dialogs; a client keeps the Altar drawn beneath them, E08 decision 371). A player keeps one
     open decision. Its view shows each artifact's and each stack's experience and the experience
     to the next level.
   - **A Hill Fort** (GAM-DEC-013), answered **`HillFort(HillFortAnswer)`**: `Upgrade(slot)` a row
     whose price the treasury holds, `All` when some row upgrades and the treasury holds every
     row's price (each paid and upgraded in slot order), `Leave`. Its view shows each row's
     upgrade, price and state and the whole's.
   - **The War Machine Factory's question** is the hire question (`Offer(Consent)`, [`Decline`]);
     accepted, **the hire dialog** (GAM-DEC-008 amended) with the factory as its source: each
     machine with a count of 1 while the hero lacks its artifact, else 0; `Buy { creature, amount:
     1 }` pays the creature's cost and gives the artifact into its position (no army change), the
     dialog of three staying open; `Close`.
   `legal_commands` lists a representative answer per action, as the hire dialog lists each
   creature's most: a market's trade of one lot per pair it can pay, its sale of one lot for gold
   per slot, its purchase in gold per unsold entry it can pay; the Altar's two "all" sacrifices
   (the original's buttons); the Hill Fort's upgrade per affordable row and its `All`; each
   `Leave`. No transfer is listed (as the management commands, E08 decision 99). Any other answer
   the view allows is still legal for `apply`. Batch E14 (E08 T130, survey `E08-trades.md` §21)
   saw the windows so: each open for more trades (E14-1, E14-5, E14-8), the factory's dialog open
   after each purchase (E14-15), a refused upgrade a message with nothing paid (E14-13), the
   Altar's level-up drawn over its dialog, which then stays open with the new level read (E14-17).
   **E09's increment 6** (GAM-DEC-011 and GAM-DEC-006 amended; E09 decisions 209, 210) adds **no
   decision kind**: a town opens E08's market and University decisions as their **place**, a variant
   of the visit's place (a map object as today, or the town by `TownId`), not a second decision.
   - **The market** (GAM-DEC-011): its hero none possible at a town (the window's hero, ADR-0028
     point 6); open until its `Leave` and blocking as E08's; its hash writes the place's tag before
     its fields (the game state hash). E08's answers stand at the trader's count (TWN-MKT-003, -004;
     confirmed by batch TWN-6's TWN-6-2): `Trade`, `Transfer` (to any other player still in the
     game), `Sell` (the window's hero's army), `Buy` on the merchants' stock (`Entry`, `NoPrice`,
     `NoRoom` with A-46 fixed as `ADV-QUIRK-48`, then `CannotPay`); a screen the town does not offer
     is refused (`Screen`). **`SellArtifact { place, take }`** (TWN-MKT-005): one artifact of the
     window's hero by its `h3_heroes::ArtifactPlace` (a worn position or a backpack index), refused
     in this order: a window without the sale (`Screen`), an empty place (`Place`), a worn artifact
     whose slot the backpack does not take (HER-ART-002's: the war machines, the Catapult, the spell
     book; `NotTradable`), a resource the parameters do not value (`NoPrice`). SoD's fifth misc
     position sells like the others (T-22 fixed, decision 205, `TWN-QUIRK-20`), every backpack
     artifact too, the Grail for 1 at least, a combination whole from its own position. Its view
     (`MarketView`, the window's owner's alone): the place, the hero, E08's `MarketOffers` (the
     screens, the count, the lots, the stock with its prices) with the sale's places and prices,
     the recipients.
   - **The University** (GAM-DEC-006): its visit names the place, the hero, the skills (the part's
     four; with `TWN-QUIRK-16`'s flag off the scenario's banned ones, `GameState::banned_skills`,
     left out when it opens; on, a banned school offered and taught, as batch TWN-6's TWN-6-7 saw
     the original) and the price; E08's answers (`Buy(slot)` by `h3_heroes::university_slot`
     against the actor's treasury, several a visit; `Leave`); no mark at a town. Its view: the
     place, the hero, the skills with their slot states, the price.
   `legal_commands` lists each new command where it is legal (point 8): `Market` where the window
   opens and offers a legal answer; the sale for the cost resource per place that sells;
   `BuyWarMachine` where the purchase is legal; `BuildBoat`; `University` per hero where a slot is
   open and the treasury holds the price; `Transform` per stack that changes. Inside a town's window
   the answers are listed as a map market's, no transfer.
   **E10's increment 1** (design note `E10-design.md` §5, §13; E10 decision 16): a step toward a
   hero of the visitor's **team** opens the meeting as the visitor's owner's decision, as for an
   own hero; onto the ally's hero it allows **gifts only**: a stack into an empty slot, an
   artifact into an empty place or the backpack, nothing taken, none of the ally's settings
   changed (typed refusals; D-82 the original's); the ally is told its hero's changes. An ally's
   dwelling's hire dialog opens read-only, `Close` alone (built). The game's end drops every open
   decision.
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
   **E09's increment 4** appends (`cross-lane`, the AI lane's paths, E09 decision 7) **the town's
   hire**: two entries, `HIRE + s` for the tavern hero of slot s, with the town by its entrance in
   the tile head (one action per `Hire` the legal commands list: a standing Tavern, no visitor, the
   heroes on the map below `most_heroes`, the price in the treasury; encoding v12). **The tavern
   decision** takes the answers by role (encoding v13, or with v12 when they land together): its
   `Hire { slot }` of slot 0 and 1 the first and second answers, its `Close` the closing role, each
   legal as the legal commands list it. **The dismissal has no entry** (a management command, as the
   army commands: E08 decision 99), nor has the Thieves' Guild (a query, no command): the AI lane
   may read the query for its agents.
   **E09's increment 5** appends (`cross-lane`, the AI lane's paths, E09 decision 7) **the town's
   opening** (one entry, the town by its entrance in the tile head; one action per `Open` the legal
   commands list, so only where it would change something: encoding the next at merge) and **the
   spell book** (one entry, the same head; listed where the town's guild stands, its hero lacks a
   book and the treasury holds the price; the next version, or the same when both land together).
   The env reads the guild through the owner's town view; no learning is an action.
   **E09's increment 6** appends (`cross-lane`, the AI lane's paths, E09 decision 7) **42 entries**
   (the encoding from 105 to 147), each legal where `legal_commands` lists its command, the town by
   its entrance in the tile head: **the market's opening** (1); **the artifact sale** for the cost
   resource by place (22: worn positions 0 to 13, then backpack places 0 to 7); **the Blacksmith's
   machine** (2: the town's first and second building with a `war_machine`, in building order);
   **the boat** (1); **the University's opening** (2: the visitor, the garrison hero); **the
   transformation** (14: the two armies' seven slots), each listed as point 6 says. The trades,
   sales and purchases inside a town's window are increment 14's 56 entries, the University's
   purchases its four, each window's `Leave` the closing role; no transfer (E08 decision 372). The
   next version at merge per task, or one when tasks land together; the sale's 22 places, the
   largest, are a candidate to trim at its task if the AI lane prefers (E09 decision 216 kept the
   42, over the env without the sale's).
   **Increment 14's action encoding** (the next version at merge: v14 after E09's v12 and v13)
   appends 67 entries, each legal where `legal_commands` lists its answer, tile 0 alone: a market's
   trade of one lot per pair (42, given then wanted in the core's order), its sale of one lot for
   gold per slot (7), its purchase in gold per entry (7); the Altar's two "all" sacrifices (2); the
   Hill Fort's upgrade per slot (7) and `All` (1); one `Leave` for the three kinds (E08 decision
   372, `cross-lane`). The factory uses the offer's and the hire dialog's entries. An amount head
   stays the AI lane's.
   **Increment 15's action encoding v21** (E08 T144a2, decisions 421, 422; the AI lane's,
   `cross-lane`): five entries per hero slot after the spell book's purchase (40, from `CAST_BASE`
   105: slot `s`'s cast of the behaviour of code `c`, the original's spell order Summon Boat,
   Scuttle Boat, Fly, Water Walk, Dimension Door, is `105 + 5s + c`), each naming the core spell
   the adventure part binds to that behaviour. Summon Boat, Fly and Water Walk take no tile (tile
   0 alone) and are legal where `legal_commands` lists the hero's cast; Scuttle Boat's boat and
   Dimension Door's arrival are the **tile head's** tile (the encoding has no slot head for
   windows around a hero, decision 421), the entry legal where the hero's targets query is not
   empty and its tile mask that query's targets (decision 422). `legal_commands` lists the casts
   without a target last, per hero in the player's list; **Scuttle Boat's and Dimension Door's
   targets are a per-hero query**, `legal_cast_targets(state, rules, actor, (hero, spell))`, in
   row order, as the moves are `legal_moves`' (listing every target wrote up to 646 commands per
   caster on every call). The env's random agent draws each listed command, each hero's move and
   each hero's targeted cast that has a target once, a move's tile and a cast's target then
   uniformly (the AI lane may choose other weights). The own hero's observation carries its day
   spells (the door's casts made among them); no field counts the casts left: the masks say
   whether a cast is legal.
   **Each player's list of heroes** (E08 decision 413): the viewer's observation carries its list
   (`GameObservation::hero_list`); the wire's and the protocol's fold keep it from the viewer's
   heroes on the map (one that left leaves it, one that came joins its end); a player's hero slots
   are its list's first eight places.
   **E10's increment 1** (design note `E10-design.md` §16): the env's step reports the game's
   result and each player's defeat; a defeated player or an ended game has no seat to move. The
   action encoding stays v23 (the hand-over is the host's). Who is human is the seat's mark
   (`PlayerSetup::human`): the verdict's "no human left" reads it, and a game built with no human
   plays to one team.
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
   **E09's increment 4** bumps the game state hash (the pool's statuses, ids and away records, the
   players' factions and tavern heroes, the taverns' stream), the adventure ruleset part (the kinds
   `Tavern` and `DenOfThieves`, ADR-0016 point 4; the adventure state hash does not move: the kinds
   hold no state), the map import (classes 95 and 97, the map-wide entries in the setup) and the
   adventure replay format (`town hire`, `answer tavern`, `manage dismiss`), each `cross-lane`; the
   protocol (the server lane's, `cross-lane`: `TownCommand::Hire`, `Manage::Dismiss`, the tavern
   decision and its answers, the observation's tavern heroes, `TavernOffered`, `HeroHired`,
   `TavernOpened`, `ThievesGuildSeen`, the guild's request and answer) and the env's action encoding
   (point 8: v12, v13; the AI lane's, `cross-lane`). Shared (E09 T35b): `StreamId::TAVERNS`
   (decision 115) and the quirk flags of `TWN-QUIRK-12` and `-13` (no version moves: the
   profiles' ruleset hashes do, as E08's Sirens flag's). The numbers are the next at merge, after E08's increment 13 (merged first: map import
   v24).
   **E09's increment 5** bumps the game state hash (the store's v4 folded in), the map import (the
   towns' required and banned spells into `TownSetup`) and the adventure replay format (`town
   open`, `town book`), each `cross-lane`; the replays that play a week re-record their outcomes
   (the guild's draw moves the towns' stream, decision 179); the protocol (the server lane's,
   `cross-lane`: the two commands and the new `TownCommandError`s, the owner's guild, Aurora mark
   and week flag in the town view) and the env's action encoding (point 8, the AI lane's,
   `cross-lane`). The adventure state hash and ruleset part do not move (`AdventureOutcome`'s town
   entered is not state, ADR-0016 point 4). Shared (E09 T45): the quirk flag of `TWN-QUIRK-14` (no
   version moves: the profiles' ruleset hashes do); no new stream (decision 179). The numbers are
   the next at merge, in merge order with E08's increment 14 (E08 T133b took game state hash v35,
   replay format v21, encoding v14 and protocol v75 after the client lane's replay format v20, E27
   M4 T3a; its T134a took game state hash v36, replay format v22, encoding v15 and protocol v76).
   **E09's increment 6** bumps the adventure state hash (the pool's marks, the merchants' stock;
   T57), the game state hash (the adventure's folded in, T57; the decisions' places, T58a and T59b)
   and the adventure replay format (`town market`, `town machine`, `town boat`, `town university`,
   `town transform`, `answer market sell-artifact`; T58a to T59b), each `cross-lane`; every golden
   adventure replay is re-recorded at T57 (the marks and the stock; the towns' stream's draws on day
   29 in the replays that pass it), each changed line classified; the protocol (the server lane's,
   `cross-lane`: `MarketWindow`'s object or town with its hero optional, the University's place,
   `SellArtifact` with its refusals, the five commands with theirs, the events' places) and the
   env's action encoding (point 8, the AI lane's, `cross-lane`), the next per task at merge. The map
   import and the adventure ruleset part do not move; the town part v10 is the town lane's
   (ADR-0028 point 9). Shared (E09 T55b): the quirk flags of `TWN-QUIRK-15` and `-16`, confirmed by
   batch TWN-6's TWN-6-5 and TWN-6-7 (no version moves: the profiles' ruleset hashes do), and the
   combinations' costs (`h3-content`, HER-ART-008: the core ruleset hash moves); no new stream
   (decision 214). The numbers are the next at merge, in merge order with the adventure lane's open
   work (E08 T142b, #1162, and its queue): the second to merge rebases and bumps again.
   **Increment 12** bumps the game state hash (v28: the exchange, its rest, the battle bonuses
   leaving the details), the adventure state hash (v22), the adventure ruleset part (v20), the
   map import (v22) and the adventure replay format (v13: the map garrison's reference in a
   management line); the env's action encoding stays v8 (the exchange is answered by the close
   role). The numbers are the next at merge, after E09's increment 3 where it merges first.
   **Increment 13** bumps the adventure state hash (v26: the banks' draws and guards), the
   adventure ruleset part (v21: the kind) and the map import (v24: the five classes); the game
   state hash, the adventure replay format (v16) and the env's action encoding (v8) stay. The
   battle lane's battle state hash and battle part and the shared pack format move with the bank
   battle (ADR-0022 point 8), in its `cross-lane` task. The numbers are the next at merge, in
   merge order with E09's increment 4.
   **Increment 14** bumps the adventure ruleset part (the five kinds, the market parameters), the
   map import (the seven classes), the adventure state hash (the stock's draw and sold entries),
   the game state hash (the three decisions, the rest's name), the adventure replay format (the
   answers' words) and the env's action encoding (the AI lane's, point 8), and `h3-protocol`'s
   protocol once per wire task (the server lane's). The core ruleset hash and the pack format
   stay. The numbers are the next at merge; after E09's increment 4 (merged first: part v22, map
   import v26, game state hash v34, replay format v19, encoding v13) they are part v23, map import
   v27, adventure state hash v27, game state hash v35, replay format v20 and encoding v14.
   **Increment 15** bumps the adventure ruleset part (the two kinds, the spells' bindings and
   parameters), the map import (the prisons and the Market of Time), the adventure state hash (the
   day spells), the game state hash (the prisoners, the cast), the adventure replay format (`cast`),
   the env's action encoding (the AI lane's, point 8) and `h3-protocol`'s protocol once per wire
   task (the server lane's); the next at merge: part v25, map import v29, adventure state hash v28,
   game state hash v39, replay format v24, encoding v18, protocol v79. *As built* (E08 T141a to
   T144b): part v25 (the kinds) and v26 (the spells), map import v29, adventure state hash v28 (the
   list of heroes) and v29 (the day spells' block), game state hash v39 (the prisoners), replay
   format v26 (`cast`, T142a), encoding v20 (the list's slots) and v21 (the casts), protocol v81
   to v84 (one per wire task). Shared (E08 T140): the six
   quirk flags of `ADV-QUIRK-15` and `-51` to `-55` in the core's profiles (no version moves: the
   faithful profiles' ruleset hashes do); no new stream.
   **E10's increment 1** (design note `E10-design.md` §18) bumps the map import (the teams), the
   adventure state hash (the teams, the explored layer per team), the game state hash (the teams,
   the defeats, the days, the result), the adventure replay format (`hand-over`, the end) and
   `h3-protocol`'s protocol (the server lane's); the next at merge: map import v30, adventure state
   hash v31, game state hash v41, replay format v29, protocol v87 (E09 T58b, merged after the
   draft, took replay format v28, protocol v86 and encoding v23). The action encoding stays.
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
    A visit's outcome may name **a Den of Thieves** (ADR-0016 point 4): the game composes the
    visitor's owner's Thieves' Guild at everything after the visit's other effects and tells it
    (point 2); nothing is paid or changed.
    **E09's increment 4: a hire's application** (the town's or the map Tavern's) is infallible once
    checked: the price paid, the hero placed by the adventure with its away record, its type taken
    by the player's colour (the colour's bit added to the type's mask, `hero_place` 0x4D7900), the
    slot's returning mark cleared, the replacement drawn into the slot. A town's hire would teach
    the town's guild spells (`town_heroes_learn_spells` 0x5BE030; batch TWN-4's TWN-4-2: a hired
    hero with a spell book learned them, a Knight none); the guild is increment 5's (`TWN-STUB-15`).
    **E09's increment 5: the town's heroes learn** (TWN-GLD-005; the original's nine callers of
    `town_heroes_learn_spells` 0x5BE030, `E09-guild.md` §12; TD-27 the original's) at each of
    these, as the town's part returns it (ADR-0028 point 2: the visitor first, then the garrison
    hero, each its `LearnSpell`s), in one infallible application with the command:
    - **a visit's end**: a step that enters a town of the team, a `Revisit` there and an
      undefended town's capture (the outcome's town entered, ADR-0016 point 4), after the
      capture's own effects: first the opening's Vortex (the garrison hero, then the visitor; the
      original's town screen opened by the visit, `town_screen_init` 0x5C63C0), then the learning;
    - **a siege's end** (whoever won; not a battle against a visitor beside a garrison hero, which
      is no siege): after the battle's outcome and a capture, the town's visitor and garrison hero
      as they then stand (E09 decision 171; the attacker stays outside, ADR-0016 point 4), no
      Vortex;
    - **a build**, the command's and a town event's (TWN-BLD-005, TWN-EVT-003): after `TownBuilt`
      and its views, the learning, then the Vortex (the garrison hero, then the visitor; the
      original's `town_build_effects` 0x5BEDE0 learns before the building's visit);
    - **a town's hire** (TWN-TAV-003; lifts `TWN-STUB-15`): after the placement and its events, the
      learning (the hired visitor, then the garrison hero), then the Vortex to the hired hero
      (`town_tavern_click` 0x5D846C); the map Tavern's hire teaches nothing;
    - **a move between the town's bars** (TWN-ROW-002 to -004): after the move, while a visitor
      stands, the learning (the original's bars rebuilt, `town_screen_bars` 0x5C7007);
    - **the opening** (`Open`) and **the book's purchase** (`BuySpellBook`), point 1;
    - **the start** (TD-10; lifts `TWN-STUB-6`), point 1.
    Town Portal's and Castle Gate's arrivals (increment 7) and the computer's re-visits and book
    purchases (E10, E13) are not built here. **The Vortex** (TWN-GLD-009) is the town's part's
    check per hero in the order named, its flag cleared at the first that takes it. **The book's
    purchase**: the price paid, the book given (`HeroChange::AddArtifact` of the core's spell book,
    worn at its position), then the learning; no refusal after the checks. Batch TWN-5 confirmed
    the triggers it ran (E09 T44, survey `E09-guild.md` §21: TWN-5-3's visit, TWN-5-4's start,
    TWN-5-5's visit and opening with the garrison hero first; no turn start).
    **E09's increment 6: the trades' gains and costs** (design note `E09-design.md` Part H §105 to
    §113), each applied infallibly once checked, a rejection leaving everything untouched, every
    rate and price the state's answer, never the command's:
    - **A trade, a transfer and a creature sale at a town** are increment 14's at the trader's
      count (TWN-MKT-003; at least 1, so never the division by zero the original's 0 makes; the
      rows confirmed by batch TWN-6's TWN-6-1, the trader's count by TWN-6-2); the transfer 1 : 1
      up to the whole stock, its gain told to the recipient alone; the sale at the creature factor
      of the count (a whole gold cost from 8 Marketplaces), the army keeping a creature.
    - **The merchants' purchase** (TWN-MKT-004): `MarketParams::artifact_price(count, cost, pay)`
      (a combination's cost its parts' sum, ADR-0028 point 5; at one Marketplace 5 times the cost,
      TWN-6-8); paid, the artifact given (`HeroChanged`), the entry emptied for
      every town and player until the next month's draw.
    - **The artifact sale** (TWN-MKT-005): `MarketParams::artifact_sale(count, cost, take)` (E08's
      module, `cross-lane`), max(1, round(f32(f32(cost × A) / v(take)))) with A the artifacts'
      factor at the count and v the trades' value table (`market_artifact_sale_rate` 0x5ECDC0,
      `E08-trades.md` §4), the X87 steps of the other three rates (a combination at its parts' sum,
      confirmed by batch TWN-6's TWN-6-4); the artifact removed
      (`HeroChange::RemoveArtifact`, its parts' locks freed, its effects gone), then the price
      gained (`ResourceGained`); the stock, the pool and the spell points untouched. **T-20 fixed
      by construction** (`TWN-QUIRK-18`): the place priced is the place removed (T-20 confirmed by
      batch TWN-6's TWN-6-3: the original prices backpack place 0 and removes the place shown).
    - **The machine** (TWN-SMT-001): its creature's cost, each resource its own (`TWN-QUIRK-22`),
      paid (`ResourcePaid`), then the machine given (`HeroChanged`; a war machine has its own
      position).
    - **The boat** (TWN-SHY-001): launched at the spot (E08's `launch`, an empty boat of the part's
      type, its builder the actor), `BoatBuilt`, then paid (`ResourcePaid`), E08's order for a
      Shipyard (batch TWN-6's TWN-6-6: the original's price taken after the ship is shown).
    - **The University's purchase** is E08's (the part's price; `HeroChange::RaiseSkill` at
      Basic); **the transformation** (TWN-TRF-001) sets the slot's creature to the target, its count
      kept (`StackChanged`), free, the army's last stack allowed, nothing merged (T-26 fixed by
      construction, `TWN-QUIRK-23`: one command one whole stack, no holding army to lose).
    **No new event kind** (Part H §101, call 13): `MarketOpened` and `UniversityOpened` name their
    place (a map object or a town); every change is an existing event (`ResourcePaid`,
    `ResourceGained`, `StackChanged`, `HeroChanged`, `BoatBuilt`, `DecisionClosed`), told to whom
    E08 tells it (`ResourceGained` to a transfer's recipient alone; `BoatBuilt` by the fog's rules;
    `StackChanged` and `HeroChanged` to the owner and to the viewers who see the army or the hero);
    the fold gate covers each.
    **Increment 12's outcomes:** the exchange is an outcome the game opens (or holds after a
    battle); a movement gain, the Sirens' losses and the Stables' upgrade, the battle bonuses and
    the timed marks are the adventure's own changes of the hero on the map and its army (E08
    decision 307: the bonuses no longer a hero change, ADR-0013 point 2); the spell points and the
    Sirens' experience are hero changes the game applies through `h3-heroes`, the experience's
    levels given as every gain's (batch E12's E12-7: 310 experience for the lost hit points). No
    price, no gain of resources.
    **Increment 13's outcomes:** a won bank's end holds its reward as the visit's rest
    (`reward_after`); the reward is a reward record's outcome (ADV-RWD-001): the creature's join
    or its group without room, the artifacts as hero changes, the resources as gains, the loot the
    adventure's own change; a looted search's once-bonus is the adventure's own change of the hero
    on the map (ADR-0013 point 1 as amended for increment 12). Batch E13's E13-10 and E13-7 saw
    the level's resources and artifacts so.
    **Increment 14's outcomes:** a visit names the window to open (the market, the Altar, the Hill
    Fort) or the factory's question. **A trade:** the adventure computes the rate (the lot) from
    the part's parameters; the game checks the lots against the treasury, pays the given resource
    and adds the wanted one (saturating). **A transfer** is the first cost and gain of two players:
    the actor pays, the recipient gains 1 : 1, each told its own change; nothing else of it is
    seen. **A sale:** the adventure checks the slot, the lot and the army's last creature and
    names the gain; the game adds it after the adventure's infallible change. **A purchase** (a
    Black Market's, a factory's): the adventure checks the entry or the machine and the hero's
    room and names the price; the game checks the treasury, pays, then calls the adventure's
    infallible change. **A sacrifice:** the adventure checks the places or amounts and names the
    total; the game applies the changes and gives the experience with its levels. **An upgrade:**
    the adventure names each row's price; the game checks the treasury, pays, then calls the
    upgrade. A rejection leaves everything untouched. Every rate, price and most is the state's
    answer, never the command's. Batch E14 saw the gains and costs so (E14-1, E14-5, E14-6: Red
    −5 ore, Blue +5 on its turn; E14-4, E14-8, E14-10, E14-12, E14-15).
    **Increment 15's outcomes:** a prison's visit names its prisoner (the game frees it or reports
    the eight; no decision); a cast's outcome names its effects (a boat moved, created or sunk, a
    hero relocated, the day spells set) or its charged failure with its reason; the spell points
    are the game's change (the hero's details), never the adventure's. Batch E15 saw them so
    (E15-1, E15-2; E15-7's charged door; E15-4's, E15-9a's and E15-9b's boats moved, E15-10's sunk).
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
    **From E09 increment 4** a hero that leaves the map **leaves the game** (TWN-TAV-006;
    GAM-BAT-003 amended; the original's `hero_leave_game` 0x4D9EC0): after a battle (lost,
    retreated, surrendered, a draw, a winner with created stacks only), a garrison hero without a
    stack at its town's capture (T-5) and a dismissal. The adventure hands back its away record
    (ADR-0016 point 3); its type goes free and its details stay in the store with its level, skills,
    spells and the artifacts it keeps (a beaten hero those the winner found no room for,
    GAM-BAT-004; a retreated or surrendered hero all of them, the battle's capture returning at
    once; a dismissed hero all of them, E09 decision 105; batch TWN-4's TWN-4-5, E09 T34: a
    dismissed hero's worn and backpack artifacts kept, T-14). **After a retreat or a surrender**
    (the battle's result for its side) its hero goes at once into its owner's tavern: a slot drawn
    (point 4), the other when that slot holds a returning hero; that slot's tavern hero, if any,
    goes free; the type offered, the returning mark set. A surrendered hero keeps its army (its
    side's survivors), a retreated hero keeps one creature of its type's first starting stack; both
    keep their points left and the day's maximum and their spell points as they were (the day's
    regain from then, point 3; batch TWN-4's TWN-4-3 and TWN-4-4: a retreated hero with 1 Pikeman, a
    surrendered one with its army, each marked with its points and spell points). **Every other
    leave** frees the type: the spell points full (HER-MAN-001's maximum) and the points left and
    the day's maximum the land maximum, which the original computes after clearing the army
    (`hero_max_land_movement` 0x4E4990 from the empty slots' start, speed 20: the table's most)
    under `TWN-QUIRK-12` (T-16, decision 116: `fix-optional`, on in `faithful`; off, the land
    maximum of the one creature it keeps; batch TWN-4's TWN-4-4: a lost hero's 2 000 points of 2 000
    over its one Centaur, its spell points refilled), the type free for any player's later draw,
    which gives it a new army. A new record's points are the land maximum of the army its first draw
    gives it (the original sets every record's at the new game with its full starting army; the
    agent's call). `ADV-STUB-9`'s retreat facts stay E10's whole (only a computer's hero reads them;
    their Tavern fact is `has_tavern` over the owner's towns from increment 4 on).
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
    **Increment 13's battles** (GAM-BAT-001 amended): a bank's guards defend as
    **`Defender::Bank`**, as they stand, unowned and without a hero (the neutral seat's side), at
    the entrance's tile; the game sets the setup's **`creature_bank`** from that defender, so the
    battle is a bank's by its cause, never by its tile (a hero met on a bank's entrance fights an
    ordinary battle: of the original's 14 calls of `adventure_battle`, only the bank's passes its
    flag). The attacker's war machines are listed as always and set up by no one (the battle's
    rule, ADR-0022 point 8). A bank of a deck kind (the Shipwreck, the Derelict Ship) fights on the
    sea terrain's deck, from the coast too. Its end: lost or retreated, the survivors stay with the
    bank; won, the reward after the winner's levels (point 6). Batch E13 confirmed it: the battle
    at the entrance (E13-11), the deck for both sea banks, the Shipwreck's from the coast (E13-5),
    the survivors left by slot after a retreat (E13-9), the worn machines not on the field
    (E13-3).

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
- Increment 13: a defender kind and the setup's bank flag (no decision kind, no rest kind, no
  command; the game state hash stays); its battles change the battle lane's setup (ADR-0022 point
  8, `cross-lane`); nothing changes on the wire (the events, the offers and the views are existing
  ones), in the replay format or in the env's encoding.
- E09's increment 4: the store of heroes grows by a record at a type's first draw and never shrinks
  (a fork copies its outer list once at that write); the pool's block (the records' away parts, the
  factions, the slots and their marks) is copied on a write, a day's clear of a set mark among them;
  the fills draw from the taverns' stream, so no existing draw moves and the golden replays keep
  their outcomes but for the new state (re-recorded with the game state hash). Two commands, a
  decision kind, four events and the guild's query reach the wire (the server lane's, `cross-lane`)
  and two of them the env's encoding (v12, v13, the AI lane's).
- Increment 14 (E08 T127b, T131): three decision kinds with typed answers and views carrying the
  rates (so neither a client nor the env needs a copy of the computation); the Altar's levels
  reuse the visit's rest; the factory reuses the hire question and dialog with a source of war
  machines; the first gain of a player other than the actor (a transfer); `h3-protocol` maps the
  new answers, views, events and errors at each wire task (`cross-lane`), and the env appends 67
  entries once every answer exists.
- Increment 15 (E08 T137b, T140): the cast is one command with an optional target, validated in
  the game and the adventure before anything changes; a charged failure is an accepted command,
  not a rejection, so a rejected cast still leaves the state untouched.
- E10 increment 1 (E10 T1b, T2): every loop over players must skip a defeated one; the check
  runs after every command, so a rule that changes a hero's or a town's count or an owner needs no
  call of its own. A finished game is a final state: replays, the env and the server stop at its
  `GameOver`.

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
- Increment 13 (ADV-Q53, T118b2): the reward before the winner's levels, a bank's exception to
  GAM-DEC-007 point 3 with the levels held until its army dialog closes (decision 337, on the
  survey's first reading; replaced by 342 when the re-reading found the original's human order
  the rest's); `BattleStart::bank`, a field (every literal, the town lane's siege among them,
  would change); the bank battle marked by its tile; a bank offer kind beside `Fight` and
  `Search`; the original's tile seed for the bank battle.
- E09's increment 4 (`TWN-Q16` to `TWN-Q19`): every allowed type's record at the build (about 140
  more heroes built and hashed a game) or a fresh record at every draw (a lost hero's level and
  skills not kept); the towns' stream for the fills (every later week's town draws shifting); VCMI's
  release of both slots (TD-18), its roles for a return's slot (TD-19) and its offers' daily
  movement (TD-20's movement half); the marks cleared only at the owner's turn end and the spell
  points set only at a draw and a leave (as drafted; the original clears and regains every record at
  the day's start, decisions 126, 127); the offers only with a Tavern (an event at every Tavern
  built or lost) and the guild in every observation (every command telling each viewer's ranks); an
  env entry per hero slot for the dismissal.
- Increment 14 (ADV-Q57, ADV-Q58): the windows as commands valid while the hero stands on the
  entrance (no decision; every trade would re-prove the object and nothing would end the window);
  the Altar's levels held until it is left (a later sacrifice would read the old level); the
  level-ups queued in front of the open Altar (two open decisions of one player); an event naming
  a transfer's giver to its recipient (the original's hot seat shows the treasury only); every
  trade amount in `legal_commands` (thousands per pair, as the hire dialog's amounts); a smaller
  env set without the market's trades, or no env change (E08 decision 372).
- Increment 15 (ADV-Q63): the cast as an adventure command validating the spell points itself
  (the adventure would hold or copy the hero's mana and the raise: rejected by ADR-0013 point 2);
  a target dialog as a pending decision (the original's modal dialog; nothing waits on another
  player, and a cancel is no command, so one command suffices); Visions as part of increment 15
  (E12 with the information objects, decision 406); the cast's book test (the draft's, until
  batch E15's E15-15 read none); the levels at load in type order, as the original's loop (E08
  decision 410: no outcome against the original differs).
- E10 increment 1 (ADV-Q65): VCMI's moment for the seven days (the defeat at the seventh turn's
  end; D-80); playing on after the last human (VCMI's, D-84); a removed player defeated with its
  towns made neutral, or kept by nobody's turn (D-83); merging gifts into an ally's stack (VCMI's,
  D-82); a check when the game is built (A-57); the check only at the original's 37 sites (each
  rule calling it: more code, the same verdicts but a days' defeat's moment).

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
  (T111, §15: every row as read) folded in. Point 6 worded as built at E08 T117 (increment 12's
  audit, `shared/e08-t117`): the exchange after a won garrison battle is the won battle's rest,
  not a rest of its own; the town's rows reuse the army reference and the owner rule outside any
  decision (E09 T28a).
- 2026-11-13 (E16 M5 T14, the server lane's `cross-lane` pull request with its `shared:` commit;
  the user at SRV-Q31 and at T14's start): point 2: the system actor's own commands,
  `GameCommand::System(SystemCommand)` (`EndTurn`, `Answer` by `default_answer`, `Battle` for a
  timed-out player's side), each naming its player and applied as that player's command; the
  hold judges them as the player's; refused from a player (`SystemOnly`); not listed among the
  legal commands (GAM-SYS-001 to GAM-SYS-003).
- 2026-11-16 (E08 T122 `shared/`, `cross-lane`; the user at ADV-Q52 to ADV-Q55, E08 decisions
  333, 335 to 339, 341 to 343, 348): points 1, 2, 4, 6, 9, 10 and 11 amended for E08 increment
  13, creature banks: no command; the banks' hint in the observation; the banks' draws when the
  game is built; no decision kind, the reward a won battle's rest after the levels (D-67
  corrected); the versions; the outcomes; `Defender::Bank` and the setup's bank flag by the
  battle's cause, the deck. Drafted in E08 T118b2 (survey `E08-banks.md`, corrected); batch E13's
  outcome (T121, survey §14: every row as read) folded in.
- 2026-11-16 (E09 T35a `shared/`; E09 decisions 99, 103 to 127; `TWN-Q16` to `TWN-Q19`): points 1 to
  4, 6, 8 to 11 amended for E09 increment 4, the Tavern: `TownCommand::Hire` and `Manage::Dismiss`,
  the pool's records and away records, the players' factions and tavern heroes with their returning
  marks, the draw; the tavern heroes' view and events, the Thieves' Guild's query and a Den's view;
  the taverns' week and the start's fill, every day's clear of the marks and every record's day's
  spell points (decisions 126, 127, the user at T35's start), the tavern heroes' turn; the taverns'
  stream and the replacement's and a return's draws; the tavern decision; the env's hire and
  answers; the versions (the adventure state hash unmoved, as the design note's §77); a Den's
  outcome and a hire's application; a hero that leaves the game; the Consequences and Alternatives.
  Drafted in E09 T33a and T33b (design note `E09-design.md` Part F); batch TWN-4's outcome (T34,
  survey `E09-tavern.md` §14: every row as read) folded in; placed after E08 increment 13's text
  (T122, merged first) where it follows E09 increment 3's.
- 2026-11-17 (E09 T37a `shared/`; E09 decisions 134, 135, `TWN-Q20`): point 2's increment 4 part:
  the tavern view is the held hero screen's (no learned spells, no offered levels, the backpack's
  first five places and its count; the code reading of `tavern_dialog_proc` 0x5D7D7D, the manual
  and VCMI agreeing), `TavernOffered` with `TavernPartSeen` per part.
- 2026-11-17 (E09 T38a1 `shared/`; E09 decision 141, `TWN-Q22`): point 2's increment 4 part: a
  hire tells its owner what the tavern slot's view lacked (`HiredPartSeen`: the levels last offered
  with the worn positions' count, the backpack past its first five places, the learned spells), so
  that a fold of the slot's view is the hero's details whole (T37a's As built's note for T38a).
- 2026-11-19 (E09 T39b2 `shared/`; E09 decision 154, `TWN-Q27`): point 2's increment 4 part: a Den
  of Thieves' guild is told as `ThievesGuildSeen` (the level) with `GuildPartSeen` per part (a
  row's place, a player's best hero and creature), as the tavern views' parts, not the view in
  one event; the wire's (protocol v74) `GameQuery::ThievesGuild` answers the query.
- 2026-11-19 (E08 T131 `shared/`; E08 decisions 361, 366, 371 to 373, 378; ADV-Q56 to ADV-Q59):
  points 1, 6, 8, 9 and 10 amended for increment 14, the trades and services: no command; the
  market, the Altar and the Hill Fort as decisions, the factory's question and hire dialog, the
  Altar's return after its level-ups as the visit's rest, the listed answers; the env's 67
  entries; the versions; the trades' outcomes, a transfer to another player; the Consequences and
  Alternatives. Drafted in E08 T127b (design note `E08-design.md` §13 to §19); batch E14's outcome
  (T130, survey `E08-trades.md` §21: every point as read; the import without the RoE mark,
  decision 378) folded in; placed after E09 increment 4's text.
- 2026-11-20 (E09 T45 `shared/`; E09 decisions 162, 163, 165 to 181; `TWN-Q31` to `TWN-Q33`):
  points 1, 2, 3, 4, 8, 9 and 10 amended for E09 increment 5, the mage guild: `TownCommand::Open`
  and `BuySpellBook`, the guild in the store and its draw at the build from the towns' stream, the
  setup's town spells, the owner's guild view, the night's refill and the Vortex's week, the env's
  two entries, the versions, the learning's triggers. Drafted in E09 T43 (design note
  `E09-design.md` Part G §82 to §100); batch TWN-5's outcome (T44, survey `E09-guild.md` §21:
  every row as read) folded in; placed after E09 increment 4's text.
- 2026-11-22 (E08 T140 `shared/`; E08 decisions 394, 404 to 406, 410, ADV-Q63): points 1, 4, 8, 9
  and 10 amended for increment 15, the cast and the prisoners. Drafted in E08 T137b (design note
  `E08-design.md` §14 to §18); batch E15's outcome (T139a, survey `E08-spells.md` §18: every row as
  read; D-79, no book test) and T139b's order (survey §2.1, decision 410) folded in.
- 2026-11-25 (E09 T55a `shared/`; E09 decisions 197, 198, 200 to 217; `TWN-Q35` to `TWN-Q37`):
  points 1, 3, 4, 6, 8, 9 and 10 amended for E09 increment 6, the town's trades: the five town
  commands and `SellArtifact`, the stock drawn last at the build and at every later month's start
  from the towns' stream, the market's and the University's decisions with a town as their place,
  the env's 42 entries, the versions, the trades' gains and costs. Drafted in E09 T53 (design note
  `E09-design.md` Part H §101 to §120), accepted at `TWN-Q37`; batch TWN-6's outcome (T54, survey
  `E09-trades.md` §14: every row as read) folded in.
- 2026-11-27 (E08 T145 `shared/`; E08 decisions 413, 414, 421, 422): points 1, 4, 8 and 9 amended
  for increment 15 as built: the cast's checks in the original's order (the target test, then the
  cost before the ground; point 1's sentence, which decision 414 placed in point 4); each
  player's list of heroes at the build (point 4) and in the observation and the env's slots
  (point 8); the action encoding v21's casts by hero slot with Scuttle Boat's and Dimension
  Door's targets by a per-hero query (point 8, replacing the paragraph written before T144); the
  versions as built (point 9). Drafted in E08 T141a and T144a2
  (`docs/lanes/adventure/adr-drafts/0016-0017-hero-list.md`, `0017-casts-encoding.md`, deleted).
- 2026-11-27 (E10 T2 `shared/`; E10 decisions 9 to 19, ADV-Q65): points 1, 2, 3, 6, 8 and 9
  amended for E10 increment 1, the teams and the game's end; a consequence, an alternative.
  Drafted in E10 T1b (design note `E10-design.md`;
  `docs/lanes/adventure/adr-drafts/0017-e10-increment-1.md`, deleted); point 8's encoding and
  point 9's next numbers moved past E09 T58b's (encoding v23, replay format v28, protocol v86),
  merged after the draft; as built with batch GAM-1's outcome at T7.
