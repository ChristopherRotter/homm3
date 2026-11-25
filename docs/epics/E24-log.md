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

### M2 T1a2 — the scroll (2026-10-26)

- **Read** (`E24-design.md` §3.4; map entries `adventure_key_down` 0x408C40,
  `adventure_mouse_move` 0x40E360 (H3API's SimulateMouseOver, read in part), `view_scroll`
  0x4195C0, `edge_scroll` 0x419820, `input_key_message` 0x4EC0E0, `window_procedure` 0x4F7C00
  (in part), `settings_check` 0x50B260 (in part), `mouse_position_now` 0x50D500 (H3API's
  GetCursorPosition), `scroll_steps` 0x63A66C, `settings_names` 0x63FF74, `scroll_time`
  0x691674, `app_active` 0x6783D0, the three speed options 0x698758, 0x69875C, 0x69878C;
  `adventure_hero_stop` 0x417680 read (H3API's DemobilizeHero), `adventure_manager_procedure`'s
  and `settings_read`'s entries extended). The direction keys scroll only with Ctrl; without it
  they walk the selected hero (T1b). A step is 1, 2 or 3 tiles by the scroll speed and redraws
  at once; the edge steps every 70 ms from the last step (or from the mouse's arrival), with the
  edge cursor; the view's corner stays within −9 to width − 10 and −8 to height − 9, and a step
  that moves nothing turns an edge cursor back to the arrow. The speeds live in the registry
  ("Window Scroll Speed" 0 to 2, "Walk Speed" 1 to 4) and in three globals. VCMI compared (§3.3):
  smooth pixel scrolling, a 15-pixel edge, no key scroll.
- **Built** (the harness, additive): `sequences.toml`'s options (`scroll_speed`, `hero_speed`,
  written into memory once the situation is staged) and `inputs` (`keydown`, `keyup` with
  xdotool's names, `pointer`), given after the step to their millisecond and settled by 500 more
  clock reads; the scroll's last time written at 0 with the other timers; the display's key
  repeat off (`xset`, added to `setup.sh`); the cursor in the timeline (`<ms> cursor <type>
  <frame> <x> <y>`, `Mem.cursor`), so the idle pair was recorded again (its other lines as
  T1a1's). The runbook's "Options and inputs".
- **Recorded** (situation `seq-scroll`): `scroll-0`, `scroll-1`, `scroll-2` (each speed: Ctrl
  and the eight keys, Ctrl and Right held, the left edge, the top-right corner) and
  `scroll-limits` (the fast speed into three corners, the limits, Ctrl and Up at the top), each
  twice the same; the idle pair again, twice; every frame at 0 exact. Everything as read: a
  key's step at its press, one step for a held key, the edge's first step 70 ms after the
  pointer's arrival, the corner's 70 ms grid kept across a move between edges (the west cursor
  shown in the corner until the next step), the arrow at a limit, the cursor's point at the
  33-ms polls.
- **MCL-Q6** (open): the edge zones are 16, 14, 16 and 15 pixels wide (the x rounding of E17's
  drawing bug narrows the east one); recommended: even zones in the client, the original's tests
  for the comparisons. A run of the six sequences takes about 40 minutes.

### M2 T1b — the walk (2026-10-26)

- **Read** (`E24-design.md` §3.5; map entries `hero_move_loop` 0x407B80 and `hero_move_step`
  0x4805E0 extended, `view_draw_here` 0x40F870, `mobilize_hero` 0x4175E0 (H3API's
  MobilizeHero), `hero_select` 0x417B20 (in part), `ambient_sounds_at` 0x4183D0 (inferred),
  `path_arrow_at` 0x41B010, `def_draw_group_blend` 0x47C120 (inferred), `hero_walk_end`
  0x47F7D0, the lifted hero's passes 0x47F860, 0x47FB40, 0x47FC80 (now read),
  `hero_walk_animation` 0x480380, `hero_step_allowed` 0x481BE0 (inferred), `music_track_play`
  0x59A1E0, the walk's tables 0x63D6C8, 0x63D6DC, 0x63D6F0, `terrain_music_tracks` 0x678330,
  `step_sound_playing` 0x6968E0, `step_sound_next` 0x6968E4; the speed options' and
  `view_shown`'s entries extended; the manager's +0xF4 corrected and +0x1EC to +0x20C, +0x360,
  +0x388, +0x390 in `structs.toml`). A human player's selected hero is lifted off its tile and
  drawn at the view's fixed centre (passes 5, 7 and 9, the last a blend); a step moves its tile
  at once, centres the view on the new tile and slides the view's pixel offset from 32 x (dx, dy)
  to 0 in 4, 3 or 2 frames of 50 ms (hero speeds 1 to 3; 8, 10-12-10, 16 pixels), one draw
  without a wait at speed 4 (the computer's speeds 2 to 5 the same tables, 5 drawing nothing);
  the walking groups 5 to 9, the frame stepped by pass 9 at each frame and carried from step to
  step; no wait between steps; the next tile's arrow zeroed at the step's start; the objects
  stand still during a walk (no idle step), the palettes step on; the standing group at the end,
  shown by the next idle redraw. §1.2's "no pixel scrolling" corrected (the walk writes the
  offset). The step sounds' calls and the terrain music's for T5. VCMI compared (§3.3): the same
  times a tile, the same centring, but continuous; objects animated during a walk.
- **Built** (the harness, additive): `frames`' option `points` (the selected hero's movement
  points, `Mem.write_points`) and input `click` (handled, 500 clock reads, before the next input:
  a walk at speed 4 ends inside it and reads the pointer); the timeline's walk lines `hero`,
  `walk`, `arrows`, only where they change (`Mem.selected`, `Mem.walk`), so the earlier
  sequences' timelines stay as recorded (`scroll-limits` run again: ok). The runbook's "The
  walk". The objects' animation phases written 0 once a sequence's situation is staged
  (`Mem.write_phases`) and their frame steps 0 at its 0 (`Mem.write_frame_steps`): two rounds of
  second runs differed only in the screen's changed boxes at idle redraws, which moved with the
  two mines' random phases and then with the steps' count since the load. T1a's sequences run
  again with both: `idle-objects`, `idle-cycles`, `scroll-0` to `scroll-2` as recorded;
  `scroll-limits` differed in one box (a hero's flag at the idle redraw at 720, one row), so it
  was recorded again, twice the same. Three VM restarts cut runs (resumed).
- **Recorded** (situations `seq-walk`, `seq-walk-follow`, `seq-sail`): `walk-1` to `walk-4` (each
  hero speed: the eight direction keys without Ctrl, a four-tile path clicked twice),
  `walk-follow` (the view not centred, an eight-tile walk, a scroll and a step), `walk-points` (a
  walk the points end, the path's rest kept), `walk-sail` (a boat on open water); each twice
  the same, every frame at 0 exact. Everything as read. No oddity of the original found (no
  `MCL-Q`). A run of the seven sequences takes about 35 minutes.
- **Decided with the user** at the start: MCL-Q6 answered (a), the client's edge zones 16 pixels
  on every side; T1b done whole (not cut); **key bindings**: T4's window takes every key through
  a table of actions (the original's keys its defaults, a user file overriding them), M3 T0 moves
  it into the common crate with the battle viewer's keys (the client lane's inbox), a rebinding
  screen with the options dialog later.

### M2 T2 — the map over time, headless (2026-10-27)

Taken whole (the user, at the session's start: offered a cut before the frame check, chose T2
whole).

- **Built** (`E24-design.md` §3.6): `h3_map_view::timeline` (*provisional*; pure: the original's
  loop a millisecond at a time from a sequence's start and the player's actions: the palette
  ticks, the idle redraws and the objects' steps, the scroll by Ctrl and the direction keys and at
  the frame's edges with the speeds' steps, the 70 ms grid and the limits, the edge cursors and
  the arrow, the mouse poll's point; `Action` (the window's actions T4's key table will bind),
  `Unplayable` (a walk, T3's; the pointer over the view, M3's), `lines` (the timeline's lines),
  `changed` (the `screen` boxes)); `h3_map_view::sequence` (`sequences.toml` read, the inputs as
  actions by the original's default keys); `h3_map_view::cursor` (`h3-view`'s copied,
  *provisional*, an origin comment); the tables' `scroll_steps`, `scroll_ms`, `poll_ms`; the
  layout's `[scroll]` (`middle`, `edge`: MCL-Q6's 16 pixels, the original's zones under
  `Bugs::Original`) and `[cursor]` (`cradvntr.def`, the arrow, the edge cursors, 43 hot spots read
  from `cursor_hot_spots` 0x67FF50); the gate checks the cursor's 43 frames. The binary's
  `sequences [<id>] [--frames <dir>] [--live <live.toml>]` (every line against the recorded
  timeline, the `screen` boxes from its own frames; a walk listed as T3's; `--bugs` original by
  default). The harness (additive): `frames`' frame check (`frames.txt` against
  `timelines/<id>-frames.txt`, `diff/`), `--reuse`, `drawn` over the sequences
  (`timelines/drawn.txt`, `STALE`); the runbook's "The frame check".
- **Checked:** T1a's six sequences recorded again (this sandbox keeps their screens; each
  timeline as recorded, the frame at 0 exact; 40 minutes): every line of every timeline as the
  viewer's, and all 181 kept screens exact outside the panels (`timelines/<id>-frames.txt`, all
  0); `drawn` draws the 40 situations the same and the six sequences' 181 frames
  (`timelines/drawn.txt`). The unit tests replay the six timelines' state lines without the
  original files. The seven walk sequences are listed as T3's. The run's live parts at 0 differ
  from the last ones in their palette steps (the load's real time), kept as the runs gave them.
  Not seen in a frame: the order of a scroll and a redraw due in the same millisecond (no
  animated object in either view then, design §3.6).
- **No oddity of the original found** (no `MCL-Q`).

### M2 T3a — the walk over time, headless (2026-10-27)

T3 cut at its seam before it started (the user, at the session's start): T3a the walk's timing,
every line but the `screen` ones and no recording; T3b the walk drawn, its screens and the frame
check, the walks recorded again first.

- **Built** (`E24-design.md` §3.7): the walk in `h3_map_view::timeline` (*provisional*): a
  direction key's step and the path shown by a click and walked by a second one, the selected
  hero lifted (back on its tile after a scroll that moves the view), the view centred by each
  step and slid by the hero speed's pixels in frames of 50 ms (speed 4 one draw), the walking
  group and the frame carried from step to step, the path's arrows zeroed step by step, the
  path's green arrows as the steps, the standing group at the walk's end and the objects' catch-up
  at the next idle redraw, the cursor hidden during a walk and read again at its end, the actions
  given meanwhile waiting (a key's) or drained (the path's: a key or a click ends it), the
  `hero`, `walk` and `arrows` lines; `Unplayable` without `Walk` (`Panel`, `NoPath` added),
  `Start`'s hero speed, hero and paths, `Draw` with the walk's part (offset, hero, lifted picture,
  arrows) and `Cursor`'s `shown` for T3b; `timeline::chain` (a path's arrows in its order);
  `sequence`'s `hero_speed` (2 by default), `paths` (the recorded `arrows` after each click),
  `selected` (the live part's hero of the viewer). The tables' `[map.walk]` (groups, frames,
  frame time, pixels per speed; checked). The binary's `sequences` plays the walks and compares
  their lines but the `screen` ones (`--frames` on a walk refused: T3b's). `sequences.toml`:
  `walk-follow`'s scroll speed 0 given (T1b's original held 0). The harness and the runbooks say
  T3b's for a walk's frame check.
- **Read** (`docs/exe/map.toml`): `wait_until` runs the tick at each turn of its wait (the poll,
  the palettes); a key's step with a path shown (cleared, drawn, the objects stepped once if due);
  the cursor hidden and shown around a walk and the mouse read again at its end; the animation's
  start, frames and the pass 9 frame's mod 8.
- **Checked:** the seven walk timelines every line but the `screen` ones (`h3-map-view
  sequences`: 13 played, 13 as recorded; the unit tests replay all 13 without the original files);
  `drawn` the same (40 situations, the six sequences' 181 frames). No recording this time.
- **Not shown by a recording** (*provisional*, design §3.7): the actions queued during a walk; a
  key at the map's side; the arrow over the view at `walk-4`'s end (M3's cursor by the tile).
- **No oddity of the original found** (no `MCL-Q`).

### M2 T3b — the walk drawn (2026-10-27)

Taken whole (the user, at the session's start, over a cut at the frame check or between land and
boat).

- **Read** (`docs/exe/map.toml`; design §3.8): `draw_frame_blend` 0x47D4F0 (encoding 3's literal
  pixels blended half and half with the screen, runs skipped with no colour; both callers pass
  none) and `def_draw_group_blend` 0x47C120; pass 9's cut at the map's sides exactly (the offset
  on the left and the top only); passes 5 and 7's lifted cells (after the objects' shadows; row
  8's after layer 1, row 7's after layer 2; after all heroes and boats on a tile with an empty
  list; none off the map); the idle redraw's one step a redraw (0x408B26), so a late one redraws
  again in the same millisecond.
- **Built** (*provisional*): `Live::walking` (`Walking`, `Lifted`: the offset and the lifted
  hero by index, group and frame); the scene's offset on every tile pass, the lifted hero off its
  tile and at the view's middle (passes 5 and 7's cells, pass 9 whole: `Pass::Blend`,
  `draw::blend`), the flags' frame the objects' step plus the hero's mod 8, a boat's flag in
  pass 9 by the hero's owner; `timeline`'s idle redraw drawing again for each step it is late
  (`step_objects` one step; the test renamed `a_late_redraw_draws_again_for_each_step_missed`);
  the gate's walking groups (`WalkArt::drawn_groups` a range, `DefNeed::walking` and
  `walk_frames`, `check::walk_problems`: the standing groups hold 1 frame, the flags 8, every
  walking group 8); `sequence::selected_index`; `h3-map-view sequences` draws the walks (its
  `Verdict::Walk` and the `--frames` refusal gone); `h3map.py`'s `walks` gone (the frame check and
  `drawn` over the walks). The client lane's inbox: 0x47D4F0 read for its `Drawer::Translucent`.
- **Recorded** (a fresh sandbox, about 35 minutes): T1b's seven walks again, each timeline as
  recorded and its frame at 0 exact; their live parts kept (`walk-1`, `-2`, `-3`, `-follow`
  changed, as every run's).
- **Checked:** all 13 sequences every line, the `screen` boxes included (13 played, 13 as
  recorded); the walks' 269 frames against the original's, all exact (`timelines/<id>-frames.txt`
  new); `drawn`: the 40 situations the same, the 13 sequences' 450 frames recorded. The first draw
  missed 8 frames, all at walks' ends: the lifted hero's flag a frame behind (the late redraw).
- **Not shown by a recording** (*provisional*): a step at the map's side (MCL-Q7).
- **MCL-Q7** (the lifted hero cut at the view's tiles during a step at the map's side): built as
  the original's in both profiles meanwhile.

### M2 T4b1 — the map's live input (2026-10-27)

T4b cut at its seam before it started (the user, at the session's start, over cutting the mouse
off or taking it whole): T4b1 the live input in the sandbox, T4b2 the Windows build tried on the
user's machine. No new reading (`input_key_message`'s scan code, T1a2's); design §3.10.

- **Built** (*provisional*): `Replay::give` (an action at a millisecond after the change shown;
  each change keeps the loop as of its millisecond, and one given behind the loop runs it again
  from the latest change before it, from 0 when none is kept), `Replay::actions`; `Start::live`
  (the pointer over the view played, the arrow outside a zone; a click off the view or on a tile
  no path leads to only the pointer's move: `Loop::refused`) and `Start::path` (the path shown at
  0, chained from the hero, clickable); the key table's keypad keys and `cursor` (F9); `h3-sdl`
  additively: `Key::Keypad1` to `Keypad9` but 5, `Window::set_key_repeats`; the binary's
  `paint` (a situation's hero the viewer's only one, its kept path; `Painter::cursor_image`; the
  last two views kept, the frame shown and the one ahead) and `window` (the input past a
  sequence's inputs: the keys through the table, repeats to the map only, the pointer's moves one
  a batch, the left button's presses, the mouse given where it lies when the map goes live; the
  hardware cursor set at each change, or drawn while on the frame; `--drawn-cursor`; the next
  frame kept when an input leaves it; `--check`'s headless check of every change shown; the
  given actions by kind, the view at the end and the frames at an input's time in the report).
- **Checked:** the timeline's live start (the pointer over the view, the west zone over it, the
  clicks it cannot play, the path at 0) and the replay's actions given behind the loop (five
  sequences, as known ahead, again after a restart) by unit tests; under Xvfb every run's frames
  read back exact and every change shown the headless loop's (design §3.10's list); `sequences`
  13 of 13, `drawn` 40 situations and 13 sequences the same.
- **Found:** a view composes in 30 to 60 ms in the sandbox (release), so a frame at an input's time
  shows that late and a walk at speed 2 lags; `xdotool click --window` sends a synthetic event
  SDL ignores (the plain `click` works: `TRAPS.md`); the sandbox's baseline started by
  `sandbox-setup.sh` stopped when the session's first turn ended (its log ends at the coverage
  step; the gates ran again by hand).

### M2 T4a — the window replaying, the key table (2026-10-27)

T4 cut at its seam before it started (the user, at the session's start, over taking it whole or
cutting off only the Windows trial): T4a the window replaying and the key table, T4b the map's
live input and the Windows build. No code reading; design §3.9.

- **Built** (*provisional*): `h3_map_view::keys` (the table: `Command`, the scroll's and the
  hero's eight directions and the window's own six; `Chord`, a key by the adapter's name with or
  without Ctrl, matched exactly; `Keys::original`, the original's direction keys and the
  window's F5 pause, F6 a frame on, F7 restart, F8 vsync, F11 fullscreen, Esc quit;
  `Keys::with_file`, a user's TOML file over the defaults command by command, `[]` unbinding, a
  chord bound to two commands refused); `h3_map_view::replay` (`Replay`: the timeline's loop run
  forward as the window asks, `at`, `after` with a look-ahead, `restart`; the changes of the
  view's draw and the cursor); `h3_map_view::clock` (`h3-view`'s copied, an origin comment);
  `Loop::drawn` and `Loop::cursor`; the binary's `paint` (the set-up and the `Painter` moved out of
  `sequences`, shared with the window; a situation's set-up: its writer's or shipped map, its kept
  live part or the start with its `hidden` rectangles, `render::hide` factored out) and `window`
  (`h3-view window`'s loop: the frame of each change composed ahead and shown when due, a change
  overtaken passed over, the replay's cursor drawn in the frame, the system's hidden; the keys
  checked against `h3_sdl::Key`, Control alone refused; `--check`, `--measure`); `h3-sdl`'s `Key`
  grown additively (Left, PageUp, PageDown, End, LeftCtrl, RightCtrl, F1 to F12; `Key::ALL`,
  `Key::from_name`), `h3-map-view` depending on `h3-sdl` for its binary (the purity test's
  `BINARY_ONLY`).
- **Checked:** the replay's changes equal the headless play's for `scroll-limits`, `walk-1` and
  `walk-sail` (a unit test, again after a restart); `sequences` 13 of 13 as recorded after the
  move; `drawn`: the 40 situations and the 13 sequences' frames the same. Under Xvfb (two cores,
  Mesa's OpenGL ES 2, scale 1): `walk-1 --check 20` 153 of 153 frames exact, `objects-flat` and
  `shipped-hoard --check 10` 56 of 56 each, `walk-sail --scale 2 --check 8` 56 of 56 at
  1600 × 1200, none passed over; `scroll-limits --measure 10`: a frame a median 5.3 ms after its
  time, p95 8.4, max 28.5. Driven by `xdotool`: F5 holds the map, F6 shows the next frame while
  held, F7 restarts, Esc exits 0; a user's file (`pause = ["Space"]`, `quit = ["Ctrl+F12"]`)
  pauses by Space, Esc then nothing, Ctrl+F12 quits; an unknown key refused with the list.
- **Not checked here:** fullscreen (bare Xvfb has no window manager; the adapter's, unchanged since
  E17 §8.1); the Windows build (T4b).

### M2 T5 — the map's sound (2026-10-27)

Taken whole (the user, at the session's start, over a cut into the readings headless and the
window, or one by sound kind). The user's computer link dropped after the staging (the MP3
folder's terrain tracks staged with the rest). Design §3.11.

- **Read:** the terrain music's requests (the map's start 0x419990, `hero_select`'s, a step's end;
  the tracks 0x684AE8; `play_music`'s same-name test) and its change (`music_stop_fade`
  0x59B080: the old track's place kept, a fade-out 114 … 12 by 100 ms, then the new one where it
  stopped, fading in); the step sounds (`horse00` to `horse10`, 0x40747A; in a boat the water's,
  flying `horse10`; one loop from the walk's start, changed only where the music's terrain
  changes, at the frame's wrap; a key's step none but a new terrain's at its end); the ambient
  loops (`ambient_sounds_at`'s two passes over the rings to 3 tiles, `ambient_sound_visit`'s
  slots, the volumes 32, 28, 20, 10; `tile_ambient_sound`'s switch by class, a dwelling's
  creature, a mine's type, the shore's water, the volcano, the special grounds; `hero_hide`'s
  restored class); the callers (the scroll none). Corrected: design §3.3's and §3.5's "+0x388 in
  a boat" (the terrain's). MCL-Q8 (the found set's shift mod 32) and MCL-Q9 (the shore's 21
  against the coast's 20) asked, both built as the original's.
- **Built** (*provisional*): the tables' `[map.sound]` and the terrains' `music` and `step`
  (`h3_map_view::sound::SoundArt`, resolved as `SoundRules` into `Tables::sound`, with
  `Tables::sounds` and `tracks` for the gate); `h3_map_view::sound` (`TileSounds`: each tile's
  loop from the art, the live part and every level's draw lists, the tile's recorded object as
  0x505610's walks give it, heroes standing silent; `Sound`: +0x58, the slots, the step sound
  playing and queued, its `Cue`s); `Start::sound`, the timeline's loop driving it (the start, the
  lift, the walk's start, a step's end, the frame's wrap, the walk's end; `Loop::take_cues`); the
  replay's cues kept apart and handed out once (`Replay::take_cues`, again after a rerun or a
  restart); `keys`' `music` (F2) and `effects` (F3); the binary's `sounds` (the archives, the
  WAVs, the MP3s; the gate's check) and `audio` (`Library`, `Player`), the window's `--mute` and
  `--cues`, `paint`'s tiles' sounds; `nanomp3` for the binary.
- **Checked:** the sound's unit tests (the tables' rules and errors, a hand-made map's tiles, the
  slots' scan order, the eviction, the volumes, the stop flag, MCL-Q8's shared bit, the music and
  the step sounds over a walk), the timeline's walk east with its cues (the start, the lift, the
  volumes by distance, the music at once and the step sound at the wrap, the key steps after a
  scroll), the replay's cues across a rerun from 0 and a restart; the gate: the 81 WAVs and the 9
  tracks found and decoded; `sequences` 13 of 13 as recorded; under Xvfb with SDL's `disk` driver
  the fade-in and the fade-out sample-exact against the tracks (design §3.11), every frame read
  back exact and every change the headless loop's; the client gates at 100 %.
- **Not checked here:** the original's cues (the harness's settings have no sound; Miles under
  Wine without a device untried); `drawn` (no drawing changed; the original's screens are not in
  this sandbox); a hero flying (none in M2's situations).

### M2 T4b2 — the Windows build tried on the user's machine (2026-10-27)

The user at the machine (the link to the user's computer). No code changed; design §3.10 ("On
the user's machine"), §3.4 (the input ring's bound).

- **Built:** `cargo build --release -p h3-map-view --target x86_64-pc-windows-gnu` in the sandbox
  (`rustup target add`, apt's `mingw-w64` after an `apt-get update`; 3 min 21 s from scratch,
  SDL included), `h3-map-view.exe` 9.0 MB importing system DLLs only; put into the user's
  `clients/out/win/` with `map-measure.cmd` (five runs: `walk-1` `--check 20`, `--measure 20`
  with vsync off and on; `path-objects` `--check 30` and `--measure 30` while the user uses the
  keys and the mouse; each run's output and exit code into `map-measure.txt`) and `map-play.cmd`
  (a sequence or a situation, `path-objects` by default). Both name `--map map` and `--install
  ..\original-data\install` (the exe's defaults are where it was built: the review list).
- **Measured** (design §3.10): every frame read back exact (203, 345) and every change shown the
  headless loop's; a quiet walk's frames 0.4 ms after their time (p95 0.6); `path-objects` with
  input p95 99 ms late, 87 to 109 frames passed over in 30 s, an input's frame a median 13.5 ms
  (p95 66). Callgrind (sandbox, `render` of `path-objects`): `compose` 775 million instructions
  of 1514 (the PNG's deflate 508, the archives' inflate 115), `draw::flagged` 392 and
  `draw::shadow_pass` 326 of them: every pixel of every frame visited through a closure, its
  place found by a division and a modulo, each written through `Screen::set`'s checks.
- **The user's findings:** the sound's transitions not clean, little fading (built as the
  original's: the music's fade 900 ms by tenths, the loops and step sounds cut and their
  volumes set at once; compared by ear on a whole map, M4, the user); the hero stepping through
  obstacles and a click showing no new path (the engine's, M4 and M5: the epic's known gaps);
  a held key walking the hero on well after its release (the original queues every repeat:
  **MCL-Q10**, kept, the user; Space cancels it in the original, not read); the frames lagging
  among many objects (**T4b3**, the user: "the client should run on really old hardware too,
  and given the simple 2D graphics we have absolutely no business having performance issues").
- **Read:** `input_key_message`'s ring (0x4EC1D9: a write meeting the read index drops the
  oldest; 63 messages kept); `wait_until` dispatches the window's messages during a step's
  frames (its map entry), so the repeats queue during a step.
- **Found on the way:** a `git fetch` through the link left `.git/objects/maintenance.lock`
  again (moved aside: `.git/stale-maintenance-lock-2026-10-27b`); apt's `mingw-w64` needs an
  `apt-get update` first in a fresh sandbox.

### M2 T4b3 — the view drawn fast (2026-10-27)

Queued and taken in the same session as T4b2 (the user: "the client should run on really old
hardware too, and given the simple 2D graphics we have absolutely no business having
performance issues"); taken whole. Design §3.12.

- **Built** (*provisional*): `h3_map_view::draw`'s pixels by rows (each row's columns from the
  canvas, the clip and the screen once; the screen's row walked forwards or mirrored backwards;
  the closure per pixel inlined; `runs_of` pads a frame whose runs end early); `compose` drawing
  the frame picture from its rows in the viewer's colours (no copy), a DEF's cycled palette by
  its name as spelled, the flag palette's colours once a scene; `Batch`'s lookups without an
  upper-cased copy for names already upper-case; `h3-render`'s `Screen::pixels_mut` (additive);
  the binary's `Painter::preload` (every asset a view of the map draws at 0, view by view on each
  level) and the window's "a frame made in" (view, cursor, widening; SDL's clock).
- **Checked:** every one of `drawn`'s 490 PNGs (40 situations, the 13 sequences' 450 frames: the
  whole frame, panels included) byte for byte as before the change, `drawn` 0 not the same,
  `sequences` 13 of 13 as recorded; the window under Xvfb (`path-objects` with Ctrl and held
  direction keys, `walk-1`): every frame read back exact, every change the headless loop's;
  `clients/scripts/check.sh` (fmt, clippy, tests, deny, coverage 100 %, the CI paths).
- **Measured:** callgrind, `render` of `path-objects`: the composition 775 → 24.6 million
  instructions. The user's machine (`map-measure.cmd` with the new exe; T4b2's kept as
  `map-measure-t4b2.txt`): a frame made in a median 4.0 to 5.8 ms; `path-objects` with input p95
  8.2 ms after its time (99), none passed over (87); the sandbox 5.2 to 5.7 ms.
- **The user's findings:** the frame's cost "will do for now", to be checked again (the inbox);
  no hooves heard: the user's runs walked by the direction keys only (none clicked a path's end),
  whose steps play none in the original (§3.11: only `hero_move_loop` and a step's terrain change
  read the step sounds); the sandbox's mix has `walk-1`'s and a clicked walk's beats; "a few
  terrain-specific samples which the original doesn't seem to play": `HORSE20` to `HORSE27` and
  `HORSE29` in `Heroes3.snd`, never loaded (**MCL-Q11**; the 20 files in the user's
  `clients/out/win/horses/`).

### M2 T6a — the reviewers and their fixes (2026-10-27)

On `map-client/e24-m2-t6a`, the user present. T6 cut by the user before it started (T6a the
reviewers and their fixes, T6b the closure); MCL-Q11 answered at the start (the original's sounds
now, the unused set an option later; the direction keys' steps kept silent); the music player's
findings cut out as T6a2 by the user once they were checked.

- **Starting point:** `master` at #512 (CI green); `clients/scripts/check.sh` in the sandbox, all
  gates passed (code 8764120, the baseline).
- **The reviewers:** three fresh read-only sub-agents in parallel, one per area (R1 the map over
  time and its drawing; R2 the window, the input and the sound; R3 the harness and the
  recordings), each against the design's readings and the epic's review list; 8, 11 and 10
  findings, each checked against the code and, where it named the original, the executable
  (`scripts/exe.py`: `hero_move_loop` 0x407E87, 0x407F80 to 0x4080CE; `input_flush` 0x4EC570 (the
  client lane's name, E17 M5 T2a, merged meanwhile); `edge_scroll` 0x419978; `play_music` 0x59AD0D to 0x59ADA6; the settings'
  names at 0x63FF74, 0x698774 `option_show_route`). R1 also compared T4b3's row drawers with the
  slow ones in 200 000 random cases (clips, sides, mirrors, offsets): no pixel differs.
- **Fixed, the library** (each new test fails on the old code): R1-1, a scroll's key-down (Ctrl
  and a direction key) during a path's walk was dropped: it ends the walk as a step's and a
  click's do (kinds 1, 8, 0x20, 0x200 at 0x407FA7); R1-2, the walk dropped every message given
  during a step: it reads up to the first that ends it, those behind it wait for the loop's turn
  (no flush on the way out); R1-3, a message taken after a walk set the pointer back to its own
  point, which the edge scroll then read: the mouse's point now (GetCursorPos), a click's tile its
  own; R1-4, a walk a message ended drew the path's redraw with the walking pose: `hero_walk_end`
  first (0x408002, then `make_hero_path` 0x4080C9); R1-5, a path's walk empties the input queue
  at its start (`input_flush` at 0x407E87): the messages queued behind its click during a
  key's step are gone; R1-6, the stop's test checked no drawn pose and locked in R1-2 (rewritten,
  four tests added); R1-7, a speed out of range is the original's default (scroll 1, hero 2,
  `settings_check`), not a tile and an instant walk; R1-8, a situation's view far off the map
  overflowed the tile under a click: `Situation::parse` refuses a corner more than the map's side
  off it, `tile_at` saturates. Design §3.7's paragraph on actions during a walk corrected.
