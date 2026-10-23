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
  owner; none in the shipped maps; built as recommended: the client the object's own, the
  comparisons the original's).
