# Server lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **M1–M4 and M6 closed** (checkpoints SRV-Q12 to SRV-Q24; M6 is discovery: ADR-0021). **M5 part 1** (SRV-Q25; the epic's "M5 part 1", the QUEUE's tasks): T0 (#799), T1 (#803; **events only**, the design note §14), T2 (#805, the observation's wire) and T3 (#814: the game's commands, rejections and events, the fold `GameObservation::fold` and its gate over the 55 golden replays, `tests/game_fold.rs`) done 2026-11-09; **T3d** done 2026-11-10: the town lane's told views (E09 T18b2a, #815) mapped and folded, which `master` 700bf95 lacked to build (#814 and #815 merged minutes apart, each green alone); `SRV-STUB-12` and `-13` lifted, the gate's town parts exempt only a reveal and `TWN-STUB-13`; 36 parts, 17 exact, the rest named by `SRV-STUB-1` to `-11` (the adventure lane's inbox). **Open pull request:** [#816](https://github.com/ChristopherRotter/homm3/pull/816), T3d's, from `claude/eloquent-thompson-40gdd4`; it repairs `master`'s build, so it merges first. **T4** done 2026-11-10: `wire::game::GameManifest` (every part's hash, the map's `MapIdentity`, the battle's and the game's key tables), `map::game::manifest(rules, map)`; on its own branch `server/e16-m5-t4` (the user, 2026-11-10); its pull request opens once #816 merged (merge `master` in first). **Next:** T5 (the game session in-process), in a fresh session (the user). The seam proposed to the user, for that session to confirm at its start: T5a the game's messages (a protocol step) and a sans-I/O game session (seats, the trusted actor, the legal commands to whoever must act, the system actor's commands from its caller, each viewer's manifest, snapshot and events); T5b the journal, the restore and each viewer's record (record format v4); T5c each viewer's newly seen tiles once the adventure lane's SRV-Q26 query lands (until then `SRV-STUB-1`). **The review list so far:** the epic's "M5 part 1".
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

### 2026-11-10 — M5 T3d (`master`'s build repaired, the town views folded) and T4 (the game's manifest)

Baseline: `master` 4701369, red: `h3-protocol` did not build since #815 (E09 T18b2a's `TownChanged`) met #814's mapping; CI's run on 700bf95 was in progress at the start. No original data. Built T3d (`E16-log.md`): the three town events on the wire, mapped, inverted, folded; the gate's numbers (garrisons and owner facts 8 viewer-commands each, the 3 without a reveal Visions' reach, checked by a printout); its pull request #816. CI's first run on #816 failed in `h3-map`'s proptest (a rare counterexample, not ours: the town lane's inbox); its job re-run. Then T4 (`E16-log.md`): the game's manifest, pushed to `server/e16-m5-t4`. The coverage tools installed by hand (the setup script's baseline would have run the red `master`). No message, no version step. Next: T5 in a fresh session, its seam above.

### 2026-11-09 — M5 T3c2: the fold's stubs and gate; T3's pull request

Baseline: the branch's T3c1 on `master` d1b90fb (CI green); the sandbox's full engine verify of that code ran at the start. No original data. Built T3c2 (`E16-log.md`): `SRV-STUB-1` to `-13` with their markers in the fold, the adventure and town lanes' inbox notes, the harness as the gate (the hero details and the decision split; the reveal-only rule). `master` 737d007 (#811) merged in, the numbers unmoved. No message, no version step. The scoped verify passed (coverage included). Then T3's one pull request, #814.

### 2026-11-09 — M5 T3c1: the game's fold and its harness

Baseline: the branch's T3a and T3b on `master` 1e8568d, the sandbox's full verify passed (`verify-once`; CI's `master` run is red by `h3-map`'s proptest, which passed here). No original data. Built the fold (`GameObservation::fold`) and the harness over the golden replays (`E16-log.md`, its table): 14 of 31 parts equal `observe` after every command for every viewer; the rest are SRV-Q26's (a) to (d) and SRV-Q27's decision views and derived values. One fold error found and fixed (a Leave Guards dialog's guards on a mine without any). `master` brought the town lane's build (#809) mid-session: mapped here. No message, no version step. Next: T3c2.
