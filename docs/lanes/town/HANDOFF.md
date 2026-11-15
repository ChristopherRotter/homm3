# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 3 closed (T10, T20, T30; `E09-archive.md`, design notes Parts A
  to E). **Increment 4** (the Tavern; decisions 98–102, `TWN-Q15`): refined at T30 as sketched:
  the Tavern's hire (two offers a player from the shared pool by the classes' tavern weights, the
  price, the most heroes, the week's refresh, the heroes that lost, retreated or surrendered
  offered again, the map's customized heroes), the Thieves' Guild's overview by taverns owned and
  the Den of Thieves. **Next:** T31 (the garrison moves below level 3, decision 101; code,
  `cross-lane`), then T32 (the survey, its checkpoint `TWN-Q16`) and T33 (the drafts, Part F from
  §59, the cut); `QUEUE.md`. The guild and the services are increment 5, travel and the Grail 6.
  Waiting elsewhere: `TWN-STUB-8` (the battle lane's inbox), `TWN-STUB-6` and `-13` (increment
  5), `TWN-STUB-2` (the victory conditions).
- **Open pull requests:** #957 (`town/e09-t30`, docs) and #956 (`shared/e09-t30-readme`, the
  README's row); watch both until they merge.
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
- **Ids** (next): questions `TWN-Q16`; decisions 103; quirks `TWN-QUIRK-12`; stubs `TWN-STUB-15`
  (`-11`, `-12` planned by design §34); rules: design §55's all built; experiment batch `TWN-4`;
  disputes `TD-18`; quirk candidates `T-14`. ADR-0028 is the lane's; no drafts wait.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-15 — T30: the audit, increment 3 closed, increment 4 refined

Baseline 396f6f8 (#942 to #953 merged; the setup's baseline lost to a VM restart, CI's green runs
on `master` stand for it). T30 whole (the user, decision 98; `TWN-Q15`, all four as recommended).
The user's computer was connected at the start only: the original data and the four manuals
staged then (28 files, SHA-256 checked), used for the scoped verify on `master`'s code (no failing
step; the census to day 29 as T29a's). The eleven criteria hold; nine rules behind batch TWN-3's
rows `verified`; the garrison moves' names below level 3 queued as T31 (decision 101). Increment 3
archived; increment 4 the Tavern as sketched (decisions 99, 100, 102); seven inbox notes archived.
Docs only; PRs #957 and #956 (the README's row).

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
