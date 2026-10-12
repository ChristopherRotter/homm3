# Server lane — HANDOFF

The server lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **M1–M3 closed** (SRV-Q12, SRV-Q16, SRV-Q18). **M4 refined (2026-10-12, SRV-Q19):** ADR-0019 points 4–9 decided; the detail in `E16-design.md` §11; the tasks in `QUEUE.md`; no turn timer in M4 (a game rule, M5). **M4 T1 (#106), T2 (#108), T3a (#109), T3b (#111), SRV-Q20 (#112) and T4 done** (T4: leaderboards by season, one's own row; wire 17). **Next: M4 T5 or T6** (both ready), then T7.
- **ADR-0018 point 5 as amended:** the mapping's matches over the engine's enums are exhaustive, and since T4b its patterns over the observation's structs too; a new battle variant or observation field fails `h3-protocol`'s build until mapped (global HANDOFF trap).
- **The 100 % gate** covers `h3-protocol` (logic); `h3-server` is an adapter outside it, tested end to end.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, 17 since M4 T4's leaderboards; bumped on every wire change), `RECORD_FORMAT_VERSION` (`h3-protocol`, 2 since M2 T5c), `JOURNAL_FORMAT_VERSION` (`h3-server`, 5 since M4 T3a's `Ranked`, `Absent`, `Forfeit`; a journal of another format is void at start).
- The lane owns (`lanes.toml`) `h3-protocol`, `h3-server`, `benches/server/` and E16's files; the workspace files stay shared. **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does. ADR-0019 (identity) was accepted at the M3 refinement; the discovery draft is decided at M6's.
- **Ids:** questions `SRV-Q<n>`, quirks `SRV-QUIRK-<n>`, stubs `SRV-STUB-<n>`. ADR drafts go to `docs/lanes/server/adr-drafts/` and get a number when the user accepts them. Branches: `server/<task>`.

## Autonomy contract (confirmed by the user, 2026-10-09)

- **Agent decides, marked provisional:** the inner structure of the server crates, names, test layout, error texts, log format; anything a later task can change without touching another crate. Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency, a message shape inside an accepted ADR, a configuration format. List them in the milestone's review list (epic file) and go on.
- **Stop the line:** seams and contracts (`h3-contracts`, the engine API the server calls), heavy dependencies (beyond the decided `tokio` features, `tokio-tungstenite`, `ed25519-dalek` and `argon2` (SRV-Q17): another runtime, an HTTP framework, a TLS stack, other crypto crates, a store), licence questions, changes to the security model (ADR-0008), anything touching another lane's paths. Write the question (`SRV-Q<n>`), stop that task and take the next independent one.

## Standing traps

