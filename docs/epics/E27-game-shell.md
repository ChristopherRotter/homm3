# E27 — The game shell: main menu and the screens outside play

Status: **refined** (2026-11-12, M0, with the user); **M1 closed** (2026-11-13, T3: the fresh
reviews, the review list accepted, decisions 19 to 22); **M2 refined** (2026-11-13, at M1 T3);
**M2 T1 done** (2026-11-13, decisions 23 and 24); next **M2 T2** and **T3**
(`docs/lanes/client/QUEUE.md`).
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
   shows (8 of the 160 shipped maps import whole today) is decided at M3's T0.

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
  original reaches it (T1b). From E17 (placed at its M7 T4): the volume settings (0x5996C0) and a
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
| M2 | (refined 2026-11-13) **The start and the menus:** the Smacker decoder (decision 5); the clips at start and their sound; start-up loading by the LODs' tables (CLT-Q2); the main menu, New Game, Load Game's choice, Multiplayer and the hot seat's names, Quit's question, their hover, press, sounds and music, exact against the captures; the shell's state machine with a headless driver (the text client first, E17's pattern) |
| M3 | **A game's setup:** scenario selection, the players' setup, the advanced options, the game's begin (the filled `GameSetup` to E24 M5; the stubs of decision 6); Options |
| M4 | **Around play:** Save and Load (the engine's record), the system menu in play (with the map-client lane), the end of a scenario's screens, high scores, credits |

### M1 — reading and captures (closed 2026-11-13)

Tasks T1a1 to T3 and the review list (accepted at T3, decision 22): `E27-archive.md` (verbatim);
the build record: `E27-log.md`; the readings and captures: `E27-design.md` §1 to §6 (§6.4: what M2
to M4 still read or record). The shell's recipes (`clients/capture/shell/`) gave T2's kept digests
back at T3, in a fresh sandbox, every recipe but the two full starts.

### M2 — the start and the menus (refined 2026-11-13, M2 T0 at M1 T3)

Decisions 20 to 22 bind it: the dialogs' input shared with the battle's (20); the shell's code as
the battle's, `h3-smacker`, the clips' sound exact against the DLLs (21); the menus' buttons
redrawn whole, Enter pressing the names' OK (22). Each screen is exact against its group's shots,
recorded again in the sandbox (`h3shell.py run`), with the sound sequences its task records
(design §6.4). The loading bar is no part of the start (design §2.1): M3's begin builds it.

| Task | Scope | Status |
|---|---|---|
| T1 | The Smacker decoder: `h3-smacker` (decision 5), FFmpeg 6.1.1's `smacker.c` video ported (integer only), the `video` command reading `.smk`; checked frame for frame against `SMACKW32.DLL` under Wine (`smacker-check.sh`, as `bink-check.sh`) on the shell's seven Smacker clips and synthetic samples | **done** 2026-11-13 (decisions 23, 24; CLT-Q119): `h3-smacker` (`Clip`, `Reader`, `Decoder`, `Frame::written`), `video --smacker`, `smackprobe.exe`, `smacker-check.sh`; the shell's eight clips, AB's `Credits.smk` and the samples the DLL's byte for byte, 8-bit and 5-6-5; log "M2 T1", design §7 |
| T2 | The clips' sound (CLT-Q104): Bink's audio in `h3-bink`, Smacker's in `h3-smacker`, the floating point emulated as `h3-bink`'s inverse DCT; checked sample for sample against the DLLs under Wine where the harness reaches them (else against FFmpeg's, a `CLT-Q`); the samples into the window's audio | after T1 |
| T3 | The shell's frame (CLT-Q103): the screens' state machine and the menu loop's requests (design §2.2) in `h3-view`, the `h3-view shell` command with a headless driver (a text client first, E17's pattern); the dialogs' input (a DEF button's own loop, a box's and a bar's press and release, the right press's help) moved from `h3_view::options` into `h3-ui`, the combat options on it unchanged (decision 20); start-up loading by the LODs' tables (CLT-Q2) | ready |
| T4 | The comparison: `h3shell.py drawn` (the shell plays a recipe's inputs at its times; its frames compared with the original's shots, as `h3drive.py drawn`); h3shell's sound (h3drive's `sound`); TCP/IP's address masked in its digests (review item 8); the main menu's first shot exact | after T3 |
| T5 | The start (design §2.1): the clips in order, `H3Intro` framed in `introrim.pcx`, skipping, `Show Intro`, Video Quality's Smacker copies (CLT-Q96); what T2 left unsettled (§6.3: `NWCLogo`'s placement, the end-of-clip call, the screens between the clips) and `game_samples_load` read; exact against `start`; every pixel of a Smacker clip through its palette (decision 24) | after T2, T4 |
| T6 | The main menu, New Game's and Load Game's menus, Quit's question (§2.3 to §2.5): hover, press, release off, popups, hotkeys, `button.wav`, `MAINMENU.MP3`, the buttons redrawn whole (CLT-Q118); Campaign and Tutorial stubs (decisions 7, 9), High Scores and Credits stubs until M4; exact against `menus` with its sound | after T4 |
| T7 | Multiplayer's first stage (decision 10's stub, CLT-Q101) and the hot seat's names (§2.6, §2.7: the edit field read first, two 21-letter names recorded, Enter pressing OK, CLT-Q107), Load Game's multiplayer; the scenario screen a stub (M3); exact against `multiplayer` | after T6 |
| T8 | M2's closure: a fresh reviewer, the review list, M3's refinement (M3 T0: the loading bar, the scenario list's maps, decision 8; §6.4's recipes for M3) | after T7 |

**M2's review list** (the agent's calls, *provisional*, for T8): (1) `h3-smacker`'s shape (T1):
the frames read in order by a `Reader` (the palette is cumulative), a frame's written blocks in
`Frame::written`, the trees walked bit by bit (H3INTRO's 2 426 frames in 4 s in release); (2) the
DLL's 16-bit copy kept in `h3_smacker::check` (`original_16`, the comparisons' only, decision 24)
rather than in `h3-view`; (3) the `video --smacker` outputs `<stem>.pal8` (indices, then the
palette) and `<stem>.565`; (4) the samples left within what the DLL reads as FFmpeg does (both
byte trees, no key frame mark: design §7.3), `MAX_WIDTH` × `MAX_HEIGHT` 800 × 600 as `h3-bink`'s.

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
- **The engine imports 8 of the 160 shipped maps whole** (2026-11-12): the scenario list either
  shows maps that cannot start yet or differs from the original's (decision 8: M3's T0).
- **Unknown size:** the screens are listed from the code at M1; M2 to M4 are a sketch until then.
