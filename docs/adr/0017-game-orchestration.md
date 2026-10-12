# ADR-0017: Game orchestration: turn groups, RNG streams per player, pending decisions

- Status: Accepted (2026-10-10)
- Date: 2026-10-09
- Deciders: user + adventure lane (refinement 2026-10-07/08, decisions 4, 8 and 9 of the epic)

<!-- Decisions only. The reasoning is in docs/epics/E08-adventure-map.md. Point 2 answers the
     server lane's SRV-Q6 (what h3-game exposes for the whole game, E16 M5). Longer than 80 lines
     because it covers decisions 4, 8 and 9 and the env at once. -->

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
   RNG streams, the pending decisions and the contexts' states (adventure now; the hero store,
   battles in progress and towns later). A game command wraps a context command (`Adventure(
   MoveHero { .. })`) or is the game's own (`EndTurn`). The game checks turn, phase and pending
   decisions; the context then validates its rules with the player as actor. A command touching
   several contexts validates in all of them before mutating any. Steps only the game triggers
   (start of a day, start of a turn) are infallible functions of the contexts, never commands.
   `GameState` holds each player's **treasury**: an amount per resource kind of the core ruleset
   (dense by `ResourceId`; kinds stay data, ADR-0006), never negative; a gain saturates at
   2³¹ − 1 (the original wraps to 0, `A-5`, a state-corrupting bug fixed without a flag). The
   starting treasury is the row of the setup's difficulty (`GameSetup`), every player the human
   row (E08 decision 16); the rows are data compiled into `GameRuleset`.
2. **Actors and viewers.** `Actor::Player(PlayerId)` for every player command, also a bot's;
   `Actor::System` for the server's trusted session layer only, with its own closed set of
   commands (end a player's turn, answer a pending decision with its default, remove a player)
   that no player can issue. `Viewer::Player(PlayerId)` and `Viewer::Spectator`, which sees
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
   runs at the start of the day; ADV-Q4). Sequential play is one player per group in player
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
   form (a shared change in T2). Where a battle's seed and obstacles come from is decided with
   the battle seam (increment 3), with the server lane, whose draft draws a hosted battle's
   obstacles from a throwaway seed.
5. **Seeds are server-only** (E08 decision 9). The master seed and the stream positions are
   authoritative state: in the state hash (which never reaches a client either), never in an
   observation or event, never seeded from client data; the observation-hygiene and
   ownership-isolation tests cover them. A replay holds the seed and is released only after
   the game.
6. **No blocking prompts: pending decisions.** What needs a player's answer (a level-up, an
   object's question, monsters offering to join, a battle) is a pending decision in the state,
   owned by the players it involves (a battle against another player's hero: both). Its owners
   may answer outside their turn; while a player has one, its only legal commands are the
   decision's answers (in a battle, the battle's commands) and it cannot end its turn; it
   blocks no one else. The engine never waits and never calls back. Increment 2 has two kinds,
   both offers answered yes or no: the Cartographer's (a price, E08 decision 15) and the border
   guard's passage (no price; ADV-Q10, D-7). A player's open decisions form a queue (at most one
   in increment 2); `GameCommand::Answer` answers the first (`Accept` or `Decline`, additive
   variants for later kinds), and a decline only closes it. While one is open, its owner's legal
   commands are its answers only: no move, no `EndTurn`. The system actor's default answer is
   `Decline`. Treasure chests wait for increment 3.
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
8. **`AdventureEnv` lives in adventure-owned sub-paths of the battle lane's crates:**
   `crates/h3-sim/src/adventure/`, `crates/h3-cli/src/adventure/`, `fixtures/replays/adventure/`
   (`lanes.toml`, longest match), until the AI lane takes `h3-sim` at E05. It drives `h3-game`,
   not `h3-adventure`, so it sees turns, days and projections as a player does: reset (a map, a
   seed), step, mask, fork, state hash. Its action is a destination among the reachable tiles of
   a hero (the env builds the path from the core's reachable-tiles query, an ADR-0006
   guardrail) or `EndTurn`, provisional until E13's macro-actions. No C ABI for it in E08 (E05,
   with the batch API). Action encoding v2: the entry head gains the answers after `EndTurn` and
   the eight hero slots (`Accept`, `Decline`; the tile head is ignored for them); while a
   decision is open the mask allows its answers only. The observation (`h3_game`'s view) shows
   its owner the treasury and the open decision (kind, price).
9. **Versions owned by the adventure lane** (T2 adds the rows): the game state hash (`h3-game`),
   the adventure state and ruleset-part hashes (`h3-adventure`), the import version (`h3-map`),
   the adventure replay format and encodings (`h3-sim`'s sub-path, to the AI lane at E05).
   Increment 2 bumps the game state hash to v2 (treasuries, decisions), the action encoding to
   v2 and, if the answer command needs it, the adventure replay format (E08 T14).
10. **Gains and costs cross by outcome.** A context's `apply` returns an outcome: the resources
    its command gains for the actor and the offers it opens, each with its price. The context
    never sees a treasury. The game adds the gains after the context applied (gains cannot fail,
    so atomicity holds). An offer whose price the payer cannot pay is not opened (an event says
    so to the payer); otherwise it becomes a pending decision. Accepting an offer: the context
    checks it still stands, the game checks the price against the treasury, then pays and calls
    the context's infallible acceptance; a rejected answer changes nothing. Towns (E09) use the
    same outcome and the same check.

## Consequences

- The server builds M5 on points 1, 2 and 6 without engine changes beyond the additive variants.
- One player's choices never move another player's dice, in sequential play too: for RL a
  cleaner signal, for simultaneous play the precondition of commuting.
- Every game command needs a turn check and a pending-decision check; `legal_commands` and
  `apply` share them (property-tested). The env and the CLI go through the game like a server.
- Conservation (increment 2): an army changes by no accepted command; a treasury only by a
  visit's gain or an accepted offer's price. Observation hygiene covers treasuries and decisions.

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
- A separate adventure sim crate: a second env stack (batching, replay header) for the AI lane
  to merge at E05.

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
