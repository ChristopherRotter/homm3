# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 8 **closed** (increment 8, boats and the sea, by T76's audit, 2026-10-30; its record and design note in `E08-log.md`, its scope and decisions 158–191 in `E08-archive.md`). **Increment 9: mines, the week and owned objects** (decisions 192, 193, 197; the scope in the epic file). Rounds 1 to 7 done (their records in `E08-log.md`; T85b's pull requests #654, #655 merged 2026-11-02). Round 8: **T86 in two** (decision 226): **T86a** done (#660 merged 2026-11-03); **T86b in two** (decision 227): **T86b1** done (the plans' economy classes and steps, design §15's checks, the town lane's failing case).
- **Open pull requests:** T86b1's `adventure/e08-t86b1` (the user: merge it once CI is green; this entry names it until then). Watch it until it merges; merge `master` in on a conflict.
- **Next:** **T86b2** (the queue's entry: the economy XL, the four workloads, the re-baselines, a `shared/` pull request for the benches README) once T86b1 merges, then T87 (the audit, increment 9 closed, increment 10 refined).
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v22 (T85b1: the month's kind, the timed events), adventure state hash v19 (T83b2a: the added objects, the fractions), adventure ruleset part v15 (T83b1: `flag_radius`), map import v15 (`MAP_IMPORT_VERSION`; T83a), adventure replay format v8 (T84b2: `BuildBoat`); the env's action encoding v6 is the AI lane's (T85b2b's `cross-lane` bump: `BuildBoat`); shared: core ruleset hash v51, pack format v50 (T81b). No flat observation encoding exists for the adventure (decision 187). Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 9 (last: T81a, with ADR-0013; T83a corrected ADR-0016 point 4's weekly objects, ADV-Q39; no draft open).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Design notes:** the code cites each as "increment N's design note §M"; write a new citation the same way.
- **Ids:** questions `ADV-Q<n>` (next 40), quirks `ADV-QUIRK-<n>` (the ledger `docs/rules/quirks-adventure.md` holds 1 to 32 and 35, 14 and 16 withdrawn; flags 19–24, 30 retired, 31, 32, 37, 38; 33 reserved by increment 9's design §16 for A-31; 34 unused, A-33 withdrawn by T80; next 36), stubs `ADV-STUB-<n>` (next 13; 8 and 9 open, E10's; 12 class 85's water tile, creature banks'), rules `ADV-*` and `GAM-*` (last `GAM-BAT-014`; increment 9's last ADV-MON-014), the E08 surveys' disputes `D-<n>` (next 51) and reading quirk candidates `A-<n>` (next 34; A-23 and A-33 withdrawn); decisions next 228. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-11-03 — increment 9, round 8: T86b1, the economy plans' properties

Baseline: #660 (T86a) merged at green CI as the user allowed (`master` f1b2f2f); the sandbox's full baseline stopped for CI's (its run on #660's code green). No original data staged. The user (decision 227): T86b in two, the properties first with the town lane's failing case; merge on green CI. The case: the battle oracle read the static map only, a creature month's monster was fought (`visited_at`). T86b1: the plans lay out increment 9's kinds (owners at load, a Shipyard on a strait's coast); `Step::{Garrison, BuildBoat}`, runs of 64 ends of turn; the oracles learn flags, mines' questions and battles, Leave Guards, the boat offer, `BuildBoat` (listed by an oracle, each probed), weekly gains, the week's step, added monsters as placed ones; the view property varies unseen owners; `the_plans_flag_guard_build_and_grow`. No version moved, no replay changed.

### 2026-11-03 — increment 9, round 8: T86a, the economy map and its replays

Baseline: `master` 65b5465 (#654, #655 merged; CI green); the sandbox's light gates failed at `check-exe-map` on `master` (the town lane's #656 cites 0x63EC80): an entry added here, which the client lane's #658 cherry-picked into `master`; `master` e02c8e5 merged in (no engine change). The user (decision 226): T86 in two, the map and replays first; no original data staged (the counts exist since T83a, T83b1). T86a: `e08-economy-36.h3m` written by `h3_map::writer` (`economy_map.rs`); random play over 60 days `economy-s792-d60`, `-s935-d60`, `-s1451-d60` (a search over 3 000 seeds, all 28 features counted); scripted `economy-guards-s1`, `-abandoned-s1`, `-shipyard-s1`, `-windmill-s1`; the duel replays' lists checked: four replaced by a search over 6 000 seeds a profile (`duel-s3560-d28`, `-s4749-d28`, `-s645-d28`, `-s5249-d28`), the inbox's 2026-10-19 note done. No version moved. Pull request #660.

### 2026-11-02 — increment 9, round 7: T85b2b, the env's `BuildBoat`; T85b's pull requests

Baseline: `master` 3ec41c6 merged into both branches (no engine code), the sandbox's full verify with the original data green. Original data staged (the LODs, the executable, `maps.tar`, `extracted.tar`). T85b2b: `h3-sim`'s action encoding v6, `BUILD_BOAT` (entry 17) with the Shipyard's anchor as its tile, masked by the listed `BuildBoat`s; the random agent now builds where it drew one (no committed map has a Shipyard). Agent's calls: decode by the adventure's checks, the game's refused by `apply`. Bench against `master`: +0.21 % to +0.43 % (T85b1's; the events' firing unmeasurable), `fork-sea-xl-144` re-baselined at +1.94 % of its count. The AI lane's inbox told. Pull requests #654 (`shared/e08-t85b`), then #655 (`adventure/e08-t85b`, `cross-lane`).
