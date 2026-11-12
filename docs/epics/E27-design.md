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
Scores and Credits (§2.6 to §2.9); T1b1 the scenario screen's frame, list and information panel
(§3), T1b2 its players' setup and random map, T1b3 the system options and the end (§4).

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

## 3. The scenario screen (M1 T1b1, 2026-11-12)

Read from `Heroes3.exe` (the map's entries of the task), every item **read** unless marked; the
human player's single and hot-seat paths followed, the network ones (a host relaying clicks and
setups, a guest waiting for the host's headers, the chat) only noted. Texts by table and record
(GENRLTXT's, `ArrayTxt.txt`'s and `Help.txt`'s records as §2's); positions (x, y, width, height)
on the 800 × 600 screen. T1b2 added the advanced options, the random map's tab and the game's
begin (§3.12 to §3.18); T1b3 the system options, the menu in play and the end (§4).

### 3.1 The dialog and its modes (`scenario_dialog_create` 0x579960)

- **One dialog, three modes**, its argument: 0 a new game (`scenario_new` 0x4F0490), 1 a load
  (`scenario_load` 0x4F0610), 2 a save from play (`game_save_dialog` 0x418160, T1b3); also the
  lobby's start (0x4EE9CC, not built). It is built on the caller's stack (0x1970 bytes, H3API's
  layout; `structs.toml` `scenario_dialog`), vtable 0x641CAC: +0x18 the run
  (`scenario_dialog_run` 0x584BF0), +0x24 the procedure (`scenario_dialog_proc` 0x587D00), +0x2C
  the right press (`scenario_dialog_right_click` 0x5822D0); `select_scenario_dialog_pointer`
  0x69FBE8. The record's +0x64 is the load mode, +0x65 the save mode.
- **Rows:** `scenario_rows` 0x69FDC8 is 18, 16 in the save mode (its name's strip below them).
- **The campaign's saves** (`campaign_saves_flag` 0x69FD90): 1 when the menu that opened the
  screen ended with 101 (Load Game's Campaign, §2.4) or `campaign_game` is set; it picks the
  campaigns' pattern (§3.3; E20).
- **The busy cursor:** while the files are read, a helper thread (`busy_cursor_thread` 0x577740,
  started by the constructor and by `busy_cursor_start` 0x577790, stopped by `busy_cursor_stop`
  0x577810) polls the mouse every 41 ms so the busy cursor (1, 1) moves; the arrow comes back at
  the stop. The client reads on its own terms (not built, §3.10).
- **The background:** `random_between(1, 100)` below 51 gives `gamselb1.pcx`, else `gamselb0.pcx`
  (800 × 600, 16-bit, item 100 at (0, 0)), copied into the screen at once; `GSelPop1.pcx` (370 ×
  585, item 100 at (396, 6), the right-hand panel) drawn into it too. The draw is the client's own
  randomness (a look, no rule).
- **Kept between visits** (statics, not saved across runs): the last map's file name
  (`scenario_last_map` 0x683238, `Arrogance.h3m` at the start), the last save's name
  (`scenario_last_save` 0x68333C, `NEWGAME.gm1` at the start, cut at its first full stop to
  `NEWGAME` by the constructor's `strtok`) and the last difficulty (`scenario_last_difficulty`
  0x683454). The list selects the entry of that name when it is built (§3.3).
- **At the end of the constructor:** load mode, the players' slots from the save (0x57F330); new
  game, the slots from the hot seat's names or the player's name (0x57F740; both §3.12); the
  players' rows (0x584C40, §3.12); then by mode: a load (not a network one) and a save open the list (§3.3);
  a new game (and a network load) opens with no left-hand panel (the background shows there)
  until a button opens one; the three panel buttons and the difficulty shown
  (`scenario_buttons_show` 0x57FB90); hints (§3.8); `chosen_heroes` 0x69FB24 all −1 and 0x69FBF8's
  eight values 3 (`scenario_bonus_choices`, the bonuses Random; §3.17); in the save mode the name's edit takes the focus and,
  with an empty list, `scenario_select`(−1); in the load and save modes the five difficulty buttons
  are disabled (they show the saved game's).

### 3.2 The items (the constructor's order; hidden ones marked)

| Id | Item | Rectangle | Notes |
|---|---|---|---|
| 101 | `SCSelBck.pcx` (575 × 585) | (3, 6, 575, 585) | the list's panel, hidden; its right part lies under `GSelPop1.pcx` (*inferred*: index 0 transparent there; T2) |
| 361 | text, `medfont.fnt`, colour 8, align 5 | (25, 23, 367, 23) | the title by mode: `ArrayTxt.txt` 229 new, 230 load, 231 save (`scenario_titles` 0x6A8098, filled by `arraytxt_load`); hidden with the list |
| 102, 103 | `AdvOptBk.pcx`, `RanMapBk.pcx` (575 × 585) | (3, 6, 557, 585) | the advanced options' and the random map's panels, hidden; only for a new game or a network or hot-seat load (§3.13, §3.16) |
| 132 | text "%s:" GENRLTXT 493 (the player's difficulty) | (414, 435, 334, 19) | `smalfont.fnt`, colour 2, align 5 |
| 133 | "%s:" GENRLTXT 219 (the rating) | (665, 435, 84, 19) | as 132 |
| 134 | GENRLTXT 495 (the map's difficulty) | (414, 435, 90, 19) | as 132; over 132's place (which shows is not read: T2) |
| 100 | GENRLTXT 496 (the name's label) | (422, 27, 278, 18) | colour 2, align 4 |
| 105 | GENRLTXT 497 (the description's label) | (422, 137, 278, 18) | as 100 |
| — | scrollable text, `smalfont.fnt`, colour 4 | (422, 155, 319, 115) | the description (the record's +0x196C) |
| 100 | GENRLTXT 498, 499 (victory's and loss's labels) | (422, 288, 278, 18), (422, 344, 278, 18) | as 100 |
| 131 | caption button `gspbut2.def` (128 × 20) | (622, 81, 128, 20) | network games only: the chat's switch (not built) |
| 189 | `scnrmpsz.def` (29 × 23, 5 frames) | (714, 28, 29, 23) | the map's size: frame 0 to 3 for 36, 72, 108, 144, else 4 |
| 100, 386 | "%s:" GENRLTXT 391 (allies), 392 (enemies) | (414, 403, 44, 23), (579, 403, 58, 23) | colour 4, align 6 |
| 112–119, 120–127 | `itgflags.def` (15 × 20, 8 colours) | (460 + 15 i, 405), (640 + 15 i, 405) | the allies' and the enemies' flags, hidden (§3.5) |
| 387 | an area (0x575220) | (456, 402, 310, 25) | the flags' strip, its right press (§3.8); new game or network or hot-seat load |
| 142–159 | text rows, `smalfont.fnt` | (57, 122 + 25 i, 314, 25) | `scenario_rows` of them, hidden; the list draws its own rows (§3.4) |
| 388 | `gsstrip.pcx` (399 × 65) | (3, below the rows, 399, 65) | save mode only |
| 160 | edit, max 61, `smalfont.fnt`, colour 4, align 4 | (35, 542, 342, 21) | save mode only: the name (`NEWGAME` first; the record's +0x380) |
| 137–141 | `scsmbut`, `scmdbut`, `sclgbut`, `scxlbut`, `scalbut.def` (44 × 33) | (161 + 47 i, 52) | the size filters S, M, L, XL, All; hidden with the list |
| 190–195 | `scbutt1`, `scbutt2`, `ScButCp`, `scbutt3`, `scbutt4`, `scbutt5.def` | (26, 92, 31, 28), (58, 92, 32, 28), (91, 92, 32, 28), (124, 92, 184, 28), (309, 92, 32, 28), (342, 92, 32, 28) | the columns: players, size, version, name, victory, loss; hidden with the list |
| 337 | scroll bar (`scenario_list_scrolled` 0x57C990) | (375, 92, 16, by the rows) | the list's; the record's +0x183C |
| 199 … 385, 338 … 344 | the players' rows, the headers, the turn's duration | — | §3.13 |
| 179–182, 336, 389 | the chat's text, edit, plug, bar (`ircentry.pcx`, `CHATPLUG.pcx`, `selslide.pcx`) | — | network games (not built) |
| 128 | caption button `gspbutt.def` (200 × 20), GENRLTXT 501 (show the scenarios), 656 in the load mode (show the saved games) | (414, 81, 200, 20) | hotkey S; new game or network or hot-seat load only |
| 130 | as 128, GENRLTXT 760 (the random map) | (414, 105, 200, 20) | hotkey R; editions 1 and 3 only (AB and Complete) |
| 129 | as 128, GENRLTXT 502 (show the advanced options) | (414, 509, 200, 20) | hotkey A |
| 107–111 | `gspbut3` … `gspbut7.def` (30 × 46) | (506 + 32 i, 456) | the difficulty, easy to impossible |
| 186 | `scnrbeg.def`, `scnrlod.def` (load), `scnrsav.def` (save) (166 × 40) | (414, 535, 166, 40) | Begin, Load or Save; hotkey Enter |
| 188 | `scnrback.def` (166 × 40, 2 frames) | (584, 535, 166, 40) | Back; closes; Escape (a campaign's variant: E20) |

Loaded with the screen: `ScSelC.def` (the list's version marks: RoE, AB, SoD), `scnrvict.def` (12
victory marks), `scnrloss.def` (4 loss marks), `ScnrStar.def`, `itpa.def`, `un44.def`, the heroes'
small portraits, `hpsrand.pcx`, `hpsrand0`, `1`, `6.pcx`, `adop%cpnl.pcx`, `adopflg%c.pcx` (the
last ones the players' rows', §3.13, §3.14).

### 3.3 The list: its panel, its files, its order

- **The panel** (`scenario_list_toggle` 0x580A70, button 128): a toggle; shown, a press hides it
  (`scenario_list_hide` 0x5819A0); else it hides the advanced options and the random map, clears
  the random map's choice (+0x37F), shows 101, 361, the rows, the bar, the filters and the columns
  (a network guest's disabled), and the first time (+0x1864 clear) builds the list with the busy
  cursor, the screen redrawn and GENRLTXT 503 (reading the map files) drawn at (123, 122, 184, 25)
  meanwhile. Its argument picks the list's kind (+0x66): 0 the files, 1 the random maps' (§3.16).
- **The files** (`scenario_list_build` 0x582B40): the folder `maps` (new), `games` (load and
  save) or `random_maps` (the kind 1); the pattern by `scenario_file_patterns` 0x683458: `*.h3m`
  (new), `*.gm?` (load, save), `*.cgm` (the campaign's saves), `*.tut` and `*.tgm` (the tutorial's
  new and saved); up to 5 000 names from `_findfirst`/`_findnext` in the folder's order; each read
  by `scenario_header_read` 0x583170 into a record of 0xCA4 bytes (`structs.toml`
  `scenario_info`): a map's header (`sub_513320`, *not read*) or a save's (`sub_4BC750`, *not
  read*), the name (60 characters) and description (300), the file's last-write time. A file that
  fails to read is left out; in a network or hot-seat game a map for fewer than 2 players is left
  out (the record's +8 below 2, *inferred*: the human players).
- **Unusable files stay listed:** a map whose format is not RoE's (0x0E), AB's (0x15) or SoD's
  (0x1C) gets the name GENRLTXT 431 (an old format) and the description 506 (save it in the
  editor), its victory, loss and difficulty cleared; a map with no players (+4 clear) the
  description 507 (not playable); a save of another version (+0x708 below 25 and not 16 to 18)
  the name 431 and the description 505 (a broken file). A save whose file name begins with
  GENRLTXT 77's text (the autosave's) shows that text as its name.
- **After the read** (`scenario_list_finish` 0x582E90): the selection 0, the bar's range (count −
  rows + 1); a load with no files sets `no_saves_notice` 0x69FDA0, and the procedure's next call
  shows GENRLTXT 686 (no saved games) once and disables Begin; the list sorted by key 2 (the
  version, then the name) while the current key is recorded as 1 (the size's: CLT-Q91, §3.4); the entry
  whose file name is the kept one (§3.1; 0x58D3E0, *not read*) selected; a save with no match:
  no selection and the current game's header shown (0x577530, *not read*).
- **The shown list** (+0x1050) is the full list (+0x1030, or the random maps' +0x1040) less the
  entries of another size than the filter (+0x1860, 0 all), in the full list's order.

### 3.4 The rows, the columns, the filters, the keys

- **Drawn by** `scenario_dialog_redraw` 0x584550 (H3API's Redraw) when the list is shown:
  GENRLTXT 511 (the sizes' label) at (25, 52, 132, 32) colour 2 align 5; then for each visible row
  *i* (top + *i*, *y* = 123 + 25 *i*), in colour 5 for the selection and 4 otherwise, all
  `smalfont.fnt` centred: the players "%d/%d" of the record's +6 and +8 (*inferred*: the players
  and the humans) at (26, *y* − 1, 30, 25); the size's letter GENRLTXT 512 to 515 (36, 72, 108,
  other) at (59, *y* − 1, 30, 25); the version's mark `ScSelC.def` frame 0 RoE, 1 AB, 2 SoD (none
  for another; in the load and save modes from the save's +0xBE8 when +0xBE0 is set, *not read*)
  at (91, *y*); the name (`scenario_entry_name` 0x584010: a map's name, GENRLTXT 509 when empty,
  508 past the list; in the load and save modes the file's name, as `save1.GM1`) at (125, *y* − 1,
  184, 25); the victory mark `scnrvict.def` frame of the type (+0x30, 0 to 11; 11 for none) at
  (309, *y*); the loss mark `scnrloss.def` (+0x7C; 3 for none) at (342, *y*).
- **The columns** (190 to 195) sort by keys 3, 1, 2, 0, 4, 5 (`scenario_list_sort` 0x585050): a
  press on the current key's column flips the order (+0x36C), another column sets it ascending;
  the selection and the top go to 0, then `scenario_select`(0). The comparisons: key 0 the names
  case-insensitively (`scenario_compare_name` 0x5903B0; in the load and save modes a file whose
  name begins `AUTOSAVE.` compares as a space and its file name, so it leads), key 1 the size
  (+0x18, inlined in 0x591310), key 2 the format (+0, RoE < AB < SoD), then as key 0
  (`scenario_compare_version` 0x591190), key 3 ten times +6 plus +8
  (`scenario_compare_players` 0x590E00), key 4 the victory's type (signed +0x30,
  `scenario_compare_victory` 0x591CB0), key 5 the loss's (+0x7C, `scenario_compare_loss`
  0x592030). The sorts are the library's (unstable): keys 1, 3, 4, 5 leave ties in an order of the
  folder's listing and the algorithm (CLT-Q90: the client sorts stably, the name then the file
  name breaking ties). The recorded key after the read is 1 while the list is sorted by 2, so the
  original's first size press sorts descending (CLT-Q91: the client records the key it sorted by;
  decision 12). A pending network transfer blocks the columns.
- **The filters** (137 to 141, `scenario_list_filter` 0x585D90): 36, 72, 108, 144, all; the
  shown list rebuilt, the top and the selection 0, the size mark set, the bar's range,
  `scenario_select`(0, or −1 when empty), the screen redrawn.
- **A row** (142 to 159, the row *r*): in the save mode `scenario_select`(top + *r*) (the name's
  edit takes the file's name) and the row kept (+0x68); else, within the list, a second press
  within 400 ms of the last row press (or of the screen's opening: +0x60) starts Begin (§3.6) with
  the selection as it is, else the time kept and (single or a network host) `scenario_select`(top
  + *r*). CLT-Q91: the client's double click needs the same row (decision 12).
- **Keys** (with the list shown): Up and Down move the selection by one (not in the save mode),
  the top following; Page Up and Page Down move the top by rows − 1 (clamped), the selection
  keeping its row; then the screen redrawn and `scenario_select`. Delete in the load and save modes
  (`saved_game_delete` 0x58DFB0): GENRLTXT 688 (delete this file?) with the file's name, style 2;
  OK deletes it from `games`, removes it, sorts by key 2 again and selects the first (or none).
- **The bar** moves the top and redraws through `scenario_select` (0x57C990).

### 3.5 The information panel

- **`scenario_select`** 0x585500 (H3API's UpdateForSelectedScenario; the index and a redraw
  flag): an index past the list does nothing; it records the selection and whether it changed;
  −1 (no random map): in the save mode the current game's header and description, else an empty
  description; else the record of the shown list (or the random map's own, +0x38C). The save mode
  copies the file's name, cut at its first full stop, into the edit; the load mode keeps the
  file's name and takes the save's turn duration; a new game keeps the map's file name (§3.1); the
  map into the game (0x583580) and the players' rows (0x584C40, `scenario_hero_lists`; §3.12).
  Then the size mark, the flags (§ below), Begin enabled (a new game: the format is RoE's, AB's or
  SoD's; a load: the list is not empty; a save: always), the load and save modes' difficulty
  buttons showing the game's, and the redraw (the whole screen when the selection changed, else
  the list's items).
- **Drawn** by `scenario_info_draw` 0x5840F0 (from the redraw): the name in `bigfont.fnt` colour 8
  at (422, 45, 324, 30); in the load and save modes with a selection the file's local time
  "%d/%d/%d - %d:%02d" at (422, 27, 278, 18) colour 4 align 6 (its fields' order *not read*); the
  map's difficulty, `ArrayTxt.txt`'s difficulty names (0x6A77EC) by the record's +5, at (414, 454,
  89, 48) colour 4 align 5; the rating "%d%%" of `scenario_ratings` 0x683440 by the chosen
  difficulty (80, 100, 130, 160, 200: the map's difficulty plays no part) at (666, 454, 83, 48);
  unless the chat is shown: the victory text `vcdesc.txt`'s record of the type + 1 (0 the
  standard; `victory_texts` 0x69FB70, `victory_texts_load` 0x577A50), joined "%s %s %s" with
  GENRLTXT 5 (or) and record 0 when the map also allows the standard victory (*inferred*: the
  record's +1), at (456, 305, 288, 32) colour 4 align 4; the loss text (`lcdesc.txt`'s alike,
  `loss_texts` 0x69FC1C, *inferred*) at (456, 364, 288, 32); the marks `scnrvict.def` (type, 11 for
  none) at (420, 308) and `scnrloss.def` (type, 3 for none) at (420, 365).
- **The flags** (`scenario_team_flags` 0x585300; not in the save mode, nor in a single load): the
  local colour (the hot seat's first record's; else the network seat's); each colour that plays
  (game +0x1F6D0 not negative) of the local colour's team (game +0x1F879 by colour; itself
  included) shows the next allies' flag (112 onward) with its colour's frame, any other the next
  enemies' (120 onward); the strip (456, 402, 290, 25) redrawn over a kept copy of its background.
- **The difficulty** (107 to 111): game +0x1F6D8 (0 easy … 4 impossible) and
  `scenario_last_difficulty` set; the five lose state 0x10, the chosen gains it (its frame 3); a
  network host sends the setup; the screen redrawn.

### 3.6 Begin, Back and the save's name

- **Back** (188): the result 0x7801 and the close; a network or hot-seat game (not the save mode)
  closes the network session (`network_close` 0x5544B0); the save mode clears the save's name
  (`save_name` 0x69FC2C).
- **Begin** (186; Enter; `scenario_begin` 0x58BCE0, continued past 0x58BFC0 by a jump): in the
  save mode, the save's name (below); else, in order: a random map chosen, it is made (§3.16); a new
  game whose map's format is not RoE's, AB's or SoD's does nothing; the edition's allowed formats
  (`0x699240` by edition, a bit per format) refuse a map or a save with GENRLTXT 745 or 744 (this
  needs AB or SoD: 746, 747; never on Complete); Begin disabled does nothing; no maps and no
  random map: GENRLTXT 530 (no maps); a new game whose local record has no colour: 531 (pick your
  place); a network or hot-seat game with fewer than two placed players: 83 (two players needed);
  then the busy cursor, `loading_screen_open`, and the load (`scenario_load_begin` 0x58C290, *not
  read*) or `new_game_start` (§3.17).
- **The save's name** (`save_name_confirm` 0x577360, from Begin or Enter with the edit's text):
  free space below 1 MB gives GENRLTXT 709 and stays; the name cut at its first full stop, empty
  stays; the file name is the name and `save_extension` 0x691268 (`game_save_dialog` writes
  `.GM` and the count of humans still playing, or `.CGM` in a campaign); when it exists in `games`,
  GENRLTXT 494 (overwrite?) style 2; OK (or no file) copies the name to `save_name`, closes with
  the busy cursor; the caller saves (T1b3).

### 3.7 The procedure and the run

`scenario_dialog_run`: in a new game or a network or hot-seat load, the turn duration's bar set
(§3.13); then the modal loop (0x6027A0 with 0x58E2D0, *not read*). `scenario_dialog_proc`: the
adventure tick first; the load's notice (§3.3); the save mode through the base procedure
0x41B1C0, the others through `dialog_proc_default` 0x5FF820 and the network messages; a key down
(§3.4; Enter in the save mode §3.6); a command (0x200/13) to `scenario_dialog_click` 0x5865B0 (its
switch: the cases of §3.4 to §3.6 here, 207 to 352 §3.14's and §3.16's); a move hovers the flags 263
to 270 (`scenario_hovered` 0x683470; §3.15); a close through the vtable's +0x38.

### 3.8 Right press and hints

`dialog_items_set_hints` gives items 104 to 344 `Help.txt`'s pairs from `scenario_hints`
0x6A6CA0: item *i* gets record *i* − 83 (107 → 24, the easy difficulty's; 128 → 45; 137 → 54; 186
→ 103 Begin's; 188 → 105; 190 → 107 to 195 → 112, the columns'). A right press on a list row shows
nothing; on 362 to 385 the players' popups (§3.15); on 387 a dialog of the teams (0x576540, *not
read*; a new game's variant); on any other item its hint's right-click text (`dialog_right_click`),
then the screen redrawn.

### 3.9 To `GameSetup` (decision 4)

| Choice | Where the original keeps it | `GameSetup` |
|---|---|---|
| the map (a row, the kept name) | game +0x1F6D9 (the file), +0x1F86C (the header) | `GameSetup` from the map's import (`h3-map`'s model: `Header`, `victory`, `loss`, `teams`); the list's columns need only the header, victory, loss and teams (a header-only read: M3's T0) |
| the difficulty (107 to 111) | game +0x1F6D8, `scenario_last_difficulty` | `difficulty` (0 to 4) |
| the rating | shown only | none |
| a saved game (load) | the file | the engine's adventure record (decision 8), its own list |
| the save's name | `save_name`, `.GM<n>`/`.CGM` | the record's file name (decision 8; the extension M4's T0) |
| who may play a colour (199 + *c*) | the map's player +0, +1 | the import's players (a colour either may play) and `PlayerSetup.human`'s default (ADV-Q27) |
| the colour (the flags, §3.14) | the records' +0x70; the game players' +0xE1, +0xE2 | `PlayerSetup.human` (a colour a human holds; the others the computer's); who sends a colour's commands is the shell's seat map (E24 M5) |
| a player's name (345 + *c*, the hot seat's names) | the record's +4; the game player's +0x20B9C | none: the shell's own (E24 M5) |
| the town (215 to 230) | +0x24; game +0x1F6B0 | `PlayerStart.choice` (`FactionChoice::Fixed` or `Random`, TWN-SET-001; drawn at the build, CLT-Q95) among `PlayerStart.factions`; the import's `Fixed` for a single faction matches the original's offer (the random-town flag adds only Random of that one) |
| the hero (231 to 246) | +0x20, +0x2C; `chosen_heroes` | `PlayerStart.pick` from `GameSetup::offer` (TWN-SET-006), `None` for Random; a computer's always `None` |
| the bonus (247 to 262) | +0x6C; game +0x1F864 | **E10's gap** (decision 6): no field; drawn and chosen, its effect a `CLT-STUB` at M3 |
| the handicap (207 to 214) | +0x78; game +0x1F6A8 | **E10's gap** (decision 6), as the bonus |
| the turn's duration (338) | +0x378; game +0x1F843 | **a gap:** no timer in the engine; drawn and chosen, the timer a `CLT-STUB` at M3 (the hot seat's clock, E24 M5) |
| a random map (§3.16) | +0x18A0 to +0x18BC; `random_maps` | none: no generator (CLT-Q92: a stub) |

T1b2 took the players' records, the advanced options, the random map and the begin (§3.12 to
§3.18).

### 3.10 Not built (the agent's calls, *provisional*; M1's review list)

The network paths (the host's relays, the guest's wait with GENRLTXT 510, the chat 131, 179 to
182, the transfers that block the columns); the busy cursor's helper thread (the client reads off
its loop); the edition's format check (one edition); the original's saves (`games/*.gm?`, decision
8: the client lists its own records, the autosave's naming kept by its file name); the lobby's
path. T1b2: the advanced options' network paths (the host's messages 0x40B to 0x43A, the guest's
requests, the edition's check of the host's map, the Conflux choices a guest's RoE turns Random);
the random map's generator and its files (CLT-Q92); the original's draw of a human's Random town at
the header's read (CLT-Q95: only the first shown town is the client's draw).

### 3.11 For T2's captures

The screen in each mode (new: no left panel; a single load: the list; a save: the list with the
name's strip); both backgrounds; the list's rows (colours, marks, a long name), each column's sort
and its flip, each filter, the keys, the bar; the notice with no saved games; the information
panel for maps with each victory and loss mark, the flags, the difficulty and the rating; which of
items 132 and 134 shows; Begin's messages; Delete's question; a save's overwrite question; the
right press on the panel buttons and the columns. The list's order among ties is not compared
(CLT-Q90).

### 3.12 The players' records and slots (M1 T1b2, 2026-11-12)

Read from `Heroes3.exe` as §3.1 (the human player's single and hot-seat paths followed, the
network ones noted). T1b2's sections: the records (§3.12), the advanced options (§3.13), a row
(§3.14), the right press and the hover (§3.15), the random map (§3.16), the begin (§3.17).

- **A record** (0x7C bytes, H3API's `H3ScenarioPlayer`; `structs.toml` `scenario_player`):
  +0 the slot's id (0 free), +4 the name, +0x1C the edition, +0x20 the hero (an index into the
  list +0x2C of +0x28 types, −1 Random), +0x24 the town (−1 Random), +0x6C the bonus (0 artifact,
  1 gold, 2 resource, 3 Random), +0x70 the colour (−1 none), +0x74 −1 (§3.14), +0x78 the handicap
  (0 none, 1 mild, 2 severe). `scenario_player_init` 0x57C790 starts one: the bonus Random, the
  handicap none, the rest −1 or 0. The dialog holds eight **human records** (+0x1064, a slot
  each, any colour) and eight **computer records** (+0x1444, by colour); a colour's record is the
  human record holding it, else its computer record.
- **A new game** (`scenario_slots_new` 0x57F740): the turn's duration 10 (unlimited; +0x378 and
  game +0x1F843); in the hot seat a slot per name of `hot_seat_names` in their order (ids 1 on;
  the names' block freed), single the local slot (`local_slot` 0x69D608: id 1,
  `network_default_name`, the edition). Each slot through `scenario_version_from_slots`
  0x58E700: `scenario_slot_add` 0x577BE0 (the first free record; none when eight), then the
  dialog's version (+0x1898): the highest format, 3 down to 0, that every slot's edition and the
  chosen map allow (`edition_formats` 0x699240; on Complete SoD); a change rebuilds a random map's
  record (§3.16) and, the panel shown and the version RoE, turns the Conflux choices Random
  (network play only). Then, single or a host, the list built, the kept map found and selected
  (§3.3).
- **A load** (`scenario_slots_load` 0x57F330): the same slots; the turn's duration the save's
  (`scenario_to_game`); the kept save selected.
- **The rows from the map** (`scenario_players_init` 0x584C40, after `scenario_to_game` 0x583580
  copied the record's players block, hero owners and header into the game): every human record's
  colour −1 (a load: the colours lost in the save, game +0x1F636, no longer playable by a human);
  then **the hot seat:** each colour a human may play (the map's player +0, in colour order) to
  the next human record without one (a load: first the save's own humans, `load_slots` 0x69FDA4); **single:**
  to the local record the first colour only a human may play, with its town the one the list's
  record drew for it (§3.17), else the first a human may play, else the first anyone plays, else
  none (its town Random). On a map with a colour only a human may play, the original makes that colour and every
  later one computer-only (the first two loops of 0x584DF0): the player may then move only to an
  earlier colour; no shipped map has such a colour (the 159 maps' headers, checked privately).
  **CLT-Q93** (decision 13): the client turns only the other human-only colours computer-only.

### 3.13 The advanced options (`scenario_advanced_toggle` 0x581100, button 129)

- **Opens** in a new game or a hot-seat or network load, never in the save mode; shown, a press
  hides it (`scenario_advanced_hide` 0x581B20: its items hidden, the name's edit unfocused, the hot
  seat's last pressed colour +0x1824 and +0x37C cleared). A network game first checks the host's
  map against the edition (GENRLTXT 744 or 745 with 746 or 747; not built).
- **Shows:** the list hidden and the random map's tab (`random_map_hide` 0x581D80); 102
  `AdvOptBk.pcx`, the headers, the duration's label 340 and bar 338 (set to +0x378, enabled
  single, in the hot seat and for a host). When the selection changed since (+0x1854) every
  record's colour, town and hero are cleared and the rows made again (§3.12). Then per colour that
  plays (game +0x1F6D0 not negative; in a load not lost), counted as rows *r* from 0: its items moved
  to its row (below); the flag shown where a human may play the colour, enabled (single, hot
  seat, host); 199, 207, 345, 362, 370, 378 + *c* shown; single the handicap disabled and the
  name's edit hidden; a load the handicap disabled. Item 199 + *c*: `player_kind_texts` 0x6A7E18
  (`ArrayTxt.txt` 206 both, 207 a human only, 208 a computer only). Then `scenario_hero_lists`; a
  hero two human records hold: the later colour's steps on (`scenario_hero_step`, +1). +0x37C set.
- **The items** (the constructor's loop over the colours, 0x57AF82 to 0x57B51B; per colour *c*,
  its row *r*; tiny is `tiny.fnt`, small `smalfont.fnt`):

| Id | Item | Rectangle | Notes |
|---|---|---|---|
| 263 + *c* | `AOFLGB%c.DEF` (the colour's letter) | (14, 130 + 50 *r*, 42, 50) | the flag: a press §3.14, the hover §3.15 |
| 199 + *c* | text, tiny, colour 4, align 5 | (62, 151 + 50 *r*, 46, 24) | who may play (GENRLTXT 500 at the start) |
| 207 + *c* | `adopb2%c.def` (`colour_letters` 0x641AE0) | (110, 151 + 50 *r*, 50, 24) | the handicap: a caption button (tiny, `handicap_names` 0x6A7800, `ArrayTxt.txt` 210 to 212) in the hot seat and network, a plain one (disabled) single |
| 215, 223 + *c* | `adoplfa.def`, `adoprta.def` (11 × 24) | (164, 133 + 50 *r*), (225, …) | the town's arrows |
| 231, 239 + *c* | as 215 | (240, …), (301, …) | the hero's |
| 247, 255 + *c* | as 215 | (316, …), (377, …) | the bonus's |
| 345 + *c* | text, small, colour 1, align 1 | (62, 130 + 50 *r*, 97, 17) | the name: the human record's, else GENRLTXT 469 |
| 353 + *c* | edit, small, at most 21 | as 345 | single: the own name's edit, hidden |
| 370, 362, 378 + *c* | areas, 48 × 32 | (176, 130 + 50 *r*), (252, …), (328, …) | the town's, the hero's, the bonus's right press |
| 338 | scroll bar, 11 steps (`scenario_duration_scrolled` 0x57C7F0) | (58, 557, 194, 16) | hidden |
| 339 | GENRLTXT 518, small, colour 2, align 5 | (58, 90, 104, 36) | the name's and handicap's header |
| 341 or 342 | GENRLTXT 519, as 339 | (163, 90, 75, 36) | the town's header; its id by the edition (+0x189C: 342 on AB and Complete), so its hint |
| 343, 344 | GENRLTXT 520, 521, as 339 | (239, 90, 75, 36), (315, 90, 75, 36) | the hero's, the bonus's headers |
| 340 | GENRLTXT 522, as 339 | (58, 534, 334, 20) | the duration's label |

- **The redraw** (`scenario_dialog_redraw` with +0x37C): the names set; GENRLTXT 516 in
  `bigfont.fnt` colour 8 align 1 at (58, 24, 334, 25), 517 small colour 4 align 1 at (58, 48,
  334, 34); the duration's text `turn_duration_texts` 0x69FB44 by game +0x1F843 (`turndur.txt`
  records 0 to 10: 1, 2, 4, 6, 8, 10, 15, 20, 25, 30 minutes, unlimited; `turn_durations_load`
  0x577AB0) small colour 4 align 5 at (256, 556, 134, 18); every row (§3.14). The bar sets +0x378
  and game +0x1F843 (a host sends it) and redraws the screen. Hints: items up to 344 as §3.8; the
  names, the edits and the areas have none.

### 3.14 A row (`scenario_player_row_draw` 0x58D510) and its presses

- **The draw** (the colour, a redraw flag, the row or −1 to count it): the flag; the colour's bar
  `adop%cpnl.pcx` at (57, 128 + 50 *r*); 199 and 207 + *c*; then from the colour's record:
  - **the town:** the record's when the map's random-town flag (its player +0xA) is set or it
    allows more than one faction (`faction_choice_offered`), else the first allowed. Random:
    `hpsrand0.pcx` at (176, 130 + 50 *r*) and GENRLTXT 523; a town: `itpa.def` frame 2 *t* + 2 there
    and `town_type_names` (`ArrayTxt.txt` 219 + *t*); the texts tiny colour 4 align 5 at (164, 162 +
    50 *r*, 71, 16). The arrows shown and enabled when `scenario_town_choosable` 0x58CEB0: not a
    load; the hot seat, or the record free (a computer's: single or a host) or the local player's;
    and the map offers a choice (the flag, or more than one faction).
  - **the hero** (`scenario_shown_hero` 0x58D0E0): with neither the generate flag (+0xB) nor the
    random-hero flag (+0x18) the map's customised main hero's portrait (+0x20, −1 none); else the
    record's choice through the map's customised heroes (game +0x1FB2C; a portrait). A hero: its
    small portrait at (252, 130 + 50 *r*) and its name (`scenario_shown_hero_name` 0x58D1F0: the
    map's custom name, a customised hero's, the hero's) at (240, 162 + 50 *r*, 71, 16); no hero:
    `hpsrand6.pcx` and GENRLTXT 524 (none) when its arrows are disabled and the colour has neither
    flag (and with a Random town no main hero), else `hpsrand1.pcx` and 523. The arrows when
    `scenario_hero_choosable` 0x58CFB0: not a load; a human's record (outside the hot seat the
    local one); the shown town not Random; the generate or the random-hero flag (as TWN-SET-006).
  - **the bonus** (+0x6C): `ScnrStar.def` frame 9 artifact, 8 gold, a resource the town's index
    (Conflux 3; under a Random town the bonus is set to Random), 10 Random, at (328, 130 + 50
    *r*); `bonus_names` 0x6A5E14 (`ArrayTxt.txt` 214 to 216) or GENRLTXT 523 at (316, 162 + 50 *r*,
    71, 16). The arrows: the local player's colour; single or a host also a computer's record whose
    town is choosable or fixed; the hot seat every colour.
  - the name 345 + *c* (single also its edit); with the flag the row's rectangle (13, 127 + 50 *r*,
    382, 54) shown.
  - **A load** shows the game's: the town (+0x1F6B0), the first hero (+0x1F844) or the main hero,
    the bonus (+0x1F864); no arrows.
- **The town's arrows** (215 to 230): from the shown town down (or up) to the next allowed faction
  (Conflux left out when the version is RoE), past the last to Random (−1) and from Random to the
  far end; the hero to Random, `scenario_hero_lists`, a clash stepped on (`scenario_heroes_unclash`
  0x58BBC0), the row redrawn.
- **The hero's arrows** (231 to 246, human records only): `scenario_hero_step` 0x58AF20 through the
  list: next −1 → 0 → … → last → −1, previous the reverse, skipping a hero another human record
  holds; the row redrawn. The engine's `GameSetup::offer` gives the same 16 and `validate` refuses a
  pick twice (`PickTaken`, TWN-SET-006), which the skip never makes.
- **The bonus's arrows** (247 to 262): left −1 (artifact to Random), right +1 modulo 4; the artifact
  skipped for a colour without a hero (the hero Random and none of the random-hero flag, the
  generate flag and a main hero: none of the shipped maps' colours with placed heroes falls under
  it), a resource skipped under a Random town; the row redrawn.
- **The handicap** (207 to 214): (+1) modulo 3, its caption `handicap_names`, the row redrawn.
- **The flag** (263 to 270, `scenario_flag_click` 0x58B790; a network guest's does nothing):
  - **single:** to a colour a human may play, not its own: the old flag's highlight cleared, the
    players block's +0x30 byte of the old colour takes the new one's and the new one's becomes its
    colour's index (*not understood*), the local record's colour the new one, its town and hero Random, the lists; then the flags (§3.5).
  - **the hot seat** (and a host): to a colour a human may play, `hot_seat_colour_cycle`
    0x577AE0: its holder loses it; the first human record with a player from index (the holder's
    +0x74 + 1), always 0 outside network play, or with no holder from the first player without a
    colour, takes it and leaves its own (kept at +0x1830; +0x1824 the colour pressed last: a
    second press on it first returns the holder to the kept colour); then the lists, a clash
    stepped on, the flags. So a press gives the colour to the first player without one, else to
    the first player, and its holder is left with none (Begin then needs two placed players, §3.6).
    **CLT-Q94** (decision 13): the client passes the colour to the next player after its holder in
    the slots' order, the two exchanging colours (a taker without a colour leaves its holder with
    none); with no holder the first player takes it, leaving its own to the computer; after the
    last player the computer (*provisional* in its detail until M3 builds it).
  - then the whole screen redrawn.
- **The name** (345 to 352, `scenario_name_click` 0x58B620): single only, the own colour: the edit
  shown over the name with the record's name, the name hidden, the focus to it (`dialog_set_focus`
  0x5FF490). Where the edit's text returns to the record: *not read* (T2).

### 3.15 Right press and the hover

- **362 to 369, the hero** (`scenario_dialog_right_click`): a hero shown: `scenario_hero_popup`
  0x575A90 (300 × 256 at (250, 172): the name in `medfont.fnt`, the portrait, "%s - %s" in small
  (*not read*: the class and level), GENRLTXT 78 and 79's headings, the specialty's picture and
  text); a Random one: `scenario_info_popup` 0x5757E0 (200 × 150 at (300, 225): a title, a
  picture, two small texts) with GENRLTXT 102, 523 and 103; none: no popup (*inferred*).
- **370 to 377, the town:** Random: `scenario_info_popup` with 104, 523 and 105; a town:
  `scenario_town_popup` 0x575E60 (H3API's town alignment dialog; 256 × 320 at (272, 140): GENRLTXT
  81's heading, the town's picture and name, 80's heading, its creatures in `cprsmall.def` 32 × 32
  in two rows at y 159 and 235 of the popup, their names in tiny below).
- **378 to 385, the bonus** (`scenario_bonus_popup` 0x5754F0, 200 × 150 at (300, 225)):
  artifact GENRLTXT 84, frame 9, 91; gold 85, frame 8, 88 and 93; resource 86, the town's frame and
  by the faction 693 to 696 or 90 (Castle, Necropolis, Stronghold, Fortress wood and ore; Rampart
  crystal; Tower gems; Inferno, Conflux mercury; Dungeon sulfur) with 689 to 692 or 94; Random 87,
  frame 10, 95. The amounts in these texts are E10's rules (§3.9).
- **The hover** (`scenario_dialog_proc` from 0x5881B2, a move): over a flag 263 to 270 the flag
  gains state 0x10 and its row is redrawn (not for a network guest), the one left loses it;
  `scenario_hovered` 0x683470 −1 off the flags.

### 3.16 The random map (button 130, `random_map_toggle` 0x57FEB0)

- **Opens** in a new game only; shown, a press hides it (`random_map_hide` 0x581D80: 103, 280 to
  335, +0x37E cleared); else the list and the advanced options hidden, 103 `RanMapBk.pcx` and 280
  to 335 shown, the record into the game, +0x37E set, and the first time +0x37F (a random map
  chosen) and `random_map_info_build` 0x580430: the screen's own record (+0x38C) from the choices
  (the name GENRLTXT 741, the description 742, the players' flags from the counts, *not read* in
  full), then the rows (§3.12), the lists, the flags, Begin enabled. The list's toggle clears
  +0x37F (§3.3).
- **The items** (`random_map_items_create` 0x57D170, from the constructor; labels in small colour
  1, align 5 for 280 and 0 for the others):

| Id | Item | Rectangle |
|---|---|---|
| 280 | GENRLTXT 753 (the size) | (58, 82, 99, 31) |
| 281 to 284, 285 | `RanSizS`, `RanSizM`, `RanSizL`, `RanSizX.def`, `RanUndr.def` (44 × 33) | (161 + 47 *i*, 81) |
| 286; 287 to 294, 295 | GENRLTXT 754 (human or computer players); `RanNum1` to `RanNum8.def` (30 × 32), `RanRand.def` (55 × 32) | (71, 133, 250, 16); (70 + 32 *i*, 153), (326, 153) |
| 296; 297 to 304, 305 | 755 (their teams); `RanNum0` to `RanNum7` (1 and 2 30 × 24), `RanRand` | (71, 199, 250, 16); y 219 |
| 306; 307 to 314, 315 | 756 (computer only players); `RanNum0` to `RanNum7`, `RanRand` | (71, 265, 250, 16); y 285 |
| 316; 317 to 323, 324 | 757 (their teams); `RanNum0` to `RanNum6`, `RanRand` | (71, 331, 250, 16); y 351 |
| 325; 326 to 328, 329 | 758 (water); `RanNone`, `RanNorm`, `RanIsld.def` (83 × 32), `RanRand` | (71, 398, 105, 16); (70, 155, 240, 326; 419) |
| 330; 331 to 333, 334 | 759 (monsters); `RanWeak`, `RanNorm`, `RanStrg.def`, `RanRand` | (71, 465, 105, 16); y 485 |
| 335 | `RanShow.def` (337 × 40) | (57, 535) |

- **The choices** (+0x18A0 to +0x18BC, by the clicks): the size 36, 72, 108, 144; the levels
  (285 toggles 1 and 2); the human or computer players 1 to 8 or −1 Random; their teams 0 to 7 or
  −1; the computer only players 0 to 7 or −1; their teams 0 to 6 or −1; the water 0 none, 1 normal,
  2 islands, 3 Random; the monsters 0 weak, 1 normal, 2 strong, −1 Random. A press redraws the tab
  (`random_map_tab_draw` 0x57EF70) and for the counts rebuilds the record; 335 opens the random
  maps' list (§3.3, kind 1). The tab's draw: the chosen buttons in state 0x10; disabled (0x1000)
  the teams from the players' count on (Random players as 8), the computer only players above 8
  less the players (Random players as 1), their teams from the computer players' count (so
  capped) on; a choice above its cap shows the cap. The redraw: GENRLTXT 739 in `bigfont.fnt`
  colour 8 align 1 at (58, 24, 334, 25), 740 small colour 4 at (58, 48, 334, 34).
- **Begin with a random map** (`scenario_begin`): `random_map_file_name` 0x5879A0 (the first
  free `random_map_%i.h3m` of 15 in `random_maps`, *inferred*), a host's message,
  `random_map_generate` 0x5860E0 (the Random choices drawn with the C library's `rand`, the busy
  cursor, a window (`random_map_progress_create` 0x576F00, presumably the progress), the generation
  and the file written; *not read*; a failure's message by its code), the file's header read into the own record and into the game. Command 279 (no item of the
  reading's; GENRLTXT 749 after a generation): *not read*.
- **CLT-Q92** (decision 13): the tab drawn and chosen as the original's; Begin with a random map
  chosen shows a stub's message; Show Random Maps lists the client's own `random_maps` (empty).

### 3.17 The game's begin

After Begin's checks (§3.6), the busy cursor and `loading_screen_open` (§2.1), `new_game_start`
0x58C570:

1. The version (+0x1898: 2 or 3 SoD, 1 AB, else RoE), then `scenario_dialog_begin` 0x588330 (1):
   every game player's human flags (+0xE1, +0xE2) cleared and set for each colour a human record
   holds (single the local one); per colour: a human record's name into the game's player
   (+0x20B9C + 0x168 *c*); the bonus into `scenario_bonus_choices` 0x69FBF8 (read by
   `new_game_heroes` at 0x4C018E: E10's) and, a new game, game +0x1F864 + *c* with the colour's first
   hero (+0x1F844) −1; the town into the chosen factions
   (+0x1F6B0): a town as it is; **Random:** a human's takes the town the list's record drew for
   that colour when the header was read (`map_file_open_header` 0x51362A: `random_between` over
   the allowed factions, each playing colour, every map of the list), a computer's is drawn now
   (`random_allowed_faction`); the handicap into game +0x1F6A8 + *c*; then
   `current_player_first` 0x4C0630 (the first human colour; in the hot seat the viewed one).
2. A step of the loading bar (`load_progress_step`), a host's message, `game_reset`, a step; the
   chosen heroes into `chosen_heroes` (E09's survey §4.1); `new_game_load` 0x4C0520 (the map's
   read and the game's setup, E08's and E09's readings); a step; a host sends the game (not built).

So a human's Random town is fixed when the list is read, not at Begin (on a map with a human-only
colour the single player's row even shows it first, §3.12; on the shipped maps the row shows
Random and the pre-draw stays hidden). **CLT-Q95** (decision 13): the client hands the engine
`FactionChoice::Random`, drawn at the build from the game's seed (TWN-SET-001); the row shows what
the original's shows (on a map with a human-only colour a town the client draws); the original's
rule for the comparisons only. For the client the begin is: the filled
`GameSetup` built and validated by the engine, handed to the hot seat (E24 M5); the loading bar's
steps are the client's (M2's loading).

### 3.18 For T2's captures (T1b2)

The advanced options single and in the hot seat (two to eight players), a load's; each row's town,
hero and bonus through their arrows (Random, none, a main hero, a resource by faction), the
handicap's three captions, the flags' hover and a press (single, the hot seat: the cycle as
read), the name's edit; the four popups; the duration's eleven texts; the random map's tab with
each choice and the disabled counts; Begin's loading bar to the first adventure frame. A Random
town's outcome is not compared (CLT-Q95).
