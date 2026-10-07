# clients/ — playable clients (client lane)

Everything with graphics, sound, input or a window lives here, outside the engine core. The engine
core (`crates/`) stays headless, deterministic and free of assets; clients use it through its
public crates and, later, through the server's protocol.

**State:** empty. The client lane (`docs/lanes/client/HANDOFF.md`) starts with the user's asset
notes and the client refinement (E17); implementation waits until the battle actions are mostly
stable (E04b).

## Layout

A separate Cargo workspace with its own `Cargo.toml` and `Cargo.lock`, excluded from the engine
workspace (root `Cargo.toml`: `exclude = ["clients"]`) and with its own CI job. Planned:

| Path | What |
|---|---|
| `clients/Cargo.toml`, `clients/Cargo.lock` | the client workspace |
| `clients/crates/h3-assets` | asset decoding: DEF sprites, PCX images, palettes, fonts, sound; reads LOD archives through `crates/h3-formats` (a path dependency into the engine workspace) |
| `clients/crates/…` | the renderer and the clients, chosen in the client ADR (Rust by default) |

The first client is a battle replay viewer on the real assets; a full game record/replay viewer
follows.

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
