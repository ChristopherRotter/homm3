# E24 — The adventure map client

Status: **refined** (2026-10-23, T0, with the user); **M1 closed** (2026-10-24, T6b: the
reviewers, the review list accepted by the user); **M2 refined** (2026-10-26, M2 T0, with the
user); next M2 T1a2 (`docs/lanes/map-client/QUEUE.md`).
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

### M2 — the map moving (refined 2026-10-26, M2 T0, with the user)

The objects' animations and the terrain's colour cycles over time, the view scrolling, a hero's
walk along its path (the path's arrows), in a window at the original's frame times (`h3-sdl`)
with the map's sound, checked frame by frame against recorded sequences as E17 M2 did for the
battle.

- **Scope:** the map standing still over time (the objects' step at each redraw, every 180 ms;
  the colour cycles' ticks every 200 ms, shown at the next redraw: `E24-design.md` §1.6); the
  scroll, by whole tiles (the manager's pixel offset is never written, `map.toml` 0x406DF0), by
  the arrow keys and the mouse at the screen's edges, at each map scroll speed, with the edge
  cursors; a hero's walk along its path: its groups and frames, its pixel steps, the timing at
  each hero speed (the enemy speed read with it), the view following it, the path's arrows
  taken away step by step, on land in the eight directions and a boat sailing; the map's sound;
  the window. **Not in M2:** boarding and landing, teleports, visits and their dialogs,
  battles, the computer's heroes moving (M4); the rest of the mouse (M3).
- **Decisions (the user, M2 T0; every recommendation taken):**
  - **Walks are scripted in M2.** The walk is built for real (its input: a hero walks along
    these tiles; M4 and M5 reuse it unchanged); in M2 its tiles come from the recorded
    sequences (the original's path, read from its memory). The engine supplies them from M4 (a
    game's record) and M5 (the player's click, the engine's path and move): the path always
    comes from the engine (server-authoritative), never from a pathfinder in the client.
  - **The map's sound is in M2** (T5): the terrain music, the objects' ambient loops, the hero's
    step sounds; the map's own (`h3_view::sound` is the battle's opening and end), on E17's
    pieces (`h3_assets::snd`, `wav`, `nanomp3`, `h3-sdl`'s audio).
  - **`h3-view`'s `clock` and `cursor` are copied** into `h3-map-view` (*provisional*, an origin
    comment each); M3 T0 proposes one common crate for both lanes (`clock`, `cursor`, `ui`,
    `text`, `layout`, `mouse`) in one `shared/` pull request agreed with the client lane, and
    the copies go.
  - **Recorded and compared as E17 M2: every frame.** `clients/map/sequences.toml`: each
    sequence a situation (of the set or its own), its options (hero speed, scroll speed) and its
    inputs with their times (a key held and released, the pointer at a place, a tile clicked
    twice), until a time. `h3map.py frames [--only <id>]` plays it in the original on the virtual
    clock 1 ms at a time and keeps every screen whose view changed, with its time and the
    state read from memory (the view's corner, the objects' step, both palette steps, the
    moving hero's tile, offset, group and frame, the arrows left): the timeline committed
    (`clients/map/timelines/<id>.txt`, numbers only), the screens local. `h3-map-view
    sequences [<id>]` makes the viewer's timeline from the situation and the inputs (every line
    as recorded, exit 1 on a difference; the unit tests replay the timelines without the
    original files) and with `--frames <dir>` draws its frame at each kept time; compared pixel
    by pixel outside the panels (the original's until M3), each frame's count recorded
    (`timelines/<id>-frames.txt`); `drawn` covers the sequences (`STALE` when one changed).
- **The queued M1 items:** R2-8 (the writer's edge bound, the inbox) waits for a task that
  needs the writer; none planned, so T6 makes its one-line `cross-lane` pull request if no task
  took it. A loaded saved game's draw lists: M4.
- **Tasks** (the queue has each in full): T1a1, the recording and the map standing still
  (**done** 2026-10-26: `h3map.py frames`, two idle sequences; `E24-log.md`, design §3), and
  T1a2, the scroll (T1a cut at its seam before it started, the user, 2026-10-26); T1b, the walk
  read and recorded; T2, the map over time headless (idle and scroll exact); T3, the walk drawn
  (exact); T4, the window; T5, the sound; T6, M2's closure.
- **Exit:** every recorded sequence as recorded, line by line and frame by frame (or each
  difference an `MCL-Q<n>`); a situation and a sequence play in the window at the original's
  frame times, with its sound, and render headless frame by frame.

### Later (refined when they come)

- **M3, the adventure screen:** the minimap, the hero and town lists, the status window, the
  resource bar and the date, the buttons, the mouse and its cursor; the string catalogue's keys.
  Needs `h3-view`'s `ui`, `text`, `layout`, `mouse` and the `clock` and `cursor` M2 copied in
  one common crate: a `lanes.toml` change (shared) agreed with the client lane, proposed at M3
  T0 (M2 T0's decision).
- **M4, the engine's games shown:** a whole game's record and replay viewer; the live part from
  the engine (`h3_game`'s observation needs an object identity matching the map's object list:
  the adventure lane's inbox), the protocol's adventure messages (the server lane, E16); a battle
  handed to the battle client's viewer (CLT-Q4's background variant and CLT-Q6's colours then
  come from the map); a loaded saved game's draw lists (the objects gone before the save: not
  read, M1 T6b).
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

## Review lists

M1's review lists (three reviewers, T6b; the list accepted): `E24-archive.md`. Later milestones add theirs here.
