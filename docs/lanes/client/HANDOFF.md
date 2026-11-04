# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E17 M0–M5 done** (M5 closed at T8c on 2026-10-31: the review list accepted; the lane split
  at M3 T9c, CLT-Q18: the battle client here, the adventure map client the map-client lane's
  E24). **M6 (the battle screen complete)**, its tasks in QUEUE and the epic's M6: T1 and T2
  done (#620 to #672, T2's cuts in the epic). T3 cut in two by the user (2026-11-03): T3a done
  (#676, the panel's controls' help, design §12.7); T3b cut in two too: T3b1 done (#679), T3b2
  done (#690; CLT-Q42 asked); T3c cut by the user (2026-11-03): T3c1 done (#692), T3c2 done
  (#699), **T3c3 done** (the buildings by the packs' town records, the user's choice; three siege
  stills exact, design §12.11; PR from `client/e17-m6-t3c3` after the shared #704, CI's path
  filter for `h3-pack-towns`: check both first). T3 done. **Next:
  T4** (the combat options dialog; cut at its start, QUEUE), then T5 to T7; then M7 ranked play
  and the server browser.
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

- **2026-11-04, E17 M6 T3c3 (done; PR open: check it first).** The original data staged (57
  files) first, the computer offline afterwards; #699 had merged; the baseline's `check.sh`
  passed. The user chose the town records over the asset tables (T3c2's question) and T3c3 with
  it, and the shared #704 (`clients.yml` watches `h3-pack-towns`). The binary compiles the
  packs' town part; the map writer builds the siege's buildings by their slots; `effect_skills`' Grail-as-Leadership fixed. A siege opens on a turret's or the
  Catapult's turn (TRAPS); three stills exact; `drawn`; `check.sh`.

- **2026-11-03, E17 M6 T3c2 (done; #699 merged).** The original data staged (57
  files) first; #692 had merged. The user asked for one part to a sub-agent, one commit: A (the
  Hourglass, the Spirit of Oppression, the Alliance's join; `stack_morale` 0x43E0D3 read) by the
  session, B (siege buildings, the Grail) by a sub-agent in a worktree, its patch applied and the
  conflicts resolved (design §12.10). The pack building indices live in the asset tables
  (*provisional*). `stats-morale-siege` added, not captured (T3c3).

- **2026-11-03, E17 M6 T3c1 (done; #692 merged).** The original data
  staged (57 files) first; #690 had merged; the baseline's `check.sh` passed. The user cut T3c
  in two (T3c1 the hero's lists and the unclamped value, T3c2 the rest). Read
  `hero_morale_description` 0x4DC320, `hero_luck_description` 0x4DCAC0 and `town_building_name`
  0x4610E0 whole (design §12.9); built `describe::HeroList`, the binary's `stats::hero_list`, the unclamped values; the
  visits and the Grail's line into the rest (T3c2 takes the Grail with the siege's buildings);
  three stills exact; `drawn`; `check.sh`.
