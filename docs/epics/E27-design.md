# E27 — design note (the game shell)

Working detail of `docs/epics/E27-game-shell.md`: the readings, the clips, layouts. Grep a section
number or a name; never read it whole once it grows.

## 1. The video archives (M0, 2026-11-12)

Listed on the user's computer from the installation's `Data/VIDEO.VID` (173 entries, 629 MB)
and `Data/H3ab_ahd.vid` (7 entries, 1.6 MB), read only: a `u32` count, then per entry a 40-byte
name and a `u32` offset; an entry's size is up to the next offset. Many clips ship twice, as
Bink (`.bik`) and Smacker (`.smk`); which copy plays is the clip table's (§2.1): below index 28
always Bink, from 28 Bink only when the table's +8 is set and `Bink Video` is on, else Smacker
(so `LoseGame`'s Smacker copy always plays, §5.4; M0's "the Bink version first", `video_open`
0x44D5A0, corrected). Sizes rounded, in MB.

| Use (as known at M0; T1a and T1b read where each plays) | Bink | Smacker only |
|---|---|---|
| The start (§2.1) | `NWCLOGO` 4.2, `H3INTRO` 54.5 | `3DOLOGO` 0.9 |
| Not at the start (§2.1): AB's intro, by no constant index (a campaign's prologue, presumably: E20) | `H3X1intr` 31.8 | |
| A network game's save-game transfer (§2.1, §2.6; not a game's loading bar, which is a DEF) | | `ProgressBar` 0.3 |
| A won scenario's score screen (0x4F3F60, T1b; not the High Scores dialog, §2.8) | | `HSANIM` 4.5, `HSLOOP` 1.8 |
| The menus' clip 33 (§2.5; nothing drawn, CLT-Q86; its frames time the credits, §2.9) | (`CREDITS.BIK` and `.SMK` in `VIDEO.VID` are 4 × 4 placeholders, found first) | `Credits.smk` 0.03 (`H3ab_ahd.vid`, AB's; reached by edition 1 only) |
| A lost scenario's screen (clip 34, §5.4) | `LOSEGAME` 4.0, never played: clip 34's table entry has +8 clear, so its Smacker copy (4.7) always plays (T1b3b's correction of T1b3a's reading) | |
| The battle's result window (E17: `battle-videos.vid`) | `WIN3`, `LBSTART`, `LBLOOP`, `RTSTART`, `RTLOOP`, `surrender`, `defendall`, `defendloop`, `LOSECSTL`, `LOSECSLP` | |
| The spell book's page turns (E17: "Animate SpellBook" off in the harness) | | `PGTRNLFT`, `PGTRNRGH` |
| Campaigns (E20, decision 7): RoE's selection | `CGOOD1`–`3`, `CEVIL1`–`2`, `CNEUTRAL`, `CSECRET` | |
| Campaigns (E20): SoD's selection, AB's (`H3ab_ahd.vid`) | `Birth`, `elixir`, `hack`, `new`, `rise`, `spectre`, `unholy`; `C1ab7`, `C1db2`, `C1ds1`, `C1fl3`, `C1fw1`, `C1pf2` | |
| Campaigns (E20): the scenarios' clips | RoE's `GOOD*`, `EVIL*`, `NEUTRAL*`, `SECRET*`, `ENDGAME` (Smacker copies too); SoD's `H3x2_UAm` | AB's `H3AB*` (34), SoD's `H3x2_*` (44) |
| Elsewhere (a town's, unknown) | `TAVERN`, `TESTING` | |

So the shell (M2 to M4) needs Smacker for the start's first clip and a won scenario's score
screen and a lost one's (decision 5; the loading bar and the menus' clip are no Smacker clips to play, §2.1, §2.5);
`h3-bink` decodes the rest.
The shell's clips: `original-data/shell-videos.vid` on the user's computer and staged (E27 M1 T1a1,
2026-11-12, ten entries; rebuilt at M1 T1b3a with the Smacker copies of `H3INTRO`, `LOSEGAME` and
`NWCLOGO`, which the system options' Low video quality plays, CLT-Q96, §4.7; `LOSEGAME`'s plays always, §5.4): `VIDEO.VID`'s 13
entries `3DOLOGO.SMK`, `CREDITS.BIK`, `CREDITS.SMK`, `H3INTRO.BIK`, `H3INTRO.SMK`, `H3X1intr.bik`,
`HSANIM.SMK`, `HSLOOP.SMK`, `LOSEGAME.BIK`, `LOSEGAME.SMK`, `NWCLOGO.BIK`, `NWCLOGO.SMK`,
`ProgressBar.smk`, in `VIDEO.VID`'s format and order (the header, then the clips in the table's
order), 184 001 948 bytes, SHA-256 `825b64d8…` (the first version's ten clips byte for byte the
same); `h3ab_ahd.vid` goes to the harness whole, its `Credits.smk` and `VIDEO.VID`'s `CREDITS.SMK`
being one name ignoring case.

## 2. The start and the menus (M1 T1a1, 2026-11-12)

Read from `Heroes3.exe` (the map's entries of the task); every item **read** unless marked
*inferred* or *not read*. Texts by table and record (a record counted from 0 with the header, as
`general_text`'s rows; `Help.txt`'s by `help_text_load`'s groups); never the text. Positions are
(x, y, width, height) on the 800 × 600 screen. T1a2 read Multiplayer, the hot seat's names, High
Scores and Credits (§2.6 to §2.9); T1b1 the scenario screen's frame, list and information panel
(§3), T1b2 its players' setup and random map, T1b3 the system options and the end (§4).

### 2.1 The start (`win_main` 0x4F7A30; `game_main` 0x4EE3E0 to 0x4EE834)

1. `win_main`: a named event keeps one instance (a second start shows a message box and ends);
   `timeBeginPeriod(1)`; `game_init` 0x4ED650 (the game's loads, the CD status 0x69957C and the
   screen asked for at the start, 0x6976D8; read whole at M2 T5b1, §11.6); the window class and an
   800 × 600 window; then `game_main`.
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
   - **Which file:** an index below 28 is always Bink (0x44DD20, read at M2 T5a: §10); from 28 Bink when the
     table's +8 is set and `Bink Video` is on (33 only for editions 2 and 3), else Smacker
     (`smacker_open_in_dialog` 0x598AF0). So `3DOLogo` (+8 clear) is always Smacker; `NWCLogo` and
     `H3Intro` are Bink with `Bink Video` on, else their `.SMK` copies. Smacker's lookup
     (`smacker_open` 0x598790) has `video_open`'s order: on Complete `Video.vid`, the CD's, then
     `h3ab_ahd.vid` (edition 1: `h3ab_ahd.vid` first).
   - **Placement:** full screen at (0, 0, 800, 600): a Smacker clip centred, `3DOLogo` (640 × 480)
     at (80, 60); a Bink clip through 0x44DD20, centred too but `NWCLogo` (400 × 300), left at
     the corner and stretched onto the screen (§10). The cursor hidden; at the end, with the
     table's +0xA set (28 to 31), the fade (0x602C50, read at M2 T5a: §10), else the clip's
     rectangle shown; the cursor shown.
   - **`H3Intro`** (`video_play_framed`): `introrim.pcx` (800 × 600) loaded; the clip opened in a
     dialog at (80, 187) in its own size (640 × 360), not playing on, not shown by the tick; the
     rim copied whole onto the back buffer; the screen shown; each frame due (`video_frame_ready`
     0x597930) shown (`video_show`) until the clip's end or a skip; the clip closed, the rim freed.
   - The clips (their headers): `3DOLogo` Smacker, 640 × 480, 81 frames at 15 a second; `NWCLogo`
     Bink, 400 × 300, 195 frames at 15, sound; `H3Intro` Bink, 640 × 360, 2 426 frames at 15, sound.
     Their Smacker copies (read at M1 T3 from `shell-videos.vid`): `NWCLOGO.SMK` 800 × 600 (full
     screen, where the Bink one is 400 × 300), 195 frames; `H3INTRO.SMK` 640 × 360, 2 426 frames;
     both, as `3DOLOGO.SMK`, at a rate of −6666 (66.66 ms a frame, 15 a second).
5. Then the high-score manager's start (0x6993CC's vtable +0), the cursor shown, the menu loop.
- **Not at the start:** clip 31 `H3x1intr` (AB's intro) is played by no constant index anywhere:
  a campaign's prologue by its movie number, presumably (H3API's `H3CmpPrologue`; E20, *not
  read*); clip 32 `Endgame` plays framed by `IntroRim.pcx` after a campaign's end (0x4EF9B1; E20).
- **The loading bar is no clip:** `loading_screen_open` 0x4ED350 copies `loadbar.pcx` (800 × 600)
  to (0, 0) and draws `loadprog.def` (16 × 16, 20 frames) by `load_progress_step` (E24-design §1.6:
  frame *i* at (395 + 18 *i*, 548)); it shows only when a game loads: the tutorial's (§2.4), the
  scenario's begin (§3.17) and the restart (§4.6), never at the start (so M3's begin builds it,
  not M2's start). Clip 63 `ProgressBar` (Smacker, 160 × 160, 20 frames at 10 a second) is opened
  only by `progress_clip_open` 0x557480, its frame driven by a fraction, from two functions whose
  callers include `next_player_turn`, the network turn, `new_game_start` and `network_map_redraw` (DirectPlay's
  "guaranteed" sends nearby): a network game's save-game transfer (§2.6).

### 2.2 The menu loop (`game_main` from 0x4EE842)

- **The top** (0x4EE842): `mainmenu.pcx` (340 × 600, `main_menu_picture` 0x699514) and
  `GamSelBk.pcx` (800 × 600, `game_select_background` 0x699518) loaded when not held; clip 33 in a
  dialog at (0, 0, 800, 600), playing on, not shown (§2.5); `GamSelBk.pcx` copied whole onto the
  back buffer; the cursor the arrow (0, 0).
- **The screen asked for** (`screen_request` 0x6976D8): −1 or 108 the main menu; any other id its
  case at once. Its writers in play: `adventure_key_down` 101, 102, 108 and 107,
  `adventure_command` and one more site 107, `system_options_open` 102, 105, 107 and 108 (§4.3);
  `game_init` the start's (0x4EDA75; §11.6).
- **The main menu's result** (the window manager's +0x38; switch 0x4EEC0D): 101 New Game, 102 Load
  Game (§2.4), 103 High Scores (videos paused, 0x4E9110: §2.8; resumed; `GamSelBk.pcx` copied
  whole; the top), 104 Credits (`mainmenu.pcx` copied, 0x4EDDA0: §2.9; the top), 105 Quit
  (both pictures freed, the clip closed, the end by 0x4F3690 with no text: *not read*), 107 the
  restart at 0x4EF1BC (§4.6), any other: play (0x4EF80B: both pictures freed, the clip closed;
  T1b).
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
  (114, 312, 300, 48) shown, `video_show`. The rectangle misses Single's top rows 4 to 6 and Back's bottom
  rows 567 to 569 (recorded, M1 T3: `menu-new`, `menu-load`: from the first redraw of Single's or
  Back's frame those rows keep the last frame shown, a faint highlight; CLT-Q118). **A press** and **its command** as the main menu's: 100
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
  0x69CA28): (173, 55, 454, 490) over the screen as the game-type menu left it (no background of
  its own beyond its picture: recorded over New Game's and Load Game's menus, §6.1), run by
  `dialog_run_modal`; its
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
- **For the client** (decision 20, CLT-Q101): the status line drawn again at a stage's change (the
  original keeps the last hint after Escape from stage 2 until the pointer moves, §6.3); the
  original's for the comparisons only.

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
    disabled at the open (`dialog_item_enable` 0x5FE940: state 0x20, frame 2; the DEF button's
    drawer shows frame 2 for state bit 8 or 0x20, E17-design §9.7, where §4.1's speeds set bit 8 by
    `SendCommand` 5 with 0x1000; whether 0x20 is the state bit or a value: to settle when the
    shared dialog code is built, decision 20);
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
- *Recorded (T2, §6.3):* OK is enabled by the key that makes the second name and disabled by its
  removal (not at a focus change); a disabled OK still shows its hint and popup; Enter in an edit
  moves to the next edit, never presses OK.
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
  §5.6).
- **The run** (`game_main`'s case 103, §2.2): the videos paused; the dialog built on the stack
  (`high_scores_dialog_init` 0x4E9880), drawn once (`high_scores_draw` 0x4E9E50), `modal_loop` with
  `high_scores_proc` 0x4EA1D0; then destroyed; the videos resumed. No clip plays: `HSANIM` and
  `HSLOOP` (clip 35) belong to a won scenario's screen (0x4F3F60, music "Win Scenario", then the
  new entry 0x4E91D0 with `HighName.pcx`; §5.5, §5.6). No music of its own.
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
  the name) and sets every creature's DEF again. Any other message: once more than 200 ms have
  passed (recorded, §6.3: every 210 ms on 10 ms steps) the tab's eleven creatures step one frame
  (their group 0's count wraps, +0xA4's counters), the table redrawn and shown. No hint line (the
  dialog's +0x10 is empty) and no right press's popup (*inferred*).
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
  reaches the window's bottom, then out at the top; near the end, once the leaving text's rows
  shown are under 435 (a test made in the leaving phase alone: §36.1, M4 T2; recorded, §6.3),
  `Credits.txt`'s record 2 (`credits_closing_text` 0x6A7704) in `smalfont.fnt`, colour 0xD,
  alignment 8 (the window's bottom) over it; (460, 10, 328, 580) shown, the clip shown. When the
  text has left, the screen ends. No music change, no sound. CLT-Q88: the client steps it ten
  times a second with no clip.

- *Recorded (M1 T2, §6.3; a correction):* the closing text shows only near the end, once the
  text's end has risen to about 435 px (first at 244.6 s); the first frame 1 100 ms after the
  screen opens, then one every 100 ms of 2 px; the last at 265.2 s, the main menu at 266.5 s.
  (M4 T2, §36.2: the first step comes 100 ms after the opening, its first change at the 12th.)

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
  `NEWGAME` by the constructor's `strtok`) and the last difficulty (`scenario_last_difficulty` 0x683454; *corrected at M3 T2a*, §21.1: a new
  game's opens at Normal each time). The list selects the entry of that name when it is built (§3.3).
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
| 134 | GENRLTXT 495 (the map's difficulty) | (414, 435, 90, 19) | as 132; in 132's place, but both show, "Map Diff:" on the left and 132's "Player Difficulty:" centred, neither covering the other (recorded, M1 T3: `scn-difficulty`, `scn-right` map-difficulty-label and rating-label) |
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
| 130 | as 128, GENRLTXT 760 (the random map) | (414, 105, 200, 20) | hotkey R; editions 1 and 3 only (AB and Complete); shown in a hot-seat load too (recorded, M1 T3: `adv-load` screen; §3.16) |
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
  meanwhile (*corrected at M3 T1b2*, §20.3: the opening builds it; only a network game's message
  1015 clears the flag). Its argument picks the list's kind (+0x66): 0 the files, 1 the random maps' (§3.16).
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
  whose file name is the kept one (§3.1; 0x58D3E0, *not read*) selected, the list scrolled so
  that it is on the top row, the thumb mid-bar (recorded, M1 T3: `scn-list-keys` list); a save
  with no match: no selection and the current game's header shown (0x577530, *not read*; with
  saves listed the panel shows the current game's name but a saved game's description, recorded,
  M1 T3: `scn-save` screen, where `scn-save-none` shows the game's own; CLT-Q108, §3.5).
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
  keeping its row; then the screen redrawn and `scenario_select`. Delete in the load mode
  (`saved_game_delete` 0x58DFB0): GENRLTXT 688 (delete this file?) with the file's name, style 2;
  OK deletes it from `games`, removes it, sorts by key 2 again and selects the first (or none).
  In the save mode Delete goes to the name's edit and asks nothing (recorded, M1 T3: `scn-save`
  delete): 688 is the load mode's alone.
- **The bar** moves the top and redraws through `scenario_select` (0x57C990).

### 3.5 The information panel

- **`scenario_select`** 0x585500 (H3API's UpdateForSelectedScenario; the index and a redraw
  flag): an index past the list does nothing; it records the selection and whether it changed;
  −1 (no random map): in the save mode the current game's header and description (as read; with
  saves listed the description is a saved game's, recorded, M1 T3: `scn-save` screen; CLT-Q108),
  else an empty description; else the record of the shown list (or the random map's own, +0x38C).
  The save mode
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
  random map: GENRLTXT 530 (no maps; recorded, M1 T3: reached only through a filter that empties
  the list, `scn-begin-filter` begin; with no maps at all Begin looks enabled and does nothing,
  `scn-begin-none` begin; CLT-Q114); a new game whose local record has no colour: 531 (pick your
  place; recorded, M1 T3: in the hot seat only when the first player has no colour,
  `adv-hotseat-begin` begin-1); a network or hot-seat game with fewer than two placed players: 83
  (two players needed); a named player left without a colour is dropped at Begin without a
  message (recorded, M1 T3: `adv-hotseat-begin` flag-red-3, turn: the game starts with A and C;
  CLT-Q109);
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
then the screen redrawn (*corrected at M4 T4b1*, §39.3: only after a popup).

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
  names, the edits and the areas have none. But the single player's disabled handicap gives no
  popup, where the hot seat's caption does (recorded, M1 T3: `adv-single-rest` popup-handicap,
  `adv-hotseat-rows` handicap-popup).

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
    +0x74 + 1: the holder's index in the hot seat, recorded at T4b, §29.1; M1 read it as always
    0), or with no holder from the first player without a colour, else from 0, takes it and
    leaves its own (kept at +0x1830; +0x1824 the colour pressed last: a second press on it first
    returns the holder to the kept colour); then the lists (a clash stepped on: network play
    only, 0x58B823), the flags. So a press on a held colour gives it to the player after its
    holder (nobody after the last), on the computer's to the first player without one, else to the
    first player; a holder is left with none (Begin then needs two placed players, §3.6).
    **CLT-Q94** (decision 13): the client passes the colour to the next player after its holder in
    the slots' order, the two exchanging colours (a taker without a colour leaves its holder with
    none); with no holder the first player takes it, leaving its own to the computer; after the
    last player the computer (*provisional* in its detail until M3 builds it). Recorded, M1 T3,
    for the comparisons beside CLT-Q94 (`adv-hotseat-two` flag-red-2, flag-green-2): with two
    players the second press on red leaves both colours to the computer; the last shot has only
    A placed, on green. In a hot-seat load a flag press can turn a saved human's colour into
    Computer (recorded, M1 T3: `adv-load` pressed; CLT-Q110).
  - then `scenario_dialog_redraw` (T3c1, §26.1: the items not drawn again).
- **The name** (345 to 352, `scenario_name_click` 0x58B620): single only, the own colour: the edit
  shown over the name with the record's name, the name hidden, the focus to it (`dialog_set_focus`
  0x5FF490). Where the edit's text returns to the record: read at T3c1 (§26.2); recorded, M1 T3: Return
  closes the edit and shows the name ("Zed"), which moves with the player to another colour
  (`adv-single-rest` name-enter, name-after-flag).

### 3.15 Right press and the hover

- **362 to 369, the hero** (`scenario_dialog_right_click`): a hero shown: `scenario_hero_popup`
  0x575A90 (320 × 256 at (250, 172), recorded, M1 T3: `adv-single-hero` hero-popup-1, not the
  reading's 300: the name in `medfont.fnt`, the portrait, "%s - %s" in small
  (*not read*: the class and level), GENRLTXT 78 and 79's headings, the specialty's picture and
  text); a Random one: `scenario_info_popup` 0x5757E0 (256 × 192 at (300, 225), recorded, M1 T3:
  `adv-single-hero` hero-popup-random, not the reading's 200 × 150: a title, a picture, two small
  texts) with GENRLTXT 102, 523 and 103; none: no popup (*inferred*; recorded, M1 T3, for some
  rows only: Rebellion's human None shows none, `adv-single-maps` map-1-hero-popup, but on
  Adventures of Jared Haret the computer's row showing None gives the Random Hero popup,
  map-2-computer-hero-popup; CLT-Q112). Each popup's changed block is 8 px wider and taller (a
  shadow, *inferred*).
- **370 to 377, the town:** Random: `scenario_info_popup` with 104, 523 and 105; a town:
  `scenario_town_popup` 0x575E60 (H3API's town alignment dialog; 256 × 320 at (272, 140), recorded,
  M1 T3: `adv-single-town` town-popup-9; GENRLTXT
  81's heading, the town's picture and name, 80's heading, its creatures in `cprsmall.def` 32 × 32
  in two rows at y 159 and 235 of the popup, their names in tiny below).
- **378 to 385, the bonus** (`scenario_bonus_popup` 0x5754F0, 256 × 192 at (300, 225), recorded,
  M1 T3: `adv-single-town` bonus-popup-1, not the reading's 200 × 150):
  artifact GENRLTXT 84, frame 9, 91; gold 85, frame 8, 88 and 93; resource 86, the town's frame and
  by the faction 693 to 696 or 90 (Castle, Necropolis, Stronghold, Fortress wood and ore; Rampart
  crystal; Tower gems; Inferno, Conflux mercury; Dungeon sulfur) with 689 to 692 or 94; Random 87,
  frame 10, 95. The amounts in these texts are E10's rules (§3.9).
- **The hover** (`scenario_dialog_proc` from 0x5881B2, a move): over a flag 263 to 270 the flag
  gains state 0x10 and its row is redrawn (not for a network guest), the one left loses it;
  `scenario_hovered` 0x683470 −1 off the flags.

### 3.16 The random map (button 130, `random_map_toggle` 0x57FEB0)

- **Opens** in a new game only (as read; its button 130 shows in a hot-seat load too, recorded,
  M1 T3: `adv-load` screen; pressed at M4 T5a: nothing, §41.1, CLT-Q161); shown, a press hides it
  (`random_map_hide` 0x581D80: 103, 280 to
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
- *Recorded (M1 T3):* after Show Random Maps (an empty list) and the tab toggled back, the panel's
  name, conditions and rating blank, the description and the seven flags stale, Begin drawn
  enabled (`adv-random` random-maps, tab-again, tab-hidden; CLT-Q111). *Corrected at M3 T5b*
  (§31.2): the record is built again; the panel draws nothing over the random maps' empty list. The size mark (189, §3.2)
  stays on S in the tab whatever the size, and over an empty list; its frame 4 is never seen
  (`adv-random` size-xl, `scn-load-none` screen; CLT-Q115).

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

## 4. The system options and the menu in play (M1 T1b3a, 2026-11-12)

Read whole: `system_options_open` 0x41AC00, `system_options_create` 0x5B1790,
`system_options_run` 0x5B30B0, `system_options_proc` 0x5B3140, the rest of `game_save_dialog`
0x418160; read in part: the restart (`game_main`'s case 107, 0x4EF1BC) and the settings' readers.
T1b3 was cut in two (decision 14): the end of a scenario is T1b3b's.

### 4.1 The opening and the dialog

- **Opened** by the adventure screen's System Options button (`adventure_command`'s item 10,
  0x409D03); the command's out byte takes the result, set when play is to end. `system_options_open`:
  a call not read (the ambient sounds', presumably), the arrow cursor (0, 0), Walk Speed kept, the
  manager's lifted-hero byte +0x20C kept and the hero put back (`adventure_hero_stop` 0, 1); built,
  run, destroyed (`system_options_destroy` 0x5B3040).
- **The dialog** (`system_options_vtable` 0x642468, `system_options_dialog` in `structs.toml`):
  the base dialog at (159, 56), 481 × 487, flags 0x12 (centred on 800 × 600); +0x60 (anything
  changed) 0, +0x64 Quick Combat as it opens. Its items in the constructor's order:

  | Ids | Item | Place (x, y), size | Detail |
  |---|---|---|---|
  | 200 | `SysOpBck.pcx` | (0, 0), 481 × 487 | flags 0x800, in the viewing player's colours |
  | 102, 106, 107, 108, 105, 30722 | `soload`, `sosave`, `sorstrt`, `somain`, `soquit`, `soretrn.def` (2 frames: 0 pressed, 1 up) | (246, 298), (357, 298), (246, 357), (357, 357), (246, 415), (357, 415); 100 × 48 | Load, Save, Restart, Main Menu, Quit, Return; frame 1, click frame 0, a click does not close, flags 2; hotkeys L (0x26), S (0x1F), R (0x13), M (0x32), Q (0x10), Escape (1) |
  | 201 to 210 | `syslb.def` DEF items (18 × 36, 10 frames) | (29 + 19 *i*, 359) | the music bars; flags 0x10 |
  | 211 to 220 | the same | (29 + 19 *i*, 425) | the effects bars |
  | 224 to 227 | `sysopb1` to `sysopb4.def` (46 × 32, 3 frames) | (28 + 48 *i*, 77) | Hero Speed: Walk Speed 1 to 4 |
  | 228 to 231 | `sysopb5` to `sysopb8.def` | (28 + 48 *i*, 144) | Enemy Speed: Computer Walk Speed 2 to 5 (5 the enemy's moves not shown) |
  | 232 to 234 | `sysopb9`, `sysob10`, `sysob11.def` (62 × 32) | (28 + 64 *i*, 210) | Map Scroll Speed: Window Scroll Speed 0 to 2 |
  | 240, 239 | `sysob12.def` (High), `sysob13.def` (Low) (94 × 32) | (28, 276), (124, 276) | Video Quality: Bink Video 1, 0 |
  | 235 to 238, 241, 242 | `sysopchk.def` DEF items (32 × 24; frame 0 the cross, 1 the tick) | (246, 55 + 32 *k*) | Show Route, Move Reminder, Quick Combat, Video Subtitles, Town Outlines, Animate SpellBook; flags 0x10 |
  | −1 | texts | | GENRLTXT 569 (the title) `bigfont.fnt` colour 7 align 5 at (26, 19, 432, 28); the rows' names `medfont.fnt` colour 7 align 5 at (26, *y*, 193, 20): 570 *y* 56, 571 122, 572 188, 21 254, 395 339, 396 406; the boxes' names `medfont.fnt` colour 1 align 4 at (282, 55 + 32 *k*, 182, 24): 573 to 578 |

  The speed, scroll and quality buttons: frame 0, click frame 1, no hotkey, flags 2. Recomposed
  from the reading over `SysOpBck.pcx`, every item sits in its box of the picture.
- **Their states at the build** (0x5B2C7B on): every bar hidden (state 4 cleared) but the one at
  each volume, shown with the volume as its frame (`syslb.def` frame *v* marks bar *v*); each
  row's buttons enabled and its chosen one disabled (`SendCommand` 5 with 0x1000 sets state bit 8,
  E17-design §9.7: frame 2, the chosen look):
  223 + Walk Speed, 226 + Computer Walk Speed, 232 + Window Scroll Speed, 239 + Bink Video; each
  box's frame its setting. In the tutorial (game +0x1F69D) Move Reminder and Quick Combat show
  frame 0, disabled and inactive (state 2 cleared); in a network game Restart and Load are
  disabled (a message to id 0 among them reaches no item). No item has a hint.

### 4.2 The procedure and the run

- `system_options_proc` 0x5B3140 (vtable +0x24), as the combat options' (E17-design §12.12):
  `adventure_tick` first (the map's animations go on); a right press (0xC or 0xE with the right
  button's flag 0x200) on 102, 105 to 108, 30722 or 201 to 242 shows a popup (style 4) of
  `system_options_help` 0x6A7580's right-click text: Help.txt 320 to 325 for Main Menu, Load, Save,
  Restart, Quit and Return, 326 + (id − 201) for an id 201 to 242 (the bars, the speeds' names,
  the boxes, Low 364 and High 365; 346 to 348 have no item); a left press (0x200/0xC) on 235 to 242
  plays `button_sample` (volume 0x40, once, category 3), and the DEF buttons (the two quality
  buttons among them) play their own press sample too: High and Low (239, 240, inside 235 to 242)
  read as clicking twice (as the combat options' speeds, CLT-Q74: a DEF button's own loop swallows
  the press, so likely one click; checked when M4 builds the dialog, with a sound recording).
- **A click** (0x200/0xD) by id (switch 0x5B328C):
  - 102 Load, 108 Main Menu, 105 Quit: GENRLTXT 579 (style 2); OK (0x7805) closes the dialog
    with the id (the window manager's +0x38, the message 0x200/10, 2), Cancel keeps it open.
  - 106 Save, 107 Restart, 30722 Return: closed with the id at once.
  - 201 to 210: Music Volume = id − 201, 211 to 220: Sound Volume and Last Sound Volume = id − 211;
    but at volume 0 without the sound manager's digital driver (+0x3C) GENRLTXT 151 (OK) and
    nothing else; the bars set as at the build; `music_volume_apply` 0x59A1B0 or
    `effects_volume_apply` 0x59A0C0 with the manager's +0x84 at 1 for the call. The GENRLTXT 151
    guard is the combat options' (E17-design §12.12; the client never refuses a bar, CLT-Q44 (c),
    §4.7).
  - 224 to 227 Walk Speed = id − 223; 228 to 231 Computer Walk Speed = id − 226 and Blackout
    Computer = (it is 5); 232 to 234 Window Scroll Speed = id − 232; 239, 240 Bink Video =
    id − 239: the row's buttons enabled (state bit 8 cleared), the new one disabled (bit 8 set).
  - 235 Show Route, 236 Move Reminder, 237 Quick Combat, 238 Video Subtitles, 241 Town Outlines,
    242 Animate SpellBook: toggled, the box's frame set.
  - Each of these sets +0x60 and redraws the whole dialog.
- **The run** (`system_options_run` 0x5B30B0): modal; then with +0x60 set: a network game whose
  Quick Combat changed writes it into the screen player's +0xE4 and sends message 0x3F1 (not
  built); `settings_save` 0x50C1B0 (all settings to the registry, §4.5).

### 4.3 The results (`system_options_open` after the run)

By the id (switch 0x41AC8D): 102 Load, 105 Quit, 108 Main Menu set `screen_request` 0x6976D8 to
it and end play; 106 Save calls `game_save_dialog` (§4.4) and play goes on; 107 Restart asks
GENRLTXT 68 (style 2) with the dialog gone: OK sets the request 107 and ends play, Cancel returns
to the map (CLT-Q98); 30722 nothing. Then the hero lifted again when it was (`mobilize_hero` 0, 0,
1, called twice: no visible effect) and, when Walk Speed changed, the step sounds `horse00` to `horse10` loaded again (the manager's +0x360; the old +0x388 not
released). After play `game_main` closes the network unless the request is 107 (0x4EFED4) and its
menu loop takes the request (§2.2): 102 Load Game's menu (§2.4), 105 the end, 108 the main menu,
107 the restart (§4.6); the path between is the end of a scenario's (§5.3).

### 4.4 The save from play (`game_save_dialog` 0x418160, its end)

After the scenario screen in its save mode (§3.6) with a name: the extension appended
(`save_extension`); in a network game the current player is the viewing one for the save;
`game_save` 0x4BEEA0 (H3API's `H3Main::SaveGame`: the name, the tutorial flag; *not read*); the current player
back; on success the name cut at its first full stop and GENRLTXT 351 formatted with it in an OK
box. Play goes on.

### 4.5 Where the settings live

The registry key under HKEY_LOCAL_MACHINE named by `settings_names` 0x63FF74's first entry (the
game's `1.0` key), a DWORD per value name: read at the start (`settings_load`: zeroed,
`settings_read`, `settings_check`'s ranges, a first run's `settings_first_run`), written whole by
`settings_write` through `settings_save` (this dialog at its close when anything changed; the
combat options' likewise, E17-design §12.12). This dialog's:

| Setting (value name) | Global | Range, else | First run | Read in play by |
|---|---|---|---|---|
| Walk Speed | 0x69875C | 1 to 4, 2 | 2 | the human's walk (E24-design §3.4) |
| Computer Walk Speed | 0x698758 | 2 to 5, 3 | 3 | the computer's walks (E24-design §3.4) |
| Blackout Computer | 0x698790 | 0 or 1 | 0 | the enemy's moves not shown: `hero_select`, `hero_move_step`, the covered view and others (E24's; *not read*) |
| Window Scroll Speed | 0x69878C | 0 to 2, 1 | 1 | `view_scroll` |
| Music Volume, Sound Volume | 0x698760, 0x698764 | 0 to 9, 5 | 5 | the sound manager (E24-design §3.11) |
| Show Route | 0x698774 | 0 or 1 | 1 | the walk (E24-design §3.7) |
| Move Reminder | 0x698778 | 0 or 1 | 1 | End Turn's question with a hero able to move (`adventure_command` 0x409D31) |
| Quick Combat | 0x69877C | 0 or 1 | 0 | `battle_is_hidden`: the battle not shown, both players' +0xE4 in a network game |
| Video Subtitles | 0x698780 | 0 or 1 | 1 | a clip's dialog (`option_video_subtitles`' readers; *not read*) |
| Town Outlines | 0x698784 | 0 or 1 | 1 | the town screen (`option_town_outlines`' reader; *not read*) |
| Animate SpellBook | 0x698788 | 0 or 1 | 1 | the spell book's page turns (E17) |
| Bink Video | 0x6987A8 | | the speed test | the clip players (§2.1) |

Autosave (0x698770, 1 at a first run; `next_player_turn` reads it, 0x4C70B1) has no item here.
The combat options' values, Main Game Show Menu, X, Y and Full Screen, the tests' and Network
Default Name are no part of this dialog.

### 4.6 The restart (`game_main`'s case 107, 0x4EF1BC; read in part)

The busy cursor; the loading screen as `loading_screen_open`'s and its steps as
`load_progress_step`'s, inline; the eight player records and the chosen factions (game +0x1F6B0)
copied aside; `game_reload_reset` 0x4CECB0 (each player record reset, `game_reset`, the map's
header read again, two calls not read); a campaign takes its own path (`campaign_game`; E20, not read); per colour the
copied faction back when the map allows it, with `chosen_heroes` from the colour's first hero
(+0x1F844), the name (+0xCC) and both human flags (+0xE1, +0xE2) back, `scenario_bonus_choices`
from game +0x1F864; `current_player_first`; `new_game_load` with the map's file name (+0x1F7D4,
+0x1F6D9) and the version; the bar's last steps; then play (0x4EF80B). So a restart keeps the
players' setup, the towns as drawn at the first begin included (§3.17), and reads the map again;
the generator is not reset there, so its other draws are new (*inferred*).

### 4.7 For the client

- **CLT-Q96** (decision 15): Video Quality honoured: Low plays the Smacker copies where the
  original does (clips from 28 whose table entry has +8 set: `NWCLogo`, `H3Intro` in the shell;
  the campaigns' are E20's; `LoseGame`'s +8 is clear, so its Smacker copy plays at either
  setting: T1b3b's correction, §5.4); `shell-videos.vid` now holds the three copies (§1). The
  start's first-run case stays out (CLT-Q85): Bink Video is High until the player sets Low.
- **CLT-Q97** (decision 15): Restart builds a new game from the first game's `GameSetup` with each
  Random town fixed to the town the engine drew for it, and a new seed.
- **CLT-Q98** (decision 15): Restart asks its question (GENRLTXT 68) inside the dialog, as Load,
  Main Menu and Quit ask theirs; Cancel keeps the dialog open; the original's order for the
  comparisons only.
- **CLT-Q102** (decision 20): the boxes and bars act on a release over them alone, as the combat
  options' already do (CLT-Q43 (2)); the GENRLTXT 151 guard as theirs (CLT-Q44 (c): the client
  never refuses a bar); both carried by the one implementation of a dialog's buttons, boxes and
  bars shared by the battle's and the shell's dialogs (`h3_view::options` the battle's); the
  original's for the comparisons only.
- **The agent's calls** (*provisional*, M1's review list): the settings in the user's
  `settings.toml` beside the combat options' (one set of the registry's values, the volumes and
  Animate SpellBook shared), written whole at the close when anything changed; Autosave kept there,
  on, with no item; the network paths (Quick Combat's message, Load and Restart disabled) and the
  tutorial's disabled boxes not built (CLT-Q84); the step sounds' reload and the second hero lift
  not copied (nothing visible). The settings' effects in play are the map client's and the battle
  client's (the map-client lane's inbox told).

### 4.8 For T2's captures

The dialog over the map with a first run's settings and with each row's other choices; a bar of
each volume; each box both ways; a right press's popup per item kind; the three questions (579 for
Load, Main Menu, Quit; 68 for Restart) and the save's message (351); Return and Escape.

