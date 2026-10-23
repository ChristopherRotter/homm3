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
| 4 | 0x412470 | not the puzzle | objects (T1b) |
| 5 | 0x411590 | always | objects through the shadow drawer (T1b) |
| 6 | 0x411F50 | a path shown (+0x4C), not the puzzle | the path (T1b) |
| 7 | 0x410C00 | always | objects (T1b) |
| 8 | 0x4120B0 | a path shown, not the puzzle | the path's arrows, `adag.def` (+0xB0) by the arrows of +0x48 (T1b) |
| 9 | 0x47FC80 | manager +0x1EC (cleared when the current player has no selected hero) | not read (T1b) |
| 10 | 0x412220 | not the puzzle | the unexplored tiles (§1.5) |
| 11 | 0x40F2F0 | always | the view's four corner gems |

So each layer lies over every lower layer of every tile: a road's lower half over the next row's
river, the shroud over the objects and the path, the gems over the shroud. **The corner gems**
(0x40F2F0): `agemul`, `agemur`, `agemll`, `agemlr` (+0xB4 … +0xC0), frame = the viewing
player (0x4CEA20, a colour 0–7), their 46 × 46 at (6, 6), (556, 6), (6, 508), (556, 508), through the plain
drawer (`def_draw_plain`, runs skipped, not mirrored): part of the view, not of the frame picture.

### 1.2 Where a tile goes

- Tile (vx + i, vy + j) is drawn at **(32 i, 8 + 32 j)** plus manager +0xF4, +0xF8, which only
  the construction writes (0x406DF0: 0). So there is no pixel scrolling: the view moves by tiles
  (M2 checks).
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
  row (y = side − 1) only its top 16 rows are drawn, so nothing reaches past the map.
- Rivers and roads skip tiles outside the map; no pass checks a picture against the DEF's frame
  count (the viewer refuses such a map, *provisional*).
- Terrain frames are encodings 0 and 2 (E17 design §3.3); a run in an encoding-2 terrain frame
  would leave what the buffer held (open point, T2's gate counts them).

### 1.4 The map's edge

The terrain pass draws a tile outside the map (0x4B1330 false; the bounds 0x6783C8, 0x6783CC)
with `edg.def` (+0xAC), not mirrored: the corners 16 (−1, −1), 17 (W, −1), 18 (W, H), 19 (−1, H);
the edges top 20 + (x and 3), right 24 + (y and 3), bottom 28 + (x and 3), left 32 + (y and 3);
every tile further out ((x + 16) mod 4) + 4 ((y + 16) mod 4). The bottom edge tests x against the
height (0x412B4D), harmless while maps are square (MCL-Q1).

### 1.5 The unexplored tiles

The pass (0x412220) covers a tile of the map whose vision word (tile_vision_word) lacks the
viewer's bit (0x69CCBC = 1 << the viewing player 0x69120C, set as a human's turn starts, 0x4C7323);
an explored tile gets nothing (the original has no fog of war over explored tiles).

- **The mask** (0x40F8C0): a bit for each of the eight neighbours **not** explored: 1 north, 2 east,
  4 south, 8 west, 0x10 north-east, 0x20 south-east, 0x40 south-west, 0x80 north-west; neighbours
  outside the map count as unexplored (set from the tile's side of the map, never read).
- **Mask 255:** `tshrc.def` (+0xC4) frame ((85 x xor 85 y) div 64) and 3.
- **Any other mask:** v = the table 0x65F694 [mask] (below; only 255 gives 0); v ≥ 100 means
  mirrored left-right and v − 100; v of 1 or 5 on an odd x takes v + 1, v of 3 on an odd y takes
  4; `tshre.def` (+0xCC) frame v − 1 (34 frames used).
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

- Terrain frames with runs (§1.3) and the DEFs' frame counts (`edg` 36, `tshre` 34, `tshrc` 4 at
  least): T2's gate on the user's files.
- The mouse's tile and the 8-pixel offset (§1.2): M3, with a capture hovering at y = 8 + 32 k + 28.
- Passes 4 to 9: T1b. The puzzle and covered modes: later (M5 for the hot seat).
- The minimap (0x412C40, inferred) takes a terrain's colour from its DEF's palette entry 22: M3.
- What 0x6989C0 means and when the DEFs are reloaded (k's origin).
