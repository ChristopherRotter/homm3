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