## 5. The end of a scenario (M1 T1b3b, 2026-11-12)

Read whole: `scenario_condition_met` 0x4F2CE0, `scenario_victory_test` 0x4F15E0,
`scenario_loss_test` 0x4F2960, `game_main` from 0x4EF80B (but the campaigns' path, §5.7),
`scenario_lose_screen` 0x4EE2B0, `scenario_win_screen` 0x4F3F60, `scenario_score_screen` 0x4F3AB0,
the scores (0x4F3E30, 0x4F3EB0), `high_score_entry` 0x4E91D0 with its items, procedure and edit.
Who meets a condition, and when, is E10's (the engine): this section is what the player sees.
T1b3b was kept whole (decision 16).

### 5.1 Victory and defeat in play (`scenario_condition_met` 0x4F2CE0)

- **The call:** ecx 0 a check (most of its 37 call sites: the events, battles, visits, town visits,
  the turn; E10's), 1 a won game and 2 a lost one (the typed codes, 0x4F43F3 and 0x4F4406; the win's
  first sets the cheat flag game +0x1F69C and, in a campaign, the campaign's own +0x1F458; 2 also
  from `hide_whole_map` 0x402822). It returns at once with `conditions_checked_here` 0x697788 clear
  (1 outside a network game), during a game's setup (0x698400), once `scenario_ended` 0x6972B8 is
  set, or re-entered (`condition_check_guard` 0x69958C). In a remote turn the viewing player's
  record is made human for the checks and put back (network; not built).
- **The eliminations**, per colour 0 to 7 not yet defeated (game +0x1F636): no hero and no town:
  `player_defeated` 0x4F11A0, then for the viewing player GENRLTXT 96 (its elimination), else
  GENRLTXT 6 with the colour's player name (`colour_player_name` 0x4CEB60) and its crest (picture
  type 10: `crest58.def`, the colour's frame and name; a network timeout of 5 s). Heroes but no
  town: the days without a town (player +0x3D) −1 become 7, with GENRLTXT 7 (the week's warning, the
  crest) when the colour is a human here whose turn it is; at 0 `player_defeated`, with GENRLTXT 8
  (the same human) or 9 (any other; both with the crest); GENRLTXT 7 and 8 take the player's name
  first (", you" when it is empty, as a single scenario's human's is, §6.3; CLT-Q100). A computer's
  name is its colour's (recorded, M1 T3: `end-conquest` purple-6, "Purple has been vanquished!",
  the crest captioned Purple). A town: −1.
  The days' count down is E10's (not read). Every box is an OK box (style 1) at the screen's middle.
- **The scenario's conditions:** the enemies counted (the colours neither defeated nor on the
  viewing player's team, game +0x1F879); the victory condition (game +0x1F89C, 0x4C bytes) through
  `scenario_victory_test`, and when it finds nothing the loss condition (game +0x1F8E8, 0x24 bytes)
  through `scenario_loss_test` (§5.2): a loss sets `scenario_ended` and `scenario_result` 0x699510
  0, a win `scenario_ended` and 1.
- **The standard victory** counts when the map has no special one (type 0xFF) or allows the standard
  one too (+0x1F89D): with no enemy left, `victory_by_conquest` 0x69951C 1, `scenario_ended`, won (a
  network game sends message 0x436), GENRLTXT 660 in an OK box. This test follows the special
  conditions', so both boxes can show. Else, with no colour left that is not defeated and a human
  here: ended, lost (no box beyond 96's). Last the argument: 1 ends the game won, 2 lost.
- **Play ends** at the adventure manager's next message: with `scenario_ended` set
  `adventure_manager_procedure` answers 2 with the executive's exit (0x4000, subtype 1; 0x4087CC),
  so `executive_loop` returns into `game_main` (§5.3). The flag's other readers (the town visit, the
  turn, `new_day`'s week and month, a message box's build, the computer's turn) stop their work (not
  read). It is no network client's flag (the name until T1b3b).

### 5.2 The conditions' messages (`scenario_victory_test` 0x4F15E0, `scenario_loss_test` 0x4F2960)

- **The victory's** (ecx the condition, edx the won flag; on the stack the lost flag and a network
  byte, 0 for the local check, 1 for another computer's message 0x3FB, `network_victory_received` 0x556940): nothing until
  the condition's met flag (+0x48) is set; +0x49 the colour that met it. On the viewing player's
  team it is won, else lost (also with no viewing player); code 5 counts only for the viewing
  player's team (another team's defeat of the hero shows nothing). The local texts (GENRLTXT; an OK
  box, no picture):

  | Code | Condition | Won | Lost | Formatted with |
  |---|---|---|---|---|
  | 0 | an artifact | 281 | 282 | the artifact's name |
  | 1 | creatures | 277 | 278 | the count and the creature's name (singular for 1) |
  | 2 | resources | 279 | 280 | the amount and the resource's name |
  | 3 | a town's upgrade | 283 | 284 | |
  | 4 | the Grail's building | 285 | 286 | |
  | 5 | a hero's defeat | 253 | none | the hero's name |
  | 6 | a town's capture | 250 | 251 | the town's name |
  | 7 | a monster's defeat | 287 | 288 | |
  | 8 | the dwellings | 289 | 290 | |
  | 9 | the mines | 291 | 292 | |
  | 10 | an artifact's transport | 293 | 294 | |
  | 11, 12 | (no shipped map: `h3m.md` rejects the codes) | the executable's own English texts, not GENRLTXT; won whoever met them | none | |

  With the network byte the texts name the team (`team_names_text` 0x4F1460: the team's colours'
  names joined with " & " and ", & "; one player or several choose between two records): 632 to
  654, a won one for codes 0, 3, 4, 7 and 10 (638, 639, 642, 645, 652); a local check in a network
  game sends 0x3FB with the condition and sets `scenario_ended` itself. A campaign's code 0 has
  three scenarios' own texts (714, 764, 738; E20).
- **The loss's** (the same arguments; a remote one from `network_loss_received` 0x5569A0, message 0x3FC): its met flag
  +0x22, +0x23 the colour it concerns. Code 0 (a town, its position +4, +8, +0xC) for a colour on
  the viewing player's team: lost, GENRLTXT 252 with the town's name; 1 (a hero, +0x1C) the same,
  254 with the hero's name when the colour is the viewing player's, 671 with the colour's player
  name and the hero's for a teammate; 2 (the time; no team test): lost, 255. An OK box, no picture;
  in a network game the local check sends 0x3FC and sets `scenario_ended`.

### 5.3 Play's end (`game_main` from 0x4EF80B)

- **Play and Quit** both pass 0x4EF80B (High Scores and Credits return to the loop's top): the
  menus' pictures freed (`mainmenu.pcx`, `GamSelBk.pcx`), every clip stopped (`video_stop_all`
  0x5975F0: the pause count undone, Bink's handles paused, Smacker's closed, `video_close`); Quit's
  flag (0x4EF804) goes to the loop's top and the end (`game_shutdown`); else play (0x4EFC58).
- **Play:** the window manager's +0x44 1; `conditions_checked_here` chosen (0x41A660: in a network
  game whether the next human to play is here; not built); the effects stopped (`effects_stop_all`
  0x599D90); a campaign's own loading (§5.7); the adventure manager added to the executive (a
  failure: GENRLTXT 1, the game ends); a network game sends Quick Combat (0x3F1); a new game (101),
  a restart (107) or a campaign's next scenario (`first_day_due` 0x699584) plays `newday.wav` and
  runs the first day's map and town events; the turn's timer started (`turn_timer_start` 0x557D90;
  the hot seat's, E24 M5); `executive_loop` until its exit (§5.1, or a request of the system
  options, §4.3). After it: the sound manager's +0x84 1, `remote_turn` 0, the effects stopped, the
  adventure manager removed, the window manager's 0x602C50 (1, 4, 0) (a redraw, *inferred*); the
  network closed unless the request is the restart; a lobby's start ends the game.
- **The end's path** (0x4EF840): with `scenario_ended` clear, the loop's top (the request, §4.3).
  Set: the network closed, the view shown (0x6989C0), the arrow cursor; GENRLTXT 116 formatted with
  the day's number (game +0x1F63E's date) into 0x699294, which no code reads (a leftover: nothing
  shows it); then by `scenario_result`: 0 the lost screen (§5.4); in a campaign its path (§5.7);
  else the won screen (§5.5), its entry always setting `high_scores_due` 0x69955C, so "Win Scenario"
  is asked for again (still playing: dropped, E24-design §3.11). At 0x4EFF2B `scenario_ended` 0;
  with `high_scores_due` set it is cleared and `high_scores_show` runs (§2.8, on the entry's tab
  with its row in colour 8); then the menu loop's top (§2.2). "Win Scenario" plays on through the
  entry and High Scores (neither has music of its own).

### 5.4 The lost scenario (`scenario_lose_screen` 0x4EE2B0)

- Clip 34 `LoseGame` (`video_play_in_dialog` at (0, 0), its own size, not playing on, its frames
  shown by the clip's tick): its table entry's +8 is clear, so the Smacker copy `LOSEGAME.SMK` (800
  × 600, 126 frames at 10 a second) plays at either Video Quality and `LOSEGAME.BIK` never does (§1,
  §4.7). `UltimateLose` once (loop count 1), the effects ended. Three times 500 ms asleep with the
  music served (`music_service` 0x59A7D0): for 1.5 s input waits; the input flushed, a due frame
  shown.
- While the clip plays: the messages, the next input message; a key down but F4, a left press (8) or
  a right press (0x20) ends it but on the first run (`first_run_setting`, as the start's clips,
  §2.1); any other message shows a due frame.
- Then every clip stopped, the window manager's 0x602C50 (1, 4, 0); when the clip ran to its end,
  the screen waits while the music streams (status 4; served, 100 ms asleep): `UltimateLose` to its
  end. A skip ends the screen at once. No text, no button.

- *Recorded (M1 T2, §6.3; a correction):* the 1.5 s wait is real time, not the game's clock; the
  clip shows its first frame 700 ms before the second.

### 5.5 The won scenario (`scenario_win_screen` 0x4F3F60, `scenario_score_screen` 0x4F3AB0)

- **The call** (ecx 1 a scenario, 0 a campaign: §5.7): the mouse hidden, the window manager's +0x44
  0.
- **The score:** the base (`scenario_base_score` 0x4F3E30) 200 − (days + 10) / (towns + 5) by
  integer division (the screen player's towns, player +0x3E), + 25 when `grail_finder` 0x69950C is
  the viewing player (set when a human digs up the Grail, 0x40EFF3; −1 by `game_reset`; saved), + 25
  with `victory_by_conquest`; the final score the base times the difficulty's factor
  (`difficulty_score_factors` 0x67F558 by game +0x1F6D8: 0.8, 1, 1.3, 1.6, 2; truncated;
  `scenario_score` 0x4F3EB0 computes the same for a campaign's record); the percent the factor ×
  100; the land the map's name (game +0x1FB40; empty when none).
- **The rank's text:** `high_score_creature` (the final score, the tab, §2.8): a creature 0 to 150
  gives its name (the creature record's +0x14), else empty; with the cheat flag GENRLTXT 261; its
  first letter upper case.
- **The clip:** 35 `hsAnim`, its loop `hsloop` (Smacker only; 800 × 600, 135 and 45 frames at 10 a
  second), at (0, 0), playing on (the loop repeats), its frames not shown by the tick; "Win
  Scenario" looping (count 0), the effects ended.
- **The score screen** (`scenario_score_screen`; edx the rank's text, then the base, the final
  score, the days): `HiScore.fnt`; the labels GENRLTXT 439, 440, 441, 442 and 677; the input
  flushed, the clip's frame decoded again; per column *i* from 0 to 4 the label in (160 *i*, 450,
  160, 100) and the value in (160 *i*, 540, 160, 50), colour 0x119, alignment 5: the days, the base,
  the difficulty's name (`difficulty_names`), the final score, the rank's text; the screen shown.
  While a clip plays: `adventure_tick`, the messages, the next input message; a key down but F4, or
  a left or right press or release (8, 0x10, 0x20, 0x40) ends it (no first-run guard); every due
  frame gets the ten texts drawn again over it before it is shown. The font freed.
- **Then** every clip stopped; `high_score_entry` with the final score, the days, the percent, the
  tab and the land (§5.6); the mouse shown, the window manager's +0x44 1 and 0x602C50 (1, 4, 0).

### 5.6 The new entry (`high_score_entry` 0x4E91D0)

- **The rank:** the manager's +0x8D0 takes the tab (High Scores' first, §2.8), the row is the tab's
  table's (§2.8). With the cheat flag, or in a campaign whose own flag (game +0x1F458's first byte;
  E20) is set, it is 10; else the first of rows 0 to 9 whose score is below the new one or −1
  (empty), else 10. Below 10, the rows from it to 9 move down one and row 10 is lost. So row 10, the
  eleventh, takes every score that misses the ten, whatever it held (CLT-Q99: copied).
- **The dialog** (`high_score_entry_vtable` 0x63EBBC): the base dialog at (284, 194), 232 × 212,
  flags 0x12 (centred); its items in order:

  | Id | Item | Place, size | Detail |
  |---|---|---|---|
  | 500 | `HighName.pcx` | (0, 0), 232 × 212 | flags 0x800 |
  | 501 | the name's edit (`high_score_name_edit_create` 0x4E96A0: H3API's `H3DlgEdit`, vtable 0x63EBF4) | (18, 105), 199 × 23 | at most 40 characters, `smalfont.fnt` colour 4, alignment 4, 0x100 and no border (H3API's parameter names) |
  | −1 | GENRLTXT 97 (the congratulation, the name asked for) | (13, 13), 205 × 100 | `smalfont.fnt` colour 4, alignment 1, flags 8 |
  | 503 | OK, `mubchck.def` (64 × 32, 4 frames) | (84, 143), 64 × 32 | frame 0, click frame 1, not closing, hotkey Enter (0x1C), flags 2 |
  | 504 | an empty text | (8, 186), 216 × 18 | `smalfont.fnt` colour 1, alignment 1, flags 8; no item sets it (a status line, *inferred*) |

- **The run:** the arrow cursor shown; the edit focused, its text the screen player's name as
  `player_display_name` 0x4BADB0 gives it (the record's +0xCC; a human's "Player" or a computer's
  GENRLTXT 469 becomes the colour's name, written into the record, as read; but a single
  scenario's human holds an empty name, which stays empty: recorded, §6.3, the edit starts empty;
  CLT-Q100 gives the client's player name, the colour's name when that is empty; the first letter
  upper case);
  modal with `dialog_proc_default`, whose click goes to `high_score_entry_command` 0x4E9790: OK
  (503, or its hotkey Enter) ends the dialog (0x7802) when the edit holds a text; an empty name does
  nothing. No Escape, no Cancel. The edit takes the keys while focused (0x4E9710).
- **The record** (100 bytes, zeroed first): +0 the name, +0x29 the land (GENRLTXT 261 with the cheat
  flag), +0x54 the final score, +0x58 the days, +0x5C the percent, +0x60 the cheat flag;
  `high_score_newest` of the tab the rank; both tables (0x898 bytes from +0x38) written to
  `.\DATA\HiScore.dat` (opened to write, created, truncated, binary; a failure `file_error_box` with
  the name); `high_scores_due` 1.

- *Recorded (M1 T2, §6.3; a correction):* a single scenario's human has an empty name: the edit
  starts empty, OK and Enter do nothing until a name is typed (CLT-Q100); the hot seat's names show.

### 5.7 The campaigns' path (noted; E20)

`game_main` 0x4EF8C9 to 0x4EFC56 (a campaign's scenario, its record at game +0x1F458: the next
scenario, `Endgame` (clip 32) in `IntroRim.pcx` at a campaign's end, the won screen with the
campaign's score, the campaigns' screens) and 0x4EFC8D to 0x4EFD8E (its loading bar, then
`scenario_result` 2).

### 5.8 For the client

- The engine decides victory and defeat (E10; until then decision 6's stubs). The shell shows §5.1's
  and §5.2's local boxes in their order, then the screens; the network's texts and messages (0x3FB,
  0x3FC, 0x436) are not built.
- What the shell needs from the engine (the adventure lane's inbox told, for E10): per colour its
  elimination and why (no hero and no town, the week without a town), the week's warning; the result
  for the shell's player with the condition's code (or the standard victory) and the values its text
  takes (the artifact, the count and creature, the amount and resource, the hero, the town); for the
  score the day, the player's towns, the difficulty and whether the player dug up the Grail.
- **CLT-Q99** (decision 17): the eleventh row as the original's.
- **CLT-Q105** (decision 21): the score is the engine's (E10 computes and reports it with the
  victory); the shell shows it, a `CLT-STUB` until E10. **CLT-Q106** (decision 21): a turn that
  meets a special victory and the standard one shows the first box alone; the original's two for
  the comparisons only.
- **CLT-Q100** (decision 20): a single scenario's player takes the client's player name, the
  colour's name when that is empty (GENRLTXT 7 and 8, the entry's edit); the original's empty name
  for the comparisons only.
- The lost screen plays `LOSEGAME.SMK` always (`shell-videos.vid` holds it); skippable after 1.5 s
  (CLT-Q85: no first run).
- **The agent's calls** (*provisional*, M1's review list): the scores in the client's own file
  (§2.8), never `HiScore.dat`; GENRLTXT 116's unread text not copied; the cheat flag, the typed
  codes and the "Cheater" land not built (the client has no cheats); codes 11 and 12 not built (no
  map has them); the map's animations under the score screen (`adventure_tick`) not copied (nothing
  shows); the network's paths not built.

### 5.9 For T2's captures

