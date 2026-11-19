# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 4 closed (T10, T20, T30, T41; `E09-archive.md`, design notes
  Parts A to F). **Increment 5** (the mage guild; decisions 161–165, `TWN-Q30`): T42 cut in two
  (decision 165); **T42a done** (survey `E09-guild.md` §0 to §9: what the guild holds). **Next:**
  T42b (what a hero gets, then the checkpoint `TWN-Q31` over both parts; `QUEUE.md`), then T43
  (the drafts, Part G, the cut). Waiting elsewhere: `TWN-STUB-8` (the battle lane's inbox),
  `TWN-STUB-6` and `-15` (increment 5), `-13` (Visions' reach), `TWN-STUB-2` (the victory
  conditions), `TWN-STUB-16` (E10's loss condition), `-17` (increment 7), `-18` (E13).
- **Open pull requests:** T42a's (#1061, `town/e09-t42a`, docs only), opened at the session's end;
  watch it until it merges.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v3) and the town
  ruleset part (`TOWN_RULESET_HASH_VERSION`, v8: T39b1), both in `h3-towns`; the adventure lane's
  (game state hash v34, the adventure part v22, map import v26 and replay format v19, all by
  T39a; the part v23 since E08 T132a2; adventure state hash v26 by E08's T124b1), the env's (v13 by T39a) and the protocol's
  (v74 by T39b2) are bumped `cross-lane` (each task's record in `E09-log.md`; the
  plan in design §77). Shared: core ruleset hash v53, pack format v52 (T24),
  quirk flags 43 and `StreamId::TAVERNS` (8) by T35b.
- **Ids** (next): questions `TWN-Q31`; decisions 166; quirks `TWN-QUIRK-14` (`-12`, `-13` entered
  by T35b; their tests by T38b1, T39b1); stubs `TWN-STUB-21` (`-11`, `-12` planned by §34; `-16`
  by T38b2, `-17`, `-18` by T39b1; `-20` lifted by T39b2); rules: TWN-TAV-001 to -009, GAM-MAN-003,
  GAM-DEC-010 and the adventure's ADV-VIS-007, -008 built; experiment batch `TWN-5`; disputes `TD-27`
  (`TD-25`, `-26` by T42a); quirk candidates `T-19` (`T-18` by T42a).
  ADR-0028 is the lane's; no drafts open.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-19 — T42a: the survey's first part, what the guild holds

Baseline 7dbe195 (CI green; the setup's run on 0d91fe8 failed coverage and determinism only on
the six taverns replays' `import_version`, re-recorded by the adventure lane's 7dbe195). The
user's computer was connected at the start only: 61 files of `original-data/` (the LODs, the
executables, DLLs, sounds and videos but `shell-videos.vid`, both archives, the 40 MP3s, the
three saved games) and the four manuals with three help files staged, SHA-256 checked. The user cut T42 in two, the guild first (decision 165). Read and
counted: survey `E09-guild.md` §0 to §9 (the chances, the draw, the counts, the Aurora, the
window, the census; TD-25, TD-26, T-18; batch TWN-5's three rows for T42b). PR #1061.

### 2026-11-19 — T41: the audit, increment 4 closed, increment 5 refined

Baseline 9404617 (#1053 merged; master's code verified by CI, the setup's light gates passed).
The user's computer was connected at the start: 63 files of `original-data/` and the four
manuals with three help files staged, SHA-256 checked. The user kept T41 whole, made increment 5
the mage guild alone (the services 6, travel and the Grail 7) and kept round 1 as before
(decisions 161 to 164, `TWN-Q30`). The audit (`E09-log.md`, T41): the twelve criteria hold;
TWN-SET-005 `verified`. Increment 4's section in `E09-archive.md`; T42, T43 queued. PRs #1059,
#1058.

### 2026-11-19 — T40b: the taverns map and its replays, the plans' map Tavern and Den

Baseline dc92305 (#1048 merged; master's code verified by CI, the setup's light gates passed).
The user's computer was connected at the start: 63 files of `original-data/` and the four
manuals with three help files staged, SHA-256 checked. The user kept T40b whole and consented
to `cross-lane` (decisions 159, 160, `TWN-Q29`). Built: design §78's As built (T40b): the plans'
`Plan::taverns` with `tavern_answers`, `check_map_hire` and the decision in `check_decisions`
and `managed`; `e09-taverns-36.h3m` and six `taverns-*-s1` replays with their tests; the golden
counts 104; `week-start-taverns-36`. PR #1053.
