# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 8 **closed** (increment 8, boats and the sea, by T76's audit, 2026-10-30; its record and design note in `E08-log.md`, its scope and decisions 158–191 in `E08-archive.md`). **Increment 9: mines, the week and owned objects** (decisions 192, 193, 197; the scope in the epic file). Rounds 1 to 4 done (the survey, the drafts and design, batch E9 run and scored, the ADRs and the core content: T77a1 to T81b; their records in `E08-log.md`). Round 5 (decision 213: T82 with T83a, T83 in two): **T82** and **T83a** done (#621, merged). T83b in two by what reads it (decision 215): **T83b1** done (`adventure/e08-t83b1`, `cross-lane`; decision 217: the flag radius is data): the owners and the start's reveal, a mine's guards, the abandoned mines' and Shipyards' draws, the first week's weekly contents, `check_owners`, `check_weekly`. T83b2 in two, the state first (decision 218): **T83b2a** done (`adventure/e08-t83b2a`): the monsters' fractions, the objects added in play in the live presence, `check_added`. **T83b2b** done (decision 219; `adventure/e08-t83b2b` with `shared/e08-t83b2b`): the views (`ObjectView`'s owner, production, guards as the original's hint, visited mark, boat tile; ADV-OWN-003). Next: round 6, **T84** the visits, then T85 the day's start (T85a with the audit of the static reads T83b2a left), T86–T87 the gates and the audit.
- **Open pull requests:** see the newest session entry.
- **Next:** once the open pull requests merge (the `shared/` one first), T84 (the queue's entry), its cut confirmed with the user at the start.
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v20, adventure state hash v19 (T83b2a: the added objects, the fractions), adventure ruleset part v15 (T83b1: `flag_radius`), map import v15 (`MAP_IMPORT_VERSION`; T83a), adventure replay format v7 (T63a); the env's action encoding v5 is the AI lane's (T63a's `cross-lane` bump); shared: core ruleset hash v51, pack format v50 (T81b). No flat observation encoding exists for the adventure (decision 187). Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 9 (last: T81a, with ADR-0013; T83a corrected ADR-0016 point 4's weekly objects, ADV-Q39; no draft open).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Design notes:** the code cites each as "increment N's design note §M"; write a new citation the same way.
- **Ids:** questions `ADV-Q<n>` (next 40), quirks `ADV-QUIRK-<n>` (the ledger `docs/rules/quirks-adventure.md` holds 1 to 32, 14 and 16 withdrawn; flags 19–24, 30 retired, 31, 32, 37, 38; 33 reserved by increment 9's design §16 for A-31; 34 unused, A-33 withdrawn by T80; next 35), stubs `ADV-STUB-<n>` (next 13; 8 and 9 open, E10's; 12 class 85's water tile, creature banks'), rules `ADV-*` and `GAM-*` (last `GAM-BAT-014`; increment 8's ADV-SEA-001 to -007, ADV-PIK-014 to -018, ADV-FOG-008), the E08 surveys' disputes `D-<n>` (next 51) and reading quirk candidates `A-<n>` (next 34; A-23 and A-33 withdrawn); decisions next 220. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-10-31 — increment 9, round 5: T83b2b, the views

Baseline 1d0df89 (#627 merged; CI on `master` green on the same engine code; the VM restarted during the setup's baseline, CI's run stands in). Original data staged at the start. The user (decision 219): T83b2b whole, the `shared/` part first; a mine's guards as the original's hint, the total's word counting every stack (an upgraded one too). `ObjectView`: `owner`, `production`, `guards` (`GuardsView`), `visited`, `boat_tile`; `production`, `visited_by`, `allied`; ADV-OWN-003; the h3-map view test on the economy map (its Shipyard has no boat tile as written). No version moved. The epic's done cuts (decisions 194–212) moved to the archive. Pull requests: `shared/e08-t83b2b` (#630, merge first), `adventure/e08-t83b2b`.

### 2026-10-31 — increment 9, round 5: T83b2a, the fractions and the objects added in play

Baseline d98f792 (#622 and #624 merged; CI on `master` green; the VM restarted early, the setup's baseline lost, CI's run stands in). Original data staged at the start. The user: T83b2 in two, the state first (decision 218). T83b2a: the added objects' own block (`Added`) and the readers, `object`/`object_at`, `fractions`, `check_added` (the plans and the census), `MapObject`'s own hash, adventure state hash v19; the first cut cost the searches +9 to +16 % (fixed: the block, the walking search's rare case, `live_placed`; random-play-sea-36 re-baselined, +2.47 % against `master`); 32 replays re-recorded, commands unchanged; the bot reports unmoved. Pull request: `adventure/e08-t83b2a`.

### 2026-10-31 — increment 9, round 5: T83b1, the state the visits read; the shore's boat

Baseline eef7863 (#621 merged; CI on `master` green, the setup's light gates). Original data staged at the start. The user: T83b in two by what reads it (decision 215); the shore's border gate case of the inbox is a walker boarding an empty boat a key holder left on its gate's water tile, so the property's oracle was fixed, not the engine (decision 216, `adventure/e08-shore-boat`, which also records 215 and 217); the flag radius is data (decision 217, part v15, the bot reports `cross-lane`). T83b1: the static map's lists and draws, the owners, guards, weekly contents, the start's reveal, the invariants, adventure state hash v18; the census (Hoard's Shipyard alone without a boat tile); 32 replays re-recorded, commands unchanged. Pull requests: #622 (`adventure/e08-shore-boat`; merge first), #624 (`adventure/e08-t83b1`, `cross-lane`).
