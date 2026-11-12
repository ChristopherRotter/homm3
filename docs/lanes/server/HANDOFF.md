# Server lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **M1–M4, M6 and M5 part 1 closed** (checkpoints SRV-Q12 to SRV-Q24 and SRV-Q28, 2026-11-12; M6 is discovery: ADR-0021). Part 1 (`E16-archive.md`, done in full in `E16-log.md`): the whole game's wire mapped exhaustively, the fold with its gate over the 72 golden replays (`tests/game_fold.rs`: 36 parts, 23 exact, the rest named by `SRV-STUB-3` to `-11` and `TWN-STUB-13`), the in-process `session::game::GameSession` with journal, restore and records (format v4, cut at snapshots), the packed `GameReveal`, `h3-server record`, the size gate (`tests/game_bounds.rs`) and the mutation tests (`tests/game_mutation.rs`). T7's pull request merged (#874). **M5 part 2 refined** (SRV-Q29, 2026-11-12): 2a whole games over the network (T8 to T13, the epic's "M5 part 2"), 2b time and 2c spectators and long journals, refined at the previous checkpoint; the server lane builds `h3-game`'s system commands (2b) and `Viewer::Spectator` (2c) in `cross-lane` pull requests (the user's consent). T8a (v45, #878), T8b (v46, #880), T9a (v47, #884) and T9b (v48, #888: the lifecycle) merged. **T10 done** (v49: the server's computer plays the unowned battle sides, `GameSession::computer`, fair by default, `[[map]] computer = "all-seeing"` an option, journaled as the system actor's). Its `shared/` change **#889** (`arch.toml`: `h3-server` may use `h3-battle-ai`) is open from `shared/server-battle-ai` and merges first; T10's pull request follows from `claude/loving-wright-evc1b7` once #889 merged (`master` merged in then; check-lane refuses shared and lane paths together). Watch both until they merge. **Next: T11**, the game's queries on the wire.
- **The TLS client** (ADR-0021 point 4, `peers::Tls`): rustls over graviola; no default features on a rustls crate (`ring`, `aws-lc` are not GPLv2-compatible; `cargo deny check` after a dependency change).
- **ADR-0018 point 5 as amended:** the mapping matches the engine's enums and the observation's structs exhaustively; a new battle variant or field fails `h3-protocol`'s build until mapped (global trap).
- **The 100 % gate** covers `h3-protocol` (logic); `h3-server` is an adapter outside it, tested end to end.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, 49 since 2026-11-12, T10: a map offer's `computer`; 48 T9b's `FetchRecord`; bumped on every wire change, a field's new meaning or a message's new order too), `RECORD_FORMAT_VERSION` (`h3-protocol`, 4 since 2026-11-10: a whole game's record, `GameRecordHeader`), `JOURNAL_FORMAT_VERSION` (`h3-server`, a battle's, 8 since BAT-GEO-002's rear hex; another format's journal is void at start), `GAME_JOURNAL_FORMAT_VERSION` (`h3-server`, a whole game's, `<data>/whole/`, 1 since T9a).
- The lane owns (`lanes.toml`) `h3-protocol`, `h3-server`, `benches/server/` and E16's files. **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
- **Ids:** questions `SRV-Q<n>`, quirks `SRV-QUIRK-<n>`, stubs `SRV-STUB-<n>`. ADR drafts go to `docs/lanes/server/adr-drafts/` and get a number when the user accepts them. Branches: `server/<task>`.

## Autonomy contract (confirmed by the user, 2026-10-09)

- **Agent decides, marked provisional:** the inner structure of the server crates, names, test layout, error texts, log format; anything a later task can change without touching another crate. Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency, a message shape inside an accepted ADR, a configuration format. List them in the milestone's review list (epic file) and go on.
- **Stop the line:** seams and contracts (`h3-contracts`, the engine API the server calls), heavy dependencies (beyond the decided `tokio` features, `tokio-tungstenite`, `ed25519-dalek` and `argon2` (SRV-Q17): another runtime, an HTTP framework, a TLS stack, other crypto crates, a store), licence questions, changes to the security model (ADR-0008), anything touching another lane's paths. Write the question (`SRV-Q<n>`), stop that task and take the next independent one.

## Standing traps

Moved to `TRAPS.md`: the wire types (`no_std`, serde forms, 64-bit integers), coverage, tests' clients and rates, and the games' threads, locks, journals and outboxes, accounts and ladders, discovery and the TLS client, Windows. Read it before touching `h3-protocol` or `h3-server`.

## Recent sessions

### 2026-11-12 — M5 T10: the unowned sides played by the server (v49)

Baseline: `master` acd9863 (#888, CI green). No original data. The user kept T10 whole and chose a `shared/` change first (#889, `arch.toml`), over going through `h3-sim`. Built T10 (`E16-log.md`): `GameSession::computer` and its `unowned` loop (the battle session's `Bot` trait), the wire's `Computer` on `MapOffer`, `[[map]] computer`, `whole::ComputerBot` on `h3_battle_ai`; `tests/game_computer.rs`, `tests/whole.rs` end to end (a monster battle over the network, a cut journal answered at a restart). Next: T11.

### 2026-11-12 — M5 T9b: a hosted whole game's lifecycle (v48)

Baseline: `master` cb89a8f (#884). No original data. The user kept T9b whole. Built T9b (`E16-log.md`): `GameSession::suspend` and `FetchRecord`, `Lobby::reopen_map` and `to_end`; the whole journals restored at start after the battles', `Hosted::PausedMap` and `bind_map`'s resume, the sweep's pause and void, the drain not waiting for a running whole game; `tests/whole.rs` end to end. Left: the card's map rulesets. Next: T10.

### 2026-11-12 — M5 T9a: a whole game hosted live (v47)

Baseline: `master` 21d6ba0 (#882; the full verify passed in the sandbox). No original data. The user split T9 (T9a live hosting, T9b the lifecycle). Built T9a (`E16-log.md`): the wire's map offers and whole games, `CreateMapGame`, `GameSession`'s host accessors; `[[map]]` pinned by hash, the whole game's thread and journal (format 1, kept untouched at start: `SRV-STUB-12`); `tests/whole.rs` end to end. Next: T9b.
