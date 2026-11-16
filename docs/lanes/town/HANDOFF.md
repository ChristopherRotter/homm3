# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 3 closed (T10, T20, T30; `E09-archive.md`, design notes Parts A
  to E). **Increment 4** (the Tavern; decisions 98–113, `TWN-Q15` to `-Q17`): T31 and T32 done;
  **T33** in two (decision 108): **T33a done** (the hire's drafts `adr-drafts/*-increment-4.md`
  and Part F §59 to §71; `TWN-Q17`, decisions 109–112: a type's record at its first draw, the
  weights in the town part, a town command and a map decision, the dismissal now; the agent's
  calls of 113, among them a new kernel stream `TAVERNS`). **Next:** T33b (the information's
  drafts, Part F from §72, the cut, the exit criteria, the checkpoint `TWN-Q18`); `QUEUE.md`. The
  guild and the services are increment 5, travel and the Grail 6. Waiting elsewhere: `TWN-STUB-8`
  (the battle lane's inbox), `TWN-STUB-6` and `-13` (increment 5), `TWN-STUB-2` (the victory
  conditions).
- **Open pull requests:** #966 (`town/e09-t33a`, docs); watch it until it merges.
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
- **Ids** (next): questions `TWN-Q18`; decisions 114; quirks `TWN-QUIRK-12`; stubs `TWN-STUB-15`
  (`-11`, `-12` planned by design §34; `-15`, `-16` by §71); rules: design §55's all built, §71's
  planned; experiment batch `TWN-4` (rows in the survey's §9, not run); disputes `TD-23`; quirk
  candidates `T-17`. ADR-0028 is the lane's; drafts `adr-drafts/*-increment-4.md` (T33a's half,
  not yet accepted).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-16 — T33a: the hire's drafts and Part F §59 to §71

Baseline f6dd6d3 (#961, #962, #964 merged; CI green on `master`). The user's computer was
connected at the start only: the executable, the LODs, the tars, the saves, the manuals and the
help files staged (18 files, SHA-256 checked). The user cut T33 in two and answered `TWN-Q17`'s
four ADR-level choices as recommended (decisions 108–112). Read for it: the dismissal's checks
(survey §12, four map entries). Docs only; no version moved.

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
