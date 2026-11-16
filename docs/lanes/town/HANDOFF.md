# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 3 closed (T10, T20, T30; `E09-archive.md`, design notes Parts A
  to E). **Increment 4** (the Tavern; decisions 98–124, `TWN-Q15` to `-Q18`): T31 to T34 done;
  its drafts accepted at T33b (`TWN-Q18`); **batch TWN-4 run and scored by T34** (two runs of each of
  three maps, `experiments/results/E09-batch-4*.md`; every row as the reading, survey
  `E09-tavern.md` §14). **Next:** T35 (`shared/`: the drafts into the ADRs with T34's verdicts;
  at its start put the two points the reading missed to the user: the new day clears the
  returning mark of every hero record and gives every record its day's spell points; `QUEUE.md`).
  The guild and the services are increment 5, travel and the Grail 6. Waiting elsewhere:
  `TWN-STUB-8` (the battle lane's inbox), `TWN-STUB-6` and `-13` (increment 5), `TWN-STUB-2` (the
  victory conditions).
- **Open pull requests:** #973 (`town/e09-t34`, `cross-lane`); watch it until it merges.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v3: T25a's counts
  and Portal) and the town ruleset part (`TOWN_RULESET_HASH_VERSION`, v6: T28a's `most_heroes`),
  both in `h3-towns`; the adventure lane's (game state hash v32, adventure state hash v25 by T28a,
  map import v23, replay format v16 by T28a), the env's (v11 by T28a) and the protocol's (v67 by
  T28b2; v69 on `master` now) are bumped `cross-lane` (each task's record in `E09-log.md`; the
  plan in design §77). Shared, from T24: core ruleset hash v53, pack format v52, quirk flags 41,
  `StreamId::TOWNS` (7).
- **Ids** (next): questions `TWN-Q19`; decisions 125; quirks `TWN-QUIRK-14` (`-12`, `-13` planned
  by §77); stubs `TWN-STUB-19` (`-11`, `-12` planned by §34; `-15` to `-18` by §71, §77); rules:
  design §55's all built, TWN-TAV-001 to -009, GAM-DEC-010, GAM-MAN-003 planned; experiment batch
  `TWN-5` (TWN-4 run by T34); disputes `TD-24`; quirk candidates `T-18`.
  ADR-0028 is the lane's; drafts `adr-drafts/*-increment-4.md` (accepted, moved by T35).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-16 — T34: batch TWN-4

Baseline 77c2eec (#969 merged; the setup ran the light gates only, CI's fingerprint on `master`).
The user kept T34 whole (decision 123) and consented to the `cross-lane` writer and
`h3-experiment` changes. The user's computer was connected at the start only: the client lane's 64
files and the manuals and help files staged (70, SHA-256 checked). Built: the writer's Tavern and
Den; the harness's taverns, guild, draws with ranges and steps (README "Taverns and the Thieves'
Guild"); three specs and recipes, two runs each, the scoring (survey §14). Two points the reading
missed (the new day's clear of the mark, the day's spell points of every record) wait for T35's
checkpoint. Inbox notes in the adventure, client and map-client lanes (rebuild `xdd.dll`).

### 2026-11-16 — T33b: the information's drafts, Part F §72 to §81, the checkpoint

Baseline fcc19e7 (#966 merged; CI green on `master`). The user kept T33b whole (decision 114)
and answered `TWN-Q18` (two rounds, seven questions) as recommended but T-17 (a `fix-optional`
flag over the agent's `preserve`). The user's computer was connected at the start only: the same
18 files as T33a staged (SHA-256 checked). Read for it: the Thieves' Guild's parts and rows
(survey §13; TD-23, T-17). Docs only; no version moved. Notes in the adventure, AI and server
lanes' inboxes.

### 2026-11-16 — T33a: the hire's drafts and Part F §59 to §71

Baseline f6dd6d3 (#961, #962, #964 merged; CI green on `master`). The user's computer was
connected at the start only: the executable, the LODs, the tars, the saves, the manuals and the
help files staged (18 files, SHA-256 checked). The user cut T33 in two and answered `TWN-Q17`'s
four ADR-level choices as recommended (decisions 108–112). Read for it: the dismissal's checks
(survey §12, four map entries). Docs only; no version moved.
