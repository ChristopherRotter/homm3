# Server lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **M1–M4, M6, M5 part 1, 2a and 2b closed** (checkpoints SRV-Q12 to SRV-Q24, SRV-Q28, SRV-Q30 and SRV-Q34; M6 is discovery: ADR-0021). Their scopes, tasks and review lists are in `E16-archive.md`, done in full in `E16-log.md`. **2c refined 2026-11-14** (SRV-Q35: a battle both of whose sides the system plays voids its game past a bound; the spectator sees everything, by the host's choice): tasks T20 to T24 in the epic's "M5 part 2", its review list there. T20 done (v65, #922 merged). **T21a done** (SRV-Q36: the spectator sees each player's view, the creator's flag before the start, closed by default, never ranked, live; ADR-0017 point 2, ADR-0018 point 7, `docs/06-security.md` amended; T21b dropped, T22 cut in two). **T22a done** (v66: the session's spectators; `h3-server` takes none yet). Both in one pull request from `claude/festive-albattani-mdvj1a`, labelled `cross-lane` (the user: one branch); watch it until it merges. **Next: T22b**, the spectators hosted in `h3-server` (its scope in the epic).
- **The TLS client** (ADR-0021 point 4, `peers::Tls`): rustls over graviola; no default features on a rustls crate (`ring`, `aws-lc` are not GPLv2-compatible; `cargo deny check` after a dependency change).
- **ADR-0018 point 5 as amended:** the mapping matches the engine's enums and the observation's structs exhaustively; a new battle variant or field fails `h3-protocol`'s build until mapped (global trap).
- **The 100 % gate** covers `h3-protocol` (logic); `h3-server` is an adapter outside it, tested end to end.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, 66 since 2026-11-14, T22a: a whole game's spectators; 65 T20: a whole game void past the system's bound in a battle; 64 the town lane's E09 T28a: the garrison hero; 63 T19: 2b's reviewer's findings; 62 T18: the seats' bounds; 61 the adventure lane's E08 T115b: the bonus visitables; bumped on every wire change, a field's new meaning or a message's new order too), `RECORD_FORMAT_VERSION` (`h3-protocol`, 5 since 2026-11-12, T11: a whole game's record may hold the start's `GameEvents`), `JOURNAL_FORMAT_VERSION` (`h3-server`, a battle's, 8 since BAT-GEO-002's rear hex; another format's journal is void at start), `GAME_JOURNAL_FORMAT_VERSION` (`h3-server`, a whole game's, `<data>/whole/`, 1 since T9a; T14's and T16's lines and T17's start clock additive).
- The lane owns (`lanes.toml`) `h3-protocol`, `h3-server`, `benches/server/` and E16's files. **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
- **Ids:** questions `SRV-Q<n>` (next SRV-Q37), quirks `SRV-QUIRK-<n>`, stubs `SRV-STUB-<n>`. ADR drafts go to `docs/lanes/server/adr-drafts/` and get a number when the user accepts them. Branches: `server/<task>`.

## Autonomy contract (confirmed by the user, 2026-10-09)

- **Agent decides, marked provisional:** the inner structure of the server crates, names, test layout, error texts, log format; anything a later task can change without touching another crate. Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency, a message shape inside an accepted ADR, a configuration format. List them in the milestone's review list (epic file) and go on.
- **Stop the line:** seams and contracts (`h3-contracts`, the engine API the server calls), heavy dependencies (beyond the decided `tokio` features, `tokio-tungstenite`, `ed25519-dalek` and `argon2` (SRV-Q17): another runtime, an HTTP framework, a TLS stack, other crypto crates, a store), licence questions, changes to the security model (ADR-0008), anything touching another lane's paths. Write the question (`SRV-Q<n>`), stop that task and take the next independent one.

## Standing traps

Moved to `TRAPS.md`: the wire types (`no_std`, serde forms, 64-bit integers), coverage, tests' clients and rates, and the games' threads, locks, journals and outboxes, accounts and ladders, discovery and the TLS client, Windows. Read it before touching `h3-protocol` or `h3-server`.

## Recent sessions

### 2026-11-14 — M5 T21a (SRV-Q36), T22a (v66)

Baseline: `master` 4dfc666 (#922, T20; CI green on it). No original data. The user (SRV-Q36): the spectator sees each player's view (not the merged full view recommended), the creator's flag before the start, never ranked, live; T21b dropped, T22 cut into T22a and T22b, T22a in this session; the ADR and security amendments in the lane's pull request, `cross-lane` (one branch). Built T22a (`E16-log.md`): `GameWatch`, `GameWatched` spliced in `route`, `Spectators`, `GameEntry::Watchable`, `SetSpectators`; `h3-server` refuses watching until T22b. The client lanes' inboxes told (v66). Next: T22b.

### 2026-11-14 — M5 2c refined (SRV-Q35), T20 (v65)

Baseline: `master` 161b565 (#919, T19; CI green on it; the VM restarted during the setup's baseline). No original data. The user (SRV-Q35): a battle both of whose sides the system plays voids its game past a bound on the system's commands in one battle; the spectator sees everything, by the host's choice (ADR-0017 point 2 amended in T21a); refine, then T20. 2c's tasks T20 to T24 in the epic. Built T20 (`E16-log.md`): `CommandBounds::system_per_battle` (`[limits] system_commands_per_battle`, 10 000; measured most 505), `GameSession::void`, `drive` at most 64 steps per call, `whole.rs` voiding as a battle's `CommandLimit`. `docs/06-security.md` left to T24. The client lanes' inboxes told. Next: T21a.

### 2026-11-14 — M5 T19: 2b's closure (SRV-Q34, v63)

Baseline: `master` daf0051 (#916, T18). No original data. Kept whole (the user). The end-to-end test (`E16-log.md`): `towns-s3435-d21`'s opening on a timed hosted game, its turn run out, the duel under the chess clocks, two held level-ups, two restarts, to the golden replay's hash. `docs/06-security.md` `cross-lane`. The reviewer: no major, four minor, two nits; the user (SRV-Q34): 1, 2, 4 and the nits fixed (v63), finding 3 first in 2c, the review list as built, 2c refined in a new session. The client lanes' inboxes told. Next: 2c's refinement.
