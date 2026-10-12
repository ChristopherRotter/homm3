# E17 log: done tasks in full

The full record of E17's finished milestones and tasks (what was built, agent decisions marked
provisional, measurements, the review lists). **Not start-of-session reading:** the epic file keeps
one line per done milestone; read the matching section here when a task builds on an earlier one
or a decision is questioned. Decisions live in ADR-0020; this log only records how and why the
work went.

## Done

### M0 — The workspace (2026-10-12)

One session (the user present). Pull requests: the client lane's `client/e17-m0`; the shared
`shared/ci-outage` (CI may be unavailable, below), `shared/adr-0020-clients` (the ADR, CLT-Q2) and
`shared/clients-ci` (the job). Merge order: `shared/ci-outage` and `shared/adr-0020-clients`, then
`client/e17-m0`, then `shared/clients-ci`: the job runs `clients/scripts/check.sh`, which the client
branch adds, so the job's pull request would go red against a `master` without it.

- **CI out (billing).** At the start, GitHub stopped every run ("recent account payments have
  failed or your spending limit needs to be increased"; `master`'s run for #118 never started). The
  user's rule, written into the process docs by `shared/ci-outage`: CI never blocks work; while it
  cannot run, the full local verify stands in, its result recorded in the pull request. For the
  client, that local verify is `clients/scripts/check.sh`.
- **T1, the workspace.** `clients/Cargo.toml` (resolver 3, edition 2024, GPL-2.0-or-later, the
  engine's lints and codegen profiles copied); `h3-formats` as a path dependency: its
  `workspace = true` fields resolve from the engine's root manifest, no change needed. The root's
  `clippy.toml` (the disallowed types and methods), `rustfmt.toml`, `rust-toolchain.toml` and
  `.cargo/config.toml` apply to `clients/` as they are (the tools search the parents; checked:
  a `HashMap` in `h3-view` fails clippy). `clients/deny.toml` copies the engine's policy.
  *Provisional:* the libraries of `h3-assets`, `h3-render` and `h3-view` are `no_std` with the
  engine's logic lint header; `h3-assets` also denies `clippy::indexing_slicing` (decoders over
  untrusted bytes); `h3-view`'s binary (`src/bin/h3-view/`) holds the adapters; the engine's
  `check-arch` does not see `clients/`, so `h3-view/tests/purity.rs` checks the headers, every
  source of the pure crates (no denied lint relaxed, `std` only under `#[cfg(test)]`) and an
  allowlist of each pure crate's dependencies.
- **T2, the job.** `clients/scripts/check.sh` (client-owned) runs fmt, clippy, the tests,
  `cargo deny`, a check that `clients/deny.toml`'s licences and sources stay the engine's (until
  the M2 licence decision), coverage and a check of the job's path filter; every cargo call
  `--locked`; a failing gate prints all of its output. `.github/workflows/clients.yml` (shared)
  runs it on Linux for pull requests and `master` pushes that touch `clients/`, `crates/h3-formats`,
  the root `Cargo.toml`, `.cargo/config.toml`, the toolchain or lint settings (`!**.md`), one run
  per push, no matrix, no fingerprint (a light job; a merged pull request runs once more on
  `master`), and weekly `cargo deny check advisories`. *Provisional:* coverage is cargo-llvm-cov's
  `--fail-under-lines 100 --fail-under-regions 100` over the pure libraries, `/src/bin/` left out
  (the engine reads the merged segment view instead, `xtask/src/coverage.rs`, because generic code
  shows false region misses in the summary: port it when that happens); the path filter's
  completeness comes from `cargo metadata` (every path package outside `clients/` must appear in
  both lists). The fixes' ledger: no `docs/rules/quirks-client.md` and no `lanes.toml` change; the
  asset tables are the ledger (ADR-0020 point 6; *provisional*, M1 T4 confirms). The job ran
  nowhere yet (CI out): verified by running `check.sh` with the workflow in place, both ways (the
  path check fails when `crates/h3-formats` is dropped from the filter).
- **T3, the real-asset gate.** `h3_assets::archive` (pure): an archive's inventory (entries,
  compressed, unpacked bytes, by extension). `h3-view gate [--install] [--out]` (adapter): finds the
  installation copy (`--install`, `$H3_INSTALL`, the repository's `original-data/install`; file
  names matched ignoring case), reads and parses the four LODs through `h3-formats`, prints the
  inventory and the times, lists every entry in `clients/out/gate/entries.tsv`. On the user's
  files (staged into the sandbox): `H3bitmap.lod` kind 200, 4 908 entries (PCX 4 809, TXT 71, H3C
  14, FNT 9, PAL 3, IFR 1, XMI 1); `H3sprite.lod` kind 200, 4 013 (DEF 2 565, MSK 1 448);
  `H3ab_bmp.lod` kind 500, 2 491 (PCX 2 406, TXT 71, H3C 14); `H3ab_spr.lod` kind 500, 569 (DEF 347,
  MSK 222). 11 981 entries, 212 MiB stored, 394 MiB unpacked; read and parsed in about 0.2 s (warm
  cache). The reads dominate; parsing all four tables takes under 10 ms.