The eliminations' boxes (96; 6 with the crest; 7, 8 and 9) and each victory and loss text a
recipe can reach on a shipped map; the standard victory's 660; the lost screen to its end and
skipped after 1.5 s; the score screen (each column over the clip's loop) and its end by a key and
by a press; the entry with the player's name, OK with an empty name, Enter; High Scores after it
with the new row in colour 8; a score that misses the ten (the eleventh row). No clip beyond
`shell-videos.vid` and `battle-videos.vid` is needed.

## 6. The captures (M1 T2, 2026-11-13)

The harness's recipes for every catalogued screen: `clients/capture/h3shell.py` and
`clients/capture/shell/` (the runbook, "The shell's screens"), each recipe recorded twice on a
clock frozen from the game's first read, the digests in `shell/kept/<group>.txt`.

### 6.1 The groups

| Group | Catalogue | Recipes | Kept shots, clips | Notes |
|---|---|---|---|---|
| `start` | §2.1 | 6 | 9, 3 | Show Intro off; the clips with Bink Video 1 and 0; one skip way per recipe: 3DOLogo by a key (Space), NWCLogo by a left press, H3Intro by a right press |
| `menus` | §2.3 to §2.5 | 5 | 88, 1 | each button's hover, press, release off it, popup; Quit's question; the tutorial's load |
| `multiplayer` | §2.6, §2.7 | 10 | 155, 3 | stage 1 over New and Load Game's menus, the hot seat's names, the other modes' stage 2; since M2 T7 the edit's keys and two 21-letter names (§16) |
| `scores` | §2.8, §2.9 | 4 | 43, 8 | both tabs and the creatures' steps, Reset, the credits' scroll whole and ended early |
| `scenario` | §3.1 to §3.11 | 13 | 178, 2 | new, load and save modes; the list, sorts, filters, the panel's marks, Begin's messages |
| `advanced` | §3.12 to §3.18 | 11 | 205, 2 | single and hot seat (two to eight), every row's arrows, the popups, the random map's tab, the begin |
| `options` | §4 | 7 | 161, 1 | the dialog over the map, every row and box, the bars, the popups, the questions |
| `end` | §5 | 30 | 51, 7 | the eliminations' boxes, 660, the victory and loss texts below, the lost screen, the score screen, the entry, the eleventh row |

In all, 84 recipes, 864 shots and 27 clips. Two recordings the same in every recipe but one: `start-smacker`'s 2 586 frames are the same
frames in the same order in both runs, but from H3Intro's first frame on the second run's times
are 10 ms earlier (the Smacker copy's open fell inside a clock step; its kept line keeps both).

### 6.2 The harness's findings (the runbook and `TRAPS.md` hold the rules)

- `SMACKW32.DLL` reads `timeGetTime` itself (now on the virtual clock); in a full-screen Bink clip
  the game's loop reads no clock (only `BinkWait`); `BinkGetSummary` (the game asks it at a
  clip's end) divides the bytes read × 1000 by the read time + 1, which a frozen clock leaves 0:
  a clip of 4 MB or more overflows (`xdd.dll` guards the call).
- The game draws its cursor at the pointer only on a later clock tick (some 50 ms); a step given
  while the game is still loading or has not yet taken an input moves the following times by a
  step (the tool waits for the game's idle loop and, for an input that starts a load, a `settle`).
- A clip's frames are due from the last one shown, so their times depend on the step (the 3DO
  logo at 1 ms, the rest at 10 ms); one long step advances a clip one frame.
- The loading bar's steps, the lost screen's 1.5 s wait and Restart's bar run while the game reads
  no clock or by real time: their shots show the bar full, and a key inside the wait cannot be
  timed.
- Wine: after a network mode is tried, its stage 2's Host, Join and Search and stage 1's modes
  (Hot Seat included) are disabled until the dialog reopens, likely Wine's DirectPlay (not built,
  §2.10); TCP/IP's stage 2 shows the sandbox's address (192.0.2.2: since M2 T4 its line is
  masked in `mp-tcp`'s digests, `mask:180,96,620,124`); Online ends the game.

### 6.3 What the recordings add to or correct in the readings

- **The start** (§2.1): the main menu shows at 0 ms with Show Intro off, the arrow already drawn
  at the parked pointer (400, 300; M2 T4's first exact shot: `GamSelBk.pcx` copied whole, the five
  buttons in frame 0, `crdeflt.def`'s frame 0 with its hot spot at the corner); 3DOLogo's first frame at
  350 ms; the clips' frames as their headers say; each skip ends the sequence, the main menu next.
  The lengths: `start-bink`'s menu at 180.48 s is 350 ms and the headers' 5.40 + 13.00 + 161.73 s;
  the Smacker copies have the same frames and rate (§2.1), so `start-smacker`'s 9.1 s more (189.6 s)
  are the steps': its frames are due from the last one shown (§6.2), at 10 ms steps every 70 ms,
  not 66.66 (2 702 × 70 ms + 350 ms = 189.5 s; Bink's own pacing keeps its rate; *inferred*).
  Settled at M2 T5a (§10, from `start-bink`'s and `start-smacker`'s frames): NWCLogo's placement,
  the end-of-clip fade, the black screen before 3DOLogo's first visible frame (its frames 0 to 3
  black, frame 4 at 335 ms) and between the clips; the Smacker frames' 70 ms at 10 ms steps
  read in `SmackWait`.
- **Credits** (§2.9, a correction): the closing text (record 2) shows only near the end, once the
  text's end has risen to about 435 px (first at 244.6 s into the scroll), not while little text
  is shown at the start; the first frame comes 1 100 ms after the screen opens, then one every
  100 ms of 2 px (20 px a second); the last at 265.2 s, the main menu at 266.5 s. A pointer move
  does not end it; Space, a left and a right press do.
- **High Scores** (§2.8): the creatures step every 210 ms on 10 ms steps (a test of "more than
  200"); a right press gives no popup; C and S switch the tabs; Escape, Enter and Exit return.
- **The hot seat's names** (§2.7): OK is enabled by the key that makes the second name (not at a
  focus change) and disabled again by its removal; a disabled OK still shows its hint and popup;
  Enter in an edit moves to the next edit, never presses OK. Stage 1's scroll bar has no hint
  (the map's picture covers it); after Escape from stage 2 the status line keeps the last hint
  until the pointer moves (CLT-Q101).
- **The system options** (§4): GENRLTXT 151 shows only when the volume pressed from is 0 (the
  harness runs with no sound driver: `opt-bars` and `opt-questions` show it from volume 0); boxes
  and bars act on a release off the item, buttons do not (CLT-Q102); in the questions Escape
  cancels and Return confirms (Quit by Return ends the game).
- **The end** (§5, corrections of §5.4 and §5.6): a single scenario's human has an empty name, so
  GENRLTXT 7 and 8 begin with ", you" and the entry's edit starts empty, OK and Enter doing nothing
  until a name is typed (the player's name only in the hot seat; CLT-Q100); the lost screen's
  1.5 s wait is real time, and its clip shows its first frame 700 ms before the second.
- **Reached** (`end`, the conditions met by writing the game's memory, then the game's own turn
  and boxes): the eliminations 96, 6 (with the crest), 7, 8, 9; the standard victory 660; victory
  won/lost 281/282, 277/278, 279/280, 283/284, 285/286, 250/251, 287/288, 289/290, 291/292,
  293/294 and 253 (code 5 has no lost text); loss 252, 254, 255. **Not reached:** 671 (neither
  shipped map with the hero's loss has teams), 632 to 654 (network), 714, 764, 738 (campaigns),
  codes 11 and 12 (no shipped map), 261 (the cheat flag), a special victory's box followed by 660.
- The `scenario` and `advanced` groups' notes are in their recipes' `what` lines; their agent's
  list of oddities was lost with a sandbox restart, so M1 T3's reviewer goes over their shots.

### 6.4 Left for M2 to M4 (M1 T3's review, 2026-11-13)

- **Sound** (all of §2 to §5): recorded by the task that builds each screen, its sequences and
  the waits a real sound driver makes (the lost screen's `UltimateLose` wait among them, §5.4),
  with a recipe's `sound = true` (h3shell's since M2 T4: the effects at volume 5, each sample's
  start kept). M2 T4's `menu-sound`: a main menu button's press and New Game's Back each start a
  151 ms sample at the press (`button.wav`'s length), and the menu after it shows at the silent
  recipes' times with their pixels: a press does not wait for its sample (T6). M2 T5b2: the clips'
  sound on the clock too (§12); AB's intro (`H3X1intr`, an 8-bit track) recorded with sound
  where it plays (E20's prologue) reads the game's Miles buffer for an 8-bit track
  (`bink-minimum`, §12.2: at a 16-bit track's size its sound would never start).
- **The fades** (§5.3 to §5.5): 0x602C50 (1, 4, 0) is read as a redraw (*inferred*), but
  `opt-restart`'s `fade` clip and `end-conquest`'s `to-scores` record fades: M4 reads it.
- **The edit field** (§2.6, §2.7: typing 0x5BAC50, the caret, the limit): read and recorded at
  M2 T7 (§16.1, `edit-keys`).
- **The names' length** (§2.7): 21 characters against 21 bytes each (0xAC = 4 + 8 × 21), so a
  name of 21 letters may lack its terminator; M2's hot seat task records two 21-letter names first
  and asks a CLT-Q if they join. Recorded at M2 T7 (`hotseat-long`): an edit holds 20, the names
  never join (§16.2).
- **The requests** (§2.2): `adventure_key_down`'s 101, 102, 107, 108 and `adventure_command`'s
  107: M4 reads them with the map-client lane.
- **The lost screen's first frame** (§5.4): shown 700 ms before the second, maybe the frozen clock
  against the real-time 1.5 s sleep: M4 settles it before copying.
- **`difficulty_names`' text source** (§5.5): M3.
- **High Scores' creatures** (§2.8): "the map's last monster template" at the main menu: M4,
  captured after a played map.
- **The score** (§5.5; the engine's, CLT-Q105, decision 21: E10 computes it): the f32 factor
  against x87 at the boundaries, no recipe at another difficulty, the Grail's +25, an empty rank
  text: M4 records them, for E10's test and the shell's screen.
- **Not reached or not recorded** (the task that builds each records it): past a special
  victory's or a special loss's box (§5.1, §5.2, §6.3); a 40-character name (§5.6, its clipping
  unread); the system options in another colour than Red (§4.1); the hotkeys' press frames (§2.3,
  §4.1); a skip of a Smacker copy (§2.1); a hot-seat restart (§4.6).
- **Recipes for M3** (M1 T3's shot review): the date's field order (§3.5: `scn-load`'s
  `games-time` gives day = month, and Wine's time zone enters the digest: a day above 12 and a
  pinned TZ); `up-past-top` never reaches the list's first row (§3.4: Page Up first; done at M3
  T1b1, `up-at-first`, §19.5);
  `popup-hero-arrow` pressed a computer row (§3.13's hints: row 2's arrow; done at M3 T3c2 after
  a town's press on that row, §27.3); `scn-load` size-1
  changes nothing (both saves XL: a save of another size); the Random Map button in a load (`adv-load`,
  §3.16); Begin after `tab-again` (§3.16); `scn-begin-none`'s `what` line (530 is a filter's,
  §3.6); GENRLTXT 503 (reading the map files, §3.3) in no shot; Delete of the last save not
  recorded (§3.4).

## 7. The Smacker clips' decoding (M2 T1, 2026-11-13)

`h3-smacker` is FFmpeg 6.1.1's Smacker demuxer and video decoder ported (decision 5); this section
holds what the check against the game's `SMACKW32.DLL` found (`clients/scripts/smacker-check.sh`,
`clients/capture/smackprobe.c`), for T2 and T5.

### 7.1 The port and FFmpeg

- Ported: the header, the frame table and the trees (`smacker_read_header`), the frames in order
  with the palette's changes and the audio packets skipped (`smacker_read_packet`, the audio
  streams discarded; read since T2: `Packet::audio`, §8.2), the four header trees and the codes' cache (`smacker_decode_*`,
  `smk_get_code`), the blocks of both versions (`decode_frame`). The byte trees are walked bit by
  bit (FFmpeg builds lookup tables from the leaves' lengths: a tree read depth first is complete,
  so its codes are its paths). `H3INTRO.SMK` (2 426 frames of 640 × 360) decodes in 4 s in
  release.
- Checked against FFmpeg 6.1.1's command line (`ffmpeg -i <clip> -map 0:v -f rawvideo -pix_fmt
  pal8`, Ubuntu's 6.1.1 build; not committed): the shell's eight clips, AB's `Credits.smk` and 120
  samples of each version, every frame and palette the same.
- The trees' sizes in the header are RAD's: four bytes an entry and three more for the cache, in
  every one of the game's clips (`TreeSpec::size`). All the game's clips are `SMK2`, without
  key-frame marks, with every byte tree present.

### 7.2 The DLL as the probe runs it

- `SMACKW32.DLL` is RAD's of 1994 to 1997 and knows `SMK2` only (no `SMK4` in it): the `SMK4`
  samples are FFmpeg's check only.
- The probe opens a clip as `smacker_open` 0x598790 does (`SmackOpen` on the archive's handle at
  the entry's offset, the flags | 0x1140, the extra buffer −1) but with no sound track, then
  `SmackToBuffer` (left 0, top 0, the pitch, the height, the buffer, the type: 0 for 8 bits, the
  game's 0x80000000 or 0xC0000000 for 5-5-5 or 5-6-5, `smacker_open_in_dialog` 0x598AF0), then per
  frame `SmackDoFrame` and `SmackNextFrame`.
- The handle's fields read: width +4, height +8, frames +0xC (the ring frame not counted), the
  ring flag in +0x14, a new palette +0x68, the palette +0x6C (768 bytes: the 6-bit levels as 8-bit
  ones, FFmpeg's `smk_pal` values exactly), the frame's number +0x374. After the last frame
  `SmackNextFrame` gives the ring frame (`HSLOOP`'s 46th, `CREDITS.SMK`'s second).
- The game's Smacker tick (`smacker_tick` 0x598EB0) is `SmackWait`, `SmackDoFrame`, `SmackNextFrame` and
  `video_show`; nothing redraws a clip on a palette change.

### 7.3 Where the DLL reads otherwise than FFmpeg (no game clip has either)

- **A byte tree left out** (a header tree's low or high bytes, its bit 0; FFmpeg reads the bytes
  as 0): the DLL faults on memory outside its tables or decodes other pixels.
- **Bit 0 of a frame's size** (FFmpeg's key-frame mark): the frames after it are read from
  elsewhere (a sample's next palette came one byte off), decode other pixels or fault.
- The samples (`build::sample`) keep to the rest; repeated values, bytes and escapes in a tree
  (which RAD's encoder would not write either) read as FFmpeg reads them (the 150 samples have
  them). FFmpeg's handling of both cases stays the decoder's (the unit
  tests), as `h3-bink` keeps FFmpeg's on malformed data.

### 7.4 The 16-bit copy (CLT-Q119, decision 24)

With a 16-bit type the DLL writes, for each block a frame writes (mono, full, fill), its pixels
through the frame's palette (each 8-bit level's top bits: red and blue >> 3, green >> 2 in 5-6-5,
>> 3 in 5-5-5); a skipped block keeps its 16-bit pixels, so after a palette change it keeps the
old colours. `h3_smacker::check::original_16` makes the same copy from `Frame::written`: every
frame of the nine clips in 5-6-5 and of `NWCLOGO` and `HSLOOP` in 5-5-5 the DLL's. The clips with
more than one palette: `HSLOOP` 16 of 46 frames with stale pixels (at most 88 509), `NWCLOGO` 164
of 195 (at most 317 998), `H3INTRO` 1 892 of 2 426 (at most 102 208); the worst pixel 112 levels
off in a channel (`NWCLOGO`'s frame 30, its fade). The shell converts every pixel (decision 24).
Which type the game passes is the screen's format (0x598BE5's test, *not read*; the harness's
screen is 5-6-5).

### 7.5 For T2 and T5

- The audio tracks: the game's clips have track 0 at 22 050 Hz, flags 0xF0 (packed, present,
  16-bit, stereo; `Track`); the reader skips their packets (a decoded size first, FFmpeg's
  `duration_size`; handed on since T2, §8.2). The probe opens no track: T2's DLL reading of the sound starts there.
- The frame time: −6666 (`3DOLOGO`, `NWCLOGO`, `H3INTRO`) is 66.66 ms, −10000 (`HSANIM`,
  `HSLOOP`, `LOSEGAME`) 100 ms, −11111 AB's `Credits.smk`, 100 (`ProgressBar`, `CREDITS.SMK`)
  100 ms.

## 8. The clips' sound (M2 T2, 2026-11-14)

`h3_smacker::sound` and `h3_bink::audio` decode the clips' tracks; `h3_view::clip_sound` makes
their samples and volume for the host. This section holds how they were checked against the
game's DLLs (`clients/scripts/smacker-check.sh`, `bink-check.sh`; the probes in
`clients/capture/README.md`, "The clips' sound") and what the readings left for T5.

### 8.1 The probes

- **A stand-in Miles** (`clients/capture/mss32.c`, built by `setup.sh` into the run directory's
  `mss/MSS32.DLL`): the probes load it by its path before the DLL asks for `MSS32.DLL`. It
  records every buffer the DLL loads into a sample (`AIL_load_sample_buffer`, the bytes in order)
  and runs the DLL's Miles timers only when the probe pumps them, on the probe's thread. Its
  driver answers preference 15 (`DIG_USE_WAVEOUT`) with 1, so `SMACKW32.DLL` streams through the
  double buffers; the game's driver answers 0 and the DLL writes the same ring into a
  DirectSound buffer of its own on Miles' DirectSound object (0x100095C0, §11.1).
- **`smackprobe audio`**: `SmackSoundUseMSS`, `SmackOpen` with the tracks' flags 0xFE000,
  `SmackVolumePan` at the game's 3640 × 9, then every frame (the ring frame too), the timer
  pumped up to 64 times a frame and 256 times at the end: what the DLL hands Miles.
- **`binkprobe sound`** calls `BINKW32.DLL`'s decoder as `BinkDoFrame` does (0x10005A90):
  `BinkAudioDecompressOpen(rate, channels, 1)` (0x10005195), then per frame's packet
  `BinkAudioDecompress` until its decoded size. The DLL's playback was not used: it drops what
  does not fit its ring (sized by the track's largest packet) when Miles takes it late, which a
  stand-in without a clock cannot pace. `sound-tables` writes the 256 band scales (0x1000F310) and
  the FFT's twiddles of 1024, 2048 and 4096 values (its `cdft` 0x1000C610); `cdft` runs the FFT
  alone on given values (the fuzzing).

### 8.2 Smacker's sound

- FFmpeg 6.1.1's `smka_decode_frame` (packed tracks, the game's) and its PCM for plain tracks,
  ported; each packet stands alone (its trees, its first samples); the sums wrap.
- Checked: the shell's Smacker clips' tracks (packed, 22 050 Hz, 16-bit stereo, §7.5; `HSLOOP`'s
  ring frame included, its 8 820 bytes; AB's `Credits.smk` has no track) and 150 samples, byte for
  byte against the DLL and FFmpeg.
- The DLL's timer service (0x10007D50) copies the ring to Miles in whole words (0x10007DC1: a
  last odd half word stays behind) and flushes the tail only after a frame without a track packet
  (`[snd+0x4C]` set at 0x10005EBA): the samples end on a frame without sound and carry whole words.
- The volume (corrected at M2 T5b1, §11.4: that is the Miles path the stand-in takes; the game's
  is DirectSound's): the game's `SmackVolumePan` with 3640 × the effects setting (0x598CBE); the DLL
  takes it to Miles' scale (0x10008110: `(min(v, 0x8000) × 1270 / 0x8000 + 5) / 10`), 0, 14, 28,
  42, 56, 71, 85, 99, 113 and 127 for the settings 0 to 9 (`h3_view::clip_sound::volume`).

### 8.3 Bink's sound

- The block decoder (0x1000F580) as `h3_bink::audio`'s header lists it: FFmpeg's revision-`b` RDFT
  format, the DLL's arithmetic. The DLL knows that coding alone (the third argument always 1).
- **The FFT** is Takuya Ooura's `fft8g.c` (`cdft` with the sign −1) compiled for binary32 arrays
  on the x87 at 53 bits (CLT-Q120: ported with Ooura's credit, decision 26). A scratch symbolic run
  of the DLL's instructions (0x1000C610 to 0x1000E5F0), aligned with Ooura's C statement by
  statement on random integer inputs, gave where each value is stored to memory (rounded to
  binary32) or kept on the stack (53 bits): the same in every iteration, the transforms of 1024,
  2048 and 4096 values; `h3_bink::fft` was written from it (its generated functions keep the
  DLL's order of operations).
- **The twiddles**: `fcos` of the 53-bit angle, `fsin` of its binary32 rounding; every value the
  correctly rounded binary32 (checked in arbitrary precision, `mpmath`, and against the DLL's
  tables), computed in 124-bit fixed point (`h3_bink::trig`; the scales' `pow10` too).
- **8-bit tracks** are decoded as 16-bit and handed to Miles as their high bytes plus 0x80
  (0x10005430); the client widens them back with the low byte 0.
- Checked: the shell's four Bink clips (`NWCLOGO`, `H3INTRO`, `LOSEGAME`, `H3X1intr`), the samples
  at the three frame sizes, the tables and 120 runs of the FFT on random values, byte for byte
  against the DLL. `NWCLOGO`'s 13 s decode in 0.6 s in release.
- **The volume**: the game never sets a Bink clip's (no `BinkSetVolume`, Miles' master volume
  untouched), so the DLL's sample keeps 127 (0x1000FE4D): the start's clips play at full volume
  whatever the setting. The client plays every clip at the effects setting (CLT-Q121, decision
  27), the original's full volume for the comparisons only (`Bugs::Original`).

### 8.4 For T2b and T5

- **The Bink clips' video (T2b):** the check of the sound found `h3-bink`'s video of the shell's
  four Bink clips not the DLL's (the battle's clips and the samples were): at the first
  differences the chroma only, at most 3 levels (`NWCLOGO` frame 7's U plane, `LOSEGAME` frame 6).
  Found and settled at T2b: the decoding was right, the DLL's copy is not (§9).
- **The end of a clip:** the game closes a Smacker clip right after its last `SmackDoFrame`
  (`smacker_tick` 0x598F74), so the sound still queued is cut; T5 reads how much plays and settles
  the client's end (the samples are whole). Read at M2 T5b1 (§11.3): the last frame's sound.
- **The pace:** the client hands the host a clip's whole track (`HostSound::play_clip`, a stream
  of its own); the DLLs' rings and their drops on a late consumer are not modelled. The 3DO logo's
  full-screen path's volume and the game's DirectSound path are read at T5 (M2 T5b1: §11).

## 9. The Bink clips' copy (M2 T2b, 2026-11-14)

The shell's four Bink clips decode as `BINKW32.DLL` decodes them; what differed is the frame
the game gets. Read in the DLL (scratch disassembly with capstone) and checked on every frame of
the fourteen game clips and the samples (`bink-check.sh`, all exact since T2b). Built:
`h3_bink::Target` (its module doc holds the reading), `Decoder`'s change mask, the samples' still
cells (`build::still_cells`); `Reading::Original` (`check::frames_read`, `h3-view video
--original`, `h3_view::clip::Video` under `Bugs::Original`) gives the copied frames.

- **The copy.** The game calls `BinkCopyToBuffer` with the surface type alone (`video_tick`
  0x44DAA0, `video_redraw` 0x44D9E0: `video_surface_type`), never `BINKCOPYALL` (0x4000000), so
  the DLL (0x10005550) copies the whole frame only when every byte of its change mask is set (a
  `strlen` of it against its size, `[h+0xD0]`) and otherwise only the marked 16 × 16 cells
  (`YUV_blit_16bpp_mask` 0x100096B0 for the game's 5-6-5, `YUV_blit_YUY2_mask` 0x10009D90 for the
  probe's YUY2: the same loop, two cells at a time), then the strips right of `width & !15` and
  below `height & !15` whatever the mask (the full copies 0x10007A90, 0x10008330; the right strip
  read only: no clip's width leaves one). The target keeps the rest from the copies before.
- **The mask** (`[h+0xC8]`, `(S/16) · (R/16)` bytes, `S` and `R` the luma plane's stride and rows,
  the picture's size rounded up to 16: `BinkOpen` 0x10004AC9): cleared by `ExpandBink` 0x10003240
  before the planes; `ExpandPlane` 0x10002040 sets a byte at every block of a type 1 to 8 (a skip,
  0, sets none; an unknown type is a skip there): a counter advanced by its last argument (1 for
  the luma, 2 for the chroma) and halved gives the cell across; the row pointer moves on after each
  odd block row by its stride argument over 16. For the luma that is right. The chroma gets half
  the luma's stride, so its row step is half a row of cells (rounded down) after every two block
  rows, where each chroma block row is a row of cells: chroma block (`bx`, `by`) marks cell
  (`by` / 2) · ⌊`S` / 32⌋ + `bx` instead of `by` · `S` / 16 + `bx`.
- **What shows:** a cell whose chroma changed while none of its four luma blocks did reaches the
  game's buffer only when a later frame marks it. In the shell's clips that is a fade's chroma a
  frame or a few late, up to a few levels (`NWCLOGO`'s logo from frame 7, `LOSEGAME` from frame 6,
  `H3INTRO` from 16, `H3X1intr` from 9); the battle's ten clips have no such cell, so their frames
  are the same either way (their captures need nothing new). **CLT-Q122** (the user, 2026-11-14:
  fix, decision 28): the client shows every decoded frame whole; the DLL's copy for the
  comparisons only.
- **The probe** copies into one buffer of zeros for the whole clip (`binkprobe frames`), as the
  game's dialog target holds its last frame; a `Target` starts at zeros too. What the game's
  buffer holds before a clip's first frame (a key frame marks every cell that has a block of a
  type but 0; the samples' first frames have no skip) is the start's (M2 T5a, §10): the client
  copies each frame onto the back buffer as it is (black, or `introrim.pcx`'s middle).
- **The samples** (`build::sample`): after the first frame a quarter of the cells keep their luma
  (skipped whole, drawn first so the other blocks stay as they were) while the chroma changes, so
  every seed's copied frames differ from its decoded ones and `bink-check.sh` checks the DLL's
  marks; the hashes (seeds 1 to 3, and three at 256 × 120 through `bink-dll.py` by hand) are the
  DLL's.

## 10. The start's pictures (M2 T5a, 2026-11-14)

Read in `Heroes3.exe` (0x44DD20, 0x6030E0; the map), `SMACKW32.DLL` (`SmackWait` 0x10007840,
`SmackNextFrame` 0x10006440, `SmackToBufferRect` 0x100072D0: whole at T5a2) and `BINKW32.DLL`
(`BinkGetRects` 0x10006380 with 0x10006650 and 0x100067B0) with capstone, and checked on the
original's frames: `h3shell.py run --once` of the `start` group in this sandbox (as kept: the
skips, `start-bink` 25 min, `start-smacker` 21 min, its second run's form). Built:
`h3_view::shell::start` (`Start` the timing, `Picture` the buffers), `h3_bink::rects` and
`copies`; the replay plays the start; `h3-view shell` draws it (`--video-quality low` for Bink
Video 0); `Videos` reads `shell-videos.vid` after `battle-videos.vid` when `VIDEO.VID` is missing.

- **The screens:** at the start (time 0 of the frozen clock) the back buffer filled black and
  shown (0x4EE776), the cursor hidden; each clip draws into the back buffer and shows what
  changed; between two clips the fade's black (below), the next clip opened at the same reading.
  `H3Intro` (`video_play_framed`): the rim copied onto the back buffer, a tick (its first frame
  decoded into the buffer), the whole screen shown. After the last clip the main menu at the same
  reading.
- **The frames' times:** Smacker's `SmackWait` (no sound track) counts in 10 µs: the first call
  sets the due time to now + the frame time and waits; a call at or past it lets the frame through
  and sets the next due time to the call's time + the frame time, unless the call came between one
  and two frame times late (an unsigned comparison of a negative difference), then to the due time
  + the frame time; `SmackNextFrame` clears the frame's flag. So at 1 ms steps 67 ms a frame
  (3DOLogo's frames at 67 (k + 1) ms: frames 0 to 3 black, the first change at 335 ms), at 10 ms
  steps 70 (`start-smacker`'s 189.59 s). **CLT-Q129 (b)** (the user, 2026-11-16: fix): in the
  client's profile each frame is due a frame time after the last one's due time, whatever the
  step (3DOLogo's last at 5 400 ms at 10 ms steps); the drift the comparisons' (`Bugs::Original`;
  `h3_view::shell::start::smack_wait`). Bink's: frame p once the clock reaches the first frame's
  time + ⌊1000 p / rate⌋ (E17's measure), the first at the clip's first tick (NWCLogo's at the
  logo's black, 5 577 ms; 6 043 shown at 6 050). The ends: `start-bink`'s menu at 180 480 ms,
  `start-smacker`'s at 189 590 (its second kept run; the first run's 10 ms later from H3Intro on).
- **The fade** (0x602C50 with 1, 4, 0 → `screen_fade_out` 0x6030E0; the clips with the table's
  +0xA, 28 to 31, at their last frame, and the full-screen clips when skipped): the back buffer
  copied, then shown at shifts 0, 1 and 2 of each channel, each step 50 ms of the clock after the
  last (`wait_until`, the first reading at or past it; a step that took more than 50 ms ends the
  loop, never on the frozen clock), then the back buffer filled black and shown 50 ms after the
  third: 3DOLogo's last frame at 5 427 ms, the steps at 5 427, 5 477, 5 527, the black and
  NWCLogo's first frame at 5 577. A clip's last frame is decoded but not shown before it (the
  fade's first step shows it). `H3Intro` skipped stops at once (`video_stop_all`, no fade). An
  input during a fade is lost when a clip follows (its loop flushes the input queue first, §2.1);
  during a fade after which none opens (`H3Intro`'s end, a skipped clip's) the queue keeps it and
  the main menu acts on it. **CLT-Q130 (a)** (the user, 2026-11-16: as the original, both
  profiles): the replay and the window give the menu those inputs, in order, as it opens
  (`Start::keeps_input`; *provisional*, not read: the menu's loop takes them as it would live
  ones at its first pass, the moves not kept, the pointer where it is).
- **NWCLogo's Bink copy** (clip 29, 400 × 300): 0x44DD20 leaves it at (0, 0) of the back buffer
  (every other clip centred), and `video_show` blits that corner stretched onto the whole screen
  (DirectDraw's `Blt`, Wine's nearest pixel: each pixel doubled). The fade copies the back buffer,
  so a skip shows the logo unstretched at the corner, fading (its last frame is black: a clip run
  to its end shows nothing of it). **CLT-Q123** (the user, 2026-11-14: fix, decision 32): the
  client fades the screen as shown; the corner the comparisons' (`Bugs::Original`).
- **What a frame shows** (`video_show` 0x5979D0): the clip's changed rectangles, `BinkGetRects`'s
  or `SmackToBufferRect`'s, as one union shown, by an arithmetic that moves the left (top) edge
  to a later rectangle's smaller one without widening the union and compares a later right
  (bottom) edge with the width (height) from the edges as they now are: the rectangles come top to
  bottom, so a lower one further left cuts off the right of those above it, and the screen there
  keeps an older frame until a later union covers it (`H3Intro`'s Bink copy: 43 of its 2 426
  frames, the right half 4 frames late at 21.3 s; its Smacker copy: 23 frames in small boxes).
  **CLT-Q124** (the user, 2026-11-14: fix, decision 32): the client shows the clip's whole
  rectangle each frame; the union the comparisons' (`h3_view::shell::start::shown_union`).
- **`BinkGetRects`** (`h3_bink::rects`; the game's surface type, neither 0x40000000, no
  rectangles, nor 0x80000000, one): the bounding rectangle of the marked cells (the change mask,
  §9) over the luma's stride and rows (the picture rounded up to 16: whole cells, no partial
  strip); then, while there are fewer than 8, the rectangle whose cut saves the most area (`>`
  the best so far, from 0: the first of equals) cut in two: across at half its width or down at
  half its height, each rounded up to 16 and shrunk to its marked cells, the cut that saves more
  (across on a tie), the new half last; its own and the new half's best cuts computed again. The
  output in the order of y · 65 536 + x (the first of equals first), each cut at the picture's
  right and bottom edge. A frame that marks nothing gives none and leaves the handle's first
  rectangle, which the game shows again. (A half with no marked cell would leave the caller's
  buffer as it was; none occurs: every rectangle is a bounding rectangle of whole cells, so both
  halves of a cut hold one of its edges' marked cells.)
- **`SmackToBufferRect`** (0x100072D0; read whole at T5a2 with 0x10007410, 0x10007680, the
  iterator 0x10012760 and the decoder's map 0x10011AB0; built: `h3_smacker::Rects`, the map
  `h3_smacker::Frame::spans`; checked against the DLL on every frame: `smackprobe rects`):
  - *The map* (the decoder state's +0x38): a byte for every 16 blocks in decoding order (64 pixels
    of a row of blocks, running on into the next row where the width is no multiple of 64). The
    block routines' prologues keep two counts from 0: each run adds its whole length to the first
    (uncut at the frame's end); when the second is not past it, a written run sets the spans from
    the second's to the first's rounded up, a skipped run clears them to the first's rounded down,
    the second then the span's start reached; after the last run the span at the second is
    cleared. So a span is 1 when it holds a written block; a last run past the end marks spans
    past it, and spans past the second keep an earlier frame's bytes (0 at the start: the state is
    allocated zeroed by `SmackToBuffer`).
  - *The iterator* (state +0x20 to +0x34, set back by each decode): the next run of changed spans,
    cut at a row's end (its rest then the next run, at x 0 of the next row; a clip narrower than a
    span gives a row a run). The rest looks at the next two spans as one word: when the first is
    unchanged and the second changed, the unchanged one joins the run.
  - *The filler* (stage +0x408: 1 set by `SmackDoFrame`'s decode, 2 filling, 3 done): from the run
    held, the next runs joined while the box over them (from the first's top to the latest's
    bottom) is at most the sum of their areas × 5 / 4 (each rounded down); the run that does not
    fit is held for the next rectangle. The box aligned to 8 pixels of the buffer: the left edge
    down where that stays right of the clip's place, the width up where the clip has more room
    than that. A frame without changed spans: none, or with a new palette an empty rectangle at
    (0, 0) and then the held run (an earlier frame's) again. The buffer's type the game passes
    (0x69FE20: 0xC0000000 or 0x80000000) takes this path (1 would join every run, 2 give none).
  - *The list* (+0x4C8, +0x4CC; at most four as left, top, right, bottom): filled from the filler
    until four or none more, each new one joined to the first that shares a whole edge with it,
    again and again by the joined one's index (which, after a removal moved it down, names the
    one after it); the first taken into +0x380 to +0x38C, first in first out.
  - *The game's reads* (`video_show`): the last rectangle after the first call whatever it
    returned (a frame with none shows the last frame's last one again), then one per call until 0;
    their union by the same arithmetic as Bink's (`shown_union`), already in the back buffer
    (`SmackToBuffer` was given the clip's place 0x69FE00, 0x69FE04).
  - *Found*, all within CLT-Q124's fix (decision 32: the client shows each frame's whole
    rectangle; the DLL's for the comparisons): the word read, the union from (0, 0) after a
    palette-only frame, the box's bottom the latest run's. A third join of one rectangle can make
    the DLL's copy run backwards and fault (no clip of the game reaches it: the filler gives the
    runs top to bottom); `Rects` stops the join there. Not ported: the types 1 and 2 and the
    interlaced and doubled clips (the header's flags 2 and 4), which the game does not use.
  - *Checked* (`clients/scripts/smacker-check.sh`): every call of every frame of the shell's eight
    clips, AB's `Credits.smk` and the eight synthetic samples (36 to 60 wide: their spans
    run on across the rows), the clip at (0, 0) and at (83, 187), as the DLL's; `H3INTRO.SMK` meets the palette-only frame twice and frames
    without changes 97 times.
- **The comparisons** (`h3shell.py drawn --group start`): all 12 lines exact since T5a2
  (`start-smacker`'s clip line differed in 23 of `H3INTRO.SMK`'s frames until SMACKW32's
  rectangles were built; 30 lines with T5b2's sound recipes, §12.3). The Bink decoder of a
  comparison reads as the DLL (`Reading::Original`, CLT-Q68: without it two pixels of NWCLogo
  differ from frame 19 on); each frame is copied onto the back buffer as the DLL copies it
  (`h3_bink::copies`; a Smacker frame's written blocks), so the cells a frame leaves keep what
  the buffer held (the black, the rim).

## 11. The start's sound (M2 T5b1, 2026-11-15)

Read in `Heroes3.exe` (`smacker_open_in_dialog` 0x598AF0, `video_open_in_dialog` 0x44D830,
`sound_digital_start` 0x5997D0, `smacker_tick` 0x598EB0, `game_init` 0x4ED650), `SMACKW32.DLL`
(`SmackSoundUseMSS` 0x100079F0, `SmackSoundUseDirectSound` 0x100095C0 and the functions it
installs, `SmackOpen`'s tracks 0x10005710, `SmackDoFrame`'s sound 0x10005DD0, `SmackWait`
0x10007840, `SmackClose` 0x10005B20) and `BINKW32.DLL` (`BinkWait` 0x10005EE0, `BinkPause`,
`BinkOpenMiles` 0x1000F970) with capstone; checked on the DLL by `smackprobe wait` on a stand-in
DirectSound (`clients/capture/vsound.h`, §11.5). Built: `h3_view::shell::start::SmackSound`,
`Op::Sound`, `Op::Quiet`, `Op::Closed`, `Start::with_sound`; `h3-view shell --effects`, the
window's sound. The cut at the game's run (decision 33): the start recorded with `sound = true`
waits for the same device in `xdd.dll` (T5b2).

### 11.1 The path

- **When a clip has sound:** `smacker_open_in_dialog` sets 0x69FE58 to 1 when the sound is not
  off (0x699290), the sound manager has a digital driver (+0x3C) and the effects setting 0x698764
  is not 0, and then opens the clip with all its tracks (0xFE000 with `SmackOpen`'s 0x1140); the
  3DO logo takes this path too (`video_play_fullscreen` 0x5972D0 opens its Smacker clips through
  it), so it plays at the effects setting like the others. Every clip of the shell but
  `ProgressBar` and the `CREDITS` placeholders has a track (22 050 Hz, 16-bit, stereo, packed).
  `video_open_in_dialog` sets 0x694D58 the same way and never reads it: a Bink clip opens its
  sound whenever Bink has a sound system.
- **The DLLs' sound:** `sound_digital_start` 0x5997D0, once the driver is open, calls
  `SmackSoundUseMSS(driver)` and `BinkSetSoundSystem(BinkOpenMiles, driver)`. `SMACKW32` asks
  Miles' preference 15 (`DIG_USE_WAVEOUT`): the game's Miles answers 0, so it calls
  `SmackSoundUseDirectSound` with the driver's +0xA0 (the DirectSound object Miles opened) and
  plays each track through a DirectSound buffer of its own (the probes' stand-in Miles answers 1,
  the double buffers of §8.1). `BINKW32` plays through Miles' double-buffered samples (the
  functions `BinkOpenMiles` takes from `MSS32.DLL` by name: `AIL_load_sample_buffer`,
  `AIL_sample_buffer_ready`, …).
- **The track's buffer** (`SmackOpen` 0x10005710, the DirectSound open 0x10009750): the track's
  bytes a second `rate × (1 + 16-bit) × (1 + stereo)` (+0x14: 88 200), a frame's bytes × 1024
  `(rate << 10) × frame time / 100 000` (+0x1C; the frame time in 10 µs), a ring of
  `((+0x1C >> 9) + rate + 0xFFF) & !0xFFF` bytes (at least 0x2000), and a looping DirectSound
  buffer of half a second rounded up to 4 KiB (+0x44: 45 056 bytes), its flags 0x18000 with the
  volume's (and the pan's) control; a sound timer of 75 ms (`timeSetEvent`, or Miles' timer at
  13 a second when Miles' functions are loaded).

### 11.2 The frames with sound (`SmackWait` 0x10007840)

- **The service** (0x10008F50; from the timer, `SmackWait`, the position, `SmackDoFrame` before
  the frame's sound and `SmackNextFrame` twice): it runs at a call 10 ms or more after its last
  (a global 0x100148E4, unsigned) and for each track reads the buffer's play cursor (the bytes
  played since its last run go to the free space), tops the buffer up from the ring when a quarter
  or more is free (pads silence and stops the buffer at an end mark when the ring runs dry), and
  starts a stopped buffer that has data (`Play`, looping). The frame 0's service runs before its
  sound is in the ring, so the buffer starts at the first call 10 ms or more after the clip's
  open.
- **The position** (0x100096A0): the bytes written less those not yet played (the cursor's total
  as the service last read it); when that equals the last value seen (+0x84) and is not 0, that
  value plus the clock's milliseconds since it was first seen (+0x88) × the bytes a second / 1000,
  at most the bytes written; never less than its last answer (+0x8C).
- **The wait:** `SmackDoFrame` sets the handle's +0x448 and adds a frame's bytes × 1024 to the
  sound's +0x20 (its overflow carried into +0x24 in steps of 256 000 bytes, the sum unchanged).
  `SmackWait` with the flag set lets the frame through once `(+0x20 >> 10) + +0x24 ≤ position + 8`,
  clearing it; without it at once. So frame 0 goes at the open (without sound 67 ms later) and
  frame k once k frames' sound less 8 bytes has played: `3DOLOGO.SMK`'s frames at 0, 77, 144,
  210, 277 … ms (the buffer's Play at 10), its close at 5 343 ms at 1 ms steps (5 350 at 10 ms;
  without sound 5 427 and 5 670); `H3INTRO.SMK`'s in 161.66 s (without sound 169.8 s at 10 ms
  steps). The silent path's 67 or 70 ms a frame would put the picture behind its sound by up to
  8 s at H3Intro's end; with the sound the frames keep its time exactly. The client's silent path
  keeps the frame time too (CLT-Q129 (b), §10: `H3INTRO.SMK` about 161.7 s at any step), the
  DLL's drift the comparisons'.
- **Not modelled** (`SmackSound`, *provisional*): the ring and its refills, the end mark and the
  silence padding, the reset of the frames' sound to the ring's bytes when it runs ahead
  (0x10006131): no clip of the game reaches them (the first frame's sound fills the buffer, and
  each frame brings its own); the synthetic samples' few hundred bytes a frame do, so their wait
  lines are not compared.

### 11.3 The end and a skip

`smacker_tick` closes the clip right after its last `SmackDoFrame` (before the fade), and
`SmackClose` stops and releases the buffer at once (0x10009CA0: `Stop`, `Release`); a skip closes
it the same way. So the sound plays from the buffer's Play to the close: the track's first bytes,
as decoded (`smackprobe wait`'s played file is the track's first so many bytes at every step), and
the last frame's never: 66 ms at the start's clips (silence there), 100 ms at `HSANIM` and
`LOSEGAME`, audible. **CLT-Q125** (the user, 2026-11-15: fix, decision 34): the client plays the
track out after its clip's last frame (`Op::Closed`), the cut the comparisons' (the replay's
`closed` lines); a skip stops it (`Op::Quiet`). A Bink clip: `BinkClose` with the clip's close
(the full-screen path pauses and closes both handles), its frames timed by `BinkWait` alone (the
clock, from the first frame's time, and `BinkPause`'s flag +0xE4: the sound never moves them);
its sound starts with Miles' first buffer (the client's at the first frame: read at T5b2, §12.2).

### 11.4 The volume

`SmackVolumePan(h, 0xFE000, 3640 × setting, 0x8000)` → 0x10009DF0: `SetVolume` with entry
`min(v, 0x8000) × 127 >> 15` of a table of `2000 log10(i / 127)` hundredths of a decibel
(0x100148E8, 128 words; 0 is −10 000), `SetPan` with entry 2 × 63 of it (−7: the middle). So the
gain is `i / 127`: 0, 14, 28, 42, 56, 70, 84, 98, 112, 126 for the settings 0 to 9
(`h3_view::clip_sound::volume`; T2's 71 to 127 from the setting 5 were the Miles path's, §8.2).
The probe's volume lines are the table's entries (−517 at the setting 5, −7 at 9).

### 11.5 The check

`smackprobe wait <vid> <entry> <step> <volume> <out>` (`clients/capture/README.md`, "The clips'
sound clock"): `SmackSoundUseDirectSound` with a stand-in DirectSound (`vsound.h`: a buffer's
cursor at its Play's place plus the clock's milliseconds × the bytes a second / 1000, whole
samples, around the buffer; the write cursor the play cursor: an ideal device), `SMACKW32`'s
`timeGetTime` pointed at the probe's clock and its `timeSetEvent` made never to fire (the
service runs from the game's calls alone, as on the harness's frozen clock), the clip opened and
set as the game sets it, then `smacker_tick` at every reading of a clock stepped by `<step>` ms
(again at one reading until `SmackWait` holds a frame back). `smacker-check.sh` runs it at 1, 5,
7, 10 and 16 ms (`smacker-dll.py`: `<stem>.wait<step>`, the volume as the table's index, the Play,
each frame, the close with the bytes played, which must be the track's first) against
`h3-view video --smacker`'s model (`SmackSound`): every line of `3DOLOGO`, `NWCLOGO`, `H3INTRO`,
`HSANIM`, `HSLOOP` and `LOSEGAME` (30 files, `H3INTRO`'s 2 426 frames each) as the DLL's at the
first run of the model. The replay with sound (`h3-view shell --steps … --effects 5`) lists
`sound`, `quiet` and `closed` with their times: the Smacker start's main menu at 180.42 s at 10 ms
steps (without sound 189.59 s).

### 11.6 `game_init` (0x4ED650, named `game_samples_load` until T5b1)

The game's initialisation: the managers, the settings, the data folder and its archives, every
text table and the game's other loads, the video archives, the CD status, `button.wav` (its one
sample), the screen asked for. It plays nothing and starts no driver: the clips' sound depends on
the driver `sound_digital_start` gives the DLLs when `game_main` starts the sound manager, and on
the effects setting.

### 11.7 For T5b2

Done at T5b2: §12.

## 12. The start's sound on the game's run (M2 T5b2, 2026-11-15)

Read in `SMACKW32.DLL` (`SmackSoundUseMSS` 0x100079F0 whole, the DirectSound open's timer
0x10009864) and `BINKW32.DLL` (`BinkSetSoundSystem` 0x10004960, `BinkOpenMiles` 0x1000F970, its
sound system 0x1000FB80 to 0x1000FE57, the sound service 0x10005270, its callers, `BinkDoFrame`'s
audio 0x10005A90, Bink's thread 0x100054D0) with capstone; built `xdd.dll`'s 7. (the runbook's
"The clips' sound on the game's run"), `binkprobe start`, the stand-in Miles' `MssMinimum`,
`MssHold`, `MssLoads`; `h3shell.py`'s voices, the replay's volume and `closed` lines.

### 12.1 Smacker

- **The game's path:** `SmackSoundUseMSS(driver)` keeps the driver (0x10015E40), loads
  `MSS32.DLL` and 24 of its functions by name at its first call (0x10007A5A on, among them
  `AIL_lock`, `AIL_unlock` and the timer's four), asks preference 15 and, on 0 (the game's Miles),
  calls `SmackSoundUseDirectSound` with the driver's +0xA0; a later call asks again. The
  DirectSound open (0x10009750) of the first track makes the sound timer: with Miles' `AIL_lock`
  loaded (0x100148D4) Miles' (`AIL_register_timer` of `_ailcallback`, 13 a second, started),
  else `timeSetEvent(75, 25, TimerFunc, 0, periodic)`; `AIL_lock` and `AIL_unlock` (when loaded)
  bracket its DirectSound calls.
- **On the harness:** `xdd.dll` points the game's import at a hook that calls
  `SmackSoundUseDirectSound(&vsound)`, so the DLL never loads Miles' functions: its timer is
  `timeSetEvent`'s, which never fires (its import a stub), and no lock brackets the calls (the
  game's main thread alone makes them then). The service, position and wait run from the game's
  calls on the virtual clock, as in `smackprobe wait`; the cursor is the ideal device's (§11.5).
  The game's run: `use`, the buffer (45 056 bytes, 88 200 a second, flags 0x18080), the timer
  (75 ms), pan −7, volume −517 (the table's 70 at the setting 5), Play 10 ms after the open.
- **Recorded** (two runs each the same; `drawn` exact): `start-smacker-sound`, the three Smacker
  clips timed by their sound, the main menu at 180 420 ms (189 590 silent), 2 586 frames
  exact; the voices 3DOLOGO 10 to 5 350 ms (the close at 10 ms steps), NWCLOGO.SMK 5 510 to
  18 450, H3INTRO.SMK 18 610 to 180 270, each at 70; `start-skip-key-sound`, Space at 2 000 ms:
  the logo's sound from 10 ms to the skip (its `before` shot not the silent run's: its frames
  follow the sound).

### 12.2 Bink

- **The sound system:** `sound_digital_start`'s `BinkSetSoundSystem(BinkOpenMiles, driver)`
  calls `BinkOpenMiles(driver)` at once (0x1000498F), which loads `MSS32.DLL` and 20 functions by
  name (`GetProcAddress`; skipped once loaded, 0x10026B0C) and returns its open (0x1000FB80). A
  clip with a track opens its sound in `BinkOpen`: a sample allocated, its two buffers of
  `AIL_minimum_sample_buffer_size(driver, rate, format)` rounded up to 16 bytes each (+0x48;
  doubled for an 8-bit track, +0x24), the sample initialised, its type (16-bit stereo: 3), rate
  and volume 127 (0x1000FDF0). The system's calls: ready (`AIL_sample_buffer_ready`, kept until
  loaded), lock (the buffer and its size), unlock (`AIL_load_sample_buffer`), volume (`v × 127 /
  0x7FFF`), pause (`AIL_stop_sample`, `AIL_resume_sample`), close (the buffers freed,
  `AIL_release_sample_handle`). BINKW32 never calls `AIL_start_sample`: Miles starts a
  double-buffered sample with its first load (Miles' double buffering as documented, *not read*
  in `MSS32.DLL`); nor `AIL_sample_status`, `AIL_serve`,
  `AIL_end_sample`, `AIL_set_sample_pan`, `AIL_set_sample_user_data` or `AIL_get_preference`,
  which it loads.
- **The service** (0x10005270): with its lock free (+0x254) and the ring holding more than a
  buffer (+0x250 > +0x280, the system's +0x24) on entry, while Miles gives a buffer: the ring's
  bytes (at most a buffer, the last load of a run partial; an 8-bit track converted,
  0x10005430) copied into it and loaded, the threshold not tested again until the next entry. It runs in
  `BinkDoFrame` before and after the frame's audio is decoded into the ring (0x10005A03,
  0x10005BE2), in `BinkCopyToBuffer`, `BinkWait` and others, and on Bink's own thread
  (0x100054D0: every 3 ms its read-ahead, the service in 8 rounds of 16). Only `BinkDoFrame`
  fills the ring (each block decoded, at most its declared size and the ring's room), so the
  first load comes in the `BinkDoFrame` of the first frame whose sound takes the ring past a
  buffer, whichever thread runs it: on any clock, at that frame's time.
- **The game's clips:** the four with a track (`NWCLOGO`, `H3INTRO`, `LOSEGAME`, AB's
  `H3X1intr`; the battle's have none) carry 67 200 bytes of sound in their first frame (0.76 s;
  the ring's 16-bit samples, its capacity 67 328), then about 5 760 a frame (`binkprobe start`:
  the ring after the first `BinkDoFrame` 67 200 less the loads; `NWCLOGO` 1 146 596 bytes in
  all, `binkprobe sound`). The game's Miles under the harness
  answers 36 864 bytes for a 16-bit stereo track (0.42 s; `bink-minimum 36864 22050`), so the
  first buffer loads in the first frame: `h3_view::shell::start`'s rule (the client's sound from
  the first frame, no longer *provisional*), checked by `bink-check.sh` for each clip with a
  track (`<stem>.start`). `H3X1intr`'s track is 8-bit stereo (flags 0xA000, not the others'
  0xE000): Bink doubles its threshold (the ring's samples are 16-bit), so at a 16-bit track's
  36 864 bytes it would need 73 728, more than the ring holds, and never load (silent); the
  check gives the probe the same 0.42 s in the track's own bytes, 18 432 (*provisional*: no run
  of the game showed Miles' answer for an 8-bit track; AB's intro plays outside the start). The
  game's Miles on another machine may give other sizes (*provisional*: the harness's alone is
  read); any under 67 200 bytes for 16-bit stereo gives the same start.
- **On the harness:** `xdd.dll` answers the 11 functions BINKW32 calls (by its `GetProcAddress`
  import): the real Miles allocates, sets up and releases the sample and gives the size, the
  buffers play on the virtual clock from the first load, one after the other at the sample's
  bytes a second (88 200), a pause holding them; Miles never gets a buffer, so the null sink
  plays nothing. The ring's later loads follow (a buffer every 0.42 s), compared nowhere.
- **Recorded:** `start-skip-left-sound` (two runs the same; `drawn` exact): the logo's sound 10
  to 5 350 ms, NWCLOGO.BIK's from 5 500 (its first frame) to the press at 12 000 (the pause, then
  the release at the same reading); `start-bink-sound`: §12.3.

### 12.3 The comparisons

`h3shell.py` keeps each clip's sound as `voice<k> <smacker|bink> t<start>-<end> v<volume>`
(`clip_voices`; the volume the index of SMACKW32's table, or Miles' 0 to 127); `drawn` builds the
same lines from the replay's `sound <file> t<ms> v<volume>`, `quiet` and `closed` (the volume
`clip_sound::volume` under `Bugs::Original`: 70 for Smacker at 5, 127 for Bink, CLT-Q121) and
compares them as it compares shots. The original's voice ends where its clip closes, by a skip
or after its last frame: the client's `Op::Closed` is the comparisons' end (CLT-Q125's cut).
`start-bink-sound` (two runs the same): the 3DO logo timed by its sound (69 frames, its close at
5 343 ms at 1 ms steps), NWCLOGO.BIK's sound from its first frame at 5 493 to its close at
18 430, H3INTRO.BIK's from 18 580 to 180 250, the main menu at 180 400 (180 480 silent). `drawn
--group start`: the group's 10 recipes, 30 lines (shots, clips, voices) exact, the four sound
recipes each at its first recording.

### 12.4 The end of a Bink clip's sound, and what is not modelled

- **The tail:** the service enters only while the ring holds more than a buffer, so a clip's
  last bytes (a buffer or less) may never reach Miles: `NWCLOGO`'s 31 loads (the second a
  partial 30 336 at the first frame) hold 1 136 256 of its 1 146 596 bytes, its last 0.12 s
  never played; on the harness the loaded ones run out 55 ms before its close (`bink-dry` at
  its release; `H3INTRO`'s too). The original's sound then ends with its
  clip's close either way; the client plays the track out (decision 34, CLT-Q125's fix: a clip's
  track, either format), the cut the comparisons' (`closed`).
- **Not modelled:** the harness's Bink sample plays its buffers back to back with no device
  latency, an underrun's next load starting a new run at its own time; the real Miles' buffer
  size on another machine; Smacker's ring and underrun as §11.2. None changes a frame or a voice
  of the recipes.

## 13. The main menu taking its inputs (M2 T6a, 2026-11-15)

Read in `Heroes3.exe` (`main_menu_proc` 0x4FB710 whole, `def_button_message`'s loop 0x456328 to
0x4564D3, `button_press` 0x456860) and taken from the recordings: the harness's `menu-main`,
`menu-quit` and `menu-sound` played again in the sandbox (every kept line as kept), their frames
compared pixel by pixel with the viewer's. Built: `h3_view::shell::menu`, `::keys`, `::samples`
(the epic's T6a row).

### 13.1 The procedure

- **Each message** (0x4FB910): the tick 0x4ED490; with the window manager's +0x48, 0x602C50 (0,
  4, 0). Then by type:
- **A right press on a button** (+0xC bit 0x200, subtype 0xC or 0xE, id 101 to 105; not in a
  lobby's start): `message_box` of style 4 with `main_menu_help`'s right-click text, entry id −
  101 (0x6A6C24 + 8 k: `Help.txt` records 3 to 7, column 1). The DEF button turns a right press
  into 0x200/0xE and does not go down (0x4567AD): the popup shows over the button as hovered
  (`menu-main`'s `*-popup`). `popup_run_held` hides the cursor and drops every message until the
  right release, a left press or a left release.
- **A command** (0x200/0xD, 101 to 105): 101 to 104 set the window manager's +0x38 to the id and
  end; 105 pauses the videos (0x5977A0), asks GENRLTXT 70 in a box of style 2 (not in a lobby's
  start), resumes them (0x597850) and ends only on 0x7805 (OK).
- **A move** (type 4): `dialog_item_id_at` against `main_menu_hovered`; on a change the five lose
  state 0x10 (`dialog_item_send` subtype 6), the new one gains it (subtype 5).
- **Then** (0x4FBACD): on a change, or a clip's frame due (0x597930), the items 101 to 105 redrawn
  in order (vtable +0x14), (520, 4, 250, 567) shown (all five inside it), `video_show`.
- **The end** (0x4FBB0F): the message becomes 0x200/10/10 and the procedure answers 2; a lobby's
  start ends it with 101 after 10 s (0x4FBB2C, `> 0x2710`).

### 13.2 The press and the hold

- `button_press` (0x456860): the state's bit 1; `button.wav` (0x694DA4) at volume 0x40, played
  once (+0x30 1), in category 3 (+0x28: the creature sounds' handles 6 to 13) through
  `play_sample`, the sound manager's +0x84 1 meanwhile (*not read*); the button drawn (vtable
  +0x10) and its box shown at once; the message 0x200/0xC.
- The loop (0x456338): every message first to the mouse manager (vtable +8: the cursor); a move
  inside a button not pressed presses it again (its sample again), outside a pressed one releases
  it (drawn, shown); the left or the right release (0x10, 0x40) ends it: pressed, the button
  drawn up and its command 0x200/0xD returned; released off, the release returned as it is,
  which the procedure ignores.
- **Recorded, not read:** after a hold ends the menu shows the hover taken at the pointer
  (`*-left`: all five in frame 0, the pointer off them), and after the question closes too
  (`cancelled`, `escaped`: Quit back in frame 0 with the pointer on the box's Cancel). The
  procedure gets no move from either path read; the client takes a move at the pointer there.
- **The key path** (0x45659F, 0x4564D6): the hotkey's key down presses the button (its sample),
  its key up clicks: N, L, H, C and Escape for 101 to 105 (`menu-quit`'s `question-key`).

### 13.3 What the recordings add (the comparisons' rules)

- **The cursor at an even x** (E17's 0x50CE04, design §9.9 there): `menu-main`'s hover and press
  shots and every `menu-quit` shot point at odd x (643, 359, 441): the original draws the arrow a
  pixel left. The shell's frame rounds the x down under `Bugs::Original` only, as the battle's.
- **A button redrawn alone** (CLT-Q126, decision 36): Load Game's DEF canvases reach rows 251 and
  252, High Scores' first two; the press draws Load Game alone over them (`load-press`: 2 pixels),
  until a hover change redraws the five in order. The client draws them in order always; the
  comparisons draw the button redrawn alone last.
- **The boxes** (`menu-quit`, `*-popup`): framed in red (player 0, the viewing player at the main
  menu), centred, the help's text at x 20 as the executable sets it (CLT-Q28 (1): the client's
  centred); the question's box draws the cursor (the arrow), the popup none. Every one exact at
  the first run.
- **The sample** (`menu-sound`'s `sound1`): New Game's press at its click's time, 151 ms
  (`button.wav`'s frames at its rate, rounded up), the harness's channel 0. The harness numbers
  the handles in the order it first sees them; `play_sample` takes category 3's first handle whose
  sample is done, else the next in turn (*inferred*: its start and step), so `menu-sound`'s
  `sound2` (Back's press 100 ms later, the first still playing) is channel 1: T6b's.
- **The music** (`main_menu_run`: "MainMenu", loop count 0): a request for the track playing is
  dropped (`play_music` 0x59AD43) before the effects' stop (0x59ADAC), so the presses' samples are
  never cut by a menu's opening after the first. The harness logs no music: the window plays it,
  the comparisons have none.

### 13.4 Not built (the agent's calls, *provisional*)

The first pass's disk check (free space below 5 MB: GENRLTXT 708 and the end; the client has no
such limit), the CD notice (GENRLTXT 730, 107 with 763, 746 or 747: not on Complete), a DirectPlay
lobby's start (§2.10), the 0x602C50 call with the window manager's +0x48 (*not read*), the
videos' pause around the question (clip 33 draws nothing, CLT-Q86).

## 14. New Game's and Load Game's menus taking their inputs (M2 T6b1, 2026-11-15)

Read in `Heroes3.exe` (`game_type_menu_proc` 0x4D5940 whole again, `game_type_menu_create`
0x4D54C0's items) and taken from the recordings: the harness's `menu-new`, `menu-load` and
`menu-sound` played again in the sandbox (every kept line as kept), their frames compared pixel by
pixel with the viewer's. Built on §13's machinery: `h3_view::shell::menu::Menu` for both kinds
(the epic's T6b1 row).

### 14.1 The procedure, against the main menu's

- **Each message:** the tick 0x4ED490 only; no 0x602C50 call (the main menu's, §13.1).
- **A right press** (+0xC bit 0x200, subtype 0xC or 0xE) on 100 to 104: `message_box` of style 4
  with `game_type_menu_help`'s right-click text of id − 100 (0x6A6BFC + 8 k: `Help.txt` records
  10 to 14); Load Game's the same records. The title (105) has none.
- **A command** (0x200/0xD, 100 to 104): the window manager's +0x38 set to the id and the end,
  Back among them; no question.
- **A move** (type 4): `dialog_item_id_at` against `game_type_menu_hovered`, so the title picture
  is an item under the mouse too: on a change 100 to 104 lose state 0x10 (with the CD notice only
  102 and 104) and the new item gains it (105 included; a picture draws nothing for it).
- **Then** (0x4D5AB6): on a change, or a clip's frame due, items 100 to 105 redrawn through the
  dialog's vtable +0x14 in the order the dialog holds them (title, Single, Campaign, Tutorial,
  Multiplayer, Back: `game_type_menu_create`'s order), (526, 7, 250, 560) and the title's (114,
  312, 300, 48) shown, `video_show`. The end as the main menu's.
- **The title** is a 16-bit picture (`H3DlgPcx16` 0x450690, flags 0x800): `newgame.pcx` and
  `loadgame.pcx` are 24-bit, drawn opaque.

### 14.2 The rows the redraw does not show (CLT-Q118's, the comparisons' rule)

The screen keeps what was last shown there. A DEF button's own loop (§13.2: the press, a hold's
move off or on, the release) draws the button alone and shows its box, the item's rectangle,
whole; the hover's redraw shows only the rectangles above. Single's box (545, 4, 209, 123) reaches
rows 4 to 6 above (526, 7, 250, 560), Back's (582, 464, 127, 107) rows 567 to 570 below it, so
those rows show the frame the button's loop last showed whole, or the opening's frame 0 until it
ran once. Recorded: `single-hover` (Single in frame 3, its rows 4 to 6 still frame 0),
`single-press` (frame 1, the rows alike: frame 1's top rows are frame 0's), `single-left` (the
release off it drew it up hovered, frame 3, and showed it; then the hover's change redrew it in
frame 0 inside the rectangle only: rows 4 to 6 stay frame 3 from then on, 66 pixels), Back's the
same at rows 567 to 569 (105 pixels). The client draws every button whole (decision 22); the
original's profile keeps, for each button whose box reaches outside the rectangles, its state at
its last whole show and draws those rows from it (`Menu::stale`, the adapter's copy over the
background; *provisional*: no other item reaches the rows here). The main menu's five lie inside
its rectangle (520, 4, 250, 567): none stale there.

### 14.3 What the recordings add

- **A menu opens with none hovered** (`menu-new`'s `menu`, `menu-sound`'s `new`): the pointer left
  over Single by New Game's click, Single in frame 0 until the first move, as the main menu
  (§13.2).
- **The sample** (`menu-sound`'s `sound2`): Back's press 100 ms after New Game's, the first
  sample still playing: the harness's channel 1, 151 ms (§13.3's rule).
- **The boxes:** the help popups framed in red and centred as the main menu's; every one exact at
  the first run.
- **Back** (`menu-new`'s `back`, `menu-sound`'s `back`) and Load Game's Escape (`menu-load`'s
  `back`): the main menu again, opened anew (none hovered), the pointer where the click left it.

### 14.4 Not built (the agent's calls, *provisional*)

The CD notice's menu (Single, Campaign and Tutorial not created, only 102 and 104's hover
cleared: not on Complete), the clip's due frames' redraws (clip 33 draws nothing, CLT-Q86), the
tutorial flag's clearing at the creation (no tutorial mode, CLT-Q84). New Game's Campaign and
Tutorial and the main menu's High Scores and Credits lead to T6b2's stubs (decision 37; §15).

## 15. The stubs and the client's own pack (M2 T6b2, 2026-11-16)

Nothing read: the stubs are the client's own (decision 37); the menus end with the buttons'
commands and the loop's switches take them (§2.2, §2.4) as T3a and T6b1 built them.

### 15.1 The stubs

- **Which:** New Game's Campaign (the campaigns' selection, E20: decision 7) and Tutorial (the
  engine's tutorial mode, CLT-Q84: decision 9), the main menu's High Scores (§2.8) and Credits
  (§2.9) until M4. Load Game's Campaign and Tutorial lead to the scenario screen's load (M3; its
  stub since T7, §16.3), not to a stub of their own.
- **What:** the menu ends with the button's command as the original's; the screen on is then the
  stub's (`Screen::built`, `Built::Stub`): the menu it was chosen from opened anew (none hovered,
  the pointer where it is) with a box of style 1 (an OK alone) over it, centred and framed in
  red as the menus' boxes, its text the client's own pack's `shell.stub.<screen>` (the driver's
  names: `campaigns`, `tutorial`, `high-scores`, `credits`). The box takes the inputs as Quit's
  question does (a press, a hold's move off and on, the release over the OK, the right release
  ending its hold, Enter; the menu's buttons, hotkeys and help popups none under it); its OK ends
  the screen (`Ended::Back`: New Game's menu; `Ended::Done`: the main menu), which opens anew.
  The window's music goes on (the menus' track).
- **The replay:** a stub is not the original's screen (it shows the campaigns' selection, the
  tutorial's load, its High Scores, its credits there), so no menu opens on it and the replay
  stops at its first shot, clip or input but a move, as at a screen not built (`stop <k>
  not-built <screen> (<task>)`): `menu-tutorial` stops at its `clip:load` as before. No kept line
  changes.
- **The words** (the agent's, *provisional*): a title in the highlight braces, as the help
  popups' (TextDraw's markup, E17 design §3.6), and one sentence: "{Campaign}" and "The campaigns
  are not available yet.", and so for the tutorial, the high scores and the credits.

### 15.2 The client's own pack

- **The file:** `clients/text/en.json`, beside the key catalogue (`h3_battle_client::text::
  OWN_PACK`): the messages of the keys without `original` forms (`shell.stub.*` so far), English,
  nested JSON with i18next's conventions (E17 design §9.2). Its language is its name, as a
  player's pack's.
- **Its check** (`text::own_pack`): an entry no key names, a placeholder its key lacks, or a key
  of the client's own it gives no last form of (`h3_ui::text::pack::Pack::lacks`: `<key>`, or a
  plural key's `<key>_other`, where a lookup ends within a pack) is an error: a client does not
  start with a fault in its own pack, as with the original's text, and the gate lists it.
- **The order:** the player's pack (only `h3-view text --pack` takes one so far), the client's
  own, the original's: `h3_battle_client::log::catalogue` (every client's catalogue: the shell's,
  the battle's windows and renders) and `h3-view text`.
- **Not built** (*provisional*): a pack chosen by language (only `en.json`; a localized
  installation's own text keeps its language, the client's strings stay English), a player's pack
  in the windows.

## 16. The Multiplayer dialog and the hot seat's names taking their inputs (M2 T7, 2026-11-16)

Read from `Heroes3.exe` (the edit's code whole, the dialogs' items' arguments again) and recorded
again (`multiplayer`, two recipes added: `edit-keys`, `hotseat-long`). Built in
`h3_view::shell::multiplayer` and `h3_ui::edit`.

### 16.1 The edit (`H3DlgEdit`: `Create` 0x5BA920, vtable 0x642D40)

- **Create** (sixteen arguments): a text item first (`text_item_create` 0x5BC280), then the room
  with the terminator (+0x5A, 21 in both dialogs), the caret (+0x58) at the text's end, the first
  shown character (+0x6A) 0, the focus (+0x6D) clear; the text box (+0x5C to +0x62) the item's own
  box, inset by the border's two arguments only with a border of 4 (none here). Both dialogs' edits:
  `smalfont.fnt`, colour 4, left, no picture.
- **The vtable:** +0x08 the item's procedure 0x5BB150 (a key to +0x3C while shown and not
  disabled; a press inside focuses it, `dialog_set_focus` 0x5FF490, as 0x200/0xC; the 0x200
  messages 3 text, 7 its pointer, 51 the room, 64 the focus by id); +0x10 `edit_draw` 0x5BB400;
  +0x28 and +0x2C focus and unfocus (+0x38 0x5BAB50 with 1 or 0; the hot seat's +0x2C is
  `hot_seat_edit_check`); +0x3C `edit_key` 0x5BAC50 (the Multiplayer name's
  `multiplayer_name_key` 0x50ED60, the hot seat's `hot_seat_edit_key` 0x50DF60); +0x40 0x5BBA20
  refuses scan codes 1 (Escape), 0x0F (Tab) and 0x1C (Enter): `edit_key` answers 0 and the dialog
  takes them.
- **`edit_key`** (only with the focus): by scan code (the switch 0x5BAD1A) Home (71) to 0, Left
  (75) back one, Right (77) on one up to the length, End (79) to the length, Delete (83) the
  character at the caret removed; any other key translated (0x4EC6F0: the input manager's table
  at its +0x848, built by 0x4EC8C0 as a fixed US layout: a word a scan code, the low byte the
  character, upper-case letters, Backspace 0x7F, Tab 9, Enter 0x0A, Escape 0x1B; the backtick,
  Ctrl, the Shifts, Alt, Caps Lock, Num Lock and the keypad's digits none; F1 to F12 their whole
  word; without Shift, the event's +0xC & 3, a letter in lower case; with it the US shifted digits
  and marks, `[` and `]` unchanged); 0x7F removes the character before the caret; else nothing
  when the text and its terminator would reach the room (`len + 1 >= room`: 20 letters in 21) or
  the character is none, a function key's or a brace (0x5BABA0); else put in at the caret, the
  caret on one; then, unless +0x68 is 3, `text_line_count` of the text *before* the character at
  the box's width above one line undoes it (an off-by-one: the 21st character's check measures 20);
  the text stored (0x5BB660: the blink's 360 ms toggle of +0x6C, which nothing reads; in mode 3 the
  first shown character moved so the text up to the caret fits); drawn and shown with +0x6E; the
  event becomes 0x200/0xC (answer 2: taken). +0x68 is never written (the item is not zeroed): every
  edit built holds its room of the widest glyph in its box (20 × `M`, 260 px of `smalfont.fnt`, in
  281 and 334 px), so neither the scroll nor the line check bites. *Measured* (M2's review, the
  installation's `smalfont.fnt` through `h3_assets::fnt`): no glyph from 0x20 to 0xFF advances
  more than `M`'s 13 px; 0x85 `…`, 0x89 `‰`, 0x97 `—` and 0x99 `™` as much, every other less.
- **`edit_draw`:** its picture or saved background (none here), then `text_draw` 0x4B5490 of the
  text in the text box, its colour (3 when disabled), its alignment, and the caret with the focus
  (−1 without). **The caret** (`text_draw` 0x4B54BC, `text_draw_line` 0x4B5306, 0x4B53C6,
  0x4B544C): the glyph `_` over the glyph whose index in its line is the caret (in the colour
  without the highlight), after the last glyph drawn when the caret is their count (with it), at a
  line's start when none of it is drawn; an empty text's at the box's corner with neither the y
  offset nor the alignment.
- **Recorded** (`edit-keys`, every line as read): the caret `_` from the open (the name has the
  focus), not blinking; Home, End, Left, Right, Delete, BackSpace; capitals by Shift; `b1!@
  -=[];',./\` typed; 20 characters, the rest refused; Tab and Enter typing nothing; a click in it
  moving no caret; after Escape and a reopening the name empty again (`Network Default Name` is
  not written back by a cancel).

### 16.2 The dialogs

- **The Multiplayer dialog** casts a shadow (recorded; its constructor's flags are 0). Its status
  line (123) and the hot seat's (518): `smalfont.fnt`, colour 1, centred, their box filled with
  `game.pal`'s entry 0x20 (black: `text_item_draw` 0x5BC5F0's +0x48 through `game_palette16`,
  0x44E4C0) under the text. The texts' last argument 8 (the title 517 too) is not their state's
  bit 8 (`dialog_item_at` 0x5FF3E0 tests +0x16): they draw in their own colour (white).
- **Hints:** records 264 + id − 100 for the items 100 to 124 (`dialog_items_set_hints` 0x5FF8E0):
  the picture's (264) and the status line's (287) are empty, the name's edit (125) and the map's
  picture (129) have none: over them the status line empties. The hot seat's edits 559, OK 560,
  Cancel 561; OK's and Cancel's right-click texts are empty, so their popups show their hints
  (`dialog_right_click` 0x5FF790). Disabling (`dialog_item_enable` 0x5FE940) sets the state's 0x20,
  which `dialog_item_at` does not skip: a disabled OK keeps its hint and popup.
- **The hover** changes the status line only when the item under the mouse changes (+0x4C); a held
  button's moves are its own loop's; *recorded*: once a hold ends (`*-left`, the hot seat's
  Cancel), the dialog on top takes the pointer's place as a move.
- **The menu under the dialog** stays as it ended; the menus' last items are statics no opening
  clears (`main_menu_hovered` 0x67FA60, `game_type_menu_hovered` 0x6780F0, written by their
  procedures alone, `refs`): reopened after the dialog's Escape, New Game's menu does not draw
  Multiplayer hovered under the pointer left on it (`edit-keys`' `reopened`; CLT-Q128: fixed,
  decision 40).
- **Two names of 21 letters** (`hotseat-long`; §6.4's question): each edit holds 20, the 21st
  refused, so a stored name (21 bytes) always ends; the advanced options show the two apart (each
  cut to its row). No question.

### 16.3 What the client does otherwise (decisions 10, 20, 22)

Hot Seat works; IPX, TCP/IP, Modem, Direct Connect and Online show the network's notice in the
client's words (`CLT-STUB-6`, style 1), the dialog then on stage 1 (its status line the hint under
the pointer at the close); the scenario screen (a new game's, a load's: Load Game's multiplayer
leads there) is a stub over its game type menu until M3. Enter in a hot seat's edit presses an
enabled OK (CLT-Q107); with OK disabled it moves on, as the original's. A menu opening anew
takes the hover at its first move (CLT-Q128, decision 40: the statics for the comparisons only).
Typing follows the keyboard's own layout in the client's profile (CLT-Q127, decision 39: T7b,
§17). The original's US table stays for the comparisons: the replay gives the edits a recipe's
key names and typed characters as scan codes (`shell::keys::scan`, `h3_ui::edit::scan_of`: the US
layout, as `xdotool`). The window gives each key down's Shift (`h3_sdl`'s `Window::scans`); it
gave its set 1 scan code too until M2's review, which dropped it: since T7b the window runs the
client's profile alone, which never reads it.

### 16.4 Not built (the agent's calls, *provisional*)

Stage 2 and every network path (decision 10, E28); the CD notice (Hot Seat absent); the deletion
of a transfer's `RMT*SC.BIN`; the session list and its refresh; `Network Default Name` as a
setting (no client settings yet: the name starts empty); the edit's scroll and line check (no
edit built needs them, §16.1); its disabled colour; a key's repeats (the shell's window takes none).

## 17. The edits typing as the keyboard prints (M2 T7b, 2026-11-16)

Decision 39 (CLT-Q127 (b)) built in the client's profile; the original's side is §16.1's, its
scan codes kept for the comparisons. Nothing read. Built in `h3_ui::edit`, `h3_ui::keys`,
`h3_view::shell::{keys, multiplayer, menu, replay}`, `h3_sdl` and the battle client's shell
window.

### 17.1 The characters

- **The window's text input** (`h3_sdl::Window::set_text_input`, SDL's `SDL_StartTextInput`; the
  characters in `Window::typed`, each with the number of the wait's events before it) is on while
  an edit takes text (`Front::typing`: in the client's profile, the Multiplayer dialog's name with
  nothing over the dialog, the hot seat's focused edit without a popup; never under a popup or the
  network's notice, never in the original's profile) and off otherwise. What the keys print is
  the system's: its layout, Shift, Caps Lock, AltGr, dead keys and an input method's compositions.
  A held key's repeats type nothing (the shell's window takes no repeats: the text of a repeat it
  leaves out is left out too).
- **The edit** (`Edit::typed`): a character goes in at the caret unless the text with its
  terminator would reach the room (20 characters in 21, as the original's) or the fonts cannot
  draw it (`edit::typable`: Windows-1252's printable characters, the controls and the braces left
  out; every byte from 0x20 to 0xFF has a glyph of some width in `smalfont.fnt`, read from the
  LOD; the widest 13 px, `M`'s, so 20 of any fit the boxes as in §16.1: measured at M2's review). The text, the caret and the room count characters (one byte each in the code page; the
  draw encodes them, `encode_1252`).
- A character's key down still reaches the key table while an edit types (no dialog with an edit
  binds a letter; *provisional*).

### 17.2 The keys

Commands of the shell's key table (`[shell]` in a user's key file), the original's keys by
default:

| Command | Keys | Taken |
|---|---|---|
| `edit-home`, `edit-end`, `edit-left`, `edit-right` | Home, End, Left, Right | while an edit types (`Context::Edit`) |
| `edit-delete`, `edit-backspace` | Delete, BackSpace | while an edit types |
| `hot-seat-next` | Tab, Down | over the hot seat's names |
| `hot-seat-previous` | Shift+Tab, Up | over the hot seat's names |
| `hot-seat-ok` | Return | over the hot seat's names: OK enabled presses it (CLT-Q107), disabled moves on (the original's Enter) |

