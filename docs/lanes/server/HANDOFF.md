# Server lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **M1–M4 and M6 closed** (checkpoints SRV-Q12 to SRV-Q24; M6 is discovery: ADR-0021). **M5 part 1** (SRV-Q25; the epic's "M5 part 1", the QUEUE's tasks): T0 to T5b merged (#799, #803, #805, #814, #816, #820, #821): the game's observation, commands, rejections and events on the wire, the fold `GameObservation::fold` with its gate over the 55 golden replays (`tests/game_fold.rs`: 36 parts, 17 exact, the rest named by `SRV-STUB-1` to `-11`, the adventure lane's inbox, and `TWN-STUB-13`), the game's manifest and messages, `session::game::GameSession` (sans I/O; `SRV-STUB-14`) with its journal, restore and records (record format v4). **T6** (`h3-server record`: a golden replay as each player's record, `h3_server::record`; the map client lane's inbox note) done 2026-11-10 on `claude/vigilant-brown-m2nolc`. **Open pull request:** T6's, from that branch (find it by the branch). **Next:** T5c once the adventure lane's SRV-Q26 query lands, then T7 (closure and the checkpoint); whether T7 may go first, T5c after part 1's checkpoint, is the user's call. **The review list so far:** the epic's "M5 part 1".
- **The TLS client** (ADR-0021 point 4, `peers::Tls`): rustls over graviola; no default features on a rustls crate (`ring`, `aws-lc` are not GPLv2-compatible; `cargo deny check` after a dependency change).
- **ADR-0018 point 5 as amended:** the mapping matches the engine's enums and the observation's structs exhaustively; a new battle variant or field fails `h3-protocol`'s build until mapped (global trap).
- **The 100 % gate** covers `h3-protocol` (logic); `h3-server` is an adapter outside it, tested end to end.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, 40 since 2026-11-10, E08 T105b2's `Revisit` mapped by the adventure lane; bumped on every wire change, a field's new meaning or a message's new order too), `RECORD_FORMAT_VERSION` (`h3-protocol`, 4 since 2026-11-10: a whole game's record, `GameRecordHeader`), `JOURNAL_FORMAT_VERSION` (`h3-server`, 8 since BAT-GEO-002's rear hex; another format's journal is void at start).
- The lane owns (`lanes.toml`) `h3-protocol`, `h3-server`, `benches/server/` and E16's files. **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
- **Ids:** questions `SRV-Q<n>`, quirks `SRV-QUIRK-<n>`, stubs `SRV-STUB-<n>`. ADR drafts go to `docs/lanes/server/adr-drafts/` and get a number when the user accepts them. Branches: `server/<task>`.

## Autonomy contract (confirmed by the user, 2026-10-09)

- **Agent decides, marked provisional:** the inner structure of the server crates, names, test layout, error texts, log format; anything a later task can change without touching another crate. Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency, a message shape inside an accepted ADR, a configuration format. List them in the milestone's review list (epic file) and go on.
- **Stop the line:** seams and contracts (`h3-contracts`, the engine API the server calls), heavy dependencies (beyond the decided `tokio` features, `tokio-tungstenite`, `ed25519-dalek` and `argon2` (SRV-Q17): another runtime, an HTTP framework, a TLS stack, other crypto crates, a store), licence questions, changes to the security model (ADR-0008), anything touching another lane's paths. Write the question (`SRV-Q<n>`), stop that task and take the next independent one.

## Standing traps

Moved to `TRAPS.md`: the wire types (`no_std`, serde forms, 64-bit integers), coverage, tests' clients and rates, and the games' threads, locks, journals and outboxes, accounts and ladders, discovery and the TLS client, Windows. Read it before touching `h3-protocol` or `h3-server`.

## Recent sessions

### 2026-11-10 — M5 T6: a golden replay as a record

Baseline: `master` ac8ae2e (#821; its CI run in progress at the start, the previous run green; the sandbox's full baseline stopped, as it built the edited tree). No original data. Reviewed the adventure lane's `Revisit` mapping (protocol v40; accepted, archived). Built T6 (`E16-log.md`): `h3-server record`, the replay checked as `h3 adventure verify` checks it and run through the session, each player's record as a file; its test over the 55 golden replays; the record's form in the map client lane's inbox. Next: T5c or T7.

### 2026-11-10 — M5 T5b: the game's journal, restore and records

Baseline: `master` 780fbcc (#820; CI green; the setup ran the light gates on CI's fingerprint). No original data. Built T5b (`E16-log.md`): the journal's entries and `GameJournal`, `GameSession::restore` (the re-run, then every player told afresh), each player's record with `GameRecordHeader` (record format v4), the battle's record moved into `session::Record` for both. The golden replays' session test checks the entries, the journal against each replay, the records, and a restore halfway. Protocol unchanged (v39). Next: T5c or T6.

### 2026-11-10 — M5 T5a: the game's messages and the in-process session

Baseline: T4's branch with `master` b1d2e94 merged in (CI green on b1d2e94; the setup's baseline ran on b1d2e94's code, light gates only; the scoped verify passed on the merge). No original data. The user confirmed T5's split (T5a, T5b, T5c) and one pull request for T4 and T5a. Built T5a (`E16-log.md`): five `Game*` server messages (v39), `GameSession` and its test over the golden replays beside the engine; the fold's golden loader shared. `SRV-STUB-14` (a battle's last snapshot) with the adventure lane's inbox note. Next: T5b.
