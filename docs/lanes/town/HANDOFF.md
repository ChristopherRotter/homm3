# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 and 2 closed (T10, T20; `E09-archive.md`, design notes Parts A
  to D). **Increment 3** (growth, recruitment and the army rows; decisions 64–83): the survey
  (T21, `E09-growth.md`), the drafts and Part E §37 to §58 (T22a, T22b) and the cut accepted at
  `TWN-Q13` and `TWN-Q14` (the rows without a decision, the garrison hero's record in the town, a
  capture's empty Portal drawn; decisions 80–83). **Next: T23**, batch TWN-3 (survey §14, six rows;
  needs the original data), then T24 (`shared/`) and the build T25 to T30 (design §56; T28, the
  rows, after the adventure lane's T114a and T115a). The Tavern is increment 4, the guild and the
  services 5, travel and the Grail 6.
  Waiting elsewhere: `TWN-STUB-8` (the battle lane's inbox), `TWN-STUB-6` and `-13` (increment 5),
  `TWN-STUB-2` (the victory conditions).
- **Open pull requests:** T22b's (branch `town/e09-t22b`, docs only: the drafts, the design note,
  the epic, the log, the answered question, the lane's queue and the adventure lane's inbox).
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
- **Ids** (next): questions `TWN-Q15`; decisions 84; quirks `TWN-QUIRK-8` (planned `-8` to `-11`:
  T-10, T-12, T-13, T-5); stubs `TWN-STUB-14` (`-11`, `-12` planned by design §34); rules planned
  TWN-GRO-001 to -007, TWN-REC-001, -002, TWN-ROW-001 to -005 (design §47, §55); experiment batch
  `TWN-4` (TWN-3 to run); disputes `TD-18`; quirk candidates `T-14`. ADR-0028 is the lane's; the
  five accepted drafts wait in `adr-drafts/` for T24.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

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

### 2026-11-11 — T21: increment 3's survey and its checkpoint

Baseline d3bf756 (#846 to #850 merged; the setup's baseline the light gates, CI's fingerprint).
The user's computer was connected only at the start: the original data staged then (57 files).
The cut measured (growth 3.5 to 5.5 KB unread, recruitment and the rows 2 to 3 KB); the agent
recommended two parts, the user kept T21 whole (decision 68). The survey `E09-growth.md` (code
reading, VCMI 1.7.5, the wiki, the census by a throwaway function in `original_data`, not
committed); 22 new address map entries; the checkpoint `TWN-Q12` (decisions 69–73); the
adventure lane's inbox told the army reference proposal. Docs only; no version moved.
