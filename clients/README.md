# clients/ — playable clients (client lane)

Everything with graphics, sound, input or a window lives here, outside the engine core. The engine
core (`crates/`) stays headless, deterministic and free of assets; clients use it through its
public crates and, later, through the server's protocol.

**State:** E17 M0 built (2026-10-12): the workspace, its gates (`scripts/check.sh`, the CI job
`.github/workflows/clients.yml`) and the real-asset gate. M1 T2 + T3 (2026-10-12): the decoders
for palettes, PCX, DEF and MSK (`h3-assets`); the gate decodes every one of them in the user's
archives (findings: `docs/epics/E17-design.md`). Decisions: ADR-0020
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
  to `clients/out/gate/report.md`. A debug build takes about ten times as long.

| Path | What |
|---|---|
| `clients/Cargo.toml`, `clients/Cargo.lock`, `clients/deny.toml` | the client workspace (`deny.toml`: the engine's policy, kept identical by `check.sh` until M2) |
| `clients/scripts/check.sh` | the client's gates (local and CI) |
| `clients/crates/h3-assets` | decoders: palettes, PCX, DEF, MSK (done), fonts, later sound and video; encoders for synthetic test assets (`build`); the gate's checks (`check`, `survey`); reads LOD archives through `crates/h3-formats` (a path dependency into the engine workspace) |
| `clients/crates/h3-render` | the software compositor: indexed sprites into a frame in the original's screen format; PNG output |
| `clients/crates/h3-view` | the scene model (messages to what is on screen at a time; the library) and its adapters (the binary, `src/bin/h3-view/`): the real-asset gate, the headless renderer, later the window |
| `clients/assets/{base,ab,sod}.toml` | the asset tables per expansion: content keys to archive entry names, animation groups, offsets, asset fixes; identifiers and numbers only |
| `clients/out/` | local output (renders, reports); gitignored |

The decoders, the compositor and the scene model are pure (no I/O, clock or threads; time is an
input) and tested on synthetic data: the libraries of `h3-assets`, `h3-render` and `h3-view` are
`no_std` and carry the engine's logic lint header; `h3-view`'s `tests/purity.rs` checks the header,
the sources and an allowlist of each pure crate's dependencies (the engine's `check-arch` does not
see this workspace). `h3-view`'s binary holds the adapters. The first client is a battle replay
viewer on the real assets; a full game record/replay viewer follows.

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
