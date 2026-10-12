# E16 — Server: protocol, hosting, lobbies and ladders

Status: **in progress** (refined 2026-10-08/09; M0 done 2026-10-10: ADR-0018 accepted, the
crate skeletons; M1 done 2026-10-10, its checkpoint passed 2026-10-11 (SRV-Q12); M2 done
2026-10-11 (T1 to T5e), its checkpoint passed the same day (SRV-Q16); M3 done 2026-10-12 (T1 to
T5), its checkpoint passed the same day (SRV-Q18); M4 refined 2026-10-12 (SRV-Q19), T1 next). Decisions: [ADR-0018](../adr/0018-server-and-protocol.md) (protocol
and hosting); [ADR-0019](../adr/0019-identity-and-ladders.md) (identity and ladders, accepted at the M3
refinement, its ladder points decided at M4's); ADR draft `discovery.md`. Working detail (messages, flows, journals, the ladder sketch):
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
  engine's internals change often (the state hash version is bumped by most increments); the
  wire should not. The translation matches the engine's enums exhaustively: when the battle lane
  adds a command or event, the build fails until the translation knows it (the user, 2026-10-10).
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
  the wire to every internal change) against the protocol's own
  types with one mapping module. **The mapping** (user, 2026-10-10; ADR-0018 point 5 amended):
  exhaustive matches over the engine's enums, so a new engine variant fails the build until it
  is mapped. The first draft's wildcard arms to `Unsupported` and `Other { kind }` (the brief's
  point 3) could never run while every variant is mapped and needed a coverage exclusion. A
  test lists the mapped kinds.
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
  queueing and playing a rated battle through a server restart. Refined 2026-10-12 (SRV-Q19):
  no turn timer (a game rule, M5).
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

- **T1 Messages and codec** (done 2026-10-10; `E16-log.md`): `wire`, `codec` and `map` in
  `h3-protocol`, 100 % covered with no exclusion; ADR-0018 point 5 amended.
- **T2 Session core** (done 2026-10-10; `E16-log.md`): `connection` (the hello gate) and
  `session` (seats, tokens, the loop, the bot hook, records, the journal) in `h3-protocol`, the
  hygiene and rejection properties over the wire; `PROTOCOL_VERSION` 2.
- **T3 The server binary** (done 2026-10-10; `E16-log.md`): `h3-server` hosts the configured
  battle (TOML configuration, OS seeds and tokens, a thread for the game, a `tokio` task per
  connection behind the transport trait, the random bot, the replay file); end-to-end tests over
  loopback with the headless client; the replays it writes verify with `h3 verify`.
- **T4 Closure** (done 2026-10-10; `E16-log.md`): the lane-only workload
  `session-random-play-full` (scripted clients through the session core, 273 M instructions);
  `docs/06-security.md`'s "Hosted games"; the reviewer agent; the proposed cut of M2
  (`docs/lanes/server/QUEUE.md`), decided at the checkpoint.

## M2 tasks

Cut at the M1 checkpoint (SRV-Q12); the tasks in full: `docs/lanes/server/QUEUE.md`.

