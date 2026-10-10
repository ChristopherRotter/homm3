# Adventure lane — HANDOFF

The adventure lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E08 increment 1** (epic `docs/epics/E08-adventure-map.md`, decisions 1–12; done tasks in `E08-log.md`). T1a, T1b, T2, T3 and T4 done; T4–T10 refined into queue entries at the T3 checkpoint (T5 and T6 split in two). The user takes one or two tasks per round with a checkpoint after each (user, 2026-10-10); next **the user's checkpoint after T4**, then the round they pick (the queue's next is T5a).
- **Map reader** (`h3-map`): `read`/`parse` into `model::RawMap`; every shipped map reads (`verify --with-original-data` runs `crates/h3-map/examples/original_data.rs`; the maps are in `original-data/install/Maps`, staged 50 per call). The import is T5a's (static layer) and T6a's (players, heroes).
- **Adventure ruleset part** (T4): `h3_adventure::AdventureRuleset` (terrain passages and costs, roads, rivers, native cost, land table, base scouting radius), compiled from one `AdventurePack` per core pack; `h3_pack_adventure::compile` takes `(pack.toml, adventure.toml)` pairs, `ORIGINAL_PACKS` is RoE to SoD. Base terrains, roads and rivers are in the `.h3m` code order. The rules it feeds (ADV-MOV-001/003/004/007, ADV-FOG-001) are `specified` until T7/T8.
- **ADRs accepted** (2026-10-10): ADR-0015 (crates, ruleset parts, three-stage import, `PlayerId`/`HeroId`, a replay names its map by hash, path and `MAP_IMPORT_VERSION`), ADR-0016 (persistent grids, object kinds, fog), ADR-0017 (turn groups, per-player streams, pending decisions, `AdventureEnv` in adventure-owned sub-paths of `h3-sim`/`h3-cli`). `arch.toml` already allows the edges T5–T9 need.
- **Waiting for the user:** three editor maps for the fixtures (one per version: a customised hero, many object kinds, a special condition, a rumor, a timed event, one underground); A-1, the fog fringe: four answers after looking at a map in SoD (survey §4), needed by T8; A-2/A-3 are classified after T7's code reading (`ADV-Q3`); deleting the merged branches `adventure/e08-t3a` and `-t3b` on GitHub (this session's access cannot). Open: `ADV-Q3`–`ADV-Q5` (ADV-Q4's D-4 is settled; its call sites go to T6b).
- **Fuzzing is stable-only for now** (user, 2026-10-10): property tests on arbitrary bytes; `cargo-fuzz` later.
- **Not before E04b closes:** the content split and the writer's move into `h3-map` (BACKLOG R01). Until then adventure crates use only the core items of `h3-content` (a domain test's core ruleset still needs battle parameters).
- **Ownership:** the four crates, `src/adventure/` in `h3-sim` and `h3-cli`, `crates/h3-cli/tests/adventure_replays.rs`, `fixtures/replays/adventure/`, `fixtures/maps/`, each pack's `adventure.toml`, `docs/formats/h3m.md`, `docs/rules/catalog/{adventure,game,town}.toml`. `h3-heroes` moves to this lane when hero progression starts (increment 3).
- **Ids:** questions `ADV-Q<n>`, quirks `ADV-QUIRK-<n>` (the fog fringe is `ADV-QUIRK-1`), stubs `ADV-STUB-<n>`, rules `ADV-*` and `GAM-*`. Versions owned: game state hash, adventure state and ruleset-part hashes (the part's is v1 since T4), map import, adventure replay format (`docs/08-work-process.md`).
- **Working agreement:** increments of a couple of tasks with a user checkpoint, as in the battle lane; the user confirms seams and ADRs; token efficiency beats speed (AGENTS.md).

## Standing traps

- Cheap forks are a hard requirement (decision 6): no per-fork deep copy of map-sized data; pointer identity never in logic or the hash.
- RNG seeds and stream positions are server-only state (decision 9): never in an observation, never seeded from client data.
- A shared change the lane needs (Cargo members, `arch.toml`, `lanes.toml`, contracts, kernel, xtask) goes in a `shared/` pull request; one that must land with lane paths (a new crate) needs the user's consent and the label `cross-lane`. A `shared/` or cross-lane branch runs the full verify (its scoped `check-lane` fails by design).

## Recent sessions

### 2026-10-10 — T3 checkpoint and T4: the adventure content

At the checkpoint the user chose T4 alone for this round; T4–T10 went into queue entries first (T5a/T5b, T6a/T6b). T4: the ruleset part compiled on the core ruleset from one adventure pack per core pack (the core's order and closures), the strict `adventure.toml` format, base values with sources (wiki and VCMI agree on the costs; VCMI's config files fetched raw from GitHub). D-4 settled by original data: the original reads the land table from `movement.txt` (0x4D7240 → 0x698A98; 0x4E4990 takes the slowest of the seven slots from 20), the wiki's row; increment-3 findings (specialty +1 speed, Logistics, sea by Navigation) in the log and the map. Cross-lane: one paragraph in `packs/README.md`. Trap: `git push --delete` and the REST ref delete are refused through this session's proxy. Detail: `E08-log.md`.

### 2026-10-10 — T3: the map reader (T3a, T3b)

The user agreed to split T3 (reader only; import to T5/T6) and had the installation's `Maps` copied into `original-data/install/Maps`. A small `shared/h3m-reader-gate` PR first (#48): `h3-map` may use `miniz_oxide` (dev: `h3-formats`), `verify --with-original-data` runs every crate's `examples/original_data.rs`, and the coverage gate passes skeleton logic crates. T3a: bounded gzip with CRC, RoE/AB/SoD to the terrain, raw codes and bytes kept, every count checked against its limit and the bytes left; a test-only encoder drives the round-trip, prefix and arbitrary-bytes properties. T3b: templates, a body per class, timed events; the shipped maps showed 124 bytes of zero padding and the town's banned-spell set in RoE. All 164 maps read to the end. Trap: read VCMI's readers unfiltered (a grep for `read` lines hid a bare block). Found: PR #44 (server) numbers its ADR 0015, which master has (left to the server lane, user). Detail: `E08-log.md`.

### 2026-10-10 — T1a checkpoint and T2 shared skeleton

The user accepted the three drafts after a one-screen summary per draft (ADR-0015–0017), answered ADV-Q1 (a) and ADV-Q2 (a), deferred A-2/A-3 to T7's reading, consented to one `cross-lane` pull request for T2 and to moving `docs/formats/h3m.md`, and chose stable-only fuzzing. T2 built as planned, with additions: `arch.toml` already allows the env's and the CLI's edges (T9 needs no shared change); the replay test `adventure_replays.rs` is adventure-owned and the determinism step picks it up. Found: the game's rule prefix is `GAM-`; T3's import needs setup types that T5/T6 own (decide the order at T3's start); `check-exe-map` was red on `master` since #33 (the survey's four H3API addresses had no map entries, and a docs-only PR runs no CI): added as guesses. Trap: a docs PR that cites an address still needs `cargo xtask check-exe-map` locally. Detail: `E08-log.md`.
