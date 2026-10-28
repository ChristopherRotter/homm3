# E24 — The adventure map client

Status: **refined** (2026-10-23, T0, with the user); **M1 closed** (2026-10-24, T6b: the
reviewers, the review list accepted by the user); **M2 refined** (2026-10-26, M2 T0, with the
user); **M2 closed** (2026-10-28, T6b2c: the user's hand test, the review list accepted at
T6b2a); **M3 refined** (2026-10-28, M3 T0, with the user); M3 T1a built (2026-10-28, the
common crate `h3-ui`); next M3 T1b (`docs/lanes/map-client/QUEUE.md`).
Lane: map-client (`docs/lanes/map-client/HANDOFF.md`: the autonomy contract, the user's asset
notes). Opened 2026-10-23 when the user split the client lane at E17 M3's closure (CLT-Q18).
Decisions so far live here (no ADR of its own yet; ADR-0020 and ADR-0024 bind it as they bind
E17). Working detail (code readings, formats, layouts): `docs/epics/E24-design.md`, created by
the first task that has detail to record; done tasks in full: [`E24-log.md`](E24-log.md).

## Goal

The adventure map drawn with the original's own assets, truthful to the original down to the
pixel and the frame as E17's battle is: stills of a map first, then the map moving in a window,
the adventure screen around it, the engine's whole games shown, and whole games played. On E17's
foundation (the decoders `h3-assets`, the compositor `h3-render`, the SDL adapter `h3-sdl`, the
string catalogue) and its capture harness, all common paths of the two client lanes. Battles
inside a game are the battle client's (E17); this epic hands them over.

## Decisions (the user, 2026-10-23, T0; E17's binding here too)

- **E17's decisions hold** (`docs/epics/E17-clients.md`, "Decisions"): exhaustive, automated
  diffs against the original's captures (the harness under Wine, never the installation); the
  original's drawing bugs fixed and reproduced only for the comparisons (`h3_render::Bugs`); its
  redraw artifacts not copied; graphics options per effect, `faithful` the default and the only
  setting the comparisons check; asset tables per expansion, committed, identifiers and numbers
  only; loading at checkpoints, as fast as the original; every text through the string
  catalogue; **every oddity of the original an open question** (`MCL-Q<n>`, the lane HANDOFF's
  autonomy contract).
- **M1 is the map view and the screen's frame** (T0): the panels right of and below the map
  (minimap, hero and town lists, status window, resource bar, buttons) wait for M3, when the
  pieces of `h3-view` they need sit in a common crate.
- **The map moving comes before the panels** (T0): M2 the animated map in a window, M3 the
  adventure screen.
- **Test maps through `h3_map::writer`, extended by this lane** (T0): what the situations need
  and the writer lacks (rivers, a tile's pictures and flags as given, any object template) this
  lane adds to the adventure lane's writer in **additive `cross-lane` pull requests** (the
  user's consent for that kind of change, 2026-10-23), each noted in the adventure lane's inbox;
  nothing the writer writes today changes.
- **The string catalogue stays common** (T0; `clients/text/`, as T9c made it): both client
  lanes add keys.

## How the viewer sees a map (T0's finding; the types since T2: `h3_map_view::art`, `live`, *provisional*)

- **Two inputs.** The engine's import (`h3_map::import`) draws nothing and keeps no pictures: no
  terrain picture, no mirror flags, no template's DEF; it imports 5 of the 160 shipped maps
  whole, and its `ObjectView`s carry no index into the map's objects. The raw map
  (`h3_map::parse` → `RawMap`) keeps all of it: each tile's terrain, river and road with their
  pictures and flags, each template's DEF, masks and priority, each object's anchor. So the
  viewer takes the **static art** from the raw `.h3m` and the **live part** (which objects stand,
  what a random object became, the owners, the heroes, what the viewer has explored, the
  moment) from a second input. In M1 that input comes from the original's memory for the
  comparisons (`h3mem.py`, an additive harness change) or from the map's start where it needs
  no draw; the engine fills it from M4 on.
- **Random objects.** The original draws a shipped map's random objects (monsters, resources,
  artifacts, towns, dwellings) when the game starts, and the harness saw them differ from run to
  run (E17 design §6.3: the piles' amounts): a comparison on such a map reads what the original
  drew from its memory, never assumes the engine's draw (T4b: each object's template in memory,
  the copy a new game made for a random one, `E24-design.md` §2.7).
