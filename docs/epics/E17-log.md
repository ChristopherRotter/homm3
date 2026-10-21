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

### M1 T5 — Code reading: the battle screen (2026-10-13)

One session with T4 (the user present; they chose T5, then T4). Branch
`client/e17-m1-t5-battle-screen`, a docs-only pull request. Findings in
[`E17-design.md`](E17-design.md) §4; 26 entries added and 3 rewritten in `docs/exe/map.toml`, 39
fields in `structs.toml`.

- **Path.** From T1's stack drawer 0x43E140 to its callers: the redraw 0x494440 (H3API's
  `H3CombatManager::Refresh`), which walks the hex records; from the hex records' screen fields
  (H3API's `H3CombatSquare`) to the routine that fills them (0x4642D0, found by its `add 0x2c`);
  from the redraw's other draws to the obstacle, hero and picture helpers; from the outline's
  palette pointer to the startup that loads `game.pal`, and from its rotation to the idle step;
  from the count box's +0x14C to `cranim.txt`'s loader. The draw order read pass by pass; the siege
  branches named only.
- **Checked against the files** with throwaway scripts (not committed): the obstacle type table
  (91 records) and the absolute one (34) dumped from the executable; obstacle DEFs hold runs of
  index 0 only; `game.pal`'s ramps; the pictures' sizes (`CmBk*.pcx` 800 × 556, `CmNumWin.pcx`
  30 × 11, `CCellGrd.pcx` 45 × 52, `CH*.def` 150 × 175).
- **VCMI 1.7.5** compared (`client/battle/`, `config/obstacles.json`): the hex boxes, the stack's
  canvas for both facings and the two-hex shift, the count box's four positions, the 36 foreground
  obstacles and all 91 sizes, the heroes' points agree; it differs in the obstacle `ObRgS03.def`'s
  y (42 px), in the draw order inside a row (obstacles and count boxes after the stacks) and in the
  count box's troop-count offset and war machines. Recorded for the M3 reference frames (§4.7).
- **For T7:** a two-hex stack is anchored at the engine's tail hex (the exe's position is the rear
  hex, the engine's the head); the background, the absolute obstacle and the layer-1 obstacles come
  first, then the heroes, then per row the layer-0 obstacles before the row's stacks.
- **Not read** (§4.7): the grid and shading options, the count box's tint and text, the stack's
  byte +0x4F1, the palette effects, the siege branches.
- Gates: `scripts/exe.py check` and `cargo xtask check-exe-map` (every cited address covered).

### M1 T4 — The asset tables (2026-10-13)

Same session as T5, on `client/e17-m1-t4-asset-tables` (stacked on T5's branch). Sources in
[`E17-design.md`](E17-design.md) §5; the format in `clients/README.md`.

- **The tables.** `clients/assets/base.toml` (the `[battle]` groups, 121 creatures, the 8
  terrains with pictures, 12 special battlefields, 125 obstacle types), `ab.toml` (23 creatures),
  `sod.toml` (the header: SoD adds no key with graphics). 30 shooters' projectiles; `cranim.txt`'s
  numbers per creature.
- **`h3_view::tables`** (pure, `no_std`): `parse` (toml 1.1 without std, serde, unknown fields
  rejected) and `load` (a selection's tables, dependencies first; unknown and circular
  dependencies, `[battle]` not exactly once, keys twice, malformed names and obstacle shapes are
  errors); `Tables::names` for the gate.
- **`h3-view harvest`** (an adapter): reads the executable through `h3_formats::exe` (the
  creature table, the projectile switch's code bytes, the obstacle and background tables) and
  `cranim.txt`, `crtraits.txt` through `h3_formats::text`; keys from the packs; prints the gaps.
- **Checks.** `tests/tables.rs` (in CI): each selection loads; every pack key has an entry but
  `base:water` and `base:rock`; extras are the special battlefields and obstacle types. The gate
  (`--tables`): all 307 names found in the first archive of their search list and decoded.
- *Provisional* (review list): the format, the dependencies, the keying by names.
- Gates: `clients/scripts/check.sh` passes; the real-asset gate passes on the user's files; CI
  could not run (billing).

### M1 T7 — A battle still (2026-10-13)

On `client/e17-m1-t7-battle-still`, after #147 (the obstacles' types on the wire, protocol v22)
and with `shared/clients-ci-paths` (#152: the client job watches the engine crates the workspace
now builds), the user present. The geometry is design §4's; nothing new was read in the
executable.

- **The record** (`h3_view::record`, pure): the header and the numbered messages through
  `h3-protocol`'s codec (a line at most 16 MiB, *provisional*); the record format and the
  protocol must be the client's own; `first_snapshot`. `h3-protocol` joins the library's
  allowlist (ADR-0020 points 1 and 8).
- **The layout as data** (`h3_view::layout`, `clients/layouts/battle.toml`; ADR-0020 point 3):
  the frame, the hex boxes (origin, step, the even rows' shift), the anchor, the stack canvas,
  the columns of the layer-1 pass, the outline's palette and entry. A file per screen and
  resolution, apart from the per-expansion asset tables, so an HD layout is another file.
- **The scene** (`h3_view::scene`, pure): snapshot + manifest + tables + layout → the draw list
  at time 0 in the executable's order (§4.6): the background (variant 0; 0x41ADB0 not read),
  the absolute obstacle (index 0 transparent), the layer-1 obstacles by anchor in the layout's
  columns, then per row passes 0–7 over the columns: the layer-0 obstacles in pass 2, the living
  stacks in pass 4 (their standing draw layer). A stack: the tables' standing group, frame 0,
  at its position hex's anchor, mirrored for the defender, the active stack outlined with
  `game.pal`'s entry 0x60. A two-hex stack is drawn from its rear hex, the engine's position since
  PR #150 (protocol v23; merged during T7: the branch first converted the engine's head in one
  function, then dropped it, and the three stills came out byte-identical). A natural obstacle
  without a type is not
  drawn, nor one whose every hex a removal cleared (*provisional*, M2). Handles the manifest
  lacks, keys the tables lack and an obstacle type of the wrong kind are errors.
- **The batch and the compositor** (`h3_view::batch`, `h3_view::compose`, pure): a checkpoint's
  assets decoded once (a DEF with every frame, PCX, PAL) and the names a list needs; each draw
  with the executable's drawer into the 16-bit 5-6-5 frame: the background opaque, the absolute
  obstacle keyed (`h3_render::blit::draw_image_keyed`, 0x44FC70), an obstacle with the plain
  drawer and runs skipped, a creature with the shadowed drawer, its canvas at the anchor minus
  (196, 267), mirrored at the anchor's x minus the DEF's width plus 196.
- **`h3-view render`** (the adapter): a scenario file runs in-process in `h3-protocol`'s session
  (its packs compiled as `h3 run` compiles them, obstacles generated from the seed when it asks);
  a spectator joins and fetches its record, written beside the PNG; or a record file is read.
  The four archives are read whole (CLT-Q2), the batch decoded, the frame composed, the PNG
  written; `--list` prints the draw list. Its scenario adapter uses `h3-pack-battle`,
  `h3-battle` and `h3-kernel`; Cargo has no binary-only dependencies, so `tests/purity.rs` now
  checks that the library never names them.
- **The stills** (`clients/scenarios/`, seed 1, the user's files): `one-hex.toml` (grass, four
  one-hex stacks a side, generated obstacles), `two-hex.toml` (rough, three two-hex stacks a
  side), `obstacles.toml` (dirt, the absolute obstacle 100, obstacle 2 of layer 1 and obstacle 12
  of layer 0 placed by hand where the generator could place them, mixed stacks). 9 to 13 draws,
  batches of 10 to 13 assets; the archives read in about 0.2 s, a batch decoded in 18–32 ms.
  Sent to the user for the comparison with the original (T8).
- *Provisional* (review list): the layout's format and file; `h3-pack-battle`, `h3-battle` and
  `h3-kernel` as the binary's dependencies; the record's line bound; the removal rule.
- Gates: `clients/scripts/check.sh` passes (coverage 100 %); CI could not run (billing).

### M1 T8 — M1's closure (2026-10-13)

On `client/e17-m1-t8-closure`, the user present; with `shared/exe-map-clients` (#154:
`check-exe-map` scans `clients/` too, the user agreed).

- **Exit checks on master:** `clients/scripts/check.sh` passed (100 %); the gate decoded all
  11 981 entries and found the tables' 307 names; the three stills rendered.
- **The fresh reviewer** (a sub-agent, read-only): 13 findings, each checked against the code.
  Fixed: the obstacle removal rule was wrong (the engine drops a removed obstacle from the list, so
  the rule could only hide one still on the field: dropped); a DEF frame could make the decoder
  reserve 512 MiB (now at most 2^20 pixels; every real frame fits); a stack's placement saturates;
  three tests that wrong code could pass (mirroring with the native offsets, layer-1 obstacles
  before a stack above their row, exact pixels clipped at each edge); `h3-formats` binary-only in
  `tests/purity.rs`; `harvest` writes to `clients/out/harvest` by default; usage text and
  descriptions. Recorded: an HD layout's missing field origin, the record bound's wording, the fix
  entry's missing flag and profile, water and rock battles without a background (inbox).
- **The review list** accepted by the user, all 13 items.
- **Reference captures** (the user: automatic, for exhaustive diffs; in the cloud sandbox under
  Wine; design §6, `clients/README.md`): five DLLs copied read-only from the installation into
  `original-data/install`; `clients/capture/` (`setup.sh`, `battle.sh`, `h3cap.py`, the `xdd.dll`
  forwarder). The main menu proved the capture exact (5-6-5). The battle lane's smoke map gives a
  battle in one key press; `scenarios/smoke.toml` rebuilds it, its obstacles found by
  `h3cap.py obstacles`. **Background, obstacles and the standing stacks match pixel for pixel**; a
  run from a fresh prefix reproduced the identical frame. Differences (M2/M3): heroes, war
  machines, count boxes, the panel, the AI's moving stack; the outline is unchecked (the AI's
  faster stack opens).
- **M2 prepared:** the capture harness first; the window/audio evaluation's candidates and method
  (QUEUE); CLT-Q1 still unowned.
- Gates: `check.sh` passes (100 %); CI could not run (billing).

### M2 T1 — The capture harness (2026-10-13)

On `client/e17-m2-t1-capture-harness`, the user present; with `shared/clients-ci-h3-map` (#161:
the client job's paths name `h3-map`, `h3-adventure` and `h3-game`; the user agreed to T1's plan).

- **The user's two asks** (mid-session): get through the menus into a situation with as few
  screenshots as possible (the menus never move), and take frames at exactly the right time,
  perhaps through a "time freeze" in the executable. Both done without touching the executable:
- **The virtual clock** (`capture/xdd.c`; design §6.1). The GOG `Heroes3.exe` reads time only
  through WINMM's `timeGetTime` (its import table; 6 call sites); our `xdd.dll`, which it loads at
  start, points that slot at a clock the harness freezes and steps by milliseconds through a
  command file, and acknowledges how often the game read it since (idle = it drew everything up
  to that time). Measured: frozen, the game reads it about 170 000 times a second.
- **The driver** (`capture/h3drive.py`): the recipes `to_scenario` and `into_battle` by fixed
  coordinates; screens by 12 probe pixels each (`screens.toml`, learned with `learn`; the battle's
  from both turns' panels); no fixed waits. Launch to a battle's opening in about 8 s (T8's script
  waited 15 s for the menu alone). Two independent runs of the smoke battle gave pixel-identical
  frames at 0, 100, 200, 500 and 1000 ms.
- **Situations:** `h3-view situation` (the scenario adapter's ruleset and setup, now shared by
  `render`: `scenario::battle`, `scenario::sides`; the creatures' original ids from
  `crtraits.txt` by name, as `harvest` keys them). Heroes without skills (no Leadership morale,
  no Tactics), artifacts or war machines; Orrin and Crag Hack (no spell book); `--tile` moves the
  battle. `h3cap.py find` renders each obstacle type of the terrain alone and matches it at every
  anchor (1.0 exact; above 0.9 partly covered; a look-alike scores about 0.5); `h3cap.py diff`
  with masks (heroes' corners, count boxes from the record, panel). `battle.sh` and `h3cap.py
  first`/`obstacles` gave way to `h3drive.py battle`/`set` and `h3cap.py find`.
- **The set** (`clients/situations/set.toml`): grass (frames 0 to 800 ms), dirt, sand, snow,
  swamp, rough, subterranean, lava, and dirt at tile (20, 20) with an absolute obstacle; 17
  frames, about 70 s unattended. Everything outside the masks matches but the outline's colour
  (412 pixels, its pulse) and the grass Pikemen's idle animation; recorded per frame. After a
  fresh `setup.sh` all 17 frames came back pixel-identical (digests of the pixels: the XWD colour
  map holds unset bytes).
- **Fixed on the way:** dirt's background. 0x41ADB0 read (design §5): the variant comes from the
  trees, mountains and dead vegetation around the battle; an empty surrounding gives the third
  picture, which the scene now takes (it took the first, `CmBkDrDd.pcx` instead of
  `CmBkDrTr.pcx` on dirt; grass and snow have the same picture first and third). CLT-Q4 asks how a
  record should carry it.
- **Findings for T3/T4** (design §6.2): the outline steps through the ramp every 100 ms, period
  800 ms, the opening already one step on; the Pikemen fidget from about 200 to 700 ms; the
  obstacles follow the battle's tile, not the armies or the clock.
- **Decided (*provisional*):** the diff stays in Python (design §6.1).
- Gates: see the pull request.

### M2 T3 — Code reading: the battle's animation timing (2026-10-13)

On `client/e17-m2-t3-animation-timing`, the user present (T3 chosen over T2 at the session's
start). The user decided **CLT-Q1: (a)**, the engine's `Moved` carries the path (for the battle
and server lanes; `OPEN-QUESTIONS.md`).

- **Read** (design §7; `docs/exe/map.toml`, `structs.toml`): the battle's tick 0x474040 and the
  frame deadline 0x698998, the idle step 0x4960D0 (read whole now) and its time 0x6989B8, the
  combat speed 0x6987EC and its factors 0x63CF7C, `stack_play_group` 0x446940, `walk_step`
  0x43F0B0 (was `stack_layer_by_direction`), the turn's groups (0x446720), the flight 0x4B4A40,
  `strike_animation` 0x468990 (was `cast_pending_spells`; H3API's ApplyAnimationToLastHitArmy),
  the melee strike's group and pauses, the shot's frames and projectile (0x43F2C0), the fidget
  clocks (0x479F30 and its copies after every action), the waits 0x4F82F0 and 0x4F83C0, the
  cosmetic draw 0x50B1D0. 21 map entries, 16 record fields.
- **The original's state:** under Wine the game's memory is readable at its addresses;
  `capture/h3mem.py` reads the clocks and every stack's group, frame and hex, so the harness
  confirms a timing to the millisecond without a screenshot. `h3drive.py sequence` (actions at
  the frozen clock, then 1 ms steps) and `sequences` (replays the recorded timelines,
  `situations/sequences.toml`: walk, shot, melee; fails on a change). A rerun reproduced all
  three exactly.
- **Confirmed:** every formula of design §7 at speed 0 (a walk, a flight, two shots, a melee
  strike and its retaliation, Defend and Wait, the idle steps and fidgets of the grass
  situation), and the walk at speeds 1 (62 and 35 ms: trunc(100 x 0.63f) is 62) and 2 (40, 22).
- **Findings beyond the queue's list:** the first frame of an animation waits for the deadline
  grid; waits outside the deadline make the next frames show at once (the hit after a shot loses
  its frame 0); the idle step runs twice after a long animation; fidgets are scheduled by a clock
  per stack, redrawn after every action, stall 8 times in 101, and draw from the rules'
  generator (the original's later rolls depend on how long a player waits); melee ignores the
  attack time; Defend and Wait do not animate. VCMI 1.7.5 approximates most of it (§7.5).
- **Decided (*provisional*):** reading the game's memory in the harness (read only); the
  sequences' format (numbers only); the timeline as the T4 check of timing, the original's poses
  read from memory as the T4 check of frames (§7.6).
- Gates: see the pull request.

### CLT-Q1 — The walk's path, built (2026-10-13)

On `battle/clt-q1-walk-path` (#169, `cross-lane`), the user present: the user chose CLT-Q1 over
T4 for the session's start, then the shape (one `Moved` per hex, `EventKind` unchanged) and the
path public (a detour around a pit only the walker's side sees shows it, as in the original).
`h3-battle`: `obstacles::go` and `travel_through` report every position of a walker's path,
`Sink::RECORDS` skips the path for `NoopSink` (the simulation unchanged; `bench-check` passed
without a re-baseline); a placement stays one move. The property `walks_are_reported_hex_by_hex`.
The wire is v29 (the server's lists took 27 and E04c T2b 28 meanwhile: merged). Docs: BAT-MOV-003, BAT-MAG-031,
the battle lane's inbox, the server lane's version notes.

### M2 T4a — The scene over time: the timeline (2026-10-13)

On `client/e17-m2-t4a-timeline` (from CLT-Q1's branch), the user present.

- **Built:** `h3_view::anim` (pure): `Speed` (the factors exact in integers: the f32 mantissas
  over powers of two), the clock (the deadline on its grid, waits outside it), walks hex by hex,
  flights in k cycles, melee strikes with their reactions (hit, defend, death) and the pause
  before a retaliation or a second strike, shots (the group by the angle, frames to the climax,
  the projectile's steps, the loop), `Timeline::at` (the poses shown at a time), `active`,
  `lines` (the harness's format), `differences`. `scene::battle_at` draws a `Moment` (poses and
  the active stack). The bin: `sequences` (each sequence's scenario played in-process with its
  engine commands, `scenario::played`), `render --at <ms> --speed`, `motion` (a creature's
  numbers from the tables and its DEF). The asset tables' `groups` name every battle group.
- **Read:** the shot's target and start points (0x43F2C0 to 0x43F409, design §7.8).
- **Checked:** `h3-view sequences`: walk, shot and melee as recorded, every line; the unit tests
  replay them from their numbers (CI without original files). Speeds 1 and 2 give §7.3's walk
  frames.
- **Decided (*provisional*, the review list):** design §7.8's list.
- **Left for T4b:** design §7.8's "Not yet", and a full battle's record frame by frame.
- Gates: see the pull request.

### M2 T6 — Experiments on the adventure map (2026-10-13)

On `client/e17-m2-t6-adventure-harness`, the user present. The session's choice, confirmed by the
user over T4b and T2: the queue inbox's high-priority request (2026-10-13, written by the
adventure lane, E08 T16c): agents play the domain lanes' experiment batches on the harness; until
then the user played them. Not in the queue's tasks before (the inbox only).

- **Built:** `capture/h3adv.py`: `to_hot_seat` (every name a human), `dismiss` and `end_turn`
  (each dialog's OK found by relative probes, `learn-button`), `select` (the hero list),
  `centre` (the minimap), `hover` (waits for the game's tile under the mouse), `walk` (two
  clicks, the path's marks read between them, the rest detected), `batch` (a recipe's steps into
  the results template, the spec's stamp checked), `compare`. `h3mem.py`: the adventure map's
  players, heroes, view, mouse tile and path; `h3mem.py adventure`. The recipe
  `capture/batches/E08-batch-2.toml`. Probes `multiplayer`, `hotseat_names`, `turn_dialog`,
  `adventure` (both colours), `[button.ok]`. `setup.sh` sets `Show Route`.
- **Observed** (design §6.3; `structs.toml`, `map.toml`, `inferred`): the hero, player and
  adventure manager fields; the path's frames (1–25 green, 26–50 red); the view's geometry and
  the minimap's.
- **Checked:** E08 batch 2 on the harness: D3a–D3d as the user's results, all 13 fields, in three
  runs; D5's 32 piles within 5–10 and 3–6 (each run with a wood pile of 5 and a mercury pile of 6, as the user's); the
  starting amounts differ by difficulty (the user: Easy).
- **Decided (*provisional*, the review list):** the recipe format, `Show Route`, memory over the
  screen, the rest after 1.5 s.
- **Cross-lane** (the user's consent): `docs/experiments/README.md` names the command; the
  adventure lane's inbox holds the note for its increment 3.
- **Left:** the hero list's scrolling, the underground, Yes/No dialogs, a battle from the map, a
  second day (design §6.3's "Not built yet"); a battle batch's cross-check (E04a batch 1a).
- Gates: see the pull request.

### M2 T4b-1 — The frame check (2026-10-13)

On `client/e17-m2-t4b1-frame-check`, the user present. T4b split in two (the session's
proposal, the user's choice over T4b's scene-first half and T2): T4b-1 the frame check, T4b-2 the
rest of the scene (`QUEUE.md`).

- **Built:** `h3drive.py frames` (every distinct battlefield of a recorded sequence kept with its
  state, the viewer's frame at the same times, the masked diff, the counts recorded in
  `situations/timelines/<id>-frames.txt`); `h3-view sequences --frames`; `h3mem.py` reads the
  active stack, the mouse's highlight and the draw layer. In `h3_view`: the projectile's flight
  (`anim::Flight`, `Timeline::projectile`, `Draw::Projectile`, the plain drawer mirrored), the
  idle steps (`anim::Idle`, `Timeline::pulse`) and the outlined stack (`Timeline::outlined`),
  the outline's colour by the idle steps (`BattleLayout::outline_after`, the layout's
  `outline_ramp`), the flight's points in f32 emulated in fixed point (`f32_round`), the
  projectile's frame by angle (tangents by series, no platform transcendental).
- **Read:** `shot_animation` from 0x43F5D6 (the projectile's steps, canvas, drawer, mirroring),
  `shot_angle_group`'s frame (0x469957), `stack_flight_animation`'s points (0x4B4BA3 to
  0x4B4D5D), `strike_animation`'s set-up for a shooter (0x468A3D).
- **Checked** (design §7.9): walk 42 frames and shot 50 exact; melee 68 of 75; the 7 others (the
  Wolf Riders walking past the Angels) listed for T4b-2; three full runs (two from scratch, one
  after merging master with the battle lane's map writer change) gave the same counts. Under a
  build's load `sequences` once read the melee's computer frame a millisecond late: its steps now
  wait for 500 clock reads (runbook "Traps"). T4a's walk offsets confirmed; its
  shooter frames and flight points corrected; the outline and idle steps modelled from the
  frames.
- **Decided (*provisional*, the review list):** a command without a time after the first counts
  as the computer's (its idle step and outline); the check draws the viewer's timeline, the
  original's memory only for fidgets; the counts per frame as the recorded format.
- **Left (T4b-2):** fidgets from the viewer's own generator, turns, the draw layers (walk 3/7,
  striker 6, struck 5), deaths by spells, created stacks, Haste and Slow, a full battle frame by
  frame headless; the walk-by difference.
- Gates: see the pull request.

### M2 T4b-2a — A full battle, frame by frame (2026-10-14)

On `client/e17-m2-t4b2a-full-battle`, the user present at the start. T4b-2 split in two (the
session's proposal, the user's choice over the references first and T2): T4b-2a a whole battle
headless with what it needs, T4b-2b the rest (`QUEUE.md`).

- **Built:** `h3-view play <scenario | record>`: a bot of the viewer's own (`bin/h3-view/bot.rs`)
  fights a scenario to its end in the engine, the protocol's session replays its commands for a
  spectator's record, and every distinct frame is drawn (`Timeline::changes`, `Moment::at`,
  `anim::commands`; `--png`, `--from`, `--to`, past the end the battle idles);
  `clients/scenarios/battle.toml`. In `h3_view::anim`: facing and the executable's position of a
  turned two-hex stack (`Live::position`, `covers`), animated turns and flips, turning back, the
  front side's test (`faces`, `Stage::neighbor` from the layout's anchors), the shooter's turn,
  `Pose::mirrored` and `Pose::layer` (the scene draws each pose in its layer's pass), the pending
  layer changes, fidgets (`anim::fidgeting`, `Fidgets`), a flight's second standing frame. The
  asset tables' `turn_left` and `turn_right` groups.
- **Read** (design §7.10, the address map): `stack_turn_around` 0x446720's groups and position
  shift; `walk_step`'s turn per step (animated on the first only) and layers; `stack_walk`'s turn
  back (0x4423A2) and who sets its flag (the move action 0x479106, not the attack 0x478971);
  `move_and_attack`'s saved facing and turn back (0x478A57); `strike_action`'s choice of side and
  turn (0x445BCE to 0x445C9B) through `stack_neighbor` 0x523DF0 and `hex_neighbor` 0x523D90;
  `melee_attack`'s target turn and turn back (0x441D79, 0x441F36) through `stack_direction_to`
  0x4458B0; the flight's turn by columns (0x4B4A53) and its two standing frames (0x4B4E66,
  0x4B4A15); `stack_fly`'s turn back; `stack_animation_backdrop` 0x446830; `stack_play_group`'s
  dirty rectangle (a lead for the walk-by difference). The struck stack's layer 5: no write found
  (the harness's reading stands).
- **Changed:** T4a's provisional "a later command starts one tick after a command that drew": the
  flight's second standing frame explains the melee's timing, so a later command without a time
  starts as the last frame shows; the three sequences still give their recorded timelines.
- **Checked:** the three recorded timelines (`h3-view sequences`) unchanged, the fourth (`turn`)
  as recorded; `play
  scenarios/battle.toml`: 24 commands, about 82 s of battle, about 1 100 distinct frames drawn in
  about 3 s; the unit tests cover each turn case (walk, flight, melee from either side, shot) and
  the fidgets; the library at 100 % coverage. Turns looked at in the bots' battle (the Wolf
  Riders' turn before their strike).
- **The turn sequence** (recorded the next morning, when the user's computer was back): the
  first plan (Angels flying behind the Wolf Riders on column 15 to strike them, both turning) does
  not exist in the original: the Angels' speed does not reach hex 83, Archangels are two hexes
  wide (the harness's clicks landed them one hex left: their position is the rear hex), and a
  two-hex target on column 15 is struck from above its middle (hex 82), where no one turns; a
  strike or shot turn cannot be set up from an opening. Recorded instead: the walk sequence's
  Champions walking one hex up and to the left (sequence `turn`): the viewer's timeline as
  recorded (33 changes) and all 39 frames exact (`h3drive.py frames --only turn`), a unit test
  replays it.
- **Decided (*provisional*, the review list):** the bot and `play`'s formats; the layers' model;
  the fidget generator; the next command's start.
- Gates: see the pull request.

### M2 T2 — The window, input and audio evaluation (2026-10-14)

The user present; [design §8](E17-design.md), [ADR-0024](../adr/0024-client-window-audio.md) (its
text accepted by the user the same day, #196).

- **Chosen by the user:** SDL 3 through `sdl3-sys`, SDL built from source and linked statically;
  defaults (*provisional*, the review list) vsync off and the hardware cursor. No licence step:
  everything is Zlib, MIT or MIT-or-Apache.
- **Method:** three spike adapters on the throwaway branch `client/e17-m2-t2-spikes`
  (`clients/spikes/`: `common`, `prep`, `sdl3`, `minifb`, `miniquad`, `mp3bench`), the same reel
  of 302 frames (20 s of `play scenarios/battle.toml`), the game's cursor, `PIKEATTK.wav` and
  `COMBAT01.MP3`, each with a report (pacing, audio, read-back). Built for Linux and, by
  cross-compiling (`x86_64-pc-windows-gnu`, `mingw-w64`; the user opened `static.rust-lang.org`
  in the session's egress for the target's standard library), for Windows; checked under Xvfb
  and Wine in the sandbox, then measured on the user's machine (`measure.cmd`, six runs) and
  looked at and listened to by the user.
- **Result:** every spike's frame exact at integer scales (screen captures and read-backs; on the
  user's GPU through Direct3D 11 and OpenGL); SDL 3 within 1.5 ms of each frame's time without
  vsync, clean WASAPI audio at 10 ms buffers, a key to the effect in about 4 ms; both `tinyaudio`
  spikes crackled (DirectSound callbacks late against a 23 ms buffer).
- **Found on the way:** the game's sounds are IMA ADPCM (SDL and a 60-line decoder agree); SDL's
  device streams (`SDL_OpenAudioDeviceStream` takes no further bindings; a one-shot clip needs a
  flush); `nanomp3` gives minimp3's samples in pure Rust, `puremp3` is wrong on `COMBAT01`;
  `minifb`'s `ScaleMode::UpperLeft` does not scale; `tinyaudio`'s macOS backend brings an
  Apache-2.0-only build dependency.
- Gates: the docs only (`check.sh` unchanged, green on `master` at the start); see the pull request.

### M2 T5a — A battle in a window (2026-10-14)

On `client/e17-m2-t5a-window` (#201), the user present; [design §8.1](E17-design.md),
[ADR-0024](../adr/0024-client-window-audio.md); with `shared/clients-ci-sdl` (#200: the CI job's SDL
headers, the user's consent).

- **Split:** T5 (the adapter, the sound, M2's closure) was more than a session; the user chose the
  window first (T5a). T5b (the sound) and T5c (M2's closure) are in the queue.
- **Built:** `clients/crates/h3-sdl`, the platform adapter: every SDL call (`sdl3-sys` 0.7.2, SDL
  3.4.18 from source, static; camera, dialog, GPU, haptic, HIDAPI, joystick, power, sensor and
  tray left out): the window at the largest integer scale that fits, nearest sampling, centred,
  high pixel density; its events; vsync and fullscreen; the hardware colour cursor at the
  frame's scale; a read-back of the whole output. `h3-view window <scenario | record>`: the
  battle loaded as `play` loads it (`bin/h3-view/battle.rs`, now shared), each frame composed
  ahead and shown when it falls due, past the end idling; Space, →, R, F, V, C, Esc;
  `--check <s>` (every frame read back and compared, the black around included) and
  `--measure <s>`. Pure: `h3_view::clock` (the battle's clock: pause, seek, a frame's delay),
  `h3_view::cursor` (`CRDEFLT.DEF`'s first frame drawn or as an RGBA image) and
  `Screen::xrgb` (the screen widened as the PNG is). `assets::clients_dir`: an executable copied
  into a checkout finds its tables, layouts and packs.
- **Checked in the sandbox** (Xvfb, Mesa's software OpenGL ES 2): `--check 20`: 301 frames, every
  one exact at scale 1; `--measure 10`: median 7.0 ms after the frame's time, p95 13.0 (the
  software renderer); driven by `xdotool`: pause, step, restart, fullscreen, the drawn cursor and
  Esc as meant. `play scenarios/battle.toml` after the refactor: 24 commands, 81 764 ms, 1 078
  frames, as before.
- **On the user's machine** (the cross-built exe, `measure.cmd`): Direct3D 11 at scale 2, 302 of
  302 frames read back exact; a frame shows a median 0.4 ms after its time (p95 1.0 without
  vsync, 1.3 with it), one 27 ms outlier per run. The user, after watching it: "looks good",
  but for the flight below.
- **Found by the user:** in the window the Angels, flying down four rows, passed under the Wolf
  Riders' corpse and the Orcs. The viewer drew a flier in its start row's pass; the executable
  takes it off its hexes for the flight (0x468730) and draws it by itself over everything
  (0x4B4D60). Fixed: the cycles' poses take layer 8 (`anim::FLYING`), drawn after every row
  (design §7.10); the four recorded sequences unchanged.
- **Found on the way:** `clients/Cargo.lock` was stale on `master` again (`h3-game` gained
  `h3-battle` and `h3-content-battle` through the adventure lane's E08 T19a1/T19a2), so
  `check.sh` failed at `--locked` before any change; fixed here, the lasting fix in the inbox for
  the user. SDL's Linux build stops without Xcursor's headers (the sandbox and the CI runner
  lacked the X11 extension headers). The battle's frame is 800 × 600: at 1080 pixels of height
  the window's scale is 1.
- Gates: `clients/scripts/check.sh` all passed (coverage 100 % with `h3-sdl` excluded as an
  adapter); the scoped verify with no failing step; see the pull request.

### M2 T5b — The sound (2026-10-14)

On `client/e17-m2-t5b-sound`, the user present; [design §8.2](E17-design.md),
[ADR-0024](../adr/0024-client-window-audio.md) point 5; with `shared/clients-lock-check` (#202:
the verify's `clients-lock` gate and `clients/Cargo.lock` a common path, the user's choice of a
failing check).

- **Decoder:** `h3_assets::snd` (the sound archives' directory, entries by stem) and
  `h3_assets::wav` (IMA ADPCM WAVs to 16-bit PCM; every complete frame up to the `fact` chunk's
  count; test encoders in `build`). The real-asset gate decodes all 1 232 entries of
  `Heroes3.snd` and `H3ab_ahd.snd` and compares them with SDL's own loader (`h3_sdl::load_wav`):
  all equal with the hints `dropframe` and `truncate`, and SDL's defaults' samples as far as both
  go. Found: SDL's defaults drop a cut last block, which 1 212 entries have (28 s of sound in all),
  and ignore the `fact` chunk; the AB archive's entries are all in `Heroes3.snd`, byte for byte.
- **Code reading** (design §8.2): `play_sample` and its channel table (the creature sounds' eight
  channels, a free one or the next in turn), the stacks' eight sounds and `stack_load_graphics`'
  rules for them, their sites in walks, flights, shots and strikes, the opening (the start sound,
  the clock's wait, the music), the music's thread (looping, the fade-in, the resume position),
  the volumes. Map entries for each; `battle_start`'s and `stack_load_graphics`' summaries
  extended.
- **Built:** the timeline's cues (`anim::Sound`, `anim::Cue`) at the frames the readings name;
  `h3_view::sound` (the opening's draws, the volumes, the fade-in); the asset tables' `sounds`
  (written by `h3-view harvest`, 743 names, all found by the gate); `h3-sdl`'s audio (device,
  streams, gain, pause; the window and the audio each start and stop their own SDL subsystem);
  `h3-view window` plays the start sound while the clock waits, then the music (decoded by
  `nanomp3`, looping, fading in) and the creatures' sounds on eight channels (keys M and E,
  `--mute`); `play` writes `cues.txt`.
- **Found by the user** (listening to the Windows build): a Cyclops' target winced about 0.7 s
  after the rock landed. The viewer counted the shooter's frames left after its climax in
  `strike_animation`'s loop; the executable marks no striker for a shot (only `melee_strike`
  does, 0x441A87), so the loop counts the reactions alone and the target reacts as the
  projectile lands. Fixed (design §7.4); the Marksmen's recorded shot, one frame after its
  climax, is the same either way and stays as recorded. Also asked: the Marksmen act twice in a
  row because the engine rolled good morale (`GoodMorale` in the record), which the viewer does
  not show yet (inbox).
- **Checked in the sandbox** (Xvfb, SDL's `disk` audio driver): `--check 25` with sound, 293 of
  293 frames exact; the mix: the start sound (`BATTLE01`, 5.8 s), then the music fading in; with
  the music off, the effects' onsets within 10–100 ms of their cues.

### M2 T5c — M2's closure (2026-10-14)

On `client/e17-m2-t5c-closure`, the user present.

- **Starting point:** master at #204; the full local verify green with the original data (CI
  still unstarted: billing). The user's answer to T5b's listening questions: no crackling, the
  sounds in sync with the animations, the balance right.
- **The fresh reviewer** (a sub-agent, read-only) over M2's code (the timeline, `h3-sdl`, the
  window, the sound, the sound decoder): 12 findings, each checked against the code, and where
  it named the original against the executable (`scripts/exe.py`). Fixed:
  1. a `TurnStarted` was timed before the melee's and the shot's turn-backs, so the next turn's
     outline showed while both stacks still turned back: the attack now ends first;
  2. → while paused kept the creature sounds playing (a move's loop never stopped) and skipped
     the next frame's cues: it silences them and keeps those cues;
  3. a command's last layer changes waited for the next command's first frame: a corpse lay in
     the struck stacks' pass through a player's wait, a survivor's standing pose came back over
     its fidget with another stack's walk (and stopped it): they show at the first idle step;
  4. a strike or shot by a stack the timeline does not know dropped its targets' kills (the dead
     stood on): the targets react alone;
  5. a breath on two stacks took groups 11–13: read, `melee_strike` takes 17–19 where the DEF has
     them, for a breath only (creature flag 0x8; 0x441ACA to 0x441B66); the viewer tells a breath
     by its second stack's place (*provisional*), the tables' `breath` groups;
  7. the music's fade-in ran on during a pause; 8. a hardware cursor SDL could not remake left
  the window without one; 9. a redraw was dropped when a skipped frame drew what showed;
  10. a sound that did not decode closed the window; 11. comments that contradicted the code;
  12. the window scanned 10 minutes of idle steps quadratically past a still battle's end
  (`Timeline::still`).
  Not a bug: 6. a one-hex flight (k = 0) has no cycles, as in the executable (0x4B4C5B jumps to
  the landing); pinned by a test. Design §7.3, §7.4, §7.10, §8.1, §8.2; the address map's
  `melee_strike` and `stack_flight_animation`.
- **The review list** (31 items) accepted by the user; the provisional marks stay where a
  recording is to check the rule (M3 T1b) or the original cannot be matched (the fidgets).
- **M3 refined with the user** (QUEUE: T1a–T9): M2's T4b-2b became T1a and T1b; the user added
  the rays (Magi and Beholders draw an energy ray, not a projectile: T1b) and the spell animations
  (T7a the reading, T7b on stacks with morale and luck, T7c area spells, rays and spell
  obstacles); sieges, live play, HD and the client's options later. The text's ADR-0020
  amendment comes with T4, where the catalogue's format is chosen.
- **M2's exit checked on the branch** (the user's files in the sandbox): `clients/scripts/check.sh`
  all gates (100 % coverage); the gate decodes all 11 981 entries, finds the tables' 309 names and
  743 sounds, all 1 232 sounds equal SDL's samples; `sequences`: the four as recorded; `play
  scenarios/battle.toml`: 24 commands, 79 064 ms, 1 051 frames (81 764 ms and 1 078 before #204
  shortened the shots' loops; T5c adds or removes no frame in this battle: no breath, no unknown
  stack); `window --check 25` under Xvfb with SDL's `disk` audio: 293 of 293 frames exact.

### M3 T1a — Turns and the walk-by (2026-10-14)

On `client/e17-m3-t1a-turns`, the user present.

- **Starting point:** master at #207 (M2 closed); the full local verify green with the original
  data (CI still unstarted: billing); the four sequences and their frames as recorded.
- **A battle in progress in the original.** The original's AI acts at once and as it likes, so
  the harness plays both sides instead: `h3-view situation --hot-seat` makes Blue a human too,
  `h3drive.py` starts the map in hot seat (`h3adv.to_hot_seat`) and, among a sequence's actions,
  `turn:<side>.<record>` steps 1 ms at a time, every step read, until that stack's turn starts
  (`at:<ms>` up to a time); the times go to `turns.txt` and into the commands' `at` in
  `sequences.toml`, which `h3-view sequences` gives the timeline (a command without one stays the
  computer's). Found on the way: the game names the striker active again while it turns back
  after its attack, so a turn is the named stack's, not any active one; an action before the turn
  starts is lost; a stack whose player waits fidgets (the viewer's timeline has none), so each
  action comes as its turn starts. Two more harness fixes: a click parks the game's cursor off the
  battlefield (the game draws it: 88 pixels in every frame after the attack's click), and the
  frame check compares the last screen of a millisecond (a wait passes the turn at once, `t<ms>b`)
  and masks a count box by the drawn pose's facing (`m` in `ours.txt`; a turned stack's box sits
  on its other side: 330 pixels at 13 044).
- **The sequences** (design §7.10): `climb` (`seq-turn.toml`, the Champions from 35 to 1: up and
  right, then up and left with a flip only, the turn back animated; the original's path is the
  engine's) and `behind` (`seq-behind.toml`: Centaur Captains and Marksmen against Silver Pegasi
  and Goblins, two factions a side so morale stays 0; eight commands over two rounds: a wait,
  two walks, a defend, the Pegasi's flight behind the Centaurs, a wait, the Centaurs' strike on
  the Pegasi standing, both turning and turning back, the Marksmen's two shots with their turn).
  The viewer's timelines as recorded on the first try (34 and 203 changes; T4b-2a's rules,
  unchanged); the unit tests replay both from their numbers (the two new creatures' frames and
  times from the user's archives). Frames: `climb` 43 of 43 exact, `behind` 164 of 194.
- **The walk-by difference read** (design §7.11): the original redraws by dirty rectangles. A
  stack's frame restores the last frame's rectangle from the saved screen (the battle without the
  animating stack, saved by the backdrop 0x446830), measures the new frame with its count box,
  widens it 17 pixels left and right (the map had said up and down), redraws every item that meets
  the union whole, over what the back buffer holds, and shows only the union; strikes and the idle
  step restore the background inside their rectangle. So shadows are drawn again where an item's
  earlier image was not restored, and the saved screen's count boxes come back under a walker.
  Checked: every pixel of the melee's 7 listed frames lies on the Angels' edge and, at 3 504 and
  3 966, is the viewer's with one more shadow; the rest at 3 966 is the saved screen's count box;
  `behind`'s 30 frames: 97 % of their pixels are the viewer's with one to four more shadows.
  **The user's decision** (asked mid-task, the agent recommending it): the artifacts are not
  copied; the viewer keeps drawing each frame whole (it gains nothing from partial redraws, and a
  stateful renderer replaying the original's redraw order would only reproduce artifacts); the
  frame check is to classify them instead, any other difference still failing: **M3 T1c**, next
  session; an ADR-0020 point 9 amendment drafted in `docs/lanes/client/adr-drafts/`. The address
  map:
  `stack_play_group`, `battle_redraw`, `battle_draw_sprite` corrected and extended;
  `battle_item_rect` 0x495F50, `battle_refresh_creatures` 0x495BF0, `screen_show_rect` 0x602BD0,
  `battlefield_rect` 0x694F18, `empty_rect` 0x6AACE8 added.
- **The queue:** T1a done; T1c added (the classifier); T1b and T7a no longer wait (the battle in
  progress exists); T5 classifies the stale count boxes once it draws the boxes.
- **Checks:** the full local verify green with the original data on the branch's code at
  `7f1dc64` (later commits change docs and the harness only); after merging master `34cb7cd0`:
  `clients/scripts/check.sh` all gates (100 %), `check-lane`, `check-docs`, `check-exe-map`;
  `h3-view sequences` and the harness's `sequences` (an idle machine): the six as recorded;
  `play scenarios/battle.toml` 1 051 frames as before.

### M3 T1c — The frame check classifies redraw artifacts (2026-10-15)

On `client/e17-m3-t1c-artifacts`, the user present.

- **Starting point:** master at #212 (T1a, merged 2026-10-14). T1a's captured frames had lived
  only in its sandbox, reclaimed since: the six sequences were captured again (`h3drive.py
  frames`, about 30 minutes on an idle sandbox); every differing count came back as recorded.
  The runbook now says so.
- **The classifier** (`h3cap.redraws`, design §7.11; *provisional*): a differing pixel is a
  redraw artifact when the original's is the viewer's at the same moment with one to four more
  shadows (body `(d >> 1) & 0x7BEF` or border `((d >> 2) & 0x39E7) + ((d >> 1) & 0x7BEF)`, in
  any order), or one the viewer drew there at an earlier moment of the sequence with at least one
  more. The frame files record `<ms> <differing> <artifacts>` (`sequences.toml`'s header); a change
  in either fails; the diff PNGs mark artifacts yellow.
- **The older moment** (the task's open point; the agent's default, put to the user at the start
  without objection): counted, with at least one more shadow; a pixel equal to an earlier frame's
  with no more shadow, which a timing slip of the viewer's would give, is no artifact. In `behind`
  73 of 3 951 pixels need it (mostly the frame before; 9 at 7 844 from 9 to 91 frames back, the
  saved screen), none is an older frame's unshadowed.
- **Results:** `melee` 6 of 7 frames artifacts only (up to four shadows); at 3 966 ms 96
  artifacts and the stale count box, 231 pixels in its 30 × 11 rectangle at (476, 197), listed
  until T5 draws the boxes (the task's text had all of 3 966 as shadows; T1a's log had it right).
  `behind` all 30 frames artifacts only. Strictness: the viewer's frames shifted one pixel differ
  in millions of pixels, of which 0.2 % fit the rule, so a misplacement still fails.
- **The ADR-0020 amendment draft** says the earlier-moment case and that stale count boxes stay
  listed until T5.
- **Checks:** `frames --reuse` all six as recorded; `clients/scripts/check.sh` (below, PR);
  the session's baseline full verify at the start was green but for this HANDOFF's budget (an
  in-progress line), trimmed.

### M3 T1b-1 — Shots and rays (2026-10-15)

On `client/e17-m3-t1b1-shots`, the user present at the start (T1b re-cut with them: T1b-1 shots
and rays, T1b-2 the provisional rules' checks and the capture set by time, Haste and Slow to T7b
with the spells), then away.

- **Starting point:** master at #219 (`fa542fc`); the clients' baseline full check green (all
  gates, 100 %); the original files staged before the user left.
- **The ray read** (design §7.12, the address map: `draw_ray` 0x5A5C20, `ray_init`, `ray_step`,
  `ray_aim`, `ray_colours` 0x68833C, `screen_masks` 0x68C860, `crt_atan2`, `crt_sin`, `crt_cos`;
  `shot_animation`'s summary): creature flag 0x800 (the Arch Magi, Beholders, Evil Eyes; the
  Magi shoot their projectile); after the climax frames a wait of trunc(115 F), then the ray on
  its own clock, a tick every trunc(10 F), trunc(d / 15) + 15 one-pixel steps per tick along an
  f32 aim renewed each tick, cross-sections of thickness 5 then 4 coloured from the edge inward
  (the colours scaled into the screen's masks, not truncated); the battle redrawn at once without
  it; then the shot group's rest on the deadline, the last frame drawn and never shown.
- **Built:** `h3_view::ray` (the ray in integers: the f32 and 53-bit roundings exact, sine,
  cosine and arc tangent by series), `anim`'s `Beam` and `Timeline::ray`, the scene's and the
  compositor's `Draw::Ray`, `PixelFormat::scale` and `Screen::set` in `h3-render`; the asset
  tables' `ray` (the colours, `h3-view harvest` writes them; the three creatures' entries as the
  harvest gives them).
- **The sequence `shots`** (`seq-shots.toml`, hot seat, six commands over two rounds): three
  rays, the Cyclops' throws down and straight, the Gnolls' defend reaction to a ray, the
  Beholders' death by a throw. The recorded timeline showed one rule wrong: a shooter plays its
  shot group out beyond the reactions (the Cyclops: 10 frames left after its climax, which T5b's
  *provisional* L = R + 1 cut; design §7.4). With that fixed, the viewer's timeline as recorded
  on its first run, and all 139 frames equal the original's outside the masks, every ray tick's
  pixels included.
- **Checks:** `clients/scripts/check.sh` all gates, 100 % (code of `838073f`, master `fa542fc`;
  verify-once's line in the pull request); `h3-view sequences`: the seven as recorded; `h3drive.py
  frames` (all seven captured afresh in this sandbox, about 40 minutes, then redrawn on the final
  code): every count as recorded; `check-lane`, `check-docs`, `check-exe-map`. `h3-view harvest`
  writes the three `ray` lines as committed; its other difference, 35 obstacles beyond the table,
  predates the task (the inbox, for T2).

### M3 T1b-2 — Flights, the breath, the mouse and the set by time (2026-10-15)

On `client/e17-m3-t1b2-checks`, the user present at the start (the increment confirmed: T1b-2 in
full), then away.

- **Starting point:** master at #221 (`103a88f`, T1b-1 merged that morning); the clients'
  baseline full check green (all gates, 100 %); the original files staged before the user left.
- **The sequence `fly`** (`seq-fly.toml`, hot seat, two commands): the Black Dragons fly from
  row 2 to row 3 beside the attacker's column and breathe through the Pikemen into the Gnolls
  behind them (group 19, both stacks hit; the Pikemen strike back), then the Angels fly from
  row 0 to row 9 over the Dragons and five stacks. Where the Dragons stand came from the
  cursor's slot inside the Pikemen's hex (three probes, design §7.13): only the slot up and to
  the right at (+14, −13) puts their head on the breath's line. The viewer's timeline as
  recorded on its first run (110 changes); frames 102, 91 exact, 11 redraw artifacts only. So
  M2's two rules from the code reading hold (a viewer built with the flier in its start row's
  pass, as before T5a, differs at 23 frames, by up to 2 480 pixels: the recording tells them
  apart): a flier's cycles over every row (layer 8, T5a) and
  the breath's groups 17–19 told by the second stack's place (T5c); their *provisional* marks
  lifted (code, design §7.4, §7.10, the review lists). A unit test replays `fly` from its events.
- **The mouse over a stack** (design §7.13, map: `battle_mouse_stack` 0x477C20): read, then
  recorded (`hover`, the `fly` map's opening): the stack the mouse enters starts group 1 at
  frame 0 at once and takes the mouse's outline (0x70, cyan); the idle step plays the group as a
  fidget; entering again while it plays stands the stack. No engine command (the viewer draws no
  mouse in a record): `h3-view sequences` and the frame check skip such an entry. The window's
  mouse-over is queued for T6 (the inbox), with the cursor rules.
- **The capture set by time:** `h3drive.py set` reads each frame's stacks from the original's
  memory and draws ours with `h3-view render --frames` (the outline's pulse on the original's
  clock, a fidget in the original's poses, `frames.toml` as the frame check's). All 17 frames of
  the nine situations exact outside the masks (412 to 2 036 differing pixels each before); the
  set's counts recorded as 0.
- **Checks:** `clients/scripts/check.sh` all gates, 100 % (code `e5142bc`, the branch merged with
  master `615286e`: verify-once's line in the pull request); `h3drive.py sequences`: all nine
  timelines as recorded in a fresh run of the original; `h3drive.py frames` (all eight with
  engine commands captured afresh in this sandbox, `fly` once more) and `h3-view sequences`:
  every count and timeline as recorded; `h3drive.py set`: 17 of 17 exact. Master's merge left
  `h3-view situation`'s maps as they were (all 17 compared), so the captures hold for it.
  `check-lane`, `check-docs`, `check-exe-map` ok.

### M3 T2 — Every battlefield (2026-10-15)

On `client/e17-m3-t2-battlefields`, the user present at the start ("start with T2").

- **Starting point:** master at #226 (`e7fd849`, T1b-2 merged); the sandbox of T1b-2 (its
  captures and builds) kept.
- **The asset tables:** the twelve special battlefields keyed by the packs' keys (E04c T2b: `base`
  shore, magic_plains, cursed_ground, deck, boats; `sod` the seven SoD fields, now in `sod.toml`).
  `harvest` writes them from each pack's `battle.toml`, the exe's field numbers 0–9 by the packs'
  keys (0x4646D0: field 0 the shore, 1–9 from 0x63D2C8; the deck and the boats their own
  pictures). The inbox's "35 obstacles more" was the harvester's: it kept in `base.toml` every
  type `base` does not define, though `sod` defines those 35; it now counts every pack's keys,
  and no table holds a key no pack defines (`tests/tables.rs`).
- **The scene:** the observation's `battlefield` (protocol v28) picks the special battlefield's
  picture over the terrain's. The client's scenario adapter generated random obstacles for the
  terrain only; it now calls `generate_setup_obstacles` as `h3 run` does (the deck's water
  obstacles, the boats' fixed hexes and nothing else). The deck and the boats render from
  scenarios on water.
- **The capture set:** `h3-view situation` sets a scenario's special battlefield up on the
  defender's (Blue's) tile, as the original reads it: water east of Blue for the shore (the
  writer's coast flag; fought on sand), a special ground's first `Objects.txt` picture under both
  heroes for the nine ground fields (classes 21, 46, 222, 224, 226–229, 231). `h3cap.py find`
  takes a field's own obstacle types (any pack's that list it). Ten situations, then an eleventh
  (the shore at (18, 5), found by trying six tiles, with its absolute obstacle 126): all 11
  frames exact outside the masks on the first comparison after the obstacles were found. The
  deck and the two boats need boats on the map: CLT-Q5 (the adventure lane).
- **Also:** the driver tries a failed `xwd` dump again (one set run lost a situation to it).
- **Checks:** `clients/scripts/check.sh` all gates, 100 % (code `a548271`, the branch merged with
  master `746ad69`: verify-once's line in the pull request); the real-asset gate (309 names);
  `h3-view harvest` writes the three tables as committed; `h3-view sequences` and `h3drive.py
  frames --reuse` (T1b-2's captures, redrawn on the final code): every timeline and count as
  recorded; `h3drive.py set` on the final code: 20 situations, all 28 frames as recorded (exact);
  `check-lane`, `check-docs`, `check-exe-map`.

### M3 T3a — The heroes (2026-10-15)

On `client/e17-m3-t3a`, the user present: the agent proposed T3 split (the heroes first, the war
machines as T3b), the user agreed.

- **Starting point:** master at #229 (`6ab2212`, T2 merged); its client code verified already
  (`verify-once`: light gates only). A fresh sandbox: the original files staged, the capture
  setup's apt wait failed once (the setup's own install held the lock) and passed on a retry.
- **Code reading** (design §4.5, §7.14; the map's entries `battle_load_heroes` 0x463370,
  `hero_battle_defs` 0x63BD40, `hero_class_table` 0x67D868, `player_id_is_human_here` 0x4CE970,
  and the battle manager's hero fields): a hero's battle DEF is record town × 2 + female of 18,
  so by faction and sex; Tower and Stronghold name theirs the other way round, Conflux takes CH16
  and CH17 by sex for both classes (VCMI: by class). The flags take their player's 32 colours of
  `PLAYERS.PAL`, wave a frame per idle step from 0 and 3; a hero's idle motion more than 4 500 ms
  after its clock (the battle's start, every action, the motion's end); a dying stack's draws ask
  for defeat (below 60) or the enemy's victory (below 80), played once a round while the active
  side is human.
- **The asset tables:** `[[hero]]` (156: `base` 128, `ab` 28; keyed by the packs' order as the
  hero types, each checked against `hotraits.txt`'s name: two heroes are called Lord Haart),
  `[battle]`'s `hero_groups` and `flags`; the layout's canvases and `PLAYERS.PAL`.
- **The viewer:** `h3_view::hero` (the flags, the idle motion, defeat and victory over the
  timeline's idle steps and actions; the timeline records each action's end, deaths and round
  starts); the scene draws each side's flag and hero after the layer-1 obstacles; the compositor
  the shadowed drawer without an outline, the flag in its player's colours (red and blue:
  CLT-Q6). `h3-view situation` writes the scenario's heroes (both required, with their primary
  skills); every situation and sequence scenario names Orrin and Crag Hack (1 in each skill, as
  the maps had them); the recorded sequences' timelines stay as recorded.
- **The capture set:** the `heroes` mask gone. The 20 situations exact as before; eight grass
  situations with a male and a female hero of each town, so all 18 pictures (exact on the first
  run, Conflux's by sex); the grass situation from 4 400 to 5 100 ms: both heroes' idle motion at
  4 600 to 4 900, exact once the set's clock stepped at most 100 ms (a longer step runs one idle
  step: the outline and the flags were behind). 28 situations, 44 frames, all exact.
- **The frame check:** the heroes' groups and frames from the original's memory
  (`h3mem.battle_heroes`; defeat and victory are drawn from its generator), the flags the
  viewer's own; the `heroes` mask gone. Every sequence captured again (about 50 minutes):
  `shot` and `shots` as recorded; `walk`, `melee`, `turn`, `climb`, `behind` and `fly` gained
  only redraw artifacts in the left hero's shadow under a nearby stack's redrawn rectangles (2 to
  210 pixels a frame), some five shadows deep (black), so `h3cap.MAX_SHADOWS` is five (four since
  T1c); every other count as recorded, `melee`'s stale count box at 3 966 included. The new
  counts recorded. No hero reacted to a death in the eight recordings (the reactions stay read,
  not seen).
- **Checks:** `clients/scripts/check.sh` all gates, 100 % (code `f8b4d84`, the branch merged with
  master `ae6d517`: verify-once's line in the pull request; a first run missed one region, an
  `if let`'s implicit else in a test); the real-asset gate (329 names: 18 hero DEFs and 2 flags
  more); `h3-view harvest` writes the three tables as committed; `h3-view sequences`: every
  timeline as recorded (the scenarios now name their heroes); `h3drive.py set` on the final
  code: 28 situations, 44 frames, all exact; `h3drive.py frames --reuse` on the final code: every
  count as recorded; `check-lane`, `check-docs`, `check-exe-map`.

### M3 T3b — The war machines (2026-10-15)

On `client/e17-m3-t3b`, the user present: the agent proposed T3b (next in the queue; T4 and T7a
as the alternatives), the user agreed.

- **Starting point:** master at #235 (`9dc7261`, T3a merged), no client pull request open; a fresh
  sandbox, the original files staged (one LOD per call), the client code's baseline `check.sh`
  green (`verify-once`: all gates, 100 %).
- **Code reading** (design §7.15; the map's new entries `play_magic_animation` 0x496840,
  `battle_load_magic_animation` 0x5A92F0, `battle_draw_magic_frame` 0x4953B0,
  `force_feedback_effect` 0x4B69F0, `magic_animations` 0x641E08, `heal_sound_name` 0x660A94;
  `draw_battle_stack`'s tail, `first_aid_tent_action` and `draw_frame_transparent` read; the
  stack's +0x20 and +0x16C, the manager's +0x132E8 to +0x132F0): war machines are drawn as any
  stack, without a count box (flag 0x40) and, their DEFs lacking group 1, without fidgets; the
  original's records put them after each side's army in the setup's order (Catapult, Ballista,
  tent, Ammo Cart). The tent's heal: no animation of the tent, `Regener.wav`, the log's line,
  magic animation 79 (`sp12_.def`) over the patient a frame per 100 ms, then the wait for the
  sound. The magic animation's place by the table entry's kind (0 on the hex, 1 centred on the
  creature, 2 above it, 3 beside it), drawn after the stack's sprite; bit 8 the translucent
  drawer (literal pixels half and half, runs skipped). The table's second name is a
  force-feedback effect's (IFC20), not a sound.
- **The capture set:** `h3-view situation` writes each side's war machines as its hero's
  artifacts (slots 13 to 15, the packs' artifacts 4 to 6); `war-machines` (all three on both
  sides, grass, 9 frames to 800 ms): exact on the first run (no count box beside a machine, the
  tent standing in frame 0 although its group 2 has six). The smoke battle (`scenarios/smoke.toml`
  with the smoke map's heroes and Orrin's Ballista) as `smoke`, in hot seat (the set's
  `hot_seat`, new) so that the Wolf Riders open idle: exact at 0 and 400 ms. 30 situations, 55
  frames, all exact.
- **The sequence `machines`** (hot seat): the Pikemen defend, the Archers shoot them; then the
  attacker's Ballista shoots right, the defender's left, the tent heals the Pikemen; round 2 the
  Archers defend. The first comparison differed only in the order of lines at the same
  millisecond: the viewer numbered war machines by the engine's ids (Ballista, Ammo Cart, tent);
  numbered as the original's records, the timeline matched. The frame check then showed the
  flags behind from the first Ballista on: the original runs an idle step at each automatic war
  machine's turn (as at the computer's, §7.9), outlining it until its first frame; built, all 106
  frames exact, the heal's translucent frames and the tent's outline through the heal included.
  The harness reads the magic animation from memory (`h3mem`) and its timeline names it
  (`magic 79 f<n> 0.0`, `magic -`): the heal's 20 frames from 5 845 ms, erased at 7 845, the next
  turn at 7 945, as the viewer's.
- **Built:** the asset tables' `[battle] magic` (83 entries, harvested) and `heal_sound`;
  `h3_view::anim` (the heal, `MagicLook`, `MagicKey`, `Magic`, `Sound::Heal`, the war machines'
  records and turns, the idle steps kept by the player), the scene's `Draw::Magic` after its
  stack's sprite, `h3_render`'s `Drawer::Translucent`; the window plays the heal's sound on the
  start sound's stream (category 2).
- **For others:** the original's automatic tent heals the most hurt stack and a machine with
  nothing to do defends (QUEUE inbox, for the battle lane through the user); the heal's sound
  wait at the faster speeds (inbox, for T8).

### M3 T4a — Fonts, the text drawer and the count boxes (2026-10-15)

On `client/e17-m3-t4a`, the user present: the agent proposed splitting T4 (its two halves are
independent, and with the panel masked the only text on a battle screen is the count boxes'),
the user chose T4a (the fonts and the count boxes in the stills) over T4 whole, T7a and T8.

- **Starting point:** master at #237 (`3b530ab`, T3b merged at the session's start), no client
  pull request open; a fresh sandbox, the original files staged (one LOD per call); the client
  code's baseline already verified on that commit (`verify-once`: the light gates).
- **Code reading** (design §3.6, §3.7, §4.3; the map's new entries `font_load` 0x55BB00,
  `font_read` 0x55B8D0, `font_from_stream` 0x55B750, `font_construct` 0x4B5070,
  `font_set_palette` 0x4B5180, `text_draw` 0x4B5490, `text_draw_line` 0x4B5260,
  `text_draw_glyph` 0x4B51A0, `text_line_count` 0x4B5820, `crt_fmod` 0x617F20, the site
  `load_startup_fonts` 0x4EE458, the globals `tiny_font` and the screen's three masks;
  `pcx16_adjust_hue_saturation` 0x44E940 read): a font is the file's first 0x1020 bytes and its
  pixels, its colours `game.pal`'s; `TextDraw`'s alignment bits, colour argument, highlight
  braces, line breaking (after the last space that fits; a long word loses the glyph at its
  break; the box's last line cut at any glyph), the line and glyph drawers. The count box:
  drawn for a living stack in group 1 or 2 that is not a war machine and whose +0x30 is clear;
  its inside tinted by the HSV routine with (0.75, 0.8) without spells, else a hue from the
  balance of its spells' kinds; `%d` of the count in `tiny.fnt`, colour 4 (white), centred in
  30 × 15. The tint's f32 V cuts a large channel's low bits (a grey comes out a shade off).
- **Built:** `h3_assets::fnt` (and `build::fnt`; the gate decodes the nine fonts:
  `FNT 9 ok`), `h3_render::text` (`TextDraw` with its quirks; no caret), `h3_render::tint` (in
  integers to 2^−48 and 2^−96, each x87 result and f32 store rounded to 53 or 24 bits: all
  65 536 pixels equal a reference in IEEE doubles for nine hues and saturations, the tests'
  digests), `PixelFormat::channel_masks`; the layout's `[count_box]`, the batch's fonts, the
  scene's `Draw::CountBox` after each stack's sprite and magic animation (the hexes ahead taken
  by every living stack's poses and the blocked hexes), compose's picture, tint and text.
- **The user's decision** (mid-session): the long word's lost glyph is a bug; fixed in what the
  client draws, kept only for the comparisons with the captures, neither a player's option nor a
  quirk profile. Built as `h3_render::Bugs` (`Fixed` the default, `Original` for `--frames`: the
  set and the frame check), taken by `compose`; fixed, the next line starts at the glyph cut off,
  the line is aligned by its own width and a first glyph wider than the box takes a line alone;
  the line count alike. The epic's Decisions and the ADR-0020 point 9 amendment draft record it.
- **The capture set:** the `count_boxes` mask removed from all 30 situations: 55 frames exact on
  the first run. The new `counts` (`grass` with 999, 12 345, 4 567 and 123: three to five digits)
  exact at 0 and 400 ms (the Pikemen's fidget keeps its box); a first try with 65 535 Pikemen
  showed the original drops that stack from the map (a signed −1?), so six digits and the cut of
  seven cannot be set up. 31 situations, 57 frames, all exact, again after the `Bugs` change.
- **The sequences** (the boxes drawn there too, under the frame check's masks until T5): all ten
  frame checks as recorded but `melee` at 3 966 ms, where 40 more of the stale count box's 231
  pixels count as redraw artifacts (136 of 327; the viewer's earlier frames now hold the box,
  so the original's box under the Wolf Riders' shadow is an earlier moment's pixel shadowed):
  recorded. `turn` once showed the computer's stack walking from 2 648 ms (thousands of pixels,
  the original's own timing; nothing of the client changes what the original does); run again,
  as recorded (the runbook's trap: a run on a busy machine).
- **For later:** T5 (the boxes over time: +0x30 set by a walk and a flight, the count's +0x50,
  the spells' tint), T4b (the catalogue; which font the panel and the log use).

### M3 T4b-1 — The string catalogue's format (2026-10-19)

On `client/e17-m3-t4b-1`, the user present: the agent proposed T5a (the count boxes over time
without the spells' tint), T8 or T4b-1 (the catalogue's format alone, no drawing); the user
chose T4b-1.

- **Starting point:** master at `093a930f` (#241, T4a, merged on 2026-10-15), no client pull
  request open; a sandbox an interrupted session of the same day had prepared (the original
  files staged, the gate run, the inbox policy written on `shared/queue-inboxes`, finished here
  as #261). The battle lane's inbox already held T3b's First Aid Tent note (E04c T10c,
  2026-10-16): the client queue's copy removed.
- **Research:** the battle log's rows in `GENRLTXT.TXT` (printf `%s`, `%d`, `%%`; singular and
  plural rows; a line break opening the perish row; the possessive's two rows), the creatures'
  names in `crtraits.txt`; VCMI 1.7.5's text identifiers (`core.genrltxt.<n>`: its parser skips
  the header, so record n + 1) and its JSON translations; Fluent (`fluent-bundle` 0.16: about
  15 crates, numbers through f64, braces that clash with the original's highlight markup) and
  `intl_pluralrules` (CLDR 37); CLDR's current `plurals.xml` (read 2026-10-19).
- **The user's decisions:** the packs are nested JSON (over TOML and Fluent); CLT-Q7, a
  translation of the original's text is never committed, a player may load a pack of their own
  which the project neither ships nor hosts (over never and committing them).
- **Built:** `h3_view::text` (keys, packs, messages, plural rules, the catalogue's lookup,
  `encode_1252`; 100 % covered; the original's tables taken as a concrete map, since a generic
  closure showed false region misses in the summary), `clients/text/keys.toml` (ten keys and the
  creature family), the asset tables' `[[creature]] original` (harvest writes it; a fresh harvest
  equals the committed tables), `h3-view text`, the gate's `--keys` and its check.
- **Checked:** the gate reads all 155 keys and their 306 forms from the user's tables; `h3-view
  text` writes the damage line and the perish line as the rows say, and a Polish test pack's
  `few` wins for 3 while 5 (`many`, which it lacks) falls back to the original's `other`.
- **For later:** T4b-2 (which row and count the executable writes for which event, which font
  the panel and the log use, a log line drawn pixel for pixel), a localized installation's code
  page and fonts.

### M3 T4b-2a — The battle log's lines read and composed (2026-10-19)

On `client/e17-m3-t4b-2a`, the user present: asked for a small increment, the agent proposed
splitting T4b-2 along the code reading (T4b-2a: the log's call sites read, the keys corrected, a
composer) and the drawing (T4b-2b: the log's font, colour and width, a line drawn pixel for pixel
against a capture); the user chose T4b-2a.

- **Starting point:** master at `d3889172` (#263, T4b-1, and #261 merged on 2026-10-19), no
  client pull request open; the client gates' baseline (`check.sh`) passed on the sandbox.
- **Read** (design §9.4, the map): `battle_log_add` 0x472E90 (an entry or the status line, the
  first line break joined or split by the small font's line count, the status line held for
  3 000 ms after an entry), `battle_log_damage` 0x469A90 and what its strike and shot callers
  pass (one target; a breath's second stack; a multi-headed striker's and the area shots' sums),
  `creature_name` 0x440100, wait 0x479918, defend 0x479758, morale 0x464AD5 and 0x464CDF, luck
  0x44187A and 0x43F998; `general_text` 0x6A5D5C. The row numbers checked against the user's
  `GENRLTXT.TXT` (in the sandbox only): the executable's row n is our record n.
- **Found:** a breath over two stacks of one creature reports the target's hit alone (copied;
  the M3 review list); the battle has no bad luck line (row 45 unused); a war machine's death
  has a row of its own (668), its defend no line; no wait line in the tactics phase.
- **Built:** `clients/text/keys.toml` (`battle.log.destroyed` added, `battle.log.luck-bad`
  dropped, the comments citing the sites), `h3_view::battle_log` (a strike's line from its hits
  and their spread, wait, defend, morale, luck, `entries`; 100 % covered, invented text only), the
  gate's battle log check and `h3-view text --log`; the map's entries for the sites read.
- **Checked:** the gate on the user's four LODs: 155 keys, 306 forms, the 12 sample lines
  composed with every argument in place; `h3-view text --log` shows each as the original's rows
  read.
- **For later:** T4b-2b (the log item's width and place, the small font's colour, the status
  line, one line drawn against a capture), T6's event mapping (which event writes which line, a
  `Struck` per target into one strike's hits, the defend's gain).

### H1 — Questions, the difficulty, battles from the map (2026-10-19)

On `client/e17-harness-e3`, the user present: the adventure lane's E08 T22a waited on the
harness's missing features (its inbox request of 2026-10-13); the user had the client lane take
them first. Refined into H1 (this) and H2 (the underground, the hero list's scrolling, the start's
reads, a second day).

- **Starting point:** master at `f8b1dccb`; the clients' full check passed in the new sandbox
  (code 76a427e1). No client pull request open (T4b-1 had merged as `f2a329e8`).
- **Found:** auto combat did nothing at first: the button toggles the battle manager's +0x132C4,
  but a human side's creature acts on its own only with the setting `Combat Auto Creatures`
  (0x698794, read from the registry at 0x50BB6D) on, which a new prefix holds at 0
  (war_machine_automatic 0x474CDF); `setup.sh` sets it. A question's text is read from the window
  manager's topmost dialog, not from the message buffer (0x6973D8), which a map's own text may not
  pass through. The offers' records: ADVEVENT 86 (join), 88 or 89 + 90 (join for gold), 91 (flee),
  87 (the insult, a message). A player left without heroes and towns is out, and the last human
  then wins: the check map gives Blue a second hero. Hot seat needs two names.
- **Built:** `h3adv.py`: `difficulty` (the scenario list's buttons, checked at game +0x1F6D8),
  questions (`answer`, per kind), battles (`battle`: `auto`, `[defender] retreat|surrender`), the
  observations (before and after the step: every player's resources, the hero's details by its
  index, which stays when it leaves the map; the battles' stacks, hexes and outcome; another
  hero's details by name; a tile's monster, aggression and flag), a stop after 30 s on an unknown
  screen; `h3mem.py`: heroes' experience, level, army, skills and primary skills, all 156 heroes,
  the difficulty, a tile's word and monster, creature names from the game's table, the battle's
  outcome, the topmost dialog's text; `h3lod.py` (the LODs' text tables, a text's record);
  `[button.cancel]`, `[button.battle_ok]`; the check spec and recipe; the runbook and design §6.3;
  `docs/exe/` entries (0x698794, 0x699280, the manager's +0x132C4, the dialog records).
- **Checked:** the check recipe on Easy plays its four encounters from memory (a won battle by
  auto combat: 3 Pikemen and 2 Halberdiers to none, 50 experience; a retreat: the hero off the
  map, the 30 Pikemen whole; Blue's surrender in hot seat: 470 gold from Blue to Red, Blue's hero
  off the map; a hero with 5 000 experience: level 5, 3/5/1/2, Diplomacy and Logistics expert).
  The questions on a local map (h3-experiment's monster patched in the sandbox to other
  dispositions, never committed): join no then flee no (the monster gone), flee yes (a battle won
  by auto combat), join yes (27 Pikemen), on Hard. E08 batch 2 again: D3a–D3d all 13 fields as the
  user's, D5's random amounts within their ranges.
- **For later:** H2; the guarded pile's question and a monster's offers in a committed check
  once h3-experiment places them (E08 T22a); a level-up dialog after a battle (not met yet).

### H2 — The underground, more heroes, the start's reads, a second day (2026-10-19)

The same session and branch as H1, right after it.

- **Built:** `h3mem.py`: the level in a packed tile (bit 26), the path's arrows of both levels,
  the map's levels, the date (game +0x1F63E); `h3adv.py`: `walk = [x, y, level]` (the panel's
  level button, the minimap on that level, the hover checked with its level), the hero list
  scrolled for a player with more than five heroes, `level_up` (the left or right skill of a
  level-up dialog, `[button.ok_disabled]`), the observations `date`, `level_ups`,
  `heroes:experience`; `h3drive.Game.start` ends a game left running first (two games on the
  display made the dumps unreadable). The `adventure` probes leave out the level button.
- **Checked:** the check recipe with three more Red heroes (seven: the list scrolled for the
  sixth and seventh), a level-up (1 500 experience: level 2, the left skill taken), both turns
  ended and a walk on day 2 (`month 1, week 1, day 2`, the day's 1 560 points), the heroes off
  the map (147: 40 to 90). The underground on the shipped `Arrogance` (hot seat with four
  humans, Purple's hero on level 1): a walk to (5, 8, 1) after the view was put on the surface,
  three green marks on level 1, arrived.
- **For later:** the underground in the check once `h3-map`'s writer places it; a committed
  check of the monsters' offers and a guarded pile's question once `h3-experiment` places them
  (E08 T22a).

### M3 T4b-2b-1 — The log's item in the panel (2026-10-19)

On `client/e17-m3-t4b-2b-1`, the user present: asked for the next task, the agent proposed
splitting T4b-2b into the item read, drawn and checked on the still capture set (T4b-2b-1) and a
strike's line against a capture (T4b-2b-2); the user chose that.

- **Starting point:** master at `6c35bb4` (#269, T4b-2a, merged on 2026-10-19).
- **Read** (design §9.5, the map): `combat_dialog_create` 0x4721D0, `battle_panel_create`
  0x46BC30 (the panel at (0, 556), 800 × 44), the log's item 0x46BCBE (a text item with a
  picture: at (214, 563), 400 × 32, `smalfont.fnt` over `cRollovr.pcx`, colour 1, alignment
  1), `text_item_create` 0x5BC280, `text_pcx_create` 0x5BC760, their drawing 0x5BC7F0 and
  0x5BC5F0, and `battle_log_show_from` 0x472C30 (the first shown entry and the next, a line
  break between, as the item's text).
- **Built:** the layout's `[log]`, `Draw::Log` and `scene::panel` (the adapters append it after
  the battlefield, empty), `compose` draws it (100 % covered); `h3cap`'s mask
  `panel_beside_log` and the capture set on it.
- **Found:** the original at the opening shows the item's picture and no text; the harness's
  parked cursor (420, 580) lay inside the item (77 pixels): parked at (618, 593) since.
- **Checked:** the capture set, 31 situations, every frame as recorded (exact) with the log's
  item compared; the recorded sequences' timelines as recorded with the new park point, but
  `machines`: in 7 of 8 runs the automatic Ballistas' shots (and the heal after one) started
  1 ms later than recorded, at either park point (the old one too, 2 of 3 runs), a run-to-run
  variation in this sandbox, not the park point's (the capture README's traps).
- **For later:** T4b-2b-2 (a strike in hot seat, its line in the log against the capture: the
  panel's frame at a sequence's end and our render with the composed line), T6.

### M3 T4b-2b-2 — A strike's line against a capture (2026-10-19)

On `client/e17-m3-t4b-2b-2`, the user present: asked for a small increment, the agent proposed
narrowing T4b-2b-2 to one line in a still (a fixed-damage strike in hot seat, the original's
frames from the next turn with the log's item compared), with when the line appears, several
entries and the scroll left to T6; the user chose that.

- **Starting point:** master at `aaa9630` (#275, T4b-2b-1, merged on 2026-10-19, and two of the AI
  lane's commits after it), no client pull request open; the client gates' baseline (`check.sh`)
  passed on the sandbox.
- **Found:** every recorded sequence's strikes roll their damage, so the engine's line is not the
  original's; the base pack's fixed damages are the Angels', Archangels', Nagas' and Naga
  Queens'. In the original the log's item shows the status line's hint over the target (`Attack
  Pikemen`) from the click until the line replaces it; the first kept frame with the line is the
  next turn's (3 730 ms).
- **Built:** the sequence `strike` (`seq-strike.toml`: the Archangel kills all ten Pikemen with
  100 damage; three factions and the Archangel's +1 make morale 0), recorded (timeline 57
  changes, 58 frames); `battle_log::shown` (0x472C30); `h3_render::text::line_count` public;
  `h3-view`'s `log` module (the original's catalogue, `first_strike`, `after_one`;
  *provisional*); a sequence's `log_at` (`h3-view sequences --frames` draws the line from then
  on, `h3drive.py frames` compares the log's item from then on).
- **Checked:** the log's item exact in the six frames from 3 730 ms (1 166 text pixels); the
  viewer's timeline as recorded; the other differences count boxes (T5's: the Pikemen's while the
  Archangel lands, the Archangel's once their hex is free); the capture set (31 situations) and
  the ten sequences' timelines as recorded.
- **For later:** T5 (the two boxes), T6 (when the line appears, the hint in the status line,
  several entries and the first shown, the event mapping that replaces `first_strike`).

### M3 T5a — The count boxes through walks and flights (2026-10-20)

On `client/e17-m3-t5a`, the user present: asked for a small increment, the agent proposed
splitting T5 by whether a box's number changes (the engine rolls damage differently from the
original, so after a rolled strike the digits differ): first the boxes of the four sequences
whose counts never change (`walk`, `turn`, `climb`, `strike`); the user chose that and moved the
spells' tint to T7b (no sequence casts on a stack before T7a). The user's computer went offline
during the session, after the original files were staged.

- **Starting point:** master at `5c4fc0b` (#280, T4b-2b-2, merged on 2026-10-19, and the adventure
  lane's #279 after it), no client pull request open; the client gates' baseline (`check.sh`,
  `verify-once`) passed on the sandbox.
- **Code reading** (design §4.3; the map's `stack_walk` and `stack_death`, the stack's +0x30):
  +0x30 is stored only by the walk (set before the path's first turn, 0x442145; cleared at the
  end, then the whole battle redrawn and shown, 0x4423D9) and the flight (set as the cycles start,
  0x4B4BD8; cleared after group 21 and the standing frame, 0x4B4E73, with no redraw); the box
  test (0x43E605) reads the hex's occupant and its +0x30; a stack killed by a strike leaves the
  hex records after the strike's iterations (`stack_death` 0x444400 from 0x4692C4).
- **Built:** `Pose::boxless` (the walk's and the flight's poses carry it; the walk's last pose
  shown again unhidden by the redraw at the same moment), `Pose::holds_hex` (not hidden, not
  among the corpses); the scene places the boxes by the stacks holding their hexes at the moment
  and draws none for a hidden one. The harness: a sequence's `boxes = true` compares the count
  boxes; the others mask them also where the viewer drew them (`ours.txt` lists them,
  ` box<x>,<y>`), so their counts stay as recorded.
- **Checked against the original** (the four captured afresh): `walk`, `turn` and `climb`
  artifacts only with the boxes compared (in `turn` and `climb` the standing frame after the
  first turn hides the box; `climb` at 1 696 ms 16 more artifact pixels, a box under a shadow);
  `strike` artifacts only but 134 pixels at 2 730 ms (a piece of the Pikemen's earlier box from
  the saved screen, 123, and the new box's last column outside the rectangle shown, 11): listed,
  not classified (a rule by a box's earlier place would pass a box moved too early, as the
  Pikemen's was before). Before this, with the boxes unmasked, 574 to 872 pixels in each frame
  of the landing and 606 in each from the Goblins' turn. One of two `strike` runs read the left
  hero standing at the first frame of its reaction (2 400 pixels at 3 730 ms; the runbook's
  "Traps"); the other, used, as recorded.
- **The other six sequences** captured again (their boxes masked): `shot`, `behind`, `shots`,
  `fly` and `machines` as recorded; `melee` at 3 966 ms now artifacts only (96): the box T1c
  listed as stale is the Angels' beside them, the Wolf Riders' hex free while they walk, which
  the viewer now draws exactly (its 330 pixels equal); recorded. The capture set: 31 situations,
  57 frames as recorded (a first run stopped on a broken screen dump; the second ran through).
  The ten sequences' timelines as recorded.
- **For later:** T5b (the numbers over time, the other six sequences' boxes compared), T7b (the
  spells' tint).

### M3 T5b-1 — The count box's number over time (2026-10-20)

On `client/e17-m3-t5b-1`, the user present: asked for a small increment, the agent proposed
splitting T5b into the reading and the build checked in `shot` and `melee` (one shot, one melee
with a fixed-damage strike back) and the other four sequences' boxes (T5b-2); the user chose
that.

- **Starting point:** master at `fee5b69` (#285, T5a, merged on 2026-10-20), no client pull
  request open; the client gates' baseline (`check.sh`, `verify-once`) passed on the sandbox.
- **Code reading** (design §4.3; the map's `stack_take_damage`, `strike_animation`, the stack's
  +0x50): +0x50 is −1 from the setup; `stack_take_damage` stores the count before a damage's
  deaths there (0x4440DB) before +0x4C loses them; strikes and shots take their damage before
  `strike_animation`, whose end (after the pending spells' casts and the deaths) sets every
  stack's +0x8 to 4 and its +0x50 to −1 and redraws the whole battle, shown at once (0x46935A,
  0x46938A). Confirmed in the original's memory (`h3mem`'s new `shown`): `shot`'s Goblins 40 →
  37 in +0x4C as the first shot lands (988 ms), +0x50 40 until 1 588.
- **Built:** `anim::CountKey` and `Timeline::counts` (each struck stack's count after its kills,
  from the strike's last frame shown), `Moment::counts` over the snapshot's count in the scene,
  `Draw::CountBox::stack`; the strike's end flushes its pending poses at once (a test of the
  layers changed: a later strike of the command now follows a layer 4). The harness: `ours.txt`
  lists each box's count and stack (` box<x>,<y>n<count>@<side>.<record>`); with `boxes`, the
  frame check masks a box only where its count is not the original's (+0x50, else +0x4C) and
  lists it (` n<side>.<record>`). `melee` runs 7 000 ms (6 000 before) to the strike back's end.
- **Checked against the original** (all ten captured afresh): `shot` exact in all 50 frames with
  the boxes compared (37 from 1 588 ms, the engine's kills the original's); `melee` artifacts
  only in its 85 frames (30 → 12 at 6 466 ms; at 3 966 ms 198 artifact pixels instead of 96, the
  Angel's shadow drawn twice where the box mask covered it); no box masked for its count. The
  other eight as recorded (`strike` on its second run: the first read the left hero's reaction a
  frame late at 3 730 ms, 2 152 pixels, the runbook's trap; `machines`' timeline 1 ms later at
  2 222 and 5 846 ms, the runbook's known variance). The capture set: 31 situations as recorded.
- **For later:** T5b-2 (`behind`, `shots`, `fly`, `machines`' boxes), T7b (the spells' tint).

### M3 T5b-2a — The count boxes in `behind` and `shots` (2026-10-20)

On `client/e17-m3-t5b-2a`, the user present: asked for a small increment, the agent proposed
splitting T5b-2 into `behind` and `shots` (battles in progress with several strikes and shots,
the mechanism T5b-1 built) and `fly` and `machines` (a breath on two stacks, the heal's reading:
T5b-2b); the user chose that.

- **Starting point:** master at `7ac0b91` (#289, the adventure lane's, after #290, T5b-1, and
  #291), no client pull request open; the client gates' baseline (`check.sh`, `verify-once`)
  passed on the sandbox.
- **`behind`** runs 4 000 ms after its last action (1 000 before) to the Marksmen's second
  shot's end (15 492 ms) and their turn back (15 892): its timeline 227 changes (203 before, the
  same up to 13 499), the viewer's as recorded. With the boxes compared its first 194 frames are
  as before (the Centaurs' 20 → 12 at 11 144 ms, the Pegasi's 10 → 9 at 9 844 and → 8 at
  14 268: the engine's kills the original's); the second shot kills one Pegasus in the engine,
  none in the original: their box masked and listed from 15 492 (` n1.0`).
- **A shot's struck in layer 5** (design §7.10): the original's memory holds the struck stacks
  of every shot and ray in layer 5, the shooter in 4 (`behind`, `shots`); the viewer had kept
  them in 4, so the hit Pegasi's wings lay under the Centaurs beside them (242 to 14 pixels a
  frame, 13 768 to 14 068 and 14 992 to 15 292 ms). Built: `shot` and `react_alone` take
  `[STANDING, STRUCK]`; two tests' poses in layer 5.
- **Listed, not classified** (design §7.11, "A projectile over the obstacles"): in the first shot
  after the Marksmen's turn the arrow's earlier steps leave the terrain alone in the rocks
  (13 636 to 13 735 ms, 1 036 to 1 913 pixels, no artifact by `h3cap.redraws`), not in the
  second; the save-under's source is the likely cause, not read.
- **`shots`** exact in all 139 frames with the boxes compared; the Gnolls' box listed from 3 471
  to 5 479 ms (` n1.1`: the Cyclops' throw kills 6 in the engine, 7 in the original; after the
  Arch Magi's ray both show 19).
- **The harness:** `xwd_bytes` also retries a dump that is no 5-6-5 dump (two `set` runs stopped
  on one, at different situations; the third, with the retry, ran through).
- **Checked against the original** (all ten captured afresh): `walk`, `shot`, `melee`, `turn`,
  `climb`, `fly`, `machines` as recorded; `strike` as recorded on its third run. The original's
  rolls varied between runs (runbook "Traps", not explained: morale and luck are 0 in these
  scenarios): one `strike` run left a Pikeman alive (9 killed, then their strike back), one gave
  the Archangel good morale at 3 730 ms (magic 20); a `machines` timeline run played magic 18
  over the Archers as they shot (its two frame runs as recorded); a `climb` timeline run had the
  computer's stack walk at 2 396 ms (its frame run as recorded). The capture set: 31 situations,
  57 frames as recorded.
- **For later:** T5b-2b (`fly`, `machines`), T7b (the spells' tint).

### M3 T5b-2b — The count boxes in `fly` and `machines` (2026-10-20)

On `client/e17-m3-t5b-2b`, the user present: asked for a small increment; the agent offered
`fly` or `machines` alone or both, the user chose both (the task as queued).

- **Starting point:** master at `c4fe6b1` (#297, after #299, T5b-2a), no client pull request
  open; the client gates' baseline (`verify-once`: the code verified on #299) passed on the
  sandbox. SDL's build dependencies (`clients.yml`'s apt list) had to be installed by hand for
  the release build: the light baseline does not.
- **The heal read** (design §4.3; `first_aid_tent_action` 0x478B90): the heal comes off the
  patient's +0x58 (0x478C22), +0x4C is only read for the log line's name (0x478C6A), +0x50 not
  touched: a heal never changes a box's number.
- **The draw order fixed** (design §4.2, §4.3, §4.6, §7.15): `machines` with its boxes compared
  differed in the patient's box through the heal (5 945 to 7 845 ms, 6 to 29 pixels a frame):
  the original's heal frames blend over the box's border and tint. `draw_battle_stack`'s part 0
  runs on from the sprite into the box (0x43E568) and draws the magic animation after it
  (0x43E795); `draw_hex_stack` calls part 1 (the box alone) only in a siege, after the moat's
  parts (0x494FD1). T3b had read "after the sprite, before the box". Built: the scene emits
  `Draw::CountBox` before `Draw::Magic` (a test of the order). `machines` then exact in all 106
  frames; the Archers' shot kills one Pikeman in both (20 → 19), the Ballistas none.
- **`fly`** with its boxes compared: all 102 frames as before (17 artifacts only); the breath
  kills 8 Pikemen and 13 Gnolls in the engine, 9 and 16 in the original, so both boxes are
  masked and listed from 4 300 ms, the breath's end (` n0.2`, ` n0.3`); before it both show 30.
  No `ms` lengthened: every strike ends inside both sequences.
- **Checked against the original** (all ten captured afresh): every sequence's frames as
  recorded, `strike` on its second run (the first left a Pikeman alive again, 10 → 1: the
  runbook's trap); the viewer's timelines as recorded; the capture set's 31 situations as
  recorded.
- **For later:** sieges draw the box a second time after the moat's parts (part 1); T7b (the
  spells' tint).

### M3 T6a — The panel's still (2026-10-20)

On `client/e17-m3-t6a`, the user present: asked for a small increment and to confirm the seam;
the agent proposed T6 split in three (the panel's still, the log over time, the status line with
the cursors and the mouse), the user chose the first and asked for reusable UI components ("there
are just a few types of buttons that are used in lots of places in the entire game").

- **Starting point:** master at `c4fbe58` (#312, after #313 and #314), no client pull request
  open; the client gates' baseline (`verify-once`: not verified yet, the full `check.sh`) passed
  on the sandbox.
- **The reading** (design §9.7): `battle_panel_base` (0x46B610) builds the picture and the seven
  48 × 36 buttons both panels share, `battle_panel_create` (0x46BC30) the log's item and its
  scroll buttons; the picture item (0x4502D0: keyed, in a player's colours by 0x450520), the DEF
  button (0x455EF0, its drawer 0x456940: a frame of group 0 by its state bits, frame 0 beyond the
  group), the custom button sharing its drawer (0x456D30), the control states (0x5FEED0,
  0x5FEF60, 0x5FE7C0 into 0x5FE4F0: subtype 5 sets, 6 clears, 0x1000 is bit 8, disabled);
  `battle_controls_state` (0x4782D0) in full and 0x46BF90 (a computer's turn disables the seven
  buttons). Thirteen map entries, two summaries extended.
- **Built:** `h3_view::ui` (the item kinds as components: `Dialog`, `Item` by `kind`, a button's
  `Frames` and `State`, `Shown` by control id, `Dialog::draws`); `h3_view::panel` (the controls'
  states at a turn's start from the observation: `human`, the heroes, the spell book and casts,
  the wait and tactics phases, the town; the acting stack's controller, `scene::acting`); the
  layout's `[panel]` with its eleven `[[panel.items]]` (replacing `[log]`); `Draw::Picture`,
  `Draw::Button`, `Draw::Text` (replacing `Draw::Log`); `h3_render::blit::draw_image_keyed_in`.
  The scenarios of the situations and sequences name their human sides (`human`; the smoke battle
  both, as its hot-seat map), and `h3-view situation` refuses a scenario whose `human` is not its
  map's: before, every record had two computer players, so the panel would have been dimmed.
- **`drawn` and the captures:** drawing the panel changes every frame, the ten sequences' too,
  though they mask the panel. `drawn` now digests each frame's pixels outside the fixed masks its
  comparison applies (`png_digest`, `compared_masks`, shared with `frames`); the digests were
  recorded again from `master`'s viewer in a worktree (the old digests there: all 41 the same, so
  the frames are the ones last compared), before this change. With the panel: the ten sequences
  the same, the 31 situations changed; `set` in full.
- **Checked against the original:** the first `set` run differed by 35 pixels in every frame, the
  game's own cursor where the harness parks the mouse (618, 593), which `panel_beside_log` had
  hidden; masked as `cursor` (`CRDEFLT.DEF`'s canvas there, h3cap), the second run: all 31
  situations' 57 frames exact with the whole panel compared (the human's opening in red with the
  spell book disabled and the scroll buttons enabled; the smoke battle on Blue's turn in blue);
  `drawn --record`.
- **For later:** the panel over time and the log (T6b), the status line, the cursors (the parked
  cursor's mask goes with them) and the mouse (T6c); the tactics phase's panel (0x46C050) and
  Auto Combat switched on (the queue's inbox).

### M3 T6b — The panel and the log over a battle (2026-10-20)

On `client/e17-m3-t6b`, the user present: offered T6b split in two (the panel over time, the log
over time), the user chose it whole.

- **Starting point:** `master` at `774ba0e` (#322 merged as `3618525`); no client pull request
  open; the client gates' baseline (`verify-once`, the full `check.sh`) passed on the sandbox.
- **The reading** (design §9.8): what sets the controls' states while a battle runs (an action
  and its animation set none; a computer's turn keeps the colours and the scroll buttons);
  `battle_log_add`'s two paths in full, `battle_log_settle` (0x472BF0) and its callers, the scroll
  (0x472DB0, 0x472DF0, 0x472E40: H3API's ShowHint guess at 0x472DF0 is the up button), the
  panel's set-text (0x46BF50: shown at once), the mouse posted at a human's turn (0x4ECC00); where
  `melee_strike`, `stack_shot`, `first_aid_tent_action` and `round_start` write their lines (rows
  415 and 413 new keys). Six map entries, four summaries extended.
- **Built:** the timeline's `log` (`battle_log::Logged`: an action's and a turn's start, a strike's
  line after its animation, an area shot's as its reactions start, wait, defend, heal, morale,
  luck, a round), `battle_log::Log` (the executable's state), `word`, `texts`, `text_at`;
  `panel::turns` (each turn's battle folded from the events), `panel::kept`; `scene::Panels`
  (replacing `scene::acting`) for the frame check, the stills and the window; the adapters' log
  (`log::panels`, replacing `first_strike`, `after_one` and `log_at`; the text tables read only
  when the battle writes a line); the gate composes the heal's and the round's lines.
- **The harness:** a frame is kept where the screen changes outside the parked cursor (the
  panel's changes fell on kept frames in all ten); `h3mem.battle_log` reads the item's text and
  the entries; the frame check masks the parked cursor only and lists the log's item (` log`)
  where the original's text is not the viewer's (`ours-log.json`) and its pixels differ (the
  memory lags the screen a step as a turn starts: at first every such frame was listed).
- **Found on the way:** on a computer's turn the scroll buttons stay as the human's turn enabled
  them (`melee`: 539 pixels until `Kept`); the hover's hint ignores the 3 000 ms rule (`behind` at
  7 144 ms; T6c, `NOTES.md`).
- **Checked against the original:** all ten sequences captured afresh: counts as recorded (896
  frames), the log's item exact in 228, listed in 668 (hints; rolled lines). One `strike` run read
  the left hero's reaction a step late at 3 730 ms (2 152 pixels, the runbook's trap); run again,
  as recorded. One `set` run failed on a screen dump (not a 5-6-5 dump); run again: 31 situations,
  57 frames exact. `drawn --record`, then `drawn` all 41 the same.

### M3 T6c-1 — The mouse's still (2026-10-20)

On `client/e17-m3-t6c-1`, the user present: T6c split with the user into the still (T6c-1) and
the mouse over time (T6c-2, which also takes the window's mouse).

- **Starting point:** `master` at `59eda2c` (#333 merged); no client pull request open; the
  client gates' code verified on `master` before (`verify-once`: the light gates only).
- **The reading** (design §9.9): `battle_buttons`' mouse move, `screen_to_hex` and the heroes'
  rectangles, the neighbour table, `battle_cursor` in full, the attack's slots in full,
  `battle_status_line` for the modelled kinds, `hex_distance`, the mouse manager (`SetCursor`,
  the cursor's frame drawer, its hot spots and its even x), the panel's hints (`SetHints`,
  `Help.txt`'s loader). Fifteen map entries, three summaries extended.
- **Built:** `h3_view::mouse` (where the mouse is, the neighbours, the distance, the legal
  commands as the cursor asks them, the kind, the slot and the twelve slots' frames with the
  executable's copy measure, the hint and its text, the pointer and its cursor draw);
  `Draw::Cursor` (plain, runs skipped); `Moment::marked` (the mouse's outline, the active one's
  winning); `ui::Item::hint`, `Dialog::item_at`; the layout's `[mouse]` and `outline_mouse`; the
  keys `battle.hint.*` and `battle.control.*`; `render --frames` draws a `frames.toml`'s mouse
  with the scenario's legal commands at the opening (`scenario::opening_legal`;
  `sequences::Interface`).
- **The harness:** a situation's `mouse` (default the parked point): moved there at the opening,
  the clock frozen; `frames.toml` names it. 26 `mouse-*` situations over four scenarios.
- **Found on the way:** at the opening itself the original shows the new cursor where the mouse
  was (its overlay moves at its next frame): the mouse frames start at 100 ms. The software
  cursor's x is even (0x50CE04): a one-pixel shift at odd points (144 pixels a sword, 101 the
  pointer); after review the user called it a drawing bug: the client draws the cursor at the
  mouse, the comparisons round it (`Bugs::Original`). Archangels are two hexes wide (the one-hex swords from Arch Devils).
  `h3lod` counts `Help.txt` one record fewer than `h3-formats` and the executable from the
  Multiplayer Selection group on (NOTES). The parked cursor is the combat cursor's frame 6.
- **Checked against the original:** the 26 mouse situations' 28 frames exact; the whole set
  captured afresh with the parked cursor drawn and the `cursor` masks gone: 57 situations, 85
  frames exact. `drawn --record`.

### M3 T6c-2 — The mouse over time (2026-10-21)

On `client/e17-m3-t6c-2`, the user present at the start: offered a split (the record, then the
window), the user took T6c-2 whole.

- **Starting point:** `master` at `f86af41` (#336 merged); no client pull request open; the
  client gates' full check on that code (`verify-once`: all gates, coverage 100 %).
- **Explored first** (the harness reading the mouse's state from memory: hex, kind, cursor type
  and frame): in `strike`, `walk` and `machines` the cursor is the pointer from the click on
  (every action shows it), the hint written at the click stays until a line or the next human
  turn, the mark goes as the action starts; `hover` showed the cursor moving to the mouse 19 ms
  after the opening in one run and 10 ms in another (the mouse poll's 33 ms grid, its phase
  left by the run before the battle).
- **The reading** (design §9.10): the mouse move on the same hex and the status line's rewrite
  (0x475544 to 0x4758D1), a human's turn start (0x47883E to 0x478873), the dispatcher's head
  and switch, the unmarking callers of `battle_clear_highlight`, `battle_stand_fidgets`
  (0x479DE0), `battle_cursor`'s target, `battle_mouse_stack`'s tail, `mouse_draw_cursor` and
  `mouse_poll` (0x50D680, 0x69CA18), the DEF button's handler (0x456190), the panel's hotkeys.
  Five map entries, four summaries extended, `structs.toml`'s mouse fields and the mouse manager.
- **Built:** `h3_view::mouse::track` (`follow`: the log's entries and the inputs in time order,
  the moves queued through actions and taken at a human's turn with the posted one, the hints
  into the log as `Logged::Status`, the marks and the entered stacks, the cursor's frames);
  `mouse::Facing`, `Field`, `pointer_in` (the target left over: "No shots left!"), `slots_in`,
  `cursor`; `Timeline::enter` (`anim/entered.rs`: group 1 on the idle steps, stood by a standing
  action, a step at the entry's time first), `anim::unfidgeted`; `hero::Action`'s `start`,
  `stands`, `unmarks`; `Logged::Scroll`; `ui::Button::keys`, `Dialog::button_at`,
  `button_for_key`, `Panels::at_pressed`; `Cursor::read`; the layout's `keys`; the key
  `battle.hint.no-shots`. Adapters: `pointer` (the harness's actions as inputs, `PARK`,
  `hex_point`), `scenario::turned` and `fought`'s commands, `log::Writer`, the sequences'
  mouse, `render --at` with the parked mouse, the window's mouse (cursor frames, hints, marks,
  entered stacks re-made on a move), pressed buttons, hotkeys, the log's scroll; the window's own
  keys moved (P, Home, H); `h3-sdl` reports key-ups and the left button.
- **The harness:** `h3mem`'s battle state gains the mouse (hex, kind, cursor type, frame and
  drawn point); a sequence's screen digest takes the cursor, no fixed mask; `frames.toml`'s
  per-frame `cursor`; the mouse poll's next time set at a sequence's opening
  (`Mem.align_mouse_poll`, its only write); `hover` captured and compared; `h3lod` reads the base
  archive's tables first (the AB copies' GENRLTXT differs from record 79, Help.txt from 96: the
  count T6c-1 found); `h3adv` prints a winner of -1 as none (the adventure lane's inbox).
- **Found on the way:** entering a stack redraws only its new frame's rectangle: in `hover` one
  pixel of the standing Pikemen (0 and 33 ms) and thirteen of their frame 2 (200 and 231 ms) stay,
  each equal to the original's screen just before (listed: the viewer composes whole frames);
  a war machine's turn in hot seat is a human's to the mouse (its hex forgotten, `machines` at
  2 221 ms); `strike`'s Pikemen, entered by the click, stand as the strike starts
  (`battle_stand_fidgets`); in `shots` the panel showed under the parked point while each ray
  plays (`draw_ray` hides the cursor, 0x5A5C46 to 0x5A61D4: built as `Beam::cursor_back`); the
  `turn:` action names the stack's next turn after the last one named (`machines`' second).
- **Checked:** every timeline as recorded (eleven, `hover`'s with the original's own fidgets left
  out); the frame check captured afresh (all eleven, the cursor compared, nothing fixed masked):
  922 frames, every count the ten sequences recorded before unchanged, their ` log` listings
  668 to 173 (each a rolled line), `shots` six frames more (the cursor hidden and shown, exact),
  `hover` 20 frames (16 exact, four with the leftover pixels); the set not captured (its capture
  did not change; `drawn`: every situation the same); `drawn --record`; the window under Xvfb
  with the mouse moved and clicked over the field and the panel: every frame read back exact;
  `master` merged in (E08 T27, E23 T2a); `check.sh` all gates (coverage 100 %); the real-asset
  gate passes.

### M3 T7a-1 — The magic animations read (2026-10-21)

On `client/e17-m3-t7a-1`, the user present at the start: T7a split with the user at its seam
(T7a-1 the code reading alone, from the executable; T7a-2 the harness casting from a hero's spell
book and the recorded sequences against the reading).

- **Starting point:** `master` at `01ce622` (#341 merged at `c0c955e`, then the battle lane's
  #340); no client pull request open; the client gates' full check on that code (`verify-once`:
  all gates).
- **The reading** (design §7.16): `play_magic_animation` whole (its `show_hit` part), the at-hex
  player 0x496A10, `mass_spell_animation`, `raise_stack`'s and Armageddon's drawing,
  `strike_animation`'s magic part, the spell projectile 0x467DB0, `cast_spell`'s opening (the
  hero's cast frames, Magic Mirror's redirection, the resisted path) and tail (the hero's
  frames left, the Familiars), every handler's drawing, `creature_cast_action`'s cast group,
  the creatures' abilities in `striker_effects`, `stack_shot` (Magogs, Liches), the turn and
  round starts (regeneration, Poison, the Wraiths, the objects' vanishing), morale, fear and
  luck; every call site of the four players (22, 11, 6, 21) with its animation and sound; the
  spell records' static sound and animation, the hero DEF records' cast point and frames.
- **Found:** the handler decides how an animation plays, not its place (seven players); several
  animations named by a call are never drawn on the paths read (Chain Lightning's 38 and Fire
  Shield's return 11 mark no stack; Animate Dead plays Resurrection's 50; 4, 13, 43, 44, 52,
  62, 63, 82 met nowhere); a mass animation never shows its last frame (n − 2 twice); the
  Liches' cloud is drawn plain though the table marks it translucent; a creature caster's group
  takes an angle whose sign and points differ from the shot's (a level target nearby: 17),
  where VCMI takes the shot's; good morale's log line comes after its animation, bad morale's
  and fear's before. All *provisional* until T7a-2's recordings.
- **The map:** 16 entries added (the at-hex player, the projectile, `battle_show_field`, the
  dissolve's two, DrawTransparent's, four `cast_spell` sites, six projectile tables), 10
  completed (`mass_spell_animation` and `enchanter_shot` now read); `structs.toml`: the stack's
  glow +0x450, the manager's +0x13300, +0x547C's users.
- **Checked:** `scripts/exe.py check` and `cited` (every cited address covered), `cargo xtask
  check-exe-map` and `check-docs`; `verify --scope client`. No code changed: no capture needed.

### M3 T7a-2a — The harness casts (2026-10-21)

On `client/e17-m3-t7a-2a`, the user present: T7a-2 split with the user (the harness first,
T7a-2a; the other recordings and candidates T7a-2b). The user's computer was linked for the
staging only (all files staged once, the session then ran in the cloud).

- **Starting point:** `master` at `71411b0` (#344 merged); no client pull request open; the
  client gates' full check on that code (`verify-once clients`: all gates).
- **The reading** (design §7.16, "The spell book"): `spell_book_dialog` 0x59BDF0 (its twelve
  places, tabs, bookmarks, close), its page 0x59C9A0 (the spells listed, their sort 0x59DBE0 by
  level, school bit, name; the school `hero_best_school` 0x4E51C0), its handler 0x59D040 (a
  click on a place returns the spell when the mana suffices), its memory of page and tab
  (0x684B40, 0x6A34F4); the Faerie Dragon's cast through the cursor (`battle_cursor`
  0x4766FC: kind 20 over any stack its drawn spell accepts). 7 map entries.
- **The harness:** `h3-view situation` writes a hero's spell book (slot 17, its spells by
  `sptraits.txt`'s names: the SoD pack adds 57, Titan's Lightning Bolt, after the base pack's
  others, so a handle is the original's number only below 57) and checks the mana (10 per knowledge, the original's start);
  `h3drive.py`'s `cast:<k>[:<hex>]` (the key C, the place, the target), `--heroes` / `heroes =
  true` (the heroes' changes in the timeline), `battle_book` (the battle screen's probes lie on
  the spell-book button, lit when the acting hero has a book; the existing scenarios never light
  it at their opening, so their openings stay: `drawn` all the same, a spot capture as recorded),
  `turn:` failing after 30 s; the viewer's `pointer::inputs` takes `cast:`. The book opens and
  casts at the frozen clock (first try, the screens looked at).
- **Recorded** (`situations/casts.toml`, kept beside `sequences.toml` until the viewer plays
  them; both replayed as recorded): `cast`, Orrin's Slow at the Wolf Riders: the hero's frames
  0–4 at 100 ms, Slow's animation 19 (20 frames, 100 ms) over the target after its effect, its
  erasure, frames 5–7, standing: as read. `dragon` (hot seat, the dragon flies beside the
  Swordsmen, then casts three rounds): group 17 at the level target beside it, 19 at one four rows
  above, 17 at one two rows below, as read (VCMI 18, 17, 19); frame 0 held 100 ms, then 41 ms
  (attack time 500 / 12 frames), the last kept through the spell; its drawn spells (the log's
  lines, kept in the sandbox): Fireball, Fireball, Inferno, at a hex (player 5, not in memory).
- **For the review list:** the caster's group by its own angle confirmed (faithful or fixed?);
  the harness's choices (above), *provisional*.
- **Checked:** the viewer's tests (`situation`, `scenario`, `pointer`, `harvest`), `cargo fmt`,
  clippy; `scripts/exe.py check` and `cited`; `h3drive.py drawn` (68 of 68 the same);
  `sequences situations/casts.toml` (2 of 2 as recorded); `set --only heroes-tower-inferno` and
  `frames --only walk` with the new harness (as recorded); the client gates' full check.

### M3 T7a-2b — The magic animations recorded (2026-10-21)

On `client/e17-m3-t7a-2b`, the user present at the start (asked to split, they chose the whole
task at once). The user's computer was linked for the staging only.

- **Starting point:** `master` at `2518a92` (#348 merged); no client pull request open; the
  client gates' full check on that code (`verify-once clients`: all gates).
- **The harness:** `screens` (`--screens`; an entry's `screens = true`): each time whose screen
  differs from the step before ends with `<ms> screen x0,y0,x1,y1`, the changed pixels' box
  (numbers only), for the players memory does not show; a `turn:` that stalls writes its states
  and timeline before it fails. A key pressed at the frozen clock after a hero's cast acts once
  the cast has played (the stack defends as the hero stands). `h3-view situation` gives a hero
  the school skills its scenario's `masteries` ask (`school_skills`; Expert Earth Magic for a mass
  Slow), checked against the book's spells.
- **Recorded** (`situations/casts.toml`, design §7.16 "Recorded"; each scenario hot seat on the
  capture set's grass): `resist` (Slow resisted by Battle Dwarves in round 2), `arrow` (Magic
  Arrow at row 0, at an Archangel flown beside the hero, at row 10), `bolt` (Lightning Bolt, Chain
  Lightning), `mass` (Slow at Expert, Death Ripple; Marksmen, speed 6, so that the attacker acts
  first: slowed Swordsmen still came before Pikemen), `raise` (Resurrection and Animate Dead on
  stacks shot dead), `obstacles` (Fire Wall, Quicksand, Force Field, Armageddon; a wall beside an
  obstacle is refused), `creatures` (an Archangel striking Efreet Sultans, Power Liches' shot),
  and `dragon` again with its screens (its memory lines as T7a-2a recorded them). `arrow`, `mass`,
  `raise`, `obstacles` and `creatures` ran first on memory alone, then with the screens: the same
  memory lines.
- **Corrected by recording:** Chain Lightning marks every struck stack (the marks copied into
  each stack's +0x20 at 0x5A65D1 to 0x5A6620) and 38 plays over them; Fire Shield's return marks
  the shielding stacks (0x440AA9) and 11 plays over them; the projectile's DEF is nearest
  atan(−Δy / |Δx|) (0x467E5D to 0x467F14: a target below takes the DEFs pointing down); a mass or
  Armageddon reaction's hit runs on through the standing group to its last frame. Four map
  entries rewritten.
- **Confirmed:** a mass animation's last frame never shown (n − 2 twice: Slow's 19, Death
  Ripple's 8), Animate Dead's 50, the Liches' cloud plain (every changed pixel one of `sp04_`'s
  colours). **New:** Chain Lightning's 37 drawn but never on the screen; a Fire Wall's second
  hex's placement drawn at the first hex's place (its vanish at each hex's own); the first frames
  after a projectile show at once (§7.1). For the review list.
- **Checked:** `sequences situations/casts.toml`: all nine as recorded (`obstacles` and
  `creatures` in a second run after a transient XWD dump error at the hot-seat dialog);
  `cast` and `dragon` replayed with the changed harness, so it drives and reads the original as
  before (the whole set is captured afresh at T9); `h3drive.py drawn` 68 of 68 the same; the
  viewer's `scenario` and `situation` tests, `cargo fmt`, clippy; `scripts/exe.py check` and
  `cited`; the client gates' full check (`verify-once clients`, a later session: all gates, code
  `3972a75`).

### M3 T8 — The battle's end tune (2026-10-21)

On `client/e17-m3-t8` (branched from `client/e17-m3-t7a-2b`; #355 merged meanwhile, `master`
merged in), the user present at the start. The user's computer was linked for the staging only (all files staged once, the session
then ran in the cloud).

- **Starting point:** T7a-2b's branch, the client gates' full check on its code (`verify-once
  clients`: all gates); its pull request #355 opened in the same session.
- **The reading** (design §8.2, "The battle's end", "A sound played by its name"; 8 map entries,
  `battle_end` extended, the manager's +0x132F8 in `structs.toml`): `battle_winner` decides the
  battle once an action's last frame showed and calls `battle_end`, which darkens the screen,
  builds the result's window (0x4702D0) and plays its tune once with the effects ended, no clock
  step between. The window shows the winner's side when its player is a human here, else the
  other (a draw: the defender's); the tune: `win battle` or `defend castle` (a town's defender)
  for the winner; `losecastle`, `surrender battle`, `retreat battle` or `losecombat` for the
  loser. The start sound's volume: its loader's default, 127 (the WAV's constructor 0x566DA0),
  as the tent's heal.
- **Recorded** (`sequences.toml`, `end`, `seq-end.toml`: `strike`'s battle without the
  defender's Goblins, the lone Pikemen at hex 100): the harness's `<ms> end` (h3mem's `decided`,
  the manager's +0x132F8) at 3 730 ms, when the next turn would start; with `--screens` the
  whole screen changes in that step (the darkening and the window), then nothing. Two runs the
  same. The viewer's timeline (`Timeline::ended`, `lines`' `end`): 57 changes, as recorded.
- **The viewer:** `h3_view::sound::{Tune, end_tune, FILE_VOLUME}` (the choice, the six names,
  the ranked game's outcomes, which no battle's event carries, as their winner or none); the
  window: `Battle::end_tune` (the snapshot's `human`, no town: *provisional*), `Library` decodes
  the tune, `Player::end` (every effect and the start sound's stream ended, the tune on the
  music's stream once at 127, no fade-in), Home brings the music back; the start sound's and
  the heal's stream at 127.
- **Checked:** the window on `end`'s record under Xvfb with the `disk` driver (`--check 20`,
  158 of 158 frames read back exact): the tune alone in the mix (correlation 1.0 with
  `Win Battle.mp3` decoded by ffmpeg), whole and once, then silence; 3 692 ms after the music's
  start against the cue's 3 730 (the disk driver's pacing, as the effects' 10–100 ms at M2 T5b).
  `frames --only end`: 53 frames, as `strike`'s but at the end, 2 730 its 134 pixels of the
  original's redraws (design §4.3), 3 730 the result's window over the screen darkened (every
  colour component halved, 0x44E5F0: 478 876 pixels; the window is not drawn yet, QUEUE's
  inbox). `drawn`: 68 of 68 the same, `end` new, recorded; `sequences --only strike`, `walk`
  and `end` as recorded with the changed harness. The viewer's tests, `cargo fmt`, clippy;
  `scripts/exe.py check` and `cited`, `check-exe-map`; on the code with `master` merged, `drawn`
  69 of 69 the same and the client gates' full check (all gates, code `eb970ce`).

### M3 T7b-1 — A hero's cast on one stack (2026-10-21)

On `client/e17-m3-t7b-1`, the user present at the start: T7b split with the user (T7b-1 a hero's
cast on one stack, checked against `cast`, `resist` and `arrow`; morale, luck, Haste and Slow's
walk, the count box's tint, the strike-lengthening animations T7b-2). The user's computer was
linked for the staging only (all files staged once, the session then ran in the cloud).

- **Starting point:** `master` at `d77815a` (#356 merged); no client pull request open; the
  client gates' full check on that code (`verify-once clients`: all gates); `drawn` 69 of 69 the
  same.
- **The reading** (design §7.16, "The cast's log lines", §8.2; 2 map entries): the cast's line
  (`battle_log_cast` 0x5A8950: GENRLTXT 196 with the hero, the spell and the target by its
  number, 197 without a target), a resisted one's (190, the verb 145 or 144), Magic Arrow's and
  Ice Bolt's (`battle_log_damage` named by the spell); the projectile's loop (n positions, not
  n + 1; the DEF by the first midpoint below the angle); the cast point (`hero_battle_defs` +4,
  +8: (x − 43, y − 19) left, (693 + W − x, y − 19) right); place 3 (the standing frame's +0x38
  and +0x30); a spell's sound through `play_sound_async` 0x59A590 (category 3, 127).
- **Harvested** (`h3-view harvest`, the three tables regenerated; every earlier line as it was):
  `[[spell]]` per pack spell (its number by `sptraits.txt`'s name, the record's sound and
  animation, Magic Arrow's and Ice Bolt's five projectile DEFs and angles), `[[hero]]`'s
  `original`, `cast`, `cast_frames`, `[battle]`'s `resist_sound`. The catalogue: the families
  `spell` and `hero` (single forms, `column`; `by` counts the records whose second column is
  filled), `battle.log.cast-on`, `battle.log.resisted`, `word.resist`.
- **The viewer:** `anim` plays `SpellCast` with its effects (`Player::cast`: the hero's frames as
  `HeroKey`s, `over_one` for player 1, `spell_projectile` and `marked` riding `iterate`'s loop for
  players 7 and 2, `Logged::{Cast, Resisted, SpellDamage}`, `Sound::{Spell, Resisted}`);
  `Timeline::lines_with_heroes`; `Heroes::drawn_at` and the cast frames between the idle steps;
  `Flight::cycle`; place 3 in `magic_at`; the outline cleared at a cast's click, back as the hero
  stands; `Panels::with_casts` (the book disabled from the click); the window's `Library` loads
  the battle's own sounds by cue (`Battle::battle_sounds`), a spell's at 127 on a creature
  channel, `MagicRes` on the start sound's stream. `h3-view sequences`: `heroes` compared, a
  `seed` per sequence, the recorded `screen` lines left out; `pointer::inputs`: a first `turn:`
  naming the opening's active stack acts at once, a `key:` right after a cast acts at the cast's
  end (the runbook's rule; before it, `arrow`'s round 2 posted the mouse over the Archangel as the
  Pikemen's turn started: "Attack" forced over the log), `cast:` aims the record's next cast
  (`Input::Aim`: the status line's "Cast %s on %s", GENRLTXT 28, not forced, *provisional*).
- **Checked:** `cast`, `resist` (seed 3) and `arrow` moved into `sequences.toml`: their timelines
  as recorded on the first try (35, 62 and 140 changes); every other sequence as recorded.
  Captured (`frames --only`, count boxes compared): `cast` 36 frames, all exact but 0 (71
  pixels: the original's one-frame restore over a rock's top at the click) and the count box from
  600 (238 pixels: Slow's tint, T7b-2); `resist` 61 frames, all exact but the count box while
  Slow lasts (197 to 228 pixels); `arrow` 168 frames, all exact (the projectile's DEFs, frames and
  steps, the marked loop, the lines). Before the fixes the frame check found the outline (cleared
  at the click), the spell book (disabled), the hero's frames (memory a deadline ahead), the
  aiming status line (`cast` 0 to 400) and the key after a cast (`arrow`'s log item in round 2).
  `drawn --record`: 72, the three new. The window on `resist`'s
  record with the `disk` driver: 131 of 131 frames read back exact; `Muckmire` and `MagicRes`
  once each in the mix (correlation 0.62 and 0.47 over the music), 2 879 ms apart against 2 900.
  `play` on the three records: the cues' sounds named. The viewer's tests (100 % of the library),
  `cargo fmt`, clippy; `scripts/exe.py check` and `cited`.

### M3 T7b-2a — The count box's tint, Haste and Slow's walk, Dispel, Cure, Disrupting Ray (2026-10-21)

On `client/e17-m3-t7b-2a`, the user present at the start: T7b-2 split with the user (T7b-2a the
parts without the original's rolls; morale, luck and the creatures' rolled animations T7b-2b).
The user's computer was linked for the staging only.

- **Starting point:** `master` at `a6b6c55` (#361 merged); no client pull request open; the
  client gates on that code verified once (`verify-once clients`: the light gates).
- **The reading** (design §4.3, §7.16 "Read at T7b-2a"; one map entry, `count_box_spell_hue`
  0x43E6AA): the box's hue r = Σ k / Σ |k| over the stack's active spells, k the spell record's +0
  (polarity), at the x87's 53 bits; Dispel's, Cure's and Disrupting Ray's handlers (the order of
  animation, effect and line; Disrupting Ray's projectile, its defense taken, GENRLTXT 92); Haste
  and Slow's W (0x444CE5, 0x444D3A: integer W0 × 65 / 100, W0 × 3 / 2 exactly).
- **Harvested:** `[[spell]]`'s `sign` (the record's +0) and Disrupting Ray's one-DEF projectile;
  every earlier line as it was. The catalogue: `battle.log.disrupted`.
- **The viewer:** `scene::spell_hue` (exact in integers; checked against the executable's
  arithmetic run in floats for all 6 722 sums of up to 81 signs), `Moment::hues`, the box's tint
  by them; `anim`'s `TintKey`, `Timeline::hues`, a stack's active spells from the snapshot and the
  effects at the handler's point, `walk_time`, Dispel, Cure and Disrupting Ray in `Player::cast`
  (`SpellLook::original`, `sign`, `value`), a stack's defense lowered by Disrupting Ray
  (`Logged::Disrupted`; `Logged::Defend` carries the defense); `motion::casts` compiles the
  record's ruleset (`--packs`) for a spell's value at the caster's mastery.
- **Recorded** (`casts.toml`'s way, `h3drive.py sequences`; hot seat, Pikemen against Vampire
  Lords): `haste` (175 changes with the screens, about 9 minutes) and `dispel` (289, about 11).
  Both moved into `sequences.toml` with their commands: their timelines as recorded on the first
  run (84 and 136 changes); every other sequence as recorded.
- **Checked:** captured (`frames --only`, count boxes compared): `cast` 36 frames, all exact but
  0 (its 71 pixels, listed at T7b-1), the box's tint from 600 exact; `resist` 61 frames, all
  exact (the tint's end at 2 900 found by the frame check: a round's start ends Slow and the next
  turn's start redraws the box, so the hues a round's start changes show at the turn's start);
  `haste` 91 frames and `dispel` 153 frames, all exact (both heroes' casts, the walks, the
  projectile and animation 14 from its third frame, Disrupting Ray's line, the Vampire Lords'
  defend gaining 1 of their 7 since, Cure's and Dispel's lines after their animations, the
  tints). `drawn`: the four listed, then `drawn --record` (74). The viewer's tests, the full
  `clients/scripts/check.sh` (all gates, the library at 100 %); `scripts/exe.py check` and
  `cited`.

### M3 T7b-2b-1 — The harness forces the original's rolls (2026-10-21)

On `client/e17-m3-t7b-2b-1`, the user present at the start: T7b-2b split with the user, who asked
first for a way to reproduce the rolled effects reliably (the generator's state in the running
game's memory, altered: feasible?), then morale, luck and fear (fear added at the user's word,
T7b-2b-2) and the creatures' rolled animations (T7b-2b-3). The user's computer was linked for the
staging only (24 files, every hash the user's).

- **Starting point:** `master` at `8519f7a` (#363 merged); no client pull request open; the
  client gates' full check on that code (`verify-once clients`: all gates).
- **The reading** (`docs/exe/map.toml`, nine `*_draw` sites): every roll T7b-2b needs is one call
  of `random_between` 0x50B230 (`a + rand() mod (b − a + 1)`, fastcall) or of the C library's
  `rand` 0x617E5C: good morale `random(1, 24) <= morale` (0x4649D3), bad morale
  `random(1, 12) <= -morale` (0x464BC7; a computer side's let-off draw 0x464BF0 after it), fear
  `rand() mod 10 = 0` (0x464DF1), luck `random(1, 24) <= luck` as a melee strike starts
  (0x441837) and for a shot (0x43F955), Death Stare `random(1, 100) <= 10` per Mighty Gorgon
  (0x440EE0), the Thunderbird's bolt and the Rust Dragon's acid `<= 20` (0x44119D, 0x4414B7),
  Poison `<= 30` (0x440839). Acid Breath's lowered defense and Life Drain draw nothing. Both
  morale draws are made for every stack asked, whatever its morale.
- **Why not the generator's state:** `rand` keeps its seed per thread (+0x14 of the C library's
  thread data), and the draws before a roll vary (one per creature of a damage roll, the fidget
  clocks after every action, the cosmetic draws of the idle step), so no state written at a step
  aims at one roll. Redirecting the roll's call does, and leaves the sequence of draws as it was.
- **The harness** (the runbook's "Forced rolls"): `xdd.dll` redirects the nine calls at its load
  (each only if its bytes are the call named) to `forced_between` or `forced_rand`, which draw
  as the call would and return a forced draw where the harness asks (`ctl`: `roll <name> <value>
  <count>`, `rolls <name> <value>|natural`; `ack` adds the draws forced, those pending, the calls
  redirected). `h3drive.py`: `ROLLS` (each roll's falling and failing draw), the action
  `roll:<name>[:fail]`, a sequence's `fail` (`--fail`); a sequence fails when fewer calls were
  redirected or a forced draw stayed untaken. `sequences` drops a recorded timeline's `screen`
  lines for an entry that does not record them (`resist` was `CHANGED` for them alone). The
  viewer's mouse takes `roll:` as no input (`pointer::inputs`).
- **Checked:** in the original, the Champions' walk (`seq-walk.toml`, morale 1 by one town) with
  `roll:good_morale` and `fail = good_morale`: active again at 2 592 ms, animation 20 over them
  at 100 ms a frame; with `fail` alone the Marksmen's turn at 2 592 ms. An Azure Dragon against
  them (an ad-hoc scenario): `roll:fear` played animation 15 over the Champions at 5 400 ms and
  skipped their turn, failed they took it. Every run's `ack`: 9 calls redirected, none pending.
  `walk`, `strike`, `shot` and `resist` replayed with the redirected calls: all as recorded.
  `scripts/exe.py check` and `cited`; the client gates (`clients/scripts/check.sh`).

### M3 T7b-2b-2a — A hero's luck and morale on the original's map (2026-10-21)

On `client/e17-m3-t7b-2b-2a`, the user present: after T7b-2b-1 the user asked to make luck
forceable as well (the hero given the skill or an artifact); T7b-2b-2 split so (T7b-2b-2a this,
T7b-2b-2b the morale, luck and fear animations and recordings). The sandbox and its staged files
were T7b-2b-1's.

- **Starting point:** `master` at `7812798` (#366 merged); no client pull request open.
- **The route:** a scenario's hero already carries its derived morale and luck as battle
  `effects` (`{ kind = "luck", amount = 3 }`, the engine's BAT-HER-003); only the original's map
  lacked them. The packs record each secondary skill level's effects (Leadership `morale` 1 to 3,
  Luck `luck` 1 to 3), so the writer gives the hero the skill level whose effects are the change
  alone, data-driven; artifacts not needed (a skill gives exactly 1 to 3, one source).
- **The viewer:** `scenario::effect_skills` (an effect no level gives alone, another kind, or two
  of one skill: refused), `SideHero::skills`, written after the book's school skills
  (`situation`); the binary depends on `h3-content` and `h3-contracts` (binary-only in
  `purity.rs`; `clients/Cargo.lock`). `h3mem`: a stack's `morale` (+0x4E8) and `luck` (+0x4EC).
- **Checked:** `seq-shot.toml` with Orrin's `effects` luck 3 and morale 2 (an ad-hoc copy): the
  original's Marksmen and Pikemen at luck 3 and morale 3 (Expert Luck; Advanced Leadership and one
  town), the defenders 0; the engine's record the same. `roll:shot_luck` with `fail` on
  `shot_luck` and `good_morale`: magic 18 over the Marksmen from 0 ms, 100 ms a frame, the log
  "Good luck shines on the Marksmen"; failed, the timeline was the recorded `shot`'s line for
  line. The binary's tests (33, two new), the client gates (`clients/scripts/check.sh`).

### M3 T7b-2b-2b — Morale, luck and fear (2026-10-21)

On `client/e17-m3-t7b-2b-2b`, the user present at the start: offered a split (good luck and good
morale, then bad morale and Fear), the user chose all four in one go. The user's computer was
linked for the staging only (the four LODs, both sound archives, `H3ab_ahd.vid`, the executables,
the five DLLs, the combat and end tunes, `maps.tar`, `extracted.tar`).

- **Starting point:** `master` at `c970a58` (#368 merged); no client pull request open; CI on
  `master` not started (billing); the client gates' full check on that code (`verify-once
  clients`: all gates passed).
- **The reading** (design §7.16, "Read at T7b-2b-2b"; `docs/exe/map.toml`'s three functions):
  good morale's sound, animation 20 without the hit, line, sound wait and redraw; bad morale's
  and Fear's sound, line, animation with the hit and sound wait, inside the pick before the
  active stack is set; good luck's sound (category 3), line and animation 18 before the strike's
  or shot's; `play_magic_animation`'s `show_hit` read whole; Fear's line GENRLTXT 729.
- **The viewer:** `Player::over_one` with the hit, `luck`, `good_morale`, `frozen`; a frozen
  stack's `TurnStarted` neither active nor logged; `Sound::GoodMorale`, `BadMorale`, `Fear`,
  `Luck`; `Logged::Fear` and `battle.log.fear`; the asset tables' `[battle]` sound names
  (harvested: `ROLL_SOUNDS`); `motion::magic` loads 15, 18, 20, 30 when the record needs them,
  `motion::casts` the frightening creatures from the ruleset (`Casts::frighteners`).
- **Recorded** (`sequences.toml`, forced rolls with `fail`, hot seat but `morale`): `luck`
  (`seq-luck.toml`, 73 changes), `morale` (`seq-morale.toml`, 69), `bad` (`seq-bad.toml`, 42;
  three towns, morale −1; the Dwarves, speed 4 on their native grass, win the tie against the
  Gnolls and act next) and `fear` (`seq-fear.toml`, 52), each recorded once, about 3 to 5
  minutes. The engine's seeds searched (a loop over `h3-view sequences <id>` reading the
  record's events): `luck` 7, `morale` 73, `bad` 9, `fear` 7. All 21 sequences as recorded
  (the original's own fidgets left out).
- **Found by the frame check** (first run: `luck` as `strike`'s, `morale` 36 frames off, `bad` 35,
  `fear` 31): the outline. The original outlines manager +0x132C8's stack, which the move's,
  strike's and shot's handlers clear while they play and set again at their end, a Defend leaves,
  a round's start clears and a turn's start sets: the stack that acted last shows outlined through
  good morale and through a freeze, and a good luck's first frame takes it away. Built
  (`Player::acting`, the command's first frame may be a magic animation's); `bad` also showed the
  panel's turns paired by index with the timeline's (`panel::turns` now skips a frozen turn,
  `anim::freezes`).
- **Checked:** `frames --only` for the four (`--reuse` after the fix): `luck` 75 frames as
  recorded (as `strike`'s to the landing), `morale` 94, `bad` 36 and `fear` 46 exact (`morale`'s
  box and log's line listed from the shot: the engine's lucky shot kills 4 Goblins, the
  original's 5); `drawn` (78, only the four new), then `drawn --record`. The viewer's tests (186
  in the library), `scripts/exe.py check` and `cited`, the full `clients/scripts/check.sh`:
  CHECK_RESULT.

## Accepted review lists (M1, M2)

Moved from the epic file on 2026-10-20 (less start-up reading), unchanged.

### M1 review list

**Accepted by the user, 2026-10-13 (T8), every item as listed;** the *provisional* marks of these
choices are lifted.

- `proptest` as a dev-dependency of `h3-assets` (the engine's 1.11.0; T2).
- `h3_assets::build`, encoders for synthetic assets, as public API of `h3-assets` (T2, T3).
- The decoders' strictness: exact lengths, an overrunning run is an error (no original file has
  one; `E17-design.md` §1; T2, T3); a DEF frame holds at most 2^20 pixels (T8).
- `miniz_oxide` (already in the lockfile through `h3-formats`; the engine's version and features)
  as a dependency of `h3-render` for the PNG encoder, instead of the `png` crate (T6).
- `h3_assets::def::Frame::runs`, a public field: which pixels came from runs, which the drawers
  need; the synthetic encoder follows it (T6).
- A PNG widens each 16-bit channel by repeating its bits; comparisons stay in 565 (T6).
- The asset tables' format (`clients/README.md`, "The asset tables"; T4): keys qualified at load;
  `cranim.txt`'s times as the executable's scaled integers, angles in degrees; the special
  battlefields keyed by VCMI's names and the 47 obstacle types no pack defines kept in
  `base.toml`; only the three DEF groups read so far in `[battle]`; the fix entry's fields (its
  flag and the `faithful`/`fixed` profile of ADR-0020 point 6 come with the first fix).
- `toml` 1.1 (without std; the engine's 0.8 needs std) and `serde` (the engine's, without std) as
  dependencies of `h3-view` for the pure loader (T4).
- The harvester keys creatures by their `crtraits.txt` singular name, as `h3-harvest` does (T4).
- `h3-protocol` as a dependency of `h3-view`'s library (ADR-0020 point 1), and `h3-pack-battle`,
  `h3-battle`, `h3-kernel` (and, since T8, `h3-formats`) as dependencies of its binary only (the scenario adapter; checked by
  `tests/purity.rs`), rather than `h3-cli`'s library, which builds the adventure crates too; the
  client job now watches those crates (#152) (T7).
- The layouts' format and place: `clients/layouts/<screen>.toml`, one file per screen and
  resolution, apart from the asset tables (T7); an HD layout will need the battlefield's origin and
  viewport (not in the format yet).
- The record parser rejects a line over 16 MiB (the adapter reads a local file whole; a live
  connection bounds each message, M2); the record format and protocol version must be the
  client's own (T7).
- ~~A natural obstacle is drawn until a removal clears every hex it blocks~~: dropped in T8 (the
  reviewer: the engine drops a removed obstacle from the list; every listed one is drawn).
- `harvest` writes to `clients/out/harvest` by default, never over the committed tables (T8).

### M2 review list

**Accepted by the user, 2026-10-14 (T5c), every item as listed.** The *provisional* marks stay
where a recording is to check the rule (the flier's layer 8, the shot's loop: M3 T1b) or where the
original cannot be matched (the fidgets' generator); the others are lifted.

- The capture tooling (T8): Wine, Xvfb, xdotool, ImageMagick and mingw-w64 as sandbox tools;
  Python with numpy for `h3cap.py` (as `scripts/exe.py` uses Python); our own `xdd.dll` forwarder.
- The virtual clock in `xdd.dll` (T1): it rewrites the running game's import slot of
  `timeGetTime` in memory (no byte of the executable changes), driven through two files.
- `h3-map` and `h3-content-battle` as dependencies of `h3-view`'s binary (T1: `h3-view situation`
  writes maps with `h3-map`'s writer; the scenario adapter names the ruleset's type); with them
  `h3-adventure` and `h3-game` enter the client's build and its CI paths (#161).
- The probes' file `capture/screens.toml` (positions and 5-6-5 values of the original's screens:
  numbers only, ADR-0009) and the set's format `situations/set.toml` (T1).
- The diff and the obstacle finder stay in Python, not `h3-view diff` (T1, design §6.1).
- The scene's background: a terrain's third picture, an empty surrounding's (T1; CLT-Q4).
- Reading the running original's memory (`capture/h3mem.py`, read only, under Wine) and the
  recorded sequences' format (`situations/sequences.toml`, `timelines/`: numbers only) (T3).
- `walk_step` (was `stack_layer_by_direction`) and `strike_animation` (was `cast_pending_spells`)
  renamed in the address map after reading them whole (T3).
- The viewer's fidgets from a generator of its own (the original draws them from its rules'
  generator, design §7.2); reference frames compared against the original's poses (T3, for T4).
- The asset tables' `groups` with every battle group the animations use, and the sequences'
  engine commands (`[[sequence.command]]`: a seat and a wire command) (T4a).
- The timeline's *provisional* choices (design §7.8): the next command's default start, at
  least A + 1 iterations, the strike's group by rows, the flight's cycles' rounding, a dying stack
  among the corpses (T4a; the shooter's frames and the flight's points corrected and the walk's
  offsets confirmed by T4b-1's frame check).
- The frame check (T4b-1, design §7.9): the viewer's own timeline drawn against the original's
  screen (memory only for fidgets); the counts per frame as the recorded format
  (`timelines/<id>-frames.txt`); the frames kept as compressed numpy arrays, local only; a command
  without a time after the first counting as the computer's (its idle step and outline); f32
  arithmetic emulated in fixed point and tangents by series in the pure crate; the layout's
  `outline_ramp`.
- `h3-view play` (T4b-2a): its bot (`bin/h3-view/bot.rs`: shoots, else strikes, else walks
  toward the nearest enemy; the engine plays, the session replays the commands), its output
  (`frames.txt`, one line per distinct frame; `f<ms>.png` with `--png`) and
  `scenarios/battle.toml`.
- The turns' and layers' model (T4b-2a, design §7.10): `Pose` carries the facing and the draw
  layer; the asset tables' `turn_left` and `turn_right` groups; the struck stack's layer 5 from
  the harness's reading (its writes not found), a shot's layers left at 4; a layer change without
  a frame of its own shows with the next frame that does not draw that stack; a dying stack in
  its layer's pass until the next frame, then among the corpses.
- The fidgets (T4b-2a): the C library's `rand()` sequence seeded by the battle's seed, only for
  stacks whose DEF has group 1, the stall draw also at a fidget's last step; the reference checks
  without them.
- A later command without a time starts as the previous one's last frame shows (T4b-2a: the
  flight's second standing frame, 0x4B4A15, replaced T4a's extra tick).
- The harness's adventure map (T6): `Show Route` on in `setup.sh`; the recipes' format
  (`capture/batches/*.toml`: steps, `record`, `expect`) and the filled template with its note on
  how it was played; every record read from the game's memory, not the screen; the adventure
  map's reads from H3API's layouts confirmed by observation only (`inferred`); a dialog's OK
  found by relative probes (`[button.ok]`); a walk over after 1.5 s at rest, in real time.
- The window adapter's library (T2, the user's choice; ADR-0024, accepted): SDL 3
  through `sdl3-sys` (not the `sdl3` wrapper), built from source and linked statically, the unused
  subsystems left out; its defaults vsync off and the hardware cursor (the user, *provisional*);
  `nanomp3` for MP3; the IMA ADPCM decoder in `h3-assets`.
- The spikes stay on the branch `client/e17-m2-t2-spikes`, never merged (T2); their reel, cursor and
  sound files are made from the user's files into `original-data/spikes/` (gitignored).
- The platform adapter (T5a): one crate `h3-sdl` for the window, its input and the cursor and,
  with T5b, the audio (named for what it holds, not `h3-window`); left out of the coverage gate
  as an adapter (`check.sh`), exercised by `h3-view window --check`; SDL's joystick and HIDAPI
  left out of its build too (no input but the keyboard and the mouse).
- `h3-view window` (T5a, design §8.1): its keys (Space, →, R, F, V, C, Esc) and options; the
  window sized to the display's usable area less 40 points; the frame widened on the CPU; the
  next frame composed as soon as the one before shows; a battle timed 60 s at a time past its
  end, waiting for input only after 10 minutes without a change; `--check` and `--measure`.
- The cursor (T5a): `CRDEFLT.DEF`'s first frame, its hot spot top-left, drawn with the battle's
  drawer without an outline; for the hardware cursor drawn over black and white (shadow pixels
  as black with alpha; this cursor has none). The executable's cursor code is M3's.
- A flier's cycles in layer 8, drawn after every row (T5a, design §7.10): from the code reading
  only; a recorded flight across rows (T4b-2b) checks it against the original's frames
  (confirmed in M3 T1b-2, sequence `fly`).
- `h3_view::clock` and `Screen::xrgb` in the pure crates (T5a); `play` and `window` share
  `bin/h3-view/battle.rs`; an executable copied into a checkout's `clients/` finds the tables,
  layouts and packs from its own place (`assets::clients_dir`).
- The decoder's length (T5b, design §8.2): every complete frame up to the `fact` chunk's count, as
  SDL with `dropframe` and `truncate` (SDL's defaults drop a cut last block); the viewer searches
  `Heroes3.snd` before `H3ab_ahd.snd` (their common entries are equal; the game's order not read).
- The asset tables' `sounds` (T5b): each creature's sound entries by name, written by `h3-view
  harvest` with `stack_load_graphics`' rules; the gate finds every one.
- The cues (T5b): `anim::Sound` and `anim::Cue` in the timeline, at the frame each reading names;
  `play` writes them to `cues.txt`.
- The opening (T5b): the start sound and the track drawn from the viewer's own sequence seeded
  with the battle's seed (as the fidgets); the start sound's volume 64 (not read); no volume
  settings yet (the music 127, the effects 64, as at equal settings); a track always from its
  start (the original resumes a track where it last stopped in the session).
- The window's sound (T5b): ten SDL streams (the music, the start sound, eight creature channels
  taken as `play_sample` takes them); loops and the music queued a second ahead without a flush;
  Space pauses the audio device with the battle, R and → stop the creatures' sounds; the keys M
  and E, `--mute`; the clock waits for the start sound only from the battle's start, and a key
  that moves the battle ends the wait; volumes as SDL gains (the adapter's only float).
- `nanomp3` with its default features off (`std` only: no SIMD `wide`, no MP1/MP2; T5b).
- A shot's loop (T5b, the user's find, design §7.4): the reactions start as the projectile lands;
  the shooter's frames left beyond the loop are cut, the loop one iteration longer than the
  reactions' frames (*provisional* until a recorded Cyclops shot, T4b-2b).
