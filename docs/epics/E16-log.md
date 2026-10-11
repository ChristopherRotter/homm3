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
