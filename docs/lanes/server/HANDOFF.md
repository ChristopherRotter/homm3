# Server lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **M1–M4, M6, M5 part 1 and M5 2a closed** (checkpoints SRV-Q12 to SRV-Q24, SRV-Q28 and SRV-Q30, the last 2026-11-13; M6 is discovery: ADR-0021). Part 1 and 2a are in `E16-archive.md`, done in full in `E16-log.md`. **2b in progress** (SRV-Q31, tasks T14 to T19 in the epic's "M5 part 2"; its review list there): T14 (#904, v56), T16 (#907, v58), T15 (#912), T17 (#913, v60) done. **T18 done** (2026-11-14, v62 after the adventure lane's E08 T115b took v61: `[limits] commands_per_turn`, a seat's map commands in a turn, beyond them `TurnLimit`; `commands_per_battle`, a counter per battle, beyond it the computer plays the side and the seat's own are `BattleLimit`; `session::game::bounds`, `CommandBounds` in `GameOptions`, `cross-lane` for the client lane's seat test with the user's consent; `tests/game_limits.rs`). **Its pull request** from `claude/awesome-sagan-lue93w`: watch it until it merges. **Next: T19, 2b's closure** (the end-to-end test, `docs/06-security.md`'s clock, a fresh reviewer agent, the user's checkpoint).
- **The TLS client** (ADR-0021 point 4, `peers::Tls`): rustls over graviola; no default features on a rustls crate (`ring`, `aws-lc` are not GPLv2-compatible; `cargo deny check` after a dependency change).
- **ADR-0018 point 5 as amended:** the mapping matches the engine's enums and the observation's structs exhaustively; a new battle variant or field fails `h3-protocol`'s build until mapped (global trap).
- **The 100 % gate** covers `h3-protocol` (logic); `h3-server` is an adapter outside it, tested end to end.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, 62 since 2026-11-14, T18: the seats' bounds; 61 the adventure lane's E08 T115b: the bonus visitables; 60 T17: the offer's clock; 59 the town lane's E09 T27: recruitment; bumped on every wire change, a field's new meaning or a message's new order too), `RECORD_FORMAT_VERSION` (`h3-protocol`, 5 since 2026-11-12, T11: a whole game's record may hold the start's `GameEvents`), `JOURNAL_FORMAT_VERSION` (`h3-server`, a battle's, 8 since BAT-GEO-002's rear hex; another format's journal is void at start), `GAME_JOURNAL_FORMAT_VERSION` (`h3-server`, a whole game's, `<data>/whole/`, 1 since T9a; T14's and T16's lines and T17's start clock additive).
- The lane owns (`lanes.toml`) `h3-protocol`, `h3-server`, `benches/server/` and E16's files. **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
- **Ids:** questions `SRV-Q<n>`, quirks `SRV-QUIRK-<n>`, stubs `SRV-STUB-<n>`. ADR drafts go to `docs/lanes/server/adr-drafts/` and get a number when the user accepts them. Branches: `server/<task>`.

## Autonomy contract (confirmed by the user, 2026-10-09)

- **Agent decides, marked provisional:** the inner structure of the server crates, names, test layout, error texts, log format; anything a later task can change without touching another crate. Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency, a message shape inside an accepted ADR, a configuration format. List them in the milestone's review list (epic file) and go on.
- **Stop the line:** seams and contracts (`h3-contracts`, the engine API the server calls), heavy dependencies (beyond the decided `tokio` features, `tokio-tungstenite`, `ed25519-dalek` and `argon2` (SRV-Q17): another runtime, an HTTP framework, a TLS stack, other crypto crates, a store), licence questions, changes to the security model (ADR-0008), anything touching another lane's paths. Write the question (`SRV-Q<n>`), stop that task and take the next independent one.

## Standing traps

Moved to `TRAPS.md`: the wire types (`no_std`, serde forms, 64-bit integers), coverage, tests' clients and rates, and the games' threads, locks, journals and outboxes, accounts and ladders, discovery and the TLS client, Windows. Read it before touching `h3-protocol` or `h3-server`.

## Recent sessions

### 2026-11-14 — M5 T18: a seat's commands bounded (v62)

Baseline: `master` 8f32864 (#913, T17). No original data. Kept whole. The user answered at the start: each battle its own counter, past it the computer plays the side; the bounds in `GameOptions` (`cross-lane`). Built T18 (`E16-log.md`): `session::game::bounds`, the two rejections, `GameTurn` filtered, `[limits]`' two fields; the defaults measured on the golden replays and the XL fixtures (26 and 348, 16 and 361: 1 000 and 4 000); `tests/game_limits.rs`. `docs/06-security.md` left to T19 (shared; the log names what it needs). The client lanes' inboxes told. Next: T19.

### 2026-11-14 — M5 T17: the clock hosted (v60)

Baseline: `master` 06a1d18 (#912, T15). No original data. Kept whole (the user: one task unless big). Built T17 (`E16-log.md`): the offers' clock in the configuration, the lobby (v60, after #911 took v59 and `master` was merged in) and the journal's first line; the game's thread reading its clock before every event and waking at the deadline on its own timer; a stop journaling its time (`GameSession::flush_time`); `tests/clock.rs` end to end with real time. The client lanes' inboxes told. Next: T18.

### 2026-11-13 — M5 T15: the survey of the original's turn timer (SRV-Q33)

Baseline: `master` aec8772 (#907, T16). Docs only; `Heroes3.exe`, the manuals and the two texts staged. Kept whole (a survey). Read the timer whole and its 34 references (`docs/epics/E16-timer-survey.md`; 25 map entries, `structs.toml`'s `turn_timer`); compared with SRV-Q31, SRV-Q32, T16 and VCMI. SRV-Q33 asked and answered at once: everything built kept. The map client lane's inbox told (the warnings, the windows). Next: T17.
