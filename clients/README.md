# clients/ — playable clients (client and map-client lanes)

**Two lanes** (since 2026-10-23, `lanes.toml`): the **client** lane builds the battle client (E17,
`h3-view`, the battle's tables, layouts, scenarios and situations), the **map-client** lane the
adventure map client (E24, its crates `crates/h3-map…` and its folder `map/`). The foundation,
`h3-assets`, `h3-render`, `h3-sdl`, this workspace's `Cargo.toml`, `deny.toml`, `.gitignore`,
this file, `scripts/` and the string catalogue `text/`, and the capture harness `capture/` are
common: either lane changes them in its own pull request, an additive change freely, a change in
behaviour noted in the other lane's inbox (`docs/08-work-process.md`, "Lanes").

Everything with graphics, sound, input or a window lives here, outside the engine core. The engine
core (`crates/`) stays headless, deterministic and free of assets; clients use it through its
public crates and, later, through the server's protocol.

**State:** E17 M0 built (2026-10-12): the workspace, its gates (`scripts/check.sh`, the CI job
`.github/workflows/clients.yml`) and the real-asset gate. M1 T2 + T3 (2026-10-12): the decoders
for palettes, PCX, DEF and MSK (`h3-assets`); the gate decodes every one of them in the user's
archives (findings: `docs/epics/E17-design.md`). M1 T1 + T6 (2026-10-12): the code reading of
the executable's drawing (design §3) and the compositor (`h3-render`) that follows it; `h3-view
sprite` draws a DEF's group over an image into a PNG. M1 T5 + T4 (2026-10-13): the battle
screen's reading (design §4) and the asset tables (`clients/assets/`, below). M1 T7 (2026-10-13):
`h3-view render`, a battle still from a record or a scenario file. M1 T8 (2026-10-13): reference
captures of the original, run under Wine in a sandbox, and the first exact diff (below). M2 T1,
T3 (2026-10-13): the capture harness and the animation timing. M2 T4a (2026-10-13): the scene
over time (`anim`), checked against the recorded sequences, and `render --at`. M2 T4b-1
(2026-10-13): the frame check, every frame of the recorded sequences against the original's. M2
T2 (2026-10-14): the window, input and audio library chosen by measurement and the user: SDL 3
(ADR-0024, `docs/epics/E17-design.md` §8). M2 T5a (2026-10-14): the SDL 3 adapter `h3-sdl` and
`h3-view window`, a battle in a window at the original's frame times (M2's exit). M2 T5b
(2026-10-14): the sound archives decoded (`h3-assets`), the battle's music and creature sounds in
the window at the times the executable plays them (design §8.2). M2 T5c (2026-10-14): M2 closed
after a fresh review; M3 (fidelity) refined. M3 T1a, T1c (2026-10-14/15): a battle in progress in
hot seat, the redraw artifacts classified. M3 T1b-1 (2026-10-15): the Arch Magi's and Beholders'
energy ray (`h3_view::ray`, design §7.12). M3 T3a, T3b (2026-10-15): the heroes and their flags;
the war machines and the First Aid Tent's heal under its magic animation (design §7.14, §7.15).
Decisions: ADR-0020
and the epic file `docs/epics/E17-clients.md`; tasks in `docs/lanes/client/QUEUE.md`.

## Layout

A separate Cargo workspace with its own `Cargo.toml` and `Cargo.lock`, excluded from the engine
workspace (root `Cargo.toml`: `exclude = ["clients"]`) and with its own CI job. The engine's lints
are copied into `clients/Cargo.toml`; the engine's `clippy.toml`, `rustfmt.toml`,
`rust-toolchain.toml` and `.cargo/config.toml` at the repository root apply as they are (the tools
search the parent directories).

- **The gates:** `clients/scripts/check.sh` runs fmt, clippy, the tests, `cargo deny`, 100 %
  coverage of the pure crates and the CI path filter's check; the CI job runs the same script, and
  it is the full local check while CI cannot run (`docs/05-testing.md`, "CI"). It needs
  `cargo-deny`, `cargo-llvm-cov` and the LLVM tools.
- **The real-asset gate:** `cargo run --release -p h3-view -- gate [--install <dir>] [--out <dir>]`
  (from `clients/`) opens the four LODs of the installation copy (default: `$H3_INSTALL`, then the
  repository's `original-data/install`), decodes every PCX, PAL, DEF and MSK (it fails if one does
  not), prints the entries by extension and the times, and writes every entry with its verdict to
  `clients/out/gate/entries.tsv` and the inventory (findings, duplicates, names nothing mentions)
  to `clients/out/gate/report.md`; then it checks that every name in the asset tables
  (`--tables <dir>`, default `clients/assets`) is found where the game searches it and decodes.
  Then the sound archives (`Heroes3.snd`, `H3ab_ahd.snd`; skipped if absent): every WAV decodes
  with `h3_assets::wav` to SDL's own loader's samples (`clients/out/gate/sounds.tsv`), and every
  sound the tables name is one of them. Then the string catalogue (`--keys <file>`, default
  `clients/text/keys.toml`): every key's original forms are read from the text tables where the
  game searches them, each row's `%s` and `%d` as the key names them, and the battle log's
  sample lines compose from them. A debug build takes about ten times as long.
- **A look at the drawers:** `cargo run --release -p h3-view -- sprite CPKMAN.DEF --group 2
  --background CmBkGrMt.pcx [--mirror]` draws every frame of one group of a DEF side by side over
  a PCX, as the battle draws a stack, into `clients/out/sprite/` (the base archives searched
  first, as the game does).
- **A battle still:** `cargo run --release -p h3-view -- render scenarios/one-hex.toml [--seed
  <n>] [--list]` (from `clients/`) runs the scenario file (as `h3 run` reads it, packs from the
  repository's `packs/`) in-process in `h3-protocol`'s session, joins it as a spectator and writes
  that record (`clients/out/render/<stem>-s<seed>.jsonl`); a record file (`.jsonl`) is read
  directly. Its first snapshot, the asset tables and `clients/layouts/battle.toml` give the draw
  list (`--list` prints it); the assets load as one checkpoint batch; the PNG goes to
  `clients/out/render/`. Drawn so far: the background (the picture of an empty surrounding,
  CLT-Q4), the obstacles, every living stack standing on its hex, the active stack's outline, the
  heroes and their flags (M3 T3a), the war machines and the heal's magic animation (M3 T3b), the
  count boxes (M3 T4a; hidden through walks and flights, M3 T5a; a strike's kills from its end,
  M3 T5b-1; not yet tinted by spells), a hero's cast on one stack (M3 T7b-1: the hero's casting
  frames, the spell's animation over its target, a resisted cast's, Magic Arrow's and Ice Bolt's
  projectile and hit, their sounds and log lines), good luck, good and bad morale and Fear over
  their stack (M3 T7b-2b-2b: the last two with its hit, their sounds and lines), the
  creatures' effects of a melee strike (M3 T7b-2b-3: Poison and Acid Breath cast at the
  strike's end, Life Drain, Death Stare, the Thunderbird's bolt, the Rust Dragon's acid), a
  hero's Lightning Bolt, the mass and area spells (M3 T7c-1: a mass timed spell, Death Ripple,
  Destroy Undead, the area spells' animation at their hex and reactions, a creature's cast such
  as the Faerie Dragon's), the spell objects and Armageddon (M3 T7c-2: Quicksand's pits, Land
  Mines, a Force Field and a Fire Wall placed by their animations at a hex, drawn in the
  obstacles' passes and gone with their vanish animations at a round's start; Armageddon tiled
  over the battlefield with every stack's reaction), raised stacks, the creatures' own animations
  and Chain Lightning (M3 T7c-3: Resurrection's animation over a stack raised, a lying one's
  death group backwards; Fire Shield's return; the Magogs' and the Liches' explosions; Chain
  Lightning's rays from stack to stack, the original's draws in the frame check), the
  panel (M3 T6a: its picture in the acting player's colours and its buttons as the turn's start
  sets them, `h3_view::panel`; since M3 T6b through the battle's turns and with its log's lines,
  `scene::Panels`); not yet
  corpses or the grid. With `--at <ms>` (and
  `--speed <0-2>`) the record's commands are played at the original's timing first and each
  stack is drawn in its pose then (`h3_view::anim`, E17 M2 T4a); `--frames <dir>` draws so each
  time a capture's `frames.toml` lists, with the original's fidgets (the capture set, M3 T1b-2)
  and its drawing bugs (M3 T4a: the client draws them fixed, the comparisons as the original).
- **The timing check:** `cargo run --release -p h3-view -- sequences [<id>]` (from `clients/`)
  plays each recorded sequence of `situations/sequences.toml` with its engine commands, times it
  with `h3_view::anim` (the original's drawing bugs in the timeline too, M3 T7d:
  `anim::Stage::bugs`; the window, `render` and `play` draw them fixed) and compares the timeline with the one the harness recorded in the
  original (`docs/epics/E17-design.md` §7.8); it exits 1 on a difference and writes the records
  and timelines to `clients/out/sequences/`. The unit tests replay the same sequences from their
  numbers, without the original files. A sequence may be a battle in progress, both sides played
  in hot seat (`situation --hot-seat`; E17 M3 T1a, the runbook's "A battle in progress").
- **A whole battle, frame by frame** (E17 M2 T4b-2a, design §7.10): `cargo run --release -p h3-view
  -- play scenarios/battle.toml [--seed <n>] [--png] [--from <ms>] [--to <ms>]` (from `clients/`)
  has a simple bot fight the scenario to its end in-process (or reads a record file), plays its
  commands at the original's timing with the viewer's own fidgets and draws every distinct frame:
  `clients/out/play/<stem>-s<seed>/` holds the record, `frames.txt` (each frame's moment),
  `cues.txt` (the creatures' sounds as they start and stop, T5b) and, with `--png`, `f<ms>.png`;
  `--to` past the end lets the battle idle.
- **A battle in a window** (E17 M2 T5a, ADR-0024, design §8.1): `cargo run --release -p h3-view
  -- window scenarios/battle.toml [--seed <n>] [--scale <n>] [--vsync] [--drawn-cursor]
  [--fullscreen]` (from `clients/`) loads the battle as `play` does and shows each frame when it
  falls due, at the largest integer scale that fits the display; P pauses, → steps a frame
  while paused, Home restarts, F, V and H switch fullscreen, vsync and the cursor's kind, M and E
  the music and the creatures' sounds, Esc quits. The mouse is the battle's (E17 M3 T6c-2,
  design §9.10): its cursor, the stack it marks and enters, its hints in the log; a panel button
  shows its click frame while the mouse or its hotkey (S, R, O, A, C, W, D, Space) holds it, the
  log's scroll buttons and ↑ ↓ scroll the log. The sound (T5b, design §8.2; the installation's
  `Data/*.snd` and `MP3/`): from the battle's start its start sound plays while the clock waits,
  then the battle music loops, and every stack's sounds follow its animations; once the battle
  is decided its end's tune plays once instead (M3 T8); `--mute` plays none (in a sandbox: `SDL_AUDIO_DRIVER=disk` writes the mix to `./sdlaudio.raw`).
  `--measure <s>` runs unattended and prints how late the frames showed; `--check <s>` also reads
  every frame back and compares it with the frame drawn headless (exit 1 on a difference; in a
  sandbox under Xvfb). Building it builds SDL 3 from its sources: CMake, a C compiler and, on
  Linux, the X11 headers (`.github/workflows/clients.yml` lists them; ALSA's and Pulse's for the
  sound). A Windows build cross-compiles (`rustup target add x86_64-pc-windows-gnu`, `mingw-w64`,
  `cargo build --release -p h3-view --target x86_64-pc-windows-gnu`); copied anywhere inside a
  checkout's `clients/`, it finds the tables, layouts and packs from its own place.
- **The frame check** (E17 M2 T4b-1, design §7.9): `python3 capture/h3drive.py frames` (from
  `clients/`, in a sandbox with the harness) keeps every distinct battlefield the original shows
  in each recorded sequence, draws the viewer's frame at the same times (`h3-view sequences
  --frames`) and compares them with masks against the counts in `situations/timelines/`; since
  M3 T6b the whole panel, the log's item where the original's text is the viewer's (design §9.8).
- **Without the original** (2026-10-20): `python3 capture/h3drive.py drawn` draws the viewer's
  frames of every sequence and situation at the times the last capture kept
  (`situations/kept/`) and compares their digests with `kept/drawn.txt`: a task captures in the
  original only what it lists as changed (the runbook, "Which captures a change needs").

- **The map's real-asset gate** (E24 M1 T2): `cargo run --release -p h3-map-view -- gate
  [--install <dir>] [--map <dir>] [--maps <dir>]` (from `clients/`) loads the map's tables and
  the adventure screen's layout (`clients/map/`), finds every name they hold where the game
  searches it in the four LODs and decodes it, and checks that each DEF's group 0 holds the
  frames the view draws, the terrains', rivers' and roads' as many as the highest picture any
  shipped map gives (`<install>/Maps`); it prints each DEF's frames, encodings and run pixels and
  exits 1 on a problem (findings: `docs/epics/E24-design.md` §1.9).

| Path | What |
|---|---|
| `clients/Cargo.toml`, `clients/Cargo.lock`, `clients/deny.toml` | the client workspace (`deny.toml`: the engine's policy, kept identical by `check.sh` until M2) |
| `clients/scripts/check.sh` | the client's gates (local and CI) |
| `clients/crates/h3-assets` | decoders: palettes, PCX, DEF, MSK, the bitmap fonts (`fnt`, M3 T4a), the sound archives and their IMA ADPCM WAVs (done), later video; encoders for synthetic test assets (`build`); the gate's checks (`check`, `survey`); reads LOD archives through `crates/h3-formats` (a path dependency into the engine workspace) |
| `clients/crates/h3-render` | the software compositor: the screen in a 16-bit pixel format, palettes truncated into it, the executable's frame drawers (shadows, outline and flag colours, mirroring, clipping), opaque images, text in the bitmap fonts (`text`: the executable's text drawer) and its tint of a rectangle (`tint`), PNG bytes |
| `clients/crates/h3-view` | the scene model (the library: `record` reads a record, `anim` plays its commands at the original's timing, `scene` makes the draw list of a snapshot at a moment, `batch` holds a checkpoint's decoded assets, `compose` draws the list into a frame; `tables` and `layout` load the asset tables and the layouts; `ui`, the dialog items screens are made of, as reusable components, and `panel`, the battle's panel of them: M3 T6a; `mouse`, the battle's mouse as a still and over time, `mouse::track`: M3 T6c); its adapters (the binary, `src/bin/h3-view/`): the real-asset gate, the tables' harvester, `sprite`, the headless renderer `render` with its scenario adapter, the timing check `sequences`, `play` (a battle fought by its bot and drawn frame by frame), `situation` (a scenario's battle as a map for the original), `window` (the battle in a window, through `h3-sdl`; with `play` it loads the battle through `battle.rs`). The library also has `cursor` (the game's mouse cursor), `clock` (the battle's clock in the window) and `sound` (the opening's start sound and music, the end's tune). Its package depends on `h3-protocol` (the library) and, for the adapters only, on `h3-pack-battle`, `h3-battle`, `h3-content-battle`, `h3-kernel`, `h3-map`, `h3-sdl` and `nanomp3` (the music), which `tests/purity.rs` keeps out of the library |
| `clients/crates/h3-sdl` | the platform adapter (ADR-0024): every SDL 3 call and all of the client's `unsafe` code; `sdl3-sys` with SDL built from its sources and linked statically; the window (integer scale, nearest sampling, centred), its events, the hardware cursor, the read-back; the audio device and its streams; SDL's own WAV loader for the gate. Left out of the coverage gate as an adapter |
| `clients/assets/{base,ab,sod}.toml` | the asset tables per expansion (below): content keys to archive entry names and the drawing's numbers, asset fixes; identifiers and numbers only |
| `clients/layouts/battle.toml` | the battle screen's layout at the native resolution (ADR-0020 point 3: layouts are data): the frame, the hex boxes, where sprites stand, the stack canvas, the layer-1 pass's columns, the outline's palette entry, the heroes' and flags' canvases, the count box (`docs/epics/E17-design.md` §4), the panel below the battlefield as a dialog of the executable's items (§9.5, §9.7: its picture, buttons and the log's item, `h3_view::ui`); an HD layout is another file of the same format (*provisional*) |
| `clients/scenarios/*.toml` | the client's battle scenarios (the scenario format of `packs/README.md`) for `h3-view render` and `play`: the stills T7 renders, `smoke.toml` the captured smoke battle (T8), `battle.toml` the whole battle `play` fights (M2 T4b-2a) |
| `clients/capture/` | the reference captures (below; the runbook `capture/README.md`): `setup.sh`, the driver `h3drive.py` and its screens' probes, `h3cap.py` (diffs, obstacles), the `xdd.dll` source (forwarder and virtual clock) |
| `clients/situations/` | the capture set (`set.toml`) and its situations' scenario files: what the harness shows the original, with each frame's recorded differences; the recorded sequences (`sequences.toml`, `timelines/`) with their engine commands (the casts' since M3 T7a-2a, with the screen's changes); `kept/`, what the last captures kept and the viewer's digests at those times |
| `clients/crates/h3-map-view` | the adventure map's viewer (the map-client lane, E24): the library (pure, `no_std`) holds the map's static art read from the raw `.h3m` (`art`), its live part (`live`: the objects standing, a random object's drawn template, owners, heroes, boats, the viewer's explored tiles, the moment), the map's asset tables (`tables`), the adventure screen's layout (`layout`) and what the gate checks of a DEF (`check`); its binary the map's real-asset gate. Its package depends on `h3-assets`, `h3-map` (the raw model) and, for the binary only, `h3-formats` (`tests/purity.rs`, a copy of `h3-view`'s for this crate) |
| `clients/map/assets/base.toml` | the map's asset tables (below, "The map's asset tables") |
| `clients/map/layouts/adventure.toml` | the adventure screen's layout at the native resolution: the frame and its picture, the view's place, tile, loops and clip, the corner gems (`docs/epics/E24-design.md` §1.1, §1.2, §1.9); an HD layout is another file of the same format (*provisional*) |
| `clients/out/` | local output (renders, reports); gitignored |

The decoders, the compositor and the scene model are pure (no I/O, clock or threads; time is an
input) and tested on synthetic data: the libraries of `h3-assets`, `h3-render` and `h3-view` are
`no_std` and carry the engine's logic lint header; `h3-view`'s `tests/purity.rs` checks the header,
the sources and an allowlist of each pure crate's dependencies (the engine's `check-arch` does not
see this workspace). `h3-view`'s binary holds the adapters. The first client is a battle replay
viewer on the real assets; a full game record/replay viewer follows.

## The asset tables

One table per rule pack, `clients/assets/<pack>.toml` (ADR-0020 point 5), generated once by
`cargo run --release -p h3-view -- harvest [--install <dir>] [--packs <dir>] [--out <dir>]` from
the installation copy's `Heroes3.exe` and `H3bitmap.lod` (`cranim.txt`, `crtraits.txt`,
`hotraits.txt`, `sptraits.txt`), keyed by
the committed packs (its default `--out` is `clients/out/harvest`, so a fresh harvest is diffed
against the committed tables, never written over them); afterwards they are source, edited by hand
and reviewed as diffs. The client
never reads the executable. Loader: `h3_view::tables` (pure); `tests/tables.rs` checks them
against the packs; the gate checks their names against the archives. The format (*provisional*,
on the M1 review list):

- `[table]`: `id` (the pack's) and `depends` (the pack's dependencies). An expansion selection
  is a table id: it loads that table and its dependencies, dependencies first; keys are
  qualified by their table's id (`base:pikeman`). Unknown sections and fields, unknown or
  circular dependencies and a key defined twice are errors.
- `[battle]` (in exactly one table of a selection): `groups`, the creature DEF groups the battle
  shows (`moving`, `mouse_over`, `standing`, the reactions `hit`, `defend`, `death`, `strike` and
  `shoot` up, straight and down, `move_start`, `move_end`; design §7), `hero_groups` (a battle
  hero's: `standing`, `idle`, `defeat`, `victory`, `cast`; M3 T3a), `flags` (the left and the
  right side's flag DEFs), `magic` (M3 T3b: the magic animations by number, each `def`, `place`,
  where it is drawn over a stack, 0 on the hex, 1 centred, 2 above, 3 beside, higher not over a
  stack, and `translucent`; design §7.15), `heal_sound` (the First Aid Tent's heal),
  `resist_sound` (a spell resisted by the stack it was aimed at; M3 T7b-1) and
  `good_morale_sound`, `bad_morale_sound`, `fear_sound`, `luck_sound` (the rolled effects over a
  stack; M3 T7b-2b-2b, optional, entry names without an extension), `drain_sound`,
  `thunderbolt_sound` (Life Drain's and the Thunderbird's bolt's; M3 T7b-2b-3, optional).
- `[[creature]]`: `key`, `original` (its original id, the executable's creature number: its
  record among `crtraits.txt`'s creature rows; M3 T4b-1), `def` (the battle sprite), `times` (`cranim.txt`'s fidget interval, walk,
  attack and flight distance as the executable keeps them: scaled by 9000, 500, 500, 115 and
  truncated), optional `troop_count_offset`, `attack_climax`, `missile` (a shooter's projectile
  DEF), `missile_offsets` (up, straight, down; x, y), `missile_angles` (12, in degrees), `ray`
  (a ray shooter's five colours, R, G, B from the ray's edge inward, M3 T1b-1) and `sounds`
  (T5b: the sound archives' entries of `move`, `attack`, `wince`, `shot`, `death`, `defend`,
  `start`, `end`; one the creature lacks is left out; `h3-view harvest` writes them).
- `[[hero]]`: `key` (a hero type's), `original` (its hero type, its record among
  `hotraits.txt`'s; M3 T7b-1), `def`, its battle picture (by its class's town and its sex, design
  §4.5; M3 T3a), and from the same record `cast` (its cast point in the DEF's frame, x, y) and
  `cast_frames` (the casting group's frames played before the spell; M3 T7b-1, design §7.16).
- `[[spell]]` (M3 T7b-1): `key`, `original` (its spell number, its record among `sptraits.txt`'s
  spell rows), optional `animation` (its magic animation's number, the spell record's +8),
  `sound` (the record's +4, an entry name without its extension) and `projectile` with
  `projectile_angles` (the DEFs a handler flies and their angles in degrees: Magic Arrow's and Ice
  Bolt's).
- `[[battlefield]]`: `key` (a terrain's, or a special battlefield's: the packs' `[[battlefield]]`
  of `battle.toml`, M3 T2) and `backgrounds` (the pictures by the variant the adventure map
  picks; a special battlefield has one, which wins over the terrain's).
- `[[town]]` (M4 T1b, design §10.1; *provisional*, a review-list item): `key` (a faction's: the
  packs' `[[faction]]`, whose order is the town type), `original` (the town type), `background`
  (the siege's, in place of the terrain's), `turrets` (`shooter`, the creature drawn in them by
  its original id; `shift`, its drawer's extra x: 44 for a two-hex creature, 5 less for the
  Medusa; `missile`; the `keep`'s, `lower` and `upper` tower's points) and `parts`, the 18 wall
  parts by index (WALLS.TXT's rows): `at` (the picture's top-left), `hex` (the hex whose pass
  draws it, absent for the drawbridge, its rope, the moat, its lip and the back wall) and
  `pictures` (five, by the part's picture state; empty: none).
- `[[obstacle]]`: `key` (`obstacle_<n>`, the packs' numbering) and either `def`, `rows` (height in
  hex rows) and `layer` (1: drawn before every stack, 0: in the row passes) for a usual obstacle,
  or `pcx` and `at` (its top-left on the screen) for an absolute one.
- `[[fix]]`: `id` (`CLT-QUIRK-<n>`), `source`, `target` (the key it changes), `summary`; the change's
  own fields come with the first fix (none yet).

Which table an entry belongs to is the pack that defines its key (since M3 T2 every entry's: the
SoD fields and their obstacles in `sod.toml`); `harvest` would keep graphics no pack defines in
`base.toml`. Where the names and numbers come from: `docs/epics/E17-design.md` §5.

## The map's asset tables

The adventure map's tables (E24 M1 T2), `clients/map/assets/<pack>.toml` (only `base.toml` so
far: the AB and SoD packs add nothing the view draws until their objects come, T4), in the rule
packs' dependency graph as the battle's; written from the code reading (`docs/epics/E24-design.md`
§1.3 to §1.6) and maintained as source. Keyed by the `.h3m`'s codes, not by the packs' keys: the
map's static art is the raw map's (*provisional*, on the M1 review list). Loader:
`h3_map_view::tables` (pure); `tests/tables.rs` checks the committed table, the gate its names.
The format:

- `[table]`: `id`, `depends`, as the battle's tables.
- `[map]` (in exactly one table of a selection): `edge` (the map's edge, `edg.def`),
  `road_offset` (how far below its tile a road is drawn, 16), `cycle_ms` (the palette cycles'
  step, 200), `frame_ms` (the objects' frame step, 180); `[map.shroud]`: `full` and `edge` (the
  unexplored tiles' DEFs) and `pieces` (each mask's piece, 256 numbers: the executable's table).
- `[[terrain]]`, `[[river]]`, `[[road]]`: `code` (the `.h3m`'s; 0 is no river or road), `def` (a
  tile's picture is its group 0's frame), optional `cycles` (the palette ranges the tick rotates,
  first and last entry). A code defined twice in a selection is an error.

## The string catalogue

Every text the client shows is a key's message, looked up in language packs by priority: the
player's pack, the client's own, the original's (E17 M3 T4b-1; the user, 2026-10-13 and
2026-10-19; ADR-0020's amendment draft in `docs/lanes/client/adr-drafts/`; detail:
`docs/epics/E17-design.md` §9). Library: `h3_view::text` (pure); the command `h3-view text`
prints every key's forms from the installation, or one key's text (`h3-view text
battle.log.damage attacker=Imps damage=4 count=2 [--pack de.json]`). The battle log's lines are
composed from the keys as the executable composes them from its rows (`h3_view::battle_log`, M3
T4b-2a, design §9.4: a strike's damage and perish rows, wait, defend, morale, luck, Fear (M3
T7b-2b-2b), Life Drain, Death Stare, the Thunderbird's bolt, Poison, Acid Breath (M3 T7b-2b-3),
a cast without a target, Death Ripple's and Destroy Undead's damage, an area spell's (M3 T7c-1),
Armageddon's (M3 T7c-2), a raise's (M3 T7c-3), and the log
entries a line becomes; the text the log's item shows, M3 T4b-2b-2; the heal's and the round's
lines, which event writes which line and when, and the log kept as the executable keeps it, M3
T6b, design §9.8); `h3-view text --log` prints sample lines, and the viewer's frames show the
log's lines over the battle.

- **The key catalogue** `clients/text/keys.toml` (committed; identifiers and numbers only, as the
  asset tables): `[[key]]` with `id` (lower-case segments of letters, digits and `-` joined by
  `.`), `args` (the names its messages may use, each `text` or `number`), `plural` (forms by the
  number `count`), `contexts` (forms by the argument `context`) and `original`, its forms in the
  user's text tables: `form` (the suffix: `one`, `other`, a context; none for the only form),
  `at = "<TABLE>:<record>[:<column>]"` (records from 0 with the header rows) or
  `at = "EXE:0x<address>"` (M3 T7b-2b-3, *provisional*: a string of the user's `Heroes3.exe`
  itself, read at run time, for a line the executable writes from its own text: Acid Breath's)
  and `printf` (the arguments the row's `%s` and `%d` or `%i` stand for, in order);
  `[[family]]`: a key per asset-table
  entry (`creature`: `creature.<table>:<key>`, a creature's names; `spell` and `hero`, M3 T7b-1),
  its row the non-blank record of its original id from `first` (with `by`, the records whose
  column `by` is filled: `sptraits.txt`'s group rows fill only their first), its forms' columns in
  `forms`, or its one form's in `column`. Unknown fields are errors.
- **A language pack** is nested JSON with i18next's conventions: objects nest a key's segments,
  a string is a message, `{{name}}` a placeholder (any other brace is text: the original's
  highlight braces stay), the suffixes `_<context>` and `_<category>` its forms (`_one`, `_few`,
  `_many`, `_other`: CLDR's integer rules for the pack's language, which is its file's name,
  `de.json`). A lookup tries `<key>_<context>_<category>`, `…_<context>_other`,
  `…_<context>`, `<key>_<category>`, `<key>_other`, `<key>` in each pack, then the next pack; a
  key no pack has shows itself.
- **The original's text** is read at run time from the user's tables through the catalogue, with
  the executable's rule (`one` for exactly 1); it is never committed, nor is a translation of it:
  a player may load a pack of their own, which the project neither ships nor hosts (CLT-Q7).

## Reference captures

*Provisional* (E17 M1 T8, M2 T1, T3; ADR-0020 point 9; `docs/epics/E17-design.md` §6, §7). The original
runs unattended in a Linux sandbox under Wine, on a virtual X display at 16 bits, from a copy under
`original-data/` (never the installation); our `xdd.dll` takes over its clock, so every frame is
taken at an exact game time, and a driver steers it by fixed coordinates, recognising screens by
probe pixels. The capture set (`situations/set.toml`: every land terrain and special
battlefield, human-first openings, two-hex stacks facing either way, usual and absolute
obstacles, every hero's battle picture and the heroes' idle motion) runs with one command and compares
each frame with our render at its time, outside masks over what we do not draw yet. The game's own battle state
is read from its memory (M2 T3, `capture/h3mem.py`), so the recorded sequences
(`situations/sequences.toml`: a walk, shots, a flight and a strike with its retaliation) check the
animations' timing millisecond by millisecond against timelines of numbers. The runbook, its
common uses and its traps: [`capture/README.md`](capture/README.md). Captures stay in the sandbox
or `clients/out/` and are never committed (ADR-0009).

## Asset policy

- **No asset ever enters git or CI** (ADR-0009). The clients read the sprites, images, fonts and
  sounds from the user's own installation at run time; the installation is read-only.
- Decoders are tested on **small synthetic sprites** built in the tests (invented pixels and
  palettes), plus a **local real-asset gate** that runs only on a machine with the installation,
  like `cargo xtask verify --with-original-data` in the engine.
- A **headless render-to-PNG mode** lets agents check what a screen looks like without a window.
  Screenshots stay local: they go to a gitignored directory (`clients/out/`) and are never
  committed.
- The client lanes need the user's computer online for the real assets (`AGENTS.md`, "Lanes").
