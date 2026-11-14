# Server lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **M1–M4, M6, M5 part 1 and M5 2a closed** (checkpoints SRV-Q12 to SRV-Q24, SRV-Q28 and SRV-Q30, the last 2026-11-13; M6 is discovery: ADR-0021). Part 1 and 2a are in `E16-archive.md`, done in full in `E16-log.md`. **2b in progress** (SRV-Q31, tasks T14 to T19 in the epic's "M5 part 2"; its review list there): T14 (#904, v56), T16 (#907, v58: the clock in the session; SRV-Q32), T15 (#912, the survey; SRV-Q33) done. **T17 done** (2026-11-14, v59: `[[map]] turn_minutes`, `level_up_ms`, `battle_ms`, `battle_increment_ms`; the offer's clock in the lobby, `OfferClock`, and in the journal's first line; the game's thread waking at `GameSession::deadline`, a stop journaling its time, `flush_time`; `tests/clock.rs`). **Its pull request** from `claude/admiring-lovelace-jlv0hu`: watch it until it merges. **Next: T18, a seat's commands per turn** (`[limits]`' bound, its default from the golden replays' and XL fixtures' largest; protocol step), then T19, 2b's closure.
- **The TLS client** (ADR-0021 point 4, `peers::Tls`): rustls over graviola; no default features on a rustls crate (`ring`, `aws-lc` are not GPLv2-compatible; `cargo deny check` after a dependency change).
- **ADR-0018 point 5 as amended:** the mapping matches the engine's enums and the observation's structs exhaustively; a new battle variant or field fails `h3-protocol`'s build until mapped (global trap).
- **The 100 % gate** covers `h3-protocol` (logic); `h3-server` is an adapter outside it, tested end to end.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, 59 since 2026-11-14, T17: the offer's clock; 58 T16: the clock; 57 the adventure lane's E08 T115a: the hero's bonuses; bumped on every wire change, a field's new meaning or a message's new order too), `RECORD_FORMAT_VERSION` (`h3-protocol`, 5 since 2026-11-12, T11: a whole game's record may hold the start's `GameEvents`), `JOURNAL_FORMAT_VERSION` (`h3-server`, a battle's, 8 since BAT-GEO-002's rear hex; another format's journal is void at start), `GAME_JOURNAL_FORMAT_VERSION` (`h3-server`, a whole game's, `<data>/whole/`, 1 since T9a; T14's and T16's lines and T17's start clock additive).
- The lane owns (`lanes.toml`) `h3-protocol`, `h3-server`, `benches/server/` and E16's files. **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
- **Ids:** questions `SRV-Q<n>`, quirks `SRV-QUIRK-<n>`, stubs `SRV-STUB-<n>`. ADR drafts go to `docs/lanes/server/adr-drafts/` and get a number when the user accepts them. Branches: `server/<task>`.

## Autonomy contract (confirmed by the user, 2026-10-09)

- **Agent decides, marked provisional:** the inner structure of the server crates, names, test layout, error texts, log format; anything a later task can change without touching another crate. Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency, a message shape inside an accepted ADR, a configuration format. List them in the milestone's review list (epic file) and go on.
- **Stop the line:** seams and contracts (`h3-contracts`, the engine API the server calls), heavy dependencies (beyond the decided `tokio` features, `tokio-tungstenite`, `ed25519-dalek` and `argon2` (SRV-Q17): another runtime, an HTTP framework, a TLS stack, other crypto crates, a store), licence questions, changes to the security model (ADR-0008), anything touching another lane's paths. Write the question (`SRV-Q<n>`), stop that task and take the next independent one.

## Standing traps

Moved to `TRAPS.md`: the wire types (`no_std`, serde forms, 64-bit integers), coverage, tests' clients and rates, and the games' threads, locks, journals and outboxes, accounts and ladders, discovery and the TLS client, Windows. Read it before touching `h3-protocol` or `h3-server`.

## Recent sessions

### 2026-11-14 — M5 T17: the clock hosted (v59)

Baseline: `master` 06a1d18 (#912, T15). No original data. Kept whole (the user: one task unless big). Built T17 (`E16-log.md`): the offers' clock in the configuration, the lobby (v59) and the journal's first line; the game's thread reading its clock before every event and waking at the deadline on its own timer; a stop journaling its time (`GameSession::flush_time`); `tests/clock.rs` end to end with real time. The client lanes' inboxes told. Next: T18.

### 2026-11-13 — M5 T15: the survey of the original's turn timer (SRV-Q33)

Baseline: `master` aec8772 (#907, T16). Docs only; `Heroes3.exe`, the manuals and the two texts staged. Kept whole (a survey). Read the timer whole and its 34 references (`docs/epics/E16-timer-survey.md`; 25 map entries, `structs.toml`'s `turn_timer`); compared with SRV-Q31, SRV-Q32, T16 and VCMI. SRV-Q33 asked and answered at once: everything built kept. The map client lane's inbox told (the warnings, the windows). Next: T17.

### 2026-11-13 — M5 T16: the clock in the session (v58)

Baseline: `master` 999bb0e (#904, T14; its code verified). No original data. The user chose at the start all of T16 in one go and the chess timer's allowances (SRV-Q32: Fischer's base and increment, the attacker's turn timer stopped meanwhile, every turn timer stopped while a level-up holds). Built T16 (`E16-log.md`): the session's timers (`session/game/clock.rs`), `elapse`, `deadline`, the system's steps, `GameEntry::Elapsed` (the journal's `elapsed` line, its format still 1), `GameClock` (v58, after #906 took v57 and `master` was merged in); `tests/game_clock.rs`; the client lane's seat test given the clock (`cross-lane`, the user's consent). The client lanes' inboxes told. Next: T17.
