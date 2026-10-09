# ADR-NNNN: Game orchestration: turn groups, RNG streams per player, pending decisions

- Status: Proposed (E08 T1a draft, 2026-10-09; under the user's review)
- Date: 2026-10-09
- Deciders: user + adventure lane (refinement 2026-10-07/08, decisions 4, 8 and 9 of the epic)

<!-- Decisions only. The reasoning is in docs/epics/E08-adventure-map.md. Point 2 answers the
     server lane's SRV-Q6 (what h3-game exposes for the whole game, E16 M5). Open: ADV-Q2. -->

## Context

`h3-game` holds what no domain context may own: players, turns, the calendar, randomness and
the wiring between contexts (ADR draft "crate shape"). Simultaneous turns are an HD/HotA
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
2. **Actors and viewers.** `Actor::Player(PlayerId)` for every player command, also a bot's;
   `Actor::System` for the server's trusted session layer only, with its own closed set of
   commands (end a player's turn, answer a pending decision with its default, remove a player)
   that no player can issue. `Viewer::Player(PlayerId)` and `Viewer::Spectator`, which sees
   public facts only. Increment 1 builds the player variants; the system actor and the spectator
   come with the server's M5 as additive variants.
3. **Turn groups.** A day is a schedule: an ordered list of turn groups, each a set of players
   who act together. A group's turn starts with each context's start-of-turn step for its
   players and ends when each of them has ended its turn or left the game. After the last group
   the day advances: the calendar, then each context's start-of-day step, in a fixed order of
   contexts and players. Which rule runs at which hook is a rule question (T1b). Sequential play
   is one player per group in player order, the only schedule of increment 1. Commands
   interleave in one authoritative order, the server's log, which is the replay; logic stays
   single-threaded per game (ADR-0003).
4. **An RNG stream per player** (kernel streams, ADR-0003 point 7). Randomness caused by a
   player's command draws only from that player's stream; the day advance draws from a game
   stream, and the random placeholders of a setup are resolved from a setup stream when the
   state is built. A battle (increment 3) draws its setup randomness (obstacles) and its own seed
   from the stream of the player whose command started it. The kernel's stream ids gain the per-player form (a shared change).
5. **Seeds are server-only** (E08 decision 9). The master seed and the stream positions are
   authoritative state: in the state hash (which never reaches a client either), never in an
   observation or event, never seeded from client data; the observation-hygiene and ownership-isolation tests cover them. A replay
   holds the seed and is released only after the game.
6. **No blocking prompts: pending decisions.** What needs a player's answer (a level-up, an
   object's question, monsters offering to join, a battle) is a pending decision in the state,
   owned by the players it involves (a battle: both). While a player has one, its only legal
   commands are the decision's answers (in a battle, the battle's commands) and it cannot end
   its turn; other players are not affected. The engine never waits and never calls back. The
   first kind comes with increment 2 (treasure chests).
7. **Simultaneous play is designed for, not built.** Built now: turn groups, streams per player,
   pending decisions. **Not decided** (deferred to the increment that builds it): the target
   that commands of the players of one group commute (any interleaving, the same state hash; a
   property test), which needs a contact rule that ends simultaneous play for players who could
   interact (HD's and HotA's surveyed in T1b) and shared state (the tavern pool, global
   counters) partitioned or accepted as order-dependent; under it, a command that conflicts with
   the authoritative order is rejected and leaves the state untouched. Until then, each piece of
   state that commands of two players can both write is listed in the epic file when it is
   added: the inventory for that decision.
8. **`AdventureEnv` lives in adventure-owned sub-paths of the battle lane's crates:**
   `crates/h3-sim/src/adventure/`, `crates/h3-cli/src/adventure/`, `fixtures/replays/adventure/`
   (`lanes.toml`, longest match), until the AI lane takes `h3-sim` at E05. It drives `h3-game`,
   not `h3-adventure`, so it sees turns, days and projections as a player does: reset (a map, a
   seed), step, mask, fork, state hash. Its action is a destination among the reachable tiles of
   a hero (the env builds the path from the core's reachable-tiles query, an ADR-0006
   guardrail) or `EndTurn`, provisional until E13's macro-actions. No C ABI for it in E08 (E05,
   with the batch API).
9. **Versions owned by the adventure lane** (T2 adds the rows): the game state hash (`h3-game`),
   the adventure state and ruleset-part hashes (`h3-adventure`), the import version (`h3-map`),
   the adventure replay format and encodings (`h3-sim`'s sub-path, to the AI lane at E05).

## Consequences

- The server builds M5 on points 1, 2 and 6 without engine changes beyond the additive variants.
- One player's choices never move another player's dice, in sequential play too: for RL a
  cleaner signal, for simultaneous play the precondition of commuting.
- Every game command needs a turn check and a pending-decision check; `legal_commands` and
  `apply` share them (property-tested). The env and the CLI go through the game like a server.

## Alternatives considered

- One global stream: a command of one player shifts every later roll of the others;
  simultaneous commands could never commute.
- Modal prompts as callbacks or blocking calls: a prompt would hold the whole game, and a
  state waiting inside a call cannot be forked.
- An env on `h3-adventure` directly: no turns, no day, and a second path beside the game's.
- A separate adventure sim crate: a second env stack (batching, replay header) for the AI lane
  to merge at E05.
