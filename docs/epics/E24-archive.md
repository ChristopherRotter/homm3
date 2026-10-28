# E24 — archive: M1 (stills of the map view), M2 (the map moving) and their review lists

Moved out of `E24-map-client.md` on 2026-10-24, verbatim: milestone M1's task list and the review lists. M2's section and its review list moved on 2026-10-28 (M2 T6b2c), verbatim, at the end.

### M1 — stills of the map view

- **Code readings** (research item 9 among them): the view's tiles (terrain, rivers, roads, the
  map's edge, the unexplored tiles and their edges), the moment (which frame an animated object
  or tile and which colour cycle a still shows at a game time), the objects' draw order and
  their shadows, heroes, their flags, boats, the owners' colours.
- **The crate and its data:** `clients/crates/h3-map-view` (a pure library, the scene and its
  draw list, as `h3-view`'s; the adapters in its binary) and `clients/map/` (the map's asset
  tables, the adventure screen's layout at 800 × 600, the capture set); the gate checks the
  tables' names against the archives.
- **`h3-map-view render <h3m> --view x,y,z [--at <ms>]`:** a PNG of the adventure screen's
  frame and the map view at that view and moment: terrain, rivers, roads, the edge, objects and
  shadows in the original's order, heroes and flags, owned objects in their owner's colours, the
  unexplored tiles.
- **The comparisons:** the harness loads the same map in the original (`h3adv.py`), sets the view
  and the moment, dumps the screen; the frame outside the panels is compared exactly. The
  situations: the writer's maps built for one drawing question each (terrain transitions,
  overlapping objects, heroes by class and facing), and shipped maps at chosen views (their
  random objects read from memory).
