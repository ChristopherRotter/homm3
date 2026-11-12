# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E27, the game shell** (refined 2026-11-12, M0: `docs/epics/E27-game-shell.md`, decisions 4
  to 9). **M1 T1a1 done** (the start, the main menu, New Game's and Load Game's menus, Quit:
  `E27-design.md` §2; CLT-Q84 to CLT-Q86). **Next: M1 T1a2** (Multiplayer and the hot seat's
  names, High Scores, Credits; `QUEUE.md`), then T1b, T2, T3. The shell's clips are in
  `original-data/shell-videos.vid` (design §1), so T2 needs the user's computer only for a clip T1b
  adds. Open pull requests: `client/e27-m1-t1a1` and `shared/e27-m1-t1a1` (BACKLOG's tutorial
  line); watch both until they merge. E17 done (`E17-archive.md`, `E17-log.md`; its open threads:
  the epic's "Placed at the closure", the queue's inbox). E28 and E29 are backlog sketches.
  Workspace `clients/` (`clients/README.md`), gates `clients/scripts/check.sh`; ADR-0020 (+ drafts
  in `adr-drafts/`), ADR-0024; the harness `clients/capture/README.md`; design §3–§13; records in
  `E17-log.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*`, `E27*` (since E17 closed: #866) and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first. **Open:** none of the lane's
  questions. Answered, waiting on other lanes: CLT-Q63 (a), Q65 (b), Q69 (a) (the queue's inbox,
  "E17's leftovers"). `CLT-STUB-1` (a server's seat lacks the other hero's Orb and Cloak),
  `CLT-STUB-4` (no Auto Combat switch in a live game's battle) and `CLT-STUB-5` (a hot-seat
  battle between two of the host's humans) are commented in `docs/stubs.toml` until
  `check-stubs` scans `clients/` (a `shared/` change). An engine change the lane needs goes as a
  small cross-lane PR with the user's consent (#739 and #745 were). Next ids: CLT-Q87,
  CLT-STUB-6.
- Stage from `original-data/install/`: the four LODs (one per call), `h3maped.exe`, both `.snd`, `MP3/COMBAT01`–`04` and the end tunes, `Heroes3.exe`, the five DLLs, `H3ab_ahd.vid`, and `original-data/battle-videos.vid` (the result window's clips: the harness's `VIDEO.VID`, the viewer's stand-in) and `original-data/shell-videos.vid` (E27's, 107 MB, a call of its own); `cp -rn` the staged `original-data/.` into the clone. Frames stay in the sandbox.

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

- **2026-11-12, E27 M1 T1a1 (done; `client/e27-m1-t1a1`, `shared/e27-m1-t1a1`).** T1a cut with
  the user (decision 9). The original data staged at once (57 files), then `shell-videos.vid` made
  on the user's computer and staged before it went offline. Read `game_main` (renamed), the clip
  players, the main menu and the game-type menu whole: 59 map entries. Found the clips' chain and
  first-run rules, Bink only with `Bink Video`, the loading bar a DEF, the menus' 4 × 4 clip.
  Asked: CLT-Q84 (tutorial a stub), CLT-Q85 (no first-run case), CLT-Q86 (no menu clip). No
  harness run. Record: `E27-log.md`, "M1 T1a1".
- **2026-11-12, E27 M0 (done; `client/e27-m0`, `shared/e27-m0`).** The refinement with the
  user, docs only (no original data staged; `VIDEO.VID`'s table listed on the user's computer,
  read only). Found: `GameSetup` holds most of a setup; the starting bonus, the handicap and
  victory and defeat are E10's; 8 of 160 maps import whole; Smacker-only clips at the start, the
  loading bar, the high scores and the credits. The user: the setup is `GameSetup`, stubbed
  until E10, no new crate (CLT-Q79); FFmpeg's Smacker decoder ported (CLT-Q80); the screens
  built, the effects stubbed (CLT-Q81); four milestones, campaigns to E20 (CLT-Q82); the agent's
  defaults kept (CLT-Q83). M1's tasks queued; the adventure and map-client lanes' inboxes told.
- **2026-11-12, E17 M7 T4 (done; `client/e17-m7-t4`, `shared/e17-closure`); E17 closed.** Kept
  whole by the user, the session a coordinator: four read-only reviewers (R1 to R4, two at a
  time on a worktree of `master`) and two fixers on the branch; 21 findings (five medium), all
  fixed, R2-4, R3-5, R4-2 and R4-3 by the user's decision. The original data staged at once (57
  files); the baseline's `check.sh` lost to a VM restart in coverage (CI's `master` run stood
  for it). The captures where M7 changed the battle screen afresh; items 1 to 15 accepted; the
  threads placed (E27, E24 M4 T5 and M5, E28, E29, the inboxes). Record: `E17-log.md`, "M7 T4".
