# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 3 closed (T10, T20, T30; `E09-archive.md`, design notes Parts A
  to E). **Increment 4** (the Tavern; decisions 98–102, `TWN-Q15`): refined at T30 as sketched
  (the Tavern's hire from the shared pool by the classes' tavern weights, the heroes that come
  back, the map's customized heroes, the Thieves' Guild by taverns owned, the Den of Thieves).
  **T31 done** (decision 101: the garrison moves by name only to the viewers that see the garrison
  hero). **T32 done** (the survey `E09-tavern.md`; `TWN-Q16`, decisions 103–107: TD-18 to TD-20 as
  read, T-14 not a quirk, the map's Tavern in the increment). **Next:** T33 (the ADR amendment
  drafts, Part F from §59, the cut with batch TWN-4's rows; the user's acceptance before building);
  `QUEUE.md`. The guild and the services are increment 5, travel and the Grail 6.
  Waiting elsewhere: `TWN-STUB-8` (the battle lane's inbox), `TWN-STUB-6` and `-13` (increment
  5), `TWN-STUB-2` (the victory conditions).
- **Open pull requests:** #959 (`shared/e09-t31-security`, the security doc's line; merges first),
  then T31's (`town/e09-t31`, `cross-lane`), then T32's (`town/e09-t32`, docs, stacked on T31's:
  after T31's squash, `git rebase --onto origin/master <T31's head>`); watch all until they merge.
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
- **Ids** (next): questions `TWN-Q17`; decisions 108; quirks `TWN-QUIRK-12`; stubs `TWN-STUB-15`
  (`-11`, `-12` planned by design §34); rules: design §55's all built; experiment batch `TWN-4`
  (rows in the survey's §9, not run); disputes `TD-22`; quirk candidates `T-16`. ADR-0028 is the lane's; no drafts wait.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-15 — T32: the Tavern survey and its checkpoint

After T31 in the same session (the user's cut: T32 whole). The staged original data and manuals
used for the code reading and a throwaway census (159 maps); VCMI 1.7.5 from GitHub, the wiki's
Tavern page. `TWN-Q16` answered as recommended (decisions 103–107). Docs only; a lead for E11 in
the battle lane's inbox (the SoD edition's Conflux rule, dead in this build). PR stacked on T31's.

### 2026-11-15 — T31: the garrison moves named only to the viewers that see the garrison hero

Baseline db8406b (#956, #957 merged; the setup's full baseline failed coverage on a server lane
clock test under load, which passes alone; CI's green run on `master` stands for it). The user
chose the projection (a `named` mask on the two events, projected in `observe.rs`) and T32 whole
after T31. The user's computer was connected at the start only: the original data, the four
manuals and `H3MAPED.HLP` staged then (29 files, SHA-256 checked). `cross-lane`: `h3-adventure`'s
`command.rs` and `observe.rs`, `h3-map`'s `view.rs` (`check_garrison_moves`), `h3-protocol`'s
mapping and tests; the security doc's line in #959. No version moved. Notes in the adventure and
server lanes' inboxes. The VM restarted during the scoped verify's determinism step.

### 2026-11-15 — T30: the audit, increment 3 closed, increment 4 refined

Baseline 396f6f8 (#942 to #953 merged; the setup's baseline lost to a VM restart, CI's green runs
on `master` stand for it). T30 whole (the user, decision 98; `TWN-Q15`, all four as recommended).
The user's computer was connected at the start only: the original data and the four manuals
staged then (28 files, SHA-256 checked), used for the scoped verify on `master`'s code (no failing
step; the census to day 29 as T29a's). The eleven criteria hold; nine rules behind batch TWN-3's
rows `verified`; the garrison moves' names below level 3 queued as T31 (decision 101). Increment 3
archived; increment 4 the Tavern as sketched (decisions 99, 100, 102); seven inbox notes archived.
Docs only; PRs #957 and #956 (the README's row).
