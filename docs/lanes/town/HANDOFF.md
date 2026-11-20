# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 4 closed (T10, T20, T30, T41; `E09-archive.md`, design notes
  Parts A to F). **Increment 5** (the mage guild; decisions 161–171, `TWN-Q30`, `TWN-Q31`): its
  survey done in two parts (decision 165): T42a (`E09-guild.md` §0 to §9, what the guild holds)
  and **T42b** (§10 to §20, what a hero gets, with the checkpoint `TWN-Q31`: TD-10, TD-25 the
  original, T-18 `fix-optional`, TD-27 the original's triggers with a free opening command;
  decisions 166–171). **T43** (kept whole, decision 172): the four drafts
  (`adr-drafts/*-increment-5.md`), the design note's Part G (§82 to §100), the cut T44 to T51 and
  twelve exit criteria, accepted at `TWN-Q33` (decisions 173–181; the guilds drawn from the towns'
  stream, decision 179, so T47a re-records the week replays' outcomes). **Next:** T44, batch TWN-5
  (`QUEUE.md`). Waiting elsewhere: `TWN-STUB-8` (the battle lane's inbox), `TWN-STUB-6` and
  `-15` (increment 5), `-13` (Visions' reach), `TWN-STUB-2` (the victory conditions),
  `TWN-STUB-16` (E10's loss condition), `-17` (increment 7), `-18` (E13).
- **Open pull requests:** T43's (`town/e09-t43`, docs only), opened at the session's end; watch it
  until it merges.
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
- **Ids** (next): questions `TWN-Q34`; decisions 182; quirks `TWN-QUIRK-14` (planned for T-18) (`-12`, `-13` entered
  by T35b; their tests by T38b1, T39b1); stubs `TWN-STUB-21` (`-11`, `-12` planned by §34; `-16`
  by T38b2, `-17`, `-18` by T39b1; `-20` lifted by T39b2); rules: TWN-TAV-001 to -009, GAM-MAN-003,
  GAM-DEC-010 and the adventure's ADV-VIS-007, -008 built; experiment batch `TWN-5` (rows in survey
  §18); disputes `TD-28` (`TD-25`, `-26` by T42a, `-27` by T42b); quirk candidates `T-19` (`T-18`
  by T42a, `fix-optional` by decision 169).
  ADR-0028 is the lane's; drafts open: the four `*-increment-5.md` (accepted, for T45a); planned
  rules TWN-GLD-001 to -009, TWN-SET-012.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-20 — T43: increment 5's drafts, Part G, the cut; the checkpoints TWN-Q32, TWN-Q33

Baseline 67b86dd (#1067 merged; the setup's light gates on `master`'s code, CI's fingerprint).
The user's computer was connected at the start only: 61 files of `original-data/` and the four
manuals with three help files staged, SHA-256 checked. The user kept T43 whole (decision 172) and
chose at `TWN-Q32` the weights in the town records, the Aurora now, `Open` listed where useful,
`BuySpellBook` (173–176); at `TWN-Q33` both heroes learn at an opening, the towns' stream (over
the agent's new stream), the views as proposed, the cut as written (178–181). Written: the four
drafts, Part G §82 to §100, four lanes' inboxes. PR on `town/e09-t43`.

### 2026-11-20 — T42b: the survey's second part, what a hero gets; the checkpoint TWN-Q31

Baseline 4c19fbb (#1061 merged; CI green there; the setup's baseline was lost to a VM restart at
its test step, CI's run stands). The user's computer was connected at the start only: 61 files of
`original-data/` and the four manuals with three help files staged, SHA-256 checked. The user kept
T42b whole (decision 166). Read and counted: survey `E09-guild.md` §10 to §19 (learning and its
nine callers, the town screen's opening among them; the spell book; the night's refill; the Mana
Vortex; VCMI and the wiki; the heroes' census; TD-27; batch TWN-5's five rows). The checkpoint
`TWN-Q31` (§20; decisions 167–171, each as recommended). PR #1067.

### 2026-11-19 — T42a: the survey's first part, what the guild holds

Baseline 7dbe195 (CI green; the setup's run on 0d91fe8 failed coverage and determinism only on
the six taverns replays' `import_version`, re-recorded by the adventure lane's 7dbe195). The
user's computer was connected at the start only: 61 files of `original-data/` (the LODs, the
executables, DLLs, sounds and videos but `shell-videos.vid`, both archives, the 40 MP3s, the
three saved games) and the four manuals with three help files staged, SHA-256 checked. The user cut T42 in two, the guild first (decision 165). Read and
counted: survey `E09-guild.md` §0 to §9 (the chances, the draw, the counts, the Aurora, the
window, the census; TD-25, TD-26, T-18; batch TWN-5's three rows for T42b). PR #1061.
