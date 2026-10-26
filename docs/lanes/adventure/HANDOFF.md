# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 6 **closed**. **Increment 7: rewards on the map** (decisions 121–142, ADV-Q26 to ADV-Q29): T54, the survey T55a, the drafts and design T55b done (drafts `adr-drafts/*-increment-7.md` accepted, answers typed by decision, decision 138; design `E08-design.md`; eight rounds with batch E7 first, then the `shared/` task T59, then the build, decision 139); **T56 done** (decision 142: the writer's reward objects and RoE format, the spec's fields `cross-lane`, the specs and templates `E08-batch-7a`, `-7a-roe`, `-7b`). Records: `E08-log.md`; increment 6's in `E08-archive.md`.
- **Open pull requests:** `adventure/e08-t56` (T56, `cross-lane`: `h3-map`'s writer, `h3-experiment` and its README, one line of the AI lane's `h3-cli` census test).
- **Next:** merge T56's pull request; then round 2: T57 (batch E7's harness and runs) and T58 (the scoring), T57's cut confirmed with the user at its start.
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v14 (T54), adventure state hash v12 (T54), adventure ruleset part v11 (T50), map import v11 (`MAP_IMPORT_VERSION`, T54), adventure replay format v6 (T51a); the env's action encoding v4 is the AI lane's. Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 6 (last: T45).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ids:** questions `ADV-Q<n>` (next 30), quirks `ADV-QUIRK-<n>` (1 the fringe; 2, 3 the diagonal; 4 neutral stacks; 5, 6 obstacles, split; 7 A-6; 8 formation; 19, 20 A-15, A-16 (no flags); flags 19–24; 15, 17, 18 increment 5's, flags 31, 32; 14, 16 withdrawn, flag 30 retired), stubs `ADV-STUB-<n>` (next 10; 8 and 9 open, E10's), rules `ADV-*` and `GAM-*` (last `GAM-BAT-013`; increment 7's planned in survey §12 and design §17); the E08 surveys' disputes `D-<n>` (next 41) and reading quirk candidates `A-<n>` (next 25); decisions next 143. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions


### 2026-10-26 — increment 7, round 1: T56, batch E7's maps and specs

Baseline 244f604 (#484 merged; CI's last code run on `master` green; the sandbox's light gates passed). The user's cut: T56 whole (decision 142); the AI lane's one line consented (`Monster::treasure`). `h3_map::writer`: `RewardObject` (artifacts, scrolls, Shrines, Witch Huts, boxes, events with their message and reward records), `Monster::treasure`, seven plain classes, `write_in(…, Format::RoE)`; `h3-experiment`: the objects in grids, `gives`, `treasure`, `fill_backpack`, `format`; the three specs compiled with the staged tables, each map's static layer imported and validated (`original_data` on them). A fresh agent reviewed the specs against survey §11: eight findings, all taken (D-40's prediction reversed, the Pyramid entered, E7-4's points, restart rules; `E08-log.md`). The scoped verify with original data: every step but `check-lane` (cross-lane, by design).

### 2026-10-26 — increment 7, round 1: T55b, the drafts, the design and the cut

Baseline b2d2653 (#482 merged; CI green; the sandbox's light gates passed). The user's cut: T55b whole (decision 137). Drafts of ADR-0013, -0016, -0017 for increment 7; `E08-design.md` rewritten (parts A to C, the cut, the exit criteria); decision 133's state measured with a throwaway callgrind program (one block behind one `Arc`; the army left a two-level table). ADV-Q29 asked and answered (decisions 138–141: answers typed by decision, every decision migrated in T63). T56 queued; the AI lane's inbox told. Docs only.

### 2026-10-26 — increment 7, round 1: T55a4, the common rules and the checkpoint

Baseline ef3c34b (#479 merged; CI's last code run on `master` green; the scoped verify passed, deny skipped). The user's cut: T55a4 whole (decision 128). Survey `E08-rewards.md` §8 (the common rules), §10 (rewritten for T55b), §11 (batch E7, 23 rows), §12 (rule ids), §13 (two fresh agents in turn: the RoE Witch Hut's mask 0xEFBF, A-21; the pathfinder's event stop, A-23; A-22, A-24), §14; four `map.toml` entries amended (0x40E360 merged with #480's). ADV-Q28 asked and answered (decisions 129–136). Docs only.
