# Server lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **M1–M4 and M6 closed** (checkpoints SRV-Q12 to SRV-Q24; M6 is discovery: ADR-0021). **M5 part 1** (SRV-Q25; the epic's "M5 part 1", the QUEUE's tasks): T0 to T6 merged (#799 to #868): the game's observation, commands, rejections and events on the wire, the fold `GameObservation::fold` and `reveal` with its gate over the 72 golden replays (`tests/game_fold.rs`: 36 parts, 23 exact, the rest named by `SRV-STUB-3` to `-11` and `TWN-STUB-13`), the manifest and messages, `session::game::GameSession` (sans I/O) with its journal, restore and records (format v4), `GameReveal`, a battle's last `Snapshot`, `h3-server record`. **T7** (closure) in three parts, one pull request (the user, 2026-11-12), on `claude/keen-tesla-q5uxid`: **T7a1 done** (the reveal packed, protocol v44; a game's record cut at its snapshots; `tests/game_bounds.rs`), **T7a2 done** (`tests/game_mutation.rs`; `docs/06-security.md`'s whole game, `cross-lane`), **next T7b** (a fresh reviewer over part 1, its fixes, the exit criteria, the checkpoint SRV-Q28, then the pull request labelled `cross-lane`). **No open pull request** (open it after T7b). **The review list so far:** the epic's "M5 part 1".
- **The TLS client** (ADR-0021 point 4, `peers::Tls`): rustls over graviola; no default features on a rustls crate (`ring`, `aws-lc` are not GPLv2-compatible; `cargo deny check` after a dependency change).
- **ADR-0018 point 5 as amended:** the mapping matches the engine's enums and the observation's structs exhaustively; a new battle variant or field fails `h3-protocol`'s build until mapped (global trap).
- **The 100 % gate** covers `h3-protocol` (logic); `h3-server` is an adapter outside it, tested end to end.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, 44 since 2026-11-12, T7a1's packed reveal and a game's fresh snapshots; bumped on every wire change, a field's new meaning or a message's new order too), `RECORD_FORMAT_VERSION` (`h3-protocol`, 4 since 2026-11-10: a whole game's record, `GameRecordHeader`), `JOURNAL_FORMAT_VERSION` (`h3-server`, 8 since BAT-GEO-002's rear hex; another format's journal is void at start).
- The lane owns (`lanes.toml`) `h3-protocol`, `h3-server`, `benches/server/` and E16's files. **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
- **Ids:** questions `SRV-Q<n>`, quirks `SRV-QUIRK-<n>`, stubs `SRV-STUB-<n>`. ADR drafts go to `docs/lanes/server/adr-drafts/` and get a number when the user accepts them. Branches: `server/<task>`.

## Autonomy contract (confirmed by the user, 2026-10-09)

- **Agent decides, marked provisional:** the inner structure of the server crates, names, test layout, error texts, log format; anything a later task can change without touching another crate. Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency, a message shape inside an accepted ADR, a configuration format. List them in the milestone's review list (epic file) and go on.
- **Stop the line:** seams and contracts (`h3-contracts`, the engine API the server calls), heavy dependencies (beyond the decided `tokio` features, `tokio-tungstenite`, `ed25519-dalek` and `argon2` (SRV-Q17): another runtime, an HTTP framework, a TLS stack, other crypto crates, a store), licence questions, changes to the security model (ADR-0008), anything touching another lane's paths. Write the question (`SRV-Q<n>`), stop that task and take the next independent one.

## Standing traps

Moved to `TRAPS.md`: the wire types (`no_std`, serde forms, 64-bit integers), coverage, tests' clients and rates, and the games' threads, locks, journals and outboxes, accounts and ladders, discovery and the TLS client, Windows. Read it before touching `h3-protocol` or `h3-server`.

## Recent sessions

### 2026-11-12 — M5 T7a1: the reveal packed, a game's record cut at its snapshots

Baseline: `master` 35fc648 (#868, T5c; CI green; the setup's light gates on CI's fingerprint). No original data. The user split T7 (T7a1, T7a2, T7b; one pull request) and agreed to T7a1's three changes after the measurement: an XL map's whole reveal was 6.5 MB, 60 times its snapshot, and a bounded game record lost its snapshot. Built T7a1 (`E16-log.md`): the reveal as a palette and a rectangle per level (v44), the record cut only at snapshots with a fresh one past half its bound, the size gate over the eight XL fixtures; the map client and client lanes' inboxes. Built T7a2: the game's mutation tests (100 000 cases each, no panic), `docs/06-security.md`'s whole game. In progress: T7b next (nothing half-done).

### 2026-11-12 — M5 T5c: what each viewer newly sees, a battle's last snapshot

Baseline: `master` 7349ed4 (#863, the adventure lane's T5c blockers; the setup's light gates on CI's fingerprint), `master` f32979c merged in. No original data. The user confirmed T5c's split (T5c1, T5c2, one pull request) and that an object's index goes on the wire only if the map client's draw order needs it (it does not: a seen object's file entry by type and anchor; the map client lane's inbox). Built T5c (`E16-log.md`): `GameReveal` (v43) and its fold, the objects in tile order, the gate exact on the reveal's parts; a battle's last `Snapshot` as the client lane's reader takes it (a separate message built first, dropped); the ladders test's lobby deadline 900 ms. Next: T7.

### 2026-11-10 — M5 T6: a golden replay as a record

Baseline: `master` ac8ae2e (#821; its CI run in progress at the start, the previous run green; the sandbox's full baseline stopped, as it built the edited tree). No original data. Reviewed the adventure lane's `Revisit` mapping (protocol v40; accepted, archived). Built T6 (`E16-log.md`): `h3-server record`, the replay checked as `h3 adventure verify` checks it and run through the session, each player's record as a file; its test over the 55 golden replays; the record's form in the map client lane's inbox. Next: T5c or T7.
