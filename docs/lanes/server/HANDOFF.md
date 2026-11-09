# Server lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **M1–M4 and M6 closed** (checkpoints SRV-Q12 to SRV-Q24; M6 is discovery: ADR-0021). **M5 part 1** (SRV-Q25; the epic's "M5 part 1", the QUEUE's tasks): T0 (#799), T1 (#803; **events only**, the design note §14; the fold's gaps are the adventure and town lanes', SRV-Q26) and T2 (#805, the observation's wire) done; #808 (the files crate `h3-files`) merged 2026-11-09. **T3** in three parts (the user; the epic's T3), one pull request after T3c, none open yet: **T3a and T3b done** on `claude/youthful-newton-lrlkoz` (the game's commands both ways and its rejections, `wire::game::{GameCommand, GameReject}`, `map::game::{command, command_to_engine, reject}`; its events projected, `wire::game::GameEvent`, `map::game::events`). **Next:** T3c in two parts (the user, SRV-Q27; the epic's T3c): **T3c1** the fold of what the events carry and a harness over the 55 golden replays (through `h3-files`) reporting where the fold and `observe` differ, then T3c2 (the stubs, the inbox notes, the gate), then the one pull request. **The review list so far:** the epic's "M5 part 1".
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

### 2026-11-09 — M5 T3a and T3b: the game's commands, rejections and events on the wire

Baseline: `master` 1e8568d (#808 merged; CI's run on it in progress at the start); the sandbox's full verify passed up to its tests, its coverage stopped (CI's is the record). No original data. Built T3a and T3b (`E16-log.md`): the wire types, the mapping (the events projected), test-only inverses, hand-built lists of every variant and played fixture games. No message, no version step (T5). The user (SRV-Q27): the game's battle by snapshots, the further gaps as SRV-Q26's, T3c as T3c1 and T3c2. Next: T3c1, in a fresh session. CI on `master` 1e8568d is red, not ours: `h3-map`'s proptest `every_step_keeps_the_commands_invariants` (the adventure lane's inbox, the same day); the pull request waits for its fix.

### 2026-11-09 — T3's split; the files crate `h3-files`

Baseline: `master` a14e30f (its code verified). No original data. The user: T3 in three parts (T3a commands and rejections, T3b events, T3c the fold), one PR after T3c; the messages and v39 with T5's session; the fold's golden-replay test needs `h3-cli`'s loaders, which the user had moved into a new crate first (`h3-files`, done on the branch, see Current state). Then the user stopped the session before T3a.

### 2026-11-09 — M5 T2: the game's observation on the wire

Baseline: `master` ae7d260, CI green; the scoped verify's steps up to its tests passed (a VM restart cut its coverage). No original data. The user split T2 (T2a the adventure map, T2b the rest), both in #805. Built: `wire::game` (the planes as `WireU64` words, the tiles as a palette and an index per tile: lossless, 4 to 64 views on the fixture maps; the decisions, heroes' details, towns, the battle), `map::game::observe`, a test-only inverse checked on random games over five fixture maps. No message, no version step.
