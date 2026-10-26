# E24 log: done tasks in full

The full record of E24's finished milestones and tasks (what was built, agent decisions marked
provisional, measurements, the review lists). **Not start-of-session reading:** the epic file
keeps one line per done task; read the matching section here when a task builds on an earlier
one or a decision is questioned.

## Done

### T0 — E24 refined with the user (2026-10-23)

One session, the user present; branch `map-client/e24-t0` (docs only, `cross-lane` for the
BACKLOG row, the user's consent). The agent read the lane's docs, `clients/README.md`, E17's epic
file and design notes §6.3 (the harness on the adventure map), `h3-map`'s raw model and import,
the executable map's adventure entries, and the adventure lane's state; it proposed the
milestones and M1's tasks, and the user took every recommendation:

- **M1's scope:** the map view and the screen's frame; the panels in M3 (over the whole
  adventure screen in M1).
- **The order:** the map moving (M2) before the panels (M3).
- **Test maps:** this lane adds what it needs to `h3_map::writer` in additive `cross-lane` pull
  requests (over the adventure lane's inbox or a writer of the lane's own).
- **The string catalogue** stays common (T9c's call, confirmed).
- **The BACKLOG row** in this pull request, `cross-lane` (over a `shared/` one).

What shaped the proposal (the epic file's "How the viewer sees a map" keeps the result):

- `h3_map::import` keeps no pictures and imports 5 of the 160 shipped maps whole; `RawMap` keeps
  every tile's terrain, river and road with their pictures and flags (`model::Tile`) and every
  template's DEF, masks, class, group and priority (`model::objects::Template`): the static art
  comes from the raw map, the live part from a second input.
- The writer has terrain patches (whole rectangles, so the transitions are whatever pictures it
  writes), straight roads (pictures 15, 12, 13), heroes, monsters, piles, grounds, plain objects
  of `PLAIN_OBJECTS`' five classes (the Cartographers, three kinds of trees, the Border Gates),
  towns and an underground level; no rivers, no free tile pictures, no other templates.
- The executable map already names the per-tile draw lists (0x505B20, 0x505230, 0x505060) and the
  shadow pass (0x47D930); the harness knows the view's geometry and its top-left in memory.
- A shipped map's random objects are drawn at the game's start in real time (E17 design §6.3).

Inbox notes written: the adventure lane's (the writer's additions to come; the object identity
M4 needs), the client lane's (the common crate M3 needs).

### M1 T1a — code reading: the view's tiles and the moment (2026-10-23)

One session, branch `map-client/e24-t1a` (docs only: the design notes §1, `docs/exe/map.toml` and
`structs.toml`, MCL-Q1, an inbox note). The user kept the task whole when asked for a seam
(the agent had proposed the moment as a second part).

- **Path:** the adventure manager's DEF loads (0x406FD0) led to the view draw 0x40F3F0 and its
  eleven passes; read in full: the terrain and edge, river, road and unexplored-tile passes, the
  unexplored tiles' mask and table, the run-shadow drawer 0x47E820 (formerly
  `draw_frame_variant`, inferred), the corner gems, the tick 0x4ED490 with its palette ranges, the
  idle step's animation counter; the object and path passes only for their place in the order and
  their frame rule (T1b reads them).
- **Findings that change earlier text:** a tile's left edge is at 32 (x − vx), not 8 + 32 (x − vx)
  (E17 design §6.3 had it from clicks; the epic file and `structs.toml` corrected); the view has no
  pixel scrolling (+0xF4, +0xF8 stay 0); the four corner gems are drawn by the view, after the
  shroud, in the viewing player's colour; objects get a random animation phase per object at load.
- **VCMI 1.7.5** (§1.7): agrees on the tile layers, mirrors, the road's offset, the edge's
  corners and sides, which hidden tiles get a piece, the palette ranges and direction; differs on
  the place and clip (one pixel), the far edge pattern for odd negative coordinates, the full
  piece's frame, 16 of 256 edge-piece masks and the parity variants, the shadows' rounding, the
  palette step (180 ms continuous against 200 ms polled) and the objects' phase. The edge-piece
  comparison ran VCMI's table through the original's masks (a throwaway script).
- **Agent decision (provisional):** a map whose picture lies past its DEF's frames is refused by
  the viewer (the original reads past the table).
- **Questions:** MCL-Q1 (the bottom edge's x test against the height; harmless). The mouse's
  possible 8-pixel offset stays an open point for M3 (§1.8) until a capture shows it.
- **Verify:** `cargo xtask check-exe-map`, `check-docs` and `check-lane` passed; `scripts/exe.py
  check` agrees with the executable (44 entries added, one renamed and read).

### M1 T1b — code reading: the objects' draw order and shadows (2026-10-23)

One session, branch `map-client/e24-t1b` (docs only: the design notes §2, `docs/exe/map.toml`
and `structs.toml`, MCL-Q1 moved to answered, a note in the adventure lane's inbox). The user kept
the task whole when asked for a seam (the agent had proposed the draw lists and object passes
first, heroes, boats and the path second) and answered MCL-Q1 (the viewer tests the width).

- **Path:** placement (0x505B20) and its two helpers (the layers 0x505060, the insertion
  0x505230) in full; passes 4, 5, 6, 7, 8 in full, pass 9 for its role; the hero and boat
  collectors and pieces (0x40FB30, 0x40FE30, 0x410760 and their shadow twins) and the tile's owner
  (0x40FD20) in full; the drawers' wrappers; the hero, flag and boat DEF tables in the manager's
  start. Not read: the moving hero's pieces and the attacking monster's substitute (M2), the
  puzzle's class filter, the owners' tables (T5), 0x4FEC30 and 0x4CE900 (roles inferred).
- **Findings:** objects are drawn as 32 × 32 pieces per tile, from each tile's draw list ordered
  by its cells' layers (a cell's row above its column's base, reset at a blocked cell on a
  passable one, copied from a blocked right neighbour), equal layers by the later placement unless
  the earlier object is higher on a shared tile; flat objects in their own pass; every shadow
  (objects', heroes', boats', the path's) before every body; heroes and boats have no entries and
  are drawn as two-row objects after layers 1 and 2, the right over the left; a hero's flag under
  it; a boat's waves, flag and body from the manager's DEFs (never its template's), the waves
  skipped on the coast; bodies and shadows only in the cells of the `.msk`'s masks (H3API); flag
  pixels coloured for eight classes only. Pass 9 and the centre pieces are the moving hero;
  manager +0x1EC renamed `hero_moving`. Earlier text changed: §1.1's passes 4–9; the E08 survey's
  "not read" on 0x505230 answered in the adventure lane's inbox (nothing to change there).
- **VCMI 1.7.5** (§2.4): agrees on per-tile pieces, nothing off the map, flat objects lowest and
  boats drawn from their own DEFs; differs in ten points (D1–D10: shadows with bodies, one order
  per pair, its tie rules, heroes' place and order, the flag over the hero, the boats' layers,
  every cell drawn, the path's shadow, flag colours for every owner). §2.5 lists a situation per
  difference for T4, and a gate check of every object DEF against its `.msk` (T4).
- **Questions:** none new; the open points (§2.5) wait for T4's and T5's captures.
- **Verify:** see the pull request (`scripts/exe.py check`, `cited`: 23 entries added, 6 read or
  renamed, 0 uncovered).

### M1 T2 — the crate, the map's asset tables and the layout (2026-10-23)

One session, branch `map-client/e24-t2`. The user kept the task whole when asked for a seam (the
agent had proposed the scene's input types as a second part, with T3a).

- **Built:** `clients/crates/h3-map-view`, a member of the client workspace. Its pure library
  (`no_std`, the logic header, 100 % lines and regions; `tests/purity.rs` a copy of `h3-view`'s
  for this crate): `art` (the static art from `h3_map::RawMap`: each tile's terrain, river and
  road with picture and mirrors, each template's DEF, masks, class and flatness, each object's
  anchor and template by its map index; refuses only a tile count off the size, a template name
  that is not printable ASCII, an object without its template), `live` (the objects standing,
  a random object's drawn template, owners, heroes with class, owner, facing and boat, empty
  boats, the viewer and its explored tiles, the moment's two counters; `Live::at_start` from a
  map without a draw), `tables` (the format in `clients/README.md`; the shroud's piece rule of
  §1.5 and the frames each DEF must hold), `layout` (the frame, its picture and the players'
  palette, the view's origin, tile, loops, clip and gems), `check` (a DEF's group 0: frames,
  encodings, run pixels by index). Its binary: `gate`. Data: `clients/map/assets/base.toml` (the
  shroud's 256 pieces copied from 0x65F694 and checked against §1.5) and
  `clients/map/layouts/adventure.toml`.
- **Reading:** the frame picture's draw (0x417420's first part, §1.9): `AdvMap.pcx` opaque at
  (0, 0) in the viewing player's colours; two map entries. VCMI 1.7.5 agrees (its frame is parts
  of the same picture, player-coloured).
- **The gate on the user's files** (§1.9): every name found in `H3sprite.lod` or `H3bitmap.lod`
  and decoded; each DEF holds exactly the frames the view draws (`edg` 36, `tshrc` 4, `tshre` 34,
  the gems 8; the terrains, rivers and roads one past the highest picture of the 159 shipped
  maps, all read); terrain frames are encoding 0, so no run is skipped (T1a's open point closed).
- **Agent decisions (provisional, on the review list):** the tables keyed by the `.h3m`'s codes;
  the library depending on `h3-map` for the raw model; `Live::at_start`'s facing (right),
  explored tiles (all) and phases (0) until T5 and the comparisons read the original's.
- **Questions:** none (no oddity found).

### M1 T3a — the writer's additions (2026-10-23)

One session, branch `map-client/e24-t3a`, a `cross-lane` pull request into the adventure lane's
`h3-map` (the user's consent of T0 for additive changes; a note in its inbox).

- **Built:** `h3_map::writer::write_with(&Map, &Extras)` beside `write` (now `write_with` without
  extras), the new code in `crates/h3-map/src/writer/extras.rs`. `GivenTile` writes a tile's seven
  bytes as stated (`RiverKind` for rivers; no coast bit computed for it, its terrain counted for
  its neighbours'); `AnyObject` places any template but a hero's, a prison's or a placeholder's
  (34, 70, 62, 214: their bodies name a hero), after the map's own objects, at a position up to
  7 columns and 5 rows past the edges, with an owner where its class's body has one, and writes
  the body the editor gives an object left as placed.
- **The bodies' defaults** (`docs/formats/h3m.md`, "The map client's additions"): a survey of the
  56 SoD maps' bodies (a throwaway program over `h3_map::read`, not committed) gave the editor's
  choices: monsters aggressive, events for every player and removed after their visit, scholars
  random, garrisons removable, shrines random, witch huts all skills but Leadership and
  Necromancy, abandoned mines all but wood, towns the standard buildings with a fort. Objects
  anchored past the right and bottom edges are common there (decorations up to 5 and 2 past).
- **Tests** (`writer::extras::tests`): a map of every kind the writer wrote before keeps its bytes
  (length and CRC-32 of `master`'s writer at 328b808); given tiles over a road, a patch and the
  underground read back as given and make the coast; extra objects reuse a map template and
  continue the identifiers; every class 0–255 but the four reads back with the expected body (so
  the writer's bodies have the reader's sizes); each refusal.
- **Agent decisions (provisional, on the review list):** the second entry point rather than new
  fields and variants (those would have changed literals and matches in the battle, client,
  adventure and map-client lanes' crates); the defaults without an editor's choice seen (a
  scroll's spell 0, the grail's radius 0, a random dwelling's levels 0–6).
- **Questions:** none.

### M1 T3b1 — the tiles drawn: `render` (2026-10-23)

One session, branch `map-client/e24-t3b1`. T3b was split with the user at its start: T3b1 the
`render` command and the tile passes, checked on the writer's maps and by a look at shipped
maps; T3b2 the harness side and the first exact comparisons (`QUEUE.md`).

- **Built** (the library, 100 % covered): `scene` (the draw list at a view: the frame's picture
  in the viewer's colours, then passes 1 terrain and edge, 2 rivers, 3 roads, 10 unexplored
  tiles, rows outer and columns inner, then the gems; each piece with its clip; the tables'
  cycling DEFs with the moment's steps; a level, a viewer or a code the tables lack refused);
  `draw` (the map's two drawers: `draw_frame_raw`'s literal pixels mirrored both ways, 0x47E820's
  runs as shadows); `batch` (the decoded DEFs, pictures and palettes by name); `compose` (the
  frame's picture opaque with entries 224–255 the viewer's, each DEF's palette cycled once, the
  unexplored tiles drawn twice, the gems plain with runs skipped); `Moment::at` (a time on a
  clock that is never late). The binary: `render` and the gate's file helpers shared
  (`files.rs`).
- **Read in the executable** (§1.3, §1.10): the road pass's cut (0x411E67 to 0x411ED3), and
  `draw_frame_raw`'s source rectangle taken in the mirrored picture's space (0x47DDD9), so the
  cuts are a clip on the screen. The last-row road's cut comes after the view's: at the view's
  bottom row the road vanishes (MCL-Q2; `Bugs::Original` reproduces it).
- **Checked:** the writer's maps in the scene's tests (given tiles: a lava terrain mirrored, a
  mud river and a cobblestone road mirrored, roads on the map's last row at the view's rows 15
  and 16 under both bugs settings, the edge around the map, the shroud's masks and pieces, the
  full piece's frame, an empty table entry); the gate unchanged after the refactor; shipped maps
  rendered and looked at (Deluge's roads, water and a hidden rectangle; Dragon Pass's rivers;
  Adventures of Jared Haret's edge and icy river; Deluge's top-left beyond the edge): seamless.
  Every frame of the twenty view DEFs has a 32 × 32 canvas (a one-off read).
- **Agent decisions (provisional, on the review list):** the drawers and the batch in the map's
  crate rather than the foundation's (no client-lane path changed); the dependency on
  `h3-render`; `render`'s flags.
- **Questions:** MCL-Q2 (the last row's road at the view's bottom row; recommendation: fix, the
  original's only for the comparisons, as built).

### M1 T3b2 — the first exact diff of the tiles (2026-10-23)

The same session as T3b1, branch `map-client/e24-t3b2` (on T3b1's, #428), after the user asked
for the second part too.

- **Built:** `h3_map_view::situation` (the situations' TOML: the map's side, terrain, patches and
  roads as the writer takes them, rectangles copied from shipped maps by file name and level,
  tiles given one by one, Red's and Blue's heroes, the view, the hidden rectangles; built into
  the writer's `Map` and `Extras`; 100 % covered); the binary's `situation` (the copies' maps read
  from the installation copy's `Maps`); `render --cycles` and `--hidden` repeated, and a
  situation's uncompressed map read as well as a shipped gzip one. The harness (common path,
  additive): `h3mem.Mem.write_explored`, `def_palette`, `frame_steps`; the new driver
  `clients/capture/h3map.py` (`still`, `set`); the runbook's "The map's stills".
- **Found:** the virtual clock drives the adventure map (frozen: two dumps 1.5 s apart the same;
  a 400 ms step moves the hero's flag and the panels); the menus run in real time, so each run's
  palette step k differs (9 in the first run) and is read from memory: each cycling range in
  memory is its file's rotated up by k, one k fitting all five DEFs. Renders at k ± 1 differ by
  about 44 000 pixels, at k + 36 by none. A minimap click reaches views past every edge and the
  view whose bottom row is the map's last: MCL-Q2's road is missing there in the original
  (exact with `--bugs original`; 2 382 pixels differ drawn fixed).
- **The map's capture set** (`clients/map/situations/set.toml`): `deluge-coast`,
  `edge-top-left`, `edge-bottom-right`, `road-last-row-19`, `road-last-row-20`, `rivers-clear`,
  `rivers-mud`, `rivers-lava` (an underground copied to the surface), `rivers-icy`,
  `lava-water`, `unexplored`: every one 0 differing pixels outside the panels (about a minute
  each). The second full run stopped at `rivers-icy`: its palettes fitted no single k, a read
  that met the tick's rotation half done (`palette16_rotate` moves the entries one by one); the
  harness now reads until two reads agree and checks again after the dump. The third run
  differed in one situation by 42 035 pixels of water, the screen a palette step behind memory:
  the idle step redraws the view only at the frame deadline (0x408A88, every 180 ms) while the
  tick rotates whenever polled (design §1.6). The harness now steps the frozen clock to the
  later of the next tick and the next redraw and waits until both have come (the clock's 20
  reads can come before the main loop's idle step: the fourth run stopped there) before the
  dump; the fifth and sixth runs as recorded (palette steps 10 and 11).
- **Agent decisions (provisional, on the review list):** the situation format; writing Red's
  explored bits instead of revealing by play; a single scenario; the masks.
- **Questions:** none new; MCL-Q2's context updated (reachable).

### M1 T4a — the objects on the writer's maps (2026-10-23)

One session, the user present at its start; branch `map-client/e24-t4a`.

- **The split** (with the user): T4 became T4a (this), T4b (the masks' gate check and shipped
  maps' views with their random objects read from memory) and, moved into T5, the heroes' and
  boats' place among the objects' layers (D4, D5) and the path's arrows (D9). MCL-Q2 answered:
  the client draws the last row's road, the comparisons reproduce the original (as built).
- **Read:** msk_load 0x514610's bit layout and which mask is which (the first the colour mask,
  the second the shadow mask), tile_draw_insert 0x505230 and object_tile_layers 0x505060 again
  (§2.1 holds, the intersection clipped to the map), pass 7's frame and source rectangle
  (0x410EA1 to 0x410F3E: the first group's frame count). A survey of the user's sprite archives
  (a throwaway script, not committed): all 1 659 object DEFs encoding 3, first group id 0, every
  body and shadow pixel inside its `.msk`'s masks but the radar's (no map object); swapped masks
  miss hundreds (design §2.6).
- **Built** (`clients/crates/h3-map-view`): the module `objects` (`Shape`: a `.msk`'s size and
  masks and the DEF's frame count; `Shapes` by DEF name; `layers`; `DrawLists::build`: the
  placement in the map's order and the insertion; `pictures`, `template`); the scene's passes 4,
  5 and 7 (`Pass::Flat`, `Shadow`, `Body`; `scene` takes the shapes; `SceneError::Objects`); the
  drawers `draw::flagged` (0x47D0A0) and `draw::shadow_pass` (0x47D930) with the view's clip;
  compose by pass. The binary: `files::load_shapes` (the `.msk` searched in the sprite archives,
  `default.msk` without one; the DEF's first group), `render --live <live.toml>` (`cycles`,
  `frames`, `phases`, `anchors` checked against the map's). The situations: `copy` takes
  `objects = true` (the objects anchored in the rectangle, but the hero classes, neutral, the
  editor's bodies) and `objects` gives objects by their template's DEF (`Objects.txt` from the
  bitmap archives, by DEF and optionally class and subclass). 100 % covered.
- **The harness** (common path, additive): `h3mem.Mem.map_objects` (the 12-byte records: anchor,
  template, phase); `h3map.py still` writes the frame step the screen shows (memory's less one),
  the anchors and the phases into `live.toml` and draws ours with `--live`.
- **Captures:** six new situations, each exact outside the panels on its first run:
  `objects-mandate` (The Mandate of Heaven (24, 24)-(45, 43), 159 objects), `objects-witch` (Good
  Witch, Bad Witch (12, 56)-(33, 75), 161), `objects-flat` (the same map's (56, 8)-(77, 27), 116,
  47 flat), `objects-myth` (Myth and Legend (12, 12)-(33, 31), 148), `objects-edge-top-left` and
  `objects-edge-bottom-right` (given objects cut by the four edges). The copies' rectangles were
  picked by a throwaway scan of the shipped maps for dense corners without heroes, random or
  owned objects. Counted in their views: D1 2 679 tile pairs, D2 17 pairs, D3 406 entries, 207
  flat pieces under shadows, so §2.5's own maps for points 1 to 3 and 9 were not built. One frame
  step more changes 620 pixels of `objects-mandate`. The first full run of the set (17) stopped
  at `objects-witch`: the original crashed on its way to the map (a null read at 0x4CCF8C).
  Read: the new game's text rumour (`rumour_pick_text` 0x4CCF20) draws n among the unused of 256
  rumours but copies row n, so a draw of 256 reads one entry past its table, about once in 800
  games; a finding for the adventure lane (its inbox), and `h3map.py` now starts a game that
  crashes on its way to the map once more. The second full run: every one as recorded.
- **Agent decisions (provisional, on the review list):** the shapes loaded by the binary and
  passed to the scene; `render --live`; the situations' objects; the owned classes' flag runs
  skipped until T5 (the copies avoid those classes).
- **Questions:** none new.

### M1 T4b — random objects and shipped maps' views (2026-10-23)

- **The seam** (asked): split T4b into the gate check with plain shipped views and the random
  objects; the user had it done whole.
- **Read** (§2.7): resolve_random_objects 0x4C9DD0 runs at the new game's setup, after the load
  placed every object (its entries and phase); object_template_redraw 0x4C9990 copies the
  object's template and replaces only the DEF name, class and subclass (an artifact `ava%04d` by
  id 0x677974, a monster the last class-54 template of its creature 0x505EA0, a pile by
  resource 0x677958, a town by its buildings 0x677A0C …); random_artifact_of_level 0x4C94D0;
  the towns' setup 0x4CAA70 (inferred); 0x505A10. VCMI 1.7.5 gives the new type its own template
  and places it again (D11).
- **Found:** the copy keeps the random template's masks: the gate's survey over `Objects.txt`
  and the shipped maps' templates (1 333 pictures, none with pixels outside its own `.msk`: D8)
  lists the pictures a random artifact's masks cut (44: 2 body pixels; `AVA0130` … `AVA0141`:
  10 to 584); `ava%04d` is one off from 128 (the editor names `AVA0129` for 128), so a random
  relic become Angelic Alliance shows Armageddon's Blade's picture (both looked at with `h3-view
  sprite`; `AVA0128` is a box no template names). 104 of the 160 shipped maps allow the
  combination artifacts. **MCL-Q3** asked and answered (a): the client draws the own template.
- **Built:** `h3_map_view::check::{undrawn, undrawn_frame, Undrawn}`; `live::Drawn` (the copy's
  DEF, the own DEF, class, subclass) in place of a whole drawn template; `objects::{template,
  def, shape}` (placement by the map's template, the drawn DEF's frames with its masks, or the
  own picture whole when the bugs are fixed); the gate's object pictures; `files::{load_shape,
  templates}`; `render --live` reads `viewer`, `defs`, `classes` and fills `own` from
  `Objects.txt`. 100 % covered.
- **The harness** (common path, additive): `h3mem.Mem.map_templates`; `h3map.py`: a situation's
  `map` loads a shipped map whole (`to_shipped`: the scenario list's setup, the first day's
  messages closed by OK, the screen `adventure`), `live.toml`'s `viewer`, `defs`, `classes`. Its
  robustness, from this sandbox's runs: a game whose Wine exits before the main menu with an
  empty log is started again; before counting redraws `still` waits for the deadlines the frozen
  clock passed, and accepts a second redraw at the same time; `set` shows a situation the harness
  fails to show once more (with why); the palettes' failure names each range's fits.
- **Found in the runs:** a shipped map's random dwelling can stand a tile from the map's anchor
  after a new game (Tale of two lands, one start of four, underground, outside the view): not
  read (resolved outside 0x4C9990); `render --live` accepts a random object's moved anchor and
  draws it at the map's until T5. `check-stubs` reads only `crates/`, so a client stub cannot be
  registered: the gap is in §2.7 and the queue's T5.
- **Captures:** `shipped-divided-loyalties` (8, 38), `shipped-tale-of-two-lands` (31, 61),
  `shipped-good-witch` (82, 48), `shipped-gauntlet` (59, 51), `shipped-dwarven-tunnels` (11, 23,
  underground), the views picked by a throwaway scan (no hero, boat, random hero, placeholder or
  owned class within reach, the most random objects of each kind): each exact on its first run.
  The full runs (22): the first two broke at `lava-water` while cargo built beside them (Wine
  exited before its menu; unsteady palettes), the next ones alone met the flakes above (an extra
  redraw, unsteady palettes three times, the moved anchor), each shown again exact; the last:
  every one as recorded, three shown again for unsteady palettes. Those reads had the terrains'
  palettes (`lavatl`, `watrtl`) one step ahead of the rivers' for the whole 2 s: the tick's two
  halves apart at the frozen clock (queued for T6: the queue's inbox).
- **Agent decisions (provisional, on the review list):** the shipped situations' format and
  loading; `live.toml`'s new keys; `Drawn` and the own template's choice (the last of
  `Objects.txt`, as 0x505EA0); the gate's listing.
- **Questions:** MCL-Q3 (answered).

### M1 T5a — heroes and boats (2026-10-23)

- **The seam** (asked): T5 split into T5a (heroes and boats) and T5b (owners' colours, the path's
  arrows, the explored tiles), the user's choice of two.
- **Read** (§2.8): hero_map_group 0x4D9110 and boat_map_group 0x4D9160 (one table: facing to
  standing group 0–4, mirrored above 4), the pictures' frames (group 0's of each DEF: a hero's and
  boat's one, a flag's eight), view_hero_piece 0x40FE30 and its shadow 0x4102C0 again (the
  hero's facing drives a boat's pictures; the shadow pass checks the owner in a boat too:
  0x410325, never seen), view_boat_piece 0x410760, hero_boat 0x4CE900 (the boats' vector at game
  +0x4E3BC, `structs.toml`'s `boat`), the coast (map_tiles_load 0x4FE220's +0xD bit 0x02).
- **Found by the original's memory:** a new game faces every hero and boat right (2), a map's
  boat has colour −1; a hero the map places on water gets no boat (on horseback on the water;
  the adventure lane's inbox has it); boarding takes the day's points (hero_boards_boat
  0x4A1010), so no hero sails on its first day; at a human player's turn start the selected hero
  is off its tile (manager +0x1EC set) until the view moves, which the harness's first facing
  write met.
- **Built** (`clients/crates/h3-map-view`): the tables' `[map.heroes]` (`HeroArt`: the DEFs by
  class, colour, type, type and colour, the facings' groups, each list checked for its length;
  `DefNeed::groups`); `TileArt::coast`; `Shapes`' frames of pictures without a `.msk`;
  `Draw::Piece`'s group (compose draws it); the scene's heroes and boats by tile, pass 5's six
  slots after the objects' shadows and pass 7's after layers 1 and 2; the gate's groups check
  (`check::problems`); the situations' `heroes` list (`HeroSpec`, `hero_type`) and `boats`
  (`BoatSpec`), the 17 earlier situations rewritten (their maps byte for byte as before);
  `render --live` reads `heroes` and `boats`, loads the hero DEFs' frames. 100 % covered.
- **The harness** (common path, additive): `h3mem.Mem.{boats, standing, map_heroes, hero_on,
  write_facing, write_bytes}`; `h3map.py still` walks Red's heroes (`arrange`, `h3adv.walk`),
  turns heroes and boats after the view moved, writes `heroes` and `boats` into `live.toml`.
- **Captures:** a probe of 12 heroes and 5 boats facing right, then turned: exact at once.
  `heroes-classes-0` to `-2` (144 heroes), `heroes-objects` (10 heroes among The Mandate of
  Heaven's objects: D4 on (16, 7) and (8, 14), found by a throwaway scan of the original's draw
  lists), `boats` (Deluge's lake; 3 boarded, 3 empty): each exact on its first run. The full set
  (27) after: every one as recorded, none shown again.
- **Agent decisions (provisional, on the review list):** the situations' format, the walks and
  written facings, the boarding tiles' open pictures, `[map.heroes]`, `Shapes`' frames,
  `Draw::Piece`'s group; the boats' flags of other colours not captured.
- **Questions:** none (the shadow pass's owner check differs from the body's only for a hero
  without an owner, which the map never has).

### M1 T5b1 — the owners' colours, the towns' pictures, the random dwellings (2026-10-24)

- **The seam** (asked): T5b split into T5b1 (the owners: colours, towns by their buildings, random
  dwellings) and T5b2 (the path's arrows, the explored tiles), the user's choice of three.
- **Session:** no GitHub in this session (no linked account in the cloud, no credentials on the
  user's computer); the user chose to work without it: the sandbox cloned from a bundle of the
  user's repo (its `origin/master` held #444's merge), the branch handed back as a bundle for the
  user to push and open the pull request.
- **Read** (§2.9): tile_object_owner 0x40FD20 and object_entrance_tile 0x4FEC30 (read now), the
  passes' colour (game_palette16's +0x9C + 2 × owner: entry 64 + owner, 63 none; §2.2 said 78,
  forgetting the palette's +0x1C header), the passes' jump tables on the drawn class, the
  template's class as loaded (map_template_read 0x503780 through object_class_values 0x63A6E4:
  219 as 33, 220 as 53, decorations as others; found when the original's memory showed the
  vertical garrisons as class 33), the towns' pictures (0x4C9990's town branch, the three tables,
  setup_towns 0x4CAA70 redrawing every town after town_buildings_setup 0x5C08C0, the map's
  building bits 0x6888C0), the random dwellings (random_dwellings_resolve 0x502B60 and
  object_set_class_template 0x505F20, from map_objects_place_all before the placement).
- **Built** (`clients/crates/h3-map-view`): the tables' `[map.owners]` (`OwnerArt::entry`) and
  `[map.towns]` (`TownArt::picture`, `FACTIONS`, the building ids); `Draw::Piece`'s `flag`,
  `Scene::flags`, the batch naming the palette, the compose's flag colour (0 none);
  `objects::{class, entrance, flag_owner}` (MCL-Q4's tile (0, 0) with `Bugs::Original`);
  `Live::at_start` gives the towns their pictures (`built_at_start`); the situations' `towns`
  (`TownSpec`, `TownBuilt`) and objects' `owner`; `render --live` reads `owners`, checks the
  towns' pictures, takes a random dwelling's template and moved anchor from memory, keeps an own
  picture for artifacts only; the gate lists the owners' palette, the pictures with flag pixels
  by class, the towns' pictures with each town template's masks (none loses a pixel) and the
  shipped maps' owned objects without an entrance on the map (none). 100 % covered.
- **The harness** (common path, additive): `h3mem.Mem.map_owners` (the original's walk and
  records); `h3map.py still` writes `owners` into `live.toml`.
- **Captures:** `owners-objects`, `owners-random-dwellings` (run until an Angel Portal came: its
  anchor one right of the map's, exact), `towns-0`, `-1`, `-2`, `towns-random`, `shipped-hoard`,
  `shipped-too-many-monsters`: each exact on its first run. The full set (35) after: every one as
  recorded, three shown again for the palettes' half tick (the inbox's T6 item).
- **Agent decisions (provisional, on the review list):** the tables' formats, the scene's flag as
  a palette entry, the owner per object, the situations' formats, the heroes outside the view
  making the owning colours players, `own` for artifacts only.
- **Questions:** MCL-Q4 (an owned object without an entrance on the map takes tile (0, 0)'s
  owner; none in the shipped maps): answered by the user at the task's end, (a) as built (the
  client the object's own, the comparisons the original's).

### M1 T5b2 — the path's arrows and the explored tiles (2026-10-24)

- **The seam:** T5b2 taken whole (the path's arrows, the explored tiles from memory, and the
  boats' flags of other colours, which proved cheap by a staging write).
- **Session:** GitHub reachable from the cloud session again (the repository attached); the
  starting point's clients check (master `d0d7bb8`, its clients code not verified since E08
  T49a) passed in full in a worktree.
- **Read** (§2.10): view_path_shadows_pass 0x411F50 and view_path_arrows_pass 0x4120B0 in detail:
  the u16 per tile at the manager's +0x48 by (level × height + y) × width + x (`structs.toml`'s
  `path_arrows` now read), skipped at 0, `adag.def` frame a − 1, pass 6 the shadows alone
  (0x47BFA0 into 0x47E820), pass 8 the pixels (0x47BF50), the tile passes' clip. `adag.def`: 50
  frames, 32 × 32, encoding 2. How the arrows are chosen is M2's.
- **Built** (`clients/crates/h3-map-view`): the tables' `[map] path` (`PATH_FRAMES`, the gate
  checks it); `live::Arrow`, `Live::path`; `Pass::PathShadow` and `Pass::Path` in the scene
  (between the shadows and the bodies, between the bodies and the shroud), the compose's
  shadows-alone drawer for pass 6; the situations' hero `path` (Red's, one) and boat `colour`;
  `render --live` reads `explored` and `path`. 100 % covered.
- **The harness** (common path, additive): `h3mem.Mem.path_arrows`, `explored_rows`,
  `write_boat_colour`; `h3map.py still` sets the view by `set_view`, shows a hero's path
  (`show_path`), leaves a shipped map's start explored with `explored = "start"`, writes the
  boats' colours, and puts `explored` and `path` into every `live.toml`.
- **Captures:** `path-objects` (D9: the arrows over the decorations' upper parts at (8, 3) to
  (11, 3), their shadows under them, the first arrow over the hero; arrows 8, 16, 17, 25, 26,
  41), `path-edge` (cut at the view's columns 18 and 0), `start-dwarven-tunnels`, `start-hoard`
  (an explored circle at the map's right column beside the edge), `boats-colours` (Blue to Pink,
  the three types): each exact on its first run. The full set (40) after: every one as recorded,
  `shipped-good-witch` shown again for the palettes' half tick (the inbox's T6 item).
- **Agent decisions (provisional, on the review list):** the arrows from memory as frame numbers,
  the two passes, `live.toml`'s `explored` and `path` always written, the situations' `path`,
  `colour` and `explored = "start"`, the boats' colours staged.
- **Questions:** none (no oddity of the original found).

### M1 T6a — the capture set's digests and the half tick (2026-10-24)

- **The seam:** T6 split with the user into T6a (the set's digests, the inbox's half tick, the
  whole set captured) and T6b (the fresh reviewer agent, the review list, the checkpoint).
- **Session:** GitHub reachable from the cloud session (the repository attached); the starting
  point's clients check (master `f89aa89`, its clients code not verified since E08 T49b) passed
  in full, its coverage step built with this task's first change (the rivers' step, below) already
  in place.
- **Read** (§1.6): adventure_manager_start 0x406FD0 loads the DEFs kind by kind with a progress
  step between the kinds (load_progress_step 0x4ED2A0, read now: the loading bar's count, its
  pieces, the bar's rectangle shown through screen_show_rect 0x602BD0, read now: the tick before
  and after); the tick (0x4ED490) skips a DEF field still empty. A tick due at the step after the
  terrains rotates water and lava alone, and the rivers trail by one step for the whole game;
  within a kind no step, so the terrains share one k and the rivers one.
- **Built:** `h3_map_view::live::Moment::river_cycles` (the scene's rivers drawn with it;
  `Moment::at` keeps it with `cycles`), `render --live`'s `river_cycles`, `--cycles` setting
  both. 100 % covered.
- **The harness** (common path, additive): `h3map.py`'s `cycle_step` reads each kind's step
  (refusing any difference but the rivers one behind) and `still` writes `river_cycles`; `set`
  keeps each capture's `live.toml` as recorded or new in `map/situations/kept/<id>.toml`; `drawn
  [--only] [--record]` draws ours from the kept ones and compares the digests of the pixels
  outside the masks with `kept/drawn.txt` (`h3drive.png_digest`); the runbook's rule for the map.
- **Captures:** the whole set (40), every one as recorded on its first run (one game started again
  after the tavern rumour's crash), none with the rivers behind; then `rivers-mud` and
  `rivers-lava` shown again and again until the rivers trailed three times: 56 stills, the
  rivers a step behind in three (`rivers-lava` twice, `rivers-mud` once), every one exact (in
  this sandbox about one game in 30, T4b's about one in seven: the load's speed). Drawn with
  the rivers in step, the first such still of `rivers-lava` differs in 3 655 pixels of its lava
  river; with the rivers' own step it is exact. `drawn --record` (40), `drawn`: every one the
  same.
- **Agent decisions (provisional, on the review list):** the kept live parts as copies of
  `live.toml`, one digest per situation, `drawn` in `h3map.py`, `river_cycles`.
- **Questions:** MCL-Q5 (the rivers' cycle a step behind the terrains' for a whole game, about one
  game in seven): answered by the user in the task, (a) as built (the viewer in step, the
  comparisons the original's two steps).

### M1 T6b — M1's closure: the reviewers and the review list (2026-10-24)

On `map-client/e24-t6b`, the user present. **The user: T6b kept whole; the reviewers may run in
parallel.**

- **Starting point:** `master` at #456's merge; its clients code verified (#456's sandbox line),
  the lane's scoped verify without a failing step, `drawn` all 40 the same.
- **The reviewers:** three fresh read-only sub-agents, one per area (the drawing library; the
  binary, the tables, the gate and the situations with the writer's extension; the harness),
  each against the design's readings; 10, 15 and 12 findings, each checked against the code and,
  where it named the original, against the executable (`scripts/exe.py`: 0x4AABB0's removal,
  0x412378's 0 from 0x40F8C0, 0x4FEC30's bounds).
- **Fixed, the library** (tests beside each): R1-1, objects fallen before a still were left out
  of the lists, where the original builds every object's and takes a fallen one's entries out
  (remove_map_object 0x4AABB0, read now: the later entries move down in their order): an object
  between two others decided their order (the reviewer's case, a test); R1-2, a shroud entry 0
  draws the full piece (0x41237F), not only the mask 255 (the same pixels with the original's
  table); R1-3, `scene` refuses a live part that does not fit the map (the objects' count, the
  explored tiles' size, a hero, boat or arrow off the map, a hero without an owner); R1-4, a
  random object's drawn or own picture without a shape and a hero's DEF without frames refused,
  not skipped; R1-6, a shape cut to the 8 × 6 cells; R1-8, the scene's doc of `bugs`; R1-9, a
  scene of the underground; R1-10, a road's cycles refused. R1-5 and R1-7 checked by the gate
  (none of the 2 622 templates has a visitable cell outside its `.msk`; no artifact's own picture
  is larger than its random template), documented; MCL-Q4's approximation on the review list.
- **Fixed, the binary and its data:** R2-1, the gate without its maps directory or with a map
  refused passed: problems now; R2-2, `defs` without `classes` skipped silently; R2-3, `render
  --live` checks the original's view against `--view` (the file's steps and viewer override the
  flags, documented); R2-4, an owner past 7 became none: refused; R2-5, a random dwelling's anchor
  only to the right on its row, its template the original's or refused; R2-6, the neutral entry's
  room; R2-7, a missing town picture aborted the gate's report; R2-9, an object picture whose first
  group is not group 0 a problem (D8 checks group 0, the passes draw the first); R2-10 to R2-12,
  a situation's comment, the usage text, three messages (the writer's error, a ninth hero, a
  shipped map's situation given to `situation`); R2-13, `--hidden` clamped to the map; R2-14,
  `read_live`'s refusals tested, the tables' owners, towns and heroes pinned; R2-15, the purity
  test's holes (any `bin` folder skipped, lint groups and attributes over several lines unseen,
  the header as a substring; `h3-view`'s copy: the client lane's inbox).
- **Fixed, the harness:** R3-1, `still` wrote into the game's memory before checking the
  executable: `game_mem` checks xdd.dll's redirected calls and that the process read is the game
  started (its clock directory); R3-2, a viewer's refusal or any failure stopped `set`: the
  situation fails alone (`REFUSED`, `FAILED`), the set goes on; R3-3, the staged facings and
  boats' colours read back, a boarded boat's facing refused; R3-4, `to_shipped`'s wait as
  `Game.wait`; R3-5, `still`'s exit code, the docstring; R3-6, the viewer's last three lines;
  R3-7, `still.xwd` checked to be the screen compared; R3-8, a game left running by an
  unexpected failure on its way; R3-9, timeouts naming the state at the timeout (`wait_for`
  takes a function, additively); R3-10, a kept live part records its situation's digest, `drawn`
  says `STALE`; R3-11, a shipped map's situation's keys checked; R3-12, the docs, `map.toml`'s
  `frame_deadline` and `remove_map_object`, `structs.toml`'s game vectors, the hero's object
  under it, the loaded DEF's palette.
- **Queued:** R2-8, the writer's edge bound one too generous (the adventure lane's crate: the
  next writer pull request, QUEUE's inbox); a loaded saved game's draw lists (M4).
- **Captures:** after the fixes the whole set (40), every one as recorded on its first run (the
  rivers a step behind in three, each exact); `drawn --record`, `drawn` all the same; the kept
  live parts carry their situations' digests.
- **The review list** (T2 to T6b, summarised in the session) accepted by the user. **M1 closed;**
  M2 T0 queued.

### M2 T0 — M2 refined with the user (2026-10-26)

One session, the user present; branch `map-client/e24-m2-t0` (docs only). Starting point: #460
merged (2026-10-24), `master`'s CI green, the lane's scoped verify without a failing step. The
agent read the lane's docs, the epic, E17 M2's record (its tasks, design §7.8, §7.9: the
timeline and the frame check), `h3-view`'s `clock`, `sound` and `cursor`, the harness's
`h3map.py` and the executable map's adventure entries; it proposed M2's scope, the recording's
format and seven tasks. The user took every recommendation:

- **Walks scripted in M2** (the user asked first whether this was for tests only or real: the
  walk is built for real, its input a hero and its tiles; in M2 the tiles come from the
  recorded sequences, from M4 the engine's records, from M5 the engine's answer to a click; the
  client never finds a path itself, and wiring the engine now would pull M4's seam work in: the
  observation's object identity, the import's 5 of 160 shipped maps, movement points).
- **The map's sound in M2** (T5).
- **`h3-view`'s `clock` and `cursor` copied** into `h3-map-view` (*provisional*); one common crate
  proposed at M3 T0.
- **Recorded and compared as E17 M2:** every frame at 1-ms steps, the timelines committed, the
  frames compared pixel by pixel outside the panels, `drawn` over the sequences.

What shaped the proposal: the original scrolls by whole tiles (the manager's pixel offset
+0xF4, +0xF8 is written only by its constructor, 0x406DF0); the idle map's moment is read
(§1.6), so M2's idle work is its schedule; the moving hero's passes are mapped but not read
(0x47F860, 0x47FB40, 0x47FC80), as is the move's animation in `hero_move_step` (E08 read its
pricing). The queued M1 items: R2-8 waits for a task that needs the writer (none planned: T6
then makes its pull request); a saved game's draw lists stay M4's.

### M2 T1a1 — the recording and the map standing still (2026-10-26)

T1a was cut in two before it started (the user, at the session's start: the queue's own seam):
T1a1 the recording and the idle loop, T1a2 the scroll with the speed options and the inputs.

- **Read** (`E24-design.md` §3.1; map entries `executive_loop` 0x4B0E40, `input_next_message`
  0x4EC590, `pump_messages` 0x4F7FB0, `network_keepalive` 0x5546B0 and `turn_timer_poll`
  0x557AF0 (inferred), `tick_60ms_time` 0x6989AC; `adventure_tick`'s and the manager
  procedure's entries extended): the managers' loop takes the tick before each message and never
  waits; the idle step redraws at the frame deadline and shows at once; the tick's 60-ms calls
  draw nothing in a single game. H3API names the input manager's +0x38 `messages`, 64 of them,
  as read. VCMI compared (§3.3): continuous time, a step every 180 ms for both.
- **Built:** `h3map.py frames [--only <id>] [--out <dir>]` (an additive harness change: `capture`
  split into `stage` and `live_part`, shared), `clients/map/sequences.toml` (a situation and `ms`;
  T1a2 and T1b add the options and inputs), the timelines `clients/map/timelines/<id>.txt` and the
  live parts at 0 `timelines/<id>-live.toml`; the runbook's "The map's sequences". The sequence's
  0 is set by writing the tick's, the redraw's and the mouse poll's next times (*provisional*:
  without it two runs' timelines differ by the load's real time); the counts in the timeline are
  from 0 for the same reason. The viewer's frame at 0, drawn from the live part read then, is
  compared with the original's in every run.
- **Recorded:** `idle-cycles` (its own situation `seq-idle-cycles`: four quadrants copied so
  that lava, water and the three cycling rivers share one view, found by the viewer's frames one
  step apart) and `idle-objects` (`objects-flat`), 8 000 ms each, twice: the same timelines, the
  frame at 0 exact each time. The screen changes only at redraws, every one but the redraw after
  a step fell on a redraw's millisecond (no palette step since): five of 44 in 8 s.
- **No oddity of the original found** (no `MCL-Q`). A run of both sequences takes 20 minutes.
