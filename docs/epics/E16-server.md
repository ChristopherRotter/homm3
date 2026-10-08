# E16 — Server: protocol, hosting, lobbies and ladders

Status: **draft** (2026-10-08, refinement session; first draft for the user's review, nothing
decided, nothing implemented). ADR drafts: `docs/lanes/server/adr-drafts/protocol.md` and
`identity-and-ladders.md`. Working detail (messages, flows, the ladder sketch):
`docs/epics/E16-design.md`. Lane: server (`docs/lanes/server/HANDOFF.md`).

## Goal

A headless host that lets untrusted clients play the engine's games against each other and
against bots, on servers that anyone can run. It turns connections into trusted actors and
bytes into validated commands, sends each client only what it may see, keeps the authoritative
command log (which is the replay), and runs lobbies and ranked ladders. First a hosted battle on
today's `BattleSetup`; the whole game once `h3-game` exists (E08, E10). The server adds no rules.

## The ideas in plain words

- **Game, seat, viewer.** A *game* is one engine instance the server hosts (a battle now, a
  whole game later). A *seat* is one side or player of it. A connection that holds a seat is
  that seat's *actor*: the engine's `apply` is called with the seat's identity, never with
  anything the client wrote. A *viewer* is a seat or a spectator; everything sent to a viewer
  passes through the engine's `observe` and `project`, which drop what the viewer may not know.
- **Token.** When a connection takes a seat, the server hands it a random secret string. A
  client that loses its connection presents the token and gets its seat back. Tokens are the
  reconnect mechanism, not accounts.
- **Published language.** The messages on the wire are types of their own, documented in
  `h3-protocol`, with one module that translates them to and from the engine's types. The
  engine's internals change often (33 state-hash versions so far); the wire should not. When
  the battle lane adds a command or event the translation does not know yet, the wire shows it
  as "other" instead of failing to build.
- **Sans-I/O.** The protocol crate never touches a socket, a clock or a file: it is given
  bytes and a time and returns bytes. That makes it a logic crate under the project's purity
  checks and the 100 % coverage gate, testable with scripted clients, and deterministic.
- **Manifest, "same content".** At join the server sends what the game is made of: pack ids
  and versions, the quirk profile, the ruleset hash, the engine and protocol versions, and the
  tables that turn the engine's numeric handles into stable keys such as `base:pikeman`. The
  client renders from keys. Because only the server runs rules, a client's version, local
  configuration or mode cannot change the game; a client with matching content may use its
  own rules for previews, any other client asks the server.
- **Replay and record.** The *replay* is the engine's: seed, full setup, command log, hashes;
  it re-runs and verifies, and it holds private inputs, so it is released only after the game.
  The *record* is one viewer's message log: what a viewer client plays back, available any
  time, never re-runnable.
- **Identity, lobby, ladder.** An *identity* is a keypair the client generates once; servers
  know the public key and a display name. A *lobby* lists open games on one server. A *ladder*
  is a fixed game configuration with a rating system and a leaderboard; its *matchmaking*
  queue pairs players of similar rating.

## What the server must provide, seen from the project's goals

- **The AI is the motivator, but it does not train here.** ADR-0011 makes the C ABI the
  training surface; a network protocol for the hot loop was rejected. The server serves the AI
  in three ways only: a trained agent as an opponent for people (a *bot seat*, in-process
  first, an *agent seat* over the wire later), records of human play as data (human data is
  scarce, `docs/07-simulation-and-ai.md` §3), and a public ladder where the AI is a
  participant. So the wire is optimized for correctness, security and simplicity, not for
  throughput.
- **Security becomes real here** (goal 9, ADR-0008). The server is the first untrusted-input
  surface for commands. It must show, end to end and in tests, that identity never comes from
  a payload, that a rejected command changes nothing, that a viewer's bytes depend only on
  what the viewer may know, and that seeds, hashes and tokens never leave.
- **Same content for everyone** (the user, 2026-10-08). Guaranteed by construction, since the
  server is the only engine in a hosted game, and made visible by the manifest.
- **Anyone can host** (the user, 2026-10-08). One binary, a configuration file, a data
  directory, no database, loopback by default, a documented reverse-proxy setup for the
  internet; identities that need no sign-up and no central party.
