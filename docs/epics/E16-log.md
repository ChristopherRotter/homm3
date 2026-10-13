# E16 log: done tasks in full

The full record of E16's finished milestones and tasks (what was built, agent decisions marked
provisional, measurements, the review lists). **Not start-of-session reading:** the epic file keeps
one line per done milestone; read the matching section here when a task builds on an earlier one
or a decision is questioned. Decisions live in ADR-0018, ADR-0019 and ADR-0021; this log only records
how and why the work went.

## Done

### M0 — The shared skeleton (2026-10-10)

- **The ADR.** The user accepted the protocol ADR draft at the checkpoint, asking for a trim
  first: 136 lines became 85, the twelve decisions unchanged; the cut text was already in the
  epic file (the measurements, the options) or the design note (§3 the manifest, §7 journals,
  §8 integrity). Numbered ADR-0018, moved to `docs/adr/` with its index row; the server lane's
  reading list in `AGENTS.md` gained it.
- **The crates.** `h3-protocol`: adapter layer, `logic = true`, the logic header, only
  `PROTOCOL_VERSION` (0 until M1 T1's first messages). `check-arch` accepted a logic crate in the
  adapter layer without a tool change (it checks purity by the `logic` flag and layers apart).
  `h3-server`: the binary `h3-server`, which prints its versions and exits 1 (SRV-STUB-1).
- **The dependencies.** The workspace gained `tokio` (`rt-multi-thread`, `net`, `time`, `sync`,
  `macros`), `tokio-tungstenite` 0.30 (`connect` only: no TLS), `futures-util` (`sink`, `std`;
  already in the graph through `tokio-tungstenite`; needed for `StreamExt`/`SinkExt` on the
  socket; *queued for the M1 review* as a light dependency), `getrandom` 0.3 (the version
  already in the lock), `postcard` (test only; *provisional*) and the path entries `h3-cli`,
  `h3-protocol`. `arch.toml` allows every edge M1 needs, so T1–T3 need no shared pull request;
  the crates declare a dependency when they use it. Checked in a throwaway worktree with all of
  them added: the build, `cargo deny check` (licences, bans, advisories, sources ok), 77 crates
  under `h3-server`, `serde_json` resolved with `alloc` only for `h3-protocol`. *Provisional:*
  `h3-server` → `h3-cli` for `load_ruleset`, `load_scenario` and `replay_to_toml`; the
  alternative is a battle-lane move of those into `h3-sim` (queued for the M1 review).
- **Docs.** The versions table (`PROTOCOL_VERSION`, `RECORD_FORMAT_VERSION` from M1,
  `JOURNAL_FORMAT_VERSION` from M2), `BACKLOG.md` E16 in progress, the global HANDOFF's server
  row, the README's crate table, `02-architecture.md`'s state paragraph, `benches/server/README.md`.
- **Found on the way.** The baseline verify on `master` failed `check-exe-map`: the adventure
  lane's E08 survey (CR-1 to CR-3) and ADV-Q3 to ADV-Q5 cite the H3API addresses of
  `GetMovementCost`, `CalcMaxMovement`, `MaxLandMovement` and `GetScoutingRadius`, which
  `docs/exe/map.toml` does not cover (the `test` step fails with it: xtask's
  `the_real_map_passes`); the merge was docs-only, so CI never ran on it. A map entry needs the function's size,
  which needs the executable (not staged in this session). The user left the fix to the adventure
  lane, which added the four map entries on `master` (`8eb412c`). Everything else passed on the
  M0 commit (coverage, determinism, ffi-smoke, bench-check, deny).
- **Also found.** The baseline's `test` step failed the battle property test
  `every_step_preserves_the_invariants` (a good-morale step without `HAD_MORALE` on the mover). Its
  inputs are random per run, so CI on `master` passed; the shrunk case reproduces in 5 s. Handed to
  the battle lane as a suggested task with the regression line; not committed here. The battle
  lane's `65a7a75` (the invariant allows a round's end after a lost turn) makes that case pass.
- **The merge with `master`** (E08 T2 and T3, E04b T8b) before the pull request merged: the
  adventure lane had accepted ADR-0015 to ADR-0017 first, so the protocol ADR became ADR-0018
  (only the references this branch added were renumbered); conflicts in the workspace path
  entries, the versions table, `02-architecture.md`, the ADR index and the global HANDOFF's rows.

### M1 T1 — Messages and codec (2026-10-10)

- **What was built** (`crates/h3-protocol`, all *provisional* in names and layout): `wire` (the
  published language: `ClientMessage` and `ServerMessage`, the battle `Command`, `Event`,
  `Observation` with `Stack`, `Hero`, `OwnHero` and `SideView`, the `Manifest` with `KeyTables`
  and `Field`, `RejectCode`, `Refusal`, `GoodbyeReason`, `WireU64` for 64-bit values as decimal
  strings), `codec` (one JSON document per frame, the size checked before parsing,
  `DEFAULT_MAX_CLIENT_FRAME` 4096 bytes, decode errors `TooLarge` and `Malformed`) and `map`
  (wire ↔ engine). `PROTOCOL_VERSION` is 1.
- **Projection by construction.** `map::observe` and `map::events` take the viewer and call the
  engine's `observe` and `project`; the raw observation conversion is crate-private, so the
  session core (T2) cannot send an unprojected observation or event.
- **The mapping is exhaustive (the user, ADR-0018 point 5 amended).** The first build had the
  ADR's wildcard arms to `Unsupported`/`Other`; llvm-cov counted each as an uncovered region
  (measured on a scratch crate and on the real one: 6 arms, 9 regions, 14 lines), so they
  needed a file exclusion in the shared `coverage-exclusions.toml` (`#[coverage(off)]` is
  still unstable on 1.97). Asked where the exclusion should land, the user chose no exclusion:
  exhaustive matches, a new engine variant fails the build until mapped; the `Unsupported` and
  `Other` wire variants went with the arms. The global HANDOFF names the trap for the battle
  lane. Engine enums matched: `Command`, `EventKind` (47 kinds), `CommandError` (29),
  `CastTarget`, `DamageRoll`, `ObstacleKind` (`h3-content`); the contract enums `BattleSide` and
  `BattleResult` are a seam anyway.
