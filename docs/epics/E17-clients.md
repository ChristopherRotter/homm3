# E17 — Clients and asset pipeline

Status: **in progress**: M0 done (2026-10-12); M1 done (2026-10-13: T1, T2, T3 and T6 on 2026-10-12, T5, T4, T7 and T8 on 2026-10-13); M2 next. Refined 2026-10-12 (E17a, the asset notes,
done the same day). Decisions: [ADR-0020](../adr/0020-clients.md) (accepted at the M0 checkpoint,
point 7 rewritten there). Working detail (code readings, formats, layouts): `docs/epics/E17-design.md`,
created by the first task that has detail to record; done tasks in full:
[`E17-log.md`](E17-log.md). Lane: client (`docs/lanes/client/HANDOFF.md`; the user's asset notes
are there). Next: M2 T1 (the capture harness).

## Goal

Clients that show the engine's games with the original's own assets, truthful to the original
down to the pixel and the frame: first a battle replay viewer, later a full game record/replay
viewer and playable clients. Everything with graphics, sound, input or a window lives in
`clients/`, outside the core.

## Decisions (user, 2026-10-12; 2026-10-13 where dated)

- **Rust**, confirmed; the `clients/` workspace as Phase 0 laid it out.
- **A software compositor** into a frame in the original's screen format; render-to-PNG writes
  that same frame. HD is planned for (frame size and layouts are data) but comes later.
- **The window, input and audio library is chosen at M2 after an evaluation**, on technical
  grounds only. `winit` and `cpal` are Apache-2.0-only, which ADR-0009 excludes; if they win the
  evaluation, the client may move to GPLv3 (its own `deny.toml`), the user deciding at M2.
- **Asset tables per expansion, committed** (`clients/assets/{base,ab,sod}.toml`); no reading of
  the executable at run time.
- **Reference captures:** automatic (2026-10-13: the user cannot take screenshots for exhaustive
  diffs): the agent runs a copy of the original from `original-data/` unattended in the cloud
  sandbox under Wine and dumps the 16-bit screen (`clients/capture/`, design §6); this replaces
  the earlier plan of computer use on the user's machine. Never from the installation: running
  the game writes into its folder (saves, settings, DDrawCompat's log).
- **M1's exit through the automated diff** (2026-10-13): the stills are compared with the
  original by the first automated diff, which passes or lists its differences, not by the user's
  eye.
- **First milestone: M0 + M1** as below: headless, independent of the battle lane, starting now.
- **Loading at checkpoints** (M0 checkpoint): at start only the archives' directories; at each
  checkpoint (the client opens, a game starts, a battle starts, …) one batch of every asset the
  coming screens need, read by offset; nothing loads while a screen is shown. The bytes come from a
  table-only reader in `h3-formats` (CLT-Q2, the battle lane), chosen over memory-mapped files and
  whole-archive reads; until it lands the adapter reads whole archives (ADR-0020 point 7).
- **Text through a string catalogue** (2026-10-13): every string the client shows is an entry of
  a key → string catalogue (nested JSON or similar), easy to translate; language packs swap. The
  original's text stays uncommitted (ADR-0009): its language comes from the user's own files
  through a committed catalogue of keys. Refined with M3's fonts (QUEUE inbox: the agent's notes).
- **Graphics options** (2026-10-13), eventually: per effect the original's way or a modern one
  (shadows first); `faithful` is the default and the reference. The draw list names what is
  drawn, the compositor decides how (QUEUE inbox).
- **The natural obstacles' types first** (2026-10-13, CLT-Q3): the engine dropped them at
  generation; a `cross-lane` change carries them to the wire before T7 draws obstacles.

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
- **Exit:** the job green on a pull request (while CI cannot run: `clients/scripts/check.sh`
  locally); the ADR draft accepted (numbered by a shared change).
