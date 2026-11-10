# Server lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **M1–M4 and M6 closed** (checkpoints SRV-Q12 to SRV-Q24; M6 is discovery: ADR-0021). **M5 part 1** (SRV-Q25; the epic's "M5 part 1", the QUEUE's tasks): T0 to T3d merged (#799, #803, #805, #814, #816): the game's observation, commands, rejections and events on the wire and the fold `GameObservation::fold` with its gate over the 55 golden replays (`tests/game_fold.rs`: 36 parts, 17 exact, the rest named by `SRV-STUB-1` to `-11`, the adventure lane's inbox, and `TWN-STUB-13`). **T4** (the game's manifest, `GameManifest`) and **T5a** (the game's messages, protocol v39, and `session::game::GameSession`, sans I/O; `SRV-STUB-14`, the adventure lane's inbox) done 2026-11-10 on `claude/zealous-ramanujan-qsbc2o`, one pull request for both (the user; `server/e16-m5-t4` is stale, its commits are in it). **Open pull request:** T4's and T5a's, from `claude/zealous-ramanujan-qsbc2o` (opened 2026-11-10; find it by that branch). **Next:** T5b (the journal, the restore and each viewer's record: record format v4), T5c once the adventure lane's SRV-Q26 query lands, then T6 and T7. **The review list so far:** the epic's "M5 part 1".
- **The TLS client** (ADR-0021 point 4, `peers::Tls`): rustls over graviola; no default features on a rustls crate (`ring`, `aws-lc` are not GPLv2-compatible; `cargo deny check` after a dependency change).
- **ADR-0018 point 5 as amended:** the mapping matches the engine's enums and the observation's structs exhaustively; a new battle variant or field fails `h3-protocol`'s build until mapped (global trap).
- **The 100 % gate** covers `h3-protocol` (logic); `h3-server` is an adapter outside it, tested end to end.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, 39 since 2026-11-10, E16 M5 T5a's game messages; bumped on every wire change, a field's new meaning or a message's new order too), `RECORD_FORMAT_VERSION` (`h3-protocol`, 3 since 2026-11-09: the start's `Events`, the last `Snapshot`), `JOURNAL_FORMAT_VERSION` (`h3-server`, 8 since BAT-GEO-002's rear hex; another format's journal is void at start).
- The lane owns (`lanes.toml`) `h3-protocol`, `h3-server`, `benches/server/` and E16's files. **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
- **Ids:** questions `SRV-Q<n>`, quirks `SRV-QUIRK-<n>`, stubs `SRV-STUB-<n>`. ADR drafts go to `docs/lanes/server/adr-drafts/` and get a number when the user accepts them. Branches: `server/<task>`.

## Autonomy contract (confirmed by the user, 2026-10-09)

- **Agent decides, marked provisional:** the inner structure of the server crates, names, test layout, error texts, log format; anything a later task can change without touching another crate. Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency, a message shape inside an accepted ADR, a configuration format. List them in the milestone's review list (epic file) and go on.
- **Stop the line:** seams and contracts (`h3-contracts`, the engine API the server calls), heavy dependencies (beyond the decided `tokio` features, `tokio-tungstenite`, `ed25519-dalek` and `argon2` (SRV-Q17): another runtime, an HTTP framework, a TLS stack, other crypto crates, a store), licence questions, changes to the security model (ADR-0008), anything touching another lane's paths. Write the question (`SRV-Q<n>`), stop that task and take the next independent one.

## Standing traps

Moved to `TRAPS.md`: the wire types (`no_std`, serde forms, 64-bit integers), coverage, tests' clients and rates, and the games' threads, locks, journals and outboxes, accounts and ladders, discovery and the TLS client, Windows. Read it before touching `h3-protocol` or `h3-server`.

## Recent sessions

### 2026-11-10 — M5 T5a: the game's messages and the in-process session

Baseline: T4's branch with `master` b1d2e94 merged in (CI green on b1d2e94; the setup's baseline ran on b1d2e94's code, light gates only; the scoped verify passed on the merge). No original data. The user confirmed T5's split (T5a, T5b, T5c) and one pull request for T4 and T5a. Built T5a (`E16-log.md`): five `Game*` server messages (v39), `GameSession` and its test over the golden replays beside the engine; the fold's golden loader shared. `SRV-STUB-14` (a battle's last snapshot) with the adventure lane's inbox note. Next: T5b.

### 2026-11-10 — M5 T3d (`master`'s build repaired, the town views folded) and T4 (the game's manifest)

Baseline: `master` 4701369, red: `h3-protocol` did not build since #815 (E09 T18b2a's `TownChanged`) met #814's mapping; CI's run on 700bf95 was in progress at the start. No original data. Built T3d (`E16-log.md`): the three town events on the wire, mapped, inverted, folded; the gate's numbers (garrisons and owner facts 8 viewer-commands each, the 3 without a reveal Visions' reach, checked by a printout); its pull request #816. CI's first run on #816 failed in `h3-map`'s proptest (a rare counterexample, not ours: the town lane's inbox); its job re-run. Then T4 (`E16-log.md`): the game's manifest, pushed to `server/e16-m5-t4`. The coverage tools installed by hand (the setup script's baseline would have run the red `master`). No message, no version step. Next: T5 in a fresh session, its seam above.

### 2026-11-09 — M5 T3c2: the fold's stubs and gate; T3's pull request

Baseline: the branch's T3c1 on `master` d1b90fb (CI green); the sandbox's full engine verify of that code ran at the start. No original data. Built T3c2 (`E16-log.md`): `SRV-STUB-1` to `-13` with their markers in the fold, the adventure and town lanes' inbox notes, the harness as the gate (the hero details and the decision split; the reveal-only rule). `master` 737d007 (#811) merged in, the numbers unmoved. No message, no version step. The scoped verify passed (coverage included). Then T3's one pull request, #814.
