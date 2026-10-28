# clients/ — playable clients (client and map-client lanes)

**Two lanes** (since 2026-10-23, `lanes.toml`): the **client** lane builds the battle client (E17,
`h3-view`, the battle's tables, layouts, scenarios and situations), the **map-client** lane the
adventure map client (E24, its crates `crates/h3-map…` and its folder `map/`). The foundation,
`h3-assets`, `h3-render`, `h3-sdl`, both viewers' common parts `h3-ui` (E24 M3 T1a), this
workspace's `Cargo.toml`, `deny.toml`, `.gitignore`,
this file, `scripts/` and the string catalogue `text/`, and the capture harness `capture/` are
common: either lane changes them in its own pull request, an additive change freely, a change in
behaviour noted in the other lane's inbox (`docs/08-work-process.md`, "Lanes").

Everything with graphics, sound, input or a window lives here, outside the engine core. The engine
core (`crates/`) stays headless, deterministic and free of assets; clients use it through its
public crates and, later, through the server's protocol.

**State:** each lane's position is in its HANDOFF (`docs/lanes/client/HANDOFF.md`,
`docs/lanes/map-client/HANDOFF.md`), the done tasks in the epics' logs (`docs/epics/E17-log.md`,
`docs/epics/E24-log.md`); this file's earlier state history (E17 M0 to M3 T3b) is in
`docs/epics/E17-archive.md`, "clients/README.md's state history". Decisions: ADR-0020
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
  has a simple bot fight the scenario to its end in-process (or reads a record file), as it is
  played since E17 M5 T1b (each command asked for when the frames reach the timeline's end, sent
  from its seat through `h3-protocol`'s session: `live.rs`), plays its commands at the
  original's timing with the viewer's own fidgets and draws every distinct frame:
  `clients/out/play/<stem>-s<seed>/` holds the record, `frames.txt` (each frame's moment),
  `cues.txt` (the creatures' sounds as they start and stop, T5b) and, with `--png`, `f<ms>.png`;
  `--to` past the end lets the battle idle. `--moments` (E17 M5 T1a) times a frame's moment for
  each quarter of the frames: `anim::Playhead`'s, which reads the timeline forward and draws
  `play`'s, `window`'s and `sequences`' frames, against `scene::Moment::at`'s, the reference read
  from the battle's start.
- **A battle in a window** (E17 M2 T5a, ADR-0024, design §8.1): `cargo run --release -p h3-view
  -- window scenarios/battle.toml [--seed <n>] [--scale <n>] [--vsync] [--drawn-cursor]
  [--fullscreen]` (from `clients/`) loads the battle as `play` does and shows each frame when it
  falls due, at the largest integer scale that fits the display. Every key goes through the
  table `h3_view::keys` (E24 M3 T1b, *provisional*): the window's own are the map window's (F2
  the music, F3 the creatures' sounds, F5 pause, F6 a frame on while paused, F7 restart, F8
  vsync, F9 the cursor's kind, F11 fullscreen, Esc quit), the battle's the original's hotkeys by
  what is open over the battle (nothing, the spell book, a message box, an aim: each takes its
  own and the window's that work whatever is open, so Esc is the book's close, the box's Cancel
  and the aim's give-up there); the user's `keys.toml` (`--keys`, else in the working directory)
  overrides them in its `[battle]` table (its format: the usage). The mouse is the battle's (E17 M3 T6c-2,
  design §9.10): its cursor, the stack it marks and enters, its hints in the log; a panel button
  shows its click frame while the mouse or its hotkey (S, R, O, A, C, W, D, Space) holds it, the
  log's scroll buttons and ↑ ↓ scroll the log. Retreat and Surrender (their buttons, R and S; E17
  M5 T4a, design §7.19) ask first in a message box (the price, then "You don't have enough gold!"
  when the side cannot pay), its OK or Enter leaving the battle, Cancel or Esc fighting on. The sound (T5b, design §8.2; the installation's
  `Data/*.snd` and `MP3/`): from the battle's start its start sound plays while the clock waits,
  then the battle music loops, and every stack's sounds follow its animations; once the battle
  is decided its end's tune plays once instead (M3 T8); `--mute` plays none (in a sandbox: `SDL_AUDIO_DRIVER=disk` writes the mix to `./sdlaudio.raw`).
  `--measure <s>` runs unattended and prints how late the frames showed; `--check <s>` also reads
  every frame back and compares it with the frame drawn headless (exit 1 on a difference; in a
  sandbox under Xvfb); `--leave retreat|surrender` makes an unattended run's human side leave at
  its first turn it may, through the question, the battle checked against the bot's up to it. Building it builds SDL 3 from its sources: CMake, a C compiler and, on
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
  shipped map gives (`<install>/Maps`); since E24 M2 T5 every sound the tables name in the sound
  archives and every terrain's music in `<install>/MP3`, decoded; it prints each DEF's frames,
  encodings and run pixels, each sound's length, and exits 1 on a problem (findings:
  `docs/epics/E24-design.md` §1.9).
- **A still of the adventure map** (E24 M1 T3b1): `cargo run --release -p h3-map-view -- render
  <map.h3m> --view <x>,<y>[,<z>]` (from `clients/`) draws the adventure screen at that view as
  the original does, so far its frame and the view's terrain, rivers, roads, edge, objects and
  their shadows (T4), heroes and boats (T5a), unexplored tiles and corner gems, and writes a PNG
  under
  `clients/out/map/`; `--at`, `--cycles`, `--viewer`, `--hidden`, `--bugs original` and `--list`
  as its usage says (findings: `docs/epics/E24-design.md` §1.10). `h3-map-view situation
  <situation.toml> --out <map.h3m>` writes a situation's map (`map/situations/`, E24 M1 T3b2);
  `python3 capture/h3map.py set` compares the map's capture set with the original (the
  runbook, "The map's stills").
- **The map over time** (E24 M2 T2): `cargo run --release -p h3-map-view -- sequences [<id>]
  [--frames <dir>]` (from `clients/`) replays the recorded sequences (`map/sequences.toml`)
  without the original: the map's timeline (`h3_map_view::timeline`: the redraws, the palette
  steps, the scroll by Ctrl and the direction keys and at the frame's edges, the edge cursors)
  from each sequence's situation, its live part at 0 and its inputs, the screen drawn wherever it
  changes, every line compared with `map/timelines/<id>.txt` (exit 1 on a difference; a walk,
  E24 M2 T3a: the selected hero's steps and the path's walk at the hero speeds, its lines but the
  `screen` ones, which wait for the walk drawn, M2 T3b); `--frames` writes our frames for the harness's frame check, `python3
  capture/h3map.py frames` (the runbook, "The map's sequences").
- **The map in a window** (E24 M2 T4a, T4b1, T5): `cargo run --release -p h3-map-view -- window
  <sequence | situation>` (from `clients/`) shows a recorded sequence (its inputs replayed at
  their times) or a situation (its kept live part) through `h3-sdl` at the original's frame
  times, drawn as `sequences` draws it. Past a sequence's inputs, and in a situation from the
  start, the map takes the player's input (T4b1, design §3.10): Ctrl and a direction key (the
  keypad's too) scroll, a direction key steps the selected hero (a held key repeats), the
  pointer at the frame's edges scrolls with the edge cursors, a click shows a path the map knows
  and walks it on its destination; the cursor is the hardware one, or drawn (`--drawn-cursor`).
  The map's sound plays (T5, design §3.11: the terrain's music as the original's music thread
  plays it (T6a2), the step sound, the ambient loops; `--mute` none, `--cues <file>` writes each cue played). Every key goes through the table
  `h3_map_view::keys` (*provisional*): F2 the music, F3 the effects, F5 pause, F6 the next frame
  while paused, F7 restart, F8 vsync, F9 the cursor's kind, F11 fullscreen, Esc quit (the
  battle window's own too, E24 M3 T1b). A user's `keys.toml` (`--keys`, else in the working
  directory) overrides them command by command in its `[map]` table (its format: the usage). `--check <s>` reads every frame back (Xvfb in the sandbox) and plays
  every action headless to check each change shown, `--measure <s>` times them; both take the
  map's input.

| Path | What |
|---|---|
| `clients/Cargo.toml`, `clients/Cargo.lock`, `clients/deny.toml` | the client workspace (`deny.toml`: the engine's policy, kept identical by `check.sh` until M2) |
| `clients/scripts/check.sh` | the client's gates (local and CI) |
| `clients/crates/h3-assets` | decoders: palettes, PCX, DEF, MSK, the bitmap fonts (`fnt`, M3 T4a), the sound archives and their IMA ADPCM WAVs (done), later video; encoders for synthetic test assets (`build`); the gate's checks (`check`, `survey`); reads LOD archives through `crates/h3-formats` (a path dependency into the engine workspace) |
| `clients/crates/h3-render` | the software compositor: the screen in a 16-bit pixel format, palettes truncated into it, the executable's frame drawers (shadows, outline and flag colours, mirroring, clipping), opaque images, text in the bitmap fonts (`text`: the executable's text drawer) and its tint of a rectangle (`tint`), PNG bytes |
| `clients/crates/h3-view` | the scene model (the library: `record` reads a record, `anim` plays its commands at the original's timing, `scene` makes the draw list of a snapshot at a moment, `batch` holds a checkpoint's decoded assets, `compose` draws the list into a frame; `tables` and `layout` load the asset tables and the layouts, of `h3-ui`'s dialog items, and `panel` is the battle's panel of them: M3 T6a; `mouse`, the battle's mouse as a still and over time, `mouse::track`: M3 T6c; `keys`, the battle window's commands by what is open over the battle, their names and the original's keys, through `h3-ui`'s table: E24 M3 T1b); its adapters (the binary, `src/bin/h3-view/`): the real-asset gate, the tables' harvester, `sprite`, the headless renderer `render` with its scenario adapter, the timing check `sequences`, `play` (a battle fought by its bot and drawn frame by frame), `situation` (a scenario's battle as a map for the original), `window` (the battle in a window, through `h3-sdl`; with `play` it loads the battle through `battle.rs`). The library also has `sound` (the opening's start sound and music, the end's tune); its clock, cursor, dialog items and string catalogue are `h3-ui`'s (E24 M3 T1a). Its package depends on `h3-protocol` and `h3-ui` (the library) and, for the adapters only, on `h3-pack-battle`, `h3-battle`, `h3-content-battle`, `h3-kernel`, `h3-map`, `h3-sdl` and `nanomp3` (the music), which `tests/purity.rs` keeps out of the library |
| `clients/crates/h3-ui` | what both viewers share (E24 M3 T1a, the user's decision at its T0; common to both client lanes as the foundation is), pure and `no_std`: `clock` (a viewer's clock in the window), `cursor` (the game's mouse cursor) with `sprite` (the decoded DEF it draws), `ui` (the dialog items screens are made of, E17 M3 T6a, and their own draw items, which a viewer's scene takes), `text` (the string catalogue and its language packs), `keys` (the table of what each key does, generic over a viewer's commands and the contexts a dialog over its screen makes, with both windows' own commands and the user's file of `[map]` and `[battle]` tables: E24 M3 T1b) and `music` (the original's music thread for a window's music stream, E24 M2 T6a2). Its package depends on `h3-assets`, `h3-render`, `serde`, `serde_json` and `toml` (its own `tests/purity.rs`) |
| `clients/crates/h3-sdl` | the platform adapter (ADR-0024): every SDL 3 call and all of the client's `unsafe` code; `sdl3-sys` with SDL built from its sources and linked statically; the window (integer scale, nearest sampling, centred), its events, the hardware cursor, the read-back; the audio device and its streams; SDL's own WAV loader for the gate. Left out of the coverage gate as an adapter |
| `clients/assets/{base,ab,sod}.toml` | the asset tables per expansion (below): content keys to archive entry names and the drawing's numbers, asset fixes; identifiers and numbers only |
| `clients/layouts/battle.toml` | the battle screen's layout at the native resolution (ADR-0020 point 3: layouts are data): the frame, the hex boxes, where sprites stand, the stack canvas, the layer-1 pass's columns, the outline's palette entry, the heroes' and flags' canvases, the count box (`docs/epics/E17-design.md` §4), the panel below the battlefield as a dialog of the executable's items (§9.5, §9.7: its picture, buttons and the log's item, `h3_ui::ui`), the hero's spell book (`[book]`, §7.17: a dialog of pictures, DEF items, texts and areas, its shadow, the schools' order, the ribbons and its cursor; `h3_view::book`); an HD layout is another file of the same format (*provisional*) |
| `clients/scenarios/*.toml` | the client's battle scenarios (the scenario format of `packs/README.md`) for `h3-view render` and `play`: the stills T7 renders, `smoke.toml` the captured smoke battle (T8), `battle.toml` the whole battle `play` fights (M2 T4b-2a) |
| `clients/capture/` | the reference captures (below; the runbook `capture/README.md`): `setup.sh`, the driver `h3drive.py` and its screens' probes, `h3cap.py` (diffs, obstacles), the `xdd.dll` source (forwarder and virtual clock) |
| `clients/situations/` | the capture set (`set.toml`) and its situations' scenario files: what the harness shows the original, with each frame's recorded differences; the recorded sequences (`sequences.toml`, `timelines/`) with their engine commands (the casts' since M3 T7a-2a, with the screen's changes); `kept/`, what the last captures kept and the viewer's digests at those times |
| `clients/crates/h3-map-view` | the adventure map's viewer (the map-client lane, E24): the library (pure, `no_std`) holds the map's static art read from the raw `.h3m` (`art`), its live part (`live`: the objects standing, a random object's drawn template, owners, heroes, boats, the viewer's explored tiles, the path shown, the moment), the map's asset tables (`tables`), the adventure screen's layout (`layout`), what the gate checks of a DEF (`check`), the still: the draw list at a view (`scene`), its decoded assets (`batch`), the map's drawers (`draw`) and the composition into a frame (`compose`), the capture set's situations (`situation`), the map over time (`timeline`: the original's loop on a clock of milliseconds; `sequence`, the recorded sequences; E24 M2 T2), the window's (E24 M2 T4a: `replay`, the timeline run forward as the clock needs, since T4b1 given the player's actions as they come; `keys`, the window's commands, their defaults and actions, through `h3-ui`'s table) and the map's sound (`sound`: the tiles' ambient sounds and the original's sound state, cued by the timeline; E24 M2 T5); its clock, cursor, key table and music are `h3-ui`'s (E24 M3 T1a); its binary the map's real-asset gate, `render`, `situation`, `sequences` and `window`. Its package depends on `h3-assets`, `h3-map` (the raw model), `h3-render`, `h3-ui` and, for the binary only, `h3-formats`, `h3-sdl` and `nanomp3` (`tests/purity.rs`, a copy of `h3-view`'s for this crate) |
| `clients/map/assets/base.toml` | the map's asset tables (below, "The map's asset tables") |
| `clients/map/layouts/adventure.toml` | the adventure screen's layout at the native resolution: the frame and its picture, the view's place, tile, loops and clip, the corner gems (`docs/epics/E24-design.md` §1.1, §1.2, §1.9), the scroll's limits and edge zones and the adventure cursor's DEF, frames and hot spots (§3.4, E24 M2 T2); an HD layout is another file of the same format (*provisional*) |
| `clients/out/` | local output (renders, reports); gitignored |

The decoders, the compositor and the scene model are pure (no I/O, clock or threads; time is an
input) and tested on synthetic data: the libraries of `h3-assets`, `h3-render`, `h3-ui` and
`h3-view` are `no_std` and carry the engine's logic lint header; `h3-view`'s `tests/purity.rs`
checks the header, the sources and an allowlist of each pure crate's dependencies (`h3-ui` and
`h3-map-view` have their own; the engine's `check-arch` does not see this workspace). `h3-view`'s binary holds the adapters. The first client is a battle replay
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
  `thunderbolt_sound` (Life Drain's and the Thunderbird's bolt's; M3 T7b-2b-3, optional),
  `wall_explosions` (a wall shot's two explosion DEFs: a shot with damage's, then one without;
  M4 T2b), `wall_sounds` (their landings' sounds, the same order) and `siege_sounds` (the
  drawbridge's and a Land Mine's blast; M4 T2c-1), each optional, the sounds entry names without
  an extension (a siege without them draws no explosion and plays no such sound).
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
- `[map]` (in exactly one table of a selection): `edge` (the map's edge, `edg.def`), `path`
  (the path's arrows and their shadows, `adag.def`, its 50 frames; E24 M1 T5b2), `road_offset`
  (how far below its tile a road is drawn, 16), `cycle_ms` (the palette cycles' step, 200),
  `frame_ms` (the objects' frame step, 180), `scroll_steps` (a scroll step's tiles at the three
  scroll speeds, 1, 2, 3), `scroll_ms` (the edge scroll's time between steps, 70) and `poll_ms`
  (the mouse poll's, 33; E24 M2 T2); `[map.walk]` (E24 M2 T3a): `groups` (the walking group by
  the step's direction, 8), `frames` (the frames a walking group steps through, 8), `frame_ms` (a
  frame's time at the hero speeds 1 to 3, 50) and `pixels` (per hero speed 1 to 4 the pixels each
  frame slides the view by, summing to a tile; none at speed 4, one draw); `[map.shroud]`: `full` and `edge` (the
  unexplored tiles' DEFs) and `pieces` (each mask's piece, 256 numbers: the executable's table);
  `[map.heroes]` (E24 M1 T5a): `classes` (a hero's map DEF by class, 18), `flags` (its flag by
  owner, 8), `boats` and `waves` (by type, 3 each), `boat_flags` (by type × 8 + colour, 24) and
  `groups` (the group each facing 0–7 draws; mirrored above 4), each list of exactly its
  length; `[map.owners]` (E24 M1 T5b1): `palette` (`game.pal`), `neutral` (the entry of an
  object without an owner, 63; a colour's is the next but the colour) and `classes` (the classes
  whose bodies draw their flag pixels in it); `[map.towns]`: `village`, `fort` and `capitol` (a
  town's picture by faction without a Fort, with one, with a Capitol: 9 each); optional
  `[map.sound]` (E24 M2 T5, *provisional*; `docs/epics/E24-design.md` §3.11; without it the map
  plays no sound): `flying` (the flying hero's step sound, `horse10.wav`), `loops` (the ambient
  loops' WAVs by the original's index, 70), `volumes` (a loop's volume by distance 0 to 3),
  `shore` and `shore_pictures` (water below that picture sounds as that loop), `loaded_as` (the
  map's classes the game loads as others, `[map's, game's]`), `ownable` and `hole` (the classes
  a tile without an entrance records first: 0x505610's walks), `objects` (an entrance tile's loop
  by class, `{ class, subclass?, loop }`, the first that matches), `dwelling_creatures` (a
  dwelling's creature by subclass), `creatures` (its loop by creature, `{ creatures, loop }`) and
  `creature_default`, `mines` (a mine's loop by type) and `abandoned`, `blocking` (a tile's loop
  without an entrance by its class) and `grounds` with `ground_loops` (the special grounds of a
  tile of no class, the first from the top, and their loops); every loop named by its WAV name,
  one of `loops`.
- `[[terrain]]`, `[[river]]`, `[[road]]`: `code` (the `.h3m`'s; 0 is no river or road), `def` (a
  tile's picture is its group 0's frame), optional `cycles` (the palette ranges the tick rotates,
  first and last entry); a terrain's optional `music` (its track's name, `MP3/<name>.mp3`; none
  where the music goes on, rock) and `step` (the walking hero's step sound; E24 M2 T5). A code
  defined twice in a selection is an error.

## The string catalogue

Every text the client shows is a key's message, looked up in language packs by priority: the
player's pack, the client's own, the original's (E17 M3 T4b-1; the user, 2026-10-13 and
2026-10-19; ADR-0020's amendment draft in `docs/lanes/client/adr-drafts/`; detail:
`docs/epics/E17-design.md` §9). Library: `h3_ui::text` (pure; `h3_view::text` until E24 M3 T1a); the command `h3-view text`
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
  `forms` (plural), in `contexts` (by the argument `context`: a spell's description by mastery,
  E17 M5 T3a), or its one form's in `column`; `list` names another family's entries (the
  `spell` list for `spell-text`); `raw = true` takes the texts as they are, not as `printf`
  rows (a `%` is a percent sign). Unknown fields are errors.
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
