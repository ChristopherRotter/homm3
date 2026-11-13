# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 and 2 closed (T10, T20; `E09-archive.md`, design notes Parts A
  to D). **Increment 3** (growth, recruitment and the army rows; decisions 64–91): the survey
  (T21), the drafts and Part E §37 to §58 (T22a, T22b), the cut (`TWN-Q13`, `TWN-Q14`), batch
  TWN-3 (T23), the `shared/` task (T24), T25a (decision 86: the part v5, the store v3,
  TWN-GRO-001, -005, `TWN-QUIRK-8`), T25b1 (decision 87: `WeekKind`, TWN-GRO-003, -004,
  `TWN-QUIRK-9`, `-10`), T25b2 (decision 88: `garrison_week`, `summon`, TWN-GRO-006, -007),
  T26a1 (decision 89: the inputs, the week and the month in the day, the events' creatures,
  `TWN-STUB-10` lifted), T26a2 (decision 90: the week's kind, game state hash v30, map import
  v23, protocol v53), T26b (decision 91: the neutral garrison week, the Portal's draws, the
  owner's counts and Portal), **T27** (decision 92, whole: recruitment, TWN-REC-001, -002; game
  state hash v32, protocol v58, replay format v15, the env's v10). **Next:** T28, the rows
  (`QUEUE.md`; E08 T115a, #906, merged), then T29, T30 (design §56). The Tavern is
  increment 4, the guild and the services 5, travel and the Grail 6.
  Waiting elsewhere: `TWN-STUB-8` (the battle lane's inbox), `TWN-STUB-6` and `-13` (increment 5),
  `TWN-STUB-2` (the victory conditions).
- **Open pull requests:** T27's (branch `town/e09-t27`, `cross-lane`; its number in the latest
  session entry); watch it until it merges.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v3: T25a's counts
  and Portal) and the town ruleset part (`TOWN_RULESET_HASH_VERSION`, v5: T25a's roles, `roe`,
  parameters), both in `h3-towns`; the adventure lane's (game state hash v30 and map import v23 by
  T26a2, adventure state and part hashes, replay format) and the env's and protocol's are bumped
  `cross-lane` (each task's record in `E09-log.md`); their current values are in the adventure
  lane's HANDOFF; T27 took game state hash v32, protocol v58, replay format v15 and the env's
  action encoding v10. Shared, from T24: core ruleset hash v53, pack format v52, quirk flags 41
  (`quirk.town.creature_month_exact` 39, `quirk.town.neutral_plague_full_growth` 40, read by
  T25b1's `month`), `StreamId::TOWNS` (7).
- **Ids** (next): questions `TWN-Q15`; decisions 93; quirks `TWN-QUIRK-11` (planned `-11`: T-5);
  stubs `TWN-STUB-14` (`-11`, `-12` planned by design §34); rules planned TWN-ROW-001 to -005
  (design §55);
  experiment batch `TWN-4`; disputes `TD-18`; quirk candidates `T-14`. ADR-0028 is the lane's;
  no drafts wait.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-13 — T27: recruitment

Baseline 202aa3f (#903 merged; the setup's baseline lost to a VM restart, CI's green run on
`master` 999bb0e stands for it). T27 whole (the user, decision 92; two parts were recommended).
The user's computer was connected at the start only: the original data staged then (17 files,
SHA-256 checked), used for the scoped verify. `master` merged in after #904 (protocol v56,
replay format v14) and #906 (game state hash v31, protocol v57). `cross-lane`: `h3-adventure`'s hire types, `h3-game`'s hire
decision, hash and views, `h3-map`'s plans, `h3-files`, `h3-protocol` (v58), `h3-server`'s
record literals, `h3-sim`'s `RECRUIT` (v10; the town map's seeds 70 and 46), the 72 replays,
`legal-towns-36` and `reach-towns-36` re-baselined (the agent's recruitments). Notes in the
adventure's, the server's and the AI lane's inboxes. T28 queued.

### 2026-11-13 — T26b: the neutral garrison week, the Portal's draws, the owner's view

Baseline effc66d (#899 merged; the setup found CI's fingerprint, the light gates). T26b whole (the
user, decision 91; two parts were recommended). The user's computer was connected at the start
only: the original data staged then (17 files, SHA-256 checked), used for the scoped verify.
`cross-lane`: `h3-adventure`'s outcome (`flagged_dwelling`) and dwelling visit, `h3-game`'s state
and day, `h3-map`'s plans (the hygiene property ends at every week's start), four towns replays
(hash lines), `day-start-towns-36` re-baselined (+8.10 % against `master`, the garrison week;
`day-start-builds-36` +1.60 % on T114b's baseline), `h3-protocol` (v55, after T114b's v54). Notes in the adventure's and the
server's inboxes. T27 queued. PR #903 (`master` merged in after the server lane's #901 and
E08 T114b's #902, which took v54 first).

### 2026-11-12 — T26a2: the week's kind

Baseline 7f9107f (#892 to #894 merged; the setup's full baseline stopped in coverage for the
disk, CI's run on `master` stands for it). T26a2 whole (the user, decision 90; the engine-now cut
was recommended). The user's computer was connected at the start only: the original data staged
then (17 files, SHA-256 checked), used for the scoped verify. First the inbox's hygiene note,
classified: the property plan's fault (a visited town drew a dwelling its owner sees), fixed in
`h3-map`'s `view.rs`. `cross-lane`: game state hash v30, map import v23, protocol v53 (the server's T11, T12 and the adventure's T114a2 took v50 to v52), the 72
adventure replays, four fork workloads re-baselined (+0.8 % to +1.1 % against `master`),
`h3-server`'s record literals, a `h3-view` test literal. Notes in the adventure's and the
server's inboxes. T26b next.
