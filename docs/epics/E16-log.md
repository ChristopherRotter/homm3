# E16 log: done tasks in full

The full record of E16's finished milestones and tasks (what was built, agent decisions marked
provisional, measurements, the review lists). **Not start-of-session reading:** the epic file keeps
one line per done milestone; read the matching section here when a task builds on an earlier one
or a decision is questioned. Decisions live in ADR-0018 and the ADR drafts; this log only records
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
