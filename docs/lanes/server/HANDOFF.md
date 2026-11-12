# Server lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **M1–M4, M6 and M5 part 1 closed** (checkpoints SRV-Q12 to SRV-Q24 and SRV-Q28, 2026-11-12; M6 is discovery: ADR-0021). Part 1 (`E16-archive.md`, done in full in `E16-log.md`): the whole game's wire mapped exhaustively, the fold with its gate over the 72 golden replays (`tests/game_fold.rs`: 36 parts, 23 exact, the rest named by `SRV-STUB-3` to `-11` and `TWN-STUB-13`), the in-process `session::game::GameSession` with journal, restore and records (format v4, cut at snapshots), the packed `GameReveal`, `h3-server record`, the size gate (`tests/game_bounds.rs`) and the mutation tests (`tests/game_mutation.rs`). T7's pull request merged (#874). **M5 part 2 refined** (SRV-Q29, 2026-11-12): 2a whole games over the network (T8 to T13, the epic's "M5 part 2"), 2b time and 2c spectators and long journals, refined at the previous checkpoint; the server lane builds `h3-game`'s system commands (2b) and `Viewer::Spectator` (2c) in `cross-lane` pull requests (the user's consent). **Next: T8a**, the whole game's seats and connections (T8 in two, T8a and T8b).
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

### 2026-11-12 — M5 part 2's refinement (SRV-Q29)

Baseline: `master` d1b5147 (#874, T7, merged). No original data. Refined part 2 with the user (SRV-Q29): three increments (2a network whole games, 2b time, 2c spectators and long journals), the engine's system commands and spectator viewer the server lane's in `cross-lane` pull requests, every map player a seat in 2a; 2a's tasks T8 (in two) to T13 in the epic; the adventure and town lanes' inboxes told; the refinement in its own pull request. Next: T8a.

### 2026-11-12 — M5 T7: part 1's closure (the reveal packed, records cut at snapshots, the review, SRV-Q28)

Baseline: `master` 35fc648 (#868, T5c; CI green; the setup's light gates on CI's fingerprint). No original data. The user split T7 (T7a1, T7a2, T7b; one pull request) and agreed to T7a1's three changes after the measurement: an XL map's whole reveal was 6.5 MB, 60 times its snapshot, and a bounded game record lost its snapshot. Built T7a1 (`E16-log.md`): the reveal as a palette and a rectangle per level (v44), the record cut only at snapshots with a fresh one past half its bound, the size gate over the eight XL fixtures; the map client and client lanes' inboxes. Built T7a2: the game's mutation tests (100 000 cases each, no panic), `docs/06-security.md`'s whole game. T7b: a fresh reviewer (two major, four minor; both major and two minor fixed), the closure; the user passed the checkpoint (SRV-Q28: the review list as built, next part 2's refinement). One pull request for T7.

### 2026-11-12 — M5 T5c: what each viewer newly sees, a battle's last snapshot

Baseline: `master` 7349ed4 (#863, the adventure lane's T5c blockers; the setup's light gates on CI's fingerprint), `master` f32979c merged in. No original data. The user confirmed T5c's split (T5c1, T5c2, one pull request) and that an object's index goes on the wire only if the map client's draw order needs it (it does not: a seen object's file entry by type and anchor; the map client lane's inbox). Built T5c (`E16-log.md`): `GameReveal` (v43) and its fold, the objects in tile order, the gate exact on the reveal's parts; a battle's last `Snapshot` as the client lane's reader takes it (a separate message built first, dropped); the ladders test's lobby deadline 900 ms. Next: T7.
