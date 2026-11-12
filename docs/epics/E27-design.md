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
| A network game's save-game transfer (§2.1, §2.6; not the start's loading bar, which is a DEF) | | `ProgressBar` 0.3 |
| A won scenario's score screen (0x4F3F60, T1b; not the High Scores dialog, §2.8) | | `HSANIM` 4.5, `HSLOOP` 1.8 |
| The menus' clip 33 (§2.5; nothing drawn, CLT-Q86; its frames time the credits, §2.9) | (`CREDITS.BIK` and `.SMK` in `VIDEO.VID` are 4 × 4 placeholders, found first) | `Credits.smk` 0.03 (`H3ab_ahd.vid`, AB's; reached by edition 1 only) |
| A scenario's end (T1b) | `LOSEGAME` 4.0 | |
| The battle's result window (E17: `battle-videos.vid`) | `WIN3`, `LBSTART`, `LBLOOP`, `RTSTART`, `RTLOOP`, `surrender`, `defendall`, `defendloop`, `LOSECSTL`, `LOSECSLP` | |
| The spell book's page turns (E17: "Animate SpellBook" off in the harness) | | `PGTRNLFT`, `PGTRNRGH` |
| Campaigns (E20, decision 7): RoE's selection | `CGOOD1`–`3`, `CEVIL1`–`2`, `CNEUTRAL`, `CSECRET` | |
| Campaigns (E20): SoD's selection, AB's (`H3ab_ahd.vid`) | `Birth`, `elixir`, `hack`, `new`, `rise`, `spectre`, `unholy`; `C1ab7`, `C1db2`, `C1ds1`, `C1fl3`, `C1fw1`, `C1pf2` | |
| Campaigns (E20): the scenarios' clips | RoE's `GOOD*`, `EVIL*`, `NEUTRAL*`, `SECRET*`, `ENDGAME` (Smacker copies too); SoD's `H3x2_UAm` | AB's `H3AB*` (34), SoD's `H3x2_*` (44) |
| Elsewhere (a town's, unknown) | `TAVERN`, `TESTING` | |

So the shell (M2 to M4) needs Smacker for the start's first clip and a won scenario's score
screen (decision 5; the loading bar and the menus' clip are no Smacker clips to play, §2.1, §2.5);
`h3-bink` decodes the rest.
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
(x, y, width, height) on the 800 × 600 screen. T1a2 read Multiplayer, the hot seat's names, High
Scores and Credits (§2.6 to §2.9); T1b the scenario screen (new and load), the system options and
the end.

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
  "guaranteed" sends nearby): a network game's save-game transfer (§2.6).

### 2.2 The menu loop (`game_main` from 0x4EE842)

- **The top** (0x4EE842): `mainmenu.pcx` (340 × 600, `main_menu_picture` 0x699514) and
  `GamSelBk.pcx` (800 × 600, `game_select_background` 0x699518) loaded when not held; clip 33 in a
  dialog at (0, 0, 800, 600), playing on, not shown (§2.5); `GamSelBk.pcx` copied whole onto the
  back buffer; the cursor the arrow (0, 0).
- **The screen asked for** (`screen_request` 0x6976D8): −1 or 108 the main menu; any other id its
  case at once. Its writers in play: `adventure_key_down` 101, 102, 108 and 107,
  `adventure_command` and one more site 107; `game_samples_load` the start's (T1b).
- **The main menu's result** (the window manager's +0x38; switch 0x4EEC0D): 101 New Game, 102 Load
  Game (§2.4), 103 High Scores (videos paused, 0x4E9110: §2.8; resumed; `GamSelBk.pcx` copied
  whole; the top), 104 Credits (`mainmenu.pcx` copied, 0x4EDDA0: §2.9; the top), 105 Quit
  (both pictures freed, the clip closed, the end by 0x4F3690 with no text: *not read*), 107 the case
  at 0x4EF1BC (the restart, presumably: it holds E09's sites of `chosen_heroes` and `new_game_load`;
  *not read*), any other: play (0x4EF80B: both pictures freed, the clip closed; T1b).
