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
- ~~The minimap (0x412C40) takes a terrain's colour from its DEF's palette entry 22~~: entries 8 and 9
  (a 16-bit palette's colours start at +0x1C; M3 T3a, §4.4).
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
sound the idle step starts (0x4088F9 to 0x408996: a network game's, §3.11), the call that ends `pump_messages`.

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
| Step sounds (T1b) | `getMovementSoundFor`: one looped sound while the terrain stays, the terrain's `horseSound` on a road at both ends, else `horseSoundPenalty`; `HORSE10` flying | one looped sample from the walk's start to its end (`horse00` to `horse09` by the terrain, `horse10` flying, the water's in a boat), changed only where the view's middle changes terrain, at the frame's wrap to 0; no road (§3.11) |

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
  every one is queued (`input_key_message`, its repeat flag not looked at; the waits of a step's
  frames dispatch the window's messages, `wait_until` 0x4F82F0), so a held key steps at the
  system's keyboard rate and the repeats that come during a step wait in the ring (64 messages,
  key-ups too; a write that meets the read index drops the oldest, 0x4EC1D9: 63 kept): the hero
  walks on after the key's release until they are used up, its points permitting (MCL-Q10). The game has no repeat of its own: the harness turns the display's
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
step sounds are and the ambient sounds (0x4183D0; both T5: §3.11).

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
- **The sounds** (T5: §3.11): the walk's start plays the start tile's step sound (0x407D03); a
  step's end where the music's terrain changes plays the new tile's at once while the frame
  +0x200 is 0, else queues it (`step_sound_next` 0x6968E4) and pass 9 plays it when the frame
  wraps to 0 (0x47FF88), stopping the one playing (`step_sound_playing` 0x6968E0); `hero_walk_end`
  stops it. The sounds: the manager's +0x360 by the tile's terrain (in a boat too), +0x388 flying
  (Angel Wings, or +0x112 not -1).

**Recorded** (`clients/map/sequences.toml`; situations `seq-walk`, `seq-walk-follow`,
`seq-sail`; the timeline's walk lines since T1b: `hero`, `walk`, `arrows`, only where they
change; the objects' phases and frame steps written 0, as the screen's changed boxes at the
idle redraws moved with the load's random phases and real time: the runbook; every frame at 0
exact):

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
- **`walk-follow`** (speed 2; the view at (0, 9), the hero at (6, 17), the view's column 6):
  the path to (14, 19) shown at 100; walked from 300: the first frame already at the view
  centred on the first step's tile ((−2, 9): x past the map's side), every step moving it with
  the hero; the standing group at 1 500. Ctrl and Down at 1 800 scroll the view to (5, 12), the
  hero back on its tile (its flag stepping there at 2 040, a row above the centre); Right at
  2 100 steps it and the first frame shows the view centred again ((6, 11)).
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

### 3.6 The map over time, headless (M2 T2)

No new reading: §3.1 and §3.4 built as `h3_map_view::timeline` (*provisional*), checked against
T1a's six sequences line by line and frame by frame.

- **One millisecond** (`Loop::run`): the palette tick when due (the next one now + 200 ms), the
  mouse poll when due (the cursor drawn where the mouse is, its x even under `Bugs::Original`, E17
  design §9.9; the next poll on by the larger of 33 ms and its lateness), then the idle message
  (the edge scroll, then the idle redraw when due), then each action given at that millisecond as
  its message, the edge scroll before each. A pointer's move is seen by the edge scroll of its own
  message (GetCursorPos reads the new point); outside the view it sets the arrow unless the point
  lies in a zone (an edge cursor stays). The actions come after the millisecond's idle turns, as
  the harness gives its inputs: a key's step at a redraw's millisecond draws the step the idle
  redraw left (`scroll-0` at 900), while an edge step there comes first, in the idle message, and
  takes the redraw's place (`scroll-limits` at 1 080). Both give the timeline's lines; neither
  view shows an animated object then, so the frames cannot tell the two orders apart.
- **The counts.** The timeline's `objects` is +0x100 less its value after 0's redraw; a draw shows
  +0x100 before its step, so the draw at a redraw after 0 shows the line's count before it plus
  one, and the live part's `frames` is what 0's draw showed. The `cycles` count the ticks after 0's
  (both kinds alike: the viewer's clock keeps the rivers with the terrains, MCL-Q5; a still's split
  comes from the live part). The objects' step after a redraw loops while the deadline is reached,
  the deadline on by the larger of 180 and the lateness: a redraw 220 ms late steps twice (as
  `walk-2`'s +2 at 3 300 showed); each step is a redraw of its own, the screen the last (§3.8).
- **The screen** (the `sequences` adapter): the view as its last draw shows it, the cursor over
  it (`cradvntr.def`'s frame by the mouse manager's drawer, runs skipped: `Cursor::read`, copied
  from `h3-view`; its hot spot from `cursor_hot_spots` 0x67FF50 + (144 + frame) × 8, the layout's
  `[cursor]`, 43 frames, read 2026-10-27: 32 to 39 as §3.4 lists them, the arrow (0, 0)). Drawn
  at each millisecond the draw or the cursor changed, its box against the last kept screen within
  (0, 0) to (608, 552) is the `screen` line: so the viewer's `screen` lines say its frames change
  where the original's did, and the frame check says they are the same frames.
- **The edge zones** (MCL-Q6): the client's 16 pixels on every side (x < 16, x ≥ 784, y < 16,
  y ≥ 584), the original's (x even, x < 16, x > 784, y < 16, y > 584) under `Bugs::Original`, which
  `h3-map-view sequences` takes by default (the comparisons). The sequences' points lie in both.
- **Not played:** a walk (a hero's step, a click: T3's, `Unplayable::Walk`; `sequences` lists the
  seven walk sequences as T3's), the pointer over the view (M3's cursor by the tile: refused).
- **Checked** (2026-10-27, T1a's six sequences recorded again in a fresh sandbox, each as
  recorded): every line of each timeline as the viewer makes it (`h3-map-view sequences`; the
  unit tests replay all but the `screen` lines without the original files), and every kept screen
  of the original against ours outside the panels, 181 frames (`idle-cycles` 40, `idle-objects`
  45, `scroll-0` and `scroll-1` 22 each, `scroll-2` 24, `scroll-limits` 28), all exact
  (`timelines/<id>-frames.txt`). The edge cursors' and the arrow's frames and hot spots are
  among them (the cursor over the frame's left border at (2, 300) and in the top-left corner).

### 3.7 The walk over time, headless (M2 T3a)

§3.5 built into `h3_map_view::timeline` (*provisional*) and checked against T1b's seven walk
sequences, every line but the `screen` ones (the walk drawn is T3b's). Read this time, filling
§3.5's gaps (`docs/exe/map.toml`):

- **A walk's waits run the tick.** `wait_until` returns at once when its time has passed, else
  runs `adventure_tick` at each turn of its wait (0x4F83A3): the mouse poll and the palettes'
  step, nothing else; so during a walk the cursor's point follows the mouse on its 33-ms grid, and
  no idle message runs (no redraw, no objects' step, no edge scroll).
- **The cursor** is hidden from a walk's start (`mouse_hide`: a key's at 0x409013, after the path's
  clearing; the path's at 0x407E7C, after the lift) to its end (`mouse_show`), whose last act reads
  the mouse's point now (`mouse_position_now`, its x even) and passes it to `adventure_mouse_move`
  with the tile under the mouse forgotten (+0xEC −1: a key's 0x4090F1, the path's 0x40812F): the
  cursor drawn there at once (`walk-4`'s 2 700, off the poll's grid), its frame by the move (off
  the view as §3.4; over it the cursor by the tile, M3's: `walk-4` recorded the arrow over a tile
  its hero could reach, which M3's reading of 0x40E91B on, the pathfinder's tile records, will
  explain; the timeline keeps the frame there).
- **A key with a path shown** clears it, draws and shows the view and steps the objects once if
  the redraw is due (0x408F52 to 0x409002: the deadline on by max(180, the lateness)), before the
  step.
- **The animation** (0x480391 on): the offset 0, the walking group (the switch 0x4805BC), +0x390,
  the hero's tile moved, the view's corner the tile less (9, 8) (immediates: the layout's
  `[scroll] middle`); speed 4 draws once without touching the offset; else n = 32 / the speed's
  pixels frames, each: its end now + 50, the offset less the pixels (speed 2's second frame 12),
  the view drawn and shown, `wait_until`; +0x390 cleared at the end. Pass 9 steps the frame by 1
  mod 8 (0x47FF74, a constant): the tables' `[map.walk]`. `mobilize_hero` (0x480914) runs before
  the facing is set (0x480946): a hero lifted by a step is drawn standing by its old facing once.

**The model** (*provisional*; what no recording shows is marked so):

- The selected hero stands on its tile at 0 (the harness sets a sequence's view through the
  minimap, which puts back the hero a turn's start lifted); a step or a walk lifts it, a scroll
  that moves the view puts it back (§3.4). Its tile and facing come from the live part at 0
  (`sequence::selected`: the viewer's only hero).
- **The path walked** is the path shown, ordered from the hero neighbour by neighbour (memory
  lists the arrows by level, row and column: `timeline::chain`), its green arrows (1 to 25) the
  steps: the first red one is a step the points do not pay (`walk-points`). Clicking shows the path
  to the tile (`make_hero_path`'s draw), the destination clicked again walks it. In M2 the paths
  are the original's, the first `arrows` line after each click (`Sequence::paths`); the engine's
  path and move from M4. A walk that stops short keeps the arrows left (the original builds the
  path again: `walk-points` the same) and draws the view.
- **The tile clicked:** the view's corner plus the point's x and y divided by 32 (§1.2's open
  point, M3's; the sequences click tiles' middles).
- **Actions given during a walk** wait; the mouse's point is read at once. A key's step leaves
  them to the loop's turn at its end, before the idle message; the path's walk empties the queue
  at its start (`input_flush` 0x4EC570 at 0x407E87, after `mouse_hide`: the messages behind its click
  gone) and after each step reads the messages up to the first key-down (a step's, a scroll's,
  Ctrl's alone), mouse button or control's message (kinds 1, 8, 0x20, 0x200: 0x407FA7), which
  ends it after that step: those before it are dropped, the one itself taken, those behind it
  left for the loop's turn after the walk (no flush on the way out); a hero flying over a tile it
  cannot stop on reads none (0x407F4D). Taken after the walk, a message's edge scroll and cursor
  read the mouse's point now, a click its own tile. A walk a message ended stands the hero
  (`hero_walk_end` 0x408002) before `make_hero_path` redraws the path left (0x4080C9; with the
  option Show Route, 0x698774, off it clears the path instead). M2 T6a (the reviewer's R1-1 to
  R1-5) corrected this paragraph, which had all the messages dropped. Not recorded: the
  sequences give only the parked pointer after a click; a key without a binding (Ctrl alone,
  M3's hotkeys) is no action of the client's yet.
- A key's step off the map's side does nothing (the engine's `hero_step_allowed` from M4; not
  recorded). The hero speed is 2 when a sequence gives none (`option_walk_speed`).

**Checked** (2026-10-27, no new recording): T1b's seven timelines line by line but the `screen`
boxes (`h3-map-view sequences`, the unit tests), the T2 six unchanged and `drawn` the same.
`walk-follow` gives its scroll speed since: T1b's original held 0 (Ctrl and Down moved one tile),
the harness otherwise leaves the game's setting.

**For T3b (the walk drawn):** done, §3.8.

### 3.8 The walk drawn (M2 T3b)

§3.5's lifted hero and the view's offset built into the scene (*provisional*), checked against
T1b's seven walk sequences recorded again, every line and every frame. Read this time
(`docs/exe/map.toml`):

- **The blend** (`draw_frame_blend` 0x47D4F0): it reads the frame as encoding 3 whatever its
  encoding (the heroes', flags' and boats' DEFs are all encoding 3: the gate), clips as the other
  drawers and mirrors across the frame's width; a literal pixel is blended half and half with the
  screen, `((c >> 1) & H) + ((d >> 1) & H)` with the half mask 0x6968A4 (0x47EF60's encoding 1
  blend, E17 design §3.3); a run of 5 is blended so with the colour argument when that is not 0;
  every other run is skipped. Both callers pass 0: `def_draw_group_blend` 0x47C120 (pass 9) and
  `draw_frame_transparent` 0x47EF60's encoding 3 (`h3_render`'s `Drawer::Translucent` draws that
  one plain, *provisional* there: the client lane's inbox). So pass 9 draws the lifted hero's
  pixels over themselves with each channel's lowest bit lost, and shows the hero half through
  whatever a later layer drew over it.
- **Pass 9's cut** (`view_moving_hero_step` 0x47FC80, drawn while `view_shown` 0x6989C0 is set):
  the 96 × 64 picture at (256, 232) loses, with the view's corner (x, y) and the map's side n,
  32 × −(x + 8) + the offset's x on the left when x + 8 < 0, 32 × (x + 11 − n) on the right when
  x + 11 ≥ n, 32 × −(y + 7) + the offset's y at the top when y + 7 < 0 and 32 × (y + 9 − n) at
  the bottom when y + 9 ≥ n. The boat's flag there is by its type and the hero's owner (pass 7's
  by the boat's colour); no waves.
- **Passes 5 and 7's cells**: on the view's tiles of columns 8 to 10 and rows 7 and 8 that lie on
  the map (off it nothing: 0x4115A1, 0x410C13), the cell (i − 8, j − 7) at its fixed place,
  clipped to it. Pass 5 (0x411A2C): after the tile's objects' shadows, before the heroes' and
  boats' (the hero's or the boat's shadow alone). Pass 7: row 8's cell after layer 1's heroes
  and boats (0x41133B), row 7's after layer 2's (0x4112FB); a tile whose list is empty (0x410D8F)
  draws all its heroes and boats, then the cell (0x411396), so a standing hero on the row below
  comes first there.
- **The map's side during a step** (MCL-Q7): passes 5 and 7 cut the lifted hero at the view's
  tiles, which the offset does not move, pass 9 on the left and the top at the map's side moved by
  the offset, on the right and the bottom at the tiles. So for a step's frames but its last, with
  a strip up to 24 pixels wide (32 less the first frame's pixels): a step away from a side shows
  the hero over the map's edge; one onto the last column or row leaves a strip of it on the map
  undrawn; one onto the first column or row shows that strip half transparent (pass 9's blend
  alone). A standing hero is cut at the map's side. No sequence walks there. **Built at T6b2b**
  (the user's answer): the client cuts all three at the map's side moved by the offset (the
  map's tiles' places with the offset: `scene`'s `map_shown`), passes 5 and 7 on every cell of the
  picture, on the map or not, so a strip off the map is cut and one on it drawn; at offset 0 the
  same as the original's. The original's cut behind `--bugs original` (`sequences` and `drawn`).
- **A late redraw** (0x408A88 to 0x408B46): each idle redraw steps the objects once, the deadline
  on by the larger of 180 and the lateness; one later than a step (only after a walk, when no idle
  message ran) leaves the deadline at the clock, so the loop's next turn, in the same
  millisecond, redraws again and steps once more. The screen shows the last draw: at `walk-1`'s
  key steps (200 ms) and every path walk's end the lifted hero's flag one frame on. T2's model
  stepped twice under one draw (no recorded frame showed an animated picture then).

**Built** (*provisional*): the live part's `walking` (the offset, the lifted hero by its index,
its group and frame; none in a still); the scene adds the offset to every tile pass's places (not
the gems' or the lifted hero's), leaves the lifted hero off its tile, draws its cells in passes 5
and 7 as read and the whole of it after the arrows (`Pass::Blend`, `draw::blend`), its flags'
frame the objects' step plus its own modulo the walk's 8 (passes 7 and 9); the timeline's idle
redraw draws again for each step it is late (`step_objects` one step); the gate asks the heroes',
flags', boats' and waves' DEFs for the walking groups 5 to 9 with 8 frames each (their standing
groups hold 1, the flags' 8: `check::walk_problems`); `h3-map-view sequences` draws a walk (each
draw puts the selected hero where the draw has it, lifted or on its tile, and the path's arrows
left), so its `screen` lines and `--frames` cover the walks; the harness's frame check and
`drawn` cover them (`walks` gone from `h3map.py`).

**VCMI 1.7.5** (`client/mapView/`): the moving hero is drawn once among the objects at its
interpolated place (`MapRenderer.cpp`; transparency only for fading, `objectTransparency`): no
second, blended draw over the later layers, and no cut by tiles at the map's side.

**Checked** (2026-10-27, T1b's seven recorded again in a fresh sandbox, each timeline as
recorded and its frame at 0 exact): every line of all 13 sequences as the viewer makes it, the
`screen` boxes included (`h3-map-view sequences`: 13 played, 13 as recorded), and every kept
screen of the walks against ours outside the panels, 269 frames (`walk-1` 64, `walk-2` 55,
`walk-3` 45, `walk-4` 32, `walk-follow` 36, `walk-points` 17, `walk-sail` 20), all exact
(`timelines/<id>-frames.txt`); `drawn` the 40 situations the same and the 13 sequences' 450
frames recorded. The first draw differed only at the walks' ends (the late redraw above).

### 3.9 The window (M2 T4a)

`h3-map-view window` built on E17's adapter (E17 design §8.1; ADR-0024), no reading of its own
(*provisional*):

- **The replay.** A sequence plays as `sequences` plays it (the same set-up and drawing, the
  binary's `paint`), a situation as a sequence without inputs (its kept live part, the speeds the
  original's defaults, the pointer parked, no hero selected). `h3_map_view::replay` runs the
  timeline's loop forward only as far as the clock needs (`at`: the change shown at a time;
  `after`: the next one, at most 600 s ahead) and forgets the changes before the one shown, so a
  window left open keeps a few. A change is a new view draw or cursor: on the map at least one
  each 180 ms (the redraw's step), so the window presents about 5.5 frames a second standing
  still. Past a sequence's inputs the map stands still on; a restart plays it again from 0.
- **The cursor** in a replay is the timeline's, drawn in the frame at its point, as the headless
  frames have it; the system's is hidden over the window (`Window::open`), the mouse does nothing
  (since T4b1 past a sequence's inputs the live pointer, the hardware or drawn cursor: §3.10).
- **The keys** (the user, 2026-10-26; `h3_map_view::keys`): one table of commands, each with its
  chords (a key by `h3_sdl::Key`'s name, `Ctrl+` holding either Control key). Defaults: the
  original's eight direction keys (§3.4) for the hero's steps and with Ctrl for the scroll; the
  window's own on the function keys and Esc (F5 pause, F6 the next frame while paused, F7
  restart, F8 vsync, F11 fullscreen, Esc quit), because the original's map takes the arrows,
  Home, End and the page keys and nearly every letter (M3's hotkeys), so `h3-view window`'s P,
  →, Home, F and V would clash there. A chord matches exactly (Ctrl and F5 is not F5). A user's
  `keys.toml` (`--keys`, else the working directory's) replaces the chords of each command it
  names; a chord bound twice, an unknown command or key, or Control alone is refused at the start.
- **Measured** (the sandbox, Xvfb, scale 1): a frame shows a median 5 to 9 ms after its time, p95
  8 to 11 ms (the software renderer's present; at scale 2, 21 and 35); every frame read back
  exact (log: M2 T4a).

### 3.10 The window's live input (M2 T4b1)

No new reading but `input_key_message` 0x4EC0E0's (T1a2: the message's code is lParam's bits 16 to
23); the rest is the timeline's (§3.4, §3.5) given as the player gives it (*provisional*).

- **When.** A sequence plays its recorded inputs as before (§3.9); after its last one, and in a
  situation (or a sequence without inputs) from the start, the map is live: the keys and the
  mouse are the timeline's actions (`Action`), given at the clock's millisecond, at least one
  after the change shown (`Replay::give`). Pointer moves of one batch of events are one move; a
  key's or a click's goes first. A restart plays the recording again, the input given forgotten.
- **The replay given actions as they come.** The window composes the next change's frame ahead,
  so the loop has run past the clock (up to 180 ms standing still). Each change keeps the loop as
  of its millisecond; an action at a time the loop ran runs it again from the latest change
  kept before that time (the one shown at least), the later changes forgotten, the next frame
  made again if it changed. Given so, the changes are a headless play's of every action (the
  replay's tests give five sequences' actions this way, the loop behind each at least once).
- **Live in the timeline** (`Start::live`): the pointer over the view is played (the recorded
  sequences refuse it: `Unplayable::OverView`): the edge zones that overlap the view (x 8 to 15,
  y 8 to 15) scroll there too, and the cursor is the arrow outside a zone (M3: the cursor by the
  tile under the pointer); a click off the view or on a tile no given path leads to only moves
  the pointer (M3: the panels; M4: the engine's paths). A click is the left button's press (which
  of press and release the original acts on is not read: M3).
- **A situation's hero.** The viewer's hero is selected when the live part holds one only
  (*provisional*: M4 selects by the game; `heroes-classes-*` have several, so no hero steps
  there); its path, as the still shows it (`path-objects`), is the timeline's at 0
  (`Start::path`, chained from the hero) and one of the paths a click shows: its destination
  clicked walks it, to its first red arrow (the points the situation gives).
- **The keys.** A held key's repeats are given (`Window::set_key_repeats`), as the original
  queues every WM_KEYDOWN (§3.4): a direction key held walks the hero on at the system's rate, the
  steps queued during a step as the timeline queues them (no bound yet: the original's ring keeps
  63 messages and the hero's points end the walk, M4; MCL-Q10); the window's own commands act once a
  press. The keypad's eight keys are bound with the direction keys: the original's code is the
  scan code, the keypad's the same in either NumLock state, and SDL 3 gives the keypad's own
  keycodes in both (its default `SDL_HINT_KEYCODE_OPTIONS`), so the adapter names them
  `Keypad1` to `Keypad9` (no 5). F9 switches the cursor's kind.
- **The cursor.** The hardware cursor (default) shows the timeline's frame (the arrow, the edge
  cursors), set when its change falls due, hidden while a walk hides it; the frame draws none,
  and a change that moves only the cursor's point presents nothing. Drawn (`--drawn-cursor`, F9),
  it is drawn in the frame at the mouse poll's point (§3.4), not while that point lies off the
  frame (the window's black border, the pointer outside: `(-1, -1)`, where no edge scrolls). SDL's
  failure to make the hardware one falls back to the drawn.
- **Checked** (Xvfb, two cores, scale 1, driven by `xdotool`): `--check` reads every frame back
  and at the end plays every action headless (`timeline::play`) and compares each change shown;
  both exact in every run: `path-objects` with Ctrl and Right, Ctrl and the keypad's Right
  (NumLock off) and 6 (on), Ctrl and Down (the view (0, 0) to (6, 2)); the east edge, then the
  west zone over the view; the path's destination clicked (the walk to its first red arrow, the
  view centred at (−1, −5)) and Down; Right held 1.5 s (23 steps, the hero walking on to the
  map's side); the same with the drawn cursor; `walk-1` with the mouse at the east edge and
  Ctrl+Down during its inputs (not taken), live after them (the pointer given where it lay: the
  view scrolled to the limit, (26, 9)); `walk-sail` at scale 2 with the drawn cursor. F9 three
  times: drawn. The 13 sequences as recorded, `drawn` the same (the window's changes do not
  touch `sequences`).
- **Measured** there: a frame a median 5 to 7 ms after its time; a frame at an input's time
  (composed once the input came) 37 to 60 ms, the view's composition in this sandbox (30 to
  60 ms for a whole view, the scene and the compositor of M1; a scroll redraws it whole); a walk
  at hero speed 2 frames late as much, its frames due faster than they compose here. An
  incremental scroll (the view moved by whole tiles, only the new strip drawn) would cut the
  input's delay (*provisional*; T4b3 makes the whole view's drawing fast first).
- **On the user's machine** (M2 T4b2: Windows, 2560 × 1440 at 100 %; the cross-built
  `h3-map-view.exe`, 9.0 MB, system DLLs only; `clients/out/win/map-measure.cmd`, five runs,
  the `.cmd` naming `--map` and `--install`, as the exe looks for its data where it was built):
  SDL took Direct3D 11 at scale 2 (1600 × 1200) and WASAPI (48 kHz, 480 frames a buffer).
  `--check`: `walk-1` 203 of 203 frames read back exact, `path-objects` with the user's keys,
  clicks and edge scrolls 345 of 345, each change shown the headless loop's (both runs).
  `walk-1` (vsync off and on): a frame a median 0.4 ms after its time, p95 0.6, the start's
  26 to 32 ms. `path-objects` with the user's input (vsync off): a median 2.6 ms, **p95 99 ms**,
  87 frames passed over in 30 s; a frame at an input's time a median 13.5 ms, p95 66; with
  `--check` (the read-back) a median 32 and 49. The view's composition is the cost: one view
  of `path-objects` (1564 draws) takes 775 million instructions (callgrind, sandbox), 92 % of
  them `draw::flagged` and `draw::shadow_pass` visiting every pixel of every frame, transparent
  ones included, with a division per pixel; a walk's frames are due every 50 ms. **T4b3** makes
  it fast, its frames unchanged. The user, watching and listening: a held direction key walks
  the hero on well after its release (the original's too: MCL-Q10), the keys step through
  obstacles and a click shows no new path (the engine's from M4 and M5), the sound's
  transitions not clean (compared by ear once whole maps load: M4).


### 3.11 The map's sound (M2 T5)

Path read: the terrain music's requests (`music_track_play` 0x59A1E0 and its callers on the map:
the manager's start 0x419990, `hero_select` 0x417B20, `hero_move_step` 0x4805E0, the walk's and
the key's ends) and the music's thread (M2 T6a2: `play_music` 0x59ACB0, `music_thread`
0x59A840, `music_stop_fade` 0x59B080; E17 §8.2 for the rest); the step sounds' loads
(`adventure_manager_start` 0x407474) and plays (0x407CD0, 0x481344, pass 9's 0x47FF88, `hero_walk_end` 0x47F7D0); the
ambient sounds (`ambient_sounds_at` 0x4183D0, `ambient_sound_visit` 0x418C10, `tile_ambient_sound`
0x418620, `sample_control` 0x599F40) and their callers. Not read: the mine record's set-up (its
`abandoned` byte +2 as H3API names it), a network game's turn start (0x419E00: the music and the
ambient sounds again 6 s later, 0x4088F9), the visits', battles' and the computer players' sounds
(M4), the AI's turn music; the sound without the focus (M2 T6a2, the lane's inbox:
`window_procedure` 0x4F7C00 calls `sound_focus_lost` 0x599C40 on losing it, every effect sample
stopped and the music faded out by the stub 0x59A830, and `music_resume` 0x59AC00 on regaining
it, the track asked for again; the effects' return not read).

- **The settings.** Nothing plays with the effects' setting (0x698764) at 0 (`ambient_sounds_at`
  returns before its scan, `sound_volume` gives 0) or the music's (0x698760) at 0 (`play_music`
  returns); a volume v is v × (s + 1) / 10 at a setting s (E17 §8.2). The harness's settings have
  no sound (`clients/capture/setup.sh`), so the original's cues are not recorded (below).
- **The music** (the manager's +0x58: the terrain whose music was asked for last). Asked for by
  the terrain of the tile at the view's middle (the corner plus (9, 8); off the map, tile (0, 0)'s)
  when the map's screen starts (0x419BFD); by the selected hero's tile's when `hero_select` centres
  the view on it (with its flag c: the turn's start, a step's start after a scroll, 0x418104);
  at a step's end by the view's middle's (the hero's new tile: 0x4812C6), each only when that
  terrain differs from +0x58, which it then becomes. `terrain_music_tracks` 0x678330 gives the
  track, `music_track_play` its name (0x684AE8: track 2 Water, 3 Grass, 4 Snow, 5 Swamp, 6 Lava,
  7 Sand, 8 Dirt, 9 Rough, 10 Underground; rock's 0 plays nothing, the music on) through
  `play_music` with a loop count of 0 and the effects kept (below: dropped when that name is
  pending or playing). The walk's and the key's ends ask for +0x58's again (no change).
- **The music's thread** (M2 T6a2; three of the manager's critical sections: +0xC0 the names,
  +0x90 Miles' calls, +0xA8 a thread's whole run). `play_music` (nothing with the music's setting
  at 0) drops a request whose name is the pending one (`music_pending_name` 0x6A3394, 0x59AD0D),
  or the one the thread opened last (`music_playing_name` 0x6A3290) while its stream
  (`music_stream` 0x69FE78) plays (`AIL_stream_status` 4, 0x59AD43 to 0x59ADA6): a fading track
  plays. That name with its stream not playing (one missing, one played once and ended) is asked
  for again once +0xA8 is free (the game's thread waits for it, 0x59AE2B). Any other name becomes
  the pending one (0x59AF1F) and starts a thread of its own (0x59AEFD). A thread enters +0xA8 first
  (0x59A859) and leaves it at its end (0x59ABE7): threads run one after another, and one that finds
  no pending name ends (0x59A883). Else `music_stop_fade` (+0xA8 too): a stream open and not yet
  faded (+0x8C, set when a thread opens one, 0x59A9CF) has its place (`AIL_stream_position`)
  kept under its name *before* its fade (0x59B146 to 0x59B1A8; `music_positions` 0x69FEC0, 0x108
  bytes an entry, one appended for a new name), then fades from its volume (`AIL_stream_volume`
  0x59B1E2: the whole, since a fade-in holds +0xA8) by a tenth every 100 ms (0x59B21B, an f32 sum
  truncated: 114, 101, 88, 76, 63, 50, 38, 25, 12 from 127, `Sleep(100)` between), paused after
  900 ms (0x59B26A). The thread closes it (0x59A8F4), takes the pending name *as it is then*
  (0x59A96A, cleared at 0x59A9A8: the last asked for during the fade), opens it, starts a looping
  one silent where its name's last fade began (0x59AA82) and fades it in (E17 §8.2: 12, 25, …
  114, then 127 after 900 ms) still holding +0xA8 (0x59AB42 to 0x59ABB1); it starts a stream only
  while the game is the active application (`app_active`, 0x59AA15). So a track asked for
  during a fade-in waits for its end, then the new track fades out from the whole; one asked for
  during a fade-out replaces the pending one; and the old track asked for again during its own
  fade-out is dropped: grass, dirt and grass again within 900 ms keeps Dirt playing on grass
  until the music's terrain next changes (**MCL-Q12**). **Built at T6b2b** (the user's answer,
  `h3_ui::music`'s filter, `h3_map_view::music`'s until M3 T1a; the original's behind `--bugs
  original`): the client drops the track opened last only while nothing else is pending; during its own fade-out it becomes the
  pending one (it comes back where its fade began, after the fade); during its fade-in or while
  it is opened (the same slip a fade-in later: grass, dirt during Grass's fade-in, grass again;
  *provisional* at T6b2b, confirmed by the user at M3 T0) the pending one is dropped and it plays
  on, the thread waiting finding no pending name (0x59A883). The fades run on the thread's real time,
  not the game's clock. The place is Miles' stream position, an offset in the file: whether it
  is what is heard or Miles' read point is Miles' own (not read; the viewer keeps what is heard).
- **The step sounds.** `horse00.wav` to `horse10.wav` loaded into the manager's +0x360 to +0x388
  (0x40747A, `horse%02d.wav`): by the terrain 0 to 9, and +0x388 for a hero flying (its Fly
  spell's level +0x112 not −1, or Angel Wings, artifact 72); a hero in a boat (+0x105 bit 18)
  takes the tile's terrain's, the water's (0x481351, 0x407CCB). Each plays looping (+0x30 = 0)
  at its loader's volume, 127, in category 0 (sample handle 0, E17 §8.2). The path's walk starts
  the start tile's at once (0x407D03: `step_sound_playing` 0x6968E0). A step's end changes it only
  where the music's terrain changes (0x4812D9 jumps past both): the new tile's plays at once when
  the lifted hero's frame +0x200 is 0 (the one playing stopped), else waits in `step_sound_next`
  0x6968E4 for pass 9's step of the frame to 0 (0x47FF88). The walk's last step stops it first
  (0x481216, the frame 0 then, so a new terrain's starts at once) and `hero_walk_end` stops it
  again (0x47F825); a key's step (no `hero_move_loop`) starts none but a new terrain's, stopped
  at its end. So a walk on one terrain plays one loop from its first step to its end, and the
  direction keys' steps play none (T4b2: the manager's +0x360 to +0x388 are read nowhere else,
  `exe.py grep`). `Heroes3.snd` also holds `HORSE20` to `HORSE27` and `HORSE29` (none for 28, the
  water's index: perhaps the roads' sounds, *inferred*), longer than their 00 to 09; the
  executable's only `horse` string is the loader's, so they never play (**MCL-Q11**, the user at
  T4b3: "a few terrain-specific samples which the original doesn't seem to play").
- **The ambient sounds** (`ambient_sounds_at`, this = the manager; a tile and a stop flag): four
  slots at +0x228 (8 bytes each: a sound's index, −1 none, and its distance, 127 none), the
  sounds loaded on demand at +0x248 (70 pointers), +0x224 a bit set of the sounds found. Called
  with the view's middle: by the map's start (0x419C22, stop 1), by `hero_select` whatever its
  flags (0x4180FF, stop 1: the turn's start, the walk's lift after a scroll, a step's), by each
  step's end (0x48141B, stop 0); the scroll calls neither it nor the music (`view_scroll`
  0x4195C0). Its run:
  1. each slot with a sound: stopped and emptied with the stop flag; its distance 127;
  2. with the effects on and a tile, +0x224 = 0, then two passes; each visits the middle at
     distance 0, then the rings at Chebyshev distance 1 to 3, each ring's 8 d tiles by i from 0
     to 2 d − 1 four at a time: the top row's (x − d + i, y − d), the right column's (x + d, y − d
     + i), the bottom row's (x + d − i, y + d), the left column's (x − d, y + d − i);
  3. a visit (0x418C10) of a tile on the map with a sound s (below): a slot holding s takes the
     smaller distance and sets +0x224's bit 1 << s (on x86 the shift's count is s mod 32); in the
     first pass nothing else; in the second the slot of the greatest distance above this one (the
     first such) is taken: its sound stopped, s at this distance, loaded (`wav_load` of
     `ambient_loop_names` 0x65F794), played looping in category 3 (the 8 handles 6 to 13) at
     `ambient_volumes` 0x63A64C[distance] (32, 28, 20, 10 for 0 to 3), and +0x224 ^= 1 << s;
  4. each slot then: a distance above 5 (not found) stopped and emptied; a sound whose bit is set
     takes `ambient_volumes`[its distance] (`sample_control` with 100 → `AIL_set_sample_volume`).
  So at most four loops, the nearest by their tiles; kept loops change only their volume as the
  view moves, unless the stop flag restarts them all. With the shift's count taken mod 32, two
  sounds 32 apart share a bit: one's XOR in the second pass clears the other's, which then keeps
  its earlier volume (**MCL-Q8**). **Built at T6b2b** (the user's answer): the client keeps a bit
  for each of the 70 loops (`sound`'s set in 128 bits), so each loop found takes its distance's
  volume; the original's 32 behind `--bugs original`.
- **A tile's sound** (`tile_ambient_sound` 0x418620, on the tile's record: §2.1, the E08 survey's
  §2.5 for which object a tile records): water (terrain 8) of a picture below 21 (the shore's,
  one more than the coast's 20: **MCL-Q9**; built at M3 T0, the user's answer: the client's the
  coast's water, below 20, the original's 21 behind `--bugs original`, `TileSounds::build`)
  `loopocea` (69), before any object; else on an entrance tile (+0xD bit 0x10) by the class
  +0x1E as the game loads it (`object_class_values` 0x63A6E4: 219 as 33, 220 as 53, 221 as 99,
  223 as 21, 230 as 46, 165–189 as 114–138, 190–205 as 143–161) and the subclass +0x22: the
  table's `classes` (`clients/map/assets/base.toml`, `[map.sound]`); a creature bank (16), the
  fourth dwelling (20) and a garrison (33) by subclass; a dwelling (17) by its creature
  (`dwelling_creatures` 0x63D570[subclass]), `LoopSwor` for one the switch lacks; a mine (53)
  `LoopCave` when abandoned (H3API's +2: the map's 220 or subclass 7, *inferred*), else by its
  type (+1, the subclass); without an entrance a volcano (158) `LoopVolc`, and a tile of class 0
  the special ground's (`tile_special_ground` 0x4FCE20: the first of 21, 46, 222, 224 to 229 and
  231 among its objects from the top: 21 `loopcurs`, 46 `LoopMagi`, the others none). A hero's
  tile (class 34: a hero standing; the lifted one restores the class under it, 0x4D7746) and
  every other class has none.
- **The viewer** (*provisional*): `h3_map_view::sound` holds the tiles' sounds (`TileSounds`,
  from the art, the live part, the draw lists and the tables: the class a tile records by
  0x505610's walks) and the original's state (`Sound`: +0x58, the slots, the step sound playing
  and queued); the timeline's loop drives it at the moments above and gives its cues (`Cue`:
  the music asked for, a step sound started or stopped, a slot's loop started, stopped or its
  volume) with their milliseconds. The window plays them (below); `sequences` does not write
  them (no recording to compare with). A hero flies in no situation (M4: the engine's).
- **The window** (`audio`, *provisional*; E17 §8.2's way): the WAVs the tables name decoded when
  the map loads, a track on a thread of its own when first asked for (the map's first before
  the window opens); one stream for the music, one for the step sound, one per ambient slot;
  the volumes as SDL gains of v / 127 (both settings at their highest). The music (M2 T6a2):
  `h3_ui::music` (`h3_map_view::music` until M3 T1a) models the thread above on a clock of the
  frames the device has played (`Music`: the pending and playing names, the stream opening, fading in, playing or fading out,
  the places; `Queue`: the model at the frames heard and a copy at the frames queued, a second
  ahead), its runs' samples scaled by their volume; a request lands at the frames heard and,
  taken, takes back what is queued (the stream cleared, queued again from there), so the fades
  follow the device's time, as the original's thread follows real time. A track still being
  decoded holds the music (the original opens its file at once; a terrain track decodes well
  within a fade-out), one not of 44.1 kHz stereo is refused (the nine terrain tracks are). F2 and
  F3 switch the music and the effects (their gains held at 0, the loops running on), `--mute`
  plays nothing, `--cues` writes each cue played.
- **Checked** (Xvfb, SDL's `disk` driver writing the mix; the tracks decoded by `ffmpeg` and
  aligned with the mix, correlation 1.0): `objects-myth` the Dirt track fading in by 100 ms steps
  to its whole after 900 ms (each step's gain measured on the samples); `seq-walk` driven by
  `xdotool` (three steps north-west and one west onto the dirt patch): the Grass track's fade-out
  114 … 12 by 100 ms steps, then the Dirt track from its start fading in, sample-exact; the step
  sound's beats from its cue to its stop (`walk-follow`); the cues of the shore (`walk-sail`:
  `horse08` sailing, `loopocea` restarted by the lift), the volcano and medusas
  (`objects-flat`), the gold and gem mines (`owners-objects`); every frame read back exact and
  every change shown the headless loop's in each run.
  M2 T6a2, the music on the model (the same way; the mix located in the tracks by normalised
  cross-correlation, 100 ms at a time): `seq-walk` onto the dirt patch and back 326 ms later:
  Grass faded out, Dirt from its start faded in and playing on (the second request dropped);
  back 1 320 ms later, during Dirt's fade-in: its fade-in ended, its fade-out from 114 at once,
  then Grass where its fade-out began, fading in; every window at the whole volume correlation
  1.0000.

### 3.12 The view drawn fast (M2 T4b3)

No reading; the frames as before, pixel for pixel (*provisional* in what follows).

- **The drawers** (`h3_map_view::draw`): a frame's pixels go row by row: a row off the canvas or
  outside the clip and the screen is passed over, the row's first and last columns inside them
  worked out once, the screen's row (`Screen::pixels_mut`, `h3-render`'s, additive) walked
  forwards or, mirrored, backwards beside the frame's indices and runs, each drawer's own test per
  pixel inlined; a frame whose runs end early (only a test's) is padded with literal ones.
- **The composition** (`compose`): the frame's picture drawn from its rows in the viewer's
  colours with no copy of it (the 480 000 pixels before were cloned and drawn through the
  general blitter's division per pixel); a DEF's cycled palette kept by its name as the scene
  spells it (no upper-cased copy a piece); the flag palette's colours once a scene; the batch's
  names looked up as they are when they hold no lower-case letter.
- **The pictures** (`paint`): every asset a view of the map draws at 0, each level scanned view
  by view, decoded when the map loads (a path's arrows shown later are still decoded when first
  drawn).
- **Measured.** Callgrind (sandbox, `render` of `path-objects`): the composition 775 million
  instructions before, 24.6 million after (31 times fewer; `draw::flagged` 392 → 8,
  `draw::shadow_pass` 326 → 9). The window reports how long each frame took to make (the view,
  the cursor, the widening to 32 bits): in the sandbox a median 5.2 to 5.7 ms, p95 9. On the
  user's machine (`map-measure.cmd` again): a frame made in a median 4.0 to 5.8 ms, p95 4.5 to 7.9,
  at most 10.7; `path-objects` with the user's input a frame a median 0.6 ms after its time, p95
  8.2, **none passed over** (87 before), an input's frame a median 7.8 ms (13.5); `walk-1` as
  before (median 0.5, p95 1.6). Every frame read back exact (162, 393), every change the headless
  loop's.
- **Left** (the user: "it will do for now, but we should check back in at some point"): a frame
  of a near-empty view still takes about 4 ms on the user's machine: the frame picture's 480 000
  pixels, the terrain's tiles, the widening of 480 000 pixels to 32 bits, the painter's two
  copies of the screen, the scene built afresh; the objects' frames are still visited pixel by
  pixel, transparent ones included. Spans of a frame's drawn pixels made once, the frame picture
  converted once a viewer, the screen not copied, and a scroll or a walk redrawing only what
  moved are the next steps (the queue's inbox).

## 4. The adventure screen (M3, from T2a, 2026-10-28)

### 4.1 The dialog at a map's start (M3 T2a)

Read: `adventure_dialog_create` 0x401510 in full, `adventure_full_update` 0x417420's panel calls,
the resource bar's constructor 0x558BA0, and every site that sets a button's state (below);
checked against the running original's dialog (`h3mem.dialog_items` on `heroes-objects` and
`path-objects`: every item's id, box and state as read). *Provisional* where marked.

- **The dialog** (the manager's +0x44, H3API's `H3AdventureMgrDlg`; vtable 0x63A5E4): the base
  dialog at (0, 0), 800 × 600 (0x5FE9F0 with flags 1), its items added in this order (0x402157:
  `AddItem` with -1, each in turn; draw order, E17-design §9.7). Areas (0x44FF10: draw nothing):
  0 the map (8, 8, 592 × 544; +0x50), 1 the minimap (630, 26, 144 × 144; +0x4C), 2 the status
  window (605, 389, 188 × 182). Buttons (0x455EF0: frame 0, click frame 1, flags 2; the hotkey a
  scan code into +0x48, 0x404200): 3 the kingdom overview `iam002.def` (679, 196; K 0x25), 4 the
  underground `iam010.def` (711, 196; U 0x16), 5 the quest log `iam004.def` (679, 228; Q 0x10),
  6 sleep `iam005.def` (711, 228; Z 0x2C), 7 move hero `iam006.def` (679, 260; M 0x32), 8 cast a
  spell `iam007.def` (711, 260; C 0x2E), 9 the adventure options `iam008.def` (679, 292; A 0x1E),
  10 the system options `iam009.def` (711, 292; O 0x18), each 32 × 32; 11 next hero `iam000.def`
  (679, 324; H 0x23) and 12 end turn `iam001.def` (679, 356; E 0x12), 64 × 32; 13 and 14 the hero
  list's arrows `iam012.def` (609, 196) and `iam013.def` (609, 372), 64 × 16. The hero list: five
  pictures without a picture (0x4502D0, flags 0x800) at (617, 212 + 32 i), 48 × 32, ids 15 to 19
  (+0x70: the portraits, `hpsxxx.pcx` in an empty slot, 0x403662) and five more there, ids 39 to
  43 (+0x84: the selection's frame `hpsyyy.pcx`, 0x4033D7); DEF items (0x4EA720, flags 0x10) of
  `imobil.def` at (610, 213 + 32 i), 8 × 32, ids 20 to 24, and `imana.def` at (666, 213 + 32 i),
  ids 25 to 29. The town list: its arrows `iam014.def` (747, 196) and `iam015.def` (747, 372),
  48 × 16, ids 30 and 31; DEF items of `itpa.def` at (747, 212 + 32 i), 48 × 32, ids 32 to 36. Then
  the status bar 200 (0x5BC760: `AdRollvr.pcx` behind `smalfont.fnt`, colour 1, centred, at (8,
  556), 592 × 18; +0x60), a text 37 (0x5BC280: `medfont.fnt` at (54, 100), 520 × 440, empty;
  +0x54) and the chat's edit 38 over the status bar (0x5541A0; +0x58; draws nothing while not
  editing, not read). The status window's panel (+0x98) is built later (0x415DE0: a hero's, a
  town's, the kingdom's; T5); the resource bar is a panel of its own (+0x5C, 0x558BA0): at (3,
  575), 794 × 22, its picture 1000 `aresbar.pcx` from its corner in the viewer's colours
  (0x450520), the seven amounts 1001 to 1007 at (37 + 84 i, 3), 50 × 18, and the date 1008 at
  (607, 3), 180 × 18 (texts in `smalfont.fnt`, T5), and seven areas 1009 to 1015 for the hints.
- **A full update** (0x417420, after `AdvMap.pcx`): the buttons 3 to 12 in the viewer's colours
  (0x403F60: 0x4569E0 puts the players' palette's 32 entries of the viewer over the DEF's
  224–255), the hero list (0x4032E0) and the town list (0x403420) rebuilt, the quest log's state
  (0x4039B0), the status window chosen (0x415DE0), the resource bar (0x403F00), the dialog's items
  redrawn (vtable +0x14 with the ids −65535 to 65535: all, in order), the selection's frames
  (0x4038C0); then the view and the minimap. Neither the lists' arrows nor the status bar take the
  viewer's colours.
- **The buttons' states** at the human player's turn (a state's bit 8 or 0x20 draws the disabled
  frame 2, E17-design §9.7): the kingdom overview, both options and end turn enabled; the
  underground disabled on a map of one level (`adventure_manager_start` 0x4070ED: state 8), its DEF
  `iam010.def` on the surface and `iam003.def` underground (0x403C40, table 0x65F22C; the view's
  level, then the viewer's colours again); the quest log disabled (0x4039B0: its vtable +0x24,
  state 0x20) unless a seer hut's or quest guard's quest (game +0x1FBD4, +0x1FBE4) was visited by
  the viewer; sleep (0x403BA0) and cast a spell (0x403BF0) enabled with a hero selected, disabled
  with a town selected (`town_select` 0x417830), sleep's DEF `iam011.def` while the hero sleeps
  (+0x11C; 0x403CC0, table 0x65F238, the hotkey W 0x11 instead); move hero enabled when the
  selected hero's path has a green arrow, a step it pays today (`make_hero_path` 0x418DD0 with its
  third argument: 0x4191A2, from a click on a tile `tile_click_path` 0x40AA17, a hero's selection
  0x417FE2), disabled without a destination (0x418EB7) or a path (0x4191ED), with a town selected
  and at a walk's start (0x407E2C, a key's step 0x408F0B); next hero enabled when a
  hero of the player's is awake and can still move (`next_hero_find` 0x4BAA40: from the one after
  the selected, round to it; `hero_can_move` 0x4E5F30: its points at least the cheapest step from
  its tile, 0x4B1880), set at the view's entry (0x419C53), after 0x40EC90 rebuilds a path (0x40F208)
  and at a walk's end when the hero cannot move on (0x4081C4); the hero list's up arrow disabled at its top
  (+0x64 0), its down arrow when the list has no more than five heroes below the top (the player's
  +0x1 heroes, 0x4032E0), the town list's the same (+0x68, the player's +0x3E towns, 0x403420).
  A change is sent with the state 0x4008 or 0x4000: the item redrawn and its rectangle shown at
  once (0x602370 through the window manager for move hero; 0x5FEED0, 0x5FEF60 for the rest).
  VCMI 1.7.5 (`client/adventureMap/AdventureMapShortcuts.cpp`) agrees on each:
  `optionCanToggleLevel` (two levels), `optionCanViewQuests` (quests not empty),
  `optionHeroSelected` (sleep, cast), `optionHeroCanMove` (a path whose next node is this turn's),
  `optionHasNextHero` (a next wandering hero), the rest `optionInMapView`.
- **Seen in the sequences** (M3 T2a's recordings, `Mem.buttons` every millisecond; the reading
  above leaves the order of these messages open, so the timeline follows the recordings,
  *provisional*): a path's walk keeps move hero as the click set it (the start's disable is not
  seen) until its end, which turns it off on the destination (the path cleared) or sets it by the
  rest's green arrows; a key's step turns it on when it lifts a hero not lifted (the selection's
  path, the step's own) and off once the hero is lifted; next hero turns off at a walk's end
  that leaves its only hero unable to move (`walk-points`). A state the original set before the
  harness wrote a hero's points stays as set (`walk-sail`: next hero off at 0 with 1 500 points):
  the live part carries the states memory holds (`buttons`).
- **Seen** (the captures): the disabled buttons show frame 2 (a blank red plate where
  `AdvMap.pcx` paints the enabled icon), the list arrows' frame 2 a plain plate; the cursor
  parked at (700, 300) is drawn over the adventure options' button (the arrow, frame 0), and the
  status bar shows that button's hint (T6).
- **Built** (*provisional*): `adventure.toml`'s `[dialog]` (h3_ui::ui's format) and
  `[resource_bar]`; `h3_map_view::panels` turns the live part's `kingdom` (the viewer's heroes in
  the list's order, each asleep or not and able to move or not, its towns, the one selected, the
  lists' tops, whether it has a quest), its `buttons` (move hero's and next hero's states, as the
  original or the timeline last set them) and the view's level into the items' states, DEFs and
  colours; the scene draws `AdvMap.pcx`, the dialog's items, the resource bar's picture, the view,
  then (a still) the cursor at the live part's pointer. Not drawn yet: the lists' contents (T4),
  the minimap (T3), the status window, the amounts and the date (T5), the hints (T6):
  `h3map.py`'s `panels` mask keeps their rectangles.

### 4.2 The frame made once (M3 T2b1)

No reading; every frame as before, pixel for pixel (*provisional* in what follows). The user's
asks at the cut (2026-10-28): a spatial structure for the draw objects (a jump to a town or
anywhere on the map, HotA's 252 × 252 maps and larger), the widening computed once.

- **Measured before** (callgrind, the window's frames on `path-objects`, 111 frames): 101 million
  instructions a frame: the widening to 32 bits 63 (`PixelFormat::unpack`'s per-channel loops,
  pixel by pixel), the composition 28.5 (the frame picture, the dialog's items and the resource
  bar about 9 of it; the view's drawers 19), the scene 7.3 (each object's shape found by its
  DEF's name for every piece, the draw lists built over the whole level), the painter's two
  copies of the screen 1.3. The sandbox: a 144 × 144 map 3.5 ms a frame more than `path-objects`.
  The user's machine (`map-measure-t2b-before.txt`): a frame made in a median 4.2 ms on `walk-1`,
  3.3 (p95 11.3) on *Tale of two lands*, 3.3 (p95 7.3) on `path-objects` with the user's input.
- **The widening** (`h3_render::screen::Widening`): every pixel of a format widened once, a table
  of 65 536 entries (256 KiB), `Screen::xrgb`'s values; `widen` into a kept vector, `widen_onto`
  a screen onto a part of a frame. The window makes each frame into a vector a frame no longer
  shown gave back (`Maker`), none allocated afresh.
- **The composer** (`compose::Composer`, the painter's): the scene's leading draws that do not
  move with the view (the frame's picture, the dialog's items and the resource bar) are drawn on
  black once as the background while they stay the same (they change with the buttons' states:
  at a walk's start and end, a selection); each scene restores from it the rectangle its other
  draws and the last scene's cover (the view's clip and the gems': (6, 6) to (602, 554)) and draws
  them over it. Each DEF's palette is kept for its cycles' steps (made again when they step), its
  sprite and palette looked up once a scene by the name as spelled. `compose` is a new composer's
  first scene.
- **The spatial index** (`scene::TileIndex`): a level's draw lists in a dense grid of its tiles
  (the map is a bounded grid: a tile's list at once, whatever the map's size, a perfect hash) and
  each listed object's look (its DEF, shape, flatness, flag's entry and phase), worked out once,
  the level's map-wide checks (every listed object's shape) made as it is built. `scene_in`
  visits only the view's tiles: a jump costs a scroll's frame, a 252 × 252 map a 36 × 36 one's.
  `scene` builds the view's level's index and calls it. The painter builds every level's at the
  map's load; it holds while the live part's objects stay as they were (their pictures, owners,
  phases and whether they stand: no draw changes them before M4, which builds it again or
  updates a tile's list).
- **The painter and the window:** the view drawn into the composer's frame, no copy of it kept;
  a frame is that frame widened, then the cursor's rectangle cut out, the cursor drawn on it and
  widened over the frame (`Cursor::rect`, `Screen::cut`). A scene's names are decoded only when
  a spelling is new (`batch::spelled`). The shadows' masks are worked out once a draw.
  `--check` also compares each frame made with the screen's own pixels widened one by one (the
  line `widened`).
- **Measured after:** callgrind on `path-objects` 25.2 million instructions a frame (101): the
  view's drawers 16 (the shadow pass 6.5, the flagged drawer 6.3, the raw one 3.4), the widening
  3.9, the scene 2.3. The sandbox's medians: `walk-1` 6.4 → 1.3 ms, `path-objects` 10.8 → 4.5,
  *Tale of two lands* 14.3 → 4.2 (the map's size no longer counts). The user's machine
  (`map-measure-t2b1.txt`): a frame made in a median 0.7 ms on `walk-1` (4.2; p95 0.9 to 1.0,
  4.5 to 4.8), 2.3 on *Tale of two lands* (3.3; p95 2.7, 11.3), 1.7 and 0.2 on `path-objects`
  with the user's input (3.4 and 3.3; p95 2.9 and 2.5, 7.1 and 7.3); an input's frame shown a
  median 3.5 ms after its time (8.8); every frame read back exact and widened as the screen's own
  (1 051 on `path-objects`).
- **Left** (T2b2): the drawers visit every pixel of a piece, transparent ones included; every
  frame draws the whole view and widens and uploads the whole screen, though an idle frame changes
  only the animated objects and the cycling water and a scroll moves what it keeps.

### 4.3 A frame redrawing only what changed (M3 T2b2)

No reading; every frame as before, pixel for pixel (*provisional* in what follows). The user's ask
at T2b's cut: draw efficiently whatever moves, the draw objects found by T2b1's spatial index; T2b2
kept whole ("no cut", 2026-10-28).

- **Measured before** (T2b1's build, the sandbox): callgrind on the window's frames 25.1 million
  instructions a frame on `path-objects` standing, 10.1 on `walk-1` (§4.2: the view's drawers 16
  of the 25, every pixel of a piece visited, the transparent ones too); every frame drew the whole
  view, widened the whole screen and uploaded it to the texture.
- **The spans** (`draw::Spans`): each frame's rows as spans of literal pixels and of runs of one
  index, worked out once as its DEF is decoded (`Batch::drawable` gives a frame with its spans),
  the runs of index 0 left out (no drawer draws them); each drawer visits only its spans, cut to
  the canvas, the clip and the screen. The gems go through the raw drawer (the plain drawer with
  runs skipped draws their frames the same), so that a cell can cut them.
- **The cells** (`compose::Composer`): of the draws over the background, each has its rectangle
  (a piece's clip, a gem's canvas). The view's 32 × 32 cells (their corners at the view's origin
  plus whole tiles, `Scene::view`'s `ViewPlace`: the clip, the origin and the tile; a scene
  without one at (0, 0)) cover this scene's rectangles and the last's. A cell is left as it is when
  its draws (each draw whose rectangle meets it, in order) are the last scene's and their DEFs'
  palettes cycle the same; the others, merged into rectangles (each row's runs, a run below one of
  the same columns joined), are restored from the background and drawn again, each draw cut to
  them. Exact by construction: every drawer puts a pixel from the screen's pixel there and the draws
  over it. A new flag palette, a frame's picture or a dialog's item among the view's draws, or a
  new background (a button's state: a walk's start and end) draw everything again.
- **The move:** when the view's origin moved (a scroll, a walk's slide; the same clip and tile, a
  move smaller than the clip) and the background is one colour across the clip (the black of
  `AdvMap.pcx` there; only then is a moved pixel of the background the background), the clip's
  pixels move by as much first, and each cell is compared with the last scene's draws moved; a cell
  not wholly inside what the move keeps (the strip it brings in, the gems' corners) is drawn
  again. A jump moves nothing: every cell differs.
- **What changed, widened and uploaded:** the composer reports the rectangles a scene changed
  (`Composer::damage`); the painter numbers each composition and keeps the last 64's
  (`damage::History`). A frame widened (the painter's `Widened`) knows the composition it holds
  and its cursor's rectangle: the window's spare frames are widened again only where the
  compositions since changed and where both cursors lie (`Widening::widen_rect`), whole when the
  history no longer reaches. The texture likewise holds the frame last uploaded: a frame shown
  uploads only what differs from it (`Window::present_part`: `SDL_UpdateTexture` a rectangle at a
  time; past 64 rectangles their bounding one), the whole frame again after the window asked for
  a redraw (its contents may be lost).
- **The names:** the painter no longer collects a scene's names each frame (T2b1's `spelled`);
  an asset the preload did not decode (a path's arrows shown later, an object's other picture) is
  decoded when the composer finds it missing, and the scene drawn again.
- **The check:** `--check`'s `afresh` line (T2b1's `widened`) compares every frame made with the
  same frame composed afresh and widened pixel by pixel; the read-back checks the texture's
  uploads. The timing line says how much went to the texture (`uploaded`).
- **Measured after** (the sandbox; Xvfb's software renderer): callgrind 25.1 → 5.6 million
  instructions a frame on `path-objects` standing (the scene 2.3, the composer 2.9: its drawers
  1.0, the cells' lists and comparisons about 1.2; the widening 0.35), 10.1 → 3.2 on `walk-1`. A
  frame made in a median 0.9 → 0.3 ms on `walk-1` (25 % of the pixels uploaded), 4.0 → 1.0 on
  `path-objects` standing (5.5 %), 3.2 → 1.3 on *Tale of two lands* standing (11 %: its water
  cycles), 1.4 → 0.6 and 1.9 → 1.5 on them with input (`xdotool`: 9 scrolls, 37 steps, 4 clicks;
  29 % and 52 %).
- **The user's machine** (`map-measure-t2b1.txt` → `map-measure-t2b2.txt`, Direct3D 11, 1600 ×
  1200): a frame made in a median 0.7 → 0.2 ms on `walk-1` (24.9 % of the pixels uploaded), 2.3 →
  0.9 on *Tale of two lands* (p95 2.7 → 0.9; 11.1 %), 0.2 → 0.0 on `path-objects` with the user's
  input (p95 2.5 → 1.5; 54 %; the check's run 1.7 → 0.3); every frame read back exact, made as the
  same frame composed afresh, and shown as the headless loop's.
- **Left:** the scene built afresh each frame (2.3 million instructions, about half of what a
  standing frame costs now; its draws' names are strings, a few thousand allocations a frame); a
  button's change (a walk's start and end) draws the background and the whole view again; M4's
  changes to the objects (the tile index again, §4.2; the cells compare whatever the scene draws).

### 4.4 The minimap (M3 T3a)

Read: `minimap_draw` 0x412C40 in full but its view-world and network branches,
`tile_object_entrance` 0x4FCAA0, the arguments of three callers (`adventure_full_update`
0x41753A and `hero_select` 0x418046: the view's corner +0xE4, then 0, 1, 0, 0, 0; `view_scroll`
0x41973E: the corner, then 1, 1, 0, 0, 0, which T3a had as the others'), and `radar.def`'s load
(0x40742B into the manager's +0xC8). Its callers and when each draws it again: T3b, below.

- **The call** (thiscall on the manager, six arguments): the view's corner as a packed tile (x and
  y signed 10 bits, the level bit 26), `show` (the area shown at once, `screen_show_rect`
  0x602BD0), a third never read, and three reveals that draw a tile of class 53 (mines), 34
  (heroes) or 98 (towns) unexplored (the spells' view of the world: 0 at the callers read). It
  returns at once during a computer's turn outside a network game (`player_is_human` 0x4BADA0 of
  the current player, `remote_turn` 0x691209); the network branch is not read.
- **The area:** the dialog's area 1 (+0x44, its +0x4C: x +0x18, y +0x1A, width +0x1C, height
  +0x1E: (630, 26), 144 × 144), written straight into the back buffer.
- **A tile's colour**, row by row from the top, each row's tiles from the left (the map's width a
  row, its height rows): black (0) when the viewer has not explored it (`viewing_player_bit`
  0x69CCBC in `tile_vision_word` 0x4F79B0's low byte) or while the covered view (0x699538) is
  set; else the terrain's open colour, its DEF's (+0x60 + 4 t) 16-bit palette (+0x20) entry 8
  (+0x2C: the colours start at +0x1C; 0x412F6C), then the first that applies:
  - the tile of the screen's player's selected hero (`game_screen_player` 0x4CE9B0, +0x4; on the view's
    level): game_palette16's entry 64 + the hero's owner (+0x22), as the flags' (§2.9);
  - a hero's tile (class 34 with the entrance bit +0xD 0x10, which `hero_show` sets with the
    class): entry 64 + its owner (game +0x4DF18 by the tile's word, the hero);
  - by `tile_object_class` 0x4FCBD0 (a hero's tile the class under it; the switch 0x412FF0, its
    table 0x413670 indexed by the class less 17): the dwellings 17 and 20, a garrison 33, a
    lighthouse 42 and a mine 53, a shipyard 87 and a town 98, where the tile is blocked or an
    entrance (+0xC bit 0x40 clear or +0xD 0x10 set): entry 64 + the owner of the object recorded
    on the entrance tile `tile_object_entrance` gives (`tile_object_setup` 0x4FCC30, its record's
    owner as `tile_object_owner` 0x40FD20 reads it), 63 with none; the terrain's colour without
    an entrance tile;
  - 21 decorations (116, 118, 119, 121, 123, 126, 128, 131, 133, 134, 135, 137, 148, 149, 152,
    153, 154, 155, 158, 159, 160: cacti, craters, dead trees, frozen and other lakes, hills, lava
    lakes, mandrakes, mounds, mountains, the trees, dunes and pits, stalagmites, stumps, tar pits,
    volcanoes; not the brush, flowers, kelp, logs, moss, rocks or skulls) on a blocked tile (+0xC
    bit 0x40 clear): the terrain's blocked colour, entry 9 (+0x2E);
  - every other class (none, a boat, a monster, a pile, the other decorations): the open colour.

  A tile records a blocking object without an entrance only through 0x505610's two blocking walks
  (§2.5 of `rules/surveys/E08-objects.md`), which clear the bit, so an owned object's tiles all
  take its owner's colour and a listed decoration's its terrain's blocked one (but the hole's,
  124, which the fourth walk records unblocked and the list leaves out). The ten terrains' entries
  8 and 9 are VCMI's `minimapUnblocked` and `minimapBlocked` (`config/terrains.json`) to within
  the 5-6-5 rounding: water's blocked colour is not VCMI's (the original's entry 9, (0, 41, 108),
  never drawn: no listed decoration stands on water).
- **The pixels** by the map's height (`map_height` 0x6783CC; the jump tables 0x4135D0 and
  0x413700 indexed by the height less 36): 36 four by four a tile, 72 two by two, 108 the columns
  2, 1, 1 and the rows 1, 1, 2 over and over (two counters: the column's carried from tile to tile,
  the row's stepped before its row is drawn), 144 one pixel; any other height draws no tile (the
  box still drawn). A row's pointer steps by the rows its tiles took.
- **The box:** `radar.def` (+0xC8), group 0, its frame by the height: 36 frame 4 at the scale 4.0,
  72 frame 3 at 2.0, 108 frame 2 at 1.33 (the f32 0x3FAA3D71), any other frame 1 at 1.0. Its
  place: x = 630 + trunc(vx × s) for the view's corner vx ≥ 0 (`fild`, `fmul`, `fadd` of the
  area's x, `fstp` to an f32, `crt_ftol` 0x6179D4), else 630 with its canvas cut from column
  trunc(−vx × s); y the same. Drawn through `def_draw_button_frame` 0x47BF00 (the plain drawer,
  runs skipped) with the width the DEF's (+0x30: 148) less the columns cut, less what lies past
  the area's right edge (x + 148 always does: 774 − x − the columns cut), the height likewise
  (+0x34: 132; cut at the area's bottom only past it). The frames: 19 × 17, 25 × 23, 38 × 34
  and 76 × 68 at their canvas's corner (frame 1 one pixel a tile: the view's 18.5 × 17 tiles
  rounded up). The x87's roundings of a product with 1.33 could carry it across an integer only
  with its fraction within 2^-14 of the next; none comes within 2^-10 for a corner of −10 to 255
  (a test checks), so the client truncates the exact product of the f32's bits.
- **The view-world dialog** (`view_world_open` 0x6AAC3C, inferred): the frames 5 to 13 by its
  scale 0x68C6B8 (16.0, 11.84, 7.68) and with one of them no box; the spells' view (M4 or later).
- **Built** (*provisional*): `h3_map_view::record`, the object a tile records (0x505610's walks,
  M2 T5's, now shared by the sound and the minimap; the tables' `[map.records]`, moved out of
  `[map.sound]`); `h3_map_view::minimap` with the tables' `[map.minimap]` (the palette entries,
  the owned and blocking classes, each size's columns, rows, frame and scale by the f32's bits),
  a level's `Base` built with its `TileIndex` (each tile's colour by its terrain and record: an
  owned class blocked or an entrance its owner's, a listed class blocked the blocked colour),
  `Base::tiles` each scene (the unexplored tiles black, every hero on the level in its owner's
  colour: the selected hero's rule and a standing hero's give the same) and `Base::radar` (the
  exact product, design above). The scene's draws `Minimap` and `Radar` after the gems; the
  composer fills each tile's rectangle (a terrain's colours from its DEF's palette as the scene
  cycles it, an owner's from the flag palette) and draws the box through the raw drawer cut as
  0x47BF00 cuts it (the cut columns taken once more when the box passes the area's right edge:
  `774 − x − cut`; no frame reaches it). Its cells compare them as draws that stay: the same only
  while the view does not move. An owned object without an entrance on the map: its owner's
  colour (MCL-Q4's answer for its flags), the terrain's under `--bugs original`. Each scene draws
  the minimap as it was last drawn (T3b, below).
- **The kept frame** (T2b2's composer): the minimap's draws stay where they are when the view
  moves (compared unmoved, their last rectangles unmoved), a cell clear of the view's clip keeps
  its pixels through a move, a terrain's palette step counts for the minimap only where a cycle
  rotates its entries (none does: 8 and 9 lie below every range), and its draw's comparison,
  which meets some 25 cells, is made once a scene. Measured (the sandbox, `--measure 20`, `master`
  against T3a): a frame made in a median 0.5 and 0.5 ms on `walk-1`, 1.0 and 1.0 on
  `path-objects`, 1.4 and 1.5 on *Tale of two lands*; uploaded 24.9 → 25.5 % (the minimap at a
  step), 5.2 and 5.2 %, 11.1 → 11.2 %. `Base::tiles` (the explored tiles and the heroes, each
  scene) costs some 36 000 instructions a frame on a 36-tile map, about 0.6 million on a 144-tile
  one: gone with T3b, which makes the tiles only when the original draws them.
- **When it is drawn again** (M3 T3b): 13 call sites in 11 functions, and 20 in 14 more through
  `minimap_redraw` 0x413790 (the manager's corner and its own five arguments). Its second
  argument shows the area at once; `hero_select` and the full update pass 0 and show the whole
  screen after (0x41809B; the full update by its first argument). What M3's sequences meet:
  - **a scroll** that moves the corner (`view_scroll` 0x41973E: Ctrl and a direction key, the
    screen's edges): after the hero is put back on its tile and the corner written, shown at once,
    then the view;
  - **a hero's selection that centres the view** (`hero_select` 0x418046: its c and `view_shown`):
    after the centring, before the view. `mobilize_hero` 0x4175E0 selects only a hero not lifted,
    so a key's step (0x480914, with 0, 0, 1) draws it when it lifts the hero, at a sequence's
    start (the harness's view puts the hero back) or after a scroll: the view centred on the tile
    the hero steps from, the hero on it; the steps after find the hero lifted and draw none there.
    The path's walk lifts the hero with 1, 0, 0 (c clear: no centring, no minimap, 0x407DEC);
  - **each step's end**: after its frames and the end's sounds, every path of `hero_move_step`
    reaches the guard's test (`after_step_guard_check` 0x480090 at 0x48169D), whose first call is
    `minimap_redraw` with 1, 1, 0, 0, 0 (0x4800A5): the view's corner centred on the new tile, the
    hero on it, shown at once. Through a step's frames the box and the hero stay where they were
    (the recordings: `walk-1`'s minimap changes at 300, 600, … and at each of the path's step
    ends, never at a frame).
  - Not met: the full update 0x41753A (a map's start, after a dialog: 21 callers); the step's own
    `minimap_redraw` (0x4810E6) only in a network game (0x481077 jumps past it otherwise);
    `hero_move_loop`'s 0x4086CD only at the end of the manager's mode 7 (its +0x40, not read; the
    walk's end, 0x408730, draws none).
- **The rest** (M4 and later, their calls only, inferred): `minimap_click` 0x40A0C0 (a left click
  on it: the corner the tile clicked less (9, 8), shown at once, 0x40A280, and while the button
  stays down, 0x40A509; a right click its help text: T6); `network_map_redraw` 0x405E30 (a network
  game's); the view world (`view_world_run` 0x5FBF90, opened by V among others: its handlers draw
  it at the dialog's corner with the three reveals); through 0x413790: `hero_relocate` 0x41D930
  (a hero put on another tile at once, 12 callers), `adventure_spell_cast` 0x41D090,
  `visit_object` 0x4A93A0, `visit_and_update` 0x4AAAA0,
  `hero_battle_on_map` 0x4AAFD0, `battle_aftermath` 0x4AC930, `adventure_battle` (three sites),
  `next_player_turn` (two), `turn_start_minimap` 0x4CA530, `hero_screen_show` 0x4E19E5,
  `battle_flag_toggles` 0x4F450D: the engine's moves, visits, battles and turns, and the dialogs.
- **Built** (T3b, *provisional*): the timeline's `Minimap`, its last draw (the view's corner and
  the selected hero's tile): the still's at 0, again at a scroll that moves the view, at a lift
  that centres it (`Loop::lift` with `centred`) and at each step's end, carried in each `Draw`; the live part's
  `minimap` (`minimap::Shown`, the selected hero by its index), which a scene draws the box and
  the selected hero from (none: the scene's view and the heroes where they stand, a still's).
  `Base` makes the level's colours with its explored tiles once, with the index (whatever changes
  the explored tiles builds it again, as the objects); `Tiles` holds them shared (`Arc`, compared
  by their allocation first) with the heroes' tiles over them, so a scene no longer copies the
  level's tiles and the composer's comparison of an unchanged minimap is the heroes' few.
- **Checked** (T3b): the 13 sequences recorded again with the minimap compared (`masks.toml` one
  list, `panels`, for the situations and the sequences): every kept screen of the 459 exact, every
  timeline line as the viewer's; the 40 situations' digests unchanged. **Measured** (the sandbox,
  `--measure 20` side by side with `master`'s build): a frame made in a median 0.3 → 0.2 ms on
  `walk-1`, 0.7 and 0.7 on `path-objects`, 0.9 and 0.9 on *Tale of two lands*; uploaded the same
  (25.7, 5.2, 11.2 %); callgrind on *Tale of two lands* (144 tiles a side): a scene (`scene_in`)
  2.33 → 1.55 million instructions.

### 4.5 The hero and town lists (M3 T4a)

Read (T4a, 2026-10-29): `hero_list_slot` 0x403560, `town_list_slot` 0x4037A0, the bars' frames
0x4E5330 and 0x4E5380, `town_list_frame` 0x5BD700, `lists_selection_frames` 0x4038C0 to its end,
the lists' updates 0x4032E0 and 0x403420 in full, the hero list's scroll 0x403220, the lists'
items in `adventure_dialog_create` (0x401BE0–0x401EB4) and the DEF item's message 0x200/4
(`dialog_def_message` 0x4EA810); checked against the running original (`h3mem.kingdom`, the
town records) and its captures (below). *Provisional* where marked.

- **The items** (in the dialog's order, §4.1): the portraits 15–19 and the selection frames 39–43,
  pictures created without one (flags 0x800) at (617, 212 + 32 i), 48 × 32; the movement bars
  `imobil.def` 20–24 at (610, 213 + 32 i) and the mana bars `imana.def` 25–29 at (666, 213 + 32 i),
  8 × 32, DEF items (flags 0x10, frame 0); after the town list's arrows, `itpa.def` 32–36 at
  (747, 212 + 32 i), 48 × 32. The bars' frames are 6 × 30 (610–615, 666–671), clear of the
  portraits and frames (617–664).
- **A hero's slot i** (`hero_list_slot`, the screen's player): the hero `heroes[top + i]` (the
  player's +0x8, `top` the dialog's +0x64); none (−1) or the view covered (0x699538: the hot seat's
  handover, M5): the picture `hpsxxx.pcx`, both bars frame 0 (sent before their state's bit 2 is
  cleared: a DEF item takes the message only with it). Else the picture `hero_info` 0x679DD0's
  +0x30 (0x5C bytes an entry, 163; the hero's +0x34 its index), the movement bar's frame by the
  points left +0x4D (0x4E5330: 0 at or below 0, points / 100 below 2 300, then 23, 24 from 2 500,
  25 from 2 800) and the mana bar's by the mana +0x18 (0x4E5380: (mana + 4) / 5 below 116, then 23,
  24 from 145, 25 from 170). The frame 39 + i is shown (state 4) when the slot holds the selected
  hero (+0x4), else hidden; with the slot's second argument the three items are drawn again and
  the frame with `hpsyyy.pcx` (0x403743); with its third the rectangle (609, 212 + 32 i, 48 × 32)
  is shown (T4b: it leaves out the portrait's last 8 columns and the mana bar).
- **A town's slot i** (`town_list_slot`): the town `towns[top + i]` (the player's +0x40, `top` the
  dialog's +0x68) when i is below the player's number of towns (+0x3E: the slot, not top + i, is
  tested; the town list's update keeps `top` within the towns when it scrolls, T4b to confirm when
  a town is lost); its frame 0x5BD700 with 1: 2 + 2 × faction (+0x4) with a Fort in the town's
  every-built set (+0x158, building 7; a Citadel or Castle keeps the bit, installation.md §7.27),
  20 + 2 × faction without, one more when it built today (+0x2): 2–37 of `itpa.def`'s 39. Else
  frame 0 (the empty slot's leather).
- **The selections** (`lists_selection_frames`, after every item is drawn, §4.1): the selected
  town (+0x3F) among the slots shown gets frame 1, `itpa.def`'s yellow border with a clear middle,
  and that item alone is drawn again: the mark over the picture just drawn; the frames 39–43 are
  hidden, then the selected hero's among the slots shown gets `hpsyyy.pcx`, is shown and drawn.
  The town's item keeps frame 1: drawn alone again later it would put the mark over whatever lies
  under it (a redraw artifact, not copied; T4b reads when it happens). Selecting a hero
  (`hero_select` 0x417B20) or a town (`town_select` 0x417830, which sets the selected hero to −1)
  updates both lists with their redraw; the arrows scroll by one (0x403220 for the heroes) and
  redraw.
- **The lists' order** (the running original): the map's order of the player's heroes and towns at
  a new game; a town record's place (+0x5, +0x6, +0x7) is its entrance, two columns left of its
  anchor.
- **VCMI 1.7.5 compared** (`client/adventureMap/CList.cpp`): the same items and places
  (`CHeroItem`, `CTownItem`: the portrait one right of the movement bar, the mana bar one right of
  the portrait), the selection the same pictures (`HPSYYY`, `ITPA` frame 1 over the picture), the
  town's frame its faction's icon + 2 by fort and built; the bars differ: `min(25, points / 100)`
  and `min(25, mana / 5)` (the original's last three frames span 2 300–2 499, 2 500–2 799 and
  2 800 on, its mana rounded up below 116: any mana shows).
- **Built** (T4a, *provisional*): `h3_map_view::lists` (the tables' `[map.lists]`: the portraits,
  the bars' rules, the towns' frames; each list's slots from its top; the selection's slot);
  `panels::dialog` gives the slots' pictures (`h3_ui::ui::Shown::pictures`, new) and frames and
  hides 39–43, `panels::selections` the mark or the frame drawn after the items (before the
  resource bar: disjoint); the live part's kingdom gains each hero's portrait, points and mana and
  each town's faction, fort and day (`[asleep, movable, portrait, points, mana]`, `[faction, fort,
  built]`; the form before, which the sequences' kept live parts hold until T4b, still read); a
  map's start gives the map's portraits and towns, no points or mana (the engine's from M4); the
  layout's items 15–29, 32–36, 39–43; the gate checks the bars' and towns' frames and every
  portrait (165 pictures, 48 × 32). The harness writes a situation's heroes' `points` and `mana`
  and towns' `built`, then selects its `select` from its list (both lists redrawn).
- **Checked** (T4a): three new situations, `lists-heroes`, `lists-scrolled`, `lists-towns` (the
  bars' every edge, both lists scrolled, a town's mark, empty slots, five factions with and
  without a fort, built today), exact first time; the set recaptured with the lists compared
  (`masks.toml`'s `stills`; the sequences keep `panels` until T4b).
- **When they are drawn again** (T4b, 2026-10-29; read: `hero_list_slot`'s and
  `hero_list_update`'s 34 call sites listed, `town_list_update` 0x403420 in full, the sites below
  read; `hero_move_step` 0x4805E0 around its points): every call of the slot is (−1, 1, 1): the
  selected hero's slot, when the list shows it (none else), drawn and its rectangle (609, 212 +
  32 i, 48 × 32) shown at once. Both updates with their redraw draw every item of the dialog
  (vtable +0x14 over −65535 to 65535), the town list's then the selected town's mark alone
  (frame 1, 0x403509) as the full update's selection does; with their show the whole screen is
  shown. What M3's sequences meet:
  - **a step's end** (`hero_move_loop` 0x407EEF, `adventure_key_down` 0x409069, after
    `hero_move_step` returns: after the step's end's sounds and the guard's minimap): the slot.
    The step's cost is priced at its start (0x4807DA) and taken after its frames (0x48118C), the
    points dropping to 0 when fewer than the cheapest step from the new tile remain (0x481191);
    a step not paid sets them to 0 at its start (0x48087A) and is not taken, the slot drawn all
    the same. So the movement bar changes at each step's end, never during its frames;
  - **a lift that centres the view** (`hero_select` with c, a key's step lifting a hero not
    lifted, 0x480914 with 0, 0, 1; §4.4): `hero_list_update` with the hero's place, c and 0
    (0x417F09: the list scrolled to show it, 0x403318: kept while it shows, else the place held
    within the last five) and `town_list_update` with −1, c, 0 (0x417F19), shown with the whole
    screen (0x41809B), before the step: the points the last step left (the harness's write of a
    sequence's `points` shows first here). The path's lift (1, 0, 0) draws neither;
  - **a walk's end that leaves the hero unable to move** (`hero_can_move` 0x4E5F30 clear; the
    path's 0x408193, a key's 0x40913E): `make_hero_path` with 1, 0, 0, then `hero_list_update`
    with −1, 1, 1 (0x4081B9, 0x409167): every slot, the whole screen shown (`walk-points`).
  - Not met: `path_rebuild` 0x40EC90's update (0x40F1D5, the hero unable to move; from a key of
    `adventure_key_down` other than the directions, 0x409318, and two more callers), the lists'
    arrows (`hero_list_scroll` 0x403220, `town_list_scroll` 0x403280: T6's clicks), the full
    update and the turn's start, a boat boarded or left (0x4A10A8, 0x49E6E0: the slot after the
    points set to 0), a spell (`adventure_spell_cast` 0x41D2D5, `hero_relocate_spell` 0x41D8B4,
    `hero_spend_mana` 0x4D931E: the slot after the mana is spent), visits, battles and the hero
    screen: M4 and later.
- **The slot's rectangle** holds the movement bar (610–615) and the portrait's and the frame's
  first 40 columns (617–656); the mana bar (666–671) and their last 8 columns lie outside, drawn
  but not shown until something shows the whole screen. In M3 the slot's hero, portrait and mana
  never change between its draws (a walk changes the points alone); after a spell's cost (M4) the
  screen would keep the old mana bar until the next update or lift (MCL-Q14).
- **The town's item left at frame 1** (T4a): drawn again by every redraw of all the items while
  its town is selected, it puts the mark over the picture and mark the screen holds there:
  nothing changes. Every town list update sets the slots' frames again before its redraw
  (`town_list_slot`, then the mark), so no picture is lost under a stale mark; M3's sequences
  have no towns.
- **Built** (T4b, *provisional*): the timeline's `Draw` carries the kingdom as the panels last
  showed it (`Arc<Kingdom>`; the live part's at 0); a step's end shows the selected hero's points
  in it (its slot, when shown), a lift that centres the view and a walk's end that leaves the hero
  unable to move the whole kingdom (the hero list scrolled to the hero at the lift,
  `lists::top_showing`); the loop's kingdom in memory takes the selected hero's points from
  `Start::points` at 0 and at each step's end (M3's walks scripted: a recording's `points` lines,
  `Sequence::points`; the engine's from M4); the timeline's `points` lines (memory's). The
  harness records `points` each millisecond (`Mem.points`) and keeps the live part's kingdom as
  read before it writes a sequence's `points` (the list as shown at 0); `masks.toml` one list
  again (`panels`); the kingdom's form before T4a no longer read.
- **Checked** (T4b): the 13 sequences recorded again with the lists compared: every kept screen
  of the 459 exact, every timeline line as the viewer's; the 43 situations' digests unchanged.
  A bar that changes is a dialog draw, which rebuilds the panels' background and uploads the
  whole screen (`walk-1`: 31.0 % of the pixels uploaded, `master` 25.7 %; T4c).

### 4.6 A changed panel item drawn again alone (M3 T4c)

No reading; every frame as before, pixel for pixel (*provisional* in what follows). Found by
T4b's measurement (§4.5, "Checked"): a scene whose panels' draws (the background: the frame's
picture, the dialog's items, the lists' selections, the resource bar) differ from the last's in
some items drew the background again whole, then the whole view, and uploaded the whole screen,
as a button's state already did at a walk's start and end (§4.3, "Left").

- **An item's reach** (`h3_ui::paint::reach`, the drawers' own): the rectangle its drawer can
  change: a picture's at most its item's size of it, a button's or a DEF item's its frame's canvas
  (frame 0 past the group's, as drawn), a darkened rectangle itself; unknown for a text and a
  label (the text drawer's lines may pass the item's box, `h3_render::text`: not read; known
  since T5a, §4.7) and for an asset the batch lacks.
- **The areas** (`compose::changed_areas`): with as many background draws as the last scene's,
  each pair that differs gives its old and its new item's reach; with more or fewer, every item
  between the two lists' common start and end. Each is cut to the screen, overlapping ones joined.
  Everything is drawn again as before when a changed item's reach is unknown (a frame's picture,
  a text, a missing asset), when an area meets the view's clip or the background's one-colour clip
  (the pixels a scroll moves, §4.3), when a frame's picture or a dialog's item lies among the
  draws over the background (no cut reaches them), or when the bugs changed.
- **The background in the areas** (`Background::redraw`): on a scratch screen kept by the
  composer, black in each area, the frame's picture cut to each, every dialog item whose reach
  meets one (or is unknown) drawn whole (the dialog's drawers have no cut; what they draw
  elsewhere on the scratch is never copied), then each area copied into the background. Exact by
  construction: an area's pixels are the background's draws in their order, and outside the areas
  neither the old nor the new items reach.
- **The screen over it:** the view's cells that meet an area are dirty whatever their draws
  (restored from the new background, every draw meeting them drawn again cut to them, §4.3); a
  scene that draws its other draws whole (another flag palette) restores the areas with what the
  last scene drew. The damage is those cells: for `walk-1`'s movement bar or a button's state the
  few cells they meet, not the screen.
- **Checked:** the window under Xvfb, `--check 10` on `walk-1`, `walk-points`, `lists-scrolled`
  and *Tale of two lands*: every frame read back exact, made as the same frame composed afresh,
  every change the headless loop's; `drawn` the same (43 situations, 13 sequences).
- **Measured** (the sandbox, `--measure 20`, alternating with `master`'s build twice each):
  `walk-1` 30.8–31.0 → 24.6–24.8 % of the pixels uploaded (25.7 before T4b), a frame made in a
  median 0.8 ms (both), p95 3.6–4.0 → 2.9–3.0; *Tale of two lands* (11.2 %) and `path-objects`
  (5.2 %) as before, their times within the runs' spread (medians 1.8–2.0 and 1.3–1.5 ms).
- **Left:** a changed text or label draws everything again (T5's status window, resource bar
  and date, T6's hints: its reach needs the text drawer's extent, read where T5 needs it; done in
  T5a, §4.7); a dialog item drawn on the scratch is drawn whole however little of it an area
  needs.

### 4.7 The resource bar's amounts and the date (M3 T5a)

Read: `resource_bar_create` 0x558BA0's texts, `resource_bar_redraw` 0x558F20 in full, the
calls of `resource_bar_update` 0x403F00 (16 sites; `hero_select`'s at 0x417F48), `shown_player`
0x69778C's writes.

- **The items** (0x558CE5's loop, then 0x558DB2; texts alone, `text_item_create` 0x5BC280): the
  amounts 1001 to 1007 at (37 + 84 i, 3) of the bar, 50 × 18, in `smalfont.fnt`, colour 4,
  alignment 0 (left, top), background 0, flags 8, each followed by its hint's area 1009 + i (T6);
  then the date 1008 at (607, 3), 180 × 18, the same but centred (alignment 1). The bar's picture
  `aresbar.pcx` is its first item, drawn under them.
- **Their texts** (0x558F20, `this, draw, show`): the picture in the colours of `shown_player`
  0x69778C (the current player at a human's turn, `next_player_turn` 0x4C7730; a saved game's
  load 0x4BDBF7), that player's record (game +0x20AD0 + 0x168 × colour) +0x9C read as seven
  `i32` in the order of the table 0x641008 (0 to 6: wood, mercury, ore, sulfur, crystal, gems,
  gold), each written by `%d` (0x660A1C) into its item; the date by the executable's format
  0x682E64 (three words, each with its number) from `general_text`'s rows 63, 64, 65 (the
  month's, the week's and the day's words) and the game's month +0x1F642, week +0x1F640 and day
  of the week +0x1F63E (`u16` each, from 1); with `draw` every item of the bar drawn (0x5AA4F0,
  the ids −65535 to 65535: the picture under the texts), shown with `show`. A full update and
  `hero_select` (0x417F48: draw, not show; the screen is shown later) call it through 0x403F00; a
  town's selection does not, so a write into memory shows after a hero's selection.
- **The catalogue** (`clients/text/keys.toml`, common): `map.resources.amount` (`EXE:0x660A1C`,
  the argument `amount`), `map.resources.date` (`EXE:0x682E64`; `month_word`, `month`,
  `week_word`, `week`, `day_word`, `day`), `word.month`, `word.week`, `word.day` (GENRLTXT 63 to
  65). The viewer's binary reads the original's forms of these keys alone
  (`h3_map_view::panels::KEYS`, its `texts.rs`: the catalogue's families, the battle's, left out)
  from the archives as the game searches a name and from `Heroes3.exe`; a form it cannot read
  stops it.
- **A text's reach** (`h3_ui::paint::text_reach`, closing §4.6's "Left"): the line drawer
  (0x4B5260) draws a line only inside the box's height, skips the glyphs that would start left of
  it and stops before the first whose advance passes its right edge, so a glyph's pixels leave the
  box only by a negative right space (past its advance) or, after the first, a negative left one:
  the reach is the box, its columns widened by the font's widest of each (`smalfont.fnt`: 1 and 1).
  A changed amount draws its box's background again and the cells it meets, not the screen.
- **Built** (*provisional*): the live part's `kingdom` has `resources` (seven, the original's
  order) and `date` (`[month, week, day]`), from `h3mem.py`'s `Mem.kingdom`; a map's start none
  and the first day (the difficulty's starting resources are the engine's rule, M4);
  `panels::resource_bar` fills the texts from the tables' catalogue (`Tables::texts`, empty from
  the committed tables: a key shows as itself); the layout's eight labels; a situation's
  `resources` and `date`, written by the harness before its selection (`Mem.write_resources`,
  `write_date`), which must be a hero's; the situation `resources-date`; `masks.toml`'s `panels`
  without the bar's eight rectangles.
- **Seen** (MCL-Q15): an amount of eight digits (12 345 678 gems) is cut to the seven that fit its
  box (`1234567`): the text drawer's last line cut at the glyph that fits (`h3_render::text`), the
  viewer's the same.
- **Checked:** `set`: the 44 situations exact with the bar (the new `resources-date` first time);
  `frames`: the 13 sequences, every frame exact, their timelines as before but `scroll-limits`'
  corner box (it now reaches the bar) and `scroll-0`'s start points (the start's random army,
  `TRAPS.md`); `drawn` the same after `--record`; the window under Xvfb (`walk-1`, `walk-points`,
  `lists-scrolled`, `resources-date`, *Tale of two lands*, `--check 10`): every frame read back
  exact, made as the same frame composed afresh, every change the headless loop's.
- **Measured** (the sandbox, `--measure 20`, alternating with `master`'s build): the same within
  the runs' spread: `walk-1` 24.6–24.8 % uploaded both, a frame made in a median 0.4 (0.5 on
  `master`); *Tale of two lands* 11.2 %, 1.2 (1.3); `path-objects` 5.2 %, 0.9 both. No sequence
  changes an amount or the date (M4's turns and pick-ups will).

### 4.8 The status window of a hero and a town (M3 T5b1)

Read: `status_window_choose` 0x415DE0 in full, its kinds' builders' callers 0x415F20, 0x415FB0,
0x416160, 0x4162F0, 0x4163A0, 0x416450 and 0x4164F0, the kind's setter 0x415D60, the hero's window
0x451AB0 and the town's 0x4521F0 in full, `town_garrison_army` 0x5C1460, `town_list_frame`
0x5BD700 (T4a's), the arguments at the chooser's call sites in M3's paths. The other kinds'
builders only by their pictures (T5b2 reads the date's and the kingdom's; the user, 2026-10-29: the
other player's turn and the messages are M4's).

- **The kinds** (the manager's +0x398 the kind asked, +0x394 the kind shown, +0x39C a timed kind's
  end): 0 the selection's; 1 the date's (`AdStatOt.pcx`, `NewDay.def`, `newday.wav`: 0x450DD0); 2
  the kingdom's (`AdStatin.pcx`, `itmtl.def`: 0x452B80); 3 a hero's (`AdStatHr.pcx`: 0x451AB0); 4 a
  town's (`AdStatCs.pcx`: 0x4521F0); 5 another player's turn (`AdStatNx.pcx`, `crest58.def`,
  `HourSand.def`, `HourGlas.def`: 0x453250); 6 a resource found (`AdStatOt.pcx`, `resour82.def`:
  0x451220, the manager's +0x3A0 to +0x3A8); 7 a text (`AdStatOt.pcx`: 0x451820); 8 none. 0x415D60
  sets a kind and its end: the time given, or without one (negative) 3 000 ms for 1 to 4 and
  5 000 for 6 (5 and 7 then end at once); the chooser turns a kind whose end has passed
  (`clock_ms` − +0x39C ≥ 0) back to 0, but 8. Kind 0 at a human's turn
  with the view not covered (`covered_view` 0x699538): the selected hero's window (the current
  player's +0x4 ≠ −1), else the selected town's (+0x3F ≠ −1), else the date's builder; otherwise
  kind 5's.
- **A builder** (`this, force`): the same kind shown and no `force` builds nothing (returns 0);
  else +0x394 set, the panel made (0x34 to 0x74 bytes) and given to the dialog (0x402C10, not
  read further); the date's and the kingdom's also redraw the resource bar
  (0x403F00 with 1, 1). Then the chooser (`this, force, draw, show`) with a panel built and `draw`
  draws the dialog's items again (0x402BC0, `show` its rectangle). The calls in M3's paths:
  `adventure_full_update` 0x4174BC (1, 0, 0), `hero_select` 0x41800B (1, 1, 0), every view redraw
  0x40F861 (0, 1, 1: only a timed kind's end builds), `path_rebuild` 0x40F173 and a key's step
  0x4090FE (1, 1, 1: the same window built again, the same pixels). The status window's click
  (the dialog's handler 0x409A70 at 0x409FD8, items 2000 to 2200): kind 2 → 1; another kind but 0
  → 0; 0: the kingdom's (2), or the date's (1) when the kingdom's shows; the end now + 3 000; then
  (1, 1, 1). T5b2 and T6 build those.
- **The panel** (0x5AA470): at (614, 400), 176 × 166, inside the dialog's area 2; its items added
  to the adventure dialog's list (`AddItem` −1, 0x5AA4A0), ids from 2000 in the order made; none
  overlaps another, nor another panel's.
- **A hero's** (0x451AB0; the current player's selected hero): 2000 the picture (0, 0); 2001 the
  large portrait (`hero_info` 0x679DD0 + 0x5C × the record's portrait +0x34, its +0x34) at (3, 2),
  58 × 64; 2002 the name (+0x23) in `smalfont.fnt`, colour 4, left (alignment 0), at (66, 2),
  107 × 17; 2003 to 2006 the primary skills (+0x476 to +0x479, i8): above 99 shown 99, above 0
  itself, else 0 for attack and defense and 1 for power and knowledge (0x451CFC: the places'
  table pointer from 0x660BC0), by `%d` at the table 0x660BB0's places (65, 51), (92, 51),
  (120, 51), (148, 51), 23 × 16, centred; 2010 the spell points (+0x18, i16) by `%d` in `tiny.fnt`
  at (145, 93), 27 × 14, centred; 2008 the morale (`imrl22.def`, frame `H3Hero::GetMoraleBonus`
  0x4E39B0 + 3) at (5, 74), 22 × 12; 2009 the luck (`ilck22.def`, `H3Hero::GetLuckBonus` 0x4E36C0
  + 3) at (5, 91); then, when a slot is filled, each slot in turn (types +0x91, counts +0xAD) at
  the table 0x660BD0's place (36, 73), (72, 73), (108, 73), (18, 121), (54, 121), (90, 121),
  (126, 121), every slot stepping the place (0x4520E9): a filled one's `cprsmall.def` frame (the
  creature + 2), 32 × 32, and below it (+34) its count in `tiny.fnt`, 32 × 13, centred: below
  10 000 by `%d`, else the thousands (0x10624DD3: truncated) by `%dk` (0x660CB0).
- **A town's** (0x4521F0; the current player's selected town, game +0x21614 + 0x168 × +0x3F):
  2000 the picture; 2001 `itpt.def` at (3, 2), 58 × 64, its frame `town_list_frame` with 0 (the
  town list's frame less its 2); 2002 the name (char* +0xC8) at (66, 1), 107 × 16, as the hero's;
  the hall from the buildings standing (u64 +0x150, by the game's ids: an upgrade clears what it
  replaces, unlike the every-built set +0x158): the Town Hall (11) 1, the City Hall (12) 2, the
  Capitol (13) 3, tested in that order, else 0 (its hint's text 0x6A6294 + 4 × it, T6's), 2003
  `itmtls.def` at (67, 31), 34 × 34; the fort: the Fort (7) 0, the Citadel (8) 1, the Castle (9) 2,
  else 3, 2004 `itmcls.def` at (105, 31); with a garrison hero (+0xC ≠ −1) 2008 `townqkgh.pcx` at
  (149, 76), 22 × 30; with a Resource Silo (15, the every-built set) the resources its income
  holds (`H3Town::GetResourceSiloIncome` 0x5C1280, seven i32: the non-zero ones from wood): two,
  2007 `smalres.def` at (6, 75) and (6, 87), 20 × 18, frames the resources; one, at (6, 81); more
  or none, nothing; 2006 the gold income (`H3Town::GoldIncome` 0x5BF600 with 1, its low 16 bits
  sign-extended: 0x45279F, MCL-Q16) by `%d` in `tiny.fnt` at (144, 54), 28 × 12, centred; then the
  army shown (0x5C1460: the garrison hero's +0x91 with one, else the town's own +0xE0), when it
  has a stack (`H3Army::GetStackCount` 0x44ACC0), each filled slot at the next of the table
  0x660C08's places (the same as 0x660BD0's), the place stepping with a filled slot only
  (0x452A38: the slots packed, unlike a hero's), its picture and count as a hero's (the count
  written by a stream: the number, or the thousands and `k` 0x660CD0, the same text).
- **Seen** (the captures): the hero's and the town's windows as read, pixel for pixel; every
  situation and sequence shows one or the other at its moment (the selection's: memory's +0x394,
  3 or 4).
- **Built** (*provisional*): `h3_map_view::status` (the panels `status_hero` and `status_town` of
  `adventure.toml`, h3_ui::ui's format, our ids: the original's for the fixed items, the army's
  pictures 2011 to 2017 and counts 2021 to 2027 by place, the silo's 2007, 2018 and 2019 by place;
  the tables' `[map.status]`: the 163 large portraits, the frames' offsets, the skills' bounds,
  the thousands, the halls and forts tested), drawn after the resource bar; the live part's
  `kingdom.status` (`Status`: a hero's type, whether its name is its own, skills, morale, luck and
  army; a town's name as a record of `TownName.txt`, buildings standing, garrison hero, army
  shown, silo and income; another kind not drawn), read by the harness from memory (`Mem.status`:
  the records and the window's own items for the bonuses, the silo and the income, the original's
  rules computed) and at a map's start the selection's with what the map gives; the names by the
  catalogue (`hero.<type>` over `HOTRAITS.TXT`, `town-name.<record>` over `TownName.txt`, an
  `optional` family of `h3_ui::text::keys`; the numbers by `map.status.number` and
  `map.status.thousands`), a hero's own name the map's (`MapArt::hero_names`: a hero object's,
  else a disposed hero's; `h3_ui::text::decode_1252`); the gate checks the window's DEFs, pictures
  and large portraits. A situation writes a hero's `skills`, `morale`, `luck` and `army` and a
  town's `garrison` and `garrison_hero`, and gives a town a `silo`; the situations `status-hero`,
  `status-town` and `status-garrison`; `masks.toml`'s `panels` without the status window; the
  harness refuses a still or a sequence whose window is another kind, or a town's name
  `TownName.txt` lacks (a map's own: M4's).
- **Checked:** `set`: the 47 situations exact with the window (the three new ones too); `frames`:
  the 13 sequences, every frame exact; `drawn` the same after `--record`; the window under Xvfb
  (`status-hero`, `status-town`, `status-garrison`, `walk-1`, `lists-scrolled`, `--check 10`):
  every frame read back exact, made as the same frame composed afresh, every change the headless
  loop's.
- **Measured** (the sandbox, `--measure 20`, alternating with `master`'s build): the same:
  `walk-1` 24.8 % uploaded, a frame made in a median 0.4–0.5 ms; *Tale of two lands* 11.2 %, 1.2;
  `path-objects` 5.2 %, 0.9–1.0.

### 4.9 The date's and the kingdom's status windows, and when each shows (M3 T5b2)

Read: `status_date_create` 0x450DD0 and `status_kingdom_create` 0x452B80 in full, the date's
panel's step `status_date_update` 0x4511A0 and its vtable 0x63BB0C, `status_panel_update`
0x402B90 and its callers, `status_window_date` 0x415FB0 again, the item message 0x5FE4F0's subtype
5 (0x5FE599 to 0x5FE5E4), every caller of `adventure_view_draw` 0x40F3F0 for its fifth argument
(1 at each), `hero_select`'s chooser call (0x417FEF to 0x41800B), a key's step's (0x4090F6). The
other kinds (another player's turn, a resource found, a text) are M4's (the user, T5b1's cut).

- **The date's** (0x450DD0; kind 1): the panel at (614, 400), 176 × 166; 2000 `AdStatOt.pcx`
  (0, 0); on a week's first day but the game's first (0x450EB4: day 1, not week 1 of month 1)
  the week's picture (the table 0x660BA0 by the week of the month: `NewWeek1.def` to
  `NewWeek4.def`, 4, 8, 12 and 16 frames) and "%s %d" (0x660C98) with GENRLTXT 64 ("Week") and
  the week, no sound; else `NewDay.def` (10 frames) and GENRLTXT 65 ("Day") with the day of the
  week, then `newday.wav` (`play_sound_async`, time-out 30 000, category 3). 2001 the picture at
  (0, 0), 175 × 166 (a column narrower than the panel), frame 0; 2002 the text in `medfont.fnt`,
  colour 4, alignment 1, at (10, 10), 148 × 146. The panel keeps the picture (+0x38), the text
  (+0x34), the frame (+0x3C, 0), its step (+0x40, 100 ms) and its last step's time (+0x44, the
  clock at the build). Every picture's frame is 175 × 166, opaque but 62 transparent pixels at
  the same places in each (so a frame drawn over the last shows the panel's picture there as
  over the picture itself).
- **Its step** (0x4511A0, the vtable's +4; `status_panel_update` 0x402B90 calls it when its
  argument is 0 or the dialog's +0x6C is set; the other panels' +4 is a lone `ret` 0x5BC690):
  while the frame is not the picture's group 0's last and the clock is 100 ms or more past +0x44,
  the frame + 1, the picture drawn and the text over it, the picture's rectangle shown at once
  (0x5FE7C0 with 5, 0x4000: the item message's subtype 5 adds the state and, with 0x4000, shows
  the item's rectangle and clears it), +0x44 the clock now. `status_window_date` 0x415FB0 calls
  it (with 0) when the date's shows and it is not forced; three sites in the computer's turns
  (M4's) and a message box (with 1) too.
- **The kingdom's** (0x452B80; kind 2): 2000 `AdStatin.pcx`; the current player's towns counted
  by their every-built sets (+0x158; `building_bits` 13, 12, 11: the Capitol, the City Hall, the
  Town Hall, tested in that order, else a village; the town's window tests the standing set
  +0x150, the same in play); for each hall i of four `itmtl.def`'s frame i at (6 + 42 i, 11),
  38 × 38 (ids 2001 to 2004), and under it, when it has towns, the count by `%d` in
  `smalfont.fnt`, colour 4, centred, at (7 + 42 i, 56), 37 × 17 (id −1); "%s:" (0x660D28) with
  GENRLTXT 391 ("Allies") at (10, 103) and 392 ("Enemies") at (10, 134), 57 × 20, left (id −1);
  then by colour each player not out of the game (game +0x1F636) of the shown player's team
  (+0x1F879, the shown player 0x69778C among them) on the first row, `itgflags.def`'s frame the
  colour at (67 + 15 k, 102), 15 × 20, the others on the second at (67 + 15 k, 133), each row
  stepping by its own flags, ids from 2005 in the order made.
- **When each shows** (§4.8's chooser, read again): every view's draw ends with the chooser (0,
  1, 1: each caller of 0x40F3F0 passes its fifth argument 1, so the idle redraws, a scroll's, a
  walk's frames and the rest), so a kind asked for a time (+0x398 and +0x39C,
  `status_window_set_kind`: a new day asks the date's for 3 000 ms) shows until the first draw at
  or after its end and then the selection's (kind 0: the selected hero's, else the selected
  town's, else the date's); the date's, asked or the selection's, steps its frame at each draw
  100 ms or more after its last step, up to its last (at the idle redraws, every 180 ms). Forced:
  `hero_select` with its c (0x417FF6: a lift that centres the view) and a key's step's end
  (0x4090FE, after its visit and the mouse's move) build the window again: the date's at its
  first frame, with its sound. The status window's click (0x409FD8, T6) cycles the kingdom's, the
  date's and the selection's.
- **Seen** (the captures): the date's (the day's at frame 1, the third week's) and the kingdom's
  (four towns by their halls, two teams written) as read, pixel for pixel, first time; the kind
  asked at a sequence's 0 shows from 0's redraw; the kingdom's gives way to the hero's at the
  redraw at 720 (asked for 700 ms); the date's steps at 180, 360 … 1 620 to its tenth frame,
  each step's screen change the picture's rectangle, and gives way to the hero's at 3 060; the
  colours out of the game (no player) show no flag.
- **Built** (*provisional*): the live part's `Status::Date` (the frame) and `Status::Kingdom`
  (the halls' counts, the allies' and the enemies' colours), `Status::kind`, the kingdom's
  `selection` (the selection's window, read before a kind is asked: what the end builds again);
  `status::window`'s date's and kingdom's (the layout's `status_date`, `status_kingdom`: our ids
  2021 to 2024 for the counts, 2031 and 2032 for the labels, 2041 and 2051 for the two rows'
  eight places), the tables' `[map.status.date]` (the day's and the weeks' pictures, their frames,
  the step, the sound; the gate checks each picture's frames exactly), `DateArt::shows`; the
  catalogue's keys `map.status.date`, `map.status.side`, `word.allies`, `word.enemies`;
  `timeline::Loop`'s chooser at the end of each draw and forced at a centring lift and a key's
  step's end (`Start::asked`: a recording's kind asked at 0 and its end; none: a date's or
  kingdom's window at 0 asked until no end), the date's step and build (`Cue::Effect` for its
  sound, played once on a stream of the window's own; `SoundRules::effects`); a status window
  the live part does not give (none) is left as it is. A map's start without a selection gives
  the date's at its first frame. The harness: `Mem.status` for the kinds 1 and 2 (the frame from
  the picture's item; the halls, allies and enemies computed from memory), `Mem.ask_status`
  (+0x398, +0x39C, with `rebuild` +0x394 none), `h3map.ask_status` (the selection's window read
  first); a situation's `status` (1 or 2, asked until long after the still) and `teams` (written
  into +0x1F879), a sequence's `status = [kind, end]` (asked at 0, built afresh by 0's redraw; a
  date's window at 0 the sequence did not ask is refused: its steps' times are unknown), the
  situations `status-date`, `status-week`, `status-kingdom` (`TownBuilt`'s `townhall` and
  `cityhall`, additive), the sequences `status-kingdom-end`, `status-date-end`. Not built: the
  date's window over a computer's walk (M4's), a message box's (+0x6C), the click (T6).
- **Checked:** `set --only` on the three new situations: exact (0 of 469 344 differing);
  `frames --only` on the two new sequences: every frame exact (6 and 19), the viewer's timeline
  the recorded one line for line; `drawn` all the 47 situations and 13 sequences recorded before
  the same, then `--record`; unit tests (the windows, what the date shows, the chooser over time:
  the date's steps and last frame, the ends, a town's selection, a selection's window unknown,
  nothing selected, a still's date stepping on, a lift's and a key's step's rebuild, the sound
  when built and not for a week or a later frame), 100 % coverage; the window under Xvfb
  (`status-date-end`, `status-kingdom-end`, `status-kingdom`, `status-week`, `--check 10`): every
  frame read back exact, made as the same frame composed afresh, every change the headless
  loop's.
- **Measured** (the sandbox, `--measure 20`, alternating with `master`'s build twice each): the
  same: `walk-1` 24.8 % uploaded, a frame made in a median 0.5 ms (0.4–0.6 on `master`); *Tale of
  two lands* 11.2 %, 1.2–1.3 (1.2). The chooser at each draw compares a status or returns.

### 4.10 The mouse over the map and the panels (M3 T6a1)

Read: `adventure_mouse_move` 0x40E360 whole (its human turn's part from 0x40E392), `view_mouse_tile`
0x40DEB0 (another player's turn: the tile, its hint and three cursors; M4's), `tile_cursor_kind`
0x40E280, `town_defended` 0x5BDFE0, `map_hint` 0x40B150's frame (0x40B150 to 0x40B215 and its tail
0x40D0B0 to 0x40D225) and its cases 0 (the classes 0, 3, 26, 36), 17 (34), 29 (54), 46 (98) and the
default, `hint_tile` 0x40AFB0, `adventure_panel_hint` 0x403010, `panel_hint_index` 0x402C50,
`dialog_item_at` 0x5FF3E0, the loads of the texts (`help_text_load` 0x5B98B0's groups,
`arraytxt_load` 0x5B9CC0's, `objnames_load` 0x5B92D0) and `pathfinder_run` 0x56B440's stop. Built in
`h3_map_view::hover` and the timeline's `Loop::moved` (*provisional*); recorded and compared:
`hover-panels`, `hover-map`, `hover-sail` (every frame exact, each timeline the recorded one; their
status bar compared, `masks.toml`'s `hinted`).

- **The tile under the mouse** (0x40E3D2): the view's corner plus the mouse message's point divided
  by 32, x and y: the point is the screen's, so a drawn row's last 8 pixels give the tile below
  (§1.2; observed: (336, 196) over the drawn row 5 is row 6; MCL-Q18). The item's bounds compare x
  with its x and with its y plus its width (both 8: no effect). The tile is looked at only when the
  view's column or row under the mouse changes (+0xEC, +0xF0; set to -1 after a walk's end, 0x40812F,
  and a key's step, 0x4090F1): a scroll under a still pointer looks at nothing, nor does a return to
  the column and row last looked at after the pointer left the view (MCL-Q17).
- **On a change** (0x40E41A on): the click's action +0x40 to -1; off the map or on a tile the
  viewer has not explored the arrow and nothing else (the hint left); else the tile's hint
  (0x40E521), then the cursor (type 1, `cradvntr.def`). Without a hero selected (0x40E570): a town
  of the player's team (by its entrance's setup, `tile_object_entrance` 0x4FCAA0 and
  `tile_object_setup` 0x4FCC30; or the manager's +0x3D, a debug flag, not modelled) frame 3 and
  action 3, a hero of the player's own (its +0x22 the current player, not the team) frame 2 and
  action 2, a shipyard of the team (its setup's low byte the owner) frame 6 and action 8; else the
  arrow. With one: another level the arrow; its own tile frame 2, action 2; a blocked tile (+0xD
  bit 0x01, 0x40E730): a town's of the team frame 3, action 5, a shipyard's frame 6, action 8, else
  the arrow (the pathfinder's path list emptied, 0x40E836: nothing drawn changes); from land a
  water tile only with a hero, a boat or a shipwreck (85) on its entrance, from a boat only water or
  the coast (class 3), else the arrow (0x40E862). Then (0x40E91B) the pathfinder run for the
  selected hero to the tile (`movement_calculations_mouse` 0x4194A0 → `pathfinder_run` 0x56B440,
  with the hero's points now), and the tile's node (+0x24 at ((y + 2 z h) w + x), 30 bytes): not
  reached the arrow; else the action 1 and the days d = 0 within the points (+0x4D), else
  (cost − points − 1) / day points (+0x49) + 1, at most 3 (0x40E98F). The kind by the tile's class
  (switch 0x40E9DE, index table 0x40EC18): the coast from a boat 7 (a landing), a boat 6 (from a
  boat 0 and the action -1), a garrison's or town's entrance not the team's with an army (its
  record +0x4 / `town_defended`: a garrisoned hero, else a visiting hero's army, else its own) 5, a
  hero of the team 8, another hero or a monster 5; else `tile_cursor_kind`: a tile a monster guards
  (the vision word's bit 8) 5 unless an entrance of a class never entered (the object settings'
  first byte, `object_classes_not_entered` 0x63AA58); from a boat a water entrance 0x1C, else 6;
  on land an entrance 9 (on water only a shipwreck's), else 4. The frame: kind + 6 d, but 0x1C
  from a boat 28 + d. Seen: 0 2 3 4 5 6 7 9 10 11 12 13 14 15 16 21 in the three sequences.
- **The pathfinder stops** (0x56BC7E): after each expansion, once the destination's node is
  reached (its +4 bit 0 set when found), so its cost is the one it was first found at, not always
  the cheapest (MCL-Q19). The harness therefore reads each tile's cost after a run to it: before a
  sequence's 0 it moves the pointer over each tile its inputs visit (`look_over_view`), then over
  the kingdom overview and back to the parking (the hint as before) and writes +0xEC and +0xF0 -1.
- **Outside the view** (0x40EB7D): an edge cursor (32 to 39) stays while the mouse now lies on the
  screen within 16 pixels of a side (x < 16 or x > 784, y < 16 or y > 584, x not rounded), any
  other cursor becomes the arrow; then the panel's hint.
- **The panel's hint** (0x403010): with the chat's edit not editing, the item under the point
  (`dialog_item_at` 0x5FF3E0: the dialog's items from the last back, the first whose box holds it
  and whose state has 0x2 without 0x8 or 0x1000); the same id as the static 0x65F228 (initially -1)
  does nothing, so the map's hints leave it (MCL-Q17). -1: the arrow and no text. Read from the
  running original's items: a disabled button (the underground on one level, move hero without a
  green arrow, a list's arrow at its end) has 0x8 and takes no mouse, the quest log's 0x20 does; a
  list's portrait and bars take it only in a slot holding a hero or a town, the hero list's
  selection frames (39 to 43, added after the portraits) always; the chat's edit 38 lies over the
  status bar (no text there); the resource bar's and the status window's items are panels of their
  own: none over the bar, the status window's area 2 over the whole window (observed). The text:
  a hero's slot (15 to 29 and 39 to 43, by its list's top) GENRLTXT's record 16 ("%s the %s") with
  its name (+0x23) and class (`hero_class_name` 0x4D8F70: the class record's +0x4, HCTRAITS; a
  campaign hero's own text, not modelled), a town's slot (32 to 36) "%s, %s" (0x65F3D4) with its
  name (+0xC8, none an empty text) and type (`town_type_names` 0x6A74F4: ARRAYTXT's records 219 to
  227), the ids 1001 to 1015 none, else `panel_hint_index`: 1 to 14 the pair id − 1, 15 to 29 and
  39 to 43 14, 30 15, 31 16, 32 to 36 17, 200 26, the resource bar's 1001 to 1015 18 to 25 (other
  screens'), else none; the pairs 0x6A56E0 are Help.txt's records 291 to 317. The text set into the
  status bar (200), the item drawn and its rectangle shown at once (0x4031AA to 0x4031E9).
- **The map's hint** (0x40B150; nothing while the chat edits): `hint_tile` 0x40AFB0 with the view's
  column and row: at (9, 8) with a selected hero off its tile (+6 0, the lifted one) the hero shown
  there for the hint (`hero_show`) and the mouse's tile; else the entrance of the tile's object
  (`tile_object_entrance`) when another: the same class there that tile, a hero or a boat standing
  there taken off (`hero_hide`) and that tile, else that tile. By its class (switch 0x40B20E, index
  table 0x40D314, 59 cases): 0, 3, 26, 36 the special ground's name (`tile_special_ground`
  0x4FCE20: none gives 0, no text), 34 a hero's text as its slot's, 98 a town's as its slot's, 54 on
  an entrance "%s %s" (0x660344) with the size word of its count (the word's low 12 bits;
  `creature_quantity_word` 0x44ADE0 with form 1: ARRAYTXT's records 175 + 3 n) and its creature's
  plural name (+0x18, none above 150); every class without a case (0x40D0E7) its name
  (`object_names` 0x6A79EC: OBJNAMES's record per class, 0 to 231); the other 58 cases (visits,
  owners, contents): T6a2. The message buffer set into the status bar, drawn and shown at once
  (0x40D178 to 0x40D1D1).
- **Built:** the live part's `[hover]` (the facts only the game knows: each object's entrance word
  and army, the towns' names and types, the heroes' indices and own names, the viewer's lists, the
  teams, the guarded tiles, the selected hero's costs of the tiles looked at and its day points;
  `h3mem.Mem.hover`); the tiles from the map's objects (`record`, the heroes and boats on them; a
  water tile is never the coast's class 3, M3 T6a1's fix to `record`); the timeline's
  `Loop::moved` over the view and outside it, the click's move first; the status bar's text in the
  scene by the catalogue (the families `map-hint`, `object`, `hero-class`, `town-type`, `creature`,
  the keys `map.hint.*`, `map.army.*`; h3-ui's families learn `count`: records one by one, blank
  ones included). A still's hint: the panel's at its parked pointer. After a walk the costs are no
  longer the hero's: the arrow over the tiles they gave (the engine's from M4).
- **Not built:** another player's turn (0x40DEB0's), the chat's edit, a debug flag +0x3D, a
  campaign hero's class text, the hints' 55 cases (T6a2a: §4.11; T6a2b), the clicks (T6b).

### 4.11 The map hint's visit marks (M3 T6a2a)

Read in the sandbox (`scripts/exe.py`, the cases of switch 0x40B20E; M3 T6a2 cut with the user,
2026-10-30, by what runs the hint: T6a2a the 35 classes whose case is inline and shows what was
visited, T6a2b the 20 others: owners, contents and the seven sub-functions their cases call).
Every one of the 35 starts as a class
without a case: its name (`object_names` 0x6A79EC) into the message buffer; on a tile that is
not its object's entrance (+0xD bit 0x10) nothing more. On the entrance it appends, each by
" %s" (0x66034C) or a space (0x660330) and the text, in this order:

- **The bonus** (XtraInfo.txt's record n, `xtrainfo_load` 0x405D80 into `xtra_info` 0x69127C,
  28 records) when the viewing player (`viewing_player` 0x4CEA20, 0 to 7) knows the kind: its
  bit in the game's `kinds_visited` +0x4E344 + k (structs.toml). Class (k, n): Buoy 11 (0, 0),
  Swan Pond 14 (1, 1), Faerie Ring 28 (2, 2), Garden of Revelation 32 (4, 4), Learning Stone 100
  (5, 5), Library 41 (6, 6), Marletto Tower 23 (7, 7), Mercenary Camp 51 (8, 8), School of Magic
  47 (9, 9), School of War 107 (10, 10), Star Axis 61 (11, 11), Fountain of Youth 31 (13, 13),
  Hill Fort 35 (14, 14), Magic Spring 48 (15, 15), Mermaid 52 (16, 16), Rally Flag 64 (17, 17),
  Idol of Fortune 38 (22, 22), Temple 96 (23, 23), University 104 (24, 24), Magic Well 49 (25,
  25), Oasis 56 (26, 26), Watering Hole 110 (27, 27); the Fountain of Fortune 30 (n 3) by its
  own word instead: bit 5 + `shown_player` (0x69778C, 0 to 7).
- **The mark** (GENRLTXT's record 353 visited, 354 not visited), by what the class keeps:
  - with a selected hero only (the screen player's +0x4, `game_screen_player` 0x4CE9B0), by its
    `status_flags` +0x105: Magic Well bit 0, Stables 94 bit 1, Buoy 2, Swan Pond 3, Idol bits 4
    or 25, Fountain of Fortune bits 5, 27, 28 or 29, Watering Hole 6, Oasis 7, Temple 8 or 26,
    Faerie Ring 13, Fountain of Youth 14, Mermaid 15, Rally Flag 16, Sirens 92 bit 20; by the bit
    (word & 0x1F) of its per-object field: Learning Stone +0x57, Marletto +0x5B, Garden +0x5F,
    Mercenary Camp +0x63, Star Axis +0x67, Library +0x6F, Arena 4 +0x73, School of Magic +0x77,
    School of War +0x7B; by that bit of the screen player's record: Corpse 22 +0xC0, Lean To 39
    +0xC4, Magic Spring +0xBC (and its word's bit 6 clear, else not visited); the Pyramid 63 by
    its word's bit 5 + the hero's owner (+0x22, 0 to 7; else not visited).
  - without one too: the Mystical Garden 55 by the bit (word & 0x1F) of the screen player's
    +0xB8 and its word's bit 10 clear; the Obelisk 57 by the viewing player's bit in the game's
    +0x4E3E9 + its word (a signed byte); the Wagon 105 and Warrior's Tomb 108 by the word's bit
    5 + `current_player` (0x69CCA8, 0 to 7; else not visited); the Water Wheel 109 and Windmill
    112 only when that bit is set: visited when what is left is none (the wheel's word & 0x1F
    times 500 in 16 bits, the mill's bits 13 to 16), else not visited.
  - none: Hill Fort, University.

The four players are one in a human's turn of a single-player game (the comparisons'); their
differences matter in the hot seat (M5). **Built:** the live part's `[hover]` `visits`, per map
object whether the viewer knows its bonus and its mark (visited, not visited or none), which the
harness derives from memory by the rules above (`h3mem.Mem.visits`) and the engine gives from M4
(for the hero selected when it is read: a selection changed by a click, T6b, reads it again);
`Hint::Visits` composes the name, the bonus (`map-bonus.<n>`) and the mark (`map.hint.visited`,
`map.hint.not-visited`) with `map.hint.more` (0x66034C) between. A situation's `visits` writes
them into the original for a comparison (`h3map.stage`: the same rules inverted).

### 4.12 The map hint's owners and contents (M3 T6a2b)

Read in the sandbox (kept whole by the user, 2026-10-30): the other 20 cases of switch 0x40B20E and
the sub-functions they call. Each starts with a name; the separators the sub-functions take are a
space (0x660330), so every part after the name is " %s" (`map.hint.more`).

- **Border Guard and Gate** (9, 212; 0x40B2FD) and **Keymaster's Tent** (10, 0x40B328): "%s %s"
  (0x660344) with the colour's name (`keymaster_colour_names` 0x6A7520, TentColr.txt by
  `tentcolr_load` 0x5B9880; the tile's +0x22 subclass) and the class's; a Tent on its entrance
  its mark (GENRLTXT 353/354) by the viewing player's bit of the game's u8[8] +0x4E364 + colour.
- **Dwellings** (17, 20; 0x40B587, 0x40B5CA): the dwelling record's (game +0x4E39C, 0x5C bytes, by
  the word) name by its +1 (CrGen1, CrGen4), with an owner (+0x57) "%s - %s" (0x66033C) and its
  "Owned by" text (`player_owned_names` 0x6A5898, ARRAYTXT 23-30); no entrance test.
- **Lighthouse** (42, 0x40BEA2): its name, on its entrance with an owner (game +0x4E38C, 0x40
  bytes, +0) " - %s" (0x660334). **Resource** (79, 0x40C785): `resource_names` 0x6A5E64
  (RESTYPES, `restypes_load` 0x5B9310) by its subclass.
- **Mine** (53, `mine_hint_text` 0x40D670, the viewing player): the name (`mine_names` 0x6A74D0,
  MineName by the record's +1; its +2 set the eighth, an abandoned mine's), then with an owner its
  "Owned by" text; where the owner is of the viewer's team (game +0x1F879) "(" (0x660354), the
  resource's name and ")"; with guards (the record's army +4) `army_description_text` 0x40ABE0
  without its details: GENRLTXT 203, the size word of the total count in form 2 (ARRAYTXT 176 +
  3 n) and the only creature's plural, or GENRLTXT 44 for several (the stacks merged by creature,
  `army_add_stack` 0x44ACE0). No entrance test.
- **Creature banks** (`creature_bank_hint` 0x40D3F0, the shown player; 16 its subclass, 85 kind 7,
  24 kind 8, 84 kind 9, 25 kind 10): the kind's name (CrBanks.txt's record 2 + 4 k, the table at
  `creature_bank_kinds_pointer` 0x67029C), then GENRLTXT 354 unless the word's bit 5 + player;
  visited, GENRLTXT 353 once looted (bit 25) or its army (game +0x4E3DC, 0x6C bytes, by bits
  13-24) empty, else "(" and the guards as a mine's and ")". No entrance test.
- **Shrines** (88-90, `shrine_hint` 0x40D8D0), **Tree of Knowledge** (102, 0x40DB00), **Witch Hut**
  (113, 0x40DCC0), the current player: the name; on the entrance, once the word's bit 5 +
  player: a Shrine's "(Learn %s)" (GENRLTXT 356) with its spell (bits 13-22, signed) and, when
  the selected hero has it (+0x430 + spell), GENRLTXT 355; a Tree's price (`tree_price_names`
  0x6A64D8, ARRAYTXT 202-204 by bits 13-15; each text begins with a space: MCL-Q20); a Witch Hut
  with a skill (bits 13-19 not all set) "(Learn %s)" (GENRLTXT 357) with it and, when the hero
  has it (+0xC9 + skill), GENRLTXT 358. Else, once the kind is known (kinds_visited 19-21, 18,
  12 by the current player), its XtraInfo row. A Tree then shows the selected hero's mark by its
  +0x6B bit (word & 0x1F).
- **Seer's Hut** (83, `seer_hut_hint` 0x5741B0; its 19-byte record at game +0x1FBD4 by the word,
  the viewing player) and **Quest Guard** (215, `quest_guard_hint` 0x573040; 5 bytes at +0x1FBE4,
  the shown player): a Seer's Hut not visited (its +4 bit) its name, visited GENRLTXT 348 with
  the seer's name (SeerHut.txt 50 + its +0x11); a Quest Guard its name; then, visited and with a
  quest (+0), a space and the quest's text: its vtable (`quest_vtables` 0x64174C + 0x3C x type,
  `quest_create` 0x573240) +0x1C, SeerHut.txt's "rollover" record 5 x type in the column of the
  quest's +4 (seer 1-3, guard 4-6) by its variant +0x38, formatted with the level (+0x40), the
  primary skills above 0 ("%s %i", PRISKILL), the hero's name (+0x44, +0x40 to be), the monster's
  plural (+0x48), the artifacts' names (+0x44's vector), the creatures ("%d %s", the singular for
  one; +0x54's types, +0x44's counts), the resources above 0 ("%d %s"), or the colour's name
  (PlColors) in lower case (0x6172B6); a list joined by ", " and GENRLTXT 142 before its last
  (`list_join` 0x56C960).

The players again are one in the comparisons (§4.11). **Built:** `hover::contents`: the live part's
`[hover]` contents (`mines`, `banks`, `shrines`, `trees`, `witch_huts`, `quests`: sparse rows by
object; the Tent's mark a `visits` rule), `Hint`'s variants (`Keymaster`, `Dwelling`,
`Lighthouse`, `Resource`, `Mine`, `Bank`, `Teaches`, `Quest`; `Hint` no longer `Copy`), the
catalogue's keys and families (`clients/text/keys.toml`); the harness's `Mem.contents`,
`Mem.quest`, `Mem.write_content` (a situation's `visits` with `hero` and `guards`) and the Quest
Log's state as shown (`Mem.item_states`: the harness's writes do not run `quest_log_state`); the
writer's quests (`Extras::quests`, a situation object's `quest`). An object whose data names
nothing (a colour or resource past its table, a defeat-hero quest without its hero) shows nothing
there (MCL-Q21).

### 4.13 The buttons' press, the hotkeys, the lists' clicks and Next Hero (M3 T6b1)

Read in the sandbox (T6b cut by the user into T6b1 to T6b3, 2026-10-30), checked by two
sequences (`choose-lists`, `press-buttons`: every frame exact, the status bar compared).

- **The dispatch.** An item's message of type 0x200 reaches `adventure_manager_procedure`
  0x4087B0: subtype 0xC (the left button down on an item, 0x408A40) `adventure_click` 0x4097E0,
  subtype 0xD (an item's command, 0x408A19) `adventure_command` 0x409A70 (named `end_turn_button`
  before; its switch 0x409AB4 on the id less 3). `adventure_click`'s table 0x409A44 by id: 0 and
  37 the tile's click (`tile_click_path` 0x40A5D0), 1 the minimap (`minimap_click` 0x40A0C0,
  T6b2), 15-19 and 39-43 a hero list's slot, 32-36 a town list's, every other item nothing.
- **A button** (`def_button_message` 0x456190; E17 design §9.10): skipped disabled (state 8) or
  0x20 (the quest log without a quest, 0x456287). The left button down inside it (0x456328)
  presses it (0x456860: state 1, `button.wav` played, 0x694DA4 loaded at 0x4EDA51; drawn in its
  click frame and shown at once) and runs the button's own loop (0x45641C: `pump_messages`, the
  tick 0x4ED490, the next message): the map's idle step does not run (no view redraw, no objects'
  step, no edge scroll; the palettes step in memory and show at the first redraw after it), every
  message but the mouse's goes to the button and is dropped; a move inside presses it again (its
  sound again, 0x4563A2), outside releases it (0x4563B5); the button up (0x10, 0x456451) releases
  it and, pressed, returns its command 0x200/0xD. A hotkey's key down (0x45659F: the message's
  scan code among +0x4C to +0x50) presses it the same way and returns 0x200/0xC (the key never
  reaches `adventure_key_down`, whose switch takes none of the letters); its key up (0x4564D6)
  releases it and returns its command. The hotkeys (`adventure_dialog_create`): K kingdom, U
  underground, Q quest log, Z sleep (W while the hero sleeps, 0x403CC0), M move hero, C cast, A
  and O the options, H next hero, E end turn.
- **Recorded, not read** (*provisional*): the mouse's release ends the hold with the hint of the
  point it left (released off a button, the status bar the point's item's); the map's idle step
  runs again in the release's millisecond (the deadline the hold kept: the objects' step there).
- **The commands** (0x409A70): 3 the kingdom overview, 5 the quest log (0x52E430), 6 sleep, 7
  move hero, 8 cast, 9 and 10 the options, 12 end turn: dialogs and the engine's actions (M4, M5:
  a recording refuses them, live nothing); 4 the underground (T6b2); 11 next hero (0x409D89); 13
  and 14 the hero list's arrows (`hero_list_scroll` 0x403220 with 1 and 0), 30 and 31 the town
  list's (0x403280).
- **Next hero** (0x409D89, a human's turn): a path shown (+0x4C) cleared, the view drawn and shown
  with the objects' step when due (0x40F270), then `hero_select` of `next_hero_find` 0x4BAA40
  (from the one after the selected, or the first, to the end, then from the first to the
  selected: the first awake that can move).
- **A hero list's slot** (0x409829): slot i the hero `heroes[top + i]` unless i (not top + i,
  0x409844: harmless while the update keeps top within the last five) is past the count; the
  selected hero's opens its screen (+0x40 = 2, `hero_move_loop`'s case 1: M4, M5), another
  `hero_select` (hero, 0, 0, 1). The selection frames 39-43 take the mouse in every slot (§4.10),
  over the portraits: a press finds them, a slot past the list does nothing.
- **A town list's slot** (0x40987D; an empty slot takes no mouse): move hero disabled (0x4098BB),
  a path shown cleared and the view drawn and shown with the objects' step when due; the selected
  town opens its screen (+0x40 = 3, case 2: M4, M5), another `town_select` (town, 0, 1).
- **`hero_select`** 0x417B20 (hero, a, b, c): the town selected −1 (0x417B54), move hero disabled
  and the path's flag cleared with c (0x417B80), the selected hero put back (0x417BAC), the new
  one lifted (b clear) and the view centred on it when the viewer sees its tile or a neighbour
  (a hero of the viewer's always: 0x417DAE), the underground's DEF by its level (a change a full
  update, 0x417EBB: T6b2); its place in the list (0x4BA9E0); both lists updated (0x417F09,
  0x417F19), cast and sleep by the hero, the resource bar, the dialog drawn; with c and a = 0 its
  path made again from its destination +0x35 (0x417FD5, `make_hero_path` 0x417FE2 with its third
  argument: move hero by its green arrows); the status window built (0x41800B), +0x1EC, the
  minimap, the view drawn and the whole screen shown (0x418046 to 0x41809B); its ambient sounds
  and its terrain's music (0x4180FF, 0x41812D); the mouse's move posted with +0xEC 0 (0x418146: the
  view's column forgotten, its row kept).
- **`town_select`** 0x417830 (town, b, c), read in full: the hero put back, the selected hero −1
  and the town set; the view's corner its record's place (+5 to +7, the entrance: §4.5) less (9,
  8), never clamped; the underground's DEF by the level (0x41790E); the town list updated to show
  it, the hero list (0x417953, 0x417961), cast and sleep disabled; move hero disabled and the
  path's flag cleared (0x4179AF); `adventure_full_update` (0x4179CE: the status window, the view,
  the minimap); the ambient sounds and the music of the terrain at the view's middle (0x417A33,
  0x417AF8); the mouse's move posted, +0xEC 0.
- **The hint hidden by a dialog's redraw.** Every update of a list with its redraw (and a full
  update) draws every item (`dialog_redraw` 0x5FF020 with the whole range); the chat's edit 38,
  added after the status bar over its box and shown, draws its picture `AdRollvr.pcx` with its
  empty text (`edit_draw` 0x5BB400): the status bar shows no text while its item keeps the hint
  and 0x65F228 the item looked at last, so only another item's or the map's next hint shows a
  text again (observed live: the item's text kept; MCL-Q22).
- **The choices' facts** (the harness): `h3map.py`'s `choose_each` chooses each of Red's heroes and
  towns from their lists before the sequence's 0 (a situation's `choose`), reading each one's
  status window, a hero's path and tile, a town's place, then the staging's selection, the lists'
  first slots and the view again; the live part's `[chosen]`. A situation with `choose` may give
  several heroes a path (each one's destination; the staging's selection's shown at 0).
- **Built** (*provisional*): `timeline::Action`'s `Press`, `Release`, `Key`, `KeyUp` (`Click` a
  press and a release at once), `Hotkey`, `Unplayable::Command`; the loop's hold, the hotkeys'
  presses, the commands (next hero, the lists' arrows), the lists' slots, `hero_select` and
  `town_select` as above, a choice's path kept for the hero chosen away from (its destination
  stays), the hint hidden by every lists' draw; `Draw::selected` and `pressed` (the panels' click
  frames: `Live::pressed`), `Minimap::index`; `live::Chosen` (`[chosen]`; `Live::at_start` gives
  each hero's and town's choice with what a map gives, a town centred on its entrance); the key
  table's `Command::Panel` (`kingdom-overview` … `end-turn`), `h3_sdl::Key`'s K, Q, U and Z; the
  tables' `[map.sound] button`; the window's presses and releases apart and hotkeys' releases.
  After a choice the mouse-over's reach and the hero's own marks are the staged hero's (the
  engine's from M4): a sequence moves the pointer over the map only before its first choice.

### 4.14 The underground, the minimap's click and drag, the status window's click (M3 T6b2)

Read in the sandbox (T6b cut by the user into T6b1 to T6b3, 2026-10-30), checked by three
sequences on the new situation `seq-underground` (a writer's map of two levels): `underground`,
`minimap-drag`, `status-click` (every frame exact, the status bar compared).

- **The underground** (`adventure_command` 0x409F6B, the command 4; its button's press and U as
  §4.13): on a map of two levels (game +0x1FC48) the selected hero put back on its tile, the view's
  level the other (0x409FAE: 1 − the level within the corner's field), its corner kept, the
  button's picture by the level (`underground_button_level` 0x403C40: `iam010.def` on the surface,
  `iam003.def` below, drawn and shown) and `adventure_full_update` with 1. No sound changes (the
  surface's music plays on below), no mouse move is posted. A hero chosen on the other level
  (`hero_select` 0x417EBB) or a town there (0x41790E) changes the level with its centring and
  updates the whole screen as before.
- **The whole screen's update** (`adventure_full_update` 0x417420, read in its middle): the
  background, the buttons' colours, both lists and the quest log updated, the status window built
  (`status_window_choose` with 1, 0, 0: forced, the same kind built again, the date's at its first
  frame with its sound) and the resource bar, then the dialog's every item drawn (its vtable +0x14
  with the whole range: the new window's items too, and the chat's edit 38 over the status bar, so
  the hint is hidden as after a list's redraw, MCL-Q22), the lists' selection frames, the view and
  the minimap; shown whole.
- **The minimap's press** (`minimap_click` 0x40A0C0, read whole; `adventure_click`'s id 1): a
  right click its help text (M4, M5); else the selected hero put back, the point less the area's
  corner (the dialog's item 1: (630, 26), 144 × 144) divided by an f32 by the map's height (4.0,
  2.0, 1.3333 (0x3FAAA993, not the box's 1.33) or 1.0; the tables' `click`) and truncated
  (`crt_ftol`), less (9, 8) the view's corner, held within −9 to the width − 10 and −8 to the
  height − 9; the minimap drawn and shown (0x40A280), the view drawn and shown ((0, 8), 608 × 544)
  and the objects stepped when due, as a scroll's step does (0x40A2D8: `frame_deadline` moved on
  by the larger of 180 and its lateness).
- **Its loop while the button is held** (0x40A344 to 0x40A5BE): `pump_messages`, then
  `input_next_message` (whose wait runs the tick: the palettes step in memory, the mouse poll
  draws the cursor where the mouse is) until the queue is empty or the left button's release; the
  last move among them is kept, every other message (keys, other buttons) taken and dropped. A move
  is held inside the area (x within its width, y from its top to its top plus the *width* less 1:
  harmless on the square area), sent to the mouse manager (its handler 0x4EC560 returns at once:
  nothing drawn), its tile within 0 to the width − 1 for both (the map is square), the corner that
  less (9, 8) with no further limit; then the minimap, the view and the objects' step as the press,
  **whether the corner moved or not** (a move drawn again at the same corner). No idle step runs
  (no redraw, no objects' step, no edge scroll; the map stands still while the mouse does), no
  hint, no cursor change (an arrow over the view stays). The release (a move to another point
  before it, the harness's, is drawn first) ends the loop; the manager's next turn, in the same
  millisecond, redraws when the hold passed the deadline (`minimap-drag`'s 2 400: two objects'
  steps in one millisecond). The release is the loop's: the area's state keeps the press's bit 1
  (observed in memory: the staging's own minimap click leaves item 1 at state 7), harmlessly
  (below).
- **An item's press and release** (`dialog_area_message` 0x44FF60, the areas' and the pictures'
  handler; the DEF items' `dialog_def_message` 0x4EA810 and the text items' 0x5BC440 alike): with
  the state's bit 2 a left button down inside the item's box sets its bit 1 and becomes 0x200/0xC
  with its id (`adventure_click`); a left button up while bit 1 is set, **wherever the mouse is**,
  clears it and becomes 0x200/0xD with its id (`adventure_command`). `dialog_send_to_items`
  0x5FEDE0 offers each message to the items from the list's head until one takes it, and
  `dialog_add_item` 0x5FECB0 puts each new item at the head (−1: one above the head's +0x12), so
  the newest are asked first: the status window's items (ids 2000 to 2024, built last), then the
  resource bar's, the status bar, …, the buttons, the areas 2, 1, 0 (observed: the adventure
  dialog's list in memory). A press on the status window's picture (614, 400, 176 × 166) is its
  items'; around it (inside area 2, 605, 389, 188 × 182) area 2's, which does nothing; a release
  goes to the first item holding a press, a stale one (the minimap's, the view's after a walk,
  whose loops took their releases) only after every newer one: their commands (ids 0 and 1) do
  nothing.
- **The status window's click** (0x409FD8, the command of an id 2000 to 2200; §4.8): the kind
  asked (+0x398) 2 gives 1, another but 0 gives 0, 0 gives 2, or 1 while the kingdom's shows
  (+0x394 2); the kingdom's and the date's asked until now + 3 000 ms (+0x39C; the tables'
  `[map.status] asked`), the selection's with the end left; then `status_window_choose` with 1, 1,
  1: the window built, its panel's items drawn and its rectangle shown (`status_panel_show`
  0x402BC0 draws only the panel's items: the hint stays). So the click fires at the release,
  wherever it is; a window built anew between the press and the release (`status_panel_set`
  0x402C10 drops the old panel's items: a timed kind's end at a redraw) takes the press with it, and
  that release does nothing. The selection's window is kept while another shows (the timeline's
  `selection`), so the chooser's kind 0 builds it again.
- **Built** (*provisional*): `timeline::Loop`'s `underground` (the command 4), `minimap_press`
  and its loop `drag` (`dragging`: the idle step stopped, every action but the moves and the
  release dropped), `release` and `status_click` (`status_held`, dropped by a window built anew),
  the selection's window kept by `choose`; `minimap::divided` and the tables' `click` (each size's
  divisor, exact in integers: no point of the area lies within 0.24 of the next integer, so the
  x87's roundings never carry it across), `[map.status] asked`; `PanelItems::area`, `in_status`;
  `Kingdom::realm` (the kingdom's window as the original would build it: a live part's `realm`,
  `h3mem.Mem.realm`; `Live::at_start`'s from the map: the towns' halls, which `built_at_start` now
  reads with the Town Hall and City Hall bits, and the players a human or the computer may play
  by the map's teams); the situation's `underground` (the writer's second level, terrains alone;
  the harness explores it whole and waits for the screen `adventure`, whose probes take the
  underground's button enabled). Not built: the minimap's right click (its help text: M4, M5), a
  hero or town of the other level in a recording (the harness reads the view's level's heroes).

### 4.15 The keys during a walk, the input ring, Space, the sound without the focus (M3 T6b3)

Read in the sandbox (T6b3 taken whole by the user, 2026-10-30), checked by four sequences on
`seq-walk` (`walk-queue`, `walk-stop-key`, `walk-stop-press`, `walk-stop-space`: every line and
frame as recorded, the status bar compared) and by the hero's tile read in the running original.

- **The input ring** (`input_key_message` 0x4EC0E0, `input_mouse_message` 0x4EC290, read whole):
  every WM_KEYDOWN (kind 1, a repeat's too), WM_KEYUP (2), WM_MOUSEMOVE (4, each one, none
  merged), the left button's down (8) and up (0x10), the right one's (0x20, 0x40) is a 32-byte
  message in a ring of 64; a write that meets the read index moves it on (0x4EC1D9, 0x4EC3F1): the
  63 newest wait, the oldest dropped. Nothing reads it during a key's step or a walk's frames
  (their waits only pump the window's messages, §3.7), so what comes meanwhile waits there. Recorded
  (`walk-queue`): a step east, and during it 80 key messages (east and west pressed and released by
  turns) and Space's two: the ring kept the last 63, whose 30 key downs (east first) stepped one
  after another, Space last.
- **Space** (`adventure_key_down`'s case for the scan code 57, 0x408CC4): nothing without a
  selected hero or on another player's turn. A hero standing on its tile (+0x20C clear: at a
  turn's start, after a scroll) is chosen again: move hero disabled (0x408D06, as `hero_select`'s
  own), a path shown cleared and the view drawn and shown with the objects' step when due (0x408D31
  to 0x408D6F), then `hero_select` with 0, 0, 1 (0x408D83): lifted, the view centred, the path made
  again from its destination (shown as before), the status window built. A lifted hero (after a
  step, a walk, a choice) is not. Then the hero's tile (0x408D88): an entrance (map_item +0xD bit
  0x10) whose class is not 3 is visited (`visit_and_update` 0x4AAAA0: `visit_object` 0x4A84F0,
  then the lists, the buttons, the status window, the resource bar, the minimap and the music
  asked again when faded; read at its calls). Observed in memory (`seq-walk`): a standing hero's
  tile is class 34 with the entrance bit (its own), and once `hero_select` lifts it the tile shows
  what lies under it (class 0, no bit on grass): Space visits the object a hero stands on, never
  the hero itself. **Space does not cancel the steps queued** (MCL-Q10's premise; MCL-Q25): it is a
  message like the others, handled after them; during a path's walk it ends the walk like any key.
- **A path's walk ended** (0x407FA7, §3.7): by any key's down (a hotkey's, Space's, Ctrl's), a
  button's down (the left's or the right's) or a control's message, taken and not handled: a
  hotkey's button is not pressed (its key's up then finds it released and does nothing,
  0x4564D6), Space chooses nothing, a press holds no button (its up, left behind, goes to an item
  holding none: nothing). Recorded: `walk-stop-key` (H; the Down key behind it stepping after the
  walk), `walk-stop-press`, `walk-stop-space`.
- **The focus** (`window_procedure` 0x4F7C00 at WM_ACTIVATEAPP, 0x4F7DE8 to 0x4F7EB6, read whole):
  `app_active` from wParam; **lost** (0x4F7E5E): 0x699614 set when the music stream plays
  unfaded (+0x8C), `sound_focus_lost` 0x599C40 (read whole: with the digital driver up, +0x84 1
  since 0x599A00, and +0x3C its handle; under +0x90 each effect sample's status kept in 0x6A3258
  as "it played" (Miles' 4) and the sample stopped, `AIL_stop_sample`; then, holding the music
  thread's lock +0xA8 (so it waits for a thread fading or fading in, up to about 1.8 s of the
  window's procedure: MCL-Q26) and +0x90, a stream playing has a thread of the stub 0x59A830
  started, which runs `music_stop_fade` alone once the lock is free: the place kept, faded out,
  paused), the videos paused (0x5977A0), the system's cursor shown and the game's hidden
  (0x50DA20 with 1), 0x699609 set. **Regained** after a loss (0x4F7E09): `music_resume` 0x59AC00
  (the track opened last asked for again: a thread fades the stream out if needed, opens it where
  its fade began and fades it in), the effects (0x599B90, read whole: `AIL_resume_sample` on each
  sample that played and stands stopped, Miles' 8; the flags cleared), the videos, the game's
  cursor shown and the system's hidden, 0x699609 cleared. A track opened without the focus is not
  started (`music_thread` 0x59AA15); an effect asked for meanwhile plays (`play_sample` reads no
  focus).
- **Built** (*provisional*): `timeline::Action`'s `Visit` (Space) and `Up` (another key's
  release: a message doing nothing), `Action::messages` (a click two, everything else one: a
  scroll's Control key and a button's move before it not counted) and `ends_walk`, the ring
  (`QUEUED`, 63: the actions waiting during a walk, the oldest dropped, a click's down first), the
  walk ended by a press, a hotkey and Space (a click's up left behind), `Loop::visit` and
  `reselect`, the visit refused in a recording (`Unplayable::Visit`, the engine's) when the tile
  the lift leaves is an entrance by the mouse-over's facts (without them nothing); the key table's
  `visit` (Space), every map key's release an `Up`; `h3_ui::music`'s focus (`Music::focus`,
  `Queue::focus`: the stub's fade once the lock is free, a stream opened without the focus held
  silent, `music_resume`'s request); the window's sound following `Window::focused()` (the effects'
  streams held where they are, `h3_sdl::Audio::hold`; a cue meanwhile plays). Not built: the
  window's procedure waiting for a fade (MCL-Q26), the cursor without the focus (the system's
  arrow, the game's hidden: the window keeps its own), the videos.