- **T1 Many games on one server** (done 2026-10-11; `E16-log.md`): the lobby (`ListGames`,
  `CreateGame` from the server's offers; wire version 4), a sans-I/O `lobby` in `h3-protocol`,
  the sectioned configuration with `[[offer]]`s, a registry and a thread per game in `h3-server`.
- **T2 The write-ahead journal and restore** (done 2026-10-11; `E16-log.md`): the session
  core's journal entries and `Session::restore`; a journal per game in `<data>/games/`, synced
  before the frames that depend on it; restore at start, void journals moved aside; the replay
  as the finished journal.
- **T3 Verification, pause and resume** (done 2026-10-11; `E16-log.md`): a finished game's journal
  read back and re-run before its replay is published (void otherwise, `Unverified`), the ruleset
  hash recomputed from memory; the shadow-instance option; `Pause` by the host, a paused game as
  its journal, resumed by a seat's join with its token (wire version 5, journal format 2).
- **T4 Spectators** (done 2026-10-11; `E16-log.md`): `Join` as a spectator, up to
  `[limits] spectators_per_game`, with the battle's spectator viewer (`observe_as`, `project_as`):
  its snapshot, the public events, its own numbers and record; the observation's `spectator` flag
  (wire version 6).
- **T4b The observation's completeness on the wire** (done 2026-10-11; `E16-log.md`): the
  fields E04b T9a added (the wait phase, the per-stack clone links, raised creatures, corpses,
  binders, poison and start hit points, the summoned creature and the cost change per side, the
  cleared hexes, the own hero's hidden magic), the effects' values; every field of the engine's
  observation and its parts named in a pattern, so a new one fails the build until it is mapped or
  left out with its reason (wire version 7).
- **T4c The host's spectator setting** (done 2026-10-11; `E16-log.md`; the user asked for it):
  `CreateGame`'s `spectators` (open when left out), the host's `SetSpectators` until the start
  (closing ends the watch), `GameInfo::spectators`, the setting journaled (wire version 8,
  journal format 3).
- **T5a Shutdown, drain, the deadlines of abandoned games** (done 2026-10-11; `E16-log.md`): the
  first signal drains (no new game, a waiting one ends, the server stops once none waits or
  runs), a second stops at once, the journals kept; a running game whose seats all stay empty
  pauses, a paused one nobody resumes is void (`[limits] empty_game_ms`, `paused_game_ms`);
  `tokio`'s `signal` (ADR-0018 point 3 amended); wire version 9 (`Refused { Draining }`).
- **T5b The connection's bounds** (done 2026-10-11; `E16-log.md`): a rate per address across its
  connections (`[limits] messages_per_address`), WebSocket pings counted, a deadline for the hello
  and one for being in a game (`hello_timeout_ms`, `lobby_timeout_ms`), a refused join leaves the
  game (`Session::holds`); no wire change.
- **T5c Memory bounded in bytes** (done 2026-10-11; `E16-log.md`): a viewer's record keeps its
  latest messages within `[limits] record_bytes` (the header's `first`), the outbound queue counts
  its bytes (`outbound_bytes`); wire version 10, record format 2.
- **T5d The decoder's mutation tests, the public archive of finished games** (done 2026-10-11;
  `E16-log.md`; SRV-Q14, SRV-Q15): mutated valid messages never make the decoders or a running
  session panic; any client lists finished games' replays and downloads one (`ListReplays`,
  `FetchArchived`), a game closed to spectators archived unlisted, `[limits] replays_kept`; wire
  version 11; ADR-0018 points 8 and 9 amended.
- **T5e M2's closure** (done 2026-10-11; `E16-log.md`): the exit criteria below; the client
  lane's note (the content check, a viewer after a restart); the inbox placed; the reviewer agent
  (1 major, 8 minor): no file I/O under the registry's lock, a new or resumed game's own join
  first, a refused join keeps the lobby's deadline, a paused game's deadline across restarts, no
  journal text in the log, the replay written whole, a panicking game thread leaves the lobby; the
  rest in the inbox.

## M3 tasks

Cut at the M3 refinement (2026-10-11, SRV-Q17); the tasks in full: `docs/lanes/server/QUEUE.md`.
ADR-0019 (accepted at the refinement; points 1 and 2): a permanent
login name that salts the key and a changeable display name, the server's id in the signed
login, `ed25519-dalek` and `argon2`, anonymous play by configuration, any seated account opting
out of its replay's listing.

- **T1 The login on the wire, sans-I/O** (done 2026-10-11; `E16-log.md`): `account` (the names'
  rules, hex, the signed bytes), `Login`, `Rekey`, `Rename`, `SetListed` answered `Account`, the
  welcome's id and challenge, the hello gate's login step; wire version 12; the server refuses
  every login until T2a (SRV-STUB-1).
- **T2a The Authenticator and the player log** (done 2026-10-11; `E16-log.md`): signatures
  checked (Ed25519, strict), registration and login, `<data>/players.log`, the derivation and its
  test vector in the test client, `[server] public_url` and `anonymous`, failed logins limited
  per address; re-keys, renames and the preference refused until T2b (SRV-STUB-2).
- **T2b Re-keying, renames, the listing preference, the operator's rebind** (done 2026-10-11;
  `E16-log.md`): the three requests answered, each a line in the player log; `h3-server rebind`
  while the server is stopped, `<data>/lock` keeping one server or command at a time.
