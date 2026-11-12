# Server lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **M1–M4 and M6 closed** (checkpoints SRV-Q12 to SRV-Q24; M6 is discovery: ADR-0021). **M5 part 1** (SRV-Q25; the epic's "M5 part 1", the QUEUE's tasks): T0 to T6 merged (#799 to #822): the game's observation, commands, rejections and events on the wire, the fold `GameObservation::fold` with its gate over the 72 golden replays (`tests/game_fold.rs`: 36 parts, 23 exact, the rest named by `SRV-STUB-3` to `-11`, the adventure lane's inbox, and `TWN-STUB-13`), the manifest and messages, `session::game::GameSession` (sans I/O) with its journal, restore and records (format v4), `h3-server record`. **T5c** (2026-11-12, on `claude/optimistic-meitner-j73r89`): `GameReveal` after the events when a viewer's explored tiles grew and its fold, a view's objects in tile order (no index, `docs/06-security.md`), a battle's last `Snapshot` (`SRV-STUB-1` and `-14` lifted). **Open pull request:** T5c's, from that branch (find it by the branch). **Next:** T7 (closure: mutation tests over the game's messages, the frame and record bounds, `docs/06-security.md`'s game part, a fresh reviewer, the checkpoint). **The review list so far:** the epic's "M5 part 1".
- **The TLS client** (ADR-0021 point 4, `peers::Tls`): rustls over graviola; no default features on a rustls crate (`ring`, `aws-lc` are not GPLv2-compatible; `cargo deny check` after a dependency change).
- **ADR-0018 point 5 as amended:** the mapping matches the engine's enums and the observation's structs exhaustively; a new battle variant or field fails `h3-protocol`'s build until mapped (global trap).
- **The 100 % gate** covers `h3-protocol` (logic); `h3-server` is an adapter outside it, tested end to end.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, 43 since 2026-11-12, T5c's `GameReveal`, the objects' order and a battle's last `Snapshot`; bumped on every wire change, a field's new meaning or a message's new order too), `RECORD_FORMAT_VERSION` (`h3-protocol`, 4 since 2026-11-10: a whole game's record, `GameRecordHeader`), `JOURNAL_FORMAT_VERSION` (`h3-server`, 8 since BAT-GEO-002's rear hex; another format's journal is void at start).
- The lane owns (`lanes.toml`) `h3-protocol`, `h3-server`, `benches/server/` and E16's files. **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
- **Ids:** questions `SRV-Q<n>`, quirks `SRV-QUIRK-<n>`, stubs `SRV-STUB-<n>`. ADR drafts go to `docs/lanes/server/adr-drafts/` and get a number when the user accepts them. Branches: `server/<task>`.

## Autonomy contract (confirmed by the user, 2026-10-09)

- **Agent decides, marked provisional:** the inner structure of the server crates, names, test layout, error texts, log format; anything a later task can change without touching another crate. Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency, a message shape inside an accepted ADR, a configuration format. List them in the milestone's review list (epic file) and go on.
- **Stop the line:** seams and contracts (`h3-contracts`, the engine API the server calls), heavy dependencies (beyond the decided `tokio` features, `tokio-tungstenite`, `ed25519-dalek` and `argon2` (SRV-Q17): another runtime, an HTTP framework, a TLS stack, other crypto crates, a store), licence questions, changes to the security model (ADR-0008), anything touching another lane's paths. Write the question (`SRV-Q<n>`), stop that task and take the next independent one.

## Standing traps

Moved to `TRAPS.md`: the wire types (`no_std`, serde forms, 64-bit integers), coverage, tests' clients and rates, and the games' threads, locks, journals and outboxes, accounts and ladders, discovery and the TLS client, Windows. Read it before touching `h3-protocol` or `h3-server`.

## Recent sessions

### 2026-11-12 — M5 T5c: what each viewer newly sees, a battle's last snapshot

Baseline: `master` 7349ed4 (#863, the adventure lane's T5c blockers; the setup's light gates on CI's fingerprint), `master` f32979c merged in. No original data. The user confirmed T5c's split (T5c1, T5c2, one pull request) and that an object's index goes on the wire only if the map client's draw order needs it (it does not: a seen object's file entry by type and anchor; the map client lane's inbox). Built T5c (`E16-log.md`): `GameReveal` (v43) and its fold, the objects in tile order, the gate exact on the reveal's parts; a battle's last `Snapshot` as the client lane's reader takes it (a separate message built first, dropped); the ladders test's lobby deadline 900 ms. Next: T7.

### 2026-11-10 — M5 T6: a golden replay as a record

Baseline: `master` ac8ae2e (#821; its CI run in progress at the start, the previous run green; the sandbox's full baseline stopped, as it built the edited tree). No original data. Reviewed the adventure lane's `Revisit` mapping (protocol v40; accepted, archived). Built T6 (`E16-log.md`): `h3-server record`, the replay checked as `h3 adventure verify` checks it and run through the session, each player's record as a file; its test over the 55 golden replays; the record's form in the map client lane's inbox. Next: T5c or T7.

### 2026-11-10 — M5 T5b: the game's journal, restore and records

Baseline: `master` 780fbcc (#820; CI green; the setup ran the light gates on CI's fingerprint). No original data. Built T5b (`E16-log.md`): the journal's entries and `GameJournal`, `GameSession::restore` (the re-run, then every player told afresh), each player's record with `GameRecordHeader` (record format v4), the battle's record moved into `session::Record` for both. The golden replays' session test checks the entries, the journal against each replay, the records, and a restore halfway. Protocol unchanged (v39). Next: T5c or T6.
