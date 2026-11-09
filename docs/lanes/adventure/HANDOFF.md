# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 10 **closed** (increment 10 by T97's audit, 2026-11-05; record in `E08-log.md`, scope, decisions 231–259 and tasks in `E08-archive.md`). **Increment 11: the teleporters** (ADV-Q44; decisions 260, 261; the scope in the epic file): Subterranean Gates (103), Monoliths (43 to 45), Whirlpools (111), on 146 of the 160 shipped maps. Rounds 1 to 6 **done** (`E08-log.md`). Round 7: T105a1, T105a2 (the jumps on land and water, decision 287) and T105b1 (an occupied exit's battle and meeting, `Rest::Jump`; decision 288) **done**; T105b2 next. Round 8 (design §17): T106 the fixtures and benchmarks, T107 the audit.
- **Open pull requests:** #810 (`adventure/e08-play-held-weekly`: the play property's oracle after decision 279's held visit; `master` red since #808 without it) and T105b1's (`adventure/e08-t105b1`, stacked on #810: once #810 is squash-merged, `git rebase --onto origin/master 64fa2c4` and force-push with lease); watch both until they merge, merge `master` in on a conflict.
- **Next:** **T105b2** (`Revisit`, `NothingToRevisit`, the legal set, replay format v11; the env's action encoding v8, `cross-lane`; the `random-play-*` workloads re-baselined; the queue entry), then round 8 (T106, T107).
- **Versions** (`docs/08-work-process.md`, "Versions"; each with its last change): game state hash v27 (T105b1: `Rest::Jump`, tag 4; v26 E09 T17a, the town lane: the store of towns' v2, the day's mark), adventure state hash v21 (E09 T8a1, the town lane: `Holds::Town`), adventure ruleset part v19 (E09 T18a, the town lane: the town's boat offsets; v18 T103a, increment 11's three kinds), map import v21 (`MAP_IMPORT_VERSION`; T103b: the teleporters, 220 by the mines' table), adventure replay format v10 (T95b: `answer hire`); the env's action encoding v7 is the AI lane's (T95b, `cross-lane`); shared: core ruleset hash v52, pack format v51 (T102: `whirlpool_protection`). T104, T105a1 and T105a2 moved none (no replay holds a teleporter). Increment 11's bumps left: design §15 (replay format v11, action encoding v8 in T105b2). No flat observation encoding exists for the adventure (decision 187). Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 11 (last: T102, increment 11's ADR-0016 points 1, 3 to 6 and ADR-0017 points 1, 4, 6, 8, 10, 11, with batch E11's outcome; T92 increment 10's; T81a with ADR-0013 for increment 9; T83a corrected ADR-0016 point 4's weekly objects, ADV-Q39); no draft open.
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Design notes:** the code cites each as "increment N's design note §M"; write a new citation the same way.
- **Ids:** questions `ADV-Q<n>` (next 48; none open), quirks `ADV-QUIRK-<n>` (the ledger `docs/rules/quirks-adventure.md` holds 1 to 33 and 35 to 45, 14 and 16 withdrawn, 34 unused; next 46), stubs `ADV-STUB-<n>` (next 17; open: 8 and 9 E10's, 12 creature banks'; 13 lifted by E09 T11, 16 by T105b1), rules `ADV-*` and `GAM-*` (last `GAM-BAT-014`; the family ADV-TEL: -001 written by T103a, its import by T103b, -002 by T104, -003 to -005 and -007 by T105a1, -006 by T105a2, -008 by T105b2, design §15), the E08 surveys' disputes `D-<n>` (next 59; D-53 to D-58 settled, decision 264) and reading quirk candidates `A-<n>` (next 44; A-37 to A-43 decided, 265–267, 272, 280); decisions next 289; batch E11 (survey §12, design §16: E11-1 to E11-19) scored (survey §15), batch E11c (E11-20 to E11-23) scored (§15.1). Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-11-09 — increment 11, round 7: T105b1, the exit's battle and meeting

Baseline: `master` 31e0ea3 (#806; the setup's full run passed in the sandbox). `master` then moved to 1e8568d (#808), red on CI in `h3-map`'s play property (a random plan: a weekly object's visit held after its guard's battle, which the battle's oracle did not count): fixed first in #810, the seed kept; this branch is stacked on it. Cut in two (decision 288). Staged `maps.tar`, `extracted.tar`, the executable and two LODs. `jump` returns the meeting or the battle with `jump_after`; `Rest::Jump` and `AdventureState::arrive`; `ADV-QUIRK-39`; game state hash v27, the 55 replays rehashed (hash lines only). The random-play workloads within 0.001 % of `master`. PRs (see Current state).

### 2026-11-09 — increment 11, round 7: T105a2, the Whirlpool

Baseline: `master` ae7d260 (#802; the setup's light gates on its verified fingerprint). Staged `maps.tar`, `extracted.tar`, the executable and two LODs. Whole (the water half of decision 287's cut). `Effect::Whirlpool(divisor)`; `jumps.rs`' `visit_whirlpool` (the loss unless `whirlpool_protection`, then `draw_exit` over the Whirlpools' list, the entrance's object excluded) and `whirlpool_loss` (the least `(fight value × count, slot)`, a lone stack keeping one); `WhirlpoolLoss` to the owner alone; the arrival with the boat as built. The sea workloads against `master`'s worktree (in the log). No version moved. PR #806 (CI green).

### 2026-11-09 — increment 11, round 7: T105a1, the land's jumps

Baseline: `master` 14e5335 (#797; CI's `CI` green, the setup's light gates; `Clients` failed with no job, re-run). Staged `maps.tar`, `extracted.tar`, the executable and two LODs. Cut by land and water (decision 287). The stream through `apply` (own commit), `state/jumps.rs` (the draw, the arrival, "leads nowhere", the occupied exit's stub), the events' projections; the reach and random-play workloads within 0.01 % of `master`'s worktree. No version moved. PR #802.
