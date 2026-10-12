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
16 384 are rejected; nothing is allocated that the input cannot fill. Raw palettes: the keyed
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
  the M3 diff compares 565 values.

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
  DEF's width (+0x30) plus 196, else at x − 196; y − 267. T5 reads the anchor and the outline.

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

The outline colour's index and when it shows, the battle hero's and the obstacles' drawers (T5);
the drawers 0x47D4F0, 0x47E820 and 0x47EF60 (a half blend, rotations, H3API's
`DrawTransparent`) and how a PCX is drawn (index 0?) not read; DDrawCompat's expansion (licence
first); 555 not modelled.
