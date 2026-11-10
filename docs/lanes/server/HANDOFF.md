# Server lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **M1–M4 and M6 closed** (checkpoints SRV-Q12 to SRV-Q24; M6 is discovery: ADR-0021). **M5 part 1** (SRV-Q25; the epic's "M5 part 1", the QUEUE's tasks): T0 (#799), T1 (#803; **events only**, the design note §14), T2 (#805, the observation's wire) and **T3** (in three parts, the user) done 2026-11-09: T3a the game's commands both ways and its rejections, T3b its events projected (`wire::game::{GameCommand, GameReject, GameEvent}`, `map::game::{command, command_to_engine, reject, events}`; the town lane's build of #809 mapped), T3c1 the fold `GameObservation::fold` (`wire/game/fold.rs`) and T3c2 its gate over the 55 golden replays (`tests/game_fold.rs`: 16 of 35 parts exact, the rest named by the stubs `SRV-STUB-1` to `-13`, SRV-Q26's and SRV-Q27's gaps, in the adventure and town lanes' inboxes). **Open pull request:** [#814](https://github.com/ChristopherRotter/homm3/pull/814), T3's, from `claude/adoring-mayer-piwsx1` (T3a and T3b first pushed to `claude/youthful-newton-lrlkoz`, T3c1 to `claude/gracious-bell-defyp6`); CI's run on it is the record. **Next:** T4 (the game's manifest and key tables). **The review list so far:** the epic's "M5 part 1".
- **The TLS client** (ADR-0021 point 4, `peers::Tls`): rustls over graviola; no default features on a rustls crate (`ring`, `aws-lc` are not GPLv2-compatible; `cargo deny check` after a dependency change).
- **ADR-0018 point 5 as amended:** the mapping matches the engine's enums and the observation's structs exhaustively; a new battle variant or field fails `h3-protocol`'s build until mapped (global trap).
- **The 100 % gate** covers `h3-protocol` (logic); `h3-server` is an adapter outside it, tested end to end.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, 38 since 2026-11-09, CLT-Q20 and CLT-STUB-3; 36 and 37 were the client lane's; bumped on every wire change, a field's new meaning or a message's new order too), `RECORD_FORMAT_VERSION` (`h3-protocol`, 3 since 2026-11-09: the start's `Events`, the last `Snapshot`), `JOURNAL_FORMAT_VERSION` (`h3-server`, 8 since BAT-GEO-002's rear hex; another format's journal is void at start).
- The lane owns (`lanes.toml`) `h3-protocol`, `h3-server`, `benches/server/` and E16's files. **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
- **Ids:** questions `SRV-Q<n>`, quirks `SRV-QUIRK-<n>`, stubs `SRV-STUB-<n>`. ADR drafts go to `docs/lanes/server/adr-drafts/` and get a number when the user accepts them. Branches: `server/<task>`.

## Autonomy contract (confirmed by the user, 2026-10-09)

- **Agent decides, marked provisional:** the inner structure of the server crates, names, test layout, error texts, log format; anything a later task can change without touching another crate. Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency, a message shape inside an accepted ADR, a configuration format. List them in the milestone's review list (epic file) and go on.
- **Stop the line:** seams and contracts (`h3-contracts`, the engine API the server calls), heavy dependencies (beyond the decided `tokio` features, `tokio-tungstenite`, `ed25519-dalek` and `argon2` (SRV-Q17): another runtime, an HTTP framework, a TLS stack, other crypto crates, a store), licence questions, changes to the security model (ADR-0008), anything touching another lane's paths. Write the question (`SRV-Q<n>`), stop that task and take the next independent one.

## Standing traps

Moved to `TRAPS.md`: the wire types (`no_std`, serde forms, 64-bit integers), coverage, tests' clients and rates, and the games' threads, locks, journals and outboxes, accounts and ladders, discovery and the TLS client, Windows. Read it before touching `h3-protocol` or `h3-server`.

## Recent sessions

### 2026-11-09 — M5 T3c2: the fold's stubs and gate; T3's pull request

Baseline: the branch's T3c1 on `master` d1b90fb (CI green); the sandbox's full engine verify of that code ran at the start. No original data. Built T3c2 (`E16-log.md`): `SRV-STUB-1` to `-13` with their markers in the fold, the adventure and town lanes' inbox notes, the harness as the gate (the hero details and the decision split; the reveal-only rule). `master` 737d007 (#811) merged in, the numbers unmoved. No message, no version step. The scoped verify passed (coverage included). Then T3's one pull request, #814.

### 2026-11-09 — M5 T3c1: the game's fold and its harness

Baseline: the branch's T3a and T3b on `master` 1e8568d, the sandbox's full verify passed (`verify-once`; CI's `master` run is red by `h3-map`'s proptest, which passed here). No original data. Built the fold (`GameObservation::fold`) and the harness over the golden replays (`E16-log.md`, its table): 14 of 31 parts equal `observe` after every command for every viewer; the rest are SRV-Q26's (a) to (d) and SRV-Q27's decision views and derived values. One fold error found and fixed (a Leave Guards dialog's guards on a mine without any). `master` brought the town lane's build (#809) mid-session: mapped here. No message, no version step. Next: T3c2.

### 2026-11-09 — M5 T3a and T3b: the game's commands, rejections and events on the wire

Baseline: `master` 1e8568d (#808 merged; CI's run on it in progress at the start); the sandbox's full verify passed up to its tests, its coverage stopped (CI's is the record). No original data. Built T3a and T3b (`E16-log.md`): the wire types, the mapping (the events projected), test-only inverses, hand-built lists of every variant and played fixture games. No message, no version step (T5). The user (SRV-Q27): the game's battle by snapshots, the further gaps as SRV-Q26's, T3c as T3c1 and T3c2. Next: T3c1, in a fresh session. CI on `master` 1e8568d is red, not ours: `h3-map`'s proptest `every_step_keeps_the_commands_invariants` (the adventure lane's inbox, the same day); the pull request waits for its fix.
