# Adventure lane — HANDOFF

The adventure lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E08 increment 1** (epic `docs/epics/E08-adventure-map.md`, decisions 1–12; done tasks in `E08-log.md`). T1a, T1b, T2 and T3a done; next **T3b** (templates, object bodies, events; `QUEUE.md`), then the user's checkpoint and the refinement of T4–T10 into queue entries. T3 builds only the reader: the import moved to T5 (`AdventureSetup`) and T6 (`GameSetup`) (user, 2026-10-10).
- **Map reader** (`h3-map`, T3a): `read`/`parse` into `model::RawMap` up to the terrain (ADV-STUB-1), `Limits`, `ReadError`; every shipped map reads (`verify --with-original-data` runs `crates/h3-map/examples/original_data.rs`; the maps are in `original-data/install/Maps`, staged 50 per call).
- **ADRs accepted** (2026-10-10): ADR-0015 (the four crates `h3-map`, `h3-pack-adventure`, `h3-adventure`, `h3-game`; ruleset parts; the three-stage import; `PlayerId`/`HeroId` in `h3-contracts`; a replay names its map by content hash, path and `MAP_IMPORT_VERSION`), ADR-0016 (persistent grids, object kinds, fog), ADR-0017 (turn groups, per-player streams, pending decisions, `AdventureEnv` in adventure-owned sub-paths of `h3-sim`/`h3-cli`).
- **T2** (`shared/adventure-skeleton`, labelled `cross-lane` with the user's consent): the four skeletons, `arch.toml` (including the edges T5–T9 need), `lanes.toml` (the new crates, the sub-paths, `docs/formats/h3m.md` moved from battle, the survey index common), ids, `StreamId::GAME`/`SETUP`/`player(i)`, the determinism step for the lane's own replays, the versions rows, BACKLOG.
- **Waiting for the user:** three editor maps for T3b's fixtures (one per version: a customised hero, many object kinds, a special condition, a rumor, a timed event, one underground); A-1, the fog fringe: four answers after looking at a map in SoD (survey `E08-adventure-core.md` §4), needed by T8; A-2/A-3 are classified after T7's code reading (`ADV-Q3`). Open: `ADV-Q3`–`ADV-Q5` (code readings in T7 and T8).
- **Fuzzing is stable-only for now** (user, 2026-10-10): property tests on arbitrary bytes; `cargo-fuzz` later.
- **Not before E04b closes:** the content split and the writer's move into `h3-map` (BACKLOG R01). Until then adventure crates use only the core items of `h3-content`.
- **Ownership:** the four crates, `src/adventure/` in `h3-sim` and `h3-cli`, `crates/h3-cli/tests/adventure_replays.rs`, `fixtures/replays/adventure/`, `fixtures/maps/`, each pack's `adventure.toml`, `docs/formats/h3m.md`, `docs/rules/catalog/{adventure,game,town}.toml`. `h3-heroes` moves to this lane when hero progression starts (increment 3).
- **Ids:** questions `ADV-Q<n>`, quirks `ADV-QUIRK-<n>` (the fog fringe is `ADV-QUIRK-1`), stubs `ADV-STUB-<n>`, rules `ADV-*` and `GAM-*`. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format (`docs/08-work-process.md`).
- **Working agreement:** increments of a couple of tasks with a user checkpoint, as in the battle lane; the user confirms seams and ADRs; token efficiency beats speed (AGENTS.md).

## Standing traps

- Cheap forks are a hard requirement (decision 6): no per-fork deep copy of map-sized data; pointer identity never in logic or the hash.
- RNG seeds and stream positions are server-only state (decision 9): never in an observation, never seeded from client data.
- A shared change the lane needs (Cargo members, `arch.toml`, `lanes.toml`, contracts, kernel, xtask) goes in a `shared/` pull request; one that must land with lane paths (a new crate) needs the user's consent and the label `cross-lane`. A `shared/` or cross-lane branch runs the full verify (its scoped `check-lane` fails by design).

## Recent sessions

### 2026-10-10 — T3a: map reader up to the terrain

The user agreed to split T3 (reader only; import to T5/T6) and had the installation's `Maps` copied into `original-data/install/Maps`. A small `shared/h3m-reader-gate` PR first: `h3-map` may use `miniz_oxide` (dev: `h3-formats`), and `verify --with-original-data` runs every crate's `examples/original_data.rs`. T3a: bounded gzip with CRC, RoE/AB/SoD to the end of the terrain, raw codes and bytes kept, every count checked against its limit and the bytes left; a test-only encoder drives the round-trip, prefix and arbitrary-bytes properties. All 164 shipped maps read on the first run. Found: PR #44 (server) numbers its ADR 0015, which master already has (left to the server lane, user). Detail: `E08-log.md`.

### 2026-10-10 — T1a checkpoint and T2 shared skeleton

The user accepted the three drafts after a one-screen summary per draft (ADR-0015–0017), answered ADV-Q1 (a) and ADV-Q2 (a), deferred A-2/A-3 to T7's reading, consented to one `cross-lane` pull request for T2 and to moving `docs/formats/h3m.md`, and chose stable-only fuzzing. T2 built as planned, with additions: `arch.toml` already allows the env's and the CLI's edges (T9 needs no shared change); the replay test `adventure_replays.rs` is adventure-owned and the determinism step picks it up. Found: the game's rule prefix is `GAM-`; T3's import needs setup types that T5/T6 own (decide the order at T3's start); `check-exe-map` was red on `master` since #33 (the survey's four H3API addresses had no map entries, and a docs-only PR runs no CI): added as guesses. Trap: a docs PR that cites an address still needs `cargo xtask check-exe-map` locally. Detail: `E08-log.md`.

### 2026-10-09 — T1b: survey (movement, fog, the day, simultaneous turns)

Sources: VCMI 1.7.5, the wiki's Movement and Scouting pages (the user approved the wiki for this session), the executable's tables already mapped, the HD mod's page. No code reading: the disputes name the function to read (CR-1 step cost, CR-2 day's points, CR-3 reveal), queued as the first step of T7, T6/T7 and T8 (`ADV-Q3`–`ADV-Q5`). Findings that touch the drafts: a path may use explored tiles only (VCMI; question 4 of the user's map check), so the action mask cannot leak; every increment-1 rule runs at the start of the day; increment 1 has no owned objects, so a game opens with only the heroes' surroundings explored (a recorded simplification). VCMI's contact rule for simultaneous turns (reachability at the start of the day) is the recommended model for later.
