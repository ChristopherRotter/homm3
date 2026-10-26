# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 5 **closed**. **Increment 6: army and hero management** (decisions 90–118; survey `E08-army.md`, design `E08-design.md`): rounds 1 to 4 done (T44a to T51b). Records: `E08-log.md`.
- **Next:** merge #468 (`adventure/e08-t51b`); then round 5, T52 + T53 (`QUEUE.md`; `E08-design.md` §12, §15): confirm the cut with the user at its start.
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v13 (T51b), adventure state hash v11, adventure ruleset part v11 (T50), map import v10 (`MAP_IMPORT_VERSION`, T49b), adventure replay format v6 (T51a); the env's action encoding v4 is the AI lane's. Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 6 (last: T45).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ids:** questions `ADV-Q<n>`, quirks `ADV-QUIRK-<n>` (1 the fringe; 2, 3 the diagonal; 4 neutral stacks; 5, 6 obstacles, split; 7 A-6; 8 formation; 19, 20 A-15, A-16 (no flags); flags 19–24; 15, 17, 18 increment 5's, flags 31, 32; 14, 16 withdrawn, flag 30 retired), stubs `ADV-STUB-<n>`, rules `ADV-*` and `GAM-*`. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-10-26 — increment 6, round 4: T51b; round 4 closed

Baseline f72f8c2 (full verify green). T51b whole (decision 118): the meeting with Scholar, the army dialog with the join offered without room, `Manage` between two armies; game state hash v13, `ADV-STUB-2` closed; the ten replays' commands unchanged. Full verify green; #468 open.

### 2026-10-24 — increment 6, round 4: T51a

Baseline e816f20. T51 cut in two (decision 116), the listing per hero (decision 117, ADR-0017 amended first); `Manage` on one hero, the flag in battle, replay format v6.

### 2026-10-24 — increment 6, round 4: T50

Baseline f89aa89. T50 whole (decision 113), the cap first (#455); the no-room offer left to T51; four duplicate decision-112 blocks removed from the epic.