- **Separation of concerns** (goal 4): an adapter with no rules, no assets, no UI. The
  protocol is the client lane's input and must stay stable while the engine churns.
- **Determinism** (goal 6): the session core is sans-I/O, so a hosted game is a replay and a
  server session is reproducible from its inputs.
- **Quality** (goal 11): the sans-I/O core under the 100 % gate, the security properties as
  tests, an instruction-count workload.

## Decisions proposed

**A. Protocol and hosting** (protocol ADR, points 1–9 and 11; the brief's points 1–4):

- *Transport.* Plain TCP (no browser, own framing), WebSocket (browser-native, framed, one
  small crate with a blocking API), QUIC (heavy, no browser). HTTP for a lobby would add a web
  framework for nothing: lobby and ladder messages ride the same WebSocket. **WebSocket, no
  HTTP, no TLS in the server.**
- *Runtime.* `tokio` (the ecosystem default, a large tree, an async session core) against std
  threads with blocking I/O (no runtime, one thread per connection and per game). The load is
  people and bots, not thousands of connections. **Threads**, the transport behind a trait.
- *Wire format.* JSON (already in the workspace, readable, browser-native) against a binary
  format (`postcard`: compact, opaque, one more dependency) or the sim encodings (for agents,
  lossy for clients, owned by the battle lane). **JSON** with one protocol version; sim
  encodings only for agent seats later.
- *Engine types on the wire.* `serde` derives on the engine's types (quick to build, couples
  the wire to every internal change, no place for "unsupported") against the protocol's own
  types with one mapping module. **The mapping.** Unknown engine commands and events become
  `Unsupported` and `Other { kind }` (the brief's point 3); the wildcard arms are the lane's
  one reviewed coverage exclusion; a test lists the mapped kinds.
- *Sessions and seats.* Seat tokens from OS randomness, reconnect by token, bot seats run by
  the server, spectators on public events only (a spectator observation needs a `Viewer` in the
  battle context, a battle-lane addition, SRV-Q4).
- *Per-viewer delivery.* `observe` and `project` for every outbound message; per-viewer
  sequence numbers; typed acknowledgements.
- *Hidden information* (the brief's point 4, the queue's inbox item). A 64-bit OS seed;
  obstacles from a separate throwaway seed, stored as pieces in the setup (so no obstacle seed
  is ever recorded); no hashes, seeds or tokens on the wire or in the log; the replay only
  after the game. Written into `docs/06-security.md` in M1 (a shared change).
- *Replays per viewer.* Two artifacts, replay and record, as above.

**B. Same content for everyone** (protocol ADR, point 10; the user's first idea):

- The question "what if players have different client versions or settings" has a structural
  answer in this architecture: a client never runs rules in a hosted game (lockstep, where
  every client runs the rules, is excluded by ADR-0004), so there is nothing a client's version
  or local overrides could diverge from. What remains is *compatibility* and *display*: the
  hello refuses a protocol version the server does not speak; the manifest tells the client
  what content it is looking at and gives it the key tables for assets; a client that compiles
  the same packs and profile locally compares its ruleset hash with the manifest's and may
  then run previews (damage ranges, reachable hexes) itself, otherwise it uses the server's
  query messages. Ladders pin a configuration, so a content change on the server starts a new
  season rather than mixing results.
- Later, optional: *content delivery* (the server offers its pack texts for download, so a
  client can display a pack it does not have; still display-only).

**C. Identity, lobbies, ladders** (the identity ADR; the user's second idea):

- *Identity.* Options: password accounts per server (reuse across hosts, hashes everywhere, a
  sign-up per server), bearer tokens per server (dependency-free, a per-server secret, no
  cross-server identity), an external provider (HTTP, TLS, a central party), **a keypair**
  (no secrets on servers, one identity everywhere, no sign-up; one crypto crate, a lost key is
  a lost identity). **Keypair**, behind an `Authenticator` seam; bearer tokens are the
  fallback if the crypto dependency is refused (SRV-Q8).
- *Lobbies.* Open games listed per server; a host picks the game configuration; seats and
  spectators; the configuration fixed at start.
- *Ladders and matchmaking.* A ladder pins a configuration, a scenario pool, Glicko-2 and a
  leaderboard with seasons; one queue per ladder with a widening rating window; results count
  only with a verified replay and bound seats; forfeits on abandonment; bots may be ladder
  participants under a flagged identity.
- *Persistence.* JSON-lines logs in a data directory, no database (SRV-Q9).
- *Internet.* TLS by a reverse proxy; native TLS later behind the transport trait.

## Milestones

Each milestone ends with a fresh reviewer agent and the user's checkpoint; tasks are about half
a context each. The order of M4 and M5 is SRV-Q10.

- **M0 Skeleton** (shared, merged first): the two ADRs accepted and numbered; `h3-protocol`
  (logic, adapter layer: `check-arch` must accept the combination, else a small tool change)
  and `h3-server` in `Cargo.toml` and `arch.toml` with their dependencies (`serde`,
  `serde_json` in `alloc` mode; `tungstenite`, `getrandom`, `toml`); `benches/server/`; the
  versions table gains `PROTOCOL_VERSION` and the record format; `BACKLOG.md` E16 in progress.
  Sequenced with the adventure lane's T2 shared PR (both touch the same files).
- **M1 A hosted battle:** T1 messages and codec; T2 session core; T3 the server binary with a
  headless test client; T4 closure (below).
- **M2 Lobby and many games:** create, list and join games on one connection; the game
  configuration shown before joining; spectators (event-only, or the battle `Viewer` if it
  exists); the replay download; per-address limits and rate limiting hardened; graceful
  shutdown; the fuzz harness on the decoder; the client-side content check documented for the
  client lane.
- **M3 Identity:** the keypair login (challenge, signature, display name), the `Authenticator`
  seam, the player store (JSON lines), seats bound to identities, reconnect by identity, the
  anonymous mode per configuration, the reverse-proxy setup documented.
- **M4 Ranked battles:** ladders with pinned configurations and scenario pools, the
  matchmaking queue, Glicko-2, leaderboards and seasons, forfeits and the grace period, bot
  identities on ladders, the results store; an end-to-end test of two clients queueing and
  playing a rated battle.
- **M5 The whole game** (after E08 increments 1 and 3): players as seats, turn groups, pending
  decisions, the per-player record; the server-side pieces E08 decision 8 lists (trusted
  system commands for disconnects and turn timers, sequence numbers against duplicates,
  concurrent battles); `h3-game` must expose what SRV-Q6 asks.
- **Later:** native TLS, agent seats with the sim encodings for an external policy process,
  content delivery, a leaderboard aggregator over signed results, server discovery.

## M1 tasks

- **T1 Messages and codec.** The wire types (hello and welcome with the manifest, join and
  joined, snapshot, events, your turn with the legal list and castable spells, command,
  accepted, rejected, queries for cast targets, game ended, replay, goodbye, ping and pong),
  the bounded codec, the mapping module (commands, legal commands, every event kind, the
  observation with the public stack values, the own hero's private part), `Unsupported` and
  `Other`, round-trip tests, the "arbitrary bytes never panic" property.
- **T2 Session core.** Seats, tokens as opaque values the host supplies, join and rejoin, the
  authoritative loop (command in, `apply`, `project` for every viewer, acknowledgements), the
  bot hook (the core asks the host for a bot seat's command), the record, the command log with
  periodic hashes for the replay; scripted two-client tests; the observation-hygiene property
  over the wire (a hidden Quicksand placed by the opponent leaves the viewer's bytes
  unchanged); out-of-turn and wrong-seat commands leave state and log unchanged.
- **T3 The server binary.** Threads and channels, the WebSocket transport, a TOML
  configuration (bind address, packs directory, scenario, profile, bot seats, limits), OS seed
  and tokens, the clock and idle timeouts, the random bot from `h3-sim`, the replay written as
  the CLI's TOML at the end; a headless test client in the crate; end-to-end tests over
  loopback: two clients, a client against the bot, a reconnect, malformed and oversized input,
  flooding.
- **T4 Closure.** The `benches/server/` workload (random play through the session core:
  decode, validate, project, encode) with its baseline; the `docs/06-security.md` additions
  (shared PR); the lane HANDOFF and the epic log; the reviewer agent; the checkpoint and the
  cut of M2.

## Exit criteria (M1)

Both ADRs accepted; the full `cargo xtask verify` green in CI with `h3-protocol` under the
100 % gate; the end-to-end tests pass over loopback; the security properties above hold in
tests; the workload has a baseline; `docs/06-security.md` carries the server rules; a replay
written by the server verifies with `h3 replay verify`; the lane HANDOFF, the epic log and
`QUEUE.md` are current.

## Autonomy contract (the HANDOFF's draft, with additions)

- **Agent decides, marked provisional:** crate-internal structure, names, the test layout, log
  format and texts, the thread and channel layout, the headless test client, token encoding,
  file names in the data directory.
- **Queue for the milestone review:** the message schema inside the accepted ADR, the record
  format, the configuration format, a light dependency (`getrandom`, a logging facade), a
  dependency on `h3-cli`'s library for the replay file writer (or a move of the writer into
  `h3-sim`, a battle-lane ask), the coverage exclusion for the wildcard arms, the Glicko-2
  constants and the matchmaking window.
- **Stop the line:** seams and contracts, the engine API the server calls (a spectator viewer,
  `h3-game`'s surface), an async runtime, a TLS stack, the crypto crate, a database or embedded
  store, licence doubts, changes to ADR-0008's model, anything in another lane's paths,
  logic-crate dependencies beyond `serde` and `serde_json`.

## Questions for the user (filed as `SRV-Q1` to `SRV-Q10` in `docs/OPEN-QUESTIONS.md`)

- **SRV-Q1 Where the server runs first.** Loopback on the user's machine, the LAN, or the
  internet. *Recommendation:* loopback and LAN until M3; the internet needs identity (M3) and
  a proxy for TLS.
- **SRV-Q2 Authentication now or later.** *Recommendation:* seat tokens only in M1–M2, the
  keypair identity in M3, so M1 stays small and the identity decision gets its own review.
- **SRV-Q3 Which client connects first.** The headless test client (M1, in the crate), then
  the client lane's viewer. *Proposal to the client lane:* the viewer consumes `h3-protocol`
  messages, live or from a record file, with the same code, so the record format is the
  viewer's file format.
- **SRV-Q4 Spectators.** Public events only, or a `Viewer::Spectator` in the battle's
  `observe` (a battle API change) before M2.
- **SRV-Q5 Runtime and format.** Threads and JSON as recommended, or `tokio` and/or a binary
  format from the start.
- **SRV-Q6 What `h3-game` must expose for M5** (input to the adventure lane's T1a ADR 3):
  `apply` with a player actor, `observe` for a player and a spectator, pending decisions as
  state, a trusted system actor for timeouts and disconnects. May the server lane file this as
  an adventure-lane request?
- **SRV-Q7 Timing.** Start M0 now (the mapping's wildcard protects the build from E04b and
  E04c additions) or wait for E04b's close as the HANDOFF says.
- **SRV-Q8 Identity model.** Keypair (recommended; one crypto crate) or bearer tokens per
  server (no dependency, no cross-server identity).
- **SRV-Q9 Persistence.** JSON-lines logs in a data directory (recommended) or an embedded
  store from the start.
- **SRV-Q10 Milestone order.** Ranked battles (M4) before the whole game (M5), as proposed,
  since a battle ladder with mirrored armies is playable long before the adventure map is.

## Risks

- The mapping drifts behind the engine: mitigated by the "other" fallback, the mapped-kinds
  test and a HANDOFF line; a new battle command stays unplayable over the wire until mapped.
- A logic-gated protocol crate needs `serde_json` in `alloc` mode and no `std` tokens; if
  `check-arch` or the gate fights it, fall back to a plain adapter crate with a high coverage
  target (queue for review).
- Two lanes need shared PRs for new crates at the same time: sequence M0 with E08 T2.
- Ladder abuse (smurfing, collusion, result farming) has no complete technical answer;
  verified replays, seat binding and rate limits cover the cheap attacks; the rest is the
  operator's.
- Information leaks through channels the engine does not model (message timing, sizes):
  bounded by per-viewer sequence numbers and projection; timing stays out of scope for now.
