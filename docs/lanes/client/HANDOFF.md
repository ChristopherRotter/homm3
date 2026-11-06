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
  T5a done (#736, design §12.17); T5b cut in two by the user (2026-11-05): T5b1 done (#743,
  design §12.18); T5b2 cut in two by the user (2026-11-05): T5b2a done (#745, #751, design
  §12.19); T5b2b done (#757, design §12.20); **T5c done** (kept whole by the user, design
  §12.21: Eagle Eye's message and the box with pictures, two sequences exact; CLT-Q65 and Q66
  asked; #762 merged; the user added **T5e**, the captured artifacts' message). T5d cut in two
  by the user (2026-11-06): T5d1 done (#771, design §12.22); T5d2 cut in two by the user
  (2026-11-06): **T5d2a done** (design §12.23: Death Blow, an Ogre Mage's Bloodlust and its
  glow; `blows` and `blows-sound` as recorded; #774: check it
  first). **Next: T5d2b** (Ice Bolt's explosion, the Familiars' channel; QUEUE), T5e, T6, T7;
  then M7 ranked play and the server browser.
  M5's record: `E17-archive.md`, `E17-log.md`.
  Workspace `clients/` (`clients/README.md`), gates `clients/scripts/check.sh`; ADR-0020 (+ drafts
  in `adr-drafts/`), ADR-0024; the harness `clients/capture/README.md`; design §3–§12; records in
  `E17-log.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*` and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first. **Open:** CLT-Q5 (its
  situations M6 T6), CLT-Q42 (T3b2: copy recommended, built so), CLT-Q43 and Q44 (T4a: fixes
  recommended, built as the original's), CLT-Q63 (T5b2a: copy recommended, not built), CLT-Q64 (T5b2b: fix recommended, CLT-Q45 (1)'s rule built), CLT-Q65 (T5c: the unlearned
  picks; fix once a whole game tells what was learned), CLT-Q66 (T5c: the window under the
  boxes; copy recommended, built so). `CLT-STUB-1` (T5a, G1: a server's seat lacks the other
  hero's Orb and Cloak; the user requires parity with local play, asked of the battle and server
  lanes) and `CLT-STUB-3` (T5c: a seat gets no Eagle Eye picks at the end; asked of the server
  lane) are commented in `docs/stubs.toml` until `check-stubs` scans `clients/` (a `shared/`
  change). An engine change the lane needs goes as a small cross-lane PR with the user's consent
  (#739 and #745 were). Next ids:
  CLT-Q67, CLT-STUB-4.
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

- **2026-11-06, E17 M6 T5d2a (done; `client/e17-m6-t5d2a`, #774).** The user cut T5d2 in two
  (T5d2b: Ice Bolt's explosion, the Familiars' channel). The original data staged by the file
  tool (the PC's shell did not start); the sandbox restarted during the setup's baseline (CI's
  green run on `master` stood in). Read and built (`anim/creature.rs`, the harness's roll
  `death_blow`), recorded by a capture agent (`blows`, `blows-sound`: as recorded on the first
  run), reviewed (the Master Genie's drawn Bloodlust takes the same path: documented
  provisional). Record: `E17-log.md`.
- **2026-11-06, E17 M6 T5d1 (done; `client/e17-m6-t5d1`, #771).**
  The user cut T5d in two. The original data staged by the file tool (the PC's shell did not
  start). Read, built (`anim/starts.rs`), recorded by a capture agent (`starts`,
  `starts-sound`: the three effects exact; `motion::magic` now loads their animations), reviewed
  (a passed stack's outline, the round start's clear fixed). Record: `E17-log.md`.
- **2026-11-06, E17 M6 T5c (done; #762).**
  Kept whole by the user, who queued the captured artifacts' message as T5e. The original data
  staged by the file tool (the PC's shell did not start; the PC went offline later, nothing
  missing). A reading (§12.21), a builder (the box with pictures, Eagle Eye's boxes, the pickup
  sounds), a capture agent (`eagle-eye`, `eagle-eye-one`: 164 and 37 frames exact after three
  corrections; CLT-Q66 asked), stopped by the user's interruption after its last digests: the
  session committed its work and ran its gates. A fresh reviewer. CLT-STUB-3 and the server
  lane's inbox: a seat's picks.
