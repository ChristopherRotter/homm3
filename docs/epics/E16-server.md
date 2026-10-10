# E16 — Server: protocol, hosting, lobbies and ladders

Status: **in progress** (refined 2026-10-08/09; M0 done 2026-10-10: ADR-0018 accepted, the
crate skeletons; M1 next). Decisions: [ADR-0018](../adr/0018-server-and-protocol.md) (protocol
and hosting); ADR drafts `docs/lanes/server/adr-drafts/identity-and-ladders.md` and
`discovery.md`. Working detail (messages, flows, journals, the ladder sketch):
`docs/epics/E16-design.md`. Done tasks in full: `docs/epics/E16-log.md`. Lane: server (`docs/lanes/server/HANDOFF.md`).

**How the drafts become decisions.** Each ADR draft stays "Proposed" until the milestone that
needs it: the protocol ADR was decided at the M0 checkpoint (ADR-0018; nothing could be built
before), the identity and ladders ADR at the M3 refinement. Until then they are direction, and
the parts of M1 and M2 that touch them (the `Authenticator` seam, the journal) are built so that
either option fits. An accepted ADR can still be amended at a later checkpoint, as the battle
lane amends ADR-0014. The user's answers of 2026-10-09 fixed the direction of all three drafts
(below, marked **user**); the protocol draft carried them and was accepted at M0 as ADR-0018,
trimmed to its decisions (the detail is here and in the design note).

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
  engine's internals change often (the state hash version is bumped by most increments); the wire should not. When
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
- **Journal.** The server writes every accepted command to the game's journal on disk before
  it answers the client. Because the engine is deterministic, the journal (ruleset identity,
  setup, seed, commands) *is* the game: after a crash, or when a paused game resumes, the
  server re-runs the journal and has the exact state back. No state serialization exists on
  the server. The journal also carries the live state's hash every N commands, which is how
  corruption is detected (below).
- **Replay and record.** The *replay* is the finished journal: it re-runs and verifies, and
  it holds private inputs, so it is released only after the game. The *record* is one
  viewer's message log: what a viewer client plays back, available any time, never
  re-runnable.
- **Account, lobby, ladder.** An *account* is a name and a password; the client turns them
  into a signing key, so the password never leaves the device and no server stores a secret.
  A *lobby* lists open games on one server. A *ladder* is a fixed game configuration with a
  rating system and a leaderboard; its *matchmaking* queue pairs players of similar rating.
  Servers are independent: a ladder and its accounts belong to one server.

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
  directory of plain files, loopback by default, a documented reverse-proxy setup for the
  internet; accounts that need no sign-up form and no central party; servers independent.
- **Nothing is lost and nothing is counted blindly.** Journals restore games after a crash
  or a pause; every result is verified by a re-run before it counts.
- **Separation of concerns** (goal 4): an adapter with no rules, no assets, no UI. The
  protocol is the client lane's input and must stay stable while the engine churns.
- **Determinism** (goal 6): the session core is sans-I/O, so a hosted game is a replay and a
  server session is reproducible from its inputs.
- **Quality** (goal 11): the sans-I/O core under the 100 % gate, the security properties as
  tests, an instruction-count workload.

## Decisions (directions confirmed by the user, 2026-10-09)

