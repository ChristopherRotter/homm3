# Server lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **M1–M4, M6 and M5 part 1 closed** (checkpoints SRV-Q12 to SRV-Q24 and SRV-Q28, 2026-11-12; M6 is discovery: ADR-0021). Part 1 (`E16-archive.md`, done in full in `E16-log.md`): the whole game's wire mapped exhaustively, the fold with its gate over the 72 golden replays (`tests/game_fold.rs`: 36 parts, 23 exact, the rest named by `SRV-STUB-3` to `-11` and `TWN-STUB-13`), the in-process `session::game::GameSession` with journal, restore and records (format v4, cut at snapshots), the packed `GameReveal`, `h3-server record`, the size gate (`tests/game_bounds.rs`) and the mutation tests (`tests/game_mutation.rs`). T7's pull request merged (#874). **M5 part 2 refined** (SRV-Q29, 2026-11-12): 2a whole games over the network (T8 to T13, the epic's "M5 part 2"), 2b time and 2c spectators and long journals, refined at the previous checkpoint; the server lane builds `h3-game`'s system commands (2b) and `Viewer::Spectator` (2c) in `cross-lane` pull requests (the user's consent). **T8a done** (the whole game's seats and connections, v45; #878 merged). **T8b done** (the commands' numbers against resends, the host's pause, `reopen` with the seats taken, v46): **its pull request is open from `server/e16-m5-t8b`; watch it until it merges.** **Next: T9**, a whole game hosted by `h3-server`.
- **The TLS client** (ADR-0021 point 4, `peers::Tls`): rustls over graviola; no default features on a rustls crate (`ring`, `aws-lc` are not GPLv2-compatible; `cargo deny check` after a dependency change).
- **ADR-0018 point 5 as amended:** the mapping matches the engine's enums and the observation's structs exhaustively; a new battle variant or field fails `h3-protocol`'s build until mapped (global trap).
- **The 100 % gate** covers `h3-protocol` (logic); `h3-server` is an adapter outside it, tested end to end.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, 46 since 2026-11-12, T8b: a seat's numbered commands, `GameJoined`'s `last`, `GamePauseAsked`, `GameReject::AlreadyAsked`/`Stale`; 45 T8a's seats; bumped on every wire change, a field's new meaning or a message's new order too), `RECORD_FORMAT_VERSION` (`h3-protocol`, 4 since 2026-11-10: a whole game's record, `GameRecordHeader`), `JOURNAL_FORMAT_VERSION` (`h3-server`, 8 since BAT-GEO-002's rear hex; another format's journal is void at start).
- The lane owns (`lanes.toml`) `h3-protocol`, `h3-server`, `benches/server/` and E16's files. **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
- **Ids:** questions `SRV-Q<n>`, quirks `SRV-QUIRK-<n>`, stubs `SRV-STUB-<n>`. ADR drafts go to `docs/lanes/server/adr-drafts/` and get a number when the user accepts them. Branches: `server/<task>`.

## Autonomy contract (confirmed by the user, 2026-10-09)

- **Agent decides, marked provisional:** the inner structure of the server crates, names, test layout, error texts, log format; anything a later task can change without touching another crate. Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency, a message shape inside an accepted ADR, a configuration format. List them in the milestone's review list (epic file) and go on.
- **Stop the line:** seams and contracts (`h3-contracts`, the engine API the server calls), heavy dependencies (beyond the decided `tokio` features, `tokio-tungstenite`, `ed25519-dalek` and `argon2` (SRV-Q17): another runtime, an HTTP framework, a TLS stack, other crypto crates, a store), licence questions, changes to the security model (ADR-0008), anything touching another lane's paths. Write the question (`SRV-Q<n>`), stop that task and take the next independent one.

## Standing traps

Moved to `TRAPS.md`: the wire types (`no_std`, serde forms, 64-bit integers), coverage, tests' clients and rates, and the games' threads, locks, journals and outboxes, accounts and ladders, discovery and the TLS client, Windows. Read it before touching `h3-protocol` or `h3-server`.

## Recent sessions

### 2026-11-12 — M5 T8b: numbers, pause, reopen (v46)

Baseline: `server/e16-m5-t8a` 7595985 (T8a). No original data. The user merged #877; T8a's pull request opened (#878, `master` merged in: the three server docs conflicted only by #877's squash, the branch's taken). T8b's worker built the session's part and was stopped by a restart (work-in-progress commit 1aa9f0d); the user asked the runner to finish it: `tests/game_resume.rs`, #878's head merged in, the docs (`E16-log.md`). Next: T9.

### 2026-11-12 — M5 T8a: the whole game's seats and connections (v45)

Baseline: `claude/gifted-franklin-4lkqk3` 944f72a (#877; the setup's light gates on its fingerprint; the scoped verify passed). No original data. Built T8a (`E16-log.md`): `GameSession::new` hosts a game for connections (a seat per player, tokens from the host), `handle` with `GameJoin` (the battle's claim shared: `refusal`, `handed`), `GameCommand`, `Leave`; the start at the last seat's join, a re-take's fresh snapshot and turn, frames routed to the seats' holders (`route`), the in-process host holding every seat (part 1's uses unchanged); `h3-server`'s answers without a game. The 2a review list started. Next: T8b.

### 2026-11-12 — M5 part 2's refinement (SRV-Q29)

Baseline: `master` d1b5147 (#874, T7, merged). No original data. Refined part 2 with the user (SRV-Q29): three increments (2a network whole games, 2b time, 2c spectators and long journals), the engine's system commands and spectator viewer the server lane's in `cross-lane` pull requests, every map player a seat in 2a; 2a's tasks T8 (in two) to T13 in the epic; the adventure and town lanes' inboxes told; the refinement in its own pull request. Next: T8a.
