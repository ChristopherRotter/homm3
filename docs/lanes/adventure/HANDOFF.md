# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 10 **closed** (increment 10 by T97's audit, 2026-11-05; record in `E08-log.md`, scope, decisions 231–259 and tasks in `E08-archive.md`). **Increment 11: the teleporters** (ADV-Q44; decisions 260, 261; the scope in the epic file): Subterranean Gates (103), Monoliths (43 to 45), Whirlpools (111), on 146 of the 160 shipped maps. Rounds 1 to 5 **done** (the survey, the design note and the ADRs, batch E11 and E11c run and scored, the kinds, the import, decision 279's order; `E08-log.md`). Round 6 **done**: **T104** (whole, decision 285): A-42 refused when the game is built (`SetupError::TeleporterSurface`, rock too; `ADV-QUIRK-44`), the channels and the gates' pairs (`StaticMap::channels`, ADV-TEL-002; `ADV-QUIRK-45`), the shipped maps' census matched (decision 286, brought forward from T106), `build-gates-xl-144` (the pairing 91.6 % of that build; its lever in the inbox). Rounds 7 and 8 (design §17): T105 the jump and the game, T106 the fixtures and benchmarks, T107 the audit.
- **Open pull requests:** T104's (#797, branch `adventure/e08-t104`) and its benches README row (#798, `shared/e08-t104-benches`); watch them until they merge, merge `master` in on a conflict.
- **Next:** **T105** (the jump and the game, in two as design §17 plans: T105a the stream in `apply`, the jump, the Whirlpool's loss, the events and views; T105b the battle and the meeting at an exit, `Rest::Jump`, `Revisit`, the env; ready, its cut confirmed at its start).
- **Versions** (`docs/08-work-process.md`, "Versions"; each with its last change): game state hash v25 (T95b, the hire dialog; T103d's `Rest::Visit` hashes as `Rest::Guard` did), adventure state hash v21 (E09 T8a1, the town lane: `Holds::Town`), adventure ruleset part v18 (T103a: increment 11's three kinds), map import v21 (`MAP_IMPORT_VERSION`; T103b: the teleporters, 220 by the mines' table), adventure replay format v10 (T95b: `answer hire`); the env's action encoding v7 is the AI lane's (T95b, `cross-lane`); shared: core ruleset hash v52, pack format v51 (T102: `whirlpool_protection`). T104 moved none (the channels follow from the objects, out of the static map's hash). Increment 11's bumps left: design §15 (game state hash v26, replay format v11, action encoding v8 in T105b). No flat observation encoding exists for the adventure (decision 187). Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 11 (last: T102, increment 11's ADR-0016 points 1, 3 to 6 and ADR-0017 points 1, 4, 6, 8, 10, 11, with batch E11's outcome; T92 increment 10's; T81a with ADR-0013 for increment 9; T83a corrected ADR-0016 point 4's weekly objects, ADV-Q39); no draft open.
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Design notes:** the code cites each as "increment N's design note §M"; write a new citation the same way.
- **Ids:** questions `ADV-Q<n>` (next 48; none open), quirks `ADV-QUIRK-<n>` (the ledger `docs/rules/quirks-adventure.md` holds 1 to 33, 35 to 38, 41, 44 and 45, 14 and 16 withdrawn, 34 unused; 39, 40, 42 and 43 accepted for A-37, A-38, A-40 and A-41, decision 272, written by T105; next 46), stubs `ADV-STUB-<n>` (next 16; open: 8 and 9 E10's, 12 creature banks'; 13 lifted by E09 T11), rules `ADV-*` and `GAM-*` (last `GAM-BAT-014`; the family ADV-TEL: -001 written by T103a, its import by T103b, -002 by T104, -003 to -008 by their tasks, design §15), the E08 surveys' disputes `D-<n>` (next 59; D-53 to D-58 settled, decision 264) and reading quirk candidates `A-<n>` (next 44; A-37 to A-43 decided, 265–267, 272, 280); decisions next 287; batch E11 (survey §12, design §16: E11-1 to E11-19) scored (survey §15), batch E11c (E11-20 to E11-23) scored (§15.1). Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-11-09 — increment 11, round 6: T104, the build

Baseline: `master` 2e5e08f (#795; CI's fingerprint, the setup's light gates). `maps.tar` staged alone. Whole (decision 285), the census brought forward (decision 286). `state/channels.rs`: A-42's refusal (rock too, the agent's call), the lists in walk order by the tile's last object, the greedy pairing by squares; the shipped maps' census matched exactly; the gates XL map and the workload `build` (981 M Ir, the pairing 91.6 %; two loop rewrites measured worse, the lever queued). No version moved. PRs (see Current state).

### 2026-11-07 — increment 11, round 5: T103d, the guard first

Baseline: `master` cffb5c1 (#790; CI's fingerprint, the setup's light gates). No original data staged (none read). Whole. `move_hero` returns a guarded entered tile's guard visit with `visit_after`; the game holds `Rest::Visit` (tag 0, as `Rest::Guard`: no version moved) and runs `visit_held` after the guard's answer or battle and levels. Three tests rewritten, a guarded mine's added; the plans' oracles moved to the new order (two failed on random plans first); 53 replays unchanged, `rewards-stone-guard-s1` and `rewards-s88-d28` re-scripted; bot reports unchanged. PR (see Current state).

### 2026-11-07 — increment 11, round 5: T103c, batch E11c

Baseline: `master` 128e135 (#787; CI green, the setup's light gates on CI's fingerprint). The original data with the MP3s staged; Wine set up by `clients/capture/setup.sh`. Whole (decision 284). The spec `E08-batch-11c` (four rows, Red's day 1), its recipe (no new read), three runs of 87 s alike but the starting gold and a level-up's offers; scored in survey §15.1, every row as read, no question. The README row `cross-lane` (the user's consent). PR (see Current state).
