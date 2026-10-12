# E17 — Clients and asset pipeline

Status: **refined** (2026-10-12; E17a, the asset notes, done the same day). Decisions: ADR draft
[`client.md`](../lanes/client/adr-drafts/client.md) (Proposed; accepted at the M0 checkpoint).
Working detail (code readings, formats, layouts): `docs/epics/E17-design.md`, created by the first
task that has detail to record; done tasks in full: `docs/epics/E17-log.md`. Lane: client
(`docs/lanes/client/HANDOFF.md`; the user's asset notes are there). Next: M0 T1.

## Goal

Clients that show the engine's games with the original's own assets, truthful to the original
down to the pixel and the frame: first a battle replay viewer, later a full game record/replay
viewer and playable clients. Everything with graphics, sound, input or a window lives in
`clients/`, outside the core.

## Decisions (user, 2026-10-12)

- **Rust**, confirmed; the `clients/` workspace as Phase 0 laid it out.
- **A software compositor** into a frame in the original's screen format; render-to-PNG writes
  that same frame. HD is planned for (frame size and layouts are data) but comes later.
- **The window, input and audio library is chosen at M2 after an evaluation**, on technical
  grounds only. `winit` and `cpal` are Apache-2.0-only, which ADR-0009 excludes; if they win the
  evaluation, the client may move to GPLv3 (its own `deny.toml`), the user deciding at M2.
- **Asset tables per expansion, committed** (`clients/assets/{base,ab,sod}.toml`); no reading of
  the executable at run time.
- **Reference captures:** an agent may run a full copy of the original game from `original-data/`
  on the user's machine through computer use. Never from the installation: running the game writes
  into its folder (saves, settings, DDrawCompat's log).
- **First milestone: M0 + M1** as below: headless, independent of the battle lane, starting now.

## The ideas in plain words

- **Indexed sprites, one frame.** The archives hold sprites as 8-bit indices into a 256-colour
  palette; some indices are not colours but instructions (transparent, shadow, selection outline,
  player colour). The compositor draws the sprites into one frame the way the executable does,
  then hands the frame to the window or to a PNG file. What an agent sees in a PNG is what a
  player sees.
- **Asset tables.** The engine names things by key (`base:pikeman`); the archives by file name
  (`CPKMAN.DEF`). A table per expansion connects them, with the animation groups, the offsets and
  the fixes. The tables are generated once from the executable, then edited as source: a fix is a
  visible, reviewed diff.
- **Real-asset gate.** CI never sees an original file. A local gate decodes every entry of the
  user's archives, checks every name in the tables, and measures how long loading takes.
- **Record.** The server's record is one viewer's message log. The viewer plays records; a live
  game is the same messages arriving over a socket. An engine replay becomes a record by running
  the protocol's session in-process.
- **Reference frames.** The original, run from a copy, shows a situation; its frame is captured
  and compared with ours exactly. Differences become either fixes in our code or entries in the
  tables, each with its source.

## Milestones

Each milestone ends at a checkpoint with the user, after a fresh reviewer agent checked it. The
autonomy contract (lane HANDOFF) says what the agent decides alone in between.

### M0 — the workspace

- `clients/` builds as its own workspace with `h3-assets`, `h3-render` and `h3-view`, path
  dependencies on the engine crates working, the engine's lint settings, `deny` with the engine's
  allowlist.
- The client CI job (a shared change): fmt, clippy, tests, coverage of the pure crates, deny.
- A local real-asset gate command that finds `original-data/install/`.
- **Exit:** the job green on a pull request; the ADR draft accepted (numbered by a shared change).

### M1 — assets and a battle still

- Code readings: archive lookup and precedence, the screen pixel format, the keyed colours and
  the shadow operation (where shadows overlap too), the battle screen's geometry and draw order.
- Decoders: palettes, PCX (8- and 24-bit), DEF (all frame formats, groups, offsets), MSK.
- The real-asset gate: every entry of the four LODs decodes; a report of duplicates, unreferenced
  entries and odd names; load time.
- The asset tables' format and their first content: creatures, battlefields and obstacles.
- The compositor and `h3-view render`: a PNG of a battle still (the background, obstacles, stacks
  standing on their hexes, facing correctly) from a record's snapshot.
- **Exit:** the gate passes on the user's files; the still renders; the user compares it with the
  original and agrees (or the differences are listed as M2/M3 items).

### M2 — the battle replay viewer

- The window, input and audio evaluation and the user's choice (and licence step, if any).
- The scene model: messages to animations at the original's timing (`CRANIM.TXT`, the
  executable), moves (CLT-Q1, the walk's path), strikes, shots and projectiles, deaths, waits and
  defends; a frame at any time, headless.
- The window adapter playing a record at the original's frame rate.
- **Exit:** a record of a full battle plays in the window and renders headless frame by frame.

### M3 — fidelity

- The reference capture setup (a full copy of the game under `original-data/`, computer use, the
  exact capture method), a representative situation set, the exact diff harness.
- Fonts and the battle interface (panel, log, stack counts, cursors).
- Sound effects and music.
- **Exit:** the reference set passes, or each difference is a listed fix or a classified quirk.

### Later (refined when they come up)

Spell effects, live play against a server (the content check, accounts; the server lane's inbox
notes), HD layouts, the adventure map and the full game viewer, video.

## Research items from the asset notes

| Item | Where |
|---|---|
| 1. Archive precedence (SoD against AB), name lookup | M1 T1 |
| 2. The off-colour pixels; the screen format | M1 T1 |
| 3. Licences of MMArchive and homm3-unpacker before reading their code | stop the line: a question first, when a task wants them |
| 4. The video archives | later |
| 5. Capturing the original | M3 (agreed: a copy under `original-data/`, computer use) |
| 6. Per-expansion content and the ruleset profiles | M1 T4 |
| 7. The HD patch as a source of fixes and for HD | M3, later |
| 8. Shadows and the other keyed colours | M1 T1 |

## Risks and known gaps

- **The walk's path (CLT-Q1).** `Moved` carries only the start and end hex; the walking animation
  needs every hex. Needed by M2.
- **Graphics per expansion.** The Complete edition holds only SoD-era graphics: a RoE ruleset
  still shows SoD art.
- **Engine churn.** `h3-protocol` depends on `h3-battle`; the engine's frequent changes rebuild the
  client and may break it. The client CI job runs when those crates change.
- **The sandbox needs the user's computer online** for the original files (staged one LOD per
  call).

## M1 review list

(Filled by the tasks: choices queued for the milestone review under the autonomy contract.)