- **T3 Seats bound to accounts** (done 2026-10-11; `E16-log.md`): a seat a logged-in connection
  takes is its account's, re-taken by it alone without a token (`session::refusal`), also paused
  and after a restart (journal format 4); the lobby's display names (`GameInfo::seated`); the
  replay listed by the seated accounts' preferences; a re-key closes the account's other
  connections (`Rekeyed`); wire version 13.
- **T4a A game's bounds** (done 2026-10-11; `E16-log.md`): `[limits] commands_per_game` (a game
  reaching it without an end void, `Goodbye { CommandLimit }`; its replay sized to fit
  `outbound_bytes`); a waiting game without a held seat leaves, its spectators told
  (`Abandoned`).
- **T4b Behind a reverse proxy** (done 2026-10-11; `E16-log.md`): `[server] trusted_proxies`
  (the client the last `X-Forwarded-For` entry names), every limit per address by IPv6 /64, the
  operator's setup with TLS (`crates/h3-server/README.md`).
- **T5 Closure** (done 2026-10-12, the checkpoint passed the same day, SRV-Q18; `E16-log.md`):
  `06-security.md`'s accounts, ADR-0018 points 6 and 8 amended, the client lane's note, the exit
  criteria checked, the reviewer agent (run before #105's pull request).

## M4 tasks

Cut at the M4 refinement (2026-10-12, SRV-Q19); the tasks in full: `docs/lanes/server/QUEUE.md`,
the working detail: `E16-design.md` §11. ADR-0019 points 4–9 (decided at the refinement): a
season follows the ladder's ruleset (ratings carried, deviations reset); one queue at a time;
the ratings written to the ladder's results log, never recomputed; a disconnect's grace and a
total per player and game (5 and 10 minutes by default), beyond either a forfeit; one unfinished
ranked game per player per ladder, bots exempt; no turn timer in M4 (a game rule, `h3-game`).

- **T1 Ladders, ratings and the results log** (ready): `[[ladder]]`, Glicko-2 with Glickman's
  example as its test vector, `<data>/ladders/<id>.log` read back at start, seasons by the
  ruleset hash; no wire change.
- **T2 The queue and the match** (after T1): `h3_protocol::ladder` (the queue, the matcher, the
  accept step, one queue per account, one unfinished ranked game per ladder), the messages, the
  matched game created with both seats bound to their accounts; wire 14.
- **T3a A ranked game's seats and forfeits** (after T2): no host; the disconnect grace and
  total, forfeits and no-shows in the journal (format 5), across a restart.
- **T3b The rated end** (after T3a): verification before `GameEnded`, the result rated and
  logged, `Rating` to the seats, a void game's players told, game ids unique for the server's
  life.
- **T4 Leaderboards** (after T3b): pages by season, one's own standing.
- **T5 Bots on ladders** (after T3b): bot accounts, the standing queue entry, flagged rows.
- **T6 The pause by agreement** (after T3a): proposed and answered within the ladder's bounds,
  the due time, the forfeit after it; one ask per seat until the game moves on.
- **T7 Closure** (after T1–T6): the end-to-end test through a restart, `06-security.md`'s
  ladders, the client lane's note, the reviewer agent, the user's checkpoint.

## Exit criteria (M4)

The full `cargo xtask verify` green in CI with `h3-protocol` under the 100 % gate, and shown in
tests over loopback: two clients queue, accept and play a rated battle through a server
restart, their ratings as Glicko-2's test vector computes them and the same after another
restart; a declined or expired match puts the other player back in the queue; a second queue,
or a second ranked game on the same ladder, is refused; an anonymous connection never queues;
a player away within the grace re-takes the seat by account, beyond the grace or the total
forfeits; both away: void, no rating change; a ranked game that does not verify is unrated and
its players are told; a content change starts a season with the deviations reset; the
leaderboard pages by season; a bot account plays a rated game, flagged; a pause by agreement
within the ladder's bounds, and a forfeit after its due time. And: `06-security.md` carries the
ladders; the client lane's inbox has the ladder's messages; a fresh reviewer agent's major
findings are fixed; the lane HANDOFF, the epic log and `QUEUE.md` are current.

## Exit criteria (M3)

The full `cargo xtask verify` green in CI with `h3-protocol` under the 100 % gate, and shown in
tests over loopback: a client registers and logs in with the key its login name and password
derive (a test vector); a login signed for another server's id, with another connection's
challenge or by another key is refused; a seat taken by an account is re-taken by a new
connection logged in as it, without a token, also after a restart; a re-keyed account logs in
with its new key only; a display name changes and stays unique; anonymous play as configured;
a game beyond its command limit is void; a waiting game without a held seat leaves; behind a
trusted proxy the limits count the client's address. And: `06-security.md` carries the
accounts; the reverse-proxy setup is documented; the client lane's inbox has the login; a fresh
reviewer agent's major findings are fixed; the lane HANDOFF, the epic log and `QUEUE.md` are
current.

## Exit criteria (M2)

The full `cargo xtask verify` green in CI with `h3-protocol` under the 100 % gate, and shown in
tests, over loopback where the server is involved:

- many games on one server through the lobby: listed with their offers before joining, created
  only from the server's offers, joined and played at once, within the limits per address and in
  total;
- the write-ahead journal: a game stopped mid-battle restores at the next start to what its
  players were told and plays on to a replay that verifies; a journal of other content, of a
  missing offer or that does not re-run restores nothing and is kept aside, void;
- every finished game re-run from its journal on disk before its replay is published, a game that
  does not verify void (`Unverified`); the shadow instance finds and repairs a divergence before
  anything is sent;
- the host's pause, the resume by a seat's token (also after a restart), the deadlines that pause
  an abandoned running game and void an abandoned paused one;
- spectators through the battle's spectator viewer, within the host's setting and the server's
  limit, seeing nothing a seat's viewer hides;
- a first signal drains and a second stops, the journals kept;
- the connection's bounds: the frame size, the rates per connection and per address with the
  WebSocket pings, the connections, the deadlines for the hello, the lobby and idling, the
  outbound queue and the record in bytes;
- the archive: finished, verified games listed and downloaded without a game, by ids it names,
  unlisted when closed to spectators, at most `replays_kept`, read back at a start;
- M1's security properties (identity, rejection, projection, secrets) held over the lobby,
  restored games and spectators; the decoders and a running session never panic on mutated valid
  messages.

And: `docs/06-security.md` carries M2's guarantees and names what is not covered yet; the
workload has a current baseline; the client lane's inbox has the content check and what a
restart does to a viewer's numbers and record; the inbox's M2 items are decided; a fresh reviewer
agent's major findings are fixed or recorded with the user's decision; the lane HANDOFF, the epic
log and `QUEUE.md` are current.

## Review list for the M4 checkpoint

Choices queued under the autonomy contract (details in `E16-log.md`): none yet.

## Review list for the M3 checkpoint (passed 2026-10-12, SRV-Q18)

Choices queued under the autonomy contract (details in `E16-log.md`):

- T1: the messages (`Login { seq, login, key, signature, display }`, a registration by its
  `display`; `Rekey { seq, key, by_old, by_new }`; `Rename`; `SetListed`; one answer `Account
  { seq, display, listed }`; `Refused` with `BadLogin`, `BadName`, `NameTaken`, `NotLoggedIn`,
  `AlreadyLoggedIn`, the connection kept open); the welcome's `id` and `challenge`; keys and
  signatures as lowercase hex; the signed bytes (a purpose with its version, then the id, the
  challenge, the login name and the display name, texts after their 4-byte length; a re-key's
  over the new key); a login name only in lowercase; display names of 3–24 ASCII letters,
  digits, `_ - .` and single inner spaces (no lookalike letters on leaderboards); without
  anonymous play only `Join` and `CreateGame` wait for the login (the lists, the archive and
  queries do not); the server's id the address it listens on until T2a's `public_url`.
- T2a: the `Authenticator` as the signature check alone (`verify(key, message, signature)`),
  the rules in `accounts`; Ed25519's strict check; the player log's line (`Registered { login,
  key, display }`) and its reading (a torn last line cut, any other bad line stops the start;
  after a failed write registrations are refused); a display name sent with a known account's
  login ignored; an unknown login name without a display name `BadLogin`; `[server] public_url`
  (`ws://` or `wss://`, at most 200 visible characters after it) and `anonymous`; `[limits]
  failed_logins_per_address` (10 a minute, kept beyond the address's connections; beyond it a
  login closes its connection `RateLimited`, the address's other requests pass); a login answered
  off the connection's task (`spawn_blocking`, the player log under its own lock); `argon2` a
  normal dependency of `h3-server` for its `client` module (tools may log in); the shared lines
  for the two crates (`Cargo.toml`, `arch.toml`) in a commit of their own on the lane's branch.
- T2b: the player log's lines as typed events (`Rekeyed`, `Rebound` apart from `Rekeyed` for the
  operator's audit, `Renamed`, `Listed`), checked against the accounts on reading and before
  writing; a request that changes nothing answered without a line; a re-key's connections logged
  in already stay so; a failed re-key not counted as a failed login; `h3-server rebind
  <config.toml> <login> <public-key>` (the key checked as an Ed25519 key that is not weak);
  `<data>/lock` (`File::try_lock`), held by a running server for its life and by the command, so
  a second server on the same data directory does not start either.
- T3: an account's seat taken by no token (`SeatTaken`, no new refusal) and its `Joined` without
  one; the seat's account fixed at its first take (a login after opening the game does not bind
  it); one account in both seats allowed; `Entry::TakenBy` beside `Entry::Taken`, the journal's
  `Taken { side, account }`; `GameInfo::seated` (`Seated { seat, display }`, anonymous and bot
  seats unnamed), its names as they stand at the seats' last change (a rename shows later); an
  account the player log does not know keeps a replay unlisted; the replay's listing decided when
  the game ends; `GoodbyeReason::Rekeyed` and the connections by account (`Service::sessions`).
- T4a: `commands_per_game` (10 000) and the replay's size estimate (160 bytes a command, 64 KiB
  besides) the configuration checks against `outbound_bytes`; the limit checked by the game's
  thread after each step (a few bot commands past it possible); void at the limit (no draw, no
  replay), `Goodbye { CommandLimit }` and `Abandoned`; a waiting game without a held seat leaves at
  once, as one without connections did.
