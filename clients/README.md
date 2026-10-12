# clients/ — playable clients (client lane)

Everything with graphics, sound, input or a window lives here, outside the engine core. The engine
core (`crates/`) stays headless, deterministic and free of assets; clients use it through its
public crates and, later, through the server's protocol.

**State:** E17 M0 built (2026-10-12): the workspace, its gates (`scripts/check.sh`, the CI job
`.github/workflows/clients.yml`) and the real-asset gate. M1 T2 + T3 (2026-10-12): the decoders
for palettes, PCX, DEF and MSK (`h3-assets`); the gate decodes every one of them in the user's
archives (findings: `docs/epics/E17-design.md`). M1 T1 + T6 (2026-10-12): the code reading of
the executable's drawing (design §3) and the compositor (`h3-render`) that follows it; `h3-view
sprite` draws a DEF's group over an image into a PNG. M1 T5 + T4 (2026-10-13): the battle
screen's reading (design §4) and the asset tables (`clients/assets/`, below). Decisions: ADR-0020
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
  (`--tables <dir>`, default `clients/assets`) is found where the game searches it and decodes. A
  debug build takes about ten times as long.
- **A look at the drawers:** `cargo run --release -p h3-view -- sprite CPKMAN.DEF --group 2
  --background CmBkGrMt.pcx [--mirror]` draws every frame of one group of a DEF side by side over
  a PCX, as the battle draws a stack, into `clients/out/sprite/` (the base archives searched
  first, as the game does).

| Path | What |
|---|---|
| `clients/Cargo.toml`, `clients/Cargo.lock`, `clients/deny.toml` | the client workspace (`deny.toml`: the engine's policy, kept identical by `check.sh` until M2) |
| `clients/scripts/check.sh` | the client's gates (local and CI) |
| `clients/crates/h3-assets` | decoders: palettes, PCX, DEF, MSK (done), fonts, later sound and video; encoders for synthetic test assets (`build`); the gate's checks (`check`, `survey`); reads LOD archives through `crates/h3-formats` (a path dependency into the engine workspace) |
| `clients/crates/h3-render` | the software compositor: the screen in a 16-bit pixel format, palettes truncated into it, the executable's frame drawers (shadows, outline and flag colours, mirroring, clipping), opaque images, PNG bytes |
| `clients/crates/h3-view` | the scene model (messages to what is on screen at a time; the library) and the asset tables' loader (`tables`); its adapters (the binary, `src/bin/h3-view/`): the real-asset gate, the tables' harvester, the headless renderer, later the window |
| `clients/assets/{base,ab,sod}.toml` | the asset tables per expansion (below): content keys to archive entry names and the drawing's numbers, asset fixes; identifiers and numbers only |
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
the installation copy's `Heroes3.exe` and `H3bitmap.lod` (`cranim.txt`, `crtraits.txt`), keyed by
the committed packs; afterwards they are source, edited by hand and reviewed as diffs. The client
never reads the executable. Loader: `h3_view::tables` (pure); `tests/tables.rs` checks them
against the packs; the gate checks their names against the archives. The format (*provisional*,
on the M1 review list):

- `[table]`: `id` (the pack's) and `depends` (the pack's dependencies). An expansion selection
  is a table id: it loads that table and its dependencies, dependencies first; keys are
  qualified by their table's id (`base:pikeman`). Unknown sections and fields, unknown or
  circular dependencies and a key defined twice are errors.
- `[battle]` (in exactly one table of a selection): `groups`, the creature DEF groups the battle
  shows (`moving`, `mouse_over`, `standing`).
- `[[creature]]`: `key`, `def` (the battle sprite), `times` (`cranim.txt`'s fidget interval, walk,
  attack and flight distance as the executable keeps them: scaled by 9000, 500, 500, 115 and
  truncated), optional `troop_count_offset`, `attack_climax`, `missile` (a shooter's projectile
  DEF), `missile_offsets` (up, straight, down; x, y) and `missile_angles` (12, in degrees).
- `[[battlefield]]`: `key` (a terrain's, or a special battlefield's while the packs define none)
  and `backgrounds` (the pictures by the variant the adventure map picks).
- `[[obstacle]]`: `key` (`obstacle_<n>`, the packs' numbering) and either `def`, `rows` (height in
  hex rows) and `layer` (1: drawn before every stack, 0: in the row passes) for a usual obstacle,
  or `pcx` and `at` (its top-left on the screen) for an absolute one.
- `[[fix]]`: `id` (`CLT-QUIRK-<n>`), `source`, `target` (the key it changes), `summary`; the change's
  own fields come with the first fix (none yet).

Which table an entry belongs to is the pack that defines its key; the graphics no pack defines yet
(special battlefields, their obstacles) sit in `base.toml`. Where the names and numbers come from:
`docs/epics/E17-design.md` §5.

## Asset policy

- **No asset ever enters git or CI** (ADR-0009). The clients read the sprites, images, fonts and
  sounds from the user's own installation at run time; the installation is read-only.
- Decoders are tested on **small synthetic sprites** built in the tests (invented pixels and
  palettes), plus a **local real-asset gate** that runs only on a machine with the installation,
  like `cargo xtask verify --with-original-data` in the engine.
- A **headless render-to-PNG mode** lets agents check what a screen looks like without a window.
  Screenshots stay local: they go to a gitignored directory (`clients/out/`) and are never
  committed.
- The client lane needs the user's computer online for the real assets (`AGENTS.md`, "Lanes").
