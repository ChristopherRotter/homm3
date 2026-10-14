# E17 — Clients and asset pipeline

Status: **in progress**: M0 done (2026-10-12); M1 done (2026-10-13: T1, T2, T3 and T6 on 2026-10-12, T5, T4, T7 and T8 on 2026-10-13); M2 done (2026-10-14: T1, T3, T4a, T6 and T4b-1 on 2026-10-13, T4b-2a, T2, T5a, T5b and T5c on 2026-10-14); M3 refined (T5c), next. Refined 2026-10-12 (E17a, the asset notes,
done the same day). Decisions: [ADR-0020](../adr/0020-clients.md) (accepted at the M0 checkpoint,
point 7 rewritten there). Working detail (code readings, formats, layouts): `docs/epics/E17-design.md`,
created by the first task that has detail to record; done tasks in full:
[`E17-log.md`](E17-log.md). Lane: client (`docs/lanes/client/HANDOFF.md`; the user's asset notes
are there). Next: M3 T1a (`docs/lanes/client/QUEUE.md`).

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
  **Chosen 2026-10-14 (T2): SDL 3** through `sdl3-sys`, built from source and linked statically;
  no licence step ([ADR-0024](../adr/0024-client-window-audio.md), design §8).
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
  executable), moves (a walk hex by hex: CLT-Q1, built 2026-10-13, wire v29), strikes, shots
  and projectiles, deaths, waits and defends; a frame at any time, headless.
