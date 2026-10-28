# E17 design notes

Working detail of E17: the formats as the client decodes them, the code readings, layouts.
Decisions live in [ADR-0020](../adr/0020-clients.md), the history of done tasks in
[`E17-log.md`](E17-log.md). Figures from the user's files (the Complete edition's four LODs) come
from the real-asset gate, `cargo run --release -p h3-view -- gate` (from `clients/`; its full report
stays local in `clients/out/gate/`).

## 1. The archives' image formats (M1 T2, T3)

Sources: the community's documentation of the formats, VCMI 1.7.5 (`client/render/CDefFile.cpp`,
compared), and the user's files: **every PCX, PAL, DEF and MSK of the four LODs decodes**
(2026-10-12). The decoders are `h3_assets::{palette, pcx, def, msk}`; their module documentation
holds the byte layouts.

- **PAL** (3, all in `H3bitmap.lod`, archive type 96): RIFF files of form `PAL `, a `data` chunk
  with a Windows `LOGPALETTE` (version `0x300`, 256 entries of red, green, blue, flags). `game.pal`
  and `PLAYERS.PAL` add three chunks of 32 bytes (`offl`, `tran`, `unde`: 256 bits, perhaps one per
  index; their use is unknown); `font.pal`'s `data` chunk carries 4 bytes after its entries.
- **PCX** (7 215; archive type 16 for the 6 971 8-bit images, 17 for the 244 24-bit ones): a
  12-byte header (`size`, `width`, `height`); 8-bit: the indices, then a 768-byte palette with
  8-bit channels; 24-bit: the pixels in the order blue, green, red (checked on `Ar_Bg.pcx`). Every
  file has exactly that length. The largest is 800 × 600.
- **DEF** (2 912): header, palette, groups (id, frame count, 8 bytes that are pointer leftovers in
  most files, 13-byte names with garbage after the NUL in 3 855 of them, `u32` offsets). A frame's
  `size` counts the bytes after its header. Distinct frames by encoding: 0 × 665, 1 × 19 732,
  2 × 187, 3 × 8 464.
  - Encoding 2's per-row offsets always equal where the previous row ends (VCMI reads on from the
    first row; the decoder reads each row from its own offset: the same on every file).
    Encoding 3: every width is a multiple of 32, every run ends at its 32-pixel block's end, every
    block starts where the previous one ended. No run in any file overruns its row, no frame lies
    outside its canvas, no offset is negative.
  - **The short header:** the 12 frames of `SGTWMTA.def` and `SGTWMTB.def` (spell sprites, type
    `0x40`) have a 16-byte header; the decoder recognises them as VCMI does (`def` module
    documentation). The game would misread them; nothing names them (§3.4).
  - **Canvas:** 6 frames of `OVSLOT.def` give a canvas of 700 × 113 or 701 × 113, the DEF 701 ×
    114.
  - **Types:** by header 0x40 × 148, 0x42 × 168, 0x43 × 1 600, 0x44 × 58, 0x45 × 21, 0x46 × 4,
    0x47 × 893, 0x49 × 20 (none of 0x41, 0x48). The archive's type equals the header's except for
    12 entries of `H3sprite.lod` typed 65 (0x41): `AVGfire0`, `AvMAbMG`, `AVXabnd0`, `AVXshyd0`
    (header 0x43), `Citrog` (0x42), `HallDung`, `HALLINFR`, `HALLRAMP`, `HALLSTRN`, `HallTowr`,
    `ILCK22`, `ILCK30` (0x47).
- **MSK** (1 670, archive type 79): always 14 bytes; width and height in tiles, at most 8 × 6;
  two masks of 6 bytes. In `AB01_.msk` (3 × 2) both masks' last two bytes are `E0`: the bottom
  rows, the high bits. Their meaning is open (research item 9). 11 MSKs have no DEF of their name
  in their archive.

**Decoder policy** (*provisional*, E17 M1 T2): exact lengths where every file is exact (PCX, MSK,
the RIFF size); a run past its row's or block's end is an error (no file has one); sides above
16 384 and DEF frames above 2^20 pixels are rejected (T8: runs and shared rows make more pixels
than the input holds, so the pixel cap bounds a frame's memory; every real frame fits). Raw palettes: the keyed
indices (transparency, shadow, …) are the compositor's (T1, T6).

## 2. The inventory (M1 T3)

- **Times** (cloud sandbox, release build; 2026-10-12): reading the four LODs (212 MiB) 120 ms,
  parsing their tables under 1 ms, inflating all 11 981 entries (394 MiB) 1.8 s, decoding every
  PCX, PAL, DEF and MSK 0.17 s. A debug build is about ten times slower. Inflating dominates: at
  that rate a battle's batch of 3–5 MiB (ADR-0020 point 7) takes about 25 ms.
- **Duplicates:** 2 848 names are in two archives, always a base archive and its AB counterpart:
  `H3bitmap` + `H3ab_bmp` 2 279 (2 246 identical, 33 different, among them text tables such as
  `ADVEVENT.TXT`), `H3sprite` + `H3ab_spr` 569 (227 identical, 342 different, mostly adventure map
  objects such as `AH16_.DEF`, `AVA0127.DEF`). No archive holds a name twice. The game uses the
  base archive's copy (§3.1).
- **Mentions:** the 142 text tables, `Heroes3.exe` and `h3maped.exe` name 5 949 files exactly and
  through 29 `printf` patterns (those with at least three literal characters count:
  `AVA%04d.def`, `AOFLGB%c.DEF`, …; an MSK counts through its DEF). Not named anywhere (candidates
  for dead entries): 455 of 2 565 DEF names, 1 112 of 5 014 PCX, 30 of 1 448 MSK, 4 TXT
  (`CMPEDCMD`, `CMPEDITR`, `CMPMOVIE`, `SERIAL`), `TIMES08R.FNT`, `FONT.PAL`, `NEUTRAL.H3C`,
  `DEFAULT.XMI`. Examples: `C01SPF.DEF` where the executable names `C01spF0.def`; the series
  `BO01SUMB.PCX` … (spell pictures). Not covered: names built at run time from tables of base
  names, and the maps and campaigns.
- **Names:** 9 133 distinct names; none breaks 8.3 (one to eight letters, digits or `_`, a dot,
  three letters or digits). Case varies (`Ar_Bg.pcx`, `AVWangl.def`); every lookup ignores it.

## 3. Code readings (M1 T1)

Read 2026-10-12 in the user's `Heroes3.exe` (`docs/exe/README.md`; names and summaries in
`docs/exe/map.toml`), compared with VCMI 1.7.5. A reading is one source: the M3 reference captures
confirm the drawing rules before a fix rests on them alone.

### 3.1 Archive lookup and precedence (research item 1)

- At start 0x559150 opens eight archive slots (0x69D870, 0x190 bytes each): 0 `h3bitmap`,
  1 `h3sprite`, 2 `h3ab_bmp`, 3 `h3ab_spr`, then the patch archives `h3pbitma`, `h3psprit`,
  `h3abp_bm`, `h3abp_sp` (absent in the user's installation).
- 0x559320 fills the search lists (0x69E538, slot orders at 0x641028): four rows, one per value of
  an edition number, each with a list for sprites (DEF), one for bitmaps (PCX, PAL, FNT, TXT) and a
  third (sprite, then bitmap; its loader not read). The row is the number behind the pointer
  0x69923C, set once to 0x67F554, which holds **3**; none of the pointer's 78 uses stores through it.
- **Row 3: DEFs search `H3sprite` then `H3ab_spr`, bitmaps `H3bitmap` then `H3ab_bmp`; the first
  archive holding the name wins.** So the base archive's copy wins for all 2 848 duplicates of §2,
  and the AB archives serve only names the base ones lack. (The other rows: 0 patch, then base;
  1 AB patch, AB, then base, the AB copy winning; 2 base only. This build never selects them.)
  VCMI agrees (`config/filesystem.json` mounts `H3bitmap.lod` and `H3sprite.lod` after the AB
  archives, overriding them).
- In one archive, 0x4FAA70 binary-searches the directory with a case-insensitive compare
  (0x4FA660; linear below five entries). Every directory of the four LODs is sorted by lower-cased
  name, so every entry is reachable. DEFs and MSKs live only in the sprite archives, PCX, PAL, FNT
  and TXT only in the bitmap ones, so a list never misses a kind.
- A loader that finds no archive with the name reports it and searches once more; the PCX loader
  (0x55A800) searches `default.pcx` instead (the others: `dfault24.pcx`, `default.pal`,
  `default.fnt`). A loaded DEF is cached by its name (12 characters) with a reference count.

### 3.2 The screen format (research item 2)

- 0x6014F0 sets the display mode 800 × 600 at 16 bits (0x60155D) and asks the primary surface for
  its pixel format; 0x55A6B0 derives each channel's shift and width from its mask. So the format is
  what the surface reports: 565 on current hardware, 555 on some early cards. The client models
  565, the widths a parameter (*provisional*; the M3 capture confirms what the user's setup reports).
- **A palette colour becomes a screen pixel by truncation** (0x5226D0, at load):
  `(r >> 3) << 11 | (g >> 2) << 5 | b >> 3`; a 24-bit PCX pixel the same way (0x44F010). The
  original never shows the 8-bit channels: the user's off-colour pixels are this quantisation (each
  channel loses its low 3 or 2 bits), no filter. All shading works on the 16-bit values (§3.3).
- **Capture and diff:** the 16-bit frame reaches a 32-bit desktop expanded by DDrawCompat or the
  driver (how: a licence question before reading DDrawCompat's code). An expansion that keeps the
  top bits (a shift, bit replication) is undone exactly by truncating the capture back to 565, so
  the M3 diff compares 565 values. **Confirmed (T8, §6):** under Wine on a 16-bit X display the
  surface reports 565, and the original's main menu equals our truncation of `GamSelBk.pcx` (24-bit)
  in every pixel outside its overlays (410 017 pixels, no near-miss).

### 3.3 Drawing a DEF frame (research item 8)

The frame record (0x47C360) holds its own canvas (+0x28, +0x2C), its encoding (+0x24) and its
pixels; small wrappers pick a frame drawer, which picks its loop by encoding. Runs are the RLE
codes; raw pixels are drawn with their palette colour by every drawer below except the shadow pass.

| Drawer | Encodings | Runs |
|---|---|---|
| 0x47C570 (plain) | 1; 3 → 0x47D0A0; 0, 2 → 0x47DD40 | encoding 1: every run in its palette colour, or every run skipped (a flag) |
| 0x47C9E0 (shadowed; battle stacks, through 0x47BD60) | 1; 3 → 0x47D0A0; 0, 2 → 0x47DD40 | 1, 7 border; 4, 6 body; 5 skipped; with an outline colour, 5, 6, 7 take it; 0, 2, 3 and every index ≥ 8 skipped |
| 0x47D0A0 | 3 (1 → 0x47C9E0) | 5 takes the flag colour when one is given; every other run skipped: no shadow |
| 0x47D930 (shadow pass; raw pixels skipped) | 3 | 1 border, 4 body, the rest skipped |
| 0x47DD40 | 0, 2 | encoding 0 has no runs: every index drawn, 0–7 included; encoding 2: runs skipped |
| 0x47EF60 (with its flag; a magic animation's, §7.15) | 1; 3 → 0x47D4F0; 0, 2 → 0x47DD40 | encoding 1: literal pixels blended half and half with the screen, every run skipped; without the flag the plain drawer with runs skipped |

- **Shadow** (on the 16-bit pixel already in the frame): body `d = (d >> 1) & H`, border
  `d = ((d >> 2) & Q) + ((d >> 1) & H)`, H and Q the half and quarter masks 0x47C460 builds from
  the channel widths (0x6968A4, 0x6968AA; 565: `0x7BEF`, `0x39E7`). Per channel: body ⌊c/2⌋,
  border ⌊c/4⌋ + ⌊c/2⌋. **Overlapping shadows compound:** each draw darkens what is there (body
  over body leaves ⌊⌊c/2⌋/2⌋). VCMI blends black at alpha 128 and 64
  (`client/renderSDL/ScalableImage.cpp`): the same intent, other rounding, and it darkens 2 and 3.
- 0x47C9E0 can also blend raw pixels 50/50 with the frame; both its callers pass 0 (unused).
- **Runs of index ≥ 8:** encoding 1 may store any index as a run. The user's files hold 128 such
  runs, 42 in `Cnomad.def` (index 8, the keyed pink (255, 128, 255) left in the art); the shadowed
  drawer skips them, so the original never shows them in battle. The compositor follows the drawer.
- **Which DEFs use what** (the user's files): creatures (type 0x42) encoding 1 with indices 0, 1, 4,
  5, 6, 7 (and 16 runs of 2); map objects (0x43) and heroes (0x44) encoding 3 with 0, 1, 4, 5;
  terrain (0x45) encodings 0 and 2; interface (0x47) encoding 1. The adventure map draws objects in
  two passes, 0x47D930's shadows then 0x47D0A0: item 9's reading starts there.
- **Player colours:** before drawing, indices 224–255 of a sprite's palette take the player's 32
  colours (0x5FFE40 for the 24-bit palette, 0x5FFE20 for the 16-bit one; from a palette of 8 × 32
  colours whose loading was not read). The drawers know nothing of players.
- **A battle stack** (0x43E140, read around its draw call only): its DEF frame (group +0x3C, frame
  +0x40) goes through 0x4951B0 to 0x47C9E0 with an outline colour from the game's 16-bit palette
  or none. When the stack's +0x44 is 0 the frame is mirrored and placed at the anchor's x minus the
  DEF's width (+0x30) plus 196, else at x − 196; y − 267. The anchor, the outline and the rest of
  the battle screen: §4.

### 3.4 The decoders' open points

- **Archive type against header type:** the DEF loader (0x55C7B0) gives the DEF its header's type
  and never reads the archive's, so the 12 entries typed 0x41 load as their headers say.
- **Frame headers by type:** types 0x40, 0x42–0x47 and 0x49 read the 32-byte frame header; any
  other type (none in the files) takes 16-byte records from a table after the palette (0x47C2B0).
  `SGTWMTA`/`SGTWMTB` (header 0x40, 16-byte frame headers) would be misread by the game; no
  executable names them: unused leftovers. The decoder keeps VCMI's reading; nothing draws them.
- **`OVSLOT.def`** (named by the executable): the drawers mirror and clip against each frame's own
  canvas, never the DEF's; so does the client.
- **Encoding 2:** 0x47DD40 starts each row at its own `u16` offset, as the decoder does.

### 3.5 Still open

The drawers 0x47D4F0 and 0x47E820 (a half blend, rotations) not read (0x47EF60, H3API's
`DrawTransparent`, read at M3 T3b: §3.3, §7.15); how a 24-bit PCX is drawn, and an 8-bit one outside the cases of §4.4 (index 0 is
transparent only when the caller asks), not read; DDrawCompat's expansion (licence first); 555 not
modelled. The battle screen's open points: §4.7.

### 3.6 The bitmap fonts and the text drawer (M3 T4a)

Read in the font loader (`font_load` 0x55BB00 → `font_read` 0x55B8D0 → `font_from_stream`
0x55B750 → 0x4B5070, 0x4B5180), `text_draw` (0x4B5490, H3API's `TextDraw`), its line drawer
(0x4B5260), its glyph drawer (0x4B51A0) and its line count (0x4B5820); built as
`h3_assets::fnt` and `h3_render::text`.

- **The fonts.** Startup loads five into globals (0x4EE458): `tiny.fnt` (0x698A04),
  `smalfont.fnt`, `medfont.fnt`, `bigfont.fnt` and `Calli10R.fnt`; others load by name. A name
  is looked up as a loose file first, then in the archives, else `default.fnt` (which the
  archives lack). The loader copies the file's first 0x1020 bytes verbatim into the font
  object (+0x1C) and the rest as the glyphs' pixels, checks nothing, and copies `game.pal`'s
  16-bit palette into the font (+0x103C; its colours at +0x1058): a font's colours are
  `game.pal`'s, not `font.pal`'s.
- **The file** (`fnt` module): a 32-byte header (byte 5 the line height, byte 6 a signed y offset
  the drawer adds to each line; the files hold `1F FF 08 00 00` or `1E FF 08 00 00` before them),
  256 glyph metrics (three `i32`: the space before the pixels, their width, the space after; the
  advance is their sum), 256 `u32` offsets, the pixels (width × height bytes a glyph, row by
  row). A pixel 0 is not drawn, 255 is drawn in the text's colour, any other in `game.pal`'s
  entry 32 (black). The game's nine fonts (all in `H3bitmap.lod`): heights 11 (`tiny`) to 31
  (`CREDITS`), y offset 0 in each, every glyph's pixels inside the file; `bigfont.fnt`'s pixels
  hold all 256 values, the others 0, 1 and 255 (`tiny` 0 and 255).
- **`TextDraw`** (text, screen, x, y, width, height, colour, alignment, caret): the colour
  argument is `game.pal`'s entry colour + 9 (or the entry itself with bit 0x100); `{` and `}`
  switch a highlight on and off (the next entry), drawn and counted as nothing. The alignment's
  bit 4 centres the lines in the height ((height − lines × line height) / 2, or (height − line
  height) / 2 when they do not fit and the box is less than two lines high), bit 8 puts them at
  the bottom, and the rest aligns each line (1 centred, 2 right, else left; halves truncated
  toward zero). A line is measured glyph by glyph up to a newline, the end, or the first glyph
  reached with the width already past the box's; a negative left space of its first glyph and a
  negative right space of its last widen it. Too wide, it breaks after the last space that lets
  it fit; with none (one long word) at the first glyph from its end that lets it fit, keeping
  the whole width for its alignment, and that glyph is lost (the next line starts after it); in
  the box's last line (two more line heights would pass its bottom) at that glyph whatever the
  spaces. Drawing stops at the first line that would pass the bottom below a line already drawn.
  The line count (bit 4 and 8) breaks the same way without the negative spaces and the last-line
  rule.
- **The line drawer:** the line moves down by the y offset and right by its first glyph's
  negative left space; it is drawn only if it lies inside the box's height; glyphs left of the
  box are skipped and the line ends before the first glyph that would pass the box's right edge.
  The glyph drawer does not clip (its callers keep texts on the screen; the client's screen
  clips). The caret (`_`, for an input field) is not built.
- **The long word's bug** (the user, 2026-10-15: fixed, reproduced only for the comparisons;
  `h3_render::Bugs`, the epic's Decisions): the executable loses the glyph at a long word's
  break, aligns that line by the whole word's width and leaves a line empty when not even the
  word's first glyph fits. Fixed, the next line starts at the glyph cut off, the line is aligned
  by its own width, and a first glyph wider than the box takes a line alone; the line count
  breaks alike. The comparisons with the captures (`--frames`: the set, the frame check) draw
  the original's. A count box shows at most five digits from a map, so no capture shows it yet.
- **Not reproducible:** a line's width taken from a glyph of 128 or above with a negative space
  reads that space at the glyph's value − 256 (a sign-extended index, before the metrics;
  `tiny.fnt`'s glyphs 236, 238, 239 have negative left spaces): the client takes the glyph's own
  in both modes (no such line is drawn yet).

### 3.7 The tint (M3 T4a)

`pcx16_adjust_hue_saturation` (0x44E940, H3API's `AdjustHueSaturation`; screen, x, y, width,
height, hue, saturation; built as `h3_render::tint`): the rectangle is cut at the screen's right
and bottom edges only. Per pixel, each channel c becomes c × K with K = 0x7FFFFFFF / its mask
(unsigned; the surface's masks at 0x694D68, 0x694D60, 0x694D64: red, green, blue); V, the largest,
is stored as an f32; with H = f32(hue × 6), i = `crt_ftol`(H) and f = `fmod`(H, 1) (0x617F20, the
CRT's), p = f32((1 − s) V), q = f32((1 − f s) V), t = f32((1 − (1 − f) s) V), each operation at
53 bits on f32 operands; the channels are (V, t, p), (q, V, p), (p, V, t), (p, q, V), (t, p, V),
(V, p, q) for i = 0 to 5, each rounded to an integer by adding 1.5 × 2^52 and taking the low
word (ties to even), divided by its K and kept to its mask. Any other i leaves the pixel as it
was (the switch's default). V's f32 loses the low bits of a large channel: with no saturation a
grey comes out a shade off (0x1234 becomes 0xA513). The client computes it in integers, each
x87 result and f32 store rounded to its 53 or 24 bits; over all 65 536 pixels it equals a
reference in IEEE doubles for nine hues and saturations (the tests' digests).

## 4. The battle screen (M1 T5)

Read 2026-10-13 in the user's `Heroes3.exe`, compared with VCMI 1.7.5 (`client/battle/`,
`config/obstacles.json`). The path read: the battle's redraw 0x494440 (H3API's
`H3CombatManager::Refresh`) as an ordinary battle shows it: no siege, not hidden (0x46A4A0), no
replay of a network game; the siege branches are read in §10.1 (M4 T1a). Coordinates are screen
pixels of the 800 × 600 frame; the battlefield takes its top 800 × 556.

### 4.1 The hexes

- 0x4642D0 fills the screen box of each of the 187 hex records (row r = 0–10, column c = 0–16,
  hex = 17r + c; e = 1 on even rows, which sit half a hex to the right, else 0), six `i16` from
  +0x0: **anchor** x = left + 22, y = top + 42; left = 14 + 44c + 22e; top = 86 + 42r;
  right = left + 44; bottom = top + 42; and +0xC = top + 52, the cell picture's bottom.
- The anchor is where sprites stand: the hex's centre column, 42 px below its top (the cell
  pictures `CCellGrd.pcx` and `CCellShd.pcx` are 45 × 52 and drawn at (left, top)).
- The mouse's hex (0x4647A0) inverts the 44 × 42 grid inside x 14–761, y 86–557, with a test of
  the hexagon's slanted edges (not read in detail).
- VCMI's `hexPositionLocal` has the same box (x = 14 + 22e + 44c, y = 86 + 42r; 45 × 52).

### 4.2 A stack's sprite

- **The call:** the redraw calls 0x494F40 per hex, which calls 0x43E140 (stack, x, y, part) with
  that hex record's anchor; 0x43E140 adds the stack's +0x104 to x and +0x100 to y (0 at rest; set
  by animations, M2). Part 0 draws the sprite, runs on into the count box (0x43E568, §4.3) and
  ends with a magic animation over the stack (0x43E795, §7.15); part 1 enters at the box and
  draws it alone, called by 0x494F40 only in a siege (+0x53A8), after the moat's parts (M3
  T5b-2b: read for the heal over the patient's box in `machines`).
- **Which hex:** 0x494F40 skips a hex whose +0x1A equals the stack's facing (+0x44). The stack's
  placement (0x4687C0; 0x43D8B0 at the start) writes +0x1A: −1 under a one-hex stack; under a
  two-hex stack (facing 1) 0 at its position (+0x38) and 1 at its other hex position + 1, (facing
  0) 1 at its position and 0 at position − 1. So every stack is drawn once, **from its position
  hex**.
- **Facing:** the setup sets +0x44 = 1 − side (0x43D846): the attacker 1, the defender 0. Facing 1
  draws the frame as stored, its canvas's top-left at **(x − 196, y − 267)**; facing 0 draws it
  mirrored at **(x − W + 196, y − 267)**, W the DEF's width (+0x30; 450 for every creature DEF).
  The drawer reflects canvas column cx to W − 1 − cx (the client's `Placement::mirrored`), so the
  creature's column 196 lands on x facing right and on x − 1 facing left.
- **Two-hex stacks: the exe's position is the rear hex.** The other hex lies ahead of it (+1 for
  the attacker, −1 for the defender), and the sprite is anchored at the rear. VCMI, whose position
  is the head, shifts by ±44 px to get there. **The user decided (2026-10-13) that the engine
  follows the executable:** since PR #150 (BAT-GEO-002, protocol v23) the engine's position of a
  two-hex stack is its rear hex, and the client anchors every stack at the engine's position
  without a conversion (M1 T7).
- In numbers, the canvas's top-left on hex (r, c): (−160 + 44c + 22e, −139 + 42r) facing right,
  (−218 + 44c + 22e, −139 + 42r) facing left. VCMI's `getStackPositionAtHex` (base (−189, −139),
  ±29) gives the same points.
- **The drawer:** 0x4951B0 → 0x47BD60 → the shadowed drawer 0x47C9E0 (§3.3), source rectangle
  (0, 0, W, H) of the DEF, group +0x3C, frame +0x40. A creature's shadow is part of its frame and is
  drawn with it; the battle has no separate shadow pass.
- **The outline:** the colour passed is entry k of `game.pal`'s 16-bit palette (0x6AACB0, loaded
  at 0x4EE430 beside `Players.pal`, 0x6AACA8 and 0x6AACAC): k = 0x60 for the active stack (manager
  +0x132C8), 0x70 when the stack's byte +0x4F1 is set or the mouse marks its hex (manager +0x132CC,
  +0x132D0), else 0, whose colour is black, 0: no outline (0x43E23F to 0x43E272). `game.pal` holds
  a yellow ramp at 96–103 ((255, 255, 0), 224, 192, 160, 128, 160, 192, 224) and a cyan one at
  112–119 ((0, 255, 255), 223, 191, 159, 128, 160, 192, 224). The battle's idle step (0x4960D0)
  rotates both ranges by one entry (0x522950 with −1: each entry takes its lower neighbour's colour,
  the first the last's) and counts manager +0x13FFC up, at most once per 100 ms (not read in
  detail). So the outline pulses with a period of eight steps; at time 0 it is (255, 255, 0).
- **Palette effects** (0x43E280 to 0x43E4C7, each on the DEF's 16-bit palette, restored after the
  draw): a clone (flag 0x800000), Stone Gaze (+0x2B0 > 0) and the flags 0x20000000 and 0x40000000
  (with +0x450) recolour the frame; not read inside (M2, M3).
- **Walking:** in group 0 a stack that does not fly moves its anchor by frame × 44 / n and
  frame × 42 / n by its direction +0x48 (the switch at 0x43E1FE); M2.

### 4.3 The count box

- 0x43E140 from 0x43E568, after the sprite in part 0 (and alone in a siege's part 1, §4.2), before
  a magic animation over the stack (M3 T5b-2b): drawn when the manager forces it (+0x13D34),
  else only for a stack that is not dead (0x200000), not a war machine (creature flag 0x40), whose
  +0x30 (H3API's `skipCountDraw`) is 0 and whose group is 1 or 2 (mouse-over, standing).
- The picture `CmNumWin.pcx` (30 × 11; manager +0x13D54), index 0 transparent (0x4958E0 →
  0x44FDB0), its top-left at the anchor plus:

| | the hex ahead free | the hex ahead taken |
|---|---|---|
| facing right | (22 + 44w + t, −15) | (−15 + 44w, −15) |
| facing left | (−52 − 44w − t, −30) | (−15 − 44w, −15) |

  w = 1 for a two-hex stack, else 0; the hex ahead is the position ± (1 + w); it is taken when a
  living stack whose +0x30 is 0 stands on it or it is blocked (flag 0x2); t is the creature's
  troop-count offset: `CRANIM.TXT`'s cell 22 of the creature's row (counting from 0; VCMI's
  `troopCountLocationOffset`), which 0x50CA00 stores at +0x3C of the creature's 0x54-byte
  animation record (table 0x6998E0) and 0x43DA0F copies into the stack at +0x110 (so +0x14C).
- The box is drawn only when `battle_draw_pcx` draws it (its dirty-rectangle tests, §7.11); then
  its inside is tinted (§3.7, 0x43E6A0: from (1, 1), 28 × 9) with the hue 0.75 and the
  saturation 0.8 when the stack has no active spell (+0x194 = 0); else the hue is
  f32((r + 1) × 0.1667) with r = Σ k / Σ |k| over its 81 spells' durations +0x198 that are not 0,
  k the spell table's first field (0x43E6AA to 0x43E73E; read at M3 T7b-2a: the record's +0, 1
  positive, −1 negative, 0 neither; r = 0 when Σ |k| is 0, so neutral spells alone give 0.1667;
  r, r + 1 and the product by the f64 constant f32(0.1667) each at the x87's 53 bits, the hue
  stored as an f32). The count is `%d` of
  +0x4C, or of +0x50 when it is not −1 (below, "The number"), written by `TextDraw` (§3.6)
  in `tiny.fnt` into (x, y, 30, 15) with colour 4 (`game.pal`'s entry 13, white) and alignment 1
  (centred, from the top). A count of seven digits or more shows its first five (the box's last
  line cut at the glyph that fits, §3.6).
- **Built** (M3 T4a): the scene's `Draw::CountBox` after each stack's sprite (and before any
  magic animation over it since M3 T5b-2b) for a living stack, not a war machine, in group 1 or
  2, by its pose's hex and facing; the hexes ahead taken by every living stack's poses and the blocked hexes; the
  layout's `[count_box]` holds the numbers. Every creature's troop-count offset is 0 in the
  harvested tables. Checked: the capture set without the count-box mask, 31 situations, all 57
  frames exact (§6.2). The spells' tint (M3 T7b-2a): the timeline's `tints` (`anim::TintKey`):
  a stack's active spells follow the record's `EffectApplied` and `EffectEnded` (the snapshot's
  effects at the start), and its box takes `scene::spell_hue` of their signs (the asset tables'
  `[[spell]]` `sign`, harvested from the record's +0; computed exactly in integers, checked
  against the executable's arithmetic run in floats for every sum of up to 81 signs) with the
  first frame drawn after the change: a timed spell's with its animation's first frame, Cure's
  likewise (its effect first), Dispel's and Disrupting Ray's with the redraw after their
  animation (their effect after it), a round's expiry with the next turn's start (its redraw;
  `resist` at 2 900 ms).
- **Over time** (M3 T5a). +0x30 is stored only by the walk and the flight (no other byte store to
  it found). `stack_walk` sets it before the path's first step, so before its turn (0x442145),
  and clears it at the end, after group 21, the standing frame, the turn back of a plain move and
  group 2 set, then redraws the whole battle and shows it at once (0x4423D9, `battle_redraw`
  0x4423E7 with its first and fifth arguments). `stack_flight_animation` sets it after the turn
  and group 20, as the cycles start (0x4B4BD8), and clears it after group 21 and the standing
  frame (0x4B4E73) without a redraw: `stack_fly`'s next frame (the turn back's, or group 2 at
  0x4B4A15) draws the box. While it is set the stack has no box and leaves the hex it stands on
  free for another's (0x43E61C reads the occupant's +0x30). A stack killed by a strike stays on
  its hexes through the strike's iterations; `stack_death` takes it off the hex records'
  occupant byte (0x444400) after them (0x4692C4), so its hex is free for the next redraw. The
  viewer: the poses carry the flag (`Pose::boxless`; the walk's last pose shown again unhidden by
  the whole redraw at the same moment), and a stack holds its hex for a box while it is not hidden
  and not among the corpses (`Pose::holds_hex`).
- **Checked** (M3 T5a; the frame check with the boxes unmasked, `boxes = true`): `walk`, `turn`
  and `climb` match outside redraw artifacts (in `turn` and `climb` the standing frame after the
  path's first turn hides the box), `strike` too but at 2 730 ms. While the Archangel lands
  (2 330 to 2 630 ms: group 21 and the standing frame, the flag set) the Pikemen's box stays
  beside them; at 2 730 (0x4B4A15, the flag clear) both boxes stand over their stacks; from the
  Goblins' turn (3 730) the Archangel's stands beside it, the Pikemen's hex free. The viewer
  before this, the boxes unmasked: 574 to 872 pixels in each frame from 2 330 to 2 630 ms, 606
  in each from 3 730 on. Left, listed (134 pixels at 2 730,
  §7.11's effects, not copied): a piece of the Pikemen's earlier box (123), which the flier's
  restored rectangle brings back from the saved screen, and the new box's last column (11), right
  of the rectangle the original shows. Not classified: a rule "a box at an earlier place" would
  also pass a box the viewer moves too early, as the Pikemen's before this task. The other six
  sequences (boxes still masked, also where the viewer draws them) as recorded but `melee` at
  3 966, now artifacts only (§7.9).
- **The number** (M3 T5b). +0x50 is −1 from the setup (0x43D69C). `stack_take_damage` (0x444090)
  takes d = (the damage + the top creature's lost hit points +0x58) / the hit points +0xC0 (a
  clone: all), keeps the rest in +0x58 and, when d > 0, stores the count before them in +0x50
  (0x4440DB) before +0x4C loses them; a strike, a shot (and the other callers: spells, Fire
  Shield, the moat) take their damage before `strike_animation` animates it (melee 0x441A19, shot
  0x43FC3B). At `strike_animation`'s end, after the iterations and the pending spells' casts, the
  dead die (0x4692C4), every stack's +0x8 is 4 again and its +0x50 −1 (0x469336 to 0x469379),
  and the whole battle is redrawn and shown at once (`battle_redraw` 0x46938A, its first and
  fifth arguments, no wait). So a box shows the count before a strike's kills through its
  iterations (a target stands with it until its reaction, and after it in its standing frame)
  and the new count from the moment the strike's last frame shows. A second damage before the
  end would store the count after the first's kills (none in the sequences). Recorded (`h3mem`'s
  `shown`): in `shot` the Goblins' +0x4C goes from 40 to 37 as the first shot lands (988 ms),
  +0x50 holds 40 until 1 588 (the standing frame drawn at 1 488 shows), then −1.
- **Built** (M3 T5b): the timeline's `counts` (`anim::CountKey`): each struck stack's count after
  its kills from the strike's last frame shown; `Moment::counts` over the snapshot's in the
  scene; the strike's end flushes the pending poses then (the striker standing, the struck back
  in layer 4, the dead among the corpses: before T5b with the next frame drawn or the next idle
  step, *provisional* since M2 T5c). The frame check of a sequence with `boxes` masks a box
  only where its count is not the one the original's memory shows (+0x50, else +0x4C) and lists
  it (` n<side>.<record>` after the frame's counts): the engine rolls its own damage.
- **Checked** (M3 T5b-1; `shot` and `melee` with `boxes = true`): `shot` exact in all 50 frames
  with the boxes compared (the Goblins' 40 until 1 588 ms, then 37: the engine's first shot
  kills three too; the second shot's end lies beyond the sequence); `melee`, run to 7 000 ms,
  artifacts only: the Wolf Riders' 30 until 6 466 ms, then 12 (the Angels' fixed damage, 18
  kills in both), and at 3 966 ms 198 artifact pixels instead of 96: the Angel's shadow drawn
  twice (§7.11's shadow rule) where the box mask covered it. No box masked for its count. The
  other eight sequences as recorded with the strike's end shown at once.
- **Checked** (M3 T5b-2a; `behind` and `shots` with `boxes = true`): `behind`, run to 4 000 ms
  after its last action (1 000 before) to the Marksmen's second shot's end and their turn back,
  matches as before up to 13 544 ms (its boxes unmasked: the Centaurs' 20 → 12 at 11 144, the
  Pegasi's 10 → 9 at 9 844 and → 8 at 14 268, the engine's kills the original's); its second
  shot kills one Pegasus in the engine and none in the original, so their box is masked and
  listed from 15 492 (` n1.0`, 7 against 8). The struck stacks of a shot take layer 5 (§7.10):
  before, the hit Pegasi's wings lay under the Centaurs beside them (242 to 14 pixels a frame,
  13 768 to 14 068 and 14 992 to 15 292 ms). Left, listed: the first arrow's steps over the
  rocks (13 636 to 13 735, §7.11's "A projectile over the obstacles"). `shots` exact in all 139
  frames with the boxes compared; the Gnolls' box listed from 3 471 to 5 479 ms (` n1.1`: the
  Cyclops' throw kills 6 in the engine, 7 in the original; the Arch Magi's ray then leaves 19 in
  both), the Beholders' 2 → 1 → none as in the original.
- **The heal** (M3 T5b-2b; `first_aid_tent_action` 0x478B90): the tent's heal, min(Random(1,
  trunc(25 × the First Aid getter)), +0x58), comes off the patient's +0x58 (0x478C22), the top
  creature's lost hit points; +0x4C is only read, for the log line's name (one or several, 0x478C6A),
  and +0x50 is not touched. So a heal never changes a box's number (no creature comes back), and
  the patient's box stands under the heal's frames (§4.2, §7.15).
- **Checked** (M3 T5b-2b; `fly` and `machines` with `boxes = true`): `fly` as before in all 102
  frames (artifacts only); its breath kills 8 Pikemen and 13 Gnolls in the engine, 9 and 16 in
  the original (30 → 21 and 30 → 14; +0x50 holds 30 from 2 700 ms), so both boxes are masked and
  listed from 4 300 ms, the breath's end, when both sides show the new count (` n0.2`, ` n0.3`;
  the Pikemen's not from 4 450 to 5 300, while they strike back: no box); the Black Dragon's
  stays 1. `machines` exact in all 106 frames with the boxes compared: the Archers' shot kills one
  of the defending Pikemen in both (20 → 19, +0x50 20 from 1 021 to 2 221 ms), neither Ballista
  kills; the war machines have no box. With T3b's order (the box over the heal) 6 to 29 pixels a
  frame differed in the patient's box from 5 945 to 7 845 ms: in the original the heal's
  translucent frames blend over the box's border and tint (`ff30` → `7b88`, the blend with
  black).
- VCMI's `showStackAmountBox` gives the same four positions (its hex centre is the anchor −
  (0, 16)) but leaves out t and counts every stack ahead as taking the hex, war machines included.

### 4.4 Background and obstacles

- **The background buffer** (manager +0x53B0, 800 × 556), composed once by 0x493CF0: the
  battlefield picture (`CmBk*.pcx`, 800 × 556; which one: §5), whole (0x55CF00 with manager
  +0x13464, H3API's `backgroundPcxName`; not read inside); the absolute obstacle, if manager
  +0x53A0 ≥ 0: that record of the table 0x63BEC0 (34 records of 0x44 bytes: terrain bits +0x0,
  screen x +0x4 and y +0x8, its blocked hexes as `u16` from +0xC ending in 0xFFFF, the PCX's name
  +0x40), drawn at (x, y) with index 0 transparent (0x44FC70);
  siege parts; then 0x493930 draws the grid and the shading of hexes (the cell pictures of §4.1,
  by the options; not read in detail, M3). Each redraw starts by copying the buffer (or its dirty
  rectangle) to the frame at (0, 0).
- **The usual obstacles** are battle objects (manager +0x13D58) of the type table 0x63C7C8 (91
  records of 0x14 bytes: terrain bits +0x0, special-field bits +0x2, height +0x4 and width +0x5 in
  hexes, blocked count +0x6, **layer +0x7**, offsets +0x8, the DEF's name +0x10). 0x495500 draws
  one with the plain drawer (0x47BCF0 → 0x47C570, runs skipped: obstacle DEFs, type 0x47, hold runs
  of index 0 only, so no keyed shadow), never mirrored, group 0, frame = the step counter +0x13FFC
  modulo the group's frame count, its canvas's top-left at **(left, top − 42 (height − 1))** of the
  anchor hex (the obstacle's bottom-left hex).
- **Layer:** 1 (55 types) are drawn in a pass of their own before every stack, 0 (36 types) in the
  row passes (§4.5). VCMI's `obstacles.json` marks exactly these 36 `foreground`, with the same
  sizes.
- VCMI places an obstacle by its picture's height H instead (y = top + 42 − H + H mod 42, 42 higher
  when the first blocked offset is negative or H mod 42 > 37): the same point for 90 of the 91, but
  `ObRgS03.def` (type 42, rough; H = 79, height 2) 42 px lower than the exe.
- Spell objects (Quicksand, Land Mine, Fire Wall, Force Field) go through the same passes; one
  owned by a side shows only when revealed (+0xA) or seen by the acting side (0x49460A to
  0x49463A).

### 4.5 The heroes

0x4952B0 draws a DEF through 0x47BD60 without an outline. Left: the flag `CmFlagL.def` (manager
+0x540C, frame +0x5414) at (29, 20), then the hero (+0x5404, group +0x53E4, frame +0x53EC) at
(−43, −19); right: `CmFlagR.def` at (755, 20), then the hero mirrored at (693, −19); each only if the
side has a hero (+0x53CC). Hero DEFs (`CH*.def`) are 150 × 175. VCMI centres the hero's frame in a
64 × 136 box at the field's edge, which gives the same two points; its flags were not compared.

**Which picture, and the flag's colours** (M3 T3a; `battle_load_heroes` 0x463370, read). The
hero's battle DEF is the first field of record town × 2 + female of 18 at 0x63BD40, the town
being the hero class's (the class table 0x67D868, +0, two classes per town) and the sex the hero
type's (the hero table 0x679DD0, +0); the other fields of a record (two numbers, perhaps a cast's
point, and 5) were not read. So the picture follows the faction and the sex, not the class:
Knights and Clerics share CH00 (male) and CH01 (female). Two towns name theirs the other way
round (Tower: CH05 male, CH04 female; Stronghold: CH013 male, CH012 female), and Conflux takes
CH16 for its men and CH17 for its women of both classes, where VCMI gives every Planeswalker
CH16 and every Elementalist CH17 (`config/heroClasses.json`, 1.7.5); the capture of Pasis (a
female Planeswalker) and Inteus (a male Elementalist) shows the executable's. All 18 DEFs have the
same groups: 0 standing (1 frame), 1 the idle motion (4), 2 defeat (11), 3 victory (11), 4 the
cast (8). The flags (`CmFlagL.def`, `CmFlagR.def`, 17 × 15, 6 frames in group 0) take their
player's colours: entries 224–255 of the DEF's palette become the player's 32 of `PLAYERS.PAL`,
from 32 × its colour (0x4634D5, 0x5FFE20 from its 16-bit conversion 0x6AACA8). A map may change a
hero's sex (SoD); the viewer knows only the hero type's (the asset tables' `[[hero]]`), and a
battle's record names no player colours: the viewer draws the attacker's flag red and the
defender's blue (*provisional*, CLT-Q6). Over time: §7.14.

### 4.6 The draw order

The redraw 0x494440, after the background:

1. Layer-1 obstacles, by anchor: rows top to bottom, columns 1–15 left to right.
2. Siege parts: the back wall (§10.1).
3. The left flag and hero, then the right ones.
4. For each row, top to bottom; in it for each pass 0–7; in it for each column 0–16, left to
   right (right to left in a siege's rows 6–10): pass 0 a siege wall part (§10.1); pass 1 the
   corpses on the hex, in their list's order (a two-hex corpse from one of its hexes, as a stack);
   pass 2 the layer-0 obstacle anchored there; in every pass the living stack on the hex if its
   **draw layer** (+0x8) equals the pass, then its count box and a magic animation over it (in
   a siege the moat's parts and the box again, §4.2).
5. Siege towers (§10.1), then the calls at 0x494AAC and 0x494ABC (not read); then the frame is
   shown.

- The draw layer (H3API calls +0x8 "visibility") is 4 from the setup (0x43D750); 6 for the
  attacker of a melee (0x441CC1); 3 or 7 by direction in 0x43F0B0 (directions 0 and 5: 3; 2 and 3:
  7); 5 for a struck stack (the harness's reading, §7.9; the viewer draws them all, §7.10). So standing stacks draw in pass 4, after the corpses and obstacles of their row, and a
  striking stack over its row's neighbours.
- So a layer-0 obstacle covers the stacks of the rows above its anchor row, its anchor row's
  stacks cover it, and every count box is drawn with its stack, under the stacks to its right in
  the same row.
- **VCMI differs** (`BattleRenderer`: by row, then layer CORPSES, WALLS, HEROES = STACKS,
  OBSTACLES = STACK_AMOUNTS): a foreground obstacle is drawn after the stacks of its anchor row, and
  the row's count boxes after all of its stacks; the heroes sit in the row passes.

### 4.7 Open

- To confirm with reference frames: the obstacle `ObRgS03.def` and the draw order against VCMI's
  (§4.4, §4.6). **Confirmed on grass (T8, §6):** a layer-0 obstacle over a layer-1 one, the
  obstacles' positions and the one- and two-hex stacks' canvases, all exact. **Confirmed on every
  land terrain (M2 T1, §6):** the backgrounds (variant 2, §5), the usual obstacles of each terrain,
  an absolute one, two-hex stacks facing either way, the outline's pixels and its rotation (a step
  per 100 ms through eight entries).
- Not read: the grid and shading options (0x493930); the stack's byte +0x4F1; the palette
  effects. The siege branches: read at M4 T1a (§10.1).

## 5. The asset tables' sources (M1 T4)

What `h3-view harvest` reads (2026-10-13; the format: `clients/README.md`, "The asset tables"):

- **Creatures:** the DEF name at +0xC of the creature records (0x6703B8 through 0x6747B0;
  `h3_formats::exe::creature_table`). A shooter's projectile: the switch at 0x43DD83 in 0x43D9F0
  (creatures 2–146: byte table 0x43DE84, jump table 0x43DE40), whose cases each load one name
  (`mov ecx, imm32`); it runs only for the shooter flag 0x4. The same routine sets stack +0x16C =
  267 − the top offset (+0x3C) of frame 0 of group 2, the value Chain Lightning's distance uses
  (Q43; 0x446630).
- **`cranim.txt`** (0x50C810, 0x50CA00; §4.3): 149 rows (creatures 0–148; the arrow tower has
  none) from record 2, in groups of 14 (eight), 6, 14, 13 and 4 with three records between. Times
  as the executable keeps them: atof, times 9000, 500, 500, 115 in 53-bit precision, truncated
  (installation.md §7.10). Every missile angle in the file is whole, so the table holds degrees.
- **Creature keys:** a pack's creature matches the `crtraits.txt` row of its singular name (from
  record 2, blank records skipped, as `h3-harvest` reads it); all 144 match.
- **Backgrounds** (0x4646D0, which sets the picture's name): a siege takes the town's
  (0x63D2A0; E04c); else a special field 1–9 (manager +0x53C0) its picture (0x63D2C8), field 0
  the shore (`CmBkBch.pcx`, 0x66FF40); else two heroes in boats `CmBkBoat.pcx` (0x66FF5C), a
  battle on a ship (+0x53C6) `CmBkDeck.pcx` (0x66FF4C); else terrain t's picture
  0x63D2F0[3t + v], v from 0x41ADB0 (M2 T1): it counts the tiles of the adventure map within 3 of
  the battle's (x − 3 to x + 3, y − 3 to y + 3, clipped to the map) whose object class (+0x1E of
  the record a helper gives for the tile, not read inside) is a tree-like one (114, 115, 135,
  137, 155, 159, 160), a mountain-like one (123, 133, 134, 136, 157, 158) or dead vegetation (119),
  by a byte table over classes 114–160. Variant 0 if dead vegetation outnumbers both others, 1 if
  the mountains outnumber both, else 2: **an empty surrounding gives 2.** (By VCMI's class names:
  brush, bush, oak, pine, willow, yucca trees, trees; hill, mound, mountain, outcropping, volcanic
  vent, volcano; dead vegetation.) The terrains differ only where the pictures do: dirt (Dd, Mt,
  Tr), grass and snow (Tr, Mt, Tr). A battle's record does not carry its surrounding (CLT-Q4), so
  the scene takes variant 2; the capture set confirms it on all eight land terrains, dirt's
  `CmBkDrTr.pcx` included. Terrains 0–7 only: water and rock have none (the gaps of
  `tests/tables.rs`). **The special battlefields** (M3 T2) are the packs' (`[[battlefield]]` of
  `battle.toml`, E04c T2b) and the observation names one (`battlefield`, protocol v28): the
  shore (field 0) `base:shore`, fields 1–9 `base:magic_plains`, `base:cursed_ground`,
  `sod:holy_ground`, `sod:evil_fog`, `sod:clover_field`, `sod:lucid_pools`, `sod:fiery_fields`,
  `sod:rocklands`, `sod:magic_clouds`, a ship's deck `base:deck`, two boats `base:boats`;
  `harvest` keys their pictures so (the field numbers by the packs' keys), and the scene draws a
  special battlefield's picture over the terrain's. A battle on water always has the deck or the
  boats. The capture set checks the shore and the nine grounds (§6.2); the deck and the boats need
  boats on the map (CLT-Q5).
- **Obstacles:** the type table 0x63C7C8 (`def`, `rows` = height, `layer`) and the absolute table
  0x63BEC0 (`pcx`, `at`), §4.4; keys `obstacle_<n>`: type n, absolute record n − 100 (VCMI's
  numbering, as the packs).
- **Expansions** (research item 6): an entry belongs to the table of the pack that defines its key
  (since M3 T2: `base` 121 creatures, the terrains, 5 special battlefields and 90 obstacle types;
  `ab` 23 creatures; `sod` the 7 SoD fields and their 35 obstacle types, E04c T2b). No table holds
  a key no pack defines (`tests/tables.rs`). Which archive holds a file plays no part: the loader
  searches as the game does (§3.1), and the gate finds every name there.

## 6. Reference captures (M1 T8)

The user (2026-10-13): captures must be automatic, for exhaustive diffs; they run in the cloud
sandbox under Wine (`clients/README.md`, "Reference captures"; the scripts in `clients/capture/`).
Findings of the first run:

- **Running it.** Wine 9.0 (32-bit prefix) on Xvfb at 800 × 600 × 16, full-screen mode. The GOG
  `Heroes3.exe` imports `DirectDrawCreate` from `xdd.dll` (DDrawCompat there, not read); a
  forwarder to Wine's ddraw replaces it. Besides the LODs it needs `Heroes3.snd`, `H3ab_ahd.snd`,
  `H3ab_ahd.vid` and five DLLs; `VIDEO.VID` (629 MB, intro videos) can be an empty archive.
- **Settings** live under `HKLM\SOFTWARE\New World Computing\Heroes of Might and Magic® III\1.0`
  (the executable's value names): `Show Intro`, `Show Combat Grid`, `Show Combat Mouse Hex`,
  `Combat Shade Level`, `Main Game Full Screen`, the volumes and others; `setup.sh` writes them.
- **Input.** Clicks reach the game as on Windows. The arrow keys move the selected hero (not the
  view); a cursor at a screen edge scrolls the adventure map, so the scripts keep it inside. The
  hero-list portrait of the selected hero opens the hero screen.
- **Exactness.** An XWD dump of the display is the screen's 5-6-5 values; the main menu proved it
  exact (§3.2).
- **Determinism.** Two runs of the smoke map placed the same obstacles and gave identical opening
  frames (pixel for pixel): the battle's setup does not depend on the clock here, so a situation
  replays exactly.
- **The opening frame.** The battle screen is complete in the first frame after the transition
  (`h3cap.py first`). A faster AI stack acts at once, so in the smoke battle the Wolf Riders are
  already in their move pose and show no outline; an idle opening needs the human's stack first.
- **The smoke battle against `scenarios/smoke.toml`:** the background (on grass variants 0 and 2
  are the same picture; M2 T1 showed the original takes 2, §5), the pond (type
  22, layer 1), the rock (type 20, layer 0, drawn over the pond), the log (type 19) and the
  Archers, Pikemen and Goblins at their hexes match exactly. Differences: the heroes, the Ballista,
  the count boxes, the panel (not drawn yet), the moving Wolf Riders. Not checked yet: the active
  stack's outline (needs a human-first opening), the background variant on other terrains, the
  absolute obstacles, mirrored two-hex stacks standing.

### 6.1 The harness (M2 T1)

The runbook: `clients/capture/README.md`. What it rests on:

- **The clock.** `Heroes3.exe` (GOG) imports one clock, WINMM's `timeGetTime` (6 call sites, two
  more reads of its import slot), and no `GetTickCount`, `QueryPerformanceCounter` or timer; so
  every animation runs on it. Our `xdd.dll`, which the game loads at start, points that import
  slot at a virtual clock the harness freezes and steps by milliseconds (`xdd.c`); the executable
  stays byte-identical. Frozen, the game keeps reading the clock (about 170 000 reads a second):
  after a step, 20 reads mean it has drawn everything up to the new time. Only the battle runs on
  the stepped clock; the menus run in real time.
- **The driver** (`h3drive.py`): fixed coordinates through the menus, each screen recognised by 12
  probe pixels (`screens.toml`), no fixed waits: from launch to a battle's opening in about 8 s.
  The opening is the first 10 ms step at which the battle panel's probes match.
- **Situations** (`clients/situations/`): `h3-view situation` writes a scenario's battle as a map
  (two heroes side by side, no skills, artifacts or war machines); the original's obstacles come
  back from the capture by `h3cap.py find` (each terrain's types rendered alone and matched at
  every anchor); masks cover the heroes' corners, the count boxes (§4.3, from the record) and the
  panel. The set runs unattended (nine situations in about 70 s; twenty since M3 T2) and fails when a frame's count of
  differing pixels changes. Since M3 T1b-2 each frame is compared with ours at its time, a fidget
  in the original's poses (§7.13): all 17 exact outside the masks.
- **Reproducible:** after a fresh `setup.sh` (a new prefix and run directory) all 17 frames of the
  set came back pixel-identical. An XWD file's colour map holds bytes the X server leaves unset, so
  the set digests the pixels.
- **The diff stays in Python** (*provisional*): the harness runs only in a Linux sandbox with Wine
  and the X tools, and no CI run can see an original file; an `h3-view diff` in Rust would add
  code nothing else uses. Revisit if the comparison moves into the client's own binary (M3).

### 6.2 Findings of the capture set (M2 T1)

- **Every land terrain matches** outside the masks but for the outline's colour: the eight
  backgrounds (variant 2, §5; dirt's `CmBkDrTr.pcx`, where variant 0 had been drawn), each
  terrain's usual obstacles (21 types), an absolute one (`obstacle_100` on dirt), one- and two-hex
  stacks facing either way (Royal Griffins and Cavaliers right; Wolf Riders, Black Knights,
  Unicorns, Nagas, Basilisks, Manticores, Hell Hounds left).
- **The obstacles depend on the battle's tile** (and its terrain), not on the armies or the
  clock: the smoke battle and the grass situation, both at (11, 10), got the same three; on dirt
  six other tiles gave six other sets, (20, 20) and (30, 30) an absolute obstacle. (The engine
  draws them from the battle's seed, BAT-GEO-004; the original's generator was not read.)
- **The outline.** Its pixels match exactly in every situation. Its colour steps through
  `game.pal`'s yellow ramp (§4.2) once per 100 ms: at 0, 100, …, 800 ms after the opening the
  original shows 224, 192, 160, 128, 160, 192, 224, 255, 224 (red = green, blue 0), so the
  opening frame is already one step on and the period is 800 ms; at 700 ms (the ramp's first
  entry) our still matches it exactly. The step falls exactly on the opening's 100 ms grid (§7.1).
  Drawn by time since M3 T1b-2 (§7.13), every frame's outline matches.
- **Idle animation.** In the grass situation the Pikemen fidget from about 200 to 700 ms after the
  opening, identically on every run; its other three stacks stand still for 800 ms. Why: §7.2.
- **The panel** shows its buttons lit on the human's turn and dimmed on the computer's.
- **The special battlefields** (M3 T2; `h3-view situation` sets them up on the defender's tile,
  as the original reads them: survey `E04c-battlefields.md` §4): the shore with water east of
  Blue (the writer gives the tile the coast flag; the battle on sand), and each of the nine
  special grounds with its class's first picture in `Objects.txt` under both heroes (its
  bottom-right tile one east and one south of Blue's, so that 3 × 3 and 6 × 4 pictures both
  cover them). Every one shows its picture and its own obstacle types (`h3cap.py find` takes the
  field's), all exact: the shore twice (at (18, 5) with its absolute obstacle 126), Magic Plains,
  Cursed Ground and the seven SoD fields. The original's obstacles follow the tile here too.
- **The heroes** (M3 T3a; §4.5, §7.14): drawn in every situation since, the `heroes` mask gone;
  each situation names its heroes (Orrin and Crag Hack, as before), and eight grass situations
  show the other 16 battle pictures (a male and a female hero of every town), all exact. The
  original's obstacles did not change with the heroes.
- **The count boxes** (M3 T4a; §3.6, §3.7, §4.3): drawn in every situation, the `count_boxes`
  mask gone from the set (the panel's stays): all 30 situations' 55 frames exact on the first
  run, the tint and the digits included (counts of one and two digits, boxes beside their
  stacks and over them); the new `counts` (999, 12 345, 4 567, 123) exact, a fidgeting stack's
  box included. A map's stack of 65 535 is dropped by the original (a signed −1?). The sequences
  keep the mask where the drawn poses show a box until M3 T5 draws them over time.

### 6.3 The adventure map: experiments by agents (M2 T6)

The user's request (2026-10-13, high priority): agents play the domain lanes' experiment batches
(`docs/experiments/README.md`) in the original themselves. `h3adv.py` (runbook, "An experiment
batch") does it on the harness; everything a record holds is read from the game's memory.

- **Hot seat.** New Game → Multiplayer → Hot Seat → one name per human → the scenario list
  (its defaults: difficulty Normal, starting bonus random) → Begin. Each human's turn opens with
  "<colour>'s turn."; a player without towns gets "you have lost your last town" at the end of
  its turn and the start of its next. Every dialog's OK is found anywhere on the screen by 12
  probes relative to the button (`[button.ok]`), so a dialog's size and place do not matter.
- **What the memory gives** (`docs/exe/structs.toml`, observed in the running game, not read,
  so `inferred`): the heroes at game +0x21620 (156 by type, 0x492 apart, H3API's layout): the
  tile (i16 +0x0, +0x2, +0x4), the owner (+0x22, -1 for heroes not on the map), the name (+0x23:
  an experiment map's custom names), the day's points and the points left (+0x49, +0x4D); the
  player record's head: colour, hero count, the selected hero, the hero list (+0x0, +0x1, +0x4,
  +0x8); the resources (+0x9C, read before). On the batch's map every hero stood where the spec
  put it with 2000 points (one Angel; the Pikeman scouts 1560).
- **The view** (the adventure manager, pointer 0x699268): its top-left tile (+0xE4) and the tile
  under the mouse (+0xE8), packed x in bits 0–9 and y in bits 16–25, signed: the view starts at
  (−3, −2) beside the map's corner. Tile (x, y) is drawn at (8 + 32 (x − vx), 8 + 32 (y − vy)), 18
  tiles across; a click on the minimap (144 pixels, 2 a tile on a 72 map) puts the clicked tile at
  (9, 8) in the view. The harness hovers a tile and waits until the game reports it under the
  mouse before it clicks, so a wrong coordinate stops the batch instead of clicking elsewhere.
- **The path.** With `Show Route` on (`setup.sh`; a new prefix holds 0, and then the first click
  walks), the first click on a tile shows the selected hero's path and the second walks. The
  path's arrows are a u16 per tile of the surface at adventure manager +0x48: 0 none, 1–25 a move
  today (green; the cross is 1, east 12, south-east 13, south 14, north-west 17), 26–50 the same
  arrows for a later day (red: the cross 26, east 37). The marks and their colours are read from
  there, not from the screen (a path shown before hides unchanged marks from a screen diff).
- **Selecting a hero.** A click on another of the player's heroes routes the selected one to it
  (a meeting): the harness clicks the hero list (rows 32 pixels apart) and checks the selected
  hero in memory. Ending a turn: key `e` (the move reminder is off).
- **Batch 2 against the user's results** (2026-10-13; the same `Heroes3.exe` under Wine, without
  the HD mod the user plays with): D3a–D3d as the user recorded them, all 13 fields (50 then a
  green step to 0; 0 and a red mark, no step; 100; 0), the paths 13 green marks each; in three
  runs, the last from a fresh prefix (`setup.sh` anew). D5's 32 piles: in each run wood 5–10 and
  mercury 3–6, with a wood pile of 5 and a mercury pile of 6, as the user's; the amounts differ
  from run to run (the piles are drawn at the game's start, which runs in real time). The starting
  amounts differ by difficulty: the user played Easy (30 wood, 15 mercury, plus the random bonus),
  the harness Normal (20, 10).
- **Questions, the difficulty and battles (E17 H1, 2026-10-19; for the adventure lane's batch E3,
  survey `docs/rules/surveys/E08-battles.md` §13).** The scenario list's five difficulty buttons
  sit at (521 + 32 *i*, 478); the game holds the choice at game +0x1F6D8 (Easy: 30 000 gold and
  30 wood, Hard 15 000). A question is a message box with an OK (item 30725) and a Cancel (30726):
  the harness finds both buttons by their probes and reads the text from the window manager's
  topmost dialog (0x699280 → +0x54; the items listed from +0x2C, the next at +0x8; a text item's
  string at +0x30, its characters at +0x34 and their number at +0x38: H3API's layouts, observed,
  `docs/exe/structs.toml`), never from the game's message buffer (0x6973D8, H3API's
  `h3_TextBuffer` 0x50 lower as its other globals), which a map's own text may not pass through.
  The text names the record it was written from in the original's tables read at run time
  (`h3lod.py`, ADR-0009: the record's number, never its text): a monster's offer to join is
  ADVEVENT 86, to join for gold 88 (or 89 and 90 together), to flee 91, the insult after a
  declined offer 87 (a plain message). Seen on a local map whose monsters had other dispositions
  (h3-experiment places savage ones that never flee until E08 T22a): a compliant group of 7
  Pikemen offered to join, then, declined, to flee, and left the map at a second no (its tile's
  class 0); a savage group of 13 that may flee offered to flee, and a yes started the battle; a
  yes to the join put the 7 into the hero's stack of Pikemen (27). The tile's word holds the
  aggression as 5 signed bits (compliant −4 read as 28 unsigned).
- **Battles from the map (E17 H1).** The battle screen's probes show it; the battle manager's
  pointer stays set between battles, so the screen decides. Stacks (side, creature, count, hex)
  are read at the start and when the result window shows (its OK, `[button.battle_ok]`, 64 × 32 at
  (549, 525)); the outcome from the manager (winner +0x13D48, retreated +0x132B0, surrendered
  +0x132B2). Auto combat (key `a`, control 2004) toggles +0x132C4, but a human side's creature
  acts on its own only with the setting `Combat Auto Creatures` (0x698794) on, 0 in a new prefix:
  `setup.sh` sets it. Retreat (`r`) and surrender (`s`) ask for a confirmation whose OK is the
  question's. The check (`capture/batches/harness-check.toml`, Easy): 20 Archangels against 5
  Pikemen (split 3 and 2 Halberdiers) won by auto combat, 50 experience; a Peasant's retreat from
  30 Pikemen (7 stacks) left the hero off the map (owner −1) and the monster whole; Blue's
  surrender in hot seat moved 470 gold from Blue to Red (D-13's reading) and took its hero off the
  map; a hero with 5 000 experience from the map started at level 5 with its primary skills 3/5/1/2
  for the map's 2/3/1/1 and Diplomacy and Logistics expert for basic and advanced (E3-6's
  question, for the adventure lane to judge). A player left without heroes and towns is out: with
  one human left the game ends in the Hall of Fame, so a map keeps each player a hero.
- **The underground, more heroes, a level-up, a second day (E17 H2, 2026-10-19).** The packed
  view and mouse tile hold the level in bit 26; the path's arrows are a u16 per tile level by
  level, the underground's after the surface's. The panel's level button (725, 211) switches the
  view (the minimap then shows that level); it is enabled only on a two-level map and changes
  with the level shown, so the `adventure` probes leave it out. Shown on a shipped map
  (`Arrogance`, four humans in hot seat, Purple's hero on level 1): the view switched from the
  surface back to level 1, the path to (5, 8, 1) three green marks there, the hero arrived; `h3-map`'s
  writer has no underground yet, so the check has none. The hero list holds the heroes by their
  type's number, not in the map's order; its arrows at (638, 203) and (638, 379) move it a row
  (no scroll position found in the adventure manager): with more than five heroes the harness
  clicks up three times (eight heroes at most) and down to the hero's row, and checks the
  selection in memory. A level-up after a battle shows two offered skills and keeps its OK
  disabled until one is clicked (`[button.ok_disabled]`, probes on the pixels that change when it
  is enabled); the skills sit at the OK's corner + (−150, −64) and (−52, −64). The date is game
  +0x1F63E (day, week, month, u16 each). The heroes off the map at a new game (E3-7) held 40 to 90
  in two runs (147 heroes; 1 and 4 at 90). A map hero's custom experience draws its level-ups at
  the start: 5 000 gave level 5 in both runs, but 3/5/1/2 with Diplomacy expert in one and
  3/4/2/2 with Diplomacy advanced in the other.


## 7. Animation timing (M2 T3)

Read in `Heroes3.exe` and confirmed in the original through the harness: its clock stepped 1 ms
at a time while `capture/h3mem.py` reads each stack's group and frame from the game's memory
(§7.6). The three recorded sequences (`clients/situations/sequences.toml`: a walk, two shots, a
flight with a melee strike and the retaliation) and the grass situation's idle field confirm every
number below at combat speed 0; a walk confirmed speeds 1 and 2. Times are milliseconds of the
game's one clock (0x4F82E0, WINMM's `timeGetTime`). VCMI 1.7.5's choices: §7.5.

### 7.1 Clocks and frames

- **The combat speed** (the setting `Combat Speed`, 0 to 2, default 0; 0x6987EC) picks a factor
  F of 0x63CF7C, f32 1.0, 0.63 or 0.40. Every battle animation's time T becomes trunc(T x F), the
  f32 operands multiplied at the x87's 53 bits (installation.md §7.10): 100 ms gives 100, **62**
  (not 63) and 40; 150 gives 150, 94, 60; 33 gives 33, 20, 13; 115 gives 115, 72, 46. The idle
  step (below) is not scaled.
- **The frame deadline** 0x698998. A frame of an animation is drawn, then waits for the deadline
  (`wait_until` 0x4F82F0), which then moves on by max(the frame's time, the lateness), and only
  then shows (`stack_play_group` 0x446940; `battle_redraw` 0x494440 with its wait flag, for the
  strikes and the shot). While nothing animates, the battle's tick 0x474040 keeps the deadline on
  a grid of trunc(100 F) ms from the battle's start (0x462FA5). So:
  - an animation's first frame shows at the next grid point, up to trunc(100 F) after the
    command (the click at the frozen opening: the walk's first frame at 100 ms);
  - a group's first frame shows at the deadline the previous frame set, so a change of frame
    time takes effect one frame late;
  - waits outside the deadline (the projectile's steps, the pause before a retaliation, §7.4)
    leave it behind: the next animation's first iterations show at once, one after the other.
- **The idle step** 0x4960D0 runs every 100 ms whatever F (its time 0x6989B8; the tick calls it):
  the first at the opening itself, so the opening frame is already one step on (§6.2), then
  exactly at 100, 200, … ms (stepping 99 → 100 shows it at 100). An animation stops it; after
  one, it runs twice at the same moment (its time had fallen behind by more than 100 ms) and then
  keeps the new grid. Its changes show at once.
- **Reading the harness** (§7.6): a stack's state read at t is, inside an animation, the frame
  drawn at t and shown from the next deadline (the screen at the walk's 0 to 99 ms still shows
  the standing frame; group 20's frame 0 shows at 100, when memory already holds frame 1); from
  the idle step, the frame shown at t.

### 7.2 Standing, fidgets and the outline

- **Standing** is group 2's frame 0: nothing read advances group 2 (the memory shows frame 0 throughout).
- **The outline's pulse**: each idle step rotates `game.pal` 0x60-0x67 (the active stack's
  outline) and 0x70-0x77 (the mouse's stack) by one entry, eight entries in 800 ms (§6.2).
- **The mouse over a stack** starts its group 1 at once, which the idle step then plays as a
  fidget, and outlines it in entry 0x70 (§7.13).
- **Fidgets** (group 1), per stack with fidget interval t (+0x154, `cranim.txt` x 9000, i.e.
  9000 ms for most creatures) whose DEF has a group 1, unless blind, petrified, paralysed or an
  arrow tower:
  - at the battle's start (0x479F30), and again after every action (seven copies in the action
    dispatcher 0x478D80, one in `move_and_attack` 0x478900, one in a routine of the battle loop,
    0x474573), every stack's (t above 51) fidget clock +0xFC = now + 2r - t, r = random(50, t): its next
    fidget starts at the first idle step after now + 2r, 100 ms to 2t later;
  - then one frame per idle step, each advance skipped when a draw of 0..100 gives less than 8
    (a stall, 8 in 101);
  - after its last frame it stands again, and its clock becomes trunc(now + r'/2 - t/4), r' =
    random(0, t): the next fidget starts 0.75 t to 1.25 t later (strictly after; 0x4960D0).
  Confirmed in the grass situation: the Pikemen's r = 51 puts their fidget at the second idle
  step (200 ms), its four frames with two stalls end at 800 ms, the new clock fits r' = 8486; the
  other stacks drew r = 8341, 7057 and 7450 (fidgets due after 16.7, 14.1 and 14.9 s). In the
  walk's situation with a longer walk (to hex 45, done at 4832 ms) every clock was drawn again
  then: the Goblins' r = 698 and the Marksmen's r = 2250 put their fidgets (nine frames, one per
  100 ms) at 6232 and 9432 ms, the first idle steps after 4832 + 2r.
- **The draws come from the rules' generator**: the start's through `random_between`, the
  fidgets' through 0x50B1D0, which outside network play is the same C library `rand()` the
  battle's rolls use (in a network game it takes the clock instead). In the original, how long a
  player waits (each fidget draws) changes the later damage, luck and morale rolls. The engine
  has its own generator (ADR-0003), so the viewer must draw its fidgets from a generator of its
  own; they cannot match the original's frame for frame, and the reference diff takes the
  original's poses instead (§7.6).
- The heroes have idle clocks of their own (+0x53FC, set at the start; an idle group after 4 500
  ms; read in part).

### 7.3 Moves

W is a stack's walk time (+0x158, `cranim.txt` x 500, ms per hex), n its DEF's frames in group 0.
A frame of group 0 lasts trunc(f32 W x F / f32 n); every other group trunc(100 F) a frame.

- **A walk** (`walk_step` 0x43F0B0 per hex): before the first hex group 20 (move start), then per
  hex one full cycle of group 0, the stack drawn frame x 44 / n and x 42 / n along the step
  (§4.2); after the last hex group 21 (move end), then standing. A walk stopped early (Quicksand,
  a mine) ends the same way where it stops. Haste makes W trunc(W0 x 0.65), Slow trunc(W0 x 1.5)
  (W0 the stack's own, +0x68; 0x444CE5, 0x444D3A); their end restores it. A record reports a
  walk hex by hex (CLT-Q1, wire v29): one `Moved` per position, so a stack's consecutive `Moved`
  in one command are one walk; the engine's path may differ from the original's on ties (Q53).
  Confirmed (sequence `walk`): the Champions (W 450, n 8) show group 20's three frames at 100,
  200 and 300 ms, group 0 every 56 ms from 400 over four hexes, group 21 at 2192 to 2492, standing
  at 2592; at speed 1 the frames are 62 and 35 ms apart, at speed 2 40 and 22. Haste and Slow
  (M3 T7b-2a, sequence `haste`): the products of the integer W0 and the doubles fall on an integer
  or at least 0.05 from one, so W is W0 × 65 / 100 and W0 × 3 / 2 in integers (the viewer's
  `walk_time` by the stack's active spells); the Pikemen (W0 575, n 6) walk hasted at 62 ms a
  frame (W 373) and slowed at 143 (W 862), as recorded on the first run.
- **A flight** (`stack_flight_animation` 0x4B4A40): group 20, then k = (d + D/2) / D cycles of
  group 0 at the walk's frame time along the straight line between the hexes (d their distance
  in pixels, D the flight distance +0x160, `cranim.txt` x 115), the hex taken at the end; group
  21, standing. Confirmed (sequence `melee`): the Angels (W 410, n 7, D 115) fly 352 pixels from
  hex 35 to 43 in k = 3 cycles of 58 ms frames from 300 ms; group 21 shows from 1518. A flight
  of one hex (d 44 or 47, under D/2) has k = 0: the executable skips the cycles (0x4B4C5B) and
  lands at once (0x4B4E1C), group 20 then 21 (E17 M2 T5c, the reviewer's question).
- **A turn** (`stack_turn_around` 0x446720, animated): facing right, group 7, the flip, group 10;
  facing left, group 9, the flip, group 8; each all its frames; then standing. When a stack turns:
  §7.10. A walk's turn is in the sequence `turn` (§7.10).

### 7.4 Strikes and shots

- **A strike's animation** (`strike_animation` 0x468990, H3API's ApplyAnimationToLastHitArmy):
  L iterations of trunc(100 F) on the frame deadline, L = max(a magic animation's frames,
  A + R - 1), A the striker's group's frames, R the longest reaction's: a death's group 5, any
  other stack struck its hit group 3, a defending one's too, which draws group 4 (0x468BD7 to
  0x468C84; M3 T7b-2b-3, `thunder`'s Vampire Lords: hit 7, defend 9, L = 8 + 7 − 1, the defend
  group from iteration 4). The striker plays its
  group from iteration 0: by the strike's direction 11 (up), 12 (straight), 13 (down); 17-19 for
  a breath on two stacks where the DEF has the group, else 11-13 (`melee_strike` 0x441ACA to
  0x441B66; its second stack, [ebp − 8] at 0x441A35, is set only for a creature with flag 0x8,
  0x44171B; the Ballista takes its shot groups). The viewer tells a breath by its second stack
  standing beyond the striker's neighbours, where a multi-headed striker's stand beside it
  (the record does not name the ability; E17 M2 T5c; confirmed by the sequence `fly`, §7.13). Each struck stack reacts
  once the iterations left are no more than its reaction's frames, so the reactions end with the
  loop (with one striker they start at its second-to-last frame, iteration L - 1 - R = A - 2): death 5 (its last
  frame stays: the corpse), defend 4 (a defending stack), else hit 3; then they stand. The
  pending spells are cast afterwards, before the dead die (0x469102; §7.16, T7b-2b-3). Before a
  retaliation and before a second strike
  `melee_attack` waits trunc(150 F) (0x441E05, 0x441EB6) outside the deadline, so the next
  strike's first iteration shows at once.
  Confirmed (sequence `melee`, times shown): the Wolf Riders walk four hexes and strike (group
  12, 7 frames) from 4166 ms; the Angels' hit (6 frames) starts at iteration 5; after 12
  iterations the Angels strike back: 150 ms after the strike's last frame shows (5266 ms) their group
  12's frame 0 shows at once (5416), then every 100 ms; the Wolf Riders' hit (7 frames) from
  iteration 4.
- **A shot** (`shot_animation` 0x43F2C0): the shot group (14 up, 15 straight, 16 down by the
  angle, 0x469880) up to its climax frame c (+0x150, `cranim.txt` cell 23; all frames without
  one), each frame trunc(trunc(A / c) x F), A the attack time (+0x15C, x 500; only shots use
  it). Then the projectile in trunc((d + 20) / 40) steps over the d pixels to the target, each
  trunc(33 F) ms from its own start, outside the deadline (about 1 200 pixels a second); a ray
  shooter waits trunc(115 F) and draws its ray (not read). Then `strike_animation`: a shot marks
  no striker (`melee_strike` sets the striker's flag, 0x441A87; `stack_shot` sets none), so A is
  0 and the loop counts the reactions alone: the longest starts as the projectile lands, however
  many frames the shooter has left after its climax (the Cyclops 9 of 16, the Marksmen 1); the
  shooter, still in its shot group (0x468A55), plays them on beside it, a frame per iteration,
  then stands. The deadline being behind, the reaction's first frame shows for no time at all.
  (Until E17 M2 T5b the viewer counted the shooter's frames left in the loop, which delayed a
  Cyclops' target by 0.7 s: the user saw it; the Marksmen's recorded shot cannot tell the two
  apart. The reactions' part of the loop has one iteration more than their frames, as the
  Marksmen's sequence shows; the reading's L = R: *provisional*. A defending stack's group 4
  counts here, not its hit group as after a melee strike: `machines`' Pikemen, hit 6, defend 13,
  play all 13 after the Archers' shot, as `arrow`'s defending targets after Magic Arrow; T7b-2b-3,
  *provisional*.) The shooter plays its group
  out beyond it and stands after its last frame (M3 T1b-1, sequence `shots`: the Cyclops' 10
  frames after its climax run on 400 ms past the Gnolls' 6-frame hit, and past the Beholders'
  5-frame death; until then the viewer cut them, T5b's rule): L = max(R + 1, the
  shooter's frames left + 1), the reactions placed by the first (*provisional*, not read).
  After a ray the shooter's group is over (§7.12) and it stands from the loop's first
  iteration.
  Confirmed (sequence `shot`): the Marksmen (A 500, c 7) show their 71 ms frames from 100 ms, the
  arrow's 14 steps every 33 ms from 526, the Goblins' hit from 988 (frame 0 never shown, then 100
  ms apart); the second shot (Marksmen shoot twice) starts 100 ms after the first's hit ends.
- **Defend and Wait** have no animation: the next stack acts at the same millisecond (the grass
  situation, keys `d` and `w`: the Wolf Riders' walk starts at once).

### 7.5 VCMI 1.7.5

`client/battle/CreatureAnimation.cpp` (`AnimationControls`), `BattleStacksController.cpp`:

| What | `Heroes3.exe` | VCMI |
|---|---|---|
| a frame (melee, hit, defend, death, turn, move start and end) | trunc(100 F) | 100 ms / factor |
| speeds | durations x 1.0, 0.63, 0.40 | factor 1, 2, 3 (its comment guesses 36/60/100) |
| walking | one cycle of group 0 per hex, W / n a frame | frames 100 x W/500 ms apart, hexes W apart: the hex speed agrees, the frames are not tied to the step |
| shot | A / c a frame up to the climax, then 100 ms | every frame 100 x A/500 ms |
| projectile | steps of 40 pixels every 33 F ms | 1 250 pixels a second, smooth |
| flight | k cycles of group 0 (D pixels each) at W / n | 250 / (W/500) pixels a second (its comment: "H3 uses Walk Animation Time"; both are used) |
| fidget | the clock and draws of §7.2, stalls | after each idle cycle, a draw against the interval; on mouse-over |
| deadline effects | the grid, late first frames, the reaction's lost frame 0, the idle step's catch-up | none |

The engine-side facts agree (melee ignores the attack time, as VCMI's comment says); VCMI's
timing is an approximation where the executable is exact.

### 7.6 The original's state (the harness)

`capture/h3mem.py` reads the running game's memory through `/proc/<pid>/mem` (Wine maps the
executable at its image base): the clocks, and per stack its creature, hex, group, frame,
facing, offsets and times (`structs.toml`). It writes nothing. `h3drive.py sequence` steps the
clock 1 ms at a time after actions at the frozen clock (`hex:<n>` clicks a hex, a key, a click)
and logs every change; `h3drive.py sequences` replays the recorded ones and fails when a
timeline changes (two runs after a fresh start agreed). For T4 this gives two checks that need no
eye: the viewer's timeline of a record against the original's, and each captured frame against
our render of the **original's poses** read at that moment, which also covers fidgets and stalls
the viewer's own generator cannot reproduce.

### 7.7 Open

- Not read: the teleporting move, the spells' magic animations (which, where, when: M3 T7a), the
  arrow towers' and the catapult's shots, the sounds' timing. (Read since: the mouse over a
  stack, §7.13; the heroes, §7.14; the First Aid Tent and `PlayMagicAnimation` over one stack,
  §7.15.)
- Not in a sequence yet: Haste and Slow (M3 T7b, with the spells), a ray up or down (the rays of
  `shots` are straight; the ray itself does not depend on the group). Recorded since: a strike's
  and a shot's turns (§7.10), a shot up (`behind`) and down, a death by a shot, a defending
  stack's reaction to a shot and the rays (`shots`, §7.12).

### 7.8 The viewer's timeline (M2 T4a)

`h3_view::anim` plays a record's commands on the original's clock: each `Events` message is one
command; its moves, strikes and shots become frames, each drawn at the clock's time and shown at
the deadline (§7.1), the deadline then moving on by max(the frame's time, the lateness). Its
`Timeline::lines` is the harness's format; `h3-view sequences` plays each recorded sequence's
scenario in-process with its engine commands (`[[sequence.command]]` in `sequences.toml`) and
compares: **all three as recorded, every line** (walk 43, shot 31, melee 82 changes; the melee's
lines up to its recording's 6 000 ms); speeds 1 and 2 give the walk's 62/35 and 40/22 ms. The
unit tests replay the three from their events and the creatures' numbers, so CI checks them
without the original files. `h3-view render <record> --at <ms>` draws the poses then.

- **Read for it** (`shot_animation` 0x43F2C0 from its start to 0x43F409): the shot aims at the
  target's anchor, x ± 22 for a two-hex target by its facing (0x43F323), y minus (267 − the top
  edge of frame 0 of its standing group) / 2, truncated (0x43F338: the stack's +0x16C, Q43); the
  start is the shooter's anchor plus its missile offset (x subtracted facing left), first the
  straight one for the angle, then the chosen group's (0x469880); the projectile's steps
  trunc((d + 20) / 40) over the distance d between them. The Marksmen at hex 35 and the Goblins
  (top 200) at 49 give 14 steps, as recorded.
- ***Provisional*** (the M2 review list): the loop runs at least A + 1 iterations (no reaction
  to wait for: the striker still stands at the end); a later command without a time starts one
  tick after the previous one's last frame shows, or at once after one that drew nothing (the
  melee's computer acts at 1 918 = 1 818 + 100; Defend and Wait, §7.4); the strike's group by
  the rows of the striker and its first target (up, straight, down); the flight's cycles with
  D / 2 truncated; kills counted from `Struck` only (death: the count reaches 0), a dying stack
  drawn among the corpses (pass 1). **Confirmed by the frame check (T4b-1, §7.9):** a walk's
  frame f drawn f × Δx / n and f × Δy / n along the step. **Corrected there:** the shooter's
  strike loop and the flight's points (§7.9).
- **Built in T4b-2a (§7.10):** turns, the draw layers, fidgets from the viewer's own generator,
  a flight's two standing frames (which replaced the next command's default tick).
- **Not yet (T4b-2b and later):** the spells' sprites; Haste and Slow (the walk time), rays,
  teleports, magic animations, deaths by spells, created stacks; the difference §7.9 lists.

### 7.9 The frame check (M2 T4b-1)

Every distinct battlefield the original shows in a recorded sequence against the viewer's own
frame at the same time: `h3drive.py frames` (runbook `clients/capture/README.md`). The harness
steps the clock 1 ms at a time and keeps each screen whose battlefield (rows 0–555) changed, with
its time and the game's state (`frames.jsonl`, the screens as `frames/t<ms>.npz`, local only);
`h3-view sequences <id> --frames <dir>` plays the sequence's record through `h3_view::anim` and
draws the moment shown at each kept time (`ours/`, and what it drew in `ours.txt`); the diff masks
the panel, the heroes' corners and the count boxes where the drawn poses show them (a stack in
group 1 or 2, §4.3; every living stack takes its hexes for the box's place; by the pose's facing
since M3 T1a, `m` in `ours.txt`). A screen the harness's actions replace within its millisecond
(a wait or a defend passes the turn at once, `t<ms>b`) is compared, not the one before it. The only thing taken
from the original is a fidget (a stack the game's memory shows in group 1): the viewer has none of
its own yet (§7.2). Each sequence's counts per frame are recorded
(`situations/timelines/<id>-frames.txt`: `<ms> <differing> <redraw artifacts among them>`, the
artifacts since M3 T1c, §7.11); the command fails when one changes.

**Result** (2026-10-13): walk 42 frames and shot 50 all exact; melee 68 of 75 exact, the 7 others
listed below. What the check settled:

- **The projectile** (`shot_animation` 0x43F5D6 to 0x43F826, read): its DEF's group 0, the frame
  `shot_angle_group` picks (0x469957: the count of leading midpoints between consecutive angles of
  the creature's table that are at least the shot's angle, the angle from the straight start
  point), drawn with the plain drawer, runs skipped (`def_draw_plain` 0x47BCF0, as the
  obstacles), mirrored when the target lies left of the start, over the screen as it stands (a
  save-under buffer restores it after each step; nothing else is redrawn meanwhile). Step k's
  canvas: the start point less half the DEF's size, plus k × trunc(Δ / steps) (`idiv`), shown from
  the climax frame's time plus k × trunc(33 F); erased after the last. The Marksmen's arrow
  (`PLCBOWX.DEF`, 30 × 30, nine frames) at all 14 steps of both shots: exact.
- **The shooter after its climax:** `strike_animation` (0x468A3D) keeps a stack still in its
  shot group in it and advances each stack's frame before drawing it, so the shooter shows frames
  c to n − 1 in the loop and stands after (T4a had frames c − 1 and c: the second iteration
  showed frame 7 where the original stands; the lines were the same). The loop's length: §7.4
  (corrected in T5b: the shooter is no striker).
- **The flight's points** (0x4B4BA3 to 0x4B4D5D, read): from the start hex's anchor (the hex
  record's +0x1C4, +0x1C6) the point moves on **before** each frame by f32((f32 Δ / f32 k) / f32 n)
  and is kept as an f32, the stack drawn at its truncation (`crt_ftol`): the x87's single
  operations on f32 operands, stored with `fstp dword`, round as one f32 operation, so the viewer
  emulates them exactly in fixed point (no float in the pure crates, ADR-0003: `f32_round` in
  `h3_view::anim`). The Angels' 21 frames over 352 pixels end 351 pixels on, not 352 (T4a: j / kn
  of the line, one frame behind).
- **The outline and the idle steps** (§7.1, §4.2): the outline shows the opening's active stack
  until the command's first frame shows, none while a command plays; when a player's turn starts
  after an animation, its stack's outline at once, and the idle steps held up by the animation run
  twice at that moment (the walk: three steps at 2 592, the outline in entry 0x65's colour, the
  fourth at 2 692); on the computer's turn the loop does not tick until it acts one tick later,
  when one idle step runs and outlines its stack until its first frame shows (the melee's Wolf
  Riders at 1 918, entry 0x66). The viewer cannot tell a player's command from the computer's in a
  record: a command without a time after the first counts as the computer's (*provisional*).
- **The draw layers** read from memory (+0x8, `h3mem.py`): 4 throughout a straight walk or flight;
  the melee's striker 6 and the struck stack 5 during the strike, both 5 during the retaliation,
  4 again after each (drawn since T4b-2a, §7.10).
- **Memory against the screen:** after an animation the game processes a turn's start in the same
  millisecond after the harness's 20 clock reads, so a state read then can lag the screen by one
  step (the walk's idle counter read 1 at 2 592 while the screen shows the third step's colour).
  The check therefore draws the viewer's timeline, not the poses read from memory.

**Listed differences** (T4b-2): the melee at 3 504 to 3 966 ms (9 to 327 pixels), while the Wolf
Riders walk past the Angels: dark pixels at the Angels' outline edges (sword tip, robe), and at
3 966 the Angels' count box at its usual place under the Wolf Riders' legs where the viewer's
poses (the walk's end on hex 45) put it beside them. Read in M3 T1a: the original's redraws by
dirty rectangles, §7.11 (shadows drawn again, the saved screen's count box); not copied, the
user's decision; classified since M3 T1c (§7.11): 6 of the 7 frames are artifacts only, and at
3 966 the 231 other pixels are the stale count box (listed until M3 T5 draws the boxes). Not
stale (M3 T5a, §4.3): the Wolf Riders, still in their walk (+0x30 set), leave the hex ahead of
the Angels free, so the original draws the Angels' box beside them; the viewer now does too, and
the frame is 96 artifacts only.

### 7.10 A whole battle: turns, layers, fidgets (M2 T4b-2a)

`h3-view play <scenario | record>` draws a battle frame by frame headless, M2's headless exit: a
scenario is fought to its end by a bot of the viewer's own in both seats (`bin/h3-view/bot.rs`:
it shoots, else strikes, else walks toward the nearest enemy, the first such command the engine
lists; the engine plays the battle, the protocol's session replays its commands for a
spectator's record), its commands are played at the original's timing and every distinct frame
is drawn (`Timeline::changes`: each pose's, projectile step's, outline's and idle step's time;
`--png` writes them). `clients/scenarios/battle.toml` (walkers, two-hex stacks, fliers and
shooters on both sides): 24 commands, 82 s, about 1 100 frames drawn in 3 s; `--to` past its end
lets it idle. Read in `Heroes3.exe` for it:

- **Facing and position.** A stack faces right (+0x44 = 1) or left; the setup faces the attacker
  right. Turning around flips it and moves a two-hex stack's position (+0x38) onto its other hex:
  + 1 for the attacker, − 1 for the defender (0x446720), so the stack keeps its hexes. The engine
  keeps a two-hex stack's position by its side (BAT-GEO-002): the viewer adds that shift while a
  stack is turned, and the timeline's lines show the executable's position. Between commands
  every living stack faces as its side does (each turn is undone at its action's end).
- **A walk** (`walk_step` 0x43F0B0): before each step the stack faces it, right for a step up and
  right, right or down and right, left otherwise (0x43F0E5); the first step's turn is animated,
  a later one only flips (its flag argument is the first step's). Its draw layer while it steps:
  3 up, 7 down, 4 sideways (0x43F15E), set before group 20; 4 again on the new hex. A plain move
  turns back at its end, animated, to the facing it started with (`stack_walk` 0x4423A2, its flag
  set by the move action 0x479106); a move that strikes does not (0x478971).
- **A flight** (0x4B4A40) turns first, animated, when its landing hex's column lies the other way
  from its facing; ends with group 21 and one frame standing (0x4B4E66); a plain move then turns
  back (`stack_fly` 0x4B49F5) and draws one more frame standing (0x4B4A15). That second frame is
  what made the melee's computer act "one tick after" the flight in T4a's model: with it, a
  later command without a time starts as soon as the last frame shows (*provisional*; the melee
  sequence unchanged).
- **A melee** (`move_and_attack` 0x478900, `strike_action` 0x445A60, `melee_attack` 0x441CB0):
  after the move the striker turns, animated, if the target is not on its front side (0x445BCE:
  the three hexes ahead of a one-hex stack, any neighbour of a two-hex stack's front hex; the
  directions through 0x523DF0, the neighbours from the table at +0x13468); it takes draw layer 6
  (0x441CC1); the target turns likewise to face it (0x4458B0, 0x441D79). Strike, retaliation,
  second strike; the target turns back if it lives and turned (0x441F36), then, after the
  Harpies' return, the striker (0x478A57). The next turn, and its outline, starts after these
  turn-backs (T5c, the reviewer: the viewer had started it before them).
- **A shot** (`shot_action` 0x440160): the shooter turns, animated, when the target's position
  lies right of its other hex while it faces left, or left while it faces right (by the hexes'
  x); its shots; it turns back if it turned (0x4402F2). A two-hex target is aimed at by its
  facing (0x43F323).
- **Draw layers in the scene.** Each pose carries its layer, the pass of its position's row that
  draws it (§4.6): the striker 6 in a command's first melee strike, 5 in a later one (the
  retaliation, a second strike); the struck stacks 5 (the harness's reading, §7.9; their writes
  were not found), a shot's and a ray's too, the shooter staying in 4 (observed in `behind` and
  `shots`, M3 T5b-2a: the viewer had kept a shot's target in 4, under the Centaurs beside the
  hit Pegasi). A layer change without a frame of its own
  (the striker's 6 before the target turns, back to 4 after a strike) shows with the next frame
  that does not draw that stack, or with the first idle step after its command if one comes
  first (*provisional*; T5c, the reviewer: a corpse lay in pass 5 through a player's wait, and a
  survivor's standing pose came back over its fidget with the next command). A dying stack
  plays its death in its layer's pass and lies among the corpses (pass 1) from the next frame
  (*provisional*: the executable moves it to the corpses' list when its death ends).
- **A flier in flight is in no row** (T5a; the user saw the Angels pass under the Wolf Riders'
  corpse and the Orcs flying down four rows). `stack_flight_animation` sets +0x30 (0x4B4BD8)
  and takes the stack off its hex records before the cycles (0x468730, from 0x4B4BFE); each frame
  restores the last frame's rectangle from a saved buffer (0x44E2B0) and draws the flier by
  itself (`draw_battle_stack` at 0x4B4D60), over everything; it stands on the landing hex again
  before group 21 (0x4B4E24). The viewer had kept it in its start row's pass; the cycles' poses
  now take layer 8 (`anim::FLYING`), which the scene draws after every row. Group 20 (before the
  removal) and 21 (after landing) stay in the row passes. Confirmed against the original's
  frames in M3 T1b-2 (sequence `fly`, §7.13: the Dragons' and the Angels' cycles across rows,
  over the stacks of the rows below, all exact); the earlier recorded flight (sequence `melee`)
  stayed in its row, where both draw alike.
- **Fidgets** (`anim::fidgeting`, §7.2): the executable's clocks and stalls on a generator of the
  viewer's own (the C library's `rand()` sequence, seeded by the battle's seed); the reference
  checks (`sequences`) take none. A stack an action drew stops fidgeting; a bystander's fidget
  resumes after it. A bots' battle idles one tick between commands, so its fidgets show only
  after its end; a record with times (a person thinking) shows them as the original does.
- **Confirmed in the original** (sequence `turn`, `seq-turn.toml`): the walk sequence's
  Champions (two hexes, 35 and 36) walk one hex up and to the left: group 7 on 35, the flip onto
  36, group 10, standing, group 20, the step's cycle in layer 3, group 21 on 19, standing, group
  9, the flip onto 18, group 8, standing. The viewer's timeline as recorded (33 changes), all 39
  distinct frames exact. A strike's or a shot's turn cannot be set up from an opening (every
  target stands on the attacker's front side; on column 15 a target turns only for a one-hex
  attacker landing up and to its right, and no one-hex flier of the base game is fast enough;
  Archangels are two hexes wide): M3 T1a set them up in a battle in progress (below).
- **Confirmed in a battle in progress** (M3 T1a; the harness plays both sides in hot seat,
  runbook "A battle in progress"). `climb` (`seq-turn.toml`): the Champions walk from 35 to 1, up
  and right (35 to 19, no turn), then up and left with a flip only (onto 20), group 21 on 2, and
  turn back animated (group 9, the flip onto 1, group 8); the original's path is the engine's.
  `behind` (`seq-behind.toml`, two rounds, eight commands): a wait, walks, a defend, the Pegasi's
  flight behind the Centaurs (its standing frame after landing), then the Centaurs strike the
  Pegasi standing: the striker turns (group 7, the flip onto 42, group 10, standing), the target
  turns (9, the flip onto 39, 8, standing), strike, reactions, retaliation, the target turns
  back, then the striker; the Marksmen shoot the Pegasi, who lie left of them: they turn (7, 10),
  shoot twice, turn back (9, 8). Every rule as T4b-2a read it: both timelines as recorded (34 and
  203 changes, the six sequences as recorded). Frames: `climb` 43 of 43 exact; `behind` 164 of
  194, the 30 others listed (6 844 to 12 444 ms, 46 to 225 pixels, each where the Centaurs and the
  Pegasi stand side by side while one turns or strikes): the redraws of §7.11, not copied,
  every pixel classified as a redraw artifact since M3 T1c (§7.11).
- **Not drawn yet** (in a bots' battle on `battle.toml` nothing of it occurs): spells and their
  animations, morale's and luck's animations, created stacks (summoned, cloned, risen), war
  machines, heroes and count boxes (M3); a stack the record names that its first snapshot lacks
  is not drawn; its strike or shot plays its targets' reactions alone, their kills taken (T5c).


### 7.11 The redraws: dirty rectangles (M3 T1a)

The original never composes a whole battlefield per frame, as the viewer does: it redraws a
rectangle and shows only that. Read for the walk-by difference (§7.9); every address in the map.

- **Three pictures.** The back buffer (the window manager's +0x40 picture), where everything is
  drawn; the screen, to which `H3Redraw` (0x602BD0) copies a rectangle; and the battle's own
  buffer +0x53B0: the composed background (terrain, the absolute obstacle, siege parts;
  `compose_battle_background` 0x493CF0, which sets +0x53B8) or, during a stack's animation, the
  saved screen.
- **The backdrop** (`stack_animation_backdrop` 0x446830, before a turn and a walk's first step):
  measures the stack into the dirty rectangle +0x13D38 (x0, y0, x1, y1; started from the empty
  rectangle 0x6AACE8, (799, 599, 0, 0)), redraws the whole battle without it (+0x31 set) into the
  back buffer, unseen, copies that into +0x53B0 and clears +0x53B8: the saved screen holds every
  other stack as it stands then, count boxes included.
- **A stack's frame** (`stack_play_group` 0x446940): restores the previous frame's rectangle from
  +0x53B0 into the back buffer; measures the stack's new frame with its count box (the flags
  +0x13D2C and +0x13D34: measured, not drawn; `battle_item_rect` 0x495F50 gives a DEF frame's
  content box, its margins and size in the DEF, mirrored by the facing, clipped to the
  battlefield 0x694F18, (0, 0, 799, 555)); widens it 17 pixels left and right; takes its union
  with the previous rectangle as the dirty rectangle; redraws the battle with the clip flag
  +0x13D30 and no background (`battle_redraw` 0x494440 with its fifth argument clear): each item
  whose rectangle meets the dirty rectangle is drawn **whole**, over whatever the back buffer
  holds (`battle_draw_sprite` 0x4951B0 tests, never clips); then shows the dirty rectangle. The
  next frame restores this frame's widened rectangle.
- **A strike's iterations and the idle step** (`strike_animation` 0x46906E, `battle_idle_step`
  0x49680B): `battle_redraw` with its second argument (`RefreshCreatures` 0x495BF0 marks the
  dirty rectangle, not read) and its fifth: the composed background restored inside the dirty
  rectangle (recomposed whole first when +0x53B8 is clear, as after a backdrop), each item meeting
  it drawn whole, the rectangle shown.
- **What it leaves on the screen:** (1) an item drawn whole where its earlier image was not
  restored is drawn over itself, and its shadow pixels darken again (§3.3: shadows compound);
  (2) under a moving stack's rectangle the saved screen comes back as it was at the backdrop, so
  a count box that has moved since shows at its old place; (3) the screen changes only inside
  each redraw's rectangle.
- **Checked against the frames.** The melee's walk-by (§7.9: 3 504 to 3 966 ms, 9 to 327
  pixels): every differing pixel lies on the Angels' edge (x 437–505, y 180–208), every one but
  the last frame's is darker in the original, and at 3 504 and 3 966 each is the viewer's pixel
  with one more shadow (border or body, §3.3); they grow frame by frame as the Wolf Riders'
  rectangle sweeps over the Angels; at 3 966 the rest (211) is the Angels' count box as the
  saved screen holds it (the hex ahead free). `behind` (M3 T1a) shows both effects wherever the
  Centaurs and the Pegasi turn and strike side by side: listed in §7.10.
- **Not copied** (the user, 2026-10-14; epic "Decisions", the ADR-0020 amendment draft): the
  viewer draws each moment whole on its background (`h3_render`) in every profile, so it matches
  the original wherever no redraw meets another stack; the artifacts are not reproduced.
- **Classified** (M3 T1c, *provisional*; `h3cap.redraws`): a differing pixel is a redraw
  artifact when the original's is the viewer's at the same moment with one to five more shadows
  (body or border in any order, §3.3; four until M3 T3a, §7.14), or one the viewer drew there at an earlier moment of the
  sequence with at least one more (a rectangle restored from the saved screen or left as it was).
  Any other difference is no artifact; the frame check records both counts and fails when
  either changes, and the diff PNG marks artifacts yellow, other differences red. On the six
  sequences captured again (T1a's frames were lost with its sandbox; every count as recorded):
  `melee` 6 of its 7 frames artifacts only (9 to 198 pixels, up to four shadows); at 3 966, 96
  artifacts and the stale count box, 231 pixels in its 30 × 11 rectangle at (476, 197), listed
  until M3 T5 draws the boxes (M3 T5a: no stale box but the walker's +0x30, §4.3; the frame
  artifacts only) (since M3 T4a draws them in the viewer's earlier frames, 136
  artifacts: the box's 40 pixels under the Wolf Riders' shadow are an earlier moment's, shadowed;
  191 left); `behind` all 30 frames artifacts only, 3 951 pixels: 3 878 at the
  same moment (up to four shadows), 73 an earlier moment's (mostly the frame before; none with no
  more shadow, which a timing slip of the viewer's would give). The rule's strictness: the
  viewer's frames shifted one pixel differ in 2.4 to 11 million pixels over `walk`, `shot` and
  `behind`, of which 0.2 % fit it; a misplacement still fails. Since the heroes are drawn (M3
  T3a) the left hero's shadow under a walker's redrawn rectangles shows such artifacts too, five
  shadows deep at most (`turn`, `climb`: black pixels).
- **A projectile over the obstacles** (M3 T5b-2a; `behind` run on to the Marksmen's shots, not
  read). In the first shot after the Marksmen's turn the arrow climbs over the rocks (obstacles
  19 and 20): the rectangles its earlier steps leave show the terrain alone, a hole growing in
  the rocks along its path (13 636 to 13 735 ms, 1 036 to 1 913 pixels, none an artifact by the
  rule above), until the strike's first iteration redraws them (13 768). In the second shot,
  the same path without a turn before it, the rocks stay as the viewer draws them. Each step of
  `shot_animation` (0x43F2C0) restores what its save-under took from the back buffer; that the
  turn left the composed background there (the "three pictures" above) is the likely cause, not
  read.
  Listed, not classified, not copied.

### 7.12 The ray (M3 T1b-1)

The Arch Magi (35), the Beholders (74) and the Evil Eyes (75) carry creature flag 0x800 (the ray
shooters, `installation.md` §5): their shot draws an energy ray instead of their projectile (the
user, at M2 T5c; the Magi (34) shoot their projectile). Read in `shot_animation` and H3API's
DrawRay (`draw_ray` 0x5A5C20 with `ray_init`, `ray_step`, `ray_aim`); every address in the map.

- **Its place in the shot.** The shot group's frames up to the climax as for any shot (§7.4);
  then the ray shooter (0x43F51D) waits trunc(115 F) outside the deadline (0x43F52F) and calls
  `draw_ray` from the missile point to the target point (those of the projectile, §7.9) with no
  branches and no randomness, thickness 5 at the start and 4 at the target, colour by creature
  (the switch 0x43F55B: 302 for the Arch Magi, 304 for 74, 75, 120 and 121, any other 0),
  trunc(d / 15) + 15 steps per tick (0x43F578; d the shot's distance as for the projectile) and a
  delay of 10 (0x43F58B); then returns, no projectile.
- **The ticks.** The ray keeps its own clock, not the frame deadline: the first tick shows at
  trunc(10 F) after the call, each next one trunc(10 F) later (the clock read once, each wait
  `wait_until` on a deadline moved on by max(delay, lateness)). Each tick moves the ray's head
  its steps of one pixel along its aim, the head an f32 pair plus the sine and cosine of the
  f32 angle at each step (the x87 at 53 bits, then stored as f32; `crt_sin`, `crt_cos`), kept
  to the battlefield (0–799, 0–555); at every pixel newly entered it draws a cross-section of
  the ray's thickness straight into the back buffer, down the column when the ray runs mostly
  sideways (|dx| > |dy| at the start), else along the row; then shows the tick's bounding box,
  widened by half the thickness. Between ticks it aims again from the pixel reached:
  atan2(dx, dy) as an f32, and the thickness 5 + trunc(−2 p), p the f32 share of the start's
  distance gone (5 to halfway, then 4). It ends at the pixel within 2 of the target (city-block
  distance), or where it moves away after coming within 15. Each tick also draws a random number
  and drops it (the rules' generator: the viewer has none to keep in step).
- **The colours** (`ray_colours` 0x68833C): five R, G, B entries from the edge inward, each
  scaled into the screen's masks (value × mask / 255, kept to the mask; 0x68C860), so not the
  palette's truncation (§3.2): the Arch Magi (152, 188, 24), (124, 216, 124), (36, 180, 36), …
  green, darker inward; the Beholders and Evil Eyes greys from 192 at the edge to 224 in the
  middle. A cross-section of 5 takes entries 0, 1, 2, 1, 0; of 4, 0, 1, 1, 0.
- **After the ray** (0x5A60CA): the battle redrawn and shown at once without the ray, the frame
  deadline neither waited for nor moved, so the last tick shows for no time; then, for a
  creature with a climax frame (+0x150; all three), the rest of its shot group from the frame
  shown on, each trunc(trunc(A / n) × F) on the deadline (n the group's frames), the last drawn
  but not shown (0x5A6136 to 0x5A61C9). Then `strike_animation` with the reactions as after a
  projectile (§7.4), the shooter's group over.
- **The viewer** (`h3_view::ray`, `anim`'s `Beam`): the same steps in integers (ADR-0003 §3):
  the f32 and 53-bit roundings exact, the sine, cosine and arc tangent by series to about
  2^-58 (they could round differently only where an exact value lies that close to a rounding
  boundary); the strokes drawn over everything with their colours scaled into the masks
  (`PixelFormat::scale`). `h3-view harvest` writes the colours into the asset tables (`ray`).
- **Checked against the frames** (sequence `shots`, M3 T1b-1): three rays, the Arch Magi's
  twice (137 to 49, rising 6 rows, and 137 to 151 along the row) and the Beholders' (49 to 35),
  each 11 ticks from trunc(115 F) after the climax frame shows (655, 4 086, 5 293 ms), the last
  tick shown for no time; the shot group's rest after them (the Arch Magi's frames 9 to 12, the
  Beholders' 5 and 6, the last drawn and never shown) and the reactions after that. The viewer's
  timeline as recorded on the first try, and all 139 frames of the sequence equal the original's
  outside the masks: every ray pixel of every tick, its colours included (the Arch Magi's
  0x95C2, 0x7EAF, 0x2584).
- **Not read:** the Enchanters' shot (0x59FDE0). Chain Lightning's call (0x5A64EF), its draws
  and colour 305 were read at M3 T7c-3 (§7.16).

### 7.13 Flights, the breath and the mouse (M3 T1b-2)

Recordings that check the rules M2 had only from the code reading, in a battle in progress in hot
seat (`seq-fly.toml`; the runbook's "A battle in progress").

- **Sequence `fly`.** The Black Dragons fly from hex 49 to 54 (row 2 to row 3, five cycles over
  about 530 pixels) and breathe down and to the left through the Pikemen on 69 into the Gnolls on
  86; the Pikemen strike back. Then the Angels fly from hex 1 to 155 (row 0 to row 9) over the
  Dragons and the stacks of rows 2 to 8, whom the rows drawn after theirs would cover if a flier
  stayed in its row. The viewer's timeline as recorded on the first try (110 changes; the breath
  in group 19, both stacks' hits at its second-to-last frame). Frames: 102, 91 exact, the other
  11 redraw artifacts only (§7.11: shadows drawn again where the landing Dragons and Angels meet
  the stacks beside them, 68 to 119 pixels each, at 2 000 to 2 700 and 7 418 to 7 718 ms).
  Every frame of both flights' cycles is exact: **a flier in flight is drawn over every row**
  (§7.10, layer 8), and **the breath's groups and its detection by place** (§7.4) hold. The
  recording tells the two models apart: a viewer that kept the flier in its start row's pass
  differs at 23 of these frames, by up to 2 480 pixels (the Angels under the stacks below row 0).
- **Where a two-hex striker stands** (the harness's clicks, `cursor_attack_from_hex` 0x474A00;
  not read further): the cursor's slot inside the target's hex picks the hexes. With the cursor
  up and to the right of the Pikemen's centre (+14, −13) the defending Dragons stand on 53 and 54
  (their head on 53, the breath's line through 69 into 86); at (+9, −16) on 52 and 53, the
  strike then down and to the right from 52, which breathes on nothing; at (+19, −9) on 70 and
  71, beside the target.
- **The mouse over a stack** (`battle_mouse_stack` 0x477C20, read; `battle_buttons` calls it
  when the mouse's hex changes, 0x47570E). Nothing happens while the battle is hidden or +0x132F8
  is set, nor when the hex's stack is the one already marked (+0x132D0, its position: so not
  between a two-hex stack's hexes). Otherwise the marked stack is unmarked and redrawn, and a
  stack on the hex is marked: it takes the mouse's outline (entry 0x70, the cyan ramp, §4.2) and,
  if its DEF has group 1 and it does not show it, **starts group 1 at frame 0** at once (else, not
  standing, it stands: group 2, frame 0). The idle step then plays the group as a fidget (§7.2:
  one frame per step, its stalls, the fidget clock drawn again at its end), so a mouse over the
  stacks changes the original's later rolls, as fidgets do. Sequence `hover` (the `fly` map's
  opening): the mouse enters the Pikemen at 0 ms (group 1, frame 0; frames 1 at 100 and 2 at 200
  on the idle steps), leaves at 150 and enters again at 200, while they still show group 1: they
  stand. Its frames show the cyan outline. The viewer draws no mouse in a record, so the
  sequence has no engine command: `h3-view sequences` and the frame check skip it; the window's
  mouse-over waits for the battle interface (M3 T6, `QUEUE.md`'s inbox).
- **The capture set by time** (§6.1): `h3drive.py set` reads each frame's stacks from the
  original's memory beside its dump and `h3-view render --frames` draws ours at that time, the
  outline's pulse on the original's clock (`Timeline::pulse`) and a fidget in the original's
  poses (as the frame check, §7.9). All 17 frames of the nine situations exact outside the masks
  (the outline's colour had differed by a step, 412 pixels a frame, and the grass Pikemen's
  fidget by up to 2 036).

### 7.14 The heroes and their flags (M3 T3a)

Read in `battle_idle_step` (0x4960D0), `stack_death`, `round_start` and the battle's setup; the
viewer's model is `h3_view::hero`.

- **The flags** wave on the idle steps only (none runs while a command plays, §7.9): each step
  shows group 0's next frame (0x496164), the left flag from frame 0 and the right one from 3
  (0x463F29), so the opening (one step on, §6.2) shows frames 1 and 4.
- **A hero** stands in group 0, frame 0. At the first idle step more than 4 500 ms after its idle
  clock (+0x53FC, 0x4965AB) it starts its idle motion, group 1 at frame 0; each later step shows
  the next frame; after the last it stands again and its clock starts then (0x4967AE). The clock
  starts with the battle (`battle_start_idle_clocks` 0x479F43) and after every action, as the
  stacks' fidget clocks do (§7.2). Unlike the stacks' fidgets nothing random is involved.
- **Defeat and victory.** When a stack dies (not a clone), a draw of 1 to 100 below 60 (from the
  rules' generator) asks its own side's hero for defeat (+0x53DC, 0x4441F5), else a draw below 80
  asks the other hero for victory (+0x53DE, 0x444221). A standing hero takes a request at the
  next idle step, defeat first: if the side whose turn it is has a human player (0x4CE970: both
  human flags of its record), the group plays (if the DEF has it) and counts as played this
  round (+0x53E0, +0x53E2); if not, both requests lapse. `round_start` clears requests and the
  once (0x4760DF). Berserk's and Chain Lightning's strikes on a stack of their own side also ask
  for defeat (0x422391, 0x4457FC, 0x5A6545; not read further). VCMI differs: it compares the
  kills of each attack (`BattleInterface::stacksAreAttacked`) and has no idle motion; both its
  flags start at frame 0.
- **The viewer** draws the flags and the idle motion as read; defeat and victory from a generator
  of its own when it plays a record (the stacks' fidgets' generator, §7.2), none in a reference
  check, which takes the original's heroes from its memory (`h3mem.battle_heroes`) as it takes
  its fidgets.
- **Checked:** the capture set's grass situation from 4 400 to 5 100 ms (both heroes' idle
  motion from 4 600 ms, a frame a step, standing at 5 000; the flags all along), exact. The
  set's clock now steps at most 100 ms at a time: a longer step runs one idle step (0x496820), so
  a frame far on had shown too few steps. The frame check without the `heroes` mask, every
  sequence captured again: `shot` and `shots` as recorded; in the other six only redraw
  artifacts were added, all in the left hero's shadow where a stack beside it is redrawn (2 to
  210 pixels a frame; five shadows deep at most, so the classifier's limit is five, §7.11); the
  flags exact throughout, so they stand still while a command plays. No hero reacted to a death
  in these recordings (the reactions are read, not yet seen).

### 7.15 The war machines and the magic animation (M3 T3b)

Read in `battle_war_machine_stacks` (0x463600), `stack_load_graphics`, `first_aid_tent_action`
(0x478B90), `play_magic_animation` (0x496840, H3API's PlayMagicAnimation), `draw_battle_stack`'s
tail (0x43E795) and the drawers behind it; recorded in the capture set's `war-machines` and
`smoke` and in the sequence `machines` (hot seat: two Ballistas shoot, the tent heals).

- **Standing.** War machines are stacks drawn as every stack (§4.2) at the engine's places
  (BAT-WAR-001; the capture shows the same hexes): the Ballista (`SMBAL.DEF`: groups 2, 3, 4, 5,
  14-16), the First Aid Tent (`SMTENT.DEF`: 2, 3, 4, 5) and the Ammo Cart (`SMCART.DEF`: 2, 3, 4,
  5). No count box (creature flag 0x40, §4.3; the capture shows none), no fidget (no group 1),
  and the tent stands in group 2's frame 0 although the group has six. The original's records
  hold each side's army, then its war machines in the setup's order: the Catapult, the Ballista,
  the tent, the Ammo Cart, then the turrets (0x463600); the viewer numbers its timeline's
  records so (the engine's ids follow the kinds: Ballista, Ammo Cart, tent).
- **Their turns.** Without Artillery or First Aid a war machine acts on its own when its turn
  starts (the engine plays that turn inside the command before it, BAT-WAR-002/003). As at the
  computer's turn (§7.9) the battle loop ticks once as it acts: an idle step (the flags, the
  outline's pulse) and the machine outlined until its action's first frame shows; a Ballista's
  shot is a shot (§7.4: its groups 14-16, `SMBALX.DEF`, its climax 5) and clears the active
  stack. Confirmed in `machines`: the attacker's Ballista at 2 221 ms, the defender's at 3 683
  (its first frame drawn a millisecond later), the tent at 5 845, each an idle step.
- **The heal.** The tent does not move and stays the active stack (outlined) through its heal:
  `Regener.wav` starts (category 2, 0x478C46), the log's line (0x472E90), then magic animation 79
  over the patient (0x478D14: 100 ms a frame, without the patient's hit), then the battle waits
  for the sound's end (`wait_for_sample`, at most 4 s). The sound lasts 1 881 ms and the
  animation 2 100 ms at speed 0, so the wait only matters at the faster speeds with sound on (not
  built: the captures play without sound; *provisional*). The engine picks the patient at
  random (BAT-WAR-003), the original's automatic tent the stack with the most lost hit points
  (0x473EA0); a tent or Ballista without anything to do defends there (action 3) and does
  nothing in the engine: both for the battle lane (QUEUE inbox).
- **The magic animation** (`play_magic_animation`): nothing while the battle is hidden; the DEF
  of entry `id` of the table 0x641E08 (83 entries: DEF, a force-feedback effect's name for
  IFC20, 0x4B69F0, and the kind) is loaded into the manager (+0x132E8, its number +0x132EC) and
  the target's byte +0x20 set; with `show_hit` the target first plays its hit group (not read
  further); then each frame of group 0 (+0x132F0) is drawn and shown by a redraw that waits
  trunc(`time` × F) on the deadline (§7.1), and a last redraw that clears the byte erases it.
  In `machines` frame i of the heal is drawn at 5 845 + 100 i and shown 100 ms later, erased at
  7 945, when the next turn starts.
- **Where it is drawn** (`draw_battle_stack`'s part 0 after the stack's sprite and its count
  box, 0x43E795; T3b had it before the box, M3 T5b-2b: the heal lies over the patient's box): x =
  the position's anchor (22 pixels toward the front of a two-hex stack) − W / 2;
  y by the kind's low nibble: 0 the anchor − H (standing on the hex), 1 the anchor − H / 2 −
  h / 2 (centred on the creature), 2 the anchor − H − h (above it), with W, H the DEF's size and h
  the creature's height (+0x16C: 267 less its standing frame's top, 0x43DD62); 3 beside the
  standing frame's front edge and higher kinds (whole-field animations) are not built; halves
  truncated toward zero; mirrored when the stack faces left. It goes through 0x4953B0 (clipped to
  the battlefield) to the drawer 0x47EF60: with the kind's bit 8 encoding 1's literal pixels
  blend half and half with the screen, ((c >> 1) & H) + ((d >> 1) & H), and every run is
  skipped; without it the plain drawer with runs skipped (§3.3). The heal's `sp12_.def` (20
  frames of 95 × 94) is centred and translucent.
- **The viewer** (`h3_view::anim`'s heal, `MagicKey`, the scene's `Draw::Magic`, `h3_render`'s
  `Drawer::Translucent`): the table is harvested into the asset tables' `[battle] magic`, the
  heal's sound as `heal_sound`; the harness reads the magic animation from memory
  (`h3mem`: +0x132EC, +0x132F0 and each stack's +0x20) and its timeline lines name it
  (`<ms> magic <number> f<frame> <side>.<record>`, `<ms> magic -`).
- **Checked:** `war-machines` (all three on both sides, 9 frames) and `smoke` (the M1 smoke
  battle with its heroes and Orrin's Ballista, in hot seat so that the Wolf Riders open idle; 2
  frames) exact; `machines`' timeline as recorded on the first run (after the records' order)
  and its 106 frames exact, the heal's translucent frames and the tent's outline included.

### 7.16 The magic animations (M3 T7a-1)

Read in `play_magic_animation` (whole), `play_magic_animation_at_hex` (0x496A10),
`mass_spell_animation` (0x5A67C0), `raise_stack`'s and Armageddon's drawing (0x5A7740 to
0x5A7867, 0x5A4CED to 0x5A50C6), `strike_animation`'s magic part, `spell_projectile` (0x467DB0),
`cast_spell`'s opening and tail and every handler's drawing, `creature_cast_action` and every
call site of these players (22, 11, 6 and 21), with the spell records' static fields (0x685450:
+4 the sound, +8 the animation, −1 none). Each player was recorded at T7a-2a and T7a-2b
(**Recorded**, below): the times read held; four readings were corrected by the recordings (Chain
Lightning's and Fire Shield's marks, the projectile's angle, a mass reaction's hit running on).

**Which player plays what.** The handler decides, not the animation's place (the asset tables'
`[battle] magic`, §7.15):

1. *Over one stack*, `play_magic_animation` (100 ms a frame unless named): a timed spell's single
   target (its own animation, after the effect; Poison with `show_hit`), Dispel 41 (before its
   effect), Cure 39 (after), Disrupting Ray 14 (after its projectile), Acid Breath 81; a cast
   resisted: `MagicRes.wav`, the log's line, then 78 (`sp09_`, place 3) and the cast ends, its
   own sound unplayed (0x5A1F89); a cast Magic Mirror redirected: Magic Mirror's 3 over the
   mirroring stack before the new target is taken (0x5A0275); Lightning Bolt's (17, 57) bolt
   from the sky 1 (`C11spA0`, place 2) at 10 ms; Chain Lightning's first 37 at 0 ms (each frame
   waits only for the deadline the last redraw set; recorded: drawn but never on the screen). The creatures': good luck 18 over the
   striker or shooter before its strike or shot (`goodluck.82m` started, the log, then the
   animation; 0x4418B3, 0x43F9D1), good morale 20 (`GoodMrle.wav`; the log's line *after* the
   animation, 0x464A84), bad morale 30 and fear 15 with the hit (`BadMrle.wav`, `Fear.wav`; the
   line before; 0x464D10, 0x464F33), Death Stare 80 (spell 79's `Deathstr.wav`, 0x44113F), the
   Rust Dragon's acid 81 (spell 80's sound, 0x44156A), Life Drain 74 over the drinker
   (`DrainLif.wav`, 0x440E83), the Thunderbird's bolt 1 at 10 ms (`LightBlt.wav`, 0x441315),
   Death Blow 73 (`Deathblo.wav`, 0x443B3C, over the stack of its caller's third argument, not
   read which), regeneration 79 (`Regener.wav`, 0x44708D), the tent's 79 (§7.15), the Wraith's
   mana drain 77 (`ManaDrai.wav`, 0x4657BD). With a sample the battle then waits for its end.
   SoD has no bad luck: nothing plays for it.
2. *Over the marked stacks*, `strike_animation` (the stacks whose +0x20 is set): Magic Arrow 64
   and Ice Bolt 46 after their projectiles, Lightning Bolt and the Thunderbird's bolt 49 after
   the bolt from the sky, Implosion 10, Sacrifice's victim 51, a Land Mine's blast 57 over the
   stack that stepped on it (0x59FC6D), Poison's damage at a round's start 67 (0x4472AB). Chain
   Lightning's 38 over every stack it struck (its marks, the manager's +0x547C, copied into each
   stack's +0x20 before the animation, 0x5A65D1 to 0x5A6620), Fire Shield's return 11 over each
   stack whose shield struck back (its +0x20 set at 0x440AA9; 0x440B6F) with the striker's hit
   (both recorded at T7a-2b; T7a-1 had read them unmarked). The Magogs' and Liches' shots name an
   animation but mark no stack: only the reactions play (their explosions are their own, below).
3. *Over several stacks*, `mass_spell_animation` (H3API's AnimateMassSpell; the stacks marked in
   the manager's +0x547C): a timed spell cast on all (Expert or a mass spell; its own
   animation), Dispel and Cure at Expert, Berserk 35 (the stacks in its area), all without
   reactions; Death Ripple 8 and Destroy Undead 29 with them.
4. *A raise*, `raise_stack`: always Resurrection's 50 (`C12spE0`, translucent): Resurrection,
   Animate Dead (its own 4 never plays), Sacrifice's target, the Archangel (0x4485BE), a
   Phoenix's rebirth (0x4695AD).
5. *At a hex*, `play_magic_animation_at_hex` (`id`, hex, `time`, `keep`): the area spells at their
   target hex before the damage (Frost Ring 45, Fireball 53, Inferno 9, Meteor Shower 16;
   0x5A49B8; the reactions follow in `strike_animation` without an animation); the spell
   obstacles' placement, kept until the object is placed and shown (each Quicksand pit 55 with
   `Quiksand.wav`, each Land Mine 47 with `landmine.wav`, each Fire Wall hex 65, Force Field 32
   or, from Advanced, 33); a removal, by the object's vanish animation (battle object +0x14:
   Quicksand 58, Land Mine 59, Force Field 60 or 61, Fire Wall 66; −1 for the battlefield's own
   obstacles) at its anchor: expiring at a round's start (0x4761A9), Dispel at Expert
   (0x5A1767), Remove Obstacle after its own 34 at the hex aimed at (0x5A1C35; an obstacle
   without one dissolves instead, below); the Familiars' channel after a hero's cast (76
   `sp07_B` at the caster's corner, hex 0 for the attacker or 16, with `MagChDrn.wav`, then 75
   `sp07_A` at the other with `MagChFil.wav`, 0x5A2709).
6. *The whole field*: Armageddon 12 (`C06spF0`), tiled (below).
7. *Their own drawers*: the projectiles (Magic Arrow `c20spx0`–`4`, Ice Bolt `c08spw0`–`4`,
   Disrupting Ray `c07spA0`; the Enchanters' shot flies Magic Arrow's, 0x59FDE0), the Magogs'
   fireball (Fireball's 53 and sound, 0x43FA15), the Liches' cloud (Death Cloud's 72 and sound,
   0x43FE07), Bloodlust's and Stone Gaze's glows, Clone's slide, the dissolves.

Not met on any path read: 4, 13 (its DEF is Disrupting Ray's projectile), 38, 43, 44, 52, 62,
63, 82 (`poof.def`). The town's code reads another table through an index offset from
0x641E08's address. Every cast's spell sound (+4) starts asynchronously after the
resistance roll, if any (category 3, 0x5A0331); Curse's is named `Curse`, without `.wav`.

**The players.**

- **`play_magic_animation`** (§7.15, completed): with `show_hit` (ignored when the target already
  shows group 3) the target plays its hit group from frame 0, one frame a redraw at
  trunc(100 F) whatever `time` is, the animation's frame i beside it while it has one; then it
  stands (group 2, frame 0) and, if the hit had at least as many frames, a redraw shows it at
  once. The animation's frames left (from the hit's count on) follow at `time`, then a last
  redraw at `time` erases it. Its place 3 (0x43E906): beside the standing frame's front, x = the
  stack's hex's anchor + (the frame's +0x38 + its width − 196) facing right, the anchor − that −
  W facing left; y as place 1; no two-hex shift.
- **`strike_animation`**: frame i at iteration i while i < n, then its last frame stays until
  the loop ends and every +0x20 is cleared (0x46905A); L = max(n, A + R − 1) (§7.4) with no
  striker (A = 0) for a spell, so the reactions end with the loop.
- **`mass_spell_animation`** (targets, `id`, reactions): loads the DEF itself (+0x132E8,
  +0x132EC; nothing while hidden) and sets every target's +0x20; with reactions each target not
  already in group 3 takes death 5 (no count left) or hit 3 at frame 0 and its death or wince
  sound (`stack_play_sound` 4, 2); L = max(n, the longest such group). Iteration i: the
  reactions advance first (frame 0 never shows; a hit runs on into the standing group 2, a frame
  an iteration, to its last frame, where the stack stays until its next animation; a death on
  its last frame), the animation's frame i is set only while i + 1 < n, so its last frame never
  shows and n − 2 shows twice (both recorded at T7a-2b: a drawing quirk of the original's, for
  the review list), then a redraw at
  trunc(100 F), shown. Then +0x20 cleared, the dead die (`stack_death`), a redraw shown at
  once, the Phoenix check.
- **`raise_stack`**: L = max(n, the death group's frames d); iteration i sets frame i (none
  after n) and plays a lying stack's death group backwards (frame d − 1 − i) until it stands; a
  redraw at trunc(100 F) each, then the dead bit cleared and a redraw.
- **At a hex** (0x496A10): the DEF through `battle_load_magic_animation` (so +0x132EC names it),
  the frame counted locally (+0x132F0 untouched, no +0x20). Its place by the kind's low nibble at
  hex h: 0 x = h's anchor − W / 2, y = h's top + 52 − H (the cell's lower edge); 1 x as 0, y = the
  anchor's y − H / 2 − 37; 4 x = h's left, y as 0; any other at (`keep`, `keep`), which no caller
  uses. Each frame: a redraw restoring the background and waiting `time` on the deadline, not
  shown; the frame over everything on the screen buffer (through 0x47C170, translucent by bit 8,
  not clipped to the battlefield); the battlefield's rectangle shown while the battle screen is
  up (+0x13300). Without `keep` a last redraw at `time`, shown, erases it.
- **Armageddon** (its handler's own loop): L and the reactions as in the mass animation; each
  iteration a redraw at trunc(100 F), not shown, then (while i < n) the frame tiled from (0, 0)
  over the 800 × 556 battle area (ceil(800 / W) columns, ceil(600 / H) rows, each clipped),
  plain; the battlefield shown each time (`battle_show_field` 0x493780).
- **The projectiles** (`spell_projectile`: start, end, the DEFs and their angles): d the distance,
  n = trunc((trunc(d) + 15) / 31) steps of (trunc(Δx / n), trunc(Δy / n)); the DEF whose angle
  (0°, −16°, −34°, −56°, −83°; tables 0x6421EC to 0x642263) is nearest atan(−Δy / |Δx|) in degrees
  (0x467E5D to 0x467F14: Δy over −|Δx|, screen y down), so a target below takes the steeper ones,
  whose pictures point down, one level or above the first, one straight below (Δx = 0) the last;
  mirrored for a target to the left (T7a-1 read the sign the other way; recorded at T7a-2b). A redraw
  neither shown nor waited, then per step: the last position's background restored, group 0's
  next frame (cycling) drawn plain, centred, the union shown clipped to the battlefield, a wait
  until trunc(33 F) after the step's start (outside the deadline, as a shot's, §7.4); at the end
  the background restored and shown (recorded: n + 1 positions 33 ms apart, the first shown with
  the hero's last cast frame; the restore shows with the next animation, 33 ms after the last;
  T7b-1 read the loop: n positions, the first at the start, k = 0 to n − 1, so `arrow`'s 22
  positions are n = 22, not n + 1). From a hero's cast point (below) or a creature's centre
  (`stack_screen_x`, `stack_screen_y`: the position's anchor, a two-hex stack's middle 22 toward
  its front, up by half its height) to the target's centre. The DEF: the angle against the
  midpoints of consecutive table angles as f32, the first midpoint below the angle ends the count
  (0x467F46).
- **The Magogs and the Liches** (after their projectile): the explosion centred on the target
  creature (as place 1), each frame a redraw waiting trunc(50 F) (Magogs) or trunc(100 F)
  (Liches), not shown, the frame through `battle_draw_magic_frame` plain (the Liches' `sp04_` too,
  though the table marks it translucent), shown; a redraw shown at once; then the reactions
  (recorded for the Liches at T7a-2b: all 20 frames, the first two at once after the
  projectile, every changed pixel one of the DEF's colours: plain).
- **Glows** (0x5A122B to 0x5A1311, 0x5A1121 to 0x5A11D1): the stack's flag 0x20000000
  (Bloodlust) or 0x40000000 (Stone Gaze) and its f32 +0x450: Bloodlust 0.0 to 0.9, then 1.0 to
  0.1 (20 redraws), Stone Gaze 1.0 to 0.1 (10), each at trunc(100 F), shown; then the flag
  cleared and a redraw shown at once (how `draw_battle_stack` paints them: not read). A mass
  Bloodlust glows every stack it reached together.
- **Clone** slides from the original's place to its own in 16 redraws at trunc(50 F) (offsets
  +0x100, +0x104; 0x5A6FDE). **Dissolves**: `screen_save_rect` 0x602CC0 saves a rectangle,
  `screen_dissolve` 0x602DC0 blends it into the new picture in 8 steps (weight k / 8), each held
  until its start plus the step (33 ms by default, 75 for a created stack, 0x47A2BD; the blend not
  read): Teleport (the 800 × 556 area, 0x5A1B6C, 0x5A1BFA), a created stack
  (`create_battle_stack`), Remove Obstacle on a battlefield obstacle.
- **The hero's cast** (caster kinds 0 and 2, the latter the artifacts' opening spells): the
  acting side's hero takes group 4 (+0x53E4) and plays frames 0 to c − 1 (c the hero DEF
  record's +0xC, 5 in all 18, `hero_battle_defs`), a redraw at trunc(100 F) each, shown, before
  anything else of the cast (0x5A01CF); then the cast (a redirection, the resistance roll, the
  sound, the handler); then every stack's flags cleared (as at `strike_animation`'s end), the
  rest of group 4 (frames c to 7, 0x5A20B8) at trunc(100 F), and it stands (group 0) in a
  redraw shown at once. Its cast point, where projectiles and rays start: the record's +4, +8
  in the hero's frame, (x − 43, y − 19) on the left, (693 + W − x, y − 19) on the right.
  **Recorded** (T7a-2a, `casts.toml`'s `cast`: Orrin's Slow at the Wolf Riders, Combat Speed 0,
  memory's frames drawn): the hero's frames 0 to 4 at 0, 100, …, 400 ms from the click; Slow's
  animation 19 over the Wolf Riders, its 20 frames at 500 to 2 400 (player 1, 100 ms a frame, no
  hit group), erased at 2 500; frames 5 to 7 at 2 600 to 2 800; standing at 2 900. As read.
- **A creature's cast** (`creature_cast_action`: Faerie Dragon, Archangel, Pit Lord, Master
  Genie, Ogre Mage), after its turn toward the target: the angle atan(Δy / |Δx|) in degrees from
  its centre (its anchor ±22 for two hexes, its anchor's y − h / 2) to the target hex's anchor,
  Δy the target's y less its own (Δx = 0: 90 for Δy ≤ 0, else −90, 0x4483CA); above 25° group
  17, above −25° 18, else 19; a
  missing or empty group 11, 12 or 13 by the same angle (0x44840C to 0x4484BB). Its shot sound
  (+0x17C) starts, its frames play at trunc(trunc(A / n) F) each (A the attack time +0x15C, as a
  shot's, §7.4), shown, and its last frame stays (0x44851C) through the cast (`cast_spell` caster
  kind 1: no hero frames); then the sound's end (0x4486E2) and its turn back. *The sign:* a
  shot's angle is atan(−Δy / |Δx|) (`shot_angle_group`), the cast's not (but for Δx = 0, which
  gives a target above 17, as a shot), and Δy starts at the caster's centre but ends at the
  target's anchor: a level target one hex away takes group 17; VCMI 1.7.5 picks a cast's group by
  the shot's angle (`CastAnimation`), 17 for a target above. **Recorded** (T7a-2a, `casts.toml`'s
  `dragon`: a Faerie Dragon, rear at hex 81, front 82, casting its drawn spell at Swordsmen,
  Combat Speed 0): beside it (83, level) group 17, four rows above (15) 19, two rows below (117)
  17, as read; VCMI's would be 18, 17, 19. By VCMI's names (17 up, 19 down) the original turns
  the caster up toward a target below and down toward one above: a drawing quirk of the
  original's, for the review list. Each cast: frame 0 at the click, held 100 ms (the deadline the
  last redraw set), then frames 1 to 11 at trunc(500 / 12) = 41 ms (its attack time 500); the
  last kept through the spell (here Fireball at 83 and 15, Inferno at 117: at a hex, player 5,
  unseen in memory) until the reactions, 1 382 ms (Fireball) or 1 682 ms (Inferno) after frame 11
  was drawn; struck itself (in the area), it plays its hit (group 3) with the others and stands
  after it; not struck, it stood 600 ms into the reactions (round 3, the Swordsmen's defending
  hit, 11 frames, still playing). The Enchanters (`enchanter_cast`): every able one of the side marked a
  striker in group 15, its shot sound, `strike_animation` without an animation, then the mass cast.
- **Recorded** (T7a-2b, `casts.toml`: hot seat, Combat Speed 0, times from each cast's click;
  memory holds a frame when drawn, the screen shows it a deadline later, the at-hex and
  Armageddon frames when drawn; the screens' boxes in the timelines):
  - `resist`: Slow resisted by the Battle Dwarves: the hero's frames 0 to 4, then the log's line
    and 78 (`sp09_`, 20 frames, 100 ms, the target marked) from 500, erased at 2 500, frames 5 to
    7, standing at 2 900; as read (place 3 beside the stack's front).
  - `arrow`: Magic Arrow's flight from the hero's cast point, then 64 (14 frames) over the marked
    target, its first two frames at once (§7.1), with the target's reaction (a defending stack's
    group 4). The DEF by the corrected angle: to row 0 (2° below) the first, to the Archangel
    beside the hero (about 72°) the last, to row 10 (35°) the third. The hero plays group 4
    whatever the target (no target is above its cast point).
  - `bolt`: Lightning Bolt's 1 (one frame) on the screen 10 ms, then 49 (12 frames) over the
    marked target; no ray. Chain Lightning: 37 drawn but not shown, three rays of 100 ms (§7.12)
    from each target to the next, each replacing the last, the deadline untouched; then 38 (12
    frames) over all four struck stacks, its first two at once.
  - `mass`: Slow at Expert: 19's frames 0 to 18 over the four Swordsmen, 18 twice, 19 never
    (n = 20); Death Ripple: 8's frames 0 to 13, 13 twice (n = 15), the reactions from their frame
    1 with it, on into standing.
  - `raise`: Resurrection and Animate Dead both play 50 (20 frames) over the stack raised, its
    death group backwards from its second-to-last frame, a frame a redraw, standing after frame
    0; the log's line with frame 0.
  - `obstacles`: at a hex each frame 100 ms: a Fire Wall's two hexes 65 (3 frames each) both at
    the target hex's place (the second hex's at the first's: the code's, 0x5A0884, and T7c-2's frames confirm it), the wall
    shown with its last frame and the log's line; Quicksand's four pits 55 (3 frames) each at its
    own place; a Force Field's 32 (3 frames). The Fire Wall gone at round 3's start: 66 at each
    hex's own place (3 frames and an erase each). Armageddon's 12 tiled, all 20 frames at 100 ms,
    with every stack's hit (on into standing).
  - `creatures`: Fire Shield's 11 (10 frames) over the Efreet Sultans with the Archangel's hit,
    before the Sultans' retaliation; the Power Liches' cloud, plain (above).
  - `dragon` again with its screens: Fireball's 53 (13 frames) and Inferno's 9 (16) at their hex,
    each frame 100 ms from the deadline the dragon's last frame set, erased with the reactions;
    memory as recorded at T7a-2a.
- **For the harness** (T7a-2): `h3mem` reads +0x132EC, +0x132F0 and each stack's +0x20
  (§7.15), the heroes' and stacks' groups and frames. That shows players 1 to 4 and the casters,
  not the frames of players 5 to 7 (+0x132EC names the at-hex and Armageddon animations, not
  their frames): their times come from the screens (`screens`, T7a-2b).
- **The spell book** (read at T7a-2a: `spell_book_dialog` 0x59BDF0, its page 0x59C9A0, its
  handler 0x59D040), through which the harness casts: a 620 × 595 dialog at (90, 2), opened in
  battle by the spell-book button or its key C (`hero_book_button` 0x59E900). Twelve spell
  places of 78 × 65, the dialog's (117 + 86 c, 89 + 96 r) on the left page and (333 + 86 c,
  89 + 96 r) on the right (c 0 or 1, r 0 to 2), numbered row by row, the left page first: place
  k's centre on the screen is (246 + 86 (k mod 2) + 216 ⌊k / 6⌋, 123 + 96 ⌊(k mod 6) / 2⌋).
  Each holds the school's mastery ribbon (`SPLEVA`, `SPLEVF`, `SPLEVW`, `SPLEVE.def`, its frame
  the mastery) under the spell's icon (`spells.def`, its frame the spell) and the name, level and
  cost below (`tiny.fnt`, greyed above the hero's mana). The page lists the hero's spells of the
  tab's schools and the bookmark's kind (combat or adventure), sorted by level, then school, then
  name (case-insensitive; 0x59DBE0); the school is the hero's best known of the spell's
  (`hero_best_school` 0x4E51C0: ties to air, fire, earth, water) compared as its bit (air 1, fire
  2, water 4, earth 8). Twelve a page on the all-schools tab; a school's tab shows its emblem
  over places 0 and 1 of its first page. The book keeps its page, tab and kind for the same hero
  (0x684B40, 0x6A34F4): the first opening of a battle's hero shows its combat spells of all
  schools, page 0. A left click on a place (0x59D3E3) closes the book and returns the spell when
  it costs at most the hero's mana (else a message); the battle then asks for its target
  (`enter_player_cast` 0x59EC50). So `cast:<k>[:<hex>]` in the harness (`h3drive.py`: the key
  C, place k, then the target hex if the spell takes one).
- **A Faerie Dragon's cast** needs no key: over any stack but itself that its drawn spell may
  target, the cursor is the cast's (`battle_cursor`, 0x4766FC: kind 20), so a click on the
  target (`hex:<n>`) casts it (action 10); the F key enters the same cast (0x475CD8).
- **The cast's log lines** (read at T7b-1): `battle_log_cast` 0x5A8950 (caster kind, spell,
  target) writes a hero's cast with a target as GENRLTXT row 196 (the hero's name +0x23, the
  spell's record +0x10, the target's name by its number), without one row 197, an opening
  spell's row 197 with the artifact's name; a creature's its own rows (not read). The timed
  handler writes it after the effect, before the animation (0x5A109F). A resisted cast: row 190
  with the name by the number and the verb row 145 for exactly 1, else 144 (0x5A1FB0). Magic
  Arrow's and Ice Bolt's: `battle_log_damage` named by the spell with a number of 1, after their
  `strike_animation` (0x5A0A1F, 0x5A0AEB). The text and its time held in the frame check
  (`cast`, `resist`).
- **Built (T7b-1, `anim`):** a hero's `SpellCast` with its effects: the hero's casting frames (the
  asset tables' `[[hero]]` `cast_frames`), then a resisted cast (`MagicRes` in category 2, the
  line, 78 at place 3) or the spell's sound (category 3 at 127, `play_sound_async` 0x59A590) and
  its handler: a timed spell's line and its animation over the target (player 1), Magic Arrow's
  and Ice Bolt's projectile (player 7) and their animation riding the marked target's loop with
  its reaction (player 2: L = max(n, R + 1), the reaction from frame 0, as after a shot), their
  line at the loop's end; then the rest of the casting group and the hero standing at once.
  Recorded and as recorded (`sequences.toml`): `cast`, `resist`, `arrow`, their timelines with
  the heroes' changes (`lines_with_heroes`; the screens' boxes left out). Found by the frame
  check: the action clears the active stack's outline at the click and the hero standing brings it
  back (the turn goes on); the panel's spell book is disabled from the click (the hero has cast
  this round); the original's memory holds a cast frame a deadline before the screen, so the
  frame check keeps the viewer's own cast frames (`h3-view sequences`); the aimed spell's status
  line (GENRLTXT 28 "Cast %s on %s" over a stack by its number, 27 elsewhere; `enter_player_cast`
  0x59EC50's loop not read) is not forced: `arrow`'s rounds keep the log's new lines, `cast`
  shows it over an empty log (`mouse::track`'s `Input::Aim`, *provisional*); a key pressed right
  after a cast acts at the cast's end. `arrow`: all 168 frames exact (the projectile's DEF by the
  angle, its frames cycling, its 22 steps; the marked loop). Not built: the aiming cursor
  (`crspell.DEF`, animated: not read; the harness clicks within the millisecond, so no capture
  shows it), the count box's tint by the stack's spells (T7b-2a, below), the other handlers
  (T7b-2a, T7c).
- **Read at T7b-2a** (`cast_spell`'s handlers): Dispel on one stack plays its animation (the
  record's +8, 41) over the target at 100 ms, then ends every spell but Poison and writes the
  cast's line (0x5A15E0 to 0x5A1605); Cure (`stack_cure` 0x446500) takes effect, then plays 39 and
  writes the line (0x5A17F5 to 0x5A1816, the spell 0x25 named); Disrupting Ray flies its
  projectile (`c07spA0`, one DEF at 0°, 0x64225C, 0x642260) from the cast point to the target's
  centre unless the battle is hidden, lowers the defense +0xCC by the value (the specialty's
  bonus too) to 0 at least, plays 14 (translucent) at 100 ms, applies the spell (0x5A1AF7) and
  writes GENRLTXT 92 with the defense taken, no cast line (0x5A1A53 to 0x5A1B43).
- **Built (T7b-2a, `anim`):** `Player::cast` by the spell's number (`SpellLook::original`):
  Dispel, Cure and Disrupting Ray as read, a timed spell as before; the effects (`EffectApplied`,
  `EffectEnded`) applied at the handler's point, so the count box's hue (§4.3) changes with the
  next frame drawn; a stack's active spells from the snapshot on; the walk time under Haste and
  Slow (§7.3); a stack's defense lowered by Disrupting Ray (`Logged::Disrupted`, the defend line's
  gain). Disrupting Ray's value: the record's ruleset (the viewer's binary compiles it from the
  record's packs) at the caster's mastery, the record's own hero's (the other's book is hidden:
  none, *provisional*). Recorded (`casts.toml`'s way, then `sequences.toml`): `haste` (Haste,
  a hasted walk, Crag Hack's Slow ending it, a slowed walk) and `dispel` (Slow, Disrupting Ray at
  the Vampire Lords, Cure, Slow, Dispel), both as recorded on the first run and their frames
  exact (91 and 153); the projectile's last position at 4 093 and the animation from its third
  frame at 4 126 (the flight's 726 ms outside the deadline: frames 0 to 2 drawn at 4 126, as the
  clock's lateness rule gives). With the tint, `cast`'s and `resist`'s boxes exact.
- **Read at T7b-2b-2b** (the rolled effects over one stack, player 1): good morale
  (`good_morale_roll` 0x4649D3 to 0x464B23) plays `GoodMrle.wav` (category 2), 20 over the stack
  without its hit at 100 ms, writes its line, waits for the sound (`wait_for_sample`, at most 4 s)
  and redraws at once; bad morale and Fear (`bad_morale_check` 0x464BFE to 0x464D1D, `fear_check`
  0x464E0D to 0x464F3D, both inside the pick, `next_stack_to_act` 0x465283, 0x465293, before it
  sets the active stack) play their sound, write their line, play 30 or 15 *with* the stack's
  hit, then wait for the sound; good luck (0x441837 to 0x4418B3 in `melee_strike`, after the
  attack's pause and turns; 0x43F955 to 0x43F9D1 in `stack_shot`, after the shooter's turn)
  starts `goodluck.82m` (the archive's `GOODLUCK`; category 3), writes its line and plays 18
  without the hit. Fear's line (GENRLTXT 729, "Fear of the %s causes the %s to freeze.") names
  creature 132 by the count of its records on the other side, then the frozen stack.
  `play_magic_animation` with `show_hit` (unless the stack already shows group 3): the hit
  group's frames from 0, each a redraw at 100 ms on the deadline with the animation's frame of
  that number beside it (+0x132F0 at the frame count past its last), then the standing frame,
  shown at once by a redraw when the hit had at least as many frames; no wince sound. Each of
  the four is skipped while the battle is hidden, and morale and Fear in a network game's
  conditions (0x464A04 to 0x464A66; not read further).
- **Built (T7b-2b-2b, `anim`):** `Player::over_one` with the hit; `luck`, `good_morale` (after
  the attack is over: the shooter or striker turned back), `frozen` (bad morale, Fear: the
  `TurnStarted` before them neither makes the stack active nor writes the turn's log entry, and
  the panel skips that turn too);
  `Sound::GoodMorale`, `BadMorale`, `Fear` (category 2) and `Luck` (category 3 at the file
  volume), their names the asset tables' `[battle]` ones; `Logged::Fear` by the frightening
  creatures the ruleset names (`Casts::frighteners`). Not built: the sound's wait (as the
  heal's, *provisional*: the captures play without sound). Recorded (`sequences.toml`, forced
  rolls, the engine's seeds searched): `luck` (the Archangel's flight, then 18 from 2 730 ms, the
  strike 1 600 ms later than `strike`'s), `morale` (18 from 0 ms before the first shot, 20 from
  4 776 ms, 100 ms after the second shot's reaction ends, erased at 6 776), `bad` (the Goblins'
  seven hit frames beside 30's first seven from 0 ms, standing at 700, erased at 2 000) and
  `fear` (the Champions' eight beside 15's, erased at 3 100); all four as recorded. The frame
  check found the outline (manager +0x132C8, `draw_battle_stack` 0x43E259): the move's, strike's
  and shot's handlers clear it while they play and set it again to their stack at their end
  (`stack_move_handler` 0x445996, 0x4459ED; `strike_action` 0x445AC4, 0x445CA6), a Defend leaves
  it, a round's start clears it (0x4761EE) and a turn's start sets it (0x47837A). So the stack
  that acted last is outlined through good morale's animation (the Marksmen from 4 876 ms) and a
  freeze's (`bad`'s Champions, who defended; `fear`'s Azure Dragon), and a good luck's first
  frame shown takes the outline away like the shot's own (`Player::acting`; the command's first
  frame shown may be a magic animation's). Then all four exact: `luck` 75 frames as `strike`'s to
  the landing (its 134 pixels at 2 730 ms, §4.3) and exact from the animation on, `morale` 94
  (from the shot on the Goblins' box and the log's line listed: the original's lucky shot
  killed 5, the engine's 4), `bad` 36, `fear` 46.
- **Read at T7b-2b-3** (the strikes' effects; `melee_strike` 0x441610, `striker_effects`
  0x440BC0, `strike_spell_choice` 0x440500, `cast_spell`'s Poison, Acid Breath and Lightning Bolt
  handlers, `battle_log_cast`'s creature rows). A melee strike first chooses its spell
  (`strike_spell_choice`: Curse, Blind, Stone Gaze, Poison, Bind, Disease, Paralyze, Age, Acid
  Breath after their draws, into the target's pending +0xEC), then plays its animation; at the
  animation's end, before the dead die, each living stack's pending spell is cast as a creature's
  (`strike_animation` 0x469102: mastery 0, power 3; no hero frames, no resistance roll, the
  spell's sound async, category 3); only then the strike's line (0x441C5D), then the striker's
  own effect (`striker_effects`, one per creature), then Fire Shield. Recorded: `acid`'s
  "… writhes in pain …" before "The Wyvern Monarchs do 49 damage."
  - Poison (0x5A1590): the effect, the line (GENRLTXT 562 or 563 by the stack's number,
    0x5A8EFC), then 67 with the stack's hit (`show_hit`) at 100 ms.
  - Acid Breath (0x5A1E70): the defense lowered by 3 (to 0 at least; no spell set, so no tint),
    81 without the hit at 100 ms, then the executable's own line, not a table's: "Acid breath
    reduces the defense of the %s by %i" (0x688450; GENRLTXT 92 is formatted first and
    overwritten).
  - Life Drain (the Vampire Lords, 0x440C01): the hit points drained (at most the damage and the
    drinkers' loss), the creatures raised, the line (GENRLTXT 362 or 363 by the drinkers' number
    before the raise, the struck stack's name by its number before the strike; then 364 or 365
    when any rose, its line break its own), `DrainLif.wav` (category 2), 74 over the drinkers
    without the hit at 100 ms, the sound waited for.
  - Death Stare (the Mighty Gorgons, 0x440EB6): one draw per Gorgon, at most ceil(Gorgons / 10)
    killed; the line (GENRLTXT 119 for one, else 120), spell 79's sound (category 2), 80 over the
    struck stack without its hit at 100 ms, the damage, then `strike_animation(-1, 1)`: its
    reaction alone; the sound waited for.
  - The Thunderbird's bolt (0x441185): its draw, Lightning Bolt's test and damage (10 per
    Thunderbird, the target's modifiers), the line (GENRLTXT 368), `LightBlt.wav` (category 2),
    the bolt from the sky 1 at 10 ms, the damage and its line (`battle_log_damage` named by
    Lightning Bolt), then 49 over the marked stack with its reaction (`strike_animation(49, 1)`),
    the sound waited for. A hero's Lightning Bolt (0x5A0B0C) plays 1 at 10 ms, then 49 over the
    marked stack, its line *after* it.
  - The Rust Dragon's acid (0x44149F): its draw, the damage (a roll of the damage range × the
    Dragons / 2), spell 80's sound (category 2), the damage and its line (named by Acid Breath),
    81 without the hit at 100 ms, the sound waited for, then the stack's reaction.
  - `strike_animation`'s second argument (1 for the effects, 0 for the strike) resets the dirty
    rectangles (0x493710) before and after its loop.
  - Every cast, a creature's too, clears the outline as it starts (0x5A004C, `clear_outline`
    0x477B60: the outlined stack's hexes redrawn, a redraw shown at once) and outlines the stack
    whose turn it is as it ends (0x5A216B, `outline_turn_stack` 0x477AC0: standing if it is not,
    a redraw shown at once); found by the frame check (`acid`: the Rust Dragon outlined from Acid
    Breath's end, 6 300 ms, to the Wyvern Monarchs' Poison's start, 10 900).
- **Built (T7b-2b-3, `anim`):** the pending spells of a melee strike (`Player::pending_casts`,
  played by `iterate` before its tail: `strike_cast`): Poison and Acid Breath as read, the others
  their effects at once (not built: Blind, Curse, Disease, Age, Paralyze, Stone Gaze's glow,
  Bind); the striker's effects after the strike's line (`strike_spell`: the Thunderbird's bolt;
  `acid`, `drain`, `death_stare`); a hero's Lightning Bolt in `Player::cast`; Acid Breath sets no
  spell (`effect`); `Sound::Drain`, `Thunderbolt` (category 2, the asset tables' `drain_sound`,
  `thunderbolt_sound`, harvested from 0x660A40 and 0x660A30) and `SpellFile` (a spell's sound
  as a file, category 2); `Logged::Drain`, `DeathStare`, `Thunderbolt`, `Poisoned`,
  `AcidBreath`; the catalogue's `EXE:<address>` source for the executable's own text (the user's
  `Heroes3.exe`, read at run time; `%i` as `%d`). The loop of a melee strike counts a defending
  reaction's hit group (§7.4). Not built: the sounds' waits (as the heal's, *provisional*: the
  captures play without sound). Recorded (`sequences.toml`, forced rolls, the engine's seeds
  searched so its own rolls and damage fall as the original's): `thunder` (two rounds: the
  Thunderbirds fly within reach, the Vampire Lords defend; then the strike, the bolt at
  5 890 ms, 49 from 6 000, the drain's 74 from 8 410; seed 110: one Vampire Lord killed, 50
  drained, one raised, as the original), `acid` (Acid Breath's 81 from 4 200 ms, the acid's from
  6 300, Poison's 67 on the Rust Dragon from 10 900; seed 55: the strike back's 49 damage),
  `stare` (two rounds: the Royal Griffins strike the defending Gorgons for 37, the strike back
  kills two, Death Stare one; seed 39) and `lightning` (`bolt`'s round 1 with the Marksmen for
  the Pikemen); all four as recorded on the first run. The frame check (`frames --only`, then
  `--reuse` after the fixes): `thunder` 121 frames, `acid` 153, `stare` 98, `lightning` 34, all
  as the original's but the original's redraw artifacts and, where a two-hex flier lands beside
  its target, the boxes' frame (§4.3, as `luck`'s at 2 730 ms: 86 pixels at 4 490 in `thunder`,
  273 at 2 200 in `acid`, 433 at 4 620 in `stare`; not copied, listed); `thunder`'s drinkers' box
  listed at 8 410 (memory holds the raised number a frame before the screen shows it). Found on
  the way: the outline of the casts (above), and the loop's hit group (§7.4).
- **Read at T7c-1** (the handlers of the spells aimed at no stack, `mass_spell_animation`'s end,
  `strike_animation`'s loop length, `cast_spell`'s tail, `creature_cast_action`'s cast):
  - *Death Ripple* (0x5A0C81): the cast's line (`battle_log_cast`, caster kind 0 without a
    target: GENRLTXT 197), each stack tested, rolled, marked and damaged, then GENRLTXT 188 ("The
    Death spell does %d damage …", its line break as any line's) with one stack's damage when it
    struck one, else the caster's (`spell_damage_modifiers` without a target, 0x5A0DC6), then
    `mass_spell_animation` with reactions, a redraw shown at once (0x5A0E6D) and the Phoenix
    check. *Destroy Undead* (0x5A0E7E): the same with its line after the animation, GENRLTXT 187
    with the spell's name (not recorded). *A timed spell on several stacks* (Expert, a mass spell;
    0x5A10AB): `mass_spell_handler`, the cast's line, then the animation without reactions.
  - *`mass_spell_animation`'s end* (0x5A6B50 to 0x5A6C47): the marks cleared without a redraw (the
    animation goes with the next frame drawn), the dead die, and only if one died a redraw shown
    at once. The count boxes keep +0x50 (the number before the kills) until the cast's tail.
  - *The area spells* (`area_spell_handler` 0x5A4970): the animation at the hex first (time 100),
    then, with a stack struck, the line (`battle_log_damage` named by the spell, number 1: one
    stack's damage and kills naming it, else the caster's damage, every kill and the word for
    creatures, 0x5A4B1D), then `strike_animation(−1, 1)` and the Phoenix check. No cast's line.
  - *`strike_animation`'s length* (0x468CBB to 0x468CF9): L = max(n, A, R, A + R − 1), R the
    longest death group or hit group (a defending stack's hit group, §7.4); with no striker (A
    = 0) every reaction starts at iteration 0 (0x468EBA). *After the loop and the pending casts*
    (0x4691A9 to 0x469268) every stack not standing plays on, whatever drew it there: a frame a
    redraw at trunc(100 F) (partial: the reacting stacks' rectangle, shown), a group ending in
    the standing group (a death stays), until none changes; only then the dead die, the layers
    and +0x50 reset and the whole battle is redrawn at once. So a defending stack's group 4
    outlasts the loop, and a creature caster still on its cast group's last frame stands at the
    first of these redraws, unseen when it did not react (its rectangle is not redrawn) until
    the whole redraw at the end. This is what the shots' and spells' "whole group 4" was (§7.4,
    `machines`, `arrow`).
  - *`cast_spell`'s tail* (0x5A2058): every stack's +0xE9 and +0x0 cleared, its layer 4 and its
    +0x50 −1 (the boxes show their counts again); a hero's last frames; then for every caster
    `outline_turn_stack` (0x5A216B): the stack whose turn it is (a creature caster) outlined
    again, standing if it is not, in a redraw shown at once. Recorded (`dragon`): in round 3 the
    Faerie Dragon, not struck, stands in memory 600 ms into the reactions (the Swordsmen's hit
    group's six iterations) as their defend group plays on to its eleventh frame, on the screen
    only with the end's whole redraw; in rounds 2 and 4, struck, it stands after its hit's seven
    frames, shown at once, outlined only at the tail.
  - *The user's answer* (CLT-Q8, 2026-10-22): the group's unnegated angle, the missed caster's
    stale frame and the highlight kept through the caster's frames are drawing bugs: fixed in
    what the client draws, the original's reproduced only for the comparisons. The other
    oddities of this section are CLT-Q9 to CLT-Q14.
  - *Fixed (T7d)*, by `anim::Stage::bugs` (`h3_render::Bugs`; `Original` only in `h3-view
    sequences`, so the timing and frame checks and `drawn`; the window, `render` and `play`
    draw them fixed): (1) the casting group by VCMI's angle (`RangedAttackAnimation::
    setAnimationGroup`: hex to hex, y up) at the shot's 25°, from the caster's centre to the same
    point over the hex aimed at, so the rise is the anchors' rows: above 25° up (17), below −25°
    down (19), else straight (18; the same row, the caster's own hex); a Faerie Dragon beside a
    level target 18, toward one above 17, toward one below 19 (*provisional*: VCMI's points with
    the original's shot threshold, not VCMI's 0.2 rad); (2) a caster its spell missed shows
    standing with the play-on's frame that stands it, as memory has it; (3) a creature's action
    unmarks the mouse's stack at its start (`Action::unmarks`), as a hero's; `cast_spell`'s
    start unmarks it again (a no-op).
  - *A creature's cast* (`creature_cast_action`, completed): its frames' redraws mark it dirty
    (`battle_redraw` with its second argument) and it stays outlined through them; `cast_spell`
    clears the outline (0x5A004C) and the mouse's highlight (0x5A0055) as it starts, a redraw at
    once each (a hero's action has cleared the highlight already).
  - *The Faerie Dragon's draw* (`faerie_dragon_draw_spell` 0x447510, at its activation): when its
    side's player is human here, the executable's own line "The %s readies %s (press F to cast)"
    (0x660AD0; "ready", 0x660AAC, for more than one) with its name by its number and the spell's.
    Not built: the draw is private to the dragon's controller, and the recorded sequences are a
    spectator's records (the frame check lists the log's item in every frame of `dragon`, its
    first line the round-1 draw, Chain Lightning, the engine's own there). A Faerie Dragon casts its drawn
    spell (+0x4E0) at the hex, caster kind 1, mastery 2, power 5 per creature (0x44863E).
- **Built (T7c-1, `anim`):** `Player::untargeted` (Death Ripple, Destroy Undead, a mass timed
  spell, an area spell of a hero), `mass` (player 3: the reactions first, a hit running on into
  the standing group, the animation's last frame never drawn, the erasure with the next frame
  drawn), `at_hex` (player 5: `SpotKey`, `Timeline::spot`, drawn over everything by
  `scene::battle_at`; places 1, 4 and the others as read), `area`, `spell_reactions`
  (`strike_animation` without a striker: L as read, the groups left over played on, a caster not
  reacting standing unseen until the end's redraw, then the dead, the counts and a redraw; the
  caster outlined at the tail, `stand_caster`), `creature_cast` (the
  `StackCast` event: its group by the angle, its frames at trunc(trunc(A / n) F), the turn
  toward the hex and back, the outline kept until the cast starts, its shot sound until the
  cast's end; `Logged::Unmark` at the cast's start); the tail's counts (`pending_counts`: with the
  next frame drawn); `Logged::CastAll`, `MassDamage`, `AreaDamage` (keys `battle.log.cast`,
  `death-ripple`, `destroy-undead`); the
  mouse's `Input::Book`: a spell without a target is cast from the book at once, whose dialog
  leaves the status line empty (the harness's `cast:<k>`). Not built: Frost Ring's and Meteor
  Shower's areas are the engine's (their animations as Fireball's path); a mass Dispel or Cure
  (their handlers at Expert, not read); the area's at-hex frames over the panel (none reaches it
  here); a melee strike's or a shot's loop (`iterate`) still counts the drawn group 4 instead of
  playing it on after the loop (the same times; a later task may switch it). Recorded
  (`sequences.toml`): `mass` (Slow at Expert, then Death Ripple) and `dragon` (seed 174: the
  engine draws Fireball, Fireball and Inferno as the original did), both as recorded on the
  first run. The frame check: `mass` 53 frames, as the original's but the click's (126 pixels,
  all redraw artifacts; `cast`'s 71) and the boxes listed at 4 800 ms (memory holds the counts
  the tail reset, the screen the old ones); `dragon` 166, as the original's but the landing's
  three frames (142 pixels, all redraw artifacts), the log's item listed throughout (the draw's
  line). Found by it: the mouse's highlight cleared as a cast starts, the empty status line
  while the book is open (`Input::Book`), the groups played on inside `strike_animation`, and
  the counts at the cast's tail.
- **Read at T7c-2** (the obstacle spells' handlers, the objects' drawing, the round start's
  expiry, Armageddon's loop and line):
  - *The objects* (`battle_object`, 0x18 bytes; descriptors after the 91 usual types:
    `docs/exe/map.toml`): Quicksand `C17SPE1.DEF` (one hex, height 1, layer 1), Land Mine
    `C09spF1.def` (1, 1, layer 1), Force Field `C15spE1.def` (two hexes, height 3, layer 0) or
    from Advanced `C15spE10.def` (three, height 4), Fire Wall `C07spF61.def` per hex (height 3,
    layer 0); two fire descriptors of two and three hexes (`C07spF1`, `C07spF10`) are referenced
    by no instruction. `battle_redraw` draws them in the obstacles' passes (layer 1 before every
    stack, layer 0 in the anchor row's pass 2: 0x4945DC, 0x4948F9) at (left, top − 42 (height −
    1)) of the anchor, group 0, frame = +0x13FFC (the idle steps) modulo the group's frames
    (0x494680). An object whose +0xA (revealed) is clear shows only to the screen's side: the
    acting side when its player is human here (+0x54A6), else the other one (0x49460A to
    0x49463A, 0x494931 to 0x494961; +0x13D75, set only by 0x4F4350, shows all). A record holds the objects its viewer
    sees, so the viewer draws them all.
  - *Placing* (`cast_spell`'s handlers): Quicksand (0x5A0354) draws its pits over the hexes
    (0x5A0397 to 0x5A03F3: not a side column, no object bit, no stack, no corpse), each with
    `Quiksand.wav` started (`play_sample`, 0x5A040A, unless hidden), animation 55 at the pit
    (`time` 100, kept), the object (vanish 58, lasting the battle, revealed by the other side's
    natives, 0x5A0442) placed and a whole redraw shown at once (0x5A04C1); after the last, a
    wait for the sample (0x5A04DD, `sub_599EC0` up to 4 s: not built, the captures play without
    sound) and the cast's line (GENRLTXT 197). Land Mine (0x5A0536) the same with
    `landmine.wav`, 47 and vanish 59. Fire Wall (0x5A0821): for each of its two hexes (three
    from Advanced) animation 65 **at the hex aimed at** (0x5A0884: `[ebp+0x18]` stays the
    target), its object (owner the acting side, revealed, two rounds, vanish 66) placed and a
    redraw shown at once (0x5A0961); then the cast's line (0x5A080B). Force Field (0x5A0759):
    32 (33 from Advanced) at the hex aimed at, kept, its object (vanish 60 or 61) placed without
    a redraw (it shows with the next frame drawn: the hero's next casting frame), then the line.
  - *Expiring* (`round_start` 0x476152 to 0x4761E9): after the stacks' round start, each object
    with rounds left loses one; at 0 it is removed (`remove_battle_object`, out of every redraw
    after) and its vanish animation plays at its anchor (`time` 100, not kept: its frames and a
    last redraw that erases it, 0x4761A9); then the round's line.
  - *Armageddon* (`mass_damage_handler` 0x5A4BC0): each stack rolled against its resistance (a
    draw of 1 to 100 per stack, 0x5A4C4A), damaged with the acting side's hero's bonuses and
    marked; then (not while hidden) the DEF loaded (+0x132E8), each marked stack its death (5,
    sound 4) or hit group (3, sound 2) at frame 0 (a defending stack too), L = max(n, the
    longest such group) (0x5A4D47 to 0x5A4EEB); iteration i: the reactions advanced first (a
    hit running on into the standing group to its last frame, a death on its last), a redraw at
    trunc(100 F) not shown, the frame i tiled while i < n (from (0, 0), ceil(800 / W) columns
    and ceil(600 / H) rows, each cut to the 800 × 556 battlefield; plain), the battlefield shown
    (0x5A4F00 to 0x5A50C6). So every frame shows, and an iteration after the last frame shows
    the field plain. Then the marks cleared, the dead die and, if one did, a redraw shown at once
    (0x5A51C2); then, if a stack was struck, GENRLTXT 89 ("The Armageddon does %d damage.") with
    the spell's value through `spell_damage_modifiers` given no hero and no stack (0x5A5212): the
    caster's bonuses (Sorcery, the Orbs) are not in it, though they are in every stack's damage
    (CLT-Q16); then the Phoenix check. No cast line.
- **Built (T7c-2, `anim`, `scene`):** `ObjectKey`, `Timeline::objects` and `Moment::objects`
  (drawn with the usual obstacles in their passes, their frame by the idle steps, Force Field's
  hexes kept from the count boxes), `Player::place_objects` (the handlers above; `at_hex`'s
  `keep`: the kept frame erased by the next frame drawn), `vanish` (the round start's objects
  before its line; another removal *provisional*: not recorded), `field` (player 6: `FieldKey`,
  `Timeline::field`, `Draw::Tiled`), `Logged::Armageddon` (key `battle.log.armageddon`), the
  objects' sound (`Sound::Object`, the asset tables' `object_sound`); the snapshot's objects at
  0, a placement outside a cast or a pit revealed to the viewer drawn from the next frame on. The
  asset tables' spell `objects` (DEF, hexes, rows, layer, placing and vanish animations) are
  harvested from the descriptors. Not built: the sample's wait; Land Mine's blast (57) and a
  Fire Wall's burn when a stack steps in (`ObstacleTriggered`: no record steps on one); Dispel's
  and Remove Obstacle's removals as their handlers order them (the vanish plays where the event
  comes). A hidden object's hiding on the other human's turn: built at M5 T2b (§9.12). Recorded (`sequences.toml`): `obstacles` (seed 517 487 719: the engine draws the
  original's four pits in its order), as recorded on the first run. The frame check: 84 frames,
  all as the original's; the boxes listed at 7 800 ms (memory holds the counts the tail reset).
- **Read at T7c-3** (Chain Lightning's rays, the raises, Fire Shield's return, the explosions):
  - *Chain Lightning* (`chain_lightning` 0x5A6360): the mouse hidden (0x5A6380); 37 over the
    first stack at 0 ms a frame (0x5A6416); for each next stack a ray from the last one's
    centre (`stack_screen_x`, `_y`) to its own: d = trunc(√(dx² + dy²)), steps per tick
    trunc(d / 10) kept to 8 to 30, then `draw_ray` with no mouse or redraw of its own (its first
    argument 0: the shooter's mouse, the redraw and the shot group's rest are skipped,
    0x5A60D3), no branches, 9 pixels thick to 2, colour 305, draw bounds 10 and 80 (negated and
    swapped, −80 to −10, when the start lies right of the target, 0x5A5C74), delay 0 (every tick
    shows at once: its deadline is the clock's own); then `wait_ms` trunc(100 F) outside the
    deadline (0x5A6517) and a redraw at once (`battle_redraw` 1, 0, 0, 0, 1, 0) without it; after
    the last, the marks copied, `strike_animation` with 38 (0x5A665F), `battle_log_damage` named
    by the spell with a number of 1, the first stack's value through `spell_damage_modifiers`
    with the caster's hero, no stack (the word for creatures) and every kill (0x5A667C), a redraw
    at once, the mouse shown. The arguments' smoothness (2, or 3 beyond 20 steps) is passed but
    never read; the `randomness range` (80) only by branches, which it has none of.
  - *The ray's draws* (`ray_init` 0x5A5A90, `ray_aim` 0x5A5260, `ray_step` 0x5A5440): the record
    keeps the bounds (+0x68, +0x6C) and the steps per tick (+0x20). `ray_aim` returns at once once
    the head came within 15 of the target (+0x44: its aim and thickness stay; the viewer had
    missed this, a shooter's ray never meets it); else, as a shooter's, then, with bounds other
    than 0 and 0 and while f64(steps) × 1.5 < the distance left, the f32 angle +0x38 turned by
    f32(f32(v / 100) × ((2 − p) / 1.5)), v = `random_between(lo, hi)` (lo itself when the bounds
    agree), p the f32 share gone, each step at 53 bits (0x5A53C5 to 0x5A5431); +0x3C (an "arched"
    angle by trunc((lo + hi) / 200), 0 here) is computed and never read. `ray_step` draws
    `random_between(7, 12)` at each call and drops it (0x5A5478). Colour 305's entries by the
    distance from the edge are the code's own numbers (0x5A5786 to 0x5A5980): (255, 255, 255),
    (240, 240, 255), (224, 224, 255), (216, 216, 255), (200, 200, 255), then (192, 192, 255);
    colours 300, 301 and 303 have no caller (301 would write its number as the pixel).
  - *The raises* (`raise_stack` 0x5A7560): the line first, GENRLTXT 117 for several, 118 for one
    ("%d %s", the creatures raised and their name; 0x5A76C7 to 0x5A772F); 50 loaded
    (0x5A7740); L = max(n, d) redraws (1, 0, 0, 100, 1, 1: shown, on the deadline), each setting
    +0x132F0 = i and a lying stack (group 5) frame d − 1 − i, group 2 frame 0 from i = d; then
    the dead bit (+0x84 0x200000) and the mark cleared and a redraw at once (0x5A7867 to
    0x5A7881). Resurrection and Animate Dead (0x5A1907) write no cast line; Sacrifice
    (0x5A199C) kills its victim (no corpse: +0x84 0x10000000), writes the cast's line on it after
    its death (so by its number 0), plays `strike_animation` with 51, then raises; an Archangel
    (0x448563) plays its cast toward the corpse, `Resurect.wav` as a file, `raise_stack`, then
    waits for the sound; a Phoenix (0x469588) Resurrection's sound (category 3), then
    `raise_stack`.
  - *Fire Shield's return* (`fire_shield_damage_striker` 0x4409C0, from `melee_strike` 0x441C80
    after the strike's line and the striker's effects): every stack whose shield struck back
    marked, the striker marked struck, the damage taken, Fire Shield's sound as a file,
    `strike_animation` with 11, its line (`battle_log_damage` named by Fire Shield with a number
    of 1, the striker, its damage and kills), the sound waited for.
  - *The explosions* (`stack_shot` 0x43FA15, 0x43FE07): the Magogs (45) Fireball's 53 and sound,
    the Liches (64, 65) Death Cloud's 72 and sound (spell 76), loaded by `load_def` (not
    +0x132E8: memory's magic animation does not name it); at the target's hex x (±22 for a
    two-hex target by its facing) less half the width, its y less half the target's height and
    half the DEF's; each frame a redraw (0, 0, 0, 50 or 100, 1, 1), the frame by
    `battle_draw_magic_frame` (plain, not mirrored) and the battlefield shown; a redraw at once;
    then the area's damage and `strike_animation`.
  - *The strike's loop* after a shot or a spell: a reaction that dies spends no iteration
    standing, so when the longest all die L = R (the reading's), else R + 1 (the recordings';
    `raise`: the Marksmen shoot 100 ms after the Pikeman's death shows its last frame).
- **Built (T7c-3, `anim`, `ray`):** `Player::raising`, `raise` (player 4), `creature_raise`,
  `reborn`, `teleported` (*provisional*: the dissolve not read), the corpses kept for a raise
  (`bury`); `shield_return` (`Casts::shielded`, the ruleset's innate Fire Shield) and
  `explosion` (`Look::explosion`, the asset tables' creature `explosion`: a format change),
  drawn as a spot; `chain` with `ray::chain` (`Shape`, the draws through `Draws`: the capture's
  kept draws, `Casts::rays`, then the viewer's own generator), `Beam::until` and `cursor`; the
  near check in `ray_aim`; the strike loop's dying reactions; `Logged::Raised` (key
  `battle.log.raised`); the timing check leaves out the heroes' rolled reactions (`unrolled`).
  `raise`, `creatures` and `bolt` moved from `casts.toml` (now gone) into `sequences.toml` with
  their engine commands: played as recorded. **The harness** (the user's choice): `xdd.dll` logs
  the draws of `ray_aim`'s call (0x5A53FC) into `<H3CLOCK>\draws` (nothing forced), `frames`
  keeps them as the frame file's `rays`, and the frame check's timeline draws the original's
  rays with them. **Not built:** a created stack (`StackCreated`, a Pit Lord's Demons
  `RoseFromCorpse`): a record holds the first snapshot only, so the viewer lacks the new stack's
  numbers to draw it (QUEUE's inbox), nor the message when no hex is free for one; a clone's
  vanishing; the dissolves; Sacrifice's victim leaving no corpse; the samples' waits.
- **Not read:** the dissolve's blend; how the glows are painted; the summons' and the Pit Lord's
  creation beyond the dissolve; Earthquake (sieges); the force-feedback effects (IFC20, nothing
  drawn).
- **Fixed (T9d)**, the user's answers to CLT-Q9 to Q14, Q16 and Q17 (M3's review), by
  `anim::Stage::bugs` as T7d's (`Original` only in `h3-view sequences`): (Q9) `mass` sets the
  animation's frame i while i < n, every frame once; (Q10) a hit in `mass` and in Armageddon's
  `field` (the same run-on, 0x5A4D47) ends on the standing group's frame 0 (`Player::react`). The
  DEFs say where Q10 shows (every creature's group 2 in the four LODs, frame 0 against the
  last): in 87 of the 145 creature DEFs the last standing frame differs from frame 0 by 213 to
  7 339 pixels, 55 end their cycle on frame 0's picture (the three war machines have one frame),
  so the fix, built for all, changes
  exactly those where it is visible (the user: fixed where the frames differ visibly, else
  copied); (Q11) Chain Lightning's opening 37 at the bolt from the sky's 10 ms a frame; (Q12)
  each Fire Wall object's 65 at its own hex; (Q13) Animate Dead raises with its spell's own
  animation (4, `C01spE0`); (Q14) the Liches' cloud translucent, as its table entry; (Q16)
  Armageddon's line with the caster's value (`SpellDamage`'s `caster`); (Q17) Sacrifice's line
  on its victim by its number before its death.
- **Read and recorded at T9d** (`sacrifice`, the frame check's below): *a stack leaving without
  a corpse.* `stack_death` gives a stack whose +0x84 holds 0x10000000 (Sacrifice's victim, set
  at 0x5A1A1D) or 0x400000 (a summoned one) no corpse, marks it in +0x13438 (side × 20 + slot)
  and sets +0x13460 (0x4442C5 to 0x4442EE); after the deaths `strike_animation` (0x469307) then
  calls `battle_dissolve_removed` (0x466E00): a rectangle around the stack saved
  (`screen_save_rect`; recorded: the Archer's, 95 × 97), the battle redrawn
  without the stack (not shown) and dissolved into the screen by `screen_dissolve` 0x602DC0
  with trunc(150 F) a step (0x4670EF): eight steps, 1 200 ms at speed 0, outside the deadline,
  memory unchanged meanwhile; then the marks' clearing and the whole redraw at once, and
  `raise_stack`, whose first two frames the late deadline shows at once (§7.1). **Built:**
  `anim::GONE` (a layer no pass draws, holding no hex; the scene skips it), `Player::vanishing`
  (Sacrifice's victim), the dissolve's wait at `iterate`'s end (`DISSOLVE_STEP`,
  `DISSOLVE_STEPS`). **Not built:** the blend (the viewer shows the picture before it until its
  end, *provisional*; with the created stacks' dissolve, QUEUE's inbox).

### 7.17 The spell book (M5 T3a)

T3 cut in two by the user on 2026-10-27 (the book here; the hero's aim, the Faerie Dragon's F key,
`cast`'s 71 pixels and an aimed cast recorded T3b). Read: `spell_book_dialog` 0x59BDF0 for its
items, `spell_book_page` 0x59C9A0 for what a page shows, `spell_book_proc` 0x59D040 for its keys,
clicks and moves, `spell_book_text` 0x59BAA0 (a spell's hint), `dialog_show` 0x5FEAE0 (the
shadow), `dialog_item_at` 0x5FF3E0, `dialog_def_draw` 0x4EAB40, the text item's drawer 0x5BC5F0
again; built: `h3_view::book`, `ui`'s new kinds, the layout's `[book]`, the binary's
`spellbook`.

- **The dialog** (620 × 595 at (90, 2), the base dialog's flags 0x12; its items in the order they
  are drawn): `Spelback.pcx` (200) in the hero's owner's colours (0x450520); the mastery
  ribbons (201 to 212) and the spells' icons (`spells.def`, 213 to 224), DEF picture items
  78 × 65 at the places of §7.16; the places' texts (237 to 248), text items 86 × 36 at
  (113 + 86 c, 154 + 96 r) and (329 + 86 c, 154 + 96 r), `tiny.fnt`, colour 1, alignment 1;
  `schools.def` (236) at (117, 74) over places 0 and 1; `SpelTab.def` (225) at (524, 88); the tabs'
  areas (226 air, 227 fire, 228 water, 229 earth, 230 all; 83 × 43 at x 523, from the top air,
  earth, fire, water, all); the corners `SpelTrnL.pcx` (234, (97, 77)) and `SpelTrnR.pcx` (235,
  (487, 74)); the bookmarks' areas (231 combat at (219, 404), 232 adventure at (353, 405)); the
  spell points (233, `smalfont.fnt`, colour 7, alignment 5, the hero's mana as `%d`); the close's
  area (0x7801); the status line (249), `smalfont.fnt` over `spelroll.pcx` at (8, 569), 605 × 19 (the
  picture is 606 wide: an item draws at most its size of it). A DEF picture item of an interface
  DEF (type 0x47) draws its frame of group 0 as a DEF button does (0x47BF00); a text item adds 2
  to its colour while disabled (state bit 8). Transparent items take the mouse and draw nothing.
- **The shadow:** the dialog's flag 0x10 makes `vShow` darken the screen under a strip right of
  it and one below it, 8 wide and offset 8, each again one pixel in (`screen_darken` four times):
  the strips' insides at a quarter, their outer edge at a half. The battle behind stands still
  while the book is open (the modal loop runs no battle idle step: the original's frames at 100
  ms and a second are the same).
- **A page** (0x59C9A0): the hero's spells of the tab's schools and the bookmark's kind, each under
  its school (the tab's; on the tab of all schools the one the hero knows best, 0x4E51C0),
  sorted by level, school bit, name (0x59DBE0); twelve a page, a school's first page's emblem over
  places 0 and 1 and its later pages starting two earlier; each place its ribbon (its school's
  `SPLEV?.def`, 0x641D94, the frame the mastery on this battlefield, 0x4E5080), icon (the frame
  the spell's number) and text: `{name}\n<level>/<mastery>\n<Spell Points>: <cost>`, without
  `/<mastery>` for none (the executable's formats 0x684BEC and 0x684BDC; the levels GENRLTXT 173
  to 177, the masteries SKILLLEV 3 to 5, the word GENRLTXT 388), disabled (greyed) above the hero's
  mana; the empty places hidden; the left corner on a page after the first, the right one while
  spells follow. The book keeps its page, tab and bookmark for the same hero (§7.16): the first
  opening shows page 0 of all schools; an opening after the adventure bookmark too.
- **Its input** (0x59D040): a left press (the item's message on the button going down) on a place
  gives the spell when the hero can pay (the dialog closes on it), else a message (GENRLTXT 207);
  with the adventure bookmark the spell's description in a message; a tab, a bookmark or a corner
  turns the book (page 0 of a new tab or bookmark); the close closes it. Keys (scan codes):
  Escape and Enter close, A and C take a bookmark, the left and right arrows turn a page where
  its corner shows, up and down take the tab above or below (round to the other end; 0x641DA4,
  0x641DB8). A move over another item than the last (0x684B4C, never reset) sets the default
  cursor (`crdeflt.def` frame 1 over an item, 0 elsewhere) and the status line: over a place its
  spell's description at the hero's mastery to its first line break with ` (<level>)`
  (0x59BAA0), over the other items their `Help.txt` hints (records 450 to 460, 0x6A52D0), else
  nothing. The item the mouse takes is the last added that is shown, enabled and not disabled
  (0x5FF3E0; a greyed text is passed over). The option "Animate SpellBook" (0x698788, on by
  default) plays the page turn's video (`PgTrnLft`, `PgTrnRgh`) before a turn: the harness sets it
  off, the client plays none (no video yet).
- **Built:** `ui` (`Def`, `Label`, `Area`, an entry's `repeat`, every item's state, the DEFs and
  frames a screen gives, `Dialog::item_taking` as read, `Dialog::shadow`); `Draw::Label`,
  `Draw::Darken`; `h3_view::book` (`Book`, `Memory`, `Page`, `Open`: the page, the items' states,
  presses, keys, the mouse); the layout's `[book]` (the dialog, the schools' order, the ribbons,
  the cursor); the catalogue's keys `book.*` and the family `spell-text` (the descriptions by the
  context of the mastery, read raw: a `%` is text; a family over another's list). The binary
  (`spellbook`): the book's hero from the side to act's own observation (its book, its spells'
  masteries), the ruleset (levels, schools, kinds, costs: the cost at the mastery plus the
  observation's `cost_change`, at least 1, *provisional*, the review list) and the names; the
  school levels from the book's spells of one school (*provisional*). The window: the panel's
  button or C (on release) opens the book while the battle waits for a human; it takes the mouse
  and its keys (Escape closes the book, not the window); a spell the engine casts without a
  target (`cast_targets`) is cast at once; one that takes a target closes the book (T3b's aim).
  The capture set's situations take `actions` (`key:`, `click:`, `move:` at the frozen clock):
  `render --frames` plays them into the book.
- **Checked:** `book`, `book-page`, `book-fire` at 0 ms and `book-hint` at 100 ms and a second:
  exact (the shadow, the corners, the emblem, the greyed texts, the hint, the cursor). At 0 ms
  after a move the original's cursor still stands where it was (the frozen clock; its status line
  follows at once), so `book-hint` compares from 100 ms. In the window (Xvfb, `xdotool`): C opens
  the book, the hint over a place, the corner turns the page, Death Ripple cast from it.
- **Not modelled** (later): the message boxes (a spell above the mana, an adventure spell's
  description, `hero_book_button`'s refusals: T4, the dialogs); the right button's help; the
  page turn's video; the aim (T3b); the book on the adventure map; an unattended run's casts
  through the book (they are given without a press).

### 7.18 The hero's aim (M5 T3b)

T3b whole (the user, 2026-10-28, against a cut into the aim and the Faerie Dragon's key). Read:
`enter_player_cast` 0x59EC50's handlers and end, its loop (the modal loop 0x602520 with the
handler 0x5A3070), `spell_target_cursor` 0x59F700, the cast's line 0x5A8690, the refusal's
`spell_fails_message` 0x5A2880 (through 0x5A2C60), the area's marks 0x59F8A0 and
`set_area_mark` 0x43EFE0, `mouse_set_cursor` 0x50CCA0 and `mouse_poll` 0x50D680 for the spell
cursor, the callers' ends (the book's command 0x474F6D, the key F 0x475C17 to 0x475CF3), the
cast action's start (0x479036, `battle_stand_fidgets` 0x479DE0, `battle_refresh_creatures`
0x495BF0). Built: `h3_view::mouse::aim`, the aim in `mouse::track` (`Input::Aim`, `Cancel`,
`Book` taking the moves), `Shown`'s spell cursor, area marks and aimed hex, `Moment::area`; the
binary's `aims` (the engine's part), the window's aim, the key F, the right button
(`h3_sdl::Key::RightButton`), unattended casts pressed through the book and the aim.

- **Entering it** (0x59EC50): nothing while the acting side's hero has cast this round (+0x54B4,
  0x59EC76). The action and its fields cleared (+0x3C, +0x40 to +0x48); the mastery the acting
  side's hero's (0x59ECB4, read with no test that there is a hero). By the spell's number
  (0x59F6AC): Quicksand and Land Mine cast at once; Earthquake at once in a siege, else a
  message; the stack spells (15 to 19, 24 to 39, 41 to 58, 60 to 62) scan the field's inner hexes
  for one the target test 0x5A39C0 accepts (Chain Lightning leaving out the side's own stacks),
  none: "That spell will affect no one!" (GENRLTXT 186, a message box); a mass spell at the
  mastery (0x59E060) cast at once; else the aim. The area spells (20 to 23 and Berserk) aim at
  once (no scan). Force Field and Fire Wall (0x5A3250), Teleport (0x5A3570, 0x5A37D0) and
  Sacrifice (0x5A2D00, 0x5A2E90) have aims of their own (not read); Remove Obstacle and Clone the
  common one after their tests; the Summons cast at once. Every end sets the combat cursor's
  frame 6 (0x59F65C). The action is 1 (the hero's) or 10 (a creature's: the second argument).
- **The aim** (0x602520 with 0x5A3070): the input queue flushed as it starts; each turn of the
  loop the mouse polled and the handler called with the next message or none, and the handler
  first runs the battle's tick (0x474040): the battle idles (fidgets, the outline's pulse) while
  aimed. A move takes the screen's hex (`screen_to_hex`: the heroes' rectangles 0xFC and 0xFD,
  nothing −1) and calls the cursor only when it differs from the hex the cursor last took
  (0x688324, −1 none). The cursor is called once as the aim starts, at the mouse's point
  (0x50D500), its answer not kept: a click before the next move casts nothing. A left press
  casts at the hex last taken (none: nothing, the aim goes on); Escape (scan code 1) or the right
  button gives the aim up. Either end takes every stack's area mark off and forgets the hex.
- **The cursor** (0x59F700) at a hex of the field outside the side columns the target test
  accepts (and for Chain Lightning no own stack there): `SetCursor(spell + 1, 3)`, the cast's
  line, then the stack there marked (`battle_mouse_stack`: outline 0x70, its group 1) or, for an
  area spell (the spell record's flags 0x280: Frost Ring, Fireball, Inferno, Meteor Shower; and
  Berserk), the area's marks. Elsewhere `SetCursor(0, 2)` (the combat cursor's frame 0), the
  refusal as the line, the mouse's stack unmarked (`battle_clear_highlight` with a redraw) and an
  area spell's marks for that hex too (a side column's hex marks its area's stacks).
- **The spell cursor** (`crspell.DEF`, 20 frames, every hot spot (18, 27)): the mouse manager
  draws a type-3 cursor's frame 0 whatever frame is asked (0x50CD41) and keeps 0 as the frame
  shown, so a later `SetCursor(spell + 1, 3)` starts it again unless the frame then shown is
  spell + 1. The poll (0x50D804) steps it a frame on, round its 20, on a second grid of 100 ms
  (0x69CA1C, its next time moved on by max(the time since, 100)); the harness sets that grid from
  the opening as it does the 33 ms one (E17 M5 T3b, `h3mem.align_mouse_poll`).
- **The cast's line** (0x5A8690, not forced; none while the battle is hidden): "Cast %s" (27)
  for Force Field, Fire Wall and the four area spells; Resurrection and Animate Dead "Cast %s on
  %s" (28) with the stack they raise there, by its number (+0x4C: one the singular, else the
  plural), else 27; Sacrifice 549 and 550, Teleport's second hex 26 (their aims); Remove
  Obstacle "Remove this obstacle" (551); any other 28 with the living stack there, else 27.
- **The refusal** (0x5A2880 with "Select Spell Target", 24): for a spell aimed at a stack (flags
  0x70) with a stack on the hex, in order: Anti-Magic (+0x220 and a level below +0x460) "%s
  protects the %s" (171: Anti-Magic's name, the stack's by its number); a hostile spell (the
  record's +0 below 0) at an own stack, "%s can only be cast on hostile creatures" (172); a
  friendly one at an enemy, "... friendly creatures" (178); the creature's factor 0 (0x44A4D0):
  Resurrection, Animate Dead and Sacrifice "%s does not work on %s" (180), any other "%s are
  immune to %s" (181), both with the plural name; else 24. Force Field and Fire Wall: 182.
- **The area's marks** (0x59F8A0): the stacks of the area (`area_stacks` 0x5A46F0: radius 2 for
  Inferno, else 1, the centre left out for Frost Ring; Berserk's by the mastery, 0x5A4810), each
  living stack the spell may affect (0x5A8090 above 0) given the mark +0x4F1 (0x43EFE0): newly
  marked it starts its group 1 as the mouse's stack does; marked, its outline is the mouse's 0x70
  even when it is the active stack (0x43E268). With the option 0x6987C0 (off in the harness) the
  area's other hexes are shaded too (0x493EA0, not read).
- **After it:** the book's command (0x474F72): the mouse's hex forgotten (+0x132D4 −1) and the
  mouse posted where it rests (0x4ECC00), so a given-up aim shows the battle's cursor and hint
  there at once; after a cast the post waits in the queue and the next turn drops it. The key F
  (0x475CE8) makes a 1 into 10 and posts nothing: after a given-up aim the cursor stays the
  pointer (frame 6) and the line the aim's until the mouse moves to another hex than the battle
  last took (CLT-Q27).
- **The Faerie Dragon's key F** (0x475C17, key code 33): outside the tactics phase, for an
  active creature 134 with casts left (+0xDC), its drawn spell (+0x4E0) through the same aim as a
  creature's (installation.md §7.21); nothing once the side's hero has cast this round. The
  client offers the engine's targets (`creature_cast_targets`: the stacks the spell may affect,
  BAT-MAG-049; the legal list holds no cast); the original's key aims an area spell at any inner
  hex (BAT-Q32, open).
- **`cast`'s 71 pixels** (M3 T7b-1's listed difference, there taken for the cast's start): the
  book's closing, not the cast. `spell-aim` (the same battlefield) shows the same 71 pixels (x
  342–352, y 287–298: the top of the rock, obstacle 22, as the terrain) from its book's closing at
  0 ms until the redraw of the Marksmen's fidget at 200 ms; in `cast` the click in the same
  millisecond starts the cast, whose redraw (`battle_stand_fidgets` 0x479DE0: the background
  restored in `battle_refresh_creatures` 0x495BF0's rectangle, each item meeting it drawn whole)
  covers them at once. A redraw artifact of the original's buffers (§7.11: what the back buffer
  holds under the dialog comes back; not read further), not copied (the user, 2026-10-14):
  listed, as the projectile's hole in the rocks.
- **Checked:** `spell-aim` (`sequences.toml`) recorded: its timeline (42 changes) as recorded;
  62 frames, 59 exact (the spell cursor's frames and restarts, the cast's lines and the
  refusals, the area's outline and fidget, the Escape's posted move, the cast), the other 3 the
  book's 71 pixels (above). In the window (Xvfb, `xdotool`): C, Fireball's place, the Wolf
  Riders' area, the side column, Escape; Magic Arrow's place, a press there (nothing), the
  Champions (nothing), the right button; again, the Wolf Riders and the press: the cast; a
  Faerie Dragon's F and its Ice Bolt pressed on a Swordsmen stack. Unattended runs with a bot
  made to cast for the check (not kept): `spell-aim`'s scenario, `seq-dragon` and `seq-arrow`
  with both sides human, every cast pressed through the book and the aim or F, every frame read
  back exact.
- **Built:** `h3_view::mouse::aim` (`Aim`: the engine's targets, the area, whether it is aimed at
  a stack, the original's number, polarity and level, Anti-Magic's handle; `takes`, `line`,
  `refusal`, `marks`; `screen_hex`), `track` (the aim's state: its last hex, the spell cursor's
  start, the area's marks; `Input::Aim` with `creature`, `Input::Cancel`; the book's moves not
  taken by the battle), `Shown::look`, `area`, `aimed`, `Moment::area` (the outline), the layout's
  spell cursor; the binary's `aims` (the hero's targets from `cast_targets`, the dragon's from its
  legal casts, the area from the ruleset's `area`, a stack spell by its `target` and `mass_from`;
  the original's numbers from the asset tables), `Turned`'s aims, `Battle`'s aim (the book's
  choice, the key F, the press, the cancel), the window's keys (Escape and the right button give
  it up, F11 for full screen), an unattended run's casts pressed through the book (turned to the
  spell's page) and the aim. *Provisional:* the refusal's immunity (the client takes any refusal
  the engine makes that the other reasons do not explain for it), the area's marks of stacks the
  spell would not affect, Berserk's area in the side columns.
- **Not modelled:** the aims of Force Field, Fire Wall, Teleport and Sacrifice (the book closes
  on them and casts nothing); the message boxes (T4); the area's shading (an option off in the
  harness); the Faerie Dragon's cast by the mouse (its cursor kind 20: once its side's hero has
  cast, the key does nothing, as the original's, and the window gives no cast then).

### 7.19 The message box, Retreat and Surrender (M5 T4a)

T4 cut in three by the user on 2026-10-28 (here the box and the two buttons; the result window
T4b; Auto Combat and the tactics phase T4c). Read: `message_box` 0x4F6570, `message_box_build`
0x4F6990, its layout 0x4F5D80, the frame's builder 0x48FF00 (from 0x48FDC0), the font's measures
0x4B5A10, 0x4B5990 and 0x4B5A80, the picture slot 0x4F4EB0 (its spell case and common tail), the
handlers 0x4F08D0 and 0x4F0FC0, the DEF item's colour message (0x4EA810's subtype 0xD); the
panel's Surrender and Retreat (`battle_buttons` 0x474FA1 to 0x475151); the book's message boxes
(0x59D415 to 0x59D4E4) and the aim's entry's (0x59F085 to 0x59F135).

- **The call** (`message_box`, fastcall: the text, the style; on the stack x, y, three pictures'
  type and subtype, a "waiting" flag and a network timeout): it fills a record (0x2A0 bytes: the
  text; x, y; the box's width 256 and height 128; the text's corner (20, 30) and size; the
  scroll flag; eight picture slots of 0x4C; the style) and lays it out (0x4F5D80) before
  building the dialog. Every battle caller passes x = y = −1 and no timeout (the timeout counts
  only in a network game).
- **The layout** (0x4F5D80), with the medium font (`medfont.fnt`, 0x698A0C): the text's width
  starts at the widest word's (0x4B5A10: the glyphs' advances of a run without spaces and line
  breaks, `{` and `}` not counted), at least 256; while the widest line (0x4B5990: by the line
  breaks alone) is wider than it and it is below 578: the lines at that width (`text_line_count`)
  × the font's height, and when that is above 200 or 3 × it above 2 × the width, the width
  becomes min(578, 3 × width / 2) and the test again; else (or at 578) the width is the widest
  line once broken at it (0x4B5A80: a line's width without the space it breaks at; a word longer
  than the width counts 0, which never happens here). When the widest line fits, the width is
  it. The height is the lines × the font's height, at most 200 (then a scrolling text: the
  scroll flag). The box's width: the text's + 50 (at least 40 + the pictures' row, below)
  rounded up to 64, at least 256 (128 for style 4, the right button's popup); the text's width
  then the box's − 50. The box's height: the text's + 60, + 50 for the buttons (not style 4), +
  20 and each picture row's height and its caption's (+ 20 between rows) with pictures; rounded
  up to 64, at least 128; the text's top 30 + (box − that sum) / 2 (C division). The place: when
  x is −1 or x + width ≥ 799, (800 − width) / 2; when y is −1 or y + height ≥ 599,
  (600 − height) / 2.
- **The picture slot** (0x4F4EB0, type 9 a spell): the DEF `spellScr.def`, its frame the spell's
  number, its caption the spell's name; the slot's size the DEF's width and height + 2 (85 × 63);
  the caption's width the widest of 50, the picture's and its words' (each 2 + its glyphs in
  `smalfont.fnt`), at most 110, its height its lines there × the small font's height (widened by
  half, to at most 110, while it takes more than one line and more than 2/3 of its width). One
  picture: its slot centred ((box − slot width) / 2), its bottom at box height − caption height −
  30 − 50; its caption below it, centred on it. (Two to eight pictures in up to two rows of four,
  the caption's overhang and 40 between them: not needed by the battle, not built.)
- **The frame** (0x48FF00 with the box's corner and size, flags 0x12): `diboxbck.pcx` (256 ×
  256) tiled from the box's corner over its size, each tile a picture item (ids from 200, flags
  0x800) cut to what is left of the box (a picture item draws at most its size of its picture
  from the picture's corner: `dialog_picture_draw` 0x450450 passes the item's width and height);
  then `dialgbox.def` (an interface DEF, 64 × 64) frames as DEF items, flags 0x10: 0 at (0, 0),
  1 at (w − 64, 0), 2 at (0, h − 64), 3 at (w − 64, h − 64), then for each x = 64 to w − 128 by 64
  frame 6 at (x, 0) and 7 at (x, h − 64), then for each y = 64 to h − 128 frame 4 at (0, y) and
  5 at (w − 64, y). Their ids (the dialog's +0x4C to +0x50) take the colours of the player whose
  view the screen shows (`viewing_player` 0x4CEA20: in a hot seat game the adventure map's
  current player when human, else the highest human colour; 0x4F6B5D sends each the message
  0x200/0xD, which the DEF item answers with the players' palette's 32 colours over its DEF's
  entries 224–255, 0x4EAA4A); the tiles keep theirs. The base dialog's flag 0x10 casts the
  shadow (§7.17).
- **The items** (0x4F6990): the text, a text item at (20, top) of the text's size, medfont,
  colour 1, alignment 1 (centred, from the top), id 1 (a scrolling text in its place above 200:
  not needed, not built); the buttons by the style: the button's back `Box64x30.pcx` (66 × 32)
  at (x, h − 62) and the DEF button (64 × 30, frames 0 and 1, one hotkey) at (x + 1, h − 61).
  Style 1 (and 5, 7) an OK alone, `iOkay.def`, id 0x7802, Enter (scan code 0x1C), at x =
  (w − 66) / 2; style 2 OK (id 0x7805, Enter) at x = max(1, w / 2 − 74) and Cancel
  (`iCancel.def`, id 0x7806, Escape) at min(w / 2 + 8, w − 66); styles 3 and 10 the same with
  OK 0x7802 and Cancel 0x7801; 4, 8, 9 none. Then each picture: its DEF item at the slot's corner
  of its size, its caption (`smalfont.fnt`, colour 1, alignment 1) and a frame item around it,
  hidden (a choice's mark). The cursor: the default one's frame 0 (`mouse_set_cursor(0, 0)` at
  0x4F74AC), the one before restored at the end.
- **Its input** (the modal loop 0x602520 with 0x4F08D0, then 0x4F0FC0): the input queue emptied
  as it opens; the battle stands still behind it; a button's command (its click: released over
  it, or its hotkey released) of id 0x7800 to 0x7806 closes the box, the window manager's +0x38
  that id (0x7802 the picture chosen in styles 7 and 10). A style-1 box has no hotkey but Enter.
- **Retreat** (2002, `R`): style 2 with GENRLTXT 29 ("Are you sure you want to retreat?"); OK
  (0x7805) makes the action 4. **Surrender** (2001, `S`): the price (`surrender_price`
  0x695030, as the computer's), then style 2 with GENRLTXT 33 formatted with the other side's
  hero's name and the price; OK: when the side's player's gold (game +0x20B84 by player) is below
  the price, style 1 with GENRLTXT 30 ("You don't have enough gold!") and nothing else, else the
  action 5 with the price. Both buttons are enabled as §9.7 says (`battle_controls_state`).
- **The book's boxes** (style 1, no picture unless said): a spell above the hero's spell points
  with the combat bookmark, GENRLTXT 207 formatted with the cost and the points (0x59D462); an
  adventure spell clicked in battle, its whole description at the hero's mastery
  (`spell_book_text` with the flag 0) with the spell's picture (type 9, 0x59D4DC). The aim's
  entry (0x59F085): a stack spell (15 to 19, 24 to 39, 41 to 58, 60 to 62) or Teleport with no
  inner hex its target test passes, GENRLTXT 186 ("That spell will affect no one!"). *Not
  built:* the other entry refusals (Sacrifice's twice, GENRLTXT 186; Remove Obstacle's 713;
  Clone's and the Summons' limits, formatted; Earthquake outside a siege; Land Mine against
  natives) and `hero_book_button` 0x59E900's (an anti-magic garrison GENRLTXT 685, the Orb of
  Inhibition 684 formatted with the hero and the artifact, a spell above level 1 on Cursed Ground
  748, Recanter's Cloak): they need facts the client lacks (the engine refuses those casts);
  the right button's popups (style 4: the book's 0x59D0EC and 0x59D19B).
- **The text tables' quotes** (`text_table_records` 0x5BBBA0, from `H3TextFile::Load`): a
  record that starts with a quote starts after it and loses its last character before the CR
  and the trailing tabs (the closing quote); then every run of quotes in it becomes one
  (0x5BBD29 to 0x5BBD6A; the tables of several columns' reader 0x5BBE70 has the same code). So
  GENRLTXT's record 33, `"%s states:\n\n""I will … gold."""`, reads `%s states:` and the terms in
  single quotes. `h3_formats::text` keeps the doubled quotes: the client's catalogue makes each
  run one when it reads a table's field (`text::pack`, not the executable's own strings).
- **Built:** `h3_render::text`'s `widest_word`, `widest_line`, `wrapped_width` and
  `blit::draw_image_keyed_in_clipped`; `h3_view::message` (`layout`: the record's layout, the
  frame's tiles and pieces, the text, the buttons, one picture and its caption as `ui` items in
  the executable's order, the pieces in the viewing player's colours through `Shown::players`;
  `Open`: a press, a move off and back, a release over the button, a key down and up, as the
  panel's buttons); a picture item drawn at most its size (`Draw::Picture`'s `size`), a DEF item
  in a player's colours (`Draw::Button`'s `player`); the layout's `[message]`; the catalogue's
  keys `battle.retreat.ask`, `battle.surrender.ask`, `battle.surrender.poor`, `book.refused`,
  `aim.no-one`. The binary: `boxes` (what each box asks and what its OK does, shared by the
  window and the stills), `Writer::message` (the fonts), `Battle`'s box (opened by the panel's
  Surrender or Retreat, released or by S or R, while the battle waits for a human whose side may
  leave: Retreat when the engine lists it, Surrender when the side's own view prices it, whether
  it can pay told after the OK as the original's; the book's refusal and description over the
  book, "affects no one" over the battle; the battle and the book standing still behind it, the
  default cursor's frame 0), the window's keys and mouse while it is open (every key but the
  viewer's own goes to it: Escape is Cancel's, never the window's), unattended runs' `--leave
  retreat|surrender` (the button clicked, the question shown 400 ms, its OK clicked); the stills'
  actions `key:r`, `key:s`, a box's `key:` and `click:` (`spellbook::still`). *Provisional:* the
  viewing player taken as the attacker's when a human plays it here, else the defender's (the
  client knows no adventure map's turn; the executable falls back on the game's highest human
  colour); Surrender enabled when the engine prices it (its terms: the side may retreat, the
  enemy has a hero, the ruleset prices it), the original's panel by a hero on both sides outside
  a town's defence (0x478573): they differ where the side may not retreat.
- **Checked:** the capture set's `msg-retreat` (0 and 100 ms: the default cursor shows at once),
  `msg-surrender` (Crag Hack's terms in single quotes, the price 25 750), `msg-poor` (its OK by
  Enter), `msg-mana` (the book's status line keeps the clicked place's hint: the harness's park
  move goes to the box), `msg-describe`, `msg-no-one`: exact (`dialogs.toml`, Normal: Red's 20 000
  gold). `drawn` the other 124 the same. The window (Xvfb, `--check`): `smoke` in hot seat left
  by Retreat and by Surrender (5 000 gold each) through their questions, every frame read back
  exact; without gold the surrender's refusal shows and the run fights on.


### 7.20 The result window (M5 T4b1)

T4b cut in two by the user on 2026-10-28 (here the window as the captures' original shows it,
without its clip; the clip T4b2, by a port of FFmpeg's Bink decoder). Read: `battle_result_dialog`
0x4702D0 whole, its handler 0x471B90, `battle_end`'s calls (0x4777E9 to 0x4779CE), the experience
`battle_experience` 0x46A350, the DEF item's constructor 0x4EA720 and drawer 0x4EAB40 (the flag
0x10's path by the DEF's type, the mirror passed to 0x47BF00).

- **The call** (`battle_end`): once the battle is decided, its last log line, the combat cursor's
  pointer (`SetCursor(6, 2)` at 0x4777FC, shown), the whole screen darkened (`screen_darken`
  0x44E5F0 over 800 × 600, 0x47782C) and shown, then the window built with the attacker's and the
  defender's heroes (the manager's +0x53CC, +0x53D0), the shown side (the winner's when its player
  is a human here, else the other one; a draw: 2, read as the defender's), the winner (−1 none),
  whether the battle is at a town (+0x53C8) and the experience (`battle_experience`'s, 0 when the
  shown side lost); its tune (§8.2), the modal loop (0x471B70); the experience given after it
  closes (0x477948), then Eagle Eye's message.
- **The dialog** (0x5FE9F0): at (165, 19), 470 × 561, the flag 0x10 (its shadow, §7.17). Its
  items in order: `CPResult.pcx` at (0, 0) in the viewing player's colours (0x4703A0); the texts
  (x, y, w × h, font, colour, alignment) GENRLTXT 408 at (0, 290, 466 × 100, `BigFont.fnt`, 8,
  centred), 409 at (0, 320) and 410 at (0, 412), both colour 4; the attacker's name at (89, 37,
  115 × 20, `smalfont.fnt`, 4, left), its portrait at (21, 38, 58 × 64): a hero's large portrait
  (`hero_table` +0x34 by the hero's portrait byte +0x34), else `TwCrPort.def` with the frame of its
  strongest creature + 2 (below); an empty text (83, 112, 65 × 20, right); the defender's name at
  (266, 37, right), portrait at (392, 38), the creature's frame **mirrored** (the constructor's
  `mirror` 1, 0x4708F6), an empty text (266, 38, left); each side's word (411 "Victorious" for the
  winner, else 412) at (17, 116) and (367, 116), 84 × 20, colour 1, centred; the result's text at
  (68, 202, 332 × 70), colour 4, centred both ways (alignment 5); each side's losses (the
  attacker's row from y 343, the defender's from 440): none, record 32 at (42, y + 10, 384 × 36),
  colour 1, centred; else at most 7, from x = (468 − 42n) / 2 + 11 by 42: `cprsmall.def`'s frame
  of the creature + 2 at (x, y), 32 × 32, and its count (`%d`) at (x − 5, y + 38, 32 × 32), colour
  1, centred; `Box64x30.pcx` at (384, 506) and the OK (`iOkay.def`, id 0x7802) at (385, 507), its
  hotkeys Enter and Escape. The executable gives most items the id 200; the client gives each of
  those its own (their texts by id). Then the clip (0x597570 with its index, at the screen's
  (272, 89), 256 × 120: §7.21).
- **A side without a hero** (0x4705A2, 0x470810): of its first 20 records but the arrow towers,
  the first with the highest AI value (the creature record's +0x40 at the stack's +0xB4); its
  name the plural when the side had more than one such record (dead ones counted), else the
  singular.
- **The losses** (0x470D89): each side's first 20 records, `start − count` (+0x60, +0x4C) where
  positive, summed by creature in the order the creatures first lost some.
- **The text** (into the message buffer): the shown side won: 303 (the enemy surrendered), 304
  (fled) or 305, then with the shown side's hero 306 with its name and the experience; it lost:
  with a hero 307, 308 or 309 with the hero's name; without, 310, 311 or 312 (the flags 0x697744
  and 0x6985A3, set by either side). The clip's index and the tune by the same branches.
- **The experience** (0x46A350): the loser's first 20 records without the summoned and war
  machine flags, `(start − count) × hit points`; 500 more for the loser's hero, 500 less when a side
  retreated or surrendered, 500 more for the attacker at a town; with a hero, `crt_ftol(Learning
  × f32(points))`.
- **Its input** (0x471B90): the OK's command (0x200/0xD of 0x7802) closes it; in a network game a
  deadline presses it (0x697784: 15 s, 2 s after a remote turn; not built).
- **Captured** (the harness's original, `VIDEO.VID` empty: the clip's slot shows the picture's own
  black panel): `result-retreat` (dialogs.toml: R, Enter) and `result-surrender` (result.toml: S,
  Enter) exact; the screen halved, the shadow's strips darkened twice more. `end`'s last frame
  (3 730 ms, a victory: 600 experience, the defender's 10 Pikemen) exact once the battle behind
  stands at its end with the heroes' flags as the frame before showed them: the end's last action
  is drawn, but the idle step that would turn the flags at that moment never runs before
  `battle_end` darkens the screen as shown (`h3_view::result::behind`). `defended`'s end (1 487
  ms, a town defended: Ufretin's 510 experience) all but 122 pixels of the keep's shooter, redraw
  artifacts: the screen keeps its shot's climax frame (its later frames never shown, CLT-Q19 (5)),
  where the viewer draws it standing; memory's last frame (1 010 pixels) is not it either.
- **Built:** `h3_view::result` (`layout`: the items in the executable's order; `text`,
  `shown_side`, `backdrop`, `behind`, `CURSOR_FRAME`), a DEF item's `mirrored` and
  `h3_ui::ui::Draw::Button`'s, the asset tables' heroes' `portrait`, the layout's `[result]`, the
  catalogue's `result.*` keys; the binary's `results` (`window`, `losses`, `strongest`,
  `experience`: from the engine's `BattleOutcome`, which the pure library does not name), the
  window's result (opened as the battle is decided, its OK closing the
  battle's screen), the stills' (a box's OK that gives a command) and the sequences' (from the
  timeline's end). *Provisional:* a hero's portrait its type's (a map's own portrait not known),
  no Learning (the client's heroes have none), a hero-less side's face not captured (the
  harness's maps give both sides a hero).

### 7.21 The result window's clip (M5 T4b2b)

Read: `video_play_in_dialog` 0x597570 whole, `video_open_in_dialog` 0x44D830, `video_open`
0x44D5A0, `video_tick` 0x44DAA0, `video_redraw` 0x44D9E0, `video_show` 0x5979D0, `video_close`
0x44DCC0, `video_service` 0x5976E0, `video_redraw_any` 0x597740, `video_archives_open` 0x598210,
the result window's show 0x471AF0, `dialog_show` 0x5FEAE0 from 0x5FEBB8, `dialog_redraw` 0x5FF020
to 0x5FF0B0, the clip table 0x6839C0. `BINKW32.DLL` (RAD's Bink 0.5b, 1998) run as a black box
under Wine (`clients/capture/binkprobe.c`) and, where that stalled, read in its colour tables
(the user, 2026-10-28: captures and black-box runs, its code read if they do not settle it fast).

- **The clip** (0x471A24): the window's last act plays clip *i* of the table (20 bytes an entry:
  the start's name, the loop's, flags) at (272, 89) in the clip's own size, playing on at its end
  and shown; *i* by the branches of its tune (§7.20; `h3_view::clip::of`): 0 `Win3`, 1 `LoseCstl`
  then `LoseCslp`, 2 `rtstart` then `rtloop`, 3 `Surrender`, 4 `Defendall` then `defendloop`, 5
  `lbstart` then `lbloop`. 0x44D830 opens the loop first (flags 0x400000), then the start, its
  frames copied into the back buffer at that point.
- **Where it is found** (0x44D5A0, the archives opened at 0x598210): `<name>.bik`, compared
  ignoring case with the directories (a count, then 44-byte entries of a 40-byte name and an
  offset); the Complete edition (3) searches `data\Video.vid`, then the CD's archive when one is
  open, then `data\h3ab_ahd.vid`; BinkOpen on the archive's handle at the entry's offset. The
  client (`videos.rs`) reads only the directories (the user's `VIDEO.VID` is 629 MB) and a clip's
  bytes when a batch takes it; an installation copy without `VIDEO.VID` takes the user's
  `original-data/battle-videos.vid`, as the harness's run directory does.
- **The playback:** each pass of a message pump or wait (`video_service`) ticks the clip
  (`video_tick`): when BinkWait lets it, BinkDoFrame, BinkCopyToBuffer, then at the start's last
  frame the start closed and the loop's handle the one playing (the table's +9 0: its first frame
  not drawn now), else BinkNextFrame (a clip without a loop circles), then `video_show`: BinkGetRects
  of the handle playing, their union shown. Showing the window (0x471AF0: `dialog_show`'s call at
  0x5FEBD4, then its own at 0x471B17) redraws the clip twice (`video_redraw`: BinkDoFrame while
  at the first frame, BinkCopyToBuffer), so frame 0 is on the screen at once.
- **The DLL's timing** (black box, its clock ours): the p-th BinkDoFrame of a clip (from 0) is
  allowed once the clock reaches the first one's time + ⌊1000 p / rate⌋ ms, every decode counted,
  a redraw's too. So the window's clip shows frame 0 from the opening, frame k ≥ 1 from
  ⌊1000 (k + 2) / 15⌋ ms (200, 266, 333, ...); at the start's last decode (⌊1000 (n + 1) / 15⌋)
  the loop's rectangles are empty (no frame decoded yet) and its first frame comes at the next
  tick, in the same millisecond, so **the start's last frame never reaches the screen**; the loop's frame j from that time
  + ⌊1000 j / 15⌋, circling. Both slips (frame 0 three periods, the start's last frame lost) are
  **CLT-Q31**: fixed in what the client draws (frame k from ⌊1000 k / 15⌋, the start's last its
  period, the loop from ⌊1000 n / 15⌋), the original's for the comparisons (`h3_render::Bugs`).
  Not modelled: a redraw while a clip stands at its first frame (a press over the window before
  its first tick) delays it a frame more.
- **The colours** (BinkCopyToBuffer into the 5-6-5 screen, BinkDDSurfaceType 3): the DLL's YUV
  tables, checked on all 2^24 combinations (a synthetic clip of every Y, U, V: `binkprobe frames`
  in 5-6-5 and YUY2): each pixel takes its 2 × 2 block's chroma unchanged; L = 0 for Y ≤ 16, else
  ⌊38 154 (Y − 16) / 32 768⌋; top five bits clamp(L + t(66 101 (V − 128))) / 8, middle six
  clamp(L − t(26 639 (U − 128)) − t(12 837 (V − 128))) / 4, low five
  clamp(L + t(52 299 (U − 128))) / 8, t a division by 32 768 rounded toward zero, U and V
  `h3-bink`'s planes 1 and 2. Exact but where a sum leaves −256 to 511 (the DLL then reads its
  neighbouring tables; the clips' sums never do) and, in the synthetic clip only, a frame's first
  ten pixels where V is 0 (the clips' V is 87 to 205). `h3_view::clip::rgb565`.
- **The decoder against the DLL's:** `binkprobe frames` in YUY2 gives the DLL's own planes. The
  chroma's hidden rows (60 to 63, review item 16): the DLL keeps them as `h3-bink` does (RTLOOP
  from frame 12 and LBSTART's frame 46 as `h3-bink`'s). But Bink 0.5b's inverse DCT is x87
  floating point (`ExpandPlane`'s 8 × 8 transform; single-precision constants √2, 1.848, 1.082,
  2.613; its quantizers floats), FFmpeg's integer: an intra or inter DCT block can differ by 1
  in a pixel, which motion carries on. Five clips are exact in every frame (`Win3`, `lbstart`,
  `lbloop`, `LoseCstl`, `LoseCslp`); `rtstart` (31 of 40 frames, at most 49 pixels a frame),
  `rtloop` (10 of 40, 39), `Surrender` (74 of 100, 3), `defendloop` (66 of 102, 3) and
  `Defendall` (all 99, up to 388) differ by 1 in a channel. **CLT-Q30** (the user, 2026-10-28):
  listed now, `h3-bink` made to decode as the DLL later (an inbox task).
- **The harness:** `setup.sh` copies `battle-videos.vid` into the run directory as `VIDEO.VID`
  (an empty one without it); `xdd.dll` points BINKW32's own `timeGetTime` import at the virtual
  clock (else the clip ran in real time under the frozen clock); `binkprobe.exe` beside the game.
- **Captured:** `result-retreat` at 100 (frame 0), 2 700 (rtstart's 38: 1 pixel, the DCT's), 2 733
  (rtloop's 0, exact) and 3 000 (rtloop's 4: 9 pixels, the DCT's); `result-surrender` at 100 and 6 866
  (its second round's frame 1), exact; `end` to 4 800 ms: its 53 frames as before, the window's
  14 frames of `Win3` from 3 930 every 66 or 67 ms exact; `defended` (below).
- **Built:** `h3_view::clip` (the table, `of`, `shown`, `decodes`, `Video`, `rgb565`), the scene's
  `Draw::Video`, the batch's `.BIK`, the compositor's copy; `h3_view::result::clip` and the
  layout's `clip`; `h3_assets::vid::directory`; the binary's `videos.rs`, the window (a frame at
  each decode while the window stands), the stills' and sequences' clip.

### 7.22 Auto Combat (M5 T4c1)

Read: `battle_buttons`' case for 2004 (0x474E5A to 0x474EEA), `battle_loop_turn` 0x4740D0 from the
tick to its action (0x47417A to 0x474265, 0x474477 to 0x4744D4, 0x474622 to 0x474643),
`battle_controls_state` 0x4782D0 again for Auto Combat (0x4782D4 to 0x478326, 0x47846E to
0x478525, 0x47864A, 0x478804 to 0x478884), `battle_panel_disable` 0x46BF90 whole, the panels'
vtables (0x63D420; the tactics phase's 0x63D430), the input manager's mouse messages 0x4EC290 (their
kinds), `ai_battle_action` 0x477EE0's head, `ai_hero_cast_turn` 0x422DA0's head.

- **The switch** (0x474E5A): the command of the button 2004 (its click, or its hotkey A's release,
  0x45659F), taken only while the side to act is played here (+0x132B4), toggles +0x132C4. On,
  `battle_controls_state` runs again for the turn under way, and the panel dims (the panel's
  vtable +0xC, 0x46BF90) if its stack now plays itself (`stack_automatic` 0x474BF0: every
  creature with the setting `Combat Auto Creatures` 0x698794, which the harness sets; a war
  machine by its own, 0x69879C to 0x6987A4; the client's bot plays them all, *provisional*: no
  options dialog). The next pass of the battle loop gives that stack's turn to the computer
  (0x4744C0, `ai_battle_action`): the action starts in the same millisecond. Both sides human,
  the button is disabled (0x46BB40, 0x478670): no switch in hot seat. It is never switched off by
  its own click: while it plays no human's turn takes one (below).
- **The panel** (0x47846E to 0x478520): a turn's start while it is on and the stack plays itself
  dims the seven buttons and, for a side whose player is human (+0x54A4 + side), gives Auto Combat
  back its enabled state and the selected one (0x10). 0x46BF90 sets only the disabled state, and
  the selected frame outranks it (§9.7): through the computer's turns, and after a stop, the
  button stays selected until a human's turn it does not play clears it (0x47864A). The picture's
  colours and the log's scroll buttons stay as they were.
- **The stop** (0x47419B to 0x47425B): each pass of the battle loop while the battle is shown, the
  side to act is played here, Auto Combat is on (or the turn is remote) and the active stack plays
  itself, reads the input queue until a message of kind 0 (none left), 1 (a key down), 8 or 0x20
  (a left or right button down, double clicks the same) or 0x10 or 0x40 (their release), the
  moves (4) and the keys' releases (2) read and dropped; one other than the idle message stops it:
  the queue emptied (0x4EC570), +0x132C4 cleared, `battle_controls_state` again (now as without
  it). The computer acts in the pass that meets its stack and the action plays inside it, so a
  pass meets an automatic stack only as a turn starts: a key or a mouse button pressed while Auto
  Combat plays stops it as the next turn starts, whoever's. The press does nothing else (taken
  from the queue by that read, the dialog's hotkeys never see it): recorded, D pressed so gives
  the human's turn it stops at no Defend, a click on a hex no walk. Only the battle's own keys
  act: on a human's turn the stop comes at, the battle's handler gets the message found
  (0x474625, `battle_buttons`' key switch): ↑ and ↓ scroll the log, F aims a Faerie Dragon's
  spell, then (read, not recorded); on the computer's turn it is lost.
- **The mouse:** the turn's start shows the pointer (0x478321) and the computer's turn unmarks the
  mouse's stack (`ai_stack_turn` 0x422200); a human's turn under Auto Combat posts the mouse
  (0x478857) but the stop's read drops the move: moved over the battlefield, the mouse keeps the
  pointer and marks and enters nothing, as on the computer's turns (`Turn::human` false).
- **Not built** (the client's bot neither casts nor leaves): the computer's cast of the human's
  hero at a turn's start (`ai_hero_cast_turn` 0x422E42, under the setting 0x698798), and its
  question to the human, once a battle (+0x1402F), when it would retreat or surrender for them
  (0x477F56: a two-button box of GENRLTXT's row; Cancel keeps fighting and asks no more).
- **Built:** `h3_view::auto` (`Auto::of`: per turn whether it plays from its start or from a time
  in it, from the inputs `Input::Auto` and `Input::Stop` and the turns' starts), the panel's
  `Autoplay` (`panel::shown`) and `Panels::with_auto`, the mouse's automatic turns
  (`pointer::shown`), the sequences' `key:a` (the original's table: the switch; a key or a click
  then: the stop) and their recording's end after an `at:` action; the window: the button or A
  on a human's waiting turn switches it on (`Battle::switch_auto`: the bot's command for the
  turn at once, `Live::set_auto`), any key or press but the window's own while it is on stops it
  (`Battle::stop_auto`; ↑, ↓ and F then act at the start of the turn it stops at, a human's); a
  human's turn it stops at waits, its stack outlined from its start
  (`Growing::outline_waiting`: the command before was pushed as followed by the computer's);
  unattended, `--auto <n>` switches it on at the human's n-th turn and stops it two commands
  later.
- **Recorded** (`auto`, the melee's battle): A at the Angels' turn, the flight at 0 ms; the mouse
  moved to (400, 300); A again at 1 500 ms, the stop at the Wolf Riders' turn; the Pikemen's turn
  at 7 530 waits. The timeline as recorded (101 changes); the 106 frames' panel and cursor exact
  throughout (dimmed, Auto Combat selected from the switch through the computer's turns, lit and
  unselected on the Pikemen's); beside the log's rolled lines (listed), the flight's redraw
  artifacts as `melee`'s, the strike's start at 2 730 (435 pixels, 222 counted as artifacts: the
  original's partial redraw of the count boxes, as `strike`'s, §4.3) and, at 7 530, Orrin's
  victory (2 400 pixels) whose first frame the screen shows a step before memory holds it (the
  frame check draws memory's heroes).

## 8. The window, input and audio evaluation (M2 T2)

ADR-0020 point 4 left the library to a measured evaluation at M2; the choice, the user's
(2026-10-14), is [ADR-0024](../adr/0024-client-window-audio.md). Three spike adapters
on the throwaway branch `client/e17-m2-t2-spikes` (`clients/spikes/`, never merged) each play the
same *reel*: 302 distinct battlefield frames of `h3-view play scenarios/battle.toml` (its first 20
s, `--png`) at their times, looping, with the game's cursor (`CRDEFLT.DEF`'s first frame), an
effect on a key (`PIKEATTK.wav` from `Heroes3.snd`) and `COMBAT01.MP3` streamed. Each writes a
report: when every frame showed against its time, the gaps between presents, the audio device and
its callbacks, the time from a key press to the first mixed buffer that holds the effect, and a
read-back of the drawn frame compared with the frame widened (the PNG's bit repetition, §3.2). A
`--selftest` drives the cursor and an effect every 3 s. Measured in the sandbox (Linux under Xvfb
with Mesa's software GL, and the Windows builds under Wine), then on the user's Windows machine
(2560 × 1440, 74.93 Hz, 100 % scaling; the `measure.cmd` runs and the user's own look and listen).

**Candidates** (`cargo info` and crates.io, 2026-10-14): `sdl3-sys` 0.7.2 (Zlib; SDL 3.4.18, the
`sdl3-src` crate builds it with CMake and a C compiler; the `sdl3` 0.20 wrapper is MIT, 0.x and
young), `minifb` 0.29 (MIT or Apache-2.0), `miniquad` 0.4.11 (MIT or Apache-2.0, OpenGL), audio
through `tinyaudio` 2.0 (MIT; DirectSound on Windows, ALSA or Pulse on Linux) with a mixer of the
spike's own for the last two. Left out: `sdl2` (SDL 2 is in maintenance; SDL 3 is its successor),
`quad-snd` (no streaming, no MP3), `oddio` (a mixer only, last release 2023), `winit`, `cpal` and
`symphonia` (outside the licence policy, ADR-0009).

**Results on Windows** (20 s each; times in ms):

| | SDL 3 | minifb + tinyaudio | miniquad + tinyaudio |
|---|---|---|---|
| backend | Direct3D 11, WASAPI | GDI `StretchDIBits`, DirectSound | OpenGL (WGL), DirectSound |
| read-back at 2× | exact, the XRGB8888 and the RGB565 texture | (none: GDI) | exact (at 1×; the window opened at 1×) |
| frame after its time, vsync off | median 1.4, p95 3.4, max 6 | median 1.0, p95 1.5 (no vsync at all) | median 0.7, but 4 400 presents a second |
| the same, vsync on | median 8, p95 15 (within one refresh) | — | median 20 (it draws every refresh) |
| audio | 48 kHz, 480 frames (10 ms), requests every 10.0; clean | 1 024 frames (23.2 ms), callbacks at 20 and 30: underruns, **heavy crackling** | the same backend, the same crackling |
| key to the effect's first mix | mean 4, max 8 | mean 12, max 24 | mean 11–13, max 17 |
| cursor | the game's as a colour cursor (scaled 2×) or drawn | drawn only | drawn only |
| the user | "no problems", no visible difference with or without vsync; the hardware cursor "a touch smoother, snappier" | heavy sound artifacts | similar to minifb |

**Other findings:**

- **Pixels.** Every library showed the frame exactly at integer scales (the sandbox's read-backs
  and screen captures at 1× and 2×, the user's GPU at 2×). Direct3D 11's sampling of an RGB565
  texture gives the same 8-bit values as the bit repetition, so the window may upload the screen
  as it is.
- **DPI.** SDL's window takes `SDL_WINDOW_HIGH_PIXEL_DENSITY` (per-monitor aware: exact pixels at
  any Windows scaling); `miniquad` declares DPI awareness with `high_dpi`; `minifb`'s window is not
  DPI-aware, so Windows would stretch it on a scaled display (not seen at the user's 100 %).
- **minifb:** integer scales 1, 2, 4 and 8 only (no 3×, the fit for 2560 × 1600 and 4K at 2400
  × 1800); no fullscreen; no vsync; `ScaleMode::UpperLeft` shows the buffer unscaled in a scaled
  window (`Stretch` scales it).
- **miniquad:** draws every refresh (or in a busy loop without vsync); no timers in its blocking
  event loop, so a frame due between refreshes waits; no custom cursor image.
- **tinyaudio:** the DirectSound backend's notifications arrive late on the user's machine (a
  larger buffer would trade the crackles for latency); its macOS backend builds `coreaudio-sys`
  with `bindgen`, whose `clang-sys` is Apache-2.0-only (the deny policy over all targets rejects
  it; Windows and Linux pass).
- **SDL 3's audio:** one logical device (`SDL_OpenAudioDevice`) with a stream per source, each
  converting and resampling to the device's format (SDL mixes them); a device opened with
  `SDL_OpenAudioDeviceStream` takes no further streams. A one-shot clip needs
  `SDL_FlushAudioStream` after it is put, or the resampler keeps its last frames back and the
  stream never empties. `SDL_SetAudioPostmixCallback` sees each mixed buffer (the spikes' latency
  measure). `SDL_WaitEventTimeout` wakes on time on Windows, X11 and Wine; it wakes the loop for
  the next frame's time and for input, so nothing polls.
- **The game's sounds are IMA ADPCM** (WAV format 17, 4 bits, 22 050 Hz, mono; 9 of the 1 115
  entries of `Heroes3.snd` stereo): `SDL_LoadWAV` decodes them, and the spikes' own decoder (the
  standard IMA step and index tables, blocks of a 4-byte header per channel) gives the same samples
  (difference 0); SDL stops at the samples the blocks declare (22 374 for `PIKEATTK`, the spike's
  decoder ran on through the last block's padding: 22 975). `h3-assets` gets the decoder (pure,
  ADR-0020 point 2), the window adapter plays PCM.
- **MP3:** `nanomp3` 0.2 (MIT or Apache-2.0, pure Rust, a port of minimp3) decodes `COMBAT01` and
  `GRASS` to minimp3's samples (largest difference 1) as fast (0.1 % of real time on Windows);
  `puremp3` 0.1 is 4–8 times slower and wrong on `COMBAT01` (differences up to 20 657); `minimp3`
  itself is C.
- **Weight.** SDL built from source: about 3 minutes once on the sandbox's two cores (CMake, the C
  compiler; on Linux the X11, Wayland, ALSA and Pulse headers decide which backends it builds),
  cached afterwards; the statically linked exe is 6.7 MB against 1.5–1.8 MB, and it imports only
  system DLLs. Normal and build dependencies: 16 crates against 18–21 and 13–14.
- **Windows builds from the sandbox:** `rustup target add x86_64-pc-windows-gnu` (it needs
  `static.rust-lang.org`, which the user's egress setting now allows) and `mingw-w64`; SDL
  cross-compiles with them. The exes ran under Wine (SDL took Direct3D 9 and WASAPI there).

### 8.1 The window adapter (M2 T5a)

ADR-0024 built: `clients/crates/h3-sdl` and `h3-view window` (log: M2 T5a).

- **The crate.** `h3-sdl` (named for what it holds; the audio joins it with T5b) has every SDL
  call and all of the client's `unsafe` code. `Window::open` creates the window hidden, high
  pixel density and resizable, sizes it to the largest integer scale that fits the display's
  usable area less 40 points for the title bar (integers only: the pixels per point come from the
  window's own sizes), centres it, takes the renderer SDL picks, sets the logical presentation to
  integer mode and makes a streaming XRGB8888 texture with nearest sampling; the window is
  shown and synced first, so that a fullscreen start gives the renderer its final size. `wait` blocks in
  `SDL_WaitEventTimeout`, then takes every pending event: Esc, Space, →, R, F, V and C, the
  pointer in frame pixels (`SDL_ConvertEventToRenderCoordinates`), and exposure, size and scale
  changes as a redraw. `present_read` reads the whole output back before presenting (the logical
  presentation lifted while it reads, as the spike did). The hardware colour cursor is scaled by
  the frame's scale and made again when the scale changes (if SDL cannot, the window draws it
  instead; T5c); SDL's clock (`ticks_ns`) spares the
  binary a `std::time::Instant`.
- **The frame** goes up widened on the CPU (`Screen::xrgb`, the PNG's bit repetition; ADR-0024
  point 3's second form), so every renderer gets the same bytes.
- **The loop** (`bin/h3-view/window.rs`). The battle loads as `play` loads it (`battle.rs`, shared
  since T5a); every frame to the end names the checkpoint batch, with `CRDEFLT.DEF`. The pure
  `h3_view::clock` turns SDL's nanoseconds into the battle's milliseconds, with a pause, a seek
  (restart, a step) and the delay of a frame. The next frame, the first later change of the
  timeline whose draw list differs, is composed and widened as soon as the one before is shown;
  the loop waits for input until it falls due (at most 250 ms at a time) and presents it then. A
  frame overtaken by a later change is passed over and the later one shown. Past the end the
  timeline is timed 60 s further at a time (the outline's pulse and the fidgets go on); a battle
  that does not change for 10 minutes only waits for input.
- **The cursor** (*provisional*; the executable's cursor code not read, M3): `CRDEFLT.DEF`'s first
  frame (20 × 24, encoding 1: an arrow, a second arrow, an hourglass), its hot spot at the top-left
  corner, drawn with the battle's drawer without an outline (`h3_view::cursor`). For the hardware
  cursor it is drawn over black and over white: a pixel alike over both is opaque, one that kept
  both backgrounds transparent, a darkened one black with the darkening as its alpha (an
  approximation; this cursor has no shadow pixels).
- **In the sandbox** (Xvfb at 1920 × 1080 × 24, Mesa's software OpenGL ES 2, two cores): the frame
  at scale 1 ((1080 − 40) / 600 rounds down to 1). `--check 20` (every frame read back): 301
  frames, all exact, the black around included. `--measure 10`: a frame shows a median 7.0 ms
  after its time, p95 13.0, max 49 (the software renderer's present; at a forced scale 2, 17.7).
  Driven by `xdotool`: a pause holds the frame (no pixel differs over 1.5 s), → shows the next,
  R the opening, the drawn cursor at the pointer, Esc exits 0. Fullscreen needs a window manager
  (bare Xvfb ignores it; with `openbox`): output 1920 × 1080, the frame at (560, 240), every frame
  read back exact once the renderer is made after the window is shown and synced
  (`SDL_SyncWindow`); made before, the first frame was drawn at the old size.
- **On the user's machine** (Windows, 2560 × 1440 at 100 %; `clients/out/win/measure.cmd`, three
  unattended runs of 20 s, the cross-built exe): SDL took Direct3D 11, the window 1600 × 1200 at
  scale 2. `--check`: 302 of 302 frames read back exact. `--measure`, vsync off: a frame shows a
  median 0.4 ms after its time, p95 1.0, one outlier of 27 ms per run (the start); vsync on:
  median 0.4, p95 1.3. The delay is taken when `SDL_RenderPresent` returns, so with vsync the
  display may show the frame up to a refresh later (the spike, measuring the same way with
  another loop, saw 8 ms). The user, watching `play.cmd`: "looks good", but for the Angels
  flying under a corpse (fixed, §7.10).
- **Building:** SDL's Linux build stops without Xcursor's headers; the X11 set it uses is x11,
  xext, xcursor, xfixes, xi, xrandr, xss and xtst (the CI job installs them, with ALSA's and
  Pulse's for the audio backends); Wayland is left out when its headers are missing. SDL takes
  2.5 min on two cores, once per profile. The Windows build (`--target x86_64-pc-windows-gnu`)
  takes 6.7 min from scratch; `h3-view.exe` is 16.8 MB and imports system DLLs only.

### 8.2 The sound (M2 T5b)

ADR-0024 point 5 built (log: M2 T5b): the sound archives' decoder, a code reading of when the
battle's sounds play, and the window playing them.

**The archives.** `Heroes3.snd` (1 115 entries) and `H3ab_ahd.snd` (117) are a `u32` count, then
48 bytes per entry: a 40-byte name field (the stem, a NUL, `WAV` and the packer's leftovers,
`WAVVVWAV`, no NUL after them), the offset and the size. Every entry is a RIFF WAV of IMA ADPCM
(format 17, 4 bits, 22 050 Hz; 9 stereo, the `BATTLE` sounds among them; blocks of 512 bytes, 1 024
stereo, of 1 017 frames; a `fact` chunk before the data). The AB archive's 117 entries are all in
`Heroes3.snd` with the same bytes, so the viewer searches `Heroes3.snd` first (the game's own
order is not read). `h3_assets::snd` reads the directory (entries by stem, ignoring case),
`h3_assets::wav` decodes.

**The decoder** (pure, 100 % covered): 1 212 of the 1 232 entries end inside a block. SDL's loader
with its defaults drops such a cut last block (up to 1 016 frames, 46 ms each) and ignores the
`fact` chunk, whose count is the sound's own length (it plays up to 7 frames of padding after the
18 entries whose count ends inside their whole blocks). The decoder gives every complete frame up
to the `fact` chunk's count, as SDL does with the hints `SDL_WAVE_TRUNCATION=dropframe` and
`SDL_WAVE_FACT_CHUNK=truncate`; in a cut sub-block each channel's nibbles come from its own word.
The real-asset gate decodes every entry both ways and with SDL's defaults: all 1 232 equal SDL's
samples for the whole sound and SDL's defaults' as far as both go (88 507 083 frames against SDL's
defaults' 87 878 970); `<out>/gate/sounds.tsv` lists each. It also finds the asset tables' 743
sound names in the archives.

**When they play** (code reading; Miles Sound System through `MSS32.DLL`; nothing while the battle
is hidden, 0x46A4A0):

- *A sample* (`play_sample` 0x59A210, H3API's ClickSound): a WAV object carries a category
  (+0x28), a volume (+0x2C, 0 to 127) and a loop count (+0x30, 0 forever). The category picks a
  range of the sound manager's 14 sample handles (0x684AB8: 0, 1, 2–5, 6–13): the first whose
  sample is done, else the range's next in turn, whose sample is cut. The volume is scaled by the
  effects' setting, the music's by its own alike (0x5996C0: v × (s + 1) / 10 for a setting of 1 to
  10, 0 at 0).
- *A stack's eight sounds* (+0x170 to +0x18C: move, attack, wince, shot, death, defend, start,
  end), loaded by `stack_load_graphics` 0x43D9F0 from the creature table's prefix and `move`,
  `attk`, `wnce`, `shot`, `kill`, `dfnd`, `ext1`, `ext2`: a war machine (flag 0x40) has no move
  sound and its wince for attack and defend (the Ballista, 146, its shot for attack); only
  shooters, the Master Genies (37) and the Ogre Magi (91) have a shot; only the Devils, Arch
  Devils (54, 55), Vampires and Vampire Lords (62, 63) a start and an end. Each in category 3 (the
  eight creature channels), volume 64; the move sound loops, the others play once. `h3-view
  harvest` applies these rules: each creature of the asset tables names its `sounds`.
- *Moves*: `walk_step` 0x43F0B0 plays the start sound with group 20 and the move sound with the
  first cycle (0x43F19B, 0x43F1CF; the first step only); the last step plays the end sound and
  stops the move sound before group 21 (0x43F267; `stack_walk`'s early stop alike, 0x4421BC). A
  flight (0x4B4A40) the same around its cycles (0x4B4BD4, 0x4B4E3B).
- *Strikes and shots*: `shot_animation` plays the shot sound with the shot's first frame
  (0x43F424). `strike_animation` starts each stack's sound as its group starts (0x468F00 to
  0x468F8D, through `stack_play_sound` 0x43D540): the striker's attack, a struck stack's wince,
  defend or death by its reaction; a shooter going on with its shot group starts none.
- *The opening* (`battle_start` 0x462A20): `battle%02d.wav`, random(1, 8) − 1 (0x462AEF,
  category 2), plays while the screen is set up; after the first redraw the battle waits while it
  plays, at most 10 s of the game clock (0x46304B, `wait_for_sample`); then `combat%02d`,
  random(1, 4) (0x46305A), starts through `play_music` 0x59ACB0 (loop count 0, the effects
  ended first), and only then the battle's clocks (0x463093).
- *The music* (`music_thread` 0x59A840): `mp3\<name>.mp3` streamed. A looping track starts silent
  where that name last stopped in this session (a table of names and positions, 0x59AA82) and its
  volume rises by a tenth of the music volume every 100 ms (0x59AB5D, an f32 sum truncated: 12,
  25, 38, 50, 63, 76, 88, 101, 114, then 127 after 900 ms); a track played once starts at its
  volume, from its start. The tracks' names: 0x66C090.
- *A sound played by its name* (M3 T8; `play_sound_file` 0x59A470: the start sound, the tent's
  heal): its loader (0x55C720, the object 0x55C3C0) builds the WAV with category 0, volume 127
  and one play (the constructor 0x566DA0, from 0x55C48A and 0x55C6DC); `play_sound_file` sets
  category 2 and keeps the volume, so both play at 127, the music's level. A resisted spell's
  `MagicRes` the same (0x5A1F98; M3 T7b-1). A spell's own sound (the spell record's +4, after the
  resistance roll, 0x5A0331) goes through `play_sound_async` 0x59A590: loaded the same way (127),
  category 3, the creature channels, in a thread of its own (M3 T7b-1). Checked in the window
  with the `disk` driver on `resist`'s record: `Muckmire` (Slow's) and `MagicRes` once each,
  2 879 ms apart against the timeline's 2 900 (the driver's pacing, as at M2 T5b and T8).
- *The battle's end* (M3 T8): once an action's last frame has shown, `battle_winner` 0x4762F0
  finds the battle over (both sides tested, 0x465830, 0x4658B0), sets the manager's +0x132F8
  (0 from the battle's start, 0x463C85), the winner (+0x13D48, −1 none) and the retreat's and
  surrender's flags (0x6985A3, 0x697744: either side's), and calls `battle_end` 0x477470 at once:
  its last log line, the cursor, the whole screen darkened (0x44E5F0 over 800 × 600, shown at
  0x477843), then the result's window (built by 0x4702D0) and, before it runs (0x471B70), its
  tune through `play_music` with a loop count of 1 and the effects ended (0x477905, 0x4779B4), no
  clock step between: the tune starts when the next turn would have. The window shows the
  winner's side when the winner's player is a human here (0x4778B1, the side's player +0x54A8;
  the network's `remote_turn` aside), else the other side (a draw: the defender's,
  1 − (−1) = 2 taken as side 1); shown the winner, the tune is `win battle`, or `defend castle`
  for the defender of a town (the manager's +0x53C8, a siege or not; 0x470B68); shown the loser,
  `losecastle` for a town's defender, else `surrender battle` or `retreat battle` when either
  flag is set, else `losecombat` (0x470BFE to 0x470CFF; the index 0x695014 into the names
  0x6701A8). A tune played once starts at its volume (above). Recorded (`end`, below).
- *Not read*: spells', heroes', walls' and towers' sounds, the turn's, the adventure map's.

**The viewer.** `h3_view::anim`'s timeline carries the cues (`Cue`: a time, a stack, a `Sound`,
start or stop) at the frames the readings name; `play` writes them to `cues.txt`. `h3_view::sound`
holds the opening: the start sound and the music are drawn from the viewer's own C-library sequence
seeded with the battle's seed (*provisional*, as the fidgets; the executable draws from its rules'
generator), the fade-in's volumes, the volumes 127 and 64, and since M3 T8 the end's tune
(`end_tune`, by the outcome, the snapshot's human sides and a town, none yet: sieges are not drawn;
`Timeline::ended`, the end of the command whose events end the battle). The window
(`bin/h3-view/audio.rs`): every sound the battle's stacks name and the start sound are decoded when
the battle loads, the music by `nanomp3` (i16; the battle's 44 clips and a 3-minute track in 0.4 s
in the sandbox); `h3-sdl` opens one device and 10 streams: the music's, the start sound's and 8
creature channels, each a free one or the next in turn, as `play_sample` takes them. A one-shot is
flushed; the move sound and the music are queued a second ahead and topped up (no flush, so no
seam). Volumes as SDL gains: the music's fade-in by its steps, the creature sounds at 64/127, the
start sound and the heal at 127/127 (since T8; *provisional*: both settings equal, no settings yet).
At the battle's end the window ends every effect and plays the tune on the music's stream once, at
127, from its start; Home brings the battle's music back. From the battle's start the window plays
the start sound over the first frame with the clock held, then starts the music and the clock (a key
that moves the battle ends the wait); Space pauses the device with the battle (the fade-in holds
with it), R and → stop the creature sounds and go on from the cues at the new time (→ keeps the next
frame's, cued as the frame stepped to shows), M and E switch the music and the creature sounds,
`--mute` plays nothing; a sound that does not decode costs that sound, as a missing one (T5c, the
reviewer).

**Checks.** The gate (above); the cues' unit tests (each with its group's first frame); in the
sandbox under Xvfb with SDL's `disk` audio driver (it writes the mix to `sdlaudio.raw` in the
current directory): `--check 25` with sound, all 293 frames read back exact; the start sound 5.8 s
(`BATTLE01`, seed 1), then the music fading in; with the music switched off (M), the effects' onsets
in the mix within 10–100 ms of their cues' times (onset detection on 10 ms windows; overlapping
sounds hide some). M3 T8: `end` (`sequences.toml`) recorded in the original, the battle decided at 3
730 ms (the harness's `end` line, the manager's +0x132F8; the whole screen changed in the same
step); the viewer's timeline as recorded, its `end` at 3 730. Its record in the window (`--check
20`): the tune (`Win Battle.mp3`, hot seat's human winner) alone in the mix, whole and once, 3 692
ms after the music's start (the disk driver's pacing, as the effects'), then silence.

## 9. The string catalogue (M3 T4b-1)

The user's decisions: every text the client shows comes from a key → string catalogue, easy to
translate, language packs swappable (2026-10-13); the packs are nested JSON (2026-10-19); the
original's text and translations of it are never committed, a player may load a pack of their
own (CLT-Q7, 2026-10-19). Built as `h3_view::text` (pure), `clients/text/keys.toml`, `h3-view
text` and the gate's check; the formats in `clients/README.md` ("The string catalogue").

### 9.1 Keys and the original's rows

- **Two files, two jobs.** The key catalogue (TOML, like the asset tables: identifiers and
  numbers) says which keys exist, which arguments their messages may use, and where the user's
  text tables hold the original's forms; the packs (JSON) hold messages. The original's own pack
  is built from the user's tables at run time, so the original's text and its language come from
  the user's installation (a localized one gives its language) and nothing of it enters git.
- **Rows** are records counted from 0 with the header rows, as `h3-formats`' parser reads them (a
  quoted field with line breaks is one record): `GENRLTXT`'s record n is VCMI's
  `core.genrltxt.<n − 1>` (its parser skips the header) and `installation.md`'s 1-based "line
  n + 1". The executable's own indexing is the same: its row n is record n (read in T4b-2a,
  §9.4).
- **Specifiers.** The original's rows use `%s`, `%d` and `%%`; the key names each specifier's
  argument in order (`printf`), and its declared kind must match (`text` for `%s`, `number` for
  `%d`), so a row and a key that disagree fail the gate. The forms of one key may name different
  arguments (`battle.log.perish`: "one …" without the count, "%d … perish" with it).
- **Plural rows.** Where the original has a singular and a plural row, the key has the forms
  `one` and `other`; the original's pack picks `one` for a count of exactly 1 (the executable's
  rule; which number picks it per line: §9.4). A
  creature's names are a family over the asset tables: `crtraits.txt`'s non-blank records from
  record 2 are the original ids in order (the harvesters' reading), its first two columns the
  singular and plural; the asset tables' `[[creature]]` carries `original` (the comment `#
  original id` before T4b-1).
- **Contexts.** A key may have forms by a context (`word.possessive`: `male`, `female`, the
  original's "his" and "her" rows), the argument `context` picking one; i18next's context
  suffix.
- **Checked:** the gate reads all 155 keys (10 keys, 145 creatures' names) and their 306 forms
  from `GENRLTXT.TXT` and `CRTRAITS.TXT` (the base archive's copies win, §3.1), every specifier
  as named; `h3-view text` formats the damage line and the perish line from them as the rows say
  (one log entry: the perish row starts with a line break).

### 9.2 Packs and lookup

- **The JSON** follows i18next's v4 format (nested keys, `{{name}}`, `_<category>` and
  `_<context>` suffixes) so that translation tools that read it (Weblate, Crowdin) can edit a
  pack; nesting, formatting functions and `$t()` references are left out. A placeholder's name
  starts with a lower-case letter and goes on with lower-case letters, digits and `_`, so the
  original's highlight braces (`{Tactics Phase}`, `{%s}`: `TextDraw`'s markup, §3.6) stay text
  in every pack and in the original's.
- **Language** is a pack file's name (`de.json`, `pt-BR.json`; its primary subtag picks the plural
  rule); a language without a rule is refused. A pack is checked against the keys: an entry no key
  names, or a placeholder its key lacks, is listed (`h3-view text --pack`).
- **Lookup** goes through the packs by priority (the player's, the client's own, the original's)
  and in each tries `<key>_<context>_<category>`, `<key>_<context>_other`, `<key>_<context>`,
  `<key>_<category>`, `<key>_other`, `<key>`: a pack that lacks a form falls back within itself
  before the next pack is asked, and a key no pack has shows as itself. The client has no strings
  of its own yet (lobby, accounts, options come later): its pack, `clients/text/<lang>.json`, is
  committed when the first one appears, in any language.
- **Values:** a number is written in decimal digits with `-` before a negative one (as `%d`); a
  placeholder without a value stays visible as `{{name}}`.

### 9.3 Plurals and code pages

- **Plural rules** are CLDR's cardinal rules (`common/supplemental/plurals.xml`, the `main`
  branch read on 2026-10-19) for whole numbers only, the battle's counts: `one`/`other` (en, de,
  nl, sv, da, nb, fi, et, hu, tr, el, bg), with `many` for a non-zero multiple of a million (it,
  es, ca), `one` for 0 and 1 with that `many` (fr, pt), Polish, Russian and Ukrainian (`one`,
  `few`, `many`), Czech and Slovak (`one`, `few`, `other`), `other` only (zh, ja, ko). CLDR's
  `zero` and `two` belong to languages without a rule here. Each rule is tested on CLDR's
  `@integer` samples. The `intl_pluralrules` crate (CLDR 37) was not taken: older than the data
  read, and a dependency for twenty lines.
- **Code pages.** Strings stay Unicode until drawn. The installation's fonts draw Windows-1252
  bytes (§3.6), so `encode_1252` gives the bytes and lists the characters it has none for (drawn
  as `?`): a pack in a language whose letters 1252 lacks (Polish, Russian, Czech) needs fonts of
  its own, from a localized installation or a modern renderer (a graphics option, later).
  `h3-formats` decodes the text tables as 1252 with every byte kept (undefined ones as their C1
  controls), so a localized installation's tables in another code page (1250, 1251) can be
  re-decoded without an engine change; that and its fonts wait for such an installation.

### 9.4 The battle log's lines (M3 T4b-2a)

T4b-2 split with the user on 2026-10-19 (the line drawn left as T4b-2b). Read: `battle_log_add`
(0x472E90), the strike's line (`battle_log_damage` 0x469A90, H3API's `ReportDamageDone`) and what
its strike and shot callers pass, `creature_name` (0x440100), and the lines of wait (0x479918),
defend (0x479758), morale (0x464AD5, 0x464CDF) and luck (0x44187A, 0x43F998). Built as
`h3_view::battle_log`; `h3-view text --log` prints sample lines from the user's tables, and the
gate composes them.

- **Rows.** The executable reads `GENRLTXT.TXT` through `general_text` (0x6A5D5C, its rows at
  +0x20): its row n is our record n, from 0 with the header, checked on the battle log's fifteen
  rows and the clone limit's against the user's file. The keys lose *provisional*;
  `battle.log.destroyed` (row 668) is new, and the bad luck key (row 45) is gone: no battle code
  writes that row, as the original has no bad luck (the protocol's `lucky` is good luck only).
- **Names.** A stack's name is its creature's singular for a number of exactly 1, else the plural,
  0 included (0x440100 and its inlined copies). The number that names a stack also picks its
  row's form: the striker's for the damage row, the kills for the perish row.
- **A strike** (0x469A90): the damage row by the attackers' number (their name, the damage); with
  kills above 0 the perish row appended, whose text begins with a line break: row 668 for a war
  machine (creature flag 0x40, its name by the kills), else rows 379 and 380 (the kills, the
  name by them), with the word for creatures (rows 43, 44, by the kills) when the caller names no
  target. Its callers: a strike or shot at one target names it (`melee_strike` 0x441C5D, the
  shot 0x43FDB4); a breath's stack behind of another creature adds its damage and kills and
  names no target, while one of the target's own creature is left out of the line (0x441B9B: the
  line reports the target's hit alone; copied, the M3 review list); a multi-headed striker's
  loop (0x440310, its line at 0x4404ED; `melee_strike` writes none for it, 0x441B95) and the
  area shots (0x43FC4F, 0x44003F) add every hit up and name their creature when all share one,
  else none. A hidden battle writes no line. The other callers (spells, the moat, fire walls,
  land mines, turrets, Fire Shield, the striker's effects) come with T6 and T7.
- **Turns.** Wait: rows 137 and 138 by the number, with the name; none in the tactics phase.
  Defend: rows 121 and 122 by the number, with the name and the gain max(1, defense × 20 / 100)
  (integer, the stack's defense skill); a war machine writes none. Morale: good row 34, bad row
  35, luck: good row 46 after a strike's or a shot's lucky roll; each with the name by the
  number.
- **Entries** (0x472E90 with its second argument 1): a text without a line break is one entry;
  with one, its first break becomes a space when the text then takes fewer than two lines of the
  small font (`smalfont.fnt`, §3.6; `text_line_count` 0x4B5820) at the log item's width, else
  the text is two entries split there. The number of new entries (at most 2) sets the first line
  the log shows. With the second argument 0 the text goes to the **status line** instead, unless
  a log entry came less than 3 000 ms ago (`clock_ms` 0x4F82E0) and the third argument is 0.
  Built: `battle_log::entries`, the line count as an argument; the log's item: §9.5; the
  scrolling and the panel are T6's.
- **The caller's.** Which event writes which line, when none (a hidden battle, a wait in the
  tactics phase, a war machine's defend), how the protocol's events (`Struck` once per target,
  `Defended` without its gain) become a strike's hits and a defend's gain: T6's event mapping.

### 9.5 The log's item in the panel (M3 T4b-2b-1)

T4b-2b split with the user on 2026-10-19 (a strike's line against a capture left as T4b-2b-2).
Read: the battle's dialog (`combat_dialog_create` 0x4721D0, the manager's +0x132FC), its panel
(`battle_panel_create` 0x46BC30), the log's item (0x46BCBE), the text items (0x5BC280, 0x5BC760)
and their drawing (0x5BC7F0, 0x5BC5F0), and how the shown entries become the item's text
(`battle_log_show_from` 0x472C30). Built: the layout's `[log]`, the draw `Draw::Log` after the
battlefield (`scene::panel`), drawn by `compose`.

- **The panel** is a panel of the dialog at (0, 556), 800 × 44, over `cbar.pcx`; outside the
  tactics phase 0x46BC30 builds it (in the phase 0x46C050, not read). Its log's item is a text
  item with a picture behind it at the panel's (214, 7), so the screen's (214, 563), 400 × 32:
  `smalfont.fnt` over `cRollovr.pcx` (400 × 32 itself), colour argument 1, alignment 1 (each
  line centred, from the box's top), no text at first. The log's two scroll buttons
  (`ComSlide.def`) sit right of it; the rest of the panel is T6's.
- **Drawn:** the picture opaque from its corner, at most the item's size of it (the picture is
  that size), then `TextDraw` (§3.6) of the item's text in its box with that colour (+2 with the
  item's flag 8, not met yet) and alignment; Windows-1252 bytes of the text (§9.3).
- **The text** is the shown entries (0x472C30): the first shown, then a line break and the next if
  there is one, so the box's two lines hold two entries; an entry too wide for 400 pixels breaks
  and pushes the rest below the box, where nothing is drawn. The status line (§9.4) sets the same
  item's text. The dialog's other item at the same place (+0x4C, over `cRollovr.pcx`) is
  taken for the edit line of a message (not read): hidden, and while it is active the status
  line is left alone.
- **Checked:** the capture set compares the log's item since T4b-2b-1 (`h3cap`'s mask
  `panel_beside_log`: the panel but the item, the layout's `[log]`): at every opening and later
  frame the original shows the picture without text, as ours, all 31 situations exact. The
  harness parked the game's cursor at (420, 580), inside the item; it parks it at (618, 593) now,
  in the panel right of the item and below the scroll buttons, where it covers no probe and
  hovers nothing that shows a hint. (Since T6a the item is the panel's text item 2005, drawn as
  `Draw::Text`, and the whole panel is compared: §9.7.)


### 9.6 A strike's line against a capture (M3 T4b-2b-2)

T4b-2b-2 narrowed with the user on 2026-10-19 to one line in a still: when the line appears
during a strike, several entries and which two show, and the scroll buttons are T6's. Built:
`battle_log::shown` (0x472C30), `h3_render::text::line_count` public (0x4B5820, the log's
measure), and in the adapters `h3-view`'s `log` module (*provisional* until T6's event mapping):
the catalogue of the original's text, the line of a record's first strike, the item's text after
one line entered an empty log; `h3-view sequences --frames` draws it from a sequence's `log_at`.

- **Why a fixed damage.** Every strike of the recorded sequences rolls its damage, so the
  engine's numbers, and its line, are not the original's. Only the Angels and Archangels (50),
  the Nagas (20) and the Naga Queens (30) have a fixed damage in the base pack. The sequence
  `strike` (`seq-strike.toml`, hot seat): the Archangel (speed 18, the native grass's +1 attack,
  defense and speed) flies beside the Pikemen and strikes them, attack 32 against defense 12
  doubles its 50, and the 100 damage kill all ten (100 hit points): no retaliation, so one
  `battle_log_add`. Three factions with the Archangel's +1 make the attacker's morale 0, two the
  defender's: nothing rolls, the engine's record is the original's battle (its timeline as
  recorded, 57 changes).
- **The original's line:** the damage row by the attacker's number of 1 and the perish row by the
  10 kills, the perish row's line break joined to a space (the line fits 400 pixels of
  `smalfont.fnt`): one entry, on the box's first line, centred. The engine's `Struck` (damage 100,
  kills 10) composes the same text from the user's tables.
- **When it shows:** the frame check keeps a frame where the battlefield changes; the first kept
  frame that shows the line is the defender's Goblins' turn at 3 730 ms (the strike's animation
  ends at 3 330, the frame before at 3 630 still shows the status line). Until then the item shows
  the status line's hint over the Pikemen, `Attack Pikemen`, set while the mouse passed over them
  for the click (the hint is T6's). The sequence's `log_at` takes 3 730 as observed; the frame
  check compares the log's item from then on (`panel_beside_log`), the whole panel before.
- **Checked:** the log's item exact in all six frames from 3 730 ms (1 166 pixels of text). The
  frame check's other differences are count boxes (T5): the Pikemen's box where the original last
  drew it while the Archangel lands (2 330 to 2 730 ms, with stale pieces), the Archangel's box
  moved in front of it at the Goblins' turn once the Pikemen's hex is free (from 3 730 ms: 324
  pixels, 6 redraw artifacts); ours places both by the opening's stacks.

### 9.7 The panel and the dialog items (M3 T6a)

T6 split with the user on 2026-10-20 (the log over time T6b; the status line, the cursors and the
mouse over a stack T6c), who asked for reusable UI components: the executable draws every
screen's controls with a few kinds of dialog item. Read: the panel's base (`battle_panel_base`
0x46B610) and `battle_panel_create` (0x46BC30) in full, the picture item (0x4502D0, its drawer
0x450450, its player colours 0x450520), the DEF button (0x455EF0, its drawer 0x456940, the frame
drawer 0x47BF00; the button with a callback 0x456D30), how a control's state is set
(0x5FEED0, 0x5FEF60, 0x5FE7C0 into the items' message handler 0x5FE4F0), `battle_controls_state`
(0x4782D0) in full and 0x46BF90. Built: `h3_view::ui` (the item kinds, their states and draws;
the layout's `[panel]` lists the items), `h3_view::panel` (the controls' states at a turn's
start), `Draw::Picture`, `Draw::Button` and `Draw::Text` (the log's item since T4b-2b-1 as a
text item) in `compose`.

- **The items.** A picture (H3API's `H3DlgPcx`) is drawn from its corner with index 0 transparent
  (0x44FDB0 with its last argument 1), at most the item's size of it; 0x450520 gives it a player's
  colours (entries 224–255, as a flag's, §4.5). A DEF button (`H3DlgDefButton`, 281 callers; the
  callback and caption buttons share its drawer) draws one frame of its DEF's group 0 at its
  corner by the plain drawer, runs skipped: the selected frame (+0x40, 3) when its state has bit
  0x10 and not bit 1 (pressed); else the disabled frame (+0x3C, 2) with bit 8 or 0x20; else the
  click frame (+0x38) when pressed; else its frame (+0x34); a frame beyond the group's count is
  frame 0; nothing without bit 4 (shown). The constructor takes the frame and the click frame and
  sets 2 and 3; a caller may change them. A text over a picture (`H3DlgTextPcx`): §9.5.
- **States.** `AddControlState` and `RemoveControlState` (0x5FEED0, 0x5FEF60) and an item's own
  `SendCommand` (0x5FE7C0) send a message of type 0x200 whose subtype 5 sets and 6 clears state
  bits of the item with the control id; the value 0x1000 stands for bit 8, disabled (0x5FE560,
  0x5FE608).
- **The panel** (800 × 44 at (0, 556)), in the executable's order: the picture `cbar.pcx` (id
  2000) in the colours of the player of the side the manager holds then (the game's player when it
  has none); the 48 × 36 buttons at y 5: Surrender (2001, `icm001.def`, x 54), Retreat (2002,
  `icm002.def`, 105), the options (2003, `icm003.def`, 3), Auto Combat (2004, `icm004.def`, 156),
  the spell book (2008, `icm005.def`, 645), Wait (2009, `icm006.def`, 696), Defend (2010,
  `icm007.def`, 747); then the log's item (2005, §9.5) and its scroll buttons, 18 × 17 of
  `ComSlide.def` at (624, 5) and (624, 24): up (2006) frames 0 and 1, down (2007) 2 and 3, each
  disabled showing its click frame. Built, Auto Combat is disabled when both sides are human and
  the scroll buttons are disabled. The hotkeys (S, R, O, A, C, W, D and space, the arrows) and the
  hints are the window's (T6c).
- **A turn's start** (`battle_controls_state`, from every stack's activation): for a side whose
  player is a human on this computer, the picture takes that player's colours; Surrender needs a
  hero on both sides and is refused to a town's defender; Retreat needs the side's hero and, for
  a town's defender, a Stronghold's Escape Tunnel; the options enabled; Auto Combat no longer
  selected, disabled when both sides are human; the scroll buttons enabled (whatever the log
  holds: the log does not set them, the panel's vtable +8 is an empty function); the spell book
  needs the side's hero to wear one and not to have cast this round (+0x54B4; +0x13D74, not read,
  overrides it); Wait outside the round's wait phase; Defend. Outside the tactics phase only; in
  it the dialog was built with the spell book, Wait and Defend disabled and its own panel
  (0x46C050, not read). For any other side 0x46BF90 disables the seven buttons and leaves the
  picture's colours and the scroll buttons as they were (the computer's turn: buttons dimmed,
  §6.2). With Auto Combat on, a stack that plays itself dims the panel too, and a human's Auto
  Combat shows selected.
- **The viewer** (`h3_view::panel::shown`, *provisional* where marked): the turn's start from the
  observation (each side's `human`, hero, spell book and casts, the wait and tactics phases, the
  town) and the acting stack's controller; a record names no player colours, so red and blue
  (CLT-Q6). Not modelled: what a click, an animation or the log's scrolling change, the panel kept
  from an earlier turn (T6b), the side the panel is built for (taken as the acting side), the
  tactics phase's panel; Auto Combat switched on since M5 T4c1 (§7.22). The situations' and sequences' scenarios name
  their human sides (`human`), as the original's maps have them; `h3-view situation` refuses a
  scenario whose `human` is not its map's.
- **Checked:** the capture set compares the whole panel (its `panel_beside_log` masks gone): all
  31 situations' 57 frames exact, the human's opening (red, the spell book disabled: the maps'
  heroes carry none; the scroll buttons enabled with the log empty) and the smoke battle's hot-seat
  opening on Blue's turn (blue). The game draws its own cursor where the harness parks the mouse
  (618, 593): 35 pixels the viewer does not draw (the cursor rules are T6c's), masked as `cursor`
  (`CRDEFLT.DEF`'s 20 × 24 canvas there). The ten sequences compare no more of the panel than
  before (T6b): `drawn` now digests only what the comparisons see, so they stayed the same.

### 9.8 The panel and the log over a battle (M3 T6b)

T6b as one increment (the user, 2026-10-20, offered the panel and the log as two). Read: what sets
the controls' states while a battle runs (the callers of 0x5FEED0 and 0x5FEF60 in the battle's
code, the action dispatcher's head 0x478D80, `battle_controls_state`'s tail), the log's state
over time (`battle_log_add` 0x472E90's two paths, `battle_log_settle` 0x472BF0 and its callers,
`battle_log_show_from` 0x472C30 and its callers, the scroll 0x472DB0, 0x472DF0, 0x472E40, the
panel's set-text 0x46BF50, the mouse posted again 0x4ECC00) and where each line is written
(`melee_strike`, `stack_shot`, `first_aid_tent_action`, `round_start`). Built: the timeline's
`log` (`battle_log::Logged`), `battle_log::Log`, `word`, `texts`; `panel::turns`, `panel::kept`;
`scene::Panels` for the frame check, the stills and the window.

- **The panel through a battle.** The battle's code sets control states only as the dialog is
  built, at the tactics phase's start (Wait, 0x462EE0), at each stack's activation
  (`battle_controls_state`) and when Auto Combat is switched (0x474E73, 0x474ED1): an action and
  its animation leave the panel as its turn's start set it, and the Wait and Defend hotkeys are
  the buttons' own (they are not in the battle's key switch 0x4758FB; not read further). A turn
  no human plays (0x46BF90) dims the seven and leaves the picture's colours and the scroll
  buttons as the last human turn set them: once a human's turn outside the tactics phase enabled
  them, they stay enabled (`melee`'s computer turn: 539 pixels until this was built). Each turn's
  battle is folded from the events (`panel::turns`: the wait phase is the stack having waited
  this round, the heroes' casts this round, the tactics phase); `panel::kept` keeps the colours
  and the scroll buttons; the panel at a time is its last turn's start's.
- **The log's state** (the combat dialog's +0x54 to +0x6C; `h3mem`'s `battle_log` reads it): the
  entries, the new entries' count +0x64, the first shown +0x68, the time it last showed them
  +0x6C. An entry (second argument 1) sets +0x6C, adds its entries to the new count (at most 2)
  and shows the log from the first new one. The status line (0) replaces the item's text unless
  the new count is above 0, +0x6C less than 3 000 ms ago and the third argument 0; it leaves
  +0x64 and +0x6C. **Settling** (0x472BF0): new entries shown 3 000 ms ago or more are new no
  longer (+0x64 = 0) and the status line is emptied; as each action starts (0x478DB2), at a human
  player's turn start (0x47884C) and at the actions' re-entries (0x479195, 0x479D6B; also
  `move_and_attack` and two click handlers, not read). Showing from an entry past the last does
  nothing; it sets +0x68, the text and +0x6C. The scroll buttons and the arrow keys (72, 80)
  show from the first shown ∓ 1, at most the entries' count less 2 and at least 0.
- **On the screen at once:** the panel's set-text (0x46BF50) redraws the item and shows its
  rectangle (state 0x4000, 0x5FE5B5), so a line or a status line shows the moment it is set.
- **The mouse at a turn's start:** a human player's turn posts a mouse move at the cursor
  (0x4ECC00), so the status line becomes the hint where the mouse rests, over nothing an empty
  one (observed: `walk`'s hover hint `Move Champions here.` gone at the next human turn, no line
  written). The viewer has no mouse: at a human's turn it sets the status line empty, under the
  3 000 ms rule (*provisional*: the hints are T6c's). The hover's own hint ignores the rule
  (`behind` at 7 144 ms: `Attack Silver Pegasi` over two lines 0 ms old; T6c).
- **Which event writes which line, and when** (the timeline's `log`, `battle_log::word`):

  | Happening | When | Its line |
  |---|---|---|
  | a melee strike, a retaliation (`Struck`, not ranged) | after its animation, `strike_animation`'s loop (0x441C5D) | the target's (`Spread::Target`, a breath's stack behind added), a multi-headed striker's every hit's (`Around`, 0x4404ED) |
  | a shot at one target | after its reactions' loop (0x43FDB4) | the target's |
  | an area shot | as its reactions start (0x43FCE0, 0x4400DC) | every hit's |
  | good luck (`lucky`) | before the strike's or shot's animation (0x4418A2, 0x43F9C0) | row 46 |
  | Wait, Defend | at the action (0x479A88, 0x479892) | rows 137/138, 121/122; none in the tactics phase, none for a war machine's defend |
  | the First Aid Tent's heal | between its sound and its magic animation (0x478D0A) | row 415: the tent and the patient each by its number, the hit points (`battle.log.heal`) |
  | a round's start | outside the tactics phase, before the next activation (0x4761E9) | row 413 (`battle.log.round`); none at the opening |
  | good morale (M3 T7b-2b-2b) | after its animation, once the erasing redraw shows (0x464AD5) | row 34 |
  | bad morale, Fear (M3 T7b-2b-2b) | at the pick, after the sound, before the animation (0x464CDF, 0x464EFF) | row 35; row 729 (`battle.log.fear`): the frightening creature by their number on that side, then the frozen stack |
  | a hero's spell on one stack (M3 T7b-1, T7b-2a) | a timed spell's after its effect, before its animation (0x5A109F); Cure's and Dispel's after their animation (0x5A1816, 0x5A1605) | row 196 (`battle.log.cast-on`) |
  | Disrupting Ray (M3 T7b-2a) | after its animation and effect (0x5A1B43); no cast line | row 92, the defense it took (`battle.log.disrupted`) |

  Not built: the lines the engine's other events write in the original (regeneration, row 372 at
  0x44707B; mana drain, 374 and 375 at 0x4657AD; the spells, Death Stare, Life Drain, Fire
  Shield, the moat, fire walls, land mines, turrets: T7's). The striker is named by its number
  before the strike (a retaliator's after the strike's kills), a defend's gain by the stack's
  defense then (the snapshot's, lowered by Disrupting Ray since M3 T7b-2a; other changes to it,
  Stone Skin's, a Behemoth's, not followed: *provisional*).
- **Checked:** the line times against the original's memory: `shot`'s line at 1 588 ms, `melee`'s
  at 5 266 and 6 466 (the unit tests pin them), `behind`'s, `shots`' and `machines`' sequences of
  lines, the round's line and the settling as modelled. The harness keeps a frame where the
  panel changes too (the parked cursor aside: no new frame in the ten, the panel's changes fall
  on kept ones) and reads the log; the frame check compares the whole panel, its log's item
  where the original's text is the viewer's, else, where its pixels differ, listed (` log`). All
  ten sequences captured afresh: every count as recorded (896 frames: the panel's buttons,
  colours and scroll buttons as the original's in all), the log's item exact in the 228 frames
  that show the viewer's text (`strike`'s line from 3 730 ms, `machines`' defend, shot and heal
  lines), listed in 668: the hover's hints (T6c) and lines whose numbers the engine rolled
  (`behind`'s second shot also kills one Pegasus in the engine, none in the original). The memory
  can lag the screen by a step as a turn starts: there the pixels agree and nothing is listed.
  The set (31 situations, 57 frames) exact.

### 9.9 The mouse's still: the cursor and the status line (M3 T6c-1)

T6c split with the user on 2026-10-20 (the still T6c-1, over time T6c-2). Read: the battle's
mouse-move branch of `battle_buttons` (0x4753A4 to 0x4758E1), `screen_to_hex` (0x4647A0) and the
heroes' rectangles (0x4626A0, 0x4626D0), the neighbour table (0x4640A0), `battle_cursor`
(0x476490) in full, the attack's slots (`cursor_attack_slots` 0x474690, `cursor_attack_from_hex`
0x474A00) in full, `battle_status_line` (0x492840) for the cases below, `hex_distance` (0x469670),
the mouse manager's `SetCursor` (0x50CCA0), its frame drawer (0x50D8B0) and position (0x50CD90,
its hot spots 0x67FF50), the panel's hints (`SetHints` 0x5FE840 from 0x6A6968 on; `Help.txt`'s
loader 0x5B98B0). Built: `h3_view::mouse` (`spot`, `neighbour`, `distance`, `Legal`, `Kind`,
`Hint`, `Pointer`, `pointer`, `slot`, `slots`), `Draw::Cursor` (the even x under
`Bugs::Original` only), the marked stack's outline
(`Moment::marked`), `ui::Item::hint` and `Dialog::item_at`, the layout's `[mouse]` and
`outline_mouse`, the keys `battle.hint.*` and `battle.control.*`.

- **Where the mouse is** (`screen_to_hex`): first four rectangles, bounds inclusive: the
  attacker's hero (x 0–57, y 16–127) is "hex" 0xFC, the defender's (741–799, 16–127) 0xFD (both
  static), two siege ones 0xFE and 0xFF (not read). Else sx = x − 14, sy = y − 86, row = sy / 42,
  on an even row sx −= 22, column = sx / 44 (C division); outside 0 ≤ sx < 748, 0 ≤ sy < 472
  none (−1). In a cell's top 10 pixels a point above the hex's slanted edge, sy mod 42 <
  |sx mod 44 − 22| / 2, is the row above's: row − 1, and on the left half (sx mod 44 < 22)
  column − 1 when that row is even, on the right half column + 1 when it is odd. A row outside
  0–10 or a column outside 0–16 is none; else row × 17 + column.
- **The mouse move** (0x4753A4): outside the battlefield (x outside 0–799 or y outside 0–555)
  the marked stack is unmarked (redrawn), the dialog's items get the message (their hints,
  below), the cursor becomes the combat cursor's frame 6 and the mouse's hex (+0x132D4) none.
  Inside, on a new hex the hex's stack is marked (`battle_mouse_stack`, §7.13; its outline is the
  mouse's, entry 0x70, unless it is the active stack, whose 0x60 wins: 0x43E23A to 0x43E261),
  the kind is `battle_cursor(hex)` (+0x132E0), the cursor set by it, and the status line written
  for it (`battle_status_line`); on the same hex only an attack's slot is computed again, so a
  turn's posted mouse move (0x4ECC00) over the same hex writes no hint (T6c-2). The army info
  windows (+0x132FC's six, shown when "Combat Army Info Level" 0x6987C8 is set) stay hidden at
  its default 0, which the harness keeps.
- **The kind** (`battle_cursor`, the acting stack's view; the battle lane read its rules,
  `installation.md` §7.14, §7.21): none (−1) or a side column (column 0 or 16) 0; a hero's
  rectangle with its hero 4 (the acting side's) or 21 (the other's), without one 0; in a network
  game on a turn not played here 5 over a stack, 6 elsewhere (0x69954C, inferred); sieges'
  catapult targets 16 and the arrow towers 22 (not modelled). A stack's hex: 5 (query) in the
  tactics phase (+0x13D68) and over the acting stack itself; a Faerie Dragon's cast 20 (not
  modelled); the acting side's own stack 17 for the First Aid Tent over a damaged patient, else
  5; an enemy (its side +0xF4 not the acting side): 3 (shoot) if the stack can shoot now and is
  the arrow tower or the shot is not halved, 15 (the broken arrow) when `shot_wall_test` or
  `shot_full_damage_distance` says halved (the latter from the shooter's front hex by its
  facing now, not turned: 0x4767E6); without a shot 7 (attack) if `cursor_path` reaches it (3 for
  the Ballista), else 0. An empty hex: the Catapult's wall targets 16 (sieges), a caster's
  empty-hex spell 20 (not modelled), else 2 (fly, the stack's flag 2) or 1 (move) when the reach
  map marks the hex as a position (+0x20E) or as a two-hex stack's front hex (+0x20F,
  0x4B2EF4), else 0. The viewer asks the legal commands the server lists for what the reach map,
  `stack_can_shoot_now` and the tent's test answer (`mouse::Legal`); the cursor's own distance
  test it ports (the shooter's `no_distance_penalty` for the bows and the Sharpshooters).
- **The cursor** (0x47587C, 0x474B85): the kind's frame of the combat cursor (`SetCursor(frame,
  2)`, `crcombat.DEF`), except 7, whose frame is its slot's, 20 (frame 0 of `crspell.DEF`, type 3,
  animated: not read) and 21 and 22 (frame 5). The types' DEFs (0x67FF38): 0 `crdeflt.DEF`, 1
  `cradvntr.DEF`, 2 `crcombat.DEF`, 3 `crspell.DEF`, 4 `artifact.DEF`. Each type's frame has a hot
  spot (0x67FF50 + (type × 144 + frame) × 8, x then y): the cursor's canvas is drawn with its top
  left at the mouse less the hot spot, the mouse's x first rounded down to even (0x50CE04, from
  `GetCursorPos`; the game's logic takes the message's x as it is). **That rounding is a drawing
  bug** (the user, 2026-10-20): the client draws the cursor at the mouse, and only the
  comparisons with the captures round it (`h3_render::Bugs::Original`, the compositor's; the
  epic's Decisions). The frame is drawn by the
  plain drawer with runs skipped (0x47BEB0 → 0x47C570, flag 1) into a surface first filled with
  the colour key (the screen's green and blue masks: 0x07FF in 5-6-5), which the screen's cursor
  overlay leaves out.
- **The attack's slot** (`cursor_attack_from_hex`): dx = x − (44 × column + 36, 22 more on an
  even row), dy = y − (42 × row + 112), r = |dx| / |dy| as f32 (100 when dy = 0), against the f64
  constants 0.58 and 1.73 (exact in integers here: 50 |dx| against 29 |dy|, 100 |dx| against
  173 |dy|; no point of a hex is at either). Clockwise from the top: dx ≥ 0, dy < 0 slots 0 (r up
  to 0.58), 1, 2 (above 1.73); dx ≥ 0, dy ≥ 0 slots 3 (r at least 1.73), 4, 5 (below 0.58); then
  6 to 8 and 9 to 11 on the left the same way from the bottom up. Each slot has a from-hex
  (+0x133FC, the attack's +0x132D8) and a frame (+0x133CC) from `cursor_attack_slots`; the
  cursor changes only with the frame.
- **The slots** (`cursor_attack_slots`): slot s looks at the hovered hex's neighbour s / 2 (the
  manager's table at +0x13468, built by 0x4640A0: up right, right, down right, down left, left, up
  left; none off the field or in a side column, and a side column's hex has only its inner three)
  as the acting stack's position: taken when the reach map marks it (+0x20E), the stack fits there
  (`stack_fits_at_hex`) and none of its hexes is a stop (a seen pit); the frame, a sword pointing
  at the hex, is then 7 + (s / 2 + 3) mod 6. A two-hex stack also tries the position beside it,
  the neighbour − 1 facing right, + 1 facing left (+0x44); with either taken the frame is the
  sword, and beside the target (s / 2 = 1 or 4) the position beside wins when it fits. Above or
  below it one is preferred: the one beside when "faces left" equals "s / 2 ≤ 2", else the
  neighbour; the other one with the vertical sword (13 below, 14 above: s / 2 = 2 or 3 is 13)
  is taken when the preferred does not fit, or when both fit and s is 0, 5, 6 or 11. A slot left
  without a frame then copies, in slot order, the first slot j with one at the least d(|s − j|),
  d(n) = n up to 6 and 6 − n above (so 11 beats 1 for slot 0, and 7 beats 1 too: the
  executable's measure), the copies made earlier counting for later slots.
- **The status line** (`battle_status_line`, written by `battle_log_add(text, 0, forced)`,
  §9.8): 0 "No shots left!" (GENRLTXT:299) for a shooter (flag 4) with no shots left and a target
  still set (+0x10, +0x14, which a hex outside the field or a side column leaves as the last one
  set: T6c-2), else empty; 1 "Move %s here." (295) and 2 "Fly %s here." (296), the acting stack's
  name by its count; 3 and 15 "Attack %s" (221) with the target's name by its count, forced,
  while the army info is off (else 297 or 38 with the damage: not modelled); 4 "View Hero Stats"
  (418), 21 "View Enemy Hero Stats" (419); 5 "View %s info." (298) with the hex's stack's singular
  name, empty over no stack; 7 "Attack %s" (221), forced (with the army info 37 and the damage:
  not modelled); 17 "Apply first aid to the %s" (420); 16, 20 and 22 sieges' and casters' (not
  modelled); every other kind empty. After 0, 1, 2 and 5 an acting Archangel, Pit Lord or Ogre
  Mage may replace it and force it (0x4922F0, not read further: not modelled).
- **The panel's hints:** each control's pair (its hint, its right-click text) from `Help.txt`,
  whose loader (0x5B98B0) copies records group by group: records 379 to 389 to 0x6A6968, in the
  panel's order Surrender, Retreat, the options, Auto Combat, the log's item, its scroll buttons
  (one pair for both), the spell book, Wait, Defend. The log's item and the scroll buttons' hints
  are empty. Over the panel the status line shows the hint of the control under the mouse, a
  disabled one's too (the spell book); the viewer takes the last item in the dialog's order whose
  area holds the mouse (`Dialog::item_at`; the picture takes none: *provisional*, the dialog's
  search not read). The records count as `h3-formats` reads `Help.txt`; the harness's `h3lod`
  counts one fewer from the Multiplayer Selection group on (NOTES).
- **The viewer** (*provisional* where marked): `h3-view render --frames` draws, where the
  `frames.toml` names a mouse, its cursor, the marked stack's outline and the status line's hint
  in the log's item (the situations' battles write no lines: the hint over the log's lines is
  T6c-2's), the legal commands from the scenario's engine at the opening; the facing is the
  side's. Not modelled: the window's mouse (T6c-2), a turn's mouse move, the stale target,
  sieges, casters, the army info, network play.
- **Checked:** the capture set's 26 `mouse-*` situations (the harness moves the mouse at the
  opening, the clock frozen, `set.toml`'s `mouse`; scenarios `mouse-dragons`, `mouse-angels`,
  `mouse-devils`, `mouse-elves`): 28 frames, every one exact from 100 ms (at the opening itself
  the original shows the new cursor where the mouse was: its overlay moves at its next frame).
  Over the field the attack's swords 7, 8, 9, 10, 13 and 14 (one-hex Arch Devils, two-hex Black
  Dragons and Archangels; slots copied at the field's edge), fly, walk, nothing (an obstacle, out
  of reach, a side column), the broken arrow, the query over the acting and another stack (the
  Pikemen's outline and fidget at 100 to 300 ms), both heroes; over the panel Surrender, the
  disabled spell book, Defend and the log; the hints. The even x showed as a one-pixel shift of
  the cursor at odd points (144 pixels a sword, 101 the pointer) until the comparisons drew it. The set's other 31
  situations draw the parked cursor (frame 6 at (618, 593)), their `cursor` masks gone: all 57
  situations' 85 frames exact.

### 9.10 The mouse over time (M3 T6c-2)

T6c-2 taken whole (the user, 2026-10-21). Read: the battle's mouse-move branch again for the same
hex and the status line's rewrite (0x475544 to 0x4758D1), a human's turn start
(`battle_controls_state` 0x47883E to 0x478873), the action dispatcher's head (0x478DA0 to
0x478E3A: every action shows the pointer) and its switch (0x47902F: the action +0x3C less 1),
who unmarks the mouse's stack (`battle_clear_highlight` 0x477E10's callers: a walk 0x4459A6, a
strike or a shot 0x445AD4, the Catapult 0x445D55, a cast 0x5A0055, the computer's turn 0x422200;
`round_start` 0x476081), who stands the fidgeting stacks (`battle_stand_fidgets` 0x479DE0 and its
callers), `battle_cursor`'s target (+0x10, +0x14 of the acting stack), `battle_mouse_stack`'s tail
(0x477D47 to 0x477DF8), the mouse manager's draw and poll (`mouse_draw_cursor` 0x50CD90,
`mouse_poll` 0x50D680, its next time 0x69CA18), the DEF button's handler (`def_button_message`
0x456190: keys and the mouse's button) and the panel's hotkeys (`battle_panel_base` 0x46B7B1 to
0x46BAF9). Built: `h3_view::mouse::track` (`follow`, `Input`, `Turn`, `Start`, `Shown`),
`mouse::Facing`, `Field`, `pointer_in`, `slots_in`, `cursor`, `Hint::NoShots`;
`Timeline::enter` (`anim/entered.rs`), `anim::unfidgeted`, `hero::Action`'s `start`, `stands`,
`unmarks`; `battle_log::Logged::Status` and `Scroll`; `ui::Button::keys`, `Dialog::button_at`,
`button_for_key`; `Panels::at_pressed`; `cursor::Cursor::read`; in the adapters
`pointer` (the harness's actions as inputs, each turn's battle from the engine:
`scenario::turned`), the window's mouse, the log's `Writer`.

- **When the battle takes a move.** Only while a human's turn waits (the dialog's loop): during
  an action, and a computer's turn, the moves wait in the input queue. On the hex the mouse was
  already on only an attack's slot is computed again (0x475553): the kind is not; the status
  line is written for a new hex (its kind's "written" +0x132DC forgotten, -99), for another
  kind, or for an attack whose slot gave another frame (0x4758A4). Outside the field (0x475402)
  the items' hints (the control under the mouse, else empty, under the 3 000 ms rule: `walk`'s
  hint gone at the next human turn, `strike`'s line not replaced), the pointer, the hex forgotten.
- **An action** shows the pointer at its start (0x478E3A, whatever the cursor was: `strike`'s
  sword gone at 0 ms); a walk, a strike, a shot, a cast and the Catapult's shot unmark the
  mouse's stack (0x477E10), as does every round's start (0x476081). A **computer's turn** shows
  the pointer (0x478873) and unmarks (0x422200). A **human's turn** (a war machine's that plays
  itself in hot seat too: `machines` at 2 221 ms) settles the log (§9.8), forgets the mouse's
  hex (0x47883E) and posts the mouse where it rests (0x478857): the moves that waited are taken
  first, then the posted one, each a new hex.
- **The hints** go into the log's status line as the battle writes them
  (`battle_log_add(text, 0, forced)`): an attack's and a shot's forced over new entries (`machines`
  at 0 ms: "Attack Pikemen" over the defend's line 0 ms old), the others not. **The target**
  (+0x10, +0x14 of the acting stack) is cleared over every hex of the field (0x476686), set over
  an enemy (0x47679D), cleared again where neither a shot nor a strike reaches it (0x476837), and
  left by nothing, a side column or a hero: there a shooter without shots shows "No shots left!"
  (GENRLTXT:299, `battle.hint.no-shots`) for the last enemy it could strike.
- **Entering a stack** (`battle_mouse_stack`): over the stack already marked nothing; else the
  old one unmarked and the new one marked, its group 1 started at frame 0 (or, showing it, it
  stands), then the battle redrawn (0x477DF8) inside the new frame's rectangle only: pixels of
  the old frame outside it stay on the screen until the next idle step's redraw (`hover`: one
  pixel of the standing Pikemen's pike at 0 and 33 ms, thirteen of frame 2's at 200 and 231 ms,
  each equal to the original's screen just before). The idle step then plays the group as a
  fidget (§7.2); an action that stands the fidgeting stacks (`battle_stand_fidgets` 0x479DE0: a
  walk, a strike, a shot, a cast, a creature's cast, the Catapult's shot) stands it at its start
  (`strike`'s Pikemen, entered at 0 ms, stand as the Archangel flies), and the heroes' idle clocks
  start again then (not modelled: the heroes are the original's in a check). The viewer plays
  no stalls (one step in about 12 in the original: a check would show it).
- **The cursor's point.** The software cursor is drawn where the mouse manager last read the
  mouse (+0x6C, +0x70, its x even): a SetCursor draws the new frame there at once, and only
  `mouse_poll` (0x50D680, from the input loop) reads the mouse again, on a grid of 33 ms of the
  game's clock (its next time 0x69CA18, moved on by max(the time since it, 33)). The grid's phase
  is whatever the run before the battle left (19 and 10 ms in two captures of `hover`), so the
  harness sets 0x69CA18 to the opening's time (its timers only, never the game's state): the
  cursor then moves at 33 k ms in every run. The frame check draws the viewer's frame where the
  original's memory says the cursor is (`frames.toml`'s `cursor`); the viewer's own point is the
  mouse's.
- **A ray hides the cursor** (`draw_ray` 0x5A5C20): `mouse_hide` (0x50D540) as it starts
  (0x5A5C46) erases the cursor from the screen, `mouse_show` (0x50D5B0) after the shot group's
  rest (0x5A61D4) draws it again: in `shots` the panel shows under the parked point from 655 to
  879 ms, 4 086 to 4 267 and 5 293 to 5 517 (35 pixels in each of 40 frames until the viewer
  hid it too: `Beam::cursor_back`, `Timeline::cursor_hidden`; the window hides its cursor then
  too).
- **The facing** the cursor asks (a two-hex stack's other hex, a shooter's front hex) is the
  timeline's at the turn's start (`Facing::Left`, the poses facing left), the opening's by side.
- **The viewer** (`mouse::track::follow`): the log's entries and the inputs in time order (a move
  after the entries at its time up to an action's start; an `Act`, a click's or a key's, takes
  that start, the moves after it wait), each turn's battle and legal commands from the engine
  (`scenario::turned`; a record without its scenario: the opening's battle for every turn, no
  legal commands); its log, with the hints in, is the panel's (`log::panels`). The capture
  harness's actions are the inputs (`pointer::inputs`: the mouse parked first, `hex:` and `click:`
  move, click and park it again at once, `move:` moves it, `key:` acts). The timeline check
  leaves out the original's own fidgets but those of the stacks the mouse entered
  (`anim::unfidgeted`); the frame check draws those stacks as the viewer's timeline has them, the
  others in the original's poses.
- **The window** (`h3-view window`, *provisional*): the battle takes the window's mouse as the
  original's (the inputs at the battle's clock; after a seek back the later ones go), its cursor
  the combat cursor's frame (hardware or drawn, `Cursor::read`: 0x50D8B0's plain drawer, runs
  skipped), the marked and entered stacks and the hints re-made on a move. A panel button shows
  its click frame while the mouse holds it (and is on it) or one of its hotkeys is down (the
  layout's `keys`, since E24 M3 T1b the battle window's table `h3_view::keys`: S, R, O, A, C, W,
  D and Space, the scan codes 0x1F, 0x13, 0x18, 0x1E, 0x2E,
  0x11, 0x20, 0x39; the key down presses, the key up releases: 0x4565E7, 0x4564D6); a disabled
  one takes none (0x4562DD). A replay takes no command, but the log's scroll buttons (2006, 2007)
  and the arrow keys (the battle's own, 0x4758FB) scroll it (`Logged::Scroll`). The viewer's own
  keys moved: P pauses, Home restarts, H the cursor's kind (since E24 M3 T1b the map window's
  F-keys). Not modelled there: the viewer's own
  fidgets meeting the mouse's.
- **The harness** (`capture/`): `h3mem`'s battle state reads the mouse's hex, the cursor's kind,
  type, frame and drawn point; a sequence keeps a frame where only the cursor changes too (its
  screen digest takes the cursor) and masks nothing fixed (the parked cursor drawn, the panel
  compared since T6b); `hover` is captured and compared (no engine command: the opening's
  battle); 0x69CA18 set at a sequence's opening.
- **Checked:** every sequence's timeline as recorded (eleven: `hover`'s with the original's own
  fidgets left out). All eleven sequences captured afresh with the cursor compared and nothing
  fixed masked: 922 frames, every count the ten sequences recorded before unchanged (the
  ` log` listings from 668 to 173, each a line whose numbers the engine rolled: the hints are
  the viewer's), `shots` six frames more where the cursor hides and shows (exact), `hover` new:
  20 frames, 16 exact, four with the leftover pixels above. The set not captured (its capture
  unchanged): `drawn`, every situation the same. The window under Xvfb with the mouse moved and
  clicked over the field and the panel: every frame read back exact.
- **Not modelled:** a stall of a mouse fidget, a hero's cast's return to the same stack's turn
  (0x479195: the mouse posted again), the army info windows, the hex shade under the mouse
  (the option "Show Combat Mouse Hex", off in the harness; its call on every move not read),
  casters' and sieges' cursors, network play, the redraw's leftover pixels (above).

### 9.11 The click (M5 T2a)

Read: `battle_buttons`' head (the message's type: 1 a key down, 4 a move, 0x200 an item's
message; its subtype 12 the left button pressed, 13 an item's command, 14 the right button
pressed), the press over the battlefield (0x475290), the action it gives by the cursor's kind
(`battle_click_action` 0x476BD0), the panel's commands (the switch 0x474E53), the attack's
from-hexes (`cursor_attack_slots` 0x474690 whole, `cursor_attack_from_hex` 0x474A00), the reach
map's front hexes (0x4B2E69 to 0x4B2EF4), who takes input when (`battle_controls_state`'s head,
0x4782D0 to 0x478301, and 0x478445; `input_flush` 0x4EC570; `battle_loop_turn` 0x4741F9 to
0x47425B). Built: `h3_view::mouse::click` and `button`, `mouse::attack_slots` (`Slot`), `Legal`'s
shots with their aims and its Wait and Defend; in the tracker, the queue emptied at a turn's start
and CLT-Q24's answer.

- **Who takes input.** `battle_controls_state` sets +0x132B4 when the side to act is played on
  this computer (0x47844D; a human elsewhere, 0x478445, clears it). The battle takes a click or a
  button's command only while it is set. Its head (0x4782FB, unless Auto Combat +0x132C4 is on or
  the turn is remote) moves Windows' pending messages into the input queue and empties it
  (`input_flush` 0x4EC570: `pump_messages`, then the read and write indices 0): a click, a key or
  a move made during an action or a computer's turn is dropped at the next stack's turn start,
  and a human's turn then posts the mouse where it rests (0x478857). This corrects §9.10: the moves
  that waited are not taken first, the resting point alone is (the recorded sequences unchanged).
  A hero's cast returns to the same stack's turn without the flush (0x479195 posts the mouse
  again): what was pressed during the cast acts after it (M3 T7b-1's key). Under Auto Combat a key
  down or a mouse button (types 1, 8, 16, 32, 64) empties the queue and ends it (0x4741F9 to
  0x47425B; M5 T4's).
- **A press over the battlefield** (subtype 12 on the battlefield's items 0 and 1) acts at once
  by the cursor's kind (+0x132E0) the last move left; the panel's items +0x74 to +0x88 are hidden
  and the battle redrawn first (not read further). By kind (0x476FBC, the kind less 1):
  - a walk or a flight (1, 2): action 2 to the mouse's hex; for a two-hex stack whose reach map
    marks the hex as a front hex (+0x20F: the hex beside a reachable position on its facing's
    side, 0x4B2EF4, not in a side column) to that position, the hex less 1 facing right, plus 1
    facing left (0x476C5F), even where the hex is a reachable position too;
  - a shot (3, 15): action 7 at the mouse's hex (the engine's aim: the hex for an area shot,
    else the target's front hex);
  - an attack (7): action 6, the target on the mouse's hex, from the slot's from-hex (+0x132D8);
  - a wall-shot target (16): action 9; the First Aid Tent's heal (17): action 11;
  - the acting side's hero (4): its spell book (`hero_book_button` 0x59E900, M5 T3); a creature's
    cast (20, the Faerie Dragon's after its key): action 10 (M5 T3);
  - a stack to look at (5): its stats dialog (`stack_stats_dialog` 0x468860, modal); a turret
    (22): its info (0x47A380); none of them a command (the inbox);
  - anything else, the other side's hero (21) and nothing (0) among them: nothing.
- **The attack's from-hex** (`cursor_attack_slots`): each slot s takes the neighbour n in
  direction s / 2 when the reach map marks it, the stack fits and none of its hexes is a stop: its
  from-hex n. A two-hex stack also tries m, n less 1 facing right (plus 1 facing left): for the
  directions right and left (1, 4) m when it is taken, else n; for the others the preferred one
  (m facing left up right and down right, or facing right down left and up left; else n) when
  taken, the other with a vertical sword when only it is taken or both are and s is 0, 5, 6 or
  11. A slot without a sword copies the nearest's frame and from-hex together (the measure of
  §9.9).
- **A panel button** gives its command when it is clicked (the mouse's button released inside
  it, or its hotkey's key up: `def_button_message` 0x456190); a disabled one gives none: Wait
  (2009; 2300 too, the tactics phase's panel's, not read) action 8, Defend (2010) action 3.
- **The viewer** (`mouse::click`, `mouse::button`): the command a press gives at a point in a
  turn's battle (its legal commands, the stacks' facing): the engine's `Move`, `Shoot` (the
  mouse's hex as its aim when listed, else the target's listed aim), `Attack` (the slot's
  from-hex), `ShootWall`, `Heal`; a panel button's `Wait` and `Defend` when listed. The window
  sends them (M5 T2b). In the client's profile the tracker forgets the hovered panel item when
  the battle takes a move over the battlefield, so the panel's picture writes its empty line again
  (CLT-Q24, answered (a)); `Bugs::Original` keeps the original's.
- **Checked:** scripted presses at every hex (in each slot of a hex the acting stack may
  strike), on the wall rectangles and on the two buttons, over every turn of every scenario the
  bot fights (the situations', the sequences' and the packs' synthetic ones: 1 455 turns, 76 162
  commands) give exactly the engine's legal walks, strikes, shots, wall shots, heals, Wait and
  Defend but one walk (`h3-view`'s `clicks` test): a two-hex stack's step one hex back, whose
  position's front hex is its own position, while it may walk two back too (157 turns; a press
  on that position walks two back). CLT-Q25: the original's copied, the fix an option later.
  The recorded sequences unchanged (the queue emptied, CLT-Q24's switch in the client's profile
  only).
- **Not modelled:** the press's hiding of the panel's items, the right button's popups (0x4769C0),
  the dialogs (the stack's and the turret's: the inbox; the spell book: M5 T3), the hex shade.

### 9.12 Hot seat in the window (M5 T2b)

The window plays a scenario's battle with a human on a side or both (`h3-view window --human
attacker|defender|both`, *provisional*: the bot plays the others). Built: `anim::Growing::waiting`
and `push_seen`, `anim::Object::owner`, `Timeline::humans` and `screen`; the binary's
`live::Live` (seats, `play`, `choice`, `obstacles`, every side's events), `Battle::command`,
`press`, `button`, `takes`, `press_for`, `pointer::press_points` (moved from T2a's `clicks`).

- **The seats.** The sides a human plays here are the battle's human players (`setup.human`), the
  others the computer's (*provisional*, the M5 review list): the panel takes the colours of the
  last human turn (§9.8), the mouse follows a human's turns (§9.10), the end's tune is the human
  side's (§8.2), and the engine's computer-only rules apply to the bot's side (the bad-morale freeze's let-off,
  BAT-QUIRK-27; the computer's difficulty branches are the original's computer's, which the bot is
  not). Without `--human` the bot plays both as at M5 T1b, the scenario's `human` kept.
- **The battle waits.** On a human's turn the battle waits at its timeline's end: the timeline
  shown is `Growing::waiting`'s, what the last command left to the next redraw shown with the first
  idle step after the end, the idle steps (the outline's pulse, the heroes) and the viewer's
  fidgets going on. The command given at `t` grows the timeline with the same moments before `t`
  (`push`'s idle steps run from the end, the first showing what was left): the replays' tests
  check it before every command given at a time. After it, the window's playhead starts afresh.
- **Input.** The battle takes a press or one of its keys only while it waits for a human's
  command and the last action played (the clock at the timeline's end or past it): a left press
  over the battlefield gives `mouse::click`'s command as the button goes down, the panel's Wait or
  Defend `mouse::button`'s when clicked (released over it, or its hotkey W, D or Space released).
  A press or key made earlier (during an action, the computer's turn) is dropped, its release then
  doing nothing (§9.11: the queue emptied at the turn's start; the button never went down); the
  viewer's own keys (pause, restart, the music) always work. Each command is sent from the side
  to act's seat in the session in-process, which validates it as a server does (a rejected one
  changes nothing), then the copy the bot decides on plays it; its action starts at the press's
  time, the mouse's `Act` input with it (§9.10).
- **The side to act's view.** `battle_redraw` draws an object whose +0xA (revealed) is clear only
  for the screen's side: the acting side when a human here plays it, else the other one
  (0x49460A to 0x49463A; §7.16). A battle a human plays here grows from every side's events
  (`Live`: the engine's events, each with the side that alone sees it, `push_seen`): an object an
  event one side alone sees places is that side's (`Object::owner`) until an `ObstacleRevealed`
  shows it to both from the next frame drawn; the opening's are both sides' (`Live::obstacles`:
  a Tower's mines). `Timeline::screen` names the screen's side at a time by the turn's stack's side
  (*provisional*: Hypnotize's controller not followed, R3-1, M5 T5); without a human here every
  object shows, as a record holds only what its viewer sees (`render --seat`, the Tower's still:
  the keep's turn, the defender's screen). The placing animations play on every screen: Land
  Mine's handler tests only whether the battle is hidden before its sample and each mine's
  animation (0x5A060F to 0x5A063D), so a spectator's record, which lacks a hidden mine's
  placement, shows less than either player's screen.
- **Unattended** (`--check`, `--measure` with `--human`): 400 ms after a human's turn started the
  window presses the bot's choice where a press gives it (`Battle::press_for`: the panel's Wait or
  Defend, else a hex's point or one of its slots, else the siege's rectangles; the mouse moved
  there first), or gives it without a press where none does (a hero's cast, the tactics phase's
  end: M5 T3, T4; CLT-Q25's step back); at the end it checks that the record is the start of the
  battle the bot fights at once with the same setup.
- **Checked:** `window --check` under Xvfb on `scenarios/battle.toml`: hot seat and a human
  attacker against the bot played to the battle's end by presses (24 commands; 1 281 and 1 248
  frames, every one exact), the battle the bot's at once; real X clicks (`xdotool`): a flight, a
  Defend by its key, Wait by its button, a walk, the panel's colours the defender's on its turn;
  the Tower's siege in hot seat: its mines on the keep's and the towers' turns, none on the
  Catapult's.

## 10. The siege (M4)

### 10.1 The siege still (M4 T1a)

Read 2026-10-23 in the user's `Heroes3.exe` (`scripts/exe.py`), compared with VCMI 1.7.5
(`client/battle/BattleSiegeController.cpp`, the factions' `siege` in `config/factions/`). A siege
here is a battle at a town with a fort (the manager's +0x132F4 above 0); a town without one is
fought on an ordinary battlefield (§5). What moves (shots, the drawbridge, Earthquake, the
cursors) is T2a's.

- **The parts** (`siege_wall_parts_by_town` 0x66D848: per town 18 records of 36 bytes): x and y
  (`i16`), the hex it is drawn at (`i16`, −1 none), five picture names at +8, then WALLS.TXT's
  name cell (+0x1C) and hit points (+0x20), filled at the game's start (`walls_txt_load`). The
  index is WALLS.TXT's row: 0 Drawbridge, 1 Drawbridge Rope, 2 Moat, 3 Moat Lip, 4 Back Wall,
  5 Upper Tower, 6 Upper Wall, 7 Upper Buttress, 8 Mid-Upper Wall, 9 Gate, 10 Mid-Lower Wall,
  11 Lower Buttress, 12 Lower Wall, 13 Lower Tower, 14 Keep, 15 the keep's cover, 16 the lower
  tower's, 17 the upper tower's. Their hexes: 5 and 17 on 255, 6 on 45, 7 on 62, 8 on 78, 9 on
  112, 10 on 130, 11 on 165, 12 on 182, 13 and 16 on 251, 14 and 15 on 135; 0 to 4 none.
- **The pictures** `Sg<town><piece>.pcx` (towns Cs, Rm, Tw, In, Nc, Dn, St, Fr, El; Conflux's in
  `H3ab_bmp.lod`, the rest in `H3bitmap.lod`; all 8-bit, drawn with index 0 transparent,
  `pcx8_draw_to_pcx16`), by slot: the drawbridge Drw3, Drw2, Drw1; the rope DrwC in slot 1;
  Moat, Mlip, the back wall TpWl (TpW1 in Rampart, Necropolis, Dungeon and Stronghold) and the
  buttresses Wa5, Wa2 in slot 0; a wall (Wa6x, Wa4x, Wa3x, Wa1x) its damaged picture x2 in slot 1,
  its whole x1 in slot 2 and nothing in 0; the upper tower Tw22 then Tw21, the lower Tw12 then
  Tw11, the keep Man2 then Man1; the arch in every slot; the covers ManC, Tw1C, Tw2C in slot 1.
  The Tower has no moat and no lip pictures. The background `Sg<town>Back.pcx` (800 × 556,
  `siege_backgrounds` 0x63D2A0). Every name is in the user's LODs.
- **Loading** (`battle_load_heroes` 0x463370, from 0x4633AB): each part's five pictures into the
  manager's +0x13DF8 + 4 (5 part + slot), empty names left empty; a Stronghold's moat (part 2) is
  not loaded in a game under RoE or AB rules (its moat flag is off then too, §7.27).
- **Which picture: the part's picture state** (+0x13FA8, one `i32` per part) is its slot. The
  siege's start (`battle_obstacles` 0x4662E3 to 0x466376) copies the hit points into it (WALLS.TXT,
  the same in every town: 2 for the towers, the four walls, the gate and the keep, 0 for the rest;
  the covers set to 1) before a Castle raises its four walls' hit points to 3; a hit
  (`wall_part_damage` 0x465990) sets 1 while hit points are left and 0 at none, and a destroyed
  tower or keep sets its cover's to 0. So a wall shows x1 whole, x2 once hit, nothing once
  destroyed; a tower or the keep its whole picture (Tw21, Tw11, Man1) until destroyed, then Tw22,
  Tw12, Man2; the arch never changes.
  Nothing tests the fort level: a Fort draws the towers, the keep and their covers too, without
  shooters. **From the engine:** `Siege.parts` holds the hit points by wall-shot target
  (`wall_shot_targets` 0x63BE60: the targets 0 to 7 are the parts 5, 6, 8, 9, 10, 12, 13, 14),
  so a target's state is 2 at its starting hit points, 1 above 0 and 0 at 0; a cover is 1 until
  its tower or keep is destroyed (Earthquake's hits on a lower fort's absent turrets included,
  BAT-QUIRK-22: a Fort's tower can show destroyed).
- **The gate** (+0x53A4: 3 closed, 2 moving, 1 open, 0 destroyed) is drawn as part 0 with its
  state as the slot: destroyed Drw3, open Drw2, moving Drw1, closed none (the arch over the
  background); the rope (part 1, DrwC) only while it is open. From the engine: `gate_open`, and the
  gate target's hit points at 0.
- **The turrets** (`siege_turret_records` 0x466C50 from `siege_turrets_by_town` 0x63CF88, 32
  bytes per town: the shooter's creature, the keep's x and y, the lower tower's, the upper
  tower's, the missile's DEF): with a Citadel the keep's record (manager +0x13D78), with a Castle
  also the lower tower's (+0x13D9C) and the upper's (+0x13DC0), each 0x24 bytes: the creature, its
  battle DEF (the creature table's +0xC), the missile's DEF, x, y, facing 0, group 2 (standing),
  frame 0, and its turret stack's index (+0x20, `battle_war_machine_stacks` 0x463600). The
  shooters: Castle the Archer (2, `clcbow.def`), Rampart the Wood Elf (18), Tower the Mage (34),
  Inferno the Gog (44), Necropolis the Lich (64), Dungeon the Medusa (76, two-hex), Stronghold the
  Orc (88), Fortress the Lizardman (100), Conflux the Storm Elemental (127). The turret stacks
  themselves (the engine's Arrow Towers on 251, 254, 255) are never drawn as stacks: the row
  passes take hexes 0 to 186 only (`draw_hex_stack`).
- **A shooter** (`draw_turret_shooter` 0x495090, from the parts drawer below): the shadowed
  drawer (§4.2) of its group and frame, but only the canvas's top 232 rows (a stack's whole
  height elsewhere), mirrored while its facing is 0 (always at rest), the canvas's top-left at
  (x − W + 196, y − 267), W the DEF's width (450), 44 further right for a two-hex creature and 5
  left for the Medusa (its id tested); outlined in `game.pal`'s entry 0x60 while it stands and
  is the acting defender's active stack (+0x132B8, +0x132BC against the record's +0x20).
- **The order** (`battle_redraw` 0x494440 with a fort; §4.6 completed):
  1. The background buffer, composed once (`compose_battle_background` 0x493CF0): the town's
     picture, then with the moat flag (+0x53A8: a Citadel or Castle, but not a Tower, nor a
     Stronghold in an older game) the moat (part 2) and its lip (3) at their points.
  2. The layer-1 obstacles (under SoD rules a siege has no natural ones, §7.27).
  3. The back wall (part 4, slot 0) at its point.
  4. The flags and heroes (§4.5).
  5. The parts on hex 255: the upper tower (5), its shooter and cover (17).
  6. Each row 0 to 10: on row 5 while the gate is not closed, first the drawbridge (above); then
     the passes 0 to 7 over the columns, left to right but **right to left on rows 6 to 10**; in
     pass 0 at each hex first the parts drawn there (below); after row 5's
     passes, while the gate is open, the rope at its point.
  7. The parts on hex 200 (the lower wall's last strip, below) and on hex 251: the lower tower
     (13), its shooter and cover (16).
- **The parts at a hex** (`siege_parts_at_hex` 0x494C20, given the hex and the row's direction
  d, +1 on rows 0 to 5 and −1 below; the parts in index order, each that has a picture for its
  state and a hex): the towers, the keep, the arch and the covers whole (`battle_draw_pcx_rect`
  0x4957B0) when the hex is theirs, a cover after its turret's shooter (when the record has a
  DEF). A wall or buttress on hex h (row r, o = r mod 2) is drawn in three vertical strips of its
  whole height, cut at h's left edge L (§4.1): the part left of L at hex h + 17d − o, the 44
  columns from L at h, the rest at h − 17d − o + 1. So the upper wall's strips fall on 62, 45
  and 29, its buttress's on 78, 62, 45, the mid-upper wall's on 95, 78, 62, the mid-lower's on
  112, 130, 147, the lower buttress's on 147, 165, 182 and the lower wall's on 165, 182 and 200
  (left, middle, right): each wall meets the stacks of the rows beside it piece by piece.
- **The moat over a stack** (`draw_hex_stack` 0x494F40 with the moat flag, after the stack's
  sprite, its box and the magic over it; not on row 5 while the gate is not closed, nor in pass
  7): when the stack's hex, or a two-hex stack's other hex (its position + 1 facing right, − 1
  facing left), is its row's moat hex (0x63BCE8; a Fortress's also 0x63BCF4's),
  `moat_over_stack` 0x495A10 draws the moat picture's (part 2) pixels at that hex from x L to
  L + 43 and y top + 36 to top + 41 (a 44 × 6 strip at its feet, clipped to the picture and the
  field, index 0 transparent); then, in the moat or not, the count box again (part 1, §4.2).
- **VCMI 1.7.5:** the executable's points but the drawbridge's, one pixel further left in every
  town (Castle's also two up, Stronghold's one up), the Fortress's mid-lower wall one left, the
  Dungeon's moat and lip one right, its towers three to five left and one down and its mid-upper
  wall, arch and lower buttress one down; Necropolis without its lip; the shooters' points the
  same (the Medusa's 44 and 5 included). It draws each wall whole on one hex in its layer WALLS
  (the upper wall on 12, its buttress on 45, the mid-upper wall on 62: a row earlier than the
  executable's middle strips), the battlements only while their turret is there (none at a
  Fort), the drawbridge open or destroyed before every row, never the moving bridge or the rope,
  no moat over a stack, and the shooter whole.
- **T1b's situations** (the user, 2026-10-23): every town with a Castle, one Fort, one Citadel;
  the wall states (hit and destroyed walls, towers and keep, the gate open and destroyed) on
  Castle and Conflux only, one town per archive (the rules are the same for every town; only the
  points, pictures, shooters and moats differ). The captures settle VCMI's differences (the
  drawbridge's pixel, Necropolis's lip, the Dungeon's points), a Fort's covers, the shooter's
  cut and the moat's strip (a stack on a moat hex).
- **Not read:** the grid and shade options in a siege; the siege drawers' dirty rectangles beyond
  the shared rule (each extends or tests the rectangle, §7.11).

**Captured (M4 T1b, 2026-10-23).** The viewer draws the reading (`h3_view::siege`, the town
tables harvested as `[[town]]`) and the 15 siege situations of the capture set are exact
(`clients/situations/set.toml`): every town with a Castle, a Rampart with a Fort, a Fortress with
a Citadel, and on Castle and Conflux the walls hit with the gate open and everything destroyed
with the gate destroyed. So the executable's points stand against VCMI's (the drawbridge, the
Dungeon's points, Necropolis's lip are VCMI's own), and a Fort's towers, keep and covers, the
shooter's cut to 232 rows, the Medusa's shift of 39 (44 − 5), the walls' strips across the rows,
the drawbridge's three states and the rope are as read. Findings:

- **The opening.** Every round the turrets act first, then the Catapult (BAT-SIE-016, BAT-SIE-011):
  on their own, they act in the original's first frames, while the engine's record opens after
  them (`BattleState::new` drops the start's events, `h3-protocol`'s session uses it). So a siege
  situation is hot seat with both heroes commanding their machines (Ballistics, Artillery; the
  situation writes their skills): the opening is the keep's turn in both, or the Catapult's
  without turrets. Recording an automatic shot needs the start's events in the record (T2b).
- **The side to act's view.** In hot seat the original shows the side to act; on the keep's turn
  the defender's hidden objects show, a Tower's mines among them (the Land Mine pictures on its
  moat hexes, BAT-SIE-007), which a spectator's record lacks: the Tower's situation renders the
  defender's record (`seat`, `h3-view render --seat`).
- **The wall states by memory.** Written after the siege's setup (the parts' states above 0) and
  before the battle screen first shows (some 100 ms of the frozen clock later), the parts' states
  (+0x13FA8) and the gate's (+0x53A4) show in the first frame. The hit points (+0x13F60) stay: a
  tower's or the keep's written 0 makes the original pass its turn (its opening then the
  attacker's). The viewer takes the same states in place of the record's (`--walls`,
  *provisional*): a record's hit points give a part's state only whole or destroyed (a Castle's
  walls start at 3), a hit's 1 comes from its history (T2b's timeline).
- **Data.** Conflux's upper tower names its destroyed picture in slot 3 and leaves slot 4 empty,
  its walls slot 4 empty, its lower tower `SGElTw11.pcx` in slot 3: states 3 and 4 never occur
  (§10.1, the picture state), so nothing shows it.
- **Not captured yet:** the moat's strip over a stack (no stack stands in the moat at an opening:
  T2c's stack in the moat), the moving drawbridge (state 2, T2c), the turrets' and the Catapult's
  poses beyond the standing frame (T2b).

### 10.2 The siege over time (M4 T2a)

Read 2026-10-23 in the user's `Heroes3.exe` (`scripts/exe.py`; not split, the user), compared
with VCMI 1.7.5 (`client/battle/BattleAnimationClasses.cpp`'s `CatapultAnimation`,
`BattleProjectileController.cpp`, `BattleSiegeController.cpp`). F is the speed factor and T =
trunc(100 F) (§7); "through the deadline" is `battle_redraw`'s wait (§7.1; its arguments, in
order show, mark the dirty rectangle, show that rectangle alone, time, restore the background and
wait: §7.11 and the map); "on its own clock" a step timed from its own start outside the
deadline, as a projectile's (§7.4). In a hidden battle every one of these only changes
the state. The harness plays without sound, so the samples' waits (`sample_wait` 0x599EC0 on a
handle, `wait_for_sample` 0x59A4C0 on a WAV, each up to 4 s of the game clock) pass at once there,
as Quicksand's (§7.16); the timeline leaves them out. The decisions on the oddities: CLT-Q19 to
Q21 (the user, 2026-10-23).

- **A wall shot** (`wall_shot_animate` 0x445FD0 for each shot of `wall_shot` 0x445D30, so twice
  with Advanced or Expert Ballistics; the Catapult, the Cyclops, the Cyclops King):
  1. *The target* is the part hit, the engine's `WallShot.part` (not `aimed`: `wall_shot_once`
     0x445EC0 passes the miss's part), at its point in `wall_shot_targets` 0x63BE60: the upper
     tower (586, 48), the upper wall (564, 128), the mid-upper wall (520, 212), the gate (498,
     296), the mid-lower wall (520, 380), the lower wall (586, 506), the lower tower (630, 506),
     the keep (762, 212).
  2. *The group:* from the shooter's anchor (its hex's point, the hex records' +0x1C4) plus its
     straight missile offset (x subtracted facing left) to the target, the angle atan(Δy / |Δx|)
     in degrees (f64, stored f32), Δy the target's y less the start's **not negated**: above 25°
     group 14 (up), above −25° 15, else 16; Δx = 0 gives 90 for Δy ≤ 0, else −90. So a target
     above takes the downward group and one below the upward, the reverse of a shot's
     (`shot_angle_group` 0x469880 negates Δy); CLT-Q19 (1). The start is the anchor plus that
     group's offset. The Catapult's groups 14 to 16 are the same 13 frames and its three offsets
     equal (69, −119): only a Cyclops shows it.
  3. *Its frames:* its shot sound (+0x17C, category 3, volume 64), then frames 0 to c − 1 (c the
     climax +0x150, all the group's without one; the Catapult's 4 of 13) at trunc(trunc(A / c) ×
     F) through the deadline (A the attack time +0x15C), only its rectangle redrawn and shown, as
     `shot_animation`'s (§7.4). Then it stands in memory (group 2, frame 0, 0x4462A3) without a
     redraw: its frames after the climax never play, and the screen keeps frame c − 1 until the
     whole redraw of step 6 (CLT-Q19 (2)).
  4. *The rock* (`catapult_rock_flight` 0x467A00), at once after the last frame, on its own clock:
     d = trunc(√(Δx² + Δy²)) from the start to the target point, n = trunc((d + 10) / 20) steps of
     trunc(100 F) ms (`crt_ftol` truncates toward 0). Step i, 0 to n − 1, at x0 + trunc(i Δx / n)
     and y0 + trunc((Δy − (n − i) a) i / n) with a = 2 |Δx| / n (f64 in that order, the sum with
     y0 then truncated; the x87 at 53 bits, `installation.md` §7.10): an arc |Δx| / 2 above the
     chord at its middle, its last step one step short of the target. The missile (+0x168; the
     Catapult's `SMCatx.def`, 8 frames) is drawn unmirrored with its canvas centred there (less
     half its width and height), group 0 frame i modulo its frames, over the back buffer with a
     save-under, shown together with the step before's rectangle, and restored at once. After the
     last step nothing is shown: its last image stays on the screen until a later show covers it
     (CLT-Q19 (3)). VCMI: a parabola y = 0.005 x² + b x + c through both points, 200 pixels a
     second times the speed.
  5. *The landing:* `WallMiss.82m` for 0 damage, else `WallHit.82m` (loaded by name: category 0,
     sample handle 0, volume 127), then the explosion, `CSGRCK.DEF` for 0 damage (10 frames growing
     to 110 × 143 in a 180 × 230 canvas) or `SGEXPL.DEF` (10 frames in 254 × 203, the largest frame
     4, smaller after), group 0, its canvas centred on the target point; drawn (0x446453) with
     its rectangle clipped to the battlefield (0x446340) as the source size from the canvas's own
     top-left corner, which the frame drawer 0x47C570 clips again: a canvas reaching above the
     top edge loses as many rows at its bottom (the upper tower's, y −53: cut at screen row 97;
     the left edge alike, which no target reaches; CLT-Q22). Each frame k, 0 to 9: an
     unshown redraw of everything without the background restored, through the deadline at T
     (`battle_redraw(0, 0, 1, 100, 0, 1)`), the frame drawn over it, its rectangle clipped to the
     battlefield shown alone. The deadline being behind after the flight, frames 0 and 1 show at
     the same moment (frame 0 for no time), then one every T. At frame 5, with damage, the part's
     damage (`wall_part_damage` 0x465990: its picture state 1, or 0 destroyed; a destroyed tower's
     or keep's cover 0 and its turret stack dead, so from then on neither the cover nor its shooter
     is drawn, §10.1), drawn into frame 5. Without the restore the earlier frames stay where a
     later one is transparent (SGEXPL's peak lingers), a hit part's whole picture under its
     damaged one, a destroyed part's whole picture, until step 6 (CLT-Q19 (4)); the stacks in the
     rectangle, drawn over themselves, darken their shadows (§7.11's artifacts, not copied).
  6. *The end:* at once after frame 9 (frame 9 for no time) a whole redraw shown (`battle_redraw(1,
     0, 0, 0, 1, 0)`: the background restored, everything drawn, the battlefield shown); then the
     waits for the shot's sample and the landing's (`sample_wait`), Berserk ended (0x4464EA).
  7. *From the engine:* `WallShot { stack, aimed, part, damage }`, and for a part destroyed
     `WallDestroyed { part }` right after it (`damage_part`), applied at that shot's frame 5.
     VCMI: the shot group by the right sign, the explosion when the projectile ends, the walls'
     pictures after the whole animation.
- **A turret's shot** (`turret_shot` 0x465AD0 from `shot_action` 0x440160 for creature 149; the
  engine's `Struck` from a turret stack on 251, 254 or 255, `RANGED`; automatic every round first,
  BAT-SIE-016, or commanded with Artillery, the same):
  1. *The record* (§10.1): 251 the lower tower, 254 the keep, 255 the upper (record 1, 0, 2). Its
     facing becomes 1 when the target's point x is at or right of the record's x, else 0; the
     target's point is a shot's (`stack_screen_x` 0x446660, `stack_screen_y` 0x446630, §7.8).
     Facing 1 draws the shooter unmirrored with its canvas at x − 196, a two-hex shooter's 44
     further left and the Medusa's 5 right (`siege_parts_at_hex` 0x494EB3; facing 0 as in §10.1).
     It is outlined only while standing (group 2).
  2. *The group, the start* by `shot_angle_group` from the record's point and facing (a shot's
     sign); the sound `<prefix>shot.82m` of the shooter creature by name (`play_sound_file`:
     category 2, volume 127, where a stack's shot plays at 64).
  3. *Its frames* 0 to c − 1 (the creature's climax and attack time, the animation table's +0x40
     and +0x4C) at trunc(trunc(A / c) × F) through the deadline, its rectangle alone (its flag
     +0x1402C + record); then an unshown whole redraw (`battle_redraw(0, 0, 0, 0, 1, 0)`).
  4. *The arrow* (`turret_arrow_flight` 0x468220): n = trunc((d + 20) / 40) steps of trunc(33 F) ms
     on its own clock, step k at the centred canvas plus k × (trunc(Δx / n), trunc(Δy / n)); its
     frame by atan(−Δy / |Δx|) from the start (×57.2957763671875, stored f32) against the
     midpoints of the creature's angles (+0xC), as `shot_angle_group` counts them (Δx = 0: frame 0
     upward, the last downward), mirrored when Δx < 0; a save-under per step, restored, and after
     the last its rectangle restored and shown (the ordinary shot's flight, §7.8, but on the
     turret's own function).
  5. *The hit:* the damage, its log line (`battle_log_damage` with GENRLTXT row 376, "arrow
     tower", and one attacker: row 377 and the kills' row), the target's reaction
     (`strike_animation(−1, 1)`: no striker, every reaction at once, §7.4).
  6. *The rest:* frames c to the group's last at the same time through the deadline, never shown:
     the reaction's `reset_dirty_rects` (0x468D08) cleared the turret's flag, so each redraw's
     rectangle is empty and the screen keeps frame c − 1 (CLT-Q19 (5)); then facing 0, group 2,
     frame 0 in a whole redraw shown at once (0x465E89); the sound's wait.
- **The drawbridge** (`gate_open` 0x4671C0, `gate_close` 0x4672E0):
  - *Opening:* when a defender's walker's next position, or a two-hex walker's other hex there, is
    94, 95 or 96 with the gate closed (`defender_on_gate_hex` 0x467130 at 0x442186), before that
    step. Mid-walk (not its first step) the walker first ends its walk: its end sound (+0x18C),
    the move sound stopped, group 21's frames, one frame of group 2, standing in a whole redraw
    shown at once (0x4421BC to 0x442225). Then `drawbrg.82m` (category 2, 127) and the gate
    states 3, 2, 1 (closed, moving, open), each in a redraw through the deadline at T that
    restores and draws the whole battlefield and shows only the drawbridge's rectangle (365, 211)
    to (542, 380) (`drawbridge_rect` 0x694F30); the pathfinder renewed; the sound's wait. The
    walk then goes on with a first step (group 20, the start and move sounds: walk_step's flag,
    0x442247). A flyer whose destination is there stands in a whole redraw shown at once and the
    gate opens before its flight (0x4B4A95); a summoned elemental placed there opens it too
    (0x5A71F1).
  - *Closing* (after a walk 0x4423F2, a flight 0x4B4A29, a teleporting move 0x4B4EF9 and every
    action 0x479C65): with the gate open and no stack or corpse on 95 and 96 (and 94 in a
    Fortress, town type 7), `drawbrg.82m` and the states 1, 2, 3 at T in the same rectangle, the
    sound's wait.
  - *From the engine:* `GateOpened` comes after the `Moved` onto 96 (BAT-SIE-003), `GateClosed`
    at the action's end: the timeline plays the opening before the step its walk's path takes
    onto 94 to 96 (the path hex by hex) and the closing after the walk's last frame. VCMI changes
    the gate's picture at once with the sound, no moving state.
- **The moat's damage** (`moat_damage` 0x469E50 from `enter_hex` 0x46A570; SoD rules and the moat
  flag, so never the Tower): after the step onto a moat hex, which is a stop (so the walk's end,
  group 21 and group 2's frame, comes first), the move sound stopped, the damage, the log line
  (row 377 named by the town's moat, `moat_names` 0x6A5D60: `ArrayTxt.txt`'s nine names after
  its "Moat Names" row, loaded by 0x5B9CC0), the stack's reaction (`strike_animation(−1, 1)`:
  hit 3 or death 5, no animation of its own), the move sound again if it lives and walked, the
  Phoenix's check. On re-entry (`reenter_own_hexes` 0x441F70: Wait at 0x479AD6, Defend at
  0x4798F3, a shot or strike from the moat, a flight's or a teleport's landing) the same without
  the sounds. The engine: `Moved`, `MoatDamage { stack, hex, damage, kills }`.
- **A Tower's mine** (`land_mine_trigger` 0x59FB40; the defender's hidden Land Mines, BAT-SIE-007,
  as any Land Mine): the move sound stopped, the damage, the log line (Land Mine's name, row 377),
  the mine removed without a vanish animation (`remove_battle_object` 0x466B30), `landkill.wav`
  (category 2, 127), animation 57 over the stack with its reaction (`strike_animation(57, 1)`,
  §7.16), the sound's wait, the move sound again; the walk goes on (an unseen mine is no stop).
  The engine: `ObstacleTriggered { stack, spell, hex }`, then `SpellDamage`. On the attacker's
  turn its screen never shows the mine (removed before the blast's first redraw), only the blast;
  the defender's record shows it before.
- **Earthquake** (`earthquake` 0x5A7C80, the handler after the hero's frames 0 to c − 1 and the
  spell's sound, §7.16; it writes no log line):
  - *The shake:* the mouse hidden, the screen saved (the back buffer into the battle's 800 × 600
    buffer +0x53B0), then 3 times the 15 offsets (2, 2), (4, 1), (3, −2), (0, −6), (2, −2), (−1,
    3), (−5, 4), (−8, 6), (−4, 2), (−1, 1), (−3, −3), (−7, −5), (−5, −7), (−2, −3), (0, 0)
    (`earthquake_shake_offsets` 0x642284), each step trunc(15 F) ms on its own clock: the saved
    picture copied into the back buffer at the offset, the battlefield shown (0x493780). The
    strips an offset uncovers keep the step before's pixels, and an upward offset shows the
    panel's top rows in the field's bottom rows (copied in every profile, CLT-Q19 (6)). Then the
    background to be composed again (+0x53B8 cleared) and a whole redraw shown at once.
  - *The hits* (BAT-SIE-015), then with any target hit `WallHit.82m` once (`play_sound_async`
    0x59A590, category 3) and SGEXPL's 10 frames, each trunc(15 F) ms on its own clock: an unshown
    redraw without the background restored (`battle_redraw(0, 0, 1, 0, 0, 0)`), then over every
    target hit its frame centred on its point, each rectangle shown; at frame 5 every target's
    damage. After the last frame a whole redraw shown at once, the mouse shown; then the cast's
    tail (the hero's last frames). The explosions' leftovers as a wall shot's (CLT-Q19 (4)).
  - *From the engine:* the hero's `SpellCast`, then a `WallQuaked { part, damage }` per target and
    `WallDestroyed`. VCMI: no shake; SGEXPL at the targets with `WALLHIT`, the walls after it.
- **The mouse** (completing §9.9):
  - *The siege's rectangles* of `screen_to_hex`, tested after the heroes' and before the hexes,
    set statically for every battle (0x462700, 0x462730): 0xFE the keep (x 742–799, y 160–337),
    0xFF the upper tower (x 564–651, y 0–85), bounds inclusive. The lower tower has none: its
    target is hex 183. Over either no stack is marked (`battle_mouse_stack` takes hexes 0 to 186).
    Outside a siege they give kind 0, but the keep's covers the right side column on rows 1 to
    5, where a defender's war machine is then not marked: the client takes them in a siege only,
    the original's everywhere for the comparisons (CLT-Q21).
  - *The kinds* (`battle_cursor` 0x476490): **16**, the Catapult's target (`crcombat.DEF` frame
    16), for an acting stack with the wall-shooter flag (+0x84 bit 5: the Catapult, the Cyclopes)
    of the attacker (+0x132C0 0) outside the tactics phase: over 0xFF with a Castle and over 0xFE
    with a Citadel or Castle while that part stands (the stack's target +0x14 the rectangle's
    number, +0x10 −1), and over a field hex that is a standing target's aim hex (29, 62, 96, 130,
    182; 183 with a Castle) when the hex is empty, for the Catapult whatever stands there
    (0x4766B2). **22** (frame 5, "View arrow tower info", row 157) over 0xFF with a Castle or 0xFE
    with a Citadel otherwise (a destroyed tower's too), and over hex 183 with a Castle when it is
    no move.
  - *The status lines* (`battle_status_line` 0x492840): 16 GENRLTXT row 221 ("Attack %s") with the
    part's WALLS.TXT name for the town's type (the parts' +0x1C), not forced; 22 row 157.
- **The end tune** (§8.2) at a town is the manager's +0x53C8, set for every battle at a town, a
  fort or not: the viewer's town is the record's `Siege`, fort 0 included.
- **Not read:** a turret's info window (22's click); the Cyclops' shot sound beside the Catapult's
  (+0x17C, the same code); the drawers' tests of the dirty rectangle during the explosions.
- **The timeline's plan** (T2b, T2c; `h3_view::anim`): `WallShot` plays steps 2 to 6 at the part
  hit, `WallDestroyed` and the parts' states at its frame 5 (`Moment::walls` then takes a hit's
  state 1 from the history, T1b's override retiring); a turret's `Struck` (the attacker's hex 251,
  254, 255) plays the turret's shot instead of a stack's; `GateOpened` moves before the walk step
  onto 94 to 96 (the stop and the restart around it), `GateClosed` after the walk; `MoatDamage` and
  a mine's `ObstacleTriggered` after their step; `WallQuaked` the shake and the explosions inside
  the hero's cast; the end tune's town from `Siege`. Under `Bugs::Original` (the comparisons) the
  slips of CLT-Q19 (1) to (5) as read; in what the client draws (CLT-Q19, as CLT-Q8): the group
  by a shot's sign, the shooter standing from the flight, the rock erased after its last step,
  each explosion frame on a restored background with the parts' new pictures alone, the turret's
  frames after the arrow shown.
- **The forced rolls** (the runbook's "Forced rolls"; new sites in `xdd.dll`'s table): the damage
  roll `wall_damage_draw` 0x445ED6 (random(1, 100) against the record's chances for 0, 1, 2), the
  hit roll `wall_hit_draw` 0x445F25 (random(1, 100) up to the target kind's chance), a miss
  between two equally near walls `wall_miss_tie_draw` 0x445F6C (up to 50 the later), the next
  shot's new target at a tie `wall_retarget_tie_draw` 0x445E44, the automatic aim among the
  weakest walls `catapult_tie_draw` 0x473DA0 (random(1, k), drawn even for k = 1), Earthquake's
  target `quake_target_draw` 0x5A7DD8 (random(1, k): the k-th standing target with hit points
  left). The turrets' damage (random(2, 4) per creature, 0x465D3E) stays free: the engine's seed
  is searched to match the line and the box.
- **T2b's sequences** (hot seat, SoD rules, a Castle unless named; the heroes' skills as T1b's):
  `keep` (Artillery: the keep's and both towers' shots at the attacker's stacks, one a two-hex
  one; free damage), `hit` (Basic Ballistics at a wall: `wall_hit` falls, `wall_damage`
  1: the wall's state 1), `missed` (`wall_hit` fails: the nearest other wall takes it, a tie
  forced where two are equally near), `breach` (Expert at a Citadel's wall of 2 hit points:
  destroyed by the first shot, the second retargeted, `wall_retarget_tie` where needed), `tower`
  (Expert at the upper tower: destroyed at frame 5, its cover and shooter gone, the keep's later
  shot), `cyclops` (a Cyclops' wall shot at the upper tower and at the lower wall: the reversed
  groups), and with CLT-Q20's change `automatic` (no Ballistics and no Artillery: the turrets' and
  the Catapult's own round, `catapult_tie`, a 0-damage roll forced: `CSGRCK` and `WallMiss`).
- **T2c's sequences:** `gate` (a defender's walk out through the gate from inside, interrupted
  mid-walk, the gate closing after it, then a walk back in from its first step; the moving state
  in the frames), `gate-fortress` (a stack on 94 keeping a Fortress's gate open), `moat` (an
  attacker's walk into a Castle's moat: `MoatDamage`, the strip over it and its box, then Wait in
  it: the re-entry), `mine` (a Tower: an attacker's walk over a hidden mine, on the attacker's
  turn: the spectator's record), `quake` (the attacker's hero casts Earthquake: `quake_target` forced per hit, a wall
  destroyed), `aim` (the mouse over the targets on the Catapult's turn, 16 and its lines over
  0xFE, 0xFF and the hexes; 22 on a defender's turn), `through` (an attacker's shooter at a stack
  behind the walls: the broken arrow and the halved damage, `shot_wall_test` 0x467510),
  `defended` (the keep kills the attacker's last stack: `defend castle`).

**Recorded (M4 T2b, 2026-10-23).** The viewer plays the reading (`h3_view::anim`'s `siege`
module; the asset tables' `wall_explosions` and `wall_sounds`, harvested; the turrets' looks the
town's shooter's with the turret's missile) and seven hot-seat sequences are as recorded
(`clients/situations/sequences.toml`; the harness's siege lines below): `keep` (the keep, the
upper and the lower tower at the Pikemen, the Griffins and the Archers), `hit` (the upper wall hit
for 1), `missed` (aimed at the gate, the tie forced to the mid-lower wall), `automatic` (the
Catapult on its own: `catapult_tie` 2, a hit of no damage: CSGRCK and `WallMiss`), `breach` (a
Citadel's mid-upper wall and then the gate destroyed, the retarget's tie forced), `tower` (the
upper tower destroyed, its cover and shooter gone at frame 5; the keep's and the lower tower's
shots in round 2, the lower tower facing right) and `cyclops` (the lower tower below a Cyclops in
group 14, the upper tower above the other in group 16: CLT-Q19 (1)). Findings:

- **The harness's siege lines.** Each turret record's pose (`<ms> turret <k> g f d`) and the
  walls' states (`<ms> walls <18> gate <n>`) as memory holds them; the turret stacks themselves
  (the defender's last records) never change. The wall shots' rolls are forced with values
  (`roll:wall_damage:1`, `roll:wall_hit::2`: a count), the runbook's "Forced rolls".
- **The turrets' counts.** The situation's town gives the keep 5 creatures and each tower 3
  (`town_turret_count` 0x5BF4F0: one structure besides the 4 counts), so the sequences' scenarios
  set `structures = 1` for the engine's (BAT-SIE-016); the turrets' damage then matches with a
  searched seed.
- **The Catapult on its own** acts at its turn's start in the same millisecond as the command
  before it, outlined by a redraw then until its first frame (*provisional*: that redraw not
  read): the defender commanding its turrets opens the record on the keep's turn, so `automatic`
  needs no change of CLT-Q20's (which the opening's automatic turret shots still wait for).
- **The rock's arc** leaves the screen at its top for a shot from hex 119 to the upper wall or
  tower (no screen changes for some 700 ms): as computed, the frames before and after exact.
- **Differences listed** (the frame check's `frames` files): a wall shot's or a turret's first
  frame leaves its standing frame's outline top row on the screen (11 and 4 pixels: the first
  frame's dirty rectangle is the new frame's alone, `reset_dirty_rects` then the shooter's flag;
  §7.11 (3), not copied); the explosions' redraws darken the stacks' shadows in their rectangle
  (§7.11 (1), classified as artifacts); a Catapult's or Cyclops' target hint in the status line
  (`Attack Upper Wall`, kind 16, GENRLTXT 221) masked as ` log` until T2c draws the siege's mouse.
  A Cyclops' first shot frame leaves more of its standing outline (103 pixels, 5 at its feet
  until the next whole redraw; §7.11 (3)). In `tower` at the lower tower's shot (13 331) a 30 × 30
  square of the background at the arrow's last place covers the Archangel's first reaction frame
  for that frame (899 pixels): a rectangle restored from a picture without the target, §7.11's
  redraws (not read further; not copied, the user's 2026-10-14 decision).
- **The explosion over the top edge** (`tower`, `cyclops`: the upper tower's, its canvas from
  y −53) shows cut at screen row 97, step 5's double clip (CLT-Q22): reproduced in the original's
  profile (`Draw::Explosion`'s rows), whole in the fixed one; with it every explosion frame exact.

**Recorded (M4 T2c-1, 2026-10-24).** T2c cut in two by the user: T2c-1 the walk's siege events
here, T2c-2 Earthquake, the shots and the mouse. The viewer plays the reading (`anim::siege`:
`drawbridge`, `halt`, `entered`, `moat`; the walk's events gathered in the `Moved` run; the
asset tables' `siege_sounds`, `drawbrg` and `landkill`, harvested from 0x66FFB0 and 0x688410;
the moat's line named by the town, the key family `moat` over ARRAYTXT's records 260 to 268) and
the sequences are as recorded (`clients/situations/sequences.toml`, hot seat, `key = "Up"`,
seeds searched for the engine's wall shots): `gate` (a Castle town's Fort, no turrets or moat:
the Hobgoblins out from 100 to 93 and, in round 2, back in), `gate-fortress` (a Fortress's
Fort: out onto the outer bridge 94, which keeps the gate open; in round 2 the step off it closes
it), `moat` and `mine` (below). Findings:

- **The walk's stop** before the step onto the gate (`gate`: the Hobgoblins at 97, 5 743 to
  5 911) is the reading's: group 21 and one frame of group 2 through the deadline, a whole redraw
  shown at once; the gate's states 3, 2, 1 each at trunc(100 F), the first drawn as the stop's
  redraw shows; the walk goes on with a first step (group 20, its start sound). At the walk's
  first step (round 2, from 93 onto 94) no stop: the gate opens first, then the step's turn
  (`walk_step`'s, animated) and group 20. The box stays hidden through the stop and the opening
  (+0x30 from 0x442145): the opening's redraws draw the walker without it.
- **The closing** follows the walk's whole redraw (and a plain move's turn back), states 1, 2, 3
  at trunc(100 F); the closed state's line repeats nothing (the walls' line lists changes). A
  stack on 94 in a Fortress keeps the gate open (`gate-fortress`), the engine's `GateClosed`
  coming only with the step off it.
- **The moat** (`moat`: a Castle town's Citadel, the keep commanded): the closed drawbridge's
  hex (95) is no hex an attacker may enter (the engine's walls, BAT-SIE-003), so the Champions'
  front enters the moat at 77. The walk's end as any (group 21, group 2 through the deadline),
  then at once the line (named by the town's moat, GENRLTXT 377 with ARRAYTXT's name) and the
  reaction without a striker; then the box's whole redraw. In round 2 their Wait: its line, then
  the re-entry's damage, its line and reaction, without the walk's sounds. Found by the frame
  check: **a standing wall's hex takes a count box's place** (the hex records' blocked flag,
  `draw_battle_stack` 0x43E629): the Champions' front at 77 faces the wall at 78, so their box
  stands 37 pixels left (the hex ahead taken, §4.3); the scene's taken hexes gain the record's
  wall hexes but those of the parts destroyed since (`Walls::blocking`). With it `moat`'s frames
  exact but the Catapult's first frames' outline rows (T2b's), the strip over the front and the
  box drawn again as read (§10.1).
- **The drawbridge's rectangle** (CLT-Q23): its redraws show it alone, so a walker's box hidden
  at the walk's start stays on the screen outside it (`gate`'s round 2: the Hobgoblins' box at
  93, 319 pixels, from the first state's showing at 13 234 to their turn's first frame at
  13 534): reproduced in the original's profile (`Timeline::bridges`, `Moment::bridge`: the
  battle shown before, the moment's inside the rectangle), the whole battle in the fixed one.
  With it `gate`'s frames are exact but T2b's listed differences (the Catapult's first frame's
  outline row, 11 pixels at 100 and 9 359; the status line's ` log` hint, T2c-2).
- **A Tower's mine** (`mine`: a Tower town's Citadel, the keep commanded, the spectator's
  record): the moat's column holds only mines between the walls and the second column, so a walk
  passes one only through a breach: the Catapult destroys the mid-upper wall (2 forced, its wall
  hex 78 opens, BAT-SIE-012) and the Titans walk from 35 into the town (79), over the mine at 61.
  After the step onto it, at once: the line (named by Land Mine), the blast's sound, animation 57
  (`C09spF3.def`, 17 frames) over them with their reaction; then the walk goes on with the
  moving group (no group 20). The screen never shows the mine on the attacker's turn. Found by
  the timeline check: **without a striker a reaction holds its last frame** while the iterations
  left are more than its frames (`strike_animation` 0x468EBA, 0x468FA1): the Titans' 7 frames of
  group 3 under the animation's 17 stand at iteration 9 (max(R, L − 1 − R)), not 7; no earlier
  recording had an animation longer than 2R + 1 over a struck stack (`thunder`'s 12 over 9 frames).
- **Differences listed** (the frame check's `frames` files): every sequence's Catapult's first
  frame leaves its standing frame's outline top row (11 pixels, T2b's) with the status line's
  ` log` hint (T2c-2's); in `mine` the walk's first frames after the blast show the Titans'
  standing frame under the moving one where the saved screen comes back under the stack's
  rectangle (11 431: 252 pixels beside 234 counted as artifacts; 11 507: 5; §7.11 (2), not
  copied), and the last frame (12 799), the Goblins' turn's start, the defender's view in hot
  seat: its mines on the moat's hexes (4 005 pixels), which a spectator's record lacks (T1b's
  side to act's view). The rest of each sequence's frames exact.

**Recorded (M4 T2c-2, 2026-10-26).** Earthquake, the shots through the walls, the town's end and
the siege's mouse (not cut, the user). The viewer plays and draws the reading:

- **Earthquake** (`anim::siege::quake`, from the cast's effects: `WallQuaked`, `WallDestroyed`):
  the spell's sound at the handler, the shake as `QuakeKey` spans (`Timeline::shake`: the steps
  shown; the cursor hidden from the shake to the end's redraw), drawn by the frame's assembler as
  `Draw::Shaken` (the frame's whole list, battle and panel, composed and copied at each offset over
  the step before, the battlefield's 800 × 556 shown, the panel as saved: every profile, CLT-Q19
  (6)); then `WallHit` and the explosions as one blast with a landing per target hit (`Landing`:
  each frame's redraw, then the frame at every target in turn, the list within each landing's
  rectangle); the damage at frame 5 after its redraw in the original's profile (its new pictures
  from frame 6 in the rectangles, elsewhere at the end), before it with the slip fixed (a wall
  shot's way). No hit, or no siege's stage: the damage at once after the shake.
- **The forced targets** (`quake_target`, `xdd.c`): a draw per hit, the k-th standing target;
  `roll:quake_target:<a>,<b>,...` forces the hits' draws in order (`xdd.c`'s queue of 8 values;
  the other rolls unchanged).
- **The end tune at a town** (`defended`): the result's window's town is the record's `Siege`
  (`end_tune`'s `town`), so the defender's victory plays `defend castle`, its defeat `losecastle`.
- **The mouse** (`mouse`): the keep's and the upper tower's rectangles in a siege (the original's
  in every battle for the comparisons, CLT-Q21; the layout's `keep` and `tower`); the kinds 16
  (`Kind::Wall`, the target standing: the server's `ShootWall` commands; the towers' with a
  Castle, the keep's with a Citadel; over an empty hex, any for the Catapult, the targets' aim
  hexes) and 22 (`Kind::Turret`, the query's frame 5; over hex 183 with a Castle where no move
  goes); their hints "Attack %s" with the target's part's name (the key family `wall`, WALLS.TXT's
  parts by town type, `wall.<faction>.<part>`) and "View arrow tower info" (`battle.hint.turret`,
  GENRLTXT 157), neither forced. The cursor's wall test (`mouse::walls_halve`: from the shooter's
  position to the mouse's hex, the row's wall hexes 0x63BD00, the line `mouse::line` in integers,
  checked against the engine's x87 steps in the binary's tests) before its distance test; the
  shooters it exempts by their original ids and the bows by the wire's `no_distance_penalty`
  (`pointer::legal`, *provisional*: the M4 review list); the arrow tower always shoots in full.
- **The status line at a human turn with the mouse parked** (found by the recaptures): the
  siege sequences in hot seat keep the last hint through the next human turn (`hit`'s and
  `missed`'s Catapult's hint at the Griffins' turn, 3 775 and 3 375: the ` log` masks T2b and
  T2c-1 listed), while `walk`, `turn` and `climb` (one human side) empty it as the model did
  (§9.10). The tracker writes the empty hint outside the field over no control only with one
  human side (*provisional*). The dispatch, read: outside the field the battle's move goes to the
  window manager (0x475512, 0x602320), each dialog's items take it (0x5FEDE0), and an item takes
  a move inside it only on entering, the hovered item being another (`hovered_item` 0x6AAC68,
  forgotten only when that item is destroyed); then the combat dialog writes its hint, an empty
  line for an item without one (0x472B80; nothing while the log's new entries show). So a move
  back over the item the mouse last entered writes nothing, the hint stays. Not read: which item
  lies under the park point, and what makes it entered again. One case the rule misses: `aim`'s
  line emptied at the Pikemen's turn after the Catapult's shot (8 175: its last 8 frames listed
  ` log`), where `hit`'s, the same shot, keeps it. **Settled at M4 T3b-1** (below): the
  hovered item, read per step.
- **Differences listed** (the frame check's `frames` files): the acting shooter's first frame
  leaves its standing frame's outline top row (T2b's: the Catapult's 11 pixels in `quake`, `aim`
  at 4 500 and `through`, `through`'s Archers' again at 6 991; the keep's 4 in `defended`);
  `defended`'s last frame (1 487: the result's window over the darkened screen, not drawn yet, as `end`'s, M3 T8); `aim`'s
  last 8 frames' ` log` (above). The rest exact: Earthquake's shake and explosions, the broken
  arrow, the end tune's frames. T2b's and T2c-1's nine with the Catapult's hint, recaptured:
  their ` log` masks gone, every other count as recorded.

**Recorded afresh (M4 T3b-1, 2026-10-27).** T3b cut in two by the user (T3b-1: the hovered item
read, the stills, the 15 siege sequences and `walk`, `turn`, `climb`; T3b-2 the other 29). The
harness reads `hovered_item` 0x6AAC68 per step (`h3mem.Mem.hovered_item`: the item's id, kind,
dialog, box, state and hint, in `states.jsonl` and `frames.jsonl`), read-only.

- **The hovered item at the park point** is the panel's picture, item 2000 (the vtable
  `dialog_picture_create` 0x4502D0 gives; its dialog the combat dialog, its box (0, 556, 800,
  44), state 6, no hint): no control lies under (618, 593). The opening enters no item: `hovered_item` is 0 in every sequence until the first
  move into the panel after it, the mouse resting where the harness placed it before the battle.
  That first move enters the picture and writes an empty line (nothing while the log's new
  entries show, 0x472B80): a human turn's post of the parked mouse (`walk` 2 592, `turn`
  2 648, `climb` 2 396, `keep` 1 654: the second human turn; `hit`, `missed`, `automatic` at 0,
  the turn after the opening's defend) or the queued park after a click. From then on the park
  point writes nothing, the hint kept, until another item is entered (a control's hint) or the
  hovered one is destroyed (the spell book's items as it closes, 0x5FE442).
- **`aim`'s exception explained:** its mouse moves over the battlefield only (600, 40 to
  630, 532) until the Catapult's click, so the panel is first entered by the park queued after
  the shot, at the Pikemen's turn (8 176): the line emptied. `hit`'s defends park the mouse at
  the opening's turns, so its Griffins' turn finds the picture hovered and keeps the hint.
- **The viewer** (`mouse::track`): the hot-seat proxy gone; the tracker keeps the hovered panel
  item (`Handler::hovered`) and writes an item's hint, an empty line without one, only as the
  mouse enters it; the first input (the mouse resting where it was placed before the battle, the
  harness's park) enters none; a book (`Input::Book`) forgets it (*provisional*: the book's
  items not read per frame; the M3 sequences with casts, T3b-2). The panel's picture takes the
  mouse by its layout's `size` (`ui::Picture::size`, the layouts' format: the M4 review list).
  The battlefield's last hint so stays over the panel's background in live play too: CLT-Q24
  asked, answered (a) at M4's review (T3c): the client forgets the hovered item over the
  battlefield, the original's for the comparisons; built as the original's meanwhile, the
  switch in M5 T2.
- **Captured:** the 15 siege sequences and `walk`, `turn`, `climb` as recorded but `aim`, whose
  last 8 frames' ` log` masks are gone (its counts recorded again); the 72 stills (99 frames) as
  recorded; `drawn` the same for all but `aim`.
- **T3b-2 (2026-10-27): the other 29 sequences** as recorded, `strike`, `luck` and `bolt` at a
  rerun (the original's variation between runs: a hero's reaction read a step late; the stacks'
  luck 0, then 3; Orrin's spell power 2, then 1: the starting bonus likely, runbook "Traps"),
  `sacrifice` with the engine's seed 9 (the original's Grand Elves 23 in three runs, 22 at
  T9d's). The hovered item changes once in every sequence, from 0 to the panel's picture at the
  first move into the panel; `states.jsonl` is read after each action, never inside a cast, so
  the 11 casts' sequences show the picture after each cast's park whether the book forgot it or
  not: the book's forgetting stays *provisional*, every frame as recorded either way.
