# RMG lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`.

## Current state

- **E14** (the original's random map generator, ported to give the same map per seed): in progress since 2026-11-23. **Increment 1 closed** (2026-11-24, RMG-Q2; `docs/epics/E14-archive.md`): the survey `docs/rules/surveys/E14-generator.md`, the ledger `docs/rules/quirks-rmg.md`, the oracle and the phase oracle `ORACLE.md`, **ADR-0029** (`docs/adr/0029-original-map-generator.md`). **Increments 2 to 7 cut** in the run's order (the epic's table, decision 8). **Increment 2's tasks done:** T2a (the crates, `CrtRand`, the catalog context), T2b (the rmg part, the three tables harvested, RMG-QUIRK-22's flag), **T2c** (2026-11-25): the settings, `resolve_random`, the templates kept and drawn, the positions, `generate`'s frame with its `Trace` (the steps RMG-STUB-1), the executable's constants (`packs/sod/rmg.toml`'s `[generator]`), the flags of RMG-QUIRK-11 and -24, the registry at 128 flags, the catalog `docs/rules/catalog/rmg.toml` (RMG-SET-001 to -009, RMG-TPL-001 to -008).
- **Next:** **increment 2's checkpoint** with the user (exit (f); the epic's exit criteria (a) to (e) and the increment's own: the 21 cases' templates and draws before step 1 equal, the harvested tables equal the text tables), then increment 3's refinement (steps 1 to 3; `QUEUE.md`).
- **Open pull requests:** **#PR** (`shared/e14-t2c`, T2c, `cross-lane`). Watch it until it merges.
- **Owns** (`lanes.toml`): `crates/h3-rmg/`, `crates/h3-pack-rmg/` (`h3-map`'s `src/generated*` joins at increment 7), the packs' `rmg.toml`, the RMG rules (`docs/rules/catalog/rmg.toml`) and quirks, E14's epic, surveys and experiments, this folder.
- **Versions:** `RMG_RULESET_HASH_VERSION` 2, `RMG_GENERATOR_VERSION` 1 (T2c; `h3-rmg`); the core's `RULESET_HASH_VERSION` 54 (T2c, shared). `CrtRand` has none (its sequence is the original's).
- **Ids:** questions `RMG-Q<n>` (next 3; none open), quirks `RMG-QUIRK-<n>` (next 25; two candidates open in the ledger), stubs `RMG-STUB-<n>` (next 2; RMG-STUB-1 open), decisions next 10. Registry: 54 of 128 positions taken (52 and 53 the rmg lane's first two `quirk.rmg` flags).
- **Waiting on other lanes:** the writer's seven gaps (survey §3.3.3) before increment 7, in the adventure (1 to 6) and town (7) lanes' inboxes; the town lane's answer on a neutral main town (RMG-QUIRK-23).
- **Working agreement:** a checkpoint with the user per increment (decision 6); tokens first; read the executable in the cloud sandbox (`docs/exe/README.md`, "Setup"); a big task cut at a seam with the user, worker sub-agents allowed for its parts (2026-11-23); a couple of small tasks or one bigger per session (2026-11-24); T2c kept whole (decision 9).

## Recent sessions

### 2026-11-25 — T2c the settings, the template, the positions, `generate`'s frame (shared, `cross-lane`)

A new chat; staged the client lane's set from the PC first (SHA-256 checked). Asked for a cut (T2c1 the lane's paths, T2c2 the hashes); the user: one session, the registry widened in the same re-hash. The reading found RMG-QUIRK-24 (asked: fix-optional, flag); a fresh agent's blind re-read confirmed it and RMG-QUIRK-11. The re-hash was three core hashes in 131 files, done by substitution (the replays reproduce). The phase oracle ran the corpus (`--phases`) for the draws before step 1. The VM restarted once (the setup's baseline lost; CI's `master` stood in).

### 2026-11-25 — T2b the three tables, the rmg part, 8XM8's flag (shared, `cross-lane`)

A new chat; staged from the PC both LODs, `Heroes3.exe`, `extracted.tar` and the six tables before its link closed. Asked for a cut (the part, the harvest, the flag); the user: no cut, a worker for a part. Built the part, the parser, the harvest and the check; the flag registered and set in the profiles, the golden replays re-hashed by one worker. #1166's CI: cancelled once at the 55-minute timeout (a slow runner), then failed on `h3-map`'s view property test (a random case, failing on `master` too: the seed in the adventure lane's inbox), green the third time. The user then asked for the CI rework (60 minutes per job, never raised): phase 1, #1170, built by one worker.

### 2026-11-24 — T2a the crates, `CrtRand`, ADR-0029 (shared, `cross-lane`)

A new chat; staged from the PC only `Heroes3.exe` (its `master` behind GitHub's; the clone GitHub's). Kept whole. The two crates empty (`h3-game → h3-rmg` forbidden), `CrtRand` with the code reading's vectors (`crt_srand` 0x617E4F, `crt_rand` 0x617E5C, a thread's state 1 at 0x61D2FB): the phase oracle's draw logs were never kept, so the oracle checks the stream from T2c on. The domain row, the catalog context `RMG`, ADR-0029 numbered with ADR-0003's history line; the adventure lane told `TileRand` and `OriginalDraws` may wrap `CrtRand`.
