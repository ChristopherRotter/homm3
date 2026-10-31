# E24 — archive: M1 (stills of the map view), M2 (the map moving), M3 (the adventure screen) and their review lists

Moved out of `E24-map-client.md` on 2026-10-24, verbatim: milestone M1's task list and the review lists. M2's section and its review list moved on 2026-10-28 (M2 T6b2c), verbatim; M3's on 2026-10-31 (M3 T7c2), verbatim, at the end.

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
  exact with the status bar; the log), and T6a2b, the 20 of the owners and contents (kept whole by
  the user; **done** 2026-10-30: the seven sub-functions and the nine quests' texts read and
  built, the facts' contents, the writer's quests; four new sequences exact with the status bar;
  MCL-Q20, MCL-Q21; the log); every capture again with the hint unmasked moved to T7; T6b, the clicks, the
  keys during a walk, the sound without the focus, cut with the user (2026-10-30) into T6b1, the
  buttons' press and hotkeys, the lists' clicks and Next Hero (**done** 2026-10-30: 0x4097E0,
  0x409A70, the button's loop, `hero_select` and `town_select` read and built; two new sequences
  exact with the status bar; MCL-Q22; the log), T6b2, the underground, the minimap's click and
  the status window's (**done** 2026-10-30: the command 4, `minimap_click` 0x40A0C0 whole with
  its loop, the items' press and release and the status window's click 0x409FD8 read and built;
  the situation's underground; three new sequences exact with the status bar; MCL-Q23, MCL-Q24;
  the log), and T6b3, the keys during a walk and the sound without the focus (kept whole by the
  user; **done** 2026-10-30: Space, the input ring and the walk's end, the focus's loss and return
  read whole and built; four new sequences exact with the status bar; MCL-Q25, MCL-Q26; the log);
  T7, M3's closure, cut with the user (2026-10-30): T7a, the status bar's hint compared
  everywhere and everything recorded again (**done** 2026-10-30: `masks.toml`'s `hint` and
  `hinted` gone, the 50 situations and the 35 sequences exact on the whole screen, 1 368 frames; a
  path's staging and a map-named town's hover facts fixed in the harness; the log), beside it M3's
  review by six fresh reviewers on fractions (42 findings on `map-client/e24-m3-t7-review`), T7b,
  the findings consolidated and fixed, cut with the user (2026-10-31) into T7b1, the redraw's
  and the input's (**done** 2026-10-31: F2-1 to F2-8 and F4-1 to F4-9 fixed, MCL-Q25's answer
  built; `drawn` all the same; the log), and T7b2, the panels', the mouse-over's, the harness's
  and the rest's (kept whole by the user; **done** 2026-10-31: the 24 findings fixed, noted or
  declined; `drawn` all the same, two sequences exact under Wine; the log); T7c cut with the user
  (2026-10-31): T7c1, the review list, the questions and the hand test (**done** 2026-10-31: the
  list accepted; MCL-Q13 to Q15 and Q20 to Q24 answered, Q22's and Q24's fixes for T7c2; the
  Windows build's catalogue read beside `--map`; the hand test passed, all as described; the log), T7c2, MCL-Q22's and Q24's
  answers built and M3 archived (**done** 2026-10-31: the hint of the mouse's item drawn again
  after a redraw, a hotkey's button released during a hold, MCL-Q28 asked and answered with
  them, the original's behind `--bugs original`; `drawn` all the same; the log); M4's
  refinement its own task after it (M4 T0). A task that grows is cut at a seam agreed with the
  user.
- **Exit:** every situation and sequence as recorded on the whole screen (no `panels` mask
  left, or each difference an `MCL-Q<n>`); the view-only clicks, the hints and the cursor in the
  window; `h3-ui` the only copy of what both viewers share; M3's review list accepted; a hand
  test on the user's machine.

## M3's review list

**M3's review list** (the autonomy contract's "queue for the milestone review"; **accepted** by
the user at T7c1, 2026-10-31, as built; the two schemes of the `hero` family's keys, by type here
and by the asset tables' name in the battle client, stand until a translation needs one):

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
- **M3 T7b1, a key's repeat** (*provisional*, MCL-Q25's answer, design §4.15): only a direction
  key's repeat (`Action::Repeat`, the window's by its key table's step) is dropped while a key's
  step waits; a scroll's, a hotkey's and Space's repeats wait in the ring as the original's.
  During a path's walk a repeat ends the walk as any key down; those behind it, taken after the
  walk, step once, the rest dropped behind that step.
- **M3 T7b1, the right button** (*provisional*, the review's F4-5): the window takes it as the
  mouse's on the map before any key table's binding (`RightButton` in a user's `[map]` table
  then does nothing); its down and up are messages in the ring, its down ends a path's walk, its
  up ends a button's hold; the original's help texts while it is held are not built (M4's).
- **M3 T7b1, the window's redraw and upload** (*provisional*, F2-5, F2-7): a redraw asked for
  uploads whatever the turn presents whole; a render device's reset (`SDL_EVENT_RENDER_DEVICE_RESET`,
  which loses the textures themselves) is not mapped: a rebuild in `h3_sdl` for both viewers, not
  a redraw. The rectangles another holds are dropped before widening and uploading; overlapping
  ones are not joined (a joined box can widen more pixels than the two).
- **M3 T7b2, a crafted layout or live part** (*provisional*, the review's F1-1, F3-1, F3-3,
  F6-1): a text box, a label's colour or a dialog's shadow at the integers' end saturates
  instead of being refused at the load (`h3_render::blit::darken` cut to the screen, the same
  pixels); a family's entry the data names past the catalogue (a hero, a creature, a spell …)
  and a quest's hero -1 show the empty text (`h3_ui::text::Catalogue::known`, additive), as
  MCL-Q21 (a). The gate asks `cprsmall.def` for 149 creatures' portraits
  (`status::ARMY_CREATURES`: the installation's DEF, 0 to 148).
- **M3 T7b2, the harness's staging** (*provisional*, F5-1, F5-4, F5-5, F5-7): a run's start
  releases the hotkeys and the button before its staging; `choose_each` waits for the window's
  name item; a pointer in a scroll zone is left out of the look; the button paired as the keys
  in a sequence's inputs.
- **M3 T7a, a map-named town in the hover facts** (*provisional*, the harness): `Mem.hover`'s
  `towns` give a town whose name is not a record of `TownName.txt` (a shipped map's own) as 65535,
  so a still of such a map is compared; a hint over that town shows an empty name (T7b2's F3-3)
  until M4 reads the map's town names.
- **M3 T6a2b, the owners and contents** (*provisional*, design §4.12): the live part's `[hover]`
  contents (sparse rows by object: `mines`, `banks`, `shrines`, `trees`, `witch_huts`, `quests`,
  each holding what the hint reads already decided for its player: the viewing, the shown or the
  current one, one in the comparisons; the engine's from M4), the Tent's mark a `visits` rule;
  `Hint`'s eight new variants (`Hint` now `Clone`, not `Copy`); the catalogue's 34 keys and 13
  families (a quest's text a key per type with six context forms, SeerHut.txt's columns); a
  situation's `visits` with `hero` and `guards` (`h3mem.Mem.write_content`) and an object's
  `quest` (the writer's `Extras::quests`, the adventure lane's crate, additive). The Quest Log's
  state is the shown item's now (`Mem.item_states`), not recomputed from the quests. Not recorded:
  the players apart (hot seat, M5), a defeat-hero quest without its hero (MCL-Q21), the skills
  quest's spaced texts (SeerHut.txt's variants 2 and 3: the new game drew variant 1 for it; the
  other types' variants have one text, several drawn).
- **M3 T6b1, the buttons' press, the hotkeys, the lists' clicks** (*provisional*, design §4.13):
  the timeline's `Action::Press`, `Release`, `Key`, `KeyUp` (a sequence's `press`, `release` and
  the hotkeys' letters; the window's presses and releases apart); `Hotkey` and the key table's
  eleven commands (`kingdom-overview` … `end-turn`, the original's letters); the live part's
  `[chosen]` (each hero's place, status window and path, each town's place and status window: the
  harness chooses each before 0, a situation's `choose`; `Live::at_start` what a map gives) and
  `pressed`; a command the M3 view does not build refused in a recording, nothing live
  (`Unplayable::Command`: the dialogs and the engine's actions). Not built here: after a choice
  the mouse-over's reach and the hero's own marks stay the staged hero's (the engine's from M4:
  the sequences move the pointer over the map only before their first choice); the release's hint
  is recorded, not read; the hotkeys of the sequences held to their end (their commands' dialogs
  after it). The original's hint hidden by a list's redraw: MCL-Q22.
- **M3 T6b2, the underground, the minimap's click, the status window's** (*provisional*, design
  §4.14): the tables' minimap `click` (each map size's divisor, by its f32's bits) and
  `[map.status] asked` (3 000 ms); the live part's `realm` (the kingdom's window as the original
  would build it, a `status` of kind 2: what a click shows) and `Live::at_start`'s from the map (the
  towns' halls: the start's Town Hall and City Hall bits now read; the players a human or the
  computer may play, by the map's teams); the situation's `underground` (terrains alone; the
  harness explores it whole and waits for `adventure`); the selection's window kept while a click
  shows another. Not built: the minimap's right click (its help text, M4, M5); a hero or town of the
  underground in a recording (the harness reads the view's level's heroes). The original's lost
  press on a rebuilt status window: MCL-Q23; a hotkey's release eaten by the minimap's loop:
  MCL-Q24.
- **M3 T6b3, the keys during a walk, the sound without the focus** (*provisional*, design §4.15):
  the key table's `visit` (Space) and every map key's release an action of its own (`Up`: a
  message in the ring that does nothing); the ring's count (a click two messages, everything else
  one: a scroll's Control key and a button's move before it not counted); a visit Space would
  make (the hero on an object's entrance) refused in a recording, nothing live (the engine's); the
  window's effects held (their streams unbound, `h3_sdl::Audio::hold`) and the music faded by the
  model without the focus. Not built: the window's wait for a music fade at the focus's loss
  (MCL-Q26), the system's cursor without the focus (the window keeps its own), the videos. Nothing
  stops the steps a held key queued, Space included: MCL-Q25.