- **The lookup** (`shell::keys::command_for`): `Context::Edit` first while an edit types, then the
  dialog's own (`Front::contexts`); the chord with Shift as held, else, with Shift held and that
  chord bound nowhere there, the key alone (Shift and Escape still cancel, Shift and Home goes
  home, as before T7b when the shell read no Shift). Since M2's review a user's file binding one
  chord to an edit's key and to its dialog's (the Multiplayer dialog's, the hot seat's) is
  refused as within one context (`h3_ui::keys::Command::asked_together`, the shell's `Edit` with
  either): `edit-home = ["Tab"]` would have shadowed the ring.
- **Shift chords** in `h3_ui::keys` (a `Shift+` chord, a parse error before, now parses: a change
  for the map's table, its inbox told): `Chord::shift`, `Shift+` in a key file after
  `Ctrl+` (`Ctrl+Shift+Up`), `Keys::chord_command` matching exactly; the Shifts never bound alone
  (`keys::SHIFT`, as Control). The window's new keys: Tab, BackSpace, Delete, LeftShift and
  RightShift (`h3_sdl::Key`).
- **The keypad's** navigation keys are not bound: SDL names them alike in either Num Lock state,
  and with Num Lock on they type their digits.

### 17.3 The replay and the checks

- **The original's profile** (every `drawn` comparison) as T7 built it: scan codes by the US
  table. `drawn`: 91 recipes as recorded, 235 shots drawn, all exact (`multiplayer`'s 113 among
  them).
- **The client's profile** in the replay (the tests): a recipe's typed text gives each character
  between its key's down and up as the window's text input would, where an edit types; `key:`
  with `shift+` asks the chord with Shift.
- **The window under a German layout** (Xvfb, the keymap `de(basic)` as its default: TRAPS):
  the keys by their places typed `zyüöäß`, Shift and 2 a `"`, the dead acute and e an `é`, Caps
  Lock a capital; Home, Delete, End and BackSpace as the table's commands; the hot seat's Tab,
  Shift and Tab, and Enter pressing an enabled OK (the scenario screen's stub).


## 18. The scenario screen's frame and information panel (M3 T1a, 2026-11-17)

Read from `Heroes3.exe` for the new-game mode with the list hidden (§3.1, §3.2, §3.5), and
recorded again (`scn-new`, `scn-list-keys`: as kept). The client: `h3_view::shell::scenario`, the
layout's `[scenario]`, `h3-battle-client`'s `maps`.

### 18.1 The items' drawing

- **A caption button** (`H3DlgCaptionButton`, created by 0x456A50, vtable 0x63BB88, drawn by
  0x456CA0): the DEF button's frame (`def_button_draw` 0x456940), then its caption (+0x5C) by
  `text_draw` in the button's box (+0x1C, +0x1E) in its font (+0x68), alignment 5, colour +0x6C
  (2 more while the state has 0x28); the box at (x, y − 1) while released, (x + 1, y) while
  pressed (state bit 1). The panel buttons: `gspbutt.def`, `smalfont.fnt`, colour 4, frames 0
  and 1; hotkeys 0x1F (S, 128), 0x1E (A, 129), 0x13 (R, 130); built in that order (128, 129,
  130). `h3_ui::ui::Caption`.
- **The scrolling text** (`H3DlgScrollableText::Create` 0x5B9FB0, the description: (422, 155,
  319, 115), `smalfont.fnt`, colour 4): the text split into lines at width − 24; more lines than
  the box's height ÷ the font's height takes the scroll bar's path (0x5BA19E, *not read*: T2);
  else split again at width − 3, padded with empty lines, each line a text item at (x, y + k ×
  height, width, height), alignment 0. The client draws the fitting case as one label of width
  − 3 (the drawer's lines are the font's height apart: the same pixels, `scn-new` exact).
- **The labels** (`text_item_create`, all `smalfont.fnt`): 132 "%s:" (0x660D28) of GENRLTXT 493
  at (414, 435, 334, 19), 133 "%s:" of 219 at (665, 435, 84, 19), 134 GENRLTXT 495 at (414, 435,
  90, 19), colour 2, alignment 5; the name's (496), the description's (497), victory's (498) and
  loss's (499) labels (id 100, 105, 100, 100) at x 422, width 278, height 18, colour 2, alignment
  4; the allies' "%s:" of 391 (id 100) at (414, 403, 44, 23) and the enemies' of 392 (386) at
  (579, 403, 58, 23), colour 4, alignment 6. The flags: `itgflags.def` items 112 + i at (460 + 15
  i, 405) and 120 + i at (640 + 15 i, 405), 15 × 20, hidden; the strip 387 (456, 402, 310, 25)
  added after them.
- **The redraw's own** (`scenario_info_draw` 0x5840F0, §3.5 corrected): the name in `bigfont.fnt`
  colour 8 **alignment 0** at (422, 45, 324, 30); the map's difficulty and the rating ("%d%%",
  0x6831E4) colour 4 alignment 5; victory's text ("%s %s %s", 0x6837B4, with GENRLTXT 5 and the
  standard's record when the map allows it, the record's +1) and loss's at alignment 4; the
  marks by `def_draw_plain`. The client draws them as items of its own ids (1000 on), no hint.