- The window adapter playing a record at the original's frame rate.
- **Exit:** a record of a full battle plays in the window and renders headless frame by frame.
- **T1 done 2026-10-13** ([log](E17-log.md#m2-t1--the-capture-harness-2026-10-13),
  [design §6.1, §6.2](E17-design.md), runbook `clients/capture/README.md`): our `xdd.dll` takes
  over the original's only clock import, so the harness freezes and steps the game's time by
  milliseconds; a driver by fixed coordinates and probe pixels; `h3-view situation`; the capture
  set (`clients/situations/set.toml`): every land terrain, human-first openings, two-hex stacks
  facing either way, usual and absolute obstacles, all matching outside the masks but for the
  outline's colour (its pulse, T3/T4) and an idle animation; reproducible from a fresh prefix. The
  scene takes a terrain's third background (0x41ADB0 read; CLT-Q4).
- **T3 done 2026-10-13** ([log](E17-log.md#m2-t3--code-reading-the-battles-animation-timing-2026-10-13),
  [design §7](E17-design.md)): the frame deadline and the idle step, the combat speed's factors,
  walks, flights, turns, strikes and their reactions, shots and projectiles, fidgets, each with its
  address and confirmed in the original to the millisecond: the harness reads the game's battle
  state from its memory (`capture/h3mem.py`) and replays recorded timelines
  (`situations/sequences.toml`). CLT-Q1 decided (a) by the user.
- **CLT-Q1 built 2026-10-13** (the user's request, `cross-lane`, #169): the engine reports a
  walk hex by hex, the wire is v29 (`OPEN-QUESTIONS.md`, answered).
- **T4a done 2026-10-13** ([log](E17-log.md#m2-t4a--the-scene-over-time-the-timeline-2026-10-13),
  [design §7.8](E17-design.md)): `h3_view::anim` plays a record's commands at the original's
  timing; `h3-view sequences` checks the viewer's timeline against the three recorded sequences:
  all three as recorded, every line; `render --at <ms>`.
- **T6 done 2026-10-13** (the user's high-priority request; [log](E17-log.md#m2-t6--experiments-on-the-adventure-map-2026-10-13),
  [design §6.3](E17-design.md)): the harness plays the domain lanes' experiment batches on the
  adventure map (`capture/h3adv.py batch`): hot seat, heroes from the hero list, a tile clicked
  twice, turns, each record read from the game's memory into the batch's results template. E08
  batch 2: D3a–D3d as the user's results, every field; D5's piles within their ranges.
- **T4b-1 done 2026-10-13** ([log](E17-log.md#m2-t4b-1--the-frame-check-2026-10-13),
  [design §7.9](E17-design.md)): the frame check (`capture/h3drive.py frames`): every distinct
  battlefield of the recorded sequences in the original against the viewer's frame at the same
  time; walk and shot exact in every frame (the arrow at every step), melee 68 of 75 (the rest
  listed); the projectile, the outline's pulse and the idle steps built; the shooter's strike
  loop and the flight's points corrected.
- **T4b-2a done 2026-10-14** ([log](E17-log.md#m2-t4b-2a--a-full-battle-frame-by-frame-2026-10-14),
  [design §7.10](E17-design.md)): M2's headless exit: `h3-view play` has a bot fight a scenario to
  its end and draws every frame of it headless (`scenarios/battle.toml`: 24 commands, about 1 100
  frames in 3 s); turns read in the executable and built (walks, flights, both sides of a melee,
  shooters, turning back; a turned two-hex stack on its other hex), the draw layers (walk 3/7,
  striker 6, struck 5, corpses 1), fidgets from the viewer's own generator, a flight's two
  standing frames. The three recorded sequences unchanged; a fourth, `turn` (a two-hex walk's
  turn and turn back), recorded in the original: its timeline as recorded and all 39 frames
  exact. A strike's and a shot's turns are read and unit-tested only (no opening sets them up).

- **T2 done 2026-10-14** ([log](E17-log.md#m2-t2--the-window-input-and-audio-evaluation-2026-10-14),
  [design §8](E17-design.md)): three spikes (SDL 3, `minifb` and `miniquad` with `tinyaudio`) on
  the throwaway branch `client/e17-m2-t2-spikes`, measured in the sandbox and on the user's
  Windows machine: all exact at integer scales; SDL 3 alone with clean sound (the others
  crackled), a hardware cursor, fullscreen and DPI awareness. The user chose SDL 3 (`sdl3-sys`,
  static); MP3 by `nanomp3`; the game's IMA ADPCM sounds decoded in `h3-assets`.
- **T5a done 2026-10-14** ([log](E17-log.md#m2-t5a--a-battle-in-a-window-2026-10-14),
  [design §8.1](E17-design.md)): M2's exit: the platform adapter `h3-sdl` (every SDL call) and
  `h3-view window`, which shows a scenario's or a record's battle at the original's frame times,
  at an integer scale, with the game's cursor (hardware or drawn), pause, steps and restart;
  `--check` reads every frame back: under Xvfb all exact. T5 split in three: T5b the sound, T5c
  M2's closure.
- **T5b done 2026-10-14** ([log](E17-log.md#m2-t5b--the-sound-2026-10-14),
  [design §8.2](E17-design.md)): the sound archives decoded (`h3_assets::snd`, `wav`: every
  complete frame up to the `fact` chunk's count; all 1 232 entries equal SDL's samples, a gate
  check); when the battle's sounds play, read in the executable (the start sound while the
  battle's clock waits, the music looping and fading in, each stack's eight sounds with their
  groups); the timeline's cues and the asset tables' `sounds`; `h3-view window` plays the opening,
  the music and the creatures' sounds (`nanomp3`; keys M and E, `--mute`).
- **T5c done 2026-10-14** ([log](E17-log.md#m2-t5c--m2s-closure-2026-10-14)): M2's exit checked
  on master; a fresh reviewer's 12 findings, each checked (10 fixed: the next turn after the
  turn-backs, layer changes at the next idle step, a created stack's strike, a breath's groups
  17–19, the window's step, redraw and cursor, the fade-in, an undecodable sound; one is the
  original's behaviour, pinned by a test); the review list accepted by the user (all 31 items);
  M3 refined with the user (QUEUE: T1a–T9; T4b-2b became M3 T1a and T1b; the rays and the spell
  animations added at the user's word). **M2 done.**

### M3 — fidelity

Refined at M2's closure (T5c) with the user; the tasks in `docs/lanes/client/QUEUE.md`.

- The scene over time completed (T1a, T1b; M2's T4b-2b): turns, the walk-by, deaths, defending,
  shots up and down, Haste and Slow, the **rays** (the user: Magi and Beholders draw an energy
  ray, not a projectile), and recordings that check the provisional rules (the flier's layer, the
  shot's loop, the breath's groups).
- A representative situation set over the harness: every battlefield (T2), the heroes and war
  machines (T3), the count boxes (T5) and the panel (T6), each mask removed as it is drawn;
  sieges later.
- Fonts, the string catalogue and the language packs (T4, an ADR-0020 amendment), and the battle
  interface (T5, T6: the panel, the log, the cursors).
- The **spell animations** (the user, at T5c: their plethora; T7a the code reading, T7b on
  stacks with morale and luck, T7c area spells, rays and spell obstacles) and their sounds.
- The sound's remainder (T8: the battle's end; the volume settings later).
- **Exit:** the reference set passes, or each difference is a listed fix or a classified quirk.
- **T1a done 2026-10-14** ([log](E17-log.md#m3-t1a--turns-and-the-walk-by-2026-10-14),
  [design §7.10, §7.11](E17-design.md)): a battle in progress in the original (hot seat, the
  harness plays both sides and acts as each turn starts); `climb` and `behind` recorded: a walk's
  later flip, a melee whose striker and target both turn, a shot whose shooter turns; both
  timelines as recorded on the first try, frames 43 of 43 and 164 of 194 exact. The walk-by
  difference read: the original redraws by dirty rectangles (shadows drawn again, the saved
  screen's count boxes); the model queued as **T1c**.

### Later (refined when they come up)

Sieges (walls, towers, the Catapult, the moat), live play against a server (the content check,
accounts; the server lane's inbox notes), HD layouts, graphics options beyond `faithful`, the
client's options (volumes), the adventure map and the full game viewer, video.

## Research items from the asset notes

| Item | Where |
|---|---|
| 1. Archive precedence (SoD against AB), name lookup | M1 T1 (done: design §3.1) |
| 2. The off-colour pixels; the screen format | M1 T1 (done: design §3.2) |
| 3. Licences of MMArchive and homm3-unpacker before reading their code | stop the line: a question first, when a task wants them |
| 4. The video archives | later |
| 5. Capturing the original | M1 T8 (done: Wine in the sandbox, design §6); M2 T1 (done: the harness, §6.1) |
| 6. Per-expansion content and the ruleset profiles | M1 T4 |
| 7. The HD patch as a source of fixes and for HD | M3, later |
| 8. Shadows and the other keyed colours | M1 T1 (done: design §3.3) |
| 9. The adventure map's object draw order (the user: neither they nor VCMI got it exactly right) | a code reading when the adventure map viewer is refined; M1 T5 reads only the battle's |

## Risks and known gaps

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
- **A record does not say who acts** (T4b-1): the original ticks its idle steps and outlines the
  active stack differently on a player's turn and the computer's; the viewer guesses from the
  commands' times (design §7.9). A record of a live game may need the seats' kinds.
- **The background variant (CLT-Q4).** The original picks a battlefield's picture from the
  adventure map around the battle; a battle's record does not carry it, so the viewer shows the
  picture of an empty surrounding.
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
  only; a recorded flight across rows (T4b-2b) checks it against the original's frames.
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

## M3 review list

(Filled by the tasks: choices queued for the milestone review under the autonomy contract.)

- T1a: a battle in progress is played by the harness in hot seat (`h3-view situation
  --hot-seat`), both sides human, rather than against the original's AI: every command chosen,
  no computer's turn (*provisional*); the frame check compares the last screen of a millisecond
  (a hot-seat wait passes the turn at once: the outline on the stack that waited is not checked)
  and places the count boxes' masks by the drawn pose's facing.

- The reviewer's fixes (M2 T5c): a command's last layer changes show at the first idle step after
  it (*provisional*, as after the last command); a breath told by its second stack standing beyond
  the striker's neighbours (*provisional*: the record does not name the ability), the asset
  tables' `breath = [17, 18, 19]`; a strike or shot by a stack the timeline does not know plays
  its targets' reactions alone; `Timeline::still`, which ends the window's look-ahead.
