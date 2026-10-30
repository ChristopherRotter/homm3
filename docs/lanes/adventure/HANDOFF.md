# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 8 **closed** (increment 8, boats and the sea, by T76's audit, 2026-10-30; its record and design note in `E08-log.md`, its scope and decisions 158–191 in `E08-archive.md`). **Increment 9: mines, the week and owned objects** (decisions 192, 193; ADV-Q36): flagging, mines and abandoned mines with the day's income, Lighthouses and Shipyards, the Windmill, the Water Wheel, the Mystical Garden and the monsters' weekly growth; dwellings and hiring are the sketch's next item (10). Round 1: **T77a** the survey (`docs/rules/surveys/E08-economy.md`) in two (decision 194): T77a1 done (what a visit does, §0 to §6), **T77a2** next (what the day's start does, the batch's plan, the ids, the second reading, the checkpoint's `ADV-Q37`; §7 on); then **T77b** the ADR amendment drafts, the design note (`E08-design.md`, a pointer until then) and the cut; the user accepts the drafts before anything is built.
- **Open pull requests:** see the newest session entry. `fork-xl-144` +1.99 % and `fork-meeting-xl-144` +1.93 % still sit at the threshold.
- **Next:** T77a2 (the queue's entry), its cut confirmed with the user at its start.
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v20, adventure state hash v16, adventure ruleset part v13, map import v14 (`MAP_IMPORT_VERSION`; all T72), adventure replay format v7 (T63a); the env's action encoding v5 is the AI lane's (T63a's `cross-lane` bump); shared: core ruleset hash v50, pack format v49 (T70). No flat observation encoding exists for the adventure (decision 187). Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 8 (last: T70, with ADR-0013).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Design notes:** the code cites each as "increment N's design note §M" (T76: all of increments 4, 5, 7 and 8 named); write a new citation the same way.
- **Ids:** questions `ADV-Q<n>` (next 37), quirks `ADV-QUIRK-<n>` (the ledger `docs/rules/quirks-adventure.md` holds 1 to 32, 14 and 16 withdrawn; flags 19–24, 30 retired, 31, 32, 37, 38; next 33), stubs `ADV-STUB-<n>` (next 13; 8 and 9 open, E10's; 12 class 85's water tile, creature banks'), rules `ADV-*` and `GAM-*` (last `GAM-BAT-014`; increment 8's ADV-SEA-001 to -007, ADV-PIK-014 to -018, ADV-FOG-008), the E08 surveys' disputes `D-<n>` (next 49) and reading quirk candidates `A-<n>` (next 32; A-23 withdrawn); decisions next 195. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-10-30 — increment 9, round 1: T77a1, the survey's visits

Baseline 283bfbd (#593 merged; CI on `master` green; the light gates passed). Original data staged at the start (the computer went offline after). The user's cut (decision 194): T77a in two by what runs the rule. Survey `E08-economy.md` §0 to §6: the census, the mine records and the Shipyard's word, flagging by team, a defeat, the viewer's hint, `visit_mine` whole, the abandoned mine, Lighthouses (+500 each, the Castle building's without an owner test), the Shipyard's boat tile fixed at load, its visit and its use without a visit (in part); D-47, D-48, A-29 to A-31; 25 map entries. Pull request: `adventure/e08-t77a1`.

### 2026-10-30 — increment 8 closed (T76's audit); increment 9 refined

Baseline 62c5bf1 (#589, #590 merged; CI on `master` green at d378a77; the light gates passed). The maps and tables staged, then the executable and two LODs. The user's cut (decision 191): whole. The nine exit criteria met on `master`. Fixed: `ADV-QUIRK-30` (A-27) had no ledger entry (now with a computer player's case in the bottle test); ADV-MOV-009, -011 and ADV-OBJ-007 gained batch E8's sources. The design notes' 96 citations named by increment. Increment 8 closed (archive, log); increment 9 refined (ADV-Q36: decisions 192, 193; the census in the log), T77a and T77b queued. Pull request: `adventure/e08-t76`.

### 2026-10-30 — increment 8, round 8: T75b, the properties at sea, the sea XL, the census

Baseline f33f837 (#586, #587 merged; CI on `master` green; the light gates passed). Original data staged. The user's cut (decision 190): whole. The plans' strait (`Sea`: boats laid out last, the coast marked on maps with boats, the four pickups, winds, monsters on water, Navigation and the sea's artifacts); `check_boats` and `check_sea_move` after every command, a leaving find's gains, the views following the boats, `the_plans_sail`. The property found a passage bug, fixed: a flight up off a closed top passed an object's water tile (D-46; ADV-PIK-018 amended). The sea XL `e08-sea-144.h3m` with `reach-sea-xl-144` (about 2.29 M Ir a boat's query: the inbox) and `fork-sea-xl-144`; the census as T72's. No version. Pull requests: `adventure/e08-t75b`, `shared/e08-t75b-benches`.
