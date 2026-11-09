# Server lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **M1–M4 and M6 closed** (checkpoints SRV-Q12 to SRV-Q24; M6 is discovery: ADR-0021). **M5 part 1 refined 2026-11-09** (SRV-Q25: the game's types mapped exhaustively; diff or events only by T1's measurement; the epic's "M5 part 1", the QUEUE's T0–T7). The inbox's battle-wire items CLT-STUB-3 and CLT-Q20 done (protocol v38, record format v3; `E16-log.md`). **Open pull requests:** this branch's (v38, the refinement) and T0's `shared/` one, [#799](https://github.com/ChristopherRotter/homm3/pull/799) (branch `shared/e16-m5-game-wire`; the crates' edges, ADR-0018 points 5 and 7, the global trap); no common file, either order. **Next:** T1 (a scratch measurement needs no edge), T2 once #799 merged. **For the next milestone's review list:** `Query::CreatureCastTargets {}` (a message shape, provisional); the last `Snapshot` before `GameEnded` and the start's `Events` (v38).
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

### 2026-11-09 — the battle-wire inbox items, M5 part 1 refined

Baseline: `master` 2e5e08f (CI's fingerprint, the setup's light gates). No original data. The user (SRV-Q25): the game's types exhaustive, diff or events by measurement, CLT-STUB-3 and CLT-Q20 first. Built: the start's events at the game's start and the last snapshot before the end (v38, record v3); the client lane's inbox told. M5 part 1's task list in the epic and the QUEUE. PR: this branch's (see Current state).

### 2026-10-19 — the inbox after M6

The battle lane's inbox entries done (`E16-log.md`): the Windows fixes (the card test's `0.0.0.0` dial; `open_log` reads through its own handle; the "Access is denied" flake unproven: the user's next Windows run), `Query::CreatureCastTargets {}`, `may_retreat` as its viewer knows it (wire 34), a siege's frames, the example's ladder on `strict-fixed` (the user's choice, BAT-QUIRK-4). #281 open, the full local verify for CI.