- T4b: `trusted_proxies` as exact addresses (no ranges); `X-Forwarded-For` alone, its last entry
  (not RFC 7239's `Forwarded`); a proxied connection counted in the total only until its
  handshake; the proxy's own address without a header that reads; the /64 for every limit per
  address, the lobby's included; the operator's page as the crate's README.
- M3's reviewer (run before T5): `account_changes_per_address` (20 an hour; every request that
  may write the player log, closing its connection beyond); the re-key's kick taken before a
  client message and only for another key; a refused line truncated off the log; a warning
  without `public_url` on an unspecified address or behind proxies.

## Review list for the M2 checkpoint (passed 2026-10-11, SRV-Q16)

Choices queued under the autonomy contract (details in `E16-log.md`):

- T1: the lobby's messages (`CreateGame` names an offer and the creator's seat and joins it at
  once, so a game never waits without a connection; no `StartGame`: a game starts when every
  seat is held; `Games` lists the offers with the games); the refusals `NoSuchOffer` and
  `TooManyGames` (also for a creation the OS randomness failed), `SeatTaken` for the bot's
  seat; the limits counted per address and over games that are not over; the configuration's
  sections; one connection in one game at a time (joining another leaves the first, its seat
  waiting for its token); a game leaves when it is not running and no connection is bound to
  it (a waiting game whose creator leaves is gone, a running one waits for its players); the
  replay file named by the offer.
