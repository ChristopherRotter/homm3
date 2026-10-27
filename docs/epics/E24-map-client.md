# E24 — The adventure map client

Status: **refined** (2026-10-23, T0, with the user); **M1 closed** (2026-10-24, T6b: the
reviewers, the review list accepted by the user); **M2 refined** (2026-10-26, M2 T0, with the
user); next M2 T3b (`docs/lanes/map-client/QUEUE.md`).
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
  scroll, by whole tiles (the manager's pixel offset only a walk writes: design §3.5), by
  Ctrl and the arrow keys and the mouse at the screen's edges, at each map scroll speed, with the
  edge cursors; a hero's walk along its path: its groups and frames, its pixel steps, the timing at
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
  - **Key bindings** (the user, 2026-10-26, at T1b's start): no key hard-coded in the client;
    T4's window takes every key through a table of actions, the original's keys as its defaults,
    a user file overriding them; M3 T0 moves the table into the common crate (the battle
    viewer's keys too); a screen to rebind them comes with the options dialog.
  - **The edge scroll's zones** (MCL-Q6, the user, 2026-10-26): 16 pixels on every side, the
    original's 16, 14, 16 and 15 for the comparisons only (`--bugs original`).
- **The queued M1 items:** R2-8 (the writer's edge bound, the inbox) waits for a task that
  needs the writer; none planned, so T6 makes its one-line `cross-lane` pull request if no task
  took it. A loaded saved game's draw lists: M4.
- **Tasks** (the queue has each in full): T1a1, the recording and the map standing still
  (**done** 2026-10-26: `h3map.py frames`, two idle sequences; `E24-log.md`, design §3), and
  T1a2, the scroll (T1a cut at its seam before it started, the user, 2026-10-26; **done**
  2026-10-26: Ctrl and the direction keys, the edge every 70 ms, the limits, the edge cursors,
  four sequences; design §3.4, MCL-Q6 answered); T1b, the walk
  read and recorded (**done** 2026-10-26, whole: the lifted hero at the view's centre, the view
  sliding by pixels, seven sequences; design §3.5); T2, the map over time headless (**done**
  2026-10-27, whole: `h3_map_view::timeline`, `h3-map-view sequences`, the edge cursors, the
  harness's frame check, `drawn` over the sequences; T1a's six sequences exact line by line and
  frame by frame; design §3.6); T3, the walk, cut at its seam before it started (the user,
  2026-10-27): T3a, its timing headless (**done** 2026-10-27: the walk in `h3_map_view::timeline`,
  T1b's seven sequences exact line by line but the `screen` boxes; design §3.7), T3b, the walk
  drawn (**done** 2026-10-27, whole: the lifted hero at the view's middle and its blended pass 9,
  the view's offset, the late redraw drawn again; T1b's seven recorded again, every line and all
  269 frames exact; design §3.8; MCL-Q7 open); T4, the window, cut at its seam before it started
  (the user, 2026-10-27): T4a, the window replaying and the key table (**done** 2026-10-27:
  `h3-map-view window`, `h3_map_view::keys` and `replay`, every frame read back exact; design
  §3.9); T4b cut at its seam before it started (the user, 2026-10-27): T4b1, the live input
  (**done** 2026-10-27: the keys, the keypad's and their repeats, the edge scroll, the clicks, the
  hardware or drawn cursor, `Replay::give`, every change shown checked against a headless play;
  design §3.10), T4b2, the Windows build tried on the user's machine (**done** 2026-10-27:
  every frame read back exact on Windows, the delays measured, the user's findings; design
  §3.10; MCL-Q10 asked and answered), T4b3, the view's drawing made fast (the user, at T4b2:
  "the client should run on really old hardware too"; **done** 2026-10-27: the drawers by rows,
  31 times fewer instructions a view, every frame unchanged, none passed over on the user's
  machine; design §3.12; MCL-Q11 asked); T5, the sound (**done**
  2026-10-27, whole: the terrain music, the step sounds and the ambient loops read;
  `h3_map_view::sound`, cued by the timeline and the replay, played by the window; the gate's
  check of the sounds; design §3.11; MCL-Q8, MCL-Q9 open); T6, M2's closure, cut at its seam
  before it started (the user, 2026-10-27): T6a, the reviewers and their fixes (**done**
  2026-10-27: three reviewers, 29 findings, 24 fixed; the music player's five cut out as T6a2 by
  the user; design §3.7 corrected; MCL-Q11 answered), T6a2, the music player as the original's
  music thread (**done** 2026-10-27, whole: `play_music`, `music_thread` and `music_stop_fade`
  read; `h3_map_view::music` and the window's music on it, the five findings fixed; design §3.11;
  MCL-Q12 asked), T6b, the closure.
- **Exit:** every recorded sequence as recorded, line by line and frame by frame (or each
  difference an `MCL-Q<n>`); a situation and a sequence play in the window at the original's
  frame times, with its sound, and render headless frame by frame.

### Later (refined when they come)

- **M3, the adventure screen:** the minimap, the hero and town lists, the status window, the
  resource bar and the date, the buttons, the mouse and its cursor; the string catalogue's keys.
  Needs `h3-view`'s `ui`, `text`, `layout`, `mouse` and the `clock` and `cursor` M2 copied in
  one common crate: a `lanes.toml` change (shared) agreed with the client lane, proposed at M3
  T0 (M2 T0's decision); T4's key bindings move there too (the client lane's inbox). The keys
  during a walk (MCL-Q10): Space's cancel of the queued steps (the user: "perhaps by other means
  too"; not read) and the input ring's bound of 63 messages.
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

M1's review lists (three reviewers, T6b; the list accepted): `E24-archive.md`. Later milestones add theirs here.

**M2's review list** (the autonomy contract's "queue for the milestone review"; T6 adds the
reviewers'):

- **M2 T4a, the key table** (*provisional*): the user's file a TOML table of commands by name
  (`scroll-north` … `hero-north-west`, `pause`, `frame`, `restart`, `vsync`, `fullscreen`,
  `quit`), each with its list of chords (`"Ctrl+Up"`), replacing that command's defaults, `[]`
  unbinding it; its place `--keys`, else `keys.toml` in the working directory; a chord matches
  exactly (Ctrl and a key is not the key).
- **M2 T4a, the window's own keys:** F5 pause, F6 the next frame while paused, F7 restart, F8
  vsync, F11 fullscreen, Esc quit, not `h3-view window`'s P, →, Home, F and V: the original's
  map takes the direction keys and nearly every letter (design §3.9). M3 T0's common table
  decides both viewers'.
- **M2 T4a, the replay's cursor:** drawn in the frame at the recorded point, the system's pointer
  hidden over the window and the mouse doing nothing until T4b.
- **M2 T4b1, the live input** (*provisional*): a held key's repeats are the system's (the
  original's way); the keypad's eight keys bound with the direction keys in either NumLock
  state; F9 the cursor's kind, the hardware cursor the default; the pointer over the view the
  arrow (M3's tile cursors), a click on the button's press; a situation's hero the viewer's only
  one; the pointer off the window parked off the frame (no edge scrolls); the view composed whole
  after an input (30 to 60 ms in the sandbox; an incremental scroll left for later); a user's
  file binds only the adapter's keys (`h3_sdl::Key`: 13 letters, M3's hotkeys add the rest).
- **M2 T5, the sound** (*provisional*): the tables' `[map.sound]` and a terrain's `music` and
  `step` (the format: `clients/README.md`); the volumes as gains at the settings' highest (no
  settings yet); F2 the music and F3 the effects (effects off holds the gains at 0, the loops
  running on); the music's fades applied to its samples as queued (the device's time); a track
  decoded on a thread when first asked for; the ambient slots on four streams (the original's
  category of eight handles, at most four used); a flying hero's step sound unused until M4 (the
  engine's); the cues not compared with the original (the harness plays without sound); an
  abandoned mine by H3API's flag (the map's 220 or subclass 7, its set-up not read).
- **M2 T4b2, the Windows build:** `h3-map-view.exe` looks for `clients/map` and the installation
  copy where it was built (`CARGO_MANIFEST_DIR`), so the user's `.cmd` files name `--map` and
  `--install` (`h3-view` looks above the exe; one way for both in M3 T0's common crate).
- **M2 T4b3, the view drawn fast** (*provisional*): `h3-render`'s `Screen::pixels_mut`
  (additive, the foundation's); every asset a view of the map draws at 0 decoded when the map
  loads (a big map's load longer, no decoding during a walk); the window's report gives how long
  each frame took to make; a frame of a near-empty view still about 4 ms on the user's machine
  (the inbox: the user, "we should check back in at some point").
- **M2 T6a, the reviewers' findings** (*provisional*): a key without a binding (Ctrl alone, M3's
  hotkeys) ends no path's walk yet (the adapter's keys only; the original's any key-down does);
  the option Show Route taken on (a walk a message ended shows the rest of its path again); the
  actions of one millisecond handled one after another, as the harness gives them (a click's
  walk empties only what waited behind it during a key's step); a situation's view at most the
  map's side off the map (the original's own range the harness's check); `h3-sdl`'s
  `Window::focused()` (additive; true until SDL says the focus left: Xvfb without a window
  manager sends none) and the pointer given off the frame without the focus; a point on the
  window off the frame taken as the frame's nearest (fullscreen's bars); `frames` writes no
  scroll time and releases the direction keys and Ctrl at a run's start.
- **M2 T6a2, the music** (*provisional*): `h3_map_view::music` on a clock of the frames the
  device has played, a second queued ahead and taken back when a request is taken; a track still
  being decoded holds the music (silence; the original opens its file at once) and counts as
  playing for a request of its name; a track not of 44.1 kHz stereo refused (one stream format);
  a track's place what is heard where its fade began (Miles' position not read); the sound
  without the focus not built (the inbox).
