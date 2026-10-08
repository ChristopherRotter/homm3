# Adventure lane — HANDOFF

The adventure lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E08 refined** (2026-10-08, with E07's map reader inside it; reviewed with the user the same day, PR #21): `docs/epics/E08-adventure-map.md`, decisions 1–12 with the review's corrections. Increment 1 = T1a, T1b, T2–T10 (`QUEUE.md`); real `.h3m` maps from the start, every object an inert blocker; four new crates `h3-map`, `h3-pack-adventure`, `h3-adventure`, `h3-game` (no domain crate depends on another; `h3-game` holds the whole-game setup, players, turn groups, calendar and RNG streams; the hero crate keeps its translations, ADR-0013 point 2).
- **Next:** T1a (three ADR drafts, written from the decisions), then a checkpoint with the user; T1b (survey, quirks) runs beside T2. Branches: `adventure/<task>`.
- **Not before E04b closes:** the split of `h3-content`/`h3-pack` and the move of the `.h3m` writer into `h3-map` (decision 3). Until then adventure crates use only the core items of `h3-content` (keys, packs, resources, factions, terrain ids, creature identity/cost/speed, hero types, artifact and spell identity).
- **Ownership:** the lane owns (`lanes.toml`) `h3-adventure` and its docs now; T2 adds the other new crates and `packs/*/adventure.toml`. `h3-heroes` moves to this lane when hero progression starts (increment 3).
- **Ids:** questions `ADV-Q<n>`, quirks `ADV-QUIRK-<n>` (the fog fringe is `ADV-QUIRK-1`), stubs `ADV-STUB-<n>`. ADR drafts go to `docs/lanes/adventure/adr-drafts/` and get a number when the user accepts them.
- **Working agreement:** increments of a couple of tasks with a user checkpoint, as in the battle lane; the user confirms seams and ADRs; token efficiency beats speed (AGENTS.md).

## Standing traps

- Cheap forks are a hard requirement (decision 6): no per-fork deep copy of map-sized data; pointer identity never in logic or the hash.
- RNG seeds and stream positions are server-only state (decision 9): never in an observation, never seeded from client data.

## Recent sessions

### 2026-10-07/08 — refinement session (E07/E08)

The brief's five points settled with the user: real maps first (reader fills E07), the crate split per context after E04b, `h3-game` from increment 1, `h3-heroes` keeps only the hero's own state and rules, cheap forks via a wide persistent grid (quadtree rejected), fog fringe as a quirk candidate (faithful: objects shown; strict-fixed: terrain only), simultaneous turns designed for, not built (turn groups, per-player server-only RNG streams, no blocking prompts; the commutation target captured, its decision deferred). Epic file, queue and this HANDOFF written; no code.

**Review with the user (2026-10-08, PR #21):** the hero crate keeps `derive_battle_hero`/`war_machines` (nothing moves into `h3-game`; ADR-0013 holds); T1 split into T1a (three ADR drafts, one decision each) and T1b (survey, quirks); only four crates (no `h3-content-adventure`: the adventure rules compile inside `h3-adventure`); added to the epic: the import boundary and setup types as an ADR topic, the leavable own tile, map input bounds and the missing fuzz harness, the two copy-on-write properties, the viewer knowledge list, adventure sub-paths in battle's crates and the versions rows for T2, `docs/formats/h3m.md` to adventure, the C ABI non-goal, E09/E10's residual scope, exit criteria, the fringe quirk's confirmation (Q30).
