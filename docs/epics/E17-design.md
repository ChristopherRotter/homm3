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
    documentation). The game's own rule is open (§3).
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
  objects such as `AH16_.DEF`, `AVA0127.DEF`). No archive holds a name twice. Which copy the game
  uses: research item 1 (T1).
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

## 3. Open for the code readings

For M1 T1 (or T5 where it is the battle's): whether the game reads the archive's type or the DEF's
(the 12 typed 0x41); how it reads `SGTWMTA`/`SGTWMTB`'s frames; which canvas it uses for
`OVSLOT.def`; whether it reads encoding 2's per-row offsets; which of two copies of a name it
loads (research item 1).