- **Choices for the review** (the epic's review list): `Refused` for a refused join, the
  session's codes `NotSeated` and `NotStarted`, the manifest in `Joined` rather than `Welcome`
  (ADR-0018 point 10 says "at join"), flag sets as named booleans, hexes as indices with the
  field's size in the manifest, the observation as the sim encoding's values (attack without
  the melee bonus, as the encoding; the own hero's spell bonuses not yet on the wire), `postcard`
  as a dev-dependency for the round trips.
- **Key tables.** Every kind has the handle-to-key direction without a battle-lane ask:
  creatures through `creature_count` and `creature_key`, the others through their slices'
  `key` fields in handle order.
- **Tests** (18): every message round-trips in JSON and `postcard`; the exact JSON forms; 64-bit
  values as strings and their refusals; unknown fields refused (a command naming a side does
  not decode); the size limit before parsing; malformed frames, deep nesting; arbitrary bytes
  and JSON-like text never panic (proptest); every engine variant maps to the wire variant of
  the same name; commands round-trip; events projected; the observation's every part;
  the legal list validates in the engine; castable spells and their targets; key tables.
  `h3-protocol` at 100 % lines and regions; `verify --scope server` with no failing step.

### M1 T2 — Session core (2026-10-10)

- **What was built** (`crates/h3-protocol`, all *provisional* in names and layout):
  `connection` (one per socket: the hello gate, `Welcome` or `Goodbye { Protocol }` for another
  version or anything before the hello or a second hello; `Pong`; a frame that does not decode
  closes with `Malformed` or `TooLarge`); `session` (`Session` over one battle, `SeatKind`
  player-with-token or bot, the `Bot` hook, `GameConfig`, `Journal`, `ConnId`, `Outbound`);
  `map::manifest` (the replay header's fields, the field, the key tables); `codec::encode_text`;
  `RECORD_FORMAT_VERSION` 1 and `wire::RecordHeader`.
- **The wire amended, `PROTOCOL_VERSION` 2** (*queued for the M1 review*): `FetchReplay` and
  `FetchRecord` carry the client's `seq`, `Replay` and `Record` echo it; `RejectCode::NotOver`.
  Without them the session could neither refuse a replay before the end nor answer a fetch from a
  connection without a seat in a way the client can pair. `Replay` and `Record` carry the
  viewer's last number and take none (so the record never contains itself).
- **Agent decisions (provisional):** frames leave as text, encoded once (the bytes sent are the
  record's lines); both sides are viewers, numbered and recorded also while no connection holds
  the seat (a bot's seat too), so a re-taken seat's numbers go on; an `Events` message for every
  accepted command, even an empty one, so the shape of a viewer's stream does not depend on what
  it may not see; a connection without a seat gets `Rejected { n: 0, NotSeated }`; a command
  before the start gets `NotStarted` (queries are answered any time); the game starts when every
  seat is held at once and stays started, so two bot seats never start; a seat is taken without a
  token only the first time (the token comes in `Joined`), afterwards only with it, and a re-take
  replaces a holder (`Goodbye { Replaced }`); `Leave` frees the seat for its token; tokens are
  compared in time independent of where they differ; the bot hook is a trait object the session
  owns (`Box<dyn Bot + Send>`), asked with the state, its side and its legal list (it must use
  only per-side functions), a rejected choice replaced by the first legal command the engine
  accepts; the replay text is the host's (`publish_replay`) and goes out only once the battle is
  over, even if published earlier; the start events of `BattleState::new` are not sent (the
  snapshot shows their result).
- **Bounds.** One command in flight per seat holds by construction (one message at a time on the
  game's thread). A record grows with its seat's numbered messages, so with the host's rate limit
  (T3); a `FetchRecord` rebuilds the text.
- **For T3.** The host keeps a `Connection` per socket, passes `Received::Game` to the session
  with the socket's `ConnId`, calls `disconnected` when a socket closes, sends each `Outbound`
  and closes after one with `close`; once `result()` is set it builds the replay from
  `journal()`, `ReplayHeader::current` and `state_hash()`, writes it as the CLI's TOML and calls
  `publish_replay`.
- **Tests** (21 new, 39 in the crate): two players join and play to the end (numbers without a
  gap, `GameEnded` to both, every `Accepted` journaled, the journal verifies as an `h3_sim::Replay`
  with its checkpoints); the acknowledgement order; a connection without a seat and a seat before
  the start (state and journal unchanged); every refusal; re-take with the token, the record
  spanning the gap, replacement, leave; a seat re-taken on its turn gets `YourTurn`; the sim's
  random agent and a bot whose every choice is rejected play a seat to the end and the journal
  verifies; two bots never start; the record (header, the numbered frames, not recorded itself);
  the replay only after the end; queries; the manifest against the replay header; journals
  without checkpoints; a refused setup. Properties: **observation hygiene over the wire**: the
  hidden obstacles the side to act casts at the first turn after the tactics phase (in the `full`
  scenario the defender's Quicksand, the synthetic `mire`) leave the other viewer's bytes (and a
  fresh snapshot after a re-take) equal for any two seeds, while the caster's differ (checked by a mutation: projecting every
  viewer's events as the defender's fails both tests); **a rejected command changes nothing**:
  any command from either seat at any point is accepted and journaled, or rejected with the
  battle's hash and the journal unchanged and nothing sent to the other seat, and the seat not to
  act always gets `NotYourTurn`. The connection: hello, version, out-of-place messages, ping,
  malformed and oversized frames. `h3-protocol` at 100 % lines and regions with no exclusion.

### M1 T3 — The server binary (2026-10-10)

- **What was built** (`crates/h3-server`, now a library and the binary; all *provisional* in
  names and layout): `config` (the TOML file, below), `game` (`Prepared::new` loads the
  scenario and draws the secrets from the OS; the game's OS thread runs the session core),
  `net` (the listener, a `tokio` task per connection, the `Transport` trait and its WebSocket
  implementation), `client` (the headless client), `start` and `Running` in `lib.rs`, `main.rs`
  (`h3-server <config.toml>`), `server.example.toml`. SRV-STUB-1 retired.
- **The configuration** (*queued for the M1 review*): a flat TOML file for M1's one game,
  relative paths from the file's directory: `bind` (default `127.0.0.1:7878`), `packs`,
  `scenario`, `profile` (overrides the scenario's), `bots` (seats the server's bot plays; both is
  refused, a hosted game needs a player), `data` (replays go to `<data>/replays/`),
  `checkpoint_every` (25), `[limits]`: `max_frame` 4096, `messages_per_second` 20, `connections`
  64, `connections_per_address` 8, `idle_timeout_ms` 300 000, `outbound_queue` 256.
- **Secrets** (ADR-0018 point 8): the battle's seed, the obstacle seed (used once for a scenario
  with `random_obstacles`, then forgotten; the obstacles travel in the setup as pieces), the
  player seats' tokens (128 bits as 32 hex digits) and the bot's seed come from `getrandom`.
  `Running.tokens` lets an operator hand the tokens out; otherwise the first joiner of a seat
  gets it in `Joined`.
- **Threads and channels** (*provisional*): the game's inbox holds 1 024 messages (a connection
  task waits when it is full); each connection has an outbound queue of `outbound_queue` frames
  that the game fills with `try_send`, and a full queue makes the game forget the connection
  (`Session::disconnected`), whose task, finding the queue closed, says `Goodbye { TooSlow }`.
  A write that stalls for the idle timeout closes the connection, and so does the WebSocket
  handshake.
- **Bounds** (*provisional*): idle means no message *from* the client within the timeout (the
  server's own frames do not count; a passive client pings); the rate is counted in one-second
  windows, beyond it `Goodbye { RateLimited }`; a binary message is `Malformed`; the WebSocket's
  own cap is 16 times the frame limit (at least 64 KiB), so a frame just over the limit gets
  `TooLarge` and anything larger is dropped unread; a connection beyond the total or
  per-address count is closed before its handshake (a counted place, released when its task
  ends).
- **Found by the flooding test: the lingering close.** A connection closed while the client's
  later messages lay unread in the socket was reset by the kernel, and the reset made the client
  drop the `Goodbye` it had not read yet (the test failed in most runs). The transport now reads
  on after its close frame, for at most a second, until the peer's close.
- **The replay** is written when the battle ends, before it is published to the seats:
  `<data>/replays/<scenario>-game1-<random>.toml`, the CLI's format through `h3_cli::replay_to_toml`;
  a failure to write is logged and the game goes on. Log lines go to standard error (start,
  the replay's path). No graceful shutdown yet (M2: `tokio`'s `signal` feature is not among the
  decided ones).
- **Dependencies.** `tokio` 1.53, `tokio-tungstenite` 0.30 (`tungstenite` 0.30), `futures-util`,
  `getrandom` 0.3 (the workspace's); `tungstenite`'s `rand` 0.10 brings `getrandom` 0.4 as well, a
  duplicate that `cargo deny` only warns about (`multiple-versions = "warn"`); `cargo deny
  check`: advisories, bans, licences and sources ok. 125 crates under `h3-server`.
- **Tests** (14; `h3-server` is an adapter, outside the coverage gate): over loopback, two
  clients play the `full` battle to the end, the replay they fetch equals the file, and the file
  verifies; a client plays the bot on the `duel` field of random obstacles, and its replay
  verifies; a dropped client re-takes its seat with its token (without it: `SeatTaken`); a
  message before the hello, another version, text that is no message, a binary message and a
  frame over the limit each get their goodbye, a frame far over it is dropped, and the game is
  unharmed; flooding and idling are cut off; the per-address limit refuses a second connection
  until the first closes. Units: the rate, the counted places, a dropped queue says `TooSlow`
  (a fake transport), a full queue frees the seat for its token; the configuration's defaults,
  paths and refusals; tokens; the example configuration loads and prepares. Ten runs of the
  suite in a row passed. `h3 verify` (the CLI) accepted the replays the tests wrote (82 and 16
  commands, the duel's with its obstacles).
- **For later.** Automated players answer every turn at once and exceed 20 messages per second
  (the end-to-end tests raise the limit): an agent seat (later) needs its own limit.
  A connection task reads a closed queue as `TooSlow`; the game's thread closes the queues
  otherwise only if it panics (it runs while the listener holds its inbox), so a dead game is
  reported as `TooSlow` until M2's shutdown and restore give it a reason of its own.

### M1 T4 — Closure (2026-10-10)

- **The workload** (`benches/server/session-random-play-full.toml`; *provisional* in its shape):
  `h3_server::bench` plays hosted battles with two scripted clients frame by frame through the
  hello gate (`Connection`) and the session core, as a connection task and the game's thread do,
  without sockets or threads; `serve_frame` (never inlined) is the server's whole work for one
  client frame and the only function counted. The clients answer each `YourTurn` (the last frame
  of an answer, found by its prefix and decoded) with a strike if there is one
  (`bench::choose`, now also the end-to-end tests' choice); fixed tokens; no queries or
  fetches. The binary `h3-server-bench` (`--scenario`, `--packs-dir`, `--games`, `--seed`).
  20 games of the `full` scenario, all ended: 2 111 frames in, 8 284 out, 4.6 MB;
  **273 113 627 Ir**, deterministic across runs and environments. Under the arguments of the
  file only: a run without arguments counts 0.04 % fewer (the arguments' allocations move the
  heap that `malloc` walks).
- **Lane-only (a shared change, *queued for the M1 review*).** The profile: about 55 % of the
  count is the battle engine (`legal_commands` 22 %, `apply` 19 %, `castable_spells` 12 %
  inclusive), 34 % JSON encoding, 2 % decoding. So the count moves with the battle lane's
  engine speed, rules and synthetic packs (the `full` scenario's battles change with nearly
  every battle increment), and a gate in the full verify would fail battle pull requests on a
  server file. Considered: excluding the engine (callgrind's toggles flip on every entry, so
  nested engine calls cannot be cut out; subtracting call edges double-counts closures run
  through `core::iter`; counting only the protocol's own functions still follows the battle's
  trajectories); a codec-only workload (misses validate and project, which the epic names). Chosen:
  `xtask bench-check` lets a workload name its package and binary and be `lane_only`, measured
  by `--lane`, `--workload` and `verify --scope <lane>`, never by the full verify. The server
  lane checks it at each session's start and re-baselines a drift `master` brought
  (`benches/server/README.md`).
- **`docs/06-security.md`** (shared): "Hosted games", the server's guarantees as built in M1
  (ADR-0018 points 6–10): the seat as identity, rejected commands, projection by construction
  and the stream's fixed shape, the secret seed and the throwaway obstacle seed (the queue's
  inbox item from E03 T9, closed), the replay after the battle, the configured bounds and their
  defaults, exposure, what is not covered yet. The global HANDOFF's server row (it said "T2
  next"; the file has 5 bytes of budget left, so the row is short).
- **Tests** (2 new, 16 in `h3-server`): the workload plays its games to the end,
  deterministically, and holds its command limit; the scripted client's choice.
- **The reviewer agent** (fresh, read-only; 2 major, 7 minor findings; no leak of state,
  hashes, seeds or tokens found, no panic from client input, identity and rejection sound).
  Fixed: (major) the close frame's write had no timeout, so a peer that stopped reading held its
  task and counted place forever; `close` now bounds it by the linger second too, and an
  end-to-end test (a client that stops reading; a new connection from its address is welcomed)
  fails without the fix, after first passing with a queue of 4 frames, which dropped the
  connection before the socket filled; the limits are range-checked at load (an idle timeout
  near `u64::MAX` panicked in every connection task, 0 refused every handshake); three doc
  claims corrected (`bench-check` without flags skips lane-only workloads; `NotYourTurn` only
  while the battle runs; the binary shows the tokens to no one); new tests: a property over
  arbitrary `Join`, `Leave`, disconnect, query, fetch and command sequences from four
  connections (`Joined` only without a seat; numbered frames only to a seat's holder, each its
  record's line of that number; number 0 only without a seat; checked by a mutation: without
  the `AlreadyJoined` refusal it fails), a seat's token reaches only its holder, a replaced
  connection is no actor. Deferred to M2 T5 (`QUEUE.md`; `06-security.md`, "Not covered yet"):
  (major) memory bounded in messages, not bytes (a seated client grows its record with queries
  within the rate limit, and each `FetchRecord` queues the whole record); a connection that never
  joins holds its place until its idle timeout; WebSocket pings outside the rate. To M2 T2: an
  operator's copy of the tokens (a client that loses its `Joined` loses the seat).

### M2 T1 — Many games on one server (2026-10-11)

- **The checkpoint.** The user merged the M1 closure (#61) and asked for the next increment:
  SRV-Q12 answered (the review list stands, M2 as cut; `signal` for T5, the token itself in
  T2's journal). The branch restarted from `master`; the lane-only workload measured −0.12 %
  after the battle lane's T8c (no re-baseline), the scoped verify had no failing step.
- **The wire, version 4** (*queued for the M2 review*): `ListGames { seq }` → `Games { seq,
  offers, games }` (an `Offer` names what the manifest names: id, scenario, packs, profile,
  ruleset hash, the bot's seats; a `GameInfo` its id, offer, `Phase` and the seats open
  without a token); `CreateGame { seq, offer, seat }` → `GameCreated { seq, game }`, then the
  game's own `Joined` and `Snapshot`, so a created game never waits without a connection; the
  refusals `NoSuchOffer` and `TooManyGames`. No `StartGame` (the design note's): a game starts
  when every seat is held at once, as in M1.
- **The lobby** (`h3-protocol`, logic, 100 %): `Lobby<O>` over an origin type the host picks
  (the server: the client's address); ids from 1, never reused; the games not over limited in
  total and per origin; a spectator, an unknown offer and the bot's seat refused; updates from
  the games; removal. `Session::phase` and `open_seats` give the lobby its view.
- **The server** (*provisional*): `config` in sections (`[server]`, `[content]`, `[limits]`
  with `games` 16 and `games_per_address` 2, `[[offer]]`); `game::Offer` compiled once at start
  (one `Arc<Ruleset>` per offer, shared by its games); `Prepared` per creation (the M1 secrets);
  `registry` (the lobby and each game's inbox behind one `std` mutex, the bound connections
  counted); the connection task answers the lobby, binds to a game for `Join` or `CreateGame`
  (a fresh outbound queue per binding; joining or creating another game leaves the first as a
  disconnect would), passes the game's messages to the bound game and answers them `NotSeated`
  without one. A game's thread reports its phase and open seats after every message and leaves
  when it is not running and no connection is bound; the check and every binding share the
  lock, so a connection never binds to a game that left (the race a check of the game's own
  connection list would have: a join queued while the game decides to close). A creation the OS
  randomness fails after the lobby's checks is logged and refused as `TooManyGames`.
- **Known, for later tasks:** a running game nobody returns to stays (T3's pause and T5's
  hardening); a connection whose join a game refused stays bound to it, keeping a waiting game
  listed until it leaves or idles out. `06-security.md`'s bounds and the global HANDOFF's server
  row are updated in a separate commit; the pull request carries `cross-lane` (the user).
- **Tests** (h3-protocol 44, h3-server 19; the scoped verify with no failing step, the workload −0.10 %): the new messages round-trip, their JSON forms; the
  lobby's creation, limits, refusals, updates and removal; the session's phase and open seats;
  over loopback, two games played at once with both replays verifying, then gone once their
  players leave; the lobby's refusals over the wire and `NotSeated` without a game; a waiting
  game leaves with its creator and frees its address's place; the earlier tests through the
  lobby; the configuration's sections and checks; the `TooSlow` unit test binds through a
  registry. Five runs of the server's suite in a row passed.

### M2 T2 — The write-ahead journal and restore (2026-10-11)

- **The start.** #67 (T1) was merged; the branch started from `master`. The scoped verify had no
  failing step, the workload −0.10 % (no re-baseline).
- **The session core** (`h3-protocol`, logic, 100 %; *queued for the M2 review*): `Entry` (a
  seat taken, the start, an accepted command with its side, a checkpoint), made in the
  session's order and drained by the host after every call (`drain_entries`); a rejected
  command makes none. `Session::restore` re-runs a journal's entries in a fresh session: the
  seats taken (`issued`), the start, the commands through the engine; the re-run makes its own
  entries, which the journal must begin, so a checkpoint whose hash differs or that is out of
  place is `Diverged { at }`, a command the engine refuses `Rejected { at, error }`, a setup that
  does not start `Setup`. A journal cut short is completed: the checkpoint after its last command
  and, in a started game whose bot is to act (a crash between a player's command and the bot's
  answers), the bot's commands are the first entries drained after it. No seat is held after a
  restore; a viewer's numbers and record start afresh (they are not journaled). `Lobby::reopen`
  lists a restored game again without the limits' check, ids continuing after it.
- **The journal** (`h3-server`, `journal.rs`; *queued for the M2 review*:
  `JOURNAL_FORMAT_VERSION` 1, the layout): `<data>/games/<game>.journal`, JSON lines. The first
  line is `Start { format, game, offer, origin, tokens, replay }`: the creator's address (the
  per-address limit after a restart), the seats' tokens (SRV-Q12 (d): the token itself; this
  line is the operator's copy, for a player who lost its `Joined`) and the game's replay before
  its first command as the CLI's TOML (header, seed, setup with the obstacles, checkpoint
  interval), so the replay format is not duplicated. Then one line per entry: `Taken { side }`,
  `Started {}`, `Command { command }` in the replay's text form, `Checkpoint { hash }` as hex.
  Created with `create_new` and synced with its directory before the game is listed; on Unix
  the directories the server creates are 0700 and the journals 0600 (*provisional*). A last line
  without its newline is a write a crash cut short: dropped on reading, truncated before the
  next append. The reader's errors name the line and column, never its text (a line may hold
  tokens).
- **Write-ahead.** The game's thread appends a call's entries and syncs them (`sync_data`)
  before it delivers that call's frames, so an `Accepted`, a `Joined` with its token or a
  `YourTurn` never leaves before what it depends on is durable. A journal that does not write
  stops the game: its connections get `Goodbye { Shutdown }`, it leaves the lobby, the next start
  restores it from its last synced line (*provisional*; no new wire reason).
- **The end.** A finished game's replay is written as before, then its journal is removed (the
  replay is the finished journal); a replay that does not write keeps the journal, which the next
  start re-runs to write it. A game that leaves without having started removes its journal.
- **Restore at start** (`registry::restore`, before the server listens): each journal is read,
  its offer looked up by id, its replay read on the offer's ruleset and its header checked
  (`ReplayHeader::check`); then the game's own thread re-runs it and appends what it lacked. A
  running game is listed again (`Phase::Running`, seats empty, re-taken with the tokens); one
  that is over writes its replay and goes; one that never started goes. A journal that does not
  read, names an offer the server no longer has, whose replay does not read on the offer's
  content or whose engine or content identity differs, or that does not re-run, is void: moved to
  `<data>/void/<game>-<random>.journal` with a log line naming why (no hashes or tokens). The
  restored seats come from the journal (a token or the bot), not from the offer's current
  `bots`; the bot gets a new seed. `start` waits for every re-run before it binds.
- **Known, for later tasks** (`QUEUE.md` inbox): a viewer's numbers and record restart after a
  restore; ids of games that ended or went void may be handed out again after a restart (the
  replay and void files carry a random part); a restored game nobody returns to stays (T3, T5).
- **Tests** (h3-protocol 49, h3-server 24; the scoped verify with no failing step, the workload −0.06 %): the entries
  of a played game are its journal and restore it (the same state hash, journal, phase, no seat
  held, `NotSeated`, `SeatTaken` without the token, the same snapshot and turn after a re-take);
  flipped and misplaced checkpoints, an extra one, a refused command and a bad setup; a journal
  cut before its checkpoint or before the bot's answer is completed; a waiting game's taken
  seats; the property over any messages restores to the live game, and a rejected command
  journals nothing. The journal's lines, permissions, a torn line, the reader's refusals, the
  void directory. Over loopback: a game stopped mid-battle (the data directory copied at that
  moment, as a crash leaves it) resumes on a second server with the same snapshot and turn, plays
  to its end, and its replay (commands from both servers) verifies; a start with a torn journal
  (restored), one of another ruleset and one of a missing offer (void), one of a waiting game
  (gone), new ids after the restored one; finished and waiting games leave no journal.

### M2 T3 — Verification, pause and resume (2026-10-11)

- **The start.** #71 (T2) was merged; the branch started from `master`. The scoped verify had no
  failing step.
- **The wire, version 5** (*queued for the M2 review*): `Pause { seq }`; `PauseAsked { n, seat }`
  to every viewer; `Phase::Paused` in the lobby's list; `GoodbyeReason::Paused`;
  `RejectCode::Unverified` for the replay of a game that did not verify. No `Resume`, `Paused` or
  `Resumed` (the design note's): a seat's `Join` with its token resumes, the goodbye says it paused.
- **The session core** (`h3-protocol`, logic, 100 %): the *host* is the seat taken first (from the
  first `Taken`, also in a restore); its `Pause` in a running game makes the entry `Paused`, closes
  every holder with `Goodbye { Paused }` and leaves the session `paused` (`Phase::Paused`); another
  seat's is told to every viewer. Refused: `NotSeated`, `NotStarted` (waiting), `BattleOver`. A
  join that takes a seat in a paused session resumes it with the entry `Resumed` (so a restart
  after a resume restores a running game, as its players were told); a restore whose journal ends
  paused is paused. `withhold_replay` makes `FetchReplay` after the end `Unverified`. The *shadow
  instance* (`GameConfig::shadow`): a second battle and the battle as it began; every command is
  applied to both, and when they differ in their hashes or in accepting it, both are rebuilt from
  the journal's commands and the command is applied again to the truth before anything is sent;
  the steps are drained by the host (`drain_mismatches`) for a log line. `same_token` is public
  for the host.
- **Verification** (`h3-server`, `game.rs`): when a game ends, its journal is read back from disk,
  its first line's replay and its commands and checkpoints make the replay, with the live game's
  final hash and result; `Replay::verify` re-runs it in a fresh battle (the header, every
  checkpoint, the final hash, the result), and the ruleset's content hash is recomputed from memory
  (`h3_kernel::hash::hash_of`) and compared with the one compiled at start. Verified: the replay
  rendered from what was re-run is written and published as before. Not: the game is void, the
  replay withheld, the journal moved to `<data>/void/`, the replay as played written beside it with
  the diagnosis as its first line (`# void: …`, hashes allowed: an operator's file), the log line
  without hashes or the journal's replay text (it holds the seed). `GameEnded` still goes out before
  the check (*provisional*); a restored game that is over goes through the same check.
- **Pause and resume** (`h3-server`): after a call that paused, the game's thread delivers the
  frames, closes its other connections with the same goodbye, hands the registry the seats' tokens
  (`Hosted::Paused`) and ends; the lobby lists it as paused, the journal stays and ends with
  `Paused`. A `Join` for a paused game is checked by the registry against the tokens (`BadToken`,
  `SeatTaken` for none or the bot's seat, `NoSpectators`), then the journal is read and its game
  restored on a new thread (the join waits in its inbox) and goes on. At start a paused journal is
  re-run as any other and listed as paused, its thread ending. A resume that does not re-run voids
  the journal and turns away the connections that came (`Goodbye { Shutdown }`); a connection
  bound before a pause no longer counts for the game a resume restores (`unbind` compares the
  inbox).
- **The journal**, format 2: the lines `Paused {}` and `Resumed {}`. **The configuration**:
  `[server] shadow_instance = false`.
- **Known, for later tasks** (`QUEUE.md`): a paused game counts against the game limits until a
  deadline exists (T5, `pause_hours`, with the running game nobody returns to); `GameEnded` before
  the verification.
- **Tests** (h3-protocol 52, h3-server 26; the scoped verify with no failing step, the workload
  −0.05 %): the wire's new messages round-trip, their JSON forms;
  the host's pause, another seat's ask, the refusals, a restore that ends paused, a refused join
  that resumes nothing, the resume and its entry; the withheld replay; the shadow instance plays
  as without, a rejected command, a fault in the live battle and one in the shadow found and
  rebuilt with the viewers' frames unchanged, a restore through both battles; the property over any
  messages with pauses. Over loopback: a journal changed on disk mid-game (a checkpoint's hash, a
  command given to the other side) is caught at the end (`Unverified`, no replay, the journal and
  the diagnosis in the void directory); a game paused by its host is listed paused, refuses joins
  without its token, is still paused on a second server started on a copy of the data directory,
  and resumes where it paused on both, its replay verifying at the end.

### M2 T4 — Spectators (2026-10-11)

- **The start.** #72 (T3) was merged and the battle lane's spectator viewer with it (#74, E04b
  T9a: `h3_battle::Viewer`, `observe_as`, `project_as`); the branch started from `master`. The
  same note listed the observation's new fields the wire does not map: queued as T4b.
- **The wire, version 6** (*queued for the M2 review*): `Observation::spectator`; `Refusal::
  NoSpectators` now means the game takes no more (its limit), or none (a creation, a paused game).
  The mapping's `observe` and `events` take a viewer (`impl Into<h3_battle::Viewer>`, so a side
  still fits).
- **The session core** (`h3-protocol`, logic, 100 %): `GameConfig::spectators`, the limit; a
  `Join` as a spectator, within it, gets `Joined` without a token and the spectator's snapshot; it
  is a viewer of its own (`Watcher`: its connection, numbers and record) in the order of joining.
  It gets the public events of every command, `GameEnded`, `PauseAsked`, and on a pause
  `Goodbye { Paused }`; never a turn. Its `Command`, `Query`, `FetchReplay` and `Pause` are
  rejected `NotSeated`, numbered and recorded; its `FetchRecord` is its record since it joined,
  the header's seat `Spectator`. A connection that holds a seat or watches is `AlreadyJoined`.
  `Leave` and a disconnect end its watch. Nothing of a spectator is journaled. The viewers' send
  is one `Viewer::post`, shared by seats and spectators.
- **The server** (*provisional*): `[limits] spectators_per_game` (8; 0 to 1024, 0 for none), given
  to each game by the registry (`Registry::new` takes the configuration's limits). A paused game
  refuses spectators before it is restored (T3).
- **Tests** (h3-protocol 53, h3-server 27; the scoped verify with no failing step, the workload
  −0.39 %, mostly the battle lane's T9a): a late spectator's snapshot (`spectator`, no own
  hero), the events and no turn, its numbered rejections and record, `PauseAsked`, the limit and a
  freed place, the pause's goodbye, the end; the joins' refusals with spectators; the hidden
  obstacles' property now also over two spectators' bytes (one from the start, one after the
  cast); the seats-and-numbers property with spectators as viewers (no token, no turn, records
  numbered without gaps, the header's seat). Over loopback: a spectator watches a game against the
  bot to its end within a limit of one, a second is refused, its `FetchReplay` is `NotSeated`.

### M2 T4b — The observation's completeness on the wire (2026-10-11)

- **The start.** #76 (T4) was merged; the branch restarted from `master` (the scoped verify on a
  clean worktree had no failing step but coverage, whose LLVM paths were not set; the workload
  −0.39 %). The user asked whether a host can turn spectators off: only the operator can, for
  every game (`[limits] spectators_per_game`); queued as T4c.
- **The wire, version 7** (*queued for the M2 review*): on `Observation` the wait phase and the
  hexes a removal cleared; on `SideView` the creature its Summon Elemental casts are bound to and
  its hero's spell-cost change; on `Stack` the base count, the retaliations per round, the flags
  `started`, `native` and `raised`, and the values the engine keeps per stack beside it (the clone
  link, the temporary creatures, the corpse's order, the binders as stack ids, Poison's tenths,
  the start hit points); on `Effect` its mastery, value and its shares of attack, defense and
  speed; on `OwnHero` the hidden magic (BAT-Q26): the spell damage bonuses (all, per school, per
  spell), the magic resistance, the duration bonus, `no_casting`, the level limit, the defences,
  `negates`, the opening spells and whether they are due, Eagle Eye's chance, spells and what it
  learned. Spell sets travel as handles, rates as basis points.
- **The completeness** (`map/mod.rs`): every engine struct the observation is made of
  (`Observation`, `Stack`, `SpellValues`, `ActiveEffect`, `SpellObstacle`, `TacticsPhase`,
  `HeroView`, `OwnHeroView`, the hero's damage, Ballista, tent, spell bonuses, defences and
  opening spells, `Rate`) is named in a pattern without `..`: a new field fails the build until it
  is mapped or bound to `_` under its reason, as a new variant does (ADR-0018 point 5 as
  amended); the queue asked for a test, the build is the earlier and surer check. Left out on
  purpose: a rate's single-precision value (the rules' arithmetic, quirk Q-0005); a stack's base
  attack, defense and speed (the values now are on the wire, the effects' shares with the
  effects); the values the stack's effects materialise (protections, slayer, fire shield, frenzy,
  spell immunity, mirror, status, the morale and luck changes; BAT-MAG-011), which follow from the
  effects with the spells' data; the traits other than the named ones (the creature's data). The
  removable obstacles' fields are private: read through their accessors. `h3-contracts` moved to
  `h3-protocol`'s dependencies for the hero's types (an edge `arch.toml` allowed already).
- **Tests** (h3-protocol 53, h3-server 27; the scoped verify with no failing step, coverage
  100 %; the workload +1.00 % against −0.39 % on `master`, the larger snapshots' encoding:
  re-baselined at 275 849 212): the mapping test gives every new field a distinctive value and finds it on the wire; the
  hygiene properties unchanged (the new hero fields ride in `own_hero`, which only the hero's side
  gets).

### M2 T4c — The host's spectator setting (2026-10-11)

- **The start.** #78 (T4b) was merged; the branch restarted from `master` (the code the T4b
  session verified). The user asked for the task and answered its two questions: the host changes
  the setting only before the start; a new game takes spectators unless its creator says otherwise.
- **The wire, version 8** (*queued for the M2 review*): `CreateGame`'s `spectators: Option<bool>`
  (`#[serde(default)]`: left out, or `None`, the game takes them); `SetSpectators { seq, open }`,
  answered `SpectatorsSet { n, seq, open }` (numbered, the host's seat) or `Rejected` (`NotSeated`,
  the new `NotHost` and `AlreadyStarted`); `Goodbye { SpectatorsClosed }` for each spectator a
  closing ends; `GameInfo::spectators`: the host's setting on a server whose limit is above 0,
  false while paused.
- **The session core** (`h3-protocol`, logic, 100 %): `GameConfig::watchable`, the creator's
  choice; `SetSpectators` from the host before the start sets it, closing drains the spectators;
  a spectator's `Join` is refused `NoSpectators` while closed; `Session::takes_spectators` for
  the lobby. **The setting is an entry once, at the start** (`Entry::Spectators` just before
  `Started`), not at each change: a game that never started is not restored, so only the
  setting it started with must last, and a toggling host cannot grow the journal or make the
  server sync a line per message (the first draft journaled every change; caught in the
  session's own review). `Session::restore` re-runs the entry. The lobby's `create` and `update`
  carry the listed value.
- **The server** (*provisional*): `net.rs` passes the creator's choice (left out: open) to the
  registry, which lists it within the server's limit; the journal's `Spectators { open }` line
  (`JOURNAL_FORMAT_VERSION` 3: a format-2 journal is void after the upgrade); a restored game
  takes its setting from that line; the game's thread reports the setting with its phase and
  seats. Not built: an offer's own setting (M4's ladders, whose games have no host).

### M2 T5a — Shutdown, drain, the deadlines of abandoned games (2026-10-11)

- **The start.** #79 (T4c) was merged; the branch restarted from `master`. T5 as cut at M1's
  checkpoint held six tasks' worth; split into T5a (this), T5b (hardening) and T5c (the fuzz
  harness, the replay download, M2's closure) in `QUEUE.md`.
- **Shared** (its own commit, the pull request `cross-lane`): `tokio`'s `signal` feature in the
  workspace, ADR-0018 point 3 amended (SRV-Q12 (c)); it brings `signal-hook-registry` and `errno`
  (Apache-2.0 OR MIT; `cargo deny` licenses, bans and sources pass).
- **The wire, version 9** (*queued for the M2 review*): `Refusal::Draining`.
- **The session core and the lobby** (`h3-protocol`, logic, 100 %): `Session::seated` (a
  connection holds a seat) and `Session::suspend` (the host pauses a running game none of whose
  seats is held, as the host's `Pause` would: the entry, the spectators' `Goodbye { Paused }`);
  `Lobby::drain` (every creation refused `Draining`) and `Lobby::in_play` (the games that wait or
  run).
- **The server** (*provisional*): `Running::drain`, `in_play`, `drained` and `shutdown`; `main`
  drains at the first SIGINT or SIGTERM, stops once drained or at a second signal. The stop is a
  `watch` the listener and every connection select on (`Goodbye { Shutdown }`; the listener then
  waits up to 3 s for the connections to close) and a `ToGame::Stop` to every game's thread (its
  connections hear it, it leaves; a waiting game's journal goes, the others stay). A drain's
  `ToGame::Drain` ends only waiting games. `[limits] empty_game_ms` (30 minutes) and
  `paused_game_ms` (72 hours), 0 for none: a sweep task ticks every game's thread
  (`ToGame::Tick`), which pauses a running game whose seats stayed empty that long, and voids a
  paused game (`<data>/void/`, the reason "abandoned") whose pause is older; after a restart the
  pause's age is the journal's (its last write). Clocks: `tokio::time::Instant` (the workspace's
  clippy bans `std::time`'s; a file's age is `metadata().modified()?.elapsed()`, a `Duration`).
- **Tests** (h3-protocol 56, h3-server 31): the session's suspend (held seats, an empty running
  game, paused, resumed by a token); the lobby's drain and its count; end to end: a drain ends a
  waiting game (its journal gone), refuses a creation, waits for the running game's end, and the
  stop closes the connections and the listener; a stopped server's game restores at its next
  start and plays to its end; an abandoned game pauses (its spectator told) and goes void at its
  deadlines; the configuration's new bounds. The binary, run by hand: SIGTERM drains, stops,
  exits 0.

### M2 T5b — The connection's bounds (2026-10-11)

- **The start.** #83 (T5a) was merged; the branch restarted from `master` (the scoped verify had
  no failing step but the example configuration's test, which read an edit made while it ran).
  The user asked about ranked games (one at a time, a pause only by agreement, the paused game's
  deadline): weighed as SRV-Q13 and answered (the absent player forfeits; the pause's length
  negotiated within the ladder's bounds), for M4; the ladder ADR draft's point 8 amended. T5b as queued
  held two tasks' worth again; split into T5b (this: the connection's bounds) and T5c (memory
  in bytes); the old T5c is T5d.
- **The session core** (`h3-protocol`, logic, 100 %): `Session::holds` (a connection holds a seat
  or watches). No wire change: `PROTOCOL_VERSION` stays 9.
- **The server** (*provisional*; `net.rs`, `game.rs`): `[limits] messages_per_address` (40 a second
  across one address's connections, counted with the connection's own rate in its counted place;
  the message beyond it closes its connection, `RateLimited`); WebSocket control frames (pings,
  unasked pongs) arrive as `Incoming::Control` and count in both rates without keeping the
  connection from idling; `hello_timeout_ms` (10 s from the handshake) and `lobby_timeout_ms`
  (60 s in no game, from the hello or from leaving one), both `Goodbye { IdleTimeout }`. A
  refused join: the game counts each connection's messages; when a connection holds nothing after
  its `k`-th message, the last frame of the answer carries `Frame::left = Some(k)`, and the
  connection's task leaves the game (the registry's count and `ToGame::Closed`) only if it passed
  the game exactly `k` messages, so a second join sent before the first one's refusal keeps the
  seat it takes.
- **Not built** (*queued for the M2 review*): a separate rate for automated seats (T3's note): no
  client can be known as an agent before accounts; M3's bot accounts carry it.
- **Tests** (h3-protocol 57, h3-server 35): the session's `holds` (a seat, a watch, a refused join,
  after a disconnect); units over the fake transport: a refusal marked after the last message
  leaves the game, one marked after an earlier message does not (checked by a mutation that
  ignores the count); the address's rate shared by two places, another address apart; end to
  end: pings cut off a connection at its rate, an address's second connection is cut off by the
  shared rate while the first goes on in the next second, a connection without a hello and one
  that only lists games are cut off at their deadlines while a seated one is not, a refused
  connection no longer keeps a waiting game listed and meets the lobby's deadline, two joins at
  once take the second seat and keep it; the configuration's new bounds. Mutations that skip the
  address's rate, that let pings pass the rate or that never leave after a refusal each fail a
  test.

### M2 T5c — Memory bounded in bytes (2026-10-11)

- **The start.** #86 (T5b) was merged; the branch restarted from `master` (the scoped verify had
  no failing step; the workload +0.00 %).
- **The wire, version 10; the record's format 2** (*queued for the M2 review*):
  `RecordHeader::first`, the number of the record's first message. A viewer's record is the
  window of its latest messages within `GameConfig::record_bytes` (the bytes of its lines, each
  with its line break): after each numbered message the oldest go while the record holds more,
  the newest always stays, the numbers go on; `Record`'s lines run from `first` to `n`.
- **The session core** (`h3-protocol`, logic, 100 %): `Viewer` with a `VecDeque` of frames, its
  bytes and its bound; `GameConfig::record_bytes`.
- **The server** (*provisional*): `[limits] record_bytes` (512 KiB) and `outbound_bytes` (2 MiB; a
  record must fit twice, since its frame escapes its text once more); `game::Outbox` and
  `FromGame`, a connection's queue counted in bytes on both ends (the game adds before it queues,
  the task takes off what it takes out, an atomic count with relaxed order, a single producer); a
  frame that would pass the bytes or the frames drops the connection (`TooSlow`), a goodbye is
  queued whatever the bytes. The workload's sessions take the server's default bound.
- **Not built** (*queued for the M2 review*): one `FetchRecord` in flight as a rule of its own, which
  the queue's bound in bytes makes unnecessary for memory. Found: a battle has no round limit, so
  an endless game grows its journal (inbox).
- **Tests** (h3-protocol 58, h3-server 38): the record's window (200 queries against a 4 KiB
  bound: the oldest gone, numbered on without a gap, no more gone than had to; the other seat's
  record its own; a bound below one message keeps the newest alone, a spectator's too); the queue
  counts its bytes (a frame beyond them refused, a goodbye queued anyway, a closed queue counts
  nothing); a queue full in bytes drops its connection like one full in frames; the task takes
  off what it takes out; end to end, a client's record through the server stays within its
  bound; the configuration's new bounds and the record-fits-twice rule. A mutation that never
  trims the record fails the session's and the end-to-end test.

### M2 T5d — The decoder's mutation tests, the public archive (2026-10-11)

- **The start.** #88 (T5c) was merged; the branch restarted from `master` (its CI green). The
  user answered SRV-Q14 (mutation tests now, `cargo-fuzz` later) and SRV-Q15 (finished games'
  replays public). T5d as queued held the closure too; split into T5d (this) and T5e (M2's
  closure).
- **Mutation tests** (`h3-protocol/src/tests/mutation.rs`): every sample message of the wire tests,
  encoded, mutated one to three times; the decoders never panic, what still decodes (client or
  server) encodes back to the same message, and a client message the hello gate passes is handled
  by a running game of the `full` scenario. A second property replaces only values in client
  frames (plausible ids and hexes, extremes, odd strings), so about one frame in six still decodes
  and reaches the session (one in two hundred with every mutation). `PROPTEST_CASES=100000`
  for a longer run.
- **The wire, version 11** (*queued for the M2 review*): `ListReplays`, `FetchArchived`,
  `Replays`, `ReplayInfo { id, offer, game }`, `Refusal::NoSuchReplay`; `Replay` numbered 0 for
  an archived one. **The session core:** `Session::watchable` (the host's setting itself).
- **The server** (*provisional*; `archive.rs`): the archive in the registry, read back at start
  (before the restore, which may finish a game) from `<data>/replays/` (listed) and
  `<data>/replays/closed/` (a game closed to spectators), oldest first by their last write; a
  verified game's replay goes to the one its setting names and joins the archive; beyond
  `[limits] replays_kept` the oldest file goes. `FetchArchived` looks the id up in the archive
  (never a path from the client) and reads the file off the task (`spawn_blocking`).
- **Tests** (h3-protocol 61, h3-server 40): the mutation properties and each mutation's effect;
  a replay's name parsed into its info, others refused; end to end: four games against the bot
  (one closed), the two newest open ones listed newest first, the oldest file gone, the closed
  one kept apart; a later page and one past the end; a client in no game downloads the newest,
  which is the file and verifies; ids not listed refused; a restart lists the same; the new bound.

### M2 T5e — M2's closure (2026-10-11)

- **The start.** #93 (T5d) was merged; the branch started from `master` (its CI green). The scoped
  verify had no failing step, the workload +0.06 % (no re-baseline).
- **Closure.** M2's exit criteria written in the epic file; the client lane's inbox has the content
  check (`E16-design.md` §3) and what a restart does to a viewer's numbers and record (a line in
  `docs/lanes/client/QUEUE.md`, cross-lane with the user's consent); the inbox's M2 items placed
  (`QUEUE.md`: ids unique for a server's life and the verification before `GameEnded` to M4, the
  automated seats' rate with the bot accounts, the commands-per-game limit to M3 before the
  internet; the viewer's record across a restart documented instead of kept). The M3 refinement
  queued, waiting for the checkpoint (SRV-Q16).
- **The reviewer agent** (fresh, read-only, over everything M2 changed in the two crates; 1 major,
  8 minor findings; identity, rejection, projection and secrets to clients, panics from client
  input sound). Fixed (*provisional* in their shape):
  - (major) **No file I/O under the registry's lock.** A creation held the lock while it created
    and synced the journal and spawned the thread; a resume read the journal under it; a waiting
    game removed its journal under it. One address creating games in a loop could stall every
    game and connection task behind the fsyncs. Now `game::launch` runs a game's preparation on
    its own thread (a new game's secrets and journal, a paused one's journal read back and
    voided there if it does not restore), and a waiting game's journal goes after the lock. A game
    is listed before its journal exists; it sends nothing before it is synced, so a crash loses
    only a game nobody holds a token of. A creation whose secrets or journal fail now ends with
    `Goodbye { Shutdown }` after its `GameCreated`, and a resume whose journal no longer reads too
    (it was `Refused { NoSuchGame }`).
  - **The creator's or resumer's join first.** The registry handed a new or resumed game's inbox
    out before the binding connection's opening was queued, so a stranger's join could take a new
    game's creator's seat (ids are sequential), or a message could reach a resuming game before
    its resumer and pause it again. `registry::Opening::first` queues the opening in the fresh
    inbox under the lock.
  - **A refused join no longer restarts the lobby's deadline** (a connection sending a refused
    join every 50 s kept its place for ever, against `06-security.md`). `Frame::held` marks a frame
    to a connection that holds a seat or watches; the deadline restarts only after leaving a game
    the connection was in, and waits while it is bound.
  - **A paused game's deadline survives restarts.** `Writer::reopen` truncated every journal to
    its own length, which refreshes its last write on Linux, so a server restarted within 72 hours
    never voided a paused game; it truncates only a torn line now.
  - **No journal text in the log.** A void journal's reason quoted the replay parsers' errors,
    which may quote the seed; fixed text now (the header's names only a field).
  - **The replay written whole or not at all** (`archive::write`: a `.part` file, synced,
    renamed, the directory synced, then the journal goes); a `.part` a crash left is removed at
    the next start, and the journal writes the replay again.
  - **A game thread that panics** leaves the lobby (`game::Panicked`), its journal kept.
  Placed in the inbox (`QUEUE.md`, the user decides at the checkpoint): a spectator keeps a waiting
  game whose creator left (no deadline for waiting games); an archived replay goes out beside the
  outbound queue's byte bound; a non-host seat repeating `Pause` fills the viewers' records; the
  per-address limits behind a reverse proxy; `Drain`, `Stop` and `Tick` lost when a game's inbox
  is full. For the user: a replay's publicity is its host's setting alone.
- **Tests** (h3-server 43; the scoped verify with no failing step): a journal reopened whole keeps
  its last write; refused joins do not keep a connection in the lobby (fails if a refused join
  restarts the deadline, checked); a created game's creator joins before a stranger who asks for
  its seat at once. Three runs of the server's suite in a row passed.

### M3 T1 — The login on the wire, sans-I/O (2026-10-11)

- **The start.** The same session as M2's closure: #96 merged, the user passed M2's checkpoint
  (SRV-Q16) and refined M3 (SRV-Q17, ADR-0019 accepted); the branch restarted from `master`.
- **`account`** (`h3-protocol`, logic, 100 %): `is_login_name` (3–20 of `a–z 0–9 _ -`; the
  client lowercases, the derivation is salted with exactly this text), `is_display_name` (3–24
  ASCII letters, digits, `_ - .`, single inner spaces; *provisional*), `display_key` (ignoring
  case), `to_hex`/`from_hex` (lowercase only), `login_message` and `rekey_message`: a purpose
  with its version (`h3-login-v1`, `h3-rekey-v1`), then the server's id, the challenge, the login
  name and the display name (a re-key: the new key), each text after its length as 4 bytes
  big-endian, so no two lists of texts sign alike (*provisional*). No cryptography here.
- **The wire, version 12** (*queued for the M3 review*): `Welcome` gains `id` and `challenge`;
  `Login`, `Rekey`, `Rename`, `SetListed`, each with a `seq`, answered `Account { seq, display,
  listed }` or `Refused` (`BadLogin`, `BadName`, `NameTaken`, `NotLoggedIn`,
  `AlreadyLoggedIn`; the connection stays open).
- **The hello gate** (`connection`): built with the connection's challenge (the host draws it)
  and whether anonymous play is allowed; the welcome names the host (`Host { program, id }`). An
  account's request of the right form, in the right state, goes to the host as
  `Received::Account(AccountRequest)` with the bytes its signatures are over; a wrong form is
  refused there (`BadName`, `BadLogin`), a request before the login `NotLoggedIn`, a second login
  `AlreadyLoggedIn`; only `Connection::logged_in`, the host's yes, logs in. Without anonymous play
  `Join` and `CreateGame` wait for the login. The session ignores the account's messages.
- **The server** (*provisional*, until T2a): a challenge of 32 bytes of OS randomness per
  connection (none, no connection); the id `ws://<the address it listens on>`; every account's
  request refused `BadLogin` (SRV-STUB-1); anonymous play on. The workload's gate takes a fixed
  challenge.
- **Tests** (h3-protocol 68): the welcome's form; a login handed over with its signed bytes,
  logged in only on the host's yes, then no second; logins of the wrong form refused, changing
  nothing; the re-key, rename and preference only after a login; without anonymous play joins and
  creations wait for it; a property: whatever a client sends, it never logs itself in nor
  reaches the host with another account's request; the names' and hex rules; the signed bytes'
  layout; the new messages in the wire samples (round trips, the mutation tests), `Login`'s
  JSON; a third of the mutated frames reach a logged-in gate.

### M3 T2a — The Authenticator and the player log (2026-10-11)

- **The start.** #99 (M3 T1) and #100 had merged; the branch started from `master`, whose scoped
  verify passed (the workload without drift).
- **The crates** (the user, SRV-Q17): `ed25519-dalek` 3.0 (default features) and `argon2` 0.6
  (`alloc`, `zeroize`; without `password-hash`); 15 new crates built, all BSD-3-Clause or MIT OR
  Apache-2.0; `cargo deny check` passes (the duplicates it warns of were on `master`). The
  workspace's entries and `arch.toml` are shared: a commit of their own on the lane's branch.
- **`accounts`** (*provisional*): the `Authenticator` trait checks a signature
  (`verify(key, message, signature)`); `Ed25519` uses `verify_strict` (a small-order key or a
  non-canonical signature never holds; a test forges the identity key's signature that a lax check
  accepts). `Accounts::answer` takes the gate's `AccountRequest`: the signature first, then the
  player log under its own lock: a known login name with its key logs in (a display name it names
  is ignored), with another key `BadLogin`; an unknown one registers if it names a free display
  name (`NameTaken` otherwise), `BadLogin` without one. Re-keys, renames and the preference are
  refused `BadLogin` until T2b (SRV-STUB-2; SRV-STUB-1 removed).
- **`players`** (*provisional*): `<data>/players.log`, `{"Registered":{"login","key","display"}}`
  per line, owner-read-write, synced before the answer, the directory synced when the file is new;
  read back at start: a torn last line dropped and cut, a line that does not read, names that break
  the rules, a bad key or a name registered before stop the start, the error naming the line only.
  After a failed write nothing more is written (a torn line stays last) and registrations are
  refused; known accounts still log in.
- **`net`**: the connection's task answers an account's request through `spawn_blocking`, then
  `Connection::logged_in` on a login's yes; `Service` (was `Identity`) carries the program, the id,
  `anonymous` and the accounts. Failed logins (the host's `BadLogin`, not a request of the wrong
  form) count per address in a minute's window kept beyond the address's connections; beyond
  `[limits] failed_logins_per_address` (10), and for any login while the address is at it, the
  connection closes `RateLimited`; the address's other requests pass. `Rate` gained its window.
- **The configuration**: `[server] public_url` (the server's id; `ws://` or `wss://`, then 1–200
  visible ASCII characters; without it `ws://` and the bound address) and `anonymous` (default
  true); `[limits] failed_logins_per_address`; the example file.
- **The client**: `client::Key` (`derive`: Argon2id 64 MiB, 3 passes, 1 lane, version 0x13, salt
  `h3-account-v1:<login>`, then the Ed25519 key of the seed; `from_seed`, `public`, `sign`);
  `Client::welcome` and `Client::login` (signs the welcome's id and challenge). The test vector
  (`E16-design.md` §10) was computed independently with Python's `argon2-cffi` and PyNaCl, and
  `Key::derive` matches it.
- **Tests** (h3-server 52, +9): the test vector; registration, login on a new connection, a
  taken display name in another case, `AlreadyLoggedIn`, the accounts after a restart and the
  log's two lines; logins signed for another `public_url`, with another connection's challenge,
  by another key for the account's key, and by another account's key refused, changing nothing;
  without anonymous play creations and joins wait for the login, the lists do not; failed logins
  beyond the limit close the connection and bar the address's next one, requests of the wrong form
  do not count; the player log's reading (a torn line, five bad logs, a failed write); Ed25519's
  strictness; the configuration's new fields. The workload as at T1 (296 420 974 Ir, +0.14 %
  against its baseline; no login in it).

### M3 T2b — Re-keys, renames, the listing preference, the operator's rebind (2026-10-11)

- **The start.** #101 (M3 T2a) had merged; the branch restarted from `master` (the same tree
  #101's CI verified).
- **`players`** (*provisional*): the lines are typed events (`Event`: `Registered`, `Rekeyed`,
  `Rebound`, `Renamed`, `Listed`), a key through `Hex` (hex in the log, bytes in memory). One
  check against the accounts as they stand serves reading and writing: names by their rules, a
  login name registered once, a display name free or the account's own (ignoring case), the other
  events about a known account. `record` checks, appends and syncs, then updates the tables.
- **`accounts`**: `Rekey`: the new key's signature first, then under the log's lock the account's
  key's (both over the gate's `rekey_message`), else `BadLogin`; a re-key to the key it has
  answers without a line; connections logged in already stay so; a failed re-key does not count
  as a failed login (the connection holds the account already). `Rename`: `NameTaken` when another
  account holds the name; one's own in another case is free. `SetListed`. Each answered `Account
  { seq, display, listed }`; SRV-STUB-2 removed. `Authenticator::accepts`: whether a key can sign
  at all (a point that is not weak).
- **The lock**: `<data>/lock`, `File::try_lock` (Rust 1.89+), taken first by `start` and held by
  `Running` until `shutdown` returns (not by the connections' tasks, which may linger), and by the
  operator's command; a second server on the same data directory refuses to start ("in use").
- **The rebind**: `h3_server::rebind(config, login, key)`: the key checked first (64 hex digits,
  an Ed25519 key that is not weak), then the lock, then the account; a `Rebound` line; the
  account's display name. `h3-server rebind <config.toml> <login> <public-key>` (*provisional*),
  exit 1 with the reason, 2 with the usage.
- **The client**: `Client::rekey` (signs the welcome's id and challenge), `rename`, `set_listed`.
- **Tests** (h3-server 56, +4): a re-key before the login, by a key not the account's, signed for
  another challenge refused; after it only the new key logs in, also after a restart, and a
  re-key to the same key writes no line; renames unique ignoring case, one's own in another case,
  a freed name taken by another, a bad name refused by the gate, the preference, both after a
  restart, the log's six lines; the rebind refused while the server runs (and a second server
  refused), for an unknown account, a short key and the identity point, then the new key alone
  logs in after the start; the player log's events read back, a taken name and an unknown
  account refused unwritten, two more bad logs. The command smoke-tested by hand (usage, a bad
  key, an unknown account).

### M3 T3 — Seats bound to accounts (2026-10-11)

- **The start.** #102 (M3 T2b) had merged; the branch restarted from `master`. The baseline
  `verify --scope server` passed from a worktree of `master` (bench-check without drift).
- **The session core** (`h3-protocol`, *provisional*): the host's `Session::logged_in(conn,
  login)` before a connection's join (forgotten at its disconnect). A seat taken the first time by
  a logged-in connection belongs to the account (`SeatState::account`, the entry
  `Entry::TakenBy(side, login)`; `Entry` lost `Copy`); one taken anonymously stays the token's
  (`Entry::Taken`). Who may take a seat is one public function, `session::refusal((token, issued,
  account), token, login)`, which the registry also applies to a paused game's seats: an account's
  seat is its account's alone whatever the token (else `SeatTaken`, no new refusal); another seat
  is taken without a token the first time and re-taken with it. An account's `Joined` carries no
  token (the journal's first line still holds one per player seat, which takes nothing).
  `Session::accounts()` names the seats' accounts. The account is fixed at the first take: a
  connection that logs in after it opened a game plays that game as it opened it; one account may
  hold both seats (playing oneself from two devices).
- **The wire** (version 13): `GameInfo::seated` (`Seated { seat, display }`: the seats that belong
  to accounts, by display name; anonymous and bot seats unnamed), `GoodbyeReason::Rekeyed`; the
  docs of `Join`, `Joined`, `SeatTaken`, `Replaced`, `Phase::Paused`. `Lobby::seat(game, seated)`.
- **The journal** (format 4): `Taken { side, account }`, the account left out for an anonymous
  seat; a format-3 journal is void at start, as every older format is.
- **The server**: `ToGame::Open` and `registry::Opening` carry the connection's login (the gate's,
  when the task routes the join or the creation); the registry holds the `Arc<Accounts>`
  (`Service` shares it), a paused game the seats' accounts (`Hosted::Paused::accounts`). A game's
  thread names its seats in the lobby whenever their accounts change (`game::seated`, display
  names as they stand, looked up on the game's thread outside the registry's lock; a rename shows
  at the next change, *provisional*); a restored game is listed with them. `finish` lists a
  replay only if the host left the game open and every seat's account lets it (an account the log
  does not know cannot).
- **A re-key closes the account's other connections** (the T2b inbox item): `Service::sessions`
  maps (login, connection) to a `Notify`; a connection is entered before its login's key is
  checked (removed if refused), so a re-key committed after the check reaches it; a re-key that
  changed the key (`Accounts::answer` says so) notifies the account's other connections, which
  close `Goodbye { Rekeyed }`; a re-key to the same key closes nothing.
- **Tests**: `h3-protocol` 70 (+2): an account's seat re-taken by its account (replacing a holder,
  any token), refused to another account, an anonymous connection and the token; an anonymous
  seat's token for a logged-in connection; a login forgotten at the disconnect; the seats bound
  again by a restore; `refusal`'s cases; the lobby's names; the wire's new field and reason.
  `h3-server` 59 (+3, `tests/seats.rs`): a seat re-taken by its account after a drop, replaced,
  paused and resumed by the account, after a crash (the journal and the player log), and refused
  to another account, a stranger and the journal's token, then played to a replay that verifies
  and is listed; the lobby names an account's seat and not an anonymous one; an opted-out account's
  replay archived unlisted, listed after it opted in; a re-key closes the account's other
  connection, the re-keying one and other accounts stay, a re-key to the same key closes nothing.
  `crashed()` copies the player log too. The suite ran three times without a failure.
- **Not done here**: ADR-0018 point 8's text still names only the host's setting (ADR-0019 point
  2 carries the decision); its amendment joins `06-security.md`'s accounts in T5's shared change.

### M3 T4a — A game's bounds (2026-10-11)

- **The command limit** (*provisional*): `[limits] commands_per_game` (default 10 000, 1 to 1 000
  000). The configuration refuses a limit whose longest replay would not fit `outbound_bytes`
  (`REPLAY_BYTES_PER_COMMAND` 160: a command's line of at most 48 bytes and a checkpoint's of at
  most 32, doubled for the frame's JSON escapes; `REPLAY_HEAD_BYTES` 64 KiB for the rest): the
  default's bound is 1.6 MB of the 2 MiB. A game whose journal holds that many commands and that is
  not over is void after the step that reached it: its connections hear `Goodbye { CommandLimit
  }` after that step's frames, it leaves the lobby, its journal moves to `<data>/void/`, no replay
  is written, and connections on their way to it hear the goodbye too. The bots' answers within
  the step may take it a few commands past the limit; the check is the game thread's, not the
  session core's. A game restored past the limit (a crash after the step's sync, or a lowered
  limit) is void the same way at its start; a paused one when it resumes. The archived replay's
  size is now bounded by the limit too.
- **A waiting game without a held seat leaves** (the M2 review's item): after a step in which a
  waiting game holds no seat (its creator left), it leaves the lobby even while spectators are
  bound to it; they hear `Goodbye { Abandoned }`, a connection whose join it refused leaves on its
  own, connections on their way hear the goodbye, its journal goes (it never started).
- **The wire**: `GoodbyeReason::CommandLimit` and `Abandoned`; the version stays 13 within this
  pull request (T3's).
- **Tests** (`h3-server` 62, +3, `tests/bounds.rs`; the configuration test +3 cases): a bot game
  with a limit of 6 is void (the player and a spectator told, no end, not listed, the journal void,
  no replay); a game restored past a lowered limit is void at the start; a waiting game whose
  creator left with a spectator watching leaves (the spectator told, nothing void).

### M3 T4b — Behind a reverse proxy (2026-10-11)

- **Trusted proxies** (*provisional*): `[server] trusted_proxies` (addresses, none by default,
  compared after `to_canonical`). A connection from one of them holds a place in the total only
  (`Place::proxied`) until its handshake; the handshake's callback (`Forwarded`, tungstenite's
  `Callback`) reads the last entry of the last `X-Forwarded-For` header (the address the proxy
  itself saw; a client cannot forge it through a proxy that sets or appends the header), then the
  connection takes its place as that client, or as the proxy without a header that reads; a
  client without a place is closed after the handshake (a direct one before it). The standard
  `Forwarded` header is not read. A forwarded header from any other peer is ignored.
- **IPv6 by prefix**: every limit per address (connections, messages, failed logins, the lobby's
  games, the journal's origin) counts an IPv6 address by its /64 (`net::counted`), an IPv4-mapped
  one as its IPv4 address.
- **The operator's setup**: `crates/h3-server/README.md` (running, the data directory, the
  signals, `rebind`; on the internet: loopback, `public_url`, `trusted_proxies`, the proxy's
  WebSocket upgrade, header and idle timeout, with nginx and Caddy examples);
  `server.example.toml` names the field.
- **The client**: `Client::connect_forwarded(address, forwarded)` sends the header as a proxy
  does.
- **Tests** (`h3-server` 66, +4): two clients behind a trusted proxy each take their place with
  one connection per address, the last entry counts, an IPv6 client by its /64, the proxy itself
  without a header, a freed place taken again; behind an untrusted peer the header is ignored;
  `counted` and `forwarded_for` (several entries and headers, none, garbage) unit-tested.

### M3's reviewer agent (2026-10-11, before T5)

- **The review** (T5's item, run before this pull request so that its fixes ride it): a fresh,
  read-only agent over everything M3 changed in the two crates (`eb52954^1..`); 1 major, 5
  minor. Nothing found in: acting as another account, taking an account's seat with a token,
  forging an address through the proxy path, leaking tokens, keys or seeds, leaking counts,
  lost wakeups, the registry's lock rules, the format-4 restore, panics from client input.
- **Fixed** (*provisional* in their shape):
  - (major) **Account changes had no limit**: registrations, re-keys, renames and preferences
    each sync a line under the player log's lock, which every login waits on; one address could
    grow the log, its memory and the start without bound. Now `[limits]
    account_changes_per_address` (20 an hour, beyond the connections' lifetimes) counts every
    request that may write a line (a login naming a display name included); the one beyond
    closes its connection `RateLimited`; a login without a change still passes.
  - **The re-key's kick before anything else**: a connection's next client message is handled
    only after its kick is taken (`kicked`, a `Notify` permit taken without waiting): an old-key
    session's pipelined join or rename does not slip in. `Service::sessions` keeps the key each
    connection logged in with: only those with another key than the new one are kicked (a login
    with the new key racing the re-key stays), the re-keying one's key is updated; a failed login
    drops a signal a re-key gave it meanwhile.
  - **A refused change's line cut off again** (`players::append`): a write or sync that failed
    truncates the log back to its length before the line (and syncs) where the file allows, so a
    whole line the server refused does not come back at the next start.
  - **An abandoned waiting game tells every connection** (a joiner whose join is still on its
    way heard `TooSlow`); one whose join it refused leaves at its refusal and never reads it.
  - **A warning at start** without `public_url` when the server listens on an unspecified address
    or behind trusted proxies: logins sign an address no client dials.
- **Left** (the inbox): the player log's readers (logins, the games' lookups) still wait behind a
  write's sync, now bounded by the rate; a rename an old-key session sent before the re-key's
  commit, while its answer is off the task, can still land.
- **Tests** (`h3-server` 67, +1): an address's account changes beyond the limit close its
  connection, a registration too, a plain login passes; the configuration's range.

### M3 T5 — M3's closure (2026-10-12)

- **The user's choice**: T5 rides #105 as separate commits, the pull request labelled
  `cross-lane` (the shared docs and the client lane's inbox).
- **The shared change**: `docs/06-security.md`'s hosted games carry the accounts (the derivation,
  the signed login, the strict check, the re-key closing the other sessions, the rebind, the
  limits per address on failed logins and on changes), seats by account, the listing by the
  seated accounts, the command limit, the abandoned waiting game, the proxies and the /64, the
  exposure behind a TLS proxy, and "Not covered yet" brought up to date. ADR-0018 point 8 amended
  (a game a seated account opted out of is archived unlisted, the user's SRV-Q17) and point 6
  (seats bound to accounts since T3).
- **The client lane's inbox** (cross-lane): the key's derivation with its test vector and the
  cache, the login only for the id the client dialed, the re-key, seats by account, the new
  goodbyes.
- **M3's exit criteria**, each shown: the full verify green in CI on #105 (`h3-protocol` at
  100 %); over loopback (`h3-server`'s tests): the registration and login with the derived key
  and its test vector (`tests/accounts.rs`); logins for another server, with another challenge or
  by another key refused (same); a seat re-taken by its account from a new connection, without a
  token, also after a restart (`tests/seats.rs`); a re-keyed account logging in with its new key
  only (`tests/accounts.rs`); a display name changed and unique (same); anonymous play as
  configured (same); a game beyond its command limit void, a waiting game without a held seat
  leaving (`tests/bounds.rs`); the client's address counted behind a trusted proxy
  (`tests/proxy.rs`). And: `06-security.md` carries the accounts; the reverse-proxy setup is
  documented (`crates/h3-server/README.md`); the client lane's inbox has the login; the reviewer
  agent's major finding is fixed (its record above); the lane HANDOFF, this log and `QUEUE.md`
  are current.
- **The checkpoint**: the user's, SRV-Q18 (the review list, the inbox's placement, what comes
  next): passed 2026-10-12, the review list as built, the M3 inbox later, M4's refinement next
  (SRV-Q19, `docs/OPEN-QUESTIONS.md`).

### M4 T1 — Ladders, ratings and the results log (2026-10-12)

- **The configuration** (*provisional*): `[[ladder]]` (`config::Ladder`): `id` (1–32 of `a–z
  0–9 _ -`, the log's file name), `name` (a display name's rules), `scenarios` (relative to the
  file), `profile`, `accept_ms` (30 s), `grace_ms` (5 min), `grace_total_ms` (10 min; the user,
  SRV-Q19), `pauses` (0) and `pause_ms` (0, needed when `pauses` is not), `spectators` (true);
  the bounds checked as the limits are; ids distinct; none by default. The example has `arena`.
- **The pool**: each scenario compiled at start as an offer named by the ladder's id
  (`game::Offer::load`), one ruleset hash (`content_hash`) for all or the start refuses; not kept
  yet (T2 keeps it, when matches use it).
- **Glicko-2** (`h3-server`'s public `rating`): Glickman's 2013 steps, his defaults, τ = 0.5, ε =
  10⁻⁶, the Illinois iteration (`f(C)·f(B) ≤ 0`) bounded at 100 steps (*provisional*); `game`
  rates one game for both players, `period` the general step; `Standing` (the rating, games, wins,
  losses, draws, no-shows). `exp` and `ln` are banned in the workspace (ADR-0003 §3): two local
  `#[allow(clippy::disallowed_methods)]` with the reason, as `Cargo.toml`'s lint header allows
  adapters (review list). The test vector: Glickman's example within the paper's rounding (it
  rounds μ' first: 1464.06 for 1464.0507) and to 10⁻⁹ against an independent Python
  implementation of the same steps (1464.050670539, 151.516524124, 0.0599959843); a win between new
  players, a draw between unequal ones, a period without games; a rating's JSON reads back bit
  for bit.
- **The results log** (`ladders.rs`, *provisional*): `<data>/ladders/<id>.log`, lines `Season {
  season, ruleset }` (`0x` and 16 hex digits), `Rated { game, replay, accounts, outcome, forfeit,
  ratings }`, `NoShow { game, replay, accounts }`; checked on reading (seasons in order from 1, a
  game after a season, login names, two accounts, both known to the player log, valid ratings, no
  forfeit drawn) and before writing; the tables per season (the rows of the accounts that played
  or failed to show) and every account's latest rating across seasons. A new ruleset at start
  writes the next `Season` (the first at a ladder's first start): ratings and volatilities kept,
  deviations reset to 350; a log line says so. Only `Season` is written in T1; T3b appends the
  others. One lock for every ladder.
- **Shared log code**: `journal::open_log` and `journal::append_line` (open or create owner-only,
  a torn last line cut; a write synced, cut off again after a failure, and nothing written after)
  now serve the player log too.
- **The server**: `start` opens the ladders after the accounts (an account a line names must be in
  the player log); `Running::season` and `Running::standing` read them.
- **A flaky test fixed**: the verify at the session's start failed `a_restored_game_past_a_lowered_
  limit_is_void` under load: `game::too_long` took the game out of the lobby before moving its
  journal to `void/`, so a test that saw the game leave could still count no void journal. The
  journal now moves first, as on every other way to the void.
- **Tests** (`h3-server` 77, +10): the rating's four; the log's four (seasons by ruleset, the
  file's mode; rated lines read back bit for bit, a torn line cut, a season's reset; ten bad lines;
  a pool on two rulesets, an empty one); the configuration's checks and the example's ladder;
  end to end, a season per content change and an unknown account stopping the start.

### M4 T2 — The queue and the match (2026-10-12)

- **The queues, sans I/O** (`h3_protocol::ladder`, *provisional*): `Queues<C>` over the ladders
  (id, name, season, accept time); a logged-in account waits in one queue at a time (`NotLoggedIn`,
  `Draining`, `NoSuchLadder`, `AlreadyQueued` across ladders and connections, `InRankedGame` as
  the host says); `tick` gives up matches past their due time (who accepted waits again,
  `MatchCancelled`, its waiting time and its place kept; who did not is out, `Unqueued {
  Expired }`), then pairs each ladder's players greedily by rating (ties by order) within
  `window` of the shorter wait; `accept` (once per player; the second makes the `Matched`),
  `decline` and `dequeue` (`Dequeued`, the other back), `closed`, `drain` (`Unqueued { Draining }`),
  `list`, `waits` (by account), `holds` (by connection).
- **The wire** (version 14): five client messages, seven server messages, five refusals,
  `LadderInfo`, `UnqueueReason`, `GameInfo::ladder`; `Lobby::create_ranked` (the limit in total
  only); `Session::reserve` (a seat its account's before anyone joins, journaled as `Taken` with
  the account; nothing for a bot's seat or one taken already).
- **The server**: a pool's scenarios compiled as offers `<ladder>.<n>` (`game::Offer::ladder`),
  which a restore and a resume find beside the configured offers; the registry holds the queues,
  each queued connection's channel (`(text, still queued)`), the matches' games not over (a match
  over, or gone, frees its accounts) and the queues' clock; `registry::accept` creates an
  accepted match's game (a pool's scenario and the sides by `os_u64`, `Lobby::create_ranked`, the
  seats reserved on the game's thread and journaled before anything else) and tells both
  `MatchReady` (or `Unqueued { Full | Draining }`); `match_every` ticks the queues every second;
  a drain empties them. The connection's task reads its queue's channel, keeps the lobby's
  deadline waiting while queued (restarted when it no longer is), leaves its queue when it
  closes, and reads the account's rating (whole points) outside the registry's lock. A match's
  game that waits for its players neither leaves the lobby when idle nor as abandoned (until
  T3a's grace, SRV-STUB-3); nothing is rated (SRV-STUB-4).
- **Tests** (`h3-protocol` 70 → 78, `h3-server` 77 → 80): the queues' five (who waits; the window
  and its widening; both accepting; a decline, a `Dequeue`, a departure, an expiry with one or
  no acceptance; a drain) and a model-based property (random requests: no account matched twice
  or with itself, never outside the window, the counts agree; 5 000 cases once); the wire's
  corpora and JSON forms; `create_ranked`; `reserve` (refused to others, taken by the accounts,
  restored, not on a bot's seat). End to end: two players queue, accept, see their game listed
  with its ladder, are refused a second match meanwhile, play it out and queue again; a decline,
  an expiry and a departure put the other back, a queued connection outlasting the lobby's
  deadline; a match's game restored after a restart with its ladder, re-taken by account.

### M4 T3a — A ranked game's seats and forfeits (2026-10-12)

- **The session core** (`h3_protocol::session`, *provisional*): `GameConfig::ranked` (a `Grace {
  stretch_ms, total_ms }`, the ladder's `grace_ms` and `grace_total_ms`) makes a game ranked:
  `Session::new` makes `Entry::Ranked` its first entry (a restore whose configuration and journal
  disagree on it diverges at entry 0), no seat becomes the host (`Pause` and `SetSpectators` are
  rejected `NotHost`). The host's clock (`Session::elapse(now)`, milliseconds since the session
  began; a clock that goes back is ignored) times each player's seat while no connection holds
  it, from the session's start: a stretch beyond `stretch_ms`, or one that takes the seat's total
  beyond `total_ms` ("beyond": strictly more), forfeits; if the other seat is empty then too, both
  forfeit: void. `GameEnded` to both viewers and the spectators with `Outcome::Forfeit { side }`
  or `Outcome::NoShow {}`; `Entry::Forfeit(side)` for each. A stretch that ends with a join is
  `Entry::Absent(side, ms)`. Over by a forfeit (`Session::over`, `forfeited`), the battle stays
  where it stood: commands are rejected `BattleOver`, a join gets no turn and starts nothing, no
  absence counts any more (as after a natural end). A restore re-runs the entries: the totals
  kept, every player seat empty from the restore on (the downtime and a stretch running at the
  stop never count).
- **The wire** (version 15): `Outcome::Forfeit { side }` and `Outcome::NoShow {}`, only in
  `GameEnded` (the engine's results map as before); `NotHost`'s text names the ranked game.
- **The server**: a pool's offers carry the ladder's grace (`Offer::grace`), so new and restored
  games of a ladder are ranked; `empty_game_ms` does not apply to them, nor does the abandoned
  waiting game (SRV-STUB-3 resolved). The game's thread starts the session's clock when it begins
  the session (a restored one afresh) and moves it before every message, closed connection and
  tick; the sweep's period counts the ladders' grace (a quarter of the shortest deadline, 10 ms to
  1 min). A forfeit ends the game as any end does (`Session::over` where `result` was): verified,
  its replay written with the commands up to it (`winner = "none"`, which `h3 replay verify`
  accepts) and archived, its journal gone. The journal's lines `Ranked {}`, `Absent { side, ms }`,
  `Forfeit { side }` (format 5). At start, a restored match's game is in the registry's ranked
  games again (its ladder and accounts), so its players queue on that ladder only once it is over
  (T3b's item, taken here).
- **Tests** (`h3-protocol` 78 → 84, `h3-server` 80 → 85): no host; beyond the total in two
  stretches, the end for the absent seat's record, no command, pause, turn or stretch after it,
  the replay up to it verifying, a restore of the forfeited game; beyond one stretch; nobody comes
  (void, the spectator told); one beyond while the other is away within (void); a bot's seat never
  away; a forfeit of a game that never started; the totals across a restore; the clock only for
  ranked games and never back; a natural end stops the counting; the journal's new lines; the
  wire's corpora and JSON forms. End to end: a player back within the grace plays the game to its
  end (and pauses nothing alone); beyond the grace a forfeit, the replay verifying with `h3
  replay verify`'s check and both queuing again; beyond the total in three drops each within the
  grace; both away (void, a spectator told); the time away across a restart (the forfeit at the
  total's rest, the restored match refusing another queue).

### M4 T3b — The rated end (2026-10-12)

- **The start.** #109 (T3a) had merged; the scoped verify on `master` had no failing step, the
  workload unchanged (no re-baseline).
- **The session core** (`h3_protocol::session`, *provisional*): a ranked game's end, by the
  battle or by a forfeit (`Session::elapse`), is held until the host's verdict: `publish_replay`
  tells it, `withhold_replay` tells `GameEnded { Unverified }` (both now take the frames' vector;
  an unranked game's end is told at once, as before). `Session::rated(ladder, points, out)` sends
  each seat's viewer `Rating` (numbered, in its record, also for a seat no connection holds);
  spectators get none. `Lobby::skip(last)`: ids handed out in earlier lives.
- **The wire** (version 16): `Outcome::Unverified {}` (only in `GameEnded`), `ServerMessage::Rating
  { n, ladder, before, after }`, `Points { rating: i32, deviation: u32 }`.
- **The server**: `game::Stores` (the accounts, the ladders, the ids) replaces the accounts in
  the registry and the prepared games. The game's thread handles the end before it delivers the
  call's frames (the journal synced first): `finish` verifies, archives the replay, rates
  (`game::rate`: the forfeit a loss for who forfeited, both a no-show; the accounts' presence in
  the player log asked before the ladders' lock), and only then removes the journal; a replay or
  a result that does not write keeps it (the next start re-runs the game, which the log rates
  once). `Ladders::rate` writes `Rated` (Glicko-2 over the ratings as they stand, one game) or
  `NoShow`, checked against the tables and synced first; a game the log names already is not
  rated again; reading a log, a game named twice stops the start. `ladders::points` rounds a
  rating for the wire and the queue. `ids::GameIds`: `<data>/next-game` (decimal, written whole
  through `journal::replace`, which the archive's write now uses too), raised on each new game's
  thread before its journal exists and at start by every journal's id; the lobby skips to it.
  SRV-STUB-4 resolved (`registry::report`).
- **Tests** (`h3-protocol` 84 → 85, `h3-server` 85 → 90): the held ends (a forfeit told at the
  verdict with the rating after it, in the absent seat's record; a forfeit withheld, told
  `Unverified`; a no-show to a spectator; a retreat; a restored game's forfeit), `skip`; the wire's
  corpora and JSON forms. `next-game` read back, raised only, bad files refused; a game rated
  once as Glicko-2 rates it (a forfeit, a defender's win, a draw, a no-show, an unknown account and
  ladder refused, the log read back); a duplicate game's line. End to end: a played game rated for
  both (Glicko-2's values, the standings), the same after a restart, no id handed out twice; a
  forfeit rated; a no-show counted, unrated; a ranked game whose journal changed told
  `Unverified`, unrated, no `Rating`, its replay withheld; ids after a restart beyond a void
  journal's.

### SRV-Q20 — Server-side pings (2026-10-12)

- **The start.** #111 (T3b) had merged (`master`'s CI green); the user chose SRV-Q20's option (a)
  and asked for it now. The branch restarted from `master`.
- **The connection's task** (`net.rs`, *provisional*): `Transport::ping` (a WebSocket ping); the
  task pings every `ping_every_ms` and closes a connection it heard nothing from (any frame: a
  message, a ping, a pong) for `ping_timeout_ms`, with `Goodbye { IdleTimeout }` bounded by
  `LINGER` (the peer is most likely gone). The time the task spends writing (a frame, a ping) is
  added to the last time heard: a large frame to a slow client delays its pong, and the task reads
  nothing while it writes. The idle timeout is unchanged (messages only).
- **The configuration**: `[limits] ping_every_ms` (15 000; 0: no pings and no timeout) and
  `ping_timeout_ms` (30 000), checked as the other timeouts and the timeout longer than the
  period; the example, the module's sample and the README's proxy step name them.
- **A test client** answers pings only while it reads (`client.rs` says so): an end-to-end test
  whose client waits unread longer than the timeout loses it (the first silent-seat test lost Bob
  while it waited a matcher's second for Ann).
- **Tests** (`h3-server` 90 → 92): a connection answering pings stays, one that falls silent is
  pinged, told and closed within the timeout and leaves its game (a fake transport); the bounds'
  cases and the period's check; end to end: a ranked seat whose client stops reading forfeits in
  seconds (the idle timeout would have held it 5 minutes), its connection gone.

### M4 T4 — Leaderboards (2026-10-12)

- **The start.** #112 (SRV-Q20) had merged; the scoped verify on `master` had no failing step,
  the workload unchanged (no re-baseline).
- **The wire** (version 17, *provisional* names): `ClientMessage::Leaderboard { seq, ladder,
  season, page }` (`season` `None` or left out: the current one; `page` from 0), answered
  `LeaderboardPage { seq, ladder, season, total, rows }`; `ClientMessage::Standing { seq, ladder
  }`, answered `OwnStanding { seq, ladder, season, total, row }` (`None` before the account's first
  rated game of the season); a row is `LeaderboardRow { rank, display, rating: Points, games,
  wins, losses, draws, bot }`; `Refusal::NoSuchSeason`. The gate (`Connection::receive`) refuses
  `Standing` without a login (`NotLoggedIn`), on any server; a page needs none.
- **The server** (`ladders.rs`, *provisional*): each season keeps its leaderboard in order beside
  its table (`Season::order`, a set of `(Ahead(rating), login)`: by the rating itself, the highest
  first, `f64::total_cmp`, ties by login name), moved by every `Rated` line, read back or new; a
  row enters at its first rated game, so an account with only no-shows is not listed. A page is
  `PAGE` (50) rows from the `page`-th, none past the last; one's own rank counts the rows ahead.
  `Ladders::page` and `own` answer (a `Board`), refused `NoSuchLadder` or `NoSuchSeason`. The
  connection's task reads them through `spawn_blocking` (the ladders' lock and the player log's
  wait on their syncs) and names each row's display name as it stands (`Accounts::displays`, one
  lock for a page). The bound is the page and the connection's message rate, as for the replay
  lists. `bot` is false until T5 brings the server's bots.
- **Tests** (`h3-protocol` 85 → 86, `h3-server` 92 → 94): the gate's login for one's own row; the
  wire's corpora and JSON forms (a leaderboard's ask without a season, a page). A leaderboard of
  120 accounts paged by rating, ties by login name, a row moved by a second win, ranks, a no-show
  unlisted, the refusals, the same pages read back and an empty next season. End to end: a
  forfeit's two rows on the first page, a page past the end, an unknown season and ladder; one's
  own row (none for an account without a game, refused anonymous); a rename shown; after a
  content change the new season's page empty and the first season's kept.

### M4 T5 — Bots on ladders (2026-10-12)

- **The start.** #114 (T4) had merged; the branch restarted from `master` (T4's scoped verify
  had no failing step, the workload unchanged; #113, the adventure lane's, touched no server
  path).
- **The queue** (`h3_protocol::ladder`, *provisional*): `Ladder::bots` (`LadderBot { login,
  display, rating }`) and `bot_after_ms`; a waiting entry's connection is `Option` (`None`: a
  bot). After the players are paired, each one left who waited `bot_after_ms` gets the bot
  nearest in rating (`min_by_key`: the first of equally near ones, the configuration's order),
  whatever the window: people first. The bot accepted at once (`MatchFound` to the player only);
  a decline, an expiry or a drain tells only the player, and the bot, never queued, plays any
  number of games at once. `Queues::rate(ladder, login, points)`: the host passes the bots'
  ratings before each matcher's turn (`registry::match_every` reads them from `Ladders::bots`
  outside the registry's lock). `Matched::players` names a bot with no connection.
- **The session** (`Session::reserve`): a bot's seat belongs to its bot account too
  (`Entry::TakenBy`, as a player's): `accounts()` names it, the lobby lists it, the end rates
  it, and a restart restores it from the journal's tokens (`None`: the bot plays) and entries.
  Journal format unchanged (5): the lines are the same.
- **The wire** (version 18): `LadderInfo::bots` (the ladder's bots' display names),
  `Seated::bot`; `LeaderboardRow::bot` set since T4.
- **The server**: `[[ladder.bot]]` (`login`, `display`, `kind`, `random` the only one;
  *provisional*), `bot_after_ms` (60 000; 0 to a year); names checked as an account's, a bot once
  per ladder, the same entry on every ladder that lists it. The player log's `Bot { login,
  display }` line: an account with no key (`Account::key` is an `Option`), which no login
  matches, no re-key or `rebind` changes (`a bot's account`), and whose names are taken as any
  account's. At start (`Accounts::bots`, before the ladders read their logs) a bot the log does
  not know is registered, one whose display name changed renamed; a player's account under a
  bot's login stops the start. `Prepared::new` takes the seats the server's bot plays (the
  offer's, or the match's bot), so a bot match's journal holds no token for it. The lobby's and
  the leaderboard's flags come from `Account::is_bot`.
- **Found on the way:** a match's game never named its accounts in the lobby (T2 on): the game's
  loop names seats only when they change, and a match's were set before its first look. The game
  now names them right after the reservation.
- **Tests** (`h3-protocol` 86 → 87, `h3-server` 94 → 96): the nearest bot after the wait (a
  tie, a rating changed on one ladder only, a decline, an expiry, people first, a drain); a bot
  seat's account reserved, restored, never taken by a connection; the wire's corpora and JSON
  form. The player log's bot accounts (names, rename, no key change, read back); the
  configuration's bot checks. End to end: a lone player matched with the bot after the wait, its
  seat named and flagged in the lobby, the game rated for both, the bot's row flagged; no login
  as the bot or under its name; a renamed bot at the next start; a bot under a player's login
  stops the start; a human match's seats named in the lobby.

### M4 T6 — The pause by agreement (2026-10-12)

- **The start.** #118 (T5) had merged; the branch restarted from `master` (the shared and client
  changes merged meanwhile touch no server path). During the task the user asked whether a
  player may opt out of bots (SRV-Q21): per ladder as built, plus a per-queue opt-out and
  `MatchFound` naming a bot; queued as T5b, outside this task.
- **The session** (`h3_protocol::session`, *provisional*): `Grace` carries the ladder's `pauses`
  and `pause_ms`. `ProposePause` from a running ranked game's seat (else `NotRanked`; the phase
  as for `Pause`): once per seat until the game moves on and while no proposal is open
  (`AlreadyAsked`), within the seat's pauses (`NoPausesLeft`) and the longest (`BadPauseLength`,
  also 0); `PauseProposed` to every viewer. `AnswerPause` answers the other seat's open proposal
  (`NoProposal` otherwise, also one's own); `PauseAnswered` to every viewer, and agreed:
  `Entry::PauseAgreed(side, ms)` (the proposer's pause counted) and the host's pause (`halt`). A
  command applied clears the asks and lapses an open proposal. `agreed_pause()` names the pause
  a restored session is in; `hold(ms)` keeps every absence from counting before then (a seat
  away, or gone later); `wake()` runs a game in an agreed pause again (`Resumed`), its players
  away. An unranked game's other seat asks (`Pause`) once until the game moves on (the M2
  review's repeated `PauseAsked`). `tell_all` sends to every viewer (`tell_end` uses it).
- **The wire** (version 19): the two client messages, `PauseProposed`, `PauseAnswered`, five
  `RejectCode`s. **The journal** (format 6): `PauseAgreed { side, ms }`.
- **The server**: a game that pauses by agreement is listed paused with its due time
  (`Hosted::Paused::due`, its pause after the journal's last write; at start, after a restart,
  the same); a restore holds the absences for the rest of it (`game::resume`). The sweep
  (`Registry::sweep`, now given the shared registry) wakes a paused game whose due time came:
  restored on its thread without a join (`Registry::restorer`, shared with `bind`;
  `Prepared::woken`), it reports itself running; `paused_game_ms` voids only games without a due
  time. The example configuration shows `pauses` and `pause_ms`.
- **Tests** (`h3-protocol` 87 → 90, `h3-server` 96 → 99): the session's bounds, the asks, a
  decline, a lapse, an agreement told to the spectator too, the restore in its pause, the hold
  for a player back early and for one gone again, the pauses used; a wake with neither back
  (void) and one in no agreed pause (nothing); an unranked game's other seat asking once; the
  wire's corpora and JSON form; the journal's line. End to end: a proposal declined, a second
  ask refused, a pause agreed and both back in time playing on to the end; a player not back
  after the due time forfeiting after the pause and the grace; neither back: woken and void (a
  no-show each), not voided by a shorter `paused_game_ms`.

### M4 T5b — A player's bot opt-out (2026-10-12, SRV-Q21)

- **The start.** #120 (T6) had merged; the branch restarted from `master` (no other lane's
  change since touches a server path). CI did not start on `master` for #118 or #120 (no
  minutes; `05-testing.md`, "When CI cannot run"): the baseline was the full local verify.
- **The queue** (`h3_protocol::ladder`, *provisional*): a waiting entry carries `bots` (whether
  it may play a bot), kept through a match given up; the matcher gives a bot only to a player
  left unpaired after `bot_after_ms` who did not opt out. An opted-out player pairs with people
  by the window as before, however long it waits. `Queues::queue` takes `(seq, ladder, bots)`.
- **The wire** (version 20): `ClientMessage::Queue::bots` (`Option<bool>`, `#[serde(default)]`:
  left out or `None`, bots welcome); `ServerMessage::MatchFound::bot` (the opponent is a bot;
  only the player of a bot match hears it `true`). No stored preference: the client sends it
  with each `Queue`. Journal format unchanged (6).
- **The server**: `net` passes `bots.unwrap_or(true)` to `Registry::queue`.
- **Found on the way:** `a_game_past_its_command_limit_is_void` (M3 T4a) assumed no battle ends
  within its limit of 6 commands, but a game's seed comes from the OS: the scoped verify drew an
  attacker that won with the 6th. Its limit is now 1 (the stacks start out of reach: no first
  command ends a battle). Its sibling `a_restored_game_past_a_lowered_limit_is_void` plays 3
  turns before its restart, two of them strikes; a battle won in those would fail it too (not
  seen; left as is).
- **Tests** (`h3-protocol` 90 → 91, `h3-server` 99): an opted-out player waiting ten minutes on a
  ladder with bots while another plays one, then matched with a person, declined and waiting on
  for people; the bot tests' `MatchFound` flagged; the property test now on a ladder with bots
  and random opt-outs (a bot match only for a waiting player who did not opt out; checked to fail
  with the opt-out ignored); the wire's corpora and JSON forms (`bots` left out). End to end:
  the lone player's bot match told `bot: true`; an opted-out player hears nothing for twice
  `bot_after_ms` and still waits (`Dequeue` answered `Dequeued`).

### M4's reviewer agent (2026-10-12, in T7)

- **The review**: a fresh, read-only agent over everything M4 changed in the two crates
  (`7a71b90..`, about 7 000 lines); no major finding, 7 minor. Found sound: account-bound seats
  (no token or other account takes one, live or paused), bot accounts refused at login, re-key and
  `rebind` and their seats unjoinable, no token, seed or key leaked; the ranked end's order
  (verify, archive, rate, journal) and a game rated once; forfeits and no-shows by the server's
  clock only; the matcher (window, bots, opt-out, the answers, a drain; another connection of the
  account cannot answer for the first); the pause by agreement's rules and due time; seasons,
  leaderboards and Glicko-2's bounded loops; the lock rules; no panic from client input or from a
  corrupted ladder log, `next-game` or journal; bounded queues, proposals and channels.
- **Fixed** (*provisional* in their shape):
  - **A player who declines again and again kept another from every match** (and from its bot:
    people are paired first). Who gives a match up (a decline, an expiry, leaving it) may queue
    again at once but is paired with no one on that ladder for `ladder::BARRED_MS` (60 s), per
    ladder and account; no wire change, the player is not told. The property test: no one paired
    while barred (checked to fail with the bar ignored).
  - **A player queueing right after its game's end was refused `InRankedGame`**: the end was told
    before the game left the registry's ranked games. `Registry::ended` now runs on the game's
    thread before the end's frames go out.
  - **A lost `next-game` handed out ids a ladder's log names**, and such a game went unrated
    (`rate` took it for one rated already). At start the ids go beyond the logs' highest game
    (`Ladders::last_game`).
  - **The ladders' lock on async tasks**: the queue's rating and the matcher's bots are read
    through `spawn_blocking` (`rate` holds the lock across a sync).
  - **The design note said a match's seats are journaled before `MatchReady`**: they are
    journaled on the game's thread before any join reaches it; the players hear `MatchReady` as the
    game is created. The note and `06-security.md` now say so (a game whose journal cannot be
    created ends, and its players find it gone).
- **Left** (the inbox, the checkpoint's question SRV-Q22): an agreed pause resumed early by one
  player is held for the other in memory only, so a restart before its due time lets the absent
  player's grace start then (a forfeit may come before the pause's due time); finished, unrated
  ranked journals re-run at a start are rated in the order their threads run, not the order the
  games ended (only after a ladder log's write failed); a game that stops because its journal no
  longer writes leaves the ranked games while its journal stays, so its players may start another
  match on the ladder and the next start restores both.

### M4 T7 — M4's closure (2026-10-12)

- **The start.** #124 (T5b) had merged; the branch restarted from `master` (#123, the adventure
  lane's, touched no server path; the scoped verify had no failing step, the workload unchanged).
  **The user's choice**: the shared doc and the client lane's inbox ride T7's pull request as
  separate commits, labelled `cross-lane` (as at M3's closure).
- **The end-to-end test** (`two_players_queue_accept_and_play_a_rated_battle_through_a_restart`):
  two players queue, accept and take their seats; after a command each (the duel's stacks start
  out of each other's reach and one strike cannot end it, so the battle always goes on; each seat
  counts the `Events` it hears to stop after the same command, `plays_until`) the server stops; at
  the next start the match is listed with its ladder, both re-take their seats by account and play
  to the end, rated as Glicko-2 rates it between new accounts; a player queues again at once; a
  restart without `next-game` reads the same ratings back and hands out a higher game id.
- **The shared change**: `docs/06-security.md`'s hosted games carry the ladders and ranked games
  (who queues and what a client names, the seats bound before any join, no host and the pause by
  agreement, absences on the server's clock, results rated by the server alone after the
  verification and written once, the bots' keyless accounts and flags, the opt-out, the
  leaderboards' names) and the server's pings among the bounds; "Not covered yet": the
  verification item done, stalling (M5), a forfeit within a ping of a server-side drop, and
  collusion between accounts added.
- **The client lane's inbox** (cross-lane): queueing with the opt-out, the accept step,
  reconnecting by account and the pings, a ranked game's ends and `Rating`, the pause by
  agreement, the leaderboards.
- **M4's exit criteria**, each shown (`h3-server`'s `tests/ladders.rs` unless named): the full
  verify green, `h3-protocol` at 100 % (CI does not start for lack of minutes: the full local
  verify, the pull request); two clients queue, accept and play a rated battle through a restart,
  rated as Glicko-2 computes (its test vector, Glickman's example, in `rating.rs`) and the same
  after another restart (the test above); a declined or expired match puts the other back
  (`a_decline_an_expiry_or_a_departure_puts_the_other_back`); a second queue, a second ranked game
  on the ladder and an anonymous queue refused (`two_players_queue_accept_and_play_their_match`); a
  player back within the grace re-takes the seat by account, beyond the grace or the total
  forfeits (`a_player_back_within_the_grace_…`, `a_player_away_beyond_the_grace_…`,
  `a_player_away_beyond_the_total_…`); both away void, unrated
  (`both_players_away_the_game_is_void`); a ranked game that does not verify unrated and its
  players told (`a_ranked_game_that_does_not_verify_…`); a content change starts a season with the
  deviations reset (`a_ladder_starts_a_season_when_its_content_changes`; the new season's first
  rated game, its `Rating` from the carried rating and a deviation of 350, in
  `the_leaderboard_pages_by_season_…`, added in T7); the leaderboard pages
  by season (`the_leaderboard_pages_by_season_…`); a bot account plays a rated game, flagged
  (`a_lone_player_plays_the_ladders_bot_…`); a pause by agreement within the ladder's bounds and
  a forfeit after its due time (`a_pause_declined_then_agreed_…`,
  `a_player_not_back_after_an_agreed_pause_forfeits`). And: `06-security.md` carries the ladders;
  the client lane's inbox has the ladder's messages; the reviewer agent's findings fixed or left
  for the checkpoint (above; no major one); the lane HANDOFF, this log and `QUEUE.md` are current.
- **Tests** (`h3-protocol` 91 → 92, `h3-server` 99 → 100): the bar (a decline, an expiry, a
  departure; per ladder); the property test's bars; the restart test; the leaderboard test's new
  season played against a bot (Ann and Bob, 324 points apart after a forfeit, would wait 150 s
  for the window).
- **The checkpoint**: the user's, SRV-Q22: passed 2026-10-12; the review list as built; the
  pause's hold across a restart a small task first (T8); the other two review items later; M6's
  refinement next (M5 waits for E08's increment 3).

### The logs on Windows (2026-10-12, after M4's checkpoint)

- **The report** (the battle lane's handover, the user's `verify --with-original-data` on Windows):
  `h3-server` failed clippy (`unused variable: new` in `journal::open_log`, read only under
  `cfg(unix)`) and two tests (`rated_games_are_read_back_bit_for_bit`,
  `the_accounts_are_read_back_and_a_torn_line_is_dropped`). CI runs on Linux only.
- **The cause**: `open_log` opened the player log and the ladders' results logs with
  `append(true)`, which on Windows grants no write-data access: cutting a torn last line
  (`set_len`, also `append_line`'s cut after a failed write) is refused. The game journals were
  not affected (`Writer::reopen` opens with `write(true)`).
- **The fix**: the logs open with `write(true)` (no append mode) and write at their end: `open_log`
  seeks to it after the cut, `append_line` seeks to it before each line and takes the length
  before it from there (one writer per log: the data directory's lock and the log's own). `new`
  exists only on Unix.
- **Shown in the sandbox**: `rustup target add x86_64-pc-windows-gnu` works here; `cargo clippy
  --target x86_64-pc-windows-gnu --workspace --all-targets -- -D warnings` reproduced the clippy
  error on `master` and is clean with the fix. The tests are not: under Wine (`wine64`,
  `gcc-mingw-w64-x86-64`) the Windows build's tests fail on `master` and with the fix alike, all
  at the same step, a test's `remove_dir_all` while its log is still open (Wine lacks the POSIX
  delete Windows 10 gives Rust; the user's Windows run passed those), and Wine does not refuse a
  cut through an append handle either: it cannot show the fix. The user's Windows verify confirms
  it (the cause as Rust's Windows `OpenOptions` gives it: append mode drops `FILE_WRITE_DATA`,
  which `set_len` needs).

### M4 T8 — An agreed pause's hold across a restart (2026-10-12, SRV-Q22 (b))

- **The start.** #130 (T7) and #134 (the logs on Windows) had merged; the branch restarted from
  `master`. The scoped verify on `master` had no failing step, the workload unchanged (no
  re-baseline).
- **The defect** (M4's reviewer): a seat taken during an agreed pause before its due time held
  the other seat's absence until then in memory only (`Session::hold`); the journal's `Resumed`
  ended the agreed pause, so a restart before the due time restored a running game with no hold,
  and the absent player's grace ran from the restart (a forfeit before the pause's due time).
- **The session** (`h3_protocol::session`, *provisional*): an early resume (a join in an agreed
  pause with time left) journals `Entry::Held(rest)` after `Resumed` and keeps the rest
  (`held_pause()`); a restore re-runs it. `elapse` ends the hold when the clock first reaches the
  due time: `Entry::Held(0)`, and a later restore holds nothing. A new agreed pause (both seats
  were back to agree) ends an earlier hold, at a restore too (`agree`).
- **The server**: `game::resume` re-applies `agreed_pause()` or else `held_pause()`, less the time
  since the journal's last write, as T6 does for a pause. **The journal** (format 7):
  `Held { ms }`. No wire change.
- ***Provisional*, when a journaled hold ends** (the queue's question): at the clock's first move
  past the due time (a message, a closed connection, the sweep: at most a sweep late). A restart
  before that write, or after commands written since the hold's line, re-applies a little more
  than the true rest (the time between the hold's line and the last write is not subtracted):
  never less, so neither the downtime nor the restart counts against a player. Exact rests would
  need the journal to carry times; not worth it for a pause of minutes.
- **Tests** (`h3-protocol` 92 → 93, `h3-server` 100 → 102): the session's hold journaled at an
  early resume, none for a seat back at the due time, re-applied at a restore (Bob back beyond the
  grace from the restart, before the due time: no forfeit), over at the due time and not
  re-applied after, a player not back by it forfeiting after the grace, a new pause ending it; the
  journal's line. End to end (`tests/ladders.rs`): an agreed pause, Ann back at once, a restart
  before the due time; Bob back 2.5 s after it (the grace is 1 s) plays on to the end, no
  forfeit; Bob not back forfeits only after the pause's due time and the grace. Both end-to-end
  tests fail without the re-apply (checked).

### M4's review items (2026-10-12, after T8, the user's request)

The user asked for the open items that need no decision of theirs, in T8's pull request. Taken
from the inbox: the two review items SRV-Q22 left for later, and two older ones inside the lane.
Left: the turn timer (M5, a game rule), the player log's lock split and the lobby's names after a
rename (both "if it shows"), a separate rate for automated clients (a configuration choice).

- **A match that stops with its journal kept** (M4's reviewer): a game whose journal no longer
  writes, or whose thread panics, left `Registry::ranked` while its journal stayed, so its players
  could start another match on the ladder and the next start restored both. `Registry::stopped`
  takes it out of the lobby and keeps the entry until the next start; `game::stop`, the panic
  guard and a restore whose journal does not reopen for writing (it names the entry, which a start
  does not hold yet) use it. *Provisional*: also for a match that never started (its journal goes
  at the next start; its players wait for it). Tested on the registry (`Registry::stopped`): a
  journal that stops writing cannot be made portably in a test (the sandbox runs as root, and the
  game holds its file open).
- **The rating order at a start** (M4's reviewer): `registry::restore` re-runs each journal to its
  end, or until its game is listed again, before the next, sorted by game id (by name, "10" came
  before "9"); finished, unrated ranked games are rated in that order. The server does not listen
  yet, so no restored game moves meanwhile. Tested end to end: two journals of ids 9 and 10
  sharing a player, taken as the players sat down and given forfeits by hand, rated in id order at
  a start (the test fails with the order by name).
- **A drain or a stop a full inbox missed** (M2 T5e's review): the registry remembers that it
  drains or stops (`Registry::signal`), and a game's thread asks under the lock it takes after each
  message anyway, then acts as on the signal (`game::shut_down`, shared with the signal's arm). A
  missed `Tick` still waits for the next sweep. Tested on the registry (a full inbox misses both
  signals, the registry tells them).
- **A rename racing a re-key** (M3's review): the connection's task passes the key it logged in
  with (or re-keyed to, its `Sessions` entry) with the request, and the player log refuses a rename
  or a listing change (`BadLogin`) unless that key is the account's, under the log's lock. A stale
  re-key was refused already by its old-key signature. Tested on `Accounts`: the old key, and none,
  refused; the new key changes. `docs/06-security.md` says so (its own commit; the pull request
  labelled `cross-lane` with the user's consent, 2026-10-13).
- **Found, left in the inbox**: a ranked game over whose ladder log did not write keeps its
  journal, but its players may play another match before the next start rates it.
- **Tests** (`h3-server` 102 → 106).
- **The merge of `master`** (2026-10-13): R01's content split (cross-lane, protocol v21) touched
  the server's code; the merge was clean but for the queue's inbox. Its note asked for the design
  note's offer line: §3 and the server card (§12) name `battle_ruleset_hash` beside
  `ruleset_hash`.

### M6's refinement (2026-10-13, SRV-Q23)

- **The start.** `master` at #139 (the client lane's); the branch had nothing of its own. CI runs on
  `master` fail within seconds without starting (minutes or billing, `05-testing.md` "When CI
  cannot run"); the scoped verify had no failing step (exit 2, the workload within its band).
- **What the refinement weighed** (the draft, the design note's §12, the code's hello and login):
  the login's bytes already start with their purpose (`h3-login-v1`), so the welcome, the card and
  the list get theirs and no signature serves two purposes; the welcome already names an `id` that
  a login signs, so a hello naming the dialed address lets one server answer at several addresses
  without changing ADR-0019 point 1; a card signed over its own text travels verbatim (no canonical
  JSON); a server dials only configured addresses (peers, lists' servers), so a card can never make
  it connect anywhere; the vouching chain is checkable from the cards alone.
- **The licences behind the TLS question** (`cargo info`): `ring` 0.17 `Apache-2.0 AND ISC`;
  `aws-lc-sys` 0.46 `… AND Apache-2.0 AND …`; `webpki-roots` `CDLA-Permissive-2.0` (not in
  `deny.toml`); `openssl` `Apache-2.0`; `rustls` 0.23, `rustls-native-certs`, `tokio-rustls`,
  `rustls-webpki`, `rustls-pki-types` with ISC or MIT among their options; `graviola` 0.4.1 and
  `rustls-graviola` 0.4.0 `Apache-2.0 OR ISC OR MIT-0`. A scratch crate (the server's `tokio` and
  `tokio-tungstenite` with `rustls-tls-native-roots`, `rustls` without default features plus
  `std` and `tls12`, `rustls-graviola`, `rustls-native-certs`) adds 13 crates on Linux (`graviola`,
  `rustls-graviola`, `rustls`, `rustls-webpki`, `rustls-pki-types`, `rustls-native-certs`,
  `openssl-probe`, `tokio-rustls`, `getrandom` 0.3, `once_cell`, `subtle`, `untrusted`, `zeroize`),
  no build script needing a C compiler, and `cargo deny check licenses` passes with the repo's
  `deny.toml`; `ring` sits in its lock file but in no target's build. Graviola: x86_64 with `aes`,
  `ssse3`, `avx`, `avx2`, `adx`, `bmi2`, `pclmulqdq` (CPUs since about 2014), aarch64 with `aes`,
  `sha2`, `pmull`, `neon` (a Raspberry Pi 5, not a 4); "very new, exercise due caution".
- **The user's answers** (SRV-Q23, asked in the session; the pros and cons of peering and lists
  asked for and given): a TLS client now, `rustls` with `graviola`; mDNS later; a list names
  servers by key and address; ADR-0021 in this pull request, a `shared:` commit, `cross-lane`.
- **Recorded:** ADR-0021 (from the draft: points 1–3 sharpened, 4 the TLS client, 5 the lists, 6
  mDNS among the not built), ADR-0018 point 2 (TLS outbound only), the ADR index; `E16-design.md`
  §12 rewritten as the working detail; the epic's decisions, milestone, M6 tasks and exit criteria;
  the queue's T1–T6; the threat table's leaked data directory (one secret from M6).

### M6 T1 — The server key and the signed welcome (2026-10-13)

- **The start.** #146 (the refinement) merged; the branch restarted from `master`; the server's code
  was the code of the session's first scoped verify (no failing step).
- **The key** (`h3-server`'s `key.rs`): `<data>/server.key`, the Ed25519 seed as hex and a newline,
  written at the first start from OS randomness (a `.part` file with mode 0600 on Unix, synced,
  renamed, its directory synced), read at every start after the data directory's lock; a file that
  does not read or parse stops the start and is never written over. `ServerKey`'s `Debug` names the
  public key only. *Provisional*: the file's name and form.
- **The welcome** (`h3-protocol`): `Hello { …, nonce, address }`, `Welcome { …, key, signature }`;
  `account::welcome_message(id, nonce, challenge)` starts with `h3-welcome-v1`. The gate signs
  through the host's `Signer` (`Debug + Sync`: a host's connections share it; this crate still holds
  no cryptography); `Host` gains `addresses` (the server's other addresses) and `signer`. The id is
  now the connection's (`Connection::id`, set by the hello): the dialed address when it is one of
  `addresses`, else `Host::id`; logins and re-keys sign it. A hello whose nonce is not 32 bytes of
  hex closes with `Protocol`. Wire 24 (the merge of `master`: 22 and 23 went to the obstacle
  types, CLT-Q3, and the rear hex, BAT-GEO-002, while T1's pull request was open).
- **The server**: `[server] addresses` (each checked as `public_url` is), the `Service` holds the key
  and the addresses; the workload's gate signs with a stand-in (`bench::Unsigned`: it counts the
  session core, not Ed25519).
- **The test client** names a fresh nonce and the address it dialed (`ws://` and the socket, or
  `dialed`), verifies every welcome's signature by its key (strict Ed25519, the accounts'
  `Authenticator`) and refuses a key other than one it `pin`ned; `server_key` reads it. Every
  end-to-end test's hello now checks the signature. `crashed()` copies the key with the journals.
- **Tests** (`h3-protocol` 93 → 96, `h3-server` 106 → 111): the welcome's bytes; the id by the
  dialed address and the login's bytes with it; a hello without a nonce; the key written once with
  its mode, read back, never replaced, signing as Ed25519; end to end, the key across a restart, a
  pinned key refused by another data directory's server, a key file that does not parse stopping
  the start, logins at each address and one signed for another address refused. Clippy also for
  `x86_64-pc-windows-gnu`.
- **Docs**: the operator's notes (the key, `addresses`), `server.example.toml`; the design note's
  T1 bullets pruned.

### M6 T2 — The card and `Describe` (2026-10-13)

- **The start.** #149 (T1) merged; the branch restarted from `master`. The session's baseline verify
  ran while the first edits were in the clone: its clippy, test and coverage steps failed on
  them alone (the new `Describe` arms), every other step passed; the closing verify is the record.
- **The card** (`h3-protocol`'s `card.rs`, the wire's `ServerCard`, `CardContent`, `CardLadder`,
  `CardPeer`, `SignedCard`): `account::card_message(text)` starts with `h3-card-v1`; `card::sign`
  signs the card's JSON text through the host's `Signer`; `card::read` takes the cryptography as a
  function and checks the text's length (8 KiB), the card's form and bounds (`card::fits`), the
  signature by the card's own key, `expires` after the reader's clock and `issued` at most
  `SKEW_MS` (10 minutes) ahead; `card::is_address` is the configuration's address rule, now shared.
  The counts are `Option`s: `null` on a card that keeps them off. *Provisional*: the bounds (16
  rulesets of 16 packs, 16 ladders, short texts 64 bytes, description 1024, operator 256, no
  control characters but the description's `\n`) and `CardError`'s grain.
- **`Describe { seq }` → `Described { seq, card }`**, after the hello and without a login on any
  server (the gate passes it to the host; before the hello it closes with `Protocol`). The
  connection's task answers it off the task (`spawn_blocking`: the counts take the registry's and
  the ladders' locks). Wire 25.
- **The server** (`h3-server`'s `card.rs`): `Cards` holds the card without its times, counts and
  ladders, built at start from the configuration (`[server] name`, the server's id by default;
  `description`; `operator`; the addresses as the welcome knows them) and each ruleset of the offers
  and the ladders' pools once; a card that would break the bounds with the largest counts stops the
  start. It re-signs when asked and the last card is a minute old (`RESIGN`), issued at the wall
  clock (`wall_ms`, the one `SystemTime` behind a local `allow`) and expiring `[discovery] card_ms`
  later (a day; 10 minutes to 30 days); `[discovery] counts` (on) names the open connections (the
  listener's counts, now the `Service`'s), the games waiting or running and each ladder's
  current-season leaderboard size (`Ladders::cards`). `card::verify` is `card::read` with strict
  Ed25519, for the test client and T3. The configuration checks `name` (1 to 64 bytes, one line),
  `description` (1024, lines), `operator` (256, one line), at most 7 `addresses` (8 on the card).
- **`h3-server card <config.toml>`** asks the running server at its `bind` (loopback for an
  unspecified address) with the test client, pinning the data directory's key
  (`ServerKey::existing`, which never writes one), and prints the signed card's JSON. *Provisional*:
  a card of the running server (its counts live) rather than one signed offline, which would need
  the data directory's lock and the ladders' logs.
- **The test client**: `describe(seq)` checks the card (`card::verify`) and that its key is the
  welcome's.
- **Tests** (`h3-protocol` 96 → 106, `h3-server` 111 → 121): the card's bytes, its text and its
  reading back, with and without counts; a tampered text (another name, another spacing), a forged
  or malformed signature, a card naming another key, expired, issued beyond the skew (and at the
  epoch's end), too large, not a card, each bound broken and all at their limits; addresses; a
  proptest that reading never panics; `Describe` before and after the hello. In `h3-server`: a card
  verifies by its key within its times, tampered, expired, early and forged cards fail, no counts
  when off, the re-signing after a minute, a card beyond its bounds stops the start; end to end, a
  client's card (the configuration's texts and addresses, the offers' rulesets once, the ladder,
  the counts, a day's expiry, the same card within the minute though a game opened), a server
  without counts or a name, `Describe` before the hello, `h3-server card` on `0.0.0.0` and against
  another data directory's key or a stopped server, and the configuration's bounds.
- **Docs**: the operator's notes (the card, `h3-server card`), `server.example.toml`, `config.rs`'s
  format, the design note's T2 bullet pruned to what later tasks need, §2's catalogue, the M6
  review list.

### The rear hex's journals (2026-10-13, the inbox; with T2)

The battle lane's #150 made a two-hex stack's position its rear hex (BAT-GEO-002, wire 23) and
re-recorded its golden replays, with no format of its own bumped. A journal holds its game's
commands in the replay's text form, so one written before would re-run its moves and attacks at
other hexes: caught at its next checkpoint hash, or restored into a game its clients never saw
where a server runs without checkpoints. `JOURNAL_FORMAT_VERSION` 8 (no line changed) makes such a
journal void at start. Records need no bump: their manifest names the protocol (22 or lower) they
were written in; the archive's older replays are the battle lane's replay format's concern.

### M6 T3 — Peers, the fetch and `ListPeers` (2026-10-13)

- **The start.** #155 (T2) merged; `master` equalled the session's branch. The baseline
  `verify --scope server` passed every step (exit 2 by design; `bench-check` too: no drift). The
  scoped verify's `check-lane` runs without `--cross-lane`, so on this branch (a `shared:` commit)
  it fails by design: `cargo xtask check-lane --lane server --cross-lane` and the full verify,
  which skips `check-lane`, stand in (a trap of every lane, for the global HANDOFF's next
  `shared:` change).
- **The `shared:` commit** (label `cross-lane`, the refinement's consent, SRV-Q23): the workspace's
  `rustls` 0.23 (no default features; `std`, `tls12`), `rustls-graviola` 0.4, `rustls-native-certs`
  0.8, `tokio-tungstenite`'s `rustls-tls-native-roots`, and `tokio-rustls` 0.26 without default
  features for the tests' TLS endpoint (a dev-dependency; already in the tree through
  `tokio-tungstenite`, so no further crate); `arch.toml`'s externals. No `ring`, no `aws-lc` in the
  graph (`Cargo.lock` lists `ring` as an optional dependency no enabled feature turns on: `cargo
  tree -i ring` prints nothing); `cargo deny check` passes with `deny.toml` unchanged. Against T2's
  `master` the TLS client adds 16 crates to `h3-server` on all targets (rustls, its pki types and
  webpki, untrusted, once_cell, tokio-rustls, graviola and its rustls glue, rustls-native-certs
  with openssl-probe, schannel, security-framework and its sys crate, core-foundation and its sys
  crate, bitflags); 10 on Linux.
- **The protocol** (wire 26): `ListPeers { seq, from }` → `Peers { seq, from, total, entries }`,
  `RelayedCard { card, via, list }` (`via` empty and `list` `None` until T4 and T5), pages of
  `card::PAGE` (50), after the hello without a login (the gate passes it to the host, as
  `Describe`). `card::chain(cards)`: each card names the next one's key among its peers, the check
  a client runs from the relaying server's card through the cards `via` names to the card itself.
- **The configuration**: `[[peer]] key address` (at most 64, as a card names; an Ed25519 key that
  `accepts` takes in lowercase hex; a WebSocket address; no key or address twice), `[discovery]
  refresh_ms` (an hour; a minute to a day) and `timeout_ms` (10 s; a second to two minutes). The
  server's own key as a peer stops the start (`Peers::new`). The card's `peers` are the
  configured ones, in order (`Cards::new` bounds the card with them, as T2 planned).
- **The fetch** (`h3-server`'s `peers.rs`): one task (`fetch_every`, aborted at shutdown) runs a
  round at start and every `refresh_ms` from the round's start, one peer at a time in the
  configuration's order, each fetch a task of its own in a `JoinSet` within `timeout_ms`. A fetch
  dials through the test client (`Client::dial`: `connect_async_tls_with_config` with
  `Connector::Rustls` for both schemes, so tokio-tungstenite never builds a default config, which
  would panic without a process-wide crypto provider; the WebSocket's message and frame bounded by
  `FRAME`, the client's reading by a byte budget, `FETCH`), says hello as the server's id, checks the
  welcome's key against the configured one and its id against the address dialed (ADR-0021 point
  1), then `describe` (the card holds and is the welcome's key's). A failure is logged (`h3-server:
  peer <address>: <why>; …`) and the peer's last good card stays; each round prunes the expired
  ones; `ListPeers` serves the cards that hold at the wall clock. The TLS client (`peers::tls`) is
  graviola's provider over the OS roots (`peers::os_roots`, read only when a peer is at `wss://`,
  its errors logged); `launch(config, roots)` takes the roots from the tests. Graviola asserts its
  CPU features at its first use (x86_64: AES, PCLMULQDQ, BMI1, ADX, AVX, AVX2; aarch64: NEON, AES,
  PMULL, SHA-2) and does not build elsewhere: the fetch's own task turns such a panic into a
  logged failure (`panic = "unwind"` in every profile). *Provisional*: the frame bounds, the
  hello's name, the log lines, one fetch at a time, that a peer of another protocol version is
  skipped (its `Goodbye { Protocol }`).
- **The test client**: `list_peers(seq, from)`, and `relayed(own)` pages through every card, checks
  each (`card::verify`) and its chain from the server's own card, and refuses a `via` until T4.
- **Tests** (`h3-protocol` 106 → 107, `h3-server` 121 → 132): the messages round-trip; `ListPeers`
  needs the hello, not a login; chains hold card by card and fail backwards or with a link left
  out. In `h3-server`: the cache's pages (60 cards: 50, then 10, none past the end) and expiry
  (served and pruned), `Peers::new`'s refusals; over loopback: A relays B's and C's cards, as
  signed and in the configured order, its card names both, pages from 1 and past the end; B down
  keeps its card through two rounds; a wrong key, a server whose id is another address (also when
  dialed by another name) are not relayed and the fetch says why; fake peers (a gate of
  `h3-protocol` behind a test socket) whose card is forged, tampered, expired, issued a day ahead or
  another server's are not relayed; B behind a TLS proxy (tokio-rustls with a test certificate
  for localhost and 127.0.0.1, signed by a test root; P-256, valid until 2126, made with openssl,
  in hex in the test with the commands) is fetched at `wss://` with the test root, refused with
  the OS roots (`UnknownIssuer`) and unreachable at `ws://`; the configuration's bounds; the
  server's own key stops the start; the OS roots read.
- **Docs**: the operator's notes (peers, the TLS client and its processors), `server.example.toml`,
  `config.rs`'s format, the design note's §2 catalogue and §12 bullets, the M6 review list.

### M6 T4 — Beyond the peers: the chain (2026-10-13)

- **The start.** #160 (T3) merged; `master` equalled the session's branch. The baseline
  `verify --scope server` ran on `master` in a worktree of its own (the session's edits had begun
  in the clone): every step passed (exit 2 by design).
- **No wire change.** `RelayedCard::via` (T3) now carries keys; `PROTOCOL_VERSION` stays 26, and
  `h3-protocol` is unchanged (`card::chain` is the check a client and the server run).
- **The configuration**: `[discovery] depth` (2; 1 to 4: 1 the peers' own cards, each more one hop
  further) and `cards` (200; 1 to 1000: the most cards relayed, and the most of a peer's cards a
  fetch reads).
- **The fetch** (`peers::fetch`): after `Describe`, below `depth` it pages the peer's `ListPeers`
  (`peers::listing`, at most `cards`). Its frames are bounded by a page of 50 cards (`FRAME`, 50 ×
  (2 × 8 KiB + 1 KiB) + 1 KiB, 851 KiB) and its bytes by `budget(cards)` (two card frames for the
  welcome and the card, a frame per page: 3.4 MiB at 200). `vouched` keeps a card the peer relays
  when it has no `list` (a card the peer fetched for a list is not vouched by it: T5 decides), its
  `via` leaves this server's chain within `depth`, it holds (`card::verify`), and the chain from
  the peer's card through the cards its `via` names (found among the peer's other cards that
  hold, the first of a key) to it holds (`card::chain`); a card that does not hold or whose chain
  does not is dropped, one log line each (`h3-server: peer <address>: a card it relays is
  dropped: …`, naming the card's key, which its bounds check made hex). A card beyond `depth` is
  not read. A failed fetch keeps the peer's last good card and relays; a round prunes the expired.
- **What `ListPeers` serves** (`peers::relay`, rebuilt after each fetch and each round): the peers'
  own cards in the configuration's order, then their relays by the length of their `via` (none
  first), the peers in order, each peer's in its order; each key once (the first is the shortest
  chain), the server's own never; at most `cards`. Each chain is checked again against the cards
  served before it (`through`): a link's card may be another fetch's (a peer that is also relayed
  deeper, or a shorter chain), and the client checks the chain with the card served, so a chain
  the served cards do not carry is not served. A card is served until the first card of its chain
  is no longer served (`until`, the least of the chain's), so a client always finds every link.
  Mutual peers are quiet: a peer relaying this server's card or cards through it costs no log.
- **The test client**: `relayed` reads every page, checks each card, then each chain from the
  server's card through the relayed cards its `via` names; it no longer refuses a `via`.
- **Tests** (`h3-server` 132 → 140): `relay`'s order, depth and `cards` bounds; a key once by its
  shortest chain and the own card never; a chain re-checked against the served cards (a peer's own
  newer card that stopped vouching); the chain's expiry and the prune; `vouched` on signed cards
  (a forged card, two chains that do not hold, a link not relayed, a list's card, depth 1 to 3,
  expiry). Over loopback: A → B → C → D with C and D shut down before A starts (A learns of them
  through B alone and dials B only): depth 2 serves B and C via B, depth 3 adds D via B and C, and
  `cards = 1` B alone, each chain checked by the test client; a fake B relaying cards outside its
  vouching and a forged one: the fetch drops them and says why, A serves B, C and D; the test
  client refuses a chain its server does not vouch for, one whose voucher does not, and one through
  a card not relayed; the configuration's new bounds. T2's test of an unknown `[discovery]` key
  named `cards`, now a key: it names `hops`.
- **Docs**: the operator's notes (beyond the peers, the memory bound), `server.example.toml`,
  `config.rs`'s format, the design note's §12, the M6 review list.

### M6 T5 — Lists (2026-10-13)

- **The start.** #165 (T4) merged; `master`'s code was the code CI's full verify (run 529) and the
  session's scoped verify had passed on, so the baseline stood. The branch restarted from it.
- **The protocol** (wire 27): `ServerList { key, name, issued, expires, servers: [ListedServer {
  key, address }] }`, travelling and stored as `SignedList { list, signature }` (the text as
  signed after `h3-list-v1`, `account::list_message`); `Lists { seq }` → `ListsAre { seq, lists
  }`, after the hello without a login (the gate passes it to the host). `h3_protocol::list`:
  `fits` (the key, a name of 64 bytes, issued before it expires, at most `MAX_SERVERS` 128 servers
  with keys and WebSocket addresses, no key twice), `sign`, `read`, `names` (the list names the
  card's key at one of the card's addresses: a client's check of a listed card), `MAX_LIST_BYTES`
  (32 KiB), `MAX_LISTS` (16). `card::read` and `list::read` share `card::read_signed` behind a
  crate-private `Signed` trait (the bounds, the parse, the key's signature, the times); the error
  stays `CardError`.
- **The server** (`h3-server`'s `lists.rs`): `<data>/lists/*.json`, each a `SignedList` as JSON,
  read at the start of each round (so at start) in their names' order; a file that does not read
  or is larger than a list's escaped text, a list that does not hold (`lists::verify`, Ed25519),
  a second list of one key and any beyond 16 are skipped, one log line each (`h3-server: list
  <file>: <why>; skipped`; never the file's text). `peers.rs`: `Peers::list` takes the lists and
  picks the servers the round fetches: each key once (the first list's), neither the server's own
  nor a peer's, at most `cards` less the peers; one named again at the same address keeps its last
  good card. After the peers, each is fetched as a peer is (`fetch_logged`, the fetch's task and
  log shared with the peers'), at depth 1. `relay` puts their cards after the peers' own and
  before the peers' relays, marked with the list's key (`RelayedCard::list`), each until it or its
  list expires (a chain through a listed server's card works as through any served card). `Lists`
  answers the lists that hold, as signed. *Provisional*: the files' form and order, the place
  after the peers' own cards, the bounds.
- **The tools**: `h3-server list-key <file>` (`ServerKey::at`, the server key's file code for any
  path: written if there is none, mode 0600, never replaced; its public key printed) and
  `h3-server sign-list --key <file> <list.toml>` (`name`, `valid_days` 1 to 366, `[[server]] key
  address`; issued now; the signed list printed as JSON, refused out of its bounds).
- **The test client**: `lists(seq)` (each list checked), and `relayed` checks a card marked with a
  list against that list among the server's (`list::names`) instead of a chain.
- **Tests** (`h3-protocol` 107 → 114, `h3-server` 140 → 146): the bytes a list signs, its text,
  tampered, foreign and too large lists, its times, its bounds, `names`; `Lists` needs the hello,
  not a login; the messages round-trip. In `h3-server`: the listed servers' place, marking and
  expiry in `relay`; which servers a round fetches (not the server, not a peer, once, within
  `cards`; a card kept at the same address); `Lists` gives the lists that hold; over loopback, a
  server with six files in `<data>/lists/` (a good list naming B; an expired, a forged, a
  malformed one and a second of the good one's key naming D; a `.txt` file) serves the good list
  and relays B marked with its key, D never, the test client checking each; the files removed,
  the next rounds serve neither; the tools make a key and a list that verifies and refuses lists
  out of bounds; 17 files read as 16, a file too large or a directory skipped; the test client
  refuses a card marked with a list its server does not give.
- **The client workspace** builds against wire 27 (`clients/scripts/check.sh`, standing in for its
  CI job, which runs because `h3-protocol` changed).
- **Docs**: the operator's notes (lists, the tools), the design note's §2 and §12, the M6 review
  list.

### M6's reviewer agent (2026-10-13, in T6)

- **The review**: a fresh, read-only agent over everything M6 changed in the two crates
  (`dfaeeee^..4319b74`, about 3 000 lines of code and 2 400 of tests); two major findings, five
  minor. Found sound: the signed texts' purposes (`h3-login-v1`, `h3-rekey-v1`, `h3-welcome-v1`,
  `h3-card-v1`, `h3-list-v1`) distinct and none a prefix of another, texts length-prefixed;
  `verify_strict` with weak keys refused; lowercase hex only, duplicate JSON fields refused; the
  times, the skew and the size checked before the parse; the fetch's checks (the configured key,
  the id the address dialed, the card the welcome's key, the answers' `seq` and `from`), no
  redirect followed, no address read from a card dialed; `vouched`, `relay` and the chains (the
  first link the peer's own card, every link relayed, no chain through the server's own key, a
  chain served until its first card expires, the shortest kept); `depth`, `cards`, the lists' cap
  and a second list of one key; every fetch bounded by its frames, its total and its time,
  `listing` always ending; no `unwrap`, slicing or overflow on external data; no lock across an
  await; the key file (0600, through `.part` and a rename under the data directory's lock, never
  replaced, the seed in no log or `Debug`).
- **Fixed** (*provisional* in their shape; `h3-server` 146 → 149 tests, each new one checked to
  fail with its fix taken out):
  - **Major: a `wss://` server named by a list only was never fetched.** The OS roots were read
    at start only when a `[[peer]]` was at `wss://`, so a server whose lists alone named `wss://`
    servers (the operator notes' own example) had none: every such fetch failed `UnknownIssuer`.
    `peers::Tls` now reads them at the first fetch at `wss://`, a peer's or a listed server's (a
    list may name one at any round); a server that fetches none still never reads them. The
    `wss://` test also lists the proxied server on a server without peers.
  - **Major: a listed server's card that did not name the listed address was relayed.** A server
    on a list answering there with its key but signing a card whose addresses leave that address
    out passed the fetch, and a client's `list::names` check then failed: the test client's
    `relayed` refused the whole listing. A listed server's card is now taken only if it names the
    address its list names (`Kept::addresses`); a peer's card stays checked by its chain alone (a
    client checks no address there). The test: a fake server, fetched fine as a peer, listed and
    not relayed.
  - **The card's default name**: the id, which a `public_url` of up to 206 bytes makes longer than
    a name's 64, stopped such a start; now the id's first 64 bytes (the test: a server with a
    100-byte host name starts, so named).
  - **The start's bound check** counted each ladder's players as they stood (0 at a first start):
    a card within 144 bytes of 8 KiB started and later outgrew it. The largest card now counts
    them at their most (the test: the largest card that starts, signed with every count at its
    most, fits).
  - **A peer's unexpected answer to `ListPeers` was logged whole** (up to a frame of 851 KiB a
    round): now its kind only.
  - **A list's file**: `metadata` then an unbounded `read_to_string`, so a pipe or a link to a
    device named `*.json` in `<data>/lists/` blocked the round or read without end. Now only a
    regular file is opened, read within its bound (the test: a link to `/dev/zero`, on Unix).
- **Left**, with the reason: the test client (`client.rs`) pages `relayed` without a cap and
  accepts a welcome naming another id than the address dialed; it is a test tool (its `login`
  says so), the client lane's note says what a client must check, and the server's own fetch
  checks both. A list's files are read on the fetch task, not through `spawn_blocking`: at most 16
  small files a round, an hour apart by default.

### M6 T6 — M6's closure (2026-10-13)

- **The start.** #167 (T5) had merged; the branch was `master`. CI's runs on `master` fail within
  seconds without starting (billing); the full local verify on `master`'s code passed every step
  (`check-docs` failed only on this task's first HANDOFF note, then trimmed).
- **The shared change** (`cross-lane`, as at M3's and M4's closures, the cut the user accepted at
  M6's refinement): `docs/06-security.md`'s hosted games carry server identity and discovery (the
  key and the welcome, purposes and strict checks, the card's bounds and times, the dials only to
  configured addresses, the chain checked at the fetch and again as served, lists and `Lists`, a
  peer's list its own), the data directory's one secret beyond the games, the outbound TLS client
  under "Exposure"; "Not covered yet": no revocation of a server key or a list, a card's claims
  its server's word, no version negotiation, discovery's answers to any client after its hello,
  graviola's platforms.
- **The client lane's inbox** (`cross-lane`): what a server browser needs: the hello's nonce and
  address, the welcome's check and pinning, `Describe` and `card::read`, `ListPeers` with
  `card::chain` and `Lists` with `list::names`, trust as the player's (chains, lists, contact on
  the player's choice), versions.
- **The design note's threat table**: a hostile peer, a forged or stale list.
- **The operator's notes** already carried the key, peers, lists and the TLS client's platforms
  (T1–T5); the default name's cut added.
- **M6's exit criteria**, each shown: the full verify (the full local verify, the pull request; CI
  does not start), `h3-protocol` at 100 %; a client verifies the welcome at each of the server's
  addresses and refuses a changed key at a pinned address, the key survives a restart, a key file
  that does not parse stops the start (`tests/identity.rs`, `key.rs`'s
  `a_key_file_that_does_not_parse_is_never_replaced`); a card verifies, a tampered, expired or
  foreign card fails (`card.rs`'s `a_card_verifies_by_its_key_within_its_times`, `h3-protocol`'s
  `tests/card.rs`, `tests/peers.rs`'s `a_forged_expired_or_foreign_card_is_not_relayed`); A → B →
  C, each link checkable, A never dialing C
  (`a_server_relays_what_its_peers_relay_to_depth_each_card_with_its_chain`,
  `a_client_checks_each_chain_a_server_relays`); a card a peer serves outside its vouching, or
  forged, dropped and logged (`a_card_a_peer_relays_outside_its_vouching_or_forged_is_dropped`);
  the depth and count bounds (`peers.rs`'s `the_cards_relayed_reach_depth_hops_the_nearer_first`,
  the depth test above); a peer at `wss://` with a test certificate and root
  (`a_peer_behind_a_tls_proxy_is_fetched_at_wss`); a list's servers marked with its key, an
  expired or forged list skipped (`tests/lists.rs`'s
  `a_lists_servers_are_relayed_marked_with_its_key_and_bad_lists_skipped`). And: `06-security.md`
  carries discovery; the client lane's inbox has what a server browser needs; the operator's notes
  carry the key, peers, lists and the TLS client's platforms; the reviewer agent's major findings
  fixed (above); the lane HANDOFF, this log and `QUEUE.md` are current.
- **The checkpoint**: the user's, SRV-Q24: passed 2026-10-13 ("The choices are sound, all
  accepted"): the review list as built, the reviewer's two left findings as left. Next, the user's
  wish: what the ecosystem around the server needs, talked through with the user (M5 waits for
  E08's increment 3). #169 (CLT-Q1, wire 29) merged meanwhile; `master` merged into #173.
