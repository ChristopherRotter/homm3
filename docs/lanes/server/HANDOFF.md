# Server lane — HANDOFF

The server lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **M1 closed (2026-10-11, SRV-Q12). M2 T1–T5c merged (#67, #71, #72, #76, #78, #79, #83, #86, #88); M2 T5d done (2026-10-11)** on `claude/server-hardening-fuzz-kxo4z4`: the decoder's mutation tests (SRV-Q14), the public archive of finished games (`ListReplays`, `FetchArchived`, `[limits] replays_kept`; SRV-Q15, ADR-0018 points 8 and 9 amended); wire 11. **Next: M2 T5e** (`QUEUE.md`): M2's closure, the reviewer agent, the user's checkpoint. Review list for the M2 checkpoint: the epic file.
- **For M4** (SRV-Q13, answered): ranked games one at a time per player, a pause only by agreement, its length negotiated within the ladder's bounds, the absent player forfeits at the due time; the reading of "one at a time" is confirmed at M4's refinement.
- **Shared changes on this branch** (ADR-0018 points 8 and 9, `05-testing.md`'s fuzzing, `06-security.md`, the global HANDOFF's server row; a commit of their own): the pull request needs the label `cross-lane`, the user's consent asked.
- **ADR-0018 point 5 as amended:** the mapping's matches over the engine's enums are exhaustive, and since T4b its patterns over the observation's structs too; a new battle variant or observation field fails `h3-protocol`'s build until mapped (global HANDOFF trap).
- **The 100 % gate** covers `h3-protocol` (logic); `h3-server` is an adapter outside it, tested end to end.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, 11 since M2 T5d's archive; 10 was M2 T5c's `RecordHeader::first`; bumped on every wire change), `RECORD_FORMAT_VERSION` (`h3-protocol`, 2 since M2 T5c), `JOURNAL_FORMAT_VERSION` (`h3-server`, 3 since M2 T4c's spectator setting; 2 was M2 T3's pause lines; a journal of another format is void at start).
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
- Automated clients answer every turn at once and exceed the default 20 messages per second and the address's 40: end-to-end tests that play raise both (`tests.rs`, `fast()`).
- A frame the game marks `left` is acted on only if the task passed the game exactly that many messages (`Bound::passed`): a join pipelined behind a refused one may take a seat, and leaving then would drop its `Joined` and the seat's token.
- A game leaves the lobby only under the registry's lock, after its bound connections are counted down (`registry.rs`): deciding by the game's own connection list races a join that is already queued.
- A socket closed over unread data is reset, and the reset makes the peer drop what it has not read yet (a `Goodbye`): the transport's close reads on for the peer's close (`net.rs`, `LINGER`). The flooding test catches a regression only in most runs, not all.
- A journal's line holds tokens: errors from reading one name the line and column, never its text (serde_json's messages quote values); a log line never carries a hash either (`journal.rs`).
- A finished game's replay is written by `game::finish` into the archive's directory its spectator setting names (`archive::dir`) and must then join the registry's archive (`Registry::archived`): a replay written elsewhere is neither listed nor trimmed.
- `PROPTEST_CASES` applies to every proptest of a crate (the session's are slow): name the test (`cargo test -p h3-protocol mutat`) for a longer mutation run.
- A game's frames go through `game::Outbox` (`offer`, or `last` for a goodbye), never a bare channel: the connection's task subtracts every frame's bytes as it takes it out, so a frame queued around the count wraps it, and the queue then refuses every frame.
- Write-ahead order in a game's thread: drain the session's entries and sync them before `deliver`; anything new that changes what a restart must redo needs an `Entry`, or the restore diverges from what clients were told (T3's resume is one: `Entry::Resumed`).
- The observation's mapping (`map/mod.rs`) names every field of the engine's types in patterns, without `..`: a field left out gets `name: _` under a comment with its reason. A field read through a method (`s.attack_now`) still needs its pattern entry.
- The workspace's clippy bans `std::time::Instant` and `SystemTime` in every crate: the server uses `tokio::time::Instant`, and a file's age is `fs::metadata(p)?.modified()?.elapsed()` (a `Duration`, no type named).
- A `watch::Receiver` whose sender is gone makes `wait_for` fail at once, again and again: a select arm on it must then never resolve (`net::stopped`), or a dropped `Running` would stop every connection.
- A paused game has no thread: the registry's `Hosted::Paused` holds its tokens, and `registry::bind` restores it for a join. A game's thread ignores `ToGame::Open` (nothing changed until the join), or a resuming game, still paused, would pause again before its join.
- **The workload is lane-only and follows `master`:** 55 % of its count is the battle engine, so battle changes move it and the full verify (CI) does not measure it. At each session's start `verify --scope server` measures it; a drift there came from `master`: re-baseline (`cargo xtask bench-check --update --workload session-random-play-full`) and name the range in the commit. Measure only with the file's arguments (they shift the heap `malloc` walks).

## Recent sessions

### 2026-10-11 — M2 T5d, mutation tests and the public archive

#88 had merged; the branch restarted from `master`. The user answered SRV-Q14 (mutation tests now) and SRV-Q15 (finished replays public). Built T5d (`docs/epics/E16-log.md`, M2 T5d): `tests/mutation.rs`; `archive.rs` with `ListReplays` and `FetchArchived` (wire 11), a game closed to spectators archived unlisted, `replays_kept`. T5d split again: T5e is M2's closure. Review list in the epic file.

### 2026-10-11 — M2 T5c, memory bounded in bytes

#86 had merged; the branch restarted from `master`. Built T5c (`docs/epics/E16-log.md`, M2 T5c): the record as a window of the latest messages (`RecordHeader::first`, wire 10, record format 2), `game::Outbox` counting a queue's bytes; one `FetchRecord` in flight left to the queue's bound; a battle's missing round limit noted (inbox). Review list in the epic file.
