# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 and 2 closed (T10, T20; `E09-archive.md`, design notes Parts A
  to D). **Increment 3** (growth, recruitment and the army rows; decisions 64–83): the survey
  (T21, `E09-growth.md`), the drafts and Part E §37 to §58 (T22a, T22b) and the cut accepted at
  `TWN-Q13` and `TWN-Q14` (decisions 80–83); batch TWN-3 run and scored (T23, survey §17: every
  row as the reading; decision 84). **Next: T24** (`shared/`: the five drafts into the ADRs,
  `StreamId::TOWNS`, pack format v52, the glossary; in merge order with E08's T112), then the build
  T25 to T30 (design §56; T28, the rows, after the adventure lane's T114a and T115a). The Tavern is
  increment 4, the guild and the services 5, travel and the Grail 6.
  Waiting elsewhere: `TWN-STUB-8` (the battle lane's inbox), `TWN-STUB-6` and `-13` (increment 5),
  `TWN-STUB-2` (the victory conditions).
- **Open pull requests:** T23's (branch `town/e09-t23`, `cross-lane`: the capture harness, the
  battle lane's experiments README, the specs, recipes and results, the survey, the docs).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v2: T17a's day's
  mark) and the town ruleset part (`TOWN_RULESET_HASH_VERSION`, v4: T16a), both in `h3-towns`;
  the adventure lane's (game state hash, adventure state and part hashes, map import, replay
  format) and the env's and protocol's are bumped `cross-lane` (each task's record in
  `E09-log.md`); their current values are in the adventure lane's HANDOFF.
- **Ids** (next): questions `TWN-Q15`; decisions 85; quirks `TWN-QUIRK-8` (planned `-8` to `-11`:
  T-10, T-12, T-13, T-5); stubs `TWN-STUB-14` (`-11`, `-12` planned by design §34); rules planned
  TWN-GRO-001 to -007, TWN-REC-001, -002, TWN-ROW-001 to -005 (design §47, §55); experiment batch
  `TWN-4`; disputes `TD-18`; quirk candidates `T-14`. ADR-0028 is the lane's; the
  five accepted drafts wait in `adr-drafts/` for T24.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-11 — T23: batch TWN-3 run and scored

Baseline 5b2e761 (#858 merged; the setup's baseline lost to a VM restart, CI's run on `master`
stands for it). The user's computer was connected at the start only: the original data staged
then (20 files). T23 kept whole (the user, decision 84). The month's kind cannot be written ahead:
the harness forces the weekly and monthly draws (xdd.dll's new sites) and logs the garrisons' and
the Portal's; two runs of each part, every row as the reading (survey §17). `cross-lane` (the
battle lane's experiments README); the client, map-client and adventure lanes' inboxes told.
No version moved.

### 2026-11-11 — T22b: recruitment's and the rows' drafts, the cut, the checkpoint

Baseline 73e5e91 (#855, #856 merged; the light gates green). T22b whole (the user, decision 79);
the army reference was settled by the adventure lane's T108b at its start (E08 decision 305). The
user's computer was connected at the start only: the executable and both tars staged then, for
`next_player_turn`'s towns' heroes (the garrison hero's points). The drafts accepted at `TWN-Q14`
(decisions 80–83, each recommended). Docs only; no version moved. The adventure lane's inbox told
the rows' and the garrison hero's plan.

### 2026-11-11 — T22a: growth's drafts and Part E's growth sections

Baseline ea6d390 (#852, #853 merged; the setup's full baseline lost to a VM restart, CI's run on
`master` stands for it). The user's
computer was connected at the start only: the original data staged then (57 files). T22 cut in
two with batch TWN-3 its own task (the user, decision 74); the ADR-level choices before the
drafts (decisions 75–77, each recommended). Read for it: `new_month`'s head (the Imp's month
after a Deity week, no second draw), `town_change_owner`'s dwelling loop (the first creature).
Docs only; no version moved. The adventure lane's inbox told the growth's `cross-lane` plan.
