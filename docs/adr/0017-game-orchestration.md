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
   heroes and battles in progress from increment 3; towns later). A game command wraps a context command (`Adventure(
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
   keeps its details (the pool and the tavern are E09's). The contexts **read a hero's details
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
   which follows the map); a rule the packs cannot express yet (the moat, E09) gains data or a
   configuration input then.
   **The setup** (increment 7): `GameSetup` holds each player's **human flag** (E08 T54: a map's
   import sets it from the map's player kinds, a colour a human may play is human; a session may
   change it before the game is built; an env's agent counts as whatever the setup says), and
   beside the banned skills the scenario's **banned spells and artifacts** (static, from the map),
   which the game passes to the adventure's build.
   **Increment 8** adds no command and no decision: boarding, sailing and landing are steps of
   `MoveHero`'s path, and the sea's pickups give at once. The setup gains the map's **boats**
   (ADR-0016 point 1), which the import fills; a hero placed on water is refused (`Unsupported`,
   D-44). The turn start's points follow the hero's surface (ADR-0016 point 3).
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
   public facts only), and `h3-game` maps onto them.
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
   owner's commands write.
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
   decision 172's `cross-lane` change).
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
    visitor's owner).
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
    `battle_end`.
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
