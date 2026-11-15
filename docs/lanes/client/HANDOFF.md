# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E27, the game shell** (refined 2026-11-12, M0: `docs/epics/E27-game-shell.md`, decisions 4
  to 36). **M1 closed** (2026-11-13; `E27-archive.md`; the catalogue `E27-design.md` §1 to §6).
  **M2 T1 to T5b2 done** (2026-11-13 to 15: `h3-smacker`, the clips' sound, the Bink copy, the
  shell's frame, the LODs' tables, `h3shell.py drawn`, the start's pictures and sound on the
  game's clock and run; the `start` group's 30 lines exact; design §7 to §12). **M2 T6a done**
  (2026-11-15, decisions 35, 36): the main menu takes its inputs (`h3_view::shell::menu`: hover,
  press, release off, hotkeys through the table `[shell]`, the help popups, Quit's question; the
  press samples as the sound manager plays them, `shell::samples`; the window's music); `drawn`
  compares the samples; `menu-main` 26 of 26, `menu-quit` 7 of 7, `menu-sound`'s T6a lines exact;
  design §13. **Next: M2 T6b** (New Game's and Load Game's menus, the stubs), then T7, T8.
  Decisions to carry: the dialogs' input one implementation (20; `h3_ui::input`); the shell's
  code as the battle's, the score the engine's (21); decision 22's fixes; every pixel of a
  Smacker clip through its palette (24); every clip's sound at the effects setting (27); every
  Bink frame whole, the comparisons through `Target` (28); the start's fade from the screen as
  shown, a frame's whole rectangle shown (32); a clip's track played out after its last frame
  (34); the main menu's buttons drawn in their order (36). Open pull request:
  `client/e27-m2-t6a`, T6a's (watch it until it merges). E17 done
  (`E17-archive.md`, `E17-log.md`; its open threads: the epic's "Placed at the closure", the
  queue's inbox). E28 and E29 are backlog sketches. Workspace `clients/` (`clients/README.md`),
  gates `clients/scripts/check.sh`; ADR-0020 (+ drafts in `adr-drafts/`), ADR-0024; the harness
  `clients/capture/README.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*`, `E27*` (since E17 closed: #866) and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first (at M1 T3: one per area, the
  shots as contact sheets). **Open:** none. Answered, waiting on other lanes: CLT-Q63 (a), Q65 (b), Q69 (a) (the queue's inbox,
  "E17's leftovers"). `CLT-STUB-1` (a server's seat lacks the other hero's Orb and Cloak),
  `CLT-STUB-4` (no Auto Combat switch in a live game's battle) and `CLT-STUB-5` (a hot-seat
  battle between two of the host's humans) are commented in `docs/stubs.toml` until
  `check-stubs` scans `clients/` (a `shared/` change). An engine change the lane needs goes as a
  small cross-lane PR with the user's consent (#739 and #745 were). Next ids: CLT-Q127,
  CLT-STUB-6.
- Stage from `original-data/install/`: the four LODs (one per call), `h3maped.exe`, both `.snd`, `MP3/COMBAT01`–`04` and the end tunes, `Heroes3.exe`, the five DLLs, `H3ab_ahd.vid`, and `original-data/battle-videos.vid` (the result window's clips: the harness's `VIDEO.VID`, the viewer's stand-in) and `original-data/shell-videos.vid` (E27's, 184 MB since T1b3a, a call of its own), `original-data/games/` (the installation's three saved games, copied at T1b1 for the load and save screens) and the 40 non-campaign MP3s; `cp -rn` the staged `original-data/.` into the clone. Frames stay in the sandbox.

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

- **2026-11-15, E27 M2 T6a (done; `client/e27-m2-t6a`).** T6 cut by screen with the user
  (decision 35). The original data staged at once (64 files, checked by SHA-256); the VM restarted
  in the baseline's coverage step (CI's green `master` stood in) and again in `check.sh`'s
  (TRAPS: coverage with the data staged). Read `main_menu_proc` whole, the button's loop and
  `button_press`; recorded `menu-main`, `menu-quit`, `menu-sound` again for their frames; built
  the menu, its keys, its samples, the window's input and music. Every T6a line exact; CLT-Q126
  answered (decision 36). Record: `E27-log.md`, "M2 T6a".
- **2026-11-15, E27 M2 T5b2 (done; `client/e27-m2-t5b2`).** Offered in two (Smacker, Bink), the
  user kept it whole. The original data staged at once (64 files, checked by SHA-256); the VM
  restarted in the baseline's coverage (CI's green `master` stood in). Read `SmackSoundUseMSS`,
  `BinkOpenMiles` and Bink's sound service; built `xdd.dll`'s 7., `binkprobe start`, the
  stand-in Miles' `MssHold`, the voices in `h3shell.py` and the replay. Every new recipe exact at
  its first recording. No new question. Record: `E27-log.md`, "M2 T5b2".
- **2026-11-15, E27 M2 T5b1 (done; `client/e27-m2-t5b1`).** T5b kept whole at first, then cut at
  the game's run with the user (decision 33) once a test run showed `SMACKW32`'s frames following
  a real-time sound cursor under Wine. The original data staged at once (64 files, checked by
  SHA-256); the baseline passed. Read the clips' sound path, the DLL's sound service, position and
  wait, the close and the volume; built `vsound.h`, `smackprobe wait`, `SmackSound`, the new ops,
  `--effects`, the window's sound. The six game clips' frames with sound as the DLL's at five
  steps at the first run; CLT-Q125 answered (decision 34). Record: `E27-log.md`, "M2 T5b1".
