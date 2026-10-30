# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 8 **closed** (increment 8, boats and the sea, by T76's audit, 2026-10-30; its record and design note in `E08-log.md`, its scope and decisions 158–191 in `E08-archive.md`). **Increment 9: mines, the week and owned objects** (decisions 192, 193, 197; the scope in the epic file). Round 1 done: the survey (`docs/rules/surveys/E08-economy.md`; T77a1, T77a2, `ADV-Q37`) and T77b (decision 200) the amendment drafts (`adr-drafts/*-increment-9.md`), the design note `E08-design.md` and the cut; `ADV-Q38` answered (decisions 201–204). Round 2 done: T78 (decisions 205, 206) the writer's ownable objects, weekly objects and timed events, the spec's fields (`cross-lane`), the specs `E08-batch-9a` and `-9b`. Round 3: T79 in two by batch (decision 207): T79a done (batch E9a's harness and runs, `experiments/results/E08-batch-9a.md`, `-run2.md`); **T79b** batch E9b's next, with T80 the scoring, then T81 `shared/`, T82–T85 the build, T86–T87 the gates and the audit.
- **Open pull requests:** see the newest session entry. `fork-xl-144` +1.99 % and `fork-meeting-xl-144` +1.93 % still sit at the threshold.
- **Next:** T79b (the queue's entry; T80 after it), its cut confirmed with the user at its start.
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v20, adventure state hash v16, adventure ruleset part v13, map import v14 (`MAP_IMPORT_VERSION`; all T72), adventure replay format v7 (T63a); the env's action encoding v5 is the AI lane's (T63a's `cross-lane` bump); shared: core ruleset hash v50, pack format v49 (T70). No flat observation encoding exists for the adventure (decision 187). Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 8 (last: T70, with ADR-0013).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Design notes:** the code cites each as "increment N's design note §M"; write a new citation the same way.
- **Ids:** questions `ADV-Q<n>` (next 39), quirks `ADV-QUIRK-<n>` (the ledger `docs/rules/quirks-adventure.md` holds 1 to 32, 14 and 16 withdrawn; flags 19–24, 30 retired, 31, 32, 37, 38; 33 and 34 reserved by increment 9's design §16; next 35), stubs `ADV-STUB-<n>` (next 13; 8 and 9 open, E10's; 12 class 85's water tile, creature banks'), rules `ADV-*` and `GAM-*` (last `GAM-BAT-014`; increment 8's ADV-SEA-001 to -007, ADV-PIK-014 to -018, ADV-FOG-008), the E08 surveys' disputes `D-<n>` (next 51) and reading quirk candidates `A-<n>` (next 34; A-23 withdrawn; A-33 the design's); decisions next 208. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-10-30 — increment 9, round 3: T79a, batch E9a's harness and runs

Baseline ef32f96 (#603 merged; the setup's baseline: light gates). Original data staged at the start, every MP3 with it. The user's cut (decision 207): T79 in two by batch. The harness's mine records, tile words, the boat dialog, Leave Guards, the battle's sides, `auto again` (auto combat is refused when both sides are human: an owned mine's guards are their owner's), `MINEEVNT` named; the recipe `E08-batch-9a`; four runs, two recorded. The harness check passed; the client and map-client inboxes told. Pull request: `adventure/e08-t79a`.

### 2026-10-30 — increment 9, round 2: T78, batch E9's maps and specs

Baseline 1b8db97 (#601 merged; CI on `master` green; the VM restarted early, killing the setup's baseline: CI stands in). Original data staged at the start (the computer went offline after). The user's cut (decision 205): whole; decision 206: the writer's `Map::events`, one line in the client, map-client and AI lanes. The writer's `MapObject::Owned`, the weekly objects, timed events; the spec's `[[map.ownable]]`, `[[map.monster]]`, `[[map.timed_event]]`; the specs `E08-batch-9a`, `-9b`, every leg priced; a fresh reviewer's ten findings taken (a town each in 9b: the seven-day loss). Pull request: `adventure/e08-t78` (`cross-lane`).

### 2026-10-30 — increment 9, round 1: T77b, the drafts, the design note and the cut; the checkpoint

Baseline 56976c7 (#599 merged; CI on `master` green at d2d4c70). Original data staged at the start (the computer went offline after). The user's cut (decision 200): whole. The three amendment drafts, `E08-design.md` for increment 9 (19 sections; a reading of its own: the specialists' factor stays in the x87 register, no Q-0005 term), the cut in rounds 2 to 8 (T78 to T87). `ADV-Q38` answered (decisions 201–204). The AI and map-client inboxes told. Pull request: `adventure/e08-t77b`.
