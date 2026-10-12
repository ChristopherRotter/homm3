# clients/ — playable clients (client lane)

Everything with graphics, sound, input or a window lives here, outside the engine core. The engine
core (`crates/`) stays headless, deterministic and free of assets; clients use it through its
public crates and, later, through the server's protocol.

**State:** refined (E17, 2026-10-12), nothing built yet. Decisions: the epic file
`docs/epics/E17-clients.md` and the ADR draft `docs/lanes/client/adr-drafts/client.md`; tasks in
`docs/lanes/client/QUEUE.md`.

## Layout

A separate Cargo workspace with its own `Cargo.toml` and `Cargo.lock`, excluded from the engine
workspace (root `Cargo.toml`: `exclude = ["clients"]`) and with its own CI job. Planned (E17 M0):

| Path | What |
|---|---|
| `clients/Cargo.toml`, `clients/Cargo.lock`, `clients/deny.toml` | the client workspace |
| `clients/crates/h3-assets` | decoders: palettes, PCX, DEF, MSK, fonts, later sound and video; reads LOD archives through `crates/h3-formats` (a path dependency into the engine workspace) |
| `clients/crates/h3-render` | the software compositor: indexed sprites into a frame in the original's screen format; PNG output |
| `clients/crates/h3-view` | the scene model (messages to what is on screen at a time) and its adapters: the headless renderer and the real-asset gate, later the window |
| `clients/assets/{base,ab,sod}.toml` | the asset tables per expansion: content keys to archive entry names, animation groups, offsets, asset fixes; identifiers and numbers only |
| `clients/out/` | local output (renders, reports); gitignored |

The decoders, the compositor and the scene model are pure (no I/O, clock or threads; time is an
input) and tested on synthetic data. The first client is a battle replay viewer on the real
assets; a full game record/replay viewer follows.

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
