# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 6 **closed**. **Increment 7: rewards on the map** (decisions 121–143, ADV-Q26 to ADV-Q29): T54, the survey T55a, the drafts and design T55b done (drafts `adr-drafts/*-increment-7.md` accepted, answers typed by decision, decision 138; design `E08-design.md`; eight rounds with batch E7 first, then the `shared/` task T59, then the build, decision 139); **T56 done** (decision 142: the writer's reward objects and RoE format, the spec's fields `cross-lane`, the specs and templates `E08-batch-7a`, `-7a-roe`, `-7b`); **T57 done** (decision 143: the harness's reads and steps for batch E7, its three recipes, the three runs, the results filled in `experiments/results/`). Records: `E08-log.md`; increment 6's in `E08-archive.md`.
- **Open pull requests:** `adventure/e08-t57` (T57: the harness, common; the recipes; the filled results; the client and map-client lanes' inboxes told).
- **Next:** merge T57's pull request; then T58 (batch E7 scored: survey §15, A-23 settled), its cut confirmed with the user at its start.
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v14 (T54), adventure state hash v12 (T54), adventure ruleset part v11 (T50), map import v11 (`MAP_IMPORT_VERSION`, T54), adventure replay format v6 (T51a); the env's action encoding v4 is the AI lane's. Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 6 (last: T45).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ids:** questions `ADV-Q<n>` (next 30), quirks `ADV-QUIRK-<n>` (1 the fringe; 2, 3 the diagonal; 4 neutral stacks; 5, 6 obstacles, split; 7 A-6; 8 formation; 19, 20 A-15, A-16 (no flags); flags 19–24; 15, 17, 18 increment 5's, flags 31, 32; 14, 16 withdrawn, flag 30 retired), stubs `ADV-STUB-<n>` (next 10; 8 and 9 open, E10's), rules `ADV-*` and `GAM-*` (last `GAM-BAT-013`; increment 7's planned in survey §12 and design §17); the E08 surveys' disputes `D-<n>` (next 41) and reading quirk candidates `A-<n>` (next 25); decisions next 144. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions


### 2026-10-27 — increment 7, round 2: T57, batch E7's harness and runs

Baseline fbe6679 (#486 merged at the start; CI on `master` green; the sandbox's light gates passed; the stale lock files in the user's `.git` removed). The user's cut: T57 whole (decision 143). The harness (`clients/capture/`, common): an object's word and what it holds, the Universities, a hero's spell points, bonuses and status flags, a battle's opening, a message box's pictures and the choice between two; grids with a selection, approaches to unexplored targets, day waits before a step's reads, `[[require]]` restarts; two traps found and fixed (a far target outside the view; the game's stale tile under a mouse that stays in its view cell). E7-5 and E7-10 got Expert Logistics in the spec (a run without it waited into a sixth day and stopped). The three runs recorded (E7a's fourth try, E7b's third), the results filled; the harness check gained Win's chest and Tomb (passed on its second run). Scoring is T58's.

### 2026-10-26 — increment 7, round 1: T56, batch E7's maps and specs

Baseline 244f604 (#484 merged; CI's last code run on `master` green; the sandbox's light gates passed). The user's cut: T56 whole (decision 142); the AI lane's one line consented (`Monster::treasure`). `h3_map::writer`: `RewardObject` (artifacts, scrolls, Shrines, Witch Huts, boxes, events with their message and reward records), `Monster::treasure`, seven plain classes, `write_in(…, Format::RoE)`; `h3-experiment`: the objects in grids, `gives`, `treasure`, `fill_backpack`, `format`; the three specs compiled with the staged tables, each map's static layer imported and validated (`original_data` on them). A fresh agent reviewed the specs against survey §11: eight findings, all taken (D-40's prediction reversed, the Pyramid entered, E7-4's points, restart rules; `E08-log.md`). The scoped verify with original data: every step but `check-lane` (cross-lane, by design).

### 2026-10-26 — increment 7, round 1: T55b, the drafts, the design and the cut

Baseline b2d2653 (#482 merged; CI green; the sandbox's light gates passed). The user's cut: T55b whole (decision 137). Drafts of ADR-0013, -0016, -0017 for increment 7; `E08-design.md` rewritten (parts A to C, the cut, the exit criteria); decision 133's state measured with a throwaway callgrind program (one block behind one `Arc`; the army left a two-level table). ADV-Q29 asked and answered (decisions 138–141: answers typed by decision, every decision migrated in T63). T56 queued; the AI lane's inbox told. Docs only.
