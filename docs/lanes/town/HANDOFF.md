# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 and 2 closed (T10, T20; `E09-archive.md`, design notes Parts A
  to D). **Increment 3** (growth, recruitment and the army rows; decisions 64–97): the survey
  (T21), the drafts and Part E §37 to §58 (T22a, T22b), the cut (`TWN-Q13`, `TWN-Q14`), batch
  TWN-3 (T23), the `shared/` task (T24), growth T25a to T26b (decisions 86 to 91), T27
  (decision 92: recruitment), T28a to T28b2 (decisions 93 to 95: the rows), T29a (decision 96:
  the builds map's replays, the census to day 29), **T29b** (decision 97, whole: the properties,
  the plans' hero moves, `month-start-builds-36`). **Next:** T30 (the audit against §57's exit
  criteria, the closure, increment 4 refined; `QUEUE.md`). The Tavern is increment 4, the guild
  and the services 5, travel and the Grail 6. Waiting elsewhere: `TWN-STUB-8` (the battle lane's
  inbox), `TWN-STUB-6` and `-13` (increment 5), `TWN-STUB-2` (the victory conditions).
- **Open pull requests:** #942 (`shared/e09-t29b-benches`, the benches README's row; merges
  first) and #946 (T29b's, `town/e09-t29b`, `cross-lane`); watch both until they merge.
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
- **Ids** (next): questions `TWN-Q15`; decisions 98; quirks `TWN-QUIRK-12`; stubs `TWN-STUB-15`
  (`-11`, `-12` planned by design §34); rules: design §55's all built; experiment batch `TWN-4`;
  disputes `TD-18`; quirk candidates `T-14`. ADR-0028 is the lane's; no drafts wait.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-15 — T29b: the properties, the plans' hero moves, the month workload

Baseline 8026f16 (#934 to #937 merged; the setup's full baseline passed but coverage, whose test
run met a hygiene case of `master`'s: a map garrison's entrance flooded by `vary`, fixed here).
T29b whole (the user, decision 97; two parts were recommended), with `month-start-builds-36`
for the week workload (the user's pick). The user's computer was connected at the start only:
the original data and the four manuals staged then (28 files, SHA-256 checked), used for the
scoped verify. `cross-lane`: `h3-map`'s `play.rs` (the counts' bound, the Plague, a purchase's
amount, a fork's week; `Step::Town`'s hero moves; five oracles taught a garrison hero that
defends) and `view.rs`, the workload. No version moved. Note in the adventure lane's inbox. T30
next. PRs #942 and #946.

### 2026-11-15 — T29a: the scripted replays on the builds map, the census of growth

Baseline 6b3087d (#930 to #933 merged; CI's fingerprint, the light gates). T29 in two, T29a now
(the user, decision 96; three parts were recommended). The user's computer was connected at the
start only: the original data and the four manuals staged then (28 files, SHA-256 checked), used
for the census and the scoped verify. `cross-lane`: `h3-map`'s builds map (the Pit, the Grove,
Valeska, a Guardhouse) and `original_data` (the census: 160 maps to day 29 against its own walk),
the six `builds-*` replays re-recorded and three new, `day-start-builds-36` re-baselined (+51.7 %,
the map). No version moved. Note in the adventure lane's inbox. T29b queued.

### 2026-11-14 — T28b2: a garrison hero defends; T-5; the town's views

Baseline 0872fc6 (#924 merged; the setup's baseline lost to a VM restart, CI's green run on
`master` stands for it). T28b2 whole (the user, decision 95; two parts were recommended). The
user's computer was connected at the start: the original data and the four manuals staged then
(28 files, SHA-256 checked), used for the scoped verify and a third reading of `town_defended`.
`cross-lane`: `h3-adventure`'s `end_battle`, `raise`, `change_marks`; `h3-game`'s battle end and
`town_told`; `h3-map`'s view property; `h3-protocol` (v67). A hygiene case found failing on
`master` too (a map garrison's index), fixed by the adventure lane's #929, merged in. Notes in the
adventure's and the server's inboxes. T29 next. PR #930.