- **Done 2026-10-12** ([log](E17-log.md#m0--the-workspace-2026-10-12)): the workspace, the gates
  (`clients/scripts/check.sh`, the job `clients.yml`), `h3-view gate` (the user's four LODs: 11 981
  entries, opened in about 0.2 s); ADR-0020 accepted; CLT-Q2 filed.

### M1 — assets and a battle still

- Code readings: archive lookup and precedence, the screen pixel format, the keyed colours and
  the shadow operation (where shadows overlap too), the battle screen's geometry and draw order.
- Decoders: palettes, PCX (8- and 24-bit), DEF (all frame formats, groups, offsets), MSK.
- The real-asset gate: every entry of the four LODs decodes; a report of duplicates, unreferenced
  entries and odd names; load time.
- The asset tables' format and their first content: creatures, battlefields and obstacles.
- The compositor and `h3-view render`: a PNG of a battle still (the background, obstacles, stacks
  standing on their hexes, facing correctly) from a record's snapshot.
- **Exit:** the gate passes on the user's files; the still renders; the first automated diff
  against the original passes or lists its differences as M2/M3 items (the user, 2026-10-13).
- **T2 + T3 done 2026-10-12** ([log](E17-log.md#m1-t2--t3--the-decoders-and-the-inventory-2026-10-12),
  [design §1–2](E17-design.md)): the PAL, PCX, DEF and MSK decoders; the gate decodes every entry
  of the user's four LODs (none fails) and writes the inventory report.
- **T1 done 2026-10-12** ([log](E17-log.md#m1-t1--code-reading-archives-screen-format-keyed-colours-2026-10-12),
  [design §3](E17-design.md)): the base archives win; the screen is 16-bit, palettes truncated to
  it; shadows halve or quarter the 16-bit pixel below, compounding; runs of index ≥ 8 skipped.
- **T6 done 2026-10-12** ([log](E17-log.md#m1-t6--the-compositor-2026-10-12)): `h3-render` draws
  decoded frames as the executable's drawers do (design §3.3) into a 16-bit screen and encodes it
  as PNG; `h3-view sprite` shows it on the user's files.
- **T5 done 2026-10-13** ([log](E17-log.md#m1-t5--code-reading-the-battle-screen-2026-10-13),
  [design §4](E17-design.md)): the hex boxes, a stack's canvas by facing (a two-hex stack anchored
  at its rear hex), the count box, background and obstacles, the heroes, the draw order; VCMI
  agrees except for one obstacle's height and the order inside a row.
- **T4 done 2026-10-13** ([log](E17-log.md#m1-t4--the-asset-tables-2026-10-13),
  [design §5](E17-design.md)): `clients/assets/{base,ab,sod}.toml` (144 creatures, the
  battlefields, 125 obstacle types), `h3-view harvest`, the pure loader `h3_view::tables`; every
  pack key has an entry but water's and rock's backgrounds (none in the original); the gate finds
  and decodes all 307 names.
- **T7 done 2026-10-13** ([log](E17-log.md#m1-t7--a-battle-still-2026-10-13)): `h3-view render`
  draws a battle still from a record's first snapshot, or from a scenario file run in-process
  as a spectator of `h3-protocol`'s session: the background, the obstacles of both layers and the
  absolute one, every living stack standing on its hex (two-hex stacks from their rear hex), the
  active stack's outline; the layout is data (`clients/layouts/battle.toml`); three stills from
  the user's files (`clients/scenarios/`).
- **T8 done 2026-10-13** ([log](E17-log.md#m1-t8--m1s-closure-2026-10-13), [design §6](E17-design.md)):
  a fresh reviewer's 13 findings (two bugs fixed: the obstacle removal rule, the DEF frame's
  memory); the review list accepted; the original captured automatically under Wine in the
  sandbox (`clients/capture/`); the smoke battle's background, obstacles and stacks match it pixel
  for pixel; the differences (heroes, war machines, count boxes, the panel, moving stacks, the
  outline unchecked) are M2/M3 items. **M1 done.**

### M2 — the battle replay viewer

- The capture harness (first, the user's requirement for exhaustive diffs): situations as
  generated maps, human-first openings, the scenario rebuilt from the capture, masks for what is
  not drawn yet, a situation set with recorded differences, unattended in the sandbox.
- The window, input and audio evaluation and the user's choice (and licence step, if any).
- The scene model: messages to animations at the original's timing (`CRANIM.TXT`, the
  executable), moves (CLT-Q1, the walk's path), strikes, shots and projectiles, deaths, waits and
  defends; a frame at any time, headless.
- The window adapter playing a record at the original's frame rate.
- **Exit:** a record of a full battle plays in the window and renders headless frame by frame.

### M3 — fidelity

- A representative situation set over the harness (M1 T8 built the capture, M2 the harness):
  every battlefield, the absolute obstacles, sieges later, animation sequences.
- Fonts, the string catalogue and the language packs, and the battle interface (panel, log, stack
  counts, cursors).
- Sound effects and music.
- **Exit:** the reference set passes, or each difference is a listed fix or a classified quirk.

### Later (refined when they come up)

Spell effects, live play against a server (the content check, accounts; the server lane's inbox
notes), HD layouts, graphics options beyond `faithful`, the adventure map and the full game
viewer, video.

## Research items from the asset notes

| Item | Where |
|---|---|
| 1. Archive precedence (SoD against AB), name lookup | M1 T1 (done: design §3.1) |
| 2. The off-colour pixels; the screen format | M1 T1 (done: design §3.2) |
| 3. Licences of MMArchive and homm3-unpacker before reading their code | stop the line: a question first, when a task wants them |
| 4. The video archives | later |
| 5. Capturing the original | M1 T8 (done: Wine in the sandbox, design §6); the harness M2 |
| 6. Per-expansion content and the ruleset profiles | M1 T4 |
| 7. The HD patch as a source of fixes and for HD | M3, later |
| 8. Shadows and the other keyed colours | M1 T1 (done: design §3.3) |
| 9. The adventure map's object draw order (the user: neither they nor VCMI got it exactly right) | a code reading when the adventure map viewer is refined; M1 T5 reads only the battle's |

## Risks and known gaps

- **The walk's path (CLT-Q1).** `Moved` carries only the start and end hex; the walking animation
  needs every hex. Needed by M2.
- **Graphics per expansion.** The Complete edition holds only SoD-era graphics: a RoE ruleset
  still shows SoD art.
- **Engine churn.** `h3-protocol` depends on `h3-battle`; the engine's frequent changes rebuild the
  client and may break it. The client CI job runs when those crates change (since T7 the client
  builds `h3-protocol`, `h3-pack-battle` and their engine dependencies).
- **The sandbox needs the user's computer online** for the original files (staged one LOD per
  call; the captures also need the five DLLs, the sound archives and `H3ab_ahd.vid`).
- **Wine is not Windows.** The captures run on Wine's DirectDraw; the exact match of the menu and
  the battlefield says it changes nothing there, but a difference only Wine could cause (timing,
  palette effects) would need a check on Windows.
- **Checkpoint loading needs an engine change (CLT-Q2).** Until the battle lane's table-only
  reader lands, the client reads whole archives (212 MiB, about 0.2 s warm).

## M1 review list

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

## M2 review list

(Filled by the tasks: choices queued for the milestone review under the autonomy contract.)

- The capture tooling (T8): Wine, Xvfb, xdotool, ImageMagick and mingw-w64 as sandbox tools;
  Python with numpy for `h3cap.py` (as `scripts/exe.py` uses Python); our own `xdd.dll` forwarder.
