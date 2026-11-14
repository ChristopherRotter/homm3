# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 and 2 closed (T10, T20; `E09-archive.md`, design notes Parts A
  to D). **Increment 3** (growth, recruitment and the army rows; decisions 64–95): the survey
  (T21), the drafts and Part E §37 to §58 (T22a, T22b), the cut (`TWN-Q13`, `TWN-Q14`), batch
  TWN-3 (T23), the `shared/` task (T24), growth T25a to T26b (decisions 86 to 91), T27
  (decision 92: recruitment), T28a (decision 93: the bars, the garrison hero's record and moves),
  T28b1 (decision 94: its day, income and growth's inputs), **T28b2** (decision 95, whole: rows 1
  and 3 of a visit of another team, T-5 as `TWN-QUIRK-11`, `TWN-STUB-14` lifted, the town's views
  with the garrison hero, protocol v67). **Next:** T29, the gates (design §56 round 8,
  `QUEUE.md`), then T30 (the audit, the closure, increment 4 refined). The Tavern is increment 4,
  the guild and the services 5, travel and the Grail 6. Waiting elsewhere: `TWN-STUB-8` (the
  battle lane's inbox), `TWN-STUB-6` and `-13` (increment 5), `TWN-STUB-2` (the victory
  conditions).
- **Open pull requests:** #930, T28b2's (branch `town/e09-t28b2`, `cross-lane`); watch it until it
  merges. A failing case of the adventure lane's hygiene property on `master` (their inbox, not
  in `proptest-regressions`): if CI meets it on a town pull request, it is not the town lane's.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v3: T25a's counts
  and Portal) and the town ruleset part (`TOWN_RULESET_HASH_VERSION`, v6: T28a's `most_heroes`),
  both in `h3-towns`; the adventure lane's (game state hash v32, adventure state hash v25 by T28a,
  map import v23, replay format v16 by T28a), the env's (v11 by T28a) and the protocol's (v67 by
  T28b2) are bumped `cross-lane` (each task's record in `E09-log.md`). Shared, from T24: core
  ruleset hash v53, pack format v52, quirk flags 41, `StreamId::TOWNS` (7).
- **Ids** (next): questions `TWN-Q15`; decisions 96; quirks `TWN-QUIRK-12`; stubs `TWN-STUB-15`
  (`-11`, `-12` planned by design §34); rules: design §55's all built; experiment batch `TWN-4`;
  disputes `TD-18`; quirk candidates `T-14`. ADR-0028 is the lane's; no drafts wait.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-14 — T28b2: a garrison hero defends; T-5; the town's views

Baseline 0872fc6 (#924 merged; the setup's baseline lost to a VM restart, CI's green run on
`master` stands for it). T28b2 whole (the user, decision 95; two parts were recommended). The
user's computer was connected at the start: the original data and the four manuals staged then
(28 files, SHA-256 checked), used for the scoped verify and a third reading of `town_defended`.
`cross-lane`: `h3-adventure`'s `end_battle`, `raise`, `change_marks`; `h3-game`'s battle end and
`town_told`; `h3-map`'s view property; `h3-protocol` (v67). A hygiene case found failing on
`master` too, left in the adventure lane's inbox with its seed. Notes in the adventure's and the
server's inboxes. T29 next. PR #930.

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
