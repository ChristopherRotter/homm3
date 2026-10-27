# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 6 **closed**. **Increment 7: rewards on the map** (decisions 121–149, ADV-Q26 to ADV-Q30): T54 to T58b done (the battle seam's facts; survey `E08-rewards.md`; the drafts and design `E08-design.md`; batch E7 scored, A-23 withdrawn; A-25 a quirk with a flag); T59 done (#497, `shared/`); T60 done (T60a #500, T60b #504); T61 done in two (decision 148): T61a #507, **T61b** (`adventure/e08-t61b`, decision 149, whole: the census, 8 maps whole; the bans; the draws, `Resolved`; the army left's table; `Visits`). Records: `E08-log.md`; increment 6's in `E08-archive.md`.
- **Open pull requests:** T61b's `adventure/e08-t61b` (`cross-lane`; see the newest session).
- **Next:** merge T61b's pull request; then T62 (the visits, events and the move, information; design §11 to §13), its cut confirmed at its start (likely two).
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v15 (T60a), adventure state hash v14 (T61b), adventure ruleset part v12 (T61a), map import v13 (`MAP_IMPORT_VERSION`, T61b), adventure replay format v6 (T51a); the env's action encoding v4 is the AI lane's. Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 7 (last: T59, with ADR-0013).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ids:** questions `ADV-Q<n>` (next 31), quirks `ADV-QUIRK-<n>` (1 the fringe; 2, 3 the diagonal; 4 neutral stacks; 5, 6 obstacles, split; 7 A-6; 8 formation; 19, 20 A-15, A-16 (no flags); flags 19–24; 15, 17, 18 increment 5's, flags 31, 32; 14, 16 withdrawn, flag 30 retired; 21, 22 A-21, A-25, flags 37, 38; 23 to 26 A-18, A-20, A-22, A-24, no flags; next 27), stubs `ADV-STUB-<n>` (next 10; 8 and 9 open, E10's), rules `ADV-*` and `GAM-*` (last `GAM-BAT-013`; `HER-CHG-003`, `-004` built by T60b; `GAM-SET-003`, `ADV-OBJ-006`, `ADV-PIK-005`, `-009` to `-012`, `ADV-TCH-003` to `-008` by T61b; the rest of increment 7's in survey §12 and design §17); the E08 surveys' disputes `D-<n>` (next 41) and reading quirk candidates `A-<n>` (next 26; A-23 withdrawn); decisions next 150. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-10-27 — increment 7, round 5: T61b, the bans, the draws and the state

Baseline c94a5ab (#507 merged, CI green; the sandbox's light gates passed; the original data staged from the user's computer). The census first: nothing broke, 8 maps import whole. The user's cut: whole (decision 149); `cross-lane` (consented): the bans' default lines in the AI lane's `census.rs` and `ai/tests.rs`. `h3-game`/`h3-map`: GAM-SET-003 (map import v13; the SoD editor's default artifact bans 141 to 143 ban nothing). `h3-adventure`: `state/draws.rs` (`Resolved`, twelve rules), `persistent::Table` for the army left, `state/visits.rs` (`Visits`, none until T62), adventure state hash v14; `ADV-QUIRK-23` to `-26`; the twelve replays re-recorded. The scoped verify with the original data passed but `check-lane` (the label). Pull request: see "Current state".

### 2026-10-27 — increment 7, round 5: T61a, the import and the kinds

Baseline 94a5039 (#504 merged, CI green; the sandbox's light gates passed). The user's cut: T61 in two as designed (decision 148). `h3-adventure`: `ruleset/rewards.rs` and twelve kinds (ruleset part v12), `Holds`/`RewardRecord`/`EventSetup` on the setup, `state/holds.rs` (`SetupError::Holds`), guards on more kinds, `Monster::artifact` (state hash v13); `h3-map`: the 31 classes and their bodies (import v12, A-24); `h3_heroes::validate_artifact`; the twelve replays re-recorded; the AI lane's inbox (the census's guarded piles); `cross-lane` (consented): three default lines in the AI lane's `h3-sim` adventure tests. Not run: `--with-original-data` (the user's computer was not reachable); the bans moved to T61b.

### 2026-10-27 — increment 7, round 4: T60b, `grant` and the teachers

Baseline c637652 (#500 merged as 40151d3, CI green on it; the sandbox's light gates passed). T60b whole (decision 147; no split needed). `h3-heroes`: `reward.rs` (`grant`, `HeroReward`, `Granted`, `can_receive_artifact` under flag 38), `teach.rs` (the teachers, `SlotState`, `ScholarGift`), `can_learn_spell` beside `learns_spell` (which delegates; `h3-game` untouched); HER-CHG-003, HER-CHG-004 (new id); `ADV-QUIRK-22`'s test in both profiles; T61 queued.
