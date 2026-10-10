# Server lane — HANDOFF

The server lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **M1 done (2026-10-10), the user's checkpoint next** (SRV-Q12): T1–T3 merged; T4 on `claude/upbeat-babbage-8bcd7n`: the lane-only workload `session-random-play-full` (`h3-server-bench`, 273 M instructions in `serve_frame`), `docs/06-security.md`'s "Hosted games", the global HANDOFF's server row, the reviewer agent. The checkpoint decides the review list (epic file), the proposed M2 cut (`QUEUE.md`) and two stop-the-line items (the `signal` feature, tokens across a restart).
- **Shared changes on this branch** (`xtask` bench-check: a workload's own binary and `lane_only`; `06-security.md`; the global row; `benches/README.md`): the session could push one branch only, so they are the branch's first commit; the pull request carries the label `cross-lane`, or the commit goes first as a `shared/` pull request if the user prefers.
- **ADR-0018 point 5 as amended:** the mapping's matches over the engine's enums are exhaustive; a new battle variant fails `h3-protocol`'s build until mapped (global HANDOFF trap).
- **The 100 % gate** covers `h3-protocol` (logic); `h3-server` is an adapter outside it, tested end to end.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, 2, bumped on every wire change), `RECORD_FORMAT_VERSION` (`h3-protocol`, 1), `JOURNAL_FORMAT_VERSION` (`h3-server`, M2).
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
- A socket closed over unread data is reset, and the reset makes the peer drop what it has not read yet (a `Goodbye`): the transport's close reads on for the peer's close (`net.rs`, `LINGER`). The flooding test catches a regression only in most runs, not all.
- **The workload is lane-only and follows `master`:** 55 % of its count is the battle engine, so battle changes move it and the full verify (CI) does not measure it. At each session's start `verify --scope server` measures it; a drift there came from `master`: re-baseline (`cargo xtask bench-check --update --workload session-random-play-full`) and name the range in the commit. Measure only with the file's arguments (they shift the heap `malloc` walks).

## Recent sessions

### 2026-10-10 — M1 T4, closure

T3 had merged; the branch started from `master`. Built the workload (`docs/epics/E16-log.md`, T4); its profile showed the count is 55 % battle engine, so a full-verify gate would fail battle pull requests on a server file: made it lane-only (a shared `xtask` change). `06-security.md`'s "Hosted games" closes the queue's E03 T9 inbox item. The reviewer agent's findings and their handling: the epic log. The M2 cut is proposed in `QUEUE.md`; SRV-Q12 asks the checkpoint.

### 2026-10-10 — M1 T3, the server binary

T2 had merged; the branch restarted from `master` (CI green on T2's pull request). Built T3 (`docs/epics/E16-log.md`, T3): configuration, game thread, connection tasks, WebSocket transport, headless client, 14 tests. The flooding test found the reset that lost the `Goodbye` (fixed by a lingering close); the first two-client test found that automated clients exceed the default rate. The replays the tests wrote verify with `h3 verify`. Review list in the epic file; `verify --scope server` with no failing step.

### 2026-10-10 — M1 T2, the session core

#54 (T1) had merged; the branch started from `master`, whose baseline verify was green. Built T2 (`docs/epics/E16-log.md`, T2). The session needed to refuse a replay before the end and to answer a fetch from a connection without a seat, so the fetches got `seq` and the reject codes `NotOver` (wire version 2, queued for the review with the record format, the start rule and the bot fallback). The hygiene property over the wire is checked by a mutation (events projected for the wrong viewer fail it). 21 new tests; `verify --scope server` with no failing step.
