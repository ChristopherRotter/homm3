# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 6 **closed**. **Increment 7: rewards on the map** (decisions 121–156, ADV-Q26 to ADV-Q30): T54 to T64a done (T64a #538 merged); round 8: **T64b** done (`adventure/e08-t64b`, decision 156, whole: increment 7's kinds in `h3-map`'s property plans with their checks and `check_rewards`, the cover test, the hidden-event property with a guard, decision 133's maps `fixtures/maps/e08-{boxes,stones}-144.h3m` and `fork-boxes-xl-144`, `fork-stones-xl-144`). **T64 done.** Records: `E08-log.md`; increment 6's in `E08-archive.md`.
- **Open pull requests:** T64b's, from `adventure/e08-t64b`, and `shared/e08-t64b-benches` (the benches README's two rows for its workloads).
- **Next:** merge T64b's pull requests; then T65 (`QUEUE.md`: the audit of increment 7's exit criteria, increment 7 closed, increment 8 refined with the user), its cut confirmed at its start.
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v19 (T63b), adventure state hash v15 (T62b2), adventure ruleset part v12 (T61a), map import v13 (`MAP_IMPORT_VERSION`, T61b), adventure replay format v7 (T63a); the env's action encoding v5 is the AI lane's (T63a's `cross-lane` bump). Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 7 (last: T59, with ADR-0013).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ids:** questions `ADV-Q<n>` (next 31), quirks `ADV-QUIRK-<n>` (1 the fringe; 2, 3 the diagonal; 4 neutral stacks; 5, 6 obstacles, split; 7 A-6; 8 formation; 19, 20 A-15, A-16 (no flags); flags 19–24; 15, 17, 18 increment 5's, flags 31, 32; 14, 16 withdrawn, flag 30 retired; 21, 22 A-21, A-25, flags 37, 38; 23 to 29 A-18, A-20, A-22, A-24, A-17, A-19, A-26, no flags; next 30), stubs `ADV-STUB-<n>` (next 11; 8 and 9 open, E10's; 10 closed by T63b), rules `ADV-*` and `GAM-*` (last `GAM-BAT-014`; increment 7's built so far by task in `E08-log.md`, the rest in survey §12 and design §17); the E08 surveys' disputes `D-<n>` (next 41) and reading quirk candidates `A-<n>` (next 27; A-23 withdrawn); decisions next 157. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-10-28 — increment 7, round 8: T64b, the properties and the 144 × 144 maps

Baseline c4ae2fa (#537, #538 merged, CI green; the sandbox's light gates passed; the original data staged from the user's computer). The user's cut (decision 156): whole. `h3-map`: 35 kinds of increment 7 in the property plans (`REWARDS`), the driver's checks taught their offers, choices, gains, losses and levels, `check_rewards`, `the_plans_visit_every_kind_and_answer_every_decision` (512 cases), the view property's end at a visit's contents, `the_plans_meet_and_manage` at 320 cases; `tests/full_maps.rs` with `e08-{boxes,stones}-144.h3m`; `h3-game`'s hidden-event property with a guard; `fork-boxes-xl-144`, `fork-stones-xl-144` (2 367 Ir a fork); no version, no engine bug found (4 000 cases of the properties run once). Pull requests: see "Current state".

### 2026-10-28 — increment 7, round 8: T64a, the rewards map, its replays and benchmarks

Baseline 87613c9 (#533 merged, CI green; the sandbox's light gates passed; the original data staged from the user's computer). The user's cut (decision 155): T64 in two. `h3-map`: `tests/rewards_map.rs` writes `fixtures/maps/e08-rewards-36.h3m` with `h3_map::writer` (every class of increment 7, the original's templates; beside red's three heroes the order-dependent paths); nine golden replays on it (three of random play from a search over 400 seeds, six scripted: design §16's paths and the two T63b added); `h3-cli`: `BenchResult::visits`, the workloads `random-play-rewards-36` and `fork-rewards-36`; no version. The scoped verify with the original data passed. Pull requests: see "Current state".

### 2026-10-28 — increment 7, round 7: T63b, the rest

Baseline fdf002f (#530 merged, CI green on its merge; the sandbox's light gates passed; the original data staged from the user's computer). The user's cut (decision 154): whole, with a bug found while sizing it fixed in it (a guard's visit started beside what an object's answer opened; since T63a an entered gift's level-up opened beside the guard's battle). `h3-game`: the rests (`state/rests.rs`, GAM-DEC-007), `ClearBonuses` after every battle (GAM-BAT-014), game state hash v19; `h3-adventure`: `guard_after` the hero for every visited object, `reward_after` and `give_won`, `event_after` and `fire_held_event` (`ADV-QUIRK-29`); `ADV-STUB-10` closed; the twelve replays re-recorded (commands unchanged). The scoped verify with the original data passed. Pull request #533.
