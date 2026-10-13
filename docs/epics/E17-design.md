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

The drawers 0x47D4F0, 0x47E820 and 0x47EF60 (a half blend, rotations, H3API's `DrawTransparent`)
not read; how a 24-bit PCX is drawn, and an 8-bit one outside the cases of §4.4 (index 0 is
transparent only when the caller asks), not read; DDrawCompat's expansion (licence first); 555 not
modelled. The battle screen's open points: §4.7.

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
  by animations, M2). Part 0 draws the sprite, part 1 the count box (§4.3).
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

- Part 1 of 0x43E140, called right after the sprite: drawn when the manager forces it (+0x13D34),
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
- Then the box is tinted (0x44E940 over its inner 28 × 9 pixels: no active spells (0.75, 0.8), else
  by the spells' kinds; not read) and the count (+0x4C, or +0x50 when it is not −1) written in
  `tiny.fnt` into the box (M3).
- VCMI's `showStackAmountBox` gives the same four positions (its hex centre is the anchor −
  (0, 16)) but leaves out t and counts every stack ahead as taking the hex, war machines included.

### 4.4 Background and obstacles

- **The background buffer** (manager +0x53B0, 800 × 556), composed once by 0x493CF0: the
  battlefield picture (`CmBk*.pcx`, 800 × 556; which one: T4), whole (0x55CF00 with manager
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

### 4.6 The draw order

The redraw 0x494440, after the background:

1. Layer-1 obstacles, by anchor: rows top to bottom, columns 1–15 left to right.
2. Siege parts (not read).
3. The left flag and hero, then the right ones.
4. For each row, top to bottom; in it for each pass 0–7; in it for each column 0–16, left to
   right (right to left in a siege's rows 6–10): pass 0 a siege wall part (not read); pass 1 the
   corpses on the hex, in their list's order (a two-hex corpse from one of its hexes, as a stack);
   pass 2 the layer-0 obstacle anchored there; in every pass the living stack on the hex if its
   **draw layer** (+0x8) equals the pass, then its count box.
5. Siege towers, then the calls at 0x494AAC and 0x494ABC (not read); then the frame is shown.

- The draw layer (H3API calls +0x8 "visibility") is 4 from the setup (0x43D750); 6 for the
  attacker of a melee (0x441CC1); 3 or 7 by direction in 0x43F0B0 (directions 0 and 5: 3; 2 and 3:
  7). So standing stacks draw in pass 4, after the corpses and obstacles of their row, and a
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
- Not read: the grid and shading options (0x493930); the count box's tint and text; the stack's
  byte +0x4F1; the palette effects; the siege branches (E04c).

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
  `tests/tables.rs`).
- **Obstacles:** the type table 0x63C7C8 (`def`, `rows` = height, `layer`) and the absolute table
  0x63BEC0 (`pcx`, `at`), §4.4; keys `obstacle_<n>`: type n, absolute record n − 100 (VCMI's
  numbering, as the packs).
- **Expansions** (research item 6): an entry belongs to the table of the pack that defines its key
  (`base` 121 creatures, the terrains, 78 obstacles; `ab` 23 creatures; `sod` none). The 47
  obstacle types and 12 special battlefields no pack defines yet sit in `base.toml`, keyed as the
  packs would key them. Which archive holds a file plays no part: the loader searches as the game
  does (§3.1), and the gate finds all 307 names there.

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
  panel. The set runs unattended (nine situations in about 70 s) and fails when a frame's count of
  differing pixels changes.
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
- **Idle animation.** In the grass situation the Pikemen fidget from about 200 to 700 ms after the
  opening, identically on every run; its other three stacks stand still for 800 ms. Why: §7.2.
- **The panel** shows its buttons lit on the human's turn and dimmed on the computer's.

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
- **Not built yet:** scrolling the hero list (more than five heroes), the underground, a visit
  that asks a question (a Yes/No dialog), a battle started from the map (the adventure lane's
  increment 3: monsters and battles), a second day.


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
  hex 35 to 43 in k = 3 cycles of 58 ms frames from 300 ms; group 21 shows from 1518.
- **A turn** (`stack_turn_around` 0x446720, animated): facing right, group 7, the flip, group 10;
  facing left, group 9, the flip, group 8; each all its frames; then standing. Not in a sequence
  yet.

### 7.4 Strikes and shots

- **A strike's animation** (`strike_animation` 0x468990, H3API's ApplyAnimationToLastHitArmy):
  L iterations of trunc(100 F) on the frame deadline, L = max(a magic animation's frames,
  A + R - 1), A the striker's group's frames, R the longest reaction's. The striker plays its
  group from iteration 0: by the strike's direction 11 (up), 12 (straight), 13 (down), 17-19 for
  a strike on two stacks (`melee_strike` 0x441AAF). Each struck stack reacts once the iterations
  left are no more than its reaction's frames, so the reactions end with the loop (with one
  striker they start at its second-to-last frame, iteration L - 1 - R = A - 2): death 5 (its last
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
  shooter waits trunc(115 F) and draws its ray (not read). Then `strike_animation`: the shooter
  plays its group out while the target reacts; the deadline being behind, the reaction's first
  frame shows for no time at all.
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

- Not read: the teleporting move, a ray's drawing (H3API's DrawRay), the spells' and the magic
  animations' frames (H3API's PlayMagicAnimation), mouse-over (whether hovering starts group 1),
  the heroes' idle groups, the arrow towers' and the catapult's shots, the first aid tent, the
  sounds' timing.
