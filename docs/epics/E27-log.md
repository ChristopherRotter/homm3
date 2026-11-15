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
- **Read** (`E27-design.md` §3; 76 new map entries, 1 updated; `structs.toml` `scenario_dialog`
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

## M1 T1b2 — the catalogue, part 2b: the advanced options, the random map, the begin (2026-11-12)

On `client/e27-m1-t1b2` (docs and the address map), the user present; `master` at `9c1a048`
(#885 merged). **Kept whole with the user at its start** (about 23 KB of code beside the dialog's
network handlers, about 10 KB, only noted).

- **Staged at the start**, the user's computer about to go offline: the four LODs one a call, the
  executables, the five DLLs, both `.snd`, `H3ab_ahd.vid`, `battle-videos.vid`, both tars, the
  three saved games, the 40 MP3s, `shell-videos.vid`; all 61 checked by SHA-256 against the
  computer's. No clip beyond `shell-videos.vid` is needed.
- **Read** (`E27-design.md` §3.12 to §3.18; 38 new map entries, 16 updated; `structs.toml`
  `scenario_player`, 16 `scenario_dialog` fields, 3 `game` fields): the slots (new and load), the
  records and their initialiser, the version from the slots, the rows from the map, the record
  into the game, the advanced options' show and hide and their items in the constructor, the row's
  draw and its four helpers, the click cases 207 to 270 and 279 to 352, the flag's press single
  and in the hot seat, the name's edit, the hero's step and clash, the hover, the right press's
  popups (their items), the turn duration's bar and texts, the random map's items, toggle, tab
  state and clicks, Begin's random map to its generator, `new_game_start` and
  `scenario_dialog_begin` whole, the header's town draw. Not read: the generator and the random
  map's record in full, the popups' texts' fields, where the name's edit returns its text, the
  network messages. Checked privately in the staged data (never committed): `ArrayTxt.txt`'s,
  GENRLTXT's and `turndur.txt`'s records' subjects; the 159 maps' player blocks (no human-only
  colour; no colour with placed heroes and neither flag nor main hero).
- **Found:** the human records (any colour) and the computer records (by colour); a human's
  Random town takes the town drawn at the list's read (a computer's is drawn at Begin), which a
  single player's row shows first only on a map with a human-only colour (none shipped; CLT-Q95's
  question overstated it as the general case, corrected after the answer, which stands); single player cannot set a handicap; the hero arrows act
  for humans only and skip another human's hero (as the engine's `PickTaken`); the bonus skips the
  Artifact for a colour without a hero and the Resource under a Random town, its popups state the
  amounts (E10's); on a map with a human-only colour the single player is pinned from it on; the
  hot seat's flag press leaves the holder colourless; the turn's duration (1 to 30 minutes or
  unlimited) has no engine field; the random map's Random choices use the C library's `rand`.
- **Asked and answered:** CLT-Q92 (the tab drawn, Begin a stub), CLT-Q93, CLT-Q94, CLT-Q95 (each
  fixed, the original's for the comparisons); decision 13. The inboxes told: the map-client lane
  (the hot seat's hand-over and the turn's clock, for its M5), the adventure lane's E10 note
  amended (the bonus's details). The agent's calls on M1's review list (item 4).
- **Not done:** no harness run (T2 confirms, design §3.18). Gates: `scripts/exe.py check`,
  `cargo xtask check-exe-map`, `check-docs`, `check-lane`, `check-stubs`, `check-rules`.

## M1 T1b3a — the catalogue, part 2c: the system options and the menu in play (2026-11-12)

On `client/e27-m1-t1b3a` (docs and the address map), the user present; `master` at `c7f41ee`.
**T1b3 cut in two with the user at its start** (decision 14): the system options' constructor alone
is 6.3 KB; the end of a scenario is T1b3b.

- **Staged at the start**, the user's computer about to go offline: the four LODs one a call, the
  executables, the five DLLs, both `.snd`, `H3ab_ahd.vid`, `battle-videos.vid`, both tars, the
  three saved games, the 40 MP3s, `shell-videos.vid`; all 61 checked by SHA-256 against the
  computer's. The end's clips (34 `LoseGame`, 35 `hsAnim`) are in `shell-videos.vid`; 32
  `Endgame` is a campaign's (E20). Neither installation has a `HiScore.dat`; the GOG script sets
  only `First Time`. After CLT-Q96's answer, with the computer still online,
  **`shell-videos.vid` rebuilt there** with the three Smacker copies (13 entries, 184 001 948
  bytes, SHA-256 `825b64d8…`; the ten old clips byte for byte the same; the installation read
  only) and staged.
- **Read** (`E27-design.md` §4; 13 new map entries, 16 updated; `structs.toml`
  `system_options_dialog`, the player record's +0xE4): `system_options_open`, `system_options_create`
  (a call summary; its items recomposed over `SysOpBck.pcx` in the sandbox, every item in its
  box), `system_options_run`, `system_options_proc`, the destructor, `game_save_dialog`'s end, the
  restart in part (`game_main`'s case 107, `game_reload_reset`), `settings_read`'s pairs of names
  and globals, each setting's readers. Checked privately in the staged data (never committed):
  GENRLTXT's and Help.txt's records' subjects, the DEFs' frames.
- **Found:** the dialog's 59 items (the picture, six buttons, twenty volume bars, eleven speed buttons, two
  quality buttons, six boxes, thirteen texts); the six bottom DEFs hold the pressed look in frame
  0; a chosen speed shows as a disabled button; `battle_tactics_global` is the setting Quick
  Combat (renamed `option_quick_combat`; the player record's +0xE4 a network player's copy);
  five more settings named (Autosave, which has no item, Move Reminder, Video Subtitles, Town
  Outlines, Blackout Computer, the last set with Computer Walk Speed 5); Load, Main Menu and Quit
  ask inside the dialog, Restart after it; the settings are saved only when something changed;
  the restart keeps the players' setup and the towns as first drawn and reads the map again.
- **Asked and answered:** CLT-Q96 (Low honoured), CLT-Q97 (the restart as the original's),
  CLT-Q98 (Restart asks inside the dialog); decision 15. The map-client lane's inbox told (the
  settings' readers in play). The agent's calls on M1's review list (item 5).
- **Not done:** no harness run (T2 captures, design §4.8). Gates: `scripts/exe.py check`,
  `cargo xtask check-exe-map`, `check-docs`, `check-lane`, `check-stubs`, `check-rules`.

## M1 T1b3b — the catalogue, part 2d: the end of a scenario (2026-11-12)

On `client/e27-m1-t1b3b` (docs and the address map), the user present; `master` at `7f9107f`.
**Kept whole with the user at its start** (decision 16): the cut offered (the screens after play
against victory and defeat in play, about 4.5 and 7.5 KB of code) was declined.

- **Staged at the start**, the user's computer about to go offline: the four LODs one a call, the
  executables, the five DLLs, both `.snd`, `H3ab_ahd.vid`, `battle-videos.vid`, both tars, the
  three saved games, `shell-videos.vid` (21 files, the MP3s not needed for a reading); all checked
  by SHA-256 against the computer's. Neither installation has a `HiScore.dat`.
- **Read** (`E27-design.md` §5; 36 new map entries, 6 updated, ten struct fields):
  `scenario_condition_met`, `scenario_victory_test` (a summary per case: its texts, its branches
  on the network byte), `scenario_loss_test`, `game_main` from 0x4EF80B (the campaigns' path only
  noted), the lost and won screens, the score screen and both scores, `high_score_entry` with its
  items, procedure and edit, and the helpers (`video_stop_all`, `music_service`,
  `player_display_name`, `turn_timer_start`). Checked privately in the staged data (never
  committed): GENRLTXT's records' subjects, `HighName.pcx`'s and `mubchck.def`'s sizes, the clips'
  sizes and frame counts in `shell-videos.vid`.
- **Found:** 0x6972B8 is the scenario's end flag (`scenario_ended`; the adventure manager's
  procedure ends play on it), not a network client's: E08-economy §7.2's reading of `new_day`
  follows (the adventure lane's inbox told); the score is 200 − (days + 10) / (towns + 5), + 25 for
  the Grail dug up by the player, + 25 for the standard victory, times 0.8 to 2 by the difficulty;
  the score screen's five columns; the entry's dialog (no Cancel; an empty name refused); the
  eleventh row takes every score that misses the ten; GENRLTXT 116 is formatted at every end and
  read by no code; the victory codes 11 and 12 have the executable's own English texts. **A
  correction of T1b3a:** clip 34 `LoseGame` has its table's +8 clear, so its Smacker copy always
  plays (design §1, §4.7; CLT-Q96's entry annotated).
- **Asked and answered:** CLT-Q99 (the eleventh row: copied); decision 17. The agent's calls on
  M1's review list (item 6).
- **Not done:** no harness run (T2 captures, design §5.9). Gates: `scripts/exe.py check`,
  `cargo xtask check-exe-map`, `check-docs`, `check-lane`, `check-stubs`, `check-rules`.

## M1 T2 — the harness on every catalogued screen, the reference captures (2026-11-13)

On `client/e27-m1-t2` (the capture harness and docs), the user present at the start; `master` at
`9c61958`. **Kept whole with the user at its start** (decision 18; the cut offered: the tool with
the start and the menus, then §2's other screens, §3, §4 with §5). The user asked for sub-agents
used efficiently: the tool and the `start` and `menus` groups by the session, the other six
groups by three sub-agents, two at a time, each on a Wine instance of its own.

- **Staged at the start** while the user's computer was online: 61 files (the four LODs one a
  call, the executables, the five DLLs, both `.snd`, `H3ab_ahd.vid`, `battle-videos.vid`, both
  tars, the three saves, the 40 non-campaign MP3s, `shell-videos.vid` alone), all checked by
  SHA-256 against the computer's; the computer went offline an hour in.
- **The harness:** `setup.sh` joins `battle-videos.vid` and `shell-videos.vid` into the run's
  `VIDEO.VID` (25 clips); `xdd.dll` puts `SMACKW32.DLL`'s clock on the virtual one, counts the
  clips' reads (in Bink's full-screen loop the game reads none itself), guards `BinkGetSummary`
  (its read speed overflowed on a frozen clock: "division by zero" at 0x100061C4, the game
  stopped at NWCLogo's end) and freezes the clock from the first read (`H3FROZEN`); its ack has 13
  fields. `h3drive.Game` takes `frozen`. `h3shell.py` (new): recipes by group under
  `capture/shell/`, `run` (twice, compared, `--record`), `play`, `learn`, `list`; the steps
  (`STEPS`), the agents' `settle`, `click-wait`, `wait`, `games-time` and `poke` among them;
  `test_h3shell.py` (no game).
- **Recorded** (design §6.1): 84 recipes in eight groups, 864 shots and 27 clips, every one the
  same in both runs but `start-smacker` (from H3Intro's first frame its second run's times 10 ms
  earlier, its 2 586 frames the same; listed). `start-bink` plays the whole start, 2 400-odd
  frames, in about an hour a run.
- **Found** (design §6.2, §6.3; traps into `TRAPS.md` and the runbook): the cursor drawn on a
  later tick; a step given while the game loads or before it took an input moves later times (the
  tool waits for its idle loop; a `settle` for an input that starts a load; the first screen drawn
  before the first step); a clip's frames due from the last one shown. Corrections of the
  readings: the credits' closing text only near the end (§2.9); the lost screen's 1.5 s wait in
  real time (§5.4); a single scenario's human has no name (§5.6). The victory and loss texts
  reached and not (§6.3).
- **Asked** (open, built as the original's): CLT-Q100 (a single scenario's player has no name),
  CLT-Q101 (the multiplayer dialog's stale hint), CLT-Q102 (the options' boxes and bars on a
  release off them). The agent's calls on M1's review list (item 7).
- **Not done:** the `scenario` and `advanced` groups' oddities (their agent's report was lost
  with a sandbox restart; its recipes' `what` lines hold its notes): T3's reviewer. The sandbox's
  VM restarted twice (the runs and agents ended, `--record` now keeps each recipe as it ends).
- **Gates:** the harness check (`h3adv.py batch capture/batches/harness-check.toml`),
  `test_h3shell`, `test_h3drive`, the scoped verify.

## M1 T3 — M1's closure: the fresh reviews, the questions, M2's refinement (2026-11-13)

On `client/e27-m1-t3` (docs only), the user present; `master` at `88ab663` (#900 merged). **Kept
whole with the user at its start** (decision 19; the cut offered: the review and the questions,
then the review list and M2's refinement), with sub-agents used efficiently: one recorded the
groups again, two fresh reviewers, one applied the doc fixes.

- **Staged at the start** while the user's computer was online: 64 files (the 61 of T2 and the
  installation's `README.TXT` and two `goggame` files), all checked by SHA-256 against the
  computer's. The sandbox's baseline (`verify-once.sh clients`) passed on `master`.
- **Recorded again** (`h3shell.py run`, a fresh sandbox, the harness set up anew): every recipe but
  `start-bink` and `start-smacker` (82 recipes), every shot and clip the same in both runs and as
  kept (TCP/IP's stage 2 included: the sandbox's address was the same). The VM restarted once, after
  the runs.
- **Reviewer 1** (the catalogue but §3 and its recipes; 35 findings): stale statements after T2's
  recordings fixed in place (the clip table's order, the loading bar only when a game loads, the
  credits' closing text, the hot seat's OK, the 151 guard as CLT-Q44's, the disabled state's bit);
  the Smacker copies' headers read from `shell-videos.vid` (design §2.1; `start-smacker`'s 9 s more
  are the 10 ms steps' 70 ms a frame, *inferred*); what M2 to M4 still read or record listed in
  design §6.4; the adventure lane's inbox told (E10: the hero's defeat by another team, the score).
  Its kind-4 oddities asked (CLT-Q106, CLT-Q107) or checked: the menus' redraw rectangle measured
  in `menu-new`, `menu-load` (CLT-Q118); a computer's name in its elimination is its colour's.
- **Reviewer 2** (§3 with the `scenario` and `advanced` groups' 383 shots as 76 contact sheets,
  the lost report's work; 22 findings): ten oddities asked (CLT-Q108 to CLT-Q117), seven readings
  corrected in §3 (the popups' sizes, Delete in the save mode, items 132 and 134, button 130 in a
  load, the handicap's popup, the name's edit, the list's scroll), the recipe fixes for M3 in §6.4.
- **The user's answers** (CLT-Q100 to CLT-Q118; decisions 20 to 22): CLT-Q102 as the combat
  options' CLT-Q43 (2), the dialogs' input one implementation (the user: "the code should be shared
  and not implemented twice"); the shell's code as the battle's; the clips' sound exact against the
  DLLs; the score the engine's (E10); CLT-Q107 and CLT-Q117 against the recommendation; CLT-Q116 a
  new option (the original's string, the number of town types from the enabled packs). M1's review
  list rewritten (twelve items) and accepted; M1 moved to `E27-archive.md`.
- **M2 refined** (M2 T0): eight tasks (the epic's M2 table, the queue), the loading bar moved to M3.
- **Change freely** (the agent's): the contact sheets and the crop tool (`/home/claude/tools/`,
  outside the repo); the review list's wording.

## M2 T1 — the Smacker decoder (2026-11-13)

On `client/e27-m2-t1`, the user present; `master` at `202aa3f` (#905 merged). **Kept whole with the
user** (decision 23).

- **Staged at the start**: the 64 files of M1 T3, all checked by SHA-256 against the computer's.
  The sandbox's baseline was lost to a VM restart (the coverage step running); CI's green run on
  `master` (`999bb0e`) stood for it.
- **`h3-smacker`** (pure, `no_std`, no dependency): `Clip` (the header, the frame table, the
  trees; FFmpeg's demuxer's checks), `Reader` (the frames in order: each palette change applied to
  the palette before, the audio packets skipped as FFmpeg skips a discarded stream's, a failed
  frame followed by the next from where its size ends it), `Decoder` and `Frame` (the trees, the
  cache, the four block types and `SMK4`'s two modes; `Frame::written`, the blocks a frame wrote),
  `build` (the writer: shapes, trees, palette operations, files, `Video`'s codes, `sample`),
  `check` (`frames`, `raw`, `hash`, `original_16`). 28 tests and three property tests; 100 % of
  lines and regions. FFmpeg's handling of malformed data kept (bits past a frame's end, a palette
  change failing midway, a size-0 palette change, audio packets' sizes); the `ffmpeg` command
  line's stop at a demuxer error and `avio_feof`'s end are not (the crate's header).
- **`h3-view video --smacker`** (`h3-battle-client`): a video archive's Smacker clips (default
  `shell-videos.vid`) and samples, written as `<stem>.pal8` and `<stem>.565`, compared with a
  reference. **`smackprobe.exe`** (`capture/smackprobe.c`, built by `setup.sh`), **`smacker-dll.py`**
  and **`smacker-check.sh`** (the shell's clips, AB's `Credits.smk` when staged, samples 1 to 8).
- **Checked:** against FFmpeg 6.1.1 (design §7.1); against the DLL, every frame, palette and
  5-6-5 copy of the shell's eight clips and AB's `Credits.smk`, the 5-5-5 copy of `NWCLOGO` and
  `HSLOOP`, and samples 1 to 150, byte for byte. The queue's "seven clips" were eight
  (`CREDITS.SMK`, 4 × 4, counted).
- **Found** (design §7): the DLL's 16-bit copy converts only the blocks a frame writes (CLT-Q119,
  asked and answered: fixed, decision 24); the DLL is `SMK2` only and fails on a byte tree left out
  and on a frame's key-frame mark, which the first samples had (the samples changed: every byte tree
  present, no mark); the trees' sizes are RAD's four bytes an entry plus three; the handle's palette
  at +0x6C is FFmpeg's levels.
- **The agent's calls** (*provisional*, M2's review list): items 1 to 4.


## M2 T2 — the clips' sound (2026-11-13 to 2026-11-14)

On `client/e27-m2-t2`, the user present at the start; `master` at `031a120` (#908 merged). **Kept
whole with the user** (decision 25).

- **The probes** (design §8.1): a stand-in Miles (`capture/mss32.c`, `.def`, `.rc`; built by
  `setup.sh` into the run directory's `mss/`) records every buffer a DLL loads into a sample and
  runs its timers when the probe pumps them; `smackprobe audio`, `binkprobe sound`, `sound-tables`
  and `cdft`; `smacker-dll.py` and `bink-dll.py` write `<stem>.t<k>.pcm` and `<stem>.pcm` beside
  the frames (a run helper that retries Wine's first start).
- **`h3_smacker::sound`**: FFmpeg 6.1.1's Smacker audio and PCM ported (`Samples`, `Codec`,
  `Track::{channels, bits, codec}`, `decode`); the reader hands each described track's packet
  (`Packet::audio`); `build::sound` and the samples carry a random track (a packet each frame but
  the last, so the DLL flushes); `check::sound`. The samples' hashes changed (re-checked against
  FFmpeg and the DLL).
- **`h3_bink::audio`**: the DLL's block decoder (`Track`, `Audio::new`, `Audio::decode`);
  `fft` (Ooura's `cdft` as the DLL compiled it: generated from a scratch symbolic run of the DLL's
  x87 code aligned with Ooura's C, then `makewt` by hand; CLT-Q120, decision 26); `trig` (cosines,
  sines and `pow10` correctly rounded in 124-bit fixed point); `Clip::tracks`, `Clip::audio`;
  `build::sound_block`, `sound_packet`, `sample_sound`; `check::sound`, `sound_tables`,
  `inverse_fft`. 100 % of lines and regions (`clients/scripts/check.sh`).
- **`h3_view::clip_sound`** (`ClipSound::bink`, `ClipSound::smacker`, `volume`; `h3-view` depends
  on `h3-smacker` now, the purity allowlist with it), **`HostSound::play_clip`** (a stream of its
  own, `clip_playing`, `stop_clip`), **`h3-view video`** writes each clip's sound, `--sound-tables`
  and `--play` (the clip's sound through the window's audio at the effects setting 9).
- **Checked** (design §8.2, §8.3): every track of the shell's clips, AB's `Credits.smk` (none) and
  the samples the DLLs' byte for byte; Bink's tables and 120 FFT runs on random values too.
- **Asked and answered:** CLT-Q120 (Ooura's code: ported with his credit, decision 26), CLT-Q121
  (the Bink clips' full volume: every clip at the effects setting, decision 27).
- **Found:** `h3-bink`'s video of the shell's four Bink clips is not the DLL's (the chroma, at
  most 3 levels at the first differences; design §8.4): queued as T2b, before T5. The game cuts a
  Smacker clip's queued sound at its last frame (T5).
- **The agent's calls** (*provisional*, M2's review list): items 5 to 9.

## M2 T2b — the shell's Bink clips' video as the DLL's (2026-11-14)

On `client/e27-m2-t2b`, the user present; `master` at `7573cd8` (#915 merged). One task, no cut.
The original data staged at once (64 files, checked by SHA-256 against the user's computer); the
setup's baseline (`clients/scripts/check.sh`) passed before any change.

- **Found** (design §9): the decoding was right. A scratch copy of `h3-bink` traced each block
  against the DLL's planes: the differing pixels were the decoded frame's one frame earlier, a
  whole block at a time. The game copies each frame by `BinkCopyToBuffer` without `BINKCOPYALL`,
  which copies only the 16 × 16 cells `ExpandPlane` marked changed (and the strips past the whole
  cells), and `ExpandPlane` files the chroma's marks under other cells (half a row of cells per two
  chroma block rows). A model of the copy made all fourteen game clips exact at once; the battle's
  clips are exact with it and without (no capture changes).
- **Built:** `Decoder`'s change mask (`changed`, `mark`: the DLL's indexing; `block_at` returns
  whether a block was of a type but 0), `h3_bink::Target` (`new`, `copy`, `frame`; its module doc
  the reading), `Plane::pixels_mut` and `Frame::planes_mut` (crate-internal), `check::frames_read`
  copying under `Reading::Original`, `h3-view video --original` writing the copied frames,
  `h3_view::clip::Video::decode` copying under `Bugs::Original`; `build::still_cells` (a quarter of
  the cells' luma skipped after the first frame, drawn first so the other blocks stay as they
  were) so that every sample checks the marks. `bink-check.sh` counts the shell's four clips' video
  now; the capture README's `frames` notes the copy.
- **Checked** (`bink-check.sh`, the VM restarted during its run, the rest run by hand on its
  outputs): the shell's four Bink clips (195, 2 426, 126 and 1 545 frames), the battle's ten and
  samples 1 to 8, every frame and track the DLL's; samples 1 to 3 at 256 × 120 through
  `bink-dll.py` by hand. The samples' hashes changed (the DLL's now); each sample's copied frames
  differ from its decoded ones.
- **Asked and answered:** CLT-Q122 (fix: every Bink frame shown whole, decision 28).
- **The agent's calls** (*provisional*, change freely): the names `Target`, `changed`,
  `still_cells`; a target starts at zeros as the probe's buffer; `Reading::Original` covers the
  copy as well as the residues (one switch for the comparisons); the samples' still cells one in
  four, from a source of their own.


## M2 T3a — the shell's frame: the dialogs' input shared, the state machine, `h3-view shell` (2026-11-14)

On `client/e27-m2-t3a`, the user present; `master` at `2e02c90` (#917 merged). T3 cut in two with
the user (decision 29): T3a here, T3b (start-up loading by the LODs' tables) next. The user's
computer left the bridge at the start and came back; the original data then staged at once (64
files, checked by SHA-256 against the user's computer); the setup's baseline
(`clients/scripts/check.sh`) passed before any change.

- **Built, `h3-ui`** (common: the map-client lane's inbox told): `input` (`Input`, `Held`, `Did`),
  the press, hold and release of a dialog's DEF buttons and DEF pictures and its button held by a
  key, the right release ending a button's own loop, the item a right press's help names, the let
  go of a popup's release; the original's profile clicks a picture at a release anywhere
  (CLT-Q43 (2), CLT-Q102), the client's over it alone. Moved from `h3_view::options` (decision
  20): the combat options keep their clicks, sounds and `Did`, their 20 tests unchanged and
  passing; `input`'s own 11 tests on an invented dialog.
- **Built, `h3-view`:** `shell` (`Shell`, `Screen`, `Ended`, `Refused`, `Request`, `Game`, `Choice`,
  `Kind`, `PlayEnd`): the menu loop's switches as read (design §2.2 the top and the main menu's
  switch, §2.4 New Game's and Load Game's, §2.6 and §2.7 the multiplayer dialog's close, §4.3 play's
  requests, §5.3 play's end: the lost screen, the won one, then High Scores); each screen names the
  task that builds it. `shell::text`: the screens' names, the endings' words, `describe`, `ending`,
  `answer`. 16 tests (every screen against every ending; a whole transcript).
- **Built, `h3-battle-client`:** `shell` (`Options`, `run`): `h3-view shell [--script <file>]
  [--no-intro]`, a word a line from standard input or a script (echoed; a refused word exit 1).
- **Checked:** `clients/scripts/check.sh` with the original data staged (the window's staged
  tests among it, the combat options' dialog in them), 100 % coverage of the pure crates; the
  command by hand on a script through the hot seat to a won game's High Scores and Quit.
- **The agent's calls** (*provisional*, M2's review list items 10 to 12): `Input` beside the
  dialog's `Shown`; `message::Open` and `stats::Open` keep their own button loops; the screens and
  endings as two enums checked together; the ids as discriminants; the words.

## M2 T3b — start-up loading by the LODs' tables (2026-11-14)

On `client/e27-m2-t3b`, the user present; `master` at `2996b51` (#921 merged). One task, no cut
(about a dozen files of `h3-battle-client`, one signature of the common `h3-assets`). The original
data staged at once at the start (64 files, checked by SHA-256 against the user's computer); the
setup's baseline (`clients/scripts/check.sh`) passed before any change.

- **Built, `h3-battle-client`:** `lods` (`Lod`, `Reader`, `Lods`; `SPRITES` and `BITMAPS` moved
  there from `render` and `sprite`): an archive listed from its head alone (`min(length,
  lod::MAX_TABLE_BYTES)` read once, `LodTable::parse`), an entry's stored bytes read by its range
  (`LodEntry::stored_range`) and inflated by `lod::read_entry`; `Lods::lookup` searches as
  `render::lookup` did (a DEF in the sprite archives, the rest in the bitmap ones, the base archive
  first); `Lods::batch` reads a batch's entries archive by archive in the order of their ranges,
  through one open file each, and gives them back in the names' order. The set holds paths and
  tables, no file handle (as `videos::Videos`). 3 tests on invented archives (the search lists, a
  batch's order, a missing archive, a bad table, an archive cut or gone after it was listed).
- **The readers switched:** `render::read_archives` and `render::lookup` gone. `render` (the
  still's batch by `Lods::batch`), `battle` (`Ground` holds the `Lods`, no longer the 212 MiB;
  `load_names` one batch, the clips from the video archives as before), `sequences` (the frames'
  batch), `motion` and `log` (`&Lods` for `&[(&str, LodArchive)]`), `text` (the bitmap archives'
  tables, `Lods::open_these`), `sprite` (the four tables; before, the archives read whole one after
  another until one held the entry), the binary's `situation` and `harvest` (`Lod::open` of
  `H3bitmap.lod`, then three and four text tables by their ranges). An asset a checkpoint's batch
  lacks is still loaded as it is drawn (`Battle::compose`, the cursors' DEFs): now a read of its
  range.
- **`h3-view gate`:** opens the archives as the client starts and prints it (each table's KiB of
  the archive's MiB and its µs; the total), then reads every entry by its range in the order of the
  ranges, the read, the inflation and the decoding timed apart; the duplicates compared through one
  reader an archive. `h3_assets::archive::inventory` takes a `LodTable` (common; the gate its one
  caller).
- **Measured** (release, the staged files, a warm cache; `master`'s binary built beside it): the
  start 145 ms for the four archives read whole (212 MiB) → 1.3 ms for their tables (1 250 KiB);
  every entry read by its range in 56 ms; `h3-view render scenarios/battle.toml`'s peak memory
  243 → 28 MiB, its wall time 346 → 201 ms, the same PNG byte for byte. The gate's other lines
  unchanged (11 981 entries, the 679 names, the 1 182 keys, the 1 232 sounds).
- **Checked:** `clients/scripts/check.sh` with the original data staged (the window's staged
  tests among it), 100 % coverage of the pure crates; `h3-view gate` on the staged files.
- **The agent's calls** (*provisional*, M2's review list items 13 and 14): the set's shape; each
  head read up to `MAX_TABLE_BYTES` (312 KiB an archive, as `h3-formats` advises) rather than the
  used table alone; `sprite` needing all four archives; `inventory` on a table.

## M2 T4 — the comparison, the main menu at rest, h3shell's sound, TCP/IP's mask, the window (2026-11-14)

On `client/e27-m2-t4`, the user present; `master` at `421b41b` (#923 merged). The cut offered
(the comparison and the main menu first, the harness's sound and mask next, the window to T5);
the user kept T4 whole (decision 30). The original data staged at once at the start (64 files,
checked by SHA-256 against the user's computer). The VM restarted during the setup's baseline
(files kept): CI's green run on `master` stood in for it.

- **Built, `h3-view`:** `shell::Built` (`Not`, `AtRest`, `Whole`; `Screen::built`: the main menu
  `AtRest`); `shell::look` (`ShellLayout`, `Menu`, `CursorLook`, `Look`; `parse_layout`, `look`,
  `names`) over `clients/layouts/shell.toml` (the cursor, the main menu's `GamSelBk.pcx` and its
  five DEF buttons from `main_menu_layout` 0x63FF28); `shell::replay` (`Step`, `ClipEnd`, `Input`,
  `parse`, `shown`, `Take`, `Stop`, `Replay`): a recipe's steps on the shell's own clock as
  h3shell's runs pass the frozen clock (an input then 100 ms, `until` and clips in 10 ms steps,
  `until` checked then stepped `limit / 10 + 1` times), each shot and clip frame handed to the
  caller; the harness's own steps, an unknown screen name, a screen not built and a press or a
  key on one built at rest stop it. 9 tests.
- **Built, `h3-battle-client`:** `shell` gains `--steps <file> --frames <dir>` (`Drawer`: the
  layout's assets in one batch by the LODs' tables, the background copied whole by `draw_image`,
  the dialog by `h3_ui::paint`, the arrow by `h3_ui::cursor`; each shot `<dir>/<shot>.565`, a
  clip's changed frames `<dir>/<clip>/t<ms>.565`, `replay.txt` with the stop) and `--window`
  (the built screens, the others black, the hardware cursor when SDL makes it, the words from
  standard input through a thread), `--install`, `--pointer`, `--input-step`, `--scale`.
- **Built, the harness:** `h3shell.py drawn [--group] [--only] [--record] [--out]` (no Wine):
  each recipe through `h3-view shell --steps`, every shot and clip against the original's kept
  line (`exact`; else the time, or the pixels' count and box when a `run`'s frames are there) and
  against the viewer's record `shell/drawn.txt`; a recipe's `sound = true` (h3drive's `Game(sound)`;
  each sample started kept as `<id> sound<k> t<ms> ch<c> <length>ms x<loops>`); the steps `mask`
  and `unmask` (a box left out of the later digests, the original's and the viewer's).
  `test_h3shell.py`: 5 more tests.
- **Recorded:** `mp-tcp` again with `mask:180,96,620,124` over its stage-2 shots (the sandbox's
  192.0.2.2 in rows 101 to 114; 18 lines changed, the rest `ok`); `menu-sound` (new): New Game's
  press and Back's each start a 151 ms sample at the press (t0, t100), and the menus' shots are
  the silent recipes' digests at the same times (`menu-new`'s `menu` t100, `back`): the presses do
  not wait for `button.wav`. `menu-main`, `start-plain`, `start-skip-key` and `scores-reset`
  played again with the changed `h3shell.py`: as kept. The adventure harness check not run:
  nothing it runs imports `h3shell.py`.
- **The comparison:** `drawn` over the 85 recipes: 4 shots drawn, 3 exact (`start-plain`'s,
  `menu-main`'s and `menu-sound`'s first: `GamSelBk.pcx`, the five buttons in frame 0 and the
  arrow at the parked pointer, at 0 ms, on the first build); `menu-main`'s `new-hover` drawn, not
  the original's (the hover is T6's); every other recipe stops at its first press (the main menu
  at rest, T6), at the start (T5) or at a harness step (`games-time`, `until-button`).
- **Checked:** `clients/scripts/check.sh` with the original data staged; `python3 -m unittest
  test_h3shell`; the window under Xvfb (SDL's software renderer: the main menu, then New Game's
  stand-in after its word).
- **The agent's calls** (*provisional*, M2's review list items 15 to 20): the replay's shape and
  its stops; the layout's format; `drawn`'s two comparisons and the raw frames; the window's
  stand-ins and cursor; the mask's box; the samples' lines.

## M2 T5a — the start's pictures: the clips timed and drawn, BinkGetRects, the fade (2026-11-14)

On `client/e27-m2-t5a`, the user present; `master` at `4be4d94` (#925 merged). T5 cut in two with
the user (decision 31: the pictures now, the sound in T5b). The original data staged at once at
the start (64 files, checked by SHA-256 against the user's computer). The sandbox's baseline
(`verify-once.sh clients`) died in its coverage step: CI's green Clients job on `master` stood in.

- **Recorded** (`h3shell.py run --once`, the `start` group, for the frames' times and pixels):
  every line as kept (`start-bink` in 25 min, `start-smacker` in 21 min: its second kept run's
  form). Not `--record`: nothing changed.
- **Read** (design §10; capstone over `Heroes3.exe`, `SMACKW32.DLL`, `BINKW32.DLL`): 0x44DD20
  (`NWCLogo` at the corner, the rest centred), the fade 0x602C50 → 0x6030E0 (three steps 50 ms
  apart, then black), `SmackWait` (frames due from the last let through: an unsigned comparison),
  `BinkGetRects` (a bounding rectangle cut in two up to 8 times, sorted top to bottom), the game's
  union of the rectangles in `video_show` (a later rectangle further left shortens it),
  `SmackToBufferRect` glanced at. The map: 0x44DD20 and 0x602C50 read, 0x6030E0 new, 0x5979D0's
  union.
- **Built, `h3-view`:** `shell::start` (`Part`, `Codec`, `Header`, `Pace`, `smack_period`,
  `smack_wait`, `Op`, `Start`, `Source`, `Picture`, `place`, `shown_union`); the replay's start
  (`Replay::new`'s start and its ops' log, `skips`; the start's end the main menu; a release with
  nothing held passing on a screen at rest); `Screen::Start` built whole. 13 tests.
- **Built, `h3-bink`:** `copies` (the DLL's copy of a pixel, `Target::copy` on it) and `rects`
  (`BinkGetRects`). 3 tests.
- **Built, `h3-battle-client`:** `h3-view shell` plays the start (`--video-quality high|low`; the
  replay under `Bugs::Original` with the DLL's reading, the window under `Bugs::Fixed` on
  `ticks_ns`); `Videos` reads `shell-videos.vid` after `battle-videos.vid` when the copy has no
  `VIDEO.VID`. The harness: `drawn` passes `--video-quality low` for `Bink Video = 0`.
- **The comparison** (`drawn --group start`): 11 of 12 exact; `start-smacker`'s clip differs in 23
  of 2 586 frames (`SMACKW32`'s rectangles, not built). Found on the way: a Bink comparison needs
  `Reading::Original` (two of NWCLogo's pixels); the union's arithmetic (43 of `H3INTRO.BIK`'s
  frames). `shell/drawn.txt` recorded (the start group's 11 lines new).
- **Asked and answered** (decision 32): CLT-Q123 (a skipped NWCLogo fades at the corner) and
  CLT-Q124 (the rectangles' union leaves parts of a frame unshown): both fixed, the original's for
  the comparisons. The SMACKW32 rectangles a task of their own (T5a2), the cut offered.
- **Checked:** `clients/scripts/check.sh` with the data staged; the window under Xvfb (the 3DO
  logo, NWCLogo stretched); `cargo xtask check-exe-map`.
- **The agent's calls** (*provisional*, M2's review list items 21 to 26).


## M2 T5a2 — SMACKW32's rectangles: SmackToBufferRect ported, the start exact (2026-11-15)

On `client/e27-m2-t5a2`, the user present at the start; `master` at `6b3087d` (#932 merged). Kept
whole (one task, as queued). The original data staged at once (64 files, checked by SHA-256
against the user's computer); the sandbox's baseline (`verify-once.sh clients`) green.

- **Read** (capstone over `SMACKW32.DLL`, `Heroes3.exe`; design §10): `SmackToBufferRect`
  0x100072D0, its filler 0x10007410, the list's join 0x10007680, the iterator 0x10012760 (some 90
  instructions, not 400: a walk over a byte map), `SmackDoFrame` 0x10005C70 (the filler's stage),
  the decoder 0x10011AB0 and its block routines' prologues (the map of changed spans, 16 blocks a
  byte), `SmackToBuffer` 0x10006B00 (the state allocated zeroed, the clip's place) and `SmackOpen`'s
  flags (interlaced and doubled clips); in the game, `video_show` 0x5979D0's Smacker reads (the
  first call's result unchecked) and the buffer type 0x69FE20 (0x5971F0, 0x598C11; map entry new).
- **Built, `h3-smacker`:** `Frame::spans` (the decoder's map, kept from frame to frame), `Rects`
  (`new`, `decoded`, `next`, `last`, `shown`), `check::rects`. 9 tests.
- **Built, `h3-view`:** `Picture` keeps a `Rects` per part at its place; a Smacker frame shown
  under `Bugs::Original` is the union of `Rects::shown` (`shown_union`), under `Bugs::Fixed` its
  whole rectangle as before (decision 32). 1 test; the Smacker copy test reads the back buffer
  for the original's.
- **Built, the harness:** `smackprobe rects` (`SmackToBuffer` at a place, every frame's calls);
  `smacker-dll.py` runs it at (0, 0) and (83, 187); `h3-view video --smacker` writes
  `<stem>.rects` and `<stem>.shifted.rects` and compares them line by line.
- **The comparisons:** every call of every frame of the shell's eight Smacker clips, AB's
  `Credits.smk` and the eight samples (`smacker-check.sh`) at both places as the DLL's at the first
  run (`H3INTRO.SMK` meets the palette-only frame twice and 97 frames
  without changes); `drawn --group start` 12 of 12 exact (`start-smacker`'s clip line now
  `e5023928774ec68b`, recorded).
- **Found** (design §10; all within CLT-Q124's fix, decision 32, so no new question): the iterator
  reads a word for a byte (an unchanged span joins a run before a changed one); a palette-only
  frame gives an empty rectangle at (0, 0) and then an earlier frame's run (the union from the
  buffer's corner); the box's bottom is the latest run's; a third join of one rectangle would
  make the DLL's copy fault (unreachable from the filler's order; `Rects` stops there).
- **The agent's calls** (*provisional*, M2's review list items 27 to 29).

## M2 T5b1 — the start's sound without the game's run: SMACKW32's frames by its sound, the probes' DirectSound (2026-11-15)

On `client/e27-m2-t5b1`, the user present; `master` at `8026f16` (#937 merged; #936, T5a2,
merged before it). The original data staged at once (64 files, checked by SHA-256 against the
user's computer); the sandbox's baseline (`verify-once.sh clients`) green. **The cut:** offered at
the start (the readings and the build, then the recording), the user kept T5b whole; the first
experiment (`h3shell.py run` of the 3DO logo with `sound = true`: frames at 30, 40, 50 … ms
against 350, 420, 490 … silent) showed `SMACKW32`'s frames following a real-time sound cursor
under Wine, and the user cut it at the game's run (decision 33).

- **Read** (design §11): `smacker_open_in_dialog`'s tracks and volume (0x69FE58; the 3DO logo
  on the same path), `video_open_in_dialog`'s unread 0x694D58, `sound_digital_start`'s
  `SmackSoundUseMSS` and `BinkSetSoundSystem`; in `SMACKW32` the DirectSound path (preference 15
  0 with the game's Miles), the track's buffer, the sound service, the position, `SmackWait` with
  a track, `SmackClose`'s stop, the volume table (`2000 log10(i / 127)`); in `BINKW32` `BinkWait`
  (the clock and the pause flag only) and the Miles functions `BinkOpenMiles` loads; `game_init`
  (`game_samples_load` renamed: the game's initialisation). The map: 0x4ED650, 0x5997D0 rewritten,
  0x694D58 and 0x69FE58 new.
- **Built, the harness:** `capture/vsound.h` (a stand-in DirectSound on a given clock, its log and
  its played bytes), `smackprobe wait` (`SMACKW32`'s WINMM imports at the probe's clock, its timer
  never firing, `smacker_tick` at each reading); `smacker-dll.py` runs it at 1, 5, 7, 10 and 16 ms
  steps (`<stem>.wait<step>`), `h3-view video --smacker` writes the model's and compares them line
  by line; `h3shell.py drawn` passes `--effects 5` for a `sound` recipe.
- **Built, `h3-view`:** `shell::start::SmackSound` (`new`, `wait`, `frame`, `played`,
  `heard_by`), `Start::with_sound`, `Op::Sound`, `Op::Quiet`, `Op::Closed`, `Replay::op_times`;
  `clip_sound::volume` the DirectSound path's (one lower from the setting 5). 4 tests.
- **Built, `h3-battle-client`:** `h3-view shell --effects <0..9>`; the window opens the host's
  sound (none: silent, the Smacker clips on `SmackWait`'s clock), decodes each clip's track on a
  thread, plays it at `Op::Sound`, stops it at `Op::Quiet` (and at `Op::Closed` under
  `Bugs::Original` only); the replay lists `sound`, `quiet` and `closed` with their times.
- **The comparisons:** `smacker-check.sh`: every `wait` line of the six game clips with a track
  (`3DOLOGO`, `NWCLOGO`, `H3INTRO`, `HSANIM`, `HSLOOP`, `LOSEGAME`) at the five steps as the
  DLL's at the model's first run; the played bytes the track's first at every step; the synthetic
  samples' tracks run the DLL's buffer dry (its underrun path, not modelled): their wait lines
  are not compared. The window under Xvfb with SDL's disk driver: the 3DO logo's sound from 0.2 s
  after the device opens, NWCLogo's after the fade.
- **Asked and answered:** CLT-Q125 (the last frame's sound cut at a clip's close: the client
  plays the track out, decision 34).
- **The agent's calls** (*provisional*, M2's review list items 30 to 33).

## M2 T5b2 — the start's sound on the game's run: xdd.dll's devices, Bink's first buffer (2026-11-15)

On `client/e27-m2-t5b2`, the user present at the start; `master` at `ba827cc` (#945 merged). The
original data staged at once (64 files, checked by SHA-256 against the user's computer). The
sandbox's VM restarted during the baseline's coverage step: CI's green `Clients` run on `ba827cc`
stood in. **The cut:** offered in two (Smacker, then Bink), the user kept T5b2 whole.

- **Read** (design §12): `SmackSoundUseMSS` whole (Miles' 24 functions loaded at its first call,
  preference 15, the DirectSound path) and the DirectSound open's timer (Miles' with `AIL_lock`
  loaded, else `timeSetEvent(75, 25)`); `BinkSetSoundSystem` (the open called at once),
  `BinkOpenMiles` (20 Miles functions by name) and its sound system (two buffers of Miles' least
  size, volume 127, no `AIL_start_sample`), the sound service (while the ring holds more than a
  buffer and Miles gives one), its callers (`BinkDoFrame` before and after the audio,
  `BinkCopyToBuffer`, `BinkWait`, Bink's thread), `BinkDoFrame`'s audio into the ring.
- **Built, the harness:** `xdd.dll`'s 7. (`vsound.h` handed to `SmackSoundUseDirectSound` in
  place of `SmackSoundUseMSS`, `SMACKW32`'s `timeSetEvent` and `timeKillEvent` stubs; Bink's 11
  Miles functions answered through its `GetProcAddress` import, the buffers on the virtual
  clock; `clock/clipsound`; the ack's fields 14 and 15), `-static-libgcc` for it (`vsound.h`'s
  64-bit divisions would import `libgcc_s_dw2-1.dll` and the game never start); `vsound.h`'s
  cursor without a played file and its brief log; the stand-in Miles' `MssMinimum`, `MssHold`
  (its always-free buffers loop Bink's service for ever), `MssLoads`; `binkprobe start`;
  `bink-dll.py`'s `<stem>.start` and `bink-check.sh`'s check (the first load in the first frame,
  at the harness's 36 864 bytes); `h3drive.py`'s 15 fields; `h3shell.py`'s voices
  (`clip_voices`, `check_clip_sound`, `kept_line`'s `voice`, `drawn`'s from the replay), 3
  tests; the recipes `start-bink-sound`, `start-smacker-sound`, `start-skip-key-sound`,
  `start-skip-left-sound`.
- **Built, the client:** the replay's `sound` lines with the volume (`clip_sound::volume` of the
  clip's format) and its `closed` lines; `h3_view::shell::start`'s Bink sound from its first
  frame no longer *provisional* (design §12.2).
- **The comparisons:** BINK_RESULTS
- **The agent's calls** (*provisional*, M2's review list items 34 to 37).
