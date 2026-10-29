# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 7 **closed** (increment 7, rewards on the map, by T65's audit, 2026-10-28). **Increment 8: boats and the sea** (decisions 158, 159; ADV-Q31): boats on the map and in the writer (CLT-Q5), boarding, sailing and landing with Navigation, the sea's artifacts and Favorable Winds, battles with a hero in a boat, the events on water, the sea's four pickups. Rounds 1 to 4 **done** (T66a to T70; decisions 160–181, the epic's list): the survey `E08-sea.md`, the ADR amendments (ADR-0013, -0016, -0017, applied by T70) and `E08-design.md` with the cut; batches E8 and E8c run and scored (survey §15, §16): D-41 to D-43 confirmed, D-44's refusal kept (ADV-Q34), D-45 reverted, **D-46 confirmed** (no flight over a water tile holding an object but a boat, a hero in a boat or class 85; ADV-Q35). Records: `E08-log.md`; increment 7's in `E08-archive.md`.
- **The cut** (decision 171; design §18): batch E8 first (T67 its maps and specs; T68 the harness and runs + T69 the scoring; T69b batch E8c, D-46), then `shared/` (T70, done: the ADRs, the three effect kinds, the glossary), then the build (T71 `h3-heroes` + T72 the import, kinds and state; T73 moving and `Reach`; T74 the pickups, the battle at sea, the game and the env's observation, `cross-lane`, decision 172; T75 the replays and gates + T76 the audit).
- **Open pull requests:** see the newest session entry (`shared/e08-t70`, T70).
- **Next:** T71 (`h3-heroes`: the day's sea points and free boarding, HER-ADV-009; `QUEUE.md`), once T70's pull request merged; the inbox's failing property case wants a fix before T72 (it may join T71).
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v19 (T63b), adventure state hash v15 (T62b2), adventure ruleset part v12 (T61a), map import v13 (`MAP_IMPORT_VERSION`, T61b), adventure replay format v7 (T63a); the env's action encoding v5 is the AI lane's (T63a's `cross-lane` bump); shared: core ruleset hash v50, pack format v49 (T70). Increment 8 plans map import v14, ruleset part v13, adventure state hash v16, game state hash v20 (T72; design §16). Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 8 (last: T70, with ADR-0013; increment 8's drafts accepted by decision 170).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ids:** questions `ADV-Q<n>` (next 36), quirks `ADV-QUIRK-<n>` (1 the fringe; 2, 3 the diagonal; 4 neutral stacks; 5, 6 obstacles, split; 7 A-6; 8 formation; 19, 20 A-15, A-16 (no flags); flags 19–24; 15, 17, 18 increment 5's, flags 31, 32; 14, 16 withdrawn, flag 30 retired; 21, 22 A-21, A-25, flags 37, 38; 23 to 29 A-18, A-20, A-22, A-24, A-17, A-19, A-26, no flags; next 30: design §16 gives 30 to 32 to A-27, A-28 and the 64 boats, no flags), stubs `ADV-STUB-<n>` (next 11; 8 and 9 open, E10's; 10 closed by T63b), rules `ADV-*` and `GAM-*` (last `GAM-BAT-014`; increment 8's planned in survey `E08-sea.md` §12), the E08 surveys' disputes `D-<n>` (next 47) and reading quirk candidates `A-<n>` (next 29; A-23 withdrawn); decisions next 182. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-10-29 — increment 8, round 4: T70, the `shared/` task

Baseline 25ff803 (#566 merged; CI on `master` green). The installation's files staged for the harvester. The user's cut (decision 181): whole, one `shared/` pull request, `cross-lane`. The three drafts applied to ADR-0013 (points 3, 13), ADR-0016 (points 1, 3, 4, 6) and ADR-0017 (points 1, 8, 10, 11), then deleted; the glossary's sea terms. `navigation`, `sea_movement_points`, `free_boarding` (core ruleset hash v50, pack format v49); the gap-fill (battle's) and `base`, `sod` re-harvested; the 31 replays' header lines and the bots reports moved only by the hash. T71 queued. Pull request: `shared/e08-t70`.

### 2026-10-29 — increment 8, round 3: T69b, batch E8c

Baseline 454810a (#565 merged; CI on `master` green; the light gates passed). The installation's files staged for the harness, as T68a. The user's cut (decision 180): whole. The spec `E08-batch-8c` (row E8-17: five flyers in pockets closed by rock; E8-17e, an empty boat crossed from water, the agent's addition), its recipe (existing reads), one run under Wine, recorded. Survey `E08-sea.md` §16: **D-46 confirmed** (no path over a Flotsam's water tile from land or water); a boat crossed from land and water; T69's re-read of 0x4B2CF3 corrected (map entry 0x4B2300, `E08-hero-movement.md` §3.1). The drafts and design in line; T70 queued. No version. Pull request: `adventure/e08-t69b`.

### 2026-10-29 — increment 8, round 3: T69, batch E8's scoring

Baseline c8c1569 (#562 merged; CI on `master` green at 64e2d5d). Docs and the address map only; `Heroes3.exe` staged for a re-read. The user's cut (decision 177): whole, with a re-read of the flyer's search. Survey `E08-sea.md` §15: the 16 rows scored; D-44's refusal kept (ADV-Q34); D-45 reverted (E8-7); E8-15's battle at the monster's tile; A-28 confirmed. The re-read (`map.toml` 0x4B2300, 0x56A0D0): no path ends on a water entrance for a hero not in a boat, and no flight passes one: **D-46**, against ADV-MOV-009 and VCMI; ADV-Q35 (decision 179): batch E8c first (T69b). The drafts, the design and the battle lane's inbox (BAT-TER-002, BAT-GEO-004) updated. No version. Pull request: `adventure/e08-t69`.
