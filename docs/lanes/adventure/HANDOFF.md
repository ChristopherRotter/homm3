# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 6 **closed**. **Increment 7: rewards on the map** (decisions 121–150, ADV-Q26 to ADV-Q30): T54 to T58b done (the battle seam's facts; survey `E08-rewards.md`; the drafts and design `E08-design.md`; batch E7 scored, A-23 withdrawn; A-25 a quirk with a flag); T59 done (#497, `shared/`); T60 done (T60a #500, T60b #504); T61 done (T61a #507, T61b #511); T62 in two (decision 150): **T62a** (`adventure/e08-t62a`: the pickups, teachers and finds; the game applies the hero's gains, the choices' defaults until T63, `ADV-STUB-10`). Records: `E08-log.md`; increment 6's in `E08-archive.md`.
- **Open pull requests:** T62a's `adventure/e08-t62a` (see the newest session).
- **Next:** merge T62a's pull request; then T62b (the record, boxes, events and the move, the Pyramid; `QUEUE.md`), its cut confirmed at its start. Then T63 (`h3-game`'s typed answers and decisions; closes `ADV-STUB-10`).
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v16 (T62a), adventure state hash v14 (T61b), adventure ruleset part v12 (T61a), map import v13 (`MAP_IMPORT_VERSION`, T61b), adventure replay format v6 (T51a); the env's action encoding v4 is the AI lane's. Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 7 (last: T59, with ADR-0013).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ids:** questions `ADV-Q<n>` (next 31), quirks `ADV-QUIRK-<n>` (1 the fringe; 2, 3 the diagonal; 4 neutral stacks; 5, 6 obstacles, split; 7 A-6; 8 formation; 19, 20 A-15, A-16 (no flags); flags 19–24; 15, 17, 18 increment 5's, flags 31, 32; 14, 16 withdrawn, flag 30 retired; 21, 22 A-21, A-25, flags 37, 38; 23 to 27 A-18, A-20, A-22, A-24, A-17, no flags; next 28), stubs `ADV-STUB-<n>` (next 11; 8 and 9 open, E10's; 10 open, T63's), rules `ADV-*` and `GAM-*` (last `GAM-BAT-013`; `HER-CHG-003`, `-004` built by T60b; `GAM-SET-003`, `ADV-OBJ-006`, `ADV-PIK-005`, `-009` to `-012`, `ADV-TCH-003` to `-008` by T61b; `ADV-PIK-006` to `-008`, `ADV-MON-011`, `ADV-TCH-001`, `-002`, `ADV-VIS-006` by T62a; the rest of increment 7's in survey §12 and design §17); the E08 surveys' disputes `D-<n>` (next 41) and reading quirk candidates `A-<n>` (next 26; A-23 withdrawn); decisions next 151. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-10-27 — increment 7, round 6: T62a, the pickups, the teachers and the finds

Baseline 54b9264 (#511 merged, CI green; the sandbox's light gates passed; the original data staged from the user's computer). The user's cut (decision 150): T62 in two, T62a the pickups, teachers and finds; `h3-game` applies the simple parts now. `h3-adventure`: `state/rewards.rs` (the visits; `HeroGains`, `Choice`/`Pick`/`choose`, `UniversityVisit`, `OfferKind::Gift`, `Search`), a monster's treasure artifact, the views' `known` and `OwnHeroView::marks`; `h3-game`: `give_hero`, `choose_by_default` (`ADV-STUB-10`), game state hash v16; seven rules new, `ADV-QUIRK-27` (A-17); the twelve replays re-recorded. `docs/06-security.md`'s line on the viewer's knowledge waits for the next `shared/` change (T62b's queue entry). Pull request: see "Current state".

### 2026-10-27 — increment 7, round 5: T61b, the bans, the draws and the state

Baseline c94a5ab (#507 merged, CI green; the sandbox's light gates passed; the original data staged from the user's computer). The census first: nothing broke, 8 maps import whole. The user's cut: whole (decision 149); `cross-lane` (consented): the bans' two default lines in the AI lane's `census.rs`. `h3-game`/`h3-map`: GAM-SET-003 (map import v13; the SoD editor's default artifact bans 141 to 143 ban nothing). `h3-adventure`: `state/draws.rs` (`Resolved`, twelve rules), `persistent::Table` for the army left, `state/visits.rs` (`Visits`, none until T62), adventure state hash v14; `ADV-QUIRK-23` to `-26`; the twelve replays re-recorded. The scoped verify with the original data passed but `check-lane` (the label). Pull request: see "Current state".

### 2026-10-27 — increment 7, round 5: T61a, the import and the kinds

Baseline 94a5039 (#504 merged, CI green; the sandbox's light gates passed). The user's cut: T61 in two as designed (decision 148). `h3-adventure`: `ruleset/rewards.rs` and twelve kinds (ruleset part v12), `Holds`/`RewardRecord`/`EventSetup` on the setup, `state/holds.rs` (`SetupError::Holds`), guards on more kinds, `Monster::artifact` (state hash v13); `h3-map`: the 31 classes and their bodies (import v12, A-24); `h3_heroes::validate_artifact`; the twelve replays re-recorded; the AI lane's inbox (the census's guarded piles); `cross-lane` (consented): three default lines in the AI lane's `h3-sim` adventure tests. Not run: `--with-original-data` (the user's computer was not reachable); the bans moved to T61b.
