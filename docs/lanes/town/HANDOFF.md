# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 and 2 closed (T10, T20; `E09-archive.md`, design notes Parts A
  to D). **Increment 3** (growth, recruitment and the army rows; decisions 64–93): the survey
  (T21), the drafts and Part E §37 to §58 (T22a, T22b), the cut (`TWN-Q13`, `TWN-Q14`), batch
  TWN-3 (T23), the `shared/` task (T24), growth T25a to T26b (decisions 86 to 91), T27
  (decision 92: recruitment), **T28a** (decision 93, whole: the town's two bars TWN-ROW-001, the
  garrison hero's record and `TownCommand::Heroes` `Enter`, `Leave`, `Swap` TWN-ROW-002 to -004,
  the hire into its army; town part v6, adventure state hash v25, protocol v64, replay format
  v16, the env's v11; `TWN-STUB-14`: a garrison hero does not defend yet). **Next:** T28b (what a
  garrison hero changes, `QUEUE.md`), then T29, T30 (design §56). The Tavern is increment 4, the
  guild and the services 5, travel and the Grail 6. Waiting elsewhere: `TWN-STUB-8` (the battle
  lane's inbox), `TWN-STUB-6` and `-13` (increment 5), `TWN-STUB-2` (the victory conditions).
- **Open pull requests:** #920, T28a's (branch `town/e09-t28a`, `cross-lane`); watch it until it
  merges. Its numbers come after #914's and #916's (merged) and #919's protocol v63 (open beside
  it). The adventure lane's #918 (open) adds 14 replays at format 15: whichever merges second
  re-records the other's (`homm3-rerecord-replays`).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v3: T25a's counts
  and Portal) and the town ruleset part (`TOWN_RULESET_HASH_VERSION`, v6: T28a's `most_heroes`),
  both in `h3-towns`; the adventure lane's (game state hash v32, adventure state hash v25 by T28a,
  map import v23, replay format v16 by T28a), the env's (v11 by T28a) and the protocol's (v64 by
  T28a) are bumped `cross-lane` (each task's record in `E09-log.md`). Shared, from T24: core
  ruleset hash v53, pack format v52, quirk flags 41, `StreamId::TOWNS` (7).
- **Ids** (next): questions `TWN-Q15`; decisions 94; quirks `TWN-QUIRK-11` (planned: T-5, T28b);
  stubs `TWN-STUB-15` (`-11`, `-12` planned by design §34); rules planned TWN-ROW-005 (design
  §55); experiment batch `TWN-4`; disputes `TD-18`; quirk candidates `T-14`. ADR-0028 is the
  lane's; no drafts wait.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-14 — T28a: the town's two bars and the garrison hero

Baseline 50b0935 (#911 merged; CI's fingerprint, the light gates). T28 cut in two, T28a whole
(the user, decision 93; the bars first were recommended). The user's computer was connected at the
start only: the original data and the manuals staged then (SHA-256 checked), used for the scoped
verify. `master` merged in after #913 (protocol v60), then after #914, #916 and #917 (the replays
re-recorded, `fork-xl-144` re-measured). `cross-lane`: `h3-adventure`'s state,
army commands, hire, events and observation; `h3-game`'s management, listing and observation;
`h3-map`'s plans; `h3-files`; `h3-harvest`'s gap-fill; `h3-protocol` (v64), `h3-server`'s
literals; `h3-sim` (v11; the fork test on seed 70); the 72 replays; five workloads re-baselined.
Notes in the adventure's, the server's, the AI and the client lane's inboxes. T28b queued. PR #920.

### 2026-11-13 — T27: recruitment

Baseline 202aa3f (#903 merged; the setup's baseline lost to a VM restart, CI's green run on
`master` 999bb0e stands for it). T27 whole (the user, decision 92; two parts were recommended).
The user's computer was connected at the start only: the original data staged then (17 files,
SHA-256 checked), used for the scoped verify. `master` merged in after #904 (protocol v56,
replay format v14), #906 (game state hash v31, protocol v57) and #907 (protocol v58).
`cross-lane`: `h3-adventure`'s hire types, `h3-game`'s hire
decision, hash and views, `h3-map`'s plans, `h3-files`, `h3-protocol` (v59), `h3-server`'s
record literals, `h3-sim`'s `RECRUIT` (v10; the town map's seeds 70 and 46), the 72 replays,
`legal-towns-36` and `reach-towns-36` re-baselined (the agent's recruitments). Notes in the
adventure's, the server's and the AI lane's inboxes. T28 queued. PR #911.

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