- **Fixed, the window and its sound:** R2-1, a track whose frames do not decode (an empty decode)
  hung the window in the music's fade: refused by `sounds::mp3`, a fade with nothing left ends;
  R2-6, a track asked for with none playing was decoded on the frame loop's thread: it starts once
  its thread has decoded it; R2-7, the view scrolled at the edges without the focus (the original
  only while `app_active`): `h3-sdl`'s `Window::focused()` (additive; true until SDL says the
  focus left) and the pointer given off the frame without it; R2-8, fullscreen's bars lay off the
  frame, where no edge scrolls: a point off the frame is the frame's nearest; R2-9, a hidden
  cursor's moves presented the same frame again: one hidden cursor; R2-10, F7 in a situation left
  the pointer parked: the mouse's point given anew.
- **Fixed, the harness** (not run: no game staged; T6b's recordings run it; the 13 sequences'
  inputs pass the new check): R3-1, `frames` ignored the viewer's own timeline verdict; R3-2,
  `drawn --record` kept the digests of a viewer's run not as recorded or failed partway; R3-3, a
  run that failed midway left the last run's states and live part beside its screens and
  `--reuse` took the mix: cleared first, `reused` needs the live part and its situation's digest;
  R3-4, a failing clean-up left Wine running and a killed run's keys stayed down: nested, the
  keys released at a run's start; R3-5, a live part from a run not exact at 0 became `drawn`'s
  reference; R3-6, a step that skipped a millisecond lost its inputs unseen; R3-7, a key name
  xdotool does not know was ignored with a warning: the eight direction keys checked; R3-8,
  `h3mem`'s docstrings denied M2's writes; R3-9, the runbook's cursor-free frames have the speed-4
  exception (`walk-4` at 2700); R3-10, the scroll's time written at 0 had no effect (the parked
  pointer sets it at every call, 0x419978): no longer written, the runbook says why.
- **Cut out as T6a2** (the user), the music player against the original's music thread, each
  checked in the code: R2-2, a track asked for during a fade-in cuts it and fades out from 114
  at once, where the original's `music_thread` holds the manager's critical section (+0xA8) from
  0x59A859 through the fade-in (0x59AB42 to 0x59ABB1, to 0x59ABE7): the request waits for the
  fade-in's end and `music_stop_fade` fades from `AIL_stream_volume`, 127 then (0x59B09E,
  0x59B1E2); R2-3, `end_fade` ends a fade-out once its 900 ms are queued and the next track
  decoded, so a cue within the next 900 ms takes back the old track's tail as the new one's
  (the new track's `next` wraps, its place saved near its end): take back at most what the new
  track queued, and a cue while the old fade is still queued is one during the fade; R2-4,
  `play_music` drops a request whose name is the pending one (0x6A3394, 0x59AD0D) or the playing
  stream's while `AIL_stream_status` is 4 (0x6A3290, 0x59AD43 to 0x59ADA6; the thread copies the
  pending name at 0x59A96A): the client filters only by the last track asked for (`Sound::asked`),
  so grass, dirt, grass 150 ms apart resumes Grass where the original keeps Dirt; R2-5, the old
  track's place is saved after its fade, the original's before it (`AIL_stream_position` at
  0x59B146 to 0x59B15A, stored 0x59B19B to 0x59B1A8; the fade from 0x59B1E2): it resumes where
  its fade began; R2-11, the music queue's bookkeeping has no test (a pure model driven by a fake
  stream's queued count, the reviewer's suggestion). The battle client's music (`h3_view::sound`,
  E17 §8.2) plays through the same `play_music`.
- **Checked:** `h3-map-view`'s tests (the 13 recorded timelines line by line among them), clippy
  and fmt over the client workspace, the lane's scoped verify; `scripts/exe.py check` and
  `cited` (every cited address covered). Not yet: the window's R2-7, R2-8 and R2-10 by hand (T6b,
  on the user's machine: Alt+Tab with the pointer at an edge, fullscreen's bars, F7 in a
  situation).

### M2 T6a2 — the music player as the original's music thread (2026-10-27)

On `map-client/e24-m2-t6a2`, the user present; taken whole (it fit a session, no seam asked).

- **Starting point:** `master` at #517 (CI green); `clients/scripts/check.sh` in the sandbox, all
  gates passed.
