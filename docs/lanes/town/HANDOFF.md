# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 and 2 closed (T10, T20; `E09-archive.md`, design notes Parts A
  to D). **Increment 3** (growth, recruitment and the army rows; decisions 64–87): the survey
  (T21), the drafts and Part E §37 to §58 (T22a, T22b), the cut (`TWN-Q13`, `TWN-Q14`), batch
  TWN-3 (T23), the `shared/` task (T24), T25a (decision 86: the part v5, the store v3,
  TWN-GRO-001, -005, `TWN-QUIRK-8`), **T25b1** (decision 87, T25b in two: `WeekKind`,
  TWN-GRO-003, -004, `TWN-QUIRK-9`, `-10`). **Next: T25b2** (TWN-GRO-006, -007 as pure functions
  over the draws given), then T26 to T30 (design §56; T28, the rows, after the adventure lane's
  T114a and T115a). The Tavern is increment 4, the guild and the services 5, travel and the
  Grail 6.
  Waiting elsewhere: `TWN-STUB-8` (the battle lane's inbox), `TWN-STUB-6` and `-13` (increment 5),
  `TWN-STUB-2` (the victory conditions).
- **Open pull requests:** T25b1's (branch `town/e09-t25b1`, the lane's paths only); watch it
  until it merges.
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
- **Ids** (next): questions `TWN-Q15`; decisions 88; quirks `TWN-QUIRK-11` (planned `-11`: T-5);
  stubs `TWN-STUB-14` (`-11`, `-12` planned by design §34); rules planned TWN-GRO-002, -006,
  -007, TWN-REC-001, -002, TWN-ROW-001 to -005 (design §47, §55);
  experiment batch `TWN-4`; disputes `TD-18`; quirk candidates `T-14`. ADR-0028 is the lane's;
  no drafts wait.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-12 — T25b1: the week's kind, a town's week and month of growth

Baseline a396c3f (#873 merged; the setup's baseline found CI's fingerprint, the light gates).
T25b cut in two (the user, decision 87; recommended). The user's computer was connected at the
start only: the original data staged then (17 files), used for the scoped verify. The lane's
paths only: `h3-towns`' `week.rs` (`WeekKind`, `week_growth`, `MonthGrowth`, `month`), the
catalog's TWN-GRO-003, -004, `TWN-QUIRK-9`, `-10`. No version moved. T25b2 queued.

### 2026-11-12 — T25a: the part v5, the store v3, a dwelling's growth and first creatures

Baseline 4a70238 (#864, #865 merged; the setup's baseline found CI's fingerprint, the light
gates). T25a kept whole (the user, decision 86; two parts recommended). The user's computer was
connected at the start only: the original data staged then (17 files), used for the re-harvest
and the scoped verify. `cross-lane`: the harvester and the re-harvested `base` and `ab` packs
(the 22 horde roles, the Portal, the Deity, Conflux's `roe`, the parameters), game state hash
v28, the 72 adventure replays (hash lines only), one `h3-game` test literal. Part v5, store v3,
TWN-GRO-001, -005, `TWN-QUIRK-8`. The adventure lane's inbox and versions line told.

### 2026-11-12 — T24: increment 3's `shared/` task (the ADRs, the stream, the effects, the flags)

Baseline 8679604 (#860 merged; CI's green run on `master` stood for the setup's baseline). T24
kept whole (the user, decision 85). The user's computer was connected at the start only: the
original data staged then (21 files), used for the re-harvest and the scoped verify. Branch
`shared/e09-t24`, `cross-lane`: `StreamId::TOWNS`, `town_growth` and `owner_town_growth` with the
packs' six Legion artifacts, the two `quirk.town` flags, core ruleset hash v53; the 82 golden
replays' `ruleset_hash` and the bot reports' `engine.core` only; the five drafts into the ADRs
with T23's verdicts; the glossary; `docs/06-security.md`; the adventure lane's inbox (T112's
order). T25 queued.
