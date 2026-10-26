# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 6 **closed**. **Increment 7: rewards on the map** (decisions 121–136, ADV-Q26 to ADV-Q28): T54 done (the battle seam's facts; `ADV-STUB-9`, E10); round 1: **T55a done** in four (decisions 125–128; survey `E08-rewards.md`, its checkpoint ADV-Q28: decisions 129–136, among them batch E7 whole before the build, A-23 only if row E7-23 confirms it, no original limits on objects and their state). Records: `E08-log.md`; increment 6's in `E08-archive.md`.
- **Open pull requests:** `adventure/e08-t55a4` (T55a4, docs only: the survey, `map.toml`, the questions).
- **Next:** merge T55a4's pull request; then T55b (the ADR drafts, ADR-0017 point 11's line for T54 among them, the design note, the cut with batch E7's harness, spec, recipe, run and scoring before the build, decision 132), its cut confirmed with the user at its start.
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v14 (T54), adventure state hash v12 (T54), adventure ruleset part v11 (T50), map import v11 (`MAP_IMPORT_VERSION`, T54), adventure replay format v6 (T51a); the env's action encoding v4 is the AI lane's. Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 6 (last: T45).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ids:** questions `ADV-Q<n>` (next 29), quirks `ADV-QUIRK-<n>` (1 the fringe; 2, 3 the diagonal; 4 neutral stacks; 5, 6 obstacles, split; 7 A-6; 8 formation; 19, 20 A-15, A-16 (no flags); flags 19–24; 15, 17, 18 increment 5's, flags 31, 32; 14, 16 withdrawn, flag 30 retired), stubs `ADV-STUB-<n>` (next 10; 8 and 9 open, E10's), rules `ADV-*` and `GAM-*` (last `GAM-BAT-013`); the E08 surveys' disputes `D-<n>` (next 41) and reading quirk candidates `A-<n>` (next 25). Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-10-26 — increment 7, round 1: T55a4, the common rules and the checkpoint

Baseline ef3c34b (#479 merged; CI's last code run on `master` green; the scoped verify passed, deny skipped). The user's cut: T55a4 whole (decision 128). Survey `E08-rewards.md` §8 (the common rules), §10 (rewritten for T55b), §11 (batch E7, 23 rows), §12 (rule ids), §13 (two fresh agents in turn: the RoE Witch Hut's mask 0xEFBF, A-21; the pathfinder's event stop, A-23; A-22, A-24), §14; four `map.toml` entries amended (0x40E360 merged with #480's). ADV-Q28 asked and answered (decisions 129–136). Docs only.

### 2026-10-26 — increment 7, round 1: T55a3, Pandora's Box, events and the Pyramid

Baseline f91cdd5 (#476 merged; CI green). Measured at the start (the banks' and the Seer's Hut's rewards are decision 121's army-dialog callers); the user's cut: Pandora's Box, events and the Pyramid, the rest placed in the sketch (decision 127). Survey `E08-rewards.md` §7 (the shared reward record, the box, an event's firing and its stop, the reward routine's order, the Pyramid), §0, §1 (bodies), §9 (D-37 to D-40, A-19, A-20), §10; 21 new `map.toml` readings (10 amended), the `reward_record` struct; a throwaway census example (deleted). Docs only. The sandbox restarted three times (the baseline's background run died). The scoped verify on 7bf3a6b passed every step but `check-exe-map` (0x4A39A0 uncited; its entry added in d1e4c1e, then `check-exe-map`, `check-docs` and `check-lane` passed).

### 2026-10-26 — increment 7, round 1: T55a2, the objects that teach

Baseline bef62ab (#474 merged; CI green; the sandbox's baseline verified the code). Measured at the start (about 6 700 instructions on the human path); the user's cut: whole, the University's dialog for its rules only (decision 126). Survey `E08-rewards.md` §6 (the 20 classes; the once-per-hero marks, 32 bits per class), §0, §1 (bodies), §9 (D-33 to D-36, A-17, A-18), §10; 39 new `map.toml` readings (six amended), 18 `structs.toml` fields; a throwaway census example (deleted). Docs only; the scoped verify passed.
