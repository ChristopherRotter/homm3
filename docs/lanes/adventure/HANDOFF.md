# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 8 **closed** (increment 8, boats and the sea, by T76's audit, 2026-10-30; its record and design note in `E08-log.md`, its scope and decisions 158–191 in `E08-archive.md`). **Increment 9: mines, the week and owned objects** (decisions 192, 193, 197; the scope in the epic file). Rounds 1 to 7 done (their records in `E08-log.md`; T85b's pull requests #654, #655 merged 2026-11-02). Round 8: **T86** done in three (decisions 226 to 228): T86a (#660), T86b1 (#663), **T86b2** (the economy XL `e08-economy-144.h3m`, `--workload day-start`, four new workloads, eight re-baselined).
- **Open pull requests:** #664 `shared/e08-t86b2` (the benches README's rows; merges first), then `adventure/e08-t86b2` (T86b2). Watch both until they merge; merge `master` in on a conflict.
- **Next:** **T87** (the queue's entry: the audit, decision 229's boat tile to the owner's team, increment 9 closed, increment 10 refined) once T86b2 merges.
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v22 (T85b1: the month's kind, the timed events), adventure state hash v19 (T83b2a: the added objects, the fractions), adventure ruleset part v15 (T83b1: `flag_radius`), map import v15 (`MAP_IMPORT_VERSION`; T83a), adventure replay format v8 (T84b2: `BuildBoat`); the env's action encoding v6 is the AI lane's (T85b2b's `cross-lane` bump: `BuildBoat`); shared: core ruleset hash v51, pack format v50 (T81b). No flat observation encoding exists for the adventure (decision 187). Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 9 (last: T81a, with ADR-0013; T83a corrected ADR-0016 point 4's weekly objects, ADV-Q39; no draft open).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Design notes:** the code cites each as "increment N's design note §M"; write a new citation the same way.
- **Ids:** questions `ADV-Q<n>` (next 41; none open), quirks `ADV-QUIRK-<n>` (the ledger `docs/rules/quirks-adventure.md` holds 1 to 32 and 35, 14 and 16 withdrawn; flags 19–24, 30 retired, 31, 32, 37, 38; 33 reserved by increment 9's design §16 for A-31; 34 unused, A-33 withdrawn by T80; next 36), stubs `ADV-STUB-<n>` (next 13; 8 and 9 open, E10's; 12 class 85's water tile, creature banks'), rules `ADV-*` and `GAM-*` (last `GAM-BAT-014`; increment 9's last ADV-MON-014), the E08 surveys' disputes `D-<n>` (next 51) and reading quirk candidates `A-<n>` (next 34; A-23 and A-33 withdrawn); decisions next 230. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-11-03 — increment 9, round 8: T86b2, the economy XL and its benchmarks

Baseline: `master` dd94aa3 (#663 merged), the sandbox's light gates green on CI's fingerprint; the original data staged. The user (decision 228): T86b2 whole; ADV-Q40 answered (decision 229: the boat tile to the owner's team, in T87). `e08-economy-144.h3m` (`economy_xl_map.rs`: the shipped maps' most of each class, 328 monsters; its tests end turns to a creature month); `h3 adventure bench --workload day-start` (`day_start_step`); `day-start-xl-144` (322 k Ir a week's start), `month-start-xl-144` (12.7 M Ir a creature month's, the walk; left for T87), `fork-economy-xl-144`, `random-play-economy-36`; eight workloads re-baselined at `master`'s counts, each reason in its description. No version moved. Pull requests #664 (`shared/`), then `adventure/e08-t86b2`.

### 2026-11-03 — increment 9, round 8: T86b1, the economy plans' properties

Baseline: #660 (T86a) merged at green CI as the user allowed (`master` f1b2f2f); the sandbox's full baseline stopped for CI's (its run on #660's code green). No original data staged. The user (decision 227): T86b in two, the properties first with the town lane's failing case; merge on green CI. The case: the battle oracle read the static map only, a creature month's monster was fought (`visited_at`). T86b1: the plans lay out increment 9's kinds (owners at load, a Shipyard on a strait's coast); `Step::{Garrison, BuildBoat}`, runs of 64 ends of turn; the oracles learn flags, mines' questions and battles, Leave Guards, the boat offer, `BuildBoat` (listed by an oracle, each probed), weekly gains, the week's step, added monsters as placed ones; the view properties vary unseen owners and follow a mine's guards; `the_plans_flag_guard_build_and_grow`. Found: a seen Shipyard's boat tile shows its earlier offsets (`ADV-Q40`, recommended: the owner's team only). No version moved, no replay changed.

### 2026-11-03 — increment 9, round 8: T86a, the economy map and its replays

Baseline: `master` 65b5465 (#654, #655 merged; CI green); the sandbox's light gates failed at `check-exe-map` on `master` (the town lane's #656 cites 0x63EC80): an entry added here, which the client lane's #658 cherry-picked into `master`; `master` e02c8e5 merged in (no engine change). The user (decision 226): T86 in two, the map and replays first; no original data staged (the counts exist since T83a, T83b1). T86a: `e08-economy-36.h3m` written by `h3_map::writer` (`economy_map.rs`); random play over 60 days `economy-s792-d60`, `-s935-d60`, `-s1451-d60` (a search over 3 000 seeds, all 28 features counted); scripted `economy-guards-s1`, `-abandoned-s1`, `-shipyard-s1`, `-windmill-s1`; the duel replays' lists checked: four replaced by a search over 6 000 seeds a profile (`duel-s3560-d28`, `-s4749-d28`, `-s645-d28`, `-s5249-d28`), the inbox's 2026-10-19 note done. No version moved. Pull request #660.