- **Read** (design §3.11, "The music's thread"; the map's entries): `play_music` 0x59ACB0 whole
  (its filters, the restart of a name whose stream stopped, a thread a request), `music_thread`
  0x59A840 whole (the lock +0xA8 from 0x59A859 to 0x59ABE7, the pending name taken after the
  fade, the open, the place, the fade-in), `music_stop_fade` 0x59B080 whole (the place before the
  fade, the fade from the stream's volume, the pause); `music_resume` 0x59AC00, `window_procedure`'s
  activation case and `sound_focus_lost` 0x599C40 in part (the inbox). Each of the reviewer's five
  confirmed: R2-2 (a request during a fade-in waits for its end, the fade-out from the whole),
  R2-3 (the take-back), R2-4 (the filters; an oddity: **MCL-Q12**), R2-5 (the place before the
  fade), R2-11 (no test).
- **Built:** `h3_map_view::music` (pure, 100 %): `Music`, the original's state on a clock of the
  frames heard (`request` with its filters, `advance` giving runs of one volume, the thread's
  steps between: the lock through each fade, the pending name taken at the fade-out's end, the
  place kept where the fade-out begins and taken back at the open), `Queue`, the model at the
  frames heard and a copy at the frames queued (a request taken: the stream cleared and queued
  again from the frames heard; the window's clear the same). The window's music (`audio`) on it:
  one stream of 44.1 kHz stereo (another format refused), the runs' samples scaled, a track
  being decoded holding the music instead of the frame loop waiting for it.
- **Checked:** 15 unit tests of the model (each finding's case, the pending and playing filters,
  grass-dirt-grass, a missing, an empty and a slow track, a place past a track's end) and the
  queue against a fake stream played in uneven turns (what it plays is what the model alone
  hears); the window under Xvfb with SDL's `disk` driver (design §3.11, "Checked"): back onto
  grass 326 ms after the dirt kept Dirt; back during Dirt's fade-in: its fade-in ended, its
  fade-out from 114, Grass where its fade began. `h3-map-view`'s tests, clippy, fmt; `exe.py
  check` and `check-exe-map`.
- **Noted:** the client lane's inbox (the battle's end tune follows the combat music's fade-out;
  the model a candidate for M3 T0's common crate); this lane's inbox (the sound without the
  focus).

### M2 T6b1 — the recordings again (2026-10-27)

On `map-client/e24-m2-t6b1` and `map-client/e24-r2-8`, the user present; T6b cut at its seam
before it started (the user): T6b1 the sandbox's part, T6b2 the closure with the user.

- **Starting point:** `master` at #521 (CI green, #520 merged); `clients/scripts/check.sh` in
  the sandbox, all gates passed; `h3map.py drawn` all the same (40 situations, 13 sequences,
  450 frames).
- **Recorded** (the runbook's "everything" at a milestone's closure; T6a's harness changes,
  R3-1 to R3-10, run for the first time): `frames`, the 13 sequences, each `ok` line by line
  (127, 132, 82, 82, 84, 89, 189, 171, 151, 108, 117, 56 and 66 lines), its frame at 0 exact and
  its frame check `ok`, all 450 frames 0 differing; about an hour. `set`, the 40 stills, each
  0 differing as recorded; one game's Wine exited before the menu and was started again (the
  runbook's known case). The kept live parts changed with the runs' load times (the palette and
  frame steps, the traps): `drawn` then 31 `CHANGED`, none `STALE`, `new` or refused; `drawn
  --record`, then `drawn` all the same.
- **R2-8** (E24 M1's reviewer; #523, `cross-lane`): `h3_map::writer`'s edge bound one tighter,
  `side + 6` columns and `side + 4` rows (7 and 5 past the last column and row, as the doc
  comment and `h3m.md` said); its tests moved with it; no situation anchors an object past it
  (the furthest (40, 40) on a 36 map). The adventure lane's inbox notes it.
- **No new question.** M2's exit holds in the sandbox: every recorded sequence as recorded, line
  by line and frame by frame; T6b2 tries the window and its sound on the user's machine.

### M2 T6b2a — the review list and the questions (2026-10-27)

On `map-client/e24-m2-t6b2a`, the user present in the chat but away from the machine soon: T6b2
cut at its seam by the user (T6b2a now, the hand test later); the answers' build then queued
between them as T6b2b. No code changed.

- **Starting point:** `master` at #527 (#523 and #527 merged; CI green on #527's code). The
  original data staged again from the user's machine (the LODs, the sounds, the nine terrain
  tracks, the maps, the text tables, the harness's DLLs).
- **The build:** `h3-map-view.exe` of `master` at #527 cross-built (`x86_64-pc-windows-gnu`,
  4 min 12 s from scratch, 9.1 MB) into the user's `clients/out/win/` (T4b3's kept as
  `h3-map-view-t4b3.exe`) with `map-try.cmd`: `path-objects` (Alt+Tab with the pointer at an
  edge, fullscreen's bars, F7 resting at an edge) and `seq-walk` with `--cues` (grass to dirt,
  back after a while, back within a second), its output in `map-try.txt` and `map-try-cues.txt`,
  the user's findings in `map-try-notes.txt` (a template).
- **The review list** (M2's six items, summarised in the chat) **accepted** by the user.
- **The questions:** MCL-Q7 and MCL-Q8 answered (a), the client's fix with the original's
  behind `--bugs original`; MCL-Q12 answered (a), the request kept; MCL-Q9 left open (the user
  decides later). The answered entries moved to the lane's `QUESTIONS-ANSWERED.md`; T6b2b builds
  them.
- **M3 T0 queued** (behind T6b2c); the common crate proposed in the client lane's inbox, its
  answer asked before M3 T0.

### M2 T6b2b — the answers built: MCL-Q7, Q8 and Q12 (2026-10-28)

On `map-client/e24-m2-t6b2b`, the sandbox's work (the user present in the chat). Each answer the
client's by default and the original's behind `--bugs original` (`h3_render::Bugs`, as MCL-Q2 to
Q6): `sequences` and `drawn` compare with `--bugs original` as before, the window and `render`
default to the fixes.

- **Starting point:** `master` at #528 (#531 merged; CI green on #530's code, #531 and #528
  docs only); `clients/scripts/check.sh` in the sandbox, all gates passed, coverage 100 %.
- **MCL-Q7** (`scene`, design §3.8): the map's rectangle on the screen (`map_shown`: the map's
  tiles' places moved by the walk's offset); with the fixes passes 5 and 7 take every cell of
  the lifted hero's picture, on the map or not, each cut to that rectangle, and pass 9 its
  picture cut to it; with the original's bugs the cells of the tiles on the map whole and pass
  9's own cut (rewritten as a rectangle: the moved side on the left and the top past a side, the
  tiles' on the right and the bottom past one). At offset 0 both the same. Tested on five steps
  at the sides in both profiles (east from column 0, east onto the last column, west onto the
  first, south from row 0, north onto it), the cells' and pass 9's clips.
- **MCL-Q8** (`sound`): the found set in 128 bits, a bit for each sound with the fixes, the
  original's `1 << (s mod 32)` with its bugs (a sound past 127 has none: the loops are 70). The
  test's kept loop 1 takes its distance 1's volume (28) with the fixes, keeps its earlier one
  with the original's (its XOR by 33).
- **MCL-Q12** (`music`): with the fixes the track opened last asked for again while another is
  pending is not dropped: during its fade-out it becomes the pending one (`Request::Taken`: it
  comes back where its fade began, after the fade; grass, dirt and grass 150 ms apart now end on
  Grass, Dirt never heard); during its fade-in or its opening the pending one is dropped
  (`Request::Kept`, *provisional*: the same slip a fade-in later, which the answer's wording did
  not name; dropping keeps it playing without a second fade, where taking it would fade it out
  and in again from the same place; the thread waiting finds no pending name, 0x59A883).
  `Request::changed` tells the queue and the window to queue the stream again (both kinds). The
  window's `Player::open` takes `--bugs`; `Sound::new`, `Music::new` and `Queue::new` take it.
- **Each new test fails on the old code** (the fixes made inert: the seven tests of the three
  answers failed, and MCL-Q2's, whose fixed branch the substitution also disabled).
- **Checked:** `clients/scripts/check.sh` all gates passed, coverage 100 % (regions, functions,
  lines); `h3map.py drawn` all the same (the 40 situations; the 13 sequences' 450 frames and
  every timeline line by line as recorded); `window seq-walk --check 4` under Xvfb in both
  profiles (every frame read back exact, every change shown the headless loop's). The music's
  fix by ear is T6b2c's hand test (the window's part: the profile handed to the player, the
  stream queued again on `Kept` as on `Taken`).
- **For T6b2c:** `h3-map-view.exe` of this branch cross-built into the user's `clients/out/win/`
  (T6b2a's kept as `h3-map-view-t6b2a.exe`); `map-try.cmd`'s item 2c and the notes' line now
  say Grass comes back (the notes were still blank).

### M2 T6b2c — the hand test and M2 archived (2026-10-28)

On `map-client/e24-m2-t6b2c` (docs only, on T6b2b's branch, #532), the user at the machine.

- **The hand test** (`clients/out/win/map-try.cmd`, the `h3-map-view.exe` of T6b2b's branch): a
  first run stopped by Ctrl+C after run 1 (the console's Ctrl+C ends the script, and the window
  without its report; Esc closes a window), then run again whole. Run 1, `path-objects` (Alt+Tab
  with the pointer at an edge, fullscreen's bars, F7 at an edge): the user, "everything in
  map-try worked for me". Run 2, `seq-walk`: "sound seems fine". Its cues: Dirt asked for at
  9 606 ms and Grass 231 ms later, Dirt at 11 415 and Grass 635 ms later, each within Grass's
  900 ms fade-out, so with MCL-Q12's fix Grass came back after its fade both times and Dirt was
  never heard. A plain switch with Dirt fading in (items 2a and 2b) did not happen in this run;
  its path is Grass's fade-in from its place, which was heard, and T6a2 checked it
  sample-exact in the sandbox. The window's figures on Windows (direct3d11, WASAPI): a frame
  made in 5.3 and 3.3 ms (median), shown at most 27.5 ms after its time.
- **No finding, no new question.** M2's exit holds; its review list was accepted at T6b2a.
- **M2 archived:** its section (scope, decisions, task list, exit) and its review list moved
  verbatim to `E24-archive.md`, a stub with the decisions that bind later milestones left in the
  epic; the queue's M2 history and T6b2c's entry to `QUEUE-ARCHIVE.md`; the HANDOFF's M2 lines
  trimmed. Next: M3 T0 with the user.

### M3 T0 — M3 refined with the user, MCL-Q9 built (2026-10-28)

On `map-client/e24-m3-t0`, the user present in the chat; `master` at #535 (#532 and #535
merged). The original data staged from the user's repo folder first (the exe, the four LODs, both
`.snd`, `maps.tar`, `extracted.tar`, the nine terrain tracks).

- **The common crate, looked at before proposing:** of what T6b2a's proposal named, `h3-view`'s
  `layout` (the battle's layout: hexes, stacks, heroes, the count box, the panel's and the
  book's dialogs) and `mouse` (`screen_to_hex`, the battle's cursor kinds, slots and clicks;
  `line` and `track` the battle's too) are the battle's through and through; only `ui`'s dialog
  format is common, and it goes with `ui`. `ui` depends on the battle scene's `Draw` and
  `PlayerColours` (its `draws` gives `Draw::Picture`, `Button`, `Text`, `Label`, `Darken`): the
  move gives it draw items of its own. `cursor` depends on `batch::Sprite` (in both viewers, the
  same type), `clock` on nothing; `text` on `serde_json`, `serde`; `keys` on the map's
  `timeline::{Action, Direction}` (the table made generic over a viewer's commands); `music` on
  `h3_render::Bugs` only.
- **The user's answers** (every recommendation taken): the crate `h3-ui` with `clock`, `cursor`
  and `Sprite`, `ui`, `text`, `keys` and `music`; this lane moves it as T1a (`cross-lane`, after
  a `shared/` pull request for `lanes.toml`), T1b the battle window's keys through the table,
  both windows' own keys the map's F-keys (the battle's letters left to the original's hotkeys:
  F the Faerie Dragon's, the client lane's M5 T3b); the clicks of M3 the view-only ones; the
  cursor over the map in M3 from given inputs; the task cut T1a to T7 as proposed;
  MCL-Q12's provisional part (a track asked for again during its own fade-in plays on)
  confirmed. MCL-Q9: the user asked what a "picture" is (a tile's frame of its terrain's DEF,
  fixed by the map; the water moves by its palette: 0 to 19 the coast's pieces, 20 to 32 open
  water's variants) and took the fix (a).
- **MCL-Q9 built** (`h3_map_view::sound`): `TileSounds::build` takes the profile; with the fixes
  a water tile is the shore's when it is the coast (`TileArt::coast`, below picture 20, the map
  load's test), with the original's bugs below the tables' `shore_pictures` (21) as before. The
  test's buoy on picture 20 sounds as its buoy (5) with the fixes, as the shore (69) with the
  original's; without draw lists the tile is silent with the fixes. `paint.rs` hands the
  window's and `render`'s profile on (`sequences` and `drawn` compare with `--bugs original`).
- **Docs:** M3's section in the epic (scope, decisions, tasks, exit); the queue's T1a to T7, the
  inbox's frame cost (T2) and the sound without the focus (T6) and the adventure lane's notice
  moved to `QUEUE-ARCHIVE.md`; the client lane's inbox told the decision (its proposal entry
  replaced, the old one archived here); MCL-Q9 answered, MCL-Q12's entry notes the confirmation.
- **Checked:** `clients/scripts/check.sh` all gates passed, coverage 100 % (regions, functions,
  lines); `h3map.py drawn` all the same (the 40 situations; the 13 sequences' frames and
  timelines as recorded); the scoped verify no failing step. The sandbox's container restarted
  during the setup's baseline (its background run gone, the files kept): the gates run again on
  the branch.

### M3 T1a — the common crate `h3-ui` (2026-10-28)

On `shared/h3-ui-lane` (#541) and `map-client/e24-m3-t1a` (`cross-lane`, the user's consent at
T0), the user present at the start; `master` at c4ae2fa1 (#540 merged). The original data staged
from the user's repo folder first (the exe, the four LODs, both `.snd`, `H3ab_ahd.vid`, the five
DLLs, `maps.tar`, `extracted.tar`, the whole `MP3/` folder); the link to the user's computer
dropped later, as the user expected. Asked whether to cut the move at the seam between its plain
moves (`clock`, `cursor`, `Sprite`, `text`, `music`) and its two redesigns (`ui`'s draw items, the
generic key table), the user chose one pull request for all of T1a.

- **#541** (`shared/`, merged by the user as b833af15): `lanes.toml`'s client foundation gains
  `clients/crates/h3-ui/` (its comment names the crate's parts and the decision);
  `docs/08-work-process.md`'s Common list names it. The global HANDOFF names no foundation crate:
  unchanged.
- **The crate** (`clients/crates/h3-ui`, pure, `no_std` + `alloc`, the workspace's lint header;
  depends on `h3-assets`, `h3-render`, `serde`, `serde_json`, `toml`): `git mv` of `h3-view`'s
  `clock`, `cursor`, `ui` (with its tests) and `text` (`keys`, `message`, `pack`, `plural`, tests)
  and `h3-map-view`'s `keys` and `music` (with their tests); `sprite::Sprite`, the decoded DEF
  (the two viewers' `batch::Sprite`, the same type, now one); `h3-map-view`'s `clock` and `cursor`
  deleted. `tests/purity.rs`: `h3-map-view`'s with R2-15's fixes, trimmed to a crate without a
  binary (no binary-only list; every folder under `src` walked). The headers of `clock`, `cursor`
  and `music` say they are both viewers'.
- **`ui`'s own draw items:** `h3_ui::ui::Draw` (`Picture`, `Button`, `Text`, `Darken`, `Label`,
  as the battle scene's variants were, docs kept) and `PlayerColours`; `Dialog::draws` and
  `shadow_draws` give them. `h3-view`'s scene takes them as one variant, `Draw::Dialog(ui::Draw)`
  (`scene::panel` and `Book::draws` wrap the dialog's list); `compose` draws each inside
  `Draw::Dialog` as before, `batch::names` names their assets as before. The tests' lists wrap
  them the same way (a brace-matched rewrite of `Draw::<item> { … }`).
- **The generic key table:** `h3_ui::keys::Command` (a viewer's commands: `all`, `name`,
  `defaults`, `from_name` from them), `Keys<C>` (`original`, `with_file`, `command`, `chords`),
  `Chord`; its tests with a small test type of commands (the defaults, the chords, a user's file
  and its errors). `h3_map_view::keys` keeps the map's `Command` (its directions, the window's own
  F-keys), its defaults (`DIRECTION_KEYS`, `KEYPAD_KEYS`, `WINDOW_KEYS`) and `action`, and
  `Keys = h3_ui::keys::Keys<Command>`: the window's call sites as they were (the trait in scope
  for `name`); its tests the map's defaults, names, actions and a user's file.
- **The viewers:** `h3-view`'s library paths `crate::{clock, cursor, text, ui}` and its binary's
  `h3_view::{…}` are `h3_ui::…` (the binary's own `text` adapter stays); `h3-map-view`'s window,
  `paint` and `audio` take `h3_ui::{clock, cursor, keys::Chord, music}`; both packages depend on
  `h3-ui`, both purity allowlists name it (`h3-view`'s `serde_json` is its binary's only now, left
  in the list). `clients/README.md`'s crate table and foundation list; `E24-design.md`'s two
  `music` paths; the client lane's inbox told (its imports, the scene's variant).
- **`master` merged after #541 and #542** (the client lane's E17 M5 T4a, the message box): six
  conflicts in `h3-view` (`lib`, `batch`, `compose`, `scene` and their tests), resolved with the
  branch's structure and #542's content: `h3_ui::ui::Draw::Picture`'s `size` and `Button`'s
  `player` (their docs as #542 wrote them); #542's edits of `ui`, `text::pack` and their tests
  came through the rename; its new `message` and `boxes` on `h3_ui`'s paths, the binary's two
  message-box draws wrapped as `Draw::Dialog`. Its inbox entry to this lane taken (T1a's part
  here, T1b's into its scope) and archived.
- **Checked:** `clients/scripts/check.sh` all gates passed, coverage 100 % (regions, functions,
  lines), before and after the merge; `h3drive.py drawn` (130 sequences and situations, #542's
  six message boxes among them) and `h3map.py drawn` (40 situations, 13 sequences) all the same
  (no capture: nothing drawn or played changed); the scoped verify (`--with-original-data`) no failing step but `check-lane`,
  which passes with `--cross-lane` against #541's branch (the label and `master` after #541 on
  the pull request). The container restarted during the setup's
  baseline (its run gone): `master`'s client code was CI's (`Clients` green on 78f4aaa9), so the
  gates ran on the branch only.

### M3 T1b — the battle window's keys through the table (2026-10-28)

On `map-client/e24-m3-t1b` (`cross-lane`: `h3-view`'s window, book and box keys, the user's
consent at T0), the user present at the start; `master` at 59105b2e (#545 found merged at
15:17 UTC). The original data staged from the user's repo folder first (the exe, the four LODs,
both `.snd`, `H3ab_ahd.vid`, `h3maped.exe`, the five DLLs, `maps.tar`, `extracted.tar`,
`battle-videos.vid`, the nine terrain tracks, the combat tracks and the end tunes). The queue's
entry said both "the battle's own keys as they are" and "every key of both windows through the
table"; asked how far to go (the window's keys only; with the battle's flat keys; everything cut
in two), the user answered **no cut**: every key of both windows through the table, in one task.

- **`h3_ui::keys`:** a viewer's `Command` gains its `Context` (what is open over its screen; the
  map's `()`), its `TABLE` in the user's file and `context()` (`None`: every context); a chord
  is refused on two commands one context can take (`together`), so one key may give a command in
  each; `Keys::command(context, ctrl, key)`. The user's file is a TOML file of viewers' tables
  (`VIEWERS`: `[map]`, `[battle]`; another name, or a top-level command as M2's file had, is
  refused; a file without the viewer's table is the defaults), errors prefixed by the table.
  `Window`: both windows' own nine commands (names, the F-keys and Esc, `everywhere()`: all but
  quit, a frame on and restart). `Keys::check(known)`: every chord's key one of the adapter's
  names, never Control alone (`CONTROL`), the windows' `check_keys` moved here. `Chord::new`.
- **`h3_map_view::keys`:** the nine window commands are `Command::Window(Window)`; names and
  defaults unchanged; `Context = ()`, `TABLE = "map"`. The window's matches and its usage (the
  file's `[map]` table) follow.
- **`h3_view::keys`** (new, *provisional*): `Context` (`of(message, book, aim)`: a box, else the
  book, else an aim, else the battle) and `Command`: `Window(_)`; with nothing open `LogUp`,
  `LogDown` (↑ ↓, 0x4758FB), `DragonSpell` (F, 0x475C17), `Button(id)` (`BUTTONS`: the panel's
  seven, S, R, O, A, C, W, D and Space, `battle_panel_base` 0x46B610's scan codes, the layout's
  `keys` until now); `Book(book::Key)` (Esc and Enter close, A, C, ← → ↑ ↓; 0x59D1B4);
  `Message(message::Key)` (Enter OK, Esc Cancel; 0x4F6990); `AimCancel` (Esc 0x5A317A, the right
  button 0x5A3184); merged with #547 (the client lane's result window, mid-task), `ResultOk` in
  `Context::Result` (Enter and Escape, 0x4715C3; its `keys` in `result::window` gone).
  Names: the window's own, `log-up`, `log-down`, `dragon-spell`, `surrender`,
  `retreat`, `options`, `auto-combat`, `spell-book`, `wait`, `defend`, `book-close`,
  `book-adventure`, `book-combat`, `book-previous`, `book-next`, `book-tab-up`,
  `book-tab-down`, `box-ok`, `box-cancel`, `aim-cancel`, `result-ok`.
- **The library's keys typed:** `Book::key` and `book::Open::key` take a `book::Key` (the key
  names gone); `message::Open::key(key, by)` a `message::Key` and the key's name (its release
  still by name; the OK is `OK` or `YES`, whichever the box has, Cancel `NO` when it has one).
  `h3_ui::ui::Button`'s `keys` and `Dialog::button_for_key` gone, the battle layout's seven panel
  `keys` with them (the table's defaults their only copy), the message box's buttons built
  without.
- **The battle window** (`window.rs`): `--keys <file>` (else `keys.toml` here), its `[battle]`
  table, checked against `h3_sdl::Key::ALL`, "keys: …" in its report; Ctrl tracked for chords;
  every key looked up in the context of what is open; the window's own (the map's F-keys: P, →,
  Home, M, E, V, H and F11's old places gone), the box's, the book's, the aim's, the log's, the
  dragon's and the buttons' each as before; the opening ended by pause, a frame on or restart as
  by P, → and Home. `viewer_key` and `book_key` gone. Unattended every key but quit does nothing
  (quit looked up as the battle's). The stills' `key:` actions (`spellbook::still`) go through
  `Keys::original()` in the still's context (the harness's keys are the original's); the
  presser turns the book by `book::Key`. `h3_sdl::Key::ALL` lacked `Return` (the variant and
  its SDL key existed; the check of the battle's defaults found it): added.
- **Behaviour changed** (M3's review list): the book takes every key while open (the panel's
  letters and F reached the battle under it); unattended, Esc quits whatever is open (it gave the
  aim up or did nothing under a box) and the right button gives no aim up; a chord with Ctrl held
  gives nothing unless bound (Ctrl and ↑ scrolled the log).
- **Checked:** `clients/scripts/check.sh` all gates passed, coverage 100 % (regions, functions,
  lines); under Xvfb `h3-view window scenarios/battle.toml --human both --check 20` 295 of 295
  frames exact, the battle the bot's; `smoke` `--leave retreat` and (5 000 gold) `--leave
  surrender` 75 of 75 each, left and over; `situations/book.toml --check 15` 220 of 220; real keys
  (`xdotool`) on the book's situation: C opens the book, F5 pauses under it, → turns its page,
  the first Esc closes it and the window runs on, the next Esc quits; a user's file
  (`quit = ["Ctrl+F12"]`, `pause = ["P"]`): Esc no longer quits, Ctrl+F12 does; `pause =
  ["Space"]` refused (Defend's); `h3-map-view window walk-1 --check 10` with the same file 98 of
  98 exact. After merging `master` (#547): check.sh again all gates passed, coverage 100 %;
  `h3drive.py drawn` all 133 the same (the book's, the boxes' and the result window's stills,
  whose `key:` actions now go through the table, included; 131 before the merge); `h3map.py
  drawn` (before the merge, which does not touch the map) 40 situations and every sequence the
  same; `smoke` `--leave retreat` with the result window open to the run's end, 8 of 8 exact;
  real keys on `smoke` in hot seat: R asks, Enter retreats, the result window opens, Esc (its
  `result-ok`) closes the battle's screen. The scoped verify (`--with-original-data`): no failing step but `check-lane`, which passes with
  `--cross-lane` (the pull request's label).
- **Docs:** `clients/README.md` (both windows' keys, the crates' rows), both usages, the E17
  design note's two mentions of the layout's `keys` and the moved keys; M3's review list opened
  (the file's tables, the contexts, the behaviour changed); the client lane's inbox told (its
  2026-10-26 entry done by this).

### M3 T2a — the adventure screen's dialog at a map's start (2026-10-28)

On `map-client/e24-m3-t2a`, the user present; `master` at d00303e8 (#548 found merged). The
original data staged from the user's repo folder first (the exe, the four LODs, both `.snd`,
`H3ab_ahd.vid`, `h3maped.exe`, the five DLLs, `maps.tar`, `extracted.tar`, `battle-videos.vid`,
the nine terrain tracks, the combat tracks and the end tunes). T2 was too big for one task: the
user cut it in two (T2a the panels right, T2b the frame's fixed cost), then, asked whether the
sequences' button changes during a walk should go to a task of their own, kept T2a whole.

- **The reading** (design §4.1; 29 map entries): `adventure_dialog_create` 0x401510 in full (its
  45 items in order: three areas, the fourteen buttons with their hotkeys, the lists' pictures,
  bars and frames, the status bar, a text and the chat's edit), `adventure_full_update`'s panel
  calls, the resource bar's panel 0x558BA0 (`aresbar.pcx` at (3, 575) in the viewer's colours,
  the amounts and the date), and every site that sets a button's state: the underground (one
  level, `iam003.def` underground), the quest log (a visited quest), sleep and cast a spell (a
  hero selected; sleep's `iam011.def` asleep), move hero (a green arrow, `make_hero_path`'s third
  argument; a disable at a walk's and a key's step's start, through the window manager at once),
  next hero (an awake hero that can move: `hero_can_move` 0x4E5F30), the lists' arrows (their tops
  and lengths). Checked against the running original's dialog (`dialog_items` on
  `heroes-objects` and `path-objects`); VCMI 1.7.5's `AdventureMapShortcuts` agrees on each.
- **Built:** `h3_ui::paint` (the dialog items' drawers, ported from `h3_view::compose`, from any
  batch through `Assets`; the client lane's inbox told), `Dialog::draws` giving a button
  `Shown`'s colours and DEF by its id; `adventure.toml`'s `[dialog]`, `[resource_bar]` and
  `[panels]` (`h3_map_view::layout`); `h3_map_view::panels` (the buttons' states, DEFs and
  colours); the live part's `kingdom` (`Kingdom`, `KingdomHero`, `Selected`, `Kingdom::read`, a
  map's start's) and `buttons` (`Buttons::of`, or the timeline's); the scene draws the dialog and
  the resource bar after the frame picture (`Draw::Dialog`), the batch holds fonts; the timeline
  carries the buttons (`Start::kingdom`, `Start::buttons`, `State::buttons`, a `buttons` line,
  set at a path's click, a key's step and a walk's end), `changed` looks outside the mask;
  `render` draws a still's parked cursor at the live part's `pointer`; `sequences` reads
  `clients/map/masks.toml`, the painter keys its views by the buttons. The harness: `Mem.kingdom`,
  `Mem.buttons`, `Mem.visited_quest`; `live_part` writes `kingdom`, `buttons` and `pointer`; `MASKS` from
  `masks.toml`, `view_changed` outside it; the timeline's `buttons` lines.
- **First model, corrected by the recordings:** built from the reading alone, move hero went
  off at a path's walk's start and at every key's step, next hero followed the points at 0; the
  first run of the 13 sequences showed 22, 17, 12, 3, 30, 12 and 20 frames of the seven walks
  differing (494 pixels each: move hero's plate). The recordings' `buttons` lines gave the
  original's moments (design §4.1, "Seen in the sequences"): the timeline now keeps move hero
  through a path's walk to its end, turns it on at a key's step that lifts the hero and off at
  one once lifted, and the live part carries the states memory holds (`walk-sail` starts with
  next hero off, set before the harness wrote the hero's points). Compared again from the same
  runs (`frames --reuse`, each live part's `buttons` from its run's state at 0).
- **Checked:** the 40 situations exact with the smaller mask (386 868 pixels compared each, 335 616
  before), twice (the second run keeping the live parts' `buttons`); the 13 sequences recorded
  again, each timeline (now with `buttons` lines and the smaller mask's screen boxes) the viewer's
  line for line, all 459 frames exact (`timelines/` and the `-frames.txt` counts recorded);
  `h3drive.py drawn` 132 of the battle's sequences and situations the same (the common crate's
  change draws nothing new there); `clients/scripts/check.sh` all gates passed, coverage 100 %
  (regions, functions, lines); after merging `master` (#549 to #551) check.sh again and `h3map.py
  drawn` all the same; the scoped verify (`--with-original-data`) without a failing step.
- **Not done (T2b):** the frame's fixed cost, measured before and after on the user's machine.

### M3 T2b1 — the frame made once (2026-10-28)

On `map-client/e24-m3-t2b1`, the user present; `master` at 2e5cf6a (#553 found merged). The
original data staged from the user's repo folder (the exe, the four LODs, both `.snd`, `maps.tar`,
`extracted.tar`, the nine terrain tracks). Profiled first (design §4.2): T2b was too big for one
task, so the user cut it (T2b1 what every frame pays, T2b2 a frame redrawing only what changed)
and asked for a spatial structure for the draw objects (a jump anywhere, HotA's large maps) and
the widening computed once. `map-measure.cmd` gained a hands-off run on *Tale of two lands*
(144 × 144); the user's checkout at #545, so the exe read `master`'s map data from
`clients/out/win/map`.

- **Built** (*provisional*): `h3_render::screen::Widening` (the table), `Screen::cut`,
  `Screen::copy_rect`; `h3_ui::cursor::Cursor::rect`; `compose::Composer` (`compose` its first
  scene); `scene::TileIndex` and `scene_in` (`scene` through them; `check_shapes` split into the
  index's objects and the scene's heroes); `batch::spelled`; the shadows' masks once a draw; the
  painter's composer, levels' indexes and `frame_into` (the cursor's rectangle widened over the
  frame), `Painter::screen` for `sequences`; the window's frames into kept vectors (`Maker`) and
  `--check`'s `widened` line.
- **Checked:** `h3map.py drawn` all 40 situations and 13 sequences' frames the same; `sequences`
  13 of 13 as recorded; the window under Xvfb (`walk-1`, `path-objects` and *Tale of two lands*,
  hardware and drawn cursor): every frame read back exact, every frame made as the screen's own
  pixels widened one by one, every change the headless loop's; `clients/scripts/check.sh`.
- **Measured:** callgrind on `path-objects` 101 → 25.2 million instructions a frame; the
  sandbox's medians `walk-1` 6.4 → 1.3 ms, `path-objects` 10.8 → 4.5, *Tale of two lands* 14.3 →
  4.2. The user's machine (`map-measure-t2b-before.txt`, `map-measure-t2b1.txt`): a median 4.2 →
  0.7 ms on `walk-1`, 3.3 → 2.3 on *Tale of two lands* (p95 11.3 → 2.7), 3.3 → 0.2 on
  `path-objects` with input (p95 7.3 → 2.5; the check's run 3.4 → 1.7); every frame exact.
  Merged as #557 (after `master`'s #556, its inbox note beside ours).
- **The user's finding:** the Gold Dragon's head under the Dragon Utopia on `path-objects`: the
  original's own drawing (the start view's still is exact), **MCL-Q13** (keep, recommended).

### M3 T2b2 — a frame redrawing only what changed (2026-10-28)

On `map-client/e24-m3-t2b2`, the user present; `master` at 1788858 (#557 found merged; the
previous session's commit of T2b1's numbers on this branch merged in). The cut proposed (the
drawers' spans, the view's cells, the changed rectangles widened and uploaded) declined by the
user: "no cut". The original data staged from the user's repo folder as for T2b1.

- **Built** (*provisional*, design §4.3): `draw::Spans` and the drawers by spans,
  `Batch::drawable`; the gems through the raw drawer; `scene::ViewPlace` (`Scene::view`);
  `compose::Composer`'s cells, the move of the clip's pixels, `damage` and `compose_owned`;
  `damage::History` and `tidy`; `h3_render::screen::Widening::widen_rect` and
  `h3_sdl::Window::present_part`, `present_read_part` (additive); the painter's history,
  `Widened`, `fresh`, assets decoded when found missing; the window's texture state, `--check`'s
  `afresh` line and the `uploaded` report.
- **Checked:** `h3map.py drawn` all 40 situations and 13 sequences' frames the same; the window
  under Xvfb (`walk-1`, `path-objects` and *Tale of two lands*, standing and with input driven by
  `xdotool`, hardware and drawn cursor): every frame read back exact, every frame made as the same
  frame composed afresh, every change the headless loop's; `clients/scripts/check.sh`.
- **Measured** (the sandbox): callgrind 25.1 → 5.6 million instructions a frame on
  `path-objects` standing, 10.1 → 3.2 on `walk-1`; a frame made in a median 0.9 → 0.3 ms on
  `walk-1`, 4.0 → 1.0 on `path-objects` and 3.2 → 1.3 on *Tale of two lands* standing, 1.4 → 0.6
  and 1.9 → 1.5 with input; a quarter of `walk-1`'s pixels uploaded, 5 to 11 % standing. The
  window cross-built for the user's machine (`clients/out/win/h3-map-view.exe`, also
  `h3-map-view-t2b2.exe`). The user's machine (`map-measure-t2b2.txt`, 2026-10-29): a median 0.7
  → 0.2 ms on `walk-1`, 2.3 → 0.9 on *Tale of two lands*, 0.2 → 0.0 on `path-objects` with input
  (p95 2.5 → 1.5); every frame read back exact and as composed afresh. Merged as #561.

### M3 T3a — the minimap as a still (2026-10-29)

On `map-client/e24-m3-t3a`, the user present; `master` at 64e2d5d (#561 found merged; CI green on
it). The original data staged from the user's repo folder first (as for T2a); the sandbox
restarted once mid-session (its files kept; the baseline check, then in its coverage step, did
not finish: CI's run on `master` stands for it). T3 cut with the user before the reading (T3a the
minimap as a still, T3b when it is drawn again); T2b2's numbers from the user's run recorded first
(the T2b2 entry above).

- **The reading** (design §4.4; map entries for `minimap_draw` 0x412C40, read, and
  `tile_object_entrance` 0x4FCAA0, `game_screen_player` 0x4CE9B0 (H3API's `GetPlayer`
  confirmed), `view_world_open` 0x6AAC3C and `view_world_scale` 0x68C6B8, inferred): a tile's
  colour (unexplored black; the terrain DEF's 16-bit palette entries 8 and 9, not 22 as §1.8 had
  it: the colours start at +0x1C; VCMI's `minimapUnblocked` and `minimapBlocked` agree), the
  selected hero's and the standing heroes' owners, the owned classes' blocked and entrance tiles,
  21 decorations' blocked tiles; the four sizes' pixels (108: columns 2, 1, 1, rows 1, 1, 2); the
  box `radar.def` by size at the corner times 4.0, 2.0, 1.33 or 1.0, truncated, cut to the area.
- **Built** (design §4.4): `record` (0x505610's walks out of `sound`), `minimap` (`Base`, `Tiles`,
  `Look`, the exact scale), the scene's `Minimap` and `Radar` draws, the composer's, `TileIndex`'s
  level-wide terrain check; the tables' `[map.records]` (moved out of `[map.sound]`) and
  `[map.minimap]`; `OwnerArt::colour`; `masks.toml`'s `stills` (the situations' mask, `panels`
  but the minimap; the sequences keep `panels` until T3b). The kept frame: the minimap's draws
  stay put through a move, a cell clear of the clip keeps its pixels, a palette step counts only
  where it rotates the minimap's entries, its comparison made once a scene (a first build redrew
  and uploaded the minimap at every palette step and its gap to the view at every scroll: 5.5 →
  11.7 % uploaded standing on `path-objects`; fixed before the checks below).
- **Checked:** `h3map.py set`: the 40 situations recaptured, each 0 differing of 407 604 pixels
  (the minimap's 20 736 now among them), first time; `drawn --record`: the 40 situations' digests
  recorded anew, the 13 sequences' 459 frames the same and their timelines as recorded; the
  window under Xvfb (`walk-1`, `path-objects`, *Tale of two lands*): every frame read back
  exact, made as the same frame composed afresh, every change the headless loop's; side by side
  with `master`'s build (`--measure 20`) the same medians (0.5, 1.0, 1.4–1.5 ms), uploads 24.9 →
  25.5 % on `walk-1`, the same standing; unit tests (the walks, the colours, the four sizes, the
  box and its exact scale, the composer's rectangles, box cut and kept frame).

### M3 T3b — the minimap over time (2026-10-29)

On `map-client/e24-m3-t3b`, the user present (T3b kept whole when asked, the reading being larger
than the queue said: 13 direct call sites and 20 through a wrapper); `master` at a26897f (#563
merged; CI green). The original data staged from the user's repo folder first; the baseline
check passed (`verify-once`, clients code 313f991).

- **The reading** (design §4.4, "When it is drawn again"; map entries `minimap_redraw` 0x413790,
  read, and nine inferred: `minimap_click` 0x40A0C0, `network_map_redraw` 0x405E30,
  `view_world_run` 0x5FBF90, `hero_relocate` 0x41D930, `adventure_spell_cast` 0x41D090,
  `visit_and_update` 0x4AAAA0, `hero_battle_on_map` 0x4AAFD0, `battle_aftermath` 0x4AC930,
  `turn_start_minimap` 0x4CA530): M3's sequences meet a scroll that moves the view (shown at
  once: T3a had its show argument as 0), a lift that centres the view (`hero_select` with c: a
  key's step lifting a hero not lifted) and each step's end (`after_step_guard_check` 0x480090's
  first call, 0x4800A5). The last was found by the recording, not the first reading: `walk-1`'s
  minimap changed at every step's end (300, 600, …), so the guard's test, set aside as M4's, is
  every step's; the step's own call (0x4810E6) is a network game's only. The rest (the minimap's
  click, the view world, teleports, spells, visits, battles, turns, the hero screen) M4's and
  later.
- **Built** (design §4.4): the timeline's `Minimap` (the corner and the selected hero's tile at
  the last draw) in each `Draw`; the live part's `minimap` (`minimap::Shown`); the scene's box and
  selected hero from it; `Base` with the explored tiles made once with the index, `Tiles` sharing
  the level's colours (`Arc`) with the heroes over them; `masks.toml` one list (`panels`; T3a's
  `stills` gone, `h3map.py` and the runbook with it); the trap of the index extended to the
  explored tiles (`TRAPS.md`).
- **Checked:** `h3map.py frames` (the minimap unmasked for the run): the original's 13 sequences,
  their timelines recorded anew (the screens' boxes take in the minimap), every kept screen of
  459 exact with the viewer (`--reuse` after the build; the old build differed on 48 of 64 frames
  of `walk-1`, the box running ahead of the original's through a step's frames), every timeline
  line as the viewer's; `drawn --record` (the 40 situations' digests unchanged, the sequences'
  459 recorded anew); the window under Xvfb (`walk-1`, `path-objects`, *Tale of two lands*):
  every frame read back exact, made as the same frame composed afresh, every change the
  headless loop's; unit tests (the redraws' moments, the scene's box and hero as last drawn, the
  shared colours); `check.sh` (below).
- **Measured** (the sandbox, `--measure 20` beside `master`'s build): a median 0.3 → 0.2 ms on
  `walk-1`, 0.7 and 0.7 on `path-objects`, 0.9 and 0.9 on *Tale of two lands*, uploads the same;
  callgrind on *Tale of two lands*: a scene 2.33 → 1.55 million instructions.

### M3 T4a — the hero and town lists as stills (2026-10-29)

On `map-client/e24-m3-t4a`, the user present (T4 cut into T4a, the stills, and T4b, the lists over
time, as T3 was); `master` at 25ff803 (#567 merged; CI green on it). The original data staged from
the user's repo folder first; the baseline check passed (`verify-once`, clients code 864e300).

- **The reading** (design §4.5; map entries `hero_list_slot` 0x403560 read, `town_list_slot`
  0x4037A0, `hero_movement_frame` 0x4E5330, `hero_mana_frame` 0x4E5380, `town_list_frame`
  0x5BD700, `dialog_picture_set` 0x4504C0, `hero_list_scroll` 0x403220 read, `town_list_scroll`
  0x403280 inferred; `hero_table` 0x679DD0 corrected to its 163 records): a hero's slot its
  portrait by its +0x34, its bars by its points left and mana (the last three frames from 2 300,
  2 500, 2 800 points and 116, 145, 170 mana; mana rounded up below; VCMI 1.7.5 linear to 25); a
  town's slot `itpa.def` 2 + 2 × faction with a Fort, 20 + 2 × faction without, one more when it
  built today; the empty slots `hpsxxx.pcx`, frame 0; the selections drawn after the items (the
  town's mark, frame 1, over its picture; the hero's frame). From the running original: the
  lists in the map's order, a town record's place its entrance.
- **Built** (design §4.5): `h3_map_view::lists`; `panels::dialog` (the slots) and `selections`;
  the live part's kingdom with portraits, points, mana and towns (the old form still read); the
  tables' `[map.lists]`; the layout's items; `h3_ui::ui::Shown::pictures`; the situations'
  `points`, `mana`, `built` and `select`; the harness's writes and selection (`h3mem`:
  `kingdom`'s new fields, `town`, `town_list`, `write_mana`, `write_town_built`;
  `h3adv.select_town`; `h3map.lists`, `town_index`); `masks.toml`'s `stills`; the gate's lists
  (the bars' and towns' frames, the 165 pictures at 48 × 32).
- **Checked:** `h3map.py set`: the three new situations (`lists-heroes`, `lists-scrolled`,
  `lists-towns`: the bars' every edge, both lists scrolled by a selection, a town's mark over its
  picture, empty slots, five factions with and without a fort, built today) exact first time,
  then the whole set recaptured with the lists compared (`stills`): each of the 43, 0 differing of
  425 588 pixels (the lists' 17 984 now among them; 407 604 before), shipped maps' heroes with their own
  points and mana included; `drawn --record`: the 43 situations' digests recorded anew, the 13
  sequences' 459 frames the same and their timelines as recorded (they keep `panels`); the gate
  with the lists; unit tests (the bars' edges, the towns' frames, the tables' rules, the slots
  from their tops, the selection's slot and draws, the kingdom's both forms, the situations'
  writes and selection, a portrait past the tables refused); `check.sh` (100 % coverage); the
  window under Xvfb (`lists-scrolled`, `lists-towns`, `walk-1`, *Tale of two lands*,
  `--check 10`): every frame read back exact, made as the same frame composed afresh, every
  change the headless loop's.
- **Measured** (the sandbox, `--measure 20`, alternating with `master`'s build twice each): the
  same within noise, a frame made in a median 0.5–0.6 ms on `walk-1` (both), 1.1–1.2 and 1.1 on
  `path-objects`, 1.6 and 1.4–1.5 on *Tale of two lands*; uploads the same (25.6–25.7, 5.2,
  11.2 %). This sandbox is slower than T3b's (its 0.2, 0.7, 0.9 ms).

### M3 T4b — the hero and town lists over time (2026-10-29)

On `map-client/e24-m3-t4b`, the user present (T4b kept whole, the seam between the viewer and
the recording named in case it grew); `master` at 86f164b (#570 merged; CI green on it). The
original data staged from the user's repo folder first; the baseline check passed
(`verify-once`, clients code cc440c1).

- **The reading** (design §4.5, "When they are drawn again"; map entries `hero_list_update`,
  `town_list_update` (read in full), `hero_list_slot`, `hero_select` amended, `hero_relocate_spell`
  0x41D360 inferred): every call of the slot is (−1, 1, 1), the selected hero's slot drawn and
  its rectangle (609, 212 + 32 i, 48 × 32) shown, the mana bar left out (MCL-Q14); both updates
  with their redraw draw every item of the dialog. M3's sequences meet three: each step's end
  (the slot, after the step's points are taken at 0x48118C), a lift that centres the view
  (`hero_select` with c: both lists, the hero list scrolled to the hero, shown with the whole
  screen) and a walk's end that leaves the hero unable to move (0x4081B9: both, the whole
  screen). The town's item left at frame 1 changes nothing on screen (every redraw puts the mark
  over the picture and mark there; every town list update sets the frames again first).
- **Built** (design §4.5, "Built"): the timeline's `Draw::kingdom` (the lists as last shown), the
  loop's memory points from `Start::points` (the recording's `points` lines,
  `Sequence::points`), the redraws at those three moments (`lists::top_showing` for the lift's
  scroll), the timeline's `points` lines; `paint`'s scene draws the lists from the draw; the
  harness's `points` each millisecond (`Mem.points`) and the live part's kingdom read before a
  sequence's `points` are written (the list as shown at 0: 1 500 points on `seq-walk`'s hero,
  memory's 2 000 once written, first shown at the lift at 100); `masks.toml` one list (`panels`;
  T4a's `stills` gone, `h3map.py` and the runbook with it); `Kingdom::read`'s form before T4a
  gone; a trap (`TRAPS.md`: a sequence's points are the recording's).
- **Checked:** `h3map.py frames` (the lists unmasked): the original's 13 sequences recorded
  anew, their timelines as before but the new `points` lines and three of `walk-3`'s `screen`
  boxes, which now reach the movement bar (x 610); with each new timeline in place (`--reuse`)
  every kept screen of the 459 exact with the viewer and every timeline line as the viewer's
  (`walk-1` first, alone: its run against the old timeline differed on 57 of 64 frames, the
  points unknown); every frame at 0 exact with the kingdom as shown; `drawn --record` (the 43
  situations' digests unchanged, the sequences' 459 recorded anew); unit tests (the redraws'
  moments, the slot's points alone, a slot off the list, the lift's scroll, the walk's end, the
  `points` lines, `Sequence::points`, `top_showing`, the kingdom's refusals); `check.sh` (100 %
  coverage); the window under Xvfb (`walk-1`, `walk-points`, `lists-scrolled`, *Tale of two
  lands*, `--check 10`): every frame read back exact, made as the same frame composed afresh,
  every change the headless loop's.
- **Measured** (the sandbox, `--measure 20`, alternating with `master`'s build): a frame made in
  a median 0.5 ms on `walk-1` (both), 1.0–1.2 and 1.1–1.2 on `path-objects`, 1.2–1.3 and
  1.4–1.5 on *Tale of two lands*; uploads the same on `path-objects` (5.2 %) and *Tale* (11.2 %),
  on `walk-1` 25.6–25.7 → 31.0 %, p95 2.2–2.3 → 2.8–3.5 ms: a bar that changes is a dialog draw,
  which rebuilds the panels' background and uploads the whole screen (13 of `walk-1`'s frames,
  as a button's change already did). Queued as T4c: a changed panel item drawn again alone (T5's
  status window and T6's hints change far more often).

### M3 T4c — a changed panel item drawn again alone (2026-10-29)

On `map-client/e24-m3-t4c`, the user present (T4c taken as the session's task); `master` at
2f7ab39 (#572 merged; CI's Clients run green on it). The original data staged from the user's
repo folder first. The local baseline was stopped early (it had begun testing the working tree
with the change in it; `verify-once` records nothing with uncommitted changes); CI's green run on
`master` stands for the starting point.

- **Built** (design §4.6): `h3_ui::paint::reach` (additive in the common crate: a picture's, a
  button's and a darkened rectangle's reach; unknown for texts, labels and missing assets;
  `button_frame` shared with the drawer); in `compose`: `changed_areas` (the changed items' old and
  new reach, or those between the lists' common start and end; joined; refused near a clip whose
  pixels may move, for an unknown reach, or with a dialog draw over the background),
  `Background::redraw` on a kept scratch screen (the frame's picture cut, the dialog's items that
  may reach an area whole), the view's cells meeting an area dirty in `Changed`, the areas in the
  restored rectangle of a scene drawn whole over the background; `frame_picture` cut to a
  rectangle.
- **Checked:** unit tests (the reach of each kind; scenes with a button's frame, a darkened
  rectangle over a gem, an item fewer and more, a label's text, a picture turned label, another
  flag palette, a dialog draw over the view, a reach meeting the clip, each as a fresh composer's
  with every changed pixel in the damage; the button's damage its 2 × 2 rectangle alone; a batch
  lacking the frame's picture or the label's font; `joined`); 100 % coverage of `h3-ui` and
  `h3-map-view`; clippy; the window under Xvfb (`walk-1`, `walk-points`, `lists-scrolled`,
  *Tale of two lands*, `--check 10`): every frame read back exact, made as the same frame
  composed afresh, every change the headless loop's; `drawn`: 43 situations and 13 sequences the
  same.
- **Measured** (the sandbox, `--measure 20`, alternating with `master`'s build twice each; the
  sandbox slower than T4b's, `master`'s median 0.8 ms on `walk-1` against T4b's 0.5): `walk-1`
  uploads 30.8–31.0 → 24.6–24.8 % (25.7 before T4b), p95 3.6–4.0 → 2.9–3.0 ms, median 0.8 both;
  *Tale of two lands* 11.2 % and `path-objects` 5.2 % as before, their times within the spread.
- **Left:** a text's reach (T5's texts change with each selection and day; queued with T5).

### M3 T5a — the resource bar's amounts and the date (2026-10-29)

On `map-client/e24-m3-t5a`, the user present (T5 cut with the user at the seam T4c's session
proposed: T5a the resource bar, the date and a text's reach; T5b the status window); `master` at
445b145 (#574 merged; CI's Clients run green on it). The original data staged from the user's
repo folder first. The local baseline stopped with the session's first background jobs (it was
testing the edited tree); CI's green run on `master` stands for the starting point.

- **The reading** (design §4.7; map entries `resource_bar_redraw` 0x558F20 read,
  `resource_bar_order` 0x641008, `decimal_format` 0x660A1C, `date_format` 0x682E64, `shown_player`
  0x69778C inferred, `hero_select` amended): the constructor's eight texts alone (`smalfont.fnt`,
  colour 4; the amounts left-aligned, the date centred), their texts from the shown player's
  record +0x9C by `%d` and the date from the executable's format with GENRLTXT's rows 63 to 65 and
  the game's month, week and day; a hero's selection sets them again from memory (0x417F48), a
  town's does not.
- **Built** (design §4.7): `h3_ui::paint::text_reach` and `reach` for texts and labels (additive in
  the common crate: the box widened by the font's overhangs); the catalogue's keys
  `map.resources.amount`, `map.resources.date`, `word.month`, `word.week`, `word.day` (common,
  additive); `panels::KEYS`, `panels::resource_bar` with the texts, `Tables::texts`; the live
  part's `Kingdom::resources` and `date` (`live::Date`; a map's start none and the first day);
  the layout's labels 1001–1008; the viewer's `texts.rs` (the original's forms of its keys from
  the archives and `Heroes3.exe`), loaded by `render`, `window`, `gate` and `sequences`; a
  situation's `resources` and `date` (with a hero's selection, a day of the game's) and the
  harness's writes (`Mem.write_resources`, `write_date`; `Mem.kingdom` with both); the situation
  `resources-date`; `masks.toml` without the bar's eight rectangles.
- **Seen:** MCL-Q15 (an amount of eight digits cut to seven by its box; the viewer the same).
- **Checked:** `h3map.py set --only resources-date` exact first time (the bar cropped and looked
  at: both draw every amount and the date); then the whole set with the bar compared: each of the
  44 exact, 0 differing of 435 128 pixels (the bar's 9 540 now among them, 425 588 before; the
  shipped maps' starts with their difficulty's resources and the first day included); `frames`:
  the 13 sequences, every frame exact (459), their timelines as before but two: `scroll-limits`'
  box at 590 ms (the cursor in the corner, now unmasked over the date: 790,585 → 782,581) and
  `scroll-0`'s `0 points` (1700 in T4b's run, 1500 in this one: the new game's random army sets
  the start's points; a trap), both taken from the run and compared again (`--reuse`); `drawn
  --record` (every digest anew), then `drawn` all the same; unit tests (the reach of texts and
  labels and the font's overhangs, a label's change drawing its cell alone, a changed item of
  unknown reach without the label's font, the kingdom's new fields and their refusals, the bar's
  texts by their keys and as the layout draws them, the situation's resources and date, the
  viewer's keys and the executable's strings); `check.sh` (its coverage found three regions of
  the composer a label had reached: tested with a batch lacking its font; then 100 %); the
  window under Xvfb (`walk-1`, `walk-points`, `lists-scrolled`, `resources-date`, *Tale of two
  lands*, `--check 10`): every frame read back exact, made as the same frame composed afresh,
  every change the headless loop's.
- **Measured** (the sandbox, `--measure 20`, alternating with `master`'s build): the same within
  the spread; `walk-1` 24.6–24.8 % uploaded (both), a frame made in a median 0.4 ms (0.5 on
  `master`), p95 1.8–2.0 (1.7–2.1); *Tale of two lands* 11.2 %, 1.2 (1.3); `path-objects` 5.2 %,
  0.9 (0.9).

### M3 T5b1 — the status window of a hero and a town (2026-10-29)

On `map-client/e24-m3-t5b`, the user present (T5b cut with the user: T5b1 a hero's and a town's
window and the choice between them, T5b2 the date's and the kingdom's and when each shows; the
other kinds, another player's turn, a resource found and a text, M4's); `master` at cb96348
(#579 merged). The original data staged from the user's repo folder first; the baseline
(`scripts/verify-once.sh clients` on `master`'s code) passed every gate.

- **The reading** (design §4.8; map entries `status_window_choose` 0x415DE0 read in full, its
  kinds' builders 0x415F20 to 0x4164F0, `status_window_set_kind` 0x415D60, `status_hero_create`
  0x451AB0 and `status_town_create` 0x4521F0 read in full, `town_garrison_army` 0x5C1460, the
  other kinds' panels by their pictures, the H3API functions the windows call, the tables
  0x660BB0, 0x660BD0, 0x660C08 and the strings 0x660CB0, 0x660CD0): the eight kinds and the
  chooser (the selection's window at a human's turn, a timed kind's end), the builders' `force`,
  the call sites' arguments in M3's paths (a path's rebuild and a key's step build the same
  window again), the status window's click (T5b2, T6); the hero's window (the large portrait,
  the name, the skills cut to 0–99 and power and knowledge to at least 1, the spell points, the
  morale and luck + 3, the army by slot) and the town's (`itpt.def` by the list's frame less 2,
  the hall and fort by the buildings standing, the garrison hero's mark, the silo's one or two
  resources, the income's low 16 bits, the army shown packed).
- **Built** (design §4.8): `h3_map_view::status` (the window from the live part's status and the
  selection, the panels' `Shown`), the layout's `status_hero` and `status_town`, the tables'
  `[map.status]`; the live part's `Kingdom::status` (`live::Status`, `HeroStatus`, `TownStatus`,
  `Stack`; `Status::read`; a map's start the selection's), `MapArt::hero_names` (a map's own hero
  names; `h3_ui::text::decode_1252`, additive); the catalogue's keys `map.status.number` and
  `map.status.thousands` and the family `town-name` (`optional`, additive in
  `h3_ui::text::keys`); the viewer's `texts.rs` with the families `hero` and `town-name` keyed by
  their ids, counted in their tables; the gate's checks of the window's DEFs, pictures and large
  portraits; a situation's hero `skills`, `morale`, `luck`, `army` and town `garrison`,
  `garrison_hero`, `silo` (the writer's map building 9), the harness's writes
  (`Mem.write_skills`, `write_bonuses`, `write_army`) and its read (`Mem.status`,
  `h3map.status_toml`: a town's name as its `TownName.txt` record, a hero's `own_name`); the
  situations `status-hero`, `status-town`, `status-garrison`; `masks.toml`'s `panels` without the
  status window.
- **Seen:** MCL-Q16 (a town's income by its low 16 bits; unreachable in play). The first capture
  of `status-town` and `status-garrison` showed the town's army packed (the reading had missed
  that the place steps with filled slots only, 0x452A38); fixed, both exact. `shipped-good-witch`'s
  hero has a name of its own (its map object's): the window takes the map's (`own_name`).
- **Checked:** `h3map.py still` on `lists-heroes` and `lists-towns` exact first time with the
  window unmasked (a hand-made live part first, then the harness's), the three new situations
  (`status-hero` first time; the towns' after the packing); the whole `set`: the 47 situations
  exact, 0 differing of 469 344 pixels (the window's 34 216 now among them; `shipped-good-witch`
  failed once on the hero's own name, captured again after `own_name`: exact); `frames`: the 13
  sequences, every frame exact (459), their timelines as before but `scroll-limits`' start points
  (1700, the start's random army: taken from the run and kept by `--reuse`); `scroll-0`'s start
  had 10 gems where T5a's run had 15 (the trap extended); `drawn --record`, then `drawn` all the
  same; unit tests (the windows' items, frames, texts and hidden places, the bounds and the
  packing, the income's 16 bits, the names, the refusals of `Status::read`, a map's start's
  window, the map's own names, the situation's writes, the tables' and layout's checks, the
  families' keys); `check.sh` (its coverage found the refused fields of `Status::read` and the
  scene's status error: tested; then 100 %); the gate (every new DEF with its frames,
  `cprsmall.def` 151, the pictures and the 163 large portraits at 58 × 64); the window under
  Xvfb (`status-hero`, `status-town`, `status-garrison`, `walk-1`, `lists-scrolled`, `--check
  10`): every frame read back exact, made as the same frame composed afresh, every change the
  headless loop's.
- **Measured** (the sandbox, `--measure 20`, alternating with `master`'s build twice each): the
  same: `walk-1` 24.8 % uploaded (24.6–24.8 on `master`), a frame made in a median 0.4–0.5 ms
  (0.4); *Tale of two lands* 11.2 %, 1.2 (1.2); `path-objects` 5.2 %, 0.9–1.0 (1.0). The
  status window is part of the background: no sequence changes it (M4's events and T6's clicks
  will, and a changed text draws only its box again, §4.7).

### M3 T5b2 — the date's and the kingdom's status windows, and when each shows (2026-10-29)

On `map-client/e24-m3-t5b2`, the user present (T5b2 kept whole by the user at its start; MCL-Q16
answered: the original's income kept); `master` at 23f739b (#584 merged). The original data
staged from the user's repo folder first; the sandbox restarted during the setup (its background
baseline stopped): CI's green `master` the starting point.

- **The reading** (design §4.9; map entries `status_date_create` 0x450DD0 and
  `status_kingdom_create` 0x452B80 read in full, `status_date_update` 0x4511A0,
  `status_date_vtable` 0x63BB0C, `status_date_destroy` 0x4510F0, `empty_ret` 0x5BC690,
  `status_panel_update` 0x402B90, `status_date_week_defs` 0x660BA0, `status_date_format`
  0x660C98, `status_side_format` 0x660D28): the date's window (the day's picture and "Day n", or
  on a week's first day but the game's first the week's and "Week n", the day's sound), its step
  (a frame at a view's draw 100 ms or more after the last, up to the picture's last, its
  rectangle shown at once); the kingdom's (the towns by their halls, the counts where there are
  towns, "Allies:" and "Enemies:", the flags of the players in the game by team); every view's
  draw ends with the chooser, so a timed kind's end is met at the first draw at or after it; a
  centring `hero_select` and a key's step's end build the window again.
- **Built** (design §4.9): the live part's `Status::Date`, `Status::Kingdom`, `Status::kind`,
  `Kingdom::selection`; `status::window`'s two windows (`DateArt`, `DateShows`), the layout's
  `status_date` and `status_kingdom`, the tables' `[map.status.date]`; the catalogue's keys
  `map.status.date`, `map.status.side`, `word.allies`, `word.enemies`; the gate's checks of the
  pictures, the halls' and flags' DEFs and the date's pictures' frames (exactly); the timeline's
  chooser (`Loop::choose`, `Loop::date_window`, `Start::asked`, `Asked`), `Cue::Effect`,
  `Sound::effect`, `SoundRules::effects`, the window's stream for a sound played once; a map's
  start without a selection the date's window; the harness's `Mem.status` for the kinds 1 and 2,
  `Mem.ask_status`, `Mem.status_shown`, `h3map.ask_status`; a situation's `status` and `teams`, a
  sequence's `status`; `TownBuilt::TownHall` and `CityHall`; the situations `status-date`,
  `status-week`, `status-kingdom`; the sequences `status-kingdom-end`, `status-date-end`.
- **Seen:** nothing the reading had not said: the three stills and both sequences exact at their
  first capture.
- **Checked:** `h3map.py still` then `set --only` on the three situations, exact; `frames --only`
  on both sequences, every frame exact (6, 19), the viewer's timeline as recorded; `drawn`: the 47
  situations and 13 sequences recorded before all the same; `drawn --record` (50 situations, 484
  frames of 15 sequences); unit tests (the windows, the date's pictures, the chooser over time,
  its rebuilds, the sound, the situations' and sequences' new keys); coverage 100 %; clippy; the
  window under Xvfb (`status-date-end`, `status-kingdom-end`, `status-kingdom`, `status-week`,
  `--check 10`): every frame read back exact, made as the same frame composed afresh, every
  change the headless loop's.
- **Measured** (the sandbox, `--measure 20`, alternating with `master`'s build twice each): the
  same: `walk-1` 24.8 % uploaded, a frame made in a median 0.5 ms (0.4–0.6); *Tale of two lands*
  11.2 %, 1.2–1.3 (1.2).

### M3 T6a1 — the mouse over the map and the panels (2026-10-30)

On `map-client/e24-m3-t6a`, the user present: T6 cut in two (T6a the mouse-over, T6b the clicks,
the keys during a walk and the sound without the focus), then T6a cut again once the map's hint
showed 59 cases (T6a1: the cursor by tile, the panels' hints, the map hint's frame, the special
grounds, the classes without a case, heroes, towns and monsters; T6a2: the other 58 cases);
`master` at d378a77 (#588 merged). The original data staged from the user's repo folder first;
the sandbox restarted during the setup (its background baseline stopped): CI's green `master` (CI
and Clients) the starting point.

- **The reading** (design §4.10; map entries `panel_hint_index` 0x402C50, `adventure_panel_hint`
  0x403010, `hint_tile` 0x40AFB0, `map_hint` 0x40B150, `tile_cursor_kind` 0x40E280,
  `hero_class_name` 0x4D8F70, `objnames_load` 0x5B92D0, `town_defended` 0x5BDFE0,
  `panel_hint_item` 0x65F228, `name_type_format` 0x65F3D4, `two_words_format` 0x660344,
  `adventure_hints` 0x6A56E0, `army_size_words` 0x6A5BB8, `town_type_names` 0x6A74F4,
  `object_names` 0x6A79EC; `adventure_mouse_move` 0x40E360 and `view_mouse_tile` 0x40DEB0 read
  whole; `movement_calculations_mouse` 0x4194A0, `pathfinder_run` 0x56B440's stop,
  `tile_special_ground` 0x4FCE20's end corrected (none gives 0), `help_text_load` 0x5B98B0 and
  `arraytxt_load` 0x5B9CC0's groups): the tile under the mouse by the screen's point (8 pixels
  off the drawn rows, MCL-Q18), looked at only when the view's column or row changes; the cursor
  by the tile, the selected hero and the pathfinder's cost there (its run stops at the tile:
  MCL-Q19); the panel's hint by the item under the pointer, written only when that item changes
  (the map's hints leave it: MCL-Q17); the map's hint by the tile of the object's entrance.
- **Seen:** the hit test's items are the adventure dialog's own and the chat's edit (over the
  status bar: no text); the status window's and the resource bar's are panels of their own (the
  status window's area under the whole window: "Status Window"); a disabled button (state 0x8)
  takes no mouse, the quest log's 0x20 does; a list's empty slots take none but the hero list's
  frames; water is never the coast's class 3 (`record`'s rule corrected: `hover-sail` showed the
  sail, not a landing, on water of a shore picture).
- **Built:** `h3_map_view::hover` (`Facts`, `Hover`, `Tile`, `Hint`, `Selection`, `Cursor`,
  `PanelItems`, `takes`, `panel_hint`, `parked_hint`, `days`, the tables of the original's classes
  `OWN_CASES` and `NOT_ENTERED`); the live part's `hover` and `hint`, `[hover]` read by `render
  --live`; the timeline's `Loop::moved` over the view (`over_map`) and outside it (`panel_hint`,
  `Rules::keeps_edge`), the click's move first, a step's stale reach, `Rules::column_at`,
  `Start::hover` and `levels`, `Draw::hint`; `panels::disabled`; the scene's status bar text; the
  binary's mouse-over from every level's draw lists, the selected hero by the facts
  (`sequence::selected_in`, `selected_hero`), a still's parked hint; the catalogue's families
  `map-hint`, `object`, `hero-class`, `town-type` and the map viewer's `creature`, the keys
  `map.hint.hero`, `map.hint.town`, `map.hint.monster`, `map.army.0` to `8`; h3-ui's family
  `count` (records one by one, blank ones included; additive); `masks.toml`'s `hint` and `hinted`
  (the status bar compared in the items recorded with it); the harness's `Mem.hover`,
  `Mem.node_cost`, `h3map.look_over_view` (each input tile's cost before 0, the parking's hint
  back, the column forgotten), the `[hover]` table in every live part, the per-item mask
  (`panel_rects`, `masked_for`); the situation `seq-hover`; the sequences `hover-panels`,
  `hover-map`, `hover-sail`.
- **Checked:** `frames --only` on the three sequences: every frame exact (62, 73, 23), each
  timeline as recorded (the cursor's frames 0, 2 to 7, 9 to 16 and 21 among them); `still` of
  `seq-hover` exact with its status bar; `drawn`: the 50 situations and 15 sequences recorded
  before all the same; `drawn --record` (50 situations, 642 frames of 18 sequences); unit tests
  (the facts, the tiles, the hints, the cursors on land, in a boat, without a hero and on another
  level, the panels' items, the loop over the map and the panels); coverage 100 %; clippy;
  `scripts/verify-once.sh clients` passed; the window under Xvfb (`hover-map` `--check 14` with
  seven moves of the pointer live after its inputs): 136 of 136 frames read back exact, each made
  as the same frame afresh, 141 of 141 changes the headless loop's.
- **Measured** (the sandbox, `walk-1`, `--measure 20`): a frame made in a median 0.5 ms, 24.6 %
  uploaded (T5b2's 0.5, 24.8 %).

### M3 T6a2a — the map hint's visit marks (2026-10-30)

On `map-client/e24-m3-t6a2a`, the user present: T6a2 cut by what runs the hint (the agent's
proposal, taken): T6a2a, the 35 classes whose case is inline and shows what was visited, T6a2b,
the 20 of the owners and contents (seven sub-functions); the recapture with the hint unmasked moved
to T7, which records everything again anyway. MCL-Q17 (the original's), MCL-Q18 (fixed, the
original's under `--bugs original`) and MCL-Q19 (copied in M4's engine query) answered at the
start. `master` at 819f41d (#595 merged), later 7f46f18 merged in (a conflict in `map.toml`:
both sides kept). The original data staged from the user's repo folder first; the sandbox
restarted during the setup, its baseline run again and green (`clients` verified on `master`'s
code); CI's `master` green.

- **The reading** (design §4.11; map entries `xtrainfo_load` 0x405D80, `crgen_load` 0x405D20,
  `space_string` 0x660330, `space_format` 0x66034C, `xtra_info` 0x69127C, `dwelling1_names`
  0x6914FC, `dwelling4_names` 0x691354, `map_hint` 0x40B150's summary; inferred for T6a2b:
  `creature_bank_hint` 0x40D3F0, `shrine_hint` 0x40D8D0, `tree_of_knowledge_hint` 0x40DB00,
  `witch_hut_hint` 0x40DCC0, `quest_guard_hint` 0x573040, `seer_hut_hint` 0x5741B0,
  `resource_names` 0x6A5E64, `keymaster_colour_names` 0x6A7520; structs: the game's
  `kinds_visited` u8[28] with each kind and `obelisks_visited` +0x4E3E9, the hero's
  `status_flags` bits, the player's +0xB8 and +0xBC): the 35 cases read whole, each the name, on
  the entrance the bonus (XtraInfo.txt's row by the viewing player's bit of the kind) and the
  mark (GENRLTXT 353 or 354) by the selected hero's flags or per-object fields, the screen
  player's fields, the object's word with the hero's owner, the shown or the current player, or
  what a Water Wheel or Windmill has left.
- **Built:** `hover::Visit` and `Facts::visits` (`[hover]`'s `visits`, left out none known),
  `Hint::Visits`, `VISITS`, `BONUSES`; the keys `map.hint.more` (`EXE:0x66034C`),
  `map.hint.visited`, `map.hint.not-visited` and the family `map-bonus` (XtraInfo.txt, 28);
  `Rules::origin` and `column_at` by it unless `Bugs::Original` (MCL-Q18); the situation's
  `visits` (`VisitSpec`, each of an object given, once); the harness's `h3mem.VISIT_RULES`,
  `Mem.visit_state`, `Mem.visit` (into `[hover]`), `Mem.write_visit` (the rules inverted, written
  by `h3map.stage` after the view); the situations `hover-visits-a`, `hover-visited-a`,
  `hover-visits-b`, `hover-visited-b` and their sequences, in `masks.toml`'s `hinted`.
- **Checked:** `frames --only` on the four (recorded, then again with the recorded timeline for
  the selected hero's points): 50, 50, 52 and 52 frames exact with the status bar compared, each
  timeline as recorded; the original's texts looked at (every mark and bonus as read, the second
  Water Wheel and Windmill not visited with something left); `drawn`: the 50 situations and 18
  sequences recorded before all the same, `drawn --record` (846 frames of 22 sequences); unit
  tests (the visits read, the hints of a visited object, their text, the drawn tile under the
  mouse, the situation's visits); coverage 100 %; clippy; fmt.
- **Not recorded:** a class's mark without a selected hero (read as none for the classes a hero
  keeps; the comparisons select one), the other bits of a flag of several (written as its lowest).

### M3 T6a2b — the map hint's owners and contents (2026-10-30)

On `map-client/e24-m3-t6a2b`, the user present: T6a2b kept whole (the agent's proposed cut,
T6a2b1 without the quests and T6a2b2 the Seer's Huts and Quest Guards, declined). `master` at
d2d4c70 (#598 merged). The original data staged from the user's repo folder first (the computer
went offline later); the sandbox's baseline green (`clients` verified on `master`'s code); CI's
`master` green.

- **The reading** (design §4.12; map entries: the seven hint functions read whole,
  `army_description_text` 0x40ABE0, `list_join` 0x56C960, the quests' texts 0x56D410 to 0x572670
  and lists 0x56DFA0, 0x56F5F0, 0x5704E0, 0x571580, `quest_create` 0x573240, `quest_vtables`
  0x64174C, the loaders 0x5B9880, 0x5B9310, 0x5B9340, 0x47AB30, the name tables and pointers,
  six format strings; `map_hint` 0x40B150's summary): the 20 cases, each a name and the parts its
  owner, contents or the viewer's knowledge give, by the viewing, the shown or the current player.
- **Built:** `hover::contents` (`Content`, `Quest`, `Mission`, `Guards`, `BankMark`, `Teach`, the
  `[hover]` rows `mines`, `banks`, `shrines`, `trees`, `witch_huts`, `quests`, the texts), `Hint`'s
  variants `Keymaster`, `Dwelling`, `Lighthouse`, `Resource`, `Mine`, `Bank`, `Teaches`, `Quest`
  (`Hint` now `Clone`), `Hover::contents` with the objects' owners; 34 keys and 13 families in
  `clients/text/keys.toml` (and the viewer's `FAMILIES`, `spell` among them); the situation's
  `QuestSpec` (an object's `quest`) and the visits' `hero` and `guards`; the writer's
  `Extras::quests` (`h3-map`, additive, `cross-lane`: the adventure lane's inbox); the harness's
  `Mem.contents`, `Mem.quest`, `Mem.write_content`, the Tent's mark among the visits, the Quest
  Log's shown state (`Mem.item_states`: the harness's writes of a quest's visitors left the
  button as the map's start set it, so the recomputed state differed from the screen); the
  situations `hover-owners`, `hover-banks`, `hover-teach`, `hover-quests` and their sequences, in
  `masks.toml`'s `hinted`.
- **Checked:** `frames --only` on the four (recorded, then again with the recorded timeline for
  the selected hero's points, then with the frames recorded): 50, 30, 45 and 44 frames exact with
  the status bar compared, each timeline as recorded; the original's texts looked at (every part
  as read: the Tree's price after two spaces, MCL-Q20; a kind known shows on every object of the
  kind); `drawn`: the 50 situations and 22 sequences recorded before all the same, `drawn --record`
  (1 015 frames of 26 sequences); unit tests (the contents read and refused, each hint's text,
  the 20 classes' hints, the guards merged, the writer's quests read back and refused, the
  situation's quests); coverage 100 % (`h3-map-view`), clippy, fmt.
- **Questions:** MCL-Q20 (the Tree's two spaces: the original's, recommended), MCL-Q21 (an object
  whose data names nothing: nothing shown there, refused at M4's import, recommended).

### M3 T6b1 — the buttons' press and hotkeys, the lists' clicks, Next Hero (2026-10-30)

On `map-client/e24-m3-t6b1`, the user present: T6b cut by the user into T6b1 (this), T6b2 (the
underground, the minimap's click, the status window's) and T6b3 (the keys during a walk, the
sound without the focus), the agent's three-part proposal taken. `master` at 618e20b (#602
merged). The original data staged from the user's repo folder first (the computer went offline
later); the VM restarted during the setup (the baseline's run stopped): CI's green `master` the
starting point.

- **The reading** (design §4.13; map entries `adventure_click` 0x4097E0 read whole,
  `adventure_command` 0x409A70 renamed from `end_turn_button`, `town_select` 0x417830 read whole,
  `hero_select` 0x417B20's path and order, `button_press` 0x456860, `edit_draw` 0x5BB400,
  `button_sample` 0x694DA4 and three inferred): the item messages' dispatch, the button's own loop
  while the mouse holds it (the map's idle step does not run), the hotkeys' key down and up, the
  commands, Next Hero, the lists' slots and arrows, a hero's and a town's choice.
- **Recorded** (two sequences on the new situation `seq-lists`, Red's seven heroes and six towns,
  `choose = true`): the status bar empty after every list's redraw while memory keeps the hint
  (observed live, then found: the chat's edit 38 paints its empty picture over it; MCL-Q22); the
  hint of the mouse's point after a hold's release; the map's idle step in the release's
  millisecond.
- **Built:** `timeline`'s `Action::Press`, `Release`, `Key`, `KeyUp`, `Hotkey`,
  `Unplayable::Command`, the hold, the hotkeys, the commands, the lists' slots, `hero_select` and
  `town_select`, a hero's path kept for its next choice; `Draw::selected` and `pressed`,
  `Minimap::index`; `live::Chosen` and `Live::pressed` (the panels' click frames), `at_start`'s
  choices; `keys::Command::Panel` (eleven hotkeys, `Command::release`); `sequence`'s `press`,
  `release` and the letters; the situation's `choose` (several paths with it); `h3_sdl::Key`'s K,
  Q, U, Z (the foundation, additive); the tables' `[map.sound] button`; the window's presses and
  releases apart, a hotkey's key up; the painter's selected hero by the draw, a town chosen
  putting the hero back; a sequence's path at 0 from its live part. The harness's `press` and
  `release` inputs, the hotkeys' letters, `choose_each` and the live part's `[chosen]`, the
  `hero` line without a hero.
- **Checked:** `choose-lists` (54 frames) and `press-buttons` (77) exact with the status bar
  compared, each timeline as recorded; `drawn`: every earlier situation and sequence the same,
  `drawn --record` (1 146 frames of 28 sequences); the window under Xvfb on both with live clicks
  and keys after them: 177 and 166 frames read back exact, every frame as composed afresh, every
  change as the headless loop's; unit tests (the choices replayed, the hold and what it drops,
  the hotkeys' skips, the slots' refusals, `[chosen]` read and refused, the press's frame and
  sound); coverage 100 % (`h3-map-view`), clippy, fmt.
- **Questions:** MCL-Q22 (the hint hidden by a list's redraw: the client's fix behind `--bugs
  original`, recommended).

### M3 T6b2 — the underground, the minimap's click and drag, the status window's click (2026-10-30)

On `map-client/e24-m3-t6b2`, the user present (T6b2 taken whole as the user's T6b cut left it).
`master` at 76290a7 (#605 and #606 merged). The original data staged from the user's repo folder
first; the baseline (`verify-once.sh clients`) green.

- **The reading** (design §4.14; map entries `minimap_click` 0x40A0C0 read whole, `adventure_command`
  0x409A70's command 4 and its status window's click 0x409FD8, `adventure_full_update` 0x417420's
  middle, `dialog_area_message` 0x44FF60, `dialog_add_item` 0x5FECB0, `dialog_send_to_items`
  0x5FEDE0, `status_panel_show` 0x402BC0, `message_ignored` 0x4EC560, `mouse_manager_vtable`
  0x640028, `dialog_picture_message` 0x450550 inferred): the level switched with the whole
  screen's update; the minimap's press and its own loop (every move drawn, the rest dropped, the
  idle step stopped; the point held in the area, the tile in the map); an item's press and
  release (the release, wherever, is the command of the item holding the press; the newest items
  asked first); the status window's click at the release, lost when the window is built anew.
- **Observed** (the running original's memory, a kept still): the adventure dialog's list from the
  status window's items (2000 to 2024, state 6) down to the areas, the minimap's area at state 7 (the
  press's bit its loop's release left).
- **Built:** `timeline::Loop`'s `underground`, `minimap_press` and `drag` (`dragging`), `release`
  and `status_click` (`status_held`, dropped by a window built anew), the selection's window kept
  by `choose`; `minimap::divided`, the tables' minimap `click` and `[map.status] asked`;
  `PanelItems::area`, `in_status`; `Kingdom::realm` (a live part's `realm`, `Live::at_start`'s from
  the map; `built_at_start` with the Town Hall and City Hall bits; `tables::TOWN_HALL`,
  `CITY_HALL`); the situation's `underground` (`UndergroundSpec`); `Unplayable::Panel` no longer
  the minimap's or the status window's, `Command` no longer the underground's. The harness: both
  levels explored, the screen `adventure` waited for on a map with an underground, the live part's
  `realm` (`Mem.realm`). The situation `seq-underground`, the sequences `underground`,
  `minimap-drag`, `status-click` (in `masks.toml`'s `hinted`).
- **Checked:** `underground` (21 frames), `minimap-drag` (26) and `status-click` (43) exact with the
  status bar compared, each timeline as recorded line for line (`status-click`'s first run released
  over the resource bar at y 590, in the bottom edge zone: the map scrolled at the original's held
  speed; the point moved to y 580); `drawn`: every earlier situation and sequence the same, `drawn
  --record` (90 frames of the 3 new sequences); the window under Xvfb on all three with live drags,
  clicks and keys after them: 90, 70 and 87 frames read back exact, every frame as composed
  afresh, every change as the headless loop's; unit tests (the recorded clicks replayed, the
  underground's button and key, the minimap's press, moves, clamps, dropped actions and releases
  on 36 and 108 tiles, the status window's click in every kind, a press dropped by a rebuild, the
  divisors exact, the kingdom's window at a map's start, the situation's underground); coverage
  100 % (`h3-map-view`), clippy, fmt; `check.sh`; the scoped verify with original data.
- **Questions:** MCL-Q23 (a status window's press lost to its rebuild, recorded), MCL-Q24 (a
  hotkey's release eaten by the minimap's loop, read); MCL-Q22 noted for the underground's update.

### M3 T6b3 — the keys during a walk, the input ring, Space, the sound without the focus (2026-10-30)

On `map-client/e24-m3-t6b3`, the user present (T6b3 kept whole: the agent's cut into the keys and
the sound declined). `master` at 1597854 (#609 merged). The original data staged from the user's
repo folder first; the VM restarted during the baseline (files kept, jobs gone): CI's green run on
`master` the starting point.

- **The reading** (design §4.15; map entries `adventure_key_down` 0x408C40's Space,
  `input_mouse_message` 0x4EC290 read whole, `window_procedure` 0x4F7C00's WM_ACTIVATEAPP whole,
  `sound_focus_lost` 0x599C40 read whole, new `sound_focus_regained` 0x599B90,
  `sound_digital_start` 0x5997D0 (inferred), `videos_pause` 0x5977A0, `mouse_system_cursor`
  0x50DA20, `focus_lost` 0x699609, `music_at_focus_loss` 0x699614, `effects_played` 0x6A3258):
  the ring of 63 messages (every key down and up, every move, every button); Space (a standing
  hero chosen again, then the tile it stands on visited when an entrance); a path's walk ended by
  any key's or button's down, taken and not handled; the focus's loss (the effects stopped as they
  were, the music faded by the stub thread once the lock is free) and return (the effects resumed,
  the music asked for again).
- **Observed** (the running original's memory, `seq-walk`): a standing hero's tile is class 34 with
  the entrance bit; once lifted (by Space's `hero_select`, by a step) the tile shows what lies
  under it (grass: class 0, no bit).
- **Built:** `timeline::Action::Visit` and `Up`, `Action::messages` and `ends_walk`, `QUEUED` and
  the ring (`keep_ring`), the walk ended by a press, a hotkey or Space (a click's up left behind),
  the mouse's point read at once for a press or release during a walk, `Loop::visit` and
  `reselect`, `Unplayable::Visit`; the key table's `visit` (Space) and every map key's release an
  `Up`; the sequences' `space` and every key's release an action; `h3_ui::music`'s focus
  (`Music::focus`, `Queue::focus`, `Stream::Held`); the window's sound following
  `Window::focused()` (`Player::focus`, `h3_sdl::Audio::hold`); the window's help (the panels'
  hotkeys, Space, its example rebinding pause to P: Space is the map's now). The harness: `space`
  among the keys a sequence may press. The sequences `walk-queue`, `walk-stop-key`,
  `walk-stop-press`, `walk-stop-space` (in `masks.toml`'s `hinted`).
- **Checked:** `walk-queue` (102 frames), `walk-stop-key` (10), `walk-stop-press` (9),
  `walk-stop-space` (7) exact with the status bar compared, each timeline as recorded line for line
  (also by the unit tests). The first run's `walk-stopped` (three walks ended in turn, each
  restarted by a click on the destination) recorded the original's hidden cursor over the clicked
  tile, which the viewer has no reach for after a walk (the known gap): split into the three short
  sequences, each walking from the staged tile (TRAPS). `drawn`: every earlier situation and
  sequence the same, `drawn --record` (128 frames of the 4 new sequences). The window under Xvfb
  with SDL's disk audio on `seq-walk` and `objects-flat`, the focus moved to an `xclock` window
  and back: the music faded out over 0.9 s to silence, the ambient loops held at once, both back
  at the return (the loops at once, the music faded in); unit tests (the ring, the walk's end by
  each, the queued steps, Space standing, lifted, with a path and on an entrance; the music's
  focus in every state, the queue following it; the keys' table and the sequences' actions);
  coverage 100 % (`h3-map-view`, `h3-ui`), clippy, fmt; `check.sh`; the scoped verify with
  original data.
- **Questions:** MCL-Q25 (nothing stops a held key's queued steps, Space included: MCL-Q10's
  premise; recorded), MCL-Q26 (the original's window waits for a music fade at the focus's loss;
  read, not reproduced).

### M3 T7a — the status bar's hint compared everywhere, everything recorded again (2026-10-30)

On `map-client/e24-m3-t7a`, the user present (T7 cut into T7a, T7b and T7c, the agent's
proposal; the user added the review beside the recordings: up to three concurrent reviewer
sub-agents, each given a fraction under about half a context, their findings left on a branch for
T7b). `master` at e456dd6 (#613 merged). The original data staged from the user's repo folder
first; the VM restarted twice (during the baseline, and during the sequences: files kept, jobs
gone): CI's green run on `master` the starting point, the sequences resumed one by one.

- **Built:** `masks.toml`'s `hint` and `hinted` gone (`panels` alone, empty); `h3map.py`'s
  `panel_rects()`, `masked_for(shape)` and `view_changed` without the item's id; the viewer's
  `files::load_masks(map)` (another key refused). The harness: `show_path` ends with the pointer
  over the minimap and parked again (`MINIMAP_POINT`): the staging's redraws (the selection's
  lists, MCL-Q22's; the path's) left the status bar empty with the parked button the item looked
  at last, so the parking alone showed no text in `path-objects` and `path-edge` (432 pixels each);
  over another item and back, the button's hint shows, as in every other still. `Mem.hover`'s
  towns: a town named by its map (`shipped-good-witch`'s, failing since T6a1's facts) is 65535, no
  record of `TownName.txt` (*provisional*: the viewer reads no map's town names before M4; only a
  hint over that town needs one).
- **Recorded:** `clients/capture/setup.sh` again (#614's harness, R7); the whole `set`: the 50
  situations exact with the status bar (0 differing of 480 000 pixels each; the three above after
  their fixes); every sequence (`frames --only` per id, a driver keeping a mark per id, after the
  VM's restart): the 35 exact in all 1 368 frames with the status bar, 14 timelines as recorded,
  21 changed only in `screen` boxes (the status bar's now inside them; a hint's box following the
  run's random draws: a hero's name, a Witch Hut's skill, a Shrine's spell, a quest, a bank's
  guards, an army in the status window) or a start's `points` (`scroll-limits`, `status-click`,
  `status-date-end`), each one's viewer giving the run's lines: their timelines copied in,
  `--reuse` ok; the four scrolls' `frames.txt` with the status bar's new screen (`scroll-*` at
  2 300, `scroll-limits` at 100) copied in, `--reuse` ok; `drawn --record`, then `drawn` all the
  same (50 situations, 1 368 frames).
- **Reviewed** (beside the recordings; `map-client/e24-m3-t7-review`,
  `docs/lanes/map-client/review-m3/`, not merged): six fresh reviewers on fractions (F1 the
  panels, F2 the redraw, F3 the mouse-over, F4 the input, F5 the harness, F6 what the others
  left; at most three at once, 202k to 243k reported tokens each): 42 findings, the bugs F2-1,
  F4-1, F4-2, F4-3; its README's summary is T7b's start. F5-9 (the mask comment) done here.
- **Checked:** clippy (`h3-map-view`, all targets); the scoped verify with original data (no
  failing step); `check.sh` all gates, after `the_status_window_over_time`'s two cases built from
  `status-date-end`'s and `status-kingdom-end`'s kept live parts cleared the kingdom's window
  (`realm`) the new recordings hold (the cases mean a live part without one; coverage had lost
  `timeline.rs`'s line for the window shown standing in for it).

### M3 T7b1 — the redraw's and the input's findings fixed, MCL-Q25 built (2026-10-31)

On `map-client/e24-m3-t7b1`, the user present: T7b cut with the user into T7b1 (F2 and F4, the
agent's three-way cut declined for two parts) and T7b2 (F1, F3, F5, F6); MCL-Q25 answered (a),
MCL-Q26 (a), F3-2 asked as MCL-Q27 and answered (the empty bar kept: T7b2 notes it). The original
data staged from the user's repo folder first; the VM restarted during the baseline (files kept,
jobs gone): CI's green run on `master` the starting point, the scoped verify with original data
run again before any change.

The outcome of each finding (the review's files on `map-client/e24-m3-t7-review`):

- **F2-1** (bug) fixed: `replay::Change::shows` compares the draw, the buttons and the cursor
  drawn; the window's `Frame::shows` calls it (a change of the buttons alone was taken as shown).
- **F2-2** fixed: the minimap's tiles cut to its own area (`Pen::minimap`), a pattern wider than
  it drawing nothing past it.
- **F2-3** fixed (the finding's first way): after a move a cell wholly kept compares with the last
  scene's draws all moved, a cell clear of the clip with them unmoved (a third `Buckets`), a cell
  partly kept drawn again; the minimap and its box are the same only unmoved (`same_moved`),
  `stays_put` gone. Its test: a minimap inside the clip through every move, as fresh.
- **F2-4** fixed: the composer's comment and design §4.3 say a jump nearer than the clip is a move.
- **F2-5** fixed in part: a redraw asked for uploads whatever the turn presents whole, and
  presents the frame shown again when nothing new is. Not mapped: the render device's reset (it
  loses the textures: a rebuild in `h3_sdl` for both viewers; M3's review list).
- **F2-6** fixed: a DEF's kept palette keyed by its cycles' ranges and steps.
- **F2-7** fixed in part: `damage::drop_held` drops the rectangles another holds (repeats too)
  before widening and in `tidy`; overlapping ones are not joined (M3's review list).
- **F2-8** fixed another way: a `Painter` needs the original's archives, so `damage::Held`,
  `History::since` (a frame's widening: the compositions since and its cursor's) and
  `History::changed` (a texture's upload: both cursors) moved to the library, which the painter
  and the window call; `compose::tests::widened_as_fresh` drives the composer, the history and
  `Widening` over three buffers reused out of order with a moving stand-in cursor and a texture
  updated by `changed`, each frame the fresh one widened (dropping the old cursor's rectangle
  fails it).
- **F4-1** (bug) fixed: the window's `HeldKeys` keeps each key's command from its down (a repeat
  that gives none keeps the one before); its up releases that one, whatever Ctrl did since.
- **F4-2** (bug) fixed: a point off the frame (the window's at the focus's loss) leaves a
  button's hold or the minimap's drag as it is (`Rules::on_frame`).
- **F4-3** (bug) fixed: `holding` and `drag` decide by the message's own point and set the pointer
  only for a fresh one; the drag's last point is its own (`dragging: Option<[i32; 2]>`).
- **F4-4** fixed: a hotkey's up acts only on a button still shown pressed.
- **F4-5** fixed: `Action::RightPress` and `RightRelease` (the window's for the right button, before
  the key table), a message each; the down ends a path's walk, the up ends a button's hold (a
  click when pressed); nothing else on the map in M3 (the help texts: M4).
- **F4-6** fixed: next hero to a hero whose choice the live part lacks is refused in a recording.
- **F4-7** fixed: a click's point clamped to the frame as a move's.
- **F4-8** fixed: `h3_sdl::Window::focus_changes` (additive) gives a batch's focus changes in
  order; the window handles each, a loss and a regain in one batch both.
- **F4-9** fixed: `key_step` calls `step_objects`.
- **MCL-Q25** built: `Action::Repeat` (the window's for a direction key's repeat) is a step, but
  while a key's step waits it is dropped, with `--bugs original` queued as a step (`wait_in`, also
  for the actions a walk's end queues again). No recording has a repeat (the harness turns the
  display's off), so no capture changes.

- **Checked:** each new test fails with its fix undone (F2-2, F2-3, F2-6, F2-8, F4-3, F4-4,
  MCL-Q25); clippy (`h3-map-view`, `h3-sdl`, all targets); `drawn` all the same (50 situations,
  every sequence; no capture needed: every change is live-only or reaches no recorded frame);
  `check.sh` all gates; the scoped verify with original data.

### M3 T7b2 — the panels', the mouse-over's, the harness's and the rest's findings fixed (2026-10-31)

On `map-client/e24-m3-t7b2`, the user present: the agent's cut by area (F1 and F6, then F3 and
F5) declined, all 24 findings in one task. The original data staged from the user's repo folder
first; the setup's baseline ran while the first fixes were made in the same tree (its coverage
step built a mix, so its failure says nothing): CI's green run on `master`'s client code (#619)
the starting point.

The outcome of each finding (the review's files on `map-client/e24-m3-t7-review`):

- **F1-1** fixed: a text item's clip and a label's colour saturate (`h3_ui::paint::draw`), as the
  picture's arm and `reach`; a test draws both at the integers' end.
- **F1-2** done by T7b1 (F2-2: `Pen::minimap` cuts the tiles to the area).
- **F1-3** fixed as docs: `finish`'s comment says the points drawn are the recording's (the
  original's refused step zeroes them in the step's own call chain, 0x480864); nothing sets them,
  since a message ending the walk before the red step would leave the original's points.
- **F1-4** fixed: `Loop::minimap` dropped; `draw` keeps `drawn.minimap`, which only
  `draw_minimap` writes.
- **F1-5** fixed: `choose` compares the kind asked first and clones only the window built;
  `Painter::view` compares the draw and buttons by reference, cloning only on a miss.
- **F3-1** fixed: a quest's hero -1 reads as none (`Mission::DefeatHero(Option<(u8, bool)>)`,
  `BeHero` likewise), its text empty there (MCL-Q21 (a)); an own flag other than 0 or 1 refused.
- **F3-2** noted (MCL-Q27 (a)): `hover.rs`'s case 54 off its entrance, its test, design §4.10.
- **F3-3** fixed: `h3_ui::text::Catalogue::known` (additive: none for a key no pack has); the
  data-named lookups (a hero, a class, a town's name and type, a creature, an artifact, a spell,
  a skill) give the empty text past the catalogue's family, as MCL-Q21 (a).
- **F3-4**, **F3-5**, **F3-6** fixed: §4.10's 55 cases; the Arena test's comment; `cursor.rs`'s
  module doc (only `Cursor::new`'s default is provisional).
- **F5-1** fixed: a run's start releases the hotkeys and the button too, before its staging
  (xdotool's failure harmless with no display yet); the runbook's line.
- **F5-2** fixed: `write_visit` reads and writes the word of a hero or boat on the entrance in
  its record (+0x14), as `hover` reads it. No situation puts one there (not exercised).
- **F5-3** fixed: `write_content`'s hero write refused outside the spell's 0..70 or the skill's
  0..28, as `contents` reads them.
- **F5-4** fixed: `choose_each` waits for the status window's name item (2002,
  `Mem.status_name`; `Mem.town_name`) to name the hero or town chosen.
- **F5-5** fixed: a pointer in a scroll zone (within 16 pixels of the screen's left or top side)
  is left out of `look_over_view`.
- **F5-6** fixed: the parser's owner check takes `HeroSpec::written` (the status window's facts
  are Red's alone); `lists` writes an army or garrison only when not empty, as the parser counts
  it. No committed situation had either.
- **F5-7** fixed: the button paired as the keys in both parsers (`sequence_inputs`,
  `Sequence::actions`): no release without a press, no press or click while it is held.
- **F5-8** fixed: `town_defended`'s summary (+0x10 the visiting hero, +0xC the garrison hero, read
  again in the executable), the harness's `hover` (the constants, `TOWN_VISITING_HERO`) and
  design §4.10's cursor line; the behaviour unchanged.
- **F5-9** done by T7a.
- **F6-1** fixed: `Dialog::shadow_draws` saturates; `h3_render::blit::darken` cuts its rectangle
  to the screen before the loop (the same pixels; a crafted `i32::MAX` shadow no longer loops).
- **F6-2** declined: the tables' load refuses a `frames` that is not one a picture already
  (`StatusArt::check`, tested); the gate's zip noted so.
- **F6-3** fixed: `every_pixel_a_drawer_changes_lies_inside_its_reach` draws each kind (texts with
  negative spaces first, inside and last, lines wider than the box, every alignment, both
  profiles; a mirrored button whose frame fills part of its canvas; pictures; a darkening) on a
  sentinel screen; it fails with the overhangs left out of `text_reach`.
- **F6-4** fixed: the gate asks `cprsmall.def` for `stack + status::ARMY_CREATURES` frames (149:
  creatures 0 to 148; the installation's DEF holds a blank, an unknown and those 149, 151 frames).
- **F6-5** fixed as docs: `Report::skipped` and the summary's "times a later one shown instead".

- **Checked:** the new reach test fails with the overhangs undone; `drawn` all the same (50
  situations, every sequence); under Wine `hover-teach` (the visits with a hero's own, F5-3) and
  `choose-lists` (F5-4, F5-1) every frame exact (45, 54), only the new game's draws in `screen`
  and `points` lines changed (TRAPS: no regression, the committed recordings kept); the first
  `choose-lists` run timed out once in the look's last wait (the bar read "Adventure Options"),
  the second passed; `check.sh` all gates; the scoped verify with original data.

### M3 T7c1 — the review list, the questions, the hand test (2026-10-31)

On `map-client/e24-m3-t7c1`, the user at the machine. `master` at a0bed50 (#623 merged; CI's
Clients green on it). The original data staged from the user's repo folder first. T7c cut with the
user at the start: T7c1 (this), T7c2 (the answers built, M3 archived), M4's refinement its own task
(M4 T0). The setup's baseline (`verify-once.sh clients`) died in its coverage step when the agent
paused it to free the CPUs for the Windows build (a `kill -STOP` by pattern): CI's green `master`
the starting point.

- **The review list** (M3's 27 items, summarised in the chat) **accepted** as built, the two
  schemes of the `hero` family's keys (by type here, by the asset tables' name in the battle
  client) kept until a translation needs one.
- **The questions:** MCL-Q13, Q20 kept as the original's; MCL-Q14 (a cast recorded in M4 first),
  Q15 (looked at again with M4's amounts), Q21 ((a) now, the import's refusal M4's) as
  recommended; MCL-Q23 kept as the original's (the agent had recommended the fix); MCL-Q22 and
  Q24 the client's fixes, the original's behind `--bugs original`, built in T7c2. The entries in
  the lane's `QUESTIONS-ANSWERED.md`; M4's follow-ups in the epic's "Later".
- **The client lane's directive** (CLT-Q28 at E17 M5 T8c: a text the original sets off its centre
  is centred in the client's profile) recorded in the epic's "Decisions"; none found in the map
  client yet.
- **Found preparing the hand test:** every command of `h3-map-view` read the key catalogue at
  `clients/text/keys.toml` by the build's own path (`env!("CARGO_MANIFEST_DIR")`), which a
  Windows build on the user's machine never has (none had run there since T5a brought the
  catalogue): the window would have stopped at its start. Fixed (*provisional*): `texts::catalogue`
  reads `keys_beside(--map)`, `<map>/../text/keys.toml`, as `clients/map` and `clients/text` lie
  in a checkout; the gate's usage says so; a test of the place; `default_keys` the tests' alone.
- **The hand test** (`clients/out/win/map-m3.cmd`, the `h3-map-view.exe` of this branch,
  `x86_64-pc-windows-gnu`; master's map data and catalogue in `out/win/m3/`, the user's checkout
  being behind): four runs, each a recording replayed and then the user's: `hover-map` (the panels,
  the cursor and the hints over the map and the panels, MCL-Q18's fix at a tile's edge),
  `choose-lists` (the lists' choices and arrows, Next Hero, the other buttons and their hotkeys,
  the status window's click), `underground` (the level's button and U, the minimap's click and
  drag), `walk-queue` (a held key stopping at most a step after its release, MCL-Q25's answer;
  four taps four steps; Alt+Tab away and back). The user: **all as described**. Every run exit 0;
  the focus's loss and return held and resumed the sound (runs 1 and 4). The window on Windows
  (direct3d11, WASAPI 48 kHz): a frame made in a median 0.0 ms (p95 0.6 to 0.9, max 7.6), shown a
  median 0.5 to 1.8 ms after its time (p95 2.2 to 3.1; max 61.2 in run 1, its focus lost once);
  19 to 54 % of the frames' pixels uploaded. Run 4's cues: one step sound in 50 steps, a path's
  walk (0x407D03 starts it; a key's step plays none, as built in M2 T5).
- **Checked:** `cargo clippy -p h3-map-view --all-targets -D warnings`, `cargo fmt --check`, the
  binary's `texts` tests. No change to anything drawn or played: the catalogue read is the same
  file at the default `--map`.

### M3 T7c2 — MCL-Q22's, Q24's and Q28's fixes built, M3 archived (2026-10-31)

On `map-client/e24-m3-t7c2`, the user present. `master` at e6dbcbc (#625 merged; CI's Clients
green on it). The original data staged from the user's repo folder first (six calls, 57 files);
the setup's baseline (`verify-once.sh clients`, the full `check.sh`) passed every gate before any
change, the work meanwhile in a `git worktree` with nothing built there.

- **MCL-Q22 (b)** (*provisional*): `Loop::draw_lists` (every dialog item drawn: a list's choice,
  arrow, Next Hero, a lift that centres, a walk's end unable to move, the underground's button and
  key, a town's full update) empties the status bar in the original's profile only; in the
  client's it marks the hint to draw again, and `Loop::hint_again`, at the end of each turn of
  `Loop::run` (after its actions: what the screen shows then), shows off the view the hint of the
  panel's item under the mouse now (a slot names the hero or town it holds now), that item the one
  looked at last; over the view the tile's hint memory holds (the mouse-over gives a new one only
  when the mouse's column or row changes, as after a scroll). A hint shown meanwhile clears the
  mark.
- **MCL-Q28, asked and answered in this task** (the user: (b), as recommended): reading the
  button's loop for MCL-Q24's fix (`def_button_message` 0x45641C: every message but the buttons'
  up handed to the mouse manager, 0x456341, and dropped unless a move, 0x45634D) showed the same
  defect while the mouse holds a panel button: another button's hotkey released then leaves that
  button pressed, its command lost. The entry in the lane's `QUESTIONS-ANSWERED.md`, design §4.13.
- **MCL-Q24 (b) and MCL-Q28 (b)** (*provisional*): `Loop::holding` takes a hotkey's key up first,
  for the minimap's drag and a button's hold alike: dropped in the original's profile; in the
  client's `Loop::key_released` releases the button its key pressed, drawn and shown, without its
  command or a sound; the button the mouse holds is left to its hold (its own key's up does not
  release it under the mouse). The window's usage names the three under `--bugs`.
- **M3 archived:** the epic's M3 section (scope, decisions, tasks T0 to T7c2, exit) and its
  review list moved verbatim to `E24-archive.md` (a script; the two files together hold every
  line of the old one but the edited ones), a stub left with the decisions that bind later
  milestones; the queue's M3 history and T7c2's entry to `QUEUE-ARCHIVE.md`; the HANDOFF's M3
  lines trimmed. T7c1's hand test left no finding.
- **Checked:** the two new tests (`the_client_draws_the_hint_a_redraw_hid_again`,
  `a_hotkeys_up_during_a_hold_releases_its_button_in_the_client`) fail on the old code and pass;
  every earlier test unchanged (the recorded choices, holds and the underground in the original's
  profile); `h3map.py drawn`: the 50 situations and the 35 sequences the same, every timeline as
  recorded; `check.sh`: every gate (coverage 100 % once a key up of a key that pressed nothing
  during a hold was tested); the window under Xvfb in both profiles (`underground` replayed, then
  live: the underground's button clicked, K held through a minimap drag and through the button's
  hold, `--check 30`): the client's status bar shows the button's hint again after the click and
  releases the kingdom overview's button at K's up in both holds, the original's leaves the bar
  empty and the button pressed (screenshots looked at); 179 and 176 frames read back exact, every
  frame as composed afresh, every change as the headless loop's; `verify --scope map-client
  --with-original-data` no failing step; `check-docs`, `check-lane`, `check-exe-map`.

### M4 T0 — M4 refined with the user (2026-10-31)

On `map-client/e24-m4-t0`, the user present in the chat; `master` at #626 (T7c2 merged, CI's
`Clients` run green). Docs only; no original data staged (nothing built or captured).

- **Read before proposing:** the viewer's live input (`h3_map_view::live::Live`: per object
  `standing`, `drawn`, `owner`, `phase`; the heroes, boats, explored tiles, the kingdom, the
  hover's facts); ADR-0020 point 8 (the viewer plays only `h3-protocol` messages; an engine replay
  becomes a record by the session in-process) and how `h3-view` follows it (`render` runs a
  scenario in `h3-protocol`'s session as a spectator, its record a `.jsonl` of wire messages);
  `h3-protocol` (no `h3-game` dependency: no whole game on the wire; E16 M5 "the whole game" not
  refined, the server lane idle); `h3_game::GameObservation`, `h3_adventure::ObjectView` (no
  identity: the adventure lane's T12b note, its archive's note (3) of 2026-10-23 leaving it to
  M4) and `GameEvent`'s variants; E08 design §12, §14 (increment 9's views, no identity among
  them); the golden adventure replays (`fixtures/replays/adventure/`, 32: commands with the
  engine's paths and the battles' commands); the import's census (`docs/formats/h3m.md`: 8 of
  160 shipped maps whole, 128 stop at a hero generated at a main town; the user was first told "a
  hero in a town", which is the reason of 3 others); `h3-game` sets a battle's background and
  colours (CLT-Q4, CLT-Q6); the treasury's cap `MAX_GAIN` 2³¹ − 1; design §4.8's kinds 5 to 7.
- **Reported to the user first:** M4's input does not exist yet (the protocol carries no whole
  game, the observation no object identity), and the engine plays 8 of the 160 shipped maps
  whole.
- **The proposal** (Part A, the original's reactions to a game's events from its memory, needing
  no engine; Part B, the engine's records; Part C, the battles; the tasks T1 to T6; the exit) and
  four questions; **the user took every recommendation:** the input through the protocol with the
  server lane first (the start of E16 M5, refined with the user there), not the engine read
  directly (ADR-0020 amended) nor this lane writing the protocol; one player's view (the other
  players' turns as the original shows a computer player's), a spectator's later; the message
  boxes only, the screens M5's; the battles in the same window by the battle client's viewer (a
  library entry from the client lane). From M3's answers: MCL-Q15 (b) (the treasury passes the
  bar's seven digits), MCL-Q21 (b) (the adventure lane's import and the viewer's load), MCL-Q14 (a
  cast recorded in T2).
- **Docs:** M4's section in the epic (scope, decisions, exit; "Later" keeps M5, the spectator's
  view, a saved game's draw lists, towns), the risks (the engine's lag; the engine's games against
  the original); the queue's T1 to T6, its inbox's increment 9 note (now T3's scope) and the
  client lane's informational note moved to `QUEUE-ARCHIVE.md`; MCL-Q15's answered entry notes
  the cap; inbox notes for the server lane (the whole game's wire, session and record, with what
  the viewer reads), the adventure lane (the object identity, random objects' drawn types,
  MCL-Q21's refusals) and the client lane (the battle as a library entry).
- **Checked:** `cargo xtask check-docs`, `check-lane --lane map-client`; nothing else changed.

### M4 T1a — the turn's end and the next day (2026-11-16 to 17)

On `map-client/e24-m4-t1a`, the user present at the start (the cut), then away; `master` at #976.
Original data staged (63 files, SHA-256 checked against the PC's); the setup's baseline stopped
at its coverage step (the client lane's trap), CI's green `master` standing in. The server behind
the session stopped twice and the VM restarted three times; the work went on from the pushed
commits each time.

- **The cut** (the user, at the start): T1 in two, T1a this (End Turn and its question, the
  turn's end, the next day, the week's and the month's messages) and T1b (a computer player's
  moves in sight). The status window's kind 5 moved to T1b with its cursor and hint: on the
  virtual clock a computer's turn out of sight takes no time (below), so only T1b's walks show it.
- **The inbox first:** the map's key table refuses a `Shift+` chord (`keys::refuse_shift`: the map
  reads no Shift, as the original's; the client lane's E27 M2 T7b note); `tests/purity.rs` took
  `h3-view`'s four fixes (E17 M6 T6a, T7b); design §3's keys bullet names the Ctrl chords; the four
  entries to `QUEUE-ARCHIVE.md`.
- **Read** (design §5.1): End Turn (0x409D12), `next_player_turn` and `player_turn_start` in full,
  a computer player's start and `covered_view_redraw`, kind 5's panel and step, the week's and the
  month's messages (ArrayTxt 128 to 135, the names 32 to 41 and 43 to 57, the draws 0x4C87A6 and
  0x4C8FCC). Two `docs/exe/map.toml` sites (`week_name_draw`, `month_name_draw`).
- **Seen first:** with no town Red's first End Turn shows "you have lost your last town" with
  Red's flag (the scenario's tests), its text starting with an empty player name in the harness's
  game: a box with a picture, T2's (its queue entry says so); the situations give Red a town.
- **The harness** (common, additive): `xdd.dll` forces `week_name` and `month_name`; a sequence's
  `move_reminder` and `rolls`, Enter and Escape; the timeline's `date` and `player` lines; the live
  part's `[[turn]]` tables (`Mem.calendar`). Seven sequences recorded, each exact at 0:
  `turn-day`, `turn-question`, `turn-week`, `turn-week-creature`, `turn-month`,
  `turn-month-creature`, `turn-plague` (situations `seq-turn`, `seq-turn-week`,
  `seq-turn-month`). The game's disk check stopped four runs at `main_menu` (free bytes 3.99 GiB
  modulo 4 GiB): a ballast file.
- **Built** (a worker; design §5.1's *Built*): the common crate's `h3_ui::message` (the client
  lane's `h3_view::message` trimmed to styles 1 and 2 without pictures; the client lane's inbox
  told), the live part's `Turn`, the module `turn`, End Turn's question and the turn's change in
  the timeline, a box's own loop, the box drawn by the binary, the key table's `Message` context;
  seven catalogue keys and two families.
- **Checked:** every frame of the seven sequences exact (22, 14, 16, 12, 9, 9, 9), the viewer's
  timelines the recorded ones; `drawn` the 50 situations and 35 earlier sequences the same, then
  `--record` (the seven joined); `RUST_TEST_THREADS=1 check.sh --no-coverage` all gates; coverage
  outside `check.sh` 100 % lines, regions and functions; the window under Xvfb on `turn-question`
  and `turn-week` (`--check 10`): every frame read back exact, live E, Escape and Enter as the
  original's. No sound played (no audio device).
- **Questions:** MCL-Q29 (the Deity's week names the Imp's growth for the Familiar), MCL-Q30 (the
  new day's sound started twice). Recorded, not questions: a redraw's deadline already passed at
  the turn's change moves on without an objects' step; the cursor drawn at the mouse at a box's
  opening and close (E17 design §9.9's even x).

### M4 T1b1 — a computer player's moves in sight, read and recorded (2026-11-17)

On `map-client/e24-m4-t1b1`, the user present at the start (the cut); `master` at #985. Original
data staged from the PC (63 files, SHA-256 checked against the PC's: one digest of the sorted
lists, `9c890f81…`; the PC's own clone behind GitHub's, nothing uncommitted); the setup's
baseline stopped at clippy: `master`'s Clients check was red.

- **`master`'s clients build:** #982 (the client lane's E27 M3 T1a) gave `h3_ui::ui::Button` a
  `caption` and #983 (T1a) built the message boxes' buttons without it; each was green alone.
  Fixed apart, first: #986 (`map-client/fix-message-caption`, `caption: None`); the client lane's
  #988 made the same fix, so #986 merged empty.
- **The cut** (the user, at the start): T1b in two, T1b1 the reading and the recordings (the
  sequences held in `clients/map/to-record.toml` until the viewer plays them), T1b2 the viewer.
- **Read** (design §5.2): `ai_player_turn` to its heroes' turns, `ai_hero_turn` to its walk,
  `ai_hero_walk` and `ai_hero_step` around their calls, `step_seen` and `tile_or_neighbour_seen`
  whole, kind 5's panel and its step whole, `hero_select`'s computer branch. `docs/exe/map.toml`:
  five new entries (`tile_or_neighbour_seen`, `ai_hero_walk`, `step_seen`,
  `status_other_turn_step`, `ai_hero_turn`), `ai_boat_planning_caller` renamed `ai_hero_step`,
  `status_other_turn_create`, `ai_player_turn`, `option_blackout_computer` and `view_shown`
  amended; the addresses §5.1 cited without an entry (the town lane's #987 had mapped them from
  §5.1 meanwhile, mostly `inferred`) now as read: `status_bar_text_set`, `town_deselect`,
  `hero_deselect`, `path_cancel` (their `hero_list_redraw`), `ai_hero_turn` (`ai_turn_part_d`),
  `ai_hero_replan` (`ai_turn_part_a`), `scenario_conditions_test`, `crt_toupper`,
  `status_other_turn_vtable`; `ai_next_hero` and five planning sites of kind 5's step new. §5.1 corrected: kind 5's items are 2190 to 2193 (T1a's reading
  had 2000 to 2003), +0x68 the sand's frame and +0x70 the glass's (swapped there), and 0x419300
  clears a shown path (T1a had it the hero list's update).
- **Seen first:** the game seeds its generator once, with the running clock's time, so two runs of
  a computer's turn walked differently; with a Tavern in its town the computer hired a hero and
  walked it into sight from the fog (a hero the live part does not hold): the situation gives Blue
  no town.
- **The harness** (common, additive): `xdd.dll`'s `seed` (its 8.: the main thread's C library
  seed at its next clock read; the ack's sixteenth field), `Game.seed`, `to_scenario`'s `seed`;
  a sequence's `computer_speed`, `blackout` and `seed`; kind 5's items read (`Mem.other_turn`,
  `Mem.status`), the timeline's `other` lines, kind 5 and the heroes and boats in the `[[turn]]`
  tables; `frames --sequences <file> --no-check`; the `player` line fixed (an integer
  `map_timeline` could not print: T1a's sequences never changed the player across a
  millisecond). The client, adventure and town lanes' inboxes told (rebuild `xdd.dll`).
- **Recorded** (each twice, the second run `ok`, the frame at 0 exact): `ai-walk` (151 timeline
  lines, 34 screens), `ai-walk-slow` (187, 50), `ai-walk-fast` and `ai-walk-blackout` (18, 5
  each, alike), `ai-walk-fog` (131, 30); their timelines and live parts in `map/timelines/`;
  situations `seq-turn-ai`, `seq-turn-ai-fog`. The live parts kept from a later run each (`ok`),
  once the `[[turn]]` tables held the heroes.
- **Questions:** MCL-Q31 (the view follows a computer's hero still under the fog). MCL-Q30's
  sound run moves to T1b2.

### M4 T1b2 — a computer player's moves in sight, played by the viewer (2026-11-17)

On `map-client/e24-m4-t1b2`, the user present at the start; `master` at #991. Original data
staged from the PC (63 files, SHA-256 checked against the PC's: one digest of the sorted hashes,
`9421f826…`); the setup's baseline stopped at its coverage step by a watcher (fmt, clippy, doc
tests and deny passed), CI's green `master` standing in.

- **No cut** (the user, at the start): T1b2 whole, though a seam was offered (the five sequences
  first, speed 4's window and the sound run after).
- **Read** (design §5.2's "The computer's screen"; `docs/exe/map.toml`): `covered_view_redraw`
  0x4CA5B0 whole (now `read`), the new `ai_shield_draw` 0x4CA780 (the shield, `aishield.pcx`, over
  the minimap's area), `minimap_draw`'s computer branch, `status_panel_set` 0x402C10 whole, the
  adventure manager's +0x38C (`ai_shield_shown`, `structs.toml`). Kind 5's value corrected: 1 000
  plus the **sum** of a computer's heroes' `hero_max_movement`, set for every colour when the
  panel is built (§5.1, §5.2; the fog's first unseen steps leave it at the full value).
- **The harness** (common, additive): kind 5's `[[turn]]` status keeps `value` (`Mem.status`:
  the dialog's panel +0x98, its +0x44 per colour); `h3map.py`'s sequences take `sound` (a sound
  run, its samples in `sounds.txt`). The five `ai-*` recorded again here (all `ok`), then
  `ai-walk`, `ai-walk-slow` and `ai-walk-fog` once more for `value`.
- **MCL-Q30's sound run** (`turn-day` with `sound = true`, a scratch sequence): `newday.wav`
  starts twice at End Turn's millisecond on two channels (handles 7 and 8, 3 739 ms each): the new
  day sounds doubled. MCL-Q30 updated, recommended (b); open for the user.
- **Built** (a worker; design §5.2's *Built*): kind 5's panel and step, the computer's screen (the
  four buttons disabled, the shield, the lists the viewer's), a computer's turn in the loop (the
  busy cursor, its hero lifted and the view centred when seen, its walk from the recording's
  `hero` lines at Computer Walk Speed 2 to 4, unseen steps taking no time, Blackout, the cursor
  hidden from the first seen step, the next day with the `[[turn]]` table's heroes placed),
  MCL-Q31 as the original's; speed 4's single draws held 20 ms each on the window's clock
  (`HOLD_MS`, *provisional*, M4's review list, which starts here). Recorded, not read: the
  redraws a computer's timed walk missed are caught up before the next day's whole redraw
  (`ai-walk` at 1660). Not built (T4's entry has them): several heroes or a boat in one
  computer's turn, its music and walk sounds, a hot seat's kind 5 texts; no recording can show
  speed 4's held draws (no steps inside End Turn's millisecond).
- **Checked:** the five sequences moved into `sequences.toml` (`to-record.toml` removed), every
  frame exact (34, 50, 5, 5, 30) and the viewer's timelines the recorded ones, checked again by
  the main session after the worker; `drawn` the rest the same, then `--record`;
  `RUST_TEST_THREADS=1 check.sh --no-coverage` all gates; coverage outside `check.sh` 100 %
  lines, regions and functions; the window under Xvfb (`--check 10`) on `ai-walk-fast`, `ai-walk`
  and a scratch speed-4 walk (its 15 held draws read back exact).

### M4 T2a1 — the finds told, read and recorded (2026-11-17)

On `map-client/e24-m4-t2a`, the user present at the start (the cut); `master` at #994. Original
data staged from the PC before it went offline (63 files, SHA-256 checked against the PC's: one
digest of the sorted hashes, `9421f826…`; the PC's own clone behind GitHub's, nothing
uncommitted); the setup's baseline stopped at its coverage step by a watcher (fmt, clippy, doc
tests and deny passed), CI's green `master` standing in.

- **The cut** (the user, at the start; the recommended one): T2 in four by dialog family, T2a the
  finds told, T2b the questions and choices, T2c a level-up's choice, T2d the map changed by a
  visit; this session T2a, cut again into T2a1 (the reading and the recordings, the sequences in
  `clients/map/to-record.toml`) and T2a2 (the viewer) once the reading showed its size (the
  fallback the user agreed with the cut).
- **Read** (design §5.3; `docs/exe/map.toml`): `message_box_picture` whole (now `read`: every
  picture type's DEF, frame and caption); `status_resource_set` 0x416060 and `status_text_set`
  0x416210 new (kind 6 asked for 5 000 ms by a pile's visit alone; kind 7 by a network game's
  messages alone: not M4's); `status_resource_create` (now `read`, its items); the finds' boxes of
  the Windmill, the Learning Stone, the Star Axis, the Temple, the Witch Hut, the Shrines, the
  Magic Well, a sign, the plain artifact pickup and a pile (its removal: the pickup's sound and
  the view's middle dissolved in 65 ms steps); `grant_rewards`' boxes located; new entries
  `plus_count_name_format`, `shrine_text_format`, `artifact_pickup_texts`, `skill_level_names`.
  The turn's two boxes from the client lane's reading (E27 design §5.1) and §5.1: both begin
  with the player's name, empty in a single scenario, which CLT-Q100 answered (b) for the shell;
  T2a2 names the player the same way (no new question).
- **The harness** (common, additive): the live part's `[[message]]` (`Mem.message_box`) and
  `[[found]]` (`Mem.found`) tables; the situations' bodies in this lane's `situation.rs` (a pile's
  `amount`, a Shrine's `spell`, a Witch Hut's `skills`, an event's `rewards`, a hero's
  `experience`; the bodied objects among the writer's placed objects, refused with quests), a
  unit test.
- **Recorded** (each twice, the second run `ok`, the frame at 0 exact): `find-windmill`,
  `find-stone`, `find-axis`, `find-temple`, `find-witch-hut`, `find-shrine`, `find-sign` (30
  timeline lines, 7 screens each), `find-pile` (98, 37), `find-artifact` (32, 13), `find-event`
  (40, 11: three boxes), `turn-no-town` (36, 10: GENRLTXT 7, then ArrayTxt 128); situations
  `seq-find-*` and `seq-turn-no-town`; their timelines and live parts in `map/timelines/`. Every
  box's texts and pictures as read; the pile's gold as the format stores it (700 is 70 000 gold).
- **Checked:** `h3-map-view`'s tests at 100 % lines, regions and functions (`cargo llvm-cov -p
  h3-map-view`, `H3_INSTALL` empty); the clients workspace's fmt and clippy clean; `cargo xtask
  verify --scope map-client --summary --with-original-data` no failing step; `scripts/exe.py
  check` and `cited` clean. No existing situation's map changes (every new branch needs a body or
  an experience), so `drawn` was not run. PR #996.

### M4 T2a2 — the finds told, played by the viewer (2026-11-17 to 18)

On `map-client/e24-m4-t2a2`, the user present at the start; `master` at #996. Original data
staged from the PC before it went offline (62 files, SHA-256 checked against the PC's: one digest
of the sorted hash lines, `29a3470f…`; the PC's own clone behind GitHub's, nothing uncommitted;
`MP3DEC.ASI` now in the PC's `original-data/install`, 39 non-campaign MP3s); the setup's baseline
stopped at its coverage step by a watcher (fmt, clippy, doc tests and deny passed), CI's green
`master` standing in. The server behind the session stopped once and the VM restarted once (part
B's worker resumed from its checkpoint commit).

- **No cut** (the user, at the start): T2a2 whole, though a seam was offered (the seven single
  boxes and the turn's two first, the pile, the artifact and the event after). Built in two parts
  by two workers one after the other, the main session reading `grant_rewards` between them.
- **Recorded again here** (the original's screens are local, never in git): the eleven
  sequences, each `ok` against its kept timeline and exact at 0 (`frames --sequences
  map/to-record.toml --no-check`, one id per run, a ballast file for the game's disk check).
- **Read** (design §5.3, "The rewards' boxes"; `docs/exe/map.toml`: `grant_rewards` amended,
  `reward_box_flush_full` 0x4A0B00 new): which parts share a box (experience to luck, then the
  resources, the artifacts, the spells, the creatures; a box at eight), each part's picture and
  line, the map's text in the first box alone; MCL-Q32 (two spells in the first box replace the
  map's text). Two corrections of T2a1's reading: a Witch Hut's known skill is ADVEVENT 172 (173
  the eight skills); the sign, the Stone, the Axis and the Shrine move the hero onto their
  entrance (only the pile and the artifact leave him beside them).
- **Built** (design §5.3's *Built (M4 T2a2)*, parts A and B): `h3_ui::message`'s pictures (the
  client lane's slots, rows and layout copied with their tests, additive), the module `finds`
  (every picture type of `message_box_picture` but a town's building; each find's box from its
  facts through the catalogue), the timeline's finds and the turn's two boxes (the player named
  by colour, CLT-Q100 (b); empty with `--bugs original`), kind 6's panel for 5 000 ms, an object
  taken (the index built again, the dissolve by `h3_render::Screen::dissolve_rect`, the pickup's
  sound), the artifact's box and the event's three; the catalogue's `map.find.*` keys, the
  tables' `[map.finds]`, `[map.turn]` and `artifact_skills`. Its *provisional* choices: M4's
  review list.
- **Checked:** the eleven moved into `sequences.toml` (`to-record.toml` removed), every frame
  exact by `frames --only <id> --reuse` (`find-windmill`, `-stone`, `-axis`, `-temple`,
  `-witch-hut`, `-shrine`, `-sign` 7 each, `turn-no-town` 10, `find-pile` 37, `find-artifact` 13,
  `find-event` 11) and the viewer's timelines the recorded ones; `drawn` the 50 situations and 47
  earlier sequences the same, then `--record`; `RUST_TEST_THREADS=1 check.sh --no-coverage` all
  gates; coverage of `h3-map-view` and `h3-ui` 100 % lines, regions and functions; the window
  under Xvfb (`--check 10`) on `find-pile` (62 frames, the dissolve's among them) and
  `find-windmill` (54), every frame read back exact; `scripts/exe.py check` and `cited` clean.
- **Cross-lane** (the user's consent at the end; the label `cross-lane`): `check.sh`'s tests
  failed on the client lane's `h3-battle-client` test `the_committed_catalogue_parses`, which
  lists the common catalogue's tables: `"ADVEVENT"` added to it (one word and its comment), the
  client lane's inbox told.

### M4 T2b1 — the questions and choices, read and recorded (2026-11-18)

On `map-client/e24-m4-t2b1`, the user present at the start (the cut); `master` at #1003. Original
data staged from the PC before it went offline (63 files, SHA-256 checked against the PC's: one
digest of the sorted hash lines, `28fcf718…`; the PC's own clone behind GitHub's, nothing
uncommitted); the setup's baseline stopped at its coverage step by a watcher (fmt, clippy, doc
tests and deny passed), CI's green `master` standing in.

- **The cut** (the user, at the start): T2b in two as T2a was, T2b1 the reading and the
  recordings, T2b2 the viewer (recommended was three: the Yes/No questions' reading and
  recordings, their viewer, then the choice box whole). Since the queue's entry the engine's
  offers had grown to fourteen kinds; the Shipyard's boat dialog and the recruit dialog stay M5's.
- **Read** (design §5.4; `docs/exe/map.toml`): `message_box`'s arguments again (the "or" flag and
  the network timeout come before the third picture: E17's order, and H3API's, corrected, nothing
  built changes); styles 2, 7 and 10 (`message_box_build`: style 7's OK disabled until a choice,
  0x4F7083, as style 10's); the choice frames (`choice_frame_vtable` 0x63FE1C,
  `choice_frame_popup` 0x4F0B20 in part) and `dialog_default_proc`'s choice
  (`message_box_chosen` 0x6993D4 new); the "or"; every question's and choice's box in the
  handlers the engine's offers and choices copy: a monster's join free and for gold and its
  flight, the Border Guard, the Cartographer, a guarded pile, artifact, scroll and mine, a bank of
  each kind, the Tree of Knowledge, the Warrior's Tomb, Pandora's Box, the Pyramid, a dwelling's
  guards and hire, a chest, the Arena, the Schools. One reading corrected by the first recording
  (style 7's OK had been read as enabled from the start). MCL-Q33: the Arena's Cancel gives +2
  Attack.
- **The harness** (common, additive): the `[[message]]` table's `style`, `chosen` and `marks`;
  `Mem.write_fields` (a monster's aggression, a chest's gold, a Tree's price) and
  `Mem.write_diplomacy`; the situations' `monster`, `guards` and Pandora's `rewards`
  (`h3_map_view::situation`), the visits' new words and a hero's `diplomacy`, a unit test.
- **Recorded** (each twice, the second run `ok`, the frame at 0 exact): `ask-join` (46 timeline
  lines, 17 screens), `ask-join-no` (51, 19), `ask-join-gold` (46, 17), `ask-guard` (46, 17),
  `ask-cartographer` (40, 10), `ask-tomb` (36, 9), `ask-pandora` (48, 18), `ask-hire` (36, 9),
  `ask-pile`, `-artifact`, `-mine`, `-bank`, `-crypt`, `-tree`, `-pyramid`, `-dwelling-guards` (31,
  7 each), `choose-chest` (57, 21), `choose-chest-gold` (52, 19), `choose-arena`, `choose-school`
  (36, 9), `choose-arena-cancel`, `choose-school-leave` (31, 7). Situations `seq-ask-*` and
  `seq-choose-*` (19); timelines and live parts in `map/timelines/`; the sequences in
  `map/to-record.toml`.
- **Checked:** `h3-map-view`'s tests at 100 % lines, regions and functions (`cargo llvm-cov -p
  h3-map-view`, `H3_INSTALL` empty); the clients workspace's fmt and clippy clean; `cargo xtask
  verify --scope map-client --summary --with-original-data` no failing step; `scripts/exe.py
  check` and `cited` clean. No existing situation changes, so `drawn` was not run.

### M4 T2b2 — the questions and choices, played by the viewer (2026-11-18)

On `map-client/e24-m4-t2b2`, the user present at the start; `master` at `dba73b7`, merged
again at #1012. Original data staged from the PC at the start (63 files, SHA-256 checked against
the PC's: one digest of the sorted hash lines, `9c890f81…`; the PC's own clone behind GitHub's,
nothing uncommitted); the setup's baseline stopped at its coverage step by a watcher (fmt,
clippy, doc tests and deny passed), CI's green `master` standing in. The VM restarted once during
the setup; the server behind the session stopped twice during part B (its worker resumed).

- **No cut** (the user, at the start): T2b2 whole, though three parts were offered (style 2 and
  the ten that end in a box or nothing, the six accepted that change the map, the six choices).
  Built in those three parts by three workers one after the other.
- **Wine at its start** (found here and by the client lane at E27 M3 T3b2 the same day): the
  sandbox now sets `TMPDIR=/tmp`, which Wine 9's 32-bit loader does not survive (`free(): invalid
  pointer`); this branch's harness fix (dropping `TMP`, `TEMP`, `TMPDIR`) gave way to `master`'s
  (`TMPDIR` alone) at the merge, a unit test kept (`test_h3drive.WineEnvironment`). The client
  lane's TRAPS has it.
- **Recorded again here** (the original's screens are local): the 22, each `ok` against its kept
  timeline and exact at 0 (`frames --sequences map/to-record.toml --no-check`, one id per run, a
  ballast file for the game's disk check; about an hour).
- **Read** (design §5.4's *Built* bullets): `visit_object`'s case 54 (0x4A97B2, a `map.toml`
  site: a visited monster drawn from `avwattak.def` until the decision returns, the adventure
  manager's +0x218 to +0x220, passes 5 and 7); `message_box_build`'s style 10 buttons, picture
  loop and "or" item in detail. Corrected: the choice frame's colour is the game palette's entry
  45 (T2b1 read 59; recorded (237, 213, 122)); §5.4's addresses of `visit_guarded_artifact`
  (0x49F070) and `visit_search_bank` (0x4AC490), whose `map.toml` entries were right.
- **Built** (design §5.4, parts A1, A2 and B; *provisional*, M4's review list): the module
  `asks` (`Ask`: every offer's and choice's facts, box and replies; a monster's offer decided in
  integers), `Reply`, the tables' `[map.asks]`, the catalogue's `map.ask.*`; `h3_ui::message`'s
  styles 7 and 10 (the OK disabled until a pick, the choice frames, the "or"; common, additive);
  the timeline's `Found::blocks` and `ask`, `Question`, `Answer`, `Loop::ask`, `pose`, `answer`,
  `answered`, `visited`, the attack's picture (`live::Attacking`), the reveal, the removals with
  `killfade` or a pickup's sound and the dissolve, `Start::reached` (the cursor after a walk from
  the recording); the binary's `Painter::asked`, `question`, `answer`, `choice`, `reveal`.
  MCL-Q33 built as the original's (the Arena's Cancel +2 Attack).
- **Checked:** the 22 moved into `sequences.toml` (`to-record.toml` removed), every frame exact by
  `frames --only <id> --reuse` (`ask-pile`, `-artifact`, `-mine`, `-bank`, `-crypt`, `-tree`,
  `-pyramid`, `-dwelling-guards` 7 each, `ask-tomb`, `ask-hire` 9, `ask-join` 17, `ask-join-no` 19,
  `ask-join-gold` 17, `ask-guard` 17, `ask-cartographer` 10, `ask-pandora` 18, `choose-chest` 21,
  `-chest-gold` 19, `choose-arena`, `choose-school` 9, `choose-arena-cancel`,
  `choose-school-leave` 7) and the viewer's timelines the recorded ones; `h3-map-view sequences`
  the 80 as recorded; `drawn` the 50 situations and 58 earlier sequences the same, then
  `--record`; coverage of `h3-map-view`'s library 100 % lines, regions and functions and of
  `h3-ui` but its `scrollbar.rs` (the client lane's, untouched: its tests are `h3-view`'s);
  `RUST_TEST_THREADS=1 check.sh --no-coverage` all gates (after `master`'s merge: the client
  lane's catalogue test passes, every key from tables it lists); the window under Xvfb (`--check
  10`) on `ask-join` (its dissolve) and `choose-chest` (the frames), 58 of 58 frames each read back
  exact, 59 of 59 changes the headless loop's; `scripts/exe.py check` and `cited` clean
  (`dialog_frame_create` 0x450130, part B's citation, a `map.toml` entry); `cargo xtask verify
  --scope map-client --summary --with-original-data` no failing step.

### M4 T2c — a level-up's choice (2026-11-18)

On `map-client/e24-m4-t2c`, the user present at the start; `master` at #1014. Original data
staged from the PC at the start (63 files, SHA-256 checked against the PC's: one digest of the
sorted hash lines, `9421f826…`; the PC's own clone behind GitHub's, nothing uncommitted); the
setup's baseline stopped at its coverage step by a watcher (fmt, clippy, doc tests and deny
passed), CI's green `master` standing in. The PC's link dropped later in the session (nothing
was needed from it by then).

- **No cut** (the user, at the start): T2c whole, the cut after the reading and the recordings
  (as T2a's and T2b's) kept as the fallback; not needed. The reading and the recordings in the
  main session, the viewer by one worker, the morale's staging fixed after it.
- **Read** (design §5.5; `docs/exe/map.toml`: `level_up_dialog` 0x4F8880 and
  `level_up_dialog_proc` 0x4F9780 whole, `chest_gold_or_experience` corrected: its experience
  without levels): the dialog's place and shadow, every item for no, one and two offers, the OK
  disabled with two, the keys 1 and 2, a click on a picture or its frame, the redraw, the
  sound `nwherolv.82m` before the build. **MCL-Q34**: the sentence naming the offers (GENRLTXT
  204, 205) never shows (two leading line breaks below medfont's box); built as the original's.
- **The harness** (common, additive; the runbook's "The level-up's dialog"): `Mem.level_up_box`
  and the live part's `[[level_up]]` tables (the hero, its level, the texts, pictures, offers,
  the choice, the marks), `Mem.message_box` leaving the dialog out; a hero's `secondary`
  (`Mem.write_secondary`, `h3_map_view::situation`'s `HeroSpec::secondary`, checked: eight at
  most, each once, levels 1 to 3); the keys `1` and `2`.
- **Recorded** (each twice, the second run `ok`, the frame at 0 exact): `level-up` (42 timeline
  lines, 11 screens), `level-up-right` (41, 12), `level-up-one` (35, 9), `level-up-none` (35, 9),
  `level-up-twice` (73, 25). `level-up-one` and `-none` staged again after the build's check:
  their first staging (Mephala, Expert Leadership written, her Basic Leadership cleared, after
  the status window at 0) showed another morale from the walk's rebuild on (TRAPS); now a Cleric
  (Rion) without a morale or luck skill. `find-stone`, `choose-chest`, `choose-chest-gold` and
  `ask-tree` recorded again here for their frame check (`ok`).
- **Built** (design §5.5's *Built*; *provisional*, M4's review list): the module `levels`
  (`LevelUp`, the levels an experience reaches by the tables' thresholds, the dialog's facts from
  the live part's tables, its texts, items and choice), the layout's `[level_up]`, the tables'
  `[map.level_up]`, the catalogue's `map.level-up.*`; the timeline's `Gain::experience`,
  `Start::experience`, `Start::level_ups`, `Loop::level_up` and `leveled` (after a find's boxes,
  an answer's gain, a chest's dissolve; the next dialog in the millisecond the last closes; the
  primary skills, a Leadership's morale and a Luck's luck shown at the last close), `Answer::after`
  and `Reply::after`, `Action::Offer` (the window's `offer-left`, `offer-right`; `h3-sdl`'s
  `Digit1`, `Digit2`, additive), `h3_ui::message::outline` (additive); a box's close keeps the
  cursor where the mouse manager last drew it (every box; every timeline as recorded).
- **Checked:** the five moved into `sequences.toml` (`to-record.toml` removed), every frame exact
  by `frames --only <id> --reuse` (11, 12, 9, 9, 25) and the viewer's timelines the recorded ones;
  `find-stone` (7), `choose-chest` (21), `choose-chest-gold` (19), `ask-tree` (7) still exact;
  `h3-map-view sequences` the 85 as recorded; `drawn --record` for the five; unit tests (567 in
  `h3-map-view`, `h3-ui` and `h3-sdl`; the client lane's catalogue test passes); coverage of
  `h3-map-view`'s library 100 % lines, regions and functions and of `h3-ui` but its
  `scrollbar.rs`; `RUST_TEST_THREADS=1 check.sh --no-coverage` all gates; the window under Xvfb
  (`--check 8`) on `level-up-right` (the keys 1 and 2, the OK clicked) and `level-up-twice`, 44 of
  44 and 38 of 38 frames read back exact, every change the headless loop's; `scripts/exe.py
  check` and `cited` clean (`two_lines_format` 0x67FA38 and `plus_one_format` 0x67FA40, the
  catalogue's citations, `map.toml` entries); `cargo xtask verify --scope map-client --summary
  --with-original-data` no failing step.

### M4 T2d1 — a flag raised (2026-11-18)

On `map-client/e24-m4-t2d1`, the user present at the start; `master` at #1021. Original data
staged from the PC at the start (63 files, SHA-256 checked against the PC's: one digest of the
sorted hash lines, `28fcf718…`; the PC's own clone behind GitHub's, nothing uncommitted; the
four manuals and three help files too); the setup's baseline stopped at its coverage step by a
watcher (fmt, clippy, doc tests and deny passed), CI's green `master` standing in.

- **The cut** (the user, at the start; the recommended one): T2d in three by family, each read,
  recorded and built in one task as T2c was (T2d1 a flag raised, T2d2 a boat boarded and left,
  T2d3 a spell cast on the map, MCL-Q14); "a monster beaten" moved to T5 (its recording needs a
  battle; T5 hands the map back with the result applied); "an object taken" was T2a2's and T2b2's.
  The user also allowed the client lane's catalogue test word for MINEEVNT (`cross-lane`); the
  build made it a family instead, so it was not needed.
- **Read** (design §5.6; `docs/exe/map.toml`: `visit_and_update` 0x4AAAA0 now `read`,
  `mineevnt_load` 0x5B9150 and `mine_event_texts` 0x6A5E20 new): a mine's and a Lighthouse's box
  and flag (`visit_mine`, `visit_object`'s case 42), a dwelling's flag before its hire question;
  nothing redrawn by the visit, the minimap at `visit_and_update`'s end, the view at its next
  redraw; the reveal of radius 3 shows nothing new.
- **Recorded** (each twice, the second run `ok`, the frame at 0 exact; no harness change):
  `flag-mine` (34 timeline lines, 8 screens), `flag-mine-blue` (34, 8), `flag-lighthouse` (34,
  8), `flag-dwelling` (36, 9); situations `seq-flag-*`.
- **Built** (one worker; design §5.6's *Built*; *provisional*, M4's review list): `Find::Mine`,
  `Find::Lighthouse`, `Find::flag_of`, `Find::flagged`; a neutral dwelling's flag before its hire
  (`Ask::flags`); the timeline's `Found::flag`, the owners in memory, `Draw::owners`,
  `Minimap::owners`; `TileIndex::minimap_again`; the tables' `mine_income`; the catalogue's
  `map.find.lighthouse` and the family `mine-event`; `timeline/tests/flags.rs` (the four replayed
  headless).
- **Checked:** the four moved into `sequences.toml` (`to-record.toml` removed), every frame exact
  by `frames --only <id> --reuse` (8, 8, 8, 9) and the viewer's timelines the recorded ones
  (`<id>-frames.txt` kept); `h3-map-view sequences` the 89 as recorded; `drawn` the 50 situations
  and 85 earlier sequences the same, `--record` for the four; 406 library tests; coverage of
  `h3-map-view`'s library 100 % lines, regions and functions (the worker's run); the window under
  Xvfb (`--check 8`) on `flag-dwelling` and `flag-mine-blue`, 44 of 44 and 45 of 45 frames read
  back exact, every change the headless loop's; `scripts/exe.py check` and `cited` clean.

### M4 T2d2 — a boat boarded and left (2026-11-18 to 2026-11-19)

On `map-client/e24-m4-t2d2`, the user present at the start (big tasks cut only with their
consent); `master` at #1026. Original data staged from the PC at the start (63 files, SHA-256
checked against the PC's line by line; the four manuals and three help files too; the PC left
about 20 minutes in); the setup's baseline stopped at its coverage step by a watcher (fmt,
clippy, doc tests and deny passed), CI's green `master` standing in. The session's server
stopped overnight while the worker built (the VM restarted, every file kept); the worker resumed
from its transcript.

- **The cut:** none. Sized at the start as T2d1 (both visits read whole by E08 T66a1, the step's
  two branches and the draws left), done whole; the fallback cut (T2d2a the reading and the
  recordings, T2d2b the viewer) stated to the user and not needed.
- **Read** (design §5.7; `docs/exe/map.toml`: `hero_move_step`, `hero_boards_boat`,
  `hero_lands_from_boat`, `hero_loss_effect` given the draws): the boarding from land (the hero
  unlifted and dissolved off with killfade before the frames, the boat lifted in its own facing's
  group after the step), the landing (the boat put back facing the step and shown before the
  frames, the hero dissolved in with killfade after); `hero_walk_end` before the visit in the
  loop and the key's step; the mirror by the hero's facing (MCL-Q35). The worker's corrections
  (the build): `hero_boards_boat`'s `view_show_step`, the key step's tail after
  `status_window_choose` (`make_hero_path`, `hero_list_update`), no button in
  `visit_and_update`.
- **Recorded** (each twice, the second run `ok`, the frame at 0 exact; speed 2, `points` 1 500,
  `seed` 7; no harness change): `board` (48 timeline lines, 18 screens), `board-walk` (74, 25),
  `land` (47, 18), `land-walk` (82, 27; its first target (12, 12) was water, moved to (13, 12));
  situations `seq-board`, `seq-board-walk`, `seq-land`. Two sound runs (scratch): killfade at
  each dissolve, the stepped tile's step sound started and stopped in one millisecond, the
  ocean's ambient loop.
- **Built** (one worker; design §5.7's *Built*; *provisional*, M4's review list): the timeline's
  `Boats`, `Start::boats`, `Start::water`, `Draw::hidden`, `Draw::boats`; the human's step as
  `mobilize`, `stride` and `animate`; `board_start`, `land_start`, the visits `board` and
  `land`, `updated`, `stuck`; `Dissolving::holds` and `then`; `Walking::hidden`; the binary's
  boats and water; `timeline/tests/boats.rs` (the four replayed headless and nine focused tests).
- **Checked:** the four moved into `sequences.toml` (`to-record.toml` removed), every frame exact
  by `frames --only <id> --reuse` (18, 25, 18, 27) and the viewer's timelines the recorded ones
  (`<id>-frames.txt` kept); `h3-map-view sequences` the 93 as recorded; `drawn` the 50 situations
  and 89 earlier sequences the same, `--record` for the four (88 digests); 416 library tests;
  coverage of `h3-map-view`'s library 100 % lines, regions and functions (the worker's run); the
  window under Xvfb (`--check 8`) on `board` and `land-walk`, 55 of 55 and 61 of 61 frames read
  back exact, every change the headless loop's; `scripts/exe.py check` and `cited` clean;
  `RUST_TEST_THREADS=1 check.sh --no-coverage` all gates passed; the scoped verify
  `--with-original-data` no failing step.

### M4 T2d3 — a spell cast on the map (2026-11-19)

On `map-client/e24-m4-t2d3`, the user present at the start (big tasks cut only with their
consent); `master` at #1034. Original data staged from the PC at the start (63 files and the four
manuals and three help files, SHA-256 checked line by line; the PC's tools went away about 90
minutes in, nothing needed by then); the setup's baseline stopped at its coverage step by a
watcher, CI's green `master` standing in; a ballast file for the game's disk check.

- **The cut:** none. The user at the start (2026-11-19): Summon Boat of the four (no target to
  pick; Dimension Door and Scuttle Boat need a target mode, View Air and Earth the View World
  screen), whole, the fallback cut (T2d3a the reading, the harness's cast and the recordings,
  T2d3b the viewer) stated and not needed.
- **Read** (design §5.8; `docs/exe/map.toml`: `adventure_cast_open` 0x41C2F0, `view_clip_init`,
  `view_clip`, `manager_clear_50`, `scuttle_boat_cast` and `hero_special_ground` new;
  `adventure_spell` 0x41C490 was `visions_cast`, its switch read; `summon_boat_cast` read whole;
  Dimension Door's and Town Portal's entries told their cases): the opening (the lift, the view
  shown, the book modal on its adventure bookmark), Summon Boat's refusals, chance, boat, box,
  dissolves, sound and cost, `hero_spend_mana`'s slot. **MCL-Q14 seen**: the slot pixel-identical
  after the cast, the mana bar stale; the client's fix as the user answered at M3 T7c1.
- **The harness** (*provisional*; the runbook's "A cast on the map"): `Mem.spell_book`,
  `Mem.modal_answer`, `Mem.mana`, the boats' `index`; no screen kept while the book shows, the
  timeline's `book` lines, `Mem.message_box` none under it; the `[[cast]]` table (its boats from
  the book's opening: in `cast-boat-far` and `-new` the boat moves or is made in the click's own
  millisecond, so the first run's snapshot at the close missed them); a situation's hero's
  `spells` (`h3_map_view::situation`: the writer's book in slot 17); a sound run's clock let run
  before its settling (the look over the view leaves it stepped: the first two sound runs never
  settled).
- **Recorded** (each twice, timelines, `[[cast]]` tables and screens the same; speed 2, `seed` 7):
  `cast-boat` (48 lines, 24 screens), `cast-boat-far` (36, 15), `cast-boat-new` (36, 15);
  situations `seq-cast-boat`, `-far`, `-new` (`seq-board`'s lake one column right, the hero at the
  view's middle with Summon Boat and Expert Water Magic). One sound run (scratch): the press, the
  ambient restarted by the lift, `SUMMBOAT` (1 200 ms) at the second dissolve and the cast's end
  waiting for it.
- **Built** (one worker; design §5.8's *Built*; *provisional*, M4's review list): `live::Cast`,
  `Start::casts`, `State::book`; `cast_open`, `book_act`, `summon`, `arrive`, `cast_dissolve`,
  `in_view`, `boat_box`, `cast_end`; the dissolve's own box, step and source (`Dissolve::from`,
  nested `before`: the second dissolve blends from the back buffer the first drew); `[map.cast]`
  and `summboat.wav` in the tables, the book's cursor in the layout; MCL-Q14 by
  `HeroStatus::mana` and `draw_slot` (`the_lists_drawn_again_by_a_lift…`, a `Bugs::Fixed` test that
  asserted the stale bar, now asserts the fix); `Unplayable::Cast`, `Unplayable::Spell`;
  `timeline/tests/casts.rs` (11 tests), the `[[cast]]` reading's tests, the situation's spells'.
- **Checked:** the three moved into `sequences.toml` (`to-record.toml` removed), every frame exact
  by `frames --only <id> --reuse` (24, 15, 15), the viewer's timelines the recorded ones;
  `drawn` the 50 situations and 93 earlier sequences the same, `--record` for the three; the
  window under Xvfb (`--check 8`) on `cast-boat` 49 of 49 read back exact (live: no cast); 430
  library tests, coverage of `h3-map-view`'s library 100 % lines, regions and functions (the
  worker's run); `RUST_TEST_THREADS=1 check.sh --no-coverage` all gates passed; `scripts/exe.py
  check` clean; the scoped verify `--with-original-data` no failing step (its `check-exe-map`
  first found `view_clip`'s kind and the cited 0x4E4EC0 without an entry: both added).
- **Found on the way** (the lane's inbox): `killfade` and the pickups are not among
  `SoundRules::effects`, which the window's library decodes, so the window seems to play them
  silent (the worker's reading, not heard).

### M4 T3a — a record's start: the map's view (2026-11-19)

On `map-client/e24-m4-t3a`, the user present at the start; `master` at #1044. Original data staged
from the PC at the start (63 files and the four manuals and three help files, SHA-256 checked line
by line; the PC's shell went away about 15 minutes in, nothing needed by then); the setup's
baseline stopped at its coverage step by a watcher (every step before it passed), CI's green
`master` standing in; a ballast file for the game's disk check.

- **The dependencies checked first:** the server lane's whole-game record there (protocol v43/v44,
  now v74; `h3-server record`), increment 9's views there (the adventure lane's T83b2b, T85); the
  object identity not built: the server lane's note of 2026-11-12 (a view's objects in tile order,
  found by type and anchor) unanswered.
- **The cut** (the user at the start, `AskUserQuestion`; the agent's recommendations): T3 in two,
  the map's view first (T3a; T3b the panels, the hover's facts, MCL-Q15 (b), the whole screen,
  shipped maps); the object identity withdrawn (the server lane's inbox answered, the adventure
  lane's told), a random artifact's drawn artifact still asked (the view names none); later in
  the task, a random dwelling's class and subclass asked of it too.
- **Built** (design §5.9; *provisional*, M4's review list): `h3_map_view::engine`: `parse` (a
  record's lines, its format and protocol), `load` (the manifest the protocol makes of the
  viewer's rules and the map file equal to the record's; the map read and imported: MCL-Q21 (b)),
  `start` (the first snapshot as `live.toml`'s table: `viewer`, `defs`, `classes`, `owners`,
  `heroes`, `boats`, `explored`; which objects the view shows), `differences`; `render --record
  [--packs] [--check]`, `render::apply_live` (a table applied as a capture's file is,
  `read_table`), `files::default_packs`; a writer situation's `explored = "start"`
  (`situation::Explored`). The library's pure dependencies grew by `h3-protocol`, `h3-game`,
  `h3-adventure`, `h3-kernel` (its purity test's allowlist), the binary's by `h3-files`, the tests'
  by `h3-contracts`; the client workspace names `h3-adventure` (additive, the common foundation).
- **Agent decisions** (change freely): the engine's start written in the original's memory's terms
  (a capture's table) rather than `Live`, so that `render`'s application (a random dwelling's
  template, a town's picture check) and the comparison are a capture's; an object out of sight
  keeps the file's owner; a hero faces right at the start; a random dwelling one column left of
  the view's anchor (the Angel Portal's shift) is found, any other random object only at its
  anchor (the first build let every random object shift: the tests' moved artifact caught it).
- **Checked:** `engine-start-owners` (`owners-objects`' map) and `engine-start-town` (Deluge's
  lake, Red's town and two heroes, Blue's hero in sight, three empty boats, monsters and piles
  with their counts given, an artifact, Blue's mine) captured (`set --only`, 0 differing of
  480 000 each); each map's start recorded by the engine (`h3 adventure run --script` with no
  command, `h3-server record`) and `render --check`ed against its capture: no difference (the
  town's: 249 explored tiles, seven objects in sight, three heroes, three boats; Purple's hero
  out of sight left out on both sides). `drawn`: the two new, every other situation and sequence
  the same; `drawn --record`. `tests/engine.rs` (six tests: a start with a random resource,
  monster, artifact and dwelling; a hero in a boat and a random resource shown as no pile; a
  random dwelling moved right; what differs from a capture; every refusal; each refusal's text).

### M4 T3b1 — a record's start: the panels (2026-11-19)

On `map-client/e24-m4-t3b1`, the user present at the start; `master` at #1048 (T3a's #1047
merged). Original data staged from the PC at the start (63 files, the four manuals and three help
files, SHA-256 checked line by line); the setup's baseline stopped at its coverage step by a
watcher (clippy, the doc tests and deny passed), CI's green `master` standing in; the Wine capture
set up beside it; a ballast file for the game's disk check.

- **The cut** (the user at the start, `AskUserQuestion`; the agent's recommendations): T3b in two,
  the panels first (T3b1; T3b2 the hover's facts, a town's picture by its buildings, the shipped
  maps); a fact the record's first observation lacks becomes an `MCL-Q<n>` with notes in the
  engine lanes' inboxes, the gap shown in the comparison, no rule copied into the client.
- **Found first** (the engine's starts of T3a's two situations against their kept live parts):
  the original's new game draws each hero's army and the starting bonus (`engine-start-town`'s
  20 500 gold, `engine-start-owners`' Orrin with an artifact's [2, 2, 2, 2] and mana 10), which the
  engine does not (armies its own draw; no bonus, E10's); the hero list in its heroes' types'
  order (every capture's `hero_list`, 17 situations with two or more).
- **Built** (design §5.10; *provisional*, M4's review list): `h3_map_view::engine`'s `panels`
  (`kingdom`, `status`, `realm`, `[chosen]` in the harness's terms) and their `differences` (a
  town's buildings by its hall and fort, a morale or luck named with MCL-Q36); `live::TownName`
  (a record or the map's own name, a `status`'s `name` as text); MCL-Q15 (b): `panels::AMOUNT_FITS`,
  the layout's `tiny.fnt` items 1016 to 1022, `resource_bar`'s `bugs`; a situation's hero `name`
  and `map_army`, town `name` (the writer's `AnyTown`), `reselect`, `bonus`; the harness's
  `bonus = "gold"` (`h3drive.to_scenario`'s `gold` a number of rows; `gold_bonus` counts the
  rows' Gold: Blue's row sat in the dialog's second array), `reselect`, `OWN_TOWN_NAMES`, `h3map.py
  engine` and `set.toml`'s `engine` entries.
- **Agent decisions** (change freely): the panels written as a capture's tables, as T3a's view;
  the town list in the view's order (one town a player here); an unnamed town's name its
  faction's first; the silo the income's resources but gold; `h3map.py engine` compares the
  engine's PNG with ours at the kept live part (exact against the original's there) rather than
  with the original's screen again.
- **MCL-Q36** (asked): a hero's morale and luck as the status window shows them, and whether it
  can still move, are not in the record; recommended (a), the engine derives them and the view
  carries them to the owner (the adventure and server lanes' inboxes). Meanwhile 0, 0 and "has
  points".
- **Checked:** `engine-start-panels` (new: Rion and Astral with their armies in the map, Astral and
  Red's town named by it, each chosen in turn) and T3a's two (their heroes' armies in the map, the
  Gold bonus; `engine-start-town`'s town named and its resources written) captured (`set --only`,
  0 differing of 480 000 each); `h3map.py engine`: `engine-start-panels` 0 pixels and no
  difference, `engine-start-town` 157 pixels (Orrin's morale, MCL-Q36), `engine-start-owners` 192
  (the bonus's gold, one hero of Red's to select; the morale). `drawn`: the three changed or new
  (recorded), every other situation and sequence the same. `tests/engine.rs` (four new tests),
  the situation's, the live part's, the status window's and the resource bar's unit tests.

### M4 T3b2 — a record's start: the hover, the towns' pictures, the shipped maps (2026-11-19)

On `map-client/e24-m4-t3b2`, the user present at the start; `master` at #1052 (T3b1 merged).
Original data staged from the PC at the start (63 files, the four manuals and three help files,
SHA-256 checked); the setup's baseline stopped at its coverage step by a watcher (fmt, clippy, the
doc tests and deny passed), the Wine capture set up beside it.

- **The cut** (the user at the start, `AskUserQuestion`; the agent recommended two: the hover and
  the towns' pictures, then the shipped maps): **kept whole**; `MCL-STUB-1` still the adventure
  lane's (its note of 2026-11-19 unanswered), the facts the record lacks an `MCL-Q<n>`.
- **Built** (design §5.11; *provisional*, M4's review list): `h3_map_view::engine`'s `hover` (the
  `[hover]` table from the first observation), a town's picture by its view's fort and Capitol (a
  town out of sight by the map's buildings), `Hover::differences` and `Hover::start_selection`,
  `render --record --check` comparing the mouse-over over every explored tile, a still's pointer
  over the view drawn (`render`: the map's hint and cursor at its tile), `Facts::name_towns` and
  `MapArt::town_names` (a capture's 65535 named from the map), `hover::SEVERAL`,
  `Hint::Town`'s `TownName`, a situation's `pointer`; the harness's still `pointer`
  (`look_over_view` first), the shipped situations' staging keys, `h3map.py engine` on a shipped
  (gzip) map.
- **Found:** the engine's views show no Seer's Hut, Quest Guard or Obelisk yet (Dwarven Tunnels'
  Obelisk: its name alone); a town out of the view's sight kept its editor's template where the
  original draws its start's picture (Jared Haret's Capitol: fixed); a scan of the 160 shipped
  starts in the engine (random objects, monsters without a count, abandoned mines and unnamed
  towns in Red's sight, random or generated heroes, heroes the map does not arm; a scratch test,
  not kept): two with no draw in sight, Adventures of Jared Haret and Good Witch, Bad Witch, both
  heroes at level 3 by the map's experience (their level-ups' primary skills drawn in each game);
  Hoard lists Red's Fortress before its Castle with no main town named (MCL-Q38: the main town
  first was tried and dropped).
- **Agent decisions** (change freely): the facts written in the capture's format (as T3a's and
  T3b1's); the comparison by every tile's hint text and cursor rather than the facts' rows (the
  counts are words in the view); the shipped starts' level-up draws listed, not staged.
- **MCL-Q37** (asked; the adventure and server lanes' inboxes): the cursor's facts (costs over
  several days, guarded tiles, a defended town at level 0) and a visited bank's guards are not in
  the record. **MCL-Q38** (asked): a shipped map's town list at a new game.
- **Checked:** `engine-start-hover` (new), `start-jared` and `start-good-witch` (new, shipped)
  captured exact; `engine-start-town` and `-panels` captured again (`[hover]`'s towns as the harness
  writes them); `h3map.py engine`: six starts as expected (every explored tile's hint and cursor the
  capture's; pixels: MCL-Q36's morale, `engine-start-owners`' bonus gold, the shipped heroes'
  level-up skills). `tests/engine.rs` (three new tests: the facts of a start, of what a player
  learnt, the towns' names), the hover's unit tests (a town the map names, two mouse-overs
  compared, the towns named from the map), the guards' `SEVERAL`.

### M4 T4a1 — a record's own walks and turns: the reading and the scripted games (2026-11-19)

On `map-client/e24-m4-t4a`, the user present at the start; `master` at #1056 (T3b2 merged).
Original data staged from the PC at the start (64 files with `shell-videos.vid`, the four manuals
and three help files, SHA-256 checked); the setup's baseline stopped at its coverage step by a
watcher (fmt, clippy, the doc tests and deny passed), the Wine capture set up beside it. The VM
restarted once mid-recording (files kept; `game-day` and `game-week` recorded again).

- **The inbox's window-sound item first** (the user's order): `tables::load` adds the pickups'
  seven and `killfade` to `SoundRules::effects`, so the window's library decodes them (95 WAVs;
  the tests count them and a table without `killfade`). All eight are in `Heroes3.snd`.
- **The cuts** (the user, `AskUserQuestion`): T4 in three by event family (T4a the viewer's own
  turns, T4b the visits, T4c the other players, the controls and a golden replay); then T4a in
  two, the agreed fallback, once the record's gaps and the unread arrows showed (T4a1 the reading
  and the recordings, T4a2 the viewer).
- **Read** (design §5.12; `docs/exe/map.toml`): `make_hero_path` 0x418DD0 whole (the arrow per
  tile from `path_arrow_frames` 0x6782AC by the step in and out, the destination's cross, the red
  twin where the points left fall below 0, move hero's state), `hero_move_step`'s order (the
  reveal at the step's destination before the animation, the points after it).
- **Recorded** (`clients/map/to-record.toml`, new; `h3map.py` accepts a sequence's `script`):
  `game-walk` (two paths clicked twice and a key's step), `game-day` (End Turn, day 2, a walk),
  `game-week` (End Turn seven times, week 2's message), on `game-walk` and `game-turns` (new
  situations the engine imports). The engine's records of their scripts: the same tiles walked,
  the same points after each command, the same income, week 2 normal in both. `game-walk`
  recorded twice: the same timeline.
- **Found:** the original banishes a player without a town after seven days (`game-week`'s first
  take: Blue, then Red's victory): E10's rule, not the engine's yet; the scripted games give
  every player a town. The kept live parts in git hold the original's box texts (41 files since
  T2a1): against ADR-0009, the user's answer: a labels task first, none new meanwhile
  (`game-week-live.toml` not kept).
- **Agent decisions** (change freely): the scripted games' second hero (its selection shows the
  engine's resources), Blue's island with a town, the scripts written from the recordings' `hero`
  lines.
- **MCL-Q39** (asked; the adventure and server lanes' inboxes): a walk's points and reveal per
  step are not in the record. **MCL-Q40** (asked; the adventure lane's inbox): a normal week's and
  month's name.
- **Checked:** `game-walk`'s engine start against its recording's live part (`render --record
  --check`): only the cursor over the two tiles whose cost the capture read (MCL-Q37);
  `check.sh --no-coverage` all gates, the gate, `drawn`, `scripts/exe.py check`, the scoped
  verify.


### M4, the labels — the kept live parts' texts as their records (2026-11-20)

On `map-client/e24-m4-labels`, the user present at the start (the task as queued: kept whole);
`master` at #1064 (T4a1 merged), CI green there. Original data staged from the PC at the start
(64 files with `shell-videos.vid`, the four manuals and three help files, SHA-256 checked: one
hash of each side's sorted list); the setup's baseline stopped at its coverage step by a watcher
(fmt, clippy, the doc tests and deny passed), the Wine capture set up beside it. The PC was not
needed after the first minutes.

- **Read:** nothing of the viewer reads a live part's box texts. The binary takes from
  `[[level_up]]` only `hero`, `level`, `offers` and the pictures' frames, nothing from
  `[[message]]` and `[[found]]`; the timeline tests compare the viewer's own texts, composed
  from their catalogues. The texts kept: 353 in 42 files (41 with boxes or level-ups, and
  `find-pile`'s found text): each box's texts and pictures' captions, each level-up's texts.
- **Built** (the harness, common, additive): `h3lod.label` (and `h3lod.py label <text>`): a text
  as the records it was made of, `[ADVEVENT:86]` for a text one record of `FORMAT_TABLES`
  (ADVEVENT, GENRLTXT, ARRAYTXT, ARTEVENT, RANDSIGN, MINEEVNT; two neighbours' `89+90`) formats
  with only numbers and names in its conversions (ADVEVENT 113's resource in lower case: names
  matched in any case there), else piece by piece (a name of `NAME_FIELDS`, `[CRTRAITS:3.1]` the
  row's field 1, or a record without conversions; numbers and punctuation kept, `[other: n
  characters]`), `[the map's text]` for a situation's own (`h3map.own_texts`: its `text` and
  `message` strings; `seq-ask-pile`'s guard); a label left as it is. `h3map.py`'s
  `message_toml`, `level_up_toml` and `found_toml` write labels (`h3map.labels`). The identify
  of before named ARTEVENT 7 (the Centaur's Axe) GENRLTXT 16 (`%s the %s`): ARTEVENT is in the
  tables now, and a conversion holding a sentence is refused.
- **The 42 files rewritten** by the same functions from the run directory's tables (a one-off
  script, not kept: every other line byte for byte; 135 lines). Checked by recording
  `find-sign`, `level-up-one` and `find-pile` again under Wine (all as recorded, exact): their
  new live parts' texts equal the rewritten ones, `find-pile`'s whole file too; the other two
  differ in the palette's phase at 0 (`cycles` 10, not 11, a run's) and `find-sign`'s table a
  button's state appends since T2b1 (TRAPS), so git's lines were kept.
- **The guard:** `h3-map-view`'s `tests/texts.rs` (no original data: CI runs it): every kept
  live part's texts hold labels, numbers and punctuation only (the grammar), at least the 353;
  a second test of what it accepts and refuses. Against git's old files it fails.
- **Scanned** (a one-off): every field of four words or more of both archives' text tables
  (6 647 pieces), looked for in every file of `clients/map/`: only names (a shipped map's name in
  three situations' `why`, the three Shrines' names in two). Outside `clients/map/`: the
  viewer's tests' own catalogues (`finds/tests.rs`, `asks/tests.rs`, `status/tests.rs`) imitate
  the catalogue's texts, a few of them the original's sentences (a sign's default text, a
  pile's and a bank's message): in the inbox for the user.
- **Agent decisions** (change freely): the labels' grammar; no arguments in a label (the viewer
  reads none, the record names the message); exact matches only outside a conversion, so a flag
  box's caption "Red" is `[other: 3 characters]` (PLCOLORS holds "red"); `[the map's text]`
  without the text (it is in the situation); the guard a client integration test (M4's review
  list).
- **Checked:** `h3-map-view`'s tests, clippy, fmt; the three recordings; `drawn` all the same;
  `check.sh --no-coverage` all gates; the scoped verify.


### M4 T4a2 — a record's own walks and turns played (2026-11-20)

On `map-client/e24-m4-t4a2`, the user present at the start (the last session's cut in two
offered, T4a2 kept whole by the user); `master` at #1068 (the labels merged), CI green there,
#1066 merged in later. Original data staged from the PC at the start (the 64 files with
`shell-videos.vid`, the four manuals and three help files, SHA-256 checked: one hash of each
side's sorted list); the setup's baseline stopped at its coverage step by a watcher (fmt,
clippy, the doc tests and deny passed), the Wine capture set up beside it. The PC was not
needed after the first minutes.

- **Recorded again** (fresh sandbox, TRAPS): `game-walk`, `game-day`, `game-week` under Wine,
  each exact at 0; `game-week`'s live part kept now (with labels); the original's timelines
  kept (`timelines/game-*.txt`); the three moved into `sequences.toml`, `to-record.toml` removed.
- **Built** (design §5.13): `engine::played` (the record folded: its walks and the viewer's
  turns' starts as `[[turn]]` tables); `timeline::Start::moves`, `Move`, `Draw::walked`,
  `Minimap::walked`, `Unplayable::Moved`, `timeline::arrows`; `[map.walk] arrows` (the 64
  numbers of `path_arrow_frames`); the binary's `sequences --record --record-map --packs` (the
  record's start and turns after the kept live part, the paths from the record's walks; the
  timeline's differences against `timelines/<id>-gaps.txt`); the painter's view and minimap
  built for their walked tiles; `h3map.py`'s `engine_game`, `engine_map` and `scripted` (the
  record of a sequence's `script` for `frames` and `drawn`), `VIEWER_SAME`; a sequence's
  `script` read by the viewer's `Sequence` (unused there).
- **Found:** the three games exact but the listed gaps (design §5.13): `game-walk` and
  `game-day`'s walks the step points and the reveal shown from the first step (MCL-Q39 (c)),
  `game-week`'s week name (MCL-Q40 (b)); End Turn, the day's start and the income exact.
- **Agent decisions** (change freely; M4's review list): the record's flags; the n-th walk the
  record's n-th `MoveHero`; a turn's table from the panels with the date's window, `at` 0, no
  heroes or boats; the client's week and month names; the gaps file's form; a step's reveal
  before a boarding's dissolve (not read).
- **Tests:** `timeline::tests::moves` (the reveal's and the points' times, a key's step, the
  steps off the record, the arrows of every pair of steps), `tests/engine.rs`'s
  `a_record_s_walks_and_turns` (a `GameSession`'s record: a walk and seven days) and
  `a_record_s_rarer_lines` (each week's and month's kind, a snapshot, a walk without tiles, a
  record ending in a walk, a tile past a byte), `sequences`' differences.
- **Checked:** see design §5.13.

### M4 T4b1 — a record's visits played: the message boxes (2026-11-20)

On `map-client/e24-m4-t4b1`, the user present at the start: T4b cut in two by family (T4b1 the
boxes, T4b2 the flags and the boats; the user's choice among a cut in three, two by phase and
whole), Summon Boat moved out of T4b (the engine casts no adventure spell). `master` at #1072
(T4a2 merged); its clients CI red on the client lane's `save` test (#1069 and #1070 crossed:
`format = 20` against replay format v21; the user told), fixed by the client lane on `master`
during the session (70f4e6b, merged in). Original
data staged from the PC at the start (the 64 files and the manuals, SHA-256 checked); the setup's
baseline stopped at its coverage step by a watcher; the Wine capture set up beside it. The PC
was not needed after the first minutes.

- **Read** (the engine's games of Part A's situations, design §5.14): each visit's events in a
  record, by class; what the record lacks (MCL-Q41, MCL-Q42; the morale and luck of an
  artifact's bonus, MCL-Q36, seen in a first take of `game-finds` with the Pendant of Courage).
- **Recorded:** `game-finds`, `game-asks`, `game-level` under Wine (situations of the same
  names), each exact at 0; `game-level`'s engine seed 74 found by a search (the original's
  level-up draws under seed 1). A first take of `game-finds` used Space on the Temple (the
  original shows the visited box; the viewer builds no visit on Space: walked off and on
  instead).
- **Built** (design §5.14): `engine::played`'s `Visit`, `Leveled`, `ends` and `experience`;
  `engine::visits` (`told`, `gain`, `Told`, `Answered`); the adapter's `record_finds` and the
  record's level-ups with `LevelBox::chosen`; the timeline's visit from beside on a record (its
  move, its points) and the level-up's OK refused with another offer; `record_paths` from where a
  walk left the hero; `h3map.py`'s `engine_seed` (`h3 adventure run --seed`), the viewer's
  `Sequence::engine_seed`; the timeline's `flush` cleared before a millisecond's actions (a box
  the loop's turn opened dropped every action after the first of a later millisecond's: found by
  `game-asks`' click and pointer at 1000).
- **Found:** `game-finds` and `game-level` exact (no gap, no frame differing); `game-asks`' timeline
  exact, 3 of its 43 frames differing by 136 pixels: the chest's box without the experience's
  amount (MCL-Q41 (c)). Each recorded twice, the second `ok`; `game-finds`' second run moved the
  objects' step after the pile's dissolve a millisecond later (3721 for 3720, not explained), a
  third run as the first.
- **Agent decisions** (change freely; M4's review list): the visit's shape and its end at
  another command; a find's later visit as its `again`; the questions' answers left out but the
  record's; `engine_seed` by a search; the pointer moved off the map after each click in the
  scripted games (MCL-Q37's reach); the Pendant of Dispassion; the `flush` fix.
- **Tests:** `tests/engine.rs`' `a_record_s_visits_and_level_ups` (visits, answers, the level-ups'
  offers and choices, a hero without details, a skill past a byte, `ends`, the experience) and
  `what_a_record_s_visits_tell` (every class and answer, the gains);
  `timeline::tests::finds::a_record_s_visit_from_beside_takes_its_move`,
  `timeline::tests::levels::a_record_s_level_up_closes_only_with_its_answer`; the sequences'
  count and ids.

### M4 T4b2 — a record's visits played: the flags and the boats (2026-11-20)

On `map-client/e24-m4-t4b2`, the user present at the start: T4b2 kept whole (offered a cut by
family, the flags first). `master` at #1078 (T4b1 merged). Original data staged from the PC at
the start (the 64 files and the manuals, SHA-256 checked); the setup's baseline stopped at its
coverage step by a watcher; the Wine capture set up beside it. The PC was not needed after the
first minutes.

- **Read** (design §5.15): the engine's records of the two games: a flag's `ObjectFlagged` in
  the visitor's colour (a dwelling's before its `OfferOpened`; the hire's `CreaturesJoined`, and
  in the engine `JoinRefused` and `NoneToRecruit`), `Boarded` and `Landed` (the boat's type, the
  water tile, the landing's facing), a turn's start's new points after `TurnStarted`.
- **Recorded:** `game-flags` (a Sawmill, a Lighthouse, a Guardhouse with its Pikemen) and
  `game-boats` (a boat boarded, End Turn, a step at sea and back, the landing) under Wine, each
  twice (the second run `ok`), exact at 0.
- **Built** (design §5.15): `engine::visits::told`'s mines, Lighthouses and dwellings (`Told`'s
  `flag`; `hired`: the free creatures as the record tells them); the adapter's `record_finds`
  raising the record's flag; `timeline::Sea` and `Move::sea` from `engine::played`, the timeline's
  `sea_on_record` and `boarded` in `stride` (Part A's `boat_on` and `landing` for a recording
  only); the viewer's turn's start table after its message is folded; `lose` keeping the status
  window its draw chose (the date's step, shown at once).
- **Found:** both games exact, their timelines and every frame, no gap; before the last two
  fixes `game-boats` differed in the points at the new day (1460 to 3000, 139 pixels) and the
  date's window through the landing's dissolve (4250 to 4705, 20 790 pixels). The record's start's
  boats with Part A's rules had played the boarding and landing as the original already.
- **Agent decisions** (change freely; M4's review list): the flag's colour the record's; a
  declined hire not built; the record's boarding and landing in place of the boats and the water,
  a boarding with no empty boat on its tile off the record; the turn table after the message;
  `lose`'s status window; the games' layouts (Blue on an island for End Turn, a boat facing the
  step).
- **Tests:** `tests/engine.rs`' `what_a_record_s_flags_tell` (the classes, the colour, the
  hire's three outcomes and the answers that tell nothing), `a_record_s_boats` (`Move::sea` of a
  boarding, a landing, another hero's and a facing past 7) and the turn's points in
  `a_record_s_walks_and_turns`; `timeline::tests::boats::a_record_boards_and_lands_as_it_says`;
  the sequences' count and ids.
- **Checked:** each game recorded twice (the second `ok`, every frame exact); `drawn` the 56
  situations and the 102 other sequences the same (`--record` for the two); `check.sh` all gates
  (`h3-map-view`'s coverage 100 %: a unit test for the dissolve's date window and the hire's other
  events added after the first run); the scoped verify.

### M4 T4c (2026-11-20; the user at the start: kept whole, the cut in two or three declined)

- **Read** (design §5.16): a viewer's record of another player's turn (`TurnStarted`, one
  `GameEvents` a command of its, the steps the viewer sees by either end, the hidden end none,
  no `MovementChanged`); `ai_hero_step` 0x42FC50 whole and `hero_move_step`'s step end again
  (0x481191 to 0x481440: the computer's walk starts no step sound, a terrain change at the view's
  middle asks for its music over the computer's and plays its step sound); `hero_select`'s music.
- **Built** (*provisional* unless said): `engine::played`'s other players' turns
  (`OtherTurn`, `OtherWalk`, `OtherStep`: a turn with a seen step a kind 5 `[[turn]]` table; its
  pickups' anchors), the viewer's commands (`Command`), each walk's hero (`walkers`), the heroes
  the start shows (`shown`) and the kingdom's places (`slots`); `turn::record_heroes`,
  `record_walks` (MCL-Q44 (b): a step into sight unseen, one out of sight left out); the
  timeline's `Steps::unseen` and `taken`, `Loop::ai_take` (a computer's pickup in sight with the
  viewer's dissolve, `Then::Ai`), the computer's music (`[map.turn] computer`, `Sound::computer`)
  and its step-end and selection sounds; `Action::Choose`, `Action::Walk` (a record's path's arrows
  in their order), `Loop::idle`, `boxed`, `walks_left`, `Unplayable::Record`, `Untaken`;
  `h3_map_view::director` (the record player); the binary's `paint::for_record`, `record`
  (headless to the end, every screen drawn) and `window --record` with `keys::Playback` (Ctrl+F2,
  F3, F4) and `h3_ui::clock::Clock::set_rate` (common, additive); each hero's walks' arrows
  chained on their own (`record_paths`).
- **Recorded** (a sub-agent, under Wine, seed 1, each twice the same): `game-ai` (Blue's two
  heroes' walks and pickups in Red's sight) and `game-ai-boat` (a boarding, a sail and a landing,
  a pickup from below); the engine scripts the original's walks, their records the same steps.
  Seen: the AI takes no pile from the tile above it; a pile visit costs 101 points; a pile's
  `amount` is in hundreds (500 gave 50 000 gold: 10 used).
- **Found:** `game-ai` exact but its listed gaps (MCL-Q43; two recorded, not read); the frames'
  differences the glass's. `game-ai-boat` waits (`to-record.toml`, `MCL-STUB-2`: T4c2). Golden
  replays viewed to their end headless: `sea-board-s1`, `sea-event-s1`, `sea-landing-guards-s1`,
  `rewards-stone-guard-s1` (a battle skipped, its level-up after), `rewards-university-s1`,
  `dwellings-free-s1`, `banks-declined-s1`, `teleporters-gates-s1`, `garrisons-passage-s1`,
  `towns-capture-s1`; `rewards-stone-guard-s1` in the window under Xvfb (`--check 6`: 27 of 27
  frames read back exact, 26 of 26 changes the headless loop's). Not playable: the duel's, the
  army's and the XL map (`obj.def` templates), `economy-shipyard-s1` (a built boat).
- **Questions:** MCL-Q43 (the computer's points: kind 5's glass), MCL-Q44 (a seen step's hidden
  end), MCL-Q45 (which heroes take a turn), notes in the adventure and server lanes' inboxes.

### M4 T4c2 (2026-11-21; offered whole at the start, no seam: a small task)

- **Read** (design §5.18): `ai_hero_step`'s call of `hero_move_step` (its fifth argument 1, the
  move's last step its second), `hero_move_step`'s `view_shown` at each step's start (0x48071F),
  its boarding and landing (the human's code, no test of the player) and its last step's draw
  (0x481237 to 0x481264); `hero_boards_boat` for a player not human (0x4A10C7: no lifted boat, no
  draw); `hero_lands_from_boat` (no test); `hero_loss_effect`; `view_show_step` and
  `adventure_hero_stop`'s objects' step; `visit_resource_pile`'s hold (+0x104).
- **Recorded** (under Wine, seed 1): `game-ai-boat` twice the same (211 lines, 64 screens: the
  boarding's dissolve 860 to 1380, the landing's frames 2160 and 2210 and its dissolve in 2260 to
  2780, day 3's pickup from below 4460 to 4980); `game-ai` once again, as recorded at T4c (its
  live part's colour cycles 21 for 22: the capture's refresh, the frame at 0 exact).
- **Built** (*provisional* unless said): `Steps::sea` from `record_walks`; a computer's boarding
  (`board_start`, `lose` with `Then::AiBoard`, `ai_animate`, `ai_board`) and landing
  (`land_start`, `ai_land`, `Then::AiLanded`) in sight, out of sight at once (`sea_unseen`); a
  step after a boarding lifts the hero without a path; the objects' steps of a computer's turn
  (`last_drawn`, `ai_hero`'s for the hero before, the turn's end's one step and its deadline);
  `Loop::step` removed. MCL-Q46 (c): `Played::taken`, `NextTurn::taken`, `Loop::take_off`.
- **Found:** `game-ai-boat` exact but its 74 listed gaps (MCL-Q43's to 2260, MCL-Q46's from
  4460), its frames' differences the glass's alone to 1430; `game-ai` with fewer gaps (46 from
  61: the objects' step at 1180 and the second pickup's dissolve's box now the original's); the
  other sequences and situations as recorded (`drawn`); `drawn --record` for the two.
- **Tests:** `a_computers_boarding_and_landing` (the dissolves, the boat sailed and put back, the
  lift without a path, the killfades, Blackout, a boarding without a boat refused, a turn's
  taken objects), `a_computers_next_hero_steps_the_objects`, `record_walks`' boats, `played`'s
  pickup without a step; 454 in the library, 100 % of its lines and regions.
- **Question:** MCL-Q46 (the hero that takes an object without a step), a note in the adventure
  lane's inbox.

### M4 T5a (2026-11-21; T5 cut in two by the user at its start: the reading and the recordings first)

- **Checked first:** the client lane's library entry is complete (E17 M7 T3a to T3c, E17 closed:
  `h3-battle-client`'s `window::Hosted`, `game_seat::GameSeat`, `audio::HostSound`) and `h3-game`
  sets the battle's background and colours (GAM-BAT-011).
- **Read** (design §5.19): `adventure_battle` after its battle (0x4AE2BD to 0x4AE99E: the
  level-ups, Necromancy's sound and box, the loser hero's leaving with `hero_loss_effect`, the
  computer's AI theme, `mobilize_hero(0, 0, 1)`), `manager_run` whole (the adventure manager
  paused by its +0x34, put back with `adventure_full_update`, the mouse-over again and the fade
  in), `battle_start`'s fade out and in, the battle manager's exit (its fade out), the adventure
  manager's exit (the terrain track left playing), `battle_aftermath` whole (the draw's case),
  `monster_fight`'s removal (killfade and §5.7's dissolve), `adventure_mouse_again`,
  `window_menu_set`, `computer_turn_music`; `hero_select`'s music again (MCL-Q47). Map entries:
  five updated, five new; `structs.toml`'s adventure manager +0x34.
- **The harness** (common, additive): `h3map.py`'s sequences take `battle` (`h3adv.fight`'s plan
  or a list), played with the clock running from the battle's screen shown whole, the pointer put
  back, the clock frozen at the result window and the battle taking none of the sequence's time;
  the palette's pending step carried across; nothing kept, written or read from the result window
  until the map is the active manager again; `<ms> battle <winner>` and a `[[battle]]` table;
  `h3adv.fight(…, close=False)`. Two first runs found the battle's dialogs read as boxes and the
  palette's phase after the battle following its real-time length: both fixed as above.
- **Recorded** (under Wine, seed 7, twice the same; `clients/map/to-record.toml`, situations
  `seq-battle-*`): `battle-won` (39 lines, 19 screens: the battle at 350, the map back at 810, the
  Peasants dissolved to 1330), `battle-mine` (58, 19: the question, the battle at 1310, ADVEVENT
  85's box with the map back at 1810, MINEEVNT's, the flag), `battle-retreat` (46, 18: the
  retreat, the hero dissolved off to 1330, the town selected and the view moved); each frame at 0
  exact. No viewer change: T5b plays them.
- **Question:** MCL-Q47 (no terrain track asked for after a battle on the hero's own terrain; (a)
  as the original recommended, checked by ear at T6).

### M4 T5b1 (2026-11-21; T5b cut in two by the user at its start: the hosting first)

- **Asked at the start:** the cut (T5b1 the window's hosting, the sound on `HostSound`, a golden
  replay's battles; T5b2 the fades, the result applied, MCL-Q47 by ear and T5a's three compared)
  and the crate structure: the user took a dependency of `h3-map-view`'s binary on
  `h3-battle-client` (and `h3-view`), not a binary over both libraries (design §5.20).
- **Built** (*provisional* unless said; design §5.20): `Move::battle` (a walk's `BattleStarted`),
  `Loop::battles`, `Replay::battles`; `engine::Command::Battle` after an answer that began one,
  `director::Directed::battles` (at its box's release, or where an answer with no box is passed
  over); the binary's `battles` (the record's `GameRecord::battles`, each loaded with
  `Ground::load_battle` and run as a hosted element in the map's window before the map's frame
  at its millisecond, the map's clock on from there), `audio::Player` on `HostSound` (`lend`,
  `take_back`; the tracks added to it once decoded), `paint::Setup::battles`, `record`'s line of
  the battles handed over; the purity test's binary-only list.
- **Checked:** every battle of the attacking side's golden replay records handed over headless (12
  in 10 records); `rewards-stone-guard-s1` and `dwellings-guards-s1` in the window under Xvfb
  (`--check`: every battle closed, every map frame read back exact, every change the headless
  loop's). Not handed over: a defending battle (`MCL-STUB-3`).


### M4 T5b2 (2026-11-21; kept whole by the user at its start)

- **Asked at the start:** a cut at the battle's edge (the switch first, or the result first); the
  user kept the task whole.
- **Built** (*provisional* unless said; design §5.21): `sequence::BattlePlan` (the harness's
  `battle`, accepted); `engine::Battled` (`Visit::battle`), `visits::Told::Fight` and
  `Told::Guarded`; `timeline::Fight`, `AfterBattle`, `Found::fight` and `Found::battles`,
  `Answer::fight` and `Answer::flag`, `Switch` (`Draw::switch`), `Draw::left`, `State::battle`
  (the timeline's `battle` line), `Action::BattleClosed`, `BATTLE_CURSOR`, `FADE_MS`,
  `FADE_BLACK`; the loop's switch (`switch_out`, `switch_step`, `switch_back`, `map_back`) and the
  result (`after_battle`, `left_map`); the layout's combat cursor (`battle_def`, `battle_frame`,
  `battle_hot`); the catalogue's `map.find.mine-guards` (ADVEVENT 85); the binary's painter's
  `switched` and `battle_frame`, `battles::first_frame`, the battle's last frame kept from its
  hosting, the window's `BattleClosed` after it, the sequences run keeping no frame from the
  result's OK until the map is back.
- **Recorded:** T5a's three moved into `sequences.toml` as scripted games; their situations as the
  other scripted games' (the original's random starting bonus had widened the hero's explored
  circle: the gold bonus, the resources written, a second hero to show them); `battle-mine` an
  abandoned mine under engine seed 5 (100 Troglodytes, written for the original; 173 reached a
  level-up). Each recorded twice the same under Wine; the battle's starting stacks equal the
  record's.
- **Checked:** the three timelines as recorded (no gap); the frames exact but the known
  differences (`timelines/battle-*-frames.txt`: MCL-Q36's morale icon, MCL-Q49's battle frames,
  the hero's pose at the attack, the panel's buttons after a retreat, MCL-Q48's MINEEVNT box);
  `h3map.py drawn` over every situation and sequence; `check.sh`; the coverage of `h3-map-view`.
- **Questions:** MCL-Q48 (an abandoned mine's resource; the adventure lane's inbox), MCL-Q49 (a
  map battle's obstacles; the battle lane's inbox).

### M4 T6 — M4's closure (2026-11-23; the user: no cut, the work spread over sub-agents)

On `map-client/e24-m4-t6`, `master` at f1c4a53 (#1106 merged; CI and Clients green). Original data
staged from the user's repo folder (the PC's shell up this time): the 64 files, the manuals and
help files, the LODs and executables SHA-256 checked; the setup's baseline stopped at its coverage
step (fmt, clippy, doc tests, deny clean); the harness set up beside it.

- **Asked at the start:** the cut (three parts as M3's T7; or the reviewers and fixes apart from
  the checkpoint). The user: no cut, one session, the work spread over sub-agents.
- **Reviewed:** six fresh reviewers, three at a time, reading only (R1 the turns, R2 the message
  boxes, R3 the map changed by a visit, R4 a record's start, R5 the window as a record's host and
  its battles, R6 the harness, the data and the rest): 64 findings, about 20 bugs, most only on
  engine records no golden replay had reached (the branch `map-client/e24-m4-t6-review`,
  `docs/lanes/map-client/review-m4/`, not merged).
- **Fixed** by seven fixers, one at a time in the clone (the harness's beside one), each finding
  with a test or a reason (their reports' tables: the review branch's notes, and the commits):
  F4 (32787c1, a record's start: the game's classes for the hover, the file's art for `render
  --record`, a cut record refused, `--check`'s options and verdict); F6 (f7ab3fe, the harness: a
  shipped start's Gold bonus, the disk check's pre-flight, the staging's refusals, kept live parts
  with their run, the engine's binaries checked up front, the labels' alphabet); F5 (132f657, a
  record's battles: a monster fought from beside, the switch waited through and closed, a mine's
  battle handed over once, the outcome's result, the battle's commands taking no map time, the
  window's focus and settings across a battle); F1 (ec9be90, the other players' turns: a viewer
  not first to act, a move per command, the record's own sight, the day's lists, a computer's
  flags, removals and losses); F2 (577b953, a record's visits each its own find, its own
  level-ups, Pandora's Box's rewards, the record player's choices; our own words in the tests,
  ADR-0009, the inbox's labels item done); F3 (2c4b2e8, a record's flags in their owners' colours
  and raised alone, its boats told after its start, a landing with no facing refused, Summon
  Boat's dissolves' status window, the gate's M4 names); F7 (3cb6d6f, the golden records to their
  end: a walk short of its path, its own path, an object taken alone, no days without a town on a
  record `MCL-STUB-4`, an unseen hero's landing boat; the library's coverage at 100 %).
- **Found on the way:** two "viewed to its end" verdicts were false (the mine records stuck in
  the switch, 11 inputs never given); the record player walked onto a monster instead of fighting
  it from beside, so T5b2's switch never ran on a record; a viewer not first to act saw every turn
  one late; 20 of the 66 golden player records were refused (the client's own seven days without
  a town, walks the engine stopped short, a path taken from an earlier walk). All 66 now play to
  their end headless.
- **Asked:** MCL-Q50 (the date's parts), MCL-Q51 (a mine's guards' battle not won), MCL-Q52 (the
  ambient loops after a battle), MCL-Q53 (a hero coming into sight), MCL-Q54 (Pandora's Box's
  untold rewards), MCL-Q55 (a flagged mine's production), MCL-Q56 (`resour82.def`'s frames); none
  blocks. Other lanes' inboxes: the client lane's (the hosted battle's `BUTTON.WAV` not found: the
  layout's name with its extension; the harness's disk check), the adventure lane's (the disk
  check; MCL-Q53, Q55 and `MCL-STUB-4`'s facts for the record).
- **The user at the checkpoint** (2026-11-24): the review list (M4's items and T6's additions by
  fixer, in the archive) read first, asked again at M5 T0; MCL-Q50 to MCL-Q56 answered at M5 T0;
  `MCL-STUB-4` kept (no days without a town on a record: the viewer shows only what the record
  holds; the warnings come back as the engine's events with E10).
- **The hand test kit** (the user's `clients/out/win/`: `map-m4.cmd`, `map-m4-notes.txt`, `m4/`
  with the Windows build, its map data, two golden records and their maps): an interim build at
  132f657 put there mid-session, then the final one; five parts (two golden replays with their
  battles; Hoard's sound against the original; the original's music after a battle won, MCL-Q47;
  the HORSE pairs, MCL-Q11 (c)). The user's notes are read at M5 T0.
- **Checked** (after the last fix): `h3map.py drawn` over every situation and sequence (56 and
  all, the same; `battle-retreat`'s three frames recorded again: `frames --only battle-retreat`
  against the original gives 0 differing where 16 were listed, F1's retreated hero off the
  minimap); `start-jared` and `start-good-witch` captured again with the Gold bonus (0 differing),
  their `engine` expectations moved (the bonus the engine lacks, Jared's mana from the level-ups'
  draw); every golden replay's player record (66) headless to its end; `dwellings-guards-s1` in
  the window under Xvfb (`--check 40`: both battles closed, 116 of 116 frames read back exact);
  `clients/scripts/check.sh --no-coverage` (all gates); `h3-map-view`'s coverage at 100 % (F7);
  `cargo xtask verify --scope map-client --summary --with-original-data` (no failing step after
  a trap's size written in bytes, not as a hex address).

### M5 T0 — M4's review list and questions, M5 refined with the user (2026-11-24)

On `map-client/e24-m5-t0`, `master` at 73b4fc7 (#1141, #1140 merged). Docs only; nothing built
or captured. From the user's machine (its bridge briefly up): `Heroes3.exe`, `extracted.tar`,
`maps.tar` (SHA-256 as the PC's) and the manuals staged for the refinement's reading.

- **The hand test was not run yet:** `map-m4-notes.txt` is the blank template, no `map-m4.txt`
  beside `map-m4.cmd`. Its notes are read at M5 T1's start (the user's choice: all of T0 now, not
  a seam before the refinement).
- **M4's review list** (45 items): the user asked for the important ones one by one and a summary
  of the rest. Asked: the record player (T4c: as built); the random picks taken in turn (in turn
  for recordings and records, play drawing them from the client's seeded generator); speed 4's
  20 ms (kept); the library's engine dependencies (kept; one record reader shared with the client
  lane at M5); the battle switch's two unasked frame differences (the hero's pose at the attack,
  the panel's buttons after a retreat: fixed in M5); the other players' turns (kept; the
  original's computer turn read once play shows one). The other 39 accepted in a summary. The
  archive's list carries the acceptance.
- **MCL-Q50 to MCL-Q56**, one by one: Q50 (b); Q51 (b) then (a); Q52 (a), (b) meanwhile; Q53 (a),
  (c) meanwhile; Q54 the user's rule, **the engine and the server the only authority for a
  client, reading files at run time phased out**, then asked how far: facts first, the pictures
  later, "when a server generates a random map, it may not reveal the content to the client until
  the player discovers them"; Q55 (a), (b) meanwhile; Q56 (a) when met, (b) meanwhile.
- **Read for the proposal:** `h3_protocol::session::game::GameSession` (seats, `play`, `query`,
  the clock: `GameOptions::clock`, `elapse`, `deadline`; `computer` plays unowned battle sides
  only, "computer players on the map E13's", now E31's); `h3-server`'s whole game (every seat a
  connection's); E31's increments (increment 2, "a turn that ends"); E27's hand-over
  (`begin::Game`, `save::Loaded`, `CLT-STUB-13`; the shell's `shell::run` owns its window);
  `CLT-STUB-4`, `-5`; design §5.1's human start in a hot seat (the covered view, GENRLTXT 14) and
  the map's `turn_time_expired_end` (GENRLTXT 202), GENRLTXT 627 to 630 (records counted, a
  quoted one across lines as one; 352 "It's %s's turn." too); the harness's `hot_seat`.
- **The user's decisions:** the milestones split (M5 hot seat on the adventure map; M6 the hero's
  screens and visit windows; M7 the town screen; M8 live play); the player's program with **the
  shell on top** ("unless architectural reasons oppose it strongly; least effort is no such
  reason"): told of the package cycle (the shell in `h3-battle-client`, which the map's battles
  need) and `h3-map-view`'s pure library, the user chose the shell's own crate on top over a
  trait of the shell's with the program's `main` in the map's package; `h3-map-client` the map's
  adapters library; a computer colour's turn ended at once until E31; the exit as recorded parts
  plus a whole two-human game; M5's tasks T1 to T8 queued as proposed.
- **Handed on:** the client lane's inbox (the shell's crate, the shared record reader, the stubs
  M5 lifts); the adventure lane's (the user's rule; MCL-Q53, Q54, Q55; the map's texts in play);
  the server lane's (MCL-Q53's `SRV-STUB-3`, Q55's `SRV-STUB-10`); the AI lane's (a computer
  colour's turn ended at once until E31's increment 2).

### M5 T1a — the hero on its tile through a battle, the panel after a retreat (2026-11-24)

On `map-client/e24-m5-t1`, `master` at 150da0a (#1152, #1153 merged; CI green). Original data
staged from the user's repo folder at the start (58 files: the four LODs, both executables, the
five DLLs, the sound and video archives, both tars, the 40 MP3s; `games/` not; the four manuals
and three help files too; SHA-256 checked, the MP3s as one digest of their hashes); the PC not
needed after the first minutes. The setup's baseline stopped at its coverage step by a VM restart (fmt, clippy, the doc
tests and deny passed), CI's green `master` standing in. A ballast file kept the game's disk
check in range (TRAPS).

- **The cut** (the user, at the start, the recommended one): T1 in three, T1a M4's two frame
  differences, T1b MCL-Q51, T1c MCL-Q52. The user's M4 hand-test notes still the blank template
  (`map-m4-notes.txt`, as at T0): they wait for T2 (the queue's own fallback).
- **Found:** the "moving pose" was no pose: the original's hero at the attack is a unit brighter
  over its whole picture, drawn on its tile without the lifted hero's pass 9 blend (the frame
  check against the viewer with the hero on its tile: the same picture; every walking frame
  tried first, none closer). `adventure_hero_stop` (0, 1) at every battle's start puts it there
  (`monster_battle` 0x4AC593, `adventure_battle` 0x4AD54E); the map's return keeps it there with
  the mouse hidden through the fade in (M4's 710 and 760 differed by the arrow too, 87 pixels,
  counted in with "a flag column"); the fade in's end shows it lifted again (a won battle's
  `mobilize_hero`; a retreat's lift before its draw 0x4AE6BB not read, *provisional*). After a
  retreat `hero_leave_game` redraws neither sleep nor cast a spell: both drawn enabled until
  the next choice (1083 pixels from 810 to 1265).
- **Built** (design §5.22): `Loop::battle_stop` (the hero put back on its tile, the view drawn,
  then the objects stepped when due: one step at `battle-mine`'s 1060, where M4's viewer stepped
  at the box's close and drew after it), the mouse hidden at `map_back`, shown and the hero
  lifted again (the view drawn) at `after_battle`'s start, a retreated hero's last view drawn
  lifted before it leaves, `Buttons::left` (sleep and cast a spell kept until `left_map`'s
  choice). `adventure_hero_stop`'s map entry read whole.
- **Recorded:** `battle-won`, `battle-retreat`, `battle-mine` again, each as recorded; their
  frame lists now M4's known differences only (MCL-Q36's morale icon, MCL-Q49's battle fade in,
  MCL-Q48's MINEEVNT box), `battle-retreat`'s dissolve exact.
- **Checked:** the crate's tests (two new: the hero on its tile through a battle, the kept
  buttons); `h3map.py drawn` over every situation and sequence (the three alone changed),
  `drawn --record`; `check.sh` (fmt, clippy, deny passed; its coverage found one region, the
  lift's test of a hero, made unconditional: the crate's coverage then 100 %); the scoped
  verify.

### M5 T1b — MCL-Q51: a mine's guards' battle not won, recorded and read (2026-11-25)

On `map-client/e24-m5-t1b`, `master` at e2e15c7 (#1160 merged; CI and Clients green). Original
data staged from the user's repo folder at the start (the four LODs, both executables, the five
DLLs, the sound and video archives, both tars, the 40 MP3s, `games/`; SHA-256 checked against the
PC's); the PC's bridge gone soon after. The setup's baseline stopped at its coverage step by hand
(fmt, clippy, the doc tests and deny passed), CI's green `master` standing in; no disk ballast
needed (the free space in the game's band).

- **The hand test's notes:** `map-m4-notes.txt` still the blank template; they wait for T2 (the
  queue's fallback, now in T2's scope).
- **Recorded** (MCL-Q51 (b)): `battle-mine-retreat`, `battle-mine`'s situation with `battle =
  "retreat"` and the engine's script `move`, `answer offer accept`, `battle retreat` under
  `engine_seed` 5; twice the same (timeline, live part, frames). The battle at 1310 (winner 1, the
  guards' two stacks 50 Troglodytes and 50 Infernal Troglodytes as `battle-mine`'s), the hero gone
  at 1810 and dissolved to 2265, the second hero chosen at 2330; no box, the mine unflagged.
- **Read** (MCL-Q51 (a), design §5.23): `visit_mine` ends the visit at a winner not the hero
  (`monster_battle`'s at 0x4A3AE5, `adventure_battle`'s at 0x4A3B59): the count left written back,
  no box, no flag; `monster_battle` selects the hero again after every outcome and returns the
  winner unchanged; a draw's `battle_aftermath` runs the beaten hero's calls one for one, so a
  draw at a mine shows as the retreat recorded. After a draw the original keeps slot 0's creature
  with a count of 0, so the next visitor is asked again: the adventure lane's inbox (the engine
  leaves none).
- **Built:** nothing new (M4 T6's `AfterBattle::of_guards` as read and recorded): comments, the
  binary's `MCL-Q51` stub marker removed, the sequence in the two pinning tests and the battles'
  tests (`a_mine_s_guards_not_beaten_show_no_box_and_raise_no_flag` now on the recording).
- **Checked:** the viewer's timeline from the engine's record as recorded (no gap); the frames
  M4's known differences until 1760, exact from 1810; `h3map.py drawn` over every situation and
  sequence (the new one alone `new`, recorded); the crate's tests.

### M5 T1c — MCL-Q52: the ambient loops after a battle (2026-11-25)

On `map-client/e24-m5-t1c`, in the same sandbox and session; begun on T1b's branch, rebased onto
`master` at c675568 once #1165 merged (with #1162, #1164: `drawn` all the same again).

- **Read** (MCL-Q52 (a), design §5.24): the hand-over's `effects_stop_all` ends the loops but
  leaves the manager's four slots; nothing at the map's return clears them; `adventure_battle`'s
  end (0x4AE8F3, every outcome) calls `mobilize_hero(0, 0, 1)`, whose `hero_select` scans with the
  stop flag (0x4180FF): each slot stopped and emptied, the loops found again started anew, the
  music only on another terrain. 0x4AD62E, which §5.22 named, is the battle without a human's.
  (b) not needed.
- **Recorded:** `battle-mine` as a sound run (a scratch sequence, `sound = true`): the mine's loop
  at 100, its channel free at the hand-over (a click on it at 1500), the loop again at 1810, the
  fade in's end.
- **Built:** `Loop::after_battle` calls the model's `Sound::select` with the lift when the hero
  stays on the map (*provisional*: before a level-up's dialogs, which the original shows first);
  a hero that leaves restarts nothing there (the next choice's lift does). Two tests
  (`a_won_battle_s_lift_restarts_the_ambient_loops`, `a_retreat_s_loops_restart_at_the_next_choice`).
- **Checked:** the window under Xvfb with `--cues` on `battle-mine`'s engine record (its loop
  restarted at the fade in's end, 138 of 138 frames read back exact) and on `dwellings-guards-s1`
  (both battles closed, 117 of 117 exact; its won battle is a plain cut, no lift: its loop stays
  silent until the next day, carried into T6's scope); `h3map.py drawn` (every situation and
  sequence the same); the crate's tests; `check.sh`; the scoped verify.

### M5 T2a — `h3-map-client`: the binary's adapters as a library (2026-11-25)

On `map-client/e24-m5-t2a`, from `master` at 6498273 (#1167, T1c's, merged).

- **Asked at the start:** the cut (the user): T2 in three, T2a the move alone (no API, nothing
  changed), T2b play's entry for the shell with the client lane's move queued, T2c the shared
  record reader. The M4 hand test's notes (`clients/out/win/map-m4-notes.txt`) still blank.
- **Built:** the new package `clients/crates/h3-map-client` (this lane's, `clients/crates/h3-map*`):
  `h3-map-view`'s binary's twelve modules and `render/tests.rs` moved by `git mv` into its
  library, unchanged (the six commands `gate`, `render`, `records`, `sequences`, `situation` and
  `window` public, their parts `audio`, `battles`, `files`, `paint`, `sounds` and `texts`
  private); the binary `h3-map-view` (`src/bin/h3-map-view/main.rs`, its usage and dispatch) one
  host of it in the new package, as `h3-view`'s went into `h3-battle-client` (E17 M7 T3a1): Cargo
  forbids the cycle a binary in `h3-map-view`'s package would make. `h3-map-view`'s manifest keeps
  its pure dependencies (`h3-files` a dev-dependency for `tests/engine.rs`), its `tests/purity.rs`
  no binary-only list (as `h3-view`'s copy); the default folders (`files.rs`, by
  `CARGO_MANIFEST_DIR`) resolve the same from the sibling folder. `check.sh` leaves
  `/crates/h3-map-client/` out of the coverage gate as `/crates/h3-battle-client/` (M5's review
  list); the builds' lines `-p h3-map-client` (`clients/README.md`, the runbook, `h3map.py`).
- **Checked:** the crates' tests (the 25 moved ones all in the library's run, 502 of the pure
  library's); every golden adventure replay's player records (118 replays, 236 records) with this
  branch's binary and `master`'s: the same 205 played to their end, every timeline identical,
  the same 31 stopped (the duel's and the army's maps, and 7 records of replays added since M4 T6:
  the record player's, the queue's inbox); `h3map.py drawn` (56 situations and every sequence the
  same); the window under Xvfb on `dwellings-guards-s1`'s record (`--check 45`: both battles
  closed, 142 of 142 frames read back exact, every change the headless loop's; its cues those of
  `master`'s binary, its "not found: BUTTON.WAV" line too); `clients/scripts/check.sh`; the scoped
  verify.
- **Found on the way:** a run of the branch's binary while the tree held `master` (no
  `crates/h3-map-client/`) finds no `clients/map`: the default folders go through the package's
  own folder (`h3-map-client/../../map`), as they went through `h3-map-view`'s; `--map` and
  `--install` (the Windows builds' way, TRAPS) do not.
