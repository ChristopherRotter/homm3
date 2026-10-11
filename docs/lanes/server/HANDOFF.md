# Server lane — HANDOFF

The server lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **M1 closed (2026-10-11, SRV-Q12). M2 T1 merged (#67); M2 T2 done (2026-10-11)** on `claude/upbeat-wright-df9owq`, pull request next: the write-ahead journal and restore (the session core's `Entry`s and `Session::restore` in `h3-protocol`; `journal.rs` and the restore at start in `h3-server`; `<data>/games/`, `<data>/void/`). **Next: M2 T3** (`QUEUE.md`), verification, pause and resume. Review list for the M2 checkpoint: the epic file.
- **Shared changes on this branch** (the global HANDOFF's server row, `06-security.md`'s journal and data directory): a separate commit; the pull request needs the label `cross-lane` (ask the user) or a `shared/` pull request first.
- **ADR-0018 point 5 as amended:** the mapping's matches over the engine's enums are exhaustive; a new battle variant fails `h3-protocol`'s build until mapped (global HANDOFF trap).
- **The 100 % gate** covers `h3-protocol` (logic); `h3-server` is an adapter outside it, tested end to end.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, 4 since M2 T1's lobby; 3 was the battle lane's `OpeningSpell` event; bumped on every wire change), `RECORD_FORMAT_VERSION` (`h3-protocol`, 1), `JOURNAL_FORMAT_VERSION` (`h3-server`, 1 since M2 T2; bumped when the journal's lines change).
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
- Write-ahead order in a game's thread: drain the session's entries and sync them before `deliver`; anything new that changes what a restart must redo needs an `Entry`, or the restore diverges from what clients were told.
- **The workload is lane-only and follows `master`:** 55 % of its count is the battle engine, so battle changes move it and the full verify (CI) does not measure it. At each session's start `verify --scope server` measures it; a drift there came from `master`: re-baseline (`cargo xtask bench-check --update --workload session-random-play-full`) and name the range in the commit. Measure only with the file's arguments (they shift the heap `malloc` walks).

## Recent sessions

### 2026-10-11 — M2 T2, the journal

#67 had merged; the branch started from `master` (the workload −0.10 %). Built T2 (`docs/epics/E16-log.md`, M2 T2): the session core hands the host `Entry`s, which the game's thread syncs before it sends a call's frames; `Session::restore` re-runs them (the re-run's entries must extend the journal's; a cut journal is completed); a JSON-lines journal per game with the tokens and the replay's TOML in its first line; restore at start before listening, void journals moved aside. An end-to-end test copies the data directory mid-battle and resumes on a second server. Review list in the epic file.

### 2026-10-11 — M2 T1, many games

The user merged #61 and asked for the next increment: SRV-Q12 answered, the branch restarted from `master` (the workload −0.12 % after the battle lane's T8c, no re-baseline). Built T1 (`docs/epics/E16-log.md`, M2 T1): the lobby's messages (wire 4), the sans-I/O lobby, offers, the registry and a thread per game. A game's leaving is decided under the registry's lock with the bindings, not by its own connection list (that would race a join). Review list in the epic file.

### 2026-10-10 — M1 T4, closure

T3 had merged; the branch started from `master`. Built the workload (`docs/epics/E16-log.md`, T4); its profile showed the count is 55 % battle engine, so a full-verify gate would fail battle pull requests on a server file: made it lane-only (a shared `xtask` change). `06-security.md`'s "Hosted games" closes the queue's E03 T9 inbox item. The reviewer agent found 2 major and 7 minor issues: the unbounded close (fixed, with an end-to-end test that fails without the fix), the limits' ranges, three doc claims and four missing tests (fixed); memory bounded in bytes and the handshake's hold went to M2 T5 (the epic log). The M2 cut is proposed in `QUEUE.md`; SRV-Q12 asks the checkpoint.
