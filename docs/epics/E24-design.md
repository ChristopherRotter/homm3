# E24 design notes

Working detail of E24: the code readings, formats and layouts of the adventure map client.
Decisions live in the epic file ([`E24-map-client.md`](E24-map-client.md)), done tasks in full in
[`E24-log.md`](E24-log.md). Addresses are `Heroes3.exe` (GOG Complete, SoD 4.0; `docs/exe/`),
each with a map entry; "the manager" is the adventure manager (pointer 0x699268, `structs.toml`
`adventure_manager`). Code readings are one source each (`docs/04-rules-and-sources.md` §2): every
point names VCMI 1.7.5's agreement or difference (§1.7), and the captures of T3b to T5 confirm.

## 1. The view's tiles and the moment (M1 T1a, 2026-10-23)

Path read: the manager's procedure (0x4087B0, its idle step at 0x408A88) draws the view with
0x40F3F0 from the view's top-left (manager +0xE4); the puzzle map's mode (0x6989F4, set by the
Obelisk's 0x41A7F0) and the covered mode (0x699538, §1.5) are clear in play. Not read: the network
branches at 0x40F40E and 0x40F426, the debug frame rate (+0x3C), the panels' redraw at the end.

### 1.1 The draw and its passes

`adventure_view_draw` (0x40F3F0; the top-left x, y, level, then two flags) first runs the tick
0x4ED490 (§1.6). Without its first flag and while 0x6989C0 is 0 it draws nothing (inferred: the
map not shown, set by the manager's construction and cleared around unseen moves, 0x480732).
Then **one pass after the other, each over the whole view**: rows j = −1 … 17 (outer), columns
i = −1 … 19 (inner), each call taking the tile (vx + i, vy + j) and its place (i, j):

| # | Pass | When | Draws |
|---|---|---|---|
| 1 | 0x412900 | always | terrain, the map's edge outside it (§1.3, §1.4) |
| 2 | 0x411B80 | not the puzzle | rivers |
| 3 | 0x411D60 | not the puzzle | roads |
| 4 | 0x412470 | not the puzzle | the flat objects (§2.2) |
| 5 | 0x411590 | always | the shadows: objects', then heroes' and boats' (§2.2) |
| 6 | 0x411F50 | a path shown (+0x4C), not the puzzle | the path arrows' shadows, `adag.def` (+0xB0) by the arrows of +0x48 (§2.2) |
| 7 | 0x410C00 | always | the bodies: objects by layer, heroes and boats among them (§2.2) |
| 8 | 0x4120B0 | a path shown, not the puzzle | the path's arrows (§2.2) |
| 9 | 0x47FC80 | manager +0x1EC (a hero moving; cleared when the current player has no selected hero) | the moving hero's frame step and sounds (§2.2; M2) |
| 10 | 0x412220 | not the puzzle | the unexplored tiles (§1.5) |
| 11 | 0x40F2F0 | always | the view's four corner gems |

So each layer lies over every lower layer of every tile: a road's lower half over the next row's
river, the shroud over the objects and the path, the gems over the shroud. **The corner gems**
(0x40F2F0): `agemul`, `agemur`, `agemll`, `agemlr` (+0xB4 … +0xC0), frame = the viewing
player (0x4CEA20, a colour 0–7), their 46 × 46 at (6, 6), (556, 6), (6, 508), (556, 508), through the plain
drawer (`def_draw_plain`, runs skipped, not mirrored): part of the view, not of the frame picture.

### 1.2 Where a tile goes

- Tile (vx + i, vy + j) is drawn at **(32 i, 8 + 32 j)** plus manager +0xF4, +0xF8, which the
  construction sets to 0 (0x406DF0). So the scroll moves the view by tiles (§3.4); **corrected by
  M2 T1b:** a walk's animation writes them, the map sliding by pixels under the walking hero
  (`hero_walk_animation` 0x480380, §3.5).
- Every pass clips each 32 × 32 tile to **x 8 … 599, y 8 … 551** (592 × 544): columns 0 and 18
  show 24 pixels each (x 8–31, 576–599), columns 1–17 whole, rows 0–16 whole; column −1 and 19,
  row 17 draw nothing, row −1 only a road's lower half. Its constants (8, 600, 544 and the 8 added
  to y; the loops' 19 and 17) are immediates in each pass: what an HD layout changes.
- The idle step shows the rectangle (0, 8) to (608, 552) (0x40F270, 0x408AF8), so the frame's
  8-pixel columns beside the view are shown with it.
- **E17's formula corrected:** E17 design §6.3 (from the harness's clicks) put tile (x, y) at
  (8 + 32 (x − vx), …); its left edge is at 32 (x − vx). The tile under the mouse (manager +0xE8,
  0x40DEB0) is (vx + mx div 32, vy + my div 32) of the mouse message's position (the procedure
  passes its +0x10, +0x14, 0x408A57): if those are screen coordinates, the bottom 8 pixel rows of
  each drawn tile report the tile below (open point for M3).

### 1.3 Terrain, rivers, roads