- **Done:** T1a (2026-10-23): the view's passes, tiles, edge, unexplored tiles and the moment
  read, VCMI compared (`E24-design.md` §1; MCL-Q1, answered: the width). T1b (2026-10-23): the
  objects' draw lists, layers and passes, shadows, heroes and boats read, VCMI's ten
  differences and T4's situations (`E24-design.md` §2). T2 (2026-10-23): the crate
  `h3-map-view` (the static art from `RawMap`, the live part, the tables
  `clients/map/assets/base.toml`, the layout `clients/map/layouts/adventure.toml`, the gate);
  the frame's picture read, T1a's open points closed by the gate (`E24-design.md` §1.9). T3a
  (2026-10-23): the writer's additions (`h3_map::writer::write_with`: tiles as given, objects of
  any template with the editor's body; `docs/formats/h3m.md`), a `cross-lane` pull request.
  T3b1 (2026-10-23): `h3-map-view render`, the frame and the view's tiles (terrain, rivers,
  roads, the edge, the unexplored tiles, the gems) as a PNG through the scene, its batch and
  compose, and the map's two drawers (`E24-design.md` §1.10; MCL-Q2 asked: the last row's road).
  T3b2 (2026-10-23): the map's stills in the original (`clients/capture/h3map.py`, the
  situations `clients/map/situations/` built by `h3-map-view situation`): eleven situations of
  terrain, water and lava, the four rivers, the roads, the edges and the unexplored tiles, all
  exact outside the panels; the clock drives the map, its palette step read from memory. T4a
  (2026-10-23; T4 split with the user into T4a, T4b and T5's share; MCL-Q2 answered): the
  objects drawn (`h3_map_view::objects`: the `.msk` shapes, the draw lists; passes 4, 5, 7),
  the situations' objects (copied with a shipped map's rectangle, or given by DEF), the phases
  and frame step read from memory; six situations of copied dense corners and edge cuts, exact
  (`E24-design.md` §2.6). T4b (2026-10-23; MCL-Q3 asked and answered): the gate checks every
  object picture against its `.msk` (D8: none outside) and lists what a random object's picture
  loses; the random objects read (a copy of the random template with the drawn DEF, §2.7) and
  taken from memory (`Drawn`); shipped maps loaded whole: five views of four maps, one
  underground, with random artifacts, monsters and resources, exact; research item 9 closed.
  T5a (2026-10-23; T5 split with the user into T5a and T5b): heroes and boats read (their groups
  by facing, frames, flags, the coast's waves, a hero's boat; `E24-design.md` §2.8) and drawn as
  two-row objects among the layers (passes 5 and 7), taken from memory with their facings; the
  situations' heroes of any colour and class, Red's walks by play, boats; five situations (the
  18 classes × 8 facings, heroes among objects: D4, D5; boats: D7), exact. T5b1 (2026-10-24; T5b
  split with the user into T5b1 and T5b2; MCL-Q4 asked and answered): the owners' colours read (the owned
  classes, the vertical garrison and the AB mine loaded as two of them, the entrance's owner,
  `game.pal`'s entries 63 to 71; D10 settled: no drawn picture loses its colours), the towns'
  pictures by their buildings and the random dwellings resolved at the load (`E24-design.md`
  §2.9); drawn (`Draw::Piece`'s flag), taken from the start and from memory; eight situations
  (owned objects of every colour, random dwellings, the nine factions' towns by their buildings,
  random towns, two shipped maps' views), exact. T5b2 (2026-10-24): the path's arrows read (passes
  6 and 8: `adag.def` by the manager's +0x48 per tile; `E24-design.md` §2.10) and drawn, taken
  from memory with the explored tiles; five situations (a path under the objects' upper parts,
  D9; a path cut by the view; two shipped maps' starts with their unexplored tiles; boats' flags
  of the seven other colours), exact, the set (40) again. T6a (2026-10-24; T6 split with the user
  into T6a and T6b; MCL-Q5 asked and answered): the half tick read (a tick at the load's progress
  step between the terrains' DEFs and the rivers' leaves the rivers a step behind for the whole
  game, `E24-design.md` §1.6), each kind's step read and drawn (`Moment::river_cycles`); `h3map.py
  drawn` (ours from each situation's kept live part, the digests in
  `map/situations/kept/drawn.txt`), the whole set (40) captured, exact, and recorded. T6b
  (2026-10-24, M1's closure): three fresh reviewers (the library, the binary and its data, the
  harness), 37 findings, 35 fixed (the fallen objects' entries taken out of built lists as
  0x4AABB0 does, the harness's checks before its writes and of its staging, a failing situation
  failing alone, `drawn`'s `STALE`, the refusals of what does not fit) and 2 queued (the writer's
  edge bound, a saved game's lists); the whole set (40) captured again, exact; the review list
  accepted by the user. **M1 closed.**
- **Exit** (met at T6b): the capture set's stills match the original pixel for pixel outside the masks of
  what M1 does not draw (the panels, the mouse), or list each difference as an M2 item; the
  review list accepted; research item 9 closed by the reading and its frames.

## Review lists

The M1 review list starts here (the autonomy contract's "queue for the milestone review").

- **M1 T2, the formats** (*provisional*): the map's tables keyed by the `.h3m`'s codes, not the
  packs' keys (the static art is the raw map's); `[map]` holds the moment's steps and the road's
  offset, the layout the view's place, loops, clip and the gems; the shroud's 256 pieces as data,
  their variant rules (mirror, odd x and y) in code.
- **M1 T2, the library's dependency on `h3-map`** (its raw model; no I/O): the static art is
  converted in the pure library, so its tests use the writer; the purity test is a copy of
  `h3-view`'s for this crate.
- **M1 T2, the live part at a map's start:** heroes and boats facing right (T5a: as the
  original's new game) and every tile explored (T5b2: what a start reveals is the engine's, the
  comparisons read the original's); phases 0 (the original's are random).
- **M1 T3a, the writer's additions** (the adventure lane's crate): a second entry point
  (`write_with(&Map, &Extras)`) instead of new `Map` fields and `MapObject` variants, which would
  have changed struct literals and matches in four other lanes' crates; the editor's bodies taken
  from the SoD maps' most common ones (a scroll's spell 0, the grail's radius 0 and a random
  dwelling's levels 0–6 without a default seen); heroes, prisons and hero placeholders not among
  the objects of any template.
- **M1 T3b1, the still** (*provisional*): the map's own drawers in `h3-map-view` (`draw`: the
  top-to-bottom mirror and the clip, which `h3_render::blit` lacks; a field in its `Placement`
  would change the client lane's literals) and its own asset batch (`h3-view`'s is the client
  lane's; M3's common crate may merge them); the dependency on `h3-render`; each piece clipped on
  the screen (the original's source rectangles, the same, §1.10); `render`'s flags (`--hidden`, a
  rectangle unexplored for a look at the shroud until T5 reads the original's; `--bugs`; `--at`
  on a clock that is never late); MCL-Q2's fix the default (`--bugs original` the comparisons').
- **M1 T3b2, the map's stills** (*provisional*): the situations' format (`h3_map_view::situation`:
  the writer's terrain, patches and roads, tiles given or copied from a rectangle of a shipped
  map named by its file, two heroes, the view, the hidden rectangles); the harness writes Red's
  explored bits into the game (as E17 M4 T1b writes a siege's walls) instead of revealing by
  play; a single scenario (Red human) rather than hot seat; the masks (the panels right of x 608
  and below y 552); `render --cycles` (the original's palette step) and its `--hidden` repeated.
- **M1 T4a, the objects** (*provisional*): the objects' shapes (`.msk` and the DEF's frame
  count) loaded by the binary and handed to the scene, which stays free of I/O; `render --live`
  (the harness's `live.toml`: the palette step, the frame step the screen shows, every object's
  anchor, checked, and phase); the situations' objects (`copy`'s `objects = true`, neutral,
  without the hero classes; `objects` given by DEF through the game's `Objects.txt`); the eight
  owned classes' flag runs skipped until T5b1 coloured them.
- **M1 T4b, random objects and shipped maps** (*provisional*): a situation names a shipped map
  whole (`map`), loaded with the scenario list's setup (its first human player) and its first
  day's messages closed; `live.toml`'s `viewer`, `defs` and `classes` (every object's template
  in memory); the live part's `Drawn` (the DEF the original's copy names and the object's own,
  `render` taking the own from `Objects.txt`'s last template of the class and subclass); MCL-Q3's
  fix the default (`--bugs original` the copy's); the gate's object pictures (D8 a problem, the
  random copies' losses listed, not problems).
- **M1 T5a, heroes and boats** (*provisional*): the situations' `heroes` a list (tile, owner,
  class, facing, walk; a class's hero types in order) and `boats` (tile, type, facing), the 17
  earlier situations rewritten to it (their maps byte for byte the same); the harness walks Red's
  heroes by play (a hero boards a boat so) and writes the heroes' and boats' facings into their
  records (a still's staging, as the explored bits), and `live.toml`'s `heroes` and `boats`; the
  `boats` situation's two boarding tiles given an open water picture (no hero sails on its first
  day); the boats' flags of colours other than Red's not captured (the gate checks them);
  `[map.heroes]`'s format; `Shapes` holding the frames of pictures without a `.msk`;
  `Draw::Piece`'s group.
- **M1 T5b1, the owners** (*provisional*): `[map.owners]` (the palette, the neutral entry, the
  classes with 219 and 220 as loaded) and `[map.towns]` (three lists by faction); `Draw::Piece`'s
  `flag` and `Scene::flags` (an entry of a named palette rather than a colour, the scene staying
  free of assets); the owner per object in the live part (the original's per entrance tile: the
  same but for MCL-Q4's case); `Live::at_start` taking the towns' pictures; `render --live`'s town
  check and its own picture only for an artifact (a class the game loads as another kept its DEF:
  the vertical garrison would have drawn a horizontal one); the situations' `towns` (a faction,
  an owner and `village`, `fort`, `citadel`, `castle` or `capitol`) and objects' `owner`, every
  owning colour given a hero outside the view; the random dwellings' situation drawing the Angel
  Portal's moved anchor only in about one run of two.
- **M1 T5b2, the path and the explored tiles** (*provisional*): `Live::path` as arrows by tile
  (the original's frame numbers, read from memory; choosing them from a path is M2's) and
  `Pass::PathShadow`, `Pass::Path`; `[map] path`; `live.toml`'s `explored` (a string of `0` and
  `1` per row of each level) and `path`, always written, so every comparison draws the explored
  tiles memory holds; the situations' hero `path` (one, Red's) and boat `colour` (written into the
  boarded boat's record: a still's staging, since a computer player's hero would have to board),
  a shipped map's `explored = "start"`.
- **M1 T6a, the digests and the rivers' step** (*provisional*): the kept live parts as copies of
  the stills' `live.toml` (`map/situations/kept/<id>.toml`: numbers and the DEFs' names, 1.6 MB
  for the 40, a shipped map's up to 0.3 MB), one digest per situation (its one still) in
  `kept/drawn.txt`, `drawn` in `h3map.py` beside E17's `h3drive.py drawn` rather than in it;
  `Moment::river_cycles` and `live.toml`'s `river_cycles` (the rivers' step, MCL-Q5), `render
  --cycles` setting both.
- **M1 T6b, the reviewers' findings** (*provisional*): `--bugs original`'s owner for MCL-Q4 the
  last object whose entrance is tile (0, 0), not the original's tile object (the topmost
  visitable entry, a hero standing for the one under it): compared nowhere, no shipped map has
  the case (R1-5); a kept live part records the digest of the situation it showed (its parsed
  keys), `drawn` calling one of another staging `STALE` (R3-10); the scene refuses a live part
  that does not fit the map and a picture without its shape (R1-3, R1-4), `render --live` a view,
  owner or dwelling anchor the original cannot have given (R2-3 to R2-5). Queued: the writer's
  edge bound one too generous (R2-8, the next writer pull request: the adventure lane's crate); a
  loaded saved game's draw lists (M4).

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
- **The queued M1 items:** R2-8 (the writer's edge bound): no task needed the writer, so T6b1
  made its one-line `cross-lane` pull request (#523). A loaded saved game's draw lists: M4.
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
  MCL-Q12 asked), T6b, the closure, cut at its seam before it started (the user, 2026-10-27):
  T6b1, the recordings again (**done** 2026-10-27: the 13 sequences, every line and all 450
  frames exact, T6a's harness changes run; the 40 stills exact; R2-8 as #523), T6b2, the
  closure with the user, cut at its seam (the user, 2026-10-27): T6b2a, the review list and the
  questions (**done** 2026-10-27: the review list accepted; MCL-Q7, Q8 and Q12 answered, MCL-Q9
  left open; a build of `master` and a checklist on the user's machine), T6b2b, the answers
  built (**done** 2026-10-28: MCL-Q7, Q8 and Q12 the client's, the original's behind `--bugs
  original`; design §3.8, §3.11), T6b2c, the hand test and M2 archived (**done** 2026-10-28:
  the window by hand on the user's machine, every item as described; M2 archived).
- **Exit** (met at T6b2c, 2026-10-28): every recorded sequence as recorded, line by line and
  frame by frame (or each difference an `MCL-Q<n>`); a situation and a sequence play in the
  window at the original's frame times, with its sound, and render headless frame by frame.

## M2's review list

**M2's review list** (the autonomy contract's "queue for the milestone review"; T6a's 29
findings all fixed, none added; **accepted by the user** at T6b2a, 2026-10-27):

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