**A. Protocol and hosting** (ADR-0018, points 1–9, 11, 12; the brief's points 1–4):

- *Transport.* Plain TCP (no browser, own framing), WebSocket (browser-native, framed, one
  small crate with a `tokio` binding), QUIC (heavy, no browser). HTTP for a lobby would add a web
  framework for nothing: lobby and ladder messages ride the same WebSocket. **WebSocket, no
  HTTP, no TLS in the server.**
- *Runtime.* `tokio` against std threads with blocking I/O. Measured on scratch crates (SRV-Q5):
  `tokio` and `tokio-tungstenite` add 11 crates to the 36 of the threads variant and no build
  time; a blocking `tungstenite` socket is read and written by one thread only, so a thread
  per connection must poll. **User: `tokio` for the connections** (one task per connection
  over socket, outbound queue and idle timer), **every game on its own OS thread**; the
  session core stays synchronous and runtime-free; no HTTP framework.
- *Wire format.* JSON (already in the workspace, readable, any language) against a binary
  format (`postcard`: about 8–10 times smaller, not self-describing) or the sim encodings (for
  agents, lossy for clients, owned by the battle lane). **User: JSON in serde's default,
  externally tagged enum form** (`postcard` cannot decode internally tagged enums, tested), bulk
  data as packed arrays, one protocol version; a binary codec negotiable later on the same
  types; sim encodings only for agent seats later.
- *Engine types on the wire.* `serde` derives on the engine's types (quick to build, couples
  the wire to every internal change, no place for "unsupported") against the protocol's own
  types with one mapping module. **The mapping.** Unknown engine commands and events become
  `Unsupported` and `Other { kind }` (the brief's point 3); the wildcard arms are the lane's
  one reviewed coverage exclusion; a test lists the mapped kinds.
- *Sessions and seats.* Seat tokens from OS randomness, reconnect by token, bot seats run by
  the server. **User (SRV-Q4): spectators through a spectator viewer the battle lane adds to
  `observe` and `project` before M2**; without it a spectator who joins late or reconnects has
  no correct snapshot.
- *Per-viewer delivery.* `observe` and `project` for every outbound message; per-viewer
  sequence numbers; typed acknowledgements.
- *Hidden information* (the brief's point 4, the queue's inbox item). A 64-bit OS seed;
  obstacles from a separate throwaway seed, stored as pieces in the setup (so no obstacle seed
  is ever recorded); no hashes, seeds or tokens on the wire or in the log; the replay only
  after the game. Written into `docs/06-security.md` in M1 (a shared change).
- *Replays per viewer.* Two artifacts, replay and record, as above.
- *Journal, restore, integrity* (the user's questions on crashes, pausing and bit flips).
  Write-ahead journal per game; restore by re-run after a crash or a pause; a journal whose
  engine or content identity no longer matches is void (operators drain games before an
  upgrade; later, state saves from ADR-0004 point 9 could carry long games across upgrades);
  every result is verified by a re-run against the recorded hashes before it counts; an
  optional shadow instance for immediate detection. What is *not* covered: a corruption of the
  shared ruleset in memory would affect the live game and the re-run alike (the ruleset hash
  is recomputed from memory at game end to catch it), and hardware faults are the operator's
  to prevent (ECC memory).

**B. Same content for everyone** (ADR-0018, point 10; the user's first idea):

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

**C. Accounts, lobbies, ladders** (the identity ADR; the user's second idea and the device
question):

- *Account.* Options: a keypair per device (no password anywhere, but a new device needs the
  key file: the user judged this insufficient), classic password accounts hashed on each
  server (works across devices; the password reaches every server, TLS for every login, each
  host keeps a hash), an external provider (HTTP, TLS, a central party), **a password-derived
  keypair** (name and password become an Ed25519 key on the device through Argon2id; the
  server stores name and public key only; any device with the password has the account; no
  secret is ever sent or stored; public keys are not shown to other clients, so a weak
  password is attackable only with a server's data directory, as with server-side hashes).
  **User (SRV-Q8): the password-derived keypair**, behind an `Authenticator` seam; the identity
  ADR is still decided at M3.
- *Independence.* Servers do not synchronize; a ladder and its accounts belong to one server;
  no global ladder is planned. The same name and password give the same key everywhere, a
  convenience and not a link.
- *Lobbies.* Open games listed per server; a host picks the game configuration; seats and
  spectators; the configuration fixed at start.
- *Ladders and matchmaking.* A ladder pins a configuration, a scenario pool, Glicko-2 and a
  leaderboard with seasons; one queue per ladder with a widening rating window; results count
  only after the journal verified and only with bound seats; forfeits on abandonment; pause
  only by agreement and within a limit; bots may be ladder participants under a flagged
  account.
- *Persistence* (**user, SRV-Q9: plain files**). Files, not a database system: journals for games, append-only JSON-lines
  logs for players and results, each line synced when written, rebuilt at start; an embedded
  store only if a server outgrows memory (SRV-Q9).
- *Internet.* TLS by a reverse proxy; native TLS later behind the transport trait.

**D. Server identity and discovery** (the discovery ADR; the user's third idea):

- *The problem.* Find servers without a registry, and know that the server reached is the one
  described. Options: a central master server (a party to trust and run), DNS records per
  operator (cheap, but browsing still needs a list), a DHT (real decentralized resolution,
  heavy, NAT trouble), **a web of trust between servers** (each server has a keypair, signs a
  card describing itself, and vouches for peers; clients browse from the servers they know and
  see the vouching chain), plus signed community lists and mDNS on the LAN. **User
  (SRV-Q11): the web of trust**, reusing the account crypto for server keys; the DHT stays the
  documented next step if the ecosystem outgrows vouching.
- *What it gives.* Cards cannot be forged or replayed; a server is pinned by its key on first
  contact, as SSH does; browsing reveals the player's address to no unknown server, because
  known servers relay the cards; spam is bounded by vouching. What it does not give: a
  judgement of a server's quality, which stays the player's.

## Milestones

Each milestone ends with a fresh reviewer agent and the user's checkpoint; tasks are about half
a context each. M4 before M5 (the user, SRV-Q10). Where the server runs (SRV-Q1): loopback and
LAN until M3, the internet afterwards behind a TLS proxy; seat tokens only until M3 (SRV-Q2).
M0 starts now, while E04b finishes (SRV-Q7).

- **M0 Skeleton** (done 2026-10-10; `E16-log.md`): ADR-0018; `h3-protocol` and `h3-server`
  declared with the edges and dependencies M1 needs; `PROTOCOL_VERSION`.
- **M1 A hosted battle:** T1 messages and codec; T2 session core; T3 the server binary with a
  headless test client; T4 closure (below). The replay is written at the end of the game; the
  write-ahead journal comes in M2.
- **M2 Lobby, journals and many games:** create, list and join games on one connection; the
  game configuration shown before joining; spectators (the battle's spectator viewer, SRV-Q4);
  the write-ahead journal, restore at start, pause and resume, verification of
  every finished game against its hashes, the identity check against the engine and content,
  the shadow-instance option; the replay download; per-address limits and rate limiting
  hardened; graceful shutdown and a drain mode; the fuzz harness on the decoder; the
  client-side content check documented for the client lane.
- **M3 Accounts:** the identity ADR accepted; the challenge-and-signature login with the
  password-derived key, the `Authenticator` seam, the player log, seats bound to accounts,
  reconnect by account, re-keying, the anonymous mode per configuration, the reverse-proxy
  setup documented.
- **M4 Ranked battles:** ladders with pinned configurations and scenario pools, the
  matchmaking queue, Glicko-2, leaderboards and seasons, forfeits, the grace period and the
  pause limit, bot accounts on ladders, the results log; an end-to-end test of two clients
  queueing and playing a rated battle through a server restart.
- **M5 The whole game** (after E08 increments 1 and 3): players as seats, turn groups, pending
  decisions, the per-player record; the server-side pieces E08 decision 8 lists (trusted
  system commands for disconnects and turn timers, sequence numbers against duplicates,
  concurrent battles); `h3-game`'s surface as the adventure lane's game-orchestration draft gives it (points 1, 2, 6; SRV-Q6); long journals and the case for
  state saves measured.
- **M6 Discovery:** the server keypair and the signed welcome (can come with M3, same crate),
  the server card and `Describe`, peers in the configuration, the peer crawl with its bounds,
  `ListPeers`, signed lists from files, mDNS on the LAN as a feature; a client-side server
  browser is the client lane's (the record of what it needs goes to CLT).
- **Later:** native TLS, agent seats with the sim encodings for an external policy process,
  content delivery, an embedded store if needed, state saves across upgrades once ADR-0004's
  save format exists, address resolution by key through a DHT if vouching no longer scales.

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
  periodic hashes (the journal's content, kept in memory in M1); scripted two-client tests;
  the observation-hygiene property over the wire (a hidden Quicksand placed by the opponent
  leaves the viewer's bytes unchanged); out-of-turn and wrong-seat commands leave state and
  log unchanged.
- **T3 The server binary.** `tokio` connection tasks and an OS thread per game with bounded
  channels, the WebSocket transport, a TOML
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

The full `cargo xtask verify` green in CI with `h3-protocol` under
the 100 % gate; the end-to-end tests pass over loopback; the security properties above hold in
tests; the workload has a baseline; `docs/06-security.md` carries the server rules; a replay
written by the server verifies with `h3 replay verify`; the lane HANDOFF, the epic log and
`QUEUE.md` are current.

## Autonomy contract (confirmed by the user, 2026-10-09)

- **Agent decides, marked provisional:** crate-internal structure, names, the test layout, log
  format and texts, the thread and channel layout, the headless test client, token encoding,
  file names in the data directory, the snapshot cadence of the logs.
- **Queue for the milestone review:** the message schema inside the accepted ADR, the record
  and journal formats, the configuration format, a light dependency (`getrandom`, a logging
  facade), a dependency on `h3-cli`'s library for the replay file writer (or a move of the
  writer into `h3-sim`, a battle-lane ask), the coverage exclusion for the wildcard arms, the
  Glicko-2 constants, the matchmaking window, the checkpoint interval of the journal, the
  default of the shadow-instance option.
- **Stop the line:** seams and contracts, the engine API the server calls (a spectator viewer,
  `h3-game`'s surface), a second runtime, more `tokio` features or an HTTP framework, a TLS
  stack, the crypto crates, a database or
  embedded store, licence doubts, changes to ADR-0008's model, anything in another lane's
  paths, logic-crate dependencies beyond `serde` and `serde_json`.

## Questions (answered 2026-10-09)

The full record is in `docs/OPEN-QUESTIONS.md`, "Answered". In short: SRV-Q1 loopback and LAN
until M3; Q2 seat tokens until M3; Q3 the headless test client, then the client lane's viewer
(the record-file proposal is in the client queue's inbox); Q4 a battle spectator viewer before
M2 (battle queue's inbox); Q5 `tokio` for connections, a thread per game, JSON in the default
enum form; Q6 the `h3-game` needs already covered by the adventure lane's game-orchestration draft; Q7 M0 now;
Q8 the password-derived keypair as direction; Q9 plain files; Q10 M4 before M5; Q11 the web of
trust as direction.

## Risks

- The mapping drifts behind the engine: mitigated by the "other" fallback, the mapped-kinds
  test and a HANDOFF line; a new battle command stays unplayable over the wire until mapped.
- A logic-gated protocol crate needs `serde_json` in `alloc` mode and no `std` tokens: M0
  checked that `check-arch` accepts a logic adapter and that `serde_json` builds with `alloc`
  only; if the coverage gate fights it, fall back to a plain adapter crate (queue for review).
- Two lanes need shared PRs for new crates at the same time: sequence M0 with E08 T2.
- Long adventure games make long journals: restore and verification re-run them; measured in
  M5, state saves are the remedy if it matters.
- An engine upgrade voids open games: a drain mode and operator guidance; state saves later.
- Ladder abuse (smurfing, collusion, result farming) has no complete technical answer;
  verified journals, seat binding and rate limits cover the cheap attacks; the rest is the
  operator's.
- Information leaks through channels the engine does not model (message timing, sizes):
  bounded by per-viewer sequence numbers and projection; timing stays out of scope for now.