- **Begin and Back:** `scnrbeg.def` (a new game's; frames 0, 1; hotkey 0x1C) at (414, 535, 166,
  40), `scnrback.def` (frames 0, 1; hotkey 1) at (584, 535, 166, 40). The difficulty's buttons
  `gspbut3` to `7.def` at (506 + 32 i, 456, 30, 46), frames 0 and 1, the chosen selected (3).

### 18.2 The texts and the draws

- **`difficulty_names`** (0x6A77EC, §6.4's item): `arraytxt_load` copies ArrayTxt.txt's records
  142 to 146 (after the label 141; the map sizes 137 to 140 into 0x6A7D94, the directions 148 to
  156 into 0x6A5C48), *read*.
- **The background's draw** is the client's own randomness; under the harness the original's
  draws are fixed by its frozen clock (`scn-new`: gamselb0, gamselb0, gamselb1 at its three
  openings), so the replay takes them from a recipe's `backgrounds` (*provisional*).
- **The cursor** (CLT-Q137): the constructor's busy cursor (`busy_cursor_start` 0x577790:
  `mouse_set_cursor`(1, 1), `busy_cursor_stop` 0x577810: (0, 1)) leaves the adventure map's
  arrow, `cradvntr.def`'s frame 0 (the cursors' DEFs by type, 0x67FF38: `crdeflt`, `cradvntr`,
  `crcombat`, `crspell`), until the menu loop's top sets `crdeflt.def`'s again (0x4EE8CC: (0, 0)).
  The two arrows have the same shape in other colours (one level of green or blue in 38 pixels):
  `scn-new`'s 25 shots with the cursor and New Game's menu after its Escape show `cradvntr`'s.
  The client keeps the default arrow, the original's for the comparisons only (CLT-Q137 (b),
  decision 46).
- **M2's leftover** (the keys without an `h3_sdl::Key` variant): met since E17 M5 T8b2, they reach
  the window as `Key::Other`, which skips the start (checked in the window under Xvfb: 7 and
  Insert skip the start, F4 does not); a hotkey still needs a variant (TRAPS).

### 18.3 Not built (T1b on)

The list and its panel (128's command, GENRLTXT 503), the right press on the flags' strip (the
teams' dialog, 0x576540) and on the labels and texts (`scn-right`), the description's scroll bar,
the difficulty's presses and Begin (T2), the advanced options (T3), the hot seat's mode (T4), the
random map (T5).

## 19. The list (M3 T1b1, 2026-11-17)

Read from `Heroes3.exe` for a new single scenario's list (§3.3, §3.4 corrected where marked) and
recorded again (`scn-list-keys` with §6.4's fix; `scn-sort`, `scn-filter`, `scn-new` as kept).
The client: `h3_view::shell::scenario` (the list), `h3_ui::scrollbar`, the layout's
`[scenario.list]`.

### 19.1 The items and the rows

- **The constructor's items** (`def_button_create` 0x455EF0 with frames 0 and 1, no hotkey): the
  filters 137 to 141 (`scsmbut` … `scalbut.def`) at (161 + 47 i, 52, 44, 33); the columns 190 to
  195 at (26, 92, 31, 28), (58, 92, 32, 28), (91, 92, 32, 28), (124, 92, 184, **33**), (309, 92,
  32, 28), (342, 92, 32, 28); the panel 101 (`SCSelBck.pcx`, flags 0x800) and the title 361
  (`medfont.fnt`, colour 8, alignment 5) hidden at once (`dialog_item_send`(6, 6)), shown by
  (6, 5). The rows 142 to 159 are text items at (57, 122 + 25 i, 314, 25) with no text: a row
  takes the press from x 57, not under the players column.
- **The redraw's rows** (`scenario_dialog_redraw` 0x584550): with the list shown GENRLTXT 511 at
  (25, 52, 132, 32) colour 2; then min(count, rows) rows from the top, each within the list: colour
  5 for the selection, else 4, `smalfont.fnt` alignment 5; "%d/%d" (0x679DC8) of +6 and +8 at (26,
  y − 1, 30, 25); GENRLTXT 512 to 515 by the size (36, 72, 108, another) at (59, y − 1, 30, 25);
  `ScSelC.def` by the format (0x0E 0, 0x15 1, 0x1C 2, another none) at (91, y); the name (184
  wide) at (125, y − 1); `scnrvict.def` +0x30 (0 to 11, else 11) at (309, y); `scnrloss.def` +0x7C
  (else 3) at (342, y); y = 123 + 25 i. The client computes +6 and +8 from the seats (the colours
  anyone plays, those a human may: `scn-list-keys` exact with them).

### 19.2 The scroll bar (`h3_ui::scrollbar`)

`scrollbar_create` 0x5960D0 with (375, 92, 16, 480), 10 ticks, blue (`SlideBuV.def`, the track
`sliderV.pcx`), a page of 18 rows, no arrow keys. Its rules (`scrollbar_proc` 0x5964E0 and its
helpers, read whole for a vertical bar): the module's notes. What the list needs: an arrow steps
at the **release** (drawn pressed meanwhile, no repeat while held); the track pages by 18 at the
press; the thumb drags; the callback (`scenario_list_scrolled` 0x57C990: the top the value, then
`scenario_select` with the selection and the redraw flag) only on a change since the last report
(+0x38: a set by the screen counts as one, so the next press reports even unchanged).

### 19.3 The list's commands (§3.3, §3.4 corrected)

- **Shown** (`scenario_list_toggle` 0x580A70): each showing resets the bar to count − rows + 1
  ticks and sets it to the top; the first builds (`scenario_list_finish`: sorted by key 2, the
  recorded key 1, `scenario_find_by_name` 0x58D3E0: the kept file's row selected and on top, the
  top at most count − rows; the bar's value the top).
- **The keys** (`scenario_dialog_proc` 0x587D00) redraw whole first (0x587ED5), then select:
  Up nothing on the first row (no redraw), Down nothing on the last; Page Up the top less rows − 1
  (at least 0), Page Down more (at most count − rows; **nothing in a list of one page**), the
  selection keeping its distance from the top (a selection off the page too; none past the list;
  before it the original selects a negative index, the client nothing: *provisional*).
- **A row** (`text_item_message` 0x5BC440): the click at the release wherever the mouse is
  (CLT-Q43 (2)'s rule: the client's over the row); within 400 ms of the last row's click or the
  screen's opening Begin, else selected (§3.4).
- **A sort** redraws whole (0x5852BF) before the click selects row 0, which it already is (+0x374
  0): the select's partial redraw follows every sort (§19.4). **A filter** selects without a
  redraw, then redraws whole; it sets the size mark 189 to its own size's frame first (0x585EFB),
  which the select replaces (T2: an empty list's panel keeps it).

- *Added at M3 T7 (the review's A5):* with the selection off the page (a move of the bar), Up and
  Down step the selection; the original's Down moves the top a row (the selection still off the
  page), its Up brings the top to it only from above; the client's first brings the page to the
  selection (CLT-Q149 (b), §33.3).

### 19.4 The partial redraw (CLT-Q138)

`scenario_select` 0x585500 with the redraw flag and an unchanged selection redraws part
(0x585A06): the panel 101, the title, the filters, the columns, the bar, then the info draw and the
rows; every other item keeps its pixels. Seen in the captures: after any sort the size mark shows
the previous selection's size (`scn-sort` name-1 and others: L for "All for One", an M map); the
panel's opaque bottom edge (rows 574 to 590, x 393 to 577) covers Begin's last row; the flags'
strip, restored from its copy of the background (`scenario_team_flags` 0x585300, kept at its first
call) without the allies' label drawn again, loses the label's colon column at x 456 (rows 413,
418). After a move of the bar, a row's click on the selection, a key that keeps the selection, a
sort. The client, original's profile: the items the partial redraw leaves drawn first with the size
mark's last whole frame, those it draws over them after, the strip as a stale rectangle
(`Look::stale`); the client's profile draws whole (CLT-Q138 (b), decision 48).

### 19.5 The recordings and the comparisons

- **`scn-list-keys`** recorded again (§6.4's item): seven Page Ups after `up-past-top` bring the
  top to the list's first row (`page-up-first`), Up there changes nothing (`up-at-first`); the 16
  shots kept, both runs the same.
- **`drawn`:** `scn-new` 31 of 31, `scn-list-keys` 16 of 16, `scn-filter` 7 of 7, `scn-sort` 5 of
  13 exact. The 8 others (size, players, victory and loss, each way) differ only in the order of
  their ties (decision 12): every row's key column is the original's, the names among equal keys
  are not (the original's library sort over the folder's order). Since T1b1 `drawn` gives a
  recipe's `maps` to the viewer (`--maps`): `scn-info` 19 of 21 (row-08 and row-15: the victory's
  text, T2), `scn-begin-filter` 3 of 6 and `scn-begin-none` 0 of 4 (the empty list's panel and
  Begin's message, T2), `scn-difficulty` 3 of 8 (the presses, T2; `impossible-list`: the build sets
  `scenario_last_difficulty` from the selected record's +0x33C, 0x58314E; **corrected at T2a,
  §21.1:** that build runs at the opening, not at the list's first showing).

### 19.6 Not built (T1b2 on)

GENRLTXT 503 while the list is read (§3.3: drawn after the items, before the build; a shot of it
needs the read held; **corrected at T1b2, §20.3: a network game's alone**), the right press's
popups on the columns, the filters, the bar and the labels and texts (`scn-right`), the flags'
strip's teams dialog (0x576540); T2's items above.

## 20. The right press and the teams' dialog (M3 T1b2, 2026-11-17)

Read from `Heroes3.exe` (§3.8 completed, §3.3 corrected) and compared with `scn-right` (its
original's frames played again as kept). The client: `h3_view::shell::scenario` (`HELPED`,
`TEAMS`, `BAR`, `TeamsLook`, `Teams`, `teams_names`), the layout's `[scenario.teams]`,
`h3_view::message::frame` (public now).

### 20.1 The right press (`scenario_dialog_right_click` 0x5822D0)

- Rows 142 to 159 nothing (true at once); 362 to 385 the players' popups (T3); 387 the teams'
  dialog (§20.2); any other id, and 386 (case 4), `dialog_right_click` 0x5FF790 with the item's
  hint (`Help.txt` record id − 83 for 104 to 344, §3.8), then 0x58AE80, the dialog's redraw
  (vtable +0x14 with 0, 0xFFFF0001, 0xFFFF) and `scenario_dialog_redraw`: the client's whole
  redraw as the popup closes (*corrected at M4 T4b1*, §39.3: no redraw without a popup).
- The item is the last in order whose box holds the point (`dialog_item_at`): 134 (`MAP_SIZE`,
  record 51) over 132 (record 49) at x 414 to 503, 132 from 504 to 664 (no recipe), 133 record
  50; 105 the description's label, record 22; the filters 137 to 141 records 54 to 58; the
  columns 190 to 195 records 107 to 112; the bar (337, created after the panel 101) record 254.
  The flags 112 to 127 and the enemies' label 386 lie under the strip 387 (later in order): their
  hints never show. The name, the description and the victory's and loss's texts are the
  redraw's (the client's own ids): nothing, as recorded.

### 20.2 The teams' dialog

- **Built** (`scenario_teams_dialog_new` 0x5764D0 with 1 unless +0x65 or +0x64; then
  `scenario_teams_dialog` 0x576540): the base (0x48FE60, flags 0x12: a shadow), the teams
  (`scenario_teams_group` 0x576930): each colour that plays (game +0x1F6D0 not negative) and not
  yet in a team starts one, every later colour that plays of its team (game +0x1F879) joins it;
  the teams in the order of their first colour.
- **Its items:** the frame (`dialog_frame_build` 0x48FF00: `diboxbck.pcx` tiles, `dialgbox.def`
  in the DEF's own colours, not recoloured) at (272, (600 − h) / 2), 256 wide, h = ((50 × teams +
  56) / 64 + 1) × 64 (4 teams: 320); the title GENRLTXT 658 (`medfont.fnt`, colour 1, alignment
  1; its braces the highlight) at (10, 20, 236, 36); each team *k* from 0 "Team %d" (GENRLTXT
  657) of *k* + 1 (`smalfont.fnt`, colour 1, alignment 1) at (10, 56 + 50 *k*, 236, 18) and its
  colours' flags (`itgflags.def`, the colour's frame) 18 apart from x (256 − (18 *n* − 3)) / 2, at
  y 76 + 50 *k*, 15 × 20.
- **Run** modally (`dialog_run_modal`): the cursor shows (the arrow as it was); its procedure
  (`scenario_popup_proc` 0x575430, the four players' popups' too) ends at the right button's
  release (message 0x40) and ignores the rest (a pending network message ends it too). Then the
  redraw as §20.1's.
- *Provisional:* the client's ids for the original's −1 (the title 1, the names 2 on, the flags
  10 on; the frame's from 200); no dialog without a selection (an empty list's panel: T2).

### 20.3 GENRLTXT 503: a network game's alone (§3.3, §19.6 corrected)

The toggle draws GENRLTXT 503 only with +0x1864 clear, and `scenario_slots_new` 0x57F740 (the
constructor's) builds the list at the opening and sets it, as `scenario_slots_load` does: a
single scenario's, the hot seat's and a load's first showing reads nothing. The flag is cleared
only by `scenario_list_reread` 0x58B120, a network game's message 1015 (`scenario_network_message`
0x5887A0, codes 1000 to 1084): the list read again with the busy cursor and 503 shown. So 503
belongs to the network modes (`CLT-STUB-6`); no shot is possible before them. The client builds
the list at its first showing (T1b1), which shows the same as the original's build at the
opening.

### 20.4 The recordings and the comparisons

`scn-right` played again (both runs as kept); `drawn`: 30 of 30 exact (15 before: the columns,
the filters, the bar, the two labels, the teams), recorded. The `scenario` group otherwise as
recorded (`scn-new` 31, `scn-list-keys` 16, `scn-filter` 7 exact; the others as at T1b1).

## 21. The difficulty, the empty list and Begin's message (M3 T2a, T2b, 2026-11-17)

Read from `Heroes3.exe` (§3.5, §3.6 completed) and recorded (`scn-difficulty` with `reopened`,
`scn-begin-none` with the strip's right press and a difficulty's press, `scn-empty-filter` new;
decision 50). The client: `h3_view::shell::scenario` (`OPENING_DIFFICULTY`, `choose`,
`show_empty`, `begin`, `Over::Notice`), `h3-battle-client`'s `maps` (`STANDARD_ALWAYS`), the
catalogue's `shell.scenario.no-maps`.

### 21.1 The difficulty and the victory's text

- **A press** (`scenario_difficulty_click` 0x586BF3, read whole): game +0x1F6D8 and
  `scenario_last_difficulty` 0x683454 take the id − 107; the five buttons lose state 0x10 and the
  chosen one gains it; a network host's message (nothing in a single game, not read); the dialog's
  redraw (vtable +0x14 with 0, 0xFFFF0001, 0xFFFF) and `scenario_dialog_redraw`: a whole redraw.
- **The opening's difficulty:** `scenario_slots_new` (the constructor's, 0x57FADC) builds the list
  every opening; `scenario_list_finish` 0x58314E then sets `scenario_last_difficulty` from the
  selected record's +0x33C and `scenario_to_game` copies it into the game (0x583673). A new
  game's map record holds 1 there (its constructors 0x579049 and 0x579177 set it; no header read
  writes it), so **each opening shows Normal** whatever was chosen before (recorded: `scn-difficulty`
  `reopened` after Impossible; T1a's "kept between visits" corrected). The list's first showing
  builds nothing (§20.3), so it keeps the choice (`impossible-list`). The constructor then selects
  the game's button (`scenario_difficulty_show` 0x586060, from 0x57C556) only when the shown list
  is not empty (0x58EAB0's count) or in the save mode.
- **The victory's "or" text** (`scn-info` row-08, row-15): `scenario_info_draw` joins the standard
  victory when the game's victory record has +1 set (+0x1F89D, read), and the header's read sets
  it (`victory_standard_set` 0x4BF780, called with 0 from the conditions' read 0x4C3555; with 1 from
  `new_game_heroes`): goal 5 (a hero's defeat) always; goals 0, 4 and 10 (an artifact, the Grail,
  a transport) cleared when exactly one colour is not defeated (+0x1F636; `game_reset` clears
  every flag before each header read, so never at the screen), else set outside a campaign (a
  campaign by its ids); the others keep the file's flag. The shipped maps: goals 0, 4 and 10 all
  carry the flag, the five of goal 5 all lack it. VCMI 1.7.5 (`MapFormatH3M.cpp`) agrees for goal 5
  ("H3 behavior") and clears the flag for any goal on a map of one player. The client's entries
  take it (`maps::STANDARD_ALWAYS`); the rule at the game's start is E10's (the adventure lane's
  inbox).

### 21.2 The empty list

- `scenario_select` past the list does nothing (0x58553C), so an emptied list leaves the game's
  map as it was: the last one shown after a filter (the filter calls `scenario_to_game` only for
  a list with a record, 0x585E8C), the reset game with no map at all.
- **The panel:** `scenario_info_draw` returns at once for an empty list outside the save mode
  (0x584128): no name, texts, marks, map's difficulty nor rating. The size mark keeps its frame: a
  filter sets its own first (0x585EFB: 36, 72, 108, 144 tiles frames 0 to 3, All 4), with no map
  at all the constructor's S. The flags keep the last map's (items' states). Recorded:
  `scn-begin-filter` extra-large and after, `scn-begin-none` screen and list, `scn-empty-filter`.
- **The difficulty's buttons:** with no map at all none shows chosen (0x586060 skipped) until a
  press, which selects its own (the rating still not drawn; `scn-begin-none` hard); after a filter
  the choice stays shown (`scn-empty-filter` hard), and All brings the map back with it (`all`).
- **The teams' dialog** (§20.2) groups the game's players: after a filter the last map's teams
  (`scn-empty-filter` teams: Arrogance's four), with no map at all the reset game's, every colour
  playing in team 0: "Team 1" with the eight flags (`scn-begin-none` teams).
- **The client's profile** (CLT-Q139 (b), decision 51): no flags and no teams' dialog over an
  empty list, the chosen difficulty always shown; the size mark's frame 4 (CLT-Q115 (b)); no
  rating, as the original. The original's for the comparisons.

### 21.3 Begin's message

`scenario_begin` checks the game's map's format before the list (§3.6): with no map at all the
reset game's format is none and Begin does nothing (`scn-begin-none` begin); after a filter
emptied the list GENRLTXT 530 shows in an OK box (`scn-begin-filter` begin), Enter's or the OK's
release closing it. The box's build (`message_box_build` 0x4F6990) keeps the cursor (the mouse
manager's +0x4C, +0x50), sets the default arrow for its loop (0x4F74AC) and puts the kept one back
after (0x4F753A): the adventure map's arrow of CLT-Q137 shows `crdeflt.def` in the box
(`scn-begin-filter` begin) and comes back at its close (`after`). The client: 530 over any empty
list (CLT-Q114 (b)), the original's only with a map of a known format shown before; Begin with a
map is T6's.

### 21.4 The recordings and the comparisons

`scn-difficulty` (`reopened` added), `scn-begin-none` (`teams`, `teams-closed`, `hard` added)
and `scn-empty-filter` (new: `extra-large`, `teams`, `teams-closed`, `hard`, `all`) recorded,
both runs alike; `scn-info`, `scn-begin-filter` played again as kept (the latter's load clip
1900 to 2100 or 2110 ms: the bar fills in real time). `drawn`: `scn-info` 21 of 21,
`scn-difficulty` 9 of 9, `scn-begin-none` 7 of 7, `scn-empty-filter` 5 of 5, `scn-begin-filter` 6
of 6 up to its load; the `scenario` and `advanced` groups otherwise as recorded.

### 21.5 Not built (T2c on)

The description's scroll bar (`H3DlgScrollableText`'s path for more lines than the box holds,
0x5BA19E, *not read*): no shipped map reaches it (159 maps: the longest descriptions fill 7 of the
box's 7 lines at width − 24, 300 characters the longest), so its recording needs a map of
the harness's own with a longer text (T2c: §22). Begin with a map (T6).

## 22. The description's scroll bar (M3 T2c, 2026-11-17)

Read from `Heroes3.exe` (`H3DlgScrollableText` whole: `scrollable_text_create` 0x5B9FB0, its
vtable 0x642D0C, `scrollable_text_set` 0x5BA6E0, the bar's subclass 0x642CC8; `font_split_lines`
0x4B5B90; `scrollbar_key_step` 0x596230) and recorded (`scn-long-text`, new: a map of the
harness's own). The client: `h3_view::shell::scenario::description` (`Description`,
`DescriptionLook`, the layout's `[scenario.description]`), `h3_render::text::split_lines`,
`h3_ui::scrollbar`'s keys (`KeyStep`), the harness's `made` maps.

### 22.1 The scrolling text

- **Built** (§18.1 completed): id −1; the text split at width − 24 (319 − 24 = 295); more lines
  than the box shows (115 ÷ 16 = 7) keep that split, else it is split again at width − 3 and padded
  with empty lines. Seven text items (id −1, alignment 0, colour 4) at (422, 155 + 16 k, 319, 16)
  hold the first lines; the bar (`scrollbar_create`) at (725, 155, 16, 115), id −1, lines − 7 + 1
  ticks, no callback, blue, a page of 7, its arrows' keys. Added to the dialog (vtable +4,
  0x5BA2E0) the items and the bar come after it; a text that fits hides the bar.
- **Set** (`scrollable_text_set`, from `scenario_to_game`): split again as built, the bar reset to
  the new ticks at 0, shown (0x5BA82C) or hidden; each item takes its line; nothing drawn.
- **Scrolled** (the bar's +0x40, 0x5B9FA0 → 0x5BA600, at every change its press, move, release or
  key reports): the box less the bar's column put back from its copy of the screen (taken at its
  first redraw, 0x5BA571), the items given lines value to value + 6 and drawn, the box shown.
- **The record's 300 bytes** (`scenario_header_read` 0x583170: `strncpy` of 0x12C into +0x5C9;
  `scenario_to_game` gives that copy): Jihad's description of exactly 300 shows whole with nothing
  after it (`scn-long-text` `jihad`: the record's next byte 0 there). The client cuts at 300
  (`DESCRIPTION_BYTES`; past them the record's next bytes are *not read*, *provisional*).

### 22.2 The split (`font_split_lines` 0x4B5B90)

Not the drawer's breaking (§18.1's one label held on the recorded maps): words up to a space, a newline or the end; **each byte at its glyph's
advance, `{` and `}` too** (7 pixels each in `smalfont.fnt`; the drawer counts neither); a newline
ends the line (the spaces before it dropped, an empty line kept); a word joins with its spaces
while the line stays at most the width, else the line ends and the word starts the next without
them; a word wider than the width is cut byte by byte. Each line being an item of its own, a
highlight ends with its line (`scn-long-text`: "past the break,}" drawn plain). The client draws
the lines shown as one label, a line left highlighted closed by a `}`, the fitting case too (the
shipped maps' recipes stay exact; none of the 159 descriptions has a brace).

### 22.3 The bar and its keys

- The mouse as the list's (§19.2): an arrow at the release, the track a page of 7 at the press,
  the thumb dragged.
- **Its keys** (`scrollbar_proc` 0x5964E0 with +0x5C, read): at a key's **release** (message type
  2) while the bar shows and is enabled: Up and Left a line back above 0, Down and Right one on
  below the last, Page Up and Page Down a page (`set_value`, clamped) at any value. The step draws
  its arrow pressed, waits 60 ms of the clock (`scrollbar_key_time` 0x6989A0), then shows the new
  value (the client steps at once, *provisional*: no recipe's step is under 60 ms).
- **With the list's keys** (CLT-Q140): the list moves at the key's **press**
  (`scenario_dialog_proc`, message type 1; its table has nothing for Left and Right), the items
  take the release first: Down onto a long description selects it at its first line, then its
  release steps the bar (`down-pressed`, `down-released`: shown from its second line); Down on
  the last row and Page Down in a list of one page step the bar alone; Up onto a short one finds
  no bar. With the list hidden the keys step the bar alone. The client's profile (CLT-Q140 (b),
  decision 52): while the list shows the bar takes none of the list's keys, Left and Right always.

### 22.4 What a select resets, what the screen shows

- `scenario_select` calls `scenario_to_game` whatever the change (0x58576F), so every select sets
  the description anew (line 0, the bar at 0); `scenario_list_sort` calls it too, before its whole
  redraw (0x5852A1, 0x5852BF), the click's select after. A difficulty's press and the list's
  showing select nothing: the scroll stays (`hard`, `list-hidden`).
- **The partial redraw** (§19.4, CLT-Q138) draws neither the items nor the bar: the select of the
  row already selected leaves the screen's lines and thumb as last drawn while both are reset
  (`row-again`); the top arrow at 0 then draws the bar alone (`row-again-up`: the thumb at the
  top, the lines still scrolled); Page Up over the list redraws whole, selects, then its release
  draws the bar at 0 (`page-up-list`). The client's original profile keeps what it last drew
  (`Description::drew`), its own draws whole (decision 48).
- **A right press** that shows no popup (no hint, not a row) redraws the screen whole at once
  (§20.1): the client's `right` now does so (the partial redraw's leftovers gone). *Corrected at
  M4 T4b1* (§39.3): the description's box alone (its scrolling text) and the players' items.
- **The kept map** (`scenario_last_map` 0x683238): every select of a new game keeps its file's
  name, the next opening selects it (`reopened`: "Arrogance Retold"). The client keeps it at every
  select and carries it to the next opening (`Menus::front`, `Scenario::keep`); before T2c it kept
  Arrogance.

### 22.5 The harness's own map and the recording

A recipe's `[[recipe.made]]` (`file`, `from`, `name`, `description`) makes a shipped map's copy with
its header's name and description replaced (`h3shell.py`'s `made_map`: the 10 bytes before the
name, the two strings, the rest as read, compressed again without a time), written into the run's
`Maps` at each run's start and into `drawn`'s folder, the viewer given its path (`--maps`: a name
with a `/`). Never kept, never committed (ADR-0009); the description is ours. `scn-long-text`:
Jihad (AB), Arrogance and "Arrogance Retold" (SoD, 277 bytes: two spaces, three before a newline,
an empty line, an indent, a word wider than a line, braces and a highlight across a line's end,
code-page letters; 18 lines at 295, 12 ticks). 36 shots, both runs alike; `drawn` 36 of 36 exact,
every other `scenario` recipe as recorded.

### 22.6 Not built (T3 on)

The 60 ms of a key's step (above); a description past 300 bytes (the record's next bytes); a
scrolling text elsewhere (the message box's long texts, 0x4F6BC7: M4's or the map client's,
through `Description` or the foundation's when needed).

## 23. The advanced options: the panel, the rows, the town and the bonus (M3 T3a, 2026-11-18)

Read from `Heroes3.exe` (`scenario_advanced_toggle` 0x581100's showing and its per-colour loop;
the constructor's colour loop 0x57AF82 to 0x57B51B; `scenario_player_row_draw` 0x58D510 whole but
the hero's portrait; `scenario_dialog_click`'s town arrows 0x586710, 0x586894 and bonus arrows
0x5873C7, 0x58753C; `scenario_bonus_popup` 0x5754F0, `scenario_info_popup` 0x5757E0 and
`scenario_town_popup` 0x575E60 whole; `scenario_dialog_right_click`'s bonus texts 0x5828A4;
`scrollbar_create` 0x5960D0's art) against `adv-single-town`, `adv-single-hero` and
`adv-single-maps` (played once: as kept). The client: `h3_view::shell::scenario::advanced`
(`Advanced`, `AdvancedLook`, `AdvancedTexts`, `Record`, `Bonus`), the layout's
`[scenario.advanced]` and the panel's items, `Scenario::records`, `h3_ui::scrollbar`'s horizontal
draw, the keys `shell.advanced.*`, the seats' factions and flags (`maps::seat`). §3.13 and §3.14
corrected below.

### 23.1 The items and the showing

- **The constructor** (0x57AF82): per colour the flag (`AOFLGB%c.DEF` at (14, y − 3), frames 0
  up and 1 pressed), who may play (199), the handicap (207: a caption button in the hot seat, a
  plain DEF button single), the arrows (left `adoplfa.def` frames 0 and 1; **right `adoprta.def`
  frames 1 up and 0 pressed**, 0x57B19B: a right arrow drawn with frame 0 looks pressed), the name
  (345), the hero's, the town's and the bonus's areas (362 at 252, 370 at 176, 378 at 328: §3.13's
  table named 370 the hero's), the edit (353, single); after the loop the headers and the
  duration's label. The client inserts the colours' items before the headers (layout order
  otherwise); the panel 102 is cut to its item's 557 pixels (`AdvOptBk.pcx` is 575 wide).
- **The showing** (0x5813B9 to 0x58197F): the list and the random map hidden, 102, the headers and
  the duration's bar (set to +0x378) shown; with +0x1854 set every record's colour, town and hero
  −1 and `scenario_players_init`; then **`scenario_to_game` always** (0x5814B3: the description
  from its first line); per colour that plays its items moved (the arrows to 133 + 50 *r*, the
  flag, the areas and the name to 130 + 50 *r*, 199 and 207 to 151 + 50 *r*), the flag shown and
  enabled where a human may play it (never selected: §3.14's highlight is the hover's, T3c), 199,
  207, 345 and the areas shown; single the handicap's bit 2 cleared (`dialog_item_send` 6, 2: not
  enabled, still drawn up) and the edit hidden.
- **The mouse** (the client): the options' hidden headers, title and rows lie over the list's
  columns, filters and rows; the original's `dialog_item_at` 0x5FF3E0 passes over hidden items,
  the shared input (`h3_ui::input`) does not, so the screen hands the mouse its dialog without
  them (`mouse_dialog`; found by `scn-sort`, `scn-filter` and `scn-info` going not as recorded).

### 23.2 A row's draw (`scenario_player_row_draw` 0x58D510)

After the dialog's items, per colour that plays: the flag (item), `adop%cpnl.pcx` (336 × 49) at
(57, 128 + 50 *r*), 199 and 207 (items); the town's arrows shown and drawn when
`scenario_town_choosable`, else hidden; the town (`itpa.def` frame 2 *t* + 2 or `hpsrand0.pcx` at
(176, 130 + 50 *r*), `town_type_names` or GENRLTXT 523 tiny colour 4 align 5 at (164, 162 + 50 *r*,
71, 16)); the hero's arrows by `scenario_hero_choosable`; the hero (0x58DB0F: none shown is
**Random** while its arrows show, or the colour has the generating main town or the random-hero
flag, or under a Random town it has a hero at all; else **none**, `hpsrand6.pcx` and GENRLTXT 524);
the bonus's text, then its picture (`ScnrStar.def` at (328, 130 + 50 *r*): 9 artifact, 8 gold, the
shown town's index for a resource, Conflux 3, 10 Random; a resource under a shown Random town is
**written** Random, 0x58DDAB); the bonus's arrows (0x58DD48: the local colour's; single or a host
also a computer record's whose town shows a town or may change; the hot seat every colour's); the
name (item). Every arrow's press redraws its row alone (no other item changes).

### 23.3 The arrows

- **The town's** (0x586710, 0x586894): the hero −1; from the shown town one back or on (left
  from −1 is −2, taken as 8; right past 8 is −1) until −1 or a faction of the mask (Conflux's bit
  cleared when +0x1898 is 0: a single player's version is SoD, so the map's mask alone decides);
  then `scenario_hero_lists`, `scenario_heroes_unclash`, the row.
- **The bonus's** (0x5873C7, 0x58753C): "heroless" is the record's hero −1 with neither the
  random-hero flag (+0x18), the generating main town (+0xB) nor a main hero (+0x1C); left: −1,
  below 0 to 3, an artifact of a heroless record to 3; right: +1 modulo 4, an artifact of a
  heroless record to 1; then under a shown Random town a resource goes to gold (left) or Random
  (right).

### 23.4 The popups