- **The review** (a fresh agent, before the checkpoint): twelve findings; fixed: the purity check
  widened from the roots to every source and the dependencies, `--locked`, full output on failure,
  a useless `allow` dropped and `#[expect]` with a reason for the clock, the README, the deny
  policy check, `.cargo/config.toml` in the filter, the path check over both lists, the docs'
  promise of a client fingerprint corrected (`ci.yml`, `05-testing.md`). Recorded: the merge
  order, the fixes' ledger decision, the coverage caveat, and point 7 against whole-file parsing
  (below).
- **The checkpoint.** The user accepted the ADR draft as ADR-0020 with point 7 rewritten: the
  client loads in batches at checkpoints (the client opens: the menus; a game starts: the
  adventure map's assets; a battle starts; …), never one asset at a time while a screen is shown.
  The bytes: three ways weighed against that model. A table-only reader in `h3-formats` (the
  directories at start, a batch's entries sorted by offset and read at the checkpoint; an engine
  change), memory-mapped archives (no engine change; `unsafe`, a crash instead of an error when a
  file changes, implicit page-fault I/O, no mapping in a browser) and whole archives read at start
  (212 MiB read and held, no path to the 629 MiB `VIDEO.VID`). Sizes from the inventory: a battle's
  batch is about 3–5 MiB (a creature DEF about 125 KiB stored; the 25 battle backgrounds 6.9 MiB),
  the adventure objects (`AV*.DEF`, 1 548) 9.3 MiB. Chosen: the table-only reader, filed as CLT-Q2
  for the battle lane; until it lands the adapter reads whole archives.

### M1 T2 + T3 — The decoders and the inventory (2026-10-12)

One session (the user present; they chose T2 + T3 over T1). Branch `client/e17-m1-decoders`, one
pull request. Findings in [`E17-design.md`](E17-design.md) §1–2.

- **The formats first, from the files.** Before writing code, a throwaway script read the user's
  LODs: the three PAL files are RIFF palettes; every PCX has exactly the community's layout (24-bit
  stored blue, green, red: checked by eye on `Ar_Bg.pcx`); every DEF frame decodes with the four
  known encodings once the 16-byte header of `SGTWMTA`/`SGTWMTB` is recognised (VCMI 1.7.5's
  `CDefFile::loadFrame` has the same case; without it 740 rows overrun); every MSK has 14 bytes.
- **T2.** `h3_assets::palette` (`Rgb`, `Palette` from or to 768-byte triples, `decode_pal`),
  `h3_assets::pcx` (`decode_pcx`: `Pcx::Indexed`, boxed for its palette, or `Pcx::Rgb`); one error
  type for the crate (`h3_assets::Error`: truncated, trailing bytes, magic, a named field, overrun,
  too large); a private bounded reader (`read`: every read answers `None` past the end, a header
  read as one array so that a cut header has a single failure path).
- **T3.** `h3_assets::def` (`parse_def`: header, palette, groups; `decode_frame` by offset;
  `Encoding` names the four encodings: `Raw`, `Rows`, `PackedRows`, `PackedBlocks`; `Frame` keeps
  the canvas, the frame's place in it and whether it had the short header), `h3_assets::msk` (the
  raw masks; their meaning waits for research item 9). `h3_assets::check` decodes an entry by its
  extension and reports findings that do not stop decoding (archive type against content, a
  frame's canvas, a frame outside its canvas, the short header, a failing frame).
  `h3_assets::survey`: duplicates across archives; the names that text mentions (exact tokens and
  `printf` patterns with `%c`, `%d`, `%0Nd`, `%s`; a pattern needs three literal characters before
  its extension; an MSK counts through its DEF); the 8.3 check.
- **Synthetic assets.** `h3_assets::build` (public, as `h3_formats::lod::build`): encoders for PAL,
  PCX, MSK, DEF and single frames in all four encodings, for this crate's tests and those of the
  crates above it (T6, T7). Round trips through the decoders are proptests (random frames in every
  encoding), as are "never panics" on random and mutated inputs.
- **The gate.** `h3-view gate` now inflates every entry, scans the text tables, `Heroes3.exe` and
  `h3maped.exe` (an entry only the editor names is not dead; both optional), decodes every PCX,
  PAL, DEF and MSK, compares duplicates byte by byte, writes `entries.tsv` (each entry's verdict,
  findings, mention and duplicate status) and `report.md` (the inventory), and fails if any entry
  does not decode. On the user's files: DEF 2 912, MSK 1 670, PAL 3, PCX 7 215, none failing;
  release build about 3 s in all (debug about 30 s).
- **`check.sh`.** Its CI-paths step passed `cargo metadata`'s JSON as an argument, which outgrew
  the system's limit once proptest's dependencies joined the lockfile ("Argument list too long");
  it now goes through stdin.
- *Provisional* (agent's choices): strict lengths and overruns as errors (`E17-design.md` §1),
  `def::MAX_SIDE` = 16 384, per-row offsets for encoding 2 (VCMI reads on from the first row; the
  same on every file), the report's layout, the pattern rule's three literals.
- **For the review list:** `proptest` as a dev-dependency of `h3-assets` (the engine's version
  1.11.0 and features, pinned in `clients/Cargo.lock`; test-only, so not in the purity allowlist);
  `h3_assets::build` as public API.
- Gates: `clients/scripts/check.sh` passes (fmt, clippy, tests, deny, 100 % lines and regions of
  the pure libraries, CI paths); CI could not run (billing).

### M1 T1 — Code reading: archives, screen format, keyed colours (2026-10-12)

One session with T6 (the user present; they chose T1, then T6). Branch
`client/e17-m1-t1-code-reading`, a docs-only pull request. Findings in
[`E17-design.md`](E17-design.md) §3; 35 entries in `docs/exe/map.toml`.

- **Path.** From H3API's guesses (`H3Lod::FindFileInLod`, `H3LoadedDef::Load`,
  `H3BasePalette565::InitiateFromPalette888`, `H3DefFrame::DrawToPcx16`) to their callers: the
  loaders' search lists, the edition row, the archive slots; from the palette conversion to the
  DirectDraw setup that hands it the masks; from the battle's redraw (H3API's
  `H3CombatManager::RefreshCreatures`) to the stack's draw and its frame drawer, then the other
  drawers by their wrappers. Each drawer's runs read through its switch tables (`exe.py switch`).
