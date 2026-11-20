# E27 — The game shell: main menu and the screens outside play

Status: **refined** (2026-11-12, M0, with the user); **M1 closed** (2026-11-13), **M2 closed**
(2026-11-17), **M3 closed** (2026-11-19, T7: the fresh reviews, decisions 66 to 68); **M4 refined**
(2026-11-19, M4 T0 at M3 T7, decision 69); M4 T1a, T1b and T2 done (2026-11-19, decision 70);
T3 cut in two (decision 71), T3a and T3b done (2026-11-20); next **M4 T4** (`docs/lanes/client/QUEUE.md`).
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

The user, 2026-11-17 (M3 T1b2's start):

49. **T1b2 whole** (offered: the popups and the teams dialog first with GENRLTXT 503's harness
    step after, T1b2 whole, or the popups alone): one task. GENRLTXT 503 then proved a network
    game's alone (design §20.3; the user told): no harness step, the item to the network modes
    (`CLT-STUB-6`).

The user, 2026-11-17 (M3 T2's start):

50. **T2 cut in three** (offered: T2a and T2b now with the scroll bar a task of its own, T2a
    alone, T2 whole, or the scroll bar a stub): **T2a** the difficulty's presses, the rating, the
    opening's difficulty and the victory's text (`scn-difficulty`, `scn-info`); **T2b** the empty
    list's panel, its teams' dialog and Begin's messages (`scn-begin-none`, `scn-begin-filter`, the
    new `scn-empty-filter`), both in one session; **T2c** the description's scroll bar, which no
    shipped map reaches (a map of the harness's own with a longer text).
51. **The empty list's leftovers** (CLT-Q139 (b)): over an empty list the client draws no flags
    and opens no teams' dialog (the original's keep the last map's players, or with no map at all
    one team of the eight colours) and always shows the chosen difficulty (the original's none with
    no map at all until a press); no rating, as the original's. The original's for the comparisons
    only. Built in T2b.
52. **The list's keys and a long description's bar** (CLT-Q140 (b), 2026-11-18): while the list
    shows, the client's description bar takes none of the list's keys (Up, Down, Page Up, Page
    Down), Left and Right always; with the list hidden the list's keys step it. The original's
    (the bar steps at the key's release after the list moved at its press) for the comparisons
    only. Built at T3a's start.
53. **T3 cut in three, by recipe** (offered: three by recipe, two by drawing and choices, two by
    recipe; the user, 2026-11-18): T3a the panel, the rows, the town's and the bonus's arrows and
    popups (`adv-single-town`); T3b the hero's arrows and popups, the computer's rows
    (`adv-single-hero`, `adv-single-maps`); T3c the flags, the name's edit, the handicap, the
    duration, the hints (`adv-single-rest`, §6.4's `popup-hero-arrow` recorded again on row 2).
54. **T3b cut in two** (offered at its start, the user, 2026-11-18): T3b1 the engine's offer (each
    map's import, `GameSetup::offer` by faction), the hero's arrows and pick, a hero's small
    portrait and name in the row, Random's popup, CLT-Q112's rows (`adv-single-hero`,
    `adv-single-maps` but the hero popups' four shots); T3b2 a hero's own popup
    (`scenario_hero_popup` 0x575A90 read whole).
55. **A main hero of its type's portrait shows that hero** (CLT-Q141 (b), the user, 2026-11-18):
    the original keeps the file's 0xFF as −1 and its row shows None (Random under a Random town),
    its name dropped (52 main heroes of the shipped maps); the client shows the type's portrait and
    the map's name, else the type's, its right press the hero's popup (T3b2); the original's for the
    comparisons only. Built at T3b1.
56. **A pick named by its own name** (CLT-Q142 (b), the user, 2026-11-18): with a flag and a named
    main hero the original names every pick by the main hero (no shipped map reaches it); the
    client names a pick by the map's customised name, else its type's; the original's for the
    comparisons only. Built at T3b1.
57. **T3c cut in two** (offered at T3b2's end, the user, 2026-11-18), at `adv-single-rest`'s own
    seam: T3c1 the flags' hover and press (single), the handicap (single: shown, no press) and the
    name's edit (the recording's first ten shots, `flag-hover-own` to `name-after-flag`); T3c2 the
    duration's bar's input and texts and the items' hints (the other 29).
58. **The name's edit drawn clean** (CLT-Q143 (b), the user, 2026-11-18): the original's opening
    of the single player's name's edit pastes the bar's piece at the bar's corner (the name box's
    frame doubled) and a row's draw shows the name again under the open edit; the client's name
    box stays clean and the name hidden while the edit is open, the original's for the
    comparisons only. Built at T3c1.
59. **T4 cut in two** (the agent's recommendation at T4's start, the user, 2026-11-18; offered:
    the cut, whole, or by recipe): the hot seat's path into the screen is Single's
    (`scenario_new` after the Multiplayer dialog) and M1 read every hot-seat branch, so T4 is
    building: T4a the screen as it opens (the entry from the names, two to eight human records,
    the list's filter, the rows, Back; exact against `adv-hotseat` and both panels), T4b the
    presses (the handicap's, the flags' cycle, a hero another player holds, the name's press,
    Begin's GENRLTXT 83 and 531; the other 17 shots of `adv-hotseat-rows` and `adv-hotseat-two`).

The user, 2026-11-18 (M3 T4b's start; `CLT-Q144`, `CLT-Q145`; T4b kept whole):

60. **The hot seat's handicap caption follows the player** (CLT-Q144 (b)): the original sets a
    colour's handicap caption only at its press, so a player a flag moves keeps its handicap while
    the rows keep the captions pressed there; the client's caption is the handicap of the record
    holding the colour, the original's for the comparisons only. Built at T4b.
61. **More names than colours: kept as is** (CLT-Q145 (c)): the hot seat lists every map two
    humans may play (the original's), and the client's Begin refuses while a named player has no
    colour (decision 22), so a map with fewer colours a human may play than names is listed but
    never begun in the client's profile.

The user, 2026-11-19 (M3 T5's start):

62. **T5 cut in two**: the random map's tab is five functions still to read and 52 shots plus a
    Begin to record, so T5a the tab drawn and its left presses (the toggle, the items, every
    choice with its caps, the levels, the own record in the panel, CLT-Q115's size mark; exact
    against `adv-random` from `tab` to `monsters-random`), T5b the rest (the right press and
    hints, Show Random Maps, the tab toggled back with CLT-Q111's record, Begin after `tab-again`
    recorded: a stub's message).

The user, 2026-11-19 (M3 T5b's start and reading; T5b kept whole):

63. **The random map's teams 1 and 2 taken 32 high** (CLT-Q146 (b), "fix if the button sprite is
    actually that large": their frames are buttons drawn to their 32nd row): the client's profile
    takes the mouse over the whole frame; the original's 24 rows for the comparisons only.
64. **The list shown after a random map gives the game its selection** (CLT-Q147 (b)): the
    client's list showing over a chosen random map, and the files' list back after Show Random
    Maps, show the selected map whole (its description, conditions and flags), where the
    original's keeps the random map's description or flags under the selection's name; the
    original's for the comparisons only.

The user, 2026-11-19 (M3 T6's start):

65. **T6 kept whole** (offered: T6a the setup and the stub, T6b the loading bar; or the bar
    first): one session and one pull request.

The user, 2026-11-19 (M3 T7's start):

66. **T7 kept whole** (offered: the closure now and M4's refinement in a later session; or the
    reviews and fixes first, the replay, the archive and M4 later): one session, as M2 T8.
67. **M3's review list accepted with the reviewers' changes, fixed in T7** (offered: bugs only,
    or the fixes as M4's first task): items 1, 4 (into 12), 9, 13, 16, 20, 21 and 22 reworded,
    Page Up's negative index and the list's ASCII fold added (`E27-archive.md`); the fixes of the
    three reviews in T7: a row or name held while a box opens let go without its click, the list's
    keys dropped under a held button (read: its own loop, 0x456341), a double press only on the
    same map, Begin taking the open edit's name, CLT-Q93 (b) built (decision 13), a random map's
    seats with the generate flag, the begin's guard on a computer-only colour, the window's
    `begin` word refused, the stubs' marker, the stale comments; a file `h3_map::read` refuses
    left out unlisted and the list's held keys unrecorded (M4's, its load mode's list).
68. **The oddities the review found** (each as recommended but CLT-Q151): the description's
    scroll kept while the map stays (CLT-Q148 (b)); Up and Down bring the page to the selection
    first (CLT-Q149 (b)); a row takes the press over its whole width (CLT-Q150 (b)); a map's name
    and description whole in the client's profile, the record's 60 and 300 bytes for the
    comparisons (CLT-Q151 (b)); the advanced options' rows made again whenever the map given to
    the game changes (CLT-Q152 (b)); a bonus kept across a map change, E10 told (CLT-Q153 (a)); the
    random map's count above its cap copied (CLT-Q154 (a)). Built in T7.
69. **M4 refined** (M4 T0 at M3 T7): M4 the screens around play but outside it, a new **M5 in
    play** (the system options and the menu in play, Save from play, Restart, the end of a
    scenario) refined once the shell hosts play (E24 M5), so decision 7's four milestones are five
    (CLT-Q155 (a)); a save is the engine's adventure replay extended with the setup's choices (each
    colour's human flag, town and hero pick: the adventure lane's format, its inbox told) and a
    table of the client's own (the names, bonuses, handicaps, the turn's duration) (CLT-Q156 (a));
    M4's task table as proposed (below).
70. **T1 cut in two** (offered at its start: the dialog drawn and pressed, then Reset's question
    and the scores file; or whole): T1a the dialog, both tabs, the buttons and keys, the rows from
    `CampHigh.txt`'s defaults and the creatures' steps, exact against `scores-tabs`; T1b Reset's
    question and the client's own scores file, exact against `scores-reset`. A score above the
    creatures' table (32 767) shows the last pair's creature (CLT-Q157 (a); the original reads
    past the table).
71. **T3 cut in two** (the user, 2026-11-20, at its start: the adventure lane had not built the
    format's extension of CLT-Q156 (a)): T3a the engine's adventure replay format v20 with the
    setup's choices, built by this lane `cross-lane` with the user's consent (the adventure lane's
    `h3-sim` `replay.rs` and `h3-files` format, its 104 golden replays; the AI and server lanes'
    literals); T3b the client's own save file and a load, on top of it.
72. **T4 cut in two** (the user, 2026-11-20, at its start, as offered): T4a the load mode's
    reading and recordings (the harness and the docs, no client code; design §38), T4b the
    client's load mode and saves exact against them.
73. **T4b cut at Delete** (the user, 2026-11-20, at its start, as offered): T4b1 the load mode's
    screen and the client's saves listed (the folder and the names, the map's header, the date,
    the save's difficulty and rating), the keys and a held key's repeats, the sorts, Load's and a
    disabled difficulty's popups, exact against `scn-load` up to `easy-pressed`; T4b2 Delete's
    question and its end, the notice with no saves, Load's end at play's stub, exact against the
    rest of `scn-load`, `scn-load-last` and `scn-load-none`.
74. **T4b2 kept whole** (the user, 2026-11-20, at its start; offered: whole, or cut after
    Delete).
75. **T5 cut in two** (the user, 2026-11-21, at its start, as offered): T5a the hot-seat load's
    reading and recordings (the harness and the docs, no client code; design §41), T5b the
    client's hot-seat load and saves exact against them, its cut confirmed at its start.
76. **T5b cut in two; a hot-seat load's names and its Random Map** (the user, 2026-11-21, at
    T5b's start, as offered and recommended): T5b1 the hot seat's load mode from the Multiplayer
    dialog, the list's filter, the panel's buttons and the flags' strip, the advanced options from
    the save with the names placed, `h3-view save` for a hot seat, exact against `mp-load`,
    `adv-load`'s screen, options and list shots and `adv-load-more`; T5b2 the presses that change
    nothing, the flags' presses and the popups, exact against the rest of `adv-load`. CLT-Q160
    (a): the names decide who is human after a hot-seat load, as the original's (the engine's
    part, the humans changed over the loaded game, E24 M5's). CLT-Q161 (b): no Random Map button
    in a load in the client's profile, the original's (shown, its press and key doing nothing)
    for the comparisons only.

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
  original reaches it (T1b: the adventure screen alone; M5's with the menu in play, decisions 44 and 69). From E17 (placed at its M7 T4): the volume settings (0x5996C0) and a
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
| M3 | (closed 2026-11-19) **A game's setup:** scenario selection, the players' setup, the advanced options, the game's begin (the filled `GameSetup` kept for E24 M5, CLT-Q135; the stubs of decision 6) |
| M4 | (refined 2026-11-19) **Around play, outside it:** High Scores, Credits, the save file (the engine's record with the setup's choices, decision 69), Load Game single and hot seat |
| M5 | **In play** (decision 69; refined once the shell hosts play, E24 M5): the system menu in play and Options, one dialog (with the map-client lane; decision 44), Save from play, Restart, the end of a scenario's screens |

### M1 — reading and captures (closed 2026-11-13)

Tasks T1a1 to T3 and the review list (accepted at T3, decision 22): `E27-archive.md` (verbatim);
the build record: `E27-log.md`; the readings and captures: `E27-design.md` §1 to §6 (§6.4: what M2
to M4 still read or record). The shell's recipes (`clients/capture/shell/`) gave T2's kept digests
back at T3, in a fresh sandbox, every recipe but the two full starts.

### M2 — the start and the menus (closed 2026-11-17)

Tasks T1 to T8 and the review list (accepted at T8 with its changes, decisions 42 and 43):
`E27-archive.md` (verbatim); the build record: `E27-log.md`; the readings: `E27-design.md` §7 to
§17. The shell's frame, its start, menus, stubs, Multiplayer dialog and edits are M3's ground.

### M3 — a game's setup (closed 2026-11-19)

Tasks T1a to T7 and the review list (accepted at T7 with its changes, decisions 67 and 68):
`E27-archive.md` (verbatim); the build record: `E27-log.md`; the readings: `E27-design.md` §18 to
§33. The scenario screen, its list, the advanced options, the random map's tab and the begin are
M4's ground (its load mode reuses the list and the options' rows).

### M4 — around play, outside it (refined 2026-11-19, M4 T0 at M3 T7)

Decisions 8 and 69 bind it: the screens around play that need no play (High Scores, Credits, Load
Game), a save the engine's adventure replay with the setup's choices and the client's own table
(CLT-Q156 (a)); what play reaches (the options, Save, Restart, the end of a scenario) is M5's
(CLT-Q155 (a)). Each screen exact against its group's shots, each of design §6.4's recipe fixes
recorded by the task that builds its screen. The readings: design §2.8, §2.9, §3.1 to §3.6 (the
load mode), §3.12 to §3.14 (a load's rows).

| Task | Scope | Status |
|---|---|---|
| T1a | High Scores (§2.8, §34; decision 70): the dialog, both tabs and their buttons and keys, the rows from `CampHigh.txt`'s defaults, the eleven creatures stepping (the adventure map's DEFs by `high_score_creatures`), M2's stub lifted, Reset a client's notice (CLT-STUB-14); exact against `scores-tabs` | done 2026-11-19 (`scores-tabs` 28 of 28 exact; CLT-Q157 (a)) |
| T1b | High Scores' Reset (§2.8, §35): its question (GENRLTXT 667) resetting both tables to the defaults, the client's own scores file (read at the start, written at Reset; never `HiScore.dat`), CLT-STUB-14 lifted; exact against `scores-reset` | done 2026-11-19 (`scores-reset` 10 of 10 exact, `scores-tabs` 28 of 28; CLT-Q158 asked) |
| T2 | Credits (§2.9): `Credits.fnt`, `Credits.txt`'s two records wrapped to 328 px, the scroll of 2 px ten times a second (CLT-Q88), the closing text's moment read from the kept frames, a key (but F4) or a press ending it, M2's stub lifted; exact against `credits-scroll`, `credits-end` | done 2026-11-19 (`credits-scroll` 5 of 5 exact, its scroll's 2 611 frames; `credits-end` 8 of 8; design §36) |
| T3a | The save file's engine part (decisions 69, 71; `cross-lane`): the adventure replay format v20, `Replay::players` (each player's human flag, faction's choice and hero's pick, recorded from the setup, set over the import by `Replay::setup` when verifying, none the import's), its `[[players]]` tables in `h3-files`' file, the 104 golden replays recorded again (only the version and the players' lines moved) | done 2026-11-20 (the adventure lane's inbox: its increment 14 takes v21) |
| T3b | The save file (decisions 69, 71): the client's own file, its table (the names, bonuses, handicaps, the turn's duration, the save's name) beside the engine's replay of T3a, written and read with its version; a load's setup rebuilt (`Replay::setup` over the map's import) and its commands replayed by the engine; tests, no screen | done 2026-11-20 (`h3-battle-client`'s `save`; design §37) |
| T4a | Load Game single's reading and recordings (§38; decision 72): the date's order, the load mode's name, Delete's end read; the original's saves of shipped maps made by maker recipes (the user's are a random map's), a pinned time zone, a held key's repeats (`h3shell.py`); `scn-load` recorded again (§6.4's fixes: a day above 12, three sizes, the held key), `scn-load-last` new (Delete of the last save) | done 2026-11-20 (25 and 9 shots, twice the same; `scn-load-none` as kept) |
| T4b1 | Load Game single's screen (§3.1, §3.3 to §3.6, §38, §39; decision 73): the load mode (title, no panel buttons or flags, Load, the save's difficulty on the disabled buttons and its rating), the client's saves listed (the folder `games`, `<name>.GM<n>`, the map's header from the map the replay names, a save it cannot list as a broken file, the file's local time), the keys and a held key's repeats (the viewer's `key-hold`, the window's), the sorts with CLT-Q138's stale buttons and rating, Load's and a disabled difficulty's popups, a right press without a popup redrawing nothing; the viewer given client saves made as the makers' (`h3-view save`, the makers' `viewer`) and `games-time`; Load and Delete the client's notice (CLT-STUB-15) | done 2026-11-20 (`scn-load` 19 of 25 exact: all but Delete's four, the load's clip and the map) |
| T4b2 | Delete's question (688) both ways and its end (the version sort in the last direction, the first selected, the stale buttons and rating), the notice with no saves (686, Load disabled), Load's end at play's stub (as M3's begin: `save::load`'s `Loaded` kept for E24 M5; a save the load refuses its call), CLT-STUB-15 lifted; exact against `scn-load`'s Delete shots, `scn-load-last`, `scn-load-none` (`scn-load`'s `load` clip and `map` are play's, M5's) | done 2026-11-20 (`scn-load` 23 of 25 exact, `scn-load-last` 9 of 9, `scn-load-none` 5 of 5; design §40; CLT-Q159) |
| T5a | Load Game hot seat's reading and recordings (§41; decision 75): the slots, the names' placing (the save's humans first; CLT-Q160), the begin after a load, the save's extension, the list's filter, the options in a load (the duration's bar and the handicap disabled), Random Map in a load (§6.4; CLT-Q161); two hot-seat makers (`Pair.GM2`, `Four.GM4`: every colour human with its choices), `adv-load` recorded again over them, `adv-load-more` new | done 2026-11-21 (19 and 2 shots, twice the same; CLT-Q160, CLT-Q161 asked) |
| T5b1 | Load Game hot seat's screen (§3.12 to §3.14, §41.4; decision 76): the hot seat's load mode over the Multiplayer dialog (the notice with no saves), the list's filter, the panel's buttons and the flags' strip (no Random Map in the client's profile: CLT-Q161 (b)), the advanced options from the save (the saved choices, no arrows, the handicap and the duration's bar disabled, the names placed: CLT-Q160 (a)); `h3-view save` for a hot seat, the makers' `viewer` tables; exact against `mp-load`, `adv-load`'s screen, options and list shots, `adv-load-more` | after T5a |
| T5b2 | The hot-seat load's presses (decision 76): the presses that change nothing (the arrows, the duration's bar, Random Map and its key), a flag's press among the save's humans (CLT-Q110 (b)), the popups; exact against the rest of `adv-load` | after T5b1 |
| T6 | M4's closure: a fresh reviewer, M4's review list to the user, M4 to the archive, M5's refinement once the shell hosts play (else its prerequisites recorded) | after T5b2 |

**M4's review list** (the agent's calls, *provisional*, for T6):

1. (T1a) High Scores' layout `[high_scores]`: the creatures' pairs as the executable's numbers,
   the columns by tab, `drawn` (the buttons' canvas for the original's stale edges).
2. (T1a) A press's stale edges (§34.2) taken as CLT-Q118's class (decision 22: the client draws
   the frame whole, the original's profile for the comparisons), not asked again.
3. (T1a) Exit's command (`scores-exit`: Enter and Escape) ends the dialog at its key down whatever
   key the user binds; C and S press their buttons as hotkeys.
4. (T1a) The creatures' DEFs loaded at the start for the tables' rows alone (T1b's file and M5's
   entry load what theirs need); a creature without a template shows none.
5. (T1a) Reset shows the client's notice until T1b (CLT-STUB-14), the replay stopping under it
   (lifted at T1b).
6. (T1b) The scores file (§35.3): `scores.toml` in the working directory (`--scores` names
   another), TOML with a version, both tables of 11 rows; a file the window cannot read stops it
   with the path and the reason, as the settings file; a name longer than the original's 40 bytes
   kept whole.
7. (T1b) A failed write of the scores file told on standard error, not the original's
   `file_error_box` (GENRLTXT 11) over the dialog.
8. (T1b) The close of Reset's question by a key runs the step due as the mouse's release does
   (§35.2; only the mouse recorded).
9. (T1b) The replay takes `press`, `point` and `until-button` on a front's original box alone
   (`Front::question`: High Scores' question so far), at the probes' point by the box's layout,
   not by a search of the frame.
10. (T2) The credits' clock (§36.4): one step a reading, the first 100 ms after the opening, each
    next 100 ms after the reading that ran the last (the original's on the harness's frozen
    clock); the window waits 5 ms at a turn while they show; a late window is not caught up.
11. (T2) The background's key pixel (§36.3, (463, 553) black during the credits) taken as a
    drawing bug (E17's binding decisions, CLT-Q138's class), not asked: the client's profile
    draws the background whole, the original's for the comparisons only.
12. (T2) `h3_ui`'s two new draws, `Draw::Part` (the text's picture made whole at each draw, 328 ×
    4 774: no cache) and `Draw::Keyed` (the original's profile); a part's source rows past its
    picture not copied (the original reads past it: no caller does).
13. (T2) The text driver names the lost and won screens' task E27 M5 (decision 69's M5).
14. (T3a) A replay records every player's choices from the setup it was recorded from (the
    golden replays' import's included), not only the ones a scenario screen changed: the import a
    verifier is given then decides nothing of them; a replay naming none of them (the server's
    whole-game journals, a file before v20) is the import's.
15. (T3a) A choice of a colour the import has no player of, or of a colour twice, is refused as
    `ReplayError::Incompatible("players")` before the game is built; the game itself checks the
    rest (an unoffered pick, a faction outside the player's).
16. (T3a) The file names a faction's choice by the faction's index or the word `random`, a pick by
    the hero type's index; the server's journals and its measure write none (the server lane's
    inbox: its seats' choices are its to carry).
17. (T3b) The save's file (design §37.1): one file, the engine's replay text whole, then the
    client's table from the line `[save]` on (its version 1; eight `[[save.colours]]`, a name for
    a human's, the bonus by its word); the part before that line is an engine replay file. The
    date not kept (the file's last-write time, as the original's list), nor the map's header (the
    map the replay names, T4's).
18. (T3b) The replay names its map by the file's name in the installation's `Maps`: a save loads
    only over the same map's bytes and import version and under the same rules (the replay's
    header: the packs, the profile, the rules' hashes, the state hash versions), so every engine
    change of the rules or of a hash version leaves the saves before it refused, the field named
    (T4 lists them; the original lists a save of another version as a broken file, §3.3).
19. (T3b) The load applies the commands itself through `h3_game::apply` with `Replay::verify`'s
    checks in its order (the engine's `verify` keeps no state; no engine change: a `Replay`
    function returning the state, the adventure lane's file, would replace the client's loop).
20. (T3b) The load refuses a save making a colour human where the map lets none play (the
    import's mark, which the engine cannot check; the begin's rule of M3 T7) and a name for a
    colour no human holds; a save file is read whole, with no size limit (as the settings and
    scores files).
21. (T4a) The load's recipes use saves the original made of shipped maps (three maker recipes,
    design §38.2: Middletown at Hard, Arrogance at Easy, Dawn of War's autosave at Expert on its
    second day) in place of the user's (a random map's, which no client's save can name); the
    makers run when their saves are missing or their recipe changed, no digest of a save kept
    (its bytes differ by a few heap pointers from run to run).
22. (T4a) After a sort in the load mode the partial redraw leaves the difficulty buttons and the
    rating stale too (the rating's text drawn over the last one; design §38.3), taken as
    CLT-Q138's class (decision 48: the client draws whole, the original's for the comparisons),
    not asked again.
23. (T4a) Every shell run's time zone pinned (`tz`, UTC by default; the load's recipes Asia/Kolkata,
    a zone without daylight saving time: the original shows a file's time by the present
    offset); the display's auto-repeat off at each run's start, on only for `key-hold`, whose
    repeats' count follows real time (a recipe holds a key until the list's end, so it does not
    matter).
24. (T4b1) The client's saves (design §39.1): the folder `games` in the working directory (the
    window's `--games` names another), a save's file `<name>.GM<n>` as the original's (the rows
    show the same names), up to 5 000 files of `*.gm?` read in their names' order; the folder read
    again at each opening of the load mode (the window), a replay's once.
25. (T4b1) A save the client cannot list (it does not read, its replay's header the rules
    refuse, its map missing or moved) shows as the original's broken file (GENRLTXT 431 and 505,
    no map's columns, difficulty Easy); the full load (the commands replayed) is Load's, T4b2's.
26. (T4b1) A save's date at the zone's present offset from SDL (`h3_sdl::local_offset`:
    `localtime_r`'s `TZ` on Linux, Windows' zone), as the original's `FileTimeToLocalFileTime`;
    no time-zone crate.
27. (T4b1) A right press without a popup redraws nothing (`dialog_right_click` 0x5FF790 read:
    the redraw follows a popup alone), but on the description's box (its scrolling text,
    `scn-long-text` right-bar) and on the players' items 362 to 385 (their own cases redraw);
    M3's "redrawn whole" for the others was an inference no recording told apart.
28. (T4b1) The shell's window takes a held key's repeats on the scenario screen alone (the
    original queues every key down everywhere; the other screens' keys were not recorded held);
    the viewer gives `key-hold`'s presses by Xvfb's rate (1 under 660 ms, then one each 40 ms).
29. (T4b1) The autosave known by its file's name: the panel's name GENRLTXT 77 where the file
    begins with that text, the name's sort's lead where it begins `AUTOSAVE.`, both any case (the
    original's compares' case not read).
30. (T4b1) The made saves (`h3-view save`, a maker's `viewer` table, left out of its digest): the
    single player's colour the map's, the advanced options untouched, seed 1, every player's End
    Turn until the day; the computers make no moves (the list shows no more of them).
31. (T4b2) Load's end: the save read and loaded by the engine at play's start (`save::load`, the
    loading screen's bar stepped by the read and the load), the game kept for E24 M5; play's
    notice in the client's words (`shell.stub.loaded`, or `shell.stub.load-refused` with the error
    on standard error) where the original plays. A save the list shows broken is loaded and
    refused there too (the original's load of a broken file not read).
32. (T4b2) Delete deletes a listed file of the client's own folder alone (`saves::delete`: a plain
    name of the pattern, a file); a failure is told on standard error and the entry leaves the
    list all the same, as the original's unchecked `DeleteFileA` (CLT-Q159).
33. (T4b2) A disabled button takes no hotkey on the scenario screen (recorded for Load: `scn-load-last`
    and `scn-load-none` enter; the other screens' disabled buttons not recorded).
34. (T4b2) The viewer's replay keeps the saves Delete left for the next opening (the original reads
    its folder again; the window lists its folder anew at each opening).
35. (T5a) The hot-seat load's saves made by the original from shipped maps with every colour
    human and its town, hero and bonus chosen by the arrows (`Pair.GM2`, `Four.GM4`), so the
    rows hold no random draw a client's save cannot repeat; a single game's save under two names
    (`adv-load-more`) with its rows' drawn cells masked instead.
36. (T5a) The hot-seat makers' `viewer` tables give the difficulty alone until T5b's `h3-view
    save` makes a hot seat's (`drawn` stops at the hot seat's load, before any shot, meanwhile).

## Dependencies and boundaries

- **E17 closed** (2026-11-12): the compositor, SDL adapter, fonts and string catalogue are used as
  they are; this epic adds screens, not a second renderer. `h3-bink` decodes the Bink clips.
- **E24 M5 (hot seat)** starts its game from the filled `GameSetup` (decision 4); what else the
  hand-over carries (the hot seat's names, the map's file) is agreed with the map-client lane at
  M3's T0 within the existing crates. The system menu in play (M5, decision 69) draws over E24's screen.
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
- **Unknown size:** the screens are listed from the code at M1; M2 to M5 are a sketch until then.