- **After play:** the high scores' dialog when one is due (0x4E9110), then the top.
- A DirectPlay lobby's start (`lobby_launched` 0x69D80C) takes another path (0x4EE903: the main
  menu, then the scenario screen straight away); not built (§2.10).

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
  decision 7); 102 the multiplayer dialog (`multiplayer_dialog_run` 0x4F0590; §2.6), then the
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

### 2.6 Multiplayer (`multiplayer_dialog_run` 0x4F0590, `multiplayer_dialog_init` 0x50E050; M1 T1a2)

Read from `Heroes3.exe` (M1 T1a2, 2026-11-12), every item **read** unless marked. New Game's and
Load Game's Multiplayer (§2.4, 102) run one dialog; its result is not 0x7801 (cancelled) for the
scenario screen (New Game) or the load (Load Game) to follow. Positions here are the dialog's own
(add (173, 55) for the screen's).

- **The dialog** (built on the caller's stack, vtable 0x6400A0, `multiplayer_dialog_pointer`
  0x69CA28): (173, 55, 454, 490) over the screen as the game-type menu left it (*inferred*: no
  background of its own beyond its picture; T2 confirms), run by `dialog_run_modal`; its
  procedure (below) ends in the default one (`dialog_proc_default` 0x5FF820: a mouse move to
  `dialog_mouse_move` 0x5FF6E0, which writes the hovered item's hint, `Help.txt`'s first field,
  into the status item and sets the arrow cursor; a right press to `dialog_right_click` 0x5FF790,
  a popup of style 4 of the item's right-click text, else its hint; a command to the click
  handler, below). No music of its own ("MainMenu" plays on); a press sounds `button.wav` (the
  DEF button's own, §2.3).
- **Items**, in draw order (each DEF button frame 0, click frame 1, not closing, flags 2; the DEFs'
  group 0 of four frames, 64 × 48 unless said):

  | Id | Item | Rectangle | Notes |
  |---|---|---|---|
  | 100 | `mupopup.pcx` (454 × 490) | (0, 0, 454, 490) | flags 0x800 |
  | 102 | `muBhot.def` Hot Seat | (373, 78, 64, 48) | only without the CD notice (`cd_notice` 0x698A2C) |
  | 103 | `muBipx.def` IPX | (373, 135, 64, 48) | |
  | 104 | `muBtcp.def` TCP/IP | (373, 192, 64, 48) | |
  | 105 | `muBmodm.def` Modem | (373, 249, 64, 48) | |
  | 106 | `muBdrct.def` Direct Connect | (373, 306, 64, 48) | |
  | 101 | `mubonl.def` Online | (373, 363, 64, 48) | |
  | 107 | `muBhost.def` Host | (373, 78, 64, 48) | stage 2 |
  | 108 | `muBjoin.def` Join | (373, 135, 64, 48) | stage 2 |
  | 109 | `muBsrch.def` Search | (373, 192, 64, 48) | stage 2, TCP/IP only |
  | 124 | `muBcanc.def` Cancel (64 × 32) | (373, 424, 64, 48) | Escape; the item 16 px taller than its DEF (CLT-Q89: kept) |
  | 125 | the name's edit | (19, 436, 334, 18) | `smalfont.fnt`, colour 4, 21 characters, the text `network_default_name` 0x698817 (the registry's `Network Default Name`, saved with the settings); vtable 0x640054 (`multiplayer_name_key` 0x50ED60: Enter ignored, a change redraws the list) |
  | 127, 128 | GENRLTXT 41, 42 (the list's headings) | (43, 91, 127, 18), (173, 91, 127, 18) | `smalfont.fnt`, colour 1, centred; stage 2 |
  | 123 | the status line | (8, 465, 438, 18) | `smalfont.fnt`, colour 1, centred, background 0x20; the dialog's +0x34 (0x50ED50) names it |
  | 122 | a scroll bar | (337, 81, 16, 330) | 10 steps, blue, `multiplayer_scroll_proc` 0x50EE10 (the list's top row, redrawn) |
  | 110 to 121 | the list's 12 rows | (18, 112 + 25 *i*, 317, 22) | text items, colour 1, centred; drawn by 0x50F0F0 |
  | 129 | `mumap.pcx` (338 × 335) | (16, 77, 338, 335) | flags 0x800; over the list in stage 1 |

  Hints (`dialog_items_set_hints` 0x5FF8E0 from group 4 of `help_text_load`, records 264 to 288):
  101 to 109 `Help.txt` 265 to 273 (in the ids' order), the scroll bar 286, Cancel 288; the rows
  none. `muGstat.def` (23 × 23, 3 frames, a session's state in the list) loaded into +0x50. At the
  open, `DATA\RMT<id>SC.BIN` (0x69880A, a network transfer's file, `settings_load`'s names) is
  deleted from the installation, then stage 1.
- **Stage 1** (`multiplayer_stage_types` 0x50EFC0, at the open and after a back): `mumap.pcx`, the
  six connection types (Hot Seat when present) shown; Host, Join, Search, the headings and item 126
  hidden; the session list emptied.
- **Stage 2** (`multiplayer_stage_sessions` 0x50EEF0): `mumap.pcx` and the six types hidden; Host
  and Join and the headings shown; Search hidden (TCP/IP shows it).
- **The click handler** (`multiplayer_dialog_click` 0x50F4E0, the switch 0x50F501):
  - 102 **Hot Seat:** `hot_seat_run` 0x511D40 (§2.7); its OK closes the dialog (game type 3), its
    Cancel stays in stage 1.
  - 103 **IPX:** stage 2, `game_type` 0x6989F0 1, the network opened (`network_open` 0x554410 with
    the type and the name; `network_connect` 0x5556E0, DirectPlay's IPX service provider); on a failure
    GENRLTXT 461 and the dialog closes with 0x7801; else the sessions listed (`network_manager_pointer`
    0x69D808, +0x68) and refreshed by the procedure.
  - 104 **TCP/IP** (`multiplayer_tcp_start` 0x5113F0, *read in part*): game type 2, stage 2 with
    Search, item 126 (`bigfont.fnt`, GENRLTXT 460 with the machine's address, at (0, 16) and 50 high);
    a failure (GENRLTXT 459) closes the dialog with 0x7801.
  - 105 **Modem** (game type 5) and 106 **Direct Connect** (game type 4): stage 2 but `mumap.pcx`
    kept (no list), Host and Join shown.
  - 101 **Online:** the current directory `online`, `ShellExecuteA` of `autorun.exe` (the SoD CD's
    online services; Complete has none), then `game_shutdown` 0x4F3690: the game ends (CLT-Q87: the client's a stub, never an end).
  - 107 **Host** (`multiplayer_host` 0x50FDA0, *inferred*: the modem's own path, GENRLTXT 448 and
    449; else the host dialog 0x510060, `MuDialog.pcx`, a session name, GENRLTXT 453 by default, and
    a password, GENRLTXT 454), 108 **Join** (`multiplayer_join` 0x510F40, *inferred*: the selected
    session, its password GENRLTXT 454, 456 on a failure, 458 a wrong password), 109 **Search**
    (`multiplayer_search` 0x511660, *inferred*: `MuDialog.pcx`, the host's address GENRLTXT 179, a
    password 462, 463 when not found, 456 and 459 on failures). Their success closes the dialog;
    a failure in Host or Join goes back to stage 1.
  - 110 to 121 a row: the session at the list's top + (id − 110) selected (when it exists), the
    list redrawn.
  - 124 **Cancel:** in stage 2 the network closed (`network_close` 0x5544B0) and stage 1; in stage 1
    the dialog closes with 0x7801.
- **The procedure** (`multiplayer_dialog_proc` 0x50F940, the vtable's +0x24): the mouse's tick;
  in stage 2 the session list enumerated again each refresh period (+0x68, the connection's figure
  plus 100 ms), the scroll bar's range the count less 12, the list redrawn; then the default.
- **`ProgressBar`** (clip 63, §2.1): played only by a network game's save-game transfer,
  `network_game_send` 0x4CAFD0 (from `next_player_turn`, `next_player_turn_network`,
  `new_game_start` and one more) and `network_game_receive` 0x4CBD40 (`network_map_redraw` and one
  more): it goes with the network modes (CLT-Q87: not built).

### 2.7 The hot seat's names (`hot_seat_run` 0x511D40, `hot_seat_dialog_init` 0x511E20)

- **The run:** the dialog built on the stack (vtable 0x6401D8), run modal; closed with 0x7801 it
  answers false (the multiplayer dialog stays); else `game_type` 3 and true.
- **The dialog** (read whole): (218, 96, 363, 407), flags 0x12; items in order (positions the
  dialog's own):
  - 500 `muhotsea.pcx` (363 × 407) at (0, 0), flags 0x800;
  - 517 GENRLTXT 447 (`bigfont.fnt`, colour 4, centred) at (0, 30, 363, 150);
  - 509 to 516 eight name edits at (59, 82 + 30 *i*, 281, 18), `smalfont.fnt`, colour 4, 21
    characters, empty (vtable 0x640210), each with `Help.txt` 559 (group 15, records 558 to 565);
    linked in a ring (+0x70 the next, +0x74 the previous, the eighth's next the first);
  - 519 OK `mubchck.def` (64 × 32, four frames) at (95, 338, 64, 32), Enter (0x1C), `Help.txt` 560,
    disabled at the open (`dialog_item_enable` 0x5FE940: state 0x20, frame 2);
  - 520 Cancel `mubcanc.def` (64 × 32) at (205, 338, 64, 32), Escape, `Help.txt` 561;
  - 518 the status line (`smalfont.fnt`, colour 1, centred, background 0x20) at (10, 382, 343, 18);
    the dialog's +0x34 (0x512530) names its +0x70 (*inferred*: the status line).
  Then the first edit takes the multiplayer dialog's name (§2.6, item 125) and the focus.
- **An edit's keys** (`hot_seat_edit_key` 0x50DF60, when it has the focus): Shift+Tab and Up to the
  previous edit, Tab, Enter and Down to the next (`edit_focus_next` 0x510850, `_previous` 0x510870:
  only to an active item), any other key typed (0x5BAC50, *not read*); after each, OK is enabled
  when more than one edit holds a name (`hot_seat_edit_check` 0x50DEE0 counts the same, vtable
  +0x2C, *inferred*: at a focus change) and the dialog redrawn. Hover and right press as §2.6's
  default.
- **The click handler** (`hot_seat_dialog_click` 0x512420): 519 `hot_seat_names_store` 0x512470:
  a record of 0xAC bytes (`hot_seat_names` 0x69CA50: a count and eight names of 21 bytes) holds the
  non-empty edits in their order, the dialog closes with 0x7802; 520 closes with 0x7801. The names
  are the scenario screen's (T1b).
- Unused on Complete: `muhotchk.pcx` (20 × 22), `mudiaddr.pcx`, `mupuaddr.pcx` and `Help.txt` 558
  (a name's check box) are named nowhere in the executable (CLT-Q89: no check box).

### 2.8 High Scores (`high_scores_show` 0x4E9110; the manager 0x6993CC)

- **The manager** (`high_score_manager_create` 0x4E9070 from 0x4EDB40: vtable 0x63EB8C, +0x8D0 1):
  at the start `high_score_defaults_load` 0x5B91D0 (from `game_samples_load`) reads `CampHigh.txt`
  (rows 1 to 11 the single scenarios' defaults into 0x6A7F08, rows 13 to 23 the campaigns' into
  0x6A5ECC: player, land or campaign, days, score) and `high_scores_reset` 0x4E8FB0 (H3API's
  `ResetScores`) copies them into the two tables: +0x38 the campaigns', +0x484 the scenarios', 11
  records of 100 bytes each (+0 the player, 41 bytes; +0x29 the land or campaign, 41; +0x54 the
  score; +0x58 the days). Then `game_main` calls its +0 (`high_score_manager_load` 0x4E90A0): the
  installation's `DATA\HiScore.dat` (`high_score_file_name` 0x67F1F0), when it exists, read over
  both tables (0x898 bytes); a non-zero answer would end the game with an initialisation error, it
  always answers 0. +0x8D0 the tab shown first (1 the scenarios; the new entry's dialog writes it,
  T1b).
- **The run** (`game_main`'s case 103, §2.2): the videos paused; the dialog built on the stack
  (`high_scores_dialog_init` 0x4E9880), drawn once (`high_scores_draw` 0x4E9E50), `modal_loop` with
  `high_scores_proc` 0x4EA1D0; then destroyed; the videos resumed. No clip plays: `HSANIM` and
  `HSLOOP` (clip 35) belong to a won scenario's screen (0x4F3F60, music "Win Scenario", then the
  new entry 0x4E91D0 with `HighName.pcx`; T1b). No music of its own.
- **The dialog** (read whole; vtable 0x63EB98, `high_scores_dialog_pointer` 0x6991C0, H3API's
  `H3HiScoreDlg`): 800 × 600 at (0, 0); +0xFC the tab (1 the scenarios, 0 the campaigns) from the
  manager's +0x8D0. Four DEF buttons (42 × 200, four frames; frame 0, click frame 1, not closing,
  flags 2): 0x7802 `HiScExt.def` (726, 346, 41, 199) Enter, 0x3EB `HiScRes.def` (726, 114, 41, 199),
  0x3E9 `HiScCam.def` (31, 114, 41, 199) C, 0x3EA `HiScSta.def` (31, 346, 41, 199) S. Then per tab
  eleven creature pictures (`dialog_def_create`, 64 × 64 at (649, 26 + 50 *i*), flags 0x10, frame 0,
  hidden): ids 1004 + *i* the scenarios', 1015 + *i* the campaigns'; each the adventure map's DEF of
  the creature its row's score earns (`high_score_creature` 0x4E9830 over `high_score_creatures`
  0x67F1FC: 118 pairs of a score and a creature, the first whose score is not below the row's; a
  campaign's score divided by 5 first; the DEF the map's last monster template of that creature,
  `class_template_last` with class 54). The tab's eleven shown. Pictures `hiscore2.pcx` (the
  campaigns') and `hiscore.pcx` (the scenarios'), 800 × 600.
- **The draw** (`high_scores_draw`): Standard and Campaign lose state 0x10, the tab's button gains
  it (its frame 3); the tab's picture onto the screen's buffer; the headings in `medfont.fnt`
  (`medium_font` 0x698A0C), colour 1, alignment 5, at y 11, 26 high: GENRLTXT 434 at x 88 (58
  wide), 435 at 163 (122), the scenarios' 436 or the campaigns' 673 at 303 (210), the scenarios' 437
  at 531 (52) or the campaigns' 76 at 531 (122), the scenarios' 76 at 601 (52). The eleven rows at
  y 53 + 50 *i*: the rank *i* + 1, the player, the land or campaign, the scenarios' days and score
  or the campaigns' score, in the headings' boxes; colour 8 for the row `high_score_newest`
  0x67F1F4 names for the tab (the entry made this run, −1 none), else 1.
- **The procedure** (`high_scores_proc`): a key Escape or Enter, or 0x7802, ends it (0x7802).
  0x3EA the scenarios, 0x3E9 the campaigns (the creatures swapped), redrawn and shown. 0x3EB Reset:
  GENRLTXT 667 (style 2); OK (0x7805) resets both tables to the defaults, writes `HiScore.dat` into
  the installation's `DATA` (0x898 bytes; a failure `file_error_box` 0x4F3A60, GENRLTXT 11 with
  the name) and sets every creature's DEF again. Any other message: every 200 ms the tab's eleven
  creatures step one frame (their group 0's count wraps, +0xA4's counters), the table redrawn and
  shown. No hint line (the dialog's +0x10 is empty) and no right press's popup (*inferred*).
- **Where the scores live:** the client keeps them in its own file, never the installation's
  (the epic's rule for settings); the defaults from `CampHigh.txt` (M1's review list).

### 2.9 Credits (`credits_screen` 0x4EDDA0)

- **The run** (`game_main`'s case 104, §2.2; the videos not paused): the clip's frame redrawn
  (`video_redraw_any`), `mainmenu.pcx` (340 × 600) copied onto the screen's buffer at (460, 0) over
  the menu's buttons; the credits; the clip redrawn, `mainmenu.pcx` copied again; the menu's top.
- **The screen** (read whole): `Credits.fnt`; `Credits.txt`'s record 1 (`credits_text` 0x6A7700,
  loaded with record 2 by `credits_text_load` 0x5B9840 as a line file) wrapped to 328 px; a buffer
  of 328 × 580 saved from the screen at (460, 10) (the background); a second of 328 × (the lines'
  height + a line) with the whole text drawn at (0, 0) in colour 0x11C (`game.pal`'s entry 28,
  E17-design §3.6), alignment 5. (A buffer that cannot be made: GENRLTXT 223 and the game ends.)
- **The loop**, while a clip plays (`any_video_running` 0x597990: the menus' clip 33; with none the
  screen ends at once): the input flushed once, then per turn the mouse's tick, the messages, the
  next input message: a key down but F4 (0x3E), a left or a right press end it. On each frame of
  the clip due (Complete's `CREDITS.SMK`: one frame at 10 a second): the background put back at
  (460, 10); the text moved 2 px: first in from the window's bottom (580 px), then up until its end
  reaches the window's bottom, then out at the top; while the text's visible part is under 435 px,
  `Credits.txt`'s record 2 (`credits_closing_text` 0x6A7704) in `smalfont.fnt`, colour 0xD,
  alignment 8 (the window's bottom) over it; (460, 10, 328, 580) shown, the clip shown. When the
  text has left, the screen ends. No music change, no sound. CLT-Q88: the client steps it ten
  times a second with no clip.

### 2.10 Not built (the agent's calls, *provisional*; M1's review list)

The CD status, its notices and the menus without Single, Campaign and Tutorial (the client has no
CD; the user's installation shows none); the free disk space check; the DirectPlay lobby's start
(its path, its hidden buttons, its timed New Game); the single instance (the client's own); F4's
window mode (E29's options). From T1a2: the network modes beyond stage 1 (CLT-Q87: the five
buttons a stub; stage 2, Host, Join, Search, the session list, their dialogs, `ProgressBar`'s
transfer), the leftover `RMT*SC.BIN`'s deletion; the high scores in the installation's `HiScore.dat` (the client's own file instead, the
original's not imported); the player's name in the registry (`Network Default Name`: the client's
own setting).

### 2.11 For T2's captures

The start's clips with `Show Intro` on (the harness turns it off): the order, a skip, the
placements (`NWCLogo` through the unread 0x44DD20, `H3Intro` at (80, 187)), the end's screen call;
`Bink Video` in the harness's registry (which copies play); the menus' hover and press frames and
their popups; no CD notice; the tutorial's load (its loading screen). From T1a2: the multiplayer
dialog (over which screen, its stage 1, hover hints in its status line, a right press's popup); the
hot seat's names (the first name, OK disabled until two names, Tab's ring); High Scores on both
tabs with the creatures' steps (the defaults: the harness has no `HiScore.dat`), Reset's question;
the credits' scroll from start to end (its speed, the closing text, a key ending it).
