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
replay of a network game; the siege branches are named, not read (E04c). Coordinates are screen
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
  k the spell table's first field (0x43E6AA to 0x43E73E; not built: M3 T7b). The count is `%d` of
  +0x4C, or of +0x50 when it is not −1 (below, "The number"), written by `TextDraw` (§3.6)
  in `tiny.fnt` into (x, y, 30, 15) with colour 4 (`game.pal`'s entry 13, white) and alignment 1
  (centred, from the top). A count of seven digits or more shows its first five (the box's last
  line cut at the glyph that fits, §3.6).
- **Built** (M3 T4a): the scene's `Draw::CountBox` after each stack's sprite (and before any
  magic animation over it since M3 T5b-2b) for a living stack, not a war machine, in group 1 or
  2, by its pose's hex and facing; the hexes ahead taken by every living stack's poses and the blocked hexes; the
  layout's `[count_box]` holds the numbers. Every creature's troop-count offset is 0 in the
  harvested tables. Checked: the capture set without the count-box mask, 31 situations, all 57
  frames exact (§6.2). Not yet: the spells' tint (M3 T7b, with the first sequences that cast on
  stacks).
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
2. Siege parts (not read).
3. The left flag and hero, then the right ones.
4. For each row, top to bottom; in it for each pass 0–7; in it for each column 0–16, left to
   right (right to left in a siege's rows 6–10): pass 0 a siege wall part (not read); pass 1 the
   corpses on the hex, in their list's order (a two-hex corpse from one of its hexes, as a stack);
   pass 2 the layer-0 obstacle anchored there; in every pass the living stack on the hex if its
   **draw layer** (+0x8) equals the pass, then its count box and a magic animation over it (in
   a siege the moat's parts and the box again, §4.2).
5. Siege towers, then the calls at 0x494AAC and 0x494ABC (not read); then the frame is shown.

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
  effects; the siege branches (E04c).

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
  at 2592; at speed 1 the frames are 62 and 35 ms apart, at speed 2 40 and 22.
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
  A + R - 1), A the striker's group's frames, R the longest reaction's. The striker plays its
  group from iteration 0: by the strike's direction 11 (up), 12 (straight), 13 (down); 17-19 for
  a breath on two stacks where the DEF has the group, else 11-13 (`melee_strike` 0x441ACA to
  0x441B66; its second stack, [ebp − 8] at 0x441A35, is set only for a creature with flag 0x8,
  0x44171B; the Ballista takes its shot groups). The viewer tells a breath by its second stack
  standing beyond the striker's neighbours, where a multi-headed striker's stand beside it
  (the record does not name the ability; E17 M2 T5c; confirmed by the sequence `fly`, §7.13). Each struck stack reacts
  once the iterations left are no more than its reaction's frames, so the reactions end with the
  loop (with one striker they start at its second-to-last frame, iteration L - 1 - R = A - 2): death 5 (its last
  frame stays: the corpse), defend 4 (a defending stack), else hit 3; then they stand. The
  pending spells are cast afterwards. Before a retaliation and before a second strike
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
  Marksmen's sequence shows; the reading's L = R: *provisional*.) The shooter plays its group
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
- **Not read:** Chain Lightning's call (0x5A64EF) and the colours 300, 301, 303 and 305 (the
  spells, M3 T7c); the Enchanters' shot (0x59FDE0).

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
- *Not read*: the start sound's volume (its loader's default), the battle's end (`battle_end`
  0x477470 calls `play_music` twice: the victory and defeat tunes and the results), spells', heroes',
  walls' and towers' sounds, the turn's, the adventure map's.

**The viewer.** `h3_view::anim`'s timeline carries the cues (`Cue`: a time, a stack, a `Sound`,
start or stop) at the frames the readings name; `play` writes them to `cues.txt`.
`h3_view::sound` holds the opening: the start sound and the music are drawn from the viewer's
own C-library sequence seeded with the battle's seed (*provisional*, as the fidgets; the
executable draws from its rules' generator), the fade-in's volumes, the volumes 127 and 64. The
window (`bin/h3-view/audio.rs`): every sound the battle's stacks name and the start sound are
decoded when the battle loads, the music by `nanomp3` (i16; the battle's 44 clips and a
3-minute track in 0.4 s in the sandbox); `h3-sdl` opens one device and 10 streams: the music's,
the start sound's and 8 creature channels, each a free one or the next in turn, as
`play_sample` takes them. A one-shot is flushed; the move sound
and the music are queued a second ahead and topped up (no flush, so no seam). Volumes as SDL
gains: the music's fade-in by its steps, the effects at 64/127 (*provisional*: both settings
equal; no settings yet). From the battle's start the window plays the start sound over the first
frame with the clock held, then starts the music and the clock (a key that moves the battle ends
the wait); Space pauses the device with the battle (the fade-in holds with it), R and → stop
the creature sounds and go on from the cues at the new time (→ keeps the next frame's, cued as
the frame stepped to shows), M and E switch the music and the creature sounds, `--mute` plays
nothing; a sound that does not decode costs that sound, as a missing one (T5c, the reviewer).

