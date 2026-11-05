# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E17 M0–M5 done** (M5 closed at T8c on 2026-10-31: the review list accepted; the lane split
  at M3 T9c, CLT-Q18: the battle client here, the adventure map client the map-client lane's
  E24). **M6 (the battle screen complete)**, its tasks in QUEUE and the epic's M6: T1 and T2
  done (#620 to #672, T2's cuts in the epic). T3 cut in two by the user (2026-11-03): T3a done
  (#676, the panel's controls' help, design §12.7); T3b cut in two too: T3b1 done (#679), T3b2
  done (#690; CLT-Q42 asked); T3c cut by the user (2026-11-03): T3c1 done (#692), T3c2 done
  (#699), T3c3 done (#706). T3 done. **T4** cut in three by the user (2026-11-04): T4a done
  (#709, design §12.12); T4b cut in three by the user (2026-11-04): T4b1 done (the Auto
  Combat options, §12.13; CLT-Q45) and T4b2a done (#714: F5, the status line's damage, §12.14;
  CLT-Q46; the windows' own keys on Ctrl+F5 to Ctrl+F8). T4b2b cut in two by the user
  (2026-11-04): T4b2b1 done (#717, the heroes' windows, §12.15; CLT-Q47 to Q50) and T4b2b2 done
  (#722). **T4c done** (kept whole by the user: the grid, the movement and cursor shadows, the
  aims' shades, F6 to F8 saved at once, §12.16; CLT-Q52 to Q56 answered, all fixed; 11 stills
  exact, 5 sequences; #729 merged). T4 done. **T5** cut in four by the user (2026-11-05):
  **T5a in progress** (the book's refusals, the aim's other messages, Force Field's and Fire
  Wall's aim; branch `client/e17-m6-t5a`), then T5b, T5c, T5d (QUEUE), T6, T7; then M7 ranked
  play and the server browser.
  M5's record: `E17-archive.md`, `E17-log.md`.
  Workspace `clients/` (`clients/README.md`), gates `clients/scripts/check.sh`; ADR-0020 (+ drafts
  in `adr-drafts/`), ADR-0024; the harness `clients/capture/README.md`; design §3–§12; records in
  `E17-log.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*` and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first. **Open:** CLT-Q5 (its
  situations M6 T6), CLT-Q42 (T3b2: copy recommended, built so), CLT-Q43 and Q44 (T4a: fixes
  recommended, built as the original's). Next id: CLT-Q57.
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

- **2026-11-05, E17 M6 T5a (in progress, branch `client/e17-m6-t5a`).**
  The original data staged (57 files, by the file tool; the device dropped once mid-staging and
  came back); #729 merged; the baseline's `check.sh` running. The user cut T5 in four and chose
  T5a. Next: the reader (design §12.17), the builder, the captures, a fresh reviewer.

- **2026-11-04/05, E17 M6 T4c (done; #729 merged).**
  The original data staged (60 files, by the file tool: the device shell did not start); the
  baseline's `check.sh` passed. The user kept T4c whole. A reader (§12.16: `battle_shade_hexes`,
  `shade_mouse_hex` 0x494390, the aims'; CLT-Q52 to Q56 asked and answered at once: fix all), a
  builder (`h3_view::shades`), the captures (11 stills exact, 5 sequences; the sandbox's server
  stopped overnight mid-capture: resumed, nothing lost) and a fresh reviewer (five findings
  fixed; the targeted aims call no `battle_set_shades`, read). `drawn` 219 the same, 16 new (235 the same after the review); `check.sh`.

- **2026-11-04, E17 M6 T4b2b2 (done; #722 merged).**
  The original data staged (93 files, by the file tool: the device shell did not start); #717
  had merged; the baseline's `check.sh` passed. The task kept whole (its windows and a still's
  status line are recorded together). A builder (`h3_view::army_info`'s stacks' windows, the
  fill from the stats dialog's numbers, CLT-Q48 by profile, a still's previews), the captures
  (six stills exact; kind 2's ids read from the original's memory), a fresh reviewer (five
  findings: the still's same-slot line rewrite, docs, tests; fixed) and CLT-Q51 (negative
  luck's icon) asked and answered: copied. `drawn` 213 the same, 6 new; `check.sh`.