- **Where the reading starts** (`docs/exe/map.toml`): each map tile keeps a draw list of the
  objects over it; `place_object_tiles` (0x505B20) adds an entry per tile of an object's
  footprint through 0x505230, `object_tile_layers` (0x505060) gives each tile a draw layer from
  the template's masks (a flat template, the special grounds, keeps layer 0), and the objects'
  shadow pass is the encoding 3 drawer 0x47D930 (E17 design §3.3). The view (T1a's reading,
  `E24-design.md` §1): eleven passes over the whole view, tile (x, y) at (32 (x − vx),
  8 + 32 (y − vy)) clipped to (8, 8)–(600, 552), the view's top-left at adventure manager +0xE4.
  The objects (T1b's reading, `E24-design.md` §2): a 32 × 32 piece per tile of each object's
  draw list, the list ordered by its cells' layers; flat objects first, then every shadow, then
  the bodies by layer with heroes and boats as two-row objects (after layers 1 and 2), then the
  path's arrows.

## Milestones

Each milestone ends at a checkpoint with the user, after a fresh reviewer agent checked it. The
autonomy contract (lane HANDOFF) says what the agent decides alone in between. Tasks: the lane's
`QUEUE.md`, each about half a context.

### M1 — stills of the map view (closed 2026-10-24)

Tasks T0 to T6b and the exit record: `E24-archive.md` (verbatim); the build record: `E24-log.md`; the design: `E24-design.md` §1, §2. The set of 40 captured situations (`clients/map/situations/`) is exact; a change to the viewer runs `h3map.py drawn` first (the runbook).

### M2 — the map moving (closed 2026-10-28)

Scope, decisions, tasks T1a1 to T6b2c and the exit record: `E24-archive.md` (verbatim); the build
record: `E24-log.md`; the design: `E24-design.md` §3. Its decisions that bind later milestones
(in the archive): the walk's path always the engine's (M4, M5), never a client pathfinder; M3
T0's common crate for both lanes, the key table moved into it; the edge scroll's 16 pixels
(MCL-Q6). The 13 recorded sequences (`clients/map/sequences.toml`, `timelines/`) are exact; a
change to the viewer runs `h3map.py drawn` first (it checks every timeline too).

### M3 — the adventure screen (refined 2026-10-28, M3 T0, with the user)

The panels right of and below the map view drawn and working as the original's: the minimap,
the hero and town lists, the buttons, the status window, the resource bar and the date, the
status bar's hints; the mouse over the map and the panels with its cursor; on one crate both
viewers share.

- **Scope:** the common crate `clients/crates/h3-ui` (T1a, T1b); the adventure screen's dialog
  (`adventure_full_update` 0x417420's panels, the manager's +0x44 and the dialog +0x50: its
  items' ids, places, pictures and order) drawn at a map's start, the frame's fixed cost taken
  down with it (T2); the minimap (`minimap_draw` 0x412C40: the terrains' colours, the owners',
  the unexplored tiles, the view's box, the underground; T3); the hero list (portraits, the
  movement and mana bars, the selection, the scroll) and the town list (T4); the status window
  (a hero's, a town's, the kingdom's), the resource bar and the date (T5); the mouse (T6): the
  status bar's hints over the panels and the map, the cursor over the map by its tile, the
  clicks below; the keys during a walk (MCL-Q10: Space's cancel of the queued steps, the input
  ring's bound of 63 messages), the panels' hotkeys through the key table; the sound without the
  focus (the inbox's). Every text through the string catalogue's keys. **Not in M3:** the
  dialogs the buttons open (the kingdom overview, the quest log, the options, the spell book on
  the map, the hero and town screens), End Turn, Sleep, Move Hero and every other action of the
  engine's (M4, M5); the engine's live part (M4).
- **Decisions (the user, M3 T0, 2026-10-28; every recommendation taken):**
  - **The common crate `h3-ui`** (T6b2a's proposal in the client lane's inbox, decided by the
    user for both lanes): a crate of the client workspace common to both client lanes, as the
    foundation is (`lanes.toml`: a `shared/` pull request first). It holds `h3-view`'s `clock`,
    `cursor` (with the decoded DEF it draws, `Sprite`), `ui` (with draw items of its own, no
    longer the battle scene's `Draw`) and `text` (the string catalogue and its packs), and
    `h3-map-view`'s `keys` (the table generic over a viewer's commands) and `music` (the music
    thread's model, for the battle's end tune too). `h3-view`'s `layout` and `mouse` stay the
    battle's (the hexes, the battle's and the book's layouts, the battle's cursor; a dialog's
    format goes with `ui`). This lane moves it as T1a in a `cross-lane` pull request (the user's
    consent: the client lane's crate changes by its imports only), nothing drawn or played
    changed, `h3-map-view`'s copies of `clock`, `cursor` and `Sprite` gone; T1b takes `h3-view
    window`'s keys through the table (the client lane's inbox, 2026-10-26), **both windows'
    own keys the map's** (F2 the music, F3 the effects, F5 pause, F6 the next frame, F7 restart,
    F8 vsync, F9 the cursor's kind, F11 fullscreen, Esc quit), the letters left to the
    original's hotkeys (F the Faerie Dragon's, the client lane's M5 T3b).
  - **The panels' live part as M1's** (the epic's two inputs): what the panels show of the game
    (the viewing player's heroes and towns in their lists' order, a hero's movement and mana,
    the resources, the date, the hero or town selected) is `h3_map_view::live`'s, from the
    original's memory for the comparisons (`h3mem.py`, additive) or from the map's start; the
    engine fills it from M4.
  - **The clicks:** what changes only the view works in M3: a hero or a town chosen in its list
    and the lists scrolled, Next Hero, the underground's button, a click on the minimap, the
    status window's own; every other button shows its press and does nothing (its dialog or the
    engine's action: M4, M5).
  - **The cursor over the map** is read and built in M3 from given inputs: what the tile holds
    (the live part) and whether and in how many days the selected hero reaches it (the path's,
    never computed by the client: M2's decision), from the original's memory in a recording,
    from the engine in M4 and M5.
  - **The frame's fixed cost** (the inbox's, the user at M2 T4b3) is taken down in T2, where
    the panels change how a frame is made.
  - **MCL-Q12's provisional part confirmed:** the track asked for again during its own fade-in
    plays on, no second fade.
  - **MCL-Q9 answered** (after its context): the shore's loop on the coast's water pictures only
    (below 20), the original's 21 behind `--bugs original`; built at T0.
- **Recorded and compared as M1 and M2:** `h3map.py`'s `panels` mask shrinks by each panel
  drawn, so the 40 situations and the 13 sequences compare it from then on (`drawn` with them);
  new situations and sequences for what those do not show (several heroes and towns, the lists'
  scroll, the status window's kinds, the clicks, the cursor's kinds).
- **Tasks** (the queue has each in full): T0, M3 refined (**done** 2026-10-28; MCL-Q9's fix
  built); T1a, `h3-ui` and the move (**done** 2026-10-28: #541's `lanes.toml`, the move's
  `cross-lane` pull request; the log); T1b, the battle window's keys through the table (**done**
2026-10-28: every key of both windows through `h3_ui::keys`, by context; the log); T2 cut with
  the user (2026-10-28): T2a, the adventure screen's dialog at a map's start, right (**done**
  2026-10-28: the dialog and the resource bar drawn, the buttons' states, the parked cursor, the
  mask in `clients/map/masks.toml`, the 40 situations and 13 sequences compared; the log), T2b,
  the frame's fixed cost; T3, the minimap; T4, the hero
  and town lists; T5, the status window, the resource bar and the date; T6, the mouse, the keys
  during a walk, the sound without the focus; T7, M3's closure. A task that grows is cut at a
  seam agreed with the user.
- **Exit:** every situation and sequence as recorded on the whole screen (no `panels` mask
  left, or each difference an `MCL-Q<n>`); the view-only clicks, the hints and the cursor in the
  window; `h3-ui` the only copy of what both viewers share; M3's review list accepted; a hand
  test on the user's machine.

### Later (refined when they come)

- **M4, the engine's games shown:** a whole game's record and replay viewer; the live part from
  the engine (`h3_game`'s observation needs an object identity matching the map's object list:
  the adventure lane's inbox), the protocol's adventure messages (the server lane, E16); a battle
  handed to the battle client's viewer (CLT-Q4's background variant and CLT-Q6's colours then
  come from the map); a loaded saved game's draw lists (the objects gone before the save: not
  read, M1 T6b); the map's sound compared by ear with the original's on a whole map (the user,
  at M2 T4b2: the transitions sounded unclean, perhaps the lag's doing; the harness's recordings
  have no sound); the nine step sounds the original ships but never plays (`HORSE20` to `HORSE29`,
  perhaps the roads') as a sound option once their purpose is known (MCL-Q11).
- **M5, hot seat:** whole games played in-process: the handover screen when the player who must
  act changes, the level-up dialog (the inbox), the hero and town screens as play needs them;
  then live play against the server.
- HD layouts, graphics options beyond `faithful`, video.

## Research items from the asset notes

| Item | Where |
|---|---|
| 9. The adventure map's object draw order (the user: neither they nor VCMI got it exactly right); the objects' shadows | **Closed** (T4b): M1 T1b (read, `E24-design.md` §2), T4a (its frames, §2.6), T4b (random objects, shipped maps' views, §2.7); heroes and boats among the layers since T5a (§2.8) |
| 3. Licences of MMArchive and homm3-unpacker before reading their code | stop the line: a question first, when a task wants them |
| 4. The video archives | later |
| 7. The HD patch as a source of fixes and for HD | later |

## Risks and known gaps

- **The original's moment.** A still is exact only at a known game time: whether the harness's
  virtual clock (`xdd.dll`) drives the adventure map's animations and colour cycles as it drives
  the battle's (T3b2: it does); the palette step is read from memory, the menus run in real time.
- **Random objects drawn in real time** (above): the comparisons depend on reading them from the
  original's memory.
- **The engine lags the viewer:** 5 of 160 shipped maps import whole, and the observation has no
  object identity; nothing before M4 needs either.
- **The writer belongs to another lane:** each addition is a `cross-lane` pull request; a
  situation waits for its merge.
- **Wine is not Windows** (E17's risk): a difference only Wine could cause needs a check on
  Windows.
- **The map's input without the engine** (M2): a direction key steps the hero through obstacles
  (the original's `hero_step_allowed` is the engine's rules, M5), a click shows only a path a
  recording or a situation gave (the engine's paths: M4, M5), and no step costs points (the
  walk on after a held key's release has no end but the queue's: MCL-Q10).

## Review lists

M1's review lists (three reviewers, T6b; the list accepted) and M2's (accepted at T6b2a):
`E24-archive.md`. Later milestones add theirs here.

**M3's review list** (the autonomy contract's "queue for the milestone review"; open):

- **M3 T1b, the key file's tables** (*provisional*): one `keys.toml` for both windows, a TOML
  table per viewer (`[map]`, `[battle]`; anything else refused, M2's top-level commands too),
  each window reading its own; the battle's commands named `log-up`, `dragon-spell`,
  `surrender` … `defend`, `book-close` … `book-tab-down`, `box-ok`, `box-cancel`, `aim-cancel`,
  `result-ok` and the window's own as the map's.
- **M3 T1b, the contexts** (*provisional*): a command is taken everywhere (the window's own but
  quit, a frame on and restart) or in one context (the battle's: nothing open, the spell book, a
  message box, an aim, the result window; the map: one); a chord is refused on two commands one context takes, so
  Esc is quit, the book's close, the box's Cancel and the aim's give-up at once, and a user's
  pause on Space is refused in the battle (Defend's). The hotkeys left the dialogs' layouts
  (`h3_ui::ui::Button`'s `keys`): the table's defaults are their only copy.
- **M3 T1b, behaviour changed in the battle window:** the book takes every key while open (the
  panel's letters and the dragon's F reached the battle under it before); unattended, Esc quits
  whatever is open (the aim's and the box's Esc before) and the right button no longer gives an
  aim up; a chord with Ctrl held gives nothing unless bound (Ctrl and ↑ scrolled the log before).
- **M3 T2a, the mask's file** (*provisional*): `clients/map/masks.toml`'s `panels`, the rectangles
  M3's later tasks draw, read by `h3map.py` and by the viewer's `sequences` (its `screen` lines:
  the box of the change outside them, the whole screen otherwise).
- **M3 T2a, the live part's kingdom and pointer** (*provisional*): `kingdom = { heroes =
  [[asleep, movable], ...], towns, selected = [kind, place], tops, quests }`, `pointer = [x, y]`
  (a still's cursor, the arrow); a hero's `movable` read as points left (a step leaves 0 or at
  least the cheapest step, 0x481182), the quest log's test without the quest's text;
  `Live::at_start` the viewer's heroes in the map's order, the first selected.
- **M3 T2a, the buttons in a walk** (*provisional*): move hero set at a path's click, a walk's and
  a key's step's start and a walk's end (its rest's green arrows); a walk whose rest has no green
  arrow leaves its hero unable to move (the step set its points to 0) and next hero follows the
  kingdom; a key's step's end does not set next hero (no recording shows one that ends the
  points); the engine's from M4.
- **M3 T2a, `h3_ui::paint`**: the dialog items' drawers moved into the common crate (an `Assets`
  trait each viewer's batch implements); `h3-view` keeps its copy until the client lane takes
  this one (its inbox); a button takes a player's colours and a DEF from `Shown` by its id.
