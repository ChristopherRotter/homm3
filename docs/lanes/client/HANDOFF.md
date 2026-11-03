# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E17 M0–M5 done** (M5 closed at T8c on 2026-10-31: the review list accepted; the lane split
  at M3 T9c, CLT-Q18: the battle client here, the adventure map client the map-client lane's
  E24). **M6 (the battle screen complete)**, its tasks in QUEUE and the epic's M6: T1 and T2
  done (#620 to #672, T2's cuts in the epic). T3 cut in two by the user (2026-11-03): T3a done
  (#676, the panel's controls' help, design §12.7); T3b cut in two too: T3b1 done (#679, the
  hero's info window, design §12.8); **T3b2 done** (its morale and luck: the hero's own, from
  CLT-Q40's carry; CLT-Q42 asked; three stills exact; PR from `client/e17-m6-t3b2`: check it
  first); **next: T3c** (morale's and luck's descriptions' carried lines, from the inbox; cut it
  with the user at its start), then T4 to T7; then M7 ranked play and the server browser.
  M5's record: `E17-archive.md`, `E17-log.md`.
  Workspace `clients/` (`clients/README.md`), gates `clients/scripts/check.sh`; ADR-0020 (+ drafts
  in `adr-drafts/`), ADR-0024; the harness `clients/capture/README.md`; design §3–§12; records in
  `E17-log.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*` and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first. **Open:** CLT-Q5 (its
  situations M6 T6), CLT-Q42 (T3b2: copy recommended, built so). Next id: CLT-Q43.
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

- **2026-11-03, E17 M6 T3b2 (done; its PR open: check it first).** The original data staged
  (57 files) first; #679 and the battle lane's carry (#685) had merged; the baseline's `check.sh`
  passed. Read `hero_morale` 0x4E39B0, `hero_luck` 0x4E36C0 and the clamp whole (design §12.8:
  the hero's own values, 0 on Cursed Ground, the Hourglass's 0); built `hero_info::values` and
  the binary's `stats::field`; CLT-Q42 asked; three stills exact; `drawn`; `check.sh`. The inbox's
  CLT-Q40 item became T3c.

- **2026-11-03, E17 M6 T3b1 (done; #679 open: check it first).** The original data staged
  (57 files) first, then the computer went offline; #676 had merged; the baseline's `check.sh`
  passed. The user cut T3b in two (T3b1 the window, T3b2 its morale and luck). Read
  `army_info_hero_create` 0x46C370 whole and `battle_right_click`'s hero branches (design §12.8);
  built `[hero_info]`, `h3_view::hero_info`, `Battle::hero_info`, `Boxes::hero_info`; two stills
  exact at their first capture; the window checked under Xvfb; `drawn`; `check.sh`.

- **2026-11-03, E17 M6 T3a (done; #676 merged).** The original data staged
  (57 files) first; #672 had merged; the VM restarted during the baseline's coverage, CI's green
  run on `master` stood for it. The user cut T3 in two (T3a the controls' help, T3b the heroes'
  windows). Read `battle_control_help`, `message_box_size` and the items' right press (design
  §12.7); built `panel::help`, `Battle::control_help`, `Boxes::help`; five stills exact; the
  window checked under Xvfb; `drawn`; `check.sh`.