- Not in a sequence yet: a turn, a death, Haste and Slow, a defending stack's reaction, a shot
  up or down, a ray.

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
- **Not yet (T4b-2):** fidgets from the viewer's own generator; turns (§7.3) and the battle
  lane's note that stacks turn to face their target for a strike or a shot; the draw layers of
  walks (3, 7) and strikes (6, the struck stack 5: §7.9); the spells' sprites; Haste and Slow
  (the walk time), rays, teleports, magic animations, deaths by spells, created stacks; the
  difference §7.9 lists.

### 7.9 The frame check (M2 T4b-1)

Every distinct battlefield the original shows in a recorded sequence against the viewer's own
frame at the same time: `h3drive.py frames` (runbook `clients/capture/README.md`). The harness
steps the clock 1 ms at a time and keeps each screen whose battlefield (rows 0–555) changed, with
its time and the game's state (`frames.jsonl`, the screens as `frames/t<ms>.npz`, local only);
`h3-view sequences <id> --frames <dir>` plays the sequence's record through `h3_view::anim` and
draws the moment shown at each kept time (`ours/`, and what it drew in `ours.txt`); the diff masks
the panel, the heroes' corners and the count boxes where the drawn poses show them (a stack in
group 1 or 2, §4.3; every living stack takes its hexes for the box's place). The only thing taken
from the original is a fidget (a stack the game's memory shows in group 1): the viewer has none of
its own yet (§7.2). Each sequence's counts per frame are recorded
(`situations/timelines/<id>-frames.txt`); the command fails when one changes.

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
- **The shooter after its climax:** `strike_animation` (0x468A3D) counts a striker still in its
  shot group with one frame less and advances each stack's frame before drawing it, so the
  shooter shows frames c to n − 1 in the loop and stands from the second iteration on, while the
  loop's length still counts n − c + 1 (T4a had frames c − 1 and c: the second iteration showed
  frame 7 where the original stands; the lines were the same).
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
  4 again after each (not drawn differently yet: T4b-2).
- **Memory against the screen:** after an animation the game processes a turn's start in the same
  millisecond after the harness's 20 clock reads, so a state read then can lag the screen by one
  step (the walk's idle counter read 1 at 2 592 while the screen shows the third step's colour).
  The check therefore draws the viewer's timeline, not the poses read from memory.

**Listed differences** (T4b-2): the melee at 3 504 to 3 966 ms (9 to 327 pixels), while the Wolf
Riders walk past the Angels: dark pixels at the Angels' outline edges (sword tip, robe), and at
3 966 the Angels' count box at its usual place under the Wolf Riders' legs where the viewer's
poses (the walk's end on hex 45) put it beside them. Not read yet: the redraw of a walk's frames
near another stack.
