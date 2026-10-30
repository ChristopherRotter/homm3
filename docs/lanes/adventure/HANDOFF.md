# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 8 **closed** (increment 8, boats and the sea, by T76's audit, 2026-10-30; its record and design note in `E08-log.md`, its scope and decisions 158–191 in `E08-archive.md`). **Increment 9: mines, the week and owned objects** (decisions 192, 193, 197; the scope in the epic file). Round 1 done: the survey (`docs/rules/surveys/E08-economy.md`; T77a1, T77a2, `ADV-Q37`) and T77b (decision 200) the amendment drafts (`adr-drafts/*-increment-9.md`), the design note `E08-design.md` and the cut; `ADV-Q38` answered (decisions 201–204). Round 2 done: T78 (decisions 205, 206) the writer's ownable objects, weekly objects and timed events, the spec's fields (`cross-lane`), the specs `E08-batch-9a` and `-9b`. Round 3: T79 in two by batch (decision 207): T79a done (batch E9a's harness and runs, `experiments/results/E08-batch-9a.md`, `-run2.md`); T79b done whole (decision 208; batch E9b's harness and runs, `E08-batch-9b.md`, `-run2.md`); T80 done (decisions 209, 210: whole with A-33's re-read; survey §19: D-47, D-49, A-32 and design §13's call confirmed, A-33 withdrawn). Round 4: **T81** `shared/` next, then T82–T85 the build, T86–T87 the gates and the audit.
- **Open pull requests:** see the newest session entry. `fork-xl-144` +1.99 % and `fork-meeting-xl-144` +1.93 % still sit at the threshold.
- **Next:** T81 (the queue's entry), its cut confirmed with the user at its start.
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v20, adventure state hash v16, adventure ruleset part v13, map import v14 (`MAP_IMPORT_VERSION`; all T72), adventure replay format v7 (T63a); the env's action encoding v5 is the AI lane's (T63a's `cross-lane` bump); shared: core ruleset hash v50, pack format v49 (T70). No flat observation encoding exists for the adventure (decision 187). Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 8 (last: T70, with ADR-0013).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Design notes:** the code cites each as "increment N's design note §M"; write a new citation the same way.
- **Ids:** questions `ADV-Q<n>` (next 39), quirks `ADV-QUIRK-<n>` (the ledger `docs/rules/quirks-adventure.md` holds 1 to 32, 14 and 16 withdrawn; flags 19–24, 30 retired, 31, 32, 37, 38; 33 reserved by increment 9's design §16 for A-31; 34 unused, A-33 withdrawn by T80; next 35), stubs `ADV-STUB-<n>` (next 13; 8 and 9 open, E10's; 12 class 85's water tile, creature banks'), rules `ADV-*` and `GAM-*` (last `GAM-BAT-014`; increment 8's ADV-SEA-001 to -007, ADV-PIK-014 to -018, ADV-FOG-008), the E08 surveys' disputes `D-<n>` (next 51) and reading quirk candidates `A-<n>` (next 34; A-23 and A-33 withdrawn); decisions next 211. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-10-30 — increment 9, round 3: T80, batch E9 scored; A-33 withdrawn

Baseline 1597854 (#608, #609 merged; CI on `master` running, the setup's full baseline stopped). Original data staged at the start (the computer went offline later). The user's cut (decisions 209, 210): whole with A-33's re-read; the experiments README's E8 and E9 rows `cross-lane`. Survey `E08-economy.md` §19: every row as read but E9-4; `reveal_owned` read whole (`adventure_manager_start` calls it at the start); the drafts, design and catalog updated. Pull request: `adventure/e08-t80` (`cross-lane`).

### 2026-10-30 — increment 9, round 3: T79b, batch E9b's harness and runs

Baseline 661683d (#607 merged; CI on `master` green; the scoped verify with original data passed every step). Original data staged at the start, every MP3 with it. The user's cut (decision 208): whole. The harness's income, Garden mask, the weekly objects' and monsters' words (through a hero on an entrance), an end-turn step's messages, repeated row labels `<label>#<n>`, the Gold bonus; the recipe `E08-batch-9b` (90 steps, fifteen days); runs recorded as `E08-batch-9b.md` and `-run2.md`. The client and map-client inboxes told. Pull request: `adventure/e08-t79b`.

### 2026-10-30 — increment 9, round 3: T79a, batch E9a's harness and runs

Baseline ef32f96 (#603 merged; the setup's baseline: light gates). Original data staged at the start, every MP3 with it. The user's cut (decision 207): T79 in two by batch. The harness's mine records, tile words, the boat dialog, Leave Guards, the battle's sides, `auto again` (auto combat is refused when both sides are human: an owned mine's guards are their owner's), `MINEEVNT` named; the recipe `E08-batch-9a`; five runs, two recorded (the last on the harness merged with #605, #606). The harness check passed; the client and map-client inboxes told. Pull request: `adventure/e08-t79a`.
