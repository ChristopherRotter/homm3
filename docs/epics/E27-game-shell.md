# E27 — The game shell: main menu and the screens outside play

Status: **refined** (2026-11-12, M0, with the user); **M1 closed** (2026-11-13, T3: the fresh
reviews, the review list accepted, decisions 19 to 22); **M2 refined** (2026-11-13, at M1 T3);
**M2 T1 done** (2026-11-13, decisions 23 and 24); **M2 T2 done** (2026-11-14, decisions 25 to
27); **M2 T2b done** (2026-11-14, decision 28); **M2 T3a done** (2026-11-14, decision 29); **M2
T3b done** (2026-11-14); **M2 T4 done** (2026-11-14, decision 30); **M2 T5a done** (2026-11-14,
decisions 31, 32); **M2 T5a2 done** (2026-11-15); next **M2 T5b** (`docs/lanes/client/QUEUE.md`).
Lane: **client** (E17 closed 2026-11-12; E24's hot seat, M5, consumes the scenario setup; the
map-client lane is consulted). Follows E17 (the battle client: its compositor, SDL adapter, string
catalogue and capture harness). Working detail (readings, the clips, layouts):
[`E27-design.md`](E27-design.md); done tasks in full: [`E27-log.md`](E27-log.md).

## Goal

Every screen of the original that is not play itself, recreated truthfully to the pixel and the
frame, as E17 does for the battle and E24 for the adventure map: from the first frame after
the start, through the main menu and the screens it opens, to a game's begin and, after play,
the way back. A player starts the client, picks a scenario, sets up a hot-seat game and plays it
(E24 M5), with no tool from the harness in between. The scope here is the **faithful
recreation**; the lobby, the ladder and the server browser come later (E28), the campaigns'
screens with the campaigns (E20).

## Decisions

The user, 2026-11-03 (the draft):

1. **A new epic**, not a milestone of E17 or E24: the screens outside play have no home in
   either (E17's "Later" lists the client's options and video; E24's M5 only the handover and
   the hero, town and kingdom screens "as play needs them").
2. **Assigned to the client lane once E17 closes** (E17 closed at its M7 T4, 2026-11-12).
3. **Faithful recreation first.** Lobby, ladder and server browser integrations are a later
   increment, after this epic's screens exist (E28 and E21 build on them). Their look is new
   design (E28).

The user, 2026-11-12 (M0; the questions `CLT-Q79` to `CLT-Q83`, `lanes/client/QUESTIONS-ANSWERED.md`):

4. **The setup is the engine's `h3_game::GameSetup`** (CLT-Q79): the scenario screen fills the
   setup the map's import gives, through the engine's own functions (`GameSetup::offer`, the
   faction choice, `validate`); what it lacks today (the starting bonus, the handicap, victory and
   defeat) E10 adds to `GameSetup`, and the shell stubs it until then. No new crate for the setup
   and no engine change ahead of E10. The shell's state between screens is the client's; where
   its code lives (an existing crate) is settled at M2's T0.
5. **The clips are played, Smacker included** (CLT-Q80): the start's first clip (the 3DO logo),
   the high scores' animation, the loading bar and the credits' clip exist only as Smacker, so a
   port of FFmpeg's Smacker decoder (LGPL-2.1-or-later, as `h3-bink` is FFmpeg's Bink: CLT-Q29),
   in a crate of its own, checked frame for frame against the game's `SMACKW32.DLL` under Wine
   (as `bink-check.sh` checks `BINKW32.DLL`), in M2. The original's order holds: a clip's Bink
   version where both exist (`video_open` 0x44D5A0).
