# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 7 **closed** (increment 7, rewards on the map, by T65's audit, 2026-10-28). **Increment 8: boats and the sea** (decisions 158, 159; ADV-Q31): boats on the map and in the writer (CLT-Q5), boarding, sailing and landing with Navigation, the sea's artifacts and Favorable Winds, battles with a hero in a boat, the events on water, the sea's four pickups. Rounds 1 to 5 **done** (T66a to T72; decisions 160–183, the epic's list; T72 whole, `cross-lane`: the import, the kinds and the sea's parameters, the boats, the views): the survey `E08-sea.md`, the ADR amendments (ADR-0013, -0016, -0017, applied by T70) and `E08-design.md` with the cut; batches E8 and E8c run and scored (survey §15, §16): D-41 to D-43 confirmed, D-44's refusal kept (ADV-Q34), D-45 reverted, **D-46 confirmed** (no flight over a water tile holding an object but a boat, a hero in a boat or class 85; ADV-Q35). Records: `E08-log.md`; increment 7's in `E08-archive.md`.
- **The cut** (decision 171; design §18): batch E8 first (T67 its maps and specs; T68 the harness and runs + T69 the scoring; T69b batch E8c, D-46), then `shared/` (T70, done: the ADRs, the three effect kinds, the glossary), then the build (T71 `h3-heroes`, done: HER-ADV-009 + T72, done: the import, kinds, state and views; T73 moving and `Reach`; T74 the pickups, the battle at sea, the game and the env's observation, `cross-lane`, decision 172; T75 the replays and gates + T76 the audit).
- **Open pull requests:** see the newest session entry (`adventure/e08-t72`, T72, `cross-lane`).
- **Next:** T73 (moving at sea, the prices and the conversion, `Reach`, visits at sea; `QUEUE.md`), once T72's pull request merged; big, its seam confirmed at its start. Pending `shared/`: `docs/06-security.md`'s viewer list for increment 8, once T74 built the events.
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v20, adventure state hash v16, adventure ruleset part v13, map import v14 (`MAP_IMPORT_VERSION`; all T72), adventure replay format v7 (T63a); the env's action encoding v5 is the AI lane's (T63a's `cross-lane` bump); shared: core ruleset hash v50, pack format v49 (T70). The env's observation encoding moves with T74 (`cross-lane`, design §16). Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 8 (last: T70, with ADR-0013; increment 8's drafts accepted by decision 170).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ids:** questions `ADV-Q<n>` (next 36), quirks `ADV-QUIRK-<n>` (1 the fringe; 2, 3 the diagonal; 4 neutral stacks; 5, 6 obstacles, split; 7 A-6; 8 formation; 19, 20 A-15, A-16 (no flags); flags 19–24; 15, 17, 18 increment 5's, flags 31, 32; 14, 16 withdrawn, flag 30 retired; 21, 22 A-21, A-25, flags 37, 38; 23 to 29 A-18, A-20, A-22, A-24, A-17, A-19, A-26, no flags; 32 the 64 boats (T72); 30, 31 reserved for A-27 (T74) and A-28 (T73), no flags; next 33), stubs `ADV-STUB-<n>` (next 11; 8 and 9 open, E10's; 10 closed by T63b), rules `ADV-*` and `GAM-*` (last `GAM-BAT-014`; increment 8's planned in survey `E08-sea.md` §12, ADV-SEA-001, -007, ADV-PIK-014 to -016 and ADV-FOG-008 written by T72), the E08 surveys' disputes `D-<n>` (next 47) and reading quirk candidates `A-<n>` (next 29; A-23 withdrawn); decisions next 184. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-10-29 — increment 8, round 5: T72, the import, the kinds, the state, the views

Baseline 43c8cdd (#571 merged; CI on `master` green; the light gates passed). The original data staged (`Heroes3.exe`, both bitmap archives, `maps.tar`, `extracted.tar`). The user's cut (decision 183): whole, one pull request; with consent the AI lane's two bots tables' adventure fingerprint (`cross-lane`). The sea's kinds and parameters (part v13), `MapSetup::boats` and `state/boats.rs` (adventure state hash v16, game v20), the views, the import of the seven classes and `HeroOnWater` (map import v14); the census: 118 boats on 41 maps, every static layer valid. The generated maps' blocker class 124 → 118; the 21 goldens re-recorded, commands unchanged. ADV-SEA-001, -007, ADV-PIK-014 to -016, ADV-FOG-008, `ADV-QUIRK-32`. Pull request: `adventure/e08-t72`.

### 2026-10-29 — increment 8, round 5: T71, the sea points and free boarding

Baseline 86f164b (#568 merged; CI on `master` green; the light gates passed). No original data staged. The user's cut (decision 182): T71 with the inbox's property case. `h3_heroes::{sea_points, free_boarding}` (any source of the three kinds counts, the agent's call), HER-ADV-009 `verified` (E8-1, E8-6, E8-10). The property case was the plan's: after an army dialog's `Close` a held event fired and its guards' battle opened (GAM-DEC-007); the `Close` branch now checks its own closing, the regression line kept. No version. Pull request: `adventure/e08-t71`.

### 2026-10-29 — increment 8, round 4: T70, the `shared/` task

Baseline 25ff803 (#566 merged; CI on `master` green). The installation's files staged for the harvester. The user's cut (decision 181): whole, one `shared/` pull request, `cross-lane`. The three drafts applied to ADR-0013 (points 3, 13), ADR-0016 (points 1, 3, 4, 6) and ADR-0017 (points 1, 8, 10, 11), then deleted; the glossary's sea terms. `navigation`, `sea_movement_points`, `free_boarding` (core ruleset hash v50, pack format v49); the gap-fill (battle's) and `base`, `sod` re-harvested; the 31 replays' header lines and the bots reports moved only by the hash. T71 queued. Pull request: `shared/e08-t70`.