- **The small popup** (`scenario_info_popup`, `scenario_bonus_popup`): created 200 × 150 at
  (300, 225), its frame 256 × 192 (the frame's own colours, a shadow); the title (medfont colour 1
  align 1) at (10, 26, 236, 36), the picture (48 × 32) centred at y 60, its caption at (10, 95,
  236, 18), the text at (15, 120, 226, 72), smalfont colour 1 align 1. A Random town: 104, the
  picture `hpsrand0.pcx`, 523, 105. The bonus (0x5828A4): artifact 84, frame 9, no caption, 91;
  gold 85, 8, 88, 93; a resource 86, the town's frame, its caption and text by the town's
  resource (690s: Rampart 693/689, Tower 694/690, Inferno and Conflux 695/691, Dungeon 696/692,
  else 90/94); Random 87, 10, no caption, 95. (The artifact's and Random's caption is the
  function's first argument reused, empty in every recording.)
- **The town's** (`scenario_town_popup`): 256 × 320 at (272, 140); 81 at (10, 26, 236, 36), the
  `itpa.def` frame centred at y 60, the town's name at (10, 95, 236, 18), 80 at (10, 127, 236, 36);
  `town_creatures` 0x6747B4's first seven of the faction (`cprsmall.def` frame id + 2, 32 × 32): three
  from x 60 at y 159, their singular names (the creature table's +0x14, tiny colour 1 align 1, 52 ×
  32) at x − 10, y 193; four from x 40 at y 235, names at y 267; 52 apart.
- Each runs modally while the right button is held, the cursor showing; then the screen redrawn
  whole.

### 23.5 The duration's bar

`scrollbar_create` with a bar wider than tall loads `SlideBuH.def` (blue) and `slider.pcx`, its
button the DEF's width (16); drawn as the vertical one on the x axis (the left arrow, the right one
a button from the end, the track between, the thumb at button + travel × value ÷ 10): exact at
value 10 in every shot. Its presses: T3c2, §27.1.

### 23.6 The recordings and the comparisons

`adv-single-town` 27 of 27 exact. `adv-single-hero` and `adv-single-maps`: every difference lies in
the hero's portrait and name and its popups (T3b's): the artifact's, gold's and Random's bonus
popups, the computers' bonus arrows, Free for All's town popup and Rebellion's rows (none, no
popup) exact but for that box. The `scenario` group as recorded; `drawn.txt` recorded for the
`advanced` group.

### 23.7 Not built (T3b on)

The hero's arrows and popups and a main hero's portrait and name (T3b: §24); the flags' press and
hover, the name's edit (353), the handicap's press (taking none), the duration's bar's input and
the items' hints (T3c); the hot seat (T4); the random map (T5); a human-only colour's town drawn at
the list's read (CLT-Q95; no shipped map).

## 24. The advanced options: the hero (M3 T3b1, 2026-11-18)

Read from `Heroes3.exe` (`scenario_hero_lists` 0x583890 again, `scenario_hero_step` 0x58AF20,
`scenario_dialog_click`'s hero arrows 0x586A93 and 0x586B69, `scenario_shown_hero` 0x58D0E0 and
`scenario_shown_hero_name` 0x58D1F0 whole, the row's draw of the hero 0x58DA6E to 0x58DC95,
`scenario_dialog_right_click`'s hero areas 0x5823FF to 0x582614, the constructor's portraits
0x57C431) against `adv-single-hero` and `adv-single-maps`. T3b cut in two (decision 54): T3b1 all
but a hero's own popup (`scenario_hero_popup` 0x575A90, T3b2). The client:
`h3_view::shell::scenario::advanced` (the hero's list, step, show, name and Random's popup),
`Heroes`, `CustomHero`, `Entry::heroes`, `MainHero::portrait` an `Option`; `h3-battle-client`'s
`maps::rules` and `maps::heroes`; the asset tables' `small_portrait`.

### 24.1 The list and the arrows

- **The list** (`scenario_hero_lists`): built for a colour a **human record** holds (a computer
  record has none, +0x28 0) whose shown town is a town: up to 16 hero types in type order that are
  free, of the faction's two classes and, where the map disposes of the type, takeable by the
  colour. It does not test the flags (the arrows do: `scenario_hero_choosable`). The engine's
  `GameSetup::offer` (TWN-SET-006) is the same list where the flags allow a pick; the client takes
  it from each map's import at the shell's start, the colour's choice made each faction in turn
  (`maps::heroes`; the 159 maps import in about 50 ms in release, every one).
- **The step** (231 + *c* left, 239 + *c* right; the human record holding the colour, else
  nothing): +0x20 an index into the list, −1 Random; an empty list changes nothing. Right: −1 to 0,
  on to the last, past it to −1; left: −1 to the last, back to 0, past it to −1. A step that lands on
  a type another human record's index names steps on (the hot seat's: T4); −1 never clashes. Then
  message 0x40C (a network game's relay) and the row redrawn. The client keeps the hero type in
  `Record::hero` (T6's `PlayerStart.pick`).

### 24.2 The hero shown and its name

- **`scenario_shown_hero`** (a portrait index or −1): a new game with neither the generating main
  town (+0xB) nor the random-hero flag (+0x18): the players block's +0x20, **the main hero's
  portrait as the file gives it** (0xFF, its type's own, kept as −1); else the record's pick
  through the map's disposed heroes (game +0x1FB2C, a tree by type: +0x10 a portrait, −1 none),
  else the pick's type. (A load: the main hero's portrait or the first hero, not built.)
- **The row** (0x58DB0F): −1 is Random or None as §23.2; else the portrait of the dialog's array
  (+0xC8: `hero_table` 0x679DD0's +0x30 for the 163 records, 156 types and the campaign heroes'
  seven, then `hpsrand.pcx` at +0x354) at (252, 130 + 50 *r*), its name tiny colour 4 align 5 at
  (240, 162 + 50 *r*, 71, 16).
- **So a main hero the file leaves at its type's portrait shows no hero** (Adventures of Jared
  Haret's blue, type 85: None; tan, type 33 named "Thenith", under a Random town: Random; 52 main
  heroes of the shipped maps): **CLT-Q141** (b), decision 55: the client shows the type's portrait
  and the main hero's name, else the type's; the original's for the comparisons only.
- **`scenario_shown_hero_name`** (new game): with no flag the main hero's type (+0x1C), else the
  pick's; the main hero's own name (+0x24) when the colour has a main hero and the name is not
  empty, **whatever hero shows**; else the disposed hero's name (its string's length not 0), else
  the type's (game +0x21643 + 0x492 × type). With a flag and a named main hero the original names
  every pick by the main hero (no shipped map: each such colour is computer-only, and only a human
  picks): **CLT-Q142** (b), decision 56: the client names a pick by its own name; the original's for
  the comparisons only. The shipped maps' disposed heroes (832) all keep their types' portraits
  and names.

### 24.3 The right press (362 + *c*)

A new game (0x5823FF): the colour's human record, else its computer record; with its hero −1 and
the colour with neither flag nor a main hero: no popup. Then with the hero −1 and none shown
(−1): Random's popup (`scenario_info_popup`: GENRLTXT 102, `hpsrand1.pcx`, 523, 103; the small
popup of §23.4). Else the hero's type (no flag: the main hero's, else the list's) and its popup
(`scenario_hero_popup` with the portrait, the name, `hero_class_name` 0x4D8F70 and
`hero_specialty_text` 0x4D7220: T3b2). So a None over a main hero of its type's portrait gives Random's popup (CLT-Q112's
recording, `map-2-computer-hero-popup`); with CLT-Q141 (b) the client shows that hero, and a None
(no hero at all) gives none.

### 24.4 The recordings and the comparisons