6. **What the engine lacks: the screens built, the effect stubbed** (CLT-Q81): the starting
   bonus, the handicap, victory and defeat (the end screens, the high scores' entry) and the
   computer players (no adventure AI: E13) are drawn and chosen as the original's, recorded in
   the setup where it has a field, each missing effect a `CLT-STUB-<n>` until E10 (E13 for the
   computer's turns); the adventure lane's inbox has the list (2026-11-12).
7. **Four milestones after M0; no campaigns** (CLT-Q82): the campaign selection, the in-between
   screens and the campaign clips move to E20 with the campaigns' rules. New Game's and Load
   Game's Campaign entries are drawn and lead to a stub until E20.
8. **The agent's defaults, kept** (CLT-Q83): the installation's (Complete's) menus only, RoE's and
   AB's are E11's; the saves are the engine's own adventure record (`h3-files`' replay format: the
   map, the difficulty, the seed and the commands; a load replays it; what it lacks of a setup's
   choices is settled at M4's T0 with the adventure lane), the original's `.GM1`
   saves not listed (E19); the original's entries the client cannot serve (the network modes, the
   tutorial) are decided at T1a from the reading, each a `CLT-Q`; which maps the scenario list
   shows (8 of the 160 shipped maps import whole today) is decided at M3's T0 (decision 44: all
   160).

The user, 2026-11-12 (M1 T1a1; `CLT-Q84` to `CLT-Q86`):

9. **T1a cut in two** (T1a1 the start, the main menu, New Game's and Load Game's menus and Quit;
   T1a2 Multiplayer and the hot seat's names, High Scores, Credits). **The tutorial** is drawn and
   leads to a stub until the engine has a tutorial mode (CLT-Q84; the flag's rules, about 45 sites,
   noted in BACKLOG's discovered work inbox). **No first-run case** (CLT-Q85): no `Testing` clip, the
   start's clips always skippable, Bink always (the Smacker copies of `NWCLOGO` and `H3INTRO`
   never play), `Show Intro` a client setting. **The menus' clip 33 draws nothing** (CLT-Q86: on
   Complete a 4 × 4 placeholder; AB's menus show a 68 × 52 animation, E11's). The reading also
   found that the loading bar is no clip (`loadbar.pcx`, `loadprog.def`) and `ProgressBar` a
   network transfer's (design §2.1), so decision 5's Smacker serves `3DOLOGO` and the high scores.

The user, 2026-11-12 (M1 T1a2; `CLT-Q87` to `CLT-Q89`):

10. **The network modes drawn, a stub** (CLT-Q87): Multiplayer's first stage is drawn as the
    original's with its six buttons; Hot Seat works; IPX, TCP/IP, Modem, Direct Connect and Online
    lead to one stub (a message) and the dialog stays on its first stage; the second stage (Host,
    Join, Search, the session list, their dialogs) and `ProgressBar`'s transfer are not built, and
    E28 decides how the project's network play enters the shell; Online never ends the client.
    **The credits' clock** (CLT-Q88): the client steps the credits ten times a second (Complete's
    placeholder clip's rate) with no clip. **Two oddities as the original's** (CLT-Q89): the
    Multiplayer Cancel's item 64 × 48 over its 64 × 32 picture; no check box in the hot seat's
    names (its assets unused). The reading also found that `HSANIM` and `HSLOOP` play at a won
    scenario's score screen (T1b), not in High Scores (design §2.8).

The user, 2026-11-12 (M1 T1b's start):

11. **T1b cut in three:** the scenario screen alone is about 80 KB of code in about 110
    functions (T1a2 read about 11 KB), one dialog for a new game, a load and a save (its mode 2,
    from play). T1b1 the scenario screen's frame, its list and information panel in the three
    modes (the saved games and the save's file name included), the difficulty, Begin and Back;
    T1b2 the advanced options (the players' setup, each choice to `GameSetup` or an E10 gap), the
    random map's tab (a `CLT-Q`) and the game's begin; T1b3 the system options and the menu in
    play, the end of a scenario (victory, defeat, their clips and messages, the win screen, the
    high score's entry).

The user, 2026-11-12 (M1 T1b1; `CLT-Q90`, `CLT-Q91`):

12. **The scenario list's order** (CLT-Q90): a stable sort by the column, then the name (as the
    name column compares it), then the file name; a capture whose list has ties may differ, each
    difference listed. **Its two oddities fixed** (CLT-Q91): the first press on the size column
    sorts ascending (the client records the key it sorted by), and a double click needs the same
    row twice; the original's rules are kept for the comparisons only.

The user, 2026-11-12 (M1 T1b2; `CLT-Q92` to `CLT-Q95`; T1b2 kept whole at its start):

13. **The random map's tab drawn, Begin a stub** (CLT-Q92): its choices as the original's; Begin
    with a random map chosen shows a stub's message; Show Random Maps lists the client's own
    `random_maps` (empty); no generator (the non-goals). **Three oddities fixed**, the original's
    rules kept for the comparisons only: single player on a map with a human-only colour, only
    the other human-only colours become the computer's and the player may return to its start
    (CLT-Q93; no shipped map has one); the hot seat's flag passes the colour to the next player
    after its holder in the slots' order, the two exchanging colours, after the last the
    computer's (CLT-Q94); a human's Random town stays Random, drawn by the engine at the build
    (CLT-Q95; the original takes the town drawn at the list's read). The reading also found that
    the bonus, the handicap and the turn's duration have no `GameSetup` field (design §3.9: the
    first two E10's, the timer the hot seat's, E24 M5).

The user, 2026-11-12 (M1 T1b3's start, and T1b3a; `CLT-Q96` to `CLT-Q98`):

14. **T1b3 cut in two** (about twice T1b2's code): T1b3a the system options and the menu in play
    (their buttons, every setting and where the original keeps it, the save from play, the
    restart); T1b3b the end of a scenario (`game_main` after play, victory and defeat, their clips
    and messages, the win screen and its score, the high score's entry).
15. **Video Quality honoured** (CLT-Q96): Low plays the Smacker copies where the original does;
    `shell-videos.vid` rebuilt with `H3INTRO.SMK`, `LOSEGAME.SMK` and `NWCLOGO.SMK` (design §1);
    CLT-Q85 stands for the first run. **The restart as the original's** (CLT-Q97): the first
    game's `GameSetup` with each Random town fixed to the one drawn, a new seed. **Restart asks
    inside the dialog** (CLT-Q98, an oddity fixed): its question (GENRLTXT 68) before the dialog
    closes, Cancel keeping it open, as Load, Main Menu and Quit; the original's order for the
    comparisons only.

The user, 2026-11-12 (M1 T1b3b's start and its reading; `CLT-Q99`):

16. **T1b3b kept whole** (about 12 KB of code; the cut offered at the screens after play against
    victory and defeat in play was declined).
17. **The high scores' eleventh row as the original's** (CLT-Q99): a won scenario's score is
    ranked among the first ten rows only; one that misses them always replaces the eleventh row,
    shown highlighted (design §5.6).

The user, 2026-11-13 (M1 T2's start):

18. **T2 kept whole** (the cut offered: the tool and the start with the menus, then §2's other
    screens, §3, §4 with §5), its recipe groups recorded by sub-agents, two at a time on their own
    Wine instances.

The user, 2026-11-13 (M1 T3's start; `CLT-Q100` to `CLT-Q102`):

19. **T3 kept whole** (the cut offered: the review and the questions, then the review list and
    M2's refinement in a session of their own), with sub-agents used efficiently: one recorded the
    groups again, two fresh reviewers (the catalogue but §3; §3 with the `scenario` and `advanced`
    groups' shots).
20. **T2's three oddities fixed**, the original's rules for the comparisons only: a single
    scenario's player takes the client's player name, the colour's name when that is empty
    (CLT-Q100); the multiplayer dialog's status line drawn again at a stage's change (CLT-Q101);
    the system options' boxes and bars act on a release over them alone, as the combat options'
    already do (CLT-Q102 as CLT-Q43 (2)), and **the code is shared, not built twice**: one
    implementation of a dialog's buttons, boxes and bars serves the battle's and the shell's
    dialogs.
21. **M2's T0's questions** (`CLT-Q103` to `CLT-Q106`): the shell's code as the battle's
    (CLT-Q103): its pure screens and state machine in `h3-view`, its element and adapters in
    `h3-battle-client` (the `h3-view` command line's `shell` the player's program until E24 M5
    hosts it), the Smacker decoder in `h3-smacker`, the dialogs' shared input in `h3-ui`; **the
    clips' sound exact against the DLLs** (CLT-Q104): Bink's and Smacker's audio ported, the
    floating point emulated as `h3-bink`'s inverse DCT, checked sample for sample against
    `BINKW32.DLL` and `SMACKW32.DLL` under Wine where the harness reaches them; **the won
    scenario's score is the engine's** (CLT-Q105: E10 computes and reports it, the shell shows
    it, a stub until then); **one victory box** (CLT-Q106: the first condition's alone, the
    original's two for the comparisons only).

The user, 2026-11-13 (M1 T3: the fresh reviews' oddities, `CLT-Q107` to `CLT-Q118`, each asked
alone; the review list; M2's cut):

22. **Fixed**, the original's rules for the comparisons only: Enter in the hot seat's names presses
    an enabled OK (CLT-Q107, against the recommendation); the save mode's description the game's
    own (CLT-Q108); Begin refused with GENRLTXT 531 while a named player has no colour (CLT-Q109);
    a hot-seat load moves colours among the save's humans only (CLT-Q110); the random map's record
    built at each opening of its tab (CLT-Q111); no popup over a hero None (CLT-Q112); GENRLTXT
    530 with no maps at all (CLT-Q114); the size mark following the random map's size, frame 4
    over an empty list (CLT-Q115); both drawing slips, the shadow and a cut name marked as cut
    (CLT-Q117, against the recommendation); New Game's and Load Game's buttons redrawn whole
    (CLT-Q118). **Copied:** Save's and Load's right press shows Begin's text (CLT-Q113). **The
    Random town's count** (CLT-Q116): the original's string with an override that puts in the
    number of town types the enabled content packs offer (packs adding factions raise it). **M1's
    review list accepted** (its twelve items, `E27-archive.md`) and **M2 cut in eight tasks**.

The user, 2026-11-13 (M2 T1; `CLT-Q119`):

23. **T1 kept whole** (the decoder, the `video` command and the check against `SMACKW32.DLL` in one
    task; the cut at the DLL's check offered should the DLL differ from FFmpeg in a way that needs
    its code read: it did not).
24. **The Smacker clips' 16-bit colours fixed** (CLT-Q119): `SMACKW32.DLL`'s 16-bit copy, the one
    the game shows, converts only the blocks a frame writes through its palette, so after a palette
    change the skipped blocks keep their old colours (`HSLOOP`, always played: 16 of its 46 frames;
    the Low quality copies of `NWCLOGO` and `H3INTRO`); the shell converts every pixel through the
    frame's palette, the DLL's copy (`h3_smacker::check::original_16`) for the comparisons only
    (`Bugs::Original`). Design §7.

The user, 2026-11-14 (M2 T2; `CLT-Q120`, `CLT-Q121`):

25. **T2 kept whole** (the cut offered: Smacker's sound, Bink's sound, the checks against the DLLs
    and the window's audio, one or two a session; all four in one, the long loops in narrow
    passes).
26. **Ooura's FFT ported** (CLT-Q120): `BINKW32.DLL`'s inverse FFT is Takuya Ooura's `fft8g.c`;
    `h3_bink::fft` ports it as the DLL compiled it, crediting Ooura and his terms in its header,
    under the project's GPL-2.0-or-later. Design §8.3.
27. **Every clip's sound at the effects setting** (CLT-Q121): the game plays Smacker clips at the
    effects setting and Bink clips (the start's) at full volume whatever it is; the client plays
    every clip at the setting, the original's full volume for the comparisons only
    (`Bugs::Original`, `h3_view::clip_sound::volume`). Design §8.3.

The user, 2026-11-14 (M2 T2b; `CLT-Q122`):

28. **The Bink clips shown whole** (CLT-Q122): `BINKW32.DLL`'s `BinkCopyToBuffer` copies only the
    cells its decoding marked changed and files the chroma's marks under other cells, so the
    original shows a chroma change a frame or a few late where the luma stood still; the client
    shows every decoded frame whole, the DLL's copy for the comparisons only (`h3_bink::Target`,
    `Reading::Original`, `Bugs::Original`). Design §9.

The user, 2026-11-14 (M2 T3's start):

29. **T3 cut in two** (the agent's recommendation): T3a the frame (the dialogs' input moved into
    `h3-ui`, the shell's state machine, `h3-view shell` with its text driver; no original data
    needed), T3b start-up loading by the LODs' tables (CLT-Q2: every archive reader of
    `h3-battle-client` on the tables and a batch's ranges, `h3-view gate` measuring it; the four
    LODs staged). T4 needs T3a only.
30. **T4 kept whole** (the user, 2026-11-14; the cut offered: the comparison and the main menu at
    rest, then the harness's sound and TCP/IP's mask, the window to T5): `h3shell.py drawn`, the
    main menu at rest exact, h3shell's `sound`, TCP/IP's address masked (M1's review item 8 (b))
    and the shell's window in one task.

The user, 2026-11-14 (M2 T5's start):

31. **T5 cut in two** (the agent's recommendation, at the picture and sound seam): T5a the start's
    pictures (the readings of NWCLogo's placement, the end-of-clip screen call, the screens
    between the clips and the buffer before a clip; the Start screen built: the clips in order,
    `H3Intro` in `introrim.pcx`, the skips, `Show Intro`, Bink Video 0's Smacker copies; `drawn`
    exact against the `start` group); T5b the start's sound (`game_samples_load`, the 3DO logo's
    sound path and volume, how much of a Smacker clip's queued sound plays at its end; the clips'
    sound in the shell at the effects setting with the window; the start recorded with `sound =
    true` and compared).

The user, 2026-11-14 (M2 T5a; `CLT-Q123`, `CLT-Q124`):

32. **The start's display fixed** (CLT-Q123, CLT-Q124; design §10): the client fades the screen as
    shown (the original fades its back buffer, where a skipped `NWCLogo` sits unstretched at the
    corner) and shows each clip frame's whole rectangle (the original shows the union of the
    DLLs' changed rectangles by an arithmetic that can leave part of a frame unshown for a few
    frames); the original's for the comparisons only (`Bugs::Original`). The SMACKW32 rectangles
    left for a task of their own (T5a2), the cut offered at T5a's end.

The user, 2026-11-15 (M2 T5b's start; `CLT-Q125`):

33. **T5b cut at the game's run** (the agent's recommendation, after the user first kept T5b whole:
    the reading found that with sound `SMACKW32.DLL` times a clip's frames by its DirectSound
    buffer's cursor, which runs in real time under Wine, so the start recorded with `sound = true`
    needs a sound device on the harness's clock first): T5b1 the readings, a stand-in DirectSound
    in the probes (`smackprobe wait`), the client's pacing with sound checked frame by frame
    against the DLL there, the shell's sound in the window; T5b2 the same device in `xdd.dll` for
    the game's runs (and Bink's Miles calls), the start recorded with `sound = true` and compared.
34. **A clip's last frame of sound played** (CLT-Q125; design §11.3): `SMACKW32.DLL` stops a
    clip's buffer at `SmackClose`, right after its last frame, so the last frame's sound never
    plays (66 ms at the start's clips, silence there; 100 ms at `HSANIM` and `LOSEGAME`,
    audible); the client plays the track out after its clip's end (`Op::Closed`), a skip still
    stopping it (`Op::Quiet`); the cut for the comparisons only (the replay's `closed` lines).

The user, 2026-11-15 (M2 T6a; `CLT-Q126`):

35. **T6 cut by screen** (the agent's recommendation; the cut offered: by screen, by pictures and
    sound, or whole): T6a the main menu whole and Quit's question (the menus' machinery: the
    menu's state, the dialogs' input in the replay and the window, the help popups, the message
    box, `button.wav`, `MAINMENU.MP3`; `menu-main`, `menu-quit` and `menu-sound`'s lines up to New
    Game's menu); T6b New Game's and Load Game's menus on it (CLT-Q118's rows for the
    comparisons), the Campaign, Tutorial, High Scores and Credits stubs, `menu-new`, `menu-load`
    and `menu-sound` whole.
36. **The main menu's buttons drawn in their order** (CLT-Q126; design §13.3): the original
    redraws a pressed or released button alone, so a pressed Load Game shows over High Scores'
    top two rows until a hover change redraws the five; the client draws them in their order
    always, the original's order for the comparisons only.

The user, 2026-11-15 (M2 T6b's start):

37. **T6b cut at the stubs** (the agent's recommendation; offered: the cut or whole): T6b1 New
    Game's and Load Game's menus on T6a's machinery (the reading, `menu-new`, `menu-load` and
    `menu-sound` recorded again and exact, CLT-Q118's rows for the comparisons); T6b2 the
    Campaign, Tutorial, High Scores and Credits stubs, each a message box in the client's own
    words: the client's first own strings, so its language pack (`clients/text/en.json`) and its
    loading come with them.

The user, 2026-11-16 (M2 T7's start):

38. **T7 kept whole** (the cut offered: the edit field, the 21-letter names recorded and
    Multiplayer's first stage first; the hot seat's names, Load Game's multiplayer and the
    scenario screen's stub second).

The user, 2026-11-16 (M2 T7; `CLT-Q127`, `CLT-Q128`):

39. **The edits type what the keyboard prints** (CLT-Q127 (b)): the window's text input in the
    system's layout (SDL's text events: Caps Lock and dead keys included), kept to the characters
    the font draws (code page 1252: accented letters typed), the edits' own keys (Home, End, the
    arrows, Delete, BackSpace; the hot seat's Tab, Shift and Tab, Up, Down, Enter) through the
    shell's key table; the original's US table by scan code for the comparisons only. Built at
    T7b.
40. **A menu opening anew takes the hover at its first move** (CLT-Q128 (b)): the original's
    statics (the item the pointer left the menu over is no change there) for the comparisons only.

The user, 2026-11-16 (M2 T8, M2's closure; `CLT-Q129` to `CLT-Q136`):

41. **T8 kept whole** (the cut offered: M2's closure first, M3's refinement in a later session),
    sub-agents used for the bounded jobs: the recordings, two fresh reviewers, the fixers.
42. **M2's review list accepted with the reviewers' changes, fixed in T8** (offered: the first
    M3 task, or split): the window's start skipped by F4 no more; `message::Open` on
    `h3_ui::input` (decision 20); one copy of the DLL's 16-bit frames; the shell window reading
    the user's key file; the music with the effects at 0; the replay stopping under the network
    notice; key conflicts across contexts asked together; the stale rows under the Multiplayer
    dialog; the shared `Did` helpers; the glyphs' widths measured; the stale headers and
    comments.
43. **The oddities the review found** (each as recommended): the silent Smacker clips' pace from
    their due times in the client's profile, `SmackWait`'s drift for the comparisons only
    (CLT-Q129 (b)); an input in `H3Intro`'s end fade reaches the main menu, as the original's
    (CLT-Q130 (a)); the caret not blinking, as the original's (CLT-Q131 (a)); no disk check in
    the client (CLT-Q132 (a)); the edit's two flaws that never show not built (CLT-Q133). Built
    in T8.
44. **M3 refined** (M3 T0): the scenario list shows all 160 shipped maps, as the original's
    (CLT-Q134 (a): all import and validate since E08 T97b1; the objects not yet built stay inert),
    each read whole (`h3_map::read`, 160 maps in about 0.15 s: no header-only read, no engine
    change; *provisional*); Begin builds and validates the engine's `GameSetup`, shows the loading
    bar, then a notice (a `CLT-STUB`) and back to the menu, the setup kept for E24 M5 (CLT-Q135
    (a)); Options move to M4 with the menu in play, one dialog over the map client's screen
    (CLT-Q136 (a)); M3's task table as proposed (below); the load and save modes are M4's, each
    of design §6.4's recipe fixes the task's that builds its screen.

The user, 2026-11-17 (M3 T1's start):

45. **T1 cut in two** (offered: T1a and T1b, T1 whole, or M2's leftover alone): the screen's
    shots show the kept map's information panel from the first, so **T1a** builds the frame with
    the panel (§3.5's drawing, T2's in the table before) and **T1b** the list; T2 keeps the
    difficulty's presses, Begin's messages and the description's scroll bar.
46. **The arrow after the scenario screen's busy cursor** (CLT-Q137 (b)): the client keeps the
    default arrow; the original's adventure map's arrow (`cradvntr.def`, until the menu loop's
    top) for the comparisons only. Built in T1a.

The user, 2026-11-17 (M3 T1b's start):

47. **T1b cut in two** (offered: T1b1 and T1b2, T1b whole, or a smaller first cut without the bar
    and the keys): **T1b1** the list itself (the toggle, rows, columns, sort, filters, keys, the
    scroll bar, a row's press and double press), exact against `list-shown`, `scn-list-keys`
    (§6.4's fix recorded), `scn-sort`, `scn-filter`; **T1b2** the right press on the list, the
    labels and texts, the teams dialog and GENRLTXT 503 in a shot, exact against `scn-right`.
48. **The list's partial redraw** (CLT-Q138 (b)): the client draws the scenario screen whole; the
    original's partial redraw (`scenario_select` with an unchanged selection: the size mark, the
    panel's edge over Begin, the flags' strip) for the comparisons only. Built in T1b1.

E17's binding decisions hold here as in E24 (`docs/epics/E17-clients.md`, "Decisions"):
exhaustive, automated diffs against the original's captures under Wine; the original's drawing
bugs fixed, reproduced for the comparisons only (`h3_render::Bugs`); every text through the
string catalogue; every oddity of the original an open question (the lane HANDOFF's autonomy
contract).

## Scope (read from the original at M1; names are the original's as far as known)

Each screen is read from `Heroes3.exe` and its assets (`docs/exe/map.toml`), captured under
the harness as a reference, and compared at the pixel. Where the editions differ (RoE, AB,
SoD) the shipped installation decides; E11 owns historical differences.

- **The start:** the clips before the main menu (the 3DO and NWC logos, the intro: `3DOLOGO`,
  `NWCLOGO`, `H3INTRO`; `H3X1intr` is not played at the start), skipping them, the loading bar
  (`loadbar.pcx` and `loadprog.def`), the main menu and its buttons, its music. The clips:
  design §1; the reading: design §2.
- **Start-up loading** (E17's CLT-Q2, placed at its M7 T4): the archive set read by the LODs'
  tables at start and a batch's entries by their ranges (`h3_formats::lod::LodTable`), `h3-view
  gate` measuring it; `lanes/client/NOTES.md`, "Assets and archives".
- **New Game and Load Game:** the choice of single scenario, multiplayer, campaign (a stub until
  E20) and the tutorial's place (T1a1: drawn, a stub, CLT-Q84).
- **Multiplayer's hot seat:** the names' dialog; the other modes drawn and stubbed (decision 10;
  design §2.6, §2.7).
- **Scenario selection:** the list with its filters and sort, the scenario's information, the
  difficulty, the players' setup (colour, human or computer, town, hero, starting bonus,
  handicap, as the original offers them), the advanced options, the map's description, and a
  game's begin: the filled `GameSetup` handed to the hot seat (E24 M5). The random map's tab is
  read and decided at T1b (the engine has no generator). The E17 design's hot-seat flow (design
  §6.3) is the start of the reading.
- **Load Game and Save Game:** the lists, the dialogs, the engine's adventure record as the file
  (decision 8).
- **Options:** the system options dialog (volumes, speeds, the original's settings), wherever the
  original reaches it (T1b: the adventure screen alone; M4's with the menu in play, decision 44). From E17 (placed at its M7 T4): the volume settings (0x5996C0) and a
  track resuming where it stopped (0x59AA82), the client's own options beside the original's
  (CLT-Q25's step of a two-hex stack one hex back by the mouse, a gameplay option off by
  default); `lanes/client/NOTES.md`, "Options and layouts". The settings in the client's own
  file, never in the installation.
- **The system menu in play** (save, load, restart, main menu, quit; over the map client's
  adventure screen, with the map-client lane) and **the end of a scenario** (victory, defeat,
  their clips and messages, the high-score entry; the effects behind E10, decision 6). The system
  options and the menu in play: design §4 (T1b3a); the end of a scenario: design §5 (T1b3b).
- **High scores and credits** (design §2.8, §2.9; the scores in the client's own file, the
  defaults `CampHigh.txt`'s; `HSANIM` and `HSLOOP` at a won scenario's end, T1b).
- **Sound and music** of the menus (E17's sound decoders).

## Non-goals

- Lobby, accounts, ladder, server browser, the pause by agreement: E28 (refined after this
  epic), E16 M2 and E21, after this epic's faithful screens (decision 3).
- The campaigns' screens, clips, rules and scripts (E20, decision 7); the original's save-file
  import (E19).
- The map editor and its screens (`h3maped.exe`); a random map generator.
- Playing a game: the adventure map (E24), the battle (E17), the town and hero screens (E24 M5,
  E09); the computer players' turns (E13).
- HD layouts and graphics options beyond `faithful` (E29).

## Milestones

Each milestone is refined at its own T0 with the user (its tasks into the lane's queue); M1's
tasks are refined here.

| M | Scope |
|---|---|
| M0 | Refinement with the user (done 2026-11-12: decisions 4 to 8, CLT-Q79 to CLT-Q83) |
| M1 | (closed 2026-11-13) **Reading and captures:** the catalogue of the original's screens, each with its entry points, transitions, assets, texts (table and record) and sounds; the clips cut from the user's `VIDEO.VID`; the harness on every screen and its captures; the traps list; the questions the reading finds |
| M2 | (closed 2026-11-16) **The start and the menus:** the Smacker decoder (decision 5); the clips at start and their sound; start-up loading by the LODs' tables (CLT-Q2); the main menu, New Game, Load Game's choice, Multiplayer and the hot seat's names, Quit's question, their hover, press, sounds and music, exact against the captures; the shell's state machine with a headless driver (the text client first, E17's pattern) |
| M3 | (refined 2026-11-16) **A game's setup:** scenario selection, the players' setup, the advanced options, the game's begin (the filled `GameSetup` kept for E24 M5, CLT-Q135; the stubs of decision 6) |
| M4 | **Around play:** Save and Load (the engine's record), the system menu in play and Options, one dialog (with the map-client lane; decision 44), the end of a scenario's screens, high scores, credits |

### M1 — reading and captures (closed 2026-11-13)

Tasks T1a1 to T3 and the review list (accepted at T3, decision 22): `E27-archive.md` (verbatim);
the build record: `E27-log.md`; the readings and captures: `E27-design.md` §1 to §6 (§6.4: what M2
to M4 still read or record). The shell's recipes (`clients/capture/shell/`) gave T2's kept digests
back at T3, in a fresh sandbox, every recipe but the two full starts.

### M2 — the start and the menus (closed 2026-11-17)

Tasks T1 to T8 and the review list (accepted at T8 with its changes, decisions 42 and 43):
`E27-archive.md` (verbatim); the build record: `E27-log.md`; the readings: `E27-design.md` §7 to
§17. The shell's frame, its start, menus, stubs, Multiplayer dialog and edits are M3's ground.

### M3 — a game's setup (refined 2026-11-16, M3 T0 at M2 T8)

Decisions 4, 6 and 44 bind it: the scenario screen fills the engine's `GameSetup` through the
engine's own functions (4); what the engine lacks is drawn and chosen, its effect a `CLT-STUB`
(6); every shipped map listed, each read whole, Begin's hand-over a notice until E24 M5, Options
M4's (44). Each screen is exact against its group's shots (`scenario`, `advanced`; recorded again
at M2 T8), with the sound sequences its task records; each of design §6.4's recipe fixes is
recorded by the task that builds its screen (`h3shell.py run --only <id> --record`). The
catalogue: design §3; the shell's frame, menus, edits and the hot seat's names: M2's
(`E27-archive.md`).

| Task | Scope | Status |
|---|---|---|
| T1a | The scenario screen's frame and its information panel (§3.1, §3.2, §3.5's drawing, §3.8; decision 45): the new-game mode of a single scenario, the list hidden; both backgrounds; the kept map's name, description, victory and loss, the flags, the size, the map's difficulty and the rating; the panel buttons (caption buttons), Begin and Back, their press, hold and popups; Back and Escape; the maps read whole; exact against `scn-new` but `list-shown`; M2's leftover (found met: `Key::Other`) | **done** 2026-11-17: `h3_view::shell::scenario` (`Scenario`, `Entry`, `Texts`, `sort`), `Front::Scenario`, `Menus::scenario`, `menu::Arrow` (CLT-Q137 (b), decision 46: the original's arrow for the comparisons only), `look::ScenarioLayout`, the replay's `with_backgrounds` (a recipe's `backgrounds`); `h3_ui::ui::Caption`; `h3-battle-client`'s `maps` and the shell's `--backgrounds`; the layout's `[scenario]`, `[adventure_cursor]`; the keys `shell.scenario.*`; `scn-new` 30 of 31 exact; log "M3 T1a", design §18 |
| T1b1 | The list (§3.3, §3.4; decision 47): its panel and the toggle (128), the rows and columns, the sort, the size filters, the list's keys and scroll bar, a row's press and double press; exact against `scn-new`'s `list-shown`, `scn-list-keys`, `scn-sort`, `scn-filter` (§6.4: `up-past-top` with Page Up first) | **done** 2026-11-17: `h3_ui::scrollbar` (`H3DlgScrollbar` read whole); the screen's list (`ListLook`, `sort_by`, `list_names`, `Scenario::clock`, `Front::clock`), the keys `Command::List`, the replay's Prior and Next, the viewer's `--maps`; CLT-Q138 (b), decision 48 (the original's partial redraw, for the comparisons); `scn-list-keys` recorded again; 59 of 67 exact, the 8 others the ties' order (decision 12); log "M3 T1b1", design §19 |
| T1b2 | The list's right press (§3.8; decision 47): the popups on the columns, the filters, the bar, a row (nothing), the labels and the texts, the flags' strip's teams dialog (387, 0x576540); GENRLTXT 503 in a shot (design §19.6); exact against `scn-right` | next |
| T2 | The information panel's rest, the difficulty, Begin and Back (§3.5, §3.6): the panel of every map (the description's scroll bar, the empty list's panel), the difficulty's presses and the rating (`difficulty_names` read at T1a, design §18.2), Begin's messages; exact against `scn-info`, `scn-difficulty`, `scn-begin-none` (its `what` corrected: 530 is a filter's), `scn-begin-filter` | after T1b |
| T3 | The advanced options of a single player (§3.12 to §3.15): the rows, the town's, hero's and bonus's arrows, the handicap, the flags, the turn's duration, the popups and the hover; the choices into `GameSetup` (§3.9: `GameSetup::offer`, the faction choice), the bonus, the handicap and the timer drawn and chosen, each effect a `CLT-STUB` (decision 6); exact against `adv-single-town`, `adv-single-hero`, `adv-single-rest`, `adv-single-maps` (§6.4: `popup-hero-arrow` on row 2's arrow) | after T2 |
| T4 | The advanced options in the hot seat (§3.14): two to eight humans, the flags' cycle, the names from the Multiplayer dialog (`Multiplayer::names`, M2's item 61); exact against `adv-hotseat`, `adv-hotseat-rows`, `adv-hotseat-two` | after T3 |
| T5 | The random map's tab (§3.16; CLT-Q92: drawn, its generator a stub); exact against `adv-random` (§6.4: Begin after `tab-again`) | after T4 |
| T6 | The game's begin (§3.17, §2.1; CLT-Q95, CLT-Q135): the setup built and validated by the engine (a refusal shown), the loading bar (`loadbar.pcx`, `loadprog.def`; the client's steps), the begin's notice (a `CLT-STUB`) and back to the menu, the setup kept for E24 M5 (the map-client lane's inbox); exact against `adv-begin` and `adv-hotseat-begin` up to the bar | after T5 |
| T7 | M3's closure: a fresh reviewer, the review list, M4's refinement (M4 T0: the saves' record and its extension, decision 8; Options with the menu in play, CLT-Q136; the load and save modes with §6.4's recipes for them) | after T6 |

**M3's review list** (the agent's calls, *provisional*, for T7): (1) the maps read whole
(`h3_map::read`: the 160 in about 0.15 s in release, 2026-11-16), no header-only read in
`h3-map`; T1a reads them once, at the shell's start (the original at each opening); (2) T1a: what
the redraw draws itself (the name, the texts and marks) and the original's unnamed items (its
100s) as layout items of the client's own ids (1000 on, no hint or right press); (3) T1a: the
background's draw given to the replay by a recipe's `backgrounds` (the original's under the
harness), the window's from the system's randomness; (4) T1a: the description as one label of
width − 3 while it fits (design §18.1); (5) T1a: the caption buttons as a `caption` of
`h3_ui::ui::Button` (the foundation, additive); (6) T1b1: the list's rows drawn by the screen
from a row template (`[scenario.list]`), the bar a `h3_ui::scrollbar` beside the dialog (not a
dialog item: a new `ui::Item` variant would change the map-client lane's code), drawn after the
items; (7) T1b1: a row's double press timed on the shell's clock given by the window and the
replay (`Front::clock`, the screen's opening its first time); (8) T1b1: the original's partial
redraw reproduced from the items' order and a stale rectangle (design §19.4), not a screen kept
between frames.

## Dependencies and boundaries

- **E17 closed** (2026-11-12): the compositor, SDL adapter, fonts and string catalogue are used as
  they are; this epic adds screens, not a second renderer. `h3-bink` decodes the Bink clips.
- **E24 M5 (hot seat)** starts its game from the filled `GameSetup` (decision 4); what else the
  hand-over carries (the hot seat's names, the map's file) is agreed with the map-client lane at
  M3's T0 within the existing crates. The system menu in play (M4) draws over E24's screen.
- **E10** (adventure lane) adds the starting bonus, the handicap and victory and defeat to the
  engine; **E13** the computer players; until then the stubs of decision 6.
- **The engine and the server** own what a game is: the shell builds a game's setup and hands
  it to the engine (ADR-0009's boundary); no rule lives in the shell.
- **Maps** are read through `h3-map` and the installation copy; the shell never writes to the
  user's installation.

## Risks and known gaps

- **Wine is not Windows** (E17's risk): the captures of menus run on Wine's DirectDraw; the
  menu is the screen E17 already proved exact on, so the risk is small, but a screen that
  runs a clip or a network probe may behave differently.
- **`VIDEO.VID` is 629 MB**, over the staging limit: the shell's clips are copied into an
  archive of their own on the user's computer (done: `shell-videos.vid`, design §1), the computer
  online for it; T1b3b found no clip beyond it (design §5.9).
- **Inert objects on the listed maps** (2026-11-16): all 160 shipped maps import and validate
  (8 at M0), but the objects the adventure lane has not built yet stand as blockers (creature
  banks on 148 maps among them); the list shows every map as the original's (decision 44), so a
  game's begin starts maps that cannot yet be played as the original's (every map's game builds
  at the census's seed; M3's begin shows an engine refusal should one come).
- **Unknown size:** the screens are listed from the code at M1; M2 to M4 are a sketch until then.
