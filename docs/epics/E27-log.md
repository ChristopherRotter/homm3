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
- **Not done:** no harness run (the code was clear; T2 confirms the start's clips, §2.7). Gates:
  `scripts/exe.py check`, `cargo xtask check-exe-map`, `check-docs`, `check-lane`.
