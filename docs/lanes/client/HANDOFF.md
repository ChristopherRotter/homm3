# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E17 M0–M5 done** (M5, live play, closed at T8c on 2026-10-31 with the user: the review
  list accepted as built; CLT-Q26 to Q28, Q32, Q33 and Q35 answered, with a general directive:
  a text the original sets off its centre is centred in the client's profile, the epic's
  "Decisions"; the lane split at M3 T9c, CLT-Q18: the battle client here, the adventure map
  client the map-client lane's E24). **M6 (the battle screen complete)** refined at T8c
  (QUEUE, the epic's M6): T1 the answered slips, T2 a stack's info, T3 the right button's other
  popups, T4 the combat options dialog, T5 M5's gaps, T6 the inbox's waiting items (the deck and
  boats, CLT-Q30's exact Bink, the Demons' dissolve, `h3_ui::paint`, the purity holes, R4-2,
  R4-9), T7 the closure; then M7 ranked play and the server browser. **T1 done** (the answered
  slips, #620 from `client/e17-m6-t1`: check it first; T8c's #618 merged). **Next: M6 T2**
  (a stack's info; likely cut at its start). M5's record:
  `E17-archive.md`, `E17-log.md` ("M5 T1a" to "M5 T8c", "M5 review list").
  Workspace `clients/` (`clients/README.md`), gates `clients/scripts/check.sh`; ADR-0020 (+ drafts
  in `adr-drafts/`), ADR-0024; the harness `clients/capture/README.md`; design §3–§11; records in
  `E17-log.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*` and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first. **Open:** CLT-Q5 (its
  situations M6 T6). Next id: CLT-Q36.
- Stage from `original-data/install/`: the four LODs (one per call), `h3maped.exe`, both `.snd`, `MP3/COMBAT01`–`04` and the end tunes, `Heroes3.exe`, the five DLLs, `H3ab_ahd.vid`, and `original-data/battle-videos.vid` (the result window's clips: the harness's `VIDEO.VID`, the viewer's stand-in); `cp -rn` the staged `original-data/.` into the clone. Frames stay in the sandbox.

## Autonomy contract (agreed by the user, 2026-10-12)

- **Agent decides, marked provisional:** the inner structure of the client crates, names, test layout, CLI flags, caching, debug output; asset fixes whose source is a code reading or a reference frame (each an entry with its id). Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency (`png`, an MP3 decoder), the asset tables' and layouts' formats, a visual deviation from the original without a reference. List them in the epic's review list and go on.
- **Ask as an open question** (the user, 2026-10-22): every oddity of the original a reading or a recording finds becomes a `CLT-Q<n>` in the same task (copy, fix with the original's for the comparisons only, or an option; a recommendation); build it as the original's meanwhile.
- **Stop the line:** a change to an engine crate or another lane's paths (CLT-Q1 was one), shared paths (CI, `lanes.toml`: a `shared/` PR), heavy dependencies (the window/audio library, a GPU stack, a UI framework, a video codec), licence questions (reading MMArchive's, homm3-unpacker's or DDrawCompat's code included), committing anything taken from the original beyond identifiers and numbers (ADR-0009), anything that would write into the installation. Write the question (`CLT-Q<n>`), stop that task and take the next independent one.

## Standing traps

Moved to `TRAPS.md`: CI out, purity allowlist, licences and SDL, the lockfile gate, never running the original from the installation, memory against screen, no floats in drawing, the harness's actions and rolls, captures, sieges.

## Asset notes

The user's dictation of 2026-10-12 and the research items it led to: `ASSET-NOTES.md` (client lane's file; the map-client lane's own research items in `docs/lanes/map-client/ASSET-NOTES.md`). Read it before decoding or drawing an asset.

## Recent sessions

- **2026-10-31, E17 M5 T8c and M6 T1 (done; #618 merged, #620 open: check it first).** The
  original data staged (57 files, before anything else); #616 and #617 merged that morning, CI
  green on `master`; the VM restarted ~20 minutes in (the baseline lost). T8c, the user's
  checkpoint: the review list accepted as built, CLT-Q35 and the five older questions answered
  one by one (Q28 with the centring directive), M6 refined (draft (A)); docs only. M6 T1: the
  answered slips fixed in the client's profile; `drawn` all the same; `check.sh`.

- **2026-10-30/31, E17 M5 T8b2 (done; #616 merged).** The original data staged
  (the user's computer offline half an hour in); the user's cut: three concurrent sub-agents, the
  runner the leftovers; the VM restarted ~10 minutes in (CI's green `master` the baseline).
  Reviewers R5a, R5b, R6, R4b, R5c and their fixers in worktrees, merged; the leftovers; the
  harness set up for a live probe of the Gold bonus (0x69FBE8, observed); `drawn` all 156 the
  same; `check.sh`. Items 78–86, CLT-Q35.

- **2026-10-30, E17 M5 T8b1 (done; #614 merged).** The original data staged
  (the user's computer offline an hour in); the user's cut of T8b in two; the VM restarted ~40
  minutes in (two captures rerun). R7 (the harness) and R2a/R2b (the decoder, the clip) and
  their fixers (F7 in a worktree, merged with both instances idle and set up again); sequences
  28 to 64 as recorded (`auto`'s 7 530 now exact, recorded); the harness check passed at its
  rerun; `drawn` and `check.sh` with the fixes. Items 73–77.
