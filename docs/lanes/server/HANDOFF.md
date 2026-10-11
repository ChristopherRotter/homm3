# Server lane — HANDOFF

The server lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **M1 closed (2026-10-11, SRV-Q12). M2 T1–T4c merged (#67, #71, #72, #76, #78, #79); M2 T5a done (2026-10-11)** on `claude/modest-turing-ttgjo8`: the first signal drains, a second stops, the journals kept; abandoned games pause, then go void (`[limits] empty_game_ms`, `paused_game_ms`); wire 9. T5 was split into T5a–T5c. **Next: M2 T5b** (`QUEUE.md`): hardening; then T5c, M2's closure and checkpoint. Review list for the M2 checkpoint: the epic file.
- **Shared changes on this branch** (`tokio`'s `signal` and ADR-0018 point 3, a commit of its own; the global HANDOFF's server row): the pull request needs the label `cross-lane` (asked of the user for T5a, as given for T2–T4c).
- **ADR-0018 point 5 as amended:** the mapping's matches over the engine's enums are exhaustive, and since T4b its patterns over the observation's structs too; a new battle variant or observation field fails `h3-protocol`'s build until mapped (global HANDOFF trap).
- **The 100 % gate** covers `h3-protocol` (logic); `h3-server` is an adapter outside it, tested end to end.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, 9 since M2 T5a's `Draining`; 8 was M2 T4c's spectator setting; bumped on every wire change), `RECORD_FORMAT_VERSION` (`h3-protocol`, 1), `JOURNAL_FORMAT_VERSION` (`h3-server`, 3 since M2 T4c's spectator setting; 2 was M2 T3's pause lines; a journal of another format is void at start).
- The lane owns (`lanes.toml`) `h3-protocol`, `h3-server`, `benches/server/` and E16's files; the workspace files stay shared. **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does. The identity and discovery ADR drafts are decided at the M3 refinement.
- **Ids:** questions `SRV-Q<n>`, quirks `SRV-QUIRK-<n>`, stubs `SRV-STUB-<n>`. ADR drafts go to `docs/lanes/server/adr-drafts/` and get a number when the user accepts them. Branches: `server/<task>`.

## Autonomy contract (confirmed by the user, 2026-10-09)

- **Agent decides, marked provisional:** the inner structure of the server crates, names, test layout, error texts, log format; anything a later task can change without touching another crate. Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency, a message shape inside an accepted ADR, a configuration format. List them in the milestone's review list (epic file) and go on.
- **Stop the line:** seams and contracts (`h3-contracts`, the engine API the server calls), heavy dependencies (beyond the decided `tokio` features and `tokio-tungstenite`: another runtime, an HTTP framework, a TLS stack, the crypto crates, a store), licence questions, changes to the security model (ADR-0008), anything touching another lane's paths. Write the question (`SRV-Q<n>`), stop that task and take the next independent one.

## Standing traps

- `h3-protocol` is `no_std`: take `serde` (`derive`, `alloc`) and `serde_json` (`alloc`) with `default-features = false` in its own manifest, not `workspace = true` (the workspace entries enable `std`).
- `postcard` (and any format that is not self-describing) cannot decode `#[serde(tag = ...)]`, `untagged` or `flatten`: keep the wire types in serde's default enum form (SRV-Q5).
- A 64-bit integer on the wire travels as a string (JavaScript clients lose precision above 2^53).
- A wildcard arm after every variant of an enum still counts as one uncovered region (llvm-cov), and `#[coverage(off)]` is unstable: no fallback arms in `h3-protocol` (ADR-0018 point 5 as amended).
- `prop_assert!(matches!(x, P { .. }))` without a message fails to compile: the macro formats the condition's text and reads `{ .. }` as a format argument; pass a message.
- A test client that picks uniformly among the legal commands rarely ends a battle of the `full` scenario (mostly moves; the defender's stacks regenerate): pick among the strikes when there are any (`tests/session.rs`, `play`).
- Automated clients answer every turn at once and exceed the default 20 messages per second: end-to-end tests that play raise the limit (`tests.rs`, `fast()`).
- A game leaves the lobby only under the registry's lock, after its bound connections are counted down (`registry.rs`): deciding by the game's own connection list races a join that is already queued.
- A socket closed over unread data is reset, and the reset makes the peer drop what it has not read yet (a `Goodbye`): the transport's close reads on for the peer's close (`net.rs`, `LINGER`). The flooding test catches a regression only in most runs, not all.
- A journal's line holds tokens: errors from reading one name the line and column, never its text (serde_json's messages quote values); a log line never carries a hash either (`journal.rs`).
- Write-ahead order in a game's thread: drain the session's entries and sync them before `deliver`; anything new that changes what a restart must redo needs an `Entry`, or the restore diverges from what clients were told (T3's resume is one: `Entry::Resumed`).
- The observation's mapping (`map/mod.rs`) names every field of the engine's types in patterns, without `..`: a field left out gets `name: _` under a comment with its reason. A field read through a method (`s.attack_now`) still needs its pattern entry.
- The workspace's clippy bans `std::time::Instant` and `SystemTime` in every crate: the server uses `tokio::time::Instant`, and a file's age is `fs::metadata(p)?.modified()?.elapsed()` (a `Duration`, no type named).
- A `watch::Receiver` whose sender is gone makes `wait_for` fail at once, again and again: a select arm on it must then never resolve (`net::stopped`), or a dropped `Running` would stop every connection.
- A paused game has no thread: the registry's `Hosted::Paused` holds its tokens, and `registry::bind` restores it for a join. A game's thread ignores `ToGame::Open` (nothing changed until the join), or a resuming game, still paused, would pause again before its join.
- **The workload is lane-only and follows `master`:** 55 % of its count is the battle engine, so battle changes move it and the full verify (CI) does not measure it. At each session's start `verify --scope server` measures it; a drift there came from `master`: re-baseline (`cargo xtask bench-check --update --workload session-random-play-full`) and name the range in the commit. Measure only with the file's arguments (they shift the heap `malloc` walks).

## Recent sessions

### 2026-10-11 — M2 T5a, shutdown, drain and deadlines

#79 had merged; the branch restarted from `master`. Split T5 into T5a–T5c (`QUEUE.md`). Built T5a (`docs/epics/E16-log.md`, M2 T5a): `tokio`'s `signal` (ADR-0018 point 3, shared commit); the first signal drains, a second stops; a stop as a `watch` for the connections and a message to every game's thread; the deadlines of abandoned games swept by a task. Review list in the epic file.

### 2026-10-11 — M2 T4c, the host's spectator setting

#78 had merged; the branch restarted from `master`. The user asked for T4c and answered its questions (before the start only; open by default). Built T4c (`docs/epics/E16-log.md`, M2 T4c): the creator's choice at `CreateGame`, the host's `SetSpectators` until the start, the lobby's `spectators`, the setting journaled once, as the game started (a host's toggles before it write nothing). Review list in the epic file.

### 2026-10-11 — M2 T4b, the observation's completeness

#76 had merged; the branch restarted from `master` (the scoped verify had no failing step but coverage, whose LLVM paths were not set; the workload −0.39 %). The user asked whether a host can turn spectators off: no, only the operator for every game; queued as T4c. Built T4b (`docs/epics/E16-log.md`, M2 T4b): the new fields per stack, per side and on the own hero; the effects' values; patterns over every engine struct of the observation. Review list in the epic file.