The tile record (`map_item`, 0x26 bytes, manager +0x5C's +0xD0) holds terrain +0x4 and its
picture +0x5, river +0x6 and picture +0x7, road +0x8 and picture +0x9, and in +0xC the mirror
bits (map_tiles_load). Each layer is group 0's frame number *picture* of a DEF the manager loads
once (0x406FD0) from a name table, drawn through 0x47BF50 into `draw_frame_raw` (encodings 0 and
2: raw pixels in the DEF's 16-bit palette +0x20, runs skipped):

| Layer | DEF (manager field, names' table) | Frame | Mirrors (+0xC) | Offset |
|---|---|---|---|---|
| terrain 0–9 | +0x60 + 4 t: `dirttl`, `sandtl`, `grastl`, `snowtl`, `swmptl`, `rougtl`, `subbtl`, `lavatl`, `watrtl`, `rocktl` (0x65F55C) | +0x5 | bit 0 left-right, 1 top-bottom | — |
| river 1–4 (0 none) | +0x88 + 4 r: `clrrvr`, `icyrvr`, `mudrvr`, `lavrvr` (0x65F588) | +0x7 | bits 2, 3 | — |
| road 1–3 (0 none) | +0x9C + 4 r: `dirtrd`, `gravrd`, `cobbrd` (0x65F59C) | +0x9 | bits 4, 5 | 16 pixels down |

- A road covers the lower half of its tile and the upper half of the one below; on the map's last
  row (y = side − 1) only its top 16 rows are drawn, so nothing reaches past the map. The pass
  takes the 16 rows off what the view's cut (§1.2) left (0x411ED3), so at the view's bottom row,
  where that cut already left 16, it draws nothing (MCL-Q2; T3b1 read it).
- Rivers and roads skip tiles outside the map; no pass checks a picture against the DEF's frame
  count (the viewer refuses such a map, *provisional*).
- A run in a terrain frame would leave what the buffer held. **None has one** (T2's gate on the
  user's files, §1.9): every terrain frame is encoding 0; the rivers' and roads' are encoding 2
  with runs of index 0 only (their transparent parts), so the raw drawer skips no shadow.

### 1.4 The map's edge

The terrain pass draws a tile outside the map (0x4B1330 false; the bounds 0x6783C8, 0x6783CC)
with `edg.def` (+0xAC), not mirrored: the corners 16 (−1, −1), 17 (W, −1), 18 (W, H), 19 (−1, H);
the edges top 20 + (x and 3), right 24 + (y and 3), bottom 28 + (x and 3), left 32 + (y and 3);
every tile further out ((x + 16) mod 4) + 4 ((y + 16) mod 4). The bottom edge tests x against the
height (0x412B4D), harmless while maps are square (MCL-Q1: the viewer tests the width, the user,
2026-10-23).

### 1.5 The unexplored tiles

The pass (0x412220) covers a tile of the map whose vision word (tile_vision_word) lacks the
viewer's bit (0x69CCBC = 1 << the viewing player 0x69120C, set as a human's turn starts, 0x4C7323);
an explored tile gets nothing (the original has no fog of war over explored tiles).

- **The mask** (0x40F8C0): a bit for each of the eight neighbours **not** explored: 1 north, 2 east,
  4 south, 8 west, 0x10 north-east, 0x20 south-east, 0x40 south-west, 0x80 north-west; neighbours
  outside the map count as unexplored (set from the tile's side of the map, never read).
- **v = 0** (the table 0x65F694 [mask], below; only mask 255 gives 0): `tshrc.def` (+0xC4) frame
  ((85 x xor 85 y) div 64) and 3 (the pass draws it whenever 0x40F8C0 returns 0, 0x41237F; M1 T6b,
  the reviewer: the viewer takes the table's 0 so too, not the mask 255).
- **Any other v:** v ≥ 100 means mirrored left-right and v − 100; v of 1 or 5 on an odd x takes
  v + 1, v of 3 on an odd y takes 4; `tshre.def` (+0xCC) frame v − 1 (34 frames used).
- **Drawn twice** (0x47BFF0): `draw_frame_raw` (the raw pixels in their colours), then 0x47E820
  over the runs of encoding 2's layout: codes 1 and 2 the border shadow ⌊c/4⌋ + ⌊c/2⌋, 3 and 4 the
  body shadow ⌊c/2⌋ per channel (E17 design §3.3's masks), the rest untouched; it mirrors both
  ways. A hidden tile beside an explored one so shows its terrain and objects through the piece.
- **Which hidden tiles get an edge piece** (the adventure lane's T8b1 question): every unexplored
  tile with an explored tile among its eight neighbours, exactly ADR-0016's fringe; every other
  unexplored tile gets the full piece. Only the viewer's own bit is tested (a team's bits are set
  together when tiles are revealed, E08).
- **The covered mode** (0x699538, set and cleared by 0x4CA5B0 and 0x4CA840 around a turn's start
  and after a battle; inferred: the hot seat's covered map, M5): the view draws from (0, 0) and
  every tile of the map, and below it, gets the full piece. Not read further.

The table, mask = 16 × row + column:

```
 11   7   8 129   9  10 128  33 108  29  30  32  28 133  34  22
 11   7   8 113   9  10 128 126 108  29  30 131  28 133  34 120
 11   7   8 129   9  10 112 127 108  29  30  32  28 133 125 121
 11   7   8 113   9  10 112 103 108  29  30 131  28 133 125 117
 11   7   8 129   9  10 128  33 108  29  30  32  12  27  25  21
 11   7   8 113   9  10 128 126 108  29  30 131  12  27  25 118
 11   7   8 129   9  10 112 127 108  29  30  32  12  27   1  19
 11   7   8 113   9  10 114 103 108  29  30 131  12  27   1 116
 11   7   8 129   9  10 128  33 108  13  30  31  28  26  34  20
 11   7   8 113   9  10 128 126 108  13  30   5  28  26  34  24
 11   7   8 129   9  10 112 127 108  13  30  31  28  26 125  18
 11   7   8 115   9  10 112 103 108  13  30   5  28  26 125 123
 11   7   8 129   9  10 128  33 108  13  30  31  12   3  25  17
 11   7   8 113   9  10 128 126 108  15  30   5  12   3  25  23
 11   7   8 129   9  10 112 127 108  13  30  31  14   3   1  16
 11   7   8 115   9  10 114 103 108  15  30   5  14   3   1   0
```

### 1.6 The moment

- **Terrain and rivers never change frame**; they animate by their palettes. The tick 0x4ED490
  (at the start and end of every view draw and from 69 other sites) runs once the clock
  (clock_ms, timeGetTime) reaches 0x6989B4: it sets that to now + 200 ms (110 while 0x698A18 is 1
  or 3, a battle's value, 0 on the map) and, while 0x67F574 is set (1 at start; the visits'
  dialogs clear and restore it), rotates by one step (`palette16_rotate` with −1: each entry takes
  its lower neighbour's colour, the first the last's) these ranges of the DEFs' 16-bit palettes:
  water 229–240 and 242–253, lava 246–254, the clear river 183–194 and 195–200, the mud river
  228–239, 183–188 and 240–245, the lava river 240–248; the icy river none. One tick rotates them
  all, so **one count k** (mod 36) gives every animated tile's colours: the ticks since the DEFs
  were loaded (whether a reload resets them: not read). A late tick is not caught up.
- **Objects:** the frame of an object's DEF group is (the object's phase + manager +0x100) mod the
  group's frames (0x410F3E, 0x4127B8); the phase, the 12-byte object record's +0xA, is
  random_between(0, 255) per object at load (0x504470, called per object by
  map_objects_place_all). +0x100 counts 180-ms steps: after each idle redraw (0x408B26) and in the
  redraw helpers (0x40F270 and eight sites like it), once the clock reaches frame_deadline 0x698998
  and +0x104 is clear, +0x100 grows by one and the deadline by max(180, the lateness). The draw
  comes first: after an idle step the screen shows +0x100 − 1. Which group a hero, boat or flag
  uses: T1b, T5.
- **What a still needs:** k (or the palettes read from memory), +0x100 at the last draw and every
  object's phase (memory): T3b settles how the harness gets them and whether its virtual clock
  drives clock_ms here.
- **The screen lags the palettes** (T3b2, the idle step 0x408A88 read and a capture): the view is
  redrawn only once the clock reaches frame_deadline (0x698998, every 180 ms), while the tick
  rotates the palettes whenever one of its 71 callers polls it after its time; between a tick and
  the next redraw (up to 180 ms) memory holds k and the screen shows k − 1 (a capture saw it:
  42 035 pixels of water differed by that step). The screen's water and lava change at the
  redraws, each showing the ticks before it (the redraw's own tick at its start included): M2's
  animation follows the redraws, not the ticks.
- **The rivers can trail the terrains by a step** (T6a, the inbox's half tick: about one still
  in seven found `lavatl` and `watrtl` one step ahead of the rivers for as long as it read). The
  manager loads the DEFs kind by kind (adventure_manager_start 0x406FD0: the terrains, then the
  rivers, the roads, …) with a step of the loading screen's progress bar between the kinds
  (load_progress_step 0x4ED2A0), which shows the bar through screen_show_rect 0x602BD0, and that
  runs the tick before and after. The map's load runs in real time, so when a tick falls due at
  the step after the terrains, it rotates their palettes alone (the rivers' fields are still
  empty, which the tick skips), and the rivers start one step behind for the whole game; within a
  kind there is no step, so all terrains share one k and all rivers one, k or k − 1. The screen
  draws each DEF with its own palette: water and lava at k, the rivers at k − 1. So a still has
  two steps, the terrains' and the rivers' (`h3_map_view::live::Moment`'s `cycles` and
  `river_cycles`); the harness reads each kind's k from memory (`h3map.py`'s `cycle_step`, which
  refuses any other difference) and `live.toml` gives both. The viewer's own clock keeps them
  together (`Moment::at`): the split is an accident of the original's load time (MCL-Q5, answered
  (a): in step, the comparisons alone take the two). The captures (T6a): three stills of 56 with
  the rivers behind (`rivers-lava`, `rivers-mud`; T6a's sandbox loaded faster than T4b's), each
  exact; drawn in step, the first differs in 3 655 pixels of its lava river.

### 1.7 VCMI 1.7.5 compared

| Point | VCMI (`client/mapView/MapRenderer.cpp` unless named) | |
|---|---|---|
| Passes | per tile: terrain, river, road, objects, path, overlay, fog (`MapRenderer::renderTile`); the original layer by layer over the view | same inside a tile; across tiles T1b |
| Place, clip | game area (7, 7) to (601, 553) (`config/widgets/adventureMap.json`), tiles placed from a pixel centre (`MapViewModel`) | differ: one pixel on each side, the original's fixed grid (§1.2) |
| Terrain, river, road | DEF by type, frame = picture, mirrors from bits 0–1, 2–3, 4–5 (its `verticalFlip` mirrors left-right); a road's halves 16 pixels down; none past the last row | agree |
| Edge | corners 16–19 and edges 20–35 as the original; further out \|x\| mod 4 + 4 (\|y\| mod 4) | differ for x or y of −3, −5, …: a view at the map's top-left corner |
| Which hidden tiles | full piece only when none of the eight neighbours is visible | agree |
| Full piece | frame ((997 x xor 1009 y) div 101) mod frames | differ |
| Edge piece | its table over visible bits (a corner counts visible beside a visible side) | same piece and mirror for 240 of 256 masks at even x and y; for the 12 masks of piece 12 frame 13 (not 11), for the 4 of piece 15 frame 12 (not 14); never the odd-x or odd-y variants (frames 1, 3, 5) |
| Shadows | black at alpha 64 for 1 and 2, 128 for 3 and 4 (`renderSDL/ScalableImage.cpp`) | same intent, other rounding (E17 design §3.3) |
| Palette cycles | the same ranges, the same direction (`config/terrains.json`, `config/rivers.json`; `ScalableImageParameters::shiftPalette`), a step every 180 ms of a continuous time (`MapRendererContext.cpp`) | differ: 200 ms, polled |
| Objects' frame | (time div 180 + the object's id) mod frames (`MapRendererContext.cpp`) | differ: a random phase, a counter that skips when late |

### 1.8 Open points

- ~~Terrain frames with runs (§1.3) and the DEFs' frame counts~~: closed by T2's gate (§1.9).
- The mouse's tile and the 8-pixel offset (§1.2): M3, with a capture hovering at y = 8 + 32 k + 28.
- Passes 4 to 9: read in T1b (§2). The puzzle and covered modes: later (M5 for the hot seat).
- The minimap (0x412C40, inferred) takes a terrain's colour from its DEF's palette entry 22: M3.
- What 0x6989C0 means and when the DEFs are reloaded (k's origin).

### 1.9 The frame and the gate's findings (M1 T2, 2026-10-23)

- **The frame picture** (adventure_full_update 0x417420, its first part read; called from 21
  sites): `AdvMap.pcx` (0x6603E0) loaded, entries 224–255 of its 16-bit palette given the viewing
  player's 32 colours (0x5FFE20, as a dialog picture's), drawn opaque from (0, 0) at its own size
  (0x44FDB0 with the last argument 0); then the panels (not read), the view (0x40F3F0 from the
  manager's +0xE4), the minimap (0x412C40), and the whole 800 × 600 shown when its first argument
  is set. The columns x 0–7 and 600–607, which the idle step shows with the view (§1.2), keep
  the frame's pixels: the passes' clip leaves them. VCMI 1.7.5 agrees: its frame is parts of the same
  picture in the player's colours (`config/widgets/adventureMap.json`, `imagesPlayerColored`).
- **The gate** (`h3-map-view gate`, T2) on the user's files: every name of the tables and the
  layout is found in the base archive searched first, `H3sprite.lod` (the DEFs) or
  `H3bitmap.lod` (`AdvMap.pcx`, 800 × 600, 8-bit; `PLAYERS.PAL`), and decodes. Group 0's frames: the terrains dirt 46, sand 24, grass,
  snow, swamp, rough, subterranean and lava 79, water 33, rock 48; each river 13, each road 17;
  `edg` 36, `tshrc` 4, `tshre` 34, each gem 8: exactly what the view draws (§1.4, §1.5, §1.1).
  The 159 shipped `.h3m` maps all read and give pictures up to dirt 45, sand 23, grass, snow,
  swamp and rough 78, subterranean 77, lava 76, water 32, rock 47, each river 12, each road 16:
  inside every DEF, so the viewer's refusal of a picture past its DEF's frames (§1.3) never
  meets a shipped map, and every code they use has its table entry. Encodings: terrains and `edg`,
  `tshrc` 0; rivers, roads and `tshre` 2 (`tshre`'s runs: index 0 and the shadow codes 1–4, as
  §1.5's second drawer reads them); the gems 1, runs of index 0 only.

### 1.10 The still (M1 T3b1 and T3b2, 2026-10-23)

- **The cut and the mirrors** (read at T3b1): a pass hands `def_draw_raw` a source rectangle of
  the tile's 32 × 32 picture as placed, (8 − x, …) when the tile starts left of the clip and so
  on (the roads' 0x411E67 to 0x411EC6; the others' the same, §1.2); `draw_frame_raw` turns it
  into the frame's own when mirrored (0x47DDD9: full width − x − w, and the height alike), so the
  rectangle cuts the picture as drawn. The viewer clips each piece on the screen to its tile's
  rectangle inside the view's clip (`h3_map_view::scene`), the same thing.
- **What `render` draws** (`h3-map-view render`): the frame's picture in the viewer's colours
  (§1.9), the passes 1, 2, 3, 10 and 11 (§1.1), each piece through the map's two drawers
  (`h3_map_view::draw`: `draw_frame_raw`'s literal pixels with runs skipped, mirrored both ways;
  0x47E820's runs 1–2 and 3–4 as shadows, the unexplored tiles' second time), the gems through
  the compositor's plain drawer with runs skipped. The palettes cycle by the moment's k, each
  step giving a range's entries their lower neighbour's colour (§1.6).
- **The canvases:** every frame of the twenty DEFs the passes draw (the terrains, rivers, roads,
  `edg`, `tshrc`, `tshre`) has a 32 × 32 canvas, the passes' source rectangle (a one-off read of
  the user's `H3sprite.lod` at T3b1).
- **The captures (T3b2):** the virtual clock (`xdd.dll`) drives the adventure map as it drives
  a battle: frozen, two dumps 1.5 s apart are the same, and a 400 ms step moves the hero's flag
  and the panels; the menus run in real time, so a capture's palette step k (mod 36) is read
  from the cycling DEFs' palettes in memory (each range its file's rotated up by k: one k fits
  all five DEFs) and the objects' frame steps from the manager's +0x100, never assumed; the
  harness steps the frozen clock to the later of the next tick and the next redraw, so the
  screen shows the palettes memory holds (§1.6, the lag). A
  minimap click reaches views past every edge ((−2, −2), (22, 21) on a 36-tile map) and the one
  whose bottom row is the map's last (MCL-Q2's case: reachable, the roads missing). Red's
  explored tiles are written into the vision words before the view is set (the harness,
  `clients/capture/README.md`, "The map's stills"); the unexplored pass reads only those bits.
  Every situation of the map's capture set (`clients/map/situations/`) matches the original pixel
  for pixel outside the panels: terrain of every kind with its transitions and mirrors, water
  and lava cycling, the four rivers, the three roads (the last row's at both views), the edge
  at both corners, the unexplored tiles' pieces.

## 2. The objects' draw order and shadows (M1 T1b, 2026-10-23; research item 9)

Path read: the placement of an object on its tiles (place_object_tiles 0x505B20, its layers
0x505060, the insertion 0x505230), passes 4 to 9 of §1.1, the hero and boat pieces (0x40FB30,
0x40FE30, 0x410760, 0x4102C0, 0x4109B0), the drawers' wrappers. Not read: the moving hero's own
pieces (0x47F860, 0x47FB40, pass 9) and the attacking monster's substitute (§2.2, both M2), the
puzzle's class filter; the owners' tables in T5b1 (§2.9). The frames that confirm it: T4a
(§2.6), T4b (§2.7), T5a (§2.8) and T5b1 (§2.9).

### 2.1 A tile's draw list

- Each map tile keeps a list (map_item +0xE) of 4-byte entries, first drawn first: the object's
  index (u16, map +0x14), its **cell** (byte 2: dy in the high nibble, dx in the low; the cell
  dx columns left of and dy rows above the object's anchor, its bottom-right corner) and its
  **layer** (byte 3).
- **Placement** (0x505B20; at load for every object in the map's order, map_objects_place_all
  0x4FD690, and for one placed in play): every cell of the template's width × height (+0x10,
  +0x11, from its picture's `.msk`, msk_load 0x514610) whose tile lies on the map gets an entry,
  column by column from the anchor leftwards, each bottom up, inserted by 0x505230 (then the
  tile's object is recomputed, 0x505610). Heroes (class 34), boats (8), the Grail (36), random
  heroes (70) and hero placeholders (214) get none; an event (26) only marks its entrance tiles
  (the tile's class and word). Cells off the map (left of or above it, or right of and below it
  when the anchor lies past the edge) have no entries: **those parts are never drawn**.
- **A cell's layer** (0x505060) comes from the template's passability (+0x1C, a set bit
  passable: the `.h3m`'s mask). A flat template (+0x40; inferred: the `.h3m` template's priority
  byte not 0, which the special grounds set) gives every cell layer 0. Otherwise, per column
  dx = 0 … w − 1, walking dy = 0 … h − 1 with L = 1 at the column's start:
  1. if dy > 0, the cell is blocked and the cell below it passable: L = 1;
  2. if dx > 0, the cell is passable and its right neighbour (dx − 1, dy) blocked: L = that
     neighbour's layer;
  3. the cell's layer is L, then L += 1.

  So a layer counts the rows from the column's base (the anchor row, or a blocked cell standing
  on a passable one) and a passable cell beside a blocked one takes the blocked cell's layer.
  Layers run 0 to 6 (never above dy + 1).
- **Insertion** (0x505230) keeps the list ordered by layer. The new entry N walks back from the
  end: past every entry of a higher layer; at an entry P of the same layer it compares the two
  objects' footprints (their rectangles' intersection, clipped to the map): if on any of those
  tiles P's layer (its entry there) is above N's (N's cell there), N goes under P and walks on;
  otherwise N stops right above P. It also stops above an entry of a lower layer, or at the
  list's start. So on a tile **the higher layer is on top; at equal layers the later-placed
  object is on top, unless the earlier one is higher on another tile they share.**
- **Removal** (remove_map_object 0x4AABB0; M1 T6b, the reviewer's R1-1): an object taken off the
  map loses its entry on each tile of its footprint, the later entries moving down a place in
  their order (0x4AACE2 to 0x4AAD03). The lists of a game played from its start are therefore
  those of every object placed, the fallen ones' entries taken out, which can differ from lists
  built without them (an object between two others decided their order): `DrawLists::build`
  builds them so. What a loaded saved game's lists hold (the objects gone before the save) is not
  read (M4).
- **The tile's object** (0x505610: the step, the visit, the pathfinder; E08 survey §2.5) is the
  topmost visitable entry of that order: at equal layers the later object in the map, as the
  engine takes it, but an object whose layer on that tile is higher wins whatever the map's order
  (the adventure lane's T14a point; its inbox, 2026-10-23).

### 2.2 How the passes draw them

Each object pass walks the view's tiles (§1.1), skips tiles off the map, and draws an entry as
**one 32 × 32 piece**: the frame (phase + manager +0x100) mod the frames of group 0 (§1.6) of the
template's DEF (map +0x24 by template), its square at (32 (w − 1 − dx), 32 (h − 1 − dy)), at the
tile's place (§1.2), clipped to the view. An object is drawn piece by piece; two objects may
stack differently on different tiles.

- **Pass 4** (0x412470): flat templates only, every cell (no mask), in list order, through the
  flagged drawer (draw_frame_flagged 0x47D0A0: raw pixels, runs of 5 in a colour when one is
  given, every other run skipped): the owner's colour (below) for classes 17, 20, 33, 42, 53, 77,
  87, 98 (0x47BE10), none for every other class (0x47BDC0 passes 0: its flag pixels are skipped).
- **Pass 5, the shadows** (0x411590): every entry, in list order, whose template is not flat and
  whose shadow mask (+0x24; H3API: the `.msk`'s shadow mask) has the cell, through 0x47BE60 into
  draw_frame_shadow_pass 0x47D930 (runs of 1 the border shadow, of 4 the body shadow; E17 design
  §3.3); then the moving hero's shadow (below); then the heroes' and boats' shadows over the tile
  (§2.3) in slots k = 0 … 5, at each k the hero's before the boat's (0x4102C0, 0x4109B0, through
  0x47C0D0 into the same drawer).
- **Pass 6** (0x411F50, a path shown): the arrows' shadows: `adag.def` frame a − 1 (a the tile's
  arrow, manager +0x48) through 0x47BFA0 into draw_frame_run_shadows 0x47E820.
- **Pass 7, the bodies** (0x410C00): for L = 0 … 6, every entry of layer L in list order whose
  template is not flat and whose colour mask (+0x14; H3API: the `.msk`'s colour mask) has the cell, through the
  flagged drawer, the owner's colour for the eight classes, none otherwise. **After L = 1** the
  heroes and boats standing on the tile's row (k = 0, 1, 2), **after L = 2** those standing on the
  row below (k = 3, 4, 5), at each k the hero before the boat (0x40FE30, 0x410760, through
  0x47C080 into the flagged drawer). A tile with an empty list draws its heroes and boats
  k = 0 … 5.
- **Pass 8** (0x4120B0): the arrows: `adag.def` frame a − 1, raw pixels (def_draw_raw 0x47BF50).
- **Pass 9** (0x47FC80): steps the moving hero's frame (manager +0x200) and plays its sounds.
- **The owner** (0x40FD20, of the entrance tile 0x4FEC30 gives): the dwelling's, garrison's,
  lighthouse's, mine's, shipyard's or town's owner from its table or word (a hero standing there:
  the object under the hero), −1 none; the colour is game_palette16's entry 64 + owner (63 for
  none; the palette's colours start at +0x1C, the code reads +0x9C + 2 × owner). The colours,
  the classes and the entrance: §2.9.
- **Substitutes, never in a still** (M2): the object whose index is manager +0x218 (a monster
  attacking a hero: monster_attacks_guarded 0x4A79D2, visit_object 0x4A97B9) is drawn in passes 5
  and 7 from the DEF at manager +0xE0, frame +0x220 + 2 × +0x21C. While a hero moves (manager
  +0x1EC, set by hero_move_step 0x480922) 0x47FB40 (pass 5) and 0x47F860 (pass 7) draw it at the
  view's places i = 8 … 10, its upper row at j = 7 after L = 2 and its lower at j = 8 after L = 1
  (group +0x1FC, frame +0x200): the view keeps the moving hero at its centre.

So every tile of the view is composed as: flat objects, **every shadow** (objects', the moving
hero's, heroes', boats', the path's), the bodies by layer with the heroes and boats between layers
1 and 2 and between 2 and 3, the path's arrows, the shroud (§1.5), the gems. **No shadow ever lies
on a body**, and every shadow lies on the flat objects.

### 2.3 Heroes and boats

- They have no entries. For a tile (x, y) the passes look at the six tiles x − 1 … x + 1,
  y … y + 1 for one whose class (+0x1E) is the hero's (34) or the boat's (8), with an entrance
  there and a word other than −1 (0x40FB30; nothing in the puzzle): slot k = 3 dy + dx + 1. The
  96 × 64 frame covers its tile's column ± 1 and its row and the one above; the piece is the
  frame's square at (32 (2 − k mod 3), 32 (1 − k div 3)).
- **So a hero is an object two rows tall with layers 1 and 2**, drawn after the objects of its
  layer: on its row over every layer-1 piece and under every piece of layer 2 or more, on the row
  above over layers 1 and 2. Among heroes and boats the right one is over the left one, the lower
  over the upper.
- **A hero on land** (0x40FE30, by the tile's word): its flag `af00` … `af07` (manager +0x16C by
  owner +0x22), then the hero `ah00_` … `ah17_` (+0x10C by class +0x30; names 0x65F5F0,
  0x65F5A8); group from 0x4D9110 (its facing, §2.8), frame manager +0x100 mod group 0's frames
  (no phase), mirrored when its facing (+0x47) is above 4.
- **A hero in a boat** (status bit 18): the waves `abm01_` … `abm03_` (manager +0x160 by the
  boat's type; names 0x65F67C) unless the hero's tile has map_item +0xD bit 0x02 (the coast,
  water beside land), then the boat's flag `abf0Nx` (+0x18C, type × 8 + the boat's colour; names
  0x65F610), then the boat `ab01_` … `ab03_` (+0x154; names 0x65F670); no hero picture. The boat
  comes from 0x4CE900 (inferred: the hero's boat), its type +0x1A, its colour +0x1C.
- **A boat without a hero** (0x410760, by the tile's word): the waves (but on the coast), then the
  boat, group from 0x4D9160; its template's DEF (`avxboat0` …) is never drawn on the map.
- 0x4102C0 and 0x4109B0 draw the same pictures in the same order through the shadow drawer.

### 2.4 VCMI 1.7.5 compared

VCMI orders a tile's objects by one comparator (`CMap::compareObjectBlitOrder`,
`lib/mapping/CMap.cpp`: the print priority, the `.h3m`'s byte × 100, below everything; then the
object with more blocked tiles directly under the other's blocked tiles on top; then the lower
anchor row on top; then heroes on top; then the id), inserted per tile
(`client/mapView/MapRendererContextState.cpp`) on every tile of the `.msk` rectangle in the map
(`ObjectTemplate::readMap`: every cell visible; `readMsk` gives only the size). Per tile it draws
terrain, river, road, then each object whole (its shadow with its body, then its flag, then its
overlay; `MapRendererObjects::renderObject`), then the path, then the fog
(`MapRenderer::renderTile`).

| Point | Original | VCMI |
|---|---|---|
| Pieces | 32 × 32 per tile, the cell's square of the frame, nothing off the map | same |
| Flat objects | layer 0, pass 4, under every shadow, without shadow or mask | print priority ≠ 0 first in the tile, with their shadow |
| D1. Shadows | all before any body (pass 5) | each with its object's body, in the tile's order |
| D2. Two objects | per tile, by their cells' layers: may stack differently on different tiles | one order for the pair on every tile |
| D3. Equal layers | the later placed on top, unless the earlier is higher on a shared tile | blocked-tile counts, anchor row, heroes, id |
| D4. Heroes and objects | a two-row object of layers 1 and 2 (§2.3) | by the comparator: above objects of its anchor row, below those anchored lower |
| D5. Heroes among themselves | right over left, lower over upper | by the comparator: lower over upper, then the id |
| D6. A hero's flag | under the hero | over it |
| D7. Boats | waves (not on the coast), flag, boat; an empty boat waves and boat | boat, flag, waves; flag and waves only with a hero aboard |
| D8. The `.msk` masks | a body only in cells of the colour mask, a shadow only in cells of the shadow mask | every cell of the rectangle |
| D9. The path | the arrows' shadows under the bodies (pass 6), the arrows over them (pass 8) | the arrow whole over the tile's objects |
| D10. Flag pixels | coloured for eight classes, skipped for the rest | coloured for any owner, neutral too |
| D11. A random object | a copy of its random template: its masks, layers and place kept, the drawn DEF's frames; an artifact's DEF by its id, one off from 128 (§2.7, MCL-Q3) | the new type's own template for the terrain, placed again (`CGObjectInstance::setType`) |

The user's note (neither their engine nor VCMI got the order right) is D1 to D5: VCMI's
comparator approximates the layers with its blocked-tile rule (the layers' rule 1) but keeps one
order per pair and draws shadows with bodies.

### 2.5 T4's situations and open points

Each a map of the writer (T3a: objects of any template) or a shipped map's view:

1. **D1:** an object with shadow pixels in a cell where another object drawn before it in VCMI's
   order has body pixels (a tree beside a resource pile on its left, the pile earlier in the
   map): the original keeps the pile's body clear.
2. **D2:** object A with a passable cell under a blocked one in a column (layers 1, 2, then 1)
   and object B anchored one row higher with its blocked base on A's fringe row: A over B on the
   fringe row, B over A on the row above.
3. **D3:** two objects anchored on one row and overlapping in a column where the later one has a
   blocked cell on a passable one (layer 1 on the upper row, rule 1) and the earlier one two
   passable cells (layer 2 there): the earlier stays on top on every shared tile; VCMI (no
   blocked tile under the other's, so the id) puts the later on top.
4. **D4:** a hero beside (left or right of) a blocked cell standing on a passable one of an object
   anchored one row lower (the cell's layer 1): the hero over that cell; VCMI under.
5. **D5:** two heroes on neighbouring tiles of one row, the left one later in the map.
6. **D6, D7:** heroes at rest (T5's situations), an empty boat and a hero in a boat, each in open
   water and on a coast tile.
7. **D8:** a template whose DEF has body or shadow pixels in a cell its `.msk` leaves clear, if any
   (T4's gate lists them by checking every object DEF against its `.msk`).
8. **D9:** a path arrow on a tile under an object's upper part.
9. Flat objects (Cursed Ground, Magic Plains, …) under another object's shadow; objects cut by
   the map's left and top edges and anchored past its right and bottom edges.

Open points: none. Settled by T5b2 (§2.10): point 8 (D9) in `path-objects`. Settled by T5b1
(§2.9): 0x4FEC30 is the entrance; the flag pixels outside the eight classes are in pictures the
map never draws (D10). Settled by T5a
(§2.8): the coast test for the waves; points 4, 5 and 6 in its captures; point 8 (D9) moved to
T5b with the path. Settled by T4a (§2.6): +0x40 as the priority byte, +0x14 and +0x24 as the
`.msk`'s masks; points 1 to 3 and 9 are in the copies of §2.6, so their own maps were not built;
points 4, 5, 6 and 8 moved to T5 with the heroes and the path, point 7 to T4b. Settled by T4b
(§2.7): point 7 (D8) finds nothing in the shipped files, the gate checks it.

### 2.6 The objects' frames (M1 T4a, 2026-10-23)

- **The `.msk`** (msk_load 0x514610 read further): two bytes, the width and height in tiles, then
  two masks of 6 bytes; bit i of a file mask (byte i div 8, bit i mod 8) becomes bit i of a 48-bit
  set (0x51486F to 0x5148AF), so a cell (dx, dy) is bit 7 − dx of byte 5 − dy, as in the `.h3m`'s
  masks; the first mask goes to the record's +8, the second to +0x10, and on to the template's
  colour (+0x14) and shadow (+0x24) masks. A survey of the user's sprite archives (every DEF
  with a `.msk`, 1 659, group 0's frames) agrees: every body pixel (literal or a flag's run of 5)
  lies in a cell of the first mask and every shadow run (1 or 4) in one of the second, while the
  masks swapped miss 800 and 604 DEFs; the one exception is `radar.def` (encoding 1, a 148 × 132
  canvas for a 3 × 3 mask), which is no map object. So D8 finds nothing in the shipped files
  (T4b's gate check keeps it so). Every object DEF is encoding 3 and its first group is id 0
  (the passes take the first group's frame count, 0x410EA1). A DEF without a `.msk` takes
  `default.msk` (1 × 1, in `H3sprite.lod`).
- **The insertion and the layers read again** (0x505230, 0x505060): as §2.1 says. At equal
  layers N's layer on each tile of the footprints' intersection comes from 0x505060 and P's from
  its entry on that tile; the intersection is clipped to the map (0x505348 to 0x505384), so cells
  off the map never decide. `h3_map_view::objects` follows it.
- **Flat templates:** the `.h3m`'s priority byte (`Objects.txt`'s last column) is 1 for the
  special grounds and for many swamp and other ground decorations (`AVLswp10` … `AVLswp70`);
  their pieces are drawn in pass 4 under every shadow, confirmed by `objects-flat` (47 flat
  objects) and `objects-witch`.
- **The moment:** a frame is (phase + the frame step the draw saw) mod the first group's frames;
  the harness reads the step after the redraw and subtracts one (§1.6), and every object's
  anchor and phase from the 12-byte records (map +0x14 to +0x18, the map's order: the anchors
  equal the `.h3m`'s, the two heroes included). One step more changes 620 pixels of
  `objects-mandate`.
- **The captures** (`clients/map/situations/`, `h3map.py`): four copies of shipped maps' dense
  corners with their objects (`objects-mandate`, `-witch`, `-flat`, `-myth`: 116 to 161 objects
  each) and objects cut by the map's four edges (`objects-edge-top-left`, `-bottom-right`: an
  8 × 6 flat ground, a volcano, a mountain, a windmill) are exact outside the panels. Counted
  from the draw lists in the views: a later entry's shadow cell over an earlier entry's body
  cell (D1) on 2 679 tile pairs, 17 pairs of objects stacked both ways on different tiles (D2),
  406 entries kept under an earlier object of the same layer (D3), 207 flat pieces under a
  shadow: the cases where VCMI's order can draw otherwise (cells, not pixels: a cell pair need
  not overlap in pixels), all drawn as the original draws them.

### 2.7 Random objects and shipped maps' views (M1 T4b, 2026-10-23; research item 9 closed)

- **When the original draws them** (resolve_random_objects 0x4C9DD0, from the new game's setup
  0x4BFE70): after the map's load has placed every object on its tiles (map_objects_place_all
  0x4FD690) and given it its phase (0x504470). Per tile with an entrance (level, row, column) a
  random artifact (65–69) becomes an artifact of its level (random_artifact_of_level 0x4C94D0), a
  random monster (71–75, 162–164) a monster of its levels (0x4C92C0), a random resource (76) a
  pile (79) of a random resource; the tile's class and subclass are set, then
  object_template_redraw 0x4C9990 gives the object its picture. The towns' setup does the same for
  a random town and the towns' pictures by their buildings (0x4CAA70; T5).
- **The picture** (0x4C9990): the object's template is **copied** onto the end of the map's
  templates and the copy's DEF name, class (+0x38) and subclass (+0x3C) replaced: an artifact
  `ava%04d.def` of its id (0x677974), a monster the DEF of the last template of class 54 with its
  creature (0x505EA0, the game's per-class lists), a pile `avtwood0` … `avtgold0` by resource
  (0x677958). The DEF is loaded onto the map's DEF list (+0x20) and the object's record points at
  the copy; its draw entries stay as placed. So **a random object keeps its random template's
  size, colour and shadow masks, passability (its layers) and priority, and is drawn with the new
  DEF's frames** (its phase as before). `h3_map_view::objects::shape` follows it.
- **What it changes on screen:** the random monsters' and resources' templates have the same
  masks as every monster and pile (the gate compares them), so nothing; a random artifact's
  (2 × 1, the body in the anchor's cell) cuts the Crown of Dragontooth (44: 2 body pixels) and the
  combination artifacts' pictures (`AVA0130` … `AVA0141`: 10 to 584 body pixels), which random
  relics can be (relics, not among the three disabled ids of 0x63B98C; 104 of the 160 shipped
  maps allow them). And `ava%04d` is one off from 128 on: the editor's templates (`Objects.txt`)
  name `AVA0129` for 128 (Armageddon's Blade), `AVA0130` for 129 and so on, `AVA0128` (a box) for
  none, so a random relic that became Angelic Alliance shows Armageddon's Blade. **MCL-Q3**
  (answered: the client draws the artifact's own picture and masks, the game's last template of
  its class and subclass, `Drawn::own`; `--bugs original` the copy, for the comparisons).
- **The gate** (`h3-map-view gate`): every template DEF of `Objects.txt` and of the shipped maps
  (1 333) has a shape and no body or shadow pixel outside its `.msk`'s masks (D8: none); and the
  pictures a random object can become, drawn with a random template's masks, listed with the
  pixels they lose (the artifacts above).
- **The harness** (`h3map.py`, `h3mem.Mem.map_templates`): a situation names a shipped map whole
  (`map`); the original loads it as a single scenario with the scenario list's setup (its first
  human player), its first day's messages closed by their OK (`adventure`, not `adventure_map`:
  that screen's probes hold the level button greyed, as a map without an underground shows it);
  the human player's explored bits written; `live.toml` gains the viewer and every object's
  template in memory (its DEF, class and subclass), which `render --live` compares with the map's:
  a differing one is what a random object became. The random dwellings (216–218) are resolved
  apart, when the map loads (T5b1, §2.9): in one start of four Tale of two lands' had moved a
  tile from the map's anchor (as an Angel Portal; at a Hydra Pond it had not).
- **The captures** (`clients/map/situations/shipped-*`): five views of four shipped maps (72, 108
  and 144 tiles) without heroes, boats or owned objects near them, one underground, with 3 to 6
  random artifacts, 4 to 9 random monsters and 2 to 8 random resources each among 110 to 217
  objects: exact outside the panels on their first runs, the whole set of 22 again after. With
  §2.6's copies this closes research item 9: the draw order (§2.1, §2.2), the shadows (pass 5,
  §2.2) and the random objects, read and confirmed by frames. Heroes and boats among the layers
  and the path's arrows: T5.

### 2.8 Heroes and boats (M1 T5a, 2026-10-23)

- **Their pictures** (view_hero_piece 0x40FE30, view_boat_piece 0x410760, their shadows 0x4102C0
  and 0x4109B0; §2.3): every picture in the group of the hero's facing (a hero in a boat: its
  waves, flag and boat too) or of an empty boat's: hero_map_group 0x4D9110 and boat_map_group
  0x4D9160 share one table, facing 0 (up) group 0, 1 and 7 group 1, 2 and 6 group 2, 3 and 5
  group 3, 4 (down) group 4, above 7 group 2; mirrored above 4. The frame is manager +0x100
  modulo group 0's frames of each DEF (no phase): the heroes `ah00_` … `ah17_`, the boats `ab01_`
  … and their waves `abm01_` … hold one frame in each standing group (ids 0–4; 5–9 are their
  moving groups, eight frames each), the flags `af00` … `af07` and `abf01l` … eight, so at rest a
  hero stands still and its flag waves with the objects' frame step. All through 0x47C080 with no
  colour: the pictures' flag runs are skipped; each colour has flag DEFs of its own (by owner
  0x65F5F0, a boat's by type × 8 + colour 0x65F610, its letters l g r d b p w k).
- **The coast** (map_item +0xD bit 0x02, map_tiles_load 0x4FE220): water of a shore picture
  (below 20) or land with the map file's coast bit, at the hero's or boat's own record's tile; no
  waves there (`TileArt::coast`).
- **Who is drawn:** a tile of class 34 or 8 with the entrance bit and a word (§2.3) names the
  hero among the 156 by type or the boat; a hero needs an owner 0–7 (but its boat's body in pass
  7, whose shadow pass checks the owner for both: 0x410325; no still has one). The hero record's
  class +0x30, facing +0x47, status +0x105 bit 18 (in a boat) and id +0x1A; its boat from
  hero_boat 0x4CE900 among the game's boats (a vector at game +0x4E3BC, 0x28 bytes: the tile, in
  use +0x18, type +0x1A, facing +0x1B, colour +0x1C, hero +0x20, aboard +0x24; `structs.toml`).
- **A new game** (observed in every capture): every hero faces right (2; hero_reset 0x4D8720
  writes 2 at 0x4D883C) and every boat; a map's boat has colour −1 and the type of its template's
  subclass. A hero the map places on water gets no boat: it stands on the water on horseback
  (`Live::at_start` keeps such a hero on land, as drawn). Boarding (hero_boards_boat 0x4A1010)
  takes the day's points without the Admiral's Hat, so a still's hero in a boat lies where the
  boat lay, beside land; the boat takes the hero's owner as its colour. At a human player's turn
  start the original lifts its selected hero off its tile (no class 34 there, adventure manager
  +0x1EC set: §2.2's moving hero) until the view moves (observed, not read: M2's).
- **The viewer** (`h3_map_view::scene`): the heroes and boats by tile from the live part; pass 5
  draws the six slots after the objects' shadows, pass 7 slots 0–2 after layer 1 and 3–5 after
  layer 2, each hero's or boat's pictures in the original's order, the frame's square on the tile
  (`Draw::Piece`'s group), clipped; tiles off the map get none. The tables' `[map.heroes]`
  (`clients/map/assets/base.toml`): the DEFs by class, colour, type, type and colour, and the
  facings' groups; the gate checks each DEF's five groups against group 0's frames (all hold
  them).
- **The harness** (`h3map.py`, `h3mem`): a situation's heroes of any colour (a class's hero
  types in order), Red's walks by play (`h3adv.walk`: a hero boards a boat by walking onto it),
  the facings written into the records after the view moved (`write_facing`, the heroes found
  by their records, `hero_on`: a still's staging, as the explored bits), the heroes and boats the
  view draws into `live.toml` (`map_heroes`), which `render --live` draws.
- **The captures:** `heroes-classes-0` to `-2` (the 18 classes × 8 facings, 48 heroes each, every
  colour's flag in each facing over the three: D6); `heroes-objects` (on The Mandate of Heaven's
  objects' passable cells, found in the original's draw lists: two beside a blocked cell on a
  layer-1 cell of an object anchored a row lower, D4; others on layer-2 and layer-3 cells; two
  side by side, the left one later in the map, and two one above the other, D5); `boats`
  (Deluge's lake: empty boats of the three types in open water and on the coast, turned; three of
  Red's heroes boarded, one on the coast and two on tiles given an open picture, D7): each exact
  on its first run.
- **Not captured:** the boats' flags of other colours than Red's (a computer player's hero would
  have to board; the gate checks their names and groups); a moving hero (M2).

### 2.9 The owners' colours, the towns' pictures, the random dwellings (M1 T5b1, 2026-10-24)

- **Which bodies take a colour** (passes 4 and 7: the jump tables at 0x41153C and in 0x412470, on
  the drawn template's class +0x38): the dwellings 17 and 20, a garrison 33, a lighthouse 42, a
  mine 53, the towns 77 and 98, a shipyard 87 draw their flag pixels (a frame's runs of index 5)
  through def_draw_flagged 0x47BE10 in their owner's colour; every other body through 0x47BDC0
  with colour 0, which the flagged drawer takes as none (its runs of 5 skipped). The map's classes
  are loaded as the game's (object_class_values 0x63A6E4 into the template's +0x38, 0x503BA3):
  the vertical garrison 219 as 33 and the AB mine 220 as 53 (and decorations 165–205, 221, 223,
  230 as others), so both are owned classes (the tables list them). The gate lists the object
  pictures with flag pixels by class: outside these, only `default.def`, a hero template's
  (`ah05_e`; heroes are drawn from their own DEFs, §2.8) and the random dwellings' (never drawn:
  resolved at the load, below), so no drawn object loses its colours (D10).
- **The owner** (tile_object_owner 0x40FD20) of the object on the entrance tile:
  object_entrance_tile 0x4FEC30 walks the template's cells, rows from the anchor's upwards within
  its height (+0x11), each from the anchor's column leftwards within its width (+0x10), and gives
  the first visitable one (+0x2C) on the map. Without one on the map the passes take tile (0, 0) of the
  surface's owner (0x410FC8, 0x4126B9): **MCL-Q4** (answered: none of the shipped maps has such
  an owned object, the gate counts them; the client takes the object's own owner, `--bugs
  original` the tile's). The tile's class and word name the record: the dwellings' (game +0x4E39C, 0x5C bytes,
  owner +0x57), the garrisons' (+0x4E3AC, 0x40 bytes, +0x0), the mines' and lighthouses'
  (+0x4E38C, 0x40 bytes, +0x0), the towns' (+0x21614, 0x168 bytes, +0x1), a shipyard's the
  word's low byte; a hero on the tile stands for the object under it (the hero record's +0xC,
  +0x14).
- **The colour**: game_palette16's entry 64 + the owner, 63 without one (`game.pal`: grey 63, then
  Red, Blue, Tan, Green, Orange, Purple, Teal, Pink; startup's 0x4EE430 converts it to 5-6-5), so
  a neutral object's flags are grey.
- **A town's picture** (object_template_redraw 0x4C9990, its town branch 0x4C9ADE): a Capitol
  (13) gives the faction's picture of 0x677A54 (`AVCcasz0` …), else a Fort, Citadel or Castle (7,
  8, 9) that of 0x677A30 (`AVCcasx0` …, the editor's), else that of 0x677A0C (`AVCcast0` …).
  setup_towns 0x4CAA70 redraws every town so at a new game, after its buildings (0x5C08C0): the
  standard set has a Fort when the map gives one (0x5C0C0F); the chosen buildings are the map's
  bits through 0x6888C0 (bits 2–5: Capitol, Fort, Citadel, Castle for every faction). A random
  town (77) first becomes a town of its faction (98). The copy keeps the template's masks (§2.7);
  the gate draws every faction's three pictures with each town template's masks and finds none
  that loses a pixel. `Live::at_start` gives each town its picture so (`[map.towns]`), and
  `render --live` refuses a town the original drew otherwise.
- **The random dwellings** (random_dwellings_resolve 0x502B60, from map_objects_place_all before
  the objects are placed on their tiles): each gets a faction (its linked town's, else one of its
  faction bits, 0x5132D0) and a level between its body's two, so a creature (0x6747B4); a Stone
  Golem makes the Golem Factory (20, 1), any other the last dwelling of class 17 that 0x63D570
  lists for it; 0x505F20 gives the object that class's last template of the subclass (appended
  to the map's templates). Its anchor's x then grows by the new template's bottom-row cells, from
  the anchor leftwards, that are passable and not visitable (of the class 17 and 20 templates only
  the Angel Portal's moves, by one); its dwelling record takes the body's owner. So its entries
  are the new template's at the new anchor; `render --live` takes the template (`Objects.txt`'s
  last with the original's DEF, class and subclass) and the anchor from memory.
- **The viewer**: the tables' `[map.owners]` (the palette, the neutral entry, the classes) and
  `[map.towns]` (the three lists by faction); `Draw::Piece`'s `flag` (an entry of the scene's
  `flags` palette) for the owned classes' flat and body pieces, the compose's flagged drawer in its
  colour; `objects::entrance`, `objects::flag_owner`; the live part's owners from the map at the
  start, from memory in the comparisons (`Mem.map_owners`, by the original's walk and records).
  `objects::entrance` walks all 8 × 6 cells where the original stays within the `.msk`'s width
  and height: the same while no template has a visitable cell outside it, which the gate checks
  (none of the 2 622 of `Objects.txt` and the shipped maps; M1 T6b, the reviewer's R1-5).
  `--bugs original`'s owner for MCL-Q4 takes the last object whose entrance is tile (0, 0), where
  the original takes that tile's object (the topmost visitable entry of its list, 0x505610, a hero
  there standing for the one under it): *provisional*, compared nowhere, since no shipped map has
  the case (the review list).
- **The captures:** `owners-objects` (the seven mines and an abandoned one, a dwelling, the Golem
  Factory and the Elemental Conflux, the garrisons, both vertical ones, a lighthouse, a shipyard:
  every colour and neutral), `owners-random-dwellings` (random dwellings of any kind, of level 7
  and of Castle: the Angel Portal's moved anchor in about one run of two, seen exact),
  `towns-0` to `-2` (the nine factions' villages, forts, citadels and castles, capitols, every
  colour and neutral), `towns-random`, and the shipped maps `shipped-hoard` and
  `shipped-too-many-monsters` (owned towns, neutral mines and dwellings, heroes): each exact on
  its first run, the whole set (35) again.

### 2.10 The path's arrows and the explored tiles (M1 T5b2, 2026-10-24)

- **The arrows** (view_path_shadows_pass 0x411F50, view_path_arrows_pass 0x4120B0, read): while
  the manager's +0x4C is set, each pass takes the tile's u16 from the array at manager +0x48, at
  (level × height + y) × width + x (every level, `structs.toml`), skips 0 and draws `adag.def`
  (+0xB0) frame a − 1 of group 0 at the tile's place, clipped to the tile inside the view's clip
  as every tile pass (§1.2): pass 6 through def_draw_run_shadows 0x47BFA0 into 0x47E820 (the
  frame's runs 1–2 as the border shadow and 3–4 as the body shadow, its pixels not drawn), pass 8
  through def_draw_raw 0x47BF50 (its pixels, the runs skipped). `adag.def` holds 50 frames of a
  32 × 32 canvas in encoding 2 (`PackedRows`, the shroud's layout): 1–25 green (a move today),
  26–50 the same arrows red (`h3mem.GREEN_FRAMES`, observed); arrow 26 (frame 25) the red cross
  at a path's end (both captures), frame 0 its green twin by its size. So an arrow's shadow lies under every body and over every other shadow, and the
  arrow over every body, under the shroud (§2.2). How a path's arrows are chosen from its steps
  and the hero's points is not read: M2's (the hero's walk), the engine's path from M4; in M1
  the live part takes them from memory.
- **The explored tiles** (§1.5): the unexplored pass reads only the viewer's bit of each tile's
  vision word; a new game sets it around the human player's heroes and towns (the reveal is the
  engine's, ADR-0016). The live part takes every level's bits from memory (`Mem.explored_rows`);
  `Live::at_start` keeps every tile explored.
- **The viewer**: the tables' `[map] path` (the gate checks its 50 frames); `Live::path`, one
  `Arrow` (tile, arrow) per tile; `Pass::PathShadow` (pass 6, the compose's `draw::shadows`
  alone) and `Pass::Path` (pass 8, `draw::raw`), each over the whole view between the shadows and
  the bodies and between the bodies and the shroud; an arrow 0, off the map or on another level
  is not drawn.
- **The harness** (`h3map.py`, `h3mem`): a situation's Red hero with a `path` is selected (the
  hero list centres the view on it), the view set again, the path's tile clicked once (as
  `h3adv.walk`'s first click) and its arrow there awaited; `explored = "start"` (a shipped map)
  leaves the new game's explored bits as they are; a boat's `colour` is written into the record
  of the boat its hero boarded (+0x1C, a still's staging as the facings, §2.8). `live.toml`
  gains `explored` (a row string of `0` and `1` per row of each level) and `path` (`[x, y,
  level, arrow]` while +0x4C is set, `Mem.path_arrows`).
- **The captures:** `path-objects` (objects-mandate's copy, a Red hero's path along a row the
  decorations' upper parts cover, D9: the arrows over their bodies, their shadows under them, the
  last arrow over the hero; green and red), `path-edge` (a path from a hero right of the view,
  cut by the view's right and left columns), `start-dwarven-tunnels` and `start-hoard` (shipped
  maps as a new game starts them, the shroud's edges around the explored circles, one reaching
  the map's right column beside the edge), `boats-colours` (seven boarded boats of the three
  types, Blue to Pink): each exact on its first run, the whole set (40) again.

## 3. The map moving (M2, from T1a1, 2026-10-26)

### 3.1 The idle loop in time order (M2 T1a1)

Path read: the managers' loop (`executive_loop` 0x4B0E40) on the adventure screen without a
dialog, a human player's turn, no network game, no hero moving; from the loop's turn through
`input_next_message` 0x4EC590 and the adventure manager's idle step (0x408A88) to the tick
(0x4ED490) and the redraws' own polls. Not read: the managers below the adventure manager (the
chain stops at it: its idle step answers 1), the panels' redraw at the end of the view draw, the
sound the idle step starts (0x4088F9 to 0x408996: M2 T5), the call that ends `pump_messages`.

- **The loop.** Each turn of `executive_loop` pumps the Windows messages (`pump_messages`
  0x4F7FB0), takes the next input message (`input_next_message`: **the tick first**, then the
  oldest of the input manager's 64 queued messages, else an **idle message**, kind 0) and hands it
  to the managers from the top down (each one's procedure, vtable +8) until one answers 1; then it
  starts over at once. Nothing waits: at a frozen clock the loop spins (the harness's "clock
  reads"), and a step of the clock is met by its next turn.
- **The tick** (§1.6) is polled by every turn of the loop, before its message, and inside each
  redraw (the view draw's start and end, 0x40F3F9 and 0x40F823; `screen_show_rect` before and
  after, 0x602BDE and 0x602C40). So a palette step falls in the first turn at or after its time
  (`palette_tick_time` 0x6989B4), and **before that turn's idle step**. Every 60 ms
  (`tick_60ms_time` 0x6989AC) it also calls `network_keepalive` 0x5546B0 (nothing outside a
  network game) and `turn_timer_poll` 0x557AF0 (inferred: a turn's timer, idle in a single game);
  neither draws.
- **The idle step** (the adventure manager's procedure on an idle message, 0x408A88): once the
  clock reaches `frame_deadline` 0x698998 it clears +0x200 (the moving hero's frame, T1b), draws
  the view (0x40F3F0, from +0xE4), shows (0, 8) to (608, 552) at once (`screen_show_rect`) and,
  while the clock still has reached the deadline and +0x104 is clear, grows +0x100 and moves the
  deadline on by max(180, the lateness) (0x408B26); then it pumps the messages. `view_show_step`
  0x40F270 is the same show and step after the other redraws (a move, a visit).
- **So, per millisecond of the clock:** the palette step if one is due, then the redraw if one is
  due. A redraw shows every palette step up to its own millisecond and draws the objects at the
  step +0x100 held before it (§1.6); the screen changes only at redraws. The palette steps come
  every 200 ms and the redraws every 180 ms, on two grids whose phase is the map's load's real
  time; neither is late when the clock moves 1 ms at a time (the harness), and a late one is
  not caught up. Since 180 < 200 a redraw shows one palette step or none, never two: one redraw
  in ten shows none, the one after a redraw that came less than 20 ms after a step (in the
  recordings, at the step's own millisecond: 180, 1 980, 3 780, … after their 0).

### 3.2 The recording (M2 T1a1)

`h3map.py frames [--only <id>]` (the runbook, "The map's sequences") plays each sequence of
`clients/map/sequences.toml`: its situation staged as a still is (`stage`, `capture`'s first
half), the clock frozen once both deadlines lie ahead of it, then **the sequence's 0** set to the
next millisecond by writing the palette tick's, the redraw's and the mouse poll's next times
there (the harness's own timers, as E17's `align_mouse_poll`: the phase between the grids is the
load's real time, so without this two runs differ), checked to have run at 0; then 1-ms steps to
`ms`, each followed by the state memory holds and the screen. Every screen whose view (outside the
panels) changed is kept (local); the **timeline** (`clients/map/timelines/<id>.txt`, numbers
only) lists the changes: `<ms> view <x> <y> <level>`, `<ms> objects <n>` (+0x100 counted from
0), `<ms> cycles <terrains> <rivers>` (each kind's palette steps counted from 0) and `<ms> screen
<x0>,<y0>,<x1>,<y1>` (where the screen changed since the screen kept before: each one kept); the
first state at 0, whose screen is always kept; the lines of one millisecond in that order, not
the events' (the tick comes before the redraw). The live part at 0 is a still's
(`timelines/<id>-live.toml`, kept from a run as recorded or new): the viewer's frame drawn with
it is compared with the original's at 0 (must be exact).

- **`idle-cycles`** (`seq-idle-cycles`: lava and water, the muddy, clear and lava rivers in four
  quadrants of one view; 8 000 ms): the palettes step at 0, 200, 400, … (40 steps after 0, both
  kinds together), the redraws at 0, 180, 360, … (+0x100 grows at each); the screen changes at
  every redraw but 180, 1 980, 3 780, 5 580 and 7 380 (39 screens after 0's), each time over the
  whole view: as §3.1 says, nothing between the redraws.
- **`idle-objects`** (`objects-flat`: 116 objects, 47 flat, among cycling terrain; 8 000 ms): the same steps; the
  screen changes at every redraw (44 after 0's), at the five redraws without a palette step only
  the objects ((8, 13) to (600, 552)).
- Each recorded twice, the second run's timelines as the first's; the frame at 0 exact in all
  four runs. So the reading holds frame by frame for the map standing still: M2 T2's schedule is
  the two grids from the sequence's 0, and at each redraw a still (§1.6) at the steps it shows.

### 3.3 VCMI 1.7.5 compared

| Point | VCMI (`client/mapView/`) | |
|---|---|---|
| Time | `MapViewController::tick` adds each rendered frame's time to the adventure context's `animationTime`; the map is drawn every frame | differ: the original draws the map only at its 180-ms redraws (§3.1) |
| Palette steps | `MapRendererAdventureContext::terrainImageIndex`: `animationTime` div 180 | differ (§1.7): every 200 ms, polled, shown at the next redraw (none at one redraw in ten) |
| Objects' steps | `objectImageIndex`: `animationTime` div 180 plus the object's id | the same 180 ms; differ (§1.7): a random phase, a step only at a redraw, a late redraw not caught up |
| Scroll (T1a2) | `AdventureMapInterface::handleMapScrollingUpdate` (`client/adventureMap/`): every frame by `scrollSpeedPixels` × the frame's time, in pixels (the options' 400, 800, 1200 for the original's three speeds) | differ: whole tiles, 1, 2 or 3 a step, a step every 70 ms at the edges (§3.4) |
| Edge (T1a2) | `borderScrollWidth` 15: x < 15, x > width − 15; none while Ctrl is down or the window has no focus | differ by a pixel: 16 (x < 16, x > 784); the original's own focus flag the same (§3.4) |
| Keys (T1a2) | the eight direction keys walk the selected hero (`config/keyBindingsConfig.json`, `moveHeroDirectional`); no key scrolls | the same without Ctrl; with Ctrl they scroll, one step a press (§3.4) |
| Edge cursors (T1a2) | `Cursor::Map::SCROLL_*` by the direction while scrolling, the pointer when the delta is nothing | the same frames (32 to 39); the arrow once the view stops at a limit (§3.4) |
| Walk's time (T1b) | `MapViewController::tick`: the move's progress by each rendered frame's time over `heroMoveTime` a tile (the options 200, 150, 100, 50, 25 ms and 0 instant, default 150; `enemyMoveTime` the computer's) | the same 200, 150, 100 ms a tile at speeds 1 to 3 and instant at 4, but in 4, 3 or 2 frames of 50 ms (§3.5); no 50 or 25 |
| The view in a walk (T1b) | centred on the hero's interpolated place every rendered frame (`setViewCenter`), the hero drawn by `objectImageOffset` | the same centring, the map sliding by 8, 10-12-10 or 16 pixels a frame under the hero drawn at a fixed place (§3.5) |
| The walking hero's frames (T1b) | `MapRendererAdventureMovingContext::objectImageIndex`: the animation time div 50 mod the group's frames; `moveGroups` by the rotation | the same 50 ms and walking groups (5 to 9, the left ones mirrored); the frame counted from the walk's start, one a drawn frame, carried from step to step (§3.5) |
| The path in a walk (T1b) | `HeroMovementController::updatePath`: the node removed when the server's move arrives, before its animation | the same: the next tile's arrow gone at the step's start (§3.5) |
| Objects in a walk (T1b) | animated on (the context's animation time runs) | differ: they stand still (no idle step runs during a walk), the palettes step on (§3.5) |
| Step sounds (T1b) | `getMovementSoundFor`: one looped sound while the terrain stays, the terrain's `horseSound` on a road at both ends, else `horseSoundPenalty`; `HORSE10` flying | one sample a cycle of 8 frames by the new tile's terrain (+0x360), +0x388 in a boat or flying; no road read (which samples: T5) |

### 3.4 The scroll (M2 T1a2)

Path read: the adventure manager's procedure (`adventure_manager_procedure` 0x4087B0) on a
human player's turn, its messages by kind (0x4089AE: 0 idle, 1 a key down, 4 the mouse moved,
0x200 a control's; a key up does nothing), the key-down's eight direction keys
(`adventure_key_down` 0x408C40), the scroll itself (`view_scroll` 0x4195C0), the edge scroll
(`edge_scroll` 0x419820), the mouse-move's branch outside the view (`adventure_mouse_move`
0x40E360 from 0x40EB7D), the key messages' queuing (`input_key_message` 0x4EC0E0) and the window
procedure's focus flag (`window_procedure` 0x4F7C00); the settings by their registry names
(`settings_read` 0x50B7B0, `settings_check` 0x50B260). Not read: the other keys, the mouse over
the view (the cursor by the tile under it: M3), the minimap's redraw (0x412C40), the options
dialog beyond its speed buttons.

- **The speeds.** The registry's "Window Scroll Speed" and "Walk Speed" (`settings_names`
  0x63FF74) are read into `option_scroll_speed` 0x69878C (0 to 2, any other 1; the options
  dialog's three buttons) and `option_walk_speed` 0x69875C (1 to 4, any other 2: the hero
  speed, its use M2 T1b); "Computer Walk Speed", `option_computer_walk_speed` 0x698758 (2 to 5,
  any other 3). The scroll reads its speed at each step, so a sequence's options are written into
  memory once its situation is staged (`h3map.py frames`).
- **A step** (`view_scroll`, a direction 0 to 7 clockwise from north, a flag): the step is 1, 2
  or 3 tiles at the speeds 0, 1, 2 (`scroll_steps` 0x63A66C); `scroll_time` 0x691674 = now; the
  view's corner moves by the step (north y less, north-east x more and y less, … north-west x and
  y less); with the flag the cursor becomes the direction's edge cursor; then the corner is held
  within x −9 to the map's width less 10 and y −8 to its height less 9 (the map's first and last
  column at the view's middle column, 9 of 19; its first and last row at row 8 of 17), so a
  diagonal step at one limit still moves along the other side. An unchanged corner ends the step:
  no redraw. A changed one: the selected hero back on its tile if the turn's start lifted it
  (`adventure_hero_stop` 0x417680, no redraw), the corner written, the minimap redrawn, **the
  view drawn and shown at once** ((0, 8) to (608, 552), as the idle step shows it, with the
  objects at the step +0x100 holds and the palettes as the tick left them), and if the clock has
  reached `frame_deadline` the objects stepped as at a redraw (+0x100 + 1, the deadline on by
  max(180, the lateness)): the step's redraw then takes the place of the redraw due. Then the
  messages are pumped.
- **The keys** (`adventure_key_down`, by scan code): 72 Up north, 73 PgUp north-east, 77 Right
  east, 81 PgDn south-east, 80 Down south, 79 End south-west, 75 Left west, 71 Home north-west
  (the keypad's with NumLock off give the same codes). **With Ctrl** (the message's modifiers
  +0xC and 0xC) a step that way, without the edge cursor; **without Ctrl** the selected hero
  steps that way (`hero_move_step` 0x4805E0: M2 T1b), nothing without a selected hero. A key-down
  message is one step at once, no delay of its own; Windows repeats a held key's WM_KEYDOWN and
  every one is queued (`input_key_message`, its repeat flag not looked at), so a held key steps at
  the system's keyboard rate. The game has no repeat of its own: the harness turns the display's
  repeat off (it repeats in real time, which one step of the virtual clock spans many times
  over), so in a sequence a key held is one press.
- **The edge** (`edge_scroll`, called before every message the adventure manager gets, idle
  ones too, while `app_active` 0x6783D0, set by WM_ACTIVATEAPP): the mouse's point now
  (`mouse_position_now` 0x50D500: GetCursorPos, x rounded down to even; not the 33-ms poll's).
  Off the 800 × 600 screen or not within 16 pixels of a side: `scroll_time` = now, nothing else.
  Within: west x < 16, east x > 784, north y < 16, south y > 584, a corner both (so the zones
  are 16, 14, 16 and 15 pixels wide, the east one narrowed by the rounding: MCL-Q6); nothing until
  70 ms after `scroll_time`, then a step with the edge cursor (which sets `scroll_time` = now);
  if then the cursor is an edge cursor and the view did not move (a limit), the arrow (frame 0
  of `cradvntr.DEF`). So the pointer arriving at an edge at t steps at t + 70, t + 140, … while
  it stays within the 16 pixels; moving between edges keeps the grid (a step whose view did not
  move counts as one); a key's step starts it anew.
- **The edge cursors:** `cradvntr.DEF` (type 1) frames 32 to 39, north clockwise to north-west;
  hot spots (`cursor_hot_spots` 0x67FF50 + (144 + frame) × 8): (6, 0), (16, 1), (22, 6),
  (16, 17), (6, 22), (0, 17), (0, 5), (0, 1). Set by each edge step (also by one whose view did
  not move, before the arrow above); a move of the mouse outside the view (0x40EB7D) keeps an
  edge cursor while the mouse lies within the 16 pixels and sets the arrow otherwise. Drawn at
  the mouse poll's point (E17 design §9.10): the point lags a move by up to 33 ms, the frame
  changes at once.

**Recorded** (`clients/map/sequences.toml`, situation `seq-scroll`: a 36-tile map copied from a
shipped one, the view at (9, 10); every frame at 0 exact):

- **`scroll-0`, `scroll-1`, `scroll-2`** (one speed each, the same inputs): Ctrl and Up, PgUp,
  Right, PgDn, Down, End, Left, Home pressed at 100, 300, … 1 500 step the view at the press's
  own millisecond by 1, 2 or 3 tiles in that direction (back at (9, 10) after the eighth); Ctrl
  and Right held from 1 700 to 2 100: one step. The pointer at the left edge (2, 300) from 2 300:
  the arrow's point moves at the poll (2 310), the first step at 2 370 with the west cursor (38),
  then every 70 ms to 2 650. In the top-right corner (798, 1) from 2 700: the west cursor drawn
  there at the poll (2 706) until the grid's next step at 2 720 turns it north-east (33), steps
  to 3 000; over the right panel at 3 000 (after that millisecond's step) the arrow. Each step
  redrew the whole view at once; the cursor lines are the same at every speed.
- **`scroll-limits`** (the fast speed): the pointer in the bottom-right corner at 100 steps at
  170, 240, … 520 by 3 tiles until (26, 27), the map's last column and row (x clamped at 520 from
  27, y from 28); the step at 590 moves nothing and the cursor turns from south-east (35) to the
  arrow. In the top-left corner from 800: the grid kept (a step at 800 before the move, the next
  at 870), (−9, −8) at 1 640, the arrow at 1 710. In the top-right corner from 1 900 with the top
  reached: x moves by 3 at 1 920 to 2 200, y stays, the cursor north-east (33) throughout. Over
  the right panel at 2 200 (the step at 2 200 came first): the arrow. Ctrl and Up at the top
  limit at 2 300: nothing. Every step that moved redrew the whole view at once (screen lines at
  the steps' own milliseconds); the cursor's point followed each move at the next poll (132,
  825, 1 914, 2 211: the 33-ms grid from 0).

### 3.5 The walk (M2 T1b)

Path read: a human player's walk along the path shown, from the click on its destination
(`hero_move_loop` 0x407B80; its visits by the destination's class, the switch at 0x408750, not
read), and the eight direction keys without Ctrl (`adventure_key_down` from 0x408E97); in
`hero_move_step` 0x4805E0 what draws and times a step (its pricing: E08; boarding and landing,
0x480949 to 0x480B0C and `hero_boards_boat` 0x4A1010, the visits and the network branches not
read: M4); the animation (`hero_walk_animation` 0x480380), the lifted hero's passes 5, 7 and 9
(0x47FB40, 0x47F860, 0x47FC80), `hero_walk_end` 0x47F7D0, `mobilize_hero` 0x4175E0 and the lift
and centring in `hero_select` 0x417B20 (the hero list and path rebuilt in between not read),
`wait_until` 0x4F82F0 during the frames. Not read: the computer player's walks (its speed's use
only, M4), `hero_step_allowed` 0x481BE0's tests, `draw_frame_blend` 0x47D4F0, which samples the
step sounds are and the ambient sounds (0x4183D0; both T5).

- **The lifted hero.** A human player's selected hero is drawn apart from the map while the
  manager's +0x1EC is set: lifted off its tile (+0x20C; `hero_select` hides it from its tile's
  draw list, +0x1F0 its class, 34, or a boat's, 8) and drawn at the screen's fixed place (256,
  232), 96 × 64: the view's columns 8 to 10 and rows 7 and 8, its tile at the view's (9, 8). Pass
  5 draws its shadow's six 32 × 32 pieces and pass 7 its pieces among the layers (§2.2: row 7
  after layer 2, row 8 after layer 1), pass 9 the whole of it once more over everything through
  `def_draw_group_blend` 0x47C120 (inferred: a 50/50 blend, `draw_frame_blend`), its part beyond
  the map's side not drawn. In a boat: the waves (not on a coast tile), the boat's flag and the
  boat; on land the flag and the hero (§2.3's DEFs). Group +0x1FC, frame +0x200, the flag's frame
  (+0x100 + +0x200) mod 8, mirrored when the facing +0x47 is above 4. The places are fixed: the
  view's pixel offset below does not move them. A scroll puts the hero back on its tile
  (`adventure_hero_stop` clears +0x1EC and +0x20C, §3.4); the turn's start and each step lift it
  again.
- **The walk** (`hero_move_loop`, the second click on the path's destination): the hero lifted
  (`mobilize_hero` with 1, 0, 0: not centred), the start tile's step sound played, then the
  pathfinder's path step by step through `hero_move_step`, **no wait between steps**. After each
  step the loop drains the input messages: a key-down, a mouse button or a control's message ends
  the walk after the step running; so do a refused or unpaid step and an event. At its end
  `hero_walk_end` (the standing group by the facing, `hero_map_group` 0x4D9110; frame 0; the step
  sound stopped), the path cleared once the hero stands on its destination, else rebuilt for the
  rest (`make_hero_path` 0x418DD0, which redraws), the cursor by the mouse, the terrain's music.
  Nothing else redraws: the standing hero shows at the next idle redraw (§3.1), at once when its
  deadline passed during the walk, which then steps +0x100 once for each 180 ms the walk spanned
  past it, up to the deadline's catching up (the deadline moves by max(180, the lateness)).
- **A step** (`hero_move_step`): `view_shown` set by whether the viewing player sees the step's
  start or end tile (0x48071F: else no frame is drawn); `mobilize_hero` with 0, 0, 1 (a hero not
  lifted, as after a scroll, is lifted, the view centred on it and the whole screen shown at once:
  `hero_select` 0x417E44, 0x41809B); +0x1EC set; the facing +0x47 and +0x1F4 the direction; **the
  next tile's arrow zeroed** (`path_arrow_at` 0x41B010, 0x480B30: no redraw of its own, the next
  frame shows it gone); the step refused by `hero_step_allowed` (the walk ends); then the
  animation; the points; at the walk's last step (the path's last tile, the points then 0, or an
  event) the standing group, frame 0 and the step sound stopped (0x4811CD); the terrain's music
  when the tile at the view's centre changed terrain (0x4812DF: `terrain_music_tracks` 0x678330,
  `music_track_play` 0x59A1E0); the new tile's step sound (below); the ambient sounds (0x48141B);
  the view's offset back to 0 (0x481423).
- **The animation** (`hero_walk_animation`): the walking group by the direction (0x4805BC: 5 up,
  6 up-right and up-left, 7 right and left, 8 down-right and down-left, 9 down; the left ones
  mirrored by the facing), +0x390 set, **the hero's tile moved to the next at once**. With
  `view_shown`: **the view's corner the new tile less (9, 8)** (centred on the hero, never
  clamped: the scroll's limits, §3.4, hold anyway) and **the view's pixel offset** (+0xF4, +0xF8,
  added to every tile's place by every pass: §1.2's "no pixel scrolling" holds for the scroll,
  not for the walk) from (32 dx, 32 dy) (`walk_start_offsets` 0x63D6F0): the screen looks as
  before the step. Then each frame: its end = now + the frame's time, the offset less the
  frame's pixels times (dx, dy), the view drawn (`view_draw_here` 0x40F870) and (0, 8) to (608,
  552) shown at once, `wait_until` its end; so the map slides under the hero, who stays at the
  view's centre. The speed is `option_walk_speed` for a human player, `option_computer_walk_speed`
  else, read at each step (0x480421), by the tables `walk_frame_pixels` 0x63D6C8 and
  `walk_frame_times` 0x63D6DC:

  | Speed | Frames a tile | Their pixels | A frame | A tile |
  |---|---|---|---|---|
  | 1 | 4 | 8 | 50 ms | 200 ms |
  | 2 (the default) | 3 | 10, 12, 10 (0x480509) | 50 ms | 150 ms |
  | 3 | 2 | 16 | 50 ms | 100 ms |
  | 4 | the view drawn and shown once at offset 0, no wait (0x480483) | | | 0 |
  | the computer's 2 to 4 | as the human's 2 to 4 | | | |
  | the computer's 5 | none (32 / 100 frames): nothing drawn | | | 0 |

  Table index 0 (16 frames of 2 pixels, 100 ms each) is no setting's. The waits run the tick
  (`wait_until` calls `adventure_tick`), so the palettes step every 200 ms during a walk and show
  at the next frame; the objects' step +0x100 does not move during a walk (no idle step runs), so
  the map's objects stand still while the hero walks, the hero's own flag stepping with its frame.
- **The frames.** Pass 9 steps the frame +0x200 after it drew it while +0x390 is set (0x47FF6D):
  one frame of the group a frame of the animation, mod 8, carried from step to step, back to 0
  at the walk's end. Memory after a frame holds the frame pass 9 draws next (the timeline's
  `walk` lines).
- **The keys without Ctrl** (`adventure_key_down` 0x408E97): with a selected hero whose step that
  way `hero_step_allowed` allows: a path shown is cleared and the view redrawn and shown at once
  (with the objects' step when due); the hero's destination the neighbour; `hero_move_step` with
  the last step's flag (one step, as above); `hero_walk_end` when the step says so; the cursor,
  the music. A key-down is one step (the original repeats nothing itself, §3.4).
- **The sounds** (for T5): the walk's start plays the start tile's step sound (0x407D03); a
  step's end plays the new tile's at once while the frame +0x200 is 0, else queues it
  (`step_sound_next` 0x6968E4) and pass 9 plays it when the frame wraps to 0 (0x47FF88), stopping
  the one playing (`step_sound_playing` 0x6968E0); `hero_walk_end` stops it. The sounds: the
  manager's +0x360 by the tile's terrain, +0x388 in a boat or flying (Angel Wings, or +0x112 not
  -1); which samples they are, T5.

**Recorded** (`clients/map/sequences.toml`; situations `seq-walk`, `seq-walk-follow`,
`seq-sail`; the timeline's walk lines since T1b: `hero`, `walk`, `arrows`, only where they
change; the objects' phases written 0 once staged, as the screen's changed boxes at the idle
redraws moved with the load's random phases: the runbook; every frame at 0 exact):

- **`walk-2`** (speed 2): each direction key steps the hero at its press's millisecond: the
  view's corner the new tile less (9, 8), the walking group by the direction (5, 6, 7, 8, 9, 8, 7,
  6 for north to north-west, the facing 0 to 7) and frames at the press, + 50 and + 100 with
  the offsets 22, 10, 0 times (dx, dy), each a whole view; at + 150 the standing group (0 to 4 by
  the facing) and frame 0 in memory and an idle redraw at once (its deadline passed during the
  step: the objects + 1, the deadline kept on its 180-ms grid), whose screen changes only at the
  hero. The path clicked at 2 500 shows four arrows at once (`make_hero_path`'s redraw, the
  objects as they stood); clicked again at 2 700, after the idle redraw due then: four steps at
  2 700, 2 850, 3 000 and 3 150, each starting with its tile's arrow gone and the view moved, the
  frame carried on (1, 2, 3, then 4, 5, 6, then 7, 0, 1, then 2, 3, 4); the palettes stepping at
  2 800, 3 000, 3 200 in the frames, the objects not; the last frame at 3 250, the standing group
  and the idle redraw at 3 300 (the objects + 2: the deadline 2 880 had passed by 420 ms).
- **`walk-1`, `walk-3`** (speeds 1 and 3, the same inputs): four frames 50 ms apart with the
  offsets 24, 16, 8, 0 (a key's step ends at + 200), two with 16, 0 (+ 100); the click's walk at
  2 700 steps every 200 and 100 ms, the frame carried on (speed 1: 1 to 4, 5 to 0, 1 to 4, 5 to
  0); otherwise as `walk-2`.
- **`walk-4`** (speed 4): a key's step is one whole-view frame at its press, the view and the
  hero's tile moved, the standing group already in memory: the frame drawn shows the walking
  group's frame 0 (the one draw), the standing picture at the next idle redraw on the 180-ms grid
  (+ 80). The click's walk of four tiles falls whole in its millisecond (2 700: the view, the
  hero, the arrows gone, the standing group); the walk's end then reads the pointer, still on the
  clicked tile (the harness settles a click before the next input: the cursor drawn there until
  the poll at 2 706 moves it to the parked point).
- **`walk-points`** (450 points, a tile six steps east): the path's arrows, four green and two
  red (the frames + 1: 12, then 37 and the destination's 26); four steps (300 to 750), the points
  then 50, less than a step: 0 and the walk's last step; the standing group at 900, the two red
  arrows kept (the path rebuilt the same).
- **`walk-sail`** (the hero in a boat, 1 500 points, a water tile five steps north-east): the
  boat's walking group 6 and its frames as a hero's, five steps 150 ms apart from 300; the
  standing group 1 at 1 050; the view's corner passing the map's top (y −1). The palettes keep
  stepping the water, shown at each redraw.
