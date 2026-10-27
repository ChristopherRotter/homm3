# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 6 **closed**. **Increment 7: rewards on the map** (decisions 121–145, ADV-Q26 to ADV-Q30): T54, the survey T55a, the drafts and design T55b done (drafts `adr-drafts/*-increment-7.md` accepted, answers typed by decision, decision 138; design `E08-design.md`; eight rounds with batch E7 first, then the `shared/` task T59, then the build, decision 139); T56 done (decision 142: batch E7's maps and specs); T57 done (decision 143: the harness, the recipes, the runs); **T58 done** (decision 144: batch E7 scored in survey `E08-rewards.md` §15; every dispute's row the code's side; **A-23 withdrawn**, a path crosses an event of any colour, the drafts and design say so); T58b done (the view test's oracle; ADV-Q30, decision 145: A-25 a quirk with a flag); **T59 done** (decision 146: whole, `shared/e08-t59`: the three drafts applied to ADR-0013, -0016, -0017 with A-25's flag, the drafts deleted; `HeroParams::mana_ceiling` 999; flags 37 `roe_witch_hut_mask`, 38 `full_backpack_refuses`; the glossary). Records: `E08-log.md`; increment 6's in `E08-archive.md`.
- **Open pull requests:** #497 `shared/e08-t59` (T59, `cross-lane`).
- **Next:** merge T59's pull request; then T60 (`h3-heroes`: design part A), its cut confirmed with the user at its start.
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v14 (T54), adventure state hash v12 (T54), adventure ruleset part v11 (T50), map import v11 (`MAP_IMPORT_VERSION`, T54), adventure replay format v6 (T51a); the env's action encoding v4 is the AI lane's. Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 7 (last: T59, with ADR-0013).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ids:** questions `ADV-Q<n>` (next 31), quirks `ADV-QUIRK-<n>` (1 the fringe; 2, 3 the diagonal; 4 neutral stacks; 5, 6 obstacles, split; 7 A-6; 8 formation; 19, 20 A-15, A-16 (no flags); flags 19–24; 15, 17, 18 increment 5's, flags 31, 32; 14, 16 withdrawn, flag 30 retired; 21, 22 A-21, A-25, flags 37, 38; next 23), stubs `ADV-STUB-<n>` (next 10; 8 and 9 open, E10's), rules `ADV-*` and `GAM-*` (last `GAM-BAT-013`; increment 7's planned in survey §12 and design §17); the E08 surveys' disputes `D-<n>` (next 41) and reading quirk candidates `A-<n>` (next 26; A-23 withdrawn); decisions next 147. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions


### 2026-10-27 — increment 7, round 3: T59, the shared part

Baseline 18f3cb1 (#496 merged; CI on `master` green; the sandbox's light gates passed; one stale lock in the user's `.git` removed). The user's cut: T59 whole, one `shared/` pull request (decision 146). The three drafts applied to ADR-0013, -0016 and -0017, A-25's flag written into ADR-0013 point 15.4 and ADR-0016 point 4 (the drafts predated decision 145); `HeroParams::mana_ceiling` (pack format v48, core ruleset hash v49), the harvester's `exe-mana-ceiling`; `ADV-QUIRK-21`, `-22` (flags 37, 38) in the ledger; the glossary; the 22 replays' headers and the AI lane's bots' tables; the epic file trimmed (the done tasks' cuts to the archive); T60 queued.

### 2026-10-27 — increment 7, round 2: T58, batch E7 scored, A-23 withdrawn; T58b, the view test

Baseline 828a703 (#490 merged; CI on `master` green; the sandbox's light gates passed; one stale lock in the user's `.git` removed). The user's cut: T58 whole with a re-read (decision 144), after the agent found E7-23's corridor half untested (its targets unexplored, beyond the radius 5). Survey §15: every dispute's row the code's side, conclusive against VCMI or the wiki; the full-backpack and consensus rows confirmed; a full backpack tested before a free position; the team's mark in a visited word. The re-read (`Heroes3.exe`): the placement clears an event tile's entrance bit (0x505C78), so the pathfinder never stops there: A-23 withdrawn, the blind way the rule. The Corpse's 20 % read again (14 of 40: chance). HER-EXP-001 and HER-CHG-002 `verified`; the drafts, design, map and the AI lane's inbox updated; T59 queued. Then **T58b** (the user: "fix the random test failure now", after #494 merged): the seed reproduced; a new day's reveal in `strict-fixed` showed a standing hero without an event, which the oracle allowed only after the viewer's own command: the oracle fixed, the seed committed. A-25 (the user: the full backpack "sounds like a bug") a quirk with a flag (ADV-Q30, decision 145).

### 2026-10-27 — increment 7, round 2: T57, batch E7's harness and runs

Baseline fbe6679 (#486 merged at the start; CI on `master` green; the sandbox's light gates passed; the stale lock files in the user's `.git` removed). The user's cut: T57 whole (decision 143). The harness (`clients/capture/`, common): an object's word and what it holds, the Universities, a hero's spell points, bonuses and status flags, a battle's opening, a message box's pictures and the choice between two; grids with a selection, approaches to unexplored targets, day waits before a step's reads, `[[require]]` restarts; two traps found and fixed (a far target outside the view; the game's stale tile under a mouse that stays in its view cell). E7-5 and E7-10 got Expert Logistics in the spec (a run without it waited into a sixth day and stopped). The three runs recorded (E7a's fourth try, E7b's third), the results filled; the harness check gained Win's chest and Tomb (passed on its second run). Scoring is T58's.
