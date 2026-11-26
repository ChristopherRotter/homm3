# RMG lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`.

## Current state

- **E14** (the original's random map generator, ported to give the same map per seed): in progress since 2026-11-23. **Increments 1 and 2 closed** (2026-11-24, RMG-Q2; 2026-11-25, decision 10; `docs/epics/E14-archive.md`): the survey `docs/rules/surveys/E14-generator.md`, the ledger `docs/rules/quirks-rmg.md`, the oracle and the phase oracle `ORACLE.md`, **ADR-0029** (`docs/adr/0029-original-map-generator.md`); `h3-rmg`'s start (the settings, `resolve_random`, the templates kept and drawn, the positions, `generate`'s frame with its `Trace`, the steps RMG-STUB-1), the rmg part with the three tables and the executable's constants harvested, the catalog `docs/rules/catalog/rmg.toml` (RMG-SET-001 to -009, RMG-TPL-001 to -008). **Increment 3 refined** (steps 1 to 3 and the terrain painter): T3a to T3e (the epic's "Increment 3's tasks", `QUEUE.md`); **T3a done** (2026-11-25): the memory record, `phases.txt`'s per-phase digests, `zones.txt`, the oracle's game environment fixed (`H3_FIXED_ENV`), the corpus at 24.
- **Next:** **T3b** (the items not read: the triangulation, the fill's tail, the water test, the clip corners, code 1), then T3c to T3e (the steps; the four flags at T3c in one re-hash, decision 10). Every oracle run needs the clone at `/home/claude/homm3` (the fixed environment's size, `TRAPS.md`).
- **Open pull requests:** **#1185** (`rmg/e14-t3a`, T3a). Watch it until it merges.
- **Owns** (`lanes.toml`): `crates/h3-rmg/`, `crates/h3-pack-rmg/` (`h3-map`'s `src/generated*` joins at increment 7), the packs' `rmg.toml`, the RMG rules (`docs/rules/catalog/rmg.toml`) and quirks, E14's epic, surveys and experiments, this folder.
- **Versions:** `RMG_RULESET_HASH_VERSION` 2, `RMG_GENERATOR_VERSION` 1 (T2c; `h3-rmg`); the core's `RULESET_HASH_VERSION` 54 (T2c, shared). `CrtRand` has none (its sequence is the original's).
- **Ids:** questions `RMG-Q<n>` (next 3; none open), quirks `RMG-QUIRK-<n>` (next 25; two candidates open in the ledger), stubs `RMG-STUB-<n>` (next 2; RMG-STUB-1 open), decisions next 11. Registry: 54 of 128 positions taken (52 and 53 the rmg lane's first two `quirk.rmg` flags).
- **Waiting on other lanes:** the writer's seven gaps (survey §3.3.3) before increment 7, in the adventure (1 to 6) and town (7) lanes' inboxes; the town lane's answer on a neutral main town (RMG-QUIRK-23).
- **Working agreement:** a checkpoint with the user per increment (decision 6); tokens first; read the executable in the cloud sandbox (`docs/exe/README.md`, "Setup"); a big task cut at a seam with the user, worker sub-agents allowed for its parts (2026-11-23); a couple of small tasks or one bigger per session (2026-11-24); T2c kept whole (decision 9).

## Recent sessions

### 2026-11-25 — T3a the memory record, the fixed environment, the corpus at 24

A new chat; staged the client lane's set from the PC (58 files, SHA-256 checked), SoD's tables and Complete's two maps; baseline green. Asked for a cut; the user: none, sub-agents used efficiently. The first hooked run showed the sandbox's environment moving step 2's heap bytes (`m2-4p`'s variant), so the game's environment was fixed before the record; one agent measured what moves the bytes (the block's size). RMG-QUIRK-24 confirmed by the oracle. `l1-4p-b` and `xl2-4p-seed0` re-kept under the fixed environment.

### 2026-11-25 — increment 2's checkpoint, increment 3's cut (docs; the README's row `shared/`)

A new chat; staged the client lane's set from the PC (58 files, SHA-256 checked), SoD's tables and Complete's two maps. #1174 had merged; the VM restarted during the baseline's coverage (tests passed; CI on `master` stood in). Checked the exit (a) to (e) (the log's "Increment 2's checkpoint"; the `original_data` check run again) and proposed the cut; the user: increment 2 closed, T3a to T3e as proposed, the four flags at T3c, this session docs only.

### 2026-11-25 — T2c the settings, the template, the positions, `generate`'s frame (shared, `cross-lane`)

A new chat; staged the client lane's set from the PC first (SHA-256 checked). Asked for a cut (T2c1 the lane's paths, T2c2 the hashes); the user: one session, the registry widened in the same re-hash. The reading found RMG-QUIRK-24 (asked: fix-optional, flag); a fresh agent's blind re-read confirmed it and RMG-QUIRK-11. The re-hash was three core hashes in 131 files, done by substitution (the replays reproduce). The phase oracle ran the corpus (`--phases`) for the draws before step 1. The VM restarted once (the setup's baseline lost; CI's `master` stood in).
