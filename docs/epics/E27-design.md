# E27 — design note (the game shell)

Working detail of `docs/epics/E27-game-shell.md`: the readings, the clips, layouts. Grep a section
number or a name; never read it whole once it grows.

## 1. The video archives (M0, 2026-11-12)

Listed on the user's computer from the installation's `Data/VIDEO.VID` (173 entries, 629 MB)
and `Data/H3ab_ahd.vid` (7 entries, 1.6 MB), read only: a `u32` count, then per entry a 40-byte
name and a `u32` offset; an entry's size is up to the next offset. Many clips ship twice, as
Bink (`.bik`) and Smacker (`.smk`); the game opens the Bink version first (`video_open`
0x44D5A0), so a clip's Smacker copy plays only where no Bink one exists. Sizes rounded, in MB.

| Use (as known at M0; T1a and T1b read where each plays) | Bink | Smacker only |
|---|---|---|
| The start (§2.1) | `NWCLOGO` 4.2, `H3INTRO` 54.5 | `3DOLOGO` 0.9 |
| Not at the start (§2.1): AB's intro, by no constant index (a campaign's prologue, presumably: E20) | `H3X1intr` 31.8 | |
| A network transfer's progress (§2.1, inferred; not the start's loading bar, which is a DEF) | | `ProgressBar` 0.3 |
| High scores | | `HSANIM` 4.5, `HSLOOP` 1.8 |
| The menus' clip 33 (§2.5; nothing drawn, CLT-Q86) | (`CREDITS.BIK` and `.SMK` in `VIDEO.VID` are 4 × 4 placeholders, found first) | `Credits.smk` 0.03 (`H3ab_ahd.vid`, AB's; reached by edition 1 only) |
| A scenario's end (T1b) | `LOSEGAME` 4.0 | |
| The battle's result window (E17: `battle-videos.vid`) | `WIN3`, `LBSTART`, `LBLOOP`, `RTSTART`, `RTLOOP`, `surrender`, `defendall`, `defendloop`, `LOSECSTL`, `LOSECSLP` | |
| The spell book's page turns (E17: "Animate SpellBook" off in the harness) | | `PGTRNLFT`, `PGTRNRGH` |
| Campaigns (E20, decision 7): RoE's selection | `CGOOD1`–`3`, `CEVIL1`–`2`, `CNEUTRAL`, `CSECRET` | |
| Campaigns (E20): SoD's selection, AB's (`H3ab_ahd.vid`) | `Birth`, `elixir`, `hack`, `new`, `rise`, `spectre`, `unholy`; `C1ab7`, `C1db2`, `C1ds1`, `C1fl3`, `C1fw1`, `C1pf2` | |
| Campaigns (E20): the scenarios' clips | RoE's `GOOD*`, `EVIL*`, `NEUTRAL*`, `SECRET*`, `ENDGAME` (Smacker copies too); SoD's `H3x2_UAm` | AB's `H3AB*` (34), SoD's `H3x2_*` (44) |
| Elsewhere (a town's, unknown) | `TAVERN`, `TESTING` | |

So the shell (M2 to M4) needs Smacker for the start's first clip and the high scores (decision 5;
the loading bar and the menus' clip are no Smacker clips to play, §2.1, §2.5); `h3-bink` decodes the rest.
The shell's clips: `original-data/shell-videos.vid` on the user's computer and staged (E27 M1 T1a1,
2026-11-12; `VIDEO.VID`'s ten entries `3DOLOGO.SMK`, `CREDITS.BIK`, `CREDITS.SMK`, `H3INTRO.BIK`,
`H3X1intr.bik`, `HSANIM.SMK`, `HSLOOP.SMK`, `LOSEGAME.BIK`, `NWCLOGO.BIK`, `ProgressBar.smk`, in
`VIDEO.VID`'s format and order, 106 901 444 bytes, SHA-256 `f04fbb39…`; `h3ab_ahd.vid` goes to the
harness whole, its `Credits.smk` and `VIDEO.VID`'s `CREDITS.SMK` being one name ignoring case). The
Smacker copies of `NWCLOGO` and `H3INTRO` are left out: with `Bink Video` on (CLT-Q85) they never play.

## 2. The start and the menus (M1 T1a1, 2026-11-12)

Read from `Heroes3.exe` (the map's entries of the task); every item **read** unless marked
*inferred* or *not read*. Texts by table and record (a record counted from 0 with the header, as
`general_text`'s rows; `Help.txt`'s by `help_text_load`'s groups); never the text. Positions are
(x, y, width, height) on the 800 × 600 screen. T1a2 reads Multiplayer, the hot seat's names, High
Scores and Credits; T1b the scenario screen (new and load), the system options and the end.

### 2.1 The start (`win_main` 0x4F7A30; `game_main` 0x4EE3E0 to 0x4EE834)

1. `win_main`: a named event keeps one instance (a second start shows a message box and ends);
   `timeBeginPeriod(1)`; `game_samples_load` 0x4ED650 (the samples; the CD status 0x69957C and the
   screen asked for at the start, 0x6976D8: *inferred*); the window class and an 800 × 600 window;
   then `game_main`.
2. `game_main`'s start: the palettes and fonts (E17-design §4); the cursor; the CD status's switch
   (0x4EE4CD): 1, 2, 5 and 6 the CD notice `cd_notice` 0x698A2C (1 also the sound off and,
   under a condition not read, GENRLTXT 249), 3 and 4 GENRLTXT 114 or 115 in a startup error box
   and the end; the sound manager started (GENRLTXT
   132 on its failure). The user's installation: status 0, nothing (the harness reaches the menu
   with no box).
3. **The first run** (the registry's `First Time`, 0x699524, 1 from `settings_first_run`), without
   the CD notice: clip 27 `Testing` full screen (Bink); Bink's counts (*not read*) become `Test Decomp`, `Test Read`, `Test Blit` (0x699528, 0x69952C, 0x699530) and
   `Bink Video` (0x6987A8) 1 when they stay below 40, 10 and 25 and an x87 timing test passes
   (0x4EE72C, *not read* in detail). CLT-Q85: the client has no first-run case.
4. **The clips** (0x4EE7A6), when `Show Intro` (0x6993C0; written 1 when the registry lacks it)
   or the first run: 28 `3DOLogo`; if it ran to its end, 29 `NWCLogo`; if that ran to its end, 30
   `H3Intro` in its frame. A skipped clip ends the sequence. After it, on the first run, `First
   Time` 0 and the settings saved. No music before the menu (the clips carry their sound).
   - **Skipping** (`video_play_fullscreen` 0x5972D0 and `video_play_framed` 0x4EE1B0 alike): a key
     down other than F4 (scan code 0x3E, the window's mode), a left press (type 8) or a right press
     (0x20) stops the clip, except on the first run (then nothing does). The input queue is flushed
     first.
   - **Which file:** an index below 28 is always Bink (0x44DD20, *not read*); from 28 Bink when the
     table's +8 is set and `Bink Video` is on (33 only for editions 2 and 3), else Smacker
     (`smacker_open_in_dialog` 0x598AF0). So `3DOLogo` (+8 clear) is always Smacker; `NWCLogo` and
     `H3Intro` are Bink with `Bink Video` on, else their `.SMK` copies. Smacker's lookup
     (`smacker_open` 0x598790) has `video_open`'s order: on Complete `Video.vid`, the CD's, then
     `h3ab_ahd.vid` (edition 1: `h3ab_ahd.vid` first).
   - **Placement:** full screen at (0, 0, 800, 600): a Smacker clip centred, `3DOLogo` (640 × 480)
     at (80, 60); a Bink clip through 0x44DD20 (*not read*; `NWCLogo` 400 × 300). The cursor hidden;
     at the end, with the table's +0xA set (28 to 31), a screen call (*not read*), else the
     clip's rectangle shown; the cursor shown.
   - **`H3Intro`** (`video_play_framed`): `introrim.pcx` (800 × 600) loaded; the clip opened in a
     dialog at (80, 187) in its own size (640 × 360), not playing on, not shown by the tick; the
     rim copied whole onto the back buffer; the screen shown; each frame due (`video_frame_ready`
     0x597930) shown (`video_show`) until the clip's end or a skip; the clip closed, the rim freed.
   - The clips (their headers): `3DOLogo` Smacker, 81 frames at 15 a second; `NWCLogo` Bink, 195
     frames at 15, sound; `H3Intro` Bink, 2 426 frames at 15, sound.
5. Then the high-score manager's start (0x6993CC's vtable +0), the cursor shown, the menu loop.
- **Not at the start:** clip 31 `H3x1intr` (AB's intro) is played by no constant index anywhere:
  a campaign's prologue by its movie number, presumably (H3API's `H3CmpPrologue`; E20, *not
  read*); clip 32 `Endgame` plays framed by `IntroRim.pcx` after a campaign's end (0x4EF9B1; E20).
- **The loading bar is no clip:** `loading_screen_open` 0x4ED350 copies `loadbar.pcx` (800 × 600)
  to (0, 0) and draws `loadprog.def` (16 × 16, 20 frames) by `load_progress_step` (E24-design §1.6:
  frame *i* at (395 + 18 *i*, 548)); here at the tutorial's load (§2.4), the scenario's begin at T1b.
  Clip 63 `ProgressBar` (Smacker, 160 × 160, 20 frames) is opened only by `progress_clip_open`
  0x557480, its frame driven by a fraction, from two functions whose callers include
  `next_player_turn`, the network turn, `new_game_start` and `network_map_redraw` (DirectPlay's
  "guaranteed" sends nearby): a network transfer's progress (*inferred*;
  T1a2 with the network modes).

### 2.2 The menu loop (`game_main` from 0x4EE842)

- **The top** (0x4EE842): `mainmenu.pcx` (340 × 600, `main_menu_picture` 0x699514) and
  `GamSelBk.pcx` (800 × 600, `game_select_background` 0x699518) loaded when not held; clip 33 in a
  dialog at (0, 0, 800, 600), playing on, not shown (§2.5); `GamSelBk.pcx` copied whole onto the
  back buffer; the cursor the arrow (0, 0).
- **The screen asked for** (`screen_request` 0x6976D8): −1 or 108 the main menu; any other id its
  case at once. Its writers in play: `adventure_key_down` 101, 102, 108 and 107,
  `adventure_command` and one more site 107; `game_samples_load` the start's (T1b).
- **The main menu's result** (the window manager's +0x38; switch 0x4EEC0D): 101 New Game, 102 Load
  Game (§2.4), 103 High Scores (videos paused, 0x4E9110: T1a2; resumed; `GamSelBk.pcx` copied
  whole; the top), 104 Credits (part of `mainmenu.pcx` copied, 0x4EDDA0: T1a2; the top), 105 Quit
  (both pictures freed, the clip closed, the end by 0x4F3690 with no text: *not read*), 107 the case
  at 0x4EF1BC (the restart, presumably: it holds E09's sites of `chosen_heroes` and `new_game_load`;
  *not read*), any other: play (0x4EF80B: both pictures freed, the clip closed; T1b).
- **After play:** the high scores' dialog when one is due (0x4E9110), then the top.
- A DirectPlay lobby's start (`lobby_launched` 0x69D80C) takes another path (0x4EE903: the main
  menu, then the scenario screen straight away); not built (§2.6).

### 2.3 The main menu (`main_menu_create` 0x4FB2A0, `main_menu_run` 0x4FB6E0, `main_menu_proc` 0x4FB710)

- A dialog at (0, 0, 800, 600) (`main_menu_vtable` 0x63FF50, `main_menu_pointer` 0x699660) with no
  picture: the loop's `GamSelBk.pcx` under it. Five DEF buttons (`def_button_create`: frame 0,
  click frame 1, a click does not close the dialog, flags 2), in this order, their rectangles from
  `main_menu_layout` 0x63FF28:

  | Id | DEF (group 0, 4 frames) | Rectangle | Hotkey (scan code) | Right press |
  |---|---|---|---|---|
  | 101 | `mmenung.def` (208 × 121) | (540, 10, 207, 121) | N (0x31) | Help.txt 3 |
  | 102 | `mmenulg.def` (227 × 121) | (532, 132, 226, 120) | L (0x26) | Help.txt 4 |
  | 103 | `mmenuhs.def` (239 × 106) | (524, 251, 239, 106) | H (0x23) | Help.txt 5 |
  | 104 | `mmenucr.def` (173 × 111) | (557, 359, 173, 110) | C (0x2E) | Help.txt 6 |
  | 105 | `mmenuqt.def` (114 × 102) | (586, 469, 114, 102) | Escape (1) | Help.txt 7 |

  The frames by the DEF button's drawer (E17-design §9.7): 0 its frame, 1 pressed, 2 disabled, 3
  selected (the hover here).