- T2: the entries (`Taken`, `Started`, `Command`, `Checkpoint`) and the restore's rules (the
  journal must begin the re-run's entries; a journal cut short is completed, the bot acting if it
  is its turn; a viewer's numbers and record start afresh); `JOURNAL_FORMAT_VERSION` 1 and the
  file's layout (JSON lines; the first line with the creator's address, the tokens and the
  replay before its first command as the CLI's TOML); `serde_json` in `h3-server` (already in the
  workspace); a journal that does not write stops its game with `Goodbye { Shutdown }`; the void
  directory; owner-only directories and journals on Unix; ids of ended games reusable after a
  restart.
- T3: the verification reads the journal back from disk (not the session's copy) and publishes
  the replay it re-ran; a game that does not verify is void: `Rejected { Unverified }` for
  `FetchReplay`, the journal in `<data>/void/` and the replay as played beside it, the diagnosis
  (with hashes) as its first line, the log line without hashes; `GameEnded` is still sent before
  the check. Who pauses: the host (the seat taken first, a created game's creator) alone; another
  seat's ask goes to every viewer as `PauseAsked` (the design note's "or all seats" adds nothing in
  a two-seat battle, the host being one of them). No `Resume` message: a seat's `Join` with its
  token resumes (the registry checks the token before it restores the journal); no `Paused` or
  `Resumed` message: `Goodbye { Paused }` closes every connection of the game. The wire's
  additions (version 5: `Pause`, `PauseAsked`, `Phase::Paused`, `GoodbyeReason::Paused`,
  `RejectCode::Unverified`) and the journal's lines `Paused`, `Resumed` (format 2; a format-1
  journal is void after the upgrade). A paused game counts against the limits like a running one;
  its deadline (`pause_hours`) moves to T5 with the deadline of a game nobody returns to. The
  shadow instance's repair is silent for the players (the command is re-applied to the rebuilt
  battle before anything is sent), a log line names the step.