- **Checked against the files** with throwaway scripts (not committed): the four directories are
  sorted by lower-cased name; each kind lives in one family of archives; per DEF type the
  encodings and the keyed indices used; the runs of index ≥ 8 (`Cnomad.def`'s 42 of index 8).
- **VCMI 1.7.5** compared: the archive order (`config/filesystem.json`), the keyed colours as
  alpha (`client/renderSDL/ScalableImage.cpp`).
- **Not read** (§3.5): three drawers, the PCX drawer, the outline colour's index (T5),
  DDrawCompat (licence first).
- Gates: `scripts/exe.py check` and `cargo xtask check-exe-map` (every cited address covered),
  `check-docs`.

### M1 T6 — The compositor (2026-10-12)

Same session as T1, on `client/e17-m1-compositor` (pull request #136). The code reading of
[`E17-design.md`](E17-design.md) §3 turned into code.

- **`h3_render::screen`.** `PixelFormat` (red, green, blue widths, packed red high; `RGB565`,
  `RGB555`): `pack` truncates as 0x5226D0, the half and quarter masks and the two shadows as
  0x47C460 builds them, `unpack` widens for display. `Screen`: 16-bit pixels, the size a
  parameter (at most 16 384 a side).
- **`h3_render::palette`.** `ScreenPalette` (a palette converted once, as at load);
  `with_player_colours` replaces indices 224–255 (where the 32 colours come from is T7's).
- **`h3_render::blit`.** `draw_frame` with a `Drawer` (`Plain { skip_runs, colour }`,
  `Shadowed { colour }`, `ShadowPass`): the table of design §3.3 per encoding and per run or
  literal pixel; the frame placed by its canvas's corner, mirrored across its own canvas, clipped
  to the canvas and the screen. `draw_image`: a PCX opaque (*provisional*: the image drawer is not
  read). Left out: 0x47C9E0's 50/50 blend (no caller enables it).
- **`h3_render::png`.** RGB8, filter none, zlib by `miniz_oxide` (already in the lockfile through
  `h3-formats`), CRC-32 bit by bit; checked against the standard check values.
- **`h3_assets`.** The drawers tell runs from literal pixels (a literal 6 is a colour, a run of 6
  a shadow; a run of 8 is skipped), which the flat indices lost: `Frame::runs` marks each pixel,
  the decoders fill it, `build::frame` follows it, and `build::runs_by_repetition` gives the old
  encoder's choice for synthetic frames. Literal keyed indices are rare in the user's files (23
  pixels of 6 and 7 in map objects).
- **`h3-view sprite`.** One group of a DEF, frame by frame on its canvas, over a PCX, through the
  shadowed drawer, into `clients/out/sprite/`; DEFs searched in `H3sprite` then `H3ab_spr`, images
  in `H3bitmap` then `H3ab_bmp` (design §3.1). On the user's files: the pikeman's standing group
  and the Nomad's (mirrored, shadows on sand, none of its index-8 runs shown), 0.2 s each.
- *Provisional*: the names, 565 as the default, the PNG's widening, the shadow pass applied to
  every encoding, opaque images, the sheet's layout.
- Gates: `clients/scripts/check.sh` passes (fmt, clippy, tests, deny, 100 % lines and regions,
  CI paths); the real-asset gate still decodes all 11 800 image entries. CI could not run.
