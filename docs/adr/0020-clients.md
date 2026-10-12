# ADR-0020: Clients: a Rust workspace, a software compositor, per-expansion asset tables, loading at checkpoints

- Status: Accepted (user, 2026-10-12, the E17 M0 checkpoint; point 7 rewritten at the checkpoint:
  loading at checkpoints, CLT-Q2)
- Date: 2026-10-12 (drafted in the E17 refinement the same day)
- Deciders: user; client lane sessions (E17 refinement, M0)
- Options and reasoning: [`docs/epics/E17-clients.md`](../epics/E17-clients.md); working detail:
  `docs/epics/E17-design.md` (created by the first task with detail to record)

<!-- Decisions only. Reasoning and options: docs/epics/E17-clients.md; detail: E17-design.md. -->

## Context

Playable clients live outside the engine core, in the `clients/` workspace (`clients/README.md`).
The first is a battle replay viewer on the real assets; a full game record/replay viewer follows.
The original is a 2D game that draws 8-bit palette sprites into an 800×600 frame; the client has
to be truthful to it, down to pixel offsets, timing and frame rate, and agents must be able to
check what it draws without a window. Assets come only from the user's installation, never from
git or CI (ADR-0009). The project is GPL-2.0-or-later and admits only GPLv2-compatible
dependencies (`deny.toml`). HD resolutions come later and must not need a rewrite. The user wants
loading as fast as the original's and the content configured per expansion, as the rules are
(asset notes in `docs/lanes/client/HANDOFF.md`).

## Decision

1. **Rust, in the separate `clients/` workspace** (own lockfile, excluded from the engine
   workspace, own CI job). Clients use the engine only through public crates, as path
   dependencies: `h3-formats` (LOD archives), `h3-protocol` (wire types, records, its sans-I/O
   session), later `h3-content`/`h3-pack` (the content check, local previews). A change the client
   needs in an engine crate goes through that crate's lane (autonomy contract: stop the line).
2. **Three crates to start:** `h3-assets` (decoders: palettes, PCX, DEF, MSK, fonts, later sound
   and video; bytes in, typed data out), `h3-render` (the compositor), `h3-view` (the scene model
   and its adapters: the headless renderer, later the window). The decoders, the compositor and
   the scene model are **pure**: no I/O, clock, threads or globals; time is an input. They are
   tested on synthetic data at 100 % coverage; file access, the CLI and the window are adapters.
3. **A software compositor, no GPU in the rendering path.** Sprites stay indexed as in the
   archives. `h3-render` composes them into a frame in the original's screen pixel format (the
   M1 code reading decides it), applying transparency, shadows, selection and player colours as
   the executable does. Render-to-PNG writes that same frame, so the window and the headless mode
   show identical pixels. The frame size and every screen layout are data, never literals in
   code; scaling happens only when a frame is shown.
4. **Window, input and audio are one thin adapter, chosen at M2** by a short evaluation of the
   candidates (`winit` + `softbuffer` + `cpal`, SDL3, `miniquad`) on technical grounds. If the
   choice needs Apache-2.0-only crates, `clients/` gets its own `deny.toml` that allows them and
   the client binaries are distributed under GPLv3; the engine stays GPL-2.0-or-later. That step
   is the user's decision at M2.
5. **Asset tables per expansion, committed:** `clients/assets/{base,ab,sod}.toml`, in the rule
   packs' dependency graph and key namespace (`base:pikeman`). They map content keys to archive
   entry names, animation groups and offsets, and hold the asset fixes. Identifiers and numbers
   only: never pixels, palettes, sound or text. A harvesting tool in `clients/` generates them
   once from the user's executable and text tables; afterwards they are maintained as source (as
   the packs are, ADR-0009 amendment 2). **The client never reads the executable at run time.**
   The real-asset gate checks every name in the tables against the archives.
6. **Asset fixes are data.** Each is a table entry with an id (`CLT-QUIRK-<n>`), a source (a code
   reading or a reference screenshot) and a flag; a client profile selects `faithful` (the
   original's pixels) or `fixed`. No fix exists outside an entry.
7. **Loading at checkpoints, without conversion.** No preprocessing step and no cache on disk. At
   start the client reads only the archives' directories. At each checkpoint (the client opens, a
   game starts, a battle starts, …) it reads, inflates and decodes, in one batch, every asset the
   coming screens need, so that nothing is loaded while a screen is shown; each checkpoint's asset
   set is data (the asset tables). A batch's entries are read by offset through a table-only
   reader in `h3-formats` (CLT-Q2); until it exists, the adapter reads whole archives.
8. **One input path.** The viewer plays `h3-protocol` messages: a record file and a live
   connection go through the same code. An engine replay becomes a record by running
   `h3-protocol`'s session in-process.
9. **Fidelity first.** The first iteration matches the original's native layout, resolution,
   timing and frame rate exactly. References are frames captured from a copy of the original game
   under `original-data/` (never from the installation), compared exactly after the conversion
   from the screen format.
10. **Assets stay local** (ADR-0009): CI uses synthetic sprites; the real-asset gate runs only
    where the installation's files are; screenshots go to the gitignored `clients/out/`.

## Consequences

- Loading costs fall at the checkpoints and scale with what a screen needs (a battle about 3–5
  MiB of the archives' 212 MiB), not with the installation.
- Exact pixel diffs against the original are possible, and every visual change is a reviewable
  diff of code or of an asset table.
- Headless rendering needs no GPU or display, in CI or in a cloud sandbox.
- The renderer does not get effects for free (blending, filtering, shaders); anything beyond the
  original's look is work in the compositor. HD may later need a faster blit path, still on the CPU.
- The asset tables need a harvesting tool and maintenance; original file names enter git (as the
  packs' names did).
- Path dependencies tie the client build to the engine crates' public API: the client CI job also
  runs when those crates change.

## Alternatives considered

- **A GPU renderer (wgpu, a game engine such as Bevy):** exact pixels are harder (sampling,
  blending, driver differences), headless rendering needs an adapter, and an engine brings a scene
  model of its own. Bevy also rests on `winit`, which is Apache-2.0-only.
- **C# (MonoGame) or TypeScript:** would duplicate the decoders and the protocol types or reach
  the engine through the C ABI; WASM keeps a browser client open for Rust.
- **Reading the executable at run time:** nothing harvested in git, but tied to one build's
  addresses, no per-expansion split, and the fixes would still need a committed file.
- **Converting the assets once into PNG atlases:** a slow first start and a second copy of the
  truth that can drift from the archives.
- **Loading at checkpoints from memory-mapped archives** (no engine change; `unsafe`, a crash
  instead of an error when a file changes, implicit page-fault I/O, no mapping in a browser) or
  **from archives read whole at start** (212 MiB read and held for the session, no path to the
  629 MiB video archive): weighed at the M0 checkpoint, the table-only reader chosen.
