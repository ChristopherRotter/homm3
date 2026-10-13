# Server lane — HANDOFF

The server lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **M1–M4 closed** (SRV-Q12, SRV-Q16, SRV-Q18, SRV-Q22). **M6** (SRV-Q23, ADR-0021, `E16-design.md` §12): T1 (the key, the welcome), T2 (the card), T3 (peers, `ListPeers`) and T4 (the chain beyond the peers) done. **Next: M6 T5** (lists). M5 waits for E08's increment 3.
- **The TLS client** (ADR-0021 point 4, `peers.rs`): rustls over graviola and the OS roots, no default features on a rustls crate (`ring`, `aws-lc` are not GPLv2-compatible; `cargo deny check` after a dependency change).
- **ADR-0018 point 5 as amended:** the mapping matches the engine's enums and the observation's structs exhaustively; a new battle variant or field fails `h3-protocol`'s build until mapped (global trap).
- **The 100 % gate** covers `h3-protocol` (logic); `h3-server` is an adapter outside it, tested end to end.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, 27 since CLT-Q1; bumped on every wire change), `RECORD_FORMAT_VERSION` (`h3-protocol`, 2 since M2 T5c), `JOURNAL_FORMAT_VERSION` (`h3-server`, 8 since BAT-GEO-002's rear hex; another format's journal is void at start).
- The lane owns (`lanes.toml`) `h3-protocol`, `h3-server`, `benches/server/` and E16's files. **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
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
- A test client picking uniformly among legal commands rarely ends a `full` battle: pick strikes first (`tests/session.rs`, `play`).
- A test `Client` answers pings only while it reads: one unread beyond `ping_timeout_ms` is dropped, and its late pong draws a reset that drops its unread frames.
- Automated clients exceed the default rates (20 messages/s, 40 per address): end-to-end tests that play raise both (`tests.rs`, `fast()`).
- A frame the game marks `left` is acted on only if the task passed the game exactly that many messages (`Bound::passed`): leaving after a pipelined join would drop its `Joined` and token.
- A game leaves the lobby only under the registry's lock, after its bound connections are counted down (`registry.rs`): deciding by the game's own connection list races a join that is already queued.
- A socket closed over unread data is reset and the peer drops its unread `Goodbye`: the close reads on for the peer's (`net.rs`, `LINGER`).
- A journal's line holds tokens: errors from reading one name the line and column, never its text (serde_json's messages quote values); a log line never carries a hash either (`journal.rs`).
- No file I/O under the registry's lock (T5e): a game creates or reads its journal on its own thread (`game::launch`), and a new or resumed game's own opening is queued under the lock (`registry::Opening::first`) before anything else reaches it.
- Replays go into `archive::dir` through `game::finish`, then `Registry::archived`: one written elsewhere is neither listed nor trimmed.
- `PROPTEST_CASES` applies to every proptest of a crate: name the test for a longer mutation run (`cargo test -p h3-protocol mutat`).
- A game's frames go through `game::Outbox` (`offer`, or `last` for a goodbye), never a bare channel: the task subtracts each frame's bytes as it takes it, so a frame queued around the count wraps it and the queue refuses every frame.
- Write-ahead order in a game's thread: drain the session's entries and sync them before `deliver`; anything new that changes what a restart must redo needs an `Entry`, or the restore diverges from what clients were told (T3's resume is one: `Entry::Resumed`).
- The observation's mapping (`map/mod.rs`) names every field of the engine's types in patterns, without `..`: a field left out gets `name: _` under a comment with its reason. A field read through a method still needs its entry.
- The workspace's clippy bans `std::time::Instant` and `SystemTime` in every crate: the server uses `tokio::time::Instant`, and a file's age is `fs::metadata(p)?.modified()?.elapsed()` (a `Duration`, no type named).
- A `watch::Receiver` whose sender is gone fails `wait_for` at once, repeatedly: a select arm on it must then never resolve (`net::stopped`).
- A paused game has no thread: the registry's `Hosted::Paused` holds its tokens, and `registry::bind` restores it for a join. A game's thread ignores `ToGame::Open`, or a resuming game would pause again before its join.
- An account's request goes through `spawn_blocking`, the player log under its own lock (`accounts.rs`), the signature checked first; never the registry's lock there. The lock waits on syncs: `Accounts::get` from a game's thread outside the registry's lock (`game::seated`), never a connection's task.
- A running server holds `<data>/lock` until `shutdown` returns: a test's second server on its data directory shuts the first down; crash tests start on a copy (`crashed()`).
- `Key::derive` costs 64 MiB and ~1 s in a debug build: tests use `Key::from_seed`.
- `f64::exp` and `ln` are banned workspace-wide (ADR-0003 §3): Glicko-2 calls them behind a local `allow` (`rating.rs`); a rating is written to the ladder's log and read back, never recomputed.
- A queued connection hears its queue only through its own channel (`registry::ToConn`, `(text, still queued)`), sent under the registry's lock with `try_send`; a message answered by nothing (`AcceptMatch` before the other's) is `Routed::Passed`.
- A ranked game's time moves only through `Session::elapse`, which the game's thread calls before every message, close and tick (`game::run`): a new event calls it first. A game's end is `Session::over` (a forfeit leaves `result()` empty).
- A server's bot account has no `key` (T5): a login, a re-key and `rebind` must never match it.
- A paused journal's last write starts its deadline and an agreed pause's due time: never append to one. A pause's hold returns at a restore from `agreed_pause()` or `held_pause()` (`Held`); `game::resume` re-applies either.
- A game whose thread ends with its journal kept leaves by `Registry::stopped`, not `leave` (a match keeps its players). A start re-runs journals one by one by id (rating order).
- A ranked game's end is told only by the host's verdict (`publish_replay`/`withhold_replay`): a new way to end must reach `finish`. A ladder's log names a game once (`<data>/next-game`): never hand out ids below `GameIds::last`.
- **The workload is lane-only and follows `master`** (55 % battle engine; not in the full verify): `verify --scope server` measures it at each session's start; re-baseline a drift from `master` (`cargo xtask bench-check --update --workload session-random-play-full`), the range in the commit; measure only with the file's arguments.
- A connection's server id is `Connection::id`, set by its hello: logins sign it, not `Host::id` (M6 T1).
- A card travels as the text its key signed (`SignedCard::card`): relay it verbatim. The server re-signs its own a minute old (`card::RESIGN`).
- Windows refuses `set_len` through an `append(true)` handle: logs open `write(true)` and write at their end (`journal::open_log`). Check with clippy `--target x86_64-pc-windows-gnu`; Wine is no test.
- A `wss://` dial passes `Connector::Rustls` (`Client::dial`), or tokio-tungstenite builds a default `ClientConfig` and panics (no process-wide provider). Graviola panics on a processor without its extensions: its calls run in a task of their own (`peers::fetch_every`).
- `ListPeers` serves `peers::relay`'s rebuild, each chain checked against the served cards: T5's lists go through it. A fetch below `depth` pages `ListPeers`: a fake peer must answer it (`tests/peers.rs`).

## Recent sessions

### 2026-10-13 — M6 T4, the chain beyond the peers

**#165 open** (`E16-log.md`): the peers' relays to `depth`, chains checked; no wire change. Check its mergeability.
