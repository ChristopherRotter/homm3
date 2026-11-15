# Server lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **M1–M4, M6, M5 part 1, 2a and 2b closed** (checkpoints SRV-Q12 to SRV-Q24, SRV-Q28, SRV-Q30 and SRV-Q34; M6 is discovery: ADR-0021). Their scopes, tasks and review lists are in `E16-archive.md`, done in full in `E16-log.md`. **2c's tasks T20 to T24 built** (#922, #927, #928, #931; T24 in the pull request from `claude/adoring-fermi-l0mvur`, `cross-lane`: watch it until it merges). **The checkpoint is open** (SRV-Q38): the reviewer's findings fixed or documented (protocol v68: the spectators' consent as the user's (b)), and the user looks at the fixes before M5 closes. **Next:** the user's answer; then 2c, M5 and E16's milestones close (the epic's "M5 part 2" to `E16-archive.md`, BACKLOG's status in a `shared/` change; "Later" stays). Nothing new is planned for the lane (SRV-Q38): the inbox's six notes wait for their triggers.
- **The TLS client** (ADR-0021 point 4, `peers::Tls`): rustls over graviola; no default features on a rustls crate (`ring`, `aws-lc` are not GPLv2-compatible; `cargo deny check` after a dependency change).
- **ADR-0018 point 5 as amended:** the mapping matches the engine's enums and the observation's structs exhaustively; a new battle variant or field fails `h3-protocol`'s build until mapped (global trap).
- **The 100 % gate** covers `h3-protocol` (logic); `h3-server` is an adapter outside it, tested end to end.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, 68 since 2026-11-15, T24: the spectators' consent, SRV-Q38; 67 the town lane's E09 T28b2: a town's garrison hero; 66 T22a: a whole game's spectators; 65 T20: a whole game void past the system's bound in a battle; 64 the town lane's E09 T28a: the garrison hero; 63 T19: 2b's reviewer's findings; 62 T18: the seats' bounds; 61 the adventure lane's E08 T115b: the bonus visitables; bumped on every wire change, a field's new meaning or a message's new order too), `RECORD_FORMAT_VERSION` (`h3-protocol`, 5 since 2026-11-12, T11: a whole game's record may hold the start's `GameEvents`), `JOURNAL_FORMAT_VERSION` (`h3-server`, a battle's, 8 since BAT-GEO-002's rear hex; another format's journal is void at start), `GAME_JOURNAL_FORMAT_VERSION` (`h3-server`, a whole game's, `<data>/whole/`, 1 since T9a; T14's and T16's lines and T17's start clock additive; T23b's interval a game's own, in its first line).
- The lane owns (`lanes.toml`) `h3-protocol`, `h3-server`, `benches/server/` and E16's files. **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
- **Ids:** questions `SRV-Q<n>` (next SRV-Q39), quirks `SRV-QUIRK-<n>`, stubs `SRV-STUB-<n>`. ADR drafts go to `docs/lanes/server/adr-drafts/` and get a number when the user accepts them. Branches: `server/<task>`.

## Autonomy contract (confirmed by the user, 2026-10-09)

- **Agent decides, marked provisional:** the inner structure of the server crates, names, test layout, error texts, log format; anything a later task can change without touching another crate. Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency, a message shape inside an accepted ADR, a configuration format. List them in the milestone's review list (epic file) and go on.
- **Stop the line:** seams and contracts (`h3-contracts`, the engine API the server calls), heavy dependencies (beyond the decided `tokio` features, `tokio-tungstenite`, `ed25519-dalek` and `argon2` (SRV-Q17): another runtime, an HTTP framework, a TLS stack, other crypto crates, a store), licence questions, changes to the security model (ADR-0008), anything touching another lane's paths. Write the question (`SRV-Q<n>`), stop that task and take the next independent one.

## Standing traps

Moved to `TRAPS.md`: the wire types (`no_std`, serde forms, 64-bit integers), coverage, tests' clients and rates, and the games' threads, locks, journals and outboxes, accounts and ladders, discovery and the TLS client, Windows. Read it before touching `h3-protocol` or `h3-server`.

## Recent sessions

### 2026-11-14/15 — M5 T24, the queue cleared (SRV-Q38, v68)

Baseline: `master` 6b3087d (#931; CI green, the setup's run on the same code); `master` merged in at 35c628b (#937: the replay counts 89). No original data. The VM restarted twice (files kept). The user: "clear out the server lane's queue"; the waiting notes as chosen (the if-needed six into the epic's "Later", the battle lane's and E21's kept). Built T24 (`E16-log.md`): the end-to-end spectator test, the security doc; the inbox's eleven review notes checked by an agent (the fold gate's three new parts, four comments, a question to the town lane) and archived; 2c's reviewer: two major findings, three minor, two nits, all fixed or documented (SRV-Q38: the consent as (b), v68). Next: the user's look, then M5's closure.

### 2026-11-14 — M5 T23 (SRV-Q37), T23b

Baseline: `master` 0872fc6 (#928; the setup's full local verify green on it; I edited the tree while it ran, so I moved my work to a worktree until it ended). No original data. The user: T23 measured then T23b. Built T23 (`E16-log.md`): `h3_server::measure` and `h3-server-journals` (a counting allocator), a year on the XL fixtures by the random agent and at a seat's bound, §15 with the case against state saves now. SRV-Q37 (the user, the recommendation): (a) to (c) now. Built T23b: the re-run's entries released, no copy of a game of connections' commands, `[server] whole_checkpoint_every` 250; the bound's year re-run 0.16 s (0.75), its session 2.1 MiB (114). The VM restarted once (files kept). Next: T24.

### 2026-11-14 — M5 T22b: spectators hosted (still v66)

Baseline: `master` 4be4d94 (#927; its CI still running at the start; the setup's full local run stopped for the scoped one, which a mid-run edit of mine broke at coverage, so CI's run on `master` is the baseline). No original data. Kept whole. Built T22b (`E16-log.md`): the creator's flag into the lobby and the session, `Lobby::update_map`'s flag, `GameSession::takes_spectators`, `registry::watch_map`, `whole::in_game` (a trap: `TRAPS.md`), `tests/watch.rs`, T20's void test with a spectator. The client lanes' T22a inbox lines updated. Next: T23.
