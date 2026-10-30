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
  the unexplored tiles, the view's box, the underground; T3a as a still, T3b over time); the hero list (portraits, the
  movement and mana bars, the selection, the scroll) and the town list (T4a as stills, T4b over
  time); the status window
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
  mask in `clients/map/masks.toml`, the 40 situations and 13 sequences compared; the log); T2b
  cut with the user (2026-10-28): T2b1, the frame's fixed cost (**done** 2026-10-28: the
  widening by a table, the background drawn once, the levels' spatial index; the log), T2b2, a
  frame redrawing only what changed (kept whole by the user; **done** 2026-10-28: the drawers'
  spans, the view's cells, the changed rectangles widened and uploaded; the log); T3 cut with the
  user (2026-10-29): T3a, the minimap as a still (**done** 2026-10-29: `minimap_draw` 0x412C40
  read and built, the 40 situations exact with it; the log), T3b, the minimap over time (**done**
  2026-10-29: a scroll, a centring lift and each step's end draw it again; the 13 sequences
  recorded with it, every frame exact; the log); T4 cut with the user (2026-10-29): T4a, the
  hero and town lists as stills (**done** 2026-10-29: the slots, the bars, the towns' pictures and
  the selections read and built, three new situations and the 40 exact with them; the log), T4b,
  the lists over time (**done** 2026-10-29: a step's end draws the hero's slot, a lift that
  centres the view and a walk's end that leaves the hero unable to move both lists; the 13
  sequences recorded with them, every frame exact; the log); T4c, a changed panel item drawn
  again alone (**done** 2026-10-29: the background drawn again where its changed items reach, by
  `h3_ui::paint::reach`, and the view's cells there; `walk-1` 31.0 → 24.7 % uploaded; the log);
  T5 cut with the user (2026-10-29): T5a, the resource bar's amounts, the date and a text's reach
  (**done** 2026-10-29: `resource_bar_redraw` 0x558F20 read and built through the catalogue's keys,
  the live part's resources and date, a text's reach; the 44 situations and the 13 sequences exact
  with the bar; MCL-Q15; the log), T5b cut with the user (2026-10-29): T5b1, the status window of
  a hero and a town (**done** 2026-10-29: 0x451AB0 and 0x4521F0 read and built, the selection's
  window, the names by the catalogue or the map's own; three new situations and the 47 and the 13
  sequences exact with the window; MCL-Q16; the log), T5b2, the date's and the kingdom's windows
  and when each shows (kept whole by the user; **done** 2026-10-29: 0x450DD0, 0x452B80 and the
  date's step 0x4511A0 read and built, the chooser at every view's draw, a timed kind's end, the
  rebuilds; three new situations and two new sequences exact; the log); T6 cut with the user
  (2026-10-30): T6a, the mouse-over, cut again with the user (2026-10-30) into T6a1, the cursor by
  tile and the hints of the panels and the map but the map's 58 own cases (**done** 2026-10-30:
  0x40E360, 0x40DEB0, 0x40B150's frame, 0x403010 read and built; three new sequences exact, their
  status bar compared; MCL-Q17 to MCL-Q19; the log), and T6a2, those cases, cut with the user
  (2026-10-30) by what runs the hint into T6a2a, the 35 classes of the visit marks (**done**
  2026-10-30: their cases read and built, the facts' `visits`, MCL-Q18's fix; four new sequences
  exact with the status bar; the log), and T6a2b, the 20 of the owners and contents (seven
  sub-functions); every capture again with the hint unmasked moved to T7; T6b, the clicks, the
  keys during a walk, the sound without the focus; T7, M3's closure. A task that grows is cut at a
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
  perhaps the roads') as a sound option once their purpose is known (MCL-Q11); the status
  window's kinds the engine's turns and events set off (the user, 2026-10-29, at M3 T5b's cut):
  another player's turn (5: `AdStatNx.pcx`, the hourglass, 0x453250), a resource found (6:
  0x451220, 5 seconds) and a text (7: 0x451820; E24 design §4.8); a town's own name from the map
  (the harness refuses one until then).
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
  [[asleep, movable], ...], towns, selected = [kind, place], tops, quests }`, `buttons = [move,
  next]` (the states memory holds), `pointer = [x, y]`
  (a still's cursor, the arrow); a hero's `movable` read as points left (a step leaves 0 or at
  least the cheapest step, 0x481182), the quest log's test without the quest's text;
  `Live::at_start` the viewer's heroes in the map's order, the first selected.
- **M3 T2a, the buttons in a walk** (*provisional*, as the recordings show them, design §4.1):
  move hero set at a path's click (its green arrows), kept through the path's walk to its end
  (off on the destination, else by the rest's green arrows), on at a key's step that lifts the
  hero and off at one once it is lifted; a walk whose rest has no green arrow leaves its hero
  unable to move (the step set its points to 0) and next hero follows the kingdom; a key's
  step's end does not set next hero (no recording shows one that ends the points); the live
  part's `buttons` the states memory holds (a stale one kept, `walk-sail`); the engine's from M4.
- **M3 T2b1, the frame made once** (*provisional*, design §4.2): `h3_render::screen::Widening`
  (a format's 65 536 pixels widened once), `Screen::cut` and `copy_rect`, `h3_ui::cursor::
  Cursor::rect` (additive, the common foundation's); `compose::Composer` (the background drawn
  while the frame's picture and the dialog's draws stay, the rectangle the view's draws cover
  restored from it); `scene::TileIndex` and `scene_in` (a level's spatial index, built at the
  load and again when the live part's objects change); the window's `--check` line `widened`.
- **M3 T2b2, a frame redrawing only what changed** (*provisional*, design §4.3): `draw::Spans`
  (each frame's rows as spans of literal pixels and of runs, made as its DEF is decoded;
  `Batch::drawable`), the gems through the raw drawer (so a cell cuts them); `scene::ViewPlace`
  (`Scene::view`: the view's clip, origin and tile); `compose::Composer`'s 32 × 32 cells compared
  with the last scene's, the clip's pixels moved with the view (only on a background of one
  colour across the clip), `damage`, `compose_owned`; `damage::History` and `tidy`;
  `h3_render::screen::Widening::widen_rect`, `h3_sdl::Window::present_part` and
  `present_read_part` (additive, the common foundation's); the painter's assets decoded when the
  composer finds one missing (no scene's names collected a frame); the window's `--check` line
  `afresh` (T2b1's `widened`: each frame made against the same frame composed afresh) and its
  timing line's `uploaded`.
- **M3 T3a, the minimap's tables and draws** (*provisional*, design §4.4): `[map.records]` (the
  tile records' rules, moved out of `[map.sound]`) and `[map.minimap]` (the palette entries, the
  owned and blocking classes, each size's `columns`, `rows` and box, a scale as its f32's bits:
  `0x3FAA3D71`, truncated exactly in integers); `h3_map_view::record` and `minimap` (`Base` in
  `TileIndex`, built with the level; `TileIndex::build` takes the layout and refuses a level with
  a terrain the tables lack); the scene's `Minimap` and `Radar` draws, the composer's minimap
  staying put through a view's move; the owned object without an entrance on the map coloured
  as MCL-Q4's answer gives its flags (its owner; the original's terrain colour under `--bugs
  original`); `masks.toml`'s `stills` (until T3b).
- **M3 T3b, the minimap's last draw** (*provisional*, design §4.4): the timeline's `Minimap` (the
  view's corner and the selected hero's tile at its last draw: a scroll that moves the view, a
  lift that centres it) in each `Draw`; the live part's `minimap` (`minimap::Shown`, the hero by
  its index), none in a still; the level's colours made once with the index, its explored tiles
  in them (the index built again when they change, as for the objects), shared by `Arc` with the
  heroes over them (`Tiles::colours`, `heroes`, `shown`); `masks.toml` one list again (`panels`).
- **M3 T4a, the lists' tables, items and live part** (*provisional*, design §4.5): `[map.lists]`
  (`portraits` from `hero_table` 0x679DD0's 163 records; the bars' `step`, `round`, `top`, `from`;
  the towns' frames in `itpa.def`); the layout's items 15–29, 32–36, 39–43 as the constructor's
  (the portraits' layout picture `hpsxxx.pcx`, the frames' `hpsyyy.pcx`); `h3_ui::ui::Shown::
  pictures` (additive, the common crate's: a picture item's picture by its id);
  `panels::selections` (the selected hero's frame or the selected town's mark drawn after the
  items, as 0x4038C0 does; the mark's leftover frame, a redraw artifact, not copied); the
  kingdom's `[asleep, movable, portrait, points, mana]` and `[faction, fort, built]`, a portrait
  past the tables' refused by the scene, `Live::at_start` with no points or mana; the situations'
  `points`, `mana`, `built` and `select` (the harness's writes and selection, both lists redrawn
  by it); `masks.toml`'s `stills` (until T4b).
- **M3 T4b, the lists as last shown** (*provisional*, design §4.5): the timeline's `Draw::kingdom`
  (`Arc<Kingdom>`, the kingdom as the panels last showed it): a step's end shows the selected
  hero's points in its slot when the list shows it (the mana bar left as last shown, the slot's
  rectangle's: MCL-Q14), a lift that centres the view (the list scrolled to the hero,
  `lists::top_showing`) and a walk's end that leaves the hero unable to move every slot; memory's
  points from `Start::points` (a recording's `points` lines, `Sequence::points`: M3's walks
  scripted, the engine's from M4), taken at 0 and at each step's end; the timeline's `points`
  lines; the live part's kingdom as the panels show it (the harness reads it before writing a
  sequence's `points`); `masks.toml` one list again; the kingdom's form before T4a no longer read.
- **M3 T2a, `h3_ui::paint`**: the dialog items' drawers moved into the common crate (an `Assets`
  trait each viewer's batch implements); `h3-view` keeps its copy until the client lane takes
  this one (its inbox); a button takes a player's colours and a DEF from `Shown` by its id.
- **M3 T5a, the resource bar's texts** (*provisional*, design §4.7): the catalogue's keys
  `map.resources.amount`, `map.resources.date`, `word.month`, `word.week`, `word.day` (the
  original's from the executable's `%d` and date format and GENRLTXT 63 to 65); the viewer's
  binary reads only `panels::KEYS` of `clients/text/keys.toml` (the families left out) and stops
  on a form it cannot read; `Tables::texts` (the catalogue, empty from the committed tables: a
  key shows as itself); the live part's `kingdom` with `resources` (the original's order) and
  `date` (`[month, week, day]`), a map's start none and the first day; a situation's
  `resources` and `date`, only with a hero's selection; `h3_ui::paint::text_reach` (additive:
  the box widened by the font's overhangs).
- **M3 T5b1, the status window** (*provisional*, design §4.8): the layout's panels `status_hero`
  and `status_town` (our ids: the original's for the fixed items, the army's and the silo's by
  place) and the tables' `[map.status]` (the large portraits, the frames' offsets, the skills'
  bounds, the thousands, the halls and forts tested); the live part's `status` (a hero's type,
  `own_name`, skills, morale, luck, army; a town's name as a record of `TownName.txt`, buildings
  standing, garrison hero, army shown, silo, income; another kind not drawn), the harness reading
  the bonuses, the silo and the income from the window's own items (the original's rules
  computed: the engine's from M4); the names by the catalogue: the `hero` family keyed by the
  type in the map viewer (`hero.12`, the battle client keys its asset tables' heroes,
  `hero.base:orrin`: one key per hero for a translation needs one scheme) and a new family
  `town-name` (`TownName.txt`), `optional` (additive in `h3_ui::text::keys`: a client that lists
  no entries for it has none of its keys, so `h3-view` is unchanged); a hero's own name the map's
  (`MapArt::hero_names`: a hero object's, else a disposed hero's, `h3_ui::text::decode_1252`), a
  town's own refused by the harness (M4); the numbers by `map.status.number` and
  `map.status.thousands`; a situation's `skills`, `morale`, `luck`, `army`, `garrison`,
  `garrison_hero` and `silo` (the garrison hero a hero of Red's standing elsewhere: only the
  window reads it).
- **M3 T5b2, the date's and the kingdom's windows** (*provisional*, design §4.9): the layout's
  `status_date` and `status_kingdom` (our ids for the original's id-less counts and labels, the
  flags by row and place), the tables' `[map.status.date]` (the pictures' frames given and checked
  by the gate, the step, the sound); the live part's `status` of the kinds 1 (`frame`) and 2
  (`halls`, `allies`, `enemies`, the harness computing them from memory: the engine's from M4)
  and `selection` (the selection's window read before a kind is asked); a sequence's `status =
  [kind, end]` (`Start::asked`; a date's window at 0 the sequence did not ask refused), a
  situation's `status` and `teams` (written by the harness: the writer has no teams); the
  timeline's chooser (every draw, forced at a centring lift and a key's step's end; a window
  the live part does not give left alone); `Cue::Effect` and an audio stream of its own for a sound
  played once (the next cuts it off).
- **M3 T6a2a, the visit marks** (*provisional*, design §4.11): the live part's `[hover]` `visits`
  (per object whether the viewer knows its bonus and its mark; left out none known), derived by
  the harness from memory by the reading's rules (`h3mem.VISIT_RULES`, `Mem.visits`: the
  engine's from M4) for the hero selected as read (a click that selects another, T6b, reads them
  again); a situation's `visits` (`h3mem.Mem.write_visit`, the rules inverted: a kind's or the
  word's bit, the hero's or the player's field; a flag of several bits written as its lowest);
  `Hint::Visits` and the keys `map.hint.more` (the executable's " %s"), `map.hint.visited`,
  `map.hint.not-visited` and the family `map-bonus` (XtraInfo.txt). Recorded with a hero
  selected only: the marks a class keeps by a hero read as needing one, not recorded without.
  MCL-Q18's fix (the mouse's tile by the view's origin; the original's under `--bugs original`)
  changes the window's clicks and hints: T7's hand test.