`adv-single-hero` 26 of 28 and `adv-single-maps` 11 of 13 exact: the four shots of a hero's own
popup differ (`hero-popup-1`, `hero-popup-last`, `map-0-hero-popup`, `map-2-hero-popup`: T3b2's).
`adv-begin`'s panel exact; every other group as recorded.

### 24.5 Not built (T3b2 on)

A hero's own popup (T3b2: `scenario_hero_popup` 0x575A90, the class and level, the specialty); a
hero two human records hold (the step's skip, `scenario_heroes_unclash` 0x58BBC0: T4); a portrait
past the hero types' (the campaign heroes' seven, 156 to 162: no table entry, not drawn; no shipped
map gives one); a load's hero (M4).

## 25. The advanced options: a hero's own popup (M3 T3b2, 2026-11-18)

Read from `Heroes3.exe` (`scenario_hero_popup` 0x575A90 whole, `scenario_dialog_right_click`'s
call 0x5825B5 to 0x5825F6, `hero_class_name` 0x4D8F70, `hero_specialty_text` 0x4D7220, the
constructor's `un44.def` 0x57C41E) against `adv-single-hero` and `adv-single-maps`, played again
under Wine this session (every shot as kept). The client: `h3_view::shell::scenario::advanced`'s
`own_hero_popup`, `HeroPopupLook` (`[scenario.advanced.hero_popup]`), `AdvancedTexts`'
`hero_popup`, `hero_line`, `hero_classes`, `specialties`; the catalogue's
`shell.advanced.hero-popup.*` and the family `hero-specialty`, `hero-class` listed by the battle
client too.

### 25.1 The popup (`scenario_hero_popup`, six arguments)

- **The arguments** (the caller, 0x5825B5): the portrait (the dialog's array +0xC8 at
  `scenario_shown_hero`'s index: the small portrait the row shows), the name
  (`scenario_shown_hero_name` of the colour: the row's), `un44.def` (+0x80), the hero type, the
  specialty's text and the class's name of the game's hero record of that type (game +0x21620 +
  0x492 × type). The type: with neither flag the main hero's (+0x1C), else the record's pick.
- **The dialog**: created 300 × 256 at (250, 172), its size rounded up to 64 (320 × 256, as the
  small popup's 200 × 150 is 256 × 192), so the items' width − 60 is 260; then, every text
  colour 1 and alignment 1:
  - GENRLTXT 78 ("{Starting Hero}", highlighted) in `medfont.fnt` at (30, 26, 260, 36);
  - the portrait (a PCX item) centred at y 56: (136, 56), 48 × 32;
  - `"%s - %s"` (`name_dash_format` 0x66033C: the name, then the class) in `smalfont.fnt` at
    (30, 91, 260, 18);
  - GENRLTXT 79 ("{Hero Specialty}") in `medfont.fnt` at (30, 122, 260, 36);
  - `un44.def` (156 frames of 44 × 44 in one group) at the hero type's frame modulo the group's
    count (a DEF of no group divides by 0), centred at y 149: (138, 149);
  - the specialty's text in `smalfont.fnt` at (30, 149 + the DEF's height + 2, 260, 36):
    (30, 195).
- **The texts**: `hero_class_name` gives the class record's +4 (HCTRAITS' first column; in a
  campaign of type 15 hero 27 GENRLTXT 736: never on this screen); `hero_specialty_text` the
  specialty record's +0x1C (`hero_specialty_pointer` 0x679C80, 0x28 bytes a type): HeroSpec.txt's
  first column, the short name ("Archery", "Griffins": recorded).
- **Recorded:** `hero-popup-1` (Orrin, Knight, Archery), `hero-popup-last` and `map-0-hero-popup`
  (picks), `map-2-hero-popup` (Adventures of Jared Haret's red: the main hero "Jared Haret" of
  Edric's type in Orrin's portrait: "Jared Haret - Knight", Griffins): the portrait the shown
  one, the name the row's, the class and the specialty the type's.

### 25.2 The client

The hero's popup is built where the row shows a hero (`Hero::Portrait`: a main hero, CLT-Q141 (b)
included, or a pick); its name is the row's (`hero_name`, CLT-Q142 (b) included), the class's name
by the hero type from the game's rules (`rules.core().hero_types()`' class: the engine's numbers
are the original's), the specialty's by the type from the catalogue (`hero-specialty.<key>`,
HeroSpec.txt's records from 2, column 0). The line is the catalogue's format resolved with
U+0001 and U+0002 for its arguments (`HERO_NAME`, `HERO_CLASS`), filled at the popup
(*provisional*: the screen holds resolved texts, no catalogue). The DEF's frame is the hero type
(every type below its 156 frames). A portrait past the table: no picture (as the row).

### 25.3 The recordings and the comparisons

`adv-single-hero` 28 of 28 and `adv-single-maps` 13 of 13 exact (the four hero popups, T3b1's
leftovers, now exact); every other group as recorded.

### 25.4 Not built

A hero's popup in a load (M4) and in the hot seat (T4: the same function); the campaign's class
text (GENRLTXT 736: no campaign here); a hero type past `un44.def`'s frames (none tabled).

## 26. The advanced options: the flags, the handicap and the name (M3 T3c1, 2026-11-18)

Read from `Heroes3.exe` (`scenario_flag_click` 0x58B790 whole, its call 0x5870E1 to 0x5870F5;
`scenario_name_click` 0x58B620 whole; the hover in `scenario_dialog_proc` 0x5881B2 to 0x588295;
the end of `scenario_player_row_draw` 0x58DEBA to 0x58DF28; the name's edit's creation 0x57B487
and its vtable 0x641C14 against the base edit's 0x642D40: +0x2C 0x57CEE0 and +0x3C 0x57CDC0, both
read whole; the showing's move of the edit 0x58165C; `dialog_set_focus` 0x5FF490; the text item's
press and release `text_item_message` 0x5BC440) against `adv-single-rest`, played again under Wine
this session (every shot as kept). The client: `h3_view::shell::scenario::advanced`'s hover,
`flag_click`, `name_click`, the edit (`EDIT`, `AdvancedLook::edit`, the layout's `edit`), the
screen's flags from the moved colour (`Scenario::team_flags`), the strip and the piece as stale
rectangles in the original's profile.

### 26.1 The flags

- **The hover** (a mouse move reaching the screen's procedure, not a button's own loop): the item
  under the mouse (`dialog_item_id_at` 0x5FF3B0); when it is a flag 263 to 270 other than the one
  hovered (`scenario_hovered` 0x683470), the hovered one's state 0x10 cleared and its row drawn,
  then (not for a network guest) the new one's set and its row drawn, the new one hovered; off the
  flags the hovered one cleared and drawn, none hovered. 0x10 draws the button's selected frame:
  `AOFLGB%c.DEF` has four frames, `n`, `s`, `s` and `h` (3, the white frame: recorded
  `flag-hover-own`, `flag-hover-blue`). The showing does not touch 0x10 (a flag hidden while
  hovered, by the key, keeps it until the next move).
- **The press** (single): the focus taken first (`dialog_set_focus` −1: the name's edit closes,
  §26.2); to a colour a human may play (the players block's +0xA0 byte) that is not the local
  record's: the old flag's 0x10 cleared, the game's +0x1F6D0 bytes of the old and new colour
  rewritten (*not understood*: both stay non-negative, so both still play), the local record's
  colour the new one, its hero and town −1, `scenario_hero_lists`; in every case
  `scenario_team_flags` with its redraw flag, then the caller's `scenario_dialog_redraw` alone (no
  items' draw): the strip restored from its copy keeps the allies' label's colon without its
  shadow (recorded from `flag-own-pressed` on: CLT-Q138's, the client's profile draws whole). The
  allies are the moved colour's team (`flag-blue`); the names follow the record (the redraw sets
  them: `name-after-flag`).

### 26.2 The name's edit

- **The edit** (the constructor, single only): `edit_create` (62, 130 + 50 *c*, the flag's y,
  moved to 130 + 50 *r* at the showing; 97 × 17, room 21, empty, `smalfont.fnt`, colour 1, left,
  id 353 + *c*), its vtable 0x641C14 (the base edit's with its own key and loss of focus), hidden.
- **The click** on a name (a text item: pressed while shown and enabled, clicked at the release
  wherever the mouse is; CLT-Q43 (2)'s rule in the client): `scenario_name_click` (single, not
  network): the focus taken; on the local record's colour only: the colour's bar
  `adop%cpnl.pcx` drawn from the edit's offset in it (5, 2) with the edit's size at (57, 128 +
  50 *r*), the bar's corner, where the edit's place was meant; recorded (`name-edit`,
  `name-typed`): the bar shifted 5 left and 2 up inside (62, 130 + 50 *r*) to (154, 145 + 50 *r*),
  the name box's frame doubled until the row is drawn again (**CLT-Q143**, (b): decision 58); the edit shown and
  enabled with the record's name (its caret at the end), the name hidden and given the same text,
  the focus to the edit (the caret `_` drawn).
- **Its keys** (0x57CDC0, every key down while it shows): scan code 0x1C (Enter, the keypad's too)
  hides it, takes the focus, copies its text into the human record holding its colour and into
  `network_default_name` (`settings_save`), sets the name's text and draws the row; any other key
  to `edit_key` (§16.1: Escape and Tab left to the dialog, Escape then Back's; every other key the
  edit's, no hotkey: S, A and R type). Enter's release clicks nothing (Begin was never pressed).
  **The loss of the focus** (0x57CEE0: a flag's press, a name's click, the options' hiding) does
  the same. No other click takes the focus (an arrow's press leaves the edit open).
- **A row's draw** shows the name (5, 6) and draws it, then the edit: while the edit is open on
  that row (a flag's hover, an arrow) the name shows again under it (empty until the record has a
  name; then the centred name under the edit's left one: CLT-Q143, not recorded).

### 26.3 The client

The edit is `h3_ui::edit` (decision 20): by scan code in the original's profile (Enter closes it,
any key but Escape and Tab taken); in the client's the window's text input types, the edit's own
keys are the key table's (`Context::Edit` first), Begin's command closes it as the original's
Enter, Back's is the screen's and every other key is taken. The name lives in the screen
(`Advanced::name`), not written to a setting (the client has none: M4). The flags' strip of a
flag's press and the edit's piece are stale rectangles in the original's profile (the piece: the
screen, the bar shifted, the edit over it, in its rectangle); the client's profile draws whole and
keeps the name hidden under the open edit (CLT-Q143 (b), decision 58). The handicap stays drawn
up and takes no press; the record's handicap reaches no game (`CLT-STUB-10`).

### 26.4 The recordings and the comparisons

`adv-single-rest`'s first twelve shots (`flag-hover-own` to `name-after-flag`; decision 57 counted
ten) exact, `hidden` too; its other 26 T3c2's; every other group as recorded.

### 26.5 Not built

The hot seat's flags, handicap and names (T4); the name written to `Network Default Name` and read
at the next opening and by the Multiplayer dialog (M4: the settings); the handicap's effect
(`CLT-STUB-10`); Begin pressed by the mouse while the edit is open (the original's path not read; the client's
takes the edit's text since M3 T7, §33.2).

## 27. The advanced options: the duration's bar and the hints (M3 T3c2, 2026-11-18)

Read from `Heroes3.exe` (the horizontal branches of `scrollbar_press` 0x596980,
`scrollbar_release` 0x596AF0, `scrollbar_drag` 0x597020, the procedure's move 0x596621 to
0x5966DD and `scrollbar_draw` 0x596C40; `scenario_duration_scrolled` 0x57C7F0 whole; the bar's
creation 0x57AF06; `dialog_item_at` 0x5FF3E0 and `dialog_right_click` 0x5FF790) against
`adv-single-rest`, played again under Wine this session (§27.3). The client: `h3_ui::scrollbar`'s
horizontal input, `Advanced`'s `bar_press`, `bar_moved`, `bar_release` and `scrolled`, the screen's
`HELPED` and `COLOUR_HELPED`.

### 27.1 The horizontal bar

- **The press, the release, the drag and the draw** test the bar's width against its height
  (+0x1C, +0x1E) and take a horizontal bar's x (the mouse +0x58, the bar +0x18) where a vertical
  one's y: the arrows a button from either end of the length (+0x4C), the release's step by the
  arrow the mouse was last on, the drag's middle at the mouse, the arrows drawn pressed (frames 1
  and 3) as the vertical one's. One difference: the press's thumb is 16 wide (0x5969DE, `add ecx,
  0x10`), the vertical one's its button (0x596A37); the duration's button is 16 (`SlideBuH.def`),
  so no shot shows it.
- **A move while held** (0x596656): a horizontal bar's y taken as the bar's within 40 pixels above
  or below it (a vertical one's x, left or right); over the track (x from the left arrow's end to
  the right arrow's start, y within the bar) the thumb dragged there.
- **The duration's bar** (338, `scrollbar_create` at 0x57AF06): (58, 557, 194, 16), 11 ticks, the
  callback `scenario_duration_scrolled`, blue, **no page** (the layout's `step` 1 of T3a corrected
  to 0: a press in the track drags the thumb), no keys.

### 27.2 The duration and the hints

- **`scenario_duration_scrolled`** (the bar's callback, with the value): the value into +0x378 and
  the game's +0x1F843; a network host sends the setup (message 0x402: not built); then the dialog's
  items drawn (vtable +0x14 with 0, 0xFFFF0001, 0xFFFF) and `scenario_dialog_redraw`: the client's
  whole redraw (the strip a flag's press left stale drawn again). An arrow's release at either end
  reports nothing: the bar alone drawn (`duration-0-again` as `duration-0`). The text:
  `turndur.txt`'s record by the duration (§3.13). No timer runs it in the client (`CLT-STUB-11`,
  decision 6).
- **The hints** (§3.8): items 104 to 344 take `Help.txt`'s record id − 83; for the options the bar
  (338: 255), the headers (339: 256; the town's 342: 259, the one with Conflux; 343: 260; 344:
  261) and the duration's label (340: 257); each colour *c*'s who may play (199 + *c*: 116 + *c*),
  handicap (124 + *c*), arrows (215 + *c* to 262: 132 + *c* to 179) and flag (263 + *c*: 180 +
  *c*). The eight colours' records are one text in both LODs: the client has one key a group, the
  first colour's record (*provisional*, review list item 17). The names (345 on), the edits and the
  areas get none (their popups are the screen's, §3.15).
- **The right press** finds its item by `dialog_item_at` (shown and enabled, not disabled): a
  single player's handicap is not enabled, so the panel under it (102, no hint) is found: no popup,
  the screen redrawn whole (`popup-handicap`, as a name's `popup-name`). A hidden arrow likewise.

### 27.3 The recordings and the comparisons

`popup-hero-arrow` (§6.4) pressed row 0's hero arrow, hidden on a computer's row; on the single
player's row (row 2 after `name-after-flag`) it is hidden too while the row's town is Random (the
hero's arrows need a shown town, §3.14). The recipe now presses that row's town arrow first
(`click:230,245`), then the hero's left arrow (`right:245,245`): its hint shows (record 148).
Recorded again (`run --only adv-single-rest --record`, both runs alike): the seven shots from
`popup-hero-arrow` to `hidden` changed, the others as kept. `adv-single-rest` 39 of 39 exact; every other recipe of
the `advanced` and `scenario` groups as recorded; `drawn.txt` recorded for `adv-single-rest`.

### 27.4 Not built

The hot seat's bar (enabled for every player there, T4); a network host's message (no network
play); the timer (`CLT-STUB-11`: the hot seat's clock, E24 M5).

## 28. The hot seat's advanced options as they open (M3 T4a, 2026-11-18)

Read from `Heroes3.exe` (New Game's Multiplayer, 0x4EED13; the constructor's handicap 0x57B07A to
0x57B115; `scenario_advanced_toggle` 0x58164A to 0x58174A; the row's draw 0x58DE13 to 0x58DF03;
`scenario_list_build` 0x582D41) against `adv-hotseat`, played again under Wine this session
(§28.3). The client: `Scenario::opened_hot_seat`, `Advanced::seated` and its human records, the
layout's `handicap_caption`, the keys `shell.advanced.handicap.*`.

### 28.1 The entry

- **The path** (0x4EED13): New Game's Multiplayer runs `multiplayer_dialog_run`; a result other
  than 0 (the hot seat's OK, `game_type` 3) falls into Single's `scenario_new` 0x4F0490: the same
  constructor with 0, the same run, a back to New Game's menu. Nothing of the Multiplayer dialog
  stays drawn: the constructor draws its background (`gamselb%d.pcx` by its own draw, recorded:
  `adv-hotseat`'s seven openings drew 0, 0, 1, 1, 0, 0, 1; the recipe's `backgrounds`).
- **The slots** (`scenario_slots_new` 0x57F740, §3.12): a human record each of `hot_seat_names`'
  names, in order, ids 1 on; `local_slot` is id 1, so the first name is the local record (its
  colour the flags' strip's, §3.5).
- **The list** (`scenario_list_build`, 0x582D41): outside network play and the hot seat every map
  read is listed; there, only one whose record's +8 is at least 2. The +8 is the "/" figure of the
  list's players column (the colours a human may play: the client's `Entry::humans`, drawn exact
  on the 160 maps since T1b1). The kept map is selected when listed, else the first.
- **The rows** (`scenario_players_init` 0x584C40's hot-seat branch, §3.12): each colour a human
  may play, in colour order, to the next human record without one; past the last name the colours
  are the computer's (their names GENRLTXT 469), past the last such colour a name has none and no
  row (`hotseat-long`, `adv-hotseat` players-2 to players-8: the names on the first rows).

### 28.2 What differs from a single player's

- **The handicap** (0x57B07A): network play or `game_type` 3 makes `adopb2%c.def` a caption
  button (`caption_button_create` at 0x57B0D3: (110, y, 50, 24), the caption `handicap_names`'
  first, `tiny.fnt`, frame 0, click frame 1, flags 2, colour 4; its draw centres it one pixel up,
  §18.1), else a plain DEF button (0x57B110). `scenario_advanced_toggle` disables it single only
  (0x58170E skips the disable and the edit's hide for `game_type` 3): enabled, it takes the mouse
  and its hint (`adv-hotseat-rows` handicap-popup exact). Its caption changes only at a press
  (T4b).
- **The name's edit** (353 + *c*): the constructor's move of it to its row (0x58164A) and the
  row's draw of it (0x58DEE5) are single only; in the hot seat it stays as made, hidden, and a name
  takes no press (`scenario_name_click`: single only).
- **The arrows** (§3.14): the town's as single's (the hot seat, or a free record, or the local
  one's: every colour); the hero's a human record's (every name's colour, not only the local one);
  the bonus's every colour's (0x58DE46: `game_type` 3 shows them whatever the record or the town).
- **The duration's bar** enabled as single's (§3.13).

### 28.3 The recordings and the comparisons

`adv-hotseat` played under Wine (`run --only adv-hotseat`, both runs as kept): its frames
differed from the viewer's only by the background at three openings (the frame's edges, 32 020
pixels); with the recipe's `backgrounds` its 8 shots are exact. `adv-hotseat-rows`' and
`adv-hotseat-two`'s `panel` exact, and with them `handicap-3` (three presses back to the first
caption: the press takes nothing yet), `handicap-popup` and `flag-hover-green`; the presses' other
shots are T4b's. `hotseat-names` and `hotseat-long` now reach the screen: their `scenario` and
`advanced` shots exact. `drawn.txt` recorded for the `advanced` group and those two recipes; every
other recipe as recorded.

### 28.4 Not built

The hot seat's presses (T4b: built, §29). A hot-seat load (M4).

## 29. The hot seat's presses (M3 T4b, 2026-11-18)

Read from `Heroes3.exe` (`hot_seat_colour_cycle` 0x577AE0 whole; `scenario_flag_click`
0x58B790's hot-seat branch 0x58B7F5 to 0x58B829; the handicap's press 0x58706A to 0x5870CA;
`scenario_hero_step` 0x58AF20 and `scenario_heroes_unclash` 0x58BBC0 whole, the showing's own loop
0x5817C1 to 0x581856, the town's arrow 0x586A23 to 0x586A47; `scenario_begin_tail`'s players'
checks 0x58C11F; `scenario_advanced_hide`'s 0x581D5F) against `adv-hotseat-rows`, `adv-hotseat-two` and
`adv-hotseat-begin`, played again under Wine this session (every shot as kept). The client:
`Advanced`'s `cycle`, `pass`, `step_handicap`, `held_elsewhere`, `unclash`, `refusal`, `Refusal`,
the screen's Begin over a map (`Texts::unplaced`, `too_few`).

### 29.1 The flags

- **The hot seat's press** (0x58B7F5): a colour a human may play (the players block's +0xA0)
  goes to `hot_seat_colour_cycle`, then `scenario_hero_lists`; outside network play nothing else
  (the clash's step 0x58B8BC and the host's message are network play's), no highlight cleared;
  then `scenario_team_flags` and the caller's `scenario_dialog_redraw` (§26.1: the strip stale).
- **The cycle** (0x577AE0, on the human records): the holder (the record holding the colour)
  loses it; the taker is searched from the holder's +0x74 + 1, with no holder from the first
  record without a colour, else from 0: the first record with a player from there takes it
  (its old colour kept at +0x1830, its town and hero −1); none past the last. A press on the
  colour pressed last (+0x1824, set by every other press and forgotten at the options' hiding,
  0x581D5F) first returns the holder to the kept colour (when one is kept); the kept colour is
  cleared at every press before the taker's. The record's +0x74 starts at −1
  (`scenario_player_init`); no write of it was found in the screen's code, but every recorded
  press reads it as the record's index (below). The holder keeps its town and hero.
- **Recorded** (each shot exact in the viewer's original profile):
  - `adv-hotseat-rows` (Arrogance: red, blue, purple, teal; A, B, C): teal (no holder, everyone
    placed): A takes it, red the computer's; teal again: A back to red, B takes teal, blue the
    computer's; blue: A takes it (red the computer's); red: A takes it (blue the computer's).
  - `adv-hotseat-two` (red, green; A, B): green (B's, the last): nobody, green the computer's;
    red (A's): B takes it; red again (B's, nothing kept): nobody, both the computer's; green: A,
    the first without a colour.
  - `adv-hotseat-begin` (red, green; A, B, C): red (A's): B takes it (green kept); red again: B
    back to green, C takes red; green (B's): C takes it; red: A, the first without one.
- **The client** (CLT-Q94, decision 13; *provisional*): to the record after the holder, the two
  exchanging colours (a taker without one leaves the holder with none); past the last to the
  computer; with no holder the first record takes it, its own the computer's. Both movers' towns
  and heroes Random.

### 29.2 The handicap, the heroes, Begin

- **The handicap's press** (207 + *c*, 0x58706A; enabled in the hot seat only): the record holding
  the colour (a human's, else the computer's) one on, modulo 3, then the item's caption
  `handicap_names`' (`dialog_item_send` 3, with the item's state bit 2 set meanwhile) and the row
  drawn. Nothing else sets the caption (the row's draw reads no handicap), so a player a flag
  moves keeps its handicap while each row keeps the caption pressed there: **CLT-Q144** (b),
  decision 60: the client's caption is the record's, set at each row's refresh; the original's
  for the comparisons. Recorded (`adv-hotseat-rows` handicap-1 to -3): Mild, Severe, None.
- **The hero's step** (0x58AF20): the same walk as single's (§24.1), and after each step to a
  hero (not to Random) the other colours' human records are compared, each by its pick's type;
  a hero held there steps again. A record without a colour is compared with none.
- **The clash** (`scenario_heroes_unclash` 0x58BBC0, after a town's arrow; the showing's same loop
  0x5817C1): colours 7 down to 0, a human record whose pick another colour's human record also
  holds steps once to the right. A town's arrow clears its own record's hero first and every
  step passes over what is held, so no clash is reachable; built as the original's.
  `row-1-choices` (blue's town, hero and bonus right arrows after the flags: blue the computer's,
  Castle, no hero's arrows, the artifact) and `name-press` (nothing: §28.2) are exact.
- **Begin** (0x58C11F, a new game): the local record (the hot seat's first) without a colour:
  GENRLTXT 531; the hot seat or network play with fewer than two records holding a colour: 83;
  each an OK box (as 530's, §21.3). Recorded: `adv-hotseat-two` begin-1 (83), `adv-hotseat-begin`
  begin-1 (531). The client's 531 refuses while any record lacks a colour (CLT-Q109 (b), decision
  22); with more names than the map's colours a human may play (listed: §28.1) one name is
  always without, so such a game is never begun in the client's profile (**CLT-Q145** (c),
  decision 61). Before the options are first shown the check reads the colours their showing
  will give (`Advanced::seating`).
- **The box's close** (`message_box`'s own restore): only the box's rectangle is drawn back, so
  a flags' strip a press left stale stays stale (`closed-1` of both recipes: the allies' colon
  without its shadow, 2 pixels); the client's screen is not drawn again whole at the box's close
  (T2b's was: no stale part showed there).

### 29.3 The recordings and the comparisons

`adv-hotseat-rows` 12 of 12, `adv-hotseat-two` 7 of 7 exact; `adv-hotseat-begin` 7 of 7 up to its
begin (T6's); every other recipe as recorded (`drawn.txt` recorded for the three).

### 29.4 Not built

The begin itself (T6); a hot-seat load's presses (M4, CLT-Q110 (b)); network play's clash step
and messages (no network play).

## 30. The random map's tab drawn and pressed (M3 T5a, 2026-11-19)

T5 cut in two with the user at its start (decision 62): T5a the tab drawn and its left presses,
T5b the right press and hints, Show Random Maps, the tab toggled back and Begin. Read from
`Heroes3.exe` (`random_map_toggle` 0x57FEB0, `random_map_items_create` 0x57D170's arguments,
`random_map_tab_draw` 0x57EF70 and `random_map_info_build` 0x580430 whole; `scenario_dialog_click`
0x5865B0's cases 130 and 279 to 335; `random_map_hide` 0x581D80's ends; `scenario_info_draw`
0x5840F0 and `scenario_entry_name` 0x584010 for what the panel reads; `scenario_dialog_redraw`'s
tab part 0x584B24; `scenario_list_toggle` 0x580A70's and `scenario_advanced_toggle` 0x581100's
calls) against `adv-random`, played again under Wine this session (52 of 52 as kept). The client:
`scenario::random` (`RandomMap`, `Choices`), the screen's `toggle_random`, `random_built`,
`random_click`, `shown_entry`; the layout's items 103, 280 to 335, 1040 and 1041; the keys
`shell.scenario.random-*`.

### 30.1 The choices and the presses

- **The constructor's choices** (0x57D192 to 0x57D1D0; each opening of the screen starts from
  them): size 72, two levels, the players Random (−1), their teams Random, the computer only
  players **0**, their teams Random, the water Random (3), the monsters Random (−1).
- **The presses** (`scenario_dialog_click`): 281 to 284 the size (36, 72, 108, 144), 285 the
  levels (3 − levels), 287 to 294 the players (id − 286), 295 Random, 297 to 304 their teams
  (id − 297), 305 Random, 307 to 314 the computer only players (id − 307), 315 Random, 317 to 323
  their teams (id − 317), 324 Random, 326 to 329 the water (id − 326: none, normal, islands,
  Random 3), 331 to 333 the monsters (id − 331), 334 Random; each `random_map_tab_draw`, the items
  drawn again and `scenario_dialog_redraw`. The players' (287 to 295), their teams' counts (297 to
  304, **not** Random's 305) and the computer only players' (307 to 315) then build the record
  again (`random_map_info_build`); the others do not. 335 is `scenario_list_toggle(1)` (T5b), 279
  Begin's generation (T5b, decision 13). A disabled button takes no press.
- **The tab's state** (`random_map_tab_draw`, only while +0x37E): each group's chosen button
  selected (0x10), the others not; the levels' switch selected for two levels. The caps: the
  players' teams 1 and up enabled below the players' count, disabled from it on (Random players as
  8: 1 to 7 enabled); the computer only players 1 and up enabled up to 8 less the players (Random
  as 1: up to 7); their teams 1 and up enabled below the computer only players shown (their choice
  up to its cap, Random as the cap). A choice above its cap is kept but shows its cap's button
  selected (0x57F0C6, 0x57F153, 0x57F201: a disabled button then shows selected; no recording has
  one, the client copies it). Group 0 (a count of 0) is never disabled, Random never.
- **The items** (§3.16's table; the constructor's order, after the advanced options' items and
  before Show Available Scenarios): the counts' buttons, Random's, the water's and the monsters'
  have the disabled frame 1 and the selected frame 2 (their +0x3C and +0x40, 0x57D73E and the
  loops after each group); the sizes, the levels' switch and Show Random Maps the defaults (2, 3).
  The labels are text items of colour 1 with the flags 8 (as the panel's labels), 280 centred, the
  others left. `scenario_dialog_redraw` draws 739 and 740 after the panel and calls
  `random_map_tab_draw` last (its states show at the next draw of the items: every caller draws
  them after a press).

### 30.2 The own record and the panel

- **`random_map_info_build`** (from the toggle's first opening, +0x37F clear, and the counts'
  presses): a header of the version's format (the single player's SoD: 0x1C), the map's difficulty
  1, the size and the levels chosen, no special victory or loss (0xFF), no teams (each colour its
  own), a human may play the colours below the players (8 for Random), anyone those below the
  players and the computer only players together (8 when either is Random), every faction, a
  random town (+0xA), +0xB set (*inferred*: a main town), no main hero (+0x1C −1); its name GENRLTXT 741 and
  description 742; the header into the game (`map_file_open_header` 0x513320, the players block
  and the hero owners copied), the description into the scrollable text, every record's town and
  hero cleared, `scenario_players_init`, `scenario_hero_lists`, `scenario_team_flags`, Begin
  enabled, the items and the redraw. The client builds an `Entry` of it (`RandomMap::build`) and
  makes the advanced options' records again at their next showing.
- **What the panel shows** (`scenario_info_draw`): the name by `scenario_entry_name` (741 while
  +0x37F), the rest from the game's header: the map's difficulty (Normal), the standard victory and
  loss with their marks (11, 3), the rating, the flags (red the single player's, every other
  colour that plays an enemy). `scenario_to_game` gives the game the own record while +0x37F
  (the advanced toggle's and the tab's toggle's call), so the advanced options over a chosen random
  map show its seats (the client's `shown_entry`).
- **The size mark** is set by the select alone: in the tab it keeps what it showed (`adv-random`:
  S, Arrogance's, whatever the size). The client's profile shows the chosen size's (CLT-Q115 (b),
  decision 22); a size's press builds no record, so the client sets the mark from the choice.
- **The toggles**: the tab's (130) shown hides it alone (+0x37E cleared, a random map still
  chosen); else the list (`scenario_list_hide`) and the options hidden, 103 and 280 to 335 shown,
  `scenario_to_game`, +0x37E set, and with +0x37F clear the record built (the client's profile at
  each opening: CLT-Q111 (b)). The list's toggle hides the tab and clears +0x37F before it shows the
  list (0x580A96, 0x580AAA); the advanced toggle hides the tab (0x58136F), +0x37F kept.
- **The teams' 1 and 2** of the human or computer players are 30 × 24 items (0x57D85B, 0x57D8A9)
  over 30 × 32 frames: their lowest 8 rows take no press (**CLT-Q146**, open; built as the
  original's).
- *The client's mouse* (*provisional*): the tab's hidden items left out of the shared input's
  dialog (`mouse_dialog`), as the advanced options' (T3a): they lie over the list's columns and
  over the options' rows.

### 30.3 The recordings and the comparisons

`adv-random` from `tab` to `monsters-random` (44 shots) exact, and `show-hover`; the other seven
(the popups, `random-maps`, `tab-again`, `tab-hidden`) are T5b's (`drawn.txt` recorded for the
recipe). Every other recipe as recorded.

### 30.4 Not built (T5b)

The tab's right press and hints (`Help.txt`); Show Random Maps (the random maps' list, kind 1,
empty: CLT-Q92); the tab toggled back after it (the record built again with an empty list:
CLT-Q111); Begin with a random map chosen (a stub's message, decision 13; `adv-random` gains a
Begin after `tab-again`, recorded with `--record`); the list's showing over a chosen random map
(read only: +0x37F cleared and no select, so the original's panel keeps the random record's header
and description but the name; not recorded: to record at T5b and ask if it shows).
*Done at T5b* (§31).

## 31. The random map's tab: the right press, Show Random Maps, Begin (M3 T5b, 2026-11-19)

T5b kept whole with the user (decision 62's second half). Read from `Heroes3.exe`:
`scenario_list_toggle` 0x580A70's kind path, `scenario_list_sort`'s call of `scenario_to_game`,
`scenario_to_game` 0x583580's and `scenario_info_draw` 0x5840F0's empty-list tests, `scenario_select`
0x585500's select of none, `scenario_begin` 0x58BCE0's random map path, `scenario_dialog_click`'s
cases 128 and 335, `scenario_dialog_right_click`'s dispatch and `dialog_proc_default` 0x5FF820. Played
under Wine and recorded: `adv-random` with Begin added (53 shots) and `adv-random-list` (new, 12
shots), each twice, the same. The client: the list's kind, the tab's popups, the panel over an
empty list, Begin's stub (`CLT-STUB-12`), CLT-Q146 (b) and CLT-Q147 (b) (decisions 63, 64).

### 31.1 The right press and the hints

- Items 280 to 335 take `Help.txt`'s records 197 to 252 (id − 83, §3.8): the labels, the sizes, the
  levels, each count, the water's, the monsters', Show Random Maps; one record each (the client's
  `random::HELPED`, the keys `shell.scenario.help.random-*`). `scenario_dialog_right_click` passes
  them to `dialog_right_click` (its cases are 362 to 387 alone): the hint's popup, then the screen
  redrawn whole. No status line (§3.8): the hints are the popups.
- **A disabled count** (0x1000, above its cap) shows its popup too (`adv-random-list`
  popup-disabled: "Create 1 human team" with one player): the item's own right press sends the
  message (`dialog_proc_default` 0x5FF820 takes the id from it); `dialog_item_at` 0x5FF3E0, which
  passes over 0x1000, is not on this path. A choice above its cap shows its cap's button selected
  though disabled (`players-1`: teams 1; §30.1, now recorded).

### 31.2 Show Random Maps and the list's kind

- **335** is `scenario_list_toggle(1)`, then the items drawn and `scenario_dialog_redraw`; 128 the
  same with 0 (0x586D3A). A shown list hides first (the kind kept); else the tab and the options
  hidden, +0x37F cleared, and **a change of the kind** (+0x66) sets it, builds the random maps' list
  (+0x1040) while it is empty (`random_maps\*.h3m`: the harness's run has no folder, so none) and
  sorts the new kind's list by key 2 (`scenario_list_sort` 0x580B38): the top and the selection 0,
  `scenario_to_game` of its first row (nothing over an empty list), the whole redraw. The same kind
  sorts nothing and selects nothing.
- **Over an empty shown list** `scenario_to_game` does nothing (0x583593: before its +0x37F test, so
  a chosen random map's record neither) and `scenario_info_draw` draws nothing (0x584128, a random
  map chosen or not): no name, conditions, marks, map's difficulty nor rating. The description and
  the flags keep what they showed; the size mark too. Only `scenario_select` of none (−1, a
  filter's) empties the description (0x585639, `empty_text`).
- **The tab again** after Show Random Maps: the list hidden (its kind stays 1), `scenario_to_game`
  (nothing), +0x37F clear, so the record is built again (CLT-Q111's recording read anew: the build
  sets the description and the flags, the panel draws nothing over the empty list). `random-maps`,
  `tab-again`, `tab-hidden` (adv-random) and `random-maps`, `list-hidden` (adv-random-list): the
  panel blank, "Play a randomly created map" and the random map's flags.
- **The files' list back** (128 twice after 335): the kind 0 again, sorted by the version, its first
  row selected and given to the game (its description), the whole redraw; no select, so **the flags
  stay the random map's** (`list-back`: Arrogance with red alone; CLT-Q147).
- **The list over a chosen random map** (128 from the tab, the kind unchanged): +0x37F cleared, no
  sort, no select, no `scenario_to_game`: the panel's name is the selection's (`scenario_entry_name`)
  over the game's header, the random map's, its description and flags (`list`: "Arrogance" with
  "Play a randomly created map" and red alone; the header's texts happen to equal Arrogance's).
  **CLT-Q147** asked and answered (b): the client's showing gives the game the selection (its
  description, conditions and flags), the original's for the comparisons (decision 64).

### 31.3 Begin with a random map chosen

`scenario_begin` tests +0x37F first (0x58BD00): `random_map_file_name`, then `random_map_generate`
whatever the list; a failure ends Begin. Recorded (`adv-random` begin, after `tab-hidden`): "Cannot
create Random_Map.H3M in data directory." in an OK box (the harness's run has no `random_maps`
folder). The client shows its stub's notice in its own words (`shell.stub.random-map`, decision
13, `CLT-STUB-12`); its OK closes it, the random map still chosen. A replay stops under it
(`Stop::Stub`), as under the network modes' notice.

### 31.4 CLT-Q146 and the client

- The teams' 1 and 2 (298, 299) are 30 × 24 over frames whose buttons are drawn to their 32nd row
  (`RanNum1`, `RanNum2.def`: every frame opaque 30 × 32, like `RanNum0` and `RanNum3`).
  `adv-random-list` teams-1-low: a press on their lowest rows does nothing. **CLT-Q146 (b)**
  (decision 63): the client's profile makes them 32 high (`RandomMap::heights`); the original's
  for the comparisons.
- **The client** (`h3_view::shell::scenario`): `random_list` (+0x66: the client's own random maps,
  none), `toggle_list(random)`, `relist` and `sort_list` by the kind, `give_to_game` nothing over an
  empty list, `select` of none emptying the description, `show_with` (the header and the flags
  apart: the original's list showing sets no flags and keeps a random map's header),
  `blank_panel`; `random_built` sets the description and the flags itself; `stubbed`. The client's
  profile over an empty list shows nothing of a map (no description either, as decision 51's
  flags), over the random maps' list with a random map chosen its record (CLT-Q111 (b)).

### 31.5 The recordings and the comparisons

`adv-random` 52 of 53 drawn, all exact, the replay stopping at `begin` under the stub (the
original's message is its generator's); `adv-random-list` 12 of 12 exact. Every other recipe of
every group as recorded.

## 32. The game's begin (M3 T6, 2026-11-19)

Read with `scripts/exe.py` (the begin's path from §3.17 on) and recorded under Wine (`adv-begin`,
`adv-hotseat-begin`).

### 32.1 The loading screen and its bar

- `loading_screen_open` 0x4ED350 (read whole): with no `loadbar.pcx` loaded (0x69956C) it loads
  it and `loadprog.def` (0x699570) and sets the count (0x699574) 0; then, loaded or not, copies the
  picture whole to (0, 0), draws the pieces so far (frame *i* of the DEF's group 0 by
  `def_draw_plain` 0x47BCF0 at (395 + 18 *i*, 548)) and shows the screen whole. Called by Begin's
  tail (0x58C255, after the busy cursor `mouse_set_cursor(1, 1)` and a network game's message),
  the tutorial's start, a network join, one more and the adventure manager's start (0x407517:
  the screen drawn again with the count kept).
- `load_progress_step` 0x4ED2A0 (read whole): nothing without the DEF; the count one more, at
  most 20 (0x14), the pieces drawn again; with `cl` set the bar's rectangle (395, 548, 358 × 16)
  shown (`screen_show_rect` 0x602BD0, which runs the tick). On the begin's path: `new_game_start`
  4 times (0x58C606, 0x58C64C, 0x58C6D5, 0x58C93F), the map's load (`map_objects_place_all`
  0x4FD6E3, `map_objects_check` 0x504470 six times, two more of its functions seven times) and the
  adventure manager's start nine times: more than 20, so the bar is full before play. Play frees
  both and sets the count 0 (`game_main` 0x4EFDBA to 0x4EFDEB, after the manager is added), so
  every begin starts empty; a campaign's scenario sets the count 20 at once (0x4EFCB3, E20).
- The load runs in real time and reads no clock: the harness sees the bar only full.

### 32.2 Play's fade

The adventure manager's start, after the loading screen again (0x407517), calls
`window_manager_state` with (1, 4, 1) (0x407526; `edi` 1 since 0x4074C3): the fade out (§10:
the screen copied, shown at shifts 0, 1, 2 of each channel, each step 50 ms of the clock after the
last began, then black; the mouse hidden first), then the map drawn and faded in (0x6032E0, not
read). On the harness's frozen clock: Begin's release at *t*, the load and the shift 0 at *t*, the
input's step to *t* + 100 (shift 1), *t* + 150 (shift 2), *t* + 200 the black and the map's first
step at once, the map whole at *t* + 300. `adv-begin` (*t* 1 000) and `adv-hotseat-begin` (*t* 2
300, the turn's dialog over the covered view) show the same two faded loading screens (the bar
full, no cursor).

### 32.3 The recordings

Both recipes gained `clip:bar:90` after Begin's `click-wait` (the two faded frames: 1 100 and 1
150, 2 400 and 2 450), their `load` clips starting 90 ms on; recorded again twice each, every other
shot as kept (a single run had once shown the map 10 ms late: the fade's last step reads the clock
after the load's real time).

### 32.4 The client

- **The begin** (`h3_view::shell::scenario`): Begin, Enter or a row's double press over a map
  passing Begin's checks ends the screen (`Ended::Scenario(true)`); `Scenario::begun` hands on the
  map's file, the difficulty, each colour's record (as the rows show them, or as the next showing
  would make them: the human records seated, Random towns and heroes, `Advanced::begun`), the
  human records' names by colour and the duration.
- **The setup** (`h3-battle-client`'s `begin`, decision 4): the map read and imported again
  (`h3_map::import`), each player's `human` the records' (a colour a human record holds), a
  record's town `FactionChoice::Fixed` (its Random the import's choice: CLT-Q95), a human's pick by
  its hero type, the difficulty; `GameSetup::validate`. What it has no field for (names, bonuses,
  handicaps, the duration) kept beside it (`begin::Game`) for E24 M5 (the map-client lane's inbox).
  The two recipes' choices validate (`the_recipes_begins_validate`).
- **Play's start** (`h3_view::shell::play`, `Front::Play`; `Screen::Play { restart: false }` built
  whole): the loading screen (the layout's `[loading]`), the bar stepped by the client's work (the
  map read, the setup made, validated: `begin::STEPS`), then filled and the fade begun; the fade's
  shift drawn by the adapter (`start::faded`, shared with the start's fade); at the black the
  client's notice over the main menu (`shell.stub.play`, `CLT-STUB-13`), or the engine's refusal
  (`shell.stub.play-refused`, the error on standard error); its OK the main menu. The replay loads
  at once and stops under the notice (`Stop::Stub`). No busy cursor on the loading screen
  (*provisional*: no recording shows it before the fade).

## 33. M3's review (M3 T7, 2026-11-19)

Three fresh reviewers (the screen and its list, T1a to T2c; the advanced options, T3a to T4b; the
random map's tab and the begin, T5a to T6), each over its code, readings and recipes, the shots as
contact sheets; their findings, the user's answers (decisions 67, 68; CLT-Q148 to CLT-Q154) and
what T7 built. The review list's new wording: `E27-archive.md` (M3).

### 33.1 Read at T7

- **A key under a held button** (`def_button_message`'s own loop, 0x456341 to 0x45641C): the loop
  takes every input message (the mouse manager's +8) and acts on a move (type 4) and the button's
  release (0x10, 0x40) alone, so a key pressed while the mouse holds a DEF button is dropped. The
  list's keys (`Command::List`) now do nothing while the shell's input holds a button, in both
  profiles (they bypassed `h3_ui::input`).
- **The selection's change** (`scenario_select` 0x585500, at 0x585542 to 0x585574): +0x1854 is
  set when the new index differs from +0x374 or in the save mode; the map behind the index is not
  compared. So a sort or a filter that leaves the selection on row 0, or the files' list shown
  again after the random map's tab, keeps the advanced options' rows made for the old map at their
  next showing (CLT-Q152): the original's profile keeps the index compare; the client's makes the
  rows again whenever the map given to the game changes, by its file (`Scenario::options_map`,
  `sync_options`).

### 33.2 Built at T7 (decision 67)

- **A held row or name under a box** (both profiles): the release that ends a hold made while a
  box, popup or notice opened (Enter's refusal, a right press's help) goes to that box; the row
  or name is let go without its click (the original's modal loops take that release).
- **The double press** (the client's profile; CLT-Q91 (c) made "the same map"): the last press
  keeps its map's file, a second press within 400 ms begins only that map, selecting it first
  (a key, a sort or a bar move between the presses no longer begins the map the selection moved
  to). The original's: any second press within 400 ms begins the selection.
- **The open name's edit at Begin** (the client's profile): `Advanced::begun` takes the edit's text
  first, as Enter does (the original's path not read, §26.5).
- **CLT-Q93 (b)** (decision 13, built at last): on a map with colours only a human may play, the
  single player starts on the first of them and only the others become the computer's; it may
  return to its start colour. The original's (that colour and every later one the computer's) for
  the comparisons; a colour no human may play stays as the map has it (*inferred*). No shipped map
  has such a colour.
- **The random map's seats** (`RandomMap::made`): the generate flag (+0xB) set, as §30.2 reads it
  (*inferred*: a main town), so a row over a chosen random map shows Random for the hero and its
  arrows once a town is chosen; each colour offered each faction's free heroes (`scenario_hero_lists`
  0x583890 over the game the build resets: the faction's first 16 hero types of its two classes,
  in type order; `maps::random_offers`, `Scenario::with_random_offers`); no main hero
  *provisional*. Recorded by `adv-random-options` (new: the tab, the options, the hero's popup and
  arrows, a town, the files' list and the options over it): 9 of 9 exact, the empty offer T5a had
  left *provisional* refuted (the arrow steps to Orrin), and the original's rows kept over the
  map shown after the random one (CLT-Q152's oddity, as read).
- **The begin's guard** (`h3-battle-client`'s `begin::setup`): a colour is human when a human
  record holds it and the import lets a human play it; a human record on a colour the import gives
  the computer alone is refused (play's refusal notice; the detail on standard error).
- **The window's typed `begin`** is refused on the scenario screen (a game begins by Begin alone).
- **The names' fold** (`caseless`): ASCII letters alone, as `kept_row` (*provisional*: the
  original's compare not read).

### 33.3 The oddities (decision 68)

- **CLT-Q148 (b):** a select that keeps the map keeps the description's scroll in the client's
  profile; the original's resets it at every select (`scenario_to_game`).
- **CLT-Q149 (b):** Up or Down with the selection off the page first moves the page to the
  selection, then steps, in the client's profile. The original's (§19.3): Down steps the selection and moves the top a row, the selection staying off
  the page; Up steps it and brings the top to it only when it is above the page.
- **CLT-Q150 (b):** a row takes the press from x 26 (the players' column, the layout's
  `[scenario.list].players`) in the client's profile, from x 57 in the original's.
- **CLT-Q151 (b):** the list's entries keep a map's name and description whole; the original's
  profile cuts them to the record's 60 and 300 bytes where the screen sorts, lists, shows and
  describes them (`NAME_BYTES`, `DESCRIPTION_BYTES`).
- **CLT-Q152 (b):** §33.1.
- **CLT-Q153 (a):** the bonus kept across a map change and a flag's press, as the original's; the
  adventure lane's inbox told for E10.
- **CLT-Q154 (a):** the random map's count above its cap shown selected and disabled, as the
  original's.

### 33.4 Not built, not recorded (M4's)

A file `h3_map::read` refuses is left out of the list (the original lists one of an unknown format
with GENRLTXT 431 and 506, Begin disabled, and a map with no players with 507; §3.3); no shipped
file reaches it. A held list key's repeats (the window takes no key repeats, §16.4) are not read
or recorded: M4's load mode's list records them. No recipe drags the list's thumb (the scroll
bar's own tests cover it).

## 34. High Scores (M4 T1a, 2026-11-19)

Read with `scripts/exe.py` (§2.8's functions again, for what the client needs) and recorded under
Wine (`scores-tabs` played again, both runs as kept). Decision 70: T1 cut in two, T1a the dialog
drawn and pressed, T1b Reset's question and the client's scores file.

### 34.1 The reading

- **The creatures' templates:** `game_init` 0x4ED650 calls 0x505DA0 on the game's map (game
  +0x1FB70, at 0x4ED98C): `objects.txt` read (0x514D80), each template pushed onto its class's
  list (map +0xDC + 16 × class; the class the template's +0x1C) in the file's order.
  `class_template_last` 0x505EA0 then finds the last template of class 54 (0x36) whose subclass is
  the creature; its +4, the DEF's name (0x63A608, an empty name, when null). At the start
  `objects.txt`'s 141 templates of class 54 give every creature of the table one (one each). After
  a played map: not read (§6.4's lead; M5's won screen reaches High Scores from play and records
  it).
- **`high_score_creature`** 0x4E9830, inlined at the dialog's constructor and at Reset: a
  campaign's score divided by 5 (0x66666667, rounded toward 0), then the first pair whose score is
  not below it; a score above the last pair's 32 767 runs the loop past the table (0x4E9859, no
  bound): CLT-Q157 (a), the client takes the last pair (13).
- **The defaults** (`high_score_defaults_load` 0x5B91D0, `high_scores_reset` 0x4E8FB0):
  `CampHigh.txt`'s records 1 to 11 and 13 to 23, fields 1 to 4; the player and the land copied by
  `strncpy` (41 bytes, so a 41-byte name keeps no terminator: none of the defaults; T1b's file and
  M5's entry), the days and the score by `atoi`.
- **The draw's order:** `high_scores_show` 0x4E9110 builds the dialog, calls `high_scores_draw`
  (the tab's picture onto the screen's buffer, then the headings and rows), then `modal_loop`
  shows the dialog (its items over them). A tab's switch and a step call `high_scores_draw` and
  the dialog's +0x14 (1, 0xFFFF0001, 0xFFFF): every item drawn, the screen shown whole.
- **The texts:** `text_draw`(`medium_font`, the text, the screen, *x*, *y*, *w*, 26, the colour, 5,
  −1); the rank `"%d"` of *i* + 1; the scenarios' days (+0x58) at 531 (52) and score (+0x54) at 601
  (52), the campaigns' score at 531 (122, the same box as their heading).
- **A creature's item** (`dialog_def_create` 0x4EA720 with (649, 26 + 50 *i*, 64, 64, the id, the
  name, 0, 0, 0, 0, 0x10)): drawn by its DEF's type (0x4EAE6C, the table 0x4EAFC8): a map
  object's (0x43) by `def_draw_flagged_plain` 0x47BDC0 (group 0's frame, colour 0: runs of 5
  skipped), the same pixels as the client's plain drawer of encoding 3 (`h3_render::blit`).
- **The timer** (the procedure's last branch, 0x4EA570): any message but a key down (type 1) and a
  command (0x200); +0x104 set by the constructor (0x4E98E2), the 22 counters (+0xA4) zeroed
  (0x4E9AFD); a counter wraps at its DEF's group 0 count.
- **The buttons:** `def_button_create` (*x*, *y*, 41, 199, the id, the DEF, 0, 1, 0, the key, 2):
  Exit 0x1C, Reset none, Campaign 0x2E, Standard 0x1F, in that order; their DEFs 42 × 200 (four
  frames: `*n`, `*s`, `*d`, `*h`).

### 34.2 The recordings

`scores-tabs` played again (both runs as kept). The viewer's replay against its 28 shots found:

- **A press shows the button's box alone:** `button_press` 0x456860 draws the pressed frame and
  shows the item's rectangle (41 × 199) of the 42 × 200 DEF, so its last column and row keep the
  frame last shown whole (`standard-press`, `campaign-press`, `reset-press`, `exit-press`: 226 to
  241 pixels). As CLT-Q118's menus (decision 22): the client draws the frame whole, the original's
  profile keeps the edges (`Scores::stale`).
- **No step while the mouse holds a button** (`reset-press`: its own loop takes the messages,
  `def_button_message` 0x45641C), and **the release that ends the hold runs the step due at its
  own time** (`exit-left` to `campaigns-anim`: the step at the release's 4 600, then 4 900 and
  5 200, and the clip's 16 frames from 5 410 by 210; the message that runs it is not read).

### 34.3 The client

- **`h3_view::shell::scores`** (`Front::Scores`; `Screen::HighScores` built whole): the layout's
  `[high_scores]` (the pairs, the columns by tab, the dialog's four buttons and 22 creature items,
  `drawn` the buttons' canvas); the tables (`Tables::defaults` from `CampHigh.txt`'s records,
  `atoi`); the creatures' pictures by creature (`Pictures`: the adapter's); the tab, the counters
  and the timer (`Scores::clock`), the hold's rules, the original's stale edges; Exit's keys
  (Enter and Escape, `scores-exit`) end it at their press, C and S press their tab's button (the
  shell's key table, `Context::HighScores`); no popup, no hover. Reset shows the client's notice
  (`shell.stub.scores-reset`, CLT-STUB-14) until T1b (lifted: §35).
- **The adapter** (`h3-battle-client`'s shell): `CampHigh.txt` and `OBJECTS.TXT` from the LODs;
  the DEFs of the creatures the tables' rows show loaded into the start's batch, each with its
  group 0's frame count; the headings through the key catalogue (`shell.high-scores.*`, GENRLTXT
  434 to 437, 673, 76); the window steps the creatures on its clock.

## 35. High Scores' Reset (M4 T1b, 2026-11-19)

Read with `scripts/exe.py` (`high_scores_proc`'s Reset branch, 0x4EA39C to 0x4EA55F, and the two
DEF item setters) and recorded under Wine (`scores-reset` played once again, as kept).

### 35.1 The reading

- **The question:** `message_box` 0x4F6570 with GENRLTXT 667 (`general_text`'s +0x20, entry
  0xA6C / 4), style 2, no picture; its own modal loop, so the dialog's procedure gets no message
  (no step) while it shows. The answer is the window manager's +0x38: anything but 0x7805 (the
  box's OK) returns at once, nothing changed, nothing redrawn.
- **OK:** `high_scores_reset` 0x4E8FB0 (both tables from the defaults, §34.1); the file
  `.\DATA\HiScore.dat` (`"%s%s"` of 0x677D88 and `high_score_file_name`) opened by 0x619B7D
  (flags 0x8301, mode 0x80: created or truncated, binary, written), the manager's 0x898 bytes
  from +0x38 written (0x62017F) and closed (0x619A9D); a failure to open shows `file_error_box`
  0x4F3A60 (GENRLTXT 11 with the name) and goes on. Then for each row *i* (0 to 10): the
  scenarios' creature item (dialog +0x78 + 4 *i*) given the DEF of its row's score
  (`dialog_def_set_def` 0x4EB030: the old DEF released, the new one loaded) and frame 0
  (`dialog_def_set_frame` 0x4EAFF0: the frame modulo the item's group's count; a DEF without
  that group divides by 0), then the campaigns' (+0x4C + 4 *i*, the score divided by 5). The
  counters (+0xA4) are not touched; the procedure redraws nothing.
- **The step after it:** the next step sets each counter's next frame (§34.1's timer), so a
  creature shows frame 0 from the OK until then and then its counter's next frame:
  **CLT-Q158** (an oddity, built as the original's).

### 35.2 The recordings

`scores-reset` (10 shots) played once again under Wine, as kept. The viewer's replay found:

- **The release that closes the box runs the step due at its own time**, as the release that
  ends a button's hold (§34.2): Cancel's release at 400 stepped (opened at 0), so Reset's OK at
  600 (200 ms later) did not and the tick at 700 did (`reset`: the frames two on from the
  opening's, not one); then the campaigns' tab at 800 shows theirs at 0. Read as the box's loop
  handing the release's follow-up message to the dialog's procedure (*inferred*).
- The box (GENRLTXT 667, centred, style 2) and the pointer on its buttons (`ok-hover`,
  `cancel-hover`) exact: the harness's `point:ok` is the middle of its probes' 66 × 33 from the
  button's frame (`Box64x30.pcx`, a pixel up and left of the button), `point:cancel` of 66 × 32.

### 35.3 The client

- **`h3_view::shell::scores`:** Reset's question (`question`, style 2, `shell.high-scores.reset`);
  Cancel keeps the tables; OK sets the defaults again (`Scores::new`'s, `CampHigh.txt`'s), every
  creature's DEF and frame 0 (counters kept), and the scores file due (`Scores::take_save`); the
  close runs the step due. The tables outlive the dialog (`Scores::keep`, from `Menus::front`
  as the main menu opens after it) and a scores file's are given at the start
  (`Scores::with_tables`).
- **The scores file** (*provisional*, the agent's: M4's review list): `Tables::read` and
  `Tables::write`, TOML: `version = 1` (`FILE_VERSION`), then `[[scenarios]]` and
  `[[campaigns]]`, each 11 rows of `player`, `land` (a campaign's name in the campaigns'), `days`
  and `score`. Another version, another count of rows, a missing or unknown field or a number out
  of 32 bits is refused.
- **The adapter:** the window's file (`--scores <file>`, else `scores.toml` in the working
  directory, beside `settings.toml` and `keys.toml`) read at the start (none there: the
  defaults; refused: the window does not start, the path and the reason told), the creatures'
  DEFs of its rows loaded beside the defaults'; written whole at the OK (a sibling renamed over
  it, as the settings file), a failure told on standard error (not the original's
  `file_error_box`). The replay starts from the defaults and writes nothing.
- **The replay** (`h3_view::shell::replay`): `press`, `point` and `until-button` on a front's
  original box (`Front::question`: High Scores' alone so far) at the probes' point
  (`replay::button_point`); `Stop::NoButton` when it does not show.


## 36. The credits (M4 T2, 2026-11-19)

Read with `scripts/exe.py` (`credits_screen` 0x4EDDA0 again, its copies' routine `pcx16_copy_rect`
0x44E2B0 whole) and recorded under Wine (`credits-scroll` and `credits-end` played once again, as
kept; every frame of both compared with a model of the reading below: all equal).

### 36.1 The reading

- **The pictures:** `Credits.fnt` (31 rows a line); `text_line_count` 0x4B5820 of record 1 at 328
  px: 153 lines, so *H* = 4 743 rows; the background a 16-bit picture of 328 × 580
  (`sub_44DF70`), the text's one of 328 × (*H* + 31) filled with the 16-bit colour 1
  (`pcx16_fill_rect` 0x44E4C0) and the text drawn into it at (0, 0) in its size, colour 0x11C,
  alignment 5: so its lines start half a line (15 rows) down.
- **`pcx16_copy_rect`** (this the source; the source's corner, the size, the target's pixels, the
  target's corner, the target's width, height and pitch, then a key byte): the target's left and
  top edges cut the part (moving the source's corner with it), its right and bottom edges too;
  the source's are not checked. A key of 0 copies whole; any other skips each source pixel equal
  to the key as a 16-bit value. The credits copy with key 1 (the text's fill) and save the
  background from the screen with key 1 too (0x4EDE77), so a screen pixel of the 16-bit colour 1
  keeps the background picture's own pixel, black (§36.3).
- **A step** (0x4EE022; the background put back first, opaque): while the entering count
  (`ebp−0x1C`, 580) is not 0 it drops by 2 and the text's rows [0, 580 − count) go to (460, 10 +
  count); then, while the first row shown (`ebp−0x14`, 0) is under *H* − 580, it moves on 2 and
  580 rows from it go to (460, 10); then, while the leaving count (`ebp−0x24`, 580) is not
  negative, it drops by 2, the first row moves on 2, and that many rows go to (460, 10), the
  closing text drawn over the window (`small_font`, colour 0xD, alignment 8: the lines at the
  window's bottom, left aligned) **when the count is then under 435** (0x1B3); a count already
  negative marks the end: the window shown with its background alone and the screen left. So:
  290 steps in, 2 082 up (to row 4 164), 291 out (to −2), the end at the 2 664th.

### 36.2 The recordings

- **The steps' times:** the first 100 ms after the screen opens (the main menu's key up at 100,
  the first step at 200: the text's top two rows, blank), then one each 100 ms; the first ink
  (row 22: 15 rows of the alignment, 7 of `C`'s blank top) reaches the window's bottom at the
  12th step (1 300 ms: §2.9's "first frame 1 100 ms after the screen opens" is this first change).
  The closing text first at the 73rd step out (count 434: 244.6 s; the reading's test of §2.9 is
  right for the leaving phase, the only one it is made in); the last ink leaves at count 22
  (265.2 s; the picture's last rows past its last line's ink are blank), the closing text alone
  until the end; the 2 664th step at 266.5 s shows the main menu at once (`menu`). §2.9's and
  §6.3's numbers stand; their "1 100 ms" and "about 435 px" are now these.
- **The ends:** a key (Space), a left press and a right press end the credits at the input; the
  main menu shows at that step (`credits-end`: `key-ended`, `left-ended`, `right-ended`); a move
  does not end them (`moved`).
- **The opening:** the main menu's dialog closed shows the screen without its buttons
  (`opened`); `mainmenu.pcx` is pixel for pixel `GamSelBk.pcx`'s right part (a 24-bit PCX), so
  copying it changes nothing.

### 36.3 The background's key

`mainmenu.pcx`'s pixel (3, 553) is RGB (5, 3, 10), the 16-bit colour 1 on the screen: the save
skips it, so (463, 553) shows black from the first step to the end (`opened` and every frame
after; `GamSelBk.pcx` shows it as 1 in the main menu). A drawing bug of the original (E17's
binding decisions; as CLT-Q138's class, not asked: M4's review list): the client's profile draws
the background whole; the original's (`Bugs::Original`) makes the window's pixels of the key
black from the first step (`h3_ui::ui::Draw::Keyed`), for the comparisons only.

### 36.4 The client

- **`h3_view::shell::credits`:** `Credits` (its counts as the reading's, `Credits::laid` counting
  the text's lines in the look's font at the window's width), `CreditsLook` (the layout's
  `[credits]`), `Texts` (the catalogue's `shell.credits.text` and `.closing`: `CREDITS:1`,
  `CREDITS:2`); its clock steps once a reading, the first 100 ms after its first reading (the
  opening), each next 100 ms after the last (CLT-Q88 (a): no clip; the window waits 5 ms at a turn
  while it shows, as play's fade); the step after the end ends it (`Front::clock` returns a `Did`
  since: the replay's tick and the window's end the screen by it). Its look: the main menu's
  background, `mainmenu.pcx`, the text's part (`h3_ui::ui::Draw::Part`: the picture filled with
  the key, the text drawn whole, the part copied with the key skipped, as the original's), the
  closing text a label over the window. A key down but F4 (`start::key_skips`, the start's rule),
  a left or a right press end it; `Context::Credits` binds no command.
- **The main menu after it** opens anew with the last items the credits found (as after High
  Scores).
- **Not built:** the original's `any_video_running` guard (no clip: CLT-Q86, CLT-Q88), a picture
  that cannot be made (GENRLTXT 223, the end: the client's `Draw::Part` refuses its draw).

## 37. The save file (M4 T3b, 2026-11-20)

No reading: the save is the client's own (decisions 8, 69 and 71; CLT-Q156 (a)), never the
original's (`game_save` 0x4BEEA0 writes the game's memory). Built in `h3-battle-client`'s `save`
(an adapter: files, the engine's crates), tests only; the list, the folder and the file's name are
Load Game's (T4) and Save's (M5).

### 37.1 The file (*provisional*)

- **One file, two parts:** the engine's adventure replay as `h3_files::adventure::replay_to_toml`
  writes it (format v20: the map, the seed, the difficulty, `[[players]]`' choices, the commands,
  the checkpoints, the final hash and day), after a two-line comment; then, from the line `[save]`
  on, the client's table: `version` (1), `name` (the save's name), `duration` (`turndur.txt`'s
  record, at most 10, unlimited), and eight `[[save.colours]]`, red first, each a `bonus`
  (`artifact`, `gold`, `resource`, `random`), a `handicap` (0 to 2) and, for a human's, its
  `name`. The reader splits at the first line `[save]` (CRLF or LF), reads the table by its own
  version first (another or none refused) and its keys strictly, the replay's part by the engine's
  reader (`parse_replay`, which refuses a key of the table's).
- **The map** by its file's name in the installation's `Maps` (`MapRef::path`, resolved against
  that folder; `save::map`), with its content hash and the import's version.
- **Not kept:** the date (the file's last-write time, as the original's list shows it, §3.5), the
  map's header (read from the map the replay names: T4).

### 37.2 The load

In order: the replay's header against the rules (`ReplayHeader::check`: a save of other packs, a
profile, the rules' hashes or the state hash versions refused before anything is read, the field
named); the map read again (`read_map`: a plain name below `Maps`, `h3-map`'s size limit), its
content hash and import version against the replay's; read and imported as the begin reads a map
(`h3_map::read`, `h3_map::import`); a colour the replay makes human must be one the import lets a
human play (E27 M3 T7's rule: the engine cannot check it again); the choices set over the import
(`Replay::setup`: a colour no player has, or twice, refused); a name only for a human's colour; the
game built (`GameState::new`, the engine's validation) and the commands applied in order through
`h3_game::apply`, each as its actor's, with `Replay::verify`'s checks in its order (the
checkpoints' count, each checkpoint, the final hash, the final day). The result: `begin::Game`
(the map's file, the rebuilt setup, the table) and the state after the commands, for M5's host.

## 38. Load Game single: the reading and the recordings (M4 T4a, 2026-11-20)

Decision 72: T4 cut in two, T4a the reading and the recordings (the harness and the docs, no
client code), T4b the client against them. Read with `scripts/exe.py` (what §3.3 to §3.6 left
*not read*); recorded under Wine (`scn-load` again, `scn-load-last` new, `scn-load-none` as kept).

### 38.1 Read

- **The date** (`scenario_info_draw` 0x5841C8 to 0x58426B): the record's file time through
  `FileTimeToLocalFileTime` and `FileTimeToSystemTime`, then "%d/%d/%d - %d:%02d" of the month,
  the day, the year (four digits), the hour (0 to 23) and the minute: `3/17/2002 - 9:07`
  (recorded). Windows' conversion takes the zone's present offset, not the one at the file's
  date, so under a zone with daylight saving time the shown hour follows the day it is shown on;
  Wine's local time is the `TZ` variable's (checked: UTC, Asia/Kolkata, Europe/Berlin). Drawn
  only with a selection and in the load or save mode (or the random maps' list, +0x66).
- **The panel's name in the load and save modes** (0x583F20, from the info draw): the record's
  +0x58C (the save's map name), GENRLTXT 509 when empty, 508 past the list, the current game's
  (game +0x1FB40) with no selection; the autosave's shows GENRLTXT 77 (`AUTOSAVE`, §3.3:
  recorded, the map's name not shown). The rows show the file's name (§3.4).
- **Delete's end** (`saved_game_delete` 0x58E219 to 0x58E2C9): the bar's range (count − rows +
  1); with saves left `scenario_list_sort`(2) in the direction the last sort left (+0x36C kept:
  recorded, a flipped size sort's descending order makes the version order descending too), then
  `scenario_select`(0); with none the whole redraw and `scenario_select`(−1).
- **A held key** (`input_key_message` 0x4EC0E0, read at E24 M2 T1a2): every WM_KEYDOWN is queued,
  a repeat's too, so a held list key moves the selection at each of the system's repeats.

### 38.2 The harness (`h3shell.py`; the runbook's "The shell's screens")

- **The saves made by the original** (the user's `AUTOSAVE.GM1` and `save1.GM1` are of a random
  map, `random_map_6.h3m`, which no client's save can name): three maker recipes (`makes`) play
  a shipped map's new game and leave a save: `scn-load-make-large` `Week_one.GM1` (Middletown,
  108, RoE, Hard), `scn-load-make-small` `small.GM1` (Arrogance, 36, SoD, Easy), both saved from
  the system options on the first day; `scn-load-make-auto` `AUTOSAVE.GM1` (Dawn of War, 72, AB,
  Expert), written at the second day's start (End Turn, the computers' turns on the stepped clock;
  no autosave is written on the first day, nor with the settings' Autosave off, setup.sh's).
  A recipe's `made_games` takes them from `clients/out/shell/games/<maker>/` (local), the maker
  played first when they are missing or its recipe changed since. A save's bytes differ from run
  to run in three bytes after its file's name (heap pointers the original never set), so no
  digest of them is kept; the recipes that list them are recorded twice as any other.
- **The time zone** (`tz`, default UTC): pinned for every shell run (a session's own `TZ` was
  Europe/Berlin on 2026-11-20, the system's UTC: the recordings before T4a took whichever their
  session had). The load's recipes take Asia/Kolkata (+5:30, no daylight saving time), so the
  local conversion shows in the minutes and across a year's end. `drawn` gives the viewer the
  recipe's zone too.
- **Each file its own time** (`games-time:<s>:<file>`): `small.GM1` 10/23/2001 - 14:05,
  `AUTOSAVE.GM1` 1/1/2002 - 4:09 (2001-12-31 22:39 UTC), `Week_one.GM1` 3/17/2002 - 9:07.
- **A held key** (`key-hold:<key>:<s>`): the display's auto-repeat on at Xvfb's rate (660 ms, then
  25 a second) for `<s>` seconds of real time, then off and the key released. A second `xdotool
  keydown` of a held key gives no X event (checked), and the frozen clock cannot time X's repeats,
  so their count follows real time: the recipe holds long enough that the outcome is the list's
  end either way. Each shell run now starts with the display's auto-repeat off (as h3drive's and
  h3map.py's do). The viewer stops at `key-hold` as the harness's own until T4b.

### 38.3 What the recordings show

- **The screen** (`scn-load` screen): title ArrayTxt 230, no Show Available Scenarios, Random
  Map or Advanced Options button (a right press there shows nothing: no-list-button), no flags'
  strip, Load `scnrlod.def`; the list built in key 2's order, the first selected (no kept save:
  §3.1's `NEWGAME` matches none): `Week_one.GM1` (RoE) 4/2 L, `AUTOSAVE.GM1` (AB) 6/4 M,
  `small.GM1` (SoD) 4/4 S, each row's victory and loss marks; the panel the map's name, the date,
  the map's difficulty, the save's difficulty selected on the disabled buttons and its rating.
- **Keys:** Down and Up move a row; a key held 0.3 s (under the delay) one row; held 1.5 s Down
  reaches the last row, Up the first (the repeats act as presses).
- **Rows:** a press selects (row-1, row-2), past the list nothing (row-past).
- **Sorts** (name twice, size twice): the order as §3.4's comparisons, the autosave leading by
  name. After each sort the partial redraw (CLT-Q138, §19.4) leaves the size mark, the difficulty
  buttons and now the rating stale: the rating's text drawn over the last one (name-1: Easy and
  "80%" over "160%" for the autosave's Expert). Decision 48's class: the client draws whole, the
  original's stale pixels for the comparisons only (review item 22).
- **Popups:** Load's is Begin's (`Help.txt` 103, "Start the selected scenario"); a disabled
  difficulty's shows its popup (easy-popup), and its press changes nothing (easy-pressed: the
  whole screen as it was before the sorts, the popups' redraw having drawn it whole).
- **Delete** (688 with the file's name, style 2): Cancel keeps it; OK deletes the selection
  (`Week_one.GM1`), the list in descending version order (`small.GM1` before `AUTOSAVE.GM1`),
  the first selected with the partial redraw's stale difficulty and rating.
- **Load:** the loading's clip, then the map (Arrogance, day 1; play is M5's).
- **The last save deleted** (`scn-load-last`): the empty list, the panel empty but for the size
  mark's last frame and the difficulty last shown, Load disabled; Load, Enter and Down do nothing;
  Back; Single again shows GENRLTXT 686 (no saved games), and after its OK no difficulty chosen.
- **No saves** (`scn-load-none`): as kept.

### 38.4 For T4b

The client's saves made to match the recipes' (the viewer given them as `drawn` gives the made
maps: the same three maps, difficulties and file names, each file's time), so that the list, the
panel and the sorts compare whole; the autosave's day 2 needs the replay's End Turns. The load
ends at play's stub (as M3's begin; `save::load`'s `Loaded` kept for E24 M5), so `scn-load`'s
`load` clip and `map` stay the original's alone. A held key's repeats in the viewer (`key-hold`)
and the window (the shell's window takes none, §16.4). The hot seat's load (T5, `adv-load`) uses
the user's random-map saves too: T5 makes hot-seat saves the same way.

## 39. Load Game single's screen (M4 T4b1, 2026-11-20)

Decision 73: T4b cut at Delete. Built in `h3_view::shell::scenario` (the load mode, `load`),
`menu`, `replay`, `keys`; `h3_ui::text::message` (`%02d`); `h3_sdl::local_offset`; the battle
client's `saves` and `shell`; `h3shell.py drawn`. Nothing new read but the right press (§39.3).

### 39.1 The saves (*provisional*, the task's calls; review items 24, 25, 30)

- **Where:** the client's own folder, `games` in the working directory (the window's `--games`
  names another; a replay's is the harness's run folder), never the original's `games` (decision
  8). A save's file `<name>.GM<n>` as the original's (`game_save_dialog` 0x418160: `.GM` and the
  count of humans still playing), so the rows show the same names; the client's format is §37's.
  The list: up to 5 000 files of `*.gm?` (any case), read in their names' order, then sorted.
- **An entry:** the map the replay names, read from the installation's `Maps` as a new game's list
  reads it (its header and heroes, `maps::entry`), the save's file's name, its difficulty and the
  file's last-write time (`Saved`). The window reads the folder again at each opening of the load
  mode (as the original); a replay once, its `games-time` setting the times (`Step::GamesTime`).
- **A save the client cannot list** (the file does not read as a save, its replay's header the
  rules refuse, its map is missing or moved): as the original lists a broken file (§3.3: GENRLTXT
  431 its name, 505 its description), no map's columns, difficulty Easy. Load's full check (the
  commands replayed, `save::load`) is T4b2's.
- **Made saves for `drawn`** (`h3-view save --map --difficulty --day --name --out`): a single
  game begun on the map as Begin begins it with the advanced options untouched (the single
  player's colour the map's), seed 1, each player's End Turn until the day; the makers' `viewer`
  tables (`difficulty`, `day`) say which, left out of the makers' digests. `drawn` writes them
  into the run's own folder (`--games`), empty without `made_games`.

### 39.2 The screen

- **Built as recorded** (§38.3): the title ArrayTxt 230; Show Available Scenarios, Random Map and
  Advanced Options hidden (their hotkeys S, R, A nothing); no allies' or enemies' labels, flags or
  strip; Load (`scnrlod.def`, the layout's `load_begin`) on Begin's place; the list shown and built
  at the opening (key 2, the kept save `NEWGAME` at the start, else the first; a select keeps the
  file's name for the next opening, *not read*); the rows the files' names; the panel the map's
  name (the autosave's GENRLTXT 77), the date (the client's item 1015, "%d/%d/%d - %d:%02d" at the
  zone's present offset: `h3_sdl::local_offset`, review item 26), the map's difficulty, the
  save's difficulty selected on the five disabled buttons (frame 3, the others 2) and its rating.
- **The sorts** (CLT-Q138, the original's profile): the partial redraw after a sort leaves the
  difficulty's buttons as the last whole redraw drew them and draws the rating over the last one
  (`Scenario::drawn_difficulties`, `drawn_rating`; a new game's select keeps the difficulty, so
  the same pixels there). The name's sort takes a file beginning `AUTOSAVE.` first (review item
  29).
- **Keys:** a held key's presses in the viewer (`key-hold`: 1 under 660 ms, then one each 40 ms,
  `replay::held_presses`), the window's repeats on the scenario screen (`Window::set_key_repeats`,
  review item 28). Delete is a command of the screen (`scenario-delete`, the key Delete).
- **Load and Delete** show the client's notice (`shell.stub.load`, CLT-STUB-15) until T4b2: a
  replay stops at it as at the random map's Begin.

### 39.3 The right press (read)

`scenario_dialog_right_click` 0x5822D0's default (0x582A76): `dialog_right_click` 0x5FF790 (the
item by id, its right-click text or hint; none or empty: false, else a style-4 box and true), and
the dialog's and the screen's redraw (0x582A84 to 0x582A9E) only when it returned true. The
players' items 362 to 385 have their own cases, most of which reach the redraw without a popup
(0x58236F, 0x5823FA). Recorded: `scn-load` no-list-button (no item where a new game has Show
Available Scenarios: the partial redraw's leftovers kept). **Corrects §3.8 and §20.1** ("then the
screen redrawn"; M3 T2c's "a right press that shows no popup redraws whole"): the client's redraws
on the description's box (its scrolling text: `scn-long-text` right-bar) and the players' items
alone (review item 27); every recording of the other groups as before.

### 39.4 The comparisons

`scn-load` 19 of 25 exact at the first build (the four Delete shots T4b2's, the load's clip and
the map M5's); `scn-load-last` 1 of 9 and `scn-load-none` 0 of 5 (the screen, then Delete and
the notice: T4b2's). The `scenario`, `advanced`, `menus`, `multiplayer`, `end`, `options` and
`scores` groups as recorded.

## 40. Delete, the notice and Load's end (M4 T4b2, 2026-11-20)

Decision 74: T4b2 kept whole. Built in `h3_view::shell::scenario`'s `load` (Delete's question,
its end, the notice, Load), `menu`, `play`, `replay`; the battle client's `shell` and `saves`.
Read with `scripts/exe.py` (§40.1); recorded again under Wine (`scn-load`, `scn-load-last`,
`scn-load-none`: as kept).

### 40.1 Read

- **Delete** (`saved_game_delete` 0x58DFB0 to 0x58E2C9): only over a shown list not empty, in the
  load or save mode (+0x64, +0x65); GENRLTXT 688 through `sprintf` with the selected record's
  file's name (+0x33D) in a box of style 2 (`message_box` 0x4F6570); its OK (0x7805) changes to
  `games`, `DeleteFileA` (its result not checked), changes back; the entry found in the full list
  by its file's name (a byte compare) and removed from both lists; the bar's range (count − rows
  + 1); with saves left `scenario_list_sort`(2, 1, 1): the key not recorded (+0x185C as it was),
  the direction +0x36C as the last sort left it, the first row to the game and the whole redraw
  (0x5852BF), then `scenario_select`(0, 1): its partial redraw; with none the whole redraw, then
  `scenario_select`(−1, 1).
- **The select of none** (`scenario_select` 0x585500 with −1): the description emptied
  (0x585639), then Begin's state (0x5858D9): in the load mode disabled over an empty full list
  (+0x1034), else enabled; a new game's by the map's format (RoE, AB, SoD enabled).
- **No saves** (`scenario_buttons_show` 0x57FE68, from the constructor): in the load mode an
  empty shown list disables Begin; `scenario_list_finish` 0x583044 sets `no_saves_notice`
  0x69FDA0, which the procedure's next call (`scenario_dialog_proc` 0x587D1B) clears, showing
  GENRLTXT 686 in a box of style 1, then disables Begin again and redraws whole. Recorded: Load
  disabled under the notice (`scn-load-none` notice).
- **A disabled button's hotkey:** Enter over a disabled Load does nothing (recorded:
  `scn-load-last` enter, `scn-load-none` enter).

### 40.2 The client

- **Delete:** the screen's question (`Texts::delete`, `Asking`: the layout, the fonts' metrics
  and GENRLTXT 688 with `{{file}}`, laid out at the press); Cancel closes it (the box's pixels
  back); OK as read, the file the window's to delete from its own folder (`take_deleted`;
  `saves::delete`: a listed plain name of the folder, a file; a failure on standard error, the
  entry gone all the same, review item 32). The replay keeps the saves as Delete left them for
  the next opening (`Scenario::keep`), as the original reads the folder again; the window lists
  its folder anew there.
- **The notice:** `Texts::no_saves`; Load disabled at the opening with no saves, the notice over
  it; its OK the whole redraw. The replay's `press` finds the scenario screen's boxes' buttons
  (`Front::question`: Delete's, the notice, Begin's messages).
- **Load:** over a selected save the screen ends as Begin does (`Scenario::loaded`; `begun`
  none in the load mode); play's start reads the save and loads its game (`save::read_file`,
  `save::load`, a step of the bar each), the `Loaded` kept for E24 M5 (CLT-STUB-13), then play's
  notice in the client's words (`shell.stub.loaded`; a save the load refuses
  `shell.stub.load-refused`, the error on standard error; review item 31). A replay stops at
  `scn-load`'s `clip-until:load:adventure` (the adventure screen is M5's).
- **A disabled button takes no hotkey** on the scenario screen (review item 33).

### 40.3 The comparisons

`scn-load` 23 of 25 exact (the load's clip and the map are M5's), `scn-load-last` 9 of 9 (its
second opening drew `gamselb1.pcx`: the recipe's `backgrounds`), `scn-load-none` 5 of 5. The
`scenario`, `advanced`, `menus`, `multiplayer`, `end`, `options` and `scores` groups as
recorded.

## 41. Load Game hot seat: the reading and the recordings (M4 T5a, 2026-11-21)

Decision 75: T5 cut in two as T4, T5a the hot-seat load's reading and recordings (the harness
and the docs, no client code), T5b the client against them. Read with `scripts/exe.py` (§41.1);
recorded under Wine (`adv-load` again over made saves, `adv-load-more` new; §41.2).

### 41.1 Read

- **The slots** (`scenario_slots_load` 0x57F581 to 0x57F69E, not network): as a new game's
  (§3.12): in the hot seat one slot per name of `hot_seat_names` 0x69CA50 (index 1 up) through
  `scenario_version_from_slots`, then the local slot 1 named as the game's player (+0x20B9C) and
  the names freed; single the local slot with `network_default_name`. The turn's duration the
  save's (game +0x1F843 to +0x378); the list built, the kept save looked for and selected.
- **The rows** (`scenario_players_init` 0x584C57 to 0x584D99, a hot-seat load): every record's
  colour −1, a colour lost in the save no longer one a human may play (game +0x1F636: the map
  player's flag at +0x1F90C + 0x44 *c* cleared); then (1) each colour in order that the save held
  as human (`load_slots` 0x69FDA4, the record's +0xC80 copied by `scenario_to_game`) and a human
  may still play, to the first record in use without a colour; (2) each colour a human may play
  that no record holds, to the first record without one, until none is left. So the save's
  humans go to the names in colour order, a name left over takes the next colour a human may
  play (a computer's in the save), and a saved human past the names keeps no record: its row
  "Computer" (GENRLTXT 469), its kind still ArrayTxt 206 (recorded).
- **The begin after a load** (`scenario_load_begin` 0x58C290, read): the bar stepped; the turn
  timer's limit by the save's duration (`turn_timer_set_limit`, `turn_duration_minutes`); single
  only `scenario_players_init` again; the game loaded (0x4BF330, the week's and month's globals
  kept around it); single only the local record's colour set to the loaded game's current
  player; then `scenario_dialog_begin`(0) in every mode: the human flags from the records
  (§3.17), so a hot-seat load makes human exactly the colours a name holds (CLT-Q160).
- **The save's extension** (`.GM<n>`): from play (`game_save_dialog`, §3.6) *n* the humans still
  playing; the autosave (`game_save` 0x4BEFB0, `%s.GM%d`) *n* is `begin_humans` 0x699274, the
  human records holding a colour counted by `scenario_dialog_begin` (0x588402; single 1); a
  campaign's `CGM`, the tutorial's `TGM` (0x4BEF66, 0x4BEF7A).
- **The list** (`scenario_list_build`, site `scenario_list_two_humans`): in the hot seat a save
  is listed only with its record's +8 (the colours a human may play, the players column's second
  figure) at least 2, as a map: a single game's save of a map for two humans or more is listed
  (recorded: `Week_one.GM1`, Middletown 4/2).
- **The advanced options in a load** (`scenario_advanced_toggle` 0x581383, 0x581743): the
  duration's bar is not enabled (its vtable +0x24 skipped: its arrows take no press, recorded; it
  is drawn as in a new game) and the handicap is disabled (no popup on its right press, where a
  new hot seat's caption shows one: recorded).
- **The Random Map button** (`random_map_toggle` 0x57FEB0): returns at once in the load and save
  modes, but the button shows in a hot-seat load (§3.16): its press and its key R change nothing,
  its popup shows (recorded; CLT-Q161).

### 41.2 The harness and the recordings

- **The makers** (`shell/advanced.toml`, as §38.2's): a hot seat's new game, every colour human
  with its town, hero and bonus chosen by the arrows, so the save holds no random draw a client's
  save could not repeat; the duration's left arrow three times (20 minutes); Begin, the turn
  dialog's Enter, the map's message's OK, then the system options' Save on the first player's
  first day. `adv-load-make-two` `Pair.GM2`: A Warm and Familiar Place at Impossible, A on Red
  (Inferno, the map's; Marius; Artifact), B on Green (Inferno; Ignatius; Gold).
  `adv-load-make-four` `Four.GM4`: Arrogance at Easy, A on Red (Castle, Orrin, Artifact), B on
  Blue (Rampart, Mephala, Artifact), C on Purple (Tower, Piquedram, Artifact), D on Teal (Inferno,
  Fiona, Artifact). Their `viewer` tables give the difficulty alone: `h3-view save` makes a
  single game's save of the name until T5b gives it the hot seat (`drawn` stops at the hot seat's
  load before any shot meanwhile).
- **`adv-load`** (recorded again; the user's random-map saves gone from it): the names A and B
  over `Four.GM4` and `Pair.GM2` (Asia/Kolkata, each file its own time); 19 shots, twice the
  same. **`adv-load-more`** (new): `Week_one.GM1` (§38.2's maker) under A and B, the rows' town,
  hero and bonus cells masked (164, 130 to 392, 330: the original's draws); 2 shots, twice the
  same.
- The list's rows lie at y 135 + 25 *k* (a press at 210 with three saves listed falls past the
  list: the first take's `row-3` selected nothing).
- Not recorded: three or more names over a save; a load's begin in the hot seat (play: M5); a
  network load.

### 41.3 What the recordings show

- **The screen** (`adv-load` screen): the single load's title (ArrayTxt 230) and Load, but the
  panel's buttons (Show Saved Games, GENRLTXT 656; Random Map; Show Advanced Options) and the
  flags' strip; the list in key 2's order (`Four.GM4`, `Pair.GM2`: SoD both, by name), the first
  selected; the save's difficulty selected on the disabled buttons, its rating.
- **The advanced options** (advanced): the save's rows (§3.14: the town, the first hero, the
  bonus; no arrows): A on Red and B on Blue (the save's first two humans), Purple and Teal
  "Computer" with their saved choices and the kind Human or CPU; the handicap disabled; the
  duration the save's (20 Minutes).
- **Presses that change nothing:** row 1's town, hero and bonus arrows' places, its handicap and
  the duration's left arrow (arrows), the duration's right arrow (duration-back), Random Map and
  its key R (random, random-key); the handicap's right press shows nothing (handicap-popup).
- **Popups:** the town's (Castle), the hero's (Orrin), the bonus's (Artifact) as a new game's; a
  saved human's past the names (hero-popup-3: Piquedram); Random Map's (random-popup).
- **Flags** (flag-3, flag-1): Purple's press moves A there, Red becomes "Computer" (the hot
  seat's press, §3.14; both are the save's humans, so CLT-Q110 (b) allows it); Red's press gives
  the opening's rows back.
- **The list** (list: the opening's screen again), the second row (row-2: `Pair.GM2`) and its
  options (advanced-2: A on Red, Inferno, Marius, Artifact; B on Green, Inferno, Ignatius, Gold;
  Impossible, 200%; 20 Minutes), the list again and hidden (list-hidden: the background on the
  left, as a new game's opening).
- **More names** (`adv-load-more` advanced): the save's one human (Red) to A, B on Blue (a
  computer in the save, one a human may play), Green and Orange "Computer" with the kind CPU; the
  duration Unlimited (the single game's).

### 41.4 For T5b

The hot seat's load mode: the Multiplayer dialog's hot seat over Load Game's menu ending in it
(`mp-load` load, load-closed: the notice with no saves), the list's hot-seat filter, the panel's
buttons and flags; the advanced options from the save (its humans, towns, first heroes, bonuses,
duration: the client's save holds them, §37), the names placed as §41.1 (CLT-Q160), the presses
that change nothing (CLT-Q161 for Random Map), a flag's press among the save's humans (CLT-Q110
(b)), the popups. `h3-view save` for a hot seat (the humans, their names and choices; the makers'
`viewer` tables to grow them), so that `drawn` gives the viewer saves of the same names; exact
against `mp-load`, `adv-load` and `adv-load-more` (its masked cells aside).

## 42. Load Game hot seat: the screen (M4 T5b1, 2026-11-21)

Decision 76: T5b cut in two, T5b1 the hot seat's load mode, its list, panel and the advanced
options from the save, T5b2 the presses (the flags among the save's humans, CLT-Q110 (b)) and
the popups. CLT-Q160 (a): the names decide who is human after the load; CLT-Q161 (b): no Random
Map in a load in the client's profile.

### 42.1 Read

- **A load's duration bar is drawn dimmed.** `scenario_advanced_toggle` skips the bar's enabling
  in a load (0x581383: the vtable +0x24, `scrollbar_enable` 0x597140, whose true clears the state
  bits 0x20 and 0x1000), so the bar keeps the disabled state it was made with; the redraw of a
  shown and disabled item calls its +0x20 after its draw (`dialog_item_message` 0x5FE54B: state &
  0xC), the base's `dialog_item_darken` 0x5FE800, `screen_darken` over its rectangle: the arrows,
  the track and the thumb each channel halved (recorded: `adv-load` advanced). The bar's own draw
  (0x596C40) has no disabled branch; a DEF button's +0x20 darkens nothing (its disabled frame).
- **A load's handicap** is not enabled as a single player's (bit 2 cleared, 0x581743): drawn up
  with its caption, taking no press and no right press (recorded: the caption's look as a new hot
  seat's, not the disabled colour).

### 42.2 As built

- **The mode** (`Scenario::opened_hot_seat` of the load mode, `Menus::front`'s Load Game
  Multiplayer after the hot seat's names; `Screen::Scenario(Load, Multiplayer)` built, its stub
  gone): the saves of maps two humans may play (the new hot seat's filter), the list shown and
  built, the panel's Show Saved Games (128, GENRLTXT 656: `shell.scenario.list-saved`) and Show
  Advanced Options, Random Map in the original's profile alone (CLT-Q161 (b)), the flags' strip
  (before the rows are made: the save's first human a human may play, else the first colour a
  human may play); Random Map's press and key do nothing in a load. The window lists its folder
  anew at the hot seat's load's opening too.
- **A save's entry** (`Saved` grown; `h3-battle-client`'s `saves::saved`): the colours its
  replay's players hold as human, each colour's town (a fixed faction's choice, else Random) and
  first hero (the pick, else Random), its table's bonus and handicap, the turn's duration.
- **The rows** (`Advanced::set_load`, `remake`, `seating`): the save's choices by colour, a human
  record taking the record of the colour it is seated on; the seating §41.1's two passes (a saved
  human no human may play any longer passed over: no engine rule loses a colour yet, E10's); no
  arrows; the handicap not enabled; the duration the save's, its bar not enabled and dimmed.
- **Load** hands on the names by colour (`Scenario::seated`); the window keeps them beside the
  loaded game for E24 M5, which makes human exactly the colours a name holds (CLT-Q160 (a): no
  engine command changes the humans yet; `CLT-STUB-13`).
- **`h3-view save`** for a hot seat (`saves::made_with`, `Player`, `Seated`): `--player
  <colour>:<name>:<town>:<hero>:<bonus>` each a human of its colour (every other the computer's),
  `--duration`; the makers' `viewer` tables name the players (`adv-load-make-two`: A on Red,
  Marius 50, the artifact; B on Green, Inferno, Ignatius 51, gold; `adv-load-make-four`: A to D on
  Red, Blue, Purple and Teal, Castle to Inferno, Orrin 0, Mephala 16, Piquedram 32, Fiona 48, the
  artifact; both 20 minutes, record 7), `h3shell.py`'s `client_save` passing them.

### 42.3 Compared

`mp-load` 6 of 6 exact; `adv-load` 14 of 19 (the screen, the options, the presses that change
nothing, the popups, the list's shots); `adv-load-more` 2 of 2. Not yet: `adv-load`'s flag-3,
flag-1 (a flag's press moves the name with a new game's rule, its town and hero Random, where the
original's takes the colour's saved choices) and random, random-key, random-popup after them
(the rows as flag-1 left them): T5b2.

## 43. Load Game hot seat: the flags (M4 T5b2, 2026-11-21)

Decision 76's second half: a flag's press in a hot-seat load and what follows it in `adv-load`.

### 43.1 Read

- **Random Map's press redraws in a load.** `scenario_dialog_click`'s case 130 calls
  `random_map_toggle` 0x57FEB0 (returning at once in a load, §41.1), then `random_map_tab_draw`
  0x57EF70 (nothing while the tab is not shown, +0x37E), then at 0x586FA7 every item drawn (the
  dialog's vtable +0x14 with 0, 0xFFFF0001, 0xFFFF) and `scenario_dialog_redraw`. So a flags'
  strip a flag's press left stale (CLT-Q138's slip, §26.1: two pixels at (456, 413 to 418) in
  `adv-load` flag-1) is drawn again by the press and by its key R: `adv-load` random equals the
  options' opening (advanced), where flag-1 does not.
- **A moved record's choices:** inferred from `flag-3` and `flag-1` (a row shows the game's
  player of its colour, §3.14): the name on purple shows purple's saved town, hero and bonus, red
  "Computer" red's; red's press gives the opening's rows back.

### 43.2 As built

- **The press** (`Advanced::flag_click`): in a load, after the original's cycle or the client's
  exchange, every human record holding a colour takes that colour's saved record
  (`Advanced::take_saved`, the rows' remake's too). In the client's profile a press is taken only
  on a colour the save holds as human (CLT-Q110 (b), decision 22; *provisional* in its detail:
  M4's review list item 41).
- **Random Map in a load** (`Scenario::toggle_random`): nothing but the whole redraw (the stale
  strip cleared).

### 43.3 Compared

`adv-load` 19 of 19 exact (flag-3, flag-1, random, random-key, random-popup since this task);
every other group as recorded.
