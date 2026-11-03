# E27 — The game shell: main menu and the screens outside play

Status: **draft** (the user, 2026-11-03; not refined, no task started) · Lane: **client**, assigned once E17 closes
(E24's hot seat, M5, consumes the scenario screens; the map-client lane is consulted) ·
Follows E17 (the battle client: its compositor, SDL adapter, string catalogue and capture harness)

## Goal

Every screen of the original that is not play itself, recreated truthfully to the pixel and the
frame, as E17 does for the battle and E24 for the adventure map: from the first frame after
the start, through the main menu and the screens it opens, to a game's begin and, after play,
the way back. A player starts the client, picks a scenario, sets up a hot-seat game and plays it
(E24 M5), with no tool from the harness in between. The scope here is the **faithful
recreation**; the lobby, the ladder and the server browser come later (below).

## Decisions (the user, 2026-11-03)

1. **A new epic**, not a milestone of E17 or E24: the screens outside play have no home in
   either (E17's "Later" lists the client's options and video; E24's M5 only the handover and
   the hero, town and kingdom screens "as play needs them").
2. **Assigned to the client lane once E17 closes** (after M6, and M7's own decision).
3. **Faithful recreation first.** Lobby, ladder and server browser integrations are a later
   increment, after this epic's screens exist (E17 M7 and E21 build on them). Their look is new
   design, as E17's "Later" already says.

## Scope (to be read from the original at T1; names are the original's as far as known)

Each screen is read from `Heroes3.exe` and its assets (`docs/exe/map.toml`), captured under
the harness as a reference, and compared at the pixel. Where the editions differ (RoE, AB,
SoD) the shipped installation decides; E11 owns historical differences.

- **The start:** the logo and intro videos (video is E17's "Later"; here only the decision of
  what the client plays), the main menu and its buttons.
- **New Game:** the choice of single scenario, campaign, multiplayer, and the tutorial's place.
- **Scenario selection:** the list with its filters and sort, the scenario's information, the
  difficulty, the players' setup (colour, hero, town, starting bonus, handicap, teams), the
  advanced options, the map's description and the preview. The E17 design's hot-seat flow
  (design §6.3) is the start of the reading.
- **Multiplayer's hot seat:** the names' dialog; the other modes (direct connection, modem)
  are listed and decided at T1, not built.
- **Load Game and Save Game:** the list, the dialogs, the engine's own save/replay record as
  the file (E19, the original's save-file import, stays apart and low priority).
- **Campaigns:** the campaign selection screen and the in-between screens; the campaign rules
  are E20's, so here only their screens, behind E20.
- **High scores and credits.**
- **Options:** the system options dialog (volumes, speeds, the original's settings), the same
  reached from the main menu and from play's system menu.
- **The system menu in play** (save, load, restart, main menu, quit) and **the end of a
  scenario** (victory, defeat, the statistics and the high-score entry).
- **Sound and music** of the menus (E17's sound decoders).

## Non-goals

- Lobby, accounts, ladder, server browser, the pause by agreement: E17 M7, E16 M2 and E21, after
  this epic's faithful screens (decision 3).
- The map editor and its screens (`h3maped.exe`).
- The campaigns' rules and scripts (E20); the original's save-file import (E19).
- Playing a game: the adventure map (E24), the battle (E17), the town and hero screens (E24 M5,
  E09).
- HD layouts and graphics options beyond `faithful` (E17's "Later").

## Milestones (sketch; refined when the lane takes the epic)

| M | Scope |
|---|---|
| M0 | Refinement with the user: the screens' list as read, the cut into milestones, the lane's ownership of the shell's state machine (below), the `CLT-Q` questions |
| M1 | **T1 reading and captures:** the catalogue of the original's screens, each with its entry points, assets and its capture; the string catalogue's menu texts; the traps list |
| M2 | The main menu and New Game screens: drawn, the buttons' hover, press and sound, exact against the captures; the shell's state machine in a library with a headless driver (the text client first, E17's pattern) |
| M3 | Scenario selection and a hot-seat game's setup, ending in a game record E24 M5 starts from; Options |
| M4 | Load and Save; the system menu in play; the scenario's end screens; high scores and credits |
| M5 | Campaign screens, once E20 has the rules; the shell's hooks for the lobby and ladder (later increment) |

## Dependencies and boundaries

- **E17 closes first**: the compositor, SDL adapter, fonts and string catalogue are used as
  they are; this epic adds screens, not a second renderer.
- **E24 M5 (hot seat)** needs the scenario setup's output: the setup's type is a shared
  contract between this lane and the map-client lane, agreed at M0, `cross-lane`.
- **The engine and the server** own what a game is: the shell builds a game configuration and
  hands it to the engine (ADR-0009's boundary); no rule lives in the shell.
- **Maps and campaigns** are read through `h3-formats` and the installation; the shell never
  writes to the user's installation.

## Open questions for the refinement (to be filed as `CLT-Q<n>`)

1. Is the main menu's list of dead or unreachable entries (the original's multiplayer modes
   without a network, the tutorial) shown greyed, hidden, or wired to our own equivalents?
2. Which saves: the engine's own record only, or also the original's `.GM1`/`.GM2` import (E19)?
3. Where does the shell's state live between screens: a client-side state machine, or the same
   session type the server's lobby will later use?
4. Are the intro videos played, skipped, or replaced (E17's video decision)?
5. Which editions' menus are built: the installation's only, or each of RoE, AB and SoD?

## Risks and known gaps

- **Wine is not Windows** (E17's risk): the captures of menus run on Wine's DirectDraw; the
  menu is the screen E17 already proved exact on, so the risk is small, but a screen that
  runs a video or a network probe may behave differently.
- **Unknown size:** the original's screens have not been listed from the code yet; the
  milestones above are a sketch until T1's catalogue exists.
- **Campaign screens depend on E20**, which is `todo`; M5 may stay open past this epic's rest.
