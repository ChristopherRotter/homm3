# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 7 **closed** (increment 7, rewards on the map, by T65's audit, 2026-10-28). **Increment 8: boats and the sea** (decisions 158, 159; ADV-Q31): boats on the map and in the writer (CLT-Q5), boarding, sailing and landing with Navigation, the sea's artifacts and Favorable Winds, battles with a hero in a boat, the events on water, the sea's four pickups. Round 1 **done**: T66a (survey `E08-sea.md`, ADV-Q32 answered: decisions 162–168) and T66b (decision 169: the drafts `adr-drafts/*-increment-8.md`, `E08-design.md` with the cut; ADV-Q33 accepted as written: decisions 170–173). Records: `E08-log.md`; increment 7's in `E08-archive.md`.
- **The cut** (decision 171; design §18): batch E8 first (T67 its maps and specs; T68 the harness and runs + T69 the scoring), then `shared/` (T70: the ADRs with batch E8's outcome, the three effect kinds, the glossary), then the build (T71 `h3-heroes` + T72 the import, kinds and state; T73 moving and `Reach`; T74 the pickups, the battle at sea, the game and the env's observation, `cross-lane`, decision 172; T75 the replays and gates + T76 the audit).
- **Open pull requests:** #552 (`adventure/e08-t66b`, T66b).
- **Next:** T67 (`QUEUE.md`), its cut confirmed at its start.
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v19 (T63b), adventure state hash v15 (T62b2), adventure ruleset part v12 (T61a), map import v13 (`MAP_IMPORT_VERSION`, T61b), adventure replay format v7 (T63a); the env's action encoding v5 is the AI lane's (T63a's `cross-lane` bump). Increment 8 plans map import v14, ruleset part v13, adventure state hash v16, game state hash v20 (T72; design §16). Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 7 (last: T59, with ADR-0013); increment 8's drafts accepted (decision 170), applied by T70.
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ids:** questions `ADV-Q<n>` (next 34), quirks `ADV-QUIRK-<n>` (1 the fringe; 2, 3 the diagonal; 4 neutral stacks; 5, 6 obstacles, split; 7 A-6; 8 formation; 19, 20 A-15, A-16 (no flags); flags 19–24; 15, 17, 18 increment 5's, flags 31, 32; 14, 16 withdrawn, flag 30 retired; 21, 22 A-21, A-25, flags 37, 38; 23 to 29 A-18, A-20, A-22, A-24, A-17, A-19, A-26, no flags; next 30: design §16 gives 30 to 32 to A-27, A-28 and the 64 boats, no flags), stubs `ADV-STUB-<n>` (next 11; 8 and 9 open, E10's; 10 closed by T63b), rules `ADV-*` and `GAM-*` (last `GAM-BAT-014`; increment 8's planned in survey `E08-sea.md` §12), the E08 surveys' disputes `D-<n>` (next 46) and reading quirk candidates `A-<n>` (next 29; A-23 withdrawn); decisions next 174. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-10-28 — increment 8, round 1: T66b, the drafts, the design note and the cut

Baseline 346100c (#551 merged; the sandbox's baseline: light gates only, passed). `Heroes3.exe` staged from the user's computer. At the start the user answered ADV-Q32 (decisions 162–168: every point as recommended) and took T66b whole (decision 169). The ADR-0013, -0016 and -0017 amendment drafts for increment 8 (`adr-drafts/*-increment-8.md`), `E08-design.md` written for increment 8 (three parts, 19 sections, the cut in eight rounds and nine exit criteria), a reading of the diagonal landing (no side-tile test) and of boarding (from a stop node only; two map entries amended); ADV-Q33 asked with the files sent: accepted as written (decisions 170–173). T67 queued; the AI lane's inbox told of the observation's change. Docs only, no version. Pull request: see "Current state".

### 2026-10-28 — increment 8, round 1: T66a2, the boats and sea survey's second part

Baseline 86c2d35 (#550 merged; CI on `master` green again, e5cfa46's run included: T66a1's report settled). The original data staged from the user's computer (`Heroes3.exe`, `maps.tar`, `extracted.tar`). The user's cut (decision 161): whole. Survey `E08-sea.md` §7 to §14: the battle at sea (the deck by either hero's boat, the two boats by both; nothing else tests a boat), the four pickups read whole (draws, visits, the full backpack, the bottle's text), who takes them (D-45: a flyer from the shore by a click), the interfaces, batch E8's plan (16 rows), the `ADV-SEA` ids, two fresh agents' second reading (A-28 and the winds' drop corrected), ADV-Q32. Docs only, no version. Pull request: see "Current state".

### 2026-10-28 — increment 8, round 1: T66a1, the boats and sea survey's first part

Baseline e5cfa46 (#546, #534 merged; CI's run on it failed in `cargo xtask verify`, the ai lane's #534; see below). The original data staged from the user's computer (`Heroes3.exe`, `maps.tar`, `extracted.tar`). The user's cut (decision 160): T66a in two. Survey `E08-sea.md` §0 to §6: the census (118 boats on 41 maps, no hero on water, 9 events on water, 48 on coasts), the boats' records (64 at most), boarding and landing (only on a free coastal tile, class 3), sailing (100, Favorable Winds 66, the diagonal between two waters), the writer's boat; D-41 to D-44; the address map and `structs.toml`. The inbox's events-on-water note archived. Docs only, no version. The sandbox's full verify of that code passed all fifteen steps (CI's failure looks like the runner's; a re-run of its job on `master` would tell); the light gates and `check-lane` passed on the branch. Pull request: see "Current state".
