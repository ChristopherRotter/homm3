# E27 — The game shell: main menu and the screens outside play

Status: **refined** (2026-11-12, M0, with the user); M1 T1a1 and T1a2 done; T1b cut in three
(decision 11); next **M1 T1b1** (`docs/lanes/client/QUEUE.md`).
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
  their clips and messages, the high-score entry; the effects behind E10, decision 6).
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
| M1 | **Reading and captures:** the catalogue of the original's screens, each with its entry points, transitions, assets, texts (table and record) and sounds; the clips cut from the user's `VIDEO.VID`; the harness on every screen and its captures; the traps list; the questions the reading finds |
| M2 | **The start and the menus:** the Smacker decoder (decision 5); the clips at start, the loading bar; start-up loading by the LODs' tables (CLT-Q2); the main menu, New Game, Load Game's choice, Multiplayer and the hot seat's names, Quit's question, their hover, press, sounds and music, exact against the captures; the shell's state machine with a headless driver (the text client first, E17's pattern) |
| M3 | **A game's setup:** scenario selection, the players' setup, the advanced options, the game's begin (the filled `GameSetup` to E24 M5; the stubs of decision 6); Options |
| M4 | **Around play:** Save and Load (the engine's record), the system menu in play (with the map-client lane), the end of a scenario's screens, high scores, credits |

### M1 — reading and captures (refined 2026-11-12, M0)

| Task | Scope | Status |
|---|---|---|
| T1a1 | The catalogue, part 1a: the start (the clips' order and skipping, the loading bar), the main menu, New Game's and Load Game's menus, Quit | done 2026-11-12 (design §2; CLT-Q84 to CLT-Q86) |
| T1a2 | The catalogue, part 1b: Multiplayer (its modes, decided as a `CLT-Q`) and the hot seat's names, High Scores, Credits | done 2026-11-12 (design §2.6 to §2.9; CLT-Q87 to CLT-Q89) |
| T1b1 | The catalogue, part 2a: the scenario screen's frame, its list and information panel in its three modes (new, load, save), the difficulty, Begin and Back (decision 11) | ready |
| T1b2 | The catalogue, part 2b: the advanced options (the players' setup to `GameSetup` or an E10 gap), the random map's tab, the game's begin | after T1b1 |
| T1b3 | The catalogue, part 2c: the system options and the system menu in play, the end of a scenario (victory, defeat, their clips and messages, the win screen, the high score's entry) | after T1b2 |
| T2 | The shell's clips cut from the user's `VIDEO.VID` (on the user's computer, as `battle-videos.vid` was); the harness on every catalogued screen; the reference captures; the traps | after T1b3 |
| T3 | M1's closure: a fresh reviewer, the review list and the questions with the user, M2's refinement (M2 T0) | after T2 |

**M1's review list** (the agent's calls, *provisional*, for T3):

1. Not built (T1a1, design §2.10): the CD status, its notices and the menus without Single,
   Campaign and Tutorial; the free disk space check; a DirectPlay lobby's start; the single
   instance; F4's window mode (E29).
2. T1a2 (design §2.10): the high scores in the client's own file, never the installation's
   `HiScore.dat` (not imported), the defaults read from `CampHigh.txt`; the player's name
   (`Network Default Name`, the registry's) a client setting; the leftover `RMT*SC.BIN`'s deletion
   not built.

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
  archive of their own on the user's computer (T2), the computer online for it.
- **The engine imports 8 of the 160 shipped maps whole** (2026-11-12): the scenario list either
  shows maps that cannot start yet or differs from the original's (decision 8: M3's T0).
- **Unknown size:** the screens are listed from the code at M1; M2 to M4 are a sketch until then.
