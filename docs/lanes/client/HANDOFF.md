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
  design §13. **M2 T6b1 done** (2026-11-15, decision 37: T6b cut at the stubs): New Game's and Load
  Game's menus on the same `shell::menu::Menu` (`Kind`, `Menus`; the original's stale rows,
  CLT-Q118, as `Stale` layers); `menu-new` 27 of 27, `menu-load` 27 of 27, `menu-sound` 5 of 5
  exact; `h3_sdl::Key` gained L and N; design §14. **M2 T6b2 done** (2026-11-16): the Campaign,
  Tutorial, High Scores and Credits stubs, each a notice in the client's own words over its menu
  (`Built::Stub`, `menu::STUBS`; the replay stops at them), the client's own pack
  `clients/text/en.json` asked between the player's and the original's (`text::own_pack`, the
  gate's check); every recipe as recorded; design §15. **M2 T7 done** (2026-11-16, kept whole:
  decision 38): the Multiplayer dialog's first stage and the hot seat's names
  (`h3_view::shell::multiplayer`, `menu::Front`, `menu::Hovers`), the edit field (`h3_ui::edit`,
  `Draw::Edit`, the caret in `h3_render::text`), the window's scan codes (`h3_sdl::Window::scans`),
  the network modes' notice (CLT-STUB-6) and the scenario screen's stub; `edit-keys` and
  `hotseat-long` recorded, the `multiplayer` group's 113 drawn shots exact; CLT-Q128 fixed
  (decision 40); design §16. **Next: M2 T7b** (the edits typing as the keyboard prints, decision
  39, CLT-Q127), then T8 (M2's closure).
  Decisions to carry: the dialogs' input one implementation (20; `h3_ui::input`); the shell's
  code as the battle's, the score the engine's (21); decision 22's fixes; every pixel of a
  Smacker clip through its palette (24); every clip's sound at the effects setting (27); every
  Bink frame whole, the comparisons through `Target` (28); the start's fade from the screen as
  shown, a frame's whole rectangle shown (32); a clip's track played out after its last frame
  (34); the main menu's buttons drawn in their order (36). Open pull request:
  `client/e27-m2-t7`, T7's (watch it until it merges). E17 done
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
  `CLT-STUB-4` (no Auto Combat switch in a live game's battle), `CLT-STUB-5` (a hot-seat
  battle between two of the host's humans) and `CLT-STUB-6` (the Multiplayer dialog's network
  modes) are commented in `docs/stubs.toml` until
  `check-stubs` scans `clients/` (a `shared/` change). An engine change the lane needs goes as a
  small cross-lane PR with the user's consent (#739 and #745 were). Next ids: CLT-Q129, CLT-STUB-7.
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

- **2026-11-16, E27 M2 T7 (done; `client/e27-m2-t7`).** Kept whole (decision 38). The original
  data staged at once (64 files, checked by SHA-256); the baseline stopped by a watcher before
  coverage. Read the edit whole and the text drawer's caret; recorded the group again with
  `edit-keys` and `hotseat-long` (a run failed on the game's disk check: TRAPS); built the edit,
  the dialog, the hot seat, the notices; every drawn shot exact after the shadow, the status
  lines, the hover after a hold and the menus' statics. CLT-Q127, CLT-Q128 asked and answered
  (b): CLT-Q128 fixed here, CLT-Q127 queued as T7b. Record:
  `E27-log.md`, "M2 T7".
- **2026-11-16, E27 M2 T6b2 (done; `client/e27-m2-t6b2`).** Kept whole. The original data staged
  at once (64 files, checked by SHA-256), copied into the clone right after the setup started;
  its baseline stopped before coverage (TRAPS), CI's green `master` on the same commit stood in.
  Built the stubs on T6b1's menu (a stub's screen is its menu opened anew under a style-1 notice)
  and the client's own pack with its check; `drawn` all as recorded; the four notices checked in
  the window. No new question. Record: `E27-log.md`, "M2 T6b2".
- **2026-11-15, E27 M2 T6b1 (done; `client/e27-m2-t6b1`).** T6b cut at the stubs with the user
  (decision 37). The original data staged at once (64 files, checked by SHA-256); the baseline
  failed on the client's files not yet copied (step 3 comes after the setup), then the VM
  restarted (CI's green `master` stood in). Read `game_type_menu_proc` again; recorded `menu-new`,
  `menu-load`, `menu-sound` again; built the game type menus on one `Menu`, the stale rows, the
  keys, the window; every T6b1 line exact at the first run. Found and fixed the window's missing N
  and L. No new question. Record: `E27-log.md`, "M2 T6b1".
