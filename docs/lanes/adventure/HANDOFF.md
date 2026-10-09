# Adventure lane — HANDOFF

The adventure lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E08 refined** (2026-10-08, with E07's map reader inside it; reviewed with the user the same day, PR #21): `docs/epics/E08-adventure-map.md`, decisions 1–12 with the review's corrections. Increment 1 = T1a, T1b, T2–T10 (`QUEUE.md`); real `.h3m` maps from the start, every object an inert blocker; four new crates `h3-map`, `h3-pack-adventure`, `h3-adventure`, `h3-game` (no domain crate depends on another; `h3-game` holds the whole-game setup, players, turn groups, calendar and RNG streams; the hero crate keeps its translations, ADR-0013 point 2).
- **T1a drafts merged** (PR #32, 2026-10-10): `adr-drafts/crate-shape.md`, `adventure-state.md`, `game-orchestration.md`; open points `ADV-Q1` (`PlayerId`/`HeroId` in `h3-contracts`, blocks T2) and `ADV-Q2` (how a replay names its map, blocks T9). Draft 3 point 2 answers the server lane's SRV-Q6 in shape.
- **T1b survey written** (2026-10-09, branch `adventure/t1b-survey`): `docs/rules/surveys/E08-adventure-core.md`; the lane's ledger `docs/rules/quirks-adventure.md` (no entries yet); quirk candidates A-1 (the fringe, `ADV-QUIRK-1` once confirmed) to A-3 wait for the user's classification and the user's look at a map; disputes as `ADV-Q3`–`ADV-Q5`, each settled by a code reading in the task that needs it.
- **Next:** the user's acceptance of the T1a drafts (ADV-Q1, ADV-Q2) and the T1b classification (A-1 to A-3, the look at a map); then T2 (shared skeleton).
- **Not before E04b closes:** the split of `h3-content`/`h3-pack` and the move of the `.h3m` writer into `h3-map` (decision 3). Until then adventure crates use only the core items of `h3-content` (keys, packs, resources, factions, terrain ids, creature identity/cost/speed, hero types, artifact and spell identity).
- **Ownership:** the lane owns (`lanes.toml`) `h3-adventure` and its docs now; T2 adds the other new crates and `packs/*/adventure.toml`. `h3-heroes` moves to this lane when hero progression starts (increment 3).
- **Ids:** questions `ADV-Q<n>`, quirks `ADV-QUIRK-<n>` (the fog fringe is `ADV-QUIRK-1`), stubs `ADV-STUB-<n>`. ADR drafts go to `docs/lanes/adventure/adr-drafts/` and get a number when the user accepts them.
- **Working agreement:** increments of a couple of tasks with a user checkpoint, as in the battle lane; the user confirms seams and ADRs; token efficiency beats speed (AGENTS.md).

## Standing traps

- Cheap forks are a hard requirement (decision 6): no per-fork deep copy of map-sized data; pointer identity never in logic or the hash.
- RNG seeds and stream positions are server-only state (decision 9): never in an observation, never seeded from client data.

## Recent sessions

### 2026-10-09 — T1b: survey (movement, fog, the day, simultaneous turns)

Sources: VCMI 1.7.5, the wiki's Movement and Scouting pages (the user approved the wiki for this session), the executable's tables already mapped, the HD mod's page. No code reading: the disputes name the function to read (CR-1 step cost, CR-2 day's points, CR-3 reveal), queued as the first step of T7, T6/T7 and T8 (`ADV-Q3`–`ADV-Q5`). Findings that touch the drafts: a path may use explored tiles only (VCMI; question 4 of the user's map check), so the action mask cannot leak; every increment-1 rule runs at the start of the day; increment 1 has no owned objects, so a game opens with only the heroes' surroundings explored (a recorded simplification). VCMI's contact rule for simultaneous turns (reachability at the start of the day) is the recommended model for later.

### 2026-10-09 — T1a: three ADR drafts

Written from decisions 2–9 as they stand; no code. A fresh reviewer agent checked them against the epic, the ADRs and the server draft; its findings are folded in. New beyond the epic, flagged in each draft's header comment for the review: per-object state in a persistent chunked table and `check-arch` tokens against pointer identity; two visiting modes (entering, or from a neighbouring tile; decision 1 had only the first) and heroes blocking in increment 1; visibility resolved at emission; the import's three stages with setup types owned by their validators; `Actor::System` and `Viewer::Spectator` as additive shapes (SRV-Q6), a setup stream, a battle with another player's hero owned by both players (widens decision 8), an inventory of state two players can write. Deferred to increment 3 with the server lane: where a battle's seed and obstacles come from. On acceptance a `shared/` change numbers the drafts and amends ADR-0005/0006, `docs/02` §3 and `docs/03` (`QUEUE.md` inbox).

### 2026-10-07/08 — refinement session (E07/E08)

The brief's five points settled with the user: real maps first (reader fills E07), the crate split per context after E04b, `h3-game` from increment 1, `h3-heroes` keeps only the hero's own state and rules, cheap forks via a wide persistent grid (quadtree rejected), fog fringe as a quirk candidate (faithful: objects shown; strict-fixed: terrain only), simultaneous turns designed for, not built (turn groups, per-player server-only RNG streams, no blocking prompts; the commutation target captured, its decision deferred). Epic file, queue and this HANDOFF written; no code.

**Review with the user (2026-10-08, PR #21):** the hero crate keeps `derive_battle_hero`/`war_machines` (nothing moves into `h3-game`; ADR-0013 holds); T1 split into T1a (three ADR drafts, one decision each) and T1b (survey, quirks); only four crates (no `h3-content-adventure`: the adventure rules compile inside `h3-adventure`); added to the epic: the import boundary and setup types as an ADR topic, the leavable own tile, map input bounds and the missing fuzz harness, the two copy-on-write properties, the viewer knowledge list, adventure sub-paths in battle's crates and the versions rows for T2, `docs/formats/h3m.md` to adventure, the C ABI non-goal, E09/E10's residual scope, exit criteria, the fringe quirk's confirmation (Q30).
