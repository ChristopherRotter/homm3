# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 10 **closed** (increment 10 by T97's audit, 2026-11-05; its record in `E08-log.md`, its design note there too (T97c), its scope, decisions 231–259 and task list in `E08-archive.md`). **Increment 11: the teleporters** (refined with the user at T97c, ADV-Q44; decisions 260, 261; the scope in the epic file): Subterranean Gates (103), Monoliths (43 to 45), Whirlpools (111), on 146 of the 160 shipped maps (T97c's census in `E08-log.md`). Round 1: the survey `docs/rules/surveys/E08-teleporters.md` (T98a) and the ADR amendment drafts with the design note and the cut (T98b); nothing built before the user accepts the drafts.
- **Open pull requests:** #747, T97c's (`adventure/e08-t97c`, docs only); watch it until it merges, merge `master` in on a conflict.
- **Next:** **T98a** (its queue entry: the census, the bodies, the code reading of a teleporter's jump, the reach and the paths through it, the views, batch E11's plan; likely in two, its cut confirmed at its start; the original data and the shipped maps staged), once #747 merges; then T98b.
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v25 (T95b: the hire dialog; v24 E09 T7b2a, the town lane: the store of towns; v23 E09 T7b1: the hero pool), adventure state hash v21 (E09 T8a1, the town lane: the towns ownable, `Holds::Town`; v20 T94b: the dwellings' weeks, the camps), adventure ruleset part v17 (E09 T8a1, the town lane: the kind `Town`; v16 T93a: the dwellings' kinds, parameters and tables), map import v19 (`MAP_IMPORT_VERSION`; T97b1: the dwellings entered from water, a tied random dwelling's checks; v18 E09 T8a1, the town lane: the towns typed with their owners), adventure replay format v10 (T95b: `answer hire`; v9 E09 T5a2, the town lane: the town part's hash); the env's action encoding v7 is the AI lane's (T95b's `cross-lane` bump: the hire dialog's four purchases; v6 T85b2b: `BuildBoat`); shared: core ruleset hash v51, pack format v50 (T81b). No flat observation encoding exists for the adventure (decision 187). Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 10 (last: T92, increment 10's ADR-0016 points 1, 3 to 6 and ADR-0017 points 1, 3, 4, 6, 8, 10, 11; T81a with ADR-0013 for increment 9; T83a corrected ADR-0016 point 4's weekly objects, ADV-Q39); no open drafts.
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Design notes:** the code cites each as "increment N's design note §M"; write a new citation the same way.
- **Ids:** questions `ADV-Q<n>` (next 45; none open), quirks `ADV-QUIRK-<n>` (the ledger `docs/rules/quirks-adventure.md` holds 1 to 33 and 35, 14 and 16 withdrawn; flags 19–24, 30 retired, 31, 32, 37, 38; 34 unused, A-33 withdrawn by T80; 36 to 38 A-34 to A-36's, T93b; next 39; D-51 none, decision 238), stubs `ADV-STUB-<n>` (next 16; 8 and 9 open, E10's; 12 class 85's water tile, creature banks'; 13 a random dwelling tied to a town, E09's; 14 and 15 closed by T95a2 and T95b), rules `ADV-*` and `GAM-*` (last `GAM-BAT-014`; increment 9's last ADV-MON-014; increment 10's proposed in the survey's §14), the E08 surveys' disputes `D-<n>` (next 53) and reading quirk candidates `A-<n>` (next 37; A-23 and A-33 withdrawn); decisions next 262; batch E11 planned by T98a (batch E10 scored, T91). Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-11-05 — increment 10 closed, increment 11 refined: T97c

Baseline: `master` 7cc511d (#744 merged at the start); docs only (no build; the docs gates run). The computer's shell did not start; `maps.tar` and `extracted.tar` staged for a census of the classes still left blockers (a throwaway example over `h3_map::read`, not committed; `E08-log.md` T97c). ADV-Q44: the user chose the teleporters (recommended: the second level of 98 maps), the first round a survey and the drafts (decisions 260, 261). Increment 10's section moved to `E08-archive.md`, its design note to the log; the queue's T97 entry, its chronicle and five settled or informational inbox notes to `QUEUE-ARCHIVE.md`; T98a and T98b queued.

### 2026-11-05 — increment 10, round 8: T97b2, the plans' dwellings on water

Baseline: `master` 7f8c8c5 (#742 merged at the start; the setup's full baseline ran beside the change, written in a worktree); the original data staged at the start (the computer's shell did not start, staging worked; the task needed none). Whole, by the main session, tests only: `lay_out` lays out dwellings and random dwellings entered from water; the boat plans (`afloat_plans`) and their cover and view tests. Every property oracle ran on their 512 cases in a throwaway test and passed; the view checks found one gap of their own (a hero that lands out of a viewer's sight and walks back into it), fixed in `check_told`. Two cover tests' cases raised (the trap). No version moved.

### 2026-11-05 — increment 10, round 8: T97b1, the dwellings entered from water

Baseline: `master` 273b55d (#738 merged; the setup's full baseline died with a VM restart in its coverage step, CI's full verify on 0e2f186 and 273b55d green); the original data staged at the start (the computer's shell did not start, staging worked). The user: T97b in two (decision 259). The reading was short: no boat test in the visit, the hire or the camp; the guards on the deck; VCMI agrees; the engine already behaved so, only the import changed. Census: all 160 shipped maps import (9 dwellings on water on the four maps). The 47 replays: only `import_version` moved. Map import v19.