- **The run:** `play_music` "MainMenu" (`MAINMENU.MP3`), loop count 0, the effects stopped; then
  `modal_loop` with the procedure.
- **The hover** (a move, type 4): the item under the mouse (`dialog_item_id_at`) against the last
  (`main_menu_hovered` 0x67FA60); on a change the five lose state 0x10 and the new one (unless −1)
  gains it, so it draws frame 3. On a change, or a frame of the clip due, the items 101 to 105 are
  redrawn, (520, 4, 250, 567) shown, then `video_show`.
- **A press** (the DEF button's own loop, `button_press` 0x456860): frame 1, `button.wav`; the
  command (0x200/13 with its id) at the release over the button; a hotkey alike.
- **The commands:** 101 to 104 end the menu with the id. 105: the videos paused, the question
  GENRLTXT 70 (a message box of style 2: OK 0x7805, Cancel), resumed (`videos_resume` 0x597850);
  OK ends the menu with 105, Cancel stays.
- **A right press** (the message's +0xC bit 0x200, subtype 12 or 14) on 101 to 105: a popup (style
  4) of the button's Help.txt record's right-click field (`main_menu_help` 0x6A6C20).
- **The first pass** (`main_menu_disk_check` 0x67FA64, once a run): free disk space (0x50C7A0,
  *inferred*) below 5 MB gives GENRLTXT 708, and the menu ends with 105 (no question).
- **The CD notice**, once a run (the dialog's +0x4C: `cd_notice` and not yet shown): the menu
  drawn, then by the CD status 5 GENRLTXT 730 with 763, 6 730 with 746, else 107 with 747
  (`string_printf`). Not on the user's installation.
- **A lobby's start:** High Scores and Credits lose states 2 and 4 (inactive, not drawn), and 10 s after
  the menu opened it ends with 101 by itself.
- The menu ends with the procedure answering 2 and the message 0x200/10/10.

### 2.4 New Game's and Load Game's menus (`game_type_menu_create` 0x4D54C0, `_run` 0x4D5910, `_proc` 0x4D5940)

- One dialog for both at (0, 0, 800, 600) (vtable 0x63E6D8, `game_type_menu_pointer` 0x6972D8),
  its argument 0 New Game, 1 Load Game; the tutorial flag (game +0x1F69D) cleared. Before it the
  loop copies `GamSelBk.pcx` whole again.
- **Items**, in order: a picture (id 105, `newgame.pcx` or `loadgame.pcx` by
  `game_type_menu_titles` 0x6780E8, 300 × 48, at (114, 312), flags 0x800); without the CD notice
  Single (100, `gtsingl.def`, (545, 4, 209, 123), S 0x1F), Campaign (101, `gtcampn.def`, (541,
  233, 211, 131), C 0x2E) and Tutorial (103, `gttutor.def`, (545, 358, 205, 108), T 0x14); always
  Multiplayer (102, `gtmulti.def`, (568, 120, 162, 120), M 0x32) and Back (104, `gtback.def`,
  (582, 464, 127, 107), Escape). Each DEF group 0 of four frames; `game_type_menu_layout` 0x63E6B0
  holds the rectangles in the order Single, Multiplayer, Campaign, Tutorial, Back.
- **The run:** "MainMenu" asked for again (the track already playing goes on: E24-design §3.11),
  `modal_loop`. **Hover** as the main menu's (`game_type_menu_hovered` 0x6780F0; with the CD notice
  only 102 and 104 cleared); the redraw: items 100 to 105, (526, 7, 250, 560) and the picture's
  (114, 312, 300, 48) shown, `video_show`. **A press** and **its command** as the main menu's: 100
  to 104 end the menu with the id. **A right press:** Help.txt record 10 + (id − 100)
  (`game_type_menu_help` 0x6A6BF8: the file's order is the ids'), Load Game's the same records.
- **New Game's results** (switch 0x4EECF5; before the menu, 0x4EEC14 resets a game's globals, `campaign_game` 0 among them, *not read*): 100 the
  scenario screen (`scenario_new` 0x4F0490: `scenario_dialog_create` 0x579960 with 0; T1b); 101 the
  campaign selection (`campaign_selection_run` 0x4F00A0, which opens clip 33 again; E20: a stub,
  decision 7); 102 the multiplayer dialog (`multiplayer_dialog_run` 0x4F0590; T1a2), then the
  scenario screen; 103 the tutorial (below); 104 the main menu. A failure or a back from these: the
  videos resumed, `GamSelBk.pcx` again, New Game's menu again. The videos stay paused after the
  menu unless it ended with 101 or 104.
