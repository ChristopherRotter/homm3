# E24 — The adventure map client

Status: **refined** (2026-10-23, T0, with the user); **M1 closed** (2026-10-24, T6b: the
reviewers, the review list accepted by the user); next M2 T0, M2 refined with the user
(`docs/lanes/map-client/QUEUE.md`).
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

### M2 — the map moving (refined in M2 T0)

The objects' animations and the terrain's colour cycles over time, the view scrolling, a hero's
walk along its path (the path's arrows), in a window at the original's frame times (`h3-sdl`),
checked frame by frame against recorded sequences as E17 M2 did for the battle.

### Later (refined when they come)

- **M3, the adventure screen:** the minimap, the hero and town lists, the status window, the
  resource bar and the date, the buttons, the mouse and its cursor; the string catalogue's keys.
  Needs `h3-view`'s `ui`, `text`, `clock`, `layout` (and `sound` for M2's window) in a common
  crate: a `lanes.toml` change (shared) agreed with the client lane (its inbox has the note).
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
