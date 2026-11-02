# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 8 **closed** (increment 8, boats and the sea, by T76's audit, 2026-10-30; its record and design note in `E08-log.md`, its scope and decisions 158–191 in `E08-archive.md`). **Increment 9: mines, the week and owned objects** (decisions 192, 193, 197; the scope in the epic file). Rounds 1 to 5 done (the survey, the drafts and design, batch E9, the ADRs and core content, T82, T83a, T83b1, T83b2a, T83b2b; their records in `E08-log.md`). Round 6: T84 in two halves, the mines first (decision 220): T84a done (the flags, the mines' visits, Leave Guards, their battles; game state hash v21); T84b in two (decision 221): **T84b1** done (`adventure/e08-t84b1` with `shared/e08-t84b1`): the Lighthouses' flag and points (ADV-OWN-005, `lighthouse_points` into `sea_day_points`), the weekly objects' visits (`visit_weekly`, `ObjectMarked`, `ObjectEmpty`; ADV-WEK-001 to -003). Next: **T84b2** the Shipyards, their boat offer and `BuildBoat` (adventure replay format v8), then T85 the day's start (T85a with the audit of the static reads T83b2a left), T86–T87 the gates and the audit.
- **Open pull requests:** see the newest session entry.
- **Next:** once the open pull requests merge (the `shared/` one first), T84b2 (the queue's entry; whole, decision 221).
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v21 (T84a: the army dialog's holder), adventure state hash v19 (T83b2a: the added objects, the fractions), adventure ruleset part v15 (T83b1: `flag_radius`), map import v15 (`MAP_IMPORT_VERSION`; T83a), adventure replay format v7 (T63a); the env's action encoding v5 is the AI lane's (T63a's `cross-lane` bump); shared: core ruleset hash v51, pack format v50 (T81b). No flat observation encoding exists for the adventure (decision 187). Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 9 (last: T81a, with ADR-0013; T83a corrected ADR-0016 point 4's weekly objects, ADV-Q39; no draft open).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Design notes:** the code cites each as "increment N's design note §M"; write a new citation the same way.
- **Ids:** questions `ADV-Q<n>` (next 40), quirks `ADV-QUIRK-<n>` (the ledger `docs/rules/quirks-adventure.md` holds 1 to 32, 14 and 16 withdrawn; flags 19–24, 30 retired, 31, 32, 37, 38; 33 reserved by increment 9's design §16 for A-31; 34 unused, A-33 withdrawn by T80; next 35), stubs `ADV-STUB-<n>` (next 13; 8 and 9 open, E10's; 12 class 85's water tile, creature banks'), rules `ADV-*` and `GAM-*` (last `GAM-BAT-014`; increment 8's ADV-SEA-001 to -007, ADV-PIK-014 to -018, ADV-FOG-008), the E08 surveys' disputes `D-<n>` (next 51) and reading quirk candidates `A-<n>` (next 34; A-23 and A-33 withdrawn); decisions next 222. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-11-02 — increment 9, round 6: T84b1, the Lighthouses and the weekly visits

Baseline d36c4d4 (#632 to #634 merged; CI on `master` green, the setup's light gates). Original data staged at the start (the computer went offline during the session). The user (decision 221): T84b in two, the Lighthouses and the weekly visits first. T84b1: `Effect::Lighthouse`, `visit_lighthouse`, `lighthouse_points` passed to `sea_day_points` (a turn start, the game's start, a boarding, the query's boarding); `Effect::Weekly`, `visit_weekly` (the marks first, the gain or `ObjectEmpty`), `ObjectMarked` (an agent's call, as T84a's `GuardsChanged`); no version moved, the replays unchanged (no replay's map has these objects); bench against `master` within ±0.01 %, none re-baselined. Pull requests: `shared/e08-t84b1` (#635, merge first), `adventure/e08-t84b1`.

### 2026-10-31 — increment 9, round 6: T84a, the flags and the mines' visits

Baseline 9b7ffc5 (#630, #631 merged; CI on `master` green, the setup's light gates). Original data staged at the start (the computer went offline right after). The user (decision 220): T84 in two halves, the mines first. T84a: `Effect::Mine`, `flag`, `visit_mine`, Leave Guards (`Holder::Mine`, `LeaveGuardsOpened`, `set_guards`), the guards' `Fight`, `Defender::Garrison`, the mine's battle terrain, `end_mine_battle` with the flag after the levels, `GuardsChanged` (an agent's call: the view property needs it once the plans have mines); game state hash v21; 32 replays re-recorded, commands unchanged; bench: reach-xl-144 +0.27 % against `master`, none re-baselined. Pull requests: `shared/e08-t84a` (#632, merge first), `adventure/e08-t84a`.

### 2026-10-31 — increment 9, round 5: T83b2b, the views

Baseline 1d0df89 (#627 merged; CI on `master` green on the same engine code; the VM restarted during the setup's baseline, CI's run stands in). Original data staged at the start. The user (decision 219): T83b2b whole, the `shared/` part first; a mine's guards as the original's hint, the total's word counting every stack (an upgraded one too). `ObjectView`: `owner`, `production`, `guards` (`GuardsView`), `visited`, `boat_tile`; `production`, `visited_by`, `allied`; ADV-OWN-003; the h3-map view test on the economy map (its Shipyard has no boat tile as written). No version moved. The epic's done cuts (decisions 194–212) moved to the archive. Pull requests: `shared/e08-t83b2b` (#630, merge first), `adventure/e08-t83b2b`.
