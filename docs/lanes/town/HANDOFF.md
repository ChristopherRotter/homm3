# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 3 closed (T10, T20, T30; `E09-archive.md`, design notes Parts A
  to E). **Increment 4** (the Tavern; decisions 98–128, `TWN-Q15` to `-Q19`): T31 to T34 done
  (batch TWN-4: every row as the reading, survey `E09-tavern.md` §14); **T35 cut in two**
  (`TWN-Q19`, decision 125): **T35a done**, the drafts in ADR-0013, -0016, -0017, -0028 with T34's
  verdicts and decisions 126 (every day's start clears the returning marks) and 127 (every record
  regains its day's spell points: TD-20's spell-point half amended); **T35b done** (decisions 129
  to 131: `StreamId::TAVERNS`, the flags 42 and 43, the base faithful hash 0x6ba8109889b34a25);
  **T36 done** (decisions 130, 132: the town part v7, the tavern weights harvested, the AB pack's
  overrides of the base records, the hire's price). **Next:** T37a (the pool, the draw and the
  fills; `QUEUE.md`), its cut confirmed at its start.
  The guild and the services are increment 5, travel and the Grail 6. Waiting elsewhere:
  `TWN-STUB-8` (the battle lane's inbox), `TWN-STUB-6` and `-13` (increment 5), `TWN-STUB-2` (the
  victory conditions).
- **Open pull requests:** T35b's #979 (`shared/e09-t35b`, `cross-lane`) and T36's #980
  (`town/e09-t36`, `cross-lane`, stacked on #979; both had `master` with E08 T123b, #977, merged
  in): merge #979 first, then merge `master` into #980; watch both until they merge.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v3: T25a's counts
  and Portal) and the town ruleset part (`TOWN_RULESET_HASH_VERSION`, v7: T36's tavern weights and price),
  both in `h3-towns`; the adventure lane's (game state hash v32, adventure state hash v25 by T28a,
  map import v23, replay format v16 by T28a), the env's (v11 by T28a) and the protocol's (v67 by
  T28b2; v69 on `master` now) are bumped `cross-lane` (each task's record in `E09-log.md`; the
  plan in design §77). Shared: core ruleset hash v53, pack format v52 (T24), quirk flags 43 and `StreamId::TAVERNS`
  (8) by T35b (no version moved; the base faithful hash 0x6ba8109889b34a25).
- **Ids** (next): questions `TWN-Q20`; decisions 133; quirks `TWN-QUIRK-14` (`-12`, `-13` entered
  by T35b, their tests with T38b, T39b); stubs `TWN-STUB-19` (`-11`, `-12` planned by §34; `-15` to `-18` by §71, §77); rules:
  design §55's all built, TWN-TAV-001 to -009, GAM-DEC-010, GAM-MAN-003 planned; experiment batch
  `TWN-5` (TWN-4 run by T34); disputes `TD-24`; quirk candidates `T-18`.
  ADR-0028 is the lane's; no drafts open (increment 4's moved by T35a).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-17 — T35b and T36: the taverns' stream, the two flags, the town part v7

Baseline 4f4fbbc (#976 merged; the setup verified the code, e3cedff5, after a VM restart). The user
kept T35b whole and T36 after it, both `cross-lane` (decisions 129, 130). The user's computer was
connected at the start only: 11 files staged for T35b and T36 (the four LODs, both executables,
both tars, the saves; SHA-256 checked). Built: `StreamId::TAVERNS`, `QuirkFlag(42)`, `(43)`, the
profiles, the ledger entries; the 86 replays' `ruleset_hash`, the bot reports, the server literal
(#979). T36 stacked on it: the records' tavern weights and the hire's price (part v7), the
harvest, the AB pack's overrides (the base records cannot name its classes), the 89 replays'
`town_ruleset_hash`.

### 2026-11-16 — T35a: increment 4's drafts into the ADRs

Baseline a1d2c71 (#973 merged; the setup verified the code, 9ccb68f4). The user cut T35 in two
(decision 125) and answered T34's two points as the original (decisions 126, 127, `TWN-Q19`). The
user's computer was connected at the start only: 9 files staged for T35b and T36 (the executable,
the editor, both bitmap LODs, both tars, the saves; SHA-256 checked). Docs only: the four drafts
into the ADRs with batch TWN-4's rows, the glossary, `docs/06-security.md`'s section; the drafts'
point 9 corrected to §77 (no adventure state hash); T35b moves no version (the flags move the
profiles' hashes only). The adventure lane's inbox told.

### 2026-11-16 — T34: batch TWN-4

Baseline 77c2eec (#969 merged; the setup ran the light gates only, CI's fingerprint on `master`).
The user kept T34 whole (decision 123) and consented to the `cross-lane` writer and
`h3-experiment` changes. The user's computer was connected at the start only: the client lane's 64
files and the manuals and help files staged (70, SHA-256 checked). Built: the writer's Tavern and
Den; the harness's taverns, guild, draws with ranges and steps (README "Taverns and the Thieves'
Guild"); three specs and recipes, two runs each, the scoring (survey §14). Two points the reading
missed (the new day's clear of the mark, the day's spell points of every record) wait for T35's
checkpoint. Inbox notes in the adventure, client and map-client lanes (rebuild `xdd.dll`).
