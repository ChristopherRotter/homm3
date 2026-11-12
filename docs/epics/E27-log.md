# E27 — log (the game shell)

Done tasks in full, oldest first. Grep a task id (`## M1 T1a`); never read it whole once it grows.

## M0 — the refinement (2026-11-12)

On `client/e27-m0` (docs only) and `shared/e27-m0` (BACKLOG), with the user; `master` at `2baf95b`
(#866 and #867 merged that morning). No original data staged.

- **Read before the questions:** `h3_game::GameSetup` and `PlayerSetup` (`crates/h3-game/src/setup.rs`,
  `town.rs`'s `PlayerStart`): the human flag, the faction's choice, the hero's pick and its offer
  (TWN-SET-006), the difficulty, `validate`; no starting bonus, handicap or victory and defeat
  (GAM-RES-003: "a computer player's bonus and the handicap (E10)"); `h3-files`' adventure replay
  format (v12) as a save; the map-client lane's HANDOFF: 8 of the 160 shipped maps import whole.
- **The clips:** the table of the installation's `Data/VIDEO.VID` (173 entries) and
  `Data/H3ab_ahd.vid` (7) listed on the user's computer by a short script reading only the
  tables (`device_bash`, the installation read only); the result in `E27-design.md` §1.
- **Asked** (four questions at once) and **answered:** CLT-Q79 (the setup: `GameSetup`, E10 adds
  what it lacks, stubbed until then, no new crate), CLT-Q80 (FFmpeg's Smacker decoder ported),
  CLT-Q81 (the screens built, the effects stubbed), CLT-Q82 (four milestones, the campaign screens
  to E20); the agent's stated defaults kept (CLT-Q83).
- **Written:** the epic refined (decisions 4 to 8, the scope without campaigns, M1's tasks), the
  design note §1, M1's tasks in the lane's queue (the inbox's E27 entry to the archive), the
  adventure lane's inbox (E10's gaps), the map-client lane's (the hand-over, the system menu);
  BACKLOG's E27 in progress and E20 with the campaign screens (`shared/e27-m0`).

## M1 T1a1 — the catalogue, part 1a: the start, the main menu, New Game's and Load Game's menus (2026-11-12)

On `client/e27-m1-t1a1` (docs and the address map) and `shared/e27-m1-t1a1` (BACKLOG's tutorial
line), the user present; `master` at `35fc648` (#868 to #871 merged that morning). T1a cut with the
user at its start (decision 9: T1a1 here, T1a2 Multiplayer, the hot seat's names, High Scores and
Credits).

- **Staged at the start** (57 files: the four LODs one a call, the executables, the five DLLs,
  both `.snd`, `H3ab_ahd.vid`, `battle-videos.vid`, both tars, the 40 MP3s); then, while the
  computer was online, **`original-data/shell-videos.vid` made on the user's computer** from the
  installation's `VIDEO.VID` (read only: its ten shell entries in `VIDEO.VID`'s format and order,
  106 901 444 bytes, SHA-256 `f04fbb39…`; a first version with `h3ab_ahd.vid`'s `Credits.smk`
  rebuilt without it, the two names being one ignoring case) and staged. The computer went
  offline during the reading; nothing else was needed.
- **Read** (`E27-design.md` §2; 59 new map entries, 4 updated): `win_main` 0x4F7A30; `game_main`
  0x4EE3E0 (was `new_game_start_other`, E09's name from two sites) from its start to the menu loop
  and its cases; the full-screen and framed clip players (0x5972D0, 0x4EE1B0), Smacker's opener
  and lookup (0x598AF0, 0x598790), `videos_resume` (H3API's guess there wrong), `video_frame_ready`;
  the settings `Show Intro`, `First Time`, `Bink Video` and the speed test's three; the main menu
  (0x4FB2A0, 0x4FB6E0, 0x4FB710 whole), the game-type menu for New Game and Load Game (0x4D54C0,
  0x4D5910, 0x4D5940 whole), the tutorial's start, the loading screen (0x4ED350), `ProgressBar`'s
  only opener (0x557480); the clip table's 141 entries (it had six). Checked privately in the
  staged data (never committed): the clips' headers, the PCXs' and DEFs' sizes and frame counts,
  the placeholder clip's pixels against `GamSelBk.pcx`'s corner (`ffmpeg` decoded it), the text
  records' subjects (GENRLTXT, Help.txt).
- **Found:** the start's three clips run in sequence and a skip ends it; nothing skips on the first
  run; `NWCLogo` and `H3Intro` play from Bink only with `Bink Video` on (design §1 assumed always);
  `H3X1intr` plays at no constant index (E20's, presumably); the loading bar is a PCX and a DEF,
  `ProgressBar` a network transfer's (design §1 corrected); the menus open clip 33, a 4 × 4
  placeholder on Complete; New Game and Load Game share one dialog; without a CD only
  Multiplayer and Back.
- **Asked and answered:** CLT-Q84 (the tutorial: drawn, a stub; BACKLOG's line), CLT-Q85 (no
  first-run case), CLT-Q86 (the menus' clip: nothing drawn). The agent's calls (not built: the CD
  paths, the disk check, the lobby, the single instance, F4) on M1's review list.
- **Not done:** no harness run (the code was clear; T2 confirms the start's clips, §2.11). Gates:
  `scripts/exe.py check`, `cargo xtask check-exe-map`, `check-docs`, `check-lane`.

## M1 T1a2 — the catalogue, part 1b: Multiplayer, the hot seat's names, High Scores, Credits (2026-11-12)

On `client/e27-m1-t1a2` (docs and the address map), the user present; `master` at `a396c3f` (#873,
#875, #876 merged). Judged one task at its start (about 5 KB of code for Multiplayer, 5 for High
Scores, 1 for Credits); no cut needed.

- **Staged at the start**, the user's computer about to go offline: the four LODs one a call, the
  executables, the five DLLs, both `.snd`, `H3ab_ahd.vid`, `battle-videos.vid`, both tars, the 40
  MP3s, `shell-videos.vid`; besides, from the SoD folder (read only) its small `VIDEO.VID`,
  `H3bitmap.lod` and `Heroes3.exe`, and Complete's three manuals (unused). Complete's and SoD's
  `VIDEO.VID` tables listed on the computer: SoD's `CREDITS.SMK` and `.BIK` are a real clip (556 ×
  100, 12 frames at 6 a second), Complete's placeholders.
- **Read** (`E27-design.md` §2.6 to §2.9; 66 new map entries, 7 updated): `multiplayer_dialog_run`
  and `multiplayer_dialog_init` whole (the items, the two stages 0x50EFC0 and 0x50EEF0, the click
  handler 0x50F4E0 and its switch, the procedure 0x50F940); the default dialog procedure and its
  mouse move and right press (0x5FF820, 0x5FF6E0, 0x5FF790: H3API's names confirmed); the hot seat
  (0x511D40, the names dialog 0x511E20 whole, its edits' keys and ring, OK's rule, the names'
  record 0x69CA50); the network paths (TCP/IP, Host, Join, Search, the DirectPlay wrapper) only by
  their entry points, texts and pictures; `ProgressBar` a network game's save transfer
  (0x4CAFD0, 0x4CBD40). High Scores: the manager (its constructor, its load of `HiScore.dat`, the
  defaults from `CampHigh.txt`, `ResetScores`), the dialog, its draw and its procedure whole, the
  creature table 0x67F1FC. Credits whole (the scroll's three phases, its clock, `Credits.txt`'s two
  records, `video_playing`). Checked privately in the staged data (never committed): the PCXs' and
  DEFs' sizes, `Help.txt`'s and GENRLTXT's records' subjects, `CampHigh.txt`'s and `Credits.txt`'s
  layouts, the clips' headers.
- **Found:** the Multiplayer dialog has two stages (the six connection types; Host, Join, Search
  and a session list); Hot Seat only without the CD notice; Online runs `online\autorun.exe` and
  ends the game; the hot seat's OK needs two names and the names are packed; `HSANIM` and `HSLOOP`
  are a won scenario's (0x4F3F60, T1b), the High Scores screen plays no clip; the creatures are
  the adventure map's monster DEFs by score (a campaign's divided by 5), stepping every 200 ms; the
  scores live in the installation's `DATA\HiScore.dat`; the credits move 2 px per frame of the
  menus' clip and end at once without one; unused assets (`muhotchk.pcx`, `mudiaddr.pcx`,
  `mupuaddr.pcx`, `Help.txt` 558).
- **Asked and answered:** CLT-Q87 (the network modes drawn, a stub), CLT-Q88 (the credits ten
  steps a second, no clip), CLT-Q89 (Cancel's item and no check box, as the original's); decision
  10. The agent's calls (the scores' file, the name's setting, the leftover file) on M1's review
  list.
- **Not done:** no harness run (T2 confirms, design §2.11). Gates: `scripts/exe.py check`,
  `cargo xtask check-exe-map`, `check-docs`, `check-lane`, `check-stubs`, `check-rules`.

## M1 T1b1 — the catalogue, part 2a: the scenario screen, its list and information panel (2026-11-12)

On `client/e27-m1-t1b1` (docs and the address map), the user present; `master` at `21d6ba0`
(#881, #882 merged). **T1b cut in three with the user at its start** (decision 11): the scenario
screen alone is about 80 KB of code in about 110 functions, one dialog for a new game, a load and
a save; T1b1 the frame, the list and the information panel, T1b2 the players' setup, the random
map and the begin, T1b3 the system options, the menu in play and the end.

- **Staged at the start**, the user's computer about to go offline: the four LODs one a call, the
  executables, the five DLLs, both `.snd`, `H3ab_ahd.vid`, `battle-videos.vid`, both tars, the 40
  MP3s, `shell-videos.vid`; all checked by SHA-256 against the computer's. Besides, the
  installation's three saved games (`AUTOSAVE.GM1`, `AUTOSAVE.TGM`, `save1.GM1`, read only) copied
  into the user's `original-data/games/` and staged, for T2's load and save screens. The end's
  clips (`LoseGame`, `hsAnim`, `hsloop`) are already in `shell-videos.vid`; `Endgame` is a
  campaign's (E20), so T1b needs no clip beyond them.
- **Read** (`E27-design.md` §3; 74 new map entries, 1 updated; `structs.toml` `scenario_dialog`
  and `scenario_info`): the constructor's items and modes (a call summary per item: the creators'
  arguments, the conditions by mode), the run, the procedure and its keys, the click handler's
  cases 107 to 195, the right press's dispatch, the list's panel, build, header read, finish,
  sort and its six comparisons, filter, the rows' drawing, the information panel's draw,
  `scenario_select`, the flags, Begin's checks (to the load and the start), the save's name, the
  delete, the busy cursor's helper thread, the vcdesc and lcdesc loaders, the hints' table. Not
  read: the map and save header readers (0x513320, 0x4BC750), the load (0x58C290), the teams
  dialog, the library sort routines beyond their comparisons, and T1b2's parts (the slots, the
  players' rows, the random map, the start). Checked privately in the staged data (never
  committed): the PCXs' and DEFs' sizes and frames, GENRLTXT's, `ArrayTxt.txt`'s and `Help.txt`'s
  records' subjects.
- **Found:** a new game opens with no left-hand panel (a load and a save open the list); the files
  are read on the main thread with a helper thread moving the busy cursor; unusable maps and saves
  stay listed under GENRLTXT 431; the load and save rows show file names; the list starts sorted by
  the format then the name but records the size's column; four of the six keys leave ties to an
  unstable sort; a double click is any two row presses within 400 ms; the rating depends only on
  the chosen difficulty (80 to 200 %); a single load shows no allies' and enemies' flags; the save's
  extension counts the humans still playing (`.GM<n>`, `.CGM`); the original remembers the last
  map, save name and difficulty between visits (`Arrogance.h3m` and `NEWGAME` at the start).
- **Asked and answered:** CLT-Q90 (ties: a stable sort with the name, then the file name), CLT-Q91
  (both oddities fixed); decision 12. The agent's calls on M1's review list (item 3).
- **Not done:** no harness run (T2 confirms, design §3.11). Gates: `scripts/exe.py check`,
  `cargo xtask check-exe-map`, `check-docs`, `check-lane`, `check-stubs`, `check-rules`.