- `h3-protocol` is `no_std`: take `serde` (`derive`, `alloc`) and `serde_json` (`alloc`) with `default-features = false` in its own manifest, not `workspace = true` (the workspace entries enable `std`).
- `postcard` (and any format that is not self-describing) cannot decode `#[serde(tag = ...)]`, `untagged` or `flatten`: keep the wire types in serde's default enum form (SRV-Q5).
- A 64-bit integer on the wire travels as a string (JavaScript clients lose precision above 2^53).
- A wildcard arm after every variant of an enum still counts as one uncovered region (llvm-cov), and `#[coverage(off)]` is unstable: no fallback arms in `h3-protocol` (ADR-0018 point 5 as amended).
- `prop_assert!(matches!(x, P { .. }))` needs a message: the macro reads `{ .. }` as a format argument.
- A test client that picks uniformly among the legal commands rarely ends a battle of the `full` scenario (mostly moves; the defender's stacks regenerate): pick among the strikes when there are any (`tests/session.rs`, `play`).
- A test `Client` answers pings only while it reads: one unread beyond `ping_timeout_ms` is dropped, and its late pong draws a reset that drops its unread frames.
- Automated clients answer every turn at once and exceed the default 20 messages per second and the address's 40: end-to-end tests that play raise both (`tests.rs`, `fast()`).
- A frame the game marks `left` is acted on only if the task passed the game exactly that many messages (`Bound::passed`): a join pipelined behind a refused one may take a seat, and leaving then would drop its `Joined` and the seat's token.
- A game leaves the lobby only under the registry's lock, after its bound connections are counted down (`registry.rs`): deciding by the game's own connection list races a join that is already queued.
- A socket closed over unread data is reset, and the reset makes the peer drop what it has not read yet (a `Goodbye`): the transport's close reads on for the peer's close (`net.rs`, `LINGER`). The flooding test catches a regression only in most runs, not all.
- A journal's line holds tokens: errors from reading one name the line and column, never its text (serde_json's messages quote values); a log line never carries a hash either (`journal.rs`).
- No file I/O under the registry's lock (T5e): a game creates or reads its journal on its own thread (`game::launch`), and a new or resumed game's own opening is queued under the lock (`registry::Opening::first`) before anything else reaches it.
- A finished game's replay is written by `game::finish` into the archive's directory its spectator setting names (`archive::dir`) and must then join the registry's archive (`Registry::archived`): a replay written elsewhere is neither listed nor trimmed.
- `PROPTEST_CASES` applies to every proptest of a crate: name the test for a longer mutation run (`cargo test -p h3-protocol mutat`).
- A game's frames go through `game::Outbox` (`offer`, or `last` for a goodbye), never a bare channel: the connection's task subtracts every frame's bytes as it takes it out, so a frame queued around the count wraps it, and the queue then refuses every frame.
- Write-ahead order in a game's thread: drain the session's entries and sync them before `deliver`; anything new that changes what a restart must redo needs an `Entry`, or the restore diverges from what clients were told (T3's resume is one: `Entry::Resumed`).
- The observation's mapping (`map/mod.rs`) names every field of the engine's types in patterns, without `..`: a field left out gets `name: _` under a comment with its reason. A field read through a method (`s.attack_now`) still needs its pattern entry.
- The workspace's clippy bans `std::time::Instant` and `SystemTime` in every crate: the server uses `tokio::time::Instant`, and a file's age is `fs::metadata(p)?.modified()?.elapsed()` (a `Duration`, no type named).
- A `watch::Receiver` whose sender is gone makes `wait_for` fail at once, again and again: a select arm on it must then never resolve (`net::stopped`), or a dropped `Running` would stop every connection.
- A paused game has no thread: the registry's `Hosted::Paused` holds its tokens, and `registry::bind` restores it for a join. A game's thread ignores `ToGame::Open` (nothing changed until the join), or a resuming game, still paused, would pause again before its join.
- An account's request is answered through `spawn_blocking`, the player log under its own lock (`accounts.rs`): never take the registry's lock there, and keep the signature check before the lock. The lock waits on the log's syncs: a game's thread asks `Accounts::get` outside the registry's lock (`game::seated`), never a connection's task.
- A running server holds `<data>/lock` (`Running`, until `shutdown` returns): a test that starts a second server on the same data directory must shut the first down; the crash tests start on a copy (`crashed()`).
- `Key::derive` costs 64 MiB and about a second in a debug build: tests use `Key::from_seed`, the test vector alone derives.
- `f64::exp` and `ln` are banned workspace-wide (ADR-0003 §3): Glicko-2 calls them behind a local `allow` (`rating.rs`); a rating is written to the ladder's log and read back, never recomputed.
- A queued connection hears its queue only through its own channel (`registry::ToConn`, `(text, still queued)`), sent under the registry's lock with `try_send`; a message for a connection that answers nothing (`AcceptMatch` while the other has not) is `Routed::Passed`.
- A ranked game's time moves only through `Session::elapse`, which the game's thread calls before every message, closed connection and tick (`game::run`): a new event must call it first. A game's end is `Session::over` (a forfeit leaves `result()` empty).
- A ranked game's end is told only by the host's verdict (`publish_replay`/`withhold_replay`): a new way for one to end must reach `finish`. A ladder's log names a game once (by id; `<data>/next-game`): never hand out ids below `GameIds::last`.
- **The workload is lane-only and follows `master`:** 55 % of its count is the battle engine, so battle changes move it and the full verify (CI) does not measure it. At each session's start `verify --scope server` measures it; a drift there came from `master`: re-baseline (`cargo xtask bench-check --update --workload session-random-play-full`) and name the range in the commit. Measure only with the file's arguments (they shift the heap `malloc` walks).

## Recent sessions

### 2026-10-12 — M4 T4, leaderboards

#112 had merged. Built T4 (`E16-log.md`): pages by season, one's own row; wire 17.

### 2026-10-12 — SRV-Q20, the server's pings

#111 merged; the user chose (a). Built (`E16-log.md`): `ping_every_ms`, `ping_timeout_ms`; no wire change.

### 2026-10-12 — M4 T3b, the rated end

#109 had merged. Built T3b (`E16-log.md`): the held end, `Rating`, `Ladders::rate`, `GameIds`; wire 16.
