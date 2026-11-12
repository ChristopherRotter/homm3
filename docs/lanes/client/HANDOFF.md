# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E27, the game shell** (refined 2026-11-12, M0: `docs/epics/E27-game-shell.md`, decisions 4
  to 17). **M1 T1a1 to T1b3b done** (the start, the menus, Multiplayer and the hot seat's names,
  High Scores, Credits: `E27-design.md` §2; the scenario screen: §3; the system options, the menu
  in play, the save from play and the restart: §4; the end of a scenario: §5; CLT-Q84 to
  CLT-Q99). **Next: M1 T2** (the harness on every screen, the reference captures; `QUEUE.md`),
  then T3. The shell's clips are all in `original-data/shell-videos.vid` (design §1; 13 clips;
  T1b3b added none). Open pull request: `client/e27-m1-t1b3b` (watch it until it merges). E17 done
  (`E17-archive.md`, `E17-log.md`; its open threads: the epic's "Placed at the closure", the
  queue's inbox). E28 and E29 are backlog sketches. Workspace `clients/` (`clients/README.md`),
  gates `clients/scripts/check.sh`; ADR-0020 (+ drafts in `adr-drafts/`), ADR-0024; the harness
  `clients/capture/README.md`; design §3–§13; records in `E17-log.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*`, `E27*` (since E17 closed: #866) and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first. **Open:** none of the lane's
  questions. Answered, waiting on other lanes: CLT-Q63 (a), Q65 (b), Q69 (a) (the queue's inbox,
  "E17's leftovers"). `CLT-STUB-1` (a server's seat lacks the other hero's Orb and Cloak),
  `CLT-STUB-4` (no Auto Combat switch in a live game's battle) and `CLT-STUB-5` (a hot-seat
  battle between two of the host's humans) are commented in `docs/stubs.toml` until
  `check-stubs` scans `clients/` (a `shared/` change). An engine change the lane needs goes as a
  small cross-lane PR with the user's consent (#739 and #745 were). Next ids: CLT-Q100,
  CLT-STUB-6.
- Stage from `original-data/install/`: the four LODs (one per call), `h3maped.exe`, both `.snd`, `MP3/COMBAT01`–`04` and the end tunes, `Heroes3.exe`, the five DLLs, `H3ab_ahd.vid`, and `original-data/battle-videos.vid` (the result window's clips: the harness's `VIDEO.VID`, the viewer's stand-in) and `original-data/shell-videos.vid` (E27's, 184 MB since T1b3a, a call of its own), and `original-data/games/` (the installation's three saved games, copied at T1b1 for the load and save screens); `cp -rn` the staged `original-data/.` into the clone. Frames stay in the sandbox.

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

- **2026-11-12, E27 M1 T1b3b (done; `client/e27-m1-t1b3b`).** Kept whole with the user
  (decision 16). The original data staged at once (21 files, checked by SHA-256) while the user's
  computer was online. Read victory and defeat in play (the eliminations, the conditions' tests and
  messages per code), `game_main` after play, the lost and won screens, the score and its screen,
  the high score's entry and `HiScore.dat`'s write: 36 new map entries, 6 updated, ten struct
  fields. Found: 0x6972B8 is the scenario's end flag (renamed; the adventure lane's inbox told);
  the score's formula; the eleventh row takes every score that misses the ten; `LoseGame` always
  plays its Smacker copy (a correction of T1b3a). Asked: CLT-Q99 (decision 17). No harness run.
  Record: `E27-log.md`, "M1 T1b3b".
- **2026-11-12, E27 M1 T1b3a (done; `client/e27-m1-t1b3a`).** T1b3 cut in two with the user
  (decision 14). The original data staged at once (61 files, checked by SHA-256); after CLT-Q96,
  `shell-videos.vid` rebuilt on the user's computer with three Smacker copies and staged. Read the
  system options whole (opener, constructor, run, procedure), the save from play's end, the
  restart in part and where each setting lives and is read: 13 new map entries, 16 updated, a
  struct. Found: `battle_tactics_global` is Quick Combat (renamed); Autosave has no item; Restart
  asks after the dialog closed. Asked: CLT-Q96 to CLT-Q98 (decision 15). No harness run.
  Record: `E27-log.md`, "M1 T1b3a".
- **2026-11-12, E27 M1 T1b2 (done; `client/e27-m1-t1b2`).** Kept whole with the user. The
  original data staged at once (61 files, checked by SHA-256) while the user's computer was
  online. Read the players' slots and records, the advanced options, a row's draw and presses,
  the popups, the hover, the turn's duration, the random map's tab and Begin to the game's start
  (`new_game_start`, `scenario_dialog_begin`): 38 new map entries, a struct. Found: a human's
  Random town is the one drawn at the list's read; the bonus, the handicap and the timer have no
  `GameSetup` field; two setup oddities. Asked: CLT-Q92 to CLT-Q95 (decision 13). No harness run.
  Record: `E27-log.md`, "M1 T1b2".
