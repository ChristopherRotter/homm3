# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E27, the game shell** (refined 2026-11-12, M0: `docs/epics/E27-game-shell.md`, decisions 4
  to 69). **M1 to M3 closed** (2026-11-13, -16, -19; `E27-archive.md`; the build record
  `E27-log.md`; the readings `E27-design.md` §1 to §33): the start's clips and their sound, the
  menus, the hot seat's names, the scenario screen (its list, panel, difficulty, description, the
  advanced options single and hot seat, the random map's tab, the begin: the setup validated and
  kept for E24 M5, the loading screen, play's stub `CLT-STUB-13`); `drawn` as recorded on every
  recipe the shell builds. M3's closure (T7, 2026-11-19): three fresh reviews, their fixes
  (decision 67, design §33), CLT-Q148 to CLT-Q154 (decision 68), every recipe played again.
  **M4 refined** (decision 69): around play but outside it (T1 High Scores, T2 Credits, T3 the save
  file, T4 and T5 Load Game, T6 its closure); a **new M5 in play** (the options and the menu in
  play, Save, Restart, the end of a scenario), refined once the shell hosts play (E24 M5); a save
  the engine's adventure replay extended with the setup's choices (CLT-Q156 (a): the adventure
  lane's inbox; M4 T3 asks the user to do it cross-lane should it not be built). **M4 T1a, T1b
  and T2 done** (2026-11-19, decision 70: T1 cut in two; High Scores whole, Reset's question, the
  window's `scores.toml`; the credits; design §34 to §36). **Next: M4 T3** (the save file; the
  queue: the adventure lane has not built the replay's setup choices yet, so ask the user first).
  Decisions to carry (all: the epic's list): the dialogs' input one implementation (20; `h3_ui::input`); the
  shell's code as the battle's (21); every pixel of a Smacker clip through its palette (24); every
  clip's sound at the effects setting (27); a menu opening anew takes the hover (40); the list
  and the description drawn whole in the client's profile (48, CLT-Q148 (b)); the client's
  profile: a map's name and description whole (CLT-Q151 (b)), the advanced rows made again when
  the map changes (CLT-Q152 (b)); a press's stale edges as CLT-Q118's (§34.2); the credits'
  background's key pixel as a drawing bug (§36.3). Open pull requests: T1b's #1060
  (`client/e27-m4-t1b`) and T2's, stacked on it (`client/e27-m4-t2`; once #1060 squash-merges,
  `git rebase --onto origin/master` its old head): watch both until they merge. E17 done (`E17-archive.md`,
  `E17-log.md`; its open threads: the epic's "Placed at the closure", the queue's inbox). E28 and
  E29 are backlog sketches. Workspace `clients/` (`clients/README.md`), gates
  `clients/scripts/check.sh`; ADR-0020 (+ drafts in `adr-drafts/`), ADR-0024; the harness
  `clients/capture/README.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*`, `E27*` (since E17 closed: #866) and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first (at M1 T3: one per area, the
  shots as contact sheets). **Open:** CLT-Q158 (after Reset the creatures' frame 0; built as the
  original's, blocks nothing). Answered, waiting on other lanes: CLT-Q63 (a), Q65 (b), Q69 (a) (the queue's inbox,
  "E17's leftovers"). `CLT-STUB-1` (a server's seat lacks the other hero's Orb and Cloak),
  `CLT-STUB-4` (no Auto Combat switch in a live game's battle), `CLT-STUB-5` (a hot-seat
  battle between two of the host's humans) and `CLT-STUB-6` (the Multiplayer dialog's network
  modes), `CLT-STUB-7` and `-8` (Campaign, Tutorial), `CLT-STUB-9` (the starting bonus), `CLT-STUB-10` (the handicap), `CLT-STUB-11` (the turn's timer), `CLT-STUB-12` (the random map's generation) and `CLT-STUB-13` (play: the game kept for E24 M5) are commented in `docs/stubs.toml` until
  `check-stubs` scans `clients/` (a `shared/` change). An engine change the lane needs goes as a
  small cross-lane PR with the user's consent (#739 and #745 were). Next ids: CLT-Q159, CLT-STUB-15.
- Stage from `original-data/install/`: the four LODs (one per call), `h3maped.exe`, both `.snd`, `MP3/COMBAT01`–`04` and the end tunes, `Heroes3.exe`, the five DLLs (`MP3DEC.ASI` there too since 2026-11-17), `H3ab_ahd.vid`, and `original-data/battle-videos.vid` (the result window's clips: the harness's `VIDEO.VID`, the viewer's stand-in) and `original-data/shell-videos.vid` (E27's, 184 MB since T1b3a, a call of its own), `original-data/games/` (the installation's three saved games, copied at T1b1 for the load and save screens) and the 40 non-campaign MP3s; `cp -rn` the staged `original-data/.` into the clone. Frames stay in the sandbox.

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

- **2026-11-19, E27 M4 T2 (done; `client/e27-m4-t2`).** The PC's 64 files and the manuals
  staged (SHA-256 checked). Kept whole. Read `credits_screen` again and `pcx16_copy_rect` whole
  (§36.1); `credits-scroll` and `credits-end` played again (as kept), every frame equal to the
  reading's model (§36.2); one pixel of `mainmenu.pcx` black (§36.3). Built `shell::credits`,
  `h3_ui`'s `Draw::Part` and `Draw::Keyed`, the window's credits; the stub gone. Both recipes
  exact (5 of 5, 8 of 8), `drawn` whole as recorded; the window checked under Xvfb. Record:
  `E27-log.md`, "M4 T2".
- **2026-11-19, E27 M4 T1b (done; `client/e27-m4-t1b`).** The PC's 64 files and the manuals
  staged (SHA-256 checked). Kept whole. Read Reset's branch (§35.1: OK resets, writes, sets every
  creature's DEF and frame 0, the counters kept: CLT-Q158); `scores-reset` played again (as
  kept): the release closing the box runs the step due (§35.2). Built Reset's question, the
  tables' reset and keep, the scores file (`Tables::read`, `write`; the window's `--scores`,
  else `scores.toml`), the replay's `press`, `point`, `until-button`; CLT-STUB-14 lifted.
  `scores-reset` 10 of 10 exact, `scores-tabs` 28 of 28; the window checked under Xvfb. Record:
  `E27-log.md`, "M4 T1b".
- **2026-11-19, E27 M4 T1a (done; `client/e27-m4-t1a`).** The PC's 64 files and the manuals
  staged (SHA-256 checked). T1 cut in two (decision 70). Read the creatures' templates, the
  lookup's unbounded loop (CLT-Q157, answered (a)), the draw and the timer; `scores-tabs` played
  again under Wine (as kept). Built `h3_view::shell::scores` (both tabs, the buttons and keys, the
  defaults, the creatures' steps with the hold's rules, the original's stale edges; Reset a
  notice, CLT-STUB-14) and its adapter. `scores-tabs` 28 of 28 exact, every other recipe as
  recorded. Record: `E27-log.md`, "M4 T1a".