- **The tutorial** (103, 0x4EED32): the tutorial flag 1; the cursor (1, 1) (the busy one,
  *inferred*); `loading_screen_open` and a step; the map `maps` + `tutorial.tut` into game +0x1F7D4
  and +0x1F6D9; `Show Combat Mouse Hex` and `Combat Shade Level` forced on (0x6987C0, 0x6987C4); the
  players' records filled (0x4EEE2F, *not read*); `new_game_load`; on its failure
  GENRLTXT 743 and New Game's menu again, else two steps and play. The flag is read at about 45
  sites (the battle's morale, fear and Phoenix rolls and animations, `town_can_buy`,
  `town_buildable_now`, `new_month`, `starting_resources_flagged` 0x6779C8, …): CLT-Q84, the button
  a stub.
- **Load Game's results** (switch 0x4EF02D): 100 the scenario screen in its load mode
  (`scenario_load` 0x4F0610: 0x579960 with 1; T1b); 101 the same, then `campaign_game` 1; 102 the
  multiplayer dialog, then the load; 103 the tutorial flag, then the load (a saved tutorial); 104
  (and 0x7801) the main menu; a cancelled load: Load Game's menu again; a loaded game: play
  (0x4EF1AD). The videos stay paused after the menu unless it ended with 104.

### 2.5 The menus' clip 33

