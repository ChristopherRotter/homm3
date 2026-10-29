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
