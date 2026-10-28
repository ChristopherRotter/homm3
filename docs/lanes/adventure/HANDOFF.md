# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 6 **closed**. **Increment 7: rewards on the map** (decisions 121–154, ADV-Q26 to ADV-Q30): T54 to T63a done (T63a #530 merged); round 7's **T63b** in progress (`adventure/e08-t63b`; decision 154, whole, the guard's wait fixed in it). Records: `E08-log.md`; increment 6's in `E08-archive.md`.
- **Open pull requests:** none yet (T63b's opens when the branch is ready).
- **Next:** finish T63b (see the newest session); then T64 + T65.
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v18 (T63a), adventure state hash v15 (T62b2), adventure ruleset part v12 (T61a), map import v13 (`MAP_IMPORT_VERSION`, T61b), adventure replay format v7 (T63a); the env's action encoding v5 is the AI lane's (T63a's `cross-lane` bump). Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 7 (last: T59, with ADR-0013).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ids:** questions `ADV-Q<n>` (next 31), quirks `ADV-QUIRK-<n>` (1 the fringe; 2, 3 the diagonal; 4 neutral stacks; 5, 6 obstacles, split; 7 A-6; 8 formation; 19, 20 A-15, A-16 (no flags); flags 19–24; 15, 17, 18 increment 5's, flags 31, 32; 14, 16 withdrawn, flag 30 retired; 21, 22 A-21, A-25, flags 37, 38; 23 to 29 A-18, A-20, A-22, A-24, A-17, A-19, A-26, no flags; next 30), stubs `ADV-STUB-<n>` (next 11; 8 and 9 open, E10's; 10 open, T63b's), rules `ADV-*` and `GAM-*` (last `GAM-BAT-013`; `HER-CHG-003`, `-004` built by T60b; `GAM-SET-003`, `ADV-OBJ-006`, `ADV-PIK-005`, `-009` to `-012`, `ADV-TCH-003` to `-008` by T61b; `ADV-PIK-006` to `-008`, `ADV-MON-011`, `ADV-TCH-001`, `-002`, `ADV-VIS-006` by T62a; `ADV-RWD-001`, `ADV-PIK-013`, `ADV-MON-012` by T62b1; `ADV-OBJ-007`, `ADV-MOV-011`, `ADV-FOG-007` by T62b2; `GAM-DEC-005`, `-006` by T63a; the rest of increment 7's in survey §12 and design §17); the E08 surveys' disputes `D-<n>` (next 41) and reading quirk candidates `A-<n>` (next 27; A-23 withdrawn); decisions next 155. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-10-28 — increment 7, round 7: T63b, the rest (in progress)

Baseline fdf002f (#530 merged, CI green on its merge; the sandbox's light gates passed; the original data staged from the user's computer). The user's cut (decision 154): whole, with a bug found while sizing it fixed in it (a guard's visit started beside what an answer opened, a gift's level-up beside the guard's battle). In progress: the rest's mechanism in `h3-game`, then the won battle's tail, `ClearBonuses`, the event after the guard; nothing half-done yet.

### 2026-10-27 — increment 7, round 7: T63a, the typed answers and increment 7's decisions

Baseline dc4d7e6 (#526 merged; the sandbox's light gates passed; the original data staged from the user's computer). The user's cut (decision 153): T63 in two, the design's; consent to the `cross-lane` mapping with a University's purchase entries now. `h3-game`: `Answer` typed by kind, `Decision::Choice` and `::University` with views, legal answers, defaults, `SlotClosed`, `CannotPay`; game state hash v18. A bug fixed: a guard's answer dropped an entered object's hero gains (since T62a). `h3-sim`: answers by role, entries 13–16 (encoding v5). Replay format v7, the twelve re-recorded. Pull request: see "Current state".

### 2026-10-27 — increment 7, round 6: T62b2, events and the move

Baseline 8ef8244 (#521 merged, CI green; the sandbox's full verify passed; the original data staged from the user's computer). The user's cut (decision 152): whole. The census first: 486 events, every template blocking its tile (an event blocks nothing: 0x505C78). `h3-adventure`: events apart from the masks (`StaticMap::events_at`, `MapObject::event`), `state/events.rs` (the fire, the guards, the record, the removal), the move's stop (`AdventureOutcome::dropped`), `EventFired` to the owner, never in a view; the seats (`AdventureSetup::seats`, adventure state hash v15); `ADV-QUIRK-29` (A-26), an event on a guarded tile waiting for T63 (`ADV-STUB-10`); the hidden-event property in `h3-game`; the twelve replays re-recorded. Pull request #526, merged.