- T4: a spectator is a viewer per connection, numbered from its join, its record since then;
  nothing of it is journaled (after a drop or a restart it joins again); its messages for a seat
  are rejected `NotSeated` *numbered* (it is a viewer, unlike a connection in no game); it gets
  `PauseAsked` and `Goodbye { Paused }` like the seats, and a paused game takes no spectator (the
  registry refuses before it restores); the limit `[limits] spectators_per_game` (8, 0 for none,
  at most 1024) refused as `NoSpectators` (its meaning widened: no more spectators); spectators
  count against the connection limits only. The wire: `Observation::spectator` (version 6).
- T4b: the new fields' shapes (per-stack values on the wire's `Stack`, per-side ones on
  `SideView`; binders as the binding stacks' ids, spell sets as handles, rates as basis points);
  left out on purpose: a rate's single-precision value (quirk Q-0005), the stacks' base attack,
  defense and speed (their values now and the effects' shares are on the wire) and the values
  materialised from the effects (protections, status, …; the effects carry their spells,
  masteries and values); the completeness as patterns in the mapping, a build failure like a new
  variant's (the queue asked for a test); `h3-contracts` a normal dependency of `h3-protocol`
  (an edge `arch.toml` allowed already).
- T4c: the messages (`CreateGame`'s optional `spectators`; `SetSpectators { seq, open }` answered
  `SpectatorsSet { n, seq, open }`, numbered to the host's seat; the reject codes `NotHost` and
  `AlreadyStarted`; `Goodbye { SpectatorsClosed }` for the spectators a closing ends;
  `GameInfo::spectators`, false for a paused game and on a server without spectators); the
  setting journaled once, as the game started (a `Spectators` line before `Started`, format 3); the
  other seat is not told of a change (in a two-seat battle it can join only a waiting game, which
  starts then; the lobby showed it the setting). Not built: an offer's own setting for the
  ladders' games, which have no host (M4).
- T5a: the signals (the first SIGINT or SIGTERM drains, a second stops; no drain without a
  signal, no other trigger); a drain ends waiting games (`Goodbye { Shutdown }`, their journals
  removed) and lets paused ones resume; `Refusal::Draining`; a stopped game's journal stays (a
  waiting one's goes), its connections hear `Goodbye { Shutdown }`; the deadlines in milliseconds
  like `idle_timeout_ms` (defaults 30 minutes and 72 hours, 0 for none, at most a year), the
  sweep every quarter of the shortest (10 ms to a minute); an abandoned running game pauses
  as if its host paused it (its spectators hear `Goodbye { Paused }`), an abandoned paused one
  goes to `<data>/void/` with the reason; after a restart a paused game's clock counts from its
  journal's last write.
- T5b: the address's rate (40 a second by default, twice a connection's) closes the connection
  whose message passes it (`RateLimited`), and starts afresh once the address has no connection;
  every WebSocket control frame counts (a ping, an unasked pong), none keeps the connection from
  idling; the two deadlines (10 s for the hello from the handshake, 60 s in no game from the hello
  or from leaving a game) say `Goodbye { IdleTimeout }` (no new reason, no wire change); a game's
  answer marks the connection's message after which it holds nothing (`Frame::left`), and the task
  leaves only if it passed the game no later message (a pipelined second join may take a seat).
  Not built: a separate rate for automated seats (T3's note): an operator cannot know a client is
  an agent before accounts, so it waits for M3's bot accounts; until then an operator raises
  `messages_per_second` and `messages_per_address`.
- T5c: the record as a window of the latest messages (the oldest go, the newest always stays,
  numbering goes on; `RecordHeader::first`, format 2, wire version 10) rather than refusing
  what would grow it; the bounds 512 KiB per record and 2 MiB per outbound queue, a record to fit
  twice in a queue (its frame escapes its text once more); the bytes counted by the game as it
  queues and by the task as it takes out, a goodbye queued whatever the bytes. Not built as its
  own rule: one `FetchRecord` in flight. The queue's bytes bound what queued records hold, and a
  client that asks for them faster than it reads is dropped as `TooSlow`.
- T5d: the mutations (a flipped byte, a cut, an insertion, a repeat, a splice of two messages, a
  number or a string replaced by a plausible or an extreme one) as proptests in `h3-protocol`,
  no fuzzing dependency (SRV-Q14). The archive's messages (`ListReplays { seq, from }` answered
  `Replays { seq, total, replays }`, 50 a page, newest first; `FetchArchived { seq, replay }`
  answered `Replay` numbered 0 or `Refused { NoSuchReplay }`), served by the connection's task
  without a game; the replay's id is its file's stem (offer, game, a random suffix: unique across
  restarts); the opt-out is the host's spectator setting as the game started, whatever the
  server's spectator limit (an unlisted replay in `<data>/replays/closed/`); `replays_kept` (10
  000 in each part), the oldest file deleted, also at a start with a lower bound; the order the
  files' last writes.
