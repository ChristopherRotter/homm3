# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 3 closed (T10, T20, T30; `E09-archive.md`, design notes Parts A
  to E). **Increment 4** (the Tavern; decisions 98–122, `TWN-Q15` to `-Q18`): T31 to T33 done;
  **its drafts accepted** at T33b's checkpoint (`TWN-Q18`, decisions 115–122: the taverns' stream,
  `TWN-QUIRK-12` T-16 and `-13` T-17 both `fix-optional`, TD-22 and TD-23 the reading, the views,
  the env, the cut T34 to T41 and twelve exit criteria; design note Part F §59 to §81).
  **Next:** T34 (batch TWN-4; needs the original data and the client lane's harness set-up;
  `QUEUE.md`). The guild and the services are increment 5, travel and the Grail 6. Waiting
  elsewhere: `TWN-STUB-8` (the battle lane's inbox), `TWN-STUB-6` and `-13` (increment 5),
  `TWN-STUB-2` (the victory conditions).
- **Open pull requests:** #969 (`town/e09-t33b`, docs); watch it until it merges.
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
- **Ids** (next): questions `TWN-Q19`; decisions 123; quirks `TWN-QUIRK-14` (`-12`, `-13` planned
  by §77); stubs `TWN-STUB-19` (`-11`, `-12` planned by §34; `-15` to `-18` by §71, §77); rules:
  design §55's all built, TWN-TAV-001 to -009, GAM-DEC-010, GAM-MAN-003 planned; experiment batch
  `TWN-4` (rows in the survey's §9, not run); disputes `TD-24`; quirk candidates `T-18`.
  ADR-0028 is the lane's; drafts `adr-drafts/*-increment-4.md` (accepted, moved by T35).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

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

### 2026-11-15 — T32: the Tavern survey and its checkpoint

After T31 in the same session (the user's cut: T32 whole). The staged original data and manuals
used for the code reading and a throwaway census (159 maps); VCMI 1.7.5 from GitHub, the wiki's
Tavern page. `TWN-Q16` answered as recommended (decisions 103–107). Docs only; a lead for E11 in
the battle lane's inbox (the SoD edition's Conflux rule, dead in this build). PR stacked on T31's.
