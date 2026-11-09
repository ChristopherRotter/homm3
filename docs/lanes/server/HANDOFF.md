# Server lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **M1–M4 and M6 closed** (checkpoints SRV-Q12 to SRV-Q24; M6 is discovery: ADR-0021). **M5 part 1** (SRV-Q25; the epic's "M5 part 1", the QUEUE's tasks): T0 (#799), T1 (#803; **events only**, the design note §14; the fold's gaps are the adventure and town lanes', SRV-Q26) and T2a (the adventure map's wire, `wire::game`, `map::game`) done 2026-11-09; T2 is two pull requests (the user). **Open pull requests:** T2a's (this branch, `claude/determined-keller-jzd3w8`). **Next:** T2b (the rest of the observation: players, kingdom, decisions, heroes' details, towns, the battle, `map::game::observe`). **The review list so far:** the epic's "M5 part 1".
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

### 2026-11-09 — M5 T2a: the adventure map on the wire

Baseline: `master` ae7d260, CI green; the scoped verify's steps up to its tests passed (a VM restart cut its coverage). No original data. The user split T2 (T2a the adventure map, T2b the rest). Built: `wire::game` (the planes as `WireU64` words, the tiles as a palette and an index per tile: lossless, 4 to 64 views on the fixture maps), `map::game::adventure`, a test-only inverse checked on random games over five fixture maps. No message, no version step.

### 2026-11-09 — M5 T1: the measurement, events only

Baseline: `master` a0d5504 by the sandbox's full verify (CI's last two `master` runs never started: no job). No original data or computer folder: the XL fixtures with eight players stood in for the shipped maps. A scratch `h3-server` example (not kept) measured the diff at 22 times an `apply` on the golden replays and 730 to 860 times on the XL maps: events only (design note §14). The fold's gaps: SRV-Q26 (the user: foldable engine events and a query), the adventure and town lanes' inboxes. PR #803.

### 2026-11-09 — the battle-wire inbox items, M5 part 1 refined

Baseline: `master` 2e5e08f (CI's fingerprint, the setup's light gates). No original data. The user (SRV-Q25): the game's types exhaustive, diff or events by measurement, CLT-STUB-3 and CLT-Q20 first. Built: the start's events at the game's start and the last snapshot before the end (v38, record v3); the client lane's inbox told. M5 part 1's task list in the epic and the QUEUE. PRs #800 and #799 (see Current state).