- T5e: the inbox's placement (`QUEUE.md`: the commands-per-game limit in M3, before the internet,
  rather than now; ids and the verification before `GameEnded` in M4; a viewer's record across a
  restart documented for the client lane, not kept). The review's fixes: a game prepares on its
  own thread (`game::launch`), so it is listed before its journal exists; a creation whose secrets
  or journal fail, and a resume whose journal no longer reads, end with `Goodbye { Shutdown }` (no
  new wire reason); `Frame::held` for the lobby's deadline; `.part` files for replays. Left for
  the checkpoint (inbox): a waiting game's deadline, the archived replay's bytes, repeated
  `PauseAsked`, the per-address limits behind a proxy, lost `Drain` and `Tick`. **For the user:** a
  finished game's replay is public unless its host closed it to spectators; the other seat has no
  say (M3's accounts could give it one).

## Review list for the M1 checkpoint (passed 2026-10-11)

Choices queued under the autonomy contract (details in `E16-log.md`):

- M0: `futures-util` for the socket's `StreamExt`/`SinkExt`; `h3-server` → `h3-cli` for the
  pack, scenario and replay-file functions (provisional; the alternative is a battle-lane move
  into `h3-sim`).
- T1: the message schema in `wire.rs`, with three additions to the design note: `Refused` for
  a refused join (the connection stays open), the session's reject codes `NotSeated` and
  `NotStarted`, and the manifest in `Joined` (ADR-0018 point 10), not in `Welcome`; the
  observation as the sim encoding's values in named fields (the own hero's spell bonuses are
  not on the wire yet); hexes as indices, the field's size in the manifest; flag sets as named
  booleans; `DEFAULT_MAX_CLIENT_FRAME` 4096 bytes; `postcard` as a dev-dependency for the
  round trips.