**Checks.** The gate (above); the cues' unit tests (each with its group's first frame); in the
sandbox under Xvfb with SDL's `disk` audio driver (it writes the mix to `sdlaudio.raw` in the
current directory): `--check 25` with sound, all 293 frames read back exact; the start sound
5.8 s (`BATTLE01`, seed 1), then the music fading in; with the music switched off (M), the
effects' onsets in the mix within 10–100 ms of their cues' times (onset detection on 10 ms
windows; overlapping sounds hide some).

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
  tactics phase's panel, Auto Combat switched on. The situations' and sequences' scenarios name
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
  | good, bad morale | at the event (*provisional*: their animations are T7b's) | rows 34, 35 |

  Not built: the lines the engine's other events write in the original (regeneration, row 372 at
  0x44707B; mana drain, 374 and 375 at 0x4657AD; the spells, Death Stare, Life Drain, Fire
  Shield, the moat, fire walls, land mines, turrets: T7's). The striker is named by its number
  before the strike (a retaliator's after the strike's kills), a defend's gain by the snapshot's
  defense.
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
heroes' rectangles (0x4626A0, 0x4626D0), `battle_cursor` (0x476490) in full, the attack's slots
(`cursor_attack_slots` 0x474690, `cursor_attack_from_hex` 0x474A00) in full, `battle_status_line`
(0x492840) for the cases below, the mouse manager's `SetCursor` (0x50CCA0), its frame drawer
(0x50D8B0) and hot spots (0x50CD90's table 0x67FF50), the panel's hints (`battle_panel_base`'s
`SetHints` 0x5FE840 from 0x6A6968 on; `Help.txt`'s loader 0x5B98B0).

- **Where the mouse is** (`screen_to_hex`): first four rectangles, bounds inclusive: the
  attacker's hero (x 0–57, y 16–127) is "hex" 0xFC, the defender's (741–799, 16–127) 0xFD (both
  static, 0x4626A0, 0x4626D0), two siege ones 0xFE and 0xFF (not read). Else sx = x − 14,
  sy = y − 86, row = sy / 42, on an even row sx −= 22, column = sx / 44 (C division); outside
  0 ≤ sx < 748, 0 ≤ sy < 472 none (−1). In a cell's top 10 pixels (sy mod 42 < 10) a point above
  the hex's slanted edge, sy mod 42 < |sx mod 44 − 22| / 2, is the row above's: row − 1, and on
  the left half (sx mod 44 < 22) column − 1 when that row is even, on the right half column + 1
  when it is odd. A row outside 0–10 or a column outside 0–16 is none; else row × 17 + column.
- **The mouse move** (0x4753A4): outside the battlefield (x outside 0–799 or y outside 0–555)
  the marked stack is unmarked (redrawn), the dialog's items get the message (their hints,
  below), the cursor becomes the combat cursor's frame 6 and the mouse's hex (+0x132D4) none.
  Inside, on a new hex the hex's stack is marked (`battle_mouse_stack`, §7.13), the kind is
  `battle_cursor(hex)` (+0x132E0), the cursor set by it, and the status line written for it
  (`battle_status_line`); on the same hex only an attack's slot is computed again. The army
  info windows (+0x132FC's six, shown when "Combat Army Info Level" 0x6987C8 is set) stay hidden
  at its default 0, which the harness keeps.
- **The kind** (`battle_cursor`, the acting stack's view; the battle lane read its rules,
  `installation.md` §7.14, §7.21): none (−1) or a side column (column 0 or 16) 0; a hero's
  rectangle with its hero 4 (the acting side's) or 21 (the other's), without one 0; in a network
  game on a turn not played here 5 over a stack, 6 elsewhere (0x69954C, not read; +0x132B4, the
  turn played here); sieges' catapult targets 16 and the arrow towers 22 (not modelled). A stack's
  hex: 5 (query) in the tactics phase (+0x13D68) and over the acting stack itself; a Faerie
  Dragon's cast 20 (`creature_cast_target_ok`; not modelled); the acting side's own stack 17 for
  the First Aid Tent over a damaged patient (not modelled yet), else 5; an enemy (side +0xF4 not
  the acting side): 3 (shoot) if the stack can shoot now (`stack_can_shoot_now`) and is not the
  arrow tower, else 15 (the broken arrow) when `shot_wall_test` or `shot_full_damage_distance`
  says halved (the latter with the shooter's current facing, not turned: 0x4767E6, NOTES); without
  a shot 7 (attack) if `cursor_path` reaches it (3 for the Ballista), else 0. An empty hex: the
  Catapult's wall targets 16 (sieges), a caster's empty-hex spell 20 (not modelled), else 2 (fly,
  the stack's flag 2) or 1 (move) when the reach map marks the hex (+0x20E or +0x20F), else 0.
- **The cursor** (0x47587C, 0x474B85): the kind's frame of the combat cursor (`SetCursor(frame,
  2)`, `crcombat.DEF`), except 7, whose frame is its slot's, 20 (frame 0 of `crspell.DEF`, type 3,
  animated: not read) and 21 and 22 (frame 5). The types' DEFs (0x67FF38): 0 `crdeflt.DEF`, 1
  `cradvntr.DEF`, 2 `crcombat.DEF`, 3 `crspell.DEF`, 4 `artifact.DEF`. Each type's frame has a hot
  spot (0x67FF50 + (type × 144 + frame) × 8, x then y): the cursor's canvas is drawn with its top
  left at the mouse less the hot spot. The frame is drawn by the plain drawer with runs skipped
  (0x47BEB0 → 0x47C570, flag 1) into a surface first filled with the colour key (the screen's green
  and blue masks: 0x07FF in 5-6-5), which the screen's cursor overlay leaves out.
- **The attack's slot** (`cursor_attack_from_hex`): dx = x − (44 × column + 36, 22 more on an
  even row), dy = y − (42 × row + 112), r = |dx| / |dy| as f32 (100 when dy = 0), against the f64
  constants 0.58 and 1.73 (exact in integers here: 50 |dx| against 29 |dy|, 100 |dx| against
  173 |dy|). dx ≥ 0, dy < 0: slot 0, 1 above 0.58, 2 above 1.73; dx ≥ 0, dy ≥ 0: 3, 4 below 1.73,
  5 below 0.58; dx < 0, dy ≥ 0: 6, 7 above 0.58, 8 above 1.73; dx < 0, dy < 0: 9, 10 below 1.73, 11
  below 0.58. Clockwise from the top: 0, 1, 2 (right of the top to the right's upper half), 3, 4,
  5, then 6 to 11 on the left from the bottom up. Each slot has a from-hex (+0x133FC, the
  attack's +0x132D8) and a frame (+0x133CC) from `cursor_attack_slots`; the cursor changes only
  with the frame.
- **The slots** (`cursor_attack_slots`): slot s looks at the target's neighbour s / 2 (the
  manager's table at +0x13468, six per hex: up right, right, down right, down left, left, up left)
  as the acting stack's position: taken when the reach map marks it (+0x20E), the stack fits there
  (`stack_fits_at_hex`) and none of its hexes is a stop (the pathfinder's +0x68: a seen pit); a
  one-hex stack's frame is then 7 + (s / 2 + 3) mod 6. A two-hex stack also tries the position
  beside it (its rear hex's side by facing +0x44); slots 2 to 3 and 8 to 9 (s / 2 = 1 or 4) take
  the beside position when only it fits, the others pick by facing and by s (≤ 2 or not) which one
  to take, frame 13 for s 2 or 3 and 14 otherwise when the beside position wins, unless s is 0, 5,
  6 or 11 with both fitting. A slot left without a frame copies, in slot order, the first slot j
  with one at the least d(|s − j|), d(n) = n up to 6 and 6 − n above (so 11 beats 1 for slot 0,
  and 7 beats 1: the executable's choice), the copies made earlier counting for later slots.
- **The status line** (`battle_status_line`, written by `battle_log_add(text, 0, forced)`,
  §9.8): 0 "No shots left!" (GENRLTXT:299) for a shooter (flag 4) with no shots left and a target
  still set (+0x10, +0x14, which a hex outside the field or a side column leaves as the last one
  set: T6c-2), else empty; 1 "Move %s here." (295) and 2 "Fly %s here." (296), the acting stack's
  name by its count; 3 and 15 "Attack %s" (221) with the target's name by its count, forced,
  while the army info is off (else 297 or 38 with the damage: not modelled); 4 "View Hero Stats"
  (418), 21 "View Enemy Hero Stats" (419); 5 "View %s info." (298) with the hex's stack's singular
  name, empty over no stack; 7 "Attack %s" (221), forced (with army info 37 and the damage: not
  modelled); 17 "Apply first aid to the %s" (420); 16, 20 and 22 sieges' and casters' (not
  modelled); every other kind empty. After 0, 1, 2 and 5, a caster (Archangel 13, Pit Lord 51,
  Ogre Mage 91) may replace it and force it (0x4922F0, not read further: not modelled).