Opened at the loop's top (§2.2) and again by the campaign selection; `Credits` is a Smacker name
(the table's +8 clear), looked up `Video.vid` first on Complete, whose `CREDITS.SMK` is a
placeholder (4 × 4, one frame of the colour of `GamSelBk.pcx`'s corner once 5-6-5; 
`CREDITS.BIK` is one too); AB's `Credits.smk` (`h3ab_ahd.vid`, 68 × 52, 15 frames at 9 a
second) is reached by edition 1 only (E11: AB's menus show it). Its due frames make the menus
redraw their buttons (§2.3, §2.4), with no visible effect. CLT-Q86: the client draws nothing.

### 2.6 Not built (the agent's calls, *provisional*; M1's review list)

The CD status, its notices and the menus without Single, Campaign and Tutorial (the client has no
CD; the user's installation shows none); the free disk space check; the DirectPlay lobby's start
(its path, its hidden buttons, its timed New Game); the single instance (the client's own); F4's
window mode (E29's options).

### 2.7 For T2's captures

The start's clips with `Show Intro` on (the harness turns it off): the order, a skip, the
placements (`NWCLogo` through the unread 0x44DD20, `H3Intro` at (80, 187)), the end's screen call;
`Bink Video` in the harness's registry (which copies play); the menus' hover and press frames and
their popups; no CD notice; the tutorial's load (its loading screen).
