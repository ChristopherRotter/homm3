# Adventure lane — HANDOFF

The adventure lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E08 increment 1** (epic `docs/epics/E08-adventure-map.md`, decisions 1–12; done tasks in `E08-log.md`). T1a, T1b and T2 done; next **T3, the map reader** (`QUEUE.md`), then the refinement of T4–T10 into queue entries.
- **ADRs accepted** (2026-10-10): ADR-0015 (the four crates `h3-map`, `h3-pack-adventure`, `h3-adventure`, `h3-game`; ruleset parts; the three-stage import; `PlayerId`/`HeroId` in `h3-contracts`; a replay names its map by content hash, path and `MAP_IMPORT_VERSION`), ADR-0016 (persistent grids, object kinds, fog), ADR-0017 (turn groups, per-player streams, pending decisions, `AdventureEnv` in adventure-owned sub-paths of `h3-sim`/`h3-cli`).
- **T2** (`shared/adventure-skeleton`, labelled `cross-lane` with the user's consent): the four skeletons, `arch.toml` (including the edges T5–T9 need), `lanes.toml` (the new crates, the sub-paths, `docs/formats/h3m.md` moved from battle, the survey index common), ids, `StreamId::GAME`/`SETUP`/`player(i)`, the determinism step for the lane's own replays, the versions rows, BACKLOG.
- **Waiting for the user:** A-1, the fog fringe: four answers after looking at a map in SoD (survey `E08-adventure-core.md` §4), needed by T8; A-2/A-3 are classified after T7's code reading (`ADV-Q3`). Open: `ADV-Q3`–`ADV-Q5` (code readings in T7 and T8).
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

### 2026-10-10 — T1a checkpoint and T2 shared skeleton

The user accepted the three drafts after a one-screen summary per draft (ADR-0015–0017), answered ADV-Q1 (a) and ADV-Q2 (a), deferred A-2/A-3 to T7's reading, consented to one `cross-lane` pull request for T2 and to moving `docs/formats/h3m.md`, and chose stable-only fuzzing. T2 built as planned, with additions: `arch.toml` already allows the env's and the CLI's edges (T9 needs no shared change); the replay test `adventure_replays.rs` is adventure-owned and the determinism step picks it up. Found: the game's rule prefix is `GAM-`; T3's import needs setup types that T5/T6 own (decide the order at T3's start); `check-exe-map` was red on `master` since #33 (the survey's four H3API addresses had no map entries, and a docs-only PR runs no CI): added as guesses. Trap: a docs PR that cites an address still needs `cargo xtask check-exe-map` locally. Detail: `E08-log.md`.

### 2026-10-09 — T1b: survey (movement, fog, the day, simultaneous turns)

Sources: VCMI 1.7.5, the wiki's Movement and Scouting pages (the user approved the wiki for this session), the executable's tables already mapped, the HD mod's page. No code reading: the disputes name the function to read (CR-1 step cost, CR-2 day's points, CR-3 reveal), queued as the first step of T7, T6/T7 and T8 (`ADV-Q3`–`ADV-Q5`). Findings that touch the drafts: a path may use explored tiles only (VCMI; question 4 of the user's map check), so the action mask cannot leak; every increment-1 rule runs at the start of the day; increment 1 has no owned objects, so a game opens with only the heroes' surroundings explored (a recorded simplification). VCMI's contact rule for simultaneous turns (reachability at the start of the day) is the recommended model for later.

### 2026-10-09 — T1a: three ADR drafts

Written from decisions 2–9 as they stand; no code. A fresh reviewer agent checked them against the epic, the ADRs and the server draft; its findings are folded in. New beyond the epic, flagged in each draft's header comment for the review: per-object state in a persistent chunked table and `check-arch` tokens against pointer identity; two visiting modes (entering, or from a neighbouring tile; decision 1 had only the first) and heroes blocking in increment 1; visibility resolved at emission; the import's three stages with setup types owned by their validators; `Actor::System` and `Viewer::Spectator` as additive shapes (SRV-Q6), a setup stream, a battle with another player's hero owned by both players (widens decision 8), an inventory of state two players can write. Deferred to increment 3 with the server lane: where a battle's seed and obstacles come from. On acceptance a `shared/` change numbers the drafts and amends ADR-0005/0006, `docs/02` §3 and `docs/03` (`QUEUE.md` inbox).
