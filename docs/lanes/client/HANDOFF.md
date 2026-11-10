# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E17 M0–M5 done** (M5 closed at T8c on 2026-10-31: the review list accepted; the lane split at M3
  T9c, CLT-Q18: the battle client here, the adventure map client the map-client lane's E24). **M6
  (the battle screen complete)**, its tasks in QUEUE and the epic's M6 (each task's cuts, dates and
  pull requests there): T1 to T6 done (T5e deferred by the user to the battle client inside an
  adventure game, the epic's "Later"). **T7** (M6's closure) cut in three by the user
  (2026-11-09): **T7a done** (#812); the inbox's protocol v38 readers done (#818, design §12.25);
  **T7b done** (2026-11-10, PR on `client/e17-m6-t7b`: check it first; 12 reviewers and 6
  fixers, 56 findings, 51 fixed, 3 left; review items 49 to 62; CLT-Q70 to Q77).
  **Next:** T7c (the review list with the user, the new questions and T7b's left items, M7
  refined, M6 closed); then M7 ranked play and the server browser.
  M5's record: `E17-archive.md`, `E17-log.md`.
  Workspace `clients/` (`clients/README.md`), gates `clients/scripts/check.sh`; ADR-0020 (+ drafts
  in `adr-drafts/`), ADR-0024; the harness `clients/capture/README.md`; design §3–§12; records in
  `E17-log.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*` and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first. **Open:** CLT-Q42 (T3b2: copy recommended, built so), CLT-Q43 and Q44 (T4a: fixes
  recommended, built as the original's), CLT-Q63 (T5b2a: copy recommended, not built), CLT-Q64 (T5b2b: fix recommended, CLT-Q45 (1)'s rule built), CLT-Q65 (T5c: the unlearned
  picks; fix once a whole game tells what was learned), CLT-Q66 (T5c: the window under the
  boxes; copy recommended, built so), CLT-Q69 (drawing the battle's start: the battle before it
  in the record recommended, a server and battle lane change; not drawn meanwhile); from T7b,
  each built as the original's or as it was: CLT-Q70 (a heroless side's colours: fix), Q71 (a
  hotkey's held panel button: a capture, then fix), Q72 (the DEF button's loop: copy, a task),
  Q73 (the music at volume 0: read, then keep the client's), Q74 (the speeds click once, against
  Q43 (1)), Q75 (F6 to F8 on a turn not taken: keys, no reach), Q76 (a wall with no free hex:
  fix), Q77 (the pictures' layout: copy).
  `CLT-STUB-1` (T5a, G1: a server's seat lacks the other hero's Orb and Cloak; the user requires
  parity with local play, asked of the battle and server lanes) is commented in
  `docs/stubs.toml` until `check-stubs` scans `clients/` (a `shared/` change). An engine change the lane needs goes as a small cross-lane PR with the user's consent
  (#739 and #745 were). Next ids:
  CLT-Q78, CLT-STUB-4.
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

- **2026-11-10, E17 M6 T7b (done; `client/e17-m6-t7b`).** The user's direction: "one session,
  delegated": the session a coordinator, 12 read-only reviewers (two at a time, a worktree of
  master 3e1026fc), 6 fixers one at a time on the branch, one docs agent. 56 findings (one high,
  R5-2, a test reproduced it; nine medium): 51 fixed, 2 in the records, 3 left; CLT-Q70 to Q77
  asked, review items 49 to 62. `master` merged once (ac8ae2ec). `check.sh` passed, `drawn`
  263 of 263 the same (no capture needed), `h3-view gate` passed.
  Record: `E17-log.md`.
- **2026-11-10, after E17 M6 T7a: the protocol v38 readers (done; `client/e17-m6-v38-readers`).**
  Kept whole with CLT-STUB-3 and the inbox's comment rename. The original data staged at once
  (the PC's shell and the file tool). `master` was red (#814 against #815 in `h3-protocol`):
  built against a local patch, never committed, until the server lane's #816 fixed it (the
  branch rebased on it). The readers leave the start's
  `Events` out (`record::commands`); `drawn` all 263 the same again; CLT-Q69 asked. Record:
  `E17-log.md`.
- **2026-11-09/10, E17 M6 T7a (done; `client/e17-m6-t7a`).** T7 cut in three by the user; T7a
  kept whole. The original data staged by the file tool at once (the PC left after an hour).
  Every capture afresh on two instances (172 situations, 91 sequences, all as recorded, about
  6 hours); R4-9 built (`h3cap.SHADOW_VALUES`), its counts moved only in listed differences'
  frames (recorded). Merging `master` (#800, protocol v38) broke `drawn` (81 of 91 sequences):
  the inbox's readers come first. Record: `E17-log.md`.
