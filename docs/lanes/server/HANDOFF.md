# Server lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **M1–M4, M6, M5 part 1 and M5 2a closed** (checkpoints SRV-Q12 to SRV-Q24, SRV-Q28 and SRV-Q30, the last 2026-11-13; M6 is discovery: ADR-0021). Part 1 and 2a are in `E16-archive.md`, done in full in `E16-log.md`. **2a** (T8a to T13, protocol v45 to v51): whole games over the network, the seats by account or token, numbered commands and resends, the host's pause, the lobby's map offers, the hosting with its journal (`<data>/whole/`) and restore, the server's computer on the unowned battle sides, the game's queries; T13's end-to-end test (`h3-server` `tests/whole.rs`, two clients through a restart inside a battle; a test-only seed pin, `config::MapOffer::seeds`), `docs/06-security.md`'s networked whole game. The reviewer's major finding (no command limit for a whole game) is documented and goes to 2b (a seat's commands per turn) and 2c (the restart's cost, the session's copy of the commands), SRV-Q30. **T13's pull request** from `claude/loving-babbage-aylytb`, `cross-lane` (`docs/06-security.md`): watch it until it merges. **2b refined** (SRV-Q31, 2026-11-13, in T13's session): the original's turn duration per offer, the timer running on in battles (the computer plays a timed-out side), a chess timer between two players' heroes, the turn timer alone for a disconnected player, the level-up's time an offer's. **Next: T14, `h3-game`'s system commands** (`cross-lane`, ADR-0017 point 2 amended there; the adventure lane's `TRAPS.md` first); T15's survey wherever the original data are staged.
- **The TLS client** (ADR-0021 point 4, `peers::Tls`): rustls over graviola; no default features on a rustls crate (`ring`, `aws-lc` are not GPLv2-compatible; `cargo deny check` after a dependency change).
- **ADR-0018 point 5 as amended:** the mapping matches the engine's enums and the observation's structs exhaustively; a new battle variant or field fails `h3-protocol`'s build until mapped (global trap).
- **The 100 % gate** covers `h3-protocol` (logic); `h3-server` is an adapter outside it, tested end to end.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, 52 since 2026-11-12, the adventure lane's E08 T114a2: a map garrison's exchange; bumped on every wire change, a field's new meaning or a message's new order too), `RECORD_FORMAT_VERSION` (`h3-protocol`, 5 since 2026-11-12, T11: a whole game's record may hold the start's `GameEvents`), `JOURNAL_FORMAT_VERSION` (`h3-server`, a battle's, 8 since BAT-GEO-002's rear hex; another format's journal is void at start), `GAME_JOURNAL_FORMAT_VERSION` (`h3-server`, a whole game's, `<data>/whole/`, 1 since T9a).
- The lane owns (`lanes.toml`) `h3-protocol`, `h3-server`, `benches/server/` and E16's files. **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
- **Ids:** questions `SRV-Q<n>`, quirks `SRV-QUIRK-<n>`, stubs `SRV-STUB-<n>`. ADR drafts go to `docs/lanes/server/adr-drafts/` and get a number when the user accepts them. Branches: `server/<task>`.

## Autonomy contract (confirmed by the user, 2026-10-09)

- **Agent decides, marked provisional:** the inner structure of the server crates, names, test layout, error texts, log format; anything a later task can change without touching another crate. Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency, a message shape inside an accepted ADR, a configuration format. List them in the milestone's review list (epic file) and go on.
- **Stop the line:** seams and contracts (`h3-contracts`, the engine API the server calls), heavy dependencies (beyond the decided `tokio` features, `tokio-tungstenite`, `ed25519-dalek` and `argon2` (SRV-Q17): another runtime, an HTTP framework, a TLS stack, other crypto crates, a store), licence questions, changes to the security model (ADR-0008), anything touching another lane's paths. Write the question (`SRV-Q<n>`), stop that task and take the next independent one.

## Standing traps

Moved to `TRAPS.md`: the wire types (`no_std`, serde forms, 64-bit integers), coverage, tests' clients and rates, and the games' threads, locks, journals and outboxes, accounts and ladders, discovery and the TLS client, Windows. Read it before touching `h3-protocol` or `h3-server`.

## Recent sessions

### 2026-11-13 — M5 T13: 2a's closure (SRV-Q30)

Baseline: `master` dca8bbb (#897, T12; CI green). No original data. The user chose a test-only seed pin and `docs/06-security.md` cross-lane in this PR. Built T13 (`E16-log.md`): the end-to-end test, the security doc, a fresh reviewer (one major finding, documented and placed in 2b and 2c; one minor, fixed: a new journal directory's entry synced). `master`'s v52 merged, the adventure lane's note reviewed (a garrison's index on the wire: its inbox). 2a closed. 2b refined with the user (SRV-Q31; its tasks T14 to T19 in the epic). Next: T14.

### 2026-11-12 — M5 T12: T7b's two findings (v51)

Baseline: `master` 698eb84 (#895, its code verified). No original data. Built T12 (`E16-log.md`): `BattleKey` (a viewer's open battle by heroes and tile), `Viewer::refresh` inside a battle; `tests/game_session.rs`' rejections inside a battle; the client lane's inbox told (v51). Next: T13.

### 2026-11-12 — M5 T11: the game's queries on the wire (v50)

Baseline: `master` 6b7b348 (#891, CI green; the sandbox's full verify passed). No original data. The user kept T11 whole and added a battle's targets inside a whole game. Built T11 (`E16-log.md`): `GameQuery`/`GameAnswer` (`wire::game::query`, `map::game::answer`), the moves' path tree with its reader, `GameSession::query` and `opening`; `tests/game_query.rs`, the bounds and mutation tests, `tests/whole.rs` end to end. Next: T12.