- T2: the wire amended (`PROTOCOL_VERSION` 2): `seq` in `FetchReplay`, `FetchRecord`, `Replay`
  and `Record`, the reject code `NotOver`; the record's format (a `RecordHeader` line, then the
  viewer's numbered frames; `Replay` and `Record` take no number); the start rule (every seat held
  at once; two bot seats never start); a seat taken without a token only the first time; an
  `Events` message for every accepted command, even an empty one; the bot hook as a trait object
  given the state, a rejected choice replaced by the first legal command.
- T3: the configuration's format and defaults (`h3-server/src/config.rs`); the thread and
  channel layout (a game inbox of 1 024, `try_send` into per-connection queues); idle meaning no
  message from the client; the lingering close; the replay file's name; `getrandom` 0.3 beside
  the 0.4 that `tungstenite` brings (a `cargo deny` warning).
- T4: the workload's shape (`serve_frame` counted, scripted clients that strike when they can,
  20 games of `full`); **lane-only workloads** (a shared `xtask` change: the server's count is
  55 % battle engine and follows the battle lane's packs, so the full verify does not measure
  it; the server lane re-baselines a drift at its sessions' start); the shared changes ride
  this branch's pull request with the label `cross-lane` (the user, 2026-10-10).

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
  writer into `h3-sim`, a battle-lane ask), the Glicko-2 constants, the matchmaking window, the checkpoint interval of the journal, the
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

- A new engine variant breaks `h3-protocol`'s build (ADR-0018 point 5 as amended): the battle
  lane adds its arm in the same pull request (label `cross-lane`) or the server lane maps it
  first; the global HANDOFF names the trap.
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
- A connected player can stall a ranked battle (SRV-Q19): M4 has no turn timer; the game's
  rules bring one (`h3-game`, M5). Until then a ladder's operator hears of it from players.
- Information leaks through channels the engine does not model (message timing, sizes):
  bounded by per-viewer sequence numbers and projection; timing stays out of scope for now.
