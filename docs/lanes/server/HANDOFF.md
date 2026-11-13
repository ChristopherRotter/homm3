# Server lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **M1–M4, M6 and M5 part 1 closed** (checkpoints SRV-Q12 to SRV-Q24 and SRV-Q28, 2026-11-12; M6 is discovery: ADR-0021). Part 1 (`E16-archive.md`, done in full in `E16-log.md`): the whole game's wire mapped exhaustively, the fold with its gate over the 72 golden replays (`tests/game_fold.rs`: 36 parts, 23 exact, the rest named by `SRV-STUB-3` to `-11` and `TWN-STUB-13`), the in-process `session::game::GameSession` with journal, restore and records (format v4, cut at snapshots), the packed `GameReveal`, `h3-server record`, the size gate (`tests/game_bounds.rs`) and the mutation tests (`tests/game_mutation.rs`). T7's pull request merged (#874). **M5 part 2 refined** (SRV-Q29, 2026-11-12): 2a whole games over the network (T8 to T13, the epic's "M5 part 2"), 2b time and 2c spectators and long journals, refined at the previous checkpoint; the server lane builds `h3-game`'s system commands (2b) and `Viewer::Spectator` (2c) in `cross-lane` pull requests (the user's consent). T8a (v45, #878), T8b (v46, #880), T9a (v47, #884), T9b (v48, #888) merged. T10 (v49, #891) and T11 (v50, record format v5, #895: the game's queries) merged. **T12 done** (v51: a viewer's open battle by its heroes and tile; the fresh `GameSnapshot` inside a battle too, so a player's rejections there stay within its record's bound); its pull request from `claude/confident-knuth-subyc4` (watch it until it merges). **Next: T13**, 2a's closure (end-to-end test, `docs/06-security.md`, reviewer agent, checkpoint).
- **The TLS client** (ADR-0021 point 4, `peers::Tls`): rustls over graviola; no default features on a rustls crate (`ring`, `aws-lc` are not GPLv2-compatible; `cargo deny check` after a dependency change).
- **ADR-0018 point 5 as amended:** the mapping matches the engine's enums and the observation's structs exhaustively; a new battle variant or field fails `h3-protocol`'s build until mapped (global trap).
- **The 100 % gate** covers `h3-protocol` (logic); `h3-server` is an adapter outside it, tested end to end.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, 51 since 2026-11-12, T12: a mid-battle `GameSnapshot`, a battle's `Snapshot` by identity; bumped on every wire change, a field's new meaning or a message's new order too), `RECORD_FORMAT_VERSION` (`h3-protocol`, 5 since 2026-11-12, T11: a whole game's record may hold the start's `GameEvents`), `JOURNAL_FORMAT_VERSION` (`h3-server`, a battle's, 8 since BAT-GEO-002's rear hex; another format's journal is void at start), `GAME_JOURNAL_FORMAT_VERSION` (`h3-server`, a whole game's, `<data>/whole/`, 1 since T9a).
- The lane owns (`lanes.toml`) `h3-protocol`, `h3-server`, `benches/server/` and E16's files. **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
- **Ids:** questions `SRV-Q<n>`, quirks `SRV-QUIRK-<n>`, stubs `SRV-STUB-<n>`. ADR drafts go to `docs/lanes/server/adr-drafts/` and get a number when the user accepts them. Branches: `server/<task>`.

## Autonomy contract (confirmed by the user, 2026-10-09)

- **Agent decides, marked provisional:** the inner structure of the server crates, names, test layout, error texts, log format; anything a later task can change without touching another crate. Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency, a message shape inside an accepted ADR, a configuration format. List them in the milestone's review list (epic file) and go on.
- **Stop the line:** seams and contracts (`h3-contracts`, the engine API the server calls), heavy dependencies (beyond the decided `tokio` features, `tokio-tungstenite`, `ed25519-dalek` and `argon2` (SRV-Q17): another runtime, an HTTP framework, a TLS stack, other crypto crates, a store), licence questions, changes to the security model (ADR-0008), anything touching another lane's paths. Write the question (`SRV-Q<n>`), stop that task and take the next independent one.

## Standing traps

Moved to `TRAPS.md`: the wire types (`no_std`, serde forms, 64-bit integers), coverage, tests' clients and rates, and the games' threads, locks, journals and outboxes, accounts and ladders, discovery and the TLS client, Windows. Read it before touching `h3-protocol` or `h3-server`.

## Recent sessions

### 2026-11-13 — M5 T13: 2a's closure (in progress)

Baseline: `master` dca8bbb (#897, T12; CI green). No original data. Branch `claude/loving-babbage-aylytb`. Done: the end-to-end test (`h3-server` `tests/whole.rs`, `two_clients_play_a_whole_game_...`; a test-only seed pin `config::MapOffer::seeds`, the user's choice), `docs/06-security.md`'s networked whole game (cross-lane in T13's PR, the user's choice). Next: the reviewer agent over `a396c3f..HEAD` (the two crates), its fixes, the docs (log, epic, queue), the PR, the user's checkpoint.

### 2026-11-12 — M5 T12: T7b's two findings (v51)

Baseline: `master` 698eb84 (#895, its code verified). No original data. Built T12 (`E16-log.md`): `BattleKey` (a viewer's open battle by heroes and tile), `Viewer::refresh` inside a battle; `tests/game_session.rs`' rejections inside a battle; the client lane's inbox told (v51). Next: T13.

### 2026-11-12 — M5 T11: the game's queries on the wire (v50)

Baseline: `master` 6b7b348 (#891, CI green; the sandbox's full verify passed). No original data. The user kept T11 whole and added a battle's targets inside a whole game. Built T11 (`E16-log.md`): `GameQuery`/`GameAnswer` (`wire::game::query`, `map::game::answer`), the moves' path tree with its reader, `GameSession::query` and `opening`; `tests/game_query.rs`, the bounds and mutation tests, `tests/whole.rs` end to end. Next: T12.

### 2026-11-12 — M5 T10: the unowned sides played by the server (v49)

Baseline: `master` acd9863 (#888, CI green). No original data. The user kept T10 whole and chose a `shared/` change first (#889, `arch.toml`), over going through `h3-sim`. Built T10 (`E16-log.md`): `GameSession::computer` and its `unowned` loop (the battle session's `Bot` trait), the wire's `Computer` on `MapOffer`, `[[map]] computer`, `whole::ComputerBot` on `h3_battle_ai`; `tests/game_computer.rs`, `tests/whole.rs` end to end (a monster battle over the network, a cut journal answered at a restart). Next: T11.
