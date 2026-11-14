# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 and 2 closed (T10, T20; `E09-archive.md`, design notes Parts A
  to D). **Increment 3** (growth, recruitment and the army rows; decisions 64–94): the survey
  (T21), the drafts and Part E §37 to §58 (T22a, T22b), the cut (`TWN-Q13`, `TWN-Q14`), batch
  TWN-3 (T23), the `shared/` task (T24), growth T25a to T26b (decisions 86 to 91), T27
  (decision 92: recruitment), T28a (decision 93: the bars, the garrison hero's record and moves;
  `TWN-STUB-14`), **T28b1** (decision 94: T28b in two; the garrison hero's day, income and
  growth's inputs, TWN-ROW-005; no version moved). **Next:** T28b2 (the visits of another team,
  T-5, `TWN-STUB-14` lifted, level 3's view, `QUEUE.md`), then T29, T30 (design §56). The Tavern
  is increment 4, the guild and the services 5, travel and the Grail 6. Waiting elsewhere:
  `TWN-STUB-8` (the battle lane's inbox), `TWN-STUB-6` and `-13` (increment 5), `TWN-STUB-2` (the
  victory conditions).
- **Open pull requests:** T28b1's (branch `town/e09-t28b1`, `cross-lane`); watch it until it
  merges. Open beside it: the server lane's #922 (protocol v65).
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
- **Ids** (next): questions `TWN-Q15`; decisions 95; quirks `TWN-QUIRK-11` (planned: T-5, T28b2);
  stubs `TWN-STUB-15` (`-11`, `-12` planned by design §34); rules: design §55's all built (TWN-ROW-005
  by T28b1); experiment batch `TWN-4`; disputes `TD-18`; quirk candidates `T-14`. ADR-0028 is the
  lane's; no drafts wait.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-14 — T28b1: a garrison hero's day, income and growth's inputs

Baseline 29f658e (#920 merged; CI's fingerprint, the light gates). T28b cut in two, T28b1 now
(the user, decision 94; three parts were recommended). The user's computer was connected at the
start: the original data and the four manuals staged then (27 files, SHA-256 checked), used for the
scoped verify. `cross-lane` (the adventure lane's `state.rs`' `start_turn`, `state/bonuses.rs`'
drops, `state/economy.rs`' income, `h3-game`'s `pay_income`, `h3-map`'s plans): no version moved,
no replay changed. A gap found: the day's and the week's mark drops skipped the garrison heroes
(`change_marks` admits only heroes on the map). Note in the adventure lane's inbox. T28b2 queued.

### 2026-11-14 — T28a: the town's two bars and the garrison hero

Baseline 50b0935 (#911 merged; CI's fingerprint, the light gates). T28 cut in two, T28a whole
(the user, decision 93; the bars first were recommended). The user's computer was connected at the
start only: the original data and the manuals staged then (SHA-256 checked), used for the scoped
verify. `master` merged in after #913 (protocol v60), after #914, #916 and #917 (the replays
re-recorded, `fork-xl-144` re-measured), after #918 and #919. `cross-lane`: `h3-adventure`'s state,
army commands, hire, events and observation; `h3-game`'s management, listing and observation;
`h3-map`'s plans; `h3-files`; `h3-harvest`'s gap-fill; `h3-protocol` (v64), `h3-server`'s
literals; `h3-sim` (v11; the fork test on seed 70); the 86 replays; five workloads re-baselined.
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
