# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 and 2 closed (T10, T20; `E09-archive.md`, design notes Parts A
  to D). **Increment 3** (growth, recruitment and the army rows; decisions 64–88): the survey
  (T21), the drafts and Part E §37 to §58 (T22a, T22b), the cut (`TWN-Q13`, `TWN-Q14`), batch
  TWN-3 (T23), the `shared/` task (T24), T25a (decision 86: the part v5, the store v3,
  TWN-GRO-001, -005, `TWN-QUIRK-8`), T25b1 (decision 87: `WeekKind`, TWN-GRO-003, -004,
  `TWN-QUIRK-9`, `-10`), T25b2 (decision 88: `garrison_week`, `summon`, TWN-GRO-006, -007),
  **T26a1** (decision 89: T26a in two; the inputs, the week and the month in the day, the events'
  creatures, `TWN-STUB-10` lifted). **Next: T26a2** (the week's kind, `cross-lane`; map import
  v22 or the next free after E08's T113b), then T26b (`QUEUE.md`), then T27 to T30
  (design §56; T28, the rows, after the adventure lane's T114a and T115a). The Tavern is
  increment 4, the guild and the services 5, travel and the Grail 6.
  Waiting elsewhere: `TWN-STUB-8` (the battle lane's inbox), `TWN-STUB-6` and `-13` (increment 5),
  `TWN-STUB-2` (the victory conditions).
- **Open pull requests:** T26a1's (branch `town/e09-t26a1`, `cross-lane`) and its `shared/` one
  (#886, `shared/e09-t26a1-quirk-marks`, comments only; merges first); watch both until they
  merge.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v3: T25a's counts
  and Portal) and the town ruleset part (`TOWN_RULESET_HASH_VERSION`, v5: T25a's roles, `roe`,
  parameters), both in `h3-towns`; the adventure lane's (game state hash v28 by T25a, adventure
  state and part hashes, map import, replay format) and the env's and protocol's are bumped
  `cross-lane` (each task's record in `E09-log.md`); their current values are in the adventure
  lane's HANDOFF. Shared, from T24: core ruleset hash v53, pack format v52, quirk flags 41
  (`quirk.town.creature_month_exact` 39, `quirk.town.neutral_plague_full_growth` 40, read by
  T25b1's `month`), `StreamId::TOWNS` (7).
- **Ids** (next): questions `TWN-Q15`; decisions 90; quirks `TWN-QUIRK-11` (planned `-11`: T-5);
  stubs `TWN-STUB-14` (`-11`, `-12` planned by design §34); rules planned TWN-GRO-002, TWN-REC-001,
  -002, TWN-ROW-001 to -005 (design §47, §55);
  experiment batch `TWN-4`; disputes `TD-18`; quirk candidates `T-14`. ADR-0028 is the lane's;
  no drafts wait.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-12 — T26a1: the towns' growth in the day

Baseline 4724477 (#883 merged; the setup's baseline found CI's fingerprint, the light gates). T26a
cut in two (the user, decision 89; recommended). The user's computer was connected at the start
only: the original data staged then (17 files), used for the scoped verify. `cross-lane`:
`h3-heroes`' Legion effects, `h3-game`'s day order, nine adventure replays (hash lines only),
`game.toml`, comments in `h3-map`, `h3-cli`, the harvester and `packs/base/core.toml`; `shared/`
#886 (two flag docs). No version moved; the bench inside its gates. T26a2 and T26b queued.

### 2026-11-12 — T25b2: a neutral town's garrison week and the Portal's draw

Baseline 21d6ba0 (#882 merged; the setup's local baseline stopped as `master` moved past it, CI's
run on `master` stands for it). T25b2 kept whole (the user, decision 88; recommended). The
user's computer was connected at the start only: the original data staged then (17 files), used
for the scoped verify. The lane's paths only: `h3-towns`' `garrison.rs` (`garrison_week`,
`GarrisonWeek`) and `portal.rs` (`summon`), the fixture's AI values, the catalog's TWN-GRO-006,
-007. No version moved. T26 queued in full.

### 2026-11-12 — T25b1: the week's kind, a town's week and month of growth

Baseline a396c3f (#873 merged; the setup's baseline found CI's fingerprint, the light gates).
T25b cut in two (the user, decision 87; recommended). The user's computer was connected at the
start only: the original data staged then (17 files), used for the scoped verify. The lane's
paths only: `h3-towns`' `week.rs` (`WeekKind`, `week_growth`, `MonthGrowth`, `month`), the
catalog's TWN-GRO-003, -004, `TWN-QUIRK-9`, `-10`. No version moved. T25b2 queued.
